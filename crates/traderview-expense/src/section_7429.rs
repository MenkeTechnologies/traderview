//! IRC § 7429 — Review of jeopardy levy or assessment
//! procedures. Trader-relevant when IRS believes collection
//! is in jeopardy (taxpayer planning to flee, conceal
//! assets, dispose of assets to evade collection) and
//! invokes jeopardy assessment under § 6861 (income / estate
//! / gift tax) + § 6862 (other taxes) + § 7508A immediate
//! collection action. Trader-landlords face § 7429 review
//! when IRS makes jeopardy assessment seizing trading
//! accounts + rental property revenue + bank accounts.
//!
//! Procedural-companion to § 7421 (Anti-Injunction Act — §
//! 7429(b) is one of 11 statutory exceptions), § 7426
//! (third-party wrongful levy), § 7433 (civil damages for
//! unauthorized collection), § 7430 (litigation costs), §
//! 6321 (lien attachment), § 6323 (priority), § 6325
//! (release/discharge), § 6334 (exempt property), and §
//! 7508A (disaster postponement).
//!
//! **§ 7429(a) administrative review framework**:
//! - § 7429(a)(1) — IRS provides written statement within 5
//!   days of jeopardy assessment / levy explaining
//!   information relied upon.
//! - § 7429(a)(2) — taxpayer may request administrative
//!   review within **30 days** after written statement
//!   furnished (or after 5-day period expires).
//! - § 7429(a)(3) — IRS has **15 calendar days** to respond
//!   to administrative review request.
//!
//! **§ 7429(b) judicial review framework**:
//! - § 7429(b)(1) — taxpayer may file civil action in
//!   district court within **90 days** from earlier of:
//!   - Date district director notifies taxpayer of
//!     determination, OR
//!   - **16th day** after taxpayer's administrative review
//!     request was made.
//! - § 7429(b)(2) — district court has EXCLUSIVE
//!   jurisdiction (no Tax Court alternative).
//! - § 7429(b)(3) — court determines within **20 calendar
//!   days** whether: (1) making of assessment was
//!   REASONABLE under the circumstances, and (2) amount
//!   assessed is APPROPRIATE.
//!
//! **§ 7429(b)(3) extension** — taxpayer may request
//! extension of 20-day period and court may grant up to **40
//! additional calendar days** for reasonable grounds
//! (combined 60-day maximum).
//!
//! **§ 7429(c) extension of time** — Secretary may extend
//! deadlines in specific circumstances.
//!
//! Citations: 26 USC § 7429(a)(1)-(3), (b)(1)-(3), (c); 26
//! CFR § 301.7429-3; § 6861 (jeopardy assessment, income /
//! estate / gift); § 6862 (jeopardy assessment, other);
//! § 7421(a)(11) (AIA exception); IRM 5.1.4 (Jeopardy,
//! Termination, Quick, and Prompt Assessments); IRM 5.17.15
//! (Termination and Jeopardy Assessments and Jeopardy
//! Collection); IRM 8.24.2 (Jeopardy Levy Appeals).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReviewStage {
    /// § 7429(a) administrative review (within IRS Appeals).
    Administrative,
    /// § 7429(b) judicial review (district court).
    Judicial,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7429Input {
    pub review_stage: ReviewStage,
    /// Days since IRS provided written statement under §
    /// 7429(a)(1) (for 30-day administrative-review window).
    pub days_since_written_statement: u32,
    /// Days since taxpayer made administrative-review request
    /// (for IRS 15-day response window + 16-day judicial-
    /// review trigger).
    pub days_since_administrative_request: u32,
    /// Whether IRS has notified taxpayer of administrative
    /// determination.
    pub irs_determination_notified: bool,
    /// Days since district director's determination notice
    /// (for 90-day judicial-review window).
    pub days_since_determination_notice: u32,
    /// Days since proceeding commenced in district court
    /// (for 20-day court-determination window).
    pub days_since_proceeding_commenced: u32,
    /// Whether court has granted extension of 20-day period.
    pub court_extension_granted: bool,
    /// Days of extension granted by court (up to 40
    /// additional calendar days).
    pub court_extension_days: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7429Result {
    pub review_timely: bool,
    pub administrative_30_day_window: bool,
    pub judicial_90_day_window: bool,
    pub court_determination_20_day_window: bool,
    pub max_court_window_days: u32,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section7429Input) -> Section7429Result {
    let mut failure_reasons: Vec<String> = Vec::new();

    let administrative_window = input.days_since_written_statement <= 30;
    let judicial_90_day =
        !input.irs_determination_notified || input.days_since_determination_notice <= 90;

    let max_court_window: u32 = if input.court_extension_granted {
        20 + input.court_extension_days.min(40)
    } else {
        20
    };

    let court_determination_window = input.days_since_proceeding_commenced <= max_court_window;

    match input.review_stage {
        ReviewStage::Administrative => {
            if !administrative_window {
                failure_reasons.push(format!(
                    "26 USC § 7429(a)(2) — administrative review request not made within 30 days after written statement furnished ({} days elapsed)",
                    input.days_since_written_statement
                ));
            }
        }
        ReviewStage::Judicial => {
            let judicial_path_available =
                input.irs_determination_notified || input.days_since_administrative_request >= 16;

            if !judicial_path_available {
                failure_reasons.push(format!(
                    "26 USC § 7429(b)(1) — judicial review unavailable; IRS has not notified taxpayer of determination AND only {} days since administrative-review request (16-day trigger not yet reached)",
                    input.days_since_administrative_request
                ));
            }

            if input.irs_determination_notified && !judicial_90_day {
                failure_reasons.push(format!(
                    "26 USC § 7429(b)(1) — judicial review not filed within 90 days from earlier of (a) district director's notice of determination ({} days elapsed) or (b) 16th day after administrative-review request",
                    input.days_since_determination_notice
                ));
            }

            if !court_determination_window {
                failure_reasons.push(format!(
                    "26 USC § 7429(b)(3) — court determination not within statutory window; {} days elapsed since proceeding commenced (max {} = 20-day default + {} extension)",
                    input.days_since_proceeding_commenced,
                    max_court_window,
                    if input.court_extension_granted { input.court_extension_days.min(40) } else { 0 }
                ));
            }
        }
    }

    let notes: Vec<String> = vec![
        "26 USC § 7429(a) — administrative review framework: (1) IRS provides written statement within 5 days; (2) taxpayer requests administrative review within 30 days; (3) IRS responds within 15 calendar days"
            .to_string(),
        "26 USC § 7429(b)(1) — judicial review filed within 90 days from earlier of (a) district director's notice of determination or (b) 16th day after administrative review request"
            .to_string(),
        "26 USC § 7429(b)(2) — district court has EXCLUSIVE jurisdiction (no Tax Court alternative); § 7429(b)(3) court determines within 20 calendar days whether (1) assessment is REASONABLE and (2) amount is APPROPRIATE; extension up to 40 additional calendar days available for reasonable grounds (combined 60-day maximum)"
            .to_string(),
        "26 USC § 7421(a)(11) Anti-Injunction Act exception — § 7429(b) judicial review is one of 11 enumerated statutory exceptions to AIA bar on suits to restrain assessment or collection"
            .to_string(),
    ];

    Section7429Result {
        review_timely: failure_reasons.is_empty(),
        administrative_30_day_window: administrative_window,
        judicial_90_day_window: judicial_90_day,
        court_determination_20_day_window: court_determination_window,
        max_court_window_days: max_court_window,
        failure_reasons,
        citation: "26 USC § 7429(a)(1)-(3), (b)(1)-(3), (c); 26 CFR § 301.7429-3; § 6861; § 6862; § 7421(a)(11); IRM 5.1.4; IRM 5.17.15; IRM 8.24.2",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn admin_within_30() -> Section7429Input {
        Section7429Input {
            review_stage: ReviewStage::Administrative,
            days_since_written_statement: 15,
            days_since_administrative_request: 0,
            irs_determination_notified: false,
            days_since_determination_notice: 0,
            days_since_proceeding_commenced: 0,
            court_extension_granted: false,
            court_extension_days: 0,
        }
    }

    fn judicial_base() -> Section7429Input {
        Section7429Input {
            review_stage: ReviewStage::Judicial,
            days_since_written_statement: 25,
            days_since_administrative_request: 16,
            irs_determination_notified: true,
            days_since_determination_notice: 30,
            days_since_proceeding_commenced: 10,
            court_extension_granted: false,
            court_extension_days: 0,
        }
    }

    #[test]
    fn admin_within_30_days_timely() {
        let r = check(&admin_within_30());
        assert!(r.review_timely);
        assert!(r.administrative_30_day_window);
    }

    #[test]
    fn admin_at_30_day_boundary_timely() {
        let mut i = admin_within_30();
        i.days_since_written_statement = 30;
        let r = check(&i);
        assert!(r.review_timely);
        assert!(r.administrative_30_day_window);
    }

    #[test]
    fn admin_at_31_days_violates() {
        let mut i = admin_within_30();
        i.days_since_written_statement = 31;
        let r = check(&i);
        assert!(!r.review_timely);
        assert!(!r.administrative_30_day_window);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7429(a)(2)") && f.contains("30 days") && f.contains("31")));
    }

    #[test]
    fn judicial_within_90_days_timely() {
        let r = check(&judicial_base());
        assert!(r.review_timely);
        assert!(r.judicial_90_day_window);
        assert!(r.court_determination_20_day_window);
    }

    #[test]
    fn judicial_at_90_day_boundary_timely() {
        let mut i = judicial_base();
        i.days_since_determination_notice = 90;
        let r = check(&i);
        assert!(r.review_timely);
        assert!(r.judicial_90_day_window);
    }

    #[test]
    fn judicial_91_days_violates() {
        let mut i = judicial_base();
        i.days_since_determination_notice = 91;
        let r = check(&i);
        assert!(!r.review_timely);
        assert!(!r.judicial_90_day_window);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7429(b)(1)") && f.contains("90 days")));
    }

    #[test]
    fn judicial_unavailable_without_notification_or_16_days() {
        let mut i = judicial_base();
        i.irs_determination_notified = false;
        i.days_since_administrative_request = 15;
        let r = check(&i);
        assert!(!r.review_timely);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7429(b)(1)") && f.contains("16-day trigger")));
    }

    #[test]
    fn judicial_available_at_16_day_administrative_trigger() {
        let mut i = judicial_base();
        i.irs_determination_notified = false;
        i.days_since_administrative_request = 16;
        let r = check(&i);
        assert!(r.review_timely);
    }

    #[test]
    fn court_determination_within_20_days_timely() {
        let r = check(&judicial_base());
        assert!(r.court_determination_20_day_window);
        assert_eq!(r.max_court_window_days, 20);
    }

    #[test]
    fn court_determination_at_20_day_boundary_timely() {
        let mut i = judicial_base();
        i.days_since_proceeding_commenced = 20;
        let r = check(&i);
        assert!(r.court_determination_20_day_window);
    }

    #[test]
    fn court_determination_21_days_without_extension_violates() {
        let mut i = judicial_base();
        i.days_since_proceeding_commenced = 21;
        let r = check(&i);
        assert!(!r.review_timely);
        assert!(!r.court_determination_20_day_window);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7429(b)(3)") && f.contains("20-day")));
    }

    #[test]
    fn court_extension_up_to_40_additional_days() {
        let mut i = judicial_base();
        i.court_extension_granted = true;
        i.court_extension_days = 40;
        i.days_since_proceeding_commenced = 55;
        let r = check(&i);
        assert!(r.court_determination_20_day_window);
        assert_eq!(r.max_court_window_days, 60);
    }

    #[test]
    fn court_extension_at_60_day_combined_boundary_timely() {
        let mut i = judicial_base();
        i.court_extension_granted = true;
        i.court_extension_days = 40;
        i.days_since_proceeding_commenced = 60;
        let r = check(&i);
        assert!(r.court_determination_20_day_window);
    }

    #[test]
    fn court_extension_61_days_violates_combined_cap() {
        let mut i = judicial_base();
        i.court_extension_granted = true;
        i.court_extension_days = 40;
        i.days_since_proceeding_commenced = 61;
        let r = check(&i);
        assert!(!r.review_timely);
    }

    #[test]
    fn court_extension_capped_at_40_additional_days() {
        let mut i = judicial_base();
        i.court_extension_granted = true;
        i.court_extension_days = 50;
        let r = check(&i);
        assert_eq!(r.max_court_window_days, 60);
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = check(&admin_within_30());
        assert!(r.citation.contains("§ 7429(a)(1)-(3)"));
        assert!(r.citation.contains("(b)(1)-(3)"));
        assert!(r.citation.contains("(c)"));
        assert!(r.citation.contains("§ 301.7429-3"));
        assert!(r.citation.contains("§ 6861"));
        assert!(r.citation.contains("§ 6862"));
        assert!(r.citation.contains("§ 7421(a)(11)"));
        assert!(r.citation.contains("IRM 5.1.4"));
        assert!(r.citation.contains("IRM 5.17.15"));
        assert!(r.citation.contains("IRM 8.24.2"));
    }

    #[test]
    fn note_pins_administrative_5_30_15_day_framework() {
        let r = check(&admin_within_30());
        assert!(r.notes.iter().any(|n| n.contains("§ 7429(a)")
            && n.contains("5 days")
            && n.contains("30 days")
            && n.contains("15 calendar days")));
    }

    #[test]
    fn note_pins_judicial_90_day_16_day_trigger() {
        let r = check(&admin_within_30());
        assert!(r.notes.iter().any(|n| n.contains("§ 7429(b)(1)")
            && n.contains("90 days")
            && n.contains("16th day")));
    }

    #[test]
    fn note_pins_district_court_exclusive_jurisdiction_20_40_court_window() {
        let r = check(&admin_within_30());
        assert!(r.notes.iter().any(|n| n.contains("§ 7429(b)(2)")
            && n.contains("EXCLUSIVE jurisdiction")
            && n.contains("§ 7429(b)(3)")
            && n.contains("20 calendar days")
            && n.contains("40 additional")));
    }

    #[test]
    fn note_pins_aia_exception() {
        let r = check(&admin_within_30());
        assert!(r.notes.iter().any(|n| n.contains("§ 7421(a)(11)")
            && n.contains("Anti-Injunction Act")
            && n.contains("11 enumerated")));
    }

    #[test]
    fn review_stage_truth_table() {
        let r_admin = check(&admin_within_30());
        assert!(r_admin.review_timely);

        let r_judicial = check(&judicial_base());
        assert!(r_judicial.review_timely);
    }

    #[test]
    fn administrative_truth_table_30_day_boundary() {
        for (days, exp_timely) in [
            (0u32, true),
            (15, true),
            (30, true),
            (31, false),
            (60, false),
        ] {
            let mut i = admin_within_30();
            i.days_since_written_statement = days;
            let r = check(&i);
            assert_eq!(r.review_timely, exp_timely);
        }
    }

    #[test]
    fn court_determination_truth_table_with_and_without_extension() {
        for (days, extension, exp_max) in [
            (10u32, false, 20u32),
            (20, false, 20),
            (50, true, 60),
            (60, true, 60),
            (100, true, 60),
        ] {
            let mut i = judicial_base();
            i.days_since_proceeding_commenced = days;
            i.court_extension_granted = extension;
            i.court_extension_days = if extension { 40 } else { 0 };
            let r = check(&i);
            assert_eq!(r.max_court_window_days, exp_max);
        }
    }

    #[test]
    fn aia_exception_engaged_for_judicial_review() {
        let r = check(&judicial_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 7421(a)(11)")));
    }

    #[test]
    fn three_failures_stack_for_judicial_review() {
        let i = Section7429Input {
            review_stage: ReviewStage::Judicial,
            days_since_written_statement: 0,
            days_since_administrative_request: 15,
            irs_determination_notified: true,
            days_since_determination_notice: 91,
            days_since_proceeding_commenced: 21,
            court_extension_granted: false,
            court_extension_days: 0,
        };
        let r = check(&i);
        assert!(!r.review_timely);
        assert_eq!(r.failure_reasons.len(), 2);
    }

    #[test]
    fn court_window_20_day_default_no_extension() {
        let r = check(&judicial_base());
        assert_eq!(r.max_court_window_days, 20);
    }

    #[test]
    fn court_window_60_day_max_with_full_extension() {
        let mut i = judicial_base();
        i.court_extension_granted = true;
        i.court_extension_days = 40;
        let r = check(&i);
        assert_eq!(r.max_court_window_days, 60);
    }

    #[test]
    fn judicial_path_available_when_determination_notified() {
        let mut i = judicial_base();
        i.irs_determination_notified = true;
        i.days_since_administrative_request = 0;
        let r = check(&i);
        assert!(r.review_timely);
    }

    #[test]
    fn judicial_path_available_when_administrative_request_at_16th_day() {
        let mut i = judicial_base();
        i.irs_determination_notified = false;
        i.days_since_administrative_request = 16;
        let r = check(&i);
        assert!(r.review_timely);
    }

    #[test]
    fn judicial_path_unavailable_at_15th_day_without_notification() {
        let mut i = judicial_base();
        i.irs_determination_notified = false;
        i.days_since_administrative_request = 15;
        let r = check(&i);
        assert!(!r.review_timely);
    }

    #[test]
    fn admin_at_zero_days_compliant() {
        let mut i = admin_within_30();
        i.days_since_written_statement = 0;
        let r = check(&i);
        assert!(r.review_timely);
    }

    #[test]
    fn court_extension_30_day_partial_extension() {
        let mut i = judicial_base();
        i.court_extension_granted = true;
        i.court_extension_days = 30;
        i.days_since_proceeding_commenced = 50;
        let r = check(&i);
        assert!(r.court_determination_20_day_window);
        assert_eq!(r.max_court_window_days, 50);
    }
}
