// IRC § 7122 — Offer in Compromise (OIC).
// IRS settles tax debt for less when collection in full doubtful.
// 3 grounds: (1) Doubt as to Collectibility (DATC), (2) Doubt as to Liability (DATL),
// (3) Effective Tax Administration (ETA).
// Reasonable Collection Potential (RCP) = current equity + future income (12 or 24 mo).
// $205 application fee + Form 656 + Form 433-A(OIC) / 433-B(OIC).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const APPLICATION_FEE_2024 = 205;
const FUTURE_INCOME_LUMP_MONTHS = 12;
const FUTURE_INCOME_PERIODIC_MONTHS = 24;

let state = {
    total_tax_debt: 0,
    grounds: 'datc',
    monthly_gross_income: 0,
    monthly_allowable_expenses: 0,
    home_equity: 0,
    vehicle_equity: 0,
    retirement_equity: 0,
    investments_value: 0,
    bank_balance: 0,
    other_assets: 0,
    payment_option: 'lump_sum',
    is_low_income: false,
};

export async function renderSection7122(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s7122.h1.title">// § 7122 OFFER IN COMPROMISE</span></h1>
        <p class="muted small" data-i18n="view.s7122.hint.intro">
            IRS settles for less when full collection doubtful. <strong>3 grounds:</strong>
            (1) Doubt as to Collectibility (DATC), (2) Doubt as to Liability (DATL), (3) Effective
            Tax Administration (ETA). <strong>Reasonable Collection Potential (RCP)</strong> =
            net equity + future income × 12 mo (lump) or 24 mo (periodic). <strong>$205
            application fee</strong> + Form 656 + Form 433-A(OIC) / 433-B(OIC). Low-income waiver
            available.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s7122.h2.inputs">Inputs</h2>
            <form id="s7122-form" class="inline-form">
                <label><span data-i18n="view.s7122.label.tax_debt">Total tax debt ($)</span>
                    <input type="number" step="1000" name="total_tax_debt" value="${state.total_tax_debt}"></label>
                <label><span data-i18n="view.s7122.label.grounds">Grounds</span>
                    <select name="grounds">
                        <option value="datc" ${state.grounds === 'datc' ? 'selected' : ''}>Doubt as to Collectibility (DATC)</option>
                        <option value="datl" ${state.grounds === 'datl' ? 'selected' : ''}>Doubt as to Liability (DATL)</option>
                        <option value="eta" ${state.grounds === 'eta' ? 'selected' : ''}>Effective Tax Administration (ETA)</option>
                    </select>
                </label>
                <hr style="grid-column:1/-1">
                <label><span data-i18n="view.s7122.label.gross_income">Monthly gross income ($)</span>
                    <input type="number" step="100" name="monthly_gross_income" value="${state.monthly_gross_income}"></label>
                <label><span data-i18n="view.s7122.label.expenses">Monthly allowable expenses ($)</span>
                    <input type="number" step="100" name="monthly_allowable_expenses" value="${state.monthly_allowable_expenses}"></label>
                <hr style="grid-column:1/-1">
                <label><span data-i18n="view.s7122.label.home_equity">Home equity ($)</span>
                    <input type="number" step="10000" name="home_equity" value="${state.home_equity}"></label>
                <label><span data-i18n="view.s7122.label.vehicle">Vehicle equity ($)</span>
                    <input type="number" step="1000" name="vehicle_equity" value="${state.vehicle_equity}"></label>
                <label><span data-i18n="view.s7122.label.retirement">Retirement equity (× 0.80) ($)</span>
                    <input type="number" step="10000" name="retirement_equity" value="${state.retirement_equity}"></label>
                <label><span data-i18n="view.s7122.label.investments">Investments ($)</span>
                    <input type="number" step="1000" name="investments_value" value="${state.investments_value}"></label>
                <label><span data-i18n="view.s7122.label.bank">Bank balance ($)</span>
                    <input type="number" step="100" name="bank_balance" value="${state.bank_balance}"></label>
                <label><span data-i18n="view.s7122.label.other">Other assets ($)</span>
                    <input type="number" step="1000" name="other_assets" value="${state.other_assets}"></label>
                <hr style="grid-column:1/-1">
                <label><span data-i18n="view.s7122.label.payment">Payment option</span>
                    <select name="payment_option">
                        <option value="lump_sum" ${state.payment_option === 'lump_sum' ? 'selected' : ''}>Lump sum (5 payments / 5 mo)</option>
                        <option value="periodic" ${state.payment_option === 'periodic' ? 'selected' : ''}>Periodic payment (24 mo)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s7122.label.low_income">Low-income waiver?</span>
                    <input type="checkbox" name="is_low_income" ${state.is_low_income ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s7122.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s7122-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7122.h2.payment_options">Payment options</h2>
            <ul class="muted small">
                <li data-i18n="view.s7122.po.lump_sum">Lump sum: 20% with offer + 5 payments within 5 months of acceptance</li>
                <li data-i18n="view.s7122.po.periodic">Periodic: initial + monthly payments DURING evaluation + after acceptance, 24 mo total</li>
                <li data-i18n="view.s7122.po.deferred">Deferred Payment Plan: pay over remaining CSED (rare, after IA failure)</li>
                <li data-i18n="view.s7122.po.application_fee">$205 application fee (waived if low-income or DATL)</li>
                <li data-i18n="view.s7122.po.during_eval">Payments during eval keep accruing — non-refundable if rejected</li>
                <li data-i18n="view.s7122.po.compliance">5-year future compliance requirement (file + pay timely)</li>
                <li data-i18n="view.s7122.po.default">Default: full balance owed + add'l penalties</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7122.h2.allowable_expenses">IRS Allowable Living Expenses (ALE) categories</h2>
            <ul class="muted small">
                <li data-i18n="view.s7122.ale.food_clothing">Food + clothing + personal care + household supplies + miscellaneous</li>
                <li data-i18n="view.s7122.ale.housing">Housing + utilities (county-specific by family size)</li>
                <li data-i18n="view.s7122.ale.transport">Transportation: ownership + operating costs by region</li>
                <li data-i18n="view.s7122.ale.health">Out-of-pocket health care (above insurance)</li>
                <li data-i18n="view.s7122.ale.medical_insurance">Health insurance premiums (actual)</li>
                <li data-i18n="view.s7122.ale.taxes">Court-ordered taxes + child support + alimony</li>
                <li data-i18n="view.s7122.ale.life_insurance">Life insurance (term up to allowance)</li>
                <li data-i18n="view.s7122.ale.education">Education for dependent kids (within standard)</li>
                <li data-i18n="view.s7122.ale.retirement_required">Mandatory retirement contribution (if employment requires)</li>
            </ul>
        </div>
    `;
    document.getElementById('s7122-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.total_tax_debt = Number(fd.get('total_tax_debt')) || 0;
        state.grounds = fd.get('grounds');
        state.monthly_gross_income = Number(fd.get('monthly_gross_income')) || 0;
        state.monthly_allowable_expenses = Number(fd.get('monthly_allowable_expenses')) || 0;
        state.home_equity = Number(fd.get('home_equity')) || 0;
        state.vehicle_equity = Number(fd.get('vehicle_equity')) || 0;
        state.retirement_equity = Number(fd.get('retirement_equity')) || 0;
        state.investments_value = Number(fd.get('investments_value')) || 0;
        state.bank_balance = Number(fd.get('bank_balance')) || 0;
        state.other_assets = Number(fd.get('other_assets')) || 0;
        state.payment_option = fd.get('payment_option');
        state.is_low_income = !!fd.get('is_low_income');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s7122-output');
    if (!el) return;
    const monthlyDisposable = Math.max(0, state.monthly_gross_income - state.monthly_allowable_expenses);
    const futureIncomeMonths = state.payment_option === 'lump_sum' ? FUTURE_INCOME_LUMP_MONTHS : FUTURE_INCOME_PERIODIC_MONTHS;
    const futureIncome = monthlyDisposable * futureIncomeMonths;
    const retirementAfterQSV = state.retirement_equity * 0.80;
    const homeAfterQSV = state.home_equity * 0.80;
    const vehicleAfterQSV = state.vehicle_equity * 0.80;
    const otherAfterQSV = state.other_assets * 0.80;
    const investmentsAfterQSV = state.investments_value * 0.80;
    const netEquity = homeAfterQSV + vehicleAfterQSV + retirementAfterQSV
        + investmentsAfterQSV + state.bank_balance + otherAfterQSV;
    const rcp = futureIncome + netEquity;
    const offerAmount = Math.min(state.total_tax_debt, rcp);
    const savings = state.total_tax_debt - offerAmount;
    const applicationFee = state.is_low_income ? 0 : APPLICATION_FEE_2024;
    const initialPayment = state.payment_option === 'lump_sum' ? offerAmount * 0.20 : Math.ceil(offerAmount / 24);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s7122.h2.result">OIC calculation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s7122.card.disposable">Monthly disposable income</div>
                    <div class="value">$${monthlyDisposable.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s7122.card.future_income">Future income (${futureIncomeMonths} mo)</div>
                    <div class="value">$${futureIncome.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s7122.card.net_equity">Net equity (QSV × 0.80)</div>
                    <div class="value">$${netEquity.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s7122.card.rcp">Reasonable Collection Potential</div>
                    <div class="value">$${rcp.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s7122.card.offer">Offer amount</div>
                    <div class="value">$${offerAmount.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s7122.card.savings">Debt savings</div>
                    <div class="value">$${savings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s7122.card.fee">Application fee</div>
                    <div class="value">$${applicationFee.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s7122.card.initial">Initial payment</div>
                    <div class="value">$${initialPayment.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
