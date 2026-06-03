// IRC § 7872 — Below-market loan rules (intra-family / shareholder / employee).
// Lend below the Applicable Federal Rate (AFR) → IRS imputes interest income.
// AFR brackets: short-term (≤3 yr), mid-term (>3 ≤9 yr), long-term (>9 yr).
// Demand loan: blended-rate revisited monthly. Gift loan: $10,000 / $100,000 safe harbors.
// Common use: child first-home loan, family business, sale to IDGT.

import { currentViewToken, viewIsCurrent } from '../app.js';

const SAFE_HARBOR_GIFT = 10_000;
const SAFE_HARBOR_NET_INVESTMENT_INCOME = 100_000;

let state = {
    loan_type: 'term',
    term_years: 5,
    loan_amount: 0,
    interest_rate_charged: 0,
    afr_short_term: 0.0470,
    afr_mid_term: 0.0460,
    afr_long_term: 0.0470,
    borrower_net_investment_income: 0,
    is_corp_to_shareholder: false,
    lender_marginal_rate: 0.37,
    borrower_marginal_rate: 0.32,
};

export async function renderSection7872(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s7872.h1.title">// § 7872 INTRA-FAMILY LOAN / AFR</span></h1>
        <p class="muted small" data-i18n="view.s7872.hint.intro">
            Loan to family below the <strong>Applicable Federal Rate (AFR)</strong> → IRS
            imputes interest income to lender + may impute a gift. AFR is the IRS-published
            minimum monthly: <strong>short-term (≤3 yr), mid-term (3-9 yr), long-term (>9 yr)</strong>.
            <strong>$10,000 gift loan safe harbor</strong> entirely exempt. <strong>$100,000 limit:</strong>
            imputed income capped at borrower's net investment income.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s7872.h2.inputs">Inputs</h2>
            <form id="s7872-form" class="inline-form">
                <label><span data-i18n="view.s7872.label.loan_type">Loan type</span>
                    <select name="loan_type">
                        <option value="term" ${state.loan_type === 'term' ? 'selected' : ''}>Term loan (fixed)</option>
                        <option value="demand" ${state.loan_type === 'demand' ? 'selected' : ''}>Demand loan (callable)</option>
                        <option value="gift" ${state.loan_type === 'gift' ? 'selected' : ''}>Gift loan</option>
                        <option value="employee" ${state.loan_type === 'employee' ? 'selected' : ''}>Compensation loan</option>
                    </select>
                </label>
                <label><span data-i18n="view.s7872.label.term_years">Term (years)</span>
                    <input type="number" step="1" name="term_years" value="${state.term_years}"></label>
                <label><span data-i18n="view.s7872.label.loan_amount">Loan amount ($)</span>
                    <input type="number" step="1000" name="loan_amount" value="${state.loan_amount}"></label>
                <label><span data-i18n="view.s7872.label.rate_charged">Interest rate charged</span>
                    <input type="number" step="0.0001" name="interest_rate_charged" value="${state.interest_rate_charged}"></label>
                <label><span data-i18n="view.s7872.label.afr_st">AFR short-term (current month)</span>
                    <input type="number" step="0.0001" name="afr_short_term" value="${state.afr_short_term}"></label>
                <label><span data-i18n="view.s7872.label.afr_mt">AFR mid-term (current month)</span>
                    <input type="number" step="0.0001" name="afr_mid_term" value="${state.afr_mid_term}"></label>
                <label><span data-i18n="view.s7872.label.afr_lt">AFR long-term (current month)</span>
                    <input type="number" step="0.0001" name="afr_long_term" value="${state.afr_long_term}"></label>
                <label><span data-i18n="view.s7872.label.nii">Borrower net investment income ($)</span>
                    <input type="number" step="100" name="borrower_net_investment_income" value="${state.borrower_net_investment_income}"></label>
                <label><span data-i18n="view.s7872.label.is_corp">Corp-to-shareholder?</span>
                    <input type="checkbox" name="is_corp_to_shareholder" ${state.is_corp_to_shareholder ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7872.label.lender_rate">Lender marginal %</span>
                    <input type="number" step="0.01" name="lender_marginal_rate" value="${state.lender_marginal_rate}"></label>
                <label><span data-i18n="view.s7872.label.borrower_rate">Borrower marginal %</span>
                    <input type="number" step="0.01" name="borrower_marginal_rate" value="${state.borrower_marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s7872.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s7872-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7872.h2.afr_sources">AFR data sources</h2>
            <ul class="muted small">
                <li data-i18n="view.s7872.afr.irs">IRS publishes monthly via Rev. Rul. (typically third Friday of prior month)</li>
                <li data-i18n="view.s7872.afr.bracket">Short = ≤ 3 yr, mid = > 3 ≤ 9 yr, long = > 9 yr</li>
                <li data-i18n="view.s7872.afr.month_lock">Use rate from month loan is made (or lower of two prior months for term loan)</li>
                <li data-i18n="view.s7872.afr.fred">Bloomberg, FRED, IRS website all carry monthly AFR tables</li>
            </ul>
        </div>
    `;
    document.getElementById('s7872-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.loan_type = fd.get('loan_type');
        state.term_years = Number(fd.get('term_years')) || 0;
        state.loan_amount = Number(fd.get('loan_amount')) || 0;
        state.interest_rate_charged = Number(fd.get('interest_rate_charged')) || 0;
        state.afr_short_term = Number(fd.get('afr_short_term')) || 0;
        state.afr_mid_term = Number(fd.get('afr_mid_term')) || 0;
        state.afr_long_term = Number(fd.get('afr_long_term')) || 0;
        state.borrower_net_investment_income = Number(fd.get('borrower_net_investment_income')) || 0;
        state.is_corp_to_shareholder = !!fd.get('is_corp_to_shareholder');
        state.lender_marginal_rate = Number(fd.get('lender_marginal_rate')) || 0.37;
        state.borrower_marginal_rate = Number(fd.get('borrower_marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function applicableAfr() {
    if (state.term_years <= 3) return state.afr_short_term;
    if (state.term_years <= 9) return state.afr_mid_term;
    return state.afr_long_term;
}

function renderOutput() {
    const el = document.getElementById('s7872-output');
    if (!el) return;
    const afr = applicableAfr();
    const rateGap = Math.max(0, afr - state.interest_rate_charged);
    const annualImputedRaw = state.loan_amount * rateGap;
    let imputedIncome = annualImputedRaw;
    let safeHarbor10k = false, safeHarbor100k = false;
    if (state.loan_type === 'gift') {
        if (state.loan_amount <= SAFE_HARBOR_GIFT) {
            imputedIncome = 0;
            safeHarbor10k = true;
        } else if (state.loan_amount <= SAFE_HARBOR_NET_INVESTMENT_INCOME) {
            imputedIncome = Math.min(annualImputedRaw, state.borrower_net_investment_income);
            safeHarbor100k = true;
        }
    }
    const giftImputed = state.loan_type === 'gift' ? imputedIncome : 0;
    const lenderTaxOwed = imputedIncome * state.lender_marginal_rate;
    const borrowerDeduction = state.loan_type === 'employee' || state.loan_type === 'gift'
        ? 0
        : (state.is_corp_to_shareholder ? 0 : imputedIncome);
    const borrowerSavings = borrowerDeduction * state.borrower_marginal_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s7872.h2.result">Imputed interest + gift</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s7872.card.afr_applicable">Applicable AFR</div>
                    <div class="value">${(afr * 100).toFixed(2)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s7872.card.gap">Rate gap</div>
                    <div class="value">${(rateGap * 100).toFixed(2)}%</div>
                </div>
                <div class="card ${imputedIncome > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s7872.card.imputed">Annual imputed interest</div>
                    <div class="value">$${imputedIncome.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s7872.card.lender_owed">Lender tax owed</div>
                    <div class="value">$${lenderTaxOwed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s7872.card.borrower_savings">Borrower deduction savings</div>
                    <div class="value">$${borrowerSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${giftImputed > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.s7872.card.gift_imputed">Imputed gift (uses § 2503 exclusion)</div>
                        <div class="value">$${giftImputed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                ${safeHarbor10k ? `
                    <div class="card pos">
                        <div class="label" data-i18n="view.s7872.card.sh10k">$10k safe harbor</div>
                        <div class="value">EXEMPT</div>
                    </div>
                ` : ''}
                ${safeHarbor100k ? `
                    <div class="card pos">
                        <div class="label" data-i18n="view.s7872.card.sh100k">$100k NII cap</div>
                        <div class="value">APPLIES</div>
                    </div>
                ` : ''}
            </div>
        </div>
    `;
}
