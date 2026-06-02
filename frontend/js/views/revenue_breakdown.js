// Revenue Breakdown — Finnhub /stock/revenue-breakdown + /revenue-breakdown2.
// Segment + geographic revenue breakdown over time. Spots secular shifts
// (e.g., AAPL services growth, MSFT cloud revenue).

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

let state = { symbol: '' };

export async function renderRevenueBreakdown(mount, _appState, symbol = '') {
    const tok = currentViewToken();
    if (symbol) state.symbol = symbol.toUpperCase();
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.rev_break.h1.title">// REVENUE BREAKDOWN</span></h1>
        <p class="muted small" data-i18n="view.rev_break.hint.intro">
            Segment + geographic revenue mix over time. Secular shifts (services vs hardware,
            cloud vs on-prem, US vs international) often precede multi-year re-ratings.
        </p>
        <div class="chart-panel">
            <form class="inline-form" id="rb-form">
                <label><span data-i18n="view.rev_break.label.symbol">Symbol</span>
                    <input type="text" name="symbol" value="${esc(state.symbol)}" placeholder="AAPL" required></label>
                <button class="primary" type="submit" data-i18n="view.rev_break.btn.load">Load</button>
            </form>
        </div>
        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.rev_break.h2.segments">Segment breakdown</h2>
                <div id="rb-segments"></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.rev_break.h2.geo">Geographic breakdown</h2>
                <div id="rb-geo"></div>
            </div>
        </div>
    `;
    document.getElementById('rb-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.symbol = (fd.get('symbol') || '').toUpperCase().trim();
        void load(tok);
    });
    if (state.symbol) await load(tok);
}

async function load(tok) {
    const segEl = document.getElementById('rb-segments');
    const geoEl = document.getElementById('rb-geo');
    [segEl, geoEl].forEach(el => el && (el.innerHTML = `<div class="boot">${esc(t('common.loading'))}</div>`));
    try {
        const [v1, v2] = await Promise.all([
            api.symbolRevenueBreakdown(state.symbol).catch(() => null),
            api.symbolRevenueBreakdown2(state.symbol).catch(() => null),
        ]);
        if (!viewIsCurrent(tok)) return;
        // Prefer v2 (segment) for segments, v1 for geo when split available.
        const segments = (v2?.data || v1?.data || []).filter(d =>
            !/^geograph|country|region|united states|asia|emea|americas/i.test(d.label || ''));
        const geo = (v1?.data || v2?.data || []).filter(d =>
            /^geograph|country|region|united states|asia|emea|americas|china|europe|japan/i.test(d.label || ''));
        renderBreakdown(segEl, segments, 'view.rev_break.empty.segments');
        renderBreakdown(geoEl, geo, 'view.rev_break.empty.geo');
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        showToast(t('view.rev_break.toast.failed'), { level: 'error' });
    }
}

function renderBreakdown(el, rows, emptyKey) {
    if (!el) return;
    if (!rows.length) {
        el.innerHTML = `<p class="muted">${esc(t(emptyKey))}</p>`;
        return;
    }
    // Group rows by period; each row is {period, segment/label, revenue, ...}.
    const byPeriod = new Map();
    for (const r of rows) {
        const period = r.period || r.year || '—';
        if (!byPeriod.has(period)) byPeriod.set(period, []);
        byPeriod.get(period).push(r);
    }
    const periods = [...byPeriod.keys()].sort().slice(-4);  // last 4 periods
    const segments = [...new Set(rows.map(r => r.label || r.segment || r.businessSegment || '—'))];
    el.innerHTML = `<table class="trades">
        <thead><tr>
            <th data-i18n="view.rev_break.th.segment">Segment</th>
            ${periods.map(p => `<th>${esc(p)}</th>`).join('')}
        </tr></thead>
        <tbody>${segments.map(seg => `
            <tr>
                <td>${esc(seg)}</td>
                ${periods.map(p => {
                    const row = (byPeriod.get(p) || []).find(r => (r.label || r.segment || r.businessSegment) === seg);
                    return `<td>${row ? fmt(row.revenue ?? row.value ?? row.amount) : '—'}</td>`;
                }).join('')}
            </tr>
        `).join('')}</tbody>
    </table>`;
}

function fmt(v) {
    if (v == null || !Number.isFinite(Number(v))) return '—';
    const n = Number(v);
    if (Math.abs(n) >= 1e9) return '$' + (n / 1e9).toFixed(2) + 'B';
    if (Math.abs(n) >= 1e6) return '$' + (n / 1e6).toFixed(1) + 'M';
    if (Math.abs(n) >= 1e3) return '$' + (n / 1e3).toFixed(0) + 'k';
    return '$' + n.toFixed(0);
}
