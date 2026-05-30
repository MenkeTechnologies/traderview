// Bollinger Oscillators (combined %B + Bandwidth) view.
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
    pbBadge, bwBadge, pbTrendBadge, summarizeCloses,
    makeDemoInput,
    fmtPrice, fmtNum, fmtInt,
} from '../_bb_osc_inputs.js';

let state = { ...makeDemoInput('normal-trend') };
let chartBands = null;
let chartOsc = null;

export async function renderBbOsc(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.bbosc.h1.title" class="view-title">// BOLLINGER OSCILLATORS</h1>

        <div class="chart-panel" data-context-scope="bollinger-oscillators">
            <h2 data-i18n="view.bbosc.h2.closes">Closes
                <small data-i18n="view.bbosc.h2.closes_hint" class="muted">(positive prices; ≥ period bars)</small></h2>
            <textarea id="bo-blob" rows="6"
                      data-tip="view.bbosc.tip.closes"
                      placeholder="100.5, 100.8, 101.2, ...">${esc(closesToBlob(state.closes))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.bbosc.label.period">Period</span>
                    <input id="bo-period" type="number" step="1" min="${MIN_PERIOD}" max="${MAX_PERIOD}" value="${state.period}"
                           data-tip="view.bbosc.tip.period"></label>
                <label><span data-i18n="view.bbosc.label.k">k (σ)</span>
                    <input id="bo-k" type="number" step="0.1" min="0" value="${state.k}"
                           data-tip="view.bbosc.tip.k"></label>
                <button data-i18n="view.bbosc.btn.compute" id="bo-run" class="primary"
                        data-tip="view.bbosc.tip.compute" data-shortcut="bb_osc_run" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.bbosc.btn.demo_norm"  id="bo-d1" class="secondary" data-tip="view.bbosc.tip.demo_norm"  type="button">Demo: normal trend</button>
                <button data-i18n="view.bbosc.btn.demo_ttm"   id="bo-d2" class="secondary" data-tip="view.bbosc.tip.demo_ttm"   type="button">Demo: TTM squeeze</button>
                <button data-i18n="view.bbosc.btn.demo_walku" id="bo-d3" class="secondary" data-tip="view.bbosc.tip.demo_walku" type="button">Demo: walking upper</button>
                <button data-i18n="view.bbosc.btn.demo_walkl" id="bo-d4" class="secondary" data-tip="view.bbosc.tip.demo_walkl" type="button">Demo: walking lower</button>
                <button data-i18n="view.bbosc.btn.demo_osc"   id="bo-d5" class="secondary" data-tip="view.bbosc.tip.demo_osc"   type="button">Demo: oscillating</button>
                <button data-i18n="view.bbosc.btn.demo_flat"  id="bo-d6" class="secondary" data-tip="view.bbosc.tip.demo_flat"  type="button">Demo: flat market</button>
                <button data-i18n="view.bbosc.btn.demo_wide"  id="bo-d7" class="secondary" data-tip="view.bbosc.tip.demo_wide"  type="button">Demo: wide bands (k=3)</button>
                <button data-i18n="view.bbosc.btn.demo_tight" id="bo-d8" class="secondary" data-tip="view.bbosc.tip.demo_tight" type="button">Demo: tight bands (k=1)</button>
            </div>
            <p data-i18n="view.bbosc.hint.about" class="muted">Combined %B + Bandwidth oscillators. %B = (close − lower) / (upper − lower) ∈ [0, 1] at bands; > 1 breakout / < 0 breakdown. Bandwidth = (upper − lower) / middle squeezes absolute width into a unitless vol regime signal — low values flag the TTM-squeeze setup.</p>
        </div>

        <div id="bo-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.bbosc.h2.bands">Price + bands</h2>
            <div id="bo-chart-bands" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.bbosc.h2.osc">%B + Bandwidth</h2>
            <div id="bo-chart-osc" style="width:100%;height:280px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.bbosc.h2.stats">Closes summary</h2>
            <div id="bo-stats"></div>
        </div>

        <div id="bo-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('bo-blob').value   = closesToBlob(state.closes);
        document.getElementById('bo-period').value = state.period;
        document.getElementById('bo-k').value      = state.k;
    };
    document.getElementById('bo-d1').addEventListener('click', () => { loadDemo('normal-trend');   void compute(tok); });
    document.getElementById('bo-d2').addEventListener('click', () => { loadDemo('ttm-squeeze');    void compute(tok); });
    document.getElementById('bo-d3').addEventListener('click', () => { loadDemo('walking-upper');  void compute(tok); });
    document.getElementById('bo-d4').addEventListener('click', () => { loadDemo('walking-lower');  void compute(tok); });
    document.getElementById('bo-d5').addEventListener('click', () => { loadDemo('oscillating');    void compute(tok); });
    document.getElementById('bo-d6').addEventListener('click', () => { loadDemo('flat');           void compute(tok); });
    document.getElementById('bo-d7').addEventListener('click', () => { loadDemo('wide-bands');     void compute(tok); });
    document.getElementById('bo-d8').addEventListener('click', () => { loadDemo('tight-bands');    void compute(tok); });
    document.getElementById('bo-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseClosesBlob(document.getElementById('bo-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.bbosc.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        showToast(t('view.bbosc.toast.parse_error'), { level: 'error' });
        return;
    }
    hideErr();
    state.closes = p.closes;
    const periodV = parseInt(document.getElementById('bo-period').value, 10);
    const kV = parseFloat(document.getElementById('bo-k').value);
    state.period = Number.isInteger(periodV) && periodV >= MIN_PERIOD && periodV <= MAX_PERIOD ? periodV : DEFAULT_PERIOD;
    state.k = Number.isFinite(kV) && kV >= 0 ? kV : DEFAULT_K;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); showToast(t('view.bbosc.toast.invalid'), { level: 'warning' }); return; }
    const local = localCompute(state.closes, state.period, state.k);
    renderSummary(local, true);
    renderChartBands(local);
    renderChartOsc(local);
    renderStats();
    let resp;
    try {
        resp = await api.anlyBollingerOscillators(buildBody(state));
    } catch (e) {
        showErr(`${t('view.bbosc.err.api')}: ${e.message || e}`);
        showToast(t('view.bbosc.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp || !Array.isArray(resp.middle)) { showErr(t('view.bbosc.err.server_rejected')); showToast(t('view.bbosc.toast.server_rejected'), { level: 'error' }); return; }
    renderSummary(resp, false);
    renderChartBands(resp);
    renderChartOsc(resp);
    renderStats();
    showToast(t('view.bbosc.toast.computed'), { level: 'success' });
}

function renderSummary(report, pending) {
    const local = localCompute(state.closes, state.period, state.k);
    let parityOk = arraysEqual(local.percent_b, report.percent_b)
        && arraysEqual(local.bandwidth, report.bandwidth)
        && arraysEqual(local.middle, report.middle)
        && arraysEqual(local.upper, report.upper)
        && arraysEqual(local.lower, report.lower);
    const lastClose = state.closes.length ? state.closes[state.closes.length - 1] : NaN;
    const lastPb  = lastDefined(report.percent_b);
    const lastBw  = lastDefined(report.bandwidth);
    const lastMid = lastDefined(report.middle);
    const lastUp  = lastDefined(report.upper);
    const lastLo  = lastDefined(report.lower);
    const pBadge = pbBadge(lastPb);
    const bBadge = bwBadge(report.bandwidth);
    const tBadge = pbTrendBadge(report.percent_b);
    const populated = countDefined(report.middle);
    const localTag = pending ? ` (${t('view.bbosc.tag.local')})` : '';
    document.getElementById('bo-summary').innerHTML = [
        card(t('view.bbosc.card.pb_zone'),  t(pBadge.key) + localTag, pBadge.cls),
        card(t('view.bbosc.card.bw_regime'), t(bBadge.key), bBadge.cls),
        card(t('view.bbosc.card.pb_trend'), t(tBadge.key), tBadge.cls),
        card(t('view.bbosc.card.last_pb'),  fmtNum(lastPb)),
        card(t('view.bbosc.card.last_bw'),  fmtNum(lastBw)),
        card(t('view.bbosc.card.last_close'), fmtPrice(lastClose)),
        card(t('view.bbosc.card.last_mid'),   fmtPrice(lastMid)),
        card(t('view.bbosc.card.last_upper'), fmtPrice(lastUp),  'pos'),
        card(t('view.bbosc.card.last_lower'), fmtPrice(lastLo), 'neg'),
        card(t('view.bbosc.card.period'),   fmtInt(state.period)),
        card(t('view.bbosc.card.k'),        fmtNum(state.k, 2)),
        card(t('view.bbosc.card.populated'), `${populated} / ${state.closes.length}`),
        card(t('view.bbosc.card.parity'),
             parityOk ? t('view.bbosc.tag.ok') : t('view.bbosc.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChartBands(report) {
    const el = document.getElementById('bo-chart-bands');
    if (!el || !window.uPlot) return;
    const xs = state.closes.map((_, i) => i);
    const closes = state.closes;
    const mid = report.middle.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const up  = report.upper.map(v  => (v == null || !Number.isFinite(v) ? null : v));
    const lo  = report.lower.map(v  => (v == null || !Number.isFinite(v) ? null : v));
    const data = [xs, closes, mid, up, lo];
    if (chartBands) { try { chartBands.destroy(); } catch {} chartBands = null; }
    chartBands = new window.uPlot({
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.bbosc.series.close'),  stroke: '#888',     width: 1 },
            { label: t('view.bbosc.series.middle'), stroke: '#ffd54f', width: 1.5 },
            { label: t('view.bbosc.series.upper'),  stroke: '#1de9b6', width: 1.5 },
            { label: t('view.bbosc.series.lower'),  stroke: '#ff5252', width: 1.5 },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    }, data, el);
}

function renderChartOsc(report) {
    const el = document.getElementById('bo-chart-osc');
    if (!el || !window.uPlot) return;
    const xs = state.closes.map((_, i) => i);
    const pb = report.percent_b.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const bw = report.bandwidth.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const data = [xs, pb, bw];
    if (chartOsc) { try { chartOsc.destroy(); } catch {} chartOsc = null; }
    chartOsc = new window.uPlot({
        width: el.clientWidth || 800,
        height: 280,
        scales: { x: { time: false }, y: {}, yBw: {} },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.bbosc.series.pb'), stroke: '#1de9b6', width: 1.5, scale: 'y' },
            { label: t('view.bbosc.series.bw'), stroke: '#ffd54f', width: 1.5, scale: 'yBw' },
        ],
        axes: [
            { stroke: '#aaa' },
            { stroke: '#1de9b6', scale: 'y' },
            { stroke: '#ffd54f', scale: 'yBw', side: 1 },
        ],
        legend: { show: true },
    }, data, el);
}

function renderStats() {
    const wrap = document.getElementById('bo-stats');
    if (!state.closes.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.bbosc.empty">${esc(t('view.bbosc.empty'))}</div>`;
        return;
    }
    const s = summarizeCloses(state.closes);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.bbosc.col.metric">Metric</th>
                <th data-i18n="view.bbosc.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.bbosc.row.count">Closes</td><td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.bbosc.row.last">Last</td>   <td>${esc(fmtPrice(s.last))}</td></tr>
                <tr><td data-i18n="view.bbosc.row.min">Min</td>     <td>${esc(fmtPrice(s.min))}</td></tr>
                <tr><td data-i18n="view.bbosc.row.max">Max</td>     <td>${esc(fmtPrice(s.max))}</td></tr>
                <tr><td data-i18n="view.bbosc.row.mean">Mean</td>   <td>${esc(fmtPrice(s.mean))}</td></tr>
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
    const el = document.getElementById('bo-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('bo-err').style.display = 'none'; }
