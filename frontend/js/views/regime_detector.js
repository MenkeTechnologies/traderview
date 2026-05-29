// Regime Detector — Hamilton 2-state Markov-switching applied to a
// pasted return series. Useful for answering "what regime were we in
// during this stretch?" against any historical window.
//
// Output:
//   * Per-state μ and σ (state 1 = high-vol by convention).
//   * Transition probabilities (p_kk = stay-in-state-k probability).
//   * Stationary distribution + expected dwell time per state.
//   * Per-bar P(state = 1) overlaid on the return series, so the user
//     can visually see where the high-vol regime kicked in.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseReturns, validateReturns,
    annualizeStdev, annualizeMean, stationaryDistribution,
    expectedDwell, highVolBarFraction,
} from '../_regime_detector_inputs.js';

import { t, applyUiI18n } from '../i18n.js';
const DEFAULT_RETURNS = `# Daily returns (decimal). One per token.
# Demo: 200 calm bars then 100 high-vol bars (regime change at index 200).
${synthRegimeDemo().join('\n')}
`;

function synthRegimeDemo() {
    let s = 0xCAFEF00Dn;
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
    for (let i = 0; i < 200; i++) out.push((normal() * 0.006).toFixed(5));    // ~0.6% σ
    for (let i = 0; i < 100; i++) out.push((normal() * 0.025).toFixed(5));    // ~2.5% σ
    return out;
}

let state = {
    text: DEFAULT_RETURNS,
    barsPerYear: 252,
};

export async function renderRegimeDetector(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 data-i18n="view.regime_detector.h1.regime_detector" class="view-title">// REGIME DETECTOR</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.regime_detector.h2.inputs">Inputs</h2>
            <div class="inline-form">
                <label><span data-i18n="view.regime_detector.label.bars_per_year">Bars per year (annualization)</span>
                    <input id="rd-bpy" type="number" step="1" min="1" value="${state.barsPerYear}"></label>
                <button data-i18n="view.regime_detector.btn.detect" id="rd-run" class="primary" type="button">Detect</button>
            </div>
            <p data-i18n="view.regime_detector.hint.2_state_markov_switching_via_hamilton_kim_filter_b" class="muted">
                2-state Markov-switching via Hamilton-Kim filter + Baum-Welch EM. State 1 is
                the higher-vol state by convention. Annualization uses √N scaling for σ and
                linear scaling for μ.
            </p>
            <h3 data-i18n="view.regime_detector.h3.return_series">Return series</h3>
            <textarea id="rd-text" rows="10"
                style="width:100%;font-family:monospace;font-size:13px">${esc(state.text)}</textarea>
        </div>

        <div id="rd-parse-errors" class="boot" style="display:none;color:var(--red)"></div>

        <div id="rd-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.regime_detector.h2.returns_p_high_vol_state_overlay">Returns + P(high-vol state) overlay</h2>
            <div id="rd-chart" style="width:100%;height:340px"></div>
            <p data-i18n="view.regime_detector.hint.returns_on_the_left_axis_grey_state_1_probability_" class="muted">
                Returns on the left axis (grey). State-1 probability on the right axis (red);
                values near 1 mark bars classified as the high-vol regime.
            </p>
        </div>

        <div id="rd-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    wireForm(mount, tok);
    void fmt;
}

function wireForm(mount, tok) {
    document.getElementById('rd-run').addEventListener('click', () => {
        state.barsPerYear = Number(document.getElementById('rd-bpy').value);
        state.text = document.getElementById('rd-text').value;
        void detect(mount, tok);
    });
}

async function detect(mount, tok) {
    hideErrs();
    const parsed = parseReturns(state.text);
    if (parsed.errors.length) renderParseErrors(parsed.errors);

    const err = validateReturns(parsed.value);
    if (err) { showErr(err); return; }
    if (!Number.isFinite(state.barsPerYear) || state.barsPerYear < 1) {
        showErr(t('view.regime_detector.err.bars_per_year_must_be_1')); return;
    }

    let res;
    try {
        res = await api.anlyMarkovSwitching2State({ returns: parsed.value });
        if (!res) throw new Error(t('view.regime_detector.error.null_result'));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        return;
    }
    if (!viewIsCurrent(tok)) return;

    renderSummary(res);
    renderChart(parsed.value, res.prob_state1);
}

