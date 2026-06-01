//! State carpet-replacement useful-life landlord compliance check for
//! security-deposit deductions.
//!
//! The most heavily litigated security-deposit dispute: landlord
//! charges tenant for carpet replacement at move-out; tenant argues
//! the carpet was old/depreciated and the charge exceeds the landlord's
//! actual loss. The doctrine has crystallized into per-state "useful
//! life" schedules — landlord may deduct only the remaining-life
//! fraction of replacement cost, never the full amount.
//!
//! Colorado (HB 25-1249, eff. 2026-01-01) — explicit STATUTORY 10-year
//! carpet useful life. Landlord MAY NOT DEEM carpet "substantially and
//! irreparably damaged" if it has not been replaced with new carpet
//! within the 10 years preceding lease termination — meaning a 10-year-
//! old carpet is effectively a $0 replacement cost. If actual cause
//! exists for replacement (substantial + irreparable damage that
//! exceeds normal wear-and-tear), landlord may retain only the MINIMUM
//! AMOUNT NECESSARY to replace.
//!
//! California — common-law 8-year carpet useful life developed
//! through landlord-tenant litigation (Killough v. McManus + progeny).
//! Landlord may charge prorated remaining life only.
//!
//! HUD Federal Section 8 — schedule of 5-7 year carpet useful life
//! for federally subsidized housing. Caretakers/owners must use the
//! federal schedule for any units receiving § 8 voucher payments.
//!
//! Default — common-law useful-life prorating (typical 5-10 year
//! ranges depending on state caselaw); landlord must prove actual
//! loss net of normal depreciation.
//!
//! Citations: Colo. Rev. Stat. § 38-12-104 (as amended by HB 25-1249
//! eff. 2026-01-01); Cal. common law (Killough v. McManus 19 Cal. App.
//! 3d 1141 (1971) and progeny); HUD Handbook 4350.3 chap. 6 useful-
//! life schedules; common-law actual-damages-net-of-depreciation
//! doctrine.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    ColoradoHb251249,
    California,
    HudSection8,
    Default,
}

