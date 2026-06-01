//! Mandatory annual security deposit interest statement disclosure
//! — when must a landlord affirmatively provide a written statement
//! to the tenant detailing security deposit interest accrued during
//! the year? Distinct from `deposit_interest` (which addresses
//! whether interest is required and at what rate) and `security_
//! deposit_bank_disclosure` (which addresses initial bank location
//! disclosure at tenancy start).
//!
//! Trader-landlord operational concern in 4 regimes that mandate
//! annual statement disclosure — failure to provide the statement
//! is a separate violation from failure to pay interest, often with
//! independent statutory penalties.
//!
//! Four regimes:
//!
//! **Massachusetts — Mass. G.L. c. 186 § 15B(2)(c)(ii)**.
//! ANNUAL STATEMENT REQUIRED at end of each year of tenancy with
//! (a) bank name and address, (b) deposit amount, (c) account
//! number, (d) interest payable. § 15B(2)(b) — interest at 5% per
//! year OR lesser actual bank rate. § 15B(2)(c) — interest paid
//! over to tenant each year OR notification that tenant may deduct
//! interest from next rental payment. § 15B(7) — willful violation
//! triggers TRIPLE damages plus 5% interest plus attorney fees.
//!
//! **New Jersey — N.J.S.A. 46:8-19(c)**. ANNUAL INTEREST PAYMENT
//! AND STATEMENT required. Landlord must provide written statement
//! and pay interest (less 1% allowed for landlord administrative
//! cost) annually to tenant. § 46:8-21.1 — willful failure to
//! comply triggers DOUBLE damages plus attorney fees.
//!
//! **Chicago (IL) — Chicago RLTO § 5-12-080(c)**. ANNUAL INTEREST
//! PAYMENT AND STATEMENT required within 30 days after end of
//! 12-month rental period. Statement must specify deposit amount,
//! interest amount, and how interest was calculated. § 5-12-080(f)
//! — willful failure triggers DOUBLE damages plus attorney fees.
//!
//! **New York — N.Y. Gen. Oblig. Law § 7-103**. TRUST FUND
//! requirement. Landlord must hold deposit in trust (not commingled
//! with personal funds). For 6+ unit buildings, deposit must be in
//! interest-bearing account with bank name/address provided in
//! writing. No statutory ANNUAL statement requirement (initial
//! disclosure only).
//!
//! **Default — varies**. Most US states lack a statutory annual
//! statement requirement. Some require interest payment without
//! statement.
//!
//! Citations: Mass. G.L. c. 186 § 15B(2)(c)(ii) (MA annual
//! statement); § 15B(2)(b) (MA 5% rate); § 15B(2)(c) (MA payment-
//! or-deduction); § 15B(7) (MA triple damages); N.J.S.A.
//! 46:8-19(c) (NJ annual interest payment + statement); N.J.S.A.
//! 46:8-21.1 (NJ double damages willful); Chicago RLTO § 5-12-080
//! (c)/(f) (Chicago 30-day statement); N.Y. Gen. Oblig. Law § 7-103
//! (NY trust fund + initial disclosure).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Massachusetts,
    NewJersey,
    Chicago,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DepositInterestStatementInput {
    pub regime: Regime,
    /// Whether the landlord provided an annual statement to the
    /// tenant.
    pub annual_statement_provided: bool,
    /// Whether the statement includes the bank name and address
    /// (MA + NY).
    pub statement_includes_bank_name_address: bool,
    /// Whether the statement includes the account number (MA).
    pub statement_includes_account_number: bool,
    /// Whether the statement includes the deposit amount (MA + IL
    /// Chicago).
    pub statement_includes_deposit_amount: bool,
    /// Whether the statement includes the interest amount (all
    /// statement-required regimes).
    pub statement_includes_interest_amount: bool,
    /// Chicago RLTO — whether the statement also explains how the
    /// interest was calculated.
    pub statement_explains_interest_calculation: bool,
    /// Whether the statement was provided within the statutory
    /// timeframe (MA: end of tenancy year; Chicago: 30 days after
    /// end of 12-month period; NJ: annually).
    pub statement_provided_within_timeframe: bool,
    /// MA-only — whether the landlord either paid the interest or
    /// notified the tenant of right to deduct from next rental
    /// payment.
    pub ma_interest_payment_or_deduction_option_provided: bool,
    /// Whether the violation is willful (drives damages multiplier).
    pub willful_violation: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DepositInterestStatementResult {
    pub compliant: bool,
    pub violations: Vec<String>,
    pub damages_multiplier: u32,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &DepositInterestStatementInput) -> DepositInterestStatementResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let mut damages_multiplier = 1;

    match input.regime {
        Regime::Massachusetts => check_massachusetts(input, &mut violations, &mut notes, &mut damages_multiplier),
        Regime::NewJersey => check_new_jersey(input, &mut violations, &mut notes, &mut damages_multiplier),
        Regime::Chicago => check_chicago(input, &mut violations, &mut notes, &mut damages_multiplier),
        Regime::NewYork => check_new_york(input, &mut violations, &mut notes),
        Regime::Default => check_default(input, &mut notes),
    }
}

