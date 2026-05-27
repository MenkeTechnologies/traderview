//! Vehicle mileage deduction — IRS standard mileage method.
//!
//! Two methods exist (standard mileage vs actual expenses); we compute the
//! standard-mileage number since it's vastly more common and doesn't need
//! receipt-level tracking. Users still get the raw miles + ratesheet for
//! the actual method.
//!
//! IRS publishes new rates each December for the upcoming year. The
//! mid-year revision happens occasionally (2022 had a Jul-1 split because
//! of gas prices) — those are modeled with two rows per year.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MileagePurpose {
    Business,
    Medical,
    Moving,        // Active-duty military only since TCJA
    Charitable,
}

#[derive(Debug, Clone, Copy)]
pub struct MileageRate {
    pub effective_from: NaiveDate,
    pub effective_to:   NaiveDate,
    pub business:   &'static str,   // cents/mile as a string for exact Decimal
    pub medical:    &'static str,
    pub moving:     &'static str,
    pub charitable: &'static str,   // Fixed by statute — only Congress can change
}

/// Multi-year table. Rates effective dates from IRS Notices.
/// 2024 Notice 2024-13, 2025 Notice 2025-5, 2026 estimated.
const RATES: &[MileageRate] = &[
    MileageRate {
        effective_from: date(2022, 1, 1), effective_to: date(2022, 6, 30),
        business: "0.585", medical: "0.18", moving: "0.18", charitable: "0.14",
    },
    MileageRate {
        // Mid-year revision: Jul 2022 due to gas prices (IRS Announcement 2022-13).
        effective_from: date(2022, 7, 1), effective_to: date(2022, 12, 31),
        business: "0.625", medical: "0.22", moving: "0.22", charitable: "0.14",
    },
    MileageRate {
        effective_from: date(2023, 1, 1), effective_to: date(2023, 12, 31),
        business: "0.655", medical: "0.22", moving: "0.22", charitable: "0.14",
    },
    MileageRate {
        effective_from: date(2024, 1, 1), effective_to: date(2024, 12, 31),
        business: "0.67",  medical: "0.21", moving: "0.21", charitable: "0.14",
    },
    MileageRate {
        effective_from: date(2025, 1, 1), effective_to: date(2025, 12, 31),
        business: "0.70",  medical: "0.21", moving: "0.21", charitable: "0.14",
    },
    MileageRate {
        effective_from: date(2026, 1, 1), effective_to: date(2026, 12, 31),
        // Pending IRS publication (Notice 2026-X expected Dec 2025).
        // Treating as +3¢ projection; users override when IRS publishes.
        business: "0.73",  medical: "0.22", moving: "0.22", charitable: "0.14",
    },
];

const fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    // const_panic on invalid args is fine — the table is hand-curated.
    match NaiveDate::from_ymd_opt(y, m, d) {
        Some(d) => d,
        None => panic!("invalid date literal in mileage rate table"),
    }
}

