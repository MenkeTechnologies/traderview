//! State late-fee caps + grace periods for residential rentals.
//!
//! Most US states have NO statutory cap on residential late fees, but a
//! growing minority do. The ones that cap either (a) limit to a
//! percentage of monthly rent, (b) impose a dollar maximum, (c) require
//! a grace period before any fee may be assessed, or (d) some combination.
//! Local ordinances (Chicago, Seattle, NYC) frequently impose stricter
//! caps than state law — caller's responsibility to layer those on top.
//!
//! Numbers in this table are statutory references — the cap *values*
//! change occasionally as legislatures amend, so update the table when
//! you spot a new revision and prefer the published statute over this
//! module for legal advice.
//!
//! Pure data + compute. Caller passes the state code + proposed fee +
//! monthly rent + days past due; we return whether the fee is compliant
//! with state law, the maximum allowed, and the citation so the
//! landlord can show their work.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub statute: &'static str,
    pub source: &'static str,
}

/// How `cap_pct` and `cap_fixed` combine when both are present.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapFormula {
    /// Use whichever of (pct × rent) and `cap_fixed` is LOWER.
    LesserOfPctAndFixed,
    /// Use whichever of (pct × rent) and `cap_fixed` is HIGHER.
    GreaterOfPctAndFixed,
    /// Cap is the percentage only (`cap_fixed` ignored).
    PctOnly,
    /// Cap is the dollar amount only (`cap_pct` ignored).
    FixedOnly,
    /// No specific cap — must be "reasonable" per case law. We return
    /// the published reasonable bound in `cap_pct` (e.g. CA case law
    /// treats ~6% as upper bound) but the result will be flagged as
    /// "reasonableness review" not a bright-line cap.
    ReasonableOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateRule {
    pub state: &'static str,
    pub formula: CapFormula,
    /// Percentage of monthly rent (0..1 scale; 0.05 = 5%).
    pub cap_pct: Decimal,
    /// Hard dollar cap.
    pub cap_fixed: Decimal,
    /// Required grace period (calendar days) before a late fee may be
    /// charged. 0 = no statutory grace.
    pub grace_days: u32,
    pub effective_year: i32,
    pub notes: &'static str,
    pub citation: Citation,
}

