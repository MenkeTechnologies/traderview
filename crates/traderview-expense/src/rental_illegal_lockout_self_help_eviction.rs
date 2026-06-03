//! Illegal-lockout / self-help-eviction landlord exposure framework.
//!
//! Self-help eviction (changing locks, shutting off utilities, removing doors/windows,
//! dumping belongings, installing bootlocks) is prohibited in every US jurisdiction with
//! a residential tenancy code. Landlord must use judicial process; the penalty regime
//! varies sharply by state.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - California: Civ. Code § 789.3 — $100/day minimum, $250 floor, actual damages,
//!   attorney fees (leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?
//!   lawCode=CIV&sectionNum=789.3.)
//! - New York: RPAPL § 853 + RPL § 768 — treble damages on property lost + cost of
//!   alternative accommodation + value of permanently lost tenancy (nysenate.gov/
//!   legislation/laws/RPA/768; ag.ny.gov/publications/residential-tenants-rights-guide)
//! - Texas: Prop. Code § 92.0081 — one month rent + $1,000 + court costs + actual
//!   damages + attorney fees; three lockout exceptions (bona-fide repairs/emergency,
//!   abandoned-contents removal, rent-delinquent door lock change with specific
//!   procedural compliance) (codes.findlaw.com/tx/property-code/prop-sect-92-0081/)
//! - Washington: RCW 59.18.290 — greater of actual damages or 3× monthly rent + court
//!   costs + attorney fees (app.leg.wa.gov/rcw/default.aspx?cite=59.18.290)
//! - Florida: Fla. Stat. § 83.67 — greater of actual+consequential damages or
//!   3 months' rent + costs + attorney fees; expressly prohibits bootlocks
//!   (leg.state.fl.us/statutes/index.cfm?App_mode=Display_Statute&URL=0000-0099%2F
//!   0083%2FSections%2F0083.67.html)
//! - Illinois: 735 ILCS 5/9-101 et seq. — Forcible Entry & Detainer Act requires
//!   court process; no codified per-day lockout penalty, exposure via common-law
//!   wrongful eviction + ILCS 5/9-101 court costs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    NewYork,
    Texas,
    Washington,
    Florida,
    Illinois,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LockoutMethod {
    ChangedLocksWithoutCourtOrder,
    ShutOffUtilitiesWaterGasElectric,
    RemovedDoorsOrWindows,
    DumpedTenantBelongings,
    InstalledBootlockOrSimilarDevice,
    BlockedAccessByOtherMeans,
    NoLockoutCourtProcessUsed,
}

/// Texas-specific safe-harbor branch for the three statutory exceptions to
/// the self-help-lockout prohibition under Tex. Prop. Code § 92.0081(b).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TexasSafeHarbor {
    NotInvoked,
    BonaFideRepairsOrEmergency,
    AbandonedContentsRemoval,
    /// Rent-delinquent door lock change permitted but ONLY with strict procedural
    /// compliance: written notice on door, key available 24/7, no fee for new key,
    /// notice contains specific statutory disclosures per § 92.0081(c)-(f).
    RentDelinquentLockChangeStrictProceduralCompliance,
    RentDelinquentLockChangeProceduralComplianceFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NoLockoutLawfulCourtProcess,
    TexasSafeHarborSatisfiedNoViolation,
    TexasRentDelinquentLockoutProceduralFailureViolation,
    CaliforniaCivCode789_3PerDayPenaltyViolation,
    NewYorkRpapl853TrebleDamagesViolation,
    TexasPropCode92_0081MinThousandPlusMonthRentViolation,
    WashingtonRcw59_18_290TripleMonthlyRentViolation,
    FloridaStat83_67ThreeMonthsRentViolation,
    IllinoisForcibleEntryDetainerCommonLawWrongfulEvictionViolation,
    DefaultJurisdictionStateLawSelfHelpEvictionViolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub lockout_method: LockoutMethod,
    pub days_lockout_continued: u32,
    pub monthly_rent_cents: u64,
    pub tenant_property_damage_cents: u64,
    pub alternative_accommodation_cost_cents: u64,
    pub texas_safe_harbor: TexasSafeHarbor,
}

