//! Win/loss streak statistics.
//!
//! Useful for sizing & expectations: a 60% win-rate system will hit a
//! 5-loss streak roughly every 100 trades. Knowing the historical
//! distribution of streak lengths helps the trader stay disciplined
//! when statistically-normal cold streaks happen.
//!
//! Reports:
//!   - Current streak (sign + length)
//:   - Max winning / losing streaks in history
//:   - Expected max streak length given win rate (binomial)
//:   - Probability of >= N consecutive losses occurring in the next M trades
//!
//! Pure compute. Input is a chronological PnL series.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StreaksReport {
    pub current_streak_length: usize,
    pub current_streak_is_winning: bool,
    pub max_winning_streak: usize,
    pub max_losing_streak: usize,
    /// All streaks observed (length, was_winning), in chronological order.
    pub all_streaks: Vec<Streak>,
    pub win_rate: f64,
    /// Expected longest streak length given win rate over `trade_count`
    /// trades. Rough estimate using log-base formula:
    ///   max_streak ≈ log(N × (1 - p)) / -log(p)  for winners
    ///   max_streak ≈ log(N × p)       / -log(1-p) for losers
    pub expected_max_winning_streak: f64,
    pub expected_max_losing_streak: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Streak {
    pub length: usize,
    pub is_winning: bool,
}

pub fn analyze(pnls: &[f64]) -> StreaksReport {
    let mut report = StreaksReport::default();
    if pnls.is_empty() {
        return report;
    }
    let n = pnls.len();
    let wins = pnls.iter().filter(|p| **p > 0.0).count();
    report.win_rate = wins as f64 / n as f64;

    // Run-length encode the sign sequence.
    let mut current_is_winning = pnls[0] > 0.0;
    let mut current_len = 1;
    for &p in &pnls[1..] {
        let is_win = p > 0.0;
        if is_win == current_is_winning {
            current_len += 1;
        } else {
            report.all_streaks.push(Streak {
                length: current_len,
                is_winning: current_is_winning,
            });
            current_is_winning = is_win;
            current_len = 1;
        }
    }
    report.all_streaks.push(Streak {
        length: current_len,
        is_winning: current_is_winning,
    });
    report.current_streak_length = current_len;
    report.current_streak_is_winning = current_is_winning;

    for s in &report.all_streaks {
        if s.is_winning {
            report.max_winning_streak = report.max_winning_streak.max(s.length);
        } else {
            report.max_losing_streak = report.max_losing_streak.max(s.length);
        }
    }

    let p = report.win_rate.clamp(0.0001, 0.9999);
    let n_f = n as f64;
    report.expected_max_winning_streak = (n_f * (1.0 - p)).ln() / -p.ln();
    report.expected_max_losing_streak = (n_f * p).ln() / -(1.0 - p).ln();

    report
}

/// Probability of seeing at least one run of `k` consecutive losses in
/// `m` trades given per-trade loss probability `q`.
///
/// Uses the Feller approximation (good for large m, small q^k):
///   P(at least one run of k losses) ≈ 1 - exp(-(m - k + 1) × q^k × (1 - q))
pub fn probability_of_losing_streak(q: f64, k: usize, m: usize) -> f64 {
    if m < k {
        return 0.0;
    }
    let q = q.clamp(0.0001, 0.9999);
    let lhs = (m - k + 1) as f64 * q.powi(k as i32) * (1.0 - q);
    1.0 - (-lhs).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[]);
        assert_eq!(r.current_streak_length, 0);
        assert!(r.all_streaks.is_empty());
    }

    #[test]
    fn single_win_streak_length_one() {
        let r = analyze(&[100.0]);
        assert_eq!(r.current_streak_length, 1);
        assert!(r.current_streak_is_winning);
        assert_eq!(r.max_winning_streak, 1);
        assert_eq!(r.max_losing_streak, 0);
    }

    #[test]
    fn alternating_pattern_streaks_length_one() {
        let r = analyze(&[100.0, -50.0, 100.0, -50.0]);
        assert_eq!(r.all_streaks.len(), 4);
        assert_eq!(r.max_winning_streak, 1);
        assert_eq!(r.max_losing_streak, 1);
        assert!(!r.current_streak_is_winning, "ends on a loss");
    }

    #[test]
    fn consecutive_losses_captured_as_one_streak() {
        let r = analyze(&[-50.0, -50.0, -50.0, 100.0]);
        // Streak 1: 3 losses, Streak 2: 1 win.
        assert_eq!(r.all_streaks.len(), 2);
        assert_eq!(r.all_streaks[0].length, 3);
        assert!(!r.all_streaks[0].is_winning);
        assert_eq!(r.max_losing_streak, 3);
    }

    #[test]
    fn current_streak_reflects_trailing_run() {
        let r = analyze(&[100.0, 100.0, -50.0, -50.0, -50.0]);
        assert_eq!(r.current_streak_length, 3);
        assert!(!r.current_streak_is_winning);
    }

    #[test]
    fn win_rate_calculated_correctly() {
        let r = analyze(&[100.0, 100.0, -50.0, -50.0, -50.0]);
        // 2 of 5 winners → 40%.
        assert!((r.win_rate - 0.4).abs() < 1e-9);
    }

    #[test]
    fn zero_pnl_classified_as_loss_for_streak_purposes() {
        // 0 is not > 0 → not a winner → losing streak side.
        let r = analyze(&[100.0, 0.0, 0.0]);
        // First win, then 2 "losses" (zero pnl).
        assert_eq!(r.all_streaks.len(), 2);
        assert!(!r.all_streaks[1].is_winning);
        assert_eq!(r.all_streaks[1].length, 2);
    }

    #[test]
    fn expected_max_streaks_finite_for_50_50_system() {
        // 50% win rate over 100 trades — both expected maxes should be finite.
        let pnls: Vec<f64> = (0..100)
            .map(|i| if i % 2 == 0 { 100.0 } else { -50.0 })
            .collect();
        let r = analyze(&pnls);
        assert!(r.expected_max_winning_streak.is_finite());
        assert!(r.expected_max_losing_streak.is_finite());
        assert!(r.expected_max_winning_streak > 0.0);
    }

    #[test]
    fn probability_of_losing_streak_monotonic_in_m() {
        // Same threshold, more trades → higher probability of seeing it.
        let p1 = probability_of_losing_streak(0.5, 5, 100);
        let p2 = probability_of_losing_streak(0.5, 5, 1000);
        assert!(p2 > p1);
    }

    #[test]
    fn probability_of_losing_streak_monotonic_in_k() {
        // Same trades, longer streak → less likely.
        let p1 = probability_of_losing_streak(0.5, 3, 100);
        let p2 = probability_of_losing_streak(0.5, 10, 100);
        assert!(p1 > p2);
    }

    #[test]
    fn probability_zero_when_m_lt_k() {
        // Can't have a 5-loss streak in 3 trades.
        let p = probability_of_losing_streak(0.5, 5, 3);
        assert_eq!(p, 0.0);
    }
}
