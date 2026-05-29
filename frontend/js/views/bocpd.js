// Bayesian Online Change-Point Detector (BOCPD) view.
//
// Adams-MacKay style: for each bar, the model maintains a posterior
// over "how long since the last regime change?" and emits the implied
// change-point probability. Spikes flag likely regime shifts.
//
// Visualization:
//   * Returns chart with change-point probability overlaid on a right
//     y-axis — spikes visually align with regime breaks.
//   * Expected run-length chart — sawtooth pattern, drops to 0 at each
//     detected change point.
//   * Cards: top-K detected change points + diagnostics.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseReturns, validateInputs, buildBody,
    topChangePoints, countAboveThreshold, fmtHazardPct,
} from '../_bocpd_inputs.js';

import { t } from '../i18n.js';
const DEFAULT_RETURNS = `# Paste a return series. One value per token.
# Demo: 100 low-vol returns, then 100 high-vol returns. BOCPD should
# flag a clear change point near index 100.
${synthBocpdDemo().join('\n')}
`;

function synthBocpdDemo() {
    let s = 0xC0DEC0DEn;
    const rand = () => {
        s = (s * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(s >> 11n) / 2 ** 53;
    };
    const normal = () => {
        const u1 = Math.max(rand(), 1e-10);
        const u2 = rand();
        return Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
    };
    const out = [];
    for (let i = 0; i < 100; i++) out.push((normal() * 0.005).toFixed(5));     // ~0.5% σ
    for (let i = 0; i < 100; i++) out.push((normal() * 0.025).toFixed(5));     // ~2.5% σ
    return out;
}

let state = {
    text: DEFAULT_RETURNS,
    hazard: 0.01,
    threshold: 0.10,
    topK: 5,
};

export async function renderBocpd(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 data-i18n="view.bocpd.h1.bayesian_change_points" class="view-title">// BAYESIAN CHANGE POINTS</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.bocpd.h2.inputs">Inputs</h2>
            <div class="inline-form">
                <label><span data-i18n="view.bocpd.label.hazard">Hazard rate (per-bar prior)</span>
                    <input id="bo-hazard" type="number" step="any" min="0.0001" max="0.5" value="${state.hazard}"></label>
                <label><span data-i18n="view.bocpd.label.threshold">Threshold (P ≥)</span>
                    <input id="bo-thresh" type="number" step="0.01" min="0" max="1" value="${state.threshold}"></label>
                <label><span data-i18n="view.bocpd.label.top_k">Show top-K</span>
                    <input id="bo-topk" type="number" step="1" min="1" max="20" value="${state.topK}"></label>
                <button data-i18n="view.bocpd.btn.detect" id="bo-run" class="primary" type="button">Detect</button>
            </div>
            <p data-i18n="view.bocpd.hint.hazard_the_per_bar_prior_probability_that_the_unde" class="muted">
                Hazard = the per-bar prior probability that the underlying distribution
                changes. Lower hazard → fewer, sharper detections. The threshold filters
                the displayed change points by posterior probability — change to 0 to see
                every bar's score.
            </p>
            <h3 data-i18n="view.bocpd.h3.return_series">Return series</h3>
            <textarea id="bo-text" rows="9"
                style="width:100%;font-family:monospace;font-size:13px">${esc(state.text)}</textarea>
        </div>

        <div id="bo-parse-errors" class="boot" style="display:none;color:var(--red)"></div>

        <div id="bo-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.bocpd.h2.returns_change_point_probability">Returns + change-point probability</h2>
            <div id="bo-cp-chart" style="width:100%;height:300px"></div>
            <p data-i18n="view.bocpd.hint.returns_on_left_axis_grey_change_point_probability" class="muted">
                Returns on left axis (grey). Change-point probability on right axis (red);
                values near 1 mark bars where BOCPD is confident the regime shifted.
            </p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.bocpd.h2.expected_run_length_bars_since_last_change">Expected run length (bars since last change)</h2>
            <div id="bo-rl-chart" style="width:100%;height:240px"></div>
            <p data-i18n="view.bocpd.hint.sawtooth_pattern_climbs_by_1_each_calm_bar_drops_t" class="muted">
                Sawtooth pattern: climbs by 1 each calm bar, drops to ~0 at every detected
                change point. Long flat-rising stretches = stable regime; frequent resets =
                noisy regime with many detected breaks.
            </p>
        </div>

        <div id="bo-detected" class="chart-panel" style="display:none">
            <h2 data-i18n="view.bocpd.h2.top_change_points_sorted_by_probability">Top change points (sorted by probability)</h2>
            <div id="bo-detected-body"></div>
        </div>

        <div id="bo-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('bo-run').addEventListener('click', () => {
        state.text = document.getElementById('bo-text').value;
        state.hazard = Number(document.getElementById('bo-hazard').value);
        state.threshold = Number(document.getElementById('bo-thresh').value);
        state.topK = parseInt(document.getElementById('bo-topk').value, 10);
        void detect(mount, tok);
    });
    void fmt;
}

