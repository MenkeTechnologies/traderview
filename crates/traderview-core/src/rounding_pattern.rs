//! Rounding Top / Bottom (Saucer) Pattern Detector — Bulkowski.
//!
//! A long, U-shaped (or inverted-U) consolidation lacking sharp pivot
//! highs or lows. Detected by fitting a quadratic y = a·x² + b·x + c
//! to a window of closes and requiring:
//!
//!   - Significant quadratic coefficient (|a| above threshold)
//!   - Sign of a determines top (a < 0) vs bottom (a > 0)
//!   - High R² of the quadratic fit (price actually follows the curve)
//!
//! Different from `cup_and_handle` (which has a specific handle phase)
//! and `triple_top_bottom` (which requires three pivots).
//!
//! Pure compute. Companion to `cup_and_handle`, `triple_top_bottom`,
//! `bump_and_run`, `diamond_pattern`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternKind { SaucerTop, SaucerBottom }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaucerCandidate {
    pub kind: PatternKind,
    pub start_index: usize,
    pub end_index: usize,
    pub quadratic_a: f64,
    pub r_squared: f64,
    pub apex_index_relative: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub window: usize,
    pub min_abs_quadratic_coef: f64,
    pub min_r_squared: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            window: 80,
            min_abs_quadratic_coef: 1e-5,
            min_r_squared: 0.80,
        }
    }
}

pub fn detect(closes: &[f64], cfg: &Config) -> Vec<SaucerCandidate> {
    let n = closes.len();
    let mut out = Vec::new();
    if n < cfg.window || cfg.window < 10
        || !cfg.min_abs_quadratic_coef.is_finite()
        || !cfg.min_r_squared.is_finite() {
        return out;
    }
    if closes.iter().any(|x| !x.is_finite()) { return out; }
    for start in 0..=(n - cfg.window) {
        let end = start + cfg.window;
        let slice = &closes[start..end];
        let (a, _b, _c, r_sq) = quadratic_fit(slice);
        if a.abs() < cfg.min_abs_quadratic_coef || r_sq < cfg.min_r_squared { continue; }
        let kind = if a < 0.0 { PatternKind::SaucerTop } else { PatternKind::SaucerBottom };
        // Apex at vertex: x_v = −b / (2a). Convert to integer index inside window.
        let apex_rel = ((-_b / (2.0 * a)).round() as i64)
            .clamp(0, (cfg.window - 1) as i64) as usize;
        out.push(SaucerCandidate {
            kind,
            start_index: start,
            end_index: end - 1,
            quadratic_a: a,
            r_squared: r_sq,
            apex_index_relative: apex_rel,
        });
    }
    out
}

