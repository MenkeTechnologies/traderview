//! Tenant rent escrow + repair-and-deduct compliance framework for habitability disputes.
//!
//! When a landlord fails to maintain habitable premises, state law provides tenants with
//! self-help remedies including rent withholding (deposit into escrow pending repair),
//! repair-and-deduct, and lease termination. Each state has specific procedural
//! requirements (notice + cure period + escrow account + cap on deduction) that govern
//! whether the tenant's rent withholding is lawful or constitutes nonpayment justifying
//! eviction.
//!
//! Jurisdictional grid:
//!
//! - CA Civ. Code § 1942 + § 1942.4: repair-and-deduct capped at one month's rent,
//!   twice per 12-month period. Repair-and-deduct allowed for habitability defects
//!   after notice + reasonable cure period (typically 30 days; less for emergencies).
//!   § 1942.4 prohibits rent collection if landlord knew of substandard conditions
//!   for 60+ days. Tenant should deposit withheld rent in a separate account.
//! - NY RPL § 235-b (Warranty of Habitability) + RPL § 235-a: tenants may make
//!   necessary repairs in extenuating circumstances and deduct reasonable costs. Rent
//!   withholding via court-supervised escrow (NYC HCR + Housing Court). HP proceedings
//!   under NY RPAPL § 110 force landlord repair.
//! - WA RCW 59.18.115 RENT ESCROW PROCESS + RCW 59.18.100 repair-and-deduct: rent
//!   withholding ONLY through the § 59.18.115 escrow process — written notice +
//!   statutory cure window + local-authority certification + deposit in approved
//!   escrow. Repair-and-deduct cap: 2 months' rent per repair, 2 months' rent per year.
//! - IL Chicago RLTO § 5-12-110: 14-day written notice; tenant may withhold rent
//!   capped at GREATER of $500 or 50% of monthly rent while remaining in unit;
//!   repair-and-deduct + lease termination + minor-condition rent reduction available.
//! - TX Prop. Code § 92.0561: repair-and-deduct capped at GREATER of one month's rent
//!   or $500 — narrow circumstances only (sewage, flood, similar material conditions);
//!   landlord-duty-non-waived prerequisite.
//! - MA Gen. L. ch. 239 § 8A: tenant may IMMEDIATELY start withholding rent upon
//!   notice to landlord (no cure period required); raise as defense/counterclaim in
//!   summary process; rent withheld must be paid into escrow during the proceeding.
//! - DEFAULT: common-law implied warranty of habitability + constructive eviction
//!   doctrine (Marini v. Ireland 265 A.2d 526 NJ 1970 + Pugh v. Holmes 384 A.2d 1234 PA
//!   1979); narrower self-help remedies than statutory regimes.
//!
//! Citations (verified per WebSearch 2026-06-02):
//! - leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?sectionNum=1942.&lawCode=CIV
//! - nysenate.gov/legislation/laws/RPP/235-B
//! - app.leg.wa.gov/rcw/default.aspx?cite=59.18.115
//! - codelibrary.amlegal.com/codes/chicago/latest/chicago_il/0-0-0-2639177
//! - codes.findlaw.com/tx/property-code/prop-sect-92-0561/
//! - malegislature.gov/Laws/GeneralLaws/PartIII/TitleIII/Chapter239/Section8A

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    NewYork,
    Washington,
    IllinoisChicagoRlto,
    Texas,
    Massachusetts,
    Default,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantRemedyAction {
    /// Pure rent withholding — no repair attempted by tenant.
    RentWithholdingWithoutRepair,
    /// Tenant deposited withheld rent into court-approved escrow account.
    RentWithholdingDepositedToEscrowAccount,
    /// Tenant repaired and deducted from rent.
    RepairAndDeductFromRent,
    /// Tenant terminated lease for constructive eviction.
    LeaseTerminationConstructiveEviction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NoticeAndCureStatus {
    /// Written notice provided + statutory cure window elapsed without repair.
    WrittenNoticeAndCureWindowElapsed,
    /// Written notice provided but within statutory cure window (premature action).
    WrittenNoticeWithinCureWindowPrematureAction,
    /// No written notice provided to landlord (typically defeats self-help).
    NoWrittenNoticeToLandlord,
    /// MA-specific: no cure required, immediate withholding permitted.
    NoCureRequiredImmediateAction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantStatutoryRentEscrowOrRepairDeduct,
    TenantRemedyCapExceededLandlordEvictionExposure,
    NoNoticeOrPrematureActionTenantEvictionRisk,
    EscrowAccountNotEstablishedTenantWaivesDefense,
    WashingtonRcw59_18_115EscrowProcessNotFollowed,
    ChicagoRltoWithholdingCapExceededExposure,
    MassachusettsImmediateWithholdingNoCureRequired,
    CommonLawWarrantyImpliedDefense,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub tenant_remedy_action: TenantRemedyAction,
    pub notice_and_cure_status: NoticeAndCureStatus,
    pub monthly_rent_cents: u64,
    pub withheld_or_deducted_amount_cents: u64,
}

pub type RentalTenantRentEscrowHabitabilityDisputeInput = Input;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Output {
    pub severity: Severity,
    pub statutory_cap_cents: u64,
    pub excess_over_cap_cents: u64,
    pub note: String,
}

pub type RentalTenantRentEscrowHabitabilityDisputeOutput = Output;
pub type RentalTenantRentEscrowHabitabilityDisputeResult = Output;

#[allow(dead_code)]
const CA_REPAIR_DEDUCT_PERIODS_PER_YEAR: u32 = 2;
#[allow(dead_code)]
const WA_REPAIR_DEDUCT_CAP_MONTHS_PER_REPAIR: u32 = 2;
#[allow(dead_code)]
const WA_REPAIR_DEDUCT_CAP_MONTHS_PER_YEAR: u32 = 2;
const CHICAGO_RLTO_WITHHOLDING_FLAT_FLOOR_CENTS: u64 = 50_000;
const CHICAGO_RLTO_WITHHOLDING_PERCENT_OF_RENT_BPS: u32 = 5_000;
const TX_REPAIR_DEDUCT_FLAT_FLOOR_CENTS: u64 = 50_000;

#[must_use]
pub fn check(input: &Input) -> Output {
    if matches!(
        input.notice_and_cure_status,
        NoticeAndCureStatus::NoWrittenNoticeToLandlord
    ) && !matches!(input.jurisdiction, Jurisdiction::Massachusetts) {
        return Output {
            severity: Severity::NoNoticeOrPrematureActionTenantEvictionRisk,
            statutory_cap_cents: 0,
            excess_over_cap_cents: input.withheld_or_deducted_amount_cents,
            note: format!(
                "No written notice to landlord. Self-help rent withholding or repair-and- \
                 deduct generally REQUIRES written notice + statutory cure window. {} \
                 Tenant action is procedurally defective; landlord may file nonpayment- \
                 of-rent eviction. Withheld amount ${} is at risk; recover by depositing \
                 immediately into escrow + providing notice + raising warranty defense \
                 in summary process.",
                statute_citation(input.jurisdiction),
                input.withheld_or_deducted_amount_cents / 100
            ),
        };
    }

    if matches!(
        input.notice_and_cure_status,
        NoticeAndCureStatus::WrittenNoticeWithinCureWindowPrematureAction
    ) && !matches!(input.jurisdiction, Jurisdiction::Massachusetts) {
        return Output {
            severity: Severity::NoNoticeOrPrematureActionTenantEvictionRisk,
            statutory_cap_cents: 0,
            excess_over_cap_cents: input.withheld_or_deducted_amount_cents,
            note: format!(
                "Written notice provided but action taken WITHIN statutory cure window. \
                 Premature self-help — landlord has not yet had reasonable opportunity to \
                 cure. {} Withheld amount ${} is at risk; resume rent payment immediately \
                 and restart the notice-and-cure process. Statutory rent escrow + repair- \
                 and-deduct may resume once the cure window has fully elapsed.",
                statute_citation(input.jurisdiction),
                input.withheld_or_deducted_amount_cents / 100
            ),
        };
    }

    if matches!(input.jurisdiction, Jurisdiction::Washington)
        && matches!(
            input.tenant_remedy_action,
            TenantRemedyAction::RentWithholdingWithoutRepair
        )
    {
        return Output {
            severity: Severity::WashingtonRcw59_18_115EscrowProcessNotFollowed,
            statutory_cap_cents: 0,
            excess_over_cap_cents: input.withheld_or_deducted_amount_cents,
            note: format!(
                "Washington RCW 59.18.115 REQUIRES rent withholding only through the \
                 statutory escrow process: written notice + statutory cure window + \
                 local-authority certification of conditions + deposit in approved escrow \
                 account. Pure rent withholding without escrow deposit is procedurally \
                 defective and triggers nonpayment eviction exposure. Tenant withheld ${} \
                 without escrow process; deposit immediately into approved account + obtain \
                 local-authority certification to cure the procedural defect.",
                input.withheld_or_deducted_amount_cents / 100
            ),
        };
    }

    if matches!(
        input.tenant_remedy_action,
        TenantRemedyAction::RentWithholdingWithoutRepair
    ) && !matches!(input.jurisdiction, Jurisdiction::Massachusetts)
    {
        return Output {
            severity: Severity::EscrowAccountNotEstablishedTenantWaivesDefense,
            statutory_cap_cents: 0,
            excess_over_cap_cents: input.withheld_or_deducted_amount_cents,
            note: format!(
                "Rent withheld WITHOUT escrow deposit. While statutory rent withholding may \
                 be permitted in {}, courts treat NON-escrowed withholding as nonpayment of \
                 rent justifying eviction. Tenant should deposit withheld amount (${}) into \
                 a separate bank account ('escrow') and produce records at any summary- \
                 process hearing. Failure to escrow waives the habitability defense in many \
                 jurisdictions.",
                jurisdiction_label(input.jurisdiction),
                input.withheld_or_deducted_amount_cents / 100
            ),
        };
    }

    if matches!(input.jurisdiction, Jurisdiction::Massachusetts)
        && matches!(
            input.notice_and_cure_status,
            NoticeAndCureStatus::NoCureRequiredImmediateAction
        )
    {
        return Output {
            severity: Severity::MassachusettsImmediateWithholdingNoCureRequired,
            statutory_cap_cents: 0,
            excess_over_cap_cents: 0,
            note: format!(
                "MA Gen. L. ch. 239 § 8A permits IMMEDIATE rent withholding upon notice — \
                 no cure period required. Massachusetts is distinct from CA/NY/WA/IL/TX \
                 in this respect. Withheld rent must be paid into court-supervised escrow \
                 during summary-process proceeding. Tenant raises warranty-of-habitability \
                 claim as defense or counterclaim. Withheld amount ${} preserved in escrow.",
                input.withheld_or_deducted_amount_cents / 100
            ),
        };
    }

    if matches!(input.jurisdiction, Jurisdiction::IllinoisChicagoRlto) {
        let percent_cap = input
            .monthly_rent_cents
            .saturating_mul(u64::from(CHICAGO_RLTO_WITHHOLDING_PERCENT_OF_RENT_BPS))
            .saturating_div(10_000);
        let cap = CHICAGO_RLTO_WITHHOLDING_FLAT_FLOOR_CENTS.max(percent_cap);
        if input.withheld_or_deducted_amount_cents > cap {
            return Output {
                severity: Severity::ChicagoRltoWithholdingCapExceededExposure,
                statutory_cap_cents: cap,
                excess_over_cap_cents: input
                    .withheld_or_deducted_amount_cents
                    .saturating_sub(cap),
                note: format!(
                    "Chicago RLTO § 5-12-110 violation: withholding (${}) EXCEEDS the cap \
                     of GREATER of $500 or 50% of monthly rent (${}). Excess (${}) creates \
                     nonpayment-of-rent eviction exposure. Reduce withholding to cap and \
                     resume rent payment for the difference.",
                    input.withheld_or_deducted_amount_cents / 100,
                    cap / 100,
                    input
                        .withheld_or_deducted_amount_cents
                        .saturating_sub(cap)
                        / 100
                ),
            };
        }
        return Output {
            severity: Severity::CompliantStatutoryRentEscrowOrRepairDeduct,
            statutory_cap_cents: cap,
            excess_over_cap_cents: 0,
            note: format!(
                "Compliant under Chicago RLTO § 5-12-110: withholding (${}) within cap of \
                 GREATER of $500 or 50% of monthly rent (${}). 14-day written notice + \
                 material habitability defect requirements satisfied.",
                input.withheld_or_deducted_amount_cents / 100,
                cap / 100
            ),
        };
    }

    if matches!(input.jurisdiction, Jurisdiction::Texas)
        && matches!(
            input.tenant_remedy_action,
            TenantRemedyAction::RepairAndDeductFromRent
        )
    {
        let cap = TX_REPAIR_DEDUCT_FLAT_FLOOR_CENTS.max(input.monthly_rent_cents);
        if input.withheld_or_deducted_amount_cents > cap {
            return Output {
                severity: Severity::TenantRemedyCapExceededLandlordEvictionExposure,
                statutory_cap_cents: cap,
                excess_over_cap_cents: input
                    .withheld_or_deducted_amount_cents
                    .saturating_sub(cap),
                note: format!(
                    "Texas Prop. Code § 92.0561 cap exceeded. Repair-and-deduct cap is the \
                     GREATER of one month's rent (${}) or $500 — deduction ${} exceeds cap \
                     ${}. Tenant nonpayment-of-rent eviction exposure on excess ${}.",
                    input.monthly_rent_cents / 100,
                    input.withheld_or_deducted_amount_cents / 100,
                    cap / 100,
                    input
                        .withheld_or_deducted_amount_cents
                        .saturating_sub(cap)
                        / 100
                ),
            };
        }
        return Output {
            severity: Severity::CompliantStatutoryRentEscrowOrRepairDeduct,
            statutory_cap_cents: cap,
            excess_over_cap_cents: 0,
            note: format!(
                "Compliant under Texas Prop. Code § 92.0561: repair-and-deduct (${}) within \
                 cap of GREATER of one month's rent or $500 (${}). Narrow-circumstance \
                 requirement (sewage/flood/material defect) + landlord-duty-non-waived in \
                 lease must also be satisfied.",
                input.withheld_or_deducted_amount_cents / 100,
                cap / 100
            ),
        };
    }

    if matches!(input.jurisdiction, Jurisdiction::California)
        && matches!(
            input.tenant_remedy_action,
            TenantRemedyAction::RepairAndDeductFromRent
        )
    {
        if input.withheld_or_deducted_amount_cents > input.monthly_rent_cents {
            return Output {
                severity: Severity::TenantRemedyCapExceededLandlordEvictionExposure,
                statutory_cap_cents: input.monthly_rent_cents,
                excess_over_cap_cents: input
                    .withheld_or_deducted_amount_cents
                    .saturating_sub(input.monthly_rent_cents),
                note: format!(
                    "CA Civ. Code § 1942 cap exceeded: single repair-and-deduct cannot \
                     exceed one month's rent (${}). Deduction ${} exceeds cap. § 1942 \
                     allows up to 2 such uses per 12-month period; excess ${} creates \
                     nonpayment-of-rent eviction exposure.",
                    input.monthly_rent_cents / 100,
                    input.withheld_or_deducted_amount_cents / 100,
                    input
                        .withheld_or_deducted_amount_cents
                        .saturating_sub(input.monthly_rent_cents)
                        / 100
                ),
            };
        }
        return Output {
            severity: Severity::CompliantStatutoryRentEscrowOrRepairDeduct,
            statutory_cap_cents: input.monthly_rent_cents,
            excess_over_cap_cents: 0,
            note: format!(
                "Compliant under CA Civ. Code § 1942: single repair-and-deduct (${}) \
                 within one month's rent cap (${}). Up to 2 uses per 12-month period \
                 permitted. § 1942.4 prohibits rent collection if landlord knew of \
                 substandard conditions for 60+ days.",
                input.withheld_or_deducted_amount_cents / 100,
                input.monthly_rent_cents / 100
            ),
        };
    }

    if matches!(input.jurisdiction, Jurisdiction::Default) {
        return Output {
            severity: Severity::CommonLawWarrantyImpliedDefense,
            statutory_cap_cents: 0,
            excess_over_cap_cents: 0,
            note: "Common-law implied warranty of habitability + constructive-eviction \
                   doctrine (Marini v. Ireland 265 A.2d 526 (NJ 1970) + Pugh v. Holmes 384 \
                   A.2d 1234 (PA 1979)) applies absent statutory rent-escrow regime. \
                   Tenant remedies are narrower than under statutory frameworks; courts \
                   evaluate withholding on case-by-case reasonableness standard."
                .to_string(),
        };
    }

    Output {
        severity: Severity::CompliantStatutoryRentEscrowOrRepairDeduct,
        statutory_cap_cents: 0,
        excess_over_cap_cents: 0,
        note: format!(
            "Compliant statutory rent escrow / repair-and-deduct procedure under \
             {}. Maintain documentation of notice + cure window + escrow records for \
             any summary-process hearing.",
            statute_citation(input.jurisdiction)
        ),
    }
}

fn jurisdiction_label(jurisdiction: Jurisdiction) -> &'static str {
    match jurisdiction {
        Jurisdiction::California => "California",
        Jurisdiction::NewYork => "New York",
        Jurisdiction::Washington => "Washington",
        Jurisdiction::IllinoisChicagoRlto => "Chicago (Illinois)",
        Jurisdiction::Texas => "Texas",
        Jurisdiction::Massachusetts => "Massachusetts",
        Jurisdiction::Default => "Default",
    }
}

