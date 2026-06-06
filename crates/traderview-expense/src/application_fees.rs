//! State-by-state rental application / tenant screening fee caps.
//!
//! Cluster of statutory regimes:
//!
//! - **Prohibited**: MA (M.G.L. c. 186 § 15B), VT (9 V.S.A. § 4456a) — no
//!   application fee may be charged at all
//! - **Flat cap**: NY $20 (RPL § 238-a, HSTPA 2019), VA $50 (Va. Code
//!   § 55.1-1203), WI $20 (ATCP § 134.05)
//! - **CPI-adjusted flat cap with actual-cost limiter**: CA $65.86 for 2026
//!   (Cal. Civ. Code § 1950.6) — adjusted each December
//! - **Lesser-of-cap-or-actual-cost**: MD $25 cap, actual cost limits below
//!   (Md. Code Real Prop. § 8-213)
//! - **Greater-of-flat-or-pct-of-rent**: DE the greater of $50 or 10% of
//!   monthly rent (25 Del. C. § 5514(d)) — landlord-friendly carve-out
//! - **Actual cost only**: WA (RCW § 59.18.257), OR (ORS § 90.297) — fee
//!   must equal the actual third-party screening cost, no markup
//! - **No statute**: most other states — landlord sets the fee, common-
//!   law unconscionability is the only ceiling
//!
//! All money is in **integer cents** to avoid `Decimal` precision drift in
//! the CPI-adjusted cap row (CA $65.86 = 6586¢ exact) and rounding errors
//! on the DE 10%-of-rent formula at uneven rent amounts.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The applicable rule for one state. Modeled as a tagged union so the
/// compute fn can distinguish the regimes cleanly.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FeeRule {
    /// Application fees prohibited entirely (MA, VT).
    Prohibited,
    /// No statutory ceiling — landlord sets the fee.
    NoStatute,
    /// Flat dollar cap. Some states (CA) also limit to actual screening
    /// cost when known; `capped_at_actual_cost` flips that on.
    Cap {
        cents: i64,
        capped_at_actual_cost: bool,
        cpi_adjusted: bool,
    },
    /// Greater of `min_cents` or `pct_basis_points / 10000` × monthly rent
    /// (e.g. DE: $50 or 10% of rent, whichever is greater).
    GreaterOfMinOrPctRent {
        min_cents: i64,
        pct_basis_points: u32,
    },
    /// Fee must equal the third-party screening cost exactly. No markup.
    ActualCostOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateApplicationFeeRule {
    pub state_code: &'static str,
    pub state_name: &'static str,
    pub rule: FeeRule,
    pub citation: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppFeeCheckInput {
    pub state_code: String,
    pub proposed_fee_cents: i64,
    pub monthly_rent_cents: i64,
    /// Third-party screening cost actually incurred, if known. Required
    /// for `ActualCostOnly` states and tightens the cap in `Cap` states
    /// with `capped_at_actual_cost = true`.
    pub actual_screening_cost_cents: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppFeeCheckResult {
    pub complies: bool,
    pub max_allowed_cents: Option<i64>,
    /// Amount the proposed fee exceeds the max (0 if compliant).
    pub excess_cents: i64,
    pub prohibited: bool,
    pub no_statute: bool,
    pub actual_cost_required_but_missing: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn lookup(state_code: &str) -> Option<&'static StateApplicationFeeRule> {
    let up = state_code.to_ascii_uppercase();
    TABLE.get(up.as_str()).copied()
}

pub fn all_states() -> Vec<&'static StateApplicationFeeRule> {
    let mut v: Vec<&'static StateApplicationFeeRule> = TABLE.values().copied().collect();
    v.sort_by_key(|r| r.state_code);
    v
}

pub fn check(input: &AppFeeCheckInput) -> AppFeeCheckResult {
    let rule = match lookup(&input.state_code) {
        Some(r) => r,
        None => {
            return AppFeeCheckResult {
                complies: false,
                max_allowed_cents: None,
                excess_cents: 0,
                prohibited: false,
                no_statute: true,
                actual_cost_required_but_missing: false,
                citation: "n/a",
                note: format!("unknown state code `{}`", input.state_code),
            };
        }
    };

    match rule.rule {
        FeeRule::Prohibited => {
            let complies = input.proposed_fee_cents == 0;
            AppFeeCheckResult {
                complies,
                max_allowed_cents: Some(0),
                excess_cents: input.proposed_fee_cents.max(0),
                prohibited: true,
                no_statute: false,
                actual_cost_required_but_missing: false,
                citation: rule.citation,
                note: format!(
                    "{} prohibits application fees entirely — any nonzero fee violates",
                    rule.state_name
                ),
            }
        }
        FeeRule::NoStatute => AppFeeCheckResult {
            complies: true,
            max_allowed_cents: None,
            excess_cents: 0,
            prohibited: false,
            no_statute: true,
            actual_cost_required_but_missing: false,
            citation: rule.citation,
            note: format!(
                "{} has no statutory cap; common-law unconscionability is the only ceiling",
                rule.state_name
            ),
        },
        FeeRule::Cap {
            cents,
            capped_at_actual_cost,
            cpi_adjusted,
        } => {
            let effective_cap = if capped_at_actual_cost {
                match input.actual_screening_cost_cents {
                    Some(c) => cents.min(c),
                    None => cents, // fall back to flat cap; surface note
                }
            } else {
                cents
            };
            let complies = input.proposed_fee_cents <= effective_cap;
            let excess = (input.proposed_fee_cents - effective_cap).max(0);
            let cpi_phrase = if cpi_adjusted { " (CPI-adjusted)" } else { "" };
            let cost_phrase = if capped_at_actual_cost {
                match input.actual_screening_cost_cents {
                    Some(c) => format!(
                        " — also limited by actual screening cost ({}¢)",
                        c
                    ),
                    None => " — actual screening cost not provided; using statutory cap only"
                        .to_string(),
                }
            } else {
                String::new()
            };
            AppFeeCheckResult {
                complies,
                max_allowed_cents: Some(effective_cap),
                excess_cents: excess,
                prohibited: false,
                no_statute: false,
                actual_cost_required_but_missing: false,
                citation: rule.citation,
                note: if complies {
                    format!(
                        "{} cap{} = {}¢{}; proposed {}¢ complies",
                        rule.state_name,
                        cpi_phrase,
                        effective_cap,
                        cost_phrase,
                        input.proposed_fee_cents
                    )
                } else {
                    format!(
                        "{} cap{} = {}¢{}; proposed {}¢ exceeds by {}¢",
                        rule.state_name,
                        cpi_phrase,
                        effective_cap,
                        cost_phrase,
                        input.proposed_fee_cents,
                        excess
                    )
                },
            }
        }
        FeeRule::GreaterOfMinOrPctRent {
            min_cents,
            pct_basis_points,
        } => {
            // Compute `pct_basis_points / 10000 × monthly_rent_cents` with
            // integer math to keep cents exact: pct_basis_points is in
            // hundredths of a percent (so 1000 = 10.00%). Use checked_mul
            // to avoid overflow on absurd rent values; floor division.
            let pct_of_rent = (input
                .monthly_rent_cents
                .saturating_mul(pct_basis_points as i64))
                / 10_000;
            let cap = min_cents.max(pct_of_rent);
            let complies = input.proposed_fee_cents <= cap;
            let excess = (input.proposed_fee_cents - cap).max(0);
            AppFeeCheckResult {
                complies,
                max_allowed_cents: Some(cap),
                excess_cents: excess,
                prohibited: false,
                no_statute: false,
                actual_cost_required_but_missing: false,
                citation: rule.citation,
                note: format!(
                    "{} cap = greater of {}¢ or {}.{:02}% of rent ({}¢ on rent of {}¢) = {}¢; proposed {}¢ {}",
                    rule.state_name,
                    min_cents,
                    pct_basis_points / 100,
                    pct_basis_points % 100,
                    pct_of_rent,
                    input.monthly_rent_cents,
                    cap,
                    input.proposed_fee_cents,
                    if complies { "complies" } else { "exceeds" },
                ),
            }
        }
        FeeRule::ActualCostOnly => match input.actual_screening_cost_cents {
            Some(actual) => {
                let complies = input.proposed_fee_cents <= actual;
                let excess = (input.proposed_fee_cents - actual).max(0);
                AppFeeCheckResult {
                    complies,
                    max_allowed_cents: Some(actual),
                    excess_cents: excess,
                    prohibited: false,
                    no_statute: false,
                    actual_cost_required_but_missing: false,
                    citation: rule.citation,
                    note: format!(
                        "{} requires fee = actual screening cost ({}¢); proposed {}¢ {}",
                        rule.state_name,
                        actual,
                        input.proposed_fee_cents,
                        if complies { "complies" } else { "exceeds" },
                    ),
                }
            }
            None => AppFeeCheckResult {
                complies: false,
                max_allowed_cents: None,
                excess_cents: 0,
                prohibited: false,
                no_statute: false,
                actual_cost_required_but_missing: true,
                citation: rule.citation,
                note: format!(
                    "{} requires the fee to equal the actual screening cost, but actual cost was not provided",
                    rule.state_name
                ),
            },
        },
    }
}

const fn rule(
    state_code: &'static str,
    state_name: &'static str,
    rule: FeeRule,
    citation: &'static str,
) -> StateApplicationFeeRule {
    StateApplicationFeeRule {
        state_code,
        state_name,
        rule,
        citation,
    }
}

static TABLE: Lazy<HashMap<&'static str, &'static StateApplicationFeeRule>> = Lazy::new(|| {
    use FeeRule::*;
    static RULES: &[StateApplicationFeeRule] = &[
        rule("AK", "Alaska", NoStatute, "no statute"),
        rule("AL", "Alabama", NoStatute, "no statute"),
        rule("AR", "Arkansas", NoStatute, "no statute"),
        rule("AZ", "Arizona", NoStatute, "no statute"),
        rule(
            "CA",
            "California",
            // 2026 CPI-adjusted cap = $65.86. The cap moves each December
            // — callers must refresh this value annually.
            Cap {
                cents: 6586,
                capped_at_actual_cost: true,
                cpi_adjusted: true,
            },
            "Cal. Civ. Code § 1950.6",
        ),
        rule("CO", "Colorado", NoStatute, "no statute"),
        rule("CT", "Connecticut", NoStatute, "no statute"),
        rule(
            "DC",
            "District of Columbia",
            Cap {
                cents: 5000,
                capped_at_actual_cost: false,
                cpi_adjusted: false,
            },
            "DC Code § 42-3504.07",
        ),
        rule(
            "DE",
            "Delaware",
            GreaterOfMinOrPctRent {
                min_cents: 5000,
                pct_basis_points: 1000,
            },
            "25 Del. C. § 5514(d)",
        ),
        rule("FL", "Florida", NoStatute, "no statute"),
        rule("GA", "Georgia", NoStatute, "no statute"),
        rule("HI", "Hawaii", NoStatute, "no statute"),
        rule("IA", "Iowa", NoStatute, "no statute"),
        rule("ID", "Idaho", NoStatute, "no statute"),
        rule(
            "IL",
            "Illinois",
            NoStatute,
            "no statewide statute (Chicago RLTO local)",
        ),
        rule("IN", "Indiana", NoStatute, "no statute"),
        rule("KS", "Kansas", NoStatute, "no statute"),
        rule("KY", "Kentucky", NoStatute, "no statute"),
        rule("LA", "Louisiana", NoStatute, "no statute"),
        rule("MA", "Massachusetts", Prohibited, "M.G.L. c. 186 § 15B"),
        rule(
            "MD",
            "Maryland",
            Cap {
                cents: 2500,
                capped_at_actual_cost: true,
                cpi_adjusted: false,
            },
            "Md. Code Real Prop. § 8-213",
        ),
        rule("ME", "Maine", NoStatute, "no statute"),
        rule("MI", "Michigan", NoStatute, "no statute"),
        rule("MN", "Minnesota", NoStatute, "no statute"),
        rule("MO", "Missouri", NoStatute, "no statute"),
        rule("MS", "Mississippi", NoStatute, "no statute"),
        rule("MT", "Montana", NoStatute, "no statute"),
        rule("NC", "North Carolina", NoStatute, "no statute"),
        rule("ND", "North Dakota", NoStatute, "no statute"),
        rule("NE", "Nebraska", NoStatute, "no statute"),
        rule("NH", "New Hampshire", NoStatute, "no statute"),
        rule(
            "NJ",
            "New Jersey",
            Cap {
                cents: 5000,
                capped_at_actual_cost: false,
                cpi_adjusted: false,
            },
            "N.J. Truth in Renting (AG enforcement)",
        ),
        rule("NM", "New Mexico", NoStatute, "no statute"),
        rule("NV", "Nevada", NoStatute, "no statute"),
        rule(
            "NY",
            "New York",
            Cap {
                cents: 2000,
                capped_at_actual_cost: false,
                cpi_adjusted: false,
            },
            "RPL § 238-a (HSTPA 2019)",
        ),
        rule("OH", "Ohio", NoStatute, "no statute"),
        rule("OK", "Oklahoma", NoStatute, "no statute"),
        rule("OR", "Oregon", ActualCostOnly, "ORS § 90.297"),
        rule("PA", "Pennsylvania", NoStatute, "no statute"),
        rule("RI", "Rhode Island", NoStatute, "no statute"),
        rule("SC", "South Carolina", NoStatute, "no statute"),
        rule("SD", "South Dakota", NoStatute, "no statute"),
        rule("TN", "Tennessee", NoStatute, "no statute"),
        rule("TX", "Texas", NoStatute, "no statute"),
        rule("UT", "Utah", NoStatute, "no statute"),
        rule(
            "VA",
            "Virginia",
            Cap {
                cents: 5000,
                capped_at_actual_cost: false,
                cpi_adjusted: false,
            },
            "Va. Code § 55.1-1203",
        ),
        rule("VT", "Vermont", Prohibited, "9 V.S.A. § 4456a"),
        rule("WA", "Washington", ActualCostOnly, "RCW § 59.18.257"),
        rule(
            "WI",
            "Wisconsin",
            Cap {
                cents: 2000,
                capped_at_actual_cost: false,
                cpi_adjusted: false,
            },
            "Wis. Admin. Code ATCP § 134.05",
        ),
        rule("WV", "West Virginia", NoStatute, "no statute"),
        rule("WY", "Wyoming", NoStatute, "no statute"),
    ];
    RULES.iter().map(|r| (r.state_code, r)).collect()
});

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: &str, proposed: i64, rent: i64, actual: Option<i64>) -> AppFeeCheckInput {
        AppFeeCheckInput {
            state_code: state.to_string(),
            proposed_fee_cents: proposed,
            monthly_rent_cents: rent,
            actual_screening_cost_cents: actual,
        }
    }

    #[test]
    fn table_covers_all_50_states_plus_dc() {
        // 51 rows — same invariant as the other landlord state-data tables.
        assert_eq!(TABLE.len(), 51);
    }

    #[test]
    fn massachusetts_prohibits_any_fee() {
        // $0 → complies. Any nonzero → violates.
        let zero = check(&input("MA", 0, 200_000, None));
        assert!(zero.complies);
        assert!(zero.prohibited);
        assert_eq!(zero.max_allowed_cents, Some(0));

        let any = check(&input("MA", 1, 200_000, None));
        assert!(!any.complies);
        assert_eq!(any.excess_cents, 1);

        let big = check(&input("MA", 5000, 200_000, None));
        assert!(!big.complies);
        assert_eq!(big.excess_cents, 5000);
    }

    #[test]
    fn vermont_prohibits_any_fee() {
        // Mirror of MA but distinct citation row. Pinning both states
        // independently catches a future copy-paste merge that would
        // collapse the citations.
        let r = check(&input("VT", 100, 200_000, None));
        assert!(!r.complies);
        assert!(r.prohibited);
        assert!(r.citation.contains("9 V.S.A. § 4456a"));
    }

    #[test]
    fn new_york_twenty_dollar_cap_exact_boundary() {
        // $20.00 = 2000¢ is the RPL § 238-a cap. Exact match complies;
        // $20.01 exceeds by 1¢.
        let pass = check(&input("NY", 2000, 200_000, None));
        assert!(pass.complies);
        assert_eq!(pass.max_allowed_cents, Some(2000));
        assert_eq!(pass.excess_cents, 0);

        let fail = check(&input("NY", 2001, 200_000, None));
        assert!(!fail.complies);
        assert_eq!(fail.excess_cents, 1);
    }

    #[test]
    fn california_cpi_cap_2026_amount_and_actual_cost_clamp() {
        // 2026 CPI cap = $65.86 = 6586¢. With actual screening cost of
        // $50.00 = 5000¢, the effective cap is the lower of the two
        // (CA is lesser-of by statute). A proposed $60 fee exceeds the
        // $50 actual cost limit even though it's under the CPI cap.
        let with_cost = check(&input("CA", 6000, 250_000, Some(5000)));
        assert!(!with_cost.complies);
        assert_eq!(with_cost.max_allowed_cents, Some(5000));
        assert_eq!(with_cost.excess_cents, 1000);

        // Same proposed fee, but no actual cost provided → use the CPI
        // cap as the ceiling. $60 < $65.86 → complies.
        let no_cost = check(&input("CA", 6000, 250_000, None));
        assert!(no_cost.complies);
        assert_eq!(no_cost.max_allowed_cents, Some(6586));
        assert!(no_cost.note.contains("CPI-adjusted"));
    }

    #[test]
    fn maryland_lesser_of_25_and_actual_cost() {
        // MD cap is the lesser of $25 (2500¢) and the actual cost.
        // Actual cost $30 (3000¢) → effective cap stays at $25.
        let actual_above_cap = check(&input("MD", 2500, 150_000, Some(3000)));
        assert!(actual_above_cap.complies);
        assert_eq!(actual_above_cap.max_allowed_cents, Some(2500));

        // Actual cost $15 (1500¢) → effective cap drops to $15.
        let actual_below_cap = check(&input("MD", 2000, 150_000, Some(1500)));
        assert!(!actual_below_cap.complies);
        assert_eq!(actual_below_cap.max_allowed_cents, Some(1500));
        assert_eq!(actual_below_cap.excess_cents, 500);
    }

    #[test]
    fn delaware_greater_of_50_or_10pct_rent() {
        // DE rule: greater of $50 (5000¢) or 10% of monthly rent.
        // Rent $400/mo (40_000¢) → 10% = $40 (4000¢) < $50 → cap = $50.
        let low_rent = check(&input("DE", 5000, 40_000, None));
        assert!(low_rent.complies);
        assert_eq!(low_rent.max_allowed_cents, Some(5000));

        // Rent $1000/mo (100_000¢) → 10% = $100 (10_000¢) > $50 → cap = $100.
        let high_rent = check(&input("DE", 10_000, 100_000, None));
        assert!(high_rent.complies);
        assert_eq!(high_rent.max_allowed_cents, Some(10_000));

        // Rent $1000/mo, proposed $101 → exceeds $100 cap by $1.
        let over = check(&input("DE", 10_100, 100_000, None));
        assert!(!over.complies);
        assert_eq!(over.excess_cents, 100);
    }

    #[test]
    fn delaware_pct_rounds_down_at_uneven_rent() {
        // Rent of $1,234.56 (123_456¢). 10% = 12_345.6¢ → integer floor
        // = 12_345¢. The greater of 5000 or 12345 = 12345.
        let r = check(&input("DE", 12_345, 123_456, None));
        assert!(r.complies);
        assert_eq!(r.max_allowed_cents, Some(12_345));

        // $12_346 exceeds by 1¢.
        let r2 = check(&input("DE", 12_346, 123_456, None));
        assert!(!r2.complies);
        assert_eq!(r2.excess_cents, 1);
    }

    #[test]
    fn washington_actual_cost_only_with_cost_provided() {
        // WA: fee must equal actual screening cost. Actual $35 → $35 OK,
        // $36 over.
        let pass = check(&input("WA", 3500, 200_000, Some(3500)));
        assert!(pass.complies);
        let fail = check(&input("WA", 3600, 200_000, Some(3500)));
        assert!(!fail.complies);
        assert_eq!(fail.excess_cents, 100);
    }

    #[test]
    fn washington_actual_cost_required_flag_set_when_missing() {
        // Without actual cost, compliance can't be determined — surface
        // the missing-input flag so the caller knows to ask.
        let r = check(&input("WA", 3500, 200_000, None));
        assert!(!r.complies);
        assert!(r.actual_cost_required_but_missing);
        assert!(r.note.contains("actual screening cost"));
    }

    #[test]
    fn oregon_actual_cost_only_mirrors_washington() {
        // Same regime as WA but distinct citation. Pinning both states
        // separately ensures a future statute change to one doesn't
        // silently drag the other.
        let r = check(&input("OR", 1000, 200_000, Some(1000)));
        assert!(r.complies);
        assert!(r.citation.contains("ORS § 90.297"));
    }

    #[test]
    fn no_statute_states_accept_any_proposed_fee() {
        // Texas, Florida, Colorado — landlord can charge what they want.
        // Compute reports compliant with `max_allowed_cents: None`.
        for code in ["TX", "FL", "CO", "AZ", "GA"] {
            let r = check(&input(code, 50_000, 200_000, None));
            assert!(r.complies);
            assert!(r.no_statute);
            assert!(r.max_allowed_cents.is_none());
        }
    }

    #[test]
    fn unknown_state_returns_no_statute_with_error_note() {
        let r = check(&input("ZZ", 5000, 200_000, None));
        assert!(!r.complies);
        assert!(r.no_statute);
        assert!(r.note.contains("unknown state code"));
    }

    #[test]
    fn lookup_case_insensitive() {
        assert!(lookup("CA").is_some());
        assert!(lookup("ca").is_some());
        assert!(lookup("Ca").is_some());
        assert!(lookup("zz").is_none());
    }

    #[test]
    fn all_states_returns_sorted_by_code() {
        let states = all_states();
        assert_eq!(states.len(), 51);
        assert_eq!(states.first().unwrap().state_code, "AK");
        assert_eq!(states.last().unwrap().state_code, "WY");
    }

    #[test]
    fn citation_present_for_every_row() {
        for r in TABLE.values() {
            assert!(
                !r.citation.is_empty(),
                "{} has empty citation",
                r.state_code
            );
        }
    }

    #[test]
    fn flat_cap_states_at_boundary_pinned() {
        // NY $20, VA $50, WI $20, DC $50, NJ $50. Each exact boundary
        // complies; $1 over fails. Catches any future cap typo.
        let cases = [
            ("NY", 2000_i64),
            ("VA", 5000),
            ("WI", 2000),
            ("DC", 5000),
            ("NJ", 5000),
        ];
        for (state, cap) in cases {
            let pass = check(&input(state, cap, 200_000, None));
            assert!(pass.complies, "{state} at cap {cap} should comply");
            let fail = check(&input(state, cap + 1, 200_000, None));
            assert!(!fail.complies, "{state} at cap+1 should fail");
        }
    }

    #[test]
    fn massachusetts_prohibited_flag_distinct_from_no_statute() {
        // MA flags `prohibited: true` and `no_statute: false`. The two
        // flags are mutually exclusive and a downstream filter must
        // distinguish them (MA tenant can sue; TX tenant cannot).
        let ma = check(&input("MA", 1, 200_000, None));
        assert!(ma.prohibited);
        assert!(!ma.no_statute);

        let tx = check(&input("TX", 1, 200_000, None));
        assert!(!tx.prohibited);
        assert!(tx.no_statute);
    }

    #[test]
    fn california_without_actual_cost_uses_cpi_cap_only() {
        // With no actual_cost, the lesser-of clause has only one operand
        // (the CPI cap) so the cap stays at $65.86. The note must mention
        // that actual cost wasn't provided so the downstream user knows
        // the result might be over-permissive.
        let r = check(&input("CA", 6586, 250_000, None));
        assert!(r.complies);
        assert!(r.note.contains("actual screening cost not provided"));
    }

    #[test]
    fn delaware_zero_rent_falls_back_to_flat_minimum() {
        // Rent = 0 → 10% × 0 = 0. Greater of $50 or $0 = $50. The flat
        // floor protects the landlord on free-rent (concession) periods.
        let r = check(&input("DE", 5000, 0, None));
        assert!(r.complies);
        assert_eq!(r.max_allowed_cents, Some(5000));
    }

    #[test]
    fn proposed_fee_zero_always_complies_in_capped_states() {
        // A landlord charging $0 fee can never violate a cap rule (caps
        // are upper bounds). Pinned for every Cap-rule state to catch
        // a future sign-flip bug.
        for state in ["NY", "VA", "WI", "CA", "MD", "DC", "NJ"] {
            let r = check(&input(state, 0, 200_000, Some(5000)));
            assert!(r.complies, "{state} should accept $0 fee");
            assert_eq!(r.excess_cents, 0);
        }
    }

    #[test]
    fn note_describes_actual_cost_clamp_for_california() {
        // CA note must spell out both the CPI flavor AND the actual-cost
        // limiter when both apply — readability for the downstream UI.
        let r = check(&input("CA", 5500, 250_000, Some(5500)));
        assert!(r.note.contains("CPI-adjusted"));
        assert!(r.note.contains("actual screening cost"));
    }

    #[test]
    fn negative_proposed_fee_treated_as_zero_for_excess() {
        // A negative proposed fee is nonsensical input — pinning that
        // excess_cents stays non-negative under the max() clamp so the
        // result is stable rather than reporting a "negative overage".
        let r = check(&input("NY", -100, 200_000, None));
        assert!(r.complies);
        assert_eq!(r.excess_cents, 0);
    }
}
