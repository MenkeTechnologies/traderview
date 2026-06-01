//! State eviction record sealing / "clean slate" compliance table.
//!
//! Direct trader-landlord screening concern: 17 states + DC have
//! enacted laws since 2020 that either auto-seal, petition-seal, or
//! restrict the screening use of eviction court records. Pulling a
//! tenant screening report that surfaces a sealed record (or
//! screening on a record that was supposed to be sealed) creates
//! state-law screening liability.
//!
//! Four-regime classification:
//!
//! 1. **`AutomaticSealing`** — court seals the record without tenant
//!    petition once statutory conditions are met. CA, CT, NV, MD, MN.
//!
//! 2. **`TenantPetitionOnly`** — tenant must file a petition to seal;
//!    no automatic action. WA, IL, OR, DC.
//!
//! 3. **`PandemicPeriodOnly`** — sealing limited to COVID-era filings
//!    (Mar 2020 – Mar 2022 window typical). NJ.
//!
//! 4. **`NoStateRule`** — no statewide eviction record sealing
//!    framework. Tenant screening reports may surface any unsealed
//!    case subject to federal FCRA's 7-year window for civil
//!    judgments. Most US states.
//!
//! Federal floor (FCRA 15 U.S.C. § 1681c(a)(2)): civil judgments more
//! than 7 years old cannot appear on tenant screening consumer
//! reports. State sealing laws layer ON TOP of this — many push the
//! window earlier (CA = 60 days at filing; CT = 30 days for favorable
//! outcomes).
//!
//! Case outcome classification reflects sealing law triggers:
//! - `LandlordWonJudgment`: most adverse to tenant; CA's 60-day
//!   auto-mask is the only auto-sealing in most landlord-won states.
//! - `DismissedOrWithdrawn`: triggers CT 30-day auto-seal, MN
//!   auto-expunge.
//! - `TenantWonJudgment`: triggers CT 30-day auto-seal, MN
//!   auto-expunge.
//! - `SettlementOrConsent`: state-specific; usually falls under
//!   "favorable outcome" sealing.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SealingRegime {
    AutomaticSealing,
    TenantPetitionOnly,
    PandemicPeriodOnly,
    NoStateRule,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseOutcome {
    LandlordWonJudgment,
    DismissedOrWithdrawn,
    TenantWonJudgment,
    SettlementOrConsent,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: SealingRegime,
    /// Days from filing or judgment to automatic sealing. For
    /// CA (AB 2819): 60 days at filing if landlord doesn't win
    /// judgment. For CT (SB 998): 30 days from favorable outcome.
    /// For MD: 60 days from judgment if non-removal outcome.
    /// `None` for non-automatic regimes.
    pub auto_seal_window_days: Option<i64>,
    /// Outcomes triggering automatic sealing. Empty under
    /// non-automatic regimes.
    pub auto_seal_eligible_outcomes: &'static [CaseOutcome],
    /// True if the state law restricts screening companies from
    /// reporting sealed cases (broader than just court sealing — adds
    /// landlord-side liability for screening on sealed records).
    pub screening_use_restricted: bool,
    pub citation: &'static str,
}

