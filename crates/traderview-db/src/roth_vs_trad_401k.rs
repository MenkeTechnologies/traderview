//! Roth vs Traditional 401(k) decision calculator.
//!
//! The canonical retirement-tax decision: take the tax deduction today
//! (Traditional) or pay tax now and have tax-free withdrawals later
//! (Roth)?
//!
//! Apples-to-apples comparison assumes you contribute the SAME PRE-TAX
//! amount in each case. With Traditional the full amount goes in; with
//! Roth you'd need (contribution × current_tax_rate) more out of pocket,
//! so we model investing those tax savings in a TAXABLE side account
//! to keep the contributions equivalent.
//!
//!   Traditional path:
//!     contribute the full pre-tax amount → grows tax-deferred → all
//!     withdrawn at retirement marginal rate
//!     after_tax_value = pre_tax × (1+r)^N × (1 − retire_tax_rate)
//!     PLUS the tax savings invested in taxable account → grows at
//!     `r` minus annual drag from dividends + capital gains realised
//!     PLUS final capital gains tax at LTCG rate
//!
//!   Roth path:
//!     contribute the pre-tax amount AFTER current tax → smaller amount
//!     grows tax-free → withdrawn tax-free at retirement
//!     after_tax_value = pre_tax × (1 − current_tax_rate) × (1+r)^N
//!
//! Pure compute (no Monte Carlo — fixed-return projection).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct RothVsTradInput {
    pub annual_pretax_contribution_usd: f64,
    pub current_marginal_tax_rate_pct: f64,
    pub retirement_marginal_tax_rate_pct: f64,
    pub expected_annual_return_pct: f64,
    pub years_to_retirement: u32,
    /// LTCG rate for the side-account capital gains in the Traditional
    /// path. Default 15% (long-term capital gains, married 25-95%).
    #[serde(default = "default_ltcg")]
    pub ltcg_rate_pct: f64,
}

fn default_ltcg() -> f64 { 15.0 }

