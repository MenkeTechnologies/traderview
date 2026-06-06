//! Security-deposit return notice + itemized-deduction compliance framework.
//!
//! Every US state with a residential tenancy code imposes a statutory deadline for
//! returning a tenant's security deposit after move-out and requires the landlord
//! to deliver an itemized written statement of any deductions claimed. Failure to
//! comply forfeits the landlord's right to retain any portion of the deposit and
//! exposes the landlord to multiplied damages (2× to 3× the deposit) plus attorney
//! fees in many jurisdictions.
//!
//! Deadlines and remedies vary sharply. CA Civ. Code § 1950.5(g) requires 21 calendar
//! days from move-out plus itemized statement plus receipts for deductions of $125 or
//! more, with bad-faith retention triggering statutory damages up to 2x deposit per
//! § 1950.5(l). NY Gen. Oblig. Law § 7-108(1-a)(e) requires 14 days plus itemized list
//! of deductions, with failure forfeiting retention right AND tenant recovering 2x
//! damages. WA RCW 59.18.280 requires 30 days (extended from 21 days) plus full
//! itemized statement plus documentation, with failure forfeiting retention right
//! plus actual damages plus court costs plus attorney fees. TX Prop. Code § 92.103
//! requires 30 days plus itemized written description, with bad-faith retention
//! triggering $100 plus 3x the wrongfully withheld amount plus attorney fees under
//! § 92.109. FL Stat. § 83.49(3)(a) requires 15 days if no claim or 30 days if claim
//! asserted by CERTIFIED-MAIL notice plus itemized statement. IL Chicago RLTO
//! § 5-12-080 requires 45 days (Chicago) and 765 ILCS 710/1 requires 30 days (IL
//! statewide for buildings with 5+ units); Chicago RLTO triggers 2x deposit plus
//! attorney fees for violation. MA Gen. L. ch. 186 § 15B(4)(iii) requires 30 days
//! plus sworn statement of damages, with failure triggering 3x deposit plus interest
//! plus attorney fees under § 15B(7).
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - ipropertymanagement.com/laws/security-deposits
//! - nolo.com/legal-encyclopedia/chart-deadline-returning-security-deposits-29018.html
//! - app.leg.wa.gov/rcw/default.aspx?cite=59.18.280
//! - leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?lawCode=CIV&sectionNum=1950.5

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    NewYork,
    Washington,
    Texas,
    Florida,
    IllinoisChicagoRlto,
    IllinoisStatewide,
    Massachusetts,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeductionsClaimedStatus {
    NoDeductionsClaimedFullReturnDue,
    DeductionsClaimedItemizedStatementProvided,
    DeductionsClaimedNoItemizedStatementProvided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryMethod {
    PersonalDeliveryOrFirstClassMail,
    CertifiedMailRequiredFlorida,
    EmailWithoutTenantConsent,
    NoDelivery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantWithinStatutoryWindow,
    CompliantNoDeductionsFullReturn,
    LateButReturnedRiskOfForfeitureOnly,
    DeductionsWithoutItemizedStatementForfeitsRetentionRight,
    BadFaithRetentionDoubleOrTripleDamages,
    NoDeliveryFullForfeitureAndStatutoryDamages,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub deductions_claimed_status: DeductionsClaimedStatus,
    pub delivery_method: DeliveryMethod,
    pub days_after_move_out_deposit_returned: u32,
    pub deposit_held_cents: u64,
    pub deductions_amount_claimed_cents: u64,
    pub bad_faith_alleged: bool,
}

pub type RentalSecurityDepositReturnNoticeInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub statutory_deadline_days: u32,
    pub estimated_landlord_exposure_cents: u64,
    pub note: String,
}

pub type RentalSecurityDepositReturnNoticeOutput = Output;
pub type RentalSecurityDepositReturnNoticeResult = Output;

const CA_DEADLINE_DAYS: u32 = 21;
const NY_DEADLINE_DAYS: u32 = 14;
const WA_DEADLINE_DAYS: u32 = 30;
const TX_DEADLINE_DAYS: u32 = 30;
const FL_NO_CLAIM_DEADLINE_DAYS: u32 = 15;
const FL_CLAIM_DEADLINE_DAYS: u32 = 30;
const IL_CHICAGO_DEADLINE_DAYS: u32 = 45;
const IL_STATEWIDE_DEADLINE_DAYS: u32 = 30;
const MA_DEADLINE_DAYS: u32 = 30;
const DEFAULT_DEADLINE_DAYS: u32 = 30;
#[allow(dead_code)]
const CA_RECEIPT_THRESHOLD_CENTS: u64 = 12_500;
const TX_BAD_FAITH_BASE_PENALTY_CENTS: u64 = 10_000;
const TX_BAD_FAITH_MULTIPLIER: u64 = 3;
const NY_BAD_FAITH_MULTIPLIER: u64 = 2;
const MA_BAD_FAITH_MULTIPLIER: u64 = 3;
const CA_BAD_FAITH_MULTIPLIER: u64 = 2;
const IL_CHICAGO_BAD_FAITH_MULTIPLIER: u64 = 2;

