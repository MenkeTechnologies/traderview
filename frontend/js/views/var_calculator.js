// VaR / Expected-Shortfall calculator. Paste a daily return series,
// pick a confidence (default 95%), and compare three methods:
//
//   * Historical Simulation     — distribution-free quantile of the
//                                  empirical return distribution.
//   * Filtered Historical (FHS) — EWMA-volatility-scaled HS, adapts
//                                  to current regime (Hull-White '98).
//   * Cornish-Fisher parametric — skew/kurtosis-adjusted Gaussian VaR.
//
// All three are positive-loss magnitudes. The chart shows the empirical
// histogram with each method's −VaR marked as a vertical bar so the
// user can eyeball where each estimator sits.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseReturns, validateReturns, confidenceToAlpha,
    histogram, formatLoss,
} from '../_var_calculator_inputs.js';

import { t } from '../i18n.js';
const DEFAULT_RETURNS = `# Paste daily returns (decimal, e.g. -0.012 or 0.018).
# One per line OR comma/space separated. # comments OK.
# 252 synthetic days (-2% to +2% with mild left skew) for demo:
${synthDemoReturns(252).join('\n')}
`;

function synthDemoReturns(n) {
    // Deterministic LCG so the textarea content is stable across reloads.
    let state = 0xDEADBEEFn;
    const rand = () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 11n) / 2 ** 53;
    };
    const normal = () => {
        const u1 = Math.max(rand(), 1e-10);
        const u2 = rand();
        return Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
    };
    const out = [];
    for (let i = 0; i < n; i++) {
        let r = normal() * 0.012;     // ~1.2% daily stdev
        // Left skew: every ~30th day, inject a 1.5× downside shock.
        if (Math.floor(rand() * 30) === 0) r = -Math.abs(r) * 1.8;
        out.push(r.toFixed(5));
    }
    return out;
}

let state = {
    text: DEFAULT_RETURNS,
    confidence: 0.95,
    ewmaLambda: 0.94,
};

export async function renderVarCalculator(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 data-i18n="view.var_calculator.h1.var_expected_shortfall" class="view-title">// VAR / EXPECTED SHORTFALL</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.var_calculator.h2.inputs">Inputs</h2>
            <div class="inline-form">
                <label><span data-i18n="view.var_calculator.label.confidence">Confidence</span>
                    <input id="vc-conf" type="number" step="0.01" min="0.5" max="0.999"
                           value="${state.confidence}"></label>
                <label><span data-i18n="view.var_calculator.label.ewma_lambda">EWMA λ (FHS only)</span>
                    <input id="vc-lambda" type="number" step="0.01" min="0.5" max="0.999"
                           value="${state.ewmaLambda}"></label>
                <button data-i18n="view.var_calculator.btn.compute" id="vc-run" class="primary" type="button">Compute</button>
            </div>
            <p data-i18n="view.var_calculator.hint.confidence_0_95_95_var_5_tail_higher_confidence_de" class="muted">
                Confidence 0.95 = 95% VaR (5% tail). Higher confidence ⇒ deeper-in-tail estimate.
                EWMA λ controls how quickly FHS adapts to recent vol — 0.94 is the RiskMetrics
                default (≈ 25-day half-life).
            </p>
            <h3 data-i18n="view.var_calculator.h3.return_series">Return series</h3>
            <textarea id="vc-text" rows="10"
                style="width:100%;font-family:monospace;font-size:13px">${esc(state.text)}</textarea>
        </div>

        <div id="vc-parse-errors" class="boot" style="display:none;color:var(--red)"></div>

        <div id="vc-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.var_calculator.h2.empirical_distribution_with_var_markers">Empirical distribution with VaR markers</h2>
            <div id="vc-chart" style="width:100%;height:340px"></div>
            <p data-i18n="view.var_calculator.hint.histogram_of_pasted_returns_vertical_lines_mark_th" class="muted" id="vc-chart-caption">
                Histogram of pasted returns. Vertical lines mark the negative of each
                method's VaR (a 5% VaR of 2% appears as a line at −0.02 on the x-axis).
            </p>
        </div>

        <div id="vc-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    wireForm(mount, tok);
    void fmt;
}

function wireForm(mount, tok) {
    document.getElementById('vc-run').addEventListener('click', () => {
        state.confidence = Number(document.getElementById('vc-conf').value);
        state.ewmaLambda = Number(document.getElementById('vc-lambda').value);
        state.text = document.getElementById('vc-text').value;
        void compute(mount, tok);
    });
}

async function compute(mount, tok) {
    hideErrs();
    const parsed = parseReturns(state.text);
    if (parsed.errors.length) renderParseErrors(parsed.errors);

    const validation = validateReturns(parsed.value);
    if (validation) { showErr(validation); return; }

    if (!Number.isFinite(state.confidence)
        || state.confidence <= 0.5 || state.confidence >= 1) {
        showErr(t('view.var_calculator.err.confidence_must_be_in_0_5_1')); return;
    }
    if (!Number.isFinite(state.ewmaLambda)
        || state.ewmaLambda <= 0.5 || state.ewmaLambda >= 1) {
        showErr(t('view.var_calculator.err.ewma_must_be_in_0_5_1')); return;
    }

    const alpha = confidenceToAlpha(state.confidence);
    let hs, fhs, cf;
    try {
        [hs, fhs, cf] = await Promise.all([
            api.anlyValueAtRiskHistorical({ returns: parsed.value, confidence: state.confidence }),
            api.anlyValueAtRiskFilteredHistorical({
                returns: parsed.value, confidence: state.confidence, lambda: state.ewmaLambda,
            }),
            api.anlyCornishFisherVar({ returns: parsed.value, alpha }),
        ]);
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        return;
    }
    if (!viewIsCurrent(tok)) return;

    renderSummary(hs, fhs, cf);
    renderChart(parsed.value, hs, fhs, cf);
}

