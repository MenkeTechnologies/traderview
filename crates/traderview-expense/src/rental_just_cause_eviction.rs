//! Multi-state just-cause / good-cause eviction compliance for
//! trader-landlords.
//!
//! Five major state regimes pinned with statutory citations and the
//! operative rent-increase caps + property-class exemptions that each
//! one applies:
//!
//! - **NJ Anti-Eviction Act of 1974** (N.J.S.A. 2A:18-61.1) — first
//!   state to enact just-cause eviction; eighteen enumerated good
//!   causes; broadest applicability (no rent-increase cap).
//!
//! - **CA Tenant Protection Act of 2019 (AB 1482)** (Cal. Civ. Code
//!   § 1946.2 + § 1947.12) — just-cause eviction + annual rent cap of
//!   5% plus regional CPI, hard ceiling at 10%.
//!
//! - **OR SB 608 (2019)** (ORS 90.323 + § 90.427) — first
//!   statewide just-cause + rent-cap regime; 7% plus annual CPI;
//!   exemption for properties built within last 15 years.
//!
//! - **NY Good Cause Eviction Law of 2024** (NY RPL § 226-c + new
//!   Chapter — Part HH of L. 2024, c. 56) — effective April 20, 2024;
//!   "local rent standard" = CPI + 5%, hard ceiling at 10%; exempts
//!   small landlords (≤ 10 units statewide), post-2009 construction,
//!   owner-occupied buildings with < 11 units, rent-stabilized,
//!   public-housing, and condo/co-op rentals.
//!
//! - **WA HB 2114 (2024)** (RCW 59.18.650) — just-cause + 7% + CPI
//!   rent-increase cap.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const NY_RENT_INCREASE_BASE_PERCENT_X_100: u32 = 500;
#[allow(dead_code)]
pub const NY_RENT_INCREASE_MAX_CAP_PERCENT_X_100: u32 = 1_000;
#[allow(dead_code)]
pub const NY_SMALL_LANDLORD_UNIT_THRESHOLD: u32 = 10;
#[allow(dead_code)]
pub const NY_NEW_CONSTRUCTION_YEAR_CUTOFF: u32 = 2009;
#[allow(dead_code)]
pub const NY_OWNER_OCCUPIED_UNIT_THRESHOLD: u32 = 11;
#[allow(dead_code)]
pub const CA_RENT_INCREASE_BASE_PERCENT_X_100: u32 = 500;
#[allow(dead_code)]
pub const CA_RENT_INCREASE_MAX_CAP_PERCENT_X_100: u32 = 1_000;
#[allow(dead_code)]
pub const OR_RENT_INCREASE_BASE_PERCENT_X_100: u32 = 700;
#[allow(dead_code)]
pub const OR_NEW_CONSTRUCTION_AGE_THRESHOLD_YEARS: u32 = 15;
#[allow(dead_code)]
pub const WA_RENT_INCREASE_BASE_PERCENT_X_100: u32 = 700;
#[allow(dead_code)]
pub const NJ_GOOD_CAUSE_ENUMERATED_GROUNDS_COUNT: u32 = 18;
#[allow(dead_code)]
pub const NY_GOOD_CAUSE_EFFECTIVE_DATE_YEAR: u32 = 2024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    NewJerseyAntiEvictionAct1974,
    CaliforniaTpaAb1482,
    OregonSb608,
    NewYorkGoodCauseEviction2024,
    WashingtonHb2114,
    DefaultNoJustCauseRegime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionType {
    NotApplicable,
    NonRenewalOrEviction,
    RentIncrease,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    GoodCauseLandlordHasValidReasonEvictionAllowed,
    GoodCauseLandlordHasNoValidReasonEvictionBarred,
    RentIncreaseWithinLocalCapAllowed,
    RentIncreaseExceedsLocalCapPresumedUnreasonable,
    ExemptSmallLandlordUnder10Units,
    ExemptNewConstructionPostThresholdYear,
    ExemptOwnerOccupiedUnderUnitThreshold,
    ExemptSubsidizedOrRentStabilizedHousing,
    DefaultJurisdictionNoJustCauseRegime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub action_type: ActionType,
    pub current_rent_cents: u64,
    pub proposed_rent_cents: u64,
    pub local_cpi_percent_x_100: u32,
    pub landlord_unit_count_in_state: u32,
    pub property_year_built: u32,
    pub subsidized_or_rent_stabilized: bool,
    pub owner_occupied_building_unit_count: u32,
    pub landlord_claims_good_cause: bool,
    pub landlord_cause_is_statutorily_enumerated: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub rent_cap_percent_x_100: u32,
    pub proposed_increase_percent_x_100: u32,
    pub proposed_rent_within_cap: bool,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type JustCauseEvictionInput = Input;
pub type JustCauseEvictionOutput = Output;
pub type JustCauseEvictionResult = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "NJ Anti-Eviction Act of 1974, N.J.S.A. 2A:18-61.1".to_string(),
        "CA Tenant Protection Act of 2019, Cal. Civ. Code § 1946.2 + § 1947.12 (AB 1482)".to_string(),
        "OR SB 608 (2019), ORS 90.323 + ORS 90.427".to_string(),
        "NY Good Cause Eviction Law of 2024, Part HH of L. 2024, c. 56 (eff. April 20, 2024)".to_string(),
        "NY RPL § 226-c".to_string(),
        "WA HB 2114 (2024), RCW 59.18.650".to_string(),
        "Urban Institute — Just Cause Eviction Laws (national survey)".to_string(),
    ];

    if matches!(
        input.jurisdiction,
        Jurisdiction::DefaultNoJustCauseRegime
    ) {
        notes.push("Jurisdiction has no statewide just-cause eviction regime; common-law lease-termination rules apply.".to_string());
        return Output {
            severity: Severity::DefaultJurisdictionNoJustCauseRegime,
            rent_cap_percent_x_100: 0,
            proposed_increase_percent_x_100: percent_change(input.current_rent_cents, input.proposed_rent_cents),
            proposed_rent_within_cap: true,
            notes,
            citations,
        };
    }

    if matches!(
        input.jurisdiction,
        Jurisdiction::NewYorkGoodCauseEviction2024
    ) {
        if input.landlord_unit_count_in_state <= NY_SMALL_LANDLORD_UNIT_THRESHOLD {
            notes.push(format!(
                "NY small-landlord exemption: ≤ {} units statewide.",
                NY_SMALL_LANDLORD_UNIT_THRESHOLD
            ));
            return Output {
                severity: Severity::ExemptSmallLandlordUnder10Units,
                rent_cap_percent_x_100: 0,
                proposed_increase_percent_x_100: percent_change(input.current_rent_cents, input.proposed_rent_cents),
                proposed_rent_within_cap: true,
                notes,
                citations,
            };
        }
        if input.property_year_built > NY_NEW_CONSTRUCTION_YEAR_CUTOFF {
            notes.push(format!(
                "NY new-construction exemption: built {} > {} cutoff.",
                input.property_year_built,
                NY_NEW_CONSTRUCTION_YEAR_CUTOFF
            ));
            return Output {
                severity: Severity::ExemptNewConstructionPostThresholdYear,
                rent_cap_percent_x_100: 0,
                proposed_increase_percent_x_100: percent_change(input.current_rent_cents, input.proposed_rent_cents),
                proposed_rent_within_cap: true,
                notes,
                citations,
            };
        }
        if input.owner_occupied_building_unit_count > 0
            && input.owner_occupied_building_unit_count < NY_OWNER_OCCUPIED_UNIT_THRESHOLD
        {
            notes.push(format!(
                "NY owner-occupied exemption: < {} units in owner-occupied building.",
                NY_OWNER_OCCUPIED_UNIT_THRESHOLD
            ));
            return Output {
                severity: Severity::ExemptOwnerOccupiedUnderUnitThreshold,
                rent_cap_percent_x_100: 0,
                proposed_increase_percent_x_100: percent_change(input.current_rent_cents, input.proposed_rent_cents),
                proposed_rent_within_cap: true,
                notes,
                citations,
            };
        }
        if input.subsidized_or_rent_stabilized {
            notes.push("NY subsidized/rent-stabilized housing exemption: covered by separate regulatory regime.".to_string());
            return Output {
                severity: Severity::ExemptSubsidizedOrRentStabilizedHousing,
                rent_cap_percent_x_100: 0,
                proposed_increase_percent_x_100: percent_change(input.current_rent_cents, input.proposed_rent_cents),
                proposed_rent_within_cap: true,
                notes,
                citations,
            };
        }
    }

    if matches!(input.jurisdiction, Jurisdiction::OregonSb608) {
        let property_age = 2026u32.saturating_sub(input.property_year_built);
        if property_age <= OR_NEW_CONSTRUCTION_AGE_THRESHOLD_YEARS {
            notes.push(format!(
                "OR SB 608 exemption: property age {} ≤ {} years (new construction).",
                property_age,
                OR_NEW_CONSTRUCTION_AGE_THRESHOLD_YEARS
            ));
            return Output {
                severity: Severity::ExemptNewConstructionPostThresholdYear,
                rent_cap_percent_x_100: 0,
                proposed_increase_percent_x_100: percent_change(input.current_rent_cents, input.proposed_rent_cents),
                proposed_rent_within_cap: true,
                notes,
                citations,
            };
        }
    }

    let cap = compute_rent_cap(input.jurisdiction, input.local_cpi_percent_x_100);

    match input.action_type {
        ActionType::NotApplicable => {
            notes.push("No eviction or rent increase action recorded.".to_string());
            Output {
                severity: Severity::NotApplicable,
                rent_cap_percent_x_100: cap,
                proposed_increase_percent_x_100: percent_change(input.current_rent_cents, input.proposed_rent_cents),
                proposed_rent_within_cap: true,
                notes,
                citations,
            }
        }
        ActionType::RentIncrease => {
            let increase = percent_change(input.current_rent_cents, input.proposed_rent_cents);
            let within_cap = increase <= cap;
            let severity = if within_cap {
                notes.push(format!(
                    "Proposed rent increase {}.{:02}% within local cap {}.{:02}%.",
                    increase / 100,
                    increase % 100,
                    cap / 100,
                    cap % 100
                ));
                Severity::RentIncreaseWithinLocalCapAllowed
            } else {
                notes.push(format!(
                    "Proposed rent increase {}.{:02}% exceeds local cap {}.{:02}% — presumed unreasonable.",
                    increase / 100,
                    increase % 100,
                    cap / 100,
                    cap % 100
                ));
                Severity::RentIncreaseExceedsLocalCapPresumedUnreasonable
            };
            Output {
                severity,
                rent_cap_percent_x_100: cap,
                proposed_increase_percent_x_100: increase,
                proposed_rent_within_cap: within_cap,
                notes,
                citations,
            }
        }
        ActionType::NonRenewalOrEviction => {
            let severity = if input.landlord_claims_good_cause
                && input.landlord_cause_is_statutorily_enumerated
            {
                notes.push("Landlord asserted statutorily-enumerated good cause; eviction permitted subject to procedural compliance.".to_string());
                Severity::GoodCauseLandlordHasValidReasonEvictionAllowed
            } else {
                notes.push("No statutorily-enumerated good cause asserted; eviction barred in this regime.".to_string());
                Severity::GoodCauseLandlordHasNoValidReasonEvictionBarred
            };
            Output {
                severity,
                rent_cap_percent_x_100: cap,
                proposed_increase_percent_x_100: percent_change(input.current_rent_cents, input.proposed_rent_cents),
                proposed_rent_within_cap: true,
                notes,
                citations,
            }
        }
    }
}

