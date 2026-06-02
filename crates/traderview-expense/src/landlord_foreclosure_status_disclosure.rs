//! Landlord foreclosure status disclosure framework
//! — when a trader-landlord's rental property is in
//! mortgage default or active foreclosure, what
//! disclosure obligations attach to (a) prospective
//! tenants signing a new lease and (b) existing
//! tenants in possession? Distinct from
//! `foreclosure_tenant_rights` (post-foreclosure
//! tenant occupation rights under Federal PTFA).
//!
//! Trader-landlord critical because rental property
//! held with mortgage financing routinely cycles
//! through default events; landlord must time
//! disclosures to (1) preserve enforceability of new
//! leases against the lender's successor-in-interest;
//! (2) avoid lease-void elections; (3) prevent
//! statutory damages and treble-damage claims; and
//! (4) align with Federal PTFA 90-day successor-
//! notice obligations.
//!
//! Companion to foreclosure_tenant_rights (post-
//! foreclosure tenant occupation), landlord_property_
//! sale_notice (sale-transfer security deposit
//! disclosure — iter 437), security_deposit_bank_
//! disclosure, landlord_identification_disclosure,
//! tenant_estoppel_certificate (iter 435).
//!
//! **Four-jurisdiction framework**:
//!
//! CALIFORNIA — Cal. Civ. Code § 2924.8 requires
//! every owner of foreclosed property to POST notice
//! of foreclosure sale on the property within 5
//! BUSINESS DAYS of the lender mailing the notice of
//! trustee's sale; notice must be posted in
//! conspicuous place AND mailed to known tenants.
//! Cal. Civ. Code § 2924.85 (operative 2013-2017,
//! REPEALED January 1, 2018) had required pre-lease
//! disclosure of notice of default to PROSPECTIVE
//! tenants — historical reference for legacy leases.
//!
//! NEW YORK — RPAPL § 1305 requires the successor in
//! interest (NOT seller-landlord) to provide notice
//! within 10 BUSINESS DAYS after judgment of
//! foreclosure that BONA FIDE TENANTS have right to
//! remain for remainder of lease OR 90 DAYS,
//! whichever is LONGER. RPAPL § 1306 requires lender
//! to file foreclosure action information with NY
//! DFS within 3 business days of service.
//!
//! FEDERAL — Protecting Tenants at Foreclosure Act
//! of 2009 (PTFA), 12 USC § 5220, requires successor
//! in interest (foreclosure-sale purchaser) to give
//! BONA FIDE TENANTS at least 90 DAYS' NOTICE before
//! termination; tenant entitled to remain for
//! remainder of lease term unless successor
//! purchaser will occupy as primary residence (in
//! which case 90 days). Made permanent by Economic
//! Growth Regulatory Relief and Consumer Protection
//! Act of 2018 (Pub. L. 115-174 § 304) effective
//! June 23, 2018.
//!
//! DEFAULT / common-law tort disclosure — Restatement
//! (Second) of Torts § 551 imposes duty to disclose
//! material facts known to one party but not the
//! other that the other party would obviously want
//! to know; pending foreclosure on rented property
//! IS material to prospective tenant's lease decision;
//! failure exposes landlord to FRAUDULENT
//! MISREPRESENTATION claim.
//!
//! **CA Civ. Code § 2924.8 notice content requirements**:
//! 1. Notice of foreclosure sale date and time;
//! 2. Notice that property is in foreclosure;
//! 3. Federal PTFA 90-day successor-tenant rights;
//! 4. Tenant's right to remain for remainder of lease
//!    or 90 days (whichever is longer).
//!
//! **CA Civ. Code § 2924.8(d) penalty for non-compliance**
//! — knowing or intentional violation = punitive
//! damages, plus injunction, plus attorney's fees.
//!
//! **NY RPAPL § 1305 successor notice content
//! requirements**:
//! 1. Identity of successor in interest;
//! 2. Tenant's right to remain for remainder of lease
//!    or 90 days (whichever is longer);
//! 3. Successor's contact information for rent payments;
//! 4. Notice that lease may be terminated for cause.
//!
//! **Federal PTFA bona fide tenant requirements
//! (12 USC § 5220(b))**:
//! 1. Tenant is NOT the mortgagor or child/spouse/
//!    parent of mortgagor;
//! 2. Lease was result of ARM'S-LENGTH transaction;
//! 3. Lease requires receipt of rent NOT
//!    SUBSTANTIALLY LESS than fair market rent.
//!
//! **Trader-landlord critical fact patterns**:
//! 1. CA trader receives notice of trustee's sale on
//!    rental property; fails to post Cal. Civ. Code
//!    § 2924.8 notice within 5 business days; tenant
//!    sues for punitive damages under § 2924.8(d).
//! 2. NY trader-owned rental property goes to
//!    judgment of foreclosure; successor purchaser
//!    fails to send RPAPL § 1305 notice within 10
//!    business days; tenant retains 90-day-plus-lease
//!    rights and may sue successor for damages.
//! 3. Federal PTFA — bona fide tenant occupies CA
//!    SFR; lender forecloses and tries to terminate
//!    with 30-day notice; tenant invokes 12 USC
//!    § 5220 90-DAY notice + lease-remainder rights.
//! 4. Trader signs new lease 3 months after receiving
//!    notice of default but BEFORE foreclosure sale;
//!    common-law fraudulent-misrepresentation claim
//!    if pending foreclosure NOT disclosed (Restatement
//!    (Second) of Torts § 551).
//! 5. PTFA NON-BONA-FIDE-TENANT exception — landlord's
//!    son rents apartment at $500/month (well below
//!    $2,000 FMV); fails 12 USC § 5220(b) bona fide
//!    test; successor purchaser may terminate without
//!    90-day notice.
//!
//! Citations: Cal. Civ. Code § 2924.8; Cal. Civ. Code
//! § 2924.85 (REPEALED January 1, 2018 — historical);
//! NY RPAPL § 1305; NY RPAPL § 1306; 12 USC § 5220
//! (Protecting Tenants at Foreclosure Act of 2009);
//! Pub. L. 115-174 § 304 (Economic Growth Regulatory
//! Relief and Consumer Protection Act of 2018 —
//! permanent reauthorization); Restatement (Second)
//! of Torts § 551 (common-law disclosure duty); 12
//! USC § 5220(b) (bona fide tenant definition).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ForeclosureStage {
    /// Notice of default received but no notice of
    /// trustee's sale or foreclosure judgment yet.
    NoticeOfDefault,
    /// Notice of trustee's sale mailed (CA) or
    /// foreclosure summons served (NY).
    NoticeOfTrusteeSaleOrSummons,
    /// Judgment of foreclosure entered.
    JudgmentOfForeclosure,
    /// Foreclosure sale completed; successor in
    /// interest holds title.
    SaleCompleted,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LandlordForeclosureStatusDisclosureInput {
    pub jurisdiction: Jurisdiction,
    pub stage: ForeclosureStage,
    /// Days since the notice trigger event occurred
    /// (notice of trustee's sale OR judgment of
    /// foreclosure).
    pub business_days_since_trigger: u32,
    /// Whether CA Civ. Code § 2924.8 notice was posted
    /// AND mailed within 5 business days.
    pub ca_notice_posted_and_mailed: bool,
    /// Whether NY RPAPL § 1305 notice was sent by
    /// successor within 10 business days of judgment.
    pub ny_rpapl_1305_notice_sent: bool,
    /// Whether prospective tenant signing new lease
    /// after default notice received.
    pub new_lease_during_default_period: bool,
    /// Whether landlord disclosed pending foreclosure
    /// to prospective tenant before lease execution.
    pub prospective_tenant_disclosed_to: bool,
    /// Whether Federal PTFA bona fide tenant requirements
    /// (arm's-length + not family of mortgagor + rent not
    /// substantially less than FMV) satisfied.
    pub ptfa_bona_fide_tenant: bool,
    /// Whether successor in interest gave PTFA 90-day
    /// notice before termination.
    pub successor_gave_ptfa_90_day_notice: bool,
    /// Whether violation was knowing or intentional
    /// (CA § 2924.8(d) punitive damages trigger).
    pub knowing_or_intentional_violation: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LandlordForeclosureStatusDisclosureResult {
    pub ca_section_2924_8_notice_compliant: bool,
    pub ny_rpapl_1305_notice_compliant: bool,
    pub federal_ptfa_compliant: bool,
    pub prospective_tenant_disclosure_compliant: bool,
    pub punitive_damages_available: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(
    input: &LandlordForeclosureStatusDisclosureInput,
) -> LandlordForeclosureStatusDisclosureResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let ca_section_2924_8_notice_compliant = match (
        input.jurisdiction,
        input.stage,
    ) {
        (Jurisdiction::California, ForeclosureStage::NoticeOfTrusteeSaleOrSummons)
        | (
            Jurisdiction::California,
            ForeclosureStage::JudgmentOfForeclosure | ForeclosureStage::SaleCompleted,
        ) => input.ca_notice_posted_and_mailed && input.business_days_since_trigger <= 5,
        _ => true,
    };

    let ny_rpapl_1305_notice_compliant = match (input.jurisdiction, input.stage) {
        (Jurisdiction::NewYork, ForeclosureStage::JudgmentOfForeclosure)
        | (Jurisdiction::NewYork, ForeclosureStage::SaleCompleted) => {
            input.ny_rpapl_1305_notice_sent && input.business_days_since_trigger <= 10
        }
        _ => true,
    };

    let federal_ptfa_compliant = match input.stage {
        ForeclosureStage::SaleCompleted => {
            !input.ptfa_bona_fide_tenant || input.successor_gave_ptfa_90_day_notice
        }
        _ => true,
    };

    let prospective_tenant_disclosure_compliant = !input.new_lease_during_default_period
        || input.prospective_tenant_disclosed_to;

    let punitive_damages_available = matches!(input.jurisdiction, Jurisdiction::California)
        && !ca_section_2924_8_notice_compliant
        && input.knowing_or_intentional_violation;

    if !ca_section_2924_8_notice_compliant {
        failure_reasons.push(format!(
            "Cal. Civ. Code § 2924.8 — landlord failed to POST notice of foreclosure sale on property AND MAIL to known tenants within 5 BUSINESS DAYS of lender mailing notice of trustee's sale ({} days since trigger); notice must include sale date/time, foreclosure status, Federal PTFA 90-day rights, and tenant's right to remain for remainder of lease or 90 days (whichever longer)",
            input.business_days_since_trigger
        ));
    }

    if punitive_damages_available {
        failure_reasons.push(
            "Cal. Civ. Code § 2924.8(d) — KNOWING OR INTENTIONAL violation triggers PUNITIVE DAMAGES plus injunction plus attorney's fees; statute creates private right of action".to_string(),
        );
    }

    if !ny_rpapl_1305_notice_compliant {
        failure_reasons.push(format!(
            "NY RPAPL § 1305 — successor in interest failed to send notice to bona fide tenants within 10 BUSINESS DAYS after judgment of foreclosure ({} days since trigger); notice must include successor identity, tenant's right to remain for remainder of lease or 90 days (whichever longer), and successor's contact information for rent payments",
            input.business_days_since_trigger
        ));
    }

    if !federal_ptfa_compliant {
        failure_reasons.push(
            "12 USC § 5220 (Protecting Tenants at Foreclosure Act of 2009 — made permanent by Pub. L. 115-174 § 304) — successor in interest must give BONA FIDE TENANTS at least 90 DAYS' NOTICE before termination; tenant entitled to remain for remainder of lease unless successor will occupy as primary residence (90 days)".to_string(),
        );
    }

    if !prospective_tenant_disclosure_compliant {
        failure_reasons.push(
            "Restatement (Second) of Torts § 551 — common-law DUTY TO DISCLOSE material fact known to one party but not the other that the other would obviously want to know; pending foreclosure on rented property IS material to prospective tenant's lease decision; failure exposes landlord to FRAUDULENT MISREPRESENTATION claim".to_string(),
        );
    }

    if input.ptfa_bona_fide_tenant {
        failure_reasons.push(
            "12 USC § 5220(b) BONA FIDE TENANT — tenant satisfies (1) NOT mortgagor or child/spouse/parent of mortgagor; (2) lease was ARM'S-LENGTH transaction; (3) lease requires rent NOT SUBSTANTIALLY LESS than fair market rent; PTFA protections apply".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "Four-jurisdiction framework: CALIFORNIA (Cal. Civ. Code § 2924.8 — 5-business-day post-and-mail notice on trustee's sale; § 2924.85 REPEALED January 1, 2018 for prospective-tenant pre-lease disclosure); NEW YORK (NY RPAPL § 1305 — 10-business-day successor notice after foreclosure judgment); FEDERAL (12 USC § 5220 PTFA — 90-day successor notice to bona fide tenants); DEFAULT (Restatement (Second) of Torts § 551 — common-law disclosure duty)".to_string(),
        "Cal. Civ. Code § 2924.8 — owner of foreclosed property must POST notice on property AND MAIL to known tenants within 5 BUSINESS DAYS of lender mailing notice of trustee's sale; notice content: (1) sale date/time; (2) foreclosure status; (3) Federal PTFA 90-day rights; (4) tenant's right to remain for remainder of lease or 90 days (whichever longer); § 2924.8(d) penalty: KNOWING OR INTENTIONAL violation = punitive damages + injunction + attorney's fees".to_string(),
        "NY RPAPL § 1305 — successor in interest (NOT seller-landlord) must provide notice within 10 BUSINESS DAYS after judgment of foreclosure; notice content: (1) successor identity; (2) tenant's right to remain for remainder of lease or 90 days (whichever LONGER); (3) successor contact for rent; (4) notice that lease may be terminated for cause; NY RPAPL § 1306 — lender must file foreclosure action information with NY DFS within 3 business days of service".to_string(),
        "12 USC § 5220 (Protecting Tenants at Foreclosure Act of 2009 — PTFA) — successor in interest (foreclosure-sale purchaser) must give BONA FIDE TENANTS at least 90 DAYS' NOTICE before termination; tenant entitled to remain for REMAINDER OF LEASE TERM unless successor will occupy as primary residence (90 days); made PERMANENT by Pub. L. 115-174 § 304 (Economic Growth Regulatory Relief and Consumer Protection Act of 2018) effective June 23, 2018".to_string(),
        "12 USC § 5220(b) BONA FIDE TENANT three-element test: (1) NOT mortgagor or child/spouse/parent of mortgagor; (2) lease was ARM'S-LENGTH transaction; (3) lease requires rent NOT SUBSTANTIALLY LESS than fair market rent; failure of any element disqualifies tenant from PTFA 90-day successor-notice protection".to_string(),
        "Restatement (Second) of Torts § 551 common-law duty to disclose — duty arises when (1) party knows material fact; (2) other party does not know and would obviously want to know; (3) parties stand in fiduciary or confidential relation OR transaction involves matters as to which one party would reasonably expect disclosure; pending foreclosure on rented property IS material to prospective tenant's lease decision".to_string(),
        "Cal. Civ. Code § 2924.85 (REPEALED January 1, 2018 by its own provisions) — historically required landlord with single-family or 1-4 unit multifamily dwelling who received notice of default to disclose to prospective tenants before lease execution; violation voided lease at tenant's election; entitled tenant to recovery of one month's rent or twice actual damages (whichever greater) + all prepaid rent; cite for legacy leases predating January 1, 2018".to_string(),
        "Trader-landlord critical fact patterns: (1) CA trader fails 5-business-day § 2924.8 post-and-mail — tenant sues for punitive damages under § 2924.8(d); (2) NY trader-owned property at judgment of foreclosure — successor's failure to send RPAPL § 1305 within 10 business days preserves tenant's 90-day-plus-lease rights; (3) federal PTFA 90-day successor notice trumps 30-day state termination; (4) trader signs new lease 3 months after default notice without disclosure — Restatement § 551 fraudulent-misrepresentation claim; (5) landlord's son at $500 vs $2,000 FMV — fails 12 USC § 5220(b) bona fide test".to_string(),
        "Companion to foreclosure_tenant_rights (post-foreclosure tenant occupation under PTFA) + landlord_property_sale_notice (sale-transfer security deposit disclosure) + security_deposit_bank_disclosure + landlord_identification_disclosure + tenant_estoppel_certificate".to_string(),
    ];

    LandlordForeclosureStatusDisclosureResult {
        ca_section_2924_8_notice_compliant,
        ny_rpapl_1305_notice_compliant,
        federal_ptfa_compliant,
        prospective_tenant_disclosure_compliant,
        punitive_damages_available,
        failure_reasons,
        citation: "Cal. Civ. Code § 2924.8; Cal. Civ. Code § 2924.85 (REPEALED January 1, 2018 — historical); NY RPAPL § 1305; NY RPAPL § 1306; 12 USC § 5220 (Protecting Tenants at Foreclosure Act of 2009); Pub. L. 115-174 § 304 (Economic Growth Regulatory Relief and Consumer Protection Act of 2018); 12 USC § 5220(b) (bona fide tenant definition); Restatement (Second) of Torts § 551",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_compliant() -> LandlordForeclosureStatusDisclosureInput {
        LandlordForeclosureStatusDisclosureInput {
            jurisdiction: Jurisdiction::California,
            stage: ForeclosureStage::NoticeOfTrusteeSaleOrSummons,
            business_days_since_trigger: 3,
            ca_notice_posted_and_mailed: true,
            ny_rpapl_1305_notice_sent: false,
            new_lease_during_default_period: false,
            prospective_tenant_disclosed_to: false,
            ptfa_bona_fide_tenant: true,
            successor_gave_ptfa_90_day_notice: true,
            knowing_or_intentional_violation: false,
        }
    }

    #[test]
    fn ca_within_5_business_days_compliant() {
        let r = check(&ca_compliant());
        assert!(r.ca_section_2924_8_notice_compliant);
    }

    #[test]
    fn ca_past_5_business_days_violation() {
        let mut i = ca_compliant();
        i.business_days_since_trigger = 10;
        let r = check(&i);
        assert!(!r.ca_section_2924_8_notice_compliant);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 2924.8")
            && f.contains("5 BUSINESS DAYS")
            && f.contains("10 days since trigger")));
    }

    #[test]
    fn ca_5_day_boundary_compliant() {
        let mut i = ca_compliant();
        i.business_days_since_trigger = 5;
        let r = check(&i);
        assert!(r.ca_section_2924_8_notice_compliant);
    }

    #[test]
    fn ca_no_posting_violation() {
        let mut i = ca_compliant();
        i.ca_notice_posted_and_mailed = false;
        let r = check(&i);
        assert!(!r.ca_section_2924_8_notice_compliant);
    }

    #[test]
    fn ca_knowing_violation_punitive_damages() {
        let mut i = ca_compliant();
        i.ca_notice_posted_and_mailed = false;
        i.knowing_or_intentional_violation = true;
        let r = check(&i);
        assert!(r.punitive_damages_available);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 2924.8(d)")
            && f.contains("KNOWING OR INTENTIONAL")
            && f.contains("PUNITIVE DAMAGES")));
    }

    #[test]
    fn ca_non_knowing_no_punitive() {
        let mut i = ca_compliant();
        i.ca_notice_posted_and_mailed = false;
        i.knowing_or_intentional_violation = false;
        let r = check(&i);
        assert!(!r.punitive_damages_available);
    }

    #[test]
    fn ny_within_10_business_days_compliant() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.stage = ForeclosureStage::JudgmentOfForeclosure;
        i.business_days_since_trigger = 8;
        i.ny_rpapl_1305_notice_sent = true;
        let r = check(&i);
        assert!(r.ny_rpapl_1305_notice_compliant);
    }

    #[test]
    fn ny_past_10_business_days_violation() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.stage = ForeclosureStage::JudgmentOfForeclosure;
        i.business_days_since_trigger = 15;
        i.ny_rpapl_1305_notice_sent = true;
        let r = check(&i);
        assert!(!r.ny_rpapl_1305_notice_compliant);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("RPAPL § 1305")
            && f.contains("10 BUSINESS DAYS")
            && f.contains("15 days since trigger")));
    }

    #[test]
    fn ny_no_notice_violation() {
        let mut i = ca_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.stage = ForeclosureStage::JudgmentOfForeclosure;
        i.business_days_since_trigger = 5;
        i.ny_rpapl_1305_notice_sent = false;
        let r = check(&i);
        assert!(!r.ny_rpapl_1305_notice_compliant);
    }

    #[test]
    fn federal_ptfa_90_day_notice_compliant() {
        let mut i = ca_compliant();
        i.stage = ForeclosureStage::SaleCompleted;
        i.ptfa_bona_fide_tenant = true;
        i.successor_gave_ptfa_90_day_notice = true;
        let r = check(&i);
        assert!(r.federal_ptfa_compliant);
    }

    #[test]
    fn federal_ptfa_failure_to_give_90_day_notice() {
        let mut i = ca_compliant();
        i.stage = ForeclosureStage::SaleCompleted;
        i.ptfa_bona_fide_tenant = true;
        i.successor_gave_ptfa_90_day_notice = false;
        let r = check(&i);
        assert!(!r.federal_ptfa_compliant);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("12 USC § 5220")
            && f.contains("90 DAYS' NOTICE")));
    }

    #[test]
    fn federal_ptfa_non_bona_fide_no_protection() {
        let mut i = ca_compliant();
        i.stage = ForeclosureStage::SaleCompleted;
        i.ptfa_bona_fide_tenant = false;
        i.successor_gave_ptfa_90_day_notice = false;
        let r = check(&i);
        assert!(r.federal_ptfa_compliant);
    }

    #[test]
    fn ptfa_bona_fide_tenant_protections_apply_note() {
        let mut i = ca_compliant();
        i.ptfa_bona_fide_tenant = true;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("12 USC § 5220(b) BONA FIDE TENANT")
            && f.contains("ARM'S-LENGTH")
            && f.contains("NOT SUBSTANTIALLY LESS")));
    }

    #[test]
    fn prospective_tenant_disclosure_compliant() {
        let mut i = ca_compliant();
        i.new_lease_during_default_period = true;
        i.prospective_tenant_disclosed_to = true;
        let r = check(&i);
        assert!(r.prospective_tenant_disclosure_compliant);
    }

    #[test]
    fn prospective_tenant_undisclosed_fraudulent_misrep() {
        let mut i = ca_compliant();
        i.new_lease_during_default_period = true;
        i.prospective_tenant_disclosed_to = false;
        let r = check(&i);
        assert!(!r.prospective_tenant_disclosure_compliant);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("Restatement (Second) of Torts § 551")
            && f.contains("FRAUDULENT MISREPRESENTATION")));
    }

    #[test]
    fn no_new_lease_no_disclosure_obligation() {
        let mut i = ca_compliant();
        i.new_lease_during_default_period = false;
        i.prospective_tenant_disclosed_to = false;
        let r = check(&i);
        assert!(r.prospective_tenant_disclosure_compliant);
    }

    #[test]
    fn jurisdiction_truth_table_three_cells_ca_5_day() {
        let mut ca = ca_compliant();
        ca.business_days_since_trigger = 10;
        ca.ca_notice_posted_and_mailed = false;
        let r_ca = check(&ca);
        assert!(!r_ca.ca_section_2924_8_notice_compliant);

        let mut ny = ca_compliant();
        ny.jurisdiction = Jurisdiction::NewYork;
        ny.stage = ForeclosureStage::JudgmentOfForeclosure;
        ny.business_days_since_trigger = 10;
        ny.ny_rpapl_1305_notice_sent = false;
        let r_ny = check(&ny);
        assert!(!r_ny.ny_rpapl_1305_notice_compliant);

        let mut default_j = ca_compliant();
        default_j.jurisdiction = Jurisdiction::Default;
        let r_d = check(&default_j);
        assert!(r_d.ca_section_2924_8_notice_compliant);
        assert!(r_d.ny_rpapl_1305_notice_compliant);
    }

    #[test]
    fn ca_uniquely_engages_punitive_damages_invariant() {
        let mut ca_violation = ca_compliant();
        ca_violation.ca_notice_posted_and_mailed = false;
        ca_violation.knowing_or_intentional_violation = true;
        let r_ca = check(&ca_violation);
        assert!(r_ca.punitive_damages_available);

        for j in [Jurisdiction::NewYork, Jurisdiction::Default] {
            let mut i = ca_compliant();
            i.jurisdiction = j;
            i.knowing_or_intentional_violation = true;
            let r = check(&i);
            assert!(!r.punitive_damages_available, "j={:?}", j);
        }
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&ca_compliant());
        assert!(r.citation.contains("Cal. Civ. Code § 2924.8"));
        assert!(r.citation.contains("Cal. Civ. Code § 2924.85 (REPEALED January 1, 2018"));
        assert!(r.citation.contains("NY RPAPL § 1305"));
        assert!(r.citation.contains("NY RPAPL § 1306"));
        assert!(r.citation.contains("12 USC § 5220"));
        assert!(r.citation.contains("Protecting Tenants at Foreclosure Act of 2009"));
        assert!(r.citation.contains("Pub. L. 115-174 § 304"));
        assert!(r.citation.contains("12 USC § 5220(b)"));
        assert!(r.citation.contains("Restatement (Second) of Torts § 551"));
    }

    #[test]
    fn note_pins_four_jurisdiction_framework() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("Four-jurisdiction framework")
            && n.contains("CALIFORNIA")
            && n.contains("§ 2924.85 REPEALED")
            && n.contains("NEW YORK")
            && n.contains("FEDERAL")
            && n.contains("DEFAULT")));
    }

    #[test]
    fn note_pins_ca_2924_8_four_elements() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("Cal. Civ. Code § 2924.8")
            && n.contains("5 BUSINESS DAYS")
            && n.contains("§ 2924.8(d)")
            && n.contains("KNOWING OR INTENTIONAL")));
    }

    #[test]
    fn note_pins_ny_rpapl_1305_four_elements() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("NY RPAPL § 1305")
            && n.contains("10 BUSINESS DAYS")
            && n.contains("LONGER")
            && n.contains("RPAPL § 1306")));
    }

    #[test]
    fn note_pins_federal_ptfa() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("12 USC § 5220")
            && n.contains("Protecting Tenants at Foreclosure Act")
            && n.contains("90 DAYS' NOTICE")
            && n.contains("Pub. L. 115-174 § 304")
            && n.contains("June 23, 2018")));
    }

    #[test]
    fn note_pins_bona_fide_tenant_three_elements() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("BONA FIDE TENANT three-element test")
            && n.contains("ARM'S-LENGTH")
            && n.contains("NOT SUBSTANTIALLY LESS than fair market rent")));
    }

    #[test]
    fn note_pins_restatement_second_torts_551() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("Restatement (Second) of Torts § 551")
            && n.contains("duty to disclose")));
    }

    #[test]
    fn note_pins_ca_2924_85_repeal_historical() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("Cal. Civ. Code § 2924.85 (REPEALED January 1, 2018")
            && n.contains("voided lease at tenant's election")
            && n.contains("legacy leases")));
    }

    #[test]
    fn note_pins_trader_fact_patterns_five() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("Trader-landlord critical fact patterns")
            && n.contains("§ 2924.8(d)")
            && n.contains("RPAPL § 1305")
            && n.contains("12 USC § 5220(b)")));
    }

    #[test]
    fn note_pins_companion_modules() {
        let r = check(&ca_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("Companion to foreclosure_tenant_rights")
            && n.contains("landlord_property_sale_notice")
            && n.contains("tenant_estoppel_certificate")));
    }

    #[test]
    fn multiple_failures_stack() {
        let mut i = ca_compliant();
        i.ca_notice_posted_and_mailed = false;
        i.business_days_since_trigger = 15;
        i.new_lease_during_default_period = true;
        i.prospective_tenant_disclosed_to = false;
        i.knowing_or_intentional_violation = true;
        let r = check(&i);
        assert!(r.failure_reasons.len() >= 4);
    }
}
