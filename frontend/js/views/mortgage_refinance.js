// Mortgage refinance calculator. Current loan + new loan + closing
// costs → monthly savings + breakeven months + lifetime interest
// comparison + status (refi wins / breakeven too long / no savings).

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import * as enh from '../calc_enhance.js';

export async function renderMortgageRefinance(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.mortgage_refinance.title">// MORTGAGE REFINANCE</span></h1>
        <p class="muted small" data-i18n-html="view.mortgage_refinance.intro">
            Fundamental refi question: do the lower monthly payments recoup the closing
            costs <em>before</em> you move or pay off the mortgage? <strong>Breakeven months
            = closing costs ÷ monthly savings</strong>. If your planning horizon exceeds
            breakeven, the refi pays for itself. Roll costs into the loan or pay at signing;
            optional cash-out increases the new principal.
        </p>
        <div class="chart-panel">
            <h2>${esc(t('view.mortgage_refinance.h2.current'))}</h2>
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small" data-i18n="view.mortgage_refinance.field.curr_balance">Current balance $</span>
                    <input type="number" id="mr-curr-bal" step="5000" min="0" value="350000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.mortgage_refinance.field.curr_apr">Current APR %</span>
                    <input type="number" id="mr-curr-apr" step="0.125" min="0" max="30" value="7.5" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.mortgage_refinance.field.curr_months">Remaining months</span>
                    <input type="number" id="mr-curr-months" step="12" min="1" max="600" value="300" style="width:100%">
                </label>
            </div>
            <h2>${esc(t('view.mortgage_refinance.h2.new'))}</h2>
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small" data-i18n="view.mortgage_refinance.field.new_apr">New APR %</span>
                    <input type="number" id="mr-new-apr" step="0.125" min="0" max="30" value="5.5" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.mortgage_refinance.field.new_term">New term months</span>
                    <input type="number" id="mr-new-term" step="60" min="1" max="600" value="360" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.mortgage_refinance.field.closing">Closing costs $</span>
                    <input type="number" id="mr-closing" step="500" min="0" value="6000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.mortgage_refinance.field.cashout">Cash-out $</span>
                    <input type="number" id="mr-cashout" step="1000" min="0" value="0" style="width:100%">
                </label>
                <label style="display:flex;align-items:center;gap:6px">
                    <input type="checkbox" id="mr-roll"> <span data-i18n="view.mortgage_refinance.field.roll">Roll costs into loan</span>
                </label>
                <label>
                    <span class="muted small" data-i18n="view.mortgage_refinance.field.horizon">Planning horizon months</span>
                    <input type="number" id="mr-horizon" step="12" min="1" max="600" value="84" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="mr-run" data-shortcut="r" data-i18n="view.mortgage_refinance.btn.run">⚡ Compute Breakeven</button>
            <div id="mr-result"></div>
        </div>
    `;
    mount.querySelector('#mr-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#mr-result');
    const input = {
        current_balance_usd: parseFloat(mount.querySelector('#mr-curr-bal').value) || 0,
        current_apr_pct: parseFloat(mount.querySelector('#mr-curr-apr').value) || 0,
        current_remaining_months: parseInt(mount.querySelector('#mr-curr-months').value, 10) || 360,
        new_apr_pct: parseFloat(mount.querySelector('#mr-new-apr').value) || 0,
        new_term_months: parseInt(mount.querySelector('#mr-new-term').value, 10) || 360,
        closing_costs_usd: parseFloat(mount.querySelector('#mr-closing').value) || 0,
        cash_out_usd: parseFloat(mount.querySelector('#mr-cashout').value) || 0,
        roll_costs_into_loan: mount.querySelector('#mr-roll').checked,
        planning_horizon_months: parseInt(mount.querySelector('#mr-horizon').value, 10) || 84,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.mortgage_refinance.status.computing'))}</p>`;
    try {
        const r = await api.request('/mortgage-refinance/compute', { method: 'POST', body: JSON.stringify(input) });
        // Current vs new monthly P&I — the payment side of the refi decision.
        const chart = enh.svgBarChart([
            { label: 'Current', value: r.current_monthly_pi_usd },
            { label: 'New', value: r.new_monthly_pi_usd },
        ]);
        const stCls = r.status === 'refi_wins' ? 'pos' : r.status === 'no_savings' ? 'neg' : '';
        const savCls = r.monthly_savings_usd > 0 ? 'pos' : 'neg';
        const beFmt = r.breakeven_months == null ? '∞'
            : `${r.breakeven_months.toFixed(1)} mo (${Math.floor(r.breakeven_months / 12)}y ${(r.breakeven_months % 12).toFixed(0)}m)`;
        const ltCls = r.lifetime_interest_delta_usd > 0 ? 'pos' : 'neg';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.mortgage_refinance.field.curr_pi'))}</div>
                    <strong>$${r.current_monthly_pi_usd.toFixed(2)}/mo</strong></div>
                <div><div class="muted small">${esc(t('view.mortgage_refinance.field.new_pi'))}</div>
                    <strong>$${r.new_monthly_pi_usd.toFixed(2)}/mo</strong></div>
                <div><div class="muted small">${esc(t('view.mortgage_refinance.field.savings'))}</div>
                    <strong class="${savCls}" style="font-size:1.3em">$${r.monthly_savings_usd.toFixed(2)}/mo</strong></div>
                <div><div class="muted small">${esc(t('view.mortgage_refinance.field.breakeven'))}</div>
                    <strong>${esc(beFmt)}</strong></div>
                <div><div class="muted small">${esc(t('view.mortgage_refinance.field.new_principal'))}</div>
                    <strong>$${(r.new_principal_usd / 1000).toFixed(0)}K</strong></div>
                <div><div class="muted small">${esc(t('view.mortgage_refinance.field.status'))}</div>
                    <strong class="${stCls}" style="text-transform:uppercase">${esc(t('view.mortgage_refinance.status.' + r.status) || r.status)}</strong></div>
            </div>
            ${chart}
            <div id="mr-tools" class="ce-toolbar"></div>
            <h2 style="margin-top:1rem">${esc(t('view.mortgage_refinance.h2.lifetime'))}</h2>
            <table class="trades">
                <tbody>
                    <tr><td><strong>${esc(t('view.mortgage_refinance.row.curr_interest'))}</strong></td>
                        <td class="neg">$${(r.current_remaining_interest_usd / 1000).toFixed(1)}K</td></tr>
                    <tr><td><strong>${esc(t('view.mortgage_refinance.row.new_interest'))}</strong></td>
                        <td class="neg">$${(r.new_total_interest_usd / 1000).toFixed(1)}K</td></tr>
                    <tr><td><strong>${esc(t('view.mortgage_refinance.row.delta'))}</strong></td>
                        <td class="${ltCls}">${r.lifetime_interest_delta_usd >= 0 ? '+' : '−'}$${(Math.abs(r.lifetime_interest_delta_usd) / 1000).toFixed(1)}K</td></tr>
                </tbody>
            </table>
        `;
        // Summary export (Copy / CSV). No permalink — id-based inputs.
        enh.mountToolbar(mount.querySelector('#mr-tools'), {
            viewId: 'mortgage-refinance',
            link: false,
            filename: 'mortgage-refinance.csv',
            getRows: () => [['metric', 'value'],
                ['current_monthly_pi_usd', r.current_monthly_pi_usd],
                ['new_monthly_pi_usd', r.new_monthly_pi_usd],
                ['monthly_savings_usd', r.monthly_savings_usd],
                ['breakeven_months', r.breakeven_months == null ? '' : r.breakeven_months],
                ['lifetime_interest_delta_usd', r.lifetime_interest_delta_usd]],
        });
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
