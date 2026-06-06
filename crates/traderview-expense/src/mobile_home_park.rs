//! State mobile home park / manufactured housing community
//! landlord-tenant compliance.
//!
//! Mobile home park (MHP) investing is a distinct asset class with
//! its own state regulatory regime. Several states have enacted
//! special MHP-specific statutes that displace the generic landlord-
//! tenant act. The patterns vary in three key axes: (1) does the
//! state cap rent increases? (2) is statutory advance notice
//! required? (3) is just-cause eviction required?
//!
//! Three regimes:
//!
//! - `JustCauseWithRentCap` — CA (Civ. Code § 798 Mobilehome
//!   Residency Law) and OR (ORS Ch. 90 + SB 608). 90-day rent
//!   increase notice + just-cause eviction. OR additionally caps
//!   annual rent increases at 7% + CPI with a maximum 10%; CA
//!   caps via local rent-control ordinances under § 798.17 except
//!   for long-term leases that elect out.
//!
//! - `NoticeAndJustCauseNoCap` — FL (Ch. 723 Mobile Home Act,
//!   applies to parks of 10+ lots) and WA (RCW 59.20
//!   Manufactured/Mobile Home Landlord-Tenant Act, with 2025
//!   amendments adding RCW 59.20.370 rent increase caps in some
//!   covered scenarios). 90-day rent increase notice required +
//!   just-cause eviction. No general statewide rent cap (FL) or
//!   limited cap with carveouts (WA 2025).
//!
//! - `GenericLandlordTenantLaw` — most other states. No MHP-specific
//!   statute; standard residential landlord-tenant act governs.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MhpRegime {
    JustCauseWithRentCap,
    NoticeAndJustCauseNoCap,
    GenericLandlordTenantLaw,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: MhpRegime,
    pub rent_increase_notice_days: u32,
    /// Annual rent increase cap in basis points (700 = 7%, 1000 = 10%).
    /// `None` when no statewide cap.
    pub rent_increase_cap_bp: Option<u32>,
    pub just_cause_required_for_termination: bool,
    /// Minimum park-lot count for the MHP statute to apply (FL = 10).
    pub minimum_lot_count_for_statute: u32,
    pub citation: &'static str,
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    regime: MhpRegime,
    rent_increase_notice_days: u32,
    rent_increase_cap_bp: Option<u32>,
    just_cause_required_for_termination: bool,
    minimum_lot_count_for_statute: u32,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        rent_increase_notice_days,
        rent_increase_cap_bp,
        just_cause_required_for_termination,
        minimum_lot_count_for_statute,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use MhpRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "CA",
        rule(
            JustCauseWithRentCap,
            90,
            None, // Statewide rent control via local ordinances under §798.17
            true,
            2,
            "Cal. Civ. Code § 798 Mobilehome Residency Law — 90-day rent increase notice; just-cause eviction; local rent control under § 798.17 (long-term lease exempt)",
        ),
    );
    m.insert(
        "OR",
        rule(
            JustCauseWithRentCap,
            90,
            Some(1000), // 7% + CPI capped at 10% under SB 608
            true,
            2,
            "Or. ORS Ch. 90 + SB 608 of 2019 — 90-day rent increase notice; 7% + CPI rent cap (10% maximum); just-cause eviction after first year",
        ),
    );

    m.insert(
        "FL",
        rule(
            NoticeAndJustCauseNoCap,
            90,
            None,
            true,
            10,
            "Fla. Stat. Ch. 723 Mobile Home Act — applies to parks with 10+ lots; 90-day rent increase notice; just-cause eviction; homeowner meeting right on rent increase",
        ),
    );
    m.insert(
        "WA",
        rule(
            NoticeAndJustCauseNoCap,
            90,
            None,
            true,
            2,
            "Wash. RCW 59.20 Manufactured/Mobile Home Landlord-Tenant Act + RCW 59.20.370 (eff. 2025-05-07 rent increase caps in covered scenarios + penalties); just-cause eviction",
        ),
    );

    // GenericLandlordTenantLaw — all other states + DC.
    let generic_states = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DC", "DE", "GA", "HI", "ID", "IL", "IN", "IA", "KS",
        "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ", "NM",
        "NY", "NC", "ND", "OH", "OK", "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT", "VA", "WV",
        "WI", "WY",
    ];
    for code in generic_states {
        m.insert(
            code,
            rule(
                GenericLandlordTenantLaw,
                30,
                None,
                false,
                2,
                "No MHP-specific statute; generic state residential landlord-tenant act applies (typically 30-day notice + contract-governed termination)",
            ),
        );
    }
    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MhpInput {
    pub state_code: String,
    pub park_lot_count: u32,
    pub proposed_rent_increase_pct_bp: u32,
    pub written_notice_days_given: u32,
    pub termination_is_just_cause: bool,
    pub landlord_initiating_termination: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MhpResult {
    pub regime: MhpRegime,
    pub subject_to_mhp_statute: bool,
    pub rent_increase_notice_required_days: u32,
    pub rent_increase_notice_compliant: bool,
    pub rent_increase_cap_bp: Option<u32>,
    pub rent_increase_within_cap: bool,
    pub just_cause_required: bool,
    pub termination_compliant: bool,
    pub overall_compliant: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &MhpInput) -> MhpResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: MhpRegime::GenericLandlordTenantLaw,
        rent_increase_notice_days: 30,
        rent_increase_cap_bp: None,
        just_cause_required_for_termination: false,
        minimum_lot_count_for_statute: 2,
        citation: "Unknown state code; assuming generic landlord-tenant act",
    });

    let subject = input.park_lot_count >= rule.minimum_lot_count_for_statute
        && rule.regime != MhpRegime::GenericLandlordTenantLaw;

    let notice_compliant = if subject {
        input.written_notice_days_given >= rule.rent_increase_notice_days
    } else {
        true
    };

    let within_cap = match rule.rent_increase_cap_bp {
        Some(cap) if subject => input.proposed_rent_increase_pct_bp <= cap,
        _ => true,
    };

    let termination_ok = if subject
        && rule.just_cause_required_for_termination
        && input.landlord_initiating_termination
    {
        input.termination_is_just_cause
    } else {
        true
    };

    let overall = notice_compliant && within_cap && termination_ok;

    let note = match (rule.regime, subject) {
        (MhpRegime::GenericLandlordTenantLaw, _) =>
            "GenericLandlordTenantLaw: no MHP-specific statute; standard residential landlord-tenant act applies.".to_string(),
        (_, false) => format!(
            "{:?}: park has {} lots; below the {}-lot minimum for state MHP statute; generic landlord-tenant law applies.",
            rule.regime, input.park_lot_count, rule.minimum_lot_count_for_statute,
        ),
        (MhpRegime::JustCauseWithRentCap, true) => format!(
            "JustCauseWithRentCap: {}-day notice {} ({} given); rent increase {}.{}% {}; termination {}.",
            rule.rent_increase_notice_days,
            if notice_compliant { "satisfied" } else { "INSUFFICIENT" },
            input.written_notice_days_given,
            input.proposed_rent_increase_pct_bp / 100,
            input.proposed_rent_increase_pct_bp % 100,
            if within_cap { "within cap" } else { "EXCEEDS CAP" },
            if termination_ok { "compliant" } else { "FAILS just-cause" },
        ),
        (MhpRegime::NoticeAndJustCauseNoCap, true) => format!(
            "NoticeAndJustCauseNoCap: {}-day notice {} ({} given); no statewide rent cap; termination {}.",
            rule.rent_increase_notice_days,
            if notice_compliant { "satisfied" } else { "INSUFFICIENT" },
            input.written_notice_days_given,
            if termination_ok { "compliant" } else { "FAILS just-cause" },
        ),
    };

    MhpResult {
        regime: rule.regime,
        subject_to_mhp_statute: subject,
        rent_increase_notice_required_days: rule.rent_increase_notice_days,
        rent_increase_notice_compliant: notice_compliant,
        rent_increase_cap_bp: rule.rent_increase_cap_bp,
        rent_increase_within_cap: within_cap,
        just_cause_required: subject && rule.just_cause_required_for_termination,
        termination_compliant: termination_ok,
        overall_compliant: overall,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: &str, lots: u32) -> MhpInput {
        MhpInput {
            state_code: state.to_string(),
            park_lot_count: lots,
            proposed_rent_increase_pct_bp: 500, // 5%
            written_notice_days_given: 90,
            termination_is_just_cause: false,
            landlord_initiating_termination: false,
        }
    }

    // CA — JustCauseWithRentCap.

    #[test]
    fn ca_90_day_notice_complies() {
        let r = check(&input("CA", 50));
        assert_eq!(r.regime, MhpRegime::JustCauseWithRentCap);
        assert!(r.subject_to_mhp_statute);
        assert!(r.rent_increase_notice_compliant);
        assert!(r.overall_compliant);
    }

    #[test]
    fn ca_89_day_notice_insufficient() {
        let mut i = input("CA", 50);
        i.written_notice_days_given = 89;
        let r = check(&i);
        assert!(!r.rent_increase_notice_compliant);
        assert!(!r.overall_compliant);
    }

    #[test]
    fn ca_landlord_termination_requires_just_cause() {
        let mut i = input("CA", 50);
        i.landlord_initiating_termination = true;
        i.termination_is_just_cause = false;
        let r = check(&i);
        assert!(r.just_cause_required);
        assert!(!r.termination_compliant);
        assert!(!r.overall_compliant);
    }

    #[test]
    fn ca_landlord_just_cause_termination_complies() {
        let mut i = input("CA", 50);
        i.landlord_initiating_termination = true;
        i.termination_is_just_cause = true;
        let r = check(&i);
        assert!(r.overall_compliant);
    }

    // OR — JustCauseWithRentCap + 10% cap.

    #[test]
    fn or_within_10_pct_cap_complies() {
        let mut i = input("OR", 50);
        i.proposed_rent_increase_pct_bp = 700; // 7%
        let r = check(&i);
        assert!(r.rent_increase_within_cap);
    }

    #[test]
    fn or_exact_10_pct_cap_complies() {
        let mut i = input("OR", 50);
        i.proposed_rent_increase_pct_bp = 1000; // 10% exact
        let r = check(&i);
        assert!(r.rent_increase_within_cap);
    }

    #[test]
    fn or_above_10_pct_violates() {
        let mut i = input("OR", 50);
        i.proposed_rent_increase_pct_bp = 1100; // 11%
        let r = check(&i);
        assert!(!r.rent_increase_within_cap);
        assert!(!r.overall_compliant);
    }

    // FL — Ch. 723 with 10-lot threshold.

    #[test]
    fn fl_small_park_under_10_lots_not_subject() {
        let r = check(&input("FL", 9));
        assert!(!r.subject_to_mhp_statute);
        assert!(r.note.contains("below the 10-lot minimum"));
    }

    #[test]
    fn fl_park_10_lots_exact_boundary_subject() {
        let r = check(&input("FL", 10));
        assert!(r.subject_to_mhp_statute);
    }

    #[test]
    fn fl_park_50_lots_subject() {
        let r = check(&input("FL", 50));
        assert_eq!(r.regime, MhpRegime::NoticeAndJustCauseNoCap);
        assert!(r.subject_to_mhp_statute);
    }

    #[test]
    fn fl_no_statewide_rent_cap() {
        let mut i = input("FL", 50);
        i.proposed_rent_increase_pct_bp = 5000; // 50%
        let r = check(&i);
        assert!(r.rent_increase_within_cap);
        assert_eq!(r.rent_increase_cap_bp, None);
    }

    // WA.

    #[test]
    fn wa_90_day_notice_required() {
        let r = check(&input("WA", 20));
        assert_eq!(r.regime, MhpRegime::NoticeAndJustCauseNoCap);
        assert!(r.subject_to_mhp_statute);
        assert!(r.rent_increase_notice_compliant);
    }

    #[test]
    fn wa_just_cause_termination_required() {
        let mut i = input("WA", 20);
        i.landlord_initiating_termination = true;
        i.termination_is_just_cause = false;
        let r = check(&i);
        assert!(!r.termination_compliant);
    }

    // Generic landlord-tenant states.

    #[test]
    fn tx_generic_law_applies_no_mhp_statute() {
        let r = check(&input("TX", 100));
        assert_eq!(r.regime, MhpRegime::GenericLandlordTenantLaw);
        assert!(!r.subject_to_mhp_statute);
    }

    #[test]
    fn ny_generic_law_no_just_cause_required() {
        let r = check(&input("NY", 50));
        assert!(!r.just_cause_required);
    }

    // Coverage / invariants.

    #[test]
    fn coverage_is_all_50_states_plus_dc() {
        let codes: Vec<&'static str> = RULES.keys().copied().collect();
        assert_eq!(
            codes.len(),
            51,
            "expected 50 states + DC, got {}",
            codes.len()
        );
    }

    #[test]
    fn citations_never_empty() {
        for (code, rule) in RULES.iter() {
            assert!(!rule.citation.is_empty(), "{code} missing citation");
        }
    }

    #[test]
    fn just_cause_with_rent_cap_only_ca_and_or() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == MhpRegime::JustCauseWithRentCap {
                count += 1;
            }
        }
        assert_eq!(count, 2, "expected CA + OR only with JustCauseWithRentCap");
    }

    #[test]
    fn notice_and_just_cause_only_fl_and_wa() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.regime == MhpRegime::NoticeAndJustCauseNoCap {
                count += 1;
            }
        }
        assert_eq!(
            count, 2,
            "expected FL + WA only with NoticeAndJustCauseNoCap"
        );
    }

    #[test]
    fn only_or_has_statewide_rent_cap() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.rent_increase_cap_bp.is_some() {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected OR only with statewide rent cap");
    }

    #[test]
    fn only_fl_has_10_lot_threshold() {
        let mut count = 0;
        for rule in RULES.values() {
            if rule.minimum_lot_count_for_statute == 10 {
                count += 1;
            }
        }
        assert_eq!(count, 1, "expected FL only with 10-lot threshold");
    }

    #[test]
    fn unknown_state_falls_back_to_generic() {
        let r = check(&input("XX", 100));
        assert_eq!(r.regime, MhpRegime::GenericLandlordTenantLaw);
    }

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&input("ca", 50));
        assert!(r.subject_to_mhp_statute);
    }

    // Notes.

    #[test]
    fn ca_note_describes_just_cause_with_rent_cap() {
        let r = check(&input("CA", 50));
        assert!(r.note.contains("JustCauseWithRentCap"));
    }

    #[test]
    fn fl_note_describes_no_statewide_cap() {
        let r = check(&input("FL", 50));
        assert!(r.note.contains("no statewide rent cap"));
    }

    #[test]
    fn fl_small_park_note_mentions_lot_threshold() {
        let r = check(&input("FL", 5));
        assert!(r.note.contains("below the 10-lot minimum"));
    }
}
