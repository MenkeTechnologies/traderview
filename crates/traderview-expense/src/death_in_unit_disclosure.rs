//! Death-in-unit landlord disclosure compliance check.
//!
//! Three jurisdictions have statutes on whether a landlord (or real estate
//! agent) must disclose prior deaths in the property. The default position
//! across the U.S. is caveat emptor — no affirmative statutory duty to
//! disclose. The carved-out exceptions are:
//!
//! **California (Cal. Civ. Code § 1710.2)** — broadest scope and longest
//! lookback. Landlord/seller MUST disclose all deaths within **3 years**
//! of the offer to rent/purchase, including natural deaths. Two important
//! carve-outs:
//! - **§ 1710.2(a) HIV/AIDS exception** — no requirement to disclose
//!   that an occupant was living with HIV or died from AIDS-related
//!   complications. Written to prevent discrimination.
//! - **§ 1710.2(b) intentional-misrepresentation override** — the 3-year
//!   window does NOT immunize a landlord who LIES in response to a direct
//!   inquiry. Even a 50-year-old death must be truthfully disclosed if
//!   directly asked.
//!
//! **South Dakota (S.D. Codified Laws § 43-4-44)** — narrowest scope
//! (homicides, suicides, felonies only) with **12-month** lookback.
//! Natural deaths are NOT disclosable. Applies to seller; lease/rental
//! disclosure typically follows.
//!
//! **Alaska (AS 08.88.615 — agent disclosure)** — limited to **real
//! estate agents** rather than landlord/owner directly. Agents must
//! disclose known murders/suicides within **12 months**; no liability
//! for deaths they do not know about.
//!
//! **Default** — no statewide statutory duty to disclose deaths. Common-
//! law caveat emptor applies. Direct-inquiry misrepresentation may still
//! be actionable under general fraud doctrines.
//!
//! Citations: Cal. Civ. Code § 1710.2(a) (3-year window + HIV/AIDS
//! exception); Cal. Civ. Code § 1710.2(b) (intentional-misrep override);
//! S.D. Codified Laws § 43-4-44 (12-month homicide/suicide/felony);
//! AS 08.88.615 (Alaska agent disclosure).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California1710_2,
    SouthDakota,
    Alaska,
    Default,
}

