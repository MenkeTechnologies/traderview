//! Roll (1984) effective bid-ask spread estimator.
//!
//! Uses serial covariance of consecutive price changes to infer the
//! implicit spread without needing quote data:
//!
//!   cov_t = Cov(Δp_t, Δp_{t−1})
//!   spread = 2 · √(−cov)            when cov < 0
//!          = 0                       when cov ≥ 0 (informed trading)
//!
//! When trades arrive randomly between bid and ask, consecutive price
//! changes are negatively correlated (bid → ask → bid → ask pattern),
//! and that negative covariance carries the spread information. When
//! covariance is positive (e.g. informed trading running through the
//! book), Roll's estimator is undefined → return 0 for that window.
//!
//! Pure compute. Rolling window estimate.

pub fn compute(prices: &[f64], window: usize) -> Vec<Option<f64>> {
    let n = prices.len();
    let mut out = vec![None; n];
    if window < 3 || n < window {
        return out;
    }
    // Pre-compute price changes Δp.
    let mut delta = vec![0.0_f64; n];
    let mut have = vec![false; n];
    for i in 1..n {
        if prices[i].is_finite() && prices[i - 1].is_finite() {
            delta[i] = prices[i] - prices[i - 1];
            have[i] = true;
        }
    }
    for (i, slot) in out.iter_mut().enumerate().skip(window - 1) {
        // Lagged covariance over the window [i−window+1, i]: need pairs
        // (Δp_t, Δp_{t−1}) for t in [i−window+2, i].
        let lo = i + 1 - window;
        let mut sum_now = 0.0;
        let mut sum_prev = 0.0;
        let mut sum_prod = 0.0;
        let mut count = 0;
        for t in (lo + 1)..=i {
            if !have[t] || !have[t - 1] {
                continue;
            }
            sum_now += delta[t];
            sum_prev += delta[t - 1];
            sum_prod += delta[t] * delta[t - 1];
            count += 1;
        }
        if count < 2 {
            continue;
        }
        let mean_now = sum_now / count as f64;
        let mean_prev = sum_prev / count as f64;
        let cov = sum_prod / count as f64 - mean_now * mean_prev;
        if !cov.is_finite() {
            continue;
        }
        let spread = if cov < 0.0 { 2.0 * (-cov).sqrt() } else { 0.0 };
        if spread.is_finite() {
            *slot = Some(spread);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_returns_empty() {
        assert!(compute(&[], 20).is_empty());
    }

    #[test]
    fn window_too_small_returns_all_none() {
        let p = vec![100.0; 30];
        assert!(compute(&p, 0).iter().all(|x| x.is_none()));
        assert!(compute(&p, 1).iter().all(|x| x.is_none()));
        assert!(compute(&p, 2).iter().all(|x| x.is_none()));
    }

    #[test]
    fn window_larger_than_input_returns_all_none() {
        let p = vec![100.0; 10];
        assert!(compute(&p, 20).iter().all(|x| x.is_none()));
    }

    #[test]
    fn random_bid_ask_bounce_recovers_spread_approximately() {
        // Roll's model assumes trades arrive randomly at bid OR ask with
        // 50/50 probability. Pure alternation violates that assumption
        // (consecutive Δp always = ±spread, making cov = −spread² → est = 2·spread).
        // Proper stochastic draws should recover the spread within ~30%.
        let bid = 99.95_f64;
        let ask = 100.05_f64;
        let spread = ask - bid;
        let mut state: u64 = 7919;
        let mut p = Vec::with_capacity(5_000);
        for _ in 0..5_000 {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let u = (state >> 32) as f64 / u32::MAX as f64;
            p.push(if u < 0.5 { bid } else { ask });
        }
        let out = compute(&p, 500);
        let est = out[4_999].expect("populated");
        // Tolerance ~50% (Roll converges slowly + finite-sample noise).
        assert!(
            (est - spread).abs() / spread < 0.5,
            "expected spread ≈ {spread}, got {est}"
        );
    }

    #[test]
    fn trending_market_yields_zero_spread() {
        // Strictly monotonic price → positive serial covariance → spread = 0.
        let p: Vec<f64> = (0..200).map(|i| 100.0 + i as f64).collect();
        let out = compute(&p, 50);
        let est = out[199].expect("populated");
        assert_eq!(est, 0.0);
    }

    #[test]
    fn flat_prices_yield_zero_spread() {
        let p = vec![100.0; 200];
        let out = compute(&p, 50);
        let est = out[199].expect("populated");
        assert_eq!(est, 0.0);
    }

    #[test]
    fn nan_prices_skipped_safely() {
        let mut p: Vec<f64> = (0..200).map(|i| 100.0 + (i % 2) as f64).collect();
        p[100] = f64::NAN;
        let out = compute(&p, 50);
        assert!(out[199].is_some());
    }
}
