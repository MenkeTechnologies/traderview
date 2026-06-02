//! IRC § 6049 — Returns regarding payments of interest.
//! Trader-relevant information-return module because traders
//! receive interest income from money market funds, T-bills
//! and Treasury securities (federal interest), municipal
//! bonds (tax-exempt for federal but state-reportable),
//! corporate bonds (taxable interest), original issue
//! discount (OID) on zero-coupon securities, bank deposit
//! interest, brokerage cash-balance interest, and § 988
//! foreign currency contracts. § 6049 governs the 1099-INT
//! and 1099-OID reporting framework. Companion to
//! section_6041 general info returns and section_6045
//! broker reporting and section_6042 dividends info
//! returns and section_6050W 1099-K payment card and
//! § 3406 backup withholding and § 6038D foreign asset
//! reporting.
//!
//! **§ 6049(a) Requirement of reporting** — every person:
//! 1. Who **makes payments of interest** aggregating **$10
//!    or more** to any person during any calendar year; OR
//! 2. Who **receives payments of interest** as a NOMINEE
//!    aggregating $10 or more during any calendar year and
//!    who makes payment to any other person;
//!
//! SHALL make a return:
//! 1. **Setting forth the aggregate amount** of such
//!    payments and the **name and address** of the person
//!    to whom paid; AND
//! 2. **Furnishing such other information** as the
//!    Secretary may by regulations prescribe.
//!
//! **§ 6049(b) Interest defined** — for purposes of
//! § 6049(a), the term "interest" means:
//! 1. Interest on **any obligation** issued in registered
//!    form OR in another form prescribed by the Secretary;
//! 2. Interest on **deposits** with persons carrying on
//!    banking business;
//! 3. Amounts (whether or not designated as interest) paid
//!    by **mutual savings bank, cooperative bank, building
//!    and loan association, homestead association**;
//! 4. Interest on amounts held by **insurance company**
//!    under agreement to pay interest;
//! 5. **Original issue discount** (OID) per § 1272;
//! 6. Interest on **amounts held by broker-dealer**
//!    custodial accounts;
//! 7. Interest on certain **tax-exempt obligations**
//!    (subject to § 103(a) carve-out for interest on state
//!    and local bonds).
//!
//! **§ 6049(c) Statements to be furnished** — every person
//! required to make a return under § 6049(a) shall furnish
//! to each person whose name is required to be set forth
//! a **written statement** showing (1) name + address of
//! the person making such return, (2) aggregate amount of
//! interest payments to person required to be shown on the
//! return.
//!
//! **§ 6049(d) Definitions and special rules**:
//! 1. **Nominee reporting** — when interest payment is
//!    received by a nominee on behalf of beneficial owner,
//!    nominee must file pass-through return identifying
//!    beneficial owner.
//! 2. **Middleman / broker** — broker (as defined in
//!    § 6045(c)) or nominee who makes payment of interest
//!    for or collects interest on behalf of another person
//!    is considered "middleman" subject to § 6049 reporting.
//! 3. **Original issue discount holder reporting** — § 6049
//!    applies to each calendar year of any holder of an
//!    obligation as to which OID is includible in gross
//!    income aggregating $10 or more.
//!
//! **§ 6049(e) Backup withholding under § 3406** — if
//! payor backup withholds under § 3406 on a payment (e.g.,
//! because payee failed to furnish Form W-9 with correct
//! TIN), payor MUST make a return under § 6049 even if
//! aggregate interest is less than $10. Backup withholding
//! triggers reporting irrespective of $10 threshold.
//!
//! **Form 1099-INT** — reports taxable interest under §
//! 6049 (Box 1 interest income, Box 2 early withdrawal
//! penalty, Box 3 interest on US savings bonds and
//! Treasury obligations, Box 4 federal tax withheld
//! including § 3406 backup, Box 8 tax-exempt interest, Box
//! 9 specified private activity bond interest).
//!
//! **Form 1099-OID** — reports original issue discount
//! under § 6049 + § 1272 OID rules (Box 1 OID for current
//! year, Box 2 other periodic interest, Box 5 market
//! discount, Box 6 acquisition premium).
//!
//! **Trader-relevant 1099-INT/OID sources**:
//! - **Treasury securities** (T-bills + T-notes + T-bonds +
//!   TIPS + Series I bonds) — federal tax but state-tax-
//!   exempt;
//! - **Municipal bonds** — federal tax-exempt under §
//!   103(a) but reportable on 1099-INT Box 8;
//! - **Corporate bonds** — fully taxable; OID reported on
//!   1099-OID;
//! - **Zero-coupon securities** — OID reported even though
//!   no cash interest paid;
//! - **Money market funds** — interest distributions on
//!   1099-INT (or 1099-DIV for SEC-registered funds);
//! - **Bank deposit interest** — checking/savings/CD
//!   interest;
//! - **Brokerage cash-balance interest** — interest on
//!   uninvested cash in brokerage account;
//! - **§ 988 foreign currency** — gain/loss on foreign
//!   currency transactions (ordinary income).
//!
//! Citations: 26 USC § 6049(a)-(e); 26 CFR §§ 1.6049-1
//! through 1.6049-7; Form 1099-INT + Form 1099-OID (2024
//! instructions); § 6041 general info returns; § 6042
//! dividends; § 6045 broker reporting; § 6045A transfer
//! statements; § 6045B issuer returns; § 6050W 1099-K
//! payment card; § 3406 backup withholding; § 103(a) tax-
//! exempt municipal bond interest; § 1272 OID rules; § 988
//! foreign currency.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InterestType {
    /// § 6049(b)(1) — interest on registered-form
    /// obligations (corporate bonds, notes).
    RegisteredObligation,
    /// § 6049(b)(2) — bank deposit interest (checking,
    /// savings, CD).
    BankDeposit,
    /// § 6049(b)(3) — mutual savings/cooperative bank/
    /// building and loan/homestead association interest.
    SavingsInstitutionInterest,
    /// § 6049(b)(4) — insurance company-held interest.
    InsuranceCompanyInterest,
    /// § 6049(b)(5) — original issue discount per § 1272.
    OriginalIssueDiscount,
    /// § 6049(b)(6) — broker-dealer custodial interest.
    BrokerDealerCustodial,
    /// § 6049(b)(7) — Treasury obligations and US savings
    /// bonds (federal tax but state-exempt).
    TreasuryObligation,
    /// Municipal bond interest (federal tax-exempt under §
    /// 103(a), but reportable on 1099-INT Box 8).
    MunicipalBondTaxExempt,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6049Input {
    pub interest_type: InterestType,
    /// Aggregate interest payments to payee in cents during
    /// calendar year.
    pub aggregate_interest_cents: u64,
    /// Whether payor is a broker or nominee (middleman)
    /// under § 6045(c).
    pub payor_is_middleman: bool,
    /// Whether payor backup-withheld under § 3406 (triggers
    /// reporting irrespective of $10 threshold).
    pub backup_withholding_applied: bool,
    /// Whether 1099-INT or 1099-OID return was filed with
    /// IRS.
    pub return_filed: bool,
    /// Whether written statement was furnished to recipient
    /// (§ 6049(c)).
    pub written_statement_to_recipient_furnished: bool,
    /// Whether payee is exempt recipient (e.g., corporation,
    /// tax-exempt org, foreign government).
    pub payee_exempt_recipient: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6049Result {
    pub reporting_required: bool,
    pub ten_dollar_threshold_met: bool,
    pub backup_withholding_overrides_threshold: bool,
    pub return_filing_compliant: bool,
    pub statement_to_recipient_compliant: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6049Input) -> Section6049Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    const TEN_DOLLAR_THRESHOLD_CENTS: u64 = 1_000;
    let threshold_met = input.aggregate_interest_cents >= TEN_DOLLAR_THRESHOLD_CENTS;

    let backup_overrides = input.backup_withholding_applied;

    let exempt_recipient = input.payee_exempt_recipient && !input.backup_withholding_applied;

    let required = (threshold_met || backup_overrides) && !exempt_recipient;

    if required && !input.return_filed {
        failure_reasons.push(
            "26 USC § 6049(a) + 26 CFR § 1.6049-4 — Form 1099-INT or Form 1099-OID return required when aggregate interest payments of $10 or more (or any amount with § 3406 backup withholding) to non-exempt recipient".to_string(),
        );
    }

    if required && !input.written_statement_to_recipient_furnished {
        failure_reasons.push(
            "26 USC § 6049(c) + 26 CFR § 1.6049-6 — written statement showing aggregate interest amount must be furnished to recipient by January 31 of year following payment".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "26 USC § 6049(a) — every person who makes payments of interest aggregating $10 or more during calendar year, or who receives interest as nominee aggregating $10 or more and pays to another, SHALL make a return setting forth aggregate amount + name and address of recipient".to_string(),
        "26 USC § 6049(b) — interest defined: (1) registered-form obligations; (2) bank deposit interest; (3) mutual savings + cooperative + building/loan + homestead association; (4) insurance company-held; (5) original issue discount per § 1272; (6) broker-dealer custodial; (7) Treasury obligations (federal tax + state-exempt)".to_string(),
        "26 USC § 6049(c) + 26 CFR § 1.6049-6 — written statement to recipient required showing name + address of payor and aggregate interest amount; January 31 deadline of year following payment".to_string(),
        "26 USC § 6049(d) — nominee/middleman pass-through reporting; broker (§ 6045(c)) or nominee receiving interest for another is middleman subject to § 6049; OID holder reporting when OID includible in gross income aggregating $10+".to_string(),
        "26 USC § 6049(e) + § 3406 — backup withholding triggers reporting IRRESPECTIVE of $10 threshold; payor must make return under § 6049 if § 3406 backup withholds (payee failed to furnish W-9 with correct TIN)".to_string(),
        "Form 1099-INT — Box 1 interest income; Box 2 early withdrawal penalty; Box 3 US savings bond/Treasury obligation interest (federal tax + state-exempt); Box 4 federal tax withheld (§ 3406 backup); Box 8 tax-exempt interest (municipal bonds § 103(a)); Box 9 specified private activity bond interest".to_string(),
        "Form 1099-OID — Box 1 OID current year; Box 2 other periodic interest; Box 5 market discount; Box 6 acquisition premium; reported per § 6049 + § 1272 OID rules".to_string(),
        "Trader-relevant 1099-INT/OID sources: Treasury securities (T-bills + T-notes + TIPS + Series I bonds federal tax/state-exempt); municipal bonds (federal tax-exempt § 103(a)); corporate bonds (fully taxable); zero-coupon securities (OID); money market funds; bank deposit; brokerage cash-balance; § 988 foreign currency".to_string(),
        "26 CFR §§ 1.6049-1 through 1.6049-7 — comprehensive regulatory framework for 1099-INT + 1099-OID reporting + exempt recipients + foreign payee withholding + middleman pass-through".to_string(),
        "Cross-references: § 6041 general info returns; § 6042 dividends 1099-DIV; § 6045 broker reporting; § 6045A transfer statements; § 6045B issuer returns; § 6050W 1099-K payment card; § 3406 backup withholding; § 103(a) tax-exempt muni; § 1272 OID; § 988 foreign currency".to_string(),
    ];

    Section6049Result {
        reporting_required: required,
        ten_dollar_threshold_met: threshold_met,
        backup_withholding_overrides_threshold: backup_overrides,
        return_filing_compliant: !required || input.return_filed,
        statement_to_recipient_compliant: !required
            || input.written_statement_to_recipient_furnished,
        failure_reasons,
        citation: "26 USC § 6049(a)-(e); 26 CFR §§ 1.6049-1 through 1.6049-7; Form 1099-INT + Form 1099-OID; § 6041; § 6042; § 6045; § 6045A; § 6045B; § 6050W; § 3406; § 103(a); § 1272; § 988",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_base() -> Section6049Input {
        Section6049Input {
            interest_type: InterestType::BankDeposit,
            aggregate_interest_cents: 5_000,
            payor_is_middleman: false,
            backup_withholding_applied: false,
            return_filed: true,
            written_statement_to_recipient_furnished: true,
            payee_exempt_recipient: false,
        }
    }

    #[test]
    fn fifty_dollars_bank_interest_filed_compliant() {
        let r = check(&valid_base());
        assert!(r.reporting_required);
        assert!(r.ten_dollar_threshold_met);
        assert!(r.return_filing_compliant);
        assert!(r.statement_to_recipient_compliant);
    }

    #[test]
    fn ten_dollar_boundary_compliant() {
        let mut i = valid_base();
        i.aggregate_interest_cents = 1_000;
        let r = check(&i);
        assert!(r.ten_dollar_threshold_met);
        assert!(r.reporting_required);
    }

    #[test]
    fn nine_dollar_below_threshold_no_reporting() {
        let mut i = valid_base();
        i.aggregate_interest_cents = 999;
        let r = check(&i);
        assert!(!r.ten_dollar_threshold_met);
        assert!(!r.reporting_required);
    }

    #[test]
    fn backup_withholding_overrides_threshold() {
        let mut i = valid_base();
        i.aggregate_interest_cents = 100;
        i.backup_withholding_applied = true;
        let r = check(&i);
        assert!(r.backup_withholding_overrides_threshold);
        assert!(r.reporting_required);
    }

    #[test]
    fn not_filed_when_required_violation() {
        let mut i = valid_base();
        i.return_filed = false;
        let r = check(&i);
        assert!(!r.return_filing_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6049(a)") && f.contains("1099-INT")));
    }

    #[test]
    fn no_statement_to_recipient_violation() {
        let mut i = valid_base();
        i.written_statement_to_recipient_furnished = false;
        let r = check(&i);
        assert!(!r.statement_to_recipient_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6049(c)") && f.contains("January 31")));
    }

    #[test]
    fn exempt_recipient_no_reporting() {
        let mut i = valid_base();
        i.payee_exempt_recipient = true;
        let r = check(&i);
        assert!(!r.reporting_required);
    }

    #[test]
    fn exempt_recipient_with_backup_withholding_still_required() {
        let mut i = valid_base();
        i.payee_exempt_recipient = true;
        i.backup_withholding_applied = true;
        let r = check(&i);
        assert!(r.reporting_required);
    }

    #[test]
    fn registered_obligation_interest_in_scope() {
        let mut i = valid_base();
        i.interest_type = InterestType::RegisteredObligation;
        let r = check(&i);
        assert!(r.reporting_required);
    }

    #[test]
    fn savings_institution_interest_in_scope() {
        let mut i = valid_base();
        i.interest_type = InterestType::SavingsInstitutionInterest;
        let r = check(&i);
        assert!(r.reporting_required);
    }

    #[test]
    fn insurance_company_interest_in_scope() {
        let mut i = valid_base();
        i.interest_type = InterestType::InsuranceCompanyInterest;
        let r = check(&i);
        assert!(r.reporting_required);
    }

    #[test]
    fn original_issue_discount_in_scope() {
        let mut i = valid_base();
        i.interest_type = InterestType::OriginalIssueDiscount;
        let r = check(&i);
        assert!(r.reporting_required);
    }

    #[test]
    fn broker_dealer_custodial_in_scope() {
        let mut i = valid_base();
        i.interest_type = InterestType::BrokerDealerCustodial;
        let r = check(&i);
        assert!(r.reporting_required);
    }

    #[test]
    fn treasury_obligation_in_scope() {
        let mut i = valid_base();
        i.interest_type = InterestType::TreasuryObligation;
        let r = check(&i);
        assert!(r.reporting_required);
    }

    #[test]
    fn municipal_bond_tax_exempt_in_scope_for_reporting() {
        let mut i = valid_base();
        i.interest_type = InterestType::MunicipalBondTaxExempt;
        let r = check(&i);
        assert!(r.reporting_required);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&valid_base());
        assert!(r.citation.contains("§ 6049(a)-(e)"));
        assert!(r.citation.contains("26 CFR §§ 1.6049-1 through 1.6049-7"));
        assert!(r.citation.contains("Form 1099-INT"));
        assert!(r.citation.contains("Form 1099-OID"));
        assert!(r.citation.contains("§ 6041"));
        assert!(r.citation.contains("§ 6042"));
        assert!(r.citation.contains("§ 6045"));
        assert!(r.citation.contains("§ 6050W"));
        assert!(r.citation.contains("§ 3406"));
        assert!(r.citation.contains("§ 103(a)"));
        assert!(r.citation.contains("§ 1272"));
        assert!(r.citation.contains("§ 988"));
    }

    #[test]
    fn note_pins_subsection_a_10_dollar_minimum() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6049(a)")
            && n.contains("$10 or more")
            && n.contains("aggregate amount")
            && n.contains("name and address")));
    }

    #[test]
    fn note_pins_subsection_b_seven_interest_categories() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6049(b)")
            && n.contains("registered-form")
            && n.contains("bank deposit")
            && n.contains("mutual savings")
            && n.contains("insurance company")
            && n.contains("original issue discount")
            && n.contains("broker-dealer custodial")
            && n.contains("Treasury obligations")));
    }

    #[test]
    fn note_pins_subsection_c_written_statement_january_31() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6049(c)")
            && n.contains("§ 1.6049-6")
            && n.contains("January 31")));
    }

    #[test]
    fn note_pins_subsection_d_nominee_middleman_oid() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6049(d)")
            && n.contains("nominee")
            && n.contains("middleman")
            && n.contains("§ 6045(c)")
            && n.contains("OID")));
    }

    #[test]
    fn note_pins_subsection_e_backup_withholding_overrides() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6049(e)")
            && n.contains("§ 3406")
            && n.contains("IRRESPECTIVE")));
    }

    #[test]
    fn note_pins_1099_int_box_breakdown() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("Form 1099-INT")
            && n.contains("Box 1")
            && n.contains("Box 3")
            && n.contains("Box 4")
            && n.contains("Box 8")
            && n.contains("§ 103(a)")));
    }

    #[test]
    fn note_pins_1099_oid_box_breakdown() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("Form 1099-OID")
            && n.contains("Box 1 OID")
            && n.contains("Box 5 market discount")
            && n.contains("§ 1272")));
    }

    #[test]
    fn note_pins_trader_relevant_sources() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("Treasury securities")
            && n.contains("TIPS")
            && n.contains("Series I bonds")
            && n.contains("municipal bonds")
            && n.contains("zero-coupon")
            && n.contains("§ 988")));
    }

    #[test]
    fn note_pins_cross_references() {
        let r = check(&valid_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6041")
            && n.contains("§ 6042")
            && n.contains("§ 6045")
            && n.contains("§ 6050W")
            && n.contains("§ 3406")));
    }

    #[test]
    fn interest_type_truth_table_eight_cells() {
        for interest_type in [
            InterestType::RegisteredObligation,
            InterestType::BankDeposit,
            InterestType::SavingsInstitutionInterest,
            InterestType::InsuranceCompanyInterest,
            InterestType::OriginalIssueDiscount,
            InterestType::BrokerDealerCustodial,
            InterestType::TreasuryObligation,
            InterestType::MunicipalBondTaxExempt,
        ] {
            let mut i = valid_base();
            i.interest_type = interest_type;
            let r = check(&i);
            assert!(r.reporting_required);
        }
    }

    #[test]
    fn backup_withholding_uniquely_overrides_threshold_invariant() {
        let mut i_below = valid_base();
        i_below.aggregate_interest_cents = 500;
        i_below.backup_withholding_applied = false;
        let r_below = check(&i_below);
        assert!(!r_below.reporting_required);

        let mut i_backup = valid_base();
        i_backup.aggregate_interest_cents = 500;
        i_backup.backup_withholding_applied = true;
        let r_backup = check(&i_backup);
        assert!(r_backup.reporting_required);
    }

    #[test]
    fn exempt_recipient_normally_no_reporting_invariant() {
        let mut i = valid_base();
        i.payee_exempt_recipient = true;
        i.backup_withholding_applied = false;
        let r = check(&i);
        assert!(!r.reporting_required);
    }

    #[test]
    fn ten_dollar_threshold_boundary_invariant() {
        let mut i_at = valid_base();
        i_at.aggregate_interest_cents = 1_000;
        let r_at = check(&i_at);
        assert!(r_at.ten_dollar_threshold_met);

        let mut i_under = valid_base();
        i_under.aggregate_interest_cents = 999;
        let r_under = check(&i_under);
        assert!(!r_under.ten_dollar_threshold_met);
    }

    #[test]
    fn multiple_failures_stack() {
        let mut i = valid_base();
        i.return_filed = false;
        i.written_statement_to_recipient_furnished = false;
        let r = check(&i);
        assert_eq!(r.failure_reasons.len(), 2);
    }

    #[test]
    fn defensive_zero_interest_no_reporting() {
        let mut i = valid_base();
        i.aggregate_interest_cents = 0;
        let r = check(&i);
        assert!(!r.reporting_required);
    }
}
