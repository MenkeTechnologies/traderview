// Federal student loan payoff calculator — compares Standard 10-yr
// against IBR / PAYE / SAVE income-driven plans. Returns per-plan
// monthly payment, total paid, months until payoff or forgiveness,
// forgiven balance, and the lowest-total-paid plan.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import * as enh from '../calc_enhance.js';

export async function renderStudentLoanPayoff(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.student_loan_payoff.title">// STUDENT LOAN PAYOFF</span></h1>
        <p class="muted small" data-i18n-html="view.student_loan_payoff.intro">
            Side-by-side comparison of the four standard federal repayment plans:
            <strong>Standard 10-yr</strong> (amortizing 120mo), <strong>IBR</strong>
            (15% discretionary, forgive 25yr), <strong>PAYE</strong> (10% disc, forgive
            20yr), <strong>SAVE</strong> (10% disc with interest-subsidy halving unpaid
            interest, forgive 20yr). Discretionary income = AGI − (FPL × multiplier).
            FPL 2026 single = $15,750.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label><span class="muted small" data-i18n="view.student_loan_payoff.field.balance">Balance $</span>
                    <input type="number" id="sl-balance" step="1000" min="0" value="50000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.student_loan_payoff.field.apr">APR %</span>
                    <input type="number" id="sl-apr" step="0.25" min="0" max="30" value="6.5" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.student_loan_payoff.field.agi">AGI $/yr</span>
                    <input type="number" id="sl-agi" step="2500" min="0" value="60000" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.student_loan_payoff.field.household">Household size</span>
                    <input type="number" id="sl-household" step="1" min="1" max="20" value="1" style="width:100%"></label>
                <label><span class="muted small" data-i18n="view.student_loan_payoff.field.fpl_mult">FPL multiplier</span>
                    <input type="number" id="sl-fpl" step="0.25" min="1" max="4" value="1.5" style="width:100%"></label>
            </div>
            <button class="btn btn-sm primary" id="sl-run" data-shortcut="r" data-i18n="view.student_loan_payoff.btn.run">⚡ Compare Plans</button>
            <div id="sl-result"></div>
        </div>
    `;
    mount.querySelector('#sl-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#sl-result');
    const input = {
        balance_usd: parseFloat(mount.querySelector('#sl-balance').value) || 0,
        apr_pct: parseFloat(mount.querySelector('#sl-apr').value) || 0,
        agi_annual_usd: parseFloat(mount.querySelector('#sl-agi').value) || 0,
        household_size: parseInt(mount.querySelector('#sl-household').value, 10) || 1,
        fpl_multiplier: parseFloat(mount.querySelector('#sl-fpl').value) || 1.5,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.student_loan_payoff.status.computing'))}</p>`;
    try {
        const r = await api.request('/student-loan-payoff/compute', { method: 'POST', body: JSON.stringify(input) });
        // Total-paid-per-plan bar chart (compare repayment plans; lowest is best).
        const chart = enh.svgBarChart((r.plans || []).map(p => ({ label: t('view.student_loan_payoff.plan.' + p.plan) || p.plan, value: p.total_paid_usd })));
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.student_loan_payoff.field.poverty'))}</div>
                    <strong>$${r.poverty_line_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.student_loan_payoff.field.disc_income'))}</div>
                    <strong>$${r.discretionary_income_annual_usd.toFixed(0)}/yr</strong></div>
                <div><div class="muted small">${esc(t('view.student_loan_payoff.field.best'))}</div>
                    <strong class="pos" style="text-transform:uppercase">${esc(t('view.student_loan_payoff.plan.' + r.best_plan_total_paid) || r.best_plan_total_paid)}</strong></div>
            </div>
            ${chart}
            <div id="sl-tools" class="ce-toolbar"></div>
            <h2 style="margin-top:1rem">${esc(t('view.student_loan_payoff.h2.plans'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.student_loan_payoff.th.plan">Plan</th>
                    <th data-i18n="view.student_loan_payoff.th.monthly">Monthly (first)</th>
                    <th data-i18n="view.student_loan_payoff.th.months">Months</th>
                    <th data-i18n="view.student_loan_payoff.th.total_paid">Total paid</th>
                    <th data-i18n="view.student_loan_payoff.th.interest">Interest paid</th>
                    <th data-i18n="view.student_loan_payoff.th.forgiven">Forgiven</th>
                </tr></thead>
                <tbody>${(r.plans || []).map(p => {
                    const isBest = p.plan === r.best_plan_total_paid;
                    return `
                    <tr style="${isBest ? 'background:rgba(57,255,20,0.08)' : ''}">
                        <td><strong>${esc(t('view.student_loan_payoff.plan.' + p.plan) || p.plan)}</strong></td>
                        <td>$${p.monthly_payment_first_usd.toFixed(2)}/mo</td>
                        <td>${p.months_to_payoff_or_forgive}</td>
                        <td><strong>$${p.total_paid_usd.toFixed(0)}</strong></td>
                        <td class="neg">$${p.interest_paid_usd.toFixed(0)}</td>
                        <td class="${p.forgiven_balance_usd > 0 ? 'pos' : ''}">${p.forgiven_balance_usd > 0 ? '$' + p.forgiven_balance_usd.toFixed(0) : '—'}</td>
                    </tr>
                `;
                }).join('')}</tbody>
            </table>
        `;
        // Per-plan export (Copy / CSV). No permalink — id-based inputs.
        enh.mountToolbar(mount.querySelector('#sl-tools'), {
            viewId: 'student-loan-payoff',
            link: false,
            filename: 'student-loan-payoff.csv',
            getRows: () => [['plan', 'monthly_first_usd', 'months', 'total_paid_usd', 'interest_paid_usd', 'forgiven_usd'],
                ...(r.plans || []).map(p => [p.plan, p.monthly_payment_first_usd, p.months_to_payoff_or_forgive, p.total_paid_usd, p.interest_paid_usd, p.forgiven_balance_usd])],
        });
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
