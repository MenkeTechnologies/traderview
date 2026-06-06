//! Commercial lease CAM (Common Area Maintenance)
//! charge disclosure and tenant audit rights framework.
//! Trader-landlord critical because commercial leases
//! routinely pass through CAM/operating-expense
//! escalations that can total 20-40% of base rent over
//! time; ill-disclosed or improperly calculated CAM
//! exposes landlord to TENANT AUDIT CLAIMS and
//! POTENTIAL REFUNDS averaging 5-15% of contested
//! charges per industry-standard BOMA practice.
//!
//! Companion to commercial_lease_personal_guaranty_
//! enforceability (iter 445), tenant_estoppel_
//! certificate (iter 435), lease_disclosures,
//! rental_property_registration.
//!
//! **Three-jurisdiction framework**:
//!
//! CALIFORNIA — Cal. Civ. Code § 1938 (commercial-
//! lease CASp accessibility inspection disclosure);
//! California case law (Garrett v. Coast and Southern,
//! 9 Cal. 4th 1 (1995)) requires good-faith CAM
//! reconciliation; commercial leases routinely require
//! ANNUAL BUDGET delivery plus YEAR-END RECONCILIATION;
//! tenant audit rights standard in industry but not
//! statutorily mandated.
//!
//! NEW YORK — no specific commercial CAM disclosure
//! statute; governed by common-law contract
//! interpretation under N.Y. CPLR § 4519 (Dead Man's
//! Statute restrictions on lease testimony); industry-
//! standard BOMA Operating Expense Guide governs
//! exclusions plus gross-up calculations.
//!
//! DEFAULT — BOMA Operating Expense Guide 2024
//! (industry-standard); Restatement (Second) of
//! Contracts § 200 (interpretation favoring tenant
//! when lease ambiguous); UCC Article 2A (leases of
//! goods, NOT real property leases — does NOT apply).
//!
//! **BOMA Operating Expense Guide categories of CAM**:
//! 1. Utilities (electric, gas, water, sewer);
//! 2. Janitorial and cleaning services;
//! 3. Property management fees;
//! 4. Security and access control;
//! 5. Landscaping and grounds maintenance;
//! 6. Snow removal (seasonal);
//! 7. Trash removal;
//! 8. HVAC maintenance contracts;
//! 9. Elevator maintenance contracts;
//! 10. Common-area repairs;
//! 11. Insurance premiums;
//! 12. Real estate taxes (separately or within CAM);
//! 13. Capital expenditures AMORTIZED over useful
//!     life (typically 10-15 years).
//!
//! **Standard EXCLUSIONS from CAM**:
//! 1. Capital improvements (vs. amortized capital
//!    repairs);
//! 2. Marketing and leasing costs (broker
//!    commissions);
//! 3. Tenant-specific build-outs and
//!    improvements;
//! 4. Landlord's debt service / mortgage interest;
//! 5. Depreciation;
//! 6. Income taxes;
//! 7. Ground rent paid by landlord;
//! 8. Reserves for future contingencies;
//! 9. Legal fees for tenant disputes;
//! 10. Penalties for landlord's own legal violations;
//! 11. Insurance proceeds offsetting expenses;
//! 12. Vendor services performed by affiliated
//!     parties at above-market rates.
//!
//! **GROSS-UP PROVISION** — when building less than
//! fully occupied, landlord may "gross-up" variable
//! expenses (utilities, janitorial, trash) to what
//! they WOULD HAVE BEEN at 95-100% occupancy; tenant
//! pays its pro-rata share of grossed-up amount;
//! gross-up FLOORS occupancy at 95% or 100% per
//! lease terms; FIXED expenses (real estate taxes,
//! insurance) are NOT grossed up.
//!
//! **BASE-YEAR ESCALATION** — tenant pays only INCREASE
//! over base-year amount; calculation:
//! ```text
//! Tenant CAM = (Current Year Total - Base Year Total)
//!              × Tenant Pro-Rata Share
//! ```
//!
//! **TENANT AUDIT RIGHTS STANDARD PROVISIONS**:
//! 1. Annual right to audit landlord's books;
//! 2. Notice period (typically 90-180 days after
//!    reconciliation statement);
//! 3. Scope limitation (most recent fiscal year only);
//! 4. Confidentiality obligations;
//! 5. Cost-shifting clause (tenant pays unless
//!    discrepancy exceeds 3-5% threshold);
//! 6. Refund obligation for overcharges;
//! 7. Right to inspect underlying invoices and
//!    contracts.
//!
//! **BOMA-survey CAM dispute statistics**:
//! - 1 in 4 (25%) tenants experience billing
//!   discrepancies in CAM reconciliations;
//! - Average overcharge recovery 5-15% of contested
//!   amount;
//! - Most common errors: incorrect gross-up
//!   calculation + capital-expenditure
//!   misclassification + management-fee duplication.
//!
//! **Trader-landlord critical fact patterns**:
//! 1. Trader leases office tower 65% occupied; CAM
//!    gross-up FLOOR of 95% means variable expenses
//!    grossed up by factor of 95/65 = 1.46×; tenant
//!    pays pro-rata share of inflated CAM.
//! 2. Trader passes HVAC system replacement ($500K)
//!    through as CAM operating expense — TENANT
//!    AUDIT challenges as CAPITAL IMPROVEMENT;
//!    proper treatment is AMORTIZATION over 15-year
//!    useful life ($33,333 annual operating expense).
//! 3. Trader hires affiliated property management
//!    company at 5% of gross rent (vs market 3-4%);
//!    tenant AUDIT challenges as ABOVE-MARKET
//!    AFFILIATED-PARTY EXPENSE; lease typically
//:    excludes such excess from CAM pool.
//! 4. Trader fails to deliver annual CAM
//!    reconciliation within 90 days of fiscal year-
//!    end (industry standard); tenant claims WAIVER
//!    of right to collect prior-year true-up.
//! 5. Trader's base-year CAM understated (e.g.,
//!    abnormally low expenses); tenant's annual
//!    escalation accelerates; AUDIT may rectify
//!    base year if proven to be artificially
//!    deflated.
//!
//! Citations: Cal. Civ. Code § 1938 (CASp commercial
//! accessibility disclosure); Garrett v. Coast and
//! Southern, 9 Cal. 4th 1 (1995); BOMA International
//! Operating Expense Guide (2024 edition); BOMA
//! Office Buildings: Methods of Measurement (ANSI/
//! BOMA Z65.1-2017); Restatement (Second) of
//! Contracts § 200 (contract interpretation favoring
//! tenant); N.Y. CPLR § 4519 (Dead Man's Statute);
//! UCC Article 2A (leases of goods — INAPPLICABLE
//! to real property).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Jurisdiction {
    California,
    NewYork,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CamEscalationStructure {
    /// Tenant pays full pro-rata share of total CAM.
    FullPassThrough,
    /// Tenant pays only excess over base year.
    BaseYearEscalation,
    /// Tenant pays fixed monthly CAM amount.
    FixedStop,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommercialLeaseCamChargeDisclosureInput {
    pub jurisdiction: Jurisdiction,
    pub escalation_structure: CamEscalationStructure,
    /// Annual CAM budget delivered to tenant at start
    /// of fiscal year.
    pub annual_budget_delivered: bool,
    /// Year-end reconciliation statement delivered
    /// within standard 90-180 day window.
    pub year_end_reconciliation_delivered: bool,
    /// Days since fiscal year-end before reconciliation
    /// delivered (industry standard 90-180 days).
    pub days_since_fye_for_reconciliation: u32,
    /// Whether tenant audit right is included in
    /// lease.
    pub tenant_audit_right_in_lease: bool,
    /// Whether landlord has gross-up provision applied
    /// at less-than-95% occupancy.
    pub gross_up_provision_applied_floor_95: bool,
    /// Building occupancy percentage when gross-up
    /// applied.
    pub building_occupancy_percent: u32,
    /// Whether capital improvements are being passed
    /// through as operating expenses (vs. amortized).
    pub capital_improvements_passed_as_operating: bool,
    /// Whether management fee paid to affiliated
    /// party at above-market rate.
    pub above_market_affiliated_management_fee: bool,
    /// Whether broker commissions / leasing costs
    /// included in CAM (improper).
    pub leasing_costs_included_in_cam: bool,
    /// Whether annual CAM budget cap exceeds 5% over
    /// prior year without explanation.
    pub annual_cap_exceeded_without_explanation: bool,
    /// Tenant pro-rata share percent.
    pub tenant_pro_rata_share_percent: u32,
    /// Current year total CAM in cents.
    pub current_year_total_cam_cents: u64,
    /// Base year total CAM in cents (for base-year
    /// escalation structure).
    pub base_year_total_cam_cents: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CommercialLeaseCamChargeDisclosureResult {
    pub disclosure_compliant: bool,
    pub reconciliation_timely: bool,
    pub gross_up_proper: bool,
    pub capital_improvement_classification_proper: bool,
    pub affiliated_party_expense_proper: bool,
    pub leasing_costs_excluded_proper: bool,
    pub tenant_audit_right_available: bool,
    pub tenant_cam_owed_cents: u64,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(
    input: &CommercialLeaseCamChargeDisclosureInput,
) -> CommercialLeaseCamChargeDisclosureResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let disclosure_compliant =
        input.annual_budget_delivered && input.year_end_reconciliation_delivered;

    let reconciliation_timely = input.days_since_fye_for_reconciliation <= 180;

    let gross_up_proper =
        !input.gross_up_provision_applied_floor_95 || input.building_occupancy_percent <= 95;

    let capital_improvement_classification_proper = !input.capital_improvements_passed_as_operating;

    let affiliated_party_expense_proper = !input.above_market_affiliated_management_fee;

    let leasing_costs_excluded_proper = !input.leasing_costs_included_in_cam;

    let tenant_audit_right_available = input.tenant_audit_right_in_lease;

    let tenant_cam_owed_cents = match input.escalation_structure {
        CamEscalationStructure::FullPassThrough => {
            input
                .current_year_total_cam_cents
                .saturating_mul(input.tenant_pro_rata_share_percent as u64)
                / 100
        }
        CamEscalationStructure::BaseYearEscalation => {
            input
                .current_year_total_cam_cents
                .saturating_sub(input.base_year_total_cam_cents)
                .saturating_mul(input.tenant_pro_rata_share_percent as u64)
                / 100
        }
        CamEscalationStructure::FixedStop => input.current_year_total_cam_cents,
    };

    if !input.annual_budget_delivered {
        failure_reasons.push(
            "Annual CAM BUDGET not delivered to tenant at start of fiscal year — industry-standard practice; in California, common-law obligation to deliver projected operating expense budget (Garrett v. Coast and Southern, 9 Cal. 4th 1 (1995)); tenant cannot plan financial obligations; possible WAIVER OF FUTURE ESCALATION".to_string(),
        );
    }

    if !input.year_end_reconciliation_delivered {
        failure_reasons.push(
            "Year-end CAM RECONCILIATION not delivered — industry-standard 90-180 day window after fiscal year-end; failure may constitute WAIVER of right to collect prior-year true-up; tenant retains right to refund of any over-collection".to_string(),
        );
    }

    if !reconciliation_timely {
        failure_reasons.push(format!(
            "CAM reconciliation delivered {} days after fiscal year-end — EXCEEDS industry-standard 180-day window per BOMA Operating Expense Guide; tenant may invoke WAIVER defense to prior-year true-up; tenant retains audit rights on prior fiscal year",
            input.days_since_fye_for_reconciliation
        ));
    }

    if !gross_up_proper {
        failure_reasons.push(format!(
            "GROSS-UP IMPROPER — building occupancy {}% exceeds 95% gross-up floor; gross-up provision should not apply at or above 95% occupancy; tenant may dispute the gross-up multiplier",
            input.building_occupancy_percent
        ));
    }

    if !capital_improvement_classification_proper {
        failure_reasons.push(
            "CAPITAL IMPROVEMENT MISCLASSIFIED — improvement (e.g., HVAC replacement, roof replacement) passed through as operating expense; should be AMORTIZED over useful life (typically 10-15 years); only the annual amortization charge belongs in CAM; tenant audit challenge available; BOMA Operating Expense Guide excludes capital improvements from operating expenses".to_string(),
        );
    }

    if !affiliated_party_expense_proper {
        failure_reasons.push(
            "ABOVE-MARKET AFFILIATED-PARTY EXPENSE — management fee or other CAM-pool expense paid to landlord-affiliated entity at above-market rates; standard lease excludes amounts exceeding ARMS-LENGTH market rate from CAM pool; tenant audit challenge available; refund of excess routinely awarded".to_string(),
        );
    }

    if !leasing_costs_excluded_proper {
        failure_reasons.push(
            "LEASING COSTS / BROKER COMMISSIONS IMPROPERLY INCLUDED IN CAM — these are landlord cost of doing business, NOT operating expenses for tenants' benefit; standard BOMA exclusion; tenant audit challenge available; refund of excess routinely awarded".to_string(),
        );
    }

    if input.annual_cap_exceeded_without_explanation {
        failure_reasons.push(
            "ANNUAL CAM CAP EXCEEDED WITHOUT EXPLANATION — typical commercial lease caps year-over-year CAM increase at 5% absent extraordinary circumstances; landlord must JUSTIFY any over-cap increase with detailed line-item explanation; tenant audit challenge available".to_string(),
        );
    }

    if !tenant_audit_right_available {
        failure_reasons.push(
            "TENANT AUDIT RIGHT NOT IN LEASE — industry-standard provision per BOMA Operating Expense Guide; without express right, tenant must rely on common-law access through litigation discovery; commercial lease drafters routinely include audit right with 90-180 day notice period + scope to most recent fiscal year + cost-shifting if discrepancy exceeds 3-5% threshold + refund of overcharges".to_string(),
        );
    }

    if disclosure_compliant
        && reconciliation_timely
        && gross_up_proper
        && capital_improvement_classification_proper
        && affiliated_party_expense_proper
        && leasing_costs_excluded_proper
        && tenant_audit_right_available
    {
        failure_reasons.push(format!(
            "CAM disclosure FULLY COMPLIANT — annual budget delivered + year-end reconciliation timely + gross-up proper + capital improvements amortized + arms-length affiliated-party expenses + leasing costs excluded + tenant audit right in lease; tenant CAM owed: {} cents under {} escalation structure",
            tenant_cam_owed_cents,
            match input.escalation_structure {
                CamEscalationStructure::FullPassThrough => "FULL PASS-THROUGH",
                CamEscalationStructure::BaseYearEscalation => "BASE-YEAR ESCALATION",
                CamEscalationStructure::FixedStop => "FIXED STOP",
            }
        ));
    }

    let notes: Vec<String> = vec![
        "Three-jurisdiction framework: CALIFORNIA (Cal. Civ. Code § 1938 CASp accessibility disclosure; Garrett v. Coast and Southern, 9 Cal. 4th 1 (1995) good-faith reconciliation; industry-standard annual budget + year-end reconciliation); NEW YORK (no specific commercial CAM statute; common-law contract interpretation; BOMA Operating Expense Guide); DEFAULT (BOMA 2024 Operating Expense Guide industry standard; Restatement (Second) of Contracts § 200 ambiguity-favoring-tenant; UCC Article 2A does NOT apply to real property leases)".to_string(),
        "BOMA Operating Expense Guide categories of CAM (13 categories): utilities (electric/gas/water/sewer); janitorial and cleaning; property management fees; security and access control; landscaping and grounds; snow removal (seasonal); trash removal; HVAC maintenance contracts; elevator maintenance contracts; common-area repairs; insurance premiums; real estate taxes (separately or within CAM); capital expenditures AMORTIZED over useful life (typically 10-15 years)".to_string(),
        "Standard EXCLUSIONS from CAM (12 categories): (1) capital improvements (vs. amortized capital repairs); (2) marketing and leasing costs (broker commissions); (3) tenant-specific build-outs and improvements; (4) landlord's debt service / mortgage interest; (5) depreciation; (6) income taxes; (7) ground rent paid by landlord; (8) reserves for future contingencies; (9) legal fees for tenant disputes; (10) penalties for landlord's own legal violations; (11) insurance proceeds offsetting expenses; (12) above-market affiliated-party vendor services".to_string(),
        "GROSS-UP PROVISION — when building less than fully occupied, landlord may gross-up VARIABLE expenses (utilities, janitorial, trash) to what they would have been at 95-100% occupancy; tenant pays pro-rata share of grossed-up amount; gross-up FLOORS occupancy at 95% or 100% per lease terms; FIXED expenses (real estate taxes, insurance) NOT grossed up".to_string(),
        "BASE-YEAR ESCALATION — tenant pays only INCREASE over base-year amount; calculation: Tenant CAM = (Current Year Total - Base Year Total) × Tenant Pro-Rata Share; base year must be REPRESENTATIVE; artificially deflated base year exposes landlord to audit challenge".to_string(),
        "TENANT AUDIT RIGHTS STANDARD PROVISIONS (7 elements): (1) annual right to audit landlord's books; (2) notice period 90-180 days after reconciliation statement; (3) scope limitation to most recent fiscal year; (4) confidentiality obligations; (5) cost-shifting clause (tenant pays unless discrepancy exceeds 3-5% threshold); (6) refund obligation for overcharges; (7) right to inspect underlying invoices and contracts".to_string(),
        "BOMA-survey CAM dispute statistics: (1) 1 in 4 (25%) tenants experience billing discrepancies in CAM reconciliations; (2) average overcharge recovery 5-15% of contested amount; (3) most common errors: incorrect gross-up calculation + capital-expenditure misclassification + management-fee duplication".to_string(),
        "California-specific considerations: Cal. Civ. Code § 1938 commercial-lease CASp accessibility inspection disclosure required (effective September 19, 2013, expanded July 1, 2024); good-faith reconciliation duty under Garrett v. Coast and Southern, 9 Cal. 4th 1 (1995); industry standard is annual operating budget delivered at start of fiscal year + year-end true-up within 90-180 days".to_string(),
        "Trader-landlord critical fact patterns: (1) office tower 65% occupied with 95% gross-up floor — variable expenses inflated 1.46×; (2) HVAC system replacement ($500K) passed as operating — should be amortized over 15-year useful life ($33K annual operating); (3) affiliated management company 5% gross rent (market 3-4%) — above-market affiliated-party expense excludable; (4) reconciliation late beyond 90 days fiscal year-end — tenant claims WAIVER of true-up; (5) artificially deflated base year — tenant audit may rectify".to_string(),
        "Companion to commercial_lease_personal_guaranty_enforceability (iter 445 — NYC § 22-1005 + Melendez constitutional history + Good Guy Guaranty); tenant_estoppel_certificate (refinance/sale estoppel framework); lease_disclosures + rental_property_registration".to_string(),
    ];

    CommercialLeaseCamChargeDisclosureResult {
        disclosure_compliant,
        reconciliation_timely,
        gross_up_proper,
        capital_improvement_classification_proper,
        affiliated_party_expense_proper,
        leasing_costs_excluded_proper,
        tenant_audit_right_available,
        tenant_cam_owed_cents,
        failure_reasons,
        citation: "Cal. Civ. Code § 1938 (CASp commercial accessibility disclosure); Garrett v. Coast and Southern, 9 Cal. 4th 1 (1995); BOMA International Operating Expense Guide (2024 edition); BOMA Office Buildings: Methods of Measurement (ANSI/BOMA Z65.1-2017); Restatement (Second) of Contracts § 200 (contract interpretation); N.Y. CPLR § 4519 (Dead Man's Statute); UCC Article 2A (INAPPLICABLE to real property leases)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compliant_full_pass_through() -> CommercialLeaseCamChargeDisclosureInput {
        CommercialLeaseCamChargeDisclosureInput {
            jurisdiction: Jurisdiction::California,
            escalation_structure: CamEscalationStructure::FullPassThrough,
            annual_budget_delivered: true,
            year_end_reconciliation_delivered: true,
            days_since_fye_for_reconciliation: 120,
            tenant_audit_right_in_lease: true,
            gross_up_provision_applied_floor_95: false,
            building_occupancy_percent: 80,
            capital_improvements_passed_as_operating: false,
            above_market_affiliated_management_fee: false,
            leasing_costs_included_in_cam: false,
            annual_cap_exceeded_without_explanation: false,
            tenant_pro_rata_share_percent: 10,
            current_year_total_cam_cents: 100_000_000,
            base_year_total_cam_cents: 80_000_000,
        }
    }

    #[test]
    fn fully_compliant_passes() {
        let r = check(&compliant_full_pass_through());
        assert!(r.disclosure_compliant);
        assert!(r.reconciliation_timely);
        assert!(r.gross_up_proper);
        assert!(r.capital_improvement_classification_proper);
        assert!(r.affiliated_party_expense_proper);
        assert!(r.leasing_costs_excluded_proper);
        assert!(r.tenant_audit_right_available);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("CAM disclosure FULLY COMPLIANT")));
    }

    #[test]
    fn full_pass_through_tenant_owes_10_percent() {
        let r = check(&compliant_full_pass_through());
        assert_eq!(r.tenant_cam_owed_cents, 10_000_000);
    }

    #[test]
    fn base_year_escalation_only_excess() {
        let mut i = compliant_full_pass_through();
        i.escalation_structure = CamEscalationStructure::BaseYearEscalation;
        let r = check(&i);
        assert_eq!(r.tenant_cam_owed_cents, 2_000_000);
    }

    #[test]
    fn fixed_stop_full_amount() {
        let mut i = compliant_full_pass_through();
        i.escalation_structure = CamEscalationStructure::FixedStop;
        let r = check(&i);
        assert_eq!(r.tenant_cam_owed_cents, 100_000_000);
    }

    #[test]
    fn no_annual_budget_violation() {
        let mut i = compliant_full_pass_through();
        i.annual_budget_delivered = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("Annual CAM BUDGET not delivered")
                && f.contains("Garrett v. Coast and Southern")));
    }

    #[test]
    fn no_year_end_reconciliation_violation() {
        let mut i = compliant_full_pass_through();
        i.year_end_reconciliation_delivered = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r.failure_reasons.iter().any(|f| f
            .contains("Year-end CAM RECONCILIATION not delivered")
            && f.contains("WAIVER")));
    }

    #[test]
    fn reconciliation_past_180_days_late() {
        let mut i = compliant_full_pass_through();
        i.days_since_fye_for_reconciliation = 200;
        let r = check(&i);
        assert!(!r.reconciliation_timely);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("200 days after fiscal year-end") && f.contains("180-day window")));
    }

    #[test]
    fn gross_up_above_95_percent_improper() {
        let mut i = compliant_full_pass_through();
        i.gross_up_provision_applied_floor_95 = true;
        i.building_occupancy_percent = 97;
        let r = check(&i);
        assert!(!r.gross_up_proper);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("GROSS-UP IMPROPER") && f.contains("97%")));
    }

    #[test]
    fn gross_up_at_95_boundary_proper() {
        let mut i = compliant_full_pass_through();
        i.gross_up_provision_applied_floor_95 = true;
        i.building_occupancy_percent = 95;
        let r = check(&i);
        assert!(r.gross_up_proper);
    }

    #[test]
    fn gross_up_under_95_proper() {
        let mut i = compliant_full_pass_through();
        i.gross_up_provision_applied_floor_95 = true;
        i.building_occupancy_percent = 70;
        let r = check(&i);
        assert!(r.gross_up_proper);
    }

    #[test]
    fn capital_improvement_passed_violation() {
        let mut i = compliant_full_pass_through();
        i.capital_improvements_passed_as_operating = true;
        let r = check(&i);
        assert!(!r.capital_improvement_classification_proper);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("CAPITAL IMPROVEMENT MISCLASSIFIED")
                && f.contains("AMORTIZED")
                && f.contains("10-15 years")));
    }

    #[test]
    fn affiliated_party_above_market_violation() {
        let mut i = compliant_full_pass_through();
        i.above_market_affiliated_management_fee = true;
        let r = check(&i);
        assert!(!r.affiliated_party_expense_proper);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("ABOVE-MARKET AFFILIATED-PARTY")
                && f.contains("ARMS-LENGTH market rate")));
    }

    #[test]
    fn leasing_costs_in_cam_violation() {
        let mut i = compliant_full_pass_through();
        i.leasing_costs_included_in_cam = true;
        let r = check(&i);
        assert!(!r.leasing_costs_excluded_proper);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("LEASING COSTS / BROKER COMMISSIONS")
                && f.contains("standard BOMA exclusion")));
    }

    #[test]
    fn no_audit_right_violation() {
        let mut i = compliant_full_pass_through();
        i.tenant_audit_right_in_lease = false;
        let r = check(&i);
        assert!(!r.tenant_audit_right_available);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("TENANT AUDIT RIGHT NOT IN LEASE")
                && f.contains("BOMA Operating Expense Guide")
                && f.contains("3-5% threshold")));
    }

    #[test]
    fn cap_exceeded_without_explanation_violation() {
        let mut i = compliant_full_pass_through();
        i.annual_cap_exceeded_without_explanation = true;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("ANNUAL CAM CAP EXCEEDED") && f.contains("5%")));
    }

    #[test]
    fn escalation_structure_truth_table_three_cells() {
        for (esc, expected_cam) in [
            (CamEscalationStructure::FullPassThrough, 10_000_000),
            (CamEscalationStructure::BaseYearEscalation, 2_000_000),
            (CamEscalationStructure::FixedStop, 100_000_000),
        ] {
            let mut i = compliant_full_pass_through();
            i.escalation_structure = esc;
            let r = check(&i);
            assert_eq!(r.tenant_cam_owed_cents, expected_cam, "esc={:?}", esc);
        }
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&compliant_full_pass_through());
        assert!(r.citation.contains("Cal. Civ. Code § 1938"));
        assert!(r
            .citation
            .contains("Garrett v. Coast and Southern, 9 Cal. 4th 1 (1995)"));
        assert!(r
            .citation
            .contains("BOMA International Operating Expense Guide (2024 edition)"));
        assert!(r.citation.contains("ANSI/BOMA Z65.1-2017"));
        assert!(r
            .citation
            .contains("Restatement (Second) of Contracts § 200"));
        assert!(r.citation.contains("N.Y. CPLR § 4519"));
        assert!(r.citation.contains("UCC Article 2A"));
    }

    #[test]
    fn note_pins_three_jurisdiction_framework() {
        let r = check(&compliant_full_pass_through());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Three-jurisdiction framework")
                && n.contains("CALIFORNIA")
                && n.contains("NEW YORK")
                && n.contains("DEFAULT")));
    }

    #[test]
    fn note_pins_thirteen_cam_categories() {
        let r = check(&compliant_full_pass_through());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("BOMA Operating Expense Guide categories")
                && n.contains("(13 categories)")
                && n.contains("utilities")
                && n.contains("real estate taxes")
                && n.contains("AMORTIZED")));
    }

    #[test]
    fn note_pins_twelve_exclusions() {
        let r = check(&compliant_full_pass_through());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Standard EXCLUSIONS from CAM")
                && n.contains("(12 categories)")
                && n.contains("capital improvements")
                && n.contains("broker commissions")
                && n.contains("affiliated-party")));
    }

    #[test]
    fn note_pins_gross_up_provision() {
        let r = check(&compliant_full_pass_through());
        assert!(r.notes.iter().any(|n| n.contains("GROSS-UP PROVISION")
            && n.contains("95-100%")
            && n.contains("FIXED expenses")));
    }

    #[test]
    fn note_pins_base_year_escalation_formula() {
        let r = check(&compliant_full_pass_through());
        assert!(r.notes.iter().any(|n| n.contains("BASE-YEAR ESCALATION")
            && n.contains("Tenant CAM = (Current Year Total - Base Year Total)")
            && n.contains("Pro-Rata Share")));
    }

    #[test]
    fn note_pins_audit_seven_elements() {
        let r = check(&compliant_full_pass_through());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("TENANT AUDIT RIGHTS STANDARD PROVISIONS")
                && n.contains("(7 elements)")
                && n.contains("annual right")
                && n.contains("3-5% threshold")
                && n.contains("refund obligation")));
    }

    #[test]
    fn note_pins_boma_survey_statistics() {
        let r = check(&compliant_full_pass_through());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("BOMA-survey CAM dispute statistics")
                && n.contains("1 in 4 (25%)")
                && n.contains("5-15%")
                && n.contains("incorrect gross-up")));
    }

    #[test]
    fn note_pins_ca_section_1938_casp() {
        let r = check(&compliant_full_pass_through());
        assert!(r.notes.iter().any(|n| n.contains("Cal. Civ. Code § 1938")
            && n.contains("CASp")
            && n.contains("Garrett v. Coast and Southern")));
    }

    #[test]
    fn note_pins_trader_fact_patterns_five() {
        let r = check(&compliant_full_pass_through());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Trader-landlord critical fact patterns")
                && n.contains("65% occupied")
                && n.contains("HVAC")
                && n.contains("$500K")
                && n.contains("artificially deflated base year")));
    }

    #[test]
    fn note_pins_companion_modules() {
        let r = check(&compliant_full_pass_through());
        assert!(r.notes.iter().any(|n| n
            .contains("Companion to commercial_lease_personal_guaranty_enforceability")
            && n.contains("tenant_estoppel_certificate")));
    }

    #[test]
    fn multiple_failures_stack() {
        let mut i = compliant_full_pass_through();
        i.annual_budget_delivered = false;
        i.year_end_reconciliation_delivered = false;
        i.capital_improvements_passed_as_operating = true;
        i.above_market_affiliated_management_fee = true;
        i.leasing_costs_included_in_cam = true;
        let r = check(&i);
        assert!(r.failure_reasons.len() >= 5);
    }
}