function renderSummary(res) {
    const bpy = state.barsPerYear;
    const stationary = stationaryDistribution(res.p00, res.p11);
    const dwell0 = expectedDwell(res.p00);
    const dwell1 = expectedDwell(res.p11);
    const highFrac = highVolBarFraction(res.prob_state1, 0.5);

    const cards = [];
    cards.push(stateCard(t('view.regime_detector.state.state0_low'), '#39ff14', {
        [t('view.regime_detector.row.mu_per_bar')]:    fmtPctish(res.mu0),
        [t('view.regime_detector.row.mu_annual')]:     fmtPctish(annualizeMean(res.mu0, bpy)),
        [t('view.regime_detector.row.sigma_per_bar')]: fmtPctish(res.sigma0),
        [t('view.regime_detector.row.sigma_annual')]:  fmtPctish(annualizeStdev(res.sigma0, bpy)),
        [t('view.regime_detector.row.p_stay_p00')]:    res.p00.toFixed(4),
        [t('view.regime_detector.row.expected_dwell')]:formatDwell(dwell0),
        [t('view.regime_detector.row.long_run_frac')]: pct(stationary.p_state0),
    }));
    cards.push(stateCard(t('view.regime_detector.state.state1_high'), '#ff3860', {
        [t('view.regime_detector.row.mu_per_bar')]:    fmtPctish(res.mu1),
        [t('view.regime_detector.row.mu_annual')]:     fmtPctish(annualizeMean(res.mu1, bpy)),
        [t('view.regime_detector.row.sigma_per_bar')]: fmtPctish(res.sigma1),
        [t('view.regime_detector.row.sigma_annual')]:  fmtPctish(annualizeStdev(res.sigma1, bpy)),
        [t('view.regime_detector.row.p_stay_p11')]:    res.p11.toFixed(4),
        [t('view.regime_detector.row.expected_dwell')]:formatDwell(dwell1),
        [t('view.regime_detector.row.long_run_frac')]: pct(stationary.p_state1),
    }));
    cards.push(`<div class="card">
        <div class="label" data-i18n="view.regime_detector.card.fit_diagnostics">Fit diagnostics</div>
        <div class="value rd-summary-value">
            <div class="vc-row"><span class="muted" data-i18n="view.regime_detector.row.log_likelihood">Log-likelihood</span> <strong>${res.log_likelihood.toFixed(3)}</strong></div>
            <div class="vc-row"><span class="muted" data-i18n="view.regime_detector.row.em_iterations">EM iterations</span> <strong>${res.iterations}</strong></div>
            <div class="vc-row"><span class="muted" data-i18n="view.regime_detector.row.high_vol_bars">Bars classified high-vol (P>0.5)</span> <strong>${pct(highFrac)}</strong></div>
            <div class="vc-row"><span class="muted" data-i18n="view.regime_detector.row.series_length">Series length</span> <strong>${res.prob_state1.length}</strong></div>
        </div>
    </div>`);
    const rdSummary = document.getElementById('rd-summary');
    rdSummary.innerHTML = cards.join('');
    try { applyUiI18n(rdSummary); } catch (_) {}
}

function stateCard(label, swatchColor, kvs) {
    const cls = swatchColor === '#39ff14' ? 'state-low' : 'state-high';
    return `<div class="card">
        <div class="label"><span class="rd-swatch ${cls}">▮</span> ${esc(label)}</div>
        <div class="value rd-summary-value">
            ${Object.entries(kvs).map(([k, v]) =>
                `<div class="vc-row"><span class="muted">${esc(k)}</span> <strong>${esc(v)}</strong></div>`
            ).join('')}
        </div>
    </div>`;
}

function fmtPctish(x) {
    if (!Number.isFinite(x)) return '—';
    return `${(x * 100).toFixed(3)}%`;
}

function pct(x) {
    if (!Number.isFinite(x)) return '—';
    return `${(x * 100).toFixed(1)}%`;
}

function formatDwell(d) {
    if (!Number.isFinite(d)) return d === Infinity ? '∞ (absorbing)' : '—';
    return `${d.toFixed(1)} bars`;
}

function renderChart(returns, probState1) {
    const el = document.getElementById('rd-chart');
    if (!window.uPlot) {
        el.textContent = t('common.error.uplot_not_loaded');
        return;
    }
    el.innerHTML = '';
    const n = returns.length;
    const xs = Array.from({ length: n }, (_, i) => i);
    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 340,
        scales: {
            x: {},
            y:    { auto: true },
            prob: { auto: false, range: [0, 1] },
        },
        series: [
            { label: t('chart.series.bar') },
            { label: t('chart.series.return'), stroke: 'rgba(170,170,170,0.7)', width: 1,
              points: { show: false }, scale: 'y' },
            { label: 'P(state 1)', stroke: '#ff3860', width: 2,
              fill: 'rgba(255,56,96,0.10)',
              points: { show: false }, scale: 'prob' },
        ],
        axes: [
            { stroke: '#aab' },
            { scale: 'y', stroke: '#aab' },
            { scale: 'prob', side: 1, stroke: '#ff3860',
              values: (_, ticks) => ticks.map(t => t.toFixed(2)) },
        ],
    }, [xs, returns, probState1], el);
}

function renderParseErrors(errors) {
    const el = document.getElementById('rd-parse-errors');
    el.innerHTML = errors.slice(0, 20).map(e =>
        `<div>line ${e.line_no}: ${esc(e.message)} <span class="muted">→ <code>${esc(e.raw || '')}</code></span></div>`
    ).join('');
    if (errors.length > 20) {
        el.innerHTML += `<div class="muted">… (+${errors.length - 20} more)</div>`;
    }
    el.style.display = 'block';
}

function showErr(msg) {
    const el = document.getElementById('rd-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErrs() {
    document.getElementById('rd-parse-errors').style.display = 'none';
    document.getElementById('rd-err').style.display = 'none';
}
