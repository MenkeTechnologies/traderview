// Hawkes intensity view — self-exciting point-process visualizer.
// Renders λ(t) over a query-time grid with event-burst overlay.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseTimesBlob, validateInputs, buildBody, localCompute, makeQueryGrid,
    stabilityBadge, clusteringRatio,
    makeDemoInput, fmtNum, fmtInt, fmtRatio,
} from '../_hawkes_intensity_inputs.js';

let state = { ...makeDemoInput('cluster-burst') };
let autoGrid = true;

export async function renderHawkesIntensity(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.hawkes.h1.title" class="view-title">// HAWKES INTENSITY</h1>

        <div class="chart-panel" data-context-scope="hawkes">
            <h2 data-i18n="view.hawkes.h2.events">Event times
                <small data-i18n="view.hawkes.h2.events_hint" class="muted">(one timestamp per line, sorted ascending)</small></h2>
            <textarea id="hk-events" rows="6"
                      data-tip="view.hawkes.tip.events"
                      placeholder="1&#10;1.2&#10;1.4">${esc(state.event_times.join('\n'))}</textarea>

            <h2 data-i18n="view.hawkes.h2.params">Parameters</h2>
            <div class="inline-form">
                <label><span data-i18n="view.hawkes.label.mu">μ (baseline)</span>
                    <input id="hk-mu" type="number" step="0.01" min="0" value="${state.params.baseline_mu}" data-tip="view.hawkes.tip.mu"></label>
                <label><span data-i18n="view.hawkes.label.alpha">α (excitation)</span>
                    <input id="hk-alpha" type="number" step="0.01" min="0" value="${state.params.excitation_alpha}" data-tip="view.hawkes.tip.alpha"></label>
                <label><span data-i18n="view.hawkes.label.beta">β (decay)</span>
                    <input id="hk-beta" type="number" step="0.01" min="0.001" value="${state.params.decay_beta}" data-tip="view.hawkes.tip.beta"></label>
                <label class="inline-check">
                    <input id="hk-auto" type="checkbox" ${autoGrid ? 'checked' : ''} data-tip="view.hawkes.tip.auto_grid">
                    <span data-i18n="view.hawkes.label.auto_grid">Auto query grid</span>
                </label>
                <button data-i18n="view.hawkes.btn.compute" id="hk-run" class="primary"
                        data-tip="view.hawkes.tip.compute" data-shortcut="hawkes_run" type="button">Compute λ(t)</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.hawkes.btn.demo_poisson"  id="hk-demo-poisson"  class="secondary" type="button" data-tip="view.hawkes.tip.demo_poisson">Demo: Poisson baseline</button>
                <button data-i18n="view.hawkes.btn.demo_cluster"  id="hk-demo-cluster"  class="secondary" type="button" data-tip="view.hawkes.tip.demo_cluster">Demo: cluster burst</button>
                <button data-i18n="view.hawkes.btn.demo_news"     id="hk-demo-news"     class="secondary" type="button" data-tip="view.hawkes.tip.demo_news">Demo: news burst (earnings)</button>
                <button data-i18n="view.hawkes.btn.demo_critical" id="hk-demo-critical" class="secondary" type="button" data-tip="view.hawkes.tip.demo_critical">Demo: critical (α/β=0.95)</button>
                <button data-i18n="view.hawkes.btn.demo_explosive" id="hk-demo-exp"     class="secondary" type="button" data-tip="view.hawkes.tip.demo_exp">Demo: explosive (α>β)</button>
                <button data-i18n="view.hawkes.btn.demo_long"     id="hk-demo-long"     class="secondary" type="button" data-tip="view.hawkes.tip.demo_long">Demo: long decay (persistent)</button>
                <button data-i18n="view.hawkes.btn.demo_fast"     id="hk-demo-fast"     class="secondary" type="button" data-tip="view.hawkes.tip.demo_fast">Demo: fast decay (spike+fade)</button>
                <button data-i18n="view.hawkes.btn.demo_none"     id="hk-demo-none"     class="secondary" type="button" data-tip="view.hawkes.tip.demo_none">Demo: no events (flat μ)</button>
            </div>
            <p data-i18n="view.hawkes.hint.about" class="muted">λ(t) = μ + Σ α·exp(−β(t − t_i)). Branching ratio α/β: stable &lt; 1, critical near 1, explosive ≥ 1. Used in HFT trade-clustering, news-cascade modeling.</p>
        </div>

        <div id="hk-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.hawkes.h2.chart">λ(t) over query grid (• = event timestamps)</h2>
            <div id="hk-chart" style="width:100%;height:320px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.hawkes.h2.inter_arrival_chart">Sorted inter-arrival gaps (smallest → largest)</h2>
            <div id="hk-inter-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.hawkes.hint.inter_arrival" class="muted">Sorted gaps between consecutive events. Pure Poisson with rate μ ≈ exponential distribution; Hawkes clustering bunches small gaps on the left. Yellow dashed line = expected mean gap 1/μ.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.hawkes.h2.table">Per-query intensities (tail — last 30)</h2>
            <div id="hk-table"></div>
        </div>

        <div id="hk-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        autoGrid = true;
        document.getElementById('hk-events').value = state.event_times.join('\n');
        document.getElementById('hk-mu').value     = state.params.baseline_mu;
        document.getElementById('hk-alpha').value  = state.params.excitation_alpha;
        document.getElementById('hk-beta').value   = state.params.decay_beta;
        document.getElementById('hk-auto').checked = true;
    };
    document.getElementById('hk-demo-poisson').addEventListener('click',  () => { loadDemo('poisson-baseline'); void compute(tok); });
    document.getElementById('hk-demo-cluster').addEventListener('click',  () => { loadDemo('cluster-burst');    void compute(tok); });
    document.getElementById('hk-demo-news').addEventListener('click',     () => { loadDemo('news-burst');       void compute(tok); });
    document.getElementById('hk-demo-critical').addEventListener('click', () => { loadDemo('critical');         void compute(tok); });
    document.getElementById('hk-demo-exp').addEventListener('click',      () => { loadDemo('explosive');        void compute(tok); });
    document.getElementById('hk-demo-long').addEventListener('click',     () => { loadDemo('long-decay');       void compute(tok); });
    document.getElementById('hk-demo-fast').addEventListener('click',     () => { loadDemo('fast-decay');       void compute(tok); });
    document.getElementById('hk-demo-none').addEventListener('click',     () => { loadDemo('no-events');        void compute(tok); });
    document.getElementById('hk-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const ev = parseTimesBlob(document.getElementById('hk-events').value);
    if (ev.errors.length) {
        showErr(`${t('view.hawkes.err.parse_prefix')}: `
            + ev.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        showToast(t('view.hawkes.toast.parse_error', { n: ev.errors.length }), { level: 'warning' });
        return;
    }
    hideErr();
    state.event_times = ev.times;
    state.params.baseline_mu      = Number(document.getElementById('hk-mu').value);
    state.params.excitation_alpha = Number(document.getElementById('hk-alpha').value);
    state.params.decay_beta       = Number(document.getElementById('hk-beta').value);
    autoGrid = document.getElementById('hk-auto').checked;
    if (autoGrid) state.query_times = makeQueryGrid(state.event_times, 200);
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); showToast(t('view.hawkes.toast.invalid'), { level: 'warning' }); return; }
    const local = localCompute(state.event_times, state.query_times, state.params);
    renderSummary(local, true);
    renderChart(local);
    renderInterArrivalChart();
    renderTable(local);
    let resp;
    try {
        resp = await api.microHawkesIntensity(buildBody(state));
    } catch (e) {
        showErr(`${t('view.hawkes.err.api')}: ${e.message || e}`);
        showToast(t('view.hawkes.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) {
        showErr(t('view.hawkes.err.server_rejected'));
        showToast(t('view.hawkes.toast.rejected'), { level: 'error' });
        return;
    }
    renderSummary(resp, false);
    renderChart(resp);
    renderInterArrivalChart();
    renderTable(resp);
    const ratio = clusteringRatio(state.params);
    const ratioStr = Number.isFinite(ratio) ? ratio.toFixed(3) : '—';
    const level = ratio >= 1 ? 'error' : ratio >= 0.9 ? 'warning' : 'success';
    showToast(t('view.hawkes.toast.computed', { stable: resp.is_stable ? 'stable' : 'EXPLOSIVE', ratio: ratioStr }), { level });
}

function renderSummary(report, pending) {
    const local = localCompute(state.event_times, state.query_times, state.params);
    const parityOk = !!local && report.intensities.length === local.intensities.length
        && report.intensities.every((v, i) => Math.abs(v - local.intensities[i]) < 1e-9)
        && report.is_stable === local.is_stable;
    const badge = stabilityBadge(state.params);
    const ratio = clusteringRatio(report.intensities, state.params.baseline_mu);
    const localTag = pending ? ` (${t('view.hawkes.tag.local')})` : '';
    const branching = state.params.excitation_alpha / state.params.decay_beta;
    document.getElementById('hk-summary').innerHTML = [
        card(t('view.hawkes.card.verdict'),    t(badge.key) + localTag, badge.cls),
        card(t('view.hawkes.card.events'),     fmtInt(state.event_times.length)),
        card(t('view.hawkes.card.queries'),    fmtInt(state.query_times.length)),
        card(t('view.hawkes.card.branching'),  fmtRatio(branching),
             branching >= 1 ? 'neg' : branching >= 0.9 ? 'neg' : ''),
        card(t('view.hawkes.card.stable'),
             report.is_stable ? t('view.hawkes.tag.yes') : t('view.hawkes.tag.no'),
             report.is_stable ? 'pos' : 'neg'),
        card(t('view.hawkes.card.mean'),
             Number.isFinite(report.unconditional_mean_intensity)
                ? fmtNum(report.unconditional_mean_intensity)
                : '∞'),
        card(t('view.hawkes.card.peak'),       fmtNum(Math.max(...report.intensities))),
        card(t('view.hawkes.card.clustering'), fmtRatio(ratio)),
        card(t('view.hawkes.card.parity'),
             parityOk ? t('view.hawkes.tag.ok') : t('view.hawkes.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(report) {
    if (!window.uPlot) return;
    const el = document.getElementById('hk-chart');
    if (!el || !report || !report.intensities.length) return;
    el.innerHTML = '';
    const xs = state.query_times;
    const ys = report.intensities;
    // Event-marker series: at each event timestamp, point at peak intensity.
    const peak = ys.length ? Math.max(...ys) : 0;
    const markerXs = state.event_times.filter(e => e >= xs[0] && e <= xs[xs.length - 1]);
    // Render two series on same x — but uPlot needs aligned x. Use shared grid
    // and place null elsewhere for markers.
    const evSet = new Set(markerXs);
    const marker = xs.map(x => evSet.has(x) ? peak : null);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 320,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('chart.series.t') },
            { label: 'λ(t)',  stroke: '#00e5ff', width: 1.5, points: { show: false } },
            { label: t('chart.series.event'), stroke: '#ff3860', width: 0,   points: { show: true, size: 6 } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 60 }],
        legend: { show: true },
    }, [xs, ys, marker], el);
}

