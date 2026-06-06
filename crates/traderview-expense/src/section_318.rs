//! IRC § 318 constructive ownership of stock.
//!
//! § 318 attributes stock ownership from one taxpayer to another for purposes of
//! related-party and ownership-percentage testing throughout the Code. Constructive
//! ownership is the engine that drives stock-redemption analysis (§ 302), brother-
//! sister redemption recharacterization (§ 304), accumulated-earnings tax (§ 531),
//! personal-holding-company tax (§ 542), corporate-attribution-to-shareholder rules,
//! and qualified-personal-service-corporation testing.
//!
//! § 318(a)(1) FAMILY ATTRIBUTION: an individual is treated as owning stock owned
//! by spouse (unless legally separated under decree of divorce or separate
//! maintenance), children, grandchildren, and parents. Notably, § 318 family
//! attribution does NOT extend to SIBLINGS or grandparents (compare § 267 which
//! does include siblings).
//!
//! § 318(a)(2) ATTRIBUTION FROM ENTITIES TO OWNERS:
//!   - (a)(2)(A) Partnership / estate: stock owned by partnership or estate is
//!     treated as owned PROPORTIONATELY by its partners or beneficiaries.
//!   - (a)(2)(B) Trust: stock owned by trust treated as owned proportionately by
//!     beneficiaries (actuarial interest under § 7520 tables for non-grantor trusts;
//!     fully attributable to grantor for grantor trusts).
//!   - (a)(2)(C) Corporation: stock owned by corporation treated as owned by any
//!     50%-or-more shareholder PROPORTIONATELY to their ownership.
//!
//! § 318(a)(3) ATTRIBUTION TO ENTITIES FROM OWNERS:
//!   - (a)(3)(A) Partnership / estate: stock owned by partner / beneficiary is
//!     treated as owned by the partnership / estate IN FULL.
//!   - (a)(3)(B) Trust: stock owned by beneficiary treated as owned by trust IN FULL.
//!   - (a)(3)(C) Corporation: stock owned by 50%-or-more shareholder treated as
//!     owned by the corporation IN FULL.
//!
//! § 318(a)(4) OPTION ATTRIBUTION: a person holding an OPTION (call option, warrant,
//! convertible bond) to acquire stock is treated as owning that stock for § 318
//! purposes.
//!
//! § 318(a)(5) CHAIN ATTRIBUTION RULES + EXCEPTIONS:
//!   - (a)(5)(A) Chain attribution is generally permitted (multi-step attribution).
//!   - (a)(5)(B) Stock attributed via family attribution (a)(1) cannot be RE-ATTRIBUTED
//!     to another family member via family attribution.
//!   - (a)(5)(C) Stock attributed via entity-from-entity rules (a)(2) cannot be
//!     re-attributed via (a)(3) entity-to-entity rules (no double-bounce).
//!   - (a)(5)(D) Option attribution (a)(4) takes priority over family attribution
//!     when both could apply.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - law.cornell.edu/uscode/text/26/318
//! - irc.bloombergtax.com/public/uscode/doc/irc/section_318
//! - taxnotes.com/research/federal/usc26/318
//! - irs.gov/pub/fatca/int_practice_units/irc958-stock-ownership.pdf

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttributionPath {
    /// § 318(a)(1) family attribution (spouse, children, grandchildren, parents).
    FamilyAttributionSpouseChildGrandchildParent,
    /// § 318(a)(1) sibling attribution attempt — DOES NOT APPLY under § 318
    /// (compare § 267 which does include siblings).
    SiblingAttemptDoesNotApplyToSection318,
    /// § 318(a)(2)(A) attribution FROM partnership / estate to partner / beneficiary
    /// (proportional).
    FromPartnershipOrEstateProportional,
    /// § 318(a)(2)(C) attribution FROM corporation to 50%+ shareholder
    /// (proportional).
    FromCorporationToFiftyPctShareholderProportional,
    /// § 318(a)(3)(A) attribution TO partnership / estate from partner /
    /// beneficiary (in full).
    ToPartnershipOrEstateInFull,
    /// § 318(a)(3)(C) attribution TO corporation from 50%+ shareholder (in full).
    ToCorporationFromFiftyPctShareholderInFull,
    /// § 318(a)(4) option attribution.
    OptionAttributionCallOrWarrantOrConvertible,
    /// § 318(a)(5)(B) re-attribution via family-to-family attempted — DISALLOWED.
    ReAttributionFamilyToFamilyDisallowed,
    /// § 318(a)(5)(C) re-attribution via entity-bounce attempted — DISALLOWED.
    ReAttributionEntityBounceDisallowed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CorporationOwnershipTier {
    /// 50% or more of corporation owned by shareholder — entity-attribution rules
    /// apply.
    FiftyPctOrMoreEntityAttributionApplies,
    /// Less than 50% of corporation owned — corporation-to-shareholder attribution
    /// does NOT apply.
    LessThanFiftyPctNoEntityAttribution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NoAttributionUnderSection318,
    Section318A1FamilyAttributionApplies,
    Section318A2EntityToOwnerProportionalAttribution,
    Section318A3OwnerToEntityFullAttribution,
    Section318A4OptionAttributionApplies,
    Section318A5BReAttributionDisallowedFamilyToFamily,
    Section318A5CReAttributionDisallowedEntityBounce,
    SiblingNotIncludedInSection318FamilyAttribution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub attribution_path: AttributionPath,
    pub corporation_ownership_tier: CorporationOwnershipTier,
    pub direct_ownership_shares: u64,
    pub direct_ownership_percentage_bps: u32,
    pub proportional_attribution_percentage_bps: u32,
}

pub type Section318ConstructiveOwnershipInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub attributed_shares: u64,
    pub attributed_percentage_bps: u32,
    pub note: String,
}