fn percent_change(current: u64, proposed: u64) -> u32 {
    if current == 0 || proposed <= current {
        return 0;
    }
    let delta = proposed - current;
    let pct_x_100 = (delta as u128).saturating_mul(10_000) / (current as u128);
    pct_x_100.min(u32::MAX as u128) as u32
}

fn compute_rent_cap(j: Jurisdiction, cpi_x_100: u32) -> u32 {
    match j {
        Jurisdiction::NewYorkGoodCauseEviction2024 => (NY_RENT_INCREASE_BASE_PERCENT_X_100
            .saturating_add(cpi_x_100))
        .min(NY_RENT_INCREASE_MAX_CAP_PERCENT_X_100),
        Jurisdiction::CaliforniaTpaAb1482 => (CA_RENT_INCREASE_BASE_PERCENT_X_100
            .saturating_add(cpi_x_100))
        .min(CA_RENT_INCREASE_MAX_CAP_PERCENT_X_100),
        Jurisdiction::OregonSb608 => OR_RENT_INCREASE_BASE_PERCENT_X_100.saturating_add(cpi_x_100),
        Jurisdiction::WashingtonHb2114 => {
            WA_RENT_INCREASE_BASE_PERCENT_X_100.saturating_add(cpi_x_100)
        }
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_ny_rent_increase() -> Input {
        Input {
            jurisdiction: Jurisdiction::NewYorkGoodCauseEviction2024,
            action_type: ActionType::RentIncrease,
            current_rent_cents: 200_000,
            proposed_rent_cents: 215_000,
            local_cpi_percent_x_100: 382,
            landlord_unit_count_in_state: 50,
            property_year_built: 1990,
            subsidized_or_rent_stabilized: false,
            owner_occupied_building_unit_count: 0,
            landlord_claims_good_cause: false,
            landlord_cause_is_statutorily_enumerated: false,
        }
    }

    #[test]
    fn ny_rent_increase_within_882_pct_cap_allowed() {
        let out = check(&base_ny_rent_increase());
        assert_eq!(out.severity, Severity::RentIncreaseWithinLocalCapAllowed);
        assert_eq!(out.rent_cap_percent_x_100, 882);
        assert_eq!(out.proposed_increase_percent_x_100, 750);
        assert!(out.proposed_rent_within_cap);
    }

    #[test]
    fn ny_rent_increase_exceeds_cap_presumed_unreasonable() {
        let mut i = base_ny_rent_increase();
        i.proposed_rent_cents = 220_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::RentIncreaseExceedsLocalCapPresumedUnreasonable
        );
        assert!(!out.proposed_rent_within_cap);
    }

    #[test]
    fn ny_rent_increase_hard_ceiling_10_pct_with_high_cpi() {
        let mut i = base_ny_rent_increase();
        i.local_cpi_percent_x_100 = 800;
        let out = check(&i);
        assert_eq!(out.rent_cap_percent_x_100, 1_000);
    }

    #[test]
    fn ny_small_landlord_exempt_under_10_units() {
        let mut i = base_ny_rent_increase();
        i.landlord_unit_count_in_state = 5;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptSmallLandlordUnder10Units);
    }

    #[test]
    fn ny_small_landlord_boundary_exactly_10_units_exempt() {
        let mut i = base_ny_rent_increase();
        i.landlord_unit_count_in_state = 10;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptSmallLandlordUnder10Units);
    }

    #[test]
    fn ny_small_landlord_boundary_11_units_not_exempt() {
        let mut i = base_ny_rent_increase();
        i.landlord_unit_count_in_state = 11;
        let out = check(&i);
        assert_eq!(out.severity, Severity::RentIncreaseWithinLocalCapAllowed);
    }

    #[test]
    fn ny_new_construction_post_2009_exempt() {
        let mut i = base_ny_rent_increase();
        i.property_year_built = 2015;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptNewConstructionPostThresholdYear);
    }

    #[test]
    fn ny_2009_construction_boundary_not_exempt() {
        let mut i = base_ny_rent_increase();
        i.property_year_built = 2009;
        let out = check(&i);
        assert_eq!(out.severity, Severity::RentIncreaseWithinLocalCapAllowed);
    }

    #[test]
    fn ny_owner_occupied_under_11_units_exempt() {
        let mut i = base_ny_rent_increase();
        i.owner_occupied_building_unit_count = 8;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ExemptOwnerOccupiedUnderUnitThreshold
        );
    }

    #[test]
    fn ny_subsidized_rent_stabilized_exempt() {
        let mut i = base_ny_rent_increase();
        i.subsidized_or_rent_stabilized = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ExemptSubsidizedOrRentStabilizedHousing
        );
    }

    #[test]
    fn ca_ab_1482_rent_cap_with_cpi() {
        let mut i = base_ny_rent_increase();
        i.jurisdiction = Jurisdiction::CaliforniaTpaAb1482;
        i.local_cpi_percent_x_100 = 350;
        let out = check(&i);
        assert_eq!(out.rent_cap_percent_x_100, 850);
    }

    #[test]
    fn or_sb_608_new_construction_under_15_years_exempt() {
        let mut i = base_ny_rent_increase();
        i.jurisdiction = Jurisdiction::OregonSb608;
        i.property_year_built = 2015;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptNewConstructionPostThresholdYear);
    }

    #[test]
    fn or_sb_608_old_property_caps_at_7_plus_cpi() {
        let mut i = base_ny_rent_increase();
        i.jurisdiction = Jurisdiction::OregonSb608;
        i.property_year_built = 1990;
        i.local_cpi_percent_x_100 = 300;
        let out = check(&i);
        assert_eq!(out.rent_cap_percent_x_100, 1_000);
    }

    #[test]
    fn nj_anti_eviction_act_good_cause_enumerated_allowed() {
        let mut i = base_ny_rent_increase();
        i.jurisdiction = Jurisdiction::NewJerseyAntiEvictionAct1974;
        i.action_type = ActionType::NonRenewalOrEviction;
        i.landlord_claims_good_cause = true;
        i.landlord_cause_is_statutorily_enumerated = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::GoodCauseLandlordHasValidReasonEvictionAllowed
        );
    }

    #[test]
    fn nj_anti_eviction_act_no_good_cause_eviction_barred() {
        let mut i = base_ny_rent_increase();
        i.jurisdiction = Jurisdiction::NewJerseyAntiEvictionAct1974;
        i.action_type = ActionType::NonRenewalOrEviction;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::GoodCauseLandlordHasNoValidReasonEvictionBarred
        );
    }

    #[test]
    fn default_jurisdiction_no_regime_falls_through() {
        let mut i = base_ny_rent_increase();
        i.jurisdiction = Jurisdiction::DefaultNoJustCauseRegime;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DefaultJurisdictionNoJustCauseRegime);
    }

    #[test]
    fn citations_pin_all_five_state_regimes() {
        let out = check(&base_ny_rent_increase());
        assert!(out.citations.iter().any(|c| c.contains("N.J.S.A. 2A:18-61.1")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1946.2")));
        assert!(out.citations.iter().any(|c| c.contains("ORS 90.323")));
        assert!(out.citations.iter().any(|c| c.contains("April 20, 2024")));
        assert!(out.citations.iter().any(|c| c.contains("RCW 59.18.650")));
    }

    #[test]
    fn constant_pin_ny_5_pct_base_x_100() {
        assert_eq!(NY_RENT_INCREASE_BASE_PERCENT_X_100, 500);
    }

    #[test]
    fn constant_pin_ny_10_pct_max_cap_x_100() {
        assert_eq!(NY_RENT_INCREASE_MAX_CAP_PERCENT_X_100, 1_000);
    }

    #[test]
    fn constant_pin_ny_small_landlord_10_units() {
        assert_eq!(NY_SMALL_LANDLORD_UNIT_THRESHOLD, 10);
    }

    #[test]
    fn constant_pin_ny_2009_construction_cutoff() {
        assert_eq!(NY_NEW_CONSTRUCTION_YEAR_CUTOFF, 2009);
    }

    #[test]
    fn constant_pin_or_15_year_new_construction_threshold() {
        assert_eq!(OR_NEW_CONSTRUCTION_AGE_THRESHOLD_YEARS, 15);
    }

    #[test]
    fn constant_pin_nj_18_enumerated_good_cause_grounds() {
        assert_eq!(NJ_GOOD_CAUSE_ENUMERATED_GROUNDS_COUNT, 18);
    }

    #[test]
    fn zero_current_rent_returns_zero_pct_change() {
        let mut i = base_ny_rent_increase();
        i.current_rent_cents = 0;
        let out = check(&i);
        assert_eq!(out.proposed_increase_percent_x_100, 0);
    }

    #[test]
    fn proposed_rent_below_current_no_increase() {
        let mut i = base_ny_rent_increase();
        i.proposed_rent_cents = 180_000;
        let out = check(&i);
        assert_eq!(out.proposed_increase_percent_x_100, 0);
    }
}
