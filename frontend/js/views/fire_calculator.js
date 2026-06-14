// FIRE (Financial Independence / Retire Early) calculator. Standard
// projection: current portfolio + monthly contributions compounded at
// expected return, vs target net worth + target retirement age. Reports
// years-to-target, final NW, required savings to hit target by date,
// year-by-year projection, sensitivity table.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import * as enh from '../calc_enhance.js';

export async function renderFireCalculator(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.fire_calculator.title">// FIRE RETIREMENT CALCULATOR</span></h1>
        <p class="muted small" data-i18n-html="view.fire_calculator.intro">
            Financial Independence / Retire Early projection. Given current portfolio,
            monthly contribution, expected return, and target net worth + retirement age:
            computes <strong>years-to-target</strong>, <strong>final NW at target age</strong>,
            <strong>required monthly savings</strong> to hit target by date, year-by-year
            projection, and a <strong>sensitivity table</strong> (return ± 2% × contribution
            ± 20%). Withdrawal income at 4% per Trinity Study (Bengen 1994).
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small" data-i18n="view.fire_calculator.field.current">Current portfolio $</span>
                    <input type="number" id="fire-current" step="1000" min="0" value="100000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.fire_calculator.field.monthly">Monthly contribution $</span>
                    <input type="number" id="fire-monthly" step="100" min="0" value="1000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.fire_calculator.field.return">Expected annual return %</span>
                    <input type="number" id="fire-return" step="0.5" min="-50" max="50" value="7" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.fire_calculator.field.target">Target net worth $</span>
                    <input type="number" id="fire-target" step="100000" min="1" value="2500000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.fire_calculator.field.current_age">Current age</span>
                    <input type="number" id="fire-current-age" step="1" min="1" max="110" value="35" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.fire_calculator.field.target_age">Target retirement age</span>
                    <input type="number" id="fire-target-age" step="1" min="2" max="110" value="60" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.fire_calculator.field.withdrawal_rate">Withdrawal rate %</span>
                    <input type="number" id="fire-withdrawal" step="0.25" min="0.25" max="20" value="4" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="fire-run" data-shortcut="r" data-i18n="view.fire_calculator.btn.run">⚡ Compute Projection</button>
            <span class="muted small" id="fire-meta" style="margin-left:1rem"></span>
            <div id="fire-result"></div>
        </div>
    `;
    mount.querySelector('#fire-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#fire-result');
    const input = {
        current_portfolio_usd: parseFloat(mount.querySelector('#fire-current').value) || 0,
        monthly_contribution_usd: parseFloat(mount.querySelector('#fire-monthly').value) || 0,
        expected_annual_return_pct: parseFloat(mount.querySelector('#fire-return').value) || 7,
        target_net_worth_usd: parseFloat(mount.querySelector('#fire-target').value) || 1000000,
        current_age: parseInt(mount.querySelector('#fire-current-age').value, 10) || 35,
        target_retirement_age: parseInt(mount.querySelector('#fire-target-age').value, 10) || 60,
        safe_withdrawal_rate_pct: parseFloat(mount.querySelector('#fire-withdrawal').value) || 4,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.fire_calculator.status.computing'))}</p>`;
    try {
        const r = await api.request('/fire-calculator/compute', { method: 'POST', body: JSON.stringify(input) });
        // Net-worth trajectory straight from the year-by-year projection.
        const chart = enh.svgLineChart((r.yearly_projection || []).map(y => ({ x: y.age, y: y.end_balance_usd })), { xlabel: 'age', ylabel: 'net worth $' });
        const yearsCls = r.years_to_target == null
            ? 'neg'
            : r.years_to_target <= (input.target_retirement_age - input.current_age)
                ? 'pos' : 'neg';
        const requiredCls = r.required_monthly_savings_for_target_date == null
            ? 'muted'
            : r.required_monthly_savings_for_target_date <= input.monthly_contribution_usd
                ? 'pos' : 'neg';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.fire_calculator.field.years_to_target'))}</div>
                    <strong class="${yearsCls}" style="font-size:1.4em">${r.years_to_target == null ? '∞' : r.years_to_target.toFixed(1)}</strong></div>
                <div><div class="muted small">${esc(t('view.fire_calculator.field.final_nw'))}</div>
                    <strong style="font-size:1.4em">$${(r.final_net_worth_at_target_age / 1000).toFixed(0)}K</strong></div>
                <div><div class="muted small">${esc(t('view.fire_calculator.field.required_savings'))}</div>
                    <strong class="${requiredCls}">${r.required_monthly_savings_for_target_date == null ? '—' : '$' + r.required_monthly_savings_for_target_date.toFixed(0) + '/mo'}</strong></div>
                <div><div class="muted small">${esc(t('view.fire_calculator.field.swr_income'))}</div>
                    <strong class="pos">$${(r.safe_withdrawal_income_annual_usd / 1000).toFixed(1)}K/yr</strong></div>
            </div>
            ${chart}
            <div id="fire-tools" class="ce-toolbar"></div>
            <h2 style="margin-top:1rem">${esc(t('view.fire_calculator.h2.sensitivity'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.fire_calculator.th.return_delta">Return Δ</th>
                    <th data-i18n="view.fire_calculator.th.contrib_delta">Contrib Δ</th>
                    <th data-i18n="view.fire_calculator.th.years">Years to Target</th>
                    <th data-i18n="view.fire_calculator.th.final_nw">Final NW at Target Age</th>
                </tr></thead>
                <tbody>${(r.sensitivity || []).map(c => `
                    <tr>
                        <td>${c.return_pct_delta >= 0 ? '+' : ''}${c.return_pct_delta}%</td>
                        <td>${c.contribution_pct_delta >= 0 ? '+' : ''}${c.contribution_pct_delta}%</td>
                        <td>${c.years_to_target == null ? '∞' : c.years_to_target.toFixed(1)}</td>
                        <td>$${(c.final_nw_at_target_age / 1000).toFixed(0)}K</td>
                    </tr>
                `).join('')}</tbody>
            </table>
            <h2 style="margin-top:1rem">${esc(t('view.fire_calculator.h2.projection'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.fire_calculator.th.age">Age</th>
                    <th data-i18n="view.fire_calculator.th.start">Start $</th>
                    <th data-i18n="view.fire_calculator.th.contrib">Contrib $</th>
                    <th data-i18n="view.fire_calculator.th.growth">Growth $</th>
                    <th data-i18n="view.fire_calculator.th.end">End $</th>
                </tr></thead>
                <tbody>${(r.yearly_projection || []).map(y => `
                    <tr>
                        <td>${y.age}</td>
                        <td>$${(y.start_balance_usd / 1000).toFixed(0)}K</td>
                        <td>$${(y.contributions_usd / 1000).toFixed(0)}K</td>
                        <td class="pos">$${(y.growth_usd / 1000).toFixed(0)}K</td>
                        <td><strong>$${(y.end_balance_usd / 1000).toFixed(0)}K</strong></td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
        // Export the year-by-year projection (Copy / CSV). No permalink — id-based inputs.
        enh.mountToolbar(mount.querySelector('#fire-tools'), {
            viewId: 'fire-calculator',
            link: false,
            filename: 'fire-projection.csv',
            getRows: () => [['age', 'start', 'contributions', 'growth', 'end'],
                ...(r.yearly_projection || []).map(y => [y.age, y.start_balance_usd, y.contributions_usd, y.growth_usd, y.end_balance_usd])],
        });
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