pub type Section318ConstructiveOwnershipOutput = Output;
pub type Section318ConstructiveOwnershipResult = Output;

const FIFTY_PCT_THRESHOLD_BPS: u32 = 5_000;

#[must_use]
pub fn check(input: &Input) -> Output {
    match input.attribution_path {
        AttributionPath::FamilyAttributionSpouseChildGrandchildParent => Output {
            severity: Severity::Section318A1FamilyAttributionApplies,
            attributed_shares: input.direct_ownership_shares,
            attributed_percentage_bps: input.direct_ownership_percentage_bps,
            note: format!(
                "§ 318(a)(1) FAMILY ATTRIBUTION applies. Stock owned by spouse (unless \
                 legally separated under decree of divorce or separate maintenance), \
                 child, grandchild, or parent attributed in FULL. § 318 family attribution \
                 does NOT extend to siblings (compare § 267 which does include siblings) \
                 or to grandparents. Attributed shares: {} ({} bps). Coordinates with § 302 \
                 redemption qualification, § 304 brother-sister recharacterization (limited \
                 by sibling-exclusion under § 318), § 542 PHC ownership test, § 856(h) REIT \
                 5/50 closely-held test.",
                input.direct_ownership_shares, input.direct_ownership_percentage_bps
            ),
        },
        AttributionPath::SiblingAttemptDoesNotApplyToSection318 => Output {
            severity: Severity::SiblingNotIncludedInSection318FamilyAttribution,
            attributed_shares: 0,
            attributed_percentage_bps: 0,
            note: "§ 318 family attribution does NOT include SIBLINGS. § 318(a)(1) covers \
                   spouse, children, grandchildren, and parents — siblings are EXPLICITLY \
                   excluded. This is a critical asymmetry vs § 267 related-party loss \
                   regime (which DOES include siblings). For § 302 redemption analysis, the \
                   sibling-exclusion under § 318 can preserve substantially-disproportionate \
                   redemption treatment that would fail under a broader family-attribution \
                   rule."
                .to_string(),
        },
        AttributionPath::FromPartnershipOrEstateProportional
        | AttributionPath::FromCorporationToFiftyPctShareholderProportional => {
            if matches!(
                input.attribution_path,
                AttributionPath::FromCorporationToFiftyPctShareholderProportional
            ) && matches!(
                input.corporation_ownership_tier,
                CorporationOwnershipTier::LessThanFiftyPctNoEntityAttribution
            ) {
                return Output {
                    severity: Severity::NoAttributionUnderSection318,
                    attributed_shares: 0,
                    attributed_percentage_bps: 0,
                    note: format!(
                        "§ 318(a)(2)(C) corporation-to-shareholder attribution does NOT \
                         apply: shareholder owns LESS THAN {}% of the corporation. Entity \
                         attribution under § 318(a)(2)(C) requires 50%-or-more ownership. \
                         No attribution.",
                        FIFTY_PCT_THRESHOLD_BPS / 100
                    ),
                };
            }
            let attributed_shares = u64::try_from(
                u128::from(input.direct_ownership_shares)
                    .saturating_mul(u128::from(input.proportional_attribution_percentage_bps))
                    .saturating_div(10_000),
            )
            .unwrap_or(u64::MAX);
            let path_label = match input.attribution_path {
                AttributionPath::FromPartnershipOrEstateProportional => {
                    "§ 318(a)(2)(A) partnership / estate-to-owner"
                }
                AttributionPath::FromCorporationToFiftyPctShareholderProportional => {
                    "§ 318(a)(2)(C) corporation-to-50%-shareholder"
                }
                _ => unreachable!(),
            };
            Output {
                severity: Severity::Section318A2EntityToOwnerProportionalAttribution,
                attributed_shares,
                attributed_percentage_bps: input.proportional_attribution_percentage_bps,
                note: format!(
                    "{} PROPORTIONAL attribution applies. Stock owned by entity ({} \
                     shares) attributed to owner proportionally to owner's interest ({} bps) \
                     = {} attributed shares. § 318(a)(2)(B) trust attribution similar but \
                     uses § 7520-actuarial interest for non-grantor trusts (full to grantor \
                     for grantor trusts).",
                    path_label,
                    input.direct_ownership_shares,
                    input.proportional_attribution_percentage_bps,
                    attributed_shares
                ),
            }
        }
        AttributionPath::ToPartnershipOrEstateInFull
        | AttributionPath::ToCorporationFromFiftyPctShareholderInFull => {
            if matches!(
                input.attribution_path,
                AttributionPath::ToCorporationFromFiftyPctShareholderInFull
            ) && matches!(
                input.corporation_ownership_tier,
                CorporationOwnershipTier::LessThanFiftyPctNoEntityAttribution
            ) {
                return Output {
                    severity: Severity::NoAttributionUnderSection318,
                    attributed_shares: 0,
                    attributed_percentage_bps: 0,
                    note: format!(
                        "§ 318(a)(3)(C) shareholder-to-corporation attribution does NOT \
                         apply: shareholder owns LESS THAN {}% of the corporation. Entity \
                         attribution under § 318(a)(3)(C) requires 50%-or-more ownership. \
                         No attribution.",
                        FIFTY_PCT_THRESHOLD_BPS / 100
                    ),
                };
            }
            let path_label = match input.attribution_path {
                AttributionPath::ToPartnershipOrEstateInFull => {
                    "§ 318(a)(3)(A) partner/beneficiary-to-partnership/estate"
                }
                AttributionPath::ToCorporationFromFiftyPctShareholderInFull => {
                    "§ 318(a)(3)(C) 50%-shareholder-to-corporation"
                }
                _ => unreachable!(),
            };
            Output {
                severity: Severity::Section318A3OwnerToEntityFullAttribution,
                attributed_shares: input.direct_ownership_shares,
                attributed_percentage_bps: input.direct_ownership_percentage_bps,
                note: format!(
                    "{} attribution IN FULL applies. Stock owned by owner attributed to \
                     the entity at 100% — different from entity-to-owner which is \
                     proportional. Attributed shares: {} ({} bps).",
                    path_label,
                    input.direct_ownership_shares,
                    input.direct_ownership_percentage_bps
                ),
            }
        }
        AttributionPath::OptionAttributionCallOrWarrantOrConvertible => Output {
            severity: Severity::Section318A4OptionAttributionApplies,
            attributed_shares: input.direct_ownership_shares,
            attributed_percentage_bps: input.direct_ownership_percentage_bps,
            note: format!(
                "§ 318(a)(4) OPTION ATTRIBUTION applies. Holder of option to acquire stock \
                 (call option, warrant, convertible debenture) treated as already OWNING \
                 the stock subject to the option. § 318(a)(5)(D) gives option attribution \
                 PRIORITY over family attribution when both could apply. Attributed shares: \
                 {} ({} bps). Critical for § 302 redemption analysis where outstanding call \
                 options held by family member can defeat substantially-disproportionate \
                 redemption qualification.",
                input.direct_ownership_shares, input.direct_ownership_percentage_bps
            ),
        },
        AttributionPath::ReAttributionFamilyToFamilyDisallowed => Output {
            severity: Severity::Section318A5BReAttributionDisallowedFamilyToFamily,
            attributed_shares: 0,
            attributed_percentage_bps: 0,
            note: "§ 318(a)(5)(B) RE-ATTRIBUTION DISALLOWED: stock attributed via family \
                   attribution under (a)(1) cannot be RE-ATTRIBUTED to another family \
                   member via family attribution. Example: Father's stock attributed to \
                   Son under (a)(1); that attributed stock CANNOT then be re-attributed \
                   from Son to Son's Spouse via another (a)(1) attribution. This rule \
                   prevents cascading family attribution that would expand § 318 reach \
                   beyond first-degree relatives in spousal/parental chains."
                .to_string(),
        },
        AttributionPath::ReAttributionEntityBounceDisallowed => Output {
            severity: Severity::Section318A5CReAttributionDisallowedEntityBounce,
            attributed_shares: 0,
            attributed_percentage_bps: 0,
            note: "§ 318(a)(5)(C) RE-ATTRIBUTION DISALLOWED: stock attributed via (a)(2) \
                   entity-to-owner cannot be re-attributed via (a)(3) owner-to-entity (no \
                   'double-bounce'). Example: stock owned by Partnership 1 attributed to \
                   Partner A under (a)(2)(A); that attributed stock CANNOT then be \
                   re-attributed from Partner A to Partnership 2 under (a)(3)(A). Prevents \
                   indirect attribution loops that would defeat the structural purpose of \
                   § 318."
                .to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Input {
        Input {
            attribution_path: AttributionPath::FamilyAttributionSpouseChildGrandchildParent,
            corporation_ownership_tier:
                CorporationOwnershipTier::FiftyPctOrMoreEntityAttributionApplies,
            direct_ownership_shares: 1_000,
            direct_ownership_percentage_bps: 1_000,
            proportional_attribution_percentage_bps: 6_000,
        }
    }

    #[test]
    fn family_attribution_full_attribution() {
        let input = base();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section318A1FamilyAttributionApplies
        );
        assert_eq!(output.attributed_shares, 1_000);
        assert!(output.note.contains("spouse"));
        assert!(output.note.contains("child"));
        assert!(output.note.contains("grandchild"));
        assert!(output.note.contains("parent"));
    }

    #[test]
    fn sibling_does_not_apply_to_section_318() {
        let mut input = base();
        input.attribution_path = AttributionPath::SiblingAttemptDoesNotApplyToSection318;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::SiblingNotIncludedInSection318FamilyAttribution
        );
        assert!(output.note.contains("§ 267"));
        assert!(output.note.contains("§ 302"));
    }

    #[test]
    fn partnership_to_partner_proportional_attribution() {
        let mut input = base();
        input.attribution_path = AttributionPath::FromPartnershipOrEstateProportional;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section318A2EntityToOwnerProportionalAttribution
        );
        // 1000 × 60% = 600
        assert_eq!(output.attributed_shares, 600);
        assert!(output.note.contains("§ 318(a)(2)(A)"));
    }

    #[test]
    fn corporation_to_50_pct_plus_shareholder_proportional() {
        let mut input = base();
        input.attribution_path = AttributionPath::FromCorporationToFiftyPctShareholderProportional;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section318A2EntityToOwnerProportionalAttribution
        );
        assert!(output.note.contains("§ 318(a)(2)(C)"));
    }

    #[test]
    fn corporation_below_50_pct_no_attribution() {
        let mut input = base();
        input.attribution_path = AttributionPath::FromCorporationToFiftyPctShareholderProportional;
        input.corporation_ownership_tier =
            CorporationOwnershipTier::LessThanFiftyPctNoEntityAttribution;
        let output = check(&input);
        assert_eq!(output.severity, Severity::NoAttributionUnderSection318);
    }

    #[test]
    fn partner_to_partnership_full_attribution() {
        let mut input = base();
        input.attribution_path = AttributionPath::ToPartnershipOrEstateInFull;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section318A3OwnerToEntityFullAttribution
        );
        assert_eq!(output.attributed_shares, 1_000); // Full attribution, not proportional
        assert!(output.note.contains("§ 318(a)(3)(A)"));
    }

    #[test]
    fn shareholder_to_corp_50_pct_plus_full_attribution() {
        let mut input = base();
        input.attribution_path = AttributionPath::ToCorporationFromFiftyPctShareholderInFull;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section318A3OwnerToEntityFullAttribution
        );
        assert!(output.note.contains("§ 318(a)(3)(C)"));
    }

    #[test]
    fn shareholder_to_corp_below_50_pct_no_attribution() {
        let mut input = base();
        input.attribution_path = AttributionPath::ToCorporationFromFiftyPctShareholderInFull;
        input.corporation_ownership_tier =
            CorporationOwnershipTier::LessThanFiftyPctNoEntityAttribution;
        let output = check(&input);
        assert_eq!(output.severity, Severity::NoAttributionUnderSection318);
    }

    #[test]
    fn option_attribution_treats_holder_as_owner() {
        let mut input = base();
        input.attribution_path = AttributionPath::OptionAttributionCallOrWarrantOrConvertible;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section318A4OptionAttributionApplies
        );
        assert!(output.note.contains("§ 318(a)(4)"));
        assert!(output.note.contains("§ 318(a)(5)(D)"));
        assert!(output.note.contains("substantially-disproportionate"));
    }

    #[test]
    fn re_attribution_family_to_family_disallowed() {
        let mut input = base();
        input.attribution_path = AttributionPath::ReAttributionFamilyToFamilyDisallowed;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section318A5BReAttributionDisallowedFamilyToFamily
        );
        assert!(output.note.contains("§ 318(a)(5)(B)"));
        assert!(output.note.contains("RE-ATTRIBUTION DISALLOWED"));
    }

    #[test]
    fn re_attribution_entity_bounce_disallowed() {
        let mut input = base();
        input.attribution_path = AttributionPath::ReAttributionEntityBounceDisallowed;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::Section318A5CReAttributionDisallowedEntityBounce
        );
        assert!(output.note.contains("§ 318(a)(5)(C)"));
        assert!(output.note.contains("double-bounce"));
    }

    #[test]
    fn fifty_pct_threshold_constant_pins_5000_bps() {
        assert_eq!(FIFTY_PCT_THRESHOLD_BPS, 5_000);
    }

    #[test]
    fn very_large_shares_no_overflow() {
        let mut input = base();
        input.attribution_path = AttributionPath::FromPartnershipOrEstateProportional;
        input.direct_ownership_shares = u64::MAX;
        let output = check(&input);
        assert!(output.attributed_shares > 0);
    }

    #[test]
    fn zero_proportional_yields_zero_attribution() {
        let mut input = base();
        input.attribution_path = AttributionPath::FromPartnershipOrEstateProportional;
        input.proportional_attribution_percentage_bps = 0;
        let output = check(&input);
        assert_eq!(output.attributed_shares, 0);
    }

    #[test]
    fn note_pins_section_267_sibling_inclusion_contrast() {
        let mut input = base();
        input.attribution_path = AttributionPath::SiblingAttemptDoesNotApplyToSection318;
        let output = check(&input);
        assert!(output.note.contains("§ 267"));
    }

    #[test]
    fn note_pins_section_542_phc_ownership_test() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("§ 542"));
    }

    #[test]
    fn note_pins_section_856_h_reit_test() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("§ 856(h)"));
    }

    #[test]
    fn note_pins_section_304_brother_sister() {
        let input = base();
        let output = check(&input);
        assert!(output.note.contains("§ 304"));
    }

    #[test]
    fn note_pins_section_7520_actuarial_for_trusts() {
        let mut input = base();
        input.attribution_path = AttributionPath::FromPartnershipOrEstateProportional;
        let output = check(&input);
        assert!(output.note.contains("§ 7520"));
    }
}
