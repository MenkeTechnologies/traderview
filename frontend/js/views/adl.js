// Accumulation / Distribution Line (Chaikin) view.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    trendBadge, divergenceBadge, phaseBadge, summarizeBars,
    makeDemoInput,
    fmtNum, fmtSigned, fmtPrice, fmtInt,
} from '../_adl_inputs.js';

let state = { ...makeDemoInput('accumulation') };
let chart = null;

export async function renderAdl(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.adl.h1.title" class="view-title">// ACCUMULATION / DISTRIBUTION LINE</h1>

        <div class="chart-panel" data-context-scope="adl">
            <h2 data-i18n="view.adl.h2.bars">Bars
                <small data-i18n="view.adl.h2.bars_hint" class="muted">(4 tokens per line: high low close volume)</small></h2>
            <textarea id="ad-blob" rows="6"
                      data-tip="view.adl.tip.bars"
                      placeholder="101 99 100.8 1500\n102 100 101.5 1800\n...">${esc(barsToBlob(state.bars))}</textarea>

            <div class="inline-form">
                <button data-i18n="view.adl.btn.compute" id="ad-run" class="primary"
                        data-tip="view.adl.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.adl.btn.demo_accum"   id="ad-d1" class="secondary" type="button">Demo: accumulation</button>
                <button data-i18n="view.adl.btn.demo_dist"    id="ad-d2" class="secondary" type="button">Demo: distribution</button>
                <button data-i18n="view.adl.btn.demo_bulldiv" id="ad-d3" class="secondary" type="button">Demo: bullish divergence</button>
                <button data-i18n="view.adl.btn.demo_beardiv" id="ad-d4" class="secondary" type="button">Demo: bearish divergence</button>
                <button data-i18n="view.adl.btn.demo_side"    id="ad-d5" class="secondary" type="button">Demo: sideways</button>
                <button data-i18n="view.adl.btn.demo_climax"  id="ad-d6" class="secondary" type="button">Demo: climax volume</button>
                <button data-i18n="view.adl.btn.demo_doji"    id="ad-d7" class="secondary" type="button">Demo: doji cluster</button>
                <button data-i18n="view.adl.btn.demo_small"   id="ad-d8" class="secondary" type="button">Demo: small-volume</button>
            </div>
            <p data-i18n="view.adl.hint.about" class="muted">Cumulative running sum of Money Flow Volume. MFM = ((C−L)−(H−C))/(H−L) ∈ [−1, +1]; MFV = MFM × Volume. Rising ADL = accumulation (closes near highs); falling ADL = distribution (closes near lows). Watch for price/ADL divergence.</p>
        </div>

        <div id="ad-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.adl.h2.chart">Price + ADL overlay</h2>
            <div id="ad-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.adl.h2.stats">Bar series summary</h2>
            <div id="ad-stats"></div>
        </div>

        <div id="ad-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('ad-blob').value = barsToBlob(state.bars);
    };
    document.getElementById('ad-d1').addEventListener('click', () => { loadDemo('accumulation');    void compute(tok); });
    document.getElementById('ad-d2').addEventListener('click', () => { loadDemo('distribution');    void compute(tok); });
    document.getElementById('ad-d3').addEventListener('click', () => { loadDemo('bull-divergence'); void compute(tok); });
    document.getElementById('ad-d4').addEventListener('click', () => { loadDemo('bear-divergence'); void compute(tok); });
    document.getElementById('ad-d5').addEventListener('click', () => { loadDemo('sideways');        void compute(tok); });
    document.getElementById('ad-d6').addEventListener('click', () => { loadDemo('climax-volume');   void compute(tok); });
    document.getElementById('ad-d7').addEventListener('click', () => { loadDemo('doji-cluster');    void compute(tok); });
    document.getElementById('ad-d8').addEventListener('click', () => { loadDemo('small-volume');    void compute(tok); });
    document.getElementById('ad-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseBarsBlob(document.getElementById('ad-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.adl.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.bars = p.bars;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.bars);
    renderSummary(local, true);
    renderChart(local);
    renderStats();
    let resp;
    try {
        resp = await api.anlyAdl(buildBody(state));
    } catch (e) {
        showErr(`${t('view.adl.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!Array.isArray(resp)) { showErr(t('view.adl.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderStats();
}

function renderSummary(adl, pending) {
    const local = localCompute(state.bars);
    let parityOk = Array.isArray(local) && Array.isArray(adl) && local.length === adl.length;
    if (parityOk) {
        for (let i = 0; i < local.length; i++) {
            const a = local[i], b = adl[i];
            if (a == null && b == null) continue;
            if (a == null || b == null || Math.abs(a - b) > 1e-6) { parityOk = false; break; }
        }
    }
    const last = lastDefined(adl);
    const first = firstDefined(adl);
    const range = last != null && first != null ? last - first : NaN;
    const lastClose = state.bars.length ? state.bars[state.bars.length - 1].close : NaN;
    const tBadge = trendBadge(adl);
    const dBadge = divergenceBadge(adl, state.bars);
    const pBadge = phaseBadge(last);
    const populated = countDefined(adl);
    const localTag = pending ? ` (${t('view.adl.tag.local')})` : '';
    document.getElementById('ad-summary').innerHTML = [
        card(t('view.adl.card.trend'),    t(tBadge.key) + localTag, tBadge.cls),
        card(t('view.adl.card.divergence'), t(dBadge.key), dBadge.cls),
        card(t('view.adl.card.phase'),    t(pBadge.key), pBadge.cls),
        card(t('view.adl.card.last_adl'), fmtSigned(last)),
        card(t('view.adl.card.first_adl'), fmtSigned(first)),
        card(t('view.adl.card.range'),    fmtSigned(range),
             range > 0 ? 'pos' : range < 0 ? 'neg' : ''),
        card(t('view.adl.card.last_close'), fmtPrice(lastClose)),
        card(t('view.adl.card.populated'), `${populated} / ${state.bars.length}`),
        card(t('view.adl.card.parity'),
             parityOk ? t('view.adl.tag.ok') : t('view.adl.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(adl) {
    const el = document.getElementById('ad-chart');
    if (!el || !window.uPlot) return;
    const xs = state.bars.map((_, i) => i);
    const closes = state.bars.map(b => b.close);
    const adlArr = adl.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const data = [xs, closes, adlArr];
    if (chart) { try { chart.destroy(); } catch {} chart = null; }
    const opts = {
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false }, y: {}, yAdl: {} },
        series: [
            { label: 'i' },
            { label: t('view.adl.series.close'), stroke: '#888',     width: 1, scale: 'y' },
            { label: t('view.adl.series.adl'),   stroke: '#1de9b6', width: 1.5, scale: 'yAdl' },
        ],
        axes: [
            { stroke: '#aaa' },
            { stroke: '#888', scale: 'y' },
            { stroke: '#1de9b6', scale: 'yAdl', side: 1 },
        ],
        legend: { show: true },
    };
    chart = new window.uPlot(opts, data, el);
}

function renderStats() {
    const wrap = document.getElementById('ad-stats');
    if (!state.bars.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.adl.empty">${esc(t('view.adl.empty'))}</div>`;
        return;
    }
    const s = summarizeBars(state.bars);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.adl.col.metric">Metric</th>
                <th data-i18n="view.adl.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.adl.row.count">Bars</td>            <td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.adl.row.last">Last close</td>       <td>${esc(fmtPrice(s.last_close))}</td></tr>
                <tr><td data-i18n="view.adl.row.tot_vol">Total volume</td>  <td>${esc(fmtNum(s.total_volume))}</td></tr>
                <tr><td data-i18n="view.adl.row.mean">Mean close</td>       <td>${esc(fmtPrice(s.mean_close))}</td></tr>
                <tr><td data-i18n="view.adl.row.minl">Min low</td>          <td>${esc(fmtPrice(s.min_low))}</td></tr>
                <tr><td data-i18n="view.adl.row.maxh">Max high</td>         <td>${esc(fmtPrice(s.max_high))}</td></tr>
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
    const el = document.getElementById('ad-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ad-err').style.display = 'none'; }
