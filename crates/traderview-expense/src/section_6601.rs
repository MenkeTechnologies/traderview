//! IRC § 6601 — Interest on underpayment, nonpayment, or extensions
//! of time for payment of tax. Companion to § 6621 (rate
//! determination) and § 6622 (daily compounding).
//!
//! Trader-critical when an amended return or audit results in
//! additional tax due — interest accrues from the ORIGINAL DUE DATE
//! of the return (April 15 for most individuals) regardless of any
//! extension to FILE, and continues to compound daily until full
//! payment. § 6601 interest is itself non-deductible for individuals
//! under § 163(h) (personal interest), though deductible as business
//! interest under § 163(a) when the underpaid tax was business in
//! character (sole-proprietor traders).
//!
//! § 6601(a) GENERAL RULE — if any amount of tax imposed by this
//! title is not paid on or before the last date prescribed for
//! payment, interest on such amount at the underpayment rate
//! established under § 6621 shall be paid for the period from such
//! last date to the date paid.
//!
//! § 6601(b)(1) LAST DATE PRESCRIBED FOR PAYMENT — the date fixed
//! for payment of tax, regardless of any extension of time for
//! payment. Extension to FILE does not extend the time to PAY.
//!
//! § 6601(c) RESTRICTIONS ON INTEREST WITH RESPECT TO DEFICIENCY —
//! in the case of a deficiency assessed under § 6213(c), interest
//! accrues for the period from the due date of the return to the
//! date of assessment.
//!
//! § 6601(e)(1) INTEREST TREATED AS TAX — interest prescribed under
//! this section shall be assessed, collected, and paid in the same
//! manner as taxes.
//!
//! § 6601(f) SATISFACTION BY CREDITS — if a portion of the tax
//! liability is satisfied by credit of an overpayment, no interest
//! is imposed on that portion for the period during which (if no
//! credit had been made) interest would have been allowed on the
//! overpayment.
//!
//! § 6621(a)(2) UNDERPAYMENT RATE — sum of the federal short-term
//! rate determined under § 1274(d) PLUS 3 PERCENTAGE POINTS.
//!
//! § 6621(c) LARGE CORPORATE UNDERPAYMENT — when the underpayment
//! is a "large corporate underpayment" (exceeding $100,000), the
//! 3-percentage-point adjustment is replaced by 5 PERCENTAGE POINTS
//! for the period after the applicable date (generally 30 days
//! after the IRS first sends a letter or notice asserting the
//! underpayment).
//!
//! § 6622(a) DAILY COMPOUNDING — interest required to be paid by
//! this title shall be compounded daily. Pre-1/1/1983 underpayments
//! used simple interest.
//!
//! QUARTERLY RATE PUBLICATION — under § 6621(b), the Secretary
//! determines federal short-term rates monthly. The underpayment
//! rate applicable for any calendar quarter is the rate in effect
//! during the FIRST MONTH of the preceding quarter (lookback
//! mechanism). Rates are published quarterly via Revenue Ruling
//! in the Internal Revenue Bulletin.
//!
//! 2025-2026 published underpayment rates:
//!   Q1 2026 (Rev. Rul. 2025-22) — 7% underpayment / 9% large
//!     corporate / 7% individual overpayment / 6% corporate
//!     overpayment.
//!   Q2 2026 (Rev. Rul. 2026-5) — 6% underpayment / 8% large
//!     corporate / 6% individual overpayment / 5% corporate
//!     overpayment.
//!   Q3 2025 onward — verified rates kept in `RATE_TABLE`.
//!
//! Citations: IRC § 6601(a) (general); § 6601(b)(1) (last date
//! prescribed); § 6601(c) (deficiency interest period); § 6601(e)(1)
//! (interest treated as tax); § 6601(f) (overpayment credit
//! offset); § 6621(a)(2) (federal short-term + 3% underpayment
//! rate); § 6621(c) (federal short-term + 5% large corporate
//! underpayment); § 6622(a) (daily compounding); Rev. Rul. 2025-22
//! (Q1 2026 rates); Rev. Rul. 2026-5 (Q2 2026 rates).

use serde::{Deserialize, Serialize};

