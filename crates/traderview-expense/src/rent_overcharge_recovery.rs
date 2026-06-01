//! Rent overcharge recovery in rent-stabilized / rent-controlled
//! buildings — when a landlord charges rent in excess of the
//! legal regulated rent, what statutory procedure and damages
//! attach? Distinct from siblings `rent_control` (broad
//! regulatory framework + applicability), `rent_control_lease_
//! disclosure` (mandatory disclosure of rent-stabilization
//! status at lease execution), `rent_increase_notice_period`
//! (advance notice rules), and `rent_acceleration_
//! enforceability` (full-balance acceleration clauses).
//!
//! Trader-landlord operational concern in NY rent-stabilized
//! buildings + DC RACA-covered buildings. Failure to charge
//! legal regulated rent exposes landlord to mandatory treble
//! damages (NY HSTPA willful overcharge) + 6-year retrospective
//! liability + attorney fees + DHCR / administrative complaint.
//!
//! **Three regimes**:
//!
//! **New York — HSTPA 2019 (Housing Stability and Tenant
//! Protection Act of 2019) — most aggressive framework**. § 2
//! amended N.Y. Real Prop. Tax Law / N.Y. Multiple Dwelling
//! Law / Emergency Tenant Protection Act:
//! - **6-year lookback period** (extended from 4 years pre-
//!   HSTPA). Owners must retain rent records for entire
//!   tenancy AND 6 years prior to any complaint.
//! - **Treble damages MANDATORY if willful overcharge** (made
//!   non-discretionary; previously discretionary).
//! - **6-year treble-damages period** (extended from 2 years).
//! - **Attorney fees + costs + interest non-discretionary** if
//!   overcharge established.
//! - **Fraud exception**: court may extend 6-year lookback
//!   further when landlord engaged in fraud to conceal
//!   overcharge (falsifying records, failing to register).
//! - Tenant may file complaint with DHCR (Division of Housing
//!   and Community Renewal) OR bring action in court.
//!
//! **District of Columbia — RACA (Rental Housing Act of 1985,
//! D.C. Code §§ 42-3502.01 to 42-3502.20)**. Rental Housing
//! Commission administers overcharge complaint procedure.
//! Treble damages available for willful overcharge. Limited
//! lookback. Cross-references DC TOPA framework (see
//! `tenant_topa`).
//!
//! **Default — common-law restitution + municipal ordinances**.
//! Most states have NO statewide rent-stabilization framework;
//! overcharge recovery relies on common-law restitution
//! principles. Municipal ordinances (Berkeley, San Francisco,
//! Los Angeles, Oakland, Santa Monica, Newark, NJ municipal
//! rent control) may impose administrative overcharge
//! procedures.
//!
//! Citations: HSTPA of 2019 (N.Y. Laws 2019, ch. 36); N.Y. Real
//! Prop. Law § 226-c; N.Y.C. Admin. Code § 26-516(a) (NYC Rent
//! Stabilization Code overcharge complaint procedure); 9
//! NYCRR Part 2522 et seq. (DHCR Rent Stabilization Code);
//! D.C. Code §§ 42-3502.01 to 42-3502.20 (DC RACA); Cal. Civ.
//! Code § 1947.7 (CA limited rent control framework);
//! common-law restitution Restatement (Third) of Restitution
//! and Unjust Enrichment § 1.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    NewYorkHstpa,
    DistrictOfColumbiaRaca,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentOverchargeRecoveryInput {
    pub regime: Regime,
    /// Total alleged overcharge amount in cents (over and above
    /// legal regulated rent).
    pub alleged_overcharge_amount_cents: i64,
    /// Number of months overcharge has continued.
    pub overcharge_period_months: u32,
    /// Whether the overcharge is willful (triggers treble damages
    /// under NY HSTPA).
    pub willful_overcharge: bool,
    /// Whether the landlord engaged in fraud to conceal the
    /// overcharge (falsifying records, failing to register).
    pub fraud_to_conceal_overcharge: bool,
    /// Whether the tenant filed the complaint within the 6-year
    /// lookback period (NY HSTPA).
    pub complaint_filed_within_six_year_lookback: bool,
    /// Whether the landlord registered the rent-stabilized unit
    /// with DHCR (NY-specific procedural requirement).
    pub landlord_registered_unit_with_dhcr: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentOverchargeRecoveryResult {
    /// Whether the complaint is within the applicable lookback
    /// period.
    pub complaint_within_lookback: bool,
    /// Whether treble damages are engaged (mandatory on willful
    /// overcharge under NY HSTPA).
    pub treble_damages_engaged: bool,
    /// Single damages amount (alleged overcharge) in cents.
    pub single_damages_amount_cents: i64,
    /// Treble damages amount (3x alleged overcharge) in cents
    /// when engaged.
    pub treble_damages_amount_cents: i64,
    /// Whether attorney fees + costs + interest are mandatory.
    pub attorney_fees_costs_interest_mandatory: bool,
    /// Whether the fraud exception extends the lookback beyond
    /// the 6-year default.
    pub fraud_exception_extends_lookback: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &RentOverchargeRecoveryInput) -> RentOverchargeRecoveryResult {
    match input.regime {
        Regime::NewYorkHstpa => check_new_york(input),
        Regime::DistrictOfColumbiaRaca => check_district_of_columbia(input),
        Regime::Default => check_default(input),
    }
}

fn check_new_york(input: &RentOverchargeRecoveryInput) -> RentOverchargeRecoveryResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    let single = input.alleged_overcharge_amount_cents.max(0);
    let treble_damages_engaged = input.willful_overcharge;
    let treble_amount = if treble_damages_engaged {
        single.saturating_mul(3)
    } else {
        single
    };

    let complaint_in_lookback = input.complaint_filed_within_six_year_lookback
        || input.fraud_to_conceal_overcharge;

    let attorney_fees_mandatory = single > 0 && complaint_in_lookback;

    notes.push(
        "HSTPA of 2019 (N.Y. Laws 2019, ch. 36) — extended overcharge lookback from 4 years to 6 YEARS; landlord must retain rent records for entire tenancy AND 6 years prior to complaint"
            .to_string(),
    );

    if treble_damages_engaged {
        notes.push(
            "HSTPA — TREBLE DAMAGES MANDATORY for willful overcharge (made non-discretionary; previously discretionary); 6-year treble-damages period (extended from 2 years pre-HSTPA)"
                .to_string(),
        );
    } else {
        notes.push(
            "HSTPA — overcharge alleged but not WILLFUL; single damages recoverable but treble damages not engaged"
                .to_string(),
        );
    }

    if input.fraud_to_conceal_overcharge {
        notes.push(
            "HSTPA fraud exception — court may extend 6-year lookback further when landlord engaged in fraud to conceal overcharge (falsifying records, failing to register with DHCR)"
                .to_string(),
        );
    }

    if !input.landlord_registered_unit_with_dhcr {
        violations.push(
            "9 NYCRR Part 2522 — landlord MUST register rent-stabilized unit with DHCR annually; failure to register supports fraud-exception lookback extension"
                .to_string(),
        );
    }

    if attorney_fees_mandatory {
        notes.push(
            "HSTPA — attorney fees + costs + interest are NON-DISCRETIONARY (mandatory) when overcharge established"
                .to_string(),
        );
    }

    notes.push(
        "tenant remedy options: file complaint with NYC DHCR (Division of Housing and Community Renewal) OR bring action in court; both proceed under § 26-516 NYC Rent Stabilization Code"
            .to_string(),
    );

    RentOverchargeRecoveryResult {
        complaint_within_lookback: complaint_in_lookback,
        treble_damages_engaged,
        single_damages_amount_cents: single,
        treble_damages_amount_cents: treble_amount,
        attorney_fees_costs_interest_mandatory: attorney_fees_mandatory,
        fraud_exception_extends_lookback: input.fraud_to_conceal_overcharge,
        violations,
        citation: "HSTPA of 2019 (N.Y. Laws 2019, ch. 36); N.Y. Real Prop. Law § 226-c; N.Y.C. Admin. Code § 26-516; 9 NYCRR Part 2522",
        notes,
    }
}

