//! IRC § 415 — Limitations on benefits and
//! contributions under qualified retirement plans.
//! Direct trader companion to section_401k (iter 448),
//! section_408 (traditional IRA — iter 432),
//! section_408a (Roth IRA — iter 430), section_457b
//! (iter 450), section_4973 (excess contribution
//! excise — iter 442), section_4974 (RMD excise —
//! iter 436), section_162m ($1M public-company exec
//! comp deduction — iter 446).
//!
//! § 415 is the UMBRELLA STATUTE governing maximum
//! limits on retirement plan benefits and
//! contributions; sets ceilings that interact with
//! lower limits in § 401(k) elective deferrals,
//! § 457(b) governmental DC plans, § 403(b) annuity
//! plans, § 408 IRAs, and § 1402 self-employed
//! contributions.
//!
//! Trader-critical because high-comp traders maxing
//! out retirement contributions across multiple plan
//! structures must navigate § 415 limits in
//! coordination with § 401(a)(17) compensation cap,
//! § 402(g)(1) elective deferral aggregation, and
//! plan-specific limitations.
//!
//! § 415(a) STATUTORY PURPOSE — denies qualified
//! status under § 401(a) to any trust that is part of
//! a plan exceeding § 415(b) or § 415(c) limits;
//! disqualification cascades to all participants
//! (not just over-limit individual).
//!
//! **§ 415(b) DEFINED BENEFIT (DB) PLAN LIMIT**:
//! Maximum annual benefit (life-only, age 62-65
//! commencement) is lesser of:
//! 1. § 415(b)(1)(A) DOLLAR LIMIT — $290,000 for 2026
//!    (up from $280,000 for 2025); indexed annually
//!    under § 415(d);
//! 2. § 415(b)(1)(B) COMPENSATION LIMIT — 100% of
//!    participant's average compensation for highest 3
//!    consecutive years; NOT subject to § 401(a)(17)
//!    cap when computing DB limit.
//!
//! § 415(b)(2) — actuarial adjustments for
//! commencement age (reduction for ages below 62;
//! increase for ages above 65); form of benefit
//! (joint and survivor); cost-of-living adjustments
//! at retirement.
//!
//! **§ 415(c) DEFINED CONTRIBUTION (DC) PLAN LIMIT**:
//! Annual addition to participant's account in DC
//! plan cannot exceed lesser of:
//! 1. § 415(c)(1)(A) DOLLAR LIMIT — $72,000 for 2026
//!    (up from $70,000 for 2025); indexed annually;
//! 2. § 415(c)(1)(B) 100% of participant's
//!    COMPENSATION for the year (subject to
//!    § 401(a)(17) $360,000 cap).
//!
//! § 415(c)(2) ANNUAL ADDITION DEFINITION:
//! 1. EMPLOYER CONTRIBUTIONS to participant's
//!    account;
//! 2. EMPLOYEE CONTRIBUTIONS (both pretax and Roth
//!    designated deferrals);
//! 3. FORFEITURES allocated to participant's account;
//! 4. NOT included: catch-up contributions under
//!    § 414(v) (separate limit);
//! 5. NOT included: rollovers from other plans;
//! 6. NOT included: § 408A Roth IRA contributions
//!    (separate § 408 limits);
//! 7. NOT included: investment earnings.
//!
//! **§ 415(d) COST-OF-LIVING ADJUSTMENTS**:
//! 1. Dollar limits in § 415(b)(1)(A) and § 415(c)(1)
//!    (A) adjusted annually by Secretary of Treasury
//!    based on CPI-U;
//! 2. Adjustments rounded to nearest $1,000 for DB
//!    limit; nearest $1,000 for DC annual addition;
//! 3. Effective January 1 of calendar year following
//!    publication.
//!
//! **2026 dollar limits framework (IRS Notice 2025-
//! 67)**:
//!
//! | Limit | 2026 Amount |
//! |-------|-------------|
//! | § 415(b)(1)(A) DB annual benefit | $290,000 |
//! | § 415(c)(1)(A) DC annual addition | $72,000 |
//! | § 401(a)(17) compensation limit | $360,000 |
//! | § 402(g)(1) elective deferral | $24,500 |
//! | Age 50+ catch-up § 414(v) | $8,000 |
//! | Ages 60-63 enhanced catch-up § 414(v)(2)(E) | $11,250 |
//! | HCE threshold | $160,000 |
//! | Roth catch-up wage threshold § 414(v)(7) | $150,000 |
//!
//! **§ 415(f) AGGREGATION RULES**:
//! 1. § 415(f)(1) — all DC plans of single employer
//!    AGGREGATED for § 415(c) limit;
//! 2. § 415(f)(1) — all DB plans of single employer
//!    AGGREGATED for § 415(b) limit;
//! 3. § 415(f)(2) — DC and DB limits applied
//!    SEPARATELY (participant may be at maximum in
//!    both);
//! 4. § 414(b)/(c)/(m)/(o) — controlled group, common
//!    control, affiliated service group, and ASG
//!    rules treat related employers as single
//!    employer for § 415 purposes;
//! 5. UNRELATED employers — separate § 415 limits
//!    apply at each employer (no aggregation).
//!
//! **§ 415(g) ANTI-CUTBACK RULE** — § 411(d)(6)
//! protected benefits cannot be reduced when § 415
//! limits adjust; plan must accommodate accrued
//! benefits even if subsequent COLA-adjusted limits
//! lower than prior years.
//!
//! **§ 415(k) GRANDFATHERED OLD-LIMIT BENEFITS**:
//! 1. § 415(k)(1) — pre-1976 accrued benefits
//!    grandfathered;
//! 2. § 415(k)(2) — pre-1982 accrued benefits subject
//!    to old TEFRA $90,000 dollar limit if greater
//!    than current § 415(b) limit.
//!
//! **§ 415(n) USERRA REINSTATEMENT** — qualifying
//! military service member returning under USERRA can
//! receive make-up contributions during reemployment
//! period without violating § 415 limits.
//!
//! **§ 415 INTERACTION WITH § 401(k) / § 402(g)**:
//! 1. § 415(c) ANNUAL ADDITION = elective deferrals +
//!    employer match + after-tax + forfeitures;
//! 2. § 402(g)(1) ELECTIVE DEFERRAL LIMIT applies
//!    separately to participant's pretax + Roth
//!    deferrals ($24,500 for 2026);
//! 3. § 414(v) CATCH-UP contributions are
//!    DISREGARDED for § 415(c) purposes (separate
//!    limit);
//! 4. MEGA BACKDOOR ROTH uses available § 415(c)
//!    room after elective deferral + employer match
//!    for after-tax contributions converted to Roth
//!    via § 408A(d)(3) in-plan rollover.
//!
//! **§ 415 INTERACTION WITH § 457(b)**:
//! 1. § 457(b) NOT subject to § 415(c) limits
//!    (governmental and tax-exempt § 457(b) have
//:    SEPARATE $24,500 limit);
//! 2. § 402(g)(1) AGGREGATION rule does NOT apply
//!    to § 457(b) (key for DOUBLE DEFERRAL strategy
//!    with § 401(k)/§ 403(b)).
//!
//! **§ 415 INTERACTION WITH § 415(b) DEFINED
//! BENEFIT**:
//! 1. § 415(b) APPLIES SEPARATELY from § 415(c) — a
//!    participant in BOTH a DB plan and a DC plan
//!    may receive MAXIMUM benefit under § 415(b)
//!    AND maximum annual addition under § 415(c) in
//!    same year (no combined limit since 2002 TEFRA
//!    repeal);
//! 2. § 415(b)(2)(B)(iii) accommodates social
//!    security supplements;
//! 3. § 415(b)(2)(C) requires actuarial reduction
//!    for early retirement.
//!
//! **Trader-critical fact patterns**:
//!
//! Trader age 35 maxes out § 401(k): $24,500 elective
//! plus $7,500 employer match plus $40,000 after-tax
//! to mega backdoor Roth equals $72,000 § 415(c)
//! annual addition (exactly at 2026 limit).
//!
//! Trader age 50 maxes § 401(k) AND § 415(c): $24,500
//! elective + $8,000 catch-up (DISREGARDED for § 415
//! (c)) + $7,500 match + $40,000 after-tax = $80,000
//! total cash flow; § 415(c) cap $72,000 = match +
//! after-tax + elective only; catch-up uncounted.
//!
//! Trader at multi-controlled-group corporation —
//! § 414(b)/(c)/(m)/(o) aggregation treats all
//! related employers as single employer; § 415(c)
//! $72,000 cap applies across ALL related-employer
//! plans, not separately.
//!
//! Trader at TWO unrelated employers — § 415(c)
//! $72,000 limit applies SEPARATELY at each employer;
//! TOTAL annual addition can be UP TO $144,000 ($72K
//! × 2) absent any § 414 aggregation; § 402(g)(1)
//! still aggregates elective deferral across
//! employers at $24,500.
//!
//! Trader participates in both DB pension and § 401
//! (k): § 415(b) annual benefit $290K AT RETIREMENT
//! AGE PLUS § 415(c) annual addition $72,000 CURRENT
//! YEAR (no combined limit since 2002 TEFRA repeal).
//!
//! Citations: 26 USC § 415(a)-(n); 26 USC § 415(b)(1)
//! (A) (DB dollar limit); 26 USC § 415(b)(1)(B)
//! (compensation limit); 26 USC § 415(c)(1)(A) (DC
//! dollar limit); 26 USC § 415(c)(2) (annual addition
//! definition); 26 USC § 415(d) (COLA adjustments);
//! 26 USC § 415(f) (aggregation rules); 26 USC
//! § 415(g) (anti-cutback); 26 USC § 415(k)
//! (grandfathered old limits); 26 USC § 415(n)
//! (USERRA); 26 USC § 401(a)(17); 26 USC § 401(k);
//! 26 USC § 402(g)(1); 26 USC § 408A(d)(3); 26 USC
//! § 414(b)/(c)/(m)/(o) (controlled group +
//! affiliated service group); 26 USC § 414(v)
//! (catch-up contributions); IRS Notice 2025-67
//! (2026 dollar limitations); Treas. Reg. § 1.415-1
//! through § 1.415-12; Rev. Proc. 2007-44.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlanType {
    DefinedContribution,
    DefinedBenefit,
    Both,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section415Input {
    pub plan_type: PlanType,
    /// Total annual addition to DC account in cents
    /// (employer contributions + employee
    /// contributions + forfeitures).
    pub dc_annual_addition_cents: u64,
    /// Catch-up contributions in cents under § 414(v)
    /// (DISREGARDED for § 415(c)).
    pub catch_up_contributions_cents: u64,
    /// Plan compensation for the year in cents.
    pub plan_compensation_cents: u64,
    /// DB plan annual benefit payable at age 62-65 in
    /// cents (life-only basis).
    pub db_annual_benefit_cents: u64,
    /// Average compensation for highest 3 consecutive
    /// years in cents.
    pub average_high_3_year_compensation_cents: u64,
    /// Whether multiple employers in controlled group
    /// / common control / affiliated service group /
    /// ASG.
    pub controlled_group_aggregation: bool,
    /// Whether genuinely unrelated employers (no
    /// § 414(b)/(c)/(m)/(o) aggregation).
    pub multiple_unrelated_employers: bool,
    /// Whether USERRA qualifying military service
    /// member returning under § 415(n) make-up
    /// rights.
    pub userra_make_up_contributions: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section415Result {
    pub dc_dollar_limit_cents: u64,
    pub dc_compensation_limit_cents: u64,
    pub dc_applicable_limit_cents: u64,
    pub db_dollar_limit_cents: u64,
    pub db_compensation_limit_cents: u64,
    pub db_applicable_limit_cents: u64,
    pub dc_compliant: bool,
    pub db_compliant: bool,
    pub catch_up_disregarded_for_dc: bool,
    pub dc_and_db_apply_separately: bool,
    pub controlled_group_aggregation_applies: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section415Input) -> Section415Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    // 2026 limits in cents
    let dc_dollar_limit_cents: u64 = 7_200_000;
    let compensation_limit_401a17: u64 = 36_000_000;
    let db_dollar_limit_cents: u64 = 29_000_000;

    let dc_compensation_limit_cents = input.plan_compensation_cents.min(compensation_limit_401a17);

    let dc_applicable_limit_cents = dc_dollar_limit_cents.min(dc_compensation_limit_cents);

    let db_compensation_limit_cents = input.average_high_3_year_compensation_cents;
    let db_applicable_limit_cents = db_dollar_limit_cents.min(db_compensation_limit_cents);

    let dc_compliant = !matches!(
        input.plan_type,
        PlanType::DefinedContribution | PlanType::Both
    ) || input.dc_annual_addition_cents <= dc_applicable_limit_cents;

    let db_compliant = !matches!(input.plan_type, PlanType::DefinedBenefit | PlanType::Both)
        || input.db_annual_benefit_cents <= db_applicable_limit_cents;

    let catch_up_disregarded_for_dc = input.catch_up_contributions_cents > 0;

    let dc_and_db_apply_separately = matches!(input.plan_type, PlanType::Both);

    let controlled_group_aggregation_applies = input.controlled_group_aggregation;

    if !dc_compliant {
        failure_reasons.push(format!(
            "26 USC § 415(c)(1)(A) DC ANNUAL ADDITION EXCEEDED — annual addition of {} cents (employer + employee + forfeitures, EXCLUDING § 414(v) catch-up) exceeds lesser of (a) $72,000 dollar limit OR (b) 100% of compensation ({}); excess subject to correction under § 415(c)(6) or plan disqualification under § 415(a)",
            input.dc_annual_addition_cents,
            dc_applicable_limit_cents
        ));
    }

    if !db_compliant {
        failure_reasons.push(format!(
            "26 USC § 415(b)(1) DB ANNUAL BENEFIT EXCEEDED — annual benefit of {} cents exceeds lesser of (a) $290,000 dollar limit OR (b) 100% of average high-3-year compensation ({}); benefit must be REDUCED or plan loses qualified status under § 415(a)",
            input.db_annual_benefit_cents,
            db_applicable_limit_cents
        ));
    }

    if input.plan_compensation_cents > compensation_limit_401a17 {
        failure_reasons.push(format!(
            "26 USC § 401(a)(17) COMPENSATION LIMIT — plan compensation of {} cents EXCEEDS 2026 cap of {} cents ($360,000); only first $360,000 considered for § 415(c) DC annual addition; § 415(b) DB plan exempt from § 401(a)(17) cap when computing the compensation-based DB limit",
            input.plan_compensation_cents,
            compensation_limit_401a17
        ));
    }

    if catch_up_disregarded_for_dc {
        failure_reasons.push(format!(
            "26 USC § 414(v) CATCH-UP CONTRIBUTIONS DISREGARDED — catch-up contributions of {} cents under § 414(v) (age 50+) or § 414(v)(2)(E) (ages 60-63 enhanced SECURE 2.0 § 109) are EXCLUDED from § 415(c) annual addition calculation; separate $8,000 / $11,250 limit governs",
            input.catch_up_contributions_cents
        ));
    }

    if dc_and_db_apply_separately {
        failure_reasons.push(
            "26 USC § 415(f)(2) — DC and DB plan limits APPLY SEPARATELY; a participant in BOTH may receive MAXIMUM § 415(b) annual benefit AND MAXIMUM § 415(c) annual addition in same year (no combined limit since 2002 TEFRA repeal); maximum tax-deferred wealth accumulation strategy for high-comp employees".to_string(),
        );
    }

    if controlled_group_aggregation_applies {
        failure_reasons.push(
            "26 USC § 414(b)/(c)/(m)/(o) CONTROLLED GROUP AGGREGATION — related employers (controlled group / common control / affiliated service group / ASG) treated as SINGLE EMPLOYER for § 415 purposes; § 415(c) $72,000 cap applies ACROSS ALL related-employer plans; § 415(b) $290,000 cap applies ACROSS ALL related-employer DB plans; participant cannot circumvent limits through related entities".to_string(),
        );
    }

    if input.multiple_unrelated_employers && !controlled_group_aggregation_applies {
        failure_reasons.push(
            "UNRELATED EMPLOYERS — § 415(c) $72,000 limit applies SEPARATELY at each genuinely unrelated employer (no § 414 aggregation); TOTAL annual addition across multiple employers can reach $72,000 × N; HOWEVER § 402(g)(1) ELECTIVE DEFERRAL LIMIT $24,500 still AGGREGATES across all employers (employee's responsibility to track); excess elective deferral subject to § 4979 excise tax".to_string(),
        );
    }

    if input.userra_make_up_contributions {
        failure_reasons.push(
            "26 USC § 415(n) USERRA REINSTATEMENT — qualifying military service member returning under USERRA may receive make-up contributions during reemployment period WITHOUT VIOLATING § 415 limits; make-up period equals lesser of (a) 3× period of military service; OR (b) 5 years; make-up contributions count against limits for years of military service, not year actually contributed".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "26 USC § 415 — UMBRELLA STATUTE governing maximum limits on retirement plan benefits and contributions; sets CEILINGS that interact with lower limits in § 401(k) elective deferrals, § 457(b) governmental DC plans, § 403(b) annuity plans, § 408 IRAs, § 1402 self-employed contributions".to_string(),
        "26 USC § 415(a) STATUTORY PURPOSE — denies qualified status under § 401(a) to any trust that is part of a plan exceeding § 415(b) or § 415(c) limits; disqualification cascades to ALL PARTICIPANTS not just over-limit individual; harsh consequence motivates strict compliance".to_string(),
        "26 USC § 415(b) DEFINED BENEFIT PLAN LIMIT — maximum annual benefit (life-only, age 62-65 commencement) = lesser of (a) § 415(b)(1)(A) DOLLAR LIMIT $290,000 for 2026; OR (b) § 415(b)(1)(B) 100% of average compensation for highest 3 consecutive years; § 415(b)(2) actuarial adjustments for early/late commencement and form of benefit".to_string(),
        "26 USC § 415(c) DEFINED CONTRIBUTION PLAN LIMIT — annual addition cannot exceed lesser of (a) § 415(c)(1)(A) DOLLAR LIMIT $72,000 for 2026; OR (b) § 415(c)(1)(B) 100% of compensation (subject to § 401(a)(17) $360,000 cap); § 415(c)(2) annual addition INCLUDES employer contributions + employee pretax + Roth + forfeitures; EXCLUDES § 414(v) catch-up + rollovers + investment earnings".to_string(),
        "26 USC § 415(d) COST-OF-LIVING ADJUSTMENTS — dollar limits in § 415(b)(1)(A) and § 415(c)(1)(A) adjusted annually by Secretary of Treasury based on CPI-U; rounded to nearest $1,000; effective January 1 of calendar year following publication".to_string(),
        "2026 dollar limits (IRS Notice 2025-67): § 415(b)(1)(A) DB annual benefit $290,000; § 415(c)(1)(A) DC annual addition $72,000; § 401(a)(17) compensation limit $360,000; § 402(g)(1) elective deferral $24,500; age 50+ catch-up $8,000; ages 60-63 enhanced catch-up $11,250 SECURE 2.0 § 109; HCE threshold $160,000; Roth catch-up wage threshold $150,000 § 414(v)(7)".to_string(),
        "26 USC § 415(f) AGGREGATION RULES: (1) § 415(f)(1) all DC plans of single employer AGGREGATED for § 415(c) limit; (2) § 415(f)(1) all DB plans of single employer AGGREGATED for § 415(b) limit; (3) § 415(f)(2) DC and DB limits applied SEPARATELY (participant may be at maximum in both); (4) § 414(b)/(c)/(m)/(o) controlled group + common control + affiliated service group + ASG rules treat related employers as single employer; (5) UNRELATED employers — separate § 415 limits apply".to_string(),
        "26 USC § 415(g) ANTI-CUTBACK RULE — § 411(d)(6) protected benefits cannot be reduced when § 415 limits adjust; plan must accommodate accrued benefits even if subsequent COLA-adjusted limits lower than prior years".to_string(),
        "26 USC § 415(k) GRANDFATHERED OLD-LIMIT BENEFITS: (1) § 415(k)(1) pre-1976 accrued benefits grandfathered; (2) § 415(k)(2) pre-1982 accrued benefits subject to old TEFRA $90,000 dollar limit if greater than current § 415(b) limit".to_string(),
        "26 USC § 415(n) USERRA REINSTATEMENT — qualifying military service member returning under USERRA may receive make-up contributions during reemployment period WITHOUT VIOLATING § 415 limits; make-up period equals lesser of (a) 3× period of military service OR (b) 5 years; make-up contributions count against limits for YEARS OF MILITARY SERVICE not year actually contributed".to_string(),
        "§ 415 interaction with § 401(k) / § 402(g): § 415(c) ANNUAL ADDITION = elective deferrals + employer match + after-tax + forfeitures; § 402(g)(1) ELECTIVE DEFERRAL LIMIT applies separately to participant's pretax + Roth deferrals ($24,500 for 2026); § 414(v) CATCH-UP contributions DISREGARDED for § 415(c) purposes; MEGA BACKDOOR ROTH uses available § 415(c) room after elective deferral + employer match for after-tax contributions converted to Roth via § 408A(d)(3)".to_string(),
        "§ 415 interaction with § 457(b): § 457(b) NOT subject to § 415(c) limits (governmental and tax-exempt § 457(b) have SEPARATE $24,500 limit); § 402(g)(1) AGGREGATION rule does NOT apply to § 457(b); enables DOUBLE DEFERRAL strategy with § 401(k)/§ 403(b) — $49,000 combined elective deferral for 2026".to_string(),
        "§ 415 interaction with DB+DC dual participation: § 415(b) APPLIES SEPARATELY from § 415(c) — participant in BOTH a DB plan and DC plan may receive MAXIMUM benefit under § 415(b) AND maximum annual addition under § 415(c) in same year; no combined limit since 2002 TEFRA repeal; maximum tax-deferred wealth accumulation strategy for high-comp employees".to_string(),
        "Trader-critical fact patterns: (1) age 35 maxes § 401(k) — $24,500 elective + $7,500 match + $40,000 after-tax = $72,000 § 415(c) annual addition (exactly at 2026 limit); (2) age 50 maxes § 401(k) + § 415(c) — $24,500 + $8,000 catch-up (DISREGARDED for § 415(c)) + $7,500 match + $40,000 after-tax = $80,000 total cash flow / $72,000 § 415(c); (3) controlled-group corporation — § 414 aggregation treats all related as single employer; $72,000 cap applies ACROSS plans; (4) two unrelated employers — $72,000 SEPARATELY at each = $144,000 total / § 402(g)(1) still aggregates elective at $24,500; (5) DB + DC dual participation — $290K + $72,000 / no combined limit since 2002 TEFRA repeal".to_string(),
        "Companion to section_401k (iter 448 § 401(k) cash or deferred arrangements) + section_408 (traditional IRA) + section_408a (Roth IRA) + section_457b (iter 450 governmental and tax-exempt deferred compensation) + section_4973 (excess contribution excise) + section_4974 (RMD excise) + section_162m ($1M public-company exec comp deduction limit)".to_string(),
    ];

    Section415Result {
        dc_dollar_limit_cents,
        dc_compensation_limit_cents,
        dc_applicable_limit_cents,
        db_dollar_limit_cents,
        db_compensation_limit_cents,
        db_applicable_limit_cents,
        dc_compliant,
        db_compliant,
        catch_up_disregarded_for_dc,
        dc_and_db_apply_separately,
        controlled_group_aggregation_applies,
        failure_reasons,
        citation: "26 USC § 415(a)-(n); 26 USC § 415(b)(1)(A); 26 USC § 415(b)(1)(B); 26 USC § 415(c)(1)(A); 26 USC § 415(c)(2); 26 USC § 415(d); 26 USC § 415(f); 26 USC § 415(g); 26 USC § 415(k); 26 USC § 415(n); 26 USC § 401(a)(17); 26 USC § 401(k); 26 USC § 402(g)(1); 26 USC § 408A(d)(3); 26 USC § 414(b); 26 USC § 414(c); 26 USC § 414(m); 26 USC § 414(o); 26 USC § 414(v); IRS Notice 2025-67 (2026 dollar limitations); Treas. Reg. § 1.415-1 through § 1.415-12; Rev. Proc. 2007-44",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dc_max_2026() -> Section415Input {
        Section415Input {
            plan_type: PlanType::DefinedContribution,
            dc_annual_addition_cents: 7_200_000,
            catch_up_contributions_cents: 0,
            plan_compensation_cents: 30_000_000,
            db_annual_benefit_cents: 0,
            average_high_3_year_compensation_cents: 0,
            controlled_group_aggregation: false,
            multiple_unrelated_employers: false,
            userra_make_up_contributions: false,
        }
    }

    #[test]
    fn dc_max_2026_compliant() {
        let r = check(&dc_max_2026());
        assert!(r.dc_compliant);
        assert_eq!(r.dc_dollar_limit_cents, 7_200_000);
    }

    #[test]
    fn dc_over_72k_violation() {
        let mut i = dc_max_2026();
        i.dc_annual_addition_cents = 7_500_000;
        let r = check(&i);
        assert!(!r.dc_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 415(c)(1)(A)")
                && f.contains("$72,000")
                && f.contains("§ 415(c)(6)")));
    }

    #[test]
    fn dc_compensation_limit_constrains_at_low_comp() {
        let mut i = dc_max_2026();
        i.plan_compensation_cents = 5_000_000;
        i.dc_annual_addition_cents = 5_000_000;
        let r = check(&i);
        assert!(r.dc_compliant);
        assert_eq!(r.dc_applicable_limit_cents, 5_000_000);
    }

    #[test]
    fn dc_compensation_over_360k_capped_at_401a17() {
        let mut i = dc_max_2026();
        i.plan_compensation_cents = 50_000_000;
        let r = check(&i);
        assert_eq!(r.dc_compensation_limit_cents, 36_000_000);
        assert_eq!(r.dc_applicable_limit_cents, 7_200_000);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 401(a)(17)")
            && f.contains("$360,000")
            && f.contains("DB plan exempt")));
    }

    #[test]
    fn catch_up_disregarded_for_dc() {
        let mut i = dc_max_2026();
        i.catch_up_contributions_cents = 800_000;
        let r = check(&i);
        assert!(r.catch_up_disregarded_for_dc);
        assert!(r.failure_reasons.iter().any(|f| f
            .contains("§ 414(v) CATCH-UP CONTRIBUTIONS DISREGARDED")
            && f.contains("$8,000")
            && f.contains("$11,250")));
    }

    #[test]
    fn db_max_2026_compliant() {
        let mut i = dc_max_2026();
        i.plan_type = PlanType::DefinedBenefit;
        i.db_annual_benefit_cents = 29_000_000;
        i.average_high_3_year_compensation_cents = 30_000_000;
        let r = check(&i);
        assert!(r.db_compliant);
        assert_eq!(r.db_dollar_limit_cents, 29_000_000);
    }

    #[test]
    fn db_over_290k_violation() {
        let mut i = dc_max_2026();
        i.plan_type = PlanType::DefinedBenefit;
        i.db_annual_benefit_cents = 30_000_000;
        i.average_high_3_year_compensation_cents = 35_000_000;
        let r = check(&i);
        assert!(!r.db_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 415(b)(1)") && f.contains("$290,000")));
    }

    #[test]
    fn db_compensation_limit_lower_than_dollar_limit_constrains() {
        let mut i = dc_max_2026();
        i.plan_type = PlanType::DefinedBenefit;
        i.db_annual_benefit_cents = 15_000_000;
        i.average_high_3_year_compensation_cents = 12_000_000;
        let r = check(&i);
        assert_eq!(r.db_applicable_limit_cents, 12_000_000);
        assert!(!r.db_compliant);
    }

    #[test]
    fn both_dc_and_db_apply_separately() {
        let mut i = dc_max_2026();
        i.plan_type = PlanType::Both;
        i.dc_annual_addition_cents = 7_200_000;
        i.db_annual_benefit_cents = 29_000_000;
        i.average_high_3_year_compensation_cents = 30_000_000;
        let r = check(&i);
        assert!(r.dc_compliant);
        assert!(r.db_compliant);
        assert!(r.dc_and_db_apply_separately);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 415(f)(2)")
            && f.contains("APPLY SEPARATELY")
            && f.contains("no combined limit since 2002 TEFRA repeal")));
    }

    #[test]
    fn controlled_group_aggregation_disclosed() {
        let mut i = dc_max_2026();
        i.controlled_group_aggregation = true;
        let r = check(&i);
        assert!(r.controlled_group_aggregation_applies);
        assert!(r.failure_reasons.iter().any(|f| f
            .contains("§ 414(b)/(c)/(m)/(o) CONTROLLED GROUP AGGREGATION")
            && f.contains("SINGLE EMPLOYER")
            && f.contains("ACROSS ALL related-employer plans")));
    }

    #[test]
    fn unrelated_employers_separate_limits() {
        let mut i = dc_max_2026();
        i.multiple_unrelated_employers = true;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("UNRELATED EMPLOYERS")
                && f.contains("SEPARATELY at each")
                && f.contains("§ 402(g)(1) ELECTIVE DEFERRAL LIMIT $24,500 still AGGREGATES")
                && f.contains("§ 4979")));
    }

    #[test]
    fn unrelated_with_controlled_group_no_separate_disclosure() {
        let mut i = dc_max_2026();
        i.multiple_unrelated_employers = true;
        i.controlled_group_aggregation = true;
        let r = check(&i);
        assert!(!r
            .failure_reasons
            .iter()
            .any(|f| f.contains("UNRELATED EMPLOYERS")));
    }

    #[test]
    fn userra_reinstatement_disclosed() {
        let mut i = dc_max_2026();
        i.userra_make_up_contributions = true;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 415(n) USERRA REINSTATEMENT")
                && f.contains("lesser of (a) 3× period")
                && f.contains("OR (b) 5 years")));
    }

    #[test]
    fn limits_pinned_at_2026() {
        let r = check(&dc_max_2026());
        assert_eq!(r.dc_dollar_limit_cents, 7_200_000);
        assert_eq!(r.db_dollar_limit_cents, 29_000_000);
    }

    #[test]
    fn plan_type_truth_table_three_cells() {
        for plan_type in [
            PlanType::DefinedContribution,
            PlanType::DefinedBenefit,
            PlanType::Both,
        ] {
            let mut i = dc_max_2026();
            i.plan_type = plan_type;
            i.average_high_3_year_compensation_cents = 30_000_000;
            i.db_annual_benefit_cents = 1_000_000;
            let r = check(&i);
            let _ = r.dc_compliant;
            let _ = r.db_compliant;
        }
    }

    #[test]
    fn dc_dollar_limit_uniquely_constrains_at_high_comp() {
        let mut high_comp = dc_max_2026();
        high_comp.plan_compensation_cents = 100_000_000;
        let r_high = check(&high_comp);
        assert_eq!(r_high.dc_applicable_limit_cents, 7_200_000);

        let mut low_comp = dc_max_2026();
        low_comp.plan_compensation_cents = 5_000_000;
        let r_low = check(&low_comp);
        assert_eq!(r_low.dc_applicable_limit_cents, 5_000_000);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&dc_max_2026());
        assert!(r.citation.contains("§ 415(a)-(n)"));
        assert!(r.citation.contains("§ 415(b)(1)(A)"));
        assert!(r.citation.contains("§ 415(b)(1)(B)"));
        assert!(r.citation.contains("§ 415(c)(1)(A)"));
        assert!(r.citation.contains("§ 415(c)(2)"));
        assert!(r.citation.contains("§ 415(d)"));
        assert!(r.citation.contains("§ 415(f)"));
        assert!(r.citation.contains("§ 415(g)"));
        assert!(r.citation.contains("§ 415(k)"));
        assert!(r.citation.contains("§ 415(n)"));
        assert!(r.citation.contains("§ 401(a)(17)"));
        assert!(r.citation.contains("§ 402(g)(1)"));
        assert!(r.citation.contains("§ 408A(d)(3)"));
        assert!(r.citation.contains("§ 414(b)"));
        assert!(r.citation.contains("§ 414(c)"));
        assert!(r.citation.contains("§ 414(m)"));
        assert!(r.citation.contains("§ 414(o)"));
        assert!(r.citation.contains("§ 414(v)"));
        assert!(r.citation.contains("IRS Notice 2025-67"));
        assert!(r.citation.contains("Treas. Reg. § 1.415-1"));
    }

    #[test]
    fn note_pins_umbrella_statute() {
        let r = check(&dc_max_2026());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 415 — UMBRELLA STATUTE") && n.contains("CEILINGS")));
    }

    #[test]
    fn note_pins_subsection_a_disqualification_cascade() {
        let r = check(&dc_max_2026());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 415(a) STATUTORY PURPOSE")
                && n.contains("ALL PARTICIPANTS")
                && n.contains("not just over-limit")));
    }

    #[test]
    fn note_pins_subsection_b_db_limit() {
        let r = check(&dc_max_2026());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 415(b) DEFINED BENEFIT")
                && n.contains("$290,000")
                && n.contains("§ 415(b)(2) actuarial")));
    }

    #[test]
    fn note_pins_subsection_c_dc_limit() {
        let r = check(&dc_max_2026());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 415(c) DEFINED CONTRIBUTION")
                && n.contains("$72,000")
                && n.contains("§ 415(c)(2) annual addition INCLUDES")
                && n.contains("EXCLUDES § 414(v)")));
    }

    #[test]
    fn note_pins_subsection_d_cola() {
        let r = check(&dc_max_2026());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 415(d) COST-OF-LIVING ADJUSTMENTS") && n.contains("CPI-U")));
    }

    #[test]
    fn note_pins_2026_eight_limits() {
        let r = check(&dc_max_2026());
        assert!(r.notes.iter().any(|n| n.contains("2026 dollar limits")
            && n.contains("$290,000")
            && n.contains("$72,000")
            && n.contains("$360,000")
            && n.contains("$24,500")
            && n.contains("$8,000")
            && n.contains("$11,250")
            && n.contains("$160,000")
            && n.contains("$150,000")));
    }

    #[test]
    fn note_pins_subsection_f_aggregation_rules() {
        let r = check(&dc_max_2026());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 415(f) AGGREGATION RULES")
                && n.contains("§ 415(f)(2) DC and DB limits applied SEPARATELY")
                && n.contains("§ 414(b)/(c)/(m)/(o)")));
    }

    #[test]
    fn note_pins_subsection_g_anti_cutback() {
        let r = check(&dc_max_2026());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 415(g) ANTI-CUTBACK RULE") && n.contains("§ 411(d)(6)")));
    }

    #[test]
    fn note_pins_subsection_k_grandfathered() {
        let r = check(&dc_max_2026());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 415(k) GRANDFATHERED OLD-LIMIT BENEFITS")
                && n.contains("pre-1976")
                && n.contains("pre-1982")
                && n.contains("$90,000")
                && n.contains("TEFRA")));
    }

    #[test]
    fn note_pins_subsection_n_userra() {
        let r = check(&dc_max_2026());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 415(n) USERRA REINSTATEMENT")
                && n.contains("YEARS OF MILITARY SERVICE")));
    }

    #[test]
    fn note_pins_section_401k_interaction() {
        let r = check(&dc_max_2026());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 415 interaction with § 401(k)")
                && n.contains("MEGA BACKDOOR ROTH")
                && n.contains("§ 408A(d)(3)")));
    }

    #[test]
    fn note_pins_section_457b_interaction() {
        let r = check(&dc_max_2026());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 415 interaction with § 457(b)")
                && n.contains("DOUBLE DEFERRAL")
                && n.contains("$49,000")));
    }

    #[test]
    fn note_pins_db_dc_dual_participation() {
        let r = check(&dc_max_2026());
        assert!(r.notes.iter().any(|n| n
            .contains("§ 415 interaction with DB+DC dual participation")
            && n.contains("no combined limit since 2002 TEFRA repeal")));
    }

    #[test]
    fn note_pins_trader_fact_patterns_five() {
        let r = check(&dc_max_2026());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Trader-critical fact patterns")
                && n.contains("$72,000 § 415(c)")
                && n.contains("ACROSS plans")
                && n.contains("$144,000 total")
                && n.contains("TEFRA repeal")));
    }

    #[test]
    fn note_pins_companion_modules() {
        let r = check(&dc_max_2026());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Companion to section_401k")
                && n.contains("section_408a")
                && n.contains("section_457b")
                && n.contains("section_162m")));
    }

    #[test]
    fn defensive_overflow_saturating() {
        let mut i = dc_max_2026();
        i.dc_annual_addition_cents = u64::MAX;
        i.plan_compensation_cents = u64::MAX;
        let r = check(&i);
        let _ = r.dc_applicable_limit_cents;
    }
}
