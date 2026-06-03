//! Multi-jurisdiction soft-story / wood-frame seismic retrofit
//! compliance for trader-landlords with California multifamily
//! property inventory.
//!
//! Two major California jurisdictions pinned with statutory citations
//! and the operative deadlines + property-class triggers:
//!
//! - **Los Angeles — Ordinance 183893 (2015)**: "Earthquake Hazard
//!   Reduction in Existing Wood-Frame Buildings with Soft, Weak, or
//!   Open-Front Walls" — the most ambitious mandatory seismic
//!   retrofit program in the United States, covering an estimated
//!   13,500 soft-story buildings citywide. **Priority 1** buildings
//!   (3+ stories with ground-floor commercial occupancy) had a
//!   compliance deadline of **April 2024**. **Priority 2** buildings
//!   (remaining smaller wood-frame structures under 16 units,
//!   typically 2-3 stories with ground-floor parking / tuck-under
//!   garages) have a deadline of **April 2026**.
//!
//! - **San Francisco — Chapter 34B + Chapter 4D + Chapter 5E** of
//!   the SF Building Code (signed April 18, 2013; operative June 17,
//!   2013): covers wood-frame buildings of 3+ stories (or 2 stories
//!   over basement), containing 5+ dwelling units, built before
//!   January 1, 1978. **Tier 1** (highest risk): Sept 15, 2017.
//!   **Tier 2**: Sept 15, 2018. **Tier 3**: Sept 15, 2019. **Tier 4**:
//!   Sept 15, 2020. As of September 15, 2021, all deadlines for all
//!   tiers have PASSED — non-compliance generates ongoing daily
//!   civil-penalty exposure with no further extension available.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const LA_ORDINANCE_NUMBER: u32 = 183893;
#[allow(dead_code)]
pub const LA_ORDINANCE_ENACTMENT_YEAR: u32 = 2015;
#[allow(dead_code)]
pub const LA_PRIORITY_1_DEADLINE_YEAR: u32 = 2024;
#[allow(dead_code)]
pub const LA_PRIORITY_2_DEADLINE_YEAR: u32 = 2026;
#[allow(dead_code)]
pub const LA_PRIORITY_1_MIN_STORIES: u32 = 3;
#[allow(dead_code)]
pub const LA_PRIORITY_2_UNIT_THRESHOLD: u32 = 16;
#[allow(dead_code)]
pub const LA_TOTAL_COVERED_BUILDINGS_ESTIMATE: u32 = 13_500;
#[allow(dead_code)]
pub const SF_CHAPTER_34B_OPERATIVE_YEAR: u32 = 2013;
#[allow(dead_code)]
pub const SF_PRE_1978_THRESHOLD_YEAR: u32 = 1978;
#[allow(dead_code)]
pub const SF_MIN_UNITS_THRESHOLD: u32 = 5;
#[allow(dead_code)]
pub const SF_MIN_STORIES_THRESHOLD: u32 = 3;
#[allow(dead_code)]
pub const SF_FINAL_DEADLINE_YEAR: u32 = 2021;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    LosAngelesOrdinance183893,
    SanFranciscoChapter34B,
    DefaultNoMandatoryRetrofitRegime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetrofitStatus {
    NotStarted,
    PlansFiled,
    PermitsObtained,
    ConstructionUnderway,
    CompleteAndCertified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    ExemptBuildingNotWoodFrame,
    ExemptStoryCountBelowJurisdictionThreshold,
    ExemptUnitCountBelowJurisdictionThreshold,
    ExemptPostThresholdYearConstruction,
    CompliantRetrofitCompleteAndCertified,
    CompliantRetrofitInProgressBeforeDeadline,
    ViolationDeadlinePastedNoRetrofitCommenced,
    ViolationDeadlinePastedRetrofitIncomplete,
    DefaultJurisdictionNoMandatoryRegime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub jurisdiction: Jurisdiction,
    pub building_year_built: u32,
    pub is_wood_frame: bool,
    pub story_count: u32,
    pub dwelling_unit_count: u32,
    pub has_ground_floor_commercial: bool,
    pub current_year: u32,
    pub retrofit_status: RetrofitStatus,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub applicable_deadline_year: u32,
    pub years_past_deadline: i32,
    pub retrofit_required: bool,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type SoftStorySeismicRetrofitInput = Input;
pub type SoftStorySeismicRetrofitOutput = Output;
pub type SoftStorySeismicRetrofitResult = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "LA Ordinance 183893 (2015) — Earthquake Hazard Reduction in Existing Wood-Frame Buildings with Soft, Weak, or Open-Front Walls".to_string(),
        "LA Municipal Code Division 93 (codifies Ordinance 183893)".to_string(),
        "SF Building Code Chapter 34B (2013) — Mandatory Seismic Retrofit Program (Wood-Frame Buildings)".to_string(),
        "SF Building Code Chapter 4D + Chapter 5E (companion seismic chapters)".to_string(),
        "LADBS Soft-Story Retrofit Program (City of Los Angeles)".to_string(),
        "SF Earthquake Safety Implementation Program (ESIP)".to_string(),
        "California Building Code (CBC) Chapter 34 (seismic standards reference)".to_string(),
    ];

    if matches!(
        input.jurisdiction,
        Jurisdiction::DefaultNoMandatoryRetrofitRegime
    ) {
        notes.push("Jurisdiction has no statutory mandatory soft-story retrofit regime; common-law habitability + insurance-driven retrofits only.".to_string());
        return Output {
            severity: Severity::DefaultJurisdictionNoMandatoryRegime,
            applicable_deadline_year: 0,
            years_past_deadline: 0,
            retrofit_required: false,
            notes,
            citations,
        };
    }

    if !input.is_wood_frame {
        notes.push("Building is not wood-frame construction — outside scope of LA Ordinance 183893 and SF Chapter 34B (which target wood-frame soft-story specifically).".to_string());
        return Output {
            severity: Severity::ExemptBuildingNotWoodFrame,
            applicable_deadline_year: 0,
            years_past_deadline: 0,
            retrofit_required: false,
            notes,
            citations,
        };
    }

    match input.jurisdiction {
        Jurisdiction::LosAngelesOrdinance183893 => check_la(input, &mut notes, citations),
        Jurisdiction::SanFranciscoChapter34B => check_sf(input, &mut notes, citations),
        Jurisdiction::DefaultNoMandatoryRetrofitRegime => unreachable!(),
    }
}

