// Accumulation/Distribution Oscillator view (per-bar CLV×Vol + EMA).
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_PERIOD, MIN_PERIOD, MAX_PERIOD,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    pressureBadge, crossBadge, trendBadge, summarizeBars,
    makeDemoInput,
    fmtNum, fmtSigned, fmtPrice, fmtInt,
} from '../_ad_osc_inputs.js';

let state = { ...makeDemoInput('buying') };
let chart = null;

export async function renderAdOscillator(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.ado.h1.title" class="view-title">// A/D OSCILLATOR</h1>

        <div class="chart-panel" data-context-scope="ad-oscillator">
            <h2 data-i18n="view.ado.h2.bars">Bars
                <small data-i18n="view.ado.h2.bars_hint" class="muted">(4 tokens per line: high low close volume; ≥ period bars)</small></h2>
            <textarea id="ao-blob" rows="6"
                      data-tip="view.ado.tip.bars"
                      placeholder="101 99 100.8 1500\n102 100 101.5 1800\n...">${esc(barsToBlob(state.bars))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.ado.label.period">Period (EMA)</span>
                    <input id="ao-period" type="number" step="1" min="${MIN_PERIOD}" max="${MAX_PERIOD}" value="${state.period}"></label>
                <button data-i18n="view.ado.btn.compute" id="ao-run" class="primary"
                        data-tip="view.ado.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.ado.btn.demo_buy"     id="ao-d1" class="secondary" type="button">Demo: buying pressure</button>
                <button data-i18n="view.ado.btn.demo_sell"    id="ao-d2" class="secondary" type="button">Demo: selling pressure</button>
                <button data-i18n="view.ado.btn.demo_neut"    id="ao-d3" class="secondary" type="button">Demo: neutral</button>
                <button data-i18n="view.ado.btn.demo_xup"     id="ao-d4" class="secondary" type="button">Demo: cross up</button>
                <button data-i18n="view.ado.btn.demo_xdn"     id="ao-d5" class="secondary" type="button">Demo: cross down</button>
                <button data-i18n="view.ado.btn.demo_climax"  id="ao-d6" class="secondary" type="button">Demo: climax buy</button>
                <button data-i18n="view.ado.btn.demo_zero"    id="ao-d7" class="secondary" type="button">Demo: zero-range</button>
                <button data-i18n="view.ado.btn.demo_short"   id="ao-d8" class="secondary" type="button">Demo: short period (5)</button>
            </div>
            <p data-i18n="view.ado.hint.about" class="muted">Per-bar value = CLV × Volume, where CLV = ((C−L)−(H−C))/(H−L) ∈ [−1, +1]. EMA(period) smooths the per-bar series. Reads as "current buying pressure" — distinct from cumulative ADL. Default period=14.</p>
        </div>

        <div id="ao-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.ado.h2.chart">Per-bar + EMA</h2>
            <div id="ao-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.ado.h2.stats">Bar series summary</h2>
            <div id="ao-stats"></div>
        </div>

        <div id="ao-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('ao-blob').value   = barsToBlob(state.bars);
        document.getElementById('ao-period').value = state.period;
    };
    document.getElementById('ao-d1').addEventListener('click', () => { loadDemo('buying');     void compute(tok); });
    document.getElementById('ao-d2').addEventListener('click', () => { loadDemo('selling');    void compute(tok); });
    document.getElementById('ao-d3').addEventListener('click', () => { loadDemo('neutral');    void compute(tok); });
    document.getElementById('ao-d4').addEventListener('click', () => { loadDemo('cross-up');   void compute(tok); });
    document.getElementById('ao-d5').addEventListener('click', () => { loadDemo('cross-down'); void compute(tok); });
    document.getElementById('ao-d6').addEventListener('click', () => { loadDemo('climax-buy'); void compute(tok); });
    document.getElementById('ao-d7').addEventListener('click', () => { loadDemo('zero-range'); void compute(tok); });
    document.getElementById('ao-d8').addEventListener('click', () => { loadDemo('short-period'); void compute(tok); });
    document.getElementById('ao-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseBarsBlob(document.getElementById('ao-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.ado.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.bars = p.bars;
    const periodV = parseInt(document.getElementById('ao-period').value, 10);
    state.period = Number.isInteger(periodV) && periodV >= MIN_PERIOD && periodV <= MAX_PERIOD ? periodV : DEFAULT_PERIOD;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.bars, state.period);
    renderSummary(local, true);
    renderChart(local);
    renderStats();
    let resp;
    try {
        resp = await api.anlyAdOscillator(buildBody(state));
    } catch (e) {
        showErr(`${t('view.ado.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp || !Array.isArray(resp.per_bar)) { showErr(t('view.ado.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderStats();
}

function renderSummary(report, pending) {
    const local = localCompute(state.bars, state.period);
    let parityOk = Array.isArray(local.per_bar) && Array.isArray(report.per_bar)
        && local.per_bar.length === report.per_bar.length
        && local.ema.length === report.ema.length;
    if (parityOk) {
        for (let i = 0; i < local.per_bar.length; i++) {
            for (const key of ['per_bar', 'ema']) {
                const a = local[key][i], b = report[key][i];
                if (a == null && b == null) continue;
                if (a == null || b == null || Math.abs(a - b) > 1e-6) { parityOk = false; break; }
            }
            if (!parityOk) break;
        }
    }
    const lastPer = lastDefined(report.per_bar);
    const lastEma = lastDefined(report.ema);
    const meanVol = state.bars.length
        ? state.bars.reduce((s, b) => s + b.volume, 0) / state.bars.length : NaN;
    const pBadge = pressureBadge(lastEma, meanVol);
    const xBadge = crossBadge(report.ema);
    const tBadge = trendBadge(report.ema);
    const xValue = xBadge.barsAgo != null ? t('common.ago.bars_paren', { label: t(xBadge.key), n: xBadge.barsAgo }) : t(xBadge.key);
    const populated = countDefined(report.ema);
    const localTag = pending ? ` (${t('view.ado.tag.local')})` : '';
    document.getElementById('ao-summary').innerHTML = [
        card(t('view.ado.card.pressure'),  t(pBadge.key) + localTag, pBadge.cls),
        card(t('view.ado.card.cross'),     xValue, xBadge.cls),
        card(t('view.ado.card.trend'),     t(tBadge.key), tBadge.cls),
        card(t('view.ado.card.last_per'),  fmtSigned(lastPer)),
        card(t('view.ado.card.last_ema'),  fmtSigned(lastEma),
             lastEma > 0 ? 'pos' : lastEma < 0 ? 'neg' : ''),
        card(t('view.ado.card.mean_vol'),  fmtNum(meanVol)),
        card(t('view.ado.card.period'),    fmtInt(state.period)),
        card(t('view.ado.card.populated'), `${populated} / ${state.bars.length}`),
        card(t('view.ado.card.parity'),
             parityOk ? t('view.ado.tag.ok') : t('view.ado.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(report) {
    const el = document.getElementById('ao-chart');
    if (!el || !window.uPlot) return;
    const xs = state.bars.map((_, i) => i);
    const per = report.per_bar.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const ema = report.ema.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const data = [xs, per, ema];
    if (chart) { try { chart.destroy(); } catch {} chart = null; }
    const opts = {
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.ado.series.per_bar'), stroke: '#666', width: 1, points: { show: false } },
            { label: t('view.ado.series.ema'),     stroke: '#1de9b6', width: 2 },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    };
    chart = new window.uPlot(opts, data, el);
}

function renderStats() {
    const wrap = document.getElementById('ao-stats');
    if (!state.bars.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.ado.empty">${esc(t('view.ado.empty'))}</div>`;
        return;
    }
    const s = summarizeBars(state.bars);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.ado.col.metric">Metric</th>
                <th data-i18n="view.ado.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.ado.row.count">Bars</td>           <td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.ado.row.last">Last close</td>      <td>${esc(fmtPrice(s.last_close))}</td></tr>
                <tr><td data-i18n="view.ado.row.tot_vol">Total volume</td> <td>${esc(fmtNum(s.total_volume))}</td></tr>
                <tr><td data-i18n="view.ado.row.mean_vol">Mean volume</td> <td>${esc(fmtNum(s.mean_volume))}</td></tr>
                <tr><td data-i18n="view.ado.row.minl">Min low</td>         <td>${esc(fmtPrice(s.min_low))}</td></tr>
                <tr><td data-i18n="view.ado.row.maxh">Max high</td>        <td>${esc(fmtPrice(s.max_high))}</td></tr>
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
    const el = document.getElementById('ao-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ao-err').style.display = 'none'; }