pub type RentalIllegalLockoutSelfHelpEvictionInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub estimated_landlord_exposure_cents: u64,
    pub note: String,
}

pub type RentalIllegalLockoutSelfHelpEvictionOutput = Output;
pub type RentalIllegalLockoutSelfHelpEvictionResult = Output;

const CA_PER_DAY_PENALTY_CENTS: u64 = 10_000;
const CA_STATUTORY_MINIMUM_CENTS: u64 = 25_000;
const TX_STATUTORY_PENALTY_CENTS: u64 = 100_000;
const NY_TREBLE_MULTIPLIER: u64 = 3;
const WA_TREBLE_RENT_MULTIPLIER: u64 = 3;
const FL_THREE_MONTHS_RENT_MULTIPLIER: u64 = 3;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(input.lockout_method, LockoutMethod::NoLockoutCourtProcessUsed) {
        return Output {
            severity: Severity::NoLockoutLawfulCourtProcess,
            estimated_landlord_exposure_cents: 0,
            note: "No self-help lockout reported — landlord used judicial process. All states \
                   permit eviction only by court order; lawful path eliminates self-help \
                   exposure. Confirm writ of possession executed by sheriff/marshal."
                .to_string(),
        };
    }

    if matches!(input.jurisdiction, Jurisdiction::Texas) {
        match input.texas_safe_harbor {
            TexasSafeHarbor::BonaFideRepairsOrEmergency
            | TexasSafeHarbor::AbandonedContentsRemoval
            | TexasSafeHarbor::RentDelinquentLockChangeStrictProceduralCompliance => {
                return Output {
                    severity: Severity::TexasSafeHarborSatisfiedNoViolation,
                    estimated_landlord_exposure_cents: 0,
                    note: "Texas safe harbor under Tex. Prop. Code § 92.0081(b) satisfied. \
                           Statutory exceptions: bona-fide repairs/construction/emergency, \
                           abandoned-contents removal, OR rent-delinquent door lock change \
                           with strict procedural compliance (24/7 key availability, no \
                           replacement-key fee, written notice on door with statutory \
                           disclosures per § 92.0081(c)-(f)). No statutory penalty. Confirm \
                           every procedural element documented; partial compliance is treated \
                           as violation."
                        .to_string(),
                };
            }
            TexasSafeHarbor::RentDelinquentLockChangeProceduralComplianceFailed => {
                let exposure = TX_STATUTORY_PENALTY_CENTS
                    .saturating_add(input.monthly_rent_cents)
                    .saturating_add(input.tenant_property_damage_cents);
                return Output {
                    severity: Severity::TexasRentDelinquentLockoutProceduralFailureViolation,
                    estimated_landlord_exposure_cents: exposure,
                    note: format!(
                        "Texas Prop. Code § 92.0081 rent-delinquent lock-change safe harbor \
                         FAILED. Landlord changed locks for non-payment but missed at least one \
                         procedural element (24/7 key availability, no replacement-key fee, \
                         written notice with statutory disclosures). Statutory penalty: \
                         one month rent (${}) + $1,000 + actual damages (${}) + court costs + \
                         attorney fees. Estimated exposure ${} excludes attorney fees.",
                        input.monthly_rent_cents / 100,
                        input.tenant_property_damage_cents / 100,
                        exposure / 100
                    ),
                };
            }
            TexasSafeHarbor::NotInvoked => {
                let exposure = TX_STATUTORY_PENALTY_CENTS
                    .saturating_add(input.monthly_rent_cents)
                    .saturating_add(input.tenant_property_damage_cents);
                return Output {
                    severity: Severity::TexasPropCode92_0081MinThousandPlusMonthRentViolation,
                    estimated_landlord_exposure_cents: exposure,
                    note: format!(
                        "Texas Prop. Code § 92.0081 violation. Self-help lockout without \
                         invoking the bona-fide-repairs / abandoned-contents / rent-delinquent \
                         safe harbors. Statutory exposure: one month rent (${}) + $1,000 + \
                         actual damages (${}) + court costs + attorney fees. Tenant may obtain \
                         writ of re-entry through justice court. Estimated exposure ${} \
                         excludes attorney fees.",
                        input.monthly_rent_cents / 100,
                        input.tenant_property_damage_cents / 100,
                        exposure / 100
                    ),
                };
            }
        }
    }

    match input.jurisdiction {
        Jurisdiction::California => {
            let per_day_total =
                CA_PER_DAY_PENALTY_CENTS.saturating_mul(u64::from(input.days_lockout_continued));
            let penalty = per_day_total.max(CA_STATUTORY_MINIMUM_CENTS);
            let exposure = penalty.saturating_add(input.tenant_property_damage_cents);
            Output {
                severity: Severity::CaliforniaCivCode789_3PerDayPenaltyViolation,
                estimated_landlord_exposure_cents: exposure,
                note: format!(
                    "California Civ. Code § 789.3 violation. Prohibits locks change, utility \
                     shutoff, door/window removal, belongings dump to force tenant exit. \
                     Statutory penalty: $100/day continued (${}) with $250 minimum floor \
                     applied = ${}, plus actual damages (${}) + attorney fees + potential \
                     punitive damages. Estimated exposure ${} excludes attorney fees and \
                     punitive damages. CA AG Bulletin 2022-DLE-05 directs law enforcement \
                     to treat lockout as criminal trespass / unlawful detainer interference.",
                    per_day_total / 100,
                    penalty / 100,
                    input.tenant_property_damage_cents / 100,
                    exposure / 100
                ),
            }
        }
        Jurisdiction::NewYork => {
            let treble_property =
                input.tenant_property_damage_cents.saturating_mul(NY_TREBLE_MULTIPLIER);
            let exposure =
                treble_property.saturating_add(input.alternative_accommodation_cost_cents);
            Output {
                severity: Severity::NewYorkRpapl853TrebleDamagesViolation,
                estimated_landlord_exposure_cents: exposure,
                note: format!(
                    "New York RPAPL § 853 + RPL § 768 violation. Forcible eviction or unlawful \
                     means by landlord triggers TREBLE DAMAGES on property lost or damaged \
                     (${} × 3 = ${}) PLUS cost of alternative accommodation (${}) PLUS value \
                     of permanently lost tenancy (rent-stabilized / rent-controlled units often \
                     carry six-figure tenancy value). RPL § 768 makes unlawful eviction a \
                     Class A misdemeanor — criminal exposure in addition to civil. Estimated \
                     exposure ${} excludes tenancy-value award and criminal penalty.",
                    input.tenant_property_damage_cents / 100,
                    treble_property / 100,
                    input.alternative_accommodation_cost_cents / 100,
                    exposure / 100
                ),
            }
        }
        Jurisdiction::Washington => {
            let treble_rent =
                input.monthly_rent_cents.saturating_mul(WA_TREBLE_RENT_MULTIPLIER);
            let exposure = treble_rent.max(input.tenant_property_damage_cents);
            Output {
                severity: Severity::WashingtonRcw59_18_290TripleMonthlyRentViolation,
                estimated_landlord_exposure_cents: exposure,
                note: format!(
                    "Washington RCW 59.18.290 violation. Residential Landlord-Tenant Act \
                     prohibits exclusion of tenant except by court order. Tenant recovers \
                     GREATER OF actual damages (${}) OR 3× monthly rent (${} × 3 = ${}) PLUS \
                     court costs and reasonable attorney fees. Tenant may also terminate the \
                     rental agreement. Estimated exposure ${} excludes attorney fees.",
                    input.tenant_property_damage_cents / 100,
                    input.monthly_rent_cents / 100,
                    treble_rent / 100,
                    exposure / 100
                ),
            }
        }
        Jurisdiction::Florida => {
            let three_months_rent =
                input.monthly_rent_cents.saturating_mul(FL_THREE_MONTHS_RENT_MULTIPLIER);
            let exposure = three_months_rent.max(input.tenant_property_damage_cents);
            Output {
                severity: Severity::FloridaStat83_67ThreeMonthsRentViolation,
                estimated_landlord_exposure_cents: exposure,
                note: format!(
                    "Florida Stat. § 83.67 violation. Statute expressly prohibits changing \
                     locks, using a bootlock or similar device, shutting off utilities, or \
                     otherwise blocking reasonable access. Tenant recovers GREATER OF actual \
                     and consequential damages (${}) OR 3 months' rent (${} × 3 = ${}) PLUS \
                     costs and attorney fees. Estimated exposure ${} excludes attorney fees.",
                    input.tenant_property_damage_cents / 100,
                    input.monthly_rent_cents / 100,
                    three_months_rent / 100,
                    exposure / 100
                ),
            }
        }
        Jurisdiction::Illinois => Output {
            severity:
                Severity::IllinoisForcibleEntryDetainerCommonLawWrongfulEvictionViolation,
            estimated_landlord_exposure_cents: input.tenant_property_damage_cents,
            note: format!(
                "Illinois Forcible Entry and Detainer Act (735 ILCS 5/9-101 et seq.) requires \
                 court process; self-help lockout is unlawful and exposes landlord to common-law \
                 wrongful-eviction damages, tenant property damage (${}), and court costs. No \
                 codified per-day lockout penalty in Illinois state law. Cook County / Chicago \
                 Residential Landlord and Tenant Ordinance (RLTO) § 5-12-160 imposes additional \
                 penalty: two months' rent or twice damages, whichever is greater, plus attorney \
                 fees for Chicago-located rentals. Estimated exposure ${} excludes attorney \
                 fees and Chicago RLTO penalty if applicable.",
                input.tenant_property_damage_cents / 100,
                input.tenant_property_damage_cents / 100
            ),
        },
        Jurisdiction::Default => Output {
            severity: Severity::DefaultJurisdictionStateLawSelfHelpEvictionViolation,
            estimated_landlord_exposure_cents: input.tenant_property_damage_cents,
            note: format!(
                "Self-help eviction is unlawful in every US state with a residential tenancy \
                 code. Specific penalty regime depends on state statute (CA $100/day + $250 \
                 floor, NY treble damages, TX $1K + month rent, WA 3× monthly rent, FL 3 \
                 months' rent, IL common-law wrongful eviction). Confirm state-law exposure \
                 with local counsel. Baseline exposure includes tenant property damage (${}) \
                 + statutory penalty + attorney fees.",
                input.tenant_property_damage_cents / 100
            ),
        },
        Jurisdiction::Texas => unreachable!("Texas branch handled above"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            lockout_method: LockoutMethod::ChangedLocksWithoutCourtOrder,
            days_lockout_continued: 30,
            monthly_rent_cents: 3_000_00,
            tenant_property_damage_cents: 2_000_00,
            alternative_accommodation_cost_cents: 5_000_00,
            texas_safe_harbor: TexasSafeHarbor::NotInvoked,
        }
    }

    #[test]
    fn no_lockout_court_process_used_returns_not_applicable() {
        let mut input = base();
        input.lockout_method = LockoutMethod::NoLockoutCourtProcessUsed;
        let output = check(&input);
        assert_eq!(output.severity, Severity::NoLockoutLawfulCourtProcess);
        assert_eq!(output.estimated_landlord_exposure_cents, 0);
        assert!(output.note.contains("judicial process"));
    }

    #[test]
    fn california_30_day_lockout_calculates_3000_penalty_plus_damages() {
        let input = base();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CaliforniaCivCode789_3PerDayPenaltyViolation
        );
        // 30 days × $100/day = $3,000 + $2,000 property damage = $5,000
        assert_eq!(output.estimated_landlord_exposure_cents, 5_000_00);
        assert!(output.note.contains("Civ. Code § 789.3"));
        assert!(output.note.contains("$100/day"));
    }

    #[test]
    fn california_1_day_lockout_applies_250_statutory_minimum_floor() {
        let mut input = base();
        input.days_lockout_continued = 1;
        input.tenant_property_damage_cents = 0;
        let output = check(&input);
        // 1 day × $100 = $100, but $250 floor applies
        assert_eq!(output.estimated_landlord_exposure_cents, 250_00);
        assert!(output.note.contains("$250 minimum floor"));
    }

    #[test]
    fn new_york_lockout_triggers_treble_damages_plus_alternative_accommodation() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::NewYork;
        let output = check(&input);
        assert_eq!(output.severity, Severity::NewYorkRpapl853TrebleDamagesViolation);
        // $2,000 property × 3 = $6,000 + $5,000 alternative accommodation = $11,000
        assert_eq!(output.estimated_landlord_exposure_cents, 11_000_00);
        assert!(output.note.contains("RPAPL § 853"));
        assert!(output.note.contains("RPL § 768"));
        assert!(output.note.contains("TREBLE"));
    }

    #[test]
    fn texas_not_invoked_safe_harbor_triggers_full_statutory_penalty() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Texas;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::TexasPropCode92_0081MinThousandPlusMonthRentViolation
        );
        // $1,000 + $3,000 monthly rent + $2,000 property = $6,000
        assert_eq!(output.estimated_landlord_exposure_cents, 6_000_00);
        assert!(output.note.contains("§ 92.0081"));
        assert!(output.note.contains("writ of re-entry"));
    }

    #[test]
    fn texas_bona_fide_repairs_safe_harbor_satisfied_no_violation() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Texas;
        input.texas_safe_harbor = TexasSafeHarbor::BonaFideRepairsOrEmergency;
        let output = check(&input);
        assert_eq!(output.severity, Severity::TexasSafeHarborSatisfiedNoViolation);
        assert_eq!(output.estimated_landlord_exposure_cents, 0);
        assert!(output.note.contains("bona-fide repairs"));
    }

    #[test]
    fn texas_rent_delinquent_procedural_compliance_satisfied_no_violation() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Texas;
        input.texas_safe_harbor =
            TexasSafeHarbor::RentDelinquentLockChangeStrictProceduralCompliance;
        let output = check(&input);
        assert_eq!(output.severity, Severity::TexasSafeHarborSatisfiedNoViolation);
        assert_eq!(output.estimated_landlord_exposure_cents, 0);
    }

    #[test]
    fn texas_rent_delinquent_procedural_failure_triggers_full_penalty() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Texas;
        input.texas_safe_harbor =
            TexasSafeHarbor::RentDelinquentLockChangeProceduralComplianceFailed;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::TexasRentDelinquentLockoutProceduralFailureViolation
        );
        // Same exposure formula: $1K + $3K + $2K = $6K
        assert_eq!(output.estimated_landlord_exposure_cents, 6_000_00);
        assert!(output.note.contains("procedural element"));
    }

    #[test]
    fn washington_lockout_triple_monthly_rent_or_actual_damages_greater() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Washington;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::WashingtonRcw59_18_290TripleMonthlyRentViolation
        );
        // 3 × $3,000 = $9,000 > $2,000 actual → $9,000
        assert_eq!(output.estimated_landlord_exposure_cents, 9_000_00);
        assert!(output.note.contains("RCW 59.18.290"));
    }

    #[test]
    fn washington_high_actual_damages_exceed_triple_rent_use_actual() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Washington;
        input.tenant_property_damage_cents = 20_000_00;
        let output = check(&input);
        // $20,000 actual > $9,000 triple rent → $20,000
        assert_eq!(output.estimated_landlord_exposure_cents, 20_000_00);
    }

    #[test]
    fn florida_lockout_three_months_rent_default() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Florida;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::FloridaStat83_67ThreeMonthsRentViolation
        );
        // 3 × $3,000 = $9,000 > $2,000 actual → $9,000
        assert_eq!(output.estimated_landlord_exposure_cents, 9_000_00);
        assert!(output.note.contains("§ 83.67"));
        assert!(output.note.contains("bootlock"));
    }

    #[test]
    fn illinois_lockout_common_law_wrongful_eviction_baseline() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Illinois;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::IllinoisForcibleEntryDetainerCommonLawWrongfulEvictionViolation
        );
        // Illinois has no codified per-day penalty; baseline is property damage
        assert_eq!(output.estimated_landlord_exposure_cents, 2_000_00);
        assert!(output.note.contains("735 ILCS 5/9-101"));
        assert!(output.note.contains("Chicago"));
    }

    #[test]
    fn default_jurisdiction_returns_state_law_violation_note() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Default;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::DefaultJurisdictionStateLawSelfHelpEvictionViolation
        );
        assert!(output.note.contains("every US state"));
    }

    #[test]
    fn utility_shutoff_method_triggers_violation_same_as_lock_change() {
        let mut input = base();
        input.lockout_method = LockoutMethod::ShutOffUtilitiesWaterGasElectric;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CaliforniaCivCode789_3PerDayPenaltyViolation
        );
    }

    #[test]
    fn dumped_belongings_method_triggers_violation() {
        let mut input = base();
        input.lockout_method = LockoutMethod::DumpedTenantBelongings;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CaliforniaCivCode789_3PerDayPenaltyViolation
        );
    }

    #[test]
    fn bootlock_method_explicitly_prohibited_in_florida() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Florida;
        input.lockout_method = LockoutMethod::InstalledBootlockOrSimilarDevice;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::FloridaStat83_67ThreeMonthsRentViolation
        );
        assert!(output.note.contains("bootlock"));
    }

    #[test]
    fn ca_per_day_penalty_constant_pins_100_dollars() {
        assert_eq!(CA_PER_DAY_PENALTY_CENTS, 10_000);
    }

    #[test]
    fn ca_statutory_minimum_floor_constant_pins_250_dollars() {
        assert_eq!(CA_STATUTORY_MINIMUM_CENTS, 25_000);
    }

    #[test]
    fn tx_statutory_penalty_constant_pins_1000_dollars() {
        assert_eq!(TX_STATUTORY_PENALTY_CENTS, 100_000);
    }

    #[test]
    fn ny_treble_multiplier_pins_three() {
        assert_eq!(NY_TREBLE_MULTIPLIER, 3);
    }

    #[test]
    fn wa_treble_rent_multiplier_pins_three() {
        assert_eq!(WA_TREBLE_RENT_MULTIPLIER, 3);
    }

    #[test]
    fn fl_three_months_rent_multiplier_pins_three() {
        assert_eq!(FL_THREE_MONTHS_RENT_MULTIPLIER, 3);
    }

    #[test]
    fn very_large_monthly_rent_no_overflow_in_washington_triple() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Washington;
        input.monthly_rent_cents = u64::MAX / 2;
        input.tenant_property_damage_cents = 0;
        let output = check(&input);
        // saturating_mul prevents overflow
        assert!(output.estimated_landlord_exposure_cents > 0);
    }

    #[test]
    fn extreme_days_count_no_overflow_in_california_per_day() {
        let mut input = base();
        input.days_lockout_continued = u32::MAX;
        input.tenant_property_damage_cents = 0;
        let output = check(&input);
        assert!(output.estimated_landlord_exposure_cents > 0);
    }

    #[test]
    fn ca_note_pins_ag_bulletin_2022_dle_05() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("CA AG Bulletin 2022-DLE-05"));
    }

    #[test]
    fn ny_note_pins_misdemeanor_criminal_exposure() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::NewYork;
        let output = check(&input);
        assert!(output.note.contains("Class A misdemeanor"));
    }

    #[test]
    fn ny_note_describes_rent_stabilized_tenancy_value() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::NewYork;
        let output = check(&input);
        assert!(output.note.contains("rent-stabilized"));
    }

    #[test]
    fn texas_note_describes_24_7_key_availability_requirement() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Texas;
        input.texas_safe_harbor =
            TexasSafeHarbor::RentDelinquentLockChangeStrictProceduralCompliance;
        let output = check(&input);
        assert!(output.note.contains("24/7 key availability"));
    }

    #[test]
    fn illinois_note_pins_chicago_rlto_5_12_160() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Illinois;
        let output = check(&input);
        assert!(output.note.contains("§ 5-12-160"));
    }

    #[test]
    fn zero_monthly_rent_no_panic_in_florida_three_months_calc() {
        let mut input = base();
        input.jurisdiction = Jurisdiction::Florida;
        input.monthly_rent_cents = 0;
        let output = check(&input);
        // Floor applied via .max() with property damage
        assert_eq!(output.estimated_landlord_exposure_cents, 2_000_00);
    }
}
