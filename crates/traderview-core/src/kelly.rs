//! Kelly Criterion position sizer.
//!
//! For a strategy with win rate `p` and payoff ratio `b` (avg win / avg loss),
//! the Kelly fraction of bankroll to risk per trade is:
//!
//!   f* = p - (1-p)/b      (binary win/loss model)
//!   f* = (b·p - q) / b    where q = 1-p
//!
//! Negative f* means no edge — don't trade. Full Kelly is theoretically
//! growth-optimal but extremely volatile in practice; half-Kelly (f*/2)
//! and quarter-Kelly (f*/4) trade some growth for far smaller drawdowns.
//!
//! Pure compute. Distinct from `optimal_f` (Vince) — Kelly uses
//! win-rate + payoff-ratio (binary); Optimal F searches a fraction
//! over the full empirical R distribution.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct KellyInput {
    /// Win probability (0..=1).
    pub win_rate: f64,
    /// Avg win / avg loss, both positive. e.g. 2.0 means winners pay 2× losers.
    pub payoff_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KellyOutput {
    /// Raw Kelly fraction. Negative = no edge → recommended_f = 0.
    pub full_kelly: f64,
    pub half_kelly: f64,
    pub quarter_kelly: f64,
    /// Recommended sizing — max(0, half_kelly). The default exposed to
    /// the UI; full-Kelly is offered as a check.
    pub recommended_f: f64,
    /// Explanation string for the UI (why negative, why clamped, etc.).
    pub note: String,
}

pub fn compute(input: &KellyInput) -> KellyOutput {
    let p = input.win_rate.clamp(0.0, 1.0);
    let b = input.payoff_ratio;
    if b <= 0.0 {
        return KellyOutput {
            note: "payoff_ratio must be > 0 — no win to size against".into(),
            ..Default::default()
        };
    }
    let q = 1.0 - p;
    let full = (b * p - q) / b;
    let half = full / 2.0;
    let quarter = full / 4.0;
    let recommended = half.max(0.0);
    let note = if full < 0.0 {
        format!("No edge: p × b = {:.3} < q = {:.3}. Don't trade.", p * b, q)
    } else if full < 0.01 {
        "Edge is tiny (< 1% Kelly). Position sizes will be tiny too.".into()
    } else if full > 0.50 {
        "Edge is very large (full-Kelly > 50%). Half-Kelly recommended due to extreme drawdown risk.".into()
    } else {
        format!(
            "Half-Kelly = {:.2}% of bankroll per trade.",
            recommended * 100.0
        )
    };
    KellyOutput {
        full_kelly: full,
        half_kelly: half,
        quarter_kelly: quarter,
        recommended_f: recommended,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_edge_yields_zero_or_negative_full_kelly() {
        // 50/50 with 1:1 payoff → f* = 0.5 - 0.5/1 = 0.
        let r = compute(&KellyInput {
            win_rate: 0.5,
            payoff_ratio: 1.0,
        });
        assert!((r.full_kelly - 0.0).abs() < 1e-12);
        assert_eq!(r.recommended_f, 0.0);
    }

    #[test]
    fn negative_edge_returns_negative_full_kelly_and_zero_recommended() {
        // 40% wr at 1:1 → f* = 0.4 - 0.6 = -0.2.
        let r = compute(&KellyInput {
            win_rate: 0.4,
            payoff_ratio: 1.0,
        });
        assert!(r.full_kelly < 0.0);
        assert_eq!(
            r.recommended_f, 0.0,
            "recommended must clamp at 0 — don't size negative bets"
        );
        assert!(r.note.contains("No edge"));
    }

    #[test]
    fn positive_edge_60_percent_wr_2_to_1_payoff() {
        // f* = (2 × 0.6 - 0.4) / 2 = (1.2 - 0.4) / 2 = 0.4.
        let r = compute(&KellyInput {
            win_rate: 0.6,
            payoff_ratio: 2.0,
        });
        assert!((r.full_kelly - 0.4).abs() < 1e-9);
        assert!((r.half_kelly - 0.2).abs() < 1e-9);
        assert!((r.quarter_kelly - 0.1).abs() < 1e-9);
        assert!((r.recommended_f - 0.2).abs() < 1e-9);
    }

    #[test]
    fn payoff_zero_returns_default_with_note() {
        let r = compute(&KellyInput {
            win_rate: 0.6,
            payoff_ratio: 0.0,
        });
        assert_eq!(r.full_kelly, 0.0);
        assert!(r.note.contains("payoff_ratio"));
    }

    #[test]
    fn extreme_edge_yields_full_over_50pct_warning() {
        // 90% wr, 5:1 payoff → f* = (5 × 0.9 - 0.1) / 5 = 0.88.
        let r = compute(&KellyInput {
            win_rate: 0.9,
            payoff_ratio: 5.0,
        });
        assert!(r.full_kelly > 0.50);
        assert!(r.note.contains("very large"));
    }

    #[test]
    fn win_rate_clamps_above_one_at_one() {
        // p > 1 is invalid input — clamp to 1.0 → f* = 1.
        let r = compute(&KellyInput {
            win_rate: 1.5,
            payoff_ratio: 1.0,
        });
        // With p=1, q=0, f* = (1×1 - 0)/1 = 1.0.
        assert!((r.full_kelly - 1.0).abs() < 1e-12);
    }

    #[test]
    fn half_and_quarter_are_strict_proportions() {
        let r = compute(&KellyInput {
            win_rate: 0.55,
            payoff_ratio: 1.5,
        });
        assert!((r.half_kelly - r.full_kelly / 2.0).abs() < 1e-12);
        assert!((r.quarter_kelly - r.full_kelly / 4.0).abs() < 1e-12);
    }

    #[test]
    fn tiny_edge_emits_dedicated_note() {
        // Edge < 1% full Kelly.
        let r = compute(&KellyInput {
            win_rate: 0.501,
            payoff_ratio: 1.0,
        });
        assert!(r.full_kelly > 0.0 && r.full_kelly < 0.01);
        assert!(r.note.contains("tiny"));
    }
}
