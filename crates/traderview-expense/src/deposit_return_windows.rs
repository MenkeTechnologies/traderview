//! State security deposit RETURN window table.
//!
//! Sibling to iter 3's `deposit_interest`, iter 17's `late_fee_caps`,
//! iter 19's `eviction_notices`, and iter 25's `contractor_1099` —
//! state landlord-tenant operations data. Every state has a
//! statutory window for returning the security deposit (often plus
//! an itemized deduction statement) after the tenancy ends. Missing
//! the deadline frequently triggers:
//!
//!   * **Automatic forfeiture** of the right to withhold any portion
//!     of the deposit — landlord must return the full amount.
//!   * **Bad-faith multiplier penalty** — many states impose 2×, 3×
//!     (or in MA's case, triple damages plus attorney's fees) when
//!     the withholding is found to be in bad faith.
//!   * **Tenant's attorney fees** in some jurisdictions.
//!
//! The table here reflects published statutes as of the cited
//! year. State legislatures amend periodically; update the table
//! when you spot a new revision, and prefer the published statute
//! over this module for legal advice.
//!
//! Pure data + compute. Caller passes the state code + tenancy-end
//! date + return date + deposit amount + bad-faith assertion; we
//! return whether compliant, the maximum penalty exposure, the
//! statute citation, and the calculated days late.

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
pub struct StateReturnRule {
    pub state: &'static str,
    /// Statutory return window in calendar days after tenancy end.
    pub return_window_days: u32,
    /// True when statute counts BUSINESS days (excluding weekends and
    /// state holidays). Caller's responsibility to filter actual
    /// calendar days received against this flag.
    pub business_days_basis: bool,
    /// True when the statute requires an itemized statement of
    /// deductions within the window (most states do).
    pub itemized_statement_required: bool,
    /// Penalty multiplier on damages when bad-faith withholding is
    /// established. 1.0 = no multiplier (just actual damages).
    pub bad_faith_damages_multiplier: Decimal,
    /// True when statute also awards reasonable attorney's fees to
    /// the prevailing tenant.
    pub attorney_fees_to_prevailing_tenant: bool,
    pub effective_year: i32,
    pub notes: &'static str,
    pub citation: Citation,
}

