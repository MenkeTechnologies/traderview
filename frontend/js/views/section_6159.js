// IRC § 6159 — Installment Agreements (IA).
// Pay off tax debt in monthly payments. Multiple types based on amount + ability:
// Guaranteed (< $10k, 36 mo, automatic), Streamlined (< $50k, 72 mo, simplified),
// Routine (< $50k+, may require Form 433-F), Non-streamlined (> $50k, full financial disclosure),
// Partial Pay (PPIA — runs out CSED with reduced payment).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const GUARANTEED_LIMIT = 10_000;
const STREAMLINED_LIMIT = 50_000;
const STREAMLINED_TERM_MONTHS = 72;
const ROUTINE_TERM_MONTHS = 72;
const CSED_DEFAULT_DAYS = 10 * 365;

let state = {
    total_debt: 0,
    months_until_csed: 120,
    monthly_disposable_income: 0,
    is_business: false,
    direct_debit: false,
    payroll_deduction: false,
    is_first_iA: true,
    has_filed_all_returns: true,
    current_on_estimated_payments: true,
    setup_fee_paid: 0,
};

const SETUP_FEES = {
    online_direct_debit: 22,
    online_other: 69,
    phone_mail_direct_debit: 107,
    phone_mail_other: 178,
    low_income_waived: 0,
};

