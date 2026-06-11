//! Laguerre RSI — John Ehlers, "Cybernetic Analysis for Stocks and
//! Futures" (2004).
//!
//! Runs price through a 4-stage Laguerre filter (damping `gamma`,
//! 0..1) and forms an RSI from the up/down differences between the
//! filter taps:
//!
//!   L0 = (1−γ)·price + γ·L0'
//!   L1 = −γ·L0 + L0' + γ·L1'        (similarly L2, L3)
//!   CU = Σ max(Lk − Lk+1, 0),  CD = Σ max(Lk+1 − Lk, 0)
//!   LaRSI = CU / (CU + CD)           ∈ [0, 1]
//!
//! Far smoother than a classic RSI at comparable lag; readings > 0.8
//! flag overbought, < 0.2 oversold. Default gamma 0.5.
//!
//! Pure compute. Output aligned with input; non-finite closes repeat
//! the previous reading.

pub fn compute(closes: &[f64], gamma: f64) -> Vec<f64> {
    let n = closes.len();
    let mut out = vec![0.0; n];
    if n == 0 || !(0.0..1.0).contains(&gamma) {
        return out;
    }
    let (mut l0, mut l1, mut l2, mut l3) = (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64);
    let mut seeded = false;
    for (i, &price) in closes.iter().enumerate() {
        if !price.is_finite() {
            out[i] = if i > 0 { out[i - 1] } else { 0.0 };
            continue;
        }
        if !seeded {
            // Seed all taps at the first finite price so the filter
            // starts converged instead of pulling from zero.
            (l0, l1, l2, l3) = (price, price, price, price);
            seeded = true;
        }
        let (p0, p1, p2, p3) = (l0, l1, l2, l3);
        l0 = (1.0 - gamma) * price + gamma * p0;
        l1 = -gamma * l0 + p0 + gamma * p1;
        l2 = -gamma * l1 + p1 + gamma * p2;
        l3 = -gamma * l2 + p2 + gamma * p3;
        let mut cu = 0.0;
        let mut cd = 0.0;
        for (a, b) in [(l0, l1), (l1, l2), (l2, l3)] {
            if a >= b {
                cu += a - b;
            } else {
                cd += b - a;
            }
        }
        out[i] = if cu + cd > 0.0 { cu / (cu + cd) } else { 0.0 };
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn laguerre_rsi_saturates_high_in_a_steady_uptrend() {
        let closes: Vec<f64> = (0..60).map(|i| 100.0 + 2.0 * i as f64).collect();
        let v = compute(&closes, 0.5);
        // After convergence every tap difference is positive ⇒ CU only.
        assert!(v[59] > 0.95, "{}", v[59]);
    }

    #[test]
    fn laguerre_rsi_saturates_low_in_a_steady_downtrend() {
        let closes: Vec<f64> = (0..60).map(|i| 220.0 - 2.0 * i as f64).collect();
        let v = compute(&closes, 0.5);
        assert!(v[59] < 0.05, "{}", v[59]);
    }

    #[test]
    fn laguerre_rsi_is_bounded_zero_one() {
        // Choppy tape: alternate ±5%.
        let mut closes = vec![100.0];
        for i in 1..100 {
            let prev = closes[i - 1];
            closes.push(if i % 2 == 0 { prev * 1.05 } else { prev * 0.95 });
        }
        for x in compute(&closes, 0.7) {
            assert!((0.0..=1.0).contains(&x), "{x}");
        }
    }

    #[test]
    fn laguerre_rsi_flat_tape_reads_zero() {
        // Constant price: all taps equal ⇒ CU = CD = 0 ⇒ defined as 0.
        let v = compute(&[100.0; 20], 0.5);
        assert!(v.iter().all(|x| *x == 0.0));
    }

    #[test]
    fn laguerre_rsi_survives_hostile_inputs() {
        assert!(compute(&[], 0.5).is_empty());
        // Out-of-range gamma → all zeros, no panic.
        assert_eq!(compute(&[100.0, 101.0], 1.5), vec![0.0, 0.0]);
        assert_eq!(compute(&[100.0, 101.0], -0.1), vec![0.0, 0.0]);
        // NaN close repeats the previous reading.
        let closes = vec![100.0, 102.0, f64::NAN, 104.0];
        let v = compute(&closes, 0.5);
        assert_eq!(v[2], v[1]);
        assert!(v[3] > 0.0);
    }
}
