// Bid/Ask Volume Ratio view — rolling Σ bid / Σ ask order-flow imbalance.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_PERIOD, MIN_PERIOD, MAX_PERIOD,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    flowBadge, trendBadge, imbalanceBadge, summarizeBars,
    makeDemoInput,
    fmtNum, fmtRatio, fmtPct, fmtInt,
} from '../_bid_ask_vol_inputs.js';

let state = { ...makeDemoInput('balanced') };
let chart = null;

export async function renderBidAskVol(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.bavr.h1.title" class="view-title">// BID/ASK VOLUME RATIO</h1>

        <div class="chart-panel" data-context-scope="bid-ask-volume-ratio">
            <h2 data-i18n="view.bavr.h2.bars">Bars
                <small data-i18n="view.bavr.h2.bars_hint" class="muted">(2 tokens per line: bid_volume ask_volume; ≥ period bars)</small></h2>
            <textarea id="bv-blob" rows="6"
                      data-tip="view.bavr.tip.bars"
                      placeholder="1000 1100\n1200 950\n...">${esc(barsToBlob(state.bars))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.bavr.label.period">Period</span>
                    <input id="bv-period" type="number" step="1" min="${MIN_PERIOD}" max="${MAX_PERIOD}" value="${state.period}"></label>
                <button data-i18n="view.bavr.btn.compute" id="bv-run" class="primary"
                        data-tip="view.bavr.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.bavr.btn.demo_bal"   id="bv-d1" class="secondary" type="button">Demo: balanced</button>
                <button data-i18n="view.bavr.btn.demo_buy"   id="bv-d2" class="secondary" type="button">Demo: buy pressure</button>
                <button data-i18n="view.bavr.btn.demo_sell"  id="bv-d3" class="secondary" type="button">Demo: sell pressure</button>
                <button data-i18n="view.bavr.btn.demo_shft_buy"  id="bv-d4" class="secondary" type="button">Demo: shifting to buy</button>
                <button data-i18n="view.bavr.btn.demo_shft_sell" id="bv-d5" class="secondary" type="button">Demo: shifting to sell</button>
                <button data-i18n="view.bavr.btn.demo_hbuy"  id="bv-d6" class="secondary" type="button">Demo: heavy buy</button>
                <button data-i18n="view.bavr.btn.demo_hsell" id="bv-d7" class="secondary" type="button">Demo: heavy sell</button>
                <button data-i18n="view.bavr.btn.demo_short" id="bv-d8" class="secondary" type="button">Demo: short period (10)</button>
            </div>
            <p data-i18n="view.bavr.hint.about" class="muted">Rolling Σ bid / Σ ask over `period` bars (Lee-Ready classified trades). > 1.5 → sell pressure (sellers hitting bids). < 0.67 → buy pressure (buyers lifting offers). ≈ 1.0 balanced. Default period=60.</p>
        </div>

        <div id="bv-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.bavr.h2.chart">Ratio overlay</h2>
            <div id="bv-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.bavr.h2.stats">Bar series summary</h2>
            <div id="bv-stats"></div>
        </div>

        <div id="bv-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('bv-blob').value   = barsToBlob(state.bars);
        document.getElementById('bv-period').value = state.period;
    };
    document.getElementById('bv-d1').addEventListener('click', () => { loadDemo('balanced');      void compute(tok); });
    document.getElementById('bv-d2').addEventListener('click', () => { loadDemo('buy-pressure');  void compute(tok); });
    document.getElementById('bv-d3').addEventListener('click', () => { loadDemo('sell-pressure'); void compute(tok); });
    document.getElementById('bv-d4').addEventListener('click', () => { loadDemo('shifting-buy');  void compute(tok); });
    document.getElementById('bv-d5').addEventListener('click', () => { loadDemo('shifting-sell'); void compute(tok); });
    document.getElementById('bv-d6').addEventListener('click', () => { loadDemo('heavy-buy');     void compute(tok); });
    document.getElementById('bv-d7').addEventListener('click', () => { loadDemo('heavy-sell');    void compute(tok); });
    document.getElementById('bv-d8').addEventListener('click', () => { loadDemo('short-period');  void compute(tok); });
    document.getElementById('bv-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseBarsBlob(document.getElementById('bv-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.bavr.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.bars = p.bars;
    const periodV = parseInt(document.getElementById('bv-period').value, 10);
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
        resp = await api.anlyBidAskVolumeRatio(buildBody(state));
    } catch (e) {
        showErr(`${t('view.bavr.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!Array.isArray(resp)) { showErr(t('view.bavr.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderStats();
}

function renderSummary(ratios, pending) {
    const local = localCompute(state.bars, state.period);
    let parityOk = Array.isArray(local) && Array.isArray(ratios) && local.length === ratios.length;
    if (parityOk) {
        for (let i = 0; i < local.length; i++) {
            const a = local[i], b = ratios[i];
            if (a == null && b == null) continue;
            if (a == null || b == null || Math.abs(a - b) > 1e-9) { parityOk = false; break; }
        }
    }
    const last = lastDefined(ratios);
    const first = firstDefined(ratios);
    const fBadge = flowBadge(last);
    const tBadge = trendBadge(ratios);
    const iBadge = imbalanceBadge(last);
    const populated = countDefined(ratios);
    const localTag = pending ? ` (${t('view.bavr.tag.local')})` : '';
    document.getElementById('bv-summary').innerHTML = [
        card(t('view.bavr.card.flow'),     t(fBadge.key) + localTag, fBadge.cls),
        card(t('view.bavr.card.trend'),    t(tBadge.key), tBadge.cls),
        card(t('view.bavr.card.imbalance'), t(iBadge.key), iBadge.cls),
        card(t('view.bavr.card.last_ratio'),  fmtRatio(last),
             last > 1 ? 'neg' : last < 1 ? 'pos' : ''),
        card(t('view.bavr.card.first_ratio'), fmtRatio(first)),
        card(t('view.bavr.card.delta'),    fmtRatio(last - first),
             (last - first) > 0 ? 'neg' : (last - first) < 0 ? 'pos' : ''),
        card(t('view.bavr.card.period'),   fmtInt(state.period)),
        card(t('view.bavr.card.populated'), `${populated} / ${state.bars.length}`),
        card(t('view.bavr.card.parity'),
             parityOk ? t('view.bavr.tag.ok') : t('view.bavr.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(ratios) {
    const el = document.getElementById('bv-chart');
    if (!el || !window.uPlot) return;
    const xs = state.bars.map((_, i) => i);
    const arr = ratios.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const data = [xs, arr];
    if (chart) { try { chart.destroy(); } catch {} chart = null; }
    const opts = {
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.bavr.series.ratio'), stroke: '#1de9b6', width: 1.5 },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    };
    chart = new window.uPlot(opts, data, el);
}

function renderStats() {
    const wrap = document.getElementById('bv-stats');
    if (!state.bars.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.bavr.empty">${esc(t('view.bavr.empty'))}</div>`;
        return;
    }
    const s = summarizeBars(state.bars);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.bavr.col.metric">Metric</th>
                <th data-i18n="view.bavr.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.bavr.row.count">Bars</td>          <td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.bavr.row.total_bid">Total bid vol</td><td>${esc(fmtNum(s.total_bid))}</td></tr>
                <tr><td data-i18n="view.bavr.row.total_ask">Total ask vol</td><td>${esc(fmtNum(s.total_ask))}</td></tr>
                <tr><td data-i18n="view.bavr.row.total">Total vol</td>     <td>${esc(fmtNum(s.total_vol))}</td></tr>
                <tr><td data-i18n="view.bavr.row.mean_bid">Mean bid</td>   <td>${esc(fmtNum(s.mean_bid))}</td></tr>
                <tr><td data-i18n="view.bavr.row.mean_ask">Mean ask</td>   <td>${esc(fmtNum(s.mean_ask))}</td></tr>
                <tr><td data-i18n="view.bavr.row.lifetime">Lifetime ratio</td><td>${esc(fmtRatio(s.lifetime_ratio))}</td></tr>
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
    const el = document.getElementById('bv-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('bv-err').style.display = 'none'; }
