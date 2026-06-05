// Market Profile (TPO) view — Time-Price Opportunity histogram.
//
// Sierra Chart-class. Each bracket (typically 30-min) contributes one
// letter per price level it visited. Stacking those letters reveals:
//   - POC (Point of Control) — price with the most time
//   - Value Area (VAH/VAL) — 70% of TPOs around POC
//   - Single prints — price levels touched in only one bracket (excess)

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseBracketBlob, validateInputs, buildBody,
    levelTier, levelLetters, tierCounts,
    makeDemoBrackets, fmtN, fmtInt,
} from '../_market_profile_inputs.js';

import { t } from '../i18n.js';
import { showToast } from '../toast.js';
let state = { brackets: '', tickSize: 0.5 };

export async function renderMarketProfile(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.market_profile.h1.market_profile_tpo_histogram" class="view-title">// MARKET PROFILE · TPO HISTOGRAM</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.market_profile.h2.brackets_one_per_time_period_typically_30_min_rth_">Brackets (one per time period — typically 30-min RTH brackets)</h2>
            <p class="muted" data-i18n="view.market_profile.hint.format">One line per bracket: bracket_index high low. Each bracket prints one letter (A, B, C, …) at every quantized price level it traded through. Demo loads a 13-bracket A-M session shaped like a typical normal-day profile.</p>
            <textarea id="mp-brackets" rows="8" placeholder="0 102.5 101.0&#10;1 101.5 100.0&#10;..." data-tip="view.market_profile.tip.brackets"></textarea>
            <div class="inline-form">
                <label><span data-i18n="view.market_profile.label.tick_size">Tick size</span>
                    <input id="mp-tick" type="number" step="0.01" min="0" value="${state.tickSize}" data-tip="view.market_profile.tip.tick"></label>
                <button data-i18n="view.market_profile.btn.load_demo_13_bracket_normal_day" id="mp-demo" class="secondary" type="button" data-tip="view.market_profile.tip.demo" data-shortcut="market_profile_demo">Load demo (13-bracket normal day)</button>
                <button data-i18n="view.market_profile.btn.clear" id="mp-clear" class="secondary" type="button" data-tip="view.market_profile.tip.clear">Clear</button>
                <button data-i18n="view.market_profile.btn.build_tpo" id="mp-run" class="primary" type="button" data-tip="view.market_profile.tip.run" data-shortcut="market_profile_run">Build TPO</button>
            </div>
        </div>

        <div id="mp-errors" class="boot" style="display:none"></div>
        <div id="mp-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.market_profile.h2.tpo_histogram">TPO histogram</h2>
            <div id="mp-tpo" class="mp-tpo"></div>
            <p data-i18n="view.market_profile.hint.yellow_row_poc_most_time_cyan_rows_value_area_70_r" class="muted">Yellow row = POC (most time). Cyan rows = Value Area (70%).
                Red rows = single prints (excess — often retested). Letters are the
                actual TPO brackets that touched each price level.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.market_profile.h2.tpo_chart">TPO count by price level</h2>
            <div id="mp-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.market_profile.h2.cum_tpo_chart">Cumulative TPO % by price (low → high)</h2>
            <div id="mp-cum-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.market_profile.hint.cum_tpo" class="muted small">Cumulative share of TPOs walking from low to high. Yellow dashed at 15% / 85% = VAL / VAH reference; the steep middle = value-area density. Flat tails = single-print excess.</p>
        </div>

        <div id="mp-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('mp-demo').addEventListener('click', () => {
        const b = makeDemoBrackets();
        document.getElementById('mp-brackets').value =
            b.map(x => `${x.bracket_index} ${x.high} ${x.low}`).join('\n');
        showToast(t('view.market_profile.toast.demo_loaded', { n: b.length }), { level: 'info' });
    });
    document.getElementById('mp-clear').addEventListener('click', () => {
        document.getElementById('mp-brackets').value = '';
        showToast(t('view.market_profile.toast.cleared'), { level: 'info' });
    });
    document.getElementById('mp-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.brackets = document.getElementById('mp-brackets').value;
    state.tickSize = Number(document.getElementById('mp-tick').value);
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('mp-errors');
    errs.style.display = 'none';
    const { brackets, errors } = parseBracketBlob(state.brackets);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            t('common.parse_error_inline', { line: e.line_no, msg: e.message, raw: e.raw.slice(0, 80) })).join('<br>');
        const more = errors.length > 8 ? `<br>${esc(t('common.and_n_more', { n: errors.length - 8 }))}` : '';
        errs.innerHTML = `<strong>${esc(t('common.parse_errors_lead', { n: errors.length }))}</strong><br>${head}${more}`;
        errs.style.display = 'block';
        showToast(t('view.market_profile.toast.parse_error', { n: errors.length }), { level: 'warning' });
        if (brackets.length === 0) return;
    }
    const err = validateInputs(brackets, state.tickSize);
    if (err) { showErr(err); showToast(t('view.market_profile.toast.invalid'), { level: 'warning' }); return; }
    let report;
    try {
        report = await api.microMarketProfile(buildBody(brackets, state.tickSize));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        showToast(t('view.market_profile.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(report);
    renderTpo(report);
    renderTpoChart(report);
    renderCumTpoChart(report);
    const lvls = Array.isArray(report?.levels) ? report.levels.length : 0;
    showToast(t('view.market_profile.toast.built', { brackets: brackets.length, levels: lvls }), { level: 'success' });
}

function renderTpoChart(report) {
    const el = document.getElementById('mp-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const levels = (report && Array.isArray(report.levels)) ? [...report.levels] : [];
    if (levels.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.market_profile.empty_chart">${esc(t('view.market_profile.empty_chart'))}</div>`;
        return;
    }
    levels.sort((a, b) => a.price - b.price);
    const xs = levels.map(l => Number(l.price));
    const ys = levels.map(l => Number(l.tpo_count));
    const pocLine = xs.map(() => null);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { auto: true }, y: { auto: true } },
        series: [
            { label: t('view.market_profile.chart.price') },
            { label: t('view.market_profile.chart.tpo_count'),
              stroke: '#00e5ff', width: 1.4,
              points: { show: true, size: 8, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.market_profile.chart.poc'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [ { stroke: '#aab', size: 28 }, { stroke: '#aab', size: 40 } ],
        legend: { show: true },
    }, [xs, ys, pocLine], el);
}

function renderCumTpoChart(report) {
    const el = document.getElementById('mp-cum-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const levels = (report && Array.isArray(report.levels)) ? [...report.levels] : [];
    if (levels.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.market_profile.empty_cum_chart">${esc(t('view.market_profile.empty_cum_chart'))}</div>`;
        return;
    }
    levels.sort((a, b) => a.price - b.price);
    const total = levels.reduce((s, l) => s + Number(l.tpo_count), 0) || 1;
    const xs = [];
    const cum = [];
    let acc = 0;
    for (const l of levels) {
        acc += Number(l.tpo_count);
        xs.push(Number(l.price));
        cum.push(acc / total * 100);
    }
    const val = xs.map(() => 15);
    const vah = xs.map(() => 85);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { auto: true }, y: { range: [0, 100] } },
        series: [
            { label: t('view.market_profile.chart.price') },
            { label: t('view.market_profile.chart.cum_tpo'),
              stroke: '#b86bff', width: 1.6, points: { show: false } },
            { label: t('view.market_profile.chart.val_ref'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
            { label: t('view.market_profile.chart.vah_ref'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 40,
            values: (_u, splits) => splits.map(v => v.toFixed(0) + '%') }],
        legend: { show: true },
    }, [xs, cum, val, vah], el);
}

function renderSummary(r) {
    const counts = tierCounts(r);
    const vaWidth = r.value_area_high - r.value_area_low;
    document.getElementById('mp-summary').innerHTML = [
        card(t('view.market_profile.card.poc'),           fmtN(r.poc_price), 'pos'),
        card(t('view.market_profile.card.vah'),           fmtN(r.value_area_high)),
        card(t('view.market_profile.card.val'),           fmtN(r.value_area_low)),
        card(t('view.market_profile.card.va_width'),      fmtN(vaWidth)),
        card(t('view.market_profile.card.total_tpos'),    fmtInt(r.total_tpos)),
        card(t('view.market_profile.card.price_levels'),  String((r.levels || []).length)),
        card(t('view.market_profile.card.single_prints'), String(counts.single), counts.single > 0 ? 'neg' : ''),
        card(t('view.market_profile.card.tick_size'),     fmtN(r.tick_size, 4)),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderTpo(report) {
    const wrap = document.getElementById('mp-tpo');
    if (!report || !Array.isArray(report.levels) || !report.levels.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.market_profile.empty.levels">No levels.</div>`;
        return;
    }
    // Order top-down: highest price first (standard Sierra Chart layout).
    const sorted = [...report.levels].sort((a, b) => b.price - a.price);
    const maxCount = Math.max(...sorted.map(l => l.tpo_count), 1);
    wrap.innerHTML = sorted.map(l => {
        const tier = levelTier(l, report);
        const widthPct = (l.tpo_count / maxCount * 100).toFixed(2);
        const letters = levelLetters(l);
        return `
            <div class="mp-row mp-tier-${tier}">
                <div class="mp-price">${esc(fmtN(l.price))}</div>
                <div class="mp-bar-track">
                    <div class="mp-bar-fill mp-fill-${tier}" data-bar-pct="${widthPct}"></div>
                    <div class="mp-bar-letters">${esc(letters)}</div>
                </div>
                <div class="mp-count">${l.tpo_count}</div>
            </div>
        `;
    }).join('');
    requestAnimationFrame(() => {
        wrap.querySelectorAll('.mp-bar-fill').forEach(el => {
            const pct = Number(el.dataset.barPct);
            if (Number.isFinite(pct)) el.style.width = pct + '%';
        });
    });
}

function showErr(msg) {
    const el = document.getElementById('mp-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('mp-err').style.display = 'none'; }
