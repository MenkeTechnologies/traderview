//! IRC § 6323 — Validity and priority of federal tax lien
//! against certain persons. Determines when a § 6321 lien
//! that ATTACHES to taxpayer's property (assessment + notice
//! and demand + neglect or refusal under § 6321) actually
//! takes PRIORITY over competing third-party interests.
//! Trader-relevant for trader-landlords whose rental property
//! holdings interact with mortgages + judgment liens +
//! mechanics' liens + other secured creditors.
//!
//! **§ 6323(a) four protected classes** — lien is NOT valid
//! against these four classes UNTIL Notice of Federal Tax
//! Lien (NFTL) is filed under § 6323(f):
//! 1. Purchaser
//! 2. Holder of security interest
//! 3. Mechanic's lienor
//! 4. Judgment lien creditor
//!
//! First-in-time wins among NFTL filing and competing third-
//! party interest perfection.
//!
//! **§ 6323(b) ten superpriorities** — priority OVER federal
//! tax lien EVEN AFTER NFTL is filed, when interest comes
//! into existence without actual notice of federal tax lien:
//! 1. Securities (§ 6323(b)(1))
//! 2. Motor vehicles (§ 6323(b)(2))
//! 3. Personal property purchased at retail (§ 6323(b)(3))
//! 4. Personal property purchased in casual sale
//!    (§ 6323(b)(4))
//! 5. Personal property subject to possessory lien
//!    (§ 6323(b)(5))
//! 6. Real property tax and special assessment liens
//!    (§ 6323(b)(6))
//! 7. Residential property subject to mechanic's lien for
//!    repair/improvement (limited; § 6323(b)(7))
//! 8. Attorney's lien (§ 6323(b)(8))
//! 9. Insurance contracts (§ 6323(b)(9))
//! 10. Passbook loans (§ 6323(b)(10))
//!
//! **§ 6323(c)+(d) commercial transaction protection** —
//! 45-day window for after-acquired property under
//! commercial transactions financing agreements + personal
//! property purchases without actual notice.
//!
//! **§ 6323(f) NFTL filing location** — typically state
//! property records / Secretary of State filing system.
//!
//! **§ 6323(g) NFTL refiling** — refiling required every 10
//! years to maintain priority status against subsequent
//! interests.
//!
//! Citations: 26 USC § 6323(a)(1)-(4), (b)(1)-(10), (c),
//! (d), (f), (g), (h); § 6321 (lien attachment); § 6322
//! (period of lien); Rev. Rul. 2003-108; IRM 5.12.1
//! (Lien Program Overview); IRM 5.12.2 (Notice of Lien
//! Determinations); IRM 5.12.7 (Notice of Lien Preparation
//! and Filing); IRM 5.12.8 (Notice of Lien Refiling).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CompetingClaimant {
    /// § 6323(a)(1) — purchaser.
    Purchaser,
    /// § 6323(a)(2) — holder of security interest (mortgage
    /// lender, UCC-1 secured party, etc.).
    HolderOfSecurityInterest,
    /// § 6323(a)(3) — mechanic's lienor.
    MechanicsLienor,
    /// § 6323(a)(4) — judgment lien creditor.
    JudgmentLienCreditor,
    /// Not within any § 6323(a) protected class (general
    /// unsecured creditor).
    UnprotectedCreditor,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SuperpriorityType {
    /// § 6323(b)(1) — securities.
    Securities,
    /// § 6323(b)(2) — motor vehicles.
    MotorVehicle,
    /// § 6323(b)(3) — personal property purchased at retail.
    RetailPurchase,
    /// § 6323(b)(4) — personal property purchased in casual
    /// sale.
    CasualSalePurchase,
    /// § 6323(b)(5) — personal property subject to possessory
    /// lien.
    PossessoryLien,
    /// § 6323(b)(6) — real property tax and special
    /// assessment liens.
    RealPropertyTaxLien,
    /// § 6323(b)(7) — residential property mechanic's lien
    /// (limited to repair/improvement).
    ResidentialMechanicsLien,
    /// § 6323(b)(8) — attorney's lien.
    AttorneysLien,
    /// § 6323(b)(9) — insurance contracts.
    InsuranceContract,
    /// § 6323(b)(10) — passbook loans.
    PassbookLoan,
    /// None.
    None,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6323Input {
    pub competing_claimant: CompetingClaimant,
    pub superpriority_type: SuperpriorityType,
    /// Whether the IRS has filed Notice of Federal Tax Lien
    /// under § 6323(f).
    pub nftl_filed: bool,
    /// Whether the NFTL was filed BEFORE the competing
    /// claimant's interest was perfected (for § 6323(a)
    /// first-in-time rule).
    pub nftl_filed_before_competing_interest: bool,
    /// Whether the competing claimant had actual notice of
    /// the federal tax lien at the time their interest arose
    /// (gating clause for § 6323(b) superpriority
    /// protection).
    pub claimant_had_actual_notice: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6323Result {
    pub federal_tax_lien_has_priority: bool,
    pub claimant_in_protected_class: bool,
    pub superpriority_applies: bool,
    pub nftl_filed: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6323Input) -> Section6323Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let claimant_protected = !matches!(
        input.competing_claimant,
        CompetingClaimant::UnprotectedCreditor
    );

    let superpriority_engaged = !matches!(input.superpriority_type, SuperpriorityType::None)
        && !input.claimant_had_actual_notice;

    if superpriority_engaged {
        failure_reasons.push(format!(
            "26 USC § 6323(b) — superpriority {:?} engaged; competing claimant takes priority over federal tax lien EVEN AFTER NFTL filed (no actual notice at time interest arose)",
            input.superpriority_type
        ));
    } else if claimant_protected {
        let lien_priority = input.nftl_filed && input.nftl_filed_before_competing_interest;
        if !lien_priority {
            if !input.nftl_filed {
                failure_reasons.push(
                    "26 USC § 6323(a) — federal tax lien NOT valid against protected class until NFTL filed under § 6323(f); competing claimant has priority".to_string(),
                );
            } else if !input.nftl_filed_before_competing_interest {
                failure_reasons.push(
                    "26 USC § 6323(a) — NFTL filed AFTER competing claimant's interest perfected; first-in-time rule grants priority to competing claimant".to_string(),
                );
            }
        }
    }

    let lien_priority_engaged = if superpriority_engaged {
        false
    } else if claimant_protected {
        input.nftl_filed && input.nftl_filed_before_competing_interest
    } else {
        true
    };

    let notes: Vec<String> = vec![
        "26 USC § 6323(a) — federal tax lien NOT valid against (1) purchaser; (2) holder of security interest; (3) mechanic's lienor; (4) judgment lien creditor UNTIL NFTL filed under § 6323(f); first-in-time wins among NFTL and competing perfection"
            .to_string(),
        "26 USC § 6323(b) — ten superpriorities take priority over federal tax lien EVEN AFTER NFTL filed when interest arose without actual notice: (1) securities; (2) motor vehicles; (3) retail purchase; (4) casual sale; (5) possessory lien; (6) real property tax / special assessment; (7) residential mechanic's lien (repair/improvement); (8) attorney's lien; (9) insurance contracts; (10) passbook loans"
            .to_string(),
        "26 USC § 6323(c) + (d) — 45-day window for commercial transactions financing agreements + after-acquired personal property purchases without actual notice"
            .to_string(),
        "26 USC § 6323(g) — NFTL refiling required every 10 years to maintain priority status against subsequent interests; paired with § 6502 10-year CSED"
            .to_string(),
    ];

    Section6323Result {
        federal_tax_lien_has_priority: lien_priority_engaged,
        claimant_in_protected_class: claimant_protected,
        superpriority_applies: superpriority_engaged,
        nftl_filed: input.nftl_filed,
        failure_reasons,
        citation: "26 USC § 6323(a)(1)-(4), (b)(1)-(10), (c), (d), (f), (g), (h); § 6321; § 6322; Rev. Rul. 2003-108; IRM 5.12.1, 5.12.2, 5.12.7, 5.12.8",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lien_priority_base() -> Section6323Input {
        Section6323Input {
            competing_claimant: CompetingClaimant::HolderOfSecurityInterest,
            superpriority_type: SuperpriorityType::None,
            nftl_filed: true,
            nftl_filed_before_competing_interest: true,
            claimant_had_actual_notice: false,
        }
    }

    #[test]
    fn nftl_filed_first_grants_lien_priority() {
        let r = check(&lien_priority_base());
        assert!(r.federal_tax_lien_has_priority);
        assert!(r.claimant_in_protected_class);
        assert!(!r.superpriority_applies);
        assert!(r.failure_reasons.is_empty());
    }

    #[test]
    fn nftl_not_filed_competing_claimant_wins() {
        let mut i = lien_priority_base();
        i.nftl_filed = false;
        let r = check(&i);
        assert!(!r.federal_tax_lien_has_priority);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6323(a)") && f.contains("until NFTL filed")));
    }

    #[test]
    fn nftl_filed_after_competing_interest_loses_priority() {
        let mut i = lien_priority_base();
        i.nftl_filed_before_competing_interest = false;
        let r = check(&i);
        assert!(!r.federal_tax_lien_has_priority);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6323(a)") && f.contains("first-in-time")));
    }

    #[test]
    fn purchaser_class_protected() {
        let mut i = lien_priority_base();
        i.competing_claimant = CompetingClaimant::Purchaser;
        i.nftl_filed = false;
        let r = check(&i);
        assert!(r.claimant_in_protected_class);
        assert!(!r.federal_tax_lien_has_priority);
    }

    #[test]
    fn mechanics_lienor_class_protected() {
        let mut i = lien_priority_base();
        i.competing_claimant = CompetingClaimant::MechanicsLienor;
        i.nftl_filed = false;
        let r = check(&i);
        assert!(r.claimant_in_protected_class);
    }

    #[test]
    fn judgment_lien_creditor_class_protected() {
        let mut i = lien_priority_base();
        i.competing_claimant = CompetingClaimant::JudgmentLienCreditor;
        i.nftl_filed = false;
        let r = check(&i);
        assert!(r.claimant_in_protected_class);
    }

    #[test]
    fn unprotected_creditor_loses_to_lien_even_without_nftl() {
        let mut i = lien_priority_base();
        i.competing_claimant = CompetingClaimant::UnprotectedCreditor;
        i.nftl_filed = false;
        let r = check(&i);
        assert!(r.federal_tax_lien_has_priority);
        assert!(!r.claimant_in_protected_class);
    }

    #[test]
    fn superpriority_securities_overrides_lien() {
        let mut i = lien_priority_base();
        i.superpriority_type = SuperpriorityType::Securities;
        let r = check(&i);
        assert!(!r.federal_tax_lien_has_priority);
        assert!(r.superpriority_applies);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6323(b)") && f.contains("Securities")));
    }

    #[test]
    fn superpriority_motor_vehicle_overrides_lien() {
        let mut i = lien_priority_base();
        i.superpriority_type = SuperpriorityType::MotorVehicle;
        let r = check(&i);
        assert!(r.superpriority_applies);
        assert!(!r.federal_tax_lien_has_priority);
    }

    #[test]
    fn superpriority_attorneys_lien_overrides() {
        let mut i = lien_priority_base();
        i.superpriority_type = SuperpriorityType::AttorneysLien;
        let r = check(&i);
        assert!(r.superpriority_applies);
    }

    #[test]
    fn superpriority_blocked_by_actual_notice() {
        let mut i = lien_priority_base();
        i.superpriority_type = SuperpriorityType::Securities;
        i.claimant_had_actual_notice = true;
        let r = check(&i);
        assert!(!r.superpriority_applies);
        assert!(r.federal_tax_lien_has_priority);
    }

    #[test]
    fn superpriority_real_property_tax_overrides() {
        let mut i = lien_priority_base();
        i.superpriority_type = SuperpriorityType::RealPropertyTaxLien;
        let r = check(&i);
        assert!(r.superpriority_applies);
    }

    #[test]
    fn superpriority_residential_mechanics_lien_overrides() {
        let mut i = lien_priority_base();
        i.superpriority_type = SuperpriorityType::ResidentialMechanicsLien;
        let r = check(&i);
        assert!(r.superpriority_applies);
    }

    #[test]
    fn superpriority_truth_table() {
        for sp in [
            SuperpriorityType::Securities,
            SuperpriorityType::MotorVehicle,
            SuperpriorityType::RetailPurchase,
            SuperpriorityType::CasualSalePurchase,
            SuperpriorityType::PossessoryLien,
            SuperpriorityType::RealPropertyTaxLien,
            SuperpriorityType::ResidentialMechanicsLien,
            SuperpriorityType::AttorneysLien,
            SuperpriorityType::InsuranceContract,
            SuperpriorityType::PassbookLoan,
        ] {
            let mut i = lien_priority_base();
            i.superpriority_type = sp;
            i.claimant_had_actual_notice = false;
            let r = check(&i);
            assert!(r.superpriority_applies);
            assert!(!r.federal_tax_lien_has_priority);
        }
    }

    #[test]
    fn competing_claimant_protection_truth_table() {
        for (claimant, exp_protected) in [
            (CompetingClaimant::Purchaser, true),
            (CompetingClaimant::HolderOfSecurityInterest, true),
            (CompetingClaimant::MechanicsLienor, true),
            (CompetingClaimant::JudgmentLienCreditor, true),
            (CompetingClaimant::UnprotectedCreditor, false),
        ] {
            let mut i = lien_priority_base();
            i.competing_claimant = claimant;
            let r = check(&i);
            assert_eq!(r.claimant_in_protected_class, exp_protected);
        }
    }

    #[test]
    fn nftl_filing_2x2_truth_table() {
        for (filed, before, exp_priority) in [
            (true, true, true),
            (true, false, false),
            (false, true, false),
            (false, false, false),
        ] {
            let mut i = lien_priority_base();
            i.nftl_filed = filed;
            i.nftl_filed_before_competing_interest = before;
            let r = check(&i);
            assert_eq!(r.federal_tax_lien_has_priority, exp_priority);
        }
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = check(&lien_priority_base());
        assert!(r.citation.contains("§ 6323(a)(1)-(4)"));
        assert!(r.citation.contains("(b)(1)-(10)"));
        assert!(r.citation.contains("(c)"));
        assert!(r.citation.contains("(d)"));
        assert!(r.citation.contains("(f)"));
        assert!(r.citation.contains("(g)"));
        assert!(r.citation.contains("(h)"));
        assert!(r.citation.contains("§ 6321"));
        assert!(r.citation.contains("§ 6322"));
        assert!(r.citation.contains("Rev. Rul. 2003-108"));
        assert!(r.citation.contains("IRM 5.12.1"));
        assert!(r.citation.contains("5.12.2"));
        assert!(r.citation.contains("5.12.7"));
        assert!(r.citation.contains("5.12.8"));
    }

    #[test]
    fn note_pins_four_protected_classes() {
        let r = check(&lien_priority_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6323(a)")
            && n.contains("purchaser")
            && n.contains("security interest")
            && n.contains("mechanic's lienor")
            && n.contains("judgment lien creditor")
            && n.contains("first-in-time")));
    }

    #[test]
    fn note_pins_ten_superpriorities() {
        let r = check(&lien_priority_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6323(b)")
            && n.contains("securities")
            && n.contains("motor vehicles")
            && n.contains("retail purchase")
            && n.contains("casual sale")
            && n.contains("possessory lien")
            && n.contains("real property tax")
            && n.contains("residential mechanic's lien")
            && n.contains("attorney's lien")
            && n.contains("insurance contracts")
            && n.contains("passbook loans")));
    }

    #[test]
    fn note_pins_45_day_commercial_window() {
        let r = check(&lien_priority_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6323(c)")
            && n.contains("(d)")
            && n.contains("45-day")));
    }

    #[test]
    fn note_pins_10_year_refiling() {
        let r = check(&lien_priority_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 6323(g)")
            && n.contains("10 years")
            && n.contains("§ 6502")));
    }

    #[test]
    fn nftl_filed_field_preserved() {
        let mut i = lien_priority_base();
        i.nftl_filed = true;
        let r = check(&i);
        assert!(r.nftl_filed);

        i.nftl_filed = false;
        let r = check(&i);
        assert!(!r.nftl_filed);
    }

    #[test]
    fn superpriority_overrides_nftl_filing() {
        let mut i = lien_priority_base();
        i.superpriority_type = SuperpriorityType::MotorVehicle;
        i.nftl_filed = true;
        i.nftl_filed_before_competing_interest = true;
        i.claimant_had_actual_notice = false;
        let r = check(&i);
        assert!(r.superpriority_applies);
        assert!(!r.federal_tax_lien_has_priority);
    }

    #[test]
    fn superpriority_with_actual_notice_loses() {
        let mut i = lien_priority_base();
        i.superpriority_type = SuperpriorityType::Securities;
        i.claimant_had_actual_notice = true;
        let r = check(&i);
        assert!(!r.superpriority_applies);
        assert!(r.federal_tax_lien_has_priority);
    }

    #[test]
    fn purchaser_no_nftl_loses_to_purchaser() {
        let mut i = lien_priority_base();
        i.competing_claimant = CompetingClaimant::Purchaser;
        i.nftl_filed = false;
        let r = check(&i);
        assert!(!r.federal_tax_lien_has_priority);
    }

    #[test]
    fn judgment_lien_creditor_nftl_first_wins() {
        let mut i = lien_priority_base();
        i.competing_claimant = CompetingClaimant::JudgmentLienCreditor;
        i.nftl_filed = true;
        i.nftl_filed_before_competing_interest = true;
        let r = check(&i);
        assert!(r.federal_tax_lien_has_priority);
    }

    #[test]
    fn unprotected_creditor_with_nftl_filed_lien_wins() {
        let mut i = lien_priority_base();
        i.competing_claimant = CompetingClaimant::UnprotectedCreditor;
        i.nftl_filed = true;
        let r = check(&i);
        assert!(r.federal_tax_lien_has_priority);
        assert!(!r.claimant_in_protected_class);
    }

    #[test]
    fn unprotected_creditor_no_nftl_lien_still_wins() {
        let mut i = lien_priority_base();
        i.competing_claimant = CompetingClaimant::UnprotectedCreditor;
        i.nftl_filed = false;
        let r = check(&i);
        assert!(r.federal_tax_lien_has_priority);
    }

    #[test]
    fn nftl_filed_priority_against_purchaser_invariant() {
        let mut i = lien_priority_base();
        i.competing_claimant = CompetingClaimant::Purchaser;
        i.nftl_filed = true;
        i.nftl_filed_before_competing_interest = true;
        let r = check(&i);
        assert!(r.federal_tax_lien_has_priority);

        i.nftl_filed = false;
        let r = check(&i);
        assert!(!r.federal_tax_lien_has_priority);
    }

    #[test]
    fn ten_superpriorities_all_distinct() {
        let all = [
            SuperpriorityType::Securities,
            SuperpriorityType::MotorVehicle,
            SuperpriorityType::RetailPurchase,
            SuperpriorityType::CasualSalePurchase,
            SuperpriorityType::PossessoryLien,
            SuperpriorityType::RealPropertyTaxLien,
            SuperpriorityType::ResidentialMechanicsLien,
            SuperpriorityType::AttorneysLien,
            SuperpriorityType::InsuranceContract,
            SuperpriorityType::PassbookLoan,
        ];
        for (i, sp_i) in all.iter().enumerate() {
            for (j, sp_j) in all.iter().enumerate() {
                if i != j {
                    assert_ne!(sp_i, sp_j);
                }
            }
        }
        assert_eq!(all.len(), 10);
    }
}
