//! Regime-conditional backtest attribution — "WHEN does this strategy
//! earn". Buckets every backtest trade by the market regime at its
//! ENTRY bar (shared regime_classifier: trend-up / trend-down / range
//! / chop) and reports per-regime stats. A trend-follower that looks
//! mediocre overall is often excellent in trends and bleeding in chop;
//! this makes that split visible so a regime gate can be added
//! deliberately instead of by vibes.
//!
//! Trades whose entry bar can't be classified (classifier warmup, or
//! an entry_time not found in the bars) land in an explicit
//! `unclassified` bucket — never silently dropped.

use crate::algo_backtest::AlgoBtTrade;
use crate::models::PriceBar;
use crate::regime_classifier::{self, MarketRegime};
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Default)]
pub struct RegimeBucket {
    pub trades: usize,
    pub wins: usize,
    pub losses: usize,
    pub win_rate: f64,
    pub total_pnl: f64,
    pub avg_pnl: f64,
    pub avg_r: f64,
    /// Gross wins / gross losses; None when there are no losses (an
    /// infinite ratio rendered as a number would be a lie).
    pub profit_factor: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RegimeAttribution {
    pub period: usize,
    pub trend_up: RegimeBucket,
    pub trend_down: RegimeBucket,
    pub range: RegimeBucket,
    pub chop: RegimeBucket,
    pub unclassified: RegimeBucket,
}

fn fold(bucket: &mut RegimeBucket, acc: &mut (f64, f64, f64), t: &AlgoBtTrade) {
    bucket.trades += 1;
    if t.pnl > 0.0 {
        bucket.wins += 1;
        acc.0 += t.pnl;
    } else {
        bucket.losses += 1;
        acc.1 += -t.pnl;
    }
    bucket.total_pnl += t.pnl;
    if t.r_multiple.is_finite() {
        acc.2 += t.r_multiple;
    }
}

fn finish(bucket: &mut RegimeBucket, acc: (f64, f64, f64)) {
    if bucket.trades > 0 {
        let n = bucket.trades as f64;
        bucket.win_rate = bucket.wins as f64 / n;
        bucket.avg_pnl = bucket.total_pnl / n;
        bucket.avg_r = acc.2 / n;
        bucket.profit_factor = (acc.1 > 0.0).then(|| acc.0 / acc.1);
    }
}

/// Bucket trades given each trade's resolved entry regime — pure, the
/// piece the tests pin without involving the classifier.
pub fn bucket_trades(
    trades: &[AlgoBtTrade],
    entry_regimes: &[Option<MarketRegime>],
    period: usize,
) -> RegimeAttribution {
    let mut out = RegimeAttribution {
        period,
        trend_up: RegimeBucket::default(),
        trend_down: RegimeBucket::default(),
        range: RegimeBucket::default(),
        chop: RegimeBucket::default(),
        unclassified: RegimeBucket::default(),
    };
    let mut acc_up = (0.0, 0.0, 0.0);
    let mut acc_down = (0.0, 0.0, 0.0);
    let mut acc_range = (0.0, 0.0, 0.0);
    let mut acc_chop = (0.0, 0.0, 0.0);
    let mut acc_un = (0.0, 0.0, 0.0);
    for (t, r) in trades.iter().zip(entry_regimes) {
        match r {
            Some(MarketRegime::TrendUp) => fold(&mut out.trend_up, &mut acc_up, t),
            Some(MarketRegime::TrendDown) => fold(&mut out.trend_down, &mut acc_down, t),
            Some(MarketRegime::Range) => fold(&mut out.range, &mut acc_range, t),
            Some(MarketRegime::Chop) => fold(&mut out.chop, &mut acc_chop, t),
            None => fold(&mut out.unclassified, &mut acc_un, t),
        }
    }
    finish(&mut out.trend_up, acc_up);
    finish(&mut out.trend_down, acc_down);
    finish(&mut out.range, acc_range);
    finish(&mut out.chop, acc_chop);
    finish(&mut out.unclassified, acc_un);
    out
}

/// Resolve each trade's entry bar by timestamp (exact match — the
/// backtester stamps trades with bar times). Misses are None.
pub fn entry_regimes(
    trades: &[AlgoBtTrade],
    bars: &[PriceBar],
    regimes: &[Option<MarketRegime>],
) -> Vec<Option<MarketRegime>> {
    trades
        .iter()
        .map(|t| {
            bars.binary_search_by_key(&t.entry_time, |b| b.bar_time)
                .ok()
                .and_then(|i| regimes.get(i).copied().flatten())
        })
        .collect()
}

/// Full attribution: classify the bars, resolve entries, bucket.
pub fn attribute(trades: &[AlgoBtTrade], bars: &[PriceBar], period: usize) -> RegimeAttribution {
    use rust_decimal::prelude::ToPrimitive;
    let closes: Vec<f64> = bars
        .iter()
        .map(|b| b.close.to_f64().unwrap_or(0.0))
        .collect();
    let report = regime_classifier::compute(&closes, period);
    let resolved = entry_regimes(trades, bars, &report.regime);
    bucket_trades(trades, &resolved, period)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algo_backtest::ExitReason;
    use crate::algo_strategies::Side;
    use chrono::{TimeZone, Utc};

    fn trade(t: i64, pnl: f64, r: f64) -> AlgoBtTrade {
        AlgoBtTrade {
            entry_time: Utc.timestamp_opt(t, 0).unwrap(),
            exit_time: Utc.timestamp_opt(t + 600, 0).unwrap(),
            side: Side::Buy,
            qty: 10.0,
            entry_price: 100.0,
            exit_price: 100.0 + pnl / 10.0,
            stop_price: 99.0,
            take_profit_price: 103.0,
            pnl,
            r_multiple: r,
            bars_held: 10,
            exit_reason: ExitReason::StrategySignal,
        }
    }

    #[test]
    fn buckets_pin_per_regime_stats() {
        use MarketRegime::*;
        let trades = vec![
            trade(0, 300.0, 1.5),   // trend_up win
            trade(1, -100.0, -0.5), // trend_up loss
            trade(2, -200.0, -1.0), // chop loss
            trade(3, 50.0, 0.25),   // unclassified win
        ];
        let regimes = vec![Some(TrendUp), Some(TrendUp), Some(Chop), None];
        let a = bucket_trades(&trades, &regimes, 20);
        assert_eq!(a.trend_up.trades, 2);
        assert_eq!(a.trend_up.wins, 1);
        assert!((a.trend_up.win_rate - 0.5).abs() < 1e-12);
        assert!((a.trend_up.total_pnl - 200.0).abs() < 1e-12);
        assert_eq!(a.trend_up.profit_factor, Some(3.0)); // 300 / 100
        assert!((a.trend_up.avg_r - 0.5).abs() < 1e-12); // (1.5 - 0.5) / 2
        assert_eq!(a.chop.trades, 1);
        assert!((a.chop.total_pnl + 200.0).abs() < 1e-12);
        assert_eq!(a.chop.profit_factor, Some(0.0)); // 0 gross win / 200 loss
        // The miss is REPORTED, not dropped.
        assert_eq!(a.unclassified.trades, 1);
        // No losses in the unclassified bucket → PF is None, not inf.
        assert_eq!(a.unclassified.profit_factor, None);
        assert_eq!(a.range.trades, 0);
        assert_eq!(a.trend_down.trades, 0);
    }

    #[test]
    fn every_trade_lands_in_exactly_one_bucket() {
        use MarketRegime::*;
        let trades: Vec<AlgoBtTrade> = (0..10).map(|i| trade(i, 10.0, 0.1)).collect();
        let regimes = vec![
            Some(TrendUp), Some(TrendDown), Some(Range), Some(Chop), None,
            Some(TrendUp), Some(TrendDown), Some(Range), Some(Chop), None,
        ];
        let a = bucket_trades(&trades, &regimes, 20);
        let total = a.trend_up.trades + a.trend_down.trades + a.range.trades
            + a.chop.trades + a.unclassified.trades;
        assert_eq!(total, trades.len());
    }

    #[test]
    fn entry_lookup_misses_resolve_to_none() {
        use crate::models::{BarInterval, PriceBar};
        use rust_decimal::Decimal;
        let bars: Vec<PriceBar> = (0..5)
            .map(|i| PriceBar {
                symbol: "TEST".into(),
                interval: BarInterval::M1,
                bar_time: Utc.timestamp_opt(1_700_000_000 + i * 60, 0).unwrap(),
                open: Decimal::from(100),
                high: Decimal::from(101),
                low: Decimal::from(99),
                close: Decimal::from(100),
                volume: Decimal::from(1_000u64),
                source: "test".into(),
            })
            .collect();
        let regimes = vec![Some(MarketRegime::Range); 5];
        // One trade on a real bar time, one between bars.
        let trades = vec![trade(1_700_000_060, 10.0, 0.1), trade(1_700_000_090, 10.0, 0.1)];
        let resolved = entry_regimes(&trades, &bars, &regimes);
        assert_eq!(resolved, vec![Some(MarketRegime::Range), None]);
    }
}
