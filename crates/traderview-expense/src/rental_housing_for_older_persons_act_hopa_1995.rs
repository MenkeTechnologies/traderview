//! Housing for Older Persons Act of 1995 (HOPA) Fair
//! Housing Act 55+ / 62+ Community Exemption Compliance
//! Module — federal Fair Housing Act exemption that allows
//! qualifying age-restricted housing communities to lawfully
//! exclude families with children under 18 from familial-
//! status protection.
//!
//! Pure-compute check for landlord compliance with the
//! Housing for Older Persons Act of 1995 (Public Law 104-76),
//! signed by President Bill Clinton on **December 28, 1995**.
//! HOPA amends Title VIII of the Civil Rights Act of 1968
//! (the Fair Housing Act) to clarify and modify the
//! age-restricted-housing exemption from the FHA's
//! prohibition on familial-status discrimination. The HOPA
//! exemption is codified at **42 USC § 3607(b)** and
//! implemented at **24 CFR Part 100 Subpart E** (§§ 100.300
//! through 100.308). HOPA eliminated the prior "significant
//! facilities and services" requirement that had applied to
//! 55+ communities under the original 1988 FHA amendments
//! and substituted the current 80 %-occupancy + age-
//! verification + written-policies framework.
//!
//! Web research (verified 2026-06-03):
//! - **HOPA Enactment**: Public Law 104-76; signed by
//!   President Bill Clinton on December 28, 1995; 104th
//!   Congress (HR 660 / S. 1437) ([Wikipedia — Housing for
//!   Older Persons Act](https://en.wikipedia.org/wiki/Housing_for_Older_Persons_Act);
//!   [GovTrack — Housing for Older Persons Act of 1995](https://www.govtrack.us/congress/bills/104/hr660);
//!   [House Report 104-91 — Housing for Older Persons Act of
//!   1995](https://www.govinfo.gov/content/pkg/CRPT-104hrpt91/html/CRPT-104hrpt91.htm);
//!   [Senate Report 104-172 — Housing for Older Persons Act
//!   of 1995](https://www.congress.gov/committee-report/104th-congress/senate-report/172/1)).
//! - **Codification**: 42 USC § 3607(b) (statutory) and
//!   24 CFR Part 100 Subpart E §§ 100.300-100.308
//!   (implementing regulations).
//! - **Three HOPA Exemption Categories**:
//!   1. **62+ COMMUNITIES**: every occupant of every unit
//!      must be at least **62 YEARS OLD** (100 % occupancy
//!      compliance); the strictest age category.
//!   2. **55+ COMMUNITIES**: must satisfy ALL THREE
//!      requirements simultaneously:
//!      - **80 % OCCUPANCY**: at least **80 PERCENT of the
//!        OCCUPIED UNITS** must be occupied by at least ONE
//!        person aged **55 OR OLDER**;
//!      - **AGE VERIFICATION**: ages of residents must be
//!        VERIFIED with reliable documentation (such as a
//!        government-issued photo ID, birth certificate, or
//!        driver's license) at LEAST ONCE EVERY 2 YEARS;
//!      - **WRITTEN POLICIES AND INTENT**: community must
//!        publish and follow written policies and procedures
//!        demonstrating an INTENT to operate as housing for
//!        persons 55 OR OLDER.
//!   3. **STATE OR FEDERALLY FUNDED ELDERLY HOUSING**:
//!      housing programs specifically established by federal
//!      or state law to assist elderly persons (e.g., HUD
//!      **Section 202 Supportive Housing Program for the
//!      Elderly**; certain LIHTC age-targeted projects).
//! - **HOPA Eliminated "Significant Facilities and Services"
//!   Requirement**: prior to 1995, the FHA 55+ exemption
//!   required the community to provide significant facilities
//!   and services specifically designed to meet the physical
//!   or social needs of older persons (meal services,
//!   transportation, organized activities, accessibility
//!   features, etc.). HOPA eliminated this requirement —
//!   communities no longer need to demonstrate special
//!   amenities to qualify ([Housing Equality Center —
//!   Understanding the Housing for Older Persons Exemption](https://www.equalhousing.org/fair-housing-topics/understanding-the-housing-for-older-persons-exemption/);
//!   [Pentiuk Couvreur & Kobiljak — What are HOPA exemptions
//!   for 55+ communities?](https://www.pck-law.com/blog/2023/01/what-are-hopa-exemptions-for-55-communities/)).
//! - **Good Faith Reliance Immunity**: HOPA provides "good
//!   faith reliance" immunity from damages to a person who
//!   in good faith believes and relies on a written statement
//!   that the property qualifies for the 55-or-older
//!   exemption, even if the property is later determined
//!   ineligible.
//! - **Familial-Status Discrimination Prohibition Without
//!   Valid Exemption**: the FHA at 42 USC § 3604(b) prohibits
//!   discrimination based on familial status (the presence
//!   of children under 18 in the household); without a valid
//!   HOPA exemption, a landlord may NOT refuse to rent, sell,
//!   or evict on the basis of children's presence.
//! - **Enforcement**: HUD investigates HOPA complaints
//!   through its Office of Fair Housing and Equal Opportunity
//!   (FHEO); private right of action under 42 USC § 3613
//!   (statutory damages, actual damages, reasonable attorney
//!   fees, and injunctive relief); civil penalties under
//!   42 USC § 3614 for pattern-or-practice violations
//!   ([Housing for Older Persons Act of 1995 (HOPA) Exemption
//!   from Familial Status Prohibitions — OMB 2529-0046](https://omb.report/omb/2529-0046)).

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const HOPA_ENACTMENT_YEAR: u32 = 1995;
pub const HOPA_ENACTMENT_MONTH: u32 = 12;
pub const HOPA_ENACTMENT_DAY: u32 = 28;
pub const HOPA_55_PLUS_OCCUPANCY_PERCENTAGE_BASIS_POINTS: u64 = 8_000;
pub const HOPA_55_PLUS_AGE_THRESHOLD_YEARS: u32 = 55;
pub const HOPA_62_PLUS_AGE_THRESHOLD_YEARS: u32 = 62;
pub const HOPA_AGE_VERIFICATION_CYCLE_YEARS: u32 = 2;
pub const HOPA_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimedExemptionCategory {
    SixtyTwoPlusCommunity100PercentOccupancyAtOrAbove62,
    FiftyFivePlusCommunity80PercentOccupancyWithAgeVerificationAndWrittenIntent,
    StateOrFederallyFundedElderlyHousingProgram,
    NotClaimingHopaExemption,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FamilialStatusDiscriminationAction {
    LandlordRefusedRentSaleOrEvictedFamilyWithChildrenUnder18,
    NoFamilialStatusDiscriminationActionTaken,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GoodFaithRelianceStatus {
    GoodFaithRelianceOnWrittenStatementOfHopaExemptionFromPropertyOwner,
    NoGoodFaithRelianceClaimed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum HopaMode {
    NotApplicableNotAgeRestrictedHousingNoExemptionClaimed,
    NotApplicableStateOrFederallyFundedElderlyHousingExemptionApplies,
    CompliantSixtyTwoPlusCommunity100PercentOccupancyAtOrAbove62,
    CompliantFiftyFivePlusCommunity80PercentOccupancyVerifiedEvery2YearsWithWrittenPolicies,
    CompliantGoodFaithRelianceOnWrittenStatementFromOwner,
    ViolationSixtyTwoPlusCommunityResidentUnder62Disqualifies100PercentRequirement,
    ViolationFiftyFivePlusCommunityOccupancyBelow80Percent,
    ViolationFiftyFivePlusCommunityAgeVerificationOlderThanTwoYears,
    ViolationFiftyFivePlusCommunityNoWrittenPoliciesOrIntent,
    ViolationFamilialStatusDiscriminationWithoutValidHopaExemption,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub claimed_exemption_category: ClaimedExemptionCategory,
    pub all_occupants_at_or_above_62: bool,
    pub percentage_of_occupied_units_with_at_least_one_resident_55_plus_basis_points: u64,
    pub years_since_last_age_verification: u32,
    pub has_written_policies_and_published_procedures_showing_intent: bool,
    pub familial_status_discrimination_action: FamilialStatusDiscriminationAction,
    pub good_faith_reliance_status: GoodFaithRelianceStatus,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: HopaMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub hopa_exemption_valid: bool,
    pub fair_housing_act_familial_status_protection_applies: bool,
}

pub type RentalHousingForOlderPersonsActHopa1995Input = Input;
pub type RentalHousingForOlderPersonsActHopa1995Output = Output;
pub type RentalHousingForOlderPersonsActHopa1995Result = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Housing for Older Persons Act of 1995 (HOPA) — Public Law 104-76; signed by President Bill Clinton on December 28, 1995; 104th Congress (HR 660 / S. 1437)".to_string(),
        "Codification — 42 USC § 3607(b) (statutory) and 24 CFR Part 100 Subpart E §§ 100.300-100.308 (implementing regulations); amends Title VIII of the Civil Rights Act of 1968 (Fair Housing Act)".to_string(),
        "Three HOPA Exemption Categories — (1) 62+ COMMUNITIES: every occupant at least 62 (100 % occupancy compliance); (2) 55+ COMMUNITIES: 80 % of occupied units with at least one person 55 or older + age verification every 2 years + written policies showing intent; (3) STATE OR FEDERALLY FUNDED ELDERLY HOUSING: HUD Section 202 supportive housing for elderly and similar programs".to_string(),
        "55+ Community 80 % Occupancy Requirement — at least 80 PERCENT of the OCCUPIED UNITS must be occupied by at least ONE person aged 55 OR OLDER".to_string(),
        "55+ Community Age Verification Requirement — ages of residents must be VERIFIED with reliable documentation (government-issued photo ID, birth certificate, driver's license, etc.) at LEAST ONCE EVERY 2 YEARS".to_string(),
        "55+ Community Written Policies and Intent Requirement — community must publish and follow written policies and procedures demonstrating an INTENT to operate as housing for persons 55 OR OLDER".to_string(),
        "HOPA Elimination of 'Significant Facilities and Services' Requirement — pre-1995 FHA 55+ exemption required significant facilities and services specifically designed to meet physical or social needs of older persons (meal services, transportation, organized activities, accessibility features); HOPA eliminated this requirement so communities no longer need to demonstrate special amenities to qualify".to_string(),
        "Good Faith Reliance Immunity — HOPA provides good-faith-reliance immunity from damages to persons who in good faith believe and rely on a written statement that the property qualifies for the 55-or-older exemption, even if property later determined ineligible".to_string(),
        "Familial Status Discrimination Prohibition Without Valid Exemption — FHA at 42 USC § 3604(b) prohibits discrimination based on familial status (presence of children under 18); without valid HOPA exemption, landlord may NOT refuse to rent, sell, or evict on basis of children's presence".to_string(),
        "Enforcement — HUD Office of Fair Housing and Equal Opportunity (FHEO) investigates HOPA complaints; private right of action under 42 USC § 3613 (statutory damages + actual damages + reasonable attorney fees + injunctive relief); civil penalties under 42 USC § 3614 for pattern-or-practice violations".to_string(),
        "Wikipedia — Housing for Older Persons Act (overview and statutory history)".to_string(),
        "GovTrack — Housing for Older Persons Act of 1995 (1995; 104th Congress H.R. 660) — bill tracking".to_string(),
        "House Report 104-91 — Housing for Older Persons Act of 1995 (House committee report)".to_string(),
        "Senate Report 104-172 — Housing for Older Persons Act of 1995 (Senate committee report)".to_string(),
        "Housing Equality Center — Understanding the Housing for Older Persons Exemption (practitioner guide)".to_string(),
        "Pentiuk, Couvreur & Kobiljak — What are HOPA exemptions for 55+ communities? (practitioner guide)".to_string(),
        "OMB Report 2529-0046 — Housing for Older Persons Act of 1995 (HOPA) Exemption from Familial Status Prohibitions".to_string(),
    ];

    if input.claimed_exemption_category
        == ClaimedExemptionCategory::StateOrFederallyFundedElderlyHousingProgram
    {
        return Output {
            mode: HopaMode::NotApplicableStateOrFederallyFundedElderlyHousingExemptionApplies,
            statutory_basis: "42 USC § 3607(b)(2)(A) — state or federally funded elderly housing program exemption".to_string(),
            notes: "NOT APPLICABLE: housing is part of a state or federally funded elderly housing program (e.g., HUD Section 202 Supportive Housing for the Elderly); HOPA exemption applies as a matter of statutory category without the 80 % occupancy / verification / intent showings required of private 55+ communities.".to_string(),
            citations,
            hopa_exemption_valid: true,
            fair_housing_act_familial_status_protection_applies: false,
        };
    }

    if input.claimed_exemption_category == ClaimedExemptionCategory::NotClaimingHopaExemption {
        if input.familial_status_discrimination_action
            == FamilialStatusDiscriminationAction::LandlordRefusedRentSaleOrEvictedFamilyWithChildrenUnder18
        {
            return Output {
                mode: HopaMode::ViolationFamilialStatusDiscriminationWithoutValidHopaExemption,
                statutory_basis: "42 USC § 3604(b) (FHA familial-status protection) — discrimination prohibited without valid HOPA exemption".to_string(),
                notes: "VIOLATION: landlord took familial-status-discriminatory action (refused rent/sale or evicted family with children under 18) WITHOUT claiming or qualifying for any HOPA exemption category; FHA familial-status protection under 42 USC § 3604(b) applies; tenant entitled to actual damages + statutory damages + attorney fees + injunctive relief under 42 USC § 3613.".to_string(),
                citations,
                hopa_exemption_valid: false,
                fair_housing_act_familial_status_protection_applies: true,
            };
        }
        return Output {
            mode: HopaMode::NotApplicableNotAgeRestrictedHousingNoExemptionClaimed,
            statutory_basis: "Housing is not claiming any HOPA exemption category and has taken no familial-status-discriminatory action".to_string(),
            notes: "NOT APPLICABLE: housing is not claiming any HOPA exemption category; no familial-status-discriminatory action taken; standard FHA familial-status protection under 42 USC § 3604(b) continues to apply.".to_string(),
            citations,
            hopa_exemption_valid: false,
            fair_housing_act_familial_status_protection_applies: true,
        };
    }

    if input.good_faith_reliance_status
        == GoodFaithRelianceStatus::GoodFaithRelianceOnWrittenStatementOfHopaExemptionFromPropertyOwner
    {
        return Output {
            mode: HopaMode::CompliantGoodFaithRelianceOnWrittenStatementFromOwner,
            statutory_basis: "42 USC § 3607(b)(5)(A) — good-faith-reliance immunity from damages".to_string(),
            notes: "COMPLIANT: defendant relied in good faith on a written statement from the property owner that the property qualifies for the HOPA 55-or-older exemption; HOPA good-faith-reliance immunity from damages applies even if the property is later determined ineligible.".to_string(),
            citations,
            hopa_exemption_valid: true,
            fair_housing_act_familial_status_protection_applies: false,
        };
    }

    if input.claimed_exemption_category
        == ClaimedExemptionCategory::SixtyTwoPlusCommunity100PercentOccupancyAtOrAbove62
    {
        if !input.all_occupants_at_or_above_62 {
            return Output {
                mode: HopaMode::ViolationSixtyTwoPlusCommunityResidentUnder62Disqualifies100PercentRequirement,
                statutory_basis: "42 USC § 3607(b)(2)(B) — 62+ community requires 100 % of occupants at or above 62".to_string(),
                notes: "VIOLATION: 62+ community exemption claimed but at least one occupant is under 62; 100 % occupancy compliance with age 62 minimum is required; HOPA exemption fails; FHA familial-status protection applies to landlord's action.".to_string(),
                citations,
                hopa_exemption_valid: false,
                fair_housing_act_familial_status_protection_applies: true,
            };
        }
        return Output {
            mode: HopaMode::CompliantSixtyTwoPlusCommunity100PercentOccupancyAtOrAbove62,
            statutory_basis: "42 USC § 3607(b)(2)(B) — 62+ community exemption with 100 % occupancy at or above 62".to_string(),
            notes: "COMPLIANT: 62+ community exemption satisfied; every occupant of every unit is at least 62 years old (100 % occupancy compliance); HOPA exemption valid; FHA familial-status protection does not apply to landlord's policies.".to_string(),
            citations,
            hopa_exemption_valid: true,
            fair_housing_act_familial_status_protection_applies: false,
        };
    }

    // 55+ community pathway
    if input.percentage_of_occupied_units_with_at_least_one_resident_55_plus_basis_points
        < HOPA_55_PLUS_OCCUPANCY_PERCENTAGE_BASIS_POINTS
    {
        return Output {
            mode: HopaMode::ViolationFiftyFivePlusCommunityOccupancyBelow80Percent,
            statutory_basis: "42 USC § 3607(b)(2)(C)(i) — 55+ community requires at least 80 % of occupied units with at least one person 55 or older".to_string(),
            notes: format!(
                "VIOLATION: 55+ community exemption claimed but only {} basis points ({}.{:02}%) of occupied units have at least one resident 55 or older; statutory threshold is 80 % (8,000 basis points); HOPA exemption fails; FHA familial-status protection applies.",
                input.percentage_of_occupied_units_with_at_least_one_resident_55_plus_basis_points,
                input.percentage_of_occupied_units_with_at_least_one_resident_55_plus_basis_points / 100,
                input.percentage_of_occupied_units_with_at_least_one_resident_55_plus_basis_points % 100
            ),
            citations,
            hopa_exemption_valid: false,
            fair_housing_act_familial_status_protection_applies: true,
        };
    }

    if input.years_since_last_age_verification > HOPA_AGE_VERIFICATION_CYCLE_YEARS {
        return Output {
            mode: HopaMode::ViolationFiftyFivePlusCommunityAgeVerificationOlderThanTwoYears,
            statutory_basis: "42 USC § 3607(b)(2)(C)(iii) + 24 CFR § 100.307 — 55+ community age verification at least once every 2 years".to_string(),
            notes: format!(
                "VIOLATION: 55+ community exemption claimed but last age verification was {} years ago (> 2-year statutory cycle); HOPA requires age verification at least once every 2 years via reliable documentation (government-issued photo ID, birth certificate, driver's license, etc.); HOPA exemption fails; FHA familial-status protection applies.",
                input.years_since_last_age_verification
            ),
            citations,
            hopa_exemption_valid: false,
            fair_housing_act_familial_status_protection_applies: true,
        };
    }

    if !input.has_written_policies_and_published_procedures_showing_intent {
        return Output {
            mode: HopaMode::ViolationFiftyFivePlusCommunityNoWrittenPoliciesOrIntent,
            statutory_basis: "42 USC § 3607(b)(2)(C)(ii) + 24 CFR § 100.306 — 55+ community must publish and follow written policies and procedures showing intent".to_string(),
            notes: "VIOLATION: 55+ community exemption claimed but the community does not publish and follow written policies and procedures demonstrating an intent to operate as housing for persons 55 or older; HOPA intent requirement fails; HOPA exemption fails; FHA familial-status protection applies.".to_string(),
            citations,
            hopa_exemption_valid: false,
            fair_housing_act_familial_status_protection_applies: true,
        };
    }

    Output {
        mode: HopaMode::CompliantFiftyFivePlusCommunity80PercentOccupancyVerifiedEvery2YearsWithWrittenPolicies,
        statutory_basis: "42 USC § 3607(b)(2)(C) + 24 CFR §§ 100.305-100.308 — 55+ community exemption with all three requirements satisfied".to_string(),
        notes: format!(
            "COMPLIANT: 55+ community HOPA exemption fully satisfied — (1) {} basis points ({}.{:02}%) of occupied units have at least one resident 55 or older (≥ 80 % threshold); (2) age verification performed {} years ago (≤ 2-year cycle); (3) community publishes and follows written policies and procedures showing intent to operate as housing for persons 55 or older; HOPA exemption valid; FHA familial-status protection does not apply.",
            input.percentage_of_occupied_units_with_at_least_one_resident_55_plus_basis_points,
            input.percentage_of_occupied_units_with_at_least_one_resident_55_plus_basis_points / 100,
            input.percentage_of_occupied_units_with_at_least_one_resident_55_plus_basis_points % 100,
            input.years_since_last_age_verification
        ),
        citations,
        hopa_exemption_valid: true,
        fair_housing_act_familial_status_protection_applies: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_55_plus_input() -> Input {
        Input {
            claimed_exemption_category:
                ClaimedExemptionCategory::FiftyFivePlusCommunity80PercentOccupancyWithAgeVerificationAndWrittenIntent,
            all_occupants_at_or_above_62: false,
            percentage_of_occupied_units_with_at_least_one_resident_55_plus_basis_points: 8_500,
            years_since_last_age_verification: 1,
            has_written_policies_and_published_procedures_showing_intent: true,
            familial_status_discrimination_action:
                FamilialStatusDiscriminationAction::NoFamilialStatusDiscriminationActionTaken,
            good_faith_reliance_status: GoodFaithRelianceStatus::NoGoodFaithRelianceClaimed,
        }
    }

    fn baseline_62_plus_input() -> Input {
        Input {
            claimed_exemption_category:
                ClaimedExemptionCategory::SixtyTwoPlusCommunity100PercentOccupancyAtOrAbove62,
            all_occupants_at_or_above_62: true,
            percentage_of_occupied_units_with_at_least_one_resident_55_plus_basis_points: 0,
            years_since_last_age_verification: 0,
            has_written_policies_and_published_procedures_showing_intent: false,
            familial_status_discrimination_action:
                FamilialStatusDiscriminationAction::NoFamilialStatusDiscriminationActionTaken,
            good_faith_reliance_status: GoodFaithRelianceStatus::NoGoodFaithRelianceClaimed,
        }
    }

    #[test]
    fn state_or_federally_funded_elderly_housing_exempt() {
        let mut input = baseline_55_plus_input();
        input.claimed_exemption_category =
            ClaimedExemptionCategory::StateOrFederallyFundedElderlyHousingProgram;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HopaMode::NotApplicableStateOrFederallyFundedElderlyHousingExemptionApplies
        );
        assert!(output.hopa_exemption_valid);
        assert!(!output.fair_housing_act_familial_status_protection_applies);
    }

    #[test]
    fn not_claiming_hopa_no_discrimination_action_not_applicable() {
        let mut input = baseline_55_plus_input();
        input.claimed_exemption_category = ClaimedExemptionCategory::NotClaimingHopaExemption;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HopaMode::NotApplicableNotAgeRestrictedHousingNoExemptionClaimed
        );
        assert!(!output.hopa_exemption_valid);
        assert!(output.fair_housing_act_familial_status_protection_applies);
    }

    #[test]
    fn not_claiming_hopa_with_discrimination_action_violation() {
        let mut input = baseline_55_plus_input();
        input.claimed_exemption_category = ClaimedExemptionCategory::NotClaimingHopaExemption;
        input.familial_status_discrimination_action =
            FamilialStatusDiscriminationAction::LandlordRefusedRentSaleOrEvictedFamilyWithChildrenUnder18;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HopaMode::ViolationFamilialStatusDiscriminationWithoutValidHopaExemption
        );
        assert!(!output.hopa_exemption_valid);
        assert!(output.fair_housing_act_familial_status_protection_applies);
    }

    #[test]
    fn good_faith_reliance_on_written_statement_immunity() {
        let mut input = baseline_55_plus_input();
        input.good_faith_reliance_status =
            GoodFaithRelianceStatus::GoodFaithRelianceOnWrittenStatementOfHopaExemptionFromPropertyOwner;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HopaMode::CompliantGoodFaithRelianceOnWrittenStatementFromOwner
        );
        assert!(output.hopa_exemption_valid);
    }

    #[test]
    fn sixty_two_plus_community_all_at_or_above_62_compliant() {
        let output = check(&baseline_62_plus_input());
        assert_eq!(
            output.mode,
            HopaMode::CompliantSixtyTwoPlusCommunity100PercentOccupancyAtOrAbove62
        );
        assert!(output.hopa_exemption_valid);
        assert!(!output.fair_housing_act_familial_status_protection_applies);
    }

    #[test]
    fn sixty_two_plus_community_one_resident_under_62_violation() {
        let mut input = baseline_62_plus_input();
        input.all_occupants_at_or_above_62 = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HopaMode::ViolationSixtyTwoPlusCommunityResidentUnder62Disqualifies100PercentRequirement
        );
        assert!(!output.hopa_exemption_valid);
        assert!(output.fair_housing_act_familial_status_protection_applies);
    }

    #[test]
    fn fifty_five_plus_community_all_three_requirements_satisfied_compliant() {
        let output = check(&baseline_55_plus_input());
        assert_eq!(
            output.mode,
            HopaMode::CompliantFiftyFivePlusCommunity80PercentOccupancyVerifiedEvery2YearsWithWrittenPolicies
        );
        assert!(output.hopa_exemption_valid);
    }

    #[test]
    fn fifty_five_plus_community_at_exactly_80_percent_boundary_compliant() {
        let mut input = baseline_55_plus_input();
        input.percentage_of_occupied_units_with_at_least_one_resident_55_plus_basis_points = 8_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HopaMode::CompliantFiftyFivePlusCommunity80PercentOccupancyVerifiedEvery2YearsWithWrittenPolicies
        );
    }

    #[test]
    fn fifty_five_plus_community_below_80_percent_violation() {
        let mut input = baseline_55_plus_input();
        input.percentage_of_occupied_units_with_at_least_one_resident_55_plus_basis_points = 7_999;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HopaMode::ViolationFiftyFivePlusCommunityOccupancyBelow80Percent
        );
    }

    #[test]
    fn fifty_five_plus_community_age_verification_at_exactly_2_years_compliant() {
        let mut input = baseline_55_plus_input();
        input.years_since_last_age_verification = 2;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HopaMode::CompliantFiftyFivePlusCommunity80PercentOccupancyVerifiedEvery2YearsWithWrittenPolicies
        );
    }

    #[test]
    fn fifty_five_plus_community_age_verification_at_3_years_violation() {
        let mut input = baseline_55_plus_input();
        input.years_since_last_age_verification = 3;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HopaMode::ViolationFiftyFivePlusCommunityAgeVerificationOlderThanTwoYears
        );
    }

    #[test]
    fn fifty_five_plus_community_no_written_policies_violation() {
        let mut input = baseline_55_plus_input();
        input.has_written_policies_and_published_procedures_showing_intent = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            HopaMode::ViolationFiftyFivePlusCommunityNoWrittenPoliciesOrIntent
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(HOPA_ENACTMENT_YEAR, 1995);
        assert_eq!(HOPA_ENACTMENT_MONTH, 12);
        assert_eq!(HOPA_ENACTMENT_DAY, 28);
        assert_eq!(HOPA_55_PLUS_OCCUPANCY_PERCENTAGE_BASIS_POINTS, 8_000);
        assert_eq!(HOPA_55_PLUS_AGE_THRESHOLD_YEARS, 55);
        assert_eq!(HOPA_62_PLUS_AGE_THRESHOLD_YEARS, 62);
        assert_eq!(HOPA_AGE_VERIFICATION_CYCLE_YEARS, 2);
        assert_eq!(HOPA_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citation_contains_landmarks() {
        let output = check(&baseline_55_plus_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("Housing for Older Persons Act of 1995"));
        assert!(joined.contains("Public Law 104-76"));
        assert!(joined.contains("December 28, 1995"));
        assert!(joined.contains("42 USC § 3607(b)"));
        assert!(joined.contains("24 CFR Part 100 Subpart E"));
        assert!(joined.contains("80 PERCENT"));
        assert!(joined.contains("at least 62"));
        assert!(joined.contains("55 OR OLDER"));
        assert!(joined.contains("2 YEARS"));
        assert!(joined.contains("HUD Section 202"));
    }
}
