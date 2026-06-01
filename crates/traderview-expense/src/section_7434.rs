//! IRC § 7434 — Civil damages for fraudulent filing of
//! information returns. Trader-relevant CIVIL remedy when a
//! third party (employer, broker, payor) willfully files a
//! fraudulent W-2, 1099, or other information return against
//! the taxpayer. Distinct from criminal statutes (`section_7201`
//! evasion, `section_7202` collection failure, `section_7203`
//! failure to file, `section_7206` perjury), civil fraud
//! (`section_6663`), and TFRP (`section_6672`) — § 7434 is a
//! VICTIM'S civil cause of action, not a government enforcement
//! tool.
//!
//! Trader-relevant scenarios:
//! - Broker files incorrect 1099-B inflating proceeds
//! - Employer files false W-2 inflating wages
//! - Payor files fake 1099-NEC inflating payments
//! - Former employer files retaliatory false W-2 / 1099 against
//!   departing trader
//!
//! **Statutory elements** (preponderance of evidence civil
//! burden):
//!
//! 1. Any PERSON (defendant)
//! 2. WILLFULLY FILED a fraudulent information return
//! 3. Information return was FRAUDULENT
//! 4. With respect to payments purported to be made to any
//!    other person (the plaintiff-taxpayer)
//!
//! **Statutory damages — greater of**:
//!
//! - $5,000 STATUTORY MINIMUM, OR
//! - SUM OF ACTUAL DAMAGES incurred
//!
//! PLUS:
//! - Court costs
//! - At court's discretion, reasonable attorney fees
//!
//! **Derolf misclassification carveout** (Derolf v. Risinger
//! Bros., Eighth Circuit / various district courts): § 7434
//! reaches information returns that intentionally MISSTATE THE
//! AMOUNT of income. Most courts hold that misclassification as
//! INDEPENDENT CONTRACTOR (1099-MISC / 1099-NEC instead of W-2)
//! does NOT give a cause of action under § 7434 when the dollar
//! amount is otherwise correct. Plaintiff must allege FRAUDULENT
//! AMOUNT MISSTATEMENT, not merely wrong form classification.
//!
//! **Statute of limitations — § 7434(d)**: civil action may be
//! brought within the LATER of:
//!
//! - 6 YEARS after the date of filing of the fraudulent
//!   information return, OR
//! - 1 YEAR after the date the fraudulent information return
//!   would have been discovered by the exercise of reasonable
//!   care
//!
//! **Trader-tax-defense leverage**: § 7434 victory provides
//! civil recovery + court order on information return falsity
//! that helps re-trace IRS deficiency notices arising from the
//! fraudulent 1099 / W-2. Civil judgment may be used as
//! collateral estoppel in later Tax Court proceedings.
//!
//! Citations: IRC § 7434(a) (cause of action); § 7434(b)
//! (damages — greater of $5,000 or actual + court costs +
//! discretionary attorney fees); § 7434(c) (requirements);
//! § 7434(d) (statute of limitations — later of 6 years from
//! filing OR 1 year from reasonable discovery); § 7434(e)
//! (notice to Secretary); § 7434(f) (definitions); Derolf v.
//! Risinger Bros. (8th Cir.) misclassification carveout.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InformationReturnType {
    W2,
    Form1099B,
    Form1099Nec,
    Form1099Misc,
    Form1099Div,
    Form1099Int,
    Other,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7434Input {
    pub return_type: InformationReturnType,
    /// Whether the defendant WILLFULLY filed an information
    /// return with respect to the plaintiff.
    pub person_willfully_filed_information_return: bool,
    /// Whether the information return was FRAUDULENT (intentional
    /// misstatement).
    pub return_was_fraudulent: bool,
    /// Whether the fraudulent return involves intentional
    /// MISSTATEMENT of the payment amount (vs mere
    /// misclassification of worker/payment type).
    pub fraudulent_misstatement_of_payment_amount: bool,
    /// Whether the only allegation is misclassification (1099
    /// when should be W-2) without amount misstatement — Derolf
    /// carveout defeats claim.
    pub misclassification_only_no_amount_misstatement: bool,
    /// Plaintiff's actual damages in cents (subject to greater-of
    /// rule with $5,000 statutory minimum).
    pub actual_damages_cents: i64,
    /// Court costs in cents.
    pub court_costs_cents: i64,
    /// Whether the court has exercised discretion to award
    /// reasonable attorney fees.
    pub attorney_fees_awarded_by_court_discretion: bool,
    /// Attorney fees amount in cents (if awarded).
    pub attorney_fees_amount_cents: i64,
    /// Years since the fraudulent information return was filed.
    pub years_since_fraudulent_return_filed: u32,
    /// Years since plaintiff would have discovered the
    /// fraudulent return by exercise of reasonable care.
    pub years_since_reasonable_discovery: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7434Result {
    pub civil_action_authorized: bool,
    pub statutory_damages_cents: i64,
    pub court_costs_recoverable_cents: i64,
    pub attorney_fees_recoverable_cents: i64,
    pub total_recovery_cents: i64,
    pub derolf_misclassification_carveout_engaged: bool,
    pub statute_of_limitations_satisfied: bool,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section7434Input) -> Section7434Result {
    let mut notes: Vec<String> = Vec::new();

    notes.push(
        "§ 7434(a) — civil action against person who willfully files fraudulent information return with respect to payments purported to be made to any other person"
            .to_string(),
    );

    notes.push(format!(
        "§ 7434 — claim involves {} information return",
        match input.return_type {
            InformationReturnType::W2 => "W-2",
            InformationReturnType::Form1099B => "1099-B (broker proceeds)",
            InformationReturnType::Form1099Nec => "1099-NEC (nonemployee compensation)",
            InformationReturnType::Form1099Misc => "1099-MISC",
            InformationReturnType::Form1099Div => "1099-DIV",
            InformationReturnType::Form1099Int => "1099-INT",
            InformationReturnType::Other => "other",
        }
    ));

    let derolf_carveout = input.misclassification_only_no_amount_misstatement
        && !input.fraudulent_misstatement_of_payment_amount;

    if derolf_carveout {
        notes.push(
            "Derolf v. Risinger Bros. — most courts hold misclassification as independent contractor (1099 instead of W-2) does NOT give § 7434 cause of action when dollar amount is otherwise correct; plaintiff must allege FRAUDULENT AMOUNT MISSTATEMENT, not merely wrong form classification"
                .to_string(),
        );
    }

    // § 7434(d) statute of limitations — later of 6 years from
    // filing OR 1 year from reasonable discovery.
    let within_6_year_filing = input.years_since_fraudulent_return_filed < 6;
    let within_1_year_discovery = input.years_since_reasonable_discovery < 1;
    let sol_satisfied = within_6_year_filing || within_1_year_discovery;

    notes.push(format!(
        "§ 7434(d) statute of limitations: later of 6 years from filing OR 1 year from reasonable discovery; {} years since filing + {} years since discovery → SOL {}",
        input.years_since_fraudulent_return_filed,
        input.years_since_reasonable_discovery,
        if sol_satisfied {
            "SATISFIED"
        } else {
            "EXPIRED"
        }
    ));

    let civil_action_authorized = input.person_willfully_filed_information_return
        && input.return_was_fraudulent
        && !derolf_carveout
        && sol_satisfied;

    let statutory_minimum = 500_000i64;
    let actual_damages = input.actual_damages_cents.max(0);
    let statutory_damages = if civil_action_authorized {
        actual_damages.max(statutory_minimum)
    } else {
        0
    };

    let court_costs = if civil_action_authorized {
        input.court_costs_cents.max(0)
    } else {
        0
    };

    let attorney_fees = if civil_action_authorized
        && input.attorney_fees_awarded_by_court_discretion
    {
        input.attorney_fees_amount_cents.max(0)
    } else {
        0
    };

    let total_recovery = statutory_damages
        .saturating_add(court_costs)
        .saturating_add(attorney_fees);

    notes.push(format!(
        "§ 7434(b) damages: greater of $5,000 (statutory minimum) OR actual damages (${}); + court costs (${}); + discretionary attorney fees (${})",
        actual_damages / 100,
        court_costs / 100,
        attorney_fees / 100
    ));

    if civil_action_authorized {
        notes.push(format!(
            "§ 7434 — civil action AUTHORIZED; total recovery: ${} (statutory damages) + ${} (court costs) + ${} (attorney fees) = ${} total",
            statutory_damages / 100,
            court_costs / 100,
            attorney_fees / 100,
            total_recovery / 100
        ));
    }

    notes.push(
        "§ 7434(e) — plaintiff must provide copy of complaint to Secretary (IRS notice) upon filing civil action"
            .to_string(),
    );

    notes.push(
        "§ 7434 victory — civil judgment provides collateral-estoppel leverage in any subsequent Tax Court / refund litigation arising from the fraudulent 1099 / W-2 deficiency notice"
            .to_string(),
    );

    Section7434Result {
        civil_action_authorized,
        statutory_damages_cents: statutory_damages,
        court_costs_recoverable_cents: court_costs,
        attorney_fees_recoverable_cents: attorney_fees,
        total_recovery_cents: total_recovery,
        derolf_misclassification_carveout_engaged: derolf_carveout,
        statute_of_limitations_satisfied: sol_satisfied,
        citation: "IRC §§ 7434(a), 7434(b), 7434(c), 7434(d), 7434(e), 7434(f); Derolf v. Risinger Bros. misclassification carveout",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full_claim() -> Section7434Input {
        Section7434Input {
            return_type: InformationReturnType::Form1099B,
            person_willfully_filed_information_return: true,
            return_was_fraudulent: true,
            fraudulent_misstatement_of_payment_amount: true,
            misclassification_only_no_amount_misstatement: false,
            actual_damages_cents: 2_000_000,
            court_costs_cents: 100_000,
            attorney_fees_awarded_by_court_discretion: true,
            attorney_fees_amount_cents: 500_000,
            years_since_fraudulent_return_filed: 2,
            years_since_reasonable_discovery: 0,
        }
    }

    #[test]
    fn full_claim_authorizes_civil_action() {
        let r = check(&full_claim());
        assert!(r.civil_action_authorized);
        assert!(r.statute_of_limitations_satisfied);
    }

    #[test]
    fn statutory_minimum_5000_when_actual_below() {
        let mut i = full_claim();
        i.actual_damages_cents = 100_000;
        let r = check(&i);
        assert_eq!(r.statutory_damages_cents, 500_000);
    }

    #[test]
    fn actual_damages_when_above_5000_minimum() {
        let r = check(&full_claim());
        assert_eq!(r.statutory_damages_cents, 2_000_000);
    }

    #[test]
    fn court_costs_recovered_when_authorized() {
        let r = check(&full_claim());
        assert_eq!(r.court_costs_recoverable_cents, 100_000);
    }

    #[test]
    fn attorney_fees_recovered_when_awarded() {
        let r = check(&full_claim());
        assert_eq!(r.attorney_fees_recoverable_cents, 500_000);
    }

    #[test]
    fn attorney_fees_zero_when_not_awarded() {
        let mut i = full_claim();
        i.attorney_fees_awarded_by_court_discretion = false;
        let r = check(&i);
        assert_eq!(r.attorney_fees_recoverable_cents, 0);
    }

    #[test]
    fn total_recovery_sums_three_components() {
        let r = check(&full_claim());
        assert_eq!(r.total_recovery_cents, 2_000_000 + 100_000 + 500_000);
    }

    #[test]
    fn no_willful_filing_defeats_action() {
        let mut i = full_claim();
        i.person_willfully_filed_information_return = false;
        let r = check(&i);
        assert!(!r.civil_action_authorized);
        assert_eq!(r.total_recovery_cents, 0);
    }

    #[test]
    fn no_fraudulent_return_defeats_action() {
        let mut i = full_claim();
        i.return_was_fraudulent = false;
        let r = check(&i);
        assert!(!r.civil_action_authorized);
    }

    #[test]
    fn derolf_misclassification_carveout_defeats_action() {
        let mut i = full_claim();
        i.fraudulent_misstatement_of_payment_amount = false;
        i.misclassification_only_no_amount_misstatement = true;
        let r = check(&i);
        assert!(!r.civil_action_authorized);
        assert!(r.derolf_misclassification_carveout_engaged);
        assert!(r.notes.iter().any(|n| n.contains("Derolf v. Risinger Bros.")));
    }

    #[test]
    fn amount_misstatement_with_misclassification_still_authorizes() {
        let mut i = full_claim();
        i.fraudulent_misstatement_of_payment_amount = true;
        i.misclassification_only_no_amount_misstatement = true;
        let r = check(&i);
        assert!(r.civil_action_authorized);
        assert!(!r.derolf_misclassification_carveout_engaged);
    }

    #[test]
    fn sol_within_6_year_satisfied() {
        let mut i = full_claim();
        i.years_since_fraudulent_return_filed = 5;
        i.years_since_reasonable_discovery = 10;
        let r = check(&i);
        assert!(r.statute_of_limitations_satisfied);
    }

    #[test]
    fn sol_at_6_year_boundary_expired() {
        let mut i = full_claim();
        i.years_since_fraudulent_return_filed = 6;
        i.years_since_reasonable_discovery = 5;
        let r = check(&i);
        assert!(!r.statute_of_limitations_satisfied);
    }

    #[test]
    fn sol_satisfied_under_1_year_discovery_even_if_over_6_filing() {
        let mut i = full_claim();
        i.years_since_fraudulent_return_filed = 10;
        i.years_since_reasonable_discovery = 0;
        let r = check(&i);
        assert!(r.statute_of_limitations_satisfied);
    }

    #[test]
    fn sol_at_1_year_discovery_boundary_expired_if_over_6_filing() {
        let mut i = full_claim();
        i.years_since_fraudulent_return_filed = 10;
        i.years_since_reasonable_discovery = 1;
        let r = check(&i);
        assert!(!r.statute_of_limitations_satisfied);
    }

    #[test]
    fn sol_truth_table_filing_x_discovery() {
        for filing_years in [0u32, 5, 6, 10] {
            for discovery_years in [0u32, 1, 5] {
                let mut i = full_claim();
                i.years_since_fraudulent_return_filed = filing_years;
                i.years_since_reasonable_discovery = discovery_years;
                let r = check(&i);
                let expected = filing_years < 6 || discovery_years < 1;
                assert_eq!(r.statute_of_limitations_satisfied, expected);
            }
        }
    }

    #[test]
    fn expired_sol_defeats_action() {
        let mut i = full_claim();
        i.years_since_fraudulent_return_filed = 10;
        i.years_since_reasonable_discovery = 5;
        let r = check(&i);
        assert!(!r.civil_action_authorized);
        assert_eq!(r.total_recovery_cents, 0);
    }

    #[test]
    fn citation_pins_subsections_and_derolf() {
        let r = check(&full_claim());
        assert!(r.citation.contains("§§ 7434(a), 7434(b), 7434(c), 7434(d), 7434(e), 7434(f)"));
        assert!(r.citation.contains("Derolf v. Risinger Bros."));
    }

    #[test]
    fn return_type_routed_in_note() {
        let mut i = full_claim();
        i.return_type = InformationReturnType::W2;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("W-2 information return")));

        i.return_type = InformationReturnType::Form1099Nec;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("1099-NEC")));
    }

    #[test]
    fn cause_of_action_note_describes_section_7434_a() {
        let r = check(&full_claim());
        assert!(r.notes.iter().any(|n| n.contains("§ 7434(a)") && n.contains("willfully")));
    }

    #[test]
    fn notice_to_secretary_note_present() {
        let r = check(&full_claim());
        assert!(r.notes.iter().any(|n| n.contains("§ 7434(e)") && n.contains("IRS notice")));
    }

    #[test]
    fn collateral_estoppel_note_present() {
        let r = check(&full_claim());
        assert!(r.notes.iter().any(|n| n.contains("collateral-estoppel") && n.contains("Tax Court")));
    }

    #[test]
    fn negative_actual_damages_clamped_to_zero() {
        let mut i = full_claim();
        i.actual_damages_cents = -1_000_000;
        let r = check(&i);
        assert_eq!(r.statutory_damages_cents, 500_000);
    }

    #[test]
    fn negative_court_costs_clamped_to_zero() {
        let mut i = full_claim();
        i.court_costs_cents = -100_000;
        let r = check(&i);
        assert_eq!(r.court_costs_recoverable_cents, 0);
    }

    #[test]
    fn negative_attorney_fees_clamped_to_zero() {
        let mut i = full_claim();
        i.attorney_fees_amount_cents = -50_000;
        let r = check(&i);
        assert_eq!(r.attorney_fees_recoverable_cents, 0);
    }

    #[test]
    fn six_information_return_types_routed_correctly() {
        for return_type in [
            InformationReturnType::W2,
            InformationReturnType::Form1099B,
            InformationReturnType::Form1099Nec,
            InformationReturnType::Form1099Misc,
            InformationReturnType::Form1099Div,
            InformationReturnType::Form1099Int,
            InformationReturnType::Other,
        ] {
            let mut i = full_claim();
            i.return_type = return_type;
            let r = check(&i);
            assert!(r.civil_action_authorized);
        }
    }

    #[test]
    fn damages_note_describes_greater_of_rule() {
        let r = check(&full_claim());
        assert!(r.notes.iter().any(|n| n.contains("§ 7434(b)") && n.contains("greater of $5,000")));
    }

    #[test]
    fn sol_note_describes_later_of_rule() {
        let r = check(&full_claim());
        assert!(r.notes.iter().any(|n| n.contains("§ 7434(d)") && n.contains("later of 6 years")));
    }

    #[test]
    fn civil_action_authorized_truth_table() {
        for willful in [false, true] {
            for fraudulent in [false, true] {
                for misstatement in [false, true] {
                    let mut i = full_claim();
                    i.person_willfully_filed_information_return = willful;
                    i.return_was_fraudulent = fraudulent;
                    i.fraudulent_misstatement_of_payment_amount = misstatement;
                    i.misclassification_only_no_amount_misstatement = !misstatement;
                    let r = check(&i);
                    let expected = willful && fraudulent && misstatement;
                    assert_eq!(r.civil_action_authorized, expected);
                }
            }
        }
    }

    #[test]
    fn statutory_minimum_5000_is_5000() {
        let mut i = full_claim();
        i.actual_damages_cents = 0;
        let r = check(&i);
        assert_eq!(r.statutory_damages_cents, 500_000);
    }

    #[test]
    fn full_recovery_note_describes_total() {
        let r = check(&full_claim());
        assert!(r.notes.iter().any(|n| n.contains("§ 7434 — civil action AUTHORIZED") && n.contains("total")));
    }
}
