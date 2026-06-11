//! Buyback EPS accretion — what a repurchase actually does to EPS
//! once the funding cost is charged.
//!
//!   shares retired = buyback / price
//!   lost income    = buyback × funding_rate × (1 − tax)
//!   new EPS        = (NI − lost income) / (shares − retired)
//!
//! The breakeven multiple falls straight out: a buyback is accretive
//! exactly when the stock's earnings yield beats the after-tax funding
//! rate, i.e. when P/E < 1 / (funding_rate × (1 − tax)).
//!
//! Pure compute. Companion to `deep_value` (shareholder yield reads
//! the same buyback from the cash-flow statement).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct BuybackInput {
    pub net_income: f64,
    pub shares_outstanding: f64,
    pub share_price: f64,
    pub buyback_amount: f64,
    /// Forgone cash yield or borrow cost funding the buyback, decimal.
    pub funding_rate: f64,
    /// Corporate tax rate, decimal.
    pub tax_rate: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BuybackReport {
    pub old_eps: f64,
    pub new_eps: f64,
    pub accretion_pct: f64,
    pub shares_retired: f64,
    /// P/E above which this buyback turns dilutive.
    pub breakeven_pe: Option<f64>,
    pub current_pe: f64,
    pub is_accretive: bool,
}

pub fn compute(inp: &BuybackInput) -> Option<BuybackReport> {
    if ![
        inp.net_income,
        inp.shares_outstanding,
        inp.share_price,
        inp.buyback_amount,
        inp.funding_rate,
        inp.tax_rate,
    ]
    .iter()
    .all(|v| v.is_finite())
        || inp.shares_outstanding <= 0.0
        || inp.share_price <= 0.0
        || inp.buyback_amount < 0.0
        || inp.funding_rate < 0.0
        || !(0.0..1.0).contains(&inp.tax_rate)
        || inp.net_income == 0.0
    {
        return None;
    }
    let retired = inp.buyback_amount / inp.share_price;
    if retired >= inp.shares_outstanding {
        return None; // buying back the whole float
    }
    let old_eps = inp.net_income / inp.shares_outstanding;
    let lost = inp.buyback_amount * inp.funding_rate * (1.0 - inp.tax_rate);
    let new_eps = (inp.net_income - lost) / (inp.shares_outstanding - retired);
    let after_tax_rate = inp.funding_rate * (1.0 - inp.tax_rate);
    Some(BuybackReport {
        old_eps,
        new_eps,
        accretion_pct: (new_eps / old_eps - 1.0) * 100.0,
        shares_retired: retired,
        breakeven_pe: (after_tax_rate > 0.0).then(|| 1.0 / after_tax_rate),
        current_pe: inp.share_price / old_eps,
        is_accretive: new_eps > old_eps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> BuybackInput {
        BuybackInput {
            net_income: 1000.0,
            shares_outstanding: 100.0,
            share_price: 200.0,
            buyback_amount: 2000.0,
            funding_rate: 0.05,
            tax_rate: 0.21,
        }
    }

    #[test]
    fn accretion_hand_walk() {
        // Retire 10 shares; lost income 2000·0.05·0.79 = 79.
        // New EPS = 921/90 = 10.2333… vs 10 ⇒ +2.333%.
        let r = compute(&base()).unwrap();
        assert!((r.shares_retired - 10.0).abs() < 1e-12);
        assert!((r.old_eps - 10.0).abs() < 1e-12);
        assert!((r.new_eps - 921.0 / 90.0).abs() < 1e-12);
        assert!((r.accretion_pct - (921.0 / 900.0 - 1.0) * 100.0).abs() < 1e-9);
        assert!(r.is_accretive);
        // Breakeven P/E = 1/(0.05·0.79) ≈ 25.32; current 20 < that.
        assert!((r.breakeven_pe.unwrap() - 1.0 / 0.0395).abs() < 1e-9);
        assert!((r.current_pe - 20.0).abs() < 1e-12);
    }

    #[test]
    fn expensive_stock_turns_buyback_dilutive() {
        // Same economics at a 30 P/E (price 300 > breakeven 25.3).
        let mut inp = base();
        inp.share_price = 300.0;
        let r = compute(&inp).unwrap();
        assert!(r.current_pe > r.breakeven_pe.unwrap());
        assert!(!r.is_accretive, "{r:?}");
    }

    #[test]
    fn at_breakeven_pe_accretion_is_zero() {
        // Set price so P/E == 1/(after-tax rate) exactly.
        let mut inp = base();
        inp.share_price = 10.0 / 0.0395; // old_eps × breakeven P/E
        let r = compute(&inp).unwrap();
        assert!(r.accretion_pct.abs() < 1e-9, "{}", r.accretion_pct);
    }

    #[test]
    fn free_funding_is_always_accretive() {
        let mut inp = base();
        inp.funding_rate = 0.0;
        let r = compute(&inp).unwrap();
        assert!(r.is_accretive);
        assert!(r.breakeven_pe.is_none());
    }

    #[test]
    fn hostile_inputs_return_none() {
        let mut bad = base();
        bad.buyback_amount = 25_000.0; // retires > float
        assert!(compute(&bad).is_none());
        let mut bad = base();
        bad.tax_rate = 1.0;
        assert!(compute(&bad).is_none());
        let mut bad = base();
        bad.share_price = 0.0;
        assert!(compute(&bad).is_none());
        let mut bad = base();
        bad.net_income = f64::NAN;
        assert!(compute(&bad).is_none());
    }
}
