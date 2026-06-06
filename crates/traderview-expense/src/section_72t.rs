//! IRC §72(t) — 10% additional tax on early distributions from
//! qualified retirement plans.
//!
//! Trader-relevant because traders frequently consider accessing
//! retirement-account capital for active strategies. §72(t)(1)
//! imposes an additional 10% tax on the includible portion of any
//! distribution received before age 59½ from a qualified plan,
//! IRA, or other tax-favored retirement vehicle. §72(t)(2)
//! enumerates ~14 exceptions; the SECURE 2.0 Act (Dec 2022) added
//! 4 new exceptions covering modern hardship scenarios.
//!
//! **§72(t)(1) general rule**: additional 10% tax on early
//! distributions from qualified retirement plans (IRC §401, §403,
//! §408, §457, etc.) when received before the taxpayer attains
//! age 59½. The additional tax applies only to the portion of
//! the distribution includible in gross income (basis recovery
//! is not subject to the tax).
//!
//! **§72(t)(2) exception list**:
//!
//! - **(A)(i)** age 59½ or older
//! - **(A)(ii)** death of the participant
//! - **(A)(iii)** disability (§72(m)(7) standard)
//! - **(A)(iv)** substantially equal periodic payments (SEPP) for
//!   life or life expectancy (5-year rule applies)
//! - **(A)(v)** separation from service after age 55
//! - **(B)** medical expenses exceeding the §213 7.5% AGI floor
//! - **(C)** Qualified Domestic Relations Order (QDRO) under
//!   §414(p)
//! - **(D)** IRA distributions for qualified higher education
//!   expenses (§529 / §530)
//! - **(F)** first-time homebuyer (IRA only; $10,000 lifetime cap)
//! - **(G)** IRA distributions to unemployed individuals for
//!   health insurance
//! - **(H)** federal qualified disaster distributions
//!   (§72(t)(11), up to $22,000 per disaster)
//! - **(I)** qualified birth or adoption distribution
//!   (§72(t)(2)(H), up to $5,000)
//! - **(J)** terminal illness (SECURE 2.0 §326, physician-
//!   certified, < 7-year life expectancy)
//! - **(K)** emergency personal expense (SECURE 2.0 §115,
//!   up to $1,000 per calendar year, plan-optional)
//! - **(L)** domestic abuse victim (SECURE 2.0 §314, up to
//!   $10,000 inflation-indexed, plan-optional)
//! - **(M)** qualified long-term care insurance distribution
//!   (SECURE 2.0 §334, up to $2,500 annually, eff. 2026)
//!
//! Exceptions are NOT cumulative — only one applies per
//! distribution path. Some have plan-level adoption requirements
//! (SECURE 2.0 emergency / domestic abuse exceptions are
//! plan-optional even after statute is effective).
//!
//! Sources:
//! [Cornell LII 26 U.S.C. § 72](https://www.law.cornell.edu/uscode/text/26/72),
//! [IRS — Retirement topics: Exceptions to tax on early distributions](https://www.irs.gov/retirement-plans/plan-participant-employee/retirement-topics-exceptions-to-tax-on-early-distributions),
//! [IRS Notice 2024-55 — SECURE 2.0 §72(t) exception guidance — Voya](https://www.voya.com/voya-insights/irs-issued-guidance-regarding-certain-exceptions-to-10-early-distribution-tax-under-irc-72t),
//! [Groom Law Group — IRS Guidance on New Exceptions](https://www.groom.com/resources/irs-guidance-on-new-exceptions-to-the-penalty-tax-for-early-qualified-plan-or-ira-withdrawals/).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Section72tException {
    None,
    /// §72(t)(2)(A)(i) — age 59½ or older.
    AgeFiftyNineHalfOrOlder,
    /// §72(t)(2)(A)(ii) — death of participant.
    DeathOfParticipant,
    /// §72(t)(2)(A)(iii) — disability.
    Disability,
    /// §72(t)(2)(A)(iv) — substantially equal periodic payments.
    SubstantiallyEqualPeriodicPayments,
    /// §72(t)(2)(A)(v) — separation from service after age 55.
    SeparationFromServiceAfter55,
    /// §72(t)(2)(B) — medical expenses > 7.5% AGI.
    MedicalExpensesOverFloor,
    /// §72(t)(2)(C) — QDRO.
    QualifiedDomesticRelationsOrder,
    /// §72(t)(2)(D) — qualified higher education (IRA only).
    QualifiedHigherEducation,
    /// §72(t)(2)(F) — first-time homebuyer ($10k lifetime, IRA only).
    FirstTimeHomebuyer,
    /// §72(t)(2)(G) — unemployed health insurance (IRA only).
    UnemployedHealthInsurance,
    /// §72(t)(11) — federally declared disaster (up to $22k).
    FederallyDeclaredDisaster,
    /// §72(t)(2)(H) — birth or adoption ($5k).
    BirthOrAdoption,
    /// SECURE 2.0 §326 — terminal illness.
    TerminalIllness,
    /// SECURE 2.0 §115 — emergency personal expense ($1k/year).
    EmergencyPersonalExpense,
    /// SECURE 2.0 §314 — domestic abuse victim ($10k indexed).
    DomesticAbuseVictim,
    /// SECURE 2.0 §334 — qualified long-term care ($2.5k/year).
    QualifiedLongTermCare,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section72tInput {
    /// Taxpayer's age at distribution date (years).
    pub age_at_distribution: u32,
    /// Distribution amount in dollars.
    pub distribution_amount_dollars: i64,
    /// Portion of the distribution includible in gross income
    /// (basis recovery is excluded — §72(t) only applies to the
    /// includible portion).
    pub includible_in_gross_income_dollars: i64,
    /// Which §72(t)(2) exception (if any) the taxpayer asserts.
    pub asserted_exception: Section72tException,
    /// True if the plan has adopted the SECURE 2.0 plan-optional
    /// exceptions (emergency / domestic abuse / etc.). Required
    /// for those exceptions to apply at the IRA / 401(k) level.
    pub plan_has_adopted_secure_20_optional_exceptions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section72tResult {
    pub additional_tax_applies: bool,
    pub asserted_exception_recognized: bool,
    /// Statutory cap applicable to the asserted exception, if any
    /// (e.g., $1k emergency / $5k birth / $10k abuse / $22k
    /// disaster). Zero when no cap.
    pub exception_cap_dollars: i64,
    /// Distribution amount qualifying for the exception (capped).
    pub qualifying_distribution_amount_dollars: i64,
    /// Distribution amount NOT qualifying (subject to 10% tax).
    pub non_qualifying_distribution_amount_dollars: i64,
    /// 10% additional tax on the non-qualifying includible amount.
    pub additional_tax_dollars: i64,
    pub citation: String,
    pub note: String,
}

const ADDITIONAL_TAX_RATE_BP: u32 = 1000; // 10%

pub fn compute(input: &Section72tInput) -> Section72tResult {
    // Age check — if 59½ or older, no §72(t) applies regardless of
    // asserted exception.
    let age_exempt = input.age_at_distribution >= 60
        || (input.age_at_distribution == 59 && input.distribution_amount_dollars >= 0);
    // Simplified: treat ≥ 59 as exempt for module purposes; caller
    // can pass exact age. The "½" is honored at age 60+.
    let age_exempt = input.age_at_distribution >= 60 || age_exempt; // (keeps simple semantic)

    let includible = input.includible_in_gross_income_dollars.max(0);

    // Determine cap based on asserted exception.
    let cap = match input.asserted_exception {
        Section72tException::EmergencyPersonalExpense => 1_000,
        Section72tException::BirthOrAdoption => 5_000,
        Section72tException::DomesticAbuseVictim => 10_000,
        Section72tException::FirstTimeHomebuyer => 10_000,
        Section72tException::FederallyDeclaredDisaster => 22_000,
        Section72tException::QualifiedLongTermCare => 2_500,
        _ => 0, // no statutory cap
    };

    // Plan-optional check for SECURE 2.0 exceptions.
    let plan_optional_exception = matches!(
        input.asserted_exception,
        Section72tException::EmergencyPersonalExpense
            | Section72tException::DomesticAbuseVictim
            | Section72tException::QualifiedLongTermCare
    );
    let plan_adoption_satisfied =
        !plan_optional_exception || input.plan_has_adopted_secure_20_optional_exceptions;

    // Exception recognition.
    let exception_recognized = age_exempt
        || (!matches!(input.asserted_exception, Section72tException::None)
            && plan_adoption_satisfied);

    // Qualifying amount = min(includible, cap if cap > 0, else
    // unlimited).
    let qualifying = if exception_recognized {
        if cap > 0 {
            includible.min(cap)
        } else {
            includible
        }
    } else {
        0
    };
    let non_qualifying = (includible - qualifying).max(0);

    let additional_tax_applies = !age_exempt && non_qualifying > 0;
    let additional_tax =
        ((non_qualifying as i128) * (ADDITIONAL_TAX_RATE_BP as i128) / 10_000) as i64;

    let exception_label = match input.asserted_exception {
        Section72tException::None => "no exception asserted",
        Section72tException::AgeFiftyNineHalfOrOlder => "§72(t)(2)(A)(i) age 59½+",
        Section72tException::DeathOfParticipant => "§72(t)(2)(A)(ii) death",
        Section72tException::Disability => "§72(t)(2)(A)(iii) disability",
        Section72tException::SubstantiallyEqualPeriodicPayments => {
            "§72(t)(2)(A)(iv) substantially equal periodic payments"
        }
        Section72tException::SeparationFromServiceAfter55 => {
            "§72(t)(2)(A)(v) separation from service after age 55"
        }
        Section72tException::MedicalExpensesOverFloor => {
            "§72(t)(2)(B) medical expenses over 7.5% AGI"
        }
        Section72tException::QualifiedDomesticRelationsOrder => {
            "§72(t)(2)(C) Qualified Domestic Relations Order"
        }
        Section72tException::QualifiedHigherEducation => {
            "§72(t)(2)(D) qualified higher education (IRA only)"
        }
        Section72tException::FirstTimeHomebuyer => {
            "§72(t)(2)(F) first-time homebuyer ($10k IRA-only)"
        }
        Section72tException::UnemployedHealthInsurance => {
            "§72(t)(2)(G) unemployed health insurance"
        }
        Section72tException::FederallyDeclaredDisaster => {
            "§72(t)(11) federally declared disaster ($22k)"
        }
        Section72tException::BirthOrAdoption => "§72(t)(2)(H) birth or adoption ($5k)",
        Section72tException::TerminalIllness => "SECURE 2.0 §326 terminal illness",
        Section72tException::EmergencyPersonalExpense => {
            "SECURE 2.0 §115 emergency personal expense ($1k, plan-optional)"
        }
        Section72tException::DomesticAbuseVictim => {
            "SECURE 2.0 §314 domestic abuse victim ($10k, plan-optional)"
        }
        Section72tException::QualifiedLongTermCare => {
            "SECURE 2.0 §334 qualified long-term care ($2.5k, eff. 2026, plan-optional)"
        }
    };

    let note = format!(
        "Age at distribution: {}; distribution ${}; includible ${}; asserted exception: {}; age-exempt: {}; exception recognized: {}{}; qualifying ${}; non-qualifying ${}; 10% additional tax ${}.",
        input.age_at_distribution,
        input.distribution_amount_dollars,
        includible,
        exception_label,
        age_exempt,
        exception_recognized,
        if plan_optional_exception && !input.plan_has_adopted_secure_20_optional_exceptions {
            " (plan has NOT adopted SECURE 2.0 optional exception — denied)"
        } else { "" },
        qualifying,
        non_qualifying,
        additional_tax,
    );

    Section72tResult {
        additional_tax_applies,
        asserted_exception_recognized: exception_recognized,
        exception_cap_dollars: cap,
        qualifying_distribution_amount_dollars: qualifying,
        non_qualifying_distribution_amount_dollars: non_qualifying,
        additional_tax_dollars: additional_tax,
        citation:
            "IRC §72(t)(1) 10% additional tax on early distributions before age 59½ from qualified retirement plans (§401 / §403 / §408 / §457); §72(t)(2) exception list: (A)(i) age 59½ + (A)(ii) death + (A)(iii) disability + (A)(iv) substantially equal periodic payments + (A)(v) separation from service after age 55 + (B) medical expenses > 7.5% AGI + (C) QDRO + (D) higher education + (F) first-time homebuyer $10k IRA-only + (G) unemployed health insurance + §72(t)(11) federally declared disaster $22k + (H) birth/adoption $5k + SECURE 2.0 §326 terminal illness + §115 emergency personal expense $1k plan-optional + §314 domestic abuse victim $10k plan-optional + §334 qualified long-term care $2.5k eff. 2026 plan-optional"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section72tInput {
        Section72tInput {
            age_at_distribution: 45,
            distribution_amount_dollars: 50_000,
            includible_in_gross_income_dollars: 50_000,
            asserted_exception: Section72tException::None,
            plan_has_adopted_secure_20_optional_exceptions: false,
        }
    }

    // ── §72(t)(1) baseline 10% tax ─────────────────────────────────

    #[test]
    fn pre_59_half_no_exception_10_pct_applies() {
        let r = compute(&base());
        assert!(r.additional_tax_applies);
        // 10% × $50k = $5k.
        assert_eq!(r.additional_tax_dollars, 5_000);
        assert_eq!(r.non_qualifying_distribution_amount_dollars, 50_000);
    }

    #[test]
    fn age_60_no_additional_tax() {
        let mut i = base();
        i.age_at_distribution = 60;
        let r = compute(&i);
        assert!(!r.additional_tax_applies);
        assert_eq!(r.additional_tax_dollars, 0);
    }

    // ── Capless exceptions ─────────────────────────────────────────

    #[test]
    fn death_exception_zeroes_tax() {
        let mut i = base();
        i.asserted_exception = Section72tException::DeathOfParticipant;
        let r = compute(&i);
        assert!(r.asserted_exception_recognized);
        assert_eq!(r.additional_tax_dollars, 0);
        assert_eq!(r.qualifying_distribution_amount_dollars, 50_000);
    }

    #[test]
    fn disability_exception_zeroes_tax() {
        let mut i = base();
        i.asserted_exception = Section72tException::Disability;
        let r = compute(&i);
        assert_eq!(r.additional_tax_dollars, 0);
    }

    #[test]
    fn substantially_equal_periodic_payments_exception_zeroes_tax() {
        let mut i = base();
        i.asserted_exception = Section72tException::SubstantiallyEqualPeriodicPayments;
        let r = compute(&i);
        assert_eq!(r.additional_tax_dollars, 0);
    }

    #[test]
    fn separation_after_55_exception_zeroes_tax() {
        let mut i = base();
        i.age_at_distribution = 56;
        i.asserted_exception = Section72tException::SeparationFromServiceAfter55;
        let r = compute(&i);
        assert_eq!(r.additional_tax_dollars, 0);
    }

    #[test]
    fn medical_expenses_over_floor_exception_zeroes_tax() {
        let mut i = base();
        i.asserted_exception = Section72tException::MedicalExpensesOverFloor;
        let r = compute(&i);
        assert_eq!(r.additional_tax_dollars, 0);
    }

    #[test]
    fn qdro_exception_zeroes_tax() {
        let mut i = base();
        i.asserted_exception = Section72tException::QualifiedDomesticRelationsOrder;
        let r = compute(&i);
        assert_eq!(r.additional_tax_dollars, 0);
    }

    #[test]
    fn higher_education_exception_zeroes_tax() {
        let mut i = base();
        i.asserted_exception = Section72tException::QualifiedHigherEducation;
        let r = compute(&i);
        assert_eq!(r.additional_tax_dollars, 0);
    }

    #[test]
    fn terminal_illness_secure_20_exception_zeroes_tax() {
        let mut i = base();
        i.asserted_exception = Section72tException::TerminalIllness;
        let r = compute(&i);
        assert_eq!(r.additional_tax_dollars, 0);
    }

    // ── Capped exceptions ──────────────────────────────────────────

    #[test]
    fn first_time_homebuyer_10k_cap() {
        let mut i = base();
        i.asserted_exception = Section72tException::FirstTimeHomebuyer;
        let r = compute(&i);
        assert_eq!(r.exception_cap_dollars, 10_000);
        // $50k − $10k cap = $40k non-qualifying × 10% = $4,000.
        assert_eq!(r.qualifying_distribution_amount_dollars, 10_000);
        assert_eq!(r.non_qualifying_distribution_amount_dollars, 40_000);
        assert_eq!(r.additional_tax_dollars, 4_000);
    }

    #[test]
    fn birth_or_adoption_5k_cap() {
        let mut i = base();
        i.asserted_exception = Section72tException::BirthOrAdoption;
        let r = compute(&i);
        assert_eq!(r.exception_cap_dollars, 5_000);
        assert_eq!(r.qualifying_distribution_amount_dollars, 5_000);
        assert_eq!(r.additional_tax_dollars, 4_500);
    }

    #[test]
    fn federally_declared_disaster_22k_cap() {
        let mut i = base();
        i.asserted_exception = Section72tException::FederallyDeclaredDisaster;
        let r = compute(&i);
        assert_eq!(r.exception_cap_dollars, 22_000);
        assert_eq!(r.qualifying_distribution_amount_dollars, 22_000);
        // $50k − $22k = $28k × 10% = $2,800.
        assert_eq!(r.additional_tax_dollars, 2_800);
    }

    // ── SECURE 2.0 plan-optional exceptions ────────────────────────

    #[test]
    fn emergency_personal_expense_plan_adopted_1k_cap() {
        let mut i = base();
        i.asserted_exception = Section72tException::EmergencyPersonalExpense;
        i.plan_has_adopted_secure_20_optional_exceptions = true;
        let r = compute(&i);
        assert_eq!(r.exception_cap_dollars, 1_000);
        assert_eq!(r.qualifying_distribution_amount_dollars, 1_000);
        // $50k − $1k = $49k × 10% = $4,900.
        assert_eq!(r.additional_tax_dollars, 4_900);
    }

    #[test]
    fn emergency_personal_expense_plan_not_adopted_no_exception() {
        let mut i = base();
        i.asserted_exception = Section72tException::EmergencyPersonalExpense;
        i.plan_has_adopted_secure_20_optional_exceptions = false;
        let r = compute(&i);
        assert!(!r.asserted_exception_recognized);
        // Full $50k × 10% = $5,000.
        assert_eq!(r.additional_tax_dollars, 5_000);
    }

    #[test]
    fn domestic_abuse_victim_plan_adopted_10k_cap() {
        let mut i = base();
        i.asserted_exception = Section72tException::DomesticAbuseVictim;
        i.plan_has_adopted_secure_20_optional_exceptions = true;
        let r = compute(&i);
        assert_eq!(r.exception_cap_dollars, 10_000);
        assert_eq!(r.qualifying_distribution_amount_dollars, 10_000);
        assert_eq!(r.additional_tax_dollars, 4_000);
    }

    #[test]
    fn domestic_abuse_victim_plan_not_adopted_no_exception() {
        let mut i = base();
        i.asserted_exception = Section72tException::DomesticAbuseVictim;
        i.plan_has_adopted_secure_20_optional_exceptions = false;
        let r = compute(&i);
        assert!(!r.asserted_exception_recognized);
        assert_eq!(r.additional_tax_dollars, 5_000);
    }

    #[test]
    fn long_term_care_plan_adopted_2_5k_cap() {
        let mut i = base();
        i.asserted_exception = Section72tException::QualifiedLongTermCare;
        i.plan_has_adopted_secure_20_optional_exceptions = true;
        let r = compute(&i);
        assert_eq!(r.exception_cap_dollars, 2_500);
        assert_eq!(r.qualifying_distribution_amount_dollars, 2_500);
        // $50k − $2,500 = $47,500 × 10% = $4,750.
        assert_eq!(r.additional_tax_dollars, 4_750);
    }

    // ── Plan-optional exception NOT plan-optional check ────────────

    #[test]
    fn terminal_illness_not_plan_optional() {
        // Terminal illness is statutory (SECURE 2.0 §326), not
        // plan-optional. Should work regardless of plan adoption.
        let mut i = base();
        i.asserted_exception = Section72tException::TerminalIllness;
        i.plan_has_adopted_secure_20_optional_exceptions = false;
        let r = compute(&i);
        assert!(r.asserted_exception_recognized);
        assert_eq!(r.additional_tax_dollars, 0);
    }

    // ── Distribution smaller than cap → full qualifying ───────────

    #[test]
    fn distribution_below_cap_full_qualifying() {
        let mut i = base();
        i.distribution_amount_dollars = 3_000;
        i.includible_in_gross_income_dollars = 3_000;
        i.asserted_exception = Section72tException::BirthOrAdoption;
        let r = compute(&i);
        assert_eq!(r.qualifying_distribution_amount_dollars, 3_000);
        assert_eq!(r.non_qualifying_distribution_amount_dollars, 0);
        assert_eq!(r.additional_tax_dollars, 0);
    }

    // ── Includible vs basis recovery ───────────────────────────────

    #[test]
    fn basis_recovery_not_subject_to_72t() {
        // Distribution $50k but only $20k includible (rest is basis).
        // 10% tax only on $20k.
        let mut i = base();
        i.distribution_amount_dollars = 50_000;
        i.includible_in_gross_income_dollars = 20_000;
        let r = compute(&i);
        assert_eq!(r.non_qualifying_distribution_amount_dollars, 20_000);
        assert_eq!(r.additional_tax_dollars, 2_000);
    }

    // ── Citation ───────────────────────────────────────────────────

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&base());
        assert!(r.citation.contains("§72(t)(1)"));
        assert!(r.citation.contains("§72(t)(2)"));
        assert!(r.citation.contains("§72(t)(11)"));
        assert!(r.citation.contains("SECURE 2.0"));
        assert!(r.citation.contains("§326 terminal illness"));
        assert!(r.citation.contains("§115 emergency"));
        assert!(r.citation.contains("§314 domestic abuse"));
        assert!(r.citation.contains("§334 qualified long-term care"));
        assert!(r.citation.contains("plan-optional"));
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn note_plan_not_adopted_explains() {
        let mut i = base();
        i.asserted_exception = Section72tException::EmergencyPersonalExpense;
        i.plan_has_adopted_secure_20_optional_exceptions = false;
        let r = compute(&i);
        assert!(r.note.contains("plan has NOT adopted"));
    }

    #[test]
    fn note_age_exempt_path() {
        let mut i = base();
        i.age_at_distribution = 60;
        let r = compute(&i);
        assert!(r.note.contains("age-exempt: true"));
    }

    // ── Precision ──────────────────────────────────────────────────

    #[test]
    fn very_large_distribution_no_precision_loss() {
        let mut i = base();
        i.distribution_amount_dollars = 10_000_000;
        i.includible_in_gross_income_dollars = 10_000_000;
        let r = compute(&i);
        // No exception → 10% × $10M = $1M.
        assert_eq!(r.additional_tax_dollars, 1_000_000);
    }
}
