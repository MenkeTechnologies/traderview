//! Max-Pain calculator.
//!
//! For an option expiration, "max pain" is the strike where the total
//! cash value of in-the-money options is MINIMIZED — i.e. the price
//! at which the MOST option contracts expire worthless. Folklore says
//! market makers nudge underlying toward this strike in the final hours
//! before expiry.
//!
//! Algorithm:
//!   For each candidate strike S:
//!     pain(S) = Σ (S - K_i) × call_OI_i  for K_i ≤ S
//!             + Σ (K_i - S) × put_OI_i   for K_i ≥ S
//!   max_pain_strike = argmin_S pain(S)
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StrikeOi {
    pub strike: f64,
    pub call_oi: u64,
    pub put_oi: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MaxPainReport {
    pub max_pain_strike: f64,
    pub min_total_pain: f64,
    /// Pain at each candidate strike, in the same order as input.
    pub pain_by_strike: Vec<(f64, f64)>,
}

pub fn compute(chain: &[StrikeOi]) -> MaxPainReport {
    let mut report = MaxPainReport::default();
    if chain.is_empty() {
        return report;
    }
    let mut min_pain = f64::INFINITY;
    let mut min_strike = chain[0].strike;
    for candidate in chain {
        let s = candidate.strike;
        let mut total = 0.0;
        for k in chain {
            if k.strike <= s {
                // Call is ITM at S when K < S → call holders take (S - K) × OI.
                total += (s - k.strike) * k.call_oi as f64;
            }
            if k.strike >= s {
                // Put is ITM at S when K > S → put holders take (K - S) × OI.
                total += (k.strike - s) * k.put_oi as f64;
            }
        }
        report.pain_by_strike.push((s, total));
        if total < min_pain {
            min_pain = total;
            min_strike = s;
        }
    }
    report.max_pain_strike = min_strike;
    report.min_total_pain = min_pain;
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(strike: f64, call_oi: u64, put_oi: u64) -> StrikeOi {
        StrikeOi {
            strike,
            call_oi,
            put_oi,
        }
    }

    #[test]
    fn empty_chain_returns_default() {
        let r = compute(&[]);
        assert_eq!(r.max_pain_strike, 0.0);
        assert!(r.pain_by_strike.is_empty());
    }

    #[test]
    fn single_strike_max_pain_is_that_strike() {
        let r = compute(&[s(100.0, 100, 100)]);
        assert_eq!(r.max_pain_strike, 100.0);
        assert_eq!(
            r.min_total_pain, 0.0,
            "no other strikes → pain at the only candidate = 0"
        );
    }

    #[test]
    fn symmetric_chain_max_pain_at_center() {
        // Symmetric OI around 100 → max pain should be at 100.
        let chain = vec![
            s(90.0, 100, 100),
            s(95.0, 100, 100),
            s(100.0, 100, 100),
            s(105.0, 100, 100),
            s(110.0, 100, 100),
        ];
        let r = compute(&chain);
        assert_eq!(r.max_pain_strike, 100.0);
    }

    #[test]
    fn heavy_calls_at_low_strikes_max_pain_pulls_down() {
        // Lots of call OI at $90 → if price closes at $110, those calls
        // would pay out massively. Max pain wants to MINIMIZE that.
        let chain = vec![s(90.0, 10_000, 0), s(110.0, 100, 100)];
        let r = compute(&compute_chain(&chain));
        // Either $90 or $110 — at $90, calls pay 0, put at $110 pays 20×100.
        // At $110, calls at $90 pay 20×10,000=200,000. Min is at $90.
        assert_eq!(r.max_pain_strike, 90.0);
    }

    #[test]
    fn heavy_puts_at_high_strikes_max_pain_pulls_up() {
        let chain = vec![s(90.0, 100, 100), s(110.0, 0, 10_000)];
        let r = compute(&compute_chain(&chain));
        // At $90 puts pay 20×10,000=200,000. At $110 they pay 0. Min at $110.
        assert_eq!(r.max_pain_strike, 110.0);
    }

    #[test]
    fn pain_by_strike_lists_every_candidate() {
        let chain = vec![s(95.0, 100, 100), s(100.0, 100, 100), s(105.0, 100, 100)];
        let r = compute(&chain);
        assert_eq!(r.pain_by_strike.len(), 3);
    }

    #[test]
    fn equal_oi_across_three_strikes_minimum_at_middle() {
        // Pain at 90 (lowest strike): puts above pay (5+10)*100 = 1500.
        // Pain at 95 (middle): calls below pay 5*100=500. Puts above pay 5*100=500.
        //                       Total = 1000. → MINIMUM.
        // Pain at 100: calls below pay (5+10)*100 = 1500.
        let chain = vec![s(90.0, 100, 100), s(95.0, 100, 100), s(100.0, 100, 100)];
        let r = compute(&chain);
        assert_eq!(r.max_pain_strike, 95.0);
        assert_eq!(r.min_total_pain, 1000.0);
    }

    fn compute_chain(c: &[StrikeOi]) -> Vec<StrikeOi> {
        c.to_vec()
    }
}