#[derive(Debug, Clone, Serialize)]
pub struct RothVsTradReport {
    pub traditional_pretax_balance_usd: f64,
    pub traditional_side_account_balance_usd: f64,
    pub traditional_side_account_after_ltcg_usd: f64,
    pub traditional_after_tax_total_usd: f64,
    pub roth_after_tax_total_usd: f64,
    pub net_winner: &'static str,
    pub winner_advantage_usd: f64,
    pub breakeven_retirement_tax_rate_pct: f64,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn future_value_single_contribution(annual: f64, return_pct: f64, years: u32) -> f64 {
    if annual <= 0.0 || years == 0 { return 0.0; }
    let r = return_pct / 100.0;
    if r.abs() < 1e-12 { return annual * years as f64; }
    annual * (((1.0 + r).powi(years as i32) - 1.0) / r)
}

pub fn compute(input: &RothVsTradInput) -> RothVsTradReport {
    let r = input.expected_annual_return_pct / 100.0;
    let n = input.years_to_retirement;
    let curr_tax = input.current_marginal_tax_rate_pct / 100.0;
    let retire_tax = input.retirement_marginal_tax_rate_pct / 100.0;
    let ltcg = input.ltcg_rate_pct / 100.0;

    let _ = r;

    // Traditional path — full pre-tax contribution compounds, withdrawn at retire rate.
    let trad_pretax = future_value_single_contribution(
        input.annual_pretax_contribution_usd, input.expected_annual_return_pct, n
    );
    let trad_after_tax = trad_pretax * (1.0 - retire_tax);
    // Side account: contribution × tax_rate is the tax savings invested annually.
    let annual_side = input.annual_pretax_contribution_usd * curr_tax;
    let side_pretax = future_value_single_contribution(
        annual_side, input.expected_annual_return_pct, n
    );
    // Capital gains at exit = gain × LTCG rate; cost basis = sum of contributions.
    let side_basis = annual_side * n as f64;
    let side_gain = (side_pretax - side_basis).max(0.0);
    let side_after_ltcg = side_pretax - side_gain * ltcg;
    let trad_total = trad_after_tax + side_after_ltcg;

    // Roth path — contribution AFTER current tax compounds, no tax at retire.
    let roth_contribution = input.annual_pretax_contribution_usd * (1.0 - curr_tax);
    let roth_total = future_value_single_contribution(
        roth_contribution, input.expected_annual_return_pct, n
    );

    let (winner, advantage) = if trad_total > roth_total {
        ("traditional", trad_total - roth_total)
    } else if roth_total > trad_total {
        ("roth", roth_total - trad_total)
    } else {
        ("tied", 0.0)
    };

    // Breakeven: at what retire tax rate does trad equal roth?
    // Trad = trad_pretax × (1 − x) + side_after_ltcg = Roth
    // (1 − x) = (Roth − side_after_ltcg) / trad_pretax
    // x = 1 − (Roth − side_after_ltcg) / trad_pretax
    let breakeven_rate = if trad_pretax > 0.0 {
        let x = 1.0 - (roth_total - side_after_ltcg) / trad_pretax;
        (x * 100.0).clamp(0.0, 100.0)
    } else { 0.0 };

    RothVsTradReport {
        traditional_pretax_balance_usd: trad_pretax,
        traditional_side_account_balance_usd: side_pretax,
        traditional_side_account_after_ltcg_usd: side_after_ltcg,
        traditional_after_tax_total_usd: trad_total,
        roth_after_tax_total_usd: roth_total,
        net_winner: winner,
        winner_advantage_usd: advantage,
        breakeven_retirement_tax_rate_pct: breakeven_rate,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> RothVsTradInput {
        RothVsTradInput {
            annual_pretax_contribution_usd: 22_500.0,
            current_marginal_tax_rate_pct: 32.0,
            retirement_marginal_tax_rate_pct: 22.0,
            expected_annual_return_pct: 7.0,
            years_to_retirement: 30,
            ltcg_rate_pct: 15.0,
        }
    }

    #[test]
    fn future_value_single_contribution_zero_return_linear() {
        assert_eq!(future_value_single_contribution(1000.0, 0.0, 10), 10_000.0);
    }

    #[test]
    fn future_value_single_contribution_basic() {
        // $1000/yr at 7% for 10 years = $13,816.45 (sum of geometric series)
        let fv = future_value_single_contribution(1000.0, 7.0, 10);
        assert!((fv - 13_816.45).abs() < 1.0, "got {fv}");
    }

    #[test]
    fn future_value_zero_years_zero() {
        assert_eq!(future_value_single_contribution(1000.0, 7.0, 0), 0.0);
    }

    #[test]
    fn compute_trad_wins_when_retire_lower_than_current() {
        let r = compute(&input());  // 32% now, 22% retire
        // Tax savings invested in side account makes traditional dominate.
        assert_eq!(r.net_winner, "traditional");
    }

    #[test]
    fn compute_roth_wins_when_retire_higher_than_current() {
        let mut i = input();
        i.current_marginal_tax_rate_pct = 12.0;
        i.retirement_marginal_tax_rate_pct = 32.0;
        let r = compute(&i);
        assert_eq!(r.net_winner, "roth");
    }

    #[test]
    fn compute_trad_pretax_basic() {
        // $22.5k/yr × 7% × 30y annuity ≈ $2.124M
        let r = compute(&input());
        assert!(r.traditional_pretax_balance_usd > 2_000_000.0);
        assert!(r.traditional_pretax_balance_usd < 2_300_000.0);
    }

    #[test]
    fn compute_roth_contribution_smaller_than_trad() {
        let r = compute(&input());
        // Roth contribution = $22.5k × (1 − 0.32) = $15.3k.
        // Roth grows over 30y at 7% → smaller than trad pretax.
        assert!(r.roth_after_tax_total_usd < r.traditional_pretax_balance_usd);
    }

    #[test]
    fn compute_advantage_non_negative() {
        let r = compute(&input());
        assert!(r.winner_advantage_usd >= 0.0);
    }

    #[test]
    fn compute_breakeven_within_0_100() {
        let r = compute(&input());
        assert!(r.breakeven_retirement_tax_rate_pct >= 0.0);
        assert!(r.breakeven_retirement_tax_rate_pct <= 100.0);
    }

    #[test]
    fn compute_side_account_after_ltcg_smaller_than_pretax() {
        let r = compute(&input());
        assert!(r.traditional_side_account_after_ltcg_usd
            <= r.traditional_side_account_balance_usd);
    }

    #[test]
    fn compute_equal_tax_rates_neutral_or_trad_wins() {
        // When current = retire tax rate, traditional has a small edge
        // from the side account LTCG arbitrage (15% LTCG < 22% ordinary).
        let mut i = input();
        i.current_marginal_tax_rate_pct = 22.0;
        i.retirement_marginal_tax_rate_pct = 22.0;
        let r = compute(&i);
        assert!(r.net_winner != "roth");
    }
}