fn check_la(input: &Input, notes: &mut Vec<String>, citations: Vec<String>) -> Output {
    if input.story_count < 2 {
        notes.push(format!(
            "Story count {} below LA Ordinance 183893 minimum threshold (typically 2+ stories with ground-floor opening).",
            input.story_count
        ));
        return Output {
            severity: Severity::ExemptStoryCountBelowJurisdictionThreshold,
            applicable_deadline_year: 0,
            years_past_deadline: 0,
            retrofit_required: false,
            notes: notes.clone(),
            citations,
        };
    }

    let is_priority_1 = input.story_count >= LA_PRIORITY_1_MIN_STORIES
        && input.has_ground_floor_commercial;
    let deadline_year = if is_priority_1 {
        LA_PRIORITY_1_DEADLINE_YEAR
    } else {
        LA_PRIORITY_2_DEADLINE_YEAR
    };

    let years_past: i32 = input.current_year as i32 - deadline_year as i32;
    let deadline_passed = input.current_year > deadline_year;

    if matches!(input.retrofit_status, RetrofitStatus::CompleteAndCertified) {
        notes.push(format!(
            "LA Ordinance 183893 — retrofit complete and certified. Priority {} deadline {}.",
            if is_priority_1 { 1 } else { 2 },
            deadline_year
        ));
        return Output {
            severity: Severity::CompliantRetrofitCompleteAndCertified,
            applicable_deadline_year: deadline_year,
            years_past_deadline: years_past,
            retrofit_required: true,
            notes: notes.clone(),
            citations,
        };
    }

    if !deadline_passed {
        let in_progress = !matches!(input.retrofit_status, RetrofitStatus::NotStarted);
        if in_progress {
            notes.push(format!(
                "LA Ordinance 183893 — Priority {} retrofit in progress; deadline {}.",
                if is_priority_1 { 1 } else { 2 },
                deadline_year
            ));
            return Output {
                severity: Severity::CompliantRetrofitInProgressBeforeDeadline,
                applicable_deadline_year: deadline_year,
                years_past_deadline: years_past,
                retrofit_required: true,
                notes: notes.clone(),
                citations,
            };
        }
        notes.push(format!(
            "LA Ordinance 183893 — Priority {} retrofit not yet commenced; deadline {} remains in future.",
            if is_priority_1 { 1 } else { 2 },
            deadline_year
        ));
        return Output {
            severity: Severity::CompliantRetrofitInProgressBeforeDeadline,
            applicable_deadline_year: deadline_year,
            years_past_deadline: years_past,
            retrofit_required: true,
            notes: notes.clone(),
            citations,
        };
    }

    let severity = if matches!(input.retrofit_status, RetrofitStatus::NotStarted) {
        notes.push(format!(
            "LA Ordinance 183893 — Priority {} deadline {} PASSED ({} years ago); no retrofit commenced. Daily civil penalties + LADBS enforcement.",
            if is_priority_1 { 1 } else { 2 },
            deadline_year,
            years_past
        ));
        Severity::ViolationDeadlinePastedNoRetrofitCommenced
    } else {
        notes.push(format!(
            "LA Ordinance 183893 — Priority {} deadline {} PASSED ({} years ago); retrofit incomplete.",
            if is_priority_1 { 1 } else { 2 },
            deadline_year,
            years_past
        ));
        Severity::ViolationDeadlinePastedRetrofitIncomplete
    };

    Output {
        severity,
        applicable_deadline_year: deadline_year,
        years_past_deadline: years_past,
        retrofit_required: true,
        notes: notes.clone(),
        citations,
    }
}

