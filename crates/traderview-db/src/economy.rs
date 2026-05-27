//! Economic calendar — rule-driven schedule of well-known US macro releases.
//!
//! No external data dependency: every release has a deterministic rule
//! ("2nd Tuesday of month at 8:30 ET", "every Thursday", etc.). We expand
//! the rules over the requested horizon and return the merged + sorted list.
//!
//! FOMC meeting dates are hand-curated through 2026 (officially published
//! by the Federal Reserve at federalreserve.gov/monetarypolicy/fomccalendars.htm).

use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Utc, Weekday};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Importance {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize)]
pub struct EconEvent {
    pub name: &'static str,
    pub category: &'static str,
    pub country: &'static str,
    pub importance: Importance,
    pub when_utc: NaiveDateTime, // approximate; ET-anchored sources
    pub when_et: NaiveDateTime,
    pub source: &'static str,
}

#[derive(Debug, Clone, Copy)]
enum Rule {
    NthWeekdayOfMonth {
        weekday: Weekday,
        occurrence: u8,
        time_et: (u32, u32),
    },
    LastWeekdayOfMonth {
        weekday: Weekday,
        time_et: (u32, u32),
    },
    EveryWeekday {
        weekday: Weekday,
        time_et: (u32, u32),
    },
    NthBusinessDay {
        n: u8,
        time_et: (u32, u32),
    },
}

struct Release {
    name: &'static str,
    category: &'static str,
    importance: Importance,
    rule: Rule,
}

const RELEASES: &[Release] = &[
    Release {
        name: "CPI",
        category: "Inflation",
        importance: Importance::High,
        rule: Rule::NthWeekdayOfMonth {
            weekday: Weekday::Tue,
            occurrence: 2,
            time_et: (8, 30),
        },
    },
    Release {
        name: "PPI",
        category: "Inflation",
        importance: Importance::Medium,
        rule: Rule::NthWeekdayOfMonth {
            weekday: Weekday::Wed,
            occurrence: 2,
            time_et: (8, 30),
        },
    },
    Release {
        name: "PCE Price Index",
        category: "Inflation",
        importance: Importance::High,
        rule: Rule::LastWeekdayOfMonth {
            weekday: Weekday::Fri,
            time_et: (8, 30),
        },
    },
    Release {
        name: "Non-Farm Payrolls (NFP)",
        category: "Employment",
        importance: Importance::High,
        rule: Rule::NthWeekdayOfMonth {
            weekday: Weekday::Fri,
            occurrence: 1,
            time_et: (8, 30),
        },
    },
    Release {
        name: "ADP Employment",
        category: "Employment",
        importance: Importance::Medium,
        rule: Rule::NthWeekdayOfMonth {
            weekday: Weekday::Wed,
            occurrence: 1,
            time_et: (8, 15),
        },
    },
    Release {
        name: "JOLTS Job Openings",
        category: "Employment",
        importance: Importance::Medium,
        rule: Rule::NthBusinessDay {
            n: 4,
            time_et: (10, 0),
        },
    },
    Release {
        name: "Initial Jobless Claims",
        category: "Employment",
        importance: Importance::Medium,
        rule: Rule::EveryWeekday {
            weekday: Weekday::Thu,
            time_et: (8, 30),
        },
    },
    Release {
        name: "Retail Sales",
        category: "Consumer",
        importance: Importance::Medium,
        rule: Rule::NthBusinessDay {
            n: 11,
            time_et: (8, 30),
        },
    },
    Release {
        name: "ISM Manufacturing PMI",
        category: "Manufacturing",
        importance: Importance::Medium,
        rule: Rule::NthBusinessDay {
            n: 1,
            time_et: (10, 0),
        },
    },
    Release {
        name: "ISM Services PMI",
        category: "Services",
        importance: Importance::Medium,
        rule: Rule::NthBusinessDay {
            n: 3,
            time_et: (10, 0),
        },
    },
    Release {
        name: "Consumer Confidence (CB)",
        category: "Consumer",
        importance: Importance::Medium,
        rule: Rule::LastWeekdayOfMonth {
            weekday: Weekday::Tue,
            time_et: (10, 0),
        },
    },
    Release {
        name: "Michigan Consumer Sentiment",
        category: "Consumer",
        importance: Importance::Medium,
        rule: Rule::NthWeekdayOfMonth {
            weekday: Weekday::Fri,
            occurrence: 2,
            time_et: (10, 0),
        },
    },
    Release {
        name: "Housing Starts",
        category: "Housing",
        importance: Importance::Low,
        rule: Rule::NthBusinessDay {
            n: 12,
            time_et: (8, 30),
        },
    },
    Release {
        name: "Existing Home Sales",
        category: "Housing",
        importance: Importance::Low,
        rule: Rule::NthBusinessDay {
            n: 15,
            time_et: (10, 0),
        },
    },
    Release {
        name: "New Home Sales",
        category: "Housing",
        importance: Importance::Low,
        rule: Rule::NthBusinessDay {
            n: 17,
            time_et: (10, 0),
        },
    },
    Release {
        name: "Durable Goods Orders",
        category: "Manufacturing",
        importance: Importance::Medium,
        rule: Rule::NthBusinessDay {
            n: 16,
            time_et: (8, 30),
        },
    },
];

