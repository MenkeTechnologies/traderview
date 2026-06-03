// IRC § 6221 — Bipartisan Budget Act (BBA) Centralized Partnership Audit Regime.
// Post-2017: IRS audits + adjusts partnership at ENTITY LEVEL, partnership itself pays tax.
// Replaced TEFRA (§ 6221 pre-2018) + electing large partnership rules.
// "Imputed underpayment" at HIGHEST rate, may push out to partners via § 6226 election.

import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    partnership_size: 0,
    is_election_out_eligible: false,
    s6221_b_election_out_made: false,
    election_out_year: 2024,
    eligible_partner_count: 0,
    has_ineligible_partner: false,
    ineligible_partner_type: 'trust',
    partnership_representative: '',
    is_designated_individual: false,
    is_designated_entity: false,
    imputed_underpayment: 0,
    imputed_rate: 37,
    s6225_modification_rate_lower: 0,
    is_pushed_out_to_partners: false,
    s6226_push_out_election: false,
    days_to_push_out: 0,
    s6227_administrative_adjustment_request: false,
    aar_filing: false,
    reviewed_year: 2022,
    adjustment_year: 2024,
    adjustments_year_partners_list: '',
    s6225_modifications_amended_returns: false,
    s6225_b_tax_attributes: 0,
    s6225_c_specific_modifications: false,
    s6225_c_3_higher_tax_rate: false,
    s6225_c_4_passive_partners: false,
    s6225_c_5_imputed_modification: false,
    s6231_judicial_review: false,
    fpaa_received: false,
    timing_180day_petition: 0,
    final_partnership_administrative_adjustment: false,
    s6231_a_tefra_compare: false,
    is_audited_year: false,
};

