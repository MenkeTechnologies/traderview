//! IRC § 446 general rule for methods of accounting.
//!
//! Foundational accounting-method provision establishing which
//! methods are permissible, when IRS may override taxpayer's chosen
//! method, and the procedure for changing methods.
//!
//! **§ 446(a) general rule**: taxable income shall be computed
//! under the method of accounting on the basis of which the
//! taxpayer regularly computes income in keeping the taxpayer's
//! books — the book-tax conformity default.
//!
//! **§ 446(b) authority of Secretary**: if no method has been
//! regularly used by the taxpayer, or if the method used does not
//! "clearly reflect income," the Secretary may compute taxable
//! income under a method that does clearly reflect income.
//! "Clearly reflect income" is an IRS-discretionary standard
//! applied case-by-case.
//!
//! **§ 446(c) permissible methods**:
//!
//! - **§ 446(c)(1) cash receipts and disbursements method** (cash
//!   basis) — income recognized when received, deductions when
//!   paid.
//! - **§ 446(c)(2) accrual method** — income recognized when all
//!   events fixing the right to receive occur and amount can be
//!   determined with reasonable accuracy; deductions when all
//!   events fixing the liability occur and economic performance
//!   occurs under § 461(h).
//! - **§ 446(c)(3) other methods permitted by chapter** — e.g.,
//!   § 475 mark-to-market, § 453 installment method, § 460 long-
//!   term contract methods.
//! - **§ 446(c)(4) combinations of methods** — hybrid methods are
//!   permitted if they clearly reflect income and are consistently
//!   used. A common combination is accrual method for purchases
//!   and sales (because of inventory requirement under § 471) plus
//!   cash method for other items.
//!
//! **§ 446(d) different methods for different trades or
//! businesses**: a taxpayer engaged in more than one trade or
//! business may use different methods for each, provided each
//! clearly reflects income and is used consistently.
//!
//! **§ 446(e) requirement of consent for change of method**: a
//! taxpayer who changes the method of accounting shall, before
//! computing taxable income under the new method, secure the
//! consent of the Secretary. Form 3115 (Application for Change in
//! Accounting Method) is the procedural mechanism — either
//! automatic consent under Rev. Proc. 2024-23 (current automatic
//! consent procedure) for specified changes, or advance non-
//! automatic consent for non-listed changes.
//!
//! **§ 446(f) long-term contract methods**: cross-reference to
//! § 460 — failure to use the percentage-of-completion method on
//! long-term contracts where required.
//!
//! **"Clearly reflect income" consistency requirement** (Treas.
//! Reg. § 1.446-1(a)(2)): no method is acceptable unless all items
//! of gross profit and deductions are treated with consistency
//! from year to year.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const METHOD_CASH_RECEIPTS_AND_DISBURSEMENTS: u32 = 1;
#[allow(dead_code)]
pub const METHOD_ACCRUAL: u32 = 2;
#[allow(dead_code)]
pub const METHOD_OTHER_PERMITTED_BY_CHAPTER: u32 = 3;
#[allow(dead_code)]
pub const METHOD_COMBINATION_HYBRID: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountingMethod {
    NotApplicable,
    CashReceiptsAndDisbursements,
    Accrual,
    OtherPermitted,
    HybridCombination,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    CompliantCashMethodSection446c1,
    CompliantAccrualMethodSection446c2,
    CompliantOtherPermittedMethodSection446c3,
    CompliantHybridMethodSection446c4,
    CompliantDifferentMethodsForDifferentTradesUnderSection446d,
    ViolationMethodDoesNotClearlyReflectIncome,
    ViolationMethodChangeWithoutForm3115Consent,
    ViolationInconsistentTreatmentYearToYear,
    IrsAuthorityToOverrideUnderSection446b,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub accounting_method: AccountingMethod,
    pub different_methods_for_different_trades: bool,
    pub method_clearly_reflects_income: bool,
    pub consistent_treatment_year_to_year: bool,
    pub method_change_made: bool,
    pub form_3115_filed_with_irs_consent: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub compliant: bool,
    pub form_3115_required: bool,
    pub irs_authority_to_change_method: bool,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type Section446Input = Input;
pub type Section446Output = Output;
pub type Section446Result = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "IRC § 446(a) (book-tax conformity general rule)".to_string(),
        "IRC § 446(b) (Secretary's authority — clearly reflect income standard)".to_string(),
        "IRC § 446(c)(1) (cash receipts and disbursements method)".to_string(),
        "IRC § 446(c)(2) (accrual method)".to_string(),
        "IRC § 446(c)(3) (other methods permitted by chapter)".to_string(),
        "IRC § 446(c)(4) (combinations of methods — hybrid)".to_string(),
        "IRC § 446(d) (different methods for different trades or businesses)".to_string(),
        "IRC § 446(e) (consent required for method change)".to_string(),
        "IRC § 446(f) (cross-reference to § 460 long-term contract methods)".to_string(),
        "IRC § 461(h) (economic performance — accrual deductions)".to_string(),
        "IRC § 471 (inventory requirement — drives accrual for purchases/sales)".to_string(),
        "Treas. Reg. § 1.446-1 (general rule implementing regulations)".to_string(),
        "Treas. Reg. § 1.446-1(a)(2) (consistency requirement)".to_string(),
        "Form 3115 (Application for Change in Accounting Method)".to_string(),
        "Rev. Proc. 2024-23 (current automatic consent procedure)".to_string(),
    ];

    if matches!(input.accounting_method, AccountingMethod::NotApplicable) {
        notes.push("No accounting method recorded.".to_string());
        return Output {
            severity: Severity::NotApplicable,
            compliant: true,
            form_3115_required: false,
            irs_authority_to_change_method: false,
            notes,
            citations,
        };
    }

    if !input.method_clearly_reflects_income {
        notes.push("Accounting method does not clearly reflect income — § 446(b) Secretary may override under § 1.446-1(b)(1).".to_string());
        return Output {
            severity: Severity::ViolationMethodDoesNotClearlyReflectIncome,
            compliant: false,
            form_3115_required: false,
            irs_authority_to_change_method: true,
            notes,
            citations,
        };
    }

    if !input.consistent_treatment_year_to_year {
        notes.push("Items of gross profit and deductions not treated consistently from year to year — Treas. Reg. § 1.446-1(a)(2) consistency violation; no method is acceptable.".to_string());
        return Output {
            severity: Severity::ViolationInconsistentTreatmentYearToYear,
            compliant: false,
            form_3115_required: false,
            irs_authority_to_change_method: true,
            notes,
            citations,
        };
    }

    if input.method_change_made && !input.form_3115_filed_with_irs_consent {
        notes.push("Method changed without IRS consent — § 446(e) violation; Form 3115 required before computing income under new method.".to_string());
        return Output {
            severity: Severity::ViolationMethodChangeWithoutForm3115Consent,
            compliant: false,
            form_3115_required: true,
            irs_authority_to_change_method: true,
            notes,
            citations,
        };
    }

    if input.different_methods_for_different_trades {
        notes.push("§ 446(d) different methods for different trades or businesses — permitted if each clearly reflects income and is consistently used.".to_string());
        return Output {
            severity: Severity::CompliantDifferentMethodsForDifferentTradesUnderSection446d,
            compliant: true,
            form_3115_required: false,
            irs_authority_to_change_method: false,
            notes,
            citations,
        };
    }

    let severity = match input.accounting_method {
        AccountingMethod::CashReceiptsAndDisbursements => {
            notes.push("§ 446(c)(1) cash receipts and disbursements method — income on receipt, deductions on payment.".to_string());
            Severity::CompliantCashMethodSection446c1
        }
        AccountingMethod::Accrual => {
            notes.push("§ 446(c)(2) accrual method — income on all-events-test fix + amount determinable; deductions on § 461(h) economic performance.".to_string());
            Severity::CompliantAccrualMethodSection446c2
        }
        AccountingMethod::OtherPermitted => {
            notes.push("§ 446(c)(3) other method permitted by chapter (e.g., § 475 MTM, § 453 installment, § 460 long-term contracts).".to_string());
            Severity::CompliantOtherPermittedMethodSection446c3
        }
        AccountingMethod::HybridCombination => {
            notes.push("§ 446(c)(4) hybrid combination — common pattern: accrual for purchases/sales (§ 471 inventory) + cash for other items.".to_string());
            Severity::CompliantHybridMethodSection446c4
        }
        AccountingMethod::NotApplicable => unreachable!(),
    };

    Output {
        severity,
        compliant: true,
        form_3115_required: false,
        irs_authority_to_change_method: false,
        notes,
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_accrual_compliant() -> Input {
        Input {
            accounting_method: AccountingMethod::Accrual,
            different_methods_for_different_trades: false,
            method_clearly_reflects_income: true,
            consistent_treatment_year_to_year: true,
            method_change_made: false,
            form_3115_filed_with_irs_consent: false,
        }
    }

    #[test]
    fn accrual_method_compliant() {
        let out = check(&base_accrual_compliant());
        assert_eq!(out.severity, Severity::CompliantAccrualMethodSection446c2);
        assert!(out.compliant);
    }

    #[test]
    fn cash_method_compliant() {
        let mut i = base_accrual_compliant();
        i.accounting_method = AccountingMethod::CashReceiptsAndDisbursements;
        let out = check(&i);
        assert_eq!(out.severity, Severity::CompliantCashMethodSection446c1);
    }

    #[test]
    fn other_permitted_method_compliant() {
        let mut i = base_accrual_compliant();
        i.accounting_method = AccountingMethod::OtherPermitted;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantOtherPermittedMethodSection446c3
        );
    }

    #[test]
    fn hybrid_method_compliant() {
        let mut i = base_accrual_compliant();
        i.accounting_method = AccountingMethod::HybridCombination;
        let out = check(&i);
        assert_eq!(out.severity, Severity::CompliantHybridMethodSection446c4);
    }

    #[test]
    fn different_methods_for_different_trades_compliant() {
        let mut i = base_accrual_compliant();
        i.different_methods_for_different_trades = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::CompliantDifferentMethodsForDifferentTradesUnderSection446d
        );
    }

    #[test]
    fn method_does_not_clearly_reflect_income_violation() {
        let mut i = base_accrual_compliant();
        i.method_clearly_reflects_income = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationMethodDoesNotClearlyReflectIncome
        );
        assert!(out.irs_authority_to_change_method);
    }

    #[test]
    fn inconsistent_treatment_year_to_year_violation() {
        let mut i = base_accrual_compliant();
        i.consistent_treatment_year_to_year = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationInconsistentTreatmentYearToYear
        );
    }

    #[test]
    fn method_change_without_form_3115_violation() {
        let mut i = base_accrual_compliant();
        i.method_change_made = true;
        i.form_3115_filed_with_irs_consent = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationMethodChangeWithoutForm3115Consent
        );
        assert!(out.form_3115_required);
    }

    #[test]
    fn method_change_with_form_3115_compliant() {
        let mut i = base_accrual_compliant();
        i.method_change_made = true;
        i.form_3115_filed_with_irs_consent = true;
        let out = check(&i);
        assert!(out.compliant);
    }

    #[test]
    fn not_applicable_returns_default() {
        let mut i = base_accrual_compliant();
        i.accounting_method = AccountingMethod::NotApplicable;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn citations_pin_446_subsections() {
        let out = check(&base_accrual_compliant());
        assert!(out.citations.iter().any(|c| c.contains("§ 446(a)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 446(b)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 446(c)(1)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 446(c)(2)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 446(c)(3)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 446(c)(4)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 446(d)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 446(e)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 446(f)")));
    }

    #[test]
    fn citations_pin_461h_471_cross_refs() {
        let out = check(&base_accrual_compliant());
        assert!(out.citations.iter().any(|c| c.contains("§ 461(h)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 471")));
    }

    #[test]
    fn citations_pin_treas_reg_1_446_1_consistency_and_form_3115() {
        let out = check(&base_accrual_compliant());
        assert!(out.citations.iter().any(|c| c.contains("§ 1.446-1")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.446-1(a)(2)")));
        assert!(out.citations.iter().any(|c| c.contains("Form 3115")));
        assert!(out
            .citations
            .iter()
            .any(|c| c.contains("Rev. Proc. 2024-23")));
    }

    #[test]
    fn constant_pin_cash_method_id_1() {
        assert_eq!(METHOD_CASH_RECEIPTS_AND_DISBURSEMENTS, 1);
    }

    #[test]
    fn constant_pin_accrual_method_id_2() {
        assert_eq!(METHOD_ACCRUAL, 2);
    }

    #[test]
    fn constant_pin_other_permitted_method_id_3() {
        assert_eq!(METHOD_OTHER_PERMITTED_BY_CHAPTER, 3);
    }

    #[test]
    fn constant_pin_hybrid_combination_method_id_4() {
        assert_eq!(METHOD_COMBINATION_HYBRID, 4);
    }
}
