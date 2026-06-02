// IRC § 707 — Partner-Partnership Transactions.
// § 707(a) — transactions between partner + partnership: treated as occurring with non-partner.
// § 707(b) — losses + gains between partners + controlled entities: limited / recharacterized.
// § 707(c) — guaranteed payments to partner: treated like compensation (deduction at PS level).
// § 707(a)(2)(B) — disguised sales: 2-year contribution + distribution presumption.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    transaction_type: 'sale',
    is_arms_length: false,
    s707_a_transaction: false,
    s707_a_2_disguised_sale: false,
    contribution_amount: 0,
    distribution_amount: 0,
    contribution_date: '',
    distribution_date: '',
    days_between: 0,
    is_within_2_years: false,
    is_facts_circumstances: false,
    s707_b_related_party: false,
    s707_b_loss_disallowance: 0,
    s707_b_gain_ordinary: 0,
    is_50pct_or_more_partner: false,
    partner_ownership_pct: 0,
    has_constructive_ownership: false,
    s707_c_guaranteed_payment: 0,
    is_for_services: false,
    is_for_capital: false,
    is_priority_distribution: false,
    s707_a_2_a_partnership_loss: false,
    s707_a_2_a_partnership_service: false,
    s707_a_recharacterization: false,
    rev_proc_2017_31_safe_harbor: false,
    operating_distribution_safe: false,
    reimbursement_safe_harbor: false,
    preferred_return_amount: 0,
    s707_c_se_tax: false,
    deferred_compensation_arrangement: false,
};