fn check_massachusetts(
    input: &DepositInterestStatementInput,
    violations: &mut Vec<String>,
    notes: &mut Vec<String>,
    damages_multiplier: &mut u32,
) -> DepositInterestStatementResult {
    if !input.annual_statement_provided {
        violations.push(
            "Mass. G.L. c. 186 § 15B(2)(c)(ii) — annual statement required at end of each year of tenancy".to_string(),
        );
    } else {
        if !input.statement_includes_bank_name_address {
            violations.push(
                "Mass. G.L. c. 186 § 15B(2)(c)(ii) — annual statement must include bank name AND address".to_string(),
            );
        }
        if !input.statement_includes_account_number {
            violations.push(
                "Mass. G.L. c. 186 § 15B(2)(c)(ii) — annual statement must include account number".to_string(),
            );
        }
        if !input.statement_includes_deposit_amount {
            violations.push(
                "Mass. G.L. c. 186 § 15B(2)(c)(ii) — annual statement must include deposit amount".to_string(),
            );
        }
        if !input.statement_includes_interest_amount {
            violations.push(
                "Mass. G.L. c. 186 § 15B(2)(c)(ii) — annual statement must include interest payable amount".to_string(),
            );
        }
        if !input.statement_provided_within_timeframe {
            violations.push(
                "Mass. G.L. c. 186 § 15B(2)(c)(ii) — statement must be provided AT END of each year of tenancy".to_string(),
            );
        }
    }

    if !input.ma_interest_payment_or_deduction_option_provided {
        violations.push(
            "Mass. G.L. c. 186 § 15B(2)(c) — landlord must either PAY the interest OR provide notification that tenant may deduct interest from next rental payment".to_string(),
        );
    }

    notes.push(
        "Mass. G.L. c. 186 § 15B(2)(b) — interest at 5% per year OR lesser actual bank rate"
            .to_string(),
    );

    if !violations.is_empty() && input.willful_violation {
        *damages_multiplier = 3;
        notes.push(
            "Mass. G.L. c. 186 § 15B(7) — willful violation triggers TRIPLE damages plus 5% interest plus attorney fees".to_string(),
        );
    }

    DepositInterestStatementResult {
        compliant: violations.is_empty(),
        violations: violations.clone(),
        damages_multiplier: *damages_multiplier,
        citation: citation_for(Regime::Massachusetts),
        notes: notes.clone(),
    }
}

