// 529 college savings planner. Projects 4-year college cost
// inflated to year of enrollment, computes monthly contribution
// needed at expected investment return.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderCollege529(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.college_529.title">// 529 COLLEGE PLANNER</span></h1>
        <p class="muted small" data-i18n-html="view.college_529.intro">
            Projects 4-year college cost inflated to the year the child enrolls and computes
            the monthly contribution needed at your expected investment return. Tuition
            inflation has historically been ~5%/yr (dropping toward 3% recently). 529
            withdrawals for qualified higher-ed expenses are <strong>federal-tax-free</strong>;
            most states also exempt them from state tax.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.college_529.field.child_age">Child age</span>
                    <input type="number" id="c5-age" step="1" min="0" max="25" value="5" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.college_529.field.start_age">College start age</span>
                    <input type="number" id="c5-start" step="1" min="1" max="30" value="18" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.college_529.field.annual_cost">Annual cost today $</span>
                    <input type="number" id="c5-cost" step="1000" min="0" value="30000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.college_529.field.infl">Tuition inflation %/yr</span>
                    <input type="number" id="c5-infl" step="0.5" min="-10" max="30" value="5" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.college_529.field.years">Years in college</span>
                    <input type="number" id="c5-years" step="1" min="1" max="10" value="4" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.college_529.field.balance">Current 529 balance $</span>
                    <input type="number" id="c5-balance" step="1000" min="0" value="20000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.college_529.field.return">Expected return %/yr</span>
                    <input type="number" id="c5-return" step="0.5" min="-20" max="30" value="6" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.college_529.field.contrib">Current contribution $/mo</span>
                    <input type="number" id="c5-contrib" step="50" min="0" value="300" style="width:100%"></label>
            </div>
            <button class="btn btn-sm primary" id="c5-run" data-shortcut="r" data-i18n="view.college_529.btn.run">⚡ Compute 529</button>
            <div id="c5-result"></div>
        </div>
    `;
    mount.querySelector('#c5-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#c5-result');
    const input = {
        child_age_years: parseInt(mount.querySelector('#c5-age').value, 10) || 0,
        college_start_age: parseInt(mount.querySelector('#c5-start').value, 10) || 18,
        annual_cost_today_usd: parseFloat(mount.querySelector('#c5-cost').value) || 0,
        tuition_inflation_pct: parseFloat(mount.querySelector('#c5-infl').value) || 0,
        years_in_college: parseInt(mount.querySelector('#c5-years').value, 10) || 4,
        current_529_balance_usd: parseFloat(mount.querySelector('#c5-balance').value) || 0,
        expected_annual_return_pct: parseFloat(mount.querySelector('#c5-return').value) || 0,
        current_monthly_contribution_usd: parseFloat(mount.querySelector('#c5-contrib').value) || 0,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.college_529.status.computing'))}</p>`;
    try {
        const r = await api.request('/college-529/compute', { method: 'POST', body: JSON.stringify(input) });
        const okCls = r.on_track ? 'pos' : 'neg';
        const reqCls = input.current_monthly_contribution_usd >= r.required_monthly_contribution_usd ? 'pos' : 'neg';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.college_529.field.years_until'))}</div>
                    <strong style="font-size:1.3em">${r.years_until_college}</strong></div>
                <div><div class="muted small">${esc(t('view.college_529.field.total_cost'))}</div>
                    <strong style="font-size:1.3em">$${(r.total_projected_cost_usd / 1000).toFixed(0)}K</strong></div>
                <div><div class="muted small">${esc(t('view.college_529.field.projected_savings'))}</div>
                    <strong>$${(r.projected_savings_at_start_usd / 1000).toFixed(0)}K</strong></div>
                <div><div class="muted small">${esc(t('view.college_529.field.shortfall'))}</div>
                    <strong class="${r.shortfall_usd > 0 ? 'neg' : 'pos'}">$${(r.shortfall_usd / 1000).toFixed(0)}K</strong></div>
                <div><div class="muted small">${esc(t('view.college_529.field.required'))}</div>
                    <strong class="${reqCls}">$${r.required_monthly_contribution_usd.toFixed(0)}/mo</strong></div>
                <div><div class="muted small">${esc(t('view.college_529.field.on_track'))}</div>
                    <strong class="${okCls}">${r.on_track ? '✓ ' + esc(t('view.college_529.status.on_track')) : '✗ ' + esc(t('view.college_529.status.short'))}</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.college_529.h2.per_year'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.college_529.th.year">Year</th>
                    <th data-i18n="view.college_529.th.age">Age</th>
                    <th data-i18n="view.college_529.th.cost">Projected cost</th>
                </tr></thead>
                <tbody>${(r.per_year_cost || []).map(y => `
                    <tr>
                        <td>${y.year_index}</td>
                        <td>${y.year_age}</td>
                        <td>$${(y.cost_usd / 1000).toFixed(1)}K</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
