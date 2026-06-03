//! IRC § 263 capital expenditures — general capitalization rule
//! for amounts paid to acquire, produce, or improve tangible
//! property.
//!
//! § 263(a) is the foundational capitalization rule that requires
//! capitalization (rather than current deduction) of amounts paid
//! for permanent improvements, betterments, and restorations. The
//! 2013 Tangible Property Regulations (T.D. 9636) finalized the
//! operative test framework — the "BAR test" — and three safe
//! harbors that distinguish currently deductible repairs from
//! capitalizable improvements.
//!
//! **§ 263(a)(1) general rule**: no deduction shall be allowed for
//! (A) any amount paid out for new buildings or for permanent
//! improvements or betterments made to increase the value of any
//! property or estate, or (B) any amount expended in restoring
//! property or in making good the exhaustion thereof for which an
//! allowance is or has been made.
//!
//! **Treas. Reg. § 1.263(a)-3 BAR test** — three-prong improvement
//! classification:
//!
//! - **B = Betterment** (§ 1.263(a)-3(j)): material increase in
//!   capacity, productivity, efficiency, strength, or quality of
//!   property; correction of material condition or defect existing
//!   at acquisition; material addition or enlargement.
//! - **A = Adaptation** (§ 1.263(a)-3(l)): conversion to a new or
//!   different use inconsistent with original purpose (e.g.,
//!   residential apartments → commercial office space).
//! - **R = Restoration** (§ 1.263(a)-3(k)): return to operating
//!   condition after major disrepair, replacement of major
//!   structural component or substantial structural part, rebuild
//!   to like-new condition after class life expiration.
//!
//! **§ 1.263(a)-1(f) de minimis safe harbor**: taxpayer may elect
//! to expense items under threshold without applying BAR test —
//! **$5,000 per invoice or item with Applicable Financial Statement
//! (AFS)**; **$2,500 per invoice or item without AFS** (increased
//! from $500 effective 2016 by Notice 2015-82). Election made
//! annually on Form 3115 or attached statement.
//!
//! **§ 1.263(a)-3(h) small taxpayer safe harbor**: qualifying
//! taxpayer (average annual gross receipts ≤ $10M) may elect on
//! eligible building (unadjusted basis ≤ $1,000,000) to deduct all
//! aggregate annual repairs/improvements up to the LESSER of (i) 2%
//! of unadjusted basis, or (ii) $10,000.
//!
//! **§ 1.263(a)-3(i) routine maintenance safe harbor**: amounts
//! paid for recurring activities expected to be performed more than
//! once during the property's class life are deemed not to improve
//! the property — **10 years for buildings**, **3 years (or class
//! life) for non-buildings**.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const DE_MINIMIS_AFS_THRESHOLD_CENTS: u64 = 500_000;
#[allow(dead_code)]
pub const DE_MINIMIS_NO_AFS_THRESHOLD_CENTS: u64 = 250_000;
#[allow(dead_code)]
pub const SMALL_TAXPAYER_BUILDING_UNADJUSTED_BASIS_THRESHOLD_CENTS: u64 = 100_000_000;
#[allow(dead_code)]
pub const SMALL_TAXPAYER_SAFE_HARBOR_PERCENT_OF_BASIS: u32 = 2;
#[allow(dead_code)]
pub const SMALL_TAXPAYER_SAFE_HARBOR_DOLLAR_LIMIT_CENTS: u64 = 1_000_000;
#[allow(dead_code)]
pub const ROUTINE_MAINTENANCE_BUILDING_YEARS: u32 = 10;
#[allow(dead_code)]
pub const ROUTINE_MAINTENANCE_NON_BUILDING_YEARS: u32 = 3;
#[allow(dead_code)]
pub const NOTICE_2015_82_THRESHOLD_INCREASE_YEAR: u32 = 2016;
#[allow(dead_code)]
pub const TANGIBLE_PROPERTY_REGULATIONS_FINAL_YEAR: u32 = 2013;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafeHarborApplied {
    None,
    DeMinimisAfs5000,
    DeMinimisNoAfs2500,
    SmallTaxpayerSafeHarbor10000Or2Pct,
    RoutineMaintenanceSafeHarbor10YearBuilding,
    RoutineMaintenanceSafeHarbor3YearNonBuilding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantCurrentDeductionRepairOrMaintenance,
    CompliantCapitalizationOfImprovementUnderBarTest,
    CompliantDeMinimisSafeHarborAfs5000,
    CompliantDeMinimisSafeHarborNoAfs2500,
    CompliantSmallTaxpayerSafeHarbor10000Or2Pct,
    CompliantRoutineMaintenanceSafeHarbor10YearBuilding,
    CompliantRoutineMaintenanceSafeHarbor3YearNonBuilding,
    ViolationImprovementDeductedAsRepairExpense,
    ViolationFailedToCapitalizeRestorationOrAdaptation,
    ViolationDeMinimisExceededThresholdImproperlyClaimed,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub expenditure_amount_per_invoice_cents: u64,
    pub has_applicable_financial_statement: bool,
    pub building_unadjusted_basis_cents: u64,
    pub aggregate_annual_repairs_improvements_on_building_cents: u64,
    pub is_betterment: bool,
    pub is_adaptation: bool,
    pub is_restoration: bool,
    pub routine_maintenance_reasonable_expectation_within_class_life: bool,
    pub property_is_building: bool,
    pub small_taxpayer_safe_harbor_claimed: bool,
    pub de_minimis_safe_harbor_claimed: bool,
    pub taxpayer_capitalized: bool,
    pub taxpayer_deducted_as_expense: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub compliant: bool,
    pub amount_capitalizable_cents: u64,
    pub amount_currently_deductible_cents: u64,
    pub safe_harbor_applied: SafeHarborApplied,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type Section263Input = Input;
pub type Section263Output = Output;
pub type Section263Result = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "IRC § 263(a) (general capitalization rule)".to_string(),
        "IRC § 263(a)(1)(A) (new buildings + permanent improvements + betterments)".to_string(),
        "IRC § 263(a)(1)(B) (restoring property or making good exhaustion)".to_string(),
        "Treas. Reg. § 1.263(a)-1 (general rule for capital expenditures)".to_string(),
        "Treas. Reg. § 1.263(a)-1(f) (de minimis safe harbor)".to_string(),
        "Treas. Reg. § 1.263(a)-2 (acquisition or production of tangible property)".to_string(),
        "Treas. Reg. § 1.263(a)-3 (amounts paid to improve tangible property — BAR test)".to_string(),
        "Treas. Reg. § 1.263(a)-3(h) (small taxpayer safe harbor)".to_string(),
        "Treas. Reg. § 1.263(a)-3(i) (routine maintenance safe harbor)".to_string(),
        "Treas. Reg. § 1.263(a)-3(j) (betterment)".to_string(),
        "Treas. Reg. § 1.263(a)-3(k) (restoration)".to_string(),
        "Treas. Reg. § 1.263(a)-3(l) (adaptation)".to_string(),
        "T.D. 9636 (2013) — final Tangible Property Regulations".to_string(),
        "Notice 2015-82 (de minimis safe harbor threshold increase to $2,500 eff. 2016)".to_string(),
        "IRC § 162 (ordinary and necessary business expense — current deduction for non-capital)".to_string(),
        "IRC § 168 (MACRS — depreciation of capitalized amounts)".to_string(),
    ];

    let de_minimis_threshold = if input.has_applicable_financial_statement {
        DE_MINIMIS_AFS_THRESHOLD_CENTS
    } else {
        DE_MINIMIS_NO_AFS_THRESHOLD_CENTS
    };

    if input.de_minimis_safe_harbor_claimed {
        if input.expenditure_amount_per_invoice_cents > de_minimis_threshold {
            notes.push(format!(
                "De minimis safe harbor claimed but ${} per invoice exceeds ${} threshold (AFS: {}).",
                input.expenditure_amount_per_invoice_cents / 100,
                de_minimis_threshold / 100,
                input.has_applicable_financial_statement
            ));
            return Output {
                severity: Severity::ViolationDeMinimisExceededThresholdImproperlyClaimed,
                compliant: false,
                amount_capitalizable_cents: input.expenditure_amount_per_invoice_cents,
                amount_currently_deductible_cents: 0,
                safe_harbor_applied: SafeHarborApplied::None,
                notes,
                citations,
            };
        }
        let (severity, safe_harbor) = if input.has_applicable_financial_statement {
            notes.push(format!(
                "De minimis safe harbor (AFS): ${} ≤ ${} per-invoice threshold; current deduction allowed.",
                input.expenditure_amount_per_invoice_cents / 100,
                DE_MINIMIS_AFS_THRESHOLD_CENTS / 100
            ));
            (
                Severity::CompliantDeMinimisSafeHarborAfs5000,
                SafeHarborApplied::DeMinimisAfs5000,
            )
        } else {
            notes.push(format!(
                "De minimis safe harbor (no AFS): ${} ≤ ${} per-invoice threshold; current deduction allowed.",
                input.expenditure_amount_per_invoice_cents / 100,
                DE_MINIMIS_NO_AFS_THRESHOLD_CENTS / 100
            ));
            (
                Severity::CompliantDeMinimisSafeHarborNoAfs2500,
                SafeHarborApplied::DeMinimisNoAfs2500,
            )
        };
        return Output {
            severity,
            compliant: true,
            amount_capitalizable_cents: 0,
            amount_currently_deductible_cents: input.expenditure_amount_per_invoice_cents,
            safe_harbor_applied: safe_harbor,
            notes,
            citations,
        };
    }

    if input.small_taxpayer_safe_harbor_claimed
        && input.building_unadjusted_basis_cents
            <= SMALL_TAXPAYER_BUILDING_UNADJUSTED_BASIS_THRESHOLD_CENTS
    {
        let two_percent_basis = input.building_unadjusted_basis_cents / 50;
        let safe_harbor_limit = two_percent_basis.min(SMALL_TAXPAYER_SAFE_HARBOR_DOLLAR_LIMIT_CENTS);
        if input.aggregate_annual_repairs_improvements_on_building_cents <= safe_harbor_limit {
            notes.push(format!(
                "§ 1.263(a)-3(h) small taxpayer safe harbor: aggregate ${} ≤ lesser of 2% of basis (${}) or ${} dollar limit; current deduction allowed.",
                input.aggregate_annual_repairs_improvements_on_building_cents / 100,
                two_percent_basis / 100,
                SMALL_TAXPAYER_SAFE_HARBOR_DOLLAR_LIMIT_CENTS / 100
            ));
            return Output {
                severity: Severity::CompliantSmallTaxpayerSafeHarbor10000Or2Pct,
                compliant: true,
                amount_capitalizable_cents: 0,
                amount_currently_deductible_cents: input
                    .aggregate_annual_repairs_improvements_on_building_cents,
                safe_harbor_applied: SafeHarborApplied::SmallTaxpayerSafeHarbor10000Or2Pct,
                notes,
                citations,
            };
        }
    }

    if input.routine_maintenance_reasonable_expectation_within_class_life {
        let (severity, safe_harbor, years) = if input.property_is_building {
            (
                Severity::CompliantRoutineMaintenanceSafeHarbor10YearBuilding,
                SafeHarborApplied::RoutineMaintenanceSafeHarbor10YearBuilding,
                ROUTINE_MAINTENANCE_BUILDING_YEARS,
            )
        } else {
            (
                Severity::CompliantRoutineMaintenanceSafeHarbor3YearNonBuilding,
                SafeHarborApplied::RoutineMaintenanceSafeHarbor3YearNonBuilding,
                ROUTINE_MAINTENANCE_NON_BUILDING_YEARS,
            )
        };
        notes.push(format!(
            "§ 1.263(a)-3(i) routine maintenance safe harbor: recurring activity expected more than once within {}-year window for {} property; deemed not to improve.",
            years,
            if input.property_is_building { "building" } else { "non-building" }
        ));
        return Output {
            severity,
            compliant: true,
            amount_capitalizable_cents: 0,
            amount_currently_deductible_cents: input.expenditure_amount_per_invoice_cents,
            safe_harbor_applied: safe_harbor,
            notes,
            citations,
        };
    }

    let bar_triggered = input.is_betterment || input.is_adaptation || input.is_restoration;

    if bar_triggered && input.taxpayer_deducted_as_expense {
        let category = if input.is_betterment {
            "Betterment"
        } else if input.is_adaptation {
            "Adaptation"
        } else {
            "Restoration"
        };
        notes.push(format!(
            "BAR test triggered ({}); taxpayer deducted as repair expense — § 263(a) violation; ${} must be capitalized.",
            category,
            input.expenditure_amount_per_invoice_cents / 100
        ));
        let severity = if input.is_adaptation || input.is_restoration {
            Severity::ViolationFailedToCapitalizeRestorationOrAdaptation
        } else {
            Severity::ViolationImprovementDeductedAsRepairExpense
        };
        return Output {
            severity,
            compliant: false,
            amount_capitalizable_cents: input.expenditure_amount_per_invoice_cents,
            amount_currently_deductible_cents: 0,
            safe_harbor_applied: SafeHarborApplied::None,
            notes,
            citations,
        };
    }

    if bar_triggered {
        let category = if input.is_betterment {
            "Betterment"
        } else if input.is_adaptation {
            "Adaptation"
        } else {
            "Restoration"
        };
        notes.push(format!(
            "BAR test triggered ({}); ${} capitalized under § 263(a) and depreciated under § 168.",
            category,
            input.expenditure_amount_per_invoice_cents / 100
        ));
        return Output {
            severity: Severity::CompliantCapitalizationOfImprovementUnderBarTest,
            compliant: true,
            amount_capitalizable_cents: input.expenditure_amount_per_invoice_cents,
            amount_currently_deductible_cents: 0,
            safe_harbor_applied: SafeHarborApplied::None,
            notes,
            citations,
        };
    }

    notes.push(format!(
        "BAR test not triggered (no betterment/adaptation/restoration); ${} treated as current repair/maintenance under § 162 ordinary and necessary business expense.",
        input.expenditure_amount_per_invoice_cents / 100
    ));
    Output {
        severity: Severity::CompliantCurrentDeductionRepairOrMaintenance,
        compliant: true,
        amount_capitalizable_cents: 0,
        amount_currently_deductible_cents: input.expenditure_amount_per_invoice_cents,
        safe_harbor_applied: SafeHarborApplied::None,
        notes,
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_repair() -> Input {
        Input {
            expenditure_amount_per_invoice_cents: 100_000,
            has_applicable_financial_statement: false,
            building_unadjusted_basis_cents: 50_000_000,
            aggregate_annual_repairs_improvements_on_building_cents: 100_000,
            is_betterment: false,
            is_adaptation: false,
            is_restoration: false,
            routine_maintenance_reasonable_expectation_within_class_life: false,
            property_is_building: true,
            small_taxpayer_safe_harbor_claimed: false,
            de_minimis_safe_harbor_claimed: false,
            taxpayer_capitalized: false,
            taxpayer_deducted_as_expense: true,
        }
    }

    #[test]
    fn repair_no_bar_currently_deductible() {
        let out = check(&base_repair());
        assert_eq!(
            out.severity,
            Severity::CompliantCurrentDeductionRepairOrMaintenance
        );
        assert_eq!(out.amount_currently_deductible_cents, 100_000);
    }

    #[test]
    fn betterment_taxpayer_capitalized_compliant() {
        let mut i = base_repair();
        i.is_betterment = true;
        i.taxpayer_deducted_as_expense = false;
        i.taxpayer_capitalized = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantCapitalizationOfImprovementUnderBarTest
        );
        assert_eq!(out.amount_capitalizable_cents, 100_000);
    }

    #[test]
    fn betterment_deducted_as_repair_violation() {
        let mut i = base_repair();
        i.is_betterment = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationImprovementDeductedAsRepairExpense
        );
    }

    #[test]
    fn adaptation_deducted_violation() {
        let mut i = base_repair();
        i.is_adaptation = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationFailedToCapitalizeRestorationOrAdaptation
        );
    }

    #[test]
    fn restoration_deducted_violation() {
        let mut i = base_repair();
        i.is_restoration = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationFailedToCapitalizeRestorationOrAdaptation
        );
    }

    #[test]
    fn de_minimis_safe_harbor_afs_5000_compliant() {
        let mut i = base_repair();
        i.expenditure_amount_per_invoice_cents = 500_000;
        i.has_applicable_financial_statement = true;
        i.de_minimis_safe_harbor_claimed = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantDeMinimisSafeHarborAfs5000
        );
        assert_eq!(out.safe_harbor_applied, SafeHarborApplied::DeMinimisAfs5000);
    }

    #[test]
    fn de_minimis_safe_harbor_no_afs_2500_compliant() {
        let mut i = base_repair();
        i.expenditure_amount_per_invoice_cents = 250_000;
        i.has_applicable_financial_statement = false;
        i.de_minimis_safe_harbor_claimed = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantDeMinimisSafeHarborNoAfs2500
        );
    }

    #[test]
    fn de_minimis_afs_exceeds_5000_violation() {
        let mut i = base_repair();
        i.expenditure_amount_per_invoice_cents = 600_000;
        i.has_applicable_financial_statement = true;
        i.de_minimis_safe_harbor_claimed = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationDeMinimisExceededThresholdImproperlyClaimed
        );
    }

    #[test]
    fn small_taxpayer_safe_harbor_2pct_under_10000_compliant() {
        let mut i = base_repair();
        i.building_unadjusted_basis_cents = 50_000_000;
        i.aggregate_annual_repairs_improvements_on_building_cents = 800_000;
        i.small_taxpayer_safe_harbor_claimed = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantSmallTaxpayerSafeHarbor10000Or2Pct
        );
    }

    #[test]
    fn small_taxpayer_safe_harbor_aggregate_exceeds_lesser_limit_not_eligible() {
        let mut i = base_repair();
        i.building_unadjusted_basis_cents = 50_000_000;
        i.aggregate_annual_repairs_improvements_on_building_cents = 1_500_000;
        i.small_taxpayer_safe_harbor_claimed = true;
        let out = check(&i);
        assert_ne!(
            out.severity,
            Severity::CompliantSmallTaxpayerSafeHarbor10000Or2Pct
        );
    }

    #[test]
    fn routine_maintenance_building_10_year_compliant() {
        let mut i = base_repair();
        i.routine_maintenance_reasonable_expectation_within_class_life = true;
        i.property_is_building = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantRoutineMaintenanceSafeHarbor10YearBuilding
        );
    }

    #[test]
    fn routine_maintenance_non_building_3_year_compliant() {
        let mut i = base_repair();
        i.routine_maintenance_reasonable_expectation_within_class_life = true;
        i.property_is_building = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantRoutineMaintenanceSafeHarbor3YearNonBuilding
        );
    }

    #[test]
    fn citations_pin_263a_treas_reg_subsections() {
        let out = check(&base_repair());
        assert!(out.citations.iter().any(|c| c.contains("§ 263(a)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.263(a)-1")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.263(a)-2")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.263(a)-3")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.263(a)-3(h)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.263(a)-3(i)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.263(a)-3(j)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.263(a)-3(k)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.263(a)-3(l)")));
    }

    #[test]
    fn citations_pin_td_9636_and_notice_2015_82() {
        let out = check(&base_repair());
        assert!(out.citations.iter().any(|c| c.contains("T.D. 9636")));
        assert!(out.citations.iter().any(|c| c.contains("Notice 2015-82")));
    }

    #[test]
    fn citations_pin_162_168_cross_refs() {
        let out = check(&base_repair());
        assert!(out.citations.iter().any(|c| c.contains("§ 162")));
        assert!(out.citations.iter().any(|c| c.contains("§ 168")));
    }

    #[test]
    fn constant_pin_5000_afs_de_minimis() {
        assert_eq!(DE_MINIMIS_AFS_THRESHOLD_CENTS, 500_000);
    }

    #[test]
    fn constant_pin_2500_no_afs_de_minimis() {
        assert_eq!(DE_MINIMIS_NO_AFS_THRESHOLD_CENTS, 250_000);
    }

    #[test]
    fn constant_pin_10000_small_taxpayer_dollar_limit() {
        assert_eq!(SMALL_TAXPAYER_SAFE_HARBOR_DOLLAR_LIMIT_CENTS, 1_000_000);
    }

    #[test]
    fn constant_pin_2_pct_basis_threshold() {
        assert_eq!(SMALL_TAXPAYER_SAFE_HARBOR_PERCENT_OF_BASIS, 2);
    }

    #[test]
    fn constant_pin_10_year_building_routine_maintenance() {
        assert_eq!(ROUTINE_MAINTENANCE_BUILDING_YEARS, 10);
    }

    #[test]
    fn constant_pin_3_year_non_building_routine_maintenance() {
        assert_eq!(ROUTINE_MAINTENANCE_NON_BUILDING_YEARS, 3);
    }
}
