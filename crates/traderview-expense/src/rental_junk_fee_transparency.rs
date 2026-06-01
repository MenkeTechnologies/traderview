//! Rental junk fee + non-rent fee transparency compliance — when
//! a residential landlord advertises, leases, or renews a rental,
//! what statutory disclosure obligations attach to fees, charges,
//! or other amounts beyond base rent? Distinct from
//! `application_fees` (application-stage caps), `late_fee_caps`
//! (post-default late charges), `pet_fees` (pet deposits), and
//! `broker_fee_allocation` (broker fee party-allocation).
//!
//! Trader-landlord operational concern in an emerging
//! transparency-regulation wave. 2024-2026 statutory wave hit MA
//! (940 CMR 38.00 eff. 2025-09-02), CO (HB25-1090 eff. 2026-01-01),
//! California pending. Federal FTC unfair-or-deceptive-fees rule
//! pending. Failure to comply with single-price-disclosure rules
//! triggers state consumer-protection penalties + private rights
//! of action.
//!
//! **Four regimes**:
//!
//! **Massachusetts — 940 CMR 38.00 (effective September 2,
//! 2025)**. Issued March 3, 2025 under the Massachusetts Consumer
//! Protection Act (Chapter 93A). Sweeping regulation on Unfair and
//! Deceptive Fees. Landlords must CLEARLY AND CONSPICUOUSLY
//! disclose the TOTAL PRICE (inclusive of ALL fees, charges, or
//! other expenses) when the rental is initially advertised.
//! Applies broadly to residential rental housing advertising,
//! leasing, and renewals. Enforced by Massachusetts Attorney
//! General; private right of action under Chapter 93A treble
//! damages.
//!
//! **Colorado — HB25-1090 (effective January 1, 2026)**. Honest
//! Pricing / Junk Fee Law. Landlords must clearly and
//! conspicuously disclose the "total price" of the rental as a
//! SINGLE NUMBER, without separating into separate fees, charges,
//! or amounts. Prohibitions:
//! - utility markups ABOVE provider charges;
//! - costs associated with landlord responsibilities;
//! - markup fees exceeding 2% OR $10 per month;
//! - charges for services NOT provided;
//! - charges for COMMON AREA MAINTENANCE (CAM charges banned).
//!
//! **California — emerging**. AB 12 (2023, eff. July 1, 2024)
//! caps non-rent security to one month's rent in TPA-covered
//! units. Pending broader junk fee transparency legislation —
//! check current bill status.
//!
//! **Default — limited transparency obligation**. Federal FTC
//! 16 CFR Part 464 (Unfair or Deceptive Fees, finalized 2024)
//! applies to short-term rentals + hotels but residential
//! long-term rental coverage incomplete. State consumer-protection
//! statutes (UDAP) may apply for materially misleading
//! advertising.
//!
//! Citations: 940 CMR 38.00 (MA Unfair and Deceptive Fees);
//! M.G.L. c. 93A (MA Consumer Protection Act + treble damages);
//! Colo. Rev. Stat. § 38-12-1101 et seq. (CO Honest Pricing Law
//! HB25-1090); Cal. Civ. Code § 1950.5 (CA security deposit cap
//! per AB 12); 16 CFR Part 464 (FTC Unfair or Deceptive Fees
//! Rule); 15 U.S.C. § 45 (FTC Act § 5 UDAP).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Massachusetts,
    Colorado,
    California,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalJunkFeeTransparencyInput {
    pub regime: Regime,
    /// Whether the total price (inclusive of all fees) is
    /// disclosed as a single number in advertising.
    pub total_price_disclosed_as_single_number: bool,
    /// Whether the disclosure is "clearly and conspicuously"
    /// presented (typeface, placement, prominence).
    pub disclosure_clear_and_conspicuous: bool,
    /// Whether the landlord imposes a utility markup ABOVE the
    /// provider's actual charge to the landlord.
    pub utility_markup_above_provider_cost: bool,
    /// Whether the landlord charges Common Area Maintenance (CAM)
    /// fees in addition to rent.
    pub cam_charge_imposed: bool,
    /// Whether any markup fee exceeds the 2% / $10-per-month cap
    /// (CO-specific).
    pub markup_fee_exceeds_two_percent_or_ten_dollars: bool,
    /// Whether the landlord charges for services NOT provided to
    /// the tenant.
    pub charge_for_undelivered_service: bool,
    /// Whether the landlord shifts costs that are landlord's
    /// responsibility (property tax, maintenance, insurance) to
    /// the tenant via fees.
    pub landlord_responsibility_costs_shifted_to_tenant: bool,
    /// Whether the advertisement separates fees into individual
    /// line items rather than disclosing total price as single
    /// number.
    pub advertisement_separates_fees: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalJunkFeeTransparencyResult {
    pub compliant: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &RentalJunkFeeTransparencyInput) -> RentalJunkFeeTransparencyResult {
    match input.regime {
        Regime::Massachusetts => check_massachusetts(input),
        Regime::Colorado => check_colorado(input),
        Regime::California => check_california(input),
        Regime::Default => check_default(input),
    }
}

fn check_massachusetts(
    input: &RentalJunkFeeTransparencyInput,
) -> RentalJunkFeeTransparencyResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if !input.total_price_disclosed_as_single_number {
        violations.push(
            "940 CMR 38.00 — TOTAL PRICE inclusive of ALL fees, charges, or other expenses MUST be disclosed when rental is initially advertised"
                .to_string(),
        );
    }

    if !input.disclosure_clear_and_conspicuous {
        violations.push(
            "940 CMR 38.00 — disclosure of total price must be CLEARLY AND CONSPICUOUSLY presented; small print or buried disclosure insufficient"
                .to_string(),
        );
    }

    notes.push(
        "940 CMR 38.00 — Massachusetts Unfair and Deceptive Fees regulation issued March 3, 2025; effective September 2, 2025"
            .to_string(),
    );
    notes.push(
        "M.G.L. c. 93A — Massachusetts Consumer Protection Act enforcement; violation triggers TREBLE damages + attorney's fees + private right of action"
            .to_string(),
    );
    notes.push(
        "MA junk-fee regulation applies broadly to residential rental housing advertising, leasing, AND renewals"
            .to_string(),
    );

    let compliant = violations.is_empty();
    RentalJunkFeeTransparencyResult {
        compliant,
        violations,
        citation: "940 CMR 38.00; M.G.L. c. 93A",
        notes,
    }
}

