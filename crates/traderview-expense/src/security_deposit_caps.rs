//! State security deposit maximum amount table — ninth state-data
//! module after `deposit_interest`, `late_fee_caps`, `eviction_notices`,
//! `contractor_1099`, `deposit_return_windows`, `lease_disclosures`,
//! `rent_control`, and `habitability_remedies`.
//!
//! Most states cap how much a landlord can require as a security
//! deposit. Common formula: a multiple of monthly rent (1× to 3×).
//! Some states have carve-outs:
//!
//!   * **CA AB12 (2023, effective July 2024)** — Cal. Civ. Code
//!     §1950.5(c): cap reduced from 2 months (unfurnished) / 3 months
//!     (furnished) to a flat **1 month**. Small landlord exception:
//!     2 months for landlords who own ≤ 2 residential rental units
//!     totaling ≤ 4 dwelling units AND are natural persons or LLCs
//!     owned by natural persons.
//!
//!   * **Furnished carve-outs** — KS, OR, NJ all permit a higher
//!     cap for furnished units.
//!
//!   * **Elderly / disabled carve-outs** — some states (DE)
//!     waive the cap on long-term tenancies.
//!
//! Pure data + compute. Caller passes the state + monthly rent +
//! proposed deposit + property facts; we return whether compliant +
//! the maximum permitted under statute.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub statute: &'static str,
    pub source: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositCapRule {
    pub state: &'static str,
    /// Months-of-rent cap. 0 = no statutory cap.
    pub max_months_rent: Decimal,
    /// Increased cap when unit is furnished. Same as base when no
    /// furnished carve-out.
    pub max_months_rent_furnished: Decimal,
    /// Small-landlord cap (CA AB12 exception for landlords with ≤ 2
    /// residential rental units). Same as base when no carve-out.
    pub max_months_rent_small_landlord: Decimal,
    /// True when state preempts local rules; false when local
    /// ordinances may impose stricter caps.
    pub state_preempts_local: bool,
    pub effective_year: i32,
    pub notes: &'static str,
    pub citation: Citation,
}

