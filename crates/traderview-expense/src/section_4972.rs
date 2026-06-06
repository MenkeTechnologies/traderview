//! IRC § 4972 — Tax on nondeductible contributions to
//! qualified employer plans. Imposes a 10% annual excise
//! tax on the EMPLOYER for any contributions to a
//! qualified retirement plan in excess of the § 404
//! deduction limit. Direct retirement-plan compliance
//! companion to section_4973 (IRA excess contributions —
//! iter 442), section_4974 (RMD excise — iter 436),
//! section_4975 (qualified plan prohibited transactions
//! — iter 434), section_4980 (employer reversion excise
//! — iter 460), section_4980h (employer shared
//! responsibility — iter 456), section_401k (iter 448),
//! section_408 (iter 432), section_408a (Roth IRA — iter
//! 430), section_415 (umbrella limits — iter 452),
//! section_457b (iter 450), section_4940 (PF NII excise —
//! iter 470), section_4941 (PF self-dealing — iter 468),
//! section_4958 (intermediate sanctions — iter 466),
//! section_4960 (ATEO executive comp 21% — iter 464),
//! section_1042 (ESOP rollover — iter 480).
//!
//! § 4972(a) imposes a 10% EXCISE TAX on the amount of
//! NONDEDUCTIBLE CONTRIBUTIONS under any qualified
//! employer plan as of the close of the employer's
//! taxable year. Tax is paid by the EMPLOYER making the
//! contributions.
//!
//! § 4972(c)(1)(A) NONDEDUCTIBLE CONTRIBUTIONS = sum of
//! current-year employer contributions PLUS unused
//! prior-year carryforwards LESS amount allowable as
//! deduction under § 404 for current year.
//!
//! § 4972(c)(2) ORDERING RULE: § 404 deduction is
//! treated as first applied to CARRYFORWARDS from
//! preceding taxable years (in order of time), and then
//! to current-year contributions. This means prior-year
//! nondeductible contributions consume current-year
//! deduction headroom before current-year contributions
//! qualify for deduction.
//!
//! § 4972(d) QUALIFIED EMPLOYER PLAN includes:
//! 1. § 401(a) — qualified retirement plan (defined
//!    benefit + defined contribution including § 401(k))
//! 2. § 403(a) — qualified annuity plan
//! 3. § 408(k) — Simplified Employee Pension (SEP-IRA)
//! 4. § 408(p) — SIMPLE IRA
//! 5. Multiple Employer Plans (MEPs) and Pooled Employer
//!    Plans (PEPs) treating each participating employer
//!    separately
//!
//! § 4972(c)(6) EXCEPTIONS — certain SEP and PSP
//! contributions are excluded:
//! 1. § 4972(c)(6)(A) — SEP contributions in excess of
//!    § 404(h)(1)(C) limit are not nondeductible IF
//!    individual receiving SEP contribution had § 415
//!    available (i.e., the contribution is allocable
//!    to the participant under § 415)
//! 2. § 4972(c)(6)(B) — contributions to a profit-
//!    sharing plan that became deductible only because
//!    of an increase in compensation paid AFTER
//!    plan-year-end are not nondeductible
//!
//! § 4972 RETURN: Form 5330 (Return of Excise Taxes
//! Related to Employee Benefit Plans) must be filed by
//! the employer annually if § 4972 tax applies. Form
//! 5330 is filed with IRS and excise tax is paid by the
//! due date.
//!
//! CARRYFORWARD MECHANICS: nondeductible amount carries
//! forward each year indefinitely. § 4972 tax is
//! imposed each year on the cumulative balance until
//! consumed by future deduction headroom or refunded
//! within statutory limits. To avoid compounding § 4972
//! tax across multiple years, employers should either
//! (a) request return of nondeductible contribution
//! from plan administrator (limited circumstances under
//! § 401(a)(2) reversion rules — see section_4980), or
//! (b) reduce future contributions until carryforward
//! consumed by § 404 deduction headroom.
//!
//! Trader-business-owner critical because (1) defined
//! benefit pension plan funding errors are the most
//! common § 4972 trigger — actuarial overfunding can
//! exceed § 404(o) deduction limits; (2) self-employed
//! § 401(k) contributions in excess of § 415 limit
//! create both § 415 violation AND § 4972 tax exposure;
//! (3) § 4972 carryforward compounds — a $50K
//! nondeductible contribution in Year 1 generates $5K
//! tax in Year 1 AND $5K each subsequent year until
//! consumed; (4) Form 5330 filing deadline is the LAST
//! DAY OF 7TH MONTH after employer tax-year-end (~July
//! 31 for calendar year); (5) § 4972 differs from § 4973
//! (iter 442) which applies to IRA-side excess
//! contributions paid by INDIVIDUAL; § 4972 applies to
//! plan-side excess paid by EMPLOYER.
//!
//! Distinction from § 4973 (iter 442): § 4973 imposes
//! 6% excise on excess IRA contributions paid by
//! INDIVIDUAL; § 4972 imposes 10% excise on plan-side
//! employer excess paid by EMPLOYER. § 4973 covers
//! traditional, Roth, SEP-IRA, SIMPLE-IRA, 403(b),
//! Coverdell, Archer, and HSA on the participant side;
//! § 4972 covers § 401(a), § 403(a), § 408(k), and
//! § 408(p) on the employer side.
//!
//! Distinction from § 4974 (iter 436): § 4974 imposes
//! 25% (post-SECURE 2.0) excise on RMD shortfall;
//! § 4972 imposes 10% on contribution excess. RMD is a
//! distribution issue; § 4972 is a contribution issue.
//!
//! Distinction from § 4975 (iter 434): § 4975 imposes
//! 15%/100% on prohibited transactions between plan and
//! disqualified person; § 4972 imposes 10% on plain
//! contribution excess regardless of related-party
//! status.
//!
//! Authority: 26 U.S.C. § 4972; § 4972(a); § 4972(b);
//! § 4972(c)(1)(A); § 4972(c)(2); § 4972(c)(6); § 4972(d);
//! § 404 (deduction); § 404(h)(1)(C) (SEP limit);
//! § 404(o) (DB plan deduction limit); § 401(a)
//! (qualified plan); § 401(a)(2) (reversion);
//! § 403(a) (qualified annuity); § 408(k) (SEP);
//! § 408(p) (SIMPLE); § 415 (compensation limit);
//! § 4973 (IRA excess contributions); § 4974 (RMD
//! excise); § 4975 (prohibited transactions); § 4980
//! (reversion excise); Form 5330 (Return of Excise
//! Taxes Related to Employee Benefit Plans); Deficit
//! Reduction Act of 1984, Pub. L. 98-369 — original
//! enactment; Omnibus Budget Reconciliation Act of
//! 1989, Pub. L. 101-239 — current 10% rate.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum QualifiedEmployerPlanType {
    Section401aQualifiedPlan,
    Section403aQualifiedAnnuity,
    Section408kSep,
    Section408pSimple,
    NotApplicable,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub plan_type: QualifiedEmployerPlanType,
    pub current_year_contributions_cents: u64,
    pub prior_year_nondeductible_carryforward_cents: u64,
    pub section_404_deduction_limit_cents: u64,
    pub section_4972c6_sep_exception_applies: bool,
    pub section_4972c6_post_year_end_compensation_increase: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Compliant,
    ExceptionApplies,
    ExciseTaxOwed,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub severity: Severity,
    pub aggregate_contributions_cents: u64,
    pub allowable_deduction_cents: u64,
    pub nondeductible_amount_cents: u64,
    pub excise_tax_cents: u64,
    pub new_carryforward_cents: u64,
    pub notes: Vec<String>,
}

