//! Engulfing Pattern Scanner — weighted multi-bar engulfing detector.
//!
//! Distinct from the simple flag in `candle_patterns`: this module
//! reports a per-bar strength score in [0, 1] and a trend-context
//! filter that requires the engulfed bar's body to be a recent
//! pullback against the prevailing trend (so the engulfing actually
//! resolves a swing rather than firing in chop).
//!
//! Strength is the ratio of the engulfing bar's body to the engulfed
//! bar's body:
//!
//!   body_t      = |close_t - open_t|
//!   strength    = body_t / body_{t-1}              (clamped to 5.0)
//!   normalized  = strength / 5.0                     ∈ [0, 1]
//!
//! Bullish engulfing requires:
//!   - prior bar bearish (close_{t-1} < open_{t-1})
//!   - current bar bullish (close_t > open_t)
//!   - current open ≤ prior close AND current close ≥ prior open
//!   - prevailing trend down: close_{t-1} < SMA(close, trend_period)
//!
//! Bearish engulfing: mirror.
//!
//! Pure compute. Default trend_period = 20.
//! Companion to `candle_patterns`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EngulfingReport {
    pub bullish_strength: Vec<Option<f64>>,
    pub bearish_strength: Vec<Option<f64>>,
    pub trend_period: usize,
}

pub fn compute(bars: &[Bar], trend_period: usize) -> EngulfingReport {
    let n = bars.len();
    let mut report = EngulfingReport {
        bullish_strength: vec![None; n],
        bearish_strength: vec![None; n],
        trend_period,
    };
    if trend_period < 2 || n < trend_period + 1 {
        return report;
    }
    if bars.iter().any(|b| {
        !b.open.is_finite() || !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite()
    }) {
        return report;
    }
    let p_f = trend_period as f64;
    let mut sma = vec![None; n];
    let mut sum: f64 = bars[..trend_period].iter().map(|b| b.close).sum();
    sma[trend_period - 1] = Some(sum / p_f);
    for i in trend_period..n {
        sum += bars[i].close - bars[i - trend_period].close;
        sma[i] = Some(sum / p_f);
    }
    for i in 1..n {
        let Some(m) = sma[i] else { continue };
        let prev = bars[i - 1];
        let cur = bars[i];
        let prev_body = (prev.close - prev.open).abs();
        let cur_body = (cur.close - cur.open).abs();
        if prev_body <= 0.0 {
            continue;
        }
        let ratio = (cur_body / prev_body).min(5.0);
        let normalized = ratio / 5.0;
        // Bullish: prior bar bearish, current bullish, engulfing geometry,
        // prior close below trend (pullback into down-leg of trend).
        if prev.close < prev.open
            && cur.close > cur.open
            && cur.open <= prev.close
            && cur.close >= prev.open
            && prev.close < m
        {
            report.bullish_strength[i] = Some(normalized);
        }
        if prev.close > prev.open
            && cur.close < cur.open
            && cur.open >= prev.close
            && cur.close <= prev.open
            && prev.close > m
        {
            report.bearish_strength[i] = Some(normalized);
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(o: f64, h: f64, l: f64, c: f64) -> Bar {
        Bar {
            open: o,
            high: h,
            low: l,
            close: c,
        }
    }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 30];
        let r = compute(&bars, 1);
        assert!(r.bullish_strength.iter().all(|x| x.is_none()));
        let r2 = compute(&bars[..5], 20);
        assert!(r2.bullish_strength.iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![bar(100.0, 101.0, 99.0, 100.5); 30];
        bars[5] = bar(f64::NAN, 101.0, 99.0, 100.5);
        let r = compute(&bars, 20);
        assert!(r.bullish_strength.iter().all(|x| x.is_none()));
    }

    #[test]
    fn classic_bullish_engulfing_detected() {
        // 20 down bars, then bearish bar, then strong bullish engulfing bar.
        let mut bars: Vec<_> = (0..20)
            .map(|i| {
                let p = 100.0 - i as f64 * 0.5;
                bar(p, p + 0.5, p - 0.5, p - 0.3)
            })
            .collect();
        // Prior bar bearish at base.
        bars.push(bar(90.5, 91.0, 89.0, 89.5));
        // Engulfing bar.
        bars.push(bar(89.0, 95.0, 88.5, 92.0));
        let r = compute(&bars, 20);
        let last = bars.len() - 1;
        assert!(r.bullish_strength[last].is_some());
        assert!(r.bullish_strength[last].unwrap() > 0.0);
    }

    #[test]
    fn classic_bearish_engulfing_detected() {
        let mut bars: Vec<_> = (0..20)
            .map(|i| {
                let p = 100.0 + i as f64 * 0.5;
                bar(p, p + 0.5, p - 0.5, p + 0.3)
            })
            .collect();
        bars.push(bar(109.5, 110.5, 109.0, 110.0));
        bars.push(bar(110.5, 111.0, 105.0, 108.0));
        let r = compute(&bars, 20);
        let last = bars.len() - 1;
        assert!(r.bearish_strength[last].is_some());
    }

    #[test]
    fn bullish_engulfing_in_uptrend_skipped() {
        // Uptrend → trend SMA below prior closes. Setup a bearish prior
        // bar + bullish engulfing — but trend filter requires prior
        // close BELOW SMA to fire bullish (counter-trend pullback). In
        // an uptrend the prior close sits ABOVE the SMA → no signal.
        let mut bars: Vec<_> = (0..20)
            .map(|i| {
                let p = 100.0 + i as f64;
                bar(p, p + 0.5, p - 0.5, p + 0.3)
            })
            .collect();
        bars.push(bar(120.5, 121.0, 119.5, 120.0));
        bars.push(bar(119.8, 122.0, 119.5, 121.0));
        let r = compute(&bars, 20);
        let last = bars.len() - 1;
        assert!(r.bullish_strength[last].is_none());
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 30];
        let r = compute(&bars, 20);
        assert_eq!(r.bullish_strength.len(), 30);
        assert_eq!(r.bearish_strength.len(), 30);
    }
}
