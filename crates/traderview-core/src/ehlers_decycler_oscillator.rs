//! Ehlers Decycler Oscillator (DSO) — John Ehlers, follow-up to the
//! Decycler filter.
//!
//! Two-pass 2-pole high-pass that isolates the oscillatory component
//! (cycles shorter than `hp_period`) and discards drift longer than it:
//!
//!   α = (cos(0.707·2π/hp) + sin(0.707·2π/hp) - 1) / cos(0.707·2π/hp)
//!   HP_t = (1 - α/2)² · (x_t - 2·x_{t-1} + x_{t-2})
//!          + 2·(1 - α)·HP_{t-1}
//!          - (1 - α)²·HP_{t-2}
//!
//! Output is centered near zero. Zero-line crosses signal cycle
//! turning points; magnitude reflects amplitude of the dominant
//! cycle band.
//!
//! Pure compute. Default hp_period = 30. Companion to `ehlers_decycler`
//! (low-pass), `roofing_filter` (HP→Super-Smoother cascade),
//! `ehlers_instant_trendline`.

pub fn compute(series: &[f64], hp_period: usize) -> Vec<Option<f64>> {
    let n = series.len();
    let mut out = vec![None; n];
    if n < 3 || hp_period < 4 {
        return out;
    }
    if series.iter().any(|x| !x.is_finite()) {
        return out;
    }
    let pi = std::f64::consts::PI;
    let arg = 0.707 * 2.0 * pi / hp_period as f64;
    let cos_arg = arg.cos();
    let alpha = (cos_arg + arg.sin() - 1.0) / cos_arg;
    let one_minus_a = 1.0 - alpha;
    let c1 = (1.0 - alpha / 2.0).powi(2);
    let c2 = 2.0 * one_minus_a;
    let c3 = one_minus_a * one_minus_a;
    let mut hp = vec![0.0_f64; n];
    for i in 2..n {
        hp[i] = c1 * (series[i] - 2.0 * series[i - 1] + series[i - 2]) + c2 * hp[i - 1]
            - c3 * hp[i - 2];
        out[i] = Some(hp[i]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let s = vec![100.0_f64; 50];
        assert!(compute(&s, 3).iter().all(|x| x.is_none()));
        assert!(compute(&s[..2], 30).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut s = vec![100.0_f64; 50];
        s[5] = f64::NAN;
        assert!(compute(&s, 30).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_signal_settles_to_zero() {
        let s = vec![100.0_f64; 200];
        let r = compute(&s, 30);
        for v in r.iter().skip(50).flatten() {
            assert!(v.abs() < 1e-6);
        }
    }

    #[test]
    fn linear_trend_removed() {
        // 2-pole HP removes both DC and linear trends.
        let s: Vec<f64> = (0..200).map(|i| i as f64).collect();
        let r = compute(&s, 30);
        let vals: Vec<f64> = r.iter().skip(50).flatten().copied().collect();
        let mean: f64 = vals.iter().sum::<f64>() / vals.len() as f64;
        assert!(
            mean.abs() < 1.0,
            "linear trend filtered, mean ≈ 0, got {mean}"
        );
    }

    #[test]
    fn sin_wave_passes_through_within_band() {
        // Period-15 sine: shorter than hp_period=30 → HP passes it through.
        let s: Vec<f64> = (0..400)
            .map(|i| 100.0 + (i as f64 * 2.0 * std::f64::consts::PI / 15.0).sin())
            .collect();
        let r = compute(&s, 30);
        let vals: Vec<f64> = r.iter().skip(100).flatten().copied().collect();
        let amp_max = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let amp_min = vals.iter().cloned().fold(f64::INFINITY, f64::min);
        // Output amplitude should reflect at least part of the 1.0 input amplitude.
        assert!(
            amp_max - amp_min > 0.5,
            "in-band sine should pass through with substantial amplitude, got {}-{}",
            amp_min,
            amp_max
        );
    }

    #[test]
    fn output_length_matches_input() {
        let s = vec![100.0_f64; 50];
        assert_eq!(compute(&s, 30).len(), 50);
    }
}
