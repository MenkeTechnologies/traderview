//! Live-vs-backtest divergence — is the LIVE strategy still the one
//! you backtested?
//!
//! Companion to `strategy_decay` (which watches the SHAPE of a rolling
//! Sharpe trajectory): this one tests the live fill record against the
//! persisted backtest expectation with a binomial z-score on the win
//! rate:
//!
//!   z = (p_live − p_bt) / √(p_bt(1 − p_bt) / n)
//!
//! Verdicts: sample < MIN_SAMPLE → insufficient_sample (a verdict on
//! 3 trades is astrology); z ≤ −2 degraded; −2 < z ≤ −1 watch;
//! z ≥ +1 outperforming; else healthy.
//!
//! Live round trips are reconstructed from fills FIFO per symbol;
//! only CLOSED trips count — an open position's paper PnL is not a
//! realized outcome.

use serde::Serialize;

pub const MIN_SAMPLE: usize = 10;

#[derive(Debug, Clone)]
pub struct Fill {
    pub buy: bool,
    pub qty: f64,
    pub price: f64,
    pub commission: f64,
}

/// FIFO round-trip PnLs from one symbol's chronological fills. The
/// trailing open position (if any) is ignored — closed outcomes only.
pub fn round_trips(fills: &[Fill]) -> Vec<f64> {
    let mut out = Vec::new();
    let mut pos = 0.0_f64; // signed shares
    let mut avg = 0.0_f64;
    let mut trip_pnl = 0.0_f64; // realized gross within the open trip
    let mut trip_comm = 0.0_f64;
    for f in fills {
        if f.qty <= 0.0 || f.price <= 0.0 {
            continue;
        }
        let signed = if f.buy { f.qty } else { -f.qty };
        trip_comm += f.commission;
        if pos == 0.0 || (pos > 0.0) == (signed > 0.0) {
            // Opening / adding — weighted average cost.
            let new_pos = pos + signed;
            avg = (avg * pos.abs() + f.price * f.qty) / new_pos.abs();
            pos = new_pos;
        } else {
            // Reducing / flipping.
            let close_qty = pos.abs().min(f.qty);
            let dir = if pos > 0.0 { 1.0 } else { -1.0 };
            trip_pnl += (f.price - avg) * close_qty * dir;
            let remaining = f.qty - close_qty;
            pos += if f.buy { close_qty } else { -close_qty };
            if pos == 0.0 {
                out.push(trip_pnl - trip_comm);
                trip_pnl = 0.0;
                trip_comm = 0.0;
                if remaining > 0.0 {
                    // Flip: the excess opens a fresh trip at this price.
                    pos = if f.buy { remaining } else { -remaining };
                    avg = f.price;
                }
            }
        }
    }
    out
}

