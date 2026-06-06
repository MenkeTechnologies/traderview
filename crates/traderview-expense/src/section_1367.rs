//! IRC § 1367 — Adjustments to basis of S corporation
//! shareholder stock.
//!
//! Core math for any trader organizing as an S corporation —
//! § 1367 governs how shareholder stock basis shifts each year
//! as the corporation's income, losses, distributions, and
//! expenses flow through. The basis figure controls (a) how much
//! corporate loss the shareholder may deduct on the personal
//! return under § 1366(d), (b) whether distributions are
//! tax-free under § 1368, and (c) how much gain is recognized
//! on disposition of the stock.
//!
//! § 1367(a)(1) INCREASES (positive items):
//!   (A) Separately stated income items per § 1366(a)(1)(A);
//!   (B) Nonseparately computed income per § 1366(a)(1)(B)
//!       (ordinary trade-or-business income);
//!   (C) Excess of deductions for depletion over basis of the
//!       depleted property.
//!
//! § 1367(a)(2) DECREASES (negative items — basis floored at $0):
//!   (A) Distributions by the corporation that were not
//!       includible in shareholder income by reason of § 1368;
//!   (B) Separately stated loss/deduction items per § 1366(a)(1)(A);
//!   (C) Nonseparately computed loss per § 1366(a)(1)(B);
//!   (D) Noncapital, nondeductible expenses (e.g., 50% of meals
//!       under § 274(n), fines, political contributions, lobbying);
//!   (E) Shareholder's deduction for depletion.
//!
//! Treas. Reg. § 1.1367-1(f) STANDARD ORDERING RULE — basis is
//! adjusted in this specific four-step order:
//!   1. INCREASES — all § 1367(a)(1) positive items;
//!   2. DISTRIBUTIONS — § 1368 distributions (excess over basis
//!      treated as capital gain per § 1368(b)(2));
//!   3. NONDEDUCTIBLE EXPENSES — § 1367(a)(2)(D) items
//!      (CANNOT be carried forward — excess over basis is LOST
//!      under the standard rule);
//!   4. LOSSES — § 1367(a)(2)(B)/(C) items (excess over basis
//!      is SUSPENDED and carried forward indefinitely under
//!      § 1366(d)(2)).
//!
//! Treas. Reg. § 1.1367-1(g) ELECTION — shareholder may elect an
//! ALTERNATIVE ORDERING that reverses steps 3 and 4:
//!   1. INCREASES;
//!   2. DISTRIBUTIONS;
//!   3. LOSSES — applied before nondeductibles;
//!   4. NONDEDUCTIBLE EXPENSES — applied last; excess SUSPENDED
//!      and carried forward (instead of being lost).
//!
//! Election is BINDING in subsequent years unless IRS approves
//! a change back to the standard rule. Election protects
//! nondeductible-expense basis erosion at the cost of consuming
//! more basis up-front for losses.
//!
//! Distribution rule under § 1368(b)(2) — distribution in excess
//! of basis is treated as gain from the sale of the stock
//! (typically capital gain).
//!
//! Citations: 26 U.S.C. § 1367 (general basis adjustments);
//! 26 U.S.C. § 1367(a)(1) (increases — separately stated income +
//! nonseparately computed income + depletion excess); 26 U.S.C.
//! § 1367(a)(2) (decreases — distributions + losses + nondeductibles +
//! depletion); 26 U.S.C. § 1366(a)(1) (pass-through items defining
//! § 1367(a)(1)/(2) categories); 26 U.S.C. § 1366(d) (loss
//! limitation tied to basis); 26 U.S.C. § 1366(d)(2) (indefinite
//! carryforward of suspended losses); 26 U.S.C. § 1368
//! (distributions); 26 U.S.C. § 1368(b)(2) (excess-distribution
//! capital gain); Treas. Reg. § 1.1367-1(f) (standard ordering
//! rule — losses last); Treas. Reg. § 1.1367-1(g) (election for
//! alternative ordering — nondeductibles last + carryforward);
//! Form 7203 (S Corporation Shareholder Stock and Debt Basis
//! Limitations). Sibling modules: § 1361 (S-corp definition);
//! § 1374 (built-in gains tax); § 1368 (distribution mechanics
//! beyond basis).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Section1367Input {
    pub beginning_stock_basis_cents: i64,
    /// § 1367(a)(1)(A) — separately stated income items per
    /// § 1366(a)(1)(A) (capital gains, charitable contribution
    /// pass-through, etc.).
    pub separately_stated_income_cents: i64,
    /// § 1367(a)(1)(B) — nonseparately computed income per
    /// § 1366(a)(1)(B) (ordinary trade-or-business income).
    pub nonseparately_computed_income_cents: i64,
    /// § 1367(a)(1)(C) — excess of deductions for depletion over
    /// basis of the depleted property.
    pub depletion_excess_over_basis_cents: i64,
    /// § 1367(a)(2)(A) — distributions by the corporation under
    /// § 1368.
    pub distributions_cents: i64,
    /// § 1367(a)(2)(B) — separately stated loss/deduction items.
    pub separately_stated_loss_cents: i64,
    /// § 1367(a)(2)(C) — nonseparately computed loss.
    pub nonseparately_computed_loss_cents: i64,
    /// § 1367(a)(2)(D) — noncapital, nondeductible expenses
    /// (50% disallowed meals, fines, political contributions,
    /// lobbying).
    pub noncapital_nondeductible_expenses_cents: i64,
    /// § 1367(a)(2)(E) — shareholder's depletion deduction.
    pub depletion_deduction_cents: i64,
    /// Treas. Reg. § 1.1367-1(g) — elects alternative ordering
    /// (losses before nondeductibles; nondeductibles suspend).
    pub elects_alternative_ordering_g: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section1367Result {
    pub beginning_basis_cents: i64,
    /// Total § 1367(a)(1) increases applied.
    pub total_increases_cents: i64,
    /// Basis after applying all § 1367(a)(1) increases.
    pub basis_after_increases_cents: i64,
    /// Distribution amount actually applied against basis.
    pub distribution_applied_cents: i64,
    /// § 1368(b)(2) — distribution in excess of basis treated
    /// as capital gain.
    pub excess_distribution_capital_gain_cents: i64,
    /// Basis after distribution step.
    pub basis_after_distributions_cents: i64,
    /// Nondeductible expense applied (limited to basis).
    pub nondeductibles_applied_cents: i64,
    /// Under standard rule: nondeductibles in excess of basis
    /// are LOST. Under § 1.1367-1(g) election: SUSPENDED for
    /// carryforward.
    pub nondeductibles_lost_or_suspended_cents: i64,
    /// Loss/deduction applied against basis (§ 1366(d) limit).
    pub losses_allowed_cents: i64,
    /// § 1366(d)(2) — losses in excess of basis suspended and
    /// carried forward indefinitely.
    pub losses_suspended_carryforward_cents: i64,
    pub ending_basis_cents: i64,
    /// True if § 1.1367-1(g) election engaged.
    pub alternative_ordering_engaged: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section1367Input) -> Section1367Result {
    let mut notes: Vec<String> = Vec::new();

    let beginning = input.beginning_stock_basis_cents.max(0);
    let inc_sep = input.separately_stated_income_cents.max(0);
    let inc_nonsep = input.nonseparately_computed_income_cents.max(0);
    let inc_depletion = input.depletion_excess_over_basis_cents.max(0);
    let distributions = input.distributions_cents.max(0);
    let loss_sep = input.separately_stated_loss_cents.max(0);
    let loss_nonsep = input.nonseparately_computed_loss_cents.max(0);
    let nondeductibles = input.noncapital_nondeductible_expenses_cents.max(0);
    let depletion_ded = input.depletion_deduction_cents.max(0);

    let total_increases = inc_sep
        .saturating_add(inc_nonsep)
        .saturating_add(inc_depletion);
    let total_losses = loss_sep.saturating_add(loss_nonsep);
    let total_nondeductibles = nondeductibles.saturating_add(depletion_ded);

    // Step 1: Apply increases.
    let basis_after_increases = beginning.saturating_add(total_increases);

    // Step 2: Apply distributions (both orderings same here).
    let distribution_applied = distributions.min(basis_after_increases);
    let excess_distribution = (distributions - distribution_applied).max(0);
    let basis_after_distributions = basis_after_increases.saturating_sub(distribution_applied);

    let (
        nondeductibles_applied,
        nondeductibles_lost_or_suspended,
        losses_allowed,
        losses_suspended,
        ending,
    ) = if input.elects_alternative_ordering_g {
        // § 1.1367-1(g) — losses BEFORE nondeductibles.
        // Step 3: Losses.
        let losses_allowed = total_losses.min(basis_after_distributions);
        let losses_suspended = total_losses.saturating_sub(losses_allowed);
        let basis_after_losses = basis_after_distributions.saturating_sub(losses_allowed);
        // Step 4: Nondeductibles (suspended if exceed basis).
        let nondeductibles_applied = total_nondeductibles.min(basis_after_losses);
        let nondeductibles_suspended = total_nondeductibles.saturating_sub(nondeductibles_applied);
        let ending = basis_after_losses.saturating_sub(nondeductibles_applied);
        (
            nondeductibles_applied,
            nondeductibles_suspended,
            losses_allowed,
            losses_suspended,
            ending,
        )
    } else {
        // Standard § 1.1367-1(f) — nondeductibles BEFORE losses.
        // Step 3: Nondeductibles (LOST if exceed basis).
        let nondeductibles_applied = total_nondeductibles.min(basis_after_distributions);
        let nondeductibles_lost = total_nondeductibles.saturating_sub(nondeductibles_applied);
        let basis_after_nondeductibles =
            basis_after_distributions.saturating_sub(nondeductibles_applied);
        // Step 4: Losses (SUSPENDED if exceed basis).
        let losses_allowed = total_losses.min(basis_after_nondeductibles);
        let losses_suspended = total_losses.saturating_sub(losses_allowed);
        let ending = basis_after_nondeductibles.saturating_sub(losses_allowed);
        (
            nondeductibles_applied,
            nondeductibles_lost,
            losses_allowed,
            losses_suspended,
            ending,
        )
    };

    // Notes.
    if input.elects_alternative_ordering_g {
        notes.push(
            "Treas. Reg. § 1.1367-1(g) ELECTION engaged — alternative ordering rule: \
             increases → distributions → LOSSES → nondeductibles. Excess nondeductibles \
             SUSPENDED and carried forward to next year (instead of being permanently \
             lost under the standard rule). Election binds future years until IRS approves \
             change back to standard ordering."
                .to_string(),
        );
    } else {
        notes.push(
            "Treas. Reg. § 1.1367-1(f) STANDARD ORDERING — increases → distributions → \
             NONDEDUCTIBLES → losses. Nondeductibles in excess of basis are \
             PERMANENTLY LOST (no carryforward). Losses in excess of basis are SUSPENDED \
             under § 1366(d)(2) and carried forward indefinitely. Consider § 1.1367-1(g) \
             election if nondeductibles regularly exceed available basis."
                .to_string(),
        );
    }

    if excess_distribution > 0 {
        notes.push(format!(
            "§ 1368(b)(2) — distribution {} cents exceeds basis available to absorb it; \
             excess {} cents treated as gain from sale of the stock (typically capital \
             gain). Apply on Form 1040 Schedule D + Form 8949.",
            distributions, excess_distribution,
        ));
    }

    if losses_suspended > 0 {
        notes.push(format!(
            "§ 1366(d)(2) — {} cents of loss/deduction suspended; carried forward \
             indefinitely to future years when basis becomes available. Reported on \
             Form 7203 (S Corporation Shareholder Stock and Debt Basis Limitations).",
            losses_suspended,
        ));
    }

    if nondeductibles_lost_or_suspended > 0 {
        notes.push(format!(
            "Nondeductible-expense {} cents {}. Compare ordering rules: standard rule \
             loses excess permanently; § 1.1367-1(g) election preserves via carryforward.",
            nondeductibles_lost_or_suspended,
            if input.elects_alternative_ordering_g {
                "SUSPENDED for carryforward under § 1.1367-1(g) election"
            } else {
                "LOST permanently under standard § 1.1367-1(f) ordering (no carryforward)"
            },
        ));
    }

    notes.push(format!(
        "Basis flow: beginning {} → after increases {} → after distributions {} → \
         after {} {} → ending {}.",
        beginning,
        basis_after_increases,
        basis_after_distributions,
        if input.elects_alternative_ordering_g {
            "losses"
        } else {
            "nondeductibles"
        },
        if input.elects_alternative_ordering_g {
            basis_after_distributions.saturating_sub(losses_allowed)
        } else {
            basis_after_distributions.saturating_sub(nondeductibles_applied)
        },
        ending,
    ));

    notes.push(
        "Sibling modules: § 1361 (S-corp definition + eligibility — must be a small \
         business corporation with ≤100 shareholders + single class of stock); § 1366 \
         (pass-through of corporate items to shareholders; § 1366(d) basis-limited loss \
         deduction with indefinite § 1366(d)(2) carryforward); § 1368 (distribution \
         mechanics — § 1368(b)(2) excess-distribution capital gain); § 1374 (built-in \
         gains tax on C-corp-to-S-corp conversions). Form 7203 is the IRS basis-tracking \
         schedule attached to the shareholder's Form 1040."
            .to_string(),
    );

    Section1367Result {
        beginning_basis_cents: beginning,
        total_increases_cents: total_increases,
        basis_after_increases_cents: basis_after_increases,
        distribution_applied_cents: distribution_applied,
        excess_distribution_capital_gain_cents: excess_distribution,
        basis_after_distributions_cents: basis_after_distributions,
        nondeductibles_applied_cents: nondeductibles_applied,
        nondeductibles_lost_or_suspended_cents: nondeductibles_lost_or_suspended,
        losses_allowed_cents: losses_allowed,
        losses_suspended_carryforward_cents: losses_suspended,
        ending_basis_cents: ending,
        alternative_ordering_engaged: input.elects_alternative_ordering_g,
        citation: "26 U.S.C. § 1367 (general basis adjustments); 26 U.S.C. § 1367(a)(1) \
                   (increases — separately stated + nonseparately computed income + \
                   depletion excess); 26 U.S.C. § 1367(a)(2) (decreases — distributions + \
                   losses + nondeductibles + depletion); 26 U.S.C. § 1366(a)(1) \
                   (pass-through items); 26 U.S.C. § 1366(d) (loss limitation tied to \
                   basis); 26 U.S.C. § 1366(d)(2) (indefinite carryforward of suspended \
                   losses); 26 U.S.C. § 1368 (distribution mechanics); 26 U.S.C. \
                   § 1368(b)(2) (excess-distribution capital gain); Treas. Reg. \
                   § 1.1367-1(f) (standard four-step ordering); Treas. Reg. § 1.1367-1(g) \
                   (alternative-ordering election — losses-before-nondeductibles + \
                   nondeductibles-carryforward); Form 7203 (S Corporation Shareholder \
                   Stock and Debt Basis Limitations)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> Section1367Input {
        Section1367Input {
            beginning_stock_basis_cents: 10_000_000, // $100K
            separately_stated_income_cents: 0,
            nonseparately_computed_income_cents: 5_000_000, // $50K
            depletion_excess_over_basis_cents: 0,
            distributions_cents: 3_000_000, // $30K
            separately_stated_loss_cents: 0,
            nonseparately_computed_loss_cents: 4_000_000, // $40K
            noncapital_nondeductible_expenses_cents: 2_000_000, // $20K
            depletion_deduction_cents: 0,
            elects_alternative_ordering_g: false,
        }
    }

    // ── Standard ordering happy path ──────────────────────────

    #[test]
    fn standard_ordering_normal_flow() {
        // $100K + $50K = $150K, -$30K dist = $120K, -$20K nondeductible = $100K,
        // -$40K loss = $60K ending.
        let r = compute(&input());
        assert_eq!(r.basis_after_increases_cents, 15_000_000);
        assert_eq!(r.distribution_applied_cents, 3_000_000);
        assert_eq!(r.excess_distribution_capital_gain_cents, 0);
        assert_eq!(r.basis_after_distributions_cents, 12_000_000);
        assert_eq!(r.nondeductibles_applied_cents, 2_000_000);
        assert_eq!(r.nondeductibles_lost_or_suspended_cents, 0);
        assert_eq!(r.losses_allowed_cents, 4_000_000);
        assert_eq!(r.losses_suspended_carryforward_cents, 0);
        assert_eq!(r.ending_basis_cents, 6_000_000);
    }

    #[test]
    fn standard_loss_exceeds_basis_suspended() {
        let mut b = input();
        b.nonseparately_computed_loss_cents = 20_000_000; // $200K loss
                                                          // After increases: $150K. After dist: $120K. After nondeductibles: $100K.
                                                          // $200K loss → $100K allowed, $100K suspended.
        let r = compute(&b);
        assert_eq!(r.losses_allowed_cents, 10_000_000);
        assert_eq!(r.losses_suspended_carryforward_cents, 10_000_000);
        assert_eq!(r.ending_basis_cents, 0);
    }

    #[test]
    fn standard_nondeductibles_exceed_basis_lost_permanently() {
        let mut b = input();
        b.beginning_stock_basis_cents = 2_000_000; // $20K
        b.nonseparately_computed_income_cents = 0;
        b.distributions_cents = 0;
        b.noncapital_nondeductible_expenses_cents = 3_000_000; // $30K
        b.nonseparately_computed_loss_cents = 0;
        // Step1: $20K. Step2: $20K. Step3: $20K - $20K applied = $0; $10K LOST.
        let r = compute(&b);
        assert_eq!(r.nondeductibles_applied_cents, 2_000_000);
        assert_eq!(r.nondeductibles_lost_or_suspended_cents, 1_000_000);
        assert_eq!(r.ending_basis_cents, 0);
        // Losses_suspended should remain 0 (no losses input).
        assert_eq!(r.losses_suspended_carryforward_cents, 0);
    }

    #[test]
    fn standard_excess_distribution_capital_gain() {
        let mut b = input();
        b.beginning_stock_basis_cents = 1_000_000; // $10K
        b.nonseparately_computed_income_cents = 0;
        b.distributions_cents = 5_000_000; // $50K — way over basis
        b.noncapital_nondeductible_expenses_cents = 0;
        b.nonseparately_computed_loss_cents = 0;
        // Basis after increases: $10K. Distribution applied: $10K. Excess: $40K capital gain.
        let r = compute(&b);
        assert_eq!(r.distribution_applied_cents, 1_000_000);
        assert_eq!(r.excess_distribution_capital_gain_cents, 4_000_000);
        assert_eq!(r.ending_basis_cents, 0);
    }

    // ── § 1.1367-1(g) election alternative ordering ───────────

    #[test]
    fn election_loss_before_nondeductible_normal_flow() {
        let mut b = input();
        b.elects_alternative_ordering_g = true;
        // $100K + $50K = $150K, -$30K dist = $120K, -$40K loss = $80K, -$20K nondeductible = $60K.
        // Same ending basis as standard order in non-exceed case.
        let r = compute(&b);
        assert_eq!(r.ending_basis_cents, 6_000_000);
        assert_eq!(r.losses_allowed_cents, 4_000_000);
        assert_eq!(r.nondeductibles_applied_cents, 2_000_000);
    }

    #[test]
    fn election_nondeductibles_suspend_when_exceed() {
        let mut b = input();
        b.beginning_stock_basis_cents = 2_000_000; // $20K
        b.nonseparately_computed_income_cents = 0;
        b.distributions_cents = 0;
        b.nonseparately_computed_loss_cents = 0;
        b.noncapital_nondeductible_expenses_cents = 3_000_000; // $30K
        b.elects_alternative_ordering_g = true;
        // Basis: $20K. Election: losses first (0). Nondeductibles: $20K applied, $10K SUSPENDED.
        let r = compute(&b);
        assert_eq!(r.nondeductibles_applied_cents, 2_000_000);
        assert_eq!(r.nondeductibles_lost_or_suspended_cents, 1_000_000);
        assert_eq!(r.ending_basis_cents, 0);
    }

    #[test]
    fn election_changes_loss_allowed_when_basis_constrained() {
        // Beginning $10K, no income, no dist, $20K loss, $20K nondeductibles.
        // Standard: nondeductibles first → $10K applied (lost $10K), basis 0 → losses 0 allowed, $20K suspended.
        // Election: losses first → $10K applied, basis 0 → nondeductibles 0 applied, $20K suspended.
        let mut b = input();
        b.beginning_stock_basis_cents = 1_000_000;
        b.nonseparately_computed_income_cents = 0;
        b.distributions_cents = 0;
        b.nonseparately_computed_loss_cents = 2_000_000;
        b.noncapital_nondeductible_expenses_cents = 2_000_000;

        let r_standard = compute(&b);
        b.elects_alternative_ordering_g = true;
        let r_election = compute(&b);

        // Standard: nondeductibles consume basis first → loss allowed 0.
        assert_eq!(r_standard.nondeductibles_applied_cents, 1_000_000);
        assert_eq!(r_standard.nondeductibles_lost_or_suspended_cents, 1_000_000);
        assert_eq!(r_standard.losses_allowed_cents, 0);
        assert_eq!(r_standard.losses_suspended_carryforward_cents, 2_000_000);

        // Election: losses consume basis first → nondeductibles 0 applied.
        assert_eq!(r_election.losses_allowed_cents, 1_000_000);
        assert_eq!(r_election.losses_suspended_carryforward_cents, 1_000_000);
        assert_eq!(r_election.nondeductibles_applied_cents, 0);
        assert_eq!(r_election.nondeductibles_lost_or_suspended_cents, 2_000_000);

        // Ending basis the same.
        assert_eq!(r_standard.ending_basis_cents, 0);
        assert_eq!(r_election.ending_basis_cents, 0);
    }

    // ── Basis floor zero ──────────────────────────────────────

    #[test]
    fn basis_never_goes_below_zero() {
        let mut b = input();
        b.beginning_stock_basis_cents = 1_000_000;
        b.nonseparately_computed_income_cents = 0;
        b.distributions_cents = 0;
        b.nonseparately_computed_loss_cents = 10_000_000;
        b.noncapital_nondeductible_expenses_cents = 5_000_000;
        let r = compute(&b);
        assert_eq!(r.ending_basis_cents, 0);
    }

    // ── Multi-step ordering matters ───────────────────────────

    #[test]
    fn increases_applied_before_distributions_invariant() {
        // $0 beginning, $50K income, $30K distribution → after increases $50K → after dist $20K.
        let mut b = input();
        b.beginning_stock_basis_cents = 0;
        b.nonseparately_computed_income_cents = 5_000_000;
        b.distributions_cents = 3_000_000;
        b.nonseparately_computed_loss_cents = 0;
        b.noncapital_nondeductible_expenses_cents = 0;
        let r = compute(&b);
        assert_eq!(r.basis_after_increases_cents, 5_000_000);
        assert_eq!(r.distribution_applied_cents, 3_000_000);
        assert_eq!(r.basis_after_distributions_cents, 2_000_000);
    }

    #[test]
    fn distribution_applied_before_losses_invariant() {
        // Both orderings: distributions step 2; losses come after.
        let mut b = input();
        b.beginning_stock_basis_cents = 3_000_000;
        b.nonseparately_computed_income_cents = 0;
        b.distributions_cents = 1_000_000;
        b.nonseparately_computed_loss_cents = 5_000_000;
        b.noncapital_nondeductible_expenses_cents = 0;
        // Standard: $30K - $10K dist = $20K - $50K loss (suspended $30K) = $0 ending.
        let r = compute(&b);
        assert_eq!(r.distribution_applied_cents, 1_000_000);
        assert_eq!(r.losses_allowed_cents, 2_000_000);
        assert_eq!(r.losses_suspended_carryforward_cents, 3_000_000);
    }

    // ── Distribution capital gain math ────────────────────────

    #[test]
    fn distribution_within_basis_no_capital_gain() {
        let r = compute(&input());
        assert_eq!(r.excess_distribution_capital_gain_cents, 0);
    }

    #[test]
    fn distribution_above_basis_only_excess_is_gain() {
        let mut b = input();
        b.beginning_stock_basis_cents = 1_000_000;
        b.nonseparately_computed_income_cents = 0;
        b.distributions_cents = 1_500_000; // $15K dist; basis only $10K
        b.nonseparately_computed_loss_cents = 0;
        b.noncapital_nondeductible_expenses_cents = 0;
        let r = compute(&b);
        assert_eq!(r.excess_distribution_capital_gain_cents, 500_000);
        assert_eq!(r.distribution_applied_cents, 1_000_000);
    }

    // ── Citation and sibling note ─────────────────────────────

    #[test]
    fn citation_pins_all_subsections() {
        let r = compute(&input());
        assert!(r.citation.contains("§ 1367"));
        assert!(r.citation.contains("§ 1367(a)(1)"));
        assert!(r.citation.contains("§ 1367(a)(2)"));
        assert!(r.citation.contains("§ 1366(a)(1)"));
        assert!(r.citation.contains("§ 1366(d)"));
        assert!(r.citation.contains("§ 1366(d)(2)"));
        assert!(r.citation.contains("§ 1368"));
        assert!(r.citation.contains("§ 1368(b)(2)"));
        assert!(r.citation.contains("§ 1.1367-1(f)"));
        assert!(r.citation.contains("§ 1.1367-1(g)"));
        assert!(r.citation.contains("Form 7203"));
    }

    #[test]
    fn sibling_modules_note_present() {
        let r = compute(&input());
        assert!(
            r.notes.iter().any(|n| n.contains("§ 1361")
                && n.contains("§ 1366")
                && n.contains("§ 1368")
                && n.contains("§ 1374")
                && n.contains("Form 7203")),
            "sibling S-corp cluster note must reference § 1361 + § 1366 + § 1368 + § 1374 + Form 7203"
        );
    }

    // ── Defensive input clamping ───────────────────────────────

    #[test]
    fn defensive_negative_beginning_basis_clamped() {
        let mut b = input();
        b.beginning_stock_basis_cents = -1_000_000;
        let r = compute(&b);
        // Beginning clamped to 0; total flow proceeds.
        assert_eq!(r.beginning_basis_cents, 0);
    }

    #[test]
    fn defensive_negative_income_clamped() {
        let mut b = input();
        b.nonseparately_computed_income_cents = -1_000_000;
        let r = compute(&b);
        // Income clamped to 0; flow proceeds.
        assert!(r.basis_after_increases_cents >= r.beginning_basis_cents);
    }

    #[test]
    fn zero_inputs_zero_ending_basis() {
        let b = Section1367Input {
            beginning_stock_basis_cents: 0,
            separately_stated_income_cents: 0,
            nonseparately_computed_income_cents: 0,
            depletion_excess_over_basis_cents: 0,
            distributions_cents: 0,
            separately_stated_loss_cents: 0,
            nonseparately_computed_loss_cents: 0,
            noncapital_nondeductible_expenses_cents: 0,
            depletion_deduction_cents: 0,
            elects_alternative_ordering_g: false,
        };
        let r = compute(&b);
        assert_eq!(r.ending_basis_cents, 0);
        assert_eq!(r.losses_allowed_cents, 0);
        assert_eq!(r.excess_distribution_capital_gain_cents, 0);
    }

    // ── Multi-regime invariants ───────────────────────────────

    #[test]
    fn standard_vs_election_same_ending_when_basis_unconstrained() {
        // When basis covers all decreases, standard vs election produce same result.
        let mut b = input();
        b.beginning_stock_basis_cents = 100_000_000; // huge basis
        let r_standard = compute(&b);
        b.elects_alternative_ordering_g = true;
        let r_election = compute(&b);
        assert_eq!(r_standard.ending_basis_cents, r_election.ending_basis_cents);
        assert_eq!(
            r_standard.losses_allowed_cents,
            r_election.losses_allowed_cents
        );
        assert_eq!(
            r_standard.nondeductibles_applied_cents,
            r_election.nondeductibles_applied_cents
        );
    }

    #[test]
    fn separately_stated_combines_with_nonseparately_income() {
        let mut b = input();
        b.separately_stated_income_cents = 1_000_000; // $10K
        b.nonseparately_computed_income_cents = 5_000_000; // $50K
        let r = compute(&b);
        assert_eq!(r.total_increases_cents, 6_000_000);
    }

    #[test]
    fn separately_stated_loss_combines_with_nonseparately_loss() {
        let mut b = input();
        b.separately_stated_loss_cents = 1_000_000;
        b.nonseparately_computed_loss_cents = 4_000_000;
        // Total loss = $50K. Basis after dist+nondeductibles = $100K. All allowed.
        let r = compute(&b);
        assert_eq!(r.losses_allowed_cents, 5_000_000);
    }

    #[test]
    fn depletion_deduction_combines_with_nondeductibles_for_basis_decrease() {
        // Both § 1367(a)(2)(D) noncapital nondeductibles AND § 1367(a)(2)(E) depletion
        // are applied at the same step in this implementation.
        let mut b = input();
        b.noncapital_nondeductible_expenses_cents = 1_000_000;
        b.depletion_deduction_cents = 500_000;
        // Total step-3 decrease = $15K.
        let r = compute(&b);
        assert_eq!(r.nondeductibles_applied_cents, 1_500_000);
    }
}
