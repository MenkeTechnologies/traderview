//! IRC § 165(h) — Treatment of casualty gains and losses for individuals.
//!
//! § 165(h) governs the **personal casualty loss** itemized deduction.
//! Three time-windowed regimes:
//!
//! 1. **Pre-TCJA (through 2017)**: any sudden, unexpected, identifiable
//!    event causing damage qualified — auto accidents, broken pipes,
//!    house fires, theft. Subject to two floors: $100 per event +
//!    10% AGI on the aggregate.
//! 2. **TCJA window (2018-2025)**: § 165(h)(5), added by TCJA § 11044,
//!    SUSPENDED deduction for personal casualty losses EXCEPT to the
//!    extent attributable to a **federally declared disaster** (FEMA
//!    declaration). State-declared disasters do not qualify.
//! 3. **OBBBA permanent expansion (2026+)**: OBBBA § 70423 makes the
//!    TCJA limitation **permanent** AND **expands** qualifying events
//!    to include **state-declared disasters** (natural catastrophes like
//!    hurricane / tornado / storm / earthquake or any fire / flood /
//!    explosion the state deems severe).
//!
//! **Calculation order** (§ 165(h)(1)/(2)):
//! 1. Raw loss = lesser of (adjusted basis, decline in FMV) − insurance
//!    or other reimbursement.
//! 2. **$100 per-event floor** (§ 165(h)(1)) — first $100 of each event
//!    is non-deductible. (Special $500 floor for "qualified disaster
//!    losses" under prior disaster-tax-relief acts.)
//! 3. **10% AGI floor** (§ 165(h)(2)) — aggregate net casualty losses
//!    are deductible only to the extent they exceed 10% of AGI.
//!    (Qualified disaster losses are EXEMPT from the AGI floor.)
//!
//! Casualty GAINS (insurance recovery exceeds basis) net against losses
//! first; that nuance is out of scope of this module's calculation.
//!
//! Citations: 26 U.S.C. § 165; § 165(h)(1) ($100 per-event floor);
//! § 165(h)(2) (10% AGI floor); § 165(h)(5) (TCJA suspension for personal
//! casualty losses except federally declared disasters); OBBBA § 70423
//! (permanent extension + state-declared disaster expansion, eff. tax
//! years beginning after 2025-12-31); IRS Pub. 547 (calculation guidance).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisasterType {
    /// Federally declared disaster (FEMA declaration). Always qualifies
    /// post-TCJA.
    FederallyDeclared,
    /// State-declared disaster — qualifies ONLY for tax years beginning
    /// after 2025-12-31 under OBBBA § 70423.
    StateDeclared,
    /// Non-disaster casualty (auto accident, household fire, theft) —
    /// suspended 2018+ under § 165(h)(5).
    NonDisaster,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section165HInput {
    pub year: u32,
    pub disaster_type: DisasterType,
    pub adjusted_basis_cents: i64,
    pub decline_in_fmv_cents: i64,
    pub insurance_reimbursement_cents: i64,
    pub agi_cents: i64,
    /// Whether Congress has designated this event a "qualified disaster
    /// loss" by name (e.g., Hurricane Katrina, Hurricane Ian, etc.).
    /// Qualified disaster losses get $500 per-event floor (instead of
    /// $100) AND are EXEMPT from the 10% AGI floor.
    pub qualified_disaster_loss: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section165HResult {
    pub loss_eligible: bool,
    pub raw_loss_cents: i64,
    pub per_event_floor_cents: i64,
    pub after_per_event_floor_cents: i64,
    pub agi_floor_cents: i64,
    pub agi_floor_applies: bool,
    pub allowed_deduction_cents: i64,
    pub citation: &'static str,
    pub note: String,
}

pub fn compute(input: &Section165HInput) -> Section165HResult {
    let basis = input.adjusted_basis_cents.max(0);
    let fmv_decline = input.decline_in_fmv_cents.max(0);
    let insurance = input.insurance_reimbursement_cents.max(0);
    let agi = input.agi_cents.max(0);

    let raw_loss_pre_insurance = basis.min(fmv_decline);
    let raw_loss = (raw_loss_pre_insurance - insurance).max(0);

    let eligible = eligibility_for_year_and_type(input.year, input.disaster_type);
    if !eligible {
        return Section165HResult {
            loss_eligible: false,
            raw_loss_cents: raw_loss,
            per_event_floor_cents: 0,
            after_per_event_floor_cents: 0,
            agi_floor_cents: 0,
            agi_floor_applies: false,
            allowed_deduction_cents: 0,
            citation: eligibility_citation(input.year, input.disaster_type),
            note: format!(
                "Personal casualty loss of {} cents is NOT deductible for tax year {} with disaster type {:?}. {}",
                raw_loss,
                input.year,
                input.disaster_type,
                eligibility_explanation(input.year, input.disaster_type),
            ),
        };
    }

    // Per-event floor: $500 if qualified-disaster, $100 otherwise.
    let per_event_floor = if input.qualified_disaster_loss { 50000 } else { 10000 };
    let after_per_event = (raw_loss - per_event_floor).max(0);

    // 10% AGI floor: exempt for qualified-disaster losses.
    let agi_floor_applies = !input.qualified_disaster_loss;
    let agi_floor = if agi_floor_applies {
        (agi as i128 * 10 / 100) as i64
    } else {
        0
    };
    let allowed = if agi_floor_applies {
        (after_per_event - agi_floor).max(0)
    } else {
        after_per_event
    };

    let note = format!(
        "Raw loss = min(basis {}, FMV decline {}) − insurance {} = {} cents. Per-event floor = {} cents (qualified-disaster = {}); after = {} cents. 10% AGI floor = {} cents ({}); allowed deduction = {} cents.",
        basis,
        fmv_decline,
        insurance,
        raw_loss,
        per_event_floor,
        input.qualified_disaster_loss,
        after_per_event,
        agi_floor,
        if agi_floor_applies { "APPLIES" } else { "EXEMPT (qualified disaster)" },
        allowed,
    );

    Section165HResult {
        loss_eligible: true,
        raw_loss_cents: raw_loss,
        per_event_floor_cents: per_event_floor,
        after_per_event_floor_cents: after_per_event,
        agi_floor_cents: agi_floor,
        agi_floor_applies,
        allowed_deduction_cents: allowed,
        citation: eligibility_citation(input.year, input.disaster_type),
        note,
    }
}

fn eligibility_for_year_and_type(year: u32, dt: DisasterType) -> bool {
    match (year, dt) {
        // Pre-TCJA: everything qualifies subject to the $100/10% AGI floors.
        (y, _) if y <= 2017 => true,
        // TCJA window 2018-2025: only federally declared.
        (y, DisasterType::FederallyDeclared) if (2018..=2025).contains(&y) => true,
        (y, _) if (2018..=2025).contains(&y) => false,
        // OBBBA permanent expansion 2026+: federally OR state-declared.
        (y, DisasterType::FederallyDeclared | DisasterType::StateDeclared) if y >= 2026 => true,
        (y, DisasterType::NonDisaster) if y >= 2026 => false,
        _ => false,
    }
}

fn eligibility_citation(year: u32, dt: DisasterType) -> &'static str {
    match (year, dt) {
        (y, _) if y <= 2017 => {
            "26 U.S.C. § 165(h) (pre-TCJA) — personal casualty losses deductible subject to $100 per-event + 10% AGI floors"
        }
        (y, DisasterType::FederallyDeclared) if (2018..=2025).contains(&y) => {
            "26 U.S.C. § 165(h)(5) (TCJA § 11044) — personal casualty loss deductible only for federally declared disaster (2018-2025)"
        }
        (y, _) if (2018..=2025).contains(&y) => {
            "26 U.S.C. § 165(h)(5) (TCJA § 11044) — personal casualty losses suspended 2018-2025 except for federally declared disaster"
        }
        (y, DisasterType::FederallyDeclared | DisasterType::StateDeclared) if y >= 2026 => {
            "26 U.S.C. § 165(h) as amended by OBBBA § 70423 (eff. tax years after 2025-12-31) — expanded to federally OR state-declared disasters"
        }
        (y, DisasterType::NonDisaster) if y >= 2026 => {
            "26 U.S.C. § 165(h)(5) as amended by OBBBA § 70423 — TCJA suspension made PERMANENT; non-disaster personal casualty losses remain non-deductible"
        }
        _ => "26 U.S.C. § 165(h) — personal casualty loss",
    }
}

