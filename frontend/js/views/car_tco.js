// Vehicle Total Cost of Ownership calculator. Depreciation +
// financing interest + fuel + insurance + maintenance (5%/yr
// inflator) + registration − residual value. Reports per-year +
// totals + cost-per-mile.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderCarTco(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.car_tco.title">// VEHICLE TCO</span></h1>
        <p class="muted small" data-i18n-html="view.car_tco.intro">
            Total cost of ownership over the years you hold the car. Sums depreciation
            (purchase + tax − residual), financing interest, fuel, insurance, maintenance
            (5%/yr inflator), registration. Reports total, cost-per-mile, and a per-year
            breakdown for budgeting.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.car_tco.field.price">Purchase price $</span>
                    <input type="number" id="ct-price" step="500" min="0" value="35000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.car_tco.field.down">Down payment $</span>
                    <input type="number" id="ct-down" step="500" min="0" value="5000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.car_tco.field.tax">Sales tax %</span>
                    <input type="number" id="ct-tax" step="0.25" min="0" max="30" value="8" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.car_tco.field.apr">APR %</span>
                    <input type="number" id="ct-apr" step="0.25" min="0" max="50" value="6.5" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.car_tco.field.term">Loan term months</span>
                    <input type="number" id="ct-term" step="12" min="0" max="120" value="60" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.car_tco.field.hold">Hold years</span>
                    <input type="number" id="ct-hold" step="1" min="1" max="30" value="7" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.car_tco.field.miles">Annual miles</span>
                    <input type="number" id="ct-miles" step="500" min="0" max="200000" value="12000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.car_tco.field.mpg">MPG</span>
                    <input type="number" id="ct-mpg" step="0.5" min="0" value="28" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.car_tco.field.fuel">Fuel $ / gallon</span>
                    <input type="number" id="ct-fuel" step="0.1" min="0" value="3.50" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.car_tco.field.insurance">Insurance $/yr</span>
                    <input type="number" id="ct-insurance" step="50" min="0" value="1500" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.car_tco.field.maintenance">Maintenance $/yr (yr 1)</span>
                    <input type="number" id="ct-maint" step="50" min="0" value="800" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.car_tco.field.registration">Registration $/yr</span>
                    <input type="number" id="ct-reg" step="25" min="0" value="200" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.car_tco.field.residual">Residual %</span>
                    <input type="number" id="ct-res" step="5" min="0" max="100" value="30" style="width:100%"></label>
            </div>
            <button class="btn btn-sm primary" id="ct-run" data-shortcut="r" data-i18n="view.car_tco.btn.run">⚡ Compute TCO</button>
            <div id="ct-result"></div>
        </div>
    `;
    mount.querySelector('#ct-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#ct-result');
    const input = {
        purchase_price_usd: parseFloat(mount.querySelector('#ct-price').value) || 0,
        down_payment_usd: parseFloat(mount.querySelector('#ct-down').value) || 0,
        sales_tax_pct: parseFloat(mount.querySelector('#ct-tax').value) || 0,
        apr_pct: parseFloat(mount.querySelector('#ct-apr').value) || 0,
        loan_term_months: parseInt(mount.querySelector('#ct-term').value, 10) || 0,
        hold_years: parseInt(mount.querySelector('#ct-hold').value, 10) || 7,
        annual_miles: parseInt(mount.querySelector('#ct-miles').value, 10) || 0,
        mpg: parseFloat(mount.querySelector('#ct-mpg').value) || 0,
        fuel_price_per_gallon_usd: parseFloat(mount.querySelector('#ct-fuel').value) || 0,
        insurance_annual_usd: parseFloat(mount.querySelector('#ct-insurance').value) || 0,
        maintenance_annual_usd: parseFloat(mount.querySelector('#ct-maint').value) || 0,
        registration_annual_usd: parseFloat(mount.querySelector('#ct-reg').value) || 0,
        residual_pct_after_hold: parseFloat(mount.querySelector('#ct-res').value) || 0,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.car_tco.status.computing'))}</p>`;
    try {
        const r = await api.request('/car-tco/compute', { method: 'POST', body: JSON.stringify(input) });
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.car_tco.field.total'))}</div>
                    <strong style="font-size:1.4em">$${r.total_cost_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.car_tco.field.cpm'))}</div>
                    <strong>$${r.cost_per_mile_usd.toFixed(3)}/mi</strong></div>
                <div><div class="muted small">${esc(t('view.car_tco.field.depreciation'))}</div>
                    <strong class="neg">$${r.depreciation_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.car_tco.field.residual'))}</div>
                    <strong>$${r.residual_value_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.car_tco.field.monthly'))}</div>
                    <strong>$${r.monthly_payment_usd.toFixed(2)}/mo</strong></div>
                <div><div class="muted small">${esc(t('view.car_tco.field.interest'))}</div>
                    <strong class="neg">$${r.total_financing_interest_usd.toFixed(0)}</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.car_tco.h2.totals'))}</h2>
            <table class="trades">
                <tbody>
                    <tr><td><strong>${esc(t('view.car_tco.row.fuel'))}</strong></td><td>$${r.total_fuel_usd.toFixed(0)}</td></tr>
                    <tr><td><strong>${esc(t('view.car_tco.row.insurance'))}</strong></td><td>$${r.total_insurance_usd.toFixed(0)}</td></tr>
                    <tr><td><strong>${esc(t('view.car_tco.row.maintenance'))}</strong></td><td>$${r.total_maintenance_usd.toFixed(0)}</td></tr>
                    <tr><td><strong>${esc(t('view.car_tco.row.registration'))}</strong></td><td>$${r.total_registration_usd.toFixed(0)}</td></tr>
                </tbody>
            </table>
            <h2 style="margin-top:1rem">${esc(t('view.car_tco.h2.yearly'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.car_tco.th.year">Yr</th>
                    <th data-i18n="view.car_tco.th.financing">Financing</th>
                    <th data-i18n="view.car_tco.th.fuel">Fuel</th>
                    <th data-i18n="view.car_tco.th.ins">Insurance</th>
                    <th data-i18n="view.car_tco.th.maint">Maintenance</th>
                    <th data-i18n="view.car_tco.th.reg">Registration</th>
                    <th data-i18n="view.car_tco.th.total_yr">Year total</th>
                </tr></thead>
                <tbody>${(r.yearly || []).map(y => `
                    <tr>
                        <td>${y.year}</td>
                        <td>$${y.financing_usd.toFixed(0)}</td>
                        <td>$${y.fuel_usd.toFixed(0)}</td>
                        <td>$${y.insurance_usd.toFixed(0)}</td>
                        <td>$${y.maintenance_usd.toFixed(0)}</td>
                        <td>$${y.registration_usd.toFixed(0)}</td>
                        <td><strong>$${y.total_year_usd.toFixed(0)}</strong></td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
