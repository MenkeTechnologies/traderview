//! Dual-Band SuperTrend — combines two SuperTrend overlays at different
//! ATR sensitivities for confluence-based signaling.
//!
//! SuperTrend (Olivier Seban):
//!   mid     = (high + low) / 2
//!   atr     = N-period Average True Range
//!   upper   = mid + multiplier · atr
//!   lower   = mid − multiplier · atr
//!   active band switches when close crosses; only one band is "live"
//!   at a time and provides the trailing stop.
//!
//! Dual-band combines a SHORT-period (fast) SuperTrend with a
//! LONG-period (slow) SuperTrend:
//!
//!   - Both bullish (close > both bands): strong long signal
//!   - Both bearish: strong short signal
//!   - Mixed: chop / no entry
//!
//! Used for filtering false breakouts and reducing whipsaw vs a single
//! SuperTrend.
//!
//! Pure compute. Companion to `supertrend`, `donchian_channels`,
//! `bollinger_band_width`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    StrongLong,
    StrongShort,
    Chop,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DualSupertrendReport {
    pub fast_supertrend: Vec<Option<f64>>,
    pub slow_supertrend: Vec<Option<f64>>,
    pub fast_direction: Vec<Option<i8>>, // +1 long / -1 short
    pub slow_direction: Vec<Option<i8>>,
    pub regime: Vec<Option<Regime>>,
}

pub fn compute(
    bars: &[Bar],
    fast_atr_period: usize,
    fast_multiplier: f64,
    slow_atr_period: usize,
    slow_multiplier: f64,
) -> DualSupertrendReport {
    let n = bars.len();
    let mut fast_st = vec![None; n];
    let mut slow_st = vec![None; n];
    let mut fast_dir = vec![None; n];
    let mut slow_dir = vec![None; n];
    let mut regime = vec![None; n];
    if n == 0
        || fast_atr_period < 2
        || slow_atr_period < 2
        || fast_multiplier <= 0.0
        || slow_multiplier <= 0.0
        || !fast_multiplier.is_finite()
        || !slow_multiplier.is_finite()
    {
        return DualSupertrendReport {
            fast_supertrend: fast_st,
            slow_supertrend: slow_st,
            fast_direction: fast_dir,
            slow_direction: slow_dir,
            regime,
        };
    }
    if bars
        .iter()
        .any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.close.is_finite())
    {
        return DualSupertrendReport {
            fast_supertrend: fast_st,
            slow_supertrend: slow_st,
            fast_direction: fast_dir,
            slow_direction: slow_dir,
            regime,
        };
    }
    let (st_f, dir_f) = supertrend(bars, fast_atr_period, fast_multiplier);
    let (st_s, dir_s) = supertrend(bars, slow_atr_period, slow_multiplier);
    fast_st = st_f;
    slow_st = st_s;
    fast_dir = dir_f;
    slow_dir = dir_s;
    for i in 0..n {
        if let (Some(f), Some(s)) = (fast_dir[i], slow_dir[i]) {
            regime[i] = Some(match (f, s) {
                (1, 1) => Regime::StrongLong,
                (-1, -1) => Regime::StrongShort,
                _ => Regime::Chop,
            });
        }
    }
    DualSupertrendReport {
        fast_supertrend: fast_st,
        slow_supertrend: slow_st,
        fast_direction: fast_dir,
        slow_direction: slow_dir,
        regime,
    }
}

fn supertrend(
    bars: &[Bar],
    atr_period: usize,
    multiplier: f64,
) -> (Vec<Option<f64>>, Vec<Option<i8>>) {
    let n = bars.len();
    let mut st = vec![None; n];
    let mut dir = vec![None; n];
    if n <= atr_period {
        return (st, dir);
    }
    let atr = compute_atr(bars, atr_period);
    let mut prev_upper = 0.0_f64;
    let mut prev_lower = 0.0_f64;
    let mut prev_direction = 1_i8;
    for i in atr_period..n {
        let mid = (bars[i].high + bars[i].low) / 2.0;
        let a = atr[i].unwrap_or(0.0);
        let basic_upper = mid + multiplier * a;
        let basic_lower = mid - multiplier * a;
        let final_upper =
            if i == atr_period || basic_upper < prev_upper || bars[i - 1].close > prev_upper {
                basic_upper
            } else {
                prev_upper
            };
        let final_lower =
            if i == atr_period || basic_lower > prev_lower || bars[i - 1].close < prev_lower {
                basic_lower
            } else {
                prev_lower
            };
        let new_dir = if i == atr_period {
            if bars[i].close > final_upper {
                1
            } else {
                -1
            }
        } else if prev_direction == 1 && bars[i].close < final_lower {
            -1
        } else if prev_direction == -1 && bars[i].close > final_upper {
            1
        } else {
            prev_direction
        };
        let trend_value = if new_dir == 1 {
            final_lower
        } else {
            final_upper
        };
        st[i] = Some(trend_value);
        dir[i] = Some(new_dir);
        prev_upper = final_upper;
        prev_lower = final_lower;
        prev_direction = new_dir;
    }
    (st, dir)
}