const fn rule(
    regime: SealingRegime,
    auto_seal_window_days: Option<i64>,
    auto_seal_eligible_outcomes: &'static [CaseOutcome],
    screening_use_restricted: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        auto_seal_window_days,
        auto_seal_eligible_outcomes,
        screening_use_restricted,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use CaseOutcome::*;
    use SealingRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    // AutomaticSealing regime.
    m.insert(
        "CA",
        rule(
            AutomaticSealing,
            Some(60),
            &[
                DismissedOrWithdrawn,
                TenantWonJudgment,
                SettlementOrConsent,
                LandlordWonJudgment, // 60-day mask at filing covers all
            ],
            true,
            "Cal. Civ. Proc. Code § 1161.2 (AB 2819, eff. 2017-01-01)",
        ),
    );
    m.insert(
        "CT",
        rule(
            AutomaticSealing,
            Some(30),
            &[DismissedOrWithdrawn, TenantWonJudgment],
            true,
            "Conn. Public Act 23-207 (SB 998, eff. 2024-07-01)",
        ),
    );
    m.insert(
        "NV",
        rule(
            AutomaticSealing,
            Some(31),
            &[DismissedOrWithdrawn, TenantWonJudgment],
            false,
            "NRS 40.2545 (AB 107 of 2017 + AB 161 of 2021)",
        ),
    );
    m.insert(
        "MD",
        rule(
            AutomaticSealing,
            Some(60),
            &[
                DismissedOrWithdrawn,
                TenantWonJudgment,
                SettlementOrConsent,
            ],
            true,
            "Md. Real Prop. § 8-401 (SB 19 of 2024)",
        ),
    );
    m.insert(
        "MN",
        rule(
            AutomaticSealing,
            Some(0), // Same-day mandatory expungement on qualifying outcomes
            &[DismissedOrWithdrawn, TenantWonJudgment],
            false,
            "Minn. Stat. § 504B.345 subd. 1(c) (SF 3492 of 2024)",
        ),
    );

    // TenantPetitionOnly regime.
    m.insert(
        "WA",
        rule(
            TenantPetitionOnly,
            None,
            &[],
            true,
            "Wash. RCW 59.18.367",
        ),
    );
    m.insert(
        "OR",
        rule(
            TenantPetitionOnly,
            None,
            &[],
            true,
            "Or. SB 282 of 2021 (covid-window + petition pathway)",
        ),
    );
    m.insert(
        "IL",
        rule(
            TenantPetitionOnly,
            None,
            &[],
            true,
            "Ill. 735 ILCS 5/9-121 (HB 2877 + COVID Emergency Period order)",
        ),
    );
    m.insert(
        "DC",
        rule(
            TenantPetitionOnly,
            None,
            &[],
            true,
            "D.C. Eviction Record Sealing Authority Act of 2022",
        ),
    );

    // PandemicPeriodOnly regime.
    m.insert(
        "NJ",
        rule(
            PandemicPeriodOnly,
            None,
            &[],
            false,
            "N.J. P.L. 2021, c. 188 (A 4463) — pandemic period only",
        ),
    );

    // NoStateRule — explicit list of all remaining states + territories
    // currently silent on eviction record sealing at the state level.
    let no_rule_states = [
        "AL", "AK", "AZ", "AR", "CO", "DE", "FL", "GA", "HI", "ID",
        "IN", "IA", "KS", "KY", "LA", "ME", "MA", "MI", "MS", "MO",
        "MT", "NE", "NH", "NM", "NY", "NC", "ND", "OH", "OK", "PA",
        "RI", "SC", "SD", "TN", "TX", "UT", "VT", "VA", "WV", "WI",
        "WY",
    ];
    for code in no_rule_states {
        m.insert(
            code,
            rule(
                NoStateRule,
                None,
                &[],
                false,
                "No statewide eviction record sealing law; FCRA 7-year limit applies",
            ),
        );
    }
    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvictionSealingInput {
    pub state_code: String,
    pub case_outcome: CaseOutcome,
    pub days_since_filing_or_qualifying_event: i64,
    /// True if the eviction case was filed during the COVID emergency
    /// period (state-specific window, generally Mar 2020 – Mar 2022).
    pub pandemic_period_case: bool,
    /// True if a tenant has filed a petition to seal (relevant under
    /// TenantPetitionOnly regime).
    pub tenant_petitioned_for_sealing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvictionSealingResult {
    pub regime: SealingRegime,
    pub auto_seal_window_days: Option<i64>,
    pub auto_seal_window_satisfied: bool,
    pub outcome_qualifies_for_auto_seal: bool,
    pub eligible_for_sealing: bool,
    pub petition_required: bool,
    pub screening_use_restricted_by_state: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &EvictionSealingInput) -> EvictionSealingResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: SealingRegime::NoStateRule,
        auto_seal_window_days: None,
        auto_seal_eligible_outcomes: &[],
        screening_use_restricted: false,
        citation: "Unknown state code; assuming no state rule",
    });

    let outcome_qualifies = rule
        .auto_seal_eligible_outcomes
        .contains(&input.case_outcome);
    let window_satisfied = match rule.auto_seal_window_days {
        Some(w) => input.days_since_filing_or_qualifying_event >= w,
        None => false,
    };

    let (eligible, petition_required) = match rule.regime {
        SealingRegime::AutomaticSealing => {
            (outcome_qualifies && window_satisfied, false)
        }
        SealingRegime::TenantPetitionOnly => {
            (input.tenant_petitioned_for_sealing, true)
        }
        SealingRegime::PandemicPeriodOnly => (input.pandemic_period_case, false),
        SealingRegime::NoStateRule => (false, false),
    };

    let note = match rule.regime {
        SealingRegime::AutomaticSealing if eligible => format!(
            "AutomaticSealing: case sealed by court after {} days from qualifying event. Screening companies must remove the record.",
            rule.auto_seal_window_days.unwrap_or(0),
        ),
        SealingRegime::AutomaticSealing if !outcome_qualifies => format!(
            "AutomaticSealing: outcome {:?} does NOT qualify for automatic sealing in this state. Tenant may need petition or other relief.",
            input.case_outcome,
        ),
        SealingRegime::AutomaticSealing => format!(
            "AutomaticSealing: {} of {} days elapsed since qualifying event; record will auto-seal at the {}-day mark.",
            input.days_since_filing_or_qualifying_event,
            rule.auto_seal_window_days.unwrap_or(0),
            rule.auto_seal_window_days.unwrap_or(0),
        ),
        SealingRegime::TenantPetitionOnly if eligible =>
            "TenantPetitionOnly: tenant has petitioned; court order pending. Once granted, screening companies must remove the record.".to_string(),
        SealingRegime::TenantPetitionOnly =>
            "TenantPetitionOnly: tenant must file petition to seal. No automatic action.".to_string(),
        SealingRegime::PandemicPeriodOnly if eligible =>
            "PandemicPeriodOnly: COVID-era case qualifies for sealing under pandemic relief statute.".to_string(),
        SealingRegime::PandemicPeriodOnly =>
            "PandemicPeriodOnly: only COVID-era cases are sealable in this state.".to_string(),
        SealingRegime::NoStateRule =>
            "NoStateRule: no statewide eviction record sealing; FCRA 15 U.S.C. § 1681c 7-year limit applies for tenant screening reports.".to_string(),
    };

    EvictionSealingResult {
        regime: rule.regime,
        auto_seal_window_days: rule.auto_seal_window_days,
        auto_seal_window_satisfied: window_satisfied,
        outcome_qualifies_for_auto_seal: outcome_qualifies,
        eligible_for_sealing: eligible,
        petition_required,
        screening_use_restricted_by_state: rule.screening_use_restricted,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: &str, outcome: CaseOutcome, days: i64) -> EvictionSealingInput {
        EvictionSealingInput {
            state_code: state.to_string(),
            case_outcome: outcome,
            days_since_filing_or_qualifying_event: days,
            pandemic_period_case: false,
            tenant_petitioned_for_sealing: false,
        }
    }

    #[test]
    fn ca_60_day_auto_mask_at_filing_complies() {
        // CA AB 2819: 60-day mask at filing covers all outcomes.
        let r = check(&input("CA", CaseOutcome::LandlordWonJudgment, 60));
        assert!(r.eligible_for_sealing);
        assert!(r.outcome_qualifies_for_auto_seal);
        assert!(r.auto_seal_window_satisfied);
        assert!(!r.petition_required);
    }

    #[test]
    fn ca_under_60_days_not_yet_sealed() {
        let r = check(&input("CA", CaseOutcome::LandlordWonJudgment, 59));
        assert!(!r.eligible_for_sealing);
        assert!(!r.auto_seal_window_satisfied);
    }

    #[test]
    fn ct_30_day_dismissed_qualifies() {
        // CT SB 998: 30-day auto-seal for dismissed cases.
        let r = check(&input("CT", CaseOutcome::DismissedOrWithdrawn, 30));
        assert!(r.eligible_for_sealing);
        assert!(r.outcome_qualifies_for_auto_seal);
    }

    #[test]
    fn ct_landlord_won_does_not_auto_seal() {
        // CT auto-seal only covers tenant-favorable outcomes.
        let r = check(&input("CT", CaseOutcome::LandlordWonJudgment, 365));
        assert!(!r.eligible_for_sealing);
        assert!(!r.outcome_qualifies_for_auto_seal);
        assert!(r.note.contains("does NOT qualify"));
    }

    #[test]
    fn ct_day_29_not_yet_sealed() {
        let r = check(&input("CT", CaseOutcome::TenantWonJudgment, 29));
        assert!(!r.auto_seal_window_satisfied);
        assert!(!r.eligible_for_sealing);
    }

    #[test]
    fn ct_day_30_exact_boundary_complies() {
        let r = check(&input("CT", CaseOutcome::TenantWonJudgment, 30));
        assert!(r.auto_seal_window_satisfied);
        assert!(r.eligible_for_sealing);
    }

    #[test]
    fn mn_same_day_expungement_complies() {
        // MN SF 3492: same-day mandatory expungement on qualifying outcomes.
        let r = check(&input("MN", CaseOutcome::DismissedOrWithdrawn, 0));
        assert!(r.eligible_for_sealing);
    }

    #[test]
    fn nv_31_day_window_landlord_won_does_not_qualify() {
        let r = check(&input("NV", CaseOutcome::LandlordWonJudgment, 365));
        assert!(!r.outcome_qualifies_for_auto_seal);
        assert!(!r.eligible_for_sealing);
    }

    #[test]
    fn nv_31_day_dismissed_qualifies() {
        let r = check(&input("NV", CaseOutcome::DismissedOrWithdrawn, 31));
        assert!(r.eligible_for_sealing);
    }

    #[test]
    fn md_60_day_settlement_qualifies() {
        // MD SB 19 of 2024 covers settlement outcomes (non-removal path).
        let r = check(&input("MD", CaseOutcome::SettlementOrConsent, 60));
        assert!(r.eligible_for_sealing);
    }

    #[test]
    fn wa_petition_required_without_petition_no_sealing() {
        let r = check(&input("WA", CaseOutcome::TenantWonJudgment, 365));
        assert_eq!(r.regime, SealingRegime::TenantPetitionOnly);
        assert!(r.petition_required);
        assert!(!r.eligible_for_sealing);
    }

    #[test]
    fn wa_with_petition_eligible() {
        let mut i = input("WA", CaseOutcome::TenantWonJudgment, 365);
        i.tenant_petitioned_for_sealing = true;
        let r = check(&i);
        assert!(r.petition_required);
        assert!(r.eligible_for_sealing);
    }

    #[test]
    fn or_petition_only_regime() {
        let r = check(&input("OR", CaseOutcome::LandlordWonJudgment, 1000));
        assert_eq!(r.regime, SealingRegime::TenantPetitionOnly);
        assert!(!r.eligible_for_sealing);
    }

    #[test]
    fn il_petition_only_regime() {
        let r = check(&input("IL", CaseOutcome::DismissedOrWithdrawn, 1000));
        assert_eq!(r.regime, SealingRegime::TenantPetitionOnly);
        assert!(r.petition_required);
        assert!(!r.eligible_for_sealing);
    }

    #[test]
    fn dc_petition_only_regime() {
        let r = check(&input("DC", CaseOutcome::TenantWonJudgment, 0));
        assert_eq!(r.regime, SealingRegime::TenantPetitionOnly);
    }

    #[test]
    fn nj_pandemic_only_non_pandemic_case_ineligible() {
        let r = check(&input("NJ", CaseOutcome::DismissedOrWithdrawn, 365));
        assert_eq!(r.regime, SealingRegime::PandemicPeriodOnly);
        assert!(!r.eligible_for_sealing);
    }

    #[test]
    fn nj_pandemic_case_eligible() {
        let mut i = input("NJ", CaseOutcome::LandlordWonJudgment, 365);
        i.pandemic_period_case = true;
        let r = check(&i);
        assert!(r.eligible_for_sealing);
        assert!(r.note.contains("COVID-era"));
    }

    #[test]
    fn no_state_rule_state_never_eligible() {
        for st in &["TX", "FL", "NY", "PA", "OH"] {
            let r = check(&input(st, CaseOutcome::TenantWonJudgment, 10_000));
            assert_eq!(r.regime, SealingRegime::NoStateRule, "state {st}");
            assert!(!r.eligible_for_sealing, "state {st}");
        }
    }

    #[test]
    fn no_state_rule_note_cites_fcra_floor() {
        let r = check(&input("TX", CaseOutcome::LandlordWonJudgment, 0));
        assert!(r.note.contains("FCRA"));
    }

    #[test]
    fn screening_use_restricted_flag_set_for_auto_seal_states() {
        // CA, CT, MD restrict screening use; NV and MN do not have the
        // explicit screening-side ban.
        assert!(RULES.get("CA").unwrap().screening_use_restricted);
        assert!(RULES.get("CT").unwrap().screening_use_restricted);
        assert!(RULES.get("MD").unwrap().screening_use_restricted);
        assert!(!RULES.get("NV").unwrap().screening_use_restricted);
    }

    #[test]
    fn screening_use_restricted_flag_set_for_petition_states() {
        // WA, IL, OR, DC all restrict screening use even though sealing
        // requires petition.
        for code in &["WA", "IL", "OR", "DC"] {
            assert!(
                RULES.get(code).unwrap().screening_use_restricted,
                "{code} screening_use_restricted should be true"
            );
        }
    }

    #[test]
    fn unknown_state_falls_back_to_no_rule() {
        let r = check(&input("XX", CaseOutcome::TenantWonJudgment, 100));
        assert_eq!(r.regime, SealingRegime::NoStateRule);
    }

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&input("ca", CaseOutcome::DismissedOrWithdrawn, 60));
        assert!(r.eligible_for_sealing);
    }

    #[test]
    fn coverage_is_all_50_states_plus_dc() {
        let codes: Vec<&'static str> = RULES.keys().copied().collect();
        assert_eq!(codes.len(), 51, "expected 50 states + DC, got {}", codes.len());
    }

    #[test]
    fn citations_never_empty() {
        for (code, rule) in RULES.iter() {
            assert!(!rule.citation.is_empty(), "{code} missing citation");
        }
    }

    #[test]
    fn auto_seal_regime_states_have_window_set() {
        // Invariant: every AutomaticSealing state must declare a window.
        for (code, rule) in RULES.iter() {
            if rule.regime == SealingRegime::AutomaticSealing {
                assert!(
                    rule.auto_seal_window_days.is_some(),
                    "{code} has AutomaticSealing without window"
                );
            }
        }
    }

    #[test]
    fn non_auto_seal_regime_states_have_no_window() {
        for (code, rule) in RULES.iter() {
            if rule.regime != SealingRegime::AutomaticSealing {
                assert!(
                    rule.auto_seal_window_days.is_none(),
                    "{code} has non-AutomaticSealing regime but a window set"
                );
            }
        }
    }

    #[test]
    fn note_describes_partial_window_progress() {
        // 15/30 days through CT window — note should describe progress.
        let r = check(&input("CT", CaseOutcome::DismissedOrWithdrawn, 15));
        assert!(r.note.contains("15 of 30 days"));
    }
}
