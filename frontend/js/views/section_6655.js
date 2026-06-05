// IRC § 6655 — Corporate Estimated Tax Payments.
// Corp must pay estimated tax in 4 installments (4/15, 6/15, 9/15, 12/15) totaling 100% of expected tax.
// Underpayment penalty: federal short-term rate + 3% on shortfall (Form 2220).
// Safe harbors: (1) 100% of prior-year tax (large corp: 100% of current year), (2) annualized income method.
// Large corp: avg taxable income ≥ $1M in any of past 3 yrs — only current-year safe harbor.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    estimated_current_year_tax: 0,
    prior_year_tax: 0,
    installment_paid_q1: 0,
    installment_paid_q2: 0,
    installment_paid_q3: 0,
    installment_paid_q4: 0,
    is_large_corp: false,
    avg_taxable_income_3yr: 0,
    seasonal_business: false,
    elect_annualized_income: false,
    elect_adjusted_seasonal: false,
    federal_short_term_rate: 5.0,
    days_underpayment_avg: 0,
    new_corp_first_year: false,
    nol_carryforward_used: 0,
};

export async function renderSection6655(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6655.h1.title">// § 6655 CORP ESTIMATED TAX</span></h1>
        <p class="muted small" data-i18n="view.s6655.hint.intro">
            Corp must pay <strong>estimated tax in 4 installments</strong> (4/15, 6/15, 9/15, 12/15) totaling
            100% of expected tax. <strong>Underpayment penalty:</strong> federal short-term rate + 3% on
            shortfall (Form 2220). <strong>Safe harbors:</strong> (1) 100% of <strong>prior-year tax</strong>
            (large corp: 100% of CURRENT year), (2) <strong>annualized income method</strong>, (3) <strong>adjusted
            seasonal installment</strong>. <strong>Large corp:</strong> avg taxable income ≥ $1M in any of past
            3 yrs — ONLY current-year safe harbor available. <strong>Form 1120-W</strong> worksheet (not filed).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6655.h2.inputs">Inputs</h2>
            <form id="s6655-form" class="inline-form">
                <label><span data-i18n="view.s6655.label.current">Estimated current year tax ($)</span>
                    <input type="number" step="0.01" name="estimated_current_year_tax" value="${state.estimated_current_year_tax}"></label>
                <label><span data-i18n="view.s6655.label.prior">Prior year tax ($)</span>
                    <input type="number" step="0.01" name="prior_year_tax" value="${state.prior_year_tax}"></label>
                <label><span data-i18n="view.s6655.label.q1">Q1 installment paid ($)</span>
                    <input type="number" step="0.01" name="installment_paid_q1" value="${state.installment_paid_q1}"></label>
                <label><span data-i18n="view.s6655.label.q2">Q2 installment paid ($)</span>
                    <input type="number" step="0.01" name="installment_paid_q2" value="${state.installment_paid_q2}"></label>
                <label><span data-i18n="view.s6655.label.q3">Q3 installment paid ($)</span>
                    <input type="number" step="0.01" name="installment_paid_q3" value="${state.installment_paid_q3}"></label>
                <label><span data-i18n="view.s6655.label.q4">Q4 installment paid ($)</span>
                    <input type="number" step="0.01" name="installment_paid_q4" value="${state.installment_paid_q4}"></label>
                <label><span data-i18n="view.s6655.label.large">Large corp (≥ $1M TI)?</span>
                    <input type="checkbox" name="is_large_corp" ${state.is_large_corp ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6655.label.avg_ti">Avg taxable income 3-yr ($)</span>
                    <input type="number" step="0.01" name="avg_taxable_income_3yr" value="${state.avg_taxable_income_3yr}"></label>
                <label><span data-i18n="view.s6655.label.seasonal">Seasonal business?</span>
                    <input type="checkbox" name="seasonal_business" ${state.seasonal_business ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6655.label.annualized">Elect annualized income method?</span>
                    <input type="checkbox" name="elect_annualized_income" ${state.elect_annualized_income ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6655.label.seasonal_method">Elect adjusted seasonal?</span>
                    <input type="checkbox" name="elect_adjusted_seasonal" ${state.elect_adjusted_seasonal ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6655.label.rate">Federal short-term rate %</span>
                    <input type="number" step="0.1" name="federal_short_term_rate" value="${state.federal_short_term_rate}"></label>
                <label><span data-i18n="view.s6655.label.days">Avg days underpayment</span>
                    <input type="number" step="1" name="days_underpayment_avg" value="${state.days_underpayment_avg}"></label>
                <label><span data-i18n="view.s6655.label.new">New corp first year?</span>
                    <input type="checkbox" name="new_corp_first_year" ${state.new_corp_first_year ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6655.label.nol">NOL carryforward used ($)</span>
                    <input type="number" step="0.01" name="nol_carryforward_used" value="${state.nol_carryforward_used}"></label>
                <button class="primary" type="submit" data-i18n="view.s6655.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6655-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6655.h2.schedule">Installment schedule</h2>
            <ul class="muted small">
                <li data-i18n="view.s6655.sched.q1">Q1: April 15 — 25% of required annual payment</li>
                <li data-i18n="view.s6655.sched.q2">Q2: June 15 — 50% cumulative (or 25%)</li>
                <li data-i18n="view.s6655.sched.q3">Q3: September 15 — 75% cumulative (or 25%)</li>
                <li data-i18n="view.s6655.sched.q4">Q4: December 15 — 100% cumulative (or 25%)</li>
                <li data-i18n="view.s6655.sched.equal">Equal installments OR annualized / seasonal methods</li>
                <li data-i18n="view.s6655.sched.weekend">If due date is weekend / holiday: next business day</li>
                <li data-i18n="view.s6655.sched.consolidated">Consolidated group: parent + all subs aggregate</li>
                <li data-i18n="view.s6655.sched.short_period">Short-period corp: prorated installments</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6655.h2.safe_harbors">Safe harbors</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s6655.th.harbor">Safe harbor</th>
                    <th data-i18n="view.s6655.th.amount">Amount</th>
                    <th data-i18n="view.s6655.th.applies">Applies</th>
                </tr></thead>
                <tbody>
                    <tr><td>Current-year tax</td><td>100% of current tax</td><td>Always available</td></tr>
                    <tr><td>Prior-year tax</td><td>100% of prior tax</td><td>NON-large corp + prior yr existed</td></tr>
                    <tr><td>Annualized income method</td><td>Based on YTD income annualized</td><td>Variable income corp</td></tr>
                    <tr><td>Adjusted seasonal installment</td><td>Based on seasonal pattern</td><td>Seasonal business (70%+ 6-mo concentration)</td></tr>
                    <tr><td>Large corp (≥ $1M TI)</td><td>100% of CURRENT only (after Q1)</td><td>Q2-Q4 installments</td></tr>
                    <tr><td>Small tax owed</td><td>&lt; $500 total tax</td><td>De minimis exception</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6655.h2.annualized">Annualized income method (§ 6655(e))</h2>
            <ul class="muted small">
                <li data-i18n="view.s6655.ann.purpose">Purpose: avoid penalty when income SPIKED in later periods</li>
                <li data-i18n="view.s6655.ann.q1">Q1: annualize Q1 income × 4 → tax × 25%</li>
                <li data-i18n="view.s6655.ann.q2">Q2: annualize Jan-May × 12/5 → tax × 50%, minus Q1</li>
                <li data-i18n="view.s6655.ann.q3">Q3: annualize Jan-Aug × 12/8 → tax × 75%, minus Q1+Q2</li>
                <li data-i18n="view.s6655.ann.q4">Q4: annualize Jan-Nov × 12/11 → tax × 100%, minus Q1+Q2+Q3</li>
                <li data-i18n="view.s6655.ann.elections">3 different election periods available (months may vary)</li>
                <li data-i18n="view.s6655.ann.book_to_tax">Book-to-tax adjustments at each period</li>
                <li data-i18n="view.s6655.ann.recalculation">Each quarter rechecked — late-year underpayment may trigger earlier penalty</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6655.h2.penalty">Penalty computation (Form 2220)</h2>
            <ul class="muted small">
                <li data-i18n="view.s6655.pen.rate">Rate: federal short-term rate + 3% (compounded daily)</li>
                <li data-i18n="view.s6655.pen.application">Applied to each installment SEPARATELY</li>
                <li data-i18n="view.s6655.pen.from">Period: due date of installment to earlier of payment OR Form 1120 filing</li>
                <li data-i18n="view.s6655.pen.formula">Underpayment × rate × (days / 365)</li>
                <li data-i18n="view.s6655.pen.deductible">Penalty NOT deductible</li>
                <li data-i18n="view.s6655.pen.add_to_tax">Reported on Form 2220 + Form 1120 Line 33</li>
                <li data-i18n="view.s6655.pen.exception_first">First-year corp + small operations: no penalty under § 6655(g)</li>
                <li data-i18n="view.s6655.pen.reasonable_cause">Reasonable cause exception: rarely granted (Boyle case strict)</li>
            </ul>
        </div>
    `;
    document.getElementById('s6655-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.estimated_current_year_tax = Number(fd.get('estimated_current_year_tax')) || 0;
        state.prior_year_tax = Number(fd.get('prior_year_tax')) || 0;
        state.installment_paid_q1 = Number(fd.get('installment_paid_q1')) || 0;
        state.installment_paid_q2 = Number(fd.get('installment_paid_q2')) || 0;
        state.installment_paid_q3 = Number(fd.get('installment_paid_q3')) || 0;
        state.installment_paid_q4 = Number(fd.get('installment_paid_q4')) || 0;
        state.is_large_corp = !!fd.get('is_large_corp');
        state.avg_taxable_income_3yr = Number(fd.get('avg_taxable_income_3yr')) || 0;
        state.seasonal_business = !!fd.get('seasonal_business');
        state.elect_annualized_income = !!fd.get('elect_annualized_income');
        state.elect_adjusted_seasonal = !!fd.get('elect_adjusted_seasonal');
        state.federal_short_term_rate = Number(fd.get('federal_short_term_rate')) || 0;
        state.days_underpayment_avg = Number(fd.get('days_underpayment_avg')) || 0;
        state.new_corp_first_year = !!fd.get('new_corp_first_year');
        state.nol_carryforward_used = Number(fd.get('nol_carryforward_used')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6655-output');
    if (!el) return;
    const isLarge = state.is_large_corp || state.avg_taxable_income_3yr >= 1_000_000;
    const targetAmount = isLarge ? state.estimated_current_year_tax : Math.min(state.estimated_current_year_tax, state.prior_year_tax || state.estimated_current_year_tax);
    const requiredPerInstallment = targetAmount / 4;
    const paidTotal = state.installment_paid_q1 + state.installment_paid_q2 + state.installment_paid_q3 + state.installment_paid_q4;
    const underpaymentTotal = Math.max(0, targetAmount - paidTotal);
    const annualRate = (state.federal_short_term_rate + 3) / 100;
    const penalty = underpaymentTotal * annualRate * (state.days_underpayment_avg / 365);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6655.h2.result">§ 6655 computation</h2>
            <div class="cards">
                <div class="card ${isLarge ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s6655.card.large">Large corp?</div>
                    <div class="value">${isLarge ? esc(t('view.s6655.status.yes')) : esc(t('view.s6655.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6655.card.target">Target annual payment</div>
                    <div class="value">$${targetAmount.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6655.card.per_installment">Per installment</div>
                    <div class="value">$${requiredPerInstallment.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6655.card.paid">Total paid</div>
                    <div class="value">$${paidTotal.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6655.card.under">Total underpayment</div>
                    <div class="value">$${underpaymentTotal.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6655.card.rate">Penalty rate</div>
                    <div class="value">${(annualRate * 100).toFixed(1)}%</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6655.card.penalty">Estimated penalty</div>
                    <div class="value">$${penalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.elect_annualized_income ? `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s6655.annualized_note">
                    Annualized income method elected: payment requirements based on YTD income annualized
                    quarterly, not flat 25%. Particularly valuable for cyclical businesses w/ Q4 spikes.
                    Form 2220 Part III computation. Reduces Q1-Q3 underpayment but Q4 cumulative test
                    catches up if not paid sufficiently.
                </p>
            ` : ''}
        </div>
    `;
}