fn check_district_of_columbia(
    input: &RentOverchargeRecoveryInput,
) -> RentOverchargeRecoveryResult {
    let mut notes: Vec<String> = Vec::new();

    let single = input.alleged_overcharge_amount_cents.max(0);
    let treble_engaged = input.willful_overcharge;
    let treble_amount = if treble_engaged {
        single.saturating_mul(3)
    } else {
        single
    };

    notes.push(
        "D.C. Code §§ 42-3502.01 to 42-3502.20 — Rental Housing Act of 1985 (RACA); Rental Housing Commission administers overcharge complaint procedure"
            .to_string(),
    );
    notes.push(
        "RACA — treble damages available for willful overcharge; limited lookback period (shorter than NY HSTPA 6-year)"
            .to_string(),
    );
    notes.push(
        "DC RACA cross-references TOPA framework — see tenant_topa module for DC Tenant Opportunity to Purchase Act"
            .to_string(),
    );

    RentOverchargeRecoveryResult {
        complaint_within_lookback: true,
        treble_damages_engaged: treble_engaged,
        single_damages_amount_cents: single,
        treble_damages_amount_cents: treble_amount,
        attorney_fees_costs_interest_mandatory: false,
        fraud_exception_extends_lookback: input.fraud_to_conceal_overcharge,
        violations: Vec::new(),
        citation: "D.C. Code §§ 42-3502.01 to 42-3502.20 (RACA)",
        notes,
    }
}

