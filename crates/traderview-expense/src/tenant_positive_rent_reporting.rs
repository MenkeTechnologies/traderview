//! Multi-jurisdictional tenant positive rent reporting
//! (credit-bureau) requirement framework. California
//! AB 2747 of 2024 (codified at Cal. Civ. Code § 1954.07,
//! effective April 1, 2025) mandates that landlords of
//! 16+ unit residential buildings OFFER tenants the
//! option to have their POSITIVE rental payments
//! reported to at least one nationwide consumer
//! reporting agency (Experian, Equifax, TransUnion).
//! Companion to landlord_annual_rent_statement,
//! tenant_data_privacy, rental_application_denial_
//! disclosure, fair_chance_housing.
//!
//! Trader-landlord critical because (1) AB 2747 imposes
//! affirmative offer obligation with annual repetition;
//! (2) $10/month fee cap is strictly enforced; (3)
//! failure to offer creates per-violation civil
//! exposure; (4) misreporting positive vs late payments
//! creates FCRA exposure (15 USC § 1681 et seq.); (5)
//! tenant non-payment of fee CANNOT be cause for
//! termination of tenancy AND CANNOT be deducted from
//! security deposit — both protections distinct from
//! general rent collection rules.
//!
//! **California Civ. Code § 1954.07** (AB 2747 of 2024,
//! **effective April 1, 2025**) — Cal. Civ. Code
//! § 1954.07 mandates that residential landlord of
//! 16+ unit building OFFER tenants the option to have
//! positive rental payment information reported to at
//! least one nationwide consumer reporting agency.
//!
//! **§ 1954.07 building-size threshold + exception**:
//! 1. **General rule**: applies to residential rental
//!    buildings of **16 OR MORE UNITS**.
//! 2. **15-or-fewer-unit exemption** WITH carveout:
//!    landlord of 15-or-fewer-unit building is EXEMPT
//!    UNLESS landlord (a) owns MORE THAN ONE rental
//!    building AND (b) is a REIT, corporation, OR LLC
//!    with at least one corporate member.
//!
//! **§ 1954.07 offer timing**:
//! 1. For leases entered into ON OR AFTER **April 1,
//!    2025**: offer must be made AT THE TIME OF LEASE
//!    AGREEMENT and AT LEAST ONCE ANNUALLY thereafter.
//! 2. For leases OUTSTANDING AS OF January 1, 2025:
//!    offer must be made NO LATER THAN April 1, 2025
//!    and AT LEAST ONCE ANNUALLY thereafter.
//!
//! **§ 1954.07 $10/month fee cap** — landlord MAY charge
//! tenant who elects positive rent reporting a fee NOT
//! EXCEEDING the LESSER OF:
//! 1. Actual cost to landlord of providing the reporting;
//!    OR
//! 2. **$10 PER MONTH**.
//!
//! **§ 1954.07 positive payment definition** — "positive
//! rental payment information" means information
//! regarding tenant's **COMPLETE, TIMELY payments of
//! rent**. Specifically EXCLUDES incomplete or late
//! payments — landlord may NOT report negative
//! information under this statute.
//!
//! **§ 1954.07 tenant non-payment protections**:
//! 1. Tenant's failure to pay the rent-reporting fee
//!    **SHALL NOT BE CAUSE FOR TERMINATION OF TENANCY**;
//! 2. Landlord **SHALL NOT DEDUCT** unpaid fee from
//!    tenant's security deposit;
//! 3. If fee remains unpaid for **30 DAYS OR MORE**,
//!    landlord MAY stop reporting tenant's rental
//!    payments AND tenant is BLOCKED from re-electing
//!    positive rental payment reporting for **6 MONTHS**
//!    from date fee first became due.
//!
//! **Related state expansions**:
//! - Colorado HB 23-1099 — positive rent reporting
//!   requirement for Section 8 Housing Choice Voucher
//!   holders + tenant-elected reporting.
//! - Washington SB 5495 of 2024 — similar
//!   landlord-offer framework for buildings of 5+ units.
//! - HUD pilot program (Tenant Credit Reporting Pilot,
//!   FY 2023-2025) — federally-subsidized housing
//!   participation.
//!
//! **Federal Fair Credit Reporting Act (15 USC § 1681
//! et seq.) interaction** — landlord furnishing positive
//! rental payment data to consumer reporting agency is a
//! "FURNISHER" under § 1681s-2; must comply with
//! accuracy and dispute-investigation requirements.
//! Failure to investigate disputed inaccurate report
//! creates private right of action under § 1681s-2(b).
//!
//! Citations: Cal. Civ. Code § 1954.07 (AB 2747 of 2024,
//! effective April 1, 2025); Colorado HB 23-1099;
//! Washington SB 5495 of 2024; HUD Tenant Credit
//! Reporting Pilot (FY 2023-2025); Federal Fair Credit
//! Reporting Act, 15 USC § 1681 et seq.; 15 USC § 1681s-2
//! (furnisher duties).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    Colorado,
    Washington,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LandlordEntityType {
    /// Individual or single-LLC landlord (not subject to
    /// CA 15-or-fewer-unit carveout).
    Individual,
    /// REIT.
    RealEstateInvestmentTrust,
    /// Corporation or LLC with at least one corporate
    /// member (engages CA carveout if multiple buildings
    /// owned).
    CorporationOrLlcWithCorporateMember,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TenantPositiveRentReportingInput {
    pub jurisdiction: Jurisdiction,
    /// Effective date year (CA: April 1, 2025).
    pub determination_year: u32,
    pub determination_month: u32,
    /// Number of dwelling units in building.
    pub dwelling_unit_count: u32,
    /// Whether landlord owns more than one rental
    /// building (CA 15-or-fewer-unit carveout trigger).
    pub landlord_owns_multiple_buildings: bool,
    /// Landlord entity type.
    pub landlord_entity_type: LandlordEntityType,
    /// Whether landlord offered tenant positive rent
    /// reporting option at lease execution / annual
    /// renewal.
    pub offer_made_at_lease: bool,
    /// Whether offer is made AT LEAST ONCE ANNUALLY.
    pub offered_annually: bool,
    /// Whether lease was entered into ON OR AFTER April
    /// 1, 2025 (post-effective) or outstanding as of
    /// January 1, 2025.
    pub lease_post_effective: bool,
    /// Whether tenant elected positive rent reporting
    /// option.
    pub tenant_elected_reporting: bool,
    /// Monthly fee charged for reporting service in
    /// cents.
    pub monthly_fee_charged_cents: u64,
    /// Landlord's actual monthly cost of providing
    /// reporting service in cents.
    pub actual_monthly_cost_cents: u64,
    /// Whether landlord reports only complete, timely
    /// payments (no incomplete/late) — positive
    /// information only.
    pub reports_only_positive_payments: bool,
    /// Whether landlord terminated tenancy due to
    /// non-payment of reporting fee (PROHIBITED).
    pub terminated_for_fee_nonpayment: bool,
    /// Whether landlord deducted unpaid fee from security
    /// deposit (PROHIBITED).
    pub deducted_fee_from_security_deposit: bool,
    /// Days fee unpaid (30+ allows landlord to stop
    /// reporting; tenant blocked 6 months).
    pub days_fee_unpaid: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TenantPositiveRentReportingResult {
    pub jurisdiction: Jurisdiction,
    pub offer_obligation_triggered: bool,
    pub ca_15_unit_carveout_engaged: bool,
    pub offer_compliant: bool,
    pub fee_cap_compliant: bool,
    pub positive_only_compliant: bool,
    pub tenant_protections_compliant: bool,
    pub stop_reporting_eligible: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(
    input: &TenantPositiveRentReportingInput,
) -> TenantPositiveRentReportingResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let post_april_2025 = input.determination_year > 2025
        || (input.determination_year == 2025 && input.determination_month >= 4);

    let ca_15_unit_carveout_engaged = input.jurisdiction == Jurisdiction::California
        && input.dwelling_unit_count <= 15
        && input.landlord_owns_multiple_buildings
        && !matches!(input.landlord_entity_type, LandlordEntityType::Individual);

    let offer_obligation_triggered = match input.jurisdiction {
        Jurisdiction::California => {
            post_april_2025
                && (input.dwelling_unit_count >= 16 || ca_15_unit_carveout_engaged)
        }
        Jurisdiction::Colorado => input.determination_year >= 2023,
        Jurisdiction::Washington => input.determination_year >= 2024
            && input.dwelling_unit_count >= 5,
        Jurisdiction::Default => false,
    };

    let offer_compliant = !offer_obligation_triggered
        || (input.offer_made_at_lease && input.offered_annually);

    let cap_cents = 1_000_u64;
    let fee_cap = cap_cents.min(input.actual_monthly_cost_cents);
    let fee_cap_compliant = !input.tenant_elected_reporting
        || input.monthly_fee_charged_cents <= fee_cap;

    let positive_only_compliant =
        !input.tenant_elected_reporting || input.reports_only_positive_payments;

    let stop_reporting_eligible =
        input.tenant_elected_reporting && input.days_fee_unpaid >= 30;

    let tenant_protections_compliant =
        !input.terminated_for_fee_nonpayment && !input.deducted_fee_from_security_deposit;

    if offer_obligation_triggered && !input.offer_made_at_lease {
        failure_reasons.push(
            "Cal. Civ. Code § 1954.07 (AB 2747 of 2024) — landlord must OFFER positive rent reporting option AT THE TIME OF LEASE AGREEMENT (for leases entered into on or after April 1, 2025) OR no later than April 1, 2025 (for leases outstanding as of January 1, 2025)".to_string(),
        );
    }

    if offer_obligation_triggered && !input.offered_annually {
        failure_reasons.push(
            "Cal. Civ. Code § 1954.07 — landlord must OFFER positive rent reporting option AT LEAST ONCE ANNUALLY in addition to lease-execution offer".to_string(),
        );
    }

    if input.tenant_elected_reporting && input.monthly_fee_charged_cents > fee_cap {
        failure_reasons.push(format!(
            "Cal. Civ. Code § 1954.07 — fee MUST NOT EXCEED the LESSER of (1) actual cost to landlord OR (2) $10 PER MONTH; cap = {} cents; charged = {} cents",
            fee_cap, input.monthly_fee_charged_cents
        ));
    }

    if input.tenant_elected_reporting && !input.reports_only_positive_payments {
        failure_reasons.push(
            "Cal. Civ. Code § 1954.07 — 'positive rental payment information' means COMPLETE, TIMELY payments of rent; landlord MAY NOT report INCOMPLETE OR LATE payments under this statute (separate FCRA framework applies for negative reporting)".to_string(),
        );
    }

    if input.terminated_for_fee_nonpayment {
        failure_reasons.push(
            "Cal. Civ. Code § 1954.07 — tenant's failure to pay the rent-reporting fee SHALL NOT BE CAUSE FOR TERMINATION OF TENANCY".to_string(),
        );
    }

    if input.deducted_fee_from_security_deposit {
        failure_reasons.push(
            "Cal. Civ. Code § 1954.07 — landlord SHALL NOT DEDUCT unpaid rent-reporting fee from tenant's security deposit".to_string(),
        );
    }

    if stop_reporting_eligible {
        failure_reasons.push(
            "Cal. Civ. Code § 1954.07 — fee unpaid for 30 DAYS OR MORE: landlord MAY STOP REPORTING tenant's rental payments; tenant BLOCKED from re-electing positive rental payment reporting for 6 MONTHS from date fee first became due".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "Cal. Civ. Code § 1954.07 (AB 2747 of 2024, effective April 1, 2025) — residential landlord of 16+ unit building MUST OFFER tenants option to have positive rental payment information reported to at least one nationwide consumer reporting agency (Experian, Equifax, TransUnion)".to_string(),
        "Cal. Civ. Code § 1954.07 building-size threshold — applies to buildings of 16 OR MORE UNITS; 15-or-fewer-unit landlord EXEMPT UNLESS (a) owns MORE THAN ONE rental building AND (b) is REIT, corporation, or LLC with at least one corporate member".to_string(),
        "Cal. Civ. Code § 1954.07 offer timing — for leases entered into ON OR AFTER April 1, 2025: offer at time of lease agreement AND at least once annually; for leases outstanding as of January 1, 2025: offer no later than April 1, 2025 AND at least once annually thereafter".to_string(),
        "Cal. Civ. Code § 1954.07 $10/month fee cap — landlord may charge tenant electing reporting a fee NOT EXCEEDING the LESSER of (1) actual cost OR (2) $10 PER MONTH".to_string(),
        "Cal. Civ. Code § 1954.07 positive-only definition — 'positive rental payment information' means COMPLETE, TIMELY payments of rent; specifically EXCLUDES incomplete or late payments; landlord MAY NOT report negative information under this statute".to_string(),
        "Cal. Civ. Code § 1954.07 tenant fee non-payment protections — (1) failure to pay fee SHALL NOT BE CAUSE FOR TERMINATION OF TENANCY; (2) landlord SHALL NOT DEDUCT unpaid fee from security deposit; (3) if unpaid 30+ days, landlord may stop reporting AND tenant blocked from re-electing for 6 MONTHS".to_string(),
        "Colorado HB 23-1099 — positive rent reporting requirement for Section 8 Housing Choice Voucher holders + tenant-elected reporting; pilot expansion for general residential market".to_string(),
        "Washington SB 5495 of 2024 — similar landlord-offer framework for residential rental buildings of 5+ units".to_string(),
        "HUD Tenant Credit Reporting Pilot (FY 2023-2025) — federally-subsidized housing participation; HUD experiments with reporting positive rent payments to assist tenants building credit history".to_string(),
        "Federal Fair Credit Reporting Act (15 USC § 1681 et seq.) — landlord furnishing positive rental payment data to consumer reporting agency is a 'FURNISHER' under § 1681s-2; must comply with accuracy and dispute-investigation requirements; failure to investigate disputed inaccurate report creates private right of action under § 1681s-2(b)".to_string(),
        "Tenant election is OPTIONAL — landlord cannot REQUIRE positive rent reporting; tenant must affirmatively elect to have payments reported; CA AB 2747 framework operates as an OFFER MANDATE not a REPORT MANDATE".to_string(),
        "Trader-landlord critical because (1) annual repeat-offer obligation; (2) $10/month fee cap strictly enforced; (3) failure to offer creates per-violation civil exposure; (4) FCRA exposure for misreporting; (5) tenant fee non-payment cannot trigger eviction or security-deposit-deduction (distinct from general rent collection rules)".to_string(),
    ];

    TenantPositiveRentReportingResult {
        jurisdiction: input.jurisdiction,
        offer_obligation_triggered,
        ca_15_unit_carveout_engaged,
        offer_compliant,
        fee_cap_compliant,
        positive_only_compliant,
        tenant_protections_compliant,
        stop_reporting_eligible,
        failure_reasons,
        citation: "Cal. Civ. Code § 1954.07 (AB 2747 of 2024, effective April 1, 2025); Colorado HB 23-1099; Washington SB 5495 of 2024; HUD Tenant Credit Reporting Pilot (FY 2023-2025); 15 USC § 1681 et seq. (Fair Credit Reporting Act); 15 USC § 1681s-2 (furnisher duties)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_compliant() -> TenantPositiveRentReportingInput {
        TenantPositiveRentReportingInput {
            jurisdiction: Jurisdiction::California,
            determination_year: 2026,
            determination_month: 6,
            dwelling_unit_count: 20,
            landlord_owns_multiple_buildings: false,
            landlord_entity_type: LandlordEntityType::Individual,
            offer_made_at_lease: true,
            offered_annually: true,
            lease_post_effective: true,
            tenant_elected_reporting: true,
            monthly_fee_charged_cents: 1_000,
            actual_monthly_cost_cents: 1_500,
            reports_only_positive_payments: true,
            terminated_for_fee_nonpayment: false,
            deducted_fee_from_security_deposit: false,
            days_fee_unpaid: 0,
        }
    }

    #[test]
    fn ca_16_plus_units_post_april_2025_obligation_engaged() {
        let r = check(&ca_compliant());
        assert!(r.offer_obligation_triggered);
        assert!(r.offer_compliant);
    }

    #[test]
    fn ca_pre_april_2025_no_obligation() {
        let mut i = ca_compliant();
        i.determination_year = 2025;
        i.determination_month = 3;
        let r = check(&i);
        assert!(!r.offer_obligation_triggered);
    }

    #[test]
    fn ca_april_2025_boundary_engages() {
        let mut i = ca_compliant();
        i.determination_year = 2025;
        i.determination_month = 4;
        let r = check(&i);
        assert!(r.offer_obligation_triggered);
    }

    #[test]
    fn ca_15_or_fewer_units_individual_no_obligation() {
        let mut i = ca_compliant();
        i.dwelling_unit_count = 15;
        i.landlord_entity_type = LandlordEntityType::Individual;
        let r = check(&i);
        assert!(!r.offer_obligation_triggered);
        assert!(!r.ca_15_unit_carveout_engaged);
    }

    #[test]
    fn ca_15_units_corporate_multiple_buildings_carveout_engages() {
        let mut i = ca_compliant();
        i.dwelling_unit_count = 15;
        i.landlord_owns_multiple_buildings = true;
        i.landlord_entity_type = LandlordEntityType::CorporationOrLlcWithCorporateMember;
        let r = check(&i);
        assert!(r.ca_15_unit_carveout_engaged);
        assert!(r.offer_obligation_triggered);
    }

    #[test]
    fn ca_15_units_reit_carveout_engages() {
        let mut i = ca_compliant();
        i.dwelling_unit_count = 15;
        i.landlord_owns_multiple_buildings = true;
        i.landlord_entity_type = LandlordEntityType::RealEstateInvestmentTrust;
        let r = check(&i);
        assert!(r.ca_15_unit_carveout_engaged);
    }

    #[test]
    fn ca_15_units_corporate_single_building_no_carveout() {
        let mut i = ca_compliant();
        i.dwelling_unit_count = 15;
        i.landlord_owns_multiple_buildings = false;
        i.landlord_entity_type = LandlordEntityType::CorporationOrLlcWithCorporateMember;
        let r = check(&i);
        assert!(!r.ca_15_unit_carveout_engaged);
        assert!(!r.offer_obligation_triggered);
    }

    #[test]
    fn ca_16_units_no_offer_violation() {
        let mut i = ca_compliant();
        i.offer_made_at_lease = false;
        let r = check(&i);
        assert!(!r.offer_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 1954.07")
            && f.contains("AB 2747 of 2024")
            && f.contains("April 1, 2025")));
    }

    #[test]
    fn ca_no_annual_offer_violation() {
        let mut i = ca_compliant();
        i.offered_annually = false;
        let r = check(&i);
        assert!(!r.offer_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 1954.07")
            && f.contains("AT LEAST ONCE ANNUALLY")));
    }

    #[test]
    fn fee_above_10_dollar_cap_violation() {
        let mut i = ca_compliant();
        i.actual_monthly_cost_cents = 1_500;
        i.monthly_fee_charged_cents = 1_500;
        let r = check(&i);
        assert!(!r.fee_cap_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 1954.07")
            && f.contains("$10 PER MONTH")
            && f.contains("LESSER of")));
    }

    #[test]
    fn fee_at_10_dollar_cap_compliant() {
        let mut i = ca_compliant();
        i.monthly_fee_charged_cents = 1_000;
        i.actual_monthly_cost_cents = 5_000;
        let r = check(&i);
        assert!(r.fee_cap_compliant);
    }

    #[test]
    fn fee_at_actual_cost_below_10_compliant() {
        let mut i = ca_compliant();
        i.monthly_fee_charged_cents = 500;
        i.actual_monthly_cost_cents = 500;
        let r = check(&i);
        assert!(r.fee_cap_compliant);
    }

    #[test]
    fn fee_above_actual_cost_violation_even_below_10() {
        let mut i = ca_compliant();
        i.actual_monthly_cost_cents = 500;
        i.monthly_fee_charged_cents = 800;
        let r = check(&i);
        assert!(!r.fee_cap_compliant);
    }

    #[test]
    fn negative_reporting_violation() {
        let mut i = ca_compliant();
        i.reports_only_positive_payments = false;
        let r = check(&i);
        assert!(!r.positive_only_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 1954.07")
            && f.contains("positive rental payment information")
            && f.contains("COMPLETE, TIMELY")));
    }

    #[test]
    fn termination_for_fee_nonpayment_violation() {
        let mut i = ca_compliant();
        i.terminated_for_fee_nonpayment = true;
        let r = check(&i);
        assert!(!r.tenant_protections_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 1954.07")
            && f.contains("SHALL NOT BE CAUSE FOR TERMINATION")));
    }

    #[test]
    fn deducting_fee_from_security_deposit_violation() {
        let mut i = ca_compliant();
        i.deducted_fee_from_security_deposit = true;
        let r = check(&i);
        assert!(!r.tenant_protections_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 1954.07")
            && f.contains("SHALL NOT DEDUCT")
            && f.contains("security deposit")));
    }

    #[test]
    fn fee_unpaid_30_days_stop_reporting_engages() {
        let mut i = ca_compliant();
        i.days_fee_unpaid = 30;
        let r = check(&i);
        assert!(r.stop_reporting_eligible);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 1954.07")
            && f.contains("30 DAYS OR MORE")
            && f.contains("6 MONTHS")));
    }

    #[test]
    fn fee_unpaid_29_days_no_stop_reporting() {
        let mut i = ca_compliant();
        i.days_fee_unpaid = 29;
        let r = check(&i);
        assert!(!r.stop_reporting_eligible);
    }

    #[test]
    fn colorado_engages_2023_and_later() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::Colorado;
        i.determination_year = 2024;
        let r = check(&i);
        assert!(r.offer_obligation_triggered);
    }

    #[test]
    fn washington_5_unit_threshold() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::Washington;
        i.determination_year = 2024;
        i.dwelling_unit_count = 5;
        let r = check(&i);
        assert!(r.offer_obligation_triggered);
        i.dwelling_unit_count = 4;
        let r2 = check(&i);
        assert!(!r2.offer_obligation_triggered);
    }

    #[test]
    fn default_jurisdiction_no_obligation() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::Default;
        let r = check(&i);
        assert!(!r.offer_obligation_triggered);
    }

    #[test]
    fn jurisdiction_truth_table_four_cells() {
        for jur in [
            Jurisdiction::California,
            Jurisdiction::Colorado,
            Jurisdiction::Washington,
            Jurisdiction::Default,
        ] {
            let mut i = ca_compliant();
            i.jurisdiction = jur;
            let r = check(&i);
            assert_eq!(r.jurisdiction, jur);
        }
    }

    #[test]
    fn ca_carveout_only_engages_in_california_invariant() {
        let mut ca = ca_compliant();
        ca.dwelling_unit_count = 15;
        ca.landlord_owns_multiple_buildings = true;
        ca.landlord_entity_type = LandlordEntityType::CorporationOrLlcWithCorporateMember;
        let r_ca = check(&ca);
        assert!(r_ca.ca_15_unit_carveout_engaged);

        for jur in [
            Jurisdiction::Colorado,
            Jurisdiction::Washington,
            Jurisdiction::Default,
        ] {
            let mut i = ca_compliant();
            i.jurisdiction = jur;
            i.dwelling_unit_count = 15;
            i.landlord_owns_multiple_buildings = true;
            i.landlord_entity_type = LandlordEntityType::CorporationOrLlcWithCorporateMember;
            let r = check(&i);
            assert!(!r.ca_15_unit_carveout_engaged, "jur={:?}", jur);
        }
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&ca_compliant());
        assert!(r.citation.contains("Cal. Civ. Code § 1954.07"));
        assert!(r.citation.contains("AB 2747 of 2024"));
        assert!(r.citation.contains("April 1, 2025"));
        assert!(r.citation.contains("Colorado HB 23-1099"));
        assert!(r.citation.contains("Washington SB 5495 of 2024"));
        assert!(r.citation.contains("HUD Tenant Credit Reporting Pilot"));
        assert!(r.citation.contains("15 USC § 1681 et seq."));
        assert!(r.citation.contains("Fair Credit Reporting Act"));
        assert!(r.citation.contains("§ 1681s-2"));
    }

    #[test]
    fn note_pins_ca_16_unit_threshold() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1954.07")
            && n.contains("AB 2747 of 2024")
            && n.contains("April 1, 2025")
            && n.contains("nationwide consumer reporting agency")));
    }

    #[test]
    fn note_pins_ca_15_unit_carveout() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1954.07")
            && n.contains("15-or-fewer-unit landlord EXEMPT")
            && n.contains("MORE THAN ONE rental building")
            && n.contains("REIT, corporation, or LLC")));
    }

    #[test]
    fn note_pins_ca_offer_timing_dual_track() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1954.07")
            && n.contains("April 1, 2025")
            && n.contains("January 1, 2025")
            && n.contains("at least once annually")));
    }

    #[test]
    fn note_pins_ca_10_fee_cap() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1954.07")
            && n.contains("$10/month")
            && n.contains("LESSER of")));
    }

    #[test]
    fn note_pins_ca_positive_only_definition() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1954.07")
            && n.contains("positive-only definition")
            && n.contains("COMPLETE, TIMELY")
            && n.contains("EXCLUDES incomplete or late")));
    }

    #[test]
    fn note_pins_ca_tenant_protections_three_prong() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1954.07")
            && n.contains("tenant fee non-payment protections")
            && n.contains("SHALL NOT BE CAUSE FOR TERMINATION")
            && n.contains("SHALL NOT DEDUCT")
            && n.contains("30+ days")
            && n.contains("6 MONTHS")));
    }

    #[test]
    fn note_pins_colorado_hb_23_1099() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Colorado HB 23-1099")
            && n.contains("Section 8")));
    }

    #[test]
    fn note_pins_washington_sb_5495() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Washington SB 5495 of 2024")
            && n.contains("5+ units")));
    }

    #[test]
    fn note_pins_hud_pilot_2023_2025() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("HUD Tenant Credit Reporting Pilot")
            && n.contains("FY 2023-2025")));
    }

    #[test]
    fn note_pins_fcra_furnisher_duties() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("15 USC § 1681 et seq.")
            && n.contains("FURNISHER")
            && n.contains("§ 1681s-2")
            && n.contains("private right of action")));
    }

    #[test]
    fn note_pins_optional_election() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Tenant election is OPTIONAL")
            && n.contains("OFFER MANDATE not a REPORT MANDATE")));
    }

    #[test]
    fn multiple_failures_stack() {
        let mut i = ca_compliant();
        i.offer_made_at_lease = false;
        i.offered_annually = false;
        i.actual_monthly_cost_cents = 2_000;
        i.monthly_fee_charged_cents = 2_000;
        i.reports_only_positive_payments = false;
        i.terminated_for_fee_nonpayment = true;
        i.deducted_fee_from_security_deposit = true;
        i.days_fee_unpaid = 30;
        let r = check(&i);
        assert!(r.failure_reasons.len() >= 6);
    }
}
