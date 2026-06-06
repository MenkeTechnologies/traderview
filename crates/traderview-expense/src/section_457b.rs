//! IRC § 457(b) — Governmental and Tax-Exempt
//! Deferred Compensation Plans. Direct trader
//! companion to section_401k (iter 448), section_408
//! (traditional IRA — iter 432), section_408a (Roth
//! IRA — iter 430), section_4973 (excess contribution
//! excise — iter 442), section_4974 (RMD excise —
//! iter 436), section_72t (10% early-withdrawal
//! penalty), section_162m ($1M public-company exec
//! comp deduction — iter 446).
//!
//! § 457(b) allows STATE/LOCAL GOVERNMENT and TAX-
//! EXEMPT ENTITY employees to make elective deferrals
//! to ELIGIBLE DEFERRED COMPENSATION PLAN; contributions
//! excluded from current gross income; growth tax-
//! deferred; distributions taxed as ordinary income at
//! receipt.
//!
//! Two structurally distinct § 457(b) plan types:
//! 1. **GOVERNMENTAL § 457(b)** — state, political
//!    subdivision, agency, or instrumentality;
//!    assets held in TRUST FOR EXCLUSIVE BENEFIT of
//!    participants and beneficiaries under § 457(g);
//!    NO 10% § 72(t) EARLY-WITHDRAWAL PENALTY (except
//!    rollovers from other plan types); rollovers
//!    permitted to/from § 401(k), § 403(b), § 408
//!    IRAs.
//! 2. **TAX-EXEMPT § 457(b)** — § 501(c) tax-exempt
//!    organization; UNFUNDED "top-hat" plan for
//!    SELECT GROUP OF MANAGEMENT OR HIGHLY
//!    COMPENSATED EMPLOYEES; assets remain
//!    EMPLOYER'S GENERAL ASSETS (not held in trust);
//!    SUBSTANTIAL CREDIT RISK; § 72(t) 10% penalty
//!    APPLIES on early withdrawal; rollovers NOT
//!    PERMITTED to other plan types (unlike
//!    governmental).
//!
//! Trader-critical because public-sector and
//! nonprofit traders can stack § 457(b) on top of
//! § 401(k) or § 403(b) for substantial DOUBLE
//! DEFERRAL strategy — separate $24,500 elective
//! deferral limits at each plan type (governmental
//! § 457(b) NOT aggregated with § 401(k) under
//! § 402(g)(1)).
//!
//! **2026 contribution limit framework**:
//!
//! Annual elective deferral § 457(b)(2): $24,500
//!
//! Age 50+ catch-up § 414(v): $8,000 (governmental
//! § 457(b) only; tax-exempt § 457(b) NOT eligible
//! for age-50 catch-up)
//!
//! Ages 60-63 SECURE 2.0 § 109 enhanced catch-up
//! § 414(v)(2)(E): $11,250 (governmental § 457(b)
//! only)
//!
//! § 457(b)(3) special 3-YEAR PRE-RETIREMENT
//! CATCH-UP: lesser of (a) 2× annual dollar
//! amount = $49,000 for 2026; OR (b) underutilized
//! limitation from prior years
//!
//! § 401(a)(17) compensation limit: $360,000 (2026)
//!
//! **§ 457(b)(3) SPECIAL 3-YEAR PRE-RETIREMENT
//! CATCH-UP**:
//! - Available in 3 taxable years immediately
//!   preceding NORMAL RETIREMENT AGE specified by
//!   plan;
//! - Maximum deferral = lesser of (a) 2× annual
//!   limit ($49,000 for 2026); OR (b) underutilized
//!   contribution capacity from prior eligible years
//!   (combination of unused prior-year limits);
//! - CANNOT BE USED in same year as age-50 catch-up
//!   or ages-60-63 enhanced catch-up (anti-stacking
//!   rule);
//! - Available to BOTH governmental and tax-exempt
//!   § 457(b) plans (unlike age-50 which is
//!   governmental-only).
//!
//! **§ 457(g) GOVERNMENTAL § 457(b) TRUST**
//! requirement — Small Business Jobs Protection Act
//! of 1996 (Pub. L. 104-188) required governmental
//! § 457(b) assets be held in TRUST for EXCLUSIVE
//! BENEFIT of participants and beneficiaries
//! effective January 1, 1999; protects assets from
//! employer creditors; distinguishes governmental
//! from tax-exempt § 457(b).
//!
//! **§ 72(t) early-withdrawal penalty interaction**:
//! 1. GOVERNMENTAL § 457(b) — NO 10% penalty (except
//!    rollovers from other plan types are subject to
//!    standard § 72(t) penalty if withdrawn from
//!    § 457(b) within the early-withdrawal trigger
//!    period of that rolled-over plan type);
//! 2. TAX-EXEMPT § 457(b) — § 72(t) 10% penalty
//!    APPLIES on early withdrawal before 59½ except
//!    for separation-from-service age-55 exception,
//!    death, disability, and other § 72(t)(2)
//!    enumerated exceptions.
//!
//! **§ 402(g)(1) AGGREGATION RULE — important
//! distinction**:
//! 1. § 401(k) + § 403(b) plans — aggregated under
//!    § 402(g)(1) single elective deferral limit
//!    ($24,500 in 2026);
//! 2. § 457(b) — NOT AGGREGATED; SEPARATE $24,500
//!    elective deferral limit;
//! 3. This creates DOUBLE DEFERRAL strategy —
//!    governmental or nonprofit employee with both
//!    § 401(k)/§ 403(b) AND § 457(b) can defer
//!    UP TO $49,000 in 2026 (vs. $24,500 single
//!    plan).
//!
//! **§ 457(b)(2) NORMAL RETIREMENT AGE** —
//! determined by plan document; generally cannot be
//! before age 65 for governmental plans or 50 for
//! tax-exempt plans; affects when special 3-year
//! catch-up can be invoked.
//!
//! **§ 457(d)(1) DISTRIBUTION TRIGGERS**:
//! 1. SEPARATION FROM SERVICE — most common trigger;
//! 2. AGE 70½ (now superseded by § 401(a)(9) RMD
//!    rules — age 73 for born 1951-1959, age 75 for
//!    born 1960+);
//! 3. UNFORESEEABLE EMERGENCY (severe financial
//!    hardship from extraordinary unforeseeable
//!    circumstances);
//! 4. PLAN TERMINATION;
//! 5. § 457(d)(9) in-service distribution at 59½
//!    (added by SECURE Act 2.0 § 314 for emergency
//!    expenses; previously age 70½).
//!
//! **§ 457(d)(2) ROLLOVER RULES**:
//! 1. GOVERNMENTAL § 457(b) — rollovers PERMITTED
//!    to/from § 401(k), § 403(b), § 408 IRAs;
//!    rolled-over amounts retain SOURCE plan's
//!    § 72(t) 10% penalty status;
//! 2. TAX-EXEMPT § 457(b) — rollovers NOT PERMITTED
//!    to other plan types (only to another tax-
//!    exempt § 457(b) within 60 days).
//!
//! Trader-critical fact patterns:
//!
//! State employee age 40 defers $24,500 § 457(b)
//! plus $24,500 § 401(k) — total $49,000 deferral
//! (DOUBLE DEFERRAL strategy under § 402(g)(1)
//! non-aggregation rule).
//!
//! State employee age 55 separates from service:
//! governmental § 457(b) allows immediate
//! distribution at any age WITHOUT § 72(t) 10%
//! penalty (key differentiator from § 401(k) which
//! requires age 55+ rule of 55 or 59½).
//!
//! Tenured nonprofit hospital CFO age 62 maxes
//! tax-exempt § 457(b): $24,500 + $11,250 enhanced
//! catch-up NOT available (tax-exempt § 457(b)
//! ineligible) — falls back to $24,500 only.
//!
//! State employee age 62 (within 3 years of plan's
//! normal retirement age 65) — invokes § 457(b)(3)
//! special 3-year catch-up: $49,000 deferral
//! (2× annual limit); cannot combine with $11,250
//! enhanced catch-up.
//!
//! Tax-exempt § 457(b) participant whose nonprofit
//! employer files Chapter 11 bankruptcy — UNFUNDED
//! plan assets are PART OF EMPLOYER'S GENERAL
//! ASSETS subject to claims of all creditors;
//! § 457(b) participant becomes UNSECURED CREDITOR
//! (key risk distinguishing tax-exempt from
//! governmental § 457(b)).
//!
//! Citations: 26 USC § 457(a)-(g); 26 USC § 457(b)
//! (eligible deferred compensation plan); 26 USC
//! § 457(b)(2) (annual deferral limit); 26 USC
//! § 457(b)(3) (special 3-year pre-retirement
//! catch-up); 26 USC § 457(d) (distribution
//! triggers); 26 USC § 457(g) (governmental trust
//! requirement); 26 USC § 402(g)(1) (elective
//! deferral aggregation); 26 USC § 72(t)(10%
//! early-withdrawal penalty); 26 USC § 414(v)
//! (catch-up contributions); 26 USC § 414(v)(2)(E)
//! (SECURE 2.0 enhanced catch-up ages 60-63); Pub.
//! L. 104-188 (Small Business Jobs Protection Act
//! of 1996 — § 457(g) trust requirement); SECURE
//! Act 2.0 of 2022 § 314 (in-service distribution
//! at 59½); SECURE Act 2.0 of 2022 § 109 (enhanced
//! catch-up ages 60-63); Pub. L. 117-328
//! (Consolidated Appropriations Act, 2023); IRS
//! Notice 2025-67 (2026 dollar limitations); Treas.
//! Reg. § 1.457-1 through § 1.457-12.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlanType {
    /// Governmental § 457(b) — state/local government;
    /// assets in § 457(g) trust; no 10% § 72(t)
    /// penalty.
    Governmental,
    /// Tax-exempt § 457(b) — § 501(c) organization;
    /// unfunded top-hat plan; substantial credit risk.
    TaxExempt,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgeBracket {
    UnderFifty,
    /// Age 50-59 (governmental § 457(b) eligible for
    /// age-50 catch-up).
    FiftyToFiftyNine,
    /// Ages 60-63 (governmental § 457(b) eligible for
    /// SECURE 2.0 enhanced catch-up).
    SixtyToSixtyThree,
    SixtyFourPlus,
    /// Within 3 years of plan's normal retirement age
    /// — § 457(b)(3) special catch-up.
    WithinThreeYearsOfNra,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section457bInput {
    pub plan_type: PlanType,
    pub age_bracket: AgeBracket,
    /// Annual elective deferral amount in cents.
    pub elective_deferral_cents: u64,
    /// Catch-up contribution amount in cents.
    pub catch_up_contribution_cents: u64,
    /// Whether § 457(b) is being stacked with a § 401
    /// (k) or § 403(b) for double deferral strategy.
    pub stacked_with_401k_or_403b: bool,
    /// Aggregate underutilized limitation from prior
    /// years (§ 457(b)(3) special catch-up basis).
    pub underutilized_prior_year_limitation_cents: u64,
    /// Whether participant is separating from service
    /// (key distribution trigger).
    pub separating_from_service: bool,
    /// Whether participant invoking § 72(t) early-
    /// withdrawal penalty (only applies to tax-exempt
    /// or rolled-over amounts).
    pub early_withdrawal_under_59_5: bool,
    /// Whether participant attempting rollover to
    /// another plan type.
    pub rollover_to_other_plan_type: bool,
    /// Whether employer is in financial distress
    /// (Chapter 11 bankruptcy) — credit-risk
    /// disclosure trigger for tax-exempt plans.
    pub employer_financial_distress: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section457bResult {
    pub annual_deferral_limit_cents: u64,
    pub catch_up_limit_cents: u64,
    pub special_three_year_catch_up_cents: u64,
    pub elective_deferral_compliant: bool,
    pub catch_up_compliant: bool,
    pub age_50_catch_up_available: bool,
    pub enhanced_catch_up_available: bool,
    pub special_three_year_catch_up_available: bool,
    pub double_deferral_strategy_available: bool,
    pub section_72t_penalty_applies: bool,
    pub rollover_permitted_to_other_plan: bool,
    pub credit_risk_disclosure_engaged: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section457bInput) -> Section457bResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    // 2026 limits in cents
    let annual_deferral_limit_cents: u64 = 2_450_000;

    let age_50_catch_up_available = matches!(input.plan_type, PlanType::Governmental)
        && matches!(
            input.age_bracket,
            AgeBracket::FiftyToFiftyNine
                | AgeBracket::SixtyToSixtyThree
                | AgeBracket::SixtyFourPlus
        );

    let enhanced_catch_up_available = matches!(input.plan_type, PlanType::Governmental)
        && matches!(input.age_bracket, AgeBracket::SixtyToSixtyThree);

    let special_three_year_catch_up_available =
        matches!(input.age_bracket, AgeBracket::WithinThreeYearsOfNra);

    let catch_up_limit_cents: u64 = if special_three_year_catch_up_available {
        0
    } else if enhanced_catch_up_available {
        1_125_000
    } else if age_50_catch_up_available {
        800_000
    } else {
        0
    };

    let max_special_three_year = annual_deferral_limit_cents.saturating_mul(2);
    let special_three_year_catch_up_cents = if special_three_year_catch_up_available {
        max_special_three_year.min(
            annual_deferral_limit_cents
                .saturating_add(input.underutilized_prior_year_limitation_cents),
        )
    } else {
        0
    };

    let elective_deferral_compliant = input.elective_deferral_cents
        <= if special_three_year_catch_up_available {
            special_three_year_catch_up_cents
        } else {
            annual_deferral_limit_cents
        };

    let catch_up_compliant = if special_three_year_catch_up_available {
        input.catch_up_contribution_cents == 0
    } else {
        input.catch_up_contribution_cents <= catch_up_limit_cents
    };

    let double_deferral_strategy_available = input.stacked_with_401k_or_403b;

    let section_72t_penalty_applies =
        matches!(input.plan_type, PlanType::TaxExempt) && input.early_withdrawal_under_59_5;

    let rollover_permitted_to_other_plan = matches!(input.plan_type, PlanType::Governmental);

    let credit_risk_disclosure_engaged =
        matches!(input.plan_type, PlanType::TaxExempt) && input.employer_financial_distress;

    if !elective_deferral_compliant {
        let applicable_limit = if special_three_year_catch_up_available {
            special_three_year_catch_up_cents
        } else {
            annual_deferral_limit_cents
        };
        failure_reasons.push(format!(
            "26 USC § 457(b)(2) ELECTIVE DEFERRAL LIMIT EXCEEDED — deferral of {} cents exceeds {} cents applicable 2026 limit; excess must be distributed under Treas. Reg. § 1.457-9 by April 15 of following year or risk plan disqualification under § 457(f)",
            input.elective_deferral_cents,
            applicable_limit
        ));
    }

    if special_three_year_catch_up_available && input.catch_up_contribution_cents > 0 {
        failure_reasons.push(
            "26 USC § 457(b)(3) ANTI-STACKING RULE — special 3-year pre-retirement catch-up CANNOT BE COMBINED with age-50 catch-up under § 414(v) OR ages-60-63 enhanced catch-up under § 414(v)(2)(E); participant must choose ONE catch-up mechanism per year".to_string(),
        );
    } else if !catch_up_compliant {
        failure_reasons.push(format!(
            "26 USC § 414(v) CATCH-UP LIMIT EXCEEDED — catch-up of {} cents exceeds applicable 2026 limit of {} cents for {} bracket",
            input.catch_up_contribution_cents,
            catch_up_limit_cents,
            match input.age_bracket {
                AgeBracket::UnderFifty => "under-50 (no catch-up available)",
                AgeBracket::FiftyToFiftyNine => "age 50-59 ($8,000)",
                AgeBracket::SixtyToSixtyThree => "ages 60-63 ($11,250 SECURE 2.0 § 109)",
                AgeBracket::SixtyFourPlus => "age 64+ ($8,000 standard)",
                AgeBracket::WithinThreeYearsOfNra => "within 3 years of NRA (special catch-up only)",
            }
        ));
    }

    if matches!(input.plan_type, PlanType::TaxExempt)
        && matches!(
            input.age_bracket,
            AgeBracket::FiftyToFiftyNine
                | AgeBracket::SixtyToSixtyThree
                | AgeBracket::SixtyFourPlus
        )
        && input.catch_up_contribution_cents > 0
        && !special_three_year_catch_up_available
    {
        failure_reasons.push(
            "26 USC § 414(v)(3) — TAX-EXEMPT § 457(b) plans NOT ELIGIBLE for § 414(v) age-50 catch-up or SECURE 2.0 § 109 enhanced catch-up; only governmental § 457(b) plans qualify; only § 457(b)(3) special 3-year pre-retirement catch-up available to tax-exempt".to_string(),
        );
    }

    if section_72t_penalty_applies {
        failure_reasons.push(
            "26 USC § 72(t) — TAX-EXEMPT § 457(b) early withdrawal before age 59½ subject to 10% additional tax; standard § 72(t)(2) exceptions apply (death + disability + separation from service age 55+ + § 72(t)(2)(A)(iv) substantially equal periodic payments + medical expenses + qualified higher education + first-time homebuyer up to $10K + qualified reservist distributions)".to_string(),
        );
    }

    if matches!(input.plan_type, PlanType::Governmental) && input.early_withdrawal_under_59_5 {
        failure_reasons.push(
            "26 USC § 457 — GOVERNMENTAL § 457(b) NO 10% § 72(t) EARLY-WITHDRAWAL PENALTY on distributions attributable to original § 457(b) contributions; rollovers from § 401(k) or § 403(b) retain source plan's § 72(t) penalty status if withdrawn within early-withdrawal trigger period of source plan".to_string(),
        );
    }

    if input.rollover_to_other_plan_type {
        if rollover_permitted_to_other_plan {
            failure_reasons.push(
                "26 USC § 457(d)(2) — GOVERNMENTAL § 457(b) ROLLOVER PERMITTED to/from § 401(k), § 403(b), § 408 traditional IRAs, § 408A Roth IRAs (after tax inclusion); rolled-over amounts retain source plan's § 72(t) 10% penalty status".to_string(),
            );
        } else {
            failure_reasons.push(
                "26 USC § 457(d)(2) — TAX-EXEMPT § 457(b) ROLLOVER NOT PERMITTED to other plan types; only to another tax-exempt § 457(b) plan within 60-day window; structural difference vs governmental § 457(b)".to_string(),
            );
        }
    }

    if credit_risk_disclosure_engaged {
        failure_reasons.push(
            "TAX-EXEMPT § 457(b) CREDIT RISK — § 501(c) employer in financial distress; UNFUNDED top-hat plan assets are PART OF EMPLOYER'S GENERAL ASSETS subject to claims of all creditors; § 457(b) participant becomes UNSECURED CREDITOR in bankruptcy proceedings; substantial credit risk vs governmental § 457(b) (which holds assets in § 457(g) trust for exclusive benefit of participants)".to_string(),
        );
    }

    if double_deferral_strategy_available {
        failure_reasons.push(
            "26 USC § 402(g)(1) NON-AGGREGATION — § 457(b) elective deferrals are NOT AGGREGATED with § 401(k) / § 403(b) under § 402(g)(1); participant may defer SEPARATE $24,500 (2026) to each plan type for DOUBLE DEFERRAL up to $49,000 in 2026; key advantage for governmental and nonprofit employees with access to both plans".to_string(),
        );
    }

    if input.separating_from_service {
        failure_reasons.push(
            "26 USC § 457(d)(1) DISTRIBUTION TRIGGER — separation from service is most common trigger; participant may elect immediate lump-sum distribution OR installment payments OR rollover (governmental § 457(b) only); governmental § 457(b) imposes NO 10% § 72(t) penalty regardless of age at separation".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "26 USC § 457(b) ELIGIBLE DEFERRED COMPENSATION PLAN — allows state/local government and tax-exempt entity employees to make elective deferrals; contributions excluded from current gross income; growth tax-deferred; distributions taxed as ordinary income at receipt".to_string(),
        "Two structurally distinct § 457(b) plan types: (1) GOVERNMENTAL § 457(b) — state, political subdivision, agency, or instrumentality; assets held in TRUST FOR EXCLUSIVE BENEFIT under § 457(g); NO 10% § 72(t) penalty; rollovers permitted; (2) TAX-EXEMPT § 457(b) — § 501(c) organization; UNFUNDED top-hat plan for select group of management or highly compensated employees; assets remain EMPLOYER'S GENERAL ASSETS subject to creditor claims; SUBSTANTIAL CREDIT RISK; § 72(t) 10% penalty applies; rollovers NOT permitted".to_string(),
        "2026 contribution limits (IRS Notice 2025-67): annual elective deferral § 457(b)(2) $24,500; age 50+ catch-up § 414(v) $8,000 (GOVERNMENTAL ONLY); ages 60-63 SECURE 2.0 § 109 enhanced catch-up § 414(v)(2)(E) $11,250 (GOVERNMENTAL ONLY); § 457(b)(3) special 3-year pre-retirement catch-up = lesser of 2× annual limit $49,000 OR underutilized prior-year limitation".to_string(),
        "26 USC § 457(b)(3) SPECIAL 3-YEAR PRE-RETIREMENT CATCH-UP: (1) available in 3 taxable years immediately preceding NORMAL RETIREMENT AGE specified by plan; (2) maximum = lesser of 2× annual limit ($49,000 for 2026) OR underutilized contribution capacity from prior eligible years; (3) ANTI-STACKING — CANNOT be combined with age-50 catch-up under § 414(v) or ages-60-63 enhanced catch-up; participant must choose ONE catch-up mechanism per year; (4) available to BOTH governmental AND tax-exempt § 457(b) plans (unlike age-50 which is governmental-only)".to_string(),
        "26 USC § 457(g) TRUST REQUIREMENT (Small Business Jobs Protection Act of 1996, Pub. L. 104-188) — effective January 1, 1999, governmental § 457(b) assets must be held in TRUST for EXCLUSIVE BENEFIT of participants and beneficiaries; protects from employer creditors; key distinction from tax-exempt § 457(b) unfunded top-hat plans".to_string(),
        "26 USC § 72(t) early-withdrawal penalty interaction: (1) GOVERNMENTAL § 457(b) — NO 10% penalty (except rollovers from other plan types retain source plan's § 72(t) status); (2) TAX-EXEMPT § 457(b) — § 72(t) 10% penalty APPLIES; standard § 72(t)(2) exceptions apply (death + disability + separation from service age 55+ + substantially equal periodic payments + medical + higher education + first-time homebuyer up to $10K + qualified reservist)".to_string(),
        "26 USC § 402(g)(1) AGGREGATION RULE: (1) § 401(k) + § 403(b) plans aggregated under single $24,500 elective deferral limit; (2) § 457(b) NOT AGGREGATED — SEPARATE $24,500 limit; (3) DOUBLE DEFERRAL STRATEGY — governmental or nonprofit employee with both § 401(k)/§ 403(b) AND § 457(b) can defer UP TO $49,000 in 2026; key advantage of public-sector and nonprofit employment".to_string(),
        "26 USC § 457(d)(1) DISTRIBUTION TRIGGERS: (1) SEPARATION FROM SERVICE — most common trigger; (2) AGE 70½ (now superseded by § 401(a)(9) RMD rules — age 73 for born 1951-1959, age 75 for born 1960+); (3) UNFORESEEABLE EMERGENCY (severe financial hardship from extraordinary unforeseeable circumstances per Treas. Reg. § 1.457-6(c)); (4) PLAN TERMINATION; (5) § 457(d)(9) IN-SERVICE DISTRIBUTION at 59½ (added by SECURE Act 2.0 § 314 for emergency expenses; previously age 70½)".to_string(),
        "26 USC § 457(d)(2) ROLLOVER RULES: (1) GOVERNMENTAL § 457(b) — rollovers PERMITTED to/from § 401(k), § 403(b), § 408 traditional IRAs, § 408A Roth IRAs (after tax inclusion); rolled-over amounts retain source plan's § 72(t) 10% penalty status if withdrawn within early-withdrawal trigger period; (2) TAX-EXEMPT § 457(b) — rollovers NOT PERMITTED to other plan types; only to another tax-exempt § 457(b) within 60-day window; structural difference reflects fundamental funding difference".to_string(),
        "Trader-critical fact patterns: (1) state employee age 40 defers $24,500 § 457(b) + $24,500 § 401(k) = $49,000 DOUBLE DEFERRAL under § 402(g)(1) non-aggregation; (2) state employee age 55 separates from service — governmental § 457(b) immediate distribution without § 72(t) penalty (vs § 401(k) requires age 55 rule or 59½); (3) tax-exempt nonprofit CFO age 62 maxes $24,500 only (no $11,250 enhanced catch-up — § 414(v)(3) ineligibility); (4) state employee age 62 (within 3 years of NRA 65) invokes § 457(b)(3) special 3-year catch-up $49,000; cannot combine with $11,250 enhanced; (5) tax-exempt § 457(b) participant in employer Chapter 11 — UNFUNDED plan assets subject to ALL creditor claims; participant becomes UNSECURED CREDITOR".to_string(),
        "Companion to section_401k (iter 448 § 401(k) cash or deferred arrangements) + section_408 (traditional IRA) + section_408a (Roth IRA) + section_4973 (excess contribution excise) + section_4974 (RMD excise) + section_72t (10% early-withdrawal penalty) + section_162m ($1M public-company exec comp deduction limit)".to_string(),
    ];

    Section457bResult {
        annual_deferral_limit_cents,
        catch_up_limit_cents,
        special_three_year_catch_up_cents,
        elective_deferral_compliant,
        catch_up_compliant,
        age_50_catch_up_available,
        enhanced_catch_up_available,
        special_three_year_catch_up_available,
        double_deferral_strategy_available,
        section_72t_penalty_applies,
        rollover_permitted_to_other_plan,
        credit_risk_disclosure_engaged,
        failure_reasons,
        citation: "26 USC § 457(a)-(g); 26 USC § 457(b); 26 USC § 457(b)(2); 26 USC § 457(b)(3); 26 USC § 457(d); 26 USC § 457(g); 26 USC § 402(g)(1); 26 USC § 72(t); 26 USC § 414(v); 26 USC § 414(v)(2)(E); 26 USC § 414(v)(3); Pub. L. 104-188 (Small Business Jobs Protection Act of 1996); SECURE Act 2.0 of 2022 § 314; SECURE Act 2.0 of 2022 § 109; Pub. L. 117-328; IRS Notice 2025-67 (2026 dollar limitations); Treas. Reg. § 1.457-1 through § 1.457-12; Treas. Reg. § 1.457-6(c) (unforeseeable emergency); Treas. Reg. § 1.457-9 (excess deferrals)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn governmental_age_40_max() -> Section457bInput {
        Section457bInput {
            plan_type: PlanType::Governmental,
            age_bracket: AgeBracket::UnderFifty,
            elective_deferral_cents: 2_450_000,
            catch_up_contribution_cents: 0,
            stacked_with_401k_or_403b: false,
            underutilized_prior_year_limitation_cents: 0,
            separating_from_service: false,
            early_withdrawal_under_59_5: false,
            rollover_to_other_plan_type: false,
            employer_financial_distress: false,
        }
    }

    #[test]
    fn governmental_max_compliant() {
        let r = check(&governmental_age_40_max());
        assert!(r.elective_deferral_compliant);
        assert_eq!(r.annual_deferral_limit_cents, 2_450_000);
    }

    #[test]
    fn over_24500_limit_violation() {
        let mut i = governmental_age_40_max();
        i.elective_deferral_cents = 2_500_000;
        let r = check(&i);
        assert!(!r.elective_deferral_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 457(b)(2)") && f.contains("EXCEEDED")));
    }

    #[test]
    fn governmental_age_50_eligible_for_8000_catch_up() {
        let mut i = governmental_age_40_max();
        i.age_bracket = AgeBracket::FiftyToFiftyNine;
        i.catch_up_contribution_cents = 800_000;
        let r = check(&i);
        assert!(r.age_50_catch_up_available);
        assert_eq!(r.catch_up_limit_cents, 800_000);
    }

    #[test]
    fn governmental_age_60_to_63_enhanced_11250() {
        let mut i = governmental_age_40_max();
        i.age_bracket = AgeBracket::SixtyToSixtyThree;
        i.catch_up_contribution_cents = 1_125_000;
        let r = check(&i);
        assert!(r.enhanced_catch_up_available);
        assert_eq!(r.catch_up_limit_cents, 1_125_000);
    }

    #[test]
    fn governmental_age_64_plus_reverts_to_8000() {
        let mut i = governmental_age_40_max();
        i.age_bracket = AgeBracket::SixtyFourPlus;
        i.catch_up_contribution_cents = 800_000;
        let r = check(&i);
        assert!(!r.enhanced_catch_up_available);
        assert_eq!(r.catch_up_limit_cents, 800_000);
    }

    #[test]
    fn tax_exempt_no_age_50_catch_up_eligibility() {
        let mut i = governmental_age_40_max();
        i.plan_type = PlanType::TaxExempt;
        i.age_bracket = AgeBracket::FiftyToFiftyNine;
        i.catch_up_contribution_cents = 800_000;
        let r = check(&i);
        assert!(!r.age_50_catch_up_available);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 414(v)(3)") && f.contains("TAX-EXEMPT")));
    }

    #[test]
    fn tax_exempt_no_enhanced_catch_up_eligibility() {
        let mut i = governmental_age_40_max();
        i.plan_type = PlanType::TaxExempt;
        i.age_bracket = AgeBracket::SixtyToSixtyThree;
        let r = check(&i);
        assert!(!r.enhanced_catch_up_available);
    }

    #[test]
    fn special_three_year_catch_up_doubles_limit() {
        let mut i = governmental_age_40_max();
        i.age_bracket = AgeBracket::WithinThreeYearsOfNra;
        i.underutilized_prior_year_limitation_cents = 4_900_000;
        i.elective_deferral_cents = 4_900_000;
        let r = check(&i);
        assert!(r.special_three_year_catch_up_available);
        assert_eq!(r.special_three_year_catch_up_cents, 4_900_000);
        assert!(r.elective_deferral_compliant);
    }

    #[test]
    fn special_three_year_catch_up_capped_at_2x() {
        let mut i = governmental_age_40_max();
        i.age_bracket = AgeBracket::WithinThreeYearsOfNra;
        i.underutilized_prior_year_limitation_cents = 100_000_000;
        let r = check(&i);
        assert_eq!(r.special_three_year_catch_up_cents, 4_900_000);
    }

    #[test]
    fn special_three_year_anti_stacking_with_catch_up_violation() {
        let mut i = governmental_age_40_max();
        i.age_bracket = AgeBracket::WithinThreeYearsOfNra;
        i.underutilized_prior_year_limitation_cents = 4_900_000;
        i.catch_up_contribution_cents = 800_000;
        let r = check(&i);
        assert!(!r.catch_up_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 457(b)(3) ANTI-STACKING")
                && f.contains("ONE catch-up mechanism per year")));
    }

    #[test]
    fn governmental_no_72t_penalty() {
        let mut i = governmental_age_40_max();
        i.early_withdrawal_under_59_5 = true;
        let r = check(&i);
        assert!(!r.section_72t_penalty_applies);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("GOVERNMENTAL § 457(b) NO 10% § 72(t)")));
    }

    #[test]
    fn tax_exempt_72t_penalty_applies() {
        let mut i = governmental_age_40_max();
        i.plan_type = PlanType::TaxExempt;
        i.early_withdrawal_under_59_5 = true;
        let r = check(&i);
        assert!(r.section_72t_penalty_applies);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 72(t)")
            && f.contains("TAX-EXEMPT")
            && f.contains("10% additional tax")));
    }

    #[test]
    fn governmental_rollover_permitted() {
        let mut i = governmental_age_40_max();
        i.rollover_to_other_plan_type = true;
        let r = check(&i);
        assert!(r.rollover_permitted_to_other_plan);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 457(d)(2)")
            && f.contains("GOVERNMENTAL")
            && f.contains("ROLLOVER PERMITTED")));
    }

    #[test]
    fn tax_exempt_rollover_not_permitted() {
        let mut i = governmental_age_40_max();
        i.plan_type = PlanType::TaxExempt;
        i.rollover_to_other_plan_type = true;
        let r = check(&i);
        assert!(!r.rollover_permitted_to_other_plan);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 457(d)(2)")
            && f.contains("TAX-EXEMPT")
            && f.contains("NOT PERMITTED")));
    }

    #[test]
    fn double_deferral_strategy_disclosed() {
        let mut i = governmental_age_40_max();
        i.stacked_with_401k_or_403b = true;
        let r = check(&i);
        assert!(r.double_deferral_strategy_available);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 402(g)(1) NON-AGGREGATION")
                && f.contains("DOUBLE DEFERRAL")
                && f.contains("$49,000")));
    }

    #[test]
    fn tax_exempt_employer_distress_credit_risk_engaged() {
        let mut i = governmental_age_40_max();
        i.plan_type = PlanType::TaxExempt;
        i.employer_financial_distress = true;
        let r = check(&i);
        assert!(r.credit_risk_disclosure_engaged);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("TAX-EXEMPT § 457(b) CREDIT RISK")
                && f.contains("UNSECURED CREDITOR")));
    }

    #[test]
    fn governmental_employer_distress_no_credit_risk() {
        let mut i = governmental_age_40_max();
        i.employer_financial_distress = true;
        let r = check(&i);
        assert!(!r.credit_risk_disclosure_engaged);
    }

    #[test]
    fn separation_from_service_disclosed() {
        let mut i = governmental_age_40_max();
        i.separating_from_service = true;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 457(d)(1) DISTRIBUTION TRIGGER")
                && f.contains("NO 10% § 72(t) penalty regardless of age")));
    }

    #[test]
    fn age_bracket_truth_table_five_cells_governmental() {
        let cases = [
            (AgeBracket::UnderFifty, 0u64),
            (AgeBracket::FiftyToFiftyNine, 800_000u64),
            (AgeBracket::SixtyToSixtyThree, 1_125_000u64),
            (AgeBracket::SixtyFourPlus, 800_000u64),
            (AgeBracket::WithinThreeYearsOfNra, 0u64),
        ];
        for (bracket, exp_catch_up_limit) in cases {
            let mut i = governmental_age_40_max();
            i.age_bracket = bracket;
            let r = check(&i);
            assert_eq!(
                r.catch_up_limit_cents, exp_catch_up_limit,
                "bracket={:?}",
                bracket
            );
        }
    }

    #[test]
    fn tax_exempt_uniquely_72t_applies_invariant() {
        let mut tax_exempt = governmental_age_40_max();
        tax_exempt.plan_type = PlanType::TaxExempt;
        tax_exempt.early_withdrawal_under_59_5 = true;
        let r_te = check(&tax_exempt);
        assert!(r_te.section_72t_penalty_applies);

        let mut governmental = governmental_age_40_max();
        governmental.early_withdrawal_under_59_5 = true;
        let r_gov = check(&governmental);
        assert!(!r_gov.section_72t_penalty_applies);
    }

    #[test]
    fn special_three_year_uniquely_doubles_limit_invariant() {
        let mut special = governmental_age_40_max();
        special.age_bracket = AgeBracket::WithinThreeYearsOfNra;
        special.underutilized_prior_year_limitation_cents = 2_450_000;
        let r_special = check(&special);
        assert!(r_special.special_three_year_catch_up_available);
        assert!(r_special.special_three_year_catch_up_cents > 0);

        let mut normal = governmental_age_40_max();
        normal.age_bracket = AgeBracket::FiftyToFiftyNine;
        let r_normal = check(&normal);
        assert!(!r_normal.special_three_year_catch_up_available);
        assert_eq!(r_normal.special_three_year_catch_up_cents, 0);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&governmental_age_40_max());
        assert!(r.citation.contains("§ 457(a)-(g)"));
        assert!(r.citation.contains("§ 457(b)(2)"));
        assert!(r.citation.contains("§ 457(b)(3)"));
        assert!(r.citation.contains("§ 457(d)"));
        assert!(r.citation.contains("§ 457(g)"));
        assert!(r.citation.contains("§ 402(g)(1)"));
        assert!(r.citation.contains("§ 72(t)"));
        assert!(r.citation.contains("§ 414(v)(2)(E)"));
        assert!(r.citation.contains("§ 414(v)(3)"));
        assert!(r.citation.contains("Pub. L. 104-188"));
        assert!(r.citation.contains("SECURE Act 2.0 of 2022 § 314"));
        assert!(r.citation.contains("SECURE Act 2.0 of 2022 § 109"));
        assert!(r.citation.contains("Pub. L. 117-328"));
        assert!(r.citation.contains("IRS Notice 2025-67"));
        assert!(r.citation.contains("Treas. Reg. § 1.457-1"));
    }

    #[test]
    fn note_pins_eligible_deferred_compensation_plan() {
        let r = check(&governmental_age_40_max());
        assert!(r.notes.iter().any(
            |n| n.contains("§ 457(b) ELIGIBLE DEFERRED COMPENSATION PLAN")
                && n.contains("state/local government")
                && n.contains("tax-exempt entity")
        ));
    }

    #[test]
    fn note_pins_two_plan_types_governmental_vs_tax_exempt() {
        let r = check(&governmental_age_40_max());
        assert!(r.notes.iter().any(|n| n
            .contains("Two structurally distinct § 457(b) plan types")
            && n.contains("GOVERNMENTAL")
            && n.contains("§ 457(g)")
            && n.contains("TAX-EXEMPT")
            && n.contains("top-hat")
            && n.contains("CREDIT RISK")));
    }

    #[test]
    fn note_pins_2026_limits() {
        let r = check(&governmental_age_40_max());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("2026 contribution limits")
                && n.contains("IRS Notice 2025-67")
                && n.contains("$24,500")
                && n.contains("$8,000")
                && n.contains("$11,250")
                && n.contains("$49,000")));
    }

    #[test]
    fn note_pins_special_three_year_anti_stacking() {
        let r = check(&governmental_age_40_max());
        assert!(r.notes.iter().any(|n| n
            .contains("§ 457(b)(3) SPECIAL 3-YEAR PRE-RETIREMENT CATCH-UP")
            && n.contains("ANTI-STACKING")
            && n.contains("ONE catch-up mechanism per year")
            && n.contains("BOTH governmental AND tax-exempt")));
    }

    #[test]
    fn note_pins_section_457g_trust_requirement() {
        let r = check(&governmental_age_40_max());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 457(g) TRUST REQUIREMENT")
                && n.contains("Pub. L. 104-188")
                && n.contains("January 1, 1999")));
    }

    #[test]
    fn note_pins_section_72t_interaction() {
        let r = check(&governmental_age_40_max());
        assert!(r.notes.iter().any(
            |n| n.contains("§ 72(t) early-withdrawal penalty interaction")
                && n.contains("GOVERNMENTAL")
                && n.contains("TAX-EXEMPT")
                && n.contains("first-time homebuyer up to $10K")
        ));
    }

    #[test]
    fn note_pins_section_402g1_double_deferral() {
        let r = check(&governmental_age_40_max());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 402(g)(1) AGGREGATION RULE")
                && n.contains("NOT AGGREGATED")
                && n.contains("DOUBLE DEFERRAL STRATEGY")
                && n.contains("$49,000")));
    }

    #[test]
    fn note_pins_section_457d1_distribution_triggers() {
        let r = check(&governmental_age_40_max());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 457(d)(1) DISTRIBUTION TRIGGERS")
                && n.contains("SEPARATION FROM SERVICE")
                && n.contains("UNFORESEEABLE EMERGENCY")
                && n.contains("SECURE Act 2.0 § 314")));
    }

    #[test]
    fn note_pins_section_457d2_rollover_rules() {
        let r = check(&governmental_age_40_max());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 457(d)(2) ROLLOVER RULES")
                && n.contains("GOVERNMENTAL § 457(b) — rollovers PERMITTED")
                && n.contains("TAX-EXEMPT § 457(b) — rollovers NOT PERMITTED")
                && n.contains("60-day window")));
    }

    #[test]
    fn note_pins_trader_fact_patterns_five() {
        let r = check(&governmental_age_40_max());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Trader-critical fact patterns")
                && n.contains("$49,000 DOUBLE DEFERRAL")
                && n.contains("§ 414(v)(3) ineligibility")
                && n.contains("special 3-year catch-up")
                && n.contains("UNSECURED CREDITOR")));
    }

    #[test]
    fn note_pins_companion_modules() {
        let r = check(&governmental_age_40_max());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Companion to section_401k")
                && n.contains("section_408")
                && n.contains("section_4974")
                && n.contains("section_162m")));
    }

    #[test]
    fn defensive_overflow_saturating() {
        let mut i = governmental_age_40_max();
        i.underutilized_prior_year_limitation_cents = u64::MAX;
        let r = check(&i);
        let _ = r.special_three_year_catch_up_cents;
    }
}
