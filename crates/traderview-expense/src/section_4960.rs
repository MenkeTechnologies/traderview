//! IRC § 4960 — Tax on excess tax-exempt
//! organization executive compensation. Excise tax on
//! applicable tax-exempt organizations (ATEOs) for
//! compensation paid to covered employees. Direct
//! companion to section_162m ($1M public company deduction
//! cap — iter 446), section_4973 (excess contribution
//! excise — iter 442), section_4974 (RMD excise — iter
//! 436), section_4975 (prohibited transactions — iter
//! 434), section_4980 (employer reversion excise — iter
//! 460), section_4980h (employer shared responsibility —
//! iter 456), section_409a (NQDC).
//!
//! § 4960(a) imposes a 21% EXCISE TAX on an ATEO equal to
//! the sum of (1) remuneration paid by the ATEO to a
//! covered employee for the taxable year exceeding
//! $1,000,000, plus (2) any "excess parachute payment"
//! paid by the ATEO to a covered employee.
//!
//! Applicable tax-exempt organization (ATEO) per
//! § 4960(c)(1):
//! - § 501(a) exempt org (most public charities, private
//!   foundations, § 501(c)(3) hospitals, universities)
//! - § 521(b)(1) exempt farmers cooperative
//! - § 115(1) excluded-income state/political subdivision
//!   instrumentality
//! - § 527(e)(1) political organization
//!
//! Covered employee per § 4960(c)(2) — TWO REGIMES:
//! Pre-OBBBA regime (tax years 2018-2025): covered employee
//! is one of the FIVE highest-compensated employees of the
//! ATEO for the taxable year, OR any preceding tax year
//! beginning after 12/31/2016. Once covered, always covered
//! ("forever covered" rule).
//! Post-OBBBA regime (tax years beginning after 12/31/2025
//! per One Big Beautiful Bill Act, Pub. L. 119-21, signed
//! July 4, 2025): the five-employee cap is REMOVED. Any
//! current or former employee receiving remuneration over
//! $1,000,000 from the ATEO triggers § 4960 tax.
//!
//! Remuneration per § 4960(c)(3) means wages within the
//! meaning of § 3401(a) EXCEPT designated Roth
//! contributions plus deferred compensation amounts
//! required to be included in income under § 457(f) (i.e.,
//! includes amounts vesting under § 457(f) plans even
//! before actual payment). Excludes payment for medical
//! services performed by a licensed medical professional
//! (doctor, nurse, veterinarian) — § 4960(c)(3)(B).
//!
//! Excess parachute payment per § 4960(c)(5): a parachute
//! payment is one or more payments in the nature of
//! compensation to a covered employee that are CONTINGENT
//! ON SEPARATION FROM EMPLOYMENT, the aggregate present
//! value of which equals or exceeds 3× the BASE AMOUNT
//! (employee's average annualized compensation over the
//! 5-year base period). If the 3× threshold is met, the
//! "excess parachute payment" is the amount of the
//! parachute payment that exceeds 1× the base amount (NOT
//! the 3× threshold). § 4960(c)(5)(B). Modeled on § 280G
//! corporate golden parachute rules.
//!
//! Coordination with § 162(m) — § 4960 does NOT apply to
//! amounts that are subject to § 162(m) $1M deduction cap
//! (which applies to publicly held corporations), per
//! § 4960(c)(7).
//!
//! Final Treasury Regulations under § 4960 issued
//! January 19, 2021 — 26 C.F.R. § 53.4960-0 through
//! § 53.4960-6.
//!
//! Trader-critical because traders sitting on boards of
//! tax-exempt hospital systems, universities, foundations,
//! political organizations face § 4960 21% excise tax
//! exposure on executive compensation paid to the
//! organization's highest-paid employees; smart structuring
//! considers medical-services exclusion, related-org
//! attribution under § 4960(c)(4), and golden parachute
//! mitigation through § 280G-style modeling.
//!
//! Authority: 26 U.S.C. § 4960; § 4960(a); § 4960(b);
//! § 4960(c)(1); § 4960(c)(2); § 4960(c)(3); § 4960(c)(4);
//! § 4960(c)(5); § 4960(c)(5)(B); § 4960(c)(6); § 4960(c)(7);
//! 26 C.F.R. § 53.4960-0; 26 C.F.R. § 53.4960-1;
//! 26 C.F.R. § 53.4960-2; 26 C.F.R. § 53.4960-3;
//! 26 C.F.R. § 53.4960-4; 26 C.F.R. § 53.4960-5;
//! 26 C.F.R. § 53.4960-6; Tax Cuts and Jobs Act,
//! Pub. L. 115-97 (Dec. 22, 2017) — original enactment;
//! One Big Beautiful Bill Act, Pub. L. 119-21
//! (July 4, 2025) — removed five-employee cap.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OrganizationType {
    Section501aExempt,
    FarmersCoop521b1,
    Section115Excluded,
    PoliticalOrg527e1,
    NonExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TaxYearRegime {
    PreObbba,
    PostObbba,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmployeeCompensation {
    pub name: String,
    pub remuneration_cents: u64,
    pub base_amount_cents: u64,
    pub parachute_payment_cents: u64,
    pub is_top_five_paid_current_or_prior_year: bool,
    pub is_licensed_medical_professional: bool,
    pub medical_services_remuneration_cents: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub organization_type: OrganizationType,
    pub tax_year_regime: TaxYearRegime,
    pub employees: Vec<EmployeeCompensation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Compliant,
    ExciseTaxOwed,
    OrgNotAteo,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EmployeeResult {
    pub name: String,
    pub is_covered: bool,
    pub non_medical_remuneration_cents: u64,
    pub remuneration_excess_cents: u64,
    pub parachute_triggered: bool,
    pub excess_parachute_cents: u64,
    pub excise_tax_cents: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub severity: Severity,
    pub total_excise_tax_cents: u64,
    pub per_employee: Vec<EmployeeResult>,
    pub notes: Vec<String>,
}

pub const REMUNERATION_THRESHOLD_CENTS: u64 = 100_000_000;
pub const EXCISE_TAX_RATE_PCT: u64 = 21;
pub const PARACHUTE_TRIGGER_MULTIPLIER: u64 = 3;

pub type Section4960Input = Input;
pub type Section4960Result = Output;

fn is_ateo(t: OrganizationType) -> bool {
    matches!(
        t,
        OrganizationType::Section501aExempt
            | OrganizationType::FarmersCoop521b1
            | OrganizationType::Section115Excluded
            | OrganizationType::PoliticalOrg527e1
    )
}

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let mut per_employee: Vec<EmployeeResult> = Vec::new();

    notes.push(
        "§ 4960(a) imposes 21% excise tax on ATEO equal to (i) remuneration over $1,000,000 plus (ii) excess parachute payment to covered employee — 4-jurisdiction-style framework: (1) ATEO definition § 4960(c)(1); (2) covered employee § 4960(c)(2); (3) remuneration § 4960(c)(3); (4) excess parachute § 4960(c)(5)."
            .to_string(),
    );
    notes.push(
        "ATEO per § 4960(c)(1): § 501(a) exempt org + § 521(b)(1) farmers coop + § 115(1) state/political subdivision instrumentality + § 527(e)(1) political organization."
            .to_string(),
    );
    notes.push(
        "Covered employee per § 4960(c)(2) — PRE-OBBBA (2018-2025): one of FIVE highest-compensated employees for taxable year OR any preceding tax year beginning after 12/31/2016 (forever-covered rule). POST-OBBBA (after 12/31/2025): ANY current or former employee receiving over $1M triggers tax — five-employee cap REMOVED by Pub. L. 119-21 (July 4, 2025)."
            .to_string(),
    );
    notes.push(
        "Remuneration per § 4960(c)(3): § 3401(a) wages excluding designated Roth + § 457(f) vested deferred comp. EXCLUDES medical services by licensed medical professional (doctor, nurse, veterinarian) — § 4960(c)(3)(B)."
            .to_string(),
    );
    notes.push(
        "Excess parachute payment per § 4960(c)(5): aggregate parachute payments contingent on SEPARATION FROM EMPLOYMENT with present value equal to or exceeding 3× base amount (employee 5-year average compensation) triggers tax on amount EXCEEDING 1× base — modeled on § 280G golden parachute rules."
            .to_string(),
    );
    notes.push(
        "Coordination with § 162(m): § 4960 does NOT apply to amounts subject to § 162(m) $1M deduction cap (publicly held corporations) — § 4960(c)(7)."
            .to_string(),
    );
    notes.push(
        "Companion: section_162m (iter 446), section_4973 (iter 442), section_4974 (iter 436), section_4975 (iter 434), section_4980 (iter 460), section_4980h (iter 456), section_409a."
            .to_string(),
    );

    if !is_ateo(input.organization_type) {
        notes.push(
            "Organization is NOT an applicable tax-exempt organization — § 4960 does not apply."
                .to_string(),
        );
        return Output {
            severity: Severity::OrgNotAteo,
            total_excise_tax_cents: 0,
            per_employee,
            notes,
        };
    }

    let mut total_tax: u64 = 0;

    for emp in &input.employees {
        let non_medical = emp
            .remuneration_cents
            .saturating_sub(emp.medical_services_remuneration_cents);

        let is_covered = match input.tax_year_regime {
            TaxYearRegime::PreObbba => emp.is_top_five_paid_current_or_prior_year,
            TaxYearRegime::PostObbba => non_medical > REMUNERATION_THRESHOLD_CENTS,
        };

        let remuneration_excess = if is_covered {
            non_medical.saturating_sub(REMUNERATION_THRESHOLD_CENTS)
        } else {
            0
        };

        let parachute_trigger_amount = emp
            .base_amount_cents
            .saturating_mul(PARACHUTE_TRIGGER_MULTIPLIER);
        let parachute_triggered =
            is_covered && emp.parachute_payment_cents >= parachute_trigger_amount;
        let excess_parachute = if parachute_triggered {
            emp.parachute_payment_cents
                .saturating_sub(emp.base_amount_cents)
        } else {
            0
        };

        let taxable_base = remuneration_excess.saturating_add(excess_parachute);
        let emp_tax = taxable_base
            .saturating_mul(EXCISE_TAX_RATE_PCT)
            .checked_div(100)
            .unwrap_or(0);

        total_tax = total_tax.saturating_add(emp_tax);

        per_employee.push(EmployeeResult {
            name: emp.name.clone(),
            is_covered,
            non_medical_remuneration_cents: non_medical,
            remuneration_excess_cents: remuneration_excess,
            parachute_triggered,
            excess_parachute_cents: excess_parachute,
            excise_tax_cents: emp_tax,
        });
    }

    let severity = if total_tax > 0 {
        Severity::ExciseTaxOwed
    } else {
        Severity::Compliant
    };

    if severity == Severity::ExciseTaxOwed {
        notes.push(format!(
            "Total § 4960 excise tax owed by ATEO: ${}.{:02}",
            total_tax / 100,
            total_tax % 100
        ));
    } else {
        notes.push("No § 4960 excise tax owed for this taxable year.".to_string());
    }

    Output {
        severity,
        total_excise_tax_cents: total_tax,
        per_employee,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn emp(
        name: &str,
        remun: u64,
        base: u64,
        parachute: u64,
        top5: bool,
        med_pro: bool,
        med_remun: u64,
    ) -> EmployeeCompensation {
        EmployeeCompensation {
            name: name.to_string(),
            remuneration_cents: remun,
            base_amount_cents: base,
            parachute_payment_cents: parachute,
            is_top_five_paid_current_or_prior_year: top5,
            is_licensed_medical_professional: med_pro,
            medical_services_remuneration_cents: med_remun,
        }
    }

    #[test]
    fn ateo_pre_obbba_below_threshold_no_tax() {
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![emp("CEO", 900_000_00, 500_000_00, 0, true, false, 0)],
        };
        let out = check(&input);
        assert_eq!(out.severity, Severity::Compliant);
        assert_eq!(out.total_excise_tax_cents, 0);
        assert_eq!(out.per_employee[0].remuneration_excess_cents, 0);
    }

    #[test]
    fn ateo_pre_obbba_one_point_five_million_one_hundred_five_k_tax() {
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![emp("CEO", 1_500_000_00, 800_000_00, 0, true, false, 0)],
        };
        let out = check(&input);
        assert_eq!(out.severity, Severity::ExciseTaxOwed);
        assert_eq!(out.per_employee[0].remuneration_excess_cents, 500_000_00);
        // 21% of $500K = $105,000
        assert_eq!(out.total_excise_tax_cents, 105_000_00);
    }

    #[test]
    fn ateo_pre_obbba_not_top_five_no_tax() {
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![emp("Junior", 1_500_000_00, 800_000_00, 0, false, false, 0)],
        };
        let out = check(&input);
        assert_eq!(out.severity, Severity::Compliant);
        assert_eq!(out.total_excise_tax_cents, 0);
        assert!(!out.per_employee[0].is_covered);
    }

    #[test]
    fn ateo_post_obbba_no_top_five_cap_any_employee_over_one_m() {
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PostObbba,
            employees: vec![emp("Junior", 1_500_000_00, 800_000_00, 0, false, false, 0)],
        };
        let out = check(&input);
        assert_eq!(out.severity, Severity::ExciseTaxOwed);
        assert!(out.per_employee[0].is_covered);
        assert_eq!(out.per_employee[0].remuneration_excess_cents, 500_000_00);
        assert_eq!(out.total_excise_tax_cents, 105_000_00);
    }

    #[test]
    fn non_ateo_no_tax_regardless_of_comp() {
        let input = Input {
            organization_type: OrganizationType::NonExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![emp("CEO", 10_000_000_00, 5_000_000_00, 0, true, false, 0)],
        };
        let out = check(&input);
        assert_eq!(out.severity, Severity::OrgNotAteo);
        assert_eq!(out.total_excise_tax_cents, 0);
    }

    #[test]
    fn farmers_coop_is_ateo() {
        let input = Input {
            organization_type: OrganizationType::FarmersCoop521b1,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![emp("CEO", 1_200_000_00, 700_000_00, 0, true, false, 0)],
        };
        let out = check(&input);
        assert_eq!(out.severity, Severity::ExciseTaxOwed);
        assert_eq!(out.total_excise_tax_cents, 42_000_00); // 21% of $200K
    }

    #[test]
    fn political_org_527e1_is_ateo() {
        let input = Input {
            organization_type: OrganizationType::PoliticalOrg527e1,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![emp("Director", 1_100_000_00, 600_000_00, 0, true, false, 0)],
        };
        let out = check(&input);
        assert_eq!(out.severity, Severity::ExciseTaxOwed);
        assert_eq!(out.total_excise_tax_cents, 21_000_00); // 21% of $100K
    }

    #[test]
    fn section_115_excluded_is_ateo() {
        let input = Input {
            organization_type: OrganizationType::Section115Excluded,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![emp("CEO", 2_000_000_00, 1_000_000_00, 0, true, false, 0)],
        };
        let out = check(&input);
        assert_eq!(out.severity, Severity::ExciseTaxOwed);
        assert_eq!(out.total_excise_tax_cents, 210_000_00); // 21% of $1M
    }

    #[test]
    fn medical_services_exclusion_doctor() {
        // CEO/doctor paid $2M total, $1.5M for medical services
        // Non-medical = $500K, below threshold, no tax
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![emp(
                "Surgeon-CEO",
                2_000_000_00,
                1_000_000_00,
                0,
                true,
                true,
                1_500_000_00,
            )],
        };
        let out = check(&input);
        assert_eq!(out.severity, Severity::Compliant);
        assert_eq!(out.per_employee[0].non_medical_remuneration_cents, 500_000_00);
        assert_eq!(out.total_excise_tax_cents, 0);
    }

    #[test]
    fn medical_services_partial_exclusion_still_taxed() {
        // CEO paid $3M, $1M for medical services
        // Non-medical = $2M, $1M excess, 21% = $210K tax
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![emp(
                "Doctor-CEO",
                3_000_000_00,
                1_500_000_00,
                0,
                true,
                true,
                1_000_000_00,
            )],
        };
        let out = check(&input);
        assert_eq!(out.severity, Severity::ExciseTaxOwed);
        assert_eq!(out.per_employee[0].non_medical_remuneration_cents, 2_000_000_00);
        assert_eq!(out.per_employee[0].remuneration_excess_cents, 1_000_000_00);
        assert_eq!(out.total_excise_tax_cents, 210_000_00);
    }

    #[test]
    fn parachute_under_three_x_base_no_trigger() {
        // Base = $500K, parachute = $1.4M (less than 3×$500K=$1.5M trigger)
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![emp(
                "CEO",
                800_000_00,
                500_000_00,
                1_400_000_00,
                true,
                false,
                0,
            )],
        };
        let out = check(&input);
        assert!(!out.per_employee[0].parachute_triggered);
        assert_eq!(out.per_employee[0].excess_parachute_cents, 0);
        assert_eq!(out.total_excise_tax_cents, 0);
    }

    #[test]
    fn parachute_three_x_base_triggers_excess_at_one_x() {
        // Base = $500K, parachute = $1.5M (exactly 3×base trigger)
        // Excess = $1.5M - $500K = $1M, 21% tax = $210K
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![emp(
                "CEO",
                800_000_00,
                500_000_00,
                1_500_000_00,
                true,
                false,
                0,
            )],
        };
        let out = check(&input);
        assert!(out.per_employee[0].parachute_triggered);
        assert_eq!(out.per_employee[0].excess_parachute_cents, 1_000_000_00);
        assert_eq!(out.total_excise_tax_cents, 210_000_00);
    }

    #[test]
    fn parachute_excess_stacks_with_remuneration_excess() {
        // Remuneration $1.8M (excess $800K), parachute $1.5M (excess $1M)
        // Total taxable = $1.8M, 21% = $378K
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![emp(
                "CEO",
                1_800_000_00,
                500_000_00,
                1_500_000_00,
                true,
                false,
                0,
            )],
        };
        let out = check(&input);
        assert_eq!(out.per_employee[0].remuneration_excess_cents, 800_000_00);
        assert_eq!(out.per_employee[0].excess_parachute_cents, 1_000_000_00);
        assert_eq!(out.total_excise_tax_cents, 378_000_00);
    }

    #[test]
    fn five_covered_employees_aggregated() {
        let mut employees = Vec::new();
        for i in 0..5 {
            employees.push(emp(
                &format!("Exec{}", i),
                2_000_000_00,
                1_000_000_00,
                0,
                true,
                false,
                0,
            ));
        }
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees,
        };
        let out = check(&input);
        // 5 employees × $1M excess × 21% = $1.05M tax
        assert_eq!(out.total_excise_tax_cents, 1_050_000_00);
        assert_eq!(out.severity, Severity::ExciseTaxOwed);
    }

    #[test]
    fn boundary_exactly_one_million_no_tax() {
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![emp("CEO", 1_000_000_00, 500_000_00, 0, true, false, 0)],
        };
        let out = check(&input);
        assert_eq!(out.per_employee[0].remuneration_excess_cents, 0);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn boundary_one_million_one_cent_one_cent_excess() {
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![emp("CEO", 1_000_000_01, 500_000_00, 0, true, false, 0)],
        };
        let out = check(&input);
        assert_eq!(out.per_employee[0].remuneration_excess_cents, 1);
        // 21% of 1 cent = 0 (integer division)
        assert_eq!(out.total_excise_tax_cents, 0);
    }

    #[test]
    fn post_obbba_exactly_one_m_no_tax() {
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PostObbba,
            employees: vec![emp("Worker", 1_000_000_00, 500_000_00, 0, false, false, 0)],
        };
        let out = check(&input);
        // Post-OBBBA covered = non_medical > $1M strictly, so $1M flat is NOT covered
        assert!(!out.per_employee[0].is_covered);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn coordination_section_162m_not_modeled_excludes_publicly_held() {
        // § 4960(c)(7) says § 4960 does not apply to amounts subject to § 162(m).
        // This module models ATEO comp only — § 162(m) lives in section_162m.
        // Verify note pinning so caller knows.
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![],
        };
        let out = check(&input);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 162(m)"));
        assert!(joined.contains("§ 4960(c)(7)"));
    }

    #[test]
    fn citation_pins_all_authorities() {
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![],
        };
        let out = check(&input);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4960(a)"));
        assert!(joined.contains("§ 4960(c)(1)"));
        assert!(joined.contains("§ 4960(c)(2)"));
        assert!(joined.contains("§ 4960(c)(3)"));
        assert!(joined.contains("§ 4960(c)(5)"));
        assert!(joined.contains("§ 501(a)"));
        assert!(joined.contains("§ 521(b)(1)"));
        assert!(joined.contains("§ 115(1)"));
        assert!(joined.contains("§ 527(e)(1)"));
        assert!(joined.contains("§ 457(f)"));
        assert!(joined.contains("§ 280G"));
        assert!(joined.contains("§ 3401(a)"));
    }

    #[test]
    fn note_pins_ateo_four_categories() {
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![],
        };
        let out = check(&input);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 501(a) exempt org"));
        assert!(joined.contains("§ 521(b)(1) farmers coop"));
        assert!(joined.contains("§ 115(1) state/political subdivision"));
        assert!(joined.contains("§ 527(e)(1) political organization"));
    }

    #[test]
    fn note_pins_pre_obbba_five_employee_cap() {
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![],
        };
        let out = check(&input);
        let joined = out.notes.join(" ");
        assert!(joined.contains("FIVE highest-compensated"));
        assert!(joined.contains("12/31/2016"));
        assert!(joined.contains("forever-covered"));
    }

    #[test]
    fn note_pins_post_obbba_cap_removed() {
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PostObbba,
            employees: vec![],
        };
        let out = check(&input);
        let joined = out.notes.join(" ");
        assert!(joined.contains("POST-OBBBA"));
        assert!(joined.contains("12/31/2025"));
        assert!(joined.contains("five-employee cap REMOVED"));
        assert!(joined.contains("Pub. L. 119-21"));
    }

    #[test]
    fn note_pins_medical_services_exclusion() {
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![],
        };
        let out = check(&input);
        let joined = out.notes.join(" ");
        assert!(joined.contains("medical services"));
        assert!(joined.contains("§ 4960(c)(3)(B)"));
        assert!(joined.contains("doctor, nurse, veterinarian"));
    }

    #[test]
    fn note_pins_parachute_three_x_one_x_rule() {
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![],
        };
        let out = check(&input);
        let joined = out.notes.join(" ");
        assert!(joined.contains("3× base amount"));
        assert!(joined.contains("EXCEEDING 1× base"));
        assert!(joined.contains("SEPARATION FROM EMPLOYMENT"));
    }

    #[test]
    fn note_pins_companion_modules() {
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![],
        };
        let out = check(&input);
        let joined = out.notes.join(" ");
        assert!(joined.contains("section_162m"));
        assert!(joined.contains("section_4980"));
        assert!(joined.contains("section_4980h"));
        assert!(joined.contains("section_409a"));
    }

    #[test]
    fn pre_post_obbba_invariant_top_five_covered_in_both() {
        let e = emp("CEO", 1_500_000_00, 800_000_00, 0, true, false, 0);
        let pre = check(&Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![e.clone()],
        });
        let post = check(&Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PostObbba,
            employees: vec![e],
        });
        // Both regimes: top-five executive earning $1.5M is covered, same tax
        assert_eq!(pre.total_excise_tax_cents, post.total_excise_tax_cents);
        assert_eq!(pre.total_excise_tax_cents, 105_000_00);
    }

    #[test]
    fn pre_post_obbba_divergence_non_top_five_over_one_m() {
        let e = emp("Junior", 1_500_000_00, 800_000_00, 0, false, false, 0);
        let pre = check(&Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![e.clone()],
        });
        let post = check(&Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PostObbba,
            employees: vec![e],
        });
        // Pre: not top-5, no tax. Post: over $1M, taxed.
        assert_eq!(pre.total_excise_tax_cents, 0);
        assert_eq!(post.total_excise_tax_cents, 105_000_00);
    }

    #[test]
    fn saturating_arithmetic_overflow_defense() {
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![emp(
                "Massive",
                u64::MAX,
                u64::MAX / 2,
                u64::MAX,
                true,
                false,
                0,
            )],
        };
        let out = check(&input);
        // No panic; saturating arithmetic bounds all math.
        assert_eq!(out.severity, Severity::ExciseTaxOwed);
    }

    #[test]
    fn empty_employees_compliant() {
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![],
        };
        let out = check(&input);
        assert_eq!(out.severity, Severity::Compliant);
        assert_eq!(out.total_excise_tax_cents, 0);
        assert!(out.per_employee.is_empty());
    }

    #[test]
    fn ateo_truth_table_four_cells() {
        // Cell 1: ATEO + over $1M + top-5 = TAX
        let c1 = check(&Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![emp("A", 1_500_000_00, 500_000_00, 0, true, false, 0)],
        });
        assert_eq!(c1.severity, Severity::ExciseTaxOwed);

        // Cell 2: ATEO + over $1M + NOT top-5 (pre-OBBBA) = no tax
        let c2 = check(&Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![emp("B", 1_500_000_00, 500_000_00, 0, false, false, 0)],
        });
        assert_eq!(c2.severity, Severity::Compliant);

        // Cell 3: Non-ATEO regardless of comp = no tax
        let c3 = check(&Input {
            organization_type: OrganizationType::NonExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![emp("C", 10_000_000_00, 1_000_000_00, 0, true, false, 0)],
        });
        assert_eq!(c3.severity, Severity::OrgNotAteo);

        // Cell 4: ATEO + under $1M + top-5 = no tax
        let c4 = check(&Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![emp("D", 900_000_00, 500_000_00, 0, true, false, 0)],
        });
        assert_eq!(c4.severity, Severity::Compliant);
    }

    #[test]
    fn note_severity_excise_tax_owed_dollar_format() {
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![emp("CEO", 1_500_000_00, 500_000_00, 0, true, false, 0)],
        };
        let out = check(&input);
        let joined = out.notes.join(" ");
        assert!(joined.contains("$105000.00") || joined.contains("$105,000.00"));
    }

    #[test]
    fn licensed_medical_professional_with_zero_medical_remuneration_no_exclusion() {
        // Flagged as medical pro but $0 of pay attributed to medical services
        // → full $2M is taxable remuneration
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![emp("Dr-Admin", 2_000_000_00, 1_000_000_00, 0, true, true, 0)],
        };
        let out = check(&input);
        assert_eq!(out.per_employee[0].remuneration_excess_cents, 1_000_000_00);
        assert_eq!(out.total_excise_tax_cents, 210_000_00);
    }

    #[test]
    fn parachute_exactly_at_trigger_boundary() {
        // Base $500K, parachute exactly $1.5M = 3× base → triggers
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![emp(
                "CEO",
                500_000_00,
                500_000_00,
                1_500_000_00,
                true,
                false,
                0,
            )],
        };
        let out = check(&input);
        assert!(out.per_employee[0].parachute_triggered);
        assert_eq!(out.per_employee[0].excess_parachute_cents, 1_000_000_00);
    }

    #[test]
    fn parachute_one_cent_below_trigger() {
        // Base $500K, parachute $1.5M − 1¢ = under trigger → no parachute tax
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![emp(
                "CEO",
                500_000_00,
                500_000_00,
                1_499_999_99,
                true,
                false,
                0,
            )],
        };
        let out = check(&input);
        assert!(!out.per_employee[0].parachute_triggered);
        assert_eq!(out.per_employee[0].excess_parachute_cents, 0);
    }

    #[test]
    fn multiple_employees_independent_calculations() {
        let input = Input {
            organization_type: OrganizationType::Section501aExempt,
            tax_year_regime: TaxYearRegime::PreObbba,
            employees: vec![
                emp("CEO", 2_000_000_00, 1_000_000_00, 0, true, false, 0),
                emp("CFO", 1_200_000_00, 700_000_00, 0, true, false, 0),
                emp("COO", 800_000_00, 500_000_00, 0, true, false, 0),
            ],
        };
        let out = check(&input);
        // CEO: $1M excess × 21% = $210K
        // CFO: $200K excess × 21% = $42K
        // COO: $0 excess
        assert_eq!(out.per_employee[0].excise_tax_cents, 210_000_00);
        assert_eq!(out.per_employee[1].excise_tax_cents, 42_000_00);
        assert_eq!(out.per_employee[2].excise_tax_cents, 0);
        assert_eq!(out.total_excise_tax_cents, 252_000_00);
    }
}
