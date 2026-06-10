// Treasury Inflation-Protected Securities (TIPS) calculator.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderTipsBond(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.tips_bond.title">// TIPS BOND</span></h1>
        <p class="muted small" data-i18n-html="view.tips_bond.intro">
            Treasury Inflation-Protected Securities (TIPS). Principal adjusts with CPI-U
            semi-annually; fixed REAL coupon paid on the adjusted principal. At maturity,
            <strong>greater of</strong> (adjusted principal, original face) is repaid
            (deflation floor at par). Reports per-period principal accretion + coupons,
            total nominal vs real return.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.tips_bond.field.face">Face value $</span>
                    <input type="number" id="tb-face" step="500" min="0" value="1000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.tips_bond.field.coupon">Real coupon rate %</span>
                    <input type="number" id="tb-coupon" step="0.125" min="-10" max="20" value="1.875" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.tips_bond.field.term">Term years</span>
                    <input type="number" id="tb-term" step="1" min="1" max="50" value="10" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.tips_bond.field.inflation">Assumed annual CPI inflation %</span>
                    <input type="number" id="tb-cpi" step="0.25" min="-20" max="30" value="2.5" style="width:100%"></label>
            </div>
            <button class="btn btn-sm primary" id="tb-run" data-shortcut="r" data-i18n="view.tips_bond.btn.run">⚡ Compute TIPS</button>
            <div id="tb-result"></div>
        </div>
    `;
    mount.querySelector('#tb-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#tb-result');
    const input = {
        face_value_usd: parseFloat(mount.querySelector('#tb-face').value) || 0,
        real_coupon_rate_pct: parseFloat(mount.querySelector('#tb-coupon').value) || 0,
        term_years: parseInt(mount.querySelector('#tb-term').value, 10) || 0,
        annual_cpi_inflation_pct: parseFloat(mount.querySelector('#tb-cpi').value) || 0,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.tips_bond.status.computing'))}</p>`;
    try {
        const r = await api.request('/tips-bond/compute', { method: 'POST', body: JSON.stringify(input) });
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.tips_bond.field.maturity'))}</div>
                    <strong>${r.maturity_periods} (semi)</strong></div>
                <div><div class="muted small">${esc(t('view.tips_bond.field.final_adj'))}</div>
                    <strong>$${r.final_adjusted_principal_usd.toFixed(2)}</strong></div>
                <div><div class="muted small">${esc(t('view.tips_bond.field.paid_at_mat'))}</div>
                    <strong style="font-size:1.3em">$${r.final_principal_paid_at_maturity_usd.toFixed(2)}</strong></div>
                <div><div class="muted small">${esc(t('view.tips_bond.field.total_coupons'))}</div>
                    <strong class="pos">$${r.total_coupons_usd.toFixed(2)}</strong></div>
                <div><div class="muted small">${esc(t('view.tips_bond.field.nominal_return'))}</div>
                    <strong>${r.total_nominal_return_pct.toFixed(2)}%</strong></div>
                <div><div class="muted small">${esc(t('view.tips_bond.field.real_return'))}</div>
                    <strong style="font-size:1.3em">${r.total_real_return_pct.toFixed(2)}%</strong></div>
                <div><div class="muted small">${esc(t('view.tips_bond.field.deflation_floor'))}</div>
                    <strong class="${r.deflation_floor_active ? 'neg' : 'pos'}">${r.deflation_floor_active ? 'ACTIVE' : 'inactive'}</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.tips_bond.h2.schedule'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.tips_bond.th.period">Period</th>
                    <th data-i18n="view.tips_bond.th.principal">Adj principal</th>
                    <th data-i18n="view.tips_bond.th.coupon">Coupon</th>
                </tr></thead>
                <tbody>${(r.schedule || []).map(p => `
                    <tr>
                        <td>${p.period}</td>
                        <td>$${p.adjusted_principal_usd.toFixed(2)}</td>
                        <td class="pos">$${p.coupon_usd.toFixed(2)}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
