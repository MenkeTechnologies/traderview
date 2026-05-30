// Accumulation Swing Index (Wilder) view.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_LIMIT_MOVE,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    trendBadge, breakoutBadge, biasBadge, summarizeBars,
    makeDemoInput,
    fmtSigned, fmtPrice, fmtInt,
} from '../_asi_inputs.js';

let state = { ...makeDemoInput('uptrend') };
let chart = null;

export async function renderAsi(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.asi.h1.title" class="view-title">// ACCUMULATION SWING INDEX</h1>

        <div class="chart-panel" data-context-scope="asi">
            <h2 data-i18n="view.asi.h2.bars">Bars
                <small data-i18n="view.asi.h2.bars_hint" class="muted">(4 tokens per line: open high low close)</small></h2>
            <textarea id="as-blob" rows="6"
                      data-tip="view.asi.tip.bars"
                      placeholder="100 101 99 100.5\n100.5 102 100 101.5\n...">${esc(barsToBlob(state.bars))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.asi.label.limit_move">Limit move</span>
                    <input id="as-limit" type="number" step="0.5" min="0.0001" value="${state.limit_move}"></label>
                <button data-i18n="view.asi.btn.compute" id="as-run" class="primary"
                        data-tip="view.asi.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.asi.btn.demo_up"      id="as-d1" class="secondary" type="button">Demo: uptrend</button>
                <button data-i18n="view.asi.btn.demo_down"    id="as-d2" class="secondary" type="button">Demo: downtrend</button>
                <button data-i18n="view.asi.btn.demo_side"    id="as-d3" class="secondary" type="button">Demo: sideways</button>
                <button data-i18n="view.asi.btn.demo_rev_up"  id="as-d4" class="secondary" type="button">Demo: reversal up</button>
                <button data-i18n="view.asi.btn.demo_rev_dn"  id="as-d5" class="secondary" type="button">Demo: reversal down</button>
                <button data-i18n="view.asi.btn.demo_wide"    id="as-d6" class="secondary" type="button">Demo: wide bars</button>
                <button data-i18n="view.asi.btn.demo_tight"   id="as-d7" class="secondary" type="button">Demo: tight limit (1)</button>
                <button data-i18n="view.asi.btn.demo_doji"    id="as-d8" class="secondary" type="button">Demo: doji-only</button>
            </div>
            <p data-i18n="view.asi.hint.about" class="muted">Wilder's cumulative Swing Index: quantifies "real" price moves with OHLC + prior bar reference. ASI breakouts of prior extremes confirm genuine trend changes. limit_move is the market's max allowed per-bar move (Wilder used futures limits; ~10% of prior close for equities).</p>
        </div>

        <div id="as-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.asi.h2.chart">Price + ASI overlay</h2>
            <div id="as-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.asi.h2.stats">Bar series summary</h2>
            <div id="as-stats"></div>
        </div>

        <div id="as-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('as-blob').value  = barsToBlob(state.bars);
        document.getElementById('as-limit').value = state.limit_move;
    };
    document.getElementById('as-d1').addEventListener('click', () => { loadDemo('uptrend');       void compute(tok); });
    document.getElementById('as-d2').addEventListener('click', () => { loadDemo('downtrend');     void compute(tok); });
    document.getElementById('as-d3').addEventListener('click', () => { loadDemo('sideways');      void compute(tok); });
    document.getElementById('as-d4').addEventListener('click', () => { loadDemo('reversal-up');   void compute(tok); });
    document.getElementById('as-d5').addEventListener('click', () => { loadDemo('reversal-down'); void compute(tok); });
    document.getElementById('as-d6').addEventListener('click', () => { loadDemo('wide-bars');     void compute(tok); });
    document.getElementById('as-d7').addEventListener('click', () => { loadDemo('tight-limit');   void compute(tok); });
    document.getElementById('as-d8').addEventListener('click', () => { loadDemo('flat-doji');     void compute(tok); });
    document.getElementById('as-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseBarsBlob(document.getElementById('as-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.asi.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.bars = p.bars;
    const limV = parseFloat(document.getElementById('as-limit').value);
    state.limit_move = Number.isFinite(limV) && limV > 0 ? limV : DEFAULT_LIMIT_MOVE;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.bars, state.limit_move);
    renderSummary(local, true);
    renderChart(local);
    renderStats();
    let resp;
    try {
        resp = await api.anlyAccumulationSwingIndex(buildBody(state));
    } catch (e) {
        showErr(`${t('view.asi.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!Array.isArray(resp)) { showErr(t('view.asi.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderStats();
}

function renderSummary(asi, pending) {
    const local = localCompute(state.bars, state.limit_move);
    let parityOk = Array.isArray(local) && Array.isArray(asi) && local.length === asi.length;
    if (parityOk) {
        for (let i = 0; i < local.length; i++) {
            const a = local[i], b = asi[i];
            if (a == null && b == null) continue;
            if (a == null || b == null || Math.abs(a - b) > 1e-6) { parityOk = false; break; }
        }
    }
    const last  = lastDefined(asi);
    const first = firstDefined(asi);
    const range = (last != null && first != null) ? last - first : NaN;
    const populated = countDefined(asi);
    const tBadge = trendBadge(asi);
    const bBadge = breakoutBadge(asi);
    const sBadge = biasBadge(last);
    const localTag = pending ? ` (${t('view.asi.tag.local')})` : '';
    document.getElementById('as-summary').innerHTML = [
        card(t('view.asi.card.trend'),    t(tBadge.key) + localTag, tBadge.cls),
        card(t('view.asi.card.breakout'), t(bBadge.key), bBadge.cls),
        card(t('view.asi.card.bias'),     t(sBadge.key), sBadge.cls),
        card(t('view.asi.card.last_asi'),  fmtSigned(last)),
        card(t('view.asi.card.first_asi'), fmtSigned(first)),
        card(t('view.asi.card.range'),     fmtSigned(range),
             range > 0 ? 'pos' : range < 0 ? 'neg' : ''),
        card(t('view.asi.card.limit'),     fmtPrice(state.limit_move, 4)),
        card(t('view.asi.card.populated'), `${populated} / ${state.bars.length}`),
        card(t('view.asi.card.parity'),
             parityOk ? t('view.asi.tag.ok') : t('view.asi.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(asi) {
    const el = document.getElementById('as-chart');
    if (!el || !window.uPlot) return;
    const xs = state.bars.map((_, i) => i);
    const closes = state.bars.map(b => b.close);
    const asiArr = asi.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const data = [xs, closes, asiArr];
    if (chart) { try { chart.destroy(); } catch {} chart = null; }
    const opts = {
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false }, y: {}, yAsi: {} },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.asi.series.close'), stroke: '#888',     width: 1, scale: 'y' },
            { label: t('view.asi.series.asi'),   stroke: '#1de9b6', width: 1.5, scale: 'yAsi' },
        ],
        axes: [
            { stroke: '#aaa' },
            { stroke: '#888', scale: 'y' },
            { stroke: '#1de9b6', scale: 'yAsi', side: 1 },
        ],
        legend: { show: true },
    };
    chart = new window.uPlot(opts, data, el);
}

function renderStats() {
    const wrap = document.getElementById('as-stats');
    if (!state.bars.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.asi.empty">${esc(t('view.asi.empty'))}</div>`;
        return;
    }
    const s = summarizeBars(state.bars);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.asi.col.metric">Metric</th>
                <th data-i18n="view.asi.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.asi.row.count">Bars</td>          <td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.asi.row.last">Last close</td>     <td>${esc(fmtPrice(s.last_close))}</td></tr>
                <tr><td data-i18n="view.asi.row.mean">Mean close</td>     <td>${esc(fmtPrice(s.mean_close))}</td></tr>
                <tr><td data-i18n="view.asi.row.minl">Min low</td>        <td>${esc(fmtPrice(s.min_low))}</td></tr>
                <tr><td data-i18n="view.asi.row.maxh">Max high</td>       <td>${esc(fmtPrice(s.max_high))}</td></tr>
                <tr><td data-i18n="view.asi.row.up_bars">Up bars</td>     <td class="pos">${fmtInt(s.up_bars)}</td></tr>
                <tr><td data-i18n="view.asi.row.down_bars">Down bars</td> <td class="neg">${fmtInt(s.down_bars)}</td></tr>
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

function firstDefined(arr) {
    if (!Array.isArray(arr)) return NaN;
    for (let i = 0; i < arr.length; i++) {
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
    const el = document.getElementById('as-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('as-err').style.display = 'none'; }
