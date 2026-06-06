//! Landlord property sale notice and security deposit
//! transfer disclosure to tenant — trader-landlord
//! critical because traders who exit rental property
//! via sale must comply with state-by-state security
//! deposit transfer + new owner notice requirements
//! within tight statutory windows (5 days NY, 45 days
//! MA, "reasonable time" CA). Failure triggers JOINT
//! AND SEVERAL liability of seller and buyer for the
//! deposit, and exposes the seller to TREBLE DAMAGES
//! in Massachusetts.
//!
//! Companion to security_deposit_bank_disclosure,
//! deposit_interest, deposit_return_windows,
//! foreclosure_tenant_rights, landlord_identification_
//! disclosure, tenant_estoppel_certificate (iter 435).
//!
//! **Four-jurisdiction framework**:
//!
//! - **California — Cal. Civ. Code § 1950.5(h)** —
//!   upon termination of landlord's interest (sale,
//!   assignment, death, receivership, or otherwise),
//!   landlord must within REASONABLE TIME (1) transfer
//!   security deposit (less lawful deductions) to
//!   successor in interest AND (2) notify tenant by
//!   PERSONAL DELIVERY or FIRST-CLASS MAIL of:
//!   (a) the transfer; (b) any claims made against
//!   the deposit; (c) the amount of the deposit;
//!   (d) successor's name, address, and telephone
//!   number. Failure triggers JOINT AND SEVERAL
//!   LIABILITY of seller and buyer for repayment.
//!
//! - **New York — NY GOL § 7-105** — within 5 DAYS
//!   of conveyance, landlord must (1) deliver deposit
//!   to grantee/assignee AND (2) notify tenant by
//!   REGISTERED OR CERTIFIED MAIL of new owner's
//!   name and address. Failure causes grantee or
//!   assignee to be JOINTLY AND SEVERALLY LIABLE
//!   with seller for repayment plus accrued interest.
//!   NY GOL § 7-103(2) requires interest-bearing
//!   account for 6+ family dwelling units, with
//!   accrued interest transferred to successor.
//!
//! - **Massachusetts — Mass. Gen. Laws c. 186
//!   § 15B(7)** — within 45 DAYS of sale, new owner
//!   (NOT seller) must provide tenant written notice
//!   that (1) new owner has received security deposit
//!   and (2) where it is being held. Successor is
//!   responsible for return regardless of receipt
//!   from prior owner. Mass. Gen. Laws c. 186
//!   § 15B(7) treble-damages remedy available for
//!   willful violation.
//!
//! - **Default** — common law requires seller to
//!   transfer deposit to successor as part of closing;
//!   no statutory window; tenant may recover from
//!   either party but typically must pursue seller
//!   first; UCC does NOT apply to real estate; many
//!   states (TX, FL, GA) follow CA-style "reasonable
//!   time" rule.
//!
//! **Trader-landlord critical fact patterns**:
//! 1. Trader sells 12-unit building in NYC for $5M;
//!    transfers $36,000 in security deposits to buyer
//!    at closing; fails to mail certified notice to
//!    tenants within 5 days; tenants later sue both
//!    seller and buyer under § 7-105 joint and
//!    several liability.
//! 2. Trader sells CA rental; transfers deposit to
//!    buyer; mails notice by REGULAR mail (not first-
//!    class or personal delivery); notice INVALID
//!    under § 1950.5(h); joint and several liability
//!    attaches.
//! 3. Trader sells MA rental; new owner fails to
//!    deliver § 15B(7) 45-day notice; tenant entitled
//!    to TREBLE DAMAGES against new owner regardless
//!    of seller compliance.
//! 4. Trader sells rental during foreclosure under
//!    Protecting Tenants at Foreclosure Act (PTFA);
//!    deposit transfer + notice obligations stack on
//!    top of PTFA 90-day notice; failure to comply
//!    with EITHER triggers separate liability.
//! 5. Trader sells rental with pending § 1950.5
//!    DEDUCTION CLAIM for damages; § 1950.5(h)
//!    requires deposit transfer NET of "any lawful
//!    deductions" AND notice of "any claims made
//!    against the security."
//!
//! Citations: Cal. Civ. Code § 1950.5(h); NY GOL
//! § 7-103(2); NY GOL § 7-105; Mass. Gen. Laws c. 186
//! § 15B(7); 12 USC § 5220 (Protecting Tenants at
//! Foreclosure Act of 2009); Restatement (Second) of
//! Property: Landlord and Tenant § 12.1; UCC Article
//! 9 (real property excluded).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    NewYork,
    Massachusetts,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NoticeMethod {
    PersonalDelivery,
    FirstClassMail,
    RegisteredOrCertifiedMail,
    RegularMail,
    Email,
    NoNotice,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LandlordPropertySaleNoticeInput {
    pub jurisdiction: Jurisdiction,
    /// Days since sale closing.
    pub days_since_sale: u32,
    /// Whether seller transferred deposit to buyer.
    pub deposit_transferred_to_buyer: bool,
    /// Whether tenant received written notice of new
    /// owner's name and address.
    pub tenant_received_new_owner_notice: bool,
    /// Method by which tenant notice was given.
    pub notice_method: NoticeMethod,
    /// Whether notice disclosed amount of deposit
    /// transferred (CA requirement).
    pub notice_includes_deposit_amount: bool,
    /// Whether notice disclosed any claims/deductions
    /// against deposit (CA requirement).
    pub notice_includes_deduction_claims: bool,
    /// Whether NY building has 6+ family dwelling units
    /// (triggers § 7-103(2) interest-bearing requirement).
    pub ny_six_plus_unit_building: bool,
    /// Whether accrued interest was transferred to
    /// successor (NY § 7-103(2)).
    pub accrued_interest_transferred: bool,
    /// Whether sale occurred during foreclosure
    /// (PTFA stacks notice obligations).
    pub sale_during_foreclosure: bool,
    /// Whether landlord/seller willfully refused to
    /// transfer or notify (MA treble damages trigger).
    pub willful_violation: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LandlordPropertySaleNoticeResult {
    pub jurisdiction_window_days: u32,
    pub deposit_transfer_compliant: bool,
    pub tenant_notice_compliant: bool,
    pub notice_method_valid: bool,
    pub joint_and_several_liability_attaches: bool,
    pub treble_damages_available: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &LandlordPropertySaleNoticeInput) -> LandlordPropertySaleNoticeResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let jurisdiction_window_days: u32 = match input.jurisdiction {
        Jurisdiction::California => 30,
        Jurisdiction::NewYork => 5,
        Jurisdiction::Massachusetts => 45,
        Jurisdiction::Default => 60,
    };

    let notice_method_valid = match input.jurisdiction {
        Jurisdiction::California => matches!(
            input.notice_method,
            NoticeMethod::PersonalDelivery | NoticeMethod::FirstClassMail
        ),
        Jurisdiction::NewYork => {
            matches!(input.notice_method, NoticeMethod::RegisteredOrCertifiedMail)
        }
        Jurisdiction::Massachusetts => !matches!(input.notice_method, NoticeMethod::NoNotice),
        Jurisdiction::Default => !matches!(input.notice_method, NoticeMethod::NoNotice),
    };

    let within_window = input.days_since_sale <= jurisdiction_window_days;

    let deposit_transfer_compliant = input.deposit_transferred_to_buyer && within_window;

    let basic_notice =
        input.tenant_received_new_owner_notice && notice_method_valid && within_window;

    let tenant_notice_compliant = match input.jurisdiction {
        Jurisdiction::California => {
            basic_notice
                && input.notice_includes_deposit_amount
                && input.notice_includes_deduction_claims
        }
        _ => basic_notice,
    };

    let joint_and_several_liability_attaches =
        !(deposit_transfer_compliant && tenant_notice_compliant);

    let treble_damages_available =
        matches!(input.jurisdiction, Jurisdiction::Massachusetts) && input.willful_violation;

    if !input.deposit_transferred_to_buyer {
        failure_reasons.push(
            "Deposit NOT TRANSFERRED to successor; seller remains primary obligor; tenant may recover from either party under joint and several liability".to_string(),
        );
    }

    if !within_window {
        match input.jurisdiction {
            Jurisdiction::California => failure_reasons.push(format!(
                "Cal. Civ. Code § 1950.5(h) — REASONABLE TIME standard exceeded ({} days); 30-day benchmark applied; joint and several liability attaches",
                input.days_since_sale
            )),
            Jurisdiction::NewYork => failure_reasons.push(format!(
                "NY GOL § 7-105 — 5-DAY transfer/notice window EXCEEDED ({} days); grantee jointly and severally liable with seller for repayment of deposit plus accrued interest",
                input.days_since_sale
            )),
            Jurisdiction::Massachusetts => failure_reasons.push(format!(
                "Mass. Gen. Laws c. 186 § 15B(7) — 45-DAY notice window EXCEEDED ({} days); new owner liable for security deposit return regardless of receipt from prior owner",
                input.days_since_sale
            )),
            Jurisdiction::Default => failure_reasons.push(format!(
                "Common-law REASONABLE TIME standard exceeded ({} days); 60-day benchmark applied; tenant may pursue seller for repayment",
                input.days_since_sale
            )),
        }
    }

    if !input.tenant_received_new_owner_notice {
        failure_reasons.push(
            "Tenant DID NOT RECEIVE written notice of new owner's name and address; statutory notice obligation breached; joint and several liability attaches".to_string(),
        );
    }

    if !notice_method_valid {
        match input.jurisdiction {
            Jurisdiction::California => failure_reasons.push(
                "Cal. Civ. Code § 1950.5(h) — notice must be by PERSONAL DELIVERY or FIRST-CLASS MAIL, postage prepaid; alternative methods (regular mail, email, no notice) INVALID".to_string(),
            ),
            Jurisdiction::NewYork => failure_reasons.push(
                "NY GOL § 7-105 — notice must be by REGISTERED OR CERTIFIED MAIL; alternative methods INVALID; joint and several liability attaches".to_string(),
            ),
            Jurisdiction::Massachusetts => failure_reasons.push(
                "Mass. Gen. Laws c. 186 § 15B(7) — written notice required; NO NOTICE invalid".to_string(),
            ),
            Jurisdiction::Default => failure_reasons.push(
                "Common-law requires written notice; NO NOTICE invalid".to_string(),
            ),
        }
    }

    if matches!(input.jurisdiction, Jurisdiction::California) {
        if !input.notice_includes_deposit_amount {
            failure_reasons.push(
                "Cal. Civ. Code § 1950.5(h) — notice must include AMOUNT of deposit transferred; omission renders notice INVALID".to_string(),
            );
        }
        if !input.notice_includes_deduction_claims {
            failure_reasons.push(
                "Cal. Civ. Code § 1950.5(h) — notice must include disclosure of ANY CLAIMS made against security; omission renders notice INVALID".to_string(),
            );
        }
    }

    if matches!(input.jurisdiction, Jurisdiction::NewYork)
        && input.ny_six_plus_unit_building
        && !input.accrued_interest_transferred
    {
        failure_reasons.push(
            "NY GOL § 7-103(2) — building with 6+ family dwelling units requires interest-bearing account; ACCRUED INTEREST must be transferred to successor with the principal; failure breaches statute".to_string(),
        );
    }

    if treble_damages_available {
        failure_reasons.push(
            "Mass. Gen. Laws c. 186 § 15B(7) — WILLFUL violation triggers TREBLE DAMAGES remedy against landlord/seller; plus attorney's fees".to_string(),
        );
    }

    if input.sale_during_foreclosure {
        failure_reasons.push(
            "12 USC § 5220 (Protecting Tenants at Foreclosure Act of 2009) — sale during foreclosure STACKS PTFA 90-day notice obligation on top of state-law deposit transfer/notice requirements; failure to comply with EITHER triggers separate liability".to_string(),
        );
    }

    if joint_and_several_liability_attaches {
        failure_reasons.push(
            "JOINT AND SEVERAL LIABILITY ATTACHES — both seller (former landlord) and buyer (successor in interest) liable to tenant for security deposit repayment under jurisdiction's statutory transfer regime".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "Four-jurisdiction framework: CALIFORNIA (Cal. Civ. Code § 1950.5(h) — reasonable time + 4-element notice); NEW YORK (NY GOL § 7-105 — 5-DAY window + registered/certified mail; NY GOL § 7-103(2) interest-bearing 6+ unit); MASSACHUSETTS (Mass. Gen. Laws c. 186 § 15B(7) — 45-DAY new-owner notice + treble damages); DEFAULT (common-law reasonable time)".to_string(),
        "Cal. Civ. Code § 1950.5(h) requirements: (1) transfer deposit (less lawful deductions) to successor in interest; (2) notify tenant by PERSONAL DELIVERY or FIRST-CLASS MAIL, postage prepaid; (3) notice must include AMOUNT of deposit; (4) any claims made against the security; (5) successor's name, address, and telephone number; FAILURE triggers JOINT AND SEVERAL LIABILITY of seller and buyer for repayment".to_string(),
        "NY GOL § 7-105 — within 5 DAYS of conveyance, landlord must (1) deliver deposit to grantee/assignee AND (2) notify tenant by REGISTERED OR CERTIFIED MAIL of new owner's name and address; failure causes grantee or assignee to be JOINTLY AND SEVERALLY LIABLE with seller for repayment PLUS ACCRUED INTEREST".to_string(),
        "NY GOL § 7-103(2) — building with 6+ family dwelling units requires interest-bearing account; 1% per annum administration expense retained by landlord; remaining accrued interest belongs to tenant and must be transferred to successor with principal at sale".to_string(),
        "Mass. Gen. Laws c. 186 § 15B(7) — within 45 DAYS of sale, NEW OWNER must provide tenant written notice that (1) new owner has received security deposit and (2) where it is being held; successor is responsible for return REGARDLESS of receipt from prior owner; willful violation triggers TREBLE DAMAGES plus attorney's fees".to_string(),
        "Default / common law — seller must transfer deposit to successor as part of closing; no fixed statutory window; tenant may recover from either party but typically must pursue seller first; UCC Article 9 does NOT apply to real estate (real property excluded)".to_string(),
        "12 USC § 5220 (Protecting Tenants at Foreclosure Act of 2009) — sale during foreclosure STACKS 90-day notice obligation on top of state-law deposit transfer/notice requirements; failure to comply with EITHER triggers separate liability".to_string(),
        "Restatement (Second) of Property: Landlord and Tenant § 12.1 — landlord's interest in security deposit passes to successor in interest upon conveyance; tenant entitled to actual notice of transfer and identity of new obligor".to_string(),
        "Joint and several liability rule — failure to transfer deposit OR notify tenant triggers liability of BOTH seller and buyer; tenant may sue either party; equitable contribution between parties governed by state law".to_string(),
        "Trader-landlord critical fact patterns: (1) trader sells 12-unit NYC building $5M, transfers $36K deposits but fails 5-day certified-mail notice — joint and several liability; (2) CA seller mails notice by regular mail (not first-class or personal) — notice INVALID under § 1950.5(h); (3) MA new owner fails 45-day notice — TREBLE DAMAGES; (4) sale during foreclosure stacks PTFA 90-day on top of deposit transfer rules; (5) CA pending deduction claim — § 1950.5(h) requires NET transfer + disclosure of claims".to_string(),
        "Companion to security_deposit_bank_disclosure (where is deposit held?) + deposit_interest (interest payment obligation) + deposit_return_windows (refund timelines) + foreclosure_tenant_rights (sale during foreclosure) + landlord_identification_disclosure (initial owner-name disclosure) + tenant_estoppel_certificate (refinance/sale estoppel framework)".to_string(),
    ];

    LandlordPropertySaleNoticeResult {
        jurisdiction_window_days,
        deposit_transfer_compliant,
        tenant_notice_compliant,
        notice_method_valid,
        joint_and_several_liability_attaches,
        treble_damages_available,
        failure_reasons,
        citation: "Cal. Civ. Code § 1950.5(h); NY GOL § 7-103(2); NY GOL § 7-105; Mass. Gen. Laws c. 186 § 15B(7); 12 USC § 5220 (Protecting Tenants at Foreclosure Act of 2009); Restatement (Second) of Property: Landlord and Tenant § 12.1",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_baseline_compliant() -> LandlordPropertySaleNoticeInput {
        LandlordPropertySaleNoticeInput {
            jurisdiction: Jurisdiction::California,
            days_since_sale: 7,
            deposit_transferred_to_buyer: true,
            tenant_received_new_owner_notice: true,
            notice_method: NoticeMethod::FirstClassMail,
            notice_includes_deposit_amount: true,
            notice_includes_deduction_claims: true,
            ny_six_plus_unit_building: false,
            accrued_interest_transferred: false,
            sale_during_foreclosure: false,
            willful_violation: false,
        }
    }

    #[test]
    fn ca_compliant_no_liability() {
        let r = check(&ca_baseline_compliant());
        assert!(r.deposit_transfer_compliant);
        assert!(r.tenant_notice_compliant);
        assert!(!r.joint_and_several_liability_attaches);
        assert_eq!(r.jurisdiction_window_days, 30);
    }

    #[test]
    fn ca_personal_delivery_valid() {
        let mut i = ca_baseline_compliant();
        i.notice_method = NoticeMethod::PersonalDelivery;
        let r = check(&i);
        assert!(r.notice_method_valid);
        assert!(!r.joint_and_several_liability_attaches);
    }

    #[test]
    fn ca_regular_mail_invalid() {
        let mut i = ca_baseline_compliant();
        i.notice_method = NoticeMethod::RegularMail;
        let r = check(&i);
        assert!(!r.notice_method_valid);
        assert!(r.joint_and_several_liability_attaches);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 1950.5(h)")
                && f.contains("PERSONAL DELIVERY or FIRST-CLASS MAIL")));
    }

    #[test]
    fn ca_missing_deposit_amount_invalid() {
        let mut i = ca_baseline_compliant();
        i.notice_includes_deposit_amount = false;
        let r = check(&i);
        assert!(!r.tenant_notice_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("AMOUNT of deposit transferred")));
    }

    #[test]
    fn ca_missing_deduction_claims_invalid() {
        let mut i = ca_baseline_compliant();
        i.notice_includes_deduction_claims = false;
        let r = check(&i);
        assert!(!r.tenant_notice_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("ANY CLAIMS made against security")));
    }

    #[test]
    fn ny_5_day_window() {
        let mut i = ca_baseline_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.notice_method = NoticeMethod::RegisteredOrCertifiedMail;
        i.days_since_sale = 4;
        let r = check(&i);
        assert_eq!(r.jurisdiction_window_days, 5);
        assert!(r.deposit_transfer_compliant);
        assert!(!r.joint_and_several_liability_attaches);
    }

    #[test]
    fn ny_window_exceeded() {
        let mut i = ca_baseline_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.notice_method = NoticeMethod::RegisteredOrCertifiedMail;
        i.days_since_sale = 10;
        let r = check(&i);
        assert!(!r.deposit_transfer_compliant);
        assert!(r.joint_and_several_liability_attaches);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("NY GOL § 7-105")
                && f.contains("5-DAY")
                && f.contains("EXCEEDED (10 days)")));
    }

    #[test]
    fn ny_first_class_mail_invalid_requires_certified() {
        let mut i = ca_baseline_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.notice_method = NoticeMethod::FirstClassMail;
        let r = check(&i);
        assert!(!r.notice_method_valid);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("NY GOL § 7-105") && f.contains("REGISTERED OR CERTIFIED MAIL")));
    }

    #[test]
    fn ny_six_plus_unit_accrued_interest_required() {
        let mut i = ca_baseline_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.notice_method = NoticeMethod::RegisteredOrCertifiedMail;
        i.ny_six_plus_unit_building = true;
        i.accrued_interest_transferred = false;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("NY GOL § 7-103(2)")
                && f.contains("6+ family dwelling units")
                && f.contains("ACCRUED INTEREST")));
    }

    #[test]
    fn ma_45_day_window() {
        let mut i = ca_baseline_compliant();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.days_since_sale = 40;
        let r = check(&i);
        assert_eq!(r.jurisdiction_window_days, 45);
        assert!(r.deposit_transfer_compliant);
    }

    #[test]
    fn ma_window_exceeded() {
        let mut i = ca_baseline_compliant();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.days_since_sale = 50;
        let r = check(&i);
        assert!(!r.deposit_transfer_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 15B(7)")
            && f.contains("45-DAY")
            && f.contains("EXCEEDED (50 days)")));
    }

    #[test]
    fn ma_willful_violation_treble_damages() {
        let mut i = ca_baseline_compliant();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.willful_violation = true;
        let r = check(&i);
        assert!(r.treble_damages_available);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 15B(7)")
            && f.contains("WILLFUL")
            && f.contains("TREBLE DAMAGES")));
    }

    #[test]
    fn ma_non_willful_no_treble() {
        let mut i = ca_baseline_compliant();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.willful_violation = false;
        let r = check(&i);
        assert!(!r.treble_damages_available);
    }

    #[test]
    fn no_notice_invalid_all_jurisdictions() {
        for j in [
            Jurisdiction::California,
            Jurisdiction::NewYork,
            Jurisdiction::Massachusetts,
            Jurisdiction::Default,
        ] {
            let mut i = ca_baseline_compliant();
            i.jurisdiction = j;
            i.notice_method = NoticeMethod::NoNotice;
            i.tenant_received_new_owner_notice = false;
            let r = check(&i);
            assert!(!r.tenant_notice_compliant, "j={:?}", j);
            assert!(r.joint_and_several_liability_attaches, "j={:?}", j);
        }
    }

    #[test]
    fn deposit_not_transferred_violation() {
        let mut i = ca_baseline_compliant();
        i.deposit_transferred_to_buyer = false;
        let r = check(&i);
        assert!(!r.deposit_transfer_compliant);
        assert!(r.joint_and_several_liability_attaches);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("NOT TRANSFERRED to successor")));
    }

    #[test]
    fn ptfa_stacks_on_state_obligations() {
        let mut i = ca_baseline_compliant();
        i.sale_during_foreclosure = true;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f| f.contains("12 USC § 5220")
            && f.contains("PTFA 90-day")
            && f.contains("STACKS")));
    }

    #[test]
    fn jurisdiction_window_truth_table_four_cells() {
        for (j, expected_days) in [
            (Jurisdiction::California, 30),
            (Jurisdiction::NewYork, 5),
            (Jurisdiction::Massachusetts, 45),
            (Jurisdiction::Default, 60),
        ] {
            let mut i = ca_baseline_compliant();
            i.jurisdiction = j;
            let r = check(&i);
            assert_eq!(r.jurisdiction_window_days, expected_days, "j={:?}", j);
        }
    }

    #[test]
    fn ny_uniquely_5_day_invariant() {
        let mut ny = ca_baseline_compliant();
        ny.jurisdiction = Jurisdiction::NewYork;
        let r_ny = check(&ny);
        assert_eq!(r_ny.jurisdiction_window_days, 5);

        for j in [
            Jurisdiction::California,
            Jurisdiction::Massachusetts,
            Jurisdiction::Default,
        ] {
            let mut i = ca_baseline_compliant();
            i.jurisdiction = j;
            let r = check(&i);
            assert!(r.jurisdiction_window_days > 5);
        }
    }

    #[test]
    fn ma_uniquely_treble_damages_invariant() {
        let mut ma = ca_baseline_compliant();
        ma.jurisdiction = Jurisdiction::Massachusetts;
        ma.willful_violation = true;
        let r_ma = check(&ma);
        assert!(r_ma.treble_damages_available);

        for j in [
            Jurisdiction::California,
            Jurisdiction::NewYork,
            Jurisdiction::Default,
        ] {
            let mut i = ca_baseline_compliant();
            i.jurisdiction = j;
            i.willful_violation = true;
            let r = check(&i);
            assert!(!r.treble_damages_available, "j={:?}", j);
        }
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&ca_baseline_compliant());
        assert!(r.citation.contains("Cal. Civ. Code § 1950.5(h)"));
        assert!(r.citation.contains("NY GOL § 7-103(2)"));
        assert!(r.citation.contains("NY GOL § 7-105"));
        assert!(r.citation.contains("Mass. Gen. Laws c. 186 § 15B(7)"));
        assert!(r.citation.contains("12 USC § 5220"));
        assert!(r
            .citation
            .contains("Protecting Tenants at Foreclosure Act of 2009"));
        assert!(r
            .citation
            .contains("Restatement (Second) of Property: Landlord and Tenant § 12.1"));
    }

    #[test]
    fn note_pins_four_jurisdiction_framework() {
        let r = check(&ca_baseline_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Four-jurisdiction framework")
                && n.contains("CALIFORNIA")
                && n.contains("NEW YORK")
                && n.contains("MASSACHUSETTS")
                && n.contains("DEFAULT")));
    }

    #[test]
    fn note_pins_ca_section_1950_5h_five_elements() {
        let r = check(&ca_baseline_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Cal. Civ. Code § 1950.5(h) requirements")
                && n.contains("PERSONAL DELIVERY or FIRST-CLASS MAIL")
                && n.contains("AMOUNT")
                && n.contains("any claims")
                && n.contains("JOINT AND SEVERAL LIABILITY")));
    }

    #[test]
    fn note_pins_ny_gol_7_105() {
        let r = check(&ca_baseline_compliant());
        assert!(r.notes.iter().any(|n| n.contains("NY GOL § 7-105")
            && n.contains("5 DAYS")
            && n.contains("REGISTERED OR CERTIFIED MAIL")
            && n.contains("ACCRUED INTEREST")));
    }

    #[test]
    fn note_pins_ny_gol_7_103_2_six_plus_unit() {
        let r = check(&ca_baseline_compliant());
        assert!(r.notes.iter().any(|n| n.contains("NY GOL § 7-103(2)")
            && n.contains("6+ family dwelling units")
            && n.contains("1% per annum")));
    }

    #[test]
    fn note_pins_ma_section_15b_7() {
        let r = check(&ca_baseline_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Mass. Gen. Laws c. 186 § 15B(7)")
                && n.contains("45 DAYS")
                && n.contains("TREBLE DAMAGES")
                && n.contains("REGARDLESS")));
    }

    #[test]
    fn note_pins_ptfa_federal() {
        let r = check(&ca_baseline_compliant());
        assert!(r.notes.iter().any(|n| n.contains("12 USC § 5220")
            && n.contains("Protecting Tenants at Foreclosure Act")
            && n.contains("90-day")
            && n.contains("STACKS")));
    }

    #[test]
    fn note_pins_restatement_second_property_12_1() {
        let r = check(&ca_baseline_compliant());
        assert!(r.notes.iter().any(|n| n
            .contains("Restatement (Second) of Property: Landlord and Tenant § 12.1")
            && n.contains("successor in interest")));
    }

    #[test]
    fn note_pins_joint_several_liability_rule() {
        let r = check(&ca_baseline_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Joint and several liability rule")
                && n.contains("BOTH seller and buyer")
                && n.contains("equitable contribution")));
    }

    #[test]
    fn note_pins_trader_fact_patterns_five() {
        let r = check(&ca_baseline_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Trader-landlord critical fact patterns")
                && n.contains("$5M")
                && n.contains("$36K deposits")
                && n.contains("TREBLE DAMAGES")
                && n.contains("PTFA")));
    }

    #[test]
    fn note_pins_companion_modules() {
        let r = check(&ca_baseline_compliant());
        assert!(r.notes.iter().any(|n| n
            .contains("Companion to security_deposit_bank_disclosure")
            && n.contains("foreclosure_tenant_rights")
            && n.contains("tenant_estoppel_certificate")));
    }

    #[test]
    fn multiple_failures_stack() {
        let mut i = ca_baseline_compliant();
        i.deposit_transferred_to_buyer = false;
        i.tenant_received_new_owner_notice = false;
        i.notice_method = NoticeMethod::NoNotice;
        i.notice_includes_deposit_amount = false;
        i.notice_includes_deduction_claims = false;
        let r = check(&i);
        assert!(r.failure_reasons.len() >= 5);
    }
}