fn check_new_jersey(
    input: &DepositInterestStatementInput,
    violations: &mut Vec<String>,
    notes: &mut Vec<String>,
    damages_multiplier: &mut u32,
) -> DepositInterestStatementResult {
    if !input.annual_statement_provided {
        violations.push(
            "N.J.S.A. 46:8-19(c) — annual interest payment AND statement required".to_string(),
        );
    } else {
        if !input.statement_includes_deposit_amount {
            violations.push(
                "N.J.S.A. 46:8-19(c) — statement must include deposit amount".to_string(),
            );
        }
        if !input.statement_includes_interest_amount {
            violations.push(
                "N.J.S.A. 46:8-19(c) — statement must include interest amount".to_string(),
            );
        }
        if !input.statement_provided_within_timeframe {
            violations.push(
                "N.J.S.A. 46:8-19(c) — statement must be provided annually".to_string(),
            );
        }
    }

    notes.push(
        "N.J.S.A. 46:8-19(c) — landlord may retain 1% of interest as administrative cost; remainder paid to tenant".to_string(),
    );

    if !violations.is_empty() && input.willful_violation {
        *damages_multiplier = 2;
        notes.push(
            "N.J.S.A. 46:8-21.1 — willful failure triggers DOUBLE damages plus attorney fees".to_string(),
        );
    }

    DepositInterestStatementResult {
        compliant: violations.is_empty(),
        violations: violations.clone(),
        damages_multiplier: *damages_multiplier,
        citation: citation_for(Regime::NewJersey),
        notes: notes.clone(),
    }
}

fn check_chicago(
    input: &DepositInterestStatementInput,
    violations: &mut Vec<String>,
    notes: &mut Vec<String>,
    damages_multiplier: &mut u32,
) -> DepositInterestStatementResult {
    if !input.annual_statement_provided {
        violations.push(
            "Chicago RLTO § 5-12-080(c) — annual interest payment AND statement required within 30 days after end of 12-month rental period".to_string(),
        );
    } else {
        if !input.statement_includes_deposit_amount {
            violations.push(
                "Chicago RLTO § 5-12-080(c) — statement must specify deposit amount".to_string(),
            );
        }
        if !input.statement_includes_interest_amount {
            violations.push(
                "Chicago RLTO § 5-12-080(c) — statement must specify interest amount".to_string(),
            );
        }
        if !input.statement_explains_interest_calculation {
            violations.push(
                "Chicago RLTO § 5-12-080(c) — statement must explain how interest was calculated".to_string(),
            );
        }
        if !input.statement_provided_within_timeframe {
            violations.push(
                "Chicago RLTO § 5-12-080(c) — statement must be provided WITHIN 30 DAYS after end of 12-month rental period".to_string(),
            );
        }
    }

    if !violations.is_empty() && input.willful_violation {
        *damages_multiplier = 2;
        notes.push(
            "Chicago RLTO § 5-12-080(f) — willful failure triggers DOUBLE damages plus attorney fees".to_string(),
        );
    }

    DepositInterestStatementResult {
        compliant: violations.is_empty(),
        violations: violations.clone(),
        damages_multiplier: *damages_multiplier,
        citation: citation_for(Regime::Chicago),
        notes: notes.clone(),
    }
}

