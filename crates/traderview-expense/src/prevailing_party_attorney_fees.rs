//! State prevailing-party attorney-fees lease-clause compliance check.
//!
//! Addresses a common landlord drafting trap: lease clauses that award
//! attorney fees only to the LANDLORD on default are often unenforceable
//! as written in states with a reciprocity statute — the clause is
//! transformed by operation of law into a bilateral right, so a
//! prevailing TENANT collects the landlord's fees even though the
//! clause never said so.
//!
//! Two states have explicit anti-one-way reciprocity statutes. Both
//! also bar contractual waiver of the reciprocity itself.
//!
//! California (Cal. Civ. Code § 1717) — applies to "any action on a
//! contract" where the contract provides that attorney fees and costs
//! shall be awarded to one of the parties OR the prevailing party. The
//! one-way clause is automatically construed as MUTUAL; the prevailing
//! party gets fees regardless of which party was named. § 1717 is a
//! fundamental policy of California; choice-of-law clauses pointing to
//! other-state law are overridden in residential contracts.
//!
//! Washington (RCW 4.84.330) — same rule for any contract or lease
//! entered into AFTER 1977-09-21. Waiver is explicitly PROHIBITED — any
//! waiver clause is void. Pre-1977 leases are grandfathered out.
//!
//! Default — American Rule applies absent a contract clause. One-way
//! clauses are enforced as written (the non-prevailing party gets
//! nothing even if the named beneficiary loses).
//!
//! Citations: Cal. Civ. Code § 1717(a) (reciprocity); § 1717(b)(1)
//! (prevailing-party definition); § 1717(d) (severability + waiver
//! limits); RCW 4.84.330 (WA reciprocity); RCW 4.84.330 final clause
//! (waiver prohibited).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    CaliforniaCivCode1717,
    WashingtonRcw484330,
    Default,
}