#[must_use]
pub fn check(input: &Input) -> Output {
    let deadline = statutory_deadline(input.jurisdiction, input.deductions_claimed_status);

    if matches!(input.delivery_method, DeliveryMethod::NoDelivery) {
        let exposure = compute_full_forfeiture_exposure(input);
        return Output {
            severity: Severity::NoDeliveryFullForfeitureAndStatutoryDamages,
            statutory_deadline_days: deadline,
            estimated_landlord_exposure_cents: exposure,
            note: format!(
                "Landlord made NO delivery within statutory window. Full forfeiture of \
                 retention right + statutory multiplied damages where state law provides. \
                 Estimated exposure ${} reflects deposit (${}) + multiplied-damages \
                 estimate per jurisdiction. Most state statutes forfeit the right to \
                 retain ANY portion of the deposit when the landlord fails to deliver \
                 the itemized statement within the statutory window.",
                exposure / 100,
                input.deposit_held_cents / 100
            ),
        };
    }

    if matches!(input.jurisdiction, Jurisdiction::Florida)
        && !matches!(
            input.delivery_method,
            DeliveryMethod::CertifiedMailRequiredFlorida
        )
        && matches!(
            input.deductions_claimed_status,
            DeductionsClaimedStatus::DeductionsClaimedItemizedStatementProvided
        )
    {
        let exposure = input.deposit_held_cents;
        return Output {
            severity: Severity::DeductionsWithoutItemizedStatementForfeitsRetentionRight,
            statutory_deadline_days: deadline,
            estimated_landlord_exposure_cents: exposure,
            note: format!(
                "Florida Stat. § 83.49(3)(a) requires CERTIFIED MAIL notice with itemized \
                 statement when claiming deductions. First-class mail or personal delivery \
                 of deduction claim fails Florida's specific method requirement. Landlord \
                 forfeits retention right; deposit ${} fully due to tenant.",
                input.deposit_held_cents / 100
            ),
        };
    }

    if input.days_after_move_out_deposit_returned > deadline {
        let exposure = compute_full_forfeiture_exposure(input);
        return Output {
            severity: Severity::BadFaithRetentionDoubleOrTripleDamages,
            statutory_deadline_days: deadline,
            estimated_landlord_exposure_cents: exposure,
            note: format!(
                "Landlord exceeded statutory {}-day deadline by returning deposit at day {}. \
                 Statutory bad-faith presumption triggers in {} (CA Civ. Code § 1950.5(l) 2x; \
                 TX Prop. Code § 92.109 $100 + 3x of wrongful withholding; NY GOL § 7-108 2x; \
                 MA Gen. L. ch. 186 § 15B(7) 3x; IL Chicago RLTO § 5-12-080 2x). Estimated \
                 exposure ${} reflects multiplied damages per applicable jurisdiction.",
                deadline,
                input.days_after_move_out_deposit_returned,
                jurisdiction_label(input.jurisdiction),
                exposure / 100
            ),
        };
    }

    if matches!(
        input.deductions_claimed_status,
        DeductionsClaimedStatus::DeductionsClaimedNoItemizedStatementProvided
    ) {
        let exposure = input.deposit_held_cents;
        return Output {
            severity: Severity::DeductionsWithoutItemizedStatementForfeitsRetentionRight,
            statutory_deadline_days: deadline,
            estimated_landlord_exposure_cents: exposure,
            note: format!(
                "Landlord claimed deductions WITHOUT providing the required itemized written \
                 statement. Every surveyed jurisdiction conditions the right to retain on \
                 timely itemization. Forfeiture of retention right; deposit ${} fully due to \
                 tenant. CA additionally requires receipts/invoices for deductions ≥ $125 \
                 (CA Civ. Code § 1950.5(g)(2)).",
                input.deposit_held_cents / 100
            ),
        };
    }

    if input.bad_faith_alleged {
        let exposure = compute_full_forfeiture_exposure(input);
        return Output {
            severity: Severity::BadFaithRetentionDoubleOrTripleDamages,
            statutory_deadline_days: deadline,
            estimated_landlord_exposure_cents: exposure,
            note: format!(
                "Bad-faith retention alleged. CA § 1950.5(l) 2× deposit + actual damages; \
                 TX § 92.109 $100 + 3× wrongfully withheld + attorney fees; MA § 15B(7) 3× \
                 deposit + interest + attorney fees; NY GOL § 7-108 2× deposit; IL Chicago \
                 RLTO § 5-12-080 2× deposit. Estimated exposure ${} reflects multiplied \
                 damages per applicable jurisdiction.",
                exposure / 100
            ),
        };
    }

    if matches!(
        input.deductions_claimed_status,
        DeductionsClaimedStatus::NoDeductionsClaimedFullReturnDue
    ) {
        return Output {
            severity: Severity::CompliantNoDeductionsFullReturn,
            statutory_deadline_days: deadline,
            estimated_landlord_exposure_cents: 0,
            note: format!(
                "Compliant: full deposit ${} returned within {}-day statutory window with \
                 no deductions claimed. No additional landlord exposure.",
                input.deposit_held_cents / 100,
                deadline
            ),
        };
    }

    Output {
        severity: Severity::CompliantWithinStatutoryWindow,
        statutory_deadline_days: deadline,
        estimated_landlord_exposure_cents: 0,
        note: format!(
            "Compliant: deductions claimed (${}) with itemized written statement delivered \
             within {}-day statutory window. Net return to tenant = deposit (${}) - \
             deductions (${}) = ${}. Retain proof of delivery: USPS certificate or delivery \
             confirmation, tenant signature, contemporaneous email log.",
            input.deductions_amount_claimed_cents / 100,
            deadline,
            input.deposit_held_cents / 100,
            input.deductions_amount_claimed_cents / 100,
            input
                .deposit_held_cents
                .saturating_sub(input.deductions_amount_claimed_cents)
                / 100
        ),
    }
}