impl Regime {
    pub fn for_state(state: &str) -> Self {
        match state.trim().to_ascii_uppercase().as_str() {
            "CA" => Self::California1710_2,
            "SD" => Self::SouthDakota,
            "AK" => Self::Alaska,
            _ => Self::Default,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CauseOfDeath {
    NaturalCauses,
    Homicide,
    Suicide,
    HivAidsRelated,
    Other,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeathDisclosureInput {
    pub regime: Regime,
    pub death_occurred: bool,
    /// Months since the death. Drives the lookback window for each regime.
    pub months_since_death: u32,
    pub cause: CauseOfDeath,
    /// Whether the prospective tenant directly asked about deaths.
    /// Triggers Cal. Civ. Code § 1710.2(b) intentional-misrepresentation
    /// override — a direct lie is actionable regardless of the 3-year window.
    pub direct_inquiry_made: bool,
    /// Whether the landlord provided truthful disclosure to the inquirer.
    pub truthful_response_given: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DisclosureStatus {
    NotRequired,
    RequiredAndProvided,
    RequiredButNotProvided,
    /// Direct inquiry made + truthful response NOT given — even if 3-year
    /// window would otherwise immunize, § 1710.2(b) gives a cause of action.
    DirectInquiryMisrepresentation,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DeathDisclosureResult {
    pub regime: Regime,
    pub lookback_window_months: u32,
    pub disclosure_required: bool,
    pub direct_inquiry_misrep_exposure: bool,
    pub status: DisclosureStatus,
    pub landlord_compliant: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn check(input: &DeathDisclosureInput) -> DeathDisclosureResult {
    if !input.death_occurred {
        return DeathDisclosureResult {
            regime: input.regime,
            lookback_window_months: lookback_months(input.regime),
            disclosure_required: false,
            direct_inquiry_misrep_exposure: false,
            status: DisclosureStatus::NotRequired,
            landlord_compliant: true,
            citation: regime_base_citation(input.regime),
            note: "No death on the premises — no statutory disclosure obligation.".to_string(),
        };
    }

    let lookback = lookback_months(input.regime);
    let citation = regime_base_citation(input.regime);

    let within_window = input.months_since_death <= lookback;
    let cause_covered = cause_covered_by_regime(input.regime, input.cause);

    // Cal. Civ. Code § 1710.2(b) override: even outside the 3-year window
    // OR if the cause is HIV/AIDS (otherwise carve-out), intentional
    // misrepresentation in response to a direct inquiry is actionable.
    let direct_inquiry_misrep =
        input.direct_inquiry_made && !input.truthful_response_given && input.regime == Regime::California1710_2;

    let disclosure_required = within_window && cause_covered;

    if direct_inquiry_misrep {
        return DeathDisclosureResult {
            regime: input.regime,
            lookback_window_months: lookback,
            disclosure_required,
            direct_inquiry_misrep_exposure: true,
            status: DisclosureStatus::DirectInquiryMisrepresentation,
            landlord_compliant: false,
            citation: "Cal. Civ. Code § 1710.2(b) — direct-inquiry intentional-misrepresentation override; 3-year window does NOT immunize a lie",
            note:
                "DIRECT INQUIRY MADE and TRUTHFUL RESPONSE NOT given — § 1710.2(b) makes this actionable under intentional-misrepresentation regardless of the 3-year window."
                    .to_string(),
        };
    }

    if !disclosure_required {
        let reason = if !within_window {
            format!(
                "Death occurred {} months ago — outside the {}-month lookback window.",
                input.months_since_death, lookback
            )
        } else {
            format!(
                "Cause of death {:?} is not within the disclosable categories for the applicable regime.",
                input.cause
            )
        };
        return DeathDisclosureResult {
            regime: input.regime,
            lookback_window_months: lookback,
            disclosure_required: false,
            direct_inquiry_misrep_exposure: false,
            status: DisclosureStatus::NotRequired,
            landlord_compliant: true,
            citation,
            note: format!("No statutory disclosure obligation. {}", reason),
        };
    }

    let provided = input.truthful_response_given || !input.direct_inquiry_made;
    DeathDisclosureResult {
        regime: input.regime,
        lookback_window_months: lookback,
        disclosure_required: true,
        direct_inquiry_misrep_exposure: false,
        status: if provided {
            DisclosureStatus::RequiredAndProvided
        } else {
            DisclosureStatus::RequiredButNotProvided
        },
        landlord_compliant: provided,
        citation,
        note: if provided {
            "Disclosure required and provided — landlord compliant.".to_string()
        } else {
            "Disclosure REQUIRED and NOT provided — landlord noncompliant.".to_string()
        },
    }
}

fn lookback_months(regime: Regime) -> u32 {
    match regime {
        Regime::California1710_2 => 36,
        Regime::SouthDakota => 12,
        Regime::Alaska => 12,
        Regime::Default => 0,
    }
}

fn regime_base_citation(regime: Regime) -> &'static str {
    match regime {
        Regime::California1710_2 => {
            "Cal. Civ. Code § 1710.2(a) — 3-year (36-month) disclosure window for all deaths; HIV/AIDS carve-out per § 1710.2(a)(1)"
        }
        Regime::SouthDakota => {
            "S.D. Codified Laws § 43-4-44 — 12-month disclosure window for homicides, suicides, and felonies only"
        }
        Regime::Alaska => {
            "AS 08.88.615 — 12-month real-estate-agent disclosure window for known murders and suicides"
        }
        Regime::Default => {
            "No statewide death-in-unit disclosure statute identified — common-law caveat emptor applies"
        }
    }
}

fn cause_covered_by_regime(regime: Regime, cause: CauseOfDeath) -> bool {
    match (regime, cause) {
        // CA HIV/AIDS carve-out: § 1710.2(a)(1) excludes HIV/AIDS from
        // mandatory disclosure to prevent discrimination.
        (Regime::California1710_2, CauseOfDeath::HivAidsRelated) => false,
        // CA covers all other causes including natural deaths.
        (Regime::California1710_2, _) => true,
        // SD: homicides, suicides, felonies only — natural causes not covered.
        (Regime::SouthDakota, CauseOfDeath::Homicide | CauseOfDeath::Suicide) => true,
        (Regime::SouthDakota, _) => false,
        // Alaska: murders + suicides only (agent disclosure).
        (Regime::Alaska, CauseOfDeath::Homicide | CauseOfDeath::Suicide) => true,
        (Regime::Alaska, _) => false,
        (Regime::Default, _) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        regime: Regime,
        occurred: bool,
        months: u32,
        cause: CauseOfDeath,
        direct_inquiry: bool,
        truthful: bool,
    ) -> DeathDisclosureInput {
        DeathDisclosureInput {
            regime,
            death_occurred: occurred,
            months_since_death: months,
            cause,
            direct_inquiry_made: direct_inquiry,
            truthful_response_given: truthful,
        }
    }

    #[test]
    fn ca_natural_death_within_3_years_must_disclose() {
        let r = check(&input(
            Regime::California1710_2,
            true,
            12,
            CauseOfDeath::NaturalCauses,
            false,
            false,
        ));
        assert!(r.disclosure_required);
        assert_eq!(r.status, DisclosureStatus::RequiredAndProvided);
        // Even with truthful_response_given=false, no direct inquiry → no misrep.
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ca_at_36_month_boundary_still_within_window() {
        let r = check(&input(
            Regime::California1710_2,
            true,
            36,
            CauseOfDeath::NaturalCauses,
            false,
            false,
        ));
        assert!(r.disclosure_required);
    }

    #[test]
    fn ca_at_37_months_outside_window() {
        let r = check(&input(
            Regime::California1710_2,
            true,
            37,
            CauseOfDeath::NaturalCauses,
            false,
            false,
        ));
        assert!(!r.disclosure_required);
        assert!(r.note.contains("37 months ago"));
    }

    #[test]
    fn ca_hiv_aids_carve_out_not_disclosable() {
        let r = check(&input(
            Regime::California1710_2,
            true,
            12,
            CauseOfDeath::HivAidsRelated,
            false,
            false,
        ));
        assert!(!r.disclosure_required);
        assert!(r.note.contains("not within the disclosable categories"));
    }

    #[test]
    fn ca_direct_inquiry_misrep_overrides_3_year_window() {
        // 50-year-old death + direct inquiry + lie = actionable.
        let r = check(&input(
            Regime::California1710_2,
            true,
            600,
            CauseOfDeath::NaturalCauses,
            true,
            false,
        ));
        assert_eq!(r.status, DisclosureStatus::DirectInquiryMisrepresentation);
        assert!(r.direct_inquiry_misrep_exposure);
        assert!(!r.landlord_compliant);
        assert!(r.citation.contains("§ 1710.2(b)"));
    }

    #[test]
    fn ca_direct_inquiry_truthful_response_compliant() {
        let r = check(&input(
            Regime::California1710_2,
            true,
            600,
            CauseOfDeath::NaturalCauses,
            true,
            true,
        ));
        assert!(!r.direct_inquiry_misrep_exposure);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ca_hiv_aids_direct_inquiry_lie_still_actionable() {
        // The HIV/AIDS carve-out is about AFFIRMATIVE disclosure; § 1710.2(b)
        // still applies to lies in response to direct inquiry.
        let r = check(&input(
            Regime::California1710_2,
            true,
            12,
            CauseOfDeath::HivAidsRelated,
            true,
            false,
        ));
        assert!(r.direct_inquiry_misrep_exposure);
        assert_eq!(r.status, DisclosureStatus::DirectInquiryMisrepresentation);
    }

    #[test]
    fn sd_homicide_within_12_months_must_disclose() {
        let r = check(&input(
            Regime::SouthDakota,
            true,
            6,
            CauseOfDeath::Homicide,
            false,
            false,
        ));
        assert!(r.disclosure_required);
    }

    #[test]
    fn sd_natural_death_not_disclosable() {
        let r = check(&input(
            Regime::SouthDakota,
            true,
            6,
            CauseOfDeath::NaturalCauses,
            false,
            false,
        ));
        assert!(!r.disclosure_required);
        assert!(r.citation.contains("homicides, suicides, and felonies only"));
    }

    #[test]
    fn sd_homicide_outside_12_months_not_disclosable() {
        let r = check(&input(
            Regime::SouthDakota,
            true,
            13,
            CauseOfDeath::Homicide,
            false,
            false,
        ));
        assert!(!r.disclosure_required);
    }

    #[test]
    fn sd_at_12_month_boundary_within_window() {
        let r = check(&input(
            Regime::SouthDakota,
            true,
            12,
            CauseOfDeath::Suicide,
            false,
            false,
        ));
        assert!(r.disclosure_required);
    }

    #[test]
    fn sd_no_direct_inquiry_override() {
        let r = check(&input(
            Regime::SouthDakota,
            true,
            100,
            CauseOfDeath::Homicide,
            true,
            false,
        ));
        // SD does not have a § 1710.2(b)-equivalent override.
        assert!(!r.direct_inquiry_misrep_exposure);
    }

    #[test]
    fn ak_homicide_within_12_months_must_disclose() {
        let r = check(&input(
            Regime::Alaska,
            true,
            6,
            CauseOfDeath::Homicide,
            false,
            false,
        ));
        assert!(r.disclosure_required);
        assert!(r.citation.contains("AS 08.88.615"));
    }

    #[test]
    fn ak_natural_cause_not_disclosable() {
        let r = check(&input(
            Regime::Alaska,
            true,
            6,
            CauseOfDeath::NaturalCauses,
            false,
            false,
        ));
        assert!(!r.disclosure_required);
    }

    #[test]
    fn default_regime_no_obligation() {
        let r = check(&input(
            Regime::Default,
            true,
            6,
            CauseOfDeath::Homicide,
            false,
            false,
        ));
        assert!(!r.disclosure_required);
        assert!(r.citation.contains("caveat emptor"));
    }

    #[test]
    fn no_death_no_obligation_across_all_regimes() {
        for regime in [
            Regime::California1710_2,
            Regime::SouthDakota,
            Regime::Alaska,
            Regime::Default,
        ] {
            let r = check(&input(
                regime,
                false,
                0,
                CauseOfDeath::NaturalCauses,
                false,
                false,
            ));
            assert!(!r.disclosure_required);
            assert!(r.landlord_compliant);
        }
    }

    #[test]
    fn ca_longest_lookback_window() {
        let ca_r = check(&input(
            Regime::California1710_2,
            true,
            0,
            CauseOfDeath::NaturalCauses,
            false,
            false,
        ));
        let sd_r = check(&input(
            Regime::SouthDakota,
            true,
            0,
            CauseOfDeath::Homicide,
            false,
            false,
        ));
        let ak_r = check(&input(
            Regime::Alaska,
            true,
            0,
            CauseOfDeath::Homicide,
            false,
            false,
        ));
        assert_eq!(ca_r.lookback_window_months, 36);
        assert_eq!(sd_r.lookback_window_months, 12);
        assert_eq!(ak_r.lookback_window_months, 12);
        assert!(ca_r.lookback_window_months > sd_r.lookback_window_months);
        assert!(ca_r.lookback_window_months > ak_r.lookback_window_months);
    }

    #[test]
    fn ca_broadest_cause_coverage() {
        // CA covers natural causes; SD + AK do not.
        let ca_r = check(&input(
            Regime::California1710_2,
            true,
            12,
            CauseOfDeath::NaturalCauses,
            false,
            false,
        ));
        let sd_r = check(&input(
            Regime::SouthDakota,
            true,
            12,
            CauseOfDeath::NaturalCauses,
            false,
            false,
        ));
        let ak_r = check(&input(
            Regime::Alaska,
            true,
            12,
            CauseOfDeath::NaturalCauses,
            false,
            false,
        ));
        assert!(ca_r.disclosure_required);
        assert!(!sd_r.disclosure_required);
        assert!(!ak_r.disclosure_required);
    }

    #[test]
    fn state_routing_ca_sd_ak_default() {
        assert_eq!(Regime::for_state("CA"), Regime::California1710_2);
        assert_eq!(Regime::for_state("SD"), Regime::SouthDakota);
        assert_eq!(Regime::for_state("AK"), Regime::Alaska);
        assert_eq!(Regime::for_state("NY"), Regime::Default);
        assert_eq!(Regime::for_state("TX"), Regime::Default);
    }

    #[test]
    fn state_routing_case_insensitive() {
        assert_eq!(Regime::for_state("ca"), Regime::California1710_2);
        assert_eq!(Regime::for_state("Sd"), Regime::SouthDakota);
        assert_eq!(Regime::for_state("aK"), Regime::Alaska);
    }

    #[test]
    fn ca_only_has_direct_inquiry_misrep_override() {
        // Same direct-inquiry-with-lie scenario across regimes — only CA
        // triggers the misrep exposure.
        let causes_exposure = |regime: Regime| {
            check(&input(
                regime,
                true,
                100,
                CauseOfDeath::Homicide,
                true,
                false,
            ))
            .direct_inquiry_misrep_exposure
        };
        assert!(causes_exposure(Regime::California1710_2));
        assert!(!causes_exposure(Regime::SouthDakota));
        assert!(!causes_exposure(Regime::Alaska));
        assert!(!causes_exposure(Regime::Default));
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let ca = check(&input(
            Regime::California1710_2,
            true,
            12,
            CauseOfDeath::NaturalCauses,
            false,
            false,
        ));
        assert!(ca.citation.contains("§ 1710.2(a)"));
        assert!(ca.citation.contains("HIV/AIDS carve-out"));

        let sd = check(&input(
            Regime::SouthDakota,
            true,
            12,
            CauseOfDeath::Homicide,
            false,
            false,
        ));
        assert!(sd.citation.contains("§ 43-4-44"));
        assert!(sd.citation.contains("12-month"));

        let ak = check(&input(
            Regime::Alaska,
            true,
            6,
            CauseOfDeath::Homicide,
            false,
            false,
        ));
        assert!(ak.citation.contains("08.88.615"));
        assert!(ak.citation.contains("agent disclosure"));
    }

    #[test]
    fn ca_not_provided_required_disclosure_when_no_inquiry() {
        // The "provided" determination collapses to true when no direct
        // inquiry — affirmative disclosure default is offer-of-rental
        // disclosure, which falls outside this module's scope.
        let r = check(&input(
            Regime::California1710_2,
            true,
            12,
            CauseOfDeath::Homicide,
            false,
            false,
        ));
        // No inquiry → not flagged as noncompliant by this check
        // (offer-of-rental disclosure is the relevant compliance gate).
        assert!(r.landlord_compliant);
        assert_eq!(r.status, DisclosureStatus::RequiredAndProvided);
    }
}
