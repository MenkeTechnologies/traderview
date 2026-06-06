//! State tenant advance-rent prepayment limits.
//!
//! Distinct from `security_deposit_caps` (which caps refundable
//! security deposits). This module captures statutory limits on
//! how much PREPAID RENT a landlord can collect at lease signing
//! — the "first month + last month + security deposit" demand
//! common in expensive markets. Trader-tenants relocating to NYC,
//! SF, Boston, etc. routinely face 6-12 months advance demands;
//! a handful of states cap or prohibit these.
//!
//! Five regimes:
//!
//! `NewYorkOneMonthFirstOnly`: NY only. RPL § 238-a (HSTPA 2019).
//! Security deposit limited to 1 month's rent statewide for
//! UNREGULATED tenancies. **Advance rent beyond the first month
//! is PROHIBITED** — landlord cannot require last month's rent
//! or multiple-month prepayments as a condition of move-in. The
//! strictest regime in the U.S.
//!
//! `MassachusettsFirstLastSecurityLock`: MA only. G.L. c. 186
//! § 15B. Landlord may charge ONLY first month's rent, last
//! month's rent, security deposit (max 1 month), and the cost of
//! a new lock for tenant safety. No additional prepayments
//! permitted. Advance last-month rent must accrue **5% annual
//! interest** (or the lower bank-deposit rate) — distinguishing
//! MA from other regimes that don't require interest on advance
//! rent.
//!
//! `CaliforniaSixMonthLeaseOnly`: CA only. Cal. Civ. Code § 1950.5.
//! Tenants may prepay rent ONLY when (i) the lease term is at
//! least 6 months AND (ii) the prepaid amount covers at least
//! 6 months of rent. Otherwise, landlord may collect only the
//! first month's rent + security deposit. Effectively bars
//! month-to-month tenants from being pressured into multi-month
//! prepayments.
//!
//! `NewJerseyAdvanceRentUnlimited`: NJ only. N.J.S.A. 46:8-21.2
//! clarifies that advance rent prepayments are NOT security
//! deposits and are NOT subject to the 1.5-month security deposit
//! cap under N.J.S.A. 46:8-19. NJ's most landlord-favorable
//! interpretation: parties may agree to any advance rent amount.
//!
//! `NoStateAdvanceRentLimit`: 46 other states + DC. No state
//! statutory cap on advance rent; lease terms govern. Some cities
//! have local ordinances (SF, LA, Chicago) but no statewide rule.
//!
//! Sources:
//! [N.Y. RPL § 238-a (HSTPA 2019) — NYSBA](https://nysba.org/nys-housing-stability-and-tenant-protection-act-of-2019-part-iii-what-lawyers-must-know/),
//! [Mass. G.L. c. 186 § 15B — Mass.gov](https://www.mass.gov/info-details/mass-general-laws-c186-ss-15b),
//! [Cal. Civ. Code § 1950.5 — apartments.com CA Rental Laws](https://www.apartments.com/rental-manager/resources/state-laws?state=California),
//! [N.J.S.A. 46:8-21.2 — Justia](https://law.justia.com/codes/new-jersey/title-46/section-46-8-21-2/),
//! [iPropertyManagement — First & Last Months' Rent + Security Deposit by State](https://ipropertymanagement.com/laws/first-last-month-security-deposit).

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdvanceRentRegime {
    NewYorkOneMonthFirstOnly,
    MassachusettsFirstLastSecurityLock,
    CaliforniaSixMonthLeaseOnly,
    NewJerseyAdvanceRentUnlimited,
    NoStateAdvanceRentLimit,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: AdvanceRentRegime,
    /// Maximum months of advance rent the landlord may demand,
    /// including the current month. NY = 1; MA = 2 (first + last);
    /// CA = 1 default (6 only when 6-mo lease); NJ / others = no
    /// statutory cap (modeled as u32::MAX).
    pub maximum_advance_months_at_signing: u32,
    /// True if state requires interest on advance last-month rent
    /// (MA 5%).
    pub interest_on_advance_rent_required: bool,
    /// Required interest rate in basis points (MA = 500 = 5%).
    pub required_interest_rate_bp: u32,
    /// True if regime imposes a minimum lease-term threshold for
    /// multi-month prepayments (CA: ≥ 6 months).
    pub minimum_lease_months_for_multi_month_prepayment: u32,
    pub citation: &'static str,
}

