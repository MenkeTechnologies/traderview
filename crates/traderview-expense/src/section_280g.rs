//! IRC § 280G — Golden Parachute Payments. Direct
//! companion to section_422 (ISO — iter 438),
//! section_423 (ESPP — iter 440), section_409a
//! (deferred compensation), section_4973 (excess
//! contributions — iter 442). Trader-critical because
//! traders at venture-backed startups frequently
//! encounter § 280G calculations during ACQUISITIONS
//! when accelerated equity vesting + transaction
//! bonuses + severance payments trigger "parachute
//! payment" status.
//!
//! § 280G(a) DENIES employer DEDUCTION for any "EXCESS
//! PARACHUTE PAYMENT" to a "DISQUALIFIED INDIVIDUAL"
//! that is "CONTINGENT ON A CHANGE IN OWNERSHIP OR
//! CONTROL" of the corporation.
//!
//! § 4999 imposes a 20% EXCISE TAX on the recipient
//! disqualified individual on any excess parachute
//! payment.
//!
//! **§ 280G(b)(1) PARACHUTE PAYMENT DEFINITION** —
//! payment to disqualified individual:
//! 1. CONTINGENT on change in ownership or control;
//! 2. To DISQUALIFIED INDIVIDUAL under § 280G(c);
//! 3. Aggregate present value EQUALS OR EXCEEDS
//!    THREE TIMES (3×) the disqualified individual's
//!    BASE AMOUNT under § 280G(b)(3).
//!
//! If aggregate parachute payments EQUAL OR EXCEED
//! 3× base amount, the ENTIRE EXCESS OVER 1× BASE
//! AMOUNT is "excess parachute payment" — NOT just
//! the over-3x portion. This creates a sharp CLIFF:
//! one dollar over 3× triggers excise tax + lost
//! deduction on the entire amount over 1× base.
//!
//! **§ 280G(b)(3) BASE AMOUNT** — disqualified
//! individual's ANNUALIZED INCLUDIBLE COMPENSATION
//! for the BASE PERIOD = 5 MOST RECENT TAX YEARS
//! ending before the change-in-control date.
//!
//! **§ 280G(c) DISQUALIFIED INDIVIDUAL** — three
//! categories:
//! 1. OFFICER (per Treas. Reg. § 1.280G-1 Q&A 18:
//!    not more than 50 employees of corporation
//!    treated as officers, regardless of title);
//! 2. SHAREHOLDER owning stock with FMV exceeding 1%
//!    of total stock FMV at any time during 12-month
//!    disqualified-individual-determination period
//!    ending on change-in-control date;
//! 3. HIGHLY COMPENSATED INDIVIDUAL — annual
//!    compensation in top 1% of all employees or top
//!    250 employees (whichever is less); 2026
//!    threshold $155,000+ AND in top 1% / top 250.
//!
//! **§ 280G(b)(2)(A) CHANGE IN OWNERSHIP OR CONTROL**
//! defined under Treas. Reg. § 1.280G-1 Q&A 27-29:
//! 1. CHANGE IN OWNERSHIP — one person or group
//!    acquires MORE THAN 50% of total FMV or voting
//!    power;
//! 2. CHANGE IN EFFECTIVE CONTROL — acquisition of
//!    35% of voting power within 12 months, OR
//!    majority of board replaced within 12 months;
//! 3. CHANGE IN OWNERSHIP OF SUBSTANTIAL ASSETS —
//!    acquisition of 40%+ of total gross FMV of
//!    corporation's assets within 12 months.
//!
//! **§ 280G(b)(5) SMALL BUSINESS EXCEPTION** —
//! § 280G DOES NOT APPLY to private corporations
//! (no public stock trading) if:
//! 1. § 280G(b)(5)(A)(i) — small business CORP
//!    (Subchapter S election under § 1361); OR
//! 2. § 280G(b)(5)(A)(ii) — payment APPROVED BY
//!    SHAREHOLDERS holding more than 75% of voting
//!    power AFTER ADEQUATE DISCLOSURE (the
//!    "SHAREHOLDER VOTE / CLEANSING VOTE" safe
//!    harbor).
//!
//! Shareholder approval requires (a) DETERMINATION
//! that "if not for shareholder approval payment
//! would be a parachute payment"; (b) BINDING VOTE
//! on right to receive payment; (c) > 75% of voting
//! power voting in favor; (d) ADEQUATE DISCLOSURE
//! of all material facts under Treas. Reg.
//! § 1.280G-1 Q&A 7.
//!
//! **§ 280G(b)(4) REASONABLE COMPENSATION
//! EXCEPTION** — payment is NOT parachute payment
//! to extent it is REASONABLE COMPENSATION FOR
//! SERVICES RENDERED ON OR AFTER change in control;
//! BURDEN ON TAXPAYER to establish reasonableness
//! by CLEAR AND CONVINCING EVIDENCE under Treas.
//! Reg. § 1.280G-1 Q&A 40.
//!
//! **§ 280G interaction with § 4999** — § 4999
//! imposes 20% EXCISE TAX on disqualified individual
//! on amount of excess parachute payment;
//! NON-DEDUCTIBLE to recipient AND employer; many
//! employment agreements include § 280G GROSS-UP
//! CLAUSES (employer reimburses excise tax + tax
//! on gross-up) — controversial post-Dodd-Frank.
//!
//! **§ 280G GROSS-UP vs MODIFIED CUTBACK** — two
//! common employment-contract structures:
//! - GROSS-UP — employer pays additional cash to
//!   make recipient whole for § 4999 excise tax +
//!   federal/state income tax on gross-up; very
//!   expensive to employer; falling out of favor;
//! - MODIFIED CUTBACK / "BEST AFTER-TAX" — cut
//!   parachute payment to maximum of (1) just-under
//!   3× base or (2) full payment net of excise tax,
//!   whichever leaves disqualified individual with
//!   GREATER AFTER-TAX AMOUNT.
//!
//! Trader-critical fact patterns:
//! 1. Trader works at startup acquired for $500M;
//!    trader's accelerated vesting + transaction
//!    bonus + severance = $4.5M; base amount
//!    (5-year average) = $1.0M; 3× base amount =
//!    $3.0M; total parachute exceeds 3× → ENTIRE
//!    $3.5M EXCESS over 1× base ($1.0M) is excess
//!    parachute payment → $700K § 4999 excise +
//!    employer loses § 162 deduction.
//! 2. Same trader at private S-corp acquisition —
//!    § 280G(b)(5) small business exception applies;
//!    75%+ shareholder vote with adequate disclosure
//!    eliminates § 280G entirely.
//! 3. Trader establishes 60% of parachute is
//!    REASONABLE COMPENSATION FOR FUTURE SERVICES
//!    under § 280G(b)(4) — that portion excluded
//!    from parachute calculation; brings remainder
//!    just under 3× threshold.
//! 4. § 280G(c) DISQUALIFIED INDIVIDUAL — trader
//!    holds 0.5% of stock and is NOT an officer +
//!    NOT in top 1% of employee comp — NOT
//!    disqualified individual; § 280G inapplicable.
//! 5. § 280G GROSS-UP CLAUSE — trader's employment
//!    agreement specifies employer reimburses § 4999
//!    excise tax + tax on gross-up; employer
//!    effectively bears ~50% additional cost on the
//!    excess parachute payment.
//!
//! Citations: 26 USC § 280G(a)-(e); 26 USC § 4999;
//! 26 USC § 162 (compensation deduction); 26 USC
//! § 1361 (S corporation election); Treas. Reg.
//! § 1.280G-1 (Golden Parachute Payments) Q&A 1
//! through Q&A 45; Rev. Rul. 2005-39; IRS Pub. 5975
//! (Golden Parachute Payments Guide).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DisqualifiedIndividualType {
    Officer,
    OnePercentShareholder,
    HighlyCompensatedEmployee,
    NotDisqualified,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ChangeInControlType {
    /// More than 50% of FMV or voting power acquired.
    ChangeInOwnership,
    /// 35% voting power within 12 months OR majority
    /// of board replaced within 12 months.
    ChangeInEffectiveControl,
    /// 40%+ of total gross FMV of assets acquired
    /// within 12 months.
    ChangeInSubstantialAssets,
    /// No change in control.
    NoChangeInControl,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section280gInput {
    pub disqualified_individual_type: DisqualifiedIndividualType,
    pub change_in_control: ChangeInControlType,
    /// Annualized includible compensation for 5-year
    /// base period in cents.
    pub base_amount_cents: u64,
    /// Aggregate present value of parachute payments
    /// in cents.
    pub aggregate_parachute_payments_cents: u64,
    /// Portion of parachute classified as REASONABLE
    /// COMPENSATION FOR SERVICES ON OR AFTER change in
    /// control under § 280G(b)(4).
    pub reasonable_compensation_carveout_cents: u64,
    /// Whether private corporation (no public stock
    /// trading) — eligible for § 280G(b)(5) small
    /// business exception.
    pub private_corporation: bool,
    /// Whether corporation is S corporation under
    /// § 1361 — § 280G(b)(5)(A)(i) automatic
    /// exception.
    pub s_corporation_election: bool,
    /// Whether shareholder approval with > 75% of
    /// voting power obtained after adequate disclosure
    /// — § 280G(b)(5)(A)(ii) cleansing vote safe
    /// harbor.
    pub shareholder_approval_above_75_with_disclosure: bool,
    /// Whether employment agreement contains § 280G
    /// gross-up clause.
    pub has_gross_up_clause: bool,
    /// Whether modified-cutback clause applies
    /// (cut to just-under 3× or accept full net of
    /// excise).
    pub modified_cutback_clause_applies: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section280gResult {
    pub is_disqualified_individual: bool,
    pub change_in_control_engaged: bool,
    pub base_amount_cents: u64,
    pub three_times_base_amount_cents: u64,
    pub parachute_after_reasonable_carveout_cents: u64,
    pub three_times_threshold_met: bool,
    pub small_business_exception_applies: bool,
    pub excess_parachute_payment_cents: u64,
    pub section_4999_excise_tax_cents: u64,
    pub employer_deduction_lost_cents: u64,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section280gInput) -> Section280gResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let is_disqualified_individual = !matches!(
        input.disqualified_individual_type,
        DisqualifiedIndividualType::NotDisqualified
    );

    let change_in_control_engaged = !matches!(
        input.change_in_control,
        ChangeInControlType::NoChangeInControl
    );

    let three_times_base_amount_cents = input.base_amount_cents.saturating_mul(3);

    let parachute_after_reasonable_carveout_cents = input
        .aggregate_parachute_payments_cents
        .saturating_sub(input.reasonable_compensation_carveout_cents);

    let three_times_threshold_met = parachute_after_reasonable_carveout_cents
        >= three_times_base_amount_cents
        && is_disqualified_individual
        && change_in_control_engaged;

    let small_business_exception_applies = input.private_corporation
        && (input.s_corporation_election || input.shareholder_approval_above_75_with_disclosure);

    let excess_parachute_payment_cents = if three_times_threshold_met
        && !small_business_exception_applies
    {
        parachute_after_reasonable_carveout_cents.saturating_sub(input.base_amount_cents)
    } else {
        0
    };

    let section_4999_excise_tax_cents = excess_parachute_payment_cents.saturating_mul(20) / 100;

    let employer_deduction_lost_cents = excess_parachute_payment_cents;

    if !is_disqualified_individual {
        failure_reasons.push(
            "26 USC § 280G(c) — recipient is NOT a DISQUALIFIED INDIVIDUAL (not an officer + not a 1%+ shareholder + not highly compensated employee in top 1% / top 250); § 280G inapplicable; payment fully deductible to employer; no § 4999 excise tax".to_string(),
        );
    }

    if !change_in_control_engaged {
        failure_reasons.push(
            "26 USC § 280G(b)(2)(A) — NO CHANGE IN OWNERSHIP OR CONTROL engaged (no > 50% ownership change + no 35% voting power acquisition + no majority board replacement + no 40%+ asset acquisition within 12 months); § 280G inapplicable absent change in control".to_string(),
        );
    }

    if small_business_exception_applies {
        let exception_basis = if input.s_corporation_election {
            "§ 280G(b)(5)(A)(i) — S corporation election under § 1361 — automatic small business exception applies"
        } else {
            "§ 280G(b)(5)(A)(ii) — SHAREHOLDER VOTE / CLEANSING VOTE safe harbor — > 75% of voting power approved with adequate disclosure under Treas. Reg. § 1.280G-1 Q&A 7"
        };
        failure_reasons.push(format!(
            "26 USC § 280G(b)(5) SMALL BUSINESS EXCEPTION applies — private corporation with {}; § 280G entirely eliminated",
            exception_basis
        ));
    }

    if three_times_threshold_met && !small_business_exception_applies {
        failure_reasons.push(format!(
            "26 USC § 280G(b)(1) THREE-TIMES THRESHOLD MET — aggregate parachute payments {} cents (after § 280G(b)(4) reasonable-compensation carveout of {} cents) EQUALS OR EXCEEDS 3× base amount {} cents; CLIFF triggered: ENTIRE EXCESS OVER 1× BASE ({}) becomes excess parachute payment",
            parachute_after_reasonable_carveout_cents,
            input.reasonable_compensation_carveout_cents,
            three_times_base_amount_cents,
            input.base_amount_cents
        ));
        failure_reasons.push(format!(
            "26 USC § 4999 — 20% EXCISE TAX of {} cents on disqualified individual on excess parachute payment of {} cents; non-deductible to recipient + payment lost as § 162 deduction to employer ({} cents)",
            section_4999_excise_tax_cents,
            excess_parachute_payment_cents,
            employer_deduction_lost_cents
        ));
    }

    if input.has_gross_up_clause && excess_parachute_payment_cents > 0 {
        failure_reasons.push(
            "26 USC § 280G GROSS-UP CLAUSE — employment agreement reimburses recipient for § 4999 excise tax + federal/state income tax on gross-up; employer effectively bears ~50% additional cost on excess parachute payment; controversial post-Dodd-Frank but legal".to_string(),
        );
    }

    if input.modified_cutback_clause_applies && three_times_threshold_met {
        failure_reasons.push(
            "26 USC § 280G MODIFIED CUTBACK / BEST-AFTER-TAX CLAUSE — payment cut to MAXIMUM of (1) just-under 3× base amount OR (2) full payment net of § 4999 excise; whichever leaves disqualified individual with GREATER AFTER-TAX AMOUNT; avoids § 4999 excise altogether at modest payment reduction".to_string(),
        );
    }

    let individual_type_label = match input.disqualified_individual_type {
        DisqualifiedIndividualType::Officer => "§ 280G(c)(1) OFFICER — Treas. Reg. § 1.280G-1 Q&A 18: not more than 50 employees treated as officers regardless of title",
        DisqualifiedIndividualType::OnePercentShareholder => "§ 280G(c)(2) 1%+ SHAREHOLDER — stock FMV exceeds 1% of total stock FMV at any time during 12-month period ending on change-in-control date",
        DisqualifiedIndividualType::HighlyCompensatedEmployee => "§ 280G(c)(3) HIGHLY COMPENSATED EMPLOYEE — compensation in top 1% of all employees or top 250 employees (whichever is less); 2026 threshold $155,000+",
        DisqualifiedIndividualType::NotDisqualified => "Not a disqualified individual under § 280G(c)",
    };
    failure_reasons.push(format!(
        "Disqualified individual category: {}",
        individual_type_label
    ));

    let notes: Vec<String> = vec![
        "26 USC § 280G(a) — DENIES employer DEDUCTION for excess parachute payment to disqualified individual contingent on change in ownership or control; complements § 4999 20% recipient excise tax".to_string(),
        "26 USC § 280G(b)(1) PARACHUTE PAYMENT DEFINITION — payment to disqualified individual (1) CONTINGENT on change in control; (2) under § 280G(c); (3) aggregate present value EQUALS OR EXCEEDS 3× base amount; CLIFF: $1 over 3× triggers excise + lost deduction on ENTIRE excess over 1× base, not just over-3× portion".to_string(),
        "26 USC § 280G(b)(3) BASE AMOUNT — annualized includible compensation for BASE PERIOD = 5 MOST RECENT TAX YEARS ending before change-in-control date".to_string(),
        "26 USC § 280G(c) DISQUALIFIED INDIVIDUAL — three categories: (1) OFFICER (max 50 employees regardless of title under Treas. Reg. § 1.280G-1 Q&A 18); (2) 1%+ SHAREHOLDER (FMV exceeds 1% of total stock FMV); (3) HIGHLY COMPENSATED EMPLOYEE (top 1% or top 250 of employees; 2026 $155K threshold)".to_string(),
        "26 USC § 280G(b)(2)(A) CHANGE IN OWNERSHIP OR CONTROL (Treas. Reg. § 1.280G-1 Q&A 27-29): (1) CHANGE IN OWNERSHIP > 50% FMV or voting power; (2) CHANGE IN EFFECTIVE CONTROL 35% voting acquired within 12 months OR majority board replaced within 12 months; (3) CHANGE IN OWNERSHIP OF SUBSTANTIAL ASSETS 40%+ of total gross FMV within 12 months".to_string(),
        "26 USC § 280G(b)(5) SMALL BUSINESS EXCEPTION — § 280G does not apply to private corporations if (a) § 1361 S corporation election; OR (b) shareholder vote > 75% voting power approval with adequate disclosure under Treas. Reg. § 1.280G-1 Q&A 7 (the cleansing-vote safe harbor)".to_string(),
        "26 USC § 280G(b)(4) REASONABLE COMPENSATION EXCEPTION — payment NOT parachute payment to extent it is reasonable compensation for services rendered ON OR AFTER change in control; burden on taxpayer to establish by CLEAR AND CONVINCING EVIDENCE under Treas. Reg. § 1.280G-1 Q&A 40".to_string(),
        "26 USC § 4999 — 20% EXCISE TAX on disqualified individual on excess parachute payment; non-deductible to recipient + non-deductible to employer; employment agreements may include § 280G GROSS-UP CLAUSE (controversial post-Dodd-Frank) OR MODIFIED CUTBACK / BEST-AFTER-TAX clause (cut to just-under 3× or accept full net of excise)".to_string(),
        "Trader-critical fact patterns: (1) startup acquired $500M; $4.5M parachute on $1M base; 3× threshold $3M met → entire $3.5M excess over 1× = excess parachute → $700K § 4999 + employer loses § 162; (2) private S-corp § 280G(b)(5) exception with 75% shareholder vote eliminates § 280G; (3) 60% of parachute treated as reasonable compensation for post-change services under § 280G(b)(4) — brings remainder below 3×; (4) trader holds 0.5% stock + not officer + not top 1% comp → not disqualified individual; (5) gross-up clause vs modified cutback structure".to_string(),
        "Form 1120 Schedule M-3 reports parachute-payment non-deductibility for employer; Form W-2 Box 12 Code K reports § 4999 excise tax withheld at 20% on excess parachute payment to disqualified individual".to_string(),
        "Companion to section_422 (ISO — acceleration on change of control triggers § 280G inclusion) + section_423 (ESPP — purchase right acceleration) + section_409a (NQDC vesting acceleration) + section_4973 (excess contribution excise tax — different excise tax regime) + section_162m (excessive employee remuneration limit)".to_string(),
    ];

    Section280gResult {
        is_disqualified_individual,
        change_in_control_engaged,
        base_amount_cents: input.base_amount_cents,
        three_times_base_amount_cents,
        parachute_after_reasonable_carveout_cents,
        three_times_threshold_met,
        small_business_exception_applies,
        excess_parachute_payment_cents,
        section_4999_excise_tax_cents,
        employer_deduction_lost_cents,
        failure_reasons,
        citation: "26 USC § 280G(a)-(e); 26 USC § 4999; 26 USC § 162 (compensation deduction); 26 USC § 1361 (S corporation election); Treas. Reg. § 1.280G-1 (Golden Parachute Payments) Q&A 1 through Q&A 45; Rev. Rul. 2005-39; IRS Pub. 5975 (Golden Parachute Payments Guide)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn officer_acquisition_excess() -> Section280gInput {
        Section280gInput {
            disqualified_individual_type: DisqualifiedIndividualType::Officer,
            change_in_control: ChangeInControlType::ChangeInOwnership,
            base_amount_cents: 100_000_000,
            aggregate_parachute_payments_cents: 450_000_000,
            reasonable_compensation_carveout_cents: 0,
            private_corporation: false,
            s_corporation_election: false,
            shareholder_approval_above_75_with_disclosure: false,
            has_gross_up_clause: false,
            modified_cutback_clause_applies: false,
        }
    }

    #[test]
    fn three_times_threshold_met_triggers_excess() {
        let r = check(&officer_acquisition_excess());
        assert!(r.three_times_threshold_met);
        assert_eq!(r.three_times_base_amount_cents, 300_000_000);
        assert_eq!(r.excess_parachute_payment_cents, 350_000_000);
    }

    #[test]
    fn excise_tax_at_20_percent() {
        let r = check(&officer_acquisition_excess());
        assert_eq!(r.section_4999_excise_tax_cents, 70_000_000);
    }

    #[test]
    fn employer_deduction_lost_full_excess() {
        let r = check(&officer_acquisition_excess());
        assert_eq!(r.employer_deduction_lost_cents, 350_000_000);
    }

    #[test]
    fn just_under_three_times_no_excess() {
        let mut i = officer_acquisition_excess();
        i.aggregate_parachute_payments_cents = 299_999_999;
        let r = check(&i);
        assert!(!r.three_times_threshold_met);
        assert_eq!(r.excess_parachute_payment_cents, 0);
        assert_eq!(r.section_4999_excise_tax_cents, 0);
    }

    #[test]
    fn three_times_boundary_triggers_cliff() {
        let mut i = officer_acquisition_excess();
        i.aggregate_parachute_payments_cents = 300_000_000;
        let r = check(&i);
        assert!(r.three_times_threshold_met);
        assert_eq!(r.excess_parachute_payment_cents, 200_000_000);
    }

    #[test]
    fn not_disqualified_individual_no_excise() {
        let mut i = officer_acquisition_excess();
        i.disqualified_individual_type = DisqualifiedIndividualType::NotDisqualified;
        let r = check(&i);
        assert!(!r.is_disqualified_individual);
        assert!(!r.three_times_threshold_met);
        assert_eq!(r.section_4999_excise_tax_cents, 0);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 280G(c)")
            && f.contains("NOT a DISQUALIFIED INDIVIDUAL")));
    }

    #[test]
    fn no_change_in_control_no_excise() {
        let mut i = officer_acquisition_excess();
        i.change_in_control = ChangeInControlType::NoChangeInControl;
        let r = check(&i);
        assert!(!r.change_in_control_engaged);
        assert!(!r.three_times_threshold_met);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 280G(b)(2)(A)")
            && f.contains("NO CHANGE IN OWNERSHIP OR CONTROL")));
    }

    #[test]
    fn s_corp_election_small_business_exception() {
        let mut i = officer_acquisition_excess();
        i.private_corporation = true;
        i.s_corporation_election = true;
        let r = check(&i);
        assert!(r.small_business_exception_applies);
        assert_eq!(r.excess_parachute_payment_cents, 0);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 280G(b)(5)(A)(i)")
            && f.contains("S corporation")));
    }

    #[test]
    fn shareholder_75_percent_vote_small_business_exception() {
        let mut i = officer_acquisition_excess();
        i.private_corporation = true;
        i.shareholder_approval_above_75_with_disclosure = true;
        let r = check(&i);
        assert!(r.small_business_exception_applies);
        assert_eq!(r.excess_parachute_payment_cents, 0);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 280G(b)(5)(A)(ii)")
            && f.contains("CLEANSING VOTE")
            && f.contains("Q&A 7")));
    }

    #[test]
    fn public_corp_no_small_business_exception() {
        let mut i = officer_acquisition_excess();
        i.private_corporation = false;
        i.shareholder_approval_above_75_with_disclosure = true;
        let r = check(&i);
        assert!(!r.small_business_exception_applies);
    }

    #[test]
    fn reasonable_compensation_carveout_reduces_parachute() {
        let mut i = officer_acquisition_excess();
        i.reasonable_compensation_carveout_cents = 200_000_000;
        let r = check(&i);
        assert_eq!(r.parachute_after_reasonable_carveout_cents, 250_000_000);
        assert!(!r.three_times_threshold_met);
    }

    #[test]
    fn reasonable_compensation_carveout_full_neutralizes() {
        let mut i = officer_acquisition_excess();
        i.reasonable_compensation_carveout_cents = 450_000_000;
        let r = check(&i);
        assert_eq!(r.parachute_after_reasonable_carveout_cents, 0);
        assert!(!r.three_times_threshold_met);
    }

    #[test]
    fn one_percent_shareholder_qualifies() {
        let mut i = officer_acquisition_excess();
        i.disqualified_individual_type = DisqualifiedIndividualType::OnePercentShareholder;
        let r = check(&i);
        assert!(r.is_disqualified_individual);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 280G(c)(2) 1%+ SHAREHOLDER")));
    }

    #[test]
    fn highly_compensated_employee_qualifies() {
        let mut i = officer_acquisition_excess();
        i.disqualified_individual_type = DisqualifiedIndividualType::HighlyCompensatedEmployee;
        let r = check(&i);
        assert!(r.is_disqualified_individual);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 280G(c)(3) HIGHLY COMPENSATED EMPLOYEE")
            && f.contains("$155,000+")));
    }

    #[test]
    fn change_in_effective_control_engages() {
        let mut i = officer_acquisition_excess();
        i.change_in_control = ChangeInControlType::ChangeInEffectiveControl;
        let r = check(&i);
        assert!(r.change_in_control_engaged);
    }

    #[test]
    fn change_in_substantial_assets_engages() {
        let mut i = officer_acquisition_excess();
        i.change_in_control = ChangeInControlType::ChangeInSubstantialAssets;
        let r = check(&i);
        assert!(r.change_in_control_engaged);
    }

    #[test]
    fn gross_up_clause_disclosed_when_excess() {
        let mut i = officer_acquisition_excess();
        i.has_gross_up_clause = true;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 280G GROSS-UP CLAUSE")
            && f.contains("Dodd-Frank")));
    }

    #[test]
    fn modified_cutback_clause_disclosed_when_threshold_met() {
        let mut i = officer_acquisition_excess();
        i.modified_cutback_clause_applies = true;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 280G MODIFIED CUTBACK")
            && f.contains("BEST-AFTER-TAX")
            && f.contains("GREATER AFTER-TAX AMOUNT")));
    }

    #[test]
    fn disqualified_individual_truth_table_four_cells() {
        for (it, expect_disq) in [
            (DisqualifiedIndividualType::Officer, true),
            (DisqualifiedIndividualType::OnePercentShareholder, true),
            (DisqualifiedIndividualType::HighlyCompensatedEmployee, true),
            (DisqualifiedIndividualType::NotDisqualified, false),
        ] {
            let mut i = officer_acquisition_excess();
            i.disqualified_individual_type = it;
            let r = check(&i);
            assert_eq!(r.is_disqualified_individual, expect_disq, "it={:?}", it);
        }
    }

    #[test]
    fn change_in_control_truth_table_four_cells() {
        for (cc, expect_engaged) in [
            (ChangeInControlType::ChangeInOwnership, true),
            (ChangeInControlType::ChangeInEffectiveControl, true),
            (ChangeInControlType::ChangeInSubstantialAssets, true),
            (ChangeInControlType::NoChangeInControl, false),
        ] {
            let mut i = officer_acquisition_excess();
            i.change_in_control = cc;
            let r = check(&i);
            assert_eq!(r.change_in_control_engaged, expect_engaged, "cc={:?}", cc);
        }
    }

    #[test]
    fn cliff_effect_invariant_one_dollar_over_3x() {
        let mut just_under = officer_acquisition_excess();
        just_under.aggregate_parachute_payments_cents = 299_999_999;
        let r_under = check(&just_under);
        assert_eq!(r_under.excess_parachute_payment_cents, 0);

        let mut at_threshold = officer_acquisition_excess();
        at_threshold.aggregate_parachute_payments_cents = 300_000_000;
        let r_at = check(&at_threshold);
        assert_eq!(r_at.excess_parachute_payment_cents, 200_000_000);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&officer_acquisition_excess());
        assert!(r.citation.contains("§ 280G(a)-(e)"));
        assert!(r.citation.contains("§ 4999"));
        assert!(r.citation.contains("§ 162"));
        assert!(r.citation.contains("§ 1361"));
        assert!(r.citation.contains("Treas. Reg. § 1.280G-1"));
        assert!(r.citation.contains("Q&A 1 through Q&A 45"));
        assert!(r.citation.contains("Rev. Rul. 2005-39"));
        assert!(r.citation.contains("IRS Pub. 5975"));
    }

    #[test]
    fn note_pins_subsection_a_employer_deduction_denial() {
        let r = check(&officer_acquisition_excess());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 280G(a)")
            && n.contains("DENIES employer DEDUCTION")
            && n.contains("§ 4999 20% recipient excise tax")));
    }

    #[test]
    fn note_pins_subsection_b1_cliff_definition() {
        let r = check(&officer_acquisition_excess());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 280G(b)(1) PARACHUTE PAYMENT")
            && n.contains("3× base amount")
            && n.contains("CLIFF")
            && n.contains("not just over-3× portion")));
    }

    #[test]
    fn note_pins_subsection_b3_base_amount() {
        let r = check(&officer_acquisition_excess());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 280G(b)(3) BASE AMOUNT")
            && n.contains("5 MOST RECENT TAX YEARS")));
    }

    #[test]
    fn note_pins_subsection_c_three_categories() {
        let r = check(&officer_acquisition_excess());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 280G(c) DISQUALIFIED INDIVIDUAL")
            && n.contains("OFFICER")
            && n.contains("1%+ SHAREHOLDER")
            && n.contains("HIGHLY COMPENSATED EMPLOYEE")
            && n.contains("$155K")));
    }

    #[test]
    fn note_pins_subsection_b2_change_categories() {
        let r = check(&officer_acquisition_excess());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 280G(b)(2)(A) CHANGE IN OWNERSHIP OR CONTROL")
            && n.contains("> 50%")
            && n.contains("35% voting")
            && n.contains("40%+ of total gross FMV")));
    }

    #[test]
    fn note_pins_subsection_b5_small_business_exception() {
        let r = check(&officer_acquisition_excess());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 280G(b)(5) SMALL BUSINESS EXCEPTION")
            && n.contains("§ 1361 S corporation election")
            && n.contains("> 75% voting power")
            && n.contains("cleansing-vote safe harbor")));
    }

    #[test]
    fn note_pins_subsection_b4_reasonable_compensation() {
        let r = check(&officer_acquisition_excess());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 280G(b)(4) REASONABLE COMPENSATION EXCEPTION")
            && n.contains("CLEAR AND CONVINCING EVIDENCE")
            && n.contains("Q&A 40")));
    }

    #[test]
    fn note_pins_section_4999_and_gross_up_modified_cutback() {
        let r = check(&officer_acquisition_excess());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 4999")
            && n.contains("20% EXCISE TAX")
            && n.contains("GROSS-UP CLAUSE")
            && n.contains("MODIFIED CUTBACK")
            && n.contains("Dodd-Frank")));
    }

    #[test]
    fn note_pins_trader_fact_patterns_five() {
        let r = check(&officer_acquisition_excess());
        assert!(r.notes.iter().any(|n|
            n.contains("Trader-critical fact patterns")
            && n.contains("$500M")
            && n.contains("$700K § 4999")
            && n.contains("0.5% stock")));
    }

    #[test]
    fn note_pins_form_1120_schedule_m3() {
        let r = check(&officer_acquisition_excess());
        assert!(r.notes.iter().any(|n|
            n.contains("Form 1120 Schedule M-3")
            && n.contains("Form W-2 Box 12 Code K")));
    }

    #[test]
    fn note_pins_companion_modules() {
        let r = check(&officer_acquisition_excess());
        assert!(r.notes.iter().any(|n|
            n.contains("section_422")
            && n.contains("section_423")
            && n.contains("section_409a")
            && n.contains("section_4973")
            && n.contains("section_162m")));
    }

    #[test]
    fn defensive_overflow_saturating() {
        let mut i = officer_acquisition_excess();
        i.base_amount_cents = u64::MAX;
        i.aggregate_parachute_payments_cents = u64::MAX;
        let r = check(&i);
        let _ = r.section_4999_excise_tax_cents;
    }
}
