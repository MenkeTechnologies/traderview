//! State fire-sprinkler disclosure requirement in residential leases.
//!
//! Distinct from `detector_requirements` (which is about smoke /
//! CO detector installation) and from `heat_requirements` (heating
//! adequacy). This module captures the statutory requirement that
//! residential leases contain a conspicuous notice about the
//! presence or absence of a fire-sprinkler system. New York is the
//! lone state with a specific statutory disclosure rule; the rest
//! rely on building codes or local fire codes that do not extend
//! to lease-text disclosure.
//!
//! Two regimes:
//!
//! `NewYorkSprinklerSystemNotice`: NY only. N.Y. RPL § 231-a (eff.
//! 2014-12-03). Every residential lease in New York State must
//! contain a **conspicuous notice in BOLD FACE TYPE** stating
//! whether or not a maintained and operative sprinkler system
//! exists in the leased premises. If a sprinkler system DOES
//! exist, the lease must also provide the **last date of
//! maintenance and inspection**.
//!
//! The statute has **NO STATUTORY PENALTY** provision — the
//! effect of noncompliance is uncertain until tested in
//! litigation. In practice, courts may construe the lease against
//! the landlord (contra proferentem) and could find the lease
//! unenforceable as to fire-related issues.
//!
//! `NoStateFireSprinklerDisclosureLaw`: 49 other states + DC.
//! No statewide residential-lease fire-sprinkler disclosure
//! statute confirmed. Building codes and local fire codes may
//! require installation in certain occupancy classes, but they
//! do not extend to lease-text disclosure. Tenants must inquire
//! independently.
//!
//! Sources:
//! [N.Y. RPL § 231-a (2025) — Justia](https://law.justia.com/codes/new-york/rpp/article-7/231-a/),
//! [N.Y. RPL § 231-a — NY Senate](https://www.nysenate.gov/legislation/laws/RPP/231-A),
//! [N.Y. RPL § 231-a — FindLaw](https://codes.findlaw.com/ny/real-property-law/rpp-sect-231-a/),
//! [Bollhofer Law — New Law Requires Sprinkler System Notice to Tenants](https://www.bollhoferlaw.com/articles/new-law-requires-sprinkler-system-notice-to-tenants),
//! [NYS Sprinkler Disclosure Rider to Residential Lease (RCG PDF)](https://www.rcgltd.net/Images_Content/Site1/Files/Pages/nys-sprinkler-disclosure-rider-residential-lease.pdf).

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FireSprinklerDisclosureRegime {
    NewYorkSprinklerSystemNotice,
    NoStateFireSprinklerDisclosureLaw,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: FireSprinklerDisclosureRegime,
    /// True if statute requires the disclosure to appear in bold
    /// face type / conspicuous format.
    pub bold_face_required: bool,
    /// True if statute requires disclosure of the last maintenance
    /// and inspection date when sprinkler system is present.
    pub last_maintenance_date_required: bool,
    /// True if the statute carries a specific monetary penalty
    /// for noncompliance.
    pub statutory_penalty_provided: bool,
    pub citation: &'static str,
}

