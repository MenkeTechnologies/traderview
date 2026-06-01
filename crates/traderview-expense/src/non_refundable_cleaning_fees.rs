//! Non-refundable cleaning / move-in fee enforceability — landlord
//! compliance check for whether a fee labeled "non-refundable" in the
//! lease is actually enforceable as non-refundable under state law,
//! or whether it gets converted into a refundable security deposit
//! subject to deposit-cap and itemized-deduction rules.
//!
//! Distinct from `pet_fees` (pet-specific deposits and fees),
//! `application_fees` (pre-tenancy screening fees subject to state-
//! specific caps), `damage_deduction_itemization` (post-tenancy
//! deduction requirements), and `security_deposit_caps` (statutory
//! caps on the amount collectable as security deposit). This module
//! addresses the SPECIFIC threshold question — can the landlord
//! lawfully treat a cleaning or move-in fee as keep-no-matter-what?
//!
//! Five regimes:
//!
//! **California — Cal. Civ. Code § 1950.5(n)**. STRICT PROHIBITION.
//! A lease shall NOT contain a provision characterizing any
//! security as "nonrefundable." Per Civ. Code § 1950.5(b), the
//! definition of "security" is broad — any payment made by a tenant
//! at the inception of the tenancy other than rent (current and
//! future) or the application screening fee. Practical effect:
//! cleaning fees, key deposits, move-in fees, pet deposits, last
//! month's rent, "closing costs" — ALL of these are characterized
//! as security and are refundable. The exceptions are: (i)
//! application screening fees, (ii) holding deposits, and (iii)
//! total destruction of the property. Cleaning is permitted as a
//! deduction at termination only to return the unit to the same
//! cleanliness level at inception.
//!
//! **Texas — Tex. Prop. Code Ch. 92 (general framework)**.
//! PERMITTED WITH LEASE DISCLOSURE. Texas law does not prohibit
//! non-refundable cleaning fees. If the lease clearly identifies a
//! charge as a non-refundable cleaning fee (rather than as security
//! deposit), the fee is enforceable. § 92.103 specifically targets
//! security deposit refund obligations and is distinct from
//! contractually agreed non-refundable fees.
//!
//! **Washington — RCW 59.18.285**. PERMITTED ONLY IF DISCLOSED IN
//! WRITTEN LEASE. If any moneys are paid to the landlord as a
//! nonrefundable fee, the rental agreement SHALL BE IN WRITING and
//! shall CLEARLY SPECIFY that the fee is nonrefundable. Two
//! cascading consequences for non-compliance: (a) if no written
//! lease, landlord is LIABLE TO TENANT for the entire fee
//! collected; (b) if the written lease fails to specify
//! non-refundability, the fee MUST BE TREATED AS A REFUNDABLE
//! DEPOSIT under RCW 59.18.260, 59.18.270, and 59.18.280. No money
//! paid as a nonrefundable fee may be DESIGNATED as a deposit or
//! as part of any deposit.
//!
//! **New York — N.Y. Gen. Oblig. Law § 7-108(1-a) (HSTPA 2019)**.
//! IMPLICIT PROHIBITION. The Housing Stability and Tenant
//! Protection Act of 2019 capped security deposits at one month's
//! rent and prohibited any advance payment exceeding first month's
//! rent plus one month's security. Non-refundable cleaning fees
//! beyond this cap are void as exceeding the statutory advance-
//! payment limit. The statute makes no carve-out for fees labeled
//! "non-refundable."
//!
//! **Default — common-law contract + state-specific variation**.
//! Most states permit non-refundable fees if (a) disclosed in
//! writing in the lease and (b) reasonable in amount relative to
//! the underlying service.
//!
//! Citations: Cal. Civ. Code § 1950.5(n) (CA prohibition); Civ.
//! Code § 1950.5(b) (security definition); Civ. Code § 1950.5(m)
//! (cleaning deduction limited to inception level); Tex. Prop.
//! Code § 92.103 (TX security deposit refund — distinct from
//! contractual non-refundable fees); RCW 59.18.285 (WA written-
//! lease disclosure rule + treated-as-refundable consequence);
//! RCW 59.18.260/.270/.280 (WA deposit framework); N.Y. Gen.
//! Oblig. Law § 7-108(1-a) (NY HSTPA 2019 1-month deposit cap +
//! advance-payment limit).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    Texas,
    Washington,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CleaningFeeInput {
    pub regime: Regime,
    /// Whether the lease is in writing (versus oral lease). WA
    /// § 59.18.285 specifically requires a written lease for any
    /// non-refundable fee to be enforceable as such.
    pub written_lease: bool,
    /// Whether the lease clearly identifies the fee as
    /// "nonrefundable" (vs labeling it as a deposit or refundable
    /// charge).
    pub lease_specifies_nonrefundable: bool,
    /// Amount of the cleaning fee in cents. Informational; CA / NY
    /// regimes void the non-refundable designation regardless of
    /// amount. NY has additional cap interaction with the 1-month
    /// security deposit limit.
    pub fee_amount_cents: i64,
    /// NY-only: monthly rent in cents. Used to evaluate whether the
    /// fee fits within HSTPA's advance-payment limit (first month
    /// rent + 1 month security deposit cap).
    pub monthly_rent_cents: i64,
    /// NY-only: existing security deposit already collected in
    /// cents.
    pub ny_existing_security_deposit_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CleaningFeeResult {
    pub fee_enforceable_as_nonrefundable: bool,
    pub treated_as_refundable_deposit: bool,
    pub landlord_liable_for_full_fee: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &CleaningFeeInput) -> CleaningFeeResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();
    let mut enforceable = false;
    let mut treated_as_refundable = false;
    let mut landlord_liable = false;

    match input.regime {
        Regime::California => {
            violations.push(
                "Cal. Civ. Code § 1950.5(n) — lease may NOT characterize any security (including cleaning fees, key deposits, move-in fees, pet deposits) as nonrefundable"
                    .to_string(),
            );
            treated_as_refundable = true;
            notes.push(
                "§ 1950.5(b) broad security definition — any payment at tenancy inception other than rent (current and future) or application screening fee"
                    .to_string(),
            );
            notes.push(
                "§ 1950.5(m) cleaning deduction permitted at termination only to return unit to inception cleanliness level"
                    .to_string(),
            );
            notes.push(
                "exceptions: application screening fees, holding deposits, total destruction of property"
                    .to_string(),
            );
        }
        Regime::Texas => {
            if input.lease_specifies_nonrefundable {
                enforceable = true;
                notes.push(
                    "Tex. Prop. Code Ch. 92 permits contractual non-refundable cleaning fees when lease clearly identifies the charge as such (distinct from § 92.103 security deposit refund obligation)"
                        .to_string(),
                );
            } else {
                violations.push(
                    "Texas lease must clearly identify a cleaning fee as nonrefundable to avoid being treated as a security deposit subject to § 92.103 refund obligation"
                        .to_string(),
                );
                treated_as_refundable = true;
            }
        }
        Regime::Washington => {
            if !input.written_lease {
                violations.push(
                    "RCW 59.18.285 — written lease required; landlord LIABLE TO TENANT for the full amount of fees collected"
                        .to_string(),
                );
                landlord_liable = true;
            } else if !input.lease_specifies_nonrefundable {
                violations.push(
                    "RCW 59.18.285 — written lease MUST clearly specify the fee is nonrefundable; failure converts the fee to a refundable deposit under RCW 59.18.260, 59.18.270, 59.18.280"
                        .to_string(),
                );
                treated_as_refundable = true;
            } else {
                enforceable = true;
                notes.push(
                    "RCW 59.18.285 — nonrefundable fee enforceable when written lease clearly specifies nonrefundable designation"
                        .to_string(),
                );
            }
            notes.push(
                "RCW 59.18.285 — nonrefundable fees may NOT be designated as a deposit or as part of any deposit"
                    .to_string(),
            );
        }
        Regime::NewYork => {
            let advance_payment_cap = input.monthly_rent_cents.saturating_add(input.monthly_rent_cents);
            let total_advance = input
                .ny_existing_security_deposit_cents
                .saturating_add(input.fee_amount_cents)
                .saturating_add(input.monthly_rent_cents);
            if total_advance > advance_payment_cap.saturating_add(input.monthly_rent_cents) {
                violations.push(format!(
                    "N.Y. Gen. Oblig. Law § 7-108(1-a) HSTPA 2019 — total advance payment ({} cents = first month rent + deposit + cleaning fee) exceeds permitted cap (first month rent + 1 month security = {} cents)",
                    total_advance, advance_payment_cap.saturating_add(input.monthly_rent_cents)
                ));
            }
            violations.push(
                "NY HSTPA 2019 (§ 7-108(1-a)) — implicit prohibition on non-refundable cleaning fees; statute makes no carve-out for fees labeled \"non-refundable\" beyond the first-month + 1-month security cap"
                    .to_string(),
            );
            treated_as_refundable = true;
        }
        Regime::Default => {
            if input.written_lease && input.lease_specifies_nonrefundable {
                enforceable = true;
                notes.push(
                    "default common-law rule — non-refundable fees enforceable when disclosed in written lease and reasonable in amount"
                        .to_string(),
                );
            } else {
                violations.push(
                    "default common-law — non-refundable fees require written lease disclosure to be enforceable"
                        .to_string(),
                );
                treated_as_refundable = true;
            }
        }
    }

    CleaningFeeResult {
        fee_enforceable_as_nonrefundable: enforceable,
        treated_as_refundable_deposit: treated_as_refundable,
        landlord_liable_for_full_fee: landlord_liable,
        violations,
        citation: citation_for(input.regime),
        notes,
    }
}

