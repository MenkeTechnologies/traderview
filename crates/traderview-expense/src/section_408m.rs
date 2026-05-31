//! IRC §408(m) — Investment in collectibles treated as distributions.
//!
//! Every self-directed IRA holder needs this rule. Acquisition of a
//! "collectible" by an IRA is treated as a **deemed distribution** of
//! the purchase price under §408(m)(1) — taxable income to the
//! beneficiary plus a 10% additional tax under §72(t) if the
//! beneficiary is under 59½. The IRA itself is not disqualified; just
//! the offending acquisition is recharacterized as a distribution.
//!
//! **§408(m)(2) defines "collectible"** as any work of art, rug or
//! antique, metal or gem, stamp or coin, alcoholic beverage, or "any
//! other tangible personal property" specified by IRS regulations.
//! Cryptocurrency is NOT explicitly listed but the IRS has signaled in
//! several rulings and notices that crypto held in an IRA is generally
//! treated as a collectible.
//!
//! **§408(m)(3) provides two narrow exceptions:**
//!
//! **(A) Statutory coin exception** — American Gold / Silver / Platinum
//! / Palladium Eagles ARE permitted regardless of fineness. The American
//! Gold Eagle is the canonical case: it's only 22-karat (.9167 fineness,
//! below the .995 standard), but it's explicitly authorized in 31 U.S.C.
//! § 5112 so it gets through §408(m)(3)(A) without a purity check.
//! State-issued coins are also covered.
//!
//! **(B) Bullion exception** — physical bullion meets the exception if
//! BOTH (i) the fineness meets the metal-specific threshold AND (ii) it
//! is in the physical possession of a qualified trustee (a bank or an
//! IRS-approved non-bank trustee). The fineness thresholds are:
//!
//!   - **Gold**: ≥ .995 (99.5%)
//!   - **Silver**: ≥ .999 (99.9%)
//!   - **Platinum**: ≥ .9995 (99.95%)
//!   - **Palladium**: ≥ .9995 (99.95%)
//!
//! Personal possession (gold in a home safe) fails the trustee custody
//! requirement and the deemed distribution fires. This is the "home
//! storage gold IRA" trap that promoters mis-sell.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Type of asset acquired by the IRA. The §408(m)(3) statutory and
/// bullion exceptions key off this enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CollectibleType {
    /// American Gold Eagle — statutorily authorized in 31 U.S.C. § 5112;
    /// exempt under §408(m)(3)(A) regardless of the 22-karat purity.
    AmericanGoldEagle,
    AmericanSilverEagle,
    AmericanPlatinumEagle,
    AmericanPalladiumEagle,
    /// Coin issued by a state government — exempt under §408(m)(3)(A).
    StateIssuedCoin,
    GoldBullion,
    SilverBullion,
    PlatinumBullion,
    PalladiumBullion,
    /// Cryptocurrency held in IRA — IRS treats as a collectible.
    Cryptocurrency,
    Artwork,
    Antique,
    Gem,
    /// Foreign or numismatic coin not on the American Eagle / state-
    /// issued whitelist. Common trap: South African Krugerrand and
    /// other foreign gold coins.
    OtherCoin,
    AlcoholicBeverage,
    Rug,
    Stamp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section408mInput {
    pub collectible_type: CollectibleType,
    pub purchase_price: Decimal,
    /// Purity expressed as parts per ten thousand (so .995 = 9950,
    /// .9995 = 9995, .999 = 9990). `None` if not applicable (coins).
    pub purity_per_ten_thousand: Option<u32>,
    /// True if the asset is in the physical possession of an IRA-
    /// qualified trustee (bank or IRS-approved non-bank trustee).
    pub held_by_qualified_trustee: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Section408mRule {
    /// §408(m)(3)(A) — statutorily exempted Eagle or state-issued coin.
    EagleOrStateCoinException,
    /// §408(m)(3)(B) — bullion meets purity AND trustee-custody both.
    BullionException,
    /// §408(m)(1) — prohibited collectible. Deemed distribution fires.
    ProhibitedDeemedDistribution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section408mResult {
    pub rule_path: Section408mRule,
    pub is_prohibited_collectible: bool,
    /// Amount treated as a §408(m)(1) deemed distribution. Equal to
    /// the purchase price for prohibited paths; zero for exempt paths.
    pub deemed_distribution_amount: Decimal,
    /// True if the asset meets the §408(m)(3)(B) bullion fineness +
    /// custody combination.
    pub meets_bullion_exception: bool,
    /// Diagnostic: does the purity meet the threshold for the asset's
    /// metal? Useful when both purity and custody fail simultaneously
    /// so the caller can surface the right fix to the user.
    pub purity_meets_threshold: Option<bool>,
    /// Diagnostic: does the custody arrangement meet the trustee
    /// requirement? Useful for the "home storage gold IRA" trap case.
    pub custody_qualifies: bool,
    pub citation: &'static str,
    pub note: String,
}

/// Purity threshold per metal (basis: parts per ten thousand).
const GOLD_THRESHOLD: u32 = 9950; // .995
const SILVER_THRESHOLD: u32 = 9990; // .999
const PLATINUM_THRESHOLD: u32 = 9995; // .9995
const PALLADIUM_THRESHOLD: u32 = 9995; // .9995

pub fn compute(input: &Section408mInput) -> Section408mResult {
    use CollectibleType::*;

    // Path 1: Eagle coin or state-issued coin → §408(m)(3)(A) exception.
    match input.collectible_type {
        AmericanGoldEagle
        | AmericanSilverEagle
        | AmericanPlatinumEagle
        | AmericanPalladiumEagle
        | StateIssuedCoin => {
            return Section408mResult {
                rule_path: Section408mRule::EagleOrStateCoinException,
                is_prohibited_collectible: false,
                deemed_distribution_amount: Decimal::ZERO,
                meets_bullion_exception: false,
                purity_meets_threshold: None,
                custody_qualifies: input.held_by_qualified_trustee,
                citation: "IRC §408(m)(3)(A) + 31 U.S.C. § 5112",
                note: format!(
                    "§408(m)(3)(A) — statutory coin exception applies; no deemed distribution (purity check waived for {} per the statute)",
                    eagle_name(input.collectible_type)
                ),
            };
        }
        _ => {}
    }

    // Path 2: Bullion → §408(m)(3)(B) bullion exception.
    let threshold = match input.collectible_type {
        GoldBullion => Some(GOLD_THRESHOLD),
        SilverBullion => Some(SILVER_THRESHOLD),
        PlatinumBullion => Some(PLATINUM_THRESHOLD),
        PalladiumBullion => Some(PALLADIUM_THRESHOLD),
        _ => None,
    };

    if let Some(min_purity) = threshold {
        let purity_ok = input
            .purity_per_ten_thousand
            .map(|p| p >= min_purity)
            .unwrap_or(false);
        let custody_ok = input.held_by_qualified_trustee;
        let exception = purity_ok && custody_ok;
        if exception {
            return Section408mResult {
                rule_path: Section408mRule::BullionException,
                is_prohibited_collectible: false,
                deemed_distribution_amount: Decimal::ZERO,
                meets_bullion_exception: true,
                purity_meets_threshold: Some(true),
                custody_qualifies: true,
                citation: "IRC §408(m)(3)(B)",
                note: format!(
                    "§408(m)(3)(B) bullion exception satisfied: {} purity {} ≥ threshold {} AND held by qualified trustee — no deemed distribution",
                    bullion_name(input.collectible_type),
                    input.purity_per_ten_thousand.unwrap_or(0),
                    min_purity
                ),
            };
        }
        // Bullion fails one or both prongs → deemed distribution.
        let fail_reason = match (purity_ok, custody_ok) {
            (false, false) => "BOTH purity below threshold AND not held by qualified trustee",
            (false, true) => "purity below threshold",
            (true, false) => "not held by qualified trustee (home-storage trap)",
            (true, true) => unreachable!(),
        };
        return Section408mResult {
            rule_path: Section408mRule::ProhibitedDeemedDistribution,
            is_prohibited_collectible: true,
            deemed_distribution_amount: input.purchase_price,
            meets_bullion_exception: false,
            purity_meets_threshold: Some(purity_ok),
            custody_qualifies: custody_ok,
            citation: "IRC §408(m)(1) + (3)(B)",
            note: format!(
                "§408(m)(1) deemed distribution = ${}: {} fails §408(m)(3)(B) bullion exception ({}). 10% §72(t) penalty if under 59½",
                input.purchase_price.round_dp(2),
                bullion_name(input.collectible_type),
                fail_reason
            ),
        };
    }

    // Path 3: Pure-collectible categories → no exception possible.
    Section408mResult {
        rule_path: Section408mRule::ProhibitedDeemedDistribution,
        is_prohibited_collectible: true,
        deemed_distribution_amount: input.purchase_price,
        meets_bullion_exception: false,
        purity_meets_threshold: None,
        custody_qualifies: input.held_by_qualified_trustee,
        citation: "IRC §408(m)(1) + (2)",
        note: format!(
            "§408(m)(1) deemed distribution = ${}: {} is a prohibited collectible under §408(m)(2) — no exception available. 10% §72(t) penalty if under 59½",
            input.purchase_price.round_dp(2),
            collectible_name(input.collectible_type)
        ),
    }
}

fn eagle_name(c: CollectibleType) -> &'static str {
    match c {
        CollectibleType::AmericanGoldEagle => "American Gold Eagle",
        CollectibleType::AmericanSilverEagle => "American Silver Eagle",
        CollectibleType::AmericanPlatinumEagle => "American Platinum Eagle",
        CollectibleType::AmericanPalladiumEagle => "American Palladium Eagle",
        CollectibleType::StateIssuedCoin => "state-issued coin",
        _ => "Eagle/state coin",
    }
}

fn bullion_name(c: CollectibleType) -> &'static str {
    match c {
        CollectibleType::GoldBullion => "gold bullion",
        CollectibleType::SilverBullion => "silver bullion",
        CollectibleType::PlatinumBullion => "platinum bullion",
        CollectibleType::PalladiumBullion => "palladium bullion",
        _ => "bullion",
    }
}

