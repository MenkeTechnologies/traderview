// Chaikin Oscillator view — MACD on cumulative ADL.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_FAST, DEFAULT_SLOW, MIN_PERIOD, MAX_PERIOD,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    signBadge, crossBadge, trendBadge, divergenceBadge, summarizeBars,
    makeDemoInput,
    fmtNum, fmtSigned, fmtPrice, fmtInt,
} from '../_chaikin_osc_inputs.js';

let state = { ...makeDemoInput('accumulation') };
let chart = null;

export async function renderChaikinOsc(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.chosc.h1.title" class="view-title">// CHAIKIN OSCILLATOR</h1>

        <div class="chart-panel" data-context-scope="chaikin-oscillator">
            <h2 data-i18n="view.chosc.h2.bars">Bars
                <small data-i18n="view.chosc.h2.bars_hint" class="muted">(4 tokens per line: high low close volume; ≥ slow bars)</small></h2>
            <textarea id="co-blob" rows="6"
                      data-tip="view.chosc.tip.bars"
                      placeholder="101 99 100.8 1500\n102 100 101.5 1800\n...">${esc(barsToBlob(state.bars))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.chosc.label.fast">Fast EMA</span>
                    <input id="co-fast" type="number" step="1" min="${MIN_PERIOD}" max="${MAX_PERIOD}" value="${state.fast}"></label>
                <label><span data-i18n="view.chosc.label.slow">Slow EMA</span>
                    <input id="co-slow" type="number" step="1" min="${MIN_PERIOD}" max="${MAX_PERIOD}" value="${state.slow}"></label>
                <button data-i18n="view.chosc.btn.compute" id="co-run" class="primary"
                        data-tip="view.chosc.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.chosc.btn.demo_acc"     id="co-d1" class="secondary" type="button">Demo: accumulation</button>
                <button data-i18n="view.chosc.btn.demo_dist"    id="co-d2" class="secondary" type="button">Demo: distribution</button>
                <button data-i18n="view.chosc.btn.demo_neut"    id="co-d3" class="secondary" type="button">Demo: sideways</button>
                <button data-i18n="view.chosc.btn.demo_bulldiv" id="co-d4" class="secondary" type="button">Demo: bull divergence</button>
                <button data-i18n="view.chosc.btn.demo_beardiv" id="co-d5" class="secondary" type="button">Demo: bear divergence</button>
                <button data-i18n="view.chosc.btn.demo_cross"   id="co-d6" class="secondary" type="button">Demo: cross up</button>
                <button data-i18n="view.chosc.btn.demo_wide"    id="co-d7" class="secondary" type="button">Demo: 5/20 EMAs</button>
                <button data-i18n="view.chosc.btn.demo_flat"    id="co-d8" class="secondary" type="button">Demo: flat-zero</button>
            </div>
            <p data-i18n="view.chosc.hint.about" class="muted">MACD-style: CO = EMA(ADL, fast) − EMA(ADL, slow). Positive = short-term ADL momentum above long-term (buying pressure). Zero-line crossovers + price/CO divergences are the primary signals. Distinct from CMF (bounded rolling MFV). Defaults: 3/10.</p>
        </div>

        <div id="co-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.chosc.h2.chart">Chaikin Oscillator</h2>
            <div id="co-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.chosc.h2.stats">Bar series summary</h2>
            <div id="co-stats"></div>
        </div>

        <div id="co-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('co-blob').value = barsToBlob(state.bars);
        document.getElementById('co-fast').value = state.fast;
        document.getElementById('co-slow').value = state.slow;
    };
    document.getElementById('co-d1').addEventListener('click', () => { loadDemo('accumulation');    void compute(tok); });
    document.getElementById('co-d2').addEventListener('click', () => { loadDemo('distribution');    void compute(tok); });
    document.getElementById('co-d3').addEventListener('click', () => { loadDemo('sideways-neutral'); void compute(tok); });
    document.getElementById('co-d4').addEventListener('click', () => { loadDemo('bull-divergence'); void compute(tok); });
    document.getElementById('co-d5').addEventListener('click', () => { loadDemo('bear-divergence'); void compute(tok); });
    document.getElementById('co-d6').addEventListener('click', () => { loadDemo('cross-up');        void compute(tok); });
    document.getElementById('co-d7').addEventListener('click', () => { loadDemo('wide-fast-slow'); void compute(tok); });
    document.getElementById('co-d8').addEventListener('click', () => { loadDemo('flat-zero');       void compute(tok); });
    document.getElementById('co-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseBarsBlob(document.getElementById('co-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.chosc.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.bars = p.bars;
    const fV = parseInt(document.getElementById('co-fast').value, 10);
    const sV = parseInt(document.getElementById('co-slow').value, 10);
    state.fast = Number.isInteger(fV) && fV >= MIN_PERIOD && fV <= MAX_PERIOD ? fV : DEFAULT_FAST;
    state.slow = Number.isInteger(sV) && sV >= MIN_PERIOD && sV <= MAX_PERIOD ? sV : DEFAULT_SLOW;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.bars, state.fast, state.slow);
    renderSummary(local, true);
    renderChart(local);
    renderStats();
    let resp;
    try {
        resp = await api.anlyChaikinOscillator(buildBody(state));
    } catch (e) {
        showErr(`${t('view.chosc.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!Array.isArray(resp)) { showErr(t('view.chosc.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderStats();
}

function renderSummary(co, pending) {
    const local = localCompute(state.bars, state.fast, state.slow);
    let parityOk = Array.isArray(local) && Array.isArray(co) && local.length === co.length;
    if (parityOk) {
        for (let i = 0; i < local.length; i++) {
            const a = local[i], b = co[i];
            if (a == null && b == null) continue;
            if (a == null || b == null || Math.abs(a - b) > 1e-6) { parityOk = false; break; }
        }
    }
    const last = lastDefined(co);
    const sBadge = signBadge(last);
    const xBadge = crossBadge(co);
    const tBadge = trendBadge(co);
    const dBadge = divergenceBadge(co, state.bars);
    const xValue = xBadge.barsAgo != null ? `${t(xBadge.key)} (${xBadge.barsAgo} bars ago)` : t(xBadge.key);
    const populated = countDefined(co);
    const localTag = pending ? ` (${t('view.chosc.tag.local')})` : '';
    document.getElementById('co-summary').innerHTML = [
        card(t('view.chosc.card.sign'),     t(sBadge.key) + localTag, sBadge.cls),
        card(t('view.chosc.card.cross'),    xValue, xBadge.cls),
        card(t('view.chosc.card.trend'),    t(tBadge.key), tBadge.cls),
        card(t('view.chosc.card.divergence'), t(dBadge.key), dBadge.cls),
        card(t('view.chosc.card.last_co'),  fmtSigned(last)),
        card(t('view.chosc.card.fast'),     fmtInt(state.fast)),
        card(t('view.chosc.card.slow'),     fmtInt(state.slow)),
        card(t('view.chosc.card.populated'), `${populated} / ${state.bars.length}`),
        card(t('view.chosc.card.parity'),
             parityOk ? t('view.chosc.tag.ok') : t('view.chosc.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(co) {
    const el = document.getElementById('co-chart');
    if (!el || !window.uPlot) return;
    const xs = state.bars.map((_, i) => i);
    const arr = co.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const data = [xs, arr];
    if (chart) { try { chart.destroy(); } catch {} chart = null; }
    chart = new window.uPlot({
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.chosc.series.co'), stroke: '#1de9b6', width: 1.5 },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    }, data, el);
}

function renderStats() {
    const wrap = document.getElementById('co-stats');
    if (!state.bars.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.chosc.empty">${esc(t('view.chosc.empty'))}</div>`;
        return;
    }
    const s = summarizeBars(state.bars);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.chosc.col.metric">Metric</th>
                <th data-i18n="view.chosc.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.chosc.row.count">Bars</td>            <td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.chosc.row.last">Last close</td>       <td>${esc(fmtPrice(s.last_close))}</td></tr>
                <tr><td data-i18n="view.chosc.row.tot_vol">Total volume</td>  <td>${esc(fmtNum(s.total_volume))}</td></tr>
                <tr><td data-i18n="view.chosc.row.mean">Mean close</td>       <td>${esc(fmtPrice(s.mean_close))}</td></tr>
                <tr><td data-i18n="view.chosc.row.minl">Min low</td>          <td>${esc(fmtPrice(s.min_low))}</td></tr>
                <tr><td data-i18n="view.chosc.row.maxh">Max high</td>         <td>${esc(fmtPrice(s.max_high))}</td></tr>
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
    const el = document.getElementById('co-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('co-err').style.display = 'none'; }
