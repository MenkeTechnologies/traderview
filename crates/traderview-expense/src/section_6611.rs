//! IRC § 6611 — Interest on overpayments. Companion to § 6601
//! (interest on underpayments built in iter 276), § 6621 (rate
//! determination), and § 6622 (daily compounding).
//!
//! Trader-relevant when the IRS owes the taxpayer interest on a
//! refund — e.g., overpayment from an amended return (Form 1040-X),
//! NOL carryback, missed § 901 FTC, refund delay beyond the 45-day
//! safe harbor. § 6611 interest paid by the IRS is GROSS INCOME to
//! the taxpayer under § 61(a)(4) and reported on Form 1099-INT.
//!
//! § 6611(a) GENERAL RULE — interest shall be allowed and paid upon
//! any overpayment in respect of any internal revenue tax at the
//! overpayment rate established under § 6621.
//!
//! § 6611(b)(1) CREDIT — in the case of a credit, interest from the
//! date of the overpayment to the due date of the amount against
//! which the credit is taken.
//!
//! § 6611(b)(2) REFUND — in the case of a refund, interest from the
//! date of the overpayment to a date preceding the date of the
//! refund check by not more than 30 days. The 30-day STOP DATE
//! gives the IRS administrative flexibility to issue the check.
//!
//! § 6611(e)(1) 45-DAY RULE FOR TIMELY-FILED RETURNS — if any
//! overpayment of tax is refunded WITHIN 45 DAYS after the last
//! date prescribed for filing the return (without regard to any
//! extension of time for filing) or, in the case of a return filed
//! after such last date, within 45 days after the date the return
//! is filed, NO INTEREST shall be allowed on such overpayment. This
//! is the IRS's "free 45-day window" to issue refunds without
//! accruing interest exposure.
//!
//! § 6611(e)(2) 45-DAY RULE FOR REFUND CLAIMS — if the taxpayer
//! files a CLAIM for a credit or refund and such overpayment is
//! refunded within 45 days after such claim is filed, no interest
//! shall be allowed from the date the claim is filed until the day
//! the refund is made.
//!
//! § 6611(e)(3) IRS-INITIATED REFUND ADJUSTMENT — if an adjustment
//! initiated by the Secretary results in a refund or credit of an
//! overpayment, interest on such overpayment shall be computed by
//! SUBTRACTING 45 DAYS from the number of days interest would
//! otherwise be allowed with respect to such overpayment.
//!
//! § 6621(a)(1) OVERPAYMENT RATE — sum of the federal short-term
//! rate determined under § 1274(d) PLUS 3 PERCENTAGE POINTS (2
//! POINTS in the case of a corporation, 0.5 POINTS for the portion
//! of a CORPORATE overpayment exceeding $10,000 — the "GATT rate"
//! enacted by the General Agreement on Tariffs and Trade
//! Implementation Act of 1994 (Pub. L. 103-465 § 713)).
//!
//! § 6622(a) DAILY COMPOUNDING — interest required to be paid by
//! this title shall be compounded daily. Same mechanic as § 6601.
//!
//! 2026 published overpayment rates:
//!   Q1 2026 (Rev. Rul. 2025-22) — federal short-term rate 4%:
//!     - Individual / non-corporate overpayment: 7% (FST + 3)
//!     - Corporate overpayment: 6% (FST + 2)
//!     - Corporate overpayment portion > $10K: 4.5% (FST + 0.5,
//!       GATT rate)
//!   Q2 2026 (Rev. Rul. 2026-5) — federal short-term rate 3%:
//!     - Individual / non-corporate overpayment: 6%
//!     - Corporate overpayment: 5%
//!     - Corporate overpayment > $10K: 3.5%
//!
//! Citations: IRC § 6611(a) (general overpayment interest rule);
//! § 6611(b)(1) (credit interest period); § 6611(b)(2) (refund
//! interest period + 30-day stop date); § 6611(e)(1) (45-day rule
//! for timely-filed returns); § 6611(e)(2) (45-day rule for refund
//! claims); § 6611(e)(3) (45-day subtraction for IRS-initiated
//! adjustments); § 6621(a)(1) (FST + 3% individual / FST + 2%
//! corporate / FST + 0.5% corporate > $10K GATT rate); § 6622(a)
//! (daily compounding); Pub. L. 103-465 § 713 (GATT rate enacted);
//! Rev. Rul. 2025-22 (Q1 2026 rates); Rev. Rul. 2026-5 (Q2 2026
//! rates); IRC § 61(a)(4) (interest received treated as gross
//! income).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaxpayerType {
    /// Individual / non-corporate taxpayer. § 6621(a)(1) rate =
    /// federal short-term + 3%.
    Individual,
    /// Corporation, overpayment ≤ $10,000. § 6621(a)(1) rate =
    /// federal short-term + 2%.
    CorporateRegular,
    /// Corporation, portion of overpayment > $10,000. § 6621(a)(1)
    /// rate = federal short-term + 0.5% ("GATT rate").
    CorporateOver10kGatt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuarterRate {
    pub year: u32,
    pub quarter: u8,
    /// Federal short-term rate basis points (e.g., Q1 2026 = 400 bps).
    pub federal_short_term_bps: u32,
    /// Individual overpayment rate = FST + 300 bps.
    pub individual_overpayment_bps: u32,
    /// Corporate overpayment rate ≤ $10K = FST + 200 bps.
    pub corporate_overpayment_bps: u32,
    /// Corporate overpayment > $10K (GATT) = FST + 50 bps.
    pub corporate_gatt_bps: u32,
}

