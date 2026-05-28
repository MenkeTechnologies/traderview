//! Holiday Seasonality — return statistics around named holidays.
//!
//! Tags each historical bar with its offset from the nearest holiday
//! (negative = trading days before, positive = trading days after) and
//! aggregates returns by that offset:
//!
//!   mean_return / std_return / hit_rate / sample_count
//!
//! Useful for the Santa-rally, day-after-Thanksgiving, pre-July-4,
//! summer-Memorial-Day-effect type studies. Caller supplies trading
//! bars (close + sortable trading-day index) and an explicit list of
//! holiday trading-day indices.
//!
//! Pure compute. Default window_before / window_after = 5 trading days.
//! Companion to `monthly_seasonality`, `intraday_seasonality`,
//! `holiday_calendar`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TradingDay {
    pub trading_day_index: u32,    // increment by 1 per trading day
    pub close: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct OffsetStats {
    pub offset: i32,
    pub mean_return: f64,
    pub std_return: f64,
    pub hit_rate: f64,
    pub sample_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HolidaySeasonalityReport {
    pub by_offset: Vec<OffsetStats>,
    pub window_before: u32,
    pub window_after: u32,
}

pub fn compute(
    days: &[TradingDay],
    holiday_indices: &[u32],
    window_before: u32,
    window_after: u32,
) -> Option<HolidaySeasonalityReport> {
    if days.len() < 2 || holiday_indices.is_empty() { return None; }
    if days.iter().any(|d| !d.close.is_finite() || d.close <= 0.0) { return None; }
    // Sort days by trading_day_index.
    let mut sorted: Vec<TradingDay> = days.to_vec();
    sorted.sort_by_key(|d| d.trading_day_index);
    // Build map from index → close for fast lookup.
    let index_to_close: std::collections::BTreeMap<u32, f64> =
        sorted.iter().map(|d| (d.trading_day_index, d.close)).collect();
    let mut returns_by_offset: std::collections::BTreeMap<i32, Vec<f64>> =
        std::collections::BTreeMap::new();
    for &h in holiday_indices {
        let lo = h.saturating_sub(window_before);
        let hi = h.saturating_add(window_after);
        for idx in lo..=hi {
            // We need close at idx and idx-1 to form a return; skip idx=0.
            if idx == 0 { continue; }
            let (Some(&prev), Some(&cur)) = (index_to_close.get(&(idx - 1)), index_to_close.get(&idx))
                else { continue };
            if prev <= 0.0 || cur <= 0.0 { continue; }
            let r = (cur / prev).ln();
            let offset = idx as i32 - h as i32;
            returns_by_offset.entry(offset).or_default().push(r);
        }
    }
    let mut report = HolidaySeasonalityReport {
        by_offset: Vec::new(),
        window_before,
        window_after,
    };
    for (offset, returns) in returns_by_offset {
        let n_f = returns.len() as f64;
        let mean: f64 = returns.iter().sum::<f64>() / n_f;
        let var: f64 = returns.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / n_f;
        let std = var.max(0.0).sqrt();
        let positive = returns.iter().filter(|r| **r > 0.0).count();
        report.by_offset.push(OffsetStats {
            offset,
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

    fn d(idx: u32, close: f64) -> TradingDay {
        TradingDay { trading_day_index: idx, close }
    }

    #[test]
    fn empty_or_invalid_returns_none() {
        assert!(compute(&[], &[100], 5, 5).is_none());
        assert!(compute(&[d(0, 100.0)], &[], 5, 5).is_none());
        assert!(compute(&[d(0, f64::NAN), d(1, 100.0)], &[1], 1, 1).is_none());
    }

    #[test]
    fn returns_aggregated_around_holiday() {
        // 21 trading days, holiday at index 10. Return +1% on day 9
        // (offset -1) AND day 11 (offset +1).
        // 21 trading days, holiday at index 10.
        let mut closes = [100.0_f64; 21];
        closes[10] = 101.0;    // ret on day 10 (offset 0) = ln(1.01)
        closes[12] = 102.01;   // ret on day 12 (offset +2) = ln(102.01/101)
        // day 11 stays at 100 → ret on day 11 = ln(100/101) (negative)
        let days: Vec<_> = closes.iter().enumerate()
            .map(|(i, &c)| d(i as u32, c)).collect();
        let r = compute(&days, &[10], 3, 3).unwrap();
        assert!(!r.by_offset.is_empty());
        // Offset 0 (= holiday itself) should have positive return.
        let off0 = r.by_offset.iter().find(|s| s.offset == 0).unwrap();
        assert!(off0.mean_return > 0.0);
    }

    #[test]
    fn multiple_holidays_aggregate() {
        // 30 days, holidays at 5 and 25.
        let mut closes = vec![100.0_f64; 30];
        closes[5] = 101.0;
        closes[25] = 101.0;
        let days: Vec<_> = closes.iter().enumerate()
            .map(|(i, &c)| d(i as u32, c)).collect();
        let r = compute(&days, &[5, 25], 2, 2).unwrap();
        // Offset 0 should have 2 samples.
        let off0 = r.by_offset.iter().find(|s| s.offset == 0).unwrap();
        assert_eq!(off0.sample_count, 2);
    }

    #[test]
    fn flat_market_yields_zero_returns() {
        let days: Vec<_> = (0..20).map(|i| d(i, 100.0)).collect();
        let r = compute(&days, &[10], 3, 3).unwrap();
        for s in &r.by_offset {
            assert!(s.mean_return.abs() < 1e-9);
        }
    }
}
