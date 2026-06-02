// IRC § 461 — When Deductions Allowed (Economic Performance + Recurring Item).
// All-events test: (1) all events fix liability, (2) amount determinable, (3) economic performance.
// "Economic performance" varies by category: services received, property used, payment for liability.
// § 461(h)(3) Recurring item exception: recurring liabilities can be accrued before EP if paid within 8.5 mo.
// § 461(l) Excess business loss limit ($305k/$610k).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const EBL_2024_SINGLE = 305_000;
const EBL_2024_MFJ = 610_000;

let state = {
    expense_kind: 'services',
    accrued_amount: 0,
    payment_date_months_after_year_end: 0,
    is_recurring_item: false,
    is_economic_performance_complete: false,
    excess_business_loss_amount: 0,
    other_business_income: 0,
    is_mfj: false,
    marginal_rate: 0.32,
};

export async function renderSection461(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s461.h1.title">// § 461 ECONOMIC PERFORMANCE</span></h1>
        <p class="muted small" data-i18n="view.s461.hint.intro">
            <strong>All-events test:</strong> (1) all events fix liability, (2) amount determinable,
            (3) economic performance. <strong>Economic performance</strong> varies: services
            received, property used, payment for liability. <strong>§ 461(h)(3) Recurring item
            exception:</strong> recurring liabilities accrue before EP if paid within 8.5 months
            after year-end. <strong>§ 461(l) Excess Business Loss limit:</strong> $305k/$610k 2024.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s461.h2.inputs">Inputs</h2>
            <form id="s461-form" class="inline-form">
                <label><span data-i18n="view.s461.label.kind">Expense kind</span>
                    <select name="expense_kind">
                        <option value="services" ${state.expense_kind === 'services' ? 'selected' : ''}>Services received</option>
                        <option value="goods" ${state.expense_kind === 'goods' ? 'selected' : ''}>Goods received</option>
                        <option value="property_use" ${state.expense_kind === 'property_use' ? 'selected' : ''}>Property use (rent / royalty)</option>
                        <option value="rebates" ${state.expense_kind === 'rebates' ? 'selected' : ''}>Rebates + refunds</option>
                        <option value="awards_prizes" ${state.expense_kind === 'awards_prizes' ? 'selected' : ''}>Awards / prizes</option>
                        <option value="insurance">Insurance premiums</option>
                        <option value="taxes" ${state.expense_kind === 'taxes' ? 'selected' : ''}>Taxes (property / payroll)</option>
                        <option value="workers_comp" ${state.expense_kind === 'workers_comp' ? 'selected' : ''}>Workers comp</option>
                        <option value="self_insurance" ${state.expense_kind === 'self_insurance' ? 'selected' : ''}>Self-insurance</option>
                        <option value="warranties" ${state.expense_kind === 'warranties' ? 'selected' : ''}>Product warranties</option>
                        <option value="environmental" ${state.expense_kind === 'environmental' ? 'selected' : ''}>Environmental remediation</option>
                    </select>
                </label>
                <label><span data-i18n="view.s461.label.accrued">Accrued amount ($)</span>
                    <input type="number" step="1000" name="accrued_amount" value="${state.accrued_amount}"></label>
                <label><span data-i18n="view.s461.label.payment_date">Payment months after year-end</span>
                    <input type="number" step="0.5" name="payment_date_months_after_year_end" value="${state.payment_date_months_after_year_end}"></label>
                <label><span data-i18n="view.s461.label.recurring">Recurring item?</span>
                    <input type="checkbox" name="is_recurring_item" ${state.is_recurring_item ? 'checked' : ''}></label>
                <label><span data-i18n="view.s461.label.ep_complete">EP complete in current year?</span>
                    <input type="checkbox" name="is_economic_performance_complete" ${state.is_economic_performance_complete ? 'checked' : ''}></label>
                <hr style="grid-column:1/-1">
                <label><span data-i18n="view.s461.label.ebl">Excess business loss ($)</span>
                    <input type="number" step="1000" name="excess_business_loss_amount" value="${state.excess_business_loss_amount}"></label>
                <label><span data-i18n="view.s461.label.other_biz_income">Other non-business income ($)</span>
                    <input type="number" step="1000" name="other_business_income" value="${state.other_business_income}"></label>
                <label><span data-i18n="view.s461.label.mfj">MFJ?</span>
                    <input type="checkbox" name="is_mfj" ${state.is_mfj ? 'checked' : ''}></label>
                <label><span data-i18n="view.s461.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s461.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s461-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s461.h2.economic_performance">Economic performance by category</h2>
            <ul class="muted small">
                <li data-i18n="view.s461.ep.services">Services: when provider performs services</li>
                <li data-i18n="view.s461.ep.goods">Property: when delivered / accepted</li>
                <li data-i18n="view.s461.ep.use_property">Use of property: ratably over period of use</li>
                <li data-i18n="view.s461.ep.payment_liabilities">Payment-type liabilities (rebates, awards, insurance, workers comp): when PAID</li>
                <li data-i18n="view.s461.ep.taxes_assessed">Real property + payroll taxes: ratably accrued</li>
                <li data-i18n="view.s461.ep.warranties">Warranty obligations: when work performed (services)</li>
                <li data-i18n="view.s461.ep.environmental">Environmental remediation: when work performed</li>
                <li data-i18n="view.s461.ep.interest">Interest: with passage of time (separate § 461 timing rules)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s461.h2.recurring_exception">§ 461(h)(3) Recurring item exception</h2>
            <ul class="muted small">
                <li data-i18n="view.s461.re.recurring">Item is recurring (year over year)</li>
                <li data-i18n="view.s461.re.paid_8_5">Liability paid within 8.5 months after year-end (or with extension)</li>
                <li data-i18n="view.s461.re.material">Either: not material OR matches income better in current year</li>
                <li data-i18n="view.s461.re.not_payment_liability">Doesn't apply to payment liabilities (rebates, prizes, workers comp, insurance, etc.)</li>
                <li data-i18n="view.s461.re.elect">Election made first year + consistent thereafter</li>
                <li data-i18n="view.s461.re.cash_method">Cash method always uses when-paid; this is accrual-method election</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s461.h2.ebl_461l">§ 461(l) Excess Business Loss limit (TCJA, IRA extended through 2028)</h2>
            <ul class="muted small">
                <li data-i18n="view.s461.ebl.limit">Limit: $305k single / $610k MFJ (2024, inflation-indexed)</li>
                <li data-i18n="view.s461.ebl.non_corp">Non-corporate taxpayers only (corps + S-corps not subject)</li>
                <li data-i18n="view.s461.ebl.disallowed">Disallowed loss carries forward as NOL</li>
                <li data-i18n="view.s461.ebl.aggregate">Aggregate all business activities (rentals + trades)</li>
                <li data-i18n="view.s461.ebl.includes_wages">Wages / salary count as business income for EBL test</li>
                <li data-i18n="view.s461.ebl.no_passive">Stacks on top of § 469 passive activity + § 465 at-risk</li>
                <li data-i18n="view.s461.ebl.ira_extension">Inflation Reduction Act 2022 extended § 461(l) through 12/31/2028</li>
            </ul>
        </div>
    `;
    document.getElementById('s461-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.expense_kind = fd.get('expense_kind');
        state.accrued_amount = Number(fd.get('accrued_amount')) || 0;
        state.payment_date_months_after_year_end = Number(fd.get('payment_date_months_after_year_end')) || 0;
        state.is_recurring_item = !!fd.get('is_recurring_item');
        state.is_economic_performance_complete = !!fd.get('is_economic_performance_complete');
        state.excess_business_loss_amount = Number(fd.get('excess_business_loss_amount')) || 0;
        state.other_business_income = Number(fd.get('other_business_income')) || 0;
        state.is_mfj = !!fd.get('is_mfj');
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s461-output');
    if (!el) return;
    const paymentTypes = ['rebates', 'awards_prizes', 'insurance', 'workers_comp', 'self_insurance'];
    const isPaymentLiability = paymentTypes.includes(state.expense_kind);
    const meetsAllEvents = state.is_economic_performance_complete
        || (state.is_recurring_item && state.payment_date_months_after_year_end <= 8.5 && !isPaymentLiability);
    const deductibleCurrentYear = meetsAllEvents ? state.accrued_amount : 0;
    const eblLimit = state.is_mfj ? EBL_2024_MFJ : EBL_2024_SINGLE;
    const eblAllowed = Math.min(state.excess_business_loss_amount, state.other_business_income + eblLimit);
    const eblDisallowed = Math.max(0, state.excess_business_loss_amount - eblAllowed);
    const taxSavings = deductibleCurrentYear * state.marginal_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s461.h2.result">All-events test + § 461(l)</h2>
            <div class="cards">
                <div class="card ${meetsAllEvents ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s461.card.meets">Meets all-events test?</div>
                    <div class="value">${meetsAllEvents ? esc(t('view.s461.status.yes')) : esc(t('view.s461.status.no'))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s461.card.deductible">Deductible current year</div>
                    <div class="value">$${deductibleCurrentYear.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s461.card.tax_savings">Tax savings</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s461.card.ebl_limit">§ 461(l) limit</div>
                    <div class="value">$${eblLimit.toLocaleString()}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s461.card.ebl_allowed">EBL allowed</div>
                    <div class="value">$${eblAllowed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${eblDisallowed > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s461.card.ebl_disallowed">EBL disallowed → NOL</div>
                    <div class="value">$${eblDisallowed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
