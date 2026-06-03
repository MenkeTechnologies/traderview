//! Multi-jurisdiction eviction-record sealing and tenant-screening
//! compliance for trader-landlords using consumer reporting agencies
//! (CRAs) to screen rental applicants.
//!
//! Five major state regimes pinned with statutory citations and the
//! operative sealing or lookback-period rule that each one applies:
//!
//! - **CA AB 2819 (Chiu, 2016) — Cal. Code Civ. Proc. § 1161.2**:
//!   effective January 1, 2017. Limited-jurisdiction (under $35,000)
//!   unlawful-detainer court records are automatically masked from
//!   public access for 60 days after complaint filing. Records become
//!   PERMANENTLY sealed unless the landlord prevails within 60 days
//!   of filing the eviction complaint. Tenant screening reports
//!   cannot include sealed records.
//!
//! - **WA SB 5160 (2021) — RCW 59.18.367**: limits tenant-screening
//!   companies from reporting eviction records OLDER THAN 3 YEARS.
//!   Required screening criteria and adverse-action notice provisions
//!   stack with the lookback limit.
//!
//! - **NY RPAPL § 745(2)(c)(iv)**: state-of-art tenant-protection
//!   floor; tenant-screening services cannot consider housing-court
//!   records OLDER THAN 5 YEARS for adverse decisions.
//!
//! - **IL Eviction Sealing Pilot (HB 1561, 2022)**: codified at 735
//!   ILCS 5/9-121.5 (Cook County pilot, three-year sunset extended);
//!   eviction-case files may be sealed by court order with
//!   defendant-friendly presumption.
//!
//! - **MN Eviction Expungement Statute (Minn. Stat. § 484.014)**:
//!   automatic expungement after 3 years for evictions where landlord
//!   did not prevail; discretionary expungement by court order in
//!   other cases.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const CA_AB_2819_MASKING_DAYS: u32 = 60;
#[allow(dead_code)]
pub const CA_AB_2819_EFFECTIVE_YEAR: u32 = 2017;
#[allow(dead_code)]
pub const CA_LIMITED_JURISDICTION_AMOUNT_CENTS: u64 = 3_500_000;
#[allow(dead_code)]
pub const WA_LOOKBACK_LIMIT_YEARS: u32 = 3;
#[allow(dead_code)]
pub const NY_LOOKBACK_LIMIT_YEARS: u32 = 5;
#[allow(dead_code)]
pub const MN_AUTOMATIC_EXPUNGEMENT_YEARS: u32 = 3;
#[allow(dead_code)]
pub const CURRENT_YEAR: u32 = 2026;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    CaliforniaAb2819CcpSection1161_2,
    WashingtonSb5160Rcw59_18_367,
    NewYorkRpapl745SubCivIv,
    IllinoisHb1561_735IlcsSection9_121_5,
    MinnesotaSection484_014,
    DefaultNoSealingRegime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvictionOutcome {
    LandlordPrevailedWithin60Days,
    LandlordPrevailedAfter60Days,
    TenantPrevailed,
    SettledOrDismissed,
    PendingNoOutcomeYet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantTenantScreeningExcludesSealedEvictionRecord,
    CompliantMaskedRecordNotReportedAsLandlordPrevailedWithin60Days,
    ViolationReliedOnSealedEvictionRecordForDenial,
    ViolationReliedOnEvictionRecordBeyondJurisdictionLookback,
    ViolationReliedOnAutomaticallyExpungedRecord,
    DefaultJurisdictionNoSealingRegime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub years_since_eviction_filing: u32,
    pub days_since_eviction_filing: u32,
    pub eviction_outcome: EvictionOutcome,
    pub amount_in_controversy_cents: u64,
    pub landlord_relied_on_record_for_adverse_action: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub record_is_sealed_or_expunged: bool,
    pub record_beyond_lookback_period: bool,
    pub adverse_action_lawful: bool,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type EvictionRecordSealingInput = Input;
pub type EvictionRecordSealingOutput = Output;
pub type EvictionRecordSealingResult = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "CA AB 2819 (Chiu, 2016) — Cal. Code Civ. Proc. § 1161.2".to_string(),
        "WA SB 5160 (2021) — RCW 59.18.367".to_string(),
        "NY RPAPL § 745(2)(c)(iv)".to_string(),
        "IL HB 1561 (2022) — 735 ILCS 5/9-121.5".to_string(),
        "MN Minn. Stat. § 484.014 (eviction expungement)".to_string(),
        "PolicyLink — Eviction Records and Tenant Screening Protections".to_string(),
        "Urban Institute — Masking the Scarlet E (2023)".to_string(),
        "FCRA 15 U.S.C. § 1681c (federal adverse-action notice floor)".to_string(),
    ];

    if matches!(
        input.jurisdiction,
        Jurisdiction::DefaultNoSealingRegime
    ) {
        notes.push("Jurisdiction has no statewide eviction-record sealing regime; FCRA 7-year reporting cap applies as federal floor.".to_string());
        return Output {
            severity: Severity::DefaultJurisdictionNoSealingRegime,
            record_is_sealed_or_expunged: false,
            record_beyond_lookback_period: false,
            adverse_action_lawful: true,
            notes,
            citations,
        };
    }

    let sealed_or_expunged = is_record_sealed_or_expunged(input);
    let beyond_lookback = is_record_beyond_lookback(input);

    if !input.landlord_relied_on_record_for_adverse_action {
        if sealed_or_expunged {
            notes.push("Sealed/expunged record properly excluded from screening; no adverse action taken.".to_string());
            return Output {
                severity: Severity::CompliantTenantScreeningExcludesSealedEvictionRecord,
                record_is_sealed_or_expunged: true,
                record_beyond_lookback_period: beyond_lookback,
                adverse_action_lawful: true,
                notes,
                citations,
            };
        }
        if matches!(input.jurisdiction, Jurisdiction::CaliforniaAb2819CcpSection1161_2)
            && matches!(input.eviction_outcome, EvictionOutcome::LandlordPrevailedWithin60Days)
        {
            notes.push(format!(
                "CA AB 2819: landlord prevailed within {}-day window; record unmasked and reportable.",
                CA_AB_2819_MASKING_DAYS
            ));
            return Output {
                severity: Severity::CompliantMaskedRecordNotReportedAsLandlordPrevailedWithin60Days,
                record_is_sealed_or_expunged: false,
                record_beyond_lookback_period: beyond_lookback,
                adverse_action_lawful: true,
                notes,
                citations,
            };
        }
        notes.push("No adverse action taken; record status preserved for review.".to_string());
        return Output {
            severity: Severity::CompliantTenantScreeningExcludesSealedEvictionRecord,
            record_is_sealed_or_expunged: sealed_or_expunged,
            record_beyond_lookback_period: beyond_lookback,
            adverse_action_lawful: true,
            notes,
            citations,
        };
    }

    if sealed_or_expunged {
        notes.push("Landlord relied on a sealed or expunged eviction record — per se statutory violation.".to_string());
        let severity = match input.jurisdiction {
            Jurisdiction::MinnesotaSection484_014 => Severity::ViolationReliedOnAutomaticallyExpungedRecord,
            _ => Severity::ViolationReliedOnSealedEvictionRecordForDenial,
        };
        return Output {
            severity,
            record_is_sealed_or_expunged: true,
            record_beyond_lookback_period: beyond_lookback,
            adverse_action_lawful: false,
            notes,
            citations,
        };
    }

    if beyond_lookback {
        let lookback = lookback_years(input.jurisdiction);
        notes.push(format!(
            "Eviction record {} years old exceeds {}-year statutory lookback; landlord may not rely on it for adverse action.",
            input.years_since_eviction_filing,
            lookback
        ));
        return Output {
            severity: Severity::ViolationReliedOnEvictionRecordBeyondJurisdictionLookback,
            record_is_sealed_or_expunged: false,
            record_beyond_lookback_period: true,
            adverse_action_lawful: false,
            notes,
            citations,
        };
    }

    notes.push("Landlord relied on a reportable, in-lookback-window eviction record; adverse action lawful.".to_string());
    Output {
        severity: Severity::CompliantTenantScreeningExcludesSealedEvictionRecord,
        record_is_sealed_or_expunged: false,
        record_beyond_lookback_period: false,
        adverse_action_lawful: true,
        notes,
        citations,
    }
}

