// DeMark Pivots view — Tom DeMark's tight 3-level pivot system.
//
// Unique among pivot systems in that the X-base formula switches with
// the prior session's close-vs-open direction. Only 3 plotted levels
// (R1 / pivot / S1) reflecting DeMark's preference for tight conservative
// bands rather than R2/R3/S2/S3 fanning out.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    validateInputs, buildBody, xBaseInfo, computeX,
    tradeBias, makeDemoSession, chartSpan, fmtN,
} from '../_demark_pivots_inputs.js';

import { t } from '../i18n.js';
import { showToast } from '../toast.js';
let state = {
    session: makeDemoSession('bullish'),
    spotNow: 105,
};

export async function renderDemarkPivots(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.demark_pivots.h1.demark_pivots" class="view-title">// DEMARK PIVOTS</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.demark_pivots.h2.prior_session_ohlc">Prior session OHLC</h2>
            <div class="inline-form">
                <label><span data-i18n="view.demark_pivots.label.open">Open</span>
                    <input id="dp-o" type="number" step="0.01" min="0" value="${state.session.open}" data-tip="view.demark_pivots.tip.open"></label>
                <label><span data-i18n="view.demark_pivots.label.high">High</span>
                    <input id="dp-h" type="number" step="0.01" min="0" value="${state.session.high}" data-tip="view.demark_pivots.tip.high"></label>
                <label><span data-i18n="view.demark_pivots.label.low">Low</span>
                    <input id="dp-l" type="number" step="0.01" min="0" value="${state.session.low}" data-tip="view.demark_pivots.tip.low"></label>
                <label><span data-i18n="view.demark_pivots.label.close">Close</span>
                    <input id="dp-c" type="number" step="0.01" min="0" value="${state.session.close}" data-tip="view.demark_pivots.tip.close"></label>
            </div>
            <div class="inline-form">
                <button data-i18n="view.demark_pivots.btn.demo_bullish_session" id="dp-demo-bull"   class="secondary" type="button" data-tip="view.demark_pivots.tip.demo_bull">Demo: bullish session</button>
                <button data-i18n="view.demark_pivots.btn.demo_bearish_session" id="dp-demo-bear"   class="secondary" type="button" data-tip="view.demark_pivots.tip.demo_bear">Demo: bearish session</button>
                <button data-i18n="view.demark_pivots.btn.demo_doji" id="dp-demo-doji"   class="secondary" type="button" data-tip="view.demark_pivots.tip.demo_doji">Demo: doji</button>
                <button data-i18n="view.demark_pivots.btn.demo_inside_day" id="dp-demo-inside" class="secondary" type="button" data-tip="view.demark_pivots.tip.demo_inside">Demo: inside day</button>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.demark_pivots.h2.today_s_spot">Today's spot</h2>
            <div class="inline-form">
                <label><span data-i18n="view.demark_pivots.label.spot_now">Spot now (for trade-bias card)</span>
                    <input id="dp-spot" type="number" step="0.01" min="0" value="${state.spotNow}" data-tip="view.demark_pivots.tip.spot"></label>
                <button data-i18n="view.demark_pivots.btn.compute" id="dp-run" class="primary" type="button" data-tip="view.demark_pivots.tip.run" data-shortcut="demark_pivots_run">Compute</button>
            </div>
            <p data-i18n="view.demark_pivots.hint.demark_s_x_base_formula_switches_on_close_vs_open_" class="muted">DeMark's X-base formula switches on close-vs-open direction:
                bearish session → low-heavy X; bullish → high-heavy X; doji → close-heavy X.
                Pivot = X/4, R1 = X/2 − low, S1 = X/2 − high.</p>
        </div>

        <div id="dp-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.demark_pivots.h2.x_base_math_shown">X-base math (shown)</h2>
            <div id="dp-xinfo" class="boot"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.demark_pivots.h2.schematic_prior_session_pivot_levels">Schematic — prior session + pivot levels</h2>
            <div id="dp-chart" style="height:280px"></div>
            <p data-i18n="view.demark_pivots.hint.magenta_prior_ohlc_envelope_yellow_pivot_red_dashe" class="muted">Magenta = prior OHLC envelope. Yellow = pivot. Red dashed = R1.
                Green dashed = S1. Cyan = today's spot. Distance from price to nearest
                level frames the day's trade setup.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.demark_pivots.h2.distance_chart">Signed distance from spot to each level (R1 / pivot / S1)</h2>
            <div id="dp-distance-chart" style="height:220px"></div>
        </div>

        <div id="dp-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (kind) => {
        const d = makeDemoSession(kind);
        document.getElementById('dp-o').value = d.open;
        document.getElementById('dp-h').value = d.high;
        document.getElementById('dp-l').value = d.low;
        document.getElementById('dp-c').value = d.close;
    };
    document.getElementById('dp-demo-bull').addEventListener('click',   () => loadDemo('bullish'));
    document.getElementById('dp-demo-bear').addEventListener('click',   () => loadDemo('bearish'));
    document.getElementById('dp-demo-doji').addEventListener('click',   () => loadDemo('doji'));
    document.getElementById('dp-demo-inside').addEventListener('click', () => loadDemo('inside'));
    document.getElementById('dp-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
    readInputs(); void compute(tok);
}

