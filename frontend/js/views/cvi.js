// Chande Volatility Index (CVI) view.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_EMA, DEFAULT_ROC, MIN_EMA, MIN_ROC, MAX_PERIOD,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    regimeBadge, crossBadge, trendBadge, summarizeBars,
    makeDemoInput,
    fmtPct, fmtPctSigned, fmtPrice, fmtInt,
} from '../_cvi_inputs.js';

let state = { ...makeDemoInput('expanding') };
let chart = null;

export async function renderCvi(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.cvi.h1.title" class="view-title">// CHANDE VOLATILITY INDEX</h1>

        <div class="chart-panel" data-context-scope="chande-volatility-index">
            <h2 data-i18n="view.cvi.h2.bars">Bars
                <small data-i18n="view.cvi.h2.bars_hint" class="muted">(2 tokens per line: high low; ≥ ema_period + roc_period bars)</small></h2>
            <textarea id="cv-blob" rows="6"
                      data-tip="view.cvi.tip.bars"
                      placeholder="101 99\n102 100\n...">${esc(barsToBlob(state.bars))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.cvi.label.ema">EMA period</span>
                    <input id="cv-ema" type="number" step="1" min="${MIN_EMA}" max="${MAX_PERIOD}" value="${state.ema_period}"></label>
                <label><span data-i18n="view.cvi.label.roc">ROC period</span>
                    <input id="cv-roc" type="number" step="1" min="${MIN_ROC}" max="${MAX_PERIOD}" value="${state.roc_period}"></label>
                <button data-i18n="view.cvi.btn.compute" id="cv-run" class="primary"
                        data-tip="view.cvi.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.cvi.btn.demo_exp"     id="cv-d1" class="secondary" type="button">Demo: expanding</button>
                <button data-i18n="view.cvi.btn.demo_con"     id="cv-d2" class="secondary" type="button">Demo: contracting</button>
                <button data-i18n="view.cvi.btn.demo_steady"  id="cv-d3" class="secondary" type="button">Demo: steady</button>
                <button data-i18n="view.cvi.btn.demo_spike"   id="cv-d4" class="secondary" type="button">Demo: range spike</button>
                <button data-i18n="view.cvi.btn.demo_osc"     id="cv-d5" class="secondary" type="button">Demo: oscillating</button>
                <button data-i18n="view.cvi.btn.demo_long"    id="cv-d6" class="secondary" type="button">Demo: long EMA (25/25)</button>
                <button data-i18n="view.cvi.btn.demo_short_r" id="cv-d7" class="secondary" type="button">Demo: short ROC (3)</button>
                <button data-i18n="view.cvi.btn.demo_climax"  id="cv-d8" class="secondary" type="button">Demo: climax volatility</button>
            </div>
            <p data-i18n="view.cvi.hint.about" class="muted">Chande CVI = (EMA(range, ema_period) − EMA(range, ema_period) [roc_period bars ago]) / prev × 100. > 0 = expanding range; < 0 = contracting; |CVI| > 30 = notable shift. Distinct from Chaikin Volatility (different smoothing nuances). Defaults: 10 / 10.</p>
        </div>

        <div id="cv-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.cvi.h2.chart">CVI series</h2>
            <div id="cv-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.cvi.h2.stats">Bar series summary</h2>
            <div id="cv-stats"></div>
        </div>

        <div id="cv-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('cv-blob').value = barsToBlob(state.bars);
        document.getElementById('cv-ema').value  = state.ema_period;
        document.getElementById('cv-roc').value  = state.roc_period;
    };
    document.getElementById('cv-d1').addEventListener('click', () => { loadDemo('expanding');         void compute(tok); });
    document.getElementById('cv-d2').addEventListener('click', () => { loadDemo('contracting');       void compute(tok); });
    document.getElementById('cv-d3').addEventListener('click', () => { loadDemo('steady');            void compute(tok); });
    document.getElementById('cv-d4').addEventListener('click', () => { loadDemo('spike');             void compute(tok); });
    document.getElementById('cv-d5').addEventListener('click', () => { loadDemo('oscillating');       void compute(tok); });
    document.getElementById('cv-d6').addEventListener('click', () => { loadDemo('long-ema');          void compute(tok); });
    document.getElementById('cv-d7').addEventListener('click', () => { loadDemo('short-roc');         void compute(tok); });
    document.getElementById('cv-d8').addEventListener('click', () => { loadDemo('climax-volatility'); void compute(tok); });
    document.getElementById('cv-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseBarsBlob(document.getElementById('cv-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.cvi.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.bars = p.bars;
    const ev = parseInt(document.getElementById('cv-ema').value, 10);
    const rv = parseInt(document.getElementById('cv-roc').value, 10);
    state.ema_period = Number.isInteger(ev) && ev >= MIN_EMA && ev <= MAX_PERIOD ? ev : DEFAULT_EMA;
    state.roc_period = Number.isInteger(rv) && rv >= MIN_ROC && rv <= MAX_PERIOD ? rv : DEFAULT_ROC;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.bars, state.ema_period, state.roc_period);
    renderSummary(local, true);
    renderChart(local);
    renderStats();
    let resp;
    try {
        resp = await api.anlyChandeVolatilityIndex(buildBody(state));
    } catch (e) {
        showErr(`${t('view.cvi.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!Array.isArray(resp)) { showErr(t('view.cvi.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderStats();
}

function renderSummary(cvi, pending) {
    const local = localCompute(state.bars, state.ema_period, state.roc_period);
    let parityOk = Array.isArray(local) && Array.isArray(cvi) && local.length === cvi.length;
    if (parityOk) {
        for (let i = 0; i < local.length; i++) {
            const a = local[i], b = cvi[i];
            if (a == null && b == null) continue;
            if (a == null || b == null || Math.abs(a - b) > 1e-6) { parityOk = false; break; }
        }
    }
    const last = lastDefined(cvi);
    const rBadge = regimeBadge(last);
    const xBadge = crossBadge(cvi);
    const tBadge = trendBadge(cvi);
    const xValue = xBadge.barsAgo != null ? `${t(xBadge.key)} (${xBadge.barsAgo} bars ago)` : t(xBadge.key);
    const populated = countDefined(cvi);
    const localTag = pending ? ` (${t('view.cvi.tag.local')})` : '';
    document.getElementById('cv-summary').innerHTML = [
        card(t('view.cvi.card.regime'),    t(rBadge.key) + localTag, rBadge.cls),
        card(t('view.cvi.card.cross'),     xValue, xBadge.cls),
        card(t('view.cvi.card.trend'),     t(tBadge.key), tBadge.cls),
        card(t('view.cvi.card.last_cvi'),  fmtPctSigned(last),
             last == null ? '' : last > 10 ? 'neg' : last < -10 ? 'pos' : ''),
        card(t('view.cvi.card.ema'),       fmtInt(state.ema_period)),
        card(t('view.cvi.card.roc'),       fmtInt(state.roc_period)),
        card(t('view.cvi.card.populated'), `${populated} / ${state.bars.length}`),
        card(t('view.cvi.card.parity'),
             parityOk ? t('view.cvi.tag.ok') : t('view.cvi.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(cvi) {
    const el = document.getElementById('cv-chart');
    if (!el || !window.uPlot) return;
    const xs = state.bars.map((_, i) => i);
    const arr = cvi.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const data = [xs, arr];
    if (chart) { try { chart.destroy(); } catch {} chart = null; }
    chart = new window.uPlot({
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.cvi.series.cvi'), stroke: '#1de9b6', width: 1.5 },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    }, data, el);
}

function renderStats() {
    const wrap = document.getElementById('cv-stats');
    if (!state.bars.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.cvi.empty">${esc(t('view.cvi.empty'))}</div>`;
        return;
    }
    const s = summarizeBars(state.bars);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.cvi.col.metric">Metric</th>
                <th data-i18n="view.cvi.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.cvi.row.count">Bars</td>           <td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.cvi.row.mean_range">Mean range</td><td>${esc(fmtPrice(s.mean_range))}</td></tr>
                <tr><td data-i18n="view.cvi.row.minl">Min low</td>         <td>${esc(fmtPrice(s.min_low))}</td></tr>
                <tr><td data-i18n="view.cvi.row.maxh">Max high</td>        <td>${esc(fmtPrice(s.max_high))}</td></tr>
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
    const el = document.getElementById('cv-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('cv-err').style.display = 'none'; }