const fn rule(
    regime: AdvanceRentRegime,
    maximum_advance_months_at_signing: u32,
    interest_on_advance_rent_required: bool,
    required_interest_rate_bp: u32,
    minimum_lease_months_for_multi_month_prepayment: u32,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        maximum_advance_months_at_signing,
        interest_on_advance_rent_required,
        required_interest_rate_bp,
        minimum_lease_months_for_multi_month_prepayment,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use AdvanceRentRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "NY",
        rule(
            NewYorkOneMonthFirstOnly,
            1,
            false,
            0,
            0,
            "N.Y. RPL § 238-a (HSTPA 2019) — security deposit limited to 1 month for unregulated tenancies; advance rent beyond first month PROHIBITED; landlord cannot require last month's rent or multi-month prepayments as condition of move-in",
        ),
    );

    m.insert(
        "MA",
        rule(
            MassachusettsFirstLastSecurityLock,
            2,
            true,
            500,
            0,
            "Mass. G.L. c. 186 § 15B — landlord may charge ONLY first month + last month + security deposit (max 1 month) + cost of new lock; no other advance prepayments permitted; advance last-month rent must accrue 5% annual interest (or lower bank-deposit rate)",
        ),
    );

    m.insert(
        "CA",
        rule(
            CaliforniaSixMonthLeaseOnly,
            1,
            false,
            0,
            6,
            "Cal. Civ. Code § 1950.5 — tenants may prepay rent only when (i) lease term ≥ 6 months AND (ii) prepaid amount covers ≥ 6 months; otherwise landlord may collect only first month + security deposit; bars month-to-month tenants from multi-month prepayment pressure",
        ),
    );

    m.insert(
        "NJ",
        rule(
            NewJerseyAdvanceRentUnlimited,
            u32::MAX,
            false,
            0,
            0,
            "N.J.S.A. 46:8-21.2 — advance rent prepayments NOT security deposits + NOT subject to 1.5-month security deposit cap under § 46:8-19; parties may agree to any advance rent amount",
        ),
    );

    // NoStateAdvanceRentLimit default — 46 other states + DC.
    let default_states = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DC", "DE", "FL", "GA", "HI", "ID", "IL", "IN", "IA",
        "KS", "KY", "LA", "ME", "MD", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NM", "NC",
        "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV",
        "WI", "WY",
    ];
    for code in default_states {
        m.insert(
            code,
            rule(
                NoStateAdvanceRentLimit,
                u32::MAX,
                false,
                0,
                0,
                "No state statutory cap on advance rent prepayment; lease terms govern; some cities (SF, LA, Chicago) have local ordinances",
            ),
        );
    }

    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvanceRentInput {
    pub state_code: String,
    /// Months of advance rent the landlord is demanding (NOT
    /// counting refundable security deposit).
    pub advance_rent_months_demanded: u32,
    /// True if the landlord is also demanding last month's rent
    /// separately from the advance-rent figure.
    pub landlord_demanding_last_month_rent: bool,
    /// Lease term in months (for CA 6-month gate).
    pub lease_term_months: u32,
    /// True if landlord is paying / accruing required 5% interest
    /// on advance last-month rent (MA gate).
    pub interest_accrued_on_advance_last_month: bool,
    /// True if tenancy is regulated (rent-stabilized, public
    /// housing). NY HSTPA cap applies only to unregulated.
    pub tenancy_is_unregulated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvanceRentResult {
    pub regime: AdvanceRentRegime,
    pub maximum_months_permitted: u32,
    pub landlord_compliant: bool,
    pub interest_compliance_satisfied: bool,
    pub ca_six_month_lease_gate_met: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &AdvanceRentInput) -> AdvanceRentResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: AdvanceRentRegime::NoStateAdvanceRentLimit,
        maximum_advance_months_at_signing: u32::MAX,
        interest_on_advance_rent_required: false,
        required_interest_rate_bp: 0,
        minimum_lease_months_for_multi_month_prepayment: 0,
        citation: "Unknown state code; no statewide advance-rent cap assumed",
    });

    let ca_gate_met = if rule.minimum_lease_months_for_multi_month_prepayment > 0 {
        // CA: multi-month prepayment requires lease ≥ 6 months.
        input.lease_term_months >= rule.minimum_lease_months_for_multi_month_prepayment
    } else {
        true
    };

    // NY rule applies only to unregulated tenancies.
    let ny_applies = !matches!(rule.regime, AdvanceRentRegime::NewYorkOneMonthFirstOnly)
        || input.tenancy_is_unregulated;

    // Total months claimed = advance rent + last month if landlord
    // demanding separately.
    let total_months_claimed = input.advance_rent_months_demanded
        + if input.landlord_demanding_last_month_rent {
            1
        } else {
            0
        };

    let cap_compliant = match rule.regime {
        AdvanceRentRegime::CaliforniaSixMonthLeaseOnly => {
            if total_months_claimed <= 1 {
                true
            } else {
                // Multi-month prepayment: requires 6-month lease.
                ca_gate_met && total_months_claimed >= 6
            }
        }
        _ => !ny_applies || total_months_claimed <= rule.maximum_advance_months_at_signing,
    };

    // Interest compliance (MA only).
    let interest_compliant = !rule.interest_on_advance_rent_required
        || !input.landlord_demanding_last_month_rent
        || input.interest_accrued_on_advance_last_month;

    let landlord_compliant = cap_compliant && interest_compliant;

    let regime_label = match rule.regime {
        AdvanceRentRegime::NewYorkOneMonthFirstOnly => "New York HSTPA 2019 — first month only",
        AdvanceRentRegime::MassachusettsFirstLastSecurityLock => {
            "Massachusetts G.L. c. 186 § 15B — first + last + deposit + lock + 5% interest"
        }
        AdvanceRentRegime::CaliforniaSixMonthLeaseOnly => {
            "California § 1950.5 — multi-month prepayment only with 6-month lease"
        }
        AdvanceRentRegime::NewJerseyAdvanceRentUnlimited => {
            "New Jersey § 46:8-21.2 — advance rent unlimited"
        }
        AdvanceRentRegime::NoStateAdvanceRentLimit => "no statewide advance-rent cap",
    };

    let note = if landlord_compliant {
        format!(
            "State applies {} regime; landlord compliant — {} months demanded within statutory limits.",
            regime_label, total_months_claimed,
        )
    } else {
        let mut reasons = vec![];
        if !cap_compliant {
            reasons.push(format!(
                "advance-rent cap violated ({} months demanded; cap = {} months)",
                total_months_claimed,
                if rule.maximum_advance_months_at_signing == u32::MAX {
                    "(no cap)".to_string()
                } else {
                    rule.maximum_advance_months_at_signing.to_string()
                }
            ));
        }
        if !interest_compliant {
            reasons.push("5% interest on advance last-month rent NOT accrued".to_string());
        }
        format!(
            "State applies {} regime; landlord NON-COMPLIANT: {}.",
            regime_label,
            reasons.join("; "),
        )
    };

    AdvanceRentResult {
        regime: rule.regime,
        maximum_months_permitted: rule.maximum_advance_months_at_signing,
        landlord_compliant,
        interest_compliance_satisfied: interest_compliant,
        ca_six_month_lease_gate_met: ca_gate_met,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline(state: &str) -> AdvanceRentInput {
        AdvanceRentInput {
            state_code: state.to_string(),
            advance_rent_months_demanded: 1,
            landlord_demanding_last_month_rent: false,
            lease_term_months: 12,
            interest_accrued_on_advance_last_month: true,
            tenancy_is_unregulated: true,
        }
    }

    // ── Regime classification ───────────────────────────────────────

    #[test]
    fn ny_one_month_regime() {
        let r = check(&baseline("NY"));
        assert_eq!(r.regime, AdvanceRentRegime::NewYorkOneMonthFirstOnly);
    }

    #[test]
    fn ma_first_last_security_lock_regime() {
        let r = check(&baseline("MA"));
        assert_eq!(
            r.regime,
            AdvanceRentRegime::MassachusettsFirstLastSecurityLock
        );
    }

    #[test]
    fn ca_six_month_lease_only_regime() {
        let r = check(&baseline("CA"));
        assert_eq!(r.regime, AdvanceRentRegime::CaliforniaSixMonthLeaseOnly);
    }

    #[test]
    fn nj_advance_rent_unlimited_regime() {
        let r = check(&baseline("NJ"));
        assert_eq!(r.regime, AdvanceRentRegime::NewJerseyAdvanceRentUnlimited);
    }

    #[test]
    fn default_state_no_limit_regime() {
        for s in ["AL", "FL", "TX", "WA", "DC", "WY", "IL"] {
            let r = check(&baseline(s));
            assert_eq!(
                r.regime,
                AdvanceRentRegime::NoStateAdvanceRentLimit,
                "expected {s} no-limit regime"
            );
        }
    }

    // ── NY: 1 month only ───────────────────────────────────────────

    #[test]
    fn ny_one_month_demand_compliant() {
        let r = check(&baseline("NY"));
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ny_first_plus_last_violates_cap() {
        let mut i = baseline("NY");
        i.landlord_demanding_last_month_rent = true;
        let r = check(&i);
        assert!(
            !r.landlord_compliant,
            "NY HSTPA prohibits last-month demand"
        );
    }

    #[test]
    fn ny_three_months_advance_violates_cap() {
        let mut i = baseline("NY");
        i.advance_rent_months_demanded = 3;
        let r = check(&i);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn ny_regulated_tenancy_does_not_apply_hstpa_cap() {
        // HSTPA cap applies only to unregulated tenancies.
        let mut i = baseline("NY");
        i.tenancy_is_unregulated = false;
        i.advance_rent_months_demanded = 3;
        let r = check(&i);
        assert!(
            r.landlord_compliant,
            "HSTPA caps unregulated tenancies only"
        );
    }

    // ── MA: first + last + 5% interest ─────────────────────────────

    #[test]
    fn ma_first_only_compliant() {
        let r = check(&baseline("MA"));
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ma_first_plus_last_with_interest_compliant() {
        let mut i = baseline("MA");
        i.landlord_demanding_last_month_rent = true;
        i.interest_accrued_on_advance_last_month = true;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ma_first_plus_last_without_interest_non_compliant() {
        let mut i = baseline("MA");
        i.landlord_demanding_last_month_rent = true;
        i.interest_accrued_on_advance_last_month = false;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert!(!r.interest_compliance_satisfied);
        assert!(r.note.contains("5% interest"));
    }

    #[test]
    fn ma_three_months_advance_violates_cap() {
        let mut i = baseline("MA");
        i.advance_rent_months_demanded = 3;
        let r = check(&i);
        assert!(!r.landlord_compliant);
    }

    // ── CA: 6-month lease gate ─────────────────────────────────────

    #[test]
    fn ca_one_month_demand_any_lease_term_compliant() {
        let mut i = baseline("CA");
        i.lease_term_months = 1;
        i.advance_rent_months_demanded = 1;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ca_6_month_advance_with_6_month_lease_compliant() {
        let mut i = baseline("CA");
        i.lease_term_months = 6;
        i.advance_rent_months_demanded = 6;
        let r = check(&i);
        assert!(r.ca_six_month_lease_gate_met);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ca_6_month_advance_with_5_month_lease_non_compliant() {
        let mut i = baseline("CA");
        i.lease_term_months = 5;
        i.advance_rent_months_demanded = 6;
        let r = check(&i);
        assert!(!r.ca_six_month_lease_gate_met);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn ca_3_month_advance_even_with_long_lease_non_compliant() {
        // 3 months is multi-month but below the 6-month threshold.
        let mut i = baseline("CA");
        i.lease_term_months = 12;
        i.advance_rent_months_demanded = 3;
        let r = check(&i);
        assert!(!r.landlord_compliant);
    }

    // ── NJ: unlimited ──────────────────────────────────────────────

    #[test]
    fn nj_unlimited_no_cap_violation() {
        let mut i = baseline("NJ");
        i.advance_rent_months_demanded = 12;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    // ── Default state ──────────────────────────────────────────────

    #[test]
    fn default_state_any_amount_compliant() {
        let mut i = baseline("TX");
        i.advance_rent_months_demanded = 12;
        i.landlord_demanding_last_month_rent = true;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    // ── Citation contents ──────────────────────────────────────────

    #[test]
    fn ny_citation_mentions_238_a_and_hstpa() {
        let r = check(&baseline("NY"));
        assert!(r.citation.contains("§ 238-a"));
        assert!(r.citation.contains("HSTPA 2019"));
        assert!(r.citation.contains("PROHIBITED"));
    }

    #[test]
    fn ma_citation_mentions_c_186_15b_and_5_pct_interest() {
        let r = check(&baseline("MA"));
        assert!(r.citation.contains("c. 186 § 15B"));
        assert!(r.citation.contains("5% annual interest"));
    }

    #[test]
    fn ca_citation_mentions_1950_5_and_6_months() {
        let r = check(&baseline("CA"));
        assert!(r.citation.contains("§ 1950.5"));
        assert!(r.citation.contains("≥ 6 months"));
    }

    #[test]
    fn nj_citation_mentions_46_8_21_2() {
        let r = check(&baseline("NJ"));
        assert!(r.citation.contains("46:8-21.2"));
    }

    // ── Coverage / single-state-uniqueness ─────────────────────────

    #[test]
    fn coverage_is_all_50_states_plus_dc() {
        assert_eq!(RULES.len(), 51);
    }

    #[test]
    fn citations_never_empty() {
        for (code, rule) in RULES.iter() {
            assert!(!rule.citation.is_empty(), "{code} empty citation");
        }
    }

    #[test]
    fn ny_only_one_month_regime_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| matches!(r.regime, AdvanceRentRegime::NewYorkOneMonthFirstOnly))
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn ma_only_interest_required_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| r.interest_on_advance_rent_required)
            .count();
        assert_eq!(count, 1, "only MA requires 5% interest");
    }

    #[test]
    fn ca_only_six_month_gate_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| r.minimum_lease_months_for_multi_month_prepayment > 0)
            .count();
        assert_eq!(count, 1, "only CA has the 6-month lease gate");
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn ny_violation_note_mentions_cap_violation() {
        let mut i = baseline("NY");
        i.advance_rent_months_demanded = 2;
        let r = check(&i);
        assert!(r.note.contains("advance-rent cap violated"));
    }

    #[test]
    fn ma_interest_violation_note_mentions_5_pct() {
        let mut i = baseline("MA");
        i.landlord_demanding_last_month_rent = true;
        i.interest_accrued_on_advance_last_month = false;
        let r = check(&i);
        assert!(r.note.contains("5% interest"));
    }

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&baseline("ny"));
        assert_eq!(r.regime, AdvanceRentRegime::NewYorkOneMonthFirstOnly);
    }
}
