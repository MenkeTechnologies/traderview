//! Butterfly spread P&L (long body, short wings of equal width).
//!
//! Construction (call butterfly):
//!   long  1 call @ lower_wing_strike
//!   short 2 calls @ body_strike (the "peak")
//!   long  1 call @ upper_wing_strike
//!
//! Constraint:
//!   upper_wing_strike − body_strike == body_strike − lower_wing_strike
//!     (i.e. equal-width wings; a "broken-wing" variant relaxes this
//!     and is excluded here — separate concept).
//!
//! At expiration:
//!   pnl(S) = max(0, S − Kₗ) − 2·max(0, S − K_body) + max(0, S − Kᵤ)
//!          − net_debit
//!
//! Max profit at S = body_strike: wing_width − net_debit.
//! Max loss = net_debit (both wings expire worthless).
//! Breakevens: body_strike ± (wing_width − net_debit).
//!
//! Pure compute. Symmetric — same shape for put butterflies.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionKind {
    Call,
    Put,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Butterfly {
    pub kind: OptionKind,
    pub lower_wing_strike: f64,
    pub body_strike: f64,
    pub upper_wing_strike: f64,
    pub net_debit_per_contract: f64, // positive number (debit paid)
    pub contracts: i64,
    pub multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ButterflyReport {
    pub max_profit: f64,
    pub max_loss: f64,
    pub lower_breakeven: f64,
    pub upper_breakeven: f64,
    pub profit_zone_width: f64,
    pub wing_width: f64,
    pub debit_to_max_profit_ratio: f64,
}

pub fn analyze(b: &Butterfly) -> Option<ButterflyReport> {
    if !b.lower_wing_strike.is_finite()
        || !b.body_strike.is_finite()
        || !b.upper_wing_strike.is_finite()
        || !b.net_debit_per_contract.is_finite()
        || !b.multiplier.is_finite()
    {
        return None;
    }
    if !(b.lower_wing_strike < b.body_strike && b.body_strike < b.upper_wing_strike) {
        return None;
    }
    let lower_w = b.body_strike - b.lower_wing_strike;
    let upper_w = b.upper_wing_strike - b.body_strike;
    if (lower_w - upper_w).abs() > 1e-9 {
        return None; // not symmetric → broken-wing, not this module
    }
    if b.net_debit_per_contract < 0.0 || b.multiplier <= 0.0 || b.contracts == 0 {
        return None;
    }
    let wing_width = lower_w;
    let max_profit_per = wing_width - b.net_debit_per_contract;
    if max_profit_per < 0.0 {
        return None; // debit exceeds wing width → mispriced
    }
    let scale = b.contracts.unsigned_abs() as f64 * b.multiplier;
    let sign = b.contracts.signum() as f64;
    Some(ButterflyReport {
        max_profit: max_profit_per * scale * sign,
        max_loss: -b.net_debit_per_contract * scale * sign,
        lower_breakeven: b.lower_wing_strike + b.net_debit_per_contract,
        upper_breakeven: b.upper_wing_strike - b.net_debit_per_contract,
        profit_zone_width: 2.0 * (wing_width - b.net_debit_per_contract),
        wing_width,
        debit_to_max_profit_ratio: if max_profit_per > 0.0 {
            b.net_debit_per_contract / max_profit_per
        } else {
            f64::INFINITY
        },
    })
}

pub fn pnl_at_expiration(b: &Butterfly, spot: f64) -> Option<f64> {
    if !spot.is_finite()
        || !b.lower_wing_strike.is_finite()
        || !b.body_strike.is_finite()
        || !b.upper_wing_strike.is_finite()
        || !b.net_debit_per_contract.is_finite()
    {
        return None;
    }
    let leg = |k: f64| match b.kind {
        OptionKind::Call => (spot - k).max(0.0),
        OptionKind::Put => (k - spot).max(0.0),
    };
    let per = leg(b.lower_wing_strike) - 2.0 * leg(b.body_strike) + leg(b.upper_wing_strike)
        - b.net_debit_per_contract;
    let scale = b.contracts.unsigned_abs() as f64 * b.multiplier;
    let sign = b.contracts.signum() as f64;
    Some(per * scale * sign)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bf() -> Butterfly {
        Butterfly {
            kind: OptionKind::Call,
            lower_wing_strike: 95.0,
            body_strike: 100.0,
            upper_wing_strike: 105.0,
            net_debit_per_contract: 1.5,
            contracts: 1,
            multiplier: 100.0,
        }
    }

    #[test]
    fn asymmetric_wings_rejected() {
        let mut bad = bf();
        bad.upper_wing_strike = 110.0; // wings 5/10 — asymmetric
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn wrong_strike_order_rejected() {
        let mut bad = bf();
        std::mem::swap(&mut bad.body_strike, &mut bad.upper_wing_strike);
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn negative_debit_rejected() {
        let mut bad = bf();
        bad.net_debit_per_contract = -1.0;
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn debit_exceeding_wing_width_rejected() {
        let mut bad = bf();
        bad.net_debit_per_contract = 10.0; // > wing_width = 5
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn zero_contracts_rejected() {
        let mut bad = bf();
        bad.contracts = 0;
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn nan_inputs_rejected() {
        let mut bad = bf();
        bad.body_strike = f64::NAN;
        assert!(analyze(&bad).is_none());
    }

    #[test]
    fn standard_butterfly_report_matches_textbook() {
        // Wing width = 5, debit = 1.5 → max profit = 3.5 → $350/contract.
        // Max loss = $150. BE_low = 96.5. BE_high = 103.5. Zone width = 7.
        let r = analyze(&bf()).unwrap();
        assert!((r.max_profit - 350.0).abs() < 1e-9);
        assert!((r.max_loss + 150.0).abs() < 1e-9);
        assert!((r.lower_breakeven - 96.5).abs() < 1e-9);
        assert!((r.upper_breakeven - 103.5).abs() < 1e-9);
        assert!((r.profit_zone_width - 7.0).abs() < 1e-9);
    }

    #[test]
    fn pnl_at_body_strike_is_max_profit() {
        let p = pnl_at_expiration(&bf(), 100.0).unwrap();
        assert!((p - 350.0).abs() < 1e-9);
    }

    #[test]
    fn pnl_at_wings_is_max_loss() {
        let p_lo = pnl_at_expiration(&bf(), 95.0).unwrap();
        let p_hi = pnl_at_expiration(&bf(), 105.0).unwrap();
        assert!((p_lo + 150.0).abs() < 1e-9);
        assert!((p_hi + 150.0).abs() < 1e-9);
    }

    #[test]
    fn pnl_at_lower_breakeven_zero() {
        let p = pnl_at_expiration(&bf(), 96.5).unwrap();
        assert!(p.abs() < 1e-9);
    }

    #[test]
    fn put_butterfly_symmetric_to_call() {
        // Put butterfly at same strikes has identical P&L curve at expiry.
        let mut put = bf();
        put.kind = OptionKind::Put;
        let p_call = pnl_at_expiration(&bf(), 100.0).unwrap();
        let p_put = pnl_at_expiration(&put, 100.0).unwrap();
        assert!((p_call - p_put).abs() < 1e-9);
    }

    #[test]
    fn short_butterfly_inverts_pnl() {
        let mut short = bf();
        short.contracts = -1;
        let p_long = pnl_at_expiration(&bf(), 100.0).unwrap();
        let p_short = pnl_at_expiration(&short, 100.0).unwrap();
        assert!((p_long + p_short).abs() < 1e-9);
    }
}
