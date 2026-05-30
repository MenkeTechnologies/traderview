// Bollinger Band Distance (BBD) view.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_PERIOD, DEFAULT_N_STDEV, MIN_PERIOD, MAX_PERIOD,
    parseClosesBlob, closesToBlob, validateInputs, buildBody, localCompute,
    positionBadge, trendBadge, kissBadge, summarizeCloses,
    makeDemoInput,
    fmtPrice, fmtNum, fmtInt,
} from '../_bbd_inputs.js';

let state = { ...makeDemoInput('oscillating') };
let chart = null;
let kissChart = null;

export async function renderBbd(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.bbd.h1.title" class="view-title">// BOLLINGER BAND DISTANCE</h1>

        <div class="chart-panel" data-context-scope="bollinger-band-distance">
            <h2 data-i18n="view.bbd.h2.closes">Closes
                <small data-i18n="view.bbd.h2.closes_hint" class="muted">(positive prices; ≥ period bars; whitespace/comma)</small></h2>
            <textarea id="bd-blob" rows="6"
                      data-tip="view.bbd.tip.closes"
                      placeholder="100.5, 100.8, 101.2, ...">${esc(closesToBlob(state.closes))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.bbd.label.period">Period</span>
                    <input id="bd-period" type="number" step="1" min="${MIN_PERIOD}" max="${MAX_PERIOD}" value="${state.period}"
                           data-tip="view.bbd.tip.period"></label>
                <label><span data-i18n="view.bbd.label.n_stdev">n_stdev</span>
                    <input id="bd-stdev" type="number" step="0.1" min="0.1" value="${state.n_stdev}"
                           data-tip="view.bbd.tip.n_stdev"></label>
                <button data-i18n="view.bbd.btn.compute" id="bd-run" class="primary"
                        data-tip="view.bbd.tip.compute" data-shortcut="bbd_run" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.bbd.btn.demo_osc"   id="bd-d1" class="secondary" data-tip="view.bbd.tip.demo_osc"   type="button">Demo: oscillating</button>
                <button data-i18n="view.bbd.btn.demo_mid"   id="bd-d2" class="secondary" data-tip="view.bbd.tip.demo_mid"   type="button">Demo: midline walk</button>
                <button data-i18n="view.bbd.btn.demo_walk"  id="bd-d3" class="secondary" data-tip="view.bbd.tip.demo_walk"  type="button">Demo: band walking</button>
                <button data-i18n="view.bbd.btn.demo_brkup" id="bd-d4" class="secondary" data-tip="view.bbd.tip.demo_brkup" type="button">Demo: breakout up</button>
                <button data-i18n="view.bbd.btn.demo_brkdn" id="bd-d5" class="secondary" data-tip="view.bbd.tip.demo_brkdn" type="button">Demo: breakdown</button>
                <button data-i18n="view.bbd.btn.demo_wide"  id="bd-d6" class="secondary" data-tip="view.bbd.tip.demo_wide"  type="button">Demo: wide bands (k=3)</button>
                <button data-i18n="view.bbd.btn.demo_tight" id="bd-d7" class="secondary" data-tip="view.bbd.tip.demo_tight" type="button">Demo: tight bands (k=1)</button>
                <button data-i18n="view.bbd.btn.demo_flat"  id="bd-d8" class="secondary" data-tip="view.bbd.tip.demo_flat"  type="button">Demo: flat (sd=0)</button>
            </div>
            <p data-i18n="view.bbd.hint.about" class="muted">BBD = min(|close − upper|, |close − lower|) / band_width. 0 = close exactly at one of the bands; 0.5 = midline; > 0.5 = outside the band (breakout). Distinct from %B which is signed position. Defaults: period=20, n_stdev=2.0.</p>
        </div>

        <div id="bd-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.bbd.h2.chart">BBD series</h2>
            <div id="bd-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.bbd.h2.kiss_chart">Close + band-kiss markers (BBD &lt; 0.05)</h2>
            <div id="bd-kiss-chart" style="width:100%;height:220px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.bbd.h2.stats">Closes summary</h2>
            <div id="bd-stats"></div>
        </div>

        <div id="bd-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('bd-blob').value   = closesToBlob(state.closes);
        document.getElementById('bd-period').value = state.period;
        document.getElementById('bd-stdev').value  = state.n_stdev;
    };
    document.getElementById('bd-d1').addEventListener('click', () => { loadDemo('oscillating');    void compute(tok); });
    document.getElementById('bd-d2').addEventListener('click', () => { loadDemo('midline-walk');   void compute(tok); });
    document.getElementById('bd-d3').addEventListener('click', () => { loadDemo('band-walking');   void compute(tok); });
    document.getElementById('bd-d4').addEventListener('click', () => { loadDemo('breakout-up');    void compute(tok); });
    document.getElementById('bd-d5').addEventListener('click', () => { loadDemo('breakdown');      void compute(tok); });
    document.getElementById('bd-d6').addEventListener('click', () => { loadDemo('wide-bands');     void compute(tok); });
    document.getElementById('bd-d7').addEventListener('click', () => { loadDemo('tight-bands');    void compute(tok); });
    document.getElementById('bd-d8').addEventListener('click', () => { loadDemo('flat');           void compute(tok); });
    document.getElementById('bd-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseClosesBlob(document.getElementById('bd-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.bbd.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        showToast(t('view.bbd.toast.parse_error'), { level: 'error' });
        return;
    }
    hideErr();
    state.closes = p.closes;
    const periodV = parseInt(document.getElementById('bd-period').value, 10);
    const stdevV  = parseFloat(document.getElementById('bd-stdev').value);
    state.period  = Number.isInteger(periodV) && periodV >= MIN_PERIOD && periodV <= MAX_PERIOD ? periodV : DEFAULT_PERIOD;
    state.n_stdev = Number.isFinite(stdevV) && stdevV > 0 ? stdevV : DEFAULT_N_STDEV;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); showToast(t('view.bbd.toast.invalid'), { level: 'warning' }); return; }
    const local = localCompute(state.closes, state.period, state.n_stdev);
    renderSummary(local, true);
    renderChart(local);
    renderKissChart(local);
    renderStats();
    let resp;
    try {
        resp = await api.anlyBollingerBandDistance(buildBody(state));
    } catch (e) {
        showErr(`${t('view.bbd.err.api')}: ${e.message || e}`);
        showToast(t('view.bbd.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!Array.isArray(resp)) { showErr(t('view.bbd.err.server_rejected')); showToast(t('view.bbd.toast.server_rejected'), { level: 'error' }); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderKissChart(resp);
    renderStats();
    showToast(t('view.bbd.toast.computed'), { level: 'success' });
}

function renderSummary(bbd, pending) {
    const local = localCompute(state.closes, state.period, state.n_stdev);
    let parityOk = Array.isArray(local) && Array.isArray(bbd) && local.length === bbd.length;
    if (parityOk) {
        for (let i = 0; i < local.length; i++) {
            const a = local[i], b = bbd[i];
            if (a == null && b == null) continue;
            if (a == null || b == null || Math.abs(a - b) > 1e-6) { parityOk = false; break; }
        }
    }
    const last = lastDefined(bbd);
    const pBadge = positionBadge(last);
    const tBadge = trendBadge(bbd);
    const kBadge = kissBadge(bbd);
    const populated = countDefined(bbd);
    const localTag = pending ? ` (${t('view.bbd.tag.local')})` : '';
    document.getElementById('bd-summary').innerHTML = [
        card(t('view.bbd.card.position'),  t(pBadge.key) + localTag, pBadge.cls),
        card(t('view.bbd.card.trend'),     t(tBadge.key), tBadge.cls),
        card(t('view.bbd.card.kiss'),      t(kBadge.key), kBadge.cls),
        card(t('view.bbd.card.last_bbd'),  fmtNum(last),
             last == null ? '' : last < 0.05 ? 'pos' : last > 0.5 ? 'neg' : ''),
        card(t('view.bbd.card.period'),    fmtInt(state.period)),
        card(t('view.bbd.card.n_stdev'),   fmtNum(state.n_stdev, 2)),
        card(t('view.bbd.card.populated'), `${populated} / ${state.closes.length}`),
        card(t('view.bbd.card.parity'),
             parityOk ? t('view.bbd.tag.ok') : t('view.bbd.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(bbd) {
    const el = document.getElementById('bd-chart');
    if (!el || !window.uPlot) return;
    const xs = state.closes.map((_, i) => i);
    const arr = bbd.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const data = [xs, arr];
    if (chart) { try { chart.destroy(); } catch {} chart = null; }
    const opts = {
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.bbd.series.bbd'), stroke: '#1de9b6', width: 1.5 },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    };
    chart = new window.uPlot(opts, data, el);
}

function renderKissChart(bbd) {
    const el = document.getElementById('bd-kiss-chart');
    if (!el || !window.uPlot) return;
    if (!Array.isArray(bbd) || state.closes.length === 0) { el.innerHTML = ''; return; }
    const xs = state.closes.map((_, i) => i);
    const closes = state.closes.map(v => Number.isFinite(v) ? v : null);
    const kiss = state.closes.map((c, i) => {
        const v = bbd[i];
        if (v == null || !Number.isFinite(v) || !(v < 0.05)) return null;
        return c;
    });
    const breakout = state.closes.map((c, i) => {
        const v = bbd[i];
        if (v == null || !Number.isFinite(v) || !(v > 0.5)) return null;
        return c;
    });
    const opts = {
        width: el.clientWidth || 800,
        height: 200,
        scales: { x: { time: false } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.bbd.series.close'),
              stroke: '#888', width: 1.0, points: { show: false } },
            { label: t('view.bbd.series.kiss'),
              stroke: '#ffd84a', width: 0,
              points: { show: true, size: 9, fill: '#ffd84a', stroke: '#ffd84a' } },
            { label: t('view.bbd.series.breakout'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 9, fill: '#ff3860', stroke: '#ff3860' } },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    };
    if (kissChart) { try { kissChart.destroy(); } catch {} kissChart = null; }
    kissChart = new window.uPlot(opts, [xs, closes, kiss, breakout], el);
}

function renderStats() {
    const wrap = document.getElementById('bd-stats');
    if (!state.closes.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.bbd.empty">${esc(t('view.bbd.empty'))}</div>`;
        return;
    }
    const s = summarizeCloses(state.closes);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.bbd.col.metric">Metric</th>
                <th data-i18n="view.bbd.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.bbd.row.count">Closes</td><td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.bbd.row.last">Last</td>   <td>${esc(fmtPrice(s.last))}</td></tr>
                <tr><td data-i18n="view.bbd.row.min">Min</td>     <td>${esc(fmtPrice(s.min))}</td></tr>
                <tr><td data-i18n="view.bbd.row.max">Max</td>     <td>${esc(fmtPrice(s.max))}</td></tr>
                <tr><td data-i18n="view.bbd.row.mean">Mean</td>   <td>${esc(fmtPrice(s.mean))}</td></tr>
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
    const el = document.getElementById('bd-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('bd-err').style.display = 'none'; }
