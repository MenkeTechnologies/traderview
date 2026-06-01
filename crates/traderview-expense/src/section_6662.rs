//! IRC §6662 — Accuracy-related penalty on underpayments.
//!
//! The most-litigated taxpayer penalty in the Code. Routinely
//! asserted against active traders whose returns the IRS audits —
//! 20% (or 40% in gross-valuation cases) of the underpayment
//! attributable to any of 8 enumerated misconduct categories,
//! stacking-prohibited but applied automatically once a category
//! is triggered.
//!
//! **§6662(a) — base rate**: 20% of the portion of underpayment
//! attributable to a listed misconduct.
//!
//! **§6662(h) — enhanced rate**: 40% for **gross** valuation
//! misstatements (value claimed ≥ 200% of correct value) or gross
//! estate/gift valuation understatements (value claimed ≤ 25% of
//! correct).
//!
//! **§6662(b) misconduct categories** (any of which triggers):
//! - (1) Negligence or disregard of rules or regulations
//! - (2) Substantial understatement of income tax (§6662(d))
//! - (3) Substantial valuation misstatement under chapter 1
//!   (claimed value ≥ 150% of correct)
//! - (4) Substantial overstatement of pension liabilities
//! - (5) Substantial estate / gift tax valuation understatement
//! - (6) Disallowance of claimed tax benefits via §7701(o)
//!   economic-substance failure
//! - (7) Undisclosed foreign financial asset understatement
//! - (8) Inconsistent estate basis (§6035)
//!
//! **§6662(d) substantial understatement thresholds** — exceeded
//! when the understatement of tax exceeds:
//! - Individual: greater of 10% of the tax required to be shown
//!   on the return OR $5,000.
//! - C corporation (non-S, non-PHC): greater of 10% of correct tax
//!   OR $10,000, capped at $10,000,000.
//!
//! **§6664(c) reasonable-cause defense**: no penalty if reasonable
//! cause + good faith with respect to the portion. Defense not
//! available for §6662(b)(6) economic-substance failures or
//! §6662(b)(7) undisclosed foreign financial assets.
//!
//! **No stacking**: §6662 caps the maximum penalty at 20% (40% for
//! gross) per portion of underpayment regardless of how many
//! categories the misconduct falls into.
//!
//! Sources:
//! [Cornell LII 26 U.S.C. § 6662](https://www.law.cornell.edu/uscode/text/26/6662),
//! [Cornell LII 26 CFR § 1.6662-2 accuracy-related penalty](https://www.law.cornell.edu/cfr/text/26/1.6662-2),
//! [Taxpayer Advocate — Accuracy-Related Penalty Under §6662(b)(1) and (2)](https://www.taxpayeradvocate.irs.gov/wp-content/uploads/2020/07/ARC18_Volume1_MLI_01_AccuracyRelatedPenalty.pdf),
//! [The Tax Adviser — Accuracy-Related Penalty Part I](https://www.thetaxadviser.com/issues/2010/apr/theaccuracy-relatedpenaltyparti/).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxpayerType {
    Individual,
    CCorporation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MisconductCategory {
    /// §6662(b)(1) negligence or disregard of rules or regulations.
    NegligenceOrDisregard,
    /// §6662(b)(2) substantial understatement of income tax.
    SubstantialUnderstatement,
    /// §6662(b)(3) substantial valuation misstatement (claimed value
    /// ≥ 150% of correct value).
    SubstantialValuationMisstatement,
    /// §6662(h) gross valuation misstatement (claimed value ≥ 200%
    /// of correct value) — 40% rate.
    GrossValuationMisstatement,
    /// §6662(b)(6) economic-substance failure under §7701(o); no
    /// reasonable-cause defense available.
    EconomicSubstanceFailure,
    /// §6662(b)(7) undisclosed foreign financial asset under-
    /// statement; no reasonable-cause defense available.
    UndisclosedForeignFinancialAsset,
    /// None of the enumerated categories triggered.
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section6662Input {
    pub taxpayer_type: TaxpayerType,
    /// The portion of underpayment subject to penalty consideration.
    pub underpayment_dollars: i64,
    /// Total correct tax required to be shown on the return
    /// (denominator for §6662(d) substantial-understatement test).
    pub correct_tax_required_dollars: i64,
    /// True if the taxpayer was negligent or disregarded rules.
    pub negligence_or_disregard: bool,
    /// Value taxpayer claimed for the §6662(b)(3) / §6662(h)
    /// valuation analysis.
    pub claimed_value_dollars: i64,
    /// Correct value for the §6662(b)(3) / §6662(h) valuation
    /// analysis. Zero if no valuation misstatement is at issue.
    pub correct_value_for_valuation_dollars: i64,
    /// True if the transaction giving rise to the underpayment
    /// failed the §7701(o) economic-substance doctrine.
    pub economic_substance_failure: bool,
    /// True if the underpayment is attributable to an undisclosed
    /// foreign financial asset.
    pub undisclosed_foreign_financial_asset: bool,
    /// True if the taxpayer has established reasonable cause + good
    /// faith under §6664(c) (defense unavailable for §6662(b)(6)
    /// and (b)(7)).
    pub reasonable_cause_and_good_faith_established: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section6662Result {
    pub triggered_category: MisconductCategory,
    pub substantial_understatement_threshold_dollars: i64,
    pub substantial_understatement_triggered: bool,
    /// Penalty rate in basis points (2000 = 20%; 4000 = 40%).
    pub penalty_rate_bp: u32,
    /// Whether the §6664(c) reasonable-cause defense applies on
    /// these facts (note: defense unavailable for §6662(b)(6)
    /// and (b)(7) regardless of cause).
    pub reasonable_cause_defense_applies: bool,
    pub penalty_dollars: i64,
    pub citation: String,
    pub note: String,
}

const INDIVIDUAL_DOLLAR_FLOOR: i64 = 5_000;
const CORPORATE_DOLLAR_FLOOR: i64 = 10_000;
const CORPORATE_DOLLAR_CAP: i64 = 10_000_000;
const SUBSTANTIAL_MISSTATEMENT_PCT_BP: u32 = 15_000; // 150%
const GROSS_MISSTATEMENT_PCT_BP: u32 = 20_000; // 200%

pub fn compute(input: &Section6662Input) -> Section6662Result {
    // §6662(d) substantial-understatement threshold.
    let ten_pct_correct = (input.correct_tax_required_dollars.max(0) as i128 * 10 / 100) as i64;
    let threshold = match input.taxpayer_type {
        TaxpayerType::Individual => ten_pct_correct.max(INDIVIDUAL_DOLLAR_FLOOR),
        TaxpayerType::CCorporation => {
            ten_pct_correct.clamp(CORPORATE_DOLLAR_FLOOR, CORPORATE_DOLLAR_CAP)
        }
    };

    let substantial_understatement = input.underpayment_dollars > threshold;

    // Valuation misstatement analysis: ratio of claimed to correct.
    let valuation_ratio_bp: u32 = if input.correct_value_for_valuation_dollars > 0 {
        ((input.claimed_value_dollars as i128 * 10_000
            / input.correct_value_for_valuation_dollars as i128)
            .max(0) as u64)
            .min(u32::MAX as u64) as u32
    } else {
        0
    };
    let gross_valuation = valuation_ratio_bp >= GROSS_MISSTATEMENT_PCT_BP;
    let substantial_valuation =
        !gross_valuation && valuation_ratio_bp >= SUBSTANTIAL_MISSTATEMENT_PCT_BP;

    // Category selection (priority: gross > substantial valuation >
    // economic substance > foreign asset > substantial understatement
    // > negligence). Highest-rate category wins when both apply.
    let triggered = if gross_valuation {
        MisconductCategory::GrossValuationMisstatement
    } else if input.economic_substance_failure {
        MisconductCategory::EconomicSubstanceFailure
    } else if input.undisclosed_foreign_financial_asset {
        MisconductCategory::UndisclosedForeignFinancialAsset
    } else if substantial_valuation {
        MisconductCategory::SubstantialValuationMisstatement
    } else if substantial_understatement {
        MisconductCategory::SubstantialUnderstatement
    } else if input.negligence_or_disregard {
        MisconductCategory::NegligenceOrDisregard
    } else {
        MisconductCategory::None
    };

    let penalty_rate_bp: u32 = match triggered {
        MisconductCategory::GrossValuationMisstatement => 4000, // 40%
        MisconductCategory::NegligenceOrDisregard
        | MisconductCategory::SubstantialUnderstatement
        | MisconductCategory::SubstantialValuationMisstatement
        | MisconductCategory::EconomicSubstanceFailure
        | MisconductCategory::UndisclosedForeignFinancialAsset => 2000, // 20%
        MisconductCategory::None => 0,
    };

    // §6664(c) reasonable-cause defense — unavailable for §6662(b)(6)
    // economic-substance failures and §6662(b)(7) undisclosed
    // foreign financial assets.
    let defense_unavailable_for_category = matches!(
        triggered,
        MisconductCategory::EconomicSubstanceFailure
            | MisconductCategory::UndisclosedForeignFinancialAsset
    );
    let reasonable_cause_defense_applies = input.reasonable_cause_and_good_faith_established
        && !defense_unavailable_for_category
        && !matches!(triggered, MisconductCategory::None);

    let final_rate_bp = if reasonable_cause_defense_applies {
        0
    } else {
        penalty_rate_bp
    };

    let penalty = ((input.underpayment_dollars.max(0) as i128) * (final_rate_bp as i128) / 10_000)
        as i64;

    let category_label = match triggered {
        MisconductCategory::NegligenceOrDisregard => "§6662(b)(1) negligence or disregard",
        MisconductCategory::SubstantialUnderstatement => {
            "§6662(b)(2) substantial understatement"
        }
        MisconductCategory::SubstantialValuationMisstatement => {
            "§6662(b)(3) substantial valuation misstatement"
        }
        MisconductCategory::GrossValuationMisstatement => {
            "§6662(h) GROSS valuation misstatement (40% rate)"
        }
        MisconductCategory::EconomicSubstanceFailure => {
            "§6662(b)(6) economic-substance failure (NO §6664(c) defense)"
        }
        MisconductCategory::UndisclosedForeignFinancialAsset => {
            "§6662(b)(7) undisclosed foreign financial asset (NO §6664(c) defense)"
        }
        MisconductCategory::None => "no §6662 category triggered",
    };

    let note = format!(
        "Taxpayer type: {:?}; underpayment ${}; correct tax ${}; substantial-understatement threshold ${} ({}); category: {}; penalty rate {}.{}%; reasonable-cause defense {}; final penalty ${}.",
        input.taxpayer_type,
        input.underpayment_dollars,
        input.correct_tax_required_dollars,
        threshold,
        if substantial_understatement { "EXCEEDED" } else { "not exceeded" },
        category_label,
        final_rate_bp / 100,
        final_rate_bp % 100,
        if reasonable_cause_defense_applies {
            "APPLIES — no penalty"
        } else if defense_unavailable_for_category {
            "UNAVAILABLE for this category"
        } else if input.reasonable_cause_and_good_faith_established {
            "asserted"
        } else {
            "not established"
        },
        penalty,
    );

    Section6662Result {
        triggered_category: triggered,
        substantial_understatement_threshold_dollars: threshold,
        substantial_understatement_triggered: substantial_understatement,
        penalty_rate_bp: final_rate_bp,
        reasonable_cause_defense_applies,
        penalty_dollars: penalty,
        citation:
            "IRC §6662(a) 20% accuracy-related penalty on underpayment; §6662(h) 40% rate for gross valuation misstatements (claimed ≥ 200% correct); §6662(b) 8 misconduct categories (negligence; substantial understatement; substantial valuation misstatement ≥ 150%; pension overstatement; estate/gift valuation understatement; §7701(o) economic substance failure; undisclosed foreign financial asset; inconsistent estate basis); §6662(d) substantial understatement = exceeds greater of 10% of correct tax or $5,000 individual / $10,000 corporate (capped at $10,000,000 for C corps); §6664(c) reasonable-cause-and-good-faith defense — UNAVAILABLE for §6662(b)(6) economic substance and §6662(b)(7) foreign-asset categories; no stacking (max 20% / 40%)"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section6662Input {
        Section6662Input {
            taxpayer_type: TaxpayerType::Individual,
            underpayment_dollars: 100_000,
            correct_tax_required_dollars: 500_000,
            negligence_or_disregard: false,
            claimed_value_dollars: 0,
            correct_value_for_valuation_dollars: 0,
            economic_substance_failure: false,
            undisclosed_foreign_financial_asset: false,
            reasonable_cause_and_good_faith_established: false,
        }
    }

    // ── §6662(d) substantial-understatement thresholds ─────────────

    #[test]
    fn individual_threshold_max_of_10_pct_or_5k() {
        // $500k correct tax × 10% = $50k > $5k floor → threshold $50k.
        let r = compute(&base());
        assert_eq!(r.substantial_understatement_threshold_dollars, 50_000);
    }

    #[test]
    fn individual_threshold_5k_floor_when_correct_tax_low() {
        // $30k correct tax × 10% = $3k < $5k floor → threshold $5k.
        let mut i = base();
        i.correct_tax_required_dollars = 30_000;
        let r = compute(&i);
        assert_eq!(r.substantial_understatement_threshold_dollars, 5_000);
    }

    #[test]
    fn corporate_threshold_10k_floor_when_correct_tax_low() {
        let mut i = base();
        i.taxpayer_type = TaxpayerType::CCorporation;
        i.correct_tax_required_dollars = 50_000;
        let r = compute(&i);
        // 10% = $5k < $10k floor → $10k.
        assert_eq!(r.substantial_understatement_threshold_dollars, 10_000);
    }

    #[test]
    fn corporate_threshold_capped_at_10m() {
        let mut i = base();
        i.taxpayer_type = TaxpayerType::CCorporation;
        i.correct_tax_required_dollars = 1_000_000_000; // $1B
        let r = compute(&i);
        // 10% = $100M > $10M cap → threshold $10M.
        assert_eq!(r.substantial_understatement_threshold_dollars, 10_000_000);
    }

    // ── Triggering categories ──────────────────────────────────────

    #[test]
    fn negligence_triggers_20_pct() {
        let mut i = base();
        i.underpayment_dollars = 10_000; // below threshold
        i.negligence_or_disregard = true;
        let r = compute(&i);
        assert_eq!(r.triggered_category, MisconductCategory::NegligenceOrDisregard);
        assert_eq!(r.penalty_rate_bp, 2000);
        assert_eq!(r.penalty_dollars, 2_000);
    }

    #[test]
    fn substantial_understatement_triggers_20_pct() {
        // $100k underpayment > $50k threshold.
        let r = compute(&base());
        assert!(r.substantial_understatement_triggered);
        assert_eq!(r.triggered_category, MisconductCategory::SubstantialUnderstatement);
        assert_eq!(r.penalty_dollars, 20_000);
    }

    #[test]
    fn under_threshold_no_substantial_understatement() {
        let mut i = base();
        i.underpayment_dollars = 49_999;
        let r = compute(&i);
        assert!(!r.substantial_understatement_triggered);
        assert_eq!(r.triggered_category, MisconductCategory::None);
        assert_eq!(r.penalty_dollars, 0);
    }

    // ── Valuation misstatement ratios ──────────────────────────────

    #[test]
    fn valuation_at_150_pct_substantial_misstatement() {
        let mut i = base();
        i.claimed_value_dollars = 150_000;
        i.correct_value_for_valuation_dollars = 100_000;
        let r = compute(&i);
        assert_eq!(
            r.triggered_category,
            MisconductCategory::SubstantialValuationMisstatement
        );
        assert_eq!(r.penalty_rate_bp, 2000);
    }

    #[test]
    fn valuation_at_149_pct_no_misstatement_trigger() {
        let mut i = base();
        i.underpayment_dollars = 1_000; // below threshold
        i.claimed_value_dollars = 149_000;
        i.correct_value_for_valuation_dollars = 100_000;
        let r = compute(&i);
        assert_ne!(
            r.triggered_category,
            MisconductCategory::SubstantialValuationMisstatement
        );
    }

    #[test]
    fn valuation_at_200_pct_gross_misstatement_40_pct_rate() {
        let mut i = base();
        i.claimed_value_dollars = 200_000;
        i.correct_value_for_valuation_dollars = 100_000;
        let r = compute(&i);
        assert_eq!(
            r.triggered_category,
            MisconductCategory::GrossValuationMisstatement
        );
        assert_eq!(r.penalty_rate_bp, 4000);
        assert_eq!(r.penalty_dollars, 40_000);
    }

    #[test]
    fn valuation_at_300_pct_still_gross_40_pct_rate() {
        let mut i = base();
        i.claimed_value_dollars = 300_000;
        i.correct_value_for_valuation_dollars = 100_000;
        let r = compute(&i);
        assert_eq!(r.penalty_rate_bp, 4000);
    }

    // ── Economic substance — no §6664(c) defense ──────────────────

    #[test]
    fn economic_substance_failure_no_reasonable_cause_defense() {
        let mut i = base();
        i.underpayment_dollars = 10_000;
        i.economic_substance_failure = true;
        i.reasonable_cause_and_good_faith_established = true;
        let r = compute(&i);
        assert_eq!(
            r.triggered_category,
            MisconductCategory::EconomicSubstanceFailure
        );
        assert!(
            !r.reasonable_cause_defense_applies,
            "§6664(c) defense unavailable for §6662(b)(6)"
        );
        assert_eq!(r.penalty_dollars, 2_000);
    }

    #[test]
    fn foreign_asset_understatement_no_reasonable_cause_defense() {
        let mut i = base();
        i.underpayment_dollars = 10_000;
        i.undisclosed_foreign_financial_asset = true;
        i.reasonable_cause_and_good_faith_established = true;
        let r = compute(&i);
        assert!(!r.reasonable_cause_defense_applies);
        assert_eq!(r.penalty_dollars, 2_000);
    }

    // ── §6664(c) reasonable-cause defense — available paths ──────

    #[test]
    fn reasonable_cause_zeros_substantial_understatement_penalty() {
        let mut i = base();
        i.reasonable_cause_and_good_faith_established = true;
        let r = compute(&i);
        assert!(r.reasonable_cause_defense_applies);
        assert_eq!(r.penalty_dollars, 0);
        assert_eq!(r.penalty_rate_bp, 0);
    }

    #[test]
    fn reasonable_cause_zeros_negligence_penalty() {
        let mut i = base();
        i.underpayment_dollars = 1_000; // below threshold
        i.negligence_or_disregard = true;
        i.reasonable_cause_and_good_faith_established = true;
        let r = compute(&i);
        assert!(r.reasonable_cause_defense_applies);
        assert_eq!(r.penalty_dollars, 0);
    }

    #[test]
    fn reasonable_cause_zeros_gross_valuation_penalty() {
        let mut i = base();
        i.claimed_value_dollars = 300_000;
        i.correct_value_for_valuation_dollars = 100_000;
        i.reasonable_cause_and_good_faith_established = true;
        let r = compute(&i);
        assert!(r.reasonable_cause_defense_applies);
        assert_eq!(r.penalty_dollars, 0);
    }

    // ── Category precedence (gross > others) ──────────────────────

    #[test]
    fn gross_valuation_beats_substantial_understatement_for_rate() {
        // Both triggered — gross wins (40%).
        let mut i = base();
        i.claimed_value_dollars = 250_000;
        i.correct_value_for_valuation_dollars = 100_000;
        let r = compute(&i);
        assert_eq!(
            r.triggered_category,
            MisconductCategory::GrossValuationMisstatement
        );
        assert_eq!(r.penalty_rate_bp, 4000);
    }

    #[test]
    fn no_stacking_max_penalty_capped_at_category_rate() {
        // Even with multiple categories triggered, penalty is the
        // single category's rate (20% or 40%), not summed.
        let mut i = base();
        i.negligence_or_disregard = true; // 20%
        i.claimed_value_dollars = 250_000;
        i.correct_value_for_valuation_dollars = 100_000; // gross → 40%
        let r = compute(&i);
        // 40% wins; not 60%.
        assert_eq!(r.penalty_rate_bp, 4000);
    }

    // ── No category triggered ─────────────────────────────────────

    #[test]
    fn no_trigger_no_penalty() {
        let mut i = base();
        i.underpayment_dollars = 1_000; // well below threshold
        let r = compute(&i);
        assert_eq!(r.triggered_category, MisconductCategory::None);
        assert_eq!(r.penalty_dollars, 0);
        assert_eq!(r.penalty_rate_bp, 0);
    }

    // ── Citation ───────────────────────────────────────────────────

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&base());
        assert!(r.citation.contains("§6662(a)"));
        assert!(r.citation.contains("§6662(h)"));
        assert!(r.citation.contains("§6662(b)"));
        assert!(r.citation.contains("§6662(d)"));
        assert!(r.citation.contains("§6664(c)"));
        assert!(r.citation.contains("§7701(o)"));
        assert!(r.citation.contains("200%"));
        assert!(r.citation.contains("150%"));
        assert!(r.citation.contains("no stacking"));
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn note_substantial_understatement_triggered_says_exceeded() {
        let r = compute(&base());
        assert!(r.note.contains("EXCEEDED"));
    }

    #[test]
    fn note_reasonable_cause_unavailable_for_econ_substance() {
        let mut i = base();
        i.economic_substance_failure = true;
        i.reasonable_cause_and_good_faith_established = true;
        let r = compute(&i);
        assert!(r.note.contains("UNAVAILABLE"));
    }

    // ── Precision ──────────────────────────────────────────────────

    #[test]
    fn very_large_underpayment_precision() {
        let mut i = base();
        i.underpayment_dollars = 1_000_000_000;
        i.correct_tax_required_dollars = 5_000_000_000;
        let r = compute(&i);
        assert_eq!(r.substantial_understatement_threshold_dollars, 500_000_000);
        assert_eq!(r.triggered_category, MisconductCategory::SubstantialUnderstatement);
        // $1B × 20% = $200M.
        assert_eq!(r.penalty_dollars, 200_000_000);
    }

    #[test]
    fn gross_valuation_billion_dollar_precision() {
        let mut i = base();
        i.underpayment_dollars = 500_000_000;
        i.claimed_value_dollars = 1_000_000_000;
        i.correct_value_for_valuation_dollars = 400_000_000; // 250% claimed
        let r = compute(&i);
        assert_eq!(
            r.triggered_category,
            MisconductCategory::GrossValuationMisstatement
        );
        // $500M × 40% = $200M.
        assert_eq!(r.penalty_dollars, 200_000_000);
    }
}
