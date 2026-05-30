// Bollinger Band Width (BBW) + %B view.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_PERIOD, DEFAULT_K, MIN_PERIOD, MAX_PERIOD,
    parseClosesBlob, closesToBlob, validateInputs, buildBody, localCompute,
    squeezeBadge, percentBBadge, widthTrendBadge, summarizeCloses,
    makeDemoInput,
    fmtPrice, fmtNum, fmtInt,
} from '../_bbw_inputs.js';

let state = { ...makeDemoInput('normal') };
let chartBands = null;
let chartBbw = null;

export async function renderBbw(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.bbw.h1.title" class="view-title">// BOLLINGER BAND WIDTH + %B</h1>

        <div class="chart-panel" data-context-scope="bollinger-band-width">
            <h2 data-i18n="view.bbw.h2.closes">Closes
                <small data-i18n="view.bbw.h2.closes_hint" class="muted">(positive prices; whitespace/comma-separated)</small></h2>
            <textarea id="bw-blob" rows="6"
                      data-tip="view.bbw.tip.closes"
                      placeholder="100.5, 100.8, 101.2, ...">${esc(closesToBlob(state.closes))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.bbw.label.period">Period</span>
                    <input id="bw-period" type="number" step="1" min="${MIN_PERIOD}" max="${MAX_PERIOD}" value="${state.period}"
                           data-tip="view.bbw.tip.period"></label>
                <label><span data-i18n="view.bbw.label.k">k (σ)</span>
                    <input id="bw-k" type="number" step="0.1" min="0" value="${state.k}"
                           data-tip="view.bbw.tip.k"></label>
                <button data-i18n="view.bbw.btn.compute" id="bw-run" class="primary"
                        data-tip="view.bbw.tip.compute" data-shortcut="bbw_run" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.bbw.btn.demo_norm"   id="bw-d1" class="secondary" data-tip="view.bbw.tip.demo_norm"   type="button">Demo: normal trend</button>
                <button data-i18n="view.bbw.btn.demo_sqzbrk" id="bw-d2" class="secondary" data-tip="view.bbw.tip.demo_sqzbrk" type="button">Demo: squeeze → break</button>
                <button data-i18n="view.bbw.btn.demo_expctr" id="bw-d3" class="secondary" data-tip="view.bbw.tip.demo_expctr" type="button">Demo: expansion → contract</button>
                <button data-i18n="view.bbw.btn.demo_trup"   id="bw-d4" class="secondary" data-tip="view.bbw.tip.demo_trup"   type="button">Demo: trending up</button>
                <button data-i18n="view.bbw.btn.demo_trdn"   id="bw-d5" class="secondary" data-tip="view.bbw.tip.demo_trdn"   type="button">Demo: trending down</button>
                <button data-i18n="view.bbw.btn.demo_walk"   id="bw-d6" class="secondary" data-tip="view.bbw.tip.demo_walk"   type="button">Demo: walking bands</button>
                <button data-i18n="view.bbw.btn.demo_wide"   id="bw-d7" class="secondary" data-tip="view.bbw.tip.demo_wide"   type="button">Demo: wide bands (k=3)</button>
                <button data-i18n="view.bbw.btn.demo_flat"   id="bw-d8" class="secondary" data-tip="view.bbw.tip.demo_flat"   type="button">Demo: flat (sd=0)</button>
            </div>
            <p data-i18n="view.bbw.hint.about" class="muted">Bollinger Bands: middle = SMA(close, period); upper = middle + k·σ; lower = middle − k·σ. BBW = (upper−lower)/middle measures volatility regime. %B = (close−lower)/(upper−lower) positions price within the bands (>1 above upper, <0 below lower). Defaults: 20 / 2.0.</p>
        </div>

        <div id="bw-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.bbw.h2.bands">Price + Bollinger Bands</h2>
            <div id="bw-chart-bands" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.bbw.h2.bbw">BBW + %B</h2>
            <div id="bw-chart-bbw" style="width:100%;height:280px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.bbw.h2.stats">Series summary</h2>
            <div id="bw-stats"></div>
        </div>

        <div id="bw-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('bw-blob').value   = closesToBlob(state.closes);
        document.getElementById('bw-period').value = state.period;
        document.getElementById('bw-k').value      = state.k;
    };
    document.getElementById('bw-d1').addEventListener('click', () => { loadDemo('normal');                void compute(tok); });
    document.getElementById('bw-d2').addEventListener('click', () => { loadDemo('squeeze-then-break');    void compute(tok); });
    document.getElementById('bw-d3').addEventListener('click', () => { loadDemo('expansion-then-contract'); void compute(tok); });
    document.getElementById('bw-d4').addEventListener('click', () => { loadDemo('trending-up');           void compute(tok); });
    document.getElementById('bw-d5').addEventListener('click', () => { loadDemo('trending-down');         void compute(tok); });
    document.getElementById('bw-d6').addEventListener('click', () => { loadDemo('walking-bands');         void compute(tok); });
    document.getElementById('bw-d7').addEventListener('click', () => { loadDemo('wide-bands');            void compute(tok); });
    document.getElementById('bw-d8').addEventListener('click', () => { loadDemo('flat-window');           void compute(tok); });
    document.getElementById('bw-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseClosesBlob(document.getElementById('bw-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.bbw.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        showToast(t('view.bbw.toast.parse_error'), { level: 'error' });
        return;
    }
    hideErr();
    state.closes = p.closes;
    const periodV = parseInt(document.getElementById('bw-period').value, 10);
    const kV = parseFloat(document.getElementById('bw-k').value);
    state.period = Number.isInteger(periodV) && periodV >= MIN_PERIOD && periodV <= MAX_PERIOD ? periodV : DEFAULT_PERIOD;
    state.k = Number.isFinite(kV) && kV >= 0 ? kV : DEFAULT_K;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); showToast(t('view.bbw.toast.invalid'), { level: 'warning' }); return; }
    const local = localCompute(state.closes, state.period, state.k);
    renderSummary(local, true);
    renderChartBands(local);
    renderChartBbw(local);
    renderStats();
    let resp;
    try {
        resp = await api.anlyBollingerBandWidth(buildBody(state));
    } catch (e) {
        showErr(`${t('view.bbw.err.api')}: ${e.message || e}`);
        showToast(t('view.bbw.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp || !Array.isArray(resp.middle)) { showErr(t('view.bbw.err.server_rejected')); showToast(t('view.bbw.toast.server_rejected'), { level: 'error' }); return; }
    renderSummary(resp, false);
    renderChartBands(resp);
    renderChartBbw(resp);
    renderStats();
    showToast(t('view.bbw.toast.computed'), { level: 'success' });
}

function renderSummary(report, pending) {
    const local = localCompute(state.closes, state.period, state.k);
    let parityOk = arraysEqual(local.middle, report.middle)
        && arraysEqual(local.upper, report.upper)
        && arraysEqual(local.lower, report.lower)
        && arraysEqual(local.band_width, report.band_width)
        && arraysEqual(local.percent_b, report.percent_b);
    const lastClose = state.closes.length ? state.closes[state.closes.length - 1] : NaN;
    const lastMid   = lastDefined(report.middle);
    const lastUp    = lastDefined(report.upper);
    const lastLow   = lastDefined(report.lower);
    const lastBbw   = lastDefined(report.band_width);
    const lastPb    = lastDefined(report.percent_b);
    const sBadge = squeezeBadge(report.band_width);
    const pBadge = percentBBadge(lastPb);
    const tBadge = widthTrendBadge(report.band_width);
    const populated = countDefined(report.middle);
    const localTag = pending ? ` (${t('view.bbw.tag.local')})` : '';
    document.getElementById('bw-summary').innerHTML = [
        card(t('view.bbw.card.squeeze'),  t(sBadge.key) + localTag, sBadge.cls),
        card(t('view.bbw.card.percent_b'), t(pBadge.key), pBadge.cls),
        card(t('view.bbw.card.width_trend'), t(tBadge.key), tBadge.cls),
        card(t('view.bbw.card.last_pb'),  fmtNum(lastPb)),
        card(t('view.bbw.card.last_bbw'), fmtNum(lastBbw)),
        card(t('view.bbw.card.last_close'), fmtPrice(lastClose)),
        card(t('view.bbw.card.last_mid'),  fmtPrice(lastMid)),
        card(t('view.bbw.card.last_upper'), fmtPrice(lastUp),  'pos'),
        card(t('view.bbw.card.last_lower'), fmtPrice(lastLow), 'neg'),
        card(t('view.bbw.card.period'),   fmtInt(state.period)),
        card(t('view.bbw.card.k'),        fmtNum(state.k, 2)),
        card(t('view.bbw.card.populated'), `${populated} / ${state.closes.length}`),
        card(t('view.bbw.card.parity'),
             parityOk ? t('view.bbw.tag.ok') : t('view.bbw.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChartBands(report) {
    const el = document.getElementById('bw-chart-bands');
    if (!el || !window.uPlot) return;
    const xs = state.closes.map((_, i) => i);
    const closes = state.closes;
    const mid = report.middle.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const up  = report.upper.map(v  => (v == null || !Number.isFinite(v) ? null : v));
    const lo  = report.lower.map(v  => (v == null || !Number.isFinite(v) ? null : v));
    const data = [xs, closes, mid, up, lo];
    if (chartBands) { try { chartBands.destroy(); } catch {} chartBands = null; }
    const opts = {
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.bbw.series.close'),  stroke: '#888',     width: 1 },
            { label: t('view.bbw.series.middle'), stroke: '#ffd54f', width: 1.5 },
            { label: t('view.bbw.series.upper'),  stroke: '#1de9b6', width: 1.5 },
            { label: t('view.bbw.series.lower'),  stroke: '#ff5252', width: 1.5 },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    };
    chartBands = new window.uPlot(opts, data, el);
}

function renderChartBbw(report) {
    const el = document.getElementById('bw-chart-bbw');
    if (!el || !window.uPlot) return;
    const xs = state.closes.map((_, i) => i);
    const bbw = report.band_width.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const pb  = report.percent_b.map(v  => (v == null || !Number.isFinite(v) ? null : v));
    const data = [xs, bbw, pb];
    if (chartBbw) { try { chartBbw.destroy(); } catch {} chartBbw = null; }
    const opts = {
        width: el.clientWidth || 800,
        height: 280,
        scales: { x: { time: false }, y: {}, yPb: {} },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.bbw.series.bbw'), stroke: '#1de9b6', width: 1.5, scale: 'y' },
            { label: t('view.bbw.series.pb'),  stroke: '#ffd54f', width: 1.5, scale: 'yPb' },
        ],
        axes: [
            { stroke: '#aaa' },
            { stroke: '#1de9b6', scale: 'y' },
            { stroke: '#ffd54f', scale: 'yPb', side: 1 },
        ],
        legend: { show: true },
    };
    chartBbw = new window.uPlot(opts, data, el);
}

function renderStats() {
    const wrap = document.getElementById('bw-stats');
    if (!state.closes.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.bbw.empty">${esc(t('view.bbw.empty'))}</div>`;
        return;
    }
    const s = summarizeCloses(state.closes);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.bbw.col.metric">Metric</th>
                <th data-i18n="view.bbw.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.bbw.row.count">Closes</td><td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.bbw.row.last">Last</td>   <td>${esc(fmtPrice(s.last))}</td></tr>
                <tr><td data-i18n="view.bbw.row.min">Min</td>     <td>${esc(fmtPrice(s.min))}</td></tr>
                <tr><td data-i18n="view.bbw.row.max">Max</td>     <td>${esc(fmtPrice(s.max))}</td></tr>
                <tr><td data-i18n="view.bbw.row.mean">Mean</td>   <td>${esc(fmtPrice(s.mean))}</td></tr>
            </tbody>
        </table>
    `;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function arraysEqual(a, b) {
    if (!Array.isArray(a) || !Array.isArray(b) || a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) {
        if (a[i] == null && b[i] == null) continue;
        if (a[i] == null || b[i] == null || Math.abs(a[i] - b[i]) > 1e-6) return false;
    }
    return true;
}

function lastDefined(arr) {
    if (!Array.isArray(arr)) return NaN;
    for (let i = arr.length - 1; i >= 0; i--) {
        if (arr[i] != null && Number.isFinite(arr[i])) return arr[i];
    }
    return NaN;
}

function countDefined(arr) {
    if (!Array.isArray(arr)) return 0;
    let n = 0;
    for (const v of arr) if (v != null && Number.isFinite(v)) n++;
    return n;
}

function showErr(msg) {
    const el = document.getElementById('bw-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('bw-err').style.display = 'none'; }
