// Portfolio Allocator view — runs 3 allocators side-by-side on a user-
// supplied covariance matrix:
//   * Min-variance + tangency (Markowitz, closed-form)
//   * Maximum Diversification (Choueifaty-Coignard)
//   * Equal Risk Contribution (Maillard et al.)
//
// User pastes the cov matrix (rows = whitespace OR comma separated),
// optional asset labels, and optional excess returns (used by the MV
// solver's tangency portfolio). Each allocator's weights render as a
// horizontal bar chart; ERC also shows its per-asset risk contributions
// so the user can verify they're equalized.

import { api } from '../api.js';
import { esc, fmt, fmtPct } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseMatrix, parseFloatList, parseLabelList,
    normalizeLabels, defaultExcessReturns, validateCovariance,
} from '../_portfolio_allocator_inputs.js';

const DEFAULT_COV = `# Symmetric covariance matrix (annualized).
# Example: 4 assets, increasing variance, mild positive correlations.
0.04  0.01  0.005 0.002
0.01  0.09  0.02  0.005
0.005 0.02  0.16  0.01
0.002 0.005 0.01  0.25
`;
const DEFAULT_LABELS = `# One ticker / label per line.
SPY
QQQ
GLD
TLT
`;
const DEFAULT_EXCESS = `# One excess-return per asset (annualized).
# Optional — leave blank to use a flat 5% for every asset.
0.06
0.08
0.03
0.02
`;

let state = {
    covText: DEFAULT_COV,
    labelsText: DEFAULT_LABELS,
    excessText: DEFAULT_EXCESS,
};

export async function renderPortfolioAllocator(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 class="view-title">// PORTFOLIO ALLOCATOR</h1>

        <div class="chart-panel">
            <h2>Inputs</h2>
            <div class="op-inputs-grid">
                <div>
                    <h3>Covariance (N×N)</h3>
                    <textarea id="pa-cov" rows="8" style="width:100%;font-family:monospace;font-size:13px">${esc(state.covText)}</textarea>
                </div>
                <div>
                    <h3>Asset labels (optional)</h3>
                    <textarea id="pa-labels" rows="8" style="width:100%;font-family:monospace;font-size:13px">${esc(state.labelsText)}</textarea>
                </div>
                <div>
                    <h3>Excess returns (optional)</h3>
                    <textarea id="pa-excess" rows="8" style="width:100%;font-family:monospace;font-size:13px">${esc(state.excessText)}</textarea>
                </div>
            </div>
            <button id="pa-run" class="primary" type="button" style="margin-top:10px">Allocate</button>
        </div>

        <div id="pa-parse-errors" class="boot" style="display:none;color:var(--red)"></div>

        <div id="pa-results">
            <div class="boot">Click <em>Allocate</em> to run all three allocators.</div>
        </div>

        <div id="pa-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('pa-run').addEventListener('click', () => {
        state.covText = document.getElementById('pa-cov').value;
        state.labelsText = document.getElementById('pa-labels').value;
        state.excessText = document.getElementById('pa-excess').value;
        void allocate(mount, tok);
    });
    void fmt; void fmtPct;
}

