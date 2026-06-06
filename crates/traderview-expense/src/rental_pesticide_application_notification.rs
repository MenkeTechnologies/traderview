//! Multi-jurisdictional residential rental pesticide
//! application notification framework. Trader-landlord
//! critical because pesticide application is routine
//! maintenance (ant colonies, cockroach extermination,
//! termite treatment, mosquito spraying, rodent bait) but
//! failure to satisfy state pre-application notice
//! requirements creates per-violation civil exposure PLUS
//! tenant common-law tort claims for chemical exposure
//! injuries (especially when a tenant has chemical
//! sensitivity, asthma, or pregnancy that the landlord
//! knew or should have known about).
//!
//! Companion to rental_carbon_monoxide_detector,
//! rental_organic_waste_collection_disclosure,
//! rental_basement_water_intrusion_disclosure,
//! landlord_emergency_entry_notice.
//!
//! **California Civ. Code § 1940.8.5** (SB 328, approved
//! September 8, 2015, **effective January 1, 2016**) —
//! pre-application notice required when pesticides applied
//! to dwelling unit by landlord or authorized agent
//! (closes gap with structural pest control operators
//! already covered by Cal. Bus. & Prof. Code § 8538).
//!
//! § 1940.8.5(a) Definitions: includes pesticide, active
//! ingredient, dwelling unit, authorized agent.
//!
//! § 1940.8.5(b) Required notice content:
//! 1. Pesticide product brand name AND active ingredient;
//! 2. Scheduled date and time of application;
//! 3. Statement that pesticide will be applied with
//!    reasonable application; AND
//! 4. Statement of label warnings (poison information).
//!
//! § 1940.8.5(c) **24-hour pre-application notice
//! requirement** — at least 24 HOURS prior to application,
//! landlord or authorized agent SHALL provide notice to
//! tenant of dwelling unit AND to tenants in adjacent
//! units required to be notified, in at least ONE of:
//! 1. First-class mail;
//! 2. Personal delivery to tenant OR someone of suitable
//!    age and discretion at the premises;
//! 3. Delivery under the usual entry door of the premises;
//! 4. Electronic delivery (if email provided by tenant);
//!    OR
//! 5. Posting written notice in a conspicuous place at
//!    the unit entry.
//!
//! § 1940.8.5(d) **Emergency/health exception** — if pest
//! poses an IMMEDIATE THREAT to health and safety making
//! 24-hour prior notice unreasonable, landlord shall post
//! notification AS SOON AS PRACTICABLE but **not later
//! than ONE HOUR after the pesticide is applied**.
//!
//! § 1940.8.5(e) **Tenant-requested oral agreement
//! exception** — prior to receipt of written notification,
//! tenant and landlord MAY AGREE ORALLY to immediate
//! application if tenant requests pesticide be applied
//! before 24-hour advance notice can be given; oral
//! agreement must include pesticide product brand name.
//!
//! **New York ECL 33-1004 + 33-1005 + 33-0903 + 33-0905**
//! (Pesticide Neighbor Notification Law) — covers schools,
//! daycare centers, commercial lawn application,
//! one-or-two-family residential dwellings.
//!
//! ECL 33-1004 residential **48-hour requirements**:
//! 1. Commercial lawn applications to one-or-two-family
//!    residences: applicator must give occupants written
//!    copy of pesticide label info BEFORE application.
//! 2. Visual notification markers (signs/flags) required
//!    for residential lawn applications.
//! 3. **2020 amendment**: 48-hour notice for commercial
//!    lawn applications must be in BOTH English AND
//!    Spanish (plus any other language Commissioner deems
//!    necessary).
//!
//! Schools: 48 HOURS prior to pesticide application to
//! registered staff/parents.
//!
//! Daycare: 48 HOURS pre-application notice POSTED in
//! common area visible to those dropping off/picking up
//! children.
//!
//! **Massachusetts G.L. c. 132B § 6F + § 6I** — Children
//! and Families Protection Act of 2000 — **48-hour notice
//! to occupants** of indoor pesticide application + visual
//! notification markers + posted in common area;
//! mandatory IPM (Integrated Pest Management) plans for
//! schools and daycare facilities.
//!
//! **Default — Federal Worker Protection Standard (40 CFR
//! Part 170) and EPA labels** — federal floor applies
//! only to agricultural applications and licensed
//! commercial applicators. State landlord-tenant pesticide
//! disclosure regimes vary widely; absence of statute does
//! not preclude tenant common-law negligence claims for
//! chemical exposure injuries.
//!
//! Citations: Cal. Civ. Code § 1940.8.5 (SB 328 of 2015);
//! Cal. Bus. & Prof. Code § 8538 (structural pest control
//! operators); NY ECL § 33-1004 + § 33-1005; NY ECL
//! § 33-0903 + § 33-0905; Mass. G.L. c. 132B § 6F and
//! § 6I; 40 CFR Part 170 (federal Worker Protection
//! Standard); EPA FIFRA 7 USC § 136 et seq.

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
pub enum ApplicationContext {
    /// Routine pesticide application by landlord or
    /// authorized agent.
    RoutineApplication,
    /// Pest poses IMMEDIATE THREAT to health and safety
    /// (§ 1940.8.5(d) emergency exception engages).
    ImmediateHealthThreat,
    /// Tenant orally requested immediate application
    /// before 24-hour advance notice (§ 1940.8.5(e)
    /// tenant-requested exception engages).
    TenantRequestedImmediateApplication,
    /// Application to common areas (less stringent
    /// requirements may apply).
    CommonAreaApplication,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalPesticideApplicationNotificationInput {
    pub jurisdiction: Jurisdiction,
    pub context: ApplicationContext,
    /// Hours BEFORE pesticide application that notice
    /// was given to tenant (0 = no advance notice).
    pub hours_before_application: u32,
    /// Hours AFTER pesticide application that emergency
    /// post-notice was provided (0 = no post-notice).
    pub hours_after_application_post_notice: u32,
    /// Whether notice included required content (brand
    /// name + active ingredient + date/time + warnings).
    pub notice_content_complete: bool,
    /// Whether oral agreement (CA § 1940.8.5(e)) included
    /// pesticide product brand name (required element).
    pub oral_agreement_includes_brand_name: bool,
    /// Whether notice was provided in required languages
    /// (NY 2020 amendment requires English + Spanish for
    /// commercial lawn applications).
    pub multilingual_notice_provided: bool,
    /// Whether visual notification markers (signs/flags)
    /// were posted at application site (NY ECL 33-1004
    /// residential lawn requirement).
    pub visual_markers_posted: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalPesticideApplicationNotificationResult {
    pub jurisdiction: Jurisdiction,
    pub required_notice_hours: u32,
    pub notice_timing_compliant: bool,
    pub notice_content_compliant: bool,
    pub emergency_exception_engaged: bool,
    pub tenant_oral_agreement_exception_engaged: bool,
    pub overall_compliant: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(
    input: &RentalPesticideApplicationNotificationInput,
) -> RentalPesticideApplicationNotificationResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let required_notice_hours: u32 = match input.jurisdiction {
        Jurisdiction::California => 24,
        Jurisdiction::NewYork => 48,
        Jurisdiction::Massachusetts => 48,
        Jurisdiction::Default => 0,
    };

    let emergency_exception_engaged =
        matches!(input.context, ApplicationContext::ImmediateHealthThreat);
    let tenant_oral_agreement_exception_engaged = matches!(
        input.context,
        ApplicationContext::TenantRequestedImmediateApplication
    );

    let notice_timing_compliant = if emergency_exception_engaged {
        input.hours_after_application_post_notice <= 1
    } else if tenant_oral_agreement_exception_engaged {
        input.oral_agreement_includes_brand_name
    } else {
        input.hours_before_application >= required_notice_hours
    };

    let notice_content_compliant = input.notice_content_complete
        || tenant_oral_agreement_exception_engaged
        || emergency_exception_engaged;

    let mut overall_compliant = notice_timing_compliant && notice_content_compliant;

    if input.jurisdiction == Jurisdiction::NewYork && !input.multilingual_notice_provided {
        overall_compliant = false;
        failure_reasons.push(
            "NY ECL 33-1004 (2020 amendment) — 48-hour notice for commercial lawn applications must be in BOTH English AND Spanish (plus any other language Commissioner deems necessary)".to_string(),
        );
    }

    if input.jurisdiction == Jurisdiction::NewYork && !input.visual_markers_posted {
        overall_compliant = false;
        failure_reasons.push(
            "NY ECL 33-1004 — visual notification markers (signs/flags) required for residential lawn pesticide applications at application site".to_string(),
        );
    }

    if !notice_timing_compliant {
        match input.jurisdiction {
            Jurisdiction::California
                if !emergency_exception_engaged && !tenant_oral_agreement_exception_engaged =>
            {
                failure_reasons.push(format!(
                    "Cal. Civ. Code § 1940.8.5(c) — at least 24 HOURS prior to application, landlord or authorized agent SHALL provide notice to tenant via first-class mail / personal delivery / under-door delivery / electronic / posted notice; received {} hours advance notice",
                    input.hours_before_application
                ));
            }
            Jurisdiction::California if emergency_exception_engaged => {
                failure_reasons.push(format!(
                    "Cal. Civ. Code § 1940.8.5(d) — emergency post-notice must be provided AS SOON AS PRACTICABLE but NOT LATER THAN ONE HOUR after pesticide is applied; {} hours post-application elapsed",
                    input.hours_after_application_post_notice
                ));
            }
            Jurisdiction::California if tenant_oral_agreement_exception_engaged => {
                failure_reasons.push(
                    "Cal. Civ. Code § 1940.8.5(e) — tenant-requested oral agreement must INCLUDE PESTICIDE PRODUCT BRAND NAME; oral agreement without brand name is non-compliant".to_string(),
                );
            }
            Jurisdiction::NewYork => {
                failure_reasons.push(format!(
                    "NY ECL 33-1004 + 33-1005 — 48-HOUR pre-application notice required (commercial lawn applications + residential lawn applications + schools + daycare facilities); received {} hours advance notice",
                    input.hours_before_application
                ));
            }
            Jurisdiction::Massachusetts => {
                failure_reasons.push(format!(
                    "Mass. G.L. c. 132B § 6F + § 6I (Children and Families Protection Act of 2000) — 48-HOUR notice to occupants of indoor pesticide application + visual notification markers + common area posting; received {} hours advance notice",
                    input.hours_before_application
                ));
            }
            Jurisdiction::Default => {
                failure_reasons.push(
                    "Default — Federal Worker Protection Standard (40 CFR Part 170) and EPA pesticide labels apply only to agricultural applications + licensed commercial applicators; state landlord-tenant pesticide disclosure varies widely; absence of statute does NOT preclude common-law negligence claims for chemical exposure injuries".to_string(),
                );
            }
            Jurisdiction::California => {}
        }
    }

    if !notice_content_compliant
        && !emergency_exception_engaged
        && !tenant_oral_agreement_exception_engaged
    {
        match input.jurisdiction {
            Jurisdiction::California => {
                failure_reasons.push(
                    "Cal. Civ. Code § 1940.8.5(b) — required notice content: pesticide product brand name AND active ingredient + scheduled date and time of application + statement of reasonable application + statement of label warnings (poison information)".to_string(),
                );
            }
            Jurisdiction::NewYork => {
                failure_reasons.push(
                    "NY ECL 33-1004 — applicator must give occupants written copy of pesticide label information BEFORE residential one-or-two-family dwelling application".to_string(),
                );
            }
            Jurisdiction::Massachusetts => {
                failure_reasons.push(
                    "Mass. G.L. c. 132B § 6F + § 6I — written notice content: pesticide product identity + active ingredients + EPA registration number + application area + application date and time".to_string(),
                );
            }
            Jurisdiction::Default => {}
        }
    }

    let notes: Vec<String> = vec![
        "Cal. Civ. Code § 1940.8.5(c) (SB 328 of 2015, effective January 1, 2016) — at least 24 HOURS prior to pesticide application, landlord or authorized agent SHALL provide written notice to tenant of dwelling unit AND adjacent units via first-class mail OR personal delivery OR delivery under entry door OR electronic delivery (if email provided) OR posting in conspicuous place".to_string(),
        "Cal. Civ. Code § 1940.8.5(b) required content: pesticide product brand name AND active ingredient + scheduled date and time of application + reasonable application statement + statement of label warnings (poison information)".to_string(),
        "Cal. Civ. Code § 1940.8.5(d) emergency exception — if pest poses IMMEDIATE THREAT to health and safety making 24-hour prior notice unreasonable, landlord shall post notification AS SOON AS PRACTICABLE but NOT LATER THAN ONE HOUR after pesticide is applied".to_string(),
        "Cal. Civ. Code § 1940.8.5(e) tenant-requested oral agreement exception — tenant and landlord MAY AGREE ORALLY to immediate application if tenant requests application before 24-hour advance notice; oral agreement must include PESTICIDE PRODUCT BRAND NAME".to_string(),
        "NY ECL 33-1004 + 33-1005 (Pesticide Neighbor Notification Law) — 48-HOUR pre-application notice for commercial lawn applications + visual notification markers (signs/flags) at residential lawn application sites + written copy of pesticide label for one-or-two-family residential dwellings".to_string(),
        "NY ECL 33-1004 2020 amendment — 48-hour notice for commercial lawn applications must be in BOTH ENGLISH AND SPANISH (plus any other language Commissioner deems necessary)".to_string(),
        "NY ECL — schools require 48-HOUR notice to registered staff/parents; daycare facilities require 48-HOUR pre-application notice POSTED in common area visible to those dropping off/picking up children".to_string(),
        "Mass. G.L. c. 132B § 6F + § 6I (Children and Families Protection Act of 2000) — 48-HOUR notice to occupants of indoor pesticide application + visual notification markers + posted in common area; mandatory IPM (Integrated Pest Management) plans for schools and daycare facilities".to_string(),
        "Default — Federal Worker Protection Standard (40 CFR Part 170) and EPA pesticide labels apply only to agricultural applications + licensed commercial applicators; state landlord-tenant pesticide disclosure regimes vary widely; absence of statute does NOT preclude tenant common-law negligence claims for chemical exposure injuries".to_string(),
        "Cal. Bus. & Prof. Code § 8538 — structural pest control operators (licensed companies) have parallel pre-application notice obligations under SPCB regulations; § 1940.8.5 closes the gap for landlord-applied pesticides".to_string(),
        "EPA FIFRA 7 USC § 136 et seq. — federal Insecticide, Fungicide, and Rodenticide Act preempts state pesticide labeling but PRESERVES state notice and disclosure requirements for landlord-tenant relationships".to_string(),
    ];

    RentalPesticideApplicationNotificationResult {
        jurisdiction: input.jurisdiction,
        required_notice_hours,
        notice_timing_compliant,
        notice_content_compliant,
        emergency_exception_engaged,
        tenant_oral_agreement_exception_engaged,
        overall_compliant,
        failure_reasons,
        citation: "Cal. Civ. Code § 1940.8.5 (SB 328 of 2015, effective January 1, 2016); Cal. Bus. & Prof. Code § 8538; NY ECL § 33-1004 and § 33-1005; NY ECL § 33-0903 and § 33-0905; Mass. G.L. c. 132B § 6F and § 6I (Children and Families Protection Act of 2000); 40 CFR Part 170 (federal Worker Protection Standard); EPA FIFRA 7 USC § 136",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_routine_compliant() -> RentalPesticideApplicationNotificationInput {
        RentalPesticideApplicationNotificationInput {
            jurisdiction: Jurisdiction::California,
            context: ApplicationContext::RoutineApplication,
            hours_before_application: 24,
            hours_after_application_post_notice: 0,
            notice_content_complete: true,
            oral_agreement_includes_brand_name: true,
            multilingual_notice_provided: true,
            visual_markers_posted: true,
        }
    }

    #[test]
    fn ca_24_hour_advance_notice_compliant() {
        let r = check(&ca_routine_compliant());
        assert!(r.overall_compliant);
        assert!(r.notice_timing_compliant);
        assert_eq!(r.required_notice_hours, 24);
    }

    #[test]
    fn ca_23_hour_advance_notice_violation() {
        let mut i = ca_routine_compliant();
        i.hours_before_application = 23;
        let r = check(&i);
        assert!(!r.notice_timing_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 1940.8.5(c)") && f.contains("24 HOURS")));
    }

    #[test]
    fn ca_missing_content_violation() {
        let mut i = ca_routine_compliant();
        i.notice_content_complete = false;
        let r = check(&i);
        assert!(!r.notice_content_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("§ 1940.8.5(b)")
            && f.contains("brand name")
            && f.contains("active ingredient")
            && f.contains("warnings")));
    }

    #[test]
    fn ca_emergency_within_one_hour_post_notice_compliant() {
        let mut i = ca_routine_compliant();
        i.context = ApplicationContext::ImmediateHealthThreat;
        i.hours_before_application = 0;
        i.hours_after_application_post_notice = 1;
        let r = check(&i);
        assert!(r.emergency_exception_engaged);
        assert!(r.notice_timing_compliant);
    }

    #[test]
    fn ca_emergency_post_notice_2_hours_violation() {
        let mut i = ca_routine_compliant();
        i.context = ApplicationContext::ImmediateHealthThreat;
        i.hours_before_application = 0;
        i.hours_after_application_post_notice = 2;
        let r = check(&i);
        assert!(r.emergency_exception_engaged);
        assert!(!r.notice_timing_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 1940.8.5(d)") && f.contains("ONE HOUR")));
    }

    #[test]
    fn ca_tenant_oral_agreement_with_brand_name_compliant() {
        let mut i = ca_routine_compliant();
        i.context = ApplicationContext::TenantRequestedImmediateApplication;
        i.hours_before_application = 0;
        i.oral_agreement_includes_brand_name = true;
        let r = check(&i);
        assert!(r.tenant_oral_agreement_exception_engaged);
        assert!(r.notice_timing_compliant);
    }

    #[test]
    fn ca_tenant_oral_agreement_without_brand_name_violation() {
        let mut i = ca_routine_compliant();
        i.context = ApplicationContext::TenantRequestedImmediateApplication;
        i.hours_before_application = 0;
        i.oral_agreement_includes_brand_name = false;
        let r = check(&i);
        assert!(!r.notice_timing_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 1940.8.5(e)") && f.contains("PESTICIDE PRODUCT BRAND NAME")));
    }

    #[test]
    fn ny_48_hour_advance_notice_compliant() {
        let mut i = ca_routine_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.hours_before_application = 48;
        let r = check(&i);
        assert!(r.notice_timing_compliant);
        assert_eq!(r.required_notice_hours, 48);
    }

    #[test]
    fn ny_47_hour_advance_notice_violation() {
        let mut i = ca_routine_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.hours_before_application = 47;
        let r = check(&i);
        assert!(!r.notice_timing_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("NY ECL 33-1004") && f.contains("48-HOUR")));
    }

    #[test]
    fn ny_no_multilingual_notice_violation() {
        let mut i = ca_routine_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.hours_before_application = 48;
        i.multilingual_notice_provided = false;
        let r = check(&i);
        assert!(!r.overall_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("ECL 33-1004")
            && f.contains("2020 amendment")
            && f.contains("English AND Spanish")));
    }

    #[test]
    fn ny_no_visual_markers_violation() {
        let mut i = ca_routine_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.hours_before_application = 48;
        i.visual_markers_posted = false;
        let r = check(&i);
        assert!(!r.overall_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("ECL 33-1004") && f.contains("visual notification markers")));
    }

    #[test]
    fn ma_48_hour_advance_notice_compliant() {
        let mut i = ca_routine_compliant();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.hours_before_application = 48;
        let r = check(&i);
        assert!(r.notice_timing_compliant);
        assert_eq!(r.required_notice_hours, 48);
    }

    #[test]
    fn ma_under_48_hours_violation() {
        let mut i = ca_routine_compliant();
        i.jurisdiction = Jurisdiction::Massachusetts;
        i.hours_before_application = 47;
        let r = check(&i);
        assert!(!r.notice_timing_compliant);
        assert!(r.failure_reasons.iter().any(|f| f.contains("c. 132B § 6F")
            && f.contains("48-HOUR")
            && f.contains("Children and Families Protection Act")));
    }

    #[test]
    fn default_no_advance_notice_compliant_but_warning_engaged() {
        let mut i = ca_routine_compliant();
        i.jurisdiction = Jurisdiction::Default;
        i.hours_before_application = 0;
        let r = check(&i);
        assert!(r.notice_timing_compliant);
        assert_eq!(r.required_notice_hours, 0);
    }

    #[test]
    fn jurisdiction_truth_table_four_cells() {
        for (jur, exp_hours) in [
            (Jurisdiction::California, 24),
            (Jurisdiction::NewYork, 48),
            (Jurisdiction::Massachusetts, 48),
            (Jurisdiction::Default, 0),
        ] {
            let mut i = ca_routine_compliant();
            i.jurisdiction = jur;
            let r = check(&i);
            assert_eq!(r.required_notice_hours, exp_hours, "jur={:?}", jur);
        }
    }

    #[test]
    fn ny_ma_share_48_hour_standard_invariant() {
        let mut ny = ca_routine_compliant();
        ny.jurisdiction = Jurisdiction::NewYork;
        let mut ma = ca_routine_compliant();
        ma.jurisdiction = Jurisdiction::Massachusetts;
        let r_ny = check(&ny);
        let r_ma = check(&ma);
        assert_eq!(r_ny.required_notice_hours, r_ma.required_notice_hours);
        assert_eq!(r_ny.required_notice_hours, 48);
    }

    #[test]
    fn ca_uniquely_provides_24_hour_standard_invariant() {
        let mut ca = ca_routine_compliant();
        ca.jurisdiction = Jurisdiction::California;
        let r_ca = check(&ca);
        for jur in [Jurisdiction::NewYork, Jurisdiction::Massachusetts] {
            let mut i = ca_routine_compliant();
            i.jurisdiction = jur;
            let r = check(&i);
            assert!(
                r_ca.required_notice_hours < r.required_notice_hours,
                "jur={:?}",
                jur
            );
        }
    }

    #[test]
    fn ca_emergency_only_engages_with_immediate_threat() {
        let r = check(&ca_routine_compliant());
        assert!(!r.emergency_exception_engaged);

        let mut emerg = ca_routine_compliant();
        emerg.context = ApplicationContext::ImmediateHealthThreat;
        let r_emerg = check(&emerg);
        assert!(r_emerg.emergency_exception_engaged);
    }

    #[test]
    fn ca_oral_agreement_only_engages_with_tenant_request() {
        let r = check(&ca_routine_compliant());
        assert!(!r.tenant_oral_agreement_exception_engaged);

        let mut oral = ca_routine_compliant();
        oral.context = ApplicationContext::TenantRequestedImmediateApplication;
        let r_oral = check(&oral);
        assert!(r_oral.tenant_oral_agreement_exception_engaged);
    }

    #[test]
    fn citation_pins_all_four_jurisdictions() {
        let r = check(&ca_routine_compliant());
        assert!(r.citation.contains("Cal. Civ. Code § 1940.8.5"));
        assert!(r.citation.contains("SB 328 of 2015"));
        assert!(r.citation.contains("January 1, 2016"));
        assert!(r.citation.contains("Cal. Bus. & Prof. Code § 8538"));
        assert!(r.citation.contains("NY ECL § 33-1004 and § 33-1005"));
        assert!(r.citation.contains("Mass. G.L. c. 132B § 6F and § 6I"));
        assert!(r
            .citation
            .contains("Children and Families Protection Act of 2000"));
        assert!(r.citation.contains("40 CFR Part 170"));
        assert!(r.citation.contains("EPA FIFRA 7 USC § 136"));
    }

    #[test]
    fn note_pins_ca_24_hour_five_delivery_methods() {
        let r = check(&ca_routine_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1940.8.5(c)")
            && n.contains("24 HOURS")
            && n.contains("first-class mail")
            && n.contains("electronic delivery")
            && n.contains("conspicuous place")));
    }

    #[test]
    fn note_pins_ca_required_content() {
        let r = check(&ca_routine_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1940.8.5(b)")
            && n.contains("brand name")
            && n.contains("active ingredient")
            && n.contains("warnings")));
    }

