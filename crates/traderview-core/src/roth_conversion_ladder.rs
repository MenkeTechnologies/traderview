//! Roth conversion ladder — an early-retirement bridge. Each year a fixed amount
//! is converted from a traditional IRA to Roth; the conversion is ordinary income
//! taxed at the marginal bracket above any other income, and the converted
//! principal becomes penalty-free to withdraw after a 5-year season-out. This
//! walks the ladder year by year: tax cost per conversion, effective/marginal
//! rate, the age each year's conversion becomes accessible, and the draining
//! traditional balance. TY2025 brackets (Rev. Proc. 2024-40). Faithful port of
//! the former client-side calculator. Pure compute, not advice.

use serde::{Deserialize, Serialize};

fn brackets(status: &str) -> &'static [(f64, f64)] {
    match status {
        "mfj" => &[(0.0, 0.10), (23_850.0, 0.12), (96_950.0, 0.22), (206_700.0, 0.24), (394_600.0, 0.32), (501_050.0, 0.35), (751_600.0, 0.37)],
        "mfs" => &[(0.0, 0.10), (11_925.0, 0.12), (48_475.0, 0.22), (103_350.0, 0.24), (197_300.0, 0.32), (250_525.0, 0.35), (375_800.0, 0.37)],
        "hoh" => &[(0.0, 0.10), (17_000.0, 0.12), (64_850.0, 0.22), (103_350.0, 0.24), (197_300.0, 0.32), (250_500.0, 0.35), (626_350.0, 0.37)],
        _ => &[(0.0, 0.10), (11_925.0, 0.12), (48_475.0, 0.22), (103_350.0, 0.24), (197_300.0, 0.32), (250_525.0, 0.35), (626_350.0, 0.37)],
    }
}

fn std_deduction(status: &str) -> f64 {
    match status {
        "mfj" => 30_000.0,
        "hoh" => 22_500.0,
        _ => 15_000.0,
    }
}

const BASE_YEAR: i32 = 2026;
const SEASON_YEARS: i32 = 5;

#[derive(Debug, Clone, Deserialize)]
pub struct LadderInput {
    pub filing_status: String,
    #[serde(default)]
    pub other_income_usd: f64,
    pub annual_conversion_usd: f64,
    pub years: u32,
    pub current_age: i32,
    pub traditional_balance_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LadderRow {
    pub convert_year: i32,
    pub age: i32,
    pub converted_usd: f64,
    pub tax_on_conversion_usd: f64,
    pub effective_rate_pct: f64,
    pub marginal_rate_pct: f64,
    pub access_age: i32,
    pub access_year: i32,
    pub traditional_balance_after_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct LadderReport {
    pub total_converted_usd: f64,
    pub total_tax_usd: f64,
    pub avg_effective_rate_pct: f64,
    pub traditional_remaining_usd: f64,
    pub first_access_age: Option<i32>,
    pub rows: Vec<LadderRow>,
    pub valid: bool,
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

fn tax_on(brk: &[(f64, f64)], taxable: f64) -> f64 {
    let mut tax = 0.0;
    for (idx, &(from, rate)) in brk.iter().enumerate() {
        if taxable <= from {
            break;
        }
        let to = brk.get(idx + 1).map(|b| b.0);
        let top = to.map_or(taxable, |t| taxable.min(t));
        tax += (top - from).max(0.0) * rate;
    }
    tax
}

fn marginal_at(brk: &[(f64, f64)], taxable: f64) -> f64 {
    let mut m = 0.0;
    for &(from, rate) in brk {
        if taxable >= from {
            m = rate;
        }
    }
    m
}

pub fn generate(i: &LadderInput) -> LadderReport {
    if i.annual_conversion_usd <= 0.0 {
        return LadderReport::default();
    }
    let years = i.years.clamp(1, 30);
    let brk = brackets(&i.filing_status);
    let ded = std_deduction(&i.filing_status);
    let baseline_taxable = (i.other_income_usd - ded).max(0.0);
    let baseline_tax = tax_on(brk, baseline_taxable);

    let mut balance = i.traditional_balance_usd;
    let mut rows = Vec::new();
    let mut total_tax = 0.0;
    let mut total_converted = 0.0;
    for y in 0..years as i32 {
        if balance <= 0.0 {
            break;
        }
        let convert = i.annual_conversion_usd.min(balance);
        balance -= convert;
        let taxable = (i.other_income_usd + convert - ded).max(0.0);
        let tax_on_conv = tax_on(brk, taxable) - baseline_tax;
        let effective = if convert > 0.0 { tax_on_conv / convert } else { 0.0 };
        rows.push(LadderRow {
            convert_year: BASE_YEAR + y,
            age: i.current_age + y,
            converted_usd: round2(convert),
            tax_on_conversion_usd: round2(tax_on_conv),
            effective_rate_pct: round4(effective * 100.0),
            marginal_rate_pct: round4(marginal_at(brk, taxable) * 100.0),
            access_age: i.current_age + y + SEASON_YEARS,
            access_year: BASE_YEAR + y + SEASON_YEARS,
            traditional_balance_after_usd: round2(balance),
        });
        total_tax += tax_on_conv;
        total_converted += convert;
    }

    let avg_eff = if total_converted > 0.0 { total_tax / total_converted } else { 0.0 };
    LadderReport {
        total_converted_usd: round2(total_converted),
        total_tax_usd: round2(total_tax),
        avg_effective_rate_pct: round4(avg_eff * 100.0),
        traditional_remaining_usd: round2(balance),
        first_access_age: rows.first().map(|r| r.access_age),
        rows,
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> LadderInput {
        LadderInput {
            filing_status: "mfj".into(),
            other_income_usd: 0.0,
            annual_conversion_usd: 60_000.0,
            years: 10,
            current_age: 45,
            traditional_balance_usd: 1_200_000.0,
        }
    }

    // Pins cross-checked against the original JS compute() in Python.
    #[test]
    fn default_mfj_ladder() {
        let d = generate(&base());
        assert_eq!(d.rows.len(), 10);
        assert!(close(d.rows[0].tax_on_conversion_usd, 3_123.0));
        assert!(close(d.total_converted_usd, 600_000.0));
        assert!(close(d.total_tax_usd, 31_230.0));
        assert!(close(d.avg_effective_rate_pct, 5.205));
        assert!(close(d.traditional_remaining_usd, 600_000.0));
        assert_eq!(d.first_access_age, Some(50));
        assert!(close(d.rows[0].marginal_rate_pct, 12.0));
    }

    #[test]
    fn ladder_stops_when_balance_drained() {
        // Small balance covers only 2 full conversions plus a partial.
        let d = generate(&LadderInput { traditional_balance_usd: 130_000.0, ..base() });
        assert_eq!(d.rows.len(), 3);
        assert!(close(d.rows[2].converted_usd, 10_000.0)); // 130k - 60k - 60k
        assert!(close(d.traditional_remaining_usd, 0.0));
    }

    #[test]
    fn access_age_is_five_years_out() {
        let d = generate(&base());
        for r in &d.rows {
            assert_eq!(r.access_age, r.age + 5);
        }
    }

    #[test]
    fn other_income_raises_conversion_tax() {
        // Other income pushes the conversion into higher brackets → more tax than baseline-zero.
        let d = generate(&LadderInput { other_income_usd: 100_000.0, ..base() });
        assert!(d.rows[0].tax_on_conversion_usd > 3_123.0);
    }

    #[test]
    fn zero_conversion_invalid() {
        let d = generate(&LadderInput { annual_conversion_usd: 0.0, ..base() });
        assert!(!d.valid);
    }
}
