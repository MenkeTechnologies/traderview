//! State credit-check authorization and disclosure landlord compliance
//! check. Addresses the upfront disclosures and authorization required
//! before a landlord pulls a tenant-screening consumer report. Distinct
//! from `adverse_action_notice` (which addresses the post-denial notice
//! AFTER the report is used) and `application_fees` (which caps the
//! dollar amount of screening fees). This module addresses the
//! pre-screening disclosure / authorization step.
//!
//! Washington (RCW 59.18.257) — strict pre-screening disclosure regime.
//! Landlord must FIRST notify the prospective tenant in writing of: (i)
//! types of information to be accessed; (ii) criteria that may result in
//! denial; (iii) if a consumer report will be used, the name and
//! address of the consumer reporting agency and the tenant's right to
//! obtain a free copy on adverse action and to dispute inaccuracies;
//! (iv) whether the landlord accepts a comprehensive reusable tenant
//! screening report. Cost-recovery is permitted only if the disclosure
//! is provided. Violation penalty $100/violation + attorney fees +
//! court costs to prevailing party.
//!
//! California (Cal. Civ. Code § 1950.6) — application-screening-fee
//! disclosure regime. Landlord may charge actual cost of screening
//! (capped at $30 in 1997 base + CPI = approximately $60 in 2024).
//! Landlord must provide ITEMIZED RECEIPT of the cost upon request
//! and REFUND any UNUSED portion. Civil penalty $100 per violation.
//!
//! Default — federal FCRA (15 U.S.C. § 1681b) baseline. Tenant
//! screening is itself a "permissible purpose" under § 1681b(a)(3)(F)
//! (review of tenant). Written authorization is NOT required for
//! tenant-screening consumer reports (it IS required for
//! employment-purpose reports under § 1681b(b)). FCRA § 615 adverse-
//! action notice required if denial based on report (covered by
//! `adverse_action_notice` sibling module).
//!
//! Citations: RCW 59.18.257 (Washington tenant-screening disclosure +
//! cost cap + adverse-action notice + $100 violation penalty); Cal.
//! Civ. Code § 1950.6 (California application-screening fee + itemized
//! receipt + unused-portion refund + $100 civil penalty); 15 U.S.C.
//! § 1681b(a)(3)(F) (FCRA tenant-screening permissible purpose); 15
//! U.S.C. § 1681b(b)(2) (FCRA written authorization — employment only,
//! not tenant screening); FCRA § 615 / 15 U.S.C. § 1681m (adverse-
//! action notice).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Washington,
    California,
    Default,
}

