//! Home maintenance budget projector.
//!
//! Two-part calculation:
//!
//!   1. The classic **1% rule** — set aside ~1% of home value each year
//!      for general maintenance + repairs (could be 0.5-2% depending on
//!      home age and condition).
//!   2. **Per-system replacement schedule** — major systems (roof, HVAC,
//!      water heater, appliances) have known service lives and
//!      replacement costs. For each system the user defines:
//!        - install_year, expected_life_years, replacement_cost_usd
//!      compute: years_until_replacement, monthly_set_aside =
//!      replacement_cost / months_until_replacement (clamped ≥ 0).
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SystemInput {
    pub name: String,
    pub install_year: i32,
    pub expected_life_years: u32,
    pub replacement_cost_usd: f64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct HomeMaintenanceInput {
    pub home_value_usd: f64,
    /// 1.0 = the classic 1% rule. Range typically 0.5 – 2.0.
    #[serde(default = "default_pct")]
    pub general_pct_of_value: f64,
    pub current_year: i32,
    #[serde(default)]
    pub systems: Vec<SystemInput>,
}

fn default_pct() -> f64 { 1.0 }

#[derive(Debug, Clone, Serialize)]
pub struct SystemResult {
    pub name: String,
    pub install_year: i32,
    pub expected_life_years: u32,
    pub end_of_life_year: i32,
    pub years_until_replacement: i32,
    pub replacement_cost_usd: f64,
    pub monthly_set_aside_usd: f64,
    pub status: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct HomeMaintenanceReport {
    pub home_value_usd: f64,
    pub general_annual_usd: f64,
    pub general_monthly_usd: f64,
    pub systems: Vec<SystemResult>,
    pub total_system_monthly_set_aside_usd: f64,
    pub total_monthly_budget_usd: f64,
    pub overdue_count: usize,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn system_status(years_until: i32) -> &'static str {
    if years_until < 0 { "overdue" }
    else if years_until <= 2 { "due_soon" }
    else { "ok" }
}

pub fn evaluate_system(s: &SystemInput, current_year: i32) -> SystemResult {
    let eol = s.install_year + s.expected_life_years as i32;
    let years_until = eol - current_year;
    let months_until = (years_until.max(0) * 12).max(1) as f64;
    let monthly_aside = s.replacement_cost_usd / months_until;
    let status = system_status(years_until);
    SystemResult {
        name: s.name.clone(),
        install_year: s.install_year,
        expected_life_years: s.expected_life_years,
        end_of_life_year: eol,
        years_until_replacement: years_until,
        replacement_cost_usd: s.replacement_cost_usd,
        monthly_set_aside_usd: monthly_aside,
        status,
    }
}

pub fn compute(input: &HomeMaintenanceInput) -> HomeMaintenanceReport {
    let general_annual = input.home_value_usd * input.general_pct_of_value / 100.0;
    let general_monthly = general_annual / 12.0;
    let systems: Vec<SystemResult> = input
        .systems
        .iter()
        .map(|s| evaluate_system(s, input.current_year))
        .collect();
    let total_system_monthly: f64 = systems.iter().map(|s| s.monthly_set_aside_usd).sum();
    let overdue = systems.iter().filter(|s| s.status == "overdue").count();
    HomeMaintenanceReport {
        home_value_usd: input.home_value_usd,
        general_annual_usd: general_annual,
        general_monthly_usd: general_monthly,
        systems,
        total_system_monthly_set_aside_usd: total_system_monthly,
        total_monthly_budget_usd: general_monthly + total_system_monthly,
        overdue_count: overdue,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sys(name: &str, install: i32, life: u32, cost: f64) -> SystemInput {
        SystemInput {
            name: name.into(),
            install_year: install,
            expected_life_years: life,
            replacement_cost_usd: cost,
        }
    }

    #[test]
    fn system_status_overdue() {
        assert_eq!(system_status(-1), "overdue");
        assert_eq!(system_status(-10), "overdue");
    }

    #[test]
    fn system_status_due_soon() {
        assert_eq!(system_status(0), "due_soon");
        assert_eq!(system_status(1), "due_soon");
        assert_eq!(system_status(2), "due_soon");
    }

    #[test]
    fn system_status_ok() {
        assert_eq!(system_status(5), "ok");
        assert_eq!(system_status(20), "ok");
    }

    #[test]
    fn evaluate_system_basic() {
        // Roof installed 2020, 25-year life, $15k → eol 2045.
        let r = evaluate_system(&sys("roof", 2020, 25, 15_000.0), 2026);
        assert_eq!(r.end_of_life_year, 2045);
        assert_eq!(r.years_until_replacement, 19);
        assert_eq!(r.status, "ok");
    }

    #[test]
    fn evaluate_system_overdue() {
        // HVAC installed 2000, 15-year life → eol 2015. In 2026 = overdue 11y.
        let r = evaluate_system(&sys("hvac", 2000, 15, 8_000.0), 2026);
        assert_eq!(r.years_until_replacement, -11);
        assert_eq!(r.status, "overdue");
    }

    #[test]
    fn evaluate_system_monthly_aside_basic() {
        // $15k over 25 years (300 months) = $50/mo
        let r = evaluate_system(&sys("roof", 2026, 25, 15_000.0), 2026);
        assert!((r.monthly_set_aside_usd - 50.0).abs() < 0.01);
    }

    #[test]
    fn evaluate_system_overdue_uses_one_month_floor() {
        // Overdue → months_until = 1 → monthly = full cost
        let r = evaluate_system(&sys("hvac", 2010, 10, 8_000.0), 2026);
        assert_eq!(r.monthly_set_aside_usd, 8_000.0);
    }

    #[test]
    fn compute_general_1pct_rule() {
        let r = compute(&HomeMaintenanceInput {
            home_value_usd: 500_000.0,
            general_pct_of_value: 1.0,
            current_year: 2026,
            systems: vec![],
        });
        assert_eq!(r.general_annual_usd, 5_000.0);
        assert!((r.general_monthly_usd - 416.67).abs() < 0.5);
    }

    #[test]
    fn compute_general_half_pct_rule() {
        let r = compute(&HomeMaintenanceInput {
            home_value_usd: 500_000.0,
            general_pct_of_value: 0.5,
            current_year: 2026,
            systems: vec![],
        });
        assert_eq!(r.general_annual_usd, 2_500.0);
    }

    #[test]
    fn compute_total_includes_systems() {
        let r = compute(&HomeMaintenanceInput {
            home_value_usd: 500_000.0,
            general_pct_of_value: 1.0,
            current_year: 2026,
            systems: vec![
                sys("roof", 2026, 25, 15_000.0),  // $50/mo
                sys("hvac", 2026, 15, 9_000.0),   // $50/mo
            ],
        });
        assert_eq!(r.systems.len(), 2);
        assert!((r.total_system_monthly_set_aside_usd - 100.0).abs() < 0.1);
        // Total = general 416.67 + system 100 = 516.67
        assert!((r.total_monthly_budget_usd - 516.67).abs() < 0.5);
    }

    #[test]
    fn compute_overdue_count() {
        let r = compute(&HomeMaintenanceInput {
            home_value_usd: 500_000.0,
            general_pct_of_value: 1.0,
            current_year: 2026,
            systems: vec![
                sys("roof",  2010, 25, 15_000.0),  // due 2035 (ok)
                sys("hvac",  2000, 15, 8_000.0),   // overdue
                sys("water", 2005, 12, 1_500.0),   // overdue
            ],
        });
        assert_eq!(r.overdue_count, 2);
    }

    #[test]
    fn compute_zero_home_value_safe() {
        let r = compute(&HomeMaintenanceInput {
            home_value_usd: 0.0,
            general_pct_of_value: 1.0,
            current_year: 2026,
            systems: vec![],
        });
        assert_eq!(r.general_monthly_usd, 0.0);
        assert_eq!(r.total_monthly_budget_usd, 0.0);
    }
}
