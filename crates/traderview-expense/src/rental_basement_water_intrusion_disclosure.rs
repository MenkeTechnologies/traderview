//! Rental property basement water intrusion / mold
//! disclosure compliance — when a trader-landlord must
//! disclose water intrusion, flood history, mold visible
//! evidence, and remediation history to prospective and
//! existing tenants. Trader-landlord operational concern:
//! undisclosed water intrusion creates breach of warranty
//! of habitability + constructive eviction + per-tenant
//! statutory damages + multi-tenant class action exposure
//! when mold-related health claims arise. Distinct from
//! siblings `mold_disclosure` (general mold framework),
//! `flood_disclosure` (flood zone), `rental_bedroom_egress_
//! window` (structural), `rental_hot_water_temperature`
//! (habitability minimums).
//!
//! **Four regimes**:
//!
//! **Maryland — Maryland Tenant Mold Protection Act (eff.
//! July 1, 2025) + MD Real Property § 8-211 + § 8-211.1**:
//! - Landlord must disclose **visible mold + water
//!   intrusion + documented remediation history in
//!   writing BEFORE move-in**.
//! - Must provide **mold information pamphlet** at lease
//!   signing.
//! - **15-day mold assessment deadline** from receipt of
//!   tenant written notice.
//! - **45-day remediation completion deadline** from
//!   assessment.
//! - Violations create breach of warranty of habitability
//!   + tenant rescission right + statutory damages.
//!
//! **Virginia — Va. Code § 55.1-1220 + § 8.01-226.12 +
//! § 55.1-1216 (move-in inspection report)**:
//! - Landlord must provide **itemized move-in inspection
//!   report within 5 days** of move-in.
//! - Report must disclose **visible mold** in readily
//!   accessible interior areas.
//! - § 55.1-1220 — landlord MUST take steps to **prevent
//!   moisture accumulation and mold growth**.
//! - If report discloses mold, tenant may either (a)
//!   terminate tenancy OR (b) stay; if tenant stays,
//!   landlord must **remediate within 5 days** of decision.
//! - § 8.01-226.12 — landlord duty regarding visible mold.
//!
//! **New York — Property Condition Disclosure Act + NYC
//! Local Law 55 of 2018 (NYC Admin Code § 27-2017) + NY
//! Property Condition Disclosure Statement (PCDS)**:
//! - Property Condition Disclosure Act requires
//!   disclosure of past or present mold problems on sale
//!   and rental.
//! - **Natural flood event disclosure** required (NY GOL §
//!   5-905); plumbing/mechanical floods NOT voluntarily
//!   required (but tenant may rescind for material
//!   nondisclosure).
//! - NYC Local Law 55 of 2018 — buildings with **more
//!   than one unit** must keep dwellings free of allergens
//!   including mold; **annual inspection required**;
//!   correct remediation procedures + tenant notification
//!   on completion.
//! - Indoor allergen hazard reduction protocols required
//!   under NYC Admin Code § 27-2017.
//!
//! **Default — common-law implied warranty of habitability
//! + state landlord-tenant statutes**:
//! - No specific statewide water intrusion / mold
//!   disclosure mandate.
//! - General implied warranty of habitability covers
//!   uninhabitable conditions including mold/flooding.
//! - Federal Fair Housing Act + ADA may apply for
//!   tenants with mold-induced respiratory conditions.
//! - EPA + CDC mold remediation guidance provides best
//!   practices.
//!
//! Citations: Maryland Tenant Mold Protection Act of 2024
//! (eff. July 1, 2025); MD Real Property § 8-211 + § 8-
//! 211.1; Va. Code § 55.1-1220 + § 8.01-226.12 + § 55.1-
//! 1216; NY Property Condition Disclosure Act; NY GOL §
//! 5-905; NYC Local Law 55 of 2018; NYC Admin Code § 27-
//! 2017; EPA Mold Remediation in Schools and Commercial
//! Buildings (EPA 402-K-01-001); CDC Stachybotrys
//! Information for Clinicians.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Maryland,
    Virginia,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalBasementWaterIntrusionDisclosureInput {
    pub regime: Regime,
    /// Whether visible mold + water intrusion + remediation
    /// history was disclosed in writing before move-in.
    pub pre_move_in_written_disclosure: bool,
    /// Whether mold information pamphlet was provided (MD).
    pub mold_information_pamphlet_provided: bool,
    /// Days since tenant gave written notice of mold (MD 15-
    /// day mold assessment window).
    pub days_since_tenant_mold_notice: u32,
    /// Whether mold assessment was completed within 15 days
    /// (MD).
    pub assessment_completed_within_15_days: bool,
    /// Days since mold assessment was completed (MD 45-day
    /// remediation completion window).
    pub days_since_mold_assessment: u32,
    /// Whether remediation was completed within 45 days (MD).
    pub remediation_completed_within_45_days: bool,
    /// Whether move-in inspection report was provided within
    /// 5 days (VA).
    pub va_5_day_move_in_report_provided: bool,
    /// Whether VA report disclosed visible mold (if present).
    pub va_visible_mold_disclosed_in_report: bool,
    /// Whether mold is present in unit (VA disclosure trigger).
    pub mold_present_in_unit: bool,
    /// Whether tenant elected to stay after VA mold disclosure
    /// (VA 5-day remediation trigger).
    pub va_tenant_elected_to_stay: bool,
    /// Days since VA tenant elected to stay (VA 5-day
    /// remediation window).
    pub va_days_since_tenant_stay_election: u32,
    /// Whether natural flood event history disclosed (NY).
    pub ny_natural_flood_history_disclosed: bool,
    /// Whether NY Property Condition Disclosure Statement
    /// disclosed mold.
    pub ny_pcds_mold_disclosed: bool,
    /// Whether NYC Local Law 55 annual inspection was
    /// performed (NYC).
    pub nyc_ll55_annual_inspection_performed: bool,
    /// Whether building is NYC multi-unit (NYC LL55 scope).
    pub nyc_multi_unit_building: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalBasementWaterIntrusionDisclosureResult {
    pub disclosure_compliant: bool,
    pub assessment_deadline_compliant: bool,
    pub remediation_deadline_compliant: bool,
    pub move_in_report_compliant: bool,
    pub visible_mold_disclosure_compliant: bool,
    pub annual_inspection_compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(
    input: &RentalBasementWaterIntrusionDisclosureInput,
) -> RentalBasementWaterIntrusionDisclosureResult {
    match input.regime {
        Regime::Maryland => check_md(input),
        Regime::Virginia => check_va(input),
        Regime::NewYork => check_ny(input),
        Regime::Default => check_default(input),
    }
}

fn check_md(
    input: &RentalBasementWaterIntrusionDisclosureInput,
) -> RentalBasementWaterIntrusionDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Maryland Tenant Mold Protection Act of 2024 (eff. July 1, 2025) — landlord must disclose visible mold + water intrusion + documented remediation history in writing BEFORE move-in".to_string(),
        "MD Tenant Mold Protection Act — mold information pamphlet must be provided at lease signing".to_string(),
        "MD Tenant Mold Protection Act — 15-day mold assessment deadline from receipt of tenant written notice; 45-day remediation completion deadline from assessment".to_string(),
        "MD Real Property § 8-211 + § 8-211.1 — breach of warranty of habitability + tenant rescission right + statutory damages for violations".to_string(),
        "Maryland regime distinct from common-law warranty regimes; statutory affirmative pre-move-in disclosure required".to_string(),
    ];

    if !input.pre_move_in_written_disclosure {
        violations.push(
            "Maryland Tenant Mold Protection Act — visible mold + water intrusion + remediation history must be disclosed in writing BEFORE move-in".to_string(),
        );
    }

    if !input.mold_information_pamphlet_provided {
        violations.push(
            "Maryland Tenant Mold Protection Act — mold information pamphlet must be provided at lease signing".to_string(),
        );
    }

    let assessment_compliant = !input.days_since_tenant_mold_notice > 0
        || input.assessment_completed_within_15_days
        || input.days_since_tenant_mold_notice <= 15;

    if input.days_since_tenant_mold_notice > 15 && !input.assessment_completed_within_15_days {
        violations.push(
            "Maryland Tenant Mold Protection Act — mold assessment must be completed within 15 days of receipt of tenant written notice".to_string(),
        );
    }

    let remediation_compliant = input.days_since_mold_assessment == 0
        || input.remediation_completed_within_45_days
        || input.days_since_mold_assessment <= 45;

    if input.days_since_mold_assessment > 45 && !input.remediation_completed_within_45_days {
        violations.push(
            "Maryland Tenant Mold Protection Act — remediation must be completed within 45 days of mold assessment".to_string(),
        );
    }

    RentalBasementWaterIntrusionDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        assessment_deadline_compliant: assessment_compliant,
        remediation_deadline_compliant: remediation_compliant,
        move_in_report_compliant: true,
        visible_mold_disclosure_compliant: input.pre_move_in_written_disclosure,
        annual_inspection_compliant: true,
        violations,
        citation: "Maryland Tenant Mold Protection Act of 2024 (eff. July 1, 2025); MD Real Property § 8-211 + § 8-211.1",
        notes,
    }
}

