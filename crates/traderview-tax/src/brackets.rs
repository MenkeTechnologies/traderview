//! 2025 federal ordinary-income tax brackets per filing status.
//!
//! Source: IRS Rev. Proc. 2024-40, § 3.01 (2025 inflation adjustments).
//! Mirror: <https://www.irs.gov/pub/irs-drop/rp-24-40.pdf>
//!
//! Brackets are stored as `(upper_inclusive_taxable_income, marginal_rate)`.
//! `upper_inclusive_taxable_income = None` is the open-ended top bracket.
//! Rates are basis points (e.g. 1000 = 10%) to keep integer-safe math.

use rust_decimal::Decimal;
use crate::engine::FilingStatus;

/// One bracket row.
#[derive(Debug, Clone, Copy)]
pub struct Bracket {
    /// Taxable income strictly less than or equal to this is in this
    /// bracket. `None` means the open-ended top tier.
    pub upper: Option<i64>,
    /// Marginal rate in basis points. 1200 = 12.0%.
    pub rate_bps: u32,
}

const fn b(upper: i64, rate_bps: u32) -> Bracket {
    Bracket { upper: Some(upper), rate_bps }
}
const fn top(rate_bps: u32) -> Bracket {
    Bracket { upper: None, rate_bps }
}

// 2025 brackets per Rev. Proc. 2024-40 § 3.01 Tables 1-4.

/// Single filer.
pub const SINGLE: &[Bracket] = &[
    b(11_925,   1000),
    b(48_475,   1200),
    b(103_350,  2200),
    b(197_300,  2400),
    b(250_525,  3200),
    b(626_350,  3500),
    top(3700),
];

/// Married filing jointly + qualifying surviving spouse.
pub const MFJ: &[Bracket] = &[
    b(23_850,   1000),
    b(96_950,   1200),
    b(206_700,  2200),
    b(394_600,  2400),
    b(501_050,  3200),
    b(751_600,  3500),
    top(3700),
];

/// Married filing separately.
pub const MFS: &[Bracket] = &[
    b(11_925,   1000),
    b(48_475,   1200),
    b(103_350,  2200),
    b(197_300,  2400),
    b(250_525,  3200),
    b(375_800,  3500),
    top(3700),
];

/// Head of household.
pub const HOH: &[Bracket] = &[
    b(17_000,   1000),
    b(64_850,   1200),
    b(103_350,  2200),
    b(197_300,  2400),
    b(250_500,  3200),
    b(626_350,  3500),
    top(3700),
];

/// 2025 standard deduction per filing status. Rev. Proc. 2024-40 § 3.16.
pub fn standard_deduction(status: FilingStatus) -> Decimal {
    let n: i64 = match status {
        FilingStatus::Single | FilingStatus::Mfs => 15_000,
        FilingStatus::Mfj                        => 30_000,
        FilingStatus::Hoh                        => 22_500,
    };
    Decimal::from(n)
}

/// Tax on `taxable_income` using the bracket table for `status`. Returns
/// the dollars-and-cents tax owed before credits.
///
/// Negative or zero input ⇒ zero tax (no negative-tax). The arithmetic
/// is integer-safe (basis points × dollar amount, divided by 10_000)
/// so there's no float rounding drift at the bracket boundaries.
pub fn ordinary_income_tax(taxable_income: Decimal, status: FilingStatus) -> Decimal {
    if taxable_income <= Decimal::ZERO {
        return Decimal::ZERO;
    }
    let table = brackets_for(status);

    // Walk brackets bottom-up. For each fully-spanned bracket, add
    // (bracket_width × marginal_rate). For the bracket containing the
    // income, add (income_in_bracket × marginal_rate). Stop.
    let mut tax = Decimal::ZERO;
    let mut prev_upper: i64 = 0;
    for b in table {
        let rate = Decimal::from(b.rate_bps);
        match b.upper {
            Some(u) => {
                let upper_d = Decimal::from(u);
                if taxable_income > upper_d {
                    // Whole bracket is spanned.
                    let width = upper_d - Decimal::from(prev_upper);
                    tax += (width * rate) / Decimal::from(10_000);
                    prev_upper = u;
                } else {
                    // Income falls in this bracket.
                    let in_bracket = taxable_income - Decimal::from(prev_upper);
                    tax += (in_bracket * rate) / Decimal::from(10_000);
                    return tax;
                }
            }
            None => {
                // Open top bracket. Everything above prev_upper is here.
                let in_bracket = taxable_income - Decimal::from(prev_upper);
                tax += (in_bracket * rate) / Decimal::from(10_000);
                return tax;
            }
        }
    }
    tax
}

