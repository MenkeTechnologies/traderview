//! SaaS magic number — sales efficiency: how much new annual recurring revenue
//! each dollar of the prior period's sales & marketing spend generated. It is the
//! quarter-over-quarter increase in revenue, annualized (×4), divided by the prior
//! quarter's S&M spend. Above ~0.75 is efficient enough to invest harder; below
//! ~0.5 signals the go-to-market is not paying back. It also reports the implied
//! months to recover S&M. Distinct from the LTV/CAC and break-even ROAS modules.
//! Pure compute, not financial advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct MagicNumberInput {
    pub current_quarter_revenue_usd: f64,
    pub prior_quarter_revenue_usd: f64,
    /// Sales & marketing spend in the PRIOR quarter (it drives this quarter's growth).
    pub prior_quarter_sm_spend_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct MagicNumberReport {
    /// (current − prior) × 4 — annualized net-new revenue.
    pub annualized_net_new_arr_usd: f64,
    /// Net-new ARR ÷ prior S&M spend.
    pub magic_number: f64,
    /// Efficiency band: "poor", "acceptable", or "efficient".
    pub efficiency: String,
    /// Implied months to recover the S&M spend (12 ÷ magic number).
    pub sm_payback_months: f64,
    pub valid: bool,
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

pub fn generate(i: &MagicNumberInput) -> MagicNumberReport {
    if i.prior_quarter_sm_spend_usd <= 0.0 {
        return MagicNumberReport::default();
    }
    let net_new = (i.current_quarter_revenue_usd - i.prior_quarter_revenue_usd) * 4.0;
    let magic = net_new / i.prior_quarter_sm_spend_usd;
    let efficiency = if magic >= 0.75 {
        "efficient"
    } else if magic >= 0.5 {
        "acceptable"
    } else {
        "poor"
    };
    let payback = if magic > 0.0 { round2(12.0 / magic) } else { 0.0 };
    MagicNumberReport {
        annualized_net_new_arr_usd: round2(net_new),
        magic_number: round4(magic),
        efficiency: efficiency.to_string(),
        sm_payback_months: payback,
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> MagicNumberInput {
        MagicNumberInput {
            current_quarter_revenue_usd: 1_100_000.0,
            prior_quarter_revenue_usd: 1_000_000.0,
            prior_quarter_sm_spend_usd: 400_000.0,
        }
    }

    #[test]
    fn magic_one_efficient() {
        let d = generate(&base());
        assert!(close(d.annualized_net_new_arr_usd, 400_000.0));
        assert!(close(d.magic_number, 1.0));
        assert_eq!(d.efficiency, "efficient");
        assert!(close(d.sm_payback_months, 12.0));
    }

    #[test]
    fn high_growth_high_magic() {
        let d = generate(&MagicNumberInput { current_quarter_revenue_usd: 1_300_000.0, ..base() });
        assert!(close(d.magic_number, 3.0));
        assert_eq!(d.efficiency, "efficient");
    }

    #[test]
    fn poor_efficiency_band() {
        // Small growth, large spend → magic < 0.5.
        let d = generate(&MagicNumberInput { current_quarter_revenue_usd: 1_030_000.0, ..base() });
        // (30k×4)/400k = 0.3.
        assert!(close(d.magic_number, 0.3));
        assert_eq!(d.efficiency, "poor");
    }

    #[test]
    fn acceptable_band() {
        let d = generate(&MagicNumberInput { current_quarter_revenue_usd: 1_060_000.0, ..base() });
        // (60k×4)/400k = 0.6.
        assert!(close(d.magic_number, 0.6));
        assert_eq!(d.efficiency, "acceptable");
    }

    #[test]
    fn zero_spend_invalid() {
        let d = generate(&MagicNumberInput { prior_quarter_sm_spend_usd: 0.0, ..base() });
        assert!(!d.valid);
    }
}