function renderSummary(hs, fhs, cf) {
    const cards = [];
    cards.push(group(t('view.var_calculator.group.historical'), [
        [t('view.var_calculator.row.var'), formatLoss(hs?.var)],
        [t('view.var_calculator.row.expected_shortfall'), formatLoss(hs?.expected_shortfall)],
        [t('view.var_calculator.row.sample_size'), hs?.n != null ? String(hs.n) : '—'],
    ]));
    cards.push(group(t('view.var_calculator.group.filtered'), [
        [t('view.var_calculator.row.var'), formatLoss(fhs?.var)],
        [t('view.var_calculator.row.expected_shortfall'), formatLoss(fhs?.expected_shortfall)],
        [t('view.var_calculator.row.current_sigma'), fhs?.current_sigma != null ? `${(fhs.current_sigma * 100).toFixed(3)}%` : '—'],
    ]));
    cards.push(group(t('view.var_calculator.group.cornish_fisher'), [
        [t('view.var_calculator.row.var_cf'), formatLoss(cf?.var_cornish_fisher)],
        [t('view.var_calculator.row.var_gauss'), formatLoss(cf?.var_gaussian)],
        [t('view.var_calculator.row.skew'), cf?.skewness != null ? cf.skewness.toFixed(3) : '—'],
        [t('view.var_calculator.row.excess_kurt'), cf?.excess_kurtosis != null ? cf.excess_kurtosis.toFixed(3) : '—'],
        [t('view.var_calculator.row.monotonic'), t(cf?.is_monotonic === false ? 'view.var_calculator.row.monotonic_fail' : 'view.var_calculator.row.monotonic_pass')],
    ]));
    document.getElementById('vc-summary').innerHTML = cards.join('');
}

function group(title, kvs) {
    return `<div class="card">
        <div class="label">${esc(title)}</div>
        ${kvs.map(([k, v]) =>
            `<div class="vc-row"><span class="muted">${esc(k)}</span> <strong>${esc(v)}</strong></div>`
        ).join('')}
    </div>`;
}

function renderChart(returns, hs, fhs, cf) {
    const el = document.getElementById('vc-chart');
    if (!window.uPlot) {
        el.textContent = t('common.error.uplot_not_loaded_install');
        return;
    }
    el.innerHTML = '';

    const hist = histogram(returns, 50);
    if (hist.centers.length < 2) {
        el.innerHTML = `<div class="boot">${esc(t('view.var_calculator.empty.no_variation'))}</div>`;
        return;
    }

    // Build series:
    //   1: histogram bin centers + counts (line w/ stepped path approx via points)
    //   2-4: VaR markers — single-point series at (-VaR, max_count) with vertical drop
    //
    // To stay portable across uPlot versions, encode each VaR as a
    // 2-point series spanning [(-VaR, 0), (-VaR, max_count)]. uPlot
    // draws a near-vertical line.
    const maxCount = hist.counts.reduce((a, b) => Math.max(a, b), 0);
    const hsX = hs ? -hs.var : null;
    const fhsX = fhs ? -fhs.var : null;
    const cfX = cf ? -cf.var_cornish_fisher : null;

    const xs = hist.centers;
    const counts = hist.counts;
    // Marker series share their x with a 2-point pair so uPlot draws a vertical:
    const hsLine = xs.map(x => (hsX != null && Math.abs(x - hsX) < hist.binWidth / 2) ? maxCount : null);
    const fhsLine = xs.map(x => (fhsX != null && Math.abs(x - fhsX) < hist.binWidth / 2) ? maxCount : null);
    const cfLine = xs.map(x => (cfX != null && Math.abs(x - cfX) < hist.binWidth / 2) ? maxCount : null);

    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 340,
        scales: { x: {}, y: {} },
        series: [
            { label: t('chart.series.return') },
            { label: 'frequency', stroke: '#00e5ff', width: 2,
              fill: 'rgba(0,229,255,0.10)' },
            { label: '−VaR (HS)',  stroke: '#ff9f1a', width: 2,
              points: { show: true, size: 8, stroke: '#ff9f1a', fill: '#ff9f1a' } },
            { label: '−VaR (FHS)', stroke: '#ff3860', width: 2,
              points: { show: true, size: 8, stroke: '#ff3860', fill: '#ff3860' } },
            { label: '−VaR (CF)',  stroke: '#a06bff', width: 2,
              points: { show: true, size: 8, stroke: '#a06bff', fill: '#a06bff' } },
        ],
        axes: [{
            stroke: '#aab',
            values: (_, ticks) => ticks.map(t => `${(t * 100).toFixed(2)}%`),
        }, { stroke: '#aab' }],
    }, [xs, counts, hsLine, fhsLine, cfLine], el);
}

function renderParseErrors(errors) {
    const el = document.getElementById('vc-parse-errors');
    el.innerHTML = errors.slice(0, 20).map(e =>
        `<div>line ${e.line_no}: ${esc(e.message)} <span class="muted">→ <code>${esc(e.raw || '')}</code></span></div>`
    ).join('');
    if (errors.length > 20) {
        el.innerHTML += `<div class="muted">${esc(t("common.plus_n_more", { n: errors.length - 20 }))}</div>`;
    }
    el.style.display = 'block';
}

function showErr(msg) {
    const el = document.getElementById('vc-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErrs() {
    document.getElementById('vc-parse-errors').style.display = 'none';
    document.getElementById('vc-err').style.display = 'none';
}
