//! State-by-state just-cause eviction availability + relocation
//! assistance formulas. Major statutory shift 2019-2021 — CA AB 1482
//! (Tenant Protection Act 2019), OR SB 608, WA HB 1236 — changed
//! whether a landlord can terminate at lease end at all, and required
//! relocation assistance equal to one month's rent for no-fault grounds.
//!
//! Four regimes:
//!
//! 1. **Statewide just-cause after N months** — CA (12mo, AB 1482),
//!    OR (12mo, SB 608). After the threshold the tenancy can be
//!    terminated only on a statutorily-listed ground. No-fault grounds
//!    (owner move-in, substantial remodel, withdrawal from market)
//!    require relocation assistance equal to one month's current rent.
//!    Some states carve out small landlords (OR ≤ 4 units exempt from
//!    relocation; CA single-family-rental owner-occupied unit exempt
//!    from the regime entirely).
//!
//! 2. **Statewide just-cause from day one** — WA (HB 1236, 2021),
//!    DC (§ 42-3505), NJ (Anti-Eviction Act 1974). Just cause applies
//!    immediately; no waiting period. WA's relocation assistance for
//!    owner-move-in and sale-of-SFR grounds is one month's rent under
//!    RCW 59.18.650.
//!
//! 3. **Partial by building** — NY HSTPA (rent-regulated only),
//!    Chicago/Mt. Prospect/Evanston rent-stabilized buildings, ME 4+
//!    unit buildings. Statewide just-cause does not reach all rentals;
//!    caller must additionally check rent-regulation status.
//!
//! 4. **No statewide regime** — most states; common-law no-cause
//!    termination at lease end remains available (subject to local
//!    ordinances and the §§ rent_control / eviction_notices modules).
//!
//! **Relocation assistance is current-rent-anchored, not market rent.**
//! The figure is the tenant's monthly rent as of the termination notice
//! date, not what a new tenant would pay. This is a deliberate pro-
//! tenant choice — the displaced tenant is compensated for what THEY
//! were paying, not the (potentially higher) market.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum JustCauseRegime {
    /// Statewide just-cause required after `threshold_months` of tenancy.
    /// Relocation assistance for no-fault grounds = monthly rent ×
    /// `relocation_months_rent`. Optional small-landlord exemption.
    StatewideAfterMonths {
        threshold_months: u32,
        relocation_months_rent: u32,
        small_landlord_exemption_units: Option<u32>,
    },
    /// Just-cause from day 1 of tenancy. Same relocation formula.
    StatewideDayOne { relocation_months_rent: u32 },
    /// Just-cause exists but only for rent-regulated / older / multi-
    /// unit buildings. Statewide regime does NOT reach all rentals;
    /// caller must check rent-regulation status separately.
    PartialByBuilding,
    /// No statewide just-cause; common-law no-cause termination at
    /// lease end remains available (subject to local ordinances).
    NoStatewide,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateJustCauseRule {
    pub state_code: &'static str,
    pub state_name: &'static str,
    pub regime: JustCauseRegime,
    pub citation: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvictionGrounds {
    /// Non-payment, lease violation, nuisance, criminal activity, etc.
    AtFault,
    NoFaultOwnerMoveIn,
    NoFaultSubstantialRemodel,
    NoFaultWithdrawalFromMarket,
    /// Common-law end-of-lease termination with no grounds stated.
    NoCause,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JustCauseInput {
    pub state_code: String,
    pub tenancy_duration_months: u32,
    pub landlord_unit_count: u32,
    pub grounds_for_eviction: EvictionGrounds,
    pub monthly_rent_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JustCauseResult {
    /// True if the state requires just cause for the current tenancy
    /// length / building.
    pub just_cause_required: bool,
    /// True if no-cause termination (lease-end notice without grounds)
    /// remains available.
    pub no_cause_termination_available: bool,
    /// True if the proposed grounds satisfy the just-cause requirement
    /// (true for AtFault grounds and the three No-Fault types; false
    /// for NoCause when just-cause is required).
    pub grounds_satisfy_just_cause: bool,
    pub relocation_assistance_required: bool,
    pub relocation_assistance_cents: i64,
    pub small_landlord_exempt_from_relocation: bool,
    pub partial_by_building_check_rent_regulation: bool,
    pub no_statewide_regime: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn lookup(state_code: &str) -> Option<&'static StateJustCauseRule> {
    let up = state_code.to_ascii_uppercase();
    TABLE.get(up.as_str()).copied()
}

pub fn all_states() -> Vec<&'static StateJustCauseRule> {
    let mut v: Vec<&'static StateJustCauseRule> = TABLE.values().copied().collect();
    v.sort_by_key(|r| r.state_code);
    v
}

pub fn check(input: &JustCauseInput) -> JustCauseResult {
    let rule = match lookup(&input.state_code) {
        Some(r) => r,
        None => {
            return JustCauseResult {
                just_cause_required: false,
                no_cause_termination_available: true,
                grounds_satisfy_just_cause: true,
                relocation_assistance_required: false,
                relocation_assistance_cents: 0,
                small_landlord_exempt_from_relocation: false,
                partial_by_building_check_rent_regulation: false,
                no_statewide_regime: true,
                citation: "n/a",
                note: format!("unknown state code `{}`", input.state_code),
            };
        }
    };

    let is_no_fault = matches!(
        input.grounds_for_eviction,
        EvictionGrounds::NoFaultOwnerMoveIn
            | EvictionGrounds::NoFaultSubstantialRemodel
            | EvictionGrounds::NoFaultWithdrawalFromMarket
    );
    let is_no_cause = matches!(input.grounds_for_eviction, EvictionGrounds::NoCause);

    match rule.regime {
        JustCauseRegime::NoStatewide => JustCauseResult {
            just_cause_required: false,
            no_cause_termination_available: true,
            grounds_satisfy_just_cause: true,
            relocation_assistance_required: false,
            relocation_assistance_cents: 0,
            small_landlord_exempt_from_relocation: false,
            partial_by_building_check_rent_regulation: false,
            no_statewide_regime: true,
            citation: rule.citation,
            note: format!(
                "{} has no statewide just-cause regime; common-law no-cause termination at lease end remains available (subject to local ordinances)",
                rule.state_name
            ),
        },
        JustCauseRegime::PartialByBuilding => JustCauseResult {
            just_cause_required: false,
            no_cause_termination_available: true,
            grounds_satisfy_just_cause: true,
            relocation_assistance_required: false,
            relocation_assistance_cents: 0,
            small_landlord_exempt_from_relocation: false,
            partial_by_building_check_rent_regulation: true,
            no_statewide_regime: false,
            citation: rule.citation,
            note: format!(
                "{} just-cause applies only to rent-regulated / qualifying buildings — caller must check rent-regulation status separately",
                rule.state_name
            ),
        },
        JustCauseRegime::StatewideAfterMonths {
            threshold_months,
            relocation_months_rent,
            small_landlord_exemption_units,
        } => {
            let past_threshold = input.tenancy_duration_months >= threshold_months;
            if !past_threshold {
                return JustCauseResult {
                    just_cause_required: false,
                    no_cause_termination_available: true,
                    grounds_satisfy_just_cause: true,
                    relocation_assistance_required: false,
                    relocation_assistance_cents: 0,
                    small_landlord_exempt_from_relocation: false,
                    partial_by_building_check_rent_regulation: false,
                    no_statewide_regime: false,
                    citation: rule.citation,
                    note: format!(
                        "{}: tenancy {}mo < {}mo threshold — just-cause not yet required; no-cause termination still available",
                        rule.state_name, input.tenancy_duration_months, threshold_months
                    ),
                };
            }
            let small_landlord_exempt = small_landlord_exemption_units
                .map(|cap| input.landlord_unit_count <= cap)
                .unwrap_or(false);
            let (reloc_required, reloc_cents) = if is_no_fault {
                if small_landlord_exempt {
                    (false, 0)
                } else {
                    (
                        true,
                        input
                            .monthly_rent_cents
                            .saturating_mul(relocation_months_rent as i64),
                    )
                }
            } else {
                (false, 0)
            };
            JustCauseResult {
                just_cause_required: true,
                no_cause_termination_available: false,
                grounds_satisfy_just_cause: !is_no_cause,
                relocation_assistance_required: reloc_required,
                relocation_assistance_cents: reloc_cents,
                small_landlord_exempt_from_relocation: small_landlord_exempt,
                partial_by_building_check_rent_regulation: false,
                no_statewide_regime: false,
                citation: rule.citation,
                note: format!(
                    "{}: tenancy {}mo ≥ {}mo threshold — just-cause required. Grounds {} satisfy. Relocation = ${} ({}{}× rent)",
                    rule.state_name,
                    input.tenancy_duration_months,
                    threshold_months,
                    if is_no_cause { "DO NOT" } else { "DO" },
                    reloc_cents,
                    if small_landlord_exempt {
                        "small-landlord exempt; was "
                    } else {
                        ""
                    },
                    relocation_months_rent
                ),
            }
        }
        JustCauseRegime::StatewideDayOne {
            relocation_months_rent,
        } => {
            let (reloc_required, reloc_cents) = if is_no_fault {
                (
                    true,
                    input
                        .monthly_rent_cents
                        .saturating_mul(relocation_months_rent as i64),
                )
            } else {
                (false, 0)
            };
            JustCauseResult {
                just_cause_required: true,
                no_cause_termination_available: false,
                grounds_satisfy_just_cause: !is_no_cause,
                relocation_assistance_required: reloc_required,
                relocation_assistance_cents: reloc_cents,
                small_landlord_exempt_from_relocation: false,
                partial_by_building_check_rent_regulation: false,
                no_statewide_regime: false,
                citation: rule.citation,
                note: format!(
                    "{}: just-cause required from day 1. Grounds {} satisfy. Relocation = ${} ({}× rent)",
                    rule.state_name,
                    if is_no_cause { "DO NOT" } else { "DO" },
                    reloc_cents,
                    relocation_months_rent
                ),
            }
        }
    }
}

const fn rule(
    state_code: &'static str,
    state_name: &'static str,
    regime: JustCauseRegime,
    citation: &'static str,
) -> StateJustCauseRule {
    StateJustCauseRule {
        state_code,
        state_name,
        regime,
        citation,
    }
}

static TABLE: Lazy<HashMap<&'static str, &'static StateJustCauseRule>> = Lazy::new(|| {
    use JustCauseRegime::*;
    static RULES: &[StateJustCauseRule] = &[
        rule("AK", "Alaska", NoStatewide, "no statute"),
        rule("AL", "Alabama", NoStatewide, "no statute"),
        rule("AR", "Arkansas", NoStatewide, "no statute"),
        rule("AZ", "Arizona", NoStatewide, "no statute"),
        rule(
            "CA",
            "California",
            StatewideAfterMonths {
                threshold_months: 12,
                relocation_months_rent: 1,
                small_landlord_exemption_units: None,
            },
            "Cal. Civ. Code § 1946.2 (AB 1482, Tenant Protection Act 2019)",
        ),
        rule("CO", "Colorado", NoStatewide, "no statewide statute"),
        rule("CT", "Connecticut", NoStatewide, "no statewide statute"),
        rule(
            "DC",
            "District of Columbia",
            StatewideDayOne {
                relocation_months_rent: 0,
            },
            "D.C. Code § 42-3505.01 (Rental Housing Act)",
        ),
        rule("DE", "Delaware", NoStatewide, "no statewide statute"),
        rule("FL", "Florida", NoStatewide, "no statewide statute"),
        rule("GA", "Georgia", NoStatewide, "no statewide statute"),
        rule("HI", "Hawaii", NoStatewide, "no statewide statute"),
        rule("IA", "Iowa", NoStatewide, "no statewide statute"),
        rule("ID", "Idaho", NoStatewide, "no statewide statute"),
        rule(
            "IL",
            "Illinois",
            PartialByBuilding,
            "no statewide; Chicago RLTO + Evanston / Mt. Prospect just-cause ordinances",
        ),
        rule("IN", "Indiana", NoStatewide, "no statewide statute"),
        rule("KS", "Kansas", NoStatewide, "no statewide statute"),
        rule("KY", "Kentucky", NoStatewide, "no statewide statute"),
        rule("LA", "Louisiana", NoStatewide, "no statewide statute"),
        rule("MA", "Massachusetts", NoStatewide, "no statewide statute"),
        rule("MD", "Maryland", NoStatewide, "no statewide statute"),
        rule(
            "ME",
            "Maine",
            PartialByBuilding,
            "14 M.R.S. § 6002 — just-cause applies to 4+ unit buildings only",
        ),
        rule("MI", "Michigan", NoStatewide, "no statewide statute"),
        rule("MN", "Minnesota", NoStatewide, "no statewide statute"),
        rule("MO", "Missouri", NoStatewide, "no statewide statute"),
        rule("MS", "Mississippi", NoStatewide, "no statewide statute"),
        rule("MT", "Montana", NoStatewide, "no statewide statute"),
        rule("NC", "North Carolina", NoStatewide, "no statewide statute"),
        rule("ND", "North Dakota", NoStatewide, "no statewide statute"),
        rule("NE", "Nebraska", NoStatewide, "no statewide statute"),
        rule("NH", "New Hampshire", NoStatewide, "no statewide statute"),
        rule(
            "NJ",
            "New Jersey",
            StatewideDayOne {
                relocation_months_rent: 0,
            },
            "N.J.S.A. § 2A:18-61.1 (Anti-Eviction Act 1974)",
        ),
        rule("NM", "New Mexico", NoStatewide, "no statewide statute"),
        rule("NV", "Nevada", NoStatewide, "no statewide statute"),
        rule(
            "NY",
            "New York",
            PartialByBuilding,
            "RPL § 226-c (HSTPA 2019) — just-cause for rent-stabilized only; statewide notice-period scaling",
        ),
        rule("OH", "Ohio", NoStatewide, "no statewide statute"),
        rule("OK", "Oklahoma", NoStatewide, "no statewide statute"),
        rule(
            "OR",
            "Oregon",
            StatewideAfterMonths {
                threshold_months: 12,
                relocation_months_rent: 1,
                small_landlord_exemption_units: Some(4),
            },
            "ORS § 90.427 (SB 608, 2019)",
        ),
        rule("PA", "Pennsylvania", NoStatewide, "no statewide statute"),
        rule("RI", "Rhode Island", NoStatewide, "no statewide statute"),
        rule("SC", "South Carolina", NoStatewide, "no statewide statute"),
        rule("SD", "South Dakota", NoStatewide, "no statewide statute"),
        rule("TN", "Tennessee", NoStatewide, "no statewide statute"),
        rule("TX", "Texas", NoStatewide, "no statewide statute"),
        rule("UT", "Utah", NoStatewide, "no statewide statute"),
        rule("VA", "Virginia", NoStatewide, "no statewide statute"),
        rule("VT", "Vermont", NoStatewide, "no statewide statute"),
        rule(
            "WA",
            "Washington",
            StatewideDayOne {
                relocation_months_rent: 1,
            },
            "RCW § 59.18.650 (HB 1236, 2021)",
        ),
        rule("WI", "Wisconsin", NoStatewide, "no statewide statute"),
        rule("WV", "West Virginia", NoStatewide, "no statewide statute"),
        rule("WY", "Wyoming", NoStatewide, "no statewide statute"),
    ];
    RULES.iter().map(|r| (r.state_code, r)).collect()
});

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        state: &str,
        months: u32,
        units: u32,
        grounds: EvictionGrounds,
        rent_cents: i64,
    ) -> JustCauseInput {
        JustCauseInput {
            state_code: state.to_string(),
            tenancy_duration_months: months,
            landlord_unit_count: units,
            grounds_for_eviction: grounds,
            monthly_rent_cents: rent_cents,
        }
    }

    #[test]
    fn table_covers_all_50_states_plus_dc() {
        assert_eq!(TABLE.len(), 51);
    }

    #[test]
    fn california_under_12_months_no_just_cause_required() {
        // CA AB 1482 only kicks in at 12 months. 11 months → no-cause
        // termination still available.
        let r = check(&input("CA", 11, 100, EvictionGrounds::NoCause, 300_000));
        assert!(!r.just_cause_required);
        assert!(r.no_cause_termination_available);
        assert!(r.grounds_satisfy_just_cause);
        assert_eq!(r.relocation_assistance_cents, 0);
    }

    #[test]
    fn california_at_12_months_just_cause_required() {
        // Exactly 12-month boundary: AB 1482 applies. NoCause grounds
        // do NOT satisfy.
        let r = check(&input("CA", 12, 100, EvictionGrounds::NoCause, 300_000));
        assert!(r.just_cause_required);
        assert!(!r.no_cause_termination_available);
        assert!(!r.grounds_satisfy_just_cause);
    }

    #[test]
    fn california_at_fault_grounds_no_relocation() {
        // At-fault grounds satisfy just-cause but do NOT require
        // relocation assistance.
        let r = check(&input("CA", 12, 100, EvictionGrounds::AtFault, 300_000));
        assert!(r.just_cause_required);
        assert!(r.grounds_satisfy_just_cause);
        assert!(!r.relocation_assistance_required);
        assert_eq!(r.relocation_assistance_cents, 0);
    }

    #[test]
    fn california_no_fault_owner_move_in_relocation_one_month_rent() {
        // No-fault owner move-in: relocation = 1 month current rent.
        // No small-landlord exemption in CA.
        let r = check(&input(
            "CA",
            24,
            100,
            EvictionGrounds::NoFaultOwnerMoveIn,
            300_000,
        ));
        assert!(r.relocation_assistance_required);
        assert_eq!(r.relocation_assistance_cents, 300_000);
        assert!(!r.small_landlord_exempt_from_relocation);
    }

    #[test]
    fn oregon_small_landlord_4_units_exempt_from_relocation() {
        // OR SB 608: small landlord (≤ 4 units) exempt from relocation
        // even on no-fault grounds. Tenant still entitled to just-cause
        // protection — landlord still needs grounds — but no $$$.
        let r = check(&input(
            "OR",
            24,
            4,
            EvictionGrounds::NoFaultOwnerMoveIn,
            250_000,
        ));
        assert!(r.just_cause_required);
        assert!(r.grounds_satisfy_just_cause);
        assert!(r.small_landlord_exempt_from_relocation);
        assert!(!r.relocation_assistance_required);
        assert_eq!(r.relocation_assistance_cents, 0);
    }

    #[test]
    fn oregon_5_unit_landlord_owes_relocation() {
        // 5 units → above the small-landlord threshold. Relocation =
        // 1 month rent = $2,500.
        let r = check(&input(
            "OR",
            24,
            5,
            EvictionGrounds::NoFaultOwnerMoveIn,
            250_000,
        ));
        assert!(r.relocation_assistance_required);
        assert_eq!(r.relocation_assistance_cents, 250_000);
        assert!(!r.small_landlord_exempt_from_relocation);
    }

    #[test]
    fn oregon_at_12_months_boundary() {
        // OR SB 608 12-month threshold mirrors CA AB 1482.
        let under = check(&input("OR", 11, 100, EvictionGrounds::NoCause, 300_000));
        assert!(!under.just_cause_required);
        let at = check(&input("OR", 12, 100, EvictionGrounds::NoCause, 300_000));
        assert!(at.just_cause_required);
    }

    #[test]
    fn washington_just_cause_day_one() {
        // WA HB 1236 (2021): just-cause from day 1, no threshold.
        // Tenancy of 1 month still gets just-cause protection.
        let r = check(&input("WA", 1, 100, EvictionGrounds::NoCause, 200_000));
        assert!(r.just_cause_required);
        assert!(!r.no_cause_termination_available);
        assert!(!r.grounds_satisfy_just_cause);
    }

    #[test]
    fn washington_no_fault_owner_move_in_one_month_relocation() {
        let r = check(&input(
            "WA",
            12,
            100,
            EvictionGrounds::NoFaultOwnerMoveIn,
            200_000,
        ));
        assert!(r.relocation_assistance_required);
        assert_eq!(r.relocation_assistance_cents, 200_000);
    }

    #[test]
    fn new_jersey_anti_eviction_act_day_one_no_relocation() {
        // NJ Anti-Eviction Act 1974 — strongest pro-tenant in the US.
        // Just-cause from day 1, but no statutory relocation assistance.
        let r = check(&input(
            "NJ",
            1,
            100,
            EvictionGrounds::NoFaultOwnerMoveIn,
            300_000,
        ));
        assert!(r.just_cause_required);
        // No relocation under the NJ regime (0× rent multiplier).
        assert_eq!(r.relocation_assistance_cents, 0);
    }

    #[test]
    fn dc_just_cause_day_one_no_relocation() {
        // DC Rental Housing Act § 42-3505.01: just-cause day 1, no
        // statewide relocation requirement (separate rent-control rules
        // may impose obligations).
        let r = check(&input("DC", 1, 100, EvictionGrounds::NoCause, 300_000));
        assert!(r.just_cause_required);
        assert!(!r.grounds_satisfy_just_cause);
        assert_eq!(r.relocation_assistance_cents, 0);
    }

    #[test]
    fn ny_partial_by_building_flag_set() {
        // NY HSTPA 2019: just-cause applies to rent-stabilized buildings
        // only. The PartialByBuilding regime flags this so the caller
        // checks rent-regulation status separately.
        let r = check(&input("NY", 24, 100, EvictionGrounds::NoCause, 300_000));
        assert!(r.partial_by_building_check_rent_regulation);
        assert!(!r.just_cause_required);
        assert!(r.no_cause_termination_available);
    }

    #[test]
    fn maine_partial_by_building_for_four_plus_unit() {
        // ME 14 M.R.S. § 6002: just-cause for 4+ unit buildings only.
        // Same flag as NY.
        let r = check(&input("ME", 24, 100, EvictionGrounds::NoCause, 300_000));
        assert!(r.partial_by_building_check_rent_regulation);
    }

    #[test]
    fn illinois_partial_by_building_flag_set_for_local_ordinances() {
        // IL: no statewide regime but Chicago RLTO + Evanston + Mt.
        // Prospect have local just-cause. Caller checks municipality.
        let r = check(&input("IL", 24, 100, EvictionGrounds::NoCause, 300_000));
        assert!(r.partial_by_building_check_rent_regulation);
    }

    #[test]
    fn no_statewide_states_pass_through_no_cause() {
        // Texas, Florida, Colorado, etc. — no statewide just-cause.
        // No-cause termination at lease end remains available.
        for code in ["TX", "FL", "CO", "AZ", "AL", "WY", "ID"] {
            let r = check(&input(code, 100, 100, EvictionGrounds::NoCause, 300_000));
            assert!(
                !r.just_cause_required,
                "{code} should not require just cause"
            );
            assert!(r.no_cause_termination_available);
            assert!(r.no_statewide_regime);
        }
    }

    #[test]
    fn unknown_state_marked_no_statewide() {
        let r = check(&input("ZZ", 12, 100, EvictionGrounds::NoCause, 300_000));
        assert!(r.no_statewide_regime);
        assert!(r.note.contains("unknown state code"));
    }

    #[test]
    fn lookup_case_insensitive() {
        assert!(lookup("CA").is_some());
        assert!(lookup("ca").is_some());
        assert!(lookup("Ca").is_some());
    }

    #[test]
    fn all_states_sorted_by_code() {
        let states = all_states();
        assert_eq!(states.len(), 51);
        assert_eq!(states.first().unwrap().state_code, "AK");
        assert_eq!(states.last().unwrap().state_code, "WY");
    }

    #[test]
    fn citation_present_for_every_row() {
        for r in TABLE.values() {
            assert!(!r.citation.is_empty(), "{} citation empty", r.state_code);
        }
    }

    #[test]
    fn no_fault_substantial_remodel_triggers_relocation_in_ca() {
        // Substantial remodel and withdrawal from market are also no-
        // fault grounds; same relocation as owner move-in.
        let r = check(&input(
            "CA",
            12,
            100,
            EvictionGrounds::NoFaultSubstantialRemodel,
            300_000,
        ));
        assert!(r.relocation_assistance_required);
        assert_eq!(r.relocation_assistance_cents, 300_000);
    }

    #[test]
    fn no_fault_withdrawal_from_market_triggers_relocation_in_ca() {
        let r = check(&input(
            "CA",
            12,
            100,
            EvictionGrounds::NoFaultWithdrawalFromMarket,
            300_000,
        ));
        assert!(r.relocation_assistance_required);
        assert_eq!(r.relocation_assistance_cents, 300_000);
    }

    #[test]
    fn oregon_at_fault_grounds_no_relocation_regardless_of_unit_count() {
        // At-fault grounds never trigger relocation, even when the
        // landlord owns 100 units. Small-landlord-exempt flag stays
        // false because the exemption only matters for no-fault grounds.
        let r = check(&input("OR", 24, 100, EvictionGrounds::AtFault, 250_000));
        assert!(!r.relocation_assistance_required);
        assert!(!r.small_landlord_exempt_from_relocation);
    }

    #[test]
    fn relocation_zero_when_zero_rent_input() {
        // 1 × $0 = $0 relocation. The math doesn't introduce a floor.
        let r = check(&input(
            "CA",
            12,
            100,
            EvictionGrounds::NoFaultOwnerMoveIn,
            0,
        ));
        assert_eq!(r.relocation_assistance_cents, 0);
        assert!(r.relocation_assistance_required);
    }

    #[test]
    fn relocation_no_overflow_at_maximum_rent() {
        // Huge rent input — saturating_mul should clamp at i64::MAX
        // rather than panic.
        let r = check(&input(
            "WA",
            12,
            100,
            EvictionGrounds::NoFaultOwnerMoveIn,
            i64::MAX / 2,
        ));
        // Relocation = 1 × monthly rent = same value as input.
        assert_eq!(r.relocation_assistance_cents, i64::MAX / 2);
    }
}
