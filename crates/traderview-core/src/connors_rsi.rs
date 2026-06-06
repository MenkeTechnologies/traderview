//! Connors RSI (CRSI) — Larry Connors (2012).
//!
//! Composite oscillator:
//!   CRSI = (RSI(close, rsi_period)
//!         + RSI(streak_length, streak_period)
//!         + percent_rank(roc_1, rank_period)) / 3
//!
//! Where:
//!   - `streak_length` = signed count of consecutive up/down days (+3 = 3
//!     consecutive higher closes; −2 = 2 consecutive lower).
//!   - `percent_rank` = where today's 1-day ROC sits within the last
//!     `rank_period` ROCs, as 0..=100.
//!
//! Standard params: RSI period 3, streak period 2, rank period 100.
//! Range 0..=100; <5 is "extreme oversold" by Connors's published edge.
//!
//! Pure compute.

pub fn compute(
    closes: &[f64],
    rsi_period: usize,
    streak_period: usize,
    rank_period: usize,
) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if rsi_period == 0 || streak_period == 0 || rank_period == 0 {
        return out;
    }
    let rsi_close = rsi(closes, rsi_period);
    let streak = streak_lengths(closes);
    let rsi_streak = rsi(&streak, streak_period);
    let roc_1: Vec<f64> = (0..n)
        .map(|i| {
            if i == 0 || closes[i - 1] == 0.0 {
                0.0
            } else {
                (closes[i] - closes[i - 1]) / closes[i - 1] * 100.0
            }
        })
        .collect();
    for i in 0..n {
        if let (Some(a), Some(b)) = (rsi_close[i], rsi_streak[i]) {
            // Percent rank needs `rank_period` observations.
            if i + 1 < rank_period {
                continue;
            }
            let window = &roc_1[i + 1 - rank_period..=i];
            let today = roc_1[i];
            // Skip if today's ROC isn't finite (e.g. zero prior close).
            if !today.is_finite() {
                continue;
            }
            let below = window
                .iter()
                .filter(|x| x.is_finite() && **x < today)
                .count() as f64;
            let pr = below / rank_period as f64 * 100.0;
            let crsi = (a + b + pr) / 3.0;
            if crsi.is_finite() {
                out[i] = Some(crsi);
            }
        }
    }
    out
}

fn streak_lengths(closes: &[f64]) -> Vec<f64> {
    let n = closes.len();
    let mut out = vec![0.0; n];
    for i in 1..n {
        if closes[i] > closes[i - 1] {
            out[i] = if out[i - 1] > 0.0 {
                out[i - 1] + 1.0
            } else {
                1.0
            };
        } else if closes[i] < closes[i - 1] {
            out[i] = if out[i - 1] < 0.0 {
                out[i - 1] - 1.0
            } else {
                -1.0
            };
        } else {
            out[i] = 0.0;
        }
    }
    out
}

fn rsi(values: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = values.len();
    let mut out = vec![None; n];
    if period == 0 || n <= period {
        return out;
    }
    let mut gain = 0.0;
    let mut loss = 0.0;
    for i in 1..=period {
        let d = values[i] - values[i - 1];
        if d >= 0.0 {
            gain += d;
        } else {
            loss -= d;
        }
    }
    gain /= period as f64;
    loss /= period as f64;
    out[period] = Some(rsi_from(gain, loss));
    for i in (period + 1)..n {
        let d = values[i] - values[i - 1];
        let (g, l) = if d >= 0.0 { (d, 0.0) } else { (0.0, -d) };
        gain = (gain * (period as f64 - 1.0) + g) / period as f64;
        loss = (loss * (period as f64 - 1.0) + l) / period as f64;
        out[i] = Some(rsi_from(gain, loss));
    }
    out
}

fn rsi_from(gain: f64, loss: f64) -> f64 {
    if loss == 0.0 {
        return 100.0;
    }
    let rs = gain / loss;
    100.0 - 100.0 / (1.0 + rs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 3, 2, 100).is_empty());
    }

    #[test]
    fn zero_period_returns_all_none() {
        let v = vec![100.0; 200];
        assert!(compute(&v, 0, 2, 100).iter().all(|x| x.is_none()));
        assert!(compute(&v, 3, 0, 100).iter().all(|x| x.is_none()));
        assert!(compute(&v, 3, 2, 0).iter().all(|x| x.is_none()));
    }

    #[test]
    fn streak_lengths_count_consecutive_direction() {
        let v = vec![100.0, 101.0, 102.0, 103.0, 102.0, 101.0, 101.0, 102.0];
        let s = streak_lengths(&v);
        assert_eq!(s, vec![0.0, 1.0, 2.0, 3.0, -1.0, -2.0, 0.0, 1.0]);
    }

    #[test]
    fn crsi_in_range_0_100() {
        // Use a noisy series so the percent_rank doesn't degenerate.
        let v: Vec<f64> = (0..200)
            .map(|i| 100.0 + (i as f64 * 0.3).sin() * 5.0 + i as f64 * 0.02)
            .collect();
        let out = compute(&v, 3, 2, 100);
        let last = out.last().copied().flatten().expect("populated");
        assert!((0.0..=100.0).contains(&last), "CRSI out of [0,100]: {last}");
    }

    #[test]
    fn huge_period_no_panic() {
        let v = vec![100.0; 10];
        let out = compute(&v, usize::MAX, 2, 100);
        assert!(out.iter().all(|x| x.is_none()));
    }
}
