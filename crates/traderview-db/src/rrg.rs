//! Relative Rotation Graph (RRG) — sector ETFs vs SPY.
//!
//! Plots each sector on (RS-Ratio, RS-Momentum) axes; the four
//! quadrants tell you where money is rotating:
//!
//! ```text
//!   ratio>100, mom>100 → LEADING     (own it)
//!   ratio>100, mom<100 → WEAKENING   (tighten stops)
//!   ratio<100, mom<100 → LAGGING     (avoid / short)
//!   ratio<100, mom>100 → IMPROVING   (watchlist)
//! ```
//!
//! JdK's exact normalization is proprietary; this is the standard open
//! approximation used by every non-Optuma implementation:
//!   rs        = close_sym / close_bench
//!   rs_ratio  = 100 × rs / SMA(rs, RATIO_WINDOW)
//!   rs_mom    = 100 × rs_ratio_t / rs_ratio_{t−MOM_LOOKBACK}
//! A short trail of recent points per symbol lets the UI draw the
//! characteristic RRG "comet tails".

use chrono::{Duration, Utc};
use serde::Serialize;
use sqlx::PgPool;
use traderview_core::BarInterval;

const RATIO_WINDOW: usize = 50;
const MOM_LOOKBACK: usize = 10;
const TAIL_POINTS: usize = 8;
/// Sample the tail every N sessions so 8 points span ~8 weeks.
const TAIL_STRIDE: usize = 5;
const LOOKBACK_DAYS: i64 = 420;
const BENCHMARK: &str = "SPY";

#[derive(Debug, Clone, Serialize)]
pub struct RrgPoint {
    pub rs_ratio: f64,
    pub rs_momentum: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RrgEntry {
    pub ticker: String,
    pub name: String,
    /// "leading" | "weakening" | "lagging" | "improving"
    pub quadrant: &'static str,
    pub current: RrgPoint,
    /// Oldest→newest trail including `current` as the last element.
    pub tail: Vec<RrgPoint>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RrgReport {
    pub benchmark: String,
    pub entries: Vec<RrgEntry>,
    pub errors: Vec<String>,
}

fn quadrant(p: &RrgPoint) -> &'static str {
    match (p.rs_ratio >= 100.0, p.rs_momentum >= 100.0) {
        (true, true) => "leading",
        (true, false) => "weakening",
        (false, false) => "lagging",
        (false, true) => "improving",
    }
}

fn closes_f64(bars: &[traderview_core::PriceBar]) -> Vec<(chrono::NaiveDate, f64)> {
    bars.iter()
        .map(|b| {
            (
                b.bar_time.date_naive(),
                b.close.to_string().parse().unwrap_or(0.0),
            )
        })
        .collect()
}

/// Align two date-keyed close series on shared dates (inner join).
fn align(a: &[(chrono::NaiveDate, f64)], b: &[(chrono::NaiveDate, f64)]) -> Vec<(f64, f64)> {
    let bmap: std::collections::BTreeMap<_, _> = b.iter().cloned().collect();
    a.iter()
        .filter_map(|(d, va)| bmap.get(d).map(|vb| (*va, *vb)))
        .collect()
}

pub async fn compute(pool: &PgPool) -> RrgReport {
    let to = Utc::now();
    let from = to - Duration::days(LOOKBACK_DAYS);
    let mut errors = Vec::new();
    let bench_bars = match crate::prices::get_bars(pool, BENCHMARK, BarInterval::D1, from, to).await
    {
        Ok(b) if b.len() >= RATIO_WINDOW + MOM_LOOKBACK + TAIL_POINTS * TAIL_STRIDE => b,
        Ok(b) => {
            errors.push(format!("benchmark {BENCHMARK}: only {} bars", b.len()));
            return RrgReport {
                benchmark: BENCHMARK.into(),
                entries: vec![],
                errors,
            };
        }
        Err(e) => {
            errors.push(format!("benchmark fetch: {e}"));
            return RrgReport {
                benchmark: BENCHMARK.into(),
                entries: vec![],
                errors,
            };
        }
    };
    let bench = closes_f64(&bench_bars);
    let mut entries = Vec::new();
    for (ticker, name) in crate::stock_recommendation::SECTOR_ETFS {
        let bars = match crate::prices::get_bars(pool, ticker, BarInterval::D1, from, to).await {
            Ok(b) => b,
            Err(e) => {
                errors.push(format!("{ticker}: {e}"));
                continue;
            }
        };
        let sym = closes_f64(&bars);
        let pairs = align(&sym, &bench);
        let n = pairs.len();
        if n < RATIO_WINDOW + MOM_LOOKBACK + TAIL_POINTS * TAIL_STRIDE {
            errors.push(format!("{ticker}: insufficient aligned history ({n})"));
            continue;
        }
        // rs series.
        let rs: Vec<f64> = pairs
            .iter()
            .map(|(s, b)| if *b > 0.0 { s / b } else { 0.0 })
            .collect();
        // rs_ratio_t = 100 × rs_t / SMA(rs, W) at t.
        let mut rs_ratio: Vec<Option<f64>> = vec![None; n];
        let mut sum = 0.0;
        for i in 0..n {
            sum += rs[i];
            if i >= RATIO_WINDOW {
                sum -= rs[i - RATIO_WINDOW];
            }
            if i + 1 >= RATIO_WINDOW {
                let sma = sum / RATIO_WINDOW as f64;
                if sma > 0.0 {
                    rs_ratio[i] = Some(100.0 * rs[i] / sma);
                }
            }
        }
        // rs_mom_t = 100 × ratio_t / ratio_{t−L}.
        let point_at = |i: usize| -> Option<RrgPoint> {
            let ratio = rs_ratio.get(i).copied().flatten()?;
            let prev = rs_ratio.get(i.checked_sub(MOM_LOOKBACK)?).copied().flatten()?;
            if prev <= 0.0 {
                return None;
            }
            Some(RrgPoint {
                rs_ratio: ratio,
                rs_momentum: 100.0 * ratio / prev,
            })
        };
        let mut tail: Vec<RrgPoint> = Vec::with_capacity(TAIL_POINTS);
        for k in (0..TAIL_POINTS).rev() {
            if let Some(p) = point_at(n - 1 - k * TAIL_STRIDE) {
                tail.push(p);
            }
        }
        let Some(current) = tail.last().cloned() else {
            errors.push(format!("{ticker}: no computable RRG point"));
            continue;
        };
        entries.push(RrgEntry {
            ticker: ticker.to_string(),
            name: name.to_string(),
            quadrant: quadrant(&current),
            current,
            tail,
        });
    }
    RrgReport {
        benchmark: BENCHMARK.into(),
        entries,
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quadrants_map_correctly() {
        let p = |r, m| RrgPoint {
            rs_ratio: r,
            rs_momentum: m,
        };
        assert_eq!(quadrant(&p(101.0, 101.0)), "leading");
        assert_eq!(quadrant(&p(101.0, 99.0)), "weakening");
        assert_eq!(quadrant(&p(99.0, 99.0)), "lagging");
        assert_eq!(quadrant(&p(99.0, 101.0)), "improving");
    }

    #[test]
    fn align_inner_joins_on_dates() {
        use chrono::NaiveDate;
        let d = |day| NaiveDate::from_ymd_opt(2026, 1, day).unwrap();
        let a = vec![(d(1), 10.0), (d(2), 11.0), (d(3), 12.0)];
        let b = vec![(d(2), 100.0), (d(3), 101.0), (d(4), 102.0)];
        let joined = align(&a, &b);
        assert_eq!(joined, vec![(11.0, 100.0), (12.0, 101.0)]);
    }
}
