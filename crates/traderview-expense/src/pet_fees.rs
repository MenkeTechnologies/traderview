//! State-level pet deposit / pet rent / pet fee regulation table.
//!
//! Four-regime classification of how each state regulates the money a
//! landlord may charge for a tenant's pet:
//!
//! 1. **`SpecificPetDepositAndRentCap`** — explicit statutory caps on
//!    BOTH the pet deposit and the recurring pet rent.
//!    Colorado (HB 23-1068, eff. 2024-01-01, CRS § 38-12-106): pet
//!    deposit ≤ $300 refundable; pet rent ≤ max($35, 1.5% × monthly
//!    rent); breed-based insurance discrimination banned; landlord pet
//!    liens banned.
//!
//! 2. **`TotalDepositCapAbsorbsPet`** — no SEPARATE pet deposit cap,
//!    but the overall security deposit cap (whatever it is) must
//!    absorb anything labelled as "pet deposit". California (AB 12,
//!    eff. 2024-07-01) caps total deposits including any pet portion
//!    at one month's rent regardless of furnished status. Same shape
//!    applies in Washington (RCW 59.18.260 — no statutory cap on
//!    amount but pet deposit must be in the written rental agreement
//!    and is part of the security deposit framework).
//!
//! 3. **`NoSeparatePetDepositAllowed`** — landlord may NOT collect any
//!    separate pet deposit; the overall security deposit (one month's
//!    rent in MA, MGL c.186 § 15B) is the only allowed up-front
//!    money. Pet rent recurring monthly is permitted by case law; no
//!    other pet fees, no pet move-in fees, no pet cleaning fees.
//!    Massachusetts is the leading example.
//!
//! 4. **`NoStateRule`** — no state-level cap. Local ordinances and the
//!    overall security deposit cap (if any) still apply. The bulk of
//!    US states fall here including FL, TX, NY, NJ, IL, OR (each has
//!    its own overall security deposit framework but no pet-specific
//!    cap).
//!
//! Federal floor that ALWAYS applies regardless of state regime: under
//! the Fair Housing Act (42 U.S.C. § 3604(f)) and ADA (42 U.S.C.
//! § 12101 et seq.), service animals AND emotional support animals
//! are NOT pets. No pet deposit, no pet rent, no pet fee may be
//! charged for an ESA/service animal in any state. This is the
//! federal-floor pattern shared with [`lead_disclosure`],
//! [`bedbug_disclosure`], etc.

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PetFeeRegime {
    SpecificPetDepositAndRentCap,
    TotalDepositCapAbsorbsPet,
    NoSeparatePetDepositAllowed,
    NoStateRule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateRule {
    pub regime: PetFeeRegime,
    /// Specific pet-deposit cap in cents, when the regime imposes one.
    /// `None` under TotalDepositCapAbsorbsPet (no separate cap — total
    /// security deposit cap is the binding constraint) and under
    /// NoSeparatePetDepositAllowed (cap is effectively zero, but
    /// modeling it as zero rather than None would conflate "banned"
    /// with "no cap"). Use `pet_deposit_banned` to distinguish.
    pub pet_deposit_cap_cents: Option<i64>,
    pub pet_deposit_banned: bool,
    /// Specific pet-rent monthly cap in cents.
    pub pet_rent_monthly_cap_cents: Option<i64>,
    /// Pet rent also capped as a percentage of monthly rent — only CO
    /// uses this so far (1.5%). When both flat and percentage are
    /// set, the actual cap is the GREATER of the two (CO statute).
    pub pet_rent_pct_of_monthly_rent_bp: Option<u32>,
    pub breed_based_insurance_discrimination_banned: bool,
    pub pet_lien_banned: bool,
    pub citation: &'static str,
}

#[allow(clippy::too_many_arguments)]
const fn rule(
    regime: PetFeeRegime,
    pet_deposit_cap_cents: Option<i64>,
    pet_deposit_banned: bool,
    pet_rent_monthly_cap_cents: Option<i64>,
    pet_rent_pct_of_monthly_rent_bp: Option<u32>,
    breed_based_insurance_discrimination_banned: bool,
    pet_lien_banned: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        pet_deposit_cap_cents,
        pet_deposit_banned,
        pet_rent_monthly_cap_cents,
        pet_rent_pct_of_monthly_rent_bp,
        breed_based_insurance_discrimination_banned,
        pet_lien_banned,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use PetFeeRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    // Specific pet deposit + rent cap (Colorado, the strictest scheme).
    m.insert(
        "CO",
        rule(
            SpecificPetDepositAndRentCap,
            Some(30_000),   // $300
            false,
            Some(3_500),    // $35 OR 1.5%
            Some(150),      // 1.5% of monthly rent
            true,
            true,
            "CRS § 38-12-106 (HB 23-1068, eff. 2024-01-01)",
        ),
    );

    // Total deposit cap absorbs pet (CA AB 12; WA RCW 59.18.260).
    m.insert(
        "CA",
        rule(
            TotalDepositCapAbsorbsPet,
            None,
            false,
            None,
            None,
            false,
            false,
            "Cal. Civ. Code § 1950.5 (AB 12, eff. 2024-07-01)",
        ),
    );
    m.insert(
        "WA",
        rule(
            TotalDepositCapAbsorbsPet,
            None,
            false,
            None,
            None,
            false,
            false,
            "RCW 59.18.260",
        ),
    );

    // No separate pet deposit allowed (MA case law + MGL c.186 § 15B).
    m.insert(
        "MA",
        rule(
            NoSeparatePetDepositAllowed,
            None,
            true,
            None,
            None,
            false,
            false,
            "MGL c.186 § 15B + case law",
        ),
    );

    // No state rule — explicit list of states currently silent on pet
    // deposit/rent (each has its own overall security deposit
    // framework but no pet-specific cap).
    let no_rule_states = [
        "AL", "AK", "AZ", "AR", "CT", "DE", "DC", "FL", "GA", "HI", "ID",
        "IL", "IN", "IA", "KS", "KY", "LA", "ME", "MD", "MI", "MN",
        "MS", "MO", "MT", "NE", "NV", "NH", "NJ", "NM", "NY", "NC", "ND",
        "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN", "TX", "UT",
        "VT", "VA", "WV", "WI", "WY",
    ];
    for code in no_rule_states {
        m.insert(
            code,
            rule(
                NoStateRule,
                None,
                false,
                None,
                None,
                false,
                false,
                "No state-level pet fee cap; security deposit framework applies",
            ),
        );
    }
    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetFeeInput {
    pub state_code: String,
    pub monthly_rent_cents: i64,
    pub charged_pet_deposit_cents: i64,
    pub charged_pet_rent_monthly_cents: i64,
    /// True if the tenant's animal is a service animal or ESA — federal
    /// FHA + ADA forbid ANY pet charges for such animals in every state.
    pub is_service_or_esa: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetFeeResult {
    pub regime: PetFeeRegime,
    pub federal_fha_ada_exemption_applies: bool,
    pub pet_deposit_cap_cents: Option<i64>,
    pub effective_pet_rent_cap_cents: Option<i64>,
    pub pet_deposit_exceeds_cap: bool,
    pub pet_deposit_banned: bool,
    pub pet_rent_exceeds_cap: bool,
    pub pet_deposit_violation_amount_cents: i64,
    pub pet_rent_violation_amount_cents: i64,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &PetFeeInput) -> PetFeeResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: PetFeeRegime::NoStateRule,
        pet_deposit_cap_cents: None,
        pet_deposit_banned: false,
        pet_rent_monthly_cap_cents: None,
        pet_rent_pct_of_monthly_rent_bp: None,
        breed_based_insurance_discrimination_banned: false,
        pet_lien_banned: false,
        citation: "Unknown state code; assuming no state rule",
    });

    // Federal FHA/ADA floor: zero-charge for service animal / ESA.
    if input.is_service_or_esa {
        return PetFeeResult {
            regime: rule.regime,
            federal_fha_ada_exemption_applies: true,
            pet_deposit_cap_cents: Some(0),
            effective_pet_rent_cap_cents: Some(0),
            pet_deposit_exceeds_cap: input.charged_pet_deposit_cents > 0,
            pet_deposit_banned: true,
            pet_rent_exceeds_cap: input.charged_pet_rent_monthly_cents > 0,
            pet_deposit_violation_amount_cents: input.charged_pet_deposit_cents.max(0),
            pet_rent_violation_amount_cents: input.charged_pet_rent_monthly_cents.max(0),
            citation: format!(
                "Federal FHA (42 U.S.C. § 3604(f)) + ADA (42 U.S.C. § 12101) — service animal/ESA exemption overrides state law; state citation: {}",
                rule.citation
            ),
            note: "Service animal / ESA: no pet deposit, no pet rent, no pet fee — federal FHA + ADA preempt.".to_string(),
        };
    }

    // Pet rent cap: GREATER of flat-dollar cap and percentage-of-rent cap.
    let pct_cap = rule
        .pet_rent_pct_of_monthly_rent_bp
        .map(|bp| input.monthly_rent_cents.saturating_mul(bp as i64) / 10_000);
    let effective_pet_rent_cap = match (rule.pet_rent_monthly_cap_cents, pct_cap) {
        (Some(flat), Some(pct)) => Some(flat.max(pct)),
        (Some(flat), None) => Some(flat),
        (None, Some(pct)) => Some(pct),
        (None, None) => None,
    };

    let pet_deposit_exceeds_cap = if rule.pet_deposit_banned {
        input.charged_pet_deposit_cents > 0
    } else {
        match rule.pet_deposit_cap_cents {
            Some(cap) => input.charged_pet_deposit_cents > cap,
            None => false,
        }
    };
    let pet_rent_exceeds_cap = match effective_pet_rent_cap {
        Some(cap) => input.charged_pet_rent_monthly_cents > cap,
        None => false,
    };
    let pet_deposit_violation = if pet_deposit_exceeds_cap {
        if rule.pet_deposit_banned {
            input.charged_pet_deposit_cents
        } else {
            input
                .charged_pet_deposit_cents
                .saturating_sub(rule.pet_deposit_cap_cents.unwrap_or(0))
        }
    } else {
        0
    };
    let pet_rent_violation = if pet_rent_exceeds_cap {
        input
            .charged_pet_rent_monthly_cents
            .saturating_sub(effective_pet_rent_cap.unwrap_or(0))
    } else {
        0
    };

    let note = match rule.regime {
        PetFeeRegime::SpecificPetDepositAndRentCap => format!(
            "State imposes explicit caps: pet deposit ≤ ${} refundable; pet rent ≤ ${}/mo (greater of flat / pct-of-rent).",
            rule.pet_deposit_cap_cents.unwrap_or(0) / 100,
            effective_pet_rent_cap.unwrap_or(0) / 100,
        ),
        PetFeeRegime::TotalDepositCapAbsorbsPet =>
            "No separate pet deposit cap — pet deposit must fit inside the overall security deposit cap.".to_string(),
        PetFeeRegime::NoSeparatePetDepositAllowed =>
            "No separate pet deposit allowed; monthly pet rent permitted by case law; no other pet fees.".to_string(),
        PetFeeRegime::NoStateRule =>
            "No state-level cap on pet deposit or pet rent.".to_string(),
    };

    PetFeeResult {
        regime: rule.regime,
        federal_fha_ada_exemption_applies: false,
        pet_deposit_cap_cents: rule.pet_deposit_cap_cents,
        effective_pet_rent_cap_cents: effective_pet_rent_cap,
        pet_deposit_exceeds_cap,
        pet_deposit_banned: rule.pet_deposit_banned,
        pet_rent_exceeds_cap,
        pet_deposit_violation_amount_cents: pet_deposit_violation,
        pet_rent_violation_amount_cents: pet_rent_violation,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: &str, dep: i64, rent_pet: i64, rent_mo: i64) -> PetFeeInput {
        PetFeeInput {
            state_code: state.to_string(),
            monthly_rent_cents: rent_mo,
            charged_pet_deposit_cents: dep,
            charged_pet_rent_monthly_cents: rent_pet,
            is_service_or_esa: false,
        }
    }

    #[test]
    fn co_pet_deposit_at_300_exact_complies() {
        let r = check(&input("CO", 30_000, 3_500, 200_000));
        assert!(!r.pet_deposit_exceeds_cap);
        assert!(!r.pet_rent_exceeds_cap);
    }

    #[test]
    fn co_pet_deposit_301_violates() {
        let r = check(&input("CO", 30_100, 3_500, 200_000));
        assert!(r.pet_deposit_exceeds_cap);
        assert_eq!(r.pet_deposit_violation_amount_cents, 100);
    }

    #[test]
    fn co_pet_rent_flat_35_dominates_low_rent() {
        // $35 > 1.5% × $1000 ($15) → cap = $35.
        let r = check(&input("CO", 0, 3_500, 100_000));
        assert_eq!(r.effective_pet_rent_cap_cents, Some(3_500));
        assert!(!r.pet_rent_exceeds_cap);
        // $36/mo violates.
        let r2 = check(&input("CO", 0, 3_600, 100_000));
        assert!(r2.pet_rent_exceeds_cap);
        assert_eq!(r2.pet_rent_violation_amount_cents, 100);
    }

    #[test]
    fn co_pet_rent_pct_dominates_high_rent() {
        // 1.5% × $5000 = $75 > $35 → cap = $75.
        let r = check(&input("CO", 0, 7_500, 500_000));
        assert_eq!(r.effective_pet_rent_cap_cents, Some(7_500));
        assert!(!r.pet_rent_exceeds_cap);
        // $76 violates.
        let r2 = check(&input("CO", 0, 7_600, 500_000));
        assert!(r2.pet_rent_exceeds_cap);
        assert_eq!(r2.pet_rent_violation_amount_cents, 100);
    }

    #[test]
    fn co_pinned_flags_breed_insurance_and_pet_lien_bans() {
        // Both flags are part of HB 23-1068 — pin them on the rule
        // table directly (caller-facing flags surface from the rule).
        let rule = RULES.get("CO").unwrap();
        assert!(rule.breed_based_insurance_discrimination_banned);
        assert!(rule.pet_lien_banned);
    }

    #[test]
    fn ca_no_separate_pet_deposit_cap_only_total_cap() {
        // CA AB 12 doesn't pet-cap, just absorbs into total. Module
        // reports no separate cap; caller must enforce the overall
        // 1-month total via the security-deposit module.
        let r = check(&input("CA", 100_000, 5_000, 200_000));
        assert_eq!(r.regime, PetFeeRegime::TotalDepositCapAbsorbsPet);
        assert_eq!(r.pet_deposit_cap_cents, None);
        assert!(!r.pet_deposit_exceeds_cap);
    }

    #[test]
    fn ma_separate_pet_deposit_banned() {
        // ANY non-zero pet deposit in MA violates.
        let r = check(&input("MA", 1, 0, 200_000));
        assert!(r.pet_deposit_banned);
        assert!(r.pet_deposit_exceeds_cap);
        assert_eq!(r.pet_deposit_violation_amount_cents, 1);
    }

    #[test]
    fn ma_zero_pet_deposit_complies_monthly_rent_allowed() {
        // Zero pet deposit but $50/mo pet rent → fine (case law allows).
        let r = check(&input("MA", 0, 5_000, 200_000));
        assert!(!r.pet_deposit_exceeds_cap);
        assert!(!r.pet_rent_exceeds_cap);
    }

    #[test]
    fn fl_tx_ny_no_state_rule_any_amount_complies() {
        // No state-level cap — module reports no violation regardless
        // of amount (overall security deposit framework is separate).
        for st in &["FL", "TX", "NY", "NJ", "IL"] {
            let r = check(&input(st, 500_000, 50_000, 200_000));
            assert_eq!(r.regime, PetFeeRegime::NoStateRule, "state {st}");
            assert!(!r.pet_deposit_exceeds_cap, "state {st}");
            assert!(!r.pet_rent_exceeds_cap, "state {st}");
        }
    }

    #[test]
    fn service_animal_zero_charge_federal_preemption() {
        // Service animal / ESA: $0 deposit + $0 rent regardless of state.
        let mut i = input("CO", 30_000, 3_500, 200_000);
        i.is_service_or_esa = true;
        let r = check(&i);
        assert!(r.federal_fha_ada_exemption_applies);
        assert!(r.pet_deposit_exceeds_cap);
        assert!(r.pet_rent_exceeds_cap);
        assert_eq!(r.pet_deposit_violation_amount_cents, 30_000);
        assert_eq!(r.pet_rent_violation_amount_cents, 3_500);
    }

    #[test]
    fn service_animal_in_no_rule_state_still_federal_protected() {
        // Even where no state rule exists, federal FHA/ADA still bans
        // any pet fee for service animal / ESA. Pinned because the
        // federal floor crossing the state-silence boundary is the
        // exact failure mode of forgetting the federal layer.
        let mut i = input("TX", 100_000, 10_000, 200_000);
        i.is_service_or_esa = true;
        let r = check(&i);
        assert!(r.federal_fha_ada_exemption_applies);
        assert!(r.pet_deposit_exceeds_cap);
        assert!(r.pet_rent_exceeds_cap);
    }

    #[test]
    fn service_animal_zero_charges_no_violation() {
        // Tenant doesn't pay anything for service animal — compliant.
        let mut i = input("CA", 0, 0, 200_000);
        i.is_service_or_esa = true;
        let r = check(&i);
        assert!(r.federal_fha_ada_exemption_applies);
        assert!(!r.pet_deposit_exceeds_cap);
        assert!(!r.pet_rent_exceeds_cap);
    }

    #[test]
    fn unknown_state_falls_back_to_no_rule() {
        let r = check(&input("XX", 100_000, 50_000, 200_000));
        assert_eq!(r.regime, PetFeeRegime::NoStateRule);
        assert!(!r.pet_deposit_exceeds_cap);
    }

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&input("co", 30_100, 3_500, 200_000));
        assert!(r.pet_deposit_exceeds_cap);
    }

    #[test]
    fn co_at_1_5_pct_exact_boundary_complies_high_rent() {
        // 1.5% × $4000 = $60 = cap → $60/mo rent complies, $61 violates.
        let r = check(&input("CO", 0, 6_000, 400_000));
        assert!(!r.pet_rent_exceeds_cap);
        let r2 = check(&input("CO", 0, 6_100, 400_000));
        assert!(r2.pet_rent_exceeds_cap);
    }

    #[test]
    fn co_pct_rounds_down_at_low_rent() {
        // 1.5% × $1 (100 cents) = 1.5 cents → integer math rounds to 0.
        // Then cap = max($35, 0) = $35.
        let r = check(&input("CO", 0, 3_500, 100));
        assert_eq!(r.effective_pet_rent_cap_cents, Some(3_500));
    }

    #[test]
    fn coverage_is_all_50_states_plus_dc() {
        let mut codes: Vec<&'static str> = RULES.keys().copied().collect();
        codes.sort_unstable();
        assert_eq!(codes.len(), 51, "expected 50 states + DC, got {}", codes.len());
    }

    #[test]
    fn citations_never_empty() {
        for (code, rule) in RULES.iter() {
            assert!(
                !rule.citation.is_empty(),
                "state {code} missing citation"
            );
        }
    }

    #[test]
    fn co_full_violation_path_compounded() {
        // $400 deposit ($100 over) + $50/mo rent ($15 over at $1k rent).
        let r = check(&input("CO", 40_000, 5_000, 100_000));
        assert!(r.pet_deposit_exceeds_cap);
        assert!(r.pet_rent_exceeds_cap);
        assert_eq!(r.pet_deposit_violation_amount_cents, 10_000);
        assert_eq!(r.pet_rent_violation_amount_cents, 1_500);
    }

    #[test]
    fn ma_zero_pet_deposit_with_pet_rent_complies() {
        // MA: no pet deposit + monthly pet rent permitted under case law.
        let r = check(&input("MA", 0, 7_500, 200_000));
        assert!(!r.pet_deposit_exceeds_cap);
        // No pet rent cap in MA — any amount of monthly pet rent
        // complies under the module.
        assert!(!r.pet_rent_exceeds_cap);
    }

    #[test]
    fn note_describes_co_explicit_cap_path() {
        let r = check(&input("CO", 30_000, 3_500, 200_000));
        assert!(r.note.contains("explicit caps"));
    }

    #[test]
    fn note_describes_ma_ban_path() {
        let r = check(&input("MA", 0, 0, 200_000));
        assert!(r.note.contains("No separate pet deposit allowed"));
    }
}
