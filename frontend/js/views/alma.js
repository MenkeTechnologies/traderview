// Arnaud Legoux Moving Average view — Gaussian-kernel FIR overlay.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_PERIOD, DEFAULT_OFFSET, DEFAULT_SIGMA, MIN_PERIOD, MAX_PERIOD,
    parseClosesBlob, closesToBlob, validateInputs, buildBody, localCompute,
    trendBadge, positionBadge, summarizeCloses,
    makeDemoInput,
    fmtPrice, fmtPriceSigned, fmtPct, fmtInt,
} from '../_alma_inputs.js';

let state = { ...makeDemoInput('uptrend') };
let chart = null;
let spreadChart = null;

export async function renderAlma(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.alma.h1.title" class="view-title">// ALMA (Arnaud Legoux MA)</h1>

        <div class="chart-panel" data-context-scope="alma">
            <h2 data-i18n="view.alma.h2.closes">Closes
                <small data-i18n="view.alma.h2.closes_hint" class="muted">(positive prices; whitespace/comma-separated)</small></h2>
            <textarea id="al-blob" rows="6"
                      data-tip="view.alma.tip.closes"
                      placeholder="100.5, 100.8, 101.2, ...">${esc(closesToBlob(state.closes))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.alma.label.period">Period</span>
                    <input id="al-period" type="number" step="1" min="${MIN_PERIOD}" max="${MAX_PERIOD}" value="${state.period}"></label>
                <label><span data-i18n="view.alma.label.offset">Offset (0–1)</span>
                    <input id="al-offset" type="number" step="0.05" min="0" max="1" value="${state.offset}"></label>
                <label><span data-i18n="view.alma.label.sigma">Sigma (>0)</span>
                    <input id="al-sigma" type="number" step="0.5" min="0.5" value="${state.sigma}"></label>
                <button data-i18n="view.alma.btn.compute" id="al-run" class="primary"
                        data-tip="view.alma.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.alma.btn.demo_up"      id="al-d1" class="secondary" type="button">Demo: uptrend</button>
                <button data-i18n="view.alma.btn.demo_down"    id="al-d2" class="secondary" type="button">Demo: downtrend</button>
                <button data-i18n="view.alma.btn.demo_side"    id="al-d3" class="secondary" type="button">Demo: sideways</button>
                <button data-i18n="view.alma.btn.demo_step"    id="al-d4" class="secondary" type="button">Demo: step</button>
                <button data-i18n="view.alma.btn.demo_hi_off"  id="al-d5" class="secondary" type="button">Demo: offset 0.95</button>
                <button data-i18n="view.alma.btn.demo_lo_off"  id="al-d6" class="secondary" type="button">Demo: offset 0.10</button>
                <button data-i18n="view.alma.btn.demo_sharp"   id="al-d7" class="secondary" type="button">Demo: sharp kernel</button>
                <button data-i18n="view.alma.btn.demo_soft"    id="al-d8" class="secondary" type="button">Demo: soft kernel</button>
            </div>
            <p data-i18n="view.alma.hint.about" class="muted">FIR filter with Gaussian-shaped weights. Offset=1.0 puts peak at the most recent bar (lowest lag, near-EMA limit); 0.0 puts it at the oldest (smoothing only); 0.5 is centered. Larger sigma sharpens the kernel — less noise rejection, faster response. Defaults: 9/0.85/6.</p>
        </div>

        <div id="al-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.alma.h2.chart">ALMA overlay</h2>
            <div id="al-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.alma.h2.spread_chart">Close − ALMA spread (% of ALMA)</h2>
            <div id="al-spread-chart" style="width:100%;height:220px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.alma.h2.stats">Series summary</h2>
            <div id="al-stats"></div>
        </div>

        <div id="al-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('al-blob').value   = closesToBlob(state.closes);
        document.getElementById('al-period').value = state.period;
        document.getElementById('al-offset').value = state.offset;
        document.getElementById('al-sigma').value  = state.sigma;
    };
    document.getElementById('al-d1').addEventListener('click', () => { loadDemo('uptrend');      void compute(tok); });
    document.getElementById('al-d2').addEventListener('click', () => { loadDemo('downtrend');    void compute(tok); });
    document.getElementById('al-d3').addEventListener('click', () => { loadDemo('sideways');     void compute(tok); });
    document.getElementById('al-d4').addEventListener('click', () => { loadDemo('step-up');      void compute(tok); });
    document.getElementById('al-d5').addEventListener('click', () => { loadDemo('high-offset');  void compute(tok); });
    document.getElementById('al-d6').addEventListener('click', () => { loadDemo('low-offset');   void compute(tok); });
    document.getElementById('al-d7').addEventListener('click', () => { loadDemo('sharp-kernel'); void compute(tok); });
    document.getElementById('al-d8').addEventListener('click', () => { loadDemo('soft-kernel');  void compute(tok); });
    document.getElementById('al-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseClosesBlob(document.getElementById('al-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.alma.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.closes = p.closes;
    const periodV = parseInt(document.getElementById('al-period').value, 10);
    const offsetV = parseFloat(document.getElementById('al-offset').value);
    const sigmaV  = parseFloat(document.getElementById('al-sigma').value);
    state.period = Number.isInteger(periodV) && periodV >= MIN_PERIOD && periodV <= MAX_PERIOD ? periodV : DEFAULT_PERIOD;
    state.offset = Number.isFinite(offsetV) && offsetV >= 0 && offsetV <= 1 ? offsetV : DEFAULT_OFFSET;
    state.sigma  = Number.isFinite(sigmaV)  && sigmaV  >  0 ? sigmaV  : DEFAULT_SIGMA;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.closes, state.period, state.offset, state.sigma);
    renderSummary(local, true);
    renderChart(local);
    renderSpreadChart(local);
    renderStats();
    let resp;
    try {
        resp = await api.anlyAlma(buildBody(state));
    } catch (e) {
        showErr(`${t('view.alma.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!Array.isArray(resp)) { showErr(t('view.alma.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderSpreadChart(resp);
    renderStats();
}

function renderSummary(alma, pending) {
    const local = localCompute(state.closes, state.period, state.offset, state.sigma);
    let parityOk = Array.isArray(local) && Array.isArray(alma) && local.length === alma.length;
    if (parityOk) {
        for (let i = 0; i < local.length; i++) {
            const a = local[i], b = alma[i];
            if (a == null && b == null) continue;
            if (a == null || b == null || Math.abs(a - b) > 1e-9) { parityOk = false; break; }
        }
    }
    const tBadge = trendBadge(alma);
    const lastClose = state.closes[state.closes.length - 1];
    const lastAlma = lastDefined(alma);
    const pBadge = positionBadge(lastClose, lastAlma);
    const populated = countDefined(alma);
    const localTag = pending ? ` (${t('view.alma.tag.local')})` : '';
    document.getElementById('al-summary').innerHTML = [
        card(t('view.alma.card.trend'),    t(tBadge.key) + localTag, tBadge.cls),
        card(t('view.alma.card.position'), t(pBadge.key), pBadge.cls),
        card(t('view.alma.card.last_close'), fmtPrice(lastClose)),
        card(t('view.alma.card.last_alma'),  fmtPrice(lastAlma)),
        card(t('view.alma.card.gap'),        fmtPriceSigned(lastClose - lastAlma),
             lastClose > lastAlma ? 'pos' : lastClose < lastAlma ? 'neg' : ''),
        card(t('view.alma.card.gap_pct'),    fmtPct(lastAlma ? (lastClose - lastAlma) / lastAlma : NaN),
             lastClose > lastAlma ? 'pos' : lastClose < lastAlma ? 'neg' : ''),
        card(t('view.alma.card.period'),   fmtInt(state.period)),
        card(t('view.alma.card.offset'),   fmtPrice(state.offset, 2)),
        card(t('view.alma.card.sigma'),    fmtPrice(state.sigma, 2)),
        card(t('view.alma.card.populated'), `${populated} / ${state.closes.length}`),
        card(t('view.alma.card.parity'),
             parityOk ? t('view.alma.tag.ok') : t('view.alma.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(alma) {
    const el = document.getElementById('al-chart');
    if (!el || !window.uPlot) return;
    const xs = state.closes.map((_, i) => i);
    const closes = state.closes;
    const overlay = alma.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const data = [xs, closes, overlay];
    if (chart) { try { chart.destroy(); } catch {} chart = null; }
    const opts = {
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.alma.series.close'), stroke: '#888', width: 1 },
            { label: t('view.alma.series.alma'),  stroke: '#1de9b6', width: 2 },
        ],
        axes: [
            { stroke: '#aaa' },
            { stroke: '#aaa' },
        ],
        legend: { show: true },
    };
    chart = new window.uPlot(opts, data, el);
}

function renderSpreadChart(alma) {
    const el = document.getElementById('al-spread-chart');
    if (!el || !window.uPlot) return;
    if (!Array.isArray(alma) || state.closes.length === 0) { el.innerHTML = ''; return; }
    const xs = state.closes.map((_, i) => i);
    const spread = state.closes.map((c, i) => {
        const a = alma[i];
        if (a == null || !Number.isFinite(a) || !(a !== 0)) return null;
        return ((c - a) / a) * 100;
    });
    const zero = xs.map(() => 0);
    const opts = {
        width: el.clientWidth || 800,
        height: 200,
        scales: { x: { time: false } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.alma.series.spread_pct'),
              stroke: '#00e5ff', width: 1.5,
              points: { show: false } },
            { label: t('view.alma.series.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    };
    if (spreadChart) { try { spreadChart.destroy(); } catch {} spreadChart = null; }
    spreadChart = new window.uPlot(opts, [xs, spread, zero], el);
}

function renderStats() {
    const wrap = document.getElementById('al-stats');
    if (!state.closes.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.alma.empty">${esc(t('view.alma.empty'))}</div>`;
        return;
    }
    const s = summarizeCloses(state.closes);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.alma.col.metric">Metric</th>
                <th data-i18n="view.alma.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.alma.row.count">Closes</td><td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.alma.row.last">Last</td>   <td>${esc(fmtPrice(s.last))}</td></tr>
                <tr><td data-i18n="view.alma.row.min">Min</td>     <td>${esc(fmtPrice(s.min))}</td></tr>
                <tr><td data-i18n="view.alma.row.max">Max</td>     <td>${esc(fmtPrice(s.max))}</td></tr>
                <tr><td data-i18n="view.alma.row.mean">Mean</td>   <td>${esc(fmtPrice(s.mean))}</td></tr>
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
    const el = document.getElementById('al-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('al-err').style.display = 'none'; }
