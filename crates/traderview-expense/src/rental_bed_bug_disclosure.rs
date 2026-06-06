//! Rental property bed bug disclosure compliance — when a
//! trader-landlord must disclose bed bug infestation history
//! to prospective tenants before lease signing. Trader-
//! landlord operational concern: missed bed bug disclosure
//! exposes landlord to statutory damages + applicant
//! constructive eviction claim + reputational risk in
//! competitive multifamily markets.
//!
//! Distinct from siblings `rental_application_denial_
//! disclosure` (FCRA-style adverse action), `rental_hot_
//! water_temperature` (habitability minimums), `tenant_fire_
//! safety_plan_disclosure` (fire safety), `rental_bedroom_
//! egress_window` (structural).
//!
//! **Four regimes**:
//!
//! **California — Cal. Civ. Code § 1954.603 (AB 551 of
//! 2015, eff. January 1, 2017 / January 1, 2018 for
//! existing tenants)**:
//! - Written notice required to prospective tenants BEFORE
//!   initiating new tenancy (and to existing tenants by July
//!   1, 2017).
//! - Notice in at least 10-point type describing bed bug
//!   appearance, behavior, lifecycle, infestation indicators.
//! - Landlord CANNOT show, rent, or lease vacant unit known
//!   to have current bed bug infestation.
//! - Landlord must notify tenant of inspection findings
//!   within 2 business days.
//!
//! **New York City — NYC Multiple Dwelling Law § 27-2018.1
//! (NYC Admin. Code; HPD bed bug rules)**:
//! - Annual building-wide bed bug report (NYC Form RA-89)
//!   filed with HPD between December 1 and December 31 each
//!   year.
//! - Prior year's filing provided to every new tenant before
//!   lease signing.
//! - Disclosure of any bed bug infestation in the building
//!   within the prior year.
//! - $250 civil penalty for failure to file the annual
//!   report.
//!
//! **Arizona — A.R.S. § 33-1319 (tenant-request-only
//! framework)**:
//! - Landlord must disclose bed bug educational materials
//!   prepared by the State Department of Health Services.
//! - No proactive pre-lease bed bug history disclosure
//!   required — only upon tenant request.
//! - Landlord may not place tenant in a unit known to be
//!   currently infested.
//!
//! **Maine — 14 M.R.S. § 6021-A (strictest single-unit
//! rule)**:
//! - Pre-rental written disclosure of any bed bug
//!   infestation in the prior 12 months in the unit OR
//!   adjacent unit.
//! - Landlord must inspect within 5 days of tenant notice of
//!   suspected infestation.
//! - 24-hour written disclosure of inspection results to
//!   tenant.
//! - Treatment at landlord's expense unless tenant caused
//!   infestation.
//! - $250 to $1,500 per-violation civil penalty.
//!
//! Citations: Cal. Civ. Code § 1954.603 (AB 551 of 2015);
//! NYC Multiple Dwelling Law § 27-2018.1; NYC Admin. Code
//! Title 28; A.R.S. § 33-1319 (Arizona); 14 M.R.S. §
//! 6021-A (Maine).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    NewYorkCity,
    Arizona,
    Maine,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalBedBugDisclosureInput {
    pub regime: Regime,
    /// Whether written pre-lease bed bug disclosure was
    /// provided to prospective tenant.
    pub pre_lease_disclosure_provided: bool,
    /// Whether disclosure describes bed bug appearance,
    /// behavior, lifecycle (CA AB 551).
    pub bed_bug_educational_content_included: bool,
    /// Whether annual building-wide bed bug report was filed
    /// with NYC HPD (NYC § 27-2018.1).
    pub annual_hpd_report_filed: bool,
    /// Whether prior year's HPD report was provided to new
    /// tenant (NYC).
    pub prior_year_hpd_report_provided: bool,
    /// Whether tenant requested bed bug educational materials
    /// (Arizona § 33-1319).
    pub tenant_requested_az_disclosure: bool,
    /// Whether unit has known current infestation.
    pub unit_currently_infested: bool,
    /// Whether unit was rented or shown despite known current
    /// infestation.
    pub rented_despite_known_infestation: bool,
    /// Whether bed bug infestation occurred in prior 12
    /// months (Maine 14 M.R.S. § 6021-A).
    pub infestation_in_prior_12_months: bool,
    /// Days since tenant gave notice of suspected
    /// infestation (Maine inspection deadline).
    pub days_since_tenant_inspection_notice: u32,
    /// Days since landlord conducted inspection without
    /// disclosing result (Maine 24-hour deadline).
    pub hours_since_inspection_without_disclosure: u32,
    /// Maine per-violation penalty amount in cents ($250
    /// to $1,500 range).
    pub maine_penalty_assessed_cents: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalBedBugDisclosureResult {
    pub disclosure_compliant: bool,
    pub pre_lease_disclosure_required: bool,
    pub annual_hpd_report_required: bool,
    pub inspection_deadline_engaged: bool,
    pub inspection_disclosure_deadline_engaged: bool,
    pub maine_penalty_in_statutory_range: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &RentalBedBugDisclosureInput) -> RentalBedBugDisclosureResult {
    match input.regime {
        Regime::California => check_ca(input),
        Regime::NewYorkCity => check_nyc(input),
        Regime::Arizona => check_az(input),
        Regime::Maine => check_me(input),
    }
}

fn check_ca(input: &RentalBedBugDisclosureInput) -> RentalBedBugDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Cal. Civ. Code § 1954.603(a) — landlord must furnish written notice describing bed bug appearance, behavior, lifecycle, and infestation indicators to prospective tenants before initiating new tenancy".to_string(),
        "Cal. Civ. Code § 1954.603 — written notice required in at least 10-point type; AB 551 of 2015 effective January 1, 2017 (new tenants) and July 1, 2017 (existing tenants)".to_string(),
        "Cal. Civ. Code § 1954.603(b) — landlord shall NOT show, rent, or lease vacant unit landlord knows has current bed bug infestation".to_string(),
        "Cal. Civ. Code § 1954.603(c) — landlord must notify tenant of inspection findings within 2 business days of receiving pest control operator report".to_string(),
    ];

    if !input.pre_lease_disclosure_provided {
        violations.push(
            "Cal. Civ. Code § 1954.603(a) — written bed bug disclosure required to prospective tenants before initiating new tenancy".to_string(),
        );
    }

    if !input.bed_bug_educational_content_included {
        violations.push(
            "Cal. Civ. Code § 1954.603 — disclosure must describe bed bug appearance, behavior, lifecycle, infestation indicators in at least 10-point type".to_string(),
        );
    }

    if input.unit_currently_infested && input.rented_despite_known_infestation {
        violations.push(
            "Cal. Civ. Code § 1954.603(b) — landlord prohibited from showing, renting, or leasing vacant unit known to have current bed bug infestation".to_string(),
        );
    }

    RentalBedBugDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        pre_lease_disclosure_required: true,
        annual_hpd_report_required: false,
        inspection_deadline_engaged: false,
        inspection_disclosure_deadline_engaged: false,
        maine_penalty_in_statutory_range: false,
        violations,
        citation: "Cal. Civ. Code § 1954.603 (AB 551 of 2015)",
        notes,
    }
}