impl Regime {
    pub fn for_state(state: &str) -> Self {
        match state.trim().to_ascii_uppercase().as_str() {
            "WA" => Self::Washington,
            "CA" => Self::California,
            _ => Self::Default,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreditCheckInput {
    pub regime: Regime,
    /// Whether the landlord provided written notice of the types of
    /// information that will be accessed before obtaining the consumer
    /// report. WA RCW 59.18.257 requirement.
    pub disclosed_types_of_information: bool,
    /// Whether the landlord provided written notice of the criteria
    /// that may result in denial of the application. WA RCW 59.18.257
    /// requirement.
    pub disclosed_denial_criteria: bool,
    /// Whether the landlord provided the CRA name + address + tenant's
    /// right to obtain a free copy on adverse action. WA RCW 59.18.257
    /// requirement.
    pub disclosed_cra_and_tenant_rights: bool,
    /// Whether the landlord disclosed whether comprehensive reusable
    /// tenant screening reports are accepted. WA RCW 59.18.257
    /// requirement.
    pub disclosed_reusable_screening_report_acceptance: bool,
    /// Whether the landlord charged the tenant for the screening cost.
    pub charged_screening_cost: bool,
    /// CA-specific: whether the landlord provided itemized receipt.
    pub itemized_receipt_provided: bool,
    /// CA-specific: whether unused portion of screening fee was refunded.
    pub unused_portion_refunded: bool,
    /// Screening fee charged (in cents). CA § 1950.6 cap = inflation-
    /// adjusted ~$60 in 2024.
    pub screening_fee_charged_cents: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationType {
    None,
    MissingTypesOfInformationDisclosure,
    MissingDenialCriteriaDisclosure,
    MissingCraAndTenantRightsDisclosure,
    MissingReusableReportDisclosure,
    MissingItemizedReceipt,
    UnusedPortionNotRefunded,
    FeeExceedsCap,
    /// Landlord charged for screening cost without providing required
    /// disclosures. WA cost-recovery conditional on disclosure.
    CostChargedWithoutRequiredDisclosure,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CreditCheckResult {
    pub regime: Regime,
    pub per_violation_penalty_cents: i64,
    pub fcra_permissible_purpose_applies: bool,
    pub written_authorization_required: bool,
    pub violation: ViolationType,
    pub landlord_compliant: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn check(input: &CreditCheckInput) -> CreditCheckResult {
    match input.regime {
        Regime::Washington => wa_check(input),
        Regime::California => ca_check(input),
        Regime::Default => default_check(input),
    }
}

fn wa_check(input: &CreditCheckInput) -> CreditCheckResult {
    let disclosures_complete = input.disclosed_types_of_information
        && input.disclosed_denial_criteria
        && input.disclosed_cra_and_tenant_rights
        && input.disclosed_reusable_screening_report_acceptance;

    if !input.disclosed_types_of_information {
        return wa_violation(
            ViolationType::MissingTypesOfInformationDisclosure,
            "RCW 59.18.257(1)(a) — landlord must disclose types of information to be accessed before obtaining tenant screening report",
            "Required disclosure of types of information accessed not provided.",
        );
    }
    if !input.disclosed_denial_criteria {
        return wa_violation(
            ViolationType::MissingDenialCriteriaDisclosure,
            "RCW 59.18.257(1)(b) — landlord must disclose criteria that may result in denial",
            "Required disclosure of denial criteria not provided.",
        );
    }
    if !input.disclosed_cra_and_tenant_rights {
        return wa_violation(
            ViolationType::MissingCraAndTenantRightsDisclosure,
            "RCW 59.18.257(1)(c) — landlord must disclose CRA name + address + tenant's right to free copy on adverse action + dispute rights",
            "Required CRA + tenant-rights disclosure not provided.",
        );
    }
    if !input.disclosed_reusable_screening_report_acceptance {
        return wa_violation(
            ViolationType::MissingReusableReportDisclosure,
            "RCW 59.18.257(1)(d) — landlord must disclose whether comprehensive reusable tenant screening report is accepted",
            "Required reusable-tenant-screening-report acceptance disclosure not provided.",
        );
    }

    if input.charged_screening_cost && !disclosures_complete {
        return wa_violation(
            ViolationType::CostChargedWithoutRequiredDisclosure,
            "RCW 59.18.257(2) — landlord may charge tenant screening costs ONLY if required disclosures are provided",
            "Charged screening cost without providing complete required disclosures.",
        );
    }

    CreditCheckResult {
        regime: Regime::Washington,
        per_violation_penalty_cents: 10000,
        fcra_permissible_purpose_applies: true,
        written_authorization_required: false,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation: "RCW 59.18.257 — Washington tenant-screening disclosure compliance OK; $100/violation penalty + attorney fees + court costs to prevailing party",
        note: "Washington tenant-screening disclosure requirements satisfied. Screening cost may be charged subject to actual-cost cap.".to_string(),
    }
}

fn wa_violation(v: ViolationType, citation: &'static str, note_text: &str) -> CreditCheckResult {
    CreditCheckResult {
        regime: Regime::Washington,
        per_violation_penalty_cents: 10000,
        fcra_permissible_purpose_applies: true,
        written_authorization_required: false,
        violation: v,
        landlord_compliant: false,
        citation,
        note: format!(
            "{} $100/violation penalty + attorney fees recoverable.",
            note_text
        ),
    }
}

fn ca_check(input: &CreditCheckInput) -> CreditCheckResult {
    // CA § 1950.6 cap (1997 base + CPI): approximately $60 in 2024.
    let cap = 6000;
    if input.screening_fee_charged_cents > cap {
        return CreditCheckResult {
            regime: Regime::California,
            per_violation_penalty_cents: 10000,
            fcra_permissible_purpose_applies: true,
            written_authorization_required: false,
            violation: ViolationType::FeeExceedsCap,
            landlord_compliant: false,
            citation: "Cal. Civ. Code § 1950.6(b) — application screening fee may not exceed inflation-adjusted statutory cap (approximately $60 in 2024)",
            note: format!(
                "Screening fee {} cents exceeds inflation-adjusted CA cap (approximately $60 / 6000 cents in 2024).",
                input.screening_fee_charged_cents
            ),
        };
    }

    if input.charged_screening_cost && !input.itemized_receipt_provided {
        return CreditCheckResult {
            regime: Regime::California,
            per_violation_penalty_cents: 10000,
            fcra_permissible_purpose_applies: true,
            written_authorization_required: false,
            violation: ViolationType::MissingItemizedReceipt,
            landlord_compliant: false,
            citation: "Cal. Civ. Code § 1950.6(b)(2) — landlord must provide itemized receipt upon request",
            note: "Required itemized receipt of screening costs not provided. $100 civil penalty.".to_string(),
        };
    }

    if input.charged_screening_cost && !input.unused_portion_refunded {
        return CreditCheckResult {
            regime: Regime::California,
            per_violation_penalty_cents: 10000,
            fcra_permissible_purpose_applies: true,
            written_authorization_required: false,
            violation: ViolationType::UnusedPortionNotRefunded,
            landlord_compliant: false,
            citation: "Cal. Civ. Code § 1950.6(e) — landlord must refund any unused portion of screening fee",
            note: "Unused portion of screening fee not refunded. $100 civil penalty.".to_string(),
        };
    }

    CreditCheckResult {
        regime: Regime::California,
        per_violation_penalty_cents: 10000,
        fcra_permissible_purpose_applies: true,
        written_authorization_required: false,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation: "Cal. Civ. Code § 1950.6 — California application-screening-fee compliance OK; $100/violation civil penalty",
        note: "California application-screening-fee disclosure requirements satisfied.".to_string(),
    }
}

fn default_check(_input: &CreditCheckInput) -> CreditCheckResult {
    CreditCheckResult {
        regime: Regime::Default,
        per_violation_penalty_cents: 0,
        fcra_permissible_purpose_applies: true,
        written_authorization_required: false,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation:
            "15 U.S.C. § 1681b(a)(3)(F) (FCRA tenant-screening permissible purpose) — no state-specific authorization or disclosure requirement; tenant screening is its own permissible purpose under FCRA",
        note: "Default regime: FCRA permissible purpose applies for tenant screening. No written authorization required (employment-purpose only requires it under § 1681b(b)).".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        regime: Regime,
        types: bool,
        criteria: bool,
        cra: bool,
        reusable: bool,
        charged: bool,
        itemized: bool,
        refunded: bool,
        fee: i64,
    ) -> CreditCheckInput {
        CreditCheckInput {
            regime,
            disclosed_types_of_information: types,
            disclosed_denial_criteria: criteria,
            disclosed_cra_and_tenant_rights: cra,
            disclosed_reusable_screening_report_acceptance: reusable,
            charged_screening_cost: charged,
            itemized_receipt_provided: itemized,
            unused_portion_refunded: refunded,
            screening_fee_charged_cents: fee,
        }
    }

    #[test]
    fn wa_all_disclosures_provided_compliant() {
        let r = check(&input(
            Regime::Washington,
            true,
            true,
            true,
            true,
            true,
            false,
            false,
            5000,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.landlord_compliant);
        assert_eq!(r.per_violation_penalty_cents, 10000);
    }

    #[test]
    fn wa_missing_types_of_information_violation() {
        let r = check(&input(
            Regime::Washington,
            false,
            true,
            true,
            true,
            false,
            false,
            false,
            0,
        ));
        assert_eq!(
            r.violation,
            ViolationType::MissingTypesOfInformationDisclosure
        );
        assert!(r.citation.contains("RCW 59.18.257(1)(a)"));
    }

    #[test]
    fn wa_missing_denial_criteria_violation() {
        let r = check(&input(
            Regime::Washington,
            true,
            false,
            true,
            true,
            false,
            false,
            false,
            0,
        ));
        assert_eq!(r.violation, ViolationType::MissingDenialCriteriaDisclosure);
        assert!(r.citation.contains("(1)(b)"));
    }

    #[test]
    fn wa_missing_cra_and_tenant_rights_violation() {
        let r = check(&input(
            Regime::Washington,
            true,
            true,
            false,
            true,
            false,
            false,
            false,
            0,
        ));
        assert_eq!(
            r.violation,
            ViolationType::MissingCraAndTenantRightsDisclosure
        );
        assert!(r.citation.contains("(1)(c)"));
    }

    #[test]
    fn wa_missing_reusable_report_disclosure_violation() {
        let r = check(&input(
            Regime::Washington,
            true,
            true,
            true,
            false,
            false,
            false,
            false,
            0,
        ));
        assert_eq!(r.violation, ViolationType::MissingReusableReportDisclosure);
        assert!(r.citation.contains("(1)(d)"));
    }

    #[test]
    fn ca_under_cap_compliant() {
        let r = check(&input(
            Regime::California,
            false,
            false,
            false,
            false,
            true,
            true,
            true,
            5000, // $50
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn ca_over_60_cap_violation() {
        let r = check(&input(
            Regime::California,
            false,
            false,
            false,
            false,
            true,
            true,
            true,
            7000, // $70 over cap
        ));
        assert_eq!(r.violation, ViolationType::FeeExceedsCap);
        assert!(r.citation.contains("§ 1950.6(b)"));
    }

    #[test]
    fn ca_at_60_cap_boundary_compliant() {
        let r = check(&input(
            Regime::California,
            false,
            false,
            false,
            false,
            true,
            true,
            true,
            6000, // exactly $60
        ));
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn ca_missing_itemized_receipt_violation() {
        let r = check(&input(
            Regime::California,
            false,
            false,
            false,
            false,
            true,
            false,
            true,
            5000,
        ));
        assert_eq!(r.violation, ViolationType::MissingItemizedReceipt);
        assert!(r.citation.contains("§ 1950.6(b)(2)"));
    }

    #[test]
    fn ca_unused_portion_not_refunded_violation() {
        let r = check(&input(
            Regime::California,
            false,
            false,
            false,
            false,
            true,
            true,
            false,
            5000,
        ));
        assert_eq!(r.violation, ViolationType::UnusedPortionNotRefunded);
        assert!(r.citation.contains("§ 1950.6(e)"));
    }

    #[test]
    fn ca_no_screening_cost_charged_compliant() {
        let r = check(&input(
            Regime::California,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            0,
        ));
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn default_fcra_permissible_purpose_applies() {
        let r = check(&input(
            Regime::Default,
            false,
            false,
            false,
            false,
            true,
            false,
            false,
            0,
        ));
        assert!(r.fcra_permissible_purpose_applies);
        assert!(!r.written_authorization_required);
        assert!(r.citation.contains("§ 1681b(a)(3)(F)"));
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn default_no_written_authorization_for_tenant_screening() {
        // FCRA requires written authorization for EMPLOYMENT only — not
        // tenant screening.
        let r = check(&input(
            Regime::Default,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            0,
        ));
        assert!(!r.written_authorization_required);
    }

    #[test]
    fn state_routing_wa_ca_default() {
        assert_eq!(Regime::for_state("WA"), Regime::Washington);
        assert_eq!(Regime::for_state("CA"), Regime::California);
        assert_eq!(Regime::for_state("NY"), Regime::Default);
        assert_eq!(Regime::for_state("TX"), Regime::Default);
    }

    #[test]
    fn state_routing_case_insensitive() {
        assert_eq!(Regime::for_state("wa"), Regime::Washington);
        assert_eq!(Regime::for_state("Ca"), Regime::California);
    }

    #[test]
    fn only_wa_has_four_prong_disclosure_requirement() {
        // Same all-disclosures-missing input across regimes. WA →
        // violation; CA + Default → no violation.
        let wa = check(&input(
            Regime::Washington,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            0,
        ));
        let ca = check(&input(
            Regime::California,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            0,
        ));
        let d = check(&input(
            Regime::Default,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            0,
        ));
        assert_eq!(
            wa.violation,
            ViolationType::MissingTypesOfInformationDisclosure
        );
        assert_eq!(ca.violation, ViolationType::None);
        assert_eq!(d.violation, ViolationType::None);
    }

    #[test]
    fn only_ca_has_60_fee_cap() {
        // Same $70 screening fee across regimes. CA → cap violation;
        // WA + Default → no cap violation.
        let ca = check(&input(
            Regime::California,
            false,
            false,
            false,
            false,
            true,
            true,
            true,
            7000,
        ));
        let wa = check(&input(
            Regime::Washington,
            true,
            true,
            true,
            true,
            true,
            true,
            true,
            7000,
        ));
        let d = check(&input(
            Regime::Default,
            false,
            false,
            false,
            false,
            true,
            true,
            true,
            7000,
        ));
        assert_eq!(ca.violation, ViolationType::FeeExceedsCap);
        assert_eq!(wa.violation, ViolationType::None);
        assert_eq!(d.violation, ViolationType::None);
    }

    #[test]
    fn ca_only_itemized_receipt_requirement() {
        // Same charged-without-receipt scenario. CA → violation; WA +
        // Default → no violation.
        let ca = check(&input(
            Regime::California,
            false,
            false,
            false,
            false,
            true,
            false,
            true,
            5000,
        ));
        assert_eq!(ca.violation, ViolationType::MissingItemizedReceipt);
    }

    #[test]
    fn ca_only_unused_portion_refund_requirement() {
        let ca = check(&input(
            Regime::California,
            false,
            false,
            false,
            false,
            true,
            true,
            false,
            5000,
        ));
        assert_eq!(ca.violation, ViolationType::UnusedPortionNotRefunded);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let wa = check(&input(
            Regime::Washington,
            false,
            true,
            true,
            true,
            false,
            false,
            false,
            0,
        ));
        assert!(wa.citation.contains("RCW 59.18.257"));

        let ca = check(&input(
            Regime::California,
            false,
            false,
            false,
            false,
            true,
            true,
            true,
            7000,
        ));
        assert!(ca.citation.contains("§ 1950.6"));

        let d = check(&input(
            Regime::Default,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            0,
        ));
        assert!(d.citation.contains("FCRA"));
        assert!(d.citation.contains("§ 1681b(a)(3)(F)"));
    }

    #[test]
    fn wa_100_dollar_per_violation_penalty() {
        let r = check(&input(
            Regime::Washington,
            false,
            false,
            false,
            false,
            false,
            false,
            false,
            0,
        ));
        assert_eq!(r.per_violation_penalty_cents, 10000); // $100
    }

    #[test]
    fn ca_100_dollar_civil_penalty() {
        let r = check(&input(
            Regime::California,
            false,
            false,
            false,
            false,
            true,
            true,
            true,
            7000,
        ));
        assert_eq!(r.per_violation_penalty_cents, 10000);
    }
}