fn check_new_york(
    input: &DepositInterestStatementInput,
    violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> DepositInterestStatementResult {
    notes.push(
        "N.Y. Gen. Oblig. Law § 7-103 — TRUST FUND requirement; deposit must be held in trust (not commingled with personal funds); no statutory ANNUAL statement requirement"
            .to_string(),
    );
    notes.push(
        "§ 7-103(2) — for 6+ unit buildings, deposit must be in interest-bearing account with bank name AND address provided in writing to tenant"
            .to_string(),
    );
    if !input.statement_includes_bank_name_address {
        violations.push(
            "N.Y. Gen. Oblig. Law § 7-103(2) — bank name and address required for 6+ unit buildings (initial disclosure)".to_string(),
        );
    }

    DepositInterestStatementResult {
        compliant: violations.is_empty(),
        violations: violations.clone(),
        damages_multiplier: 1,
        citation: citation_for(Regime::NewYork),
        notes: notes.clone(),
    }
}

fn check_default(
    _input: &DepositInterestStatementInput,
    notes: &mut Vec<String>,
) -> DepositInterestStatementResult {
    notes.push(
        "default rule — most US states lack statutory annual statement requirement; deposit_interest module addresses interest-payment obligation separately".to_string(),
    );
    DepositInterestStatementResult {
        compliant: true,
        violations: Vec::new(),
        damages_multiplier: 1,
        citation: citation_for(Regime::Default),
        notes: notes.clone(),
    }
}

fn citation_for(regime: Regime) -> &'static str {
    match regime {
        Regime::Massachusetts => "Mass. G.L. c. 186 § 15B(2)(b)/(c)/(c)(ii)/(7)",
        Regime::NewJersey => "N.J.S.A. 46:8-19(c); N.J.S.A. 46:8-21.1",
        Regime::Chicago => "Chicago RLTO § 5-12-080(c)/(f)",
        Regime::NewYork => "N.Y. Gen. Oblig. Law § 7-103/(2)",
        Regime::Default => "no statewide annual statement statute; deposit_interest module covers interest-payment obligation",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ma_base() -> DepositInterestStatementInput {
        DepositInterestStatementInput {
            regime: Regime::Massachusetts,
            annual_statement_provided: true,
            statement_includes_bank_name_address: true,
            statement_includes_account_number: true,
            statement_includes_deposit_amount: true,
            statement_includes_interest_amount: true,
            statement_explains_interest_calculation: false,
            statement_provided_within_timeframe: true,
            ma_interest_payment_or_deduction_option_provided: true,
            willful_violation: false,
        }
    }

    fn nj_base() -> DepositInterestStatementInput {
        DepositInterestStatementInput {
            regime: Regime::NewJersey,
            annual_statement_provided: true,
            statement_includes_bank_name_address: false,
            statement_includes_account_number: false,
            statement_includes_deposit_amount: true,
            statement_includes_interest_amount: true,
            statement_explains_interest_calculation: false,
            statement_provided_within_timeframe: true,
            ma_interest_payment_or_deduction_option_provided: false,
            willful_violation: false,
        }
    }

    fn chicago_base() -> DepositInterestStatementInput {
        DepositInterestStatementInput {
            regime: Regime::Chicago,
            annual_statement_provided: true,
            statement_includes_bank_name_address: false,
            statement_includes_account_number: false,
            statement_includes_deposit_amount: true,
            statement_includes_interest_amount: true,
            statement_explains_interest_calculation: true,
            statement_provided_within_timeframe: true,
            ma_interest_payment_or_deduction_option_provided: false,
            willful_violation: false,
        }
    }

    fn ny_base() -> DepositInterestStatementInput {
        DepositInterestStatementInput {
            regime: Regime::NewYork,
            annual_statement_provided: false,
            statement_includes_bank_name_address: true,
            statement_includes_account_number: false,
            statement_includes_deposit_amount: false,
            statement_includes_interest_amount: false,
            statement_explains_interest_calculation: false,
            statement_provided_within_timeframe: false,
            ma_interest_payment_or_deduction_option_provided: false,
            willful_violation: false,
        }
    }

    #[test]
    fn ma_full_compliance_passes() {
        let r = check(&ma_base());
        assert!(r.compliant);
        assert_eq!(r.damages_multiplier, 1);
    }

    #[test]
    fn ma_missing_statement_violation() {
        let mut i = ma_base();
        i.annual_statement_provided = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 15B(2)(c)(ii)") && v.contains("annual statement")));
    }

    #[test]
    fn ma_missing_account_number_violation() {
        let mut i = ma_base();
        i.statement_includes_account_number = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("account number")));
    }

    #[test]
    fn ma_missing_bank_name_address_violation() {
        let mut i = ma_base();
        i.statement_includes_bank_name_address = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("bank name AND address")));
    }

    #[test]
    fn ma_missing_deposit_amount_violation() {
        let mut i = ma_base();
        i.statement_includes_deposit_amount = false;
        let r = check(&i);
        assert!(!r.compliant);
    }

    #[test]
    fn ma_missing_interest_amount_violation() {
        let mut i = ma_base();
        i.statement_includes_interest_amount = false;
        let r = check(&i);
        assert!(!r.compliant);
    }

    #[test]
    fn ma_late_statement_violation() {
        let mut i = ma_base();
        i.statement_provided_within_timeframe = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("AT END of each year")));
    }

    #[test]
    fn ma_no_payment_or_deduction_option_violation() {
        let mut i = ma_base();
        i.ma_interest_payment_or_deduction_option_provided = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 15B(2)(c)") && v.contains("PAY the interest")));
    }

    #[test]
    fn ma_willful_violation_triples_damages() {
        let mut i = ma_base();
        i.annual_statement_provided = false;
        i.willful_violation = true;
        let r = check(&i);
        assert_eq!(r.damages_multiplier, 3);
        assert!(r.notes.iter().any(|n| n.contains("§ 15B(7)") && n.contains("TRIPLE damages")));
    }

    #[test]
    fn ma_5_percent_rate_note_always_present() {
        let r = check(&ma_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 15B(2)(b)") && n.contains("5%")));
    }

    #[test]
    fn nj_full_compliance_passes() {
        let r = check(&nj_base());
        assert!(r.compliant);
    }

    #[test]
    fn nj_missing_statement_violation() {
        let mut i = nj_base();
        i.annual_statement_provided = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("46:8-19(c)") && v.contains("annual")));
    }

    #[test]
    fn nj_willful_violation_doubles_damages() {
        let mut i = nj_base();
        i.annual_statement_provided = false;
        i.willful_violation = true;
        let r = check(&i);
        assert_eq!(r.damages_multiplier, 2);
        assert!(r.notes.iter().any(|n| n.contains("46:8-21.1") && n.contains("DOUBLE damages")));
    }

    #[test]
    fn nj_1_percent_admin_cost_note_present() {
        let r = check(&nj_base());
        assert!(r.notes.iter().any(|n| n.contains("1% of interest as administrative cost")));
    }

    #[test]
    fn chicago_full_compliance_passes() {
        let r = check(&chicago_base());
        assert!(r.compliant);
    }

    #[test]
    fn chicago_missing_calculation_explanation_violation() {
        let mut i = chicago_base();
        i.statement_explains_interest_calculation = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("how interest was calculated")));
    }

    #[test]
    fn chicago_missing_30_day_timeframe_violation() {
        let mut i = chicago_base();
        i.statement_provided_within_timeframe = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("WITHIN 30 DAYS")));
    }

    #[test]
    fn chicago_willful_violation_doubles_damages() {
        let mut i = chicago_base();
        i.annual_statement_provided = false;
        i.willful_violation = true;
        let r = check(&i);
        assert_eq!(r.damages_multiplier, 2);
        assert!(r.notes.iter().any(|n| n.contains("§ 5-12-080(f)") && n.contains("DOUBLE")));
    }

    #[test]
    fn ny_trust_fund_note_always_present() {
        let r = check(&ny_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7-103") && n.contains("TRUST FUND")));
    }

    #[test]
    fn ny_no_annual_statement_requirement_compliant_with_initial_disclosure() {
        let r = check(&ny_base());
        assert!(r.compliant);
        assert!(r.notes.iter().any(|n| n.contains("no statutory ANNUAL statement requirement")));
    }

    #[test]
    fn ny_missing_bank_name_address_initial_disclosure_violation() {
        let mut i = ny_base();
        i.statement_includes_bank_name_address = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 7-103(2)") && v.contains("6+ unit")));
    }

    #[test]
    fn default_no_statement_requirement() {
        let mut i = DepositInterestStatementInput {
            regime: Regime::Default,
            annual_statement_provided: false,
            statement_includes_bank_name_address: false,
            statement_includes_account_number: false,
            statement_includes_deposit_amount: false,
            statement_includes_interest_amount: false,
            statement_explains_interest_calculation: false,
            statement_provided_within_timeframe: false,
            ma_interest_payment_or_deduction_option_provided: false,
            willful_violation: false,
        };
        let r = check(&i);
        assert!(r.compliant);
        assert!(r.notes.iter().any(|n| n.contains("most US states lack")));
        let _ = &mut i;
    }

    #[test]
    fn ma_unique_account_number_requirement_invariant() {
        let mut i_ma = ma_base();
        i_ma.statement_includes_account_number = false;
        let r_ma = check(&i_ma);
        assert!(!r_ma.compliant);

        for regime in [Regime::NewJersey, Regime::Chicago, Regime::NewYork, Regime::Default] {
            let mut i = ma_base();
            i.regime = regime;
            i.statement_includes_account_number = false;
            let r = check(&i);
            let account_violations: Vec<_> = r.violations.iter().filter(|v| v.contains("account number")).collect();
            assert!(account_violations.is_empty(), "regime {:?} does not require account number disclosure", regime);
        }
    }

    #[test]
    fn chicago_unique_calculation_explanation_invariant() {
        let mut i_chicago = chicago_base();
        i_chicago.statement_explains_interest_calculation = false;
        let r_chicago = check(&i_chicago);
        assert!(!r_chicago.compliant);

        for regime in [Regime::Massachusetts, Regime::NewJersey, Regime::NewYork, Regime::Default] {
            let mut i = chicago_base();
            i.regime = regime;
            i.statement_explains_interest_calculation = false;
            i.statement_includes_bank_name_address = true;
            i.statement_includes_account_number = true;
            i.ma_interest_payment_or_deduction_option_provided = true;
            let r = check(&i);
            let calc_violations: Vec<_> = r.violations.iter().filter(|v| v.contains("how interest was calculated")).collect();
            assert!(calc_violations.is_empty(), "regime {:?} does not require calculation explanation", regime);
        }
    }

    #[test]
    fn ma_uniquely_triples_damages_invariant() {
        let regimes_with_multiplier = [
            (Regime::Massachusetts, 3),
            (Regime::NewJersey, 2),
            (Regime::Chicago, 2),
        ];
        for (regime, expected_multiplier) in regimes_with_multiplier {
            let mut i = ma_base();
            i.regime = regime;
            i.annual_statement_provided = false;
            i.willful_violation = true;
            i.statement_includes_bank_name_address = true;
            i.statement_includes_account_number = true;
            i.statement_explains_interest_calculation = true;
            i.ma_interest_payment_or_deduction_option_provided = true;
            let r = check(&i);
            assert_eq!(r.damages_multiplier, expected_multiplier, "regime {:?}", regime);
        }
    }

    #[test]
    fn citation_ma_pins_subsections() {
        let r = check(&ma_base());
        assert!(r.citation.contains("§ 15B(2)(b)"));
        assert!(r.citation.contains("(c)"));
        assert!(r.citation.contains("(c)(ii)"));
        assert!(r.citation.contains("(7)"));
    }

    #[test]
    fn citation_nj_pins_46_8_19c_and_21_1() {
        let r = check(&nj_base());
        assert!(r.citation.contains("46:8-19(c)"));
        assert!(r.citation.contains("46:8-21.1"));
    }

    #[test]
    fn citation_chicago_pins_rlto_subsections() {
        let r = check(&chicago_base());
        assert!(r.citation.contains("§ 5-12-080(c)"));
        assert!(r.citation.contains("(f)"));
    }

    #[test]
    fn citation_ny_pins_7_103_and_subsection_2() {
        let r = check(&ny_base());
        assert!(r.citation.contains("§ 7-103"));
        assert!(r.citation.contains("(2)"));
    }

    #[test]
    fn willful_violation_without_underlying_violation_no_multiplier() {
        let mut i = ma_base();
        i.willful_violation = true;
        let r = check(&i);
        assert_eq!(r.damages_multiplier, 1);
    }

    #[test]
    fn ma_multiple_violations_accumulate() {
        let mut i = ma_base();
        i.annual_statement_provided = false;
        i.ma_interest_payment_or_deduction_option_provided = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.len() >= 2);
    }
}
