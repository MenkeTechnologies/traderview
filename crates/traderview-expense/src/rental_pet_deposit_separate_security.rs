//! Rental property pet deposit and separate security
//! charge compliance — when a trader-landlord wants to
//! collect a pet deposit, pet rent, or pet fee separate
//! from the general security deposit, what statutory
//! caps + disclosure + refundability rules apply? Trader-
//! landlord operational concern: post-2024 California
//! framework folds all pet deposits into a single security
//! deposit cap; misclassifying a "pet deposit" as separate
//! from the security deposit creates per-violation
//! statutory damages and refund liability. Distinct from
//! siblings `pet_fees` (general pet fee framework),
//! `security_deposit_caps` (general security deposit
//! caps), `rental_application_denial_disclosure`
//! (screening).
//!
//! **Four regimes**:
//!
//! **California — Cal. Civ. Code § 1950.5 + AB 12 of 2023
//! (eff. July 1, 2024) + SB 611 of 2023 (eff. July 1,
//! 2025)**:
//! - Pet deposits **fold into single security deposit
//!   cap**; no separate pet-deposit category.
//! - **Standard cap: 1 month's rent** (furnished or
//!   unfurnished, unified post-AB 12).
//! - **Small-landlord exception: 2 months rent** for
//!   natural persons owning no more than 2 residential
//!   rental properties with collectively no more than 4
//!   units.
//! - Monthly pet rent is permitted as recurring fee
//!   (not a deposit) IF clearly disclosed upfront in
//!   rental agreement.
//! - SB 611 (eff. July 1, 2025) prohibits separate
//!   security deposit surcharge on military tenants;
//!   requires itemized disclosure of all fees.
//! - § 1950.5(g) — refundable; landlord must return
//!   within 21 days with itemized statement.
//!
//! **Washington — RCW 59.18.260 + RCW 59.18.285**:
//! - **Pet damage deposit cap: $150**.
//! - Non-refundable pet fee permitted if clearly
//!   disclosed as non-refundable AND separate from
//!   security deposit (RCW 59.18.285).
//! - Distinction matters: pet "damage deposit" capped at
//!   $150; pet "non-refundable fee" uncapped but must be
//!   labeled non-refundable.
//! - Monthly pet rent permitted as recurring charge.
//!
//! **New York — NY GOL § 7-103 + NYC Pet Law (NYC Admin
//! Code § 27-2009.1)**:
//! - **ONE security deposit maximum: one month's rent;
//!   FULLY REFUNDABLE**.
//! - **NO separate pet deposit permitted** (distinct from
//!   CA single-cap framework — NY explicitly prohibits a
//!   pet-labeled deposit category).
//! - Monthly pet rent permitted only if clearly in lease;
//!   **prohibited in rent-stabilized apartments** under NY
//!   rent regulation.
//! - HSTPA of 2019 codified one-month cap with refund
//!   requirement.
//!
//! **Texas — common law + Tex. Prop. Code §§ 92.101-
//! 92.110**:
//! - **No statutory cap** on pet deposit or pet fee.
//! - Pet deposit + pet fee + pet rent may be charged
//!   simultaneously.
//! - Refundable unless lease specifies non-refundable.
//! - § 92.103 — landlord must return security deposit
//!   within 30 days of surrender + forwarding address.
//! - Most permissive regime among comparators.
//!
//! Citations: Cal. Civ. Code § 1950.5 (AB 12 of 2023 + SB
//! 611 of 2023); RCW 59.18.260 + RCW 59.18.285; NY GOL §
//! 7-103; NYC Admin Code § 27-2009.1; NY HSTPA of 2019;
//! Tex. Prop. Code §§ 92.101-92.110.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    Washington,
    NewYork,
    Texas,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PetCharge {
    /// Refundable pet damage deposit (counts against
    /// security-deposit cap in CA + NY).
    RefundablePetDeposit,
    /// Non-refundable one-time pet fee (permitted in WA +
    /// TX; not permitted in NY).
    NonRefundablePetFee,
    /// Monthly recurring pet rent (permitted in all four
    /// regimes if disclosed in lease).
    MonthlyPetRent,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalPetDepositSeparateSecurityInput {
    pub regime: Regime,
    /// Monthly rent in cents.
    pub monthly_rent_cents: u64,
    /// General security deposit charged (excluding pet
    /// component) in cents.
    pub general_security_deposit_cents: u64,
    /// Pet-related charge amount in cents.
    pub pet_charge_cents: u64,
    /// Type of pet charge.
    pub pet_charge_type: PetCharge,
    /// Whether landlord is a small-landlord (natural person,
    /// ≤ 2 rental properties, ≤ 4 units collectively) for
    /// CA AB 12 exception.
    pub ca_small_landlord_exception: bool,
    /// Whether pet rent is disclosed upfront in lease.
    pub disclosed_in_lease: bool,
    /// Whether unit is rent-stabilized (NY pet rent
    /// prohibition trigger).
    pub nyc_rent_stabilized: bool,
    /// Whether non-refundable fee is clearly labeled
    /// non-refundable (WA requirement).
    pub clearly_labeled_non_refundable: bool,
    /// Whether tenant is military (CA SB 611 surcharge
    /// prohibition trigger).
    pub tenant_military: bool,
    /// Whether landlord surcharged military tenant (CA SB
    /// 611 violation trigger).
    pub military_surcharge_imposed: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalPetDepositSeparateSecurityResult {
    pub deposit_within_cap: bool,
    pub separate_pet_deposit_permitted: bool,
    pub effective_cap_cents: u64,
    pub total_deposit_cents: u64,
    pub monthly_pet_rent_permitted: bool,
    pub non_refundable_fee_permitted: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(
    input: &RentalPetDepositSeparateSecurityInput,
) -> RentalPetDepositSeparateSecurityResult {
    match input.regime {
        Regime::California => check_ca(input),
        Regime::Washington => check_wa(input),
        Regime::NewYork => check_ny(input),
        Regime::Texas => check_tx(input),
    }
}

fn check_ca(
    input: &RentalPetDepositSeparateSecurityInput,
) -> RentalPetDepositSeparateSecurityResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Cal. Civ. Code § 1950.5 + AB 12 of 2023 (eff. July 1, 2024) — pet deposits fold into single security deposit cap; no separate pet-deposit category".to_string(),
        "Cal. Civ. Code § 1950.5(c) — standard cap 1 month's rent (furnished or unfurnished; unified post-AB 12)".to_string(),
        "Cal. Civ. Code § 1950.5(c)(4) — small-landlord exception 2 months rent for natural persons owning ≤ 2 residential rental properties with collectively ≤ 4 units".to_string(),
        "Cal. SB 611 of 2023 (eff. July 1, 2025) — prohibits separate security deposit surcharge on military tenants; requires itemized disclosure of all fees".to_string(),
        "Cal. Civ. Code § 1950.5(g) — security deposit fully refundable; landlord must return within 21 days with itemized statement".to_string(),
    ];

    let monthly_rent = input.monthly_rent_cents;
    let multiplier: u64 = if input.ca_small_landlord_exception { 2 } else { 1 };
    let effective_cap: u64 = monthly_rent.saturating_mul(multiplier);

    let pet_counts_against_cap = matches!(
        input.pet_charge_type,
        PetCharge::RefundablePetDeposit | PetCharge::NonRefundablePetFee
    );

    let total_deposit = if pet_counts_against_cap {
        input.general_security_deposit_cents.saturating_add(input.pet_charge_cents)
    } else {
        input.general_security_deposit_cents
    };

    if total_deposit > effective_cap {
        violations.push(
            "Cal. Civ. Code § 1950.5(c) + AB 12 of 2023 — combined security deposit (including pet component) exceeds 1-month cap (or 2-month small-landlord exception)".to_string(),
        );
    }

    if input.tenant_military && input.military_surcharge_imposed {
        violations.push(
            "Cal. SB 611 of 2023 — prohibited surcharge on military tenant; standard security deposit cap applies regardless of tenant military status".to_string(),
        );
    }

    let monthly_rent_permitted = matches!(input.pet_charge_type, PetCharge::MonthlyPetRent)
        && input.disclosed_in_lease;
    if matches!(input.pet_charge_type, PetCharge::MonthlyPetRent) && !input.disclosed_in_lease {
        violations.push(
            "Cal. Civ. Code § 1950.5 + SB 611 — monthly pet rent permitted only if clearly disclosed upfront in rental agreement".to_string(),
        );
    }

    RentalPetDepositSeparateSecurityResult {
        deposit_within_cap: total_deposit <= effective_cap,
        separate_pet_deposit_permitted: false,
        effective_cap_cents: effective_cap,
        total_deposit_cents: total_deposit,
        monthly_pet_rent_permitted: monthly_rent_permitted
            || !matches!(input.pet_charge_type, PetCharge::MonthlyPetRent),
        non_refundable_fee_permitted: false,
        violations,
        citation: "Cal. Civ. Code § 1950.5 (AB 12 of 2023 + SB 611 of 2023)",
        notes,
    }
}

fn check_wa(
    input: &RentalPetDepositSeparateSecurityInput,
) -> RentalPetDepositSeparateSecurityResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "RCW 59.18.260 — Washington security deposit framework; pet damage deposit capped at $150".to_string(),
        "RCW 59.18.285 — non-refundable pet fee permitted if clearly disclosed as non-refundable AND separate from security deposit".to_string(),
        "Washington distinction: pet damage deposit (refundable) capped at $150; pet non-refundable fee uncapped but must be labeled non-refundable".to_string(),
        "Monthly pet rent permitted as recurring charge if disclosed in lease".to_string(),
        "RCW 59.18.260 — security deposit framework requires landlord to provide written checklist documenting unit condition before charging deposit".to_string(),
    ];

    const WA_PET_DEPOSIT_CAP_CENTS: u64 = 15_000;

    let pet_deposit_within_cap = match input.pet_charge_type {
        PetCharge::RefundablePetDeposit => input.pet_charge_cents <= WA_PET_DEPOSIT_CAP_CENTS,
        _ => true,
    };

    if matches!(input.pet_charge_type, PetCharge::RefundablePetDeposit)
        && !pet_deposit_within_cap
    {
        violations.push(
            "RCW 59.18.260 — refundable pet damage deposit capped at $150".to_string(),
        );
    }

    let non_refundable_compliant = match input.pet_charge_type {
        PetCharge::NonRefundablePetFee => input.clearly_labeled_non_refundable,
        _ => true,
    };

    if matches!(input.pet_charge_type, PetCharge::NonRefundablePetFee)
        && !input.clearly_labeled_non_refundable
    {
        violations.push(
            "RCW 59.18.285 — non-refundable pet fee must be CLEARLY LABELED non-refundable AND separate from security deposit".to_string(),
        );
    }

    let total_deposit = input
        .general_security_deposit_cents
        .saturating_add(input.pet_charge_cents);

    RentalPetDepositSeparateSecurityResult {
        deposit_within_cap: pet_deposit_within_cap,
        separate_pet_deposit_permitted: true,
        effective_cap_cents: WA_PET_DEPOSIT_CAP_CENTS,
        total_deposit_cents: total_deposit,
        monthly_pet_rent_permitted: true,
        non_refundable_fee_permitted: non_refundable_compliant,
        violations,
        citation: "RCW 59.18.260 + RCW 59.18.285",
        notes,
    }
}