export async function renderSection707(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s707.h1.title">// § 707 PARTNER-PARTNERSHIP TRANSACTIONS</span></h1>
        <p class="muted small" data-i18n="view.s707.hint.intro">
            <strong>§ 707(a)(1)</strong> — partner transacts with partnership as if NOT a partner
            (sale, service, lease). <strong>§ 707(a)(2)(B) disguised sale:</strong> contribution +
            distribution within 2 YEARS = REBUTTABLE PRESUMPTION of sale. <strong>§ 707(b)(1)</strong>
            — loss on sale between partner + partnership controlled (50%+) by partner: LOSS DISALLOWED.
            <strong>§ 707(b)(2)</strong> — gain between commonly-controlled partnerships: ORDINARY
            character. <strong>§ 707(c) guaranteed payments:</strong> fixed payments determined without
            regard to partnership income, treated like § 162 compensation — DEDUCTIBLE by PS, subject
            to SE tax for recipient. <strong>§ 707(a)(2)(A)</strong> — partnership allocation +
            distribution may be recharacterized as compensation if not bona fide. <strong>Rev. Proc.
            2017-31</strong> safe harbors for preferred return + operating distributions.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s707.h2.inputs">Inputs</h2>
            <form id="s707-form" class="inline-form">
                <label><span data-i18n="view.s707.label.type">Transaction type</span>
                    <select name="transaction_type">
                        <option value="sale" ${state.transaction_type === 'sale' ? 'selected' : ''}>Sale partner → PS</option>
                        <option value="loan" ${state.transaction_type === 'loan' ? 'selected' : ''}>Loan</option>
                        <option value="lease" ${state.transaction_type === 'lease' ? 'selected' : ''}>Lease</option>
                        <option value="services" ${state.transaction_type === 'services' ? 'selected' : ''}>Services</option>
                        <option value="guaranteed_payment" ${state.transaction_type === 'guaranteed_payment' ? 'selected' : ''}>Guaranteed payment</option>
                        <option value="disguised_sale" ${state.transaction_type === 'disguised_sale' ? 'selected' : ''}>Disguised sale</option>
                        <option value="contribution_distribution" ${state.transaction_type === 'contribution_distribution' ? 'selected' : ''}>Contribution + distribution</option>
                    </select>
                </label>
                <label><span data-i18n="view.s707.label.arms_length">Arm's length?</span>
                    <input type="checkbox" name="is_arms_length" ${state.is_arms_length ? 'checked' : ''}></label>
                <label><span data-i18n="view.s707.label.s707a">§ 707(a) transaction?</span>
                    <input type="checkbox" name="s707_a_transaction" ${state.s707_a_transaction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s707.label.s707a2b">§ 707(a)(2)(B) disguised?</span>
                    <input type="checkbox" name="s707_a_2_disguised_sale" ${state.s707_a_2_disguised_sale ? 'checked' : ''}></label>
                <label><span data-i18n="view.s707.label.contribution">Contribution ($)</span>
                    <input type="number" step="10000" name="contribution_amount" value="${state.contribution_amount}"></label>
                <label><span data-i18n="view.s707.label.distribution">Distribution ($)</span>
                    <input type="number" step="10000" name="distribution_amount" value="${state.distribution_amount}"></label>
                <label><span data-i18n="view.s707.label.contrib_date">Contribution date</span>
                    <input type="date" name="contribution_date" value="${state.contribution_date}"></label>
                <label><span data-i18n="view.s707.label.dist_date">Distribution date</span>
                    <input type="date" name="distribution_date" value="${state.distribution_date}"></label>
                <label><span data-i18n="view.s707.label.days">Days between</span>
                    <input type="number" step="1" name="days_between" value="${state.days_between}"></label>
                <label><span data-i18n="view.s707.label.within_2yr">Within 2 yrs?</span>
                    <input type="checkbox" name="is_within_2_years" ${state.is_within_2_years ? 'checked' : ''}></label>
                <label><span data-i18n="view.s707.label.facts">Facts &amp; circumstances?</span>
                    <input type="checkbox" name="is_facts_circumstances" ${state.is_facts_circumstances ? 'checked' : ''}></label>
                <label><span data-i18n="view.s707.label.related">Related party?</span>
                    <input type="checkbox" name="s707_b_related_party" ${state.s707_b_related_party ? 'checked' : ''}></label>
                <label><span data-i18n="view.s707.label.loss_disallow">§ 707(b)(1) loss ($)</span>
                    <input type="number" step="1000" name="s707_b_loss_disallowance" value="${state.s707_b_loss_disallowance}"></label>
                <label><span data-i18n="view.s707.label.gain_ord">§ 707(b)(2) gain ($)</span>
                    <input type="number" step="1000" name="s707_b_gain_ordinary" value="${state.s707_b_gain_ordinary}"></label>
                <label><span data-i18n="view.s707.label.50pct">50%+ partner?</span>
                    <input type="checkbox" name="is_50pct_or_more_partner" ${state.is_50pct_or_more_partner ? 'checked' : ''}></label>
                <label><span data-i18n="view.s707.label.pct">Partner ownership %</span>
                    <input type="number" step="0.1" name="partner_ownership_pct" value="${state.partner_ownership_pct}"></label>
                <label><span data-i18n="view.s707.label.constructive">Constructive ownership?</span>
                    <input type="checkbox" name="has_constructive_ownership" ${state.has_constructive_ownership ? 'checked' : ''}></label>
                <label><span data-i18n="view.s707.label.guaranteed">§ 707(c) guaranteed pmt ($)</span>
                    <input type="number" step="10000" name="s707_c_guaranteed_payment" value="${state.s707_c_guaranteed_payment}"></label>
                <label><span data-i18n="view.s707.label.for_services">For services?</span>
                    <input type="checkbox" name="is_for_services" ${state.is_for_services ? 'checked' : ''}></label>
                <label><span data-i18n="view.s707.label.for_capital">For capital?</span>
                    <input type="checkbox" name="is_for_capital" ${state.is_for_capital ? 'checked' : ''}></label>
                <label><span data-i18n="view.s707.label.priority">Priority distribution?</span>
                    <input type="checkbox" name="is_priority_distribution" ${state.is_priority_distribution ? 'checked' : ''}></label>
                <label><span data-i18n="view.s707.label.s707a2a_loss">§ 707(a)(2)(A) loss?</span>
                    <input type="checkbox" name="s707_a_2_a_partnership_loss" ${state.s707_a_2_a_partnership_loss ? 'checked' : ''}></label>
                <label><span data-i18n="view.s707.label.s707a2a_serv">§ 707(a)(2)(A) services?</span>
                    <input type="checkbox" name="s707_a_2_a_partnership_service" ${state.s707_a_2_a_partnership_service ? 'checked' : ''}></label>
                <label><span data-i18n="view.s707.label.recharacterize">Recharacterization?</span>
                    <input type="checkbox" name="s707_a_recharacterization" ${state.s707_a_recharacterization ? 'checked' : ''}></label>
                <label><span data-i18n="view.s707.label.rp2017">Rev Proc 2017-31 safe?</span>
                    <input type="checkbox" name="rev_proc_2017_31_safe_harbor" ${state.rev_proc_2017_31_safe_harbor ? 'checked' : ''}></label>
                <label><span data-i18n="view.s707.label.op_safe">Operating dist safe?</span>
                    <input type="checkbox" name="operating_distribution_safe" ${state.operating_distribution_safe ? 'checked' : ''}></label>
                <label><span data-i18n="view.s707.label.reimburse">Reimbursement safe?</span>
                    <input type="checkbox" name="reimbursement_safe_harbor" ${state.reimbursement_safe_harbor ? 'checked' : ''}></label>
                <label><span data-i18n="view.s707.label.preferred">Preferred return ($)</span>
                    <input type="number" step="10000" name="preferred_return_amount" value="${state.preferred_return_amount}"></label>
                <label><span data-i18n="view.s707.label.se_tax">§ 707(c) SE tax applies?</span>
                    <input type="checkbox" name="s707_c_se_tax" ${state.s707_c_se_tax ? 'checked' : ''}></label>
                <label><span data-i18n="view.s707.label.deferred">Deferred comp?</span>
                    <input type="checkbox" name="deferred_compensation_arrangement" ${state.deferred_compensation_arrangement ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s707.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s707-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s707.h2.s707a">§ 707(a) — partner as outsider</h2>
            <ul class="muted small">
                <li data-i18n="view.s707.a.purpose">Partner transacts with partnership AS IF NOT a partner</li>
                <li data-i18n="view.s707.a.sale">Sale: ordinary income/loss + § 1031/§ 1239 may apply</li>
                <li data-i18n="view.s707.a.loan">Loan: interest income + § 163(j) limits</li>
                <li data-i18n="view.s707.a.lease">Lease: rent income/deduction</li>
                <li data-i18n="view.s707.a.services">Services: compensation income to partner, deduction by PS</li>
                <li data-i18n="view.s707.a.s707_a_2_a">§ 707(a)(2)(A) — distinction between bona fide allocation/distribution + § 707(a) transaction</li>
                <li data-i18n="view.s707.a.s707_a_2_b">§ 707(a)(2)(B) — disguised sale anti-abuse rule</li>
                <li data-i18n="view.s707.a.s707_a_2_c">§ 707(a)(2)(C) — disguised service payment + property transfer</li>
                <li data-i18n="view.s707.a.related_party">Related party transactions: arm's length scrutiny + § 482-like</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s707.h2.disguised">§ 707(a)(2)(B) disguised sale</h2>
            <ol class="muted small">
                <li data-i18n="view.s707.dis.purpose">Anti-abuse: cannot avoid sale treatment via contribution + distribution</li>
                <li data-i18n="view.s707.dis.2year">2-YEAR presumption: contribution + distribution within 2 years = sale</li>
                <li data-i18n="view.s707.dis.rebuttable">PRESUMPTION rebuttable by facts &amp; circumstances</li>
                <li data-i18n="view.s707.dis.factors">Factors: timing, conditioning, certainty, entrepreneurial risk, related parties</li>
                <li data-i18n="view.s707.dis.outside_2year">Outside 2 years: presumption REVERSED — not disguised sale unless facts indicate</li>
                <li data-i18n="view.s707.dis.assumed_liability">Assumed liability counted as part of "distribution" (Reg § 1.707-5)</li>
                <li data-i18n="view.s707.dis.qualified_liability">"Qualified liability" excluded from sale treatment (Reg § 1.707-5(a)(6))</li>
                <li data-i18n="view.s707.dis.safe_harbors">Safe harbors: operating distributions, reimbursements (Rev. Proc. 2017-31)</li>
                <li data-i18n="view.s707.dis.f8949">Form 8949 reports disguised sale element</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s707.h2.s707b">§ 707(b) related parties</h2>
            <ul class="muted small">
                <li data-i18n="view.s707.b.related">"Controlling" = more than 50% interest by partner</li>
                <li data-i18n="view.s707.b.loss">§ 707(b)(1) — LOSS on sale partner ↔ controlled PS: DISALLOWED entirely</li>
                <li data-i18n="view.s707.b.gain">§ 707(b)(2) — GAIN on sale between commonly-controlled PSs: ORDINARY (recharacterization)</li>
                <li data-i18n="view.s707.b.constructive">§ 267(c) constructive ownership rules apply</li>
                <li data-i18n="view.s707.b.s267">Compare § 267: similar disallowance for related individuals + entities</li>
                <li data-i18n="view.s707.b.disallow_carryover">Disallowed loss: lost to seller; basis NOT increased to buyer either</li>
                <li data-i18n="view.s707.b.s267_d">§ 267(d) subsequent gain: buyer offsets prior disallowed loss when buyer sells</li>
                <li data-i18n="view.s707.b.brother_sister">"Brother-sister" entities: both controlled by same partner = § 707(b)(2) applies</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s707.h2.s707c">§ 707(c) guaranteed payments</h2>
            <ul class="muted small">
                <li data-i18n="view.s707.c.fixed">Fixed payment determined WITHOUT regard to partnership income</li>
                <li data-i18n="view.s707.c.like_compensation">Treated as § 162 compensation (deductible by PS, ordinary to partner)</li>
                <li data-i18n="view.s707.c.timing">Timing: deducted by PS when accrued/paid (Rev. Rul. 2007-40)</li>
                <li data-i18n="view.s707.c.se_tax">Subject to SE tax (§ 1402(a)(13) general partner exception does NOT apply)</li>
                <li data-i18n="view.s707.c.no_partner_status">Does NOT confer partner status if not otherwise partner</li>
                <li data-i18n="view.s707.c.priority_distribution">Distinguish from PRIORITY DISTRIBUTION (varies with PS income) — § 704(b) allocation</li>
                <li data-i18n="view.s707.c.preferred_return">Preferred return on capital: may be guaranteed payment OR § 704(b) allocation</li>
                <li data-i18n="view.s707.c.documentation">Documentation in partnership agreement key for characterization</li>
                <li data-i18n="view.s707.c.k1_box_4">K-1 Box 4a-c reports guaranteed payments</li>
                <li data-i18n="view.s707.c.s162_ordinary">PS deducts as § 162 ordinary expense unless capital expenditure</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s707.h2.safe_harbors">Rev. Proc. 2017-31 + safe harbors</h2>
            <ul class="muted small">
                <li data-i18n="view.s707.safe.operating_dist">Operating distributions: not deemed sale if pursuant to bona fide PS agreement</li>
                <li data-i18n="view.s707.safe.reimbursement">Reimbursement of preformation costs: within 2 years OK if &lt; 20% FMV at contribution</li>
                <li data-i18n="view.s707.safe.preferred_return">Preferred return: safe harbor for 150% of AFR — not deemed disguised sale</li>
                <li data-i18n="view.s707.safe.s704_c_protection">§ 704(c) "anti-mixing bowl" — protects book/tax allocations</li>
                <li data-i18n="view.s707.safe.cash_flow">Cash flow distributions: based on partnership income — safe</li>
                <li data-i18n="view.s707.safe.qualified_liab">Qualified liabilities (Reg § 1.707-5(a)(6)) — not counted as distribution</li>
                <li data-i18n="view.s707.safe.s737_seven_year">§ 737 7-yr lookback parallel for built-in gain protection</li>
                <li data-i18n="view.s707.safe.s704_c_7yr">§ 704(c)(1)(B) 7-year built-in gain recognition on contributed property</li>
            </ul>
        </div>
    `;
    document.getElementById('s707-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.transaction_type = fd.get('transaction_type');
        state.is_arms_length = !!fd.get('is_arms_length');
        state.s707_a_transaction = !!fd.get('s707_a_transaction');
        state.s707_a_2_disguised_sale = !!fd.get('s707_a_2_disguised_sale');
        state.contribution_amount = Number(fd.get('contribution_amount')) || 0;
        state.distribution_amount = Number(fd.get('distribution_amount')) || 0;
        state.contribution_date = fd.get('contribution_date') || '';
        state.distribution_date = fd.get('distribution_date') || '';
        state.days_between = Number(fd.get('days_between')) || 0;
        state.is_within_2_years = !!fd.get('is_within_2_years');
        state.is_facts_circumstances = !!fd.get('is_facts_circumstances');
        state.s707_b_related_party = !!fd.get('s707_b_related_party');
        state.s707_b_loss_disallowance = Number(fd.get('s707_b_loss_disallowance')) || 0;
        state.s707_b_gain_ordinary = Number(fd.get('s707_b_gain_ordinary')) || 0;
        state.is_50pct_or_more_partner = !!fd.get('is_50pct_or_more_partner');
        state.partner_ownership_pct = Number(fd.get('partner_ownership_pct')) || 0;
        state.has_constructive_ownership = !!fd.get('has_constructive_ownership');
        state.s707_c_guaranteed_payment = Number(fd.get('s707_c_guaranteed_payment')) || 0;
        state.is_for_services = !!fd.get('is_for_services');
        state.is_for_capital = !!fd.get('is_for_capital');
        state.is_priority_distribution = !!fd.get('is_priority_distribution');
        state.s707_a_2_a_partnership_loss = !!fd.get('s707_a_2_a_partnership_loss');
        state.s707_a_2_a_partnership_service = !!fd.get('s707_a_2_a_partnership_service');
        state.s707_a_recharacterization = !!fd.get('s707_a_recharacterization');
        state.rev_proc_2017_31_safe_harbor = !!fd.get('rev_proc_2017_31_safe_harbor');
        state.operating_distribution_safe = !!fd.get('operating_distribution_safe');
        state.reimbursement_safe_harbor = !!fd.get('reimbursement_safe_harbor');
        state.preferred_return_amount = Number(fd.get('preferred_return_amount')) || 0;
        state.s707_c_se_tax = !!fd.get('s707_c_se_tax');
        state.deferred_compensation_arrangement = !!fd.get('deferred_compensation_arrangement');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s707-output');
    if (!el) return;
    const presumption = state.is_within_2_years && state.contribution_amount > 0 && state.distribution_amount > 0;
    let disguised = presumption && !state.is_facts_circumstances;
    if (state.rev_proc_2017_31_safe_harbor) disguised = false;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s707.h2.result">§ 707 transaction analysis</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s707.card.type">Transaction</div><div class="value">${esc(state.transaction_type)}</div></div>
                <div class="card ${presumption ? 'warn' : 'pos'}"><div class="label" data-i18n="view.s707.card.presumption">2-yr presumption?</div><div class="value">${presumption ? 'YES' : 'NO'}</div></div>
                <div class="card ${disguised ? 'neg' : 'pos'}"><div class="label" data-i18n="view.s707.card.disguised">Disguised sale?</div><div class="value">${disguised ? 'YES' : 'NO'}</div></div>
                <div class="card ${state.s707_b_loss_disallowance > 0 ? 'neg' : 'pos'}"><div class="label" data-i18n="view.s707.card.loss">§ 707(b)(1) loss disallowed</div><div class="value">$${state.s707_b_loss_disallowance.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s707.card.guaranteed">§ 707(c) guaranteed pmt</div><div class="value">$${state.s707_c_guaranteed_payment.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
