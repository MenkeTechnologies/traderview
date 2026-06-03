//! Rental property lead service line / lead pipe disclosure
//! compliance — when a trader-landlord operating a property
//! served by a water service line that contains lead must
//! notify tenants of lead service line presence and
//! replacement plans under EPA Lead and Copper Rule
//! Revisions (LCRR, eff. October 16, 2024) and Lead and
//! Copper Rule Improvements (LCRI, eff. November 1, 2027).
//! Trader-landlord operational concern: undisclosed lead
//! service line creates breach of warranty of habitability
//! and per-tenant statutory damages and tort liability for
//! lead poisoning and post-Flint heightened regulatory
//! scrutiny. Distinct from siblings rental_underground_
//! storage_tank_disclosure UST and rental_basement_water_
//! intrusion_disclosure water/mold and rental_sinkhole_
//! disclosure and federal § 4852d lead-based paint
//! disclosure paint not pipes.
//!
//! **Three regimes**:
//!
//! **Federal — EPA Lead and Copper Rule Revisions (LCRR,
//! 40 CFR Part 141 Subpart I) + Lead and Copper Rule
//! Improvements (LCRI)**:
//! - **LCRR compliance date: October 16, 2024**.
//! - Public water systems must complete initial **service
//!   line inventory** + notification to persons served of
//!   known or potential lead service lines.
//! - **Tier 1 public notification within 24 hours** after
//!   water system learns of lead action level exceedance.
//! - **LCRI compliance date: November 1, 2027**.
//! - LCRI reduces lead action level to **10 ppb** (from
//!   prior 15 ppb).
//! - **Community water systems must replace ALL lead
//!   service lines by November 1, 2037** (10-year
//!   mandate).
//! - LCRI mandates information accessible to consumers
//!   including renters + limited English proficiency.
//!
//! **Illinois — Lead Service Line Replacement and
//! Notification Act (415 ILCS 5/17.12, eff. January 1,
//! 2022)**:
//! - Each community water supply must maintain a **lead
//!   service line inventory** and submit to IL EPA by
//!   April 15, 2023.
//! - **All lead service lines must be replaced** by
//!   community water supplies on a defined schedule (50-
//!   year statewide goal; specific milestones by
//!   population).
//! - Landlords must **provide written notice** to tenants
//!   of known or potential lead service line within **30
//!   days** of receiving water supply notification.
//! - Civil penalty under IL EPA Act § 42 — up to **$50,000
//!   per violation** + $10,000 per day continuing.
//!
//! **New Jersey — Lead Service Line Replacement Act
//! (P.L. 2021, c.183, N.J.S.A. 58:12A-40 et seq.)**:
//! - All lead service lines must be replaced **within 10
//!   years** of enactment (by July 2031).
//! - Property owners must **disclose lead service line
//!   presence to tenants** before lease signing.
//! - NJDEP administers replacement program; cost-sharing
//!   between water utility and property owner.
//! - Civil penalty under N.J.S.A. 58:10A-10 — up to
//!   $50,000 per violation per day for failure to
//!   replace.
//!
//! Citations: 40 CFR Part 141 Subpart I (EPA Lead and
//! Copper Rule Revisions + Lead and Copper Rule
//! Improvements); EPA LCRR October 16 2024 effective
//! date; EPA LCRI October 8 2024 final rule and November 1
//! 2027 compliance date; 415 ILCS 5/17.12 (Illinois Lead
//! Service Line Replacement and Notification Act); N.J.S.A.
//! 58:12A-40 et seq. (NJ Lead Service Line Replacement
//! Act P.L. 2021 c.183); Safe Drinking Water Act (42 USC
//! § 300f et seq.).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Federal,
    Illinois,
    NewJersey,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceLineStatus {
    /// Known lead service line.
    KnownLead,
    /// Potential lead service line (status unknown but
    /// possibly lead).
    PotentialLead,
    /// Galvanized service line requiring replacement
    /// (post-LCRI).
    GalvanizedRequiringReplacement,
    /// Non-lead service line (copper, PEX, ductile iron).
    NonLead,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RentalLeadPipeDisclosureInput {
    pub regime: Regime,
    pub service_line_status: ServiceLineStatus,
    /// Whether public water system notified of known or
    /// potential lead service line under LCRR.
    pub water_system_notified: bool,
    /// Whether landlord notified tenants within required
    /// timeframe (30 days IL).
    pub landlord_notified_tenants: bool,
    /// Days from water system notification to landlord
    /// tenant notification (IL 30-day rule).
    pub days_from_water_system_to_tenant_notice: u32,
    /// Whether Tier 1 24-hour notice was provided when lead
    /// action level exceedance learned (LCRR/LCRI).
    pub tier_1_24_hour_notice_provided: bool,
    /// Whether lead service line inventory was completed
    /// and shared with tenant.
    pub service_line_inventory_completed: bool,
    /// Whether landlord disclosed lead service line BEFORE
    /// lease signing (NJ requirement).
    pub nj_pre_lease_disclosure: bool,
    /// Civil penalty amount in cents (IL $50K cap / NJ $50K
    /// per day cap).
    pub civil_penalty_cents: u64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RentalLeadPipeDisclosureResult {
    pub disclosure_compliant: bool,
    pub lcrr_engaged: bool,
    pub lcri_engaged: bool,
    pub thirty_day_notice_compliant: bool,
    pub tier_1_24_hour_compliant: bool,
    pub civil_penalty_in_range: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &RentalLeadPipeDisclosureInput) -> RentalLeadPipeDisclosureResult {
    match input.regime {
        Regime::Federal => check_federal(input),
        Regime::Illinois => check_il(input),
        Regime::NewJersey => check_nj(input),
    }
}

fn check_federal(
    input: &RentalLeadPipeDisclosureInput,
) -> RentalLeadPipeDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "40 CFR Part 141 Subpart I — EPA Lead and Copper Rule Revisions (LCRR) compliance date October 16, 2024; public water systems must complete initial service line inventory + notification to persons served of known or potential lead service lines".to_string(),
        "LCRR Tier 1 public notification within 24 hours after water system learns of lead action level exceedance".to_string(),
        "EPA Lead and Copper Rule Improvements (LCRI) compliance date November 1, 2027; reduces lead action level to 10 ppb from prior 15 ppb".to_string(),
        "LCRI mandates community water systems replace ALL lead service lines by November 1, 2037 (10-year replacement mandate)".to_string(),
        "LCRI mandates information accessible to consumers INCLUDING renters and limited English proficiency individuals".to_string(),
        "Safe Drinking Water Act 42 USC § 300f et seq. — federal statutory framework underlying EPA regulations".to_string(),
    ];

    let lead_present = matches!(
        input.service_line_status,
        ServiceLineStatus::KnownLead
            | ServiceLineStatus::PotentialLead
            | ServiceLineStatus::GalvanizedRequiringReplacement
    );

    if lead_present && !input.service_line_inventory_completed {
        violations.push(
            "40 CFR Part 141 Subpart I (LCRR) — public water systems must complete service line inventory by October 16, 2024".to_string(),
        );
    }

    if lead_present && !input.tier_1_24_hour_notice_provided {
        violations.push(
            "40 CFR Part 141 Subpart I (LCRR) — Tier 1 public notification required within 24 hours after water system learns of lead action level exceedance".to_string(),
        );
    }

    RentalLeadPipeDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        lcrr_engaged: true,
        lcri_engaged: true,
        thirty_day_notice_compliant: true,
        tier_1_24_hour_compliant: input.tier_1_24_hour_notice_provided,
        civil_penalty_in_range: true,
        violations,
        citation: "40 CFR Part 141 Subpart I (EPA Lead and Copper Rule Revisions LCRR + Lead and Copper Rule Improvements LCRI); Safe Drinking Water Act 42 USC § 300f et seq.",
        notes,
    }
}

