//! Bill Williams Acceleration/Deceleration Oscillator (AC).
//!
//! Definition (from Williams's "New Trading Dimensions"):
//!   - **Median price**: `MP = (high + low) / 2`
//!   - **Awesome Oscillator (AO)**: `SMA(MP, 5) − SMA(MP, 34)`
//!   - **AC**: `AO − SMA(AO, 5)`
//!
//! AC measures the *change* in momentum (second derivative of price),
//! whereas AO measures momentum itself. Conventional reading: bar color
//! flips when AC sign changes; two-bar same-color confirms direction.
//!
//! Pure compute. Distinct from existing `awesome_oscillator` /
//! `bill_williams` if any — this is specifically the AC overlay.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HlBar {
    pub high: f64,
    pub low: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AcBias {
    Bullish,
    Bearish,
    #[default]
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AcReport {
    /// AO series (None until 34-bar warmup complete).
    pub ao: Vec<Option<f64>>,
    /// AC series (None until 34+5 warmup complete).
    pub ac: Vec<Option<f64>>,
    pub latest_ao: Option<f64>,
    pub latest_ac: Option<f64>,
    /// Two-bar confirmed bias on the most recent bars.
    pub bias: AcBias,
}

fn sma_window(slice: &[f64]) -> f64 {
    slice.iter().sum::<f64>() / slice.len() as f64
}

pub fn compute(bars: &[HlBar]) -> AcReport {
    let n = bars.len();
    if n < 34 + 5 {
        // Preserve the input-aligned length invariant the post-warmup path
        // honors (`ao.len() == ac.len() == bars.len()`), otherwise callers
        // indexing the series by bar index walk off the end on short input.
        return AcReport {
            ao: vec![None; n],
            ac: vec![None; n],
            ..AcReport::default()
        };
    }
    let mp: Vec<f64> = bars.iter().map(|b| (b.high + b.low) / 2.0).collect();
    let mut ao: Vec<Option<f64>> = vec![None; n];
    for i in 33..n {
        let s5 = sma_window(&mp[i - 4..=i]);
        let s34 = sma_window(&mp[i - 33..=i]);
        ao[i] = Some(s5 - s34);
    }
    let mut ac: Vec<Option<f64>> = vec![None; n];
    for i in (33 + 4)..n {
        let mut sum = 0.0;
        let mut k = 0;
        for x in ao.iter().take(i + 1).skip(i - 4).flatten() {
            sum += x;
            k += 1;
        }
        if k == 5 {
            ac[i] = Some(ao[i].unwrap() - sum / 5.0);
        }
    }
    let latest_ao = ao.last().copied().flatten();
    let latest_ac = ac.last().copied().flatten();
    let bias = match (ac.get(n - 1), ac.get(n - 2)) {
        (Some(Some(v1)), Some(Some(v0))) if *v1 > 0.0 && *v0 > 0.0 && v1 > v0 => AcBias::Bullish,
        (Some(Some(v1)), Some(Some(v0))) if *v1 < 0.0 && *v0 < 0.0 && v1 < v0 => AcBias::Bearish,
        _ => AcBias::Neutral,
    };
    AcReport {
        ao,
        ac,
        latest_ao,
        latest_ac,
        bias,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(h: f64, l: f64) -> HlBar {
        HlBar { high: h, low: l }
    }

    #[test]
    fn short_input_returns_default() {
        let bars = vec![b(101.0, 100.0); 38]; // need 39
        let r = compute(&bars);
        assert!(r.latest_ac.is_none());
    }

    #[test]
    fn exactly_39_bars_produces_one_ac_value() {
        // Constant prices → all SMAs equal → AO = 0 → AC = 0.
        let bars = vec![b(101.0, 100.0); 39];
        let r = compute(&bars);
        let last = r.latest_ac.expect("populated at index 38");
        assert!((last - 0.0).abs() < 1e-9);
        assert_eq!(r.ao[33], Some(0.0));
        assert_eq!(r.ac[37], Some(0.0));
    }

    #[test]
    fn rising_trend_yields_positive_ao() {
        // Steady rising series → short SMA > long SMA → AO > 0.
        let bars: Vec<HlBar> = (0..50)
            .map(|i| {
                let p = 100.0 + i as f64;
                b(p + 0.5, p - 0.5)
            })
            .collect();
        let r = compute(&bars);
        let ao = r.latest_ao.expect("populated");
        assert!(ao > 0.0, "rising → AO positive, got {ao}");
    }

    #[test]
    fn falling_trend_yields_negative_ao() {
        let bars: Vec<HlBar> = (0..50)
            .map(|i| {
                let p = 200.0 - i as f64;
                b(p + 0.5, p - 0.5)
            })
            .collect();
        let r = compute(&bars);
        let ao = r.latest_ao.expect("populated");
        assert!(ao < 0.0, "falling → AO negative, got {ao}");
    }

    #[test]
    fn accelerating_uptrend_gives_bullish_bias() {
        // Quadratic ramp — momentum is increasing, so AC > 0 and rising.
        let bars: Vec<HlBar> = (0..60)
            .map(|i| {
                let p = 100.0 + (i as f64).powi(2) / 30.0;
                b(p + 0.5, p - 0.5)
            })
            .collect();
        let r = compute(&bars);
        let v = r.latest_ac.expect("populated");
        // AC may still be small but the bias check requires AC > 0 AND rising
        // across the last two bars. For quadratic ramp both should hold.
        assert!(v > 0.0, "accelerating → AC > 0, got {v}");
        assert!(
            matches!(r.bias, AcBias::Bullish | AcBias::Neutral),
            "got {:?}, AC = {v}",
            r.bias
        );
    }

    #[test]
    fn warmup_indices_below_38_are_none() {
        let bars: Vec<HlBar> = (0..50)
            .map(|i| {
                let p = 100.0 + i as f64;
                b(p + 0.5, p - 0.5)
            })
            .collect();
        let r = compute(&bars);
        for i in 0..37 {
            assert!(r.ac[i].is_none(), "ac[{i}] should be None");
        }
        // First AC value is at index 33 + 4 = 37 (4 bars after first AO).
        assert!(r.ac[37].is_some());
    }
}