fn collectible_name(c: CollectibleType) -> &'static str {
    use CollectibleType::*;
    match c {
        Cryptocurrency => "cryptocurrency",
        Artwork => "artwork",
        Antique => "antique",
        Gem => "gem",
        OtherCoin => "non-Eagle / non-state-issued coin (Krugerrand etc.)",
        AlcoholicBeverage => "alcoholic beverage",
        Rug => "rug",
        Stamp => "stamp",
        _ => "collectible",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn input(t: CollectibleType, price: Decimal) -> Section408mInput {
        Section408mInput {
            collectible_type: t,
            purchase_price: price,
            purity_per_ten_thousand: None,
            held_by_qualified_trustee: false,
        }
    }

    #[test]
    fn american_gold_eagle_exempt_regardless_of_purity() {
        // The Gold Eagle is 22-karat (.9167) which is BELOW the .995
        // bullion threshold — but it's statutorily authorized so the
        // purity check is waived. Pinned because this is the canonical
        // §408(m)(3)(A) edge case.
        let r = compute(&input(CollectibleType::AmericanGoldEagle, dec!(2000)));
        assert!(!r.is_prohibited_collectible);
        assert_eq!(r.rule_path, Section408mRule::EagleOrStateCoinException);
        assert_eq!(r.deemed_distribution_amount, Decimal::ZERO);
        assert!(r.note.contains("31 U.S.C. § 5112") || r.citation.contains("31 U.S.C. § 5112"));
    }

    #[test]
    fn american_silver_eagle_exempt() {
        let r = compute(&input(CollectibleType::AmericanSilverEagle, dec!(25)));
        assert!(!r.is_prohibited_collectible);
        assert_eq!(r.rule_path, Section408mRule::EagleOrStateCoinException);
    }

    #[test]
    fn american_platinum_eagle_exempt() {
        let r = compute(&input(CollectibleType::AmericanPlatinumEagle, dec!(1000)));
        assert!(!r.is_prohibited_collectible);
    }

    #[test]
    fn american_palladium_eagle_exempt() {
        let r = compute(&input(CollectibleType::AmericanPalladiumEagle, dec!(800)));
        assert!(!r.is_prohibited_collectible);
    }

    #[test]
    fn state_issued_coin_exempt() {
        let r = compute(&input(CollectibleType::StateIssuedCoin, dec!(100)));
        assert!(!r.is_prohibited_collectible);
        assert_eq!(r.rule_path, Section408mRule::EagleOrStateCoinException);
    }

    #[test]
    fn gold_bullion_995_with_trustee_passes() {
        // .995 exact = 9950 — the bright-line gold threshold.
        let mut i = input(CollectibleType::GoldBullion, dec!(50_000));
        i.purity_per_ten_thousand = Some(9950);
        i.held_by_qualified_trustee = true;
        let r = compute(&i);
        assert!(!r.is_prohibited_collectible);
        assert_eq!(r.rule_path, Section408mRule::BullionException);
        assert!(r.meets_bullion_exception);
    }

    #[test]
    fn gold_bullion_994_purity_fails() {
        // 9949 = .9949 < .995 → fails purity prong even with trustee.
        let mut i = input(CollectibleType::GoldBullion, dec!(50_000));
        i.purity_per_ten_thousand = Some(9949);
        i.held_by_qualified_trustee = true;
        let r = compute(&i);
        assert!(r.is_prohibited_collectible);
        assert_eq!(r.deemed_distribution_amount, dec!(50_000));
        assert_eq!(r.purity_meets_threshold, Some(false));
        assert!(r.custody_qualifies);
        assert!(r.note.contains("purity below threshold"));
    }

    #[test]
    fn gold_bullion_995_without_trustee_home_storage_trap() {
        // Purity ok but not held by trustee = "home storage gold IRA"
        // trap that promoters sell. Pinned — this is the load-bearing
        // anti-fraud failure mode the rule catches.
        let mut i = input(CollectibleType::GoldBullion, dec!(100_000));
        i.purity_per_ten_thousand = Some(9999);
        i.held_by_qualified_trustee = false;
        let r = compute(&i);
        assert!(r.is_prohibited_collectible);
        assert_eq!(r.deemed_distribution_amount, dec!(100_000));
        assert_eq!(r.purity_meets_threshold, Some(true));
        assert!(!r.custody_qualifies);
        assert!(r.note.contains("home-storage trap"));
    }

    #[test]
    fn gold_bullion_both_prongs_fail() {
        // Low purity AND no trustee. Note must mention both.
        let mut i = input(CollectibleType::GoldBullion, dec!(75_000));
        i.purity_per_ten_thousand = Some(9000);
        i.held_by_qualified_trustee = false;
        let r = compute(&i);
        assert!(r.is_prohibited_collectible);
        assert!(r.note.contains("BOTH"));
    }

    #[test]
    fn silver_bullion_999_with_trustee_passes() {
        let mut i = input(CollectibleType::SilverBullion, dec!(5_000));
        i.purity_per_ten_thousand = Some(9990);
        i.held_by_qualified_trustee = true;
        let r = compute(&i);
        assert!(!r.is_prohibited_collectible);
    }

    #[test]
    fn silver_bullion_995_fails_too_low() {
        // Silver requires .999 not .995 — distinct from gold.
        let mut i = input(CollectibleType::SilverBullion, dec!(5_000));
        i.purity_per_ten_thousand = Some(9950);
        i.held_by_qualified_trustee = true;
        let r = compute(&i);
        assert!(r.is_prohibited_collectible);
    }

    #[test]
    fn platinum_bullion_9995_passes() {
        // Platinum requires .9995 — higher than silver/gold.
        let mut i = input(CollectibleType::PlatinumBullion, dec!(20_000));
        i.purity_per_ten_thousand = Some(9995);
        i.held_by_qualified_trustee = true;
        let r = compute(&i);
        assert!(!r.is_prohibited_collectible);
    }

    #[test]
    fn platinum_bullion_9994_fails() {
        // .9994 < .9995 → fails by one part.
        let mut i = input(CollectibleType::PlatinumBullion, dec!(20_000));
        i.purity_per_ten_thousand = Some(9994);
        i.held_by_qualified_trustee = true;
        let r = compute(&i);
        assert!(r.is_prohibited_collectible);
    }

    #[test]
    fn palladium_bullion_9995_passes() {
        // Same threshold as platinum.
        let mut i = input(CollectibleType::PalladiumBullion, dec!(15_000));
        i.purity_per_ten_thousand = Some(9995);
        i.held_by_qualified_trustee = true;
        let r = compute(&i);
        assert!(!r.is_prohibited_collectible);
    }

    #[test]
    fn cryptocurrency_in_ira_is_prohibited_collectible() {
        // IRS treats crypto in IRA as collectible per several rulings.
        // Deemed distribution of full purchase price.
        let r = compute(&input(CollectibleType::Cryptocurrency, dec!(100_000)));
        assert!(r.is_prohibited_collectible);
        assert_eq!(r.deemed_distribution_amount, dec!(100_000));
        assert!(r.note.contains("cryptocurrency"));
    }

    #[test]
    fn artwork_in_ira_prohibited() {
        let r = compute(&input(CollectibleType::Artwork, dec!(500_000)));
        assert!(r.is_prohibited_collectible);
        assert_eq!(r.deemed_distribution_amount, dec!(500_000));
    }

    #[test]
    fn other_coin_krugerrand_prohibited() {
        // South African Krugerrand is the classic foreign-coin trap.
        // Not on the Eagle whitelist and not state-issued → prohibited.
        let r = compute(&input(CollectibleType::OtherCoin, dec!(2_500)));
        assert!(r.is_prohibited_collectible);
        assert!(r.note.contains("Krugerrand"));
    }

    #[test]
    fn alcoholic_beverage_prohibited() {
        // Wine investing in IRA is a known trap.
        let r = compute(&input(CollectibleType::AlcoholicBeverage, dec!(50_000)));
        assert!(r.is_prohibited_collectible);
    }

    #[test]
    fn antique_gem_rug_stamp_all_prohibited() {
        for t in [
            CollectibleType::Antique,
            CollectibleType::Gem,
            CollectibleType::Rug,
            CollectibleType::Stamp,
        ] {
            let r = compute(&input(t, dec!(10_000)));
            assert!(r.is_prohibited_collectible);
            assert_eq!(r.rule_path, Section408mRule::ProhibitedDeemedDistribution);
        }
    }

    #[test]
    fn bullion_missing_purity_input_treated_as_failing() {
        // purity_per_ten_thousand = None for bullion → can't verify
        // exception → defaults to failing purity check.
        let mut i = input(CollectibleType::GoldBullion, dec!(10_000));
        i.purity_per_ten_thousand = None;
        i.held_by_qualified_trustee = true;
        let r = compute(&i);
        assert!(r.is_prohibited_collectible);
        assert_eq!(r.purity_meets_threshold, Some(false));
    }

    #[test]
    fn diagnostic_flags_set_correctly_for_purity_only_failure() {
        // Custody good, purity bad → purity flag false, custody flag true.
        let mut i = input(CollectibleType::GoldBullion, dec!(10_000));
        i.purity_per_ten_thousand = Some(9000);
        i.held_by_qualified_trustee = true;
        let r = compute(&i);
        assert_eq!(r.purity_meets_threshold, Some(false));
        assert!(r.custody_qualifies);
    }

    #[test]
    fn eagle_coin_does_not_require_trustee_to_be_exempt() {
        // §408(m)(3)(A) doesn't have a trustee-custody prong like
        // bullion does. Eagle coins are exempt even without trustee
        // custody. Pinned because a future implementation might
        // accidentally add the requirement.
        let mut i = input(CollectibleType::AmericanGoldEagle, dec!(2000));
        i.held_by_qualified_trustee = false;
        let r = compute(&i);
        assert!(!r.is_prohibited_collectible);
        assert_eq!(r.rule_path, Section408mRule::EagleOrStateCoinException);
    }

    #[test]
    fn note_mentions_72t_penalty_on_prohibited_paths() {
        // Every prohibited-collectible note must mention the §72(t)
        // 10% penalty since that's the load-bearing tax consequence.
        let r = compute(&input(CollectibleType::Cryptocurrency, dec!(50_000)));
        assert!(r.note.contains("§72(t)"));
        let r2 = compute(&input(CollectibleType::Artwork, dec!(50_000)));
        assert!(r2.note.contains("§72(t)"));
    }

    #[test]
    fn deemed_distribution_zero_for_exempt_paths() {
        // Eagle + state coin + valid bullion all return zero distribution.
        let eagle = compute(&input(CollectibleType::AmericanGoldEagle, dec!(2000)));
        assert_eq!(eagle.deemed_distribution_amount, Decimal::ZERO);

        let state = compute(&input(CollectibleType::StateIssuedCoin, dec!(100)));
        assert_eq!(state.deemed_distribution_amount, Decimal::ZERO);

        let mut bullion = input(CollectibleType::GoldBullion, dec!(50_000));
        bullion.purity_per_ten_thousand = Some(9999);
        bullion.held_by_qualified_trustee = true;
        let bullion_r = compute(&bullion);
        assert_eq!(bullion_r.deemed_distribution_amount, Decimal::ZERO);
    }

    #[test]
    fn very_large_purchase_no_precision_loss() {
        // $100M of gold bullion at .9999 with trustee — fully exempt.
        let mut i = input(CollectibleType::GoldBullion, dec!(100_000_000));
        i.purity_per_ten_thousand = Some(9999);
        i.held_by_qualified_trustee = true;
        let r = compute(&i);
        assert!(!r.is_prohibited_collectible);
        assert_eq!(r.deemed_distribution_amount, Decimal::ZERO);
    }

    #[test]
    fn citation_correct_per_path() {
        let eagle = compute(&input(CollectibleType::AmericanGoldEagle, dec!(2000)));
        assert!(eagle.citation.contains("§408(m)(3)(A)"));

        let mut bullion = input(CollectibleType::GoldBullion, dec!(50_000));
        bullion.purity_per_ten_thousand = Some(9999);
        bullion.held_by_qualified_trustee = true;
        let bullion_r = compute(&bullion);
        assert!(bullion_r.citation.contains("§408(m)(3)(B)"));

        let crypto = compute(&input(CollectibleType::Cryptocurrency, dec!(100_000)));
        assert!(crypto.citation.contains("§408(m)(1)"));
    }
}