impl Regime {
    pub fn for_state_and_section_8(state: &str, section_8: bool) -> Self {
        if section_8 {
            return Self::HudSection8;
        }
        match state.trim().to_ascii_uppercase().as_str() {
            "CO" => Self::ColoradoHb251249,
            "CA" => Self::California,
            _ => Self::Default,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CarpetReplacementInput {
    pub regime: Regime,
    pub carpet_age_years: u32,
    pub replacement_cost_cents: i64,
    /// Whether the damage exceeds normal wear-and-tear. Colorado's
    /// "substantial and irreparable damage" gate requires this.
    pub substantial_irreparable_damage: bool,
    /// Whether the carpet was replaced new within the regime's useful-
    /// life window before lease termination. Colorado's regression-
    /// critical gate: cannot deem damage unless replacement happened
    /// within useful-life window.
    pub replaced_new_within_useful_life: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CarpetReplacementResult {
    pub regime: Regime,
    pub useful_life_years: u32,
    pub carpet_past_useful_life: bool,
    pub allowed_deduction_cents: i64,
    pub remaining_life_years: u32,
    pub citation: &'static str,
    pub note: String,
}

pub fn check(input: &CarpetReplacementInput) -> CarpetReplacementResult {
    let cost = input.replacement_cost_cents.max(0);
    let useful_life = useful_life_years_for(input.regime);
    let past_useful = input.carpet_age_years >= useful_life;
    let remaining_life = useful_life.saturating_sub(input.carpet_age_years);

    let allowed = match input.regime {
        Regime::ColoradoHb251249 => {
            // CO HB 25-1249: landlord cannot deem damage unless
            // (a) substantial + irreparable AND (b) carpet was replaced
            // new within 10 years AND (c) carpet still has remaining life.
            let gate_open = input.substantial_irreparable_damage
                && input.replaced_new_within_useful_life
                && !past_useful;
            if gate_open {
                // Minimum amount necessary, prorated by remaining life.
                ((cost as i128 * remaining_life as i128) / useful_life as i128) as i64
            } else {
                0
            }
        }
        Regime::California | Regime::HudSection8 | Regime::Default => {
            // Common-law prorating: allowed = cost × remaining_life /
            // useful_life. No special damage gate.
            if past_useful {
                0
            } else {
                ((cost as i128 * remaining_life as i128) / useful_life as i128) as i64
            }
        }
    };

    let citation = match input.regime {
        Regime::ColoradoHb251249 => {
            "Colo. Rev. Stat. § 38-12-104 as amended by HB 25-1249 (eff. 2026-01-01) — STATUTORY 10-year carpet useful life; landlord may not deem damage unless substantial + irreparable AND replaced new within 10 years; may retain only MINIMUM AMOUNT NECESSARY"
        }
        Regime::California => {
            "California common law — Killough v. McManus 19 Cal. App. 3d 1141 (1971) + progeny — 8-year carpet useful life; landlord may charge prorated remaining-life fraction of replacement cost"
        }
        Regime::HudSection8 => {
            "HUD Handbook 4350.3 chap. 6 — federally subsidized § 8 useful-life schedule (5-7 year carpet); applies to all units receiving § 8 voucher payments"
        }
        Regime::Default => {
            "Common-law actual-damages-net-of-depreciation doctrine — landlord must prove actual loss net of normal wear-and-tear depreciation"
        }
    };

    let note = format!(
        "Carpet age {} years; {} useful life {} years; remaining life {} years; replacement cost {} cents; substantial+irreparable damage = {}; replaced-new-within-useful-life = {}; allowed deduction = {} cents.{}",
        input.carpet_age_years,
        match input.regime {
            Regime::ColoradoHb251249 => "Colorado",
            Regime::California => "California common-law",
            Regime::HudSection8 => "HUD § 8",
            Regime::Default => "common-law default",
        },
        useful_life,
        remaining_life,
        cost,
        input.substantial_irreparable_damage,
        input.replaced_new_within_useful_life,
        allowed,
        if past_useful { " (carpet PAST useful life — fully depreciated)" } else { "" },
    );

    CarpetReplacementResult {
        regime: input.regime,
        useful_life_years: useful_life,
        carpet_past_useful_life: past_useful,
        allowed_deduction_cents: allowed,
        remaining_life_years: remaining_life,
        citation,
        note,
    }
}

fn useful_life_years_for(regime: Regime) -> u32 {
    match regime {
        Regime::ColoradoHb251249 => 10,
        Regime::California => 8,
        Regime::HudSection8 => 7,
        Regime::Default => 7,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        regime: Regime,
        age: u32,
        cost: i64,
        substantial: bool,
        replaced_recently: bool,
    ) -> CarpetReplacementInput {
        CarpetReplacementInput {
            regime,
            carpet_age_years: age,
            replacement_cost_cents: cost,
            substantial_irreparable_damage: substantial,
            replaced_new_within_useful_life: replaced_recently,
        }
    }

    #[test]
    fn co_carpet_past_10_years_zero_deduction() {
        let r = check(&input(
            Regime::ColoradoHb251249,
            11,
            2_000_00,
            true,
            true,
        ));
        assert_eq!(r.allowed_deduction_cents, 0);
        assert!(r.carpet_past_useful_life);
        assert_eq!(r.useful_life_years, 10);
    }

    #[test]
    fn co_carpet_at_10_year_boundary_past() {
        // At 10 years, carpet is considered past useful life.
        let r = check(&input(
            Regime::ColoradoHb251249,
            10,
            2_000_00,
            true,
            true,
        ));
        assert!(r.carpet_past_useful_life);
        assert_eq!(r.allowed_deduction_cents, 0);
    }

    #[test]
    fn co_carpet_9_years_with_damage_5_years_remaining_proration() {
        // 9 years old, 1 year remaining. Cost $2000 × 1/10 = $200.
        let r = check(&input(
            Regime::ColoradoHb251249,
            9,
            2_000_00,
            true,
            true,
        ));
        assert_eq!(r.allowed_deduction_cents, 200_00);
        assert_eq!(r.remaining_life_years, 1);
    }

    #[test]
    fn co_no_substantial_damage_no_deduction() {
        // Regression-critical: without substantial+irreparable damage,
        // CO HB 25-1249 bars deduction.
        let r = check(&input(
            Regime::ColoradoHb251249,
            5,
            2_000_00,
            false,
            true,
        ));
        assert_eq!(r.allowed_deduction_cents, 0);
        assert!(r.citation.contains("substantial + irreparable"));
    }

    #[test]
    fn co_not_replaced_new_within_10_years_no_deduction() {
        // Regression-critical: CO HB 25-1249 — cannot deem damage unless
        // carpet was replaced new within useful-life window.
        let r = check(&input(
            Regime::ColoradoHb251249,
            5,
            2_000_00,
            true,
            false,
        ));
        assert_eq!(r.allowed_deduction_cents, 0);
    }

    #[test]
    fn ca_carpet_past_8_years_zero_deduction() {
        let r = check(&input(Regime::California, 9, 2_000_00, true, true));
        assert_eq!(r.allowed_deduction_cents, 0);
        assert!(r.carpet_past_useful_life);
        assert_eq!(r.useful_life_years, 8);
    }

    #[test]
    fn ca_carpet_at_8_year_boundary_past() {
        let r = check(&input(Regime::California, 8, 2_000_00, true, true));
        assert!(r.carpet_past_useful_life);
        assert_eq!(r.allowed_deduction_cents, 0);
    }

    #[test]
    fn ca_carpet_4_year_proration_half() {
        // 4 of 8 years used → 4 years remaining → 50% × $2000 = $1000.
        let r = check(&input(Regime::California, 4, 2_000_00, true, true));
        assert_eq!(r.allowed_deduction_cents, 1_000_00);
        assert_eq!(r.remaining_life_years, 4);
    }

    #[test]
    fn ca_no_damage_gate_proration_still_applies() {
        // California does NOT have a substantial+irreparable damage gate —
        // landlord can charge prorated remaining-life cost regardless of
        // damage severity (subject to normal-wear-and-tear analysis).
        let r = check(&input(Regime::California, 4, 2_000_00, false, true));
        assert_eq!(r.allowed_deduction_cents, 1_000_00);
    }

    #[test]
    fn hud_section_8_carpet_at_7_years_past() {
        let r = check(&input(Regime::HudSection8, 7, 1_000_00, true, true));
        assert_eq!(r.allowed_deduction_cents, 0);
        assert!(r.carpet_past_useful_life);
        assert_eq!(r.useful_life_years, 7);
    }

    #[test]
    fn hud_section_8_carpet_3_year_proration() {
        // 3 of 7 used → 4 remaining → cost × 4/7.
        let r = check(&input(Regime::HudSection8, 3, 700_00, true, true));
        // $700 × 4/7 = $400.
        assert_eq!(r.allowed_deduction_cents, 400_00);
    }

    #[test]
    fn default_7_year_useful_life() {
        let r = check(&input(Regime::Default, 3, 700_00, true, true));
        assert_eq!(r.useful_life_years, 7);
        assert_eq!(r.allowed_deduction_cents, 400_00);
    }

    #[test]
    fn default_past_7_years_zero_deduction() {
        let r = check(&input(Regime::Default, 8, 1_000_00, true, true));
        assert_eq!(r.allowed_deduction_cents, 0);
    }

    #[test]
    fn negative_cost_clamped() {
        let r = check(&input(Regime::California, 4, -1, true, true));
        assert_eq!(r.allowed_deduction_cents, 0);
    }

    #[test]
    fn state_routing_co_ca_default_section8() {
        assert_eq!(
            Regime::for_state_and_section_8("CO", false),
            Regime::ColoradoHb251249
        );
        assert_eq!(
            Regime::for_state_and_section_8("CA", false),
            Regime::California
        );
        assert_eq!(
            Regime::for_state_and_section_8("TX", false),
            Regime::Default
        );
        assert_eq!(
            Regime::for_state_and_section_8("CA", true),
            Regime::HudSection8
        );
        assert_eq!(
            Regime::for_state_and_section_8("CO", true),
            Regime::HudSection8
        );
    }

    #[test]
    fn section_8_overrides_state_routing() {
        // § 8 voucher applies federal HUD schedule regardless of state.
        let r_ca_with_section_8 = Regime::for_state_and_section_8("CA", true);
        assert_eq!(r_ca_with_section_8, Regime::HudSection8);
    }

    #[test]
    fn co_useful_life_longest_at_10_years() {
        // CO 10 > CA 8 > HUD 7 = Default 7. Regression-critical ordering.
        let co = check(&input(
            Regime::ColoradoHb251249,
            0,
            2_000_00,
            true,
            true,
        ));
        let ca = check(&input(Regime::California, 0, 2_000_00, true, true));
        let hud = check(&input(Regime::HudSection8, 0, 2_000_00, true, true));
        let d = check(&input(Regime::Default, 0, 2_000_00, true, true));
        assert!(co.useful_life_years > ca.useful_life_years);
        assert!(ca.useful_life_years > hud.useful_life_years);
        assert_eq!(hud.useful_life_years, d.useful_life_years);
    }

    #[test]
    fn only_co_has_substantial_damage_gate() {
        // Same no-substantial-damage scenario across regimes. CO →
        // zero deduction; others → prorated.
        let co = check(&input(
            Regime::ColoradoHb251249,
            5,
            2_000_00,
            false,
            true,
        ));
        let ca = check(&input(Regime::California, 5, 2_000_00, false, true));
        let hud = check(&input(Regime::HudSection8, 5, 2_000_00, false, true));
        let d = check(&input(Regime::Default, 5, 2_000_00, false, true));
        assert_eq!(co.allowed_deduction_cents, 0);
        assert!(ca.allowed_deduction_cents > 0);
        assert!(hud.allowed_deduction_cents > 0);
        assert!(d.allowed_deduction_cents > 0);
    }

    #[test]
    fn only_co_requires_replaced_new_within_useful_life() {
        // Same not-recently-replaced scenario. CO → zero; others → prorated.
        let co = check(&input(
            Regime::ColoradoHb251249,
            5,
            2_000_00,
            true,
            false,
        ));
        let ca = check(&input(Regime::California, 5, 2_000_00, true, false));
        assert_eq!(co.allowed_deduction_cents, 0);
        assert!(ca.allowed_deduction_cents > 0);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let co = check(&input(
            Regime::ColoradoHb251249,
            5,
            2_000_00,
            true,
            true,
        ));
        assert!(co.citation.contains("HB 25-1249"));
        assert!(co.citation.contains("2026-01-01"));

        let ca = check(&input(Regime::California, 5, 2_000_00, true, true));
        assert!(ca.citation.contains("Killough"));
        assert!(ca.citation.contains("8-year"));

        let hud = check(&input(Regime::HudSection8, 3, 700_00, true, true));
        assert!(hud.citation.contains("HUD Handbook 4350.3"));

        let d = check(&input(Regime::Default, 3, 700_00, true, true));
        assert!(d.citation.contains("Common-law"));
    }

    #[test]
    fn ca_new_carpet_full_replacement_cost_minus_proration() {
        // Brand new carpet (0 years) — landlord can charge full
        // replacement cost (entire useful life remaining).
        let r = check(&input(Regime::California, 0, 2_000_00, true, true));
        assert_eq!(r.allowed_deduction_cents, 2_000_00);
        assert_eq!(r.remaining_life_years, 8);
    }

    #[test]
    fn co_with_damage_and_recent_replacement_proration_applies() {
        // CO gates: substantial damage YES + replaced within useful life YES
        // → standard proration applies.
        let r = check(&input(
            Regime::ColoradoHb251249,
            5,
            2_000_00,
            true,
            true,
        ));
        // 5 of 10 used → 5 remaining → $2000 × 5/10 = $1000.
        assert_eq!(r.allowed_deduction_cents, 1_000_00);
    }
}