fn check_colorado(input: &RentalJunkFeeTransparencyInput) -> RentalJunkFeeTransparencyResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if !input.total_price_disclosed_as_single_number || input.advertisement_separates_fees {
        violations.push(
            "Colo. Rev. Stat. § 38-12-1101 (HB25-1090) — total price MUST be disclosed as a SINGLE NUMBER without separating into separate fees, charges, or amounts"
                .to_string(),
        );
    }

    if input.utility_markup_above_provider_cost {
        violations.push(
            "Colo. Rev. Stat. § 38-12-1101 (HB25-1090) — landlord PROHIBITED from charging utility markup ABOVE provider's actual charges"
                .to_string(),
        );
    }

    if input.cam_charge_imposed {
        violations.push(
            "Colo. Rev. Stat. § 38-12-1101 (HB25-1090) — Common Area Maintenance (CAM) charges PROHIBITED for residential rentals"
                .to_string(),
        );
    }

    if input.markup_fee_exceeds_two_percent_or_ten_dollars {
        violations.push(
            "Colo. Rev. Stat. § 38-12-1101 (HB25-1090) — markup fees may NOT exceed 2% OR $10 per month, whichever is greater"
                .to_string(),
        );
    }

    if input.charge_for_undelivered_service {
        violations.push(
            "Colo. Rev. Stat. § 38-12-1101 (HB25-1090) — charges for services NOT actually provided PROHIBITED"
                .to_string(),
        );
    }

    if input.landlord_responsibility_costs_shifted_to_tenant {
        violations.push(
            "Colo. Rev. Stat. § 38-12-1101 (HB25-1090) — costs associated with landlord's responsibilities (property tax, maintenance, insurance) may NOT be passed through as separate tenant fees"
                .to_string(),
        );
    }

    notes.push(
        "Colo. Rev. Stat. § 38-12-1101 (HB25-1090 Honest Pricing Law) — effective January 1, 2026"
            .to_string(),
    );
    notes.push(
        "Colorado AG guidance — enforced by AG investigation + civil penalties + private right of action"
            .to_string(),
    );

    let compliant = violations.is_empty();
    RentalJunkFeeTransparencyResult {
        compliant,
        violations,
        citation: "Colo. Rev. Stat. § 38-12-1101 et seq. (HB25-1090)",
        notes,
    }
}