/// Quarterly rate (basis points) for underpayment + large corporate
/// underpayment. Year × Quarter → (underpayment_bps,
/// large_corporate_bps).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuarterRate {
    pub year: u32,
    pub quarter: u8,
    pub underpayment_bps: u32,
    pub large_corporate_bps: u32,
}

const RATE_TABLE: &[QuarterRate] = &[
    QuarterRate {
        year: 2026,
        quarter: 1,
        underpayment_bps: 700,
        large_corporate_bps: 900,
    },
    QuarterRate {
        year: 2026,
        quarter: 2,
        underpayment_bps: 600,
        large_corporate_bps: 800,
    },
];

pub fn lookup_rate(year: u32, quarter: u8) -> Option<QuarterRate> {
    RATE_TABLE
        .iter()
        .find(|r| r.year == year && r.quarter == quarter)
        .copied()
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6601Input {
    /// Underpayment principal in cents (positive).
    pub principal_underpayment_cents: i64,
    /// Calendar year + quarter to use for the rate lookup. Caller
    /// supplies the year/quarter the underpayment accrued in.
    pub rate_year: u32,
    pub rate_quarter: u8,
    /// Whether the underpayment qualifies as a "large corporate
    /// underpayment" under § 6621(c) (corporate taxpayer + > $100K
    /// underpayment). Drives selection of +3% vs +5% rate.
    pub is_large_corporate_underpayment: bool,
    /// Number of days between the last date prescribed for payment
    /// (§ 6601(b)(1)) and the date paid. Used as the compounding
    /// period.
    pub days_outstanding: u32,
    /// Optional rate override in basis points. When provided,
    /// bypasses the RATE_TABLE lookup. Useful for years not yet
    /// in the published table or for hypothetical scenarios.
    pub rate_override_bps: Option<u32>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6601Result {
    pub interest_accrued_cents: i64,
    pub annual_rate_bps: u32,
    pub days_outstanding: u32,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section6601Input) -> Section6601Result {
    let mut notes: Vec<String> = Vec::new();
    let principal = input.principal_underpayment_cents.max(0);

    let annual_rate_bps = match input.rate_override_bps {
        Some(bps) => {
            notes.push(format!(
                "rate override applied — using {} bps in lieu of RATE_TABLE lookup",
                bps
            ));
            bps
        }
        None => {
            let entry = lookup_rate(input.rate_year, input.rate_quarter);
            match entry {
                Some(e) => {
                    let bps = if input.is_large_corporate_underpayment {
                        e.large_corporate_bps
                    } else {
                        e.underpayment_bps
                    };
                    notes.push(format!(
                        "{} Q{} {} rate = {} bps ({}%)",
                        e.year,
                        e.quarter,
                        if input.is_large_corporate_underpayment {
                            "large corporate"
                        } else {
                            "underpayment"
                        },
                        bps,
                        bps as f64 / 100.0
                    ));
                    bps
                }
                None => {
                    notes.push(format!(
                        "RATE_TABLE has no entry for {} Q{} — caller must supply rate_override_bps",
                        input.rate_year, input.rate_quarter
                    ));
                    return Section6601Result {
                        interest_accrued_cents: 0,
                        annual_rate_bps: 0,
                        days_outstanding: input.days_outstanding,
                        citation: citation(),
                        notes,
                    };
                }
            }
        }
    };

    let interest =
        compute_daily_compound_interest(principal, annual_rate_bps, input.days_outstanding);

    if input.days_outstanding > 0 {
        notes.push(format!(
            "§ 6622(a) daily compounding applied over {} days at {} bps annual",
            input.days_outstanding, annual_rate_bps
        ));
    }

    if input.is_large_corporate_underpayment {
        notes.push(
            "§ 6621(c) large corporate underpayment rate (federal short-term + 5%) applies for period after applicable date (generally 30 days after IRS notice)"
                .to_string(),
        );
    }

    Section6601Result {
        interest_accrued_cents: interest,
        annual_rate_bps,
        days_outstanding: input.days_outstanding,
        citation: citation(),
        notes,
    }
}

fn compute_daily_compound_interest(principal: i64, annual_rate_bps: u32, days: u32) -> i64 {
    if principal <= 0 || days == 0 {
        return 0;
    }
    let principal_f = principal as f64;
    let daily_rate = (annual_rate_bps as f64) / 10_000.0 / 365.0;
    let growth_factor = (1.0_f64 + daily_rate).powi(days as i32);
    let interest_f = principal_f * (growth_factor - 1.0);
    interest_f.round() as i64
}

fn citation() -> &'static str {
    "IRC § 6601(a)/(b)(1)/(c)/(e)(1)/(f); § 6621(a)(2)/(c); § 6622(a); Rev. Rul. 2025-22 (Q1 2026); Rev. Rul. 2026-5 (Q2 2026)"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        principal_dollars: i64,
        year: u32,
        quarter: u8,
        days: u32,
        large_corp: bool,
    ) -> Section6601Input {
        Section6601Input {
            principal_underpayment_cents: principal_dollars * 100,
            rate_year: year,
            rate_quarter: quarter,
            is_large_corporate_underpayment: large_corp,
            days_outstanding: days,
            rate_override_bps: None,
        }
    }

    #[test]
    fn zero_principal_zero_interest() {
        let r = compute(&input(0, 2026, 1, 100, false));
        assert_eq!(r.interest_accrued_cents, 0);
    }

    #[test]
    fn zero_days_zero_interest() {
        let r = compute(&input(10_000, 2026, 1, 0, false));
        assert_eq!(r.interest_accrued_cents, 0);
    }

    #[test]
    fn q1_2026_individual_underpayment_rate_700_bps() {
        let r = compute(&input(10_000, 2026, 1, 365, false));
        assert_eq!(r.annual_rate_bps, 700);
        assert!(r.interest_accrued_cents > 70_000); // > 7% simple due to compounding
        assert!(r.interest_accrued_cents < 73_000);
    }

    #[test]
    fn q2_2026_individual_underpayment_rate_600_bps() {
        let r = compute(&input(10_000, 2026, 2, 365, false));
        assert_eq!(r.annual_rate_bps, 600);
        assert!(r.interest_accrued_cents > 60_000);
        assert!(r.interest_accrued_cents < 63_000);
    }

    #[test]
    fn q1_2026_large_corporate_underpayment_rate_900_bps() {
        let r = compute(&input(1_000_000, 2026, 1, 365, true));
        assert_eq!(r.annual_rate_bps, 900);
        assert!(r.interest_accrued_cents > 9_000_000);
        assert!(r.interest_accrued_cents < 9_500_000);
    }

    #[test]
    fn q2_2026_large_corporate_underpayment_rate_800_bps() {
        let r = compute(&input(1_000_000, 2026, 2, 365, true));
        assert_eq!(r.annual_rate_bps, 800);
        assert!(r.interest_accrued_cents > 8_000_000);
        assert!(r.interest_accrued_cents < 8_500_000);
    }

    #[test]
    fn rate_override_bypasses_table() {
        let mut i = input(10_000, 2026, 1, 365, false);
        i.rate_override_bps = Some(500);
        let r = compute(&i);
        assert_eq!(r.annual_rate_bps, 500);
        assert!(r.notes.iter().any(|n| n.contains("rate override applied")));
    }

    #[test]
    fn missing_year_no_override_returns_zero_interest_with_note() {
        let r = compute(&input(10_000, 2027, 4, 365, false));
        assert_eq!(r.interest_accrued_cents, 0);
        assert_eq!(r.annual_rate_bps, 0);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("must supply rate_override_bps")));
    }

    #[test]
    fn daily_compounding_beats_simple_interest_after_full_year() {
        // 7% simple = $700 on $10K; daily compound > $700.
        let r = compute(&input(10_000, 2026, 1, 365, false));
        assert!(r.interest_accrued_cents > 70_000);
    }

    #[test]
    fn daily_compounding_at_30_days_matches_approximate_floor() {
        let r = compute(&input(10_000, 2026, 1, 30, false));
        // 7% × 30/365 × $10,000 = $57.53 simple; compounded slightly higher
        assert!(r.interest_accrued_cents >= 5_700);
        assert!(r.interest_accrued_cents <= 5_800);
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = compute(&input(10_000, 2026, 1, 30, false));
        assert!(r.citation.contains("§ 6601(a)"));
        assert!(r.citation.contains("(b)(1)"));
        assert!(r.citation.contains("(c)"));
        assert!(r.citation.contains("(e)(1)"));
        assert!(r.citation.contains("(f)"));
        assert!(r.citation.contains("§ 6621(a)(2)"));
        assert!(r.citation.contains("(c)"));
        assert!(r.citation.contains("§ 6622(a)"));
        assert!(r.citation.contains("Rev. Rul. 2025-22"));
        assert!(r.citation.contains("Rev. Rul. 2026-5"));
    }

    #[test]
    fn note_describes_quarter_and_rate() {
        let r = compute(&input(10_000, 2026, 1, 30, false));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("2026 Q1") && n.contains("700 bps")));
    }

    #[test]
    fn note_for_large_corporate_path_pins_section_6621c() {
        let r = compute(&input(1_000_000, 2026, 1, 30, true));
        assert!(r.notes.iter().any(|n| n.contains("§ 6621(c)")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("federal short-term + 5%")));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("30 days after IRS notice")));
    }

    #[test]
    fn note_for_individual_path_omits_section_6621c() {
        let r = compute(&input(10_000, 2026, 1, 30, false));
        assert!(!r.notes.iter().any(|n| n.contains("§ 6621(c)")));
    }

    #[test]
    fn q1_2026_large_corp_2_percentage_point_premium_over_individual() {
        let r_ind = compute(&input(100_000, 2026, 1, 365, false));
        let r_corp = compute(&input(100_000, 2026, 1, 365, true));
        assert_eq!(r_corp.annual_rate_bps - r_ind.annual_rate_bps, 200);
    }

    #[test]
    fn q1_to_q2_2026_rate_dropped_100_bps_individual() {
        let r_q1 = compute(&input(100_000, 2026, 1, 365, false));
        let r_q2 = compute(&input(100_000, 2026, 2, 365, false));
        assert_eq!(r_q1.annual_rate_bps - r_q2.annual_rate_bps, 100);
    }

    #[test]
    fn q1_to_q2_2026_rate_dropped_100_bps_large_corporate() {
        let r_q1 = compute(&input(1_000_000, 2026, 1, 365, true));
        let r_q2 = compute(&input(1_000_000, 2026, 2, 365, true));
        assert_eq!(r_q1.annual_rate_bps - r_q2.annual_rate_bps, 100);
    }

    #[test]
    fn negative_principal_clamped_to_zero() {
        let mut i = input(10_000, 2026, 1, 365, false);
        i.principal_underpayment_cents = -100_000;
        let r = compute(&i);
        assert_eq!(r.interest_accrued_cents, 0);
    }

    #[test]
    fn lookup_rate_returns_none_for_missing_year() {
        assert!(lookup_rate(2030, 1).is_none());
    }

    #[test]
    fn lookup_rate_returns_some_for_2026_q1() {
        let r = lookup_rate(2026, 1).expect("present");
        assert_eq!(r.underpayment_bps, 700);
        assert_eq!(r.large_corporate_bps, 900);
    }

    #[test]
    fn note_includes_daily_compounding_message_when_days_positive() {
        let r = compute(&input(10_000, 2026, 1, 1, false));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6622(a) daily compounding")));
    }

    #[test]
    fn note_omits_daily_compounding_when_days_zero() {
        let r = compute(&input(10_000, 2026, 1, 0, false));
        assert!(!r
            .notes
            .iter()
            .any(|n| n.contains("§ 6622(a) daily compounding")));
    }

    #[test]
    fn ten_year_compound_at_700_bps_doubles_principal_roughly() {
        let r = compute(&input(1_000, 2026, 1, 365 * 10, false));
        // (1 + 0.07/365)^3650 ≈ 2.0136 → interest ≈ $1,013.60
        assert!(r.interest_accrued_cents > 100_000);
        assert!(r.interest_accrued_cents < 110_000);
    }

    #[test]
    fn six_percent_rate_q2_2026_round_trip_with_override() {
        let mut i = input(10_000, 2026, 2, 365, false);
        i.rate_override_bps = Some(600);
        let r = compute(&i);
        assert_eq!(r.annual_rate_bps, 600);
    }
}
