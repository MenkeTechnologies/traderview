//! IRC § 6325 — Release of lien or discharge of property.
//! Completes the foundational federal tax lien constellation
//! (§ 6321 attachment + § 6322 period of lien + § 6323
//! priority + § 6325 release/discharge/subordination).
//! Trader-relevant when trader-landlord seeks to (a)
//! extinguish a federal tax lien upon full payment, (b)
//! discharge individual property from lien for sale or
//! refinancing, or (c) subordinate IRS lien to allow junior
//! financing (mortgage refinance to extract equity for IRS
//! payment).
//!
//! **§ 6325(a) RELEASE of lien** — Secretary SHALL issue
//! certificate of release within **30 days** after finding
//! that:
//! - § 6325(a)(1) — liability + interest has been fully
//!   satisfied OR has become legally unenforceable, OR
//! - § 6325(a)(2) — bond conditioned upon payment is
//!   furnished and accepted.
//!
//! Certificate of release is **CONCLUSIVE** that the lien is
//! extinguished.
//!
//! **§ 6325(b) DISCHARGE of property** — Secretary MAY issue
//! certificate of discharge releasing specific property from
//! the lien (other property still subject to lien):
//! - § 6325(b)(1) — property value ≥ 2× sum of (1) federal
//!   tax lien amount + (2) all other senior liens.
//! - § 6325(b)(2)(A) — partial payment equal to property's
//!   net interest of the United States.
//! - § 6325(b)(2)(B) — IRS determines United States interest
//!   in property has no value.
//! - § 6325(b)(3) — sale proceeds substituted (held as fund
//!   subject to lien with same priority).
//! - § 6325(b)(4) — purchaser deposits with IRS.
//!
//! **§ 6325(c) DISCHARGE of estate tax lien** — special rule
//! for estate / gift tax liens under § 6324(a) + (b).
//!
//! **§ 6325(d) SUBORDINATION of lien** — Secretary MAY issue
//! certificate of subordination allowing junior creditor
//! priority over federal tax lien:
//! - § 6325(d)(1) — payment over to Secretary equal to amount
//!   of the lien or interest subordinated, OR
//! - § 6325(d)(2) — IRS believes ultimate collection will be
//!   facilitated by subordination (typical trader-landlord
//!   mortgage refinance scenario).
//!
//! **§ 6325(e) NON-ATTACHMENT** — Secretary may issue
//! certificate of non-attachment where confusion of names
//! suggests tax lien attaches to property not owned by
//! taxpayer.
//!
//! **§ 6325(f) effect of certificates** — certificate of
//! release is CONCLUSIVE that lien is extinguished;
//! certificate of discharge is CONCLUSIVE that property is
//! discharged; certificate of subordination is CONCLUSIVE
//! that subordinated interest is superior to tax lien.
//!
//! Citations: 26 USC § 6325(a)(1)-(2), (b)(1)-(4), (c), (d)
//! (1)-(2), (e), (f); 26 CFR § 301.6325-1; IRM 5.12.10 (Lien
//! Related Certificates); IRS Pub. 783 (Discharge); IRS Pub.
//! 784 (Subordination).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CertificateType {
    /// § 6325(a) — release extinguishes lien entirely.
    Release,
    /// § 6325(b) — discharge of specific property.
    Discharge,
    /// § 6325(d) — subordination of lien to junior creditor.
    Subordination,
    /// § 6325(e) — non-attachment certificate.
    NonAttachment,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseBasis {
    /// § 6325(a)(1) — full satisfaction of liability +
    /// interest.
    FullSatisfaction,
    /// § 6325(a)(1) — liability legally unenforceable (paired
    /// with § 6502 CSED expiration).
    LegallyUnenforceable,
    /// § 6325(a)(2) — bond accepted.
    BondAccepted,
    /// Not applicable for non-release certificate types.
    NotApplicable,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DischargeBasis {
    /// § 6325(b)(1) — property value ≥ 2× (lien + senior
    /// liens).
    DoubleValueRule,
    /// § 6325(b)(2)(A) — partial payment equal to US net
    /// interest.
    PartialPayment,
    /// § 6325(b)(2)(B) — US interest has no value.
    NoValueDetermination,
    /// § 6325(b)(3) — sale proceeds substituted.
    ProceedsSubstituted,
    /// § 6325(b)(4) — purchaser deposits with IRS.
    PurchaserDeposit,
    /// Not applicable for non-discharge certificate types.
    NotApplicable,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubordinationBasis {
    /// § 6325(d)(1) — payment equal to subordinated amount.
    PaymentForSubordinatedAmount,
    /// § 6325(d)(2) — ultimate collection facilitated
    /// (typical trader refinance scenario).
    UltimateCollectionFacilitated,
    /// Not applicable for non-subordination certificate
    /// types.
    NotApplicable,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6325Input {
    pub certificate_type: CertificateType,
    pub release_basis: ReleaseBasis,
    pub discharge_basis: DischargeBasis,
    pub subordination_basis: SubordinationBasis,
    /// Days elapsed since qualifying event (for § 6325(a)
    /// 30-day release deadline).
    pub days_since_qualifying_event: u32,
    /// For § 6325(b)(1) double-value rule: property value
    /// in cents.
    pub property_value_cents: i64,
    /// For § 6325(b)(1): federal tax lien amount.
    pub federal_tax_lien_amount_cents: i64,
    /// For § 6325(b)(1): sum of all senior liens (other than
    /// federal tax lien).
    pub senior_liens_total_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6325Result {
    pub certificate_issuable: bool,
    pub release_within_30_days: bool,
    pub double_value_rule_satisfied: bool,
    pub certificate_conclusive_under_6325f: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6325Input) -> Section6325Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let property_value = input.property_value_cents.max(0);
    let lien_amount = input.federal_tax_lien_amount_cents.max(0);
    let senior_liens = input.senior_liens_total_cents.max(0);
    let required_value = lien_amount
        .saturating_add(senior_liens)
        .saturating_mul(2);
    let double_value_satisfied = property_value >= required_value;

    let release_30_day = input.days_since_qualifying_event <= 30;

    match input.certificate_type {
        CertificateType::Release => {
            if matches!(input.release_basis, ReleaseBasis::NotApplicable) {
                failure_reasons.push(
                    "26 USC § 6325(a) — release certificate requires § 6325(a)(1) full satisfaction / legally unenforceable OR § 6325(a)(2) bond accepted".to_string(),
                );
            }
            if !release_30_day {
                failure_reasons.push(format!(
                    "26 USC § 6325(a) — Secretary SHALL issue certificate of release within 30 days; {} days elapsed",
                    input.days_since_qualifying_event
                ));
            }
        }
        CertificateType::Discharge => {
            if matches!(input.discharge_basis, DischargeBasis::NotApplicable) {
                failure_reasons.push(
                    "26 USC § 6325(b) — discharge certificate requires one of (b)(1) double-value rule + (b)(2)(A) partial payment + (b)(2)(B) no-value determination + (b)(3) proceeds substituted + (b)(4) purchaser deposit".to_string(),
                );
            }
            if matches!(
                input.discharge_basis,
                DischargeBasis::DoubleValueRule
            ) && !double_value_satisfied
            {
                failure_reasons.push(format!(
                    "26 USC § 6325(b)(1) — property value ({} cents) does not meet 2× threshold (required {} cents = 2 × (lien {} + senior liens {}))",
                    property_value, required_value, lien_amount, senior_liens
                ));
            }
        }
        CertificateType::Subordination => {
            if matches!(
                input.subordination_basis,
                SubordinationBasis::NotApplicable
            ) {
                failure_reasons.push(
                    "26 USC § 6325(d) — subordination certificate requires (d)(1) payment for subordinated amount OR (d)(2) ultimate collection facilitated".to_string(),
                );
            }
        }
        CertificateType::NonAttachment => {}
    }

    let certificate_issuable = failure_reasons.is_empty();

    let notes: Vec<String> = vec![
        "26 USC § 6325(a) — Secretary SHALL issue certificate of release within 30 days upon (1) full satisfaction OR legally unenforceable OR (2) bond accepted"
            .to_string(),
        "26 USC § 6325(b) — discharge of property: (b)(1) double-value rule (property ≥ 2× (lien + senior liens)); (b)(2)(A) partial payment for US net interest; (b)(2)(B) no-value determination; (b)(3) proceeds substituted; (b)(4) purchaser deposit"
            .to_string(),
        "26 USC § 6325(d) — subordination: (d)(1) payment for subordinated amount OR (d)(2) ultimate collection facilitated (typical trader refinance to extract equity for IRS payment)"
            .to_string(),
        "26 USC § 6325(f) — certificates are CONCLUSIVE: release extinguishes lien; discharge releases property; subordination establishes superior priority of subordinated interest"
            .to_string(),
    ];

    Section6325Result {
        certificate_issuable,
        release_within_30_days: release_30_day,
        double_value_rule_satisfied: double_value_satisfied,
        certificate_conclusive_under_6325f: certificate_issuable,
        failure_reasons,
        citation: "26 USC § 6325(a)(1)-(2), (b)(1)-(4), (c), (d)(1)-(2), (e), (f); 26 CFR § 301.6325-1; IRM 5.12.10; IRS Pub. 783 (Discharge); IRS Pub. 784 (Subordination)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release_base() -> Section6325Input {
        Section6325Input {
            certificate_type: CertificateType::Release,
            release_basis: ReleaseBasis::FullSatisfaction,
            discharge_basis: DischargeBasis::NotApplicable,
            subordination_basis: SubordinationBasis::NotApplicable,
            days_since_qualifying_event: 15,
            property_value_cents: 0,
            federal_tax_lien_amount_cents: 0,
            senior_liens_total_cents: 0,
        }
    }

    fn discharge_double_value_base() -> Section6325Input {
        Section6325Input {
            certificate_type: CertificateType::Discharge,
            release_basis: ReleaseBasis::NotApplicable,
            discharge_basis: DischargeBasis::DoubleValueRule,
            subordination_basis: SubordinationBasis::NotApplicable,
            days_since_qualifying_event: 0,
            property_value_cents: 50_000_000,
            federal_tax_lien_amount_cents: 10_000_000,
            senior_liens_total_cents: 5_000_000,
        }
    }

    fn subordination_base() -> Section6325Input {
        Section6325Input {
            certificate_type: CertificateType::Subordination,
            release_basis: ReleaseBasis::NotApplicable,
            discharge_basis: DischargeBasis::NotApplicable,
            subordination_basis: SubordinationBasis::UltimateCollectionFacilitated,
            days_since_qualifying_event: 0,
            property_value_cents: 0,
            federal_tax_lien_amount_cents: 0,
            senior_liens_total_cents: 0,
        }
    }

    #[test]
    fn release_full_satisfaction_within_30_days_issuable() {
        let r = check(&release_base());
        assert!(r.certificate_issuable);
        assert!(r.release_within_30_days);
        assert!(r.certificate_conclusive_under_6325f);
    }

    #[test]
    fn release_at_30_day_boundary_issuable() {
        let mut i = release_base();
        i.days_since_qualifying_event = 30;
        let r = check(&i);
        assert!(r.certificate_issuable);
        assert!(r.release_within_30_days);
    }

    #[test]
    fn release_31_days_violates() {
        let mut i = release_base();
        i.days_since_qualifying_event = 31;
        let r = check(&i);
        assert!(!r.certificate_issuable);
        assert!(!r.release_within_30_days);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6325(a)") && f.contains("30 days") && f.contains("31")));
    }

    #[test]
    fn release_legally_unenforceable_basis_valid() {
        let mut i = release_base();
        i.release_basis = ReleaseBasis::LegallyUnenforceable;
        let r = check(&i);
        assert!(r.certificate_issuable);
    }

    #[test]
    fn release_bond_accepted_basis_valid() {
        let mut i = release_base();
        i.release_basis = ReleaseBasis::BondAccepted;
        let r = check(&i);
        assert!(r.certificate_issuable);
    }

    #[test]
    fn release_no_basis_violates() {
        let mut i = release_base();
        i.release_basis = ReleaseBasis::NotApplicable;
        let r = check(&i);
        assert!(!r.certificate_issuable);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6325(a)") && f.contains("(a)(1)") && f.contains("(a)(2)")));
    }

    #[test]
    fn discharge_double_value_2x_satisfied() {
        let r = check(&discharge_double_value_base());
        assert!(r.certificate_issuable);
        assert!(r.double_value_rule_satisfied);
    }

    #[test]
    fn discharge_double_value_at_boundary_satisfied() {
        let mut i = discharge_double_value_base();
        i.property_value_cents = 30_000_000;
        i.federal_tax_lien_amount_cents = 10_000_000;
        i.senior_liens_total_cents = 5_000_000;
        let r = check(&i);
        assert!(r.certificate_issuable);
        assert!(r.double_value_rule_satisfied);
    }

    #[test]
    fn discharge_double_value_just_under_violates() {
        let mut i = discharge_double_value_base();
        i.property_value_cents = 29_999_999;
        i.federal_tax_lien_amount_cents = 10_000_000;
        i.senior_liens_total_cents = 5_000_000;
        let r = check(&i);
        assert!(!r.certificate_issuable);
        assert!(!r.double_value_rule_satisfied);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6325(b)(1)") && f.contains("2×")));
    }

    #[test]
    fn discharge_partial_payment_basis_valid() {
        let mut i = discharge_double_value_base();
        i.discharge_basis = DischargeBasis::PartialPayment;
        let r = check(&i);
        assert!(r.certificate_issuable);
    }

    #[test]
    fn discharge_no_value_basis_valid() {
        let mut i = discharge_double_value_base();
        i.discharge_basis = DischargeBasis::NoValueDetermination;
        let r = check(&i);
        assert!(r.certificate_issuable);
    }

    #[test]
    fn discharge_proceeds_substituted_basis_valid() {
        let mut i = discharge_double_value_base();
        i.discharge_basis = DischargeBasis::ProceedsSubstituted;
        let r = check(&i);
        assert!(r.certificate_issuable);
    }

    #[test]
    fn discharge_purchaser_deposit_basis_valid() {
        let mut i = discharge_double_value_base();
        i.discharge_basis = DischargeBasis::PurchaserDeposit;
        let r = check(&i);
        assert!(r.certificate_issuable);
    }

    #[test]
    fn discharge_no_basis_violates() {
        let mut i = discharge_double_value_base();
        i.discharge_basis = DischargeBasis::NotApplicable;
        let r = check(&i);
        assert!(!r.certificate_issuable);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6325(b)") && f.contains("discharge certificate")));
    }

    #[test]
    fn subordination_facilitates_collection_basis_valid() {
        let r = check(&subordination_base());
        assert!(r.certificate_issuable);
    }

    #[test]
    fn subordination_payment_basis_valid() {
        let mut i = subordination_base();
        i.subordination_basis = SubordinationBasis::PaymentForSubordinatedAmount;
        let r = check(&i);
        assert!(r.certificate_issuable);
    }

    #[test]
    fn subordination_no_basis_violates() {
        let mut i = subordination_base();
        i.subordination_basis = SubordinationBasis::NotApplicable;
        let r = check(&i);
        assert!(!r.certificate_issuable);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6325(d)") && f.contains("(d)(1)") && f.contains("(d)(2)")));
    }

    #[test]
    fn non_attachment_certificate_issuable() {
        let i = Section6325Input {
            certificate_type: CertificateType::NonAttachment,
            release_basis: ReleaseBasis::NotApplicable,
            discharge_basis: DischargeBasis::NotApplicable,
            subordination_basis: SubordinationBasis::NotApplicable,
            days_since_qualifying_event: 0,
            property_value_cents: 0,
            federal_tax_lien_amount_cents: 0,
            senior_liens_total_cents: 0,
        };
        let r = check(&i);
        assert!(r.certificate_issuable);
    }

    #[test]
    fn certificate_type_truth_table() {
        let r_release = check(&release_base());
        assert!(r_release.certificate_issuable);

        let r_discharge = check(&discharge_double_value_base());
        assert!(r_discharge.certificate_issuable);

        let r_subordination = check(&subordination_base());
        assert!(r_subordination.certificate_issuable);
    }

    #[test]
    fn release_basis_truth_table() {
        for (basis, exp_valid) in [
            (ReleaseBasis::FullSatisfaction, true),
            (ReleaseBasis::LegallyUnenforceable, true),
            (ReleaseBasis::BondAccepted, true),
            (ReleaseBasis::NotApplicable, false),
        ] {
            let mut i = release_base();
            i.release_basis = basis;
            let r = check(&i);
            assert_eq!(r.certificate_issuable, exp_valid);
        }
    }

    #[test]
    fn discharge_basis_truth_table() {
        for (basis, exp_valid) in [
            (DischargeBasis::DoubleValueRule, true),
            (DischargeBasis::PartialPayment, true),
            (DischargeBasis::NoValueDetermination, true),
            (DischargeBasis::ProceedsSubstituted, true),
            (DischargeBasis::PurchaserDeposit, true),
            (DischargeBasis::NotApplicable, false),
        ] {
            let mut i = discharge_double_value_base();
            i.discharge_basis = basis;
            let r = check(&i);
            assert_eq!(r.certificate_issuable, exp_valid);
        }
    }

    #[test]
    fn subordination_basis_truth_table() {
        for (basis, exp_valid) in [
            (SubordinationBasis::PaymentForSubordinatedAmount, true),
            (SubordinationBasis::UltimateCollectionFacilitated, true),
            (SubordinationBasis::NotApplicable, false),
        ] {
            let mut i = subordination_base();
            i.subordination_basis = basis;
            let r = check(&i);
            assert_eq!(r.certificate_issuable, exp_valid);
        }
    }

    #[test]
    fn certificate_conclusive_when_issuable() {
        let r = check(&release_base());
        assert!(r.certificate_conclusive_under_6325f);
    }

    #[test]
    fn certificate_not_conclusive_when_not_issuable() {
        let mut i = release_base();
        i.release_basis = ReleaseBasis::NotApplicable;
        let r = check(&i);
        assert!(!r.certificate_conclusive_under_6325f);
    }

    #[test]
    fn defensive_negative_property_value_clamped() {
        let mut i = discharge_double_value_base();
        i.property_value_cents = -1_000_000;
        let r = check(&i);
        assert!(!r.double_value_rule_satisfied);
    }

    #[test]
    fn defensive_overflow_double_value_saturating() {
        let mut i = discharge_double_value_base();
        i.federal_tax_lien_amount_cents = i64::MAX;
        i.senior_liens_total_cents = i64::MAX;
        let r = check(&i);
        assert!(!r.double_value_rule_satisfied);
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = check(&release_base());
        assert!(r.citation.contains("§ 6325(a)(1)-(2)"));
        assert!(r.citation.contains("(b)(1)-(4)"));
        assert!(r.citation.contains("(c)"));
        assert!(r.citation.contains("(d)(1)-(2)"));
        assert!(r.citation.contains("(e)"));
        assert!(r.citation.contains("(f)"));
        assert!(r.citation.contains("§ 301.6325-1"));
        assert!(r.citation.contains("IRM 5.12.10"));
        assert!(r.citation.contains("Pub. 783"));
        assert!(r.citation.contains("Pub. 784"));
    }

    #[test]
    fn note_pins_30_day_release() {
        let r = check(&release_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6325(a)")
            && n.contains("30 days")
            && n.contains("bond accepted")));
    }

    #[test]
    fn note_pins_discharge_five_bases() {
        let r = check(&release_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6325(b)")
            && n.contains("(b)(1)")
            && n.contains("(b)(2)(A)")
            && n.contains("(b)(2)(B)")
            && n.contains("(b)(3)")
            && n.contains("(b)(4)")));
    }

    #[test]
    fn note_pins_subordination_two_bases() {
        let r = check(&release_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6325(d)")
            && n.contains("(d)(1)")
            && n.contains("(d)(2)")
            && n.contains("trader refinance")));
    }

    #[test]
    fn note_pins_6325f_conclusiveness() {
        let r = check(&release_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6325(f)")
            && n.contains("CONCLUSIVE")
            && n.contains("extinguishes")
            && n.contains("subordinated interest")));
    }

    #[test]
    fn release_uniquely_engages_30_day_clock_invariant() {
        let mut i_release = release_base();
        i_release.days_since_qualifying_event = 31;
        let r_release = check(&i_release);
        assert!(!r_release.certificate_issuable);

        let mut i_discharge = discharge_double_value_base();
        i_discharge.days_since_qualifying_event = 31;
        let r_discharge = check(&i_discharge);
        assert!(r_discharge.certificate_issuable);
    }

    #[test]
    fn double_value_rule_only_engages_for_b1_discharge_invariant() {
        let mut i_b1 = discharge_double_value_base();
        i_b1.discharge_basis = DischargeBasis::DoubleValueRule;
        i_b1.property_value_cents = 1_000;
        let r_b1 = check(&i_b1);
        assert!(!r_b1.certificate_issuable);

        let mut i_b3 = discharge_double_value_base();
        i_b3.discharge_basis = DischargeBasis::ProceedsSubstituted;
        i_b3.property_value_cents = 1_000;
        let r_b3 = check(&i_b3);
        assert!(r_b3.certificate_issuable);
    }
}
