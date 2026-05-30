// Bollinger %B (standalone) view.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_PERIOD, DEFAULT_N_STDEV, MIN_PERIOD, MAX_PERIOD,
    parseClosesBlob, closesToBlob, validateInputs, buildBody, localCompute,
    zoneBadge, crossBadge, trendBadge, summarizeCloses,
    makeDemoInput,
    fmtPrice, fmtPb, fmtInt,
} from '../_bb_pb_inputs.js';

let state = { ...makeDemoInput('walking-up') };
let chart = null;
let zoneChart = null;

export async function renderBbPercentB(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.bbpb.h1.title" class="view-title">// BOLLINGER %B</h1>

        <div class="chart-panel" data-context-scope="bollinger-percent-b">
            <h2 data-i18n="view.bbpb.h2.closes">Closes
                <small data-i18n="view.bbpb.h2.closes_hint" class="muted">(positive prices; ≥ period bars; whitespace/comma)</small></h2>
            <textarea id="pb-blob" rows="6"
                      data-tip="view.bbpb.tip.closes"
                      placeholder="100.5, 100.8, 101.2, ...">${esc(closesToBlob(state.closes))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.bbpb.label.period">Period</span>
                    <input id="pb-period" type="number" step="1" min="${MIN_PERIOD}" max="${MAX_PERIOD}" value="${state.period}"></label>
                <label><span data-i18n="view.bbpb.label.n_stdev">n_stdev</span>
                    <input id="pb-stdev" type="number" step="0.1" min="0.1" value="${state.n_stdev}"></label>
                <button data-i18n="view.bbpb.btn.compute" id="pb-run" class="primary"
                        data-tip="view.bbpb.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.bbpb.btn.demo_up"     id="pb-d1" class="secondary" type="button">Demo: walking up</button>
                <button data-i18n="view.bbpb.btn.demo_down"   id="pb-d2" class="secondary" type="button">Demo: walking down</button>
                <button data-i18n="view.bbpb.btn.demo_osc"    id="pb-d3" class="secondary" type="button">Demo: oscillating</button>
                <button data-i18n="view.bbpb.btn.demo_brkup"  id="pb-d4" class="secondary" type="button">Demo: breakout up</button>
                <button data-i18n="view.bbpb.btn.demo_brkdn"  id="pb-d5" class="secondary" type="button">Demo: breakdown</button>
                <button data-i18n="view.bbpb.btn.demo_revert" id="pb-d6" class="secondary" type="button">Demo: mean-revert</button>
                <button data-i18n="view.bbpb.btn.demo_flat"   id="pb-d7" class="secondary" type="button">Demo: flat (=0.5)</button>
                <button data-i18n="view.bbpb.btn.demo_tight"  id="pb-d8" class="secondary" type="button">Demo: tight bands (k=1)</button>
            </div>
            <p data-i18n="view.bbpb.hint.about" class="muted">%B = (close − lower) / (upper − lower). Reads 0 at the lower band, 0.5 at midline, 1 at upper band. > 1 = breakout above upper; < 0 = breakdown below lower. Defaults: period=20, n_stdev=2.0.</p>
        </div>

        <div id="pb-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.bbpb.h2.chart">%B series</h2>
            <div id="pb-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.bbpb.h2.zone_chart">Zone classification per bar (+1 / 0 / −1)</h2>
            <div id="pb-zone-chart" style="width:100%;height:200px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.bbpb.h2.stats">Closes summary</h2>
            <div id="pb-stats"></div>
        </div>

        <div id="pb-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('pb-blob').value   = closesToBlob(state.closes);
        document.getElementById('pb-period').value = state.period;
        document.getElementById('pb-stdev').value  = state.n_stdev;
    };
    document.getElementById('pb-d1').addEventListener('click', () => { loadDemo('walking-up');   void compute(tok); });
    document.getElementById('pb-d2').addEventListener('click', () => { loadDemo('walking-down'); void compute(tok); });
    document.getElementById('pb-d3').addEventListener('click', () => { loadDemo('oscillating');  void compute(tok); });
    document.getElementById('pb-d4').addEventListener('click', () => { loadDemo('breakout-up');  void compute(tok); });
    document.getElementById('pb-d5').addEventListener('click', () => { loadDemo('breakdown');    void compute(tok); });
    document.getElementById('pb-d6').addEventListener('click', () => { loadDemo('mean-revert');  void compute(tok); });
    document.getElementById('pb-d7').addEventListener('click', () => { loadDemo('flat');         void compute(tok); });
    document.getElementById('pb-d8').addEventListener('click', () => { loadDemo('tight-bands');  void compute(tok); });
    document.getElementById('pb-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseClosesBlob(document.getElementById('pb-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.bbpb.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.closes = p.closes;
    const periodV = parseInt(document.getElementById('pb-period').value, 10);
    const stdevV  = parseFloat(document.getElementById('pb-stdev').value);
    state.period  = Number.isInteger(periodV) && periodV >= MIN_PERIOD && periodV <= MAX_PERIOD ? periodV : DEFAULT_PERIOD;
    state.n_stdev = Number.isFinite(stdevV) && stdevV > 0 ? stdevV : DEFAULT_N_STDEV;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.closes, state.period, state.n_stdev);
    renderSummary(local, true);
    renderChart(local);
    renderZoneChart(local);
    renderStats();
    let resp;
    try {
        resp = await api.anlyBollingerPercentB(buildBody(state));
    } catch (e) {
        showErr(`${t('view.bbpb.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!Array.isArray(resp)) { showErr(t('view.bbpb.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderZoneChart(resp);
    renderStats();
}

function renderSummary(pb, pending) {
    const local = localCompute(state.closes, state.period, state.n_stdev);
    let parityOk = Array.isArray(local) && Array.isArray(pb) && local.length === pb.length;
    if (parityOk) {
        for (let i = 0; i < local.length; i++) {
            const a = local[i], b = pb[i];
            if (a == null && b == null) continue;
            if (a == null || b == null || Math.abs(a - b) > 1e-6) { parityOk = false; break; }
        }
    }
    const last = lastDefined(pb);
    const zBadge = zoneBadge(last);
    const xBadge = crossBadge(pb);
    const tBadge = trendBadge(pb);
    const populated = countDefined(pb);
    const xValue = xBadge.barsAgo != null ? t('common.ago.bars_paren', { label: t(xBadge.key), n: xBadge.barsAgo }) : t(xBadge.key);
    const localTag = pending ? ` (${t('view.bbpb.tag.local')})` : '';
    document.getElementById('pb-summary').innerHTML = [
        card(t('view.bbpb.card.zone'),     t(zBadge.key) + localTag, zBadge.cls),
        card(t('view.bbpb.card.cross'),    xValue, xBadge.cls),
        card(t('view.bbpb.card.trend'),    t(tBadge.key), tBadge.cls),
        card(t('view.bbpb.card.last_pb'),  fmtPb(last),
             last == null ? '' : last >= 0.8 ? 'pos' : last <= 0.2 ? 'neg' : ''),
        card(t('view.bbpb.card.period'),   fmtInt(state.period)),
        card(t('view.bbpb.card.n_stdev'),  fmtPb(state.n_stdev, 2)),
        card(t('view.bbpb.card.populated'), `${populated} / ${state.closes.length}`),
        card(t('view.bbpb.card.parity'),
             parityOk ? t('view.bbpb.tag.ok') : t('view.bbpb.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(pb) {
    const el = document.getElementById('pb-chart');
    if (!el || !window.uPlot) return;
    const xs = state.closes.map((_, i) => i);
    const arr = pb.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const data = [xs, arr];
    if (chart) { try { chart.destroy(); } catch {} chart = null; }
    const opts = {
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.bbpb.series.pb'), stroke: '#1de9b6', width: 1.5 },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    };
    chart = new window.uPlot(opts, data, el);
}

function renderZoneChart(pb) {
    const el = document.getElementById('pb-zone-chart');
    if (!el || !window.uPlot) return;
    if (!Array.isArray(pb) || pb.length === 0) { el.innerHTML = ''; return; }
    const xs = pb.map((_, i) => i);
    const zone = pb.map(v => {
        if (v == null || !Number.isFinite(v)) return null;
        if (v >= 0.8) return 1;
        if (v <= 0.2) return -1;
        return 0;
    });
    const zero = xs.map(() => 0);
    const opts = {
        width: el.clientWidth || 800,
        height: 180,
        scales: { x: { time: false }, y: { range: [-1.5, 1.5] } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.bbpb.series.zone'),
              stroke: '#00e5ff', width: 1.5,
              points: { show: true, size: 5, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.bbpb.series.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    };
    if (zoneChart) { try { zoneChart.destroy(); } catch {} zoneChart = null; }
    zoneChart = new window.uPlot(opts, [xs, zone, zero], el);
}

function renderStats() {
    const wrap = document.getElementById('pb-stats');
    if (!state.closes.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.bbpb.empty">${esc(t('view.bbpb.empty'))}</div>`;
        return;
    }
    const s = summarizeCloses(state.closes);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.bbpb.col.metric">Metric</th>
                <th data-i18n="view.bbpb.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.bbpb.row.count">Closes</td><td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.bbpb.row.last">Last</td>   <td>${esc(fmtPrice(s.last))}</td></tr>
                <tr><td data-i18n="view.bbpb.row.min">Min</td>     <td>${esc(fmtPrice(s.min))}</td></tr>
                <tr><td data-i18n="view.bbpb.row.max">Max</td>     <td>${esc(fmtPrice(s.max))}</td></tr>
                <tr><td data-i18n="view.bbpb.row.mean">Mean</td>   <td>${esc(fmtPrice(s.mean))}</td></tr>
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
    const el = document.getElementById('pb-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('pb-err').style.display = 'none'; }
