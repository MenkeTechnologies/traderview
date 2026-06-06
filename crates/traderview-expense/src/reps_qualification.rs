//! IRC §469(c)(7) — Real Estate Professional Status qualification.
//!
//! REPS is the gate that flips rental losses from per-se passive
//! (iter 5's `section_469`) to NON-PASSIVE, removing the §25k allowance
//! cap entirely. The bar is high — most landlords don't qualify. This
//! module checks whether a taxpayer's facts pass §469(c)(7)'s
//! three-prong test:
//!
//!   1. **750-hour test** — more than 750 hours of services performed
//!      during the year in real property trades or businesses in which
//!      the taxpayer materially participates.
//!   2. **>50% of personal services test** — more than half of the
//!      taxpayer's total personal services (across ALL work,
//!      including W-2 employment outside real estate) performed that
//!      year are in real property trades or businesses.
//!   3. **Material participation** — per-activity unless §469(c)(7)(A)
//!      grouping election is filed (treats all rentals as one activity).
//!      Caller asserts whether material participation is met via one
//!      of the seven §1.469-5T tests.
//!
//! §469(c)(7)(B) "real property trade or business" enumerates eleven
//! qualifying activity classes — development, redevelopment,
//! construction, reconstruction, acquisition, conversion, rental,
//! operation, management, leasing, brokerage. Hours in NON-qualifying
//! activities (W-2 software job, retail clerk, etc.) are excluded
//! from numerator of both tests.
//!
//! **MFJ rule (§469(c)(7)(B) flush language)**: REPS qualification is
//! **per-spouse**. One spouse alone must meet the 750-hour AND >50%
//! tests. Spouses CANNOT aggregate hours to qualify jointly. Once one
//! spouse qualifies as a real-estate professional, however, both
//! spouses' rental activities are tested for material participation
//! (and material participation IS aggregated per §469(h)(5)).
//!
//! Pure compute. Caller passes hours in each activity category +
//! material-participation assertion. We return whether REPS is met
//! and (when not) which prong(s) failed.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// §469(c)(7)(B) eleven qualifying real-property trade-or-business
/// categories. Hours in these count toward both the 750-hour and
/// the "more than half of personal services" tests. Stored as an enum
/// so the API can sum hours per category without ambiguity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RptbActivity {
    Development,
    Redevelopment,
    Construction,
    Reconstruction,
    Acquisition,
    Conversion,
    Rental,
    Operation,
    Management,
    Leasing,
    Brokerage,
}

