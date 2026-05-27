//! Ralph Vince's Optimal F position sizer + half/quarter Kelly variants.
//!
//! Optimal F is the fraction of bankroll that maximizes geometric growth
//! given a historical (or simulated) trade return series. It's the
//! continuous-return analog of full Kelly. Most practitioners run at
//! half-Kelly or quarter-Kelly because:
//!   1. Estimated edges are usually overstated → full Kelly over-bets
//!   2. Drawdowns under full Kelly hit ~50% even with positive edge
//!
//! Method: scan f ∈ (0, 1) at 0.001 resolution, pick the f that
//! maximizes ∏(1 + f × R_i / |worst_loss|). Fixed-grid search, no
//! optimization library required.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OptimalFReport {
    /// Best f in (0, 1].
    pub optimal_f: f64,
    /// Terminal wealth multiple (TWR) at optimal_f.
    pub twr_at_optimal: f64,
    /// Half-Kelly fraction = optimal_f / 2 — conservative practitioner default.
    pub half_kelly: f64,
    /// Quarter-Kelly fraction.
    pub quarter_kelly: f64,
    /// Worst single-trade loss in the series (absolute value).
    pub worst_loss: f64,
    /// Notes about input quality (insufficient samples, all positive, etc).
    pub note: String,
}

/// Returns: trade returns as $-P&L (NOT as fractions). The engine pulls
/// out the worst loss for normalization, then scans.
pub fn compute(returns: &[f64]) -> OptimalFReport {
    if returns.is_empty() {
        return OptimalFReport {
            note: "no trade returns supplied".into(),
            ..Default::default()
        };
    }
    let worst_loss = returns.iter().cloned().fold(0.0_f64, |acc, r| acc.min(r));
    let worst_abs = worst_loss.abs();
    if worst_abs == 0.0 {
        // No losing trades — optimal f is 1.0 (bet everything).
        // Compute TWR at f=1.
        let twr = returns.iter().fold(1.0_f64, |w, r| w * (1.0 + r.max(0.0)));
        return OptimalFReport {
            optimal_f: 1.0,
            twr_at_optimal: twr,
            half_kelly: 0.5,
            quarter_kelly: 0.25,
            worst_loss: 0.0,
            note: "no losing trades — full bet maximizes TWR".into(),
        };
    }

    // Grid search f from 0.001 to 0.999.
    let mut best_f = 0.0_f64;
    let mut best_twr = 1.0_f64; // f=0 → wealth never moves
    let mut f = 0.001_f64;
    while f < 1.0 {
        let twr = returns.iter().fold(1.0_f64, |w, r| {
            // Normalized return per the worst loss.
            let factor = 1.0 + f * (r / worst_abs);
            // Truncated bankroll — if factor goes to 0 or below the
            // sequence ruins out.
            if factor <= 0.0 {
                0.0
            } else {
                w * factor
            }
        });
        if twr > best_twr {
            best_twr = twr;
            best_f = f;
        }
        f += 0.001;
    }
    let note = if returns.len() < 30 {
        "warning: <30 trades is too few for reliable Optimal F".into()
    } else if best_f == 0.0 {
        "no positive f maximizes TWR — edge is too thin or negative".into()
    } else {
        "Optimal F via 0.001-grid scan over (0, 1)".into()
    };
    OptimalFReport {
        optimal_f: best_f,
        twr_at_optimal: best_twr,
        half_kelly: best_f / 2.0,
        quarter_kelly: best_f / 4.0,
        worst_loss,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_returns_zero_with_note() {
        let r = compute(&[]);
        assert_eq!(r.optimal_f, 0.0);
        assert!(r.note.contains("no trade"));
    }

    #[test]
    fn all_winners_returns_full_f() {
        let r = compute(&[100.0, 100.0, 100.0]);
        assert_eq!(r.optimal_f, 1.0);
        assert!(r.note.contains("no losing"));
    }

    #[test]
    fn coin_flip_50_50_with_positive_edge_picks_positive_f() {
        // 5 wins of +100, 4 losses of -50. Avg = (500-200)/9 = +33.33,
        // positive edge → optimal_f > 0.
        let returns = vec![
            100.0, 100.0, 100.0, 100.0, 100.0, -50.0, -50.0, -50.0, -50.0,
        ];
        let r = compute(&returns);
        assert!(r.optimal_f > 0.0);
        assert!(r.optimal_f < 1.0, "won't over-bet to ruin");
    }

    #[test]
    fn pure_loser_series_picks_zero_or_near_zero() {
        // All losses → f=0 is best (don't bet).
        let r = compute(&[-100.0, -50.0, -75.0, -30.0]);
        // Grid skips 0.0; best_f stays 0.0 because no positive f beats
        // f=0 TWR of 1.0.
        assert_eq!(r.optimal_f, 0.0);
    }

    #[test]
    fn half_and_quarter_kelly_are_proportional_to_optimal() {
        let returns = vec![100.0; 30]
            .into_iter()
            .chain(vec![-50.0; 20])
            .collect::<Vec<_>>();
        let r = compute(&returns);
        assert!((r.half_kelly - r.optimal_f / 2.0).abs() < 1e-9);
        assert!((r.quarter_kelly - r.optimal_f / 4.0).abs() < 1e-9);
    }

    #[test]
    fn small_sample_warns_in_note() {
        let r = compute(&[100.0, -50.0, 100.0, -50.0]);
        assert!(
            r.note.contains("<30 trades"),
            "small sample should disclose the caveat"
        );
    }

    #[test]
    fn worst_loss_recorded_for_normalization_audit() {
        let r = compute(&[100.0, -200.0, 50.0, -75.0]);
        assert_eq!(r.worst_loss, -200.0);
    }
}
