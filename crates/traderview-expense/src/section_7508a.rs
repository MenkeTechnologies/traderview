//! IRC § 7508A — Authority to postpone certain deadlines by
//! reason of federally declared disaster, significant fire,
//! or terroristic or military actions. Trader-relevant for
//! any trader located in a federally declared disaster area
//! (CA wildfires + FL hurricanes + TX flooding + tornado
//! disasters etc.) who needs to extend filing + payment +
//! amended-return + Tax Court petition + § 6511 refund-claim
//! + § 6212 SNOD-response deadlines.
//!
//! Procedural-companion to § 7421 (Anti-Injunction Act + §
//! 7426 wrongful-levy exception), § 7433 (civil damages for
//! unauthorized collection), § 7430 (litigation costs against
//! IRS), § 6212 (SNOD), § 6213 (Tax Court petition timing),
//! and § 6511 (refund-claim limitations).
//!
//! **§ 7508A(a) Secretary's discretionary postponement** —
//! IRS Secretary may postpone the time for performing certain
//! tax acts up to **one year** for taxpayers affected by a
//! federally declared disaster (as defined under Stafford
//! Act, 42 USC § 5121 et seq.) or significant fire.
//!
//! **§ 7508A(b) terroristic or military action** —
//! Secretary may similarly postpone deadlines for taxpayers
//! affected by terroristic or military actions.
//!
//! **§ 7508A(c) special rules for pensions** — special rules
//! for IRA contributions, retirement plan loan repayments.
//!
//! **§ 7508A(d) MANDATORY 60-day postponement** — added by
//! Taxpayer Certainty and Disaster Tax Relief Act of 2019
//! (Pub. L. 116-94, Div. Q § 205, eff. for disasters
//! declared after December 20, 2019). For federally declared
//! disasters with specified incident date, taxpayers in
//! affected area get MINIMUM 60-day mandatory postponement
//! period. Runs concurrently with § 7508A(a)+(b)
//! discretionary postponement if Secretary postponement
//! period ≥ 60 days.
//!
//! **Postponement period defined** — period of time (up to
//! ONE YEAR) that IRS postpones deadlines.
//!
//! **Disaster area** — area determined under § 1033(h)(3) =
//! area eligible for federal assistance under Robert T.
//! Stafford Disaster Relief and Emergency Assistance Act, 42
//! USC § 5121 et seq.
//!
//! Citations: 26 USC § 7508A(a), (b), (c), (d); 26 CFR §
//! 301.7508A-1; Pub. L. 116-94 Div. Q § 205 (Taxpayer
//! Certainty and Disaster Tax Relief Act of 2019); 42 USC §
//! 5121 et seq. (Stafford Act); § 1033(h)(3) (disaster area
//! cross-reference).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PostponementBasis {
    /// § 7508A(a) — federally declared disaster.
    FederallyDeclaredDisaster,
    /// § 7508A(a) — significant fire.
    SignificantFire,
    /// § 7508A(b) — terroristic or military action.
    TerroristicOrMilitaryAction,
    /// No qualifying event.
    NoQualifyingEvent,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section7508AInput {
    pub postponement_basis: PostponementBasis,
    /// Whether taxpayer is located in the federally-declared
    /// disaster area (Stafford Act / § 1033(h)(3) eligible).
    pub taxpayer_in_disaster_area: bool,
    /// Whether the federal disaster was declared after
    /// December 20, 2019 (for § 7508A(d) mandatory 60-day
    /// postponement engagement).
    pub disaster_declared_after_dec_20_2019: bool,
    /// Number of days IRS Secretary has postponed deadlines
    /// under § 7508A(a) or (b) (discretionary).
    pub days_secretary_postponed: u32,
    /// Number of days requested by taxpayer / total
    /// postponement sought.
    pub days_requested: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section7508AResult {
    pub postponement_available: bool,
    pub mandatory_60_day_engaged: bool,
    pub days_postponed: u32,
    pub max_postponement_days: u32,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section7508AInput) -> Section7508AResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let qualifying_event = !matches!(
        input.postponement_basis,
        PostponementBasis::NoQualifyingEvent
    );

    if !qualifying_event {
        failure_reasons.push(
            "26 USC § 7508A(a)+(b) — postponement requires (a) federally declared disaster, (a) significant fire, or (b) terroristic or military action"
                .to_string(),
        );
    }

    if qualifying_event && !input.taxpayer_in_disaster_area {
        failure_reasons.push(
            "26 USC § 7508A + 26 USC § 1033(h)(3) + 42 USC § 5121 et seq. (Stafford Act) — taxpayer must be located in federally declared disaster area eligible for Stafford Act assistance"
                .to_string(),
        );
    }

    let mandatory_60_day_engaged = qualifying_event
        && input.taxpayer_in_disaster_area
        && matches!(
            input.postponement_basis,
            PostponementBasis::FederallyDeclaredDisaster | PostponementBasis::SignificantFire
        )
        && input.disaster_declared_after_dec_20_2019;

    let max_postponement: u32 = 365;

    let secretary_postponement = input.days_secretary_postponed.min(max_postponement);
    let base_postponement = if mandatory_60_day_engaged {
        secretary_postponement.max(60)
    } else {
        secretary_postponement
    };

    let days_postponed = base_postponement.min(max_postponement);

    if qualifying_event
        && input.taxpayer_in_disaster_area
        && input.days_requested > max_postponement
    {
        failure_reasons.push(format!(
            "26 USC § 7508A — postponement period CAPPED at one year (365 days); requested {} days exceeds statutory maximum",
            input.days_requested
        ));
    }

    let notes: Vec<String> = vec![
        "26 USC § 7508A(a) — Secretary may postpone tax-act deadlines up to ONE YEAR (365 days) for taxpayers affected by federally declared disaster or significant fire; § 7508A(b) terroristic or military action"
            .to_string(),
        "26 USC § 7508A(d) — MANDATORY 60-day postponement period for federally declared disasters with specified incident date, declared AFTER December 20, 2019 (Pub. L. 116-94 Div. Q § 205, Taxpayer Certainty and Disaster Tax Relief Act of 2019); runs CONCURRENTLY with Secretary's discretionary postponement under (a)+(b) if Secretary period ≥ 60 days"
            .to_string(),
        "26 USC § 1033(h)(3) + 42 USC § 5121 et seq. (Robert T. Stafford Disaster Relief and Emergency Assistance Act) — 'disaster area' means area eligible for federal assistance under Stafford Act"
            .to_string(),
        "26 CFR § 301.7508A-1 — implementing regulations; postponed acts include filing returns + paying tax + filing amended returns + Tax Court petitions + § 6511 refund claims + § 6212 SNOD responses + § 6213 deficiency challenges"
            .to_string(),
    ];

    Section7508AResult {
        postponement_available: failure_reasons.is_empty(),
        mandatory_60_day_engaged,
        days_postponed: if failure_reasons.is_empty() {
            days_postponed
        } else {
            0
        },
        max_postponement_days: max_postponement,
        failure_reasons,
        citation: "26 USC § 7508A(a), (b), (c), (d); 26 CFR § 301.7508A-1; Pub. L. 116-94 Div. Q § 205; 42 USC § 5121 et seq.; § 1033(h)(3)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn disaster_base() -> Section7508AInput {
        Section7508AInput {
            postponement_basis: PostponementBasis::FederallyDeclaredDisaster,
            taxpayer_in_disaster_area: true,
            disaster_declared_after_dec_20_2019: true,
            days_secretary_postponed: 90,
            days_requested: 90,
        }
    }

    #[test]
    fn federally_declared_disaster_postponement_available() {
        let r = check(&disaster_base());
        assert!(r.postponement_available);
        assert!(r.mandatory_60_day_engaged);
        assert_eq!(r.days_postponed, 90);
        assert_eq!(r.max_postponement_days, 365);
    }

    #[test]
    fn no_qualifying_event_no_postponement() {
        let mut i = disaster_base();
        i.postponement_basis = PostponementBasis::NoQualifyingEvent;
        let r = check(&i);
        assert!(!r.postponement_available);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 7508A(a)+(b)") && f.contains("federally declared disaster")));
    }

    #[test]
    fn not_in_disaster_area_no_postponement() {
        let mut i = disaster_base();
        i.taxpayer_in_disaster_area = false;
        let r = check(&i);
        assert!(!r.postponement_available);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("Stafford Act") && f.contains("disaster area")));
    }

    #[test]
    fn significant_fire_postponement_available() {
        let mut i = disaster_base();
        i.postponement_basis = PostponementBasis::SignificantFire;
        let r = check(&i);
        assert!(r.postponement_available);
        assert!(r.mandatory_60_day_engaged);
    }

    #[test]
    fn terroristic_or_military_action_postponement_available() {
        let mut i = disaster_base();
        i.postponement_basis = PostponementBasis::TerroristicOrMilitaryAction;
        let r = check(&i);
        assert!(r.postponement_available);
        assert!(!r.mandatory_60_day_engaged);
    }

    #[test]
    fn mandatory_60_day_floor_when_secretary_lt_60() {
        let mut i = disaster_base();
        i.days_secretary_postponed = 30;
        i.days_requested = 30;
        let r = check(&i);
        assert!(r.mandatory_60_day_engaged);
        assert_eq!(r.days_postponed, 60);
    }

    #[test]
    fn mandatory_60_day_runs_concurrently_when_secretary_gte_60() {
        let mut i = disaster_base();
        i.days_secretary_postponed = 90;
        let r = check(&i);
        assert!(r.mandatory_60_day_engaged);
        assert_eq!(r.days_postponed, 90);
    }

    #[test]
    fn pre_dec_20_2019_disaster_no_mandatory_60_day() {
        let mut i = disaster_base();
        i.disaster_declared_after_dec_20_2019 = false;
        i.days_secretary_postponed = 30;
        let r = check(&i);
        assert!(!r.mandatory_60_day_engaged);
        assert_eq!(r.days_postponed, 30);
    }

    #[test]
    fn terroristic_action_does_not_engage_mandatory_60_day() {
        let mut i = disaster_base();
        i.postponement_basis = PostponementBasis::TerroristicOrMilitaryAction;
        i.days_secretary_postponed = 30;
        let r = check(&i);
        assert!(!r.mandatory_60_day_engaged);
        assert_eq!(r.days_postponed, 30);
    }

    #[test]
    fn one_year_cap_engaged_at_365() {
        let mut i = disaster_base();
        i.days_secretary_postponed = 365;
        i.days_requested = 365;
        let r = check(&i);
        assert!(r.postponement_available);
        assert_eq!(r.days_postponed, 365);
    }

    #[test]
    fn over_one_year_requested_violates_cap() {
        let mut i = disaster_base();
        i.days_requested = 400;
        let r = check(&i);
        assert!(!r.postponement_available);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("one year") && f.contains("365 days") && f.contains("400")));
    }

    #[test]
    fn secretary_postponement_capped_at_365() {
        let mut i = disaster_base();
        i.days_secretary_postponed = 500;
        i.days_requested = 300;
        let r = check(&i);
        assert!(r.postponement_available);
        assert_eq!(r.days_postponed, 365);
    }

    #[test]
    fn citation_pins_all_subsections() {
        let r = check(&disaster_base());
        assert!(r.citation.contains("§ 7508A(a)"));
        assert!(r.citation.contains("(b)"));
        assert!(r.citation.contains("(c)"));
        assert!(r.citation.contains("(d)"));
        assert!(r.citation.contains("§ 301.7508A-1"));
        assert!(r.citation.contains("Pub. L. 116-94"));
        assert!(r.citation.contains("42 USC § 5121"));
        assert!(r.citation.contains("§ 1033(h)(3)"));
    }

    #[test]
    fn note_pins_one_year_and_60_day() {
        let r = check(&disaster_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 7508A(a)") && n.contains("ONE YEAR") && n.contains("365 days")));
        assert!(r.notes.iter().any(|n| n.contains("§ 7508A(d)")
            && n.contains("MANDATORY 60-day")
            && n.contains("December 20, 2019")));
    }

    #[test]
    fn note_pins_stafford_act() {
        let r = check(&disaster_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 1033(h)(3)")
            && n.contains("Stafford")
            && n.contains("42 USC § 5121")));
    }

    #[test]
    fn note_pins_regs_postponed_acts() {
        let r = check(&disaster_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 301.7508A-1")
            && n.contains("§ 6511")
            && n.contains("§ 6212")
            && n.contains("§ 6213")));
    }

    #[test]
    fn note_pins_pub_l_116_94() {
        let r = check(&disaster_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Pub. L. 116-94") && n.contains("Div. Q § 205")));
    }

    #[test]
    fn postponement_basis_truth_table() {
        for (basis, exp_avail) in [
            (PostponementBasis::FederallyDeclaredDisaster, true),
            (PostponementBasis::SignificantFire, true),
            (PostponementBasis::TerroristicOrMilitaryAction, true),
            (PostponementBasis::NoQualifyingEvent, false),
        ] {
            let mut i = disaster_base();
            i.postponement_basis = basis;
            let r = check(&i);
            assert_eq!(r.postponement_available, exp_avail);
        }
    }

    #[test]
    fn mandatory_60_day_only_for_disaster_and_fire_invariant() {
        for (basis, exp_engaged) in [
            (PostponementBasis::FederallyDeclaredDisaster, true),
            (PostponementBasis::SignificantFire, true),
            (PostponementBasis::TerroristicOrMilitaryAction, false),
        ] {
            let mut i = disaster_base();
            i.postponement_basis = basis;
            let r = check(&i);
            assert_eq!(r.mandatory_60_day_engaged, exp_engaged);
        }
    }

    #[test]
    fn mandatory_60_day_requires_post_dec_20_2019_invariant() {
        let mut i_post = disaster_base();
        i_post.disaster_declared_after_dec_20_2019 = true;
        let r_post = check(&i_post);
        assert!(r_post.mandatory_60_day_engaged);

        let mut i_pre = disaster_base();
        i_pre.disaster_declared_after_dec_20_2019 = false;
        let r_pre = check(&i_pre);
        assert!(!r_pre.mandatory_60_day_engaged);
    }

    #[test]
    fn defensive_zero_days_secretary_postponed_engages_60_day_floor() {
        let mut i = disaster_base();
        i.days_secretary_postponed = 0;
        i.days_requested = 60;
        let r = check(&i);
        assert!(r.postponement_available);
        assert_eq!(r.days_postponed, 60);
    }

    #[test]
    fn defensive_at_max_365_boundary() {
        let mut i = disaster_base();
        i.days_secretary_postponed = 365;
        i.days_requested = 365;
        let r = check(&i);
        assert_eq!(r.days_postponed, 365);
    }

    #[test]
    fn defensive_at_366_days_requested_violates() {
        let mut i = disaster_base();
        i.days_requested = 366;
        let r = check(&i);
        assert!(!r.postponement_available);
    }

    #[test]
    fn pre_2019_amendment_no_mandatory_floor_when_secretary_lt_60() {
        let mut i = disaster_base();
        i.disaster_declared_after_dec_20_2019 = false;
        i.days_secretary_postponed = 10;
        let r = check(&i);
        assert!(!r.mandatory_60_day_engaged);
        assert_eq!(r.days_postponed, 10);
    }

    #[test]
    fn post_2019_amendment_60_day_floor_engages_when_secretary_zero() {
        let mut i = disaster_base();
        i.disaster_declared_after_dec_20_2019 = true;
        i.days_secretary_postponed = 0;
        i.days_requested = 60;
        let r = check(&i);
        assert!(r.mandatory_60_day_engaged);
        assert_eq!(r.days_postponed, 60);
    }

    #[test]
    fn secretary_180_days_postponed_overrides_60_day_floor() {
        let mut i = disaster_base();
        i.days_secretary_postponed = 180;
        i.days_requested = 180;
        let r = check(&i);
        assert!(r.mandatory_60_day_engaged);
        assert_eq!(r.days_postponed, 180);
    }

    #[test]
    fn taxpayer_outside_disaster_area_with_qualifying_event_no_postponement() {
        let mut i = disaster_base();
        i.taxpayer_in_disaster_area = false;
        let r = check(&i);
        assert!(!r.postponement_available);
        assert_eq!(r.days_postponed, 0);
    }

    #[test]
    fn three_failure_modes_stack_when_no_event_no_area_over_cap() {
        let i = Section7508AInput {
            postponement_basis: PostponementBasis::NoQualifyingEvent,
            taxpayer_in_disaster_area: false,
            disaster_declared_after_dec_20_2019: true,
            days_secretary_postponed: 90,
            days_requested: 400,
        };
        let r = check(&i);
        assert!(!r.postponement_available);
        assert_eq!(r.failure_reasons.len(), 1);
    }

    #[test]
    fn max_postponement_always_365() {
        let r = check(&disaster_base());
        assert_eq!(r.max_postponement_days, 365);
    }
}
