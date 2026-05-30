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
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseMatrix, parseFloatList, parseLabelList,
    normalizeLabels, defaultExcessReturns, validateCovariance,
} from '../_portfolio_allocator_inputs.js';
import { showToast } from '../toast.js';

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
        <h1 data-i18n="view.portfolio_allocator.h1.portfolio_allocator" class="view-title">// PORTFOLIO ALLOCATOR</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.portfolio_allocator.h2.inputs">Inputs</h2>
            <div class="op-inputs-grid">
                <div>
                    <h3 data-i18n="view.portfolio_allocator.h3.covariance_n_n">Covariance (N×N)</h3>
                    <textarea id="pa-cov" rows="8" style="width:100%;font-family:monospace;font-size:13px"
                              data-tip="view.portfolio_allocator.tip.cov">${esc(state.covText)}</textarea>
                </div>
                <div>
                    <h3 data-i18n="view.portfolio_allocator.h3.asset_labels_optional">Asset labels (optional)</h3>
                    <textarea id="pa-labels" rows="8" style="width:100%;font-family:monospace;font-size:13px"
                              data-tip="view.portfolio_allocator.tip.labels">${esc(state.labelsText)}</textarea>
                </div>
                <div>
                    <h3 data-i18n="view.portfolio_allocator.h3.excess_returns_optional">Excess returns (optional)</h3>
                    <textarea id="pa-excess" rows="8" style="width:100%;font-family:monospace;font-size:13px"
                              data-tip="view.portfolio_allocator.tip.excess">${esc(state.excessText)}</textarea>
                </div>
            </div>
            <button data-i18n="view.portfolio_allocator.btn.allocate" data-tip="view.portfolio_allocator.tip.allocate" data-shortcut="portfolio_allocator_run" id="pa-run" class="primary" type="button" style="margin-top:10px">Allocate</button>
        </div>

        <div id="pa-parse-errors" class="boot" style="display:none;color:var(--red)"></div>

        <div id="pa-results">
            <div class="boot" data-i18n-html="view.portfolio_allocator.hint.click_allocate">Click <em>Allocate</em> to run all three allocators.</div>
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
    if (validation) { showErr(validation); showToast(validation, { level: 'warning' }); return; }

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
        const m = t("common.error.api", { msg: e.message || e });
        showErr(m); showToast(m, { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderResults({ labels, mv, mxd, erc });
    showToast(t('view.portfolio_allocator.toast.done', { n }), { level: 'success' });
}

function renderResults({ labels, mv, mxd, erc }) {
    const root = document.getElementById('pa-results');
    root.innerHTML = `
        ${cardSection('Min-Variance + Tangency', mv ? `
            ${kv(t('view.portfolio_allocator.kv.mv_portfolio_vol'), mv.min_variance_portfolio_volatility?.toFixed(4))}
            ${kv(t('view.portfolio_allocator.kv.tangency_vol'), mv.tangency_volatility?.toFixed(4))}
            ${kv(t('view.portfolio_allocator.kv.tangency_excess_return'), mv.tangency_expected_return?.toFixed(4))}
            ${kv(t('view.portfolio_allocator.kv.tangency_sharpe'), mv.tangency_sharpe?.toFixed(3))}
            <h4 data-i18n="view.portfolio_allocator.h4.min_variance_weights">Min-variance weights</h4>
            <div id="pa-mv-weights"></div>
            <h4 data-i18n="view.portfolio_allocator.h4.tangency_weights">Tangency weights</h4>
            <div id="pa-mv-tan-weights"></div>
        ` : '<p data-i18n="view.portfolio_allocator.hint.mv_solver_returned_null_covariance_not_invertible" class="muted">MV solver returned null (covariance not invertible?).</p>')}

        ${cardSection(t('view.portfolio_allocator.section.max_diversification'), mxd ? `
            ${kv(t('view.portfolio_allocator.kv.diversification_ratio'), mxd.diversification_ratio?.toFixed(3))}
            ${kv(t('view.portfolio_allocator.kv.portfolio_vol'), mxd.portfolio_volatility?.toFixed(4))}
            ${kv(t('view.portfolio_allocator.kv.wtd_avg_single_asset_vol'), mxd.weighted_average_volatility?.toFixed(4))}
            <h4 data-i18n="view.portfolio_allocator.h4.weights">Weights</h4>
            <div id="pa-mxd-weights"></div>
        ` : '<p data-i18n="view.portfolio_allocator.hint.maxdiv_solver_returned_null" class="muted">MaxDiv solver returned null.</p>')}

        ${cardSection(t('view.portfolio_allocator.section.equal_risk_contrib'), erc ? `
            ${kv(t('view.portfolio_allocator.kv.portfolio_vol'), erc.portfolio_stdev?.toFixed(4))}
            ${kv(t('view.portfolio_allocator.kv.iterations'), erc.iterations)} ${kv(t('view.portfolio_allocator.kv.converged'), erc.converged ? '✓' : '✗')}
            <h4 data-i18n="view.portfolio_allocator.h4.weights">Weights</h4>
            <div id="pa-erc-weights"></div>
            <h4>${esc(t('view.portfolio_allocator.h4.risk_contrib', { pct: (100 / labels.length).toFixed(1) }))}</h4>
            <div id="pa-erc-risk"></div>
        ` : '<p data-i18n="view.portfolio_allocator.hint.erc_solver_returned_null" class="muted">ERC solver returned null.</p>')}
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
        el.innerHTML = `<div class="muted">${esc(t('common.empty.no_data'))}</div>`;
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
        `<div>${esc(t('common.parse_error_line', { line: e.line_no, msg: e.message }))} <span class="muted">→ <code>${esc(e.raw || '')}</code></span></div>`
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
