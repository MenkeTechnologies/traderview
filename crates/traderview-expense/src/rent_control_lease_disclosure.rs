//! Mandatory landlord disclosure of rent control / rent stabilization
//! status at lease execution. Trader-landlord operational concern
//! when leasing units in jurisdictions with statewide rent caps or
//! rent stabilization regimes — the disclosure is a SEPARATE
//! requirement from the substantive rent limit and carries its own
//! compliance regime.
//!
//! Distinct from `rent_control` (substantive rent-cap mechanics),
//! `rent_increase_notice_period` (advance notice before raising
//! rent), and `lease_disclosures` (general lease-required
//! disclosures). This module addresses ONLY the STATUS-DISCLOSURE
//! AT LEASE EXECUTION pathway — at the moment of tenancy formation,
//! must the landlord affirmatively inform the tenant that the unit
//! is or is not subject to rent control?
//!
//! Four regimes:
//!
//! **California — Cal. Civ. Code § 1947.12 + § 1946.2 (AB 1482
//! Tenant Protection Act of 2019)**. EXPRESS ADDENDUM REQUIRED for
//! any tenancy commenced or renewed on or after July 1, 2020. The
//! notice must be provided AS AN ADDENDUM to the lease or as a
//! written notice SIGNED by the tenant with a copy provided. § 1947.12
//! (d)(5) requires the notice be in NO LESS THAN 12-POINT TYPE. Two
//! distinct statutory texts apply:
//!   - COVERED property text: "California law limits the amount your
//!     rent can be increased. See Section 1947.12 of the Civil Code
//!     for more information. California law also provides that after
//!     all of the tenants have continuously and lawfully occupied the
//!     property for 12 months or more or at least one of the tenants
//!     has continuously and lawfully occupied the property for 24
//!     months or more, a landlord must provide a statement of cause
//!     in any notice to terminate a tenancy. See Section 1946.2 of
//!     the Civil Code for more information."
//!   - EXEMPT property text: "This property is not subject to the
//!     rent limits imposed by Section 1947.12 of the Civil Code and
//!     is not subject to the just cause requirements of Section
//!     1946.2 of the Civil Code. This property meets the requirements
//!     of Sections 1947.12(d)(5) and 1946.2(e)(8) of the Civil Code
//!     and the owner is not any of the following: (1) a real estate
//!     investment trust, as defined by Section 856 of the Internal
//!     Revenue Code; (2) a corporation; or (3) a limited liability
//!     company in which at least one member is a corporation."
//!
//! **Oregon — ORS 90.323 (SB 608 of 2019)**. NO ADDENDUM REQUIRED at
//! lease execution. Oregon's 7% + CPI cap operates through the rent-
//! increase notice statute — landlord must include the new rent
//! amount, effective date, and landlord contact information in the
//! rent-increase notice itself (90-day for ≤ 10%, 180-day for > 10%).
//! Exemption from cap: < 15 years old, first tenancy after vacancy,
//! government-subsidized, landlord-shared unit.
//!
//! **New York — N.Y. Real Prop. Law § 234 + Rent Stabilization Code
//! § 2522.5(c)**. RSC-REQUIRED NOTICE for rent-stabilized units. The
//! rent stabilization lease rider (RSC § 2522.5(c)) must be attached
//! to every initial lease and renewal lease for a rent-stabilized
//! unit, identifying the unit's stabilized status, registered legal
//! rent, and tenant rights. Failure renders the lease unenforceable
//! as to any provision conflicting with stabilization law.
//!
//! **Default — no statewide disclosure obligation**. Most states
//! lack statewide rent control; lease execution does not require
//! status disclosure. Municipal rent control jurisdictions (e.g.,
//! Berkeley, Santa Monica, NJ municipalities) may impose their own
//! local disclosure rules.
//!
//! Citations: Cal. Civ. Code § 1947.12 (CA rent cap + addendum
//! requirement); § 1947.12(d)(5) (12-point font + signed notice
//! format); § 1946.2 (just cause termination disclosure paired with
//! cap); § 1946.2(e)(8) (single-family / condo exemption notice);
//! Or. Rev. Stat. § 90.323 (OR 7% + CPI cap); ORS 90.323(3) (90-day
//! / 180-day rent-increase notice); N.Y. Real Prop. Law § 234
//! (warranty of habitability + attorney fees parity); N.Y. Comp.
//! Codes R. & Regs. tit. 9 § 2522.5(c) (RSC rent stabilization
//! lease rider).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    Oregon,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentControlDisclosureInput {
    pub regime: Regime,
    /// California-only — § 1947.12(d) addendum required for any
    /// tenancy commenced or renewed on or after July 1, 2020.
    pub tenancy_post_july_1_2020: bool,
    /// Whether the property is EXEMPT from the rent cap (single-
    /// family / condo where owner is not REIT/corp/LLC-with-corp,
    /// new construction < 15 years, etc.). Different disclosure
    /// text required for exempt vs covered properties.
    pub property_exempt_from_rent_cap: bool,
    /// Whether the landlord provided the § 1947.12 addendum (or
    /// signed written notice) at lease execution.
    pub addendum_or_signed_notice_provided: bool,
    /// CA-specific — whether the disclosure text is in 12-point
    /// font or larger.
    pub font_size_at_least_12_point: bool,
    /// CA-only — for exempt properties, whether the disclosure
    /// includes the specific § 1947.12(d)(5) exempt-property
    /// language identifying that owner is not REIT/corp/LLC-with-
    /// corp.
    pub exempt_property_disclosure_includes_required_language: bool,
    /// NY-only — whether the unit is rent-stabilized under RSC.
    pub ny_rent_stabilized_unit: bool,
    /// NY-only — whether the RSC § 2522.5(c) rent stabilization
    /// lease rider is attached.
    pub ny_rent_stabilization_rider_attached: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentControlDisclosureResult {
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &RentControlDisclosureInput) -> RentControlDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    match input.regime {
        Regime::California => check_california(input, &mut violations, &mut notes),
        Regime::Oregon => check_oregon(input, &mut violations, &mut notes),
        Regime::NewYork => check_new_york(input, &mut violations, &mut notes),
        Regime::Default => check_default(input, &mut violations, &mut notes),
    }
}

