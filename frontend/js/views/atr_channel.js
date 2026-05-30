// ATR Channel view — EMA/SMA midline + Wilder-ATR upper/lower bands.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_PERIOD, DEFAULT_MULTIPLIER, MIN_PERIOD, MAX_PERIOD,
    parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    positionBadge, trendBadge, widthBadge, summarizeBars,
    makeDemoInput,
    fmtPrice, fmtPct, fmtInt,
} from '../_atr_channel_inputs.js';

let state = { ...makeDemoInput('uptrend') };
let chart = null;
let widthChart = null;

export async function renderAtrChannel(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.atrc.h1.title" class="view-title">// ATR CHANNEL</h1>

        <div class="chart-panel" data-context-scope="atr-channel">
            <h2 data-i18n="view.atrc.h2.bars">Bars
                <small data-i18n="view.atrc.h2.bars_hint" class="muted">(3 tokens per line: high low close; ≥ period+1 bars)</small></h2>
            <textarea id="ac-blob" rows="6"
                      data-tip="view.atrc.tip.bars"
                      placeholder="101.5 99.5 100.5\n102.0 100.0 101.2\n...">${esc(barsToBlob(state.bars))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.atrc.label.period">Period</span>
                    <input id="ac-period" type="number" step="1" min="${MIN_PERIOD}" max="${MAX_PERIOD}" value="${state.period}"></label>
                <label><span data-i18n="view.atrc.label.mult">Multiplier</span>
                    <input id="ac-mult" type="number" step="0.1" min="0.1" value="${state.multiplier}"></label>
                <label><span data-i18n="view.atrc.label.use_ema">Use EMA midline</span>
                    <input id="ac-ema" type="checkbox" ${state.use_ema ? 'checked' : ''}></label>
                <button data-i18n="view.atrc.btn.compute" id="ac-run" class="primary"
                        data-tip="view.atrc.tip.compute" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.atrc.btn.demo_up"     id="ac-d1" class="secondary" type="button">Demo: uptrend</button>
                <button data-i18n="view.atrc.btn.demo_down"   id="ac-d2" class="secondary" type="button">Demo: downtrend</button>
                <button data-i18n="view.atrc.btn.demo_vside"  id="ac-d3" class="secondary" type="button">Demo: volatile side</button>
                <button data-i18n="view.atrc.btn.demo_tside"  id="ac-d4" class="secondary" type="button">Demo: tight side</button>
                <button data-i18n="view.atrc.btn.demo_brup"   id="ac-d5" class="secondary" type="button">Demo: breakout up</button>
                <button data-i18n="view.atrc.btn.demo_brdn"   id="ac-d6" class="secondary" type="button">Demo: breakdown</button>
                <button data-i18n="view.atrc.btn.demo_sma"    id="ac-d7" class="secondary" type="button">Demo: SMA midline</button>
                <button data-i18n="view.atrc.btn.demo_wide"   id="ac-d8" class="secondary" type="button">Demo: wide bands (3.5×)</button>
            </div>
            <p data-i18n="view.atrc.hint.about" class="muted">Volatility envelope: middle = EMA/SMA(close, period); upper/lower = middle ± multiplier × Wilder-ATR(period). Width tracks volatility; close above upper signals breakout, below lower signals breakdown. Defaults: 20 / 2.0× / EMA.</p>
        </div>

        <div id="ac-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.atrc.h2.chart">ATR Channel overlay</h2>
            <div id="ac-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.atrc.h2.width_chart">Channel width % per bar (volatility evolution)</h2>
            <div id="ac-width-chart" style="width:100%;height:220px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.atrc.h2.stats">Bar series summary</h2>
            <div id="ac-stats"></div>
        </div>

        <div id="ac-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('ac-blob').value   = barsToBlob(state.bars);
        document.getElementById('ac-period').value = state.period;
        document.getElementById('ac-mult').value   = state.multiplier;
        document.getElementById('ac-ema').checked  = state.use_ema;
    };
    document.getElementById('ac-d1').addEventListener('click', () => { loadDemo('uptrend');       void compute(tok); });
    document.getElementById('ac-d2').addEventListener('click', () => { loadDemo('downtrend');     void compute(tok); });
    document.getElementById('ac-d3').addEventListener('click', () => { loadDemo('volatile-side'); void compute(tok); });
    document.getElementById('ac-d4').addEventListener('click', () => { loadDemo('tight-side');    void compute(tok); });
    document.getElementById('ac-d5').addEventListener('click', () => { loadDemo('breakout');      void compute(tok); });
    document.getElementById('ac-d6').addEventListener('click', () => { loadDemo('breakdown');     void compute(tok); });
    document.getElementById('ac-d7').addEventListener('click', () => { loadDemo('sma');           void compute(tok); });
    document.getElementById('ac-d8').addEventListener('click', () => { loadDemo('wide-bands');    void compute(tok); });
    document.getElementById('ac-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseBarsBlob(document.getElementById('ac-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.atrc.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.bars = p.bars;
    const periodV = parseInt(document.getElementById('ac-period').value, 10);
    const multV   = parseFloat(document.getElementById('ac-mult').value);
    state.period     = Number.isInteger(periodV) && periodV >= MIN_PERIOD && periodV <= MAX_PERIOD ? periodV : DEFAULT_PERIOD;
    state.multiplier = Number.isFinite(multV) && multV > 0 ? multV : DEFAULT_MULTIPLIER;
    state.use_ema    = !!document.getElementById('ac-ema').checked;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.bars, state.period, state.multiplier, state.use_ema);
    renderSummary(local, true);
    renderChart(local);
    renderWidthChart(local);
    renderStats();
    let resp;
    try {
        resp = await api.anlyAtrChannel(buildBody(state));
    } catch (e) {
        showErr(`${t('view.atrc.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp || !Array.isArray(resp.middle)) { showErr(t('view.atrc.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderWidthChart(resp);
    renderStats();
}

function renderSummary(report, pending) {
    const local = localCompute(state.bars, state.period, state.multiplier, state.use_ema);
    let parityOk = Array.isArray(local.middle) && Array.isArray(report.middle)
        && local.middle.length === report.middle.length
        && local.upper.length  === report.upper.length
        && local.lower.length  === report.lower.length;
    if (parityOk) {
        for (let i = 0; i < local.middle.length; i++) {
            for (const key of ['middle', 'upper', 'lower']) {
                const a = local[key][i], b = report[key][i];
                if (a == null && b == null) continue;
                if (a == null || b == null || Math.abs(a - b) > 1e-9) { parityOk = false; break; }
            }
            if (!parityOk) break;
        }
    }
    const lastClose  = state.bars.length ? state.bars[state.bars.length - 1].close : NaN;
    const lastMid    = lastDefined(report.middle);
    const lastUp     = lastDefined(report.upper);
    const lastLow    = lastDefined(report.lower);
    const pBadge = positionBadge(lastClose, lastUp, lastLow, lastMid);
    const tBadge = trendBadge(report.middle);
    const wBadge = widthBadge(lastUp, lastLow, lastMid);
    const populated = countDefined(report.middle);
    const widthPct = (lastUp != null && lastLow != null && lastMid != null && lastMid !== 0)
        ? (lastUp - lastLow) / Math.abs(lastMid) : NaN;
    const localTag = pending ? ` (${t('view.atrc.tag.local')})` : '';
    document.getElementById('ac-summary').innerHTML = [
        card(t('view.atrc.card.position'),  t(pBadge.key) + localTag, pBadge.cls),
        card(t('view.atrc.card.trend'),     t(tBadge.key), tBadge.cls),
        card(t('view.atrc.card.width'),     t(wBadge.key), wBadge.cls),
        card(t('view.atrc.card.last_close'), fmtPrice(lastClose)),
        card(t('view.atrc.card.last_mid'),   fmtPrice(lastMid)),
        card(t('view.atrc.card.last_upper'), fmtPrice(lastUp),  'pos'),
        card(t('view.atrc.card.last_lower'), fmtPrice(lastLow), 'neg'),
        card(t('view.atrc.card.width_pct'),  fmtPct(widthPct)),
        card(t('view.atrc.card.midline_kind'),
             state.use_ema ? t('view.atrc.kind.ema') : t('view.atrc.kind.sma')),
        card(t('view.atrc.card.period'),    fmtInt(state.period)),
        card(t('view.atrc.card.mult'),      fmtPrice(state.multiplier, 2)),
        card(t('view.atrc.card.populated'), `${populated} / ${state.bars.length}`),
        card(t('view.atrc.card.parity'),
             parityOk ? t('view.atrc.tag.ok') : t('view.atrc.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(report) {
    const el = document.getElementById('ac-chart');
    if (!el || !window.uPlot) return;
    const xs = state.bars.map((_, i) => i);
    const closes = state.bars.map(b => b.close);
    const mid = report.middle.map(v => (v == null || !Number.isFinite(v) ? null : v));
    const up  = report.upper.map(v  => (v == null || !Number.isFinite(v) ? null : v));
    const lo  = report.lower.map(v  => (v == null || !Number.isFinite(v) ? null : v));
    const data = [xs, closes, mid, up, lo];
    if (chart) { try { chart.destroy(); } catch {} chart = null; }
    const opts = {
        width: el.clientWidth || 800,
        height: 340,
        scales: { x: { time: false } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.atrc.series.close'), stroke: '#888',     width: 1 },
            { label: t('view.atrc.series.mid'),   stroke: '#ffd54f', width: 1.5 },
            { label: t('view.atrc.series.upper'), stroke: '#1de9b6', width: 1.5 },
            { label: t('view.atrc.series.lower'), stroke: '#ff5252', width: 1.5 },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    };
    chart = new window.uPlot(opts, data, el);
}

function renderWidthChart(report) {
    const el = document.getElementById('ac-width-chart');
    if (!el || !window.uPlot) return;
    if (!Array.isArray(report.middle) || state.bars.length === 0) { el.innerHTML = ''; return; }
    const xs = state.bars.map((_, i) => i);
    const widthPct = xs.map(i => {
        const m = report.middle[i], u = report.upper[i], l = report.lower[i];
        if (m == null || u == null || l == null || !(Math.abs(m) > 0)) return null;
        return ((u - l) / Math.abs(m)) * 100;
    });
    const populated = widthPct.filter(v => v != null);
    const mean = populated.length ? populated.reduce((s, v) => s + v, 0) / populated.length : 0;
    const meanLine = xs.map(() => mean);
    const opts = {
        width: el.clientWidth || 800,
        height: 200,
        scales: { x: { time: false } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.atrc.series.width_pct'),
              stroke: '#7af0a8', width: 1.5, points: { show: false } },
            { label: t('view.atrc.series.width_mean'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [{ stroke: '#aaa' }, { stroke: '#aaa' }],
        legend: { show: true },
    };
    if (widthChart) { try { widthChart.destroy(); } catch {} widthChart = null; }
    widthChart = new window.uPlot(opts, [xs, widthPct, meanLine], el);
}

function renderStats() {
    const wrap = document.getElementById('ac-stats');
    if (!state.bars.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.atrc.empty">${esc(t('view.atrc.empty'))}</div>`;
        return;
    }
    const s = summarizeBars(state.bars);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.atrc.col.metric">Metric</th>
                <th data-i18n="view.atrc.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.atrc.row.count">Bars</td>      <td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.atrc.row.last">Last close</td> <td>${esc(fmtPrice(s.last_close))}</td></tr>
                <tr><td data-i18n="view.atrc.row.minl">Min low</td>    <td>${esc(fmtPrice(s.min_low))}</td></tr>
                <tr><td data-i18n="view.atrc.row.maxh">Max high</td>   <td>${esc(fmtPrice(s.max_high))}</td></tr>
                <tr><td data-i18n="view.atrc.row.mean">Mean close</td> <td>${esc(fmtPrice(s.mean_close))}</td></tr>
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
    const el = document.getElementById('ac-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ac-err').style.display = 'none'; }
