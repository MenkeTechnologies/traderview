//! Technical indicators — pure compute over `&[PriceBar]` (or `&[f64]` closes).
//!
//! All series-out functions return a `Vec<Option<f64>>` aligned with the input
//! length; entries before the indicator has enough data are `None`.

use crate::models::PriceBar;
use serde::Serialize;

/// Convenience: pull closes out of bars.
pub fn closes(bars: &[PriceBar]) -> Vec<f64> {
    bars.iter().map(|b| dec_f64(b.close)).collect()
}
pub fn highs(bars: &[PriceBar]) -> Vec<f64> {
    bars.iter().map(|b| dec_f64(b.high)).collect()
}
pub fn lows(bars: &[PriceBar]) -> Vec<f64> {
    bars.iter().map(|b| dec_f64(b.low)).collect()
}
pub fn volumes(bars: &[PriceBar]) -> Vec<f64> {
    bars.iter().map(|b| dec_f64(b.volume)).collect()
}

fn dec_f64(d: rust_decimal::Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}

// ===========================================================================
// Moving averages
// ===========================================================================

pub fn sma(values: &[f64], window: usize) -> Vec<Option<f64>> {
    let n = values.len();
    let mut out = vec![None; n];
    if window == 0 || window > n {
        return out;
    }
    let mut sum = 0.0;
    for i in 0..n {
        sum += values[i];
        if i >= window {
            sum -= values[i - window];
        }
        if i + 1 >= window {
            out[i] = Some(sum / window as f64);
        }
    }
    out
}

/// Wilder / EMA smoothing with alpha = 2/(n+1).
pub fn ema(values: &[f64], window: usize) -> Vec<Option<f64>> {
    let n = values.len();
    let mut out = vec![None; n];
    if window == 0 || n < window {
        return out;
    }
    let alpha = 2.0 / (window as f64 + 1.0);
    // Seed with SMA of the first `window` values.
    let seed: f64 = values[..window].iter().sum::<f64>() / window as f64;
    out[window - 1] = Some(seed);
    let mut prev = seed;
    for i in window..n {
        prev = alpha * values[i] + (1.0 - alpha) * prev;
        out[i] = Some(prev);
    }
    out
}

// ===========================================================================
// MACD (12, 26, 9)
// ===========================================================================

#[derive(Debug, Clone, Serialize)]
pub struct Macd {
    pub line: Vec<Option<f64>>,
    pub signal: Vec<Option<f64>>,
    pub histogram: Vec<Option<f64>>,
}

pub fn macd(closes: &[f64], fast: usize, slow: usize, signal: usize) -> Macd {
    let ef = ema(closes, fast);
    let es = ema(closes, slow);
    let line: Vec<Option<f64>> = ef
        .iter()
        .zip(es.iter())
        .map(|(a, b)| match (a, b) {
            (Some(a), Some(b)) => Some(a - b),
            _ => None,
        })
        .collect();
    // EMA of the line ignoring leading Nones.
    let line_compact: Vec<f64> = line.iter().filter_map(|x| *x).collect();
    let sig_compact = ema(&line_compact, signal);
    let mut sig: Vec<Option<f64>> = vec![None; closes.len()];
    let offset = closes.len() - line_compact.len();
    for (i, v) in sig_compact.iter().enumerate() {
        sig[offset + i] = *v;
    }
    let histogram: Vec<Option<f64>> = line
        .iter()
        .zip(sig.iter())
        .map(|(l, s)| match (l, s) {
            (Some(l), Some(s)) => Some(l - s),
            _ => None,
        })
        .collect();
    Macd {
        line,
        signal: sig,
        histogram,
    }
}

// ===========================================================================
// RSI (Wilder smoothing)
// ===========================================================================

pub fn rsi(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period == 0 || n <= period {
        return out;
    }
    let mut gain = 0.0;
    let mut loss = 0.0;
    for i in 1..=period {
        let d = closes[i] - closes[i - 1];
        if d >= 0.0 {
            gain += d;
        } else {
            loss += -d;
        }
    }
    gain /= period as f64;
    loss /= period as f64;
    out[period] = Some(rsi_from(gain, loss));
    for i in (period + 1)..n {
        let d = closes[i] - closes[i - 1];
        let (g, l) = if d >= 0.0 { (d, 0.0) } else { (0.0, -d) };
        gain = (gain * (period as f64 - 1.0) + g) / period as f64;
        loss = (loss * (period as f64 - 1.0) + l) / period as f64;
        out[i] = Some(rsi_from(gain, loss));
    }
    out
}
fn rsi_from(gain: f64, loss: f64) -> f64 {
    if loss == 0.0 {
        return 100.0;
    }
    let rs = gain / loss;
    100.0 - 100.0 / (1.0 + rs)
}

