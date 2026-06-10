// Lease vs Buy car NPV comparison over an N-year analysis horizon.
// Strips out the shared components (fuel/insurance/registration are
// equal for the same car) and compares only the contractual costs:
// lease payments + drive-off + disposition vs depreciation + interest
// + opportunity cost on down payment. Reports winner + breakeven
// monthly lease payment.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderLeaseVsBuyCar(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.lease_vs_buy_car.title">// LEASE VS BUY CAR</span></h1>
        <p class="muted small" data-i18n-html="view.lease_vs_buy_car.intro">
            Compares lease vs buy over an N-year analysis horizon, focusing on
            <strong>contractual differences only</strong> (fuel/insurance/registration are
            the same for both paths and cancel). LEASE = monthly payments × horizon + repeated
            drive-off + disposition fees − opportunity return on cash you'd otherwise put down.
            BUY = depreciation + financing interest in horizon + opportunity cost on down
            payment. Breakeven monthly = lease payment that equalises the two paths.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.lease_vs_buy_car.field.price">Vehicle price $</span>
                    <input type="number" id="lb-price" step="500" min="0" value="35000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.lease_vs_buy_car.field.tax">Sales tax %</span>
                    <input type="number" id="lb-tax" step="0.25" min="0" max="30" value="8" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.lease_vs_buy_car.field.lease_pay">Monthly lease $</span>
                    <input type="number" id="lb-lease" step="25" min="0" value="400" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.lease_vs_buy_car.field.lease_term">Lease term months</span>
                    <input type="number" id="lb-lt" step="12" min="1" max="120" value="36" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.lease_vs_buy_car.field.drive_off">Drive-off cost $</span>
                    <input type="number" id="lb-drive" step="100" min="0" value="2500" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.lease_vs_buy_car.field.disposition">Disposition fee $</span>
                    <input type="number" id="lb-disp" step="50" min="0" value="400" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.lease_vs_buy_car.field.down">Down payment $ (buy)</span>
                    <input type="number" id="lb-down" step="500" min="0" value="5000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.lease_vs_buy_car.field.apr">APR % (buy)</span>
                    <input type="number" id="lb-apr" step="0.25" min="0" max="50" value="6.5" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.lease_vs_buy_car.field.loan_term">Loan term months (buy)</span>
                    <input type="number" id="lb-loan" step="12" min="1" max="120" value="60" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.lease_vs_buy_car.field.residual">Residual % at horizon (buy)</span>
                    <input type="number" id="lb-res" step="5" min="0" max="100" value="40" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.lease_vs_buy_car.field.horizon">Analysis years</span>
                    <input type="number" id="lb-horizon" step="1" min="1" max="30" value="6" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.lease_vs_buy_car.field.return">Investment return %/yr</span>
                    <input type="number" id="lb-return" step="0.5" min="-20" max="30" value="7" style="width:100%"></label>
            </div>
            <button class="btn btn-sm primary" id="lb-run" data-shortcut="r" data-i18n="view.lease_vs_buy_car.btn.run">⚡ Compare</button>
            <div id="lb-result"></div>
        </div>
    `;
    mount.querySelector('#lb-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#lb-result');
    const input = {
        vehicle_price_usd: parseFloat(mount.querySelector('#lb-price').value) || 0,
        sales_tax_pct: parseFloat(mount.querySelector('#lb-tax').value) || 0,
        monthly_lease_payment_usd: parseFloat(mount.querySelector('#lb-lease').value) || 0,
        lease_term_months: parseInt(mount.querySelector('#lb-lt').value, 10) || 36,
        drive_off_cost_usd: parseFloat(mount.querySelector('#lb-drive').value) || 0,
        disposition_fee_usd: parseFloat(mount.querySelector('#lb-disp').value) || 0,
        down_payment_usd: parseFloat(mount.querySelector('#lb-down').value) || 0,
        apr_pct: parseFloat(mount.querySelector('#lb-apr').value) || 0,
        loan_term_months: parseInt(mount.querySelector('#lb-loan').value, 10) || 60,
        residual_at_horizon_pct: parseFloat(mount.querySelector('#lb-res').value) || 0,
        analysis_years: parseInt(mount.querySelector('#lb-horizon').value, 10) || 6,
        investment_return_pct: parseFloat(mount.querySelector('#lb-return').value) || 0,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.lease_vs_buy_car.status.computing'))}</p>`;
    try {
        const r = await api('/lease-vs-buy-car/compute', { method: 'POST', body: JSON.stringify(input) });
        const winnerCls = r.net_winner === 'buy' ? 'pos' : '';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.lease_vs_buy_car.field.lease_total'))}</div>
                    <strong>$${r.lease_total_cost_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.lease_vs_buy_car.field.buy_total'))}</div>
                    <strong>$${r.buy_total_cost_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.lease_vs_buy_car.field.winner'))}</div>
                    <strong class="${winnerCls}" style="font-size:1.4em;text-transform:uppercase">${esc(t('view.lease_vs_buy_car.winner.' + r.net_winner) || r.net_winner)}</strong></div>
                <div><div class="muted small">${esc(t('view.lease_vs_buy_car.field.savings'))}</div>
                    <strong class="pos">$${r.savings_winner_minus_loser_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.lease_vs_buy_car.field.breakeven'))}</div>
                    <strong>$${r.breakeven_monthly_lease_usd.toFixed(2)}/mo</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.lease_vs_buy_car.h2.lease'))}</h2>
            <table class="trades">
                <tbody>
                    <tr><td><strong>${esc(t('view.lease_vs_buy_car.row.lease_payments'))}</strong></td>
                        <td>$${r.lease_total_payments_usd.toFixed(0)}</td></tr>
                    <tr><td><strong>${esc(t('view.lease_vs_buy_car.row.lease_fees'))}</strong></td>
                        <td>$${r.lease_drive_off_plus_disposition_usd.toFixed(0)}</td></tr>
                    <tr><td><strong>${esc(t('view.lease_vs_buy_car.row.lease_opp_credit'))}</strong></td>
                        <td class="pos">−$${r.lease_opportunity_credit_usd.toFixed(0)}</td></tr>
                </tbody>
            </table>
            <h2 style="margin-top:1rem">${esc(t('view.lease_vs_buy_car.h2.buy'))}</h2>
            <table class="trades">
                <tbody>
                    <tr><td><strong>${esc(t('view.lease_vs_buy_car.row.buy_principal'))}</strong></td>
                        <td>$${r.buy_principal_usd.toFixed(0)}</td></tr>
                    <tr><td><strong>${esc(t('view.lease_vs_buy_car.row.buy_monthly_pi'))}</strong></td>
                        <td>$${r.buy_monthly_pi_usd.toFixed(2)}/mo</td></tr>
                    <tr><td><strong>${esc(t('view.lease_vs_buy_car.row.buy_interest'))}</strong></td>
                        <td class="neg">$${r.buy_total_interest_paid_in_horizon_usd.toFixed(0)}</td></tr>
                    <tr><td><strong>${esc(t('view.lease_vs_buy_car.row.buy_residual'))}</strong></td>
                        <td>$${r.buy_residual_value_usd.toFixed(0)}</td></tr>
                    <tr><td><strong>${esc(t('view.lease_vs_buy_car.row.buy_depreciation'))}</strong></td>
                        <td class="neg">$${r.buy_depreciation_usd.toFixed(0)}</td></tr>
                    <tr><td><strong>${esc(t('view.lease_vs_buy_car.row.buy_down_opp'))}</strong></td>
                        <td class="neg">$${r.buy_down_opportunity_cost_usd.toFixed(0)}</td></tr>
                </tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