/// OLS quadratic fit y = a·x² + b·x + c. Returns (a, b, c, R²).
fn quadratic_fit(y: &[f64]) -> (f64, f64, f64, f64) {
    let n = y.len();
    let n_f = n as f64;
    let mut sx = 0.0_f64;
    let mut sx2 = 0.0_f64;
    let mut sx3 = 0.0_f64;
    let mut sx4 = 0.0_f64;
    let mut sy = 0.0_f64;
    let mut sxy = 0.0_f64;
    let mut sx2y = 0.0_f64;
    for (i, yi) in y.iter().enumerate() {
        let x = i as f64;
        let x2 = x * x;
        sx += x;
        sx2 += x2;
        sx3 += x2 * x;
        sx4 += x2 * x2;
        sy += yi;
        sxy += x * yi;
        sx2y += x2 * yi;
    }
    // Normal equations: M · [c, b, a]ᵀ = [sy, sxy, sx2y]ᵀ
    let m = [
        [n_f, sx, sx2],
        [sx, sx2, sx3],
        [sx2, sx3, sx4],
    ];
    let rhs = [sy, sxy, sx2y];
    let det = m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    if det.abs() < 1e-18 { return (0.0, 0.0, sy / n_f, 0.0); }
    // Cramer's rule.
    let det_a = m[0][0] * (m[1][1] * rhs[2] - rhs[1] * m[2][1])
        - m[0][1] * (m[1][0] * rhs[2] - rhs[1] * m[2][0])
        + rhs[0] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    let det_b = m[0][0] * (rhs[1] * m[2][2] - m[1][2] * rhs[2])
        - rhs[0] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * rhs[2] - rhs[1] * m[2][0]);
    let det_c = rhs[0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (rhs[1] * m[2][2] - m[1][2] * rhs[2])
        + m[0][2] * (rhs[1] * m[2][1] - m[1][1] * rhs[2]);
    let a = det_a / det;
    let b = det_b / det;
    let c = det_c / det;
    // R²
    let y_mean = sy / n_f;
    let tss: f64 = y.iter().map(|yi| (yi - y_mean).powi(2)).sum();
    let ssr: f64 = y.iter().enumerate().map(|(i, yi)| {
        let xi = i as f64;
        let yhat = a * xi * xi + b * xi + c;
        (yi - yhat).powi(2)
    }).sum();
    let r_sq = if tss > 1e-18 { 1.0 - ssr / tss } else { 0.0 };
    (a, b, c, r_sq.clamp(0.0, 1.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_or_short_returns_empty() {
        assert!(detect(&[], &Config::default()).is_empty());
        let c = vec![100.0_f64; 50];
        assert!(detect(&c, &Config::default()).is_empty());
    }

    #[test]
    fn nan_returns_empty() {
        let mut c = vec![100.0_f64; 100];
        c[5] = f64::NAN;
        assert!(detect(&c, &Config::default()).is_empty());
    }

    #[test]
    fn linear_trend_no_saucer() {
        // Linear → quadratic coefficient ~0 → no saucer.
        let c: Vec<f64> = (0..100).map(|i| 100.0 + i as f64).collect();
        let cands = detect(&c, &Config::default());
        // a should be near zero so no candidates emitted.
        assert!(cands.is_empty());
    }

    #[test]
    fn parabolic_top_detected() {
        // Inverted-U: y = -0.01·(i - 40)² + 116. Min at i=0 and i=80; max at i=40.
        let c: Vec<f64> = (0..100).map(|i| {
            let x = i as f64 - 40.0;
            -0.01 * x * x + 116.0
        }).collect();
        let cands = detect(&c, &Config::default());
        assert!(cands.iter().any(|x| x.kind == PatternKind::SaucerTop),
            "expected saucer top, got {cands:?}");
    }

    #[test]
    fn parabolic_bottom_detected() {
        let c: Vec<f64> = (0..100).map(|i| {
            let x = i as f64 - 40.0;
            0.01 * x * x + 100.0
        }).collect();
        let cands = detect(&c, &Config::default());
        assert!(cands.iter().any(|x| x.kind == PatternKind::SaucerBottom));
    }

    #[test]
    fn apex_near_vertex() {
        // Vertex at i=40 in the input. Find candidate starting at 0 (or near it).
        let c: Vec<f64> = (0..100).map(|i| {
            let x = i as f64 - 40.0;
            -0.01 * x * x + 116.0
        }).collect();
        let cands = detect(&c, &Config { window: 80, ..Default::default() });
        if let Some(c0) = cands.first() {
            // Window 0..79, apex ~40 in that window.
            assert!((c0.apex_index_relative as i64 - 40).abs() < 10);
        }
    }

    #[test]
    fn high_min_r_squared_filters_noisy_input() {
        let mut state: u64 = 42;
        let c: Vec<f64> = (0..100).map(|_| {
            state = state.wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            100.0 + ((state >> 32) as f64 / u32::MAX as f64 - 0.5) * 50.0
        }).collect();
        let cands = detect(&c, &Config { min_r_squared: 0.95, ..Default::default() });
        assert!(cands.is_empty(),
            "random noise shouldn't fit a clean parabola at R² ≥ 0.95");
    }
}
