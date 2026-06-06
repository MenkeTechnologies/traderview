//! State landlord duty to provide tenant a signed copy of the lease.
//!
//! Distinct from `owner_identification` (which is about disclosing
//! the OWNER's identity and agent for service). This module covers
//! the obligation to physically deliver the executed lease document
//! itself to the tenant within a statutorily specified window after
//! signing. Knowing what's in the lease is foundational — a tenant
//! who has never received a copy cannot enforce the terms they
//! signed up for and cannot easily defend against eviction
//! proceedings based on lease provisions.
//!
//! Four regimes:
//!
//! `California15DayDelivery`: CA only. Cal. Civ. Code § 1962(a)(1)
//! — landlord must provide a copy of the rental agreement or lease
//! to the tenant within 15 days of execution. On annual request the
//! landlord must provide additional copies within 15 days of each
//! request.
//!
//! `Massachusetts30DayWith300DollarFine`: MA only. G.L. c. 186
//! § 15D — when a lessor agreed orally to execute a lease and
//! obtains the lessee's signature, the lessor must deliver a copy
//! duly signed and executed within 30 days. Violation = $300 fine.
//! Any waiver in any lease or rental agreement is **void and
//! unenforceable**.
//!
//! `Texas3BusinessDayDelivery`: TX only. Tex. Prop. Code § 92.024 —
//! not later than the 3rd business day after the date the lease is
//! signed by each party, the landlord must provide at least one
//! complete copy of the lease to at least one tenant who is a
//! party. If more than one tenant is a party, additional copies
//! must be provided within 3 business days of a written request.
//!
//! `NoStateLeaseCopyDeliveryDeadline`: 47 other states + DC. No
//! state-specific lease-copy-delivery deadline confirmed. The
//! lease is still enforceable but the tenant may be limited to
//! common-law remedies (specific performance / declaratory
//! judgment) if denied a copy. Many local landlord-tenant
//! ordinances impose similar duties (e.g., Chicago RLTO).
//!
//! Sources:
//! [Cal. Civ. Code § 1962 — California Legislative Information](https://leginfo.legislature.ca.gov/faces/codes_displaySection.xhtml?lawCode=CIV&sectionNum=1962),
//! [Mass. G.L. c. 186 § 15D — Mass. General Laws](https://malegislature.gov/Laws/GeneralLaws/PartII/TitleI/Chapter186/Section15D),
//! [Tex. Prop. Code § 92.024 — FindLaw](https://codes.findlaw.com/tx/property-code/prop-sect-92-024/),
//! [Tex. Prop. Code § 92.024 — Texas Public Law](https://texas.public.law/statutes/tex._prop._code_section_92.024).

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseCopyDeliveryRegime {
    California15DayDelivery,
    Massachusetts30DayWith300DollarFine,
    Texas3BusinessDayDelivery,
    NoStateLeaseCopyDeliveryDeadline,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: LeaseCopyDeliveryRegime,
    /// Days the landlord must deliver the copy by, measured from
    /// lease execution date. Zero for the no-statute regime.
    pub delivery_deadline_days: u32,
    /// True if the deadline is measured in business days (TX) vs
    /// calendar days (CA, MA).
    pub days_are_business_days: bool,
    /// Statutory fine for noncompliance in dollars. Zero for
    /// regimes without a fixed-dollar penalty.
    pub statutory_fine_dollars: i64,
    /// True if any contractual waiver of the delivery duty is
    /// statutorily declared void and unenforceable (MA).
    pub waiver_prohibited: bool,
    pub citation: &'static str,
}