// 2026 FOMC meetings (8 per year). Two-day meetings; the decision lands at
// 2pm ET on day 2.
const FOMC_2026: &[(i32, u32, u32)] = &[
    (2026, 1, 28),
    (2026, 3, 18),
    (2026, 4, 29),
    (2026, 6, 17),
    (2026, 7, 29),
    (2026, 9, 16),
    (2026, 10, 28),
    (2026, 12, 16),
];

pub fn upcoming(days: i64, importance_min: Importance) -> Vec<EconEvent> {
    let today = Utc::now().date_naive();
    let horizon = today + Duration::days(days);
    let mut out = Vec::new();

    for r in RELEASES {
        if importance_lt(r.importance, importance_min) {
            continue;
        }
        let (h, m) = match r.rule {
            Rule::NthWeekdayOfMonth { time_et, .. } => time_et,
            Rule::LastWeekdayOfMonth { time_et, .. } => time_et,
            Rule::EveryWeekday { time_et, .. } => time_et,
            Rule::NthBusinessDay { time_et, .. } => time_et,
        };
        for d in expand_rule(r.rule, today, horizon) {
            let t = NaiveTime::from_hms_opt(h, m, 0).unwrap();
            let et = NaiveDateTime::new(d, t);
            // ET → UTC: EST = UTC-5, EDT = UTC-4. We use a simple monthly
            // approximation: DST roughly Mar→Nov. Real users go by ET anyway.
            let offset_h = if (3..=11).contains(&d.month()) { 4 } else { 5 };
            let utc = et + Duration::hours(offset_h);
            out.push(EconEvent {
                name: r.name,
                category: r.category,
                country: "US",
                importance: r.importance,
                when_et: et,
                when_utc: utc,
                source: "rule",
            });
        }
    }
    // FOMC dates (high importance).
    if !importance_lt(Importance::High, importance_min) {
        for &(y, m, d) in FOMC_2026 {
            let date = NaiveDate::from_ymd_opt(y, m, d).unwrap();
            if date < today || date > horizon {
                continue;
            }
            let et = NaiveDateTime::new(date, NaiveTime::from_hms_opt(14, 0, 0).unwrap());
            let offset_h = if (3..=11).contains(&date.month()) {
                4
            } else {
                5
            };
            let utc = et + Duration::hours(offset_h);
            out.push(EconEvent {
                name: "FOMC Rate Decision",
                category: "Monetary Policy",
                country: "US",
                importance: Importance::High,
                when_et: et,
                when_utc: utc,
                source: "fomc",
            });
            // FOMC minutes ~ 3 weeks after.
            let min_date = date + Duration::days(21);
            if min_date <= horizon {
                let et = NaiveDateTime::new(min_date, NaiveTime::from_hms_opt(14, 0, 0).unwrap());
                let offset_h = if (3..=11).contains(&min_date.month()) {
                    4
                } else {
                    5
                };
                let utc = et + Duration::hours(offset_h);
                out.push(EconEvent {
                    name: "FOMC Minutes",
                    category: "Monetary Policy",
                    country: "US",
                    importance: Importance::Medium,
                    when_et: et,
                    when_utc: utc,
                    source: "fomc",
                });
            }
        }
    }

    out.sort_by_key(|e| e.when_utc);
    out
}

