//! State tenant late-payment grace period compliance.
//!
//! Distinct from `late_fee_caps` (which caps the dollar amount of
//! a late fee) — this module captures the **window before any late
//! fee can attach at all**. Six distinct statutory regimes:
//!
//! `MassachusettsLongGracePeriod`: MA only. G.L. c. 186 § 15B(1)(c)
//! — landlord cannot impose any late charge until rent has been due
//! and unpaid for THIRTY FULL DAYS. Most generous tenant-protective
//! grace period in the U.S. by a wide margin. Massachusetts courts
//! generally find late fees of 4–5% of monthly rent reasonable.
//!
//! `ConnecticutNineDayGracePeriod`: CT only. Conn. Gen. Stat.
//! § 47a-15a — nine-day grace period for monthly tenancies before
//! late fee may attach.
//!
//! `StandardFiveDayGracePeriod`: NY, NC, WA, VA. Most common
//! statutory regime — late fee may not attach until rent is at
//! least 5 days late:
//!   - NY RPL § 238-a (HSTPA 2019): 5-day grace; late fee capped
//!     at $50 or 5% of monthly rent, whichever is less.
//!   - NC G.S. § 42-46: 5-day grace; late fee capped at $15 or 5%
//!     of monthly rent, whichever is greater.
//!   - WA RCW 59.18.170: 5-day grace; no late fee until rent more
//!     than 5 days past due.
//!   - VA Code § 55.1-1204: 5-day grace; 10% late fee cap.
//!
//! `OregonFourDayGracePeriod`: OR only. ORS 90.260 — landlord may
//! impose late charge only if rent not received by the FOURTH day
//! of the rental period AND late charge terms are in writing.
//!
//! `TexasShortGracePeriod`: TX only. Tex. Prop. Code § 92.019 —
//! late fee may not be collected unless rent has remained unpaid
//! TWO FULL DAYS after the due date AND notice of the fee is
//! included in a written lease. § 92.019(b) reasonable-fee safe
//! harbor: 12% for buildings ≤ 4 units; 10% for > 4 units.
//!
//! `NoStatutoryGracePeriodReasonablenessOnly`: 44 other states +
//! DC. No statutory minimum grace period; late fee enforceable
//! only if it is "reasonable" and specifically disclosed in the
//! written lease (general contract / unconscionability doctrine).
//!
//! Sources:
//! [N.Y. RPL § 238-a (HSTPA 2019)](https://nysba.org/nys-housing-stability-and-tenant-protection-act-of-2019-part-iii-what-lawyers-must-know/),
//! [Tex. Prop. Code § 92.019 (FindLaw)](https://codes.findlaw.com/tx/property-code/prop-sect-92-019/),
//! [N.C. G.S. § 42-46](https://www.ncleg.net/enactedlegislation/statutes/html/bysection/chapter_42/gs_42-46.html),
//! [Mass. G.L. c. 186 § 15B — Landager guide](https://landager.com/en/property-compliance/usa/massachusetts/late-fees),
//! [ORS Chapter 90 — Oregon Legislature](https://www.oregonlegislature.gov/bills_laws/ors/ors090.html),
//! [WA RCW 59.18.170](https://wa-law.org/bill/2021-22/hb/2023/1/rcw/59_landlord_and_tenant/59.18_residential_landlord-tenant_act.html).

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GracePeriodRegime {
    MassachusettsLongGracePeriod,
    ConnecticutNineDayGracePeriod,
    StandardFiveDayGracePeriod,
    OregonFourDayGracePeriod,
    TexasShortGracePeriod,
    NoStatutoryGracePeriodReasonablenessOnly,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: GracePeriodRegime,
    /// Minimum days the landlord must wait after rent due date
    /// before any late fee can attach. Zero for the no-statute
    /// regime (lease-specified reasonable fees only).
    pub grace_period_days: u32,
    /// True if the regime requires the late-fee terms to appear in
    /// a written lease as a precondition to enforcement.
    pub requires_written_lease_disclosure: bool,
    pub citation: &'static str,
}

