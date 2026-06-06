//! IRC §3406 — Backup withholding.
//!
//! Universal rule for any trader or business making "reportable
//! payments" — interest, dividends, rents, royalties, non-employee
//! compensation, broker proceeds, attorney fees, etc. When any of
//! four statutory triggers fires, the PAYOR must withhold **24%**
//! of the payment and remit it to the IRS. The 24% rate is tied to
//! the "fourth lowest rate of tax applicable under § 1(c)" under
//! § 3406(b)(1)(A); the rate was 28% pre-TCJA and 31% earlier
//! ([Cornell LII 26 U.S.C. § 3406](https://www.law.cornell.edu/uscode/text/26/3406),
//! [IRS BWH-B Program](https://www.irs.gov/pub/irs-pdf/p1281.pdf)).
//!
//! **§3406(a) four trigger conditions** — any one fires the
//! 24% withholding obligation on the next reportable payment:
//!
//! - **A trigger (§3406(a)(1)(A))** — payee fails to furnish TIN
//!   to the payor in the manner required (e.g., missing Form W-9).
//! - **B trigger (§3406(a)(1)(B))** — Secretary notifies the payor
//!   that the TIN furnished by the payee is INCORRECT. IRS sends
//!   CP 2100 or CP 2100A to the payor under the BWH-B program; the
//!   payor must notify the payee + commence backup withholding if
//!   the payee does not certify a correct TIN within the specified
//!   window.
//! - **C trigger (§3406(a)(1)(C))** — IRS-notified payee
//!   underreporting of interest or dividends (BWH-C program). Only
//!   applies to interest/dividend payments. IRS sends notice
//!   directly to the payee; backup withholding continues until
//!   IRS notifies otherwise.
//! - **D trigger (§3406(a)(1)(D))** — payee certification failure
//!   under § 3406(d) (typically the "perjury statement" attesting
//!   that the payee is not subject to backup withholding).
//!
//! **Reportable payment types** — module categorizes 10 payment
//! types: Interest, Dividends, Rent, Royalties, NonEmployeeComp,
//! BrokerProceeds, Barter, AttorneyFees, FishingBoatPayments, Other.
//! Each maps to the relevant Form 1099 information return ([IRS
//! Publication 1281](https://www.irs.gov/pub/irs-pdf/p1281.pdf)).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportablePaymentType {
    Interest,
    Dividends,
    Rent,
    Royalties,
    NonEmployeeCompensation,
    BrokerProceeds,
    Barter,
    AttorneyFees,
    FishingBoatPayments,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackupTrigger {
    TinNotFurnished,
    IrsNotifiedIncorrectTin,
    NotifiedPayeeUnderreporting,
    PayeeCertificationFailure,
    NoTrigger,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section3406Input {
    pub payment_type: ReportablePaymentType,
    pub payment_amount_dollars: i64,
    /// True if the payee provided a valid TIN (Form W-9 or equivalent).
    pub tin_furnished: bool,
    /// True if the IRS has notified the payor under the BWH-B program
    /// (CP 2100 / CP 2100A) that the TIN is incorrect AND the payee
    /// has not yet responded with a correct certified TIN.
    pub irs_notified_incorrect_tin: bool,
    /// True if the IRS has notified the payee under the BWH-C program
    /// of underreporting of interest or dividends.
    pub notified_payee_underreporting: bool,
    /// True if the payee certification failed under § 3406(d) (e.g.,
    /// missing or false perjury statement on Form W-9).
    pub payee_certification_failure: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section3406Result {
    pub backup_withholding_required: bool,
    pub trigger: BackupTrigger,
    pub backup_withholding_rate_bp: u32,
    pub backup_withholding_amount_dollars: i64,
    pub net_payment_to_payee_dollars: i64,
    pub citation: String,
    pub note: String,
}

const BACKUP_WITHHOLDING_RATE_BP: u32 = 2400; // 24% post-TCJA

pub fn compute(input: &Section3406Input) -> Section3406Result {
    // §3406(a) trigger evaluation in statutory order (A, B, C, D).
    let trigger = if !input.tin_furnished {
        BackupTrigger::TinNotFurnished
    } else if input.irs_notified_incorrect_tin {
        BackupTrigger::IrsNotifiedIncorrectTin
    } else if input.notified_payee_underreporting
        && matches!(
            input.payment_type,
            ReportablePaymentType::Interest | ReportablePaymentType::Dividends
        )
    {
        BackupTrigger::NotifiedPayeeUnderreporting
    } else if input.payee_certification_failure {
        BackupTrigger::PayeeCertificationFailure
    } else {
        BackupTrigger::NoTrigger
    };

    let required = trigger != BackupTrigger::NoTrigger;

    let withholding = if required {
        (input.payment_amount_dollars as i128 * BACKUP_WITHHOLDING_RATE_BP as i128 / 10_000) as i64
    } else {
        0
    };
    let net = input.payment_amount_dollars - withholding;

    let note = if required {
        format!(
            "§3406(a) BACKUP WITHHOLDING TRIGGERED ({:?}): withhold 24% of ${} payment ({:?}) = ${} sent to IRS; ${} net to payee.",
            trigger,
            input.payment_amount_dollars,
            input.payment_type,
            withholding,
            net,
        )
    } else {
        format!(
            "§3406(a) no trigger fired: full ${} {:?} payment to payee; no backup withholding.",
            input.payment_amount_dollars, input.payment_type,
        )
    };

    Section3406Result {
        backup_withholding_required: required,
        trigger,
        backup_withholding_rate_bp: BACKUP_WITHHOLDING_RATE_BP,
        backup_withholding_amount_dollars: withholding,
        net_payment_to_payee_dollars: net,
        citation:
            "IRC §3406(a)(1)(A) TIN-not-furnished trigger; §3406(a)(1)(B) IRS-notified-incorrect-TIN trigger (BWH-B program, CP 2100 / CP 2100A); §3406(a)(1)(C) notified-payee-underreporting trigger (BWH-C program, interest/dividend only); §3406(a)(1)(D) payee-certification-failure trigger; §3406(b)(1)(A) rate = fourth lowest § 1(c) rate (24% post-TCJA); IRS Publication 1281"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section3406Input {
        Section3406Input {
            payment_type: ReportablePaymentType::NonEmployeeCompensation,
            payment_amount_dollars: 10_000,
            tin_furnished: true,
            irs_notified_incorrect_tin: false,
            notified_payee_underreporting: false,
            payee_certification_failure: false,
        }
    }

    // No trigger.

    #[test]
    fn no_trigger_no_withholding() {
        let r = compute(&base());
        assert!(!r.backup_withholding_required);
        assert_eq!(r.trigger, BackupTrigger::NoTrigger);
        assert_eq!(r.backup_withholding_amount_dollars, 0);
        assert_eq!(r.net_payment_to_payee_dollars, 10_000);
    }

    // A trigger — TIN not furnished.

    #[test]
    fn tin_not_furnished_triggers_24_pct_withholding() {
        let mut i = base();
        i.tin_furnished = false;
        let r = compute(&i);
        assert!(r.backup_withholding_required);
        assert_eq!(r.trigger, BackupTrigger::TinNotFurnished);
        assert_eq!(r.backup_withholding_amount_dollars, 2_400);
        assert_eq!(r.net_payment_to_payee_dollars, 7_600);
    }

    // B trigger — IRS-notified incorrect TIN.

    #[test]
    fn irs_notified_incorrect_tin_triggers_withholding() {
        let mut i = base();
        i.irs_notified_incorrect_tin = true;
        let r = compute(&i);
        assert_eq!(r.trigger, BackupTrigger::IrsNotifiedIncorrectTin);
        assert!(r.backup_withholding_required);
    }

    // C trigger — only applies to interest/dividends.

    #[test]
    fn c_trigger_fires_for_interest() {
        let mut i = base();
        i.payment_type = ReportablePaymentType::Interest;
        i.notified_payee_underreporting = true;
        let r = compute(&i);
        assert_eq!(r.trigger, BackupTrigger::NotifiedPayeeUnderreporting);
        assert!(r.backup_withholding_required);
    }

    #[test]
    fn c_trigger_fires_for_dividends() {
        let mut i = base();
        i.payment_type = ReportablePaymentType::Dividends;
        i.notified_payee_underreporting = true;
        let r = compute(&i);
        assert_eq!(r.trigger, BackupTrigger::NotifiedPayeeUnderreporting);
    }

    #[test]
    fn c_trigger_does_not_fire_for_rent() {
        // §3406(a)(1)(C) C-notice underreporting trigger applies ONLY
        // to interest and dividend payments.
        let mut i = base();
        i.payment_type = ReportablePaymentType::Rent;
        i.notified_payee_underreporting = true;
        let r = compute(&i);
        assert_eq!(r.trigger, BackupTrigger::NoTrigger);
        assert!(!r.backup_withholding_required);
    }

    #[test]
    fn c_trigger_does_not_fire_for_non_employee_comp() {
        let mut i = base();
        i.notified_payee_underreporting = true;
        let r = compute(&i);
        assert_eq!(r.trigger, BackupTrigger::NoTrigger);
    }

    // D trigger — certification failure.

    #[test]
    fn certification_failure_triggers_withholding() {
        let mut i = base();
        i.payee_certification_failure = true;
        let r = compute(&i);
        assert_eq!(r.trigger, BackupTrigger::PayeeCertificationFailure);
        assert!(r.backup_withholding_required);
    }

    // Trigger precedence.

    #[test]
    fn tin_not_furnished_takes_priority_over_b_notice() {
        // If TIN missing AND IRS B-notice also present, A trigger wins.
        let mut i = base();
        i.tin_furnished = false;
        i.irs_notified_incorrect_tin = true;
        let r = compute(&i);
        assert_eq!(r.trigger, BackupTrigger::TinNotFurnished);
    }

    #[test]
    fn b_notice_takes_priority_over_c_notice() {
        let mut i = base();
        i.payment_type = ReportablePaymentType::Interest;
        i.irs_notified_incorrect_tin = true;
        i.notified_payee_underreporting = true;
        let r = compute(&i);
        assert_eq!(r.trigger, BackupTrigger::IrsNotifiedIncorrectTin);
    }

    #[test]
    fn c_notice_takes_priority_over_d_trigger() {
        let mut i = base();
        i.payment_type = ReportablePaymentType::Interest;
        i.notified_payee_underreporting = true;
        i.payee_certification_failure = true;
        let r = compute(&i);
        assert_eq!(r.trigger, BackupTrigger::NotifiedPayeeUnderreporting);
    }

    // Rate / amount.

    #[test]
    fn withholding_rate_24_percent() {
        let mut i = base();
        i.tin_furnished = false;
        let r = compute(&i);
        assert_eq!(r.backup_withholding_rate_bp, 2400);
    }

    #[test]
    fn withholding_at_100k_payment() {
        let mut i = base();
        i.tin_furnished = false;
        i.payment_amount_dollars = 100_000;
        let r = compute(&i);
        assert_eq!(r.backup_withholding_amount_dollars, 24_000);
        assert_eq!(r.net_payment_to_payee_dollars, 76_000);
    }

    #[test]
    fn very_large_payment_precision() {
        let mut i = base();
        i.tin_furnished = false;
        i.payment_amount_dollars = 1_000_000_000;
        let r = compute(&i);
        assert_eq!(r.backup_withholding_amount_dollars, 240_000_000);
    }

    #[test]
    fn zero_payment_zero_withholding() {
        let mut i = base();
        i.tin_furnished = false;
        i.payment_amount_dollars = 0;
        let r = compute(&i);
        assert_eq!(r.backup_withholding_amount_dollars, 0);
        assert_eq!(r.net_payment_to_payee_dollars, 0);
    }

    // Payment type sweep.

    #[test]
    fn all_payment_types_subject_to_a_trigger() {
        // TIN-not-furnished (§3406(a)(1)(A)) applies to all reportable
        // payment types, not limited like the C trigger.
        for ptype in &[
            ReportablePaymentType::Interest,
            ReportablePaymentType::Dividends,
            ReportablePaymentType::Rent,
            ReportablePaymentType::Royalties,
            ReportablePaymentType::NonEmployeeCompensation,
            ReportablePaymentType::BrokerProceeds,
            ReportablePaymentType::Barter,
            ReportablePaymentType::AttorneyFees,
            ReportablePaymentType::FishingBoatPayments,
            ReportablePaymentType::Other,
        ] {
            let mut i = base();
            i.payment_type = *ptype;
            i.tin_furnished = false;
            let r = compute(&i);
            assert!(
                r.backup_withholding_required,
                "expected A trigger to fire for {:?}",
                ptype
            );
        }
    }

    // Notes / citations.

    #[test]
    fn triggered_note_describes_trigger_and_24_pct() {
        let mut i = base();
        i.tin_furnished = false;
        let r = compute(&i);
        assert!(r.note.contains("BACKUP WITHHOLDING TRIGGERED"));
        assert!(r.note.contains("24%"));
        assert!(r.note.contains("TinNotFurnished"));
    }

    #[test]
    fn untriggered_note_describes_no_trigger() {
        let r = compute(&base());
        assert!(r.note.contains("no trigger fired"));
    }

    #[test]
    fn citation_mentions_all_4_trigger_subsections() {
        let r = compute(&base());
        assert!(r.citation.contains("§3406(a)(1)(A)"));
        assert!(r.citation.contains("§3406(a)(1)(B)"));
        assert!(r.citation.contains("§3406(a)(1)(C)"));
        assert!(r.citation.contains("§3406(a)(1)(D)"));
        assert!(r.citation.contains("§3406(b)(1)(A)"));
        assert!(r.citation.contains("CP 2100"));
        assert!(r.citation.contains("Publication 1281"));
    }
}