fn check_ny(
    input: &RentalPetDepositSeparateSecurityInput,
) -> RentalPetDepositSeparateSecurityResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "NY GOL § 7-103 (HSTPA of 2019) — ONE security deposit maximum one month's rent, fully refundable; NO separate pet deposit permitted".to_string(),
        "NYC Admin Code § 27-2009.1 (NYC Pet Law) — landlord cannot prohibit pets after 90-day open-and-notorious harboring; cannot collect separate pet deposit".to_string(),
        "NY distinction from CA single-cap framework — NY explicitly PROHIBITS pet-labeled deposit category, not just folds it in".to_string(),
        "Monthly pet rent permitted only if clearly in lease; PROHIBITED in rent-stabilized apartments under NY rent regulation".to_string(),
        "HSTPA of 2019 codified one-month cap with refund requirement + 14-day return deadline + itemized statement".to_string(),
    ];

    let monthly_rent = input.monthly_rent_cents;

    if matches!(input.pet_charge_type, PetCharge::RefundablePetDeposit)
        && input.pet_charge_cents > 0
    {
        violations.push(
            "NY GOL § 7-103 — separate pet deposit NOT permitted; all security must be in single one-month-rent security deposit".to_string(),
        );
    }

    if matches!(input.pet_charge_type, PetCharge::NonRefundablePetFee)
        && input.pet_charge_cents > 0
    {
        violations.push(
            "NY GOL § 7-103 — non-refundable pet fee NOT permitted (NY security deposits must be fully refundable)".to_string(),
        );
    }

    let general_within_cap = input.general_security_deposit_cents <= monthly_rent;
    if !general_within_cap {
        violations.push(
            "NY GOL § 7-103 — security deposit exceeds one month's rent (HSTPA of 2019 statutory cap)".to_string(),
        );
    }

    if matches!(input.pet_charge_type, PetCharge::MonthlyPetRent)
        && input.nyc_rent_stabilized
        && input.pet_charge_cents > 0
    {
        violations.push(
            "NY rent stabilization — monthly pet rent PROHIBITED in rent-stabilized apartments (hidden-rent doctrine)".to_string(),
        );
    }

    let monthly_pet_rent_permitted = matches!(input.pet_charge_type, PetCharge::MonthlyPetRent)
        && input.disclosed_in_lease
        && !input.nyc_rent_stabilized;

    RentalPetDepositSeparateSecurityResult {
        deposit_within_cap: general_within_cap,
        separate_pet_deposit_permitted: false,
        effective_cap_cents: monthly_rent,
        total_deposit_cents: input.general_security_deposit_cents,
        monthly_pet_rent_permitted: monthly_pet_rent_permitted
            || !matches!(input.pet_charge_type, PetCharge::MonthlyPetRent),
        non_refundable_fee_permitted: false,
        violations,
        citation: "NY GOL § 7-103 (HSTPA of 2019); NYC Admin Code § 27-2009.1",
        notes,
    }
}

