// Two-asset portfolio — Markowitz risk/return with correlation, showing the
// diversification benefit, via /calc/two-asset-portfolio. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const pct = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 3 }) + '%';
const num = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 3 }));

export async function renderTwoAssetPortfolio(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.twoasset.h1.title">// TWO-ASSET PORTFOLIO</span></h1>
        <p class="muted small" data-i18n="view.twoasset.hint.intro">
            Risk and return of a two-asset mix. When the assets aren't perfectly correlated, the
            portfolio's volatility falls below the weighted average of the two — the diversification
            benefit. At correlation 1 there's no benefit; at −1 the volatility is minimized. Updates
            as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.twoasset.h2.inputs">The mix</h2>
            <form id="twoasset-form" class="inline-form">
                <label><span data-i18n="view.twoasset.label.weighta">Weight of A (%)</span>
                    <input type="number" step="0.1" min="0" max="100" name="weight_a_pct" value="60" required></label>
                <label><span data-i18n="view.twoasset.label.returna">Return A (%)</span>
                    <input type="number" step="0.01" name="return_a_pct" value="8" required></label>
                <label><span data-i18n="view.twoasset.label.returnb">Return B (%)</span>
                    <input type="number" step="0.01" name="return_b_pct" value="4" required></label>
                <label><span data-i18n="view.twoasset.label.vola">Volatility A (%)</span>
                    <input type="number" step="0.01" min="0" name="volatility_a_pct" value="20" required></label>
                <label><span data-i18n="view.twoasset.label.volb">Volatility B (%)</span>
                    <input type="number" step="0.01" min="0" name="volatility_b_pct" value="10" required></label>
                <label><span data-i18n="view.twoasset.label.corr">Correlation (−1 to 1)</span>
                    <input type="number" step="0.01" min="-1" max="1" name="correlation" value="0.2" required></label>
                <label><span data-i18n="view.twoasset.label.rf">Risk-free rate (%)</span>
                    <input type="number" step="0.01" name="risk_free_pct" value="2"></label>
            </form>
        </div>
        <div id="twoasset-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#twoasset-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            weight_a_pct: Number(fd.get('weight_a_pct')) || 0,
            return_a_pct: Number(fd.get('return_a_pct')) || 0,
            return_b_pct: Number(fd.get('return_b_pct')) || 0,
            volatility_a_pct: Number(fd.get('volatility_a_pct')) || 0,
            volatility_b_pct: Number(fd.get('volatility_b_pct')) || 0,
            correlation: Number(fd.get('correlation')) || 0,
            risk_free_pct: Number(fd.get('risk_free_pct')) || 0,
        };
        try {
            const r = await api.calcTwoAssetPortfolio(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.twoasset.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#twoasset-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.twoasset.h2.result">The portfolio</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.twoasset.card.return">Return</div>
                    <div class="value pos">${pct(r.portfolio_return_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.twoasset.card.vol">Volatility</div>
                    <div class="value">${pct(r.portfolio_volatility_pct)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.twoasset.card.benefit">Diversification benefit</div>
                    <div class="value pos">${pct(r.diversification_benefit_pct)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.twoasset.row.weights">Weights (A / B)</td><td>${pct(r.weight_a_pct)} / ${pct(r.weight_b_pct)}</td></tr>
                    <tr><td data-i18n="view.twoasset.row.return">Portfolio return</td><td>${pct(r.portfolio_return_pct)}</td></tr>
                    <tr><td data-i18n="view.twoasset.row.vol">Portfolio volatility</td><td>${pct(r.portfolio_volatility_pct)}</td></tr>
                    <tr><td data-i18n="view.twoasset.row.weightedvol">Weighted-avg volatility</td><td>${pct(r.weighted_avg_volatility_pct)}</td></tr>
                    <tr><td data-i18n="view.twoasset.row.sharpe">Sharpe ratio</td><td>${num(r.sharpe_ratio)}</td></tr>
                    <tr class="emph"><td data-i18n="view.twoasset.row.benefit">Diversification benefit</td><td>${pct(r.diversification_benefit_pct)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
