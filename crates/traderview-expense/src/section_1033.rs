//! IRC § 1033 — Involuntary conversions.
//!
//! 26 U.S.C. § 1033 lets a taxpayer DEFER gain on property that was
//! compulsorily or involuntarily converted (destroyed, stolen, condemned,
//! seized, requisitioned, lost to federally declared disaster) when the
//! proceeds are reinvested in qualifying replacement property within the
//! statutory replacement period.
//!
//! **Gain recognized formula (§ 1033(a)(2)(A))**:
//! `recognized = max(0, amount_realized − replacement_cost)`,
//! capped by realized gain. The unrecognized portion is deferred.
//!
//! **Realized gain**:
//! `realized = amount_realized − adjusted_basis`. If realized is zero or
//! negative, § 1033 is irrelevant (loss is taken under ordinary rules).
//!
//! **Basis of replacement property (§ 1033(b)(2))**:
//! `basis = replacement_cost − deferred_gain`. The deferred gain is
//! preserved in the lower basis of the replacement, so it remains
//! taxable on later disposition.
//!
//! **Replacement-period clocks**:
//! - **2 years** — general rule (§ 1033(a)(2)(B)(i))
//! - **3 years** — real property held for productive use in trade or
//!   business or for investment (§ 1033(g)(4) condemnation)
//! - **4 years** — principal residence destroyed in a federally declared
//!   disaster (§ 1033(h)(1)(B))
//! - **5 years** — qualifying disaster property (§ 1033(h)(2)(A); enacted
//!   for Hurricane Katrina-class events, extended on a per-disaster basis)
//!
//! Replacement property must be **similar or related in service or use**
//! to the converted property (§ 1033(a)(2)). Treas. Reg. § 1.1033(a)-2
//! applies a functional-use test for owner-users and an end-use test for
//! lessors of investment real estate.
//!
//! Citations: 26 U.S.C. § 1033; § 1033(a)(2)(A) (gain recognition);
//! § 1033(a)(2)(B)(i) (2-year general window); § 1033(b)(2) (replacement-
//! property basis); § 1033(g)(4) (3-year condemnation window for trade-
//! or-investment real property); § 1033(h)(1)(B) (4-year disaster-area
//! principal-residence window); § 1033(h)(2)(A) (5-year qualifying disaster);
//! Treas. Reg. § 1.1033(a)-2 (similar-or-related-in-service-or-use test).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConversionType {
    DestructionFireTheftCasualty,
    Condemnation,
    FederallyDeclaredDisaster,
    QualifyingDisaster,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyUse {
    PersonalResidence,
    TradeOrBusinessRealProperty,
    InvestmentRealProperty,
    PersonalUseTangible,
    TradeOrBusinessTangible,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section1033Input {
    pub amount_realized_cents: i64,
    pub adjusted_basis_cents: i64,
    pub replacement_cost_cents: i64,
    pub conversion_type: ConversionType,
    pub property_use: PropertyUse,
    /// Months from the end of the first taxable year in which any part of
    /// the gain was realized to the close of the replacement period that
    /// the caller is testing. The compute fn compares to the regime's
    /// statutory window.
    pub months_to_replacement: u32,
    /// Whether the replacement is "similar or related in service or use"
    /// to the converted property (Treas. Reg. § 1.1033(a)-2). The compute
    /// fn flags non-qualifying replacements regardless of the dollar math.
    pub similar_or_related_in_service_or_use: bool,
    /// Whether the taxpayer ELECTED non-recognition under § 1033(a)(2).
    /// Mandatory non-recognition under § 1033(a)(1) only applies when
    /// proceeds are converted directly into property (rare). Default-false.
    pub election_made: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section1033Result {
    pub realized_gain_cents: i64,
    pub recognized_gain_cents: i64,
    pub deferred_gain_cents: i64,
    pub basis_in_replacement_cents: i64,
    pub replacement_window_months: u32,
    pub replacement_period_satisfied: bool,
    pub similar_use_test_satisfied: bool,
    pub election_required: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn compute(input: &Section1033Input) -> Section1033Result {
    let amount_realized = input.amount_realized_cents.max(0);
    let adjusted_basis = input.adjusted_basis_cents.max(0);
    let replacement_cost = input.replacement_cost_cents.max(0);

    let realized_gain = (amount_realized - adjusted_basis).max(0);
    let replacement_window = replacement_window_months(input.conversion_type, input.property_use);
    let period_satisfied = input.months_to_replacement <= replacement_window;
    let similar_test = input.similar_or_related_in_service_or_use;
    let election_required = !matches!(input.conversion_type, ConversionType::QualifyingDisaster);

    // If period or similar-use test fails, § 1033 deferral collapses —
    // entire realized gain is recognized.
    let qualifies = period_satisfied && similar_test && (input.election_made || !election_required);
    let recognized_gain = if !qualifies {
        realized_gain
    } else {
        // Standard formula: gain recognized to extent amount realized
        // exceeds replacement cost.
        (amount_realized - replacement_cost).max(0).min(realized_gain)
    };
    let deferred_gain = realized_gain - recognized_gain;
    // § 1033(b)(2): basis in replacement = cost − deferred gain.
    let basis_in_replacement = (replacement_cost - deferred_gain).max(0);

    let citation = match input.property_use {
        PropertyUse::PersonalResidence
            if input.conversion_type == ConversionType::FederallyDeclaredDisaster =>
        {
            "§ 1033(h)(1)(B) — 4-year window for principal residence destroyed in federally declared disaster"
        }
        _ if input.conversion_type == ConversionType::QualifyingDisaster => {
            "§ 1033(h)(2)(A) — 5-year window for qualifying disaster property"
        }
        PropertyUse::TradeOrBusinessRealProperty | PropertyUse::InvestmentRealProperty
            if input.conversion_type == ConversionType::Condemnation =>
        {
            "§ 1033(g)(4) — 3-year condemnation window for real property held for productive use or investment"
        }
        _ => "26 U.S.C. § 1033 — 2-year general replacement window (§ 1033(a)(2)(B)(i))",
    };

    let mut note = format!(
        "Realized gain = amount realized ({}) − adjusted basis ({}) = {} cents. Recognized gain = max(0, amount realized − replacement cost ({})) = {} cents (deferred = {} cents). Basis in replacement = replacement cost − deferred = {} cents. Replacement window = {} months ({}).",
        amount_realized,
        adjusted_basis,
        realized_gain,
        replacement_cost,
        recognized_gain,
        deferred_gain,
        basis_in_replacement,
        replacement_window,
        if period_satisfied { "SATISFIED" } else { "MISSED" }
    );
    if !similar_test {
        note.push_str(" Similar-or-related-in-service-or-use test FAILED — Treas. Reg. § 1.1033(a)-2 requires functional-use match for owner-users / end-use match for lessors.");
    }
    if election_required && !input.election_made {
        note.push_str(" § 1033(a)(2) ELECTION not made — non-recognition is unavailable unless proceeds were converted directly into property (§ 1033(a)(1)).");
    }

    Section1033Result {
        realized_gain_cents: realized_gain,
        recognized_gain_cents: recognized_gain,
        deferred_gain_cents: deferred_gain,
        basis_in_replacement_cents: basis_in_replacement,
        replacement_window_months: replacement_window,
        replacement_period_satisfied: period_satisfied,
        similar_use_test_satisfied: similar_test,
        election_required,
        citation,
        note,
    }
}

fn replacement_window_months(conv: ConversionType, use_: PropertyUse) -> u32 {
    match (conv, use_) {
        (ConversionType::QualifyingDisaster, _) => 60,
        (ConversionType::FederallyDeclaredDisaster, PropertyUse::PersonalResidence) => 48,
        (
            ConversionType::Condemnation,
            PropertyUse::TradeOrBusinessRealProperty | PropertyUse::InvestmentRealProperty,
        ) => 36,
        _ => 24,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_input() -> Section1033Input {
        Section1033Input {
            amount_realized_cents: 500_000_00,
            adjusted_basis_cents: 200_000_00,
            replacement_cost_cents: 500_000_00,
            conversion_type: ConversionType::DestructionFireTheftCasualty,
            property_use: PropertyUse::TradeOrBusinessTangible,
            months_to_replacement: 12,
            similar_or_related_in_service_or_use: true,
            election_made: true,
        }
    }

    #[test]
    fn full_reinvestment_defers_all_gain() {
        let r = compute(&base_input());
        assert_eq!(r.realized_gain_cents, 300_000_00);
        assert_eq!(r.recognized_gain_cents, 0);
        assert_eq!(r.deferred_gain_cents, 300_000_00);
        // Basis = replacement_cost (500K) − deferred (300K) = 200K (carryover).
        assert_eq!(r.basis_in_replacement_cents, 200_000_00);
        assert!(r.replacement_period_satisfied);
    }

    #[test]
    fn partial_reinvestment_recognizes_shortfall() {
        let mut i = base_input();
        i.amount_realized_cents = 500_000_00;
        i.adjusted_basis_cents = 200_000_00;
        i.replacement_cost_cents = 400_000_00;
        let r = compute(&i);
        // Realized 300K. Amount realized 500K minus reinvested 400K = 100K
        // recognized. Deferred = 200K. Basis = 400K − 200K = 200K.
        assert_eq!(r.realized_gain_cents, 300_000_00);
        assert_eq!(r.recognized_gain_cents, 100_000_00);
        assert_eq!(r.deferred_gain_cents, 200_000_00);
        assert_eq!(r.basis_in_replacement_cents, 200_000_00);
    }

    #[test]
    fn no_reinvestment_full_recognition() {
        let mut i = base_input();
        i.replacement_cost_cents = 0;
        let r = compute(&i);
        // Without reinvestment qualifying replacement is zero — entire
        // realized gain recognized. Basis in (nonexistent) replacement = 0.
        assert_eq!(r.recognized_gain_cents, 300_000_00);
        assert_eq!(r.basis_in_replacement_cents, 0);
    }

    #[test]
    fn replacement_cost_exceeds_realized_caps_at_realized() {
        let mut i = base_input();
        i.replacement_cost_cents = 1_000_000_00;
        let r = compute(&i);
        assert_eq!(r.recognized_gain_cents, 0);
        assert_eq!(r.deferred_gain_cents, 300_000_00);
        // Basis = 1M − 300K = 700K (lifted by extra reinvestment).
        assert_eq!(r.basis_in_replacement_cents, 700_000_00);
    }

    #[test]
    fn loss_realized_no_section_1033_recognition() {
        let mut i = base_input();
        i.amount_realized_cents = 100_000_00;
        i.adjusted_basis_cents = 200_000_00;
        let r = compute(&i);
        // Realized gain clamped to zero.
        assert_eq!(r.realized_gain_cents, 0);
        assert_eq!(r.recognized_gain_cents, 0);
        assert_eq!(r.deferred_gain_cents, 0);
    }

    #[test]
    fn general_2_year_window() {
        let r = compute(&base_input());
        assert_eq!(r.replacement_window_months, 24);
        assert!(r.citation.contains("2-year general"));
    }

    #[test]
    fn condemnation_real_property_3_year_window() {
        let mut i = base_input();
        i.conversion_type = ConversionType::Condemnation;
        i.property_use = PropertyUse::TradeOrBusinessRealProperty;
        let r = compute(&i);
        assert_eq!(r.replacement_window_months, 36);
        assert!(r.citation.contains("§ 1033(g)(4)"));
        assert!(r.citation.contains("3-year"));
    }

    #[test]
    fn condemnation_investment_real_property_3_year_window() {
        let mut i = base_input();
        i.conversion_type = ConversionType::Condemnation;
        i.property_use = PropertyUse::InvestmentRealProperty;
        let r = compute(&i);
        assert_eq!(r.replacement_window_months, 36);
    }

    #[test]
    fn condemnation_personal_residence_stays_at_2_year() {
        let mut i = base_input();
        i.conversion_type = ConversionType::Condemnation;
        i.property_use = PropertyUse::PersonalResidence;
        let r = compute(&i);
        assert_eq!(
            r.replacement_window_months, 24,
            "§ 1033(g)(4) is real-property-trade-or-investment only — personal residence keeps the 2-year general window"
        );
    }

    #[test]
    fn federally_declared_disaster_principal_residence_4_year_window() {
        let mut i = base_input();
        i.conversion_type = ConversionType::FederallyDeclaredDisaster;
        i.property_use = PropertyUse::PersonalResidence;
        let r = compute(&i);
        assert_eq!(r.replacement_window_months, 48);
        assert!(r.citation.contains("§ 1033(h)(1)(B)"));
        assert!(r.citation.contains("4-year"));
    }

    #[test]
    fn qualifying_disaster_5_year_window() {
        let mut i = base_input();
        i.conversion_type = ConversionType::QualifyingDisaster;
        let r = compute(&i);
        assert_eq!(r.replacement_window_months, 60);
        assert!(r.citation.contains("§ 1033(h)(2)(A)"));
        assert!(r.citation.contains("5-year"));
    }

    #[test]
    fn replacement_at_window_boundary_satisfied() {
        let mut i = base_input();
        i.months_to_replacement = 24;
        let r = compute(&i);
        assert!(r.replacement_period_satisfied);
    }

    #[test]
    fn replacement_one_month_after_window_missed() {
        let mut i = base_input();
        i.months_to_replacement = 25;
        let r = compute(&i);
        assert!(!r.replacement_period_satisfied);
        // Window missed collapses deferral — full gain recognized.
        assert_eq!(r.recognized_gain_cents, 300_000_00);
        assert_eq!(r.deferred_gain_cents, 0);
        assert!(r.note.contains("MISSED"));
    }

    #[test]
    fn similar_use_test_failure_collapses_deferral() {
        let mut i = base_input();
        i.similar_or_related_in_service_or_use = false;
        let r = compute(&i);
        assert_eq!(r.recognized_gain_cents, 300_000_00);
        assert_eq!(r.deferred_gain_cents, 0);
        assert!(!r.similar_use_test_satisfied);
        assert!(r.note.contains("Similar-or-related-in-service-or-use"));
        assert!(r.note.contains("Treas. Reg. § 1.1033(a)-2"));
    }

    #[test]
    fn missing_election_collapses_deferral() {
        let mut i = base_input();
        i.election_made = false;
        let r = compute(&i);
        assert_eq!(r.recognized_gain_cents, 300_000_00);
        assert!(r.note.contains("ELECTION not made"));
    }

    #[test]
    fn qualifying_disaster_does_not_require_election() {
        let mut i = base_input();
        i.conversion_type = ConversionType::QualifyingDisaster;
        i.election_made = false;
        let r = compute(&i);
        // QualifyingDisaster gets mandatory non-recognition.
        assert!(!r.election_required);
        assert_eq!(r.recognized_gain_cents, 0);
    }

    #[test]
    fn basis_carryover_preserves_deferred_gain() {
        // Adjusted basis 100K, amount realized 500K, replacement cost 500K.
        // Realized 400K, all deferred. Basis = 500K − 400K = 100K (carries the
        // original basis forward — the textbook § 1033 outcome).
        let mut i = base_input();
        i.amount_realized_cents = 500_000_00;
        i.adjusted_basis_cents = 100_000_00;
        i.replacement_cost_cents = 500_000_00;
        let r = compute(&i);
        assert_eq!(r.realized_gain_cents, 400_000_00);
        assert_eq!(r.recognized_gain_cents, 0);
        assert_eq!(r.deferred_gain_cents, 400_000_00);
        assert_eq!(r.basis_in_replacement_cents, 100_000_00);
    }

    #[test]
    fn partial_reinvestment_deferred_gain_formula_correctness() {
        // The "lesser of (realized gain, amount realized − replacement cost)"
        // formula: when reinvestment is between basis and amount realized, the
        // shortfall ABOVE replacement is recognized.
        let mut i = base_input();
        i.amount_realized_cents = 1_000_000_00;
        i.adjusted_basis_cents = 300_000_00;
        i.replacement_cost_cents = 600_000_00;
        let r = compute(&i);
        assert_eq!(r.realized_gain_cents, 700_000_00);
        // Shortfall = 1M − 600K = 400K. Capped by realized 700K → 400K recognized.
        assert_eq!(r.recognized_gain_cents, 400_000_00);
        assert_eq!(r.deferred_gain_cents, 300_000_00);
        // Basis = 600K − 300K = 300K.
        assert_eq!(r.basis_in_replacement_cents, 300_000_00);
    }

    #[test]
    fn negative_basis_clamped_to_zero() {
        let mut i = base_input();
        i.adjusted_basis_cents = -500_00;
        let r = compute(&i);
        assert_eq!(r.realized_gain_cents, 500_000_00); // 500K - 0
    }

    #[test]
    fn citation_priority_disaster_beats_condemnation_beats_general() {
        // Federal disaster principal residence — strongest citation.
        let mut i = base_input();
        i.conversion_type = ConversionType::FederallyDeclaredDisaster;
        i.property_use = PropertyUse::PersonalResidence;
        let r1 = compute(&i);
        assert!(r1.citation.contains("§ 1033(h)(1)(B)"));

        // Condemnation real property — § 1033(g)(4).
        i.conversion_type = ConversionType::Condemnation;
        i.property_use = PropertyUse::TradeOrBusinessRealProperty;
        let r2 = compute(&i);
        assert!(r2.citation.contains("§ 1033(g)(4)"));

        // General — § 1033(a)(2)(B)(i).
        i.conversion_type = ConversionType::DestructionFireTheftCasualty;
        i.property_use = PropertyUse::TradeOrBusinessTangible;
        let r3 = compute(&i);
        assert!(r3.citation.contains("§ 1033(a)(2)(B)(i)"));
    }

    #[test]
    fn note_includes_replacement_window_status() {
        let mut i = base_input();
        i.months_to_replacement = 12;
        let r1 = compute(&i);
        assert!(r1.note.contains("SATISFIED"));

        i.months_to_replacement = 36;
        let r2 = compute(&i);
        assert!(r2.note.contains("MISSED"));
    }

    #[test]
    fn five_year_qualifying_disaster_satisfied_at_60_months() {
        let mut i = base_input();
        i.conversion_type = ConversionType::QualifyingDisaster;
        i.months_to_replacement = 60;
        let r = compute(&i);
        assert!(r.replacement_period_satisfied);
        assert_eq!(r.replacement_window_months, 60);
    }

    #[test]
    fn five_year_qualifying_disaster_missed_at_61_months() {
        let mut i = base_input();
        i.conversion_type = ConversionType::QualifyingDisaster;
        i.months_to_replacement = 61;
        let r = compute(&i);
        assert!(!r.replacement_period_satisfied);
    }

    #[test]
    fn four_year_disaster_residence_satisfied_at_48_months() {
        let mut i = base_input();
        i.conversion_type = ConversionType::FederallyDeclaredDisaster;
        i.property_use = PropertyUse::PersonalResidence;
        i.months_to_replacement = 48;
        let r = compute(&i);
        assert!(r.replacement_period_satisfied);
    }

    #[test]
    fn four_year_disaster_residence_missed_at_49_months() {
        let mut i = base_input();
        i.conversion_type = ConversionType::FederallyDeclaredDisaster;
        i.property_use = PropertyUse::PersonalResidence;
        i.months_to_replacement = 49;
        let r = compute(&i);
        assert!(!r.replacement_period_satisfied);
    }

    #[test]
    fn three_year_condemnation_satisfied_at_36_months() {
        let mut i = base_input();
        i.conversion_type = ConversionType::Condemnation;
        i.property_use = PropertyUse::InvestmentRealProperty;
        i.months_to_replacement = 36;
        let r = compute(&i);
        assert!(r.replacement_period_satisfied);
    }

    #[test]
    fn three_year_condemnation_missed_at_37_months() {
        let mut i = base_input();
        i.conversion_type = ConversionType::Condemnation;
        i.property_use = PropertyUse::InvestmentRealProperty;
        i.months_to_replacement = 37;
        let r = compute(&i);
        assert!(!r.replacement_period_satisfied);
    }
}
