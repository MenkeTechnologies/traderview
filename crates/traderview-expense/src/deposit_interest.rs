//! State-specific security-deposit-interest rules for residential landlords.
//!
//! Most US states do NOT require landlords to pay interest on security
//! deposits. A handful — concentrated in the Northeast plus IL, NJ, MD —
//! do require it, and the formula varies (CPI-linked, fixed statutory
//! rate, or the actual bank rate on the holding account).
//!
//! Sources cited in `Citation::source` per row. Numbers are statutory
//! references; the *rate values* change year to year and are stamped
//! into each rule with the year they apply to. Update annually.
//!
//! Pure data + compute. Caller passes the state code + deposit + holding
//! period and we return the interest owed plus the citation so the
//! landlord can show their work.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub statute: &'static str,
    pub source: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateRule {
    /// USPS two-letter state code.
    pub state: &'static str,
    /// True when state law actually requires interest.
    pub required: bool,
    /// Most recent published rate at compile time. Annual rate (0.0001
    /// scale). Update yearly per state regulator publication.
    pub annual_rate: Decimal,
    /// Threshold below which interest is not required (e.g. IL exempts
    /// landlords under 25 units; we don't model that here).
    pub min_holding_months: u32,
    /// Year the rate above was published / effective.
    pub effective_year: i32,
    /// Free-text condition the caller MUST honor manually (e.g. "only
    /// applies to buildings of 5+ units in IL").
    pub notes: &'static str,
    pub citation: Citation,
}

/// Annual snapshot. Update yearly as state regulators publish. Order
/// follows USPS state codes alphabetically for readability.
fn rules() -> &'static [StateRule] {
    static R: once_cell::sync::Lazy<Vec<StateRule>> = once_cell::sync::Lazy::new(|| {
        let d = |s: &str| Decimal::from_str(s).unwrap();
        vec![
            StateRule {
                state: "CT",
                required: true,
                annual_rate: d("0.0145"),
                min_holding_months: 1,
                effective_year: 2024,
                notes: "Annual rate set by Banking Commissioner; deposit must be in escrow.",
                citation: Citation {
                    statute: "Conn. Gen. Stat. §47a-21(i)",
                    source: "https://www.cga.ct.gov/current/pub/chap_830.htm",
                },
            },
            StateRule {
                state: "DC",
                required: true,
                annual_rate: d("0.0006"),
                min_holding_months: 12,
                effective_year: 2024,
                notes: "Interest paid every 6 months at the federally insured passbook savings rate; this is a sample 2024 figure.",
                citation: Citation {
                    statute: "D.C. Code §42-3502.17",
                    source: "https://code.dccouncil.us/dc/council/code/sections/42-3502.17.html",
                },
            },
            StateRule {
                state: "FL",
                required: true,
                annual_rate: d("0"),
                min_holding_months: 0,
                effective_year: 2024,
                notes: "FL requires interest ONLY if landlord chooses to keep deposit in interest-bearing account; rate = 75% of bank rate or 5%/yr at landlord election. Caller supplies actual rate.",
                citation: Citation {
                    statute: "Fla. Stat. §83.49(2)(b)",
                    source: "https://www.flsenate.gov/Laws/Statutes/2024/0083.49",
                },
            },
            StateRule {
                state: "IA",
                required: true,
                annual_rate: d("0"),
                min_holding_months: 5,
                effective_year: 2024,
                notes: "IA: interest accrues to landlord during first 5 years of tenancy; after 5 years tenant entitled to interest above the rate landlord earned. Custom calc — pass actual bank rate.",
                citation: Citation {
                    statute: "Iowa Code §562A.12(2)",
                    source: "https://www.legis.iowa.gov/docs/code/562A.12.pdf",
                },
            },
            StateRule {
                state: "IL",
                required: true,
                annual_rate: d("0.0001"),
                min_holding_months: 6,
                effective_year: 2024,
                notes: "Applies only to buildings with 25+ units. Rate = passbook savings rate.",
                citation: Citation {
                    statute: "765 ILCS 710/1",
                    source: "https://www.ilga.gov/legislation/ilcs/ilcs3.asp?ActID=2204",
                },
            },
            StateRule {
                state: "MA",
                required: true,
                annual_rate: d("0.0005"),
                min_holding_months: 12,
                effective_year: 2024,
                notes: "Held in MA bank, separate account. Pay interest annually at rate paid by bank, or 5% if not in interest account.",
                citation: Citation {
                    statute: "Mass. Gen. Laws ch.186 §15B(3)(b)",
                    source: "https://malegislature.gov/Laws/GeneralLaws/PartII/TitleI/Chapter186/Section15B",
                },
            },
            StateRule {
                state: "MD",
                required: true,
                annual_rate: d("0.015"),
                min_holding_months: 6,
                effective_year: 2024,
                notes: "Statutory minimum 1.5%/yr or US Treasury 1-year rate as of Jan 1, whichever is higher; compounded semiannually.",
                citation: Citation {
                    statute: "Md. Code Real Prop. §8-203(e)(2)",
                    source: "https://mgaleg.maryland.gov/mgawebsite/Laws/StatuteText?article=gre&section=8-203",
                },
            },
            StateRule {
                state: "MN",
                required: true,
                annual_rate: d("0.01"),
                min_holding_months: 0,
                effective_year: 2024,
                notes: "1% simple interest per year, payable on return of deposit.",
                citation: Citation {
                    statute: "Minn. Stat. §504B.178(2)",
                    source: "https://www.revisor.mn.gov/statutes/cite/504B.178",
                },
            },
            StateRule {
                state: "NH",
                required: true,
                annual_rate: d("0"),
                min_holding_months: 12,
                effective_year: 2024,
                notes: "Required only if deposit held > 1 year, and only for buildings of 6+ units. Rate = bank's actual rate.",
                citation: Citation {
                    statute: "N.H. Rev. Stat. §540-A:6(IV)",
                    source: "https://www.gencourt.state.nh.us/rsa/html/LV/540-A/540-A-6.htm",
                },
            },
            StateRule {
                state: "NJ",
                required: true,
                annual_rate: d("0"),
                min_holding_months: 0,
                effective_year: 2024,
                notes: "Held in interest-bearing money market or insured account; tenant entitled to actual interest minus landlord's 1% fee. Caller supplies actual bank rate.",
                citation: Citation {
                    statute: "N.J. Stat. §46:8-19",
                    source: "https://lis.njleg.state.nj.us/nxt/gateway.dll?f=templates&fn=default.htm&vid=Publish:10.1048/Enu",
                },
            },
            StateRule {
                state: "NY",
                required: true,
                annual_rate: d("0"),
                min_holding_months: 0,
                effective_year: 2024,
                notes: "Buildings of 6+ units. Held in NY bank; tenant entitled to bank's rate minus landlord's 1% administrative fee. Caller supplies actual bank rate.",
                citation: Citation {
                    statute: "N.Y. Gen. Oblig. §7-103",
                    source: "https://www.nysenate.gov/legislation/laws/GOB/7-103",
                },
            },
            StateRule {
                state: "PA",
                required: true,
                annual_rate: d("0"),
                min_holding_months: 24,
                effective_year: 2024,
                notes: "Interest required only after 2 years; rate = institution's rate minus 1% landlord fee. Caller supplies actual rate.",
                citation: Citation {
                    statute: "68 Pa. Cons. Stat. §250.511b",
                    source: "https://www.legis.state.pa.us/cfdocs/legis/LI/uconsCheck.cfm?txtType=HTM&yr=1951&sessInd=0&smthLwInd=0&act=20&chpt=5",
                },
            },
            StateRule {
                state: "RI",
                required: false,
                annual_rate: Decimal::ZERO,
                min_holding_months: 0,
                effective_year: 2024,
                notes: "No state-wide requirement.",
                citation: Citation {
                    statute: "R.I. Gen. Laws §34-18-19",
                    source: "http://webserver.rilegislature.gov/Statutes/TITLE34/34-18/34-18-19.HTM",
                },
            },
        ]
    });
    &R
}