fn check_sf(input: &Input, notes: &mut Vec<String>, citations: Vec<String>) -> Output {
    if input.building_year_built >= SF_PRE_1978_THRESHOLD_YEAR {
        notes.push(format!(
            "SF Chapter 34B exempts construction in or after {} — building year {} not covered.",
            SF_PRE_1978_THRESHOLD_YEAR, input.building_year_built
        ));
        return Output {
            severity: Severity::ExemptPostThresholdYearConstruction,
            applicable_deadline_year: 0,
            years_past_deadline: 0,
            retrofit_required: false,
            notes: notes.clone(),
            citations,
        };
    }

    if input.story_count < SF_MIN_STORIES_THRESHOLD {
        notes.push(format!(
            "Story count {} below SF Chapter 34B minimum threshold {} (or 2 stories over basement).",
            input.story_count, SF_MIN_STORIES_THRESHOLD
        ));
        return Output {
            severity: Severity::ExemptStoryCountBelowJurisdictionThreshold,
            applicable_deadline_year: 0,
            years_past_deadline: 0,
            retrofit_required: false,
            notes: notes.clone(),
            citations,
        };
    }

    if input.dwelling_unit_count < SF_MIN_UNITS_THRESHOLD {
        notes.push(format!(
            "Unit count {} below SF Chapter 34B minimum threshold {}.",
            input.dwelling_unit_count, SF_MIN_UNITS_THRESHOLD
        ));
        return Output {
            severity: Severity::ExemptUnitCountBelowJurisdictionThreshold,
            applicable_deadline_year: 0,
            years_past_deadline: 0,
            retrofit_required: false,
            notes: notes.clone(),
            citations,
        };
    }

    let years_past: i32 = input.current_year as i32 - SF_FINAL_DEADLINE_YEAR as i32;

    if matches!(input.retrofit_status, RetrofitStatus::CompleteAndCertified) {
        notes.push(format!(
            "SF Chapter 34B — retrofit complete and certified (all tier deadlines passed by {}).",
            SF_FINAL_DEADLINE_YEAR
        ));
        return Output {
            severity: Severity::CompliantRetrofitCompleteAndCertified,
            applicable_deadline_year: SF_FINAL_DEADLINE_YEAR,
            years_past_deadline: years_past,
            retrofit_required: true,
            notes: notes.clone(),
            citations,
        };
    }

    let severity = if matches!(input.retrofit_status, RetrofitStatus::NotStarted) {
        notes.push(format!(
            "SF Chapter 34B — all deadlines passed by {} ({} years ago); no retrofit commenced. Daily civil penalties + DBI enforcement.",
            SF_FINAL_DEADLINE_YEAR, years_past
        ));
        Severity::ViolationDeadlinePastedNoRetrofitCommenced
    } else {
        notes.push(format!(
            "SF Chapter 34B — all deadlines passed by {} ({} years ago); retrofit incomplete.",
            SF_FINAL_DEADLINE_YEAR, years_past
        ));
        Severity::ViolationDeadlinePastedRetrofitIncomplete
    };

    Output {
        severity,
        applicable_deadline_year: SF_FINAL_DEADLINE_YEAR,
        years_past_deadline: years_past,
        retrofit_required: true,
        notes: notes.clone(),
        citations,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_la_priority_2() -> Input {
        Input {
            jurisdiction: Jurisdiction::LosAngelesOrdinance183893,
            building_year_built: 1968,
            is_wood_frame: true,
            story_count: 2,
            dwelling_unit_count: 12,
            has_ground_floor_commercial: false,
            current_year: 2026,
            retrofit_status: RetrofitStatus::CompleteAndCertified,
        }
    }

    #[test]
    fn la_priority_2_complete_certified_compliant() {
        let out = check(&base_la_priority_2());
        assert_eq!(out.severity, Severity::CompliantRetrofitCompleteAndCertified);
        assert_eq!(out.applicable_deadline_year, 2026);
    }

    #[test]
    fn la_priority_1_ground_floor_commercial_3_plus_stories_deadline_2024() {
        let mut i = base_la_priority_2();
        i.story_count = 3;
        i.has_ground_floor_commercial = true;
        let out = check(&i);
        assert_eq!(out.applicable_deadline_year, 2024);
    }

    #[test]
    fn la_priority_1_deadline_2024_not_started_in_2026_is_violation() {
        let mut i = base_la_priority_2();
        i.story_count = 3;
        i.has_ground_floor_commercial = true;
        i.retrofit_status = RetrofitStatus::NotStarted;
        i.current_year = 2026;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationDeadlinePastedNoRetrofitCommenced);
        assert_eq!(out.years_past_deadline, 2);
    }

    #[test]
    fn la_priority_2_deadline_2026_in_progress_in_2025_compliant() {
        let mut i = base_la_priority_2();
        i.retrofit_status = RetrofitStatus::ConstructionUnderway;
        i.current_year = 2025;
        let out = check(&i);
        assert_eq!(out.severity, Severity::CompliantRetrofitInProgressBeforeDeadline);
    }

    #[test]
    fn la_priority_2_in_2027_not_started_violation() {
        let mut i = base_la_priority_2();
        i.retrofit_status = RetrofitStatus::NotStarted;
        i.current_year = 2027;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationDeadlinePastedNoRetrofitCommenced);
    }

    #[test]
    fn la_priority_2_in_2027_incomplete_violation() {
        let mut i = base_la_priority_2();
        i.retrofit_status = RetrofitStatus::PermitsObtained;
        i.current_year = 2027;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationDeadlinePastedRetrofitIncomplete);
    }

    #[test]
    fn la_not_wood_frame_exempt() {
        let mut i = base_la_priority_2();
        i.is_wood_frame = false;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptBuildingNotWoodFrame);
    }

    #[test]
    fn la_single_story_below_threshold_exempt() {
        let mut i = base_la_priority_2();
        i.story_count = 1;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptStoryCountBelowJurisdictionThreshold);
    }

    #[test]
    fn sf_chapter_34b_pre_1978_wood_3_stories_5_units_complete_compliant() {
        let mut i = base_la_priority_2();
        i.jurisdiction = Jurisdiction::SanFranciscoChapter34B;
        i.building_year_built = 1965;
        i.story_count = 3;
        i.dwelling_unit_count = 8;
        let out = check(&i);
        assert_eq!(out.severity, Severity::CompliantRetrofitCompleteAndCertified);
        assert_eq!(out.applicable_deadline_year, 2021);
    }

    #[test]
    fn sf_post_1978_building_exempt() {
        let mut i = base_la_priority_2();
        i.jurisdiction = Jurisdiction::SanFranciscoChapter34B;
        i.building_year_built = 1990;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptPostThresholdYearConstruction);
    }

    #[test]
    fn sf_below_5_units_exempt() {
        let mut i = base_la_priority_2();
        i.jurisdiction = Jurisdiction::SanFranciscoChapter34B;
        i.building_year_built = 1965;
        i.story_count = 3;
        i.dwelling_unit_count = 4;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptUnitCountBelowJurisdictionThreshold);
    }

    #[test]
    fn sf_below_3_stories_exempt() {
        let mut i = base_la_priority_2();
        i.jurisdiction = Jurisdiction::SanFranciscoChapter34B;
        i.building_year_built = 1965;
        i.story_count = 2;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ExemptStoryCountBelowJurisdictionThreshold);
    }

    #[test]
    fn sf_not_started_in_2026_violation_5_years_past_deadline() {
        let mut i = base_la_priority_2();
        i.jurisdiction = Jurisdiction::SanFranciscoChapter34B;
        i.building_year_built = 1965;
        i.story_count = 3;
        i.dwelling_unit_count = 8;
        i.retrofit_status = RetrofitStatus::NotStarted;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationDeadlinePastedNoRetrofitCommenced);
        assert_eq!(out.years_past_deadline, 5);
    }

    #[test]
    fn sf_incomplete_in_2026_violation() {
        let mut i = base_la_priority_2();
        i.jurisdiction = Jurisdiction::SanFranciscoChapter34B;
        i.building_year_built = 1965;
        i.story_count = 3;
        i.dwelling_unit_count = 8;
        i.retrofit_status = RetrofitStatus::ConstructionUnderway;
        let out = check(&i);
        assert_eq!(out.severity, Severity::ViolationDeadlinePastedRetrofitIncomplete);
    }

    #[test]
    fn default_jurisdiction_no_mandatory_regime() {
        let mut i = base_la_priority_2();
        i.jurisdiction = Jurisdiction::DefaultNoMandatoryRetrofitRegime;
        let out = check(&i);
        assert_eq!(out.severity, Severity::DefaultJurisdictionNoMandatoryRegime);
    }

    #[test]
    fn citations_pin_la_ordinance_and_sf_chapter() {
        let out = check(&base_la_priority_2());
        assert!(out.citations.iter().any(|c| c.contains("Ordinance 183893")));
        assert!(out.citations.iter().any(|c| c.contains("Chapter 34B")));
    }

    #[test]
    fn citations_pin_ladbs_and_esip_programs() {
        let out = check(&base_la_priority_2());
        assert!(out.citations.iter().any(|c| c.contains("LADBS")));
        assert!(out.citations.iter().any(|c| c.contains("ESIP")));
    }

    #[test]
    fn constant_pin_la_ordinance_183893() {
        assert_eq!(LA_ORDINANCE_NUMBER, 183893);
    }

    #[test]
    fn constant_pin_la_priority_1_deadline_2024() {
        assert_eq!(LA_PRIORITY_1_DEADLINE_YEAR, 2024);
    }

    #[test]
    fn constant_pin_la_priority_2_deadline_2026() {
        assert_eq!(LA_PRIORITY_2_DEADLINE_YEAR, 2026);
    }

    #[test]
    fn constant_pin_la_13500_buildings_estimate() {
        assert_eq!(LA_TOTAL_COVERED_BUILDINGS_ESTIMATE, 13_500);
    }

    #[test]
    fn constant_pin_sf_chapter_34b_operative_2013() {
        assert_eq!(SF_CHAPTER_34B_OPERATIVE_YEAR, 2013);
    }

    #[test]
    fn constant_pin_sf_pre_1978_threshold() {
        assert_eq!(SF_PRE_1978_THRESHOLD_YEAR, 1978);
    }

    #[test]
    fn constant_pin_sf_5_unit_minimum() {
        assert_eq!(SF_MIN_UNITS_THRESHOLD, 5);
    }

    #[test]
    fn constant_pin_sf_2021_final_deadline() {
        assert_eq!(SF_FINAL_DEADLINE_YEAR, 2021);
    }
}