const fn rule(
    regime: FireSprinklerDisclosureRegime,
    bold_face_required: bool,
    last_maintenance_date_required: bool,
    statutory_penalty_provided: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        bold_face_required,
        last_maintenance_date_required,
        statutory_penalty_provided,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use FireSprinklerDisclosureRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "NY",
        rule(
            NewYorkSprinklerSystemNotice,
            true,
            true,
            false,
            "N.Y. RPL § 231-a (eff. 2014-12-03) — every residential lease must contain conspicuous notice in BOLD FACE TYPE as to existence or non-existence of a maintained and operative sprinkler system in the leased premises; if system exists, lease must include last date of maintenance and inspection; NO statutory penalty provided (enforcement uncertain until litigation)",
        ),
    );

    // NoStateFireSprinklerDisclosureLaw default — 49 other states + DC.
    let no_state = [
        "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DC",
        "DE", "FL", "GA", "HI", "ID", "IL", "IN", "IA",
        "KS", "KY", "LA", "ME", "MD", "MA", "MI", "MN",
        "MS", "MO", "MT", "NE", "NV", "NH", "NJ", "NM",
        "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC",
        "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV",
        "WI", "WY",
    ];
    for code in no_state {
        m.insert(
            code,
            rule(
                NoStateFireSprinklerDisclosureLaw,
                false,
                false,
                false,
                "No statewide residential-lease fire-sprinkler disclosure statute; building codes and local fire codes may require installation but do not extend to lease-text disclosure",
            ),
        );
    }

    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FireSprinklerDisclosureInput {
    pub state_code: String,
    /// True if the leased premises has a maintained and operative
    /// sprinkler system.
    pub sprinkler_system_present: bool,
    /// True if the lease contains the required notice.
    pub lease_contains_notice: bool,
    /// True if the notice is in bold face type / conspicuous
    /// format.
    pub notice_is_bold_face: bool,
    /// True if the lease discloses the last maintenance/inspection
    /// date (required only when sprinkler system is present).
    pub last_maintenance_date_disclosed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FireSprinklerDisclosureResult {
    pub regime: FireSprinklerDisclosureRegime,
    pub disclosure_required: bool,
    pub landlord_compliant: bool,
    pub bold_face_compliance_satisfied: bool,
    pub maintenance_date_compliance_satisfied: bool,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &FireSprinklerDisclosureInput) -> FireSprinklerDisclosureResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: FireSprinklerDisclosureRegime::NoStateFireSprinklerDisclosureLaw,
        bold_face_required: false,
        last_maintenance_date_required: false,
        statutory_penalty_provided: false,
        citation: "Unknown state code; no statewide fire-sprinkler disclosure statute assumed",
    });

    let required = matches!(
        rule.regime,
        FireSprinklerDisclosureRegime::NewYorkSprinklerSystemNotice
    );

    // Bold-face compliance: only relevant when statute requires it.
    let bold_face_ok = !rule.bold_face_required
        || !input.lease_contains_notice
        || input.notice_is_bold_face;

    // Maintenance-date compliance: only when sprinkler present AND
    // statute requires it.
    let maintenance_date_ok = !(rule.last_maintenance_date_required
        && input.sprinkler_system_present)
        || input.last_maintenance_date_disclosed;

    let landlord_compliant = !required
        || (input.lease_contains_notice && bold_face_ok && maintenance_date_ok);

    let regime_label = match rule.regime {
        FireSprinklerDisclosureRegime::NewYorkSprinklerSystemNotice => {
            "New York RPL § 231-a sprinkler-system notice"
        }
        FireSprinklerDisclosureRegime::NoStateFireSprinklerDisclosureLaw => {
            "no statewide fire-sprinkler disclosure"
        }
    };

    let note = if !required {
        format!(
            "State applies {} regime; no statutory disclosure obligation.",
            regime_label,
        )
    } else if landlord_compliant {
        format!(
            "State applies {} regime; landlord compliant — required notice in lease, bold face, and maintenance date (if applicable) disclosed.",
            regime_label,
        )
    } else {
        let mut reasons = vec![];
        if !input.lease_contains_notice {
            reasons.push("required notice not in lease");
        }
        if !bold_face_ok {
            reasons.push("notice not in bold face type");
        }
        if !maintenance_date_ok {
            reasons.push("last maintenance / inspection date not disclosed");
        }
        format!(
            "State applies {} regime; landlord NON-COMPLIANT: {}.",
            regime_label,
            reasons.join("; "),
        )
    };

    FireSprinklerDisclosureResult {
        regime: rule.regime,
        disclosure_required: required,
        landlord_compliant,
        bold_face_compliance_satisfied: bold_face_ok,
        maintenance_date_compliance_satisfied: maintenance_date_ok,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline(state: &str) -> FireSprinklerDisclosureInput {
        FireSprinklerDisclosureInput {
            state_code: state.to_string(),
            sprinkler_system_present: true,
            lease_contains_notice: true,
            notice_is_bold_face: true,
            last_maintenance_date_disclosed: true,
        }
    }

    // ── Regime classification ───────────────────────────────────────

    #[test]
    fn ny_sprinkler_system_notice_regime() {
        let r = check(&baseline("NY"));
        assert_eq!(
            r.regime,
            FireSprinklerDisclosureRegime::NewYorkSprinklerSystemNotice
        );
    }

    #[test]
    fn default_state_no_law_regime() {
        for s in ["AL", "CA", "FL", "TX", "WA", "DC", "WY", "MA", "NJ", "IL"] {
            let r = check(&baseline(s));
            assert_eq!(
                r.regime,
                FireSprinklerDisclosureRegime::NoStateFireSprinklerDisclosureLaw,
                "expected {s} no-law regime"
            );
        }
    }

    // ── NY compliance ──────────────────────────────────────────────

    #[test]
    fn ny_all_4_compliance_facets_met() {
        let r = check(&baseline("NY"));
        assert!(r.disclosure_required);
        assert!(r.landlord_compliant);
        assert!(r.bold_face_compliance_satisfied);
        assert!(r.maintenance_date_compliance_satisfied);
    }

    #[test]
    fn ny_no_notice_non_compliant() {
        let mut i = baseline("NY");
        i.lease_contains_notice = false;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert!(r.note.contains("required notice not in lease"));
    }

    #[test]
    fn ny_notice_not_bold_face_non_compliant() {
        let mut i = baseline("NY");
        i.notice_is_bold_face = false;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert!(!r.bold_face_compliance_satisfied);
        assert!(r.note.contains("not in bold face type"));
    }

    #[test]
    fn ny_no_maintenance_date_non_compliant_when_system_present() {
        let mut i = baseline("NY");
        i.last_maintenance_date_disclosed = false;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert!(!r.maintenance_date_compliance_satisfied);
        assert!(r.note.contains("last maintenance / inspection date not disclosed"));
    }

    #[test]
    fn ny_no_sprinkler_system_no_maintenance_date_required() {
        // When sprinkler system NOT present, maintenance date
        // compliance is moot — statute only requires it when
        // system present.
        let mut i = baseline("NY");
        i.sprinkler_system_present = false;
        i.last_maintenance_date_disclosed = false;
        let r = check(&i);
        assert!(r.landlord_compliant, "no system → no maintenance-date requirement");
    }

    // ── Default state ──────────────────────────────────────────────

    #[test]
    fn default_state_no_disclosure_required() {
        let r = check(&baseline("CA"));
        assert!(!r.disclosure_required);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn default_state_no_notice_still_compliant() {
        let mut i = baseline("FL");
        i.lease_contains_notice = false;
        i.notice_is_bold_face = false;
        i.last_maintenance_date_disclosed = false;
        let r = check(&i);
        assert!(r.landlord_compliant, "no state law → no compliance question");
    }

    // ── Citation contents ──────────────────────────────────────────

    #[test]
    fn ny_citation_mentions_231_a_and_bold_face() {
        let r = check(&baseline("NY"));
        assert!(r.citation.contains("§ 231-a"));
        assert!(r.citation.contains("BOLD FACE TYPE"));
        assert!(r.citation.contains("2014-12-03"));
    }

    #[test]
    fn ny_citation_mentions_no_statutory_penalty() {
        let r = check(&baseline("NY"));
        assert!(r.citation.contains("NO statutory penalty"));
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
    fn ny_only_sprinkler_notice_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    FireSprinklerDisclosureRegime::NewYorkSprinklerSystemNotice
                )
            })
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn ny_only_bold_face_required_state() {
        let count = RULES.iter().filter(|(_, r)| r.bold_face_required).count();
        assert_eq!(count, 1);
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn ny_compliant_note_mentions_regime() {
        let r = check(&baseline("NY"));
        assert!(r.note.contains("New York RPL § 231-a"));
    }

    #[test]
    fn default_state_note_says_no_obligation() {
        let r = check(&baseline("TX"));
        assert!(r.note.contains("no statutory disclosure obligation"));
    }

    // ── Normalization ──────────────────────────────────────────────

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&baseline("ny"));
        assert_eq!(
            r.regime,
            FireSprinklerDisclosureRegime::NewYorkSprinklerSystemNotice
        );
    }
}
