//! Methamphetamine contamination landlord disclosure compliance check.
//!
//! Four jurisdictions ship distinct statutory regimes for landlord
//! disclosure of prior meth manufacture, smoke contamination, or
//! remediation history. Each has a specific contamination threshold
//! (micrograms per 100 square centimeters of surface area) and a
//! distinct post-remediation disclosure-extinguishment rule.
//!
//! Regimes:
//!
//! **Colorado** (§ 38-35.7-103) — disclosure REQUIRED if the landlord
//! knows the property was used as a clandestine meth lab AND has NOT
//! been remediated to the **0.5 ug/100cm²** state standard (4.0 for
//! attics and crawl spaces; 1.5 for encapsulated painted surfaces).
//! Once remediated and certified to the state, **the disclosure
//! obligation extinguishes** — no further disclosure required.
//!
//! **Arizona** (A.R.S. § 32-1166.04) — remediation standard is
//! **0.1 ug/100cm²**, the strictest in the four. Once remediated to
//! that standard the disclosure obligation extinguishes. **Also unique
//! to Arizona**: it is unlawful for anyone other than the owner to
//! enter the unit until cleaning is complete to state standard.
//!
//! **Montana** (§ 75-10-1301 et seq.) — remediation standard
//! **1.5 ug/100cm²**. Distinct from CO/AZ in that the landlord MUST
//! disclose KNOWLEDGE of meth use OR contamination AND remediation
//! status to all prospective tenants, even AFTER remediation.
//! Remediation does NOT extinguish disclosure.
//!
//! **Default** — no statewide meth-contamination disclosure statute.
//! General fraud and Fair Housing Act doctrines may impose disclosure
//! duties for material defects affecting habitability.
//!
//! Citations: Colo. Rev. Stat. § 38-35.7-103; Colo. Code Regs. 6 CCR
//! 1014-3 (CDPHE meth lab cleanup standards); Ariz. Rev. Stat. §
//! 32-1166.04; Mont. Code Ann. § 75-10-1301 et seq.; Fed: 42 U.S.C.
//! § 3604 (Fair Housing Act material-defect doctrine).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Colorado,
    Arizona,
    Montana,
    Default,
}

