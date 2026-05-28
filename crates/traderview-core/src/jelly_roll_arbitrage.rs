//! Jelly Roll Arbitrage Detector — exploits forward-rate inconsistencies
//! between options of two different expiries on the same underlying.
//!
//! A jelly roll combines a short calendar spread on the call side with
//! a long calendar spread on the put side at the same strike:
//!
//!   + long call(T_long, K) − short call(T_short, K)
//!   + long put(T_short, K) − short put(T_long, K)
//!
//! By put-call parity, this synthesizes a forward-versus-forward
//! position. The fair jelly-roll price is:
//!
//!   fair = K · (D(T_short) − D(T_long))
//!
//! where D(T) = exp(−r · T) is the discount factor. Market-traded
//! jelly-roll mispricings indicate inconsistent forward curves or
//! exploitable arbitrage.
//!
//! Pure compute. Companion to `box_spread`, `iron_condor`,
//! `calendar_spread`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JellyRollReport {
    pub strike: f64,
    pub time_short_years: f64,
    pub time_long_years: f64,
    pub market_premium_debit: f64,
    pub fair_premium_no_arb: f64,
    pub mispricing: f64,
    pub mispricing_basis_points: f64,
    pub is_arbitrage_opportunity: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn compute(
    strike: f64,
    risk_free_rate: f64,
    time_short_years: f64,
    time_long_years: f64,
    call_short_price: f64,
    call_long_price: f64,
    put_short_price: f64,
    put_long_price: f64,
    arbitrage_threshold_bps: f64,
) -> Option<JellyRollReport> {
    if !strike.is_finite() || strike <= 0.0
        || !risk_free_rate.is_finite()
        || !time_short_years.is_finite() || time_short_years <= 0.0
        || !time_long_years.is_finite() || time_long_years <= time_short_years
        || ![call_short_price, call_long_price, put_short_price, put_long_price]
            .iter().all(|p| p.is_finite() && *p >= 0.0)
        || !arbitrage_threshold_bps.is_finite() || arbitrage_threshold_bps < 0.0
    {
        return None;
    }
    let d_short = (-risk_free_rate * time_short_years).exp();
    let d_long = (-risk_free_rate * time_long_years).exp();
    let fair = strike * (d_short - d_long);
    // Market premium (debit) of the long jelly-roll position.
    let market = (call_long_price - call_short_price)
        + (put_short_price - put_long_price);
    let mispricing = market - fair;
    let mispricing_bps = (mispricing / strike) * 10_000.0;
    let is_arb = mispricing_bps.abs() > arbitrage_threshold_bps;
    Some(JellyRollReport {
        strike,
        time_short_years,
        time_long_years,
        market_premium_debit: market,
        fair_premium_no_arb: fair,
        mispricing,
        mispricing_basis_points: mispricing_bps,
        is_arbitrage_opportunity: is_arb,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_strikes_or_times_return_none() {
        assert!(compute(0.0, 0.05, 0.25, 0.5, 5.0, 5.0, 4.0, 4.0, 50.0).is_none());
        assert!(compute(100.0, 0.05, 0.0, 0.5, 5.0, 5.0, 4.0, 4.0, 50.0).is_none());
        assert!(compute(100.0, 0.05, 0.5, 0.25, 5.0, 5.0, 4.0, 4.0, 50.0).is_none());
        // T_long must exceed T_short.
    }

    #[test]
    fn invalid_prices_return_none() {
        assert!(compute(100.0, 0.05, 0.25, 0.5, -1.0, 5.0, 4.0, 4.0, 50.0).is_none());
        assert!(compute(100.0, 0.05, 0.25, 0.5, f64::NAN, 5.0, 4.0, 4.0, 50.0).is_none());
    }

    #[test]
    fn fair_jelly_roll_yields_zero_mispricing() {
        let strike = 100.0_f64;
        let r = 0.05_f64;
        let t_s = 0.25_f64;
        let t_l = 0.5_f64;
        let fair = strike * ((-r * t_s).exp() - (-r * t_l).exp());
        // Construct market prices that exactly match fair: e.g. all puts
        // equal, all calls equal except call_long = call_short + fair.
        let call_short = 5.0;
        let call_long = call_short + fair;
        let put_short = 4.0;
        let put_long = 4.0;
        let result = compute(strike, r, t_s, t_l,
            call_short, call_long, put_short, put_long, 50.0).unwrap();
        assert!(result.mispricing.abs() < 1e-9,
            "fair jelly-roll should yield 0 mispricing, got {}", result.mispricing);
        assert!(!result.is_arbitrage_opportunity);
    }

    #[test]
    fn overpriced_jelly_roll_flagged_arbitrage() {
        let strike = 100.0_f64;
        let r = 0.05_f64;
        let t_s = 0.25_f64;
        let t_l = 0.5_f64;
        let fair = strike * ((-r * t_s).exp() - (-r * t_l).exp());
        // Market premium 5% above fair → arbitrage.
        let market_excess = fair * 0.20;    // 20% above fair
        let call_short = 5.0;
        let call_long = call_short + fair + market_excess;
        let put_short = 4.0;
        let put_long = 4.0;
        let result = compute(strike, r, t_s, t_l,
            call_short, call_long, put_short, put_long, 1.0).unwrap();
        assert!(result.mispricing > 0.0);
        assert!(result.is_arbitrage_opportunity);
    }

    #[test]
    fn arbitrage_threshold_honored() {
        let strike = 100.0_f64;
        let r = 0.05_f64;
        let t_s = 0.25_f64;
        let t_l = 0.5_f64;
        let fair = strike * ((-r * t_s).exp() - (-r * t_l).exp());
        let call_short = 5.0;
        let call_long = call_short + fair + 0.01;    // tiny mispricing
        let put_short = 4.0;
        let put_long = 4.0;
        let result = compute(strike, r, t_s, t_l,
            call_short, call_long, put_short, put_long, 100.0).unwrap();
        assert!(!result.is_arbitrage_opportunity);
    }

    #[test]
    fn mispricing_basis_points_scaled_by_strike() {
        let strike = 100.0_f64;
        let r = 0.05_f64;
        let t_s = 0.25_f64;
        let t_l = 0.5_f64;
        let fair = strike * ((-r * t_s).exp() - (-r * t_l).exp());
        let call_short = 5.0;
        let call_long = call_short + fair + 0.10;
        let put_short = 4.0;
        let put_long = 4.0;
        let result = compute(strike, r, t_s, t_l,
            call_short, call_long, put_short, put_long, 1.0).unwrap();
        // 0.10 mispricing / 100 strike = 10 bps.
        assert!((result.mispricing_basis_points - 10.0).abs() < 1e-9);
    }
}