fn check_nyc(input: &RentalBedBugDisclosureInput) -> RentalBedBugDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "NYC Multiple Dwelling Law § 27-2018.1 — owner of every multiple dwelling shall file annual building-wide bed bug report (NYC Form RA-89) with HPD between December 1 and December 31 each year".to_string(),
        "NYC § 27-2018.1 — prior year's filing must be provided to every new tenant before lease signing".to_string(),
        "NYC § 27-2018.1 — disclosure of any bed bug infestation in the building within the prior year required".to_string(),
        "NYC § 27-2018.1 — $250 civil penalty for failure to file annual report; tenant remedies include rent abatement and constructive eviction".to_string(),
    ];

    if !input.annual_hpd_report_filed {
        violations.push(
            "NYC Multiple Dwelling Law § 27-2018.1 — annual building-wide bed bug report (Form RA-89) must be filed with HPD between December 1 and December 31".to_string(),
        );
    }

    if !input.prior_year_hpd_report_provided {
        violations.push(
            "NYC § 27-2018.1 — prior year's HPD bed bug filing must be provided to every new tenant before lease signing".to_string(),
        );
    }

    if input.unit_currently_infested && input.rented_despite_known_infestation {
        violations.push(
            "NYC § 27-2018.1 / NYC Admin. Code Title 28 — landlord prohibited from renting unit with known current bed bug infestation".to_string(),
        );
    }

    RentalBedBugDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        pre_lease_disclosure_required: true,
        annual_hpd_report_required: true,
        inspection_deadline_engaged: false,
        inspection_disclosure_deadline_engaged: false,
        maine_penalty_in_statutory_range: false,
        violations,
        citation: "NYC Multiple Dwelling Law § 27-2018.1; NYC Admin. Code Title 28",
        notes,
    }
}