impl Regime {
    pub fn for_state(state: &str) -> Self {
        match state.trim().to_ascii_uppercase().as_str() {
            "CA" => Self::CaliforniaCivCode1717,
            "WA" => Self::WashingtonRcw484330,
            _ => Self::Default,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Party {
    Landlord,
    Tenant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClauseScope {
    /// No attorney-fee clause at all — American Rule applies.
    NoClause,
    /// Clause names ONE party (typically landlord) as the sole beneficiary.
    OneWayLandlord,
    OneWayTenant,
    /// Clause is bilateral (both-or-prevailing-party language).
    Bilateral,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PrevailingPartyFeesInput {
    pub regime: Regime,
    pub clause_scope: ClauseScope,
    pub prevailing_party: Party,
    /// Whether the lease was entered into after 1977-09-21 — Washington's
    /// statutory effective date. Pre-statute leases are not reformed.
    pub lease_entered_after_1977_09_21: bool,
    pub attorneys_fees_incurred_cents: i64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PrevailingPartyFeesResult {
    pub regime: Regime,
    pub reciprocity_applies: bool,
    pub prevailing_party_entitled_to_fees: bool,
    pub amount_recoverable_cents: i64,
    pub citation: &'static str,
    pub note: String,
}

pub fn check(input: &PrevailingPartyFeesInput) -> PrevailingPartyFeesResult {
    let fees = input.attorneys_fees_incurred_cents.max(0);
    if input.clause_scope == ClauseScope::NoClause {
        return PrevailingPartyFeesResult {
            regime: input.regime,
            reciprocity_applies: false,
            prevailing_party_entitled_to_fees: false,
            amount_recoverable_cents: 0,
            citation:
                "American Rule — each party bears own attorney fees absent a contract clause or fee-shifting statute",
            note: "Lease contains no attorney-fee clause. American Rule applies; neither party recovers fees from the other.".to_string(),
        };
    }
    match input.regime {
        Regime::CaliforniaCivCode1717 => ca_check(input, fees),
        Regime::WashingtonRcw484330 => wa_check(input, fees),
        Regime::Default => default_check(input, fees),
    }
}

fn ca_check(input: &PrevailingPartyFeesInput, fees: i64) -> PrevailingPartyFeesResult {
    let reciprocity = matches!(
        input.clause_scope,
        ClauseScope::OneWayLandlord | ClauseScope::OneWayTenant | ClauseScope::Bilateral
    );
    PrevailingPartyFeesResult {
        regime: Regime::CaliforniaCivCode1717,
        reciprocity_applies: reciprocity && input.clause_scope != ClauseScope::Bilateral,
        prevailing_party_entitled_to_fees: true,
        amount_recoverable_cents: fees,
        citation: match input.clause_scope {
            ClauseScope::OneWayLandlord | ClauseScope::OneWayTenant => {
                "Cal. Civ. Code § 1717(a) — one-way attorney-fee clause construed as MUTUAL; prevailing party entitled regardless of which party clause named"
            }
            ClauseScope::Bilateral => {
                "Cal. Civ. Code § 1717(a) — bilateral attorney-fee clause; prevailing party entitled to fees"
            }
            ClauseScope::NoClause => unreachable!(),
        },
        note: format!(
            "{:?} party prevails. § 1717 transforms any one-way fee clause into a reciprocal obligation. Prevailing party recovers {} cents in attorney fees regardless of which party the clause named.",
            input.prevailing_party, fees
        ),
    }
}

fn wa_check(input: &PrevailingPartyFeesInput, fees: i64) -> PrevailingPartyFeesResult {
    // RCW 4.84.330 only applies to leases entered AFTER 1977-09-21.
    if !input.lease_entered_after_1977_09_21 {
        return PrevailingPartyFeesResult {
            regime: Regime::WashingtonRcw484330,
            reciprocity_applies: false,
            prevailing_party_entitled_to_fees: input.clause_scope == ClauseScope::Bilateral
                || (input.clause_scope == ClauseScope::OneWayLandlord
                    && input.prevailing_party == Party::Landlord)
                || (input.clause_scope == ClauseScope::OneWayTenant
                    && input.prevailing_party == Party::Tenant),
            amount_recoverable_cents: 0,
            citation:
                "RCW 4.84.330 — reciprocity applies only to leases entered AFTER 1977-09-21; pre-statute lease enforced as written",
            note: "Lease entered before 1977-09-21 is grandfathered out of RCW 4.84.330. One-way clause enforced as written.".to_string(),
        };
    }
    let reciprocity = matches!(
        input.clause_scope,
        ClauseScope::OneWayLandlord | ClauseScope::OneWayTenant
    );
    PrevailingPartyFeesResult {
        regime: Regime::WashingtonRcw484330,
        reciprocity_applies: reciprocity,
        prevailing_party_entitled_to_fees: true,
        amount_recoverable_cents: fees,
        citation: match input.clause_scope {
            ClauseScope::OneWayLandlord | ClauseScope::OneWayTenant => {
                "RCW 4.84.330 — one-way fee clause construed as reciprocal; prevailing party entitled regardless; waiver PROHIBITED"
            }
            ClauseScope::Bilateral => {
                "RCW 4.84.330 — bilateral fee clause; prevailing party entitled to fees; waiver PROHIBITED"
            }
            ClauseScope::NoClause => unreachable!(),
        },
        note: format!(
            "Post-1977 WA lease: {:?} party prevails. RCW 4.84.330 reciprocity applies. Prevailing party recovers {} cents in attorney fees. Waiver of reciprocity is VOID.",
            input.prevailing_party, fees
        ),
    }
}

fn default_check(input: &PrevailingPartyFeesInput, fees: i64) -> PrevailingPartyFeesResult {
    // No reciprocity statute — one-way clauses enforced as written.
    let entitled = match input.clause_scope {
        ClauseScope::OneWayLandlord => input.prevailing_party == Party::Landlord,
        ClauseScope::OneWayTenant => input.prevailing_party == Party::Tenant,
        ClauseScope::Bilateral => true,
        ClauseScope::NoClause => false,
    };
    PrevailingPartyFeesResult {
        regime: Regime::Default,
        reciprocity_applies: false,
        prevailing_party_entitled_to_fees: entitled,
        amount_recoverable_cents: if entitled { fees } else { 0 },
        citation:
            "No statewide attorney-fee reciprocity statute — one-way lease clauses enforced as written; American Rule otherwise",
        note: if entitled {
            format!(
                "Default regime: {:?} party prevails and the clause names this party. Recovers {} cents in attorney fees.",
                input.prevailing_party, fees
            )
        } else {
            format!(
                "Default regime: {:?} party prevails but the clause names the opposing party; reciprocity does NOT apply. No fee recovery.",
                input.prevailing_party
            )
        },
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;

    fn input(
        regime: Regime,
        scope: ClauseScope,
        prevailing: Party,
        post_1977: bool,
        fees: i64,
    ) -> PrevailingPartyFeesInput {
        PrevailingPartyFeesInput {
            regime,
            clause_scope: scope,
            prevailing_party: prevailing,
            lease_entered_after_1977_09_21: post_1977,
            attorneys_fees_incurred_cents: fees,
        }
    }

    #[test]
    fn no_clause_american_rule_no_recovery() {
        let r = check(&input(
            Regime::CaliforniaCivCode1717,
            ClauseScope::NoClause,
            Party::Tenant,
            true,
            10_000_00,
        ));
        assert!(!r.prevailing_party_entitled_to_fees);
        assert_eq!(r.amount_recoverable_cents, 0);
        assert!(r.citation.contains("American Rule"));
    }

    #[test]
    fn ca_one_way_landlord_clause_tenant_prevails_gets_fees() {
        // Classic CA § 1717 case: lease has one-way clause favoring
        // landlord, but tenant wins the eviction case. § 1717 transforms
        // the clause and tenant collects fees from landlord.
        let r = check(&input(
            Regime::CaliforniaCivCode1717,
            ClauseScope::OneWayLandlord,
            Party::Tenant,
            true,
            10_000_00,
        ));
        assert!(r.reciprocity_applies);
        assert!(r.prevailing_party_entitled_to_fees);
        assert_eq!(r.amount_recoverable_cents, 10_000_00);
        assert!(r.citation.contains("§ 1717(a)"));
        assert!(r.citation.contains("MUTUAL"));
    }

    #[test]
    fn ca_one_way_landlord_landlord_prevails_gets_fees() {
        let r = check(&input(
            Regime::CaliforniaCivCode1717,
            ClauseScope::OneWayLandlord,
            Party::Landlord,
            true,
            10_000_00,
        ));
        assert!(r.prevailing_party_entitled_to_fees);
        assert_eq!(r.amount_recoverable_cents, 10_000_00);
    }

    #[test]
    fn ca_bilateral_clause_prevailing_party_gets_fees() {
        let r = check(&input(
            Regime::CaliforniaCivCode1717,
            ClauseScope::Bilateral,
            Party::Tenant,
            true,
            5_000_00,
        ));
        // Bilateral clause — reciprocity not needed, but party still
        // entitled to fees.
        assert!(!r.reciprocity_applies);
        assert!(r.prevailing_party_entitled_to_fees);
    }

    #[test]
    fn wa_post_1977_one_way_landlord_tenant_prevails_gets_fees() {
        let r = check(&input(
            Regime::WashingtonRcw484330,
            ClauseScope::OneWayLandlord,
            Party::Tenant,
            true,
            10_000_00,
        ));
        assert!(r.reciprocity_applies);
        assert!(r.prevailing_party_entitled_to_fees);
        assert_eq!(r.amount_recoverable_cents, 10_000_00);
        assert!(r.citation.contains("RCW 4.84.330"));
        assert!(r.citation.contains("waiver PROHIBITED"));
    }

    #[test]
    fn wa_pre_1977_lease_no_reciprocity_one_way_enforced() {
        // Pre-1977 lease: WA reciprocity does NOT apply.
        // One-way clause favors landlord, but tenant prevails.
        let r = check(&input(
            Regime::WashingtonRcw484330,
            ClauseScope::OneWayLandlord,
            Party::Tenant,
            false,
            10_000_00,
        ));
        assert!(!r.reciprocity_applies);
        assert!(!r.prevailing_party_entitled_to_fees);
        assert_eq!(r.amount_recoverable_cents, 0);
        assert!(r.citation.contains("1977-09-21"));
    }

    #[test]
    fn wa_pre_1977_one_way_landlord_prevails_still_gets_fees() {
        // Pre-1977 lease: one-way clause enforced as written. Landlord
        // named, landlord prevails → fees.
        let r = check(&input(
            Regime::WashingtonRcw484330,
            ClauseScope::OneWayLandlord,
            Party::Landlord,
            false,
            10_000_00,
        ));
        // No reciprocity, but clause as-written applies.
        assert!(r.prevailing_party_entitled_to_fees);
    }

    #[test]
    fn wa_post_1977_one_way_tenant_landlord_prevails_gets_fees() {
        // Reciprocity flips a one-way-tenant clause to favor landlord
        // when landlord wins.
        let r = check(&input(
            Regime::WashingtonRcw484330,
            ClauseScope::OneWayTenant,
            Party::Landlord,
            true,
            8_000_00,
        ));
        assert!(r.reciprocity_applies);
        assert!(r.prevailing_party_entitled_to_fees);
        assert_eq!(r.amount_recoverable_cents, 8_000_00);
    }

    #[test]
    fn wa_post_1977_bilateral_compliant() {
        let r = check(&input(
            Regime::WashingtonRcw484330,
            ClauseScope::Bilateral,
            Party::Landlord,
            true,
            5_000_00,
        ));
        assert!(r.prevailing_party_entitled_to_fees);
        assert_eq!(r.amount_recoverable_cents, 5_000_00);
    }

    #[test]
    fn default_one_way_landlord_tenant_prevails_NO_fees() {
        // Default regime: one-way clause enforced as written.
        let r = check(&input(
            Regime::Default,
            ClauseScope::OneWayLandlord,
            Party::Tenant,
            true,
            10_000_00,
        ));
        assert!(!r.prevailing_party_entitled_to_fees);
        assert_eq!(r.amount_recoverable_cents, 0);
        assert!(r.note.contains("reciprocity does NOT apply"));
    }

    #[test]
    fn default_one_way_landlord_landlord_prevails_gets_fees() {
        let r = check(&input(
            Regime::Default,
            ClauseScope::OneWayLandlord,
            Party::Landlord,
            true,
            10_000_00,
        ));
        assert!(r.prevailing_party_entitled_to_fees);
        assert_eq!(r.amount_recoverable_cents, 10_000_00);
    }

    #[test]
    fn default_bilateral_prevailing_party_gets_fees() {
        let r = check(&input(
            Regime::Default,
            ClauseScope::Bilateral,
            Party::Tenant,
            true,
            5_000_00,
        ));
        assert!(r.prevailing_party_entitled_to_fees);
        assert_eq!(r.amount_recoverable_cents, 5_000_00);
    }

    #[test]
    fn state_routing_ca_wa_default() {
        assert_eq!(Regime::for_state("CA"), Regime::CaliforniaCivCode1717);
        assert_eq!(Regime::for_state("WA"), Regime::WashingtonRcw484330);
        assert_eq!(Regime::for_state("TX"), Regime::Default);
        assert_eq!(Regime::for_state("NY"), Regime::Default);
    }

    #[test]
    fn state_routing_case_insensitive() {
        assert_eq!(Regime::for_state("ca"), Regime::CaliforniaCivCode1717);
        assert_eq!(Regime::for_state("Wa"), Regime::WashingtonRcw484330);
    }

    #[test]
    fn ca_no_pre_1977_carve_out() {
        // CA § 1717 has no effective-date carve-out — applies regardless
        // of when lease was entered. (Compare WA which excludes pre-1977.)
        let r = check(&input(
            Regime::CaliforniaCivCode1717,
            ClauseScope::OneWayLandlord,
            Party::Tenant,
            false,
            10_000_00,
        ));
        assert!(r.reciprocity_applies);
        assert!(r.prevailing_party_entitled_to_fees);
    }

    #[test]
    fn wa_post_1977_reciprocity_creates_paradox_for_pre_post_split() {
        // Same one-way-landlord scenario, tenant prevails: pre-1977 →
        // no fees; post-1977 → fees. Regression-critical pin.
        let pre = check(&input(
            Regime::WashingtonRcw484330,
            ClauseScope::OneWayLandlord,
            Party::Tenant,
            false,
            10_000_00,
        ));
        let post = check(&input(
            Regime::WashingtonRcw484330,
            ClauseScope::OneWayLandlord,
            Party::Tenant,
            true,
            10_000_00,
        ));
        assert!(!pre.prevailing_party_entitled_to_fees);
        assert!(post.prevailing_party_entitled_to_fees);
    }

    #[test]
    fn only_ca_and_wa_have_reciprocity() {
        // Same one-way landlord clause, tenant prevails across all 3
        // regimes. CA + WA → tenant gets fees. Default → no.
        let ca = check(&input(
            Regime::CaliforniaCivCode1717,
            ClauseScope::OneWayLandlord,
            Party::Tenant,
            true,
            10_000_00,
        ));
        let wa = check(&input(
            Regime::WashingtonRcw484330,
            ClauseScope::OneWayLandlord,
            Party::Tenant,
            true,
            10_000_00,
        ));
        let d = check(&input(
            Regime::Default,
            ClauseScope::OneWayLandlord,
            Party::Tenant,
            true,
            10_000_00,
        ));
        assert!(ca.prevailing_party_entitled_to_fees);
        assert!(wa.prevailing_party_entitled_to_fees);
        assert!(!d.prevailing_party_entitled_to_fees);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let ca = check(&input(
            Regime::CaliforniaCivCode1717,
            ClauseScope::OneWayLandlord,
            Party::Tenant,
            true,
            5_000_00,
        ));
        assert!(ca.citation.contains("§ 1717(a)"));

        let wa = check(&input(
            Regime::WashingtonRcw484330,
            ClauseScope::OneWayLandlord,
            Party::Tenant,
            true,
            5_000_00,
        ));
        assert!(wa.citation.contains("RCW 4.84.330"));

        let d = check(&input(
            Regime::Default,
            ClauseScope::OneWayLandlord,
            Party::Tenant,
            true,
            5_000_00,
        ));
        assert!(d.citation.contains("American Rule") || d.citation.contains("reciprocity statute"));
    }

    #[test]
    fn negative_fees_clamped_to_zero() {
        let r = check(&input(
            Regime::CaliforniaCivCode1717,
            ClauseScope::Bilateral,
            Party::Tenant,
            true,
            -100,
        ));
        assert_eq!(r.amount_recoverable_cents, 0);
    }

    #[test]
    fn ca_one_way_tenant_clause_landlord_prevails_gets_fees() {
        // Symmetric case: clause favors tenant, but landlord prevails.
        // CA § 1717 reciprocity flips → landlord collects.
        let r = check(&input(
            Regime::CaliforniaCivCode1717,
            ClauseScope::OneWayTenant,
            Party::Landlord,
            true,
            7_500_00,
        ));
        assert!(r.reciprocity_applies);
        assert!(r.prevailing_party_entitled_to_fees);
        assert_eq!(r.amount_recoverable_cents, 7_500_00);
    }
}
