//! Blended debt rate — the balance-weighted average APR across debts.
//!
//! With several balances at different rates, the single rate that matters is
//! the **balance-weighted** average — a big balance at a middling rate costs
//! more than a tiny balance at a brutal one:
//!
//!   * blended APR = Σ(balanceᵢ × APRᵢ) / Σ(balanceᵢ)
//!   * monthly interest = Σ(balanceᵢ × APRᵢ / 12)
//!
//! Compare the blended rate to a consolidation-loan rate: if the loan rate
//! is below the blended APR, consolidating cuts the monthly interest.
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Debt {
    #[serde(default)]
    pub name: String,
    pub balance_usd: f64,
    pub apr_pct: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BlendedDebtInput {
    pub debts: Vec<Debt>,
    /// Optional consolidation-loan APR to compare against (0 ⇒ skip).
    #[serde(default)]
    pub consolidation_apr_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BlendedDebtResult {
    pub total_balance_usd: f64,
    pub blended_apr_pct: f64,
    pub total_monthly_interest_usd: f64,
    /// Monthly interest if all balances moved to the consolidation rate.
    pub consolidation_monthly_interest_usd: f64,
    /// Current monthly interest − consolidation monthly interest.
    pub monthly_savings_usd: f64,
    /// True when the consolidation rate beats the blended APR.
    pub consolidation_worth_it: bool,
}

pub fn analyze(i: &BlendedDebtInput) -> BlendedDebtResult {
    let total_balance: f64 = i.debts.iter().map(|d| d.balance_usd.max(0.0)).sum();
    let weighted: f64 = i.debts.iter().map(|d| d.balance_usd.max(0.0) * d.apr_pct).sum();

    let blended_apr = if total_balance > 0.0 { weighted / total_balance } else { 0.0 };
    let monthly_interest = total_balance * blended_apr / 100.0 / 12.0;

    let consol_monthly = total_balance * i.consolidation_apr_pct / 100.0 / 12.0;
    // Only treat consolidation as beneficial when a positive rate was given.
    let consol_given = i.consolidation_apr_pct > 0.0;
    let savings = monthly_interest - consol_monthly;
    let worth_it = consol_given && i.consolidation_apr_pct < blended_apr;

    BlendedDebtResult {
        total_balance_usd: total_balance,
        blended_apr_pct: blended_apr,
        total_monthly_interest_usd: monthly_interest,
        consolidation_monthly_interest_usd: consol_monthly,
        monthly_savings_usd: savings,
        consolidation_worth_it: worth_it,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn debt(b: f64, apr: f64) -> Debt {
        Debt { name: String::new(), balance_usd: b, apr_pct: apr }
    }

    #[test]
    fn blended_of_two_debts() {
        // 10k@20% + 30k@8% → (10k×20 + 30k×8)/40k = (200k+240k)/40k = 11%.
        let r = analyze(&BlendedDebtInput {
            debts: vec![debt(10_000.0, 20.0), debt(30_000.0, 8.0)],
            consolidation_apr_pct: 0.0,
        });
        assert!((r.blended_apr_pct - 11.0).abs() < 1e-9);
    }

    #[test]
    fn total_balance_sums() {
        let r = analyze(&BlendedDebtInput {
            debts: vec![debt(10_000.0, 20.0), debt(30_000.0, 8.0)],
            consolidation_apr_pct: 0.0,
        });
        assert!((r.total_balance_usd - 40_000.0).abs() < 1e-9);
    }

    #[test]
    fn monthly_interest_at_blended() {
        // 40k × 11% / 12 = 366.67.
        let r = analyze(&BlendedDebtInput {
            debts: vec![debt(10_000.0, 20.0), debt(30_000.0, 8.0)],
            consolidation_apr_pct: 0.0,
        });
        assert!((r.total_monthly_interest_usd - 40_000.0 * 0.11 / 12.0).abs() < 1e-6);
    }

    #[test]
    fn single_debt_blended_equals_its_apr() {
        let r = analyze(&BlendedDebtInput { debts: vec![debt(5_000.0, 17.99)], consolidation_apr_pct: 0.0 });
        assert!((r.blended_apr_pct - 17.99).abs() < 1e-9);
    }

    #[test]
    fn bigger_balance_dominates_the_blend() {
        // Tiny brutal balance barely moves the blend.
        let r = analyze(&BlendedDebtInput {
            debts: vec![debt(100.0, 30.0), debt(100_000.0, 5.0)],
            consolidation_apr_pct: 0.0,
        });
        assert!(r.blended_apr_pct < 5.1);
    }

    #[test]
    fn consolidation_saves_when_rate_below_blended() {
        // Blended 11%; consolidate at 9% → worth it, positive savings.
        let r = analyze(&BlendedDebtInput {
            debts: vec![debt(10_000.0, 20.0), debt(30_000.0, 8.0)],
            consolidation_apr_pct: 9.0,
        });
        assert!(r.consolidation_worth_it);
        assert!(r.monthly_savings_usd > 0.0);
    }

    #[test]
    fn consolidation_not_worth_it_above_blended() {
        let r = analyze(&BlendedDebtInput {
            debts: vec![debt(10_000.0, 20.0), debt(30_000.0, 8.0)],
            consolidation_apr_pct: 15.0,
        });
        assert!(!r.consolidation_worth_it);
        assert!(r.monthly_savings_usd < 0.0);
    }

    #[test]
    fn empty_or_zero_balance_guards() {
        let r = analyze(&BlendedDebtInput { debts: vec![], consolidation_apr_pct: 9.0 });
        assert!(r.total_balance_usd.abs() < 1e-9);
        assert!(r.blended_apr_pct.abs() < 1e-9);
        assert!(!r.consolidation_worth_it); // nothing to consolidate
    }
}