fn check_va(
    input: &RentalBasementWaterIntrusionDisclosureInput,
) -> RentalBasementWaterIntrusionDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Va. Code § 55.1-1216 — landlord must provide itemized move-in inspection report within 5 days of move-in".to_string(),
        "Va. Code § 55.1-1220 — landlord must take steps to prevent moisture accumulation and mold growth in dwelling".to_string(),
        "Va. Code § 8.01-226.12 — landlord duty regarding visible mold; report must disclose visible mold in readily accessible interior areas".to_string(),
        "Va. Code § 55.1-1216 — if report discloses mold, tenant may either (a) terminate tenancy OR (b) stay; if stay, landlord must remediate within 5 days of tenant decision".to_string(),
        "VA regime distinct from MD in shorter remediation window (5 days vs MD 45 days) but only triggered by tenant election after disclosure".to_string(),
    ];

    if !input.va_5_day_move_in_report_provided {
        violations.push(
            "Va. Code § 55.1-1216 — itemized move-in inspection report must be provided within 5 days of move-in".to_string(),
        );
    }

    if input.mold_present_in_unit && !input.va_visible_mold_disclosed_in_report {
        violations.push(
            "Va. Code § 55.1-1216 + § 8.01-226.12 — move-in inspection report must disclose visible mold in readily accessible interior areas".to_string(),
        );
    }

    let va_remediation_compliant = !input.va_tenant_elected_to_stay
        || input.va_days_since_tenant_stay_election <= 5;

    if input.va_tenant_elected_to_stay && input.va_days_since_tenant_stay_election > 5 {
        violations.push(
            "Va. Code § 55.1-1216 — if tenant elects to stay after mold disclosure, landlord must remediate within 5 days of tenant decision".to_string(),
        );
    }

    RentalBasementWaterIntrusionDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        assessment_deadline_compliant: true,
        remediation_deadline_compliant: va_remediation_compliant,
        move_in_report_compliant: input.va_5_day_move_in_report_provided,
        visible_mold_disclosure_compliant: !input.mold_present_in_unit
            || input.va_visible_mold_disclosed_in_report,
        annual_inspection_compliant: true,
        violations,
        citation: "Va. Code § 55.1-1220 + § 8.01-226.12 + § 55.1-1216",
        notes,
    }
}

