// IRC § 318 — Constructive Ownership of Stock.
// Attributes stock ownership for tax purposes across family + entities + options.
// 5 attribution rules: family + entity-to-owner + owner-to-entity + option + reattribution.
// Used by § 302/303/304 redemptions, § 267 related party, § 1361 S-corp eligibility, etc.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    purpose: 'section_302',
    direct_ownership: 0,
    family_spouse: 0,
    family_parents: 0,
    family_children: 0,
    family_grandchildren: 0,
    family_siblings: 0,
    is_brother_sister_excluded: true,
    is_grandparents_excluded: true,
    partnership_share_pct: 0,
    partnership_stock_pct: 0,
    corporation_50_pct_owned: 0,
    corporation_stock_pct: 0,
    trust_beneficial_pct: 0,
    estate_beneficial_pct: 0,
    estate_distribution_rights: 0,
    has_options: false,
    option_underlying_shares: 0,
    option_in_money: false,
    is_employee_option_excluded: false,
    is_iso_option_excluded: false,
    is_distribution_in_redemption: false,
    s302_qualifying_redemption: false,
    s302_b_substantial_disprop: false,
    s302_b_complete_termination: false,
    waiver_of_family_attribution: false,
    s302_c_2_a_waiver_filed: false,
    no_interest_acquired_10yr: true,
    s302_b_3_complete: false,
    s302_b_1_essentially_equivalent: false,
    s302_e_partial_liq_2_year: false,
    is_corporate_purposes: false,
    s1563_attribution_more_strict: false,
    s267_b_related: false,
    s954_d_active_business: false,
};