fn check_az(input: &RentalBedBugDisclosureInput) -> RentalBedBugDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "A.R.S. § 33-1319 — Arizona landlord must provide bed bug educational materials prepared by Arizona Department of Health Services upon tenant request only".to_string(),
        "A.R.S. § 33-1319 — no proactive pre-lease bed bug history disclosure required unless tenant requests".to_string(),
        "A.R.S. § 33-1319 — landlord may not place tenant in unit known to be currently infested".to_string(),
        "A.R.S. § 33-1319 — Arizona framework less stringent than CA / NYC / ME; pure tenant-request model".to_string(),
    ];

    if input.tenant_requested_az_disclosure && !input.pre_lease_disclosure_provided {
        violations.push(
            "A.R.S. § 33-1319 — landlord must provide ADHS bed bug educational materials upon tenant request".to_string(),
        );
    }

    if input.unit_currently_infested && input.rented_despite_known_infestation {
        violations.push(
            "A.R.S. § 33-1319 — landlord prohibited from placing tenant in unit known to be currently infested".to_string(),
        );
    }

    RentalBedBugDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        pre_lease_disclosure_required: false,
        annual_hpd_report_required: false,
        inspection_deadline_engaged: false,
        inspection_disclosure_deadline_engaged: false,
        maine_penalty_in_statutory_range: false,
        violations,
        citation: "A.R.S. § 33-1319 (Arizona Residential Landlord and Tenant Act)",
        notes,
    }
}

