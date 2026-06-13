// Dividend Calendar — an investing.com-style, market-wide calendar of
// *upcoming* ex-dividend events.
//
// Data source: the backend `/dividends/calendar?days=N` endpoint, served from
// a background-refreshed cache (see market_data::refresh_dividends_calendar).
// The full 90-day Nasdaq feed is precomputed on a 6h interval and the soonest
// names' yields (annualized ÷ price) are enriched server-side, so opening this
// view never triggers a per-date fan-out or a client-side quote sweep — it
// just renders the prepopulated window. Events are grouped by ex-date so the
// table reads like a day-by-day calendar; names without an enriched yield
// show "—".

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { searchScore, getMatchIndices, highlightWithIndices } from '../fzf.js';
import {
    daysBetween, fmtDate, fmtYield, fmtAmount,
    extractCalendarRows, freqLabelFromPpy, fmtAnnualized,
} from '../_dividend_calendar_inputs.js';

const HORIZON_OPTIONS = [
    { value: 7,  n: 7 },
    { value: 14, n: 14 },
    { value: 30, n: 30 },
    { value: 60, n: 60 },
    { value: 90, n: 90 },
];
function horizonLabel(o) {
    return t('view.dividend_calendar.horizon.next_n_days', { n: o.n });
}

let state = {
    horizon: 14,
    filter: '',
    rows: null,
    sortKey: 'ex_date',
    sortDir: 'asc',
};

export async function renderDividendCalendar(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 data-i18n="view.dividend_calendar.h1.dividend_yield_calendar" class="view-title">// DIVIDEND CALENDAR</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.dividend_calendar.h2.calendar">Calendar</h2>
            <div class="inline-form">
                <label><span data-i18n="view.dividend_calendar.label.horizon">Horizon</span>
                    <select id="dc-horizon" data-tip="view.dividend_calendar.tip.horizon">
                        ${HORIZON_OPTIONS.map(o =>
                            `<option value="${o.value}" ${o.value === state.horizon ? 'selected' : ''}>${esc(horizonLabel(o))}</option>`
                        ).join('')}
                    </select></label>
                <label><span data-i18n="view.dividend_calendar.label.filter">Filter</span>
                    <input id="dc-filter" type="text" placeholder="symbol / company"
                        data-i18n-placeholder="view.dividend_calendar.placeholder.filter"
                        value="${esc(state.filter)}" data-tip="view.dividend_calendar.tip.filter"></label>
            </div>
            <p data-i18n="view.dividend_calendar.hint.finnhub" class="muted">
                Market-wide upcoming ex-dividend dates across all listed companies, sourced from the
                Nasdaq dividend feed. Yield (annualized ÷ price) is filled best-effort for the soonest names.
            </p>
        </div>

        <div id="dc-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.dividend_calendar.h2.upcoming_dividends">Upcoming dividends</h2>
            <div id="dc-table"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.dividend_calendar.h2.yield_chart">Dividend yield by symbol</h2>
            <div id="dc-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.dividend_calendar.h2.dte_chart">Days-to-ex distribution (when are dividends clustering)</h2>
            <div id="dc-dte-chart" style="width:100%;height:220px"></div>
        </div>

        <div id="dc-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    wireForm(mount, tok);
    void fmt;
    void runFetch(tok);
}

function wireForm(mount, tok) {
    document.getElementById('dc-horizon').addEventListener('change', e => {
        state.horizon = Number(e.target.value);
        void runFetch(tok);
    });
    document.getElementById('dc-filter').addEventListener('input', e => {
        state.filter = e.target.value;
        if (state.rows) renderAll(state.rows);
    });
}

