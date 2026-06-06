//! Compound Pivots — multi-timeframe floor-pivot levels combined into a
//! single ordered set with frame tags.
//!
//! Computes classic floor pivots for daily, weekly, and monthly prior
//! sessions, then returns the unified sorted list of all 7 levels per
//! timeframe (P/R1/R2/R3/S1/S2/S3) with timeframe tags so a UI can
//! overlay 21 lines (3 timeframes × 7 levels each).
//!
//! Pure compute. Companion to `floor_pivots`, `camarilla_pivots`,
//! `woodie_pivots`, `fibonacci_pivots`, `demark_pivots`.

use serde::{Deserialize, Serialize};

pub use crate::floor_pivots::PriorSession;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PivotTimeframe {
    #[default]
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PivotLevelKind {
    #[default]
    Pivot,
    R1,
    R2,
    R3,
    S1,
    S2,
    S3,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CompoundPivotLevel {
    pub timeframe: PivotTimeframe,
    pub kind: PivotLevelKind,
    pub price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CompoundPivotsReport {
    pub levels: Vec<CompoundPivotLevel>,
}

pub fn compute(
    daily: Option<PriorSession>,
    weekly: Option<PriorSession>,
    monthly: Option<PriorSession>,
) -> CompoundPivotsReport {
    let mut levels = Vec::new();
    let push_tf =
        |levels: &mut Vec<CompoundPivotLevel>, tf: PivotTimeframe, session: PriorSession| {
            let p = crate::floor_pivots::compute(session);
            if let Some(p) = p {
                levels.push(CompoundPivotLevel {
                    timeframe: tf,
                    kind: PivotLevelKind::Pivot,
                    price: p.pivot,
                });
                levels.push(CompoundPivotLevel {
                    timeframe: tf,
                    kind: PivotLevelKind::R1,
                    price: p.r1,
                });
                levels.push(CompoundPivotLevel {
                    timeframe: tf,
                    kind: PivotLevelKind::R2,
                    price: p.r2,
                });
                levels.push(CompoundPivotLevel {
                    timeframe: tf,
                    kind: PivotLevelKind::R3,
                    price: p.r3,
                });
                levels.push(CompoundPivotLevel {
                    timeframe: tf,
                    kind: PivotLevelKind::S1,
                    price: p.s1,
                });
                levels.push(CompoundPivotLevel {
                    timeframe: tf,
                    kind: PivotLevelKind::S2,
                    price: p.s2,
                });
                levels.push(CompoundPivotLevel {
                    timeframe: tf,
                    kind: PivotLevelKind::S3,
                    price: p.s3,
                });
            }
        };
    if let Some(d) = daily {
        push_tf(&mut levels, PivotTimeframe::Daily, d);
    }
    if let Some(w) = weekly {
        push_tf(&mut levels, PivotTimeframe::Weekly, w);
    }
    if let Some(m) = monthly {
        push_tf(&mut levels, PivotTimeframe::Monthly, m);
    }
    levels.sort_by(|a, b| {
        a.price
            .partial_cmp(&b.price)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    CompoundPivotsReport { levels }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(h: f64, l: f64, c: f64) -> PriorSession {
        PriorSession {
            high: h,
            low: l,
            close: c,
        }
    }

    #[test]
    fn no_inputs_returns_empty() {
        let r = compute(None, None, None);
        assert!(r.levels.is_empty());
    }

    #[test]
    fn invalid_session_skipped() {
        let r = compute(Some(s(f64::NAN, 99.0, 100.0)), None, None);
        assert!(r.levels.is_empty());
    }

    #[test]
    fn single_timeframe_yields_seven_levels() {
        let r = compute(Some(s(110.0, 100.0, 105.0)), None, None);
        assert_eq!(r.levels.len(), 7);
        assert!(r
            .levels
            .iter()
            .all(|l| l.timeframe == PivotTimeframe::Daily));
    }

    #[test]
    fn three_timeframes_yield_21_levels() {
        let daily = s(110.0, 100.0, 105.0);
        let weekly = s(115.0, 95.0, 105.0);
        let monthly = s(120.0, 90.0, 105.0);
        let r = compute(Some(daily), Some(weekly), Some(monthly));
        assert_eq!(r.levels.len(), 21);
    }

    #[test]
    fn levels_sorted_by_price() {
        let r = compute(
            Some(s(110.0, 100.0, 105.0)),
            Some(s(115.0, 95.0, 105.0)),
            Some(s(120.0, 90.0, 105.0)),
        );
        for w in r.levels.windows(2) {
            assert!(w[0].price <= w[1].price);
        }
    }

    #[test]
    fn timeframe_tags_distinct() {
        let daily = s(110.0, 100.0, 105.0);
        let weekly = s(115.0, 95.0, 105.0);
        let r = compute(Some(daily), Some(weekly), None);
        let daily_count = r
            .levels
            .iter()
            .filter(|l| l.timeframe == PivotTimeframe::Daily)
            .count();
        let weekly_count = r
            .levels
            .iter()
            .filter(|l| l.timeframe == PivotTimeframe::Weekly)
            .count();
        assert_eq!(daily_count, 7);
        assert_eq!(weekly_count, 7);
    }
}
