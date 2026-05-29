// Candle Strength Index (CSI) view — body-to-range ratio EMA.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_PERIOD, MIN_PERIOD, MAX_PERIOD,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    strengthBadge, trendBadge, crossBadge, summarizeBars,
    makeDemoInput,
    fmtRatio, fmtPrice, fmtPct, fmtInt,
} from '../_csi_inputs.js';

let state = { ...makeDemoInput('mixed') };
let chart = null;

export async function renderCsi(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.csi.h1.title" class="view-title">// CANDLE STRENGTH INDEX</h1>

        <div class="chart-panel" data-context-scope="candle-strength-index">
            <h2 data-i18n="view.csi.h2.bars">Bars
                <small data-i18n="view.csi.h2.bars_hint" class="muted">(4 tokens per line: open high low close)</small></h2>
            <textarea id="cs-blob" rows="6"
                      data-tip="view.csi.tip.bars"
                      placeholder="100 101 99 100.5\n100.5 102 100 101.5\n...">${esc(barsToBlob(state.bars))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.csi.label.period">Period (EMA)</span>
                    <input id="cs-period" type="number" step="1" min="${MIN_PERIOD}" max="${MAX_PERIOD}" value="${state.period}"></label>
                <button data-i18n="view.csi.btn.compute" id="cs-run" class="primary"
                        data-tip="view.csi.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.csi.btn.demo_mixed"  id="cs-d1" class="secondary" type="button">Demo: mixed</button>
                <button data-i18n="view.csi.btn.demo_green"  id="cs-d2" class="secondary" type="button">Demo: green marubozu</button>
                <button data-i18n="view.csi.btn.demo_red"    id="cs-d3" class="secondary" type="button">Demo: red marubozu</button>
                <button data-i18n="view.csi.btn.demo_doji"   id="cs-d4" class="secondary" type="button">Demo: doji cluster</button>
                <button data-i18n="view.csi.btn.demo_alt"    id="cs-d5" class="secondary" type="button">Demo: alternating ±1</button>
                <button data-i18n="view.csi.btn.demo_shift"  id="cs-d6" class="secondary" type="button">Demo: shifting bullish</button>
                <button data-i18n="view.csi.btn.demo_long"   id="cs-d7" class="secondary" type="button">Demo: long period (30)</button>
                <button data-i18n="view.csi.btn.demo_brk"    id="cs-d8" class="secondary" type="button">Demo: breakout up</button>
            </div>
            <p data-i18n="view.csi.hint.about" class="muted">Per-bar signed (close − open) / (high − low) ratio, smoothed by an EMA. Range [−1, +1]: +1 = green marubozu, −1 = red marubozu, 0 = doji or balanced. Persistent positive readings = sustained buying commitment. Default period=14.</p>
        </div>

        <div id="cs-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.csi.h2.chart">CSI series</h2>
            <div id="cs-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.csi.h2.stats">Bar series summary</h2>
            <div id="cs-stats"></div>
        </div>

        <div id="cs-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('cs-blob').value   = barsToBlob(state.bars);
        document.getElementById('cs-period').value = state.period;
    };
    document.getElementById('cs-d1').addEventListener('click', () => { loadDemo('mixed');            void compute(tok); });
    document.getElementById('cs-d2').addEventListener('click', () => { loadDemo('green-marubozu');   void compute(tok); });
    document.getElementById('cs-d3').addEventListener('click', () => { loadDemo('red-marubozu');     void compute(tok); });
    document.getElementById('cs-d4').addEventListener('click', () => { loadDemo('doji-cluster');     void compute(tok); });
    document.getElementById('cs-d5').addEventListener('click', () => { loadDemo('alternating');      void compute(tok); });
    document.getElementById('cs-d6').addEventListener('click', () => { loadDemo('shifting-bullish'); void compute(tok); });
    document.getElementById('cs-d7').addEventListener('click', () => { loadDemo('long-period');      void compute(tok); });
    document.getElementById('cs-d8').addEventListener('click', () => { loadDemo('breakout-up');      void compute(tok); });
    document.getElementById('cs-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseBarsBlob(document.getElementById('cs-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.csi.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.bars = p.bars;
    const periodV = parseInt(document.getElementById('cs-period').value, 10);
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
        resp = await api.anlyCandleStrengthIndex(buildBody(state));
    } catch (e) {
        showErr(`${t('view.csi.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!Array.isArray(resp)) { showErr(t('view.csi.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderStats();
}

function renderSummary(csi, pending) {
    const local = localCompute(state.bars, state.period);
    let parityOk = Array.isArray(local) && Array.isArray(csi) && local.length === csi.length;
    if (parityOk) {
        for (let i = 0; i < local.length; i++) {
            const a = local[i], b = csi[i];
            if (a == null && b == null) continue;
            if (a == null || b == null || Math.abs(a - b) > 1e-9) { parityOk = false; break; }
        }
    }
    const last = lastDefined(csi);
    const sBadge = strengthBadge(last);
    const tBadge = trendBadge(csi);
    const xBadge = crossBadge(csi);
    const xValue = xBadge.barsAgo != null ? `${t(xBadge.key)} (${xBadge.barsAgo} bars ago)` : t(xBadge.key);
    const populated = countDefined(csi);
    const localTag = pending ? ` (${t('view.csi.tag.local')})` : '';
    document.getElementById('cs-summary').innerHTML = [
        card(t('view.csi.card.strength'), t(sBadge.key) + localTag, sBadge.cls),
        card(t('view.csi.card.trend'),    t(tBadge.key), tBadge.cls),
        card(t('view.csi.card.cross'),    xValue, xBadge.cls),
        card(t('view.csi.card.last_csi'), fmtRatio(last),
             last == null ? '' : last > 0.15 ? 'pos' : last < -0.15 ? 'neg' : ''),
        card(t('view.csi.card.period'),   fmtInt(state.period)),
        card(t('view.csi.card.populated'), `${populated} / ${state.bars.length}`),
        card(t('view.csi.card.parity'),
             parityOk ? t('view.csi.tag.ok') : t('view.csi.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(csi) {
    const el = document.getElementById('cs-chart');
    if (!el || !window.uPlot) return;
    const xs = state.bars.map((_, i) => i);
    const arr = csi.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const data = [xs, arr];
    if (chart) { try { chart.destroy(); } catch {} chart = null; }
    chart = new window.uPlot({
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false }, y: { range: [-1.05, 1.05] } },
        series: [
            { label: 'i' },
            { label: t('view.csi.series.csi'), stroke: '#1de9b6', width: 1.5 },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    }, data, el);
}

function renderStats() {
    const wrap = document.getElementById('cs-stats');
    if (!state.bars.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.csi.empty">${esc(t('view.csi.empty'))}</div>`;
        return;
    }
    const s = summarizeBars(state.bars);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.csi.col.metric">Metric</th>
                <th data-i18n="view.csi.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.csi.row.count">Bars</td>          <td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.csi.row.last">Last close</td>     <td>${esc(fmtPrice(s.last_close))}</td></tr>
                <tr><td data-i18n="view.csi.row.minl">Min low</td>        <td>${esc(fmtPrice(s.min_low))}</td></tr>
                <tr><td data-i18n="view.csi.row.maxh">Max high</td>       <td>${esc(fmtPrice(s.max_high))}</td></tr>
                <tr><td data-i18n="view.csi.row.up_bars">Up bars</td>     <td class="pos">${fmtInt(s.up_bars)}</td></tr>
                <tr><td data-i18n="view.csi.row.down_bars">Down bars</td> <td class="neg">${fmtInt(s.down_bars)}</td></tr>
                <tr><td data-i18n="view.csi.row.doji_bars">Doji bars</td> <td>${fmtInt(s.doji_bars)}</td></tr>
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
    const el = document.getElementById('cs-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('cs-err').style.display = 'none'; }