pub const TAX_RATE_PCT: u64 = 10;

pub type Section4972Input = Input;
pub type Section4972Result = Output;

pub fn check(input: &Input) -> Output {
    let notes: Vec<String> = vec![
        "§ 4972(a) imposes 10% EXCISE TAX on amount of NONDEDUCTIBLE CONTRIBUTIONS under any qualified employer plan as of close of employer's taxable year. Tax paid by EMPLOYER making the contributions.".to_string(),
        "§ 4972(c)(1)(A) NONDEDUCTIBLE CONTRIBUTIONS = current-year employer contributions PLUS unused prior-year carryforwards LESS amount allowable as deduction under § 404 for current year.".to_string(),
        "§ 4972(c)(2) ORDERING RULE: § 404 deduction treated as first applied to CARRYFORWARDS from preceding taxable years (in order of time), THEN to current-year contributions. Prior-year nondeductible consume current-year deduction headroom before current-year qualify.".to_string(),
        "§ 4972(d) QUALIFIED EMPLOYER PLAN includes § 401(a) qualified retirement plan + § 403(a) qualified annuity + § 408(k) SEP-IRA + § 408(p) SIMPLE IRA + multiple/pooled employer plan participating employer.".to_string(),
        "§ 4972(c)(6) EXCEPTIONS: (A) § 4972(c)(6)(A) SEP contributions in excess of § 404(h)(1)(C) limit are not nondeductible IF allocable to participant under § 415; (B) § 4972(c)(6)(B) PSP contributions that became deductible only because of post-plan-year-end compensation increase not nondeductible.".to_string(),
        "Form 5330 (Return of Excise Taxes Related to Employee Benefit Plans): employer must file annually if § 4972 tax applies; filing deadline last day of 7th month after employer tax-year-end (approximately July 31 for calendar-year employer).".to_string(),
        "Carryforward mechanics: nondeductible amount carries forward each year indefinitely. § 4972 tax imposed each year on cumulative balance until consumed by future § 404 deduction headroom. To avoid compounding § 4972 tax, employer must either request return of contribution under § 401(a)(2) reversion (§ 4980 (iter 460) reversion excise tax also applies) OR reduce future contributions until carryforward consumed.".to_string(),
        "Distinction from § 4973 (iter 442): § 4973 6% on IRA-side excess paid by INDIVIDUAL; § 4972 10% on plan-side excess paid by EMPLOYER. Distinction from § 4974 (iter 436): § 4974 25% on RMD distribution shortfall; § 4972 on contribution excess. Distinction from § 4975 (iter 434): § 4975 15%/100% on prohibited transactions; § 4972 on plain contribution excess regardless of related-party.".to_string(),
        "Companion: section_4973 (iter 442), section_4974 (iter 436), section_4975 (iter 434), section_4980 (iter 460), section_4980h (iter 456), section_401k (iter 448), section_408 (iter 432), section_408a (iter 430), section_415 (iter 452), section_457b (iter 450), section_4940 (iter 470), section_4941 (iter 468), section_4958 (iter 466), section_4960 (iter 464), section_1042 (iter 480).".to_string(),
    ];

    if matches!(input.plan_type, QualifiedEmployerPlanType::NotApplicable) {
        let mut n = notes;
        n.push(
            "Plan type not subject to § 4972 — not a qualified employer plan under § 4972(d)."
                .to_string(),
        );
        return Output {
            severity: Severity::NotApplicable,
            aggregate_contributions_cents: 0,
            allowable_deduction_cents: 0,
            nondeductible_amount_cents: 0,
            excise_tax_cents: 0,
            new_carryforward_cents: 0,
            notes: n,
        };
    }

    if input.section_4972c6_sep_exception_applies
        || input.section_4972c6_post_year_end_compensation_increase
    {
        let mut n = notes;
        let citation = if input.section_4972c6_sep_exception_applies {
            "§ 4972(c)(6)(A) SEP exception (contribution allocable to participant under § 415)"
        } else {
            "§ 4972(c)(6)(B) post-plan-year-end compensation increase exception"
        };
        n.push(format!(
            "§ 4972 exception applies: {} — contribution not treated as nondeductible.",
            citation
        ));
        return Output {
            severity: Severity::ExceptionApplies,
            aggregate_contributions_cents: input
                .current_year_contributions_cents
                .saturating_add(input.prior_year_nondeductible_carryforward_cents),
            allowable_deduction_cents: input.section_404_deduction_limit_cents,
            nondeductible_amount_cents: 0,
            excise_tax_cents: 0,
            new_carryforward_cents: 0,
            notes: n,
        };
    }

    let aggregate_contributions = input
        .current_year_contributions_cents
        .saturating_add(input.prior_year_nondeductible_carryforward_cents);

    let allowable_deduction = input
        .section_404_deduction_limit_cents
        .min(aggregate_contributions);

    let nondeductible_amount = aggregate_contributions.saturating_sub(allowable_deduction);

    let excise_tax = nondeductible_amount
        .saturating_mul(TAX_RATE_PCT)
        .checked_div(100)
        .unwrap_or(0);

    let severity = if nondeductible_amount == 0 {
        Severity::Compliant
    } else {
        Severity::ExciseTaxOwed
    };

    let mut n = notes;
    n.push(format!(
        "Computed § 4972: aggregate contributions ${}.{:02}; allowable § 404 deduction ${}.{:02}; nondeductible amount ${}.{:02}; 10% excise tax ${}.{:02}; new carryforward to next year ${}.{:02}.",
        aggregate_contributions / 100,
        aggregate_contributions % 100,
        allowable_deduction / 100,
        allowable_deduction % 100,
        nondeductible_amount / 100,
        nondeductible_amount % 100,
        excise_tax / 100,
        excise_tax % 100,
        nondeductible_amount / 100,
        nondeductible_amount % 100
    ));

    Output {
        severity,
        aggregate_contributions_cents: aggregate_contributions,
        allowable_deduction_cents: allowable_deduction,
        nondeductible_amount_cents: nondeductible_amount,
        excise_tax_cents: excise_tax,
        new_carryforward_cents: nondeductible_amount,
        notes: n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            plan_type: QualifiedEmployerPlanType::Section401aQualifiedPlan,
            current_year_contributions_cents: 100_000_00,
            prior_year_nondeductible_carryforward_cents: 0,
            section_404_deduction_limit_cents: 100_000_00,
            section_4972c6_sep_exception_applies: false,
            section_4972c6_post_year_end_compensation_increase: false,
        }
    }

    #[test]
    fn not_applicable_plan_type() {
        let mut i = baseline();
        i.plan_type = QualifiedEmployerPlanType::NotApplicable;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn compliant_when_contributions_within_limit() {
        let i = baseline(); // $100K = $100K limit
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
        assert_eq!(out.nondeductible_amount_cents, 0);
        assert_eq!(out.excise_tax_cents, 0);
        assert_eq!(out.new_carryforward_cents, 0);
    }

    #[test]
    fn over_contribution_50k_excise_5k() {
        let mut i = baseline();
        i.current_year_contributions_cents = 150_000_00; // $50K over $100K limit
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExciseTaxOwed);
        assert_eq!(out.nondeductible_amount_cents, 50_000_00);
        assert_eq!(out.excise_tax_cents, 5_000_00); // 10% × $50K
        assert_eq!(out.new_carryforward_cents, 50_000_00);
    }

    #[test]
    fn ordering_rule_carryforward_consumes_deduction_first() {
        // Prior carryforward $30K + current contributions $80K = $110K
        // Deduction limit $100K
        // Per § 4972(c)(2): deduction first applies to carryforward, then current
        // Result: nondeductible = $110K - $100K = $10K
        // Computed: aggregate $110K - min($100K, $110K) = $10K
        let mut i = baseline();
        i.prior_year_nondeductible_carryforward_cents = 30_000_00;
        i.current_year_contributions_cents = 80_000_00;
        i.section_404_deduction_limit_cents = 100_000_00;
        let out = check(&i);
        assert_eq!(out.aggregate_contributions_cents, 110_000_00);
        assert_eq!(out.allowable_deduction_cents, 100_000_00);
        assert_eq!(out.nondeductible_amount_cents, 10_000_00);
        assert_eq!(out.excise_tax_cents, 1_000_00);
    }

    #[test]
    fn full_carryforward_consumption_zero_new() {
        // Carryforward $30K + current contributions $0 = $30K aggregate
        // Deduction limit $100K (ample headroom)
        // Nondeductible = $0; carryforward fully consumed
        let mut i = baseline();
        i.prior_year_nondeductible_carryforward_cents = 30_000_00;
        i.current_year_contributions_cents = 0;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
        assert_eq!(out.nondeductible_amount_cents, 0);
        assert_eq!(out.new_carryforward_cents, 0);
    }

    #[test]
    fn carryforward_compounds_when_deduction_limit_exhausted() {
        // Year 1: carryforward $50K + current $100K = $150K
        // Deduction limit $100K → nondeductible $50K (continues to compound)
        let mut i = baseline();
        i.prior_year_nondeductible_carryforward_cents = 50_000_00;
        i.current_year_contributions_cents = 100_000_00;
        let out = check(&i);
        assert_eq!(out.nondeductible_amount_cents, 50_000_00);
        assert_eq!(out.excise_tax_cents, 5_000_00);
    }

    #[test]
    fn section_4972c6_sep_exception_applies() {
        let mut i = baseline();
        i.current_year_contributions_cents = 200_000_00;
        i.section_4972c6_sep_exception_applies = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExceptionApplies);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4972(c)(6)(A)"));
        assert!(joined.contains("§ 415"));
    }

    #[test]
    fn section_4972c6_post_year_end_comp_increase_exception() {
        let mut i = baseline();
        i.current_year_contributions_cents = 200_000_00;
        i.section_4972c6_post_year_end_compensation_increase = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExceptionApplies);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4972(c)(6)(B)"));
    }

    #[test]
    fn section_403a_qualified_annuity_plan() {
        let mut i = baseline();
        i.plan_type = QualifiedEmployerPlanType::Section403aQualifiedAnnuity;
        i.current_year_contributions_cents = 150_000_00;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExciseTaxOwed);
    }

    #[test]
    fn section_408k_sep_plan() {
        let mut i = baseline();
        i.plan_type = QualifiedEmployerPlanType::Section408kSep;
        i.current_year_contributions_cents = 150_000_00;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExciseTaxOwed);
    }

    #[test]
    fn section_408p_simple_plan() {
        let mut i = baseline();
        i.plan_type = QualifiedEmployerPlanType::Section408pSimple;
        i.current_year_contributions_cents = 150_000_00;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExciseTaxOwed);
    }

    #[test]
    fn zero_contributions_compliant() {
        let mut i = baseline();
        i.current_year_contributions_cents = 0;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn under_contribution_compliant() {
        let mut i = baseline();
        i.current_year_contributions_cents = 50_000_00; // $50K under $100K
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
        assert_eq!(out.allowable_deduction_cents, 50_000_00);
    }

    #[test]
    fn boundary_exactly_equal_limit_compliant() {
        let i = baseline(); // $100K = $100K
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn boundary_one_cent_over_limit() {
        let mut i = baseline();
        i.current_year_contributions_cents = 100_000_01;
        let out = check(&i);
        assert_eq!(out.nondeductible_amount_cents, 1);
        // 10% of 1 cent = 0
        assert_eq!(out.excise_tax_cents, 0);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4972(a)"));
        assert!(joined.contains("§ 4972(c)(1)(A)"));
        assert!(joined.contains("§ 4972(c)(2)"));
        assert!(joined.contains("§ 4972(c)(6)"));
        assert!(joined.contains("§ 4972(d)"));
        assert!(joined.contains("§ 404"));
        assert!(joined.contains("§ 401(a)"));
        assert!(joined.contains("§ 403(a)"));
        assert!(joined.contains("§ 408(k)"));
        assert!(joined.contains("§ 408(p)"));
        assert!(joined.contains("§ 4973 (iter 442)"));
        assert!(joined.contains("§ 4974 (iter 436)"));
        assert!(joined.contains("§ 4975 (iter 434)"));
        assert!(joined.contains("§ 4980 (iter 460)"));
        assert!(joined.contains("Form 5330"));
    }

    #[test]
    fn note_pins_10_percent_rate() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("10% EXCISE TAX"));
        assert!(joined.contains("EMPLOYER"));
    }

    #[test]
    fn note_pins_nondeductible_definition() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("NONDEDUCTIBLE CONTRIBUTIONS"));
        assert!(joined.contains("PLUS unused prior-year carryforwards"));
    }

    #[test]
    fn note_pins_ordering_rule() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4972(c)(2) ORDERING RULE"));
        assert!(joined.contains("first applied to CARRYFORWARDS"));
    }

    #[test]
    fn note_pins_four_plan_types() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 401(a) qualified retirement plan"));
        assert!(joined.contains("§ 403(a) qualified annuity"));
        assert!(joined.contains("§ 408(k) SEP-IRA"));
        assert!(joined.contains("§ 408(p) SIMPLE IRA"));
    }

    #[test]
    fn note_pins_two_exceptions() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4972(c)(6)(A) SEP"));
        assert!(joined.contains("§ 4972(c)(6)(B) PSP"));
        assert!(joined.contains("post-plan-year-end compensation"));
    }

    #[test]
    fn note_pins_form_5330_deadline() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("Form 5330"));
        assert!(joined.contains("last day of 7th month"));
        assert!(joined.contains("July 31"));
    }

    #[test]
    fn note_pins_carryforward_mechanics() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("indefinitely"));
        assert!(joined.contains("§ 401(a)(2) reversion"));
        assert!(joined.contains("§ 4980 (iter 460)"));
    }

    #[test]
    fn note_pins_4973_4974_4975_distinctions() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4973 (iter 442)"));
        assert!(joined.contains("§ 4974 (iter 436)"));
        assert!(joined.contains("§ 4975 (iter 434)"));
        assert!(joined.contains("INDIVIDUAL"));
        assert!(joined.contains("EMPLOYER"));
    }

    #[test]
    fn note_pins_companion_modules() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("section_4973"));
        assert!(joined.contains("section_4974"));
        assert!(joined.contains("section_4975"));
        assert!(joined.contains("section_4980"));
        assert!(joined.contains("section_401k"));
        assert!(joined.contains("section_415"));
        assert!(joined.contains("section_1042"));
    }

    #[test]
    fn truth_table_six_severity_cells() {
        // NotApplicable plan type
        let c1 = check(&Input {
            plan_type: QualifiedEmployerPlanType::NotApplicable,
            ..baseline()
        });
        assert_eq!(c1.severity, Severity::NotApplicable);

        // SEP exception applies
        let c2 = check(&Input {
            section_4972c6_sep_exception_applies: true,
            current_year_contributions_cents: 200_000_00,
            ..baseline()
        });
        assert_eq!(c2.severity, Severity::ExceptionApplies);

        // Post-year-end comp increase exception
        let c3 = check(&Input {
            section_4972c6_post_year_end_compensation_increase: true,
            current_year_contributions_cents: 200_000_00,
            ..baseline()
        });
        assert_eq!(c3.severity, Severity::ExceptionApplies);

        // Compliant within limit
        let c4 = check(&baseline());
        assert_eq!(c4.severity, Severity::Compliant);

        // Excise tax owed
        let c5 = check(&Input {
            current_year_contributions_cents: 150_000_00,
            ..baseline()
        });
        assert_eq!(c5.severity, Severity::ExciseTaxOwed);

        // Compliant with carryforward consumption
        let c6 = check(&Input {
            prior_year_nondeductible_carryforward_cents: 30_000_00,
            current_year_contributions_cents: 0,
            ..baseline()
        });
        assert_eq!(c6.severity, Severity::Compliant);
    }

    #[test]
    fn saturating_arithmetic_overflow_defense() {
        let i = Input {
            current_year_contributions_cents: u64::MAX,
            prior_year_nondeductible_carryforward_cents: u64::MAX,
            section_404_deduction_limit_cents: 100_000_00,
            ..baseline()
        };
        let out = check(&i);
        // No panic
        assert_eq!(out.severity, Severity::ExciseTaxOwed);
    }

    #[test]
    fn realistic_db_plan_overfunding_50k_compounds_5k_yearly() {
        // Year 1: DB plan over-funded by $50K
        let mut i = baseline();
        i.current_year_contributions_cents = 250_000_00; // $250K contribution
        i.section_404_deduction_limit_cents = 200_000_00; // § 404(o) DB limit $200K
        let out = check(&i);
        assert_eq!(out.nondeductible_amount_cents, 50_000_00);
        assert_eq!(out.excise_tax_cents, 5_000_00);
        // Compounding: this same $50K carryforward generates $5K tax each future year
        // until consumed by deduction headroom.
        assert_eq!(out.new_carryforward_cents, 50_000_00);
    }

    #[test]
    fn realistic_self_employed_401k_excess_over_415_limit() {
        // Self-employed § 401(k) plan; § 415 individual limit creates effective
        // contribution cap; excess over § 415 limit creates § 4972 exposure
        let mut i = baseline();
        i.current_year_contributions_cents = 80_000_00; // $80K contribution
        i.section_404_deduction_limit_cents = 70_000_00; // § 415-derived limit
        let out = check(&i);
        assert_eq!(out.nondeductible_amount_cents, 10_000_00);
        assert_eq!(out.excise_tax_cents, 1_000_00);
    }

    #[test]
    fn multi_year_carryforward_simulation_year_3() {
        // Simulate year 3 of compounded $50K carryforward
        // Each year: $50K carryforward + $50K contribution = $100K aggregate
        // Deduction limit $80K → nondeductible = $20K
        // (This shows reduction strategy works over time)
        let mut i = baseline();
        i.prior_year_nondeductible_carryforward_cents = 50_000_00;
        i.current_year_contributions_cents = 50_000_00;
        i.section_404_deduction_limit_cents = 80_000_00;
        let out = check(&i);
        assert_eq!(out.aggregate_contributions_cents, 100_000_00);
        assert_eq!(out.allowable_deduction_cents, 80_000_00);
        assert_eq!(out.nondeductible_amount_cents, 20_000_00);
        assert_eq!(out.excise_tax_cents, 2_000_00);
        // Strategy success: carryforward reduced from $50K to $20K
    }
}