function readInputs() {
    state.session = {
        open:  Number(document.getElementById('dp-o').value),
        high:  Number(document.getElementById('dp-h').value),
        low:   Number(document.getElementById('dp-l').value),
        close: Number(document.getElementById('dp-c').value),
    };
    state.spotNow = Number(document.getElementById('dp-spot').value);
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.session);
    if (err) { showErr(err); showToast(t('view.demark_pivots.toast.invalid'), { level: 'warning' }); return; }
    let levels;
    try {
        levels = await api.anlyDemarkPivots(buildBody(state.session));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        showToast(t('view.demark_pivots.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!levels) {
        showErr(t('view.demark_pivots.err.backend_returned_null_invalid_session'));
        showToast(t('view.demark_pivots.toast.degenerate'), { level: 'warning' });
        return;
    }
    renderSummary(levels);
    renderXInfo();
    renderChart(levels);
    renderDistanceChart(levels);
    const bias = tradeBias(state.spotNow, levels);
    showToast(t('view.demark_pivots.toast.computed', { pivot: fmtN(levels.pivot), bias: bias.label }), { level: 'success' });
}

function renderSummary(levels) {
    const xinfo = xBaseInfo(state.session);
    const bias = tradeBias(state.spotNow, levels);
    const range = state.session.high - state.session.low;
    const closeBias = state.session.close - state.session.open;
    document.getElementById('dp-summary').innerHTML = [
        card(t('view.demark_pivots.card.r1_resistance'), fmtN(levels.r1), 'neg'),
        card(t('view.demark_pivots.card.pivot'),           fmtN(levels.pivot), 'pos'),
        card(t('view.demark_pivots.card.s1_support'),    fmtN(levels.s1), 'pos'),
        card(t('view.demark_pivots.card.r1_s1_band'),    fmtN(levels.r1 - levels.s1)),
        card(t('view.demark_pivots.card.prior_range'),     fmtN(range)),
        card(t('view.demark_pivots.card.close_open'),    fmtN(closeBias), closeBias >= 0 ? 'pos' : 'neg'),
        card(t('view.demark_pivots.card.x_base'),          xinfo.label, xinfo.cls),
        card(t('view.demark_pivots.card.trade_bias_now'),  bias.label, bias.cls),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderXInfo() {
    const x = computeX(state.session);
    const info = xBaseInfo(state.session);
    document.getElementById('dp-xinfo').innerHTML = `
        <div class="${info.cls}"><strong>${esc(info.label)}</strong></div>
        <div class="muted"><span data-i18n="view.demark_pivots.formula_prefix">Formula:</span> <code>${esc(info.formula)}</code></div>
        <div>X = <strong>${esc(fmtN(x, 4))}</strong> · pivot = X/4 = <strong>${esc(fmtN(x/4, 4))}</strong></div>
        <div class="muted">${esc(info.hint)}</div>
    `;
}

function renderChart(levels) {
    if (!window.uPlot) return;
    const el = document.getElementById('dp-chart');
    const { min, max } = chartSpan(state.session, levels);
    const xs = [0, 1, 2, 3, 4];
    // Prior session: high at x=1, low at x=3, open/close as bookends.
    const ohlc = xs.map((_, i) => {
        if (i === 0) return state.session.open;
        if (i === 1) return state.session.high;
        if (i === 3) return state.session.low;
        if (i === 4) return state.session.close;
        return null;
    });
    const r1Ys = xs.map(() => levels.r1);
    const pvYs = xs.map(() => levels.pivot);
    const s1Ys = xs.map(() => levels.s1);
    const spotYs = xs.map(() => state.spotNow);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 280,
        scales: { x: { time: false,}, y: { range: [min, max] } },
        series: [
            { label: t('chart.series.t') },
            { label: t('chart.series.prior_ohlc'), stroke: '#a06bff', width: 1.5,
              fill: '#a06bff14', points: { show: true, size: 8 } },
            { label: t('chart.series.r1'),     stroke: '#ff3860', width: 1.0, dash: [4, 4], points: { show: false } },
            { label: t('chart.series.pivot'),  stroke: '#ffd84a', width: 1.5,               points: { show: false } },
            { label: t('chart.series.s1'),     stroke: '#39ff14', width: 1.0, dash: [4, 4], points: { show: false } },
            { label: t('chart.series.spot_now'), stroke: '#00e5ff', width: 1.0, dash: [2, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 24 }, { stroke: '#aab', size: 60 }],
        legend: { show: true },
    }, [xs, ohlc, r1Ys, pvYs, s1Ys, spotYs], el);
}

function renderDistanceChart(levels) {
    if (!window.uPlot) return;
    const el = document.getElementById('dp-distance-chart');
    if (!el) return;
    el.innerHTML = '';
    const spot = Number(state.spotNow);
    if (!Number.isFinite(spot) || !Number.isFinite(levels.pivot)) {
        el.innerHTML = `<div class="muted" data-i18n="view.demark_pivots.empty_distance_chart">${esc(t('view.demark_pivots.empty_distance_chart'))}</div>`;
        return;
    }
    const labels = ['R1', 'pivot', 'S1'];
    const values = [levels.r1, levels.pivot, levels.s1];
    const distances = values.map(v => spot - v);
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.demark_pivots.chart.level_idx') },
            { label: t('view.demark_pivots.chart.spot_minus_level'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 14, fill: '#b86bff', stroke: '#b86bff' } },
            { label: t('view.demark_pivots.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, distances, zero], el);
}

function showErr(msg) {
    const el = document.getElementById('dp-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('dp-err').style.display = 'none'; }
