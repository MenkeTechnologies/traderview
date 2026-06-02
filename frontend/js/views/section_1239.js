// IRC § 1239 — Sale of Property Between Related Persons.
// Gain on sale to related party = ORDINARY INCOME (not capital gain).
// Property must be DEPRECIABLE in hands of recipient (not seller).
// Related person: > 50% controlled entity, family members, partnerships, trusts.
// Prevents conversion of ordinary income (depreciation recapture eligible) to LTCG.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    sale_price: 0,
    seller_basis: 0,
    is_related_party: true,
    is_depreciable_to_buyer: true,
    family_member: false,
    is_controlled_entity: false,
    ownership_pct: 0,
    is_trust_arrangement: false,
    is_corp_partnership: false,
    held_as_capital_asset: false,
    seller_marginal: 37,
    long_term_capital_rate: 20,
    holding_period_days: 0,
};

export async function renderSection1239(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1239.h1.title">// § 1239 RELATED PARTY SALE</span></h1>
        <p class="muted small" data-i18n="view.s1239.hint.intro">
            <strong>Gain on sale to RELATED PARTY = ORDINARY INCOME</strong> (not capital gain).
            <strong>Property must be DEPRECIABLE</strong> in hands of recipient (not seller).
            <strong>Related person:</strong> &gt; 50% controlled entity, family members, partnerships,
            trusts, controlled corps. <strong>Purpose:</strong> prevents conversion of ordinary income
            (depreciation recapture eligible) to LTCG via sale to controlled buyer who then takes new
            depreciation basis. <strong>Always recharacterize as ordinary.</strong> Form 4797 + Schedule D.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1239.h2.inputs">Inputs</h2>
            <form id="s1239-form" class="inline-form">
                <label><span data-i18n="view.s1239.label.sale">Sale price ($)</span>
                    <input type="number" step="10000" name="sale_price" value="${state.sale_price}"></label>
                <label><span data-i18n="view.s1239.label.basis">Seller's basis ($)</span>
                    <input type="number" step="10000" name="seller_basis" value="${state.seller_basis}"></label>
                <label><span data-i18n="view.s1239.label.related">Related party?</span>
                    <input type="checkbox" name="is_related_party" ${state.is_related_party ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1239.label.depreciable">Depreciable in buyer's hands?</span>
                    <input type="checkbox" name="is_depreciable_to_buyer" ${state.is_depreciable_to_buyer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1239.label.family">Family member?</span>
                    <input type="checkbox" name="family_member" ${state.family_member ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1239.label.entity">Controlled entity (> 50%)?</span>
                    <input type="checkbox" name="is_controlled_entity" ${state.is_controlled_entity ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1239.label.ownership">Ownership %</span>
                    <input type="number" step="0.1" name="ownership_pct" value="${state.ownership_pct}"></label>
                <label><span data-i18n="view.s1239.label.trust">Trust arrangement?</span>
                    <input type="checkbox" name="is_trust_arrangement" ${state.is_trust_arrangement ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1239.label.corp_ps">Corp / partnership?</span>
                    <input type="checkbox" name="is_corp_partnership" ${state.is_corp_partnership ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1239.label.capital">Held as capital asset by seller?</span>
                    <input type="checkbox" name="held_as_capital_asset" ${state.held_as_capital_asset ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1239.label.marginal">Seller marginal rate %</span>
                    <input type="number" step="0.1" name="seller_marginal" value="${state.seller_marginal}"></label>
                <label><span data-i18n="view.s1239.label.ltcg">LTCG rate %</span>
                    <input type="number" step="0.1" name="long_term_capital_rate" value="${state.long_term_capital_rate}"></label>
                <label><span data-i18n="view.s1239.label.holding">Holding period days</span>
                    <input type="number" step="1" name="holding_period_days" value="${state.holding_period_days}"></label>
                <button class="primary" type="submit" data-i18n="view.s1239.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1239-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1239.h2.related_definitions">Related person definitions (§ 1239(b) + § 267)</h2>
            <ul class="muted small">
                <li data-i18n="view.s1239.rel.individual_corp">Individual + corporation: &gt; 50% owned (directly or constructively)</li>
                <li data-i18n="view.s1239.rel.corp_corp">Corporation + corporation: members of controlled group (&gt; 50% common ownership)</li>
                <li data-i18n="view.s1239.rel.partnership_ind">Partnership + partner: any partner having any partnership interest</li>
                <li data-i18n="view.s1239.rel.partnership_corp">Partnership + corporation: &gt; 50% common ownership</li>
                <li data-i18n="view.s1239.rel.trust_grantor">Trust + grantor: any trust where grantor / beneficiary related</li>
                <li data-i18n="view.s1239.rel.fiduciary">Fiduciary + beneficiary: of same trust</li>
                <li data-i18n="view.s1239.rel.family_def">Family: spouse, ancestors, lineal descendants, siblings (full / half blood)</li>
                <li data-i18n="view.s1239.rel.constructive_267">§ 267 constructive ownership rules apply (family + entity + options)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1239.h2.examples">§ 1239 application examples</h2>
            <ul class="muted small">
                <li data-i18n="view.s1239.ex.individual_corp">Individual sells equipment (capital asset to her) to her 100% S-corp → § 1239 ordinary</li>
                <li data-i18n="view.s1239.ex.corp_subsidiary">Parent sells building to 60% subsidiary → § 1239 ordinary on gain</li>
                <li data-i18n="view.s1239.ex.partnership_partner">Partnership sells truck to controlling partner → § 1239 ordinary</li>
                <li data-i18n="view.s1239.ex.family">Father sells rental property (depreciable in son's hands) to son → § 1239 ordinary</li>
                <li data-i18n="view.s1239.ex.no_depreciation">Sale of land (NOT depreciable) to controlled entity → § 1239 N/A — LTCG OK</li>
                <li data-i18n="view.s1239.ex.investment_to_personal">Sale to spouse for personal use (not depreciable) → § 1239 N/A</li>
                <li data-i18n="view.s1239.ex.s_corp_owners">Two S-corps with same 100% owner sell to each other → both § 1239 + § 267</li>
                <li data-i18n="view.s1239.ex.basis_to_buyer">Buyer's basis = purchase price (full step-up for depreciation purposes)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1239.h2.coordination">Coordination with § 267, § 1245, § 1250</h2>
            <ul class="muted small">
                <li data-i18n="view.s1239.coord.s267">§ 267 disallows LOSS on related-party sale (separate from § 1239 ordinary characterization)</li>
                <li data-i18n="view.s1239.coord.s1245">§ 1245 recapture: ordinary up to depreciation; § 1239 stacks AFTER recapture</li>
                <li data-i18n="view.s1239.coord.s1250">§ 1250 recapture (real estate): also ordinary; § 1239 stacks on top of any remaining cap gain</li>
                <li data-i18n="view.s1239.coord.s1031">§ 1031 like-kind exchange: § 1239 holding period rules — must hold 2+ yrs after exchange to avoid</li>
                <li data-i18n="view.s1239.coord.installment">§ 453 installment method: § 1239 applies to FULL gain even if reported via installment</li>
                <li data-i18n="view.s1239.coord.no_loss">§ 1239 doesn't apply to LOSSES — losses limited / disallowed by § 267 instead</li>
                <li data-i18n="view.s1239.coord.character_election">No election to opt-out — § 1239 mandatory if conditions met</li>
                <li data-i18n="view.s1239.coord.s453_2yr">§ 453 2-yr resale rule: related buyer can't resell within 2 yrs to outsiders without acceleration</li>
            </ul>
        </div>
    `;
    document.getElementById('s1239-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.sale_price = Number(fd.get('sale_price')) || 0;
        state.seller_basis = Number(fd.get('seller_basis')) || 0;
        state.is_related_party = !!fd.get('is_related_party');
        state.is_depreciable_to_buyer = !!fd.get('is_depreciable_to_buyer');
        state.family_member = !!fd.get('family_member');
        state.is_controlled_entity = !!fd.get('is_controlled_entity');
        state.ownership_pct = Number(fd.get('ownership_pct')) || 0;
        state.is_trust_arrangement = !!fd.get('is_trust_arrangement');
        state.is_corp_partnership = !!fd.get('is_corp_partnership');
        state.held_as_capital_asset = !!fd.get('held_as_capital_asset');
        state.seller_marginal = Number(fd.get('seller_marginal')) || 0;
        state.long_term_capital_rate = Number(fd.get('long_term_capital_rate')) || 0;
        state.holding_period_days = Number(fd.get('holding_period_days')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1239-output');
    if (!el) return;
    const gain = Math.max(0, state.sale_price - state.seller_basis);
    const sec1239Applies = state.is_related_party && state.is_depreciable_to_buyer;
    const ordinaryAmount = sec1239Applies ? gain : 0;
    const capGainAmount = sec1239Applies ? 0 : gain;
    const ordinaryTax = ordinaryAmount * (state.seller_marginal / 100);
    const capGainTax = capGainAmount * (state.long_term_capital_rate / 100);
    const totalTax = ordinaryTax + capGainTax;
    const counterFactualTax = gain * (state.long_term_capital_rate / 100);
    const additionalTax = totalTax - counterFactualTax;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1239.h2.result">§ 1239 outcome</h2>
            <div class="cards">
                <div class="card ${sec1239Applies ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s1239.card.applies">§ 1239 applies?</div>
                    <div class="value">${sec1239Applies ? esc(t('view.s1239.status.yes')) : esc(t('view.s1239.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1239.card.gain">Realized gain</div>
                    <div class="value">$${gain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1239.card.ordinary">Ordinary (recharacterized)</div>
                    <div class="value">$${ordinaryAmount.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1239.card.capgain">Capital gain</div>
                    <div class="value">$${capGainAmount.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1239.card.total_tax">Total tax</div>
                    <div class="value">$${totalTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1239.card.additional">§ 1239 additional cost</div>
                    <div class="value">$${additionalTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${sec1239Applies ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s1239.applies_note">
                    § 1239 forces ORDINARY rate (up to 37%) instead of LTCG (20%). Loss of preferential rate
                    on gain. To avoid: sell to UNRELATED buyer (may include LLC where ownership reduced &lt; 50%),
                    OR distribute property to shareholder first then sale outside related-party rules.
                </p>
            ` : ''}
        </div>
    `;
}
