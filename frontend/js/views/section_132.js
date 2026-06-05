// IRC § 132 — Statutory Fringe Benefits (Excluded from Income).
// 8 categories: (a)(1) No-Additional-Cost Service, (a)(2) Qualified Employee Discount,
// (a)(3) Working Condition Fringe, (a)(4) De Minimis, (a)(5) Qualified Transportation,
// (a)(6) Qualified Moving (military only post-TCJA), (a)(7) Retirement Planning, (a)(8) Athletic.
// Non-discrimination rules limit highly-compensated.

import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const TRANSIT_LIMIT_2024 = 315;
const PARKING_LIMIT_2024 = 315;

let state = {
    is_employee: true,
    employer_qualified_discount_pct: 0,
    cost_of_property_or_service_to_employer: 0,
    customer_price: 0,
    transit_passes_monthly: 0,
    parking_monthly: 0,
    bicycle_reimbursement: 0,
    other_fringe_value: 0,
    is_highly_compensated: false,
    employer_passes_nondiscrim: true,
    is_retired_employee: false,
};

export async function renderSection132(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s132.h1.title">// § 132 STATUTORY FRINGE BENEFITS</span></h1>
        <p class="muted small" data-i18n="view.s132.hint.intro">
            8 categories excluded from employee income: <strong>(a)(1) No-Additional-Cost,
            (a)(2) Qualified Employee Discount, (a)(3) Working Condition, (a)(4) De Minimis,
            (a)(5) Qualified Transportation ($315/mo 2024), (a)(6) Qualified Moving (military
            only), (a)(7) Retirement Planning, (a)(8) Athletic Facility.</strong> Non-discrim
            rules limit highly-compensated. TCJA 2018 eliminated employer deduction for
            qualified transportation.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s132.h2.inputs">Inputs</h2>
            <form id="s132-form" class="inline-form">
                <label><span data-i18n="view.s132.label.employee">Are you the employee?</span>
                    <input type="checkbox" name="is_employee" ${state.is_employee ? 'checked' : ''}></label>
                <label><span data-i18n="view.s132.label.discount_pct">Employer discount %</span>
                    <input type="number" step="0.01" name="employer_qualified_discount_pct" value="${state.employer_qualified_discount_pct}"></label>
                <label><span data-i18n="view.s132.label.cost">Employer cost ($)</span>
                    <input type="number" step="0.01" name="cost_of_property_or_service_to_employer" value="${state.cost_of_property_or_service_to_employer}"></label>
                <label><span data-i18n="view.s132.label.customer">Customer price ($)</span>
                    <input type="number" step="0.01" name="customer_price" value="${state.customer_price}"></label>
                <label><span data-i18n="view.s132.label.transit">Transit passes monthly ($)</span>
                    <input type="number" step="0.01" name="transit_passes_monthly" value="${state.transit_passes_monthly}"></label>
                <label><span data-i18n="view.s132.label.parking">Parking monthly ($)</span>
                    <input type="number" step="0.01" name="parking_monthly" value="${state.parking_monthly}"></label>
                <label><span data-i18n="view.s132.label.bike">Bicycle reimbursement annually ($)</span>
                    <input type="number" step="0.01" name="bicycle_reimbursement" value="${state.bicycle_reimbursement}"></label>
                <label><span data-i18n="view.s132.label.other">Other fringe value ($)</span>
                    <input type="number" step="0.01" name="other_fringe_value" value="${state.other_fringe_value}"></label>
                <label><span data-i18n="view.s132.label.hce">Highly compensated employee?</span>
                    <input type="checkbox" name="is_highly_compensated" ${state.is_highly_compensated ? 'checked' : ''}></label>
                <label><span data-i18n="view.s132.label.nondiscrim">Employer passes non-discrim?</span>
                    <input type="checkbox" name="employer_passes_nondiscrim" ${state.employer_passes_nondiscrim ? 'checked' : ''}></label>
                <label><span data-i18n="view.s132.label.retired">Retired employee?</span>
                    <input type="checkbox" name="is_retired_employee" ${state.is_retired_employee ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s132.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s132-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s132.h2.categories">8 Categories</h2>
            <ol class="muted small">
                <li data-i18n="view.s132.cat.no_add_cost">(a)(1) No-Additional-Cost: airline space-available, hotel rooms — no employer marginal cost</li>
                <li data-i18n="view.s132.cat.discount">(a)(2) Qualified Discount: ≤ employer gross profit % (or 20% for services)</li>
                <li data-i18n="view.s132.cat.working">(a)(3) Working Condition: laptop, magazine subscription, professional dues</li>
                <li data-i18n="view.s132.cat.de_minimis">(a)(4) De Minimis: occasional snacks, supper money, holiday gifts < $100</li>
                <li data-i18n="view.s132.cat.transport">(a)(5) Qualified Transportation: $315/mo (2024) transit + parking each</li>
                <li data-i18n="view.s132.cat.moving">(a)(6) Qualified Moving: MILITARY ONLY post-TCJA</li>
                <li data-i18n="view.s132.cat.retirement">(a)(7) Retirement Planning: ≤ $1k qualified retirement education</li>
                <li data-i18n="view.s132.cat.athletic">(a)(8) Athletic Facility: on-premises gym, mostly used by employees</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s132.h2.de_minimis">De minimis examples (§ 132(e))</h2>
            <ul class="muted small">
                <li data-i18n="view.s132.dm.cab">Occasional taxi for late-night work</li>
                <li data-i18n="view.s132.dm.snacks">Coffee, donuts, snacks at office</li>
                <li data-i18n="view.s132.dm.holiday">Holiday gift (turkey, ham, fruit basket)</li>
                <li data-i18n="view.s132.dm.tickets">Occasional sports / theater tickets</li>
                <li data-i18n="view.s132.dm.copies">Personal use of office copier</li>
                <li data-i18n="view.s132.dm.party">Company picnics + parties</li>
                <li data-i18n="view.s132.dm.flowers">Flowers / fruit for special events (achievement, illness)</li>
                <li data-i18n="view.s132.dm.tshirts">Inexpensive promotional t-shirts</li>
                <li data-i18n="view.s132.dm.gift_cards">CASH or CASH EQUIVALENT (gift cards) NEVER de minimis</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s132.h2.transit_2024">Transportation fringe (§ 132(f)) 2024 limits</h2>
            <ul class="muted small">
                <li data-i18n="view.s132.tr.transit">Transit pass / van pool: $315/month</li>
                <li data-i18n="view.s132.tr.parking">Qualified parking: $315/month</li>
                <li data-i18n="view.s132.tr.cash">Cash-in-lieu options taxable as wages</li>
                <li data-i18n="view.s132.tr.bicycle">Bicycle commuting reimbursement: SUSPENDED 2018-2025 (TCJA)</li>
                <li data-i18n="view.s132.tr.no_deduct">Employer can't DEDUCT qualified transportation expense (TCJA repeal)</li>
                <li data-i18n="view.s132.tr.combine">Cannot combine: transit + parking allowed but separate limits</li>
                <li data-i18n="view.s132.tr.salary_reduction">Pre-tax salary reduction allowed (employee pays less FICA + income tax)</li>
            </ul>
        </div>
    `;
    document.getElementById('s132-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.is_employee = !!fd.get('is_employee');
        state.employer_qualified_discount_pct = Number(fd.get('employer_qualified_discount_pct')) || 0;
        state.cost_of_property_or_service_to_employer = Number(fd.get('cost_of_property_or_service_to_employer')) || 0;
        state.customer_price = Number(fd.get('customer_price')) || 0;
        state.transit_passes_monthly = Number(fd.get('transit_passes_monthly')) || 0;
        state.parking_monthly = Number(fd.get('parking_monthly')) || 0;
        state.bicycle_reimbursement = Number(fd.get('bicycle_reimbursement')) || 0;
        state.other_fringe_value = Number(fd.get('other_fringe_value')) || 0;
        state.is_highly_compensated = !!fd.get('is_highly_compensated');
        state.employer_passes_nondiscrim = !!fd.get('employer_passes_nondiscrim');
        state.is_retired_employee = !!fd.get('is_retired_employee');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s132-output');
    if (!el) return;
    const employerGrossProfit = state.customer_price > 0
        ? (state.customer_price - state.cost_of_property_or_service_to_employer) / state.customer_price
        : 0;
    const allowedDiscount = employerGrossProfit;
    const actualDiscountValue = state.customer_price * state.employer_qualified_discount_pct;
    const excessDiscount = Math.max(0, state.employer_qualified_discount_pct - allowedDiscount) * state.customer_price;
    const transitMonthly = Math.min(state.transit_passes_monthly, TRANSIT_LIMIT_2024);
    const parkingMonthly = Math.min(state.parking_monthly, PARKING_LIMIT_2024);
    const annualTransportExclusion = (transitMonthly + parkingMonthly) * 12;
    const transitExcess = (state.transit_passes_monthly - TRANSIT_LIMIT_2024 + state.parking_monthly - PARKING_LIMIT_2024) * 12;
    const totalExcluded = actualDiscountValue - excessDiscount + annualTransportExclusion + state.other_fringe_value;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s132.h2.result">Exclusion calculation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s132.card.discount_limit">Discount limit (gross profit %)</div>
                    <div class="value">${(allowedDiscount * 100).toFixed(0)}%</div>
                </div>
                <div class="card ${excessDiscount > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s132.card.excess_discount">Excess discount (taxable)</div>
                    <div class="value">$${excessDiscount.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s132.card.annual_transport">Annual transportation excluded</div>
                    <div class="value">$${annualTransportExclusion.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${transitExcess > 0 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s132.card.transit_excess">Transit/parking excess</div>
                    <div class="value">$${Math.max(0, transitExcess).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s132.card.total">Total excluded from income</div>
                    <div class="value">$${totalExcluded.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