#[derive(Debug, Clone, Serialize)]
pub struct Expectation {
    pub win_rate: f64,
    pub profit_factor: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DivergenceReport {
    pub live_trades: usize,
    pub live_wins: usize,
    pub live_win_rate: Option<f64>,
    pub live_profit_factor: Option<f64>,
    pub expected: Expectation,
    /// Binomial z-score of the live win rate vs expectation; None
    /// below MIN_SAMPLE or with a degenerate expectation.
    pub win_rate_z: Option<f64>,
    pub verdict: &'static str,
}

pub fn compare(live_pnls: &[f64], expected: Expectation) -> DivergenceReport {
    let n = live_pnls.len();
    let wins = live_pnls.iter().filter(|p| **p > 0.0).count();
    let gross_win: f64 = live_pnls.iter().filter(|p| **p > 0.0).sum();
    let gross_loss: f64 = -live_pnls.iter().filter(|p| **p < 0.0).sum::<f64>();
    let live_win_rate = (n > 0).then(|| wins as f64 / n as f64);
    let live_profit_factor = (gross_loss > 0.0).then(|| gross_win / gross_loss);
    let p_bt = expected.win_rate;
    let z = (n >= MIN_SAMPLE && p_bt > 0.0 && p_bt < 1.0).then(|| {
        let p_live = wins as f64 / n as f64;
        (p_live - p_bt) / (p_bt * (1.0 - p_bt) / n as f64).sqrt()
    });
    let verdict = match z {
        None => "insufficient_sample",
        Some(z) if z <= -2.0 => "degraded",
        Some(z) if z <= -1.0 => "watch",
        Some(z) if z >= 1.0 => "outperforming",
        Some(_) => "healthy",
    };
    DivergenceReport {
        live_trades: n,
        live_wins: wins,
        live_win_rate,
        live_profit_factor,
        expected,
        win_rate_z: z,
        verdict,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fill(buy: bool, qty: f64, price: f64, c: f64) -> Fill {
        Fill { buy, qty, price, commission: c }
    }

    #[test]
    fn long_and_short_round_trips_pin_pnl_net_of_commissions() {
        // Long: 100 @ 10 → 100 @ 12, $1 each leg: 200 − 2 = 198.
        // Short: 50 @ 20 → cover 50 @ 18, $1 each leg: 100 − 2 = 98.
        let pnls = round_trips(&[
            fill(true, 100.0, 10.0, 1.0),
            fill(false, 100.0, 12.0, 1.0),
            fill(false, 50.0, 20.0, 1.0),
            fill(true, 50.0, 18.0, 1.0),
        ]);
        assert_eq!(pnls.len(), 2);
        assert!((pnls[0] - 198.0).abs() < 1e-9);
        assert!((pnls[1] - 98.0).abs() < 1e-9);
    }

    #[test]
    fn flip_closes_the_trip_and_opens_the_remainder() {
        // Buy 100 @ 10, sell 150 @ 12: trip realizes 200 − comms; the
        // surplus 50 opens a short @ 12 that stays OPEN → not counted.
        let pnls = round_trips(&[
            fill(true, 100.0, 10.0, 1.0),
            fill(false, 150.0, 12.0, 1.0),
        ]);
        assert_eq!(pnls.len(), 1);
        assert!((pnls[0] - 198.0).abs() < 1e-9);
    }

    #[test]
    fn open_position_is_not_a_realized_outcome() {
        let pnls = round_trips(&[fill(true, 100.0, 10.0, 1.0)]);
        assert!(pnls.is_empty());
    }

    #[test]
    fn z_score_pins_the_binomial_math() {
        // Expectation 60% over 20 live trades.
        let exp = || Expectation { win_rate: 0.6, profit_factor: 1.8 };
        // 6/20 live (30%): z = −0.3 / √(0.24/20) = −2.7386 → degraded.
        let mut pnls = vec![10.0; 6];
        pnls.extend(vec![-10.0; 14]);
        let r = compare(&pnls, exp());
        assert!((r.win_rate_z.unwrap() + 2.7386).abs() < 1e-3);
        assert_eq!(r.verdict, "degraded");
        // 16/20 (80%): z = +1.826 → outperforming.
        let mut pnls = vec![10.0; 16];
        pnls.extend(vec![-10.0; 4]);
        assert_eq!(compare(&pnls, exp()).verdict, "outperforming");
        // 12/20 (60%): z = 0 → healthy.
        let mut pnls = vec![10.0; 12];
        pnls.extend(vec![-10.0; 8]);
        let r = compare(&pnls, exp());
        assert!(r.win_rate_z.unwrap().abs() < 1e-12);
        assert_eq!(r.verdict, "healthy");
    }

    #[test]
    fn small_samples_refuse_a_verdict() {
        let r = compare(&[10.0, -5.0, 8.0], Expectation { win_rate: 0.6, profit_factor: 1.8 });
        assert_eq!(r.verdict, "insufficient_sample");
        assert_eq!(r.win_rate_z, None);
        // Degenerate expectation also refuses.
        let pnls = vec![10.0; 12];
        let r = compare(&pnls, Expectation { win_rate: 1.0, profit_factor: 2.0 });
        assert_eq!(r.verdict, "insufficient_sample");
    }
}
