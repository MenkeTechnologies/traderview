//! Emotional Support Animal (ESA) documentation requirements
//! for housing reasonable-accommodation requests.
//!
//! Distinct from sibling module `service_animal` (ADA-covered
//! service animals — typically dogs trained for specific
//! disability-related tasks). ESAs are a separate category
//! under the Fair Housing Act (42 U.S.C. § 3604(f)) — they
//! provide emotional support / therapeutic benefit but are NOT
//! trained for specific tasks. ESAs are protected under FHA
//! reasonable-accommodation doctrine but receive narrower
//! protection than service animals (e.g., no ADA Title II/III
//! public-accommodation rights).
//!
//! Trader-critical for landlord-investors processing reasonable-
//! accommodation requests; states are increasingly imposing
//! documentation-validity standards to combat fraudulent
//! online "ESA letter" mills that grant landlords grounds to
//! reject the accommodation request.
//!
//! California — Cal. Health & Safety Code § 122318 (AB 468,
//! effective January 1, 2022). Healthcare practitioners may NOT
//! provide ESA documentation unless they satisfy ALL FIVE
//! elements:
//!   (1) Valid, active license + writes all information in the
//!       document;
//!   (2) Licensed in the jurisdiction;
//!   (3) Established a provider relationship for at least 30
//!       days prior to issuing the documentation;
//!   (4) Completes a clinical evaluation of the person;
//!   (5) Advises (in writing or verbally) that misrepresenting
//!       the ESA as a service dog is a misdemeanor.
//! Practitioners violating § 122318 are subject to discipline
//! by their licensing board.
//!
//! Florida — Fla. Stat. § 760.27 (effective July 1, 2020).
//! ESA documentation is "reliable" only if the practitioner /
//! telehealth provider has PERSONAL KNOWLEDGE of the person's
//! disability and is acting within the scope of practice.
//! Telehealth providers must be Florida-licensed. Out-of-state
//! practitioners may provide documentation ONLY if they have
//! seen the person IN PERSON at least once. Internet-only
//! "registrations," ESA registration cards, vest patches, and
//! similar products are EXPLICITLY NOT reliable proof. Knowingly
//! providing fraudulent ESA documentation is a SECOND-DEGREE
//! MISDEMEANOR.
//!
//! Default — Federal Fair Housing Act + HUD Notice FHEO-2020-01
//! (January 28, 2020). Landlord may request reliable
//! documentation if disability or disability-related need for
//! the animal is NOT readily apparent. HUD guidance generally
//! disfavors documentation from "internet-only" therapeutic
//! relationships but stops short of categorical rejection.
//! Reasonable accommodation analysis under 42 U.S.C. § 3604(f).
//!
//! Citations: Cal. Health & Safety Code § 122318 (AB 468 ESA
//! practitioner requirements); Cal. Health & Safety Code
//! § 122318(b) (5-element test); Cal. Health & Safety Code
//! § 122318(c) (practitioner discipline); Fla. Stat. § 760.27
//! (ESA housing documentation reliability + telehealth/out-
//! of-state rules); Fla. Stat. § 760.27(2)(b) (in-person visit
//! requirement for out-of-state practitioners); Fla. Stat.
//! § 760.27(3) (second-degree misdemeanor for fraudulent
//! documentation); Federal Fair Housing Act, 42 U.S.C.
//! § 3604(f) (reasonable accommodation); HUD Notice FHEO-2020-01
//! (January 28, 2020) (assistance animal guidance); 24 CFR
//! § 100.204 (reasonable accommodations regulations); ADA,
//! 42 U.S.C. § 12102 (disability definition — incorporated by
//! reference).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    /// Cal. Health & Safety Code § 122318 (AB 468, eff. Jan 1,
    /// 2022) — 5-element practitioner test.
    California,
    /// Fla. Stat. § 760.27 (eff. July 1, 2020) — personal
    /// knowledge requirement + telehealth/out-of-state rules.
    Florida,
    /// Federal FHA + HUD Notice FHEO-2020-01 default.
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub regime: Regime,
    /// True if ESA documentation comes from a licensed healthcare
    /// practitioner with a valid active license.
    pub documentation_provided_by_licensed_practitioner: bool,
    /// California-specific — true if practitioner is licensed in
    /// the relevant jurisdiction (CA for AB 468 purposes).
    pub practitioner_licensed_in_jurisdiction: bool,
    /// Length of the provider-patient relationship at time of
    /// documentation issuance (days). CA requires ≥ 30 days.
    pub therapeutic_relationship_days: i64,
    /// True if practitioner completed a clinical evaluation
    /// (CA-specific element).
    pub clinical_evaluation_completed: bool,
    /// True if practitioner advised the person that
    /// misrepresenting the ESA as a service dog is a
    /// misdemeanor (CA-specific element).
    pub misdemeanor_warning_provided: bool,
    /// Florida-specific — true if practitioner has PERSONAL
    /// KNOWLEDGE of the person's disability.
    pub practitioner_has_personal_knowledge_of_disability: bool,
    /// Florida-specific — true if telehealth provider is
    /// Florida-licensed (or out-of-state practitioner has seen
    /// the person in person at least once).
    pub fl_telehealth_or_out_of_state_compliant: bool,
    /// True if documentation is from an internet-only ESA-letter
    /// service with no real therapeutic relationship.
    pub internet_only_documentation: bool,
    /// True if disability or disability-related need is readily
    /// apparent (federal FHA factor — landlord may not request
    /// documentation in this case).
    pub disability_or_need_readily_apparent: bool,
    /// True if person knowingly provided fraudulent
    /// documentation (triggers misdemeanor exposure).
    pub knowingly_fraudulent_documentation: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CheckResult {
    /// True if documentation meets the regime's reliability
    /// standard.
    pub documentation_reliable: bool,
    /// California-specific — true if all 5 elements of § 122318
    /// are satisfied.
    pub ca_five_element_test_satisfied: bool,
    /// Florida-specific — true if personal-knowledge + telehealth/
    /// out-of-state requirement satisfied.
    pub fl_personal_knowledge_satisfied: bool,
    /// True if landlord must accept the accommodation request
    /// (FHA reasonable accommodation analysis).
    pub landlord_must_accept_accommodation: bool,
    /// True if practitioner is subject to licensing board
    /// discipline (CA / FL violation).
    pub practitioner_subject_to_discipline: bool,
    /// True if knowingly fraudulent documentation triggers
    /// misdemeanor exposure (CA / FL).
    pub misdemeanor_exposure: bool,
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

/// California § 122318(b)(3) — 30-day therapeutic relationship.
pub const CA_RELATIONSHIP_DAYS_THRESHOLD: i64 = 30;

pub fn check(input: &Input) -> CheckResult {
    let mut notes: Vec<String> = Vec::new();
    let mut violations: Vec<String> = Vec::new();

    let relationship_days = input.therapeutic_relationship_days.max(0);

    // Federal FHA — readily-apparent disability bypasses
    // documentation requirement entirely.
    if input.disability_or_need_readily_apparent {
        notes.push(
            "Federal FHA + HUD Notice FHEO-2020-01 — disability OR disability-related \
             need for ESA is READILY APPARENT. Landlord may NOT request reliable \
             documentation in this case. Reasonable accommodation must be granted \
             absent other lawful basis for denial. § 122318 / § 760.27 documentation \
             standards do NOT apply when documentation is not requested."
                .to_string(),
        );
        return CheckResult {
            documentation_reliable: true,
            ca_five_element_test_satisfied: false,
            fl_personal_knowledge_satisfied: false,
            landlord_must_accept_accommodation: true,
            practitioner_subject_to_discipline: false,
            misdemeanor_exposure: false,
            compliant: true,
            violations,
            citation: citation_text(),
            notes,
        };
    }

    let mut ca_five_element_test_satisfied = false;
    let mut fl_personal_knowledge_satisfied = false;
    let mut practitioner_subject_to_discipline = false;

    let documentation_reliable = match input.regime {
        Regime::California => {
            // § 122318 — all 5 elements required.
            let element1_license = input.documentation_provided_by_licensed_practitioner;
            let element2_jurisdiction = input.practitioner_licensed_in_jurisdiction;
            let element3_relationship = relationship_days >= CA_RELATIONSHIP_DAYS_THRESHOLD;
            let element4_evaluation = input.clinical_evaluation_completed;
            let element5_warning = input.misdemeanor_warning_provided;
            ca_five_element_test_satisfied = element1_license
                && element2_jurisdiction
                && element3_relationship
                && element4_evaluation
                && element5_warning;

            if !ca_five_element_test_satisfied {
                let mut missing: Vec<&str> = Vec::new();
                if !element1_license {
                    missing.push("(1) valid active license + writes all information");
                }
                if !element2_jurisdiction {
                    missing.push("(2) licensed in jurisdiction");
                }
                if !element3_relationship {
                    missing.push("(3) 30-day prior therapeutic relationship");
                }
                if !element4_evaluation {
                    missing.push("(4) clinical evaluation");
                }
                if !element5_warning {
                    missing.push("(5) misdemeanor-misrepresentation warning");
                }
                violations.push(format!(
                    "Cal. Health & Safety Code § 122318 (AB 468) — ESA documentation \
                     UNRELIABLE. Missing element(s): {}. Practitioner subject to \
                     licensing-board discipline for issuing documentation without \
                     satisfying all 5 elements.",
                    missing.join("; "),
                ));
                practitioner_subject_to_discipline =
                    input.documentation_provided_by_licensed_practitioner;
            }
            ca_five_element_test_satisfied
        }
        Regime::Florida => {
            // § 760.27 — personal knowledge + telehealth/out-of-state compliance.
            fl_personal_knowledge_satisfied = input.documentation_provided_by_licensed_practitioner
                && input.practitioner_has_personal_knowledge_of_disability
                && input.fl_telehealth_or_out_of_state_compliant
                && !input.internet_only_documentation;

            if !fl_personal_knowledge_satisfied {
                let mut missing: Vec<&str> = Vec::new();
                if !input.documentation_provided_by_licensed_practitioner {
                    missing.push("licensed practitioner");
                }
                if !input.practitioner_has_personal_knowledge_of_disability {
                    missing.push("PERSONAL KNOWLEDGE of disability");
                }
                if !input.fl_telehealth_or_out_of_state_compliant {
                    missing.push(
                        "telehealth FL-licensed OR out-of-state practitioner in-person visit",
                    );
                }
                if input.internet_only_documentation {
                    missing.push("internet-only registration EXPLICITLY UNRELIABLE under § 760.27");
                }
                violations.push(format!(
                    "Fla. Stat. § 760.27 — ESA documentation UNRELIABLE. Missing/failed \
                     element(s): {}. Practitioner subject to discipline; knowingly \
                     fraudulent documentation = second-degree misdemeanor.",
                    missing.join("; "),
                ));
                practitioner_subject_to_discipline =
                    input.documentation_provided_by_licensed_practitioner;
            }
            fl_personal_knowledge_satisfied
        }
        Regime::Default => {
            // Federal FHA — HUD Notice FHEO-2020-01 reliability factors.
            let is_reliable = input.documentation_provided_by_licensed_practitioner
                && !input.internet_only_documentation;
            if !is_reliable {
                violations.push(
                    "Federal FHA + HUD Notice FHEO-2020-01 — documentation UNRELIABLE. \
                     Internet-only ESA letters generally do not satisfy reasonable-\
                     accommodation documentation standards. Landlord may reject \
                     accommodation request based on insufficient documentation."
                        .to_string(),
                );
            }
            is_reliable
        }
    };

    // Misdemeanor exposure — only when fraudulent documentation provided
    // AND state regime has misdemeanor provision (CA + FL).
    let misdemeanor_exposure = input.knowingly_fraudulent_documentation
        && matches!(input.regime, Regime::California | Regime::Florida);

    let landlord_must_accept_accommodation = documentation_reliable;

    // Notes.
    match input.regime {
        Regime::California => {
            notes.push(format!(
                "Cal. Health & Safety Code § 122318 (AB 468, eff. Jan 1, 2022) — 5-element \
                 practitioner test. Required: (1) valid active license; (2) licensed in \
                 jurisdiction; (3) ≥ 30-day prior relationship ({} days documented); \
                 (4) clinical evaluation; (5) misdemeanor-warning provided. All 5 must be \
                 satisfied for documentation to be reliable.",
                relationship_days,
            ));
        }
        Regime::Florida => {
            notes.push(
                "Fla. Stat. § 760.27 (eff. July 1, 2020) — practitioner must have PERSONAL \
                 KNOWLEDGE of the person's disability and act within scope of practice. \
                 Telehealth providers must be Florida-licensed; out-of-state practitioners \
                 may issue documentation ONLY if they have seen the person IN PERSON at \
                 least once. Internet-only registrations, vest patches, and similar \
                 products are EXPLICITLY UNRELIABLE under the statute."
                    .to_string(),
            );
        }
        Regime::Default => {
            notes.push(
                "Federal FHA + HUD Notice FHEO-2020-01 (January 28, 2020) — assistance \
                 animal reasonable-accommodation analysis under 42 U.S.C. § 3604(f). \
                 Landlord may request reliable documentation if disability or disability-\
                 related need is not readily apparent. HUD guidance disfavors internet-\
                 only therapeutic relationships but stops short of categorical \
                 rejection."
                    .to_string(),
            );
        }
    }

    if input.knowingly_fraudulent_documentation {
        notes.push(
            "KNOWINGLY FRAUDULENT documentation provided — CA and FL impose misdemeanor \
             penalties. Cal. Health & Safety Code § 122318 — misrepresenting ESA as \
             service dog is misdemeanor; Fla. Stat. § 760.27(3) — knowingly providing \
             false documentation is a SECOND-DEGREE MISDEMEANOR with fines + community \
             service possible. Federal FHA does not directly penalize fraudulent \
             documentation but the accommodation request may be denied for \
             unreliability."
                .to_string(),
        );
    }

    notes.push(
        "Sibling distinction: ESAs are DISTINCT from ADA service animals (covered in \
         `service_animal` module). Service animals are trained for specific disability-\
         related tasks and receive broader protection (ADA Title II/III + FHA + Air \
         Carrier Access Act). ESAs receive housing-only protection under FHA reasonable \
         accommodation. Other related modules: `reasonable_accommodation_modification` \
         (broader FHA reasonable-accommodation framework); `fair_chance_housing` \
         (criminal-background fair-housing); `pet_fees` (pet-deposit / fee caps — \
         ESAs are exempt from pet fees under FHA). § 122318 and § 760.27 were both \
         enacted in 2020-2022 in response to widespread fraudulent ESA-letter mills."
            .to_string(),
    );

    let compliant = violations.is_empty();

    CheckResult {
        documentation_reliable,
        ca_five_element_test_satisfied,
        fl_personal_knowledge_satisfied,
        landlord_must_accept_accommodation,
        practitioner_subject_to_discipline,
        misdemeanor_exposure,
        compliant,
        violations,
        citation: citation_text(),
        notes,
    }
}

fn citation_text() -> &'static str {
    "Cal. Health & Safety Code § 122318 (AB 468 ESA practitioner requirements, eff. \
     Jan 1, 2022); Cal. Health & Safety Code § 122318(b) (5-element test); Cal. Health \
     & Safety Code § 122318(c) (practitioner discipline); Fla. Stat. § 760.27 (ESA \
     housing documentation reliability + telehealth/out-of-state rules, eff. July 1, \
     2020); Fla. Stat. § 760.27(2)(b) (in-person visit requirement for out-of-state \
     practitioners); Fla. Stat. § 760.27(3) (second-degree misdemeanor for fraudulent \
     documentation); Federal Fair Housing Act, 42 U.S.C. § 3604(f) (reasonable \
     accommodation); HUD Notice FHEO-2020-01 (January 28, 2020) (assistance animal \
     guidance); 24 CFR § 100.204 (reasonable accommodations regulations); ADA, \
     42 U.S.C. § 12102 (disability definition incorporated by reference)"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(regime: Regime) -> Input {
        Input {
            regime,
            documentation_provided_by_licensed_practitioner: true,
            practitioner_licensed_in_jurisdiction: true,
            therapeutic_relationship_days: 30,
            clinical_evaluation_completed: true,
            misdemeanor_warning_provided: true,
            practitioner_has_personal_knowledge_of_disability: true,
            fl_telehealth_or_out_of_state_compliant: true,
            internet_only_documentation: false,
            disability_or_need_readily_apparent: false,
            knowingly_fraudulent_documentation: false,
        }
    }

    // ── California § 122318 5-element test ────────────────────

    #[test]
    fn ca_all_5_elements_satisfied_reliable() {
        let r = check(&input(Regime::California));
        assert!(r.documentation_reliable);
        assert!(r.ca_five_element_test_satisfied);
        assert!(r.landlord_must_accept_accommodation);
        assert!(r.compliant);
    }

    #[test]
    fn ca_missing_license_unreliable() {
        let mut b = input(Regime::California);
        b.documentation_provided_by_licensed_practitioner = false;
        let r = check(&b);
        assert!(!r.documentation_reliable);
        assert!(!r.ca_five_element_test_satisfied);
        assert!(!r.compliant);
    }

    #[test]
    fn ca_missing_jurisdiction_unreliable() {
        let mut b = input(Regime::California);
        b.practitioner_licensed_in_jurisdiction = false;
        let r = check(&b);
        assert!(!r.ca_five_element_test_satisfied);
    }

    #[test]
    fn ca_29_day_relationship_unreliable() {
        let mut b = input(Regime::California);
        b.therapeutic_relationship_days = 29;
        let r = check(&b);
        assert!(!r.ca_five_element_test_satisfied);
        assert!(r.violations.iter().any(|v| v.contains("30-day")));
    }

    #[test]
    fn ca_exactly_30_day_relationship_reliable() {
        let mut b = input(Regime::California);
        b.therapeutic_relationship_days = 30;
        let r = check(&b);
        assert!(r.ca_five_element_test_satisfied);
    }

    #[test]
    fn ca_missing_evaluation_unreliable() {
        let mut b = input(Regime::California);
        b.clinical_evaluation_completed = false;
        let r = check(&b);
        assert!(!r.ca_five_element_test_satisfied);
    }

    #[test]
    fn ca_missing_warning_unreliable() {
        let mut b = input(Regime::California);
        b.misdemeanor_warning_provided = false;
        let r = check(&b);
        assert!(!r.ca_five_element_test_satisfied);
    }

    #[test]
    fn ca_practitioner_discipline_on_violation() {
        let mut b = input(Regime::California);
        b.therapeutic_relationship_days = 15; // violates element 3
        let r = check(&b);
        assert!(r.practitioner_subject_to_discipline);
    }

    // ── Florida § 760.27 ─────────────────────────────────────

    #[test]
    fn fl_all_elements_satisfied_reliable() {
        let r = check(&input(Regime::Florida));
        assert!(r.documentation_reliable);
        assert!(r.fl_personal_knowledge_satisfied);
        assert!(r.landlord_must_accept_accommodation);
        assert!(r.compliant);
    }

    #[test]
    fn fl_missing_personal_knowledge_unreliable() {
        let mut b = input(Regime::Florida);
        b.practitioner_has_personal_knowledge_of_disability = false;
        let r = check(&b);
        assert!(!r.fl_personal_knowledge_satisfied);
        assert!(!r.documentation_reliable);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("PERSONAL KNOWLEDGE")));
    }

    #[test]
    fn fl_missing_telehealth_compliance_unreliable() {
        let mut b = input(Regime::Florida);
        b.fl_telehealth_or_out_of_state_compliant = false;
        let r = check(&b);
        assert!(!r.fl_personal_knowledge_satisfied);
    }

    #[test]
    fn fl_internet_only_explicitly_unreliable() {
        let mut b = input(Regime::Florida);
        b.internet_only_documentation = true;
        let r = check(&b);
        assert!(!r.fl_personal_knowledge_satisfied);
        assert!(r.violations.iter().any(|v| v.contains("internet-only")));
    }

    // ── Default — Federal FHA + HUD Notice ───────────────────

    #[test]
    fn default_licensed_practitioner_non_internet_reliable() {
        let r = check(&input(Regime::Default));
        assert!(r.documentation_reliable);
        assert!(r.landlord_must_accept_accommodation);
    }

    #[test]
    fn default_internet_only_unreliable() {
        let mut b = input(Regime::Default);
        b.internet_only_documentation = true;
        let r = check(&b);
        assert!(!r.documentation_reliable);
        assert!(r.violations.iter().any(|v| v.contains("Internet-only")));
    }

    #[test]
    fn default_unlicensed_unreliable() {
        let mut b = input(Regime::Default);
        b.documentation_provided_by_licensed_practitioner = false;
        let r = check(&b);
        assert!(!r.documentation_reliable);
    }

    // ── Readily-apparent disability short-circuit ────────────

    #[test]
    fn readily_apparent_disability_bypasses_documentation() {
        let mut b = input(Regime::California);
        b.disability_or_need_readily_apparent = true;
        // Even missing 4 of 5 CA elements, readily-apparent bypasses.
        b.documentation_provided_by_licensed_practitioner = false;
        b.practitioner_licensed_in_jurisdiction = false;
        b.therapeutic_relationship_days = 0;
        b.clinical_evaluation_completed = false;
        let r = check(&b);
        assert!(r.documentation_reliable);
        assert!(r.landlord_must_accept_accommodation);
        assert!(r.compliant);
        assert!(r.notes.iter().any(|n| n.contains("READILY APPARENT")));
    }

    #[test]
    fn readily_apparent_applies_to_all_regimes() {
        for regime in [Regime::California, Regime::Florida, Regime::Default] {
            let mut b = input(regime);
            b.disability_or_need_readily_apparent = true;
            b.documentation_provided_by_licensed_practitioner = false;
            let r = check(&b);
            assert!(r.landlord_must_accept_accommodation, "{:?}", regime);
            assert!(r.compliant, "{:?}", regime);
        }
    }

    // ── Misdemeanor exposure ─────────────────────────────────

    #[test]
    fn ca_fraudulent_documentation_misdemeanor() {
        let mut b = input(Regime::California);
        b.knowingly_fraudulent_documentation = true;
        let r = check(&b);
        assert!(r.misdemeanor_exposure);
    }

    #[test]
    fn fl_fraudulent_documentation_misdemeanor() {
        let mut b = input(Regime::Florida);
        b.knowingly_fraudulent_documentation = true;
        let r = check(&b);
        assert!(r.misdemeanor_exposure);
    }

    #[test]
    fn default_fraudulent_documentation_no_state_misdemeanor() {
        let mut b = input(Regime::Default);
        b.knowingly_fraudulent_documentation = true;
        let r = check(&b);
        // Default regime is federal FHA only; no state misdemeanor.
        assert!(!r.misdemeanor_exposure);
    }

    // ── Multi-regime invariants ──────────────────────────────

    #[test]
    fn only_ca_uses_5_element_test_invariant() {
        for regime in [Regime::California, Regime::Florida, Regime::Default] {
            let r = check(&input(regime));
            let expected = matches!(regime, Regime::California);
            assert_eq!(r.ca_five_element_test_satisfied, expected, "{:?}", regime);
        }
    }

    #[test]
    fn only_fl_uses_personal_knowledge_test_invariant() {
        for regime in [Regime::California, Regime::Florida, Regime::Default] {
            let r = check(&input(regime));
            let expected = matches!(regime, Regime::Florida);
            assert_eq!(r.fl_personal_knowledge_satisfied, expected, "{:?}", regime);
        }
    }

    #[test]
    fn misdemeanor_exposure_only_in_ca_and_fl_invariant() {
        for regime in [Regime::California, Regime::Florida, Regime::Default] {
            let mut b = input(regime);
            b.knowingly_fraudulent_documentation = true;
            let r = check(&b);
            let expected = matches!(regime, Regime::California | Regime::Florida);
            assert_eq!(r.misdemeanor_exposure, expected, "{:?}", regime);
        }
    }

    #[test]
    fn ca_5_element_truth_table() {
        // 5-prong sweep — break each element one at a time.
        let setters: Vec<Box<dyn Fn(&mut Input)>> = vec![
            Box::new(|b| b.documentation_provided_by_licensed_practitioner = false),
            Box::new(|b| b.practitioner_licensed_in_jurisdiction = false),
            Box::new(|b| b.therapeutic_relationship_days = 15),
            Box::new(|b| b.clinical_evaluation_completed = false),
            Box::new(|b| b.misdemeanor_warning_provided = false),
        ];
        for setter in &setters {
            let mut b = input(Regime::California);
            setter(&mut b);
            let r = check(&b);
            assert!(!r.ca_five_element_test_satisfied);
            assert!(!r.documentation_reliable);
        }
    }

    #[test]
    fn ca_30_day_threshold_constant_invariant() {
        assert_eq!(CA_RELATIONSHIP_DAYS_THRESHOLD, 30);
    }

    // ── Citation + sibling note ──────────────────────────────

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&input(Regime::California));
        assert!(r.citation.contains("§ 122318"));
        assert!(r.citation.contains("AB 468"));
        assert!(r.citation.contains("Jan 1, 2022"));
        assert!(r.citation.contains("§ 122318(b)"));
        assert!(r.citation.contains("§ 122318(c)"));
        assert!(r.citation.contains("§ 760.27"));
        assert!(r.citation.contains("§ 760.27(2)(b)"));
        assert!(r.citation.contains("§ 760.27(3)"));
        assert!(r.citation.contains("July 1, 2020"));
        assert!(r.citation.contains("42 U.S.C. § 3604(f)"));
        assert!(r.citation.contains("HUD Notice FHEO-2020-01"));
        assert!(r.citation.contains("January 28, 2020"));
        assert!(r.citation.contains("24 CFR § 100.204"));
        assert!(r.citation.contains("42 U.S.C. § 12102"));
    }

    #[test]
    fn sibling_distinction_note_present() {
        let r = check(&input(Regime::California));
        assert!(
            r.notes.iter().any(|n| n.contains("service_animal")
                && n.contains("reasonable_accommodation_modification")
                && n.contains("fair_chance_housing")
                && n.contains("pet_fees")
                && n.contains("ADA")
                && n.contains("ESAs are exempt")),
            "sibling distinction note must reference service_animal + other related modules + ESA-vs-service-animal + pet-fee exemption"
        );
    }

    // ── Defensive input clamping ──────────────────────────────

    #[test]
    fn defensive_negative_relationship_days_clamped() {
        let mut b = input(Regime::California);
        b.therapeutic_relationship_days = -10;
        let r = check(&b);
        // Negative → 0; below 30-day threshold → fails element 3.
        assert!(!r.ca_five_element_test_satisfied);
    }

    #[test]
    fn boundary_29_30_31_day_truth_table() {
        let cells = [(29, false), (30, true), (31, true)];
        for (days, expected) in cells.iter() {
            let mut b = input(Regime::California);
            b.therapeutic_relationship_days = *days;
            let r = check(&b);
            assert_eq!(r.ca_five_element_test_satisfied, *expected, "days={}", days);
        }
    }
}