fn brackets_for(status: FilingStatus) -> &'static [Bracket] {
    match status {
        FilingStatus::Single => SINGLE,
        FilingStatus::Mfj    => MFJ,
        FilingStatus::Mfs    => MFS,
        FilingStatus::Hoh    => HOH,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── Spot checks against the IRS 2025 brackets ──────────────────────

    #[test]
    fn single_zero_income_no_tax() {
        assert_eq!(ordinary_income_tax(Decimal::ZERO, FilingStatus::Single), Decimal::ZERO);
    }

    #[test]
    fn single_first_bracket_only() {
        // $10,000 — well within 10% bracket. Tax = $1,000.
        let t = ordinary_income_tax(Decimal::from(10_000), FilingStatus::Single);
        assert_eq!(t, Decimal::from(1_000));
    }

    #[test]
    fn single_at_first_bracket_boundary() {
        // $11,925 — exactly the top of the 10% bracket. Tax = $1,192.50.
        let t = ordinary_income_tax(Decimal::from(11_925), FilingStatus::Single);
        assert_eq!(t, "1192.5".parse::<Decimal>().unwrap());
    }

    #[test]
    fn single_into_12_pct_bracket() {
        // $20,000 — $11,925 @ 10% + ($20,000 - $11,925) @ 12%
        //        = 1192.50 + 969.00 = 2161.50
        let t = ordinary_income_tax(Decimal::from(20_000), FilingStatus::Single);
        assert_eq!(t, "2161.5".parse::<Decimal>().unwrap());
    }

    #[test]
    fn mfj_50k_income() {
        // $50,000 MFJ:
        //   $23,850 @ 10%       = 2,385.00
        //   $26,150 @ 12% (rest) = 3,138.00
        //                  total = 5,523.00
        let t = ordinary_income_tax(Decimal::from(50_000), FilingStatus::Mfj);
        assert_eq!(t, Decimal::from(5_523));
    }

    #[test]
    fn single_top_bracket_37_pct() {
        // $700,000 single — straddles into top 37% bracket ($626,350+).
        // Hand-computed:
        //   11,925  @ 10% =     1,192.50
        //   36,550  @ 12% =     4,386.00   (48,475-11,925)
        //   54,875  @ 22% =    12,072.50   (103,350-48,475)
        //   93,950  @ 24% =    22,548.00   (197,300-103,350)
        //   53,225  @ 32% =    17,032.00   (250,525-197,300)
        //  375,825  @ 35% =   131,538.75   (626,350-250,525)
        //   73,650  @ 37% =    27,250.50   (700,000-626,350)
        //                  -----------
        //                     216,020.25
        let t = ordinary_income_tax(Decimal::from(700_000), FilingStatus::Single);
        assert_eq!(t, "216020.25".parse::<Decimal>().unwrap());
    }

    #[test]
    fn standard_deduction_values_match_irs() {
        // Rev. Proc. 2024-40 § 3.16 — pins all four statuses.
        assert_eq!(standard_deduction(FilingStatus::Single), Decimal::from(15_000));
        assert_eq!(standard_deduction(FilingStatus::Mfj),    Decimal::from(30_000));
        assert_eq!(standard_deduction(FilingStatus::Mfs),    Decimal::from(15_000));
        assert_eq!(standard_deduction(FilingStatus::Hoh),    Decimal::from(22_500));
    }

    #[test]
    fn mfj_exactly_at_12_pct_boundary() {
        // MFJ $96,950 — the exact upper edge of the 12% bracket. Tax =
        // 23,850 @ 10% + 73,100 @ 12% = 2,385 + 8,772 = 11,157. The
        // bracket walker's `<=` boundary handling is load-bearing —
        // an off-by-one would push the next dollar into 22%.
        let t = ordinary_income_tax(Decimal::from(96_950), FilingStatus::Mfj);
        assert_eq!(t, Decimal::from(11_157));
    }

    #[test]
    fn mfj_one_dollar_over_boundary_enters_next_bracket() {
        // MFJ $96,951 — the bracket walker should consume the bracket
        // entirely at 12%, then $1 at 22%. Tax = 11,157 + 0.22 = 11,157.22.
        let t = ordinary_income_tax(Decimal::from(96_951), FilingStatus::Mfj);
        assert_eq!(t, "11157.22".parse::<Decimal>().unwrap());
    }

    #[test]
    fn hoh_uses_distinct_bracket_table() {
        // HoH has its own 24% top-of-12% at $64,850 (vs $48,475 single
        // and $96,950 MFJ). Income $60k HoH stays in the 12% bracket;
        // same income single crosses into 22%.
        let t_hoh = ordinary_income_tax(Decimal::from(60_000), FilingStatus::Hoh);
        let t_single = ordinary_income_tax(Decimal::from(60_000), FilingStatus::Single);
        // HoH: 17,000 @ 10% + 43,000 @ 12% = 1,700 + 5,160 = 6,860.
        assert_eq!(t_hoh, Decimal::from(6_860));
        // Single (same income) is higher because the 22% bracket starts lower.
        assert!(t_single > t_hoh,
            "single must owe more than HoH at $60k; got single={t_single} hoh={t_hoh}");
    }

    #[test]
    fn negative_income_yields_zero_tax() {
        // Defensive: net loss → no negative tax. Refundable credits
        // come later in the pipeline.
        let t = ordinary_income_tax(Decimal::from(-50_000), FilingStatus::Single);
        assert_eq!(t, Decimal::ZERO);
    }
}
