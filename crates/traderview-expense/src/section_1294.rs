//! IRC § 1294 — Election to extend time for payment of
//! tax on undistributed earnings. Natural sibling to
//! section_1293 (QEF current-taxation mechanic). Where
//! § 1293 imposes immediate current-year inclusion of a
//! shareholder's pro rata share of QEF ordinary earnings
//! and net capital gain (regardless of whether the fund
//! has actually distributed the cash), § 1294 provides
//! a SAFETY VALVE — the shareholder MAY ELECT to extend
//! time for payment of the tax attributable to
//! UNDISTRIBUTED EARNINGS, posting § 6601 interest on
//! the deferred amount until the election terminates.
//!
//! Completes the PFIC framework cluster: section_1291
//! (default excess distribution + interest charge);
//! section_1293 (QEF current-taxation mechanic);
//! section_1294 (QEF undistributed-earnings tax deferral
//! — this module); section_1295 (QEF election mechanism);
//! section_1296 (mark-to-market alternative); section_1297
//! (PFIC definition); section_1298 (special rules).
//!
//! Companion to section_6601 (underpayment interest rate
//! used for § 1294 interest charge), section_6038d (Form
//! 8938 FATCA captures QEF interests), section_988
//! (foreign currency translation).
//!
//! Trader-critical because § 1294 deferral is the only
//! escape valve for cash-poor QEF shareholders — when a
//! QEF generates substantial paper income but distributes
//! minimal cash, § 1293 inclusion creates a tax-payment
//! shortfall. § 1294 lets the shareholder defer payment
//! until the QEF actually distributes the cash or the
//! shareholder disposes of the QEF stock, at the cost of
//! § 6601 interest accrual.
//!
//! **§ 1294(a)(1) Election available** — a U.S. person
//! who is a shareholder in a QEF MAY ELECT to extend
//! time for payment of its tax liability ATTRIBUTABLE TO
//! ITS SHARE OF THE UNDISTRIBUTED EARNINGS of the QEF.
//!
//! **§ 1294(b) Undistributed earnings definition** —
//! EXCESS, IF ANY, of:
//! 1. Amount includible in gross income under § 1293(a)
//!    for shareholder's taxable year (the "includible
//!    amount"); OVER
//! 2. SUM of:
//!    - Amount of any distribution to the shareholder
//!      during the QEF's taxable year; AND
//!    - Portion of the includible amount attributable to
//!      stock in the QEF that the shareholder TRANSFERRED
//!      OR OTHERWISE DISPOSED OF before the end of the
//!      QEF's year.
//!
//! **§ 1294(c) Interest charge under § 6601** — interest
//! is imposed under § 6601 on the amount of the tax
//! liability that is subject to the extension. This
//! interest must be paid on TERMINATION of the election.
//!
//! § 6601 underpayment interest rate (set quarterly by
//! Treasury under § 6621) is the rate applied. Interest
//! compounded DAILY per § 6622.
//!
//! **§ 1294(d) Coordination — election unavailable**:
//! 1. § 1294(d)(1) — election CANNOT be made for taxable
//!    year of shareholder if ANY PORTION of QEF's
//!    earnings is includible in shareholder's gross income
//!    under § 551 (foreign personal holding companies);
//!    OR
//! 2. § 1294(d)(2) — under § 951 (controlled foreign
//!    corporations).
//!
//! **§ 1294(e) Termination of election** — election
//! terminates upon EARLIEST of:
//! 1. Actual distribution by QEF that reduces undistributed
//!    earnings (taxable distribution applied against
//!    deferred amount);
//! 2. Disposition of QEF stock (transfer, sale, exchange,
//!    redemption);
//! 3. Affirmative termination by shareholder;
//! 4. Death of individual shareholder;
//! 5. QEF ceases to be a QEF;
//! 6. Shareholder ceases to be a U.S. person.
//!
//! Upon termination: deferred tax plus accrued § 6601
//! interest becomes DUE AND PAYABLE.
//!
//! **Form 8621 reporting** — § 1294 election filed with
//! Form 8621 (Information Return by Shareholder of PFIC
//! or QEF); election made annually (must be re-elected
//! each year for new undistributed earnings); deferred-
//! tax-and-interest balance reported on each year's Form
//! 8621.
//!
//! **Practical relevance** — § 1294 deferral is RARELY
//! USED in practice because:
//! 1. Interest charge accrual at § 6601 rate (quarterly
//!    underpayment rate, often 7-8% during 2023-2025)
//!    erodes deferral benefit;
//! 2. Termination upon disposition or distribution
//!    creates lump-sum tax + interest liability;
//! 3. Annual election + complex reporting (Treas. Reg.
//!    § 1.1294-1T) increase compliance burden;
//! 4. Most QEFs distribute cash sufficient to cover the
//!    tax liability (the § 1294 problem only arises with
//!    reinvestment-focused QEFs).
//!
//! Citations: 26 USC § 1294(a)(1) and § 1294(b) and
//! § 1294(c) and § 1294(d)(1)-(2) and § 1294(e); 26 USC
//! § 1293(a) and § 1295 and § 1296 and § 1297; 26 USC
//! § 551 and § 951; 26 USC § 6601 and § 6621 and § 6622;
//! Treas. Reg. § 1.1294-1T; Form 8621; Tax Reform Act of
//! 1986 § 1235 (Pub. L. 99-514, October 22, 1986).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TerminationEvent {
    /// Election still in effect (no termination).
    NoTermination,
    /// QEF made distribution reducing undistributed
    /// earnings.
    DistributionReducingUndistributedEarnings,
    /// Shareholder disposed of QEF stock (transfer / sale
    /// / exchange / redemption).
    QefStockDisposition,
    /// Shareholder affirmatively terminated election.
    AffirmativeTermination,
    /// Death of individual shareholder.
    ShareholderDeath,
    /// QEF ceased to be a QEF.
    QefCeasedToBeQef,
    /// Shareholder ceased to be a U.S. person.
    ShareholderCeasedToBeUsPerson,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section1294Input {
    /// Amount includible in gross income under § 1293(a)
    /// for shareholder's taxable year in cents (the
    /// "includible amount").
    pub section_1293_includible_amount_cents: u64,
    /// Amount of distribution received by shareholder
    /// during QEF's taxable year in cents.
    pub distribution_received_cents: u64,
    /// Portion of includible amount attributable to QEF
    /// stock transferred or disposed of before end of
    /// QEF's year in cents.
    pub disposed_stock_portion_cents: u64,
    /// Whether § 551 foreign personal holding company
    /// rules engaged (election unavailable).
    pub section_551_fphc_engaged: bool,
    /// Whether § 951 subpart F CFC rules engaged
    /// (election unavailable).
    pub section_951_cfc_engaged: bool,
    /// Whether shareholder elected § 1294 deferral.
    pub election_made: bool,
    /// Termination event status.
    pub termination_event: TerminationEvent,
    /// Tax liability rate on undistributed earnings in
    /// basis points (e.g., 3700 = 37%).
    pub tax_rate_on_undistributed_bps: u32,
    /// § 6601 underpayment interest rate in basis points
    /// (e.g., 800 = 8%).
    pub section_6601_interest_rate_bps: u32,
    /// Days of interest accrual (election period until
    /// termination).
    pub days_of_interest_accrual: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section1294Result {
    pub election_available: bool,
    pub election_made: bool,
    pub undistributed_earnings_cents: u64,
    pub deferred_tax_cents: u64,
    pub accrued_interest_cents: u64,
    pub total_due_on_termination_cents: u64,
    pub termination_event: TerminationEvent,
    pub termination_engaged: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section1294Input) -> Section1294Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let election_available = !input.section_551_fphc_engaged && !input.section_951_cfc_engaged;

    let undistributed_earnings_cents = input
        .section_1293_includible_amount_cents
        .saturating_sub(input.distribution_received_cents)
        .saturating_sub(input.disposed_stock_portion_cents);

    let deferred_tax_cents = if input.election_made && election_available {
        undistributed_earnings_cents.saturating_mul(input.tax_rate_on_undistributed_bps as u64)
            / 10_000
    } else {
        0
    };

    let accrued_interest_cents = if deferred_tax_cents > 0 {
        let interest_factor = (input.section_6601_interest_rate_bps as u64)
            .saturating_mul(input.days_of_interest_accrual as u64);
        deferred_tax_cents.saturating_mul(interest_factor) / 3_650_000
    } else {
        0
    };

    let termination_engaged = !matches!(input.termination_event, TerminationEvent::NoTermination);

    let total_due_on_termination_cents = if termination_engaged {
        deferred_tax_cents.saturating_add(accrued_interest_cents)
    } else {
        0
    };

    if input.election_made && !election_available {
        if input.section_551_fphc_engaged {
            failure_reasons.push(
                "26 USC § 1294(d)(1) — § 1294 election CANNOT BE MADE for taxable year of shareholder if ANY PORTION of QEF's earnings is includible in shareholder's gross income under § 551 (foreign personal holding companies)".to_string(),
            );
        }
        if input.section_951_cfc_engaged {
            failure_reasons.push(
                "26 USC § 1294(d)(2) — § 1294 election CANNOT BE MADE if QEF's earnings are includible in shareholder's gross income under § 951 (controlled foreign corporation subpart F); § 1297(d) PFIC-CFC overlap rule resolves the overlap".to_string(),
            );
        }
    }

    if input.election_made && election_available && undistributed_earnings_cents > 0 {
        failure_reasons.push(format!(
            "26 USC § 1294(a)(1) — § 1294 election MADE; undistributed earnings = {} cents; deferred tax at {} bps = {} cents; § 6601 interest accrues at {} bps over {} days = {} cents; total due on termination = {} cents",
            undistributed_earnings_cents,
            input.tax_rate_on_undistributed_bps,
            deferred_tax_cents,
            input.section_6601_interest_rate_bps,
            input.days_of_interest_accrual,
            accrued_interest_cents,
            total_due_on_termination_cents
        ));
    }

    if termination_engaged && input.election_made {
        let termination_label = match input.termination_event {
            TerminationEvent::NoTermination => "(none)",
            TerminationEvent::DistributionReducingUndistributedEarnings => {
                "distribution reducing undistributed earnings"
            }
            TerminationEvent::QefStockDisposition => "QEF stock disposition",
            TerminationEvent::AffirmativeTermination => "affirmative termination by shareholder",
            TerminationEvent::ShareholderDeath => "death of individual shareholder",
            TerminationEvent::QefCeasedToBeQef => "QEF ceased to be a QEF",
            TerminationEvent::ShareholderCeasedToBeUsPerson => {
                "shareholder ceased to be a U.S. person"
            }
        };
        failure_reasons.push(format!(
            "26 USC § 1294(e) — § 1294 election TERMINATED by event: {}; deferred tax plus accrued § 6601 interest becomes DUE AND PAYABLE = {} cents",
            termination_label, total_due_on_termination_cents
        ));
    }

    let notes: Vec<String> = vec![
        "26 USC § 1294(a)(1) — U.S. person who is a shareholder in a qualified electing fund MAY ELECT to extend time for payment of tax liability ATTRIBUTABLE TO ITS SHARE OF UNDISTRIBUTED EARNINGS of the QEF".to_string(),
        "26 USC § 1294(b) — 'UNDISTRIBUTED EARNINGS' = EXCESS of (1) amount includible in gross income under § 1293(a) for shareholder's taxable year (the 'includible amount') OVER (2) SUM of (a) distribution received during QEF's taxable year AND (b) portion of includible amount attributable to QEF stock TRANSFERRED OR DISPOSED OF before end of QEF's year".to_string(),
        "26 USC § 1294(c) — INTEREST imposed under § 6601 on amount of tax liability subject to extension; rate = § 6621 underpayment rate (set quarterly); compounded DAILY per § 6622; interest must be paid on TERMINATION of election".to_string(),
        "26 USC § 1294(d)(1) — election CANNOT BE MADE if any portion of QEF's earnings includible under § 551 (foreign personal holding companies)".to_string(),
        "26 USC § 1294(d)(2) — election CANNOT BE MADE if any portion of QEF's earnings includible under § 951 (controlled foreign corporation subpart F); § 1297(d) PFIC-CFC overlap rule resolves the overlap".to_string(),
        "26 USC § 1294(e) — election terminates upon EARLIEST of: (1) distribution by QEF reducing undistributed earnings; (2) disposition of QEF stock; (3) affirmative termination by shareholder; (4) death of individual shareholder; (5) QEF ceases to be a QEF; (6) shareholder ceases to be a U.S. person".to_string(),
        "Upon § 1294(e) TERMINATION — deferred tax plus accrued § 6601 interest becomes DUE AND PAYABLE on termination date".to_string(),
        "Form 8621 reporting — § 1294 election filed with Form 8621 (Information Return by Shareholder of PFIC or QEF); election made annually (re-elected each year for new undistributed earnings); deferred-tax-and-interest balance reported on each year's Form 8621".to_string(),
        "Treas. Reg. § 1.1294-1T — temporary regulations governing § 1294 election procedure + content + annual re-election + termination reporting".to_string(),
        "§ 1294 deferral RARELY USED in practice — interest charge accrual at § 6601 rate (quarterly underpayment rate, often 7-8% during 2023-2025) erodes deferral benefit; termination upon disposition or distribution creates lump-sum tax + interest liability; annual election + complex reporting increase compliance burden; most QEFs distribute cash sufficient to cover the tax liability".to_string(),
        "PFIC framework cluster: § 1291 (DEFAULT excess distribution + interest charge — punitive); § 1293 (QEF current-taxation MECHANIC); § 1294 (QEF undistributed-earnings tax DEFERRAL — this module); § 1295 (QEF election mechanism); § 1296 (mark-to-market alternative); § 1297 (PFIC definition); § 1298 (special rules)".to_string(),
        "Enacted by Tax Reform Act of 1986 § 1235 (Pub. L. 99-514, October 22, 1986)".to_string(),
    ];

    Section1294Result {
        election_available,
        election_made: input.election_made,
        undistributed_earnings_cents,
        deferred_tax_cents,
        accrued_interest_cents,
        total_due_on_termination_cents,
        termination_event: input.termination_event,
        termination_engaged,
        failure_reasons,
        citation: "26 USC § 1294(a)(1) + § 1294(b) + § 1294(c) + § 1294(d)(1)-(2) + § 1294(e); 26 USC § 1293(a); 26 USC § 1295; 26 USC § 1296; 26 USC § 1297; 26 USC § 551; 26 USC § 951; 26 USC § 6601; 26 USC § 6621; 26 USC § 6622; Treas. Reg. § 1.1294-1T; Form 8621; Tax Reform Act of 1986 § 1235 (Pub. L. 99-514, October 22, 1986)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_election() -> Section1294Input {
        Section1294Input {
            section_1293_includible_amount_cents: 100_000_000,
            distribution_received_cents: 20_000_000,
            disposed_stock_portion_cents: 0,
            section_551_fphc_engaged: false,
            section_951_cfc_engaged: false,
            election_made: true,
            termination_event: TerminationEvent::NoTermination,
            tax_rate_on_undistributed_bps: 3700,
            section_6601_interest_rate_bps: 800,
            days_of_interest_accrual: 365,
        }
    }

    #[test]
    fn baseline_election_undistributed_earnings_80m() {
        let r = check(&baseline_election());
        assert_eq!(r.undistributed_earnings_cents, 80_000_000);
    }

    #[test]
    fn election_available_when_no_fphc_no_cfc() {
        let r = check(&baseline_election());
        assert!(r.election_available);
    }

    #[test]
    fn deferred_tax_at_37_percent() {
        let r = check(&baseline_election());
        assert_eq!(r.deferred_tax_cents, 29_600_000);
    }

    #[test]
    fn distribution_reduces_undistributed_earnings() {
        let mut i = baseline_election();
        i.distribution_received_cents = 50_000_000;
        let r = check(&i);
        assert_eq!(r.undistributed_earnings_cents, 50_000_000);
    }

    #[test]
    fn disposed_stock_portion_reduces_undistributed() {
        let mut i = baseline_election();
        i.disposed_stock_portion_cents = 30_000_000;
        let r = check(&i);
        assert_eq!(r.undistributed_earnings_cents, 50_000_000);
    }

    #[test]
    fn distribution_exceeds_includible_no_undistributed() {
        let mut i = baseline_election();
        i.distribution_received_cents = 150_000_000;
        let r = check(&i);
        assert_eq!(r.undistributed_earnings_cents, 0);
        assert_eq!(r.deferred_tax_cents, 0);
    }

    #[test]
    fn section_551_engaged_election_unavailable() {
        let mut i = baseline_election();
        i.section_551_fphc_engaged = true;
        let r = check(&i);
        assert!(!r.election_available);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 1294(d)(1)")
            && f.contains("§ 551")
            && f.contains("foreign personal holding companies")));
    }

    #[test]
    fn section_951_engaged_election_unavailable() {
        let mut i = baseline_election();
        i.section_951_cfc_engaged = true;
        let r = check(&i);
        assert!(!r.election_available);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 1294(d)(2)") && f.contains("§ 951") && f.contains("§ 1297(d)")));
    }

    #[test]
    fn election_not_made_no_deferral() {
        let mut i = baseline_election();
        i.election_made = false;
        let r = check(&i);
        assert_eq!(r.deferred_tax_cents, 0);
        assert_eq!(r.accrued_interest_cents, 0);
    }

    #[test]
    fn interest_accrues_at_6601_rate() {
        let r = check(&baseline_election());
        assert!(r.accrued_interest_cents > 0);
    }

    #[test]
    fn termination_distribution_engages() {
        let mut i = baseline_election();
        i.termination_event = TerminationEvent::DistributionReducingUndistributedEarnings;
        let r = check(&i);
        assert!(r.termination_engaged);
        assert!(r.total_due_on_termination_cents > 0);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 1294(e)")
            && f.contains("distribution reducing undistributed earnings")
            && f.contains("DUE AND PAYABLE")));
    }

    #[test]
    fn termination_disposition_engages() {
        let mut i = baseline_election();
        i.termination_event = TerminationEvent::QefStockDisposition;
        let r = check(&i);
        assert!(r.termination_engaged);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("QEF stock disposition")));
    }

    #[test]
    fn termination_affirmative_engages() {
        let mut i = baseline_election();
        i.termination_event = TerminationEvent::AffirmativeTermination;
        let r = check(&i);
        assert!(r.termination_engaged);
    }

    #[test]
    fn termination_death_engages() {
        let mut i = baseline_election();
        i.termination_event = TerminationEvent::ShareholderDeath;
        let r = check(&i);
        assert!(r.termination_engaged);
    }

    #[test]
    fn termination_qef_ceased_engages() {
        let mut i = baseline_election();
        i.termination_event = TerminationEvent::QefCeasedToBeQef;
        let r = check(&i);
        assert!(r.termination_engaged);
    }

    #[test]
    fn termination_us_person_ceased_engages() {
        let mut i = baseline_election();
        i.termination_event = TerminationEvent::ShareholderCeasedToBeUsPerson;
        let r = check(&i);
        assert!(r.termination_engaged);
    }

    #[test]
    fn no_termination_no_due_amount() {
        let r = check(&baseline_election());
        assert!(!r.termination_engaged);
        assert_eq!(r.total_due_on_termination_cents, 0);
    }

    #[test]
    fn termination_event_truth_table_seven_cells() {
        for (event, exp_engaged) in [
            (TerminationEvent::NoTermination, false),
            (
                TerminationEvent::DistributionReducingUndistributedEarnings,
                true,
            ),
            (TerminationEvent::QefStockDisposition, true),
            (TerminationEvent::AffirmativeTermination, true),
            (TerminationEvent::ShareholderDeath, true),
            (TerminationEvent::QefCeasedToBeQef, true),
            (TerminationEvent::ShareholderCeasedToBeUsPerson, true),
        ] {
            let mut i = baseline_election();
            i.termination_event = event;
            let r = check(&i);
            assert_eq!(r.termination_engaged, exp_engaged, "event={:?}", event);
        }
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&baseline_election());
        assert!(r.citation.contains("§ 1294(a)(1)"));
        assert!(r.citation.contains("§ 1294(b)"));
        assert!(r.citation.contains("§ 1294(c)"));
        assert!(r.citation.contains("§ 1294(d)(1)-(2)"));
        assert!(r.citation.contains("§ 1294(e)"));
        assert!(r.citation.contains("§ 1293(a)"));
        assert!(r.citation.contains("§ 1295"));
        assert!(r.citation.contains("§ 1296"));
        assert!(r.citation.contains("§ 1297"));
        assert!(r.citation.contains("§ 551"));
        assert!(r.citation.contains("§ 951"));
        assert!(r.citation.contains("§ 6601"));
        assert!(r.citation.contains("§ 6621"));
        assert!(r.citation.contains("§ 6622"));
        assert!(r.citation.contains("Treas. Reg. § 1.1294-1T"));
        assert!(r.citation.contains("Form 8621"));
        assert!(r.citation.contains("Tax Reform Act of 1986 § 1235"));
        assert!(r.citation.contains("Pub. L. 99-514"));
        assert!(r.citation.contains("October 22, 1986"));
    }

    #[test]
    fn note_pins_subsection_a1_election_available() {
        let r = check(&baseline_election());
        assert!(r.notes.iter().any(|n| n.contains("§ 1294(a)(1)")
            && n.contains("MAY ELECT")
            && n.contains("UNDISTRIBUTED EARNINGS")));
    }

    #[test]
    fn note_pins_subsection_b_undistributed_definition() {
        let r = check(&baseline_election());
        assert!(r.notes.iter().any(|n| n.contains("§ 1294(b)")
            && n.contains("UNDISTRIBUTED EARNINGS")
            && n.contains("includible amount")
            && n.contains("TRANSFERRED OR DISPOSED OF")));
    }

    #[test]
    fn note_pins_subsection_c_6601_interest() {
        let r = check(&baseline_election());
        assert!(r.notes.iter().any(|n| n.contains("§ 1294(c)")
            && n.contains("INTEREST imposed under § 6601")
            && n.contains("§ 6621")
            && n.contains("§ 6622")
            && n.contains("DAILY")));
    }

    #[test]
    fn note_pins_subsection_d1_fphc_carveout() {
        let r = check(&baseline_election());
        assert!(r.notes.iter().any(|n| n.contains("§ 1294(d)(1)")
            && n.contains("§ 551")
            && n.contains("foreign personal holding companies")));
    }

    #[test]
    fn note_pins_subsection_d2_cfc_carveout() {
        let r = check(&baseline_election());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1294(d)(2)") && n.contains("§ 951") && n.contains("§ 1297(d)")));
    }

    #[test]
    fn note_pins_subsection_e_six_termination_events() {
        let r = check(&baseline_election());
        assert!(r.notes.iter().any(|n| n.contains("§ 1294(e)")
            && n.contains("EARLIEST")
            && n.contains("distribution by QEF")
            && n.contains("disposition")
            && n.contains("death")
            && n.contains("ceases to be a QEF")
            && n.contains("ceases to be a U.S. person")));
    }

    #[test]
    fn note_pins_termination_due_and_payable() {
        let r = check(&baseline_election());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 1294(e) TERMINATION") && n.contains("DUE AND PAYABLE")));
    }

    #[test]
    fn note_pins_form_8621_annual_election() {
        let r = check(&baseline_election());
        assert!(r.notes.iter().any(|n| n.contains("Form 8621")
            && n.contains("annually")
            && n.contains("re-elected")));
    }

    #[test]
    fn note_pins_treas_reg_1_1294_1t_temporary() {
        let r = check(&baseline_election());
        assert!(r.notes.iter().any(|n| n.contains("Treas. Reg. § 1.1294-1T")
            && n.contains("temporary regulations")
            && n.contains("annual re-election")));
    }

    #[test]
    fn note_pins_practical_relevance_rarely_used() {
        let r = check(&baseline_election());
        assert!(r.notes.iter().any(|n| n.contains("RARELY USED")
            && n.contains("interest charge accrual")
            && n.contains("compliance burden")));
    }

    #[test]
    fn note_pins_pfic_framework_cluster() {
        let r = check(&baseline_election());
        assert!(r.notes.iter().any(|n| n.contains("PFIC framework cluster")
            && n.contains("§ 1291")
            && n.contains("§ 1293")
            && n.contains("§ 1294")
            && n.contains("§ 1295")
            && n.contains("§ 1296")
            && n.contains("§ 1297")
            && n.contains("§ 1298")));
    }

    #[test]
    fn note_pins_1986_tax_reform_origin() {
        let r = check(&baseline_election());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Tax Reform Act of 1986 § 1235")
                && n.contains("Pub. L. 99-514")
                && n.contains("October 22, 1986")));
    }

    #[test]
    fn election_uniquely_engages_with_no_carveouts_invariant() {
        let mut clean = baseline_election();
        let r_clean = check(&clean);
        assert!(r_clean.election_available);

        clean.section_551_fphc_engaged = true;
        let r_551 = check(&clean);
        assert!(!r_551.election_available);

        clean.section_551_fphc_engaged = false;
        clean.section_951_cfc_engaged = true;
        let r_951 = check(&clean);
        assert!(!r_951.election_available);
    }

    #[test]
    fn defensive_zero_includible_no_undistributed() {
        let mut i = baseline_election();
        i.section_1293_includible_amount_cents = 0;
        let r = check(&i);
        assert_eq!(r.undistributed_earnings_cents, 0);
        assert_eq!(r.deferred_tax_cents, 0);
    }

    #[test]
    fn defensive_overflow_saturating() {
        let mut i = baseline_election();
        i.section_1293_includible_amount_cents = u64::MAX;
        i.tax_rate_on_undistributed_bps = 10_000;
        let r = check(&i);
        let _ = r.deferred_tax_cents;
    }
}
