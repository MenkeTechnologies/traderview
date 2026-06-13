//! Overtime pay — weekly gross from regular, overtime, and double-time hours.
//!
//! Under the FLSA, hours over 40 in a week are paid at 1.5× the regular rate;
//! some states/contracts add 2× double-time. This sums the three buckets and
//! reports the blended effective hourly and the annualized gross.
//!
//! ```text
//! weekly gross = regular·rate + OT·rate·1.5 + DT·rate·2
//! effective hourly = weekly gross / total hours
//! ```

use serde::{Deserialize, Serialize};

fn d_ot() -> f64 {
    1.5
}
fn d_dt() -> f64 {
    2.0
}
fn d_weeks() -> f64 {
    52.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct OvertimeInput {
    pub hourly_rate_usd: f64,
    pub regular_hours: f64,
    #[serde(default)]
    pub overtime_hours: f64,
    #[serde(default)]
    pub double_time_hours: f64,
    #[serde(default = "d_ot")]
    pub overtime_multiplier: f64,
    #[serde(default = "d_dt")]
    pub double_time_multiplier: f64,
    #[serde(default = "d_weeks")]
    pub weeks_per_year: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct OvertimeResult {
    pub regular_pay_usd: f64,
    pub overtime_pay_usd: f64,
    pub double_time_pay_usd: f64,
    pub weekly_gross_usd: f64,
    pub total_hours: f64,
    /// Weekly gross / total hours — the blended effective rate.
    pub effective_hourly_usd: f64,
    /// The extra pay from the overtime/double-time premium over straight time.
    pub premium_pay_usd: f64,
    pub annual_gross_usd: f64,
}

pub fn analyze(input: &OvertimeInput) -> OvertimeResult {
    let rate = input.hourly_rate_usd;
    let regular = input.regular_hours * rate;
    let ot = input.overtime_hours * rate * input.overtime_multiplier;
    let dt = input.double_time_hours * rate * input.double_time_multiplier;
    let weekly = regular + ot + dt;

    let total_hours = input.regular_hours + input.overtime_hours + input.double_time_hours;
    let effective = if total_hours > 0.0 {
        weekly / total_hours
    } else {
        0.0
    };
    // Premium = pay above what every hour at straight time would have cost.
    let straight = total_hours * rate;
    let premium = weekly - straight;

    OvertimeResult {
        regular_pay_usd: regular,
        overtime_pay_usd: ot,
        double_time_pay_usd: dt,
        weekly_gross_usd: weekly,
        total_hours,
        effective_hourly_usd: effective,
        premium_pay_usd: premium,
        annual_gross_usd: weekly * input.weeks_per_year,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn run(rate: f64, reg: f64, ot: f64, dt: f64) -> OvertimeResult {
        analyze(&OvertimeInput {
            hourly_rate_usd: rate,
            regular_hours: reg,
            overtime_hours: ot,
            double_time_hours: dt,
            overtime_multiplier: 1.5,
            double_time_multiplier: 2.0,
            weeks_per_year: 52.0,
        })
    }

    #[test]
    fn regular_and_overtime() {
        let r = run(20.0, 40.0, 10.0, 0.0);
        assert!(close(r.regular_pay_usd, 800.0));
        // 10 × 20 × 1.5 = 300.
        assert!(close(r.overtime_pay_usd, 300.0));
        assert!(close(r.weekly_gross_usd, 1_100.0));
    }

    #[test]
    fn double_time() {
        let r = run(20.0, 40.0, 0.0, 5.0);
        // 5 × 20 × 2 = 200.
        assert!(close(r.double_time_pay_usd, 200.0));
        assert!(close(r.weekly_gross_usd, 1_000.0));
    }

    #[test]
    fn effective_hourly_blends() {
        let r = run(20.0, 40.0, 10.0, 0.0);
        // 1,100 / 50 hours = 22.
        assert!(close(r.effective_hourly_usd, 22.0));
    }

    #[test]
    fn premium_is_pay_above_straight_time() {
        let r = run(20.0, 40.0, 10.0, 0.0);
        // Straight time on 50h = 1,000; weekly 1,100 → 100 premium.
        assert!(close(r.premium_pay_usd, 100.0));
    }

    #[test]
    fn total_hours() {
        let r = run(20.0, 40.0, 10.0, 5.0);
        assert!(close(r.total_hours, 55.0));
    }

    #[test]
    fn annual_gross() {
        let r = run(20.0, 40.0, 10.0, 0.0);
        assert!(close(r.annual_gross_usd, 1_100.0 * 52.0));
    }

    #[test]
    fn no_overtime_equals_straight() {
        let r = run(25.0, 40.0, 0.0, 0.0);
        assert!(close(r.weekly_gross_usd, 1_000.0));
        assert!(close(r.premium_pay_usd, 0.0));
        assert!(close(r.effective_hourly_usd, 25.0));
    }

    #[test]
    fn double_time_pays_more_than_overtime() {
        let ot = run(20.0, 40.0, 5.0, 0.0);
        let dt = run(20.0, 40.0, 0.0, 5.0);
        assert!(dt.weekly_gross_usd > ot.weekly_gross_usd);
    }
}
