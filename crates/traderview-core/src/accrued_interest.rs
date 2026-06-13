//! Bond accrued interest and dirty (full) price.
//!
//! Between coupon dates a bond's buyer owes the seller the interest that has
//! accrued since the last coupon. The quoted *clean* price excludes it; the
//! *dirty* price the buyer actually pays adds it back:
//!
//! ```text
//! accrued    = coupon_payment × (days_accrued / days_in_period)
//! dirty_price = clean_price + accrued
//! ```
//!
//! Two day-count conventions are supported: 30/360 (US corporate/municipal
//! bond basis) and Actual/Actual (US Treasuries), selected per the security.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DayCount {
    /// 30/360 bond basis (corporates, munis).
    Thirty360,
    /// Actual/Actual on the coupon period (Treasuries).
    ActualActual,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccruedInput {
    /// Par / face value the coupon is computed on (e.g. 1000.0).
    pub face_value: f64,
    /// Annual coupon rate in percent (e.g. 5.0 for a 5% coupon).
    pub coupon_rate_pct: f64,
    /// Coupon payments per year (1, 2, 4, or 12).
    pub frequency: u32,
    /// Quoted price excluding accrued interest, in the same units as face.
    pub clean_price: f64,
    /// Previous coupon date, ISO `YYYY-MM-DD`.
    pub last_coupon: String,
    /// Next coupon date, ISO `YYYY-MM-DD`.
    pub next_coupon: String,
    /// Trade settlement date, ISO `YYYY-MM-DD`.
    pub settlement: String,
    pub day_count: DayCount,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AccruedResult {
    /// Full coupon paid each period (face × rate / frequency).
    pub coupon_payment: f64,
    /// Days accrued from last coupon to settlement, per the convention.
    pub days_accrued: f64,
    /// Days in the full coupon period, per the convention.
    pub days_in_period: f64,
    /// days_accrued / days_in_period.
    pub accrual_fraction: f64,
    /// Interest accrued since the last coupon.
    pub accrued_interest: f64,
    /// clean_price (echoed for the UI).
    pub clean_price: f64,
    /// clean_price + accrued_interest — what the buyer pays.
    pub dirty_price: f64,
}

/// 30/360 (bond basis) day count between two dates.
fn days_30_360(start: NaiveDate, end: NaiveDate) -> f64 {
    use chrono::Datelike;
    let (y1, m1, mut d1) = (start.year(), start.month() as i32, start.day() as i32);
    let (y2, m2, mut d2) = (end.year(), end.month() as i32, end.day() as i32);
    if d1 == 31 {
        d1 = 30;
    }
    if d2 == 31 && d1 == 30 {
        d2 = 30;
    }
    (360 * (y2 - y1) + 30 * (m2 - m1) + (d2 - d1)) as f64
}

fn parse(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

pub fn analyze(input: &AccruedInput) -> AccruedResult {
    let freq = input.frequency.max(1) as f64;
    let coupon_payment = input.face_value * (input.coupon_rate_pct / 100.0) / freq;

    let last = parse(&input.last_coupon);
    let next = parse(&input.next_coupon);
    let settle = parse(&input.settlement);

    let (days_accrued, days_in_period) = match (last, next, settle) {
        (Some(last), Some(next), Some(settle)) => match input.day_count {
            DayCount::Thirty360 => {
                (days_30_360(last, settle), days_30_360(last, next))
            }
            DayCount::ActualActual => (
                (settle - last).num_days() as f64,
                (next - last).num_days() as f64,
            ),
        },
        _ => (0.0, 0.0),
    };

    let fraction = if days_in_period != 0.0 {
        days_accrued / days_in_period
    } else {
        0.0
    };
    let accrued = coupon_payment * fraction;

    AccruedResult {
        coupon_payment,
        days_accrued,
        days_in_period,
        accrual_fraction: fraction,
        accrued_interest: accrued,
        clean_price: input.clean_price,
        dirty_price: input.clean_price + accrued,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn run(day_count: DayCount, last: &str, next: &str, settle: &str) -> AccruedResult {
        analyze(&AccruedInput {
            face_value: 1000.0,
            coupon_rate_pct: 6.0,
            frequency: 2,
            clean_price: 980.0,
            last_coupon: last.into(),
            next_coupon: next.into(),
            settlement: settle.into(),
            day_count,
        })
    }

    #[test]
    fn coupon_payment_semiannual() {
        // 6% on 1000, twice a year → 30 per coupon.
        let r = run(DayCount::Thirty360, "2026-01-01", "2026-07-01", "2026-01-01");
        assert!(close(r.coupon_payment, 30.0));
    }

    #[test]
    fn thirty_360_half_period() {
        // Jan 1 → Jul 1 is 180 days 30/360; settle Apr 1 is 90 → half.
        let r = run(DayCount::Thirty360, "2026-01-01", "2026-07-01", "2026-04-01");
        assert!(close(r.days_in_period, 180.0));
        assert!(close(r.days_accrued, 90.0));
        assert!(close(r.accrual_fraction, 0.5));
        assert!(close(r.accrued_interest, 15.0));
    }

    #[test]
    fn dirty_is_clean_plus_accrued() {
        let r = run(DayCount::Thirty360, "2026-01-01", "2026-07-01", "2026-04-01");
        assert!(close(r.dirty_price, 980.0 + 15.0));
        assert!(close(r.clean_price, 980.0));
    }

    #[test]
    fn settlement_on_last_coupon_is_zero() {
        let r = run(DayCount::Thirty360, "2026-01-01", "2026-07-01", "2026-01-01");
        assert!(close(r.accrued_interest, 0.0));
        assert!(close(r.dirty_price, 980.0));
    }

    #[test]
    fn settlement_on_next_coupon_is_full() {
        // Just before the next coupon, nearly the whole period has accrued.
        let r = run(DayCount::Thirty360, "2026-01-01", "2026-07-01", "2026-07-01");
        assert!(close(r.accrual_fraction, 1.0));
        assert!(close(r.accrued_interest, 30.0));
    }

    #[test]
    fn actual_actual_uses_calendar_days() {
        // Jan 1 → Jul 1 2026 actual = 181 days; settle Feb 1 = 31 days.
        let r = run(DayCount::ActualActual, "2026-01-01", "2026-07-01", "2026-02-01");
        assert!(close(r.days_in_period, 181.0));
        assert!(close(r.days_accrued, 31.0));
        assert!(close(r.accrued_interest, 30.0 * 31.0 / 181.0));
    }

    #[test]
    fn thirty_360_treats_31st_as_30th() {
        // Jan 31 → settle Feb 28: D1=31→30, so 30/360 = 28 days.
        let r = analyze(&AccruedInput {
            face_value: 1000.0,
            coupon_rate_pct: 6.0,
            frequency: 2,
            clean_price: 1000.0,
            last_coupon: "2026-01-31".into(),
            next_coupon: "2026-07-31".into(),
            settlement: "2026-02-28".into(),
            day_count: DayCount::Thirty360,
        });
        assert!(close(r.days_accrued, 28.0));
        assert!(close(r.days_in_period, 180.0));
    }

    #[test]
    fn bad_dates_guard_to_zero() {
        let r = run(DayCount::Thirty360, "not-a-date", "2026-07-01", "2026-04-01");
        assert!(close(r.accrued_interest, 0.0));
        assert!(close(r.dirty_price, 980.0));
    }
}
