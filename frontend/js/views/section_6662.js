// IRC § 6662 — Accuracy-Related Penalty (20% / 40%).
// 20% on portion of underpayment attributable to: negligence, substantial understatement,
// substantial valuation, substantial overstatement of pension liability, substantial estate or gift tax valuation.
// 40% gross valuation misstatement (200%+) + 40% undisclosed foreign financial asset § 6662(j).

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    underpayment_amount: 0,
    penalty_base: 0,
    penalty_rate: 20,
    is_substantial_understatement: false,
    substantial_understatement_threshold: 5000,
    individual_pct_threshold: 10,
    corporate_pct_threshold: 10,
    is_negligence: false,
    is_disregard_rules: false,
    is_intentional_disregard: false,
    is_substantial_valuation_misstatement: false,
    valuation_pct_of_correct: 0,
    is_gross_valuation_misstatement: false,
    is_pension_liability_overstatement: false,
    is_estate_gift_undervaluation: false,
    s6662_j_undisclosed_foreign: false,
    s6662_j_40pct_amount: 0,
    s6662_h_transfer_pricing: false,
    transfer_pricing_misstatement: 0,
    s6662a_listed_transaction: false,
    s6662a_30pct_undisclosed: false,
    s6662_b_listed: false,
    s6664_reasonable_cause_defense: false,
    s6664_good_faith_defense: false,
    qualified_disclosure_made: false,
    Schedule_UTP_filed: false,
    F8275_disclosure: false,
    F8275_R_regulation_disagreement: false,
    substantial_authority: false,
    s6662_d_2_b_iii_substantial_authority: false,
    reasonable_basis: false,
    s6662_d_2_c_reportable_transaction: false,
    s6662_e_substantial_under_amount: 0,
    s6662_i_b_higher_rate_corporate: false,
    s6662_a_minimum_amount: 0,
    is_listed_or_RT: false,
    s6664_d_RT_reasonable_cause_limited: false,
    s6707a_penalty_separate: 0,
    s6662_h_excess_2_million: false,
    relief_under_s6404: false,
    waiver_negotiated: false,
    waiver_percentage: 0,
    pre_2010_penalty_pct: 0,
    post_2010_penalty_pct: 0,
    s6751_b_supervisor_approval: false,
};

