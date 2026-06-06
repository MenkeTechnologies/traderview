//! Day-of-Week Seasonality — return statistics by weekday over a
//! historical sample.
//!
//! Caller supplies closes tagged by weekday (1=Mon..5=Fri).
//! Computes per-weekday:
//!   mean log-return / std dev / hit rate / sample count.
//!
//! Used to study the "Monday-effect" (negative Monday return),
//! Friday close-out behavior, etc.
//!
//! Pure compute. Companion to `monthly_seasonality`,
//! `intraday_seasonality`, `holiday_seasonality`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DailyClose {
    pub day_of_week: u8, // 1=Mon, 5=Fri
    pub close: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct WeekdayStats {
    pub day_of_week: u8,
    pub mean_return: f64,
    pub std_return: f64,
    pub hit_rate: f64,
    pub sample_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DayOfWeekSeasonalityReport {
    pub by_weekday: Vec<WeekdayStats>,
    pub n_observations: usize,
}

pub fn compute(closes: &[DailyClose]) -> Option<DayOfWeekSeasonalityReport> {
    if closes.len() < 2 {
        return None;
    }
    if closes
        .iter()
        .any(|c| !c.close.is_finite() || c.close <= 0.0 || !(1..=7).contains(&c.day_of_week))
    {
        return None;
    }
    let mut returns_per_dow: Vec<Vec<f64>> = vec![Vec::new(); 8];
    for w in closes.windows(2) {
        let (prev, cur) = (w[0], w[1]);
        if prev.close > 0.0 && cur.close > 0.0 {
            let r = (cur.close / prev.close).ln();
            returns_per_dow[cur.day_of_week as usize].push(r);
        }
    }
    let mut report = DayOfWeekSeasonalityReport {
        by_weekday: Vec::new(),
        n_observations: closes.len(),
    };
    for dow in 1..=7 {
        let returns = &returns_per_dow[dow as usize];
        if returns.is_empty() {
            report.by_weekday.push(WeekdayStats {
                day_of_week: dow,
                ..Default::default()
            });
            continue;
        }
        let n_f = returns.len() as f64;
        let mean: f64 = returns.iter().sum::<f64>() / n_f;
        let var: f64 = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n_f;
        let std = var.max(0.0).sqrt();
        let positive = returns.iter().filter(|r| **r > 0.0).count();
        report.by_weekday.push(WeekdayStats {
            day_of_week: dow,
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

    fn d(dow: u8, close: f64) -> DailyClose {
        DailyClose {
            day_of_week: dow,
            close,
        }
    }

    #[test]
    fn empty_or_invalid_returns_none() {
        assert!(compute(&[]).is_none());
        assert!(compute(&[d(1, 100.0)]).is_none());
        assert!(compute(&[d(0, 100.0), d(1, 100.0)]).is_none());
        assert!(compute(&[d(1, f64::NAN), d(2, 100.0)]).is_none());
    }

    #[test]
    fn flat_market_yields_zero_returns() {
        let closes: Vec<_> = (0..14).map(|i| d((i % 5 + 1) as u8, 100.0)).collect();
        let r = compute(&closes).unwrap();
        for s in &r.by_weekday {
            if s.sample_count > 0 {
                assert!(s.mean_return.abs() < 1e-9);
            }
        }
    }

    #[test]
    fn monday_effect_detected() {
        // 4 weeks: Monday closes 99, Tuesday-Friday flat at 100.
        // Each week starts Friday at 100, then Monday drops to 99
        // (return = ln(99/100) negative).
        let closes = vec![
            d(5, 100.0),
            d(1, 99.0),
            d(2, 100.0),
            d(3, 100.0),
            d(4, 100.0),
            d(5, 100.0),
            d(1, 99.0),
            d(2, 100.0),
            d(3, 100.0),
            d(4, 100.0),
            d(5, 100.0),
            d(1, 99.0),
            d(2, 100.0),
            d(3, 100.0),
            d(4, 100.0),
        ];
        let r = compute(&closes).unwrap();
        let mon = r.by_weekday.iter().find(|s| s.day_of_week == 1).unwrap();
        assert!(mon.mean_return < 0.0);
        assert_eq!(mon.hit_rate, 0.0);
    }

    #[test]
    fn report_has_seven_weekday_slots() {
        let closes = vec![d(1, 100.0), d(2, 101.0)];
        let r = compute(&closes).unwrap();
        assert_eq!(r.by_weekday.len(), 7);
    }
}