async function detect(mount, tok) {
    hideErrs();
    const parsed = parseReturns(state.text);
    if (parsed.errors.length) renderParseErrors(parsed.errors);

    const err = validateInputs(parsed.value, state.hazard);
    if (err) { showErr(err); return; }
    if (!Number.isFinite(state.threshold) || state.threshold < 0 || state.threshold > 1) {
        showErr(t('view.bocpd.err.threshold_must_be_in_0_1')); return;
    }

    let res;
    try {
        res = await api.anlyBayesianChangePoint(buildBody(parsed.value, state.hazard));
        if (!res) throw new Error(t('view.bocpd.error.null_result'));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        return;
    }
    if (!viewIsCurrent(tok)) return;

    renderSummary(parsed.value, res);
    renderCpChart(parsed.value, res);
    renderRlChart(res);
    renderDetectedList(res);
}

function renderSummary(returns, res) {
    const detected = countAboveThreshold(res.change_point_probability, state.threshold);
    document.getElementById('bo-summary').innerHTML = [
        card(t('view.bocpd.card.hazard_per_bar'), fmtHazardPct(res.hazard)),
        card(t('view.bocpd.card.threshold'), `${(state.threshold * 100).toFixed(1)}%`),
        card(`Change points ≥ threshold`, String(detected),
            detected > 0 ? 'pos' : ''),
        card(t('view.bocpd.card.series_length'), String(returns.length)),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderCpChart(returns, res) {
    const el = document.getElementById('bo-cp-chart');
    if (!window.uPlot) { el.textContent = t('common.error.uplot_not_loaded'); return; }
    el.innerHTML = '';
    const n = returns.length;
    const xs = Array.from({ length: n }, (_, i) => i);
    // Map Option<f64> to null for uPlot — already null when missing.
    const cpProb = res.change_point_probability.map(v => Number.isFinite(v) ? v : null);

    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 300,
        scales: {
            x: {},
            ret:  { auto: true },
            prob: { auto: false, range: [0, 1] },
        },
        series: [
            { label: t('chart.series.bar') },
            { label: 'return',       stroke: 'rgba(170,170,170,0.7)', width: 1,
              points: { show: false }, scale: 'ret' },
            { label: 'P(change pt)', stroke: '#ff3860', width: 2,
              fill: 'rgba(255,56,96,0.10)',
              points: { show: false }, scale: 'prob' },
        ],
        axes: [
            { stroke: '#aab' },
            { scale: 'ret',  stroke: '#aab' },
            { scale: 'prob', side: 1, stroke: '#ff3860',
              values: (_, ticks) => ticks.map(t => t.toFixed(2)) },
        ],
    }, [xs, returns, cpProb], el);
}

function renderRlChart(res) {
    const el = document.getElementById('bo-rl-chart');
    if (!window.uPlot) { el.textContent = t('common.error.uplot_not_loaded'); return; }
    el.innerHTML = '';
    const rl = res.expected_run_length.map(v => Number.isFinite(v) ? v : null);
    const xs = Array.from({ length: rl.length }, (_, i) => i);
    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 240,
        scales: { x: {}, y: {} },
        series: [
            { label: t('chart.series.bar') },
            { label: 'expected run length', stroke: '#00e5ff', width: 1.5,
              fill: 'rgba(0,229,255,0.08)', points: { show: false } },
        ],
        axes: [{ stroke: '#aab' }, { stroke: '#aab' }],
    }, [xs, rl], el);
}

function renderDetectedList(res) {
    const panel = document.getElementById('bo-detected');
    const body = document.getElementById('bo-detected-body');
    const top = topChangePoints(res.change_point_probability, state.threshold, state.topK);
    if (top.length === 0) {
        panel.style.display = 'none';
        return;
    }
    panel.style.display = '';
    body.innerHTML = top.map(d =>
        `<div class="vc-row"><span class="muted">index ${d.index}</span>
            <strong>P = ${(d.probability * 100).toFixed(1)}%</strong></div>`
    ).join('');
}

function renderParseErrors(errors) {
    const el = document.getElementById('bo-parse-errors');
    el.innerHTML = errors.slice(0, 20).map(e =>
        `<div>line ${e.line_no}: ${esc(e.message)} <span class="muted">→ <code>${esc(e.raw || '')}</code></span></div>`
    ).join('');
    if (errors.length > 20) {
        el.innerHTML += `<div class="muted">… (+${errors.length - 20} more)</div>`;
    }
    el.style.display = 'block';
}

function showErr(msg) {
    const el = document.getElementById('bo-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErrs() {
    document.getElementById('bo-parse-errors').style.display = 'none';
    document.getElementById('bo-err').style.display = 'none';
}