export async function renderSection318(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s318.h1.title">// § 318 CONSTRUCTIVE OWNERSHIP</span></h1>
        <p class="muted small" data-i18n="view.s318.hint.intro">
            <strong>§ 318 attributes</strong> stock ownership across persons + entities for many
            Code purposes. <strong>5 rules:</strong> (1) FAMILY (spouse, children, grandchildren,
            parents — NOT siblings, NOT in-laws), (2) ENTITY → OWNER (partnership, S-corp, trust:
            full attribution; C-corp 50%+ owned: pro-rata), (3) OWNER → ENTITY (similar pro-rata or
            full), (4) OPTIONS (treated as owning underlying), (5) RE-ATTRIBUTION (chain limits).
            <strong>Uses:</strong> § 302/303/304 redemptions, § 267 related party, § 318 itself in
            § 304, § 1361 S-corp shareholder count, § 4975 prohibited transactions.
            <strong>NOT to be confused with § 267(c)</strong> attribution (different, includes siblings,
            applies to related-party loss disallowance) or <strong>§ 1563</strong> attribution
            (controlled groups). <strong>§ 302(c)(2)(A)</strong> family-attribution waiver in
            complete-termination redemptions.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s318.h2.inputs">Inputs</h2>
            <form id="s318-form" class="inline-form">
                <label><span data-i18n="view.s318.label.purpose">Purpose</span>
                    <select name="purpose">
                        <option value="section_302" ${state.purpose === 'section_302' ? 'selected' : ''}>§ 302 redemption</option>
                        <option value="section_267" ${state.purpose === 'section_267' ? 'selected' : ''}>§ 267 related party</option>
                        <option value="section_304" ${state.purpose === 'section_304' ? 'selected' : ''}>§ 304 related corp redemption</option>
                        <option value="section_1361" ${state.purpose === 'section_1361' ? 'selected' : ''}>§ 1361 S-corp eligibility</option>
                        <option value="section_4975" ${state.purpose === 'section_4975' ? 'selected' : ''}>§ 4975 prohibited transactions</option>
                        <option value="section_338" ${state.purpose === 'section_338' ? 'selected' : ''}>§ 338 elections</option>
                        <option value="section_465_e" ${state.purpose === 'section_465_e' ? 'selected' : ''}>§ 465(e) at-risk recapture</option>
                        <option value="section_469_e" ${state.purpose === 'section_469_e' ? 'selected' : ''}>§ 469(e) passive activity</option>
                        <option value="other" ${state.purpose === 'other' ? 'selected' : ''}>Other</option>
                    </select>
                </label>
                <label><span data-i18n="view.s318.label.direct">Direct ownership ($/shares)</span>
                    <input type="number" step="1" name="direct_ownership" value="${state.direct_ownership}"></label>
                <label><span data-i18n="view.s318.label.spouse">Spouse ($)</span>
                    <input type="number" step="1" name="family_spouse" value="${state.family_spouse}"></label>
                <label><span data-i18n="view.s318.label.parents">Parents ($)</span>
                    <input type="number" step="1" name="family_parents" value="${state.family_parents}"></label>
                <label><span data-i18n="view.s318.label.children">Children ($)</span>
                    <input type="number" step="1" name="family_children" value="${state.family_children}"></label>
                <label><span data-i18n="view.s318.label.grandchildren">Grandchildren ($)</span>
                    <input type="number" step="1" name="family_grandchildren" value="${state.family_grandchildren}"></label>
                <label><span data-i18n="view.s318.label.siblings">Siblings (NOT attributed)</span>
                    <input type="number" step="1" name="family_siblings" value="${state.family_siblings}"></label>
                <label><span data-i18n="view.s318.label.sibs_excluded">Siblings excluded?</span>
                    <input type="checkbox" name="is_brother_sister_excluded" ${state.is_brother_sister_excluded ? 'checked' : ''}></label>
                <label><span data-i18n="view.s318.label.grand_excluded">Grandparents excluded?</span>
                    <input type="checkbox" name="is_grandparents_excluded" ${state.is_grandparents_excluded ? 'checked' : ''}></label>
                <label><span data-i18n="view.s318.label.ps_share">PS share %</span>
                    <input type="number" step="0.1" name="partnership_share_pct" value="${state.partnership_share_pct}"></label>
                <label><span data-i18n="view.s318.label.ps_stock">PS stock %</span>
                    <input type="number" step="0.1" name="partnership_stock_pct" value="${state.partnership_stock_pct}"></label>
                <label><span data-i18n="view.s318.label.corp50">Corp 50%+ owned?</span>
                    <input type="number" step="0.1" name="corporation_50_pct_owned" value="${state.corporation_50_pct_owned}"></label>
                <label><span data-i18n="view.s318.label.corp_stock">Corp stock %</span>
                    <input type="number" step="0.1" name="corporation_stock_pct" value="${state.corporation_stock_pct}"></label>
                <label><span data-i18n="view.s318.label.trust">Trust beneficial %</span>
                    <input type="number" step="0.1" name="trust_beneficial_pct" value="${state.trust_beneficial_pct}"></label>
                <label><span data-i18n="view.s318.label.estate">Estate beneficial %</span>
                    <input type="number" step="0.1" name="estate_beneficial_pct" value="${state.estate_beneficial_pct}"></label>
                <label><span data-i18n="view.s318.label.estate_rights">Estate distribution rights %</span>
                    <input type="number" step="0.1" name="estate_distribution_rights" value="${state.estate_distribution_rights}"></label>
                <label><span data-i18n="view.s318.label.options">Has options?</span>
                    <input type="checkbox" name="has_options" ${state.has_options ? 'checked' : ''}></label>
                <label><span data-i18n="view.s318.label.option_shares">Option underlying shares</span>
                    <input type="number" step="1" name="option_underlying_shares" value="${state.option_underlying_shares}"></label>
                <label><span data-i18n="view.s318.label.in_money">Option in money?</span>
                    <input type="checkbox" name="option_in_money" ${state.option_in_money ? 'checked' : ''}></label>
                <label><span data-i18n="view.s318.label.employee_option">Employee option excluded?</span>
                    <input type="checkbox" name="is_employee_option_excluded" ${state.is_employee_option_excluded ? 'checked' : ''}></label>
                <label><span data-i18n="view.s318.label.iso">ISO excluded?</span>
                    <input type="checkbox" name="is_iso_option_excluded" ${state.is_iso_option_excluded ? 'checked' : ''}></label>
                <label><span data-i18n="view.s318.label.redemption">Distribution in redemption?</span>
                    <input type="checkbox" name="is_distribution_in_redemption" ${state.is_distribution_in_redemption ? 'checked' : ''}></label>
                <label><span data-i18n="view.s318.label.s302_qual">§ 302 qualifying?</span>
                    <input type="checkbox" name="s302_qualifying_redemption" ${state.s302_qualifying_redemption ? 'checked' : ''}></label>
                <label><span data-i18n="view.s318.label.s302_b">§ 302(b) substantial disprop?</span>
                    <input type="checkbox" name="s302_b_substantial_disprop" ${state.s302_b_substantial_disprop ? 'checked' : ''}></label>
                <label><span data-i18n="view.s318.label.complete_term">Complete termination?</span>
                    <input type="checkbox" name="s302_b_complete_termination" ${state.s302_b_complete_termination ? 'checked' : ''}></label>
                <label><span data-i18n="view.s318.label.waiver">Waiver of family attribution?</span>
                    <input type="checkbox" name="waiver_of_family_attribution" ${state.waiver_of_family_attribution ? 'checked' : ''}></label>
                <label><span data-i18n="view.s318.label.s302c2a">§ 302(c)(2)(A) waiver filed?</span>
                    <input type="checkbox" name="s302_c_2_a_waiver_filed" ${state.s302_c_2_a_waiver_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s318.label.no_interest">No interest acquired 10yr?</span>
                    <input type="checkbox" name="no_interest_acquired_10yr" ${state.no_interest_acquired_10yr ? 'checked' : ''}></label>
                <label><span data-i18n="view.s318.label.s302b3">§ 302(b)(3) complete?</span>
                    <input type="checkbox" name="s302_b_3_complete" ${state.s302_b_3_complete ? 'checked' : ''}></label>
                <label><span data-i18n="view.s318.label.s302b1">§ 302(b)(1) ess equiv?</span>
                    <input type="checkbox" name="s302_b_1_essentially_equivalent" ${state.s302_b_1_essentially_equivalent ? 'checked' : ''}></label>
                <label><span data-i18n="view.s318.label.s302e">§ 302(e) partial liq?</span>
                    <input type="checkbox" name="s302_e_partial_liq_2_year" ${state.s302_e_partial_liq_2_year ? 'checked' : ''}></label>
                <label><span data-i18n="view.s318.label.corp_purpose">Corporate purpose?</span>
                    <input type="checkbox" name="is_corporate_purposes" ${state.is_corporate_purposes ? 'checked' : ''}></label>
                <label><span data-i18n="view.s318.label.s1563">§ 1563 stricter attribution?</span>
                    <input type="checkbox" name="s1563_attribution_more_strict" ${state.s1563_attribution_more_strict ? 'checked' : ''}></label>
                <label><span data-i18n="view.s318.label.s267b">§ 267(b) related?</span>
                    <input type="checkbox" name="s267_b_related" ${state.s267_b_related ? 'checked' : ''}></label>
                <label><span data-i18n="view.s318.label.s954d">§ 954(d) active business?</span>
                    <input type="checkbox" name="s954_d_active_business" ${state.s954_d_active_business ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s318.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s318-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s318.h2.family">§ 318(a)(1) family attribution</h2>
            <ul class="muted small">
                <li data-i18n="view.s318.fam.spouse">SPOUSE: full attribution (not legally separated)</li>
                <li data-i18n="view.s318.fam.children">CHILDREN: full attribution (including adopted)</li>
                <li data-i18n="view.s318.fam.grandchildren">GRANDCHILDREN: full attribution</li>
                <li data-i18n="view.s318.fam.parents">PARENTS: full attribution</li>
                <li data-i18n="view.s318.fam.NOT_siblings">NOT siblings (brothers, sisters)</li>
                <li data-i18n="view.s318.fam.NOT_grandparents">NOT grandparents</li>
                <li data-i18n="view.s318.fam.NOT_in_laws">NOT in-laws</li>
                <li data-i18n="view.s318.fam.NOT_step">NOT step-relatives (in most cases)</li>
                <li data-i18n="view.s318.fam.s302_c2a">§ 302(c)(2)(A) waiver: family attribution can be WAIVED for complete-termination redemption</li>
                <li data-i18n="view.s318.fam.adoption">Legally adopted = blood relationship (Reg § 1.318-1(c))</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s318.h2.entity">§ 318(a)(2) + (3) entity attribution</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s318.tbl.entity">Entity</th><th data-i18n="view.s318.tbl.to_owner">To owner (a)(2)</th><th data-i18n="view.s318.tbl.from_owner">From owner (a)(3)</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s318.tbl.partnership">Partnership</td><td data-i18n="view.s318.tbl.full_partner">Pro-rata by interest</td><td data-i18n="view.s318.tbl.full_partner_2">Full attribution from partner</td></tr>
                    <tr><td data-i18n="view.s318.tbl.s_corp">S-corp</td><td data-i18n="view.s318.tbl.full_s">Same as partnership</td><td>Full</td></tr>
                    <tr><td data-i18n="view.s318.tbl.c_corp">C-corp (50%+ owner)</td><td data-i18n="view.s318.tbl.pro_rata">Pro-rata by ownership %</td><td data-i18n="view.s318.tbl.full_from">Full from 50%+ owner</td></tr>
                    <tr><td data-i18n="view.s318.tbl.c_corp_under_50">C-corp (under 50%)</td><td data-i18n="view.s318.tbl.no_to">No attribution to corp</td><td data-i18n="view.s318.tbl.no_from">No attribution from corp</td></tr>
                    <tr><td data-i18n="view.s318.tbl.trust">Trust</td><td data-i18n="view.s318.tbl.beneficial">Beneficial interest %</td><td>Full from beneficiary</td></tr>
                    <tr><td data-i18n="view.s318.tbl.estate">Estate</td><td data-i18n="view.s318.tbl.dist_rights">Distribution rights %</td><td>Full from beneficiary</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s318.h2.options">§ 318(a)(4) options</h2>
            <ul class="muted small">
                <li data-i18n="view.s318.opt.included">Person treated as owning underlying stock</li>
                <li data-i18n="view.s318.opt.scope">Options on stock not yet issued — covered</li>
                <li data-i18n="view.s318.opt.warrants">Warrants + convertible debt — covered</li>
                <li data-i18n="view.s318.opt.s382_options">§ 382 has separate options regs — typically more inclusive</li>
                <li data-i18n="view.s318.opt.employee">Employee options: included if currently exercisable</li>
                <li data-i18n="view.s318.opt.contingent">Contingent options: facts &amp; circumstances</li>
                <li data-i18n="view.s318.opt.no_exercise_no_attribution">Out-of-money options: still treated as ownership (no in-the-money requirement under § 318)</li>
                <li data-i18n="view.s318.opt.s382_compare">Contrast § 382 + § 1.382-2T: more aggressive option counting</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s318.h2.reattribution">§ 318(a)(5) re-attribution limits</h2>
            <ul class="muted small">
                <li data-i18n="view.s318.reat.purpose">Anti-doubling: limits chain attribution</li>
                <li data-i18n="view.s318.reat.family_to_family">§ 318(a)(5)(A): NO family-to-family re-attribution (parent's stock → child → child's spouse)</li>
                <li data-i18n="view.s318.reat.option_to_family">§ 318(a)(5)(D): NO option-to-family attribution chain (option treated AS stock, then family rule)</li>
                <li data-i18n="view.s318.reat.option_first">Option always attributed FIRST before family</li>
                <li data-i18n="view.s318.reat.s_to_s_yes">Family → S → owner (partnership) → up to top: YES allowed</li>
                <li data-i18n="view.s318.reat.entity_to_entity_OK">Entity to entity via individual OK</li>
                <li data-i18n="view.s318.reat.partial_chain">Family → entity → entity OK (no family link to start chain blocked)</li>
                <li data-i18n="view.s318.reat.s302">Important for § 302 redemption tests — count ownership pre/post</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s318.h2.s302_redemption">§ 302 redemption qualification</h2>
            <ol class="muted small">
                <li data-i18n="view.s318.s302.s302_b_1">§ 302(b)(1): "essentially equivalent to dividend" (facts &amp; circumstances)</li>
                <li data-i18n="view.s318.s302.s302_b_2">§ 302(b)(2): substantial disproportion — reduced ownership &lt; 50% AND &lt; 80% × prior</li>
                <li data-i18n="view.s318.s302.s302_b_3">§ 302(b)(3): complete termination of shareholder interest</li>
                <li data-i18n="view.s318.s302.s302_b_4">§ 302(b)(4): partial liquidation of corp business</li>
                <li data-i18n="view.s318.s302.s302_e">§ 302(e): partial liq — at least 2 active businesses, one continuation</li>
                <li data-i18n="view.s318.s302.qualifying">If qualifies under § 302(b): redemption = sale/exchange (capital gain treatment)</li>
                <li data-i18n="view.s318.s302.non_qual">If not: dividend treatment (ordinary, then capital gain on basis recovery)</li>
                <li data-i18n="view.s318.s302.waiver">§ 302(c)(2)(A) family attribution waiver: 10-yr no-interest covenant</li>
            </ol>
        </div>
    `;
    document.getElementById('s318-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.purpose = fd.get('purpose');
        state.direct_ownership = Number(fd.get('direct_ownership')) || 0;
        state.family_spouse = Number(fd.get('family_spouse')) || 0;
        state.family_parents = Number(fd.get('family_parents')) || 0;
        state.family_children = Number(fd.get('family_children')) || 0;
        state.family_grandchildren = Number(fd.get('family_grandchildren')) || 0;
        state.family_siblings = Number(fd.get('family_siblings')) || 0;
        state.is_brother_sister_excluded = !!fd.get('is_brother_sister_excluded');
        state.is_grandparents_excluded = !!fd.get('is_grandparents_excluded');
        state.partnership_share_pct = Number(fd.get('partnership_share_pct')) || 0;
        state.partnership_stock_pct = Number(fd.get('partnership_stock_pct')) || 0;
        state.corporation_50_pct_owned = Number(fd.get('corporation_50_pct_owned')) || 0;
        state.corporation_stock_pct = Number(fd.get('corporation_stock_pct')) || 0;
        state.trust_beneficial_pct = Number(fd.get('trust_beneficial_pct')) || 0;
        state.estate_beneficial_pct = Number(fd.get('estate_beneficial_pct')) || 0;
        state.estate_distribution_rights = Number(fd.get('estate_distribution_rights')) || 0;
        state.has_options = !!fd.get('has_options');
        state.option_underlying_shares = Number(fd.get('option_underlying_shares')) || 0;
        state.option_in_money = !!fd.get('option_in_money');
        state.is_employee_option_excluded = !!fd.get('is_employee_option_excluded');
        state.is_iso_option_excluded = !!fd.get('is_iso_option_excluded');
        state.is_distribution_in_redemption = !!fd.get('is_distribution_in_redemption');
        state.s302_qualifying_redemption = !!fd.get('s302_qualifying_redemption');
        state.s302_b_substantial_disprop = !!fd.get('s302_b_substantial_disprop');
        state.s302_b_complete_termination = !!fd.get('s302_b_complete_termination');
        state.waiver_of_family_attribution = !!fd.get('waiver_of_family_attribution');
        state.s302_c_2_a_waiver_filed = !!fd.get('s302_c_2_a_waiver_filed');
        state.no_interest_acquired_10yr = !!fd.get('no_interest_acquired_10yr');
        state.s302_b_3_complete = !!fd.get('s302_b_3_complete');
        state.s302_b_1_essentially_equivalent = !!fd.get('s302_b_1_essentially_equivalent');
        state.s302_e_partial_liq_2_year = !!fd.get('s302_e_partial_liq_2_year');
        state.is_corporate_purposes = !!fd.get('is_corporate_purposes');
        state.s1563_attribution_more_strict = !!fd.get('s1563_attribution_more_strict');
        state.s267_b_related = !!fd.get('s267_b_related');
        state.s954_d_active_business = !!fd.get('s954_d_active_business');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s318-output');
    if (!el) return;
    const family_total = state.family_spouse + state.family_parents + state.family_children + state.family_grandchildren;
    const total_constructive = state.direct_ownership + family_total + (state.has_options ? state.option_underlying_shares : 0);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s318.h2.result">§ 318 constructive ownership</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s318.card.direct">Direct</div><div class="value">${state.direct_ownership.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s318.card.family">Family (attributed)</div><div class="value">${family_total.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s318.card.options">Options</div><div class="value">${(state.has_options ? state.option_underlying_shares : 0).toLocaleString()}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s318.card.total">Total constructive</div><div class="value">${total_constructive.toLocaleString()}</div></div>
                <div class="card warn"><div class="label" data-i18n="view.s318.card.siblings">Siblings (NOT attributed)</div><div class="value">${state.family_siblings.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
