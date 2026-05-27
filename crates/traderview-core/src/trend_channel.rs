//! Trend-channel detector.
//!
//! Fits two parallel lines to recent swing highs (upper channel) and
//! swing lows (lower channel) using OLS. Channel slope = average of
//! the two; intercepts captured separately so the user can see the
//! current upper/lower projections.
//!
//! Pure compute. Caller provides the swing-point list (typically from
//! crate::swing_points).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct SwingPoint {
    pub bar_index: usize,
    pub price: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ChannelReport {
    pub slope: f64,
    pub upper_intercept: f64,
    pub lower_intercept: f64,
    pub channel_width_at_last_bar: f64,
    pub current_upper: f64,
    pub current_lower: f64,
}

pub fn fit(swings_high: &[SwingPoint], swings_low: &[SwingPoint], last_bar: usize)
    -> Option<ChannelReport>
{
    if swings_high.len() < 2 || swings_low.len() < 2 { return None; }
    let (slope_h, intercept_h) = ols_fit(swings_high)?;
    let (slope_l, intercept_l) = ols_fit(swings_low)?;
    // Average the two slopes to enforce parallel channel.
    let slope = (slope_h + slope_l) / 2.0;
    // Re-fit intercepts using the averaged slope:
    //   intercept = mean(price) - slope × mean(bar_index)
    let h_mean_x = swings_high.iter().map(|s| s.bar_index as f64).sum::<f64>() / swings_high.len() as f64;
    let h_mean_y = swings_high.iter().map(|s| s.price).sum::<f64>() / swings_high.len() as f64;
    let upper_intercept = h_mean_y - slope * h_mean_x;
    let l_mean_x = swings_low.iter().map(|s| s.bar_index as f64).sum::<f64>() / swings_low.len() as f64;
    let l_mean_y = swings_low.iter().map(|s| s.price).sum::<f64>() / swings_low.len() as f64;
    let lower_intercept = l_mean_y - slope * l_mean_x;
    let current_upper = upper_intercept + slope * last_bar as f64;
    let current_lower = lower_intercept + slope * last_bar as f64;
    let _ = intercept_h;
    let _ = intercept_l;
    Some(ChannelReport {
        slope,
        upper_intercept,
        lower_intercept,
        channel_width_at_last_bar: current_upper - current_lower,
        current_upper,
        current_lower,
    })
}

fn ols_fit(points: &[SwingPoint]) -> Option<(f64, f64)> {
    let n = points.len() as f64;
    if n < 2.0 { return None; }
    let mean_x = points.iter().map(|p| p.bar_index as f64).sum::<f64>() / n;
    let mean_y = points.iter().map(|p| p.price).sum::<f64>() / n;
    let mut num = 0.0;
    let mut den = 0.0;
    for p in points {
        let dx = p.bar_index as f64 - mean_x;
        num += dx * (p.price - mean_y);
        den += dx * dx;
    }
    if den == 0.0 { return None; }
    let slope = num / den;
    let intercept = mean_y - slope * mean_x;
    Some((slope, intercept))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(idx: usize, price: f64) -> SwingPoint {
        SwingPoint { bar_index: idx, price }
    }

    #[test]
    fn under_two_swings_each_none() {
        assert!(fit(&[p(0, 100.0)], &[p(0, 90.0)], 50).is_none());
    }

    #[test]
    fn horizontal_channel_slope_zero() {
        let h = vec![p(0, 110.0), p(50, 110.0), p(100, 110.0)];
        let l = vec![p(0, 100.0), p(50, 100.0), p(100, 100.0)];
        let r = fit(&h, &l, 100).unwrap();
        assert!(r.slope.abs() < 1e-9);
        assert!((r.current_upper - 110.0).abs() < 1e-9);
        assert!((r.current_lower - 100.0).abs() < 1e-9);
    }

    #[test]
    fn upward_sloping_channel_positive_slope() {
        let h = vec![p(0, 110.0), p(100, 130.0)];
        let l = vec![p(0, 100.0), p(100, 120.0)];
        let r = fit(&h, &l, 100).unwrap();
        assert!(r.slope > 0.0);
        // Slope = 20/100 = 0.2.
        assert!((r.slope - 0.2).abs() < 1e-9);
    }

    #[test]
    fn downward_sloping_channel_negative_slope() {
        let h = vec![p(0, 130.0), p(100, 110.0)];
        let l = vec![p(0, 120.0), p(100, 100.0)];
        let r = fit(&h, &l, 100).unwrap();
        assert!(r.slope < 0.0);
    }

    #[test]
    fn channel_width_correct_at_last_bar() {
        let h = vec![p(0, 110.0), p(100, 130.0)];
        let l = vec![p(0, 100.0), p(100, 120.0)];
        let r = fit(&h, &l, 100).unwrap();
        // Width = 10 (130-120).
        assert!((r.channel_width_at_last_bar - 10.0).abs() < 1e-9);
    }

    #[test]
    fn parallel_channel_enforced_even_with_diverging_swings() {
        // Upper swings slope +0.5, lower swings slope -0.5 → averaged 0.
        let h = vec![p(0, 110.0), p(100, 160.0)];
        let l = vec![p(0, 100.0), p(100, 50.0)];
        let r = fit(&h, &l, 100).unwrap();
        // Slope = (0.5 + -0.5) / 2 = 0.
        assert!(r.slope.abs() < 1e-9);
    }

    #[test]
    fn channel_extrapolation_works_beyond_last_swing() {
        let h = vec![p(0, 110.0), p(100, 130.0)];
        let l = vec![p(0, 100.0), p(100, 120.0)];
        // Extrapolate to bar 200. Slope=0.2, upper_intercept = 120 - 0.2×50 = 110.
        // current_upper at bar 200 = 110 + 0.2 × 200 = 150.
        let r = fit(&h, &l, 200).unwrap();
        assert!((r.current_upper - 150.0).abs() < 1e-9);
        // lower_intercept = 110 - 0.2×50 = 100. current_lower = 100 + 40 = 140.
        assert!((r.current_lower - 140.0).abs() < 1e-9);
    }
}
