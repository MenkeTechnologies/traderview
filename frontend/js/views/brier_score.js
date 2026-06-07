// Brier Score view — probabilistic forecast accuracy + Murphy decomposition.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_BINS,
    parsePairsBlob, pairsToBlob, validateInputs, buildBody, localCompute,
    brierBadge, skillScore, reliabilityBins,
    makeDemoInput,
    fmtBrier, fmtSkill, fmtPct, fmtInt,
} from '../_brier_score_inputs.js';

let state = { ...makeDemoInput('well-calibrated') };

export async function renderBrierScore(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.brier.h1.title" class="view-title">// BRIER SCORE</h1>

        <div class="chart-panel" data-context-scope="brier">
            <h2 data-i18n="view.brier.h2.pairs">Forecasts
                <small data-i18n="view.brier.h2.pairs_hint" class="muted">(per line: probability outcome — prob ∈ [0,1], outcome ∈ {0,1})</small></h2>
            <textarea id="br-blob" rows="6"
                      data-tip="view.brier.tip.pairs"
                      placeholder="0.75 1&#10;0.30 0">${esc(pairsToBlob(state.probabilities, state.outcomes))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.brier.label.bins">Bins</span>
                    <input id="br-bins" type="number" step="1" min="1" value="${state.n_bins}"></label>
                <button data-i18n="view.brier.btn.compute" id="br-run" class="primary"
                        data-tip="view.brier.tip.compute" type="button">Score</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.brier.btn.demo_perfect" id="br-demo-perf"   class="secondary" type="button">Demo: perfect forecast</button>
                <button data-i18n="view.brier.btn.demo_coin"    id="br-demo-coin"   class="secondary" type="button">Demo: coin-flip 0.5s</button>
                <button data-i18n="view.brier.btn.demo_calib"   id="br-demo-calib"  class="secondary" type="button">Demo: well-calibrated</button>
                <button data-i18n="view.brier.btn.demo_over"    id="br-demo-over"   class="secondary" type="button">Demo: overconfident</button>
                <button data-i18n="view.brier.btn.demo_under"   id="br-demo-under"  class="secondary" type="button">Demo: underconfident</button>
                <button data-i18n="view.brier.btn.demo_flip"    id="br-demo-flip"   class="secondary" type="button">Demo: flipped sign</button>
                <button data-i18n="view.brier.btn.demo_rare"    id="br-demo-rare"   class="secondary" type="button">Demo: rare event (5%)</button>
                <button data-i18n="view.brier.btn.demo_fine"    id="br-demo-fine"   class="secondary" type="button">Demo: fine bins (50)</button>
            </div>
            <p data-i18n="view.brier.hint.about" class="muted">BS = (1/N)·Σ(p−y)². Range [0, 1]: 0 = perfect, 0.25 = balanced coin-flip. Murphy: BS = Reliability − Resolution + Uncertainty. Skill = 1 − BS/Uncertainty (> 0 beats climatology).</p>
        </div>

        <div id="br-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.brier.h2.reliability">Reliability diagram (perfect calibration = identity line)</h2>
            <div id="br-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.brier.h2.hist_chart">Forecast histogram (counts per bin)</h2>
            <div id="br-hist-chart" style="width:100%;height:220px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.brier.h2.table">Per-bin calibration</h2>
            <div id="br-table"></div>
        </div>

        <div id="br-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('br-blob').value = pairsToBlob(state.probabilities, state.outcomes);
        document.getElementById('br-bins').value = state.n_bins;
    };
    document.getElementById('br-demo-perf').addEventListener('click',  () => { loadDemo('perfect');           void compute(tok); });
    document.getElementById('br-demo-coin').addEventListener('click',  () => { loadDemo('random-coin-flip'); void compute(tok); });
    document.getElementById('br-demo-calib').addEventListener('click', () => { loadDemo('well-calibrated');  void compute(tok); });
    document.getElementById('br-demo-over').addEventListener('click',  () => { loadDemo('overconfident');    void compute(tok); });
    document.getElementById('br-demo-under').addEventListener('click', () => { loadDemo('underconfident');   void compute(tok); });
    document.getElementById('br-demo-flip').addEventListener('click',  () => { loadDemo('flipped-sign');     void compute(tok); });
    document.getElementById('br-demo-rare').addEventListener('click',  () => { loadDemo('rare-event');       void compute(tok); });
    document.getElementById('br-demo-fine').addEventListener('click',  () => { loadDemo('fine-bins');        void compute(tok); });
    document.getElementById('br-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parsePairsBlob(document.getElementById('br-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.brier.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.probabilities = p.probabilities;
    state.outcomes = p.outcomes;
    const bins = parseInt(document.getElementById('br-bins').value, 10);
    state.n_bins = Number.isInteger(bins) && bins >= 1 ? bins : DEFAULT_BINS;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.probabilities, state.outcomes, state.n_bins);
    if (!local) { showErr(t('view.brier.err.degenerate')); return; }
    renderSummary(local, true);
    renderChart();
    renderHistChart();
    renderTable();
    let resp;
    try {
        resp = await api.anlyBrierScore(buildBody(state));
    } catch (e) {
        showErr(`${t('view.brier.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) { showErr(t('view.brier.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart();
    renderHistChart();
    renderTable();
}

function renderSummary(report, pending) {
    const local = localCompute(state.probabilities, state.outcomes, state.n_bins);
    const parityOk = !!local
        && Math.abs(local.brier_score - report.brier_score) < 1e-9
        && Math.abs(local.reliability - report.reliability) < 1e-9
        && Math.abs(local.resolution  - report.resolution)  < 1e-9;
    const badge = brierBadge(report.brier_score, report.uncertainty);
    const skill = skillScore(report.brier_score, report.uncertainty);
    const localTag = pending ? ` (${t('view.brier.tag.local')})` : '';
    const recomposed = report.reliability - report.resolution + report.uncertainty;
    const decompOk = Math.abs(recomposed - report.brier_score) < 1e-6;
    document.getElementById('br-summary').innerHTML = [
        card(t('view.brier.card.verdict'),     t(badge.key) + localTag, badge.cls),
        card(t('view.brier.card.brier'),       fmtBrier(report.brier_score), badge.cls),
        card(t('view.brier.card.skill'),       fmtSkill(skill),
             skill > 0.1 ? 'pos' : skill < 0 ? 'neg' : ''),
        card(t('view.brier.card.reliability'), fmtBrier(report.reliability),
             report.reliability < 0.01 ? 'pos' : report.reliability > 0.05 ? 'neg' : ''),
        card(t('view.brier.card.resolution'),  fmtBrier(report.resolution),
             report.resolution > 0.05 ? 'pos' : ''),
        card(t('view.brier.card.uncertainty'), fmtBrier(report.uncertainty)),
        card(t('view.brier.card.base_rate'),   fmtPct(report.base_rate)),
        card(t('view.brier.card.n'),           fmtInt(report.n_observations)),
        card(t('view.brier.card.decomp'),
             decompOk ? t('view.brier.tag.ok') : t('view.brier.tag.diverged'),
             decompOk ? 'pos' : 'neg'),
        card(t('view.brier.card.parity'),
             parityOk ? t('view.brier.tag.ok') : t('view.brier.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart() {
    if (!window.uPlot) return;
    const el = document.getElementById('br-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!state.probabilities.length) {
        el.innerHTML = `<div class="muted" data-i18n="view.brier.empty">${esc(t('view.brier.empty'))}</div>`;
        return;
    }
    const bins = reliabilityBins(state.probabilities, state.outcomes, state.n_bins);
    const populated = bins.filter(b => b.count > 0);
    const xs = populated.map(b => b.mean_pred);
    const ys = populated.map(b => b.mean_actual);
    // Identity reference line — drawn on the same x grid.
    const ident = xs.slice();
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 340,
        scales: { x: { time: false, range: [0, 1] }, y: { range: [0, 1] } },
        series: [
            { label: t('chart.series.predicted') },
            { label: t('chart.series.observed'),    stroke: '#00e5ff', width: 1.5, points: { show: true, size: 6 } },
            { label: t('chart.series.perfect'),     stroke: '#ff3860', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => v.toFixed(1)) },
            { stroke: '#aab', size: 50,
              values: (_u, splits) => splits.map(v => v.toFixed(1)) },
        ],
        legend: { show: true },
    }, [xs, ys, ident], el);
}

function renderHistChart() {
    if (!window.uPlot) return;
    const el = document.getElementById('br-hist-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!state.probabilities.length) {
        el.innerHTML = `<div class="muted" data-i18n="view.brier.empty_hist">${esc(t('view.brier.empty_hist'))}</div>`;
        return;
    }
    const bins = reliabilityBins(state.probabilities, state.outcomes, state.n_bins);
    const labels = bins.map(b => `${b.lo.toFixed(2)}–${b.hi.toFixed(2)}`);
    const ys = bins.map(b => b.count);
    const xs = bins.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.brier.chart.bin_idx') },
            { label: t('view.brier.chart.count'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 10, fill: '#7af0a8', stroke: '#7af0a8' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderTable() {
    const wrap = document.getElementById('br-table');
    if (!state.probabilities.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.brier.empty">${esc(t('view.brier.empty'))}</div>`;
        return;
    }
    const bins = reliabilityBins(state.probabilities, state.outcomes, state.n_bins);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.brier.col.bin">Bin</th>
                <th data-i18n="view.brier.col.range">Range</th>
                <th data-i18n="view.brier.col.count">Count</th>
                <th data-i18n="view.brier.col.mean_pred">Mean pred</th>
                <th data-i18n="view.brier.col.mean_actual">Mean actual</th>
                <th data-i18n="view.brier.col.gap">Calibration gap</th>
            </tr></thead>
            <tbody>
                ${bins.map(b => {
                    const gap = Number.isFinite(b.mean_pred) && Number.isFinite(b.mean_actual)
                        ? b.mean_pred - b.mean_actual
                        : NaN;
                    const gapCls = !Number.isFinite(gap) ? '' : Math.abs(gap) < 0.05 ? 'pos' : Math.abs(gap) > 0.15 ? 'neg' : '';
                    return `<tr>
                        <td>${b.bin}</td>
                        <td>${b.lo.toFixed(2)} − ${b.hi.toFixed(2)}</td>
                        <td>${fmtInt(b.count)}</td>
                        <td>${esc(fmtBrier(b.mean_pred))}</td>
                        <td>${esc(fmtBrier(b.mean_actual))}</td>
                        <td class="${gapCls}">${esc(fmtSkill(gap))}</td>
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
    const el = document.getElementById('br-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('br-err').style.display = 'none'; }
