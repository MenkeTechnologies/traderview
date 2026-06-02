// IRC § 736 — Payments to Retiring Partner or Deceased Partner's Successor in Interest.
// § 736(a) — payments NOT in exchange for partnership property: ordinary income (treated as § 707(c) GP or distributive share).
// § 736(b) — payments IN exchange for partnership property: capital gain/loss (under § 731 + § 741).
// Allocation crucial for partner's tax + partnership's deduction.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    total_payment_amount: 0,
    s736_a_ordinary_portion: 0,
    s736_b_capital_portion: 0,
    partner_outside_basis: 0,
    partner_capital_account: 0,
    partner_share_of_inside_basis: 0,
    is_retiring: false,
    is_deceased: false,
    s736_b_property_payments_partnership: 0,
    s736_b_unrealized_receivables: 0,
    s736_b_substantially_appreciated_inventory: 0,
    s736_a_goodwill_share: 0,
    s736_a_2_b_unstated_goodwill: 0,
    s736_b_goodwill_if_provided: 0,
    is_general_partner: true,
    is_capital_partnership: false,
    is_service_partnership: false,
    s736_b_2_unrealized_receivables_amount: 0,
    s736_b_2_inventory_amount: 0,
    s736_c_payments_excess: 0,
    payments_year: 2024,
    payment_made_in_installments: false,
    installment_periods: 0,
    s736_a_treats_distributive_share: false,
    s736_a_treats_guaranteed_payment: false,
    s453_installment_method: false,
    aggregate_method_s736b: false,
};

