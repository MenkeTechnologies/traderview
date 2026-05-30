// ATR Trailing Stop view — long & short stops trailed by N×ATR with ratchet.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_PERIOD, DEFAULT_MULTIPLIER, MIN_PERIOD, MAX_PERIOD,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    longBadge, shortBadge, regimeBadge, summarizeBars,
    makeDemoInput,
    fmtPrice, fmtPriceSigned, fmtPct, fmtInt,
} from '../_atr_trail_stop_inputs.js';

let state = { ...makeDemoInput('uptrend') };
let chart = null;
let gapChart = null;

export async function renderAtrTrailStop(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.atrts.h1.title" class="view-title">// ATR TRAILING STOP</h1>

        <div class="chart-panel" data-context-scope="atr-trailing-stop">
            <h2 data-i18n="view.atrts.h2.bars">Bars
                <small data-i18n="view.atrts.h2.bars_hint" class="muted">(3 tokens per line: high low close; ≥ period+1 bars)</small></h2>
            <textarea id="ts-blob" rows="6"
                      data-tip="view.atrts.tip.bars"
                      placeholder="101.5 99.5 100.5\n102.0 100.0 101.2\n...">${esc(barsToBlob(state.bars))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.atrts.label.period">Period</span>
                    <input id="ts-period" type="number" step="1" min="${MIN_PERIOD}" max="${MAX_PERIOD}" value="${state.period}"
                           data-tip="view.atrts.tip.period"></label>
                <label><span data-i18n="view.atrts.label.mult">Multiplier</span>
                    <input id="ts-mult" type="number" step="0.1" min="0.1" value="${state.multiplier}"
                           data-tip="view.atrts.tip.mult"></label>
                <button data-i18n="view.atrts.btn.compute" id="ts-run" class="primary"
                        data-tip="view.atrts.tip.compute" data-shortcut="atr_trailing_stop_run" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.atrts.btn.demo_up"     id="ts-d1" class="secondary" data-tip="view.atrts.tip.demo_up"    type="button">Demo: uptrend</button>
                <button data-i18n="view.atrts.btn.demo_down"   id="ts-d2" class="secondary" data-tip="view.atrts.tip.demo_down"  type="button">Demo: downtrend</button>
                <button data-i18n="view.atrts.btn.demo_side"   id="ts-d3" class="secondary" data-tip="view.atrts.tip.demo_side"  type="button">Demo: sideways</button>
                <button data-i18n="view.atrts.btn.demo_ltrig"  id="ts-d4" class="secondary" data-tip="view.atrts.tip.demo_ltrig" type="button">Demo: long-stop triggered</button>
                <button data-i18n="view.atrts.btn.demo_strig"  id="ts-d5" class="secondary" data-tip="view.atrts.tip.demo_strig" type="button">Demo: short-stop triggered</button>
                <button data-i18n="view.atrts.btn.demo_tight"  id="ts-d6" class="secondary" data-tip="view.atrts.tip.demo_tight" type="button">Demo: mult 1.0</button>
                <button data-i18n="view.atrts.btn.demo_wide"   id="ts-d7" class="secondary" data-tip="view.atrts.tip.demo_wide"  type="button">Demo: mult 5.0</button>
                <button data-i18n="view.atrts.btn.demo_flat"   id="ts-d8" class="secondary" data-tip="view.atrts.tip.demo_flat"  type="button">Demo: flat market</button>
            </div>
            <p data-i18n="view.atrts.hint.about" class="muted">Long stop = close − N×ATR, ratcheted up. Short stop = close + N×ATR, ratcheted down. Triggered when close crosses the stop. Defaults: period=14, mult=3.0. Companion to Chandelier Exit, Elder Safezone, Parabolic SAR.</p>
        </div>

        <div id="ts-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.atrts.h2.chart">Stops overlay</h2>
            <div id="ts-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.atrts.h2.gap_chart">Stop-gap % per bar (long &amp; short breathing room)</h2>
            <div id="ts-gap-chart" style="width:100%;height:220px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.atrts.h2.stats">Bar series summary</h2>
            <div id="ts-stats"></div>
        </div>

        <div id="ts-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('ts-blob').value   = barsToBlob(state.bars);
        document.getElementById('ts-period').value = state.period;
        document.getElementById('ts-mult').value   = state.multiplier;
    };
    document.getElementById('ts-d1').addEventListener('click', () => { loadDemo('uptrend');        void compute(tok); });
    document.getElementById('ts-d2').addEventListener('click', () => { loadDemo('downtrend');      void compute(tok); });
    document.getElementById('ts-d3').addEventListener('click', () => { loadDemo('sideways');       void compute(tok); });
    document.getElementById('ts-d4').addEventListener('click', () => { loadDemo('long-trigger');   void compute(tok); });
    document.getElementById('ts-d5').addEventListener('click', () => { loadDemo('short-trigger');  void compute(tok); });
    document.getElementById('ts-d6').addEventListener('click', () => { loadDemo('tight-mult');     void compute(tok); });
    document.getElementById('ts-d7').addEventListener('click', () => { loadDemo('wide-mult');      void compute(tok); });
    document.getElementById('ts-d8').addEventListener('click', () => { loadDemo('flat');           void compute(tok); });
    document.getElementById('ts-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseBarsBlob(document.getElementById('ts-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.atrts.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        showToast(t('view.atrts.toast.parse_error'), { level: 'error' });
        return;
    }
    hideErr();
    state.bars = p.bars;
    const periodV = parseInt(document.getElementById('ts-period').value, 10);
    const multV   = parseFloat(document.getElementById('ts-mult').value);
    state.period     = Number.isInteger(periodV) && periodV >= MIN_PERIOD && periodV <= MAX_PERIOD ? periodV : DEFAULT_PERIOD;
    state.multiplier = Number.isFinite(multV) && multV > 0 ? multV : DEFAULT_MULTIPLIER;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); showToast(t('view.atrts.toast.invalid'), { level: 'warning' }); return; }
    const local = localCompute(state.bars, state.period, state.multiplier);
    renderSummary(local, true);
    renderChart(local);
    renderGapChart(local);
    renderStats();
    let resp;
    try {
        resp = await api.anlyAtrTrailingStop(buildBody(state));
    } catch (e) {
        showErr(`${t('view.atrts.err.api')}: ${e.message || e}`);
        showToast(t('view.atrts.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp || !Array.isArray(resp.long_stop)) { showErr(t('view.atrts.err.server_rejected')); showToast(t('view.atrts.toast.server_rejected'), { level: 'error' }); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderGapChart(resp);
    renderStats();
    showToast(t('view.atrts.toast.computed'), { level: 'success' });
}

function renderSummary(report, pending) {
    const local = localCompute(state.bars, state.period, state.multiplier);
    let parityOk = Array.isArray(local.long_stop) && Array.isArray(report.long_stop)
        && local.long_stop.length === report.long_stop.length
        && local.short_stop.length === report.short_stop.length;
    if (parityOk) {
        for (let i = 0; i < local.long_stop.length; i++) {
            for (const key of ['long_stop', 'short_stop']) {
                const a = local[key][i], b = report[key][i];
                if (a == null && b == null) continue;
                if (a == null || b == null || Math.abs(a - b) > 1e-9) { parityOk = false; break; }
            }
            if (!parityOk) break;
        }
    }
    const lastClose = state.bars.length ? state.bars[state.bars.length - 1].close : NaN;
    const lastLong  = lastDefined(report.long_stop);
    const lastShort = lastDefined(report.short_stop);
    const lBadge = longBadge(lastClose, lastLong);
    const sBadge = shortBadge(lastClose, lastShort);
    const rBadge = regimeBadge(lastClose, lastLong, lastShort);
    const populated = countDefined(report.long_stop);
    const localTag = pending ? ` (${t('view.atrts.tag.local')})` : '';
    document.getElementById('ts-summary').innerHTML = [
        card(t('view.atrts.card.regime'),   t(rBadge.key) + localTag, rBadge.cls),
        card(t('view.atrts.card.long'),     t(lBadge.key), lBadge.cls),
        card(t('view.atrts.card.short'),    t(sBadge.key), sBadge.cls),
        card(t('view.atrts.card.last_close'), fmtPrice(lastClose)),
        card(t('view.atrts.card.last_long'),  fmtPrice(lastLong),  'pos'),
        card(t('view.atrts.card.last_short'), fmtPrice(lastShort), 'neg'),
        card(t('view.atrts.card.long_gap'),   fmtPriceSigned(lastClose - lastLong),
             lastClose > lastLong ? 'pos' : 'neg'),
        card(t('view.atrts.card.short_gap'),  fmtPriceSigned(lastShort - lastClose),
             lastClose < lastShort ? 'pos' : 'neg'),
        card(t('view.atrts.card.long_gap_pct'),
             fmtPct(lastLong  ? (lastClose - lastLong)  / lastLong  : NaN)),
        card(t('view.atrts.card.short_gap_pct'),
             fmtPct(lastShort ? (lastShort - lastClose) / lastShort : NaN)),
        card(t('view.atrts.card.period'),    fmtInt(state.period)),
        card(t('view.atrts.card.mult'),      fmtPrice(state.multiplier, 2)),
        card(t('view.atrts.card.populated'), `${populated} / ${state.bars.length}`),
        card(t('view.atrts.card.parity'),
             parityOk ? t('view.atrts.tag.ok') : t('view.atrts.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(report) {
    const el = document.getElementById('ts-chart');
    if (!el || !window.uPlot) return;
    const xs = state.bars.map((_, i) => i);
    const closes = state.bars.map(b => b.close);
    const longArr  = report.long_stop.map(v  => (v == null || !Number.isFinite(v) ? null : v));
    const shortArr = report.short_stop.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const data = [xs, closes, longArr, shortArr];
    if (chart) { try { chart.destroy(); } catch {} chart = null; }
    const opts = {
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.atrts.series.close'),     stroke: '#888',     width: 1 },
            { label: t('view.atrts.series.long_stop'),  stroke: '#1de9b6', width: 1.5 },
            { label: t('view.atrts.series.short_stop'), stroke: '#ff5252', width: 1.5 },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    };
    chart = new window.uPlot(opts, data, el);
}

function renderGapChart(report) {
    const el = document.getElementById('ts-gap-chart');
    if (!el || !window.uPlot) return;
    if (!Array.isArray(report.long_stop) || state.bars.length === 0) { el.innerHTML = ''; return; }
    const xs = state.bars.map((_, i) => i);
    const longGap = state.bars.map((b, i) => {
        const s = report.long_stop[i];
        if (s == null || !Number.isFinite(s) || !(s > 0)) return null;
        return ((b.close - s) / s) * 100;
    });
    const shortGap = state.bars.map((b, i) => {
        const s = report.short_stop[i];
        if (s == null || !Number.isFinite(s) || !(s > 0)) return null;
        return ((s - b.close) / s) * 100;
    });
    const zero = xs.map(() => 0);
    const opts = {
        width: el.clientWidth || 800,
        height: 200,
        scales: { x: { time: false } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.atrts.series.long_gap_pct'),
              stroke: '#7af0a8', width: 1.5, points: { show: false } },
            { label: t('view.atrts.series.short_gap_pct'),
              stroke: '#ff3860', width: 1.5, points: { show: false } },
            { label: t('view.atrts.series.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    };
    if (gapChart) { try { gapChart.destroy(); } catch {} gapChart = null; }
    gapChart = new window.uPlot(opts, [xs, longGap, shortGap, zero], el);
}

function renderStats() {
    const wrap = document.getElementById('ts-stats');
    if (!state.bars.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.atrts.empty">${esc(t('view.atrts.empty'))}</div>`;
        return;
    }
    const s = summarizeBars(state.bars);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.atrts.col.metric">Metric</th>
                <th data-i18n="view.atrts.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.atrts.row.count">Bars</td>      <td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.atrts.row.last">Last close</td> <td>${esc(fmtPrice(s.last_close))}</td></tr>
                <tr><td data-i18n="view.atrts.row.minl">Min low</td>    <td>${esc(fmtPrice(s.min_low))}</td></tr>
                <tr><td data-i18n="view.atrts.row.maxh">Max high</td>   <td>${esc(fmtPrice(s.max_high))}</td></tr>
                <tr><td data-i18n="view.atrts.row.mean">Mean close</td> <td>${esc(fmtPrice(s.mean_close))}</td></tr>
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
    const el = document.getElementById('ts-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ts-err').style.display = 'none'; }
