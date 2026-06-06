//! IRC § 4974 — Excise tax on certain accumulations in
//! qualified retirement plans. Companion to section_408
//! (traditional IRA — iter 432), section_408a (Roth
//! IRA — iter 430), section_4975 (prohibited
//! transactions — iter 434), section_72t (10% early-
//! withdrawal penalty), section_401a9 (RMD timing
//! rules).
//!
//! § 4974(a) imposes an EXCISE TAX on the payee equal
//! to a percentage of the amount by which the AMOUNT
//! DISTRIBUTED during the taxable year FALLS SHORT OF
//! the MINIMUM REQUIRED DISTRIBUTION (RMD) for that
//! year under § 401(a)(9) or § 408(a)(6) / § 408(b)(3).
//!
//! Trader-critical for traditional IRA holders + 401(k)
//! plan participants approaching RMD age:
//! - SECURE Act 2.0 of 2022 § 107 — RMD age increased
//!   from 72 to 73 effective January 1, 2023; further
//!   increases to age 75 effective January 1, 2033;
//! - SECURE Act 2.0 of 2022 § 302 — REDUCED excise tax
//!   from 50% to 25% of shortfall;
//! - SECURE Act 2.0 of 2022 § 302 — FURTHER REDUCED to
//!   10% if RMD corrected within § 4974(e) correction
//!   window (broadly 2 years from original RMD
//!   deadline).
//!
//! **§ 4974(a) rate structure (post-SECURE 2.0)**:
//! - 25% standard excise tax on shortfall (was 50%
//!   prior to SECURE 2.0);
//! - 10% reduced rate if corrected within § 4974(e)
//!   correction window.
//!
//! **§ 4974(e) correction window** — begins on the
//! ORIGINAL RMD DUE DATE and ends on the EARLIEST of:
//! 1. Last day of the SECOND TAX YEAR following the
//!    year the RMD was missed;
//! 2. Date IRS sends NOTICE OF DEFICIENCY for the
//!    excise tax;
//! 3. Date IRS ASSESSES the excise tax.
//!
//! **§ 401(a)(9) Required Beginning Date**:
//! - PRE-SECURE 2.0 (born before 1951): April 1 of
//!   year following age 70½ (original rule) OR April 1
//!   of year following age 72 (SECURE Act 1.0);
//! - POST-SECURE 2.0 (born 1951-1959): April 1 of
//!   year following age 73;
//! - POST-SECURE 2.0 (born 1960 or later): April 1 of
//!   year following age 75 (effective January 1, 2033).
//!
//! **§ 4974(c) qualified plans subject to excise**:
//! 1. § 401(a) qualified plans (401(k), pension,
//!    profit-sharing);
//! 2. § 403(a) annuity plans;
//! 3. § 403(b) tax-sheltered annuities;
//! 4. § 408(a) individual retirement accounts (IRAs);
//! 5. § 408(b) individual retirement annuities;
//! 6. § 408A Roth IRAs — NO LIFETIME RMD for original
//!    owner under § 408A(c)(5) BUT § 401(a)(9)(B)
//!    POST-DEATH RMD rules apply to beneficiaries;
//! 7. § 457(b) governmental deferred compensation
//!    plans (not nongovernmental);
//! 8. § 408(p) SIMPLE IRAs;
//! 9. § 408(k) SEP-IRAs.
//!
//! **§ 401(a)(9)(B) post-death RMD rules** (10-year
//! rule under SECURE Act 1.0 + 5 categories of
//! eligible designated beneficiaries):
//! - SURVIVING SPOUSE: stretch over spouse's life
//!   expectancy OR 10-year rule election;
//! - MINOR CHILD of decedent: stretch until majority
//!   THEN 10-year rule;
//! - DISABLED beneficiary: stretch over life
//!   expectancy;
//! - CHRONICALLY ILL beneficiary: stretch over life
//!   expectancy;
//! - BENEFICIARY < 10 years younger than decedent:
//!   stretch over life expectancy;
//! - ALL OTHER beneficiaries: 10-year rule (full
//!   distribution by December 31 of 10th year after
//!   death).
//!
//! **§ 4974(d) waiver of tax** — IRS MAY WAIVE excise
//! tax if (1) shortfall due to REASONABLE ERROR and
//! (2) reasonable steps being taken to remedy. Form
//! 5329 with statement explaining reasonable cause +
//! correction.
//!
//! **§ 408(d)(8) qualified charitable distribution
//! (QCD)** — for taxpayers 70½ or older, up to $108,000
//! (2026 SECURE 2.0 inflation-adjusted limit) of IRA
//! distribution directly to qualified charity COUNTS
//! TOWARD RMD without inclusion in gross income; can
//! ZERO OUT RMD obligation if QCD ≥ RMD.
//!
//! **Trader-critical fact patterns**:
//! 1. 75-year-old trader with $2M IRA fails to take
//!    $80,000 RMD — owes $20,000 excise tax at 25%
//!    standard rate (was $40,000 at 50% pre-SECURE
//!    2.0);
//! 2. Trader corrects $80,000 shortfall by filing
//!    Form 5329 within 2 years — rate drops to 10%
//!    ($8,000 excise);
//! 3. Trader directs $80,000 QCD to charity — satisfies
//!    RMD without inclusion in gross income; § 4974
//!    excise tax = $0;
//! 4. Trader inherits IRA in 2024 — beneficiary subject
//!    to SECURE 1.0 10-year rule (full distribution by
//!    December 31, 2034); failure to deplete = § 4974
//!    excise tax on undistributed amount;
//! 5. Roth IRA owner age 80 — NO RMD during lifetime
//!    under § 408A(c)(5); § 4974 inapplicable.
//!
//! Citations: 26 USC § 4974(a)-(e); 26 USC § 401(a)(9);
//! 26 USC § 408(a)(6); 26 USC § 408(b)(3); 26 USC
//! § 408A(c)(5); 26 USC § 408(d)(8); SECURE Act of
//! 2019 § 114; SECURE Act 2.0 of 2022 § 107 (RMD age);
//! SECURE Act 2.0 of 2022 § 302 (excise rate
//! reduction); Pub. L. 117-328 (Consolidated
//! Appropriations Act, 2023 — SECURE 2.0); Form 5329;
//! Treas. Reg. § 54.4974-2; Treas. Reg. § 1.401(a)(9)-1
//! through § 1.401(a)(9)-9.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlanType {
    /// § 401(a) qualified plan (401(k), pension, profit-
    /// sharing).
    QualifiedPlan401a,
    /// § 403(a) annuity plan.
    Annuity403a,
    /// § 403(b) tax-sheltered annuity.
    TaxShelteredAnnuity403b,
    /// § 408(a) traditional individual retirement
    /// account.
    TraditionalIra408a,
    /// § 408(b) individual retirement annuity.
    IndividualRetirementAnnuity408b,
    /// § 408A Roth IRA (no lifetime RMD for original
    /// owner under § 408A(c)(5)).
    RothIra408a,
    /// § 457(b) governmental deferred compensation.
    GovernmentalDeferredComp457b,
    /// § 408(p) SIMPLE IRA.
    SimpleIra408p,
    /// § 408(k) SEP-IRA.
    SepIra408k,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OwnerStatus {
    /// Original owner / participant.
    OriginalOwner,
    /// Surviving spouse beneficiary.
    SurvivingSpouseBeneficiary,
    /// Minor child of decedent.
    MinorChildBeneficiary,
    /// Disabled or chronically ill beneficiary.
    DisabledOrChronicallyIllBeneficiary,
    /// Beneficiary less than 10 years younger than
    /// decedent.
    LessThanTenYearsYoungerBeneficiary,
    /// All other beneficiaries (10-year rule applies).
    OtherDesignatedBeneficiary,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section4974Input {
    pub plan_type: PlanType,
    pub owner_status: OwnerStatus,
    /// Owner's age at end of distribution year.
    pub owner_age_at_year_end: u32,
    /// Required minimum distribution amount in cents.
    pub rmd_required_cents: u64,
    /// Amount actually distributed during year in cents.
    pub amount_distributed_cents: u64,
    /// Amount distributed as qualified charitable
    /// distribution (QCD) under § 408(d)(8).
    pub qcd_distributed_cents: u64,
    /// Whether shortfall corrected within § 4974(e)
    /// correction window (2 years).
    pub corrected_within_window: bool,
    /// Whether IRS granted waiver under § 4974(d) for
    /// reasonable error.
    pub waiver_granted_for_reasonable_error: bool,
    /// Year of birth (for RMD age determination).
    pub year_of_birth: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section4974Result {
    pub rmd_required: bool,
    pub rmd_age: u32,
    pub shortfall_cents: u64,
    pub effective_excise_rate_percent: u32,
    pub excise_tax_cents: u64,
    pub qcd_satisfies_rmd: bool,
    pub waiver_granted: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section4974Input) -> Section4974Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let rmd_age: u32 = if input.year_of_birth < 1951 {
        72
    } else if input.year_of_birth <= 1959 {
        73
    } else {
        75
    };

    let original_owner_subject_to_rmd = !matches!(input.plan_type, PlanType::RothIra408a)
        && matches!(input.owner_status, OwnerStatus::OriginalOwner)
        && input.owner_age_at_year_end >= rmd_age;

    let beneficiary_subject_to_rmd = !matches!(input.owner_status, OwnerStatus::OriginalOwner);

    let rmd_required = original_owner_subject_to_rmd || beneficiary_subject_to_rmd;

    let total_satisfying_distribution = input
        .amount_distributed_cents
        .saturating_add(input.qcd_distributed_cents);

    let qcd_satisfies_rmd = input.qcd_distributed_cents >= input.rmd_required_cents && rmd_required;

    let shortfall_cents = if rmd_required {
        input
            .rmd_required_cents
            .saturating_sub(total_satisfying_distribution)
    } else {
        0
    };

    let effective_excise_rate_percent: u32 =
        if !rmd_required || shortfall_cents == 0 || input.waiver_granted_for_reasonable_error {
            0
        } else if input.corrected_within_window {
            10
        } else {
            25
        };

    let excise_tax_cents = if effective_excise_rate_percent == 0 {
        0
    } else {
        shortfall_cents.saturating_mul(effective_excise_rate_percent as u64) / 100
    };

    let waiver_granted =
        rmd_required && shortfall_cents > 0 && input.waiver_granted_for_reasonable_error;

    if matches!(input.plan_type, PlanType::RothIra408a)
        && matches!(input.owner_status, OwnerStatus::OriginalOwner)
    {
        failure_reasons.push(
            "26 USC § 408A(c)(5) — ROTH IRA original owner NOT SUBJECT to lifetime RMDs; § 4974 inapplicable during owner's lifetime; post-death RMDs under § 401(a)(9)(B) apply to beneficiaries".to_string(),
        );
    }

    if rmd_required && shortfall_cents > 0 && !input.waiver_granted_for_reasonable_error {
        failure_reasons.push(format!(
            "26 USC § 4974(a) — RMD SHORTFALL of {} cents triggers {}% excise tax of {} cents on payee; {}",
            shortfall_cents,
            effective_excise_rate_percent,
            excise_tax_cents,
            if input.corrected_within_window {
                "REDUCED to 10% via § 4974(e) correction window (2 years from original RMD due date)"
            } else {
                "standard 25% rate post-SECURE 2.0 § 302 (was 50% pre-SECURE 2.0)"
            }
        ));
    }

    if rmd_required && qcd_satisfies_rmd {
        failure_reasons.push(format!(
            "26 USC § 408(d)(8) — Qualified Charitable Distribution (QCD) of {} cents directly to charity SATISFIES RMD; counted toward RMD WITHOUT inclusion in gross income; § 4974 excise tax = $0",
            input.qcd_distributed_cents
        ));
    }

    if waiver_granted {
        failure_reasons.push(
            "26 USC § 4974(d) — IRS WAIVED excise tax for REASONABLE ERROR with reasonable steps being taken to remedy; Form 5329 with statement explaining reasonable cause + correction".to_string(),
        );
    }

    if rmd_required && !matches!(input.owner_status, OwnerStatus::OriginalOwner) {
        let beneficiary_rule = match input.owner_status {
            OwnerStatus::SurvivingSpouseBeneficiary => {
                "26 USC § 401(a)(9)(B)(iv) — SURVIVING SPOUSE may stretch over spouse's life expectancy OR elect 10-year rule"
            }
            OwnerStatus::MinorChildBeneficiary => {
                "26 USC § 401(a)(9)(E)(ii)(II) — MINOR CHILD stretches until majority, then 10-year rule"
            }
            OwnerStatus::DisabledOrChronicallyIllBeneficiary => {
                "26 USC § 401(a)(9)(E)(ii)(III)-(IV) — DISABLED or CHRONICALLY ILL beneficiary stretches over life expectancy"
            }
            OwnerStatus::LessThanTenYearsYoungerBeneficiary => {
                "26 USC § 401(a)(9)(E)(ii)(V) — beneficiary LESS THAN 10 YEARS YOUNGER than decedent stretches over life expectancy"
            }
            OwnerStatus::OtherDesignatedBeneficiary => {
                "26 USC § 401(a)(9)(H) — SECURE Act 1.0 10-YEAR RULE: full distribution by December 31 of 10th year after death"
            }
            _ => "",
        };
        if !beneficiary_rule.is_empty() {
            failure_reasons.push(beneficiary_rule.to_string());
        }
    }

    let notes: Vec<String> = vec![
        "26 USC § 4974(a) — EXCISE TAX on payee equal to percentage of shortfall between RMD required and actual distribution; SECURE Act 2.0 § 302 REDUCED rate from 50% to 25% (standard) + 10% (within correction window)".to_string(),
        "26 USC § 4974(e) CORRECTION WINDOW — begins on ORIGINAL RMD DUE DATE; ends on EARLIEST of (1) last day of SECOND TAX YEAR following missed-RMD year; (2) IRS notice of deficiency; (3) IRS assessment of excise tax".to_string(),
        "26 USC § 401(a)(9) Required Beginning Date — PRE-SECURE 2.0 (born before 1951) = April 1 following age 72 (SECURE 1.0) or 70½ (original); POST-SECURE 2.0 (born 1951-1959) = April 1 following age 73; POST-2033 (born 1960+) = April 1 following age 75".to_string(),
        "26 USC § 4974(c) — QUALIFIED PLANS subject to excise: (1) § 401(a) qualified plans (401(k), pension, profit-sharing); (2) § 403(a) annuity plans; (3) § 403(b) tax-sheltered annuities; (4) § 408(a) IRAs; (5) § 408(b) individual retirement annuities; (6) § 408A ROTH IRAs (NO lifetime RMD for original owner but post-death beneficiary rules apply); (7) § 457(b) GOVERNMENTAL deferred compensation; (8) § 408(p) SIMPLE IRAs; (9) § 408(k) SEP-IRAs".to_string(),
        "26 USC § 401(a)(9)(B) POST-DEATH RMD rules — (1) SURVIVING SPOUSE stretches life expectancy or 10-year election; (2) MINOR CHILD stretches until majority then 10-year; (3) DISABLED stretches life expectancy; (4) CHRONICALLY ILL stretches life expectancy; (5) < 10 YEARS YOUNGER stretches life expectancy; (6) ALL OTHER DESIGNATED BENEFICIARIES use SECURE Act 1.0 10-YEAR RULE (full distribution by Dec 31 of 10th year)".to_string(),
        "26 USC § 4974(d) WAIVER — IRS may waive excise tax if (1) shortfall due to REASONABLE ERROR and (2) reasonable steps being taken to remedy; Form 5329 with statement explaining reasonable cause + correction".to_string(),
        "26 USC § 408(d)(8) Qualified Charitable Distribution (QCD) — for taxpayers age 70½ or older, up to $108,000 (2026 SECURE 2.0 inflation-adjusted) of IRA distribution DIRECTLY to qualified charity COUNTS TOWARD RMD WITHOUT inclusion in gross income; can ZERO OUT RMD obligation if QCD ≥ RMD; SECURE Act 2.0 § 307 added inflation indexing".to_string(),
        "26 USC § 408A(c)(5) — ROTH IRA original owner NOT subject to lifetime RMD; § 4974 inapplicable during owner's lifetime; post-death beneficiary RMD rules under § 401(a)(9)(B) DO apply".to_string(),
        "SECURE Act of 2019 § 114 — original RMD age raised from 70½ to 72; SECURE Act 1.0 also implemented 10-YEAR RULE for non-eligible designated beneficiaries (replacing pre-2020 stretch-IRA)".to_string(),
        "SECURE Act 2.0 of 2022 § 107 (Pub. L. 117-328) — RMD age raised to 73 (born 1951-1959) effective January 1, 2023; further raised to 75 (born 1960+) effective January 1, 2033".to_string(),
        "SECURE Act 2.0 of 2022 § 302 — excise tax REDUCED from 50% to 25% standard; FURTHER REDUCED to 10% if corrected within § 4974(e) correction window".to_string(),
        "Form 5329 (Additional Taxes on Qualified Plans (Including IRAs) and Other Tax-Favored Accounts) — required filing for RMD shortfall + § 4974 excise tax + § 4974(d) waiver request".to_string(),
        "Trader-critical fact patterns: (1) 75-year-old trader with $2M IRA fails $80,000 RMD — $20,000 excise at 25%; (2) corrects within 2 years — rate drops to 10% ($8,000); (3) $80,000 QCD to charity satisfies RMD = $0 excise; (4) inherited IRA in 2024 beneficiary subject to 10-year rule (depletion by Dec 31, 2034); (5) Roth IRA owner age 80 = NO RMD during lifetime under § 408A(c)(5)".to_string(),
        "Companion to section_408 (traditional IRA) + section_408a (Roth IRA) + section_4975 (prohibited transactions) + section_72t (10% early-withdrawal penalty) + section_401a9 (RMD timing rules)".to_string(),
    ];

    Section4974Result {
        rmd_required,
        rmd_age,
        shortfall_cents,
        effective_excise_rate_percent,
        excise_tax_cents,
        qcd_satisfies_rmd,
        waiver_granted,
        failure_reasons,
        citation: "26 USC § 4974(a)-(e); 26 USC § 401(a)(9); 26 USC § 408(a)(6); 26 USC § 408(b)(3); 26 USC § 408A(c)(5); 26 USC § 408(d)(8); SECURE Act of 2019 § 114; SECURE Act 2.0 of 2022 § 107; SECURE Act 2.0 of 2022 § 302; Pub. L. 117-328 (Consolidated Appropriations Act, 2023 — SECURE 2.0); Form 5329; Treas. Reg. § 54.4974-2; Treas. Reg. § 1.401(a)(9)-1 through § 1.401(a)(9)-9",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_owner_75() -> Section4974Input {
        Section4974Input {
            plan_type: PlanType::TraditionalIra408a,
            owner_status: OwnerStatus::OriginalOwner,
            owner_age_at_year_end: 75,
            rmd_required_cents: 8_000_000,
            amount_distributed_cents: 0,
            qcd_distributed_cents: 0,
            corrected_within_window: false,
            waiver_granted_for_reasonable_error: false,
            year_of_birth: 1951,
        }
    }

    #[test]
    fn standard_25_percent_excise_75_year_old() {
        let r = check(&baseline_owner_75());
        assert!(r.rmd_required);
        assert_eq!(r.shortfall_cents, 8_000_000);
        assert_eq!(r.effective_excise_rate_percent, 25);
        assert_eq!(r.excise_tax_cents, 2_000_000);
    }

    #[test]
    fn ten_percent_reduced_rate_correction_window() {
        let mut i = baseline_owner_75();
        i.corrected_within_window = true;
        let r = check(&i);
        assert_eq!(r.effective_excise_rate_percent, 10);
        assert_eq!(r.excise_tax_cents, 800_000);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 4974(e) correction window")));
    }

    #[test]
    fn partial_distribution_partial_shortfall() {
        let mut i = baseline_owner_75();
        i.amount_distributed_cents = 3_000_000;
        let r = check(&i);
        assert_eq!(r.shortfall_cents, 5_000_000);
        assert_eq!(r.excise_tax_cents, 1_250_000);
    }

    #[test]
    fn full_distribution_no_shortfall() {
        let mut i = baseline_owner_75();
        i.amount_distributed_cents = 8_000_000;
        let r = check(&i);
        assert_eq!(r.shortfall_cents, 0);
        assert_eq!(r.effective_excise_rate_percent, 0);
        assert_eq!(r.excise_tax_cents, 0);
    }

    #[test]
    fn qcd_satisfies_rmd_zeros_excise() {
        let mut i = baseline_owner_75();
        i.qcd_distributed_cents = 8_000_000;
        let r = check(&i);
        assert!(r.qcd_satisfies_rmd);
        assert_eq!(r.shortfall_cents, 0);
        assert_eq!(r.excise_tax_cents, 0);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 408(d)(8)") && f.contains("SATISFIES RMD")));
    }

    #[test]
    fn partial_qcd_partial_distribution_zero_shortfall() {
        let mut i = baseline_owner_75();
        i.qcd_distributed_cents = 3_000_000;
        i.amount_distributed_cents = 5_000_000;
        let r = check(&i);
        assert_eq!(r.shortfall_cents, 0);
    }

    #[test]
    fn waiver_zeros_excise_with_reasonable_error() {
        let mut i = baseline_owner_75();
        i.waiver_granted_for_reasonable_error = true;
        let r = check(&i);
        assert!(r.waiver_granted);
        assert_eq!(r.effective_excise_rate_percent, 0);
        assert_eq!(r.excise_tax_cents, 0);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 4974(d)") && f.contains("REASONABLE ERROR")));
    }

    #[test]
    fn roth_ira_no_lifetime_rmd_original_owner() {
        let mut i = baseline_owner_75();
        i.plan_type = PlanType::RothIra408a;
        let r = check(&i);
        assert!(!r.rmd_required);
        assert_eq!(r.excise_tax_cents, 0);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 408A(c)(5)") && f.contains("NOT SUBJECT to lifetime RMDs")));
    }

    #[test]
    fn rmd_age_72_pre_1951_born() {
        let mut i = baseline_owner_75();
        i.year_of_birth = 1950;
        let r = check(&i);
        assert_eq!(r.rmd_age, 72);
    }

    #[test]
    fn rmd_age_73_born_1951_1959() {
        let mut i = baseline_owner_75();
        i.year_of_birth = 1955;
        let r = check(&i);
        assert_eq!(r.rmd_age, 73);
    }

    #[test]
    fn rmd_age_75_born_1960_or_later() {
        let mut i = baseline_owner_75();
        i.year_of_birth = 1965;
        let r = check(&i);
        assert_eq!(r.rmd_age, 75);
    }

    #[test]
    fn under_rmd_age_no_rmd_required() {
        let mut i = baseline_owner_75();
        i.owner_age_at_year_end = 70;
        i.year_of_birth = 1955;
        let r = check(&i);
        assert!(!r.rmd_required);
        assert_eq!(r.excise_tax_cents, 0);
    }

    #[test]
    fn surviving_spouse_beneficiary_stretch_or_ten_year() {
        let mut i = baseline_owner_75();
        i.owner_status = OwnerStatus::SurvivingSpouseBeneficiary;
        let r = check(&i);
        assert!(r.rmd_required);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 401(a)(9)(B)(iv)") && f.contains("SURVIVING SPOUSE")));
    }

    #[test]
    fn minor_child_beneficiary_stretch_until_majority() {
        let mut i = baseline_owner_75();
        i.owner_status = OwnerStatus::MinorChildBeneficiary;
        let r = check(&i);
        assert!(r.rmd_required);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 401(a)(9)(E)(ii)(II)") && f.contains("MINOR CHILD")));
    }

    #[test]
    fn disabled_chronically_ill_beneficiary_stretch_life_expectancy() {
        let mut i = baseline_owner_75();
        i.owner_status = OwnerStatus::DisabledOrChronicallyIllBeneficiary;
        let r = check(&i);
        assert!(r.rmd_required);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 401(a)(9)(E)(ii)(III)-(IV)") && f.contains("DISABLED")));
    }

    #[test]
    fn less_than_ten_years_younger_beneficiary_stretch() {
        let mut i = baseline_owner_75();
        i.owner_status = OwnerStatus::LessThanTenYearsYoungerBeneficiary;
        let r = check(&i);
        assert!(r.rmd_required);
        assert!(r.failure_reasons.iter().any(
            |f| f.contains("§ 401(a)(9)(E)(ii)(V)") && f.contains("LESS THAN 10 YEARS YOUNGER")
        ));
    }

    #[test]
    fn other_designated_beneficiary_ten_year_rule() {
        let mut i = baseline_owner_75();
        i.owner_status = OwnerStatus::OtherDesignatedBeneficiary;
        let r = check(&i);
        assert!(r.rmd_required);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 401(a)(9)(H)") && f.contains("10-YEAR RULE")));
    }

    #[test]
    fn plan_type_truth_table_nine_cells() {
        for (plan, expect_rmd) in [
            (PlanType::QualifiedPlan401a, true),
            (PlanType::Annuity403a, true),
            (PlanType::TaxShelteredAnnuity403b, true),
            (PlanType::TraditionalIra408a, true),
            (PlanType::IndividualRetirementAnnuity408b, true),
            (PlanType::RothIra408a, false),
            (PlanType::GovernmentalDeferredComp457b, true),
            (PlanType::SimpleIra408p, true),
            (PlanType::SepIra408k, true),
        ] {
            let mut i = baseline_owner_75();
            i.plan_type = plan;
            let r = check(&i);
            assert_eq!(r.rmd_required, expect_rmd, "plan={:?}", plan);
        }
    }

    #[test]
    fn excise_rate_truth_table_three_cells() {
        let mut std_rate = baseline_owner_75();
        std_rate.corrected_within_window = false;
        std_rate.waiver_granted_for_reasonable_error = false;
        let r_std = check(&std_rate);
        assert_eq!(r_std.effective_excise_rate_percent, 25);

        let mut reduced_rate = baseline_owner_75();
        reduced_rate.corrected_within_window = true;
        reduced_rate.waiver_granted_for_reasonable_error = false;
        let r_reduced = check(&reduced_rate);
        assert_eq!(r_reduced.effective_excise_rate_percent, 10);

        let mut waiver = baseline_owner_75();
        waiver.corrected_within_window = false;
        waiver.waiver_granted_for_reasonable_error = true;
        let r_waiver = check(&waiver);
        assert_eq!(r_waiver.effective_excise_rate_percent, 0);
    }

    #[test]
    fn roth_uniquely_exempt_lifetime_rmd_invariant() {
        let mut roth = baseline_owner_75();
        roth.plan_type = PlanType::RothIra408a;
        let r_roth = check(&roth);
        assert!(!r_roth.rmd_required);

        let mut trad = baseline_owner_75();
        trad.plan_type = PlanType::TraditionalIra408a;
        let r_trad = check(&trad);
        assert!(r_trad.rmd_required);
    }

    #[test]
    fn correction_uniquely_drops_rate_invariant() {
        let mut uncorrected = baseline_owner_75();
        uncorrected.corrected_within_window = false;
        let r_uncorr = check(&uncorrected);
        assert_eq!(r_uncorr.effective_excise_rate_percent, 25);

        let mut corrected = baseline_owner_75();
        corrected.corrected_within_window = true;
        let r_corr = check(&corrected);
        assert_eq!(r_corr.effective_excise_rate_percent, 10);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&baseline_owner_75());
        assert!(r.citation.contains("§ 4974(a)-(e)"));
        assert!(r.citation.contains("§ 401(a)(9)"));
        assert!(r.citation.contains("§ 408(a)(6)"));
        assert!(r.citation.contains("§ 408(b)(3)"));
        assert!(r.citation.contains("§ 408A(c)(5)"));
        assert!(r.citation.contains("§ 408(d)(8)"));
        assert!(r.citation.contains("SECURE Act of 2019 § 114"));
        assert!(r.citation.contains("SECURE Act 2.0 of 2022 § 107"));
        assert!(r.citation.contains("SECURE Act 2.0 of 2022 § 302"));
        assert!(r.citation.contains("Pub. L. 117-328"));
        assert!(r.citation.contains("Form 5329"));
        assert!(r.citation.contains("Treas. Reg. § 54.4974-2"));
        assert!(r
            .citation
            .contains("Treas. Reg. § 1.401(a)(9)-1 through § 1.401(a)(9)-9"));
    }

    #[test]
    fn note_pins_subsection_a_excise_tax_rate_history() {
        let r = check(&baseline_owner_75());
        assert!(r.notes.iter().any(|n| n.contains("§ 4974(a)")
            && n.contains("EXCISE TAX")
            && n.contains("SECURE Act 2.0 § 302 REDUCED rate from 50% to 25%")));
    }

    #[test]
    fn note_pins_subsection_e_correction_window() {
        let r = check(&baseline_owner_75());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 4974(e) CORRECTION WINDOW")
                && n.contains("ORIGINAL RMD DUE DATE")
                && n.contains("SECOND TAX YEAR")));
    }

    #[test]
    fn note_pins_section_401a9_required_beginning_date_three_eras() {
        let r = check(&baseline_owner_75());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 401(a)(9) Required Beginning Date")
                && n.contains("(born before 1951)")
                && n.contains("(born 1951-1959)")
                && n.contains("(born 1960+)")));
    }

    #[test]
    fn note_pins_subsection_c_qualified_plans_nine() {
        let r = check(&baseline_owner_75());
        assert!(r.notes.iter().any(|n| n.contains("§ 4974(c)")
            && n.contains("QUALIFIED PLANS")
            && n.contains("§ 401(a)")
            && n.contains("§ 403(b)")
            && n.contains("§ 408A ROTH IRAs")
            && n.contains("§ 457(b) GOVERNMENTAL")
            && n.contains("SEP-IRAs")));
    }

    #[test]
    fn note_pins_section_401a9b_post_death_six_categories() {
        let r = check(&baseline_owner_75());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 401(a)(9)(B) POST-DEATH")
                && n.contains("SURVIVING SPOUSE")
                && n.contains("MINOR CHILD")
                && n.contains("DISABLED")
                && n.contains("CHRONICALLY ILL")
                && n.contains("< 10 YEARS YOUNGER")
                && n.contains("10-YEAR RULE")));
    }

    #[test]
    fn note_pins_subsection_d_waiver_reasonable_error() {
        let r = check(&baseline_owner_75());
        assert!(r.notes.iter().any(|n| n.contains("§ 4974(d) WAIVER")
            && n.contains("REASONABLE ERROR")
            && n.contains("Form 5329")));
    }

    #[test]
    fn note_pins_qcd_408d8() {
        let r = check(&baseline_owner_75());
        assert!(r.notes.iter().any(|n| n
            .contains("§ 408(d)(8) Qualified Charitable Distribution")
            && n.contains("$108,000")
            && n.contains("ZERO OUT RMD obligation")
            && n.contains("SECURE Act 2.0 § 307")));
    }

    #[test]
    fn note_pins_secure_2_act_107_age_increase() {
        let r = check(&baseline_owner_75());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("SECURE Act 2.0 of 2022 § 107")
                && n.contains("Pub. L. 117-328")
                && n.contains("January 1, 2023")
                && n.contains("January 1, 2033")));
    }

    #[test]
    fn note_pins_secure_2_act_302_rate_reduction() {
        let r = check(&baseline_owner_75());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("SECURE Act 2.0 of 2022 § 302")
                && n.contains("50% to 25%")
                && n.contains("10% if corrected")));
    }

    #[test]
    fn note_pins_form_5329() {
        let r = check(&baseline_owner_75());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Form 5329") && n.contains("Additional Taxes on Qualified Plans")));
    }

    #[test]
    fn note_pins_trader_fact_patterns() {
        let r = check(&baseline_owner_75());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Trader-critical fact patterns")
                && n.contains("75-year-old trader with $2M IRA")
                && n.contains("Roth IRA owner age 80")));
    }

    #[test]
    fn note_pins_companion_modules() {
        let r = check(&baseline_owner_75());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Companion to section_408")
                && n.contains("section_408a")
                && n.contains("section_4975")
                && n.contains("section_72t")
                && n.contains("section_401a9")));
    }

    #[test]
    fn defensive_overflow_saturating() {
        let mut i = baseline_owner_75();
        i.rmd_required_cents = u64::MAX;
        let r = check(&i);
        let _ = r.excise_tax_cents;
    }
}
