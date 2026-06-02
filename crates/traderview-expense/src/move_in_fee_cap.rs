//! Move-in fee cap and disclosure obligations — when a
//! landlord charges a one-time non-refundable move-in fee
//! (cleaning + screening + administrative + lease-prep), how
//! much may that fee be, what purposes may it cover, and
//! what disclosure must accompany it? Trader-landlord
//! critical: misclassifying a refundable security deposit as
//! a non-refundable fee in a regulated jurisdiction
//! reclassifies the entire amount as a refundable deposit
//! AND can void enforcement.
//!
//! Distinct from siblings `application_fees` (pre-tenancy
//! screening cost), `late_fee_caps` (post-tenancy delinquency
//! penalty), `advance_rent_limit` (advance rent payments),
//! and `move_in_inspection` (procedural walk-through).
//!
//! **Four regimes**:
//!
//! **Seattle SMC § 7.24 + RCW 59.18.285 (most explicit)** —
//! non-refundable fees ONLY for cleaning + screening, capped
//! at 10% of one month's rent. Security deposit + fees
//! combined CANNOT exceed one month's rent. Disclosure as
//! non-refundable required by RCW 59.18.285.
//!
//! **Washington RCW 59.18.285 (state-wide disclosure-only)**
//! — landlord must disclose purpose and amount of all
//! non-refundable move-in fees in the lease. If the lease
//! does not specify a fee as non-refundable, it MUST be
//! treated as a refundable security deposit. No state-wide
//! amount cap (Seattle layers caps on top).
//!
//! **Chicago RLTO (Chicago Mun. Code § 5-12)** — no amount
//! cap. Move-in fee is landlord's property; can be used for
//! any expense; no interest requirement. Itemized purpose
//! disclosure required. Effectively shifted Chicago
//! landlords from regulated security deposits to unregulated
//! non-refundable move-in fees.
//!
//! **Default — no cap, no disclosure obligation** other than
//! common-law unconscionability + lease integration.
//!
//! Citations: Seattle SMC § 7.24.030; RCW 59.18.285;
//! RCW 59.18.610; Chicago RLTO Chicago Mun. Code § 5-12-080,
//! § 5-12-081 (security deposit + move-in fee distinction).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Seattle,
    Washington,
    Chicago,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FeePurpose {
    /// Screening/background check.
    Screening,
    /// Cleaning charge.
    Cleaning,
    /// Lease preparation / administrative.
    LeasePreparation,
    /// Pet-related fee.
    Pet,
    /// Key replacement.
    KeyReplacement,
    /// Other / unspecified.
    Other,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MoveInFeeCapInput {
    pub regime: Regime,
    pub fee_purpose: FeePurpose,
    pub one_month_rent_cents: i64,
    pub fee_amount_cents: i64,
    pub security_deposit_cents: i64,
    pub disclosed_as_nonrefundable: bool,
    pub itemized_purpose_in_lease: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct MoveInFeeCapResult {
    pub compliant: bool,
    pub max_allowed_fee_cents: i64,
    pub fee_reclassified_as_deposit: bool,
    pub combined_cap_engaged: bool,
    pub combined_cap_cents: i64,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &MoveInFeeCapInput) -> MoveInFeeCapResult {
    let rent = input.one_month_rent_cents.max(0);
    let fee = input.fee_amount_cents.max(0);
    let deposit = input.security_deposit_cents.max(0);

    match input.regime {
        Regime::Seattle => check_seattle(input, rent, fee, deposit),
        Regime::Washington => check_washington(input, fee),
        Regime::Chicago => check_chicago(input),
        Regime::Default => check_default(input, fee),
    }
}

fn check_seattle(
    input: &MoveInFeeCapInput,
    rent: i64,
    fee: i64,
    deposit: i64,
) -> MoveInFeeCapResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Seattle SMC § 7.24.030 — non-refundable move-in fees ONLY for cleaning + screening; capped at 10% of one month's rent; security deposit + fees COMBINED cannot exceed one month's rent"
            .to_string(),
        "RCW 59.18.285 — must disclose as non-refundable in lease; otherwise treated as refundable security deposit"
            .to_string(),
    ];

    let max_fee = rent / 10;
    let permissible_purpose = matches!(
        input.fee_purpose,
        FeePurpose::Cleaning | FeePurpose::Screening
    );

    if !permissible_purpose {
        violations.push(
            "Seattle SMC § 7.24.030 — non-refundable move-in fees may ONLY be for cleaning or screening".to_string(),
        );
    }

    if fee > max_fee {
        violations.push(format!(
            "Seattle SMC § 7.24.030 — fee ${} cents exceeds 10% of one month's rent cap (${} cents)",
            fee, max_fee
        ));
    }

    let combined = fee.saturating_add(deposit);
    let combined_cap_violated = combined > rent;
    if combined_cap_violated {
        violations.push(format!(
            "Seattle SMC § 7.24.030 — combined security deposit + move-in fees ${} cents exceed one month's rent (${} cents)",
            combined, rent
        ));
    }

    let reclassified = !input.disclosed_as_nonrefundable;
    if reclassified {
        violations.push(
            "RCW 59.18.285 — fee not disclosed as non-refundable in lease; reclassified as refundable security deposit".to_string(),
        );
    }

    MoveInFeeCapResult {
        compliant: violations.is_empty(),
        max_allowed_fee_cents: max_fee,
        fee_reclassified_as_deposit: reclassified,
        combined_cap_engaged: true,
        combined_cap_cents: rent,
        violations,
        citation: "Seattle SMC § 7.24.030; RCW 59.18.285; RCW 59.18.610",
        notes,
    }
}

