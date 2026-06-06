//! Rental property bedroom egress window requirement
//! compliance — when must a trader-landlord ensure every
//! bedroom in a residential rental unit has an emergency
//! escape and rescue opening (EERO) meeting International
//! Residential Code (IRC) § R310 minimum standards?
//! Trader-landlord critical for any rental owner: failure
//! to provide compliant egress window breaches implied
//! warranty of habitability + creates fire-safety liability
//! exposure + violates state building code adoption (CA +
//! NY + 49 other state IRC adoptions).
//!
//! Distinct from siblings `detector_requirements` (smoke +
//! CO detector hardware), `fire_sprinkler_disclosure`
//! (suppression system), `tenant_fire_safety_plan_
//! disclosure` (HPD signage), and `window_guard_
//! requirements` (NYC HMC § 27-2046.1 child-safety).
//!
//! **Two regimes**:
//!
//! **IRC R310 Standard (49 state-adopting jurisdictions
//! including California + New York)**:
//! Every sleeping room must have at least one Emergency
//! Escape and Rescue Opening (EERO) meeting ALL FOUR
//! requirements simultaneously:
//! 1. § R310.2.1 **minimum net clear opening = 5.7 sq ft**
//!    (5.0 sq ft at grade-floor exception when sill ≤ 44
//!    inches above/below adjacent ground)
//! 2. § R310.2.1 **minimum net clear opening height = 24
//!    inches**
//! 3. § R310.2.1 **minimum net clear opening width = 20
//!    inches**
//! 4. § R310.2.2 **maximum sill height = 44 inches above
//!    finished floor**
//!
//! § R310.2.3 window well requirements (if EERO below
//! adjacent ground level):
//! - Minimum horizontal area = 9 sq ft
//! - Minimum horizontal projection = 36 inches from
//!   exterior face of wall
//! - Permanently affixed ladder/steps required if well depth
//!   > 44 inches
//!
//! **Default** — no statewide IRC adoption; local building
//! code or common-law habitability governs.
//!
//! Citations: IRC § R310 (Emergency Escape and Rescue
//! Openings); IRC § R310.2.1 (minimum opening dimensions);
//! IRC § R310.2.2 (sill height); IRC § R310.2.3 (window
//! wells); California Residential Code § R310 (CA IRC
//! adoption); 2020 New York State Residential Code § R310
//! (NY IRC adoption).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    IrcR310,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalBedroomEgressWindowInput {
    pub regime: Regime,
    /// Net clear opening area in square inches (1 sq ft = 144
    /// sq in).
    pub net_clear_opening_sq_in: u32,
    /// Net clear opening height in inches.
    pub net_clear_height_in: u32,
    /// Net clear opening width in inches.
    pub net_clear_width_in: u32,
    /// Sill height above finished floor in inches.
    pub sill_height_in: u32,
    /// Whether the EERO is at grade-floor level (sill ≤ 44
    /// inches above or below adjacent ground level).
    pub at_grade_floor: bool,
    /// Whether the EERO is below adjacent ground level
    /// (requires window well per § R310.2.3).
    pub below_grade_level: bool,
    /// Window well horizontal area in square inches.
    pub window_well_area_sq_in: u32,
    /// Window well horizontal projection in inches.
    pub window_well_projection_in: u32,
    /// Window well depth in inches.
    pub window_well_depth_in: u32,
    /// Whether window well has permanently affixed
    /// ladder/steps.
    pub window_well_ladder_provided: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalBedroomEgressWindowResult {
    pub compliant: bool,
    pub required_opening_sq_in: u32,
    pub required_height_in: u32,
    pub required_width_in: u32,
    pub maximum_sill_height_in: u32,
    pub window_well_required: bool,
    pub window_well_ladder_required: bool,
    pub habitability_breach: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &RentalBedroomEgressWindowInput) -> RentalBedroomEgressWindowResult {
    match input.regime {
        Regime::IrcR310 => check_irc_r310(input),
        Regime::Default => check_default(input),
    }
}