pub fn rule_for(state: &str) -> Option<&'static StateRule> {
    let upper = state.to_uppercase();
    rules().iter().find(|r| r.state.eq_ignore_ascii_case(&upper))
}

/// Catch-all for the 36 US states with no interest requirement.
pub fn no_requirement(state: &str) -> StateRule {
    StateRule {
        // SAFETY: stored as &'static str in the const table; for the
        // dynamic catch-all we leak the state name to get a 'static slice.
        state: Box::leak(state.to_uppercase().into_boxed_str()),
        required: false,
        annual_rate: Decimal::ZERO,
        min_holding_months: 0,
        effective_year: 2024,
        notes: "No state-wide security-deposit-interest requirement.",
        citation: Citation {
            statute: "",
            source: "",
        },
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccrualInput {
    pub state: String,
    pub deposit: Decimal,
    pub deposit_date: NaiveDate,
    pub through_date: NaiveDate,
    /// Optional override of the statutory rate — required for NY/NJ/PA/IA
    /// and any state where the rule_for row is `annual_rate == 0` because
    /// the actual rate comes from the landlord's bank account, not the
    /// statute.
    pub override_annual_rate: Option<Decimal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccrualResult {
    pub required: bool,
    pub interest_owed: Decimal,
    pub days_held: i64,
    pub applied_annual_rate: Decimal,
    pub statute: String,
    pub source: String,
    pub notes: String,
}

/// Simple-interest accrual: deposit × rate × (days_held / 365).
/// States that mandate compounding (MD semiannual) require a follow-up
/// computation by the caller — we expose the rate so they can.
pub fn accrue(input: &AccrualInput) -> AccrualResult {
    let rule_owned = rule_for(&input.state)
        .cloned()
        .unwrap_or_else(|| no_requirement(&input.state));
    let days = (input.through_date - input.deposit_date).num_days().max(0);
    let months = (days as f64 / 30.4375) as u32;

    if !rule_owned.required || months < rule_owned.min_holding_months {
        return AccrualResult {
            required: false,
            interest_owed: Decimal::ZERO,
            days_held: days,
            applied_annual_rate: Decimal::ZERO,
            statute: rule_owned.citation.statute.to_string(),
            source: rule_owned.citation.source.to_string(),
            notes: if months < rule_owned.min_holding_months {
                format!(
                    "deposit held {months}mo < required {}mo",
                    rule_owned.min_holding_months
                )
            } else {
                rule_owned.notes.to_string()
            },
        };
    }

    let rate = input
        .override_annual_rate
        .filter(|r| *r > Decimal::ZERO)
        .unwrap_or(rule_owned.annual_rate);
    let owed = (input.deposit * rate * Decimal::from(days) / Decimal::from(365)).round_dp(2);

    AccrualResult {
        required: true,
        interest_owed: owed.max(Decimal::ZERO),
        days_held: days,
        applied_annual_rate: rate,
        statute: rule_owned.citation.statute.to_string(),
        source: rule_owned.citation.source.to_string(),
        notes: rule_owned.notes.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn d(y: i32, m: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, day).unwrap()
    }

    #[test]
    fn texas_has_no_requirement() {
        let r = accrue(&AccrualInput {
            state: "TX".into(),
            deposit: dec!(2000),
            deposit_date: d(2023, 1, 1),
            through_date: d(2024, 1, 1),
            override_annual_rate: None,
        });
        assert!(!r.required);
        assert_eq!(r.interest_owed, Decimal::ZERO);
    }

    #[test]
    fn minnesota_one_percent_simple_interest_full_year() {
        // MN: 1% simple, no min holding. $2000 × 0.01 × 365/365 = $20.00
        let r = accrue(&AccrualInput {
            state: "MN".into(),
            deposit: dec!(2000),
            deposit_date: d(2023, 1, 1),
            through_date: d(2024, 1, 1),
            override_annual_rate: None,
        });
        assert!(r.required);
        assert_eq!(r.applied_annual_rate, dec!(0.01));
        assert_eq!(r.interest_owed, dec!(20.00));
        assert_eq!(r.days_held, 365);
    }

    #[test]
    fn maryland_one_point_five_percent_full_year() {
        // MD: 1.5% min. $3000 × 0.015 × 365/365 = $45.00
        let r = accrue(&AccrualInput {
            state: "MD".into(),
            deposit: dec!(3000),
            deposit_date: d(2023, 1, 1),
            through_date: d(2024, 1, 1),
            override_annual_rate: None,
        });
        assert_eq!(r.applied_annual_rate, dec!(0.015));
        assert_eq!(r.interest_owed, dec!(45.00));
    }

    #[test]
    fn ny_uses_override_rate_because_statute_is_bank_rate() {
        // NY: actual bank rate; statute is 0. Caller passes the rate.
        let r = accrue(&AccrualInput {
            state: "NY".into(),
            deposit: dec!(2500),
            deposit_date: d(2023, 1, 1),
            through_date: d(2024, 1, 1),
            override_annual_rate: Some(dec!(0.045)),
        });
        assert_eq!(r.applied_annual_rate, dec!(0.045));
        // $2500 × 0.045 × 365/365 = $112.50
        assert_eq!(r.interest_owed, dec!(112.50));
    }

    #[test]
    fn pa_requires_two_year_hold_before_interest_owed() {
        // PA: nothing for the first 24 months.
        let r = accrue(&AccrualInput {
            state: "PA".into(),
            deposit: dec!(1500),
            deposit_date: d(2023, 1, 1),
            through_date: d(2024, 6, 1),
            override_annual_rate: Some(dec!(0.04)),
        });
        assert!(!r.required);
        assert_eq!(r.interest_owed, Decimal::ZERO);
        assert!(r.notes.contains("required") || r.notes.contains("24mo"));
    }

    #[test]
    fn case_insensitive_state_lookup() {
        let r = accrue(&AccrualInput {
            state: "mn".into(),
            deposit: dec!(2000),
            deposit_date: d(2023, 1, 1),
            through_date: d(2024, 1, 1),
            override_annual_rate: None,
        });
        assert!(r.required);
        assert_eq!(r.applied_annual_rate, dec!(0.01));
    }

    #[test]
    fn negative_window_returns_zero_no_panic() {
        let r = accrue(&AccrualInput {
            state: "MN".into(),
            deposit: dec!(2000),
            deposit_date: d(2024, 6, 1),
            through_date: d(2024, 1, 1),
            override_annual_rate: None,
        });
        assert_eq!(r.days_held, 0);
        assert_eq!(r.interest_owed, Decimal::ZERO);
    }

    #[test]
    fn rule_for_returns_citation_for_known_states() {
        let r = rule_for("CT").unwrap();
        assert!(r.required);
        assert!(r.citation.statute.contains("47a-21"));
        let r = rule_for("MD").unwrap();
        assert!(r.citation.statute.contains("8-203"));
    }

    #[test]
    fn rule_for_unknown_state_returns_none() {
        assert!(rule_for("XX").is_none());
    }
}