export async function renderSection736(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s736.h1.title">// § 736 RETIRING / DECEASED PARTNER PAYMENTS</span></h1>
        <p class="muted small" data-i18n="view.s736.hint.intro">
            <strong>§ 736</strong> bifurcates payments to retiring partner (or successor of deceased
            partner) into two categories: <strong>§ 736(b)</strong> — payments for partnership
            PROPERTY (capital treatment under § 731/§ 741). <strong>§ 736(a)</strong> — payments NOT
            in exchange for property (ordinary treatment as distributive share or § 707(c) guaranteed
            payment). <strong>§ 736(b)(2) exception for SERVICE PARTNERSHIPS:</strong> general
            partner only — payments for unrealized receivables + unstated goodwill are § 736(a).
            <strong>§ 736(b)(3):</strong> capital-intensive partnerships — all property payments are
            § 736(b). <strong>Allocation crucial:</strong> § 736(a) provides PS deduction (or
            distributive share reduction) — § 736(b) does NOT. <strong>Partnership agreement
            controls</strong> allocation — court generally respects bona fide allocations.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s736.h2.inputs">Inputs</h2>
            <form id="s736-form" class="inline-form">
                <label><span data-i18n="view.s736.label.total">Total payment ($)</span>
                    <input type="number" step="10000" name="total_payment_amount" value="${state.total_payment_amount}"></label>
                <label><span data-i18n="view.s736.label.736a">§ 736(a) ordinary ($)</span>
                    <input type="number" step="10000" name="s736_a_ordinary_portion" value="${state.s736_a_ordinary_portion}"></label>
                <label><span data-i18n="view.s736.label.736b">§ 736(b) capital ($)</span>
                    <input type="number" step="10000" name="s736_b_capital_portion" value="${state.s736_b_capital_portion}"></label>
                <label><span data-i18n="view.s736.label.outside">Outside basis ($)</span>
                    <input type="number" step="10000" name="partner_outside_basis" value="${state.partner_outside_basis}"></label>
                <label><span data-i18n="view.s736.label.capital_acct">Capital account ($)</span>
                    <input type="number" step="10000" name="partner_capital_account" value="${state.partner_capital_account}"></label>
                <label><span data-i18n="view.s736.label.inside">Share of inside basis ($)</span>
                    <input type="number" step="10000" name="partner_share_of_inside_basis" value="${state.partner_share_of_inside_basis}"></label>
                <label><span data-i18n="view.s736.label.retiring">Retiring?</span>
                    <input type="checkbox" name="is_retiring" ${state.is_retiring ? 'checked' : ''}></label>
                <label><span data-i18n="view.s736.label.deceased">Deceased?</span>
                    <input type="checkbox" name="is_deceased" ${state.is_deceased ? 'checked' : ''}></label>
                <label><span data-i18n="view.s736.label.property_pay">Property payments ($)</span>
                    <input type="number" step="10000" name="s736_b_property_payments_partnership" value="${state.s736_b_property_payments_partnership}"></label>
                <label><span data-i18n="view.s736.label.unrealized">Unrealized receivables ($)</span>
                    <input type="number" step="10000" name="s736_b_unrealized_receivables" value="${state.s736_b_unrealized_receivables}"></label>
                <label><span data-i18n="view.s736.label.inventory">Substantially appreciated inventory ($)</span>
                    <input type="number" step="10000" name="s736_b_substantially_appreciated_inventory" value="${state.s736_b_substantially_appreciated_inventory}"></label>
                <label><span data-i18n="view.s736.label.goodwill_share">§ 736(a) goodwill share ($)</span>
                    <input type="number" step="10000" name="s736_a_goodwill_share" value="${state.s736_a_goodwill_share}"></label>
                <label><span data-i18n="view.s736.label.unstated_gw">Unstated goodwill ($)</span>
                    <input type="number" step="10000" name="s736_a_2_b_unstated_goodwill" value="${state.s736_a_2_b_unstated_goodwill}"></label>
                <label><span data-i18n="view.s736.label.goodwill_if">Goodwill (if provided in agmt) ($)</span>
                    <input type="number" step="10000" name="s736_b_goodwill_if_provided" value="${state.s736_b_goodwill_if_provided}"></label>
                <label><span data-i18n="view.s736.label.gp">General partner?</span>
                    <input type="checkbox" name="is_general_partner" ${state.is_general_partner ? 'checked' : ''}></label>
                <label><span data-i18n="view.s736.label.capital_ps">Capital-intensive PS?</span>
                    <input type="checkbox" name="is_capital_partnership" ${state.is_capital_partnership ? 'checked' : ''}></label>
                <label><span data-i18n="view.s736.label.service_ps">Service PS?</span>
                    <input type="checkbox" name="is_service_partnership" ${state.is_service_partnership ? 'checked' : ''}></label>
                <label><span data-i18n="view.s736.label.b2_ur">§ 736(b)(2) UR amount ($)</span>
                    <input type="number" step="10000" name="s736_b_2_unrealized_receivables_amount" value="${state.s736_b_2_unrealized_receivables_amount}"></label>
                <label><span data-i18n="view.s736.label.b2_inv">§ 736(b)(2) inventory ($)</span>
                    <input type="number" step="10000" name="s736_b_2_inventory_amount" value="${state.s736_b_2_inventory_amount}"></label>
                <label><span data-i18n="view.s736.label.excess">§ 736(c) excess ($)</span>
                    <input type="number" step="10000" name="s736_c_payments_excess" value="${state.s736_c_payments_excess}"></label>
                <label><span data-i18n="view.s736.label.year">Year</span>
                    <input type="number" step="1" name="payments_year" value="${state.payments_year}"></label>
                <label><span data-i18n="view.s736.label.installments">Installments?</span>
                    <input type="checkbox" name="payment_made_in_installments" ${state.payment_made_in_installments ? 'checked' : ''}></label>
                <label><span data-i18n="view.s736.label.installment_periods">Installment periods</span>
                    <input type="number" step="1" name="installment_periods" value="${state.installment_periods}"></label>
                <label><span data-i18n="view.s736.label.treats_ds">§ 736(a) as distributive share?</span>
                    <input type="checkbox" name="s736_a_treats_distributive_share" ${state.s736_a_treats_distributive_share ? 'checked' : ''}></label>
                <label><span data-i18n="view.s736.label.treats_gp">§ 736(a) as guaranteed payment?</span>
                    <input type="checkbox" name="s736_a_treats_guaranteed_payment" ${state.s736_a_treats_guaranteed_payment ? 'checked' : ''}></label>
                <label><span data-i18n="view.s736.label.s453">§ 453 installment method?</span>
                    <input type="checkbox" name="s453_installment_method" ${state.s453_installment_method ? 'checked' : ''}></label>
                <label><span data-i18n="view.s736.label.aggregate">Aggregate method § 736(b)?</span>
                    <input type="checkbox" name="aggregate_method_s736b" ${state.aggregate_method_s736b ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s736.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s736-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s736.h2.bifurcation">§ 736 bifurcation</h2>
            <ol class="muted small">
                <li data-i18n="view.s736.bif.s736b_property">§ 736(b) — payments FOR partnership property: capital treatment under § 731/§ 741</li>
                <li data-i18n="view.s736.bif.s736a_not_property">§ 736(a) — payments NOT for property: ordinary treatment</li>
                <li data-i18n="view.s736.bif.s736a_categories">§ 736(a) further bifurcates:
                    <ul>
                        <li data-i18n="view.s736.bif.s736a_1">§ 736(a)(1): based on PS INCOME = distributive share (reduces other partners' share)</li>
                        <li data-i18n="view.s736.bif.s736a_2">§ 736(a)(2): fixed amount = § 707(c) guaranteed payment (PS deducts)</li>
                    </ul>
                </li>
                <li data-i18n="view.s736.bif.partnership_agreement">Partnership agreement controls — taxpayer can specify allocation</li>
                <li data-i18n="view.s736.bif.economic_substance">Must have economic substance — court will not respect blatant tax-motivated allocation</li>
                <li data-i18n="view.s736.bif.bona_fide">Bona fide allocation: facts &amp; circumstances + reasonable</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s736.h2.s736b">§ 736(b) capital treatment</h2>
            <ul class="muted small">
                <li data-i18n="view.s736.b.s731_731c">Treated as distribution under § 731/§ 731(c)</li>
                <li data-i18n="view.s736.b.gain_on_excess">Gain = LESSER of (payment) OR (payment over outside basis)</li>
                <li data-i18n="view.s736.b.loss_liquidation">Loss only if liquidating + cash/receivables/inventory only</li>
                <li data-i18n="view.s736.b.installment">Installment method: § 453 capital gain spread</li>
                <li data-i18n="view.s736.b.s731_c_securities">§ 731(c) marketable securities = money (FMV)</li>
                <li data-i18n="view.s736.b.s751_b_hot">§ 751(b) hot assets — disproportionate ordinary recognition</li>
                <li data-i18n="view.s736.b.no_ps_deduction">PS does NOT deduct § 736(b) payments — capital character</li>
                <li data-i18n="view.s736.b.s754_step_up">§ 754 election: § 734(b) inside basis adjustment available</li>
                <li data-i18n="view.s736.b.s732_basis">Basis of distributed property: § 732</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s736.h2.s736a">§ 736(a) ordinary treatment</h2>
            <ul class="muted small">
                <li data-i18n="view.s736.a.ordinary">ORDINARY income to retiring/deceased partner</li>
                <li data-i18n="view.s736.a.distributive">If varies with PS income → distributive share (reduces other partners')</li>
                <li data-i18n="view.s736.a.guaranteed">If fixed amount → § 707(c) guaranteed payment (PS deducts ordinary)</li>
                <li data-i18n="view.s736.a.se_tax">Subject to SE tax (if partner was general partner)</li>
                <li data-i18n="view.s736.a.no_capital_gain">NOT capital gain — even if would be capital if § 736(b)</li>
                <li data-i18n="view.s736.a.installment">Installment payments: PS deducts in year accrued/paid</li>
                <li data-i18n="view.s736.a.spread_dedn">PS gets ordinary deduction spread across payment periods</li>
                <li data-i18n="view.s736.a.partner_recognizes">Partner recognizes ordinary as RECEIVED (cash method) — installment method does NOT apply</li>
                <li data-i18n="view.s736.a.s451">§ 451 timing — accrual at fixed/determinable + economic performance</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s736.h2.b2_exception">§ 736(b)(2) service partnership exception</h2>
            <ul class="muted small">
                <li data-i18n="view.s736.b2.purpose">Applies to GENERAL PARTNER in SERVICE PARTNERSHIP (law, accounting, etc.)</li>
                <li data-i18n="view.s736.b2.scope">Treats payments for unrealized receivables + unstated goodwill as § 736(a)</li>
                <li data-i18n="view.s736.b2.unrealized">Unrealized receivables: accrued but unbilled fees</li>
                <li data-i18n="view.s736.b2.goodwill">"Unstated goodwill" — only counted as § 736(a) if NOT specified in PS agreement</li>
                <li data-i18n="view.s736.b2.if_specified">If PS agreement DOES specify goodwill payments: treated as § 736(b)</li>
                <li data-i18n="view.s736.b2.limited_partner">Limited partner NOT subject to § 736(b)(2) — all property payments are § 736(b)</li>
                <li data-i18n="view.s736.b2.s736_b_3">§ 736(b)(3) — capital-intensive PS — all property payments § 736(b)</li>
                <li data-i18n="view.s736.b2.s197">§ 197 intangibles — generally retiring partner's share of acquired goodwill</li>
                <li data-i18n="view.s736.b2.law_firm">Law/CPA firm example: retiring partner gets ordinary on uncollected fees + unstated goodwill</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s736.h2.allocation_planning">Allocation planning</h2>
            <ul class="muted small">
                <li data-i18n="view.s736.alloc.optimization">Optimize: balance partner's character + PS deduction</li>
                <li data-i18n="view.s736.alloc.partner_low_bracket">Partner low tax bracket: prefer § 736(a) (ordinary at low rate + PS deducts)</li>
                <li data-i18n="view.s736.alloc.partner_high_bracket">Partner high bracket + remaining partners low: prefer § 736(b) (capital at lower partner rate)</li>
                <li data-i18n="view.s736.alloc.cap_intensive_default">Capital partnership: all § 736(b) by default — limited flexibility</li>
                <li data-i18n="view.s736.alloc.service_default">Service partnership: § 736(b)(2) restricts — unrealized receivables ALWAYS § 736(a)</li>
                <li data-i18n="view.s736.alloc.deal_term">Deal term: allocation in retirement agreement</li>
                <li data-i18n="view.s736.alloc.s754">§ 754 election: combined with § 736 allocation</li>
                <li data-i18n="view.s736.alloc.installment">Installment notes: § 453 capital gain (§ 736(b)) + PS deduction over period (§ 736(a))</li>
            </ul>
        </div>
    `;
    document.getElementById('s736-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.total_payment_amount = Number(fd.get('total_payment_amount')) || 0;
        state.s736_a_ordinary_portion = Number(fd.get('s736_a_ordinary_portion')) || 0;
        state.s736_b_capital_portion = Number(fd.get('s736_b_capital_portion')) || 0;
        state.partner_outside_basis = Number(fd.get('partner_outside_basis')) || 0;
        state.partner_capital_account = Number(fd.get('partner_capital_account')) || 0;
        state.partner_share_of_inside_basis = Number(fd.get('partner_share_of_inside_basis')) || 0;
        state.is_retiring = !!fd.get('is_retiring');
        state.is_deceased = !!fd.get('is_deceased');
        state.s736_b_property_payments_partnership = Number(fd.get('s736_b_property_payments_partnership')) || 0;
        state.s736_b_unrealized_receivables = Number(fd.get('s736_b_unrealized_receivables')) || 0;
        state.s736_b_substantially_appreciated_inventory = Number(fd.get('s736_b_substantially_appreciated_inventory')) || 0;
        state.s736_a_goodwill_share = Number(fd.get('s736_a_goodwill_share')) || 0;
        state.s736_a_2_b_unstated_goodwill = Number(fd.get('s736_a_2_b_unstated_goodwill')) || 0;
        state.s736_b_goodwill_if_provided = Number(fd.get('s736_b_goodwill_if_provided')) || 0;
        state.is_general_partner = !!fd.get('is_general_partner');
        state.is_capital_partnership = !!fd.get('is_capital_partnership');
        state.is_service_partnership = !!fd.get('is_service_partnership');
        state.s736_b_2_unrealized_receivables_amount = Number(fd.get('s736_b_2_unrealized_receivables_amount')) || 0;
        state.s736_b_2_inventory_amount = Number(fd.get('s736_b_2_inventory_amount')) || 0;
        state.s736_c_payments_excess = Number(fd.get('s736_c_payments_excess')) || 0;
        state.payments_year = Number(fd.get('payments_year')) || 0;
        state.payment_made_in_installments = !!fd.get('payment_made_in_installments');
        state.installment_periods = Number(fd.get('installment_periods')) || 0;
        state.s736_a_treats_distributive_share = !!fd.get('s736_a_treats_distributive_share');
        state.s736_a_treats_guaranteed_payment = !!fd.get('s736_a_treats_guaranteed_payment');
        state.s453_installment_method = !!fd.get('s453_installment_method');
        state.aggregate_method_s736b = !!fd.get('aggregate_method_s736b');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s736-output');
    if (!el) return;
    const capital_gain = Math.max(0, state.s736_b_capital_portion - state.partner_outside_basis);
    const ordinary = state.s736_a_ordinary_portion;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s736.h2.result">§ 736 allocation</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s736.card.total">Total payment</div><div class="value">$${state.total_payment_amount.toLocaleString()}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s736.card.capital">§ 736(b) capital</div><div class="value">$${state.s736_b_capital_portion.toLocaleString()}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.s736.card.ordinary">§ 736(a) ordinary</div><div class="value">$${ordinary.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s736.card.gain">Capital gain (§ 731)</div><div class="value">$${capital_gain.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s736.card.ps_dedn">PS § 736(a) deduction</div><div class="value">$${ordinary.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
