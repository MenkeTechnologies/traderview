// Mortgage amortization calculator. PITI + extra-payment what-if.
// Reports principal, LTV, monthly P+I, PMI (when LTV > 80%), tax,
// insurance, HOA, total monthly. Extra-payment what-if shows months
// saved and interest saved vs baseline.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderMortgageAmortization(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.mortgage_amortization.title">// MORTGAGE AMORTIZATION</span></h1>
        <p class="muted small" data-i18n-html="view.mortgage_amortization.intro">
            Standard fixed-rate amortization (P&I = closed-form annuity) plus PMI when
            LTV > 80%, property tax escrow, homeowners insurance, HOA dues. Total monthly
            = <strong>PITIA</strong>. Optional <strong>extra principal</strong> field
            shows months saved and interest saved vs the baseline schedule.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small" data-i18n="view.mortgage_amortization.field.price">Home price $</span>
                    <input type="number" id="ma-price" step="5000" min="0" value="500000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.mortgage_amortization.field.down">Down payment $</span>
                    <input type="number" id="ma-down" step="5000" min="0" value="100000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.mortgage_amortization.field.apr">APR %</span>
                    <input type="number" id="ma-apr" step="0.125" min="0" max="30" value="6.5" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.mortgage_amortization.field.term">Term months</span>
                    <input type="number" id="ma-term" step="60" min="1" max="600" value="360" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.mortgage_amortization.field.tax">Annual property tax $</span>
                    <input type="number" id="ma-tax" step="500" min="0" value="6000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.mortgage_amortization.field.insurance">Annual insurance $</span>
                    <input type="number" id="ma-ins" step="100" min="0" value="1500" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.mortgage_amortization.field.hoa">Monthly HOA $</span>
                    <input type="number" id="ma-hoa" step="25" min="0" value="0" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.mortgage_amortization.field.pmi">PMI annual rate %</span>
                    <input type="number" id="ma-pmi" step="0.05" min="0" max="10" value="0.5" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.mortgage_amortization.field.extra">Extra principal $/mo</span>
                    <input type="number" id="ma-extra" step="50" min="0" value="0" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="ma-run" data-shortcut="r" data-i18n="view.mortgage_amortization.btn.run">⚡ Compute Mortgage</button>
            <div id="ma-result"></div>
        </div>
    `;
    mount.querySelector('#ma-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#ma-result');
    const input = {
        home_price_usd: parseFloat(mount.querySelector('#ma-price').value) || 0,
        down_payment_usd: parseFloat(mount.querySelector('#ma-down').value) || 0,
        apr_pct: parseFloat(mount.querySelector('#ma-apr').value) || 0,
        term_months: parseInt(mount.querySelector('#ma-term').value, 10) || 360,
        annual_property_tax_usd: parseFloat(mount.querySelector('#ma-tax').value) || 0,
        annual_insurance_usd: parseFloat(mount.querySelector('#ma-ins').value) || 0,
        monthly_hoa_usd: parseFloat(mount.querySelector('#ma-hoa').value) || 0,
        pmi_annual_rate_pct: parseFloat(mount.querySelector('#ma-pmi').value) || 0,
        extra_principal_usd: parseFloat(mount.querySelector('#ma-extra').value) || 0,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.mortgage_amortization.status.computing'))}</p>`;
    try {
        const r = await api('/mortgage-amortization/compute', { method: 'POST', body: JSON.stringify(input) });
        const ltvCls = r.ltv_pct > 80 ? 'neg' : 'pos';
        const baseY = `${Math.floor(r.baseline_term_months / 12)}y ${r.baseline_term_months % 12}m`;
        const extraY = `${Math.floor(r.extra_term_months / 12)}y ${r.extra_term_months % 12}m`;
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.mortgage_amortization.field.principal'))}</div>
                    <strong>$${(r.principal_financed_usd / 1000).toFixed(0)}K</strong></div>
                <div><div class="muted small">${esc(t('view.mortgage_amortization.field.ltv'))}</div>
                    <strong class="${ltvCls}">${r.ltv_pct.toFixed(1)}%</strong></div>
                <div><div class="muted small">${esc(t('view.mortgage_amortization.field.monthly_pi'))}</div>
                    <strong style="font-size:1.3em">$${r.monthly_pi_usd.toFixed(2)}</strong></div>
                <div><div class="muted small">${esc(t('view.mortgage_amortization.field.monthly_pmi'))}</div>
                    <strong>${r.monthly_pmi_usd > 0 ? '$' + r.monthly_pmi_usd.toFixed(2) : '—'}</strong></div>
                <div><div class="muted small">${esc(t('view.mortgage_amortization.field.monthly_tax'))}</div>
                    <strong>$${r.monthly_tax_usd.toFixed(2)}</strong></div>
                <div><div class="muted small">${esc(t('view.mortgage_amortization.field.monthly_ins'))}</div>
                    <strong>$${r.monthly_insurance_usd.toFixed(2)}</strong></div>
                <div><div class="muted small">${esc(t('view.mortgage_amortization.field.monthly_hoa'))}</div>
                    <strong>$${r.monthly_hoa_usd.toFixed(2)}</strong></div>
                <div><div class="muted small">${esc(t('view.mortgage_amortization.field.total_monthly'))}</div>
                    <strong style="font-size:1.4em">$${r.total_monthly_usd.toFixed(2)}</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.mortgage_amortization.h2.payoff'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th></th>
                    <th data-i18n="view.mortgage_amortization.th.term">Term</th>
                    <th data-i18n="view.mortgage_amortization.th.interest">Total interest</th>
                </tr></thead>
                <tbody>
                    <tr><td><strong>${esc(t('view.mortgage_amortization.row.baseline'))}</strong></td>
                        <td>${r.baseline_term_months} (${esc(baseY)})</td>
                        <td class="neg">$${r.baseline_total_interest_usd.toFixed(0)}</td></tr>
                    <tr><td><strong>${esc(t('view.mortgage_amortization.row.extra'))}</strong></td>
                        <td>${r.extra_term_months} (${esc(extraY)})</td>
                        <td class="neg">$${r.extra_total_interest_usd.toFixed(0)}</td></tr>
                    <tr><td><strong>${esc(t('view.mortgage_amortization.row.saved'))}</strong></td>
                        <td class="pos">${r.months_saved} mo</td>
                        <td class="pos">$${r.interest_saved_usd.toFixed(0)}</td></tr>
                </tbody>
            </table>
            <h2 style="margin-top:1rem">${esc(t('view.mortgage_amortization.h2.schedule'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.mortgage_amortization.th.month">Month</th>
                    <th data-i18n="view.mortgage_amortization.th.payment">Payment</th>
                    <th data-i18n="view.mortgage_amortization.th.principal">Principal</th>
                    <th data-i18n="view.mortgage_amortization.th.interest_s">Interest</th>
                    <th data-i18n="view.mortgage_amortization.th.balance">Balance</th>
                </tr></thead>
                <tbody>${(r.schedule_head || []).map(m => `
                    <tr>
                        <td>${m.month}</td>
                        <td>$${m.payment_usd.toFixed(2)}</td>
                        <td class="pos">$${m.principal_usd.toFixed(2)}</td>
                        <td class="neg">$${m.interest_usd.toFixed(2)}</td>
                        <td>$${m.balance_usd.toFixed(0)}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