fn d(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

fn rules() -> &'static [StateReturnRule] {
    static R: once_cell::sync::Lazy<Vec<StateReturnRule>> = once_cell::sync::Lazy::new(|| {
        vec![
                StateReturnRule {
                    state: "AZ",
                    return_window_days: 14,
                    business_days_basis: true,
                    itemized_statement_required: true,
                    bad_faith_damages_multiplier: d("2.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "Arizona Residential Landlord-Tenant Act: 14 business days after demand. Wrongful withholding = 2× plus attorney's fees.",
                    citation: Citation {
                        statute: "A.R.S. §33-1321(D)",
                        source: "https://www.azleg.gov/ars/33/01321.htm",
                    },
                },
                StateReturnRule {
                    state: "CA",
                    return_window_days: 21,
                    business_days_basis: false,
                    itemized_statement_required: true,
                    bad_faith_damages_multiplier: d("2.0"),
                    attorney_fees_to_prevailing_tenant: false,
                    effective_year: 2024,
                    notes: "Cal. Civ. Code §1950.5(g): 21 calendar days. Bad-faith retention = actual damages + up to 2× statutory penalty. Itemized statement + receipts for any deduction over $125.",
                    citation: Citation {
                        statute: "Cal. Civ. Code §1950.5(g)",
                        source: "https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?sectionNum=1950.5.&lawCode=CIV",
                    },
                },
                StateReturnRule {
                    state: "CO",
                    return_window_days: 30,
                    business_days_basis: false,
                    itemized_statement_required: true,
                    bad_faith_damages_multiplier: d("3.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "C.R.S. §38-12-103: 30 days standard, 60 days if lease specifies. Wrongful withholding = treble damages + attorney's fees.",
                    citation: Citation {
                        statute: "C.R.S. §38-12-103",
                        source: "https://leg.colorado.gov/sites/default/files/2023a_1095_signed.pdf",
                    },
                },
                StateReturnRule {
                    state: "CT",
                    return_window_days: 30,
                    business_days_basis: false,
                    itemized_statement_required: true,
                    bad_faith_damages_multiplier: d("2.0"),
                    attorney_fees_to_prevailing_tenant: false,
                    effective_year: 2024,
                    notes: "Conn. Gen. Stat. §47a-21(d): 30 days after tenancy end OR 15 days after receiving tenant's forwarding address, whichever is later. 2× penalty on wrongful withholding.",
                    citation: Citation {
                        statute: "Conn. Gen. Stat. §47a-21(d)",
                        source: "https://www.cga.ct.gov/current/pub/chap_830.htm",
                    },
                },
                StateReturnRule {
                    state: "DC",
                    return_window_days: 45,
                    business_days_basis: false,
                    itemized_statement_required: true,
                    bad_faith_damages_multiplier: d("3.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "D.C. Mun. Reg. 14-309: 45 days, itemized statement required. Bad-faith withholding = treble damages.",
                    citation: Citation {
                        statute: "14 D.C.M.R. §309",
                        source: "https://os.dc.gov/page/dc-municipal-regulations-dcmr",
                    },
                },
                StateReturnRule {
                    state: "FL",
                    return_window_days: 15,
                    business_days_basis: false,
                    itemized_statement_required: true,
                    bad_faith_damages_multiplier: d("1.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "Fla. Stat. §83.49(3): 15 days if no deductions claimed; 30 days if landlord makes a claim (with written notice within 30). Tenant has 15 days to object.",
                    citation: Citation {
                        statute: "Fla. Stat. §83.49(3)",
                        source: "https://www.flsenate.gov/Laws/Statutes/2024/0083.49",
                    },
                },
                StateReturnRule {
                    state: "GA",
                    return_window_days: 30,
                    business_days_basis: false,
                    itemized_statement_required: true,
                    bad_faith_damages_multiplier: d("3.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "O.C.G.A. §44-7-34: 30 days. Bad-faith withholding = 3× actual damages + attorney's fees per O.C.G.A. §44-7-35.",
                    citation: Citation {
                        statute: "O.C.G.A. §44-7-34",
                        source: "https://law.justia.com/codes/georgia/2022/title-44/chapter-7/article-2/section-44-7-34/",
                    },
                },
                StateReturnRule {
                    state: "IL",
                    return_window_days: 30,
                    business_days_basis: false,
                    itemized_statement_required: true,
                    bad_faith_damages_multiplier: d("2.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "765 ILCS 710: 30 days, 45 days for buildings of 25+ units. Wrongful withholding = 2× damages + reasonable attorney's fees. Chicago RLTO adds local protections.",
                    citation: Citation {
                        statute: "765 ILCS 710/1",
                        source: "https://www.ilga.gov/legislation/ilcs/ilcs3.asp?ActID=2204",
                    },
                },
                StateReturnRule {
                    state: "MA",
                    return_window_days: 30,
                    business_days_basis: false,
                    itemized_statement_required: true,
                    bad_faith_damages_multiplier: d("3.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "G.L. c. 186 §15B(7): 30 days. Among the strictest in the country: triple damages + attorney's fees + 5% interest on bad-faith retention.",
                    citation: Citation {
                        statute: "Mass. Gen. Laws ch.186 §15B(7)",
                        source: "https://malegislature.gov/Laws/GeneralLaws/PartII/TitleI/Chapter186/Section15B",
                    },
                },
                StateReturnRule {
                    state: "MD",
                    return_window_days: 45,
                    business_days_basis: false,
                    itemized_statement_required: true,
                    bad_faith_damages_multiplier: d("3.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "Md. Code Real Prop. §8-203(e): 45 days + itemized statement within 45 days. Withholding without good cause = 3× damages + attorney's fees.",
                    citation: Citation {
                        statute: "Md. Code Real Prop. §8-203(e)",
                        source: "https://mgaleg.maryland.gov/mgawebsite/Laws/StatuteText?article=gre&section=8-203",
                    },
                },
                StateReturnRule {
                    state: "MI",
                    return_window_days: 30,
                    business_days_basis: false,
                    itemized_statement_required: true,
                    bad_faith_damages_multiplier: d("2.0"),
                    attorney_fees_to_prevailing_tenant: false,
                    effective_year: 2024,
                    notes: "MCL 554.609: 30 days. Itemized statement of damages required. Wrongful retention = 2× actual damages.",
                    citation: Citation {
                        statute: "MCL 554.609",
                        source: "https://www.legislature.mi.gov/(S(0))/mileg.aspx?page=getobject&objectname=mcl-554-609",
                    },
                },
                StateReturnRule {
                    state: "MN",
                    return_window_days: 21,
                    business_days_basis: false,
                    itemized_statement_required: true,
                    bad_faith_damages_multiplier: d("2.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "Minn. Stat. §504B.178(3): 21 days. Bad-faith retention = damages × 2 + reasonable attorney's fees + $500 statutory penalty.",
                    citation: Citation {
                        statute: "Minn. Stat. §504B.178",
                        source: "https://www.revisor.mn.gov/statutes/cite/504B.178",
                    },
                },
                StateReturnRule {
                    state: "NJ",
                    return_window_days: 30,
                    business_days_basis: false,
                    itemized_statement_required: true,
                    bad_faith_damages_multiplier: d("2.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "NJSA §46:8-21.1: 30 days, 5 days for natural-disaster terminations. Wrongful withholding = 2× + court costs + attorney's fees.",
                    citation: Citation {
                        statute: "N.J. Stat. §46:8-21.1",
                        source: "https://lis.njleg.state.nj.us/nxt/gateway.dll?f=templates&fn=default.htm",
                    },
                },
                StateReturnRule {
                    state: "NV",
                    return_window_days: 30,
                    business_days_basis: false,
                    itemized_statement_required: true,
                    bad_faith_damages_multiplier: d("2.0"),
                    attorney_fees_to_prevailing_tenant: false,
                    effective_year: 2024,
                    notes: "NRS 118A.242: 30 days. Wrongful refusal = 2× damages.",
                    citation: Citation {
                        statute: "NRS 118A.242",
                        source: "https://www.leg.state.nv.us/NRS/NRS-118A.html",
                    },
                },
                StateReturnRule {
                    state: "NY",
                    return_window_days: 14,
                    business_days_basis: false,
                    itemized_statement_required: true,
                    bad_faith_damages_multiplier: d("2.0"),
                    attorney_fees_to_prevailing_tenant: false,
                    effective_year: 2024,
                    notes: "NY GOL §7-108 (HSTPA 2019): 14 days + itemized statement. Failure to provide statement = forfeiture of right to withhold. 2× damages on bad-faith.",
                    citation: Citation {
                        statute: "N.Y. Gen. Oblig. Law §7-108",
                        source: "https://www.nysenate.gov/legislation/laws/GOB/7-108",
                    },
                },
                StateReturnRule {
                    state: "NC",
                    return_window_days: 30,
                    business_days_basis: false,
                    itemized_statement_required: true,
                    bad_faith_damages_multiplier: d("1.0"),
                    attorney_fees_to_prevailing_tenant: false,
                    effective_year: 2024,
                    notes: "N.C.G.S. §42-52: 30 days, 60 days if final accounting requires more time + interim notice.",
                    citation: Citation {
                        statute: "N.C.G.S. §42-52",
                        source: "https://www.ncleg.gov/EnactedLegislation/Statutes/HTML/BySection/Chapter_42/GS_42-52.html",
                    },
                },
                StateReturnRule {
                    state: "OH",
                    return_window_days: 30,
                    business_days_basis: false,
                    itemized_statement_required: true,
                    bad_faith_damages_multiplier: d("2.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "R.C. §5321.16: 30 days. Wrongful withholding = 2× damages + reasonable attorney's fees.",
                    citation: Citation {
                        statute: "Ohio Rev. Code §5321.16",
                        source: "https://codes.ohio.gov/ohio-revised-code/section-5321.16",
                    },
                },
                StateReturnRule {
                    state: "OR",
                    return_window_days: 31,
                    business_days_basis: false,
                    itemized_statement_required: true,
                    bad_faith_damages_multiplier: d("2.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "ORS 90.300(13): 31 days. Wrongful withholding = 2× amount wrongfully withheld + attorney's fees.",
                    citation: Citation {
                        statute: "ORS 90.300",
                        source: "https://oregon.public.law/statutes/ors_90.300",
                    },
                },
                StateReturnRule {
                    state: "PA",
                    return_window_days: 30,
                    business_days_basis: false,
                    itemized_statement_required: true,
                    bad_faith_damages_multiplier: d("2.0"),
                    attorney_fees_to_prevailing_tenant: false,
                    effective_year: 2024,
                    notes: "68 P.S. §250.512: 30 days. Wrongful withholding = 2× damages.",
                    citation: Citation {
                        statute: "68 Pa. Cons. Stat. §250.512",
                        source: "https://www.legis.state.pa.us/cfdocs/legis/LI/uconsCheck.cfm?txtType=HTM&yr=1951&sessInd=0&smthLwInd=0&act=20",
                    },
                },
                StateReturnRule {
                    state: "TX",
                    return_window_days: 30,
                    business_days_basis: false,
                    itemized_statement_required: true,
                    bad_faith_damages_multiplier: d("3.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "Tex. Prop. Code §92.103: 30 days. Bad-faith withholding = $100 + 3× wrongful amount + attorney's fees per §92.109.",
                    citation: Citation {
                        statute: "Tex. Prop. Code §92.103",
                        source: "https://statutes.capitol.texas.gov/Docs/PR/htm/PR.92.htm",
                    },
                },
                StateReturnRule {
                    state: "VA",
                    return_window_days: 45,
                    business_days_basis: false,
                    itemized_statement_required: true,
                    bad_faith_damages_multiplier: d("1.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "Va. Code §55.1-1226: 45 days + itemized statement.",
                    citation: Citation {
                        statute: "Va. Code §55.1-1226",
                        source: "https://law.lis.virginia.gov/vacode/title55.1/chapter12/section55.1-1226/",
                    },
                },
                StateReturnRule {
                    state: "WA",
                    return_window_days: 30,
                    business_days_basis: false,
                    itemized_statement_required: true,
                    bad_faith_damages_multiplier: d("2.0"),
                    attorney_fees_to_prevailing_tenant: true,
                    effective_year: 2024,
                    notes: "RCW 59.18.280: 30 days. Wrongful withholding = 2× damages + court costs + reasonable attorney's fees.",
                    citation: Citation {
                        statute: "RCW 59.18.280",
                        source: "https://app.leg.wa.gov/RCW/default.aspx?cite=59.18.280",
                    },
                },
            ]
    });
    &R
}

pub fn rule_for(state: &str) -> Option<&'static StateReturnRule> {
    let upper = state.to_uppercase();
    rules()
        .iter()
        .find(|r| r.state.eq_ignore_ascii_case(&upper))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositReturnCheckInput {
    pub state: String,
    pub tenancy_end_date: NaiveDate,
    pub return_date: NaiveDate,
    pub deposit_amount: Decimal,
    pub deductions_claimed: Decimal,
    /// True when the landlord's withholding is alleged to be in bad
    /// faith (no good-faith dispute over damage). Drives the
    /// multiplier penalty calculation.
    pub bad_faith_alleged: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositReturnCheckResult {
    pub state_recognized: bool,
    pub required_return_window_days: u32,
    pub days_elapsed: i64,
    pub compliant: bool,
    pub days_late: i64,
    /// Maximum penalty exposure under bad-faith conditions = wrongful
    /// withholding × multiplier. Zero when compliant or no bad faith.
    pub max_penalty_exposure: Decimal,
    pub attorney_fees_at_risk: bool,
    pub statute: String,
    pub source: String,
    pub notes: String,
}

pub fn check(input: &DepositReturnCheckInput) -> DepositReturnCheckResult {
    let rule = match rule_for(&input.state) {
        Some(r) => r,
        None => {
            return DepositReturnCheckResult {
                state_recognized: false,
                required_return_window_days: 0,
                days_elapsed: 0,
                compliant: true,
                days_late: 0,
                max_penalty_exposure: Decimal::ZERO,
                attorney_fees_at_risk: false,
                statute: String::new(),
                source: String::new(),
                notes: format!(
                    "no deposit-return statute on file for {} — consult state landlord-tenant code directly",
                    input.state.to_uppercase()
                ),
            };
        }
    };

    let days_elapsed = (input.return_date - input.tenancy_end_date).num_days();
    let required = rule.return_window_days as i64;
    let compliant = days_elapsed <= required;
    let days_late = (days_elapsed - required).max(0);

    let wrongful_withholding = if input.bad_faith_alleged {
        input.deductions_claimed.max(Decimal::ZERO)
    } else {
        Decimal::ZERO
    };
    let max_penalty_exposure = wrongful_withholding * rule.bad_faith_damages_multiplier;

    DepositReturnCheckResult {
        state_recognized: true,
        required_return_window_days: rule.return_window_days,
        days_elapsed,
        compliant,
        days_late,
        max_penalty_exposure,
        attorney_fees_at_risk: input.bad_faith_alleged && rule.attorney_fees_to_prevailing_tenant,
        statute: rule.citation.statute.into(),
        source: rule.citation.source.into(),
        notes: rule.notes.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn ca_21_day_window_compliant_at_day_21() {
        let r = check(&DepositReturnCheckInput {
            state: "CA".into(),
            tenancy_end_date: date(2024, 6, 1),
            return_date: date(2024, 6, 22), // 21 days later
            deposit_amount: dec!(2000),
            deductions_claimed: Decimal::ZERO,
            bad_faith_alleged: false,
        });
        assert_eq!(r.required_return_window_days, 21);
        assert_eq!(r.days_elapsed, 21);
        assert!(r.compliant);
        assert_eq!(r.days_late, 0);
    }

    #[test]
    fn ca_day_22_one_day_late() {
        let r = check(&DepositReturnCheckInput {
            state: "CA".into(),
            tenancy_end_date: date(2024, 6, 1),
            return_date: date(2024, 6, 23),
            deposit_amount: dec!(2000),
            deductions_claimed: Decimal::ZERO,
            bad_faith_alleged: false,
        });
        assert!(!r.compliant);
        assert_eq!(r.days_late, 1);
    }

    #[test]
    fn tx_30_day_window_with_bad_faith_3x_multiplier() {
        // Texas: 30 days + 3× multiplier on bad-faith wrongful withholding.
        // $1500 wrongfully withheld → $4500 max penalty exposure.
        let r = check(&DepositReturnCheckInput {
            state: "TX".into(),
            tenancy_end_date: date(2024, 6, 1),
            return_date: date(2024, 8, 1), // 61 days — way late
            deposit_amount: dec!(2000),
            deductions_claimed: dec!(1500),
            bad_faith_alleged: true,
        });
        assert!(!r.compliant);
        assert_eq!(r.days_late, 31);
        assert_eq!(r.max_penalty_exposure, dec!(4500));
        assert!(r.attorney_fees_at_risk);
    }

    #[test]
    fn ma_30_day_window_with_3x_multiplier_strictest_in_country() {
        let r = check(&DepositReturnCheckInput {
            state: "MA".into(),
            tenancy_end_date: date(2024, 6, 1),
            return_date: date(2024, 7, 15),
            deposit_amount: dec!(3000),
            deductions_claimed: dec!(2000),
            bad_faith_alleged: true,
        });
        assert_eq!(r.required_return_window_days, 30);
        assert_eq!(r.max_penalty_exposure, dec!(6000)); // 3× $2k
        assert!(r.attorney_fees_at_risk);
        assert!(r.notes.contains("triple"));
    }

    #[test]
    fn ny_14_day_window_post_hstpa() {
        let r = check(&DepositReturnCheckInput {
            state: "NY".into(),
            tenancy_end_date: date(2024, 6, 1),
            return_date: date(2024, 6, 14), // day 13
            deposit_amount: dec!(2500),
            deductions_claimed: Decimal::ZERO,
            bad_faith_alleged: false,
        });
        assert_eq!(r.required_return_window_days, 14);
        assert!(r.compliant);
    }

    #[test]
    fn ny_day_15_one_day_late_with_hstpa() {
        let r = check(&DepositReturnCheckInput {
            state: "NY".into(),
            tenancy_end_date: date(2024, 6, 1),
            return_date: date(2024, 6, 16),
            deposit_amount: dec!(2500),
            deductions_claimed: Decimal::ZERO,
            bad_faith_alleged: false,
        });
        assert!(!r.compliant);
        assert_eq!(r.days_late, 1);
    }

    #[test]
    fn fl_15_day_window_shortest_for_no_deductions() {
        let r = check(&DepositReturnCheckInput {
            state: "FL".into(),
            tenancy_end_date: date(2024, 6, 1),
            return_date: date(2024, 6, 16),
            deposit_amount: dec!(1500),
            deductions_claimed: Decimal::ZERO,
            bad_faith_alleged: false,
        });
        assert_eq!(r.required_return_window_days, 15);
        assert!(r.compliant);
    }

    #[test]
    fn va_45_day_window_longest_among_modeled() {
        let r = check(&DepositReturnCheckInput {
            state: "VA".into(),
            tenancy_end_date: date(2024, 6, 1),
            return_date: date(2024, 7, 16), // 45 days
            deposit_amount: dec!(2000),
            deductions_claimed: Decimal::ZERO,
            bad_faith_alleged: false,
        });
        assert_eq!(r.required_return_window_days, 45);
        assert!(r.compliant);
    }

    #[test]
    fn no_bad_faith_zero_penalty_exposure_regardless_of_lateness() {
        let r = check(&DepositReturnCheckInput {
            state: "CA".into(),
            tenancy_end_date: date(2024, 6, 1),
            return_date: date(2024, 8, 1), // way late
            deposit_amount: dec!(2000),
            deductions_claimed: dec!(1000),
            bad_faith_alleged: false,
        });
        assert!(!r.compliant);
        assert!(r.days_late > 0);
        assert_eq!(r.max_penalty_exposure, Decimal::ZERO);
    }

    #[test]
    fn unknown_state_returns_not_recognized() {
        let r = check(&DepositReturnCheckInput {
            state: "XX".into(),
            tenancy_end_date: date(2024, 6, 1),
            return_date: date(2024, 6, 30),
            deposit_amount: dec!(2000),
            deductions_claimed: Decimal::ZERO,
            bad_faith_alleged: false,
        });
        assert!(!r.state_recognized);
        assert!(r.notes.contains("no deposit-return statute"));
    }

    #[test]
    fn case_insensitive_state_lookup() {
        let r = check(&DepositReturnCheckInput {
            state: "ca".into(),
            tenancy_end_date: date(2024, 6, 1),
            return_date: date(2024, 6, 22),
            deposit_amount: dec!(2000),
            deductions_claimed: Decimal::ZERO,
            bad_faith_alleged: false,
        });
        assert!(r.state_recognized);
        assert_eq!(r.required_return_window_days, 21);
    }

    #[test]
    fn co_3x_multiplier_with_attorney_fees() {
        let r = check(&DepositReturnCheckInput {
            state: "CO".into(),
            tenancy_end_date: date(2024, 6, 1),
            return_date: date(2024, 8, 1),
            deposit_amount: dec!(2500),
            deductions_claimed: dec!(1000),
            bad_faith_alleged: true,
        });
        assert_eq!(r.max_penalty_exposure, dec!(3000)); // 3× $1k
        assert!(r.attorney_fees_at_risk);
    }

    #[test]
    fn fl_1x_multiplier_no_penalty_escalation() {
        // FL doesn't statutorily multiply; max exposure = wrongful amount.
        let r = check(&DepositReturnCheckInput {
            state: "FL".into(),
            tenancy_end_date: date(2024, 6, 1),
            return_date: date(2024, 7, 16),
            deposit_amount: dec!(1500),
            deductions_claimed: dec!(800),
            bad_faith_alleged: true,
        });
        assert_eq!(r.max_penalty_exposure, dec!(800)); // 1× wrongful
    }

    #[test]
    fn rule_for_returns_citation_for_known_states() {
        let r = rule_for("MA").unwrap();
        assert!(r.citation.statute.contains("15B"));
        let r = rule_for("TX").unwrap();
        assert!(r.citation.statute.contains("92.103"));
        let r = rule_for("CA").unwrap();
        assert!(r.citation.statute.contains("1950.5"));
    }

    #[test]
    fn return_before_tenancy_end_negative_days_compliant() {
        // Pathological — return before tenancy end. Just verify no panic.
        let r = check(&DepositReturnCheckInput {
            state: "CA".into(),
            tenancy_end_date: date(2024, 6, 15),
            return_date: date(2024, 6, 1), // before
            deposit_amount: dec!(2000),
            deductions_claimed: Decimal::ZERO,
            bad_faith_alleged: false,
        });
        assert!(r.compliant);
        assert!(r.days_elapsed < 0);
        assert_eq!(r.days_late, 0);
    }

    #[test]
    fn states_with_attorney_fees_flag_correctly() {
        for state in ["CO", "GA", "MA", "MD", "MN", "NJ", "OH", "OR", "TX", "WA"] {
            let r = rule_for(state).unwrap();
            assert!(
                r.attorney_fees_to_prevailing_tenant,
                "{} should award attorney fees",
                state
            );
        }
    }

    #[test]
    fn states_without_attorney_fees_flag_correctly() {
        for state in ["CA", "CT", "MI", "NV", "NY", "NC", "PA"] {
            let r = rule_for(state).unwrap();
            assert!(
                !r.attorney_fees_to_prevailing_tenant,
                "{} should NOT award attorney fees by statute",
                state
            );
        }
    }

    #[test]
    fn good_faith_no_attorney_fees_at_risk_even_in_attorney_fee_state() {
        let r = check(&DepositReturnCheckInput {
            state: "MA".into(),
            tenancy_end_date: date(2024, 6, 1),
            return_date: date(2024, 7, 15),
            deposit_amount: dec!(2000),
            deductions_claimed: dec!(500),
            bad_faith_alleged: false,
        });
        // Even though MA statutorily awards fees, no bad faith alleged
        // → no attorney_fees_at_risk flag.
        assert!(!r.attorney_fees_at_risk);
    }
}
