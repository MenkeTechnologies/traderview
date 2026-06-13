//! Portfolio longevity — how many years a nest egg sustains inflation-adjusted
//! withdrawals before it runs out.
//!
//! Each year the balance grows at the return, then the (inflation-grown)
//! withdrawal comes out:
//!
//! ```text
//! balance = balance × (1 + return) − withdrawalₜ
//! withdrawalₜ = withdrawal₁ × (1 + inflation)^(t−1)
//! ```
//!
//! If the balance never depletes within the horizon, the draw is sustainable.
//! Distinct from FIRE (computes the target) and Guyton-Klinger (guardrails).

use serde::{Deserialize, Serialize};

fn d_cap() -> f64 {
    100.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct LongevityInput {
    pub starting_balance_usd: f64,
    /// First-year withdrawal.
    pub annual_withdrawal_usd: f64,
    pub annual_return_pct: f64,
    /// Annual inflation that grows the withdrawal, percent.
    #[serde(default)]
    pub inflation_pct: f64,
    /// Horizon cap in years; surviving it means the draw is sustainable.
    #[serde(default = "d_cap")]
    pub max_years: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct LongevityResult {
    /// First-year withdrawal as a percent of the balance.
    pub withdrawal_rate_pct: f64,
    /// Full withdrawal years the portfolio sustains.
    pub years_lasted: u32,
    /// Whether it survives the whole horizon (sustainable draw).
    pub sustainable: bool,
    /// Balance remaining at the horizon (0 if depleted).
    pub final_balance_usd: f64,
}

pub fn analyze(input: &LongevityInput) -> LongevityResult {
    let r = input.annual_return_pct / 100.0;
    let infl = input.inflation_pct / 100.0;
    let cap = input.max_years.max(0.0) as u32;

    let mut balance = input.starting_balance_usd;
    let mut years = 0u32;
    for y in 1..=cap {
        let grown = balance * (1.0 + r);
        let draw = input.annual_withdrawal_usd * (1.0 + infl).powi((y - 1) as i32);
        if grown - draw <= 0.0 {
            balance = 0.0;
            break;
        }
        balance = grown - draw;
        years = y;
    }

    let sustainable = years >= cap && cap > 0;
    LongevityResult {
        withdrawal_rate_pct: if input.starting_balance_usd > 0.0 {
            input.annual_withdrawal_usd / input.starting_balance_usd * 100.0
        } else {
            0.0
        },
        years_lasted: years,
        sustainable,
        final_balance_usd: if sustainable { balance } else { 0.0 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-2
    }

    #[test]
    fn depletes_after_thirteen_years() {
        // 1M, 80k (8%), 4% return, 3% inflation → 13 full years.
        let r = analyze(&LongevityInput {
            starting_balance_usd: 1_000_000.0,
            annual_withdrawal_usd: 80_000.0,
            annual_return_pct: 4.0,
            inflation_pct: 3.0,
            max_years: 100.0,
        });
        assert_eq!(r.years_lasted, 13);
        assert!(!r.sustainable);
        assert!(close(r.final_balance_usd, 0.0));
    }

    #[test]
    fn withdrawal_rate() {
        let r = analyze(&LongevityInput {
            starting_balance_usd: 1_000_000.0,
            annual_withdrawal_usd: 80_000.0,
            annual_return_pct: 4.0,
            inflation_pct: 3.0,
            max_years: 100.0,
        });
        assert!(close(r.withdrawal_rate_pct, 8.0));
    }

    #[test]
    fn sustainable_when_return_beats_draw() {
        // 4% flat draw, 5% return → grows forever, survives the horizon.
        let r = analyze(&LongevityInput {
            starting_balance_usd: 1_000_000.0,
            annual_withdrawal_usd: 40_000.0,
            annual_return_pct: 5.0,
            inflation_pct: 0.0,
            max_years: 100.0,
        });
        assert!(r.sustainable);
        assert_eq!(r.years_lasted, 100);
        assert!(r.final_balance_usd > 1_000_000.0);
    }

    #[test]
    fn higher_withdrawal_shortens_life() {
        let low = analyze(&LongevityInput {
            starting_balance_usd: 1_000_000.0,
            annual_withdrawal_usd: 60_000.0,
            annual_return_pct: 4.0,
            inflation_pct: 3.0,
            max_years: 100.0,
        });
        let high = analyze(&LongevityInput {
            starting_balance_usd: 1_000_000.0,
            annual_withdrawal_usd: 100_000.0,
            annual_return_pct: 4.0,
            inflation_pct: 3.0,
            max_years: 100.0,
        });
        assert!(high.years_lasted < low.years_lasted);
    }

    #[test]
    fn inflation_shortens_life() {
        let no_infl = analyze(&LongevityInput {
            starting_balance_usd: 1_000_000.0,
            annual_withdrawal_usd: 80_000.0,
            annual_return_pct: 4.0,
            inflation_pct: 0.0,
            max_years: 100.0,
        });
        let with_infl = analyze(&LongevityInput {
            starting_balance_usd: 1_000_000.0,
            annual_withdrawal_usd: 80_000.0,
            annual_return_pct: 4.0,
            inflation_pct: 5.0,
            max_years: 100.0,
        });
        assert!(with_infl.years_lasted < no_infl.years_lasted);
    }

    #[test]
    fn higher_return_extends_life() {
        let low = analyze(&LongevityInput {
            starting_balance_usd: 1_000_000.0,
            annual_withdrawal_usd: 80_000.0,
            annual_return_pct: 2.0,
            inflation_pct: 3.0,
            max_years: 100.0,
        });
        let high = analyze(&LongevityInput {
            starting_balance_usd: 1_000_000.0,
            annual_withdrawal_usd: 80_000.0,
            annual_return_pct: 6.0,
            inflation_pct: 3.0,
            max_years: 100.0,
        });
        assert!(high.years_lasted > low.years_lasted);
    }

    #[test]
    fn withdrawal_exceeding_balance_lasts_zero() {
        let r = analyze(&LongevityInput {
            starting_balance_usd: 50_000.0,
            annual_withdrawal_usd: 80_000.0,
            annual_return_pct: 4.0,
            inflation_pct: 0.0,
            max_years: 100.0,
        });
        assert_eq!(r.years_lasted, 0);
    }

    #[test]
    fn final_balance_zero_when_depleted() {
        let r = analyze(&LongevityInput {
            starting_balance_usd: 1_000_000.0,
            annual_withdrawal_usd: 80_000.0,
            annual_return_pct: 4.0,
            inflation_pct: 3.0,
            max_years: 100.0,
        });
        assert!(close(r.final_balance_usd, 0.0));
    }
}
