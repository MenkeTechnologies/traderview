//! CD early-withdrawal penalty — the cost of breaking a CD early.
//!
//! Cashing out a certificate of deposit before maturity triggers a penalty,
//! usually a set number of months of interest. If you haven't held it long
//! enough to earn that much, the penalty eats into principal.
//!
//!   * interest earned = principal × rate × (months held / 12)
//!   * penalty = principal × rate × (penalty months / 12)
//!   * net interest = earned − penalty (negative ⇒ principal loss)
//!   * net annualized yield = (net interest / principal) × (12 / months held)
//!
//! Pure compute (simple interest at the CD rate; ignores compounding within
//! the period, which is conservative for a short hold).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CdInput {
    pub principal_usd: f64,
    /// The CD's annual rate (APY).
    pub apy_pct: f64,
    /// Months held before the early withdrawal.
    pub months_held: f64,
    /// Penalty expressed as this many months of interest.
    pub penalty_months: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CdResult {
    pub interest_earned_usd: f64,
    pub penalty_usd: f64,
    /// Earned − penalty (negative when the penalty exceeds interest).
    pub net_interest_usd: f64,
    pub net_proceeds_usd: f64,
    /// Annualized effective yield over the holding period.
    pub net_annualized_yield_pct: f64,
    /// True when the penalty exceeds the interest earned (principal is lost).
    pub principal_loss: bool,
}

pub fn analyze(i: &CdInput) -> CdResult {
    let principal = i.principal_usd.max(0.0);
    let rate = i.apy_pct / 100.0;

    let interest_earned = principal * rate * (i.months_held / 12.0);
    let penalty = principal * rate * (i.penalty_months / 12.0);
    let net_interest = interest_earned - penalty;
    let net_proceeds = principal + net_interest;

    let net_yield = if principal > 0.0 && i.months_held > 0.0 {
        net_interest / principal * (12.0 / i.months_held) * 100.0
    } else {
        0.0
    };

    CdResult {
        interest_earned_usd: interest_earned,
        penalty_usd: penalty,
        net_interest_usd: net_interest,
        net_proceeds_usd: net_proceeds,
        net_annualized_yield_pct: net_yield,
        principal_loss: penalty > interest_earned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> CdInput {
        CdInput {
            principal_usd: 10_000.0,
            apy_pct: 5.0,
            months_held: 12.0,
            penalty_months: 6.0,
        }
    }

    #[test]
    fn interest_earned_over_hold() {
        // 10k × 5% × (12/12) = 500.
        let r = analyze(&base());
        assert!((r.interest_earned_usd - 500.0).abs() < 1e-6);
    }

    #[test]
    fn penalty_is_months_of_interest() {
        // 10k × 5% × (6/12) = 250.
        let r = analyze(&base());
        assert!((r.penalty_usd - 250.0).abs() < 1e-6);
    }

    #[test]
    fn net_interest_and_proceeds() {
        let r = analyze(&base());
        assert!((r.net_interest_usd - 250.0).abs() < 1e-6); // 500 − 250
        assert!((r.net_proceeds_usd - 10_250.0).abs() < 1e-6);
        assert!(!r.principal_loss);
    }

    #[test]
    fn principal_loss_when_withdrawn_early() {
        // Held only 3 months (earn 125) but 6-month penalty (250) → lose 125.
        let r = analyze(&CdInput { months_held: 3.0, ..base() });
        assert!((r.interest_earned_usd - 125.0).abs() < 1e-6);
        assert!((r.net_interest_usd - (-125.0)).abs() < 1e-6);
        assert!(r.net_proceeds_usd < 10_000.0); // below principal
        assert!(r.principal_loss);
    }

    #[test]
    fn net_annualized_yield() {
        // Net interest 250 on 10k over 12 months → 2.5% annualized.
        let r = analyze(&base());
        assert!((r.net_annualized_yield_pct - 2.5).abs() < 1e-9);
    }

    #[test]
    fn zero_penalty_net_equals_earned() {
        let r = analyze(&CdInput { penalty_months: 0.0, ..base() });
        assert!((r.net_interest_usd - r.interest_earned_usd).abs() < 1e-9);
        assert!(!r.principal_loss);
    }

    #[test]
    fn longer_hold_higher_net() {
        let short = analyze(&CdInput { months_held: 9.0, ..base() });
        let long = analyze(&CdInput { months_held: 24.0, ..base() });
        assert!(long.net_interest_usd > short.net_interest_usd);
    }

    #[test]
    fn breakeven_hold_equals_penalty_months() {
        // Held exactly the penalty period → net interest zero.
        let r = analyze(&CdInput { months_held: 6.0, ..base() });
        assert!(r.net_interest_usd.abs() < 1e-6);
        assert!(!r.principal_loss); // penalty == earned, not >
    }
}
