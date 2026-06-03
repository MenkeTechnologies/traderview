//! IRC § 221 — Interest on education loans (student loan interest
//! deduction).
//!
//! § 221 provides an ABOVE-THE-LINE deduction of up to **$2,500** for
//! interest paid on qualified education loans. Available regardless of
//! whether the taxpayer itemizes. The $2,500 cap is STATUTORY and does
//! NOT inflation-adjust (§ 221(b)(1)).
//!
//! The deduction phases out by MAGI under § 221(b)(2) over a $15,000
//! window for unmarried filers / $30,000 window for joint filers.
//! Above the high end, deduction = $0. **Married filing separately
//! cannot claim § 221 at all** (§ 221(e)(2)).
//!
//! 2026 phaseout ranges (per IRS Rev. Proc. 2025-XX inflation
//! adjustments):
//!
//! Single / Head of Household / Qualifying Widow: **$85,000-$100,000**.
//!
//! Married filing jointly: **$175,000-$205,000**.
//!
//! Married filing separately: **NO deduction** ($0 cap).
//!
//! 2025 phaseout ranges (per Rev. Proc. 2024-XX):
//!
//! Single / HoH: $80,000-$95,000.
//!
//! MFJ: $165,000-$195,000.
//!
//! Citations: 26 U.S.C. § 221; § 221(a) (general deduction); § 221(b)(1)
//! ($2,500 statutory cap, not inflation-adjusted); § 221(b)(2) (MAGI
//! phaseout); § 221(e)(2) (MFS excluded); IRS Topic No. 456 (Student
//! loan interest deduction); IRS Pub. 970 (Tax Benefits for Education).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    HeadOfHousehold,
    MarriedFilingJointly,
    MarriedFilingSeparately,
    QualifyingWidow,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section221Input {
    pub year: u32,
    pub filing_status: FilingStatus,
    pub modified_agi_cents: i64,
    pub interest_paid_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section221Result {
    pub statutory_cap_cents: i64,
    pub mfs_excluded: bool,
    pub phaseout_low_cents: i64,
    pub phaseout_high_cents: i64,
    pub in_phaseout_range: bool,
    pub phaseout_reduction_cents: i64,
    pub allowed_deduction_cents: i64,
    pub citation: &'static str,
    pub note: String,
}

pub fn compute(input: &Section221Input) -> Section221Result {
    let interest = input.interest_paid_cents.max(0);
    let magi = input.modified_agi_cents.max(0);

    // MFS exclusion — § 221(e)(2).
    if matches!(input.filing_status, FilingStatus::MarriedFilingSeparately) {
        return Section221Result {
            statutory_cap_cents: 250000,
            mfs_excluded: true,
            phaseout_low_cents: 0,
            phaseout_high_cents: 0,
            in_phaseout_range: false,
            phaseout_reduction_cents: 0,
            allowed_deduction_cents: 0,
            citation: "26 U.S.C. § 221(e)(2) — Married Filing Separately taxpayers CANNOT claim the student loan interest deduction",
            note: "MFS filers excluded from § 221 entirely. Allowed deduction = $0.".to_string(),
        };
    }

    let (low, high) = phaseout_range(input.year, input.filing_status);
    let pre_phaseout = interest.min(250000);

    let (in_phaseout, allowed) = if magi <= low {
        (false, pre_phaseout)
    } else if magi >= high {
        (true, 0)
    } else {
        let range = high - low;
        let remaining = high - magi;
        let reduced = (pre_phaseout as i128 * remaining as i128 / range as i128) as i64;
        (true, reduced.max(0))
    };
    let phaseout_reduction = pre_phaseout - allowed;

    let note = format!(
        "Statutory cap = $2,500 (statutory, not inflation-adjusted). Interest paid = {} cents; capped tentative = {} cents. MAGI = {} cents. Phaseout range {}-{} cents ({:?} {}). {}{}Allowed deduction = {} cents.",
        interest,
        pre_phaseout,
        magi,
        low,
        high,
        input.filing_status,
        input.year,
        if in_phaseout { "IN-PHASEOUT — " } else { "" },
        if in_phaseout && allowed == 0 { "fully phased out. " } else { "" },
        allowed,
    );

    Section221Result {
        statutory_cap_cents: 250000,
        mfs_excluded: false,
        phaseout_low_cents: low,
        phaseout_high_cents: high,
        in_phaseout_range: in_phaseout,
        phaseout_reduction_cents: phaseout_reduction,
        allowed_deduction_cents: allowed,
        citation:
            "26 U.S.C. § 221(a)/(b)(1)/(b)(2) — $2,500 above-the-line deduction phased out by MAGI; § 221(e)(2) excludes MFS",
        note,
    }
}

fn phaseout_range(year: u32, fs: FilingStatus) -> (i64, i64) {
    match (year, fs) {
        // 2026 inflation-adjusted.
        (2026, FilingStatus::MarriedFilingJointly | FilingStatus::QualifyingWidow) => {
            (17500000, 20500000)
        }
        (2026, _) => (8500000, 10000000),
        // 2025.
        (2025, FilingStatus::MarriedFilingJointly | FilingStatus::QualifyingWidow) => {
            (16500000, 19500000)
        }
        (2025, _) => (8000000, 9500000),
        // Default uses 2026.
        (_, FilingStatus::MarriedFilingJointly | FilingStatus::QualifyingWidow) => {
            (17500000, 20500000)
        }
        (_, _) => (8500000, 10000000),
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    fn input(
        year: u32,
        fs: FilingStatus,
        magi: i64,
        interest: i64,
    ) -> Section221Input {
        Section221Input {
            year,
            filing_status: fs,
            modified_agi_cents: magi,
            interest_paid_cents: interest,
        }
    }

    #[test]
    fn single_2026_under_phaseout_full_2500_deduction() {
        let r = compute(&input(2026, FilingStatus::Single, 50_000_00, 3_000_00));
        assert_eq!(r.allowed_deduction_cents, 2_500_00);
        assert!(!r.in_phaseout_range);
        assert!(!r.mfs_excluded);
    }

    #[test]
    fn single_2026_at_85k_boundary_full() {
        let r = compute(&input(2026, FilingStatus::Single, 85_000_00, 3_000_00));
        assert!(!r.in_phaseout_range);
        assert_eq!(r.allowed_deduction_cents, 2_500_00);
    }

    #[test]
    fn single_2026_at_100k_boundary_zero() {
        let r = compute(&input(2026, FilingStatus::Single, 100_000_00, 3_000_00));
        assert!(r.in_phaseout_range);
        assert_eq!(r.allowed_deduction_cents, 0);
    }

    #[test]
    fn single_2026_in_phaseout_midpoint() {
        // $92.5K midpoint of $85K-$100K → 50% allowed = $1,250.
        let r = compute(&input(2026, FilingStatus::Single, 92_500_00, 3_000_00));
        assert!(r.in_phaseout_range);
        assert_eq!(r.allowed_deduction_cents, 1_250_00);
    }

    #[test]
    fn single_2026_interest_below_cap_no_phaseout() {
        // Only paid $1,500 → that's the limit, not $2,500.
        let r = compute(&input(2026, FilingStatus::Single, 50_000_00, 1_500_00));
        assert_eq!(r.allowed_deduction_cents, 1_500_00);
    }

    #[test]
    fn mfj_2026_under_175k_full_deduction() {
        let r = compute(&input(
            2026,
            FilingStatus::MarriedFilingJointly,
            150_000_00,
            3_000_00,
        ));
        assert_eq!(r.allowed_deduction_cents, 2_500_00);
        assert_eq!(r.phaseout_low_cents, 175_000_00);
        assert_eq!(r.phaseout_high_cents, 205_000_00);
    }

    #[test]
    fn mfj_2026_at_205k_high_end_zero() {
        let r = compute(&input(
            2026,
            FilingStatus::MarriedFilingJointly,
            205_000_00,
            3_000_00,
        ));
        assert_eq!(r.allowed_deduction_cents, 0);
    }

    #[test]
    fn mfj_2026_in_phaseout_midpoint() {
        // Midpoint $190K → 50% × $2,500 = $1,250.
        let r = compute(&input(
            2026,
            FilingStatus::MarriedFilingJointly,
            190_000_00,
            3_000_00,
        ));
        assert!(r.in_phaseout_range);
        assert_eq!(r.allowed_deduction_cents, 1_250_00);
    }

    #[test]
    fn mfs_excluded_zero_deduction() {
        // MFS cannot claim § 221 regardless of income or interest.
        let r = compute(&input(
            2026,
            FilingStatus::MarriedFilingSeparately,
            50_000_00,
            5_000_00,
        ));
        assert!(r.mfs_excluded);
        assert_eq!(r.allowed_deduction_cents, 0);
        assert!(r.citation.contains("§ 221(e)(2)"));
    }

    #[test]
    fn hoh_uses_single_phaseout_range() {
        let r = compute(&input(
            2026,
            FilingStatus::HeadOfHousehold,
            85_000_00,
            3_000_00,
        ));
        assert_eq!(r.phaseout_low_cents, 85_000_00);
        assert_eq!(r.phaseout_high_cents, 100_000_00);
    }

    #[test]
    fn qualifying_widow_uses_mfj_phaseout() {
        let r = compute(&input(
            2026,
            FilingStatus::QualifyingWidow,
            190_000_00,
            3_000_00,
        ));
        assert_eq!(r.phaseout_low_cents, 175_000_00);
        assert_eq!(r.phaseout_high_cents, 205_000_00);
    }

    #[test]
    fn year_aware_2025_single_80k_95k() {
        let r = compute(&input(2025, FilingStatus::Single, 85_000_00, 3_000_00));
        assert_eq!(r.phaseout_low_cents, 80_000_00);
        assert_eq!(r.phaseout_high_cents, 95_000_00);
    }

    #[test]
    fn year_aware_2025_mfj_165k_195k() {
        let r = compute(&input(
            2025,
            FilingStatus::MarriedFilingJointly,
            180_000_00,
            3_000_00,
        ));
        assert_eq!(r.phaseout_low_cents, 165_000_00);
        assert_eq!(r.phaseout_high_cents, 195_000_00);
    }

    #[test]
    fn statutory_2500_cap_does_NOT_inflation_adjust() {
        // Cap is $2,500 in both 2025 + 2026 (and forever absent
        // Congressional amendment).
        let r2025 = compute(&input(2025, FilingStatus::Single, 50_000_00, 5_000_00));
        let r2026 = compute(&input(2026, FilingStatus::Single, 50_000_00, 5_000_00));
        assert_eq!(r2025.allowed_deduction_cents, 2_500_00);
        assert_eq!(r2026.allowed_deduction_cents, 2_500_00);
        assert_eq!(r2025.statutory_cap_cents, 2_500_00);
        assert_eq!(r2026.statutory_cap_cents, 2_500_00);
    }

    #[test]
    fn negative_inputs_clamped() {
        let r = compute(&input(2026, FilingStatus::Single, -100, -100));
        assert_eq!(r.allowed_deduction_cents, 0);
    }

    #[test]
    fn zero_interest_zero_deduction() {
        let r = compute(&input(2026, FilingStatus::Single, 50_000_00, 0));
        assert_eq!(r.allowed_deduction_cents, 0);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let r = compute(&input(2026, FilingStatus::Single, 50_000_00, 3_000_00));
        assert!(r.citation.contains("§ 221(a)"));
        assert!(r.citation.contains("(b)(1)"));
        assert!(r.citation.contains("(b)(2)"));
        assert!(r.citation.contains("§ 221(e)(2)"));

        let mfs = compute(&input(
            2026,
            FilingStatus::MarriedFilingSeparately,
            50_000_00,
            3_000_00,
        ));
        assert!(mfs.citation.contains("§ 221(e)(2)"));
    }

    #[test]
    fn one_dollar_above_phaseout_low_starts_phaseout() {
        let r = compute(&input(
            2026,
            FilingStatus::Single,
            85_000_00 + 1,
            3_000_00,
        ));
        assert!(r.in_phaseout_range);
        assert!(r.allowed_deduction_cents < 2_500_00);
    }

    #[test]
    fn one_dollar_above_phaseout_high_no_deduction() {
        // 100K+1 → above high end → zero deduction.
        let r = compute(&input(
            2026,
            FilingStatus::Single,
            100_000_00 + 1,
            3_000_00,
        ));
        assert!(r.in_phaseout_range);
        assert_eq!(r.allowed_deduction_cents, 0);
    }

    #[test]
    fn interest_below_cap_phaseout_proportional() {
        // Interest $1,000 (under $2,500 cap). At $92.5K midpoint → 50%
        // × $1,000 = $500.
        let r = compute(&input(2026, FilingStatus::Single, 92_500_00, 1_000_00));
        assert_eq!(r.allowed_deduction_cents, 500_00);
    }

    #[test]
    fn worked_example_single_high_income_phased_out() {
        // Single, $98K MAGI ($85K-$100K range). Remaining = $2K of $15K
        // window. $2,500 × (2/15) = $333.33 → rounded.
        let r = compute(&input(2026, FilingStatus::Single, 98_000_00, 3_000_00));
        assert!(r.in_phaseout_range);
        // Expected: 2500_00 × 2_000_00 / 15_000_00 = 333_33 cents.
        let expected = (2_500_00_i128 * 2_000_00 / 15_000_00) as i64;
        assert_eq!(r.allowed_deduction_cents, expected);
    }
}
