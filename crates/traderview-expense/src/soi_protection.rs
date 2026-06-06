//! State-by-state **source-of-income (SOI) discrimination** protection
//! for housing voucher holders.
//!
//! The **federal Fair Housing Act does NOT cover source of income**
//! as a protected class — landlords may refuse Section 8 Housing Choice
//! Voucher (HCV) holders nationwide unless a state or local law
//! provides protection. Approximately 18-20 states + DC + many cities
//! have enacted SOI protections since CT (1989) became the first.
//!
//! Three regimes:
//!
//! 1. **Statewide full protection** — landlord may NOT refuse a voucher.
//!    CT (1989), MA (1989), VT (1989), RI (1996), NJ (2002), ME (2009),
//!    OR (2014, first post-2010 wave), WA (2018), CA (SB 329 2019),
//!    CO (2020), DE (2020), MD (2020), VA (2020), IL (2023 statewide),
//!    MN (2023), NM (2023), DC.
//!
//! 2. **Statewide protection but legally challenged / unstable** —
//!    NY HSTPA 2019 statewide SOI protection was struck down by an
//!    appellate court in March 2026 (per news coverage). NYC Human
//!    Rights Law still applies; statewide enforcement is contested.
//!    Caller should treat NY as "check current court status + locality".
//!
//! 3. **No statewide protection** — landlord may refuse vouchers
//!    subject only to fair-housing rules on race, religion, sex, etc.
//!    Federal floor; local ordinances in some cities (Atlanta, Austin,
//!    Memphis, Miami-Dade, Philadelphia, Pittsburgh, others) provide
//!    municipal-level SOI protection that this module does not enumerate.
//!
//! UT has a limited SOI provision under § 13-21-302 covering veterans
//! but not all voucher holders — classified as `StatewidePartialOrChallenged`
//! to flag for caller review.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum SoiProtectionRegime {
    /// Full statewide protection; landlord cannot refuse on SOI basis.
    StatewideFull,
    /// Statewide protection exists but is legally challenged, partial,
    /// or limited to specific voucher types. Caller must verify the
    /// current legal status and any applicable local ordinances.
    StatewidePartialOrChallenged,
    /// No statewide protection. Federal FHA does not cover SOI.
    /// Landlord may refuse vouchers (subject to other fair-housing rules).
    NoProtection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSoiRule {
    pub state_code: &'static str,
    pub state_name: &'static str,
    pub regime: SoiProtectionRegime,
    /// Year the statewide statute was enacted. `None` for no-protection
    /// states.
    pub year_enacted: Option<u32>,
    pub citation: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VoucherType {
    /// HUD Housing Choice Voucher (Section 8) — the canonical case.
    Section8Hcv,
    /// HUD-VASH (Veterans Affairs Supportive Housing).
    Vash,
    /// Family Unification Program.
    Fup,
    /// Other federal or state subsidy voucher.
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoiCheckInput {
    pub state_code: String,
    pub voucher_type: VoucherType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoiCheckResult {
    /// True if the landlord may legally refuse the voucher.
    pub landlord_may_refuse: bool,
    pub protection_regime: SoiProtectionRegime,
    /// True if only the federal floor (no statewide statute) applies.
    pub federal_floor_only: bool,
    /// True if state law exists but is legally challenged / partial /
    /// limited — caller should verify current status.
    pub verify_current_status_needed: bool,
    pub year_enacted: Option<u32>,
    pub citation: &'static str,
    pub note: String,
}

pub fn lookup(state_code: &str) -> Option<&'static StateSoiRule> {
    let up = state_code.to_ascii_uppercase();
    TABLE.get(up.as_str()).copied()
}

pub fn all_states() -> Vec<&'static StateSoiRule> {
    let mut v: Vec<&'static StateSoiRule> = TABLE.values().copied().collect();
    v.sort_by_key(|r| r.state_code);
    v
}

pub fn check(input: &SoiCheckInput) -> SoiCheckResult {
    let rule = match lookup(&input.state_code) {
        Some(r) => r,
        None => {
            return SoiCheckResult {
                landlord_may_refuse: true,
                protection_regime: SoiProtectionRegime::NoProtection,
                federal_floor_only: true,
                verify_current_status_needed: false,
                year_enacted: None,
                citation: "n/a",
                note: format!(
                    "unknown state code `{}` — defaulting to federal floor (no SOI protection)",
                    input.state_code
                ),
            };
        }
    };

    let (may_refuse, federal_floor, verify_needed, note) = match rule.regime {
        SoiProtectionRegime::StatewideFull => (
            false,
            false,
            false,
            format!(
                "{} provides full statewide source-of-income protection (enacted {}); landlord may NOT refuse vouchers",
                rule.state_name,
                rule.year_enacted.unwrap_or(0)
            ),
        ),
        SoiProtectionRegime::StatewidePartialOrChallenged => (
            false,
            false,
            true,
            format!(
                "{} statewide SOI law exists but is legally challenged / partial / limited — verify current court status and local ordinances",
                rule.state_name
            ),
        ),
        SoiProtectionRegime::NoProtection => (
            true,
            true,
            false,
            format!(
                "{} has no statewide SOI protection — federal FHA does not cover source of income; landlord may refuse vouchers (subject to local ordinances)",
                rule.state_name
            ),
        ),
    };

    SoiCheckResult {
        landlord_may_refuse: may_refuse,
        protection_regime: rule.regime,
        federal_floor_only: federal_floor,
        verify_current_status_needed: verify_needed,
        year_enacted: rule.year_enacted,
        citation: rule.citation,
        note,
    }
}

const fn rule(
    state_code: &'static str,
    state_name: &'static str,
    regime: SoiProtectionRegime,
    year_enacted: Option<u32>,
    citation: &'static str,
) -> StateSoiRule {
    StateSoiRule {
        state_code,
        state_name,
        regime,
        year_enacted,
        citation,
    }
}

static TABLE: Lazy<HashMap<&'static str, &'static StateSoiRule>> = Lazy::new(|| {
    use SoiProtectionRegime::*;
    static RULES: &[StateSoiRule] = &[
        rule("AK", "Alaska", NoProtection, None, "no statewide statute"),
        rule("AL", "Alabama", NoProtection, None, "no statewide statute"),
        rule("AR", "Arkansas", NoProtection, None, "no statewide statute"),
        rule("AZ", "Arizona", NoProtection, None, "no statewide statute"),
        rule(
            "CA",
            "California",
            StatewideFull,
            Some(2019),
            "Cal. Gov. Code § 12955 (SB 329)",
        ),
        rule(
            "CO",
            "Colorado",
            StatewideFull,
            Some(2020),
            "C.R.S. § 24-34-502.2",
        ),
        rule(
            "CT",
            "Connecticut",
            StatewideFull,
            Some(1989),
            "Conn. Gen. Stat. § 46a-64c (oldest statewide)",
        ),
        rule(
            "DC",
            "District of Columbia",
            StatewideFull,
            Some(1977),
            "D.C. Code § 2-1402.21",
        ),
        rule(
            "DE",
            "Delaware",
            StatewideFull,
            Some(2020),
            "6 Del. C. § 4604",
        ),
        rule("FL", "Florida", NoProtection, None, "no statewide statute"),
        rule(
            "GA",
            "Georgia",
            NoProtection,
            None,
            "no statewide; Atlanta local ordinance",
        ),
        rule("HI", "Hawaii", NoProtection, None, "no statewide statute"),
        rule("IA", "Iowa", NoProtection, None, "no statewide statute"),
        rule("ID", "Idaho", NoProtection, None, "no statewide statute"),
        rule(
            "IL",
            "Illinois",
            StatewideFull,
            Some(2023),
            "775 ILCS 5/3-102 (statewide 2023)",
        ),
        rule("IN", "Indiana", NoProtection, None, "no statewide statute"),
        rule("KS", "Kansas", NoProtection, None, "no statewide statute"),
        rule("KY", "Kentucky", NoProtection, None, "no statewide statute"),
        rule(
            "LA",
            "Louisiana",
            NoProtection,
            None,
            "no statewide statute",
        ),
        rule(
            "MA",
            "Massachusetts",
            StatewideFull,
            Some(1989),
            "M.G.L. c. 151B § 4(10)",
        ),
        rule(
            "MD",
            "Maryland",
            StatewideFull,
            Some(2020),
            "Md. Code Real Prop. § 20-705",
        ),
        rule(
            "ME",
            "Maine",
            StatewideFull,
            Some(2009),
            "5 M.R.S. § 4581-A",
        ),
        rule("MI", "Michigan", NoProtection, None, "no statewide statute"),
        rule(
            "MN",
            "Minnesota",
            StatewideFull,
            Some(2023),
            "Minn. Stat. § 363A.09",
        ),
        rule("MO", "Missouri", NoProtection, None, "no statewide statute"),
        rule(
            "MS",
            "Mississippi",
            NoProtection,
            None,
            "no statewide statute",
        ),
        rule("MT", "Montana", NoProtection, None, "no statewide statute"),
        rule(
            "NC",
            "North Carolina",
            NoProtection,
            None,
            "no statewide statute",
        ),
        rule(
            "ND",
            "North Dakota",
            NoProtection,
            None,
            "no statewide statute",
        ),
        rule("NE", "Nebraska", NoProtection, None, "no statewide statute"),
        rule(
            "NH",
            "New Hampshire",
            NoProtection,
            None,
            "no statewide statute",
        ),
        rule(
            "NJ",
            "New Jersey",
            StatewideFull,
            Some(2002),
            "N.J.S.A. § 10:5-12",
        ),
        rule(
            "NM",
            "New Mexico",
            StatewideFull,
            Some(2023),
            "NMSA § 28-1-7",
        ),
        rule("NV", "Nevada", NoProtection, None, "no statewide statute"),
        rule(
            "NY",
            "New York",
            StatewidePartialOrChallenged,
            Some(2019),
            "RPL § 296 (HRL 2019) — statewide statute challenged March 2026; NYC local law remains",
        ),
        rule("OH", "Ohio", NoProtection, None, "no statewide statute"),
        rule("OK", "Oklahoma", NoProtection, None, "no statewide statute"),
        rule(
            "OR",
            "Oregon",
            StatewideFull,
            Some(2014),
            "ORS § 659A.421 (first statewide post-2010)",
        ),
        rule(
            "PA",
            "Pennsylvania",
            NoProtection,
            None,
            "no statewide; Philadelphia + Pittsburgh local",
        ),
        rule(
            "RI",
            "Rhode Island",
            StatewideFull,
            Some(1996),
            "R.I.G.L. § 34-37-4.3",
        ),
        rule(
            "SC",
            "South Carolina",
            NoProtection,
            None,
            "no statewide statute",
        ),
        rule(
            "SD",
            "South Dakota",
            NoProtection,
            None,
            "no statewide statute",
        ),
        rule(
            "TN",
            "Tennessee",
            NoProtection,
            None,
            "no statewide; Memphis local",
        ),
        rule(
            "TX",
            "Texas",
            NoProtection,
            None,
            "no statewide; Austin local ordinance",
        ),
        rule(
            "UT",
            "Utah",
            StatewidePartialOrChallenged,
            Some(2010),
            "Utah Code § 13-21-302 — limited to veterans, not all vouchers",
        ),
        rule(
            "VA",
            "Virginia",
            StatewideFull,
            Some(2020),
            "Va. Code § 36-96.3",
        ),
        rule(
            "VT",
            "Vermont",
            StatewideFull,
            Some(1989),
            "9 V.S.A. § 4503",
        ),
        rule(
            "WA",
            "Washington",
            StatewideFull,
            Some(2018),
            "RCW § 59.18.255",
        ),
        rule(
            "WI",
            "Wisconsin",
            NoProtection,
            None,
            "no statewide; Madison + Dane County local",
        ),
        rule(
            "WV",
            "West Virginia",
            NoProtection,
            None,
            "no statewide statute",
        ),
        rule("WY", "Wyoming", NoProtection, None, "no statewide statute"),
    ];
    RULES.iter().map(|r| (r.state_code, r)).collect()
});

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: &str, voucher: VoucherType) -> SoiCheckInput {
        SoiCheckInput {
            state_code: state.to_string(),
            voucher_type: voucher,
        }
    }

    #[test]
    fn table_covers_all_50_states_plus_dc() {
        assert_eq!(TABLE.len(), 51);
    }

    #[test]
    fn california_landlord_may_not_refuse_voucher() {
        let r = check(&input("CA", VoucherType::Section8Hcv));
        assert!(!r.landlord_may_refuse);
        assert!(!r.federal_floor_only);
        assert_eq!(r.year_enacted, Some(2019));
        assert!(r.citation.contains("SB 329"));
    }

    #[test]
    fn texas_landlord_may_refuse_no_statewide() {
        let r = check(&input("TX", VoucherType::Section8Hcv));
        assert!(r.landlord_may_refuse);
        assert!(r.federal_floor_only);
        assert!(r.year_enacted.is_none());
        assert!(r.note.contains("federal FHA"));
    }

    #[test]
    fn connecticut_oldest_statewide_1989() {
        // CT was the first state to enact statewide SOI protection.
        let r = check(&input("CT", VoucherType::Section8Hcv));
        assert!(!r.landlord_may_refuse);
        assert_eq!(r.year_enacted, Some(1989));
        assert!(r.citation.contains("oldest"));
    }

    #[test]
    fn oregon_first_statewide_post_2010() {
        // OR was the first new statewide SOI law of the 2010s wave.
        let r = check(&input("OR", VoucherType::Section8Hcv));
        assert!(!r.landlord_may_refuse);
        assert_eq!(r.year_enacted, Some(2014));
    }

    #[test]
    fn new_york_challenged_regime_verify_needed() {
        // NY HSTPA 2019 SOI provision was challenged in court (March
        // 2026 appellate ruling per news coverage). Status uncertain.
        let r = check(&input("NY", VoucherType::Section8Hcv));
        assert!(matches!(
            r.protection_regime,
            SoiProtectionRegime::StatewidePartialOrChallenged
        ));
        assert!(r.verify_current_status_needed);
        assert!(!r.landlord_may_refuse);
        assert!(r.note.contains("challenged"));
    }

    #[test]
    fn utah_partial_only_veterans() {
        // UT § 13-21-302 covers veterans only, not full voucher
        // population. Flagged as partial.
        let r = check(&input("UT", VoucherType::Section8Hcv));
        assert!(r.verify_current_status_needed);
        assert!(r.note.contains("challenged"));
    }

    #[test]
    fn all_full_statewide_states_pinned() {
        // All states with regime = StatewideFull should return
        // landlord_may_refuse: false. Catches a regression where someone
        // accidentally flips the regime bit.
        let full_states = [
            "CA", "CO", "CT", "DC", "DE", "IL", "MA", "MD", "ME", "MN", "NJ", "NM", "OR", "RI",
            "VA", "VT", "WA",
        ];
        for code in full_states {
            let r = check(&input(code, VoucherType::Section8Hcv));
            assert!(
                !r.landlord_may_refuse,
                "{code} should be full protection (landlord may not refuse)"
            );
            assert!(!r.federal_floor_only);
            assert!(r.year_enacted.is_some());
        }
    }

    #[test]
    fn voucher_type_does_not_change_state_law_outcome() {
        // All voucher types (Section 8 HCV / VASH / FUP / Other) are
        // treated the same by the per-state SOI statute. Voucher type
        // is captured on input for future-proofing only.
        let s8 = check(&input("CA", VoucherType::Section8Hcv));
        let vash = check(&input("CA", VoucherType::Vash));
        let fup = check(&input("CA", VoucherType::Fup));
        let other = check(&input("CA", VoucherType::Other));
        assert_eq!(s8.landlord_may_refuse, vash.landlord_may_refuse);
        assert_eq!(vash.landlord_may_refuse, fup.landlord_may_refuse);
        assert_eq!(fup.landlord_may_refuse, other.landlord_may_refuse);
    }

    #[test]
    fn unknown_state_defaults_to_federal_floor() {
        let r = check(&input("ZZ", VoucherType::Section8Hcv));
        assert!(r.landlord_may_refuse);
        assert!(r.federal_floor_only);
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
    fn year_2023_wave_states_pinned() {
        // IL, MN, NM all enacted statewide SOI protection in 2023.
        // Catches a future regression that downgrades one of them.
        for code in ["IL", "MN", "NM"] {
            let r = lookup(code).unwrap();
            assert!(matches!(r.regime, SoiProtectionRegime::StatewideFull));
            assert_eq!(r.year_enacted, Some(2023));
        }
    }

    #[test]
    fn year_2020_wave_states_pinned() {
        // 2020 wave: CO, DE, MD, VA. All passed during the racial-
        // equity / fair-housing legislative push.
        for code in ["CO", "DE", "MD", "VA"] {
            let r = lookup(code).unwrap();
            assert!(matches!(r.regime, SoiProtectionRegime::StatewideFull));
            assert_eq!(r.year_enacted, Some(2020));
        }
    }

    #[test]
    fn pre_2000_pioneers_have_full_protection() {
        // CT (1989), MA (1989), VT (1989), RI (1996), DC (1977) —
        // the pre-2000 statewide SOI pioneers.
        for code in ["CT", "MA", "VT", "RI", "DC"] {
            let r = lookup(code).unwrap();
            assert!(matches!(r.regime, SoiProtectionRegime::StatewideFull));
            assert!(r.year_enacted.unwrap() < 2000);
        }
    }

    #[test]
    fn local_only_states_flagged_in_citation() {
        // GA (Atlanta), PA (Phila/Pittsburgh), TN (Memphis), TX (Austin),
        // WI (Madison) all mention local ordinances in the citation
        // even though the regime is NoProtection (no statewide statute).
        // This tells the caller to check municipal law.
        for code in ["GA", "PA", "TN", "TX", "WI"] {
            let r = lookup(code).unwrap();
            assert!(matches!(r.regime, SoiProtectionRegime::NoProtection));
            assert!(
                r.citation.contains("local"),
                "{code} citation should mention local ordinance"
            );
        }
    }

    #[test]
    fn challenged_states_set_verify_needed_flag() {
        // NY and UT both have verify_current_status_needed set true.
        // The flag is what tells the UI to surface a "check current
        // status" warning rather than reporting a definitive answer.
        for code in ["NY", "UT"] {
            let r = check(&input(code, VoucherType::Section8Hcv));
            assert!(r.verify_current_status_needed);
        }
    }
}