export async function renderSection6221(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6221.h1.title">// § 6221 BBA PARTNERSHIP AUDIT REGIME</span></h1>
        <p class="muted small" data-i18n="view.s6221.hint.intro">
            <strong>BBA centralized partnership audit</strong> (post-2017) replaces TEFRA. <strong>Default
            rule:</strong> IRS examines partnership at ENTITY level; partnership pays "imputed
            underpayment" at HIGHEST individual rate (37%) on net adjustments in REVIEWED year, paid
            in ADJUSTMENT year. <strong>§ 6221(b) election out:</strong> partnerships with ≤ 100
            partners (ALL eligible: individuals, C-corps, S-corps + their shareholders, estates,
            certain trusts; NOT regular trusts, partnerships, foreign entities) elect annually on
            timely filed return → revert to partner-level audit. <strong>§ 6225 modifications:</strong>
            reduce imputed underpayment via amended returns, tax-exempt partners, lower rates.
            <strong>§ 6226 push-out election:</strong> 45 days from FPAA to push to partners — they
            pay tax in adjustment year. <strong>§ 6227 AAR:</strong> partnership amended return
            (administrative adjustment request) — replaces partner-amended in BBA. <strong>§ 6223
            partnership representative</strong> (PR) — single point of contact, BINDING authority.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6221.h2.inputs">Inputs</h2>
            <form id="s6221-form" class="inline-form">
                <label><span data-i18n="view.s6221.label.size">Partnership size</span>
                    <input type="number" step="1" name="partnership_size" value="${state.partnership_size}"></label>
                <label><span data-i18n="view.s6221.label.eligible">Election out eligible?</span>
                    <input type="checkbox" name="is_election_out_eligible" ${state.is_election_out_eligible ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6221.label.elected">§ 6221(b) elected out?</span>
                    <input type="checkbox" name="s6221_b_election_out_made" ${state.s6221_b_election_out_made ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6221.label.year">Election-out year</span>
                    <input type="number" step="1" name="election_out_year" value="${state.election_out_year}"></label>
                <label><span data-i18n="view.s6221.label.eligible_partners">Eligible partner count</span>
                    <input type="number" step="1" name="eligible_partner_count" value="${state.eligible_partner_count}"></label>
                <label><span data-i18n="view.s6221.label.ineligible">Has ineligible partner?</span>
                    <input type="checkbox" name="has_ineligible_partner" ${state.has_ineligible_partner ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6221.label.ineligible_type">Ineligible type</span>
                    <select name="ineligible_partner_type">
                        <option value="trust" ${state.ineligible_partner_type === 'trust' ? 'selected' : ''}>Trust (non-grantor)</option>
                        <option value="partnership" ${state.ineligible_partner_type === 'partnership' ? 'selected' : ''}>Another partnership</option>
                        <option value="foreign" ${state.ineligible_partner_type === 'foreign' ? 'selected' : ''}>Foreign entity</option>
                        <option value="disregarded" ${state.ineligible_partner_type === 'disregarded' ? 'selected' : ''}>Disregarded entity</option>
                        <option value="none" ${state.ineligible_partner_type === 'none' ? 'selected' : ''}>None (all eligible)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6221.label.pr">Partnership rep</span>
                    <input type="text" name="partnership_representative" value="${esc(state.partnership_representative)}"></label>
                <label><span data-i18n="view.s6221.label.individual">Individual PR?</span>
                    <input type="checkbox" name="is_designated_individual" ${state.is_designated_individual ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6221.label.entity">Entity PR?</span>
                    <input type="checkbox" name="is_designated_entity" ${state.is_designated_entity ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6221.label.imputed">Imputed underpayment ($)</span>
                    <input type="number" step="10000" name="imputed_underpayment" value="${state.imputed_underpayment}"></label>
                <label><span data-i18n="view.s6221.label.rate">Imputed rate %</span>
                    <input type="number" step="0.1" name="imputed_rate" value="${state.imputed_rate}"></label>
                <label><span data-i18n="view.s6221.label.modified">§ 6225 modified rate %</span>
                    <input type="number" step="0.1" name="s6225_modification_rate_lower" value="${state.s6225_modification_rate_lower}"></label>
                <label><span data-i18n="view.s6221.label.pushed_out">Pushed out to partners?</span>
                    <input type="checkbox" name="is_pushed_out_to_partners" ${state.is_pushed_out_to_partners ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6221.label.s6226">§ 6226 push-out elected?</span>
                    <input type="checkbox" name="s6226_push_out_election" ${state.s6226_push_out_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6221.label.push_days">Days to push out</span>
                    <input type="number" step="1" name="days_to_push_out" value="${state.days_to_push_out}"></label>
                <label><span data-i18n="view.s6221.label.s6227">§ 6227 AAR?</span>
                    <input type="checkbox" name="s6227_administrative_adjustment_request" ${state.s6227_administrative_adjustment_request ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6221.label.aar">AAR filing?</span>
                    <input type="checkbox" name="aar_filing" ${state.aar_filing ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6221.label.reviewed">Reviewed year</span>
                    <input type="number" step="1" name="reviewed_year" value="${state.reviewed_year}"></label>
                <label><span data-i18n="view.s6221.label.adj_year">Adjustment year</span>
                    <input type="number" step="1" name="adjustment_year" value="${state.adjustment_year}"></label>
                <label><span data-i18n="view.s6221.label.partner_list">Adjustments yr partners list</span>
                    <input type="text" name="adjustments_year_partners_list" value="${esc(state.adjustments_year_partners_list)}"></label>
                <label><span data-i18n="view.s6221.label.modifications">§ 6225 modifications via amended returns?</span>
                    <input type="checkbox" name="s6225_modifications_amended_returns" ${state.s6225_modifications_amended_returns ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6221.label.tax_attrib">§ 6225(b) tax attributes ($)</span>
                    <input type="number" step="10000" name="s6225_b_tax_attributes" value="${state.s6225_b_tax_attributes}"></label>
                <label><span data-i18n="view.s6221.label.specific">§ 6225(c) specific mods?</span>
                    <input type="checkbox" name="s6225_c_specific_modifications" ${state.s6225_c_specific_modifications ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6221.label.c3">§ 6225(c)(3) higher tax rate?</span>
                    <input type="checkbox" name="s6225_c_3_higher_tax_rate" ${state.s6225_c_3_higher_tax_rate ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6221.label.c4">§ 6225(c)(4) passive?</span>
                    <input type="checkbox" name="s6225_c_4_passive_partners" ${state.s6225_c_4_passive_partners ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6221.label.c5">§ 6225(c)(5) imputed mod?</span>
                    <input type="checkbox" name="s6225_c_5_imputed_modification" ${state.s6225_c_5_imputed_modification ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6221.label.s6231">§ 6231 judicial review?</span>
                    <input type="checkbox" name="s6231_judicial_review" ${state.s6231_judicial_review ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6221.label.fpaa">FPAA received?</span>
                    <input type="checkbox" name="fpaa_received" ${state.fpaa_received ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6221.label.petition">180-day petition</span>
                    <input type="number" step="1" name="timing_180day_petition" value="${state.timing_180day_petition}"></label>
                <label><span data-i18n="view.s6221.label.final">Final PAA?</span>
                    <input type="checkbox" name="final_partnership_administrative_adjustment" ${state.final_partnership_administrative_adjustment ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6221.label.tefra">TEFRA comparison?</span>
                    <input type="checkbox" name="s6231_a_tefra_compare" ${state.s6231_a_tefra_compare ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6221.label.audited">Audited year?</span>
                    <input type="checkbox" name="is_audited_year" ${state.is_audited_year ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6221.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6221-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6221.h2.election_out">§ 6221(b) election-out</h2>
            <ol class="muted small">
                <li data-i18n="view.s6221.eo.partners">≤ 100 PARTNERS (counted include S-corp shareholders)</li>
                <li data-i18n="view.s6221.eo.eligible">ALL partners ELIGIBLE: individuals, C-corps, S-corps, estates, foreign individuals, eligible foreign entities</li>
                <li data-i18n="view.s6221.eo.ineligible">INELIGIBLE: partnerships (any), non-grantor trusts, disregarded entities, foreign LLCs</li>
                <li data-i18n="view.s6221.eo.election_annual">Election made ANNUALLY on timely-filed Form 1065 + Schedule B-2</li>
                <li data-i18n="view.s6221.eo.consequences">If elected: TEFRA-like partner-level audits — each partner's return adjusted separately</li>
                <li data-i18n="view.s6221.eo.disclosure">Election requires disclosure of each partner's TIN + ownership %</li>
                <li data-i18n="view.s6221.eo.s_corp_count">S-corp counted as 1 partner BUT S-corp shareholders counted toward 100 limit</li>
                <li data-i18n="view.s6221.eo.bba_default">Default if no election: BBA centralized regime applies</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6221.h2.imputed">Imputed underpayment computation</h2>
            <ul class="muted small">
                <li data-i18n="view.s6221.imp.adjustments">Net all partnership-level adjustments to reviewed year items</li>
                <li data-i18n="view.s6221.imp.rate">Apply HIGHEST INDIVIDUAL RATE (37% for 2024)</li>
                <li data-i18n="view.s6221.imp.modifications">§ 6225 modifications reduce: amended partner returns + lower rates for specific partners</li>
                <li data-i18n="view.s6221.imp.netting">Netting only across SAME taxable year + partnership</li>
                <li data-i18n="view.s6221.imp.s6225_c_3">§ 6225(c)(3) — tax-exempt partners ($0), lower-rate partners (15%/20% LTCG)</li>
                <li data-i18n="view.s6221.imp.s6225_c_4">§ 6225(c)(4) — passive activity, foreign partners</li>
                <li data-i18n="view.s6221.imp.character_NOT_traced">Character of adjustment NOT traced — all ordinary at default 37%</li>
                <li data-i18n="view.s6221.imp.interest">Interest from due date of reviewed year return</li>
                <li data-i18n="view.s6221.imp.penalties">§ 6662 / § 6663 penalties apply at partnership level</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6221.h2.push_out">§ 6226 push-out election</h2>
            <ul class="muted small">
                <li data-i18n="view.s6221.push.purpose">Shift tax liability from partnership to REVIEWED-YEAR partners</li>
                <li data-i18n="view.s6221.push.timing">Elected within 45 DAYS of final partnership adjustment</li>
                <li data-i18n="view.s6221.push.notice">Partnership issues PUSH-OUT STATEMENT to each reviewed-year partner</li>
                <li data-i18n="view.s6221.push.partners_amend">Each partner files amended return for reviewed year (or pays tax via current year)</li>
                <li data-i18n="view.s6221.push.tiered">Tiered partnerships: passes through to UPPER partners</li>
                <li data-i18n="view.s6221.push.s6226_b">§ 6226(b) — payable in CURRENT (adjustment) year by partners</li>
                <li data-i18n="view.s6221.push.s6226_c">§ 6226(c) — partners include interest in adjustment year</li>
                <li data-i18n="view.s6221.push.partner_character">Character preserved at PARTNER level (different from default § 6225)</li>
                <li data-i18n="view.s6221.push.s6226_d">§ 6226(d) — penalties at partnership AND partner level</li>
                <li data-i18n="view.s6221.push.coordination">Often preferred where partners' actual tax rate &lt; 37%</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6221.h2.pr">Partnership Representative (§ 6223)</h2>
            <ul class="muted small">
                <li data-i18n="view.s6221.pr.replaces_tmp">REPLACES TMP (Tax Matters Partner) in TEFRA</li>
                <li data-i18n="view.s6221.pr.single_contact">SINGLE point of contact with IRS</li>
                <li data-i18n="view.s6221.pr.binding">PR's actions BIND partnership + all partners</li>
                <li data-i18n="view.s6221.pr.designation">Annual designation on Form 1065 (line F)</li>
                <li data-i18n="view.s6221.pr.individual_or_entity">Can be INDIVIDUAL or ENTITY (entity must have substantial US presence)</li>
                <li data-i18n="view.s6221.pr.designated_individual">If entity PR: must designate INDIVIDUAL to act for entity</li>
                <li data-i18n="view.s6221.pr.cannot_revoke">PR cannot revoke once designated except during audit (special procedure)</li>
                <li data-i18n="view.s6221.pr.no_partner_status">PR need NOT be partner — independent professional acceptable</li>
                <li data-i18n="view.s6221.pr.s6223_b">§ 6223(b) — IRS may designate PR if none designated</li>
                <li data-i18n="view.s6221.pr.indemnification">Common: indemnification + insurance for PR liability</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6221.h2.aar">§ 6227 Administrative Adjustment Request (AAR)</h2>
            <ul class="muted small">
                <li data-i18n="view.s6221.aar.purpose">PARTNERSHIP-level amended return (replaces amended Schedule K-1)</li>
                <li data-i18n="view.s6221.aar.no_partner_amend">Partners CANNOT file amended return for partnership items individually</li>
                <li data-i18n="view.s6221.aar.timing">Within 3 years of original return filing + before issuance of notice of audit</li>
                <li data-i18n="view.s6221.aar.process">Form 8985 + 8986 to reviewed-year partners</li>
                <li data-i18n="view.s6221.aar.imputed">Imputed underpayment OR push-out election available</li>
                <li data-i18n="view.s6221.aar.s6225_c">§ 6225(c) modifications available</li>
                <li data-i18n="view.s6221.aar.s6227_b">§ 6227(b) — special rules for partnership-level adjustments increasing income</li>
                <li data-i18n="view.s6221.aar.qualified_amended">Qualifies for § 6664(c) penalty relief (qualified amended return)</li>
            </ul>
        </div>
    `;
    document.getElementById('s6221-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.partnership_size = Number(fd.get('partnership_size')) || 0;
        state.is_election_out_eligible = !!fd.get('is_election_out_eligible');
        state.s6221_b_election_out_made = !!fd.get('s6221_b_election_out_made');
        state.election_out_year = Number(fd.get('election_out_year')) || 0;
        state.eligible_partner_count = Number(fd.get('eligible_partner_count')) || 0;
        state.has_ineligible_partner = !!fd.get('has_ineligible_partner');
        state.ineligible_partner_type = fd.get('ineligible_partner_type');
        state.partnership_representative = fd.get('partnership_representative') || '';
        state.is_designated_individual = !!fd.get('is_designated_individual');
        state.is_designated_entity = !!fd.get('is_designated_entity');
        state.imputed_underpayment = Number(fd.get('imputed_underpayment')) || 0;
        state.imputed_rate = Number(fd.get('imputed_rate')) || 0;
        state.s6225_modification_rate_lower = Number(fd.get('s6225_modification_rate_lower')) || 0;
        state.is_pushed_out_to_partners = !!fd.get('is_pushed_out_to_partners');
        state.s6226_push_out_election = !!fd.get('s6226_push_out_election');
        state.days_to_push_out = Number(fd.get('days_to_push_out')) || 0;
        state.s6227_administrative_adjustment_request = !!fd.get('s6227_administrative_adjustment_request');
        state.aar_filing = !!fd.get('aar_filing');
        state.reviewed_year = Number(fd.get('reviewed_year')) || 0;
        state.adjustment_year = Number(fd.get('adjustment_year')) || 0;
        state.adjustments_year_partners_list = fd.get('adjustments_year_partners_list') || '';
        state.s6225_modifications_amended_returns = !!fd.get('s6225_modifications_amended_returns');
        state.s6225_b_tax_attributes = Number(fd.get('s6225_b_tax_attributes')) || 0;
        state.s6225_c_specific_modifications = !!fd.get('s6225_c_specific_modifications');
        state.s6225_c_3_higher_tax_rate = !!fd.get('s6225_c_3_higher_tax_rate');
        state.s6225_c_4_passive_partners = !!fd.get('s6225_c_4_passive_partners');
        state.s6225_c_5_imputed_modification = !!fd.get('s6225_c_5_imputed_modification');
        state.s6231_judicial_review = !!fd.get('s6231_judicial_review');
        state.fpaa_received = !!fd.get('fpaa_received');
        state.timing_180day_petition = Number(fd.get('timing_180day_petition')) || 0;
        state.final_partnership_administrative_adjustment = !!fd.get('final_partnership_administrative_adjustment');
        state.s6231_a_tefra_compare = !!fd.get('s6231_a_tefra_compare');
        state.is_audited_year = !!fd.get('is_audited_year');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6221-output');
    if (!el) return;
    const effective_rate = state.s6225_modification_rate_lower || state.imputed_rate;
    const computed_underpayment = state.imputed_underpayment * (effective_rate / 100);
    const eligible_for_election_out = state.partnership_size <= 100 && !state.has_ineligible_partner;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6221.h2.result">§ 6221 BBA assessment</h2>
            <div class="cards">
                <div class="card ${eligible_for_election_out ? 'pos' : 'neg'}"><div class="label" data-i18n="view.s6221.card.eligible">Election-out eligible?</div><div class="value">${eligible_for_election_out ? 'YES' : 'NO'}</div></div>
                <div class="card"><div class="label" data-i18n="view.s6221.card.rate">Effective rate</div><div class="value">${effective_rate.toFixed(1)}%</div></div>
                <div class="card neg"><div class="label" data-i18n="view.s6221.card.computed">Computed underpayment</div><div class="value">$${computed_underpayment.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
                <div class="card ${state.s6226_push_out_election ? 'warn' : ''}"><div class="label" data-i18n="view.s6221.card.push">Push-out?</div><div class="value">${state.s6226_push_out_election ? 'YES (45-day)' : 'NO'}</div></div>
            </div>
        </div>
    `;
}