fn check_il(input: &RentalLeadPipeDisclosureInput) -> RentalLeadPipeDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "415 ILCS 5/17.12 (Illinois Lead Service Line Replacement and Notification Act, eff. January 1, 2022) — community water supplies must maintain lead service line inventory and submit to IL EPA by April 15, 2023".to_string(),
        "415 ILCS 5/17.12 — landlords must provide written notice to tenants of known or potential lead service line within 30 DAYS of receiving water supply notification".to_string(),
        "415 ILCS 5/17.12 — all lead service lines must be replaced by community water supplies on defined schedule (50-year statewide goal; population-based milestones)".to_string(),
        "IL EPA Act § 42 — civil penalty up to $50,000 per violation + $10,000 per day continuing".to_string(),
        "Illinois regime distinct from federal LCRR/LCRI in 30-day tenant notification requirement and $50,000 penalty cap".to_string(),
    ];

    let lead_present = matches!(
        input.service_line_status,
        ServiceLineStatus::KnownLead
            | ServiceLineStatus::PotentialLead
            | ServiceLineStatus::GalvanizedRequiringReplacement
    );

    let thirty_day_compliant = !lead_present
        || !input.water_system_notified
        || input.days_from_water_system_to_tenant_notice <= 30;

    if lead_present
        && input.water_system_notified
        && !input.landlord_notified_tenants
    {
        violations.push(
            "415 ILCS 5/17.12 — landlord must provide written notice to tenants of known or potential lead service line".to_string(),
        );
    }

    if lead_present
        && input.water_system_notified
        && input.days_from_water_system_to_tenant_notice > 30
    {
        violations.push(
            "415 ILCS 5/17.12 — landlord must notify tenants within 30 DAYS of receiving water supply notification".to_string(),
        );
    }

    const IL_PENALTY_CAP_CENTS: u64 = 5_000_000;
    let penalty_in_range = input.civil_penalty_cents == 0
        || input.civil_penalty_cents <= IL_PENALTY_CAP_CENTS;
    if !penalty_in_range {
        violations.push(
            "IL EPA Act § 42 — civil penalty capped at $50,000 per violation".to_string(),
        );
    }

    RentalLeadPipeDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        lcrr_engaged: true,
        lcri_engaged: true,
        thirty_day_notice_compliant: thirty_day_compliant,
        tier_1_24_hour_compliant: true,
        civil_penalty_in_range: penalty_in_range,
        violations,
        citation: "415 ILCS 5/17.12 (Illinois Lead Service Line Replacement and Notification Act); IL EPA Act § 42; 40 CFR Part 141 Subpart I (LCRR/LCRI)",
        notes,
    }
}