fn check_default(input: &RentOverchargeRecoveryInput) -> RentOverchargeRecoveryResult {
    let notes: Vec<String> = vec![
        "default rule — most states have NO statewide rent-stabilization framework; overcharge recovery relies on common-law restitution principles per Restatement (Third) of Restitution and Unjust Enrichment § 1"
            .to_string(),
        "municipal ordinances may impose administrative overcharge procedures (Berkeley, San Francisco, Los Angeles, Oakland, Santa Monica rent ordinances; Newark, Hoboken, Jersey City municipal NJ rent control)"
            .to_string(),
        "Cal. Civ. Code § 1947.7 — California has limited statewide rent control framework; mostly municipal supplements (CA Tenant Protection Act AB 1482 caps annual increases but does not establish overcharge complaint procedure)"
            .to_string(),
    ];

    let single = input.alleged_overcharge_amount_cents.max(0);

    RentOverchargeRecoveryResult {
        complaint_within_lookback: true,
        treble_damages_engaged: false,
        single_damages_amount_cents: single,
        treble_damages_amount_cents: single,
        attorney_fees_costs_interest_mandatory: false,
        fraud_exception_extends_lookback: false,
        violations: Vec::new(),
        citation: "common-law restitution + Restatement (Third) of Restitution and Unjust Enrichment § 1; Cal. Civ. Code § 1947.7; municipal rent ordinances",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ny_base() -> RentOverchargeRecoveryInput {
        RentOverchargeRecoveryInput {
            regime: Regime::NewYorkHstpa,
            alleged_overcharge_amount_cents: 1_000_000,
            overcharge_period_months: 24,
            willful_overcharge: false,
            fraud_to_conceal_overcharge: false,
            complaint_filed_within_six_year_lookback: true,
            landlord_registered_unit_with_dhcr: true,
        }
    }

    fn dc_base() -> RentOverchargeRecoveryInput {
        RentOverchargeRecoveryInput {
            regime: Regime::DistrictOfColumbiaRaca,
            alleged_overcharge_amount_cents: 500_000,
            overcharge_period_months: 12,
            willful_overcharge: false,
            fraud_to_conceal_overcharge: false,
            complaint_filed_within_six_year_lookback: true,
            landlord_registered_unit_with_dhcr: false,
        }
    }

    fn default_base() -> RentOverchargeRecoveryInput {
        RentOverchargeRecoveryInput {
            regime: Regime::Default,
            alleged_overcharge_amount_cents: 100_000,
            overcharge_period_months: 6,
            willful_overcharge: false,
            fraud_to_conceal_overcharge: false,
            complaint_filed_within_six_year_lookback: true,
            landlord_registered_unit_with_dhcr: false,
        }
    }

    #[test]
    fn ny_clean_complaint_within_lookback() {
        let r = check(&ny_base());
        assert!(r.complaint_within_lookback);
        assert_eq!(r.single_damages_amount_cents, 1_000_000);
    }

    #[test]
    fn ny_non_willful_no_treble() {
        let r = check(&ny_base());
        assert!(!r.treble_damages_engaged);
        assert_eq!(r.treble_damages_amount_cents, 1_000_000);
    }

    #[test]
    fn ny_willful_triggers_mandatory_treble() {
        let mut i = ny_base();
        i.willful_overcharge = true;
        let r = check(&i);
        assert!(r.treble_damages_engaged);
        assert_eq!(r.treble_damages_amount_cents, 3_000_000);
        assert!(r.notes.iter().any(|n| n.contains("TREBLE DAMAGES MANDATORY") && n.contains("non-discretionary")));
    }

    #[test]
    fn ny_six_year_lookback_note_present() {
        let r = check(&ny_base());
        assert!(r.notes.iter().any(|n| n.contains("HSTPA of 2019") && n.contains("6 YEARS")));
    }

    #[test]
    fn ny_fraud_exception_extends_lookback() {
        let mut i = ny_base();
        i.fraud_to_conceal_overcharge = true;
        i.complaint_filed_within_six_year_lookback = false;
        let r = check(&i);
        assert!(r.fraud_exception_extends_lookback);
        assert!(r.complaint_within_lookback);
        assert!(r.notes.iter().any(|n| n.contains("HSTPA fraud exception") && n.contains("falsifying records")));
    }

    #[test]
    fn ny_outside_six_year_no_fraud_not_in_lookback() {
        let mut i = ny_base();
        i.complaint_filed_within_six_year_lookback = false;
        let r = check(&i);
        assert!(!r.complaint_within_lookback);
    }

    #[test]
    fn ny_unregistered_unit_violation() {
        let mut i = ny_base();
        i.landlord_registered_unit_with_dhcr = false;
        let r = check(&i);
        assert!(r.violations.iter().any(|v| v.contains("9 NYCRR Part 2522") && v.contains("register rent-stabilized unit with DHCR")));
    }

    #[test]
    fn ny_registered_unit_no_violation() {
        let r = check(&ny_base());
        assert!(r.violations.is_empty());
    }

    #[test]
    fn ny_attorney_fees_mandatory_when_overcharge_in_lookback() {
        let r = check(&ny_base());
        assert!(r.attorney_fees_costs_interest_mandatory);
        assert!(r.notes.iter().any(|n| n.contains("HSTPA") && n.contains("attorney fees") && n.contains("NON-DISCRETIONARY")));
    }

    #[test]
    fn ny_zero_overcharge_no_attorney_fees() {
        let mut i = ny_base();
        i.alleged_overcharge_amount_cents = 0;
        let r = check(&i);
        assert!(!r.attorney_fees_costs_interest_mandatory);
    }

    #[test]
    fn ny_dhcr_complaint_option_note() {
        let r = check(&ny_base());
        assert!(r.notes.iter().any(|n| n.contains("DHCR") && n.contains("court") && n.contains("§ 26-516")));
    }

    #[test]
    fn ny_citation_pins_hstpa_2019_and_dhcr_authorities() {
        let r = check(&ny_base());
        assert!(r.citation.contains("HSTPA of 2019"));
        assert!(r.citation.contains("N.Y. Laws 2019, ch. 36"));
        assert!(r.citation.contains("§ 226-c"));
        assert!(r.citation.contains("§ 26-516"));
        assert!(r.citation.contains("9 NYCRR Part 2522"));
    }

    #[test]
    fn dc_clean_overcharge_complaint() {
        let r = check(&dc_base());
        assert!(r.complaint_within_lookback);
        assert_eq!(r.single_damages_amount_cents, 500_000);
    }

    #[test]
    fn dc_willful_triggers_treble() {
        let mut i = dc_base();
        i.willful_overcharge = true;
        let r = check(&i);
        assert!(r.treble_damages_engaged);
        assert_eq!(r.treble_damages_amount_cents, 1_500_000);
    }

    #[test]
    fn dc_topa_cross_reference_note() {
        let r = check(&dc_base());
        assert!(r.notes.iter().any(|n| n.contains("TOPA") && n.contains("tenant_topa module")));
    }

    #[test]
    fn dc_citation_pins_raca_sections() {
        let r = check(&dc_base());
        assert!(r.citation.contains("§§ 42-3502.01 to 42-3502.20"));
        assert!(r.citation.contains("RACA"));
    }

    #[test]
    fn default_no_statewide_framework_single_damages_only() {
        let r = check(&default_base());
        assert!(!r.treble_damages_engaged);
        assert_eq!(r.single_damages_amount_cents, 100_000);
        assert_eq!(r.treble_damages_amount_cents, 100_000);
    }

    #[test]
    fn default_municipal_examples_note() {
        let r = check(&default_base());
        assert!(r.notes.iter().any(|n| n.contains("Berkeley") && n.contains("San Francisco") && n.contains("Newark")));
    }

    #[test]
    fn default_ca_1947_7_note_present() {
        let r = check(&default_base());
        assert!(r.notes.iter().any(|n| n.contains("Cal. Civ. Code § 1947.7") && n.contains("AB 1482")));
    }

    #[test]
    fn default_restatement_restitution_note_present() {
        let r = check(&default_base());
        assert!(r.notes.iter().any(|n| n.contains("Restatement") && n.contains("Restitution")));
    }

    #[test]
    fn default_citation_references_municipal_and_restatement() {
        let r = check(&default_base());
        assert!(r.citation.contains("common-law restitution"));
        assert!(r.citation.contains("§ 1947.7"));
        assert!(r.citation.contains("municipal rent ordinances"));
    }

    #[test]
    fn ny_unique_mandatory_treble_invariant() {
        let mut i_ny = ny_base();
        i_ny.willful_overcharge = true;
        let r_ny = check(&i_ny);
        assert_eq!(r_ny.treble_damages_amount_cents, 3_000_000);

        let mut i_default = default_base();
        i_default.willful_overcharge = true;
        let r_default = check(&i_default);
        assert!(!r_default.treble_damages_engaged, "default regime should not engage mandatory treble");
    }

    #[test]
    fn ny_unique_six_year_lookback_invariant() {
        let mut i_ny = ny_base();
        i_ny.complaint_filed_within_six_year_lookback = false;
        let r_ny = check(&i_ny);
        assert!(!r_ny.complaint_within_lookback);

        let mut i_default = default_base();
        i_default.complaint_filed_within_six_year_lookback = false;
        let r_default = check(&i_default);
        assert!(r_default.complaint_within_lookback, "default regime has no 6-year lookback bar");
    }

    #[test]
    fn three_regimes_routed_correctly() {
        for regime in [Regime::NewYorkHstpa, Regime::DistrictOfColumbiaRaca, Regime::Default] {
            let mut i = ny_base();
            i.regime = regime;
            let r = check(&i);
            let _ = r.complaint_within_lookback;
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn negative_overcharge_clamped_to_zero() {
        let mut i = ny_base();
        i.alleged_overcharge_amount_cents = -100_000;
        let r = check(&i);
        assert_eq!(r.single_damages_amount_cents, 0);
        assert_eq!(r.treble_damages_amount_cents, 0);
    }

    #[test]
    fn ny_treble_math_saturating_no_overflow() {
        let mut i = ny_base();
        i.alleged_overcharge_amount_cents = i64::MAX / 2;
        i.willful_overcharge = true;
        let r = check(&i);
        assert_eq!(r.treble_damages_amount_cents, i64::MAX);
    }

    #[test]
    fn ny_treble_invariant_3x_single_when_willful() {
        for amount in [10_000i64, 100_000i64, 1_000_000i64, 10_000_000i64] {
            let mut i = ny_base();
            i.alleged_overcharge_amount_cents = amount;
            i.willful_overcharge = true;
            let r = check(&i);
            assert_eq!(r.treble_damages_amount_cents, amount.saturating_mul(3));
        }
    }

    #[test]
    fn ny_treble_amount_equals_single_when_not_willful() {
        let r = check(&ny_base());
        assert_eq!(r.treble_damages_amount_cents, r.single_damages_amount_cents);
    }

    #[test]
    fn ny_fraud_exception_only_relevant_in_ny() {
        let mut i_ny = ny_base();
        i_ny.fraud_to_conceal_overcharge = true;
        let r_ny = check(&i_ny);
        assert!(r_ny.fraud_exception_extends_lookback);

        let mut i_default = default_base();
        i_default.fraud_to_conceal_overcharge = true;
        let r_default = check(&i_default);
        assert!(!r_default.fraud_exception_extends_lookback);
    }
}