    #[test]
    fn note_pins_ca_emergency_exception_one_hour() {
        let r = check(&ca_routine_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1940.8.5(d)")
            && n.contains("IMMEDIATE THREAT")
            && n.contains("ONE HOUR after pesticide is applied")));
    }

    #[test]
    fn note_pins_ca_tenant_oral_agreement_brand_name() {
        let r = check(&ca_routine_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 1940.8.5(e)")
            && n.contains("oral agreement")
            && n.contains("PESTICIDE PRODUCT BRAND NAME")));
    }

    #[test]
    fn note_pins_ny_ecl_48_hour_visual_markers() {
        let r = check(&ca_routine_compliant());
        assert!(r.notes.iter().any(|n| n.contains("NY ECL 33-1004")
            && n.contains("48-HOUR")
            && n.contains("visual notification markers")));
    }

    #[test]
    fn note_pins_ny_2020_amendment_english_spanish() {
        let r = check(&ca_routine_compliant());
        assert!(
            r.notes
                .iter()
                .any(|n| n.contains("ECL 33-1004 2020 amendment")
                    && n.contains("ENGLISH AND SPANISH"))
        );
    }

    #[test]
    fn note_pins_ny_school_daycare_48_hour_registry() {
        let r = check(&ca_routine_compliant());
        assert!(r.notes.iter().any(|n| n.contains("NY ECL")
            && n.contains("schools require 48-HOUR")
            && n.contains("daycare")));
    }

    #[test]
    fn note_pins_ma_ipm_children_families_protection() {
        let r = check(&ca_routine_compliant());
        assert!(r.notes.iter().any(|n| n.contains("c. 132B § 6F")
            && n.contains("§ 6I")
            && n.contains("Children and Families Protection Act of 2000")
            && n.contains("IPM (Integrated Pest Management)")));
    }

    #[test]
    fn note_pins_default_federal_floor() {
        let r = check(&ca_routine_compliant());
        assert!(r.notes.iter().any(|n| n.contains("Default")
            && n.contains("40 CFR Part 170")
            && n.contains("agricultural applications")
            && n.contains("common-law negligence")));
    }

    #[test]
    fn note_pins_ca_spcb_8538_parallel() {
        let r = check(&ca_routine_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Cal. Bus. & Prof. Code § 8538")
                && n.contains("structural pest control operators")
                && n.contains("SPCB")));
    }

    #[test]
    fn note_pins_fifra_federal_preemption_carveout() {
        let r = check(&ca_routine_compliant());
        assert!(r.notes.iter().any(|n| n.contains("EPA FIFRA 7 USC § 136")
            && n.contains("PRESERVES state notice")
            && n.contains("landlord-tenant")));
    }

    #[test]
    fn defensive_zero_hours_routine_non_compliant() {
        let mut i = ca_routine_compliant();
        i.hours_before_application = 0;
        let r = check(&i);
        assert!(!r.notice_timing_compliant);
    }

    #[test]
    fn multiple_failures_stack() {
        let mut i = ca_routine_compliant();
        i.jurisdiction = Jurisdiction::NewYork;
        i.hours_before_application = 20;
        i.multilingual_notice_provided = false;
        i.visual_markers_posted = false;
        let r = check(&i);
        assert!(r.failure_reasons.len() >= 3);
    }
}