/// Look up the rate row in effect on a specific date. None if the date
/// predates 2022 or is past the last published year.
pub fn rate_on(day: NaiveDate) -> Option<MileageRate> {
    RATES.iter()
        .find(|r| day >= r.effective_from && day <= r.effective_to)
        .copied()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trip {
    pub date: NaiveDate,
    pub miles: Decimal,
    pub purpose: MileagePurpose,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MileageReport {
    pub total_miles: Decimal,
    pub business_miles: Decimal,
    pub medical_miles: Decimal,
    pub moving_miles: Decimal,
    pub charitable_miles: Decimal,
    /// Standard-mileage dollar deduction summed across trips.
    pub deduction_total: Decimal,
    pub deduction_business: Decimal,
    pub deduction_medical: Decimal,
    pub deduction_moving: Decimal,
    pub deduction_charitable: Decimal,
    /// Trips that fell outside the rate table (pre-2022 or future).
    pub unrated_trips: usize,
}

pub fn report(trips: &[Trip]) -> MileageReport {
    let mut r = MileageReport::default();
    for t in trips {
        r.total_miles += t.miles;
        match t.purpose {
            MileagePurpose::Business   => r.business_miles   += t.miles,
            MileagePurpose::Medical    => r.medical_miles    += t.miles,
            MileagePurpose::Moving     => r.moving_miles     += t.miles,
            MileagePurpose::Charitable => r.charitable_miles += t.miles,
        }
        let Some(rt) = rate_on(t.date) else {
            r.unrated_trips += 1;
            continue;
        };
        let rate_str = match t.purpose {
            MileagePurpose::Business   => rt.business,
            MileagePurpose::Medical    => rt.medical,
            MileagePurpose::Moving     => rt.moving,
            MileagePurpose::Charitable => rt.charitable,
        };
        let rate = Decimal::from_str(rate_str).unwrap();
        let amount = t.miles * rate;
        r.deduction_total += amount;
        match t.purpose {
            MileagePurpose::Business   => r.deduction_business   += amount,
            MileagePurpose::Medical    => r.deduction_medical    += amount,
            MileagePurpose::Moving     => r.deduction_moving     += amount,
            MileagePurpose::Charitable => r.deduction_charitable += amount,
        }
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }

    #[test]
    fn rate_on_resolves_2024() {
        let r = rate_on(NaiveDate::from_ymd_opt(2024, 6, 15).unwrap()).unwrap();
        assert_eq!(r.business, "0.67");
    }

    #[test]
    fn rate_on_handles_2022_midyear_split() {
        // Jun 30 = first-half rate (58.5¢).
        let h1 = rate_on(NaiveDate::from_ymd_opt(2022, 6, 30).unwrap()).unwrap();
        assert_eq!(h1.business, "0.585");
        // Jul 1 = revised rate (62.5¢).
        let h2 = rate_on(NaiveDate::from_ymd_opt(2022, 7, 1).unwrap()).unwrap();
        assert_eq!(h2.business, "0.625");
    }

    #[test]
    fn rate_on_returns_none_for_pre_2022() {
        let r = rate_on(NaiveDate::from_ymd_opt(2019, 1, 1).unwrap());
        assert!(r.is_none());
    }

    #[test]
    fn empty_trips_returns_zeros() {
        let r = report(&[]);
        assert_eq!(r.total_miles, Decimal::ZERO);
        assert_eq!(r.deduction_total, Decimal::ZERO);
    }

    #[test]
    fn single_business_trip_2024_uses_67_cents() {
        let trips = vec![Trip {
            date: NaiveDate::from_ymd_opt(2024, 6, 15).unwrap(),
            miles: d("1000"),
            purpose: MileagePurpose::Business,
            note: "Conference travel".into(),
        }];
        let r = report(&trips);
        // 1000 mi × $0.67 = $670.
        assert_eq!(r.deduction_total, d("670.00"));
        assert_eq!(r.deduction_business, d("670.00"));
        assert_eq!(r.business_miles, d("1000"));
        assert_eq!(r.unrated_trips, 0);
    }

    #[test]
    fn trip_in_unrated_year_still_counts_miles() {
        // Pre-2022 trip — miles count but no dollar amount.
        let trips = vec![Trip {
            date: NaiveDate::from_ymd_opt(2019, 6, 15).unwrap(),
            miles: d("500"),
            purpose: MileagePurpose::Business,
            note: "".into(),
        }];
        let r = report(&trips);
        assert_eq!(r.business_miles, d("500"));
        assert_eq!(r.deduction_total, Decimal::ZERO);
        assert_eq!(r.unrated_trips, 1);
    }

    #[test]
    fn mixed_purpose_trips_compute_each_rate_separately() {
        let trips = vec![
            Trip { date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                   miles: d("100"), purpose: MileagePurpose::Business,   note: "".into() },
            Trip { date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                   miles: d("50"),  purpose: MileagePurpose::Medical,    note: "".into() },
            Trip { date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                   miles: d("200"), purpose: MileagePurpose::Charitable, note: "".into() },
        ];
        let r = report(&trips);
        // 100×0.67=67 + 50×0.21=10.50 + 200×0.14=28 = 105.50
        assert_eq!(r.deduction_business,   d("67.00"));
        assert_eq!(r.deduction_medical,    d("10.50"));
        assert_eq!(r.deduction_charitable, d("28.00"));
        assert_eq!(r.deduction_total,      d("105.50"));
    }

    #[test]
    fn rate_table_covers_2022_through_2026() {
        // Multi-year amended returns matter. Every year between 2022 and
        // current+1 must resolve for January, June, December.
        for y in 2022..=2026 {
            for m in [1, 6, 12] {
                let d = NaiveDate::from_ymd_opt(y, m, 15).unwrap();
                assert!(rate_on(d).is_some(),
                    "missing rate for {y}-{m:02}-15");
            }
        }
    }

    #[test]
    fn charitable_rate_is_pinned_at_14_cents() {
        // Charitable rate is fixed by statute (IRC §170) since 1998.
        // Every row in the table must report 0.14.
        for r in RATES {
            assert_eq!(r.charitable, "0.14",
                "charitable rate changed in {} — verify Congress passed a law", r.effective_from);
        }
    }
}