fn check_washington(input: &MoveInFeeCapInput, _fee: i64) -> MoveInFeeCapResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "RCW 59.18.285 — landlord must disclose purpose and amount of all non-refundable move-in fees in the lease"
            .to_string(),
        "RCW 59.18.285 — if lease does not specify a fee as non-refundable, fee MUST be treated as refundable security deposit; no state-wide amount cap"
            .to_string(),
    ];

    let reclassified = !input.disclosed_as_nonrefundable;
    if reclassified {
        violations.push(
            "RCW 59.18.285 — fee not disclosed as non-refundable in lease; reclassified as refundable security deposit".to_string(),
        );
    }

    MoveInFeeCapResult {
        compliant: violations.is_empty(),
        max_allowed_fee_cents: 0,
        fee_reclassified_as_deposit: reclassified,
        combined_cap_engaged: false,
        combined_cap_cents: 0,
        violations,
        citation: "RCW 59.18.285",
        notes,
    }
}

fn check_chicago(input: &MoveInFeeCapInput) -> MoveInFeeCapResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "Chicago Mun. Code § 5-12-080 + § 5-12-081 — no amount cap on non-refundable move-in fees; itemized purpose disclosure required"
            .to_string(),
        "Chicago RLTO — non-refundable move-in fee is landlord's property; can be used for any expense; no interest obligation (contrast with security deposit + interest under § 5-12-081)"
            .to_string(),
    ];

    if !input.itemized_purpose_in_lease {
        violations.push(
            "Chicago Mun. Code § 5-12-080 — landlord MUST provide itemized list detailing what the non-refundable move-in fee is for".to_string(),
        );
    }

    MoveInFeeCapResult {
        compliant: violations.is_empty(),
        max_allowed_fee_cents: 0,
        fee_reclassified_as_deposit: false,
        combined_cap_engaged: false,
        combined_cap_cents: 0,
        violations,
        citation: "Chicago Mun. Code § 5-12-080, § 5-12-081 (Chicago RLTO)",
        notes,
    }
}

