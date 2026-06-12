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
    /// Fill time, epoch seconds — stamps the trip's close.
    pub ts: i64,
    /// Caller-defined marker carried onto the trip from its OPENING
    /// fill (e.g. "had a written plan"). Closing fills' flags are
    /// ignored — the discipline question is about entry.
    pub flag: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct Trip {
    pub pnl: f64,
    /// Epoch seconds of the fill that OPENED the trip (the flip fill
    /// for a flip-opened remainder).
    pub opened_ts: i64,
    /// Epoch seconds of the fill that closed the trip.
    pub closed_ts: i64,
    /// The OPENING fill's flag (flip remainders inherit the flip
    /// fill's flag).
    pub opened_flag: bool,
}

/// FIFO round trips from one symbol's chronological fills. The
/// trailing open position (if any) is ignored — closed outcomes only.
pub fn round_trips(fills: &[Fill]) -> Vec<Trip> {
    let mut out = Vec::new();
    let mut pos = 0.0_f64; // signed shares
    let mut avg = 0.0_f64;
    let mut trip_pnl = 0.0_f64; // realized gross within the open trip
    let mut trip_comm = 0.0_f64;
    let mut trip_open_ts = 0_i64;
    let mut trip_open_flag = false;
    for f in fills {
        if f.qty <= 0.0 || f.price <= 0.0 {
            continue;
        }
        let signed = if f.buy { f.qty } else { -f.qty };
        trip_comm += f.commission;
        if pos == 0.0 || (pos > 0.0) == (signed > 0.0) {
            // Opening / adding — weighted average cost.
            if pos == 0.0 {
                trip_open_ts = f.ts;
                trip_open_flag = f.flag;
            }
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
                out.push(Trip {
                    pnl: trip_pnl - trip_comm,
                    opened_ts: trip_open_ts,
                    closed_ts: f.ts,
                    opened_flag: trip_open_flag,
                });
                trip_pnl = 0.0;
                trip_comm = 0.0;
                if remaining > 0.0 {
                    // Flip: the excess opens a fresh trip at this price
                    // AND this time.
                    pos = if f.buy { remaining } else { -remaining };
                    avg = f.price;
                    trip_open_ts = f.ts;
                    trip_open_flag = f.flag;
                }
            }
        }
    }
    out
}

#[derive(Debug, Clone, Serialize)]
pub struct TripStats {
    pub trades: usize,
    pub wins: usize,
    pub win_rate: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    /// Gross wins / gross losses; None with no losses (infinity is a
    /// sample-size artifact, not a ratio).
    pub profit_factor: Option<f64>,
    /// Mean PnL per trade — must be positive for the process to be
    /// worth running.
    pub expectancy: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
    /// Kelly fraction implied by the RECORD: f* = W − (1−W)/R with
    /// R = avg_win/avg_loss. None without both wins and losses.
    pub kelly_fraction: Option<f64>,
    pub longest_win_streak: usize,
    pub longest_loss_streak: usize,
    /// Signed: +n = currently on an n-win streak, −n = n-loss streak.
    pub current_streak: i64,
}