// ===========================================================================
// ADX / +DI / -DI (Wilder)
// ===========================================================================

#[derive(Debug, Clone, Serialize)]
pub struct Adx {
    pub plus_di: Vec<Option<f64>>,
    pub minus_di: Vec<Option<f64>>,
    pub adx: Vec<Option<f64>>,
}

pub fn adx(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> Adx {
    let n = highs.len();
    let mut plus_dm = vec![0.0; n];
    let mut minus_dm = vec![0.0; n];
    let mut tr = vec![0.0; n];
    for i in 1..n {
        let up = highs[i] - highs[i - 1];
        let down = lows[i - 1] - lows[i];
        plus_dm[i] = if up > down && up > 0.0 { up } else { 0.0 };
        minus_dm[i] = if down > up && down > 0.0 { down } else { 0.0 };
        tr[i] = (highs[i] - lows[i])
            .max((highs[i] - closes[i - 1]).abs())
            .max((lows[i] - closes[i - 1]).abs());
    }
    let smoothed = |v: &[f64]| -> Vec<Option<f64>> { wilder_smooth(v, period) };
    let pdi_smooth = smoothed(&plus_dm);
    let mdi_smooth = smoothed(&minus_dm);
    let tr_smooth = smoothed(&tr);
    let mut plus_di = vec![None; n];
    let mut minus_di = vec![None; n];
    let mut dx = vec![None; n];
    for i in 0..n {
        if let (Some(p), Some(m), Some(t)) = (pdi_smooth[i], mdi_smooth[i], tr_smooth[i]) {
            if t > 0.0 {
                let pd = 100.0 * p / t;
                let md = 100.0 * m / t;
                plus_di[i] = Some(pd);
                minus_di[i] = Some(md);
                if pd + md > 0.0 {
                    dx[i] = Some(100.0 * (pd - md).abs() / (pd + md));
                }
            }
        }
    }
    let adx_series = wilder_smooth_optional(&dx, period);
    Adx {
        plus_di,
        minus_di,
        adx: adx_series,
    }
}

fn wilder_smooth(values: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = values.len();
    let mut out = vec![None; n];
    if period == 0 || n < period + 1 {
        return out;
    }
    let mut sum: f64 = values[1..=period].iter().sum();
    out[period] = Some(sum);
    for i in (period + 1)..n {
        sum = sum - sum / period as f64 + values[i];
        out[i] = Some(sum);
    }
    out
}

fn wilder_smooth_optional(values: &[Option<f64>], period: usize) -> Vec<Option<f64>> {
    let n = values.len();
    let mut out = vec![None; n];
    // Find first index where we have `period` consecutive Somes.
    let mut start = None;
    for i in 0..n {
        if values[i].is_none() {
            start = None;
            continue;
        }
        if start.is_none() {
            start = Some(i);
        }
        if let Some(s) = start {
            if i + 1 - s >= period {
                let sum: f64 = values[s..=i].iter().map(|x| x.unwrap()).sum();
                out[i] = Some(sum / period as f64);
                // Now smooth forward.
                let mut prev = out[i].unwrap();
                for j in (i + 1)..n {
                    if let Some(v) = values[j] {
                        prev = (prev * (period as f64 - 1.0) + v) / period as f64;
                        out[j] = Some(prev);
                    } else {
                        break;
                    }
                }
                break;
            }
        }
    }
    out
}

// ===========================================================================
// Stochastic %K %D
// ===========================================================================

#[derive(Debug, Clone, Serialize)]
pub struct Stoch {
    pub k: Vec<Option<f64>>,
    pub d: Vec<Option<f64>>,
}

pub fn stochastic(
    highs: &[f64],
    lows: &[f64],
    closes: &[f64],
    k_period: usize,
    d_period: usize,
) -> Stoch {
    let n = highs.len();
    let mut k = vec![None; n];
    for i in 0..n {
        if i + 1 < k_period {
            continue;
        }
        let lo = lows[(i + 1 - k_period)..=i]
            .iter()
            .cloned()
            .fold(f64::INFINITY, f64::min);
        let hi = highs[(i + 1 - k_period)..=i]
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        if hi - lo > 0.0 {
            k[i] = Some(100.0 * (closes[i] - lo) / (hi - lo));
        }
    }
    // %D is the SMA of %K. The previous compact-then-shift implementation
    // dropped Nones and then offset-rewrote, which silently produced wrong
    // values whenever %K had a None mid-series (e.g., a zero-range bar
    // setting k[i]=None). Do an Option-aware window scan instead.
    let mut d = vec![None; n];
    if d_period > 0 && d_period <= n {
        for i in (d_period - 1)..n {
            let window = &k[i + 1 - d_period..=i];
            if let Some(sum) = window.iter().try_fold(0.0_f64, |s, x| x.map(|v| s + v)) {
                d[i] = Some(sum / d_period as f64);
            }
        }
    }
    Stoch { k, d }
}

// ===========================================================================
// Bollinger bands (period, multiplier)
// ===========================================================================

#[derive(Debug, Clone, Serialize)]
pub struct Bollinger {
    pub middle: Vec<Option<f64>>,
    pub upper: Vec<Option<f64>>,
    pub lower: Vec<Option<f64>>,
}

pub fn bollinger(closes: &[f64], period: usize, k: f64) -> Bollinger {
    let m = sma(closes, period);
    let n = closes.len();
    let mut upper = vec![None; n];
    let mut lower = vec![None; n];
    for i in 0..n {
        if let Some(mid) = m[i] {
            if i + 1 >= period {
                let window = &closes[(i + 1 - period)..=i];
                let var = window.iter().map(|x| (x - mid).powi(2)).sum::<f64>() / period as f64;
                let sd = var.sqrt();
                upper[i] = Some(mid + k * sd);
                lower[i] = Some(mid - k * sd);
            }
        }
    }
    Bollinger {
        middle: m,
        upper,
        lower,
    }
}

// ===========================================================================
// Pivot points (classic, using prior period's HLC)
// ===========================================================================

#[derive(Debug, Clone, Serialize)]
pub struct Pivots {
    pub pivot: f64,
    pub r1: f64,
    pub r2: f64,
    pub r3: f64,
    pub s1: f64,
    pub s2: f64,
    pub s3: f64,
}

pub fn classic_pivots(high: f64, low: f64, close: f64) -> Pivots {
    let p = (high + low + close) / 3.0;
    Pivots {
        pivot: p,
        r1: 2.0 * p - low,
        s1: 2.0 * p - high,
        r2: p + (high - low),
        s2: p - (high - low),
        r3: high + 2.0 * (p - low),
        s3: low - 2.0 * (high - p),
    }
}

// ===========================================================================
// ATR — Average True Range (Wilder smoothing).
// ===========================================================================

/// Returns ATR aligned with the input length. The first `period` slots are
/// `None`; the seed at index `period` is the simple average of the first
/// `period` true-range values; subsequent values use Wilder smoothing.
pub fn atr(highs: &[f64], lows: &[f64], closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period == 0 || n < period + 1 || highs.len() != n || lows.len() != n {
        return out;
    }
    let tr = |i: usize| -> f64 {
        let prev_close = closes[i - 1];
        let a = highs[i] - lows[i];
        let b = (highs[i] - prev_close).abs();
        let c = (lows[i] - prev_close).abs();
        a.max(b).max(c)
    };
    let mut sum = 0.0;
    for i in 1..=period { sum += tr(i); }
    let p = period as f64;
    let mut prev = sum / p;
    out[period] = Some(prev);
    for (i, slot) in out.iter_mut().enumerate().take(n).skip(period + 1) {
        prev = (prev * (p - 1.0) + tr(i)) / p;
        *slot = Some(prev);
    }
    out
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sma_basic() {
        let v = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let s = sma(&v, 3);
        assert_eq!(s, vec![None, None, Some(2.0), Some(3.0), Some(4.0)]);
    }

    #[test]
    fn ema_seed_is_sma() {
        let v = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let e = ema(&v, 3);
        assert_eq!(e[2], Some(2.0));
        assert!(e[3].is_some() && e[4].is_some());
    }

    #[test]
    fn rsi_all_gains_yields_100() {
        let v: Vec<f64> = (1..=20).map(|x| x as f64).collect();
        let r = rsi(&v, 14);
        assert_eq!(r[19], Some(100.0));
    }

    #[test]
    fn stochastic_at_high_is_100() {
        let highs: Vec<f64> = (1..=10).map(|x| x as f64).collect();
        let lows: Vec<f64> = (1..=10).map(|x| x as f64 - 0.5).collect();
        let closes: Vec<f64> = (1..=10).map(|x| x as f64).collect();
        let s = stochastic(&highs, &lows, &closes, 5, 3);
        assert!(s.k[9].unwrap() > 90.0);
    }

    #[test]
    fn stochastic_d_handles_mid_series_none_in_k() {
        // %K is None when the *entire k-window* has zero range
        // (max(highs)==min(lows) across the window). Construct a 3-bar
        // flat patch at indices 5-7 so the k_period=3 window at i=7 is
        // fully flat → k[7]=None. The previous compact-then-shift impl
        // dropped k[7] from k_vals and silently produced %D values at
        // indices 8 and 9 that read across the missing slot, yielding
        // wrong numbers. After the fix, any d[i] whose window contains
        // k[7] must be None.
        let highs  = vec![10.0, 11.0, 12.0, 13.0, 14.0, 7.0, 7.0, 7.0, 18.0, 19.0];
        let lows   = vec![ 5.0,  6.0,  7.0,  8.0,  9.0, 7.0, 7.0, 7.0, 13.0, 14.0];
        let closes = vec![ 7.0,  8.0,  9.0, 10.0, 11.0, 7.0, 7.0, 7.0, 15.0, 16.0];
        let s = stochastic(&highs, &lows, &closes, 3, 3);
        // Only the i=7 window [5,6,7] is fully flat → only k[7] is None.
        assert!(s.k[5].is_some());
        assert!(s.k[6].is_some());
        assert!(s.k[7].is_none(), "fully-flat window must yield k=None at i=7");
        assert!(s.k[8].is_some());
        assert!(s.k[9].is_some());
        // d windows containing k[7]: d[7] = [5,6,7], d[8] = [6,7,8], d[9] = [7,8,9].
        assert!(s.d[7].is_none(), "d[7] window includes k[7]=None");
        assert!(s.d[8].is_none(), "d[8] window includes k[7]=None");
        assert!(s.d[9].is_none(), "d[9] window includes k[7]=None");
        // d[5] and d[6] don't touch index 7 and have full Some windows → Some.
        assert!(s.d[5].is_some());
        assert!(s.d[6].is_some());
    }

    #[test]
    fn pivots_basic() {
        let p = classic_pivots(110.0, 90.0, 100.0);
        assert_eq!(p.pivot, 100.0);
        assert_eq!(p.r1, 110.0);
        assert_eq!(p.s1, 90.0);
    }

    #[test]
    fn atr_returns_none_until_period_then_wilder_smooths() {
        // Flat HLC bars where high-low = 2 and close sits inside the range.
        // Every TR collapses to (high - low) = 2.0, so ATR converges to 2.0.
        let h = vec![2.0; 15];
        let l = vec![0.0; 15];
        let c = vec![1.0; 15];
        let a = atr(&h, &l, &c, 5);
        for v in a.iter().take(5) { assert!(v.is_none()); }
        let seed = a[5].expect("seed at period index");
        assert!((seed - 2.0).abs() < 1e-9, "constant TR of 2.0 must yield ATR=2.0");
        let tail = a[14].expect("smoothed tail");
        assert!((tail - 2.0).abs() < 1e-9);
    }

    #[test]
    fn atr_rejects_mismatched_input_lengths() {
        // Length mismatch must return all-None rather than panic.
        let h = vec![1.0, 2.0, 3.0];
        let l = vec![0.5, 1.5];
        let c = vec![0.7, 1.7, 2.7];
        let a = atr(&h, &l, &c, 2);
        assert!(a.iter().all(|v| v.is_none()));
    }
}
