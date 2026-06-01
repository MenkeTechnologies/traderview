//! State tenant rent-payment-to-credit-bureau reporting requirements.
//!
//! Recent legislative trend allowing tenants to build credit
//! history by having on-time rent payments reported to consumer
//! credit bureaus. California is currently the only state with a
//! mandatory landlord-offer requirement; other states have
//! voluntary opt-in services from third-party providers (Esusu,
//! RentReporters, CredHub, etc.) but no statutory mandate.
//!
//! Two regimes:
//!
//! `CaliforniaAB2747RentReporting`: CA only. Assembly Bill 2747
//! (eff. 2025-04-01) added Cal. Civ. Code § 1954.06 requiring
//! landlords who own or manage a residential rental building with
//! **15 or more units** to OFFER tenants the option of positive
//! rent reporting to at least one consumer reporting agency
//! (Experian, Equifax, TransUnion, or similar).
//!
//! Key compliance prongs:
//!   - Written notice of the rent-reporting option must be
//!     provided at lease signing AND annually thereafter.
//!   - Tenant fee cap: lesser of $10/month OR landlord's actual
//!     cost (zero if landlord incurs no cost).
//!   - Tenant may opt in OR opt out at any time.
//!   - Tenant must remain current on rent to qualify for positive
//!     reporting (delinquency reports separately governed by FCRA).
//!
//! `NoStateRentReportingRequirement`: 49 other states + DC. No
//! statewide landlord-side rent-reporting mandate. Tenants may
//! still use voluntary third-party services that report
//! self-reported / verified rent payments to credit bureaus, but
//! landlord participation is not statutorily required.
//!
//! Sources:
//! [Cal. AB 2747 (2023-2024 session) — California Legislative Information](https://leginfo.legislature.ca.gov/faces/billTextClient.xhtml?bill_id=202320240AB2747),
//! [FrontLobby Guide — California Rent Reporting Law AB 2747](https://frontlobby.com/en/2025/03/understanding-californias-new-rent-reporting-law-ab-2747/),
//! [SNS Law Group — California's New Law on Positive Rent Reporting](https://snslawgroup.com/californias-new-law-on-positive-rent-reporting/),
//! [HBR Rentals — AB 2747 New California Law](https://www.hbrrentals.com/landlord-law-hub/ab-2747-new-california-law-mandates-credit-reporting-for-rental-applicants),
//! [Esusu — AB 2747 Basics for California Landlords](https://esusurent.com/blog/ab2747-basics).

use chrono::NaiveDate;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RentCreditReportingRegime {
    CaliforniaAB2747RentReporting,
    NoStateRentReportingRequirement,
}

#[derive(Debug, Clone)]
pub struct StateRule {
    pub regime: RentCreditReportingRegime,
    /// Effective date of the statute (None for default regime).
    pub effective_date: Option<NaiveDate>,
    /// Minimum unit count that triggers the landlord-offer
    /// requirement (CA = 15).
    pub minimum_units_for_requirement: u32,
    /// Maximum monthly fee the landlord may charge the tenant
    /// for the reporting service (CA = $10).
    pub maximum_monthly_fee_dollars: i64,
    /// True if statute requires both lease-signing notice AND
    /// annual renewal notice.
    pub annual_notice_required: bool,
    pub citation: &'static str,
}

const fn rule(
    regime: RentCreditReportingRegime,
    effective_date: Option<NaiveDate>,
    minimum_units_for_requirement: u32,
    maximum_monthly_fee_dollars: i64,
    annual_notice_required: bool,
    citation: &'static str,
) -> StateRule {
    StateRule {
        regime,
        effective_date,
        minimum_units_for_requirement,
        maximum_monthly_fee_dollars,
        annual_notice_required,
        citation,
    }
}

