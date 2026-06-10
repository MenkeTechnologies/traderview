//! Federal marginal tax-bracket optimizer.
//!
//! Lets the user see how much "room" they have left in their current
//! marginal bracket, and how much they could realise (Roth conversion,
//! IRA withdrawal, capital gain harvest) before bumping into the next
//! bracket.
//!
//! Brackets are 2026 published IRS values for ordinary income, single
//! / married-filing-jointly / head-of-household. Standard deduction
//! 2026: single $15,000, MFJ $30,000, HoH $22,500 (per IRS inflation
//! adjustments).
//!
//! Pure compute — no tax-table lookups beyond the embedded values.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct TaxBracketInput {
    pub filing_status: String,  // "single" | "mfj" | "hoh"
    pub taxable_ordinary_income_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BracketRow {
    pub rate_pct: f64,
    pub lower_usd: f64,
    pub upper_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TaxBracketReport {
    pub filing_status: String,
    pub taxable_income_usd: f64,
    pub current_marginal_rate_pct: f64,
    pub current_bracket_upper_usd: f64,
    pub room_in_current_bracket_usd: f64,
    pub next_bracket_rate_pct: f64,
    pub federal_tax_liability_usd: f64,
    pub effective_rate_pct: f64,
    pub brackets: Vec<BracketRow>,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

/// Returns 2026 brackets for the given filing status as (rate%, lower).
/// Upper is implied by the next bracket's lower (last upper = infinity).
fn brackets_2026(status: &str) -> Vec<(f64, f64)> {
    match status {
        "mfj" => vec![
            (10.0, 0.0),
            (12.0, 23_850.0),
            (22.0, 96_950.0),
            (24.0, 206_700.0),
            (32.0, 394_600.0),
            (35.0, 501_050.0),
            (37.0, 751_600.0),
        ],
        "hoh" => vec![
            (10.0, 0.0),
            (12.0, 17_000.0),
            (22.0, 64_850.0),
            (24.0, 103_350.0),
            (32.0, 197_300.0),
            (35.0, 250_500.0),
            (37.0, 626_350.0),
        ],
        _ => vec![ // single (default)
            (10.0, 0.0),
            (12.0, 11_925.0),
            (22.0, 48_475.0),
            (24.0, 103_350.0),
            (32.0, 197_300.0),
            (35.0, 250_525.0),
            (37.0, 626_350.0),
        ],
    }
}

pub fn liability(brackets: &[(f64, f64)], taxable: f64) -> f64 {
    if taxable <= 0.0 { return 0.0; }
    let mut tax = 0.0_f64;
    for i in 0..brackets.len() {
        let (rate, lower) = brackets[i];
        let upper = brackets.get(i + 1).map(|b| b.1).unwrap_or(f64::INFINITY);
        if taxable <= lower { break; }
        let bracket_top = taxable.min(upper);
        tax += (bracket_top - lower) * rate / 100.0;
    }
    tax
}

pub fn compute(input: &TaxBracketInput) -> TaxBracketReport {
    let raw = brackets_2026(&input.filing_status);
    let taxable = input.taxable_ordinary_income_usd.max(0.0);

    // Find current bracket.
    let mut current_rate = raw[0].0;
    let mut current_upper = f64::INFINITY;
    let mut next_rate = raw[0].0;
    for i in 0..raw.len() {
        let (rate, lower) = raw[i];
        let upper = raw.get(i + 1).map(|b| b.1).unwrap_or(f64::INFINITY);
        if taxable >= lower && taxable < upper {
            current_rate = rate;
            current_upper = upper;
            next_rate = raw.get(i + 1).map(|b| b.0).unwrap_or(rate);
            break;
        }
    }
    let room = (current_upper - taxable).max(0.0);
    let liab = liability(&raw, taxable);
    let eff_rate = if taxable > 0.0 { liab / taxable * 100.0 } else { 0.0 };

    let brackets: Vec<BracketRow> = raw
        .iter()
        .enumerate()
        .map(|(i, (rate, lower))| BracketRow {
            rate_pct: *rate,
            lower_usd: *lower,
            upper_usd: raw.get(i + 1).map(|b| b.1).unwrap_or(f64::INFINITY),
        })
        .collect();

    TaxBracketReport {
        filing_status: input.filing_status.clone(),
        taxable_income_usd: taxable,
        current_marginal_rate_pct: current_rate,
        current_bracket_upper_usd: current_upper,
        room_in_current_bracket_usd: room,
        next_bracket_rate_pct: next_rate,
        federal_tax_liability_usd: liab,
        effective_rate_pct: eff_rate,
        brackets,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brackets_2026_single_first_bracket() {
        let b = brackets_2026("single");
        assert_eq!(b[0], (10.0, 0.0));
        assert_eq!(b[1].0, 12.0);
        assert_eq!(b[1].1, 11_925.0);
    }

    #[test]
    fn brackets_2026_mfj_first_bracket() {
        let b = brackets_2026("mfj");
        assert_eq!(b[1].1, 23_850.0);
    }

    #[test]
    fn brackets_unknown_status_defaults_single() {
        let single = brackets_2026("single");
        let bogus = brackets_2026("bogus");
        assert_eq!(single, bogus);
    }

    #[test]
    fn liability_zero_income() {
        let b = brackets_2026("single");
        assert_eq!(liability(&b, 0.0), 0.0);
    }

    #[test]
    fn liability_only_first_bracket_single() {
        let b = brackets_2026("single");
        // $10k income, all at 10% = $1000
        let l = liability(&b, 10_000.0);
        assert!((l - 1_000.0).abs() < 0.5);
    }

    #[test]
    fn liability_spans_first_two_brackets() {
        let b = brackets_2026("single");
        // $20k taxable: first $11,925 @ 10% = $1192.50, next $8075 @ 12% = $969
        let l = liability(&b, 20_000.0);
        let expected = 11_925.0 * 0.10 + (20_000.0 - 11_925.0) * 0.12;
        assert!((l - expected).abs() < 0.5);
    }

    #[test]
    fn compute_current_marginal_single_first_bracket() {
        let r = compute(&TaxBracketInput {
            filing_status: "single".into(),
            taxable_ordinary_income_usd: 5_000.0,
        });
        assert_eq!(r.current_marginal_rate_pct, 10.0);
        assert_eq!(r.next_bracket_rate_pct, 12.0);
        // room = $11,925 − $5,000 = $6,925
        assert!((r.room_in_current_bracket_usd - 6_925.0).abs() < 0.5);
    }

    #[test]
    fn compute_current_marginal_22_pct_mid_bracket() {
        let r = compute(&TaxBracketInput {
            filing_status: "single".into(),
            taxable_ordinary_income_usd: 80_000.0,
        });
        // $80k single: bracket = 22% [48,475 → 103,350]
        assert_eq!(r.current_marginal_rate_pct, 22.0);
        // room = $103,350 − $80,000 = $23,350
        assert!((r.room_in_current_bracket_usd - 23_350.0).abs() < 0.5);
    }

    #[test]
    fn compute_mfj_lower_marginal_at_same_income() {
        let single = compute(&TaxBracketInput {
            filing_status: "single".into(), taxable_ordinary_income_usd: 100_000.0,
        });
        let mfj = compute(&TaxBracketInput {
            filing_status: "mfj".into(), taxable_ordinary_income_usd: 100_000.0,
        });
        // MFJ brackets wider, so $100k MFJ is in lower marginal than single.
        assert!(mfj.current_marginal_rate_pct <= single.current_marginal_rate_pct);
    }

    #[test]
    fn compute_effective_rate_below_marginal() {
        let r = compute(&TaxBracketInput {
            filing_status: "single".into(), taxable_ordinary_income_usd: 200_000.0,
        });
        // 32% marginal but effective is lower (progressive).
        assert!(r.effective_rate_pct < r.current_marginal_rate_pct);
    }

    #[test]
    fn compute_bracket_row_count_seven() {
        let r = compute(&TaxBracketInput {
            filing_status: "single".into(), taxable_ordinary_income_usd: 50_000.0,
        });
        assert_eq!(r.brackets.len(), 7);
    }

    #[test]
    fn compute_zero_income_zero_liability() {
        let r = compute(&TaxBracketInput {
            filing_status: "single".into(), taxable_ordinary_income_usd: 0.0,
        });
        assert_eq!(r.federal_tax_liability_usd, 0.0);
        assert_eq!(r.effective_rate_pct, 0.0);
    }
}