fn check_me(input: &RentalBedBugDisclosureInput) -> RentalBedBugDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "14 M.R.S. § 6021-A(2) — Maine landlord must disclose in writing any bed bug infestation in the unit or any adjacent unit within the prior 12 months before tenant occupancy".to_string(),
        "14 M.R.S. § 6021-A(3) — landlord must inspect within 5 days of tenant written notice of suspected infestation".to_string(),
        "14 M.R.S. § 6021-A(3) — landlord must disclose inspection results to tenant in writing within 24 hours of inspection completion".to_string(),
        "14 M.R.S. § 6021-A(4) — treatment at landlord's expense unless tenant caused infestation".to_string(),
        "14 M.R.S. § 6021-A(7) — civil penalty $250 to $1,500 per violation".to_string(),
        "Maine 14 M.R.S. § 6021-A — strictest single-unit pre-rental disclosure rule among comparator states; combines pre-lease + inspection deadline + 24-hour disclosure".to_string(),
    ];

    if input.infestation_in_prior_12_months && !input.pre_lease_disclosure_provided {
        violations.push(
            "14 M.R.S. § 6021-A(2) — written pre-rental disclosure required for any bed bug infestation in unit OR adjacent unit within prior 12 months".to_string(),
        );
    }

    if input.unit_currently_infested && input.rented_despite_known_infestation {
        violations.push(
            "14 M.R.S. § 6021-A(4) — Maine warranty of habitability + landlord-expense treatment obligation prohibit renting unit with known current bed bug infestation".to_string(),
        );
    }

    let inspection_deadline_engaged = input.days_since_tenant_inspection_notice > 0;
    if inspection_deadline_engaged && input.days_since_tenant_inspection_notice > 5 {
        violations.push(
            "14 M.R.S. § 6021-A(3) — landlord must inspect within 5 days of tenant written notice of suspected infestation".to_string(),
        );
    }

    let inspection_disclosure_deadline_engaged =
        input.hours_since_inspection_without_disclosure > 0;
    if inspection_disclosure_deadline_engaged
        && input.hours_since_inspection_without_disclosure > 24
    {
        violations.push(
            "14 M.R.S. § 6021-A(3) — landlord must disclose inspection results to tenant in writing within 24 hours of inspection completion".to_string(),
        );
    }

    let me_penalty_in_range: bool = input.maine_penalty_assessed_cents == 0
        || (input.maine_penalty_assessed_cents >= 25_000
            && input.maine_penalty_assessed_cents <= 150_000);

    if !me_penalty_in_range {
        violations.push(
            "14 M.R.S. § 6021-A(7) — civil penalty must be between $250 and $1,500 per violation"
                .to_string(),
        );
    }

    RentalBedBugDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        pre_lease_disclosure_required: true,
        annual_hpd_report_required: false,
        inspection_deadline_engaged,
        inspection_disclosure_deadline_engaged,
        maine_penalty_in_statutory_range: me_penalty_in_range,
        violations,
        citation: "14 M.R.S. § 6021-A (Maine)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_clean() -> RentalBedBugDisclosureInput {
        RentalBedBugDisclosureInput {
            regime: Regime::California,
            pre_lease_disclosure_provided: true,
            bed_bug_educational_content_included: true,
            annual_hpd_report_filed: false,
            prior_year_hpd_report_provided: false,
            tenant_requested_az_disclosure: false,
            unit_currently_infested: false,
            rented_despite_known_infestation: false,
            infestation_in_prior_12_months: false,
            days_since_tenant_inspection_notice: 0,
            hours_since_inspection_without_disclosure: 0,
            maine_penalty_assessed_cents: 0,
        }
    }

    fn nyc_clean() -> RentalBedBugDisclosureInput {
        let mut i = ca_clean();
        i.regime = Regime::NewYorkCity;
        i.annual_hpd_report_filed = true;
        i.prior_year_hpd_report_provided = true;
        i
    }

    fn az_clean() -> RentalBedBugDisclosureInput {
        let mut i = ca_clean();
        i.regime = Regime::Arizona;
        i.pre_lease_disclosure_provided = false;
        i.bed_bug_educational_content_included = false;
        i
    }

    fn me_clean() -> RentalBedBugDisclosureInput {
        let mut i = ca_clean();
        i.regime = Regime::Maine;
        i
    }

    #[test]
    fn ca_clean_disclosure_compliant() {
        let r = check(&ca_clean());
        assert!(r.disclosure_compliant);
        assert!(r.pre_lease_disclosure_required);
    }

    #[test]
    fn ca_no_pre_lease_disclosure_violation() {
        let mut i = ca_clean();
        i.pre_lease_disclosure_provided = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 1954.603(a)")));
    }

    #[test]
    fn ca_missing_educational_content_violation() {
        let mut i = ca_clean();
        i.bed_bug_educational_content_included = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r.violations.iter().any(|v| v.contains("10-point type")));
    }

    #[test]
    fn ca_renting_known_infested_unit_violation() {
        let mut i = ca_clean();
        i.unit_currently_infested = true;
        i.rented_despite_known_infestation = true;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 1954.603(b)")));
    }

    #[test]
    fn ca_known_infested_not_rented_no_violation() {
        let mut i = ca_clean();
        i.unit_currently_infested = true;
        i.rented_despite_known_infestation = false;
        let r = check(&i);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn nyc_clean_disclosure_compliant() {
        let r = check(&nyc_clean());
        assert!(r.disclosure_compliant);
        assert!(r.annual_hpd_report_required);
    }

    #[test]
    fn nyc_missing_annual_hpd_filing_violation() {
        let mut i = nyc_clean();
        i.annual_hpd_report_filed = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 27-2018.1") && v.contains("Form RA-89")));
    }

    #[test]
    fn nyc_missing_prior_year_disclosure_violation() {
        let mut i = nyc_clean();
        i.prior_year_hpd_report_provided = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r.violations.iter().any(|v| v.contains("prior year")));
    }

    #[test]
    fn az_no_tenant_request_no_disclosure_required() {
        let r = check(&az_clean());
        assert!(r.disclosure_compliant);
        assert!(!r.pre_lease_disclosure_required);
    }

    #[test]
    fn az_tenant_requested_disclosure_required() {
        let mut i = az_clean();
        i.tenant_requested_az_disclosure = true;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 33-1319") && v.contains("ADHS")));
    }

    #[test]
    fn az_tenant_requested_disclosure_provided_compliant() {
        let mut i = az_clean();
        i.tenant_requested_az_disclosure = true;
        i.pre_lease_disclosure_provided = true;
        let r = check(&i);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn az_renting_known_infested_violation() {
        let mut i = az_clean();
        i.unit_currently_infested = true;
        i.rented_despite_known_infestation = true;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
    }

    #[test]
    fn me_clean_disclosure_compliant() {
        let r = check(&me_clean());
        assert!(r.disclosure_compliant);
        assert!(r.pre_lease_disclosure_required);
    }

    #[test]
    fn me_prior_12_month_infestation_without_disclosure_violation() {
        let mut i = me_clean();
        i.infestation_in_prior_12_months = true;
        i.pre_lease_disclosure_provided = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 6021-A(2)") && v.contains("12 months")));
    }

    #[test]
    fn me_inspection_within_5_days_compliant() {
        let mut i = me_clean();
        i.days_since_tenant_inspection_notice = 5;
        let r = check(&i);
        assert!(r.disclosure_compliant);
        assert!(r.inspection_deadline_engaged);
    }

    #[test]
    fn me_inspection_at_6_days_violation() {
        let mut i = me_clean();
        i.days_since_tenant_inspection_notice = 6;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 6021-A(3)") && v.contains("5 days")));
    }

    #[test]
    fn me_24_hour_disclosure_compliant() {
        let mut i = me_clean();
        i.hours_since_inspection_without_disclosure = 24;
        let r = check(&i);
        assert!(r.disclosure_compliant);
        assert!(r.inspection_disclosure_deadline_engaged);
    }

    #[test]
    fn me_25_hour_disclosure_violation() {
        let mut i = me_clean();
        i.hours_since_inspection_without_disclosure = 25;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 6021-A(3)") && v.contains("24 hours")));
    }

    #[test]
    fn me_penalty_at_minimum_in_range() {
        let mut i = me_clean();
        i.maine_penalty_assessed_cents = 25_000;
        let r = check(&i);
        assert!(r.maine_penalty_in_statutory_range);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn me_penalty_at_maximum_in_range() {
        let mut i = me_clean();
        i.maine_penalty_assessed_cents = 150_000;
        let r = check(&i);
        assert!(r.maine_penalty_in_statutory_range);
    }

    #[test]
    fn me_penalty_below_minimum_out_of_range_violation() {
        let mut i = me_clean();
        i.maine_penalty_assessed_cents = 24_999;
        let r = check(&i);
        assert!(!r.maine_penalty_in_statutory_range);
        assert!(!r.disclosure_compliant);
    }

    #[test]
    fn me_penalty_above_maximum_out_of_range_violation() {
        let mut i = me_clean();
        i.maine_penalty_assessed_cents = 150_001;
        let r = check(&i);
        assert!(!r.maine_penalty_in_statutory_range);
    }

    #[test]
    fn citation_pins_ca_authority() {
        let r = check(&ca_clean());
        assert!(r.citation.contains("§ 1954.603"));
        assert!(r.citation.contains("AB 551"));
    }

    #[test]
    fn citation_pins_nyc_authority() {
        let r = check(&nyc_clean());
        assert!(r.citation.contains("§ 27-2018.1"));
        assert!(r.citation.contains("Title 28"));
    }

    #[test]
    fn citation_pins_az_authority() {
        let r = check(&az_clean());
        assert!(r.citation.contains("§ 33-1319"));
        assert!(r.citation.contains("Arizona"));
    }

    #[test]
    fn citation_pins_me_authority() {
        let r = check(&me_clean());
        assert!(r.citation.contains("§ 6021-A"));
        assert!(r.citation.contains("Maine"));
    }

    #[test]
    fn nyc_strictest_annual_report_invariant() {
        let r_ca = check(&ca_clean());
        let r_nyc = check(&nyc_clean());
        let r_az = check(&az_clean());
        let r_me = check(&me_clean());
        assert!(r_nyc.annual_hpd_report_required);
        assert!(!r_ca.annual_hpd_report_required);
        assert!(!r_az.annual_hpd_report_required);
        assert!(!r_me.annual_hpd_report_required);
    }

    #[test]
    fn az_least_stringent_no_pre_lease_default() {
        let r_ca = check(&ca_clean());
        let r_nyc = check(&nyc_clean());
        let r_az = check(&az_clean());
        let r_me = check(&me_clean());
        assert!(!r_az.pre_lease_disclosure_required);
        assert!(r_ca.pre_lease_disclosure_required);
        assert!(r_nyc.pre_lease_disclosure_required);
        assert!(r_me.pre_lease_disclosure_required);
    }

    #[test]
    fn me_strictest_inspection_and_disclosure_dual_deadlines_invariant() {
        let mut i = me_clean();
        i.days_since_tenant_inspection_notice = 6;
        i.hours_since_inspection_without_disclosure = 25;
        let r = check(&i);
        assert_eq!(r.violations.len(), 2);
        assert!(r.inspection_deadline_engaged);
        assert!(r.inspection_disclosure_deadline_engaged);
    }

    #[test]
    fn known_infestation_truth_table_all_regimes() {
        for regime in [
            Regime::California,
            Regime::NewYorkCity,
            Regime::Arizona,
            Regime::Maine,
        ] {
            let mut i = match regime {
                Regime::California => ca_clean(),
                Regime::NewYorkCity => nyc_clean(),
                Regime::Arizona => az_clean(),
                Regime::Maine => me_clean(),
            };
            i.unit_currently_infested = true;
            i.rented_despite_known_infestation = true;
            let r = check(&i);
            assert!(!r.disclosure_compliant);
        }
    }

    #[test]
    fn note_pins_ca_ab_551_effective_dates() {
        let r = check(&ca_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("AB 551") && n.contains("January 1, 2017")));
    }

    #[test]
    fn note_pins_nyc_december_filing_window() {
        let r = check(&nyc_clean());
        assert!(r.notes.iter().any(|n| n.contains("December 1")
            && n.contains("December 31")
            && n.contains("Form RA-89")));
    }

    #[test]
    fn note_pins_az_tenant_request_only_model() {
        let r = check(&az_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("tenant-request") || n.contains("tenant request only")));
    }

    #[test]
    fn note_pins_me_civil_penalty_range() {
        let r = check(&me_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("$250") && n.contains("$1,500")));
    }

    #[test]
    fn defensive_maine_penalty_zero_treated_as_compliant() {
        let mut i = me_clean();
        i.maine_penalty_assessed_cents = 0;
        let r = check(&i);
        assert!(r.maine_penalty_in_statutory_range);
        assert!(r.disclosure_compliant);
    }
}