fn d(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

fn rules() -> &'static [DepositCapRule] {
    static R: once_cell::sync::Lazy<Vec<DepositCapRule>> =
        once_cell::sync::Lazy::new(|| {
            vec![
                DepositCapRule {
                    state: "CA",
                    max_months_rent: d("1"),
                    max_months_rent_furnished: d("1"),
                    max_months_rent_small_landlord: d("2"),
                    state_preempts_local: false,
                    effective_year: 2024,
                    notes: "AB12 (eff. July 1, 2024): max 1 month for most landlords; 2 months for natural-person or natural-person-owned-LLC landlords with ≤ 2 rental properties totaling ≤ 4 dwelling units.",
                    citation: Citation {
                        statute: "Cal. Civ. Code §1950.5(c)",
                        source: "https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?sectionNum=1950.5.&lawCode=CIV",
                    },
                },
                DepositCapRule {
                    state: "NY",
                    max_months_rent: d("1"),
                    max_months_rent_furnished: d("1"),
                    max_months_rent_small_landlord: d("1"),
                    state_preempts_local: true,
                    effective_year: 2024,
                    notes: "HSTPA 2019: max 1 month state-wide; preempts higher local rules.",
                    citation: Citation {
                        statute: "N.Y. Real Property Law §7-108",
                        source: "https://www.nysenate.gov/legislation/laws/RPP/7-108",
                    },
                },
                DepositCapRule {
                    state: "MA",
                    max_months_rent: d("1"),
                    max_months_rent_furnished: d("1"),
                    max_months_rent_small_landlord: d("1"),
                    state_preempts_local: true,
                    effective_year: 2024,
                    notes: "G.L. c. 186 §15B: max 1 month security; landlord may separately collect first + last month rent + key cost.",
                    citation: Citation {
                        statute: "Mass. Gen. Laws ch.186 §15B",
                        source: "https://malegislature.gov/Laws/GeneralLaws/PartII/TitleI/Chapter186/Section15B",
                    },
                },
                DepositCapRule {
                    state: "NJ",
                    max_months_rent: d("1.5"),
                    max_months_rent_furnished: d("1.5"),
                    max_months_rent_small_landlord: d("1.5"),
                    state_preempts_local: false,
                    effective_year: 2024,
                    notes: "N.J.S.A. 46:8-21.2: max 1.5 months. Annual increase capped at 10% of existing deposit.",
                    citation: Citation {
                        statute: "N.J. Stat. §46:8-21.2",
                        source: "https://lis.njleg.state.nj.us/nxt/gateway.dll?f=templates&fn=default.htm",
                    },
                },
                DepositCapRule {
                    state: "VA",
                    max_months_rent: d("2"),
                    max_months_rent_furnished: d("2"),
                    max_months_rent_small_landlord: d("2"),
                    state_preempts_local: false,
                    effective_year: 2024,
                    notes: "Va. Code §55.1-1226: max 2 months.",
                    citation: Citation {
                        statute: "Va. Code §55.1-1226",
                        source: "https://law.lis.virginia.gov/vacode/55.1-1226/",
                    },
                },
                DepositCapRule {
                    state: "DC",
                    max_months_rent: d("1"),
                    max_months_rent_furnished: d("1"),
                    max_months_rent_small_landlord: d("1"),
                    state_preempts_local: true,
                    effective_year: 2024,
                    notes: "DC Code §42-3502.17: max 1 month + interest accrual required.",
                    citation: Citation {
                        statute: "D.C. Code §42-3502.17",
                        source: "https://code.dccouncil.us/dc/council/code/sections/42-3502.17.html",
                    },
                },
                DepositCapRule {
                    state: "MD",
                    max_months_rent: d("2"),
                    max_months_rent_furnished: d("2"),
                    max_months_rent_small_landlord: d("2"),
                    state_preempts_local: false,
                    effective_year: 2024,
                    notes: "Md. Real Prop. §8-203: max 2 months. Penalty for exceeding = 3× excess + attorney's fees.",
                    citation: Citation {
                        statute: "Md. Code Real Prop. §8-203",
                        source: "https://mgaleg.maryland.gov/mgawebsite/Laws/StatuteText?article=gre&section=8-203",
                    },
                },
                DepositCapRule {
                    state: "NV",
                    max_months_rent: d("3"),
                    max_months_rent_furnished: d("3"),
                    max_months_rent_small_landlord: d("3"),
                    state_preempts_local: false,
                    effective_year: 2024,
                    notes: "NRS 118A.242: max 3 months — among the highest in the country.",
                    citation: Citation {
                        statute: "NRS 118A.242",
                        source: "https://www.leg.state.nv.us/NRS/NRS-118A.html",
                    },
                },
                DepositCapRule {
                    state: "OR",
                    max_months_rent: d("1"),
                    max_months_rent_furnished: d("1.5"),
                    max_months_rent_small_landlord: d("1"),
                    state_preempts_local: false,
                    effective_year: 2024,
                    notes: "ORS 90.300: max 1 month standard; 1.5 months for furnished units (pets / waterbed may permit additional).",
                    citation: Citation {
                        statute: "ORS 90.300",
                        source: "https://oregon.public.law/statutes/ors_90.300",
                    },
                },
                DepositCapRule {
                    state: "MI",
                    max_months_rent: d("1.5"),
                    max_months_rent_furnished: d("1.5"),
                    max_months_rent_small_landlord: d("1.5"),
                    state_preempts_local: false,
                    effective_year: 2024,
                    notes: "MCL 554.602: max 1.5 months.",
                    citation: Citation {
                        statute: "MCL 554.602",
                        source: "https://www.legislature.mi.gov/(S(0))/mileg.aspx?page=getobject&objectname=mcl-554-602",
                    },
                },
                DepositCapRule {
                    state: "IA",
                    max_months_rent: d("2"),
                    max_months_rent_furnished: d("2"),
                    max_months_rent_small_landlord: d("2"),
                    state_preempts_local: false,
                    effective_year: 2024,
                    notes: "Iowa Code §562A.12: max 2 months.",
                    citation: Citation {
                        statute: "Iowa Code §562A.12",
                        source: "https://www.legis.iowa.gov/docs/code/562A.12.pdf",
                    },
                },
                DepositCapRule {
                    state: "DE",
                    max_months_rent: d("1"),
                    max_months_rent_furnished: d("1"),
                    max_months_rent_small_landlord: d("1"),
                    state_preempts_local: false,
                    effective_year: 2024,
                    notes: "25 Del. C. §5514: max 1 month for fixed-term leases > 1 year. No cap on month-to-month or shorter terms.",
                    citation: Citation {
                        statute: "25 Del. C. §5514",
                        source: "https://delcode.delaware.gov/title25/c055/sc02/index.html",
                    },
                },
                DepositCapRule {
                    state: "KS",
                    max_months_rent: d("1"),
                    max_months_rent_furnished: d("1.5"),
                    max_months_rent_small_landlord: d("1"),
                    state_preempts_local: false,
                    effective_year: 2024,
                    notes: "K.S.A. §58-2550: max 1 month unfurnished; 1.5 months furnished.",
                    citation: Citation {
                        statute: "K.S.A. §58-2550",
                        source: "https://www.kslegislature.org/li_2024/b2023_24/statute/058_000_0000_chapter/058_025_0000_article/058_025_0050_section/058_025_0050_k/",
                    },
                },
                DepositCapRule {
                    state: "IL",
                    max_months_rent: d("0"),
                    max_months_rent_furnished: d("0"),
                    max_months_rent_small_landlord: d("0"),
                    state_preempts_local: false,
                    effective_year: 2024,
                    notes: "No state cap. Chicago RLTO max 2 months for buildings of 6+ units. Other municipalities may impose local caps.",
                    citation: Citation {
                        statute: "765 ILCS 710 + Chicago Mun. Code §5-12-080",
                        source: "https://www.chicago.gov/city/en/depts/doh/provdrs/landlords/svcs/rlto-summary.html",
                    },
                },
            ]
        });
    &R
}

