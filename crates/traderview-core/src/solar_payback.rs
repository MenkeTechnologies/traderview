//! Solar payback — when a rooftop PV system pays for itself and the lifetime
//! return.
//!
//! Net cost is the install price less the 30% federal credit and any other
//! incentives. Annual savings (avoided electricity) rise with utility inflation
//! but fall as panels degrade:
//!
//! ```text
//! net cost     = system cost × (1 − federal credit) − incentives
//! year y saving = savings₁ × (1 + inflation)^(y−1) × (1 − degradation)^(y−1)
//! payback       = year cumulative savings first cover net cost
//! ```

use serde::{Deserialize, Serialize};

fn d_fed() -> f64 {
    30.0
}
fn d_deg() -> f64 {
    0.5
}
fn d_inf() -> f64 {
    3.0
}
fn d_horizon() -> f64 {
    25.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct SolarInput {
    pub system_cost_usd: f64,
    /// Federal tax credit, percent of system cost.
    #[serde(default = "d_fed")]
    pub federal_credit_pct: f64,
    /// Other rebates/incentives, dollars.
    #[serde(default)]
    pub other_incentives_usd: f64,
    /// First-year electricity savings.
    pub annual_savings_usd: f64,
    /// Annual panel output degradation, percent.
    #[serde(default = "d_deg")]
    pub annual_degradation_pct: f64,
    /// Annual utility-rate inflation, percent.
    #[serde(default = "d_inf")]
    pub electricity_inflation_pct: f64,
    /// Analysis horizon in years (system life).
    #[serde(default = "d_horizon")]
    pub horizon_years: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SolarResult {
    /// System cost after the federal credit and incentives.
    pub net_cost_usd: f64,
    pub federal_credit_usd: f64,
    /// Total savings over the horizon.
    pub lifetime_savings_usd: f64,
    /// Years until cumulative savings cover the net cost; `None` if never.
    pub payback_years: Option<f64>,
    /// Lifetime savings − net cost.
    pub net_profit_usd: f64,
    /// Net profit / net cost, percent.
    pub roi_pct: f64,
}

pub fn analyze(input: &SolarInput) -> SolarResult {
    let credit = input.system_cost_usd * input.federal_credit_pct / 100.0;
    let net_cost = (input.system_cost_usd - credit - input.other_incentives_usd).max(0.0);
    let inf = input.electricity_inflation_pct / 100.0;
    let deg = input.annual_degradation_pct / 100.0;
    let years = input.horizon_years.max(0.0) as i64;

    let mut cum = 0.0;
    let mut lifetime = 0.0;
    let mut payback = None;
    for y in 1..=years {
        let s = input.annual_savings_usd
            * (1.0 + inf).powi((y - 1) as i32)
            * (1.0 - deg).powi((y - 1) as i32);
        let prev = cum;
        cum += s;
        lifetime += s;
        if payback.is_none() && prev < net_cost && cum >= net_cost && s != 0.0 {
            payback = Some((y - 1) as f64 + (net_cost - prev) / s);
        }
    }
    // Net cost could be zero (incentives ≥ cost) → instant payback.
    if payback.is_none() && net_cost <= 0.0 {
        payback = Some(0.0);
    }

    SolarResult {
        net_cost_usd: net_cost,
        federal_credit_usd: credit,
        lifetime_savings_usd: lifetime,
        payback_years: payback,
        net_profit_usd: lifetime - net_cost,
        roi_pct: if net_cost > 0.0 {
            (lifetime - net_cost) / net_cost * 100.0
        } else {
            0.0
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-2
    }

    fn base() -> SolarInput {
        SolarInput {
            system_cost_usd: 30_000.0,
            federal_credit_pct: 30.0,
            other_incentives_usd: 0.0,
            annual_savings_usd: 2_000.0,
            annual_degradation_pct: 0.5,
            electricity_inflation_pct: 3.0,
            horizon_years: 25.0,
        }
    }

    #[test]
    fn net_cost_after_credit() {
        let r = analyze(&base());
        assert!(close(r.federal_credit_usd, 9_000.0));
        assert!(close(r.net_cost_usd, 21_000.0));
    }

    #[test]
    fn lifetime_savings() {
        assert!(close(analyze(&base()).lifetime_savings_usd, 68_182.96));
    }

    #[test]
    fn payback() {
        assert!(close(analyze(&base()).payback_years.unwrap(), 9.4422));
    }

    #[test]
    fn profit_and_roi() {
        let r = analyze(&base());
        assert!(close(r.net_profit_usd, 47_182.96));
        assert!(close(r.roi_pct, 224.6808));
    }

    #[test]
    fn flat_case_simple_payback() {
        // No inflation/degradation: 21,000 / 2,000 = 10.5 years.
        let r = analyze(&SolarInput {
            annual_degradation_pct: 0.0,
            electricity_inflation_pct: 0.0,
            ..base()
        });
        assert!(close(r.payback_years.unwrap(), 10.5));
        assert!(close(r.lifetime_savings_usd, 50_000.0));
    }

    #[test]
    fn incentives_lower_net_cost() {
        let r = analyze(&SolarInput {
            other_incentives_usd: 5_000.0,
            ..base()
        });
        assert!(close(r.net_cost_usd, 16_000.0));
        assert!(r.payback_years.unwrap() < analyze(&base()).payback_years.unwrap());
    }

    #[test]
    fn never_pays_back_short_horizon() {
        // 3-year horizon can't recover a 21k net cost at ~2k/yr.
        let r = analyze(&SolarInput {
            horizon_years: 3.0,
            ..base()
        });
        assert!(r.payback_years.is_none());
        assert!(r.net_profit_usd < 0.0);
    }

    #[test]
    fn higher_inflation_speeds_payback() {
        let low = analyze(&base());
        let high = analyze(&SolarInput {
            electricity_inflation_pct: 8.0,
            ..base()
        });
        assert!(high.payback_years.unwrap() < low.payback_years.unwrap());
    }
}