function renderInterArrivalChart() {
    const el = document.getElementById('hk-inter-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const ev = state.event_times;
    if (!Array.isArray(ev) || ev.length < 2) {
        el.innerHTML = `<div class="muted" data-i18n="view.hawkes.empty_inter">${esc(t('view.hawkes.empty_inter'))}</div>`;
        return;
    }
    const gaps = [];
    for (let i = 1; i < ev.length; i++) gaps.push(ev[i] - ev[i - 1]);
    gaps.sort((a, b) => a - b);
    const xs = gaps.map((_, i) => i + 1);
    const mu = state.params.baseline_mu;
    const expected = Number.isFinite(mu) && mu > 0 ? xs.map(() => 1 / mu) : xs.map(() => null);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('chart.series.idx') },
            { label: t('view.hawkes.series.gap'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 6, fill: '#b86bff', stroke: '#b86bff' } },
            { label: t('view.hawkes.series.expected_gap'),
              stroke: '#ffd84a', width: 1, dash: [6, 4],
              points: { show: false } },
        ],
        axes: [{ stroke: '#aab', size: 28 }, { stroke: '#aab', size: 60 }],
        legend: { show: true },
    }, [xs, gaps, expected], el);
}

function renderTable(report) {
    const wrap = document.getElementById('hk-table');
    if (!report || !report.intensities.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.hawkes.empty">${esc(t('view.hawkes.empty'))}</div>`;
        return;
    }
    const mu = state.params.baseline_mu || 1;
    const start = Math.max(0, report.intensities.length - 30);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.hawkes.col.idx">#</th>
                <th data-i18n="view.hawkes.col.t">t</th>
                <th data-i18n="view.hawkes.col.intensity">λ(t)</th>
                <th data-i18n="view.hawkes.col.ratio">λ / μ</th>
            </tr></thead>
            <tbody>
                ${report.intensities.slice(start).map((v, idx) => {
                    const i = start + idx;
                    const r = v / mu;
                    const cls = r > 3 ? 'neg' : r > 1.5 ? '' : 'pos';
                    return `<tr>
                        <td>${i}</td>
                        <td>${esc(fmtNum(state.query_times[i], 3))}</td>
                        <td class="${cls}">${esc(fmtNum(v))}</td>
                        <td class="${cls}">${esc(fmtRatio(r))}</td>
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
    const el = document.getElementById('hk-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('hk-err').style.display = 'none'; }