fn check_california(
    input: &RentControlDisclosureInput,
    violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> RentControlDisclosureResult {
    if !input.tenancy_post_july_1_2020 {
        notes.push(
            "Cal. Civ. Code § 1947.12(d) — addendum requirement applies only to tenancies commenced or renewed on or after July 1, 2020; pre-July 2020 tenancies grandfathered"
                .to_string(),
        );
        return RentControlDisclosureResult {
            compliant: true,
            violations: violations.clone(),
            citation: citation_for(Regime::California),
            notes: notes.clone(),
        };
    }

    if !input.addendum_or_signed_notice_provided {
        violations.push(
            "Cal. Civ. Code § 1947.12(d) — written addendum or signed notice required at lease execution for tenancies post-July 1, 2020"
                .to_string(),
        );
    }

    if input.addendum_or_signed_notice_provided && !input.font_size_at_least_12_point {
        violations.push(
            "Cal. Civ. Code § 1947.12(d)(5) — disclosure notice must be in no less than 12-point type"
                .to_string(),
        );
    }

    if input.property_exempt_from_rent_cap
        && !input.exempt_property_disclosure_includes_required_language
    {
        violations.push(
            "Cal. Civ. Code § 1947.12(d)(5) + § 1946.2(e)(8) — exempt-property disclosure must identify that owner is NOT a REIT (§ 856 IRC), corporation, or LLC with a corporate member"
                .to_string(),
        );
    }

    if input.property_exempt_from_rent_cap {
        notes.push(
            "exempt property — disclosure text differs from covered-property text and must affirm § 1947.12(d)(5) ownership requirements"
                .to_string(),
        );
    } else {
        notes.push(
            "covered property — disclosure text must reference § 1947.12 rent limits AND § 1946.2 just-cause termination requirements after 12/24-month occupancy"
                .to_string(),
        );
    }

    RentControlDisclosureResult {
        compliant: violations.is_empty(),
        violations: violations.clone(),
        citation: citation_for(Regime::California),
        notes: notes.clone(),
    }
}

fn check_oregon(
    _input: &RentControlDisclosureInput,
    _violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> RentControlDisclosureResult {
    notes.push(
        "Or. Rev. Stat. § 90.323 — NO ADDENDUM required at lease execution; SB 608 7% + CPI cap operates through rent-increase notice statute (ORS 90.323(3))"
            .to_string(),
    );
    notes.push(
        "rent-increase notice must include new rent amount + effective date + landlord contact; 90 days for ≤ 10% increase, 180 days for > 10%"
            .to_string(),
    );
    notes.push(
        "ORS 90.323 cap exemptions: < 15 years old, first tenancy after vacancy, government-subsidized, landlord-shared unit"
            .to_string(),
    );
    RentControlDisclosureResult {
        compliant: true,
        violations: Vec::new(),
        citation: citation_for(Regime::Oregon),
        notes: notes.clone(),
    }
}

fn check_new_york(
    input: &RentControlDisclosureInput,
    violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> RentControlDisclosureResult {
    if !input.ny_rent_stabilized_unit {
        notes.push(
            "unit not rent-stabilized — RSC § 2522.5(c) rider requirement does not engage; general lease disclosures still apply"
                .to_string(),
        );
        return RentControlDisclosureResult {
            compliant: true,
            violations: violations.clone(),
            citation: citation_for(Regime::NewYork),
            notes: notes.clone(),
        };
    }

    if !input.ny_rent_stabilization_rider_attached {
        violations.push(
            "N.Y. Comp. Codes R. & Regs. tit. 9 § 2522.5(c) — rent stabilization lease rider must be attached to every initial lease and renewal lease; failure renders the lease unenforceable as to any provision conflicting with stabilization law"
                .to_string(),
        );
    } else {
        notes.push(
            "RSC § 2522.5(c) — rent stabilization rider attached; rider must identify unit's stabilized status, registered legal rent, and tenant rights"
                .to_string(),
        );
    }

    RentControlDisclosureResult {
        compliant: violations.is_empty(),
        violations: violations.clone(),
        citation: citation_for(Regime::NewYork),
        notes: notes.clone(),
    }
}

fn check_default(
    _input: &RentControlDisclosureInput,
    _violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> RentControlDisclosureResult {
    notes.push(
        "default rule — no statewide rent control disclosure obligation; municipal rent control jurisdictions (Berkeley, Santa Monica, NJ municipalities) may impose local disclosure rules"
            .to_string(),
    );
    RentControlDisclosureResult {
        compliant: true,
        violations: Vec::new(),
        citation: citation_for(Regime::Default),
        notes: notes.clone(),
    }
}

fn citation_for(regime: Regime) -> &'static str {
    match regime {
        Regime::California => {
            "Cal. Civ. Code §§ 1947.12, 1947.12(d)(5), 1946.2, 1946.2(e)(8); AB 1482 (2019)"
        }
        Regime::Oregon => "Or. Rev. Stat. § 90.323; SB 608 (2019)",
        Regime::NewYork => "N.Y. Real Prop. Law § 234; N.Y. Comp. Codes R. & Regs. tit. 9 § 2522.5(c)",
        Regime::Default => "no statewide rent-control disclosure statute; municipal control may apply",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_base() -> RentControlDisclosureInput {
        RentControlDisclosureInput {
            regime: Regime::California,
            tenancy_post_july_1_2020: true,
            property_exempt_from_rent_cap: false,
            addendum_or_signed_notice_provided: true,
            font_size_at_least_12_point: true,
            exempt_property_disclosure_includes_required_language: false,
            ny_rent_stabilized_unit: false,
            ny_rent_stabilization_rider_attached: false,
        }
    }

    fn or_base() -> RentControlDisclosureInput {
        RentControlDisclosureInput {
            regime: Regime::Oregon,
            tenancy_post_july_1_2020: true,
            property_exempt_from_rent_cap: false,
            addendum_or_signed_notice_provided: false,
            font_size_at_least_12_point: false,
            exempt_property_disclosure_includes_required_language: false,
            ny_rent_stabilized_unit: false,
            ny_rent_stabilization_rider_attached: false,
        }
    }

    fn ny_base() -> RentControlDisclosureInput {
        RentControlDisclosureInput {
            regime: Regime::NewYork,
            tenancy_post_july_1_2020: false,
            property_exempt_from_rent_cap: false,
            addendum_or_signed_notice_provided: false,
            font_size_at_least_12_point: false,
            exempt_property_disclosure_includes_required_language: false,
            ny_rent_stabilized_unit: true,
            ny_rent_stabilization_rider_attached: true,
        }
    }

    fn default_base() -> RentControlDisclosureInput {
        RentControlDisclosureInput {
            regime: Regime::Default,
            tenancy_post_july_1_2020: false,
            property_exempt_from_rent_cap: false,
            addendum_or_signed_notice_provided: false,
            font_size_at_least_12_point: false,
            exempt_property_disclosure_includes_required_language: false,
            ny_rent_stabilized_unit: false,
            ny_rent_stabilization_rider_attached: false,
        }
    }

    #[test]
    fn ca_post_july_2020_covered_property_full_compliance() {
        let r = check(&ca_base());
        assert!(r.compliant);
        assert!(r.notes.iter().any(|n| n.contains("§ 1947.12") && n.contains("§ 1946.2")));
    }

    #[test]
    fn ca_pre_july_2020_tenancy_grandfathered_no_disclosure_required() {
        let mut i = ca_base();
        i.tenancy_post_july_1_2020 = false;
        i.addendum_or_signed_notice_provided = false;
        let r = check(&i);
        assert!(r.compliant);
        assert!(r.notes.iter().any(|n| n.contains("grandfathered")));
    }

    #[test]
    fn ca_missing_addendum_violation() {
        let mut i = ca_base();
        i.addendum_or_signed_notice_provided = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 1947.12(d)") && v.contains("addendum")));
    }

    #[test]
    fn ca_font_below_12_point_violation() {
        let mut i = ca_base();
        i.font_size_at_least_12_point = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 1947.12(d)(5)") && v.contains("12-point")));
    }

    #[test]
    fn ca_exempt_property_missing_ownership_language_violation() {
        let mut i = ca_base();
        i.property_exempt_from_rent_cap = true;
        i.exempt_property_disclosure_includes_required_language = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("REIT") && v.contains("corporation")));
    }

    #[test]
    fn ca_exempt_property_with_correct_ownership_language_compliant() {
        let mut i = ca_base();
        i.property_exempt_from_rent_cap = true;
        i.exempt_property_disclosure_includes_required_language = true;
        let r = check(&i);
        assert!(r.compliant);
        assert!(r.notes.iter().any(|n| n.contains("exempt property") && n.contains("§ 1947.12(d)(5)")));
    }

    #[test]
    fn ca_covered_property_note_references_just_cause() {
        let r = check(&ca_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 1946.2 just-cause") && n.contains("12/24-month")));
    }

    #[test]
    fn or_no_addendum_required_compliant_default() {
        let r = check(&or_base());
        assert!(r.compliant);
        assert!(r.notes.iter().any(|n| n.contains("NO ADDENDUM required")));
    }

    #[test]
    fn or_notes_describe_rent_increase_notice_framework() {
        let r = check(&or_base());
        assert!(r.notes.iter().any(|n| n.contains("90 days") && n.contains("180 days")));
    }

    #[test]
    fn or_notes_describe_cap_exemptions() {
        let r = check(&or_base());
        assert!(r.notes.iter().any(|n| n.contains("< 15 years old") && n.contains("government-subsidized")));
    }

    #[test]
    fn ny_rent_stabilized_with_rider_compliant() {
        let r = check(&ny_base());
        assert!(r.compliant);
        assert!(r.notes.iter().any(|n| n.contains("§ 2522.5(c)") && n.contains("registered legal rent")));
    }

    #[test]
    fn ny_rent_stabilized_missing_rider_violation() {
        let mut i = ny_base();
        i.ny_rent_stabilization_rider_attached = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("§ 2522.5(c)") && v.contains("unenforceable")));
    }

    #[test]
    fn ny_non_stabilized_unit_no_rider_required() {
        let mut i = ny_base();
        i.ny_rent_stabilized_unit = false;
        i.ny_rent_stabilization_rider_attached = false;
        let r = check(&i);
        assert!(r.compliant);
        assert!(r.notes.iter().any(|n| n.contains("not rent-stabilized")));
    }

    #[test]
    fn default_no_obligation_compliant() {
        let r = check(&default_base());
        assert!(r.compliant);
        assert!(r.notes.iter().any(|n| n.contains("no statewide") && n.contains("municipal")));
    }

    #[test]
    fn citation_california_pins_subsections_and_ab_1482() {
        let r = check(&ca_base());
        assert!(r.citation.contains("§§ 1947.12, 1947.12(d)(5), 1946.2, 1946.2(e)(8)"));
        assert!(r.citation.contains("AB 1482"));
    }

    #[test]
    fn citation_oregon_pins_90_323_and_sb_608() {
        let r = check(&or_base());
        assert!(r.citation.contains("§ 90.323"));
        assert!(r.citation.contains("SB 608"));
    }

    #[test]
    fn citation_newyork_pins_section_234_and_rsc_2522_5() {
        let r = check(&ny_base());
        assert!(r.citation.contains("§ 234"));
        assert!(r.citation.contains("§ 2522.5(c)"));
    }

    #[test]
    fn citation_default_pins_no_statewide_statute() {
        let r = check(&default_base());
        assert!(r.citation.contains("no statewide"));
    }

    #[test]
    fn california_uniquely_requires_addendum_at_lease_execution_invariant() {
        let r_ca = check(&ca_base());
        let r_or = check(&or_base());
        let r_ny_non_stab = {
            let mut i = ny_base();
            i.ny_rent_stabilized_unit = false;
            check(&i)
        };
        let r_default = check(&default_base());
        let _ = (r_or.compliant, r_ny_non_stab.compliant, r_default.compliant);
        let mut i_ca_no_addendum = ca_base();
        i_ca_no_addendum.addendum_or_signed_notice_provided = false;
        let r_ca_violation = check(&i_ca_no_addendum);
        assert!(r_ca.compliant);
        assert!(!r_ca_violation.compliant);
        for regime in [Regime::Oregon, Regime::Default] {
            let mut i = ca_base();
            i.regime = regime;
            i.addendum_or_signed_notice_provided = false;
            let r = check(&i);
            assert!(r.compliant, "regime {:?} does not require addendum at lease execution", regime);
        }
    }

    #[test]
    fn ca_multiple_violations_accumulate() {
        let mut i = ca_base();
        i.addendum_or_signed_notice_provided = false;
        i.property_exempt_from_rent_cap = true;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.len() >= 2);
    }

    #[test]
    fn ca_no_double_violation_when_addendum_missing() {
        let mut i = ca_base();
        i.addendum_or_signed_notice_provided = false;
        i.font_size_at_least_12_point = false;
        let r = check(&i);
        let font_violations: Vec<_> = r
            .violations
            .iter()
            .filter(|v| v.contains("12-point"))
            .collect();
        assert!(font_violations.is_empty(), "font check skipped when addendum not provided");
    }

    #[test]
    fn ny_lease_unenforceable_consequence_in_violation_message() {
        let mut i = ny_base();
        i.ny_rent_stabilization_rider_attached = false;
        let r = check(&i);
        assert!(r.violations.iter().any(|v| v.contains("unenforceable")));
    }

    #[test]
    fn or_compliance_invariant_regardless_of_lease_inputs() {
        let cases = [
            (false, false, false),
            (true, true, true),
            (false, true, false),
        ];
        for (addendum, font, exempt_lang) in cases {
            let mut i = or_base();
            i.addendum_or_signed_notice_provided = addendum;
            i.font_size_at_least_12_point = font;
            i.exempt_property_disclosure_includes_required_language = exempt_lang;
            let r = check(&i);
            assert!(r.compliant, "Oregon does not require addendum regardless of inputs");
        }
    }

    #[test]
    fn ca_exempt_property_note_appears_when_exempt() {
        let mut i = ca_base();
        i.property_exempt_from_rent_cap = true;
        i.exempt_property_disclosure_includes_required_language = true;
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("§ 1947.12(d)(5) ownership requirements")));
    }

    #[test]
    fn ny_uniquely_requires_rsc_rider_for_stabilized_units() {
        let r_ny = check(&ny_base());
        assert!(r_ny.compliant);
        for regime in [Regime::California, Regime::Oregon, Regime::Default] {
            let mut i = ny_base();
            i.regime = regime;
            i.ny_rent_stabilized_unit = true;
            i.ny_rent_stabilization_rider_attached = false;
            i.tenancy_post_july_1_2020 = false;
            let r = check(&i);
            assert!(r.compliant || matches!(regime, Regime::California), "regime {:?} does not impose RSC § 2522.5(c) requirement", regime);
        }
    }
}