fn rules() -> &'static [StateRule] {
    static R: once_cell::sync::Lazy<Vec<StateRule>> = once_cell::sync::Lazy::new(|| {
        let d = |s: &str| Decimal::from_str(s).unwrap();
        vec![
            StateRule {
                state: "CA",
                formula: CapFormula::ReasonableOnly,
                cap_pct: d("0.06"),
                cap_fixed: Decimal::ZERO,
                grace_days: 0,
                effective_year: 2024,
                notes: "Cal. Civil Code §1671(d) — must be a reasonable estimate of damages; case law treats ~6% as upper bound but no bright-line cap. Reasonableness review required.",
                citation: Citation {
                    statute: "Cal. Civil Code §1671",
                    source: "https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?sectionNum=1671.&lawCode=CIV",
                },
            },
            StateRule {
                state: "CO",
                formula: CapFormula::GreaterOfPctAndFixed,
                cap_pct: d("0.05"),
                cap_fixed: d("50"),
                grace_days: 7,
                effective_year: 2024,
                notes: "HB21-1121 + HB23-1099: greater of $50 or 5% of past-due rent; 7-day grace before any fee.",
                citation: Citation {
                    statute: "C.R.S. §38-12-105",
                    source: "https://leg.colorado.gov/sites/default/files/2021a_1121_signed.pdf",
                },
            },
            StateRule {
                state: "CT",
                formula: CapFormula::ReasonableOnly,
                cap_pct: Decimal::ZERO,
                cap_fixed: Decimal::ZERO,
                grace_days: 9,
                effective_year: 2024,
                notes: "9-day grace before fee may be charged; no cap on amount but courts review.",
                citation: Citation {
                    statute: "Conn. Gen. Stat. §47a-15a",
                    source: "https://www.cga.ct.gov/current/pub/chap_830.htm",
                },
            },
            StateRule {
                state: "DC",
                formula: CapFormula::PctOnly,
                cap_pct: d("0.05"),
                cap_fixed: Decimal::ZERO,
                grace_days: 5,
                effective_year: 2024,
                notes: "5% of monthly rent cap; 5-day grace required by §42-3505.31.",
                citation: Citation {
                    statute: "D.C. Code §42-3505.31",
                    source: "https://code.dccouncil.us/dc/council/code/sections/42-3505.31.html",
                },
            },
            StateRule {
                state: "DE",
                formula: CapFormula::PctOnly,
                cap_pct: d("0.05"),
                cap_fixed: Decimal::ZERO,
                grace_days: 5,
                effective_year: 2024,
                notes: "5% of monthly rent cap with 5-day grace.",
                citation: Citation {
                    statute: "25 Del. C. §5501(d)",
                    source: "https://delcode.delaware.gov/title25/c055/sc01/index.html",
                },
            },
            StateRule {
                state: "MA",
                formula: CapFormula::ReasonableOnly,
                cap_pct: Decimal::ZERO,
                cap_fixed: Decimal::ZERO,
                grace_days: 30,
                effective_year: 2024,
                notes: "No late fee permitted until rent is 30 days past due. Effectively delays the fee a month.",
                citation: Citation {
                    statute: "Mass. Gen. Laws ch.186 §15B(1)(c)",
                    source: "https://malegislature.gov/Laws/GeneralLaws/PartII/TitleI/Chapter186/Section15B",
                },
            },
            StateRule {
                state: "MD",
                formula: CapFormula::PctOnly,
                cap_pct: d("0.05"),
                cap_fixed: Decimal::ZERO,
                grace_days: 0,
                effective_year: 2024,
                notes: "5% of monthly rent cap. Lease must specify the fee for it to apply.",
                citation: Citation {
                    statute: "Md. Code Real Prop. §8-208(d)(3)",
                    source: "https://mgaleg.maryland.gov/mgawebsite/Laws/StatuteText?article=gre&section=8-208",
                },
            },
            StateRule {
                state: "ME",
                formula: CapFormula::PctOnly,
                cap_pct: d("0.04"),
                cap_fixed: Decimal::ZERO,
                grace_days: 15,
                effective_year: 2024,
                notes: "4% of monthly rent cap; 15-day grace required.",
                citation: Citation {
                    statute: "14 M.R.S. §6028",
                    source: "http://www.mainelegislature.org/legis/statutes/14/title14sec6028.html",
                },
            },
            StateRule {
                state: "MN",
                formula: CapFormula::PctOnly,
                cap_pct: d("0.08"),
                cap_fixed: Decimal::ZERO,
                grace_days: 0,
                effective_year: 2024,
                notes: "8% of overdue rent; lease must specify the fee.",
                citation: Citation {
                    statute: "Minn. Stat. §504B.177",
                    source: "https://www.revisor.mn.gov/statutes/cite/504B.177",
                },
            },
            StateRule {
                state: "NC",
                formula: CapFormula::GreaterOfPctAndFixed,
                cap_pct: d("0.05"),
                cap_fixed: d("15"),
                grace_days: 5,
                effective_year: 2024,
                notes: "Greater of $15 or 5% of monthly rent; 5-day grace.",
                citation: Citation {
                    statute: "N.C.G.S. §42-46",
                    source: "https://www.ncleg.gov/EnactedLegislation/Statutes/HTML/BySection/Chapter_42/GS_42-46.html",
                },
            },
            StateRule {
                state: "NJ",
                formula: CapFormula::ReasonableOnly,
                cap_pct: Decimal::ZERO,
                cap_fixed: Decimal::ZERO,
                grace_days: 5,
                effective_year: 2024,
                notes: "5-business-day grace required; no statutory cap but courts review.",
                citation: Citation {
                    statute: "N.J. Stat. §2A:42-6.1",
                    source: "https://lis.njleg.state.nj.us/nxt/gateway.dll?f=templates&fn=default.htm",
                },
            },
            StateRule {
                state: "NV",
                formula: CapFormula::PctOnly,
                cap_pct: d("0.05"),
                cap_fixed: Decimal::ZERO,
                grace_days: 0,
                effective_year: 2024,
                notes: "5% of monthly rent cap.",
                citation: Citation {
                    statute: "NRS 118A.210",
                    source: "https://www.leg.state.nv.us/NRS/NRS-118A.html",
                },
            },
            StateRule {
                state: "NY",
                formula: CapFormula::LesserOfPctAndFixed,
                cap_pct: d("0.05"),
                cap_fixed: d("50"),
                grace_days: 5,
                effective_year: 2024,
                notes: "Lesser of $50 or 5% of monthly rent; 5-day grace required by HSTPA.",
                citation: Citation {
                    statute: "N.Y. Real Property Law §238-a(2)",
                    source: "https://www.nysenate.gov/legislation/laws/RPP/238-A",
                },
            },
            StateRule {
                state: "OR",
                formula: CapFormula::ReasonableOnly,
                cap_pct: d("0.05"),
                cap_fixed: Decimal::ZERO,
                grace_days: 4,
                effective_year: 2024,
                notes: "4-day grace; flat or reasonable fee permitted; case law caps around 5%.",
                citation: Citation {
                    statute: "ORS 90.260",
                    source: "https://oregon.public.law/statutes/ors_90.260",
                },
            },
            StateRule {
                state: "TX",
                formula: CapFormula::PctOnly,
                cap_pct: d("0.12"),
                cap_fixed: Decimal::ZERO,
                grace_days: 2,
                effective_year: 2024,
                notes: "Tex. Prop. Code §92.019: 2-day grace, then reasonable fee; SAFE HARBOR if ≤ 12% for 1-4 unit buildings, ≤ 10% for 5+ unit buildings.",
                citation: Citation {
                    statute: "Tex. Prop. Code §92.019",
                    source: "https://statutes.capitol.texas.gov/Docs/PR/htm/PR.92.htm",
                },
            },
            StateRule {
                state: "VA",
                formula: CapFormula::PctOnly,
                cap_pct: d("0.10"),
                cap_fixed: Decimal::ZERO,
                grace_days: 0,
                effective_year: 2024,
                notes: "10% of past-due rent OR 10% of monthly rent cap (whichever lower).",
                citation: Citation {
                    statute: "Va. Code §55.1-1204(I)",
                    source: "https://law.lis.virginia.gov/vacode/55.1-1204/",
                },
            },
            StateRule {
                state: "WA",
                formula: CapFormula::ReasonableOnly,
                cap_pct: Decimal::ZERO,
                cap_fixed: Decimal::ZERO,
                grace_days: 5,
                effective_year: 2024,
                notes: "5-day no-fee period after due date; no statutory cap but reasonableness review applies.",
                citation: Citation {
                    statute: "RCW 59.18.170",
                    source: "https://app.leg.wa.gov/rcw/default.aspx?cite=59.18.170",
                },
            },
        ]
    });
    &R
}

