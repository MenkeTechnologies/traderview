// AlphaTrend view — ATR-trailing trend line with RSI gate.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_PERIOD, DEFAULT_MULTIPLIER, MIN_PERIOD, MAX_PERIOD,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    dirBadge, trendBadge, positionBadge, summarizeBars,
    makeDemoInput,
    fmtPrice, fmtPriceSigned, fmtPct, fmtInt, } from '../_alphatrend_inputs.js';

let state = { ...makeDemoInput('uptrend') };
let chart = null;
let dirChart = null;

export async function renderAlphatrend(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.atrend.h1.title" class="view-title">// ALPHATREND</h1>

        <div class="chart-panel" data-context-scope="alphatrend">
            <h2 data-i18n="view.atrend.h2.bars">Bars
                <small data-i18n="view.atrend.h2.bars_hint" class="muted">(3 tokens per line: high low close; ≥ period+1 bars)</small></h2>
            <textarea id="at-blob" rows="6"
                      data-tip="view.atrend.tip.bars"
                      placeholder="101.5 99.5 100.5\n102.0 100.0 101.2\n...">${esc(barsToBlob(state.bars))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.atrend.label.period">Period</span>
                    <input id="at-period" type="number" step="1" min="${MIN_PERIOD}" max="${MAX_PERIOD}" value="${state.period}"></label>
                <label><span data-i18n="view.atrend.label.mult">Multiplier</span>
                    <input id="at-mult" type="number" step="0.1" min="0.1" value="${state.multiplier}"></label>
                <button data-i18n="view.atrend.btn.compute" id="at-run" class="primary"
                        data-tip="view.atrend.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.atrend.btn.demo_up"     id="at-d1" class="secondary" type="button">Demo: uptrend</button>
                <button data-i18n="view.atrend.btn.demo_down"   id="at-d2" class="secondary" type="button">Demo: downtrend</button>
                <button data-i18n="view.atrend.btn.demo_rev_up" id="at-d3" class="secondary" type="button">Demo: reversal-up</button>
                <button data-i18n="view.atrend.btn.demo_rev_dn" id="at-d4" class="secondary" type="button">Demo: reversal-down</button>
                <button data-i18n="view.atrend.btn.demo_side"   id="at-d5" class="secondary" type="button">Demo: sideways</button>
                <button data-i18n="view.atrend.btn.demo_vol"    id="at-d6" class="secondary" type="button">Demo: volatile</button>
                <button data-i18n="view.atrend.btn.demo_hi_m"   id="at-d7" class="secondary" type="button">Demo: mult 3.0</button>
                <button data-i18n="view.atrend.btn.demo_lo_m"   id="at-d8" class="secondary" type="button">Demo: mult 0.3</button>
            </div>
            <p data-i18n="view.atrend.hint.about" class="muted">Trailing trend line built from ATR (SMA-of-TR) ratcheted up while RSI ≥ 50 and ratcheted down when RSI &lt; 50. Multiplier widens/tightens the trail. Defaults: period=14, mult=1.0. Companion to Supertrend, Parabolic SAR, Chandelier Exit.</p>
        </div>

        <div id="at-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.atrend.h2.chart">AlphaTrend overlay</h2>
            <div id="at-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.atrend.h2.dir_chart">Direction per bar (+1 bull / −1 bear)</h2>
            <div id="at-dir-chart" style="width:100%;height:160px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.atrend.h2.stats">Bar series summary</h2>
            <div id="at-stats"></div>
        </div>

        <div id="at-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('at-blob').value   = barsToBlob(state.bars);
        document.getElementById('at-period').value = state.period;
        document.getElementById('at-mult').value   = state.multiplier;
    };
    document.getElementById('at-d1').addEventListener('click', () => { loadDemo('uptrend');       void compute(tok); });
    document.getElementById('at-d2').addEventListener('click', () => { loadDemo('downtrend');     void compute(tok); });
    document.getElementById('at-d3').addEventListener('click', () => { loadDemo('reversal-up');   void compute(tok); });
    document.getElementById('at-d4').addEventListener('click', () => { loadDemo('reversal-down'); void compute(tok); });
    document.getElementById('at-d5').addEventListener('click', () => { loadDemo('sideways');      void compute(tok); });
    document.getElementById('at-d6').addEventListener('click', () => { loadDemo('volatile');      void compute(tok); });
    document.getElementById('at-d7').addEventListener('click', () => { loadDemo('high-mult');     void compute(tok); });
    document.getElementById('at-d8').addEventListener('click', () => { loadDemo('low-mult');      void compute(tok); });
    document.getElementById('at-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseBarsBlob(document.getElementById('at-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.atrend.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.bars = p.bars;
    const periodV = parseInt(document.getElementById('at-period').value, 10);
    const multV   = parseFloat(document.getElementById('at-mult').value);
    state.period     = Number.isInteger(periodV) && periodV >= MIN_PERIOD && periodV <= MAX_PERIOD ? periodV : DEFAULT_PERIOD;
    state.multiplier = Number.isFinite(multV) && multV > 0 ? multV : DEFAULT_MULTIPLIER;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.bars, state.period, state.multiplier);
    renderSummary(local, true);
    renderChart(local);
    renderDirChart(local);
    renderStats();
    let resp;
    try {
        resp = await api.anlyAlphatrend(buildBody(state));
    } catch (e) {
        showErr(`${t('view.atrend.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp || !Array.isArray(resp.alpha)) { showErr(t('view.atrend.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderDirChart(resp);
    renderStats();
}

function renderSummary(report, pending) {
    const local = localCompute(state.bars, state.period, state.multiplier);
    let parityOk = Array.isArray(local.alpha) && Array.isArray(report.alpha)
        && local.alpha.length === report.alpha.length
        && local.direction.length === report.direction.length;
    if (parityOk) {
        for (let i = 0; i < local.alpha.length; i++) {
            const a = local.alpha[i], b = report.alpha[i];
            if (a == null && b == null) continue;
            if (a == null || b == null || Math.abs(a - b) > 1e-9) { parityOk = false; break; }
        }
    }
    const dBadge = dirBadge(report.direction);
    const tBadge = trendBadge(report.direction);
    const lastClose = state.bars.length ? state.bars[state.bars.length - 1].close : NaN;
    const lastAlpha = lastDefined(report.alpha);
    const pBadge = positionBadge(lastClose, lastAlpha);
    const populated = countDefined(report.alpha);
    const flips = countFlips(report.direction);
    const localTag = pending ? ` (${t('view.atrend.tag.local')})` : '';
    document.getElementById('at-summary').innerHTML = [
        card(t('view.atrend.card.dir'),       t(dBadge.key) + localTag, dBadge.cls),
        card(t('view.atrend.card.trend'),     t(tBadge.key), tBadge.cls),
        card(t('view.atrend.card.position'),  t(pBadge.key), pBadge.cls),
        card(t('view.atrend.card.last_close'), fmtPrice(lastClose)),
        card(t('view.atrend.card.last_alpha'), fmtPrice(lastAlpha)),
        card(t('view.atrend.card.gap'),       fmtPriceSigned(lastClose - lastAlpha),
             lastClose > lastAlpha ? 'pos' : lastClose < lastAlpha ? 'neg' : ''),
        card(t('view.atrend.card.gap_pct'),   fmtPct(lastAlpha ? (lastClose - lastAlpha) / lastAlpha : NaN),
             lastClose > lastAlpha ? 'pos' : lastClose < lastAlpha ? 'neg' : ''),
        card(t('view.atrend.card.flips'),     fmtInt(flips)),
        card(t('view.atrend.card.period'),    fmtInt(state.period)),
        card(t('view.atrend.card.mult'),      fmtPrice(state.multiplier, 2)),
        card(t('view.atrend.card.populated'), `${populated} / ${state.bars.length}`),
        card(t('view.atrend.card.parity'),
             parityOk ? t('view.atrend.tag.ok') : t('view.atrend.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(report) {
    const el = document.getElementById('at-chart');
    if (!el || !window.uPlot) return;
    const xs = state.bars.map((_, i) => i);
    const closes = state.bars.map(b => b.close);
    const alpha = report.alpha.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const upLine   = report.alpha.map((v, i) => {
        if (v == null) return null;
        return report.direction[i] != null && report.direction[i] >= 0 ? v : null;
    });
    const downLine = report.alpha.map((v, i) => {
        if (v == null) return null;
        return report.direction[i] != null && report.direction[i] < 0 ? v : null;
    });
    const data = [xs, closes, alpha, upLine, downLine];
    if (chart) { try { chart.destroy(); } catch {} chart = null; }
    const opts = {
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.atrend.series.close'),  stroke: '#888',     width: 1 },
            { label: t('view.atrend.series.alpha'),  stroke: '#aaa',     width: 1.2 },
            { label: t('view.atrend.series.up'),     stroke: '#1de9b6', width: 2 },
            { label: t('view.atrend.series.down'),   stroke: '#ff5252', width: 2 },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    };
    chart = new window.uPlot(opts, data, el);
}

function renderDirChart(report) {
    const el = document.getElementById('at-dir-chart');
    if (!el || !window.uPlot) return;
    if (!Array.isArray(report.direction) || state.bars.length === 0) { el.innerHTML = ''; return; }
    const xs = state.bars.map((_, i) => i);
    const dir = report.direction.map(v => (v == null || !Number.isFinite(v) ? null : (v >= 0 ? 1 : -1)));
    const zero = xs.map(() => 0);
    const opts = {
        width: el.clientWidth || 800,
        height: 140,
        scales: { x: { time: false }, y: { range: [-1.5, 1.5] } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.atrend.series.direction'),
              stroke: '#00e5ff', width: 1.5,
              points: { show: true, size: 4, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.atrend.series.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    };
    if (dirChart) { try { dirChart.destroy(); } catch {} dirChart = null; }
    dirChart = new window.uPlot(opts, [xs, dir, zero], el);
}

function renderStats() {
    const wrap = document.getElementById('at-stats');
    if (!state.bars.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.atrend.empty">${esc(t('view.atrend.empty'))}</div>`;
        return;
    }
    const s = summarizeBars(state.bars);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.atrend.col.metric">Metric</th>
                <th data-i18n="view.atrend.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.atrend.row.count">Bars</td>          <td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.atrend.row.last">Last close</td>     <td>${esc(fmtPrice(s.last_close))}</td></tr>
                <tr><td data-i18n="view.atrend.row.minl">Min low</td>        <td>${esc(fmtPrice(s.min_low))}</td></tr>
                <tr><td data-i18n="view.atrend.row.maxh">Max high</td>       <td>${esc(fmtPrice(s.max_high))}</td></tr>
                <tr><td data-i18n="view.atrend.row.mean">Mean close</td>     <td>${esc(fmtPrice(s.mean_close))}</td></tr>
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

function countFlips(direction) {
    if (!Array.isArray(direction)) return 0;
    let flips = 0;
    let prev = null;
    for (const d of direction) {
        if (d == null || d === 0) continue;
        if (prev != null && Math.sign(d) !== Math.sign(prev)) flips++;
        prev = d;
    }
    return flips;
}

function showErr(msg) {
    const el = document.getElementById('at-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('at-err').style.display = 'none'; }