export async function renderSection6662(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6662.h1.title">// § 6662 ACCURACY-RELATED PENALTY (20%/40%)</span></h1>
        <p class="muted small" data-i18n="view.s6662.hint.intro">
            <strong>§ 6662(a)</strong> — 20% penalty on portion of underpayment attributable to:
            (1) NEGLIGENCE / DISREGARD of rules, (2) SUBSTANTIAL UNDERSTATEMENT (&gt; $5K + 10% of
            correct tax for individual / lesser of 10% or $10M for C-corp), (3) SUBSTANTIAL VALUATION
            (150%+ misstatement), (4) PENSION liability overstatement, (5) ESTATE/GIFT undervaluation
            (65% or less of correct value). <strong>§ 6662(h) 40% gross valuation misstatement</strong>
            (200%+). <strong>§ 6662(j) 40% undisclosed foreign financial asset</strong> understatement.
            <strong>§ 6662(i) 30% undisclosed reportable transaction</strong> — applies separately
            from § 6707A. <strong>§ 6664 reasonable cause + good faith DEFENSE</strong> — but LIMITED
            for tax shelters + listed transactions. <strong>§ 6751(b)</strong> requires supervisor
            written approval before assessment. <strong>§ 6664(c)(2)</strong> qualified amended return
            avoids penalty if filed before audit notice.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6662.h2.inputs">Inputs</h2>
            <form id="s6662-form" class="inline-form">
                <label><span data-i18n="view.s6662.label.under">Underpayment ($)</span>
                    <input type="number" step="0.01" name="underpayment_amount" value="${state.underpayment_amount}"></label>
                <label><span data-i18n="view.s6662.label.base">Penalty base ($)</span>
                    <input type="number" step="0.01" name="penalty_base" value="${state.penalty_base}"></label>
                <label><span data-i18n="view.s6662.label.rate">Rate %</span>
                    <input type="number" step="1" name="penalty_rate" value="${state.penalty_rate}"></label>
                <label><span data-i18n="view.s6662.label.substantial">Substantial understatement?</span>
                    <input type="checkbox" name="is_substantial_understatement" ${state.is_substantial_understatement ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.threshold">$ threshold</span>
                    <input type="number" step="0.01" name="substantial_understatement_threshold" value="${state.substantial_understatement_threshold}"></label>
                <label><span data-i18n="view.s6662.label.indiv_pct">Indiv % threshold</span>
                    <input type="number" step="1" name="individual_pct_threshold" value="${state.individual_pct_threshold}"></label>
                <label><span data-i18n="view.s6662.label.corp_pct">Corp % threshold</span>
                    <input type="number" step="1" name="corporate_pct_threshold" value="${state.corporate_pct_threshold}"></label>
                <label><span data-i18n="view.s6662.label.negligence">Negligence?</span>
                    <input type="checkbox" name="is_negligence" ${state.is_negligence ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.disregard">Disregard rules?</span>
                    <input type="checkbox" name="is_disregard_rules" ${state.is_disregard_rules ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.intentional">Intentional disregard?</span>
                    <input type="checkbox" name="is_intentional_disregard" ${state.is_intentional_disregard ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.valuation">Substantial valuation?</span>
                    <input type="checkbox" name="is_substantial_valuation_misstatement" ${state.is_substantial_valuation_misstatement ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.val_pct">Valuation %</span>
                    <input type="number" step="1" name="valuation_pct_of_correct" value="${state.valuation_pct_of_correct}"></label>
                <label><span data-i18n="view.s6662.label.gross">Gross valuation?</span>
                    <input type="checkbox" name="is_gross_valuation_misstatement" ${state.is_gross_valuation_misstatement ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.pension">Pension overstmt?</span>
                    <input type="checkbox" name="is_pension_liability_overstatement" ${state.is_pension_liability_overstatement ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.estate">Estate/gift under?</span>
                    <input type="checkbox" name="is_estate_gift_undervaluation" ${state.is_estate_gift_undervaluation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.s6662j">§ 6662(j) foreign?</span>
                    <input type="checkbox" name="s6662_j_undisclosed_foreign" ${state.s6662_j_undisclosed_foreign ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.s6662j_amt">§ 6662(j) 40% amt ($)</span>
                    <input type="number" step="0.01" name="s6662_j_40pct_amount" value="${state.s6662_j_40pct_amount}"></label>
                <label><span data-i18n="view.s6662.label.s6662h">§ 6662(h) TP?</span>
                    <input type="checkbox" name="s6662_h_transfer_pricing" ${state.s6662_h_transfer_pricing ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.tp_amt">TP misstmt ($)</span>
                    <input type="number" step="0.01" name="transfer_pricing_misstatement" value="${state.transfer_pricing_misstatement}"></label>
                <label><span data-i18n="view.s6662.label.s6662a">§ 6662A listed?</span>
                    <input type="checkbox" name="s6662a_listed_transaction" ${state.s6662a_listed_transaction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.s6662a_30">§ 6662A 30%?</span>
                    <input type="checkbox" name="s6662a_30pct_undisclosed" ${state.s6662a_30pct_undisclosed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.s6662b">§ 6662(b) listed?</span>
                    <input type="checkbox" name="s6662_b_listed" ${state.s6662_b_listed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.s6664_rc">§ 6664 RC?</span>
                    <input type="checkbox" name="s6664_reasonable_cause_defense" ${state.s6664_reasonable_cause_defense ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.good_faith">Good faith?</span>
                    <input type="checkbox" name="s6664_good_faith_defense" ${state.s6664_good_faith_defense ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.disclosure">Qualified disclosure?</span>
                    <input type="checkbox" name="qualified_disclosure_made" ${state.qualified_disclosure_made ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.utp">Schedule UTP?</span>
                    <input type="checkbox" name="Schedule_UTP_filed" ${state.Schedule_UTP_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.f8275">Form 8275?</span>
                    <input type="checkbox" name="F8275_disclosure" ${state.F8275_disclosure ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.f8275r">Form 8275-R?</span>
                    <input type="checkbox" name="F8275_R_regulation_disagreement" ${state.F8275_R_regulation_disagreement ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.substantial_auth">Substantial authority?</span>
                    <input type="checkbox" name="substantial_authority" ${state.substantial_authority ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.reasonable_basis">Reasonable basis?</span>
                    <input type="checkbox" name="reasonable_basis" ${state.reasonable_basis ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.s6662d2c">§ 6662(d)(2)(C) RT?</span>
                    <input type="checkbox" name="s6662_d_2_c_reportable_transaction" ${state.s6662_d_2_c_reportable_transaction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.s6662e">§ 6662(e) under ($)</span>
                    <input type="number" step="0.01" name="s6662_e_substantial_under_amount" value="${state.s6662_e_substantial_under_amount}"></label>
                <label><span data-i18n="view.s6662.label.s6662ib">§ 6662(i)(b) corp?</span>
                    <input type="checkbox" name="s6662_i_b_higher_rate_corporate" ${state.s6662_i_b_higher_rate_corporate ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.min">§ 6662(a) min ($)</span>
                    <input type="number" step="0.01" name="s6662_a_minimum_amount" value="${state.s6662_a_minimum_amount}"></label>
                <label><span data-i18n="view.s6662.label.listed_rt">Listed/RT?</span>
                    <input type="checkbox" name="is_listed_or_RT" ${state.is_listed_or_RT ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.s6664d">§ 6664(d) RT limit?</span>
                    <input type="checkbox" name="s6664_d_RT_reasonable_cause_limited" ${state.s6664_d_RT_reasonable_cause_limited ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.s6707a">§ 6707A sep ($)</span>
                    <input type="number" step="0.01" name="s6707a_penalty_separate" value="${state.s6707a_penalty_separate}"></label>
                <label><span data-i18n="view.s6662.label.s6662h_2m">§ 6662(h) &gt; $2M?</span>
                    <input type="checkbox" name="s6662_h_excess_2_million" ${state.s6662_h_excess_2_million ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.s6404">§ 6404 relief?</span>
                    <input type="checkbox" name="relief_under_s6404" ${state.relief_under_s6404 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.waiver">Waiver?</span>
                    <input type="checkbox" name="waiver_negotiated" ${state.waiver_negotiated ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6662.label.waiver_pct">Waiver %</span>
                    <input type="number" step="1" name="waiver_percentage" value="${state.waiver_percentage}"></label>
                <label><span data-i18n="view.s6662.label.pre2010">Pre-2010 %</span>
                    <input type="number" step="1" name="pre_2010_penalty_pct" value="${state.pre_2010_penalty_pct}"></label>
                <label><span data-i18n="view.s6662.label.post2010">Post-2010 %</span>
                    <input type="number" step="1" name="post_2010_penalty_pct" value="${state.post_2010_penalty_pct}"></label>
                <label><span data-i18n="view.s6662.label.s6751b">§ 6751(b) approval?</span>
                    <input type="checkbox" name="s6751_b_supervisor_approval" ${state.s6751_b_supervisor_approval ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6662.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6662-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6662.h2.categories">§ 6662(b) penalty categories</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s6662.tbl.cat">Category</th><th data-i18n="view.s6662.tbl.rate">Rate</th><th data-i18n="view.s6662.tbl.trigger">Trigger</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s6662.tbl.negl">Negligence / disregard</td><td>20%</td><td data-i18n="view.s6662.tbl.fail_care">Fail reasonable care</td></tr>
                    <tr><td data-i18n="view.s6662.tbl.subst">Substantial understatement</td><td>20%</td><td data-i18n="view.s6662.tbl.subst_thresh">&gt; $5K + 10% of correct tax (individual) / 10% or $10M (C-corp)</td></tr>
                    <tr><td data-i18n="view.s6662.tbl.subst_val">Substantial valuation</td><td>20%</td><td data-i18n="view.s6662.tbl.150pct">150%+ misstatement</td></tr>
                    <tr><td data-i18n="view.s6662.tbl.gross_val">Gross valuation</td><td>40% (§ 6662(h))</td><td data-i18n="view.s6662.tbl.200pct">200%+ misstatement</td></tr>
                    <tr><td data-i18n="view.s6662.tbl.pension">Pension overstmt</td><td>20%</td><td data-i18n="view.s6662.tbl.pension_thresh">200%+</td></tr>
                    <tr><td data-i18n="view.s6662.tbl.estate">Estate/gift undervaluation</td><td>20%</td><td data-i18n="view.s6662.tbl.65pct">65% or less of correct value</td></tr>
                    <tr><td data-i18n="view.s6662.tbl.tp">Transfer pricing</td><td>20% (§ 6662(e)) / 40% (§ 6662(h))</td><td>200% / 400% threshold</td></tr>
                    <tr><td data-i18n="view.s6662.tbl.foreign">Undisclosed foreign asset</td><td>40% (§ 6662(j))</td><td data-i18n="view.s6662.tbl.foreign_under">Foreign asset understmt</td></tr>
                    <tr><td data-i18n="view.s6662.tbl.s6662a">§ 6662A reportable trans</td><td>20% or 30% (undisclosed)</td><td data-i18n="view.s6662.tbl.rt_under">Reportable transaction understmt</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6662.h2.s6664_defense">§ 6664 reasonable cause + good faith defense</h2>
            <ul class="muted small">
                <li data-i18n="view.s6662.def.standards">Standard: "reasonable cause + good faith" (subjective + objective)</li>
                <li data-i18n="view.s6662.def.reliance">Reliance on professional advice: Boyle factors + specific issue analyzed</li>
                <li data-i18n="view.s6662.def.competent">Advisor must be competent + provided ALL relevant facts</li>
                <li data-i18n="view.s6662.def.disclosure">§ 6662(d)(2)(B)(ii) — adequate disclosure via Form 8275 / 8275-R</li>
                <li data-i18n="view.s6662.def.subst_auth">Substantial authority: more likely than not (40%+ chance of prevailing)</li>
                <li data-i18n="view.s6662.def.reasonable_basis">Reasonable basis: less than substantial authority but reasonable (20-30% chance)</li>
                <li data-i18n="view.s6662.def.limit_tax_shelter">LIMITED for tax shelters (§ 6662(d)(2)(C)): must satisfy MLTN standard</li>
                <li data-i18n="view.s6662.def.s6664_d">§ 6664(d) — defense LIMITED for listed transactions / reportable transactions</li>
                <li data-i18n="view.s6662.def.s6664_d_must">Must show: (a) full disclosure + (b) substantial authority + (c) reasonable belief MLTN</li>
                <li data-i18n="view.s6662.def.s6662_h_no_defense">§ 6662(h) NO § 6664 defense for gross valuation misstatement (post-PPA)</li>
                <li data-i18n="view.s6662.def.qualified_amended">Qualified amended return (filed before audit notice) avoids penalty</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6662.h2.subst_under">Substantial understatement thresholds</h2>
            <ul class="muted small">
                <li data-i18n="view.s6662.su.individual">Individual: GREATER of $5,000 OR 10% of correct tax</li>
                <li data-i18n="view.s6662.su.corp_under_eligible">C-corp (NOT S-corp): LESSER of $10M OR 10% of correct tax</li>
                <li data-i18n="view.s6662.su.s_corp">S-corp / partnership: tested at INDIVIDUAL level on K-1 items</li>
                <li data-i18n="view.s6662.su.trust">Trust/estate: $5K + 10% (individual rules apply)</li>
                <li data-i18n="view.s6662.su.amounts">Computed on UNDERSTATEMENT before adjustment for substantial authority + disclosure</li>
                <li data-i18n="view.s6662.su.s6662_d_2_a">§ 6662(d)(2)(A) — items WITH substantial authority excluded from understatement calculation</li>
                <li data-i18n="view.s6662.su.s6662_d_2_b">§ 6662(d)(2)(B) — items WITH disclosure + reasonable basis excluded</li>
                <li data-i18n="view.s6662.su.s6662_d_2_c">§ 6662(d)(2)(C) — tax shelter items require MLTN substantial authority</li>
                <li data-i18n="view.s6662.su.subst_auth_examples">Substantial authority sources: Code, regs, court cases, IRS publications, revenue rulings</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6662.h2.valuation">Valuation misstatement</h2>
            <ul class="muted small">
                <li data-i18n="view.s6662.val.s_substantial">SUBSTANTIAL: 150% or more of correct value (post-PPA 2007 — was 200%)</li>
                <li data-i18n="view.s6662.val.gross">GROSS: 200%+ of correct value (post-PPA — was 400%)</li>
                <li data-i18n="view.s6662.val.charitable">Charitable contribution: appraisal required for $5K+ — failure invalidates substantial authority</li>
                <li data-i18n="view.s6662.val.estate_gift">Estate/gift: 65% or less of correct value (substantial) / 40% (gross)</li>
                <li data-i18n="view.s6662.val.pension">Pension liability: 200%+ overstatement (substantial) / 400%+ (gross)</li>
                <li data-i18n="view.s6662.val.transfer_pricing">Transfer pricing: 200% (subst) / 400% (gross) OR net § 482 adjustment &gt; $5M (subst) / $20M (gross)</li>
                <li data-i18n="view.s6662.val.basis_step_up">Built-in gain valuations on basis step-ups</li>
                <li data-i18n="view.s6662.val.fair_market">FMV determined as of relevant date</li>
                <li data-i18n="view.s6662.val.qualified_appraisal">Qualified appraisal required for charitable contributions (Reg § 1.170A-13)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6662.h2.s6751">§ 6751(b) supervisor approval</h2>
            <ul class="muted small">
                <li data-i18n="view.s6662.s6751.required">Written supervisor approval required BEFORE penalty assessed</li>
                <li data-i18n="view.s6662.s6751.timing">Approval must be made before initial determination communicated</li>
                <li data-i18n="view.s6662.s6751.s6751_b_2">§ 6751(b)(2) — exception for automatically calculated penalties</li>
                <li data-i18n="view.s6662.s6751.graev">Graev v. Comm. + Chai v. Comm. + Laidlaw — judicial scrutiny</li>
                <li data-i18n="view.s6662.s6751.kestin">Kestin v. Comm. — initial determination = formal communication to taxpayer</li>
                <li data-i18n="view.s6662.s6751.belair_w">Belair Woods v. Comm. — Form 30-day letter triggers § 6751(b)</li>
                <li data-i18n="view.s6662.s6751.litigation">Litigation tactic: penalties often vacated where § 6751(b) not satisfied</li>
                <li data-i18n="view.s6662.s6751.NRA_exception">§ 6751(b)(2)(B) — IRS-Chief Counsel approval for some penalties</li>
                <li data-i18n="view.s6662.s6751.summons_to_irs">Discovery requests on § 6751(b) compliance common</li>
            </ul>
        </div>
    `;
    document.getElementById('s6662-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.underpayment_amount = Number(fd.get('underpayment_amount')) || 0;
        state.penalty_base = Number(fd.get('penalty_base')) || 0;
        state.penalty_rate = Number(fd.get('penalty_rate')) || 0;
        state.is_substantial_understatement = !!fd.get('is_substantial_understatement');
        state.substantial_understatement_threshold = Number(fd.get('substantial_understatement_threshold')) || 0;
        state.individual_pct_threshold = Number(fd.get('individual_pct_threshold')) || 0;
        state.corporate_pct_threshold = Number(fd.get('corporate_pct_threshold')) || 0;
        state.is_negligence = !!fd.get('is_negligence');
        state.is_disregard_rules = !!fd.get('is_disregard_rules');
        state.is_intentional_disregard = !!fd.get('is_intentional_disregard');
        state.is_substantial_valuation_misstatement = !!fd.get('is_substantial_valuation_misstatement');
        state.valuation_pct_of_correct = Number(fd.get('valuation_pct_of_correct')) || 0;
        state.is_gross_valuation_misstatement = !!fd.get('is_gross_valuation_misstatement');
        state.is_pension_liability_overstatement = !!fd.get('is_pension_liability_overstatement');
        state.is_estate_gift_undervaluation = !!fd.get('is_estate_gift_undervaluation');
        state.s6662_j_undisclosed_foreign = !!fd.get('s6662_j_undisclosed_foreign');
        state.s6662_j_40pct_amount = Number(fd.get('s6662_j_40pct_amount')) || 0;
        state.s6662_h_transfer_pricing = !!fd.get('s6662_h_transfer_pricing');
        state.transfer_pricing_misstatement = Number(fd.get('transfer_pricing_misstatement')) || 0;
        state.s6662a_listed_transaction = !!fd.get('s6662a_listed_transaction');
        state.s6662a_30pct_undisclosed = !!fd.get('s6662a_30pct_undisclosed');
        state.s6662_b_listed = !!fd.get('s6662_b_listed');
        state.s6664_reasonable_cause_defense = !!fd.get('s6664_reasonable_cause_defense');
        state.s6664_good_faith_defense = !!fd.get('s6664_good_faith_defense');
        state.qualified_disclosure_made = !!fd.get('qualified_disclosure_made');
        state.Schedule_UTP_filed = !!fd.get('Schedule_UTP_filed');
        state.F8275_disclosure = !!fd.get('F8275_disclosure');
        state.F8275_R_regulation_disagreement = !!fd.get('F8275_R_regulation_disagreement');
        state.substantial_authority = !!fd.get('substantial_authority');
        state.reasonable_basis = !!fd.get('reasonable_basis');
        state.s6662_d_2_c_reportable_transaction = !!fd.get('s6662_d_2_c_reportable_transaction');
        state.s6662_e_substantial_under_amount = Number(fd.get('s6662_e_substantial_under_amount')) || 0;
        state.s6662_i_b_higher_rate_corporate = !!fd.get('s6662_i_b_higher_rate_corporate');
        state.s6662_a_minimum_amount = Number(fd.get('s6662_a_minimum_amount')) || 0;
        state.is_listed_or_RT = !!fd.get('is_listed_or_RT');
        state.s6664_d_RT_reasonable_cause_limited = !!fd.get('s6664_d_RT_reasonable_cause_limited');
        state.s6707a_penalty_separate = Number(fd.get('s6707a_penalty_separate')) || 0;
        state.s6662_h_excess_2_million = !!fd.get('s6662_h_excess_2_million');
        state.relief_under_s6404 = !!fd.get('relief_under_s6404');
        state.waiver_negotiated = !!fd.get('waiver_negotiated');
        state.waiver_percentage = Number(fd.get('waiver_percentage')) || 0;
        state.pre_2010_penalty_pct = Number(fd.get('pre_2010_penalty_pct')) || 0;
        state.post_2010_penalty_pct = Number(fd.get('post_2010_penalty_pct')) || 0;
        state.s6751_b_supervisor_approval = !!fd.get('s6751_b_supervisor_approval');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6662-output');
    if (!el) return;
    let rate = 20;
    if (state.is_gross_valuation_misstatement || state.s6662_j_undisclosed_foreign) rate = 40;
    else if (state.s6662a_30pct_undisclosed) rate = 30;
    const has_defense = (state.s6664_reasonable_cause_defense && state.s6664_good_faith_defense) || state.qualified_disclosure_made || state.substantial_authority;
    const penalty = has_defense ? 0 : state.underpayment_amount * (rate / 100);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6662.h2.result">§ 6662 penalty assessment</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s6662.card.under">Underpayment</div><div class="value">$${state.underpayment_amount.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s6662.card.rate">Rate</div><div class="value">${rate}%</div></div>
                <div class="card ${has_defense ? 'pos' : 'neg'}"><div class="label" data-i18n="view.s6662.card.defense">Defense?</div><div class="value">${has_defense ? 'YES' : 'NO'}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.s6662.card.penalty">§ 6662 penalty</div><div class="value">$${penalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div></div>
                <div class="card ${state.s6751_b_supervisor_approval ? 'pos' : 'warn'}"><div class="label" data-i18n="view.s6662.card.s6751b">§ 6751(b) approval?</div><div class="value">${state.s6751_b_supervisor_approval ? 'YES' : 'NO (vulnerable)'}</div></div>
            </div>
        </div>
    `;
}