async function runFetch(tok) {
    hideErr();
    document.getElementById('dc-table').innerHTML =
        `<div class="tv-spinner-wrap"><div class="tv-spinner"></div></div>`;

    let payload;
    try {
        payload = await api.dividendsCalendar(state.horizon);
    } catch (e) {
        showErr(`${t('view.dividend_calendar.error.load')}: ${e.message || e}`);
        showToast(t('view.dividend_calendar.toast.error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;

    const rows = extractCalendarRows(payload);
    rows.sort((a, b) => a.ex_date - b.ex_date || a.symbol.localeCompare(b.symbol));
    state.rows = rows;
    renderAll(rows);
    showToast(t('view.dividend_calendar.toast.loaded', { events: rows.length }), { level: 'success' });
}

function renderAll(rows) {
    renderSummary(rows);
    renderTable(rows);
    renderYieldChart(rows);
    renderDteChart(rows);
}

const NUMERIC_SORT_KEYS = new Set(['amount', 'payments_per_year', 'annualized', 'yield']);

function sortValue(r, key) {
    switch (key) {
        case 'symbol':            return r.symbol || '';
        case 'company':           return r.company || '';
        case 'ex_date':           return r.ex_date ? r.ex_date.getTime() : null;
        case 'pay_date':          return r.pay_date ? r.pay_date.getTime() : null;
        case 'record_date':       return r.record_date ? r.record_date.getTime() : null;
        case 'amount':            return r.amount;
        case 'payments_per_year': return r.payments_per_year;
        case 'annualized':        return r.annualized;
        case 'yield':             return r.yield;
        default:                  return null;
    }
}

function tieBreak(a, b) {
    const at = a.ex_date ? a.ex_date.getTime() : Infinity;
    const bt = b.ex_date ? b.ex_date.getTime() : Infinity;
    return (at - bt) || a.symbol.localeCompare(b.symbol);
}

function isBlank(v) {
    return v == null || (typeof v === 'number' && !Number.isFinite(v));
}

function sortRows(rows, key, dir) {
    const mul = dir === 'asc' ? 1 : -1;
    const isStr = key === 'symbol' || key === 'company';
    return [...rows].sort((a, b) => {
        const av = sortValue(a, key);
        const bv = sortValue(b, key);
        const an = isBlank(av);
        const bn = isBlank(bv);
        // Blanks always sort to the bottom, regardless of direction.
        if (an && bn) return tieBreak(a, b);
        if (an) return 1;
        if (bn) return -1;
        const c = isStr ? String(av).localeCompare(String(bv)) : av - bv;
        return c === 0 ? tieBreak(a, b) : c * mul;
    });
}


function applyFilter(rows) {
    const q = state.filter.trim();
    if (!q) return rows;
    return rows
        .map(r => ({ r, score: searchScore(q, [r.symbol || '', r.company || '']) }))
        .filter(x => x.score > 0)
        .sort((a, b) => b.score - a.score)
        .map(x => x.r);
}

function _dcHighlight(text) {
    const q = state.filter.trim();
    const str = String(text == null ? '' : text);
    if (!q) return esc(str);
    return highlightWithIndices(str, getMatchIndices(q, str));
}

function renderSummary(rows) {
    const visible = applyFilter(rows);
    const payers = new Set(visible.map(r => r.symbol)).size;
    const yields = visible.map(r => r.yield).filter(y => Number.isFinite(y));
    const avgYield = yields.length ? yields.reduce((a, b) => a + b, 0) / yields.length : NaN;
    const maxYield = yields.length ? Math.max(...yields) : NaN;
    const soonest = visible.length ? fmtDate(visible[0].ex_date) : '—';

    document.getElementById('dc-summary').innerHTML = [
        card(t('view.dividend_calendar.card.events_in_horizon', { window: state.horizon + 'd' }), String(visible.length)),
        card(t('view.dividend_calendar.card.dividend_payers_found'), String(payers)),
        card(t('view.dividend_calendar.card.soonest_ex'), soonest),
        card(t('view.dividend_calendar.card.avg_yield_paying_set'), fmtYield(avgYield)),
        card(t('view.dividend_calendar.card.max_yield'), fmtYield(maxYield)),
    ].join('');
}

function card(label, value) {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value">${esc(value)}</div>
    </div>`;
}

function renderTable(rows) {
    const visible = sortRows(applyFilter(rows), state.sortKey, state.sortDir);
    const wrap = document.getElementById('dc-table');
    if (visible.length === 0) {
        wrap.innerHTML = `<div class="boot">${esc(t('view.dividend_calendar.empty.no_upcoming'))}</div>`;
        return;
    }
    const now = new Date();

    const th = (key, labelKey, label) => {
        const active = state.sortKey === key;
        const arrow = active ? (state.sortDir === 'asc' ? ' ▲' : ' ▼') : '';
        return `<th data-sort-key="${esc(key)}" class="sortable${active ? ' active' : ''}"
                   data-i18n="${labelKey}">${esc(label)}${arrow}</th>`;
    };

    const body = visible.map(r => {
        const days = daysBetween(now, r.ex_date);
        const dayTxt = days == null ? '' : (days <= 0 ? t('view.dividend_calendar.today') : `${days}d`);
        return `
            <tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
                <td>${esc(fmtDate(r.ex_date))}${dayTxt ? ` <span class="muted">· ${esc(dayTxt)}</span>` : ''}</td>
                <td><a class="link" href="#research/${encodeURIComponent(r.symbol)}">${_dcHighlight(r.symbol)}</a></td>
                <td class="dc-company">${_dcHighlight(r.company)}</td>
                <td class="dc-amount">${esc(fmtAmount(r.amount))}</td>
                <td>${esc(freqLabelFromPpy(r.payments_per_year))}</td>
                <td class="dc-amount">${esc(fmtAnnualized(r.annualized))}</td>
                <td class="dc-yield">${esc(fmtYield(r.yield))}</td>
                <td>${esc(fmtDate(r.pay_date))}</td>
                <td>${esc(fmtDate(r.record_date))}</td>
            </tr>`;
    }).join('');

    wrap.innerHTML = `
        <table class="trades dc-table">
            <thead><tr>
                ${th('ex_date',           'view.dividend_calendar.th.ex_date',         'Ex-date')}
                ${th('symbol',            'view.dividend_calendar.th.symbol',          'Symbol')}
                ${th('company',           'view.dividend_calendar.th.company',         'Company')}
                ${th('amount',            'view.dividend_calendar.th.amount',          'Amount')}
                ${th('payments_per_year', 'view.dividend_calendar.th.frequency',       'Frequency')}
                ${th('annualized',        'view.dividend_calendar.th.annualized',      'Annualized')}
                ${th('yield',             'view.dividend_calendar.th.indicated_yield', 'Indicated yield')}
                ${th('pay_date',          'view.dividend_calendar.th.pay_date',        'Pay date')}
                ${th('record_date',       'view.dividend_calendar.th.record_date',     'Record date')}
            </tr></thead>
            <tbody>${body}</tbody>
        </table>`;

    wrap.querySelectorAll('th.sortable').forEach(thEl => {
        thEl.addEventListener('click', () => {
            const key = thEl.dataset.sortKey;
            if (state.sortKey === key) {
                state.sortDir = state.sortDir === 'asc' ? 'desc' : 'asc';
            } else {
                state.sortKey = key;
                state.sortDir = NUMERIC_SORT_KEYS.has(key) ? 'desc' : 'asc';
            }
            if (state.rows) renderTable(state.rows);
        });
    });
}

function renderYieldChart(rows) {
    const el = document.getElementById('dc-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const bySym = new Map();
    for (const r of applyFilter(rows)) {
        if (!Number.isFinite(r.yield)) continue;
        const prev = bySym.get(r.symbol);
        if (prev == null || r.yield > prev) bySym.set(r.symbol, r.yield);
    }
    const data = [...bySym.entries()].map(([symbol, y]) => ({ symbol, yield: y }));
    if (data.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.dividend_calendar.empty_chart">${esc(t('view.dividend_calendar.empty_chart'))}</div>`;
        return;
    }
    data.sort((a, b) => b.yield - a.yield);
    const top = data.slice(0, 30);
    const labels = top.map(r => r.symbol);
    const yields = top.map(r => r.yield * 100);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false }, y: { auto: true } },
        series: [
            { label: t('view.dividend_calendar.chart.symbol_idx') },
            { label: t('view.dividend_calendar.chart.yield_pct'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 10, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, yields], el);
}

function renderDteChart(rows) {
    const el = document.getElementById('dc-dte-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const now = new Date();
    const days = applyFilter(rows)
        .map(r => daysBetween(now, r.ex_date))
        .filter(d => Number.isFinite(d) && d >= 0);
    if (days.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.dividend_calendar.empty_dte_chart">${esc(t('view.dividend_calendar.empty_dte_chart'))}</div>`;
        return;
    }
    const buckets = [
        { lo: 0,  hi: 7,   label: '0–7d' },
        { lo: 7,  hi: 14,  label: '7–14d' },
        { lo: 14, hi: 30,  label: '14–30d' },
        { lo: 30, hi: 60,  label: '30–60d' },
        { lo: 60, hi: 90,  label: '60–90d' },
        { lo: 90, hi: Infinity, label: '≥90d' },
    ];
    const counts = new Array(buckets.length).fill(0);
    for (const d of days) {
        for (let i = 0; i < buckets.length; i++) {
            if (d >= buckets[i].lo && d < buckets[i].hi) { counts[i] += 1; break; }
        }
    }
    const labels = buckets.map(b => b.label);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false }, y: { auto: true } },
        series: [
            { label: t('view.dividend_calendar.chart.dte_bucket') },
            { label: t('view.dividend_calendar.chart.payer_count'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 14, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, counts], el);
}

function showErr(msg) {
    const el = document.getElementById('dc-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('dc-err').style.display = 'none'; }