async function allocate(mount, tok) {
    hideErrs();
    const cov = parseMatrix(state.covText);
    if (cov.errors.length) renderParseErrors(cov.errors);

    const validation = validateCovariance(cov.value);
    if (validation) { showErr(validation); return; }

    const n = cov.value.length;
    const labels = normalizeLabels(parseLabelList(state.labelsText), n);
    const excessParsed = parseFloatList(state.excessText);
    if (excessParsed.errors.length) renderParseErrors(excessParsed.errors);
    const excess = excessParsed.value.length === n
        ? excessParsed.value
        : defaultExcessReturns(n);

    // Fire all three allocator calls in parallel.
    let mv, mxd, erc;
    try {
        [mv, mxd, erc] = await Promise.all([
            api.minVariancePortfolio({
                covariance: cov.value,
                expected_excess_returns: excess,
            }),
            api.maxDiversification({ covariance: cov.value }),
            api.equalRiskContributionPortfolio({ cov: cov.value }),
        ]);
    } catch (e) {
        showErr(`API error: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderResults({ labels, mv, mxd, erc });
}

function renderResults({ labels, mv, mxd, erc }) {
    const root = document.getElementById('pa-results');
    root.innerHTML = `
        ${cardSection('Min-Variance + Tangency', mv ? `
            ${kv('MV portfolio vol', mv.min_variance_portfolio_volatility?.toFixed(4))}
            ${kv('Tangency vol', mv.tangency_volatility?.toFixed(4))}
            ${kv('Tangency excess return', mv.tangency_expected_return?.toFixed(4))}
            ${kv('Tangency Sharpe', mv.tangency_sharpe?.toFixed(3))}
            <h4>Min-variance weights</h4>
            <div id="pa-mv-weights"></div>
            <h4>Tangency weights</h4>
            <div id="pa-mv-tan-weights"></div>
        ` : '<p class="muted">MV solver returned null (covariance not invertible?).</p>')}

        ${cardSection('Maximum Diversification', mxd ? `
            ${kv('Diversification ratio', mxd.diversification_ratio?.toFixed(3))}
            ${kv('Portfolio vol', mxd.portfolio_volatility?.toFixed(4))}
            ${kv('Wtd-avg single-asset vol', mxd.weighted_average_volatility?.toFixed(4))}
            <h4>Weights</h4>
            <div id="pa-mxd-weights"></div>
        ` : '<p class="muted">MaxDiv solver returned null.</p>')}

        ${cardSection('Equal Risk Contribution', erc ? `
            ${kv('Portfolio vol', erc.portfolio_stdev?.toFixed(4))}
            ${kv('Iterations', erc.iterations)} ${kv('Converged', erc.converged ? '✓' : '✗')}
            <h4>Weights</h4>
            <div id="pa-erc-weights"></div>
            <h4>Risk contributions (should be ≈ ${(100 / labels.length).toFixed(1)}% each)</h4>
            <div id="pa-erc-risk"></div>
        ` : '<p class="muted">ERC solver returned null.</p>')}
    `;

    if (mv) {
        renderBarChart('pa-mv-weights', labels, mv.min_variance_weights, { unit: '%' });
        renderBarChart('pa-mv-tan-weights', labels, mv.tangency_weights, { unit: '%' });
    }
    if (mxd) renderBarChart('pa-mxd-weights', labels, mxd.weights, { unit: '%' });
    if (erc) {
        renderBarChart('pa-erc-weights', labels, erc.weights, { unit: '%' });
        renderBarChart('pa-erc-risk', labels, erc.risk_contributions, { unit: '%' });
    }
}

function cardSection(title, innerHtml) {
    return `<div class="chart-panel">
        <h2>${esc(title)}</h2>
        ${innerHtml}
    </div>`;
}

function kv(label, value) {
    const v = value == null ? '—' : String(value);
    return `<div class="pa-kv"><span class="muted">${esc(label)}</span> <strong>${esc(v)}</strong></div>`;
}

// Render a single horizontal bar chart by hand (no uPlot dependency,
// stays readable in screenshots). Negative weights render as red bars
// extending left of the zero baseline.
function renderBarChart(elId, labels, values, opts = {}) {
    const el = document.getElementById(elId);
    if (!el) return;
    if (!Array.isArray(values) || values.length === 0) {
        el.innerHTML = '<div class="muted">no data</div>';
        return;
    }
    const maxAbs = values.reduce((a, b) => Math.max(a, Math.abs(b)), 0) || 1;
    const unit = opts.unit || '';
    const rows = labels.map((lab, i) => {
        const v = values[i];
        const pct = v == null ? 0 : (v * 100);
        const widthPct = (Math.abs(v) / maxAbs) * 100;
        const cls = v >= 0 ? 'pos' : 'neg';
        const sideStyle = v >= 0
            ? `left:50%;width:${(widthPct / 2).toFixed(2)}%`
            : `right:50%;width:${(widthPct / 2).toFixed(2)}%`;
        return `<div class="pa-bar-row">
            <span class="pa-bar-label">${esc(lab)}</span>
            <div class="pa-bar-track">
                <div class="pa-bar ${cls}" style="${sideStyle}"></div>
                <span class="pa-bar-zero"></span>
            </div>
            <span class="pa-bar-value ${cls}">${pct.toFixed(2)}${unit}</span>
        </div>`;
    });
    el.innerHTML = rows.join('');
}

function renderParseErrors(errors) {
    const el = document.getElementById('pa-parse-errors');
    el.innerHTML = errors.map(e =>
        `<div>line ${e.line_no}: ${esc(e.message)} <span class="muted">→ <code>${esc(e.raw || '')}</code></span></div>`
    ).join('');
    el.style.display = 'block';
}

function showErr(msg) {
    const el = document.getElementById('pa-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErrs() {
    document.getElementById('pa-parse-errors').style.display = 'none';
    document.getElementById('pa-err').style.display = 'none';
}