fn compute_atr(bars: &[Bar], period: usize) -> Vec<Option<f64>> {
    let n = bars.len();
    let mut out = vec![None; n];
    if n < 2 {
        return out;
    }
    let mut trs = vec![0.0_f64; n];
    trs[0] = bars[0].high - bars[0].low;
    for i in 1..n {
        let tr = (bars[i].high - bars[i].low)
            .max((bars[i].high - bars[i - 1].close).abs())
            .max((bars[i].low - bars[i - 1].close).abs());
        trs[i] = tr;
    }
    if period == 0 || n < period {
        return out;
    }
    let mut sum: f64 = trs[..period].iter().sum();
    let mut avg = sum / period as f64;
    out[period - 1] = Some(avg);
    for i in period..n {
        avg = (avg * (period - 1) as f64 + trs[i]) / period as f64;
        sum = avg * period as f64; // suppresses unused-var lint without changing logic
        out[i] = Some(avg);
    }
    let _ = sum;
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64, c: f64) -> Bar {
        Bar {
            high: h,
            low: l,
            close: c,
        }
    }

    #[test]
    fn empty_returns_empty_outputs() {
        let r = compute(&[], 7, 1.5, 21, 3.0);
        assert!(r.fast_supertrend.is_empty());
    }

    #[test]
    fn invalid_params_return_all_none() {
        let bars: Vec<_> = (0..50).map(|_| b(101.0, 99.0, 100.0)).collect();
        let r = compute(&bars, 1, 1.5, 21, 3.0);
        assert!(r.fast_supertrend.iter().all(|x| x.is_none()));
        let r2 = compute(&bars, 7, 0.0, 21, 3.0);
        assert!(r2.fast_supertrend.iter().all(|x| x.is_none()));
    }

    #[test]
    fn uptrend_yields_strong_long_regime() {
        let bars: Vec<_> = (0..100)
            .map(|i| {
                let mid = 100.0 + i as f64;
                b(mid + 0.5, mid - 0.5, mid)
            })
            .collect();
        let r = compute(&bars, 7, 1.5, 21, 3.0);
        let last_regime = r.regime[99].unwrap();
        assert_eq!(last_regime, Regime::StrongLong);
    }

    #[test]
    fn downtrend_yields_strong_short_regime() {
        let bars: Vec<_> = (0..100)
            .map(|i| {
                let mid = 200.0 - i as f64;
                b(mid + 0.5, mid - 0.5, mid)
            })
            .collect();
        let r = compute(&bars, 7, 1.5, 21, 3.0);
        let last_regime = r.regime[99].unwrap();
        assert_eq!(last_regime, Regime::StrongShort);
    }

    #[test]
    fn reversal_produces_chop_regime_during_transition() {
        // Reversal: 50 bars up, then 50 bars down. During the transition
        // window, the two SuperTrends should disagree → Chop regime
        // surfaces at some point between bars 50 and 99.
        let mut bars: Vec<Bar> = (0..50)
            .map(|i| {
                let mid = 100.0 + i as f64;
                b(mid + 0.5, mid - 0.5, mid)
            })
            .collect();
        bars.extend((0..50).map(|i| {
            let mid = 150.0 - i as f64;
            b(mid + 0.5, mid - 0.5, mid)
        }));
        let r = compute(&bars, 7, 1.5, 21, 3.0);
        let any_chop = (50..100).any(|i| r.regime[i] == Some(Regime::Chop));
        assert!(
            any_chop,
            "expected at least one Chop regime during reversal"
        );
    }

    #[test]
    fn output_lengths_match_input() {
        let bars: Vec<_> = (0..50).map(|_| b(101.0, 99.0, 100.0)).collect();
        let r = compute(&bars, 7, 1.5, 21, 3.0);
        assert_eq!(r.fast_supertrend.len(), 50);
        assert_eq!(r.slow_supertrend.len(), 50);
        assert_eq!(r.regime.len(), 50);
    }
}