fn check_ny(
    input: &RentalBasementWaterIntrusionDisclosureInput,
) -> RentalBasementWaterIntrusionDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "NY Property Condition Disclosure Act — disclosure of past or present mold problems required on sale and rental".to_string(),
        "NY GOL § 5-905 — natural flood event history disclosure required; plumbing/mechanical floods NOT voluntarily required (but tenant may rescind for material nondisclosure)".to_string(),
        "NYC Local Law 55 of 2018 + NYC Admin Code § 27-2017 — buildings with more than one unit must keep dwellings free of allergens including mold; annual inspection required".to_string(),
        "NYC LL55 — correct remediation procedures + tenant notification on completion of remediation".to_string(),
        "NYC LL55 indoor allergen hazard reduction protocols required under NYC Admin Code § 27-2017".to_string(),
    ];

    if !input.ny_pcds_mold_disclosed {
        violations.push(
            "NY Property Condition Disclosure Act — past or present mold problems must be disclosed on Property Condition Disclosure Statement (PCDS)".to_string(),
        );
    }

    if !input.ny_natural_flood_history_disclosed {
        violations.push(
            "NY GOL § 5-905 — natural flood event history must be disclosed".to_string(),
        );
    }

    let ll55_engaged = input.nyc_multi_unit_building;
    if ll55_engaged && !input.nyc_ll55_annual_inspection_performed {
        violations.push(
            "NYC Local Law 55 of 2018 + NYC Admin Code § 27-2017 — annual inspection required in buildings with more than one unit".to_string(),
        );
    }

    RentalBasementWaterIntrusionDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        assessment_deadline_compliant: true,
        remediation_deadline_compliant: true,
        move_in_report_compliant: true,
        visible_mold_disclosure_compliant: input.ny_pcds_mold_disclosed,
        annual_inspection_compliant: !ll55_engaged
            || input.nyc_ll55_annual_inspection_performed,
        violations,
        citation: "NY Property Condition Disclosure Act; NY GOL § 5-905; NYC Local Law 55 of 2018; NYC Admin Code § 27-2017",
        notes,
    }
}

