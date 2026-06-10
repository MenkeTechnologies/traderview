// Rent vs Buy NPV calculator (NYT-calculator-style). Year-by-year
// cumulative cost comparison over N years, with breakeven year and
// net winner.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderRentVsBuy(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.rent_vs_buy.title">// RENT VS BUY</span></h1>
        <p class="muted small" data-i18n-html="view.rent_vs_buy.intro">
            NYT-calculator-style year-by-year comparison. <strong>RENT</strong> path =
            yearly rent − investment return on what would have been your down payment +
            closing costs. <strong>BUY</strong> path = PITI + maintenance + opportunity
            cost on equity − home appreciation − mortgage paydown, plus selling costs at
            exit. <strong>Breakeven year</strong> = first year where cumulative buy cost
            ≤ cumulative rent cost.
        </p>
        <div class="chart-panel">
            <h2>${esc(t('view.rent_vs_buy.h2.buy'))}</h2>
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.rent_vs_buy.field.price">Home price $</span>
                    <input type="number" id="rb-price" step="5000" min="0" value="500000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.rent_vs_buy.field.down_pct">Down payment %</span>
                    <input type="number" id="rb-down" step="1" min="0" max="100" value="20" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.rent_vs_buy.field.apr">Mortgage APR %</span>
                    <input type="number" id="rb-apr" step="0.125" min="0" max="30" value="6.5" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.rent_vs_buy.field.term">Mortgage term months</span>
                    <input type="number" id="rb-term" step="60" min="1" max="600" value="360" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.rent_vs_buy.field.closing">Closing costs %</span>
                    <input type="number" id="rb-closing" step="0.25" min="0" max="20" value="2" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.rent_vs_buy.field.proptax">Property tax %/yr</span>
                    <input type="number" id="rb-proptax" step="0.1" min="0" max="10" value="1.2" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.rent_vs_buy.field.insurance">Insurance $/yr</span>
                    <input type="number" id="rb-insurance" step="100" min="0" value="1500" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.rent_vs_buy.field.maint">Maintenance %/yr</span>
                    <input type="number" id="rb-maint" step="0.25" min="0" max="10" value="1" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.rent_vs_buy.field.hoa">HOA $/mo</span>
                    <input type="number" id="rb-hoa" step="25" min="0" value="0" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.rent_vs_buy.field.appr">Home appreciation %/yr</span>
                    <input type="number" id="rb-appr" step="0.25" min="-20" max="30" value="3" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.rent_vs_buy.field.selling">Selling costs %</span>
                    <input type="number" id="rb-selling" step="0.5" min="0" max="20" value="6" style="width:100%"></label>
            </div>
            <h2>${esc(t('view.rent_vs_buy.h2.rent'))}</h2>
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.rent_vs_buy.field.rent">Monthly rent $</span>
                    <input type="number" id="rb-rent" step="50" min="0" value="2500" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.rent_vs_buy.field.renter_ins">Renter insurance $/yr</span>
                    <input type="number" id="rb-renter-ins" step="50" min="0" value="200" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.rent_vs_buy.field.rent_infl">Rent inflation %/yr</span>
                    <input type="number" id="rb-rent-infl" step="0.5" min="-20" max="30" value="3" style="width:100%"></label>
            </div>
            <h2>${esc(t('view.rent_vs_buy.h2.params'))}</h2>
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.rent_vs_buy.field.inv_return">Investment return %/yr</span>
                    <input type="number" id="rb-inv" step="0.25" min="-20" max="30" value="7" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.rent_vs_buy.field.horizon">Horizon years</span>
                    <input type="number" id="rb-horizon" step="1" min="1" max="60" value="10" style="width:100%"></label>
            </div>
            <button class="btn btn-sm primary" id="rb-run" data-shortcut="r" data-i18n="view.rent_vs_buy.btn.run">⚡ Compute NPV</button>
            <div id="rb-result"></div>
        </div>
    `;
    mount.querySelector('#rb-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#rb-result');
    const input = {
        home_price_usd: parseFloat(mount.querySelector('#rb-price').value) || 0,
        down_payment_pct: parseFloat(mount.querySelector('#rb-down').value) || 0,
        mortgage_apr_pct: parseFloat(mount.querySelector('#rb-apr').value) || 0,
        mortgage_term_months: parseInt(mount.querySelector('#rb-term').value, 10) || 360,
        closing_costs_pct: parseFloat(mount.querySelector('#rb-closing').value) || 0,
        property_tax_annual_pct: parseFloat(mount.querySelector('#rb-proptax').value) || 0,
        insurance_annual_usd: parseFloat(mount.querySelector('#rb-insurance').value) || 0,
        maintenance_annual_pct: parseFloat(mount.querySelector('#rb-maint').value) || 0,
        monthly_hoa_usd: parseFloat(mount.querySelector('#rb-hoa').value) || 0,
        home_appreciation_pct: parseFloat(mount.querySelector('#rb-appr').value) || 0,
        selling_costs_pct: parseFloat(mount.querySelector('#rb-selling').value) || 0,
        monthly_rent_usd: parseFloat(mount.querySelector('#rb-rent').value) || 0,
        renter_insurance_annual_usd: parseFloat(mount.querySelector('#rb-renter-ins').value) || 0,
        rent_inflation_pct: parseFloat(mount.querySelector('#rb-rent-infl').value) || 0,
        investment_return_pct: parseFloat(mount.querySelector('#rb-inv').value) || 0,
        horizon_years: parseInt(mount.querySelector('#rb-horizon').value, 10) || 10,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.rent_vs_buy.status.computing'))}</p>`;
    try {
        const r = await api.request('/rent-vs-buy/compute', { method: 'POST', body: JSON.stringify(input) });
        const winnerCls = r.net_winner === 'buy' ? 'pos' : '';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.rent_vs_buy.field.cum_rent'))}</div>
                    <strong>$${(r.cum_rent_total_usd / 1000).toFixed(1)}K</strong></div>
                <div><div class="muted small">${esc(t('view.rent_vs_buy.field.cum_buy'))}</div>
                    <strong>$${(r.cum_buy_total_usd / 1000).toFixed(1)}K</strong></div>
                <div><div class="muted small">${esc(t('view.rent_vs_buy.field.winner'))}</div>
                    <strong class="${winnerCls}" style="font-size:1.4em;text-transform:uppercase">${esc(t('view.rent_vs_buy.winner.' + r.net_winner) || r.net_winner)}</strong></div>
                <div><div class="muted small">${esc(t('view.rent_vs_buy.field.breakeven'))}</div>
                    <strong>${r.breakeven_year == null ? esc(t('view.rent_vs_buy.field.never')) : 'year ' + r.breakeven_year}</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.rent_vs_buy.h2.yearly'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.rent_vs_buy.th.year">Yr</th>
                    <th data-i18n="view.rent_vs_buy.th.rent_year">Rent (yr)</th>
                    <th data-i18n="view.rent_vs_buy.th.buy_year">Buy (yr)</th>
                    <th data-i18n="view.rent_vs_buy.th.cum_rent">Cum rent</th>
                    <th data-i18n="view.rent_vs_buy.th.cum_buy">Cum buy</th>
                    <th data-i18n="view.rent_vs_buy.th.home_value">Home value</th>
                    <th data-i18n="view.rent_vs_buy.th.balance">Balance</th>
                    <th data-i18n="view.rent_vs_buy.th.equity">Equity</th>
                </tr></thead>
                <tbody>${(r.yearly || []).map(y => `
                    <tr>
                        <td>${y.year}</td>
                        <td>$${(y.rent_year_cost_usd / 1000).toFixed(1)}K</td>
                        <td>$${(y.buy_year_cost_usd / 1000).toFixed(1)}K</td>
                        <td>$${(y.cum_rent_usd / 1000).toFixed(1)}K</td>
                        <td>$${(y.cum_buy_usd / 1000).toFixed(1)}K</td>
                        <td>$${(y.home_value_usd / 1000).toFixed(0)}K</td>
                        <td>$${(y.mortgage_balance_usd / 1000).toFixed(0)}K</td>
                        <td class="pos">$${(y.equity_usd / 1000).toFixed(0)}K</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