pub static RULES: Lazy<HashMap<&'static str, StateRule>> = Lazy::new(|| {
    use RentCreditReportingRegime::*;
    let mut m: HashMap<&'static str, StateRule> = HashMap::new();

    m.insert(
        "CA",
        rule(
            CaliforniaAB2747RentReporting,
            NaiveDate::from_ymd_opt(2025, 4, 1),
            15,
            10,
            true,
            "Cal. Civ. Code § 1954.06 (added by AB 2747, eff. 2025-04-01) — landlords with 15+ residential units must offer tenants positive rent reporting to at least one consumer reporting agency; written notice at lease signing + annually; tenant fee cap = lesser of $10/month or landlord's actual cost (zero if no cost); tenant may opt in or out at any time",
        ),
    );

    // NoStateRentReportingRequirement default — 49 other states + DC.
    let no_state = [
        "AL", "AK", "AZ", "AR", "CO", "CT", "DC", "DE",
        "FL", "GA", "HI", "ID", "IL", "IN", "IA", "KS",
        "KY", "LA", "ME", "MD", "MA", "MI", "MN", "MS",
        "MO", "MT", "NE", "NV", "NH", "NJ", "NM", "NY",
        "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC",
        "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV",
        "WI", "WY",
    ];
    for code in no_state {
        m.insert(
            code,
            rule(
                NoStateRentReportingRequirement,
                None,
                0,
                0,
                false,
                "No state rent-reporting requirement; tenants may use voluntary third-party services (Esusu, RentReporters, CredHub) but landlord participation not statutorily mandated",
            ),
        );
    }

    m
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RentCreditReportingInput {
    pub state_code: String,
    /// Number of dwelling units in the building.
    pub building_unit_count: u32,
    /// Date the analysis is being made (for CA effective-date gate).
    pub analysis_date: NaiveDate,
    /// True if the landlord has provided the required written
    /// rent-reporting notice to the tenant.
    pub landlord_provided_written_notice: bool,
    /// True if the landlord has provided the annual renewal
    /// notice (CA requires both lease-signing AND annual).
    pub landlord_provided_annual_notice: bool,
    /// Monthly fee the landlord is charging the tenant for the
    /// reporting service.
    pub monthly_fee_charged_dollars: i64,
    /// Landlord's actual cost per month for the reporting service.
    pub landlord_actual_monthly_cost_dollars: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RentCreditReportingResult {
    pub regime: RentCreditReportingRegime,
    pub statute_in_effect_on_date: bool,
    pub building_meets_unit_threshold: bool,
    pub offer_requirement_applies: bool,
    pub landlord_compliant: bool,
    pub fee_within_statutory_cap: bool,
    /// Maximum fee permitted on these facts (lesser of $10 or
    /// actual cost).
    pub maximum_fee_permitted_dollars: i64,
    pub citation: String,
    pub note: String,
}

pub fn check(input: &RentCreditReportingInput) -> RentCreditReportingResult {
    let code = input.state_code.trim().to_uppercase();
    let rule = RULES.get(code.as_str()).cloned().unwrap_or(StateRule {
        regime: RentCreditReportingRegime::NoStateRentReportingRequirement,
        effective_date: None,
        minimum_units_for_requirement: 0,
        maximum_monthly_fee_dollars: 0,
        annual_notice_required: false,
        citation: "Unknown state code; no statewide rent-reporting requirement assumed",
    });

    // Effective-date gate.
    let in_effect = match rule.effective_date {
        Some(eff) => input.analysis_date >= eff,
        None => !matches!(
            rule.regime,
            RentCreditReportingRegime::NoStateRentReportingRequirement
        ),
    };

    let meets_threshold = input.building_unit_count >= rule.minimum_units_for_requirement
        && rule.minimum_units_for_requirement > 0;

    let requirement_applies = in_effect && meets_threshold;

    // Notice compliance.
    let notice_satisfied = !requirement_applies
        || (input.landlord_provided_written_notice
            && (!rule.annual_notice_required || input.landlord_provided_annual_notice));

    // Fee cap: lesser of statutory cap or landlord's actual cost.
    let actual_cost = input.landlord_actual_monthly_cost_dollars.max(0);
    let max_fee = if matches!(
        rule.regime,
        RentCreditReportingRegime::CaliforniaAB2747RentReporting
    ) {
        rule.maximum_monthly_fee_dollars.min(actual_cost.max(0))
    } else {
        0
    };
    let fee_compliant = !requirement_applies
        || input.monthly_fee_charged_dollars <= max_fee;

    let landlord_compliant = !requirement_applies || (notice_satisfied && fee_compliant);

    let regime_label = match rule.regime {
        RentCreditReportingRegime::CaliforniaAB2747RentReporting => {
            "California AB 2747 rent-reporting (eff. 2025-04-01)"
        }
        RentCreditReportingRegime::NoStateRentReportingRequirement => {
            "no statewide rent-reporting requirement"
        }
    };

    let note = if !in_effect && !matches!(rule.regime, RentCreditReportingRegime::NoStateRentReportingRequirement) {
        format!(
            "State applies {} regime; statute not yet in effect on analysis date {}.",
            regime_label, input.analysis_date,
        )
    } else if !requirement_applies {
        format!(
            "State applies {} regime; offer requirement does not apply on these facts (building has {} units; threshold is {} units, or no statewide requirement).",
            regime_label, input.building_unit_count, rule.minimum_units_for_requirement,
        )
    } else if landlord_compliant {
        format!(
            "State applies {} regime; landlord compliant on these facts (notice provided + fee within $${} cap).",
            regime_label, max_fee,
        )
    } else {
        let mut reasons = vec![];
        if !notice_satisfied {
            reasons.push("required notice not provided (or annual renewal missed)");
        }
        if !fee_compliant {
            reasons.push("fee exceeds statutory cap");
        }
        format!(
            "State applies {} regime; landlord NON-COMPLIANT: {}.",
            regime_label,
            reasons.join("; "),
        )
    };

    RentCreditReportingResult {
        regime: rule.regime,
        statute_in_effect_on_date: in_effect,
        building_meets_unit_threshold: meets_threshold,
        offer_requirement_applies: requirement_applies,
        landlord_compliant,
        fee_within_statutory_cap: fee_compliant,
        maximum_fee_permitted_dollars: max_fee,
        citation: rule.citation.to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline(state: &str) -> RentCreditReportingInput {
        RentCreditReportingInput {
            state_code: state.to_string(),
            building_unit_count: 30,
            analysis_date: NaiveDate::from_ymd_opt(2026, 1, 1).unwrap(),
            landlord_provided_written_notice: true,
            landlord_provided_annual_notice: true,
            monthly_fee_charged_dollars: 5,
            landlord_actual_monthly_cost_dollars: 5,
        }
    }

    // ── Regime classification ───────────────────────────────────────

    #[test]
    fn ca_ab_2747_regime() {
        let r = check(&baseline("CA"));
        assert_eq!(
            r.regime,
            RentCreditReportingRegime::CaliforniaAB2747RentReporting
        );
    }

    #[test]
    fn default_state_no_requirement_regime() {
        for s in ["AL", "FL", "NY", "TX", "WA", "DC", "WY", "MA", "NJ"] {
            let r = check(&baseline(s));
            assert_eq!(
                r.regime,
                RentCreditReportingRegime::NoStateRentReportingRequirement,
                "expected {s} default regime"
            );
        }
    }

    // ── CA effective date ──────────────────────────────────────────

    #[test]
    fn ca_pre_effective_date_statute_not_in_effect() {
        let mut i = baseline("CA");
        i.analysis_date = NaiveDate::from_ymd_opt(2025, 3, 31).unwrap();
        let r = check(&i);
        assert!(!r.statute_in_effect_on_date);
        assert!(!r.offer_requirement_applies);
    }

    #[test]
    fn ca_on_effective_date_statute_in_effect() {
        let mut i = baseline("CA");
        i.analysis_date = NaiveDate::from_ymd_opt(2025, 4, 1).unwrap();
        let r = check(&i);
        assert!(r.statute_in_effect_on_date);
        assert!(r.offer_requirement_applies);
    }

    // ── CA 15-unit threshold ───────────────────────────────────────

    #[test]
    fn ca_14_units_below_threshold() {
        let mut i = baseline("CA");
        i.building_unit_count = 14;
        let r = check(&i);
        assert!(!r.building_meets_unit_threshold);
        assert!(!r.offer_requirement_applies);
    }

    #[test]
    fn ca_15_units_at_threshold() {
        let mut i = baseline("CA");
        i.building_unit_count = 15;
        let r = check(&i);
        assert!(r.building_meets_unit_threshold);
        assert!(r.offer_requirement_applies);
    }

    // ── CA notice compliance ───────────────────────────────────────

    #[test]
    fn ca_all_notice_provided_compliant() {
        let r = check(&baseline("CA"));
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ca_no_initial_notice_non_compliant() {
        let mut i = baseline("CA");
        i.landlord_provided_written_notice = false;
        let r = check(&i);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn ca_no_annual_notice_non_compliant() {
        let mut i = baseline("CA");
        i.landlord_provided_annual_notice = false;
        let r = check(&i);
        assert!(!r.landlord_compliant);
        assert!(r.note.contains("notice not provided"));
    }

    // ── CA fee cap (lesser of $10 or actual cost) ──────────────────

    #[test]
    fn ca_fee_at_or_below_actual_cost_compliant() {
        let mut i = baseline("CA");
        i.monthly_fee_charged_dollars = 5;
        i.landlord_actual_monthly_cost_dollars = 5;
        let r = check(&i);
        assert!(r.fee_within_statutory_cap);
        assert_eq!(r.maximum_fee_permitted_dollars, 5);
    }

    #[test]
    fn ca_fee_capped_at_10_when_actual_cost_higher() {
        // Actual cost $15, but cap is $10. Max permitted = lesser
        // of $10 / actual = $10.
        let mut i = baseline("CA");
        i.monthly_fee_charged_dollars = 10;
        i.landlord_actual_monthly_cost_dollars = 15;
        let r = check(&i);
        assert_eq!(r.maximum_fee_permitted_dollars, 10);
        assert!(r.fee_within_statutory_cap);
    }

    #[test]
    fn ca_fee_above_10_dollar_cap_non_compliant() {
        let mut i = baseline("CA");
        i.monthly_fee_charged_dollars = 15;
        i.landlord_actual_monthly_cost_dollars = 20;
        let r = check(&i);
        assert!(!r.fee_within_statutory_cap);
        assert!(!r.landlord_compliant);
    }

    #[test]
    fn ca_fee_above_actual_cost_non_compliant_even_below_10() {
        // Actual cost $3 → max permitted = min($10, $3) = $3.
        // Charging $5 violates.
        let mut i = baseline("CA");
        i.monthly_fee_charged_dollars = 5;
        i.landlord_actual_monthly_cost_dollars = 3;
        let r = check(&i);
        assert_eq!(r.maximum_fee_permitted_dollars, 3);
        assert!(!r.fee_within_statutory_cap);
    }

    #[test]
    fn ca_zero_actual_cost_zero_permitted_fee() {
        // Landlord incurs no cost → max fee = $0.
        let mut i = baseline("CA");
        i.monthly_fee_charged_dollars = 1;
        i.landlord_actual_monthly_cost_dollars = 0;
        let r = check(&i);
        assert_eq!(r.maximum_fee_permitted_dollars, 0);
        assert!(!r.fee_within_statutory_cap);
    }

    // ── Default state ──────────────────────────────────────────────

    #[test]
    fn default_state_no_obligation() {
        let r = check(&baseline("TX"));
        assert!(!r.offer_requirement_applies);
        assert!(r.landlord_compliant);
    }

    // ── Citation contents ──────────────────────────────────────────

    #[test]
    fn ca_citation_mentions_1954_06_and_ab_2747() {
        let r = check(&baseline("CA"));
        assert!(r.citation.contains("§ 1954.06"));
        assert!(r.citation.contains("AB 2747"));
        assert!(r.citation.contains("2025-04-01"));
        assert!(r.citation.contains("15+ residential units"));
        assert!(r.citation.contains("$10/month"));
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
    fn ca_only_rent_reporting_state() {
        let count = RULES
            .iter()
            .filter(|(_, r)| {
                matches!(
                    r.regime,
                    RentCreditReportingRegime::CaliforniaAB2747RentReporting
                )
            })
            .count();
        assert_eq!(count, 1);
    }

    // ── Notes ──────────────────────────────────────────────────────

    #[test]
    fn ca_pre_effective_note_describes_not_yet_in_effect() {
        let mut i = baseline("CA");
        i.analysis_date = NaiveDate::from_ymd_opt(2024, 12, 1).unwrap();
        let r = check(&i);
        assert!(r.note.contains("not yet in effect"));
    }

    #[test]
    fn ca_below_threshold_note_explains() {
        let mut i = baseline("CA");
        i.building_unit_count = 10;
        let r = check(&i);
        assert!(r.note.contains("offer requirement does not apply"));
        assert!(r.note.contains("threshold is 15 units"));
    }

    // ── Normalization ──────────────────────────────────────────────

    #[test]
    fn lowercase_state_code_normalizes() {
        let r = check(&baseline("ca"));
        assert_eq!(
            r.regime,
            RentCreditReportingRegime::CaliforniaAB2747RentReporting
        );
    }
}
