// IRC § 451 — Income Inclusion + Advance Payments (TCJA + IRA 2022 Updates).
// § 451(b): "all events test" — fixed right to receive + amount determinable + economic performance.
// § 451(c) (TCJA): advance payments deferred up to 1 year if also deferred for book purposes.
// Rev. Proc. 2004-34 + 2024 final regs: detailed advance payment rules for goods, services, IP licenses.
// All-events test: income reported when ALL EVENTS have occurred fixing right to receive.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    advance_payment_received: 0,
    services_provided_year_1: 0,
    services_provided_year_2: 0,
    payment_type: 'mixed_goods_services',
    is_book_deferring: false,
    can_defer_under_451c: true,
    is_applicable_financial_statement: true,
    marginal_rate: 0.21,
    is_full_inclusion_election: false,
};

export async function renderSection451(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s451.h1.title">// § 451 INCOME + ADVANCE PAYMENTS</span></h1>
        <p class="muted small" data-i18n="view.s451.hint.intro">
            <strong>§ 451(b) "all events test":</strong> fixed right to receive + amount
            determinable + economic performance. <strong>§ 451(c) (TCJA):</strong> advance payments
            deferred up to 1 year if also deferred for book purposes. <strong>Rev. Proc. 2004-34</strong>
            + 2024 final regs: detailed advance payment rules for goods, services, IP licenses.
            <strong>Full inclusion election:</strong> include all advance payments in year-1.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s451.h2.inputs">Inputs</h2>
            <form id="s451-form" class="inline-form">
                <label><span data-i18n="view.s451.label.payment">Advance payment received ($)</span>
                    <input type="number" step="1000" name="advance_payment_received" value="${state.advance_payment_received}"></label>
                <label><span data-i18n="view.s451.label.year_1_services">Services delivered year 1 ($)</span>
                    <input type="number" step="1000" name="services_provided_year_1" value="${state.services_provided_year_1}"></label>
                <label><span data-i18n="view.s451.label.year_2_services">Services delivered year 2 ($)</span>
                    <input type="number" step="1000" name="services_provided_year_2" value="${state.services_provided_year_2}"></label>
                <label><span data-i18n="view.s451.label.type">Payment type</span>
                    <select name="payment_type">
                        <option value="mixed_goods_services" ${state.payment_type === 'mixed_goods_services' ? 'selected' : ''}>Mixed goods + services</option>
                        <option value="goods" ${state.payment_type === 'goods' ? 'selected' : ''}>Goods</option>
                        <option value="services" ${state.payment_type === 'services' ? 'selected' : ''}>Services</option>
                        <option value="memberships" ${state.payment_type === 'memberships' ? 'selected' : ''}>Memberships / subscriptions</option>
                        <option value="ip_license" ${state.payment_type === 'ip_license' ? 'selected' : ''}>IP license</option>
                        <option value="guarantees" ${state.payment_type === 'guarantees' ? 'selected' : ''}>Warranties / guarantees</option>
                        <option value="travel">Travel + transport reservations</option>
                        <option value="rent" ${state.payment_type === 'rent' ? 'selected' : ''}>Rent / leases (NOT eligible for deferral)</option>
                        <option value="insurance" ${state.payment_type === 'insurance' ? 'selected' : ''}>Insurance premiums (different rules)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s451.label.book_deferring">Book deferring as well?</span>
                    <input type="checkbox" name="is_book_deferring" ${state.is_book_deferring ? 'checked' : ''}></label>
                <label><span data-i18n="view.s451.label.eligible_451c">Eligible for § 451(c) deferral?</span>
                    <input type="checkbox" name="can_defer_under_451c" ${state.can_defer_under_451c ? 'checked' : ''}></label>
                <label><span data-i18n="view.s451.label.afs">Has applicable financial statement?</span>
                    <input type="checkbox" name="is_applicable_financial_statement" ${state.is_applicable_financial_statement ? 'checked' : ''}></label>
                <label><span data-i18n="view.s451.label.marginal">Marginal rate</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <label><span data-i18n="view.s451.label.full_election">Full inclusion election?</span>
                    <input type="checkbox" name="is_full_inclusion_election" ${state.is_full_inclusion_election ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s451.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s451-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s451.h2.eligible_payments">Eligible advance payment categories (post-2024 final regs)</h2>
            <ul class="muted small">
                <li data-i18n="view.s451.elig.goods">Goods + components (incl. shipping if separable)</li>
                <li data-i18n="view.s451.elig.services">Services + warranty + maintenance</li>
                <li data-i18n="view.s451.elig.memberships">Memberships / club dues / subscriptions</li>
                <li data-i18n="view.s451.elig.ip_use">Use of intellectual property (license)</li>
                <li data-i18n="view.s451.elig.occupancy">Occupancy of space (excl. real property rent)</li>
                <li data-i18n="view.s451.elig.eligible_loans">Loan commitments + standby letters of credit</li>
                <li data-i18n="view.s451.elig.guaranty">Guaranty / warranty fees</li>
                <li data-i18n="view.s451.elig.eligible_transactions">Eligible transactions: prepayment for future delivery</li>
                <li data-i18n="view.s451.elig.NOT_rent">NOT eligible: rent for real property, royalties for resource extraction, financial instruments</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s451.h2.deferral_methods">Deferral methods</h2>
            <ul class="muted small">
                <li data-i18n="view.s451.def.deferral_method">Deferral method: defer revenue NOT recognized for book in year of receipt</li>
                <li data-i18n="view.s451.def.full_inclusion">Full inclusion election: irrevocable, include all advance payments in year 1</li>
                <li data-i18n="view.s451.def.afs_method">AFS method: track recognition matching AFS treatment</li>
                <li data-i18n="view.s451.def.non_afs">Non-AFS method: recognize based on tax / book pattern</li>
                <li data-i18n="view.s451.def.maximum_1_year">Maximum 1-year deferral; year-2 forced inclusion</li>
                <li data-i18n="view.s451.def.cessation">Cessation of business: full inclusion in final year</li>
                <li data-i18n="view.s451.def.section_481">Change requires § 481(a) Form 3115</li>
            </ul>
        </div>
    `;
    document.getElementById('s451-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.advance_payment_received = Number(fd.get('advance_payment_received')) || 0;
        state.services_provided_year_1 = Number(fd.get('services_provided_year_1')) || 0;
        state.services_provided_year_2 = Number(fd.get('services_provided_year_2')) || 0;
        state.payment_type = fd.get('payment_type');
        state.is_book_deferring = !!fd.get('is_book_deferring');
        state.can_defer_under_451c = !!fd.get('can_defer_under_451c');
        state.is_applicable_financial_statement = !!fd.get('is_applicable_financial_statement');
        state.is_full_inclusion_election = !!fd.get('is_full_inclusion_election');
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.21;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s451-output');
    if (!el) return;
    const eligibleTypes = ['mixed_goods_services', 'goods', 'services', 'memberships', 'ip_license', 'guarantees', 'travel'];
    const eligible = state.can_defer_under_451c && eligibleTypes.includes(state.payment_type);
    let year1Income, year2Income;
    if (state.is_full_inclusion_election || !eligible) {
        year1Income = state.advance_payment_received;
        year2Income = 0;
    } else if (state.is_book_deferring) {
        // Mirror book treatment: defer based on services not yet performed
        year1Income = state.services_provided_year_1;
        year2Income = state.advance_payment_received - year1Income;
    } else {
        year1Income = state.advance_payment_received;
        year2Income = 0;
    }
    const year1Tax = year1Income * state.marginal_rate;
    const year2Tax = year2Income * state.marginal_rate;
    const taxIfFullInclusion = state.advance_payment_received * state.marginal_rate;
    const deferralValue = taxIfFullInclusion - year1Tax;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s451.h2.result">Recognition timing</h2>
            <div class="cards">
                <div class="card ${eligible ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s451.card.eligible">Eligible for deferral?</div>
                    <div class="value">${eligible ? esc(t('view.s451.status.yes')) : esc(t('view.s451.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s451.card.year1">Year-1 income</div>
                    <div class="value">$${year1Income.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s451.card.year2">Year-2 income</div>
                    <div class="value">$${year2Income.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s451.card.year1_tax">Year-1 tax</div>
                    <div class="value">$${year1Tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s451.card.year2_tax">Year-2 tax</div>
                    <div class="value">$${year2Tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s451.card.deferral_value">Year-1 deferral value</div>
                    <div class="value">$${deferralValue.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