fn statutory_deadline(
    jurisdiction: Jurisdiction,
    deductions_claimed_status: DeductionsClaimedStatus,
) -> u32 {
    match jurisdiction {
        Jurisdiction::California => CA_DEADLINE_DAYS,
        Jurisdiction::NewYork => NY_DEADLINE_DAYS,
        Jurisdiction::Washington => WA_DEADLINE_DAYS,
        Jurisdiction::Texas => TX_DEADLINE_DAYS,
        Jurisdiction::Florida => {
            if matches!(
                deductions_claimed_status,
                DeductionsClaimedStatus::NoDeductionsClaimedFullReturnDue
            ) {
                FL_NO_CLAIM_DEADLINE_DAYS
            } else {
                FL_CLAIM_DEADLINE_DAYS
            }
        }
        Jurisdiction::IllinoisChicagoRlto => IL_CHICAGO_DEADLINE_DAYS,
        Jurisdiction::IllinoisStatewide => IL_STATEWIDE_DEADLINE_DAYS,
        Jurisdiction::Massachusetts => MA_DEADLINE_DAYS,
        Jurisdiction::Default => DEFAULT_DEADLINE_DAYS,
    }
}

fn jurisdiction_label(jurisdiction: Jurisdiction) -> &'static str {
    match jurisdiction {
        Jurisdiction::California => "California",
        Jurisdiction::NewYork => "New York",
        Jurisdiction::Washington => "Washington",
        Jurisdiction::Texas => "Texas",
        Jurisdiction::Florida => "Florida",
        Jurisdiction::IllinoisChicagoRlto => "Chicago RLTO",
        Jurisdiction::IllinoisStatewide => "Illinois (statewide)",
        Jurisdiction::Massachusetts => "Massachusetts",
        Jurisdiction::Default => "Default",
    }
}