/// One of seven §1.469-5T material-participation tests. Caller asserts
/// which one was satisfied (or none). We do NOT compute these
/// automatically — they require facts the API doesn't have (hours of
/// participation by OTHER people, prior-year participation, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MaterialParticipationTest {
    /// Test 1: > 500 hours.
    OverFiveHundredHours,
    /// Test 2: substantially all participation in the activity.
    SubstantiallyAll,
    /// Test 3: > 100 hours AND no other individual participated more.
    OverHundredHoursAndMost,
    /// Test 4: Significant Participation Activity, total SPA hours > 500.
    SpaTotalOverFiveHundred,
    /// Test 5: Materially participated in 5 of the last 10 years.
    PriorFiveOfTen,
    /// Test 6: Personal service activity, prior 3 years.
    PersonalServicePriorThree,
    /// Test 7: Facts and circumstances, > 100 hours.
    FactsAndCircumstances,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepsInput {
    /// Hours per qualifying RPTB activity for THIS taxpayer this year.
    pub hours_by_activity: Vec<RptbHourEntry>,
    /// All NON-RPTB personal services hours this year (W-2 day job,
    /// other non-real-estate self-employment, etc.). Used as the
    /// denominator for the >50% test together with total RPTB hours.
    pub other_personal_services_hours: Decimal,
    /// Caller asserts WHICH §1.469-5T test the taxpayer satisfied
    /// for material participation. None → no material participation.
    pub material_participation_test_satisfied: Option<MaterialParticipationTest>,
    /// Filing status — affects the per-spouse rule note but does not
    /// change the math (each spouse is tested independently).
    pub filing_status: String,
    /// True when §469(c)(7)(A) grouping election is filed — treats
    /// all rental real estate as ONE activity so material participation
    /// is tested at the aggregate level.
    pub grouping_election_filed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RptbHourEntry {
    pub activity: RptbActivity,
    pub hours: Decimal,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepsResult {
    pub total_rptb_hours: Decimal,
    pub total_personal_services_hours: Decimal,
    pub rptb_share_of_personal_services: Decimal,
    pub passes_750_hour_test: bool,
    pub passes_more_than_half_test: bool,
    pub material_participation_met: bool,
    pub qualifies_as_reps: bool,
    pub failure_reasons: Vec<String>,
    pub note: String,
}

fn seven_fifty() -> Decimal {
    Decimal::from_str("750").unwrap()
}

fn half() -> Decimal {
    Decimal::from_str("0.5").unwrap()
}

pub fn compute(input: &RepsInput) -> RepsResult {
    let mut r = RepsResult::default();

    r.total_rptb_hours = input
        .hours_by_activity
        .iter()
        .map(|e| e.hours.max(Decimal::ZERO))
        .sum();
    r.total_personal_services_hours =
        r.total_rptb_hours + input.other_personal_services_hours.max(Decimal::ZERO);

    r.rptb_share_of_personal_services = if r.total_personal_services_hours > Decimal::ZERO {
        (r.total_rptb_hours / r.total_personal_services_hours).round_dp(5)
    } else {
        Decimal::ZERO
    };

    r.passes_750_hour_test = r.total_rptb_hours > seven_fifty();
    r.passes_more_than_half_test = r.rptb_share_of_personal_services > half();
    r.material_participation_met = input.material_participation_test_satisfied.is_some();

    r.qualifies_as_reps =
        r.passes_750_hour_test && r.passes_more_than_half_test && r.material_participation_met;

    if !r.passes_750_hour_test {
        r.failure_reasons.push(format!(
            "750-hour test failed: {} hours in RPTB (>750 required)",
            r.total_rptb_hours
        ));
    }
    if !r.passes_more_than_half_test {
        r.failure_reasons.push(format!(
            ">50% test failed: {} / {} = {} of personal services in RPTB (>0.5 required)",
            r.total_rptb_hours, r.total_personal_services_hours, r.rptb_share_of_personal_services,
        ));
    }
    if !r.material_participation_met {
        r.failure_reasons.push(
            "material participation not asserted (must satisfy one of seven §1.469-5T tests)"
                .into(),
        );
    }

    r.note = if r.qualifies_as_reps {
        let grouping = if input.grouping_election_filed {
            " (§469(c)(7)(A) grouping election active — material participation tested at aggregate level)"
        } else {
            ""
        };
        format!(
            "REPS qualified: {} RPTB hours, {} of personal services, MP via {:?}{}",
            r.total_rptb_hours,
            r.rptb_share_of_personal_services,
            input.material_participation_test_satisfied.unwrap(),
            grouping,
        )
    } else if input
        .filing_status
        .eq_ignore_ascii_case("married_filing_jointly")
    {
        format!(
            "REPS NOT qualified for this spouse — §469(c)(7)(B) is per-spouse, spouses cannot aggregate hours: {}",
            r.failure_reasons.join("; ")
        )
    } else {
        format!("REPS NOT qualified: {}", r.failure_reasons.join("; "))
    };
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn hr(a: RptbActivity, h: Decimal) -> RptbHourEntry {
        RptbHourEntry {
            activity: a,
            hours: h,
        }
    }

    fn passing() -> RepsInput {
        // 800 hours rental + 0 other = 100% RPTB. Passes everything.
        RepsInput {
            hours_by_activity: vec![hr(RptbActivity::Rental, dec!(800))],
            other_personal_services_hours: Decimal::ZERO,
            material_participation_test_satisfied: Some(
                MaterialParticipationTest::OverFiveHundredHours,
            ),
            filing_status: "single".into(),
            grouping_election_filed: false,
        }
    }

    #[test]
    fn full_time_landlord_qualifies() {
        let r = compute(&passing());
        assert!(r.qualifies_as_reps);
        assert!(r.passes_750_hour_test);
        assert!(r.passes_more_than_half_test);
        assert!(r.material_participation_met);
        assert_eq!(r.rptb_share_of_personal_services, Decimal::ONE);
    }

    #[test]
    fn w2_software_job_kills_more_than_half_test() {
        // 800 RPTB hours + 2000 W-2 software job hours = 28.6% RPTB. Fails.
        let mut i = passing();
        i.other_personal_services_hours = dec!(2000);
        let r = compute(&i);
        assert!(!r.qualifies_as_reps);
        assert!(!r.passes_more_than_half_test);
        assert!(r.passes_750_hour_test); // still passes 750-hour
        assert!(r.failure_reasons.iter().any(|s| s.contains(">50%")));
    }

    #[test]
    fn boundary_exactly_750_hours_fails() {
        // §469(c)(7)(B)(i) is STRICT >, not >=. 750 itself fails.
        let mut i = passing();
        i.hours_by_activity = vec![hr(RptbActivity::Rental, dec!(750))];
        let r = compute(&i);
        assert!(!r.qualifies_as_reps);
        assert!(!r.passes_750_hour_test);
    }

    #[test]
    fn boundary_751_hours_passes_750_test() {
        let mut i = passing();
        i.hours_by_activity = vec![hr(RptbActivity::Rental, dec!(751))];
        let r = compute(&i);
        assert!(r.qualifies_as_reps);
        assert!(r.passes_750_hour_test);
    }

    #[test]
    fn boundary_exactly_50_pct_fails_more_than_half() {
        // 1000 RPTB + 1000 other = 50.00%. Fails the strict > test.
        let mut i = passing();
        i.hours_by_activity = vec![hr(RptbActivity::Rental, dec!(1000))];
        i.other_personal_services_hours = dec!(1000);
        let r = compute(&i);
        assert!(!r.qualifies_as_reps);
        assert!(!r.passes_more_than_half_test);
        assert_eq!(r.rptb_share_of_personal_services, half());
    }

    #[test]
    fn over_50pct_by_one_hour_passes() {
        let mut i = passing();
        i.hours_by_activity = vec![hr(RptbActivity::Rental, dec!(1001))];
        i.other_personal_services_hours = dec!(1000);
        let r = compute(&i);
        assert!(r.qualifies_as_reps);
        assert!(r.passes_more_than_half_test);
    }

    #[test]
    fn material_participation_missing_kills_qualification() {
        let mut i = passing();
        i.material_participation_test_satisfied = None;
        let r = compute(&i);
        assert!(!r.qualifies_as_reps);
        assert!(!r.material_participation_met);
        assert!(r
            .failure_reasons
            .iter()
            .any(|s| s.contains("material participation")));
    }

    #[test]
    fn hours_summed_across_all_eleven_activities() {
        let mut i = passing();
        i.hours_by_activity = vec![
            hr(RptbActivity::Development, dec!(100)),
            hr(RptbActivity::Construction, dec!(150)),
            hr(RptbActivity::Acquisition, dec!(80)),
            hr(RptbActivity::Rental, dec!(200)),
            hr(RptbActivity::Management, dec!(150)),
            hr(RptbActivity::Leasing, dec!(80)),
            hr(RptbActivity::Brokerage, dec!(50)),
        ];
        let r = compute(&i);
        assert_eq!(r.total_rptb_hours, dec!(810));
        assert!(r.qualifies_as_reps);
    }

    #[test]
    fn mfj_failure_note_calls_out_per_spouse_rule() {
        let mut i = passing();
        i.filing_status = "married_filing_jointly".into();
        i.hours_by_activity = vec![hr(RptbActivity::Rental, dec!(500))]; // < 750
        let r = compute(&i);
        assert!(!r.qualifies_as_reps);
        assert!(r.note.contains("per-spouse"));
    }

    #[test]
    fn grouping_election_called_out_in_note_when_qualified() {
        let mut i = passing();
        i.grouping_election_filed = true;
        let r = compute(&i);
        assert!(r.qualifies_as_reps);
        assert!(r.note.contains("grouping election"));
    }

    #[test]
    fn negative_hours_clamped_to_zero() {
        let mut i = passing();
        i.hours_by_activity = vec![hr(RptbActivity::Rental, dec!(-100))];
        let r = compute(&i);
        assert_eq!(r.total_rptb_hours, Decimal::ZERO);
        assert!(!r.qualifies_as_reps);
    }

    #[test]
    fn zero_hours_zero_other_returns_zero_share_no_divide_by_zero() {
        let i = RepsInput {
            hours_by_activity: vec![],
            other_personal_services_hours: Decimal::ZERO,
            material_participation_test_satisfied: Some(
                MaterialParticipationTest::OverFiveHundredHours,
            ),
            filing_status: "single".into(),
            grouping_election_filed: false,
        };
        let r = compute(&i);
        assert_eq!(r.rptb_share_of_personal_services, Decimal::ZERO);
        assert!(!r.qualifies_as_reps);
    }

    #[test]
    fn all_three_failures_listed_when_all_three_miss() {
        let i = RepsInput {
            hours_by_activity: vec![hr(RptbActivity::Rental, dec!(100))],
            other_personal_services_hours: dec!(2000),
            material_participation_test_satisfied: None,
            filing_status: "single".into(),
            grouping_election_filed: false,
        };
        let r = compute(&i);
        assert_eq!(r.failure_reasons.len(), 3);
    }

    #[test]
    fn all_seven_material_participation_tests_accepted() {
        for t in [
            MaterialParticipationTest::OverFiveHundredHours,
            MaterialParticipationTest::SubstantiallyAll,
            MaterialParticipationTest::OverHundredHoursAndMost,
            MaterialParticipationTest::SpaTotalOverFiveHundred,
            MaterialParticipationTest::PriorFiveOfTen,
            MaterialParticipationTest::PersonalServicePriorThree,
            MaterialParticipationTest::FactsAndCircumstances,
        ] {
            let mut i = passing();
            i.material_participation_test_satisfied = Some(t);
            let r = compute(&i);
            assert!(r.qualifies_as_reps, "MP test {:?} should be accepted", t);
        }
    }

    #[test]
    fn just_750_hours_with_zero_other_still_fails_strict_gt() {
        // Reproduces the case where someone meets the share test but
        // not the strict > 750.
        let mut i = passing();
        i.hours_by_activity = vec![hr(RptbActivity::Rental, dec!(750))];
        let r = compute(&i);
        assert!(r.passes_more_than_half_test); // 750/750 = 100% > 50%
        assert!(!r.passes_750_hour_test); // 750 NOT > 750
        assert!(!r.qualifies_as_reps);
    }
}