fn check_irc_r310(input: &RentalBedroomEgressWindowInput) -> RentalBedroomEgressWindowResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "IRC § R310.2.1 — every sleeping room must have an Emergency Escape and Rescue Opening (EERO) meeting all four requirements: net clear opening ≥ 5.7 sq ft (5.0 sq ft grade exception); height ≥ 24 in; width ≥ 20 in; sill ≤ 44 in above floor"
            .to_string(),
        "IRC § R310.2.2 — maximum sill height 44 inches measured from finished floor to bottom of EERO clear opening"
            .to_string(),
        "IRC § R310.2.3 — if EERO below adjacent ground level: window well minimum horizontal area 9 sq ft + horizontal projection 36 in from exterior face of wall; permanently affixed ladder/steps required if well depth > 44 in"
            .to_string(),
        "IRC § R310 adopted by California Residential Code § R310 + 2020 New York State Residential Code § R310 + 49 other state jurisdictions; ALL FOUR dimensions must be met simultaneously"
            .to_string(),
    ];

    let required_opening = if input.at_grade_floor { 720 } else { 821 };
    let required_height = 24;
    let required_width = 20;
    let max_sill = 44;

    if input.net_clear_opening_sq_in < required_opening {
        violations.push(format!(
            "IRC § R310.2.1 — net clear opening {} sq in below required {} sq in ({} sq ft)",
            input.net_clear_opening_sq_in,
            required_opening,
            if input.at_grade_floor { "5.0" } else { "5.7" }
        ));
    }

    if input.net_clear_height_in < required_height {
        violations.push(format!(
            "IRC § R310.2.1 — net clear height {} in below required {} in minimum",
            input.net_clear_height_in, required_height
        ));
    }

    if input.net_clear_width_in < required_width {
        violations.push(format!(
            "IRC § R310.2.1 — net clear width {} in below required {} in minimum",
            input.net_clear_width_in, required_width
        ));
    }

    if input.sill_height_in > max_sill {
        violations.push(format!(
            "IRC § R310.2.2 — sill height {} in exceeds maximum {} in above finished floor",
            input.sill_height_in, max_sill
        ));
    }

    let well_required = input.below_grade_level;
    let ladder_required = well_required && input.window_well_depth_in > 44;

    if well_required {
        let required_well_area = 1296;
        if input.window_well_area_sq_in < required_well_area {
            violations.push(format!(
                "IRC § R310.2.3 — window well area {} sq in below required {} sq in (9 sq ft minimum)",
                input.window_well_area_sq_in, required_well_area
            ));
        }
        if input.window_well_projection_in < 36 {
            violations.push(format!(
                "IRC § R310.2.3 — window well projection {} in below required 36 in minimum",
                input.window_well_projection_in
            ));
        }
        if ladder_required && !input.window_well_ladder_provided {
            violations.push(format!(
                "IRC § R310.2.3 — window well depth {} in exceeds 44 in; permanently affixed ladder/steps required",
                input.window_well_depth_in
            ));
        }
    }

    let habitability_breach = !violations.is_empty();

    RentalBedroomEgressWindowResult {
        compliant: violations.is_empty(),
        required_opening_sq_in: required_opening,
        required_height_in: required_height,
        required_width_in: required_width,
        maximum_sill_height_in: max_sill,
        window_well_required: well_required,
        window_well_ladder_required: ladder_required,
        habitability_breach,
        violations,
        citation: "IRC § R310 (Emergency Escape and Rescue Openings); IRC § R310.2.1; § R310.2.2; § R310.2.3; California Residential Code § R310; 2020 New York State Residential Code § R310",
        notes,
    }
}

