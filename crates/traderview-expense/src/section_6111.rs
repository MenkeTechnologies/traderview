//! IRC § 6111 — Material advisor disclosure (Form 8918).
//!
//! Direct sibling to § 6011 (taxpayer disclosure side) and
//! § 6707A (taxpayer penalty side). § 6111 governs the ADVISOR
//! side of the reportable-transaction disclosure regime: any
//! person providing material aid, assistance, or advice with
//! respect to organizing, managing, promoting, selling,
//! implementing, insuring, or carrying out a reportable
//! transaction AND who derives gross income above the
//! statutory threshold must file Form 8918 with the IRS Office
//! of Tax Shelter Analysis (OTSA).
//!
//! § 6111(a) FILING REQUIREMENT: Each material advisor with
//! respect to any reportable transaction must make a return
//! setting forth (1) information identifying and describing the
//! transaction, (2) the expected tax treatment and all potential
//! tax benefits, and (3) any tax-result protection arrangements.
//!
//! § 6111(b)(1) MATERIAL ADVISOR — two-prong test, BOTH required:
//!   (A) provides material aid, assistance, or advice with
//!       respect to organizing, managing, promoting, selling,
//!       implementing, insuring, or carrying out any reportable
//!       transaction; AND
//!   (B) directly or indirectly derives gross income in excess
//!       of the threshold amount for such aid, assistance, or
//!       advice (defined in Notice 2004-80 / Treas. Reg.
//!       § 301.6111-3(b)(3)).
//!
//! GROSS INCOME THRESHOLDS (per Treas. Reg. § 301.6111-3(b)(3)):
//!   - $50,000 if the reportable transaction provides
//!     substantially all of the tax benefits to NATURAL PERSONS
//!     (looking through partnerships, S corporations, trusts).
//!   - $250,000 for ALL OTHER reportable transactions.
//!
//! FILING DEADLINE (Treas. Reg. § 301.6111-3(e)): Form 8918 must
//! be filed with OTSA by the LAST DAY OF THE MONTH that follows
//! the end of the calendar quarter in which the advisor became
//! a material advisor with respect to the reportable transaction.
//!
//! § 6707 PENALTY for failure to disclose:
//!   - REPORTABLE TRANSACTION (non-listed): $50,000 per failure.
//!   - LISTED TRANSACTION: greater of $200,000 OR 50% of the
//!     material advisor's gross income from the transaction.
//!   - UNINTENTIONAL listed-transaction failures reduced to
//!     non-listed $50,000 amount (§ 6707(b)(1) flush — "the
//!     higher penalty for listed transactions will not apply
//!     to any material advisor whose failure ... was
//!     unintentional"). Subsequent true and complete filing
//!     prior to IRS contact or taxpayer Form 8886 filing
//!     establishes unintentionality.
//!
//! § 6707 STATUTE OF LIMITATIONS: 3 years from Form 8918 filing;
//! UNLIMITED if no return filed (Treas. Reg. § 301.6707-1).
//!
//! Citations: 26 U.S.C. § 6111 (general material advisor
//! disclosure); 26 U.S.C. § 6111(b)(1) (two-prong material advisor
//! definition); 26 U.S.C. § 6111(b)(2) (advisor list maintenance —
//! cross to § 6112); 26 CFR § 301.6111-3(b)(3) (gross income
//! thresholds); 26 CFR § 301.6111-3(e) (quarterly filing
//! deadline); 26 U.S.C. § 6707 (advisor failure-to-disclose
//! penalty); 26 CFR § 301.6707-1 (penalty regulations); Notice
//! 2004-80 (threshold guidance); Form 8918 instructions. Sibling
//! modules: § 6011 (taxpayer disclosure — Form 8886); § 6707A
//! (taxpayer penalty); § 6112 (advisor list maintenance);
//! § 6662A (reportable-transaction-understatement penalty on
//! the underlying tax).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    /// Treas. Reg. § 1.6011-4(b)(2) — IRS-listed transaction.
    /// Triggers § 6707 listed-transaction penalty tier.
    ListedTransaction,
    /// Treas. Reg. § 1.6011-4(b)(3)–(6) — confidential,
    /// contractual protection, loss, or transaction-of-interest.
    /// Triggers § 6707 non-listed $50K penalty.
    ReportableTransactionNonListed,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AdvisorStatus {
    /// § 6111(b)(1) two-prong test satisfied — material advisor
    /// with filing obligation.
    MaterialAdvisor,
    /// Either material-aid prong (A) failed OR gross-income
    /// threshold prong (B) failed — not a material advisor.
    NotMaterialAdvisor,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6111Input {
    pub transaction_type: TransactionType,
    /// § 6111(b)(1)(A) prong — provided material aid, assistance,
    /// or advice with respect to the reportable transaction.
    pub provided_material_aid: bool,
    /// § 6111(b)(1)(B) prong — gross income derived from the
    /// transaction (cents).
    pub gross_income_from_transaction_cents: i64,
    /// True if substantially all tax benefits of the transaction
    /// flow to natural persons (lower $50K threshold applies).
    pub substantially_all_tax_benefits_to_individuals: bool,
    /// Whether Form 8918 has been filed for this transaction.
    pub form_8918_filed: bool,
    /// Days late after the quarter-end filing deadline (0 = on
    /// time; positive = late filing or never filed).
    pub days_late_after_quarter_end: i64,
    /// § 6707 unintentionality reduction — true if the failure
    /// was unintentional (e.g., subsequent true return filed
    /// before IRS contact or taxpayer Form 8886 filing).
    pub failure_was_unintentional: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6111Result {
    pub material_advisor_status: AdvisorStatus,
    pub filing_required: bool,
    /// True if § 6111(b)(1)(B) gross-income threshold is met.
    pub income_threshold_met: bool,
    /// Applicable threshold amount (cents) — $50K natural-person
    /// or $250K other.
    pub gross_income_threshold_cents: i64,
    /// § 6707 penalty exposure (cents) if filing required and
    /// not made.
    pub section_6707_penalty_cents: i64,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// Treas. Reg. § 301.6111-3(b)(3)(i) — natural-person threshold.
pub const THRESHOLD_NATURAL_PERSON_CENTS: i64 = 5_000_000;
/// Treas. Reg. § 301.6111-3(b)(3)(ii) — other-transaction threshold.
pub const THRESHOLD_OTHER_CENTS: i64 = 25_000_000;
/// § 6707(a) penalty — reportable transaction (non-listed).
pub const PENALTY_NON_LISTED_CENTS: i64 = 5_000_000;
/// § 6707(b)(1)(A) minimum listed-transaction penalty.
pub const PENALTY_LISTED_FLOOR_CENTS: i64 = 20_000_000;

pub fn compute(input: &Section6111Input) -> Section6111Result {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    let gross_income = input.gross_income_from_transaction_cents.max(0);

    // § 6111(b)(1)(B) — gross-income threshold determination.
    let threshold = if input.substantially_all_tax_benefits_to_individuals {
        THRESHOLD_NATURAL_PERSON_CENTS
    } else {
        THRESHOLD_OTHER_CENTS
    };
    let income_threshold_met = gross_income >= threshold;

    // § 6111(b)(1) — both prongs must satisfy for material advisor.
    let is_material_advisor = input.provided_material_aid && income_threshold_met;
    let material_advisor_status = if is_material_advisor {
        AdvisorStatus::MaterialAdvisor
    } else {
        AdvisorStatus::NotMaterialAdvisor
    };
    let filing_required = is_material_advisor;

    // § 6707 penalty calculation.
    let raw_50_percent = gross_income / 2;
    let penalty =
        if filing_required && (!input.form_8918_filed || input.days_late_after_quarter_end > 0) {
            match input.transaction_type {
                TransactionType::ListedTransaction => {
                    if input.failure_was_unintentional {
                        PENALTY_NON_LISTED_CENTS
                    } else {
                        raw_50_percent.max(PENALTY_LISTED_FLOOR_CENTS)
                    }
                }
                TransactionType::ReportableTransactionNonListed => PENALTY_NON_LISTED_CENTS,
            }
        } else {
            0
        };

    if filing_required && !input.form_8918_filed {
        violations.push(format!(
            "§ 6111(a) + Treas. Reg. § 301.6111-3(e) — Form 8918 required for material \
             advisor on {:?} but not filed. § 6707 penalty exposure: {} cents.",
            input.transaction_type, penalty,
        ));
    } else if filing_required && input.days_late_after_quarter_end > 0 {
        violations.push(format!(
            "§ 6111(a) + Treas. Reg. § 301.6111-3(e) — Form 8918 filed {} days late after \
             quarter-end deadline. § 6707 penalty exposure: {} cents.",
            input.days_late_after_quarter_end, penalty,
        ));
    }

    // Status notes.
    if is_material_advisor {
        notes.push(format!(
            "§ 6111(b)(1) MATERIAL ADVISOR — both prongs satisfied: (A) provided material \
             aid/assistance/advice for reportable transaction; (B) gross income {} cents \
             meets threshold {} cents ({} category). Form 8918 must be filed by last day \
             of month following quarter-end in which material-advisor status engaged \
             (Treas. Reg. § 301.6111-3(e)).",
            gross_income,
            threshold,
            if input.substantially_all_tax_benefits_to_individuals {
                "natural-person — $50K"
            } else {
                "other transactions — $250K"
            },
        ));
    } else {
        notes.push(format!(
            "§ 6111(b)(1) — NOT material advisor: {}. No Form 8918 filing required.",
            if !input.provided_material_aid {
                "prong (A) failed — no material aid/assistance/advice provided"
            } else {
                "prong (B) failed — gross income below applicable threshold"
            },
        ));
    }

    // Penalty-tier explanation.
    match input.transaction_type {
        TransactionType::ListedTransaction => {
            notes.push(format!(
                "§ 6707(b)(1)(A) — LISTED TRANSACTION penalty: greater of $200,000 OR 50% \
                 of gross income from transaction. 50% of {} cents = {} cents; floor at \
                 {} cents. Unintentional-failure reduction (§ 6707(b)(1) flush): {}.",
                gross_income,
                raw_50_percent,
                PENALTY_LISTED_FLOOR_CENTS,
                if input.failure_was_unintentional {
                    "ENGAGED — penalty reduced to $50K non-listed amount"
                } else {
                    "NOT ENGAGED — full listed-transaction penalty applies"
                },
            ));
        }
        TransactionType::ReportableTransactionNonListed => {
            notes.push(
                "§ 6707(a) — NON-LISTED reportable transaction penalty: flat $50,000 \
                 per failure. Per-transaction assessment under Sec. 6707 Material Adviser \
                 Penalty regulations — Treas. Reg. § 301.6707-1."
                    .to_string(),
            );
        }
    }

    notes.push(
        "Sibling modules: § 6011 (taxpayer-side disclosure — Form 8886); § 6707A \
         (taxpayer-side penalty — 75% of tax reduction capped); § 6112 (advisor list \
         maintenance — § 6111(b)(2) cross-reference); § 6662A (reportable-transaction-\
         understatement 20%/30% accuracy penalty on the underlying tax). § 6707 statute \
         of limitations: 3 years from Form 8918 filing; UNLIMITED if no return filed."
            .to_string(),
    );

    Section6111Result {
        material_advisor_status,
        filing_required,
        income_threshold_met,
        gross_income_threshold_cents: threshold,
        section_6707_penalty_cents: penalty,
        compliant: violations.is_empty(),
        violations,
        citation: "26 U.S.C. § 6111 (general material advisor disclosure); 26 U.S.C. \
                   § 6111(b)(1)(A)–(B) (two-prong material advisor definition); \
                   26 U.S.C. § 6111(b)(2) (advisor list cross-reference to § 6112); \
                   Treas. Reg. § 301.6111-3(b)(3) (gross income thresholds — $50K \
                   natural-person / $250K other); Treas. Reg. § 301.6111-3(e) (quarterly \
                   filing deadline); 26 U.S.C. § 6707(a) (non-listed $50K penalty); \
                   26 U.S.C. § 6707(b)(1)(A) (listed greater-of-$200K-or-50%-gross); \
                   26 U.S.C. § 6707(b)(1) flush (unintentional-failure reduction); \
                   Treas. Reg. § 301.6707-1 (penalty regulations); Notice 2004-80 \
                   (threshold guidance); Form 8918 (Material Advisor Disclosure \
                   Statement)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        transaction_type: TransactionType,
        material_aid: bool,
        gross_income: i64,
        individuals: bool,
        filed: bool,
        days_late: i64,
        unintentional: bool,
    ) -> Section6111Input {
        Section6111Input {
            transaction_type,
            provided_material_aid: material_aid,
            gross_income_from_transaction_cents: gross_income,
            substantially_all_tax_benefits_to_individuals: individuals,
            form_8918_filed: filed,
            days_late_after_quarter_end: days_late,
            failure_was_unintentional: unintentional,
        }
    }

    // ── Material advisor classification ────────────────────────

    #[test]
    fn natural_person_threshold_at_50k_engaged() {
        let r = compute(&input(
            TransactionType::ReportableTransactionNonListed,
            true,
            5_000_000,
            true,
            true,
            0,
            false,
        ));
        assert_eq!(r.material_advisor_status, AdvisorStatus::MaterialAdvisor);
        assert!(r.income_threshold_met);
        assert!(r.filing_required);
    }

    #[test]
    fn natural_person_below_50k_not_material_advisor() {
        let r = compute(&input(
            TransactionType::ReportableTransactionNonListed,
            true,
            4_999_999,
            true,
            false,
            0,
            false,
        ));
        assert_eq!(r.material_advisor_status, AdvisorStatus::NotMaterialAdvisor);
        assert!(!r.income_threshold_met);
        assert!(!r.filing_required);
        assert!(r.compliant);
    }

    #[test]
    fn other_threshold_at_250k_engaged() {
        let r = compute(&input(
            TransactionType::ReportableTransactionNonListed,
            true,
            25_000_000,
            false,
            true,
            0,
            false,
        ));
        assert_eq!(r.material_advisor_status, AdvisorStatus::MaterialAdvisor);
        assert!(r.income_threshold_met);
        assert_eq!(r.gross_income_threshold_cents, THRESHOLD_OTHER_CENTS);
    }

    #[test]
    fn other_below_250k_not_material_advisor() {
        let r = compute(&input(
            TransactionType::ReportableTransactionNonListed,
            true,
            24_999_999,
            false,
            false,
            0,
            false,
        ));
        assert_eq!(r.material_advisor_status, AdvisorStatus::NotMaterialAdvisor);
        assert!(!r.income_threshold_met);
    }

    #[test]
    fn prong_a_failed_no_material_aid_not_advisor_regardless_of_income() {
        let r = compute(&input(
            TransactionType::ListedTransaction,
            false,
            100_000_000, // $1M — way above any threshold
            true,
            false,
            0,
            false,
        ));
        assert_eq!(r.material_advisor_status, AdvisorStatus::NotMaterialAdvisor);
        assert!(!r.filing_required);
        assert!(r.compliant);
    }

    // ── § 6707 penalty — non-listed reportable ─────────────────

    #[test]
    fn non_listed_not_filed_50k_penalty() {
        let r = compute(&input(
            TransactionType::ReportableTransactionNonListed,
            true,
            10_000_000,
            true,
            false,
            0,
            false,
        ));
        assert!(!r.compliant);
        assert_eq!(r.section_6707_penalty_cents, PENALTY_NON_LISTED_CENTS);
    }

    #[test]
    fn non_listed_filed_late_50k_penalty() {
        let r = compute(&input(
            TransactionType::ReportableTransactionNonListed,
            true,
            10_000_000,
            true,
            true,
            30,
            false,
        ));
        assert!(!r.compliant);
        assert_eq!(r.section_6707_penalty_cents, PENALTY_NON_LISTED_CENTS);
    }

    #[test]
    fn non_listed_filed_on_time_compliant() {
        let r = compute(&input(
            TransactionType::ReportableTransactionNonListed,
            true,
            10_000_000,
            true,
            true,
            0,
            false,
        ));
        assert!(r.compliant);
        assert_eq!(r.section_6707_penalty_cents, 0);
    }

    // ── § 6707 penalty — listed transaction ────────────────────

    #[test]
    fn listed_not_filed_intentional_200k_floor() {
        let r = compute(&input(
            TransactionType::ListedTransaction,
            true,
            10_000_000, // 50% = 5M ; floor 20M wins
            true,
            false,
            0,
            false,
        ));
        assert_eq!(r.section_6707_penalty_cents, PENALTY_LISTED_FLOOR_CENTS);
    }

    #[test]
    fn listed_not_filed_50_percent_gross_exceeds_200k_floor() {
        // Gross income $1M cents = 100M cents → 50% = 50M cents > 20M floor.
        let r = compute(&input(
            TransactionType::ListedTransaction,
            true,
            100_000_000,
            true,
            false,
            0,
            false,
        ));
        assert_eq!(r.section_6707_penalty_cents, 50_000_000);
    }

    #[test]
    fn listed_not_filed_unintentional_reduced_to_50k() {
        let r = compute(&input(
            TransactionType::ListedTransaction,
            true,
            100_000_000,
            true,
            false,
            0,
            true,
        ));
        assert_eq!(r.section_6707_penalty_cents, PENALTY_NON_LISTED_CENTS);
    }

    #[test]
    fn listed_filed_on_time_compliant() {
        let r = compute(&input(
            TransactionType::ListedTransaction,
            true,
            100_000_000,
            true,
            true,
            0,
            false,
        ));
        assert!(r.compliant);
        assert_eq!(r.section_6707_penalty_cents, 0);
    }

    // ── Multi-regime invariants ────────────────────────────────

    #[test]
    fn natural_person_threshold_lower_than_other_invariant() {
        assert!(THRESHOLD_NATURAL_PERSON_CENTS < THRESHOLD_OTHER_CENTS);
        assert_eq!(THRESHOLD_OTHER_CENTS, 5 * THRESHOLD_NATURAL_PERSON_CENTS);
    }

    #[test]
    fn listed_penalty_floor_above_non_listed_penalty_invariant() {
        assert!(PENALTY_LISTED_FLOOR_CENTS > PENALTY_NON_LISTED_CENTS);
        assert_eq!(PENALTY_LISTED_FLOOR_CENTS, 4 * PENALTY_NON_LISTED_CENTS);
    }

    #[test]
    fn listed_50_percent_gross_floor_truth_table() {
        // 5-cell truth table: gross income vs penalty.
        let cells = [
            (10_000_000_i64, PENALTY_LISTED_FLOOR_CENTS), // 50% = 5M; floor wins
            (40_000_000, PENALTY_LISTED_FLOOR_CENTS),     // 50% = 20M; ties floor
            (40_000_001, 20_000_000),                     // 50% = 20.00M; floor still wins via max
            (50_000_000, 25_000_000),                     // 50% = 25M; gross wins
            (200_000_000, 100_000_000),                   // 50% = 100M; gross wins
        ];
        for (gross, expected) in cells.iter() {
            let r = compute(&input(
                TransactionType::ListedTransaction,
                true,
                *gross,
                true,
                false,
                0,
                false,
            ));
            // The max(50%, floor) calculation.
            let computed_50 = gross / 2;
            let want = computed_50.max(PENALTY_LISTED_FLOOR_CENTS);
            assert_eq!(want, *expected, "gross={}", gross);
            assert_eq!(r.section_6707_penalty_cents, *expected);
        }
    }

    #[test]
    fn both_prongs_required_invariant() {
        // 4-cell truth table for material advisor status.
        let cells = [
            (true, true, AdvisorStatus::MaterialAdvisor), // both prongs
            (true, false, AdvisorStatus::NotMaterialAdvisor), // only A
            (false, true, AdvisorStatus::NotMaterialAdvisor), // only B
            (false, false, AdvisorStatus::NotMaterialAdvisor), // neither
        ];
        for (prong_a, prong_b_met, expected) in cells.iter() {
            let gross = if *prong_b_met { 25_000_000 } else { 0 };
            let r = compute(&input(
                TransactionType::ReportableTransactionNonListed,
                *prong_a,
                gross,
                false,
                false,
                0,
                false,
            ));
            assert_eq!(
                r.material_advisor_status, *expected,
                "A={} B={}",
                prong_a, prong_b_met
            );
        }
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = compute(&input(
            TransactionType::ListedTransaction,
            true,
            100_000_000,
            true,
            true,
            0,
            false,
        ));
        assert!(r.citation.contains("§ 6111"));
        assert!(r.citation.contains("§ 6111(b)(1)(A)"));
        assert!(r.citation.contains("§ 6111(b)(2)"));
        assert!(r.citation.contains("§ 301.6111-3(b)(3)"));
        assert!(r.citation.contains("§ 301.6111-3(e)"));
        assert!(r.citation.contains("§ 6707(a)"));
        assert!(r.citation.contains("§ 6707(b)(1)(A)"));
        assert!(r.citation.contains("§ 6707(b)(1) flush"));
        assert!(r.citation.contains("§ 301.6707-1"));
        assert!(r.citation.contains("Notice 2004-80"));
        assert!(r.citation.contains("Form 8918"));
    }

    #[test]
    fn sibling_modules_note_present() {
        let r = compute(&input(
            TransactionType::ListedTransaction,
            true,
            100_000_000,
            true,
            true,
            0,
            false,
        ));
        assert!(
            r.notes.iter().any(|n| n.contains("§ 6011")
                && n.contains("§ 6707A")
                && n.contains("§ 6112")
                && n.contains("§ 6662A")),
            "sibling-modules trio note must be present"
        );
    }

    #[test]
    fn defensive_negative_gross_income_clamped() {
        let r = compute(&input(
            TransactionType::ListedTransaction,
            true,
            -1_000_000,
            true,
            false,
            0,
            false,
        ));
        // Negative clamps to 0; threshold not met.
        assert!(!r.income_threshold_met);
        assert_eq!(r.material_advisor_status, AdvisorStatus::NotMaterialAdvisor);
        assert_eq!(r.section_6707_penalty_cents, 0);
    }

    #[test]
    fn threshold_boundary_natural_person_exact_50k() {
        // At exactly $50K = met (≥ comparison).
        let r = compute(&input(
            TransactionType::ReportableTransactionNonListed,
            true,
            THRESHOLD_NATURAL_PERSON_CENTS,
            true,
            true,
            0,
            false,
        ));
        assert!(r.income_threshold_met);
    }

    #[test]
    fn threshold_boundary_natural_person_one_cent_below() {
        let r = compute(&input(
            TransactionType::ReportableTransactionNonListed,
            true,
            THRESHOLD_NATURAL_PERSON_CENTS - 1,
            true,
            false,
            0,
            false,
        ));
        assert!(!r.income_threshold_met);
    }

    #[test]
    fn threshold_boundary_other_exact_250k() {
        let r = compute(&input(
            TransactionType::ReportableTransactionNonListed,
            true,
            THRESHOLD_OTHER_CENTS,
            false,
            true,
            0,
            false,
        ));
        assert!(r.income_threshold_met);
    }

    #[test]
    fn threshold_boundary_other_one_cent_below() {
        let r = compute(&input(
            TransactionType::ReportableTransactionNonListed,
            true,
            THRESHOLD_OTHER_CENTS - 1,
            false,
            false,
            0,
            false,
        ));
        assert!(!r.income_threshold_met);
    }

    #[test]
    fn natural_person_uses_lower_threshold_invariant() {
        // Same gross income — natural-person path engages; other-path doesn't.
        let gross = 10_000_000_i64; // $100K
        let ind = compute(&input(
            TransactionType::ReportableTransactionNonListed,
            true,
            gross,
            true,
            true,
            0,
            false,
        ));
        let other = compute(&input(
            TransactionType::ReportableTransactionNonListed,
            true,
            gross,
            false,
            false,
            0,
            false,
        ));
        assert_eq!(ind.material_advisor_status, AdvisorStatus::MaterialAdvisor);
        assert_eq!(
            other.material_advisor_status,
            AdvisorStatus::NotMaterialAdvisor
        );
    }

    #[test]
    fn unintentional_only_affects_listed_transactions() {
        // For non-listed, unintentionality has no effect (already $50K).
        let intentional = compute(&input(
            TransactionType::ReportableTransactionNonListed,
            true,
            100_000_000,
            true,
            false,
            0,
            false,
        ));
        let unintentional = compute(&input(
            TransactionType::ReportableTransactionNonListed,
            true,
            100_000_000,
            true,
            false,
            0,
            true,
        ));
        assert_eq!(
            intentional.section_6707_penalty_cents,
            PENALTY_NON_LISTED_CENTS
        );
        assert_eq!(
            unintentional.section_6707_penalty_cents,
            PENALTY_NON_LISTED_CENTS
        );
        // For listed, unintentionality reduces from listed tier to non-listed.
        let listed_intentional = compute(&input(
            TransactionType::ListedTransaction,
            true,
            100_000_000,
            true,
            false,
            0,
            false,
        ));
        let listed_unintentional = compute(&input(
            TransactionType::ListedTransaction,
            true,
            100_000_000,
            true,
            false,
            0,
            true,
        ));
        assert_eq!(listed_intentional.section_6707_penalty_cents, 50_000_000);
        assert_eq!(
            listed_unintentional.section_6707_penalty_cents,
            PENALTY_NON_LISTED_CENTS
        );
        assert!(
            listed_intentional.section_6707_penalty_cents
                > listed_unintentional.section_6707_penalty_cents
        );
    }
}
