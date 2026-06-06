//! IRC §911 — Foreign earned income exclusion for U.S. taxpayers
//! working abroad.
//!
//! Trader-relevant for U.S. citizens / lawful permanent residents
//! who relocate abroad to lower-cost or tax-favorable jurisdictions
//! while continuing to earn income from services performed in the
//! foreign country. §911 lets qualified individuals **exclude**
//! foreign earned income up to an inflation-adjusted ceiling
//! (2025: $130,000; 2026: $132,900) plus a housing exclusion equal
//! to housing expenses above a base amount, capped at 30% of the
//! FEIE.
//!
//! **§911(a)(1) foreign earned income exclusion**: a qualified
//! individual may exclude from gross income their foreign earned
//! income up to the inflation-indexed annual cap.
//!
//! **§911(a)(2) housing exclusion / deduction**: in addition, the
//! individual may exclude (or deduct, if self-employed) a housing
//! amount = (housing expenses − base amount), capped at 30% × FEIE.
//! Base amount = 16% × FEIE.
//!
//! **§911(b)(1) "foreign earned income"**: amounts received from
//! services performed in a foreign country during the period for
//! which the qualified-individual status applies. Does NOT include
//! passive income (interest, dividends, capital gains), pension
//! distributions, or amounts paid by the U.S. government.
//!
//! **§911(d)(1) qualified individual** — must satisfy ONE of:
//! - **(A) Bona fide residence test**: U.S. citizen who is a bona
//!   fide resident of a foreign country for an uninterrupted
//!   period that includes at least one full taxable year.
//! - **(B) Physical presence test**: U.S. citizen or lawful
//!   permanent resident who is present in a foreign country for
//!   at least **330 full days** during any 12-consecutive-month
//!   period.
//!
//! **§911(d)(7) base housing amount**: 16% of the FEIE
//! (2025: $20,800; 2026: $21,264). Housing expenses below this
//! base are NOT excludable.
//!
//! **§911(d)(2)(B) inflation adjustment** — published annually by
//! IRS Rev. Proc. (e.g., Rev. Proc. 2025-32 for 2026 figures).
//!
//! **Married-couple stacking**: each spouse who qualifies
//! independently can take the full FEIE. 2025 MFJ stacked
//! exclusion = $260,000 (two qualifying individuals).
//!
//! Sources:
//! [Cornell LII 26 U.S.C. § 911](https://www.law.cornell.edu/uscode/text/26/911),
//! [IRS — Foreign earned income exclusion](https://www.irs.gov/individuals/international-taxpayers/foreign-earned-income-exclusion),
//! [IRS — Figuring the FEIE](https://www.irs.gov/individuals/international-taxpayers/figuring-the-foreign-earned-income-exclusion),
//! [Rev. Proc. 2025-32 — 2026 inflation adjustments (Current Federal Tax Developments)](https://www.currentfederaltaxdevelopments.com/blog/2025/10/9/2026-inflation-adjustments-for-tax-professionals-revenue-procedure-2025-32-analysis).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualifyingTest {
    /// §911(d)(1)(A) bona fide residence — uninterrupted period
    /// including at least one full tax year.
    BonaFideResidence,
    /// §911(d)(1)(B) physical presence — ≥ 330 full days in any
    /// 12-consecutive-month period.
    PhysicalPresence,
    /// Neither test satisfied — taxpayer is not a qualified
    /// individual under §911(d)(1).
    NotQualified,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section911Input {
    pub tax_year: i32,
    /// Inflation-adjusted FEIE for the tax year, per Rev. Proc.
    /// (2025 = $130,000; 2026 = $132,900). Caller-supplied so the
    /// module is year-agnostic.
    pub feie_inflation_adjusted_cap_dollars: i64,
    /// Foreign earned income (services performed abroad) for the
    /// tax year.
    pub foreign_earned_income_dollars: i64,
    /// Actual housing expenses paid for the foreign housing
    /// (rent / utilities / parking / etc.) — for §911(a)(2)
    /// computation.
    pub housing_expenses_dollars: i64,
    /// Number of full days the taxpayer was physically present in
    /// a foreign country during the relevant 12-consecutive-month
    /// period.
    pub physical_presence_days_in_12_month_period: u32,
    /// True if the taxpayer satisfies the bona-fide-residence test
    /// (independent of the 330-day count).
    pub bona_fide_residence_satisfied: bool,
    /// True if any of the foreign earned income is from services
    /// for the U.S. government (categorically excluded from FEIE).
    pub income_from_us_government: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section911Result {
    pub qualifying_test: QualifyingTest,
    pub is_qualified_individual: bool,
    pub feie_amount_dollars: i64,
    pub foreign_earned_income_excluded_dollars: i64,
    /// Base housing amount under §911(d)(7) = 16% × FEIE.
    pub base_housing_amount_dollars: i64,
    /// Housing-amount cap under §911(c)(2) = 30% × FEIE.
    pub housing_amount_cap_dollars: i64,
    pub housing_exclusion_dollars: i64,
    /// Total amount excludable from gross income.
    pub total_section_911_exclusion_dollars: i64,
    /// Foreign earned income net of §911 exclusion.
    pub foreign_earned_income_after_exclusion_dollars: i64,
    pub citation: String,
    pub note: String,
}

const PHYSICAL_PRESENCE_THRESHOLD_DAYS: u32 = 330;
const BASE_HOUSING_PCT_BP: u32 = 1600; // 16%
const HOUSING_CAP_PCT_BP: u32 = 3000; // 30%

pub fn compute(input: &Section911Input) -> Section911Result {
    // §911(d)(1) qualified-individual determination.
    let qualifying_test = if input.bona_fide_residence_satisfied {
        QualifyingTest::BonaFideResidence
    } else if input.physical_presence_days_in_12_month_period >= PHYSICAL_PRESENCE_THRESHOLD_DAYS {
        QualifyingTest::PhysicalPresence
    } else {
        QualifyingTest::NotQualified
    };
    let qualified = !matches!(qualifying_test, QualifyingTest::NotQualified);

    let feie = input.feie_inflation_adjusted_cap_dollars.max(0);

    // §911(b)(1) U.S. government income exclusion bar.
    let eligible_foreign_income = if input.income_from_us_government {
        0
    } else {
        input.foreign_earned_income_dollars.max(0)
    };

    // §911(a)(1) FEIE.
    let income_excluded = if qualified {
        eligible_foreign_income.min(feie)
    } else {
        0
    };

    // §911(d)(7) base housing amount + §911(c)(2) cap.
    let base_housing = ((feie as i128) * (BASE_HOUSING_PCT_BP as i128) / 10_000) as i64;
    let housing_cap = ((feie as i128) * (HOUSING_CAP_PCT_BP as i128) / 10_000) as i64;

    // §911(a)(2) housing exclusion = housing expenses − base, capped
    // at 30% × FEIE.
    let housing_excess = (input.housing_expenses_dollars - base_housing).max(0);
    let housing_exclusion = if qualified {
        housing_excess.min(housing_cap)
    } else {
        0
    };

    let total_exclusion = income_excluded + housing_exclusion;
    let after_exclusion = eligible_foreign_income - income_excluded;

    let test_label = match qualifying_test {
        QualifyingTest::BonaFideResidence => "§911(d)(1)(A) bona fide residence test",
        QualifyingTest::PhysicalPresence => {
            "§911(d)(1)(B) physical presence test (≥ 330 full days)"
        }
        QualifyingTest::NotQualified => "neither §911(d)(1) test satisfied",
    };

    let note = format!(
        "Tax year {}; FEIE inflation cap ${}; qualifying test: {}; foreign earned income ${} (US-gov bar: {}); income excluded ${}; base housing ${} (16% × FEIE); housing cap ${} (30% × FEIE); housing expenses ${}; housing exclusion ${}; total §911 exclusion ${}; foreign earned income after exclusion ${}.",
        input.tax_year,
        feie,
        test_label,
        input.foreign_earned_income_dollars,
        if input.income_from_us_government { "TRIGGERED — income zeroed" } else { "not triggered" },
        income_excluded,
        base_housing,
        housing_cap,
        input.housing_expenses_dollars,
        housing_exclusion,
        total_exclusion,
        after_exclusion,
    );

    Section911Result {
        qualifying_test,
        is_qualified_individual: qualified,
        feie_amount_dollars: feie,
        foreign_earned_income_excluded_dollars: income_excluded,
        base_housing_amount_dollars: base_housing,
        housing_amount_cap_dollars: housing_cap,
        housing_exclusion_dollars: housing_exclusion,
        total_section_911_exclusion_dollars: total_exclusion,
        foreign_earned_income_after_exclusion_dollars: after_exclusion,
        citation:
            "IRC §911(a)(1) foreign earned income exclusion (inflation-indexed annual cap; 2025 = $130,000; 2026 = $132,900); §911(a)(2) housing exclusion / deduction; §911(b)(1) foreign earned income = services performed in foreign country (no interest / dividends / capital gains / pensions / U.S. government income); §911(c)(2) housing-amount cap = 30% × FEIE; §911(d)(1)(A) bona fide residence test (uninterrupted period including full tax year); §911(d)(1)(B) physical presence test (≥ 330 full days in 12-consecutive-month period); §911(d)(2)(B) inflation adjustment per Rev. Proc.; §911(d)(7) base housing amount = 16% × FEIE"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section911Input {
        Section911Input {
            tax_year: 2026,
            feie_inflation_adjusted_cap_dollars: 132_900,
            foreign_earned_income_dollars: 100_000,
            housing_expenses_dollars: 0,
            physical_presence_days_in_12_month_period: 330,
            bona_fide_residence_satisfied: false,
            income_from_us_government: false,
        }
    }

    // ── §911(d)(1) qualification ────────────────────────────────────

    #[test]
    fn physical_presence_330_days_qualifies() {
        let r = compute(&base());
        assert!(r.is_qualified_individual);
        assert_eq!(r.qualifying_test, QualifyingTest::PhysicalPresence);
    }

    #[test]
    fn physical_presence_329_days_does_not_qualify() {
        let mut i = base();
        i.physical_presence_days_in_12_month_period = 329;
        let r = compute(&i);
        assert!(!r.is_qualified_individual);
        assert_eq!(r.qualifying_test, QualifyingTest::NotQualified);
    }

    #[test]
    fn bona_fide_residence_qualifies_regardless_of_day_count() {
        let mut i = base();
        i.physical_presence_days_in_12_month_period = 0;
        i.bona_fide_residence_satisfied = true;
        let r = compute(&i);
        assert!(r.is_qualified_individual);
        assert_eq!(r.qualifying_test, QualifyingTest::BonaFideResidence);
    }

    #[test]
    fn bona_fide_residence_takes_precedence_when_both_apply() {
        let mut i = base();
        i.bona_fide_residence_satisfied = true;
        i.physical_presence_days_in_12_month_period = 340;
        let r = compute(&i);
        assert_eq!(r.qualifying_test, QualifyingTest::BonaFideResidence);
    }

    // ── §911(a)(1) FEIE ────────────────────────────────────────────

    #[test]
    fn income_below_feie_full_exclusion() {
        let mut i = base();
        i.foreign_earned_income_dollars = 100_000;
        let r = compute(&i);
        assert_eq!(r.foreign_earned_income_excluded_dollars, 100_000);
        assert_eq!(r.foreign_earned_income_after_exclusion_dollars, 0);
    }

    #[test]
    fn income_above_feie_capped_at_inflation_amount() {
        let mut i = base();
        i.foreign_earned_income_dollars = 200_000;
        let r = compute(&i);
        // 2026 FEIE = $132,900.
        assert_eq!(r.foreign_earned_income_excluded_dollars, 132_900);
        assert_eq!(r.foreign_earned_income_after_exclusion_dollars, 67_100);
    }

    #[test]
    fn income_exactly_at_feie_full_exclusion() {
        let mut i = base();
        i.foreign_earned_income_dollars = 132_900;
        let r = compute(&i);
        assert_eq!(r.foreign_earned_income_excluded_dollars, 132_900);
    }

    // ── §911(d)(7) base housing + §911(c)(2) cap ───────────────────

    #[test]
    fn base_housing_amount_16_pct_of_feie() {
        let r = compute(&base());
        // 16% × $132,900 = $21,264.
        assert_eq!(r.base_housing_amount_dollars, 21_264);
    }

    #[test]
    fn housing_cap_30_pct_of_feie() {
        let r = compute(&base());
        // 30% × $132,900 = $39,870.
        assert_eq!(r.housing_amount_cap_dollars, 39_870);
    }

    #[test]
    fn housing_expenses_below_base_no_exclusion() {
        let mut i = base();
        i.housing_expenses_dollars = 20_000; // < $21,264 base
        let r = compute(&i);
        assert_eq!(r.housing_exclusion_dollars, 0);
    }

    #[test]
    fn housing_expenses_between_base_and_cap_excess_excluded() {
        let mut i = base();
        i.housing_expenses_dollars = 35_000;
        let r = compute(&i);
        // $35,000 − $21,264 = $13,736 excludable.
        assert_eq!(r.housing_exclusion_dollars, 13_736);
    }

    #[test]
    fn housing_expenses_above_cap_capped_at_30_pct() {
        let mut i = base();
        i.housing_expenses_dollars = 100_000;
        let r = compute(&i);
        // Cap = $39,870. $100k − $21,264 = $78,736, capped at $39,870.
        assert_eq!(r.housing_exclusion_dollars, 39_870);
    }

    // ── §911(b)(1) U.S. government income bar ──────────────────────

    #[test]
    fn us_government_income_zeroes_eligible_income() {
        let mut i = base();
        i.income_from_us_government = true;
        let r = compute(&i);
        assert_eq!(r.foreign_earned_income_excluded_dollars, 0);
    }

    // ── Not qualified — no exclusion ───────────────────────────────

    #[test]
    fn not_qualified_no_exclusion() {
        let mut i = base();
        i.physical_presence_days_in_12_month_period = 100;
        i.bona_fide_residence_satisfied = false;
        let r = compute(&i);
        assert_eq!(r.foreign_earned_income_excluded_dollars, 0);
        assert_eq!(r.housing_exclusion_dollars, 0);
        assert_eq!(r.total_section_911_exclusion_dollars, 0);
    }

    // ── 2025 vs 2026 inflation amounts ─────────────────────────────

    #[test]
    fn year_2025_feie_130000() {
        let mut i = base();
        i.tax_year = 2025;
        i.feie_inflation_adjusted_cap_dollars = 130_000;
        i.foreign_earned_income_dollars = 200_000;
        let r = compute(&i);
        assert_eq!(r.foreign_earned_income_excluded_dollars, 130_000);
        // 16% × $130k = $20,800.
        assert_eq!(r.base_housing_amount_dollars, 20_800);
        // 30% × $130k = $39,000.
        assert_eq!(r.housing_amount_cap_dollars, 39_000);
    }

    #[test]
    fn year_2026_feie_132900() {
        let r = compute(&base());
        assert_eq!(r.feie_amount_dollars, 132_900);
    }

    // ── Total §911 exclusion ────────────────────────────────────────

    #[test]
    fn total_exclusion_combines_income_and_housing() {
        let mut i = base();
        i.foreign_earned_income_dollars = 150_000;
        i.housing_expenses_dollars = 35_000;
        let r = compute(&i);
        assert_eq!(r.foreign_earned_income_excluded_dollars, 132_900);
        assert_eq!(r.housing_exclusion_dollars, 13_736);
        assert_eq!(r.total_section_911_exclusion_dollars, 132_900 + 13_736);
    }

    // ── Citation ───────────────────────────────────────────────────

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&base());
        assert!(r.citation.contains("§911(a)(1)"));
        assert!(r.citation.contains("§911(a)(2)"));
        assert!(r.citation.contains("§911(b)(1)"));
        assert!(r.citation.contains("§911(c)(2)"));
        assert!(r.citation.contains("§911(d)(1)(A)"));
        assert!(r.citation.contains("§911(d)(1)(B)"));
        assert!(r.citation.contains("§911(d)(2)(B)"));
        assert!(r.citation.contains("§911(d)(7)"));
        assert!(r.citation.contains("330 full days"));
        assert!(r.citation.contains("16%"));
        assert!(r.citation.contains("30%"));
    }

    #[test]
    fn citation_mentions_2025_and_2026_amounts() {
        let r = compute(&base());
        assert!(r.citation.contains("$130,000"));
        assert!(r.citation.contains("$132,900"));
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn note_qualifying_test_described() {
        let r = compute(&base());
        assert!(r.note.contains("§911(d)(1)(B) physical presence"));
    }

    #[test]
    fn note_bona_fide_path_described() {
        let mut i = base();
        i.bona_fide_residence_satisfied = true;
        let r = compute(&i);
        assert!(r.note.contains("§911(d)(1)(A) bona fide residence"));
    }

    #[test]
    fn note_us_government_bar_described() {
        let mut i = base();
        i.income_from_us_government = true;
        let r = compute(&i);
        assert!(r.note.contains("US-gov bar: TRIGGERED"));
    }

    // ── Precision ──────────────────────────────────────────────────

    #[test]
    fn very_high_income_precision() {
        let mut i = base();
        i.foreign_earned_income_dollars = 10_000_000;
        i.housing_expenses_dollars = 200_000;
        let r = compute(&i);
        // Income exclusion capped at $132,900.
        assert_eq!(r.foreign_earned_income_excluded_dollars, 132_900);
        // Housing exclusion capped at $39,870.
        assert_eq!(r.housing_exclusion_dollars, 39_870);
        // Total = $172,770.
        assert_eq!(r.total_section_911_exclusion_dollars, 172_770);
    }
}