const fn rule(
    regime: GracePeriodRegime,
    grace_period_days: u32,
    requires_written_lease_disclosure: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        grace_period_days,
        requires_written_lease_disclosure,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use GracePeriodRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "MA",
        rule(
            MassachusettsLongGracePeriod,
            30,
            false,
            "Mass. G.L. c. 186 § 15B(1)(c) — landlord may not impose late charge until rent has been due and unpaid 30 full days; courts find 4-5% of monthly rent reasonable",
        ),
    );

    m.insert(
        "CT",
        rule(
            ConnecticutNineDayGracePeriod,
            9,
            false,
            "Conn. Gen. Stat. § 47a-15a — nine-day grace period for monthly tenancies before late fee may attach",
        ),
    );

    // StandardFiveDayGracePeriod — 4 states.
    m.insert(
        "NY",
        rule(
            StandardFiveDayGracePeriod,
            5,
            true,
            "N.Y. RPL § 238-a (HSTPA 2019) — 5-day grace period; late fee capped at $50 or 5% of monthly rent, whichever is less; must be disclosed in lease",
        ),
    );
    m.insert(
        "NC",
        rule(
            StandardFiveDayGracePeriod,
            5,
            false,
            "N.C. G.S. § 42-46 — 5-day grace period (first day being day after rent was due); late fee capped at $15 or 5% of monthly rent, whichever greater",
        ),
    );
    m.insert(
        "WA",
        rule(
            StandardFiveDayGracePeriod,
            5,
            false,
            "RCW 59.18.170 (eff. 2020) — landlord may not charge late fee until rent is more than 5 days past due",
        ),
    );
    m.insert(
        "VA",
        rule(
            StandardFiveDayGracePeriod,
            5,
            true,
            "Va. Code § 55.1-1204 — 5-day grace period; late fee capped at 10% of monthly rent; must be in written lease",
        ),
    );

    m.insert(
        "OR",
        rule(
            OregonFourDayGracePeriod,
            4,
            true,
            "ORS 90.260 — landlord may impose late charge only if rent not received by the fourth day of the rental period AND late charge terms in written rental agreement",
        ),
    );

    m.insert(
        "TX",
        rule(
            TexasShortGracePeriod,
            2,
            true,
            "Tex. Prop. Code § 92.019 — late fee may not be collected unless rent unpaid 2 full days past due AND notice of fee in written lease; § 92.019(b) reasonable-fee safe harbor: 12% for buildings ≤ 4 units, 10% for > 4 units",
        ),
    );

    // NoStatutoryGracePeriodReasonablenessOnly default — 44 other
    // states + DC.
    let no_state = [
        "AL", "AK", "AZ", "AR", "CA", "CO", "DC", "DE", "FL", "GA", "HI", "ID", "IL", "IN", "IA",
        "KS", "KY", "LA", "ME", "MD", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ", "NM",
        "ND", "OH", "OK", "PA", "RI", "SC", "SD", "TN", "UT", "VT", "WV", "WI", "WY",
    ];
    for code in no_state {
        m.insert(
            code,
            rule(
                NoStatutoryGracePeriodReasonablenessOnly,
                0,
                false,
                "No statutory minimum grace period; late fee enforceable only if reasonable and disclosed in written lease (general contract / unconscionability doctrine)",
            ),
        );
    }

    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GracePeriodInput {
    pub state_code: String,
    /// Days since rent due date (0 = same day as due; 1 = one day
    /// late; etc.).
    pub days_past_due: u32,
    /// Late fee amount the landlord is attempting to charge.
    pub late_fee_charged_dollars: i64,
    /// True if the late fee terms are disclosed in a written lease.
    pub late_fee_in_written_lease: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GracePeriodResult {
    pub regime: GracePeriodRegime,
    /// Statutory minimum days that must pass before any late fee
    /// can attach in this state.
    pub minimum_grace_period_days: u32,
    /// True if the state's grace-period requirement has been met
    /// on these facts.
    pub grace_period_satisfied: bool,
    /// True if a written-lease disclosure of late-fee terms is
    /// required by this state's statute.
    pub written_lease_disclosure_required: bool,
    /// True if the landlord is compliant with both grace-period
    /// timing AND lease-disclosure requirements.
    pub landlord_compliant: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &GracePeriodInput) -> GracePeriodResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: GracePeriodRegime::NoStatutoryGracePeriodReasonablenessOnly,
        grace_period_days: 0,
        requires_written_lease_disclosure: false,
        citation: "Unknown state code; no statutory minimum grace period assumed",
    });

    let grace_satisfied = input.days_past_due > rule.grace_period_days
        || matches!(
            rule.regime,
            GracePeriodRegime::NoStatutoryGracePeriodReasonablenessOnly
        );

    let lease_disclosure_satisfied =
        !rule.requires_written_lease_disclosure || input.late_fee_in_written_lease;

    let attempting_to_charge = input.late_fee_charged_dollars > 0;
    let landlord_compliant =
        !attempting_to_charge || (grace_satisfied && lease_disclosure_satisfied);

    let regime_label = match rule.regime {
        GracePeriodRegime::MassachusettsLongGracePeriod => "Massachusetts 30-day long grace period",
        GracePeriodRegime::ConnecticutNineDayGracePeriod => "Connecticut 9-day grace period",
        GracePeriodRegime::StandardFiveDayGracePeriod => "standard 5-day grace period",
        GracePeriodRegime::OregonFourDayGracePeriod => "Oregon 4-day grace period",
        GracePeriodRegime::TexasShortGracePeriod => "Texas 2-day short grace period",
        GracePeriodRegime::NoStatutoryGracePeriodReasonablenessOnly => {
            "no statutory grace period — lease reasonableness only"
        }
    };

    let note = if !attempting_to_charge {
        format!(
            "State applies {} regime; no late fee being charged on these facts so compliance question dormant.",
            regime_label,
        )
    } else if landlord_compliant {
        format!(
            "State applies {} regime; rent is {} days past due (≥ {}+1 required); landlord compliant.",
            regime_label, input.days_past_due, rule.grace_period_days,
        )
    } else if !grace_satisfied {
        format!(
            "State applies {} regime; rent only {} days past due — grace period of {} days NOT satisfied; landlord NON-COMPLIANT.",
            regime_label, input.days_past_due, rule.grace_period_days,
        )
    } else {
        format!(
            "State applies {} regime; grace period met but late fee NOT disclosed in written lease as required; landlord NON-COMPLIANT.",
            regime_label,
        )
    };

    GracePeriodResult {
        regime: rule.regime,
        minimum_grace_period_days: rule.grace_period_days,
        grace_period_satisfied: grace_satisfied,
        written_lease_disclosure_required: rule.requires_written_lease_disclosure,
        landlord_compliant,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(state: &str) -> GracePeriodInput {
        GracePeriodInput {
            state_code: state.to_string(),
            days_past_due: 10,
            late_fee_charged_dollars: 50,
            late_fee_in_written_lease: true,
        }
    }

    // ── Regime classification ───────────────────────────────────────

    #[test]
    fn ma_long_grace_regime() {
        let r = check(&input("MA"));
        assert_eq!(r.regime, GracePeriodRegime::MassachusettsLongGracePeriod);
        assert_eq!(r.minimum_grace_period_days, 30);
    }

    #[test]
    fn ct_nine_day_regime() {
        let r = check(&input("CT"));
        assert_eq!(r.regime, GracePeriodRegime::ConnecticutNineDayGracePeriod);
        assert_eq!(r.minimum_grace_period_days, 9);
    }

    #[test]
    fn ny_nc_wa_va_standard_five_day_regime() {
        for s in ["NY", "NC", "WA", "VA"] {
            let r = check(&input(s));
            assert_eq!(
                r.regime,
                GracePeriodRegime::StandardFiveDayGracePeriod,
                "expected {s} StandardFiveDay"
            );
            assert_eq!(r.minimum_grace_period_days, 5);
        }
    }

    #[test]
    fn or_four_day_regime() {
        let r = check(&input("OR"));
        assert_eq!(r.regime, GracePeriodRegime::OregonFourDayGracePeriod);
        assert_eq!(r.minimum_grace_period_days, 4);
    }

    #[test]
    fn tx_short_two_day_regime() {
        let r = check(&input("TX"));
        assert_eq!(r.regime, GracePeriodRegime::TexasShortGracePeriod);
        assert_eq!(r.minimum_grace_period_days, 2);
    }

    #[test]
    fn default_state_no_statute_regime() {
        for s in ["AL", "FL", "CA", "IL", "GA", "DC", "NJ", "WY"] {
            let r = check(&input(s));
            assert_eq!(
                r.regime,
                GracePeriodRegime::NoStatutoryGracePeriodReasonablenessOnly,
                "expected {s} no-statute regime"
            );
            assert_eq!(r.minimum_grace_period_days, 0);
        }
    }

    // ── MA 30-day boundary ──────────────────────────────────────────

    #[test]
    fn ma_30_days_past_due_does_not_satisfy_grace() {
        // Statute requires "30 full days" → must be > 30 to attach.
        let mut i = input("MA");
        i.days_past_due = 30;
        let r = check(&i);
        assert!(!r.grace_period_satisfied);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn ma_31_days_past_due_satisfies_grace() {
        let mut i = input("MA");
        i.days_past_due = 31;
        let r = check(&i);
        assert!(r.grace_period_satisfied);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ma_short_lateness_non_compliant() {
        let mut i = input("MA");
        i.days_past_due = 10;
        let r = check(&i);
        assert!(!r.landlord_compliant, "10 days < 30-day MA grace period");
    }

    // ── NY 5-day boundary + lease-disclosure requirement ──────────

    #[test]
    fn ny_5_days_does_not_satisfy_grace() {
        let mut i = input("NY");
        i.days_past_due = 5;
        let r = check(&i);
        assert!(!r.grace_period_satisfied);
    }

    #[test]
    fn ny_6_days_satisfies_grace() {
        let mut i = input("NY");
        i.days_past_due = 6;
        let r = check(&i);
        assert!(r.grace_period_satisfied);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ny_late_fee_without_written_lease_disclosure_non_compliant() {
        let mut i = input("NY");
        i.days_past_due = 6;
        i.late_fee_in_written_lease = false;
        let r = check(&i);
        assert!(r.grace_period_satisfied);
        assert!(
            !r.landlord_compliant,
            "NY requires written-lease disclosure for late fee enforceability"
        );
    }

    // ── TX 2-day boundary + written-lease disclosure ──────────────

    #[test]
    fn tx_2_days_does_not_satisfy() {
        let mut i = input("TX");
        i.days_past_due = 2;
        let r = check(&i);
        assert!(!r.grace_period_satisfied);
    }

    #[test]
    fn tx_3_days_satisfies() {
        let mut i = input("TX");
        i.days_past_due = 3;
        let r = check(&i);
        assert!(r.grace_period_satisfied);
    }

    #[test]
    fn tx_no_written_lease_disclosure_non_compliant() {
        let mut i = input("TX");
        i.days_past_due = 30;
        i.late_fee_in_written_lease = false;
        let r = check(&i);
        assert!(r.grace_period_satisfied);
        assert!(
            !r.landlord_compliant,
            "TX § 92.019 requires written-lease disclosure"
        );
    }

    // ── OR 4-day boundary ──────────────────────────────────────────

    #[test]
    fn or_4_days_does_not_satisfy() {
        let mut i = input("OR");
        i.days_past_due = 4;
        let r = check(&i);
        assert!(!r.grace_period_satisfied);
    }

    #[test]
    fn or_5_days_satisfies() {
        let mut i = input("OR");
        i.days_past_due = 5;
        let r = check(&i);
        assert!(r.grace_period_satisfied);
    }

    // ── CT 9-day boundary ──────────────────────────────────────────

    #[test]
    fn ct_9_days_does_not_satisfy() {
        let mut i = input("CT");
        i.days_past_due = 9;
        let r = check(&i);
        assert!(!r.grace_period_satisfied);
    }

    #[test]
    fn ct_10_days_satisfies() {
        let mut i = input("CT");
        i.days_past_due = 10;
        let r = check(&i);
        assert!(r.grace_period_satisfied);
    }

    // ── No-statute default ─────────────────────────────────────────

    #[test]
    fn default_state_any_day_satisfies_grace() {
        let mut i = input("FL");
        i.days_past_due = 1;
        let r = check(&i);
        assert!(r.grace_period_satisfied);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ca_no_statute_landlord_compliant_immediately() {
        let mut i = input("CA");
        i.days_past_due = 0;
        let r = check(&i);
        assert!(r.landlord_compliant);
    }

    // ── No-fee path ────────────────────────────────────────────────

    #[test]
    fn no_late_fee_charged_landlord_compliant_dormant() {
        let mut i = input("MA");
        i.late_fee_charged_dollars = 0;
        i.days_past_due = 1;
        let r = check(&i);
        assert!(r.landlord_compliant, "no fee charged → compliance dormant");
        assert!(r.note.contains("dormant"));
    }

    // ── Citations ──────────────────────────────────────────────────

    #[test]
    fn ma_citation_mentions_c_186_15b_and_30() {
        let r = check(&input("MA"));
        assert!(r.citation.contains("c. 186 § 15B"));
        assert!(r.citation.contains("30 full days"));
    }

    #[test]
    fn ny_citation_mentions_238_a_and_hstpa() {
        let r = check(&input("NY"));
        assert!(r.citation.contains("§ 238-a"));
        assert!(r.citation.contains("HSTPA 2019"));
    }

    #[test]
    fn tx_citation_mentions_92_019_and_2_days() {
        let r = check(&input("TX"));
        assert!(r.citation.contains("§ 92.019"));
        assert!(r.citation.contains("2 full days"));
    }

    #[test]
    fn nc_citation_mentions_42_46_and_5_days() {
        let r = check(&input("NC"));
        assert!(r.citation.contains("§ 42-46"));
        assert!(r.citation.contains("5-day grace"));
    }

    #[test]
    fn or_citation_mentions_90_260_and_4_days() {
        let r = check(&input("OR"));
        assert!(r.citation.contains("90.260"));
        assert!(r.citation.contains("fourth day"));
    }

    #[test]
    fn ct_citation_mentions_47a_15a_and_9_days() {
        let r = check(&input("CT"));
        assert!(r.citation.contains("§ 47a-15a"));
        assert!(r.citation.contains("nine-day"));
    }

    // ── Coverage / single-regime-count invariants ─────────────────

    #[test]
    fn coverage_is_all_50_states_plus_dc() {
        let count = RULES.len();
        assert_eq!(count, 51, "expected 50 states + DC, got {count}");
    }

    #[test]
    fn citations_never_empty() {
        for (code, rule) in RULES.iter() {
            assert!(!rule.citation.is_empty(), "{code} empty citation");
        }
    }

    #[test]
    fn ma_only_long_grace_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| matches!(r.regime, GracePeriodRegime::MassachusettsLongGracePeriod))
            .count();
        assert_eq!(count, 1, "MA only");
    }

    #[test]
    fn ct_only_nine_day_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| matches!(r.regime, GracePeriodRegime::ConnecticutNineDayGracePeriod))
            .count();
        assert_eq!(count, 1, "CT only");
    }

    #[test]
    fn tx_only_short_two_day_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| matches!(r.regime, GracePeriodRegime::TexasShortGracePeriod))
            .count();
        assert_eq!(count, 1, "TX only");
    }

    #[test]
    fn or_only_four_day_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| matches!(r.regime, GracePeriodRegime::OregonFourDayGracePeriod))
            .count();
        assert_eq!(count, 1, "OR only");
    }

    #[test]
    fn standard_five_day_regime_has_exactly_4_states() {
        let count = RULES
            .iter()
            .filter(|(_, r)| matches!(r.regime, GracePeriodRegime::StandardFiveDayGracePeriod))
            .count();
        assert_eq!(count, 4, "NY + NC + WA + VA = 4");
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn note_non_compliant_describes_grace_failure() {
        let mut i = input("MA");
        i.days_past_due = 10;
        let r = check(&i);
        assert!(r.note.contains("NOT satisfied"));
        assert!(r.note.contains("NON-COMPLIANT"));
    }

    #[test]
    fn note_compliant_describes_compliance() {
        let mut i = input("NY");
        i.days_past_due = 10;
        let r = check(&i);
        assert!(r.note.contains("compliant"));
    }

    #[test]
    fn note_lease_disclosure_failure_distinct_from_grace_failure() {
        let mut i = input("NY");
        i.days_past_due = 10;
        i.late_fee_in_written_lease = false;
        let r = check(&i);
        assert!(r.note.contains("NOT disclosed in written lease"));
    }

    // ── Normalization ──────────────────────────────────────────────

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&input("ma"));
        assert_eq!(r.regime, GracePeriodRegime::MassachusettsLongGracePeriod);
    }
}