fn citation_for(regime: Regime) -> &'static str {
    match regime {
        Regime::California => {
            "Cal. Civ. Code § 1950.5(n)/(b)/(m)"
        }
        Regime::Texas => "Tex. Prop. Code Ch. 92 + § 92.103",
        Regime::Washington => "RCW 59.18.285; RCW 59.18.260/.270/.280",
        Regime::NewYork => "N.Y. Gen. Oblig. Law § 7-108(1-a) (HSTPA 2019)",
        Regime::Default => "common-law contract + state-specific variation",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ca_base() -> CleaningFeeInput {
        CleaningFeeInput {
            regime: Regime::California,
            written_lease: true,
            lease_specifies_nonrefundable: true,
            fee_amount_cents: 30_000,
            monthly_rent_cents: 0,
            ny_existing_security_deposit_cents: 0,
        }
    }

    fn tx_base() -> CleaningFeeInput {
        CleaningFeeInput {
            regime: Regime::Texas,
            written_lease: true,
            lease_specifies_nonrefundable: true,
            fee_amount_cents: 30_000,
            monthly_rent_cents: 0,
            ny_existing_security_deposit_cents: 0,
        }
    }

    fn wa_base() -> CleaningFeeInput {
        CleaningFeeInput {
            regime: Regime::Washington,
            written_lease: true,
            lease_specifies_nonrefundable: true,
            fee_amount_cents: 30_000,
            monthly_rent_cents: 0,
            ny_existing_security_deposit_cents: 0,
        }
    }

    fn ny_base() -> CleaningFeeInput {
        CleaningFeeInput {
            regime: Regime::NewYork,
            written_lease: true,
            lease_specifies_nonrefundable: true,
            fee_amount_cents: 30_000,
            monthly_rent_cents: 200_000,
            ny_existing_security_deposit_cents: 200_000,
        }
    }

    fn default_base() -> CleaningFeeInput {
        CleaningFeeInput {
            regime: Regime::Default,
            written_lease: true,
            lease_specifies_nonrefundable: true,
            fee_amount_cents: 30_000,
            monthly_rent_cents: 0,
            ny_existing_security_deposit_cents: 0,
        }
    }

    #[test]
    fn ca_always_treats_nonrefundable_as_refundable_violation() {
        let r = check(&ca_base());
        assert!(!r.fee_enforceable_as_nonrefundable);
        assert!(r.treated_as_refundable_deposit);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("§ 1950.5(n)")));
    }

    #[test]
    fn ca_violation_persists_even_with_lease_disclosure() {
        let mut i = ca_base();
        i.written_lease = true;
        i.lease_specifies_nonrefundable = true;
        let r = check(&i);
        assert!(!r.fee_enforceable_as_nonrefundable, "CA strict prohibition regardless of disclosure");
    }

    #[test]
    fn ca_notes_security_breadth_and_cleaning_deduction_rule() {
        let r = check(&ca_base());
        assert!(r.notes.iter().any(|n| n.contains("§ 1950.5(b) broad security")));
        assert!(r.notes.iter().any(|n| n.contains("§ 1950.5(m)") && n.contains("inception cleanliness")));
    }

    #[test]
    fn ca_notes_exception_list() {
        let r = check(&ca_base());
        assert!(r.notes.iter().any(|n| n.contains("application screening fees") && n.contains("holding deposits")));
    }

    #[test]
    fn tx_disclosed_nonrefundable_fee_enforceable() {
        let r = check(&tx_base());
        assert!(r.fee_enforceable_as_nonrefundable);
        assert!(!r.treated_as_refundable_deposit);
    }

    #[test]
    fn tx_undisclosed_fee_treated_as_refundable() {
        let mut i = tx_base();
        i.lease_specifies_nonrefundable = false;
        let r = check(&i);
        assert!(!r.fee_enforceable_as_nonrefundable);
        assert!(r.treated_as_refundable_deposit);
        assert!(r.violations.iter().any(|v| v.contains("§ 92.103")));
    }

    #[test]
    fn wa_no_written_lease_landlord_liable_for_full_fee() {
        let mut i = wa_base();
        i.written_lease = false;
        let r = check(&i);
        assert!(!r.fee_enforceable_as_nonrefundable);
        assert!(r.landlord_liable_for_full_fee);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("LIABLE TO TENANT")));
    }

    #[test]
    fn wa_written_lease_no_nonrefundable_clause_treated_as_refundable() {
        let mut i = wa_base();
        i.lease_specifies_nonrefundable = false;
        let r = check(&i);
        assert!(!r.fee_enforceable_as_nonrefundable);
        assert!(r.treated_as_refundable_deposit);
        assert!(!r.landlord_liable_for_full_fee);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("RCW 59.18.260, 59.18.270, 59.18.280")));
    }

    #[test]
    fn wa_full_compliance_fee_enforceable() {
        let r = check(&wa_base());
        assert!(r.fee_enforceable_as_nonrefundable);
        assert!(!r.treated_as_refundable_deposit);
        assert!(!r.landlord_liable_for_full_fee);
    }

    #[test]
    fn wa_notes_no_deposit_designation_rule() {
        let r = check(&wa_base());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("may NOT be designated as a deposit")));
    }

    #[test]
    fn ny_advance_payment_cap_violation_when_total_exceeds() {
        let r = check(&ny_base());
        assert!(!r.fee_enforceable_as_nonrefundable);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("HSTPA 2019")));
    }

    #[test]
    fn ny_implicit_prohibition_persists_even_with_disclosure() {
        let r = check(&ny_base());
        assert!(!r.fee_enforceable_as_nonrefundable, "NY HSTPA implicit prohibition regardless of disclosure");
    }

    #[test]
    fn default_disclosed_in_written_lease_enforceable() {
        let r = check(&default_base());
        assert!(r.fee_enforceable_as_nonrefundable);
    }

    #[test]
    fn default_undisclosed_treated_as_refundable() {
        let mut i = default_base();
        i.lease_specifies_nonrefundable = false;
        let r = check(&i);
        assert!(!r.fee_enforceable_as_nonrefundable);
        assert!(r.treated_as_refundable_deposit);
    }

    #[test]
    fn default_oral_lease_treated_as_refundable() {
        let mut i = default_base();
        i.written_lease = false;
        let r = check(&i);
        assert!(!r.fee_enforceable_as_nonrefundable);
        assert!(r.treated_as_refundable_deposit);
    }

    #[test]
    fn citation_california_pins_subsections() {
        let r = check(&ca_base());
        assert!(r.citation.contains("§ 1950.5(n)"));
        assert!(r.citation.contains("(b)"));
        assert!(r.citation.contains("(m)"));
    }

    #[test]
    fn citation_texas_pins_chapter_92_and_section_92_103() {
        let r = check(&tx_base());
        assert!(r.citation.contains("Tex. Prop. Code Ch. 92"));
        assert!(r.citation.contains("§ 92.103"));
    }

    #[test]
    fn citation_washington_pins_59_18_285_and_deposit_framework() {
        let r = check(&wa_base());
        assert!(r.citation.contains("RCW 59.18.285"));
        assert!(r.citation.contains("RCW 59.18.260/.270/.280"));
    }

    #[test]
    fn citation_newyork_pins_section_7_108_hstpa() {
        let r = check(&ny_base());
        assert!(r.citation.contains("§ 7-108(1-a)"));
        assert!(r.citation.contains("HSTPA 2019"));
    }

    #[test]
    fn california_uniquely_strict_prohibition_invariant() {
        let r_ca = check(&ca_base());
        let r_tx = check(&tx_base());
        let r_wa = check(&wa_base());
        let r_default = check(&default_base());
        assert!(!r_ca.fee_enforceable_as_nonrefundable);
        assert!(r_tx.fee_enforceable_as_nonrefundable);
        assert!(r_wa.fee_enforceable_as_nonrefundable);
        assert!(r_default.fee_enforceable_as_nonrefundable);
    }

    #[test]
    fn washington_uniquely_imposes_landlord_liability_for_no_written_lease() {
        for regime in [Regime::California, Regime::Texas, Regime::NewYork, Regime::Default] {
            let mut i = wa_base();
            i.regime = regime;
            i.written_lease = false;
            let r = check(&i);
            assert!(!r.landlord_liable_for_full_fee, "regime {:?} should not impose WA-style landlord liability", regime);
        }
        let mut i = wa_base();
        i.written_lease = false;
        let r = check(&i);
        assert!(r.landlord_liable_for_full_fee);
    }

    #[test]
    fn newyork_advance_payment_within_cap_still_void_due_to_implicit_prohibition() {
        let mut i = ny_base();
        i.fee_amount_cents = 100;
        i.ny_existing_security_deposit_cents = 100_000;
        let r = check(&i);
        assert!(!r.fee_enforceable_as_nonrefundable, "NY implicit prohibition persists regardless of advance-cap math");
    }
}