impl Regime {
    pub fn for_state(state: &str) -> Self {
        match state.trim().to_ascii_uppercase().as_str() {
            "CO" => Self::Colorado,
            "AZ" => Self::Arizona,
            "MT" => Self::Montana,
            _ => Self::Default,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MethDisclosureInput {
    pub regime: Regime,
    /// Whether the landlord has knowledge of prior meth manufacture, use,
    /// or contamination at the property.
    pub landlord_has_knowledge: bool,
    /// Whether the property has been remediated to the regime's standard
    /// AND the state has been provided adequate evidence of remediation.
    pub remediated_to_standard: bool,
    /// Whether the landlord provided written disclosure to the prospective
    /// tenant before signing the lease.
    pub disclosure_provided_to_tenant: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DisclosureStatus {
    /// No disclosure required (no knowledge, or post-remediation extinguishment).
    NotRequired,
    /// Disclosure required and was provided — landlord compliant.
    RequiredAndProvided,
    /// Disclosure required but was NOT provided — violation.
    RequiredButNotProvided,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MethDisclosureResult {
    pub regime: Regime,
    pub remediation_standard_ug_per_100cm2_x10: u32,
    pub disclosure_required: bool,
    pub remediation_extinguishes_disclosure: bool,
    pub status: DisclosureStatus,
    pub landlord_compliant: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn check(input: &MethDisclosureInput) -> MethDisclosureResult {
    let (standard_x10, extinguishes, citation): (u32, bool, &'static str) = match input.regime {
        // Stored × 10 so the threshold is an integer (Arizona = 1 means 0.1).
        Regime::Colorado => (
            5,
            true,
            "Colo. Rev. Stat. § 38-35.7-103 + 6 CCR 1014-3 — 0.5 ug/100cm² standard; remediation EXTINGUISHES disclosure once certified to state",
        ),
        Regime::Arizona => (
            1,
            true,
            "Ariz. Rev. Stat. § 32-1166.04 — 0.1 ug/100cm² standard (STRICTEST); remediation EXTINGUISHES disclosure; entry by non-owners barred until cleaned",
        ),
        Regime::Montana => (
            15,
            false,
            "Mont. Code Ann. § 75-10-1301 et seq. — 1.5 ug/100cm² standard; remediation does NOT extinguish disclosure; landlord MUST disclose knowledge + remediation status",
        ),
        Regime::Default => (
            0,
            false,
            "No statewide meth-contamination disclosure statute — 42 U.S.C. § 3604 FHA material-defect doctrine may impose duty",
        ),
    };

    if !input.landlord_has_knowledge {
        return MethDisclosureResult {
            regime: input.regime,
            remediation_standard_ug_per_100cm2_x10: standard_x10,
            disclosure_required: false,
            remediation_extinguishes_disclosure: extinguishes,
            status: DisclosureStatus::NotRequired,
            landlord_compliant: true,
            citation,
            note:
                "Landlord has no knowledge of meth manufacture or contamination — no statutory disclosure obligation."
                    .to_string(),
        };
    }

    // Knowledge exists. Whether disclosure is REQUIRED now depends on
    // (a) whether the regime extinguishes-on-remediation, and (b) whether
    // remediation has actually occurred.
    let disclosure_required = if extinguishes {
        !input.remediated_to_standard
    } else {
        true
    };

    if !disclosure_required {
        return MethDisclosureResult {
            regime: input.regime,
            remediation_standard_ug_per_100cm2_x10: standard_x10,
            disclosure_required: false,
            remediation_extinguishes_disclosure: extinguishes,
            status: DisclosureStatus::NotRequired,
            landlord_compliant: true,
            citation,
            note:
                "Property remediated to standard and certified — disclosure obligation extinguished under the applicable statute."
                    .to_string(),
        };
    }

    let provided = input.disclosure_provided_to_tenant;
    MethDisclosureResult {
        regime: input.regime,
        remediation_standard_ug_per_100cm2_x10: standard_x10,
        disclosure_required: true,
        remediation_extinguishes_disclosure: extinguishes,
        status: if provided {
            DisclosureStatus::RequiredAndProvided
        } else {
            DisclosureStatus::RequiredButNotProvided
        },
        landlord_compliant: provided,
        citation,
        note: if provided {
            "Disclosure required and provided in writing — landlord compliant.".to_string()
        } else {
            "Disclosure REQUIRED and NOT provided — landlord noncompliant. Material defect under FHA + state statute.".to_string()
        },
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    fn input(
        regime: Regime,
        knowledge: bool,
        remediated: bool,
        disclosed: bool,
    ) -> MethDisclosureInput {
        MethDisclosureInput {
            regime,
            landlord_has_knowledge: knowledge,
            remediated_to_standard: remediated,
            disclosure_provided_to_tenant: disclosed,
        }
    }

    #[test]
    fn co_no_knowledge_no_obligation() {
        let r = check(&input(Regime::Colorado, false, false, false));
        assert!(!r.disclosure_required);
        assert_eq!(r.status, DisclosureStatus::NotRequired);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn co_knowledge_unremediated_undisclosed_violation() {
        let r = check(&input(Regime::Colorado, true, false, false));
        assert!(r.disclosure_required);
        assert_eq!(r.status, DisclosureStatus::RequiredButNotProvided);
        assert!(!r.landlord_compliant);
        assert!(r.citation.contains("0.5 ug/100cm²"));
    }

    #[test]
    fn co_knowledge_unremediated_disclosed_compliant() {
        let r = check(&input(Regime::Colorado, true, false, true));
        assert!(r.disclosure_required);
        assert_eq!(r.status, DisclosureStatus::RequiredAndProvided);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn co_knowledge_remediated_extinguishes_disclosure() {
        let r = check(&input(Regime::Colorado, true, true, false));
        assert!(!r.disclosure_required);
        assert_eq!(r.status, DisclosureStatus::NotRequired);
        assert!(r.landlord_compliant);
        assert!(r.remediation_extinguishes_disclosure);
        assert!(r.note.contains("extinguished"));
    }

    #[test]
    fn az_strictest_remediation_standard() {
        let r = check(&input(Regime::Arizona, true, false, false));
        assert_eq!(
            r.remediation_standard_ug_per_100cm2_x10, 1,
            "Arizona 0.1 ug/100cm² is the strictest standard of the 3 regulated regimes"
        );
        assert!(r.citation.contains("STRICTEST"));
        assert!(r.citation.contains("entry by non-owners barred"));
    }

    #[test]
    fn az_post_remediation_extinguishes() {
        let r = check(&input(Regime::Arizona, true, true, false));
        assert!(!r.disclosure_required);
        assert!(r.remediation_extinguishes_disclosure);
    }

    #[test]
    fn mt_remediation_does_NOT_extinguish_disclosure() {
        let r = check(&input(Regime::Montana, true, true, false));
        assert!(
            r.disclosure_required,
            "Montana requires disclosure EVEN AFTER remediation — distinct from CO + AZ"
        );
        assert_eq!(r.status, DisclosureStatus::RequiredButNotProvided);
        assert!(!r.landlord_compliant);
        assert!(!r.remediation_extinguishes_disclosure);
    }

    #[test]
    fn mt_post_remediation_with_disclosure_compliant() {
        let r = check(&input(Regime::Montana, true, true, true));
        assert!(r.disclosure_required);
        assert_eq!(r.status, DisclosureStatus::RequiredAndProvided);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn mt_standard_is_1_5_ug() {
        let r = check(&input(Regime::Montana, true, true, true));
        assert_eq!(r.remediation_standard_ug_per_100cm2_x10, 15);
    }

    #[test]
    fn mt_no_knowledge_no_obligation() {
        let r = check(&input(Regime::Montana, false, false, false));
        assert!(!r.disclosure_required);
    }

    #[test]
    fn default_no_statewide_statute_falls_back_to_fha() {
        let r = check(&input(Regime::Default, true, false, false));
        assert!(r.disclosure_required);
        assert!(r.citation.contains("3604"));
        assert!(r.citation.contains("FHA"));
    }

    #[test]
    fn default_no_knowledge_no_disclosure() {
        let r = check(&input(Regime::Default, false, false, false));
        assert!(!r.disclosure_required);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn state_routing_co_az_mt_default() {
        assert_eq!(Regime::for_state("CO"), Regime::Colorado);
        assert_eq!(Regime::for_state("AZ"), Regime::Arizona);
        assert_eq!(Regime::for_state("MT"), Regime::Montana);
        assert_eq!(Regime::for_state("TX"), Regime::Default);
        assert_eq!(Regime::for_state("CA"), Regime::Default);
    }

    #[test]
    fn state_routing_case_insensitive() {
        assert_eq!(Regime::for_state("co"), Regime::Colorado);
        assert_eq!(Regime::for_state("Az"), Regime::Arizona);
        assert_eq!(Regime::for_state("mt"), Regime::Montana);
    }

    #[test]
    fn standards_strict_ordering_az_co_mt() {
        let az = check(&input(Regime::Arizona, true, false, false));
        let co = check(&input(Regime::Colorado, true, false, false));
        let mt = check(&input(Regime::Montana, true, false, false));
        assert!(
            az.remediation_standard_ug_per_100cm2_x10 < co.remediation_standard_ug_per_100cm2_x10
        );
        assert!(
            co.remediation_standard_ug_per_100cm2_x10 < mt.remediation_standard_ug_per_100cm2_x10
        );
    }

    #[test]
    fn extinguishment_invariants() {
        // CO + AZ extinguish; MT + Default do not.
        let co = check(&input(Regime::Colorado, true, false, false));
        let az = check(&input(Regime::Arizona, true, false, false));
        let mt = check(&input(Regime::Montana, true, false, false));
        let d = check(&input(Regime::Default, true, false, false));
        assert!(co.remediation_extinguishes_disclosure);
        assert!(az.remediation_extinguishes_disclosure);
        assert!(!mt.remediation_extinguishes_disclosure);
        assert!(!d.remediation_extinguishes_disclosure);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let co = check(&input(Regime::Colorado, true, false, false));
        assert!(co.citation.contains("38-35.7-103"));
        assert!(co.citation.contains("6 CCR 1014-3"));

        let az = check(&input(Regime::Arizona, true, false, false));
        assert!(az.citation.contains("32-1166.04"));

        let mt = check(&input(Regime::Montana, true, false, false));
        assert!(mt.citation.contains("75-10-1301"));
        assert!(mt.citation.contains("does NOT extinguish"));
    }

    #[test]
    fn co_remediation_note_mentions_extinguished() {
        let r = check(&input(Regime::Colorado, true, true, false));
        assert!(r.note.contains("extinguished"));
    }

    #[test]
    fn mt_unremediated_violation_note_mentions_material_defect() {
        let r = check(&input(Regime::Montana, true, false, false));
        assert!(r.note.contains("Material defect"));
    }

    #[test]
    fn az_only_entry_restriction_in_citation() {
        let r_az = check(&input(Regime::Arizona, true, false, false));
        let r_co = check(&input(Regime::Colorado, true, false, false));
        let r_mt = check(&input(Regime::Montana, true, false, false));
        assert!(r_az.citation.contains("entry by non-owners barred"));
        assert!(!r_co.citation.contains("entry by non-owners barred"));
        assert!(!r_mt.citation.contains("entry by non-owners barred"));
    }
}
