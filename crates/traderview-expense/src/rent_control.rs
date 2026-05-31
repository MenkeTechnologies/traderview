//! State rent control / rent stabilization — 7th state-data module
//! after `deposit_interest`, `late_fee_caps`, `eviction_notices`,
//! `contractor_1099`, `deposit_return_windows`, and
//! `lease_disclosures`.
//!
//! Three classes of state law govern rent increases:
//!
//!   * **Statewide rent cap** — CA (AB1482, 2019), OR (SB608, 2019),
//!     WA (HB1217, 2024 effective 2025). Cap is typically a fixed
//!     percentage + local CPI, capped at 10%. Common exemptions:
//!     new construction (< 15 years old), single-family homes not
//!     corporate-owned, owner-occupied 2-4 unit buildings.
//!
//!   * **Local rent stabilization permitted** — NY, NJ, MD, MN, IL
//!     all permit municipalities to enact local stabilization
//!     (NYC, Saint Paul, Takoma Park, DC). The state-level rule
//!     here doesn't impose a cap; caller's responsibility to layer
//!     on the applicable local ordinance.
//!
//!   * **State preemption** — ~20 states have statutes that
//!     explicitly PROHIBIT local rent control (TX Loc. Gov't Code
//!     §214.902, FL Stat. §125.0103, AZ §33-1329, GA §44-7-19,
//!     TN §66-35-102, IN §32-31-1-21, IL Rent Control Preemption
//!     Act, KS, MS, NV, NM, NC, OK, SC, UT, WI, WY). In these
//!     states, rent is fully market-rate — caller can set any
//!     increase subject to the existing lease and notice rules.
//!
//! Pure data + compute. Caller passes the state code + property
//! facts (year_built, single_family_corporate_owned, current rent,
//! proposed new rent, tenancy_months); we return whether the
//! increase is compliant, the maximum allowed, and the statute
//! citation.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Citation {
    pub statute: &'static str,
    pub source: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapKind {
    /// Statewide statutory cap on annual rent increases.
    StatewideCap,
    /// Local stabilization permitted but no state cap (NY, NJ, etc.).
    /// Caller is responsible for local ordinance.
    LocalStabilizationPermitted,
    /// State law preempts local rent control. Rent fully market.
    StatewidePreemption,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateRule {
    pub state: &'static str,
    pub kind: CapKind,
    /// Fixed percentage cap (0.05 = 5%). Combined with CPI per the
    /// statutory formula. Zero when kind != StatewideCap.
    pub fixed_pct: Decimal,
    /// Absolute cap on the combined fixed + CPI. Most state laws cap
    /// at 10% regardless of CPI. Zero when not applicable.
    pub absolute_max_pct: Decimal,
    /// Minimum tenancy length (months) before the cap applies. CA and
    /// OR don't cap year-1 increases — the cap kicks in after 12mo.
    pub minimum_tenancy_months_for_cap: u32,
    /// New construction exemption: buildings newer than this many
    /// years are exempt. CA + OR both use 15.
    pub new_construction_exempt_under_years: u32,
    /// Single-family homes (and condos) owned by non-corporate entities
    /// are exempt under CA AB1482 + OR SB608.
    pub single_family_non_corporate_exempt: bool,
    /// Owner-occupied 2-4 unit buildings exempt.
    pub owner_occupied_2_to_4_unit_exempt: bool,
    /// Just-cause termination required AFTER first 12 months under
    /// CA AB1482 / OR SB608 / WA HB1217.
    pub just_cause_required_after_12mo: bool,
    pub effective_year: i32,
    pub notes: &'static str,
    pub citation: Citation,
}

fn d(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

fn rules() -> &'static [StateRule] {
    static R: once_cell::sync::Lazy<Vec<StateRule>> =
        once_cell::sync::Lazy::new(|| {
            vec![
                // ─── Statewide caps ───────────────────────────────
                StateRule {
                    state: "CA",
                    kind: CapKind::StatewideCap,
                    fixed_pct: d("0.05"),
                    absolute_max_pct: d("0.10"),
                    minimum_tenancy_months_for_cap: 12,
                    new_construction_exempt_under_years: 15,
                    single_family_non_corporate_exempt: true,
                    owner_occupied_2_to_4_unit_exempt: true,
                    just_cause_required_after_12mo: true,
                    effective_year: 2024,
                    notes: "AB1482 (Tenant Protection Act): 5% + local CPI annual cap, capped at 10%. Applies after 12mo tenancy. Exemptions: new construction <15yrs, single-family non-corporate, owner-occupied 2-4 unit. Just cause required after 12mo.",
                    citation: Citation {
                        statute: "Cal. Civ. Code §1946.2 + §1947.12",
                        source: "https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?sectionNum=1947.12.&lawCode=CIV",
                    },
                },
                StateRule {
                    state: "OR",
                    kind: CapKind::StatewideCap,
                    fixed_pct: d("0.07"),
                    absolute_max_pct: d("0.10"),
                    minimum_tenancy_months_for_cap: 12,
                    new_construction_exempt_under_years: 15,
                    single_family_non_corporate_exempt: false,
                    owner_occupied_2_to_4_unit_exempt: false,
                    just_cause_required_after_12mo: true,
                    effective_year: 2024,
                    notes: "SB608 (2019, first-in-nation statewide cap): 7% + CPI annual cap capped at 10%. Applies after 12mo. New construction <15yrs exempt. Just cause required after 12mo.",
                    citation: Citation {
                        statute: "ORS 90.323 + 90.427",
                        source: "https://oregon.public.law/statutes/ors_90.323",
                    },
                },
                StateRule {
                    state: "WA",
                    kind: CapKind::StatewideCap,
                    fixed_pct: d("0.07"),
                    absolute_max_pct: d("0.10"),
                    minimum_tenancy_months_for_cap: 12,
                    new_construction_exempt_under_years: 15,
                    single_family_non_corporate_exempt: false,
                    owner_occupied_2_to_4_unit_exempt: false,
                    just_cause_required_after_12mo: true,
                    effective_year: 2025,
                    notes: "HB1217 (2024, effective 2025): 7% + CPI annual cap with 10% absolute max. 5% during 2024-2025 transition. Just cause required statewide (existing 2021 act). New construction <15yrs exempt.",
                    citation: Citation {
                        statute: "RCW 59.18 (HB1217)",
                        source: "https://app.leg.wa.gov/billsummary?BillNumber=1217&Year=2023",
                    },
                },
                // ─── Local stabilization permitted ────────────────
                StateRule {
                    state: "NY",
                    kind: CapKind::LocalStabilizationPermitted,
                    fixed_pct: Decimal::ZERO,
                    absolute_max_pct: Decimal::ZERO,
                    minimum_tenancy_months_for_cap: 0,
                    new_construction_exempt_under_years: 0,
                    single_family_non_corporate_exempt: false,
                    owner_occupied_2_to_4_unit_exempt: false,
                    just_cause_required_after_12mo: false,
                    effective_year: 2024,
                    notes: "No statewide cap. NYC Rent Guidelines Board sets annual increases for ~1M stabilized units. ETPA permits Westchester/Nassau/Rockland municipalities to opt in. Caller responsible for local ordinance.",
                    citation: Citation {
                        statute: "NYC Rent Stabilization Law + ETPA",
                        source: "https://rentguidelinesboard.cityofnewyork.us/",
                    },
                },
                StateRule {
                    state: "NJ",
                    kind: CapKind::LocalStabilizationPermitted,
                    fixed_pct: Decimal::ZERO,
                    absolute_max_pct: Decimal::ZERO,
                    minimum_tenancy_months_for_cap: 0,
                    new_construction_exempt_under_years: 0,
                    single_family_non_corporate_exempt: false,
                    owner_occupied_2_to_4_unit_exempt: false,
                    just_cause_required_after_12mo: false,
                    effective_year: 2024,
                    notes: "Local rent control permitted in 100+ municipalities (Newark, Jersey City, Hoboken, Paterson, Elizabeth). Each sets its own cap formula. Caller responsible for local ordinance.",
                    citation: Citation {
                        statute: "N.J.S.A. various municipal codes",
                        source: "https://www.nj.gov/dca/codes/publications/",
                    },
                },
                StateRule {
                    state: "MD",
                    kind: CapKind::LocalStabilizationPermitted,
                    fixed_pct: Decimal::ZERO,
                    absolute_max_pct: Decimal::ZERO,
                    minimum_tenancy_months_for_cap: 0,
                    new_construction_exempt_under_years: 0,
                    single_family_non_corporate_exempt: false,
                    owner_occupied_2_to_4_unit_exempt: false,
                    just_cause_required_after_12mo: false,
                    effective_year: 2024,
                    notes: "Takoma Park has stabilization; Montgomery County considering. Most of MD market-rate.",
                    citation: Citation {
                        statute: "Md. Real Prop. + Takoma Park municipal code",
                        source: "https://takomaparkmd.gov/government/landlord-tenant-affairs/",
                    },
                },
                StateRule {
                    state: "MN",
                    kind: CapKind::LocalStabilizationPermitted,
                    fixed_pct: Decimal::ZERO,
                    absolute_max_pct: Decimal::ZERO,
                    minimum_tenancy_months_for_cap: 0,
                    new_construction_exempt_under_years: 0,
                    single_family_non_corporate_exempt: false,
                    owner_occupied_2_to_4_unit_exempt: false,
                    just_cause_required_after_12mo: false,
                    effective_year: 2024,
                    notes: "Saint Paul voters approved 3% annual cap (2021), tenants' bill of rights (2023). Minneapolis stabilization charter amendment pending. Rest of MN market-rate.",
                    citation: Citation {
                        statute: "Saint Paul Ord. (St. Paul Ch. 193A) + Minneapolis Charter Amendment",
                        source: "https://www.stpaul.gov/departments/safety-inspections/rent-stabilization",
                    },
                },
                StateRule {
                    state: "DC",
                    kind: CapKind::LocalStabilizationPermitted,
                    fixed_pct: Decimal::ZERO,
                    absolute_max_pct: Decimal::ZERO,
                    minimum_tenancy_months_for_cap: 0,
                    new_construction_exempt_under_years: 0,
                    single_family_non_corporate_exempt: false,
                    owner_occupied_2_to_4_unit_exempt: false,
                    just_cause_required_after_12mo: true,
                    effective_year: 2024,
                    notes: "Rental Housing Act of 1985: buildings 1975 and earlier subject to rent stabilization at CPI + 2% (8.9% absolute cap for 2024 per RAD). Just-cause termination required. Buildings post-1975 generally exempt.",
                    citation: Citation {
                        statute: "D.C. Code §42-3501 et seq.",
                        source: "https://code.dccouncil.us/dc/council/code/titles/42/chapters/35/",
                    },
                },
                // ─── State preemption (no local rent control allowed) ────
                StateRule {
                    state: "TX",
                    kind: CapKind::StatewidePreemption,
                    fixed_pct: Decimal::ZERO,
                    absolute_max_pct: Decimal::ZERO,
                    minimum_tenancy_months_for_cap: 0,
                    new_construction_exempt_under_years: 0,
                    single_family_non_corporate_exempt: false,
                    owner_occupied_2_to_4_unit_exempt: false,
                    just_cause_required_after_12mo: false,
                    effective_year: 2024,
                    notes: "Tex. Loc. Gov't Code §214.902 explicitly prohibits municipalities from enacting rent control. Rent fully market-rate; landlord must still honor existing lease terms.",
                    citation: Citation {
                        statute: "Tex. Loc. Gov't Code §214.902",
                        source: "https://statutes.capitol.texas.gov/Docs/LG/htm/LG.214.htm",
                    },
                },
                StateRule {
                    state: "FL",
                    kind: CapKind::StatewidePreemption,
                    fixed_pct: Decimal::ZERO,
                    absolute_max_pct: Decimal::ZERO,
                    minimum_tenancy_months_for_cap: 0,
                    new_construction_exempt_under_years: 0,
                    single_family_non_corporate_exempt: false,
                    owner_occupied_2_to_4_unit_exempt: false,
                    just_cause_required_after_12mo: false,
                    effective_year: 2024,
                    notes: "Fla. Stat. §125.0103 + §166.043: counties/cities cannot impose rent control absent housing emergency declaration. Tenant-friendly Local Option exceptions narrow.",
                    citation: Citation {
                        statute: "Fla. Stat. §125.0103 + §166.043",
                        source: "https://www.flsenate.gov/Laws/Statutes/2024/0125.0103",
                    },
                },
                StateRule {
                    state: "AZ",
                    kind: CapKind::StatewidePreemption,
                    fixed_pct: Decimal::ZERO,
                    absolute_max_pct: Decimal::ZERO,
                    minimum_tenancy_months_for_cap: 0,
                    new_construction_exempt_under_years: 0,
                    single_family_non_corporate_exempt: false,
                    owner_occupied_2_to_4_unit_exempt: false,
                    just_cause_required_after_12mo: false,
                    effective_year: 2024,
                    notes: "A.R.S. §33-1329: state preempts all local rent control. Rent fully market-rate.",
                    citation: Citation {
                        statute: "A.R.S. §33-1329",
                        source: "https://www.azleg.gov/ars/33/01329.htm",
                    },
                },
                StateRule {
                    state: "GA",
                    kind: CapKind::StatewidePreemption,
                    fixed_pct: Decimal::ZERO,
                    absolute_max_pct: Decimal::ZERO,
                    minimum_tenancy_months_for_cap: 0,
                    new_construction_exempt_under_years: 0,
                    single_family_non_corporate_exempt: false,
                    owner_occupied_2_to_4_unit_exempt: false,
                    just_cause_required_after_12mo: false,
                    effective_year: 2024,
                    notes: "O.C.G.A. §44-7-19: state law preempts municipal rent control.",
                    citation: Citation {
                        statute: "O.C.G.A. §44-7-19",
                        source: "https://law.justia.com/codes/georgia/2022/title-44/chapter-7/article-1/section-44-7-19/",
                    },
                },
                StateRule {
                    state: "TN",
                    kind: CapKind::StatewidePreemption,
                    fixed_pct: Decimal::ZERO,
                    absolute_max_pct: Decimal::ZERO,
                    minimum_tenancy_months_for_cap: 0,
                    new_construction_exempt_under_years: 0,
                    single_family_non_corporate_exempt: false,
                    owner_occupied_2_to_4_unit_exempt: false,
                    just_cause_required_after_12mo: false,
                    effective_year: 2024,
                    notes: "Tenn. Code §66-35-102: state preempts local rent control.",
                    citation: Citation {
                        statute: "Tenn. Code Ann. §66-35-102",
                        source: "https://law.justia.com/codes/tennessee/2021/title-66/chapter-35/section-66-35-102/",
                    },
                },
                StateRule {
                    state: "IL",
                    kind: CapKind::StatewidePreemption,
                    fixed_pct: Decimal::ZERO,
                    absolute_max_pct: Decimal::ZERO,
                    minimum_tenancy_months_for_cap: 0,
                    new_construction_exempt_under_years: 0,
                    single_family_non_corporate_exempt: false,
                    owner_occupied_2_to_4_unit_exempt: false,
                    just_cause_required_after_12mo: false,
                    effective_year: 2024,
                    notes: "Illinois Rent Control Preemption Act (50 ILCS 825): preempts local rent control. Chicago RLTO exists but cannot cap rents.",
                    citation: Citation {
                        statute: "50 ILCS 825",
                        source: "https://www.ilga.gov/legislation/ilcs/ilcs3.asp?ActID=826",
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RentIncreaseCheckInput {
    pub state: String,
    pub current_rent: Decimal,
    pub proposed_new_rent: Decimal,
    pub tenancy_months: u32,
    /// Property year built — drives the new-construction exemption.
    /// `None` = year unknown; we assume not new-construction.
    pub year_built: Option<u32>,
    /// Property is a single-family home or condo owned by a
    /// non-corporate landlord. Exempt under CA AB1482 §1947.12(d)(5)
    /// + OR SB608.
    pub single_family_non_corporate: bool,
    /// Owner-occupied 2-4 unit building. Exempt under CA AB1482
    /// §1947.12(d)(7).
    pub owner_occupied_2_to_4_unit: bool,
    /// Local CPI percentage for the cap formula (e.g. 0.03 = 3%
    /// CPI). Only used for statewide-cap states. Caller supplies
    /// from BLS regional CPI data — we don't fetch.
    pub local_cpi_pct: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RentIncreaseCheckResult {
    pub state_recognized: bool,
    pub cap_kind: Option<CapKind>,
    /// Maximum permitted percentage increase under the cap formula.
    /// Zero for preempted states (no statewide cap; caller can set
    /// any increase). Also zero when an exemption applies.
    pub max_permitted_pct: Decimal,
    /// Maximum permitted new rent given current rent.
    pub max_permitted_new_rent: Decimal,
    /// Actual proposed increase as a percentage.
    pub proposed_increase_pct: Decimal,
    pub compliant: bool,
    pub exemption_applied: Option<&'static str>,
    pub just_cause_required: bool,
    pub statute: String,
    pub source: String,
    pub notes: String,
}

pub fn check(input: &RentIncreaseCheckInput) -> RentIncreaseCheckResult {
    let rule = match rule_for(&input.state) {
        Some(r) => r,
        None => {
            return RentIncreaseCheckResult {
                state_recognized: false,
                cap_kind: None,
                max_permitted_pct: Decimal::ZERO,
                max_permitted_new_rent: input.proposed_new_rent,
                proposed_increase_pct: Decimal::ZERO,
                compliant: true,
                exemption_applied: None,
                just_cause_required: false,
                statute: String::new(),
                source: String::new(),
                notes: format!(
                    "no rent-control statute on file for {} — caller should verify state code directly",
                    input.state.to_uppercase()
                ),
            };
        }
    };

    let proposed_pct = if input.current_rent > Decimal::ZERO {
        ((input.proposed_new_rent - input.current_rent) / input.current_rent)
            .round_dp(4)
    } else {
        Decimal::ZERO
    };

    let mut r = RentIncreaseCheckResult {
        state_recognized: true,
        cap_kind: Some(rule.kind),
        max_permitted_pct: Decimal::ZERO,
        max_permitted_new_rent: input.proposed_new_rent,
        proposed_increase_pct: proposed_pct,
        compliant: true,
        exemption_applied: None,
        just_cause_required: rule.just_cause_required_after_12mo
            && input.tenancy_months >= 12,
        statute: rule.citation.statute.into(),
        source: rule.citation.source.into(),
        notes: rule.notes.into(),
    };

    if !matches!(rule.kind, CapKind::StatewideCap) {
        // Preemption or local-stabilization states: no statewide cap
        // computation. Caller responsible for local ordinance.
        return r;
    }

    // Statewide cap analysis. Check exemptions first.
    if input.single_family_non_corporate && rule.single_family_non_corporate_exempt {
        r.exemption_applied = Some("single-family non-corporate owner");
        return r;
    }
    if input.owner_occupied_2_to_4_unit && rule.owner_occupied_2_to_4_unit_exempt {
        r.exemption_applied = Some("owner-occupied 2-4 unit building");
        return r;
    }
    if let Some(year) = input.year_built {
        let now = 2024u32; // approximate; caller can correct
        let age = now.saturating_sub(year);
        if age < rule.new_construction_exempt_under_years {
            r.exemption_applied = Some("new construction (< 15 years old)");
            return r;
        }
    }
    if input.tenancy_months < rule.minimum_tenancy_months_for_cap {
        r.exemption_applied = Some("tenancy < 12 months (cap doesn't kick in until renewal)");
        return r;
    }

    // Apply the cap: fixed_pct + CPI, absolute max ceiling.
    let raw_cap = rule.fixed_pct + input.local_cpi_pct.max(Decimal::ZERO);
    r.max_permitted_pct = raw_cap.min(rule.absolute_max_pct);
    r.max_permitted_new_rent =
        (input.current_rent * (Decimal::ONE + r.max_permitted_pct)).round_dp(2);
    r.compliant = proposed_pct <= r.max_permitted_pct;

    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> RentIncreaseCheckInput {
        RentIncreaseCheckInput {
            state: "CA".into(),
            current_rent: dec!(2000),
            proposed_new_rent: dec!(2100),
            tenancy_months: 24,
            year_built: Some(2000),
            single_family_non_corporate: false,
            owner_occupied_2_to_4_unit: false,
            local_cpi_pct: dec!(0.03),
        }
    }

    #[test]
    fn ca_5pct_5pct_increase_compliant() {
        // CA: 5% + 3% CPI = 8% cap. Proposed 5% = compliant.
        let r = check(&base());
        assert_eq!(r.max_permitted_pct, dec!(0.08));
        assert!(r.compliant);
    }

    #[test]
    fn ca_increase_at_cap_compliant() {
        let mut i = base();
        i.proposed_new_rent = dec!(2160); // exactly 8% increase
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn ca_increase_over_cap_not_compliant() {
        let mut i = base();
        i.proposed_new_rent = dec!(2200); // 10% > 8% cap
        let r = check(&i);
        assert!(!r.compliant);
    }

    #[test]
    fn ca_high_cpi_caps_at_10pct_absolute() {
        // 5% + 8% CPI = 13% raw, capped at 10% absolute.
        let mut i = base();
        i.local_cpi_pct = dec!(0.08);
        let r = check(&i);
        assert_eq!(r.max_permitted_pct, dec!(0.10));
    }

    #[test]
    fn ca_single_family_non_corporate_exempt() {
        let mut i = base();
        i.single_family_non_corporate = true;
        i.proposed_new_rent = dec!(3000); // huge increase
        let r = check(&i);
        assert!(r.compliant); // exempt
        assert_eq!(r.exemption_applied, Some("single-family non-corporate owner"));
    }

    #[test]
    fn ca_owner_occupied_2_to_4_unit_exempt() {
        let mut i = base();
        i.owner_occupied_2_to_4_unit = true;
        i.proposed_new_rent = dec!(5000);
        let r = check(&i);
        assert!(r.compliant);
        assert_eq!(r.exemption_applied, Some("owner-occupied 2-4 unit building"));
    }

    #[test]
    fn ca_new_construction_under_15_years_exempt() {
        let mut i = base();
        i.year_built = Some(2020); // age 4 as of 2024
        i.proposed_new_rent = dec!(3000);
        let r = check(&i);
        assert!(r.compliant);
        assert_eq!(r.exemption_applied, Some("new construction (< 15 years old)"));
    }

    #[test]
    fn ca_old_construction_not_exempt() {
        let mut i = base();
        i.year_built = Some(1990); // age 34 — cap applies
        let r = check(&i);
        assert!(r.exemption_applied.is_none());
    }

    #[test]
    fn ca_short_tenancy_under_12mo_exempt_from_cap() {
        let mut i = base();
        i.tenancy_months = 6;
        i.proposed_new_rent = dec!(3000);
        let r = check(&i);
        assert!(r.compliant);
        assert_eq!(r.exemption_applied, Some("tenancy < 12 months (cap doesn't kick in until renewal)"));
    }

    #[test]
    fn or_7pct_cpi_3pct_yields_10pct_cap_max() {
        let mut i = base();
        i.state = "OR".into();
        let r = check(&i);
        // 7% + 3% CPI = 10% (at absolute max).
        assert_eq!(r.max_permitted_pct, dec!(0.10));
    }

    #[test]
    fn or_single_family_not_exempt_unlike_ca() {
        // OR SB608 doesn't exempt single-family non-corporate (only
        // new construction). The flag should be ignored.
        let mut i = base();
        i.state = "OR".into();
        i.single_family_non_corporate = true;
        i.proposed_new_rent = dec!(2300); // 15% — over cap
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.exemption_applied.is_none());
    }

    #[test]
    fn wa_7pct_plus_cpi_capped_at_10() {
        let mut i = base();
        i.state = "WA".into();
        let r = check(&i);
        assert_eq!(r.max_permitted_pct, dec!(0.10));
    }

    #[test]
    fn tx_preemption_no_cap_any_increase_compliant() {
        let mut i = base();
        i.state = "TX".into();
        i.proposed_new_rent = dec!(5000); // 150% increase
        let r = check(&i);
        assert!(r.compliant);
        assert_eq!(r.cap_kind, Some(CapKind::StatewidePreemption));
    }

    #[test]
    fn fl_preemption_no_cap() {
        let mut i = base();
        i.state = "FL".into();
        i.proposed_new_rent = dec!(4000);
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn ny_local_stabilization_permitted_state_no_cap_check() {
        // NY: no statewide cap. Module returns compliant=true with
        // LocalStabilizationPermitted; caller responsible for NYC RGB.
        let mut i = base();
        i.state = "NY".into();
        i.proposed_new_rent = dec!(2500);
        let r = check(&i);
        assert!(r.compliant);
        assert_eq!(r.cap_kind, Some(CapKind::LocalStabilizationPermitted));
    }

    #[test]
    fn just_cause_required_flag_set_for_ca_after_12mo() {
        let r = check(&base());
        assert!(r.just_cause_required);
    }

    #[test]
    fn just_cause_not_required_under_12mo_tenancy() {
        let mut i = base();
        i.tenancy_months = 6;
        let r = check(&i);
        assert!(!r.just_cause_required);
    }

    #[test]
    fn just_cause_not_required_in_preemption_states() {
        let mut i = base();
        i.state = "TX".into();
        let r = check(&i);
        assert!(!r.just_cause_required);
    }

    #[test]
    fn unknown_state_returns_not_recognized() {
        let mut i = base();
        i.state = "XX".into();
        let r = check(&i);
        assert!(!r.state_recognized);
        assert!(r.notes.contains("no rent-control statute"));
    }

    #[test]
    fn case_insensitive_state_lookup() {
        let mut i = base();
        i.state = "ca".into();
        let r = check(&i);
        assert!(r.state_recognized);
    }

    #[test]
    fn proposed_increase_pct_calculated_correctly() {
        let mut i = base();
        i.proposed_new_rent = dec!(2200); // 10% increase
        let r = check(&i);
        assert_eq!(r.proposed_increase_pct, dec!(0.10));
    }

    #[test]
    fn ca_exemption_priority_single_family_beats_construction() {
        // Single-family flag should win over year_built check.
        let mut i = base();
        i.single_family_non_corporate = true;
        i.year_built = Some(2000); // old enough to not be new construction
        let r = check(&i);
        assert_eq!(r.exemption_applied, Some("single-family non-corporate owner"));
    }

    #[test]
    fn dc_local_stabilization_just_cause_required() {
        let mut i = base();
        i.state = "DC".into();
        let r = check(&i);
        assert_eq!(r.cap_kind, Some(CapKind::LocalStabilizationPermitted));
        assert!(r.just_cause_required);
    }

    #[test]
    fn rule_for_returns_citation_for_known_states() {
        let r = rule_for("CA").unwrap();
        assert!(r.citation.statute.contains("1947.12"));
        let r = rule_for("OR").unwrap();
        assert!(r.citation.statute.contains("90.323"));
        let r = rule_for("TX").unwrap();
        assert!(r.citation.statute.contains("214.902"));
    }
}