/// Trade-quality stats over closed-trip PnLs. None for an empty set.
/// Zero-PnL trips count as losses (they paid fees for nothing).
/// Input must be CHRONOLOGICAL (by close time) — streaks depend on
/// order; the ratios don't.
pub fn trip_stats(pnls: &[f64]) -> Option<TripStats> {
    if pnls.is_empty() {
        return None;
    }
    let n = pnls.len();
    let wins: Vec<f64> = pnls.iter().copied().filter(|p| *p > 0.0).collect();
    let losses: Vec<f64> = pnls.iter().copied().filter(|p| *p <= 0.0).collect();
    let gross_win: f64 = wins.iter().sum();
    let gross_loss: f64 = -losses.iter().sum::<f64>();
    let (mut longest_win, mut longest_loss, mut run, mut run_is_win) =
        (0usize, 0usize, 0usize, false);
    for p in pnls {
        let is_win = *p > 0.0;
        if run > 0 && is_win == run_is_win {
            run += 1;
        } else {
            run = 1;
            run_is_win = is_win;
        }
        if is_win {
            longest_win = longest_win.max(run);
        } else {
            longest_loss = longest_loss.max(run);
        }
    }
    let current_streak = if run == 0 {
        0
    } else if run_is_win {
        run as i64
    } else {
        -(run as i64)
    };
    let win_rate = wins.len() as f64 / n as f64;
    let avg_win = if wins.is_empty() { 0.0 } else { gross_win / wins.len() as f64 };
    let avg_loss = if losses.is_empty() { 0.0 } else { gross_loss / losses.len() as f64 };
    let kelly_fraction = (avg_win > 0.0 && avg_loss > 0.0).then(|| {
        let r = avg_win / avg_loss;
        win_rate - (1.0 - win_rate) / r
    });
    Some(TripStats {
        trades: n,
        wins: wins.len(),
        win_rate,
        avg_win,
        avg_loss,
        profit_factor: (gross_loss > 0.0).then(|| gross_win / gross_loss),
        expectancy: pnls.iter().sum::<f64>() / n as f64,
        largest_win: pnls.iter().copied().fold(f64::MIN, f64::max).max(0.0),
        largest_loss: pnls.iter().copied().fold(f64::MAX, f64::min).min(0.0),
        kelly_fraction,
        longest_win_streak: longest_win,
        longest_loss_streak: longest_loss,
        current_streak,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct HoldStats {
    pub avg_hold_secs_winners: f64,
    pub avg_hold_secs_losers: f64,
    /// Set when losers are held ≥ 1.5× longer than winners — the
    /// classic "cutting winners, riding losers" asymmetry. None when
    /// either side is empty.
    pub behavioral_flag: Option<&'static str>,
}

/// Hold-duration discipline check over (pnl, hold_secs) pairs.
pub fn hold_stats(trips: &[(f64, i64)]) -> Option<HoldStats> {
    let winners: Vec<i64> = trips.iter().filter(|(p, _)| *p > 0.0).map(|(_, h)| *h).collect();
    let losers: Vec<i64> = trips.iter().filter(|(p, _)| *p <= 0.0).map(|(_, h)| *h).collect();
    if winners.is_empty() && losers.is_empty() {
        return None;
    }
    let avg = |v: &[i64]| {
        if v.is_empty() { 0.0 } else { v.iter().sum::<i64>() as f64 / v.len() as f64 }
    };
    let aw = avg(&winners);
    let al = avg(&losers);
    let behavioral_flag = (!winners.is_empty() && !losers.is_empty() && al >= aw * 1.5)
        .then_some("cutting_winners_riding_losers");
    Some(HoldStats {
        avg_hold_secs_winners: aw,
        avg_hold_secs_losers: al,
        behavioral_flag,
    })
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
        // ts increments per construction so closed_ts is observable.
        use std::sync::atomic::{AtomicI64, Ordering};
        static T: AtomicI64 = AtomicI64::new(1_000);
        Fill {
            buy, qty, price, commission: c,
            ts: T.fetch_add(60, Ordering::Relaxed),
            flag: false,
        }
    }

    #[test]
    fn trip_inherits_the_opening_fills_flag_only() {
        // Flagged open, unflagged close → trip flagged.
        let mut f1 = fill(true, 100.0, 10.0, 0.0);
        f1.flag = true;
        let close = fill(false, 100.0, 12.0, 0.0);
        let trips = round_trips(&[f1, close]);
        assert!(trips[0].opened_flag);
        // Unflagged open, FLAGGED close → trip unflagged: the
        // discipline question is about entry.
        let open = fill(true, 100.0, 10.0, 0.0);
        let mut c2 = fill(false, 100.0, 12.0, 0.0);
        c2.flag = true;
        assert!(!round_trips(&[open, c2])[0].opened_flag);
        // Flip: the remainder's trip carries the FLIP fill's flag.
        let open = fill(true, 100.0, 10.0, 0.0);
        let mut flip = fill(false, 150.0, 12.0, 0.0);
        flip.flag = true;
        let mut cover = fill(true, 50.0, 11.0, 0.0);
        cover.flag = false;
        let trips = round_trips(&[open, flip, cover]);
        assert_eq!(trips.len(), 2);
        assert!(!trips[0].opened_flag); // original long opened unflagged
        assert!(trips[1].opened_flag); // short remainder opened BY the flip
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
        assert!((pnls[0].pnl - 198.0).abs() < 1e-9);
        assert!((pnls[1].pnl - 98.0).abs() < 1e-9);
        // The trip closes at the CLOSING fill's timestamp and opens at
        // the OPENING fill's — hold time is their difference.
        assert!(pnls[0].closed_ts < pnls[1].closed_ts);
        assert!(pnls[0].opened_ts < pnls[0].closed_ts);
        assert_eq!(pnls[0].closed_ts - pnls[0].opened_ts, 60);
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
        assert!((pnls[0].pnl - 198.0).abs() < 1e-9);
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
