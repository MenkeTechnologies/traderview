//! Center of Gravity (CG) — John Ehlers (2002).
//!
//! Smoothed, low-lag oscillator that highlights cyclic turning points:
//!   numerator   = Σ (i+1) · price_{t−i}     for i in 0..period
//!   denominator = Σ price_{t−i}             for i in 0..period
//!   CG          = − numerator / denominator + (period + 1) / 2
//!
//! The (period+1)/2 offset centres CG near zero. Zero-line cross marks
//! the dominant cycle turn — typically faster than RSI/MACD and with
//! less lag than a centered MA.
//!
//! Pure compute.

pub fn compute(closes: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = closes.len();
    let mut out = vec![None; n];
    if period < 2 || n < period {
        return out;
    }
    let center = (period as f64 + 1.0) / 2.0;
    for i in (period - 1)..n {
        let window = &closes[i + 1 - period..=i];
        // Most-recent bar has weight 1, oldest has weight `period`.
        let mut num = 0.0_f64;
        let mut den = 0.0_f64;
        for (k, &p) in window.iter().rev().enumerate() {
            // k=0 → most recent → weight (0+1)=1.
            let w = (k as f64) + 1.0;
            num += w * p;
            den += p;
        }
        if den.is_finite() && den.abs() > 0.0 {
            let cg = -num / den + center;
            if cg.is_finite() {
                out[i] = Some(cg);
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 10).is_empty());
    }

    #[test]
    fn period_too_small_returns_all_none() {
        let v = vec![100.0; 20];
        assert!(compute(&v, 0).iter().all(|x| x.is_none()));
        assert!(compute(&v, 1).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_series_cg_zero() {
        // Σ k·c = c·Σ k = c·n(n+1)/2; Σ c = c·n; ratio = (n+1)/2.
        // CG = -(n+1)/2 + (n+1)/2 = 0.
        let v = vec![100.0; 50];
        let out = compute(&v, 10);
        let last = out[49].expect("populated");
        assert!(last.abs() < 1e-9, "flat CG should be 0, got {last}");
    }

    #[test]
    fn cg_sign_inverts_with_trend_direction() {
        let up: Vec<f64> = (1..=40).map(|i| 100.0 + i as f64).collect();
        let down: Vec<f64> = (1..=40).map(|i| 200.0 - i as f64).collect();
        let cg_up = compute(&up, 10)[39].expect("populated");
        let cg_down = compute(&down, 10)[39].expect("populated");
        assert!(
            cg_up.signum() == -cg_down.signum() || cg_up.abs() < 1e-3 || cg_down.abs() < 1e-3,
            "CG should have opposite signs for up vs down trends, got {cg_up} / {cg_down}"
        );
    }

    #[test]
    fn zero_window_skipped_safely() {
        // All-zero window → denominator 0 → None.
        let v = vec![0.0; 20];
        let out = compute(&v, 5);
        for v in &out {
            assert!(v.is_none());
        }
    }

    #[test]
    fn huge_period_no_panic() {
        let v = vec![1.0; 5];
        assert!(compute(&v, usize::MAX).iter().all(|x| x.is_none()));
    }
}