fn compute_full_forfeiture_exposure(input: &Input) -> u64 {
    match input.jurisdiction {
        Jurisdiction::Texas => TX_BAD_FAITH_BASE_PENALTY_CENTS.saturating_add(
            input
                .deductions_amount_claimed_cents
                .saturating_mul(TX_BAD_FAITH_MULTIPLIER),
        ),
        Jurisdiction::Massachusetts => input
            .deposit_held_cents
            .saturating_mul(MA_BAD_FAITH_MULTIPLIER),
        Jurisdiction::California => input
            .deposit_held_cents
            .saturating_mul(CA_BAD_FAITH_MULTIPLIER),
        Jurisdiction::NewYork => input
            .deposit_held_cents
            .saturating_mul(NY_BAD_FAITH_MULTIPLIER),
        Jurisdiction::IllinoisChicagoRlto => input
            .deposit_held_cents
            .saturating_mul(IL_CHICAGO_BAD_FAITH_MULTIPLIER),
        _ => input.deposit_held_cents,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_ca() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            deductions_claimed_status:
                DeductionsClaimedStatus::DeductionsClaimedItemizedStatementProvided,
            delivery_method: DeliveryMethod::PersonalDeliveryOrFirstClassMail,
            days_after_move_out_deposit_returned: 20,
            deposit_held_cents: 3_000_00,
            deductions_amount_claimed_cents: 500_00,
            bad_faith_alleged: false,
        }
    }

    #[test]
    fn california_compliant_within_21_day_window() {
        let input = base_ca();
        let output = check(&input);
        assert_eq!(output.severity, Severity::CompliantWithinStatutoryWindow);
        assert_eq!(output.statutory_deadline_days, 21);
        assert_eq!(output.estimated_landlord_exposure_cents, 0);
    }

    #[test]
    fn california_22_days_late_triggers_bad_faith_2x_damages() {
        let mut input = base_ca();
        input.days_after_move_out_deposit_returned = 22;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::BadFaithRetentionDoubleOrTripleDamages
        );
        // 2 × $3,000 = $6,000
        assert_eq!(output.estimated_landlord_exposure_cents, 6_000_00);
        assert!(output.note.contains("§ 1950.5(l)"));
    }

    #[test]
    fn california_at_21_day_boundary_compliant() {
        let mut input = base_ca();
        input.days_after_move_out_deposit_returned = 21;
        let output = check(&input);
        assert_eq!(output.severity, Severity::CompliantWithinStatutoryWindow);
    }

    #[test]
    fn california_no_deductions_full_return_compliant() {
        let mut input = base_ca();
        input.deductions_claimed_status = DeductionsClaimedStatus::NoDeductionsClaimedFullReturnDue;
        let output = check(&input);
        assert_eq!(output.severity, Severity::CompliantNoDeductionsFullReturn);
    }

    #[test]
    fn deductions_without_itemized_statement_forfeits_retention() {
        let mut input = base_ca();
        input.deductions_claimed_status =
            DeductionsClaimedStatus::DeductionsClaimedNoItemizedStatementProvided;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::DeductionsWithoutItemizedStatementForfeitsRetentionRight
        );
        assert_eq!(output.estimated_landlord_exposure_cents, 3_000_00);
        assert!(output.note.contains("§ 1950.5(g)(2)"));
        assert!(output.note.contains("$125"));
    }

    #[test]
    fn new_york_14_day_deadline() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::NewYork;
        let output = check(&input);
        assert_eq!(output.statutory_deadline_days, 14);
    }

    #[test]
    fn new_york_15_days_triggers_bad_faith_2x() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::NewYork;
        input.days_after_move_out_deposit_returned = 15;
        let output = check(&input);
        // 2 × $3,000 = $6,000
        assert_eq!(output.estimated_landlord_exposure_cents, 6_000_00);
        assert!(output.note.contains("NY GOL § 7-108"));
    }

    #[test]
    fn washington_30_day_deadline_extended_from_21() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Washington;
        let output = check(&input);
        assert_eq!(output.statutory_deadline_days, 30);
    }

    #[test]
    fn texas_30_day_deadline_with_bad_faith_triple_plus_100() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Texas;
        input.days_after_move_out_deposit_returned = 31;
        let output = check(&input);
        // $100 + 3 × $500 deductions = $100 + $1,500 = $1,600
        assert_eq!(output.estimated_landlord_exposure_cents, 1_600_00);
        assert!(output.note.contains("§ 92.109"));
    }

    #[test]
    fn florida_no_deductions_15_day_deadline() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Florida;
        input.deductions_claimed_status = DeductionsClaimedStatus::NoDeductionsClaimedFullReturnDue;
        let output = check(&input);
        assert_eq!(output.statutory_deadline_days, 15);
    }

    #[test]
    fn florida_with_deductions_30_day_deadline_certified_mail_required() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Florida;
        let output = check(&input);
        // Default first-class mail fails FL certified-mail requirement
        assert_eq!(
            output.severity,
            Severity::DeductionsWithoutItemizedStatementForfeitsRetentionRight
        );
        assert!(output.note.contains("CERTIFIED MAIL"));
        assert!(output.note.contains("§ 83.49(3)(a)"));
    }

    #[test]
    fn florida_certified_mail_with_deductions_compliant() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Florida;
        input.delivery_method = DeliveryMethod::CertifiedMailRequiredFlorida;
        let output = check(&input);
        assert_eq!(output.severity, Severity::CompliantWithinStatutoryWindow);
        assert_eq!(output.statutory_deadline_days, 30);
    }

    #[test]
    fn illinois_chicago_rlto_45_day_deadline() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::IllinoisChicagoRlto;
        let output = check(&input);
        assert_eq!(output.statutory_deadline_days, 45);
    }

    #[test]
    fn illinois_chicago_bad_faith_2x_damages() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::IllinoisChicagoRlto;
        input.days_after_move_out_deposit_returned = 46;
        let output = check(&input);
        // 2 × $3,000 = $6,000
        assert_eq!(output.estimated_landlord_exposure_cents, 6_000_00);
    }

    #[test]
    fn illinois_statewide_30_day_deadline() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::IllinoisStatewide;
        let output = check(&input);
        assert_eq!(output.statutory_deadline_days, 30);
    }

    #[test]
    fn massachusetts_bad_faith_3x_damages() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Massachusetts;
        input.days_after_move_out_deposit_returned = 31;
        let output = check(&input);
        // 3 × $3,000 = $9,000
        assert_eq!(output.estimated_landlord_exposure_cents, 9_000_00);
        assert!(output.note.contains("§ 15B(7)"));
    }

    #[test]
    fn no_delivery_full_forfeiture() {
        let mut input = base_ca();
        input.delivery_method = DeliveryMethod::NoDelivery;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoDeliveryFullForfeitureAndStatutoryDamages
        );
        // CA: 2 × $3,000 = $6,000
        assert_eq!(output.estimated_landlord_exposure_cents, 6_000_00);
    }

    #[test]
    fn bad_faith_flag_triggers_multiplied_damages_within_window() {
        let mut input = base_ca();
        input.bad_faith_alleged = true;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::BadFaithRetentionDoubleOrTripleDamages
        );
        // CA: 2 × $3,000 = $6,000
        assert_eq!(output.estimated_landlord_exposure_cents, 6_000_00);
    }

    #[test]
    fn ca_deadline_days_constant_pins_21() {
        assert_eq!(CA_DEADLINE_DAYS, 21);
    }

    #[test]
    fn ny_deadline_days_constant_pins_14() {
        assert_eq!(NY_DEADLINE_DAYS, 14);
    }

    #[test]
    fn wa_deadline_days_constant_pins_30() {
        assert_eq!(WA_DEADLINE_DAYS, 30);
    }

    #[test]
    fn tx_deadline_days_constant_pins_30() {
        assert_eq!(TX_DEADLINE_DAYS, 30);
    }

    #[test]
    fn fl_no_claim_deadline_constant_pins_15() {
        assert_eq!(FL_NO_CLAIM_DEADLINE_DAYS, 15);
    }

    #[test]
    fn fl_claim_deadline_constant_pins_30() {
        assert_eq!(FL_CLAIM_DEADLINE_DAYS, 30);
    }

    #[test]
    fn il_chicago_deadline_constant_pins_45() {
        assert_eq!(IL_CHICAGO_DEADLINE_DAYS, 45);
    }

    #[test]
    fn il_statewide_deadline_constant_pins_30() {
        assert_eq!(IL_STATEWIDE_DEADLINE_DAYS, 30);
    }

    #[test]
    fn ma_deadline_constant_pins_30() {
        assert_eq!(MA_DEADLINE_DAYS, 30);
    }

    #[test]
    fn ca_receipt_threshold_constant_pins_125() {
        assert_eq!(CA_RECEIPT_THRESHOLD_CENTS, 12_500);
    }

    #[test]
    fn tx_bad_faith_base_penalty_constant_pins_100() {
        assert_eq!(TX_BAD_FAITH_BASE_PENALTY_CENTS, 10_000);
    }

    #[test]
    fn tx_bad_faith_multiplier_pins_3() {
        assert_eq!(TX_BAD_FAITH_MULTIPLIER, 3);
    }

    #[test]
    fn ma_bad_faith_multiplier_pins_3() {
        assert_eq!(MA_BAD_FAITH_MULTIPLIER, 3);
    }

    #[test]
    fn very_large_deposit_no_overflow() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Massachusetts;
        input.deposit_held_cents = u64::MAX / 2;
        input.days_after_move_out_deposit_returned = 35;
        let output = check(&input);
        // saturating_mul defense
        assert!(output.estimated_landlord_exposure_cents > 0);
    }

    #[test]
    fn zero_deposit_no_panic() {
        let mut input = base_ca();
        input.deposit_held_cents = 0;
        let output = check(&input);
        assert_eq!(output.estimated_landlord_exposure_cents, 0);
    }
}
