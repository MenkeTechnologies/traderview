// Emergency-fund readiness. Given monthly expenses + current fund +
// target months + monthly contribution: computes months covered now,
// target amount, gap, months-to-target, status, and a 3/6/9/12-month
// sensitivity table so the user sees how far they are from each preset.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';

export async function renderEmergencyFund(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.emergency_fund.title">// EMERGENCY FUND</span></h1>
        <p class="muted small" data-i18n-html="view.emergency_fund.intro">
            Standard personal-finance rule: hold <strong>3 / 6 / 9 / 12 months</strong> of essential
            monthly expenses in liquid cash. Computes <strong>months covered now</strong>,
            <strong>target amount</strong>, <strong>gap</strong>, and <strong>months-to-target</strong>
            at your monthly contribution rate. Sensitivity table shows how far you are from
            each preset.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small" data-i18n="view.emergency_fund.field.monthly_expenses">Monthly expenses $</span>
                    <input type="number" id="ef-expenses" step="100" min="0" value="3000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.emergency_fund.field.current_fund">Current fund $</span>
                    <input type="number" id="ef-current" step="100" min="0" value="6000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.emergency_fund.field.target_months">Target months</span>
                    <input type="number" id="ef-target-months" step="1" min="1" max="60" value="6" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.emergency_fund.field.contribution">Monthly contribution $</span>
                    <input type="number" id="ef-contribution" step="50" min="0" value="500" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="ef-run" data-shortcut="r" data-i18n="view.emergency_fund.btn.run">⚡ Compute Readiness</button>
            <span class="muted small" id="ef-meta" style="margin-left:1rem"></span>
            <div id="ef-result"></div>
        </div>
    `;
    mount.querySelector('#ef-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#ef-result');
    const input = {
        monthly_expenses_usd: parseFloat(mount.querySelector('#ef-expenses').value) || 0,
        current_fund_usd: parseFloat(mount.querySelector('#ef-current').value) || 0,
        target_months: parseFloat(mount.querySelector('#ef-target-months').value) || 6,
        monthly_contribution_usd: parseFloat(mount.querySelector('#ef-contribution').value) || 0,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.emergency_fund.status.computing'))}</p>`;
    try {
        const r = await api('/emergency-fund/compute', { method: 'POST', body: JSON.stringify(input) });
        const statusCls = r.status === 'complete' ? 'pos' : r.status === 'on-track' ? '' : 'neg';
        const statusLbl = t(`view.emergency_fund.status.${r.status.replace('-', '_')}`) || r.status;
        const monthsCls = r.months_covered_now >= input.target_months
            ? 'pos'
            : r.months_covered_now >= input.target_months * 0.5 ? '' : 'neg';
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.emergency_fund.field.months_covered'))}</div>
                    <strong class="${monthsCls}" style="font-size:1.4em">${r.months_covered_now.toFixed(1)}</strong></div>
                <div><div class="muted small">${esc(t('view.emergency_fund.field.target_amount'))}</div>
                    <strong style="font-size:1.4em">$${(r.target_amount_usd / 1000).toFixed(1)}K</strong></div>
                <div><div class="muted small">${esc(t('view.emergency_fund.field.gap'))}</div>
                    <strong class="${r.gap_usd > 0 ? 'neg' : 'pos'}">$${(r.gap_usd / 1000).toFixed(1)}K</strong></div>
                <div><div class="muted small">${esc(t('view.emergency_fund.field.months_to_target'))}</div>
                    <strong>${r.months_to_target == null ? '∞' : r.months_to_target.toFixed(1)}</strong></div>
                <div><div class="muted small">${esc(t('view.emergency_fund.field.status'))}</div>
                    <strong class="${statusCls}" style="text-transform:uppercase">${esc(statusLbl)}</strong></div>
            </div>
            <h2 style="margin-top:1rem">${esc(t('view.emergency_fund.h2.scenarios'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.emergency_fund.th.preset">Preset</th>
                    <th data-i18n="view.emergency_fund.th.target_amount">Target $</th>
                    <th data-i18n="view.emergency_fund.th.gap">Gap $</th>
                    <th data-i18n="view.emergency_fund.th.months_to_target">Months to Target</th>
                </tr></thead>
                <tbody>${(r.scenarios || []).map(s => `
                    <tr>
                        <td><strong>${s.target_months}m</strong></td>
                        <td>$${(s.target_amount_usd / 1000).toFixed(1)}K</td>
                        <td class="${s.gap_usd > 0 ? 'neg' : 'pos'}">$${(s.gap_usd / 1000).toFixed(1)}K</td>
                        <td>${s.months_to_target == null ? '∞' : s.months_to_target.toFixed(1)}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