fn check_tx(
    input: &RentalPetDepositSeparateSecurityInput,
) -> RentalPetDepositSeparateSecurityResult {
    let violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Tex. Prop. Code §§ 92.101-92.110 — no statutory cap on pet deposit or pet fee".to_string(),
        "Texas — pet deposit + pet fee + pet rent may be charged simultaneously; no statewide cap".to_string(),
        "Texas — refundable unless lease specifies non-refundable".to_string(),
        "Tex. Prop. Code § 92.103 — landlord must return security deposit within 30 days of surrender plus forwarding address".to_string(),
        "Texas most permissive regime among comparators; CA / NY / WA all impose statutory caps or category restrictions".to_string(),
    ];

    let total_deposit = input
        .general_security_deposit_cents
        .saturating_add(input.pet_charge_cents);

    RentalPetDepositSeparateSecurityResult {
        deposit_within_cap: true,
        separate_pet_deposit_permitted: true,
        effective_cap_cents: u64::MAX,
        total_deposit_cents: total_deposit,
        monthly_pet_rent_permitted: true,
        non_refundable_fee_permitted: true,
        violations,
        citation: "Tex. Prop. Code §§ 92.101-92.110",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_base() -> RentalPetDepositSeparateSecurityInput {
        RentalPetDepositSeparateSecurityInput {
            regime: Regime::California,
            monthly_rent_cents: 200_000,
            general_security_deposit_cents: 150_000,
            pet_charge_cents: 50_000,
            pet_charge_type: PetCharge::RefundablePetDeposit,
            ca_small_landlord_exception: false,
            disclosed_in_lease: true,
            nyc_rent_stabilized: false,
            clearly_labeled_non_refundable: false,
            tenant_military: false,
            military_surcharge_imposed: false,
        }
    }

    fn wa_base() -> RentalPetDepositSeparateSecurityInput {
        let mut i = ca_base();
        i.regime = Regime::Washington;
        i.pet_charge_cents = 15_000;
        i
    }

    fn ny_base() -> RentalPetDepositSeparateSecurityInput {
        let mut i = ca_base();
        i.regime = Regime::NewYork;
        i.pet_charge_cents = 0;
        i.general_security_deposit_cents = 200_000;
        i.pet_charge_type = PetCharge::MonthlyPetRent;
        i
    }

    fn tx_base() -> RentalPetDepositSeparateSecurityInput {
        let mut i = ca_base();
        i.regime = Regime::Texas;
        i
    }

    #[test]
    fn ca_within_one_month_cap_compliant() {
        let r = check(&ca_base());
        assert!(r.deposit_within_cap);
        assert_eq!(r.effective_cap_cents, 200_000);
        assert_eq!(r.total_deposit_cents, 200_000);
    }

    #[test]
    fn ca_combined_deposit_exceeds_one_month_cap_violation() {
        let mut i = ca_base();
        i.general_security_deposit_cents = 200_000;
        i.pet_charge_cents = 50_000;
        let r = check(&i);
        assert!(!r.deposit_within_cap);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1950.5(c)") && v.contains("AB 12")));
    }

    #[test]
    fn ca_small_landlord_two_month_exception_compliant() {
        let mut i = ca_base();
        i.ca_small_landlord_exception = true;
        i.general_security_deposit_cents = 350_000;
        i.pet_charge_cents = 50_000;
        let r = check(&i);
        assert!(r.deposit_within_cap);
        assert_eq!(r.effective_cap_cents, 400_000);
    }

    #[test]
    fn ca_monthly_pet_rent_does_not_count_against_cap() {
        let mut i = ca_base();
        i.pet_charge_type = PetCharge::MonthlyPetRent;
        i.general_security_deposit_cents = 200_000;
        i.pet_charge_cents = 100_000;
        let r = check(&i);
        assert!(r.deposit_within_cap);
        assert_eq!(r.total_deposit_cents, 200_000);
    }

    #[test]
    fn ca_undisclosed_pet_rent_violation() {
        let mut i = ca_base();
        i.pet_charge_type = PetCharge::MonthlyPetRent;
        i.disclosed_in_lease = false;
        let r = check(&i);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("SB 611") && v.contains("disclosed upfront")));
    }

    #[test]
    fn ca_military_surcharge_violation() {
        let mut i = ca_base();
        i.tenant_military = true;
        i.military_surcharge_imposed = true;
        let r = check(&i);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("SB 611") && v.contains("military")));
    }

    #[test]
    fn ca_no_separate_pet_deposit_invariant() {
        let r = check(&ca_base());
        assert!(!r.separate_pet_deposit_permitted);
    }

    #[test]
    fn wa_pet_deposit_at_150_cap_compliant() {
        let r = check(&wa_base());
        assert!(r.deposit_within_cap);
        assert_eq!(r.effective_cap_cents, 15_000);
    }

    #[test]
    fn wa_pet_deposit_above_150_violation() {
        let mut i = wa_base();
        i.pet_charge_cents = 15_001;
        let r = check(&i);
        assert!(!r.deposit_within_cap);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("RCW 59.18.260") && v.contains("$150")));
    }

    #[test]
    fn wa_non_refundable_fee_with_label_compliant() {
        let mut i = wa_base();
        i.pet_charge_type = PetCharge::NonRefundablePetFee;
        i.pet_charge_cents = 100_000;
        i.clearly_labeled_non_refundable = true;
        let r = check(&i);
        assert!(r.non_refundable_fee_permitted);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn wa_non_refundable_fee_without_label_violation() {
        let mut i = wa_base();
        i.pet_charge_type = PetCharge::NonRefundablePetFee;
        i.pet_charge_cents = 100_000;
        i.clearly_labeled_non_refundable = false;
        let r = check(&i);
        assert!(!r.non_refundable_fee_permitted);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("RCW 59.18.285") && v.contains("CLEARLY LABELED")));
    }

    #[test]
    fn ny_no_separate_pet_deposit_violation() {
        let mut i = ny_base();
        i.pet_charge_type = PetCharge::RefundablePetDeposit;
        i.pet_charge_cents = 50_000;
        let r = check(&i);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 7-103") && v.contains("separate pet deposit NOT permitted")));
    }

    #[test]
    fn ny_no_non_refundable_pet_fee_violation() {
        let mut i = ny_base();
        i.pet_charge_type = PetCharge::NonRefundablePetFee;
        i.pet_charge_cents = 50_000;
        let r = check(&i);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("non-refundable pet fee NOT permitted")));
    }

    #[test]
    fn ny_security_deposit_exceeds_one_month_violation() {
        let mut i = ny_base();
        i.general_security_deposit_cents = 250_000;
        let r = check(&i);
        assert!(!r.deposit_within_cap);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("HSTPA")));
    }

    #[test]
    fn ny_rent_stabilized_pet_rent_violation() {
        let mut i = ny_base();
        i.nyc_rent_stabilized = true;
        i.pet_charge_cents = 5_000;
        let r = check(&i);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("rent-stabilized") && v.contains("PROHIBITED")));
    }

    #[test]
    fn ny_non_rent_stabilized_pet_rent_compliant() {
        let mut i = ny_base();
        i.nyc_rent_stabilized = false;
        i.pet_charge_cents = 5_000;
        let r = check(&i);
        assert!(r.monthly_pet_rent_permitted);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn tx_no_statutory_cap_any_amount_compliant() {
        let mut i = tx_base();
        i.general_security_deposit_cents = 500_000;
        i.pet_charge_cents = 1_000_000;
        let r = check(&i);
        assert!(r.deposit_within_cap);
        assert_eq!(r.effective_cap_cents, u64::MAX);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn tx_simultaneous_charges_permitted() {
        let mut i = tx_base();
        i.pet_charge_type = PetCharge::NonRefundablePetFee;
        let r = check(&i);
        assert!(r.non_refundable_fee_permitted);
        assert!(r.monthly_pet_rent_permitted);
    }

    #[test]
    fn citation_pins_ca_authority() {
        let r = check(&ca_base());
        assert!(r.citation.contains("§ 1950.5"));
        assert!(r.citation.contains("AB 12 of 2023"));
        assert!(r.citation.contains("SB 611 of 2023"));
    }

    #[test]
    fn citation_pins_wa_authority() {
        let r = check(&wa_base());
        assert!(r.citation.contains("RCW 59.18.260"));
        assert!(r.citation.contains("RCW 59.18.285"));
    }

    #[test]
    fn citation_pins_ny_authority() {
        let r = check(&ny_base());
        assert!(r.citation.contains("§ 7-103"));
        assert!(r.citation.contains("HSTPA"));
        assert!(r.citation.contains("§ 27-2009.1"));
    }

    #[test]
    fn citation_pins_tx_authority() {
        let r = check(&tx_base());
        assert!(r.citation.contains("§§ 92.101"));
    }

    #[test]
    fn note_pins_ca_july_2024_effective_date() {
        let r = check(&ca_base());
        assert!(r.notes.iter().any(|n| n.contains("July 1, 2024")
            && n.contains("AB 12")));
    }

    #[test]
    fn note_pins_ca_sb_611_july_2025_effective_date() {
        let r = check(&ca_base());
        assert!(r.notes.iter().any(|n| n.contains("July 1, 2025")
            && n.contains("SB 611")));
    }

    #[test]
    fn note_pins_wa_150_dollar_cap() {
        let r = check(&wa_base());
        assert!(r.notes.iter().any(|n| n.contains("$150")));
    }

    #[test]
    fn note_pins_ny_hstpa_one_month_cap() {
        let r = check(&ny_base());
        assert!(r.notes.iter().any(|n| n.contains("HSTPA")
            && n.contains("one month")));
    }

    #[test]
    fn note_pins_tx_most_permissive_regime() {
        let r = check(&tx_base());
        assert!(r.notes.iter().any(|n| n.contains("most permissive")));
    }

    #[test]
    fn tx_uniquely_permits_simultaneous_all_three_invariant() {
        let r_ca = check(&ca_base());
        let r_wa = check(&wa_base());
        let r_ny = check(&ny_base());
        let r_tx = check(&tx_base());
        assert!(r_tx.non_refundable_fee_permitted);
        assert!(!r_ca.non_refundable_fee_permitted);
        assert!(!r_ny.non_refundable_fee_permitted);
        assert!(r_wa.non_refundable_fee_permitted);
    }

    #[test]
    fn ny_uniquely_prohibits_separate_pet_categories_invariant() {
        let r_ca = check(&ca_base());
        let r_wa = check(&wa_base());
        let r_ny = check(&ny_base());
        let r_tx = check(&tx_base());
        assert!(!r_ny.separate_pet_deposit_permitted);
        assert!(!r_ca.separate_pet_deposit_permitted);
        assert!(r_wa.separate_pet_deposit_permitted);
        assert!(r_tx.separate_pet_deposit_permitted);
    }

    #[test]
    fn ca_small_landlord_exception_doubles_cap_invariant() {
        let mut i_normal = ca_base();
        i_normal.ca_small_landlord_exception = false;
        let r_normal = check(&i_normal);

        let mut i_small = ca_base();
        i_small.ca_small_landlord_exception = true;
        let r_small = check(&i_small);

        assert_eq!(r_small.effective_cap_cents, r_normal.effective_cap_cents * 2);
    }

    #[test]
    fn pet_charge_type_truth_table_for_ca() {
        for (charge_type, count_against_cap) in [
            (PetCharge::RefundablePetDeposit, true),
            (PetCharge::NonRefundablePetFee, true),
            (PetCharge::MonthlyPetRent, false),
        ] {
            let mut i = ca_base();
            i.pet_charge_type = charge_type;
            let r = check(&i);
            let expected_total = if count_against_cap {
                i.general_security_deposit_cents + i.pet_charge_cents
            } else {
                i.general_security_deposit_cents
            };
            assert_eq!(r.total_deposit_cents, expected_total);
        }
    }

    #[test]
    fn defensive_overflow_saturating_arithmetic() {
        let mut i = ca_base();
        i.general_security_deposit_cents = u64::MAX;
        i.pet_charge_cents = u64::MAX;
        let r = check(&i);
        assert_eq!(r.total_deposit_cents, u64::MAX);
    }
}
