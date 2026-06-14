// Required Minimum Distribution calculator (IRS Pub 590-B + SECURE 2.0).
// Reports current-year RMD, years until RMDs start, projection table.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import * as enh from '../calc_enhance.js';

export async function renderRmdCalculator(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.rmd_calculator.title">// REQUIRED MINIMUM DISTRIBUTION</span></h1>
        <p class="muted small" data-i18n-html="view.rmd_calculator.intro">
            Per <strong>SECURE 2.0 Act (2022)</strong> and IRS Publication 590-B:
            RMDs begin at <strong>age 73</strong> for those born 1951-1959 and at
            <strong>age 75</strong> for those born 1960+.
            <code>RMD = prior_year_end_balance / Uniform_Lifetime_Factor</code>.
            Missed-RMD penalty: 25% (10% if corrected within 2 years). Embedded Uniform
            Lifetime Table covers ages 72-120 per Pub 590-B Appendix B Table III.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.rmd_calculator.field.birth">Birth year</span>
                    <input type="number" id="rc-birth" step="1" min="1900" max="2100" value="1955" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.rmd_calculator.field.age">Current age</span>
                    <input type="number" id="rc-age" step="1" min="1" max="120" value="73" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.rmd_calculator.field.balance">Retirement balance $</span>
                    <input type="number" id="rc-balance" step="10000" min="0" value="1000000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.rmd_calculator.field.return">Expected return %/yr</span>
                    <input type="number" id="rc-return" step="0.25" min="-20" max="30" value="6" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.rmd_calculator.field.years">Project years</span>
                    <input type="number" id="rc-years" step="1" min="0" max="60" value="20" style="width:100%"></label>
            </div>
            <button class="btn btn-sm primary" id="rc-run" data-shortcut="r" data-i18n="view.rmd_calculator.btn.run">⚡ Compute RMD</button>
            <div id="rc-result"></div>
        </div>
    `;
    mount.querySelector('#rc-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#rc-result');
    const input = {
        birth_year: parseInt(mount.querySelector('#rc-birth').value, 10) || 0,
        current_age: parseInt(mount.querySelector('#rc-age').value, 10) || 0,
        balance_usd: parseFloat(mount.querySelector('#rc-balance').value) || 0,
        expected_annual_return_pct: parseFloat(mount.querySelector('#rc-return').value) || 0,
        project_years: parseInt(mount.querySelector('#rc-years').value, 10) || 0,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.rmd_calculator.status.computing'))}</p>`;
    try {
        const r = await api.request('/rmd-calculator/compute', { method: 'POST', body: JSON.stringify(input) });
        // RMD-per-age bar chart from the projection (the withdrawal ramp).
        const chart = enh.svgBarChart((r.projection || []).map(y => ({ label: String(y.age), value: y.rmd_amount_usd })));
        const untilCls = r.years_until_rmd <= 0 ? '' : 'pos';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.rmd_calculator.field.start_age'))}</div>
                    <strong>${r.rmd_start_age}</strong></div>
                <div><div class="muted small">${esc(t('view.rmd_calculator.field.years_until'))}</div>
                    <strong class="${untilCls}">${r.years_until_rmd}</strong></div>
                <div><div class="muted small">${esc(t('view.rmd_calculator.field.current_factor'))}</div>
                    <strong>${r.current_factor == null ? '—' : r.current_factor.toFixed(1)}</strong></div>
                <div><div class="muted small">${esc(t('view.rmd_calculator.field.current_rmd'))}</div>
                    <strong style="font-size:1.3em">${r.current_rmd_usd == null ? '—' : '$' + r.current_rmd_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.rmd_calculator.field.total_rmds'))}</div>
                    <strong class="neg">$${(r.total_rmds_through_projection_usd / 1000).toFixed(0)}K</strong></div>
            </div>
            ${chart}
            <div id="rmd-tools" class="ce-toolbar"></div>
            <h2 style="margin-top:1rem">${esc(t('view.rmd_calculator.h2.projection'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.rmd_calculator.th.age">Age</th>
                    <th data-i18n="view.rmd_calculator.th.start">Start balance</th>
                    <th data-i18n="view.rmd_calculator.th.factor">Factor</th>
                    <th data-i18n="view.rmd_calculator.th.rmd">RMD</th>
                    <th data-i18n="view.rmd_calculator.th.end">End balance</th>
                </tr></thead>
                <tbody>${(r.projection || []).map(y => `
                    <tr>
                        <td>${y.age}</td>
                        <td>$${(y.start_balance_usd / 1000).toFixed(0)}K</td>
                        <td>${y.rmd_factor.toFixed(1)}</td>
                        <td class="neg">$${y.rmd_amount_usd.toFixed(0)}</td>
                        <td>$${(y.end_balance_after_rmd_usd / 1000).toFixed(0)}K</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
        // Projection export (Copy / CSV). No permalink — id-based inputs.
        enh.mountToolbar(mount.querySelector('#rmd-tools'), {
            viewId: 'rmd-calculator',
            link: false,
            filename: 'rmd-calculator.csv',
            getRows: () => [['age', 'start_balance_usd', 'rmd_factor', 'rmd_amount_usd', 'end_balance_usd'],
                ...(r.projection || []).map(y => [y.age, y.start_balance_usd, y.rmd_factor, y.rmd_amount_usd, y.end_balance_after_rmd_usd])],
        });
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