fn check_default(_input: &MoveInFeeCapInput, _fee: i64) -> MoveInFeeCapResult {
    let notes: Vec<String> = vec![
        "default rule — no statutory cap on non-refundable move-in fees; no specific disclosure obligation other than common-law unconscionability and lease integration"
            .to_string(),
        "default rule — local protections may apply (Bellingham WA, Portland OR, Minneapolis MN, etc.); verify jurisdiction-specific ordinances before relying on default rule"
            .to_string(),
    ];

    MoveInFeeCapResult {
        compliant: true,
        max_allowed_fee_cents: 0,
        fee_reclassified_as_deposit: false,
        combined_cap_engaged: false,
        combined_cap_cents: 0,
        violations: Vec::new(),
        citation: "common-law unconscionability + lease integration",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seattle_base() -> MoveInFeeCapInput {
        MoveInFeeCapInput {
            regime: Regime::Seattle,
            fee_purpose: FeePurpose::Cleaning,
            one_month_rent_cents: 200_000,
            fee_amount_cents: 15_000,
            security_deposit_cents: 100_000,
            disclosed_as_nonrefundable: true,
            itemized_purpose_in_lease: true,
        }
    }

    fn washington_base() -> MoveInFeeCapInput {
        let mut i = seattle_base();
        i.regime = Regime::Washington;
        i
    }

    fn chicago_base() -> MoveInFeeCapInput {
        let mut i = seattle_base();
        i.regime = Regime::Chicago;
        i.security_deposit_cents = 0;
        i
    }

    fn default_base() -> MoveInFeeCapInput {
        let mut i = seattle_base();
        i.regime = Regime::Default;
        i
    }

    #[test]
    fn seattle_compliant_passes() {
        let r = check(&seattle_base());
        assert!(r.compliant);
        assert!(r.combined_cap_engaged);
        assert_eq!(r.max_allowed_fee_cents, 20_000);
        assert_eq!(r.combined_cap_cents, 200_000);
    }

    #[test]
    fn seattle_fee_at_10_percent_boundary_compliant() {
        let mut i = seattle_base();
        i.fee_amount_cents = 20_000;
        i.security_deposit_cents = 180_000;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn seattle_fee_exceeds_10_percent_violates() {
        let mut i = seattle_base();
        i.fee_amount_cents = 25_000;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("10%") && v.contains("25000")));
    }

    #[test]
    fn seattle_pet_fee_purpose_violates() {
        let mut i = seattle_base();
        i.fee_purpose = FeePurpose::Pet;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("ONLY be for cleaning or screening")));
    }

    #[test]
    fn seattle_screening_purpose_compliant() {
        let mut i = seattle_base();
        i.fee_purpose = FeePurpose::Screening;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn seattle_combined_cap_violation() {
        let mut i = seattle_base();
        i.fee_amount_cents = 20_000;
        i.security_deposit_cents = 200_000;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("combined security deposit + move-in fees")));
    }

    #[test]
    fn seattle_undisclosed_reclassifies_as_deposit() {
        let mut i = seattle_base();
        i.disclosed_as_nonrefundable = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.fee_reclassified_as_deposit);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("RCW 59.18.285") && v.contains("reclassified")));
    }

    #[test]
    fn seattle_citation_pins_all_three_statutes() {
        let r = check(&seattle_base());
        assert!(r.citation.contains("SMC § 7.24.030"));
        assert!(r.citation.contains("RCW 59.18.285"));
        assert!(r.citation.contains("RCW 59.18.610"));
    }

    #[test]
    fn washington_compliant_when_disclosed() {
        let r = check(&washington_base());
        assert!(r.compliant);
        assert!(!r.combined_cap_engaged);
        assert_eq!(r.max_allowed_fee_cents, 0);
    }

    #[test]
    fn washington_undisclosed_reclassifies_as_deposit() {
        let mut i = washington_base();
        i.disclosed_as_nonrefundable = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.fee_reclassified_as_deposit);
    }

    #[test]
    fn washington_no_amount_cap_high_fee_ok() {
        let mut i = washington_base();
        i.fee_amount_cents = 500_000;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn washington_citation_pins_rcw() {
        let r = check(&washington_base());
        assert!(r.citation.contains("RCW 59.18.285"));
    }

    #[test]
    fn chicago_compliant_with_itemized_purpose() {
        let r = check(&chicago_base());
        assert!(r.compliant);
        assert!(!r.combined_cap_engaged);
    }

    #[test]
    fn chicago_no_amount_cap_high_fee_ok() {
        let mut i = chicago_base();
        i.fee_amount_cents = 1_000_000;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn chicago_missing_itemization_violates() {
        let mut i = chicago_base();
        i.itemized_purpose_in_lease = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 5-12-080") && v.contains("itemized list")));
    }

    #[test]
    fn chicago_citation_pins_rlto_sections() {
        let r = check(&chicago_base());
        assert!(r.citation.contains("§ 5-12-080"));
        assert!(r.citation.contains("§ 5-12-081"));
        assert!(r.citation.contains("Chicago RLTO"));
    }

    #[test]
    fn default_compliant_always() {
        let r = check(&default_base());
        assert!(r.compliant);
        assert_eq!(r.max_allowed_fee_cents, 0);
    }

    #[test]
    fn default_no_cap_or_disclosure_obligation() {
        let mut i = default_base();
        i.fee_amount_cents = 1_000_000;
        i.disclosed_as_nonrefundable = false;
        i.itemized_purpose_in_lease = false;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn four_regimes_routed_correctly() {
        for regime in [Regime::Seattle, Regime::Washington, Regime::Chicago, Regime::Default] {
            let mut i = seattle_base();
            i.regime = regime;
            let r = check(&i);
            assert!(!r.citation.is_empty());
        }
    }

    #[test]
    fn seattle_uniquely_engages_combined_cap_invariant() {
        let r_seattle = check(&seattle_base());
        assert!(r_seattle.combined_cap_engaged);

        for regime in [Regime::Washington, Regime::Chicago, Regime::Default] {
            let mut i = seattle_base();
            i.regime = regime;
            let r = check(&i);
            assert!(!r.combined_cap_engaged);
        }
    }

    #[test]
    fn defensive_negative_rent_clamped() {
        let mut i = seattle_base();
        i.one_month_rent_cents = -100_000;
        i.fee_amount_cents = 5_000;
        let r = check(&i);
        assert_eq!(r.max_allowed_fee_cents, 0);
        assert!(!r.compliant);
    }

    #[test]
    fn defensive_negative_fee_clamped() {
        let mut i = seattle_base();
        i.fee_amount_cents = -50_000;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn fee_purpose_truth_table_seattle() {
        for (purpose, exp_violation) in [
            (FeePurpose::Cleaning, false),
            (FeePurpose::Screening, false),
            (FeePurpose::LeasePreparation, true),
            (FeePurpose::Pet, true),
            (FeePurpose::KeyReplacement, true),
            (FeePurpose::Other, true),
        ] {
            let mut i = seattle_base();
            i.fee_purpose = purpose;
            let r = check(&i);
            let purpose_violation = r
                .violations
                .iter()
                .any(|v| v.contains("ONLY be for cleaning or screening"));
            assert_eq!(purpose_violation, exp_violation);
        }
    }

    #[test]
    fn seattle_stacks_3_violations_when_high_fee_wrong_purpose_undisclosed() {
        let mut i = seattle_base();
        i.fee_amount_cents = 50_000;
        i.fee_purpose = FeePurpose::Pet;
        i.disclosed_as_nonrefundable = false;
        let r = check(&i);
        assert!(!r.compliant);
        assert!(r.violations.len() >= 3);
    }

    #[test]
    fn defensive_overflow_combined_saturating() {
        let mut i = seattle_base();
        i.fee_amount_cents = i64::MAX - 1_000_000;
        i.security_deposit_cents = i64::MAX - 1_000_000;
        let r = check(&i);
        assert!(!r.compliant);
    }

    #[test]
    fn seattle_note_pins_10_percent_cap_and_combined_cap() {
        let r = check(&seattle_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("10% of one month's rent") && n.contains("COMBINED")));
    }

    #[test]
    fn washington_note_pins_reclassification_rule() {
        let r = check(&washington_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("treated as refundable security deposit")));
    }

    #[test]
    fn chicago_note_pins_landlords_property_rule() {
        let r = check(&chicago_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("landlord's property") && n.contains("no interest")));
    }

    #[test]
    fn default_note_pins_local_protections_warning() {
        let r = check(&default_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Bellingham") && n.contains("Portland")));
    }

    #[test]
    fn seattle_at_1_month_combined_boundary_compliant() {
        let mut i = seattle_base();
        i.fee_amount_cents = 20_000;
        i.security_deposit_cents = 180_000;
        let r = check(&i);
        assert!(r.compliant);
    }

    #[test]
    fn seattle_1_cent_over_combined_violates() {
        let mut i = seattle_base();
        i.fee_amount_cents = 20_000;
        i.security_deposit_cents = 180_001;
        let r = check(&i);
        assert!(!r.compliant);
    }

    #[test]
    fn reclassification_uniquely_in_seattle_and_washington_invariant() {
        for regime in [Regime::Seattle, Regime::Washington] {
            let mut i = seattle_base();
            i.regime = regime;
            i.disclosed_as_nonrefundable = false;
            let r = check(&i);
            assert!(r.fee_reclassified_as_deposit);
        }

        for regime in [Regime::Chicago, Regime::Default] {
            let mut i = chicago_base();
            i.regime = regime;
            i.disclosed_as_nonrefundable = false;
            let r = check(&i);
            assert!(!r.fee_reclassified_as_deposit);
        }
    }

    #[test]
    fn chicago_uniquely_requires_itemized_purpose_invariant() {
        let mut i_chicago = chicago_base();
        i_chicago.itemized_purpose_in_lease = false;
        let r_chicago = check(&i_chicago);
        assert!(!r_chicago.compliant);

        let mut i_default = default_base();
        i_default.itemized_purpose_in_lease = false;
        let r_default = check(&i_default);
        assert!(r_default.compliant);
    }
}
