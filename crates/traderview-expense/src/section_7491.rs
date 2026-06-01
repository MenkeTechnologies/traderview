//! IRC § 7491 — Burden of proof shifts to Secretary. Civil
//! procedural defense that flips the burden of proof from
//! taxpayer to IRS once taxpayer satisfies threshold
//! requirements. Distinct from `section_6664` (reasonable cause
//! and good faith defense to penalties), § 7454(a) (fraud burden
//! specific to § 6663 and accumulated earnings), and
//! `section_6663` (civil fraud penalty).
//!
//! **§ 7491(a)(1) — General burden shift on factual issues**.
//! If a taxpayer introduces CREDIBLE EVIDENCE with respect to
//! any factual issue relevant to determining the taxpayer's
//! liability under Subtitle A (income tax) or B (estate, gift,
//! GST tax), the Secretary bears the burden of proof on that
//! issue. "Credible evidence" = evidence the court would find
//! sufficient to base a decision on if no contrary evidence
//! were submitted (post-critical-analysis quality test).
//!
//! **§ 7491(a)(2) — Three threshold conditions for the shift**:
//!
//! - **§ 7491(a)(2)(A) — Substantiation**. Taxpayer has
//!   complied with substantiation requirements under the Code.
//! - **§ 7491(a)(2)(B) — Records maintained**. Taxpayer has
//!   maintained all required records.
//! - **§ 7491(a)(2)(C) — Cooperation**. Taxpayer has
//!   cooperated with reasonable IRS requests for witnesses,
//!   information, documents, meetings, and interviews.
//!
//! All three required (conjunctive). Missing any one defeats
//! the (a)(1) burden shift.
//!
//! **§ 7491(a)(2)(C) — Net worth limitation**. CORPORATIONS,
//! PARTNERSHIPS, and TRUSTS with net worth EXCEEDING $7,000,000
//! at the time of proceeding are EXCLUDED from the (a)(1)
//! burden-shifting provision. Individuals + entities ≤ $7M not
//! limited. Excludes most large business entities.
//!
//! **§ 7491(b) — Statistical reconstruction burden**. In any
//! court proceeding under Subtitle A (income tax) involving a
//! taxpayer who is an INDIVIDUAL, the Secretary bears the
//! burden of proof regarding any item of income reconstructed
//! by use of STATISTICAL METHODS from unrelated taxpayers
//! (e.g., Bureau of Labor Statistics consumer expenditure
//! surveys, market-segment analysis).
//!
//! **§ 7491(c) — Penalty production burden**. The Secretary
//! bears the BURDEN OF PRODUCTION (not persuasion) for any
//! PENALTY or ADDITION TO TAX (including § 6651, § 6662,
//! § 6663, § 6672, etc.) in any court proceeding. IRS must
//! come forward with evidence supporting the penalty before
//! taxpayer must rebut. Lower bar than full burden of
//! persuasion but still procedurally significant.
//!
//! **Trader-relevant**: Aggressive § 1256 mark-to-market,
//! § 988 currency, § 1202 QSBS holding-period, § 475(f)
//! trader-tax-status claims often face IRS challenge on
//! factual records. Maintaining contemporaneous records +
//! cooperating with audit + introducing credible evidence at
//! Tax Court shifts the substantive burden to IRS under
//! § 7491(a). Penalty assessments separately face § 7491(c)
//! production burden on IRS.
//!
//! Citations: IRC § 7491(a)(1) general burden shift;
//! § 7491(a)(2)(A) substantiation; § 7491(a)(2)(B) records;
//! § 7491(a)(2)(C) cooperation + $7M net worth limitation;
//! § 7491(b) statistical reconstruction burden; § 7491(c)
//! penalty production burden; cross-references § 7454(a)
//! (fraud + accumulated earnings burden), § 6664(c) (reasonable
//! cause defense), § 6663 (civil fraud), § 6662 (accuracy
//! penalty). § 7491 enacted as part of IRS Restructuring and
//! Reform Act of 1998 (Pub. L. No. 105-206).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Individual,
    Corporation,
    Partnership,
    Trust,
    Estate,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaxType {
    /// Subtitle A income tax (including SE tax, NIIT).
    IncomeOrSubtitleA,
    /// Subtitle B estate, gift, GST tax.
    EstateGiftGstSubtitleB,
    /// Subtitle C employment tax (excluded from § 7491(a)).
    EmploymentSubtitleC,
    /// Subtitle D excise tax (excluded).
    ExciseSubtitleD,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7491Input {
    pub tax_type: TaxType,
    pub entity_type: EntityType,
    /// Net worth in cents at time of proceeding (for § 7491(a)(2)(C)
    /// $7M corporate/partnership/trust limitation).
    pub net_worth_cents: i64,
    /// Whether the taxpayer introduced CREDIBLE EVIDENCE on the
    /// factual issue (§ 7491(a)(1) general burden-shift trigger).
    pub taxpayer_introduced_credible_evidence: bool,
    /// Whether the taxpayer complied with substantiation
    /// requirements (§ 7491(a)(2)(A)).
    pub taxpayer_substantiation_complied: bool,
    /// Whether the taxpayer maintained all required records
    /// (§ 7491(a)(2)(B)).
    pub taxpayer_maintained_records: bool,
    /// Whether the taxpayer cooperated with reasonable IRS
    /// requests (§ 7491(a)(2)(C)).
    pub taxpayer_cooperated_reasonably: bool,
    /// Whether the IRS used statistical reconstruction methods
    /// to determine income (§ 7491(b)).
    pub irs_used_statistical_reconstruction: bool,
    /// Whether the IRS is asserting a penalty or addition to tax
    /// (§ 7491(c) production burden).
    pub irs_seeks_penalty_or_addition: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7491Result {
    pub subsection_a_1_burden_shift_engaged: bool,
    pub subsection_b_statistical_burden_engaged: bool,
    pub subsection_c_penalty_production_burden_engaged: bool,
    pub net_worth_limitation_engaged: bool,
    /// Whether any burden-shift pathway is engaged.
    pub any_burden_shift_engaged: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section7491Input) -> Section7491Result {
    let mut notes: Vec<String> = Vec::new();

    let tax_within_subtitle_a_b = matches!(
        input.tax_type,
        TaxType::IncomeOrSubtitleA | TaxType::EstateGiftGstSubtitleB
    );

    let net_worth_threshold_cents = 700_000_000i64;
    let net_worth_limited_entity = matches!(
        input.entity_type,
        EntityType::Corporation | EntityType::Partnership | EntityType::Trust
    );
    let net_worth_limitation_engaged = net_worth_limited_entity
        && input.net_worth_cents > net_worth_threshold_cents;

    let a_2_threshold_met = input.taxpayer_substantiation_complied
        && input.taxpayer_maintained_records
        && input.taxpayer_cooperated_reasonably;

    let a_1_engaged = tax_within_subtitle_a_b
        && input.taxpayer_introduced_credible_evidence
        && a_2_threshold_met
        && !net_worth_limitation_engaged;

    let b_engaged = matches!(input.tax_type, TaxType::IncomeOrSubtitleA)
        && matches!(input.entity_type, EntityType::Individual)
        && input.irs_used_statistical_reconstruction;

    let c_engaged = input.irs_seeks_penalty_or_addition;

    let any_engaged = a_1_engaged || b_engaged || c_engaged;

    notes.push(
        "§ 7491(a)(1) — Secretary bears burden of proof when taxpayer introduces CREDIBLE EVIDENCE on factual issue under Subtitle A or B taxes"
            .to_string(),
    );

    notes.push(
        "§ 7491(a)(1) credible evidence = quality of evidence court would find sufficient to base a decision on if no contrary evidence submitted (post-critical-analysis test)"
            .to_string(),
    );

    if !tax_within_subtitle_a_b {
        notes.push(
            "§ 7491(a) — burden-shift framework applies only to Subtitle A (income) and Subtitle B (estate/gift/GST) taxes; employment + excise taxes outside scope"
                .to_string(),
        );
    }

    if net_worth_limitation_engaged {
        notes.push(format!(
            "§ 7491(a)(2)(C) net worth limitation — corporations / partnerships / trusts with net worth exceeding $7,000,000 EXCLUDED from (a)(1) burden-shifting; entity net worth = ${}",
            input.net_worth_cents / 100
        ));
    }

    if a_2_threshold_met {
        notes.push(
            "§ 7491(a)(2) three threshold conditions satisfied: (A) substantiation + (B) records + (C) cooperation"
                .to_string(),
        );
    } else {
        notes.push(
            "§ 7491(a)(2) three threshold conditions NOT all satisfied: requires (A) substantiation + (B) records maintained + (C) cooperation with reasonable IRS requests for witnesses/information/documents/meetings/interviews; missing any one defeats shift"
                .to_string(),
        );
    }

    if a_1_engaged {
        notes.push(
            "§ 7491(a)(1) burden shift ENGAGED — IRS bears burden of proof on factual issues taxpayer introduced credible evidence about"
                .to_string(),
        );
    }

    if input.irs_used_statistical_reconstruction {
        if b_engaged {
            notes.push(
                "§ 7491(b) — Secretary bears burden of proof regarding any item of income reconstructed by statistical methods from unrelated taxpayers (BLS surveys, market-segment analysis)"
                    .to_string(),
            );
        } else {
            notes.push(
                "§ 7491(b) — statistical reconstruction burden applies only to INDIVIDUAL taxpayers under Subtitle A; current taxpayer outside scope"
                    .to_string(),
            );
        }
    }

    if c_engaged {
        notes.push(
            "§ 7491(c) — Secretary bears BURDEN OF PRODUCTION (not persuasion) for any penalty or addition to tax including § 6651, § 6662, § 6663, § 6672; IRS must come forward with evidence before taxpayer must rebut"
                .to_string(),
        );
    }

    notes.push(
        "§ 7491 enacted under IRS Restructuring and Reform Act of 1998 (Pub. L. No. 105-206) as part of taxpayer-protection package; cross-references § 7454(a) fraud + accumulated earnings burden + § 6664(c) reasonable cause defense"
            .to_string(),
    );

    Section7491Result {
        subsection_a_1_burden_shift_engaged: a_1_engaged,
        subsection_b_statistical_burden_engaged: b_engaged,
        subsection_c_penalty_production_burden_engaged: c_engaged,
        net_worth_limitation_engaged,
        any_burden_shift_engaged: any_engaged,
        citation: "IRC §§ 7491(a)(1), 7491(a)(2)(A), 7491(a)(2)(B), 7491(a)(2)(C), 7491(b), 7491(c); IRS Restructuring and Reform Act of 1998 (Pub. L. No. 105-206); cross-reference § 7454(a), § 6664(c)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn individual_base() -> Section7491Input {
        Section7491Input {
            tax_type: TaxType::IncomeOrSubtitleA,
            entity_type: EntityType::Individual,
            net_worth_cents: 100_000_000,
            taxpayer_introduced_credible_evidence: true,
            taxpayer_substantiation_complied: true,
            taxpayer_maintained_records: true,
            taxpayer_cooperated_reasonably: true,
            irs_used_statistical_reconstruction: false,
            irs_seeks_penalty_or_addition: false,
        }
    }

    #[test]
    fn individual_full_compliance_burden_shifts() {
        let r = check(&individual_base());
        assert!(r.subsection_a_1_burden_shift_engaged);
        assert!(r.any_burden_shift_engaged);
        assert!(!r.net_worth_limitation_engaged);
    }

    #[test]
    fn missing_credible_evidence_no_shift() {
        let mut i = individual_base();
        i.taxpayer_introduced_credible_evidence = false;
        let r = check(&i);
        assert!(!r.subsection_a_1_burden_shift_engaged);
    }

    #[test]
    fn missing_substantiation_no_shift() {
        let mut i = individual_base();
        i.taxpayer_substantiation_complied = false;
        let r = check(&i);
        assert!(!r.subsection_a_1_burden_shift_engaged);
        assert!(r.notes.iter().any(|n| n.contains("§ 7491(a)(2)") && n.contains("missing any one")));
    }

    #[test]
    fn missing_records_no_shift() {
        let mut i = individual_base();
        i.taxpayer_maintained_records = false;
        let r = check(&i);
        assert!(!r.subsection_a_1_burden_shift_engaged);
    }

    #[test]
    fn missing_cooperation_no_shift() {
        let mut i = individual_base();
        i.taxpayer_cooperated_reasonably = false;
        let r = check(&i);
        assert!(!r.subsection_a_1_burden_shift_engaged);
    }

    #[test]
    fn three_threshold_conjunctive_truth_table() {
        for s in [false, true] {
            for r_records in [false, true] {
                for c in [false, true] {
                    let mut i = individual_base();
                    i.taxpayer_substantiation_complied = s;
                    i.taxpayer_maintained_records = r_records;
                    i.taxpayer_cooperated_reasonably = c;
                    let result = check(&i);
                    assert_eq!(
                        result.subsection_a_1_burden_shift_engaged,
                        s && r_records && c
                    );
                }
            }
        }
    }

    #[test]
    fn corporation_net_worth_above_7m_limitation_engaged() {
        let mut i = individual_base();
        i.entity_type = EntityType::Corporation;
        i.net_worth_cents = 700_000_001;
        let r = check(&i);
        assert!(r.net_worth_limitation_engaged);
        assert!(!r.subsection_a_1_burden_shift_engaged);
        assert!(r.notes.iter().any(|n| n.contains("§ 7491(a)(2)(C)") && n.contains("$7,000,000")));
    }

    #[test]
    fn corporation_net_worth_at_7m_boundary_no_limitation() {
        let mut i = individual_base();
        i.entity_type = EntityType::Corporation;
        i.net_worth_cents = 700_000_000;
        let r = check(&i);
        assert!(!r.net_worth_limitation_engaged);
        assert!(r.subsection_a_1_burden_shift_engaged);
    }

    #[test]
    fn partnership_net_worth_above_7m_limitation_engaged() {
        let mut i = individual_base();
        i.entity_type = EntityType::Partnership;
        i.net_worth_cents = 1_000_000_000;
        let r = check(&i);
        assert!(r.net_worth_limitation_engaged);
    }

    #[test]
    fn trust_net_worth_above_7m_limitation_engaged() {
        let mut i = individual_base();
        i.entity_type = EntityType::Trust;
        i.net_worth_cents = 1_000_000_000;
        let r = check(&i);
        assert!(r.net_worth_limitation_engaged);
    }

    #[test]
    fn individual_unlimited_by_net_worth() {
        let mut i = individual_base();
        i.entity_type = EntityType::Individual;
        i.net_worth_cents = 100_000_000_000;
        let r = check(&i);
        assert!(!r.net_worth_limitation_engaged);
        assert!(r.subsection_a_1_burden_shift_engaged);
    }

    #[test]
    fn estate_not_subject_to_7m_limitation() {
        let mut i = individual_base();
        i.entity_type = EntityType::Estate;
        i.tax_type = TaxType::EstateGiftGstSubtitleB;
        i.net_worth_cents = 100_000_000_000;
        let r = check(&i);
        assert!(!r.net_worth_limitation_engaged);
    }

    #[test]
    fn employment_tax_outside_burden_shift_scope() {
        let mut i = individual_base();
        i.tax_type = TaxType::EmploymentSubtitleC;
        let r = check(&i);
        assert!(!r.subsection_a_1_burden_shift_engaged);
        assert!(r.notes.iter().any(|n| n.contains("§ 7491(a)") && n.contains("employment + excise taxes outside")));
    }

    #[test]
    fn excise_tax_outside_burden_shift_scope() {
        let mut i = individual_base();
        i.tax_type = TaxType::ExciseSubtitleD;
        let r = check(&i);
        assert!(!r.subsection_a_1_burden_shift_engaged);
    }

    #[test]
    fn estate_gift_gst_within_burden_shift_scope() {
        let mut i = individual_base();
        i.tax_type = TaxType::EstateGiftGstSubtitleB;
        i.entity_type = EntityType::Estate;
        let r = check(&i);
        assert!(r.subsection_a_1_burden_shift_engaged);
    }

    #[test]
    fn statistical_reconstruction_individual_subtitle_a_engages_b() {
        let mut i = individual_base();
        i.irs_used_statistical_reconstruction = true;
        let r = check(&i);
        assert!(r.subsection_b_statistical_burden_engaged);
        assert!(r.notes.iter().any(|n| n.contains("§ 7491(b)") && n.contains("BLS surveys")));
    }

    #[test]
    fn statistical_reconstruction_corporation_not_engaged() {
        let mut i = individual_base();
        i.entity_type = EntityType::Corporation;
        i.net_worth_cents = 1_000_000;
        i.irs_used_statistical_reconstruction = true;
        let r = check(&i);
        assert!(!r.subsection_b_statistical_burden_engaged);
        assert!(r.notes.iter().any(|n| n.contains("§ 7491(b)") && n.contains("INDIVIDUAL taxpayers")));
    }

    #[test]
    fn statistical_reconstruction_estate_subtitle_b_not_engaged() {
        let mut i = individual_base();
        i.tax_type = TaxType::EstateGiftGstSubtitleB;
        i.entity_type = EntityType::Estate;
        i.irs_used_statistical_reconstruction = true;
        let r = check(&i);
        assert!(!r.subsection_b_statistical_burden_engaged);
    }

    #[test]
    fn penalty_production_burden_engaged() {
        let mut i = individual_base();
        i.irs_seeks_penalty_or_addition = true;
        let r = check(&i);
        assert!(r.subsection_c_penalty_production_burden_engaged);
        assert!(r.notes.iter().any(|n| n.contains("§ 7491(c)") && n.contains("BURDEN OF PRODUCTION")));
    }

    #[test]
    fn penalty_production_burden_not_engaged_when_no_penalty() {
        let r = check(&individual_base());
        assert!(!r.subsection_c_penalty_production_burden_engaged);
    }

    #[test]
    fn penalty_production_burden_independent_of_other_subsections() {
        let mut i = individual_base();
        i.taxpayer_introduced_credible_evidence = false;
        i.irs_seeks_penalty_or_addition = true;
        let r = check(&i);
        assert!(!r.subsection_a_1_burden_shift_engaged);
        assert!(r.subsection_c_penalty_production_burden_engaged);
    }

    #[test]
    fn any_burden_shift_engaged_when_any_subsection_engaged() {
        let mut i_a = individual_base();
        assert!(check(&i_a).any_burden_shift_engaged);

        i_a.taxpayer_introduced_credible_evidence = false;
        i_a.irs_used_statistical_reconstruction = true;
        assert!(check(&i_a).any_burden_shift_engaged);

        i_a.irs_used_statistical_reconstruction = false;
        i_a.irs_seeks_penalty_or_addition = true;
        assert!(check(&i_a).any_burden_shift_engaged);
    }

    #[test]
    fn any_burden_shift_not_engaged_when_no_subsection() {
        let mut i = individual_base();
        i.taxpayer_introduced_credible_evidence = false;
        let r = check(&i);
        assert!(!r.any_burden_shift_engaged);
    }

    #[test]
    fn credible_evidence_note_describes_post_critical_analysis() {
        let r = check(&individual_base());
        assert!(r.notes.iter().any(|n| n.contains("credible evidence") && n.contains("court would find sufficient")));
    }

    #[test]
    fn three_threshold_satisfied_note_lists_a_b_c() {
        let r = check(&individual_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7491(a)(2)") && n.contains("(A) substantiation") && n.contains("(B) records") && n.contains("(C) cooperation")));
    }

    #[test]
    fn rra_1998_enactment_note_present() {
        let r = check(&individual_base());
        assert!(r.notes.iter().any(|n| n.contains("IRS Restructuring and Reform Act of 1998") && n.contains("Pub. L. No. 105-206")));
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = check(&individual_base());
        assert!(r.citation.contains("§§ 7491(a)(1)"));
        assert!(r.citation.contains("7491(a)(2)(A)"));
        assert!(r.citation.contains("7491(a)(2)(B)"));
        assert!(r.citation.contains("7491(a)(2)(C)"));
        assert!(r.citation.contains("7491(b)"));
        assert!(r.citation.contains("7491(c)"));
        assert!(r.citation.contains("Pub. L. No. 105-206"));
        assert!(r.citation.contains("§ 7454(a)"));
        assert!(r.citation.contains("§ 6664(c)"));
    }

    #[test]
    fn three_subsections_independent_routing() {
        for credible in [false, true] {
            for stat in [false, true] {
                for pen in [false, true] {
                    let mut i = individual_base();
                    i.taxpayer_introduced_credible_evidence = credible;
                    i.irs_used_statistical_reconstruction = stat;
                    i.irs_seeks_penalty_or_addition = pen;
                    let r = check(&i);
                    assert_eq!(r.subsection_a_1_burden_shift_engaged, credible);
                    assert_eq!(r.subsection_b_statistical_burden_engaged, stat);
                    assert_eq!(r.subsection_c_penalty_production_burden_engaged, pen);
                }
            }
        }
    }

    #[test]
    fn net_worth_limitation_only_for_corp_partnership_trust() {
        for entity in [EntityType::Individual, EntityType::Estate] {
            let mut i = individual_base();
            i.entity_type = entity;
            i.net_worth_cents = 100_000_000_000;
            assert!(!check(&i).net_worth_limitation_engaged);
        }

        for entity in [EntityType::Corporation, EntityType::Partnership, EntityType::Trust] {
            let mut i = individual_base();
            i.entity_type = entity;
            i.net_worth_cents = 700_000_001;
            assert!(check(&i).net_worth_limitation_engaged);
        }
    }

    #[test]
    fn corp_with_low_net_worth_can_engage_a_1_shift() {
        let mut i = individual_base();
        i.entity_type = EntityType::Corporation;
        i.net_worth_cents = 100_000_000;
        let r = check(&i);
        assert!(r.subsection_a_1_burden_shift_engaged);
        assert!(!r.net_worth_limitation_engaged);
    }
}
