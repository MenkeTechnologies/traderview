// IRC § 72(p) — 401(k) / 403(b) Plan Loans.
// Max: lesser of $50,000 OR 50% of vested balance. 5-year max term (longer for principal residence).
// Quarterly payments minimum. Loan default = deemed distribution + 10% penalty if < 59½.
// SECURE 2.0: Roth match source for loans + relief on plan-to-plan rollover loans + COVID-style flexibility.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const MAX_LOAN_AMOUNT = 50_000;
const VESTED_BALANCE_PCT = 0.50;
const STANDARD_TERM_YEARS = 5;
const HOME_TERM_YEARS = 30;

let state = {
    vested_balance: 0,
    loan_amount: 0,
    is_principal_residence: false,
    loan_term_years: 5,
    interest_rate: 0.08,
    employer_outstanding_loans: 0,
    your_age: 45,
    fed_marginal_rate: 0.32,
    expected_market_return: 0.08,
};

export async function renderSection72p(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s72p.h1.title">// § 72(p) 401(k) PLAN LOAN</span></h1>
        <p class="muted small" data-i18n="view.s72p.hint.intro">
            Max: <strong>lesser of $50,000 OR 50% vested balance</strong>. <strong>5-year max term</strong>
            (up to 30 yrs for principal residence). Quarterly payments minimum. <strong>Loan default
            = deemed distribution</strong> + 10% penalty if &lt; 59½. SECURE 2.0: Roth source for
            loans + plan-to-plan rollover loans preserved. <strong>Hidden cost:</strong> opportunity
            cost (money out of market) + double taxation of interest (you repay with after-tax dollars,
            then pay tax again on withdrawal).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s72p.h2.inputs">Inputs</h2>
            <form id="s72p-form" class="inline-form">
                <label><span data-i18n="view.s72p.label.balance">Vested balance ($)</span>
                    <input type="number" step="0.01" name="vested_balance" value="${state.vested_balance}"></label>
                <label><span data-i18n="view.s72p.label.amount">Desired loan ($)</span>
                    <input type="number" step="0.01" name="loan_amount" value="${state.loan_amount}"></label>
                <label><span data-i18n="view.s72p.label.residence">Principal residence?</span>
                    <input type="checkbox" name="is_principal_residence" ${state.is_principal_residence ? 'checked' : ''}></label>
                <label><span data-i18n="view.s72p.label.term">Loan term (years)</span>
                    <input type="number" step="1" name="loan_term_years" value="${state.loan_term_years}"></label>
                <label><span data-i18n="view.s72p.label.rate">Interest rate</span>
                    <input type="number" step="0.001" name="interest_rate" value="${state.interest_rate}"></label>
                <label><span data-i18n="view.s72p.label.outstanding">Outstanding employer loans ($)</span>
                    <input type="number" step="0.01" name="employer_outstanding_loans" value="${state.employer_outstanding_loans}"></label>
                <label><span data-i18n="view.s72p.label.age">Your age</span>
                    <input type="number" step="1" name="your_age" value="${state.your_age}"></label>
                <label><span data-i18n="view.s72p.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" name="fed_marginal_rate" value="${state.fed_marginal_rate}"></label>
                <label><span data-i18n="view.s72p.label.market">Expected market return %</span>
                    <input type="number" step="0.01" name="expected_market_return" value="${state.expected_market_return}"></label>
                <button class="primary" type="submit" data-i18n="view.s72p.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s72p-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s72p.h2.benefits">Benefits</h2>
            <ul class="muted small">
                <li data-i18n="view.s72p.ben.no_credit">No credit check (loan from your own balance)</li>
                <li data-i18n="view.s72p.ben.low_rate">Interest typically lower than credit cards / personal loans</li>
                <li data-i18n="view.s72p.ben.interest_to_you">Interest goes BACK to your account (paying yourself)</li>
                <li data-i18n="view.s72p.ben.no_tax">Not a taxable distribution if repaid on time</li>
                <li data-i18n="view.s72p.ben.no_credit_report">Doesn't appear on credit report</li>
                <li data-i18n="view.s72p.ben.flexible">Can be re-paid early without penalty</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s72p.h2.risks">Risks + costs</h2>
            <ul class="muted small">
                <li data-i18n="view.s72p.risk.opportunity">Opportunity cost: money OUT of market while loan outstanding</li>
                <li data-i18n="view.s72p.risk.double_tax">Double-taxation: interest paid with after-tax dollars + taxed again on withdrawal</li>
                <li data-i18n="view.s72p.risk.job_loss">Job loss / separation triggers loan acceleration (60-90 days to repay)</li>
                <li data-i18n="view.s72p.risk.default">Default = deemed distribution + tax + 10% penalty</li>
                <li data-i18n="view.s72p.risk.contributions">Suspend new contributions during loan period (some plans)</li>
                <li data-i18n="view.s72p.risk.market_timing">Borrowing in down market locks in losses</li>
                <li data-i18n="view.s72p.risk.balloon">Balloon payment at term-end if quarterly amounts insufficient</li>
            </ul>
        </div>
    `;
    document.getElementById('s72p-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.vested_balance = Number(fd.get('vested_balance')) || 0;
        state.loan_amount = Number(fd.get('loan_amount')) || 0;
        state.is_principal_residence = !!fd.get('is_principal_residence');
        state.loan_term_years = Number(fd.get('loan_term_years')) || 5;
        state.interest_rate = Number(fd.get('interest_rate')) || 0.08;
        state.employer_outstanding_loans = Number(fd.get('employer_outstanding_loans')) || 0;
        state.your_age = Number(fd.get('your_age')) || 45;
        state.fed_marginal_rate = Number(fd.get('fed_marginal_rate')) || 0.32;
        state.expected_market_return = Number(fd.get('expected_market_return')) || 0.08;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s72p-output');
    if (!el) return;
    const maxByCap = Math.min(MAX_LOAN_AMOUNT, state.vested_balance * VESTED_BALANCE_PCT);
    const maxAfterOutstanding = Math.max(0, maxByCap - state.employer_outstanding_loans);
    const allowedLoan = Math.min(state.loan_amount, maxAfterOutstanding);
    const maxTerm = state.is_principal_residence ? HOME_TERM_YEARS : STANDARD_TERM_YEARS;
    const overTerm = state.loan_term_years > maxTerm;
    const monthlyPayment = state.loan_term_years > 0
        ? (allowedLoan * state.interest_rate / 12) / (1 - Math.pow(1 + state.interest_rate / 12, -state.loan_term_years * 12))
        : 0;
    const totalRepaid = monthlyPayment * state.loan_term_years * 12;
    const interestPaid = totalRepaid - allowedLoan;
    const opportunityCost = allowedLoan * (Math.pow(1 + state.expected_market_return, state.loan_term_years) - 1) - interestPaid;
    const defaultPenalty = state.your_age < 59.5 ? allowedLoan * 0.10 : 0;
    const defaultFedTax = allowedLoan * state.fed_marginal_rate;
    const defaultCost = defaultPenalty + defaultFedTax;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s72p.h2.result">Loan outcome</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s72p.card.max_50">Max loan (50% / $50k)</div>
                    <div class="value">$${maxByCap.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s72p.card.allowed">Allowed loan</div>
                    <div class="value">$${allowedLoan.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${overTerm ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s72p.card.term">Max term</div>
                    <div class="value">${maxTerm} ${esc(t('view.s72p.units.years'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s72p.card.monthly">Monthly payment</div>
                    <div class="value">$${monthlyPayment.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s72p.card.total_repaid">Total repaid</div>
                    <div class="value">$${totalRepaid.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s72p.card.interest">Interest paid to self</div>
                    <div class="value">$${interestPaid.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s72p.card.opportunity">Opportunity cost</div>
                    <div class="value">$${opportunityCost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s72p.card.default_cost">If default cost</div>
                    <div class="value">$${defaultCost.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