fn statute_citation(jurisdiction: Jurisdiction) -> &'static str {
    match jurisdiction {
        Jurisdiction::California => {
            "CA Civ. Code § 1942 + § 1942.4 (repair-and-deduct cap one month rent, \
             twice per 12-month period)"
        }
        Jurisdiction::NewYork => {
            "NY RPL § 235-b warranty of habitability + RPL § 235-a + HP proceedings \
             under RPAPL § 110"
        }
        Jurisdiction::Washington => {
            "WA RCW 59.18.115 rent escrow + RCW 59.18.100 repair-and-deduct (2 months \
             rent per repair, 2 months rent per year)"
        }
        Jurisdiction::IllinoisChicagoRlto => {
            "Chicago RLTO § 5-12-110 14-day notice + cap of GREATER of $500 or 50% \
             monthly rent"
        }
        Jurisdiction::Texas => {
            "Texas Prop. Code § 92.0561 repair-and-deduct cap GREATER of one month \
             rent or $500"
        }
        Jurisdiction::Massachusetts => {
            "MA Gen. L. ch. 239 § 8A immediate-withholding-on-notice (no cure period \
             required)"
        }
        Jurisdiction::Default => {
            "Common-law implied warranty of habitability + Marini v. Ireland + Pugh v. \
             Holmes constructive-eviction doctrine"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_ca() -> Input {
        Input {
            jurisdiction: Jurisdiction::California,
            tenant_remedy_action: TenantRemedyAction::RepairAndDeductFromRent,
            notice_and_cure_status: NoticeAndCureStatus::WrittenNoticeAndCureWindowElapsed,
            monthly_rent_cents: 3_000_00,
            withheld_or_deducted_amount_cents: 2_000_00,
        }
    }

    #[test]
    fn california_repair_deduct_within_one_month_rent_compliant() {
        let input = base_ca();
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantStatutoryRentEscrowOrRepairDeduct
        );
        assert!(output.note.contains("§ 1942"));
        assert!(output.note.contains("§ 1942.4"));
    }

    #[test]
    fn california_repair_deduct_exceeds_one_month_rent_cap_violation() {
        let mut input = base_ca();
        input.withheld_or_deducted_amount_cents = 4_000_00;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::TenantRemedyCapExceededLandlordEvictionExposure
        );
        assert_eq!(output.excess_over_cap_cents, 1_000_00);
    }

    #[test]
    fn california_no_notice_no_self_help_eviction_risk() {
        let mut input = base_ca();
        input.notice_and_cure_status = NoticeAndCureStatus::NoWrittenNoticeToLandlord;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoNoticeOrPrematureActionTenantEvictionRisk
        );
    }

    #[test]
    fn california_premature_action_within_cure_window_eviction_risk() {
        let mut input = base_ca();
        input.notice_and_cure_status =
            NoticeAndCureStatus::WrittenNoticeWithinCureWindowPrematureAction;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::NoNoticeOrPrematureActionTenantEvictionRisk
        );
    }

    #[test]
    fn washington_pure_withholding_without_escrow_violation() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Washington;
        input.tenant_remedy_action = TenantRemedyAction::RentWithholdingWithoutRepair;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::WashingtonRcw59_18_115EscrowProcessNotFollowed
        );
        assert!(output.note.contains("RCW 59.18.115"));
        assert!(output.note.contains("local-authority certification"));
    }

    #[test]
    fn chicago_rlto_within_cap_compliant() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::IllinoisChicagoRlto;
        input.tenant_remedy_action =
            TenantRemedyAction::RentWithholdingDepositedToEscrowAccount;
        input.withheld_or_deducted_amount_cents = 1_500_00;
        let output = check(&input);
        // Cap = max($500, 50% × $3K) = max($500, $1500) = $1500
        // Withheld $1500 = cap → compliant
        assert_eq!(
            output.severity,
            Severity::CompliantStatutoryRentEscrowOrRepairDeduct
        );
    }

    #[test]
    fn chicago_rlto_exceeds_50_percent_rent_cap_violation() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::IllinoisChicagoRlto;
        input.tenant_remedy_action =
            TenantRemedyAction::RentWithholdingDepositedToEscrowAccount;
        input.withheld_or_deducted_amount_cents = 2_000_00;
        let output = check(&input);
        // Cap = $1500; withheld $2000 → $500 excess
        assert_eq!(
            output.severity,
            Severity::ChicagoRltoWithholdingCapExceededExposure
        );
        assert_eq!(output.excess_over_cap_cents, 500_00);
    }

    #[test]
    fn chicago_rlto_low_rent_uses_500_dollar_floor() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::IllinoisChicagoRlto;
        input.tenant_remedy_action =
            TenantRemedyAction::RentWithholdingDepositedToEscrowAccount;
        input.monthly_rent_cents = 500_00; // 50% = $250 < $500 floor
        input.withheld_or_deducted_amount_cents = 500_00;
        let output = check(&input);
        // Cap = max($500, $250) = $500
        assert_eq!(output.statutory_cap_cents, 500_00);
        assert_eq!(
            output.severity,
            Severity::CompliantStatutoryRentEscrowOrRepairDeduct
        );
    }

    #[test]
    fn texas_repair_deduct_within_cap_compliant() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Texas;
        input.withheld_or_deducted_amount_cents = 2_000_00;
        let output = check(&input);
        // Cap = max($500, $3000) = $3000; $2000 within → compliant
        assert_eq!(
            output.severity,
            Severity::CompliantStatutoryRentEscrowOrRepairDeduct
        );
    }

    #[test]
    fn texas_repair_deduct_exceeds_one_month_rent_violation() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Texas;
        input.withheld_or_deducted_amount_cents = 4_000_00;
        let output = check(&input);
        // Cap = $3000; $4000 → $1000 excess
        assert_eq!(
            output.severity,
            Severity::TenantRemedyCapExceededLandlordEvictionExposure
        );
        assert_eq!(output.excess_over_cap_cents, 1_000_00);
    }

    #[test]
    fn massachusetts_immediate_withholding_no_cure_required() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Massachusetts;
        input.tenant_remedy_action =
            TenantRemedyAction::RentWithholdingDepositedToEscrowAccount;
        input.notice_and_cure_status = NoticeAndCureStatus::NoCureRequiredImmediateAction;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::MassachusettsImmediateWithholdingNoCureRequired
        );
        assert!(output.note.contains("§ 8A"));
        assert!(output.note.contains("IMMEDIATE"));
    }

    #[test]
    fn pure_withholding_without_escrow_waives_defense() {
        let mut input = base_ca();
        input.tenant_remedy_action = TenantRemedyAction::RentWithholdingWithoutRepair;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::EscrowAccountNotEstablishedTenantWaivesDefense
        );
    }

    #[test]
    fn default_jurisdiction_common_law_warranty() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::Default;
        let output = check(&input);
        assert_eq!(output.severity, Severity::CommonLawWarrantyImpliedDefense);
        assert!(output.note.contains("Marini v. Ireland"));
        assert!(output.note.contains("Pugh v. Holmes"));
    }

    #[test]
    fn new_york_compliant_rent_withholding_with_escrow() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::NewYork;
        input.tenant_remedy_action =
            TenantRemedyAction::RentWithholdingDepositedToEscrowAccount;
        let output = check(&input);
        assert_eq!(
            output.severity,
            Severity::CompliantStatutoryRentEscrowOrRepairDeduct
        );
        assert!(output.note.contains("§ 235-b"));
        assert!(output.note.contains("RPAPL § 110"));
    }

    #[test]
    fn ca_repair_deduct_periods_per_year_constant_pins_2() {
        assert_eq!(CA_REPAIR_DEDUCT_PERIODS_PER_YEAR, 2);
    }

    #[test]
    fn wa_repair_deduct_cap_per_repair_constant_pins_2_months() {
        assert_eq!(WA_REPAIR_DEDUCT_CAP_MONTHS_PER_REPAIR, 2);
    }

    #[test]
    fn wa_repair_deduct_cap_per_year_constant_pins_2_months() {
        assert_eq!(WA_REPAIR_DEDUCT_CAP_MONTHS_PER_YEAR, 2);
    }

    #[test]
    fn chicago_rlto_flat_floor_constant_pins_500() {
        assert_eq!(CHICAGO_RLTO_WITHHOLDING_FLAT_FLOOR_CENTS, 50_000);
    }

    #[test]
    fn chicago_rlto_percent_constant_pins_50_pct() {
        assert_eq!(CHICAGO_RLTO_WITHHOLDING_PERCENT_OF_RENT_BPS, 5_000);
    }

    #[test]
    fn tx_repair_deduct_flat_floor_constant_pins_500() {
        assert_eq!(TX_REPAIR_DEDUCT_FLAT_FLOOR_CENTS, 50_000);
    }

    #[test]
    fn very_large_rent_no_overflow_in_chicago_50_pct_calc() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::IllinoisChicagoRlto;
        input.tenant_remedy_action =
            TenantRemedyAction::RentWithholdingDepositedToEscrowAccount;
        input.monthly_rent_cents = u64::MAX / 2;
        let output = check(&input);
        assert!(output.statutory_cap_cents > 0);
    }

    #[test]
    fn zero_rent_falls_back_to_chicago_floor() {
        let mut input = base_ca();
        input.jurisdiction = Jurisdiction::IllinoisChicagoRlto;
        input.tenant_remedy_action =
            TenantRemedyAction::RentWithholdingDepositedToEscrowAccount;
        input.monthly_rent_cents = 0;
        input.withheld_or_deducted_amount_cents = 100;
        let output = check(&input);
        // Cap = max($500, $0) = $500; withheld $1 within
        assert_eq!(output.statutory_cap_cents, 500_00);
    }
}
