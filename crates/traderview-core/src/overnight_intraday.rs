//! Overnight vs intraday return decomposition.
//!
//! Each session splits into two legs:
//!   overnight  = open_t / close_{t−1} − 1   (the gap)
//!   intraday   = close_t / open_t − 1       (the session)
//!
//! Compounded separately across the sample, these reveal the
//! well-documented effect that equity-index returns concentrate
//! overnight while intraday is roughly flat — load-bearing for
//! anyone holding day-trades past the close (or refusing to).
//!
//! Pure compute over (open, close) pairs. Companion to
//! `day_of_week_seasonality`, `range_volatility`.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct OvernightReport {
    pub sessions: usize,
    /// Compounded returns over the sample, %.
    pub overnight_total_pct: f64,
    pub intraday_total_pct: f64,
    pub close_to_close_total_pct: f64,
    /// Mean per-session returns, %.
    pub overnight_avg_pct: f64,
    pub intraday_avg_pct: f64,
    /// Hit rates, %.
    pub overnight_hit_rate_pct: f64,
    pub intraday_hit_rate_pct: f64,
}

/// `bars` = (open, close) pairs, oldest-first.
pub fn compute(bars: &[(f64, f64)]) -> Option<OvernightReport> {
    if bars.len() < 2
        || bars
            .iter()
            .any(|(o, c)| !o.is_finite() || *o <= 0.0 || !c.is_finite() || *c <= 0.0)
    {
        return None;
    }
    let mut on_growth = 1.0_f64;
    let mut id_growth = 1.0_f64;
    let mut on_rets: Vec<f64> = Vec::with_capacity(bars.len() - 1);
    let mut id_rets: Vec<f64> = Vec::with_capacity(bars.len() - 1);
    for w in bars.windows(2) {
        let (_, prev_close) = w[0];
        let (open, close) = w[1];
        let on = open / prev_close - 1.0;
        let id = close / open - 1.0;
        on_growth *= 1.0 + on;
        id_growth *= 1.0 + id;
        on_rets.push(on);
        id_rets.push(id);
    }
    let n = on_rets.len() as f64;
    let mean = |v: &[f64]| v.iter().sum::<f64>() / n * 100.0;
    let hits = |v: &[f64]| v.iter().filter(|r| **r > 0.0).count() as f64 / n * 100.0;
    let first_close = bars[0].1;
    let last_close = bars[bars.len() - 1].1;
    Some(OvernightReport {
        sessions: on_rets.len(),
        overnight_total_pct: (on_growth - 1.0) * 100.0,
        intraday_total_pct: (id_growth - 1.0) * 100.0,
        close_to_close_total_pct: (last_close / first_close - 1.0) * 100.0,
        overnight_avg_pct: mean(&on_rets),
        intraday_avg_pct: mean(&id_rets),
        overnight_hit_rate_pct: hits(&on_rets),
        intraday_hit_rate_pct: hits(&id_rets),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_gains_gap_overnight() {
        // Gaps up 1% every open, flat sessions: overnight compounds
        // 1.01⁴ − 1, intraday exactly 0, and overnight × intraday
        // recombines to close-to-close.
        let bars = vec![
            (100.0, 100.0),
            (101.0, 101.0),
            (102.01, 102.01),
            (103.0301, 103.0301),
            (104.060401, 104.060401),
        ];
        let r = compute(&bars).unwrap();
        assert!((r.overnight_total_pct - (1.01_f64.powi(4) - 1.0) * 100.0).abs() < 1e-9);
        assert!(r.intraday_total_pct.abs() < 1e-9);
        assert!((r.overnight_hit_rate_pct - 100.0).abs() < 1e-12);
        assert!((r.intraday_hit_rate_pct - 0.0).abs() < 1e-12);
        assert!(
            (r.close_to_close_total_pct - (1.01_f64.powi(4) - 1.0) * 100.0).abs() < 1e-9
        );
    }

    #[test]
    fn legs_recombine_multiplicatively() {
        // Arbitrary opens/closes: (1+on_total)·(1+id_total) must equal
        // 1 + close-to-close total exactly.
        let bars = vec![(100.0, 103.0), (102.0, 101.0), (104.0, 107.0), (105.0, 104.0)];
        let r = compute(&bars).unwrap();
        let lhs = (1.0 + r.overnight_total_pct / 100.0) * (1.0 + r.intraday_total_pct / 100.0);
        let rhs = 1.0 + r.close_to_close_total_pct / 100.0;
        assert!((lhs - rhs).abs() < 1e-12, "{lhs} vs {rhs}");
        assert_eq!(r.sessions, 3);
    }

    #[test]
    fn intraday_bleed_overnight_pop_is_separable() {
        // Open +2% above prior close, fade −1% into the close, daily:
        // overnight avg +2%, intraday avg −1%, both at 100%/0% hit.
        let mut bars = vec![(100.0_f64, 100.0_f64)];
        for _ in 0..10 {
            let prev_close = bars.last().expect("non-empty").1;
            let open = prev_close * 1.02;
            bars.push((open, open * 0.99));
        }
        let r = compute(&bars).unwrap();
        assert!((r.overnight_avg_pct - 2.0).abs() < 1e-9);
        assert!((r.intraday_avg_pct + 1.0).abs() < 1e-9);
        assert!(r.overnight_total_pct > 0.0 && r.intraday_total_pct < 0.0);
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(&[(100.0, 100.0)]).is_none()); // one bar
        assert!(compute(&[(100.0, 100.0), (0.0, 101.0)]).is_none());
        assert!(compute(&[(100.0, f64::NAN), (101.0, 102.0)]).is_none());
    }
}