pub fn rule_for(state: &str) -> Option<&'static StateRule> {
    let upper = state.to_uppercase();
    rules()
        .iter()
        .find(|r| r.state.eq_ignore_ascii_case(&upper))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LateFeeCheckInput {
    pub state: String,
    pub monthly_rent: Decimal,
    pub proposed_late_fee: Decimal,
    /// Calendar days past the rent due date when the fee is being assessed.
    pub days_past_due: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LateFeeCheckResult {
    pub state_has_specific_cap: bool,
    pub max_fee_permitted: Decimal,
    /// True when the proposed fee is at or below the cap AND grace days
    /// have passed.
    pub compliant: bool,
    pub grace_days_required: u32,
    pub grace_violation: bool,
    pub reasonableness_review_required: bool,
    pub statute: String,
    pub source: String,
    pub notes: String,
}

fn max_fee_for_rule(rule: &StateRule, monthly_rent: Decimal) -> Decimal {
    match rule.formula {
        CapFormula::PctOnly => (rule.cap_pct * monthly_rent).round_dp(2),
        CapFormula::FixedOnly => rule.cap_fixed,
        CapFormula::LesserOfPctAndFixed => {
            let pct_amount = (rule.cap_pct * monthly_rent).round_dp(2);
            pct_amount.min(rule.cap_fixed)
        }
        CapFormula::GreaterOfPctAndFixed => {
            let pct_amount = (rule.cap_pct * monthly_rent).round_dp(2);
            pct_amount.max(rule.cap_fixed)
        }
        CapFormula::ReasonableOnly => {
            // Surface the case-law upper bound (cap_pct) when present;
            // otherwise zero — caller treats as informational.
            (rule.cap_pct * monthly_rent).round_dp(2)
        }
    }
}

pub fn check(input: &LateFeeCheckInput) -> LateFeeCheckResult {
    if let Some(rule) = rule_for(&input.state) {
        let max = max_fee_for_rule(rule, input.monthly_rent);
        let reasonableness = matches!(rule.formula, CapFormula::ReasonableOnly);
        let grace_violation = input.days_past_due < rule.grace_days;
        let amount_ok = if reasonableness {
            // No bright-line; flag for human review but allow the
            // posted case-law bound as the "permitted" figure.
            true
        } else {
            input.proposed_late_fee <= max
        };
        let compliant = amount_ok && !grace_violation;
        LateFeeCheckResult {
            state_has_specific_cap: !reasonableness,
            max_fee_permitted: max,
            compliant,
            grace_days_required: rule.grace_days,
            grace_violation,
            reasonableness_review_required: reasonableness,
            statute: rule.citation.statute.into(),
            source: rule.citation.source.into(),
            notes: rule.notes.into(),
        }
    } else {
        // No statutory cap on record — most states. Still inform the
        // user that case-law reasonableness review applies.
        LateFeeCheckResult {
            state_has_specific_cap: false,
            max_fee_permitted: Decimal::ZERO,
            compliant: true,
            grace_days_required: 0,
            grace_violation: false,
            reasonableness_review_required: true,
            statute: String::new(),
            source: String::new(),
            notes: format!(
                "no statutory cap on file for {} — courts apply reasonableness review",
                input.state.to_uppercase()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn ny_lesser_of_50_or_5pct_uses_50_at_2000_rent() {
        // Rent $2000 × 5% = $100. Lesser of $100 and $50 = $50.
        let r = check(&LateFeeCheckInput {
            state: "NY".into(),
            monthly_rent: dec!(2000),
            proposed_late_fee: dec!(50),
            days_past_due: 6,
        });
        assert_eq!(r.max_fee_permitted, dec!(50));
        assert!(r.compliant);
    }

    #[test]
    fn ny_proposed_above_50_not_compliant() {
        let r = check(&LateFeeCheckInput {
            state: "NY".into(),
            monthly_rent: dec!(2000),
            proposed_late_fee: dec!(75),
            days_past_due: 6,
        });
        assert!(!r.compliant);
    }

    #[test]
    fn ny_within_5_day_grace_violation() {
        let r = check(&LateFeeCheckInput {
            state: "NY".into(),
            monthly_rent: dec!(2000),
            proposed_late_fee: dec!(50),
            days_past_due: 3, // < 5 grace
        });
        assert!(!r.compliant);
        assert!(r.grace_violation);
    }

    #[test]
    fn nc_greater_of_15_or_5pct_uses_5pct_at_1000_rent() {
        // Rent $1000 × 5% = $50. Greater of $50 and $15 = $50.
        let r = check(&LateFeeCheckInput {
            state: "NC".into(),
            monthly_rent: dec!(1000),
            proposed_late_fee: dec!(50),
            days_past_due: 6,
        });
        assert_eq!(r.max_fee_permitted, dec!(50));
        assert!(r.compliant);
    }

    #[test]
    fn nc_greater_of_15_or_5pct_uses_15_at_low_rent() {
        // Rent $200 × 5% = $10. Greater of $10 and $15 = $15.
        let r = check(&LateFeeCheckInput {
            state: "NC".into(),
            monthly_rent: dec!(200),
            proposed_late_fee: dec!(15),
            days_past_due: 6,
        });
        assert_eq!(r.max_fee_permitted, dec!(15));
        assert!(r.compliant);
    }

    #[test]
    fn co_50_floor_at_low_rent() {
        // CO: greater of $50 or 5%. At $500 rent: 5% = $25, but $50 floor wins.
        let r = check(&LateFeeCheckInput {
            state: "CO".into(),
            monthly_rent: dec!(500),
            proposed_late_fee: dec!(50),
            days_past_due: 8,
        });
        assert_eq!(r.max_fee_permitted, dec!(50));
        assert!(r.compliant);
    }

    #[test]
    fn co_5pct_at_high_rent() {
        // CO: $5000 rent × 5% = $250 > $50, so cap is $250.
        let r = check(&LateFeeCheckInput {
            state: "CO".into(),
            monthly_rent: dec!(5000),
            proposed_late_fee: dec!(200),
            days_past_due: 8,
        });
        assert_eq!(r.max_fee_permitted, dec!(250));
        assert!(r.compliant);
    }

    #[test]
    fn ma_30_day_grace_blocks_early_fee() {
        let r = check(&LateFeeCheckInput {
            state: "MA".into(),
            monthly_rent: dec!(2500),
            proposed_late_fee: dec!(50),
            days_past_due: 10,
        });
        assert!(r.grace_violation);
        assert!(!r.compliant);
    }

    #[test]
    fn mn_8pct_cap() {
        // $1500 rent × 8% = $120.
        let r = check(&LateFeeCheckInput {
            state: "MN".into(),
            monthly_rent: dec!(1500),
            proposed_late_fee: dec!(120),
            days_past_due: 1,
        });
        assert_eq!(r.max_fee_permitted, dec!(120));
        assert!(r.compliant);
    }

    #[test]
    fn tx_12pct_safe_harbor() {
        // $2000 × 12% = $240. Compliant at safe harbor.
        let r = check(&LateFeeCheckInput {
            state: "TX".into(),
            monthly_rent: dec!(2000),
            proposed_late_fee: dec!(240),
            days_past_due: 3,
        });
        assert_eq!(r.max_fee_permitted, dec!(240));
        assert!(r.compliant);
    }

    #[test]
    fn ca_reasonableness_review_flagged() {
        let r = check(&LateFeeCheckInput {
            state: "CA".into(),
            monthly_rent: dec!(3000),
            proposed_late_fee: dec!(150),
            days_past_due: 5,
        });
        assert!(r.reasonableness_review_required);
        assert!(!r.state_has_specific_cap);
        // Reasonableness still returns compliant=true; caller decides.
        assert!(r.compliant);
    }

    #[test]
    fn unknown_state_no_cap_but_reasonableness_noted() {
        let r = check(&LateFeeCheckInput {
            state: "AL".into(), // no rule on file
            monthly_rent: dec!(1000),
            proposed_late_fee: dec!(100),
            days_past_due: 1,
        });
        assert!(!r.state_has_specific_cap);
        assert!(r.reasonableness_review_required);
        assert!(r.compliant);
        assert!(r.notes.contains("no statutory cap"));
    }

    #[test]
    fn case_insensitive_state_lookup() {
        let r = check(&LateFeeCheckInput {
            state: "ny".into(),
            monthly_rent: dec!(2000),
            proposed_late_fee: dec!(50),
            days_past_due: 6,
        });
        assert!(r.state_has_specific_cap);
        assert_eq!(r.max_fee_permitted, dec!(50));
    }

    #[test]
    fn rule_for_returns_citation_for_known_states() {
        let r = rule_for("MD").unwrap();
        assert!(r.citation.statute.contains("8-208"));
        let r = rule_for("NY").unwrap();
        assert!(r.citation.statute.contains("238-a"));
        let r = rule_for("TX").unwrap();
        assert!(r.citation.statute.contains("92.019"));
    }

    #[test]
    fn grace_satisfaction_exactly_at_grace_day_compliant() {
        // NY: 5-day grace. days_past_due = 5 should pass (>=).
        let r = check(&LateFeeCheckInput {
            state: "NY".into(),
            monthly_rent: dec!(2000),
            proposed_late_fee: dec!(50),
            days_past_due: 5,
        });
        assert!(!r.grace_violation);
        assert!(r.compliant);
    }

    #[test]
    fn ny_at_3pct_under_lesser_cap() {
        // $1000 × 3% = $30. Under the $50 cap AND under the 5% cap.
        let r = check(&LateFeeCheckInput {
            state: "NY".into(),
            monthly_rent: dec!(1000),
            proposed_late_fee: dec!(30),
            days_past_due: 6,
        });
        assert!(r.compliant);
        // Max = min(50, 50) = 50 at this rent.
        assert_eq!(r.max_fee_permitted, dec!(50));
    }

    #[test]
    fn va_10pct_cap_at_2000_rent() {
        let r = check(&LateFeeCheckInput {
            state: "VA".into(),
            monthly_rent: dec!(2000),
            proposed_late_fee: dec!(200),
            days_past_due: 1,
        });
        assert_eq!(r.max_fee_permitted, dec!(200));
        assert!(r.compliant);
    }

    #[test]
    fn me_4pct_cap_with_15_day_grace() {
        let r = check(&LateFeeCheckInput {
            state: "ME".into(),
            monthly_rent: dec!(1000),
            proposed_late_fee: dec!(40),
            days_past_due: 16,
        });
        assert_eq!(r.max_fee_permitted, dec!(40));
        assert!(!r.grace_violation);
        assert!(r.compliant);

        // Same fee, day 14 — grace violated.
        let r2 = check(&LateFeeCheckInput {
            state: "ME".into(),
            monthly_rent: dec!(1000),
            proposed_late_fee: dec!(40),
            days_past_due: 14,
        });
        assert!(r2.grace_violation);
    }
}
