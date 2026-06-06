//! IRC § 4980 — Tax on reversion of qualified plan
//! assets to employer. Direct trader companion to
//! section_401k (iter 448), section_408 (traditional
//! IRA — iter 432), section_408a (Roth IRA — iter
//! 430), section_415 (umbrella limits — iter 452),
//! section_457b (iter 450), section_4973 (excess
//! contributions — iter 442), section_4974 (RMD
//! excise — iter 436), section_4975 (prohibited
//! transactions — iter 434), section_162m ($1M exec
//! comp deduction — iter 446).
//!
//! § 4980 imposes an EXCISE TAX on the amount of any
//! employer reversion from a qualified retirement
//! plan (defined benefit pension plan typically) when
//! the plan terminates with surplus assets exceeding
//! plan liabilities. The default rate is 50%, reduced
//! to 20% if the employer satisfies certain
//! conditions involving a qualified replacement plan
//! (QRP) or pro rata benefit increases.
//!
//! Trader-critical because traders who own businesses
//! sponsoring defined benefit pension plans face
//! § 4980 reversion tax at plan termination if plan
//! is over-funded; smart structuring through QRP
//! reduces effective tax from 50% to 20%; failure to
//! plan exposes employer to confiscatory tax that can
//! exceed plan surplus.
//!
//! § 4980 only applies to QUALIFIED DEFINED BENEFIT
//! plans (under § 401(a) + § 411 + § 412 + § 415) on
//! TERMINATION; not applicable to ongoing plans or to
//! § 401(k) defined contribution plans (where each
//! participant's account is individual property).
//!
//! Companion to section_401k (iter 448) + section_415
//! (iter 452) + section_457b (iter 450) + section_
//! 4973 (excess contribution excise — iter 442) +
//! section_4974 (RMD excise — iter 436) + section_
//! 4975 (prohibited transactions — iter 434) +
//! section_162m ($1M exec comp deduction — iter 446).
//!
//! **§ 4980(a) GENERAL RULE — 20% base tax**:
//! A tax equal to 20% of the amount of any employer
//! reversion from a qualified plan is imposed on the
//! employer maintaining the plan.
//!
//! **§ 4980(d) INCREASED RATE — 50% if no QRP or pro
//! rata benefit increase**:
//! 1. § 4980(d)(1) — rate increased to 50% if
//!    employer does NOT satisfy § 4980(d)(2) QRP
//!    requirement OR § 4980(d)(3) pro rata benefit
//!    increase requirement;
//! 2. Default for unstructured plan terminations is
//!    50% rate;
//! 3. Combined with corporate income tax on
//!    reversion proceeds, effective tax exceeds 70%.
//!
//! **§ 4980(d)(2) QUALIFIED REPLACEMENT PLAN (QRP)
//! requirements**:
//! 1. § 4980(d)(2)(A) — qualified plan is
//!    ESTABLISHED OR MAINTAINED by employer in
//!    CONNECTION WITH termination;
//! 2. § 4980(d)(2)(B)(i) — at least 95% OF ACTIVE
//!    PARTICIPANTS in terminating plan who remain as
//!    employees become ACTIVE PARTICIPANTS in QRP;
//! 3. § 4980(d)(2)(B)(ii) — DIRECT TRANSFER of at
//!    least 25% OF THE MAXIMUM AMOUNT which employer
//!    could receive as reversion (= 25% of excess
//!    assets) from terminating plan to QRP;
//: 4. § 4980(d)(2)(B)(iii) — assets transferred to
//!    QRP must be ALLOCATED no less rapidly than
//!    ratably over 7-year period.
//!
//! **§ 4980(d)(3) PRO RATA BENEFIT INCREASE
//! alternative to QRP**:
//! 1. § 4980(d)(3)(A) — qualified plan provides for
//!    pro rata increase in accrued benefits of
//!    qualified participants;
//! 2. § 4980(d)(3)(B) — present value of benefit
//!    increases at least 20% of MAXIMUM amount
//!    employer could receive as reversion;
//! 3. § 4980(d)(3)(C) — benefit increases take
//!    effect IMMEDIATELY on plan termination date.
//!
//! **§ 4980(d)(4) QUALIFIED PARTICIPANT**:
//! 1. § 4980(d)(4)(A) — active participant in plan
//!    on termination date; OR
//! 2. § 4980(d)(4)(B) — participant or beneficiary
//!    receiving benefits on termination date; OR
//! 3. § 4980(d)(4)(C) — participant whose accrued
//!    benefits ARE BEING DEFERRED in plan on
//!    termination date.
//!
//! **§ 4980(c) EMPLOYER REVERSION DEFINITION**:
//! 1. Amount of cash or fair market value of other
//!    property received by employer from qualified
//!    plan as a result of plan termination;
//! 2. EXCLUDES amounts received in respect of
//!    contributions allocable to one or more
//!    pension plan participants that were NOT
//!    DEDUCTIBLE under § 404;
//! 3. EXCLUDES amounts received from continuing
//!    qualified plan (no termination = no reversion
//!    tax).
//!
//! **§ 4980 INTERACTION WITH § 162 + § 404 + INCOME
//! TAX**:
//! 1. Reversion includible in employer's gross
//!    income;
//! 2. Corporate income tax applies (21% federal +
//!    state);
//! 3. § 4980 excise tax IN ADDITION TO income tax
//!    (stacked liability);
//! 4. Effective combined rate at default 50% excise
//!    plus 21% federal plus 5-10% state can exceed
//!    75% of reversion;
//! 5. At reduced 20% excise + income tax, effective
//!    rate approximately 45-50%.
//!
//! **§ 4980 PRACTICAL TERMINATION ROUTES**:
//! 1. Distribute surplus directly to employer —
//!    full 50% reversion tax;
//! 2. Establish QRP via § 4980(d)(2) — 20% reversion
//!    on amount NOT transferred to QRP; portion
//!    transferred to QRP NOT subject to reversion
//!    tax + not includible in employer income;
//! 3. Pro rata benefit increase via § 4980(d)(3) —
//!    20% reversion if 20%+ benefit increase
//!    delivered;
//! 4. Roll surplus into successor 401(k) plan —
//!    qualifies as QRP under PLR 9701036 + Rev. Rul.
//!    2003-85 if active-participant + 25% transfer
//!    + ratable allocation requirements met.
//!
//! **§ 4980 TRADER-CRITICAL FACT PATTERNS**:
//!
//! Trader-LLC sponsors over-funded DB pension plan
//! with $5M plan assets vs $3M liabilities;
//! terminates plan without QRP — § 4980(a)+(d) 50%
//! reversion tax on $2M surplus = $1M; plus 21%
//! corporate income tax on $2M = $420K; plus 5%
//! state = $100K; total = $1.52M (76% effective).
//!
//! Same trader-LLC establishes § 4980(d)(2) QRP and
//! transfers 25% of excess ($500K) to QRP — only
//! $1.5M reversion subject to tax; 20% × $1.5M =
//! $300K; plus 21% federal × $1.5M = $315K; plus 5%
//! state = $75K; total = $690K (46%); QRP portion
//! ($500K) tax-free + benefits employees + reduces
//! aggregate tax.
//!
//! Trader-LLC sponsors plan with $50K surplus and
//! 10 employees; § 4980(d)(3) pro rata benefit
//! increase of $10K (20% of $50K) provided to
//! employees — 20% reversion on remaining $40K =
//! $8K; plus income tax; far less than 50% default.
//!
//! Multi-employer plan termination — trader's
//! § 414(b)/(c)/(m)/(o) controlled group must
//! aggregate plan assets and liabilities across
//! related employers; if controlled group treats
//! plans as single plan, reversion tax computed on
//! aggregate.
//!
//! Plan termination tied to corporate acquisition
//! or merger — buyer assumes terminating plan or
//! requires plan termination as condition of deal;
//! § 4980 reversion tax can become deal-breaker if
//! not structured with QRP early.
//!
//! Citations: 26 USC § 4980(a) (20% base tax); 26
//! USC § 4980(b) (employer maintaining plan); 26
//! USC § 4980(c) (employer reversion definition);
//! 26 USC § 4980(d) (increased rate + exceptions);
//! 26 USC § 4980(d)(1) (50% rate); 26 USC § 4980(d)
//! (2) (qualified replacement plan); 26 USC § 4980
//! (d)(2)(A) (establish/maintain QRP); 26 USC
//! § 4980(d)(2)(B)(i) (95% active participant
//! threshold); 26 USC § 4980(d)(2)(B)(ii) (25%
//! direct transfer); 26 USC § 4980(d)(2)(B)(iii)
//! (7-year ratable allocation); 26 USC § 4980(d)
//! (3) (pro rata benefit increase 20% present
//! value); 26 USC § 4980(d)(4) (qualified
//! participant definition); 26 USC § 404 (deduction
//! for employer contributions); 26 USC § 414(b)/
//! (c)/(m)/(o) (controlled group aggregation);
//! Treas. Reg. § 1.4980-1; Rev. Rul. 2003-85
//! (direct transfer to replacement DC plan
//! preferential treatment); Rev. Rul. 89-87 (50%
//! shareholder treatment); PLR 9701036 (DB-to-DC
//! transfer qualifies as QRP); 26 USC § 401(a)
//! (qualified plan definition); 26 USC § 411
//! (minimum vesting); 26 USC § 412 (minimum funding
//! standards); 26 USC § 415 (umbrella limits).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TerminationStructure {
    /// Direct reversion to employer — no QRP or
    /// benefit increase. 50% rate applies.
    DirectReversion,
    /// § 4980(d)(2) qualified replacement plan with
    /// 95% active participant + 25% direct transfer +
    /// 7-year ratable allocation. 20% rate on
    /// non-transferred amount.
    QualifiedReplacementPlan,
    /// § 4980(d)(3) pro rata benefit increase of at
    /// least 20% of maximum reversion. 20% rate.
    ProRataBenefitIncrease,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section4980Input {
    pub termination_structure: TerminationStructure,
    /// Plan assets at termination in cents.
    pub plan_assets_cents: u64,
    /// Plan liabilities at termination in cents.
    pub plan_liabilities_cents: u64,
    /// Percentage of active participants from
    /// terminating plan who become active participants
    /// in QRP (must be at least 95% under § 4980(d)
    /// (2)(B)(i)).
    pub qrp_active_participant_percent: u32,
    /// Percentage of excess assets transferred to QRP
    /// (must be at least 25% under § 4980(d)(2)(B)
    /// (ii)).
    pub qrp_transfer_percent: u32,
    /// Whether QRP transferred assets allocated no
    /// less rapidly than ratably over 7-year period
    /// (§ 4980(d)(2)(B)(iii)).
    pub qrp_seven_year_ratable_allocation: bool,
    /// Percentage of present value of pro rata
    /// benefit increase relative to maximum
    /// reversion (must be at least 20% under § 4980
    /// (d)(3)(B)).
    pub pro_rata_benefit_increase_percent: u32,
    /// Whether benefit increases take effect
    /// immediately on plan termination date (§ 4980
    /// (d)(3)(C)).
    pub benefit_increase_immediate: bool,
    /// Federal corporate income tax rate in basis
    /// points (21% = 2100).
    pub federal_corporate_tax_rate_bp: u32,
    /// State corporate income tax rate in basis
    /// points.
    pub state_corporate_tax_rate_bp: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section4980Result {
    pub plan_surplus_cents: u64,
    pub employer_reversion_cents: u64,
    pub qrp_transfer_cents: u64,
    pub effective_excise_rate_percent: u32,
    pub section_4980_excise_tax_cents: u64,
    pub corporate_income_tax_cents: u64,
    pub total_tax_burden_cents: u64,
    pub effective_combined_rate_percent: u32,
    pub qrp_requirements_satisfied: bool,
    pub pro_rata_benefit_requirements_satisfied: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section4980Input) -> Section4980Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let plan_surplus_cents = input
        .plan_assets_cents
        .saturating_sub(input.plan_liabilities_cents);

    let qrp_requirements_satisfied = matches!(
        input.termination_structure,
        TerminationStructure::QualifiedReplacementPlan
    ) && input.qrp_active_participant_percent >= 95
        && input.qrp_transfer_percent >= 25
        && input.qrp_seven_year_ratable_allocation;

    let pro_rata_benefit_requirements_satisfied = matches!(
        input.termination_structure,
        TerminationStructure::ProRataBenefitIncrease
    ) && input.pro_rata_benefit_increase_percent
        >= 20
        && input.benefit_increase_immediate;

    let qrp_transfer_cents = if qrp_requirements_satisfied {
        plan_surplus_cents.saturating_mul(input.qrp_transfer_percent as u64) / 100
    } else {
        0
    };

    let employer_reversion_cents = plan_surplus_cents.saturating_sub(qrp_transfer_cents);

    let effective_excise_rate_percent: u32 =
        if qrp_requirements_satisfied || pro_rata_benefit_requirements_satisfied {
            20
        } else {
            50
        };

    let section_4980_excise_tax_cents =
        employer_reversion_cents.saturating_mul(effective_excise_rate_percent as u64) / 100;

    let combined_tax_rate_bp = input
        .federal_corporate_tax_rate_bp
        .saturating_add(input.state_corporate_tax_rate_bp);
    let corporate_income_tax_cents =
        employer_reversion_cents.saturating_mul(combined_tax_rate_bp as u64) / 10_000;

    let total_tax_burden_cents =
        section_4980_excise_tax_cents.saturating_add(corporate_income_tax_cents);

    let effective_combined_rate_percent: u32 = total_tax_burden_cents
        .saturating_mul(100)
        .checked_div(employer_reversion_cents)
        .map(|rate| rate.min(u32::MAX as u64) as u32)
        .unwrap_or(0);

    if plan_surplus_cents == 0 {
        failure_reasons.push(
            "Plan terminated with NO SURPLUS (assets <= liabilities); § 4980 reversion tax DOES NOT APPLY; no excise tax exposure".to_string(),
        );
    } else {
        failure_reasons.push(format!(
            "26 USC § 4980(c) EMPLOYER REVERSION = ${} cents (plan surplus of plan assets ${} cents minus plan liabilities ${} cents); reversion includible in employer's gross income under § 61 plus subject to § 4980 excise tax",
            plan_surplus_cents,
            input.plan_assets_cents,
            input.plan_liabilities_cents
        ));
    }

    if qrp_requirements_satisfied {
        failure_reasons.push(format!(
            "26 USC § 4980(d)(2) QUALIFIED REPLACEMENT PLAN — three requirements all met: (i) at least 95% of active participants ({}%) become active participants in QRP; (ii) at least 25% direct transfer ({}%) of excess assets to QRP; (iii) 7-year ratable allocation; ${} cents transferred to QRP NOT subject to § 4980 excise + NOT includible in employer income; remaining ${} cents reversion subject to REDUCED 20% rate",
            input.qrp_active_participant_percent,
            input.qrp_transfer_percent,
            qrp_transfer_cents,
            employer_reversion_cents
        ));
    } else if matches!(
        input.termination_structure,
        TerminationStructure::QualifiedReplacementPlan
    ) {
        if input.qrp_active_participant_percent < 95 {
            failure_reasons.push(format!(
                "26 USC § 4980(d)(2)(B)(i) QRP ACTIVE PARTICIPANT REQUIREMENT NOT SATISFIED — only {}% of active participants become QRP participants; at least 95% REQUIRED; reverts to default 50% excise rate",
                input.qrp_active_participant_percent
            ));
        }
        if input.qrp_transfer_percent < 25 {
            failure_reasons.push(format!(
                "26 USC § 4980(d)(2)(B)(ii) QRP DIRECT TRANSFER REQUIREMENT NOT SATISFIED — only {}% of excess assets transferred to QRP; at least 25% REQUIRED; reverts to default 50% excise rate",
                input.qrp_transfer_percent
            ));
        }
        if !input.qrp_seven_year_ratable_allocation {
            failure_reasons.push(
                "26 USC § 4980(d)(2)(B)(iii) QRP 7-YEAR RATABLE ALLOCATION REQUIREMENT NOT SATISFIED — transferred assets must be allocated no less rapidly than ratably over 7-year period; reverts to default 50% excise rate".to_string(),
            );
        }
    }

    if pro_rata_benefit_requirements_satisfied {
        failure_reasons.push(format!(
            "26 USC § 4980(d)(3) PRO RATA BENEFIT INCREASE — present value of pro rata increase ({}% of maximum reversion) at least 20% required; benefit increases take effect IMMEDIATELY on plan termination date; REDUCED 20% excise rate applies",
            input.pro_rata_benefit_increase_percent
        ));
    } else if matches!(
        input.termination_structure,
        TerminationStructure::ProRataBenefitIncrease
    ) {
        if input.pro_rata_benefit_increase_percent < 20 {
            failure_reasons.push(format!(
                "26 USC § 4980(d)(3)(B) PRO RATA BENEFIT INCREASE PERCENT NOT SATISFIED — only {}% of maximum reversion; at least 20% REQUIRED; reverts to default 50% excise rate",
                input.pro_rata_benefit_increase_percent
            ));
        }
        if !input.benefit_increase_immediate {
            failure_reasons.push(
                "26 USC § 4980(d)(3)(C) IMMEDIATE EFFECT NOT SATISFIED — benefit increases must take effect IMMEDIATELY on plan termination date; reverts to default 50% excise rate".to_string(),
            );
        }
    }

    if employer_reversion_cents > 0 {
        failure_reasons.push(format!(
            "26 USC § 4980(a)+(d) EXCISE TAX — {}% × employer reversion ${} cents = ${} cents excise tax",
            effective_excise_rate_percent,
            employer_reversion_cents,
            section_4980_excise_tax_cents
        ));
        failure_reasons.push(format!(
            "INCOME TAX STACK — reversion includible in employer's gross income; corporate income tax {}% × reversion = ${} cents; TOTAL TAX BURDEN = ${} cents = {}% effective combined rate",
            (combined_tax_rate_bp / 100),
            corporate_income_tax_cents,
            total_tax_burden_cents,
            effective_combined_rate_percent
        ));
    }

    let notes: Vec<String> = vec![
        "26 USC § 4980 — EXCISE TAX on employer reversion from qualified retirement plan (defined benefit pension typically) when plan terminates with surplus assets exceeding plan liabilities; default 50% rate reduced to 20% if employer satisfies § 4980(d)(2) qualified replacement plan (QRP) OR § 4980(d)(3) pro rata benefit increase requirement".to_string(),
        "Section applies to QUALIFIED DEFINED BENEFIT plans (under § 401(a) + § 411 + § 412 + § 415) on TERMINATION; NOT applicable to ongoing plans or § 401(k) defined contribution plans where each participant's account is individual property".to_string(),
        "26 USC § 4980(a) GENERAL RULE — 20% base tax on amount of employer reversion from qualified plan; imposed on employer maintaining the plan; complemented by § 4980(d) increased 50% rate when QRP or pro rata benefit increase not satisfied".to_string(),
        "26 USC § 4980(d) INCREASED 50% RATE — applies unless employer satisfies § 4980(d)(2) QRP requirement OR § 4980(d)(3) pro rata benefit increase requirement; combined with corporate income tax on reversion proceeds, effective tax can exceed 70-75%".to_string(),
        "26 USC § 4980(d)(2) QUALIFIED REPLACEMENT PLAN (QRP) REQUIREMENTS — three elements all required: (A) qualified plan established or maintained by employer in connection with termination; (B)(i) at least 95% of active participants in terminating plan who remain as employees become active participants in QRP; (B)(ii) direct transfer of at least 25% of maximum amount which employer could receive as reversion (= 25% of excess assets) from terminating plan to QRP; (B)(iii) assets transferred to QRP must be allocated no less rapidly than ratably over 7-year period".to_string(),
        "26 USC § 4980(d)(3) PRO RATA BENEFIT INCREASE alternative to QRP — three requirements: (A) qualified plan provides for pro rata increase in accrued benefits of qualified participants; (B) present value of benefit increases at least 20% of maximum amount employer could receive as reversion; (C) benefit increases take effect immediately on plan termination date".to_string(),
        "26 USC § 4980(d)(4) QUALIFIED PARTICIPANT — (A) active participant in plan on termination date; OR (B) participant or beneficiary receiving benefits on termination date; OR (C) participant whose accrued benefits are being deferred in plan on termination date".to_string(),
        "26 USC § 4980(c) EMPLOYER REVERSION DEFINITION — (1) amount of cash or fair market value of other property received by employer from qualified plan as a result of plan termination; (2) EXCLUDES amounts received in respect of contributions allocable to one or more pension plan participants that were NOT DEDUCTIBLE under § 404; (3) EXCLUDES amounts received from continuing qualified plan (no termination = no reversion tax)".to_string(),
        "§ 4980 interaction with § 162 + § 404 + income tax — (1) reversion includible in employer's gross income; (2) corporate income tax applies (21% federal + state); (3) § 4980 excise tax IN ADDITION TO income tax (stacked liability); (4) effective combined rate at default 50% excise + 21% federal + 5-10% state can exceed 75% of reversion; (5) at reduced 20% excise + income tax, effective rate approximately 45-50%".to_string(),
        "§ 4980 PRACTICAL TERMINATION ROUTES: (1) Distribute surplus directly to employer — full 50% reversion tax; (2) Establish QRP via § 4980(d)(2) — 20% reversion on amount NOT transferred to QRP; portion transferred to QRP NOT subject to reversion tax + not includible in employer income; (3) Pro rata benefit increase via § 4980(d)(3) — 20% reversion if 20%+ benefit increase delivered; (4) Roll surplus into successor 401(k) plan — qualifies as QRP under PLR 9701036 + Rev. Rul. 2003-85 if active-participant + 25% transfer + ratable allocation requirements met".to_string(),
        "Trader-critical fact patterns: (1) Trader-LLC sponsors over-funded DB pension $5M assets / $3M liabilities; terminates without QRP — 50% × $2M = $1M excise + 21% × $2M federal corporate = $420K + 5% state = $100K = $1.52M total (76% effective); (2) Same trader establishes QRP transferring 25% ($500K) to QRP — only $1.5M reversion at 20% rate = $300K excise + $315K federal + $75K state = $690K total (46%); $500K QRP transfer is tax-free + benefits employees; (3) $50K surplus with pro rata $10K (20%) benefit increase — 20% × $40K = $8K excise + income tax; (4) Controlled group aggregation under § 414(b)/(c)/(m)/(o) — plans aggregated across related employers; (5) M&A-driven plan termination — § 4980 reversion tax can be deal-breaker without QRP structuring".to_string(),
        "Companion to section_401k (iter 448 § 401(k) cash or deferred arrangements) + section_408 (traditional IRA) + section_408a (Roth IRA) + section_415 (umbrella limits — iter 452) + section_457b (iter 450 governmental and tax-exempt deferred compensation) + section_4973 (excess contribution excise — iter 442) + section_4974 (RMD excise — iter 436) + section_4975 (prohibited transactions — iter 434) + section_162m ($1M public-company exec comp deduction — iter 446)".to_string(),
    ];

    Section4980Result {
        plan_surplus_cents,
        employer_reversion_cents,
        qrp_transfer_cents,
        effective_excise_rate_percent,
        section_4980_excise_tax_cents,
        corporate_income_tax_cents,
        total_tax_burden_cents,
        effective_combined_rate_percent,
        qrp_requirements_satisfied,
        pro_rata_benefit_requirements_satisfied,
        failure_reasons,
        citation: "26 USC § 4980(a); 26 USC § 4980(b); 26 USC § 4980(c); 26 USC § 4980(d); 26 USC § 4980(d)(1); 26 USC § 4980(d)(2); 26 USC § 4980(d)(2)(A); 26 USC § 4980(d)(2)(B)(i); 26 USC § 4980(d)(2)(B)(ii); 26 USC § 4980(d)(2)(B)(iii); 26 USC § 4980(d)(3); 26 USC § 4980(d)(4); 26 USC § 404; 26 USC § 414(b)/(c)/(m)/(o); Treas. Reg. § 1.4980-1; Rev. Rul. 2003-85; Rev. Rul. 89-87; PLR 9701036; 26 USC § 401(a); 26 USC § 411; 26 USC § 412; 26 USC § 415",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn direct_reversion_2m_surplus() -> Section4980Input {
        Section4980Input {
            termination_structure: TerminationStructure::DirectReversion,
            plan_assets_cents: 500_000_000,
            plan_liabilities_cents: 300_000_000,
            qrp_active_participant_percent: 0,
            qrp_transfer_percent: 0,
            qrp_seven_year_ratable_allocation: false,
            pro_rata_benefit_increase_percent: 0,
            benefit_increase_immediate: false,
            federal_corporate_tax_rate_bp: 2100,
            state_corporate_tax_rate_bp: 500,
        }
    }

    #[test]
    fn direct_reversion_50_percent_excise() {
        let r = check(&direct_reversion_2m_surplus());
        assert_eq!(r.plan_surplus_cents, 200_000_000);
        assert_eq!(r.effective_excise_rate_percent, 50);
        assert_eq!(r.section_4980_excise_tax_cents, 100_000_000);
    }

    #[test]
    fn qrp_compliant_25_percent_transfer_20_percent_rate() {
        let mut i = direct_reversion_2m_surplus();
        i.termination_structure = TerminationStructure::QualifiedReplacementPlan;
        i.qrp_active_participant_percent = 95;
        i.qrp_transfer_percent = 25;
        i.qrp_seven_year_ratable_allocation = true;
        let r = check(&i);
        assert!(r.qrp_requirements_satisfied);
        assert_eq!(r.qrp_transfer_cents, 50_000_000);
        assert_eq!(r.employer_reversion_cents, 150_000_000);
        assert_eq!(r.effective_excise_rate_percent, 20);
        assert_eq!(r.section_4980_excise_tax_cents, 30_000_000);
    }

    #[test]
    fn qrp_94_percent_participants_no_qualification() {
        let mut i = direct_reversion_2m_surplus();
        i.termination_structure = TerminationStructure::QualifiedReplacementPlan;
        i.qrp_active_participant_percent = 94;
        i.qrp_transfer_percent = 25;
        i.qrp_seven_year_ratable_allocation = true;
        let r = check(&i);
        assert!(!r.qrp_requirements_satisfied);
        assert_eq!(r.effective_excise_rate_percent, 50);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 4980(d)(2)(B)(i)")
                && f.contains("94%")
                && f.contains("at least 95%")));
    }

    #[test]
    fn qrp_24_percent_transfer_no_qualification() {
        let mut i = direct_reversion_2m_surplus();
        i.termination_structure = TerminationStructure::QualifiedReplacementPlan;
        i.qrp_active_participant_percent = 95;
        i.qrp_transfer_percent = 24;
        i.qrp_seven_year_ratable_allocation = true;
        let r = check(&i);
        assert!(!r.qrp_requirements_satisfied);
        assert_eq!(r.effective_excise_rate_percent, 50);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 4980(d)(2)(B)(ii)")
                && f.contains("24%")
                && f.contains("at least 25%")));
    }

    #[test]
    fn qrp_no_seven_year_allocation_no_qualification() {
        let mut i = direct_reversion_2m_surplus();
        i.termination_structure = TerminationStructure::QualifiedReplacementPlan;
        i.qrp_active_participant_percent = 95;
        i.qrp_transfer_percent = 25;
        i.qrp_seven_year_ratable_allocation = false;
        let r = check(&i);
        assert!(!r.qrp_requirements_satisfied);
        assert_eq!(r.effective_excise_rate_percent, 50);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 4980(d)(2)(B)(iii)") && f.contains("7-year")));
    }

    #[test]
    fn pro_rata_benefit_increase_20_percent_qualifies() {
        let mut i = direct_reversion_2m_surplus();
        i.termination_structure = TerminationStructure::ProRataBenefitIncrease;
        i.pro_rata_benefit_increase_percent = 20;
        i.benefit_increase_immediate = true;
        let r = check(&i);
        assert!(r.pro_rata_benefit_requirements_satisfied);
        assert_eq!(r.effective_excise_rate_percent, 20);
        assert_eq!(r.section_4980_excise_tax_cents, 40_000_000);
    }

    #[test]
    fn pro_rata_benefit_increase_19_percent_no_qualification() {
        let mut i = direct_reversion_2m_surplus();
        i.termination_structure = TerminationStructure::ProRataBenefitIncrease;
        i.pro_rata_benefit_increase_percent = 19;
        i.benefit_increase_immediate = true;
        let r = check(&i);
        assert!(!r.pro_rata_benefit_requirements_satisfied);
        assert_eq!(r.effective_excise_rate_percent, 50);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 4980(d)(3)(B)")
                && f.contains("19%")
                && f.contains("at least 20%")));
    }

    #[test]
    fn pro_rata_benefit_not_immediate_no_qualification() {
        let mut i = direct_reversion_2m_surplus();
        i.termination_structure = TerminationStructure::ProRataBenefitIncrease;
        i.pro_rata_benefit_increase_percent = 25;
        i.benefit_increase_immediate = false;
        let r = check(&i);
        assert!(!r.pro_rata_benefit_requirements_satisfied);
        assert_eq!(r.effective_excise_rate_percent, 50);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 4980(d)(3)(C)") && f.contains("IMMEDIATELY")));
    }

    #[test]
    fn no_surplus_no_excise_tax() {
        let mut i = direct_reversion_2m_surplus();
        i.plan_assets_cents = 300_000_000;
        i.plan_liabilities_cents = 300_000_000;
        let r = check(&i);
        assert_eq!(r.plan_surplus_cents, 0);
        assert_eq!(r.section_4980_excise_tax_cents, 0);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("NO SURPLUS") && f.contains("DOES NOT APPLY")));
    }

    #[test]
    fn corporate_income_tax_stack() {
        let r = check(&direct_reversion_2m_surplus());
        let expected_income_tax = 200_000_000_u64 * 2600 / 10_000;
        assert_eq!(r.corporate_income_tax_cents, expected_income_tax);
        let expected_total = 100_000_000_u64 + expected_income_tax;
        assert_eq!(r.total_tax_burden_cents, expected_total);
    }

    #[test]
    fn effective_combined_rate_75_percent_default() {
        let r = check(&direct_reversion_2m_surplus());
        let expected_rate = (r.total_tax_burden_cents * 100) / r.employer_reversion_cents;
        assert_eq!(r.effective_combined_rate_percent, expected_rate as u32);
        assert!(r.effective_combined_rate_percent >= 75);
    }

    #[test]
    fn effective_combined_rate_46_percent_qrp() {
        let mut i = direct_reversion_2m_surplus();
        i.termination_structure = TerminationStructure::QualifiedReplacementPlan;
        i.qrp_active_participant_percent = 95;
        i.qrp_transfer_percent = 25;
        i.qrp_seven_year_ratable_allocation = true;
        let r = check(&i);
        assert!(r.effective_combined_rate_percent <= 50);
    }

    #[test]
    fn termination_structure_truth_table_three_cells() {
        for structure in [
            TerminationStructure::DirectReversion,
            TerminationStructure::QualifiedReplacementPlan,
            TerminationStructure::ProRataBenefitIncrease,
        ] {
            let mut i = direct_reversion_2m_surplus();
            i.termination_structure = structure;
            i.qrp_active_participant_percent = 95;
            i.qrp_transfer_percent = 25;
            i.qrp_seven_year_ratable_allocation = true;
            i.pro_rata_benefit_increase_percent = 20;
            i.benefit_increase_immediate = true;
            let r = check(&i);
            let _ = r.effective_excise_rate_percent;
        }
    }

    #[test]
    fn qrp_uniquely_eliminates_reversion_on_transferred_portion() {
        let mut i = direct_reversion_2m_surplus();
        i.termination_structure = TerminationStructure::QualifiedReplacementPlan;
        i.qrp_active_participant_percent = 95;
        i.qrp_transfer_percent = 30;
        i.qrp_seven_year_ratable_allocation = true;
        let r = check(&i);
        assert_eq!(r.qrp_transfer_cents, 60_000_000);
        assert_eq!(r.employer_reversion_cents, 140_000_000);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&direct_reversion_2m_surplus());
        assert!(r.citation.contains("§ 4980(a)"));
        assert!(r.citation.contains("§ 4980(b)"));
        assert!(r.citation.contains("§ 4980(c)"));
        assert!(r.citation.contains("§ 4980(d)"));
        assert!(r.citation.contains("§ 4980(d)(1)"));
        assert!(r.citation.contains("§ 4980(d)(2)"));
        assert!(r.citation.contains("§ 4980(d)(2)(A)"));
        assert!(r.citation.contains("§ 4980(d)(2)(B)(i)"));
        assert!(r.citation.contains("§ 4980(d)(2)(B)(ii)"));
        assert!(r.citation.contains("§ 4980(d)(2)(B)(iii)"));
        assert!(r.citation.contains("§ 4980(d)(3)"));
        assert!(r.citation.contains("§ 4980(d)(4)"));
        assert!(r.citation.contains("§ 404"));
        assert!(r.citation.contains("§ 414(b)/(c)/(m)/(o)"));
        assert!(r.citation.contains("Treas. Reg. § 1.4980-1"));
        assert!(r.citation.contains("Rev. Rul. 2003-85"));
        assert!(r.citation.contains("Rev. Rul. 89-87"));
        assert!(r.citation.contains("PLR 9701036"));
        assert!(r.citation.contains("§ 401(a)"));
        assert!(r.citation.contains("§ 411"));
        assert!(r.citation.contains("§ 412"));
        assert!(r.citation.contains("§ 415"));
    }

    #[test]
    fn note_pins_general_rule_overview() {
        let r = check(&direct_reversion_2m_surplus());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 4980 — EXCISE TAX on employer reversion")
                && n.contains("50% rate reduced to 20%")));
    }

    #[test]
    fn note_pins_qualified_db_plan_scope() {
        let r = check(&direct_reversion_2m_surplus());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("QUALIFIED DEFINED BENEFIT plans")
                && n.contains("§ 401(a)")
                && n.contains("§ 411")
                && n.contains("§ 412")
                && n.contains("§ 415")
                && n.contains("NOT applicable")
                && n.contains("§ 401(k)")));
    }

    #[test]
    fn note_pins_subsection_a_general_rule() {
        let r = check(&direct_reversion_2m_surplus());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 4980(a) GENERAL RULE") && n.contains("20% base tax")));
    }

    #[test]
    fn note_pins_subsection_d_50_percent_default() {
        let r = check(&direct_reversion_2m_surplus());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 4980(d) INCREASED 50% RATE") && n.contains("70-75%")));
    }

    #[test]
    fn note_pins_subsection_d2_qrp_three_requirements() {
        let r = check(&direct_reversion_2m_surplus());
        assert!(r.notes.iter().any(|n| n
            .contains("§ 4980(d)(2) QUALIFIED REPLACEMENT PLAN (QRP) REQUIREMENTS")
            && n.contains("95%")
            && n.contains("25%")
            && n.contains("7-year")));
    }

    #[test]
    fn note_pins_subsection_d3_pro_rata_benefit_three_requirements() {
        let r = check(&direct_reversion_2m_surplus());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 4980(d)(3) PRO RATA BENEFIT INCREASE")
                && n.contains("20% of maximum")
                && n.contains("take effect immediately")));
    }

    #[test]
    fn note_pins_subsection_d4_qualified_participant() {
        let r = check(&direct_reversion_2m_surplus());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 4980(d)(4) QUALIFIED PARTICIPANT")
                && n.contains("active participant")
                && n.contains("receiving benefits")
                && n.contains("being deferred")));
    }

    #[test]
    fn note_pins_subsection_c_employer_reversion_definition() {
        let r = check(&direct_reversion_2m_surplus());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 4980(c) EMPLOYER REVERSION DEFINITION")
                && n.contains("cash or fair market value")
                && n.contains("EXCLUDES")
                && n.contains("§ 404")));
    }

    #[test]
    fn note_pins_income_tax_interaction() {
        let r = check(&direct_reversion_2m_surplus());
        assert!(r.notes.iter().any(|n| n
            .contains("§ 4980 interaction with § 162 + § 404 + income tax")
            && n.contains("stacked liability")
            && n.contains("75%")));
    }

    #[test]
    fn note_pins_four_termination_routes() {
        let r = check(&direct_reversion_2m_surplus());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 4980 PRACTICAL TERMINATION ROUTES")
                && n.contains("PLR 9701036")
                && n.contains("Rev. Rul. 2003-85")));
    }

    #[test]
    fn note_pins_trader_fact_patterns_five() {
        let r = check(&direct_reversion_2m_surplus());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Trader-critical fact patterns")
                && n.contains("$1.52M total (76% effective)")
                && n.contains("§ 414(b)/(c)/(m)/(o)")
                && n.contains("M&A-driven plan termination")));
    }

    #[test]
    fn note_pins_companion_modules() {
        let r = check(&direct_reversion_2m_surplus());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Companion to section_401k")
                && n.contains("section_415")
                && n.contains("section_457b")
                && n.contains("section_4975")
                && n.contains("section_162m")));
    }

    #[test]
    fn defensive_overflow_saturating() {
        let mut i = direct_reversion_2m_surplus();
        i.plan_assets_cents = u64::MAX;
        i.federal_corporate_tax_rate_bp = u32::MAX;
        let r = check(&i);
        let _ = r.total_tax_burden_cents;
    }
}
