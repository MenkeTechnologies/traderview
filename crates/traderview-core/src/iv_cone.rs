//! IV cone — forward expected-move bands from the implied-vol term
//! structure. The options-market counterpart to the realized-vol cone
//! (`vol_cone`): where the market prices the underlying at each
//! horizon,
//!
//!   move_1σ = S · iv · √(d/252)
//!
//! per term point, with ±1σ/±2σ price bands. Realized cone says where
//! vol HAS lived; this says where it's PRICED to live.
//!
//! Pure compute. Companion to `vol_cone`, `probability_of_profit`.

use serde::{Deserialize, Serialize};

const TRADING_DAYS: f64 = 252.0;

#[derive(Debug, Clone, Deserialize)]
pub struct TermPoint {
    pub days: f64,
    pub iv_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConeRow {
    pub days: f64,
    pub iv_pct: f64,
    pub move_1s_pct: f64,
    pub low_1s: f64,
    pub high_1s: f64,
    pub low_2s: f64,
    pub high_2s: f64,
}

pub fn compute(spot: f64, term: &[TermPoint]) -> Option<Vec<ConeRow>> {
    if !spot.is_finite()
        || spot <= 0.0
        || term.is_empty()
        || term.len() > 50
        || term
            .iter()
            .any(|t| !t.days.is_finite() || t.days <= 0.0 || !t.iv_pct.is_finite() || t.iv_pct <= 0.0)
    {
        return None;
    }
    let mut rows: Vec<ConeRow> = term
        .iter()
        .map(|t| {
            let sigma = t.iv_pct / 100.0 * (t.days / TRADING_DAYS).sqrt();
            let m1 = spot * sigma;
            ConeRow {
                days: t.days,
                iv_pct: t.iv_pct,
                move_1s_pct: sigma * 100.0,
                low_1s: spot - m1,
                high_1s: spot + m1,
                low_2s: (spot - 2.0 * m1).max(0.0),
                high_2s: spot + 2.0 * m1,
            }
        })
        .collect();
    rows.sort_by(|a, b| a.days.partial_cmp(&b.days).unwrap_or(std::cmp::Ordering::Equal));
    Some(rows)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tp(days: f64, iv: f64) -> TermPoint {
        TermPoint { days, iv_pct: iv }
    }

    #[test]
    fn quarter_horizon_hand_walk() {
        // S 100, IV 20% at 63 days: σ√T = 0.2·√0.25 = 10% ⇒ 90/110
        // at 1σ, 80/120 at 2σ.
        let rows = compute(100.0, &[tp(63.0, 20.0)]).unwrap();
        let r = &rows[0];
        assert!((r.move_1s_pct - 10.0).abs() < 1e-9);
        assert!((r.low_1s - 90.0).abs() < 1e-9);
        assert!((r.high_1s - 110.0).abs() < 1e-9);
        assert!((r.low_2s - 80.0).abs() < 1e-9);
        assert!((r.high_2s - 120.0).abs() < 1e-9);
    }

    #[test]
    fn rows_sort_by_horizon_and_widen() {
        let rows = compute(100.0, &[tp(63.0, 20.0), tp(5.0, 25.0), tp(252.0, 18.0)]).unwrap();
        assert_eq!(rows.len(), 3);
        assert!(rows[0].days < rows[1].days && rows[1].days < rows[2].days);
        // Even with falling IV, the 1y band is wider than the 1w band.
        assert!(rows[2].move_1s_pct > rows[0].move_1s_pct);
        // 1y at 18%: exactly ±18.
        assert!((rows[2].move_1s_pct - 18.0).abs() < 1e-9);
    }

    #[test]
    fn two_sigma_floor_at_zero() {
        // 200% IV over a year would put −2σ below zero — clamp.
        let rows = compute(100.0, &[tp(252.0, 200.0)]).unwrap();
        assert_eq!(rows[0].low_2s, 0.0);
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(0.0, &[tp(63.0, 20.0)]).is_none());
        assert!(compute(100.0, &[]).is_none());
        assert!(compute(100.0, &[tp(0.0, 20.0)]).is_none());
        assert!(compute(100.0, &[tp(63.0, f64::NAN)]).is_none());
    }
}
