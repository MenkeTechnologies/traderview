//! FX carry trade + covered interest parity.
//!
//! Long the high-yield (base) currency funded in the low-yield (quote)
//! currency: the carry is the rate differential, collected as long as
//! spot holds. Covered interest parity prices the forward:
//!
//!   F = S × (1 + r_quote·t) / (1 + r_base·t)
//!
//! The high-yielder trades at a FORWARD DISCOUNT — hedging the FX risk
//! with that forward locks your return at the QUOTE rate exactly: the
//! hedged carry trade is arbitraged away by construction. The unhedged
//! carry is real but is compensation for depreciation risk; the
//! breakeven figure is the spot drop that erases it.
//!
//! Simple (non-compounded) interest over t = days/365 — the convention
//! FX forward points actually quote in.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    /// Spot, quote-per-base (e.g. USDJPY 150 = 150 JPY per USD).
    pub spot: f64,
    /// Annual rate of the BASE currency (the one you're long), percent.
    pub base_rate_pct: f64,
    /// Annual rate of the QUOTE (funding) currency, percent.
    pub quote_rate_pct: f64,
    pub days: f64,
    /// Position size in BASE-currency units of notional.
    pub notional: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    /// CIP fair forward rate.
    pub forward_rate: f64,
    /// F − S; negative = the base trades at a forward discount.
    pub forward_points: f64,
    /// Annualized rate differential, base − quote.
    pub carry_apr_pct: f64,
    /// Unhedged carry collected over the period (base-ccy units).
    pub carry_income: f64,
    /// Spot depreciation (%) that exactly erases the period's carry.
    pub breakeven_depreciation_pct: f64,
    /// Hedged return = the QUOTE rate, exactly — CIP leaves no free
    /// lunch once the forward is sold.
    pub hedged_return_apr_pct: f64,
}

pub fn compute(i: &Input) -> Option<Report> {
    let t = i.days / 365.0;
    if i.spot <= 0.0
        || i.notional <= 0.0
        || i.days <= 0.0
        || !i.base_rate_pct.is_finite()
        || !i.quote_rate_pct.is_finite()
    {
        return None;
    }
    let rb = i.base_rate_pct / 100.0;
    let rq = i.quote_rate_pct / 100.0;
    // Simple-interest CIP; reject a degenerate denominator.
    let denom = 1.0 + rb * t;
    if denom <= 0.0 {
        return None;
    }
    let forward_rate = i.spot * (1.0 + rq * t) / denom;
    let diff = rb - rq;
    Some(Report {
        forward_rate,
        forward_points: forward_rate - i.spot,
        carry_apr_pct: diff * 100.0,
        carry_income: i.notional * diff * t,
        breakeven_depreciation_pct: diff * t * 100.0,
        hedged_return_apr_pct: i.quote_rate_pct,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn usdjpy() -> Input {
        Input {
            spot: 150.0,
            base_rate_pct: 5.0,  // USD
            quote_rate_pct: 0.5, // JPY
            days: 365.0,
            notional: 100_000.0,
        }
    }

    #[test]
    fn one_year_usdjpy_pins_the_whole_ledger() {
        let r = compute(&usdjpy()).unwrap();
        // CIP: 150 × 1.005 / 1.05 = 143.5714…— the high-yielder at a
        // forward DISCOUNT.
        assert!((r.forward_rate - 150.0 * 1.005 / 1.05).abs() < 1e-9);
        assert!(r.forward_points < 0.0);
        assert!((r.carry_apr_pct - 4.5).abs() < 1e-9);
        // 100k × 4.5% × 1y = 4,500 of unhedged carry.
        assert!((r.carry_income - 4_500.0).abs() < 1e-9);
        // A 4.5% spot drop erases exactly that.
        assert!((r.breakeven_depreciation_pct - 4.5).abs() < 1e-9);
        // Hedged: you earn the JPY rate, nothing more.
        assert!((r.hedged_return_apr_pct - 0.5).abs() < 1e-9);
    }

    #[test]
    fn half_period_halves_income_and_pulls_forward_toward_spot() {
        let full = compute(&usdjpy()).unwrap();
        let half = compute(&Input { days: 182.5, ..usdjpy() }).unwrap();
        assert!((half.carry_income - 2_250.0).abs() < 1e-9);
        assert!((half.breakeven_depreciation_pct - 2.25).abs() < 1e-9);
        // Shorter tenor → forward closer to spot.
        assert!(half.forward_points.abs() < full.forward_points.abs());
        // The APR is tenor-independent.
        assert!((half.carry_apr_pct - full.carry_apr_pct).abs() < 1e-12);
    }

    #[test]
    fn negative_differential_means_paying_the_carry() {
        // Long the LOW yielder: carry negative, forward at a premium.
        let r = compute(&Input {
            base_rate_pct: 0.5,
            quote_rate_pct: 5.0,
            ..usdjpy()
        })
        .unwrap();
        assert!((r.carry_apr_pct + 4.5).abs() < 1e-9);
        assert!(r.carry_income < 0.0);
        assert!(r.forward_points > 0.0);
    }

    #[test]
    fn degenerate_inputs_are_none() {
        assert!(compute(&Input { spot: 0.0, ..usdjpy() }).is_none());
        assert!(compute(&Input { days: 0.0, ..usdjpy() }).is_none());
        assert!(compute(&Input { notional: -1.0, ..usdjpy() }).is_none());
        assert!(compute(&Input { base_rate_pct: f64::NAN, ..usdjpy() }).is_none());
    }
}
