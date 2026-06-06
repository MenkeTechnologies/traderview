//! IRC § 6511 — Limitations on credit or refund.
//!
//! Trader-critical procedural rule controlling WHEN a taxpayer may
//! file a claim for refund or credit of an overpayment. Applies to
//! every amended return (Form 1040-X) where the taxpayer seeks money
//! back from the IRS. Beyond § 6511's deadline, a claim is forever
//! barred regardless of merits.
//!
//! § 6511(a) GENERAL RULE — claim for credit or refund shall be filed
//! within 3 YEARS from the time the return was filed or 2 YEARS from
//! the time the tax was paid, WHICHEVER OF SUCH PERIODS EXPIRES THE
//! LATER. If no return was filed, then the 2-year period applies.
//!
//! § 6511(b)(2)(A) LIMIT ON AMOUNT — when claim is filed within the
//! 3-year period under (a), the refund is limited to the portion of
//! tax paid during the 3 YEARS IMMEDIATELY PRECEDING the claim
//! (LOOKBACK rule).
//!
//! § 6511(b)(2)(B) LIMIT ON AMOUNT — when claim is filed within only
//! the 2-year period under (a) (i.e., 3-year period from filing has
//! expired), the refund is limited to the portion of tax paid during
//! the 2 YEARS IMMEDIATELY PRECEDING the claim.
//!
//! § 6511(d)(1) BAD DEBTS AND WORTHLESS SECURITIES — if claim
//! relates to overpayment on account of deductibility under § 166
//! (bad debts), § 832(c), or § 165(g) (worthless securities), in
//! lieu of the 3-year period the period shall be 7 YEARS from the
//! date prescribed by law for filing the return for the year with
//! respect to which the claim is made. Trader-critical for capital
//! traders holding securities that become worthless mid-cycle.
//!
//! § 6511(d)(2)(A) NOL OR CAPITAL LOSS CARRYBACK — if claim relates
//! to overpayment attributable to NOL or capital loss carryback,
//! in lieu of the 3-year period the period ENDS 3 YEARS AFTER THE
//! TIME PRESCRIBED BY LAW FOR FILING THE RETURN FOR THE TAXABLE YEAR
//! OF THE NOL OR CAPITAL LOSS WHICH RESULTS IN SUCH CARRYBACK — not
//! 3 years from the carryback-year return.
//!
//! § 6511(d)(3)(A) FOREIGN TAX CREDIT — if claim relates to
//! overpayment attributable to foreign taxes paid or accrued for
//! which § 901 credit is allowed, in lieu of the 3-year period the
//! period shall be 10 YEARS from the date prescribed by law for
//! filing the return for the year in which such taxes were actually
//! paid or accrued. Rev. Rul. 2020-8 suspended Rev. Rul. 71-533
//! pending IRS reconsideration of whether the 10-year period applies
//! when the FTC carryback arises from a subsequent-year NOL
//! carryback.
//!
//! § 6511(h) FINANCIALLY DISABLED — period suspended during periods
//! of financial disability under § 6511(h)(2). Out of scope of this
//! module's compute mechanic but flagged in the notes.
//!
//! TRADER APPLICATION: a § 475(f) MTM trader who later discovers a
//! Section 165(g) worthless security loss in a prior year has the
//! 7-year window under § 6511(d)(1) to file Form 1040-X claiming the
//! loss. A trader with foreign-source ordinary income (UK dividends,
//! Japanese bonds) facing a missed § 901 FTC has the 10-year window
//! under § 6511(d)(3)(A). Both periods are materially longer than
//! the default 3-year window and prevent permanent loss of refund
//! rights for taxpayers who would otherwise be barred.
//!
//! Citations: IRC § 6511(a) (general 3-year-or-2-year rule);
//! § 6511(b)(2)(A) (3-year lookback); § 6511(b)(2)(B) (2-year
//! lookback); § 6511(d)(1) (7-year bad debt / worthless securities);
//! § 6511(d)(2)(A) (NOL/capital loss carryback ending 3 years from
//! loss-year return due date); § 6511(d)(3)(A) (10-year foreign tax
//! credit); § 6511(h) (financially disabled suspension); Rev. Rul.
//! 2020-8 (FTC carryback from NOL carryback — open question).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ClaimType {
    /// Standard § 6511(a) general rule — 3 years from return filed
    /// or 2 years from tax paid.
    Standard,
    /// § 6511(d)(1) — § 166 bad debt, § 832(c), or § 165(g)
    /// worthless security. 7 years from due date for loss year.
    BadDebtOrWorthlessSecurity,
    /// § 6511(d)(2)(A) — NOL or capital loss carryback. 3 years
    /// from due date of LOSS YEAR return (not carryback year).
    NolOrCapitalLossCarryback,
    /// § 6511(d)(3)(A) — § 901 foreign tax credit. 10 years from
    /// due date for year taxes were paid or accrued.
    ForeignTaxCredit,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6511Input {
    pub claim_type: ClaimType,
    /// Year of the tax return that the refund claim covers.
    /// (For NOL carryback this is the AMENDED year; the loss year
    /// is `carryback_loss_year`.)
    pub return_tax_year: i32,
    /// Year the return was actually filed (April 15 of year+1
    /// default; later if extension granted).
    pub return_filed_year: i32,
    /// Year the tax for `return_tax_year` was paid.
    pub tax_paid_year: i32,
    /// Year the refund claim (Form 1040-X) is filed.
    pub claim_filed_year: i32,
    /// For NOL or capital loss carryback claims: the year the loss
    /// arose that generates the carryback. Period runs 3 years from
    /// due date of THIS year's return.
    pub carryback_loss_year: Option<i32>,
    /// Whether the taxpayer was financially disabled under § 6511(h)
    /// during any portion of the limitations period. If true, the
    /// caller has determined the period suspension and the compute
    /// here flags the open-question status.
    pub financially_disabled: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6511Result {
    pub statute_period_years: i32,
    pub deadline_year: i32,
    pub claim_timely: bool,
    pub lookback_years: i32,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section6511Input) -> Section6511Result {
    let mut notes: Vec<String> = Vec::new();

    let (statute_period_years, deadline_year, lookback_years, base_citation) =
        compute_deadline(input, &mut notes);

    let claim_timely = input.claim_filed_year <= deadline_year;

    if !claim_timely {
        notes.push(format!(
            "claim filed in year {} after deadline year {} — barred under § 6511",
            input.claim_filed_year, deadline_year
        ));
    }

    if input.financially_disabled {
        notes.push(
            "taxpayer flagged as financially disabled under § 6511(h)(2) — limitations period suspended during disability; caller must compute net unsuspended period separately"
                .to_string(),
        );
    }

    Section6511Result {
        statute_period_years,
        deadline_year,
        claim_timely,
        lookback_years,
        citation: base_citation,
        notes,
    }
}

fn compute_deadline(
    input: &Section6511Input,
    notes: &mut Vec<String>,
) -> (i32, i32, i32, &'static str) {
    match input.claim_type {
        ClaimType::Standard => {
            let three_year_deadline = input.return_filed_year + 3;
            let two_year_deadline = input.tax_paid_year + 2;
            let (deadline, period_years, lookback) = if three_year_deadline >= two_year_deadline {
                (three_year_deadline, 3, 3)
            } else {
                (two_year_deadline, 2, 2)
            };
            notes.push(format!(
                "§ 6511(a) — later of 3 years from return filed (year {}) or 2 years from tax paid (year {}) → deadline year {}",
                three_year_deadline, two_year_deadline, deadline
            ));
            notes.push(format!(
                "§ 6511(b)(2) — refund limited to tax paid within {} years immediately preceding claim (lookback)",
                lookback
            ));
            (
                period_years,
                deadline,
                lookback,
                "IRC § 6511(a); § 6511(b)(2)(A); § 6511(b)(2)(B)",
            )
        }
        ClaimType::BadDebtOrWorthlessSecurity => {
            let return_due_year = input.return_tax_year + 1;
            let deadline = return_due_year + 7;
            notes.push(format!(
                "§ 6511(d)(1) — 7 years from return-due year ({}) → deadline year {}",
                return_due_year, deadline
            ));
            notes.push(
                "applies to § 166 bad debt, § 832(c) insurance, § 165(g) worthless security"
                    .to_string(),
            );
            (7, deadline, 3, "IRC § 6511(d)(1); §§ 166, 832(c), 165(g)")
        }
        ClaimType::NolOrCapitalLossCarryback => {
            let loss_year = input.carryback_loss_year.unwrap_or(input.return_tax_year);
            let loss_year_return_due = loss_year + 1;
            let deadline = loss_year_return_due + 3;
            notes.push(format!(
                "§ 6511(d)(2)(A) — 3 years from due date of LOSS YEAR (loss year {} → due year {}) → deadline year {}",
                loss_year, loss_year_return_due, deadline
            ));
            notes.push(
                "deadline keyed to loss-year return due date, NOT carryback-year return due date"
                    .to_string(),
            );
            (3, deadline, 3, "IRC § 6511(d)(2)(A)")
        }
        ClaimType::ForeignTaxCredit => {
            let return_due_year = input.return_tax_year + 1;
            let deadline = return_due_year + 10;
            notes.push(format!(
                "§ 6511(d)(3)(A) — 10 years from due date for year foreign taxes were paid/accrued (year {} → due year {}) → deadline year {}",
                input.return_tax_year, return_due_year, deadline
            ));
            notes.push(
                "Rev. Rul. 2020-8 suspended Rev. Rul. 71-533 — open question whether 10-year period applies to FTC carryback arising from subsequent-year NOL carryback"
                    .to_string(),
            );
            (10, deadline, 10, "IRC § 6511(d)(3)(A); § 901")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn standard(tax_year: i32, filed: i32, paid: i32, claim: i32) -> Section6511Input {
        Section6511Input {
            claim_type: ClaimType::Standard,
            return_tax_year: tax_year,
            return_filed_year: filed,
            tax_paid_year: paid,
            claim_filed_year: claim,
            carryback_loss_year: None,
            financially_disabled: false,
        }
    }

    #[test]
    fn standard_three_years_from_filing_when_filed_after_payment() {
        let r = compute(&standard(2022, 2023, 2023, 2026));
        assert_eq!(r.statute_period_years, 3);
        assert_eq!(r.deadline_year, 2026);
        assert!(r.claim_timely);
        assert_eq!(r.lookback_years, 3);
    }

    #[test]
    fn standard_three_year_deadline_barred_in_year_27() {
        let r = compute(&standard(2022, 2023, 2023, 2027));
        assert!(!r.claim_timely);
        assert_eq!(r.deadline_year, 2026);
        assert!(r.notes.iter().any(|n| n.contains("barred under § 6511")));
    }

    #[test]
    fn standard_two_year_from_payment_when_later_than_three_year_from_filing() {
        let r = compute(&standard(2018, 2019, 2025, 2026));
        assert_eq!(r.deadline_year, 2027);
        assert_eq!(r.statute_period_years, 2);
        assert_eq!(r.lookback_years, 2);
        assert!(r.claim_timely);
    }

    #[test]
    fn standard_two_year_lookback_invoked_when_late_payment() {
        let r = compute(&standard(2018, 2019, 2024, 2025));
        assert_eq!(r.lookback_years, 2);
    }

    #[test]
    fn standard_three_year_period_pinned_in_notes() {
        let r = compute(&standard(2022, 2023, 2023, 2026));
        assert!(r.notes.iter().any(|n| n.contains("§ 6511(a)")));
        assert!(r.notes.iter().any(|n| n.contains("§ 6511(b)(2)")));
        assert!(r.notes.iter().any(|n| n.contains("3 years")));
    }

    #[test]
    fn bad_debt_seven_year_period() {
        let i = Section6511Input {
            claim_type: ClaimType::BadDebtOrWorthlessSecurity,
            return_tax_year: 2020,
            return_filed_year: 2021,
            tax_paid_year: 2021,
            claim_filed_year: 2028,
            carryback_loss_year: None,
            financially_disabled: false,
        };
        let r = compute(&i);
        assert_eq!(r.statute_period_years, 7);
        assert_eq!(r.deadline_year, 2028);
        assert!(r.claim_timely);
    }

    #[test]
    fn bad_debt_barred_in_year_29() {
        let mut i = Section6511Input {
            claim_type: ClaimType::BadDebtOrWorthlessSecurity,
            return_tax_year: 2020,
            return_filed_year: 2021,
            tax_paid_year: 2021,
            claim_filed_year: 2029,
            carryback_loss_year: None,
            financially_disabled: false,
        };
        i.claim_filed_year = 2029;
        let r = compute(&i);
        assert!(!r.claim_timely);
        assert_eq!(r.deadline_year, 2028);
    }

    #[test]
    fn bad_debt_citation_pins_sections_166_832c_165g() {
        let i = Section6511Input {
            claim_type: ClaimType::BadDebtOrWorthlessSecurity,
            return_tax_year: 2020,
            return_filed_year: 2021,
            tax_paid_year: 2021,
            claim_filed_year: 2025,
            carryback_loss_year: None,
            financially_disabled: false,
        };
        let r = compute(&i);
        assert!(r.citation.contains("§ 6511(d)(1)"));
        assert!(r.citation.contains("§§ 166, 832(c), 165(g)"));
    }

    #[test]
    fn nol_carryback_three_years_from_loss_year_return_due() {
        let i = Section6511Input {
            claim_type: ClaimType::NolOrCapitalLossCarryback,
            return_tax_year: 2019,
            return_filed_year: 2020,
            tax_paid_year: 2020,
            claim_filed_year: 2024,
            carryback_loss_year: Some(2021),
            financially_disabled: false,
        };
        let r = compute(&i);
        assert_eq!(r.deadline_year, 2025);
        assert!(r.claim_timely);
    }

    #[test]
    fn nol_carryback_deadline_keyed_to_loss_year_not_amended_year() {
        let i = Section6511Input {
            claim_type: ClaimType::NolOrCapitalLossCarryback,
            return_tax_year: 2018,
            return_filed_year: 2019,
            tax_paid_year: 2019,
            claim_filed_year: 2024,
            carryback_loss_year: Some(2020),
            financially_disabled: false,
        };
        let r = compute(&i);
        assert_eq!(r.deadline_year, 2024);
        assert!(r.claim_timely);
        assert!(r.notes.iter().any(|n| n.contains("LOSS YEAR")));
    }

    #[test]
    fn nol_carryback_barred_when_filed_after_loss_year_plus_4() {
        let i = Section6511Input {
            claim_type: ClaimType::NolOrCapitalLossCarryback,
            return_tax_year: 2018,
            return_filed_year: 2019,
            tax_paid_year: 2019,
            claim_filed_year: 2025,
            carryback_loss_year: Some(2020),
            financially_disabled: false,
        };
        let r = compute(&i);
        assert!(!r.claim_timely);
        assert_eq!(r.deadline_year, 2024);
    }

    #[test]
    fn ftc_ten_year_period() {
        let i = Section6511Input {
            claim_type: ClaimType::ForeignTaxCredit,
            return_tax_year: 2014,
            return_filed_year: 2015,
            tax_paid_year: 2015,
            claim_filed_year: 2025,
            carryback_loss_year: None,
            financially_disabled: false,
        };
        let r = compute(&i);
        assert_eq!(r.statute_period_years, 10);
        assert_eq!(r.deadline_year, 2025);
        assert!(r.claim_timely);
        assert_eq!(r.lookback_years, 10);
    }

    #[test]
    fn ftc_barred_in_year_26() {
        let i = Section6511Input {
            claim_type: ClaimType::ForeignTaxCredit,
            return_tax_year: 2014,
            return_filed_year: 2015,
            tax_paid_year: 2015,
            claim_filed_year: 2026,
            carryback_loss_year: None,
            financially_disabled: false,
        };
        let r = compute(&i);
        assert!(!r.claim_timely);
        assert_eq!(r.deadline_year, 2025);
    }

    #[test]
    fn ftc_citation_pins_subsection_and_section_901() {
        let i = Section6511Input {
            claim_type: ClaimType::ForeignTaxCredit,
            return_tax_year: 2020,
            return_filed_year: 2021,
            tax_paid_year: 2021,
            claim_filed_year: 2025,
            carryback_loss_year: None,
            financially_disabled: false,
        };
        let r = compute(&i);
        assert!(r.citation.contains("§ 6511(d)(3)(A)"));
        assert!(r.citation.contains("§ 901"));
    }

    #[test]
    fn ftc_notes_rev_rul_2020_8_open_question() {
        let i = Section6511Input {
            claim_type: ClaimType::ForeignTaxCredit,
            return_tax_year: 2020,
            return_filed_year: 2021,
            tax_paid_year: 2021,
            claim_filed_year: 2025,
            carryback_loss_year: None,
            financially_disabled: false,
        };
        let r = compute(&i);
        assert!(r.notes.iter().any(|n| n.contains("Rev. Rul. 2020-8")));
        assert!(r.notes.iter().any(|n| n.contains("Rev. Rul. 71-533")));
    }

    #[test]
    fn financial_disability_flag_surfaces_suspension_note() {
        let mut i = standard(2022, 2023, 2023, 2026);
        i.financially_disabled = true;
        let r = compute(&i);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6511(h)") && n.contains("suspended")));
    }

    #[test]
    fn ftc_period_beats_standard_for_long_horizon_overpayment() {
        let standard_i = standard(2014, 2015, 2015, 2020);
        let ftc_i = Section6511Input {
            claim_type: ClaimType::ForeignTaxCredit,
            return_tax_year: 2014,
            return_filed_year: 2015,
            tax_paid_year: 2015,
            claim_filed_year: 2020,
            carryback_loss_year: None,
            financially_disabled: false,
        };
        let rs = compute(&standard_i);
        let rf = compute(&ftc_i);
        assert!(!rs.claim_timely);
        assert!(rf.claim_timely);
    }

    #[test]
    fn bad_debt_period_beats_standard_for_seven_year_horizon() {
        let i = Section6511Input {
            claim_type: ClaimType::BadDebtOrWorthlessSecurity,
            return_tax_year: 2018,
            return_filed_year: 2019,
            tax_paid_year: 2019,
            claim_filed_year: 2025,
            carryback_loss_year: None,
            financially_disabled: false,
        };
        let r = compute(&i);
        assert!(r.claim_timely);
        let standard_i = standard(2018, 2019, 2019, 2025);
        let rs = compute(&standard_i);
        assert!(!rs.claim_timely);
    }

    #[test]
    fn nol_carryback_falls_back_to_return_tax_year_when_loss_year_omitted() {
        let i = Section6511Input {
            claim_type: ClaimType::NolOrCapitalLossCarryback,
            return_tax_year: 2020,
            return_filed_year: 2021,
            tax_paid_year: 2021,
            claim_filed_year: 2025,
            carryback_loss_year: None,
            financially_disabled: false,
        };
        let r = compute(&i);
        assert_eq!(r.deadline_year, 2024);
    }

    #[test]
    fn standard_equal_filing_and_payment_year_uses_three_year() {
        let r = compute(&standard(2022, 2023, 2023, 2026));
        assert_eq!(r.statute_period_years, 3);
    }

    #[test]
    fn standard_lookback_three_when_three_year_period_governs() {
        let r = compute(&standard(2022, 2023, 2023, 2025));
        assert_eq!(r.lookback_years, 3);
    }

    #[test]
    fn standard_lookback_two_when_two_year_period_governs() {
        let r = compute(&standard(2018, 2019, 2024, 2025));
        assert_eq!(r.lookback_years, 2);
    }

    #[test]
    fn claim_at_deadline_boundary_year_timely() {
        let r = compute(&standard(2022, 2023, 2023, 2026));
        assert!(
            r.claim_timely,
            "claim filed in exact deadline year is timely"
        );
    }

    #[test]
    fn claim_one_year_after_deadline_barred() {
        let r = compute(&standard(2022, 2023, 2023, 2027));
        assert!(!r.claim_timely);
    }

    #[test]
    fn ftc_period_is_three_times_standard_period_invariant() {
        let i = Section6511Input {
            claim_type: ClaimType::ForeignTaxCredit,
            return_tax_year: 2020,
            return_filed_year: 2021,
            tax_paid_year: 2021,
            claim_filed_year: 2030,
            carryback_loss_year: None,
            financially_disabled: false,
        };
        let r = compute(&i);
        assert_eq!(r.statute_period_years, 10);
        let s = compute(&standard(2020, 2021, 2021, 2030));
        assert_eq!(s.statute_period_years, 3);
        assert_eq!(r.deadline_year - s.deadline_year, 7);
    }
}
