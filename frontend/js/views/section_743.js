// IRC § 743 — Special Rules Where Section 754 Election or Substantial Built-in Loss.
// § 743(b) — basis adjustment to partnership assets upon transfer of partnership interest IF § 754 election OR substantial built-in loss.
// "Substantial built-in loss" = adjusted basis of partnership property > FMV by > $250,000.
// Adjusts INSIDE basis to reflect transferee's OUTSIDE basis paid.

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    transferor_outside_basis: 0,
    transferee_purchase_price: 0,
    partnership_inside_basis_share: 0,
    s743_b_adjustment: 0,
    is_s754_election: false,
    substantial_built_in_loss: false,
    built_in_loss_amount: 0,
    s754_election_year: 2024,
    transfer_date: '',
    transferred_pct: 0,
    s755_allocation_method: 'residual',
    capital_gain_property_share: 0,
    ordinary_income_property_share: 0,
    s751_hot_assets: 0,
    s751_a_collapsible_gain: 0,
    s743_d_substantial_loss_test: false,
    s743_e_securitization_partnership: false,
    s197_intangible_amount: 0,
    s754_revocation_pending: false,
    is_eligible_termination: false,
    s708_b_1_termination: false,
    s732_distributions: false,
    s734_b_distribution_adjustment: false,
    mandatory_s732_e_basis_reduction: false,
};