fn check_default(_input: &RentalBedroomEgressWindowInput) -> RentalBedroomEgressWindowResult {
    let notes: Vec<String> = vec![
        "default rule — no statewide IRC adoption; local building code or common-law warranty of habitability governs egress requirements"
            .to_string(),
        "default rule — verify local jurisdiction adoption of IRC R310 or equivalent (most U.S. jurisdictions adopt IRC; some legacy areas use older codes)"
            .to_string(),
    ];

    RentalBedroomEgressWindowResult {
        compliant: true,
        required_opening_sq_in: 0,
        required_height_in: 0,
        required_width_in: 0,
        maximum_sill_height_in: 0,
        window_well_required: false,
        window_well_ladder_required: false,
        habitability_breach: false,
        violations: Vec::new(),
        citation: "local building code + common-law warranty of habitability",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ircr310_compliant() -> RentalBedroomEgressWindowInput {
        RentalBedroomEgressWindowInput {
            regime: Regime::IrcR310,
            net_clear_opening_sq_in: 850,
            net_clear_height_in: 30,
            net_clear_width_in: 24,
            sill_height_in: 36,
            at_grade_floor: false,
            below_grade_level: false,
            window_well_area_sq_in: 0,
            window_well_projection_in: 0,
            window_well_depth_in: 0,
            window_well_ladder_provided: false,
        }
    }

    fn default_base() -> RentalBedroomEgressWindowInput {
        let mut i = ircr310_compliant();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn ircr310_compliant_passes() {
        let r = check(&ircr310_compliant());
        assert!(r.compliant);
        assert!(!r.habitability_breach);
        assert_eq!(r.required_opening_sq_in, 821);
        assert_eq!(r.required_height_in, 24);
        assert_eq!(r.required_width_in, 20);
        assert_eq!(r.maximum_sill_height_in, 44);
    }

    #[test]
    fn ircr310_opening_at_5_7_sqft_boundary_compliant() {
        let mut i = ircr310_compliant();
        i.net_clear_opening_sq_in = 821;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn ircr310_opening_below_5_7_sqft_violates() {
        let mut i = ircr310_compliant();
        i.net_clear_opening_sq_in = 820;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ R310.2.1") && v.contains("820 sq in")));
    }

    #[test]
    fn ircr310_grade_floor_5_0_sqft_compliant() {
        let mut i = ircr310_compliant();
        i.at_grade_floor = true;
        i.net_clear_opening_sq_in = 720;
        let r = check(&i);
        assert!(r.compliant);
        assert_eq!(r.required_opening_sq_in, 720);
    }

    #[test]
    fn ircr310_grade_floor_below_5_0_sqft_violates() {
        let mut i = ircr310_compliant();
        i.at_grade_floor = true;
        i.net_clear_opening_sq_in = 719;
        let r = check(&i);
        assert!(!r.compliant);
    }

    #[test]
    fn ircr310_height_at_24_in_boundary_compliant() {
        let mut i = ircr310_compliant();
        i.net_clear_height_in = 24;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn ircr310_height_below_24_in_violates() {
        let mut i = ircr310_compliant();
        i.net_clear_height_in = 23;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("23 in") && v.contains("height")));
    }

    #[test]
    fn ircr310_width_at_20_in_boundary_compliant() {
        let mut i = ircr310_compliant();
        i.net_clear_width_in = 20;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn ircr310_width_below_20_in_violates() {
        let mut i = ircr310_compliant();
        i.net_clear_width_in = 19;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("19 in") && v.contains("width")));
    }

    #[test]
    fn ircr310_sill_at_44_in_boundary_compliant() {
        let mut i = ircr310_compliant();
        i.sill_height_in = 44;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn ircr310_sill_above_44_in_violates() {
        let mut i = ircr310_compliant();
        i.sill_height_in = 45;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ R310.2.2") && v.contains("45 in")));
    }

    #[test]
    fn ircr310_below_grade_well_required() {
        let mut i = ircr310_compliant();
        i.below_grade_level = true;
        i.window_well_area_sq_in = 1300;
        i.window_well_projection_in = 36;
        i.window_well_depth_in = 30;
        let r = check(&i);
        assert!(r.compliant);
        assert!(r.window_well_required);
        assert!(!r.window_well_ladder_required);
    }

    #[test]
    fn ircr310_below_grade_well_below_9_sqft_violates() {
        let mut i = ircr310_compliant();
        i.below_grade_level = true;
        i.window_well_area_sq_in = 1200;
        i.window_well_projection_in = 36;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ R310.2.3") && v.contains("9 sq ft")));
    }

    #[test]
    fn ircr310_well_projection_below_36_in_violates() {
        let mut i = ircr310_compliant();
        i.below_grade_level = true;
        i.window_well_area_sq_in = 1300;
        i.window_well_projection_in = 35;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("projection") && v.contains("35 in")));
    }

    #[test]
    fn ircr310_well_depth_over_44_in_requires_ladder() {
        let mut i = ircr310_compliant();
        i.below_grade_level = true;
        i.window_well_area_sq_in = 1300;
        i.window_well_projection_in = 36;
        i.window_well_depth_in = 48;
        i.window_well_ladder_provided = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.window_well_ladder_required);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("48 in") && v.contains("ladder")));
    }

    #[test]
    fn ircr310_well_depth_over_44_in_with_ladder_compliant() {
        let mut i = ircr310_compliant();
        i.below_grade_level = true;
        i.window_well_area_sq_in = 1300;
        i.window_well_projection_in = 36;
        i.window_well_depth_in = 48;
        i.window_well_ladder_provided = true;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn ircr310_well_depth_at_44_in_boundary_no_ladder_required() {
        let mut i = ircr310_compliant();
        i.below_grade_level = true;
        i.window_well_area_sq_in = 1300;
        i.window_well_projection_in = 36;
        i.window_well_depth_in = 44;
        let r = check(&i);
        assert!(r.compliant);
        assert!(!r.window_well_ladder_required);
    }

    #[test]
    fn ircr310_all_four_violations_stack() {
        let mut i = ircr310_compliant();
        i.net_clear_opening_sq_in = 700;
        i.net_clear_height_in = 20;
        i.net_clear_width_in = 18;
        i.sill_height_in = 48;
        let r = check(&i);
        assert!(!r.compliant);
        assert_eq!(r.violations.len(), 4);
    }

    #[test]
    fn ircr310_citation_pins_authorities() {
        let r = check(&ircr310_compliant());
        assert!(r.citation.contains("§ R310"));
        assert!(r.citation.contains("§ R310.2.1"));
        assert!(r.citation.contains("§ R310.2.2"));
        assert!(r.citation.contains("§ R310.2.3"));
        assert!(r.citation.contains("California Residential Code"));
        assert!(r.citation.contains("2020 New York State Residential Code"));
    }

    #[test]
    fn ircr310_note_pins_four_dimensions() {
        let r = check(&ircr310_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ R310.2.1")
            && n.contains("5.7 sq ft")
            && n.contains("5.0 sq ft")
            && n.contains("24 in")
            && n.contains("20 in")
            && n.contains("44 in")));
    }

    #[test]
    fn ircr310_note_pins_window_well_requirements() {
        let r = check(&ircr310_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ R310.2.3")
            && n.contains("9 sq ft")
            && n.contains("36 in")
            && n.contains("ladder")));
    }

    #[test]
    fn ircr310_note_pins_state_adoption() {
        let r = check(&ircr310_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("California Residential Code")
                && n.contains("New York State Residential Code")
                && n.contains("ALL FOUR")));
    }

    #[test]
    fn default_no_violations() {
        let r = check(&default_base());
        assert!(r.compliant);
        assert!(!r.habitability_breach);
    }

    #[test]
    fn default_citation_pins_local_code() {
        let r = check(&default_base());
        assert!(r.citation.contains("local building code"));
        assert!(r.citation.contains("common-law warranty of habitability"));
    }

    #[test]
    fn two_regimes_routed_correctly() {
        for regime in [Regime::IrcR310, Regime::Default] {
            let mut i = ircr310_compliant();
            i.regime = regime;
            let r = check(&i);
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn ircr310_grade_floor_uniquely_lower_threshold_invariant() {
        let mut i_grade = ircr310_compliant();
        i_grade.at_grade_floor = true;
        let r_grade = check(&i_grade);
        assert_eq!(r_grade.required_opening_sq_in, 720);

        let mut i_no_grade = ircr310_compliant();
        i_no_grade.at_grade_floor = false;
        let r_no_grade = check(&i_no_grade);
        assert_eq!(r_no_grade.required_opening_sq_in, 821);
    }

    #[test]
    fn ircr310_well_required_only_below_grade_invariant() {
        let mut i_above = ircr310_compliant();
        i_above.below_grade_level = false;
        let r_above = check(&i_above);
        assert!(!r_above.window_well_required);

        let mut i_below = ircr310_compliant();
        i_below.below_grade_level = true;
        i_below.window_well_area_sq_in = 1300;
        i_below.window_well_projection_in = 36;
        let r_below = check(&i_below);
        assert!(r_below.window_well_required);
    }

    #[test]
    fn ircr310_ladder_required_only_when_well_depth_over_44_invariant() {
        let mut i_44 = ircr310_compliant();
        i_44.below_grade_level = true;
        i_44.window_well_area_sq_in = 1300;
        i_44.window_well_projection_in = 36;
        i_44.window_well_depth_in = 44;
        let r_44 = check(&i_44);
        assert!(!r_44.window_well_ladder_required);

        let mut i_45 = ircr310_compliant();
        i_45.below_grade_level = true;
        i_45.window_well_area_sq_in = 1300;
        i_45.window_well_projection_in = 36;
        i_45.window_well_depth_in = 45;
        i_45.window_well_ladder_provided = true;
        let r_45 = check(&i_45);
        assert!(r_45.window_well_ladder_required);
    }

    #[test]
    fn ircr310_habitability_breach_engages_with_any_violation() {
        let mut i = ircr310_compliant();
        i.net_clear_opening_sq_in = 100;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.habitability_breach);
    }
}
