//! Win-rate confidence — is the edge real or sample noise?
//!
//! Wilson score interval on the observed win rate (robust at small n
//! where the naive normal interval breaks):
//!
//!   center = (p̂ + z²/2n) / (1 + z²/n)
//!   half   = (z / (1 + z²/n)) · √(p̂(1−p̂)/n + z²/4n²)
//!
//! The verdict compares the LOWER bound against the breakeven win rate
//! for the system's payoff ratio, 1/(1+R) — a 60% win rate over 50
//! trades sounds great until the interval still straddles breakeven.
//! Also reports the n needed for significance at the observed rate.
//!
//! Pure compute. Companion to `r_distribution` (SQN), `profit_factor`
//! (PRR — the small-sample haircut on the SAME worry).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct WinRateInput {
    pub wins: u32,
    pub losses: u32,
    /// Average winner / average loser (R multiple).
    pub payoff_ratio: f64,
    /// z for the interval (default 1.96 ≈ 95%).
    #[serde(default = "default_z")]
    pub z: f64,
}

fn default_z() -> f64 {
    1.96
}

#[derive(Debug, Clone, Serialize)]
pub struct WinRateReport {
    pub observed_win_rate_pct: f64,
    pub wilson_low_pct: f64,
    pub wilson_high_pct: f64,
    /// 1/(1+R) — the rate where expectancy is zero.
    pub breakeven_win_rate_pct: f64,
    /// Lower bound clears breakeven at the chosen confidence.
    pub statistically_significant: bool,
    /// Normal-approx trades needed for significance at the observed
    /// rate (None when observed ≤ breakeven — no n fixes a bad edge).
    pub trades_needed: Option<u64>,
}

pub fn compute(inp: &WinRateInput) -> Option<WinRateReport> {
    let n = (inp.wins + inp.losses) as f64;
    if n < 1.0
        || !inp.payoff_ratio.is_finite()
        || inp.payoff_ratio <= 0.0
        || !inp.z.is_finite()
        || inp.z <= 0.0
    {
        return None;
    }
    let p = inp.wins as f64 / n;
    let z = inp.z;
    let z2 = z * z;
    let denom = 1.0 + z2 / n;
    let center = (p + z2 / (2.0 * n)) / denom;
    let half = (z / denom) * (p * (1.0 - p) / n + z2 / (4.0 * n * n)).sqrt();
    let lo = (center - half).max(0.0);
    let hi = (center + half).min(1.0);
    let be = 1.0 / (1.0 + inp.payoff_ratio);
    let trades_needed = (p > be).then(|| {
        // n = z²·p(1−p)/(p − be)² — normal approximation.
        (z2 * p * (1.0 - p) / ((p - be) * (p - be))).ceil() as u64
    });
    Some(WinRateReport {
        observed_win_rate_pct: p * 100.0,
        wilson_low_pct: lo * 100.0,
        wilson_high_pct: hi * 100.0,
        breakeven_win_rate_pct: be * 100.0,
        statistically_significant: lo > be,
        trades_needed,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wilson_interval_matches_hand_arithmetic() {
        // 30/50 at z = 2 (literals computed independently, not via the
        // function): denom 1.08, center 0.64/1.08, half
        // 1.851852·√0.0052.
        let r = compute(&WinRateInput {
            wins: 30,
            losses: 20,
            payoff_ratio: 1.0,
            z: 2.0,
        })
        .unwrap();
        assert!((r.wilson_low_pct - 45.9054).abs() < 1e-3, "{}", r.wilson_low_pct);
        assert!((r.wilson_high_pct - 72.6132).abs() < 1e-3, "{}", r.wilson_high_pct);
        assert!((r.observed_win_rate_pct - 60.0).abs() < 1e-12);
    }

    #[test]
    fn good_looking_small_sample_is_not_significant() {
        // 60% over 50 trades at 1:1 payoff: breakeven 50%, Wilson low
        // ~46% — the interval still straddles breakeven.
        let r = compute(&WinRateInput {
            wins: 30,
            losses: 20,
            payoff_ratio: 1.0,
            z: 1.96,
        })
        .unwrap();
        assert!(!r.statistically_significant);
        // …but a target n exists and is materially larger.
        assert!(r.trades_needed.unwrap() > 50);
    }

    #[test]
    fn same_rate_at_scale_becomes_significant() {
        let r = compute(&WinRateInput {
            wins: 300,
            losses: 200,
            payoff_ratio: 1.0,
            z: 1.96,
        })
        .unwrap();
        assert!(r.statistically_significant);
    }

    #[test]
    fn high_payoff_lowers_the_bar() {
        // 40% win rate is below 1:1 breakeven but clears the 2:1
        // breakeven (33.3%) with enough trades.
        let r = compute(&WinRateInput {
            wins: 200,
            losses: 300,
            payoff_ratio: 2.0,
            z: 1.96,
        })
        .unwrap();
        assert!((r.breakeven_win_rate_pct - 100.0 / 3.0).abs() < 1e-9);
        assert!(r.statistically_significant);
        // At 1:1 the same record is hopeless: no n fixes it.
        let bad = compute(&WinRateInput {
            wins: 200,
            losses: 300,
            payoff_ratio: 1.0,
            z: 1.96,
        })
        .unwrap();
        assert!(bad.trades_needed.is_none());
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(&WinRateInput { wins: 0, losses: 0, payoff_ratio: 1.0, z: 1.96 }).is_none());
        assert!(compute(&WinRateInput { wins: 10, losses: 5, payoff_ratio: 0.0, z: 1.96 }).is_none());
        assert!(compute(&WinRateInput { wins: 10, losses: 5, payoff_ratio: 1.0, z: 0.0 }).is_none());
    }
}
