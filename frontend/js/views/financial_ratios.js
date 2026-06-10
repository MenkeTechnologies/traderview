// Canonical personal-finance ratios — savings rate, DTI, 28/36 front-end,
// liquidity ratio, solvency ratio, emergency fund ratio, retirement
// savings multiple — each with a published benchmark and traffic-light
// rating. Composite score 0-100 across all 7 ratios.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderFinancialRatios(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.financial_ratios.title">// PERSONAL FINANCIAL RATIOS</span></h1>
        <p class="muted small" data-i18n-html="view.financial_ratios.intro">
            Seven canonical household-finance ratios: <strong>savings rate</strong>,
            <strong>debt-to-income</strong> (CFPB 43% QM cap), <strong>front-end housing ratio</strong>
            (28/36 rule), <strong>liquidity</strong>, <strong>solvency</strong>,
            <strong>emergency fund</strong>, and <strong>retirement savings multiple</strong>
            (Fidelity glide: 1× @ 30, 3× @ 40, 6× @ 50, 8× @ 60, 10× @ 67).
            Each ratio gets a <span class="pos">good</span> / <span>ok</span> /
            <span class="neg">poor</span> rating; composite is the average score.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small" data-i18n="view.financial_ratios.field.gross_income">Gross monthly income $</span>
                    <input type="number" id="fr-gross" step="100" min="0" value="10000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.financial_ratios.field.expenses">Total monthly expenses $</span>
                    <input type="number" id="fr-expenses" step="100" min="0" value="5000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.financial_ratios.field.debt_payments">Monthly debt payments $</span>
                    <input type="number" id="fr-debt" step="50" min="0" value="2000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.financial_ratios.field.housing">Monthly housing payment $</span>
                    <input type="number" id="fr-housing" step="50" min="0" value="1500" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.financial_ratios.field.liquid">Liquid assets $</span>
                    <input type="number" id="fr-liquid" step="500" min="0" value="60000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.financial_ratios.field.emergency">Emergency fund balance $</span>
                    <input type="number" id="fr-emergency" step="500" min="0" value="60000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.financial_ratios.field.total_assets">Total assets $</span>
                    <input type="number" id="fr-assets" step="1000" min="0" value="800000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.financial_ratios.field.total_liab">Total liabilities $</span>
                    <input type="number" id="fr-liab" step="1000" min="0" value="200000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.financial_ratios.field.retirement">Retirement assets $</span>
                    <input type="number" id="fr-retirement" step="1000" min="0" value="500000" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="fr-run" data-shortcut="r" data-i18n="view.financial_ratios.btn.run">⚡ Compute Ratios</button>
            <div id="fr-result"></div>
        </div>
    `;
    mount.querySelector('#fr-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#fr-result');
    const input = {
        gross_monthly_income_usd: parseFloat(mount.querySelector('#fr-gross').value) || 0,
        total_monthly_expenses_usd: parseFloat(mount.querySelector('#fr-expenses').value) || 0,
        monthly_debt_payments_usd: parseFloat(mount.querySelector('#fr-debt').value) || 0,
        monthly_housing_payment_usd: parseFloat(mount.querySelector('#fr-housing').value) || 0,
        liquid_assets_usd: parseFloat(mount.querySelector('#fr-liquid').value) || 0,
        emergency_fund_balance_usd: parseFloat(mount.querySelector('#fr-emergency').value) || 0,
        total_assets_usd: parseFloat(mount.querySelector('#fr-assets').value) || 0,
        total_liabilities_usd: parseFloat(mount.querySelector('#fr-liab').value) || 0,
        retirement_assets_usd: parseFloat(mount.querySelector('#fr-retirement').value) || 0,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.financial_ratios.status.computing'))}</p>`;
    try {
        const r = await api.request('/financial-ratios/compute', { method: 'POST', body: JSON.stringify(input) });
        const compCls = r.composite_score_pct >= 75 ? 'pos' : r.composite_score_pct >= 50 ? '' : 'neg';
        const fmtValue = (key, v) => {
            switch (key) {
                case 'savings_rate':
                case 'debt_to_income':
                case 'front_end_ratio':
                case 'solvency_ratio':
                    return v.toFixed(1) + '%';
                case 'liquidity_ratio':
                case 'emergency_fund_ratio':
                    return v.toFixed(1) + ' months';
                case 'retirement_savings_x':
                    return v.toFixed(2) + '×';
                default:
                    return v.toFixed(2);
            }
        };
        const ratingCls = rating => rating === 'good' ? 'pos' : rating === 'poor' ? 'neg' : '';
        result.innerHTML = `
            <div style="margin-top:1rem">
                <div class="muted small">${esc(t('view.financial_ratios.field.composite'))}</div>
                <strong class="${compCls}" style="font-size:1.6em">${r.composite_score_pct.toFixed(0)}/100</strong>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.financial_ratios.h2.ratios'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.financial_ratios.th.ratio">Ratio</th>
                    <th data-i18n="view.financial_ratios.th.value">Value</th>
                    <th data-i18n="view.financial_ratios.th.benchmark">Benchmark</th>
                    <th data-i18n="view.financial_ratios.th.rating">Rating</th>
                </tr></thead>
                <tbody>${(r.ratings || []).map(c => `
                    <tr>
                        <td><strong>${esc(t('view.financial_ratios.ratio.' + c.key) || c.key)}</strong></td>
                        <td>${esc(fmtValue(c.key, c.value))}</td>
                        <td class="muted small">${esc(c.benchmark)}</td>
                        <td class="${ratingCls(c.rating)}" style="text-transform:uppercase"><strong>${esc(t('view.financial_ratios.rating.' + c.rating) || c.rating)}</strong></td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
