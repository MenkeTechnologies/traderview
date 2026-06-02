// IRC § 4940 — Private Foundation Net Investment Income Tax.
// 1.39% flat tax on PF net investment income (interest, dividends, royalties, rents, capital gains).
// Pre-2020 was 2% with 1% rate available if distribution test met (eliminated by SECURE Act).
// Form 990-PF Part XI. Net long-term cap losses do NOT offset other income.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const RATE = 0.0139;

let state = {
    gross_investment_income: 0,
    capital_gains_net: 0,
    investment_expenses: 0,
    excise_payments_made: 0,
};

export async function renderSection4940(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s4940.h1.title">// § 4940 PF NET INVESTMENT INCOME</span></h1>
        <p class="muted small" data-i18n="view.s4940.hint.intro">
            <strong>1.39% flat tax</strong> on PF net investment income (interest, dividends,
            royalties, rents, net capital gains). Pre-2020 was 2% with reduced 1% rate available
            if distribution test met — <strong>SECURE Act eliminated 2-tier</strong>, made flat 1.39%.
            Reported on <strong>Form 990-PF Part XI</strong>. <strong>Net long-term capital losses
            do NOT offset other income</strong> (PF specific). Quarterly estimated tax payments
            required if &gt; $500.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s4940.h2.inputs">Inputs</h2>
            <form id="s4940-form" class="inline-form">
                <label><span data-i18n="view.s4940.label.gross">Gross investment income ($)</span>
                    <input type="number" step="1000" name="gross_investment_income" value="${state.gross_investment_income}"></label>
                <label><span data-i18n="view.s4940.label.cap_gains">Net capital gains ($)</span>
                    <input type="number" step="1000" name="capital_gains_net" value="${state.capital_gains_net}"></label>
                <label><span data-i18n="view.s4940.label.expenses">Investment expenses ($)</span>
                    <input type="number" step="1000" name="investment_expenses" value="${state.investment_expenses}"></label>
                <label><span data-i18n="view.s4940.label.payments">Excise payments made YTD ($)</span>
                    <input type="number" step="100" name="excise_payments_made" value="${state.excise_payments_made}"></label>
                <button class="primary" type="submit" data-i18n="view.s4940.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s4940-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4940.h2.includable">Includable investment income</h2>
            <ul class="muted small">
                <li data-i18n="view.s4940.inc.dividends">Dividends + qualified / ordinary</li>
                <li data-i18n="view.s4940.inc.interest">Interest (taxable + tax-exempt to lesser extent)</li>
                <li data-i18n="view.s4940.inc.royalties">Royalties from mineral / IP holdings</li>
                <li data-i18n="view.s4940.inc.rents">Rents (less mortgage interest + depreciation)</li>
                <li data-i18n="view.s4940.inc.cap_gain">Net long-term capital gains (cap losses do NOT offset NII)</li>
                <li data-i18n="view.s4940.inc.short_term">Short-term capital gains (treated as ordinary income)</li>
                <li data-i18n="view.s4940.inc.foreign">Foreign-source income (with FTC if applicable)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4940.h2.deductions">Allowable deductions against NII</h2>
            <ul class="muted small">
                <li data-i18n="view.s4940.ded.investment_advisor">Investment advisor + custodian fees</li>
                <li data-i18n="view.s4940.ded.mortgage_int">Mortgage interest on investment property</li>
                <li data-i18n="view.s4940.ded.tax">State + local property taxes on investment real estate</li>
                <li data-i18n="view.s4940.ded.depreciation">Depreciation on income-producing property (straight-line only)</li>
                <li data-i18n="view.s4940.ded.necessary">Other ordinary + necessary expenses related to income production</li>
                <li data-i18n="view.s4940.ded.no_charitable">NOT charitable program expenses (those go in Part I-A)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4940.h2.estimated">Estimated tax requirements</h2>
            <p class="muted small" data-i18n="view.s4940.est.body">
                If § 4940 tax expected to exceed <strong>$500</strong>, quarterly estimated payments
                required by 5/15, 6/15, 9/15, 12/15. <strong>Form 990-W</strong> for computation.
                Safe harbor: 100% prior year OR 90% current year. Underpayment penalty under § 6655.
            </p>
        </div>
    `;
    document.getElementById('s4940-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.gross_investment_income = Number(fd.get('gross_investment_income')) || 0;
        state.capital_gains_net = Number(fd.get('capital_gains_net')) || 0;
        state.investment_expenses = Number(fd.get('investment_expenses')) || 0;
        state.excise_payments_made = Number(fd.get('excise_payments_made')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s4940-output');
    if (!el) return;
    const netII = Math.max(0, state.gross_investment_income + state.capital_gains_net - state.investment_expenses);
    const excise = netII * RATE;
    const balance = Math.max(0, excise - state.excise_payments_made);
    const requiresEstimatedTax = excise > 500;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s4940.h2.result">§ 4940 calculation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s4940.card.gross">Gross NII</div>
                    <div class="value">$${state.gross_investment_income.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s4940.card.net">Net investment income</div>
                    <div class="value">$${netII.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s4940.card.excise">§ 4940 excise (1.39%)</div>
                    <div class="value">$${excise.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s4940.card.paid">Already paid</div>
                    <div class="value">$${state.excise_payments_made.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${balance > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s4940.card.balance">Balance due</div>
                    <div class="value">$${balance.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${requiresEstimatedTax ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s4940.card.requires_est">Requires quarterly est?</div>
                    <div class="value">${requiresEstimatedTax ? esc(t('view.s4940.status.yes')) : esc(t('view.s4940.status.no'))}</div>
                </div>
            </div>
        </div>
    `;
}
