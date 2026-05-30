// Chande-Kroll Stop view — two-pass volatility trailing stop.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_P, DEFAULT_X, DEFAULT_Q, MIN_PERIOD, MAX_PERIOD,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    regimeBadge, widthBadge, longTrendBadge, shortTrendBadge, summarizeBars,
    makeDemoInput,
    fmtPrice, fmtPriceSigned, fmtPct, fmtInt,
} from '../_cks_inputs.js';

let state = { ...makeDemoInput('uptrend') };
let chart = null;

export async function renderCks(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.cks.h1.title" class="view-title">// CHANDE-KROLL STOP</h1>

        <div class="chart-panel" data-context-scope="chande-kroll-stop">
            <h2 data-i18n="view.cks.h2.bars">Bars
                <small data-i18n="view.cks.h2.bars_hint" class="muted">(3 tokens per line: high low close; ≥ p + q bars)</small></h2>
            <textarea id="ck-blob" rows="6"
                      data-tip="view.cks.tip.bars"
                      placeholder="101 99 100.5\n102 100 101.2\n...">${esc(barsToBlob(state.bars))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.cks.label.p">p (ATR/HH/LL period)</span>
                    <input id="ck-p" type="number" step="1" min="${MIN_PERIOD}" max="${MAX_PERIOD}" value="${state.p}"></label>
                <label><span data-i18n="view.cks.label.x">x (ATR multiplier)</span>
                    <input id="ck-x" type="number" step="0.1" min="0.1" value="${state.x}"></label>
                <label><span data-i18n="view.cks.label.q">q (smoothing period)</span>
                    <input id="ck-q" type="number" step="1" min="${MIN_PERIOD}" max="${MAX_PERIOD}" value="${state.q}"></label>
                <button data-i18n="view.cks.btn.compute" id="ck-run" class="primary"
                        data-tip="view.cks.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.cks.btn.demo_flat"   id="ck-d1" class="secondary" type="button">Demo: flat market</button>
                <button data-i18n="view.cks.btn.demo_up"     id="ck-d2" class="secondary" type="button">Demo: uptrend</button>
                <button data-i18n="view.cks.btn.demo_down"   id="ck-d3" class="secondary" type="button">Demo: downtrend</button>
                <button data-i18n="view.cks.btn.demo_rev_up" id="ck-d4" class="secondary" type="button">Demo: reversal up</button>
                <button data-i18n="view.cks.btn.demo_rev_dn" id="ck-d5" class="secondary" type="button">Demo: reversal down</button>
                <button data-i18n="view.cks.btn.demo_high_x" id="ck-d6" class="secondary" type="button">Demo: high x (3.0)</button>
                <button data-i18n="view.cks.btn.demo_short"  id="ck-d7" class="secondary" type="button">Demo: short bars (5/4)</button>
                <button data-i18n="view.cks.btn.demo_vol"    id="ck-d8" class="secondary" type="button">Demo: volatile</button>
            </div>
            <p data-i18n="view.cks.hint.about" class="muted">Two-pass volatility trailing stop: long_stop = highest(HH(p) − x·ATR(p), q); short_stop = lowest(LL(p) + x·ATR(p), q). When long_stop crosses BELOW short_stop, long bias is active. Defaults: p=10, x=1.0, q=9. Companion to Parabolic SAR, Volatility Stop.</p>
        </div>

        <div id="ck-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.cks.h2.chart">Stops overlay</h2>
            <div id="ck-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.cks.h2.stats">Bar series summary</h2>
            <div id="ck-stats"></div>
        </div>

        <div id="ck-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('ck-blob').value = barsToBlob(state.bars);
        document.getElementById('ck-p').value    = state.p;
        document.getElementById('ck-x').value    = state.x;
        document.getElementById('ck-q').value    = state.q;
    };
    document.getElementById('ck-d1').addEventListener('click', () => { loadDemo('flat');          void compute(tok); });
    document.getElementById('ck-d2').addEventListener('click', () => { loadDemo('uptrend');       void compute(tok); });
    document.getElementById('ck-d3').addEventListener('click', () => { loadDemo('downtrend');     void compute(tok); });
    document.getElementById('ck-d4').addEventListener('click', () => { loadDemo('reversal-up');   void compute(tok); });
    document.getElementById('ck-d5').addEventListener('click', () => { loadDemo('reversal-down'); void compute(tok); });
    document.getElementById('ck-d6').addEventListener('click', () => { loadDemo('high-x');        void compute(tok); });
    document.getElementById('ck-d7').addEventListener('click', () => { loadDemo('short-bars');    void compute(tok); });
    document.getElementById('ck-d8').addEventListener('click', () => { loadDemo('volatile');      void compute(tok); });
    document.getElementById('ck-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseBarsBlob(document.getElementById('ck-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.cks.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.bars = p.bars;
    const pv = parseInt(document.getElementById('ck-p').value, 10);
    const xv = parseFloat(document.getElementById('ck-x').value);
    const qv = parseInt(document.getElementById('ck-q').value, 10);
    state.p = Number.isInteger(pv) && pv >= MIN_PERIOD && pv <= MAX_PERIOD ? pv : DEFAULT_P;
    state.x = Number.isFinite(xv) && xv > 0 ? xv : DEFAULT_X;
    state.q = Number.isInteger(qv) && qv >= MIN_PERIOD && qv <= MAX_PERIOD ? qv : DEFAULT_Q;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.bars, state.p, state.x, state.q);
    renderSummary(local, true);
    renderChart(local);
    renderStats();
    let resp;
    try {
        resp = await api.anlyChandeKrollStop(buildBody(state));
    } catch (e) {
        showErr(`${t('view.cks.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp || !Array.isArray(resp.long_stop)) { showErr(t('view.cks.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderStats();
}

function renderSummary(report, pending) {
    const local = localCompute(state.bars, state.p, state.x, state.q);
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
    const rBadge = regimeBadge(lastLong, lastShort, lastClose);
    const wBadge = widthBadge(lastLong, lastShort);
    const ltBadge = longTrendBadge(report.long_stop);
    const stBadge = shortTrendBadge(report.short_stop);
    const populated = countDefined(report.long_stop);
    const localTag = pending ? ` (${t('view.cks.tag.local')})` : '';
    document.getElementById('ck-summary').innerHTML = [
        card(t('view.cks.card.regime'),     t(rBadge.key) + localTag, rBadge.cls),
        card(t('view.cks.card.width'),      t(wBadge.key), wBadge.cls),
        card(t('view.cks.card.long_trend'), t(ltBadge.key), ltBadge.cls),
        card(t('view.cks.card.short_trend'), t(stBadge.key), stBadge.cls),
        card(t('view.cks.card.last_close'), fmtPrice(lastClose)),
        card(t('view.cks.card.last_long'),  fmtPrice(lastLong),  'pos'),
        card(t('view.cks.card.last_short'), fmtPrice(lastShort), 'neg'),
        card(t('view.cks.card.gap'),        fmtPriceSigned(lastShort - lastLong)),
        card(t('view.cks.card.p'),          fmtInt(state.p)),
        card(t('view.cks.card.x'),          fmtPrice(state.x, 2)),
        card(t('view.cks.card.q'),          fmtInt(state.q)),
        card(t('view.cks.card.populated'),  `${populated} / ${state.bars.length}`),
        card(t('view.cks.card.parity'),
             parityOk ? t('view.cks.tag.ok') : t('view.cks.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(report) {
    const el = document.getElementById('ck-chart');
    if (!el || !window.uPlot) return;
    const xs = state.bars.map((_, i) => i);
    const closes = state.bars.map(b => b.close);
    const longArr  = report.long_stop.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const shortArr = report.short_stop.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const data = [xs, closes, longArr, shortArr];
    if (chart) { try { chart.destroy(); } catch {} chart = null; }
    chart = new window.uPlot({
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.cks.series.close'),      stroke: '#888',     width: 1 },
            { label: t('view.cks.series.long_stop'),  stroke: '#1de9b6', width: 1.5 },
            { label: t('view.cks.series.short_stop'), stroke: '#ff5252', width: 1.5 },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    }, data, el);
}

function renderStats() {
    const wrap = document.getElementById('ck-stats');
    if (!state.bars.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.cks.empty">${esc(t('view.cks.empty'))}</div>`;
        return;
    }
    const s = summarizeBars(state.bars);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.cks.col.metric">Metric</th>
                <th data-i18n="view.cks.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.cks.row.count">Bars</td>      <td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.cks.row.last">Last close</td> <td>${esc(fmtPrice(s.last_close))}</td></tr>
                <tr><td data-i18n="view.cks.row.minl">Min low</td>    <td>${esc(fmtPrice(s.min_low))}</td></tr>
                <tr><td data-i18n="view.cks.row.maxh">Max high</td>   <td>${esc(fmtPrice(s.max_high))}</td></tr>
                <tr><td data-i18n="view.cks.row.mean">Mean close</td> <td>${esc(fmtPrice(s.mean_close))}</td></tr>
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
    const el = document.getElementById('ck-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ck-err').style.display = 'none'; }
