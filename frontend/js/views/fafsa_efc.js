// FAFSA / Student Aid Index (SAI) estimator. Simplified formula
// approximating the FAFSA Simplification Act methodology.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderFafsaEfc(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.fafsa_efc.title">// FAFSA SAI ESTIMATOR</span></h1>
        <p class="muted small" data-i18n-html="view.fafsa_efc.intro">
            Simplified Student Aid Index (SAI, the successor to EFC under the FAFSA
            Simplification Act, effective 2024-25). SAI = parent income contribution
            (graduated 22-47% on available income) + parent asset contribution (5.64%) +
            student income contribution (50% above $9,410 allowance) + student asset
            contribution (20%). Big change vs old EFC: SAI does NOT divide by number in
            college. This is a simplified estimate, not an official calculation.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.fafsa_efc.field.parent_agi">Parent AGI $</span>
                    <input type="number" id="fa-pagi" step="2500" min="0" value="100000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.fafsa_efc.field.parent_assets">Parent assets $</span>
                    <input type="number" id="fa-passets" step="1000" min="0" value="50000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.fafsa_efc.field.student_agi">Student AGI $</span>
                    <input type="number" id="fa-sagi" step="500" min="0" value="0" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.fafsa_efc.field.student_assets">Student assets $</span>
                    <input type="number" id="fa-sassets" step="500" min="0" value="0" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.fafsa_efc.field.household">Household size</span>
                    <input type="number" id="fa-house" step="1" min="1" max="20" value="4" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.fafsa_efc.field.in_college">Dependents in college</span>
                    <input type="number" id="fa-incol" step="1" min="1" max="10" value="1" style="width:100%"></label>
            </div>
            <button class="btn btn-sm primary" id="fa-run" data-shortcut="r" data-i18n="view.fafsa_efc.btn.run">⚡ Estimate SAI</button>
            <div id="fa-result"></div>
        </div>
    `;
    mount.querySelector('#fa-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#fa-result');
    const input = {
        parent_agi_usd: parseFloat(mount.querySelector('#fa-pagi').value) || 0,
        parent_assets_usd: parseFloat(mount.querySelector('#fa-passets').value) || 0,
        student_agi_usd: parseFloat(mount.querySelector('#fa-sagi').value) || 0,
        student_assets_usd: parseFloat(mount.querySelector('#fa-sassets').value) || 0,
        household_size: parseInt(mount.querySelector('#fa-house').value, 10) || 4,
        dependents_in_college: parseInt(mount.querySelector('#fa-incol').value, 10) || 1,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.fafsa_efc.status.computing'))}</p>`;
    try {
        const r = await api('/fafsa-efc/compute', { method: 'POST', body: JSON.stringify(input) });
        const tierCls = r.aid_tier === 'max_pell' || r.aid_tier === 'pell_eligible' ? 'pos'
                       : r.aid_tier === 'full_pay_likely' ? 'neg' : '';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.fafsa_efc.field.sai'))}</div>
                    <strong style="font-size:1.4em">$${r.sai_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.fafsa_efc.field.sai_per'))}</div>
                    <strong>$${r.sai_per_student_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.fafsa_efc.field.tier'))}</div>
                    <strong class="${tierCls}" style="text-transform:uppercase">${esc(t('view.fafsa_efc.tier.' + r.aid_tier) || r.aid_tier)}</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.fafsa_efc.h2.breakdown'))}</h2>
            <table class="trades">
                <tbody>
                    <tr><td><strong>${esc(t('view.fafsa_efc.row.parent_protection'))}</strong></td>
                        <td>$${r.parent_income_protection_usd.toFixed(0)}</td></tr>
                    <tr><td><strong>${esc(t('view.fafsa_efc.row.parent_available'))}</strong></td>
                        <td>$${r.parent_available_income_usd.toFixed(0)}</td></tr>
                    <tr><td><strong>${esc(t('view.fafsa_efc.row.parent_income_contrib'))}</strong></td>
                        <td>$${r.parent_income_contribution_usd.toFixed(0)}</td></tr>
                    <tr><td><strong>${esc(t('view.fafsa_efc.row.parent_asset_contrib'))}</strong></td>
                        <td>$${r.parent_asset_contribution_usd.toFixed(0)}</td></tr>
                    <tr><td><strong>${esc(t('view.fafsa_efc.row.student_income_contrib'))}</strong></td>
                        <td>$${r.student_income_contribution_usd.toFixed(0)}</td></tr>
                    <tr><td><strong>${esc(t('view.fafsa_efc.row.student_asset_contrib'))}</strong></td>
                        <td>$${r.student_asset_contribution_usd.toFixed(0)}</td></tr>
                </tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
