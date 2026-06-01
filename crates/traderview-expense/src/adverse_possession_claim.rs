//! Adverse possession (squatter) statutory limitations check —
//! when may a third party acquire title to landlord property by
//! continuous unauthorized occupation? Critical for absentee
//! landlords (snowbirds, trader-investors with multiple rentals,
//! out-of-state owners) where property may be occupied without
//! the owner's notice for extended periods.
//!
//! Distinct from `landlord_possession_delivery` (delivery of
//! possession at tenancy start), `tenant_abandonment` (procedures
//! for declaring tenant possessions abandoned), and `foreclosure_
//! tenant_rights` (post-foreclosure tenant occupation under PTFA).
//! This module addresses the title-acquisition pathway under
//! state statutes of limitations.
//!
//! Five regimes:
//!
//! **California — Cal. Civ. Proc. Code § 325**. FIVE-YEAR period
//! with all five common-law elements (actual, open and notorious,
//! hostile, exclusive, continuous) PLUS payment of state, county,
//! and municipal taxes throughout the 5-year period. Per Gilardi v.
//! Hallam (1981), payment must be made in the year taxes are
//! ASSESSED — paying back taxes does not satisfy the requirement.
//! Hostile means without owner's consent; does not require
//! ill-will.
//!
//! **Texas — Tex. Civ. Prac. & Rem. Code Ch. 16**. Four tiered
//! periods depending on color of title, taxes, and cultivation:
//!   § 16.024 — 3 YEARS with COLOR OF TITLE (instrument purporting
//!     to convey valid title to claimant, regardless of recording);
//!   § 16.025 — 5 YEARS with RECORDED DEED + cultivation/use +
//!     payment of taxes;
//!   § 16.026 — 10 YEARS with peaceable + adverse possession +
//!     cultivation/use (NO tax payment, NO color of title required);
//!   § 16.027 — 25 YEARS regardless of any legal disability of the
//!     true owner (minority, mental incapacity);
//!   § 16.028 — 25 YEARS with recorded but void deed.
//!
//! **Florida — Fla. Stat. §§ 95.16, 95.18**. SEVEN-YEAR period
//! either WITH or WITHOUT color of title plus tax payment. § 95.16
//! (WITH color of title) requires 7 years continuous occupation
//! under color of title plus payment of all taxes. § 95.18
//! (WITHOUT color of title) requires 7 years continuous occupation
//! plus (a) payment of all outstanding taxes and matured special
//! improvement liens within 1 year of entering possession,
//! (b) filing a return with the property appraiser by proper legal
//! description within 30 days after complying with (a), and
//! (c) ongoing tax payment for all remaining years. Failure to pay
//! taxes defeats the claim. The appraiser-return requirement makes
//! Florida § 95.18 the strictest US adverse-possession regime.
//!
//! **New York — N.Y. RPAPL §§ 501(3), 511, 521 + CPLR § 212(a)**.
//! TEN-YEAR period with all common-law elements PLUS a "claim of
//! right" under RPAPL § 501(3), defined as "a reasonable basis for
//! the belief that the property belongs to the adverse possessor."
//! 2008 amendment (effective July 8, 2008) overruled Walling v.
//! Prysbylo — bad-faith adverse possessors who KNOW they do not
//! own the property can no longer acquire title. Reasonable
//! basis requirement applies to both § 511 (under written
//! instrument) and § 521 (not under written instrument).
//!
//! **Default — common-law (typically 20 years)**. Most US states
//! that do not have a specific shorter statute follow a common-law
//! 20-year continuous-possession rule with the five elements
//! (actual, open, hostile, exclusive, continuous). State-specific
//! periods range from 5 (CA) to 30 (NJ for woodlands).
//!
//! Citations: Cal. Civ. Proc. Code § 325 (CA 5-year + taxes);
//! Gilardi v. Hallam, 30 Cal. 3d 317 (1981) (assessed-year tax
//! payment rule); Tex. Civ. Prac. & Rem. Code §§ 16.024
//! (3-year color of title), 16.025 (5-year recorded deed + taxes),
//! 16.026 (10-year cultivation), 16.027 (25-year disability),
//! 16.028 (25-year void deed); Fla. Stat. § 95.16 (FL 7-year
//! with color of title); § 95.18 (FL 7-year without color of
//! title + appraiser return + 1-year tax cure); N.Y. RPAPL §
//! 501(3) (claim of right reasonable-basis definition); § 511
//! (under written instrument); § 521 (not under written
//! instrument); N.Y. CPLR § 212(a) (10-year limitations
//! period); Walling v. Prysbylo, 7 N.Y.3d 228 (2006) (overruled
//! by July 8, 2008 amendment).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    Texas,
    Florida,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TexasPath {
    /// § 16.024 — 3 years with color of title.
    ThreeYearColorOfTitle,
    /// § 16.025 — 5 years with recorded deed + cultivation + taxes.
    FiveYearRecordedDeed,
    /// § 16.026 — 10 years cultivation/use; no color, no taxes.
    TenYearCultivation,
    /// § 16.027 or § 16.028 — 25 years.
    TwentyFiveYear,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AdversePossessionInput {
    pub regime: Regime,
    pub years_of_possession: u32,
    pub paid_taxes_for_full_period: bool,
    /// Five common-law elements (actual + open and notorious +
    /// hostile + exclusive + continuous). All required for any
    /// claim to succeed regardless of regime.
    pub five_elements_satisfied: bool,
    /// Texas-only: which Ch. 16 subsection the claimant invokes.
    pub tx_path: Option<TexasPath>,
    /// Florida § 95.18 only: filed return with property appraiser
    /// within 30 days of paying taxes.
    pub fl_filed_return_with_appraiser: bool,
    /// Florida § 95.18 only: paid all outstanding taxes within 1
    /// year of entering possession (claim cure-window).
    pub fl_paid_taxes_within_one_year_of_entry: bool,
    /// New York post-2008: claim of right with reasonable basis
    /// under RPAPL § 501(3). Bad-faith possessor who KNOWS they
    /// do not own cannot acquire title.
    pub ny_reasonable_basis_for_claim_of_right: bool,
    /// Indicates whether the FL claim is under color of title
    /// (§ 95.16) or without (§ 95.18). Color of title cuts the
    /// appraiser-return + 1-year-tax-cure requirements.
    pub fl_has_color_of_title: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct AdversePossessionResult {
    pub claim_satisfied: bool,
    pub required_period_years: u32,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &AdversePossessionInput) -> AdversePossessionResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if !input.five_elements_satisfied {
        violations.push(
            "five common-law elements not satisfied — actual + open and notorious + hostile + exclusive + continuous all required regardless of regime"
                .to_string(),
        );
    }

    let required_period: u32 = match input.regime {
        Regime::California => check_california(input, &mut violations, &mut notes),
        Regime::Texas => check_texas(input, &mut violations, &mut notes),
        Regime::Florida => check_florida(input, &mut violations, &mut notes),
        Regime::NewYork => check_new_york(input, &mut violations, &mut notes),
        Regime::Default => check_default(input, &mut violations, &mut notes),
    };

    if input.years_of_possession < required_period {
        violations.push(format!(
            "possession {} years short of required {} years",
            input.years_of_possession, required_period
        ));
    }

    AdversePossessionResult {
        claim_satisfied: violations.is_empty(),
        required_period_years: required_period,
        violations: violations.clone(),
        citation: citation_for(input.regime),
        notes: notes.clone(),
    }
}

fn check_california(
    input: &AdversePossessionInput,
    violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> u32 {
    if !input.paid_taxes_for_full_period {
        violations.push(
            "California § 325 requires payment of state, county, and municipal taxes throughout the entire 5-year period — Gilardi v. Hallam payment must be in assessment year, not back-taxes"
                .to_string(),
        );
    }
    notes.push(
        "Gilardi v. Hallam (1981) — tax payment must occur in year of assessment; paying back taxes does not satisfy"
            .to_string(),
    );
    5
}

fn check_texas(
    input: &AdversePossessionInput,
    violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> u32 {
    let path = match input.tx_path {
        Some(p) => p,
        None => {
            violations.push(
                "Texas claim must specify Ch. 16 subsection path (3/5/10/25-year)".to_string(),
            );
            return 25;
        }
    };

    match path {
        TexasPath::ThreeYearColorOfTitle => {
            notes.push(
                "§ 16.024 — color of title (instrument purporting to convey valid title regardless of recording)"
                    .to_string(),
            );
            3
        }
        TexasPath::FiveYearRecordedDeed => {
            if !input.paid_taxes_for_full_period {
                violations.push(
                    "§ 16.025 requires tax payment throughout the 5-year period".to_string(),
                );
            }
            notes.push(
                "§ 16.025 — recorded deed + cultivation/use + tax payment + 5 years required"
                    .to_string(),
            );
            5
        }
        TexasPath::TenYearCultivation => {
            notes.push(
                "§ 16.026 — peaceable + adverse + cultivation/use; NO tax payment, NO color of title required"
                    .to_string(),
            );
            10
        }
        TexasPath::TwentyFiveYear => {
            notes.push(
                "§ 16.027 (regardless of disability) or § 16.028 (recorded but void deed)"
                    .to_string(),
            );
            25
        }
    }
}

fn check_florida(
    input: &AdversePossessionInput,
    violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> u32 {
    if !input.paid_taxes_for_full_period {
        violations.push(
            "Florida § 95.16 / § 95.18 require ongoing tax payment for all years — failure defeats claim"
                .to_string(),
        );
    }
    if !input.fl_has_color_of_title {
        if !input.fl_paid_taxes_within_one_year_of_entry {
            violations.push(
                "Florida § 95.18 (without color of title) requires payment of all outstanding taxes and matured special improvement liens WITHIN 1 YEAR of entering possession"
                    .to_string(),
            );
        }
        if !input.fl_filed_return_with_appraiser {
            violations.push(
                "Florida § 95.18 (without color of title) requires filing a return with the property appraiser within 30 days after tax payment"
                    .to_string(),
            );
        }
        notes.push(
            "§ 95.18 without color of title — strictest US adverse-possession regime due to appraiser-return + 1-year tax-cure requirements"
                .to_string(),
        );
    } else {
        notes.push("§ 95.16 — color of title cuts the appraiser-return and 1-year-tax-cure burdens".to_string());
    }
    7
}

fn check_new_york(
    input: &AdversePossessionInput,
    violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> u32 {
    if !input.ny_reasonable_basis_for_claim_of_right {
        violations.push(
            "NY RPAPL § 501(3) (post-July 8, 2008) requires claim of right with reasonable basis for belief the property belongs to claimant — bad-faith possessor who knows they do not own cannot acquire title"
                .to_string(),
        );
    }
    notes.push(
        "2008 amendment overruled Walling v. Prysbylo — applies to both RPAPL § 511 (under written instrument) and § 521 (not under written instrument)"
            .to_string(),
    );
    10
}

fn check_default(
    _input: &AdversePossessionInput,
    _violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> u32 {
    notes.push(
        "default common-law rule — most states 15-30 years continuous possession with five common-law elements; state-specific period required to be verified"
            .to_string(),
    );
    20
}

fn citation_for(regime: Regime) -> &'static str {
    match regime {
        Regime::California => "Cal. Civ. Proc. Code § 325; Gilardi v. Hallam, 30 Cal. 3d 317 (1981)",
        Regime::Texas => {
            "Tex. Civ. Prac. & Rem. Code §§ 16.024, 16.025, 16.026, 16.027, 16.028"
        }
        Regime::Florida => "Fla. Stat. §§ 95.16, 95.18",
        Regime::NewYork => "N.Y. RPAPL §§ 501(3), 511, 521; N.Y. CPLR § 212(a); Walling v. Prysbylo, 7 N.Y.3d 228 (2006) overruled by July 8, 2008 amendment",
        Regime::Default => "common-law adverse possession (state-specific period typically 15-30 years)",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_full_satisfied() -> AdversePossessionInput {
        AdversePossessionInput {
            regime: Regime::California,
            years_of_possession: 5,
            paid_taxes_for_full_period: true,
            five_elements_satisfied: true,
            tx_path: None,
            fl_filed_return_with_appraiser: false,
            fl_paid_taxes_within_one_year_of_entry: false,
            ny_reasonable_basis_for_claim_of_right: false,
            fl_has_color_of_title: false,
        }
    }

    fn tx_base() -> AdversePossessionInput {
        AdversePossessionInput {
            regime: Regime::Texas,
            years_of_possession: 3,
            paid_taxes_for_full_period: false,
            five_elements_satisfied: true,
            tx_path: Some(TexasPath::ThreeYearColorOfTitle),
            fl_filed_return_with_appraiser: false,
            fl_paid_taxes_within_one_year_of_entry: false,
            ny_reasonable_basis_for_claim_of_right: false,
            fl_has_color_of_title: false,
        }
    }

    fn fl_with_color_full() -> AdversePossessionInput {
        AdversePossessionInput {
            regime: Regime::Florida,
            years_of_possession: 7,
            paid_taxes_for_full_period: true,
            five_elements_satisfied: true,
            tx_path: None,
            fl_filed_return_with_appraiser: false,
            fl_paid_taxes_within_one_year_of_entry: false,
            ny_reasonable_basis_for_claim_of_right: false,
            fl_has_color_of_title: true,
        }
    }

    fn fl_without_color_full() -> AdversePossessionInput {
        AdversePossessionInput {
            regime: Regime::Florida,
            years_of_possession: 7,
            paid_taxes_for_full_period: true,
            five_elements_satisfied: true,
            tx_path: None,
            fl_filed_return_with_appraiser: true,
            fl_paid_taxes_within_one_year_of_entry: true,
            ny_reasonable_basis_for_claim_of_right: false,
            fl_has_color_of_title: false,
        }
    }

    fn ny_full() -> AdversePossessionInput {
        AdversePossessionInput {
            regime: Regime::NewYork,
            years_of_possession: 10,
            paid_taxes_for_full_period: false,
            five_elements_satisfied: true,
            tx_path: None,
            fl_filed_return_with_appraiser: false,
            fl_paid_taxes_within_one_year_of_entry: false,
            ny_reasonable_basis_for_claim_of_right: true,
            fl_has_color_of_title: false,
        }
    }

    #[test]
    fn ca_5_years_with_taxes_and_five_elements_claim_satisfied() {
        let r = check(&ca_full_satisfied());
        assert!(r.claim_satisfied);
        assert_eq!(r.required_period_years, 5);
    }

    #[test]
    fn ca_5_years_without_taxes_fails() {
        let mut i = ca_full_satisfied();
        i.paid_taxes_for_full_period = false;
        let r = check(&i);
        assert!(!r.claim_satisfied);
        assert!(r.violations.iter().any(|v| v.contains("§ 325")));
    }

    #[test]
    fn ca_4_years_with_taxes_fails_period() {
        let mut i = ca_full_satisfied();
        i.years_of_possession = 4;
        let r = check(&i);
        assert!(!r.claim_satisfied);
        assert!(r.violations.iter().any(|v| v.contains("short of required 5 years")));
    }

    #[test]
    fn ca_notes_gilardi_assessed_year_rule() {
        let r = check(&ca_full_satisfied());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Gilardi v. Hallam")));
    }

    #[test]
    fn tx_3_year_color_of_title_satisfied() {
        let r = check(&tx_base());
        assert!(r.claim_satisfied);
        assert_eq!(r.required_period_years, 3);
    }

    #[test]
    fn tx_5_year_recorded_deed_requires_tax_payment() {
        let mut i = tx_base();
        i.years_of_possession = 5;
        i.tx_path = Some(TexasPath::FiveYearRecordedDeed);
        i.paid_taxes_for_full_period = false;
        let r = check(&i);
        assert!(!r.claim_satisfied);
        assert!(r.violations.iter().any(|v| v.contains("§ 16.025")));
    }

    #[test]
    fn tx_5_year_recorded_deed_with_tax_payment_satisfied() {
        let mut i = tx_base();
        i.years_of_possession = 5;
        i.tx_path = Some(TexasPath::FiveYearRecordedDeed);
        i.paid_taxes_for_full_period = true;
        let r = check(&i);
        assert!(r.claim_satisfied);
        assert_eq!(r.required_period_years, 5);
    }

    #[test]
    fn tx_10_year_cultivation_no_taxes_required() {
        let mut i = tx_base();
        i.years_of_possession = 10;
        i.tx_path = Some(TexasPath::TenYearCultivation);
        i.paid_taxes_for_full_period = false;
        let r = check(&i);
        assert!(r.claim_satisfied);
        assert_eq!(r.required_period_years, 10);
    }

    #[test]
    fn tx_25_year_satisfied_regardless_of_disability() {
        let mut i = tx_base();
        i.years_of_possession = 25;
        i.tx_path = Some(TexasPath::TwentyFiveYear);
        let r = check(&i);
        assert!(r.claim_satisfied);
        assert_eq!(r.required_period_years, 25);
    }

    #[test]
    fn tx_no_path_specified_violation() {
        let mut i = tx_base();
        i.tx_path = None;
        let r = check(&i);
        assert!(!r.claim_satisfied);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Ch. 16 subsection path")));
    }

    #[test]
    fn fl_color_of_title_7_years_with_taxes_satisfied() {
        let r = check(&fl_with_color_full());
        assert!(r.claim_satisfied);
        assert_eq!(r.required_period_years, 7);
    }

    #[test]
    fn fl_without_color_full_compliance_satisfied() {
        let r = check(&fl_without_color_full());
        assert!(r.claim_satisfied);
    }

    #[test]
    fn fl_without_color_missing_appraiser_return_fails() {
        let mut i = fl_without_color_full();
        i.fl_filed_return_with_appraiser = false;
        let r = check(&i);
        assert!(!r.claim_satisfied);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("property appraiser")));
    }

    #[test]
    fn fl_without_color_missing_one_year_tax_cure_fails() {
        let mut i = fl_without_color_full();
        i.fl_paid_taxes_within_one_year_of_entry = false;
        let r = check(&i);
        assert!(!r.claim_satisfied);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("WITHIN 1 YEAR")));
    }

    #[test]
    fn fl_with_color_no_appraiser_or_one_year_cure_needed() {
        let r = check(&fl_with_color_full());
        assert!(r.claim_satisfied);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 95.16")));
    }

    #[test]
    fn fl_no_tax_payment_fails_in_both_paths() {
        for has_color in [true, false] {
            let mut i = if has_color { fl_with_color_full() } else { fl_without_color_full() };
            i.paid_taxes_for_full_period = false;
            let r = check(&i);
            assert!(!r.claim_satisfied);
        }
    }

    #[test]
    fn ny_10_years_with_claim_of_right_satisfied() {
        let r = check(&ny_full());
        assert!(r.claim_satisfied);
        assert_eq!(r.required_period_years, 10);
    }

    #[test]
    fn ny_without_reasonable_basis_fails() {
        let mut i = ny_full();
        i.ny_reasonable_basis_for_claim_of_right = false;
        let r = check(&i);
        assert!(!r.claim_satisfied);
        assert!(r.violations.iter().any(|v| v.contains("§ 501(3)")));
    }

    #[test]
    fn ny_notes_walling_overruled() {
        let r = check(&ny_full());
        assert!(r.notes.iter().any(|n| n.contains("Walling v. Prysbylo")));
    }

    #[test]
    fn five_elements_missing_universal_violation() {
        let mut i = ca_full_satisfied();
        i.five_elements_satisfied = false;
        let r = check(&i);
        assert!(!r.claim_satisfied);
        assert!(r.violations.iter().any(|v| v.contains("five common-law elements")));
    }

    #[test]
    fn default_regime_returns_20_year_period() {
        let i = AdversePossessionInput {
            regime: Regime::Default,
            years_of_possession: 20,
            paid_taxes_for_full_period: true,
            five_elements_satisfied: true,
            tx_path: None,
            fl_filed_return_with_appraiser: false,
            fl_paid_taxes_within_one_year_of_entry: false,
            ny_reasonable_basis_for_claim_of_right: false,
            fl_has_color_of_title: false,
        };
        let r = check(&i);
        assert!(r.claim_satisfied);
        assert_eq!(r.required_period_years, 20);
    }

    #[test]
    fn default_regime_19_years_fails_period() {
        let i = AdversePossessionInput {
            regime: Regime::Default,
            years_of_possession: 19,
            paid_taxes_for_full_period: true,
            five_elements_satisfied: true,
            tx_path: None,
            fl_filed_return_with_appraiser: false,
            fl_paid_taxes_within_one_year_of_entry: false,
            ny_reasonable_basis_for_claim_of_right: false,
            fl_has_color_of_title: false,
        };
        let r = check(&i);
        assert!(!r.claim_satisfied);
    }

    #[test]
    fn citation_california_pins_section_and_gilardi() {
        let r = check(&ca_full_satisfied());
        assert!(r.citation.contains("§ 325"));
        assert!(r.citation.contains("Gilardi v. Hallam"));
        assert!(r.citation.contains("1981"));
    }

    #[test]
    fn citation_texas_pins_all_five_subsections() {
        let r = check(&tx_base());
        assert!(r.citation.contains("16.024"));
        assert!(r.citation.contains("16.025"));
        assert!(r.citation.contains("16.026"));
        assert!(r.citation.contains("16.027"));
        assert!(r.citation.contains("16.028"));
    }

    #[test]
    fn citation_florida_pins_both_sections() {
        let r = check(&fl_with_color_full());
        assert!(r.citation.contains("95.16"));
        assert!(r.citation.contains("95.18"));
    }

    #[test]
    fn citation_newyork_pins_walling_overruled() {
        let r = check(&ny_full());
        assert!(r.citation.contains("§ 501(3)"));
        assert!(r.citation.contains("CPLR § 212(a)"));
        assert!(r.citation.contains("Walling"));
        assert!(r.citation.contains("July 8, 2008"));
    }

    #[test]
    fn shortest_required_period_california_5_years_invariant() {
        let r_ca = check(&ca_full_satisfied());
        let r_tx_10 = check(&AdversePossessionInput {
            tx_path: Some(TexasPath::TenYearCultivation),
            regime: Regime::Texas,
            years_of_possession: 10,
            paid_taxes_for_full_period: false,
            five_elements_satisfied: true,
            fl_filed_return_with_appraiser: false,
            fl_paid_taxes_within_one_year_of_entry: false,
            ny_reasonable_basis_for_claim_of_right: false,
            fl_has_color_of_title: false,
        });
        let r_fl = check(&fl_with_color_full());
        let r_ny = check(&ny_full());
        assert!(r_ca.required_period_years < r_fl.required_period_years);
        assert!(r_ca.required_period_years < r_ny.required_period_years);
        assert!(r_ca.required_period_years < r_tx_10.required_period_years);
    }

    #[test]
    fn tx_three_year_is_shortest_subsection_invariant() {
        let r_tx_3 = check(&tx_base());
        let mut i_5 = tx_base();
        i_5.years_of_possession = 5;
        i_5.tx_path = Some(TexasPath::FiveYearRecordedDeed);
        i_5.paid_taxes_for_full_period = true;
        let r_tx_5 = check(&i_5);
        assert!(r_tx_3.required_period_years < r_tx_5.required_period_years);
    }
}
