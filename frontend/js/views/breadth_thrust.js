// Zweig Breadth Thrust view — classic market-bottom signal (1986).
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_EMA_PERIOD, DEFAULT_MAX_WINDOW, DEFAULT_LOW, DEFAULT_HIGH,
    parseBreadthBlob, breadthToBlob, validateInputs, buildBody, localCompute,
    regimeBadge, thrustBadge, lastThrustIndex, summarize,
    makeDemoInput,
    fmtRatio, fmtPct, fmtInt,
} from '../_breadth_thrust_inputs.js';

let state = { ...makeDemoInput('classic-thrust') };

export async function renderBreadthThrust(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.breadth.h1.title" class="view-title">// ZWEIG BREADTH THRUST</h1>

        <div class="chart-panel" data-context-scope="breadth-thrust">
            <h2 data-i18n="view.breadth.h2.days">Daily breadth
                <small data-i18n="view.breadth.h2.days_hint" class="muted">(per line: advancing declining — non-negative integers)</small></h2>
            <textarea id="bt-blob" rows="6"
                      data-tip="view.breadth.tip.days"
                      placeholder="30 70&#10;40 60&#10;90 10">${esc(breadthToBlob(state.breadth))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.breadth.label.ema">EMA period</span>
                    <input id="bt-ema" type="number" step="1" min="2" value="${state.ema_period}"
                           data-tip="view.breadth.tip.ema"></label>
                <label><span data-i18n="view.breadth.label.window">Max window bars</span>
                    <input id="bt-window" type="number" step="1" min="2" value="${state.max_window_bars}"
                           data-tip="view.breadth.tip.window"></label>
                <label><span data-i18n="view.breadth.label.low">Low threshold</span>
                    <input id="bt-low" type="number" step="0.01" min="0" max="1" value="${state.low_threshold}"
                           data-tip="view.breadth.tip.low"></label>
                <label><span data-i18n="view.breadth.label.high">High threshold</span>
                    <input id="bt-high" type="number" step="0.01" min="0" max="1" value="${state.high_threshold}"
                           data-tip="view.breadth.tip.high"></label>
                <button data-i18n="view.breadth.btn.compute" id="bt-run" class="primary"
                        data-tip="view.breadth.tip.compute" data-shortcut="breadth_thrust_run" type="button">Detect thrust</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.breadth.btn.demo_classic" id="bt-demo-classic" class="secondary" data-tip="view.breadth.tip.demo_classic" type="button">Demo: classic thrust</button>
                <button data-i18n="view.breadth.btn.demo_flat"    id="bt-demo-flat"    class="secondary" data-tip="view.breadth.tip.demo_flat"    type="button">Demo: flat balanced</button>
                <button data-i18n="view.breadth.btn.demo_slow"    id="bt-demo-slow"    class="secondary" data-tip="view.breadth.tip.demo_slow"    type="button">Demo: slow recovery (no thrust)</button>
                <button data-i18n="view.breadth.btn.demo_multi"   id="bt-demo-multi"   class="secondary" data-tip="view.breadth.tip.demo_multi"   type="button">Demo: 2 thrusts in series</button>
                <button data-i18n="view.breadth.btn.demo_washout" id="bt-demo-washout" class="secondary" data-tip="view.breadth.tip.demo_washout" type="button">Demo: deep washout</button>
                <button data-i18n="view.breadth.btn.demo_noisy"   id="bt-demo-noisy"   class="secondary" data-tip="view.breadth.tip.demo_noisy"   type="button">Demo: noisy walk</button>
                <button data-i18n="view.breadth.btn.demo_tight"   id="bt-demo-tight"   class="secondary" data-tip="view.breadth.tip.demo_tight"   type="button">Demo: tight window (3 bars)</button>
                <button data-i18n="view.breadth.btn.demo_custom"  id="bt-demo-custom"  class="secondary" data-tip="view.breadth.tip.demo_custom"  type="button">Demo: looser thresholds (0.45/0.55)</button>
            </div>
            <p data-i18n="view.breadth.hint.about" class="muted">ratio_t = adv/(adv+dec). EMA over ema_period bars. Thrust triggers when EMA went from below low_threshold to above high_threshold within max_window_bars. Zweig defaults: 10 / 10 / 0.40 / 0.615.</p>
        </div>

        <div id="bt-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.breadth.h2.chart">EMA breadth ratio + thresholds</h2>
            <div id="bt-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.breadth.h2.cum_chart">Cumulative advance − decline (A-D line)</h2>
            <div id="bt-cum-chart" style="width:100%;height:220px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.breadth.h2.table">Per-bar breadth (tail — last 30)</h2>
            <div id="bt-table"></div>
        </div>

        <div id="bt-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('bt-blob').value   = breadthToBlob(state.breadth);
        document.getElementById('bt-ema').value    = state.ema_period;
        document.getElementById('bt-window').value = state.max_window_bars;
        document.getElementById('bt-low').value    = state.low_threshold;
        document.getElementById('bt-high').value   = state.high_threshold;
    };
    document.getElementById('bt-demo-classic').addEventListener('click',  () => { loadDemo('classic-thrust');    void compute(tok); });
    document.getElementById('bt-demo-flat').addEventListener('click',     () => { loadDemo('flat-balanced');     void compute(tok); });
    document.getElementById('bt-demo-slow').addEventListener('click',     () => { loadDemo('slow-recovery');     void compute(tok); });
    document.getElementById('bt-demo-multi').addEventListener('click',    () => { loadDemo('multi-thrust');      void compute(tok); });
    document.getElementById('bt-demo-washout').addEventListener('click',  () => { loadDemo('washout-only');      void compute(tok); });
    document.getElementById('bt-demo-noisy').addEventListener('click',    () => { loadDemo('noisy-walk');        void compute(tok); });
    document.getElementById('bt-demo-tight').addEventListener('click',    () => { loadDemo('tight-window');      void compute(tok); });
    document.getElementById('bt-demo-custom').addEventListener('click',   () => { loadDemo('custom-thresholds'); void compute(tok); });
    document.getElementById('bt-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseBreadthBlob(document.getElementById('bt-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.breadth.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        showToast(t('view.breadth.toast.parse_error'), { level: 'error' });
        return;
    }
    hideErr();
    state.breadth         = p.breadth;
    const ema    = parseInt(document.getElementById('bt-ema').value, 10);
    const window = parseInt(document.getElementById('bt-window').value, 10);
    const low    = Number(document.getElementById('bt-low').value);
    const high   = Number(document.getElementById('bt-high').value);
    state.ema_period      = Number.isInteger(ema) && ema >= 2 ? ema : DEFAULT_EMA_PERIOD;
    state.max_window_bars = Number.isInteger(window) && window >= 2 ? window : DEFAULT_MAX_WINDOW;
    state.low_threshold   = Number.isFinite(low) && low >= 0 && low <= 1 ? low : DEFAULT_LOW;
    state.high_threshold  = Number.isFinite(high) && high >= 0 && high <= 1 ? high : DEFAULT_HIGH;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); showToast(t('view.breadth.toast.invalid'), { level: 'warning' }); return; }
    const local = localCompute(state.breadth, state.ema_period, state.max_window_bars,
        state.low_threshold, state.high_threshold);
    renderSummary(local, true);
    renderChart(local);
    renderCumChart();
    renderTable(local);
    let resp;
    try {
        resp = await api.anlyBreadthThrust(buildBody(state));
    } catch (e) {
        showErr(`${t('view.breadth.err.api')}: ${e.message || e}`);
        showToast(t('view.breadth.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(resp);
    renderCumChart();
    renderTable(resp);
    if (resp && resp.thrust_triggered) {
        showToast(t('view.breadth.toast.thrust'), { level: 'success' });
    } else {
        showToast(t('view.breadth.toast.computed'), { level: 'info' });
    }
}

function renderSummary(report, pending) {
    const local = localCompute(state.breadth, state.ema_period, state.max_window_bars,
        state.low_threshold, state.high_threshold);
    const parityOk = report.ratio.length === local.ratio.length
        && report.ema_ratio.every((v, i) => {
            if (v == null && local.ema_ratio[i] == null) return true;
            if (v == null || local.ema_ratio[i] == null) return false;
            return Math.abs(v - local.ema_ratio[i]) < 1e-9;
        })
        && report.thrust_triggered.every((v, i) => v === local.thrust_triggered[i]);
    const s = summarize(report);
    const rBadge = regimeBadge(s.last_ema, state.low_threshold, state.high_threshold);
    const tBadgeR = thrustBadge(report.thrust_triggered);
    const lastIdx = lastThrustIndex(report.thrust_triggered);
    const localTag = pending ? ` (${t('view.breadth.tag.local')})` : '';
    const lastIdxLabel = lastIdx == null ? t('view.breadth.tag.no_trigger') : '#' + (lastIdx + 1);
    document.getElementById('bt-summary').innerHTML = [
        card(t('view.breadth.card.verdict'),       t(rBadge.key) + localTag, rBadge.cls),
        card(t('view.breadth.card.thrust'),        t(tBadgeR.key), tBadgeR.cls),
        card(t('view.breadth.card.days'),          fmtInt(s.count)),
        card(t('view.breadth.card.populated'),     fmtInt(s.populated)),
        card(t('view.breadth.card.last_ratio'),    fmtPct(s.last_ratio)),
        card(t('view.breadth.card.last_ema'),      fmtRatio(s.last_ema), rBadge.cls),
        card(t('view.breadth.card.min_ema'),       fmtRatio(s.min_ema)),
        card(t('view.breadth.card.max_ema'),       fmtRatio(s.max_ema)),
        card(t('view.breadth.card.thrust_count'),  fmtInt(s.thrust_count),
             s.thrust_count > 0 ? 'pos' : ''),
        card(t('view.breadth.card.last_thrust'),   lastIdxLabel,
             lastIdx != null ? 'pos' : ''),
        card(t('view.breadth.card.parity'),
             parityOk ? t('view.breadth.tag.ok') : t('view.breadth.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(report) {
    if (!window.uPlot) return;
    const el = document.getElementById('bt-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!report.ratio || report.ratio.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.breadth.empty">${esc(t('view.breadth.empty'))}</div>`;
        return;
    }
    const n = report.ratio.length;
    const xs = report.ratio.map((_, i) => i);
    const lows = new Array(n).fill(state.low_threshold);
    const highs = new Array(n).fill(state.high_threshold);
    // Marker series for triggers — point at trigger bar, null elsewhere.
    const peak = state.high_threshold + 0.05;
    const markers = report.thrust_triggered.map(v => v ? peak : null);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 340,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('chart.series.bar') },
            { label: t('chart.series.ratio'),    stroke: '#aab',     width: 1.0, dash: [2, 2], points: { show: false } },
            { label: t('chart.series.ema'),      stroke: '#00e5ff', width: 1.5, points: { show: false } },
            { label: t('chart.series.low'),      stroke: '#ff3860', width: 1.0, dash: [4, 4], points: { show: false } },
            { label: t('chart.series.high'),     stroke: '#3ad96b', width: 1.0, dash: [4, 4], points: { show: false } },
            { label: t('chart.series.thrust'),   stroke: '#ffd84a', width: 0,                  points: { show: true, size: 8 } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 60,
              values: (_u, splits) => splits.map(v => v.toFixed(2)) },
        ],
        legend: { show: true },
    }, [xs, report.ratio, report.ema_ratio, lows, highs, markers], el);
}

function renderCumChart() {
    if (!window.uPlot) return;
    const el = document.getElementById('bt-cum-chart');
    if (!el) return;
    el.innerHTML = '';
    const breadth = state.breadth || [];
    if (breadth.length < 2) {
        el.innerHTML = `<div class="muted" data-i18n="view.breadth.empty_cum">${esc(t('view.breadth.empty_cum'))}</div>`;
        return;
    }
    const xs = breadth.map((_, i) => i);
    let acc = 0;
    const cum = breadth.map(b => { acc += (b.advancing - b.declining); return acc; });
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('chart.series.bar') },
            { label: t('view.breadth.chart.ad_cum'),
              stroke: '#7af0a8', width: 1.5, points: { show: false } },
            { label: t('view.breadth.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, cum, zero], el);
}

function renderTable(report) {
    const wrap = document.getElementById('bt-table');
    const n = report.ratio?.length || 0;
    if (n === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.breadth.empty">${esc(t('view.breadth.empty'))}</div>`;
        return;
    }
    const start = Math.max(0, n - 30);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.breadth.col.idx">#</th>
                <th data-i18n="view.breadth.col.adv">Adv</th>
                <th data-i18n="view.breadth.col.dec">Dec</th>
                <th data-i18n="view.breadth.col.ratio">Ratio</th>
                <th data-i18n="view.breadth.col.ema">EMA</th>
                <th data-i18n="view.breadth.col.thrust">Thrust</th>
            </tr></thead>
            <tbody>
                ${Array.from({ length: n - start }, (_, k) => {
                    const i = start + k;
                    const b = state.breadth[i];
                    const fired = report.thrust_triggered[i];
                    const cls = fired ? 'pos' : '';
                    const key = fired ? 'view.breadth.cell.fired' : 'view.breadth.cell.no';
                    return `<tr>
                        <td>${i + 1}</td>
                        <td class="pos">${esc(fmtInt(b?.advancing))}</td>
                        <td class="neg">${esc(fmtInt(b?.declining))}</td>
                        <td>${esc(fmtPct(report.ratio[i]))}</td>
                        <td>${esc(fmtRatio(report.ema_ratio[i]))}</td>
                        <td data-i18n="${esc(key)}" class="${cls}">${esc(t(key))}</td>
                    </tr>`;
                }).join('')}
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

function showErr(msg) {
    const el = document.getElementById('bt-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('bt-err').style.display = 'none'; }
