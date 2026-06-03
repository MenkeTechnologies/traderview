//! Landlord retaliation prohibition framework.
//!
//! Every state with a residential tenancy code prohibits landlord retaliation against
//! tenants for exercising protected rights: complaining to the landlord or a government
//! agency about habitability, joining a tenants' organization, or invoking statutory
//! protections (rent withholding, repair-and-deduct, fair-housing complaint). The
//! prohibition operates through a rebuttable presumption: any adverse landlord action
//! (eviction, rent increase, service decrease, refusal to renew) within the presumption
//! window after a protected tenant act is presumed retaliatory; the landlord must rebut
//! by proving a lawful, non-retaliatory motive.
//!
//! Presumption windows and damages vary sharply:
//!
//! - CA Civ. Code § 1942.5: 180-day presumption + $100-$2,000 per retaliatory act
//!   punitive damages (showing fraud/oppression/malice) + attorney fees.
//! - NY RPL § 223-b: 1-year presumption (3 years for rent-stabilized/controlled units)
//!   + tenant defense to eviction + reinstatement.
//! - WA RCW 59.18.240 + § 59.18.250: 90-day presumption + tenant remedies under RCW
//!   59.18.060.
//! - TX Prop. Code §§ 92.331-92.335: 6-month presumption + $500 statutory civil penalty
//!   + one month's rent + actual damages + attorney fees + court costs.
//! - FL Stat. § 83.64: rebuttable presumption (no statutory window — common-law
//!   reasonable temporal proximity); defense to eviction + actual damages.
//! - IL Chicago RLTO § 5-12-150: 1-year presumption + two months' rent OR twice damages
//!   (greater) + attorney fees for Chicago-located rentals. Illinois statewide has no
//!   codified anti-retaliation statute; common-law retaliatory-eviction doctrine via
//!   Clore v. Fredman, 59 Ill. 2d 20 (1974) operates as defense to forcible-entry.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - nolo.com/legal-encyclopedia/state-laws-prohibiting-landlord-retaliation.html
//! - leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?sectionNum=1942.5.
//! - codelibrary.amlegal.com/codes/chicago/latest/chicago_il/0-0-0-2639270
//! - codes.findlaw.com/ca/civil-code/civ-sect-1942-5/

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    NewYork,
    NewYorkRentStabilizedOrControlled,
    Washington,
    Texas,
    Florida,
    IllinoisChicagoRlto,
    IllinoisStatewideCommonLawOnly,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProtectedTenantAct {
    ComplaintToLandlordAboutHabitability,
    ComplaintToGovernmentAgencyHousingCodeOrHealth,
    JoinedTenantsAssociationOrOrganization,
    ExercisedRepairAndDeductOrRentWithholdRight,
    FiledFairHousingDiscriminationComplaint,
    LegalProceedingAgainstLandlord,
    NoProtectedActExercised,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LandlordAction {
    EvictionOrTerminationServed,
    RentIncreaseImposed,
    ServiceDecreaseOrMaintenanceWithheld,
    RefusalToRenewLease,
    HarassmentOrThreatsToTenant,
    NoAdverseAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LandlordRebuttalEvidence {
    /// No rebuttal evidence offered.
    None,
    /// Documented legitimate business reason (non-payment, lease violation, owner
    /// move-in, withdrawal from rental market per Ellis Act in California).
    DocumentedLegitimateBusinessReason,
    /// Action commenced BEFORE the protected tenant act — temporal sequence rebuts.
    ActionPredatesTenantProtectedAct,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NoProtectedActNoPresumptionTriggered,
    NoAdverseActionNoRetaliationClaim,
    LandlordRebuttalProbablySuccessful,
    OutsidePresumptionWindowBurdenOnTenant,
    CaliforniaCiv1942_5PresumptionRetaliationPunitiveDamages,
    NewYorkRpl223BOneYearPresumption,
    NewYorkRpl223BThreeYearPresumptionRentStabilized,
    WashingtonRcw59_18_240NinetyDayPresumption,
    TexasPropCode92_331SixMonthPresumptionFiveHundredPenalty,
    FloridaStat83_64CommonLawTemporalProximityPresumption,
    IllinoisChicagoRlto5_12_150OneYearPresumptionTwoMonthRent,
    IllinoisStatewideCommonLawCloreVFredmanDefense,
    DefaultJurisdictionStateLawRetaliationPresumption,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub protected_tenant_act: ProtectedTenantAct,
    pub landlord_action: LandlordAction,
    pub days_between_protected_act_and_landlord_action: u32,
    pub monthly_rent_cents: u64,
    pub tenant_actual_damages_cents: u64,
    pub landlord_rebuttal_evidence: LandlordRebuttalEvidence,
}

pub type RentalRetaliationProhibitionInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub presumption_window_days: u32,
    pub estimated_landlord_exposure_cents: u64,
    pub note: String,
}

pub type RentalRetaliationProhibitionOutput = Output;
pub type RentalRetaliationProhibitionResult = Output;

const CA_PRESUMPTION_WINDOW_DAYS: u32 = 180;
#[allow(dead_code)]
const CA_PUNITIVE_DAMAGE_MIN_CENTS: u64 = 10_000;
const CA_PUNITIVE_DAMAGE_MAX_CENTS: u64 = 200_000;
const NY_PRESUMPTION_WINDOW_DAYS: u32 = 365;
const NY_RENT_STABILIZED_PRESUMPTION_WINDOW_DAYS: u32 = 1_095;
const WA_PRESUMPTION_WINDOW_DAYS: u32 = 90;
const TX_PRESUMPTION_WINDOW_DAYS: u32 = 180;
const TX_STATUTORY_CIVIL_PENALTY_CENTS: u64 = 50_000;
const IL_CHICAGO_RLTO_PRESUMPTION_WINDOW_DAYS: u32 = 365;
const IL_CHICAGO_RLTO_TWO_MONTH_MULTIPLIER: u64 = 2;
const DEFAULT_PRESUMPTION_WINDOW_DAYS: u32 = 180;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(input.protected_tenant_act, ProtectedTenantAct::NoProtectedActExercised) {
        return Output {
            severity: Severity::NoProtectedActNoPresumptionTriggered,
            presumption_window_days: 0,
            estimated_landlord_exposure_cents: 0,
            note: "No protected tenant act asserted — retaliation presumption not triggered. \
                   Statutory presumption arises only when tenant has exercised one of: \
                   habitability complaint to landlord/government, joined tenants' association, \
                   exercised repair-and-deduct or rent-withhold right, filed fair-housing \
                   complaint, or commenced legal proceeding against landlord."
                .to_string(),
        };
    }

    if matches!(input.landlord_action, LandlordAction::NoAdverseAction) {
        return Output {
            severity: Severity::NoAdverseActionNoRetaliationClaim,
            presumption_window_days: 0,
            estimated_landlord_exposure_cents: 0,
            note: "No adverse landlord action reported — retaliation claim requires an \
                   eviction, termination, rent increase, service decrease, refusal to renew, \
                   or harassment. Confirm no adverse action occurred during the presumption \
                   window."
                .to_string(),
        };
    }

    if matches!(
        input.landlord_rebuttal_evidence,
        LandlordRebuttalEvidence::ActionPredatesTenantProtectedAct
    ) {
        return Output {
            severity: Severity::LandlordRebuttalProbablySuccessful,
            presumption_window_days: 0,
            estimated_landlord_exposure_cents: 0,
            note: "Landlord rebuttal evidence: adverse action commenced BEFORE protected \
                   tenant act. Temporal sequence rebuts the retaliation presumption — \
                   landlord cannot retaliate for an act that had not yet occurred. Document \
                   the eviction-notice service date, rent-increase notice date, or \
                   non-renewal letter date with date-stamped records (USPS proof of mailing, \
                   certified-mail receipt, served-notice declaration)."
                .to_string(),
        };
    }

    let presumption_window = match input.jurisdiction {
        Jurisdiction::California => CA_PRESUMPTION_WINDOW_DAYS,
        Jurisdiction::NewYork => NY_PRESUMPTION_WINDOW_DAYS,
        Jurisdiction::NewYorkRentStabilizedOrControlled => {
            NY_RENT_STABILIZED_PRESUMPTION_WINDOW_DAYS
        }
        Jurisdiction::Washington => WA_PRESUMPTION_WINDOW_DAYS,
        Jurisdiction::Texas => TX_PRESUMPTION_WINDOW_DAYS,
        Jurisdiction::Florida => DEFAULT_PRESUMPTION_WINDOW_DAYS,
        Jurisdiction::IllinoisChicagoRlto => IL_CHICAGO_RLTO_PRESUMPTION_WINDOW_DAYS,
        Jurisdiction::IllinoisStatewideCommonLawOnly => DEFAULT_PRESUMPTION_WINDOW_DAYS,
        Jurisdiction::Default => DEFAULT_PRESUMPTION_WINDOW_DAYS,
    };

    if input.days_between_protected_act_and_landlord_action > presumption_window {
        return Output {
            severity: Severity::OutsidePresumptionWindowBurdenOnTenant,
            presumption_window_days: presumption_window,
            estimated_landlord_exposure_cents: 0,
            note: format!(
                "Adverse landlord action occurred {} days after protected tenant act — \
                 outside the {}-day presumption window. Tenant retains common-law retaliation \
                 claim but must prove retaliatory motive directly (no statutory presumption \
                 shifts the burden to landlord). Substantially harder to prove.",
                input.days_between_protected_act_and_landlord_action, presumption_window
            ),
        };
    }

    match input.jurisdiction {
        Jurisdiction::California => {
            let punitive_estimate = CA_PUNITIVE_DAMAGE_MAX_CENTS;
            let exposure = punitive_estimate.saturating_add(input.tenant_actual_damages_cents);
            Output {
                severity: Severity::CaliforniaCiv1942_5PresumptionRetaliationPunitiveDamages,
                presumption_window_days: CA_PRESUMPTION_WINDOW_DAYS,
                estimated_landlord_exposure_cents: exposure,
                note: format!(
                    "California Civ. Code § 1942.5 retaliation presumption applies. Adverse \
                     action within 180-day window after protected tenant act. Punitive damages \
                     range $100-$2,000 per retaliatory act on showing fraud, oppression, or \
                     malice (modeled at max $2,000 = ${}) + actual damages (${}) + reasonable \
                     attorney fees + tenant defense to eviction. Estimated exposure ${} \
                     excludes attorney fees. Ellis Act withdrawal from rental market is one \
                     of few documented legitimate-business-reason rebuttals.",
                    punitive_estimate / 100,
                    input.tenant_actual_damages_cents / 100,
                    exposure / 100
                ),
            }
        }
        Jurisdiction::NewYork => Output {
            severity: Severity::NewYorkRpl223BOneYearPresumption,
            presumption_window_days: NY_PRESUMPTION_WINDOW_DAYS,
            estimated_landlord_exposure_cents: input.tenant_actual_damages_cents,
            note: format!(
                "New York RPL § 223-b retaliation presumption applies. Adverse action within \
                 1-year window after protected tenant act creates rebuttable presumption. \
                 Tenant defense to eviction + reinstatement + actual damages (${}). NY does \
                 NOT codify per-violation civil penalty; recovery is actual damages + attorney \
                 fees where statute permits. Rent-stabilized / rent-controlled units have \
                 EXPANDED 3-year presumption window — use NewYorkRentStabilizedOrControlled \
                 jurisdiction for stabilized units.",
                input.tenant_actual_damages_cents / 100
            ),
        },
        Jurisdiction::NewYorkRentStabilizedOrControlled => Output {
            severity: Severity::NewYorkRpl223BThreeYearPresumptionRentStabilized,
            presumption_window_days: NY_RENT_STABILIZED_PRESUMPTION_WINDOW_DAYS,
            estimated_landlord_exposure_cents: input.tenant_actual_damages_cents,
            note: format!(
                "New York RPL § 223-b EXPANDED 3-year (1,095-day) presumption window applies \
                 to rent-stabilized and rent-controlled units. Statute creates strong \
                 burden-shift onto landlord. Tenant defense to eviction + reinstatement + \
                 value of permanently lost stabilized tenancy (often six figures) + actual \
                 damages (${}) + attorney fees where statute permits.",
                input.tenant_actual_damages_cents / 100
            ),
        },
        Jurisdiction::Washington => {
            let triple_rent = input.monthly_rent_cents.saturating_mul(3);
            let exposure = triple_rent.max(input.tenant_actual_damages_cents);
            Output {
                severity: Severity::WashingtonRcw59_18_240NinetyDayPresumption,
                presumption_window_days: WA_PRESUMPTION_WINDOW_DAYS,
                estimated_landlord_exposure_cents: exposure,
                note: format!(
                    "Washington RCW 59.18.240 retaliation presumption applies. Adverse action \
                     within 90-day window after protected tenant act creates rebuttable \
                     presumption (shorter window than CA, NY, TX, IL). Remedies under RCW \
                     59.18.060: greater of actual damages (${}) or 3× monthly rent (${} × 3 \
                     = ${}) + attorney fees. Estimated exposure ${} excludes attorney fees.",
                    input.tenant_actual_damages_cents / 100,
                    input.monthly_rent_cents / 100,
                    triple_rent / 100,
                    exposure / 100
                ),
            }
        }
        Jurisdiction::Texas => {
            let exposure = TX_STATUTORY_CIVIL_PENALTY_CENTS
                .saturating_add(input.monthly_rent_cents)
                .saturating_add(input.tenant_actual_damages_cents);
            Output {
                severity: Severity::TexasPropCode92_331SixMonthPresumptionFiveHundredPenalty,
                presumption_window_days: TX_PRESUMPTION_WINDOW_DAYS,
                estimated_landlord_exposure_cents: exposure,
                note: format!(
                    "Texas Prop. Code §§ 92.331-92.335 retaliation presumption applies. \
                     Adverse action within 180-day (6-month) window after protected tenant \
                     act creates rebuttable presumption. Statutory civil penalty $500 (${}) \
                     + one month rent (${}) + actual damages (${}) + court costs + attorney \
                     fees. Estimated exposure ${} excludes attorney fees. Texas defines \
                     retaliation narrowly — landlord may rebut with documented bona-fide \
                     justification.",
                    TX_STATUTORY_CIVIL_PENALTY_CENTS / 100,
                    input.monthly_rent_cents / 100,
                    input.tenant_actual_damages_cents / 100,
                    exposure / 100
                ),
            }
        }
        Jurisdiction::Florida => Output {
            severity: Severity::FloridaStat83_64CommonLawTemporalProximityPresumption,
            presumption_window_days: DEFAULT_PRESUMPTION_WINDOW_DAYS,
            estimated_landlord_exposure_cents: input.tenant_actual_damages_cents,
            note: format!(
                "Florida Stat. § 83.64 retaliatory conduct prohibition applies. Statute does \
                 NOT codify a presumption window; common-law reasonable-temporal-proximity \
                 analysis applies. Tenant defense to eviction + actual damages (${}) + \
                 attorney fees where statute permits. Florida courts apply 90- to 180-day \
                 reasonable-window heuristic but no statutory cutoff.",
                input.tenant_actual_damages_cents / 100
            ),
        },
        Jurisdiction::IllinoisChicagoRlto => {
            let two_month_rent =
                input.monthly_rent_cents.saturating_mul(IL_CHICAGO_RLTO_TWO_MONTH_MULTIPLIER);
            let twice_damages = input.tenant_actual_damages_cents.saturating_mul(2);
            let exposure = two_month_rent.max(twice_damages);
            Output {
                severity:
                    Severity::IllinoisChicagoRlto5_12_150OneYearPresumptionTwoMonthRent,
                presumption_window_days: IL_CHICAGO_RLTO_PRESUMPTION_WINDOW_DAYS,
                estimated_landlord_exposure_cents: exposure,
                note: format!(
                    "Chicago Residential Landlord and Tenant Ordinance (RLTO) § 5-12-150 \
                     retaliation presumption applies. Adverse action within 1-year window \
                     after protected tenant act creates rebuttable presumption (Chicago RLTO \
                     codifies stricter standard than IL statewide). Tenant recovers GREATER \
                     OF two months' rent (${} × 2 = ${}) OR twice damages (${} × 2 = ${}) + \
                     attorney fees. Tenant may recover possession or terminate rental \
                     agreement. Estimated exposure ${} excludes attorney fees.",
                    input.monthly_rent_cents / 100,
                    two_month_rent / 100,
                    input.tenant_actual_damages_cents / 100,
                    twice_damages / 100,
                    exposure / 100
                ),
            }
        }
        Jurisdiction::IllinoisStatewideCommonLawOnly => Output {
            severity: Severity::IllinoisStatewideCommonLawCloreVFredmanDefense,
            presumption_window_days: DEFAULT_PRESUMPTION_WINDOW_DAYS,
            estimated_landlord_exposure_cents: input.tenant_actual_damages_cents,
            note: format!(
                "Illinois statewide has NO codified anti-retaliation statute. Common-law \
                 retaliatory-eviction doctrine via Clore v. Fredman, 59 Ill. 2d 20 (1974) \
                 operates as defense to forcible-entry-and-detainer action under 735 ILCS \
                 5/9-101 et seq. No statutory presumption window; tenant must prove \
                 retaliatory motive. Actual damages (${}) + attorney fees where contract \
                 permits. Chicago-located rentals get codified protection via RLTO § 5-12-150 \
                 — use IllinoisChicagoRlto jurisdiction for Chicago units.",
                input.tenant_actual_damages_cents / 100
            ),
        },
        Jurisdiction::Default => Output {
            severity: Severity::DefaultJurisdictionStateLawRetaliationPresumption,
            presumption_window_days: DEFAULT_PRESUMPTION_WINDOW_DAYS,
            estimated_landlord_exposure_cents: input.tenant_actual_damages_cents,
            note: format!(
                "Most US states codify a landlord retaliation prohibition with rebuttable \
                 presumption window (90-365 days post-protected-act). Specific penalty regime \
                 depends on state statute (CA 180d + $100-$2K punitives, NY 1yr / 3yr \
                 stabilized, WA 90d + 3× rent, TX 6mo + $500 + month rent, FL common-law \
                 temporal proximity, IL Chicago 1yr + 2-month rent). Default 180-day window \
                 assumed. Confirm state-law exposure with local counsel. Estimated baseline \
                 exposure includes tenant actual damages (${}).",
                input.tenant_actual_damages_cents / 100
            ),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            protected_tenant_act: ProtectedTenantAct::ComplaintToGovernmentAgencyHousingCodeOrHealth,
            landlord_action: LandlordAction::EvictionOrTerminationServed,
            days_between_protected_act_and_landlord_action: 30,
            monthly_rent_cents: 3_000_00,
            tenant_actual_damages_cents: 5_000_00,
            landlord_rebuttal_evidence: LandlordRebuttalEvidence::None,
        }
    }

    #[test]
    fn no_protected_act_no_presumption_triggered() {
        let mut input = base();
        input.protected_tenant_act = ProtectedTenantAct::NoProtectedActExercised;
        let output = check(&input);
        assert_eq!(output.severity, Severity::NoProtectedActNoPresumptionTriggered);
        assert_eq!(output.estimated_landlord_exposure_cents, 0);
    }

    #[test]
    fn no_adverse_action_no_retaliation_claim() {
        let mut input = base();
        input.landlord_action = LandlordAction::NoAdverseAction;
        let output = check(&input);
        assert_eq!(output.severity, Severity::NoAdverseActionNoRetaliationClaim);
        assert_eq!(output.estimated_landlord_exposure_cents, 0);
    }

    #[test]
    fn landlord_rebuttal_action_predates_protected_act_rebuts_presumption() {
        let mut input = base();
        input.landlord_rebuttal_evidence =
            LandlordRebuttalEvidence::ActionPredatesTenantProtectedAct;
        let output = check(&input);
        assert_eq!(output.severity, Severity::LandlordRebuttalProbablySuccessful);
        assert!(output.note.contains("BEFORE"));
        assert!(output.note.contains("Temporal sequence"));
    }

    #[test]
    fn california_within_180_day_window_triggers_civ_1942_5_presumption() {
        let input = base();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CaliforniaCiv1942_5PresumptionRetaliationPunitiveDamages
        );
        assert_eq!(output.presumption_window_days, 180);
        // $2,000 max punitive + $5,000 actual = $7,000
        assert_eq!(output.estimated_landlord_exposure_cents, 7_000_00);
        assert!(output.note.contains("§ 1942.5"));
        assert!(output.note.contains("Ellis Act"));
    }

    #[test]
    fn california_outside_180_day_window_burden_on_tenant() {
        let mut input = base();
        input.days_between_protected_act_and_landlord_action = 181;
        let output = check(&input);
        assert_eq!(output.severity, Severity::OutsidePresumptionWindowBurdenOnTenant);
        assert_eq!(output.presumption_window_days, 180);
        assert_eq!(output.estimated_landlord_exposure_cents, 0);
    }

    #[test]
    fn california_at_180_day_boundary_still_within_window() {
        let mut input = base();
        input.days_between_protected_act_and_landlord_action = 180;
        let output = check(&input);
        // > 180 returns outside; exactly 180 should still trigger presumption
        assert_eq!(
            output.severity,
            Severity::CaliforniaCiv1942_5PresumptionRetaliationPunitiveDamages
        );
    }

    #[test]
    fn new_york_one_year_presumption_window_applies() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::NewYork;
        input.days_between_protected_act_and_landlord_action = 200;
        let output = check(&input);
        assert_eq!(output.severity, Severity::NewYorkRpl223BOneYearPresumption);
        assert_eq!(output.presumption_window_days, 365);
        assert!(output.note.contains("RPL § 223-b"));
    }

    #[test]
    fn new_york_rent_stabilized_three_year_expanded_presumption_window() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::NewYorkRentStabilizedOrControlled;
        input.days_between_protected_act_and_landlord_action = 800;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NewYorkRpl223BThreeYearPresumptionRentStabilized
        );
        assert_eq!(output.presumption_window_days, 1_095);
        assert!(output.note.contains("3-year"));
        assert!(output.note.contains("rent-stabilized"));
    }

    #[test]
    fn washington_ninety_day_window_triggers_rcw_59_18_240() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Washington;
        input.days_between_protected_act_and_landlord_action = 60;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::WashingtonRcw59_18_240NinetyDayPresumption
        );
        assert_eq!(output.presumption_window_days, 90);
        // 3 × $3,000 = $9,000 > $5,000 actual
        assert_eq!(output.estimated_landlord_exposure_cents, 9_000_00);
        assert!(output.note.contains("RCW 59.18.240"));
    }

    #[test]
    fn washington_91_days_outside_presumption_burden_on_tenant() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Washington;
        input.days_between_protected_act_and_landlord_action = 91;
        let output = check(&input);
        assert_eq!(output.severity, Severity::OutsidePresumptionWindowBurdenOnTenant);
    }

    #[test]
    fn texas_six_month_window_triggers_500_dollar_penalty_plus_month_rent() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Texas;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::TexasPropCode92_331SixMonthPresumptionFiveHundredPenalty
        );
        assert_eq!(output.presumption_window_days, 180);
        // $500 + $3,000 rent + $5,000 actual = $8,500
        assert_eq!(output.estimated_landlord_exposure_cents, 8_500_00);
        assert!(output.note.contains("§§ 92.331-92.335"));
    }

    #[test]
    fn florida_default_180_day_window_with_common_law_temporal_proximity() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Florida;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::FloridaStat83_64CommonLawTemporalProximityPresumption
        );
        assert_eq!(output.presumption_window_days, 180);
        assert!(output.note.contains("§ 83.64"));
        assert!(output.note.contains("temporal-proximity"));
    }

    #[test]
    fn illinois_chicago_rlto_one_year_window_two_month_rent_exposure() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::IllinoisChicagoRlto;
        input.days_between_protected_act_and_landlord_action = 200;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::IllinoisChicagoRlto5_12_150OneYearPresumptionTwoMonthRent
        );
        assert_eq!(output.presumption_window_days, 365);
        // 2 × $3,000 = $6,000; 2 × $5,000 damages = $10,000 → $10,000 greater
        assert_eq!(output.estimated_landlord_exposure_cents, 10_000_00);
        assert!(output.note.contains("§ 5-12-150"));
    }

    #[test]
    fn illinois_chicago_high_rent_two_month_rent_exceeds_double_damages() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::IllinoisChicagoRlto;
        input.monthly_rent_cents = 6_000_00;
        input.tenant_actual_damages_cents = 1_000_00;
        let output = check(&input);
        // 2 × $6,000 = $12,000 > 2 × $1,000 = $2,000
        assert_eq!(output.estimated_landlord_exposure_cents, 12_000_00);
    }

    #[test]
    fn illinois_statewide_common_law_clore_v_fredman_defense() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::IllinoisStatewideCommonLawOnly;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::IllinoisStatewideCommonLawCloreVFredmanDefense
        );
        assert!(output.note.contains("Clore v. Fredman"));
        assert!(output.note.contains("59 Ill. 2d 20"));
        assert!(output.note.contains("1974"));
    }

    #[test]
    fn default_jurisdiction_returns_state_law_presumption_note() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Default;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::DefaultJurisdictionStateLawRetaliationPresumption
        );
        assert!(output.note.contains("Most US states"));
    }

    #[test]
    fn habitability_complaint_to_landlord_triggers_protection() {
        let mut input = base();
        input.protected_tenant_act = ProtectedTenantAct::ComplaintToLandlordAboutHabitability;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CaliforniaCiv1942_5PresumptionRetaliationPunitiveDamages
        );
    }

    #[test]
    fn joined_tenants_association_triggers_protection() {
        let mut input = base();
        input.protected_tenant_act = ProtectedTenantAct::JoinedTenantsAssociationOrOrganization;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CaliforniaCiv1942_5PresumptionRetaliationPunitiveDamages
        );
    }

    #[test]
    fn rent_increase_qualifies_as_adverse_action() {
        let mut input = base();
        input.landlord_action = LandlordAction::RentIncreaseImposed;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CaliforniaCiv1942_5PresumptionRetaliationPunitiveDamages
        );
    }

    #[test]
    fn service_decrease_qualifies_as_adverse_action() {
        let mut input = base();
        input.landlord_action = LandlordAction::ServiceDecreaseOrMaintenanceWithheld;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CaliforniaCiv1942_5PresumptionRetaliationPunitiveDamages
        );
    }

    #[test]
    fn ca_presumption_window_constant_pins_180_days() {
        assert_eq!(CA_PRESUMPTION_WINDOW_DAYS, 180);
    }

    #[test]
    fn ca_punitive_damage_min_constant_pins_100_dollars() {
        assert_eq!(CA_PUNITIVE_DAMAGE_MIN_CENTS, 10_000);
    }

    #[test]
    fn ca_punitive_damage_max_constant_pins_2000_dollars() {
        assert_eq!(CA_PUNITIVE_DAMAGE_MAX_CENTS, 200_000);
    }

    #[test]
    fn ny_presumption_window_constant_pins_365_days() {
        assert_eq!(NY_PRESUMPTION_WINDOW_DAYS, 365);
    }

    #[test]
    fn ny_rent_stabilized_presumption_window_constant_pins_1095_days() {
        assert_eq!(NY_RENT_STABILIZED_PRESUMPTION_WINDOW_DAYS, 1_095);
    }

    #[test]
    fn wa_presumption_window_constant_pins_90_days() {
        assert_eq!(WA_PRESUMPTION_WINDOW_DAYS, 90);
    }

    #[test]
    fn tx_presumption_window_constant_pins_180_days() {
        assert_eq!(TX_PRESUMPTION_WINDOW_DAYS, 180);
    }

    #[test]
    fn tx_statutory_civil_penalty_constant_pins_500_dollars() {
        assert_eq!(TX_STATUTORY_CIVIL_PENALTY_CENTS, 50_000);
    }

    #[test]
    fn il_chicago_rlto_presumption_window_constant_pins_365_days() {
        assert_eq!(IL_CHICAGO_RLTO_PRESUMPTION_WINDOW_DAYS, 365);
    }

    #[test]
    fn very_large_monthly_rent_no_overflow_in_washington_triple_calc() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Washington;
        input.monthly_rent_cents = u64::MAX / 2;
        input.tenant_actual_damages_cents = 0;
        let output = check(&input);
        // saturating_mul defense
        assert!(output.estimated_landlord_exposure_cents > 0);
    }

    #[test]
    fn zero_monthly_rent_no_panic_in_chicago_two_month_calc() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::IllinoisChicagoRlto;
        input.monthly_rent_cents = 0;
        input.days_between_protected_act_and_landlord_action = 100;
        let output = check(&input);
        // Two-month rent = 0; twice damages = $10,000 → $10,000 floor
        assert_eq!(output.estimated_landlord_exposure_cents, 10_000_00);
    }
}