fn check_nj(input: &RentalLeadPipeDisclosureInput) -> RentalLeadPipeDisclosureResult {
    let mut violations: Vec<String> = Vec::new();
    let notes: Vec<String> = vec![
        "N.J.S.A. 58:12A-40 et seq. (NJ Lead Service Line Replacement Act, P.L. 2021, c.183) — all lead service lines must be replaced within 10 years of enactment (by July 2031)".to_string(),
        "NJ Lead Service Line Replacement Act — property owners must disclose lead service line presence to tenants BEFORE LEASE SIGNING".to_string(),
        "NJDEP administers replacement program; cost-sharing between water utility and property owner".to_string(),
        "N.J.S.A. 58:10A-10 — civil penalty up to $50,000 per violation PER DAY for failure to replace".to_string(),
        "NJ regime distinct from IL in pre-lease disclosure requirement (NJ before signing vs IL within 30 days of water system notice) and per-day penalty escalation".to_string(),
    ];

    let lead_present = matches!(
        input.service_line_status,
        ServiceLineStatus::KnownLead
            | ServiceLineStatus::PotentialLead
            | ServiceLineStatus::GalvanizedRequiringReplacement
    );

    if lead_present && !input.nj_pre_lease_disclosure {
        violations.push(
            "N.J.S.A. 58:12A-40 et seq. — property owner must disclose lead service line presence to tenants BEFORE LEASE SIGNING".to_string(),
        );
    }

    const NJ_PENALTY_PER_DAY_CAP_CENTS: u64 = 5_000_000;
    let penalty_in_range = input.civil_penalty_cents == 0
        || input.civil_penalty_cents <= NJ_PENALTY_PER_DAY_CAP_CENTS;
    if !penalty_in_range {
        violations.push(
            "N.J.S.A. 58:10A-10 — civil penalty capped at $50,000 per violation PER DAY for failure to replace".to_string(),
        );
    }

    RentalLeadPipeDisclosureResult {
        disclosure_compliant: violations.is_empty(),
        lcrr_engaged: true,
        lcri_engaged: true,
        thirty_day_notice_compliant: true,
        tier_1_24_hour_compliant: true,
        civil_penalty_in_range: penalty_in_range,
        violations,
        citation: "N.J.S.A. 58:12A-40 et seq. (NJ Lead Service Line Replacement Act P.L. 2021 c.183); N.J.S.A. 58:10A-10; 40 CFR Part 141 Subpart I (LCRR/LCRI)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fed_clean() -> RentalLeadPipeDisclosureInput {
        RentalLeadPipeDisclosureInput {
            regime: Regime::Federal,
            service_line_status: ServiceLineStatus::KnownLead,
            water_system_notified: true,
            landlord_notified_tenants: true,
            days_from_water_system_to_tenant_notice: 7,
            tier_1_24_hour_notice_provided: true,
            service_line_inventory_completed: true,
            nj_pre_lease_disclosure: false,
            civil_penalty_cents: 0,
        }
    }

    fn il_clean() -> RentalLeadPipeDisclosureInput {
        let mut i = fed_clean();
        i.regime = Regime::Illinois;
        i
    }

    fn nj_clean() -> RentalLeadPipeDisclosureInput {
        let mut i = fed_clean();
        i.regime = Regime::NewJersey;
        i.nj_pre_lease_disclosure = true;
        i
    }

    #[test]
    fn fed_clean_compliant() {
        let r = check(&fed_clean());
        assert!(r.disclosure_compliant);
        assert!(r.lcrr_engaged);
        assert!(r.lcri_engaged);
    }

    #[test]
    fn fed_no_inventory_violation() {
        let mut i = fed_clean();
        i.service_line_inventory_completed = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("LCRR") && v.contains("service line inventory")));
    }

    #[test]
    fn fed_no_tier_1_notice_violation() {
        let mut i = fed_clean();
        i.tier_1_24_hour_notice_provided = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(!r.tier_1_24_hour_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("Tier 1") && v.contains("24 hours")));
    }

    #[test]
    fn fed_non_lead_no_lcrr_obligations() {
        let mut i = fed_clean();
        i.service_line_status = ServiceLineStatus::NonLead;
        i.service_line_inventory_completed = false;
        i.tier_1_24_hour_notice_provided = false;
        let r = check(&i);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn il_clean_compliant() {
        let r = check(&il_clean());
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn il_no_tenant_notice_violation() {
        let mut i = il_clean();
        i.landlord_notified_tenants = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("415 ILCS 5/17.12") && v.contains("written notice")));
    }

    #[test]
    fn il_30_day_boundary_compliant() {
        let mut i = il_clean();
        i.days_from_water_system_to_tenant_notice = 30;
        let r = check(&i);
        assert!(r.thirty_day_notice_compliant);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn il_31_day_violation() {
        let mut i = il_clean();
        i.days_from_water_system_to_tenant_notice = 31;
        let r = check(&i);
        assert!(!r.thirty_day_notice_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("415 ILCS 5/17.12") && v.contains("30 DAYS")));
    }

    #[test]
    fn il_penalty_at_50k_cap_compliant() {
        let mut i = il_clean();
        i.civil_penalty_cents = 5_000_000;
        let r = check(&i);
        assert!(r.civil_penalty_in_range);
    }

    #[test]
    fn il_penalty_above_50k_cap_violation() {
        let mut i = il_clean();
        i.civil_penalty_cents = 5_000_001;
        let r = check(&i);
        assert!(!r.civil_penalty_in_range);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("IL EPA Act § 42") && v.contains("$50,000")));
    }

    #[test]
    fn nj_clean_compliant() {
        let r = check(&nj_clean());
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn nj_no_pre_lease_disclosure_violation() {
        let mut i = nj_clean();
        i.nj_pre_lease_disclosure = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("N.J.S.A. 58:12A-40")
                && v.contains("BEFORE LEASE SIGNING")));
    }

    #[test]
    fn nj_penalty_at_50k_per_day_cap_compliant() {
        let mut i = nj_clean();
        i.civil_penalty_cents = 5_000_000;
        let r = check(&i);
        assert!(r.civil_penalty_in_range);
    }

    #[test]
    fn nj_penalty_above_50k_per_day_cap_violation() {
        let mut i = nj_clean();
        i.civil_penalty_cents = 5_000_001;
        let r = check(&i);
        assert!(!r.civil_penalty_in_range);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("58:10A-10") && v.contains("PER DAY")));
    }

    #[test]
    fn nj_galvanized_requires_replacement_disclosure() {
        let mut i = nj_clean();
        i.service_line_status = ServiceLineStatus::GalvanizedRequiringReplacement;
        i.nj_pre_lease_disclosure = false;
        let r = check(&i);
        assert!(!r.disclosure_compliant);
    }

    #[test]
    fn nj_non_lead_no_disclosure_required() {
        let mut i = nj_clean();
        i.service_line_status = ServiceLineStatus::NonLead;
        i.nj_pre_lease_disclosure = false;
        let r = check(&i);
        assert!(r.disclosure_compliant);
    }

    #[test]
    fn citation_pins_fed_authority() {
        let r = check(&fed_clean());
        assert!(r.citation.contains("40 CFR Part 141 Subpart I"));
        assert!(r.citation.contains("LCRR"));
        assert!(r.citation.contains("LCRI"));
        assert!(r.citation.contains("42 USC § 300f"));
    }

    #[test]
    fn citation_pins_il_authority() {
        let r = check(&il_clean());
        assert!(r.citation.contains("415 ILCS 5/17.12"));
        assert!(r.citation.contains("IL EPA Act § 42"));
    }

    #[test]
    fn citation_pins_nj_authority() {
        let r = check(&nj_clean());
        assert!(r.citation.contains("58:12A-40"));
        assert!(r.citation.contains("58:10A-10"));
        assert!(r.citation.contains("P.L. 2021 c.183"));
    }

    #[test]
    fn note_pins_lcrr_october_2024_compliance_date() {
        let r = check(&fed_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("October 16, 2024") && n.contains("LCRR")));
    }

    #[test]
    fn note_pins_lcri_november_2027_compliance_date() {
        let r = check(&fed_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("November 1, 2027") && n.contains("LCRI")));
    }

    #[test]
    fn note_pins_lcri_2037_replacement_mandate() {
        let r = check(&fed_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("November 1, 2037") && n.contains("ALL lead service lines")));
    }

    #[test]
    fn note_pins_lcri_10_ppb_action_level() {
        let r = check(&fed_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("10 ppb") && n.contains("15 ppb")));
    }

    #[test]
    fn note_pins_il_30_day_tenant_notice() {
        let r = check(&il_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("30 DAYS") && n.contains("water supply notification")));
    }

    #[test]
    fn note_pins_il_50000_penalty() {
        let r = check(&il_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("$50,000") && n.contains("IL EPA Act § 42")));
    }

    #[test]
    fn note_pins_nj_pre_lease_disclosure() {
        let r = check(&nj_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("BEFORE LEASE SIGNING")));
    }

    #[test]
    fn note_pins_nj_2031_replacement_deadline() {
        let r = check(&nj_clean());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("July 2031")));
    }

    #[test]
    fn il_uniquely_requires_30_day_tenant_notice_invariant() {
        let mut i_il = il_clean();
        i_il.days_from_water_system_to_tenant_notice = 60;
        let r_il = check(&i_il);
        assert!(!r_il.disclosure_compliant);

        let mut i_fed = fed_clean();
        i_fed.days_from_water_system_to_tenant_notice = 60;
        let r_fed = check(&i_fed);
        assert!(r_fed.disclosure_compliant);
    }

    #[test]
    fn nj_uniquely_requires_pre_lease_disclosure_invariant() {
        let mut i_nj = nj_clean();
        i_nj.nj_pre_lease_disclosure = false;
        let r_nj = check(&i_nj);
        assert!(!r_nj.disclosure_compliant);

        let mut i_il = il_clean();
        i_il.nj_pre_lease_disclosure = false;
        let r_il = check(&i_il);
        assert!(r_il.disclosure_compliant);
    }

    #[test]
    fn service_line_status_truth_table_four_cells() {
        for (status, exp_lead_present) in [
            (ServiceLineStatus::KnownLead, true),
            (ServiceLineStatus::PotentialLead, true),
            (ServiceLineStatus::GalvanizedRequiringReplacement, true),
            (ServiceLineStatus::NonLead, false),
        ] {
            let mut i = nj_clean();
            i.service_line_status = status;
            i.nj_pre_lease_disclosure = false;
            let r = check(&i);
            if exp_lead_present {
                assert!(!r.disclosure_compliant);
            } else {
                assert!(r.disclosure_compliant);
            }
        }
    }

    #[test]
    fn multiple_il_violations_stack() {
        let mut i = il_clean();
        i.landlord_notified_tenants = false;
        i.days_from_water_system_to_tenant_notice = 60;
        i.civil_penalty_cents = 10_000_000;
        let r = check(&i);
        assert_eq!(r.violations.len(), 3);
    }

    #[test]
    fn lcrr_and_lcri_engaged_invariant_across_regimes() {
        for regime in [Regime::Federal, Regime::Illinois, Regime::NewJersey] {
            let i = match regime {
                Regime::Federal => fed_clean(),
                Regime::Illinois => il_clean(),
                Regime::NewJersey => nj_clean(),
            };
            let r = check(&i);
            assert!(r.lcrr_engaged);
            assert!(r.lcri_engaged);
            let _ = i.service_line_status;
        }
    }
}