fn check_default(
    input: &RentalBasementWaterIntrusionDisclosureInput,
) -> RentalBasementWaterIntrusionDisclosureResult {
    let violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Default — no specific statewide water intrusion or mold disclosure mandate; common-law implied warranty of habitability covers uninhabitable conditions including mold/flooding".to_string(),
        "Federal Fair Housing Act + ADA may apply for tenants with mold-induced respiratory conditions or other disabilities".to_string(),
        "EPA Mold Remediation in Schools and Commercial Buildings (EPA 402-K-01-001) provides remediation best practices".to_string(),
        "CDC Stachybotrys Information for Clinicians provides health guidance".to_string(),
        "Default — verify local jurisdiction landlord-tenant statutes and municipal ordinances for specific disclosure mandates".to_string(),
    ];

    let _ = input;

    RentalBasementWaterIntrusionDisclosureResult {
        disclosure_compliant: true,
        assessment_deadline_compliant: true,
        remediation_deadline_compliant: true,
        move_in_report_compliant: true,
        visible_mold_disclosure_compliant: true,
        annual_inspection_compliant: true,
        violations,
        citation: "Default common-law warranty of habitability + state landlord-tenant statutes + Fair Housing Act + ADA + EPA Mold Remediation Guidance + CDC Stachybotrys",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn md_clean() -> RentalBasementWaterIntrusionDisclosureInput {
        RentalBasementWaterIntrusionDisclosureInput {
            regime: Regime::Maryland,
            pre_move_in_written_disclosure: true,
            mold_information_pamphlet_provided: true,
            days_since_tenant_mold_notice: 0,
            assessment_completed_within_15_days: true,
            days_since_mold_assessment: 0,
            remediation_completed_within_45_days: true,
            va_5_day_move_in_report_provided: false,
            va_visible_mold_disclosed_in_report: false,
            mold_present_in_unit: false,
            va_tenant_elected_to_stay: false,
            va_days_since_tenant_stay_election: 0,
            ny_natural_flood_history_disclosed: false,
            ny_pcds_mold_disclosed: false,
            nyc_ll55_annual_inspection_performed: false,
            nyc_multi_unit_building: false,
        }
    }

    fn va_clean() -> RentalBasementWaterIntrusionDisclosureInput {
        let mut i = md_clean();
        i.regime = Regime::Virginia;
        i.va_5_day_move_in_report_provided = true;
        i
    }

    fn ny_clean() -> RentalBasementWaterIntrusionDisclosureInput {
        let mut i = md_clean();
        i.regime = Regime::NewYork;
        i.ny_pcds_mold_disclosed = true;
        i.ny_natural_flood_history_disclosed = true;
        i
    }

    fn default_clean() -> RentalBasementWaterIntrusionDisclosureInput {
        let mut i = md_clean();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn md_clean_compliant() {
        let r = check(&md_clean());
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn md_no_pre_move_in_disclosure_violation() {
        let mut i = md_clean();
        i.pre_move_in_written_disclosure = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Mold Protection Act") && v.contains("BEFORE move-in")));
    }

    #[test]
    fn md_no_mold_pamphlet_violation() {
        let mut i = md_clean();
        i.mold_information_pamphlet_provided = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("mold information pamphlet")));
    }

    #[test]
    fn md_assessment_at_15_day_boundary_compliant() {
        let mut i = md_clean();
        i.days_since_tenant_mold_notice = 15;
        i.assessment_completed_within_15_days = true;
        let r = check(&i);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn md_assessment_at_16_days_without_completion_violation() {
        let mut i = md_clean();
        i.days_since_tenant_mold_notice = 16;
        i.assessment_completed_within_15_days = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("15 days") && v.contains("assessment")));
    }

    #[test]
    fn md_remediation_at_45_day_boundary_compliant() {
        let mut i = md_clean();
        i.days_since_mold_assessment = 45;
        i.remediation_completed_within_45_days = true;
        let r = check(&i);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn md_remediation_at_46_days_without_completion_violation() {
        let mut i = md_clean();
        i.days_since_mold_assessment = 46;
        i.remediation_completed_within_45_days = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("45 days") && v.contains("remediation")));
    }

    #[test]
    fn va_clean_compliant() {
        let r = check(&va_clean());
        assert!(r.disclosure_compliant);
        assert!(r.move_in_report_compliant);
    }

    #[test]
    fn va_no_move_in_report_violation() {
        let mut i = va_clean();
        i.va_5_day_move_in_report_provided = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 55.1-1216") && v.contains("5 days")));
    }

    #[test]
    fn va_mold_present_but_not_disclosed_violation() {
        let mut i = va_clean();
        i.mold_present_in_unit = true;
        i.va_visible_mold_disclosed_in_report = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 8.01-226.12") && v.contains("visible mold")));
    }

    #[test]
    fn va_mold_present_disclosed_compliant() {
        let mut i = va_clean();
        i.mold_present_in_unit = true;
        i.va_visible_mold_disclosed_in_report = true;
        let r = check(&i);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn va_tenant_stay_election_5_day_remediation_compliant() {
        let mut i = va_clean();
        i.va_tenant_elected_to_stay = true;
        i.va_days_since_tenant_stay_election = 5;
        let r = check(&i);
        assert!(r.remediation_deadline_compliant);
    }

    #[test]
    fn va_tenant_stay_election_6_day_remediation_violation() {
        let mut i = va_clean();
        i.va_tenant_elected_to_stay = true;
        i.va_days_since_tenant_stay_election = 6;
        let r = check(&i);
        assert!(!r.remediation_deadline_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("5 days") && v.contains("tenant decision")));
    }

    #[test]
    fn ny_clean_compliant() {
        let r = check(&ny_clean());
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn ny_no_pcds_mold_disclosure_violation() {
        let mut i = ny_clean();
        i.ny_pcds_mold_disclosed = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Property Condition Disclosure Act")));
    }

    #[test]
    fn ny_no_flood_history_disclosure_violation() {
        let mut i = ny_clean();
        i.ny_natural_flood_history_disclosed = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 5-905")));
    }

    #[test]
    fn nyc_ll55_multi_unit_no_annual_inspection_violation() {
        let mut i = ny_clean();
        i.nyc_multi_unit_building = true;
        i.nyc_ll55_annual_inspection_performed = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Local Law 55") && v.contains("annual inspection")));
    }

    #[test]
    fn nyc_ll55_single_unit_no_annual_inspection_no_violation() {
        let mut i = ny_clean();
        i.nyc_multi_unit_building = false;
        i.nyc_ll55_annual_inspection_performed = false;
        let r = check(&i);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn default_no_violations() {
        let r = check(&default_clean());
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn citation_pins_md_authority() {
        let r = check(&md_clean());
        assert!(r.citation.contains("Maryland Tenant Mold Protection Act"));
        assert!(r.citation.contains("§ 8-211"));
    }

    #[test]
    fn citation_pins_va_authority() {
        let r = check(&va_clean());
        assert!(r.citation.contains("§ 55.1-1220"));
        assert!(r.citation.contains("§ 8.01-226.12"));
        assert!(r.citation.contains("§ 55.1-1216"));
    }

    #[test]
    fn citation_pins_ny_authority() {
        let r = check(&ny_clean());
        assert!(r.citation.contains("Property Condition Disclosure Act"));
        assert!(r.citation.contains("§ 5-905"));
        assert!(r.citation.contains("Local Law 55"));
        assert!(r.citation.contains("§ 27-2017"));
    }

    #[test]
    fn citation_pins_default_authority() {
        let r = check(&default_clean());
        assert!(r.citation.contains("warranty of habitability"));
        assert!(r.citation.contains("EPA Mold Remediation"));
    }

    #[test]
    fn note_pins_md_july_2025_effective_date() {
        let r = check(&md_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("July 1, 2025") && n.contains("Mold Protection Act")));
    }

    #[test]
    fn note_pins_md_15_45_day_deadlines() {
        let r = check(&md_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("15-day") && n.contains("45-day")));
    }

    #[test]
    fn note_pins_va_5_day_move_in_report() {
        let r = check(&va_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 55.1-1216") && n.contains("5 days")));
    }

    #[test]
    fn note_pins_va_5_day_remediation_after_tenant_stay() {
        let r = check(&va_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 55.1-1216")
                && n.contains("remediate within 5 days")));
    }

    #[test]
    fn note_pins_ny_natural_vs_mechanical_flood_distinction() {
        let r = check(&ny_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 5-905")
                && n.contains("plumbing/mechanical floods NOT")));
    }

    #[test]
    fn note_pins_nyc_ll55_multi_unit_annual_inspection() {
        let r = check(&ny_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Local Law 55") && n.contains("annual inspection")));
    }

    #[test]
    fn note_pins_default_epa_mold_remediation_guidance() {
        let r = check(&default_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("EPA Mold Remediation") && n.contains("EPA 402-K-01-001")));
    }

    #[test]
    fn md_uniquely_requires_mold_pamphlet_invariant() {
        let mut i_md = md_clean();
        i_md.mold_information_pamphlet_provided = false;
        let r_md = check(&i_md);
        assert!(!r_md.disclosure_compliant);

        let mut i_va = va_clean();
        i_va.mold_information_pamphlet_provided = false;
        let r_va = check(&i_va);
        assert!(r_va.disclosure_compliant);
    }

    #[test]
    fn va_5_day_remediation_uniquely_shorter_than_md_45_invariant() {
        let mut i_va = va_clean();
        i_va.va_tenant_elected_to_stay = true;
        i_va.va_days_since_tenant_stay_election = 10;
        let r_va = check(&i_va);
        assert!(!r_va.remediation_deadline_compliant);

        let mut i_md = md_clean();
        i_md.days_since_mold_assessment = 10;
        let r_md = check(&i_md);
        assert!(r_md.remediation_deadline_compliant);
    }

    #[test]
    fn nyc_ll55_uniquely_requires_annual_inspection_invariant() {
        let mut i_nyc = ny_clean();
        i_nyc.nyc_multi_unit_building = true;
        i_nyc.nyc_ll55_annual_inspection_performed = false;
        let r_nyc = check(&i_nyc);
        assert!(!r_nyc.disclosure_compliant);

        let i_md = md_clean();
        let r_md = check(&i_md);
        assert!(r_md.disclosure_compliant);
        let _ = i_md.nyc_multi_unit_building;
    }

    #[test]
    fn multiple_md_violations_stack() {
        let mut i = md_clean();
        i.pre_move_in_written_disclosure = false;
        i.mold_information_pamphlet_provided = false;
        i.days_since_tenant_mold_notice = 20;
        i.assessment_completed_within_15_days = false;
        let r = check(&i);
        assert!(r.violations.len() >= 3);
    }
}
