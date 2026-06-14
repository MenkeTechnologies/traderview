//! Stretch-IRA / SECURE 10-year-rule modeler. Most non-spouse inherited IRAs
//! must be fully distributed within 10 years. This compares four withdrawal
//! strategies — even (1/10 each year), back-loaded (all in year 10), RMD-mimic
//! (small early, larger late), and front-loaded (heavy year 1) — projecting the
//! year-by-year distribution, the incremental tax it triggers above the
//! beneficiary's baseline ordinary income (2025 brackets, standard deduction by
//! filing status), and the after-tax total, with the remaining balance growing
//! between withdrawals. The winner maximizes total after-tax dollars received.
//! Faithful port of the former client-side calculator. Pure compute, not advice.

use serde::{Deserialize, Serialize};

fn std_deduction(status: &str) -> f64 {
    match status {
        "single" => 15_000.0,
        _ => 30_000.0,
    }
}

/// 2025 brackets (lower bound, rate).
fn brackets(status: &str) -> &'static [(f64, f64)] {
    match status {
        "single" => &[
            (0.0, 0.10),
            (11_925.0, 0.12),
            (48_475.0, 0.22),
            (103_350.0, 0.24),
            (197_300.0, 0.32),
            (250_525.0, 0.35),
            (626_350.0, 0.37),
        ],
        _ => &[
            (0.0, 0.10),
            (23_850.0, 0.12),
            (96_950.0, 0.22),
            (206_700.0, 0.24),
            (394_600.0, 0.32),
            (501_050.0, 0.35),
            (751_600.0, 0.37),
        ],
    }
}

/// Strategy keys in display order.
const STRATEGY_KEYS: [&str; 4] = ["even", "backloaded", "rmd", "frontloaded"];

#[derive(Debug, Clone, Deserialize)]
pub struct StretchIraInput {
    pub balance_usd: f64,
    pub growth_pct: f64,
    pub other_income_usd: f64,
    pub filing_status: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct StretchRow {
    pub year: u32,
    pub distribution_usd: f64,
    pub tax_on_dist_usd: f64,
    pub after_tax_usd: f64,
    pub balance_after_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct StrategyRun {
    pub key: String,
    pub total_tax_usd: f64,
    pub total_received_usd: f64,
    pub rows: Vec<StretchRow>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct StretchIraReport {
    pub strategies: Vec<StrategyRun>,
    /// Key of the strategy with the highest after-tax total.
    pub winner_key: String,
    pub valid: bool,
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn apply_brackets(brk: &[(f64, f64)], taxable: f64) -> f64 {
    let mut tax = 0.0;
    for (i, &(from, rate)) in brk.iter().enumerate() {
        if taxable <= from {
            break;
        }
        let top = match brk.get(i + 1) {
            Some(&(next_from, _)) => taxable.min(next_from),
            None => taxable,
        };
        tax += (top - from).max(0.0) * rate;
    }
    tax
}

/// Pre-cap payout for strategy `key` in year `y` (0-based) given balance `bal`.
fn payout(key: &str, bal: f64, y: u32) -> f64 {
    let yf = y as f64;
    match key {
        "even" => bal / (10.0 - yf),
        "backloaded" => if y == 9 { bal } else { 0.0 },
        "rmd" => bal / (10.0 - yf).max(1.0) * (0.4 + 0.06 * yf),
        "frontloaded" => if y == 0 { bal * 0.4 } else { bal / (9.0 - yf + 1.0) },
        _ => 0.0,
    }
}

pub fn generate(i: &StretchIraInput) -> StretchIraReport {
    if i.balance_usd <= 0.0 {
        return StretchIraReport::default();
    }
    let growth = i.growth_pct / 100.0;
    let brk = brackets(&i.filing_status);
    let std_ded = std_deduction(&i.filing_status);
    let baseline_taxable = (i.other_income_usd - std_ded).max(0.0);
    let baseline_tax = apply_brackets(brk, baseline_taxable);

    let mut strategies = Vec::with_capacity(STRATEGY_KEYS.len());
    for &key in STRATEGY_KEYS.iter() {
        let mut bal = i.balance_usd;
        let mut total_tax = 0.0;
        let mut total_received = 0.0;
        let mut rows = Vec::with_capacity(10);
        for y in 0..10u32 {
            let distribution = bal.min(payout(key, bal, y));
            let taxable = (i.other_income_usd + distribution - std_ded).max(0.0);
            let tax_on_dist = apply_brackets(brk, taxable) - baseline_tax;
            let after_tax = distribution - tax_on_dist;
            total_tax += tax_on_dist;
            total_received += after_tax;
            bal = (bal - distribution) * (1.0 + growth);
            rows.push(StretchRow {
                year: y + 1,
                distribution_usd: round2(distribution),
                tax_on_dist_usd: round2(tax_on_dist),
                after_tax_usd: round2(after_tax),
                balance_after_usd: round2(bal),
            });
        }
        strategies.push(StrategyRun {
            key: key.to_string(),
            total_tax_usd: round2(total_tax),
            total_received_usd: round2(total_received),
            rows,
        });
    }

    let winner_key = strategies
        .iter()
        .fold(&strategies[0], |best, r| {
            if r.total_received_usd > best.total_received_usd { r } else { best }
        })
        .key
        .clone();

    StretchIraReport { strategies, winner_key, valid: true }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> StretchIraInput {
        StretchIraInput {
            balance_usd: 500_000.0,
            growth_pct: 6.0,
            other_income_usd: 120_000.0,
            filing_status: "mfj".into(),
        }
    }

    fn run<'a>(d: &'a StretchIraReport, key: &str) -> &'a StrategyRun {
        d.strategies.iter().find(|s| s.key == key).unwrap()
    }

    // Pins cross-checked against the JS compute() in Python.
    #[test]
    fn default_mfj_strategies() {
        let d = generate(&base());
        assert!(d.valid);
        assert_eq!(d.strategies.len(), 4);
        assert!(close(run(&d, "even").total_received_usd, 521_001.0));
        assert!(close(run(&d, "even").total_tax_usd, 138_038.74));
        assert!(close(run(&d, "backloaded").total_received_usd, 585_146.37));
        assert!(close(run(&d, "backloaded").total_tax_usd, 259_593.11));
        assert!(close(run(&d, "rmd").total_received_usd, 552_751.87));
        assert!(close(run(&d, "frontloaded").total_received_usd, 477_984.67));
        // Back-loaded wins on after-tax dollars (growth compounds longest).
        assert_eq!(d.winner_key, "backloaded");
        assert_eq!(d.strategies[0].rows.len(), 10);
    }

    #[test]
    fn backloaded_only_distributes_in_year_10() {
        let d = generate(&base());
        let bl = run(&d, "backloaded");
        for y in 0..9 {
            assert!(close(bl.rows[y].distribution_usd, 0.0));
        }
        assert!(bl.rows[9].distribution_usd > 0.0);
    }

    #[test]
    fn single_filer_differs_from_mfj() {
        let s = generate(&StretchIraInput { filing_status: "single".into(), ..base() });
        let m = generate(&base());
        // Single brackets are tighter → more tax on the same distributions.
        assert!(run(&s, "even").total_tax_usd > run(&m, "even").total_tax_usd);
    }

    #[test]
    fn invalid_when_balance_zero() {
        let d = generate(&StretchIraInput { balance_usd: 0.0, ..base() });
        assert!(!d.valid);
    }
}
