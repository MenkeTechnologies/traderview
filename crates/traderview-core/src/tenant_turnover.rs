//! Tenant turnover cost — the all-in cost a landlord absorbs when a unit turns
//! over: rent lost while the unit sits vacant, make-ready (cleaning, paint,
//! repairs), leasing/marketing to find the next tenant, and any move-in
//! concession. It totals these and expresses the result as a percentage of annual
//! rent and as months of rent — the metric that shows why minimizing turnover
//! matters. Distinct from the single-property NOI and the rent roll. Not advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct TurnoverInput {
    pub property_label: String,
    pub monthly_rent_usd: f64,
    /// Days the unit is vacant between tenants.
    #[serde(default)]
    pub vacancy_days: f64,
    /// Make-ready cost (cleaning, paint, repairs).
    #[serde(default)]
    pub make_ready_usd: f64,
    /// Leasing / marketing cost (commission, listing, screening).
    #[serde(default)]
    pub leasing_cost_usd: f64,
    /// Move-in concession granted to the new tenant (free rent, signing bonus).
    #[serde(default)]
    pub concession_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct TurnoverReport {
    /// Annual rent ÷ 365.
    pub daily_rent_usd: f64,
    /// Daily rent × vacancy days.
    pub lost_rent_usd: f64,
    /// Lost rent + make-ready + leasing + concession.
    pub total_turnover_cost_usd: f64,
    /// Total ÷ annual rent, percent.
    pub pct_of_annual_rent: f64,
    /// Total ÷ monthly rent — turnover cost expressed in months of rent.
    pub months_of_rent: f64,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

pub fn generate(i: &TurnoverInput) -> TurnoverReport {
    if i.monthly_rent_usd <= 0.0 {
        return TurnoverReport::default();
    }
    let daily = i.monthly_rent_usd * 12.0 / 365.0;
    let lost = daily * i.vacancy_days;
    let total = lost + i.make_ready_usd + i.leasing_cost_usd + i.concession_usd;
    let annual = i.monthly_rent_usd * 12.0;
    TurnoverReport {
        daily_rent_usd: round4(daily),
        lost_rent_usd: cents(lost),
        total_turnover_cost_usd: cents(total),
        pct_of_annual_rent: round2(total / annual * 100.0),
        months_of_rent: round2(total / i.monthly_rent_usd),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> TurnoverInput {
        TurnoverInput {
            property_label: "Unit 4B".into(),
            monthly_rent_usd: 1_500.0,
            vacancy_days: 30.0,
            make_ready_usd: 800.0,
            leasing_cost_usd: 750.0,
            concession_usd: 500.0,
        }
    }

    #[test]
    fn turnover_total_and_ratios() {
        let d = generate(&base());
        assert!(close(d.lost_rent_usd, 1_479.45));
        assert!(close(d.total_turnover_cost_usd, 3_529.45));
        assert!(close(d.pct_of_annual_rent, 19.61));
        assert!(close(d.months_of_rent, 2.35));
    }

    #[test]
    fn longer_vacancy_higher_cost() {
        let short = generate(&base());
        let long = generate(&TurnoverInput { vacancy_days: 60.0, ..base() });
        assert!(long.total_turnover_cost_usd > short.total_turnover_cost_usd);
    }

    #[test]
    fn no_vacancy_only_fixed_costs() {
        let d = generate(&TurnoverInput { vacancy_days: 0.0, ..base() });
        assert!(close(d.lost_rent_usd, 0.0));
        // 800 + 750 + 500 = 2,050.
        assert!(close(d.total_turnover_cost_usd, 2_050.0));
    }

    #[test]
    fn months_consistent_with_total() {
        let d = generate(&base());
        assert!(close(d.months_of_rent, d.total_turnover_cost_usd / 1_500.0));
    }

    #[test]
    fn zero_rent_empty() {
        let d = generate(&TurnoverInput { monthly_rent_usd: 0.0, ..base() });
        assert!(close(d.total_turnover_cost_usd, 0.0));
    }
}
