//! IRC § 6304 — Fair Tax Collection Practices. Added by the
//! IRS Restructuring and Reform Act of 1998 (RRA 98 § 3466)
//! to import Fair Debt Collection Practices Act (FDCPA, 15
//! USC § 1692) limits into IRS collection activity.
//! Trader-relevant when an IRS revenue officer or private
//! collection agency contractor engaged under § 6306 contacts
//! the trader-taxpayer repeatedly at inconvenient times or
//! places, calls workplace after employer disapproval, or
//! uses harassing or abusive collection tactics. Companion to
//! § 6303 (notice and demand), § 7433 (civil damages for
//! unauthorized collection actions — vehicle for recovery
//! when § 6304 violated), § 7811 (Taxpayer Assistance Order),
//! § 7521 (audio recording of in-person interviews).
//!
//! **§ 6304(a) Communications without consent**: without the
//! prior consent of the taxpayer given directly to the
//! Secretary OR the express permission of a court of
//! competent jurisdiction, the Secretary may NOT communicate
//! with a taxpayer in connection with the collection of any
//! unpaid tax:
//! 1. at any unusual time or place or a time or place known
//!    or which should be known to be inconvenient to the
//!    taxpayer (in the absence of knowledge of circumstances
//!    to the contrary, between 8 a.m. and 9 p.m. local time
//!    at the taxpayer's location is convenient — outside that
//!    window is rebuttably inconvenient);
//! 2. if the Secretary knows the taxpayer is represented by a
//!    person authorized to practice before the IRS with
//!    respect to such unpaid tax and has knowledge of (or can
//!    readily ascertain) such person's name and address,
//!    unless such person fails to respond within a reasonable
//!    period to a communication from the Secretary or unless
//!    such person consents to direct communication with the
//!    taxpayer;
//! 3. at the taxpayer's place of employment if the Secretary
//!    knows or has reason to know that the taxpayer's
//!    employer prohibits the taxpayer from receiving such
//!    communication.
//!
//! **§ 6304(b) Prohibition on harassment and abuse**: the
//! Secretary may NOT engage in any conduct the natural
//! consequence of which is to harass, oppress, or abuse any
//! person in connection with the collection of any unpaid
//! tax. Without limiting the general application of the
//! foregoing, the following conduct is a violation:
//! 1. the use or threat of use of violence or other criminal
//!    means to harm the physical person, reputation, or
//!    property of any person;
//! 2. the use of obscene or profane language or language the
//!    natural consequence of which is to abuse the hearer or
//!    reader;
//! 3. causing a telephone to ring or engaging any person in
//!    telephone conversation repeatedly or continuously with
//!    intent to annoy, abuse, or harass any person at the
//!    called number;
//! 4. except as provided under rules similar to the rules in
//!    § 804 of the FDCPA (15 USC § 1692b), the placement of
//!    telephone calls without meaningful disclosure of the
//!    caller's identity.
//!
//! **§ 6304(c) Civil action**: notwithstanding any other
//! provision of this section, any action taken in violation
//! of subsection (a) or (b) shall be subject to the
//! provisions of § 7433 — civil damages capped at $1,000,000
//! for reckless or intentional violations and $100,000 for
//! negligence, less amount of such damages awarded by reason
//! of a judgment under § 7433 with respect to the same
//! action.
//!
//! Citations: 26 USC § 6304(a)-(c); RRA 98 § 3466; 15 USC §
//! 1692 (FDCPA); IRM 5.1.10.3 (Communications with
//! Taxpayers); 26 USC § 7433 (civil damages); 26 USC § 7811
//! (TAO); 26 USC § 7521 (audio recording); 26 USC § 6306
//! (private collection agencies).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TimeOfDay {
    /// Convenient default window 8 a.m. to 9 p.m. local time.
    Convenient8amTo9pm,
    /// Before 8 a.m. local time (rebuttably inconvenient).
    BeforeEightAm,
    /// After 9 p.m. local time (rebuttably inconvenient).
    AfterNinePm,
    /// Time known or should be known to be inconvenient.
    KnownInconvenient,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HarassmentConduct {
    /// No harassing conduct alleged.
    None,
    /// § 6304(b)(1) — threat of violence or criminal means.
    ThreatOfViolenceOrCriminal,
    /// § 6304(b)(2) — obscene or profane language.
    ObsceneOrProfaneLanguage,
    /// § 6304(b)(3) — repeated or continuous phone ringing.
    RepeatedPhoneRinging,
    /// § 6304(b)(4) — anonymous calls without identity
    /// disclosure.
    AnonymousCallsNoIdentity,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DamagesTier {
    /// § 7433(b)(1) — reckless or intentional violations cap
    /// at $1,000,000.
    RecklessOrIntentional,
    /// § 7433(b) flush — negligence cap at $100,000.
    Negligence,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section6304Input {
    /// Time of day Secretary communicated with taxpayer.
    pub time_of_day: TimeOfDay,
    /// Whether taxpayer gave prior consent OR court
    /// authorized communication.
    pub consent_or_court_order: bool,
    /// Whether Secretary knows taxpayer is represented by
    /// authorized practitioner under § 7521.
    pub taxpayer_represented: bool,
    /// Whether representative failed to respond within a
    /// reasonable period (§ 6304(a)(2) exception).
    pub representative_unresponsive: bool,
    /// Whether representative consented to direct
    /// communication with taxpayer.
    pub representative_consented_to_direct: bool,
    /// Whether Secretary contacted taxpayer at place of
    /// employment.
    pub contacted_at_workplace: bool,
    /// Whether Secretary knows employer prohibits collection
    /// communications.
    pub employer_prohibits_workplace_contact: bool,
    /// § 6304(b) harassment conduct alleged.
    pub harassment_conduct: HarassmentConduct,
    /// Whether reckless/intentional vs negligence (sets
    /// § 7433 damages cap).
    pub damages_tier: DamagesTier,
    /// Actual damages claimed (cents).
    pub actual_damages_cents: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section6304Result {
    pub section_a_violation: bool,
    pub section_b_violation: bool,
    pub any_violation: bool,
    pub time_violation: bool,
    pub represented_taxpayer_bypass_violation: bool,
    pub workplace_contact_violation: bool,
    pub harassment_violation: bool,
    pub section_7433_damages_cap_cents: u64,
    pub recoverable_damages_cents: u64,
    pub violation_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section6304Input) -> Section6304Result {
    let mut violation_reasons: Vec<String> = Vec::new();

    let inconvenient_time = matches!(
        input.time_of_day,
        TimeOfDay::BeforeEightAm | TimeOfDay::AfterNinePm | TimeOfDay::KnownInconvenient
    );

    let time_violation = inconvenient_time && !input.consent_or_court_order;

    if time_violation {
        violation_reasons.push(
            "26 USC § 6304(a)(1) — Secretary may not communicate at unusual time or place; default convenient window is 8 a.m. to 9 p.m. local time at taxpayer location absent consent or court order"
                .to_string(),
        );
    }

    let represented_bypass_violation = input.taxpayer_represented
        && !input.consent_or_court_order
        && !input.representative_unresponsive
        && !input.representative_consented_to_direct;

    if represented_bypass_violation {
        violation_reasons.push(
            "26 USC § 6304(a)(2) — Secretary may not bypass authorized representative under § 7521 without representative non-response or consent"
                .to_string(),
        );
    }

    let workplace_violation = input.contacted_at_workplace
        && input.employer_prohibits_workplace_contact
        && !input.consent_or_court_order;

    if workplace_violation {
        violation_reasons.push(
            "26 USC § 6304(a)(3) — Secretary may not contact taxpayer at place of employment if employer prohibits such communication"
                .to_string(),
        );
    }

    let harassment_violation =
        !matches!(input.harassment_conduct, HarassmentConduct::None);

    if harassment_violation {
        match input.harassment_conduct {
            HarassmentConduct::ThreatOfViolenceOrCriminal => violation_reasons.push(
                "26 USC § 6304(b)(1) — threat of violence or criminal means to harm physical person, reputation, or property prohibited"
                    .to_string(),
            ),
            HarassmentConduct::ObsceneOrProfaneLanguage => violation_reasons.push(
                "26 USC § 6304(b)(2) — obscene or profane language whose natural consequence is to abuse hearer or reader prohibited"
                    .to_string(),
            ),
            HarassmentConduct::RepeatedPhoneRinging => violation_reasons.push(
                "26 USC § 6304(b)(3) — causing telephone to ring or engaging in telephone conversation repeatedly or continuously with intent to annoy abuse or harass prohibited"
                    .to_string(),
            ),
            HarassmentConduct::AnonymousCallsNoIdentity => violation_reasons.push(
                "26 USC § 6304(b)(4) — placement of telephone calls without meaningful disclosure of caller identity (rules similar to FDCPA § 804) prohibited"
                    .to_string(),
            ),
            HarassmentConduct::None => {}
        }
    }

    let section_a_violation =
        time_violation || represented_bypass_violation || workplace_violation;
    let section_b_violation = harassment_violation;
    let any_violation = section_a_violation || section_b_violation;

    let cap_cents: u64 = match input.damages_tier {
        DamagesTier::RecklessOrIntentional => 100_000_000,
        DamagesTier::Negligence => 10_000_000,
    };

    let recoverable: u64 = if any_violation {
        input.actual_damages_cents.min(cap_cents)
    } else {
        0
    };

    let notes: Vec<String> = vec![
        "26 USC § 6304(a) — Secretary may not communicate with taxpayer without prior consent or court order at unusual time or place, while taxpayer represented under § 7521, or at workplace if employer prohibits"
            .to_string(),
        "26 USC § 6304(a)(1) — default convenient window 8 a.m. to 9 p.m. local time at taxpayer location; outside window rebuttably inconvenient"
            .to_string(),
        "26 USC § 6304(b) — Secretary may not engage in conduct whose natural consequence is to harass, oppress, or abuse: threats, obscene language, repeated phone ringing, anonymous calls without identity disclosure"
            .to_string(),
        "26 USC § 6304(c) — violations subject to § 7433 civil action; damages cap is $1,000,000 for reckless or intentional violations, $100,000 for negligence; less amount of § 7433 judgment for same action"
            .to_string(),
        "Added by RRA 98 § 3466 to import FDCPA (15 USC § 1692) protections into IRS collection activity; companion provisions § 6303 notice and demand, § 7433 damages, § 7811 TAO, § 7521 audio recording, § 6306 private collection agencies"
            .to_string(),
        "IRM 5.1.10.3 — Communications with Taxpayers — internal IRS guidance implementing § 6304; revenue officers train on fair tax collection practices"
            .to_string(),
    ];

    Section6304Result {
        section_a_violation,
        section_b_violation,
        any_violation,
        time_violation,
        represented_taxpayer_bypass_violation: represented_bypass_violation,
        workplace_contact_violation: workplace_violation,
        harassment_violation,
        section_7433_damages_cap_cents: cap_cents,
        recoverable_damages_cents: recoverable,
        violation_reasons,
        citation: "26 USC § 6304(a)-(c); RRA 98 § 3466; 15 USC § 1692; IRM 5.1.10.3; § 7433; § 7811; § 7521; § 6306",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn clean_base() -> Section6304Input {
        Section6304Input {
            time_of_day: TimeOfDay::Convenient8amTo9pm,
            consent_or_court_order: false,
            taxpayer_represented: false,
            representative_unresponsive: false,
            representative_consented_to_direct: false,
            contacted_at_workplace: false,
            employer_prohibits_workplace_contact: false,
            harassment_conduct: HarassmentConduct::None,
            damages_tier: DamagesTier::RecklessOrIntentional,
            actual_damages_cents: 0,
        }
    }

    #[test]
    fn clean_communication_no_violation() {
        let r = check(&clean_base());
        assert!(!r.any_violation);
        assert!(!r.section_a_violation);
        assert!(!r.section_b_violation);
    }

    #[test]
    fn before_8am_inconvenient_time_violation() {
        let mut i = clean_base();
        i.time_of_day = TimeOfDay::BeforeEightAm;
        let r = check(&i);
        assert!(r.time_violation);
        assert!(r.any_violation);
    }

    #[test]
    fn after_9pm_inconvenient_time_violation() {
        let mut i = clean_base();
        i.time_of_day = TimeOfDay::AfterNinePm;
        let r = check(&i);
        assert!(r.time_violation);
    }

    #[test]
    fn known_inconvenient_time_violation() {
        let mut i = clean_base();
        i.time_of_day = TimeOfDay::KnownInconvenient;
        let r = check(&i);
        assert!(r.time_violation);
        assert!(r.violation_reasons.iter().any(|v| v.contains("§ 6304(a)(1)")
            && v.contains("8 a.m. to 9 p.m.")));
    }

    #[test]
    fn consent_cures_time_violation() {
        let mut i = clean_base();
        i.time_of_day = TimeOfDay::BeforeEightAm;
        i.consent_or_court_order = true;
        let r = check(&i);
        assert!(!r.time_violation);
        assert!(!r.any_violation);
    }

    #[test]
    fn represented_taxpayer_bypass_violation() {
        let mut i = clean_base();
        i.taxpayer_represented = true;
        let r = check(&i);
        assert!(r.represented_taxpayer_bypass_violation);
        assert!(r.violation_reasons.iter().any(|v| v.contains("§ 6304(a)(2)")
            && v.contains("§ 7521")));
    }

    #[test]
    fn unresponsive_representative_cures_bypass() {
        let mut i = clean_base();
        i.taxpayer_represented = true;
        i.representative_unresponsive = true;
        let r = check(&i);
        assert!(!r.represented_taxpayer_bypass_violation);
    }

    #[test]
    fn representative_consent_cures_bypass() {
        let mut i = clean_base();
        i.taxpayer_represented = true;
        i.representative_consented_to_direct = true;
        let r = check(&i);
        assert!(!r.represented_taxpayer_bypass_violation);
    }

    #[test]
    fn workplace_contact_with_employer_prohibition_violation() {
        let mut i = clean_base();
        i.contacted_at_workplace = true;
        i.employer_prohibits_workplace_contact = true;
        let r = check(&i);
        assert!(r.workplace_contact_violation);
        assert!(r.violation_reasons.iter().any(|v| v.contains("§ 6304(a)(3)")
            && v.contains("place of employment")));
    }

    #[test]
    fn workplace_contact_without_prohibition_no_violation() {
        let mut i = clean_base();
        i.contacted_at_workplace = true;
        i.employer_prohibits_workplace_contact = false;
        let r = check(&i);
        assert!(!r.workplace_contact_violation);
    }

    #[test]
    fn threat_of_violence_harassment_violation() {
        let mut i = clean_base();
        i.harassment_conduct = HarassmentConduct::ThreatOfViolenceOrCriminal;
        let r = check(&i);
        assert!(r.harassment_violation);
        assert!(r.section_b_violation);
        assert!(r.violation_reasons.iter().any(|v| v.contains("§ 6304(b)(1)")));
    }

    #[test]
    fn obscene_language_harassment_violation() {
        let mut i = clean_base();
        i.harassment_conduct = HarassmentConduct::ObsceneOrProfaneLanguage;
        let r = check(&i);
        assert!(r.harassment_violation);
        assert!(r.violation_reasons.iter().any(|v| v.contains("§ 6304(b)(2)")));
    }

    #[test]
    fn repeated_phone_ringing_harassment_violation() {
        let mut i = clean_base();
        i.harassment_conduct = HarassmentConduct::RepeatedPhoneRinging;
        let r = check(&i);
        assert!(r.harassment_violation);
        assert!(r.violation_reasons.iter().any(|v| v.contains("§ 6304(b)(3)")));
    }

    #[test]
    fn anonymous_calls_harassment_violation() {
        let mut i = clean_base();
        i.harassment_conduct = HarassmentConduct::AnonymousCallsNoIdentity;
        let r = check(&i);
        assert!(r.harassment_violation);
        assert!(r.violation_reasons.iter().any(|v| v.contains("§ 6304(b)(4)")
            && v.contains("FDCPA § 804")));
    }

    #[test]
    fn reckless_intentional_damages_cap_at_1m() {
        let mut i = clean_base();
        i.harassment_conduct = HarassmentConduct::ThreatOfViolenceOrCriminal;
        i.damages_tier = DamagesTier::RecklessOrIntentional;
        i.actual_damages_cents = 1_500_000_00;
        let r = check(&i);
        assert_eq!(r.section_7433_damages_cap_cents, 100_000_000);
        assert_eq!(r.recoverable_damages_cents, 100_000_000);
    }

    #[test]
    fn negligence_damages_cap_at_100k() {
        let mut i = clean_base();
        i.harassment_conduct = HarassmentConduct::ThreatOfViolenceOrCriminal;
        i.damages_tier = DamagesTier::Negligence;
        i.actual_damages_cents = 200_000_00;
        let r = check(&i);
        assert_eq!(r.section_7433_damages_cap_cents, 10_000_000);
        assert_eq!(r.recoverable_damages_cents, 10_000_000);
    }

    #[test]
    fn negligence_actual_damages_under_cap_recoverable_in_full() {
        let mut i = clean_base();
        i.harassment_conduct = HarassmentConduct::ObsceneOrProfaneLanguage;
        i.damages_tier = DamagesTier::Negligence;
        i.actual_damages_cents = 50_000_00;
        let r = check(&i);
        assert_eq!(r.recoverable_damages_cents, 50_000_00);
    }

    #[test]
    fn no_violation_recoverable_zero() {
        let mut i = clean_base();
        i.actual_damages_cents = 500_000_00;
        let r = check(&i);
        assert_eq!(r.recoverable_damages_cents, 0);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&clean_base());
        assert!(r.citation.contains("§ 6304(a)-(c)"));
        assert!(r.citation.contains("RRA 98 § 3466"));
        assert!(r.citation.contains("15 USC § 1692"));
        assert!(r.citation.contains("IRM 5.1.10.3"));
        assert!(r.citation.contains("§ 7433"));
        assert!(r.citation.contains("§ 7811"));
        assert!(r.citation.contains("§ 7521"));
        assert!(r.citation.contains("§ 6306"));
    }

    #[test]
    fn note_pins_8am_9pm_window() {
        let r = check(&clean_base());
        assert!(r.notes.iter().any(|n| n.contains("8 a.m. to 9 p.m.")
            && n.contains("§ 6304(a)(1)")));
    }

    #[test]
    fn note_pins_rra_98_fdcpa_origin() {
        let r = check(&clean_base());
        assert!(r.notes.iter().any(|n| n.contains("RRA 98 § 3466")
            && n.contains("FDCPA")
            && n.contains("§ 1692")));
    }

    #[test]
    fn note_pins_damages_cap_tiers() {
        let r = check(&clean_base());
        assert!(r.notes.iter().any(|n| n.contains("$1,000,000")
            && n.contains("$100,000")
            && n.contains("§ 7433")));
    }

    #[test]
    fn multiple_section_a_violations_stack() {
        let mut i = clean_base();
        i.time_of_day = TimeOfDay::BeforeEightAm;
        i.taxpayer_represented = true;
        i.contacted_at_workplace = true;
        i.employer_prohibits_workplace_contact = true;
        let r = check(&i);
        assert!(r.section_a_violation);
        assert_eq!(r.violation_reasons.len(), 3);
    }

    #[test]
    fn section_a_and_b_combined_violations() {
        let mut i = clean_base();
        i.time_of_day = TimeOfDay::AfterNinePm;
        i.harassment_conduct = HarassmentConduct::RepeatedPhoneRinging;
        let r = check(&i);
        assert!(r.section_a_violation);
        assert!(r.section_b_violation);
        assert!(r.any_violation);
        assert_eq!(r.violation_reasons.len(), 2);
    }

    #[test]
    fn damages_cap_tier_invariant_reckless_higher_than_negligence() {
        let mut i_r = clean_base();
        i_r.damages_tier = DamagesTier::RecklessOrIntentional;
        i_r.harassment_conduct = HarassmentConduct::ThreatOfViolenceOrCriminal;
        let r_reckless = check(&i_r);

        let mut i_n = clean_base();
        i_n.damages_tier = DamagesTier::Negligence;
        i_n.harassment_conduct = HarassmentConduct::ThreatOfViolenceOrCriminal;
        let r_negligence = check(&i_n);

        assert!(
            r_reckless.section_7433_damages_cap_cents
                > r_negligence.section_7433_damages_cap_cents
        );
        assert_eq!(
            r_reckless.section_7433_damages_cap_cents
                / r_negligence.section_7433_damages_cap_cents,
            10
        );
    }

    #[test]
    fn harassment_truth_table() {
        for (conduct, exp_viol) in [
            (HarassmentConduct::None, false),
            (HarassmentConduct::ThreatOfViolenceOrCriminal, true),
            (HarassmentConduct::ObsceneOrProfaneLanguage, true),
            (HarassmentConduct::RepeatedPhoneRinging, true),
            (HarassmentConduct::AnonymousCallsNoIdentity, true),
        ] {
            let mut i = clean_base();
            i.harassment_conduct = conduct;
            let r = check(&i);
            assert_eq!(r.harassment_violation, exp_viol);
        }
    }

    #[test]
    fn time_truth_table() {
        for (tod, exp_viol) in [
            (TimeOfDay::Convenient8amTo9pm, false),
            (TimeOfDay::BeforeEightAm, true),
            (TimeOfDay::AfterNinePm, true),
            (TimeOfDay::KnownInconvenient, true),
        ] {
            let mut i = clean_base();
            i.time_of_day = tod;
            let r = check(&i);
            assert_eq!(r.time_violation, exp_viol);
        }
    }

    #[test]
    fn represented_bypass_truth_table() {
        for (represented, unresponsive, consented, exp_viol) in [
            (false, false, false, false),
            (true, false, false, true),
            (true, true, false, false),
            (true, false, true, false),
            (true, true, true, false),
        ] {
            let mut i = clean_base();
            i.taxpayer_represented = represented;
            i.representative_unresponsive = unresponsive;
            i.representative_consented_to_direct = consented;
            let r = check(&i);
            assert_eq!(r.represented_taxpayer_bypass_violation, exp_viol);
        }
    }

    #[test]
    fn court_order_overrides_all_section_a_restrictions() {
        let mut i = clean_base();
        i.consent_or_court_order = true;
        i.time_of_day = TimeOfDay::BeforeEightAm;
        i.taxpayer_represented = true;
        i.contacted_at_workplace = true;
        i.employer_prohibits_workplace_contact = true;
        let r = check(&i);
        assert!(!r.section_a_violation);
    }

    #[test]
    fn court_order_does_not_cure_harassment_section_b() {
        let mut i = clean_base();
        i.consent_or_court_order = true;
        i.harassment_conduct = HarassmentConduct::ThreatOfViolenceOrCriminal;
        let r = check(&i);
        assert!(!r.section_a_violation);
        assert!(r.section_b_violation);
        assert!(r.any_violation);
    }

    #[test]
    fn defensive_max_damages_clamp_to_reckless_cap() {
        let mut i = clean_base();
        i.harassment_conduct = HarassmentConduct::ObsceneOrProfaneLanguage;
        i.damages_tier = DamagesTier::RecklessOrIntentional;
        i.actual_damages_cents = u64::MAX;
        let r = check(&i);
        assert_eq!(r.recoverable_damages_cents, 100_000_000);
    }
}