export async function renderSection743(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s743.h1.title">// § 743 PARTNERSHIP BASIS ADJUSTMENT</span></h1>
        <p class="muted small" data-i18n="view.s743.hint.intro">
            <strong>§ 743(b) basis adjustment</strong> aligns transferee's INSIDE basis in partnership
            property with OUTSIDE basis paid. <strong>Triggers:</strong> (1) § 754 election in effect,
            OR (2) "substantial built-in loss" (adjusted basis of PS property exceeds FMV by &gt; $250K
            — mandatory adjustment per § 743(d)). <strong>§ 755 allocation:</strong> first to ordinary
            income property (§ 1245 / § 751 hot assets), then to capital gain property by residual
            method (FMV vs basis). <strong>§ 754 election</strong> is permanent unless IRS consents to
            revocation. <strong>§ 743(e) securitization</strong> partnership: mandatory adjustment for
            built-in loss transfer regardless of § 754. <strong>Practical effect:</strong> step-up in
            depreciation deductions + reduced gain on subsequent partnership asset sale for
            transferee partner.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s743.h2.inputs">Inputs</h2>
            <form id="s743-form" class="inline-form">
                <label><span data-i18n="view.s743.label.transferor_basis">Transferor outside basis ($)</span>
                    <input type="number" step="10000" name="transferor_outside_basis" value="${state.transferor_outside_basis}"></label>
                <label><span data-i18n="view.s743.label.purchase_price">Transferee purchase price ($)</span>
                    <input type="number" step="10000" name="transferee_purchase_price" value="${state.transferee_purchase_price}"></label>
                <label><span data-i18n="view.s743.label.inside_share">Inside basis share ($)</span>
                    <input type="number" step="10000" name="partnership_inside_basis_share" value="${state.partnership_inside_basis_share}"></label>
                <label><span data-i18n="view.s743.label.s743b">§ 743(b) adjustment ($)</span>
                    <input type="number" step="10000" name="s743_b_adjustment" value="${state.s743_b_adjustment}"></label>
                <label><span data-i18n="view.s743.label.s754">§ 754 election?</span>
                    <input type="checkbox" name="is_s754_election" ${state.is_s754_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s743.label.sbil">Substantial built-in loss?</span>
                    <input type="checkbox" name="substantial_built_in_loss" ${state.substantial_built_in_loss ? 'checked' : ''}></label>
                <label><span data-i18n="view.s743.label.bil_amt">Built-in loss amount ($)</span>
                    <input type="number" step="10000" name="built_in_loss_amount" value="${state.built_in_loss_amount}"></label>
                <label><span data-i18n="view.s743.label.year">§ 754 election year</span>
                    <input type="number" step="1" name="s754_election_year" value="${state.s754_election_year}"></label>
                <label><span data-i18n="view.s743.label.date">Transfer date</span>
                    <input type="date" name="transfer_date" value="${state.transfer_date}"></label>
                <label><span data-i18n="view.s743.label.pct">Transferred %</span>
                    <input type="number" step="0.1" name="transferred_pct" value="${state.transferred_pct}"></label>
                <label><span data-i18n="view.s743.label.method">§ 755 method</span>
                    <select name="s755_allocation_method">
                        <option value="residual" ${state.s755_allocation_method === 'residual' ? 'selected' : ''}>Residual method</option>
                        <option value="anti-stuffing" ${state.s755_allocation_method === 'anti-stuffing' ? 'selected' : ''}>Anti-stuffing variation</option>
                    </select>
                </label>
                <label><span data-i18n="view.s743.label.cap_gain">Cap gain property share ($)</span>
                    <input type="number" step="10000" name="capital_gain_property_share" value="${state.capital_gain_property_share}"></label>
                <label><span data-i18n="view.s743.label.ord_inc">Ord inc property share ($)</span>
                    <input type="number" step="10000" name="ordinary_income_property_share" value="${state.ordinary_income_property_share}"></label>
                <label><span data-i18n="view.s743.label.hot">§ 751 hot assets ($)</span>
                    <input type="number" step="10000" name="s751_hot_assets" value="${state.s751_hot_assets}"></label>
                <label><span data-i18n="view.s743.label.collapsible">§ 751(a) collapsible gain ($)</span>
                    <input type="number" step="10000" name="s751_a_collapsible_gain" value="${state.s751_a_collapsible_gain}"></label>
                <label><span data-i18n="view.s743.label.test">§ 743(d) substantial loss test?</span>
                    <input type="checkbox" name="s743_d_substantial_loss_test" ${state.s743_d_substantial_loss_test ? 'checked' : ''}></label>
                <label><span data-i18n="view.s743.label.securitization">§ 743(e) securitization?</span>
                    <input type="checkbox" name="s743_e_securitization_partnership" ${state.s743_e_securitization_partnership ? 'checked' : ''}></label>
                <label><span data-i18n="view.s743.label.s197">§ 197 intangible ($)</span>
                    <input type="number" step="10000" name="s197_intangible_amount" value="${state.s197_intangible_amount}"></label>
                <label><span data-i18n="view.s743.label.revoke">§ 754 revocation pending?</span>
                    <input type="checkbox" name="s754_revocation_pending" ${state.s754_revocation_pending ? 'checked' : ''}></label>
                <label><span data-i18n="view.s743.label.eligible_term">Eligible termination?</span>
                    <input type="checkbox" name="is_eligible_termination" ${state.is_eligible_termination ? 'checked' : ''}></label>
                <label><span data-i18n="view.s743.label.s708">§ 708(b)(1) termination?</span>
                    <input type="checkbox" name="s708_b_1_termination" ${state.s708_b_1_termination ? 'checked' : ''}></label>
                <label><span data-i18n="view.s743.label.s732">§ 732 distributions?</span>
                    <input type="checkbox" name="s732_distributions" ${state.s732_distributions ? 'checked' : ''}></label>
                <label><span data-i18n="view.s743.label.s734b">§ 734(b) distribution adj?</span>
                    <input type="checkbox" name="s734_b_distribution_adjustment" ${state.s734_b_distribution_adjustment ? 'checked' : ''}></label>
                <label><span data-i18n="view.s743.label.mandatory_732e">§ 732(e) basis reduction?</span>
                    <input type="checkbox" name="mandatory_s732_e_basis_reduction" ${state.mandatory_s732_e_basis_reduction ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s743.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s743-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s743.h2.adjustment">§ 743(b) adjustment mechanics</h2>
            <ol class="muted small">
                <li data-i18n="view.s743.adj.formula">Adjustment = Transferee outside basis − Transferee's share of inside basis</li>
                <li data-i18n="view.s743.adj.positive">Positive adjustment (step-up): transferee paid premium for PS interest — depreciation step-up</li>
                <li data-i18n="view.s743.adj.negative">Negative adjustment (step-down): transferee paid discount — reduces depreciation</li>
                <li data-i18n="view.s743.adj.personal">Personal to transferee — does NOT affect other partners</li>
                <li data-i18n="view.s743.adj.s743b_inside_basis">"Transferee's share of inside basis" = transferee's share of common basis</li>
                <li data-i18n="view.s743.adj.future_dep">Future depreciation: transferee gets ADDITIONAL deduction or REDUCED deduction</li>
                <li data-i18n="view.s743.adj.future_gain">Future gain on sale: transferee recognizes LESS gain or MORE loss</li>
                <li data-i18n="view.s743.adj.allocated">Allocated under § 755 first to property class then within class</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s743.h2.allocation">§ 755 allocation</h2>
            <ol class="muted small">
                <li data-i18n="view.s743.alloc.classes">Two classes: (1) ordinary income property + (2) capital gain property</li>
                <li data-i18n="view.s743.alloc.between">Allocate § 743(b) adj between classes proportional to net unrealized gain/loss in each class</li>
                <li data-i18n="view.s743.alloc.within">Within class: residual method — FMV minus AB minus prior § 743(b) adjustments</li>
                <li data-i18n="view.s743.alloc.s197_residual">§ 197 intangibles residual — goodwill last after specifically identified assets</li>
                <li data-i18n="view.s743.alloc.s751_first">§ 751 hot assets receive allocation FIRST in ordinary class</li>
                <li data-i18n="view.s743.alloc.alternative">Alternative methods only with IRS consent (rare)</li>
                <li data-i18n="view.s743.alloc.no_below_zero">Adjustment may NOT reduce basis of property below $0 (Reg § 1.755-1(b)(3))</li>
                <li data-i18n="view.s743.alloc.anti_stuffing">Anti-stuffing rules: protect against artificially shifting basis</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s743.h2.s754_election">§ 754 election</h2>
            <ul class="muted small">
                <li data-i18n="view.s743.s754.timing">Made on partnership return for year of transfer / distribution</li>
                <li data-i18n="view.s743.s754.permanent">PERMANENT — binding for that year + all subsequent years</li>
                <li data-i18n="view.s743.s754.revocation">Revocation requires IRS consent (limited grounds in Reg § 1.754-1(c))</li>
                <li data-i18n="view.s743.s754.s734_b">Also triggers § 734(b) for distributions (separate from § 743)</li>
                <li data-i18n="view.s743.s754.accounting">Partnership must track basis adjustments per partner ($K admin burden)</li>
                <li data-i18n="view.s743.s754.partners_taxable">Election generally taxable for incoming partner (step-up benefit)</li>
                <li data-i18n="view.s743.s754.no_revisit">Cannot re-elect — once revoked, requires another transfer to make new election</li>
                <li data-i18n="view.s743.s754.tax_matters">Tax Matters Partner / partnership rep must sign</li>
                <li data-i18n="view.s743.s754.k1">Form K-1 must reflect partner's § 743(b) adjustment annually</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s743.h2.s743_d">§ 743(d) substantial built-in loss (mandatory)</h2>
            <ul class="muted small">
                <li data-i18n="view.s743.sbil.250k">"Substantial built-in loss" = adjusted basis exceeds FMV by &gt; $250,000</li>
                <li data-i18n="view.s743.sbil.tested_at_transfer">Tested AT TIME of transfer</li>
                <li data-i18n="view.s743.sbil.mandatory">Mandatory adjustment EVEN WITHOUT § 754 election</li>
                <li data-i18n="view.s743.sbil.tcja">TCJA 2017 added: pre-TCJA only required § 754 election</li>
                <li data-i18n="view.s743.sbil.transferee_loss">Or transferee would be allocated loss > $250K had PS sold all assets</li>
                <li data-i18n="view.s743.sbil.s743_d_3">§ 743(d)(3): each partner's loss allocation tested separately for transferee</li>
                <li data-i18n="view.s743.sbil.s743_e_security">§ 743(e) securitization partnership: ALWAYS mandatory adjustment</li>
                <li data-i18n="view.s743.sbil.s743_d_3_b">Substantial built-in loss exists even if § 754 election NOT in place</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s743.h2.related">Related provisions</h2>
            <ul class="muted small">
                <li data-i18n="view.s743.rel.s734">§ 734(b) — basis adjustment on distribution (analogous to § 743(b) for distributions)</li>
                <li data-i18n="view.s743.rel.s732">§ 732 — basis of distributed property to recipient partner</li>
                <li data-i18n="view.s743.rel.s732_e">§ 732(e) — mandatory basis reduction for built-in loss property distribution</li>
                <li data-i18n="view.s743.rel.s731">§ 731 — recognition of gain on distribution exceeding outside basis</li>
                <li data-i18n="view.s743.rel.s751">§ 751 — unrealized receivables + inventory items (hot assets) — ordinary income classification</li>
                <li data-i18n="view.s743.rel.s197">§ 197 — 15-year amortization of acquired intangibles (frequent target of § 743(b))</li>
                <li data-i18n="view.s743.rel.s1245">§ 1245 — depreciation recapture on subsequent sale</li>
                <li data-i18n="view.s743.rel.s708">§ 708 — partnership termination (no longer triggers technical termination post-TCJA)</li>
                <li data-i18n="view.s743.rel.s7704">§ 7704 PTP — special rules for publicly traded partnerships</li>
            </ul>
        </div>
    `;
    document.getElementById('s743-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.transferor_outside_basis = Number(fd.get('transferor_outside_basis')) || 0;
        state.transferee_purchase_price = Number(fd.get('transferee_purchase_price')) || 0;
        state.partnership_inside_basis_share = Number(fd.get('partnership_inside_basis_share')) || 0;
        state.s743_b_adjustment = Number(fd.get('s743_b_adjustment')) || 0;
        state.is_s754_election = !!fd.get('is_s754_election');
        state.substantial_built_in_loss = !!fd.get('substantial_built_in_loss');
        state.built_in_loss_amount = Number(fd.get('built_in_loss_amount')) || 0;
        state.s754_election_year = Number(fd.get('s754_election_year')) || 0;
        state.transfer_date = fd.get('transfer_date') || '';
        state.transferred_pct = Number(fd.get('transferred_pct')) || 0;
        state.s755_allocation_method = fd.get('s755_allocation_method');
        state.capital_gain_property_share = Number(fd.get('capital_gain_property_share')) || 0;
        state.ordinary_income_property_share = Number(fd.get('ordinary_income_property_share')) || 0;
        state.s751_hot_assets = Number(fd.get('s751_hot_assets')) || 0;
        state.s751_a_collapsible_gain = Number(fd.get('s751_a_collapsible_gain')) || 0;
        state.s743_d_substantial_loss_test = !!fd.get('s743_d_substantial_loss_test');
        state.s743_e_securitization_partnership = !!fd.get('s743_e_securitization_partnership');
        state.s197_intangible_amount = Number(fd.get('s197_intangible_amount')) || 0;
        state.s754_revocation_pending = !!fd.get('s754_revocation_pending');
        state.is_eligible_termination = !!fd.get('is_eligible_termination');
        state.s708_b_1_termination = !!fd.get('s708_b_1_termination');
        state.s732_distributions = !!fd.get('s732_distributions');
        state.s734_b_distribution_adjustment = !!fd.get('s734_b_distribution_adjustment');
        state.mandatory_s732_e_basis_reduction = !!fd.get('mandatory_s732_e_basis_reduction');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s743-output');
    if (!el) return;
    const computed_adjustment = state.transferee_purchase_price - state.partnership_inside_basis_share;
    const triggered = state.is_s754_election || state.substantial_built_in_loss || state.s743_e_securitization_partnership;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s743.h2.result">§ 743(b) adjustment</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s743.card.price">Transferee paid</div><div class="value">$${state.transferee_purchase_price.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s743.card.share">Inside basis share</div><div class="value">$${state.partnership_inside_basis_share.toLocaleString()}</div></div>
                <div class="card ${computed_adjustment > 0 ? 'pos' : (computed_adjustment < 0 ? 'neg' : '')}"><div class="label" data-i18n="view.s743.card.adj">§ 743(b) adjustment</div><div class="value">$${computed_adjustment.toLocaleString()}</div></div>
                <div class="card ${triggered ? 'pos' : 'neg'}"><div class="label" data-i18n="view.s743.card.triggered">Adjustment triggered?</div><div class="value">${triggered ? 'YES' : 'NO'}</div></div>
                <div class="card"><div class="label" data-i18n="view.s743.card.bil">§ 743(d) SBIL test</div><div class="value">${state.built_in_loss_amount > 250_000 ? 'YES (mandatory)' : 'NO'}</div></div>
            </div>
        </div>
    `;
}
