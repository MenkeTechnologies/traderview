//! Correlation regime detector — rolling Pearson correlation between
//! two return series with regime classification and break detection.
//!
//! Regimes by rolling-window correlation ρ:
//!   coupled    ρ ≥ +0.5
//!   neutral    −0.5 < ρ < +0.5
//!   inverse    ρ ≤ −0.5
//!
//! A BREAK is any bar where the regime label changes — the moments a
//! hedge stops hedging (SPY/TLT flipping from inverse to coupled is
//! the classic risk-parity pain trade). Reports the rolling series,
//! current/mean correlation, time-in-regime percentages, and the break
//! list.
//!
//! Pure compute over two pre-aligned close series.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Coupled,
    Neutral,
    Inverse,
}

const COUPLE_THRESHOLD: f64 = 0.5;

fn classify(rho: f64) -> Regime {
    if rho >= COUPLE_THRESHOLD {
        Regime::Coupled
    } else if rho <= -COUPLE_THRESHOLD {
        Regime::Inverse
    } else {
        Regime::Neutral
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct RegimeBreak {
    /// Index into the rolling series (bar offset within the aligned
    /// input, first window end = index 0).
    pub at: usize,
    pub from: Regime,
    pub to: Regime,
    pub correlation: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CorrelationRegimeReport {
    pub window: usize,
    /// One rolling ρ per bar from the first complete window.
    pub rolling: Vec<f64>,
    pub current: f64,
    pub current_regime: Regime,
    pub mean: f64,
    pub pct_coupled: f64,
    pub pct_neutral: f64,
    pub pct_inverse: f64,
    pub breaks: Vec<RegimeBreak>,
}

fn pearson(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len() as f64;
    let ma = a.iter().sum::<f64>() / n;
    let mb = b.iter().sum::<f64>() / n;
    let mut cov = 0.0;
    let mut va = 0.0;
    let mut vb = 0.0;
    for (x, y) in a.iter().zip(b) {
        cov += (x - ma) * (y - mb);
        va += (x - ma).powi(2);
        vb += (y - mb).powi(2);
    }
    if va <= 0.0 || vb <= 0.0 {
        0.0
    } else {
        cov / (va.sqrt() * vb.sqrt())
    }
}

/// `closes_a` and `closes_b` must be pre-aligned (same bar per index).
pub fn compute(
    closes_a: &[f64],
    closes_b: &[f64],
    window: usize,
) -> Option<CorrelationRegimeReport> {
    let n = closes_a.len().min(closes_b.len());
    if window < 3 || n < window + 1 {
        return None;
    }
    let ret = |c: &[f64]| -> Vec<f64> {
        c.windows(2)
            .map(|w| {
                if w[0] > 0.0 && w[1] > 0.0 && w[0].is_finite() && w[1].is_finite() {
                    (w[1] / w[0]).ln()
                } else {
                    0.0
                }
            })
            .collect()
    };
    let ra = ret(&closes_a[..n]);
    let rb = ret(&closes_b[..n]);
    let rolling: Vec<f64> = (0..=ra.len() - window)
        .map(|i| pearson(&ra[i..i + window], &rb[i..i + window]))
        .collect();
    let current = *rolling.last().expect("n >= window + 1 guarantees a window");
    let mean = rolling.iter().sum::<f64>() / rolling.len() as f64;
    let mut counts = [0usize; 3];
    let mut breaks = Vec::new();
    let mut prev: Option<Regime> = None;
    for (i, &rho) in rolling.iter().enumerate() {
        let regime = classify(rho);
        counts[match regime {
            Regime::Coupled => 0,
            Regime::Neutral => 1,
            Regime::Inverse => 2,
        }] += 1;
        if let Some(p) = prev {
            if p != regime {
                breaks.push(RegimeBreak {
                    at: i,
                    from: p,
                    to: regime,
                    correlation: rho,
                });
            }
        }
        prev = Some(regime);
    }
    let total = rolling.len() as f64;
    Some(CorrelationRegimeReport {
        window,
        current,
        current_regime: classify(current),
        mean,
        pct_coupled: counts[0] as f64 / total * 100.0,
        pct_neutral: counts[1] as f64 / total * 100.0,
        pct_inverse: counts[2] as f64 / total * 100.0,
        breaks,
        rolling,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Geometric walks where b moves `sign` × a's daily move.
    fn pair(n: usize, sign: f64) -> (Vec<f64>, Vec<f64>) {
        let mut a = vec![100.0];
        let mut b = vec![100.0];
        for i in 1..n {
            // Deterministic pseudo-noise, mean-free over any window.
            let step = if i % 2 == 0 { 0.01 } else { -0.008 };
            a.push(a[i - 1] * (1.0 + step));
            b.push(b[i - 1] * (1.0 + sign * step));
        }
        (a, b)
    }

    #[test]
    fn lockstep_series_read_fully_coupled() {
        let (a, b) = pair(60, 1.0);
        let r = compute(&a, &b, 20).unwrap();
        assert!((r.current - 1.0).abs() < 1e-9);
        assert_eq!(r.current_regime, Regime::Coupled);
        assert!((r.pct_coupled - 100.0).abs() < 1e-12);
        assert!(r.breaks.is_empty());
    }

    #[test]
    fn mirrored_series_read_fully_inverse() {
        let (a, b) = pair(60, -1.0);
        let r = compute(&a, &b, 20).unwrap();
        assert!((r.current + 1.0).abs() < 1e-9);
        assert_eq!(r.current_regime, Regime::Inverse);
        assert!((r.pct_inverse - 100.0).abs() < 1e-12);
    }

    #[test]
    fn regime_flip_is_reported_as_a_break() {
        // 40 coupled bars then 40 mirrored bars: the rolling window
        // must cross from coupled to inverse with ≥1 break en route.
        let (mut a, mut b) = pair(40, 1.0);
        for i in 1..40usize {
            let step = if i % 2 == 0 { 0.01 } else { -0.008 };
            let na = a.last().expect("non-empty") * (1.0 + step);
            let nb = b.last().expect("non-empty") * (1.0 - step);
            a.push(na);
            b.push(nb);
        }
        let r = compute(&a, &b, 20).unwrap();
        assert_eq!(r.current_regime, Regime::Inverse);
        assert!(!r.breaks.is_empty());
        assert_eq!(r.breaks.last().expect("breaks").to, Regime::Inverse);
        assert!(r.pct_coupled > 0.0 && r.pct_inverse > 0.0);
    }

    #[test]
    fn constant_series_reads_zero_correlation() {
        let a = vec![100.0; 40];
        let (_, b) = pair(40, 1.0);
        let r = compute(&a, &b, 20).unwrap();
        assert_eq!(r.current, 0.0);
        assert_eq!(r.current_regime, Regime::Neutral);
    }

    #[test]
    fn hostile_inputs_return_none() {
        let (a, b) = pair(10, 1.0);
        assert!(compute(&a, &b, 2).is_none()); // window too small
        assert!(compute(&a, &b, 10).is_none()); // not enough bars
        assert!(compute(&[], &[], 20).is_none());
    }
}
