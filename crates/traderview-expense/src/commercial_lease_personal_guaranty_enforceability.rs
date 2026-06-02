//! Commercial lease personal guaranty enforceability
//! framework — when may a landlord enforce a personal
//! guaranty signed by a commercial tenant's principal
//! against the principal individually? Distinct from
//! sibling `tenant_lease_guarantor_disclosure` (which
//! covers RESIDENTIAL guarantor disclosure under NY
//! HSTPA 2019 + CA Civ. Code + Restatement (Third) of
//! Suretyship and Guaranty).
//!
//! Trader-landlord critical because commercial leases
//! routinely require personal guaranty of corporate or
//! LLC tenant obligations as condition of rental.
//! Recent NYC COVID-era legislation + Second Circuit
//! constitutional rulings significantly altered the
//! enforceability landscape; "Good Guy Guaranty"
//! industry-standard limitations cap guarantor
//! exposure to surrender-date arrears; common-law
//! Statute of Frauds and material-modification rules
//! continue to constrain enforceability.
//!
//! Companion to tenant_lease_guarantor_disclosure
//! (residential — iter 433), lease_disclosures,
//! lease_assignment_consent, lease_waiver_
//! enforceability, tenant_estoppel_certificate.
//!
//! **Four-jurisdiction framework**:
//!
//! NEW YORK CITY (post-COVID) — NYC Admin. Code
//! § 22-1005 enacted May 26, 2020 to render
//! UNENFORCEABLE personal guaranties of commercial
//! lease obligations arising March 7, 2020 through
//! June 30, 2021 for businesses required to (a) cease
//! serving food/beverages for on-premises consumption,
//! (b) operate as non-essential retail, or (c) close
//! to public under executive orders. Second Circuit
//! in Melendez v. City of New York (April 26, 2022)
//! VACATED dismissal and remanded; on remand SDNY
//! Judge Abrams held law VIOLATES CONTRACTS CLAUSE
//! (Melendez II, March 31, 2023) because law
//! PERMANENTLY extinguished guaranties rather than
//! deferring enforcement. Still operative as of
//! mid-2025 pending Supreme Court review.
//!
//! NEW YORK — NY GOL § 5-701(a)(1) Statute of Frauds
//! requires WRITTEN GUARANTY for any lease (or
//! guaranty) > 12 MONTHS signed by guarantor;
//! NY GOL § 5-701(a)(2) requires writing for
//! collateral promise to answer for another's debt;
//! "Good Guy Guaranty" (industry-standard NYC
//! commercial lease term) limits guarantor liability
//! to UNPAID RENT THROUGH SURRENDER DATE (not full
//! remaining lease term).
//!
//! CALIFORNIA — Cal. Civ. Code § 2787-2856
//! (suretyship); § 2819 MATERIAL MODIFICATION
//! discharges guarantor (rent increase without
//! consent extinguishes guaranty); § 1670.5
//! unconscionability; § 2799 continuing-guaranty
//! revocation right after notice.
//!
//! DEFAULT / common law — Restatement (Third) of
//! Suretyship and Guaranty (1996); § 41 material
//! modification extinguishes; § 39 novation
//! extinguishes; strict construction against
//! creditor; uniform commercial code Article 9 does
//! NOT apply to real property.
//!
//! **"Good Guy Guaranty" (GGG) industry-standard
//! terms**:
//! 1. Guarantor liability LIMITED to rent through
//!    date of surrender;
//! 2. Surrender requires (a) tenant out of premises;
//!    (b) keys delivered; (c) advance notice (typically
//!    30-90 days);
//! 3. Guarantor NOT liable for (a) future rent post-
//!    surrender; (b) future damages from re-tenanting;
//!    (c) attorney's fees post-surrender;
//! 4. Effectively converts "full-recourse" personal
//!    guaranty into "surrender-date arrears"
//!    guaranty;
//! 5. Strong incentive for tenant to vacate cleanly
//!    rather than abandon.
//!
//! **NY GOL § 5-701(a) Statute of Frauds —
//! guaranty writing requirements**:
//! - § 5-701(a)(1) — agreement not to be performed
//!   within one year (covers > 12-month leases);
//! - § 5-701(a)(2) — promise to answer for the
//!   debt of another (covers ALL guaranties);
//! - Both subsections require WRITING SIGNED by
//!   guarantor or authorized agent.
//!
//! **Trader-landlord critical fact patterns**:
//!
//! 1. NYC trader-landlord leases retail space to LLC
//!    tenant 2019; LLC default March 2020; landlord
//:    seeks to enforce personal guaranty of LLC
//!    principal — UNENFORCEABLE under NYC § 22-1005
//!    for arrears March 7, 2020 through June 30,
//!    2021 (assuming tenant non-essential retail);
//!    arrears outside that window enforceable.
//! 2. NY commercial lease 36-month term with oral
//!    personal guaranty — UNENFORCEABLE under NY
//!    GOL § 5-701(a)(1) Statute of Frauds.
//! 3. Good Guy Guaranty — tenant LLC defaults month
//!    18 of 60-month lease; tenant vacates with 30-
//!    day notice and keys delivered; guarantor
//!    liable only for month-18 arrears, NOT
//!    remaining 42 months of rent.
//! 4. CA commercial lease — landlord raises rent 15%
//!    at year 3 without guarantor consent; § 2819
//!    MATERIAL MODIFICATION rule DISCHARGES guarantor
//!    from entire obligation.
//! 5. Default jurisdiction — landlord enters novation
//!    with new tenant LLC (substituting parties); RESTATEMENT
//!    (Third) of Suretyship § 39 NOVATION extinguishes
//!    original guaranty.
//!
//! Citations: NYC Admin. Code § 22-1005; Melendez v.
//! City of New York, 16 F.4th 992 (2d Cir. 2021)
//! (initial); Melendez v. City of New York, 27 F.4th
//! 119 (2d Cir. 2022) (rehearing); Melendez v. City
//! of New York, 668 F. Supp. 3d 184 (S.D.N.Y. Mar.
//! 31, 2023); NY GOL § 5-701(a)(1); NY GOL
//! § 5-701(a)(2); Cal. Civ. Code § 2787-2856; Cal.
//! Civ. Code § 2819; Cal. Civ. Code § 1670.5; Cal.
//! Civ. Code § 2799; Restatement (Third) of
//! Suretyship and Guaranty (1996); Restatement
//! (Third) of Suretyship § 41; Restatement (Third)
//! of Suretyship § 39; U.S. Const. art. I § 10
//! (Contracts Clause).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    NewYorkCity,
    NewYorkState,
    California,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommercialLeasePersonalGuarantyEnforceabilityInput {
    pub jurisdiction: Jurisdiction,
    /// Lease term in months.
    pub lease_term_months: u32,
    /// Whether guaranty is in writing signed by
    /// guarantor (NY GOL § 5-701(a)).
    pub guaranty_in_writing_signed_by_guarantor: bool,
    /// Whether obligation arose within COVID-19
    /// blackout window (March 7, 2020 - June 30,
    /// 2021).
    pub obligation_within_covid_blackout: bool,
    /// Whether tenant business was one of the three
    /// covered categories under NYC § 22-1005:
    /// food/beverage on-premises consumption,
    /// non-essential retail, or required to close
    /// under EO.
    pub tenant_covered_business_category: bool,
    /// Whether guaranty contains Good Guy Guaranty
    /// surrender-date limitation.
    pub has_good_guy_guaranty: bool,
    /// Whether tenant has properly surrendered
    /// (vacated + keys delivered + advance notice).
    pub tenant_properly_surrendered: bool,
    /// Whether landlord made material modification
    /// (rent increase, term extension) without
    /// guarantor consent.
    pub material_modification_without_consent: bool,
    /// Whether novation between landlord and new
    /// tenant.
    pub novation_with_new_tenant: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CommercialLeasePersonalGuarantyEnforceabilityResult {
    pub guaranty_enforceable: bool,
    pub nyc_section_22_1005_engaged: bool,
    pub statute_of_frauds_satisfied: bool,
    pub good_guy_guaranty_limits_liability: bool,
    pub material_modification_discharge: bool,
    pub novation_extinguishes_guaranty: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(
    input: &CommercialLeasePersonalGuarantyEnforceabilityInput,
) -> CommercialLeasePersonalGuarantyEnforceabilityResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let statute_of_frauds_satisfied = match input.jurisdiction {
        Jurisdiction::NewYorkCity | Jurisdiction::NewYorkState => {
            input.lease_term_months <= 12 || input.guaranty_in_writing_signed_by_guarantor
        }
        _ => input.guaranty_in_writing_signed_by_guarantor || input.lease_term_months <= 12,
    };

    let nyc_section_22_1005_engaged = matches!(input.jurisdiction, Jurisdiction::NewYorkCity)
        && input.obligation_within_covid_blackout
        && input.tenant_covered_business_category;

    let good_guy_guaranty_limits_liability = input.has_good_guy_guaranty
        && input.tenant_properly_surrendered;

    let material_modification_discharge = input.material_modification_without_consent;

    let novation_extinguishes_guaranty = input.novation_with_new_tenant;

    let guaranty_enforceable = statute_of_frauds_satisfied
        && !nyc_section_22_1005_engaged
        && !material_modification_discharge
        && !novation_extinguishes_guaranty;

    if !statute_of_frauds_satisfied {
        match input.jurisdiction {
            Jurisdiction::NewYorkCity | Jurisdiction::NewYorkState => failure_reasons.push(format!(
                "NY GOL § 5-701(a)(1) Statute of Frauds — lease term {} months exceeds 12-month threshold; personal guaranty MUST be in WRITING SIGNED by guarantor or authorized agent; ORAL guaranty UNENFORCEABLE; NY GOL § 5-701(a)(2) separately requires writing for ALL promises to answer for debt of another",
                input.lease_term_months
            )),
            _ => failure_reasons.push(format!(
                "Common-law Statute of Frauds — lease term {} months exceeds 12-month threshold; personal guaranty must be in WRITING SIGNED by guarantor; oral guaranty UNENFORCEABLE",
                input.lease_term_months
            )),
        }
    }

    if nyc_section_22_1005_engaged {
        failure_reasons.push(
            "NYC Admin. Code § 22-1005 (enacted May 26, 2020) — personal guaranty of commercial lease obligations arising March 7, 2020 through June 30, 2021 UNENFORCEABLE for businesses required to (a) cease serving food/beverages for on-premises consumption, (b) operate as non-essential retail, or (c) close under executive orders".to_string(),
        );
        failure_reasons.push(
            "Melendez v. City of New York, 27 F.4th 119 (2d Cir. 2022) — Second Circuit VACATED dismissal and remanded; on remand 668 F. Supp. 3d 184 (S.D.N.Y. Mar. 31, 2023), SDNY held law VIOLATES Contracts Clause (U.S. Const. art. I § 10) because PERMANENTLY extinguishes guaranties rather than deferring enforcement; pending Supreme Court review; arrears OUTSIDE the March 7, 2020 to June 30, 2021 window remain enforceable".to_string(),
        );
    }

    if good_guy_guaranty_limits_liability {
        failure_reasons.push(
            "GOOD GUY GUARANTY (industry-standard NYC commercial lease term) — guarantor liability LIMITED to unpaid rent through DATE OF SURRENDER; surrender requires (a) tenant out of premises; (b) keys delivered to landlord; (c) advance notice (typically 30-90 days); guarantor NOT LIABLE for (1) future rent post-surrender; (2) future damages from re-tenanting; (3) attorney's fees post-surrender; effectively converts full-recourse guaranty into surrender-date-arrears guaranty".to_string(),
        );
    }

    if material_modification_discharge {
        match input.jurisdiction {
            Jurisdiction::California => failure_reasons.push(
                "Cal. Civ. Code § 2819 MATERIAL MODIFICATION RULE — material modification of underlying obligation (rent increase, term extension, scope change) without guarantor consent DISCHARGES guarantor from entire obligation; harshest jurisdiction for landlords seeking lease changes".to_string(),
            ),
            _ => failure_reasons.push(
                "Restatement (Third) of Suretyship § 41 MATERIAL MODIFICATION — material modification of underlying obligation without guarantor's consent EXTINGUISHES guaranty; strict construction against creditor (landlord)".to_string(),
            ),
        }
    }

    if novation_extinguishes_guaranty {
        failure_reasons.push(
            "Restatement (Third) of Suretyship § 39 NOVATION — substitution of new tenant for original tenant (with landlord consent) EXTINGUISHES original personal guaranty; novation requires (1) new party; (2) intent to release original party; (3) consent of all three parties; documented through novation agreement".to_string(),
        );
    }

    if guaranty_enforceable {
        failure_reasons.push(
            "Personal guaranty ENFORCEABLE — Statute of Frauds satisfied; no NYC § 22-1005 COVID blackout engagement; no material modification without consent; no novation; landlord may pursue guarantor personally for tenant's lease obligations subject to any Good Guy Guaranty surrender-date limitations".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "Four-jurisdiction framework: NEW YORK CITY (NYC Admin. Code § 22-1005 COVID-era guaranty law; Second Circuit Melendez ruling; SDNY held violates Contracts Clause March 31, 2023); NEW YORK STATE (NY GOL § 5-701(a)(1)/(2) Statute of Frauds; Good Guy Guaranty industry standard); CALIFORNIA (Cal. Civ. Code § 2787-2856 suretyship + § 2819 material modification + § 1670.5 unconscionability + § 2799 continuing-guaranty revocation); DEFAULT (Restatement (Third) of Suretyship § 41 material modification + § 39 novation)".to_string(),
        "NYC Admin. Code § 22-1005 (enacted May 26, 2020) — UNENFORCEABLE personal guaranty for commercial lease obligations arising March 7, 2020 through June 30, 2021 for three covered tenant categories: (1) food/beverage on-premises consumption ceased; (2) non-essential retail; (3) required to close under executive orders; remains operative pending Supreme Court review of constitutional challenge".to_string(),
        "Melendez v. City of New York constitutional history: (1) Melendez v. City of New York, 16 F.4th 992 (2d Cir. 2021) — initial Second Circuit panel decision; (2) Melendez v. City of New York, 27 F.4th 119 (2d Cir. 2022) — panel rehearing VACATED dismissal and remanded; Second Circuit held law substantially impaired contracts and served legitimate public purpose BUT was not appropriate and reasonable means; (3) Melendez v. City of New York, 668 F. Supp. 3d 184 (S.D.N.Y. Mar. 31, 2023) — SDNY Judge Abrams held law VIOLATES Contracts Clause (U.S. Const. art. I § 10) because it PERMANENTLY extinguished guaranties rather than deferring enforcement".to_string(),
        "Second Circuit constitutional concerns (Melendez 27 F.4th 119): (1) law effected PERMANENT (not temporary) unenforceability of guaranties; (2) not limited to circumstances where guarantors owned or intended to reopen the businesses; (3) allocated economic risk to landlords rather than guarantors; (4) not conditioned on demonstrated financial need; (5) provided landlords with no alternative remedial avenues".to_string(),
        "GOOD GUY GUARANTY (GGG) industry-standard NYC commercial lease term: (1) guarantor liability LIMITED to rent through DATE OF SURRENDER; (2) surrender requires (a) tenant out of premises, (b) keys delivered, (c) advance notice typically 30-90 days; (3) guarantor NOT LIABLE for (a) future rent post-surrender, (b) future damages from re-tenanting, (c) attorney's fees post-surrender; (4) effectively converts full-recourse guaranty into surrender-date-arrears guaranty; (5) strong tenant incentive to vacate cleanly rather than abandon".to_string(),
        "NY GOL § 5-701(a) Statute of Frauds — guaranty writing requirements: § 5-701(a)(1) — agreement not to be performed within one year (covers > 12-month leases); § 5-701(a)(2) — promise to answer for debt of another (covers ALL guaranties); both subsections require WRITING SIGNED by guarantor or authorized agent".to_string(),
        "Cal. Civ. Code suretyship framework: § 2787 statutory recognition of guarantor suretyship; § 2819 MATERIAL MODIFICATION RULE discharges guarantor from entire obligation if landlord modifies underlying without consent (harshest jurisdiction); § 1670.5 unconscionability defense; § 2799 continuing-guaranty revocation right after notice; § 2810 statute of limitations on guarantor claims".to_string(),
        "Restatement (Third) of Suretyship and Guaranty (1996): § 41 MATERIAL MODIFICATION extinguishes guaranty; § 39 NOVATION extinguishes guaranty; strict construction against creditor (landlord); Uniform Commercial Code Article 9 (secured transactions) does NOT apply to real property leases".to_string(),
        "Trader-landlord critical fact patterns: (1) NYC retail LLC default March 2020 — § 22-1005 bars enforcement for blackout-window arrears (non-essential retail); arrears outside enforceable; (2) NY 36-month lease oral guaranty — § 5-701(a)(1) UNENFORCEABLE; (3) Good Guy Guaranty — tenant LLC vacates with 30-day notice + keys at month 18 of 60-month lease — guarantor liable only for month-18 arrears not remaining 42 months; (4) CA 15% rent increase without guarantor consent — § 2819 MATERIAL MODIFICATION discharges; (5) landlord novation with new tenant LLC — Restatement § 39 NOVATION extinguishes".to_string(),
        "Companion to tenant_lease_guarantor_disclosure (residential — iter 433 NY HSTPA 2019 one-month-aggregate cap + FCRA + Restatement (Third)); lease_disclosures + lease_assignment_consent + lease_waiver_enforceability + tenant_estoppel_certificate".to_string(),
    ];

    CommercialLeasePersonalGuarantyEnforceabilityResult {
        guaranty_enforceable,
        nyc_section_22_1005_engaged,
        statute_of_frauds_satisfied,
        good_guy_guaranty_limits_liability,
        material_modification_discharge,
        novation_extinguishes_guaranty,
        failure_reasons,
        citation: "NYC Admin. Code § 22-1005; Melendez v. City of New York, 16 F.4th 992 (2d Cir. 2021); Melendez v. City of New York, 27 F.4th 119 (2d Cir. 2022); Melendez v. City of New York, 668 F. Supp. 3d 184 (S.D.N.Y. Mar. 31, 2023); NY GOL § 5-701(a)(1); NY GOL § 5-701(a)(2); Cal. Civ. Code § 2787-2856; Cal. Civ. Code § 2819; Cal. Civ. Code § 1670.5; Cal. Civ. Code § 2799; Restatement (Third) of Suretyship and Guaranty (1996); Restatement (Third) of Suretyship § 41; Restatement (Third) of Suretyship § 39; U.S. Const. art. I § 10 (Contracts Clause)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ny_enforceable() -> CommercialLeasePersonalGuarantyEnforceabilityInput {
        CommercialLeasePersonalGuarantyEnforceabilityInput {
            jurisdiction: Jurisdiction::NewYorkState,
            lease_term_months: 60,
            guaranty_in_writing_signed_by_guarantor: true,
            obligation_within_covid_blackout: false,
            tenant_covered_business_category: false,
            has_good_guy_guaranty: false,
            tenant_properly_surrendered: false,
            material_modification_without_consent: false,
            novation_with_new_tenant: false,
        }
    }

    #[test]
    fn ny_60_month_written_enforceable() {
        let r = check(&ny_enforceable());
        assert!(r.guaranty_enforceable);
        assert!(r.statute_of_frauds_satisfied);
    }

    #[test]
    fn ny_36_month_oral_unenforceable() {
        let mut i = ny_enforceable();
        i.lease_term_months = 36;
        i.guaranty_in_writing_signed_by_guarantor = false;
        let r = check(&i);
        assert!(!r.guaranty_enforceable);
        assert!(!r.statute_of_frauds_satisfied);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 5-701(a)(1)")
            && f.contains("UNENFORCEABLE")
            && f.contains("§ 5-701(a)(2)")));
    }

    #[test]
    fn ny_12_month_oral_enforceable() {
        let mut i = ny_enforceable();
        i.lease_term_months = 12;
        i.guaranty_in_writing_signed_by_guarantor = false;
        let r = check(&i);
        assert!(r.statute_of_frauds_satisfied);
    }

    #[test]
    fn nyc_22_1005_blackout_unenforceable() {
        let mut i = ny_enforceable();
        i.jurisdiction = Jurisdiction::NewYorkCity;
        i.obligation_within_covid_blackout = true;
        i.tenant_covered_business_category = true;
        let r = check(&i);
        assert!(r.nyc_section_22_1005_engaged);
        assert!(!r.guaranty_enforceable);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("§ 22-1005")
            && f.contains("UNENFORCEABLE")
            && f.contains("March 7, 2020")
            && f.contains("June 30, 2021")));
    }

    #[test]
    fn nyc_22_1005_engages_only_for_covered_business() {
        let mut i = ny_enforceable();
        i.jurisdiction = Jurisdiction::NewYorkCity;
        i.obligation_within_covid_blackout = true;
        i.tenant_covered_business_category = false;
        let r = check(&i);
        assert!(!r.nyc_section_22_1005_engaged);
        assert!(r.guaranty_enforceable);
    }

    #[test]
    fn nyc_22_1005_engages_only_within_blackout() {
        let mut i = ny_enforceable();
        i.jurisdiction = Jurisdiction::NewYorkCity;
        i.obligation_within_covid_blackout = false;
        i.tenant_covered_business_category = true;
        let r = check(&i);
        assert!(!r.nyc_section_22_1005_engaged);
    }

    #[test]
    fn melendez_constitutional_history_disclosed() {
        let mut i = ny_enforceable();
        i.jurisdiction = Jurisdiction::NewYorkCity;
        i.obligation_within_covid_blackout = true;
        i.tenant_covered_business_category = true;
        let r = check(&i);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("Melendez")
            && f.contains("27 F.4th 119")
            && f.contains("Contracts Clause")
            && f.contains("U.S. Const. art. I § 10")
            && f.contains("PERMANENTLY")));
    }

    #[test]
    fn good_guy_guaranty_with_surrender_limits_liability() {
        let mut i = ny_enforceable();
        i.has_good_guy_guaranty = true;
        i.tenant_properly_surrendered = true;
        let r = check(&i);
        assert!(r.good_guy_guaranty_limits_liability);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("GOOD GUY GUARANTY")
            && f.contains("DATE OF SURRENDER")
            && f.contains("keys delivered")
            && f.contains("surrender-date-arrears")));
    }

    #[test]
    fn good_guy_guaranty_without_surrender_no_limitation() {
        let mut i = ny_enforceable();
        i.has_good_guy_guaranty = true;
        i.tenant_properly_surrendered = false;
        let r = check(&i);
        assert!(!r.good_guy_guaranty_limits_liability);
    }

    #[test]
    fn ca_material_modification_discharges() {
        let mut i = ny_enforceable();
        i.jurisdiction = Jurisdiction::California;
        i.material_modification_without_consent = true;
        let r = check(&i);
        assert!(r.material_modification_discharge);
        assert!(!r.guaranty_enforceable);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("Cal. Civ. Code § 2819")
            && f.contains("MATERIAL MODIFICATION")
            && f.contains("DISCHARGES")));
    }

    #[test]
    fn default_material_modification_restatement_41() {
        let mut i = ny_enforceable();
        i.jurisdiction = Jurisdiction::Default;
        i.material_modification_without_consent = true;
        let r = check(&i);
        assert!(r.material_modification_discharge);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("Restatement (Third) of Suretyship § 41")
            && f.contains("EXTINGUISHES")));
    }

    #[test]
    fn novation_extinguishes() {
        let mut i = ny_enforceable();
        i.novation_with_new_tenant = true;
        let r = check(&i);
        assert!(r.novation_extinguishes_guaranty);
        assert!(!r.guaranty_enforceable);
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("Restatement (Third) of Suretyship § 39")
            && f.contains("NOVATION")
            && f.contains("EXTINGUISHES")));
    }

    #[test]
    fn enforceable_guaranty_message_displayed() {
        let r = check(&ny_enforceable());
        assert!(r.failure_reasons.iter().any(|f|
            f.contains("Personal guaranty ENFORCEABLE")
            && f.contains("Statute of Frauds satisfied")));
    }

    #[test]
    fn jurisdiction_truth_table_four_cells() {
        for j in [
            Jurisdiction::NewYorkCity,
            Jurisdiction::NewYorkState,
            Jurisdiction::California,
            Jurisdiction::Default,
        ] {
            let mut i = ny_enforceable();
            i.jurisdiction = j;
            i.lease_term_months = 36;
            i.guaranty_in_writing_signed_by_guarantor = false;
            let r = check(&i);
            assert!(!r.statute_of_frauds_satisfied, "j={:?}", j);
        }
    }

    #[test]
    fn nyc_uniquely_engages_22_1005_invariant() {
        let mut nyc = ny_enforceable();
        nyc.jurisdiction = Jurisdiction::NewYorkCity;
        nyc.obligation_within_covid_blackout = true;
        nyc.tenant_covered_business_category = true;
        let r_nyc = check(&nyc);
        assert!(r_nyc.nyc_section_22_1005_engaged);

        for j in [
            Jurisdiction::NewYorkState,
            Jurisdiction::California,
            Jurisdiction::Default,
        ] {
            let mut i = ny_enforceable();
            i.jurisdiction = j;
            i.obligation_within_covid_blackout = true;
            i.tenant_covered_business_category = true;
            let r = check(&i);
            assert!(!r.nyc_section_22_1005_engaged, "j={:?}", j);
        }
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&ny_enforceable());
        assert!(r.citation.contains("NYC Admin. Code § 22-1005"));
        assert!(r.citation.contains("Melendez v. City of New York, 16 F.4th 992"));
        assert!(r.citation.contains("Melendez v. City of New York, 27 F.4th 119"));
        assert!(r.citation.contains("Melendez v. City of New York, 668 F. Supp. 3d 184"));
        assert!(r.citation.contains("NY GOL § 5-701(a)(1)"));
        assert!(r.citation.contains("NY GOL § 5-701(a)(2)"));
        assert!(r.citation.contains("Cal. Civ. Code § 2787-2856"));
        assert!(r.citation.contains("Cal. Civ. Code § 2819"));
        assert!(r.citation.contains("Cal. Civ. Code § 1670.5"));
        assert!(r.citation.contains("Cal. Civ. Code § 2799"));
        assert!(r.citation.contains("Restatement (Third) of Suretyship and Guaranty"));
        assert!(r.citation.contains("Restatement (Third) of Suretyship § 41"));
        assert!(r.citation.contains("Restatement (Third) of Suretyship § 39"));
        assert!(r.citation.contains("U.S. Const. art. I § 10"));
    }

    #[test]
    fn note_pins_four_jurisdiction_framework() {
        let r = check(&ny_enforceable());
        assert!(r.notes.iter().any(|n|
            n.contains("Four-jurisdiction framework")
            && n.contains("NEW YORK CITY")
            && n.contains("§ 22-1005")
            && n.contains("CALIFORNIA")
            && n.contains("DEFAULT")));
    }

    #[test]
    fn note_pins_nyc_22_1005_three_categories() {
        let r = check(&ny_enforceable());
        assert!(r.notes.iter().any(|n|
            n.contains("NYC Admin. Code § 22-1005")
            && n.contains("May 26, 2020")
            && n.contains("March 7, 2020 through June 30, 2021")
            && n.contains("food/beverage on-premises")
            && n.contains("non-essential retail")
            && n.contains("close under executive orders")));
    }

    #[test]
    fn note_pins_melendez_three_decisions() {
        let r = check(&ny_enforceable());
        assert!(r.notes.iter().any(|n|
            n.contains("Melendez v. City of New York constitutional history")
            && n.contains("16 F.4th 992")
            && n.contains("27 F.4th 119")
            && n.contains("668 F. Supp. 3d 184")
            && n.contains("Contracts Clause")));
    }

    #[test]
    fn note_pins_second_circuit_five_concerns() {
        let r = check(&ny_enforceable());
        assert!(r.notes.iter().any(|n|
            n.contains("Second Circuit constitutional concerns")
            && n.contains("PERMANENT")
            && n.contains("not limited")
            && n.contains("not conditioned")
            && n.contains("alternative remedial avenues")));
    }

    #[test]
    fn note_pins_good_guy_guaranty_five_elements() {
        let r = check(&ny_enforceable());
        assert!(r.notes.iter().any(|n|
            n.contains("GOOD GUY GUARANTY (GGG)")
            && n.contains("DATE OF SURRENDER")
            && n.contains("30-90 days")
            && n.contains("surrender-date-arrears")));
    }

    #[test]
    fn note_pins_ny_gol_5_701_two_subsections() {
        let r = check(&ny_enforceable());
        assert!(r.notes.iter().any(|n|
            n.contains("NY GOL § 5-701(a)")
            && n.contains("§ 5-701(a)(1)")
            && n.contains("§ 5-701(a)(2)")
            && n.contains("ALL guaranties")));
    }

    #[test]
    fn note_pins_ca_suretyship_framework() {
        let r = check(&ny_enforceable());
        assert!(r.notes.iter().any(|n|
            n.contains("Cal. Civ. Code suretyship framework")
            && n.contains("§ 2787")
            && n.contains("§ 2819")
            && n.contains("§ 1670.5")
            && n.contains("§ 2799")
            && n.contains("§ 2810")));
    }

    #[test]
    fn note_pins_restatement_third_suretyship_1996() {
        let r = check(&ny_enforceable());
        assert!(r.notes.iter().any(|n|
            n.contains("Restatement (Third) of Suretyship and Guaranty (1996)")
            && n.contains("§ 41 MATERIAL MODIFICATION")
            && n.contains("§ 39 NOVATION")
            && n.contains("strict construction")));
    }

    #[test]
    fn note_pins_trader_fact_patterns_five() {
        let r = check(&ny_enforceable());
        assert!(r.notes.iter().any(|n|
            n.contains("Trader-landlord critical fact patterns")
            && n.contains("NYC retail LLC")
            && n.contains("Good Guy Guaranty")
            && n.contains("§ 2819 MATERIAL MODIFICATION")
            && n.contains("§ 39 NOVATION")));
    }

    #[test]
    fn note_pins_companion_modules() {
        let r = check(&ny_enforceable());
        assert!(r.notes.iter().any(|n|
            n.contains("Companion to tenant_lease_guarantor_disclosure")
            && n.contains("residential")
            && n.contains("tenant_estoppel_certificate")));
    }

    #[test]
    fn multiple_failures_stack() {
        let mut i = ny_enforceable();
        i.jurisdiction = Jurisdiction::NewYorkCity;
        i.lease_term_months = 36;
        i.guaranty_in_writing_signed_by_guarantor = false;
        i.obligation_within_covid_blackout = true;
        i.tenant_covered_business_category = true;
        i.material_modification_without_consent = true;
        let r = check(&i);
        assert!(r.failure_reasons.len() >= 4);
    }
}
