// EV vs ICE total cost comparison. Per path: depreciation + financing
// interest + fuel (electricity or gas) + maintenance + insurance +
// registration − residual − credits + optional battery replacement.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

const STATE = {
    annual_miles: 12000,
    hold_years: 10,
    apr_pct: 6.5,
    loan_term_months: 60,
    sales_tax_pct: 8.0,
    electricity_price_per_kwh_usd: 0.15,
    gasoline_price_per_gallon_usd: 3.50,
    ev: {
        purchase_price_usd: 45000,
        federal_credit_usd: 7500,
        state_credit_usd: 2500,
        kwh_per_100mi: 28.0,
        mpg: 0.0,
        maintenance_annual_usd: 400,
        insurance_annual_usd: 1400,
        registration_annual_usd: 200,
        residual_pct: 35,
    },
    ice: {
        purchase_price_usd: 35000,
        federal_credit_usd: 0,
        state_credit_usd: 0,
        kwh_per_100mi: 0,
        mpg: 28.0,
        maintenance_annual_usd: 900,
        insurance_annual_usd: 1500,
        registration_annual_usd: 200,
        residual_pct: 30,
    },
    battery_replacement_year: 0,
    battery_replacement_cost_usd: 0,
};

export async function renderEvVsIce(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ev_vs_ice.title">// EV VS ICE</span></h1>
        <p class="muted small" data-i18n-html="view.ev_vs_ice.intro">
            Total cost comparison of electric vs internal-combustion vehicles over an
            N-year ownership horizon. Includes purchase + financing + fuel (electricity vs
            gas) + maintenance + insurance + registration − residual, minus federal/state
            EV credits and optionally including a battery replacement cost mid-hold.
            Breakeven year = when cumulative EV cost crosses ICE cost (0 if EV cheaper
            upfront).
        </p>
        <div class="chart-panel">
            <h2>${esc(t('view.ev_vs_ice.h2.shared'))}</h2>
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.ev_vs_ice.field.miles">Annual miles</span>
                    <input type="number" id="ei-miles" step="500" min="0" max="200000" value="${STATE.annual_miles}" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.ev_vs_ice.field.hold">Hold years</span>
                    <input type="number" id="ei-hold" step="1" min="1" max="30" value="${STATE.hold_years}" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.ev_vs_ice.field.apr">APR %</span>
                    <input type="number" id="ei-apr" step="0.25" min="0" max="50" value="${STATE.apr_pct}" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.ev_vs_ice.field.term">Loan term months</span>
                    <input type="number" id="ei-term" step="12" min="1" max="120" value="${STATE.loan_term_months}" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.ev_vs_ice.field.tax">Sales tax %</span>
                    <input type="number" id="ei-tax" step="0.25" min="0" max="30" value="${STATE.sales_tax_pct}" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.ev_vs_ice.field.kwh">Electricity $/kWh</span>
                    <input type="number" id="ei-kwh" step="0.01" min="0" value="${STATE.electricity_price_per_kwh_usd}" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.ev_vs_ice.field.gas">Gasoline $/gallon</span>
                    <input type="number" id="ei-gas" step="0.1" min="0" value="${STATE.gasoline_price_per_gallon_usd}" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.ev_vs_ice.field.bat_year">Battery repl year (0=none)</span>
                    <input type="number" id="ei-bat-year" step="1" min="0" max="30" value="${STATE.battery_replacement_year}" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.ev_vs_ice.field.bat_cost">Battery repl cost $</span>
                    <input type="number" id="ei-bat-cost" step="500" min="0" value="${STATE.battery_replacement_cost_usd}" style="width:100%"></label>
            </div>
            ${vehiclePanel('ev', STATE.ev, 'EV', true)}
            ${vehiclePanel('ice', STATE.ice, 'ICE', false)}
            <div style="margin-top:1rem">
                <button class="btn btn-sm primary" id="ei-run" data-shortcut="r" data-i18n="view.ev_vs_ice.btn.run">⚡ Compare</button>
            </div>
            <div id="ei-result"></div>
        </div>
    `;
    bindInputs(mount);
    mount.querySelector('#ei-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

function vehiclePanel(prefix, v, label, isEv) {
    return `
        <h2 style="margin-top:1rem">${esc(label)}</h2>
        <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
            <label><span class="muted small">Purchase $</span>
                <input type="number" id="ei-${prefix}-price" step="500" min="0" value="${v.purchase_price_usd}"></label>
            ${isEv ? `
            <label><span class="muted small">Federal credit $</span>
                <input type="number" id="ei-${prefix}-fcred" step="100" min="0" value="${v.federal_credit_usd}"></label>
            <label><span class="muted small">State credit $</span>
                <input type="number" id="ei-${prefix}-scred" step="100" min="0" value="${v.state_credit_usd}"></label>
            <label><span class="muted small">kWh / 100mi</span>
                <input type="number" id="ei-${prefix}-eff" step="0.5" min="0" value="${v.kwh_per_100mi}"></label>
            ` : `
            <label><span class="muted small">MPG</span>
                <input type="number" id="ei-${prefix}-mpg" step="0.5" min="0" value="${v.mpg}"></label>
            `}
            <label><span class="muted small">Maintenance $/yr</span>
                <input type="number" id="ei-${prefix}-maint" step="50" min="0" value="${v.maintenance_annual_usd}"></label>
            <label><span class="muted small">Insurance $/yr</span>
                <input type="number" id="ei-${prefix}-ins" step="50" min="0" value="${v.insurance_annual_usd}"></label>
            <label><span class="muted small">Registration $/yr</span>
                <input type="number" id="ei-${prefix}-reg" step="25" min="0" value="${v.registration_annual_usd}"></label>
            <label><span class="muted small">Residual %</span>
                <input type="number" id="ei-${prefix}-res" step="5" min="0" max="100" value="${v.residual_pct}"></label>
        </div>
    `;
}

function bindInputs(mount) {
    const num = id => parseFloat(mount.querySelector('#' + id).value) || 0;
    const int = id => parseInt(mount.querySelector('#' + id).value, 10) || 0;
    mount.querySelectorAll('input').forEach(inp => inp.addEventListener('input', () => {
        STATE.annual_miles = int('ei-miles');
        STATE.hold_years = int('ei-hold');
        STATE.apr_pct = num('ei-apr');
        STATE.loan_term_months = int('ei-term');
        STATE.sales_tax_pct = num('ei-tax');
        STATE.electricity_price_per_kwh_usd = num('ei-kwh');
        STATE.gasoline_price_per_gallon_usd = num('ei-gas');
        STATE.battery_replacement_year = int('ei-bat-year');
        STATE.battery_replacement_cost_usd = num('ei-bat-cost');
        STATE.ev = {
            purchase_price_usd: num('ei-ev-price'),
            federal_credit_usd: num('ei-ev-fcred'),
            state_credit_usd: num('ei-ev-scred'),
            kwh_per_100mi: num('ei-ev-eff'),
            mpg: 0,
            maintenance_annual_usd: num('ei-ev-maint'),
            insurance_annual_usd: num('ei-ev-ins'),
            registration_annual_usd: num('ei-ev-reg'),
            residual_pct: num('ei-ev-res'),
        };
        STATE.ice = {
            purchase_price_usd: num('ei-ice-price'),
            federal_credit_usd: 0,
            state_credit_usd: 0,
            kwh_per_100mi: 0,
            mpg: num('ei-ice-mpg'),
            maintenance_annual_usd: num('ei-ice-maint'),
            insurance_annual_usd: num('ei-ice-ins'),
            registration_annual_usd: num('ei-ice-reg'),
            residual_pct: num('ei-ice-res'),
        };
    }));
}

async function runCompute(mount) {
    const result = mount.querySelector('#ei-result');
    result.innerHTML = `<p class="muted">${esc(t('view.ev_vs_ice.status.computing'))}</p>`;
    try {
        const r = await api('/ev-vs-ice/compute', { method: 'POST', body: JSON.stringify(STATE) });
        const winCls = r.net_winner === 'ev' ? 'pos' : '';
        const sCls = r.savings_ev_minus_ice_usd > 0 ? 'pos' : 'neg';
        const beFmt = r.years_to_breakeven == null ? '∞'
                    : r.years_to_breakeven === 0 ? esc(t('view.ev_vs_ice.field.day_one'))
                    : r.years_to_breakeven.toFixed(1) + ' yr';
        const pathTable = p => `
            <table class="trades" style="margin-top:0.5rem">
                <tbody>
                    <tr><td><strong>${esc(t('view.ev_vs_ice.row.principal'))}</strong></td><td>$${p.principal_financed_usd.toFixed(0)}</td></tr>
                    <tr><td><strong>${esc(t('view.ev_vs_ice.row.interest'))}</strong></td><td class="neg">$${p.financing_interest_usd.toFixed(0)}</td></tr>
                    <tr><td><strong>${esc(t('view.ev_vs_ice.row.fuel'))}</strong></td><td class="neg">$${p.fuel_total_usd.toFixed(0)}</td></tr>
                    <tr><td><strong>${esc(t('view.ev_vs_ice.row.maint'))}</strong></td><td class="neg">$${p.maintenance_total_usd.toFixed(0)}</td></tr>
                    <tr><td><strong>${esc(t('view.ev_vs_ice.row.ins'))}</strong></td><td>$${p.insurance_total_usd.toFixed(0)}</td></tr>
                    <tr><td><strong>${esc(t('view.ev_vs_ice.row.reg'))}</strong></td><td>$${p.registration_total_usd.toFixed(0)}</td></tr>
                    <tr><td><strong>${esc(t('view.ev_vs_ice.row.depreciation'))}</strong></td><td class="neg">$${p.depreciation_usd.toFixed(0)}</td></tr>
                    <tr><td><strong>${esc(t('view.ev_vs_ice.row.credits'))}</strong></td><td class="pos">−$${p.credits_applied_usd.toFixed(0)}</td></tr>
                    ${p.battery_replacement_usd > 0 ? `<tr><td><strong>${esc(t('view.ev_vs_ice.row.battery'))}</strong></td><td class="neg">$${p.battery_replacement_usd.toFixed(0)}</td></tr>` : ''}
                    <tr style="background:rgba(0,229,255,0.08)"><td><strong>${esc(t('view.ev_vs_ice.row.total'))}</strong></td><td><strong>$${p.total_cost_usd.toFixed(0)}</strong></td></tr>
                    <tr><td><strong>${esc(t('view.ev_vs_ice.row.cpm'))}</strong></td><td>$${p.cost_per_mile_usd.toFixed(3)}/mi</td></tr>
                </tbody>
            </table>
        `;
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.ev_vs_ice.field.winner'))}</div>
                    <strong class="${winCls}" style="font-size:1.4em;text-transform:uppercase">${esc(t('view.ev_vs_ice.winner.' + r.net_winner) || r.net_winner)}</strong></div>
                <div><div class="muted small">${esc(t('view.ev_vs_ice.field.savings'))}</div>
                    <strong class="${sCls}">$${Math.abs(r.savings_ev_minus_ice_usd).toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.ev_vs_ice.field.breakeven'))}</div>
                    <strong>${beFmt}</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.ev_vs_ice.h2.ev'))}</h2>
            ${pathTable(r.ev)}
            <h2 style="margin-top:1rem">${esc(t('view.ev_vs_ice.h2.ice'))}</h2>
            ${pathTable(r.ice)}
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