fn eligibility_explanation(year: u32, dt: DisasterType) -> &'static str {
    match (year, dt) {
        (y, _) if (2018..=2025).contains(&y) && !matches!(dt, DisasterType::FederallyDeclared) => {
            "TCJA § 165(h)(5) suspends personal casualty losses 2018-2025 except for federally declared disasters."
        }
        (y, DisasterType::NonDisaster) if y >= 2026 => {
            "OBBBA § 70423 makes TCJA suspension PERMANENT — non-disaster personal casualty losses remain non-deductible."
        }
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        year: u32,
        dt: DisasterType,
        basis: i64,
        fmv_decline: i64,
        insurance: i64,
        agi: i64,
        qualified: bool,
    ) -> Section165HInput {
        Section165HInput {
            year,
            disaster_type: dt,
            adjusted_basis_cents: basis,
            decline_in_fmv_cents: fmv_decline,
            insurance_reimbursement_cents: insurance,
            agi_cents: agi,
            qualified_disaster_loss: qualified,
        }
    }

    #[test]
    fn pre_tcja_non_disaster_qualifies() {
        // 2017 auto accident, $20K loss, $0 insurance, $100K AGI.
        let r = compute(&input(2017, DisasterType::NonDisaster, 50_000_00, 20_000_00, 0, 100_000_00, false));
        assert!(r.loss_eligible);
        // Raw $20K − $100 floor = $19,900. − 10% AGI ($10K) = $9,900.
        assert_eq!(r.allowed_deduction_cents, 9_900_00);
        assert!(r.citation.contains("pre-TCJA"));
    }

    #[test]
    fn tcja_2024_federally_declared_qualifies() {
        let r = compute(&input(2024, DisasterType::FederallyDeclared, 50_000_00, 30_000_00, 5_000_00, 100_000_00, false));
        assert!(r.loss_eligible);
        // Raw = min($50K, $30K) - $5K = $25K. - $100 floor = $24,900. - 10% AGI $10K = $14,900.
        assert_eq!(r.raw_loss_cents, 25_000_00);
        assert_eq!(r.allowed_deduction_cents, 14_900_00);
        assert!(r.citation.contains("§ 165(h)(5)"));
    }

    #[test]
    fn tcja_2024_state_declared_NOT_eligible() {
        let r = compute(&input(2024, DisasterType::StateDeclared, 50_000_00, 30_000_00, 0, 100_000_00, false));
        assert!(!r.loss_eligible);
        assert_eq!(r.allowed_deduction_cents, 0);
        assert!(r.note.contains("suspends personal casualty losses 2018-2025"));
    }

    #[test]
    fn tcja_2024_non_disaster_NOT_eligible() {
        let r = compute(&input(2024, DisasterType::NonDisaster, 50_000_00, 30_000_00, 0, 100_000_00, false));
        assert!(!r.loss_eligible);
        assert_eq!(r.allowed_deduction_cents, 0);
    }

    #[test]
    fn obbba_2026_state_declared_now_qualifies() {
        let r = compute(&input(2026, DisasterType::StateDeclared, 50_000_00, 30_000_00, 5_000_00, 100_000_00, false));
        assert!(r.loss_eligible);
        assert_eq!(r.allowed_deduction_cents, 14_900_00);
        assert!(r.citation.contains("OBBBA § 70423"));
        assert!(r.citation.contains("state-declared"));
    }

    #[test]
    fn obbba_2026_federally_declared_still_qualifies() {
        let r = compute(&input(2026, DisasterType::FederallyDeclared, 50_000_00, 30_000_00, 5_000_00, 100_000_00, false));
        assert!(r.loss_eligible);
        assert_eq!(r.allowed_deduction_cents, 14_900_00);
    }

    #[test]
    fn obbba_2026_non_disaster_PERMANENTLY_suspended() {
        let r = compute(&input(2026, DisasterType::NonDisaster, 50_000_00, 30_000_00, 0, 100_000_00, false));
        assert!(!r.loss_eligible);
        assert!(r.citation.contains("PERMANENT"));
        assert!(r.note.contains("PERMANENT"));
    }

    #[test]
    fn raw_loss_uses_lesser_of_basis_or_fmv_decline() {
        // Basis $50K, FMV decline $30K → use $30K.
        let r1 = compute(&input(2024, DisasterType::FederallyDeclared, 50_000_00, 30_000_00, 0, 100_000_00, false));
        assert_eq!(r1.raw_loss_cents, 30_000_00);
        // Basis $30K, FMV decline $50K → use $30K (basis caps).
        let r2 = compute(&input(2024, DisasterType::FederallyDeclared, 30_000_00, 50_000_00, 0, 100_000_00, false));
        assert_eq!(r2.raw_loss_cents, 30_000_00);
    }

    #[test]
    fn insurance_reimbursement_reduces_raw_loss() {
        let r = compute(&input(2024, DisasterType::FederallyDeclared, 50_000_00, 50_000_00, 40_000_00, 100_000_00, false));
        // Raw = $50K - $40K = $10K. - $100 = $9,900. - 10% AGI $10K = $0.
        assert_eq!(r.raw_loss_cents, 10_000_00);
        assert_eq!(r.allowed_deduction_cents, 0);
    }

    #[test]
    fn insurance_exceeding_loss_zero_deduction() {
        let r = compute(&input(2024, DisasterType::FederallyDeclared, 50_000_00, 30_000_00, 100_000_00, 100_000_00, false));
        assert_eq!(r.raw_loss_cents, 0);
        assert_eq!(r.allowed_deduction_cents, 0);
    }

    #[test]
    fn qualified_disaster_500_floor_not_100() {
        let r = compute(&input(2024, DisasterType::FederallyDeclared, 50_000_00, 30_000_00, 0, 100_000_00, true));
        assert_eq!(r.per_event_floor_cents, 50000);
        // Raw $30K - $500 = $29,500. NO 10% AGI floor (qualified-disaster exempt).
        assert_eq!(r.allowed_deduction_cents, 29_500_00);
    }

    #[test]
    fn qualified_disaster_exempt_from_10_percent_agi() {
        let r = compute(&input(2024, DisasterType::FederallyDeclared, 50_000_00, 30_000_00, 0, 100_000_00, true));
        assert!(!r.agi_floor_applies);
        assert_eq!(r.agi_floor_cents, 0);
    }

    #[test]
    fn non_qualified_disaster_uses_100_floor_and_agi() {
        let r = compute(&input(2024, DisasterType::FederallyDeclared, 50_000_00, 30_000_00, 0, 100_000_00, false));
        assert_eq!(r.per_event_floor_cents, 10000);
        assert!(r.agi_floor_applies);
        assert_eq!(r.agi_floor_cents, 10_000_00);
    }

    #[test]
    fn agi_floor_can_zero_allowed_deduction() {
        // Raw $15K - $100 = $14,900. 10% AGI on $200K = $20K → exceeds, allowed = 0.
        let r = compute(&input(2024, DisasterType::FederallyDeclared, 20_000_00, 15_000_00, 0, 200_000_00, false));
        assert_eq!(r.allowed_deduction_cents, 0);
    }

    #[test]
    fn small_loss_per_event_floor_zeros_loss() {
        // Raw $50 < $100 floor → 0.
        let r = compute(&input(2024, DisasterType::FederallyDeclared, 100_00, 50_00, 0, 50_000_00, false));
        assert_eq!(r.after_per_event_floor_cents, 0);
        assert_eq!(r.allowed_deduction_cents, 0);
    }

    #[test]
    fn tcja_window_boundary_2018_and_2025() {
        // 2018 federally declared still works.
        let r_2018 = compute(&input(2018, DisasterType::FederallyDeclared, 50_000_00, 30_000_00, 0, 100_000_00, false));
        assert!(r_2018.loss_eligible);
        // 2025 federally declared still works.
        let r_2025 = compute(&input(2025, DisasterType::FederallyDeclared, 50_000_00, 30_000_00, 0, 100_000_00, false));
        assert!(r_2025.loss_eligible);
        // 2025 state-declared NOT eligible (OBBBA expansion not effective yet).
        let r_2025_state = compute(&input(2025, DisasterType::StateDeclared, 50_000_00, 30_000_00, 0, 100_000_00, false));
        assert!(!r_2025_state.loss_eligible);
    }

    #[test]
    fn obbba_threshold_2026_state_declared() {
        let r_2025 = compute(&input(2025, DisasterType::StateDeclared, 50_000_00, 30_000_00, 0, 100_000_00, false));
        let r_2026 = compute(&input(2026, DisasterType::StateDeclared, 50_000_00, 30_000_00, 0, 100_000_00, false));
        assert!(!r_2025.loss_eligible);
        assert!(r_2026.loss_eligible);
    }

    #[test]
    fn pre_tcja_2017_state_declared_qualifies() {
        // Pre-TCJA: state-declared (or any casualty) qualified.
        let r = compute(&input(2017, DisasterType::StateDeclared, 50_000_00, 30_000_00, 0, 100_000_00, false));
        assert!(r.loss_eligible);
    }

    #[test]
    fn citation_pins_obbba_70423_for_2026_plus() {
        let r = compute(&input(2026, DisasterType::FederallyDeclared, 50_000_00, 30_000_00, 0, 100_000_00, false));
        assert!(r.citation.contains("OBBBA § 70423"));
        assert!(r.citation.contains("2025-12-31"));
    }

    #[test]
    fn citation_pins_tcja_11044_for_tcja_window() {
        let r = compute(&input(2022, DisasterType::FederallyDeclared, 50_000_00, 30_000_00, 0, 100_000_00, false));
        assert!(r.citation.contains("TCJA § 11044"));
    }

    #[test]
    fn negative_inputs_clamped() {
        let r = compute(&input(2026, DisasterType::FederallyDeclared, -100, -100, -100, -100, false));
        assert_eq!(r.raw_loss_cents, 0);
        assert_eq!(r.allowed_deduction_cents, 0);
    }

    #[test]
    fn worked_example_hurricane_50k_loss_100k_agi() {
        // FEMA-declared hurricane. $80K basis home, $50K FMV decline,
        // $10K insurance, $100K AGI. Not qualified-disaster (no Congress
        // act).
        // Raw = min($80K, $50K) - $10K = $40K.
        // Per-event $100 floor → $39,900.
        // 10% AGI on $100K = $10K floor → $39,900 - $10K = $29,900.
        let r = compute(&input(2024, DisasterType::FederallyDeclared, 80_000_00, 50_000_00, 10_000_00, 100_000_00, false));
        assert_eq!(r.raw_loss_cents, 40_000_00);
        assert_eq!(r.allowed_deduction_cents, 29_900_00);
    }

    #[test]
    fn worked_example_qualified_disaster_no_agi_floor() {
        // Same hurricane but Congress designated as qualified-disaster
        // (e.g., Hurricane Katrina-type act).
        // Raw = $40K.
        // Per-event $500 floor → $39,500.
        // NO 10% AGI floor → allowed = $39,500.
        let r = compute(&input(2024, DisasterType::FederallyDeclared, 80_000_00, 50_000_00, 10_000_00, 100_000_00, true));
        assert_eq!(r.allowed_deduction_cents, 39_500_00);
        assert!(!r.agi_floor_applies);
    }

    #[test]
    fn eligibility_invariant_2025_state_vs_2026_state() {
        let r25 = compute(&input(2025, DisasterType::StateDeclared, 50_000_00, 30_000_00, 0, 100_000_00, false));
        let r26 = compute(&input(2026, DisasterType::StateDeclared, 50_000_00, 30_000_00, 0, 100_000_00, false));
        assert!(!r25.loss_eligible);
        assert!(r26.loss_eligible);
        assert!(r25.allowed_deduction_cents == 0);
        assert!(r26.allowed_deduction_cents > 0);
    }

    #[test]
    fn obbba_2030_still_permanent_suspension_for_non_disaster() {
        let r = compute(&input(2030, DisasterType::NonDisaster, 50_000_00, 30_000_00, 0, 100_000_00, false));
        assert!(!r.loss_eligible);
        assert!(r.citation.contains("PERMANENT"));
    }
}