fn expand_rule(rule: Rule, from: NaiveDate, to: NaiveDate) -> Vec<NaiveDate> {
    let mut out = Vec::new();
    match rule {
        Rule::NthWeekdayOfMonth {
            weekday,
            occurrence,
            ..
        } => {
            let mut d = first_of_month(from);
            while d <= to {
                if let Some(target) = nth_weekday_of_month(d, weekday, occurrence) {
                    if target >= from && target <= to {
                        out.push(target);
                    }
                }
                d = next_month(d);
            }
        }
        Rule::LastWeekdayOfMonth { weekday, .. } => {
            let mut d = first_of_month(from);
            while d <= to {
                let lwd = last_weekday_of_month(d, weekday);
                if lwd >= from && lwd <= to {
                    out.push(lwd);
                }
                d = next_month(d);
            }
        }
        Rule::EveryWeekday { weekday, .. } => {
            let mut d = from;
            while d <= to {
                if d.weekday() == weekday {
                    out.push(d);
                }
                d = d.succ_opt().unwrap_or(to);
                if d == to {
                    if d.weekday() == weekday {
                        out.push(d);
                    }
                    break;
                }
            }
        }
        Rule::NthBusinessDay { n, .. } => {
            let mut d = first_of_month(from);
            while d <= to {
                let bd = nth_business_day(d, n);
                if bd >= from && bd <= to {
                    out.push(bd);
                }
                d = next_month(d);
            }
        }
    }
    out
}

fn first_of_month(d: NaiveDate) -> NaiveDate {
    NaiveDate::from_ymd_opt(d.year(), d.month(), 1).unwrap()
}

fn next_month(d: NaiveDate) -> NaiveDate {
    if d.month() == 12 {
        NaiveDate::from_ymd_opt(d.year() + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(d.year(), d.month() + 1, 1).unwrap()
    }
}

fn nth_weekday_of_month(
    any_in_month: NaiveDate,
    weekday: Weekday,
    occurrence: u8,
) -> Option<NaiveDate> {
    let first = first_of_month(any_in_month);
    let offset =
        (weekday.num_days_from_sunday() as i32 - first.weekday().num_days_from_sunday() as i32 + 7)
            % 7;
    let first_match = first + Duration::days(offset as i64);
    let day = first_match + Duration::days(7 * (occurrence as i64 - 1));
    if day.month() == any_in_month.month() {
        Some(day)
    } else {
        None
    }
}

fn last_weekday_of_month(any_in_month: NaiveDate, weekday: Weekday) -> NaiveDate {
    let last_day = if any_in_month.month() == 12 {
        NaiveDate::from_ymd_opt(any_in_month.year() + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(any_in_month.year(), any_in_month.month() + 1, 1).unwrap()
    } - Duration::days(1);
    let back = (last_day.weekday().num_days_from_sunday() as i32
        - weekday.num_days_from_sunday() as i32
        + 7)
        % 7;
    last_day - Duration::days(back as i64)
}

fn nth_business_day(any_in_month: NaiveDate, n: u8) -> NaiveDate {
    let mut d = first_of_month(any_in_month);
    let mut count = 0u8;
    loop {
        if !matches!(d.weekday(), Weekday::Sat | Weekday::Sun) {
            count += 1;
            if count == n {
                return d;
            }
        }
        d = d.succ_opt().unwrap();
        if d.month() != any_in_month.month() {
            return d - Duration::days(1);
        }
    }
}

fn importance_lt(a: Importance, b: Importance) -> bool {
    let n = |i| match i {
        Importance::Low => 0,
        Importance::Medium => 1,
        Importance::High => 2,
    };
    n(a) < n(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nth_weekday_correct() {
        // 2nd Tuesday of Jan 2026 = Jan 13 2026.
        let d = nth_weekday_of_month(
            NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            Weekday::Tue,
            2,
        )
        .unwrap();
        assert_eq!(d, NaiveDate::from_ymd_opt(2026, 1, 13).unwrap());
    }

    #[test]
    fn last_friday_correct() {
        // Last Friday of Feb 2026 = Feb 27.
        let d = last_weekday_of_month(NaiveDate::from_ymd_opt(2026, 2, 1).unwrap(), Weekday::Fri);
        assert_eq!(d, NaiveDate::from_ymd_opt(2026, 2, 27).unwrap());
    }

    #[test]
    fn upcoming_returns_sorted_list() {
        let evs = upcoming(60, Importance::Low);
        assert!(!evs.is_empty());
        let mut prev = evs[0].when_utc;
        for e in &evs[1..] {
            assert!(e.when_utc >= prev);
            prev = e.when_utc;
        }
    }
}