fn check_california(input: &RentalJunkFeeTransparencyInput) -> RentalJunkFeeTransparencyResult {
    let mut notes: Vec<String> = Vec::new();

    notes.push(
        "Cal. Civ. Code § 1950.5 (AB 12, eff. July 1, 2024) — non-rent security capped at ONE month's rent in TPA-covered units"
            .to_string(),
    );
    notes.push(
        "California — pending broader junk fee transparency legislation; consult current bill status"
            .to_string(),
    );

    if !input.total_price_disclosed_as_single_number {
        notes.push(
            "California — no statewide statutory total-price-disclosure mandate as of 2026-06; municipal ordinances (SF, LA, Berkeley, Oakland) may impose local disclosure requirements"
                .to_string(),
        );
    }

    RentalJunkFeeTransparencyResult {
        compliant: true,
        violations: Vec::new(),
        citation: "Cal. Civ. Code § 1950.5 (AB 12)",
        notes,
    }
}

fn check_default(input: &RentalJunkFeeTransparencyInput) -> RentalJunkFeeTransparencyResult {
    let mut notes: Vec<String> = Vec::new();

    notes.push(
        "default rule — most states have NO comprehensive junk-fee transparency statute; federal 16 CFR Part 464 (FTC Unfair or Deceptive Fees, finalized 2024) primarily covers short-term rentals + hotels"
            .to_string(),
    );
    notes.push(
        "15 U.S.C. § 45 (FTC Act § 5) — federal Unfair or Deceptive Acts or Practices doctrine applies to materially misleading advertising regardless of state statute"
            .to_string(),
    );

    if !input.total_price_disclosed_as_single_number {
        notes.push(
            "non-disclosure of total price may trigger state-level UDAP (Unfair and Deceptive Acts and Practices) consumer-protection statute review depending on state framework"
                .to_string(),
        );
    }

    RentalJunkFeeTransparencyResult {
        compliant: true,
        violations: Vec::new(),
        citation: "16 CFR Part 464 (FTC); 15 U.S.C. § 45; state-specific UDAP statutes",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ma_compliant() -> RentalJunkFeeTransparencyInput {
        RentalJunkFeeTransparencyInput {
            regime: Regime::Massachusetts,
            total_price_disclosed_as_single_number: true,
            disclosure_clear_and_conspicuous: true,
            utility_markup_above_provider_cost: false,
            cam_charge_imposed: false,
            markup_fee_exceeds_two_percent_or_ten_dollars: false,
            charge_for_undelivered_service: false,
            landlord_responsibility_costs_shifted_to_tenant: false,
            advertisement_separates_fees: false,
        }
    }

    fn co_compliant() -> RentalJunkFeeTransparencyInput {
        RentalJunkFeeTransparencyInput {
            regime: Regime::Colorado,
            total_price_disclosed_as_single_number: true,
            disclosure_clear_and_conspicuous: true,
            utility_markup_above_provider_cost: false,
            cam_charge_imposed: false,
            markup_fee_exceeds_two_percent_or_ten_dollars: false,
            charge_for_undelivered_service: false,
            landlord_responsibility_costs_shifted_to_tenant: false,
            advertisement_separates_fees: false,
        }
    }

    fn ca_base() -> RentalJunkFeeTransparencyInput {
        RentalJunkFeeTransparencyInput {
            regime: Regime::California,
            total_price_disclosed_as_single_number: false,
            disclosure_clear_and_conspicuous: false,
            utility_markup_above_provider_cost: false,
            cam_charge_imposed: false,
            markup_fee_exceeds_two_percent_or_ten_dollars: false,
            charge_for_undelivered_service: false,
            landlord_responsibility_costs_shifted_to_tenant: false,
            advertisement_separates_fees: false,
        }
    }

    fn default_base() -> RentalJunkFeeTransparencyInput {
        RentalJunkFeeTransparencyInput {
            regime: Regime::Default,
            total_price_disclosed_as_single_number: false,
            disclosure_clear_and_conspicuous: false,
            utility_markup_above_provider_cost: false,
            cam_charge_imposed: false,
            markup_fee_exceeds_two_percent_or_ten_dollars: false,
            charge_for_undelivered_service: false,
            landlord_responsibility_costs_shifted_to_tenant: false,
            advertisement_separates_fees: false,
        }
    }

    #[test]
    fn ma_clean_disclosure_compliant() {
        let r = check(&ma_compliant());
        assert!(r.compliant);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn ma_missing_total_price_violates() {
        let mut i = ma_compliant();
        i.total_price_disclosed_as_single_number = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("940 CMR 38.00") && v.contains("TOTAL PRICE")));
    }

    #[test]
    fn ma_buried_disclosure_violates() {
        let mut i = ma_compliant();
        i.disclosure_clear_and_conspicuous = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("CLEARLY AND CONSPICUOUSLY")));
    }

    #[test]
    fn ma_chapter_93a_treble_damages_note_present() {
        let r = check(&ma_compliant());
        assert!(r.notes.iter().any(|n| n.contains("c. 93A") && n.contains("TREBLE damages")));
    }

    #[test]
    fn ma_effective_date_2025_09_02_note_present() {
        let r = check(&ma_compliant());
        assert!(r.notes.iter().any(|n| n.contains("September 2, 2025")));
    }

    #[test]
    fn ma_advertising_leasing_renewals_scope_note() {
        let r = check(&ma_compliant());
        assert!(r.notes.iter().any(|n| n.contains("advertising, leasing, AND renewals")));
    }

    #[test]
    fn ma_citation_pins_940_cmr_38_and_chapter_93a() {
        let r = check(&ma_compliant());
        assert!(r.citation.contains("940 CMR 38.00"));
        assert!(r.citation.contains("c. 93A"));
    }

    #[test]
    fn co_clean_disclosure_compliant() {
        let r = check(&co_compliant());
        assert!(r.compliant);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn co_utility_markup_violates() {
        let mut i = co_compliant();
        i.utility_markup_above_provider_cost = true;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("utility markup ABOVE provider's actual charges")));
    }

    #[test]
    fn co_cam_charge_violates() {
        let mut i = co_compliant();
        i.cam_charge_imposed = true;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("Common Area Maintenance (CAM)")));
    }

    #[test]
    fn co_markup_above_2_percent_or_10_violates() {
        let mut i = co_compliant();
        i.markup_fee_exceeds_two_percent_or_ten_dollars = true;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("2% OR $10 per month")));
    }

    #[test]
    fn co_undelivered_service_charge_violates() {
        let mut i = co_compliant();
        i.charge_for_undelivered_service = true;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("services NOT actually provided")));
    }

    #[test]
    fn co_landlord_responsibility_passthrough_violates() {
        let mut i = co_compliant();
        i.landlord_responsibility_costs_shifted_to_tenant = true;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("landlord's responsibilities")));
    }

    #[test]
    fn co_separating_fees_in_advertisement_violates() {
        let mut i = co_compliant();
        i.advertisement_separates_fees = true;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.iter().any(|v| v.contains("SINGLE NUMBER")));
    }

    #[test]
    fn co_effective_date_2026_01_01_note_present() {
        let r = check(&co_compliant());
        assert!(r.notes.iter().any(|n| n.contains("January 1, 2026")));
    }

    #[test]
    fn co_all_six_violations_simultaneous() {
        let mut i = co_compliant();
        i.total_price_disclosed_as_single_number = false;
        i.utility_markup_above_provider_cost = true;
        i.cam_charge_imposed = true;
        i.markup_fee_exceeds_two_percent_or_ten_dollars = true;
        i.charge_for_undelivered_service = true;
        i.landlord_responsibility_costs_shifted_to_tenant = true;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.len() >= 6);
    }

    #[test]
    fn co_citation_pins_38_12_1101_and_hb25_1090() {
        let r = check(&co_compliant());
        assert!(r.citation.contains("§ 38-12-1101"));
        assert!(r.citation.contains("HB25-1090"));
    }

    #[test]
    fn ca_ab_12_security_deposit_cap_note_present() {
        let r = check(&ca_base());
        assert!(r.notes.iter().any(|n| n.contains("AB 12") && n.contains("ONE month's rent")));
    }

    #[test]
    fn ca_no_statewide_mandate_note_when_missing_disclosure() {
        let r = check(&ca_base());
        assert!(r.notes.iter().any(|n| n.contains("no statewide statutory total-price-disclosure mandate")));
    }

    #[test]
    fn ca_compliant_even_without_full_disclosure() {
        let r = check(&ca_base());
        assert!(r.compliant);
    }

    #[test]
    fn default_ftc_part_464_note_present() {
        let r = check(&default_base());
        assert!(r.notes.iter().any(|n| n.contains("16 CFR Part 464") && n.contains("short-term rentals")));
    }

    #[test]
    fn default_ftc_act_section_5_note_present() {
        let r = check(&default_base());
        assert!(r.notes.iter().any(|n| n.contains("15 U.S.C. § 45") && n.contains("Unfair or Deceptive")));
    }

    #[test]
    fn default_udap_note_when_no_disclosure() {
        let r = check(&default_base());
        assert!(r.notes.iter().any(|n| n.contains("UDAP")));
    }

    #[test]
    fn default_no_violations_even_with_aggressive_fees() {
        let mut i = default_base();
        i.utility_markup_above_provider_cost = true;
        i.cam_charge_imposed = true;
        i.markup_fee_exceeds_two_percent_or_ten_dollars = true;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn co_unique_cam_prohibition_invariant() {
        let mut i_co = co_compliant();
        i_co.cam_charge_imposed = true;
        let r_co = check(&i_co);
        assert!(!r_co.compliant);

        for regime in [Regime::Massachusetts, Regime::California, Regime::Default] {
            let mut i = co_compliant();
            i.regime = regime;
            i.cam_charge_imposed = true;
            let r = check(&i);
            let cam_violations: Vec<_> = r.violations.iter().filter(|v| v.contains("Common Area Maintenance")).collect();
            assert!(cam_violations.is_empty(), "regime {:?} should not flag CAM as violation", regime);
        }
    }

    #[test]
    fn co_unique_utility_markup_prohibition_invariant() {
        let mut i_co = co_compliant();
        i_co.utility_markup_above_provider_cost = true;
        let r_co = check(&i_co);
        assert!(!r_co.compliant);

        for regime in [Regime::Massachusetts, Regime::California, Regime::Default] {
            let mut i = co_compliant();
            i.regime = regime;
            i.utility_markup_above_provider_cost = true;
            let r = check(&i);
            let util_violations: Vec<_> = r.violations.iter().filter(|v| v.contains("utility markup")).collect();
            assert!(util_violations.is_empty(), "regime {:?} should not flag utility markup as violation", regime);
        }
    }

    #[test]
    fn four_regimes_routed_correctly() {
        for regime in [Regime::Massachusetts, Regime::Colorado, Regime::California, Regime::Default] {
            let mut i = ma_compliant();
            i.regime = regime;
            let r = check(&i);
            let _ = r.compliant;
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn ma_compliant_clean_path_no_violations() {
        let r = check(&ma_compliant());
        assert_eq!(r.violations.len(), 0);
    }

    #[test]
    fn co_compliant_clean_path_no_violations() {
        let r = check(&co_compliant());
        assert_eq!(r.violations.len(), 0);
    }
}
