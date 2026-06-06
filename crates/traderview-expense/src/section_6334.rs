//! IRC § 6334 — Property exempt from levy. Trader-relevant
//! because trader-landlords with IRS collection exposure
//! need to know precisely what assets are exempt from levy:
//! tools-of-trade (trading rigs / monitors / books) + fuel +
//! household provisions + wages + principal residence
//! (judicial-approval gate) + retirement income + workers'
//! comp + Job Training assistance. Procedural-companion to
//! § 7421 (Anti-Injunction Act + § 7426 wrongful levy
//! exception), § 7433 (civil damages for unauthorized
//! collection), § 7430 (litigation costs), and § 7811 (TAOs).
//!
//! **§ 6334(a) thirteen exemption categories**:
//! 1. wearing apparel + school books (no dollar cap)
//! 2. fuel + provisions + furniture + household personal
//!    effects ≤ **$11,980 (2026 indexed)**
//! 3. books + tools necessary for trade, business, or
//!    profession ≤ **$5,990 (2026 indexed)**
//! 4. unemployment benefits
//! 5. undelivered mail
//! 6. certain annuity + pension payments (Railroad
//!    Retirement, Federal Civil Service Retirement)
//! 7. workmen's compensation
//! 8. judgments for support of minor children
//! 9. minimum exemption for wages / salary computed via
//!    § 6334(d) — **$5,300 (2026 § 6334(d)(4)(B) parameter)**
//! 10. service-connected military disability payments
//! 11. certain public assistance payments (TANF + state
//!     welfare)
//! 12. assistance under Job Training Partnership Act
//! 13. residences exempt in SMALL DEFICIENCY cases (≤ $5,000
//!     unpaid tax)
//!
//! **§ 6334(e)(1) principal residence judicial-approval gate**
//! — principal residence within meaning of § 121 is exempt
//! from levy except where judge or magistrate of district
//! court approves the levy IN WRITING. District courts have
//! EXCLUSIVE jurisdiction.
//!
//! **§ 6334(e)(2) self-employed assets + non-rental
//! residential real property** — IRS area director approval
//! required.
//!
//! Citations: 26 USC § 6334(a)(1)-(13), (d), (e)(1), (e)(2);
//! Rev. Proc. 2025-32 (2026 inflation-adjusted amounts);
//! Pub. L. 119-21 (One Big Beautiful Bill Act amendments).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PropertyCategory {
    /// § 6334(a)(1) — wearing apparel + school books.
    WearingApparelAndSchoolBooks,
    /// § 6334(a)(2) — fuel + provisions + furniture +
    /// household personal effects + arms for personal use +
    /// livestock + poultry; 2026 cap $11,980.
    FuelProvisionsFurnitureHousehold,
    /// § 6334(a)(3) — books + tools necessary for trade /
    /// business / profession; 2026 cap $5,990.
    BooksAndToolsOfTrade,
    /// § 6334(a)(4) — unemployment benefits.
    UnemploymentBenefits,
    /// § 6334(a)(5) — undelivered mail.
    UndeliveredMail,
    /// § 6334(a)(6) — certain annuity / pension payments
    /// (Railroad Retirement, Federal Civil Service
    /// Retirement).
    AnnuityOrPensionPayments,
    /// § 6334(a)(7) — workmen's compensation.
    WorkmensCompensation,
    /// § 6334(a)(8) — judgments for support of minor
    /// children.
    SupportOfMinorChildren,
    /// § 6334(a)(9) — minimum exemption for wages / salary.
    WageMinimumExemption,
    /// § 6334(a)(10) — service-connected military disability
    /// payments.
    MilitaryDisabilityPayments,
    /// § 6334(a)(11) — public assistance (TANF + state
    /// welfare).
    PublicAssistance,
    /// § 6334(a)(12) — Job Training Partnership Act
    /// assistance.
    JobTrainingPartnershipAct,
    /// § 6334(a)(13) — residence in SMALL DEFICIENCY cases
    /// (unpaid tax ≤ $5,000).
    ResidenceSmallDeficiency,
    /// § 6334(e)(1) — principal residence requires district
    /// court judge or magistrate written approval.
    PrincipalResidence,
    /// § 6334(e)(2) — self-employed assets + non-rental
    /// residential real property require IRS area director
    /// approval.
    SelfEmployedAssetsOrNonRentalResidence,
    /// Property NOT in any § 6334 exemption category.
    NotExempt,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6334Input {
    pub property_category: PropertyCategory,
    /// Property value in cents (for dollar-capped exemptions).
    pub property_value_cents: i64,
    /// Whether district court judge / magistrate has approved
    /// principal residence levy IN WRITING (for § 6334(e)(1)
    /// gate).
    pub district_court_written_approval: bool,
    /// Whether IRS area director has approved self-employed
    /// asset or non-rental residence levy (for § 6334(e)(2)
    /// gate).
    pub area_director_approval: bool,
    /// Unpaid tax amount in cents (for § 6334(a)(13) small
    /// deficiency residence exemption — ≤ $5,000 / 500,000
    /// cents).
    pub unpaid_tax_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6334Result {
    pub exempt_from_levy: bool,
    pub exemption_amount_cents: i64,
    pub cap_engaged: bool,
    pub cap_cents: i64,
    pub judicial_or_admin_approval_required: bool,
    pub approval_satisfied: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6334Input) -> Section6334Result {
    let mut failure_reasons: Vec<String> = Vec::new();
    let value = input.property_value_cents.max(0);
    let unpaid = input.unpaid_tax_cents.max(0);

    let (exempt, exemption_amount, cap, judicial_or_admin) = match input.property_category {
        PropertyCategory::WearingApparelAndSchoolBooks => (true, value, 0, false),
        PropertyCategory::FuelProvisionsFurnitureHousehold => {
            let cap = 1_198_000_i64;
            (true, value.min(cap), cap, false)
        }
        PropertyCategory::BooksAndToolsOfTrade => {
            let cap = 599_000_i64;
            (true, value.min(cap), cap, false)
        }
        PropertyCategory::UnemploymentBenefits => (true, value, 0, false),
        PropertyCategory::UndeliveredMail => (true, value, 0, false),
        PropertyCategory::AnnuityOrPensionPayments => (true, value, 0, false),
        PropertyCategory::WorkmensCompensation => (true, value, 0, false),
        PropertyCategory::SupportOfMinorChildren => (true, value, 0, false),
        PropertyCategory::WageMinimumExemption => (true, value, 0, false),
        PropertyCategory::MilitaryDisabilityPayments => (true, value, 0, false),
        PropertyCategory::PublicAssistance => (true, value, 0, false),
        PropertyCategory::JobTrainingPartnershipAct => (true, value, 0, false),
        PropertyCategory::ResidenceSmallDeficiency => {
            let unpaid_cap = 500_000_i64;
            let small = unpaid <= unpaid_cap;
            if !small {
                failure_reasons.push(format!(
                    "26 USC § 6334(a)(13) — small-deficiency residence exemption applies only when unpaid tax ≤ $5,000 (${} cents claimed)",
                    unpaid
                ));
            }
            (small, value, unpaid_cap, false)
        }
        PropertyCategory::PrincipalResidence => {
            let approved = input.district_court_written_approval;
            if approved {
                failure_reasons.push(
                    "26 USC § 6334(e)(1) — district court judge or magistrate approved levy of principal residence IN WRITING; § 121 residence exemption overridden".to_string(),
                );
            }
            (!approved, value, 0, true)
        }
        PropertyCategory::SelfEmployedAssetsOrNonRentalResidence => {
            let approved = input.area_director_approval;
            if approved {
                failure_reasons.push(
                    "26 USC § 6334(e)(2) — IRS area director approved levy of self-employed asset or non-rental residence; exemption overridden".to_string(),
                );
            }
            (!approved, value, 0, true)
        }
        PropertyCategory::NotExempt => {
            failure_reasons.push(
                "26 USC § 6334(a) — property not within any of the 13 enumerated exemption categories; subject to levy".to_string(),
            );
            (false, 0, 0, false)
        }
    };

    let cap_engaged = cap > 0 && value > cap;

    let approval_satisfied = match input.property_category {
        PropertyCategory::PrincipalResidence => !input.district_court_written_approval,
        PropertyCategory::SelfEmployedAssetsOrNonRentalResidence => !input.area_director_approval,
        _ => true,
    };

    let notes: Vec<String> = vec![
        "26 USC § 6334(a) — 13 enumerated exemption categories: (1) wearing apparel + school books; (2) fuel + provisions + furniture + household ≤ $11,980 (2026); (3) books + tools of trade ≤ $5,990 (2026); (4) unemployment; (5) undelivered mail; (6) annuity/pension; (7) workmen's comp; (8) child support; (9) wage minimum; (10) military disability; (11) public assistance; (12) Job Training Partnership Act; (13) residence in small deficiency cases"
            .to_string(),
        "26 USC § 6334(e)(1) — principal residence (within meaning of § 121) requires district court judge or magistrate WRITTEN approval before levy; district courts have EXCLUSIVE jurisdiction"
            .to_string(),
        "26 USC § 6334(e)(2) — self-employed assets + non-rental residential real property require IRS area director approval before levy"
            .to_string(),
        "26 USC § 6334(d)(4)(B) — 2026 wage exemption parameter = $5,300; Rev. Proc. 2025-32 (2026 inflation-adjusted amounts); Pub. L. 119-21 (One Big Beautiful Bill Act)"
            .to_string(),
    ];

    Section6334Result {
        exempt_from_levy: exempt,
        exemption_amount_cents: exemption_amount,
        cap_engaged,
        cap_cents: cap,
        judicial_or_admin_approval_required: judicial_or_admin,
        approval_satisfied,
        failure_reasons,
        citation: "26 USC § 6334(a)(1)-(13), (d)(4)(B), (e)(1), (e)(2); Rev. Proc. 2025-32",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section6334Input {
        Section6334Input {
            property_category: PropertyCategory::WearingApparelAndSchoolBooks,
            property_value_cents: 100_000,
            district_court_written_approval: false,
            area_director_approval: false,
            unpaid_tax_cents: 0,
        }
    }

    #[test]
    fn wearing_apparel_exempt_no_cap() {
        let r = check(&base());
        assert!(r.exempt_from_levy);
        assert_eq!(r.cap_cents, 0);
    }

    #[test]
    fn fuel_provisions_within_cap_full_exemption() {
        let mut i = base();
        i.property_category = PropertyCategory::FuelProvisionsFurnitureHousehold;
        i.property_value_cents = 1_000_000;
        let r = check(&i);
        assert!(r.exempt_from_levy);
        assert_eq!(r.exemption_amount_cents, 1_000_000);
        assert_eq!(r.cap_cents, 1_198_000);
        assert!(!r.cap_engaged);
    }

    #[test]
    fn fuel_provisions_at_cap_boundary() {
        let mut i = base();
        i.property_category = PropertyCategory::FuelProvisionsFurnitureHousehold;
        i.property_value_cents = 1_198_000;
        let r = check(&i);
        assert!(r.exempt_from_levy);
        assert_eq!(r.exemption_amount_cents, 1_198_000);
        assert!(!r.cap_engaged);
    }

    #[test]
    fn fuel_provisions_over_cap_engaged() {
        let mut i = base();
        i.property_category = PropertyCategory::FuelProvisionsFurnitureHousehold;
        i.property_value_cents = 2_000_000;
        let r = check(&i);
        assert!(r.exempt_from_levy);
        assert_eq!(r.exemption_amount_cents, 1_198_000);
        assert!(r.cap_engaged);
    }

    #[test]
    fn tools_of_trade_within_cap() {
        let mut i = base();
        i.property_category = PropertyCategory::BooksAndToolsOfTrade;
        i.property_value_cents = 500_000;
        let r = check(&i);
        assert!(r.exempt_from_levy);
        assert_eq!(r.exemption_amount_cents, 500_000);
        assert_eq!(r.cap_cents, 599_000);
        assert!(!r.cap_engaged);
    }

    #[test]
    fn tools_of_trade_at_cap_boundary() {
        let mut i = base();
        i.property_category = PropertyCategory::BooksAndToolsOfTrade;
        i.property_value_cents = 599_000;
        let r = check(&i);
        assert!(r.exempt_from_levy);
        assert_eq!(r.exemption_amount_cents, 599_000);
        assert!(!r.cap_engaged);
    }

    #[test]
    fn tools_of_trade_over_cap_engaged() {
        let mut i = base();
        i.property_category = PropertyCategory::BooksAndToolsOfTrade;
        i.property_value_cents = 1_000_000;
        let r = check(&i);
        assert_eq!(r.exemption_amount_cents, 599_000);
        assert!(r.cap_engaged);
    }

    #[test]
    fn principal_residence_no_judicial_approval_exempt() {
        let mut i = base();
        i.property_category = PropertyCategory::PrincipalResidence;
        let r = check(&i);
        assert!(r.exempt_from_levy);
        assert!(r.judicial_or_admin_approval_required);
        assert!(r.approval_satisfied);
        assert!(r.failure_reasons.is_empty());
    }

    #[test]
    fn principal_residence_with_judicial_approval_levied() {
        let mut i = base();
        i.property_category = PropertyCategory::PrincipalResidence;
        i.district_court_written_approval = true;
        let r = check(&i);
        assert!(!r.exempt_from_levy);
        assert!(!r.approval_satisfied);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6334(e)(1)") && f.contains("WRITING")));
    }

    #[test]
    fn self_employed_no_area_director_approval_exempt() {
        let mut i = base();
        i.property_category = PropertyCategory::SelfEmployedAssetsOrNonRentalResidence;
        let r = check(&i);
        assert!(r.exempt_from_levy);
        assert!(r.judicial_or_admin_approval_required);
    }

    #[test]
    fn self_employed_with_area_director_approval_levied() {
        let mut i = base();
        i.property_category = PropertyCategory::SelfEmployedAssetsOrNonRentalResidence;
        i.area_director_approval = true;
        let r = check(&i);
        assert!(!r.exempt_from_levy);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 6334(e)(2)")));
    }

    #[test]
    fn small_deficiency_residence_at_5000_boundary_exempt() {
        let mut i = base();
        i.property_category = PropertyCategory::ResidenceSmallDeficiency;
        i.unpaid_tax_cents = 500_000;
        let r = check(&i);
        assert!(r.exempt_from_levy);
    }

    #[test]
    fn small_deficiency_residence_over_5000_levied() {
        let mut i = base();
        i.property_category = PropertyCategory::ResidenceSmallDeficiency;
        i.unpaid_tax_cents = 500_001;
        let r = check(&i);
        assert!(!r.exempt_from_levy);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6334(a)(13)")));
    }

    #[test]
    fn unemployment_benefits_exempt() {
        let mut i = base();
        i.property_category = PropertyCategory::UnemploymentBenefits;
        let r = check(&i);
        assert!(r.exempt_from_levy);
    }

    #[test]
    fn workmens_compensation_exempt() {
        let mut i = base();
        i.property_category = PropertyCategory::WorkmensCompensation;
        let r = check(&i);
        assert!(r.exempt_from_levy);
    }

    #[test]
    fn child_support_exempt() {
        let mut i = base();
        i.property_category = PropertyCategory::SupportOfMinorChildren;
        let r = check(&i);
        assert!(r.exempt_from_levy);
    }

    #[test]
    fn wage_minimum_exemption_engaged() {
        let mut i = base();
        i.property_category = PropertyCategory::WageMinimumExemption;
        let r = check(&i);
        assert!(r.exempt_from_levy);
    }

    #[test]
    fn military_disability_exempt() {
        let mut i = base();
        i.property_category = PropertyCategory::MilitaryDisabilityPayments;
        let r = check(&i);
        assert!(r.exempt_from_levy);
    }

    #[test]
    fn public_assistance_exempt() {
        let mut i = base();
        i.property_category = PropertyCategory::PublicAssistance;
        let r = check(&i);
        assert!(r.exempt_from_levy);
    }

    #[test]
    fn job_training_act_exempt() {
        let mut i = base();
        i.property_category = PropertyCategory::JobTrainingPartnershipAct;
        let r = check(&i);
        assert!(r.exempt_from_levy);
    }

    #[test]
    fn annuity_or_pension_exempt() {
        let mut i = base();
        i.property_category = PropertyCategory::AnnuityOrPensionPayments;
        let r = check(&i);
        assert!(r.exempt_from_levy);
    }

    #[test]
    fn undelivered_mail_exempt() {
        let mut i = base();
        i.property_category = PropertyCategory::UndeliveredMail;
        let r = check(&i);
        assert!(r.exempt_from_levy);
    }

    #[test]
    fn not_exempt_property_levied() {
        let mut i = base();
        i.property_category = PropertyCategory::NotExempt;
        let r = check(&i);
        assert!(!r.exempt_from_levy);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 6334(a)") && f.contains("13 enumerated")));
    }

    #[test]
    fn citation_pins_subsections_and_rev_proc() {
        let r = check(&base());
        assert!(r.citation.contains("§ 6334(a)(1)-(13)"));
        assert!(r.citation.contains("(d)(4)(B)"));
        assert!(r.citation.contains("(e)(1)"));
        assert!(r.citation.contains("(e)(2)"));
        assert!(r.citation.contains("Rev. Proc. 2025-32"));
    }

    #[test]
    fn note_pins_2026_inflation_amounts() {
        let r = check(&base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("$11,980") && n.contains("$5,990") && n.contains("2026")));
    }

    #[test]
    fn note_pins_principal_residence_judicial_gate() {
        let r = check(&base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6334(e)(1)") && n.contains("EXCLUSIVE jurisdiction")));
    }

    #[test]
    fn note_pins_self_employed_area_director_gate() {
        let r = check(&base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 6334(e)(2)") && n.contains("area director")));
    }

    #[test]
    fn note_pins_2026_wage_5300_parameter() {
        let r = check(&base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("$5,300") && n.contains("§ 6334(d)(4)(B)")));
    }

    #[test]
    fn defensive_negative_property_value_clamped() {
        let mut i = base();
        i.property_category = PropertyCategory::BooksAndToolsOfTrade;
        i.property_value_cents = -100_000;
        let r = check(&i);
        assert!(r.exempt_from_levy);
        assert_eq!(r.exemption_amount_cents, 0);
        assert!(!r.cap_engaged);
    }

    #[test]
    fn defensive_negative_unpaid_tax_clamped() {
        let mut i = base();
        i.property_category = PropertyCategory::ResidenceSmallDeficiency;
        i.unpaid_tax_cents = -100_000;
        let r = check(&i);
        assert!(r.exempt_from_levy);
    }

    #[test]
    fn fuel_provisions_uniquely_higher_cap_than_tools_invariant() {
        let mut i_fuel = base();
        i_fuel.property_category = PropertyCategory::FuelProvisionsFurnitureHousehold;
        let r_fuel = check(&i_fuel);

        let mut i_tools = base();
        i_tools.property_category = PropertyCategory::BooksAndToolsOfTrade;
        let r_tools = check(&i_tools);

        assert!(r_fuel.cap_cents > r_tools.cap_cents);
        assert_eq!(r_fuel.cap_cents, 1_198_000);
        assert_eq!(r_tools.cap_cents, 599_000);
    }

    #[test]
    fn principal_residence_uniquely_requires_district_court_invariant() {
        let mut i_principal = base();
        i_principal.property_category = PropertyCategory::PrincipalResidence;
        let r_principal = check(&i_principal);
        assert!(r_principal.judicial_or_admin_approval_required);

        let mut i_tools = base();
        i_tools.property_category = PropertyCategory::BooksAndToolsOfTrade;
        let r_tools = check(&i_tools);
        assert!(!r_tools.judicial_or_admin_approval_required);
    }

    #[test]
    fn small_deficiency_5000_uniquely_caps_unpaid_tax_invariant() {
        let mut i_small = base();
        i_small.property_category = PropertyCategory::ResidenceSmallDeficiency;
        i_small.unpaid_tax_cents = 500_000;
        let r_small = check(&i_small);
        assert!(r_small.exempt_from_levy);
        assert_eq!(r_small.cap_cents, 500_000);
    }

    #[test]
    fn property_category_truth_table_capped_vs_uncapped() {
        let mut i = base();

        for cat in [
            PropertyCategory::FuelProvisionsFurnitureHousehold,
            PropertyCategory::BooksAndToolsOfTrade,
        ] {
            i.property_category = cat;
            let r = check(&i);
            assert!(r.cap_cents > 0);
        }

        for cat in [
            PropertyCategory::WearingApparelAndSchoolBooks,
            PropertyCategory::UnemploymentBenefits,
            PropertyCategory::UndeliveredMail,
            PropertyCategory::AnnuityOrPensionPayments,
            PropertyCategory::WorkmensCompensation,
            PropertyCategory::SupportOfMinorChildren,
            PropertyCategory::WageMinimumExemption,
            PropertyCategory::MilitaryDisabilityPayments,
            PropertyCategory::PublicAssistance,
            PropertyCategory::JobTrainingPartnershipAct,
            PropertyCategory::PrincipalResidence,
            PropertyCategory::SelfEmployedAssetsOrNonRentalResidence,
        ] {
            i.property_category = cat;
            let r = check(&i);
            assert_eq!(r.cap_cents, 0);
        }
    }

    #[test]
    fn approval_satisfied_truth_table_for_residence_gates() {
        let mut i_principal_no_approval = base();
        i_principal_no_approval.property_category = PropertyCategory::PrincipalResidence;
        i_principal_no_approval.district_court_written_approval = false;
        let r_p_no = check(&i_principal_no_approval);
        assert!(r_p_no.approval_satisfied);
        assert!(r_p_no.exempt_from_levy);

        let mut i_principal_approval = base();
        i_principal_approval.property_category = PropertyCategory::PrincipalResidence;
        i_principal_approval.district_court_written_approval = true;
        let r_p_yes = check(&i_principal_approval);
        assert!(!r_p_yes.approval_satisfied);
        assert!(!r_p_yes.exempt_from_levy);

        let mut i_self_emp_no = base();
        i_self_emp_no.property_category = PropertyCategory::SelfEmployedAssetsOrNonRentalResidence;
        i_self_emp_no.area_director_approval = false;
        let r_se_no = check(&i_self_emp_no);
        assert!(r_se_no.approval_satisfied);

        let mut i_self_emp_yes = base();
        i_self_emp_yes.property_category = PropertyCategory::SelfEmployedAssetsOrNonRentalResidence;
        i_self_emp_yes.area_director_approval = true;
        let r_se_yes = check(&i_self_emp_yes);
        assert!(!r_se_yes.approval_satisfied);
    }

    #[test]
    fn fuel_at_1_cent_under_cap_compliant() {
        let mut i = base();
        i.property_category = PropertyCategory::FuelProvisionsFurnitureHousehold;
        i.property_value_cents = 1_197_999;
        let r = check(&i);
        assert_eq!(r.exemption_amount_cents, 1_197_999);
        assert!(!r.cap_engaged);
    }

    #[test]
    fn fuel_at_1_cent_over_cap_engaged() {
        let mut i = base();
        i.property_category = PropertyCategory::FuelProvisionsFurnitureHousehold;
        i.property_value_cents = 1_198_001;
        let r = check(&i);
        assert_eq!(r.exemption_amount_cents, 1_198_000);
        assert!(r.cap_engaged);
    }
}
