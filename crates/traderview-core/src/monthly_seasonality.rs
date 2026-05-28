//! Monthly Seasonality — average return by calendar month over a
//! historical sample.
//!
//! For each month index (0..12) computes:
//!   - mean log-return for bars in that month
//!   - std dev of log-returns in that month
//!   - hit rate (fraction of months that returned positive)
//!   - sample count
//!
//! Useful for "sell in May" / "Santa rally" / January-effect style
//! analysis. Pure compute. Companion to `intraday_seasonality`,
//! `holiday_seasonality`, `post_earnings_drift`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DailyClose {
    pub year: u16,
    pub month: u8,    // 1..=12
    pub close: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct MonthStats {
    pub month: u8,
    pub mean_return: f64,
    pub std_return: f64,
    pub hit_rate: f64,
    pub sample_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonthlySeasonalityReport {
    pub by_month: Vec<MonthStats>,
    pub n_observations: usize,
}

pub fn compute(closes: &[DailyClose]) -> Option<MonthlySeasonalityReport> {
    if closes.len() < 2 { return None; }
    if closes.iter().any(|c| !c.close.is_finite() || c.close <= 0.0
        || !(1..=12).contains(&c.month)) {
        return None;
    }
    // Aggregate month-end closes per (year, month).
    let mut month_end: std::collections::BTreeMap<(u16, u8), f64> =
        std::collections::BTreeMap::new();
    for c in closes {
        month_end.insert((c.year, c.month), c.close);
    }
    if month_end.len() < 2 { return None; }
    // Compute month-over-month log returns keyed by month-of-year.
    let mut returns_per_month: Vec<Vec<f64>> = vec![Vec::new(); 13];
    let mut prev: Option<((u16, u8), f64)> = None;
    for ((y, m), close) in month_end.iter() {
        if let Some((_, prev_close)) = prev {
            if prev_close > 0.0 && *close > 0.0 {
                let r = (close / prev_close).ln();
                returns_per_month[*m as usize].push(r);
            }
        }
        prev = Some(((*y, *m), *close));
    }
    let mut report = MonthlySeasonalityReport {
        by_month: Vec::new(),
        n_observations: month_end.len(),
    };
    for m in 1..=12 {
        let returns = &returns_per_month[m as usize];
        if returns.is_empty() {
            report.by_month.push(MonthStats {
                month: m, ..Default::default()
            });
            continue;
        }
        let n_f = returns.len() as f64;
        let mean: f64 = returns.iter().sum::<f64>() / n_f;
        let var: f64 = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n_f;
        let std = var.max(0.0).sqrt();
        let positive = returns.iter().filter(|r| **r > 0.0).count();
        report.by_month.push(MonthStats {
            month: m,
            mean_return: mean,
            std_return: std,
            hit_rate: positive as f64 / n_f,
            sample_count: returns.len() as u32,
        });
    }
    Some(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(year: u16, month: u8, close: f64) -> DailyClose {
        DailyClose { year, month, close }
    }

    #[test]
    fn empty_returns_none() {
        assert!(compute(&[]).is_none());
        assert!(compute(&[d(2020, 1, 100.0)]).is_none());
    }

    #[test]
    fn nan_or_invalid_month_returns_none() {
        assert!(compute(&[d(2020, 1, f64::NAN), d(2020, 2, 100.0)]).is_none());
        assert!(compute(&[d(2020, 0, 100.0), d(2020, 1, 100.0)]).is_none());
        assert!(compute(&[d(2020, 13, 100.0), d(2020, 1, 100.0)]).is_none());
    }

    #[test]
    fn flat_market_yields_zero_returns() {
        let closes = vec![
            d(2020, 1, 100.0), d(2020, 2, 100.0), d(2020, 3, 100.0),
            d(2021, 1, 100.0), d(2021, 2, 100.0), d(2021, 3, 100.0),
        ];
        let r = compute(&closes).unwrap();
        for m in &r.by_month {
            if m.sample_count > 0 {
                assert!(m.mean_return.abs() < 1e-9);
            }
        }
    }

    #[test]
    fn january_outperformance_detected() {
        // January up 10%, other months flat for 3 years.
        let closes = vec![
            d(2020, 1, 110.0), d(2020, 2, 110.0), d(2020, 12, 100.0),
            d(2021, 1, 110.0), d(2021, 2, 110.0), d(2021, 12, 100.0),
            d(2022, 1, 110.0),
        ];
        let r = compute(&closes).unwrap();
        // Find january entry.
        let jan = r.by_month.iter().find(|m| m.month == 1).unwrap();
        assert!(jan.mean_return > 0.0);
        assert_eq!(jan.hit_rate, 1.0);
    }

    #[test]
    fn output_has_12_months() {
        let closes = vec![d(2020, 1, 100.0), d(2020, 2, 105.0)];
        let r = compute(&closes).unwrap();
        assert_eq!(r.by_month.len(), 12);
    }
}