const fn rule(
    regime: LeaseCopyDeliveryRegime,
    delivery_deadline_days: u32,
    days_are_business_days: bool,
    statutory_fine_dollars: i64,
    waiver_prohibited: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        delivery_deadline_days,
        days_are_business_days,
        statutory_fine_dollars,
        waiver_prohibited,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use LeaseCopyDeliveryRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "CA",
        rule(
            California15DayDelivery,
            15,
            false,
            0,
            false,
            "Cal. Civ. Code § 1962(a)(1) — landlord must provide copy of rental agreement or lease to tenant within 15 days of execution; annual request entitles tenant to additional copy within 15 days of each request",
        ),
    );

    m.insert(
        "MA",
        rule(
            Massachusetts30DayWith300DollarFine,
            30,
            false,
            300,
            true,
            "Mass. G.L. c. 186 § 15D — lessor must deliver duly signed and executed lease copy to lessee within 30 days after obtaining lessee signature; violation = $300 fine; any waiver in any lease or rental agreement is void and unenforceable",
        ),
    );

    m.insert(
        "TX",
        rule(
            Texas3BusinessDayDelivery,
            3,
            true,
            0,
            false,
            "Tex. Prop. Code § 92.024 — landlord must provide at least one complete copy of the lease to at least one tenant party within 3 business days of all parties signing; additional copies to other tenant parties within 3 business days of written request",
        ),
    );

    // NoStateLeaseCopyDeliveryDeadline default — 47 other states + DC.
    let no_state = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DC", "DE", "FL", "GA", "HI", "ID", "IL", "IN", "IA",
        "KS", "KY", "LA", "ME", "MD", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ", "NM",
        "NY", "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC", "SD", "TN", "UT", "VT", "VA", "WA",
        "WV", "WI", "WY",
    ];
    for code in no_state {
        m.insert(
            code,
            rule(
                NoStateLeaseCopyDeliveryDeadline,
                0,
                false,
                0,
                false,
                "No state-specific lease-copy-delivery deadline confirmed; lease remains enforceable but tenant may rely on common-law remedies (specific performance / declaratory judgment) if denied a copy; many local ordinances impose similar duties (e.g., Chicago RLTO)",
            ),
        );
    }

    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseCopyDeliveryInput {
    pub state_code: String,
    /// Days elapsed since the lease was fully executed by all
    /// parties.
    pub days_since_lease_execution: u32,
    /// Business days elapsed since the lease was fully executed
    /// (for TX which measures in business days).
    pub business_days_since_lease_execution: u32,
    /// True if the landlord has delivered a complete copy of the
    /// executed lease to the tenant.
    pub copy_delivered: bool,
    /// True if the lease contains a contractual waiver of the
    /// landlord's copy-delivery duty (irrelevant in MA where any
    /// such waiver is statutorily void).
    pub lease_contains_waiver_of_delivery: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaseCopyDeliveryResult {
    pub regime: LeaseCopyDeliveryRegime,
    pub statutory_deadline_days: u32,
    pub deadline_measured_in_business_days: bool,
    pub landlord_compliant: bool,
    /// True if the statute imposes a fine for noncompliance.
    pub statutory_fine_available: bool,
    pub statutory_fine_amount_dollars: i64,
    /// True if the lease contains a waiver but the state's statute
    /// declares the waiver void.
    pub contractual_waiver_void: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &LeaseCopyDeliveryInput) -> LeaseCopyDeliveryResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: LeaseCopyDeliveryRegime::NoStateLeaseCopyDeliveryDeadline,
        delivery_deadline_days: 0,
        days_are_business_days: false,
        statutory_fine_dollars: 0,
        waiver_prohibited: false,
        citation: "Unknown state code; no statewide lease-copy-delivery deadline assumed",
    });

    let waiver_void = input.lease_contains_waiver_of_delivery && rule.waiver_prohibited;

    let deadline_passed = if rule.days_are_business_days {
        input.business_days_since_lease_execution > rule.delivery_deadline_days
    } else {
        input.days_since_lease_execution > rule.delivery_deadline_days
    };

    // Compliance:
    // - delivered: always compliant
    // - not delivered + deadline not yet passed: compliant (still within window)
    // - not delivered + deadline passed: non-compliant
    // - no-state regime: always compliant (no deadline)
    let compliant = input.copy_delivered
        || !deadline_passed
        || matches!(
            rule.regime,
            LeaseCopyDeliveryRegime::NoStateLeaseCopyDeliveryDeadline
        );

    let fine_available = !compliant && rule.statutory_fine_dollars > 0;
    let fine_amount = if fine_available {
        rule.statutory_fine_dollars
    } else {
        0
    };

    let regime_label = match rule.regime {
        LeaseCopyDeliveryRegime::California15DayDelivery => "California 15-day delivery",
        LeaseCopyDeliveryRegime::Massachusetts30DayWith300DollarFine => {
            "Massachusetts 30-day delivery with $300 fine"
        }
        LeaseCopyDeliveryRegime::Texas3BusinessDayDelivery => "Texas 3-business-day delivery",
        LeaseCopyDeliveryRegime::NoStateLeaseCopyDeliveryDeadline => "no state-specific deadline",
    };

    let note = if matches!(
        rule.regime,
        LeaseCopyDeliveryRegime::NoStateLeaseCopyDeliveryDeadline
    ) {
        format!(
            "State applies {} regime; landlord copy-delivery duty not codified statewide; common-law remedies available if copy is denied.",
            regime_label,
        )
    } else if compliant {
        format!(
            "State applies {} regime; landlord compliant on these facts ({} {} since execution; {} delivered).",
            regime_label,
            if rule.days_are_business_days {
                input.business_days_since_lease_execution
            } else {
                input.days_since_lease_execution
            },
            if rule.days_are_business_days { "business days" } else { "days" },
            if input.copy_delivered { "copy" } else { "copy NOT yet" },
        )
    } else {
        let mut parts = vec![format!(
            "State applies {} regime; landlord NON-COMPLIANT — copy not delivered within {} {}",
            regime_label,
            rule.delivery_deadline_days,
            if rule.days_are_business_days {
                "business days"
            } else {
                "days"
            },
        )];
        if fine_available {
            parts.push(format!("${} statutory fine available", fine_amount));
        }
        if waiver_void {
            parts.push("contractual waiver of delivery duty is VOID and unenforceable".to_string());
        }
        format!("{}.", parts.join("; "))
    };

    LeaseCopyDeliveryResult {
        regime: rule.regime,
        statutory_deadline_days: rule.delivery_deadline_days,
        deadline_measured_in_business_days: rule.days_are_business_days,
        landlord_compliant: compliant,
        statutory_fine_available: fine_available,
        statutory_fine_amount_dollars: fine_amount,
        contractual_waiver_void: waiver_void,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: &str) -> LeaseCopyDeliveryInput {
        LeaseCopyDeliveryInput {
            state_code: state.to_string(),
            days_since_lease_execution: 20,
            business_days_since_lease_execution: 5,
            copy_delivered: false,
            lease_contains_waiver_of_delivery: false,
        }
    }

    // ── Regime classification ───────────────────────────────────────

    #[test]
    fn ca_15_day_regime() {
        let r = check(&input("CA"));
        assert_eq!(r.regime, LeaseCopyDeliveryRegime::California15DayDelivery);
        assert_eq!(r.statutory_deadline_days, 15);
        assert!(!r.deadline_measured_in_business_days);
    }

    #[test]
    fn ma_30_day_300_fine_regime() {
        let r = check(&input("MA"));
        assert_eq!(
            r.regime,
            LeaseCopyDeliveryRegime::Massachusetts30DayWith300DollarFine
        );
        assert_eq!(r.statutory_deadline_days, 30);
    }

    #[test]
    fn tx_3_business_day_regime() {
        let r = check(&input("TX"));
        assert_eq!(r.regime, LeaseCopyDeliveryRegime::Texas3BusinessDayDelivery);
        assert_eq!(r.statutory_deadline_days, 3);
        assert!(r.deadline_measured_in_business_days);
    }

    #[test]
    fn default_state_no_deadline_regime() {
        for s in ["AL", "FL", "NY", "WA", "DC", "WY", "NJ", "IL"] {
            let r = check(&input(s));
            assert_eq!(
                r.regime,
                LeaseCopyDeliveryRegime::NoStateLeaseCopyDeliveryDeadline,
                "expected {s} no-state regime"
            );
        }
    }

    // ── CA 15-day boundary ──────────────────────────────────────────

    #[test]
    fn ca_15_days_not_delivered_still_within_window() {
        let mut i = input("CA");
        i.days_since_lease_execution = 15;
        let r = check(&i);
        assert!(
            r.landlord_compliant,
            "day 15 still within window since deadline test is strict-greater"
        );
    }

    #[test]
    fn ca_16_days_not_delivered_non_compliant() {
        let mut i = input("CA");
        i.days_since_lease_execution = 16;
        let r = check(&i);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn ca_delivered_within_window_compliant() {
        let mut i = input("CA");
        i.days_since_lease_execution = 10;
        i.copy_delivered = true;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ca_delivered_after_window_still_compliant_per_facts() {
        // If the copy has now been delivered, the landlord is in
        // compliance — fine isn't owed for the lateness since the
        // duty has been satisfied (statute itself doesn't impose
        // a per-day penalty in CA).
        let mut i = input("CA");
        i.days_since_lease_execution = 90;
        i.copy_delivered = true;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    // ── MA 30-day boundary + $300 fine + waiver-void ──────────────

    #[test]
    fn ma_30_days_not_delivered_still_within_window() {
        let mut i = input("MA");
        i.days_since_lease_execution = 30;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ma_31_days_not_delivered_non_compliant_with_300_fine() {
        let mut i = input("MA");
        i.days_since_lease_execution = 31;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert!(r.statutory_fine_available);
        assert_eq!(r.statutory_fine_amount_dollars, 300);
    }

    #[test]
    fn ma_waiver_in_lease_is_void() {
        let mut i = input("MA");
        i.days_since_lease_execution = 100;
        i.lease_contains_waiver_of_delivery = true;
        let r = check(&i);
        assert!(r.contractual_waiver_void);
        assert!(r.note.contains("VOID"));
    }

    #[test]
    fn ma_waiver_in_lease_does_not_excuse_noncompliance() {
        // The waiver attempt itself doesn't save the landlord —
        // they remain non-compliant.
        let mut i = input("MA");
        i.days_since_lease_execution = 100;
        i.lease_contains_waiver_of_delivery = true;
        let r = check(&i);
        assert!(!r.landlord_compliant);
    }

    // ── TX 3-business-day boundary ─────────────────────────────────

    #[test]
    fn tx_3_business_days_not_delivered_still_within_window() {
        let mut i = input("TX");
        i.business_days_since_lease_execution = 3;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn tx_4_business_days_not_delivered_non_compliant() {
        let mut i = input("TX");
        i.business_days_since_lease_execution = 4;
        let r = check(&i);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn tx_no_statutory_fine_dollar_amount() {
        let mut i = input("TX");
        i.business_days_since_lease_execution = 30;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        // TX § 92.024 doesn't impose a fixed-dollar fine.
        assert!(!r.statutory_fine_available);
        assert_eq!(r.statutory_fine_amount_dollars, 0);
    }

    // ── Default state ──────────────────────────────────────────────

    #[test]
    fn default_state_always_compliant_no_deadline() {
        let mut i = input("NY");
        i.days_since_lease_execution = 365;
        i.copy_delivered = false;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn default_state_waiver_not_void() {
        let mut i = input("NY");
        i.lease_contains_waiver_of_delivery = true;
        let r = check(&i);
        assert!(!r.contractual_waiver_void);
    }

    // ── Citations ──────────────────────────────────────────────────

    #[test]
    fn ca_citation_mentions_1962_a_1_and_15_days() {
        let r = check(&input("CA"));
        assert!(r.citation.contains("§ 1962(a)(1)"));
        assert!(r.citation.contains("15 days"));
    }

    #[test]
    fn ma_citation_mentions_186_15d_300_fine_and_void_waiver() {
        let r = check(&input("MA"));
        assert!(r.citation.contains("c. 186 § 15D"));
        assert!(r.citation.contains("$300"));
        assert!(r.citation.contains("void and unenforceable"));
    }

    #[test]
    fn tx_citation_mentions_92_024_and_3_business_days() {
        let r = check(&input("TX"));
        assert!(r.citation.contains("§ 92.024"));
        assert!(r.citation.contains("3 business days"));
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
    fn ca_only_15_day_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| matches!(r.regime, LeaseCopyDeliveryRegime::California15DayDelivery))
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn ma_only_30_day_fine_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    LeaseCopyDeliveryRegime::Massachusetts30DayWith300DollarFine
                )
            })
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn tx_only_business_day_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| matches!(r.regime, LeaseCopyDeliveryRegime::Texas3BusinessDayDelivery))
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn ma_only_waiver_prohibited_state() {
        let count = RULES.iter().filter(|(_, r)| r.waiver_prohibited).count();
        assert_eq!(count, 1, "only MA prohibits contractual waiver");
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn ma_non_compliant_note_mentions_300_fine() {
        let mut i = input("MA");
        i.days_since_lease_execution = 60;
        let r = check(&i);
        assert!(r.note.contains("$300 statutory fine"));
    }

    #[test]
    fn ca_compliant_note_mentions_regime() {
        let mut i = input("CA");
        i.copy_delivered = true;
        let r = check(&i);
        assert!(r.note.contains("California 15-day delivery"));
    }

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&input("ma"));
        assert_eq!(
            r.regime,
            LeaseCopyDeliveryRegime::Massachusetts30DayWith300DollarFine
        );
    }
}