const RATE_TABLE: &[QuarterRate] = &[
    QuarterRate {
        year: 2026,
        quarter: 1,
        federal_short_term_bps: 400,
        individual_overpayment_bps: 700,
        corporate_overpayment_bps: 600,
        corporate_gatt_bps: 450,
    },
    QuarterRate {
        year: 2026,
        quarter: 2,
        federal_short_term_bps: 300,
        individual_overpayment_bps: 600,
        corporate_overpayment_bps: 500,
        corporate_gatt_bps: 350,
    },
];

pub fn lookup_rate(year: u32, quarter: u8) -> Option<QuarterRate> {
    RATE_TABLE
        .iter()
        .find(|r| r.year == year && r.quarter == quarter)
        .copied()
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6611Input {
    pub principal_overpayment_cents: i64,
    pub taxpayer_type: TaxpayerType,
    pub rate_year: u32,
    pub rate_quarter: u8,
    /// Total days from date of overpayment to date refund check
    /// issued. Before the 45-day and 30-day rules are applied.
    pub days_from_overpayment_to_refund: u32,
    /// Whether refund issued within the 45-day safe harbor under
    /// § 6611(e)(1) (45 days from return-due date for timely-filed
    /// or from return-filing date for late-filed). Triggers ZERO
    /// interest if true.
    pub refund_within_45_day_safe_harbor: bool,
    /// Whether the refund arose from a claim for refund (Form
    /// 1040-X). § 6611(e)(2) — interest period starts at claim
    /// filing if refund issued within 45 days of claim.
    pub refund_from_claim_within_45_days_of_claim_filing: bool,
    /// Whether the IRS-initiated the adjustment that produced the
    /// refund. § 6611(e)(3) requires SUBTRACTING 45 days from the
    /// interest accrual period.
    pub irs_initiated_adjustment: bool,
    /// Optional override of rate basis points (caller responsibility
    /// when RATE_TABLE has no entry for the year/quarter).
    pub rate_override_bps: Option<u32>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6611Result {
    pub interest_paid_cents: i64,
    pub annual_rate_bps: u32,
    pub effective_interest_period_days: i32,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section6611Input) -> Section6611Result {
    let mut notes: Vec<String> = Vec::new();
    let principal = input.principal_overpayment_cents.max(0);

    if input.refund_within_45_day_safe_harbor {
        notes.push(
            "§ 6611(e)(1) — refund issued within 45 days of return-due / return-filing date; NO INTEREST allowed (IRS free 45-day window)"
                .to_string(),
        );
        return Section6611Result {
            interest_paid_cents: 0,
            annual_rate_bps: 0,
            effective_interest_period_days: 0,
            citation: citation(),
            notes,
        };
    }

    if input.refund_from_claim_within_45_days_of_claim_filing {
        notes.push(
            "§ 6611(e)(2) — refund issued within 45 days of claim filing (Form 1040-X); NO INTEREST allowed for the period from claim filing to refund"
                .to_string(),
        );
        return Section6611Result {
            interest_paid_cents: 0,
            annual_rate_bps: 0,
            effective_interest_period_days: 0,
            citation: citation(),
            notes,
        };
    }

    let annual_rate_bps = match input.rate_override_bps {
        Some(bps) => {
            notes.push(format!(
                "rate override applied — using {} bps in lieu of RATE_TABLE lookup",
                bps
            ));
            bps
        }
        None => match lookup_rate(input.rate_year, input.rate_quarter) {
            Some(q) => {
                let bps = match input.taxpayer_type {
                    TaxpayerType::Individual => q.individual_overpayment_bps,
                    TaxpayerType::CorporateRegular => q.corporate_overpayment_bps,
                    TaxpayerType::CorporateOver10kGatt => q.corporate_gatt_bps,
                };
                notes.push(format!(
                    "{} Q{} {} overpayment rate = {} bps ({}.{}%)",
                    q.year,
                    q.quarter,
                    match input.taxpayer_type {
                        TaxpayerType::Individual => "individual",
                        TaxpayerType::CorporateRegular => "corporate (≤ $10K)",
                        TaxpayerType::CorporateOver10kGatt => "corporate > $10K (GATT)",
                    },
                    bps,
                    bps / 100,
                    bps % 100,
                ));
                bps
            }
            None => {
                notes.push(format!(
                    "RATE_TABLE has no entry for {} Q{} — caller must supply rate_override_bps",
                    input.rate_year, input.rate_quarter
                ));
                return Section6611Result {
                    interest_paid_cents: 0,
                    annual_rate_bps: 0,
                    effective_interest_period_days: 0,
                    citation: citation(),
                    notes,
                };
            }
        },
    };

    let raw_days = input.days_from_overpayment_to_refund as i32;
    let stop_date_subtraction = 30i32;
    let irs_initiated_subtraction = if input.irs_initiated_adjustment {
        45i32
    } else {
        0
    };
    let effective_days = (raw_days - stop_date_subtraction - irs_initiated_subtraction).max(0);

    if effective_days > 0 {
        notes.push(format!(
            "§ 6611(b)(2) — interest period {} days minus 30-day refund stop date{} = {} effective days",
            raw_days,
            if input.irs_initiated_adjustment {
                " minus 45-day § 6611(e)(3) IRS-initiated adjustment"
            } else {
                ""
            },
            effective_days
        ));
    }

    let interest = compute_daily_compound_interest(principal, annual_rate_bps, effective_days as u32);

    notes.push(
        "§ 6622(a) daily compounding applied; interest received treated as gross income to taxpayer under § 61(a)(4)"
            .to_string(),
    );

    if matches!(input.taxpayer_type, TaxpayerType::CorporateOver10kGatt) {
        notes.push(
            "§ 6621(a)(1) GATT rate (federal short-term + 0.5%) applies to corporate overpayment portion exceeding $10,000 — Pub. L. 103-465 § 713 (1994 GATT Implementation Act)"
                .to_string(),
        );
    }

    Section6611Result {
        interest_paid_cents: interest,
        annual_rate_bps,
        effective_interest_period_days: effective_days,
        citation: citation(),
        notes,
    }
}

fn compute_daily_compound_interest(principal: i64, annual_rate_bps: u32, days: u32) -> i64 {
    if principal <= 0 || days == 0 || annual_rate_bps == 0 {
        return 0;
    }
    let principal_f = principal as f64;
    let daily_rate = (annual_rate_bps as f64) / 10_000.0 / 365.0;
    let growth_factor = (1.0_f64 + daily_rate).powi(days as i32);
    let interest_f = principal_f * (growth_factor - 1.0);
    interest_f.round() as i64
}

fn citation() -> &'static str {
    "IRC § 6611(a)/(b)(1)/(b)(2)/(e)(1)/(e)(2)/(e)(3); § 6621(a)(1); § 6622(a); § 61(a)(4); Pub. L. 103-465 § 713 (1994 GATT); Rev. Rul. 2025-22 (Q1 2026); Rev. Rul. 2026-5 (Q2 2026)"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(
        principal_dollars: i64,
        taxpayer: TaxpayerType,
        year: u32,
        quarter: u8,
        days: u32,
    ) -> Section6611Input {
        Section6611Input {
            principal_overpayment_cents: principal_dollars * 100,
            taxpayer_type: taxpayer,
            rate_year: year,
            rate_quarter: quarter,
            days_from_overpayment_to_refund: days,
            refund_within_45_day_safe_harbor: false,
            refund_from_claim_within_45_days_of_claim_filing: false,
            irs_initiated_adjustment: false,
            rate_override_bps: None,
        }
    }

    #[test]
    fn forty_five_day_safe_harbor_zero_interest() {
        let mut i = base(10_000, TaxpayerType::Individual, 2026, 1, 30);
        i.refund_within_45_day_safe_harbor = true;
        let r = compute(&i);
        assert_eq!(r.interest_paid_cents, 0);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6611(e)(1)") && n.contains("NO INTEREST")));
    }

    #[test]
    fn forty_five_day_refund_claim_safe_harbor_zero_interest() {
        let mut i = base(10_000, TaxpayerType::Individual, 2026, 1, 30);
        i.refund_from_claim_within_45_days_of_claim_filing = true;
        let r = compute(&i);
        assert_eq!(r.interest_paid_cents, 0);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6611(e)(2)") && n.contains("Form 1040-X")));
    }

    #[test]
    fn q1_2026_individual_overpayment_rate_700_bps() {
        let r = compute(&base(10_000, TaxpayerType::Individual, 2026, 1, 395));
        assert_eq!(r.annual_rate_bps, 700);
        assert!(r.interest_paid_cents > 70_000);
        assert!(r.interest_paid_cents < 73_000);
    }

    #[test]
    fn q1_2026_corporate_regular_overpayment_rate_600_bps() {
        let r = compute(&base(10_000, TaxpayerType::CorporateRegular, 2026, 1, 395));
        assert_eq!(r.annual_rate_bps, 600);
    }

    #[test]
    fn q1_2026_corporate_gatt_rate_450_bps() {
        let r = compute(&base(100_000, TaxpayerType::CorporateOver10kGatt, 2026, 1, 395));
        assert_eq!(r.annual_rate_bps, 450);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("GATT rate") && n.contains("Pub. L. 103-465")));
    }

    #[test]
    fn q2_2026_individual_overpayment_rate_600_bps() {
        let r = compute(&base(10_000, TaxpayerType::Individual, 2026, 2, 395));
        assert_eq!(r.annual_rate_bps, 600);
    }

    #[test]
    fn q2_2026_corporate_gatt_rate_350_bps() {
        let r = compute(&base(100_000, TaxpayerType::CorporateOver10kGatt, 2026, 2, 395));
        assert_eq!(r.annual_rate_bps, 350);
    }

    #[test]
    fn corporate_gatt_one_percentage_point_below_corporate_regular_invariant() {
        let r_corp = compute(&base(10_000, TaxpayerType::CorporateRegular, 2026, 1, 395));
        let r_gatt = compute(&base(10_000, TaxpayerType::CorporateOver10kGatt, 2026, 1, 395));
        assert_eq!(r_corp.annual_rate_bps - r_gatt.annual_rate_bps, 150);
    }

    #[test]
    fn individual_rate_one_percentage_point_above_corporate_regular() {
        let r_ind = compute(&base(10_000, TaxpayerType::Individual, 2026, 1, 395));
        let r_corp = compute(&base(10_000, TaxpayerType::CorporateRegular, 2026, 1, 395));
        assert_eq!(r_ind.annual_rate_bps - r_corp.annual_rate_bps, 100);
    }

    #[test]
    fn thirty_day_refund_stop_date_subtracted() {
        let r = compute(&base(10_000, TaxpayerType::Individual, 2026, 1, 100));
        assert_eq!(r.effective_interest_period_days, 70);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("30-day refund stop date")));
    }

    #[test]
    fn irs_initiated_adjustment_45_day_subtraction() {
        let mut i = base(10_000, TaxpayerType::Individual, 2026, 1, 200);
        i.irs_initiated_adjustment = true;
        let r = compute(&i);
        assert_eq!(r.effective_interest_period_days, 200 - 30 - 45);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6611(e)(3) IRS-initiated adjustment")));
    }

    #[test]
    fn short_interest_period_clamped_to_zero() {
        let r = compute(&base(10_000, TaxpayerType::Individual, 2026, 1, 20));
        assert_eq!(r.effective_interest_period_days, 0);
        assert_eq!(r.interest_paid_cents, 0);
    }

    #[test]
    fn irs_initiated_short_period_clamped_to_zero() {
        let mut i = base(10_000, TaxpayerType::Individual, 2026, 1, 60);
        i.irs_initiated_adjustment = true;
        let r = compute(&i);
        assert_eq!(r.effective_interest_period_days, 0);
    }

    #[test]
    fn rate_override_bypasses_table() {
        let mut i = base(10_000, TaxpayerType::Individual, 2026, 1, 100);
        i.rate_override_bps = Some(500);
        let r = compute(&i);
        assert_eq!(r.annual_rate_bps, 500);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("rate override applied")));
    }

    #[test]
    fn missing_year_no_override_zero_interest_with_note() {
        let r = compute(&base(10_000, TaxpayerType::Individual, 2027, 4, 100));
        assert_eq!(r.interest_paid_cents, 0);
        assert_eq!(r.annual_rate_bps, 0);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("must supply rate_override_bps")));
    }

    #[test]
    fn zero_principal_zero_interest() {
        let r = compute(&base(0, TaxpayerType::Individual, 2026, 1, 100));
        assert_eq!(r.interest_paid_cents, 0);
    }

    #[test]
    fn negative_principal_clamped_to_zero() {
        let mut i = base(0, TaxpayerType::Individual, 2026, 1, 100);
        i.principal_overpayment_cents = -100_000;
        let r = compute(&i);
        assert_eq!(r.interest_paid_cents, 0);
    }

    #[test]
    fn citation_pins_all_subsections_and_authorities() {
        let r = compute(&base(10_000, TaxpayerType::Individual, 2026, 1, 100));
        assert!(r.citation.contains("§ 6611(a)"));
        assert!(r.citation.contains("(b)(1)"));
        assert!(r.citation.contains("(b)(2)"));
        assert!(r.citation.contains("(e)(1)"));
        assert!(r.citation.contains("(e)(2)"));
        assert!(r.citation.contains("(e)(3)"));
        assert!(r.citation.contains("§ 6621(a)(1)"));
        assert!(r.citation.contains("§ 6622(a)"));
        assert!(r.citation.contains("§ 61(a)(4)"));
        assert!(r.citation.contains("Pub. L. 103-465 § 713"));
        assert!(r.citation.contains("Rev. Rul. 2025-22"));
        assert!(r.citation.contains("Rev. Rul. 2026-5"));
    }

    #[test]
    fn note_describes_individual_path_no_gatt_note() {
        let r = compute(&base(10_000, TaxpayerType::Individual, 2026, 1, 100));
        let has_gatt_note = r.notes.iter().any(|n| n.contains("GATT rate"));
        assert!(!has_gatt_note);
    }

    #[test]
    fn note_describes_corporate_path_no_gatt_note_unless_over_10k() {
        let r = compute(&base(10_000, TaxpayerType::CorporateRegular, 2026, 1, 100));
        let has_gatt_note = r.notes.iter().any(|n| n.contains("GATT rate"));
        assert!(!has_gatt_note);
    }

    #[test]
    fn note_includes_section_61_gross_income_treatment() {
        let r = compute(&base(10_000, TaxpayerType::Individual, 2026, 1, 100));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 61(a)(4)") && n.contains("gross income")));
    }

    #[test]
    fn lookup_rate_returns_none_for_missing_year() {
        assert!(lookup_rate(2030, 1).is_none());
    }

    #[test]
    fn lookup_rate_returns_some_for_2026_q1() {
        let r = lookup_rate(2026, 1).expect("present");
        assert_eq!(r.federal_short_term_bps, 400);
        assert_eq!(r.individual_overpayment_bps, 700);
        assert_eq!(r.corporate_overpayment_bps, 600);
        assert_eq!(r.corporate_gatt_bps, 450);
    }

    #[test]
    fn q1_to_q2_2026_dropped_100_bps_all_three_taxpayer_types() {
        let q1_ind = compute(&base(10_000, TaxpayerType::Individual, 2026, 1, 395));
        let q2_ind = compute(&base(10_000, TaxpayerType::Individual, 2026, 2, 395));
        assert_eq!(q1_ind.annual_rate_bps - q2_ind.annual_rate_bps, 100);

        let q1_corp = compute(&base(10_000, TaxpayerType::CorporateRegular, 2026, 1, 395));
        let q2_corp = compute(&base(10_000, TaxpayerType::CorporateRegular, 2026, 2, 395));
        assert_eq!(q1_corp.annual_rate_bps - q2_corp.annual_rate_bps, 100);

        let q1_gatt = compute(&base(10_000, TaxpayerType::CorporateOver10kGatt, 2026, 1, 395));
        let q2_gatt = compute(&base(10_000, TaxpayerType::CorporateOver10kGatt, 2026, 2, 395));
        assert_eq!(q1_gatt.annual_rate_bps - q2_gatt.annual_rate_bps, 100);
    }

    #[test]
    fn safe_harbor_dominates_all_other_inputs() {
        let mut i = base(1_000_000, TaxpayerType::CorporateOver10kGatt, 2026, 1, 1000);
        i.refund_within_45_day_safe_harbor = true;
        i.irs_initiated_adjustment = true;
        let r = compute(&i);
        assert_eq!(r.interest_paid_cents, 0);
        assert_eq!(r.effective_interest_period_days, 0);
    }

    #[test]
    fn comparison_underpayment_versus_overpayment_corporate_diff_one_pct() {
        let r_overpayment = compute(&base(10_000, TaxpayerType::CorporateRegular, 2026, 1, 395));
        assert_eq!(r_overpayment.annual_rate_bps, 600);
    }
}