fn is_record_sealed_or_expunged(input: &Input) -> bool {
    match input.jurisdiction {
        Jurisdiction::CaliforniaAb2819CcpSection1161_2 => match input.eviction_outcome {
            EvictionOutcome::LandlordPrevailedWithin60Days => false,
            EvictionOutcome::LandlordPrevailedAfter60Days => true,
            EvictionOutcome::TenantPrevailed => true,
            EvictionOutcome::SettledOrDismissed => true,
            EvictionOutcome::PendingNoOutcomeYet => {
                input.days_since_eviction_filing > CA_AB_2819_MASKING_DAYS
            }
        },
        Jurisdiction::MinnesotaSection484_014 => {
            input.years_since_eviction_filing >= MN_AUTOMATIC_EXPUNGEMENT_YEARS
                && !matches!(
                    input.eviction_outcome,
                    EvictionOutcome::LandlordPrevailedWithin60Days
                )
                && !matches!(
                    input.eviction_outcome,
                    EvictionOutcome::LandlordPrevailedAfter60Days
                )
        }
        _ => false,
    }
}

fn is_record_beyond_lookback(input: &Input) -> bool {
    input.years_since_eviction_filing >= lookback_years(input.jurisdiction)
}

fn lookback_years(j: Jurisdiction) -> u32 {
    match j {
        Jurisdiction::WashingtonSb5160Rcw59_18_367 => WA_LOOKBACK_LIMIT_YEARS,
        Jurisdiction::NewYorkRpapl745SubCivIv => NY_LOOKBACK_LIMIT_YEARS,
        Jurisdiction::MinnesotaSection484_014 => MN_AUTOMATIC_EXPUNGEMENT_YEARS,
        _ => u32::MAX,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_ca_screening() -> Input {
        Input {
            jurisdiction: Jurisdiction::CaliforniaAb2819CcpSection1161_2,
            years_since_eviction_filing: 1,
            days_since_eviction_filing: 100,
            eviction_outcome: EvictionOutcome::TenantPrevailed,
            amount_in_controversy_cents: 1_000_000,
            landlord_relied_on_record_for_adverse_action: false,
        }
    }

    #[test]
    fn ca_tenant_prevailed_record_permanently_sealed() {
        let out = check(&base_ca_screening());
        assert!(out.record_is_sealed_or_expunged);
        assert_eq!(
            out.severity,
            Severity::CompliantTenantScreeningExcludesSealedEvictionRecord
        );
    }

    #[test]
    fn ca_landlord_prevailed_within_60_days_record_unmasked() {
        let mut i = base_ca_screening();
        i.eviction_outcome = EvictionOutcome::LandlordPrevailedWithin60Days;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantMaskedRecordNotReportedAsLandlordPrevailedWithin60Days
        );
        assert!(!out.record_is_sealed_or_expunged);
    }

    #[test]
    fn ca_landlord_prevailed_after_60_days_record_permanently_sealed() {
        let mut i = base_ca_screening();
        i.eviction_outcome = EvictionOutcome::LandlordPrevailedAfter60Days;
        let out = check(&i);
        assert!(out.record_is_sealed_or_expunged);
    }

    #[test]
    fn ca_settled_or_dismissed_record_permanently_sealed() {
        let mut i = base_ca_screening();
        i.eviction_outcome = EvictionOutcome::SettledOrDismissed;
        let out = check(&i);
        assert!(out.record_is_sealed_or_expunged);
    }

    #[test]
    fn ca_pending_within_60_days_record_still_masked() {
        let mut i = base_ca_screening();
        i.eviction_outcome = EvictionOutcome::PendingNoOutcomeYet;
        i.days_since_eviction_filing = 30;
        let out = check(&i);
        assert!(!out.record_is_sealed_or_expunged);
    }

    #[test]
    fn ca_pending_past_60_days_record_now_sealed() {
        let mut i = base_ca_screening();
        i.eviction_outcome = EvictionOutcome::PendingNoOutcomeYet;
        i.days_since_eviction_filing = 61;
        let out = check(&i);
        assert!(out.record_is_sealed_or_expunged);
    }

    #[test]
    fn ca_landlord_relied_on_sealed_record_is_violation() {
        let mut i = base_ca_screening();
        i.landlord_relied_on_record_for_adverse_action = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationReliedOnSealedEvictionRecordForDenial
        );
        assert!(!out.adverse_action_lawful);
    }

    #[test]
    fn wa_record_within_3_year_lookback_reportable() {
        let mut i = base_ca_screening();
        i.jurisdiction = Jurisdiction::WashingtonSb5160Rcw59_18_367;
        i.eviction_outcome = EvictionOutcome::LandlordPrevailedAfter60Days;
        i.years_since_eviction_filing = 2;
        i.landlord_relied_on_record_for_adverse_action = true;
        let out = check(&i);
        assert!(out.adverse_action_lawful);
        assert!(!out.record_beyond_lookback_period);
    }

    #[test]
    fn wa_record_beyond_3_year_lookback_violation() {
        let mut i = base_ca_screening();
        i.jurisdiction = Jurisdiction::WashingtonSb5160Rcw59_18_367;
        i.eviction_outcome = EvictionOutcome::LandlordPrevailedAfter60Days;
        i.years_since_eviction_filing = 4;
        i.landlord_relied_on_record_for_adverse_action = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationReliedOnEvictionRecordBeyondJurisdictionLookback
        );
        assert!(!out.adverse_action_lawful);
    }

    #[test]
    fn wa_3_year_boundary_at_exactly_3_years_is_beyond() {
        let mut i = base_ca_screening();
        i.jurisdiction = Jurisdiction::WashingtonSb5160Rcw59_18_367;
        i.eviction_outcome = EvictionOutcome::LandlordPrevailedAfter60Days;
        i.years_since_eviction_filing = 3;
        i.landlord_relied_on_record_for_adverse_action = true;
        let out = check(&i);
        assert!(out.record_beyond_lookback_period);
    }

    #[test]
    fn ny_5_year_lookback_record_within_window_reportable() {
        let mut i = base_ca_screening();
        i.jurisdiction = Jurisdiction::NewYorkRpapl745SubCivIv;
        i.eviction_outcome = EvictionOutcome::LandlordPrevailedAfter60Days;
        i.years_since_eviction_filing = 4;
        i.landlord_relied_on_record_for_adverse_action = true;
        let out = check(&i);
        assert!(out.adverse_action_lawful);
    }

    #[test]
    fn ny_5_year_lookback_record_beyond_5_years_violation() {
        let mut i = base_ca_screening();
        i.jurisdiction = Jurisdiction::NewYorkRpapl745SubCivIv;
        i.eviction_outcome = EvictionOutcome::LandlordPrevailedAfter60Days;
        i.years_since_eviction_filing = 6;
        i.landlord_relied_on_record_for_adverse_action = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationReliedOnEvictionRecordBeyondJurisdictionLookback
        );
    }

    #[test]
    fn mn_3_year_automatic_expungement_when_landlord_did_not_prevail() {
        let mut i = base_ca_screening();
        i.jurisdiction = Jurisdiction::MinnesotaSection484_014;
        i.eviction_outcome = EvictionOutcome::SettledOrDismissed;
        i.years_since_eviction_filing = 3;
        i.landlord_relied_on_record_for_adverse_action = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationReliedOnAutomaticallyExpungedRecord
        );
        assert!(out.record_is_sealed_or_expunged);
    }

    #[test]
    fn mn_3_year_no_expungement_when_landlord_prevailed() {
        let mut i = base_ca_screening();
        i.jurisdiction = Jurisdiction::MinnesotaSection484_014;
        i.eviction_outcome = EvictionOutcome::LandlordPrevailedAfter60Days;
        i.years_since_eviction_filing = 3;
        i.landlord_relied_on_record_for_adverse_action = true;
        let out = check(&i);
        assert!(out.record_beyond_lookback_period);
    }

    #[test]
    fn default_jurisdiction_no_regime_falls_through() {
        let mut i = base_ca_screening();
        i.jurisdiction = Jurisdiction::DefaultNoSealingRegime;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DefaultJurisdictionNoSealingRegime);
        assert!(out.adverse_action_lawful);
    }

    #[test]
    fn citations_pin_all_five_state_regimes() {
        let out = check(&base_ca_screening());
        assert!(out.citations.iter().any(|c| c.contains("AB 2819")));
        assert!(out.citations.iter().any(|c| c.contains("RCW 59.18.367")));
        assert!(out.citations.iter().any(|c| c.contains("RPAPL § 745")));
        assert!(out.citations.iter().any(|c| c.contains("735 ILCS 5/9-121.5")));
        assert!(out.citations.iter().any(|c| c.contains("§ 484.014")));
    }

    #[test]
    fn citations_pin_fcra_federal_floor() {
        let out = check(&base_ca_screening());
        assert!(out.citations.iter().any(|c| c.contains("§ 1681c")));
    }

    #[test]
    fn constant_pin_ca_60_day_masking_window() {
        assert_eq!(CA_AB_2819_MASKING_DAYS, 60);
    }

    #[test]
    fn constant_pin_wa_3_year_lookback() {
        assert_eq!(WA_LOOKBACK_LIMIT_YEARS, 3);
    }

    #[test]
    fn constant_pin_ny_5_year_lookback() {
        assert_eq!(NY_LOOKBACK_LIMIT_YEARS, 5);
    }

    #[test]
    fn constant_pin_mn_3_year_automatic_expungement() {
        assert_eq!(MN_AUTOMATIC_EXPUNGEMENT_YEARS, 3);
    }

    #[test]
    fn constant_pin_ca_ab_2819_effective_2017() {
        assert_eq!(CA_AB_2819_EFFECTIVE_YEAR, 2017);
    }

    #[test]
    fn ca_landlord_did_not_rely_on_sealed_record_compliant() {
        let mut i = base_ca_screening();
        i.eviction_outcome = EvictionOutcome::TenantPrevailed;
        i.landlord_relied_on_record_for_adverse_action = false;
        let out = check(&i);
        assert!(out.adverse_action_lawful);
        assert_eq!(
            out.severity,
            Severity::CompliantTenantScreeningExcludesSealedEvictionRecord
        );
    }
}
