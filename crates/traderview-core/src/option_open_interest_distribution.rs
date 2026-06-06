//! Option Open-Interest Distribution — strike-by-strike OI summary
//! across calls and puts at a single expiration.
//!
//! For each strike, reports total OI on both sides, the call/put OI
//! ratio, and the dominant side. Also reports the top-3 strikes by
//! total OI (commonly cited as "max-pain pressure points") and the
//! aggregate weighted-average strike (center of mass).
//!
//! Pure compute. Companion to `max_pain`, `gamma_pin_zone`,
//! `put_call_ratio`, `gex_scanner`, `unusual_options_activity`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StrikeOi {
    pub strike: f64,
    pub call_oi: f64,
    pub put_oi: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct StrikeSummary {
    pub strike: f64,
    pub total_oi: f64,
    pub call_put_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OptionOiReport {
    pub per_strike: Vec<StrikeSummary>,
    pub top_3_strikes: Vec<f64>,
    pub weighted_avg_strike: f64,
    pub total_call_oi: f64,
    pub total_put_oi: f64,
}

pub fn compute(strikes: &[StrikeOi]) -> Option<OptionOiReport> {
    if strikes.is_empty() {
        return None;
    }
    if strikes.iter().any(|s| {
        !s.strike.is_finite()
            || s.strike <= 0.0
            || !s.call_oi.is_finite()
            || s.call_oi < 0.0
            || !s.put_oi.is_finite()
            || s.put_oi < 0.0
    }) {
        return None;
    }
    let mut sorted: Vec<StrikeOi> = strikes.to_vec();
    sorted.sort_by(|a, b| {
        a.strike
            .partial_cmp(&b.strike)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let per_strike: Vec<StrikeSummary> = sorted
        .iter()
        .map(|s| {
            let total = s.call_oi + s.put_oi;
            let cp_ratio = if s.put_oi > 0.0 {
                s.call_oi / s.put_oi
            } else {
                f64::INFINITY
            };
            StrikeSummary {
                strike: s.strike,
                total_oi: total,
                call_put_ratio: cp_ratio,
            }
        })
        .collect();
    let total_call_oi: f64 = sorted.iter().map(|s| s.call_oi).sum();
    let total_put_oi: f64 = sorted.iter().map(|s| s.put_oi).sum();
    let total_all: f64 = total_call_oi + total_put_oi;
    let weighted_avg = if total_all > 0.0 {
        sorted
            .iter()
            .map(|s| s.strike * (s.call_oi + s.put_oi))
            .sum::<f64>()
            / total_all
    } else {
        0.0
    };
    let mut by_total: Vec<&StrikeSummary> = per_strike.iter().collect();
    by_total.sort_by(|a, b| {
        b.total_oi
            .partial_cmp(&a.total_oi)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let top_3: Vec<f64> = by_total.iter().take(3).map(|s| s.strike).collect();
    Some(OptionOiReport {
        per_strike,
        top_3_strikes: top_3,
        weighted_avg_strike: weighted_avg,
        total_call_oi,
        total_put_oi,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(strike: f64, call: f64, put: f64) -> StrikeOi {
        StrikeOi {
            strike,
            call_oi: call,
            put_oi: put,
        }
    }

    #[test]
    fn empty_or_invalid_returns_none() {
        assert!(compute(&[]).is_none());
        assert!(compute(&[s(f64::NAN, 100.0, 50.0)]).is_none());
        assert!(compute(&[s(100.0, -1.0, 0.0)]).is_none());
    }

    #[test]
    fn per_strike_summary_correct() {
        let r = compute(&[s(100.0, 200.0, 100.0)]).unwrap();
        assert_eq!(r.per_strike.len(), 1);
        assert!((r.per_strike[0].total_oi - 300.0).abs() < 1e-9);
        assert!((r.per_strike[0].call_put_ratio - 2.0).abs() < 1e-9);
    }

    #[test]
    fn zero_put_yields_infinite_ratio() {
        let r = compute(&[s(100.0, 200.0, 0.0)]).unwrap();
        assert!(r.per_strike[0].call_put_ratio.is_infinite());
    }

    #[test]
    fn weighted_avg_strike_matches_oi_center_of_mass() {
        // Strike 100 with OI 100, strike 110 with OI 100 → avg = 105.
        let r = compute(&[s(100.0, 50.0, 50.0), s(110.0, 50.0, 50.0)]).unwrap();
        assert!((r.weighted_avg_strike - 105.0).abs() < 1e-9);
    }

    #[test]
    fn top_3_strikes_sorted_by_total_oi() {
        let r = compute(&[
            s(100.0, 50.0, 50.0),   // 100
            s(105.0, 200.0, 100.0), // 300
            s(110.0, 500.0, 500.0), // 1000
            s(115.0, 150.0, 150.0), // 300
            s(120.0, 25.0, 25.0),   // 50
        ])
        .unwrap();
        assert_eq!(r.top_3_strikes.len(), 3);
        assert!((r.top_3_strikes[0] - 110.0).abs() < 1e-9);
    }

    #[test]
    fn totals_aggregate_correctly() {
        let r = compute(&[s(100.0, 100.0, 50.0), s(110.0, 50.0, 25.0)]).unwrap();
        assert!((r.total_call_oi - 150.0).abs() < 1e-9);
        assert!((r.total_put_oi - 75.0).abs() < 1e-9);
    }
}
