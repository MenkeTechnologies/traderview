//! IRC §59A — Base Erosion and Anti-Abuse Tax (BEAT).
//!
//! Added by TCJA § 14401 (P.L. 115-97), eff. tax years beginning after
//! 2017-12-31, and substantially modified by the One Big Beautiful
//! Bill Act (OBBBA) of 2025 for tax years beginning after 2025-12-31.
//! Targets large US C corporations that erode the US tax base via
//! deductible payments to foreign related parties.
//!
//! **Applicability gates** (§59A(e)) — all must be met:
//!
//! - **Gross receipts test**: 3-year average annual gross receipts
//!   (including aggregated §52(a)/(b) group) ≥ $500,000,000.
//! - **Base erosion percentage test**: base erosion tax benefits ÷
//!   modified denominator ≥ 3% (2% for banks and registered
//!   securities dealers).
//! - **Entity exclusions**: S corporations, REITs, and RICs are NOT
//!   "applicable taxpayers" regardless of size.
//!
//! **BEAT rate by tax year** (§59A(b)(1)(A)):
//!
//! | Tax year period         | Standard rate | Bank/dealer surcharge |
//! |-------------------------|---------------|------------------------|
//! | 2018                    | 5%            | +1% = 6%               |
//! | 2019-2025               | 10%           | +1% = 11%              |
//! | 2026+ (post-OBBBA)      | 10.5%         | +1% = 11.5%            |
//! | 2026+ (pre-OBBBA repealed) | 12.5% (would-have)  | +1% = 13.5% (would-have)  |
//!
//! **Computation** (§59A(b)(1)):
//!
//!   BEAT = max(0, BEAT_rate × Modified_Taxable_Income − Regular_Tax_Liability)
//!
//! where Modified Taxable Income (MTI) = regular taxable income +
//! base erosion tax benefits + NOL × base-erosion %; Regular Tax
//! Liability is reduced by certain credits.
//!
//! **OBBBA permanence**: TCJA scheduled the BEAT rate to step up to
//! 12.5% (13.5% banks) in 2026 and made several credits (R&D, low-
//! income housing) no longer reduce regular tax liability for BEAT —
//! OBBBA reversed both: rate becomes a permanent 10.5% (11.5% banks)
//! and pre-2026 credit treatment is made permanent.
//!
//! Sources:
//! [Cornell LII 26 U.S.C. § 59A](https://www.law.cornell.edu/uscode/text/26/59A),
//! [Tax Foundation — BEAT after OBBBA](https://taxfoundation.org/blog/beat-tax-changes-obbba-section-899/),
//! [Concentro — BEAT After OBBBA practical guide](https://www.concentro.io/blog/beat-after-obbba-a-practical-guide-to-navigating-base-erosion-and-anti-abuse-tax-rules),
//! [PKF O'Connor Davies — 2026 International Tax Planning](https://www.pkfod.com/insights/2026-international-tax-planning-what-obbba-means-for-us-multinationals/).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    /// US C corporation — subject to BEAT if other gates met.
    CCorp,
    /// Bank or registered securities dealer — 2% base-erosion gate
    /// and +1% rate surcharge.
    BankOrSecuritiesDealer,
    /// S corporation — categorically excluded by §59A(e)(2).
    SCorp,
    /// Real Estate Investment Trust — categorically excluded.
    Reit,
    /// Regulated Investment Company — categorically excluded.
    Ric,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApplicabilityResult {
    /// BEAT applies — all gates passed.
    Applicable,
    /// Entity type excluded under §59A(e)(2) (S corp / REIT / RIC).
    EntityExcluded,
    /// Gross receipts 3-year average below $500M.
    BelowGrossReceiptsThreshold,
    /// Base erosion percentage below 3% (or 2% for banks/dealers).
    BelowBaseErosionPercentage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section59aInput {
    pub tax_year: i32,
    pub entity_type: EntityType,
    /// Gross receipts for each of the 3 preceding tax years, including
    /// §52(a)/(b) aggregation group. Used to compute the 3-year average.
    pub gross_receipts_year_minus_1_dollars: i64,
    pub gross_receipts_year_minus_2_dollars: i64,
    pub gross_receipts_year_minus_3_dollars: i64,
    /// Regular taxable income under §63 before BEAT.
    pub taxable_income_dollars: i64,
    /// Deductible payments made to foreign related parties (§59A(d)
    /// base erosion payments — interest, royalties, services, etc.).
    pub base_erosion_payments_dollars: i64,
    /// Total allowable deductions (denominator of base-erosion %).
    pub total_deductions_dollars: i64,
    /// Net operating loss deduction included in taxable income (added
    /// back proportionally to MTI under §59A(c)(1)(B)).
    pub nol_deduction_dollars: i64,
    /// Regular tax liability before BEAT, reduced by allowed credits.
    pub regular_tax_liability_dollars: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section59aResult {
    pub post_obbba: bool,
    pub applicability: ApplicabilityResult,
    /// 3-year average gross receipts in dollars.
    pub three_year_average_gross_receipts_dollars: i64,
    /// Whether the $500M gross receipts test was met.
    pub meets_gross_receipts_test: bool,
    /// Base erosion percentage in basis points (e.g. 350 = 3.50%).
    pub base_erosion_percentage_bp: u32,
    /// Required threshold (300 for standard C corp, 200 for banks).
    pub base_erosion_percentage_threshold_bp: u32,
    pub meets_base_erosion_percentage_test: bool,
    /// BEAT rate in basis points (e.g. 1050 = 10.50%).
    pub beat_rate_bp: u32,
    /// Modified Taxable Income = taxable income + base erosion tax
    /// benefits + NOL × base erosion percentage.
    pub modified_taxable_income_dollars: i64,
    /// MTI × BEAT rate (tentative gross BEAT before credit offset).
    pub tentative_minimum_tax_dollars: i64,
    /// Final BEAT = max(0, tentative_minimum_tax − regular_tax_liability).
    pub beat_tax_dollars: i64,
    pub citation: String,
    pub note: String,
}

const GROSS_RECEIPTS_THRESHOLD_DOLLARS: i64 = 500_000_000;
const STANDARD_BASE_EROSION_THRESHOLD_BP: u32 = 300; // 3.0%
const BANK_BASE_EROSION_THRESHOLD_BP: u32 = 200; // 2.0%

pub fn compute(input: &Section59aInput) -> Section59aResult {
    let post_obbba = input.tax_year >= 2026;
    let is_bank = matches!(input.entity_type, EntityType::BankOrSecuritiesDealer);

    let beat_rate_bp: u32 = match (input.tax_year, is_bank, post_obbba) {
        // 2018 phase-in.
        (y, false, _) if y <= 2018 => 500,
        (y, true, _) if y <= 2018 => 600,
        // 2019-2025: 10% / 11%.
        (y, false, _) if (2019..=2025).contains(&y) => 1000,
        (y, true, _) if (2019..=2025).contains(&y) => 1100,
        // 2026+ post-OBBBA permanent rate: 10.5% / 11.5%.
        (_, false, true) => 1050,
        (_, true, true) => 1150,
        // Defensive fallback for years before 2018 — treat as not yet
        // in effect (§14401 first applied to 2018).
        _ => 0,
    };

    let three_year_avg = (input.gross_receipts_year_minus_1_dollars
        + input.gross_receipts_year_minus_2_dollars
        + input.gross_receipts_year_minus_3_dollars)
        / 3;
    let meets_gross_receipts = three_year_avg >= GROSS_RECEIPTS_THRESHOLD_DOLLARS;

    let bep_threshold_bp = if is_bank {
        BANK_BASE_EROSION_THRESHOLD_BP
    } else {
        STANDARD_BASE_EROSION_THRESHOLD_BP
    };

    let bep_bp: u32 = if input.total_deductions_dollars > 0 {
        ((input.base_erosion_payments_dollars as i128 * 10_000
            / input.total_deductions_dollars as i128)
            .max(0) as u64)
            .min(u32::MAX as u64) as u32
    } else {
        0
    };
    let meets_bep = bep_bp >= bep_threshold_bp;

    // Entity gate.
    let applicability = match input.entity_type {
        EntityType::SCorp => ApplicabilityResult::EntityExcluded,
        EntityType::Reit => ApplicabilityResult::EntityExcluded,
        EntityType::Ric => ApplicabilityResult::EntityExcluded,
        _ if !meets_gross_receipts => ApplicabilityResult::BelowGrossReceiptsThreshold,
        _ if !meets_bep => ApplicabilityResult::BelowBaseErosionPercentage,
        _ => ApplicabilityResult::Applicable,
    };

    // MTI = taxable income + base erosion tax benefits + NOL × base
    // erosion percentage. §59A(c)(1)(B). NOL is added back proportional
    // to the share of total deductions that were base-erosion payments.
    let nol_addback = if input.total_deductions_dollars > 0 {
        (input.nol_deduction_dollars as i128 * bep_bp as i128 / 10_000) as i64
    } else {
        0
    };
    let mti = input.taxable_income_dollars + input.base_erosion_payments_dollars + nol_addback;

    let tentative_min_tax = ((mti.max(0) as i128) * (beat_rate_bp as i128) / 10_000) as i64;

    let beat_tax = if matches!(applicability, ApplicabilityResult::Applicable) {
        (tentative_min_tax - input.regular_tax_liability_dollars).max(0)
    } else {
        0
    };

    let entity_label = match input.entity_type {
        EntityType::CCorp => "C corporation",
        EntityType::BankOrSecuritiesDealer => "bank or registered securities dealer",
        EntityType::SCorp => "S corporation",
        EntityType::Reit => "REIT",
        EntityType::Ric => "RIC",
    };

    let applicability_label = match applicability {
        ApplicabilityResult::Applicable => "BEAT applies",
        ApplicabilityResult::EntityExcluded => {
            "BEAT does not apply — entity type categorically excluded under §59A(e)(2)"
        }
        ApplicabilityResult::BelowGrossReceiptsThreshold => {
            "BEAT does not apply — 3-yr average gross receipts below $500M threshold"
        }
        ApplicabilityResult::BelowBaseErosionPercentage => {
            "BEAT does not apply — base erosion percentage below threshold"
        }
    };

    let note = format!(
        "Tax year {} ({}); entity type {}; 3-yr avg gross receipts ${} ({}); base erosion percentage {}.{}% vs {}.{}% threshold ({}); BEAT rate {}.{}%; MTI ${}; tentative minimum tax ${}; regular tax liability ${}; BEAT = ${}. {}.",
        input.tax_year,
        if post_obbba { "post-OBBBA permanent regime" } else { "pre-OBBBA TCJA regime" },
        entity_label,
        three_year_avg,
        if meets_gross_receipts { "meets $500M test" } else { "fails $500M test" },
        bep_bp / 100,
        bep_bp % 100,
        bep_threshold_bp / 100,
        bep_threshold_bp % 100,
        if meets_bep { "meets BEP test" } else { "fails BEP test" },
        beat_rate_bp / 100,
        beat_rate_bp % 100,
        mti,
        tentative_min_tax,
        input.regular_tax_liability_dollars,
        beat_tax,
        applicability_label,
    );

    Section59aResult {
        post_obbba,
        applicability,
        three_year_average_gross_receipts_dollars: three_year_avg,
        meets_gross_receipts_test: meets_gross_receipts,
        base_erosion_percentage_bp: bep_bp,
        base_erosion_percentage_threshold_bp: bep_threshold_bp,
        meets_base_erosion_percentage_test: meets_bep,
        beat_rate_bp,
        modified_taxable_income_dollars: mti,
        tentative_minimum_tax_dollars: tentative_min_tax,
        beat_tax_dollars: beat_tax,
        citation:
            "IRC §59A Base Erosion and Anti-Abuse Tax (TCJA P.L. 115-97 §14401, eff. tax years beginning after 2017-12-31); One Big Beautiful Bill Act of 2025 §59A amendments (eff. tax years beginning after 2025-12-31) make 10.5% rate permanent (11.5% banks/dealers), preserve 2025 credit treatment; $500M 3-yr avg gross receipts gate; 3% base erosion percentage gate (2% banks); S corps / REITs / RICs categorically excluded under §59A(e)(2)"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section59aInput {
        Section59aInput {
            tax_year: 2025,
            entity_type: EntityType::CCorp,
            gross_receipts_year_minus_1_dollars: 600_000_000,
            gross_receipts_year_minus_2_dollars: 600_000_000,
            gross_receipts_year_minus_3_dollars: 600_000_000,
            taxable_income_dollars: 100_000_000,
            base_erosion_payments_dollars: 50_000_000,
            total_deductions_dollars: 1_000_000_000,
            nol_deduction_dollars: 0,
            regular_tax_liability_dollars: 21_000_000,
        }
    }

    // ── Rate-by-year pinning ────────────────────────────────────────

    #[test]
    fn rate_2018_phase_in_5_pct() {
        let mut i = base();
        i.tax_year = 2018;
        let r = compute(&i);
        assert_eq!(r.beat_rate_bp, 500, "2018 phase-in BEAT rate is 5%");
    }

    #[test]
    fn rate_2018_banks_6_pct() {
        let mut i = base();
        i.tax_year = 2018;
        i.entity_type = EntityType::BankOrSecuritiesDealer;
        let r = compute(&i);
        assert_eq!(r.beat_rate_bp, 600, "2018 bank rate is 5%+1%=6%");
    }

    #[test]
    fn rate_2019_standard_10_pct() {
        let mut i = base();
        i.tax_year = 2019;
        let r = compute(&i);
        assert_eq!(r.beat_rate_bp, 1000);
    }

    #[test]
    fn rate_2025_standard_10_pct() {
        let mut i = base();
        i.tax_year = 2025;
        let r = compute(&i);
        assert_eq!(r.beat_rate_bp, 1000, "2025 still 10% pre-OBBBA");
    }

    #[test]
    fn rate_2025_banks_11_pct() {
        let mut i = base();
        i.tax_year = 2025;
        i.entity_type = EntityType::BankOrSecuritiesDealer;
        let r = compute(&i);
        assert_eq!(r.beat_rate_bp, 1100);
    }

    #[test]
    fn rate_2026_post_obbba_10_5_pct() {
        let mut i = base();
        i.tax_year = 2026;
        let r = compute(&i);
        assert!(r.post_obbba);
        assert_eq!(
            r.beat_rate_bp, 1050,
            "OBBBA permanent rate is 10.5%, NOT TCJA's scheduled 12.5%"
        );
    }

    #[test]
    fn rate_2026_post_obbba_banks_11_5_pct() {
        let mut i = base();
        i.tax_year = 2026;
        i.entity_type = EntityType::BankOrSecuritiesDealer;
        let r = compute(&i);
        assert_eq!(r.beat_rate_bp, 1150, "OBBBA bank rate is 10.5%+1%=11.5%");
    }

    // ── Year boundary 2025 → 2026 ──────────────────────────────────

    #[test]
    fn year_boundary_2025_pre_obbba() {
        let mut i = base();
        i.tax_year = 2025;
        let r = compute(&i);
        assert!(!r.post_obbba);
        assert_eq!(r.beat_rate_bp, 1000);
    }

    #[test]
    fn year_boundary_2026_post_obbba() {
        let mut i = base();
        i.tax_year = 2026;
        let r = compute(&i);
        assert!(r.post_obbba);
        assert_eq!(r.beat_rate_bp, 1050);
    }

    // ── Gross receipts threshold ───────────────────────────────────

    #[test]
    fn gross_receipts_below_500m_not_applicable() {
        let mut i = base();
        i.gross_receipts_year_minus_1_dollars = 400_000_000;
        i.gross_receipts_year_minus_2_dollars = 400_000_000;
        i.gross_receipts_year_minus_3_dollars = 400_000_000;
        let r = compute(&i);
        assert!(!r.meets_gross_receipts_test);
        assert_eq!(
            r.applicability,
            ApplicabilityResult::BelowGrossReceiptsThreshold
        );
        assert_eq!(r.beat_tax_dollars, 0);
    }

    #[test]
    fn gross_receipts_3yr_average_computed_correctly() {
        // Average of (1B, 500M, 300M) = 600M.
        let mut i = base();
        i.gross_receipts_year_minus_1_dollars = 1_000_000_000;
        i.gross_receipts_year_minus_2_dollars = 500_000_000;
        i.gross_receipts_year_minus_3_dollars = 300_000_000;
        let r = compute(&i);
        assert_eq!(r.three_year_average_gross_receipts_dollars, 600_000_000);
        assert!(r.meets_gross_receipts_test);
    }

    #[test]
    fn gross_receipts_exactly_500m_meets_test() {
        let mut i = base();
        i.gross_receipts_year_minus_1_dollars = 500_000_000;
        i.gross_receipts_year_minus_2_dollars = 500_000_000;
        i.gross_receipts_year_minus_3_dollars = 500_000_000;
        let r = compute(&i);
        assert!(
            r.meets_gross_receipts_test,
            "$500M exact meets threshold (>=)"
        );
    }

    // ── Base erosion percentage ────────────────────────────────────

    #[test]
    fn bep_3_pct_meets_standard_threshold() {
        // 30M / 1B = 3.0% — exactly at threshold.
        let mut i = base();
        i.base_erosion_payments_dollars = 30_000_000;
        i.total_deductions_dollars = 1_000_000_000;
        let r = compute(&i);
        assert_eq!(r.base_erosion_percentage_bp, 300);
        assert!(r.meets_base_erosion_percentage_test);
    }

    #[test]
    fn bep_below_3_pct_not_applicable_standard() {
        // 20M / 1B = 2.0% — below standard 3% threshold.
        let mut i = base();
        i.base_erosion_payments_dollars = 20_000_000;
        i.total_deductions_dollars = 1_000_000_000;
        let r = compute(&i);
        assert_eq!(r.base_erosion_percentage_bp, 200);
        assert!(!r.meets_base_erosion_percentage_test);
        assert_eq!(
            r.applicability,
            ApplicabilityResult::BelowBaseErosionPercentage
        );
    }

    #[test]
    fn bep_2_pct_meets_bank_threshold() {
        // Bank at exactly 2.0% — meets bank threshold.
        let mut i = base();
        i.entity_type = EntityType::BankOrSecuritiesDealer;
        i.base_erosion_payments_dollars = 20_000_000;
        i.total_deductions_dollars = 1_000_000_000;
        let r = compute(&i);
        assert_eq!(r.base_erosion_percentage_threshold_bp, 200);
        assert!(r.meets_base_erosion_percentage_test);
    }

    #[test]
    fn bep_2_pct_does_not_meet_standard_threshold() {
        // Standard C corp at 2.0% — fails 3% standard threshold.
        let mut i = base();
        i.entity_type = EntityType::CCorp;
        i.base_erosion_payments_dollars = 20_000_000;
        i.total_deductions_dollars = 1_000_000_000;
        let r = compute(&i);
        assert_eq!(r.base_erosion_percentage_threshold_bp, 300);
        assert!(!r.meets_base_erosion_percentage_test);
    }

    // ── Entity exclusion gate ───────────────────────────────────────

    #[test]
    fn s_corp_categorically_excluded() {
        let mut i = base();
        i.entity_type = EntityType::SCorp;
        let r = compute(&i);
        assert_eq!(r.applicability, ApplicabilityResult::EntityExcluded);
        assert_eq!(r.beat_tax_dollars, 0);
    }

    #[test]
    fn reit_categorically_excluded() {
        let mut i = base();
        i.entity_type = EntityType::Reit;
        let r = compute(&i);
        assert_eq!(r.applicability, ApplicabilityResult::EntityExcluded);
    }

    #[test]
    fn ric_categorically_excluded() {
        let mut i = base();
        i.entity_type = EntityType::Ric;
        let r = compute(&i);
        assert_eq!(r.applicability, ApplicabilityResult::EntityExcluded);
    }

    // ── BEAT computation arithmetic ────────────────────────────────

    #[test]
    fn beat_basic_2025_computation() {
        // Taxable income $100M + base erosion $50M = MTI $150M.
        // BEAT @ 10% = $15M; regular tax $21M.
        // Tentative ($15M) < regular ($21M) → BEAT = 0.
        let r = compute(&base());
        assert_eq!(r.modified_taxable_income_dollars, 150_000_000);
        assert_eq!(r.tentative_minimum_tax_dollars, 15_000_000);
        assert_eq!(r.beat_tax_dollars, 0, "tentative ≤ regular, no BEAT due");
    }

    #[test]
    fn beat_positive_when_tentative_exceeds_regular() {
        let mut i = base();
        i.regular_tax_liability_dollars = 10_000_000; // lower regular tax
        let r = compute(&i);
        // Tentative $15M − regular $10M = $5M BEAT.
        assert_eq!(r.beat_tax_dollars, 5_000_000);
    }

    #[test]
    fn beat_post_obbba_uses_10_5_pct() {
        let mut i = base();
        i.tax_year = 2026;
        i.regular_tax_liability_dollars = 10_000_000;
        let r = compute(&i);
        // MTI $150M × 10.5% = $15.75M − regular $10M = $5.75M.
        assert_eq!(r.tentative_minimum_tax_dollars, 15_750_000);
        assert_eq!(r.beat_tax_dollars, 5_750_000);
    }

    #[test]
    fn beat_zero_when_not_applicable_even_if_arithmetic_positive() {
        // BEP fails → applicability = BelowBaseErosionPercentage.
        // BEAT must report 0 regardless of what tentative math says.
        let mut i = base();
        i.base_erosion_payments_dollars = 1_000_000; // 0.1% BEP
        i.regular_tax_liability_dollars = 0;
        let r = compute(&i);
        assert!(!r.meets_base_erosion_percentage_test);
        assert_eq!(r.beat_tax_dollars, 0);
    }

    // ── NOL addback proportional to BEP ───────────────────────────

    #[test]
    fn nol_addback_proportional_to_bep() {
        // BEP = 5% of deductions; NOL $10M × 5% = $500k addback.
        let mut i = base();
        i.base_erosion_payments_dollars = 50_000_000;
        i.total_deductions_dollars = 1_000_000_000; // 5% BEP
        i.nol_deduction_dollars = 10_000_000;
        let r = compute(&i);
        // MTI = $100M + $50M + $500k = $150,500,000.
        assert_eq!(r.modified_taxable_income_dollars, 150_500_000);
    }

    // ── Citation & note ────────────────────────────────────────────

    #[test]
    fn citation_mentions_tcja_and_obbba() {
        let r = compute(&base());
        assert!(r.citation.contains("TCJA"));
        assert!(r.citation.contains("§14401"));
        assert!(r.citation.contains("One Big Beautiful Bill Act"));
        assert!(r.citation.contains("10.5%"));
    }

    #[test]
    fn note_for_2025_says_pre_obbba() {
        let r = compute(&base());
        assert!(r.note.contains("pre-OBBBA TCJA regime"));
    }

    #[test]
    fn note_for_2026_says_post_obbba() {
        let mut i = base();
        i.tax_year = 2026;
        let r = compute(&i);
        assert!(r.note.contains("post-OBBBA permanent regime"));
    }

    // ── Defensive / boundary ───────────────────────────────────────

    #[test]
    fn zero_deductions_yields_zero_bep_and_no_beat() {
        let mut i = base();
        i.total_deductions_dollars = 0;
        let r = compute(&i);
        assert_eq!(r.base_erosion_percentage_bp, 0);
        assert!(!r.meets_base_erosion_percentage_test);
        assert_eq!(r.beat_tax_dollars, 0);
    }

    #[test]
    fn negative_taxable_income_does_not_produce_negative_tentative() {
        let mut i = base();
        i.taxable_income_dollars = -200_000_000; // big loss
        i.base_erosion_payments_dollars = 0;
        i.total_deductions_dollars = 1_000_000_000;
        let r = compute(&i);
        assert!(r.modified_taxable_income_dollars < 0);
        // tentative_min_tax floored at 0 via MTI.max(0).
        assert_eq!(r.tentative_minimum_tax_dollars, 0);
    }
}