pub fn rule_for(state: &str) -> Option<&'static DepositCapRule> {
    let upper = state.to_uppercase();
    rules().iter().find(|r| r.state.eq_ignore_ascii_case(&upper))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityDepositCheckInput {
    pub state: String,
    pub monthly_rent: Decimal,
    pub proposed_deposit_amount: Decimal,
    pub furnished: bool,
    /// True when landlord qualifies for the CA AB12 small-landlord
    /// carve-out (natural person owner, ≤ 2 properties totaling
    /// ≤ 4 dwelling units).
    pub small_landlord_carve_out: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityDepositCheckResult {
    pub state_recognized: bool,
    pub state_has_cap: bool,
    pub applicable_months_cap: Decimal,
    pub max_permitted_deposit: Decimal,
    pub compliant: bool,
    pub excess_amount: Decimal,
    pub statute: String,
    pub source: String,
    pub notes: String,
}

pub fn check(input: &SecurityDepositCheckInput) -> SecurityDepositCheckResult {
    let rule = match rule_for(&input.state) {
        Some(r) => r,
        None => {
            return SecurityDepositCheckResult {
                state_recognized: false,
                state_has_cap: false,
                applicable_months_cap: Decimal::ZERO,
                max_permitted_deposit: Decimal::ZERO,
                compliant: true,
                excess_amount: Decimal::ZERO,
                statute: String::new(),
                source: String::new(),
                notes: format!(
                    "no security-deposit-cap statute on file for {} — caller should verify state code directly",
                    input.state.to_uppercase()
                ),
            };
        }
    };

    // Determine applicable cap: small landlord > furnished > base.
    let months = if input.small_landlord_carve_out {
        rule.max_months_rent_small_landlord
    } else if input.furnished {
        rule.max_months_rent_furnished
    } else {
        rule.max_months_rent
    };

    let monthly_rent = input.monthly_rent.max(Decimal::ZERO);
    let proposed = input.proposed_deposit_amount.max(Decimal::ZERO);

    let has_cap = months > Decimal::ZERO;
    let max_permitted = if has_cap {
        (monthly_rent * months).round_dp(2)
    } else {
        proposed
    };
    let compliant = !has_cap || proposed <= max_permitted;
    let excess = if has_cap {
        (proposed - max_permitted).max(Decimal::ZERO)
    } else {
        Decimal::ZERO
    };

    SecurityDepositCheckResult {
        state_recognized: true,
        state_has_cap: has_cap,
        applicable_months_cap: months,
        max_permitted_deposit: max_permitted,
        compliant,
        excess_amount: excess,
        statute: rule.citation.statute.into(),
        source: rule.citation.source.into(),
        notes: rule.notes.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> SecurityDepositCheckInput {
        SecurityDepositCheckInput {
            state: "CA".into(),
            monthly_rent: dec!(2000),
            proposed_deposit_amount: dec!(2000),
            furnished: false,
            small_landlord_carve_out: false,
        }
    }

    #[test]
    fn ca_ab12_1_month_cap_for_standard_landlord() {
        // CA AB12 standard cap: 1 month. $2k rent → $2k max.
        let r = check(&base());
        assert_eq!(r.applicable_months_cap, dec!(1));
        assert_eq!(r.max_permitted_deposit, dec!(2000));
        assert!(r.compliant);
    }

    #[test]
    fn ca_proposed_above_1_month_not_compliant_standard_landlord() {
        let mut i = base();
        i.proposed_deposit_amount = dec!(4000); // 2 months
        let r = check(&i);
        assert!(!r.compliant);
        assert_eq!(r.excess_amount, dec!(2000));
    }

    #[test]
    fn ca_small_landlord_2_month_cap_allowed() {
        let mut i = base();
        i.small_landlord_carve_out = true;
        i.proposed_deposit_amount = dec!(4000); // 2 months
        let r = check(&i);
        assert_eq!(r.applicable_months_cap, dec!(2));
        assert_eq!(r.max_permitted_deposit, dec!(4000));
        assert!(r.compliant);
    }

    #[test]
    fn ny_1_month_cap_post_hstpa() {
        let mut i = base();
        i.state = "NY".into();
        let r = check(&i);
        assert_eq!(r.applicable_months_cap, dec!(1));
    }

    #[test]
    fn ma_1_month_cap_security_deposit_only() {
        // MA caps the security deposit at 1 month. Note: MA permits
        // separate collection of first + last month rent + key cost.
        let mut i = base();
        i.state = "MA".into();
        let r = check(&i);
        assert_eq!(r.applicable_months_cap, dec!(1));
        assert!(r.notes.contains("first + last"));
    }

    #[test]
    fn nj_1_5_month_cap() {
        let mut i = base();
        i.state = "NJ".into();
        i.proposed_deposit_amount = dec!(3000); // 1.5 months
        let r = check(&i);
        assert_eq!(r.applicable_months_cap, dec!(1.5));
        assert!(r.compliant);
    }

    #[test]
    fn nv_3_month_cap_highest_in_country() {
        let mut i = base();
        i.state = "NV".into();
        i.proposed_deposit_amount = dec!(6000); // 3 months
        let r = check(&i);
        assert_eq!(r.applicable_months_cap, dec!(3));
        assert!(r.compliant);
    }

    #[test]
    fn or_furnished_carve_out_1_5_months() {
        let mut i = base();
        i.state = "OR".into();
        i.furnished = true;
        i.proposed_deposit_amount = dec!(3000); // 1.5 months
        let r = check(&i);
        assert_eq!(r.applicable_months_cap, dec!(1.5));
        assert!(r.compliant);
    }

    #[test]
    fn or_unfurnished_1_month_cap() {
        let mut i = base();
        i.state = "OR".into();
        let r = check(&i);
        assert_eq!(r.applicable_months_cap, dec!(1));
    }

    #[test]
    fn ks_furnished_carve_out_1_5_months_unfurnished_1_month() {
        let mut i = base();
        i.state = "KS".into();
        let r_unf = check(&i);
        assert_eq!(r_unf.applicable_months_cap, dec!(1));

        i.furnished = true;
        let r_furn = check(&i);
        assert_eq!(r_furn.applicable_months_cap, dec!(1.5));
    }

    #[test]
    fn small_landlord_priority_over_furnished_in_ca() {
        // CA: small landlord 2 months > furnished 1 month. Small landlord wins.
        let mut i = base();
        i.small_landlord_carve_out = true;
        i.furnished = true;
        let r = check(&i);
        assert_eq!(r.applicable_months_cap, dec!(2));
    }

    #[test]
    fn il_no_state_cap_returns_state_has_cap_false() {
        let mut i = base();
        i.state = "IL".into();
        i.proposed_deposit_amount = dec!(10000);
        let r = check(&i);
        assert!(r.state_recognized);
        assert!(!r.state_has_cap);
        assert!(r.compliant);
        assert!(r.notes.contains("No state cap"));
    }

    #[test]
    fn de_long_term_1_month_cap() {
        let mut i = base();
        i.state = "DE".into();
        let r = check(&i);
        assert_eq!(r.applicable_months_cap, dec!(1));
        assert!(r.notes.contains("> 1 year"));
    }

    #[test]
    fn unknown_state_no_cap_returned_recognized_false() {
        let mut i = base();
        i.state = "XX".into();
        i.proposed_deposit_amount = dec!(5000);
        let r = check(&i);
        assert!(!r.state_recognized);
        assert!(r.compliant);
    }

    #[test]
    fn case_insensitive_state_lookup() {
        let mut i = base();
        i.state = "ca".into();
        let r = check(&i);
        assert!(r.state_recognized);
    }

    #[test]
    fn md_2_month_cap_with_3x_penalty_note() {
        let mut i = base();
        i.state = "MD".into();
        let r = check(&i);
        assert_eq!(r.applicable_months_cap, dec!(2));
        assert!(r.notes.contains("3×"));
    }

    #[test]
    fn proposed_deposit_at_exactly_cap_compliant() {
        let mut i = base();
        i.proposed_deposit_amount = dec!(2000); // exactly 1 month
        let r = check(&i);
        assert!(r.compliant);
        assert_eq!(r.excess_amount, Decimal::ZERO);
    }

    #[test]
    fn excess_amount_calculated_correctly() {
        let mut i = base();
        i.proposed_deposit_amount = dec!(5000);
        let r = check(&i);
        assert_eq!(r.excess_amount, dec!(3000)); // $5k - $2k
    }

    #[test]
    fn citation_present_for_known_states() {
        let r = check(&base());
        assert!(r.statute.contains("1950.5"));
        let mut i = base();
        i.state = "MA".into();
        let r_ma = check(&i);
        assert!(r_ma.statute.contains("15B"));
    }

    #[test]
    fn rule_for_returns_citation_for_known_states() {
        let r = rule_for("NV").unwrap();
        assert!(r.citation.statute.contains("118A.242"));
        let r = rule_for("OR").unwrap();
        assert!(r.citation.statute.contains("90.300"));
    }
}