export async function renderSection6159(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6159.h1.title">// § 6159 INSTALLMENT AGREEMENT</span></h1>
        <p class="muted small" data-i18n="view.s6159.hint.intro">
            Pay debt monthly. <strong>Guaranteed IA:</strong> &lt; $10k, 36 mo, automatic.
            <strong>Streamlined:</strong> &lt; $50k, 72 mo. <strong>Routine:</strong> &lt; $250k
            (post-2020). <strong>Non-streamlined:</strong> &gt; $50k requires Form 433-F.
            <strong>Partial Pay IA (PPIA):</strong> reduced payment runs out CSED; debt forgiven
            at CSED. Setup fee: $22-$178 (waived low-income). Failure to pay = default.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6159.h2.inputs">Inputs</h2>
            <form id="s6159-form" class="inline-form">
                <label><span data-i18n="view.s6159.label.debt">Total debt ($)</span>
                    <input type="number" step="0.01" name="total_debt" value="${state.total_debt}"></label>
                <label><span data-i18n="view.s6159.label.csed_months">Months until CSED</span>
                    <input type="number" step="1" name="months_until_csed" value="${state.months_until_csed}"></label>
                <label><span data-i18n="view.s6159.label.income">Monthly disposable income ($)</span>
                    <input type="number" step="0.01" name="monthly_disposable_income" value="${state.monthly_disposable_income}"></label>
                <label><span data-i18n="view.s6159.label.business">Business taxpayer?</span>
                    <input type="checkbox" name="is_business" ${state.is_business ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6159.label.direct_debit">Direct debit?</span>
                    <input type="checkbox" name="direct_debit" ${state.direct_debit ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6159.label.payroll">Payroll deduction?</span>
                    <input type="checkbox" name="payroll_deduction" ${state.payroll_deduction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6159.label.first_ia">First IA in 5 years?</span>
                    <input type="checkbox" name="is_first_iA" ${state.is_first_iA ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6159.label.filed">All returns filed?</span>
                    <input type="checkbox" name="has_filed_all_returns" ${state.has_filed_all_returns ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6159.label.estimated">Current on estimated payments?</span>
                    <input type="checkbox" name="current_on_estimated_payments" ${state.current_on_estimated_payments ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6159.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6159-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6159.h2.types">IA types</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s6159.th.type">Type</th>
                    <th data-i18n="view.s6159.th.limit">Debt limit</th>
                    <th data-i18n="view.s6159.th.term">Term</th>
                    <th data-i18n="view.s6159.th.financials">Financials required</th>
                </tr></thead>
                <tbody>
                    <tr><td>Guaranteed</td><td>&lt; $10k</td><td>3 yrs</td><td>NO</td></tr>
                    <tr><td>Streamlined</td><td>&lt; $50k</td><td>6 yrs</td><td>NO</td></tr>
                    <tr><td>Routine (Express)</td><td>&lt; $250k post-2020</td><td>≤ CSED</td><td>Sometimes 433-F</td></tr>
                    <tr><td>Non-streamlined</td><td>&gt; $50k</td><td>≤ CSED</td><td>Form 433-F + supporting docs</td></tr>
                    <tr><td>Partial Pay (PPIA)</td><td>Unable to pay full</td><td>Runs out CSED</td><td>Full 433-F + reviews every 2 yrs</td></tr>
                    <tr><td>Business In-Business Trust Fund Express</td><td>&lt; $25k payroll trust fund</td><td>24 mo</td><td>NO</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6159.h2.fees">Setup fees (2024)</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s6159.th.application">Application method</th>
                    <th data-i18n="view.s6159.th.fee">Fee</th>
                </tr></thead>
                <tbody>
                    <tr><td>Online + Direct Debit</td><td>$22</td></tr>
                    <tr><td>Online (no DD)</td><td>$69</td></tr>
                    <tr><td>Phone / Mail + Direct Debit</td><td>$107</td></tr>
                    <tr><td>Phone / Mail (no DD)</td><td>$178</td></tr>
                    <tr><td>Low-income waiver</td><td>$0 (or refunded for some)</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6159.h2.defaults">Default + termination triggers</h2>
            <ul class="muted small">
                <li data-i18n="view.s6159.def.missed">Missed monthly payment</li>
                <li data-i18n="view.s6159.def.new_balance">Owed new tax balance + didn't pay</li>
                <li data-i18n="view.s6159.def.didnt_file">Didn't file required return</li>
                <li data-i18n="view.s6159.def.didnt_provide">Failed to provide requested financial info</li>
                <li data-i18n="view.s6159.def.misleading">Misleading financial info on application</li>
                <li data-i18n="view.s6159.def.bankruptcy">Bankruptcy filed</li>
                <li data-i18n="view.s6159.def.estimated">Falls behind on estimated tax payments</li>
                <li data-i18n="view.s6159.def.cure">30-day cure period after notice</li>
                <li data-i18n="view.s6159.def.csed_runs">IA running CSED restarts at termination</li>
            </ul>
        </div>
    `;
    document.getElementById('s6159-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.total_debt = Number(fd.get('total_debt')) || 0;
        state.months_until_csed = Number(fd.get('months_until_csed')) || 120;
        state.monthly_disposable_income = Number(fd.get('monthly_disposable_income')) || 0;
        state.is_business = !!fd.get('is_business');
        state.direct_debit = !!fd.get('direct_debit');
        state.payroll_deduction = !!fd.get('payroll_deduction');
        state.is_first_iA = !!fd.get('is_first_iA');
        state.has_filed_all_returns = !!fd.get('has_filed_all_returns');
        state.current_on_estimated_payments = !!fd.get('current_on_estimated_payments');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6159-output');
    if (!el) return;
    let recommendedType, term;
    if (state.total_debt < GUARANTEED_LIMIT && state.is_first_iA) {
        recommendedType = 'view.s6159.type.guaranteed';
        term = 36;
    } else if (state.total_debt < STREAMLINED_LIMIT) {
        recommendedType = 'view.s6159.type.streamlined';
        term = STREAMLINED_TERM_MONTHS;
    } else if (state.total_debt < 250_000) {
        recommendedType = 'view.s6159.type.routine';
        term = Math.min(state.months_until_csed, ROUTINE_TERM_MONTHS);
    } else {
        const fullPaymentTerm = state.monthly_disposable_income > 0
            ? state.total_debt / state.monthly_disposable_income : 999;
        if (fullPaymentTerm <= state.months_until_csed) {
            recommendedType = 'view.s6159.type.non_streamlined';
            term = Math.min(fullPaymentTerm, state.months_until_csed);
        } else {
            recommendedType = 'view.s6159.type.ppia';
            term = state.months_until_csed;
        }
    }
    const requiredMonthly = state.total_debt / term;
    const monthlyOk = state.monthly_disposable_income >= requiredMonthly;
    const setupFee = state.direct_debit ? SETUP_FEES.online_direct_debit : SETUP_FEES.online_other;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6159.h2.result">IA recommendation</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.s6159.card.recommended">Recommended type</div>
                    <div class="value">${esc(t(recommendedType))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6159.card.term">Term (months)</div>
                    <div class="value">${Math.round(term)}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6159.card.required">Required monthly</div>
                    <div class="value">$${requiredMonthly.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${monthlyOk ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s6159.card.affordable">Affordable?</div>
                    <div class="value">${monthlyOk ? esc(t('view.s6159.status.yes')) : esc(t('view.s6159.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6159.card.setup_fee">Setup fee</div>
                    <div class="value">$${setupFee.toLocaleString()}</div>
                </div>
            </div>
        </div>
    `;
}
