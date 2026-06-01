//! State demolition-tenant-notice landlord compliance check.
//!
//! Three jurisdictions have substantive statutory notice requirements
//! when a landlord intends to demolish a residential rental property.
//! Distinct from `owner_move_in_eviction` (which addresses landlord
//! moving INTO the unit) and `tenant_relocation_assistance` (which
//! addresses the DOLLAR amount owed); this module addresses the
//! NOTICE PERIOD that must be given BEFORE the tenant must vacate.
//!
//! **California (Ellis Act — Cal. Govt Code § 7060 et seq.)** —
//! enacted 1986 to allow rental-property owners to exit the residential
//! rental market (including via demolition). Notice mechanics:
//! - **120 days** written notice for standard tenants (§ 7060.4(a)).
//! - **365 days (1 year) extension** for tenants who are ≥ 62 OR
//!   disabled AND have lived in the unit ≥ 1 year (§ 7060.4(b)).
//! - **Six-month minimum** between notice service and start of
//!   construction/demolition activities (per most local ordinances
//!   implementing Ellis).
//! - Relocation assistance under local ordinances paid 50% at notice
//!   service + 50% at vacate (LA RSO + SF + Berkeley pattern).
//!
//! **Oregon (ORS 90.427 — termination without tenant cause for
//! demolition)** — 90-day notice for landlord-cause termination
//! including conversion/demolition. ORS 90.323 first-year-prohibition
//! still applies — landlord may not demo-terminate in first 12 months.
//! Relocation assistance per PCC 30.01.085 (Portland-specific) modeled
//! separately in `tenant_relocation_assistance`.
//!
//! **Washington (RCW 59.18.650 — substantial rehabilitation /
//! demolition)** — uniform 120-day written notice. Notice must state
//! the specific landlord-cause reason (demolition / change-of-use /
//! substantial rehabilitation requiring tenant to vacate ≥ 30 days).
//!
//! **Default** — no statewide demolition-specific tenant notice statute.
//! General lease-term and just-cause-eviction rules (where applicable)
//! apply.
//!
//! Citations: Cal. Govt Code § 7060 et seq. (Ellis Act); § 7060.4(a)
//! (120-day default notice); § 7060.4(b) (365-day extension); ORS 90.427
//! (landlord-cause termination); ORS 90.323(3) (first-year prohibition);
//! RCW 59.18.650 (substantial-rehab / demo 120-day notice).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    CaliforniaEllisAct,
    Oregon,
    Washington,
    Default,
}

impl Regime {
    pub fn for_state(state: &str) -> Self {
        match state.trim().to_ascii_uppercase().as_str() {
            "CA" => Self::CaliforniaEllisAct,
            "OR" => Self::Oregon,
            "WA" => Self::Washington,
            _ => Self::Default,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DemolitionNoticeInput {
    pub regime: Regime,
    pub notice_days_provided: u32,
    /// Tenant age (drives CA Ellis-Act § 7060.4(b) extension to 365 days).
    pub tenant_age: u32,
    /// Tenant has a disability under FEHA / ADA. Drives CA § 7060.4(b).
    pub tenant_disabled: bool,
    /// Months since the tenancy began. Drives both CA § 7060.4(b)
    /// (one-year residency requirement for 365-day extension) and OR
    /// § 90.323(3) (first-year-prohibition floor).
    pub months_since_tenancy_started: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationType {
    None,
    InsufficientNotice,
    /// Oregon first-year prohibition (ORS 90.323(3)) blocks any termination
    /// during the first 12 months of a non-week-to-week tenancy.
    FirstYearProhibited,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DemolitionNoticeResult {
    pub regime: Regime,
    pub required_notice_days: u32,
    /// Whether the CA Ellis-Act 365-day extension applied (tenant ≥ 62
    /// OR disabled AND ≥ 1 year residency).
    pub ellis_365_day_extension_applied: bool,
    pub in_first_year: bool,
    pub violation: ViolationType,
    pub landlord_compliant: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn check(input: &DemolitionNoticeInput) -> DemolitionNoticeResult {
    match input.regime {
        Regime::CaliforniaEllisAct => ca_check(input),
        Regime::Oregon => or_check(input),
        Regime::Washington => wa_check(input),
        Regime::Default => default_check(input),
    }
}

fn ca_check(input: &DemolitionNoticeInput) -> DemolitionNoticeResult {
    // § 7060.4(b): 365-day extension if tenant is ≥ 62 OR disabled AND
    // has been in the unit ≥ 1 year.
    let qualifies_for_extension =
        (input.tenant_age >= 62 || input.tenant_disabled) && input.months_since_tenancy_started >= 12;
    let required = if qualifies_for_extension { 365 } else { 120 };
    let citation = if qualifies_for_extension {
        "Cal. Govt Code § 7060.4(b) — 365-day notice extension for tenants ≥ 62 or disabled with ≥ 1 year residency"
    } else {
        "Cal. Govt Code § 7060.4(a) — 120-day notice for standard tenants"
    };
    if input.notice_days_provided < required {
        return DemolitionNoticeResult {
            regime: Regime::CaliforniaEllisAct,
            required_notice_days: required,
            ellis_365_day_extension_applied: qualifies_for_extension,
            in_first_year: false,
            violation: ViolationType::InsufficientNotice,
            landlord_compliant: false,
            citation,
            note: format!(
                "Required {} days; provided {} days. Insufficient notice under Ellis Act.",
                required, input.notice_days_provided
            ),
        };
    }
    DemolitionNoticeResult {
        regime: Regime::CaliforniaEllisAct,
        required_notice_days: required,
        ellis_365_day_extension_applied: qualifies_for_extension,
        in_first_year: false,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation,
        note: format!(
            "Notice of {} days satisfies the {}-day Ellis Act requirement.",
            input.notice_days_provided, required
        ),
    }
}

fn or_check(input: &DemolitionNoticeInput) -> DemolitionNoticeResult {
    // ORS 90.323(3): no termination in first 12 months. This first-year
    // prohibition fires regardless of how much notice is given.
    if input.months_since_tenancy_started < 12 {
        return DemolitionNoticeResult {
            regime: Regime::Oregon,
            required_notice_days: 90,
            ellis_365_day_extension_applied: false,
            in_first_year: true,
            violation: ViolationType::FirstYearProhibited,
            landlord_compliant: false,
            citation: "ORS 90.323(3) — no landlord-cause termination during first 12 months of non-week-to-week tenancy",
            note: format!(
                "Tenancy has been in effect {} months — within the 12-month first-year prohibition window. Demolition termination blocked regardless of notice.",
                input.months_since_tenancy_started
            ),
        };
    }
    if input.notice_days_provided < 90 {
        return DemolitionNoticeResult {
            regime: Regime::Oregon,
            required_notice_days: 90,
            ellis_365_day_extension_applied: false,
            in_first_year: false,
            violation: ViolationType::InsufficientNotice,
            landlord_compliant: false,
            citation: "ORS 90.427 — 90 days written notice required for landlord-cause termination including demolition",
            note: format!(
                "Required 90 days; provided {} days. Insufficient notice under ORS 90.427.",
                input.notice_days_provided
            ),
        };
    }
    DemolitionNoticeResult {
        regime: Regime::Oregon,
        required_notice_days: 90,
        ellis_365_day_extension_applied: false,
        in_first_year: false,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation: "ORS 90.427 — 90 days written notice; ORS 90.323(3) first-year prohibition not implicated",
        note: format!(
            "Notice of {} days satisfies ORS 90.427 90-day requirement; outside first-year window.",
            input.notice_days_provided
        ),
    }
}

fn wa_check(input: &DemolitionNoticeInput) -> DemolitionNoticeResult {
    if input.notice_days_provided < 120 {
        return DemolitionNoticeResult {
            regime: Regime::Washington,
            required_notice_days: 120,
            ellis_365_day_extension_applied: false,
            in_first_year: false,
            violation: ViolationType::InsufficientNotice,
            landlord_compliant: false,
            citation:
                "RCW 59.18.650 — 120 days written notice required for substantial rehabilitation, change of use, or demolition",
            note: format!(
                "Required 120 days; provided {} days. WA RCW 59.18.650 requires 120-day notice for demo / substantial-rehab / change-of-use.",
                input.notice_days_provided
            ),
        };
    }
    DemolitionNoticeResult {
        regime: Regime::Washington,
        required_notice_days: 120,
        ellis_365_day_extension_applied: false,
        in_first_year: false,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation: "RCW 59.18.650 — 120 days written notice",
        note: format!(
            "Notice of {} days satisfies WA 120-day requirement.",
            input.notice_days_provided
        ),
    }
}

fn default_check(_input: &DemolitionNoticeInput) -> DemolitionNoticeResult {
    DemolitionNoticeResult {
        regime: Regime::Default,
        required_notice_days: 0,
        ellis_365_day_extension_applied: false,
        in_first_year: false,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation:
            "No statewide demolition-specific tenant-notice statute identified — lease terms + just-cause-eviction rules apply",
        note: "Default regime: lease terms govern demolition notice. State has no statewide demolition-specific notice statute.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        regime: Regime,
        notice_days: u32,
        age: u32,
        disabled: bool,
        months_in: u32,
    ) -> DemolitionNoticeInput {
        DemolitionNoticeInput {
            regime,
            notice_days_provided: notice_days,
            tenant_age: age,
            tenant_disabled: disabled,
            months_since_tenancy_started: months_in,
        }
    }

    #[test]
    fn ca_standard_tenant_120_days_compliant() {
        let r = check(&input(Regime::CaliforniaEllisAct, 120, 40, false, 24));
        assert_eq!(r.required_notice_days, 120);
        assert_eq!(r.violation, ViolationType::None);
        assert!(!r.ellis_365_day_extension_applied);
        assert!(r.citation.contains("§ 7060.4(a)"));
    }

    #[test]
    fn ca_standard_tenant_119_days_insufficient() {
        let r = check(&input(Regime::CaliforniaEllisAct, 119, 40, false, 24));
        assert_eq!(r.violation, ViolationType::InsufficientNotice);
        assert!(r.note.contains("Required 120 days; provided 119 days"));
    }

    #[test]
    fn ca_senior_62_plus_with_one_year_residency_365_days() {
        let r = check(&input(Regime::CaliforniaEllisAct, 365, 65, false, 18));
        assert_eq!(r.required_notice_days, 365);
        assert!(r.ellis_365_day_extension_applied);
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.citation.contains("§ 7060.4(b)"));
    }

    #[test]
    fn ca_disabled_tenant_with_one_year_residency_365_days() {
        let r = check(&input(Regime::CaliforniaEllisAct, 365, 40, true, 18));
        assert!(r.ellis_365_day_extension_applied);
        assert_eq!(r.required_notice_days, 365);
    }

    #[test]
    fn ca_senior_without_one_year_residency_only_120_days() {
        // 62+ but < 12 months residency → no extension.
        let r = check(&input(Regime::CaliforniaEllisAct, 120, 65, false, 11));
        assert!(!r.ellis_365_day_extension_applied);
        assert_eq!(r.required_notice_days, 120);
    }

    #[test]
    fn ca_under_62_not_disabled_only_120_days() {
        let r = check(&input(Regime::CaliforniaEllisAct, 120, 61, false, 24));
        assert!(!r.ellis_365_day_extension_applied);
        assert_eq!(r.required_notice_days, 120);
    }

    #[test]
    fn ca_at_62_boundary_qualifies() {
        let r = check(&input(Regime::CaliforniaEllisAct, 365, 62, false, 12));
        assert!(r.ellis_365_day_extension_applied);
    }

    #[test]
    fn ca_at_12_month_residency_boundary_qualifies() {
        let r = check(&input(Regime::CaliforniaEllisAct, 365, 65, false, 12));
        assert!(r.ellis_365_day_extension_applied);
    }

    #[test]
    fn ca_senior_with_365_day_notice_compliant_at_boundary() {
        let r = check(&input(Regime::CaliforniaEllisAct, 365, 65, false, 18));
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn ca_senior_with_only_120_days_insufficient() {
        let r = check(&input(Regime::CaliforniaEllisAct, 120, 65, false, 18));
        assert_eq!(r.violation, ViolationType::InsufficientNotice);
        assert_eq!(r.required_notice_days, 365);
    }

    #[test]
    fn or_first_year_prohibition_blocks_demolition() {
        // 11 months in → blocked regardless of notice.
        let r = check(&input(Regime::Oregon, 365, 30, false, 11));
        assert_eq!(r.violation, ViolationType::FirstYearProhibited);
        assert!(r.in_first_year);
        assert!(r.citation.contains("ORS 90.323(3)"));
    }

    #[test]
    fn or_at_12_month_boundary_demolition_allowed() {
        let r = check(&input(Regime::Oregon, 90, 30, false, 12));
        assert!(!r.in_first_year);
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn or_90_day_notice_compliant() {
        let r = check(&input(Regime::Oregon, 90, 30, false, 24));
        assert_eq!(r.required_notice_days, 90);
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.citation.contains("ORS 90.427"));
    }

    #[test]
    fn or_89_day_notice_insufficient() {
        let r = check(&input(Regime::Oregon, 89, 30, false, 24));
        assert_eq!(r.violation, ViolationType::InsufficientNotice);
    }

    #[test]
    fn or_first_year_takes_precedence_over_insufficient_notice() {
        // 11 months in WITH 90-day notice — first-year prohibition still fires.
        let r = check(&input(Regime::Oregon, 90, 30, false, 11));
        assert_eq!(r.violation, ViolationType::FirstYearProhibited);
    }

    #[test]
    fn wa_120_day_notice_compliant() {
        let r = check(&input(Regime::Washington, 120, 30, false, 24));
        assert_eq!(r.required_notice_days, 120);
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.citation.contains("RCW 59.18.650"));
    }

    #[test]
    fn wa_119_day_notice_insufficient() {
        let r = check(&input(Regime::Washington, 119, 30, false, 24));
        assert_eq!(r.violation, ViolationType::InsufficientNotice);
        assert!(r.note.contains("demo / substantial-rehab"));
    }

    #[test]
    fn wa_no_age_extension() {
        // WA has no senior/disabled extension — 120 days uniform.
        let r = check(&input(Regime::Washington, 120, 75, true, 36));
        assert_eq!(r.required_notice_days, 120);
        assert!(!r.ellis_365_day_extension_applied);
    }

    #[test]
    fn wa_no_first_year_prohibition() {
        // WA has no first-year prohibition — 11 months residency, 120-day
        // notice → compliant.
        let r = check(&input(Regime::Washington, 120, 30, false, 11));
        assert!(!r.in_first_year);
        assert_eq!(r.violation, ViolationType::None);
    }

    #[test]
    fn default_no_obligation() {
        let r = check(&input(Regime::Default, 0, 30, false, 24));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.landlord_compliant);
        assert!(r.citation.contains("lease terms"));
    }

    #[test]
    fn state_routing_ca_or_wa_default() {
        assert_eq!(Regime::for_state("CA"), Regime::CaliforniaEllisAct);
        assert_eq!(Regime::for_state("OR"), Regime::Oregon);
        assert_eq!(Regime::for_state("WA"), Regime::Washington);
        assert_eq!(Regime::for_state("TX"), Regime::Default);
        assert_eq!(Regime::for_state("NY"), Regime::Default);
    }

    #[test]
    fn state_routing_case_insensitive() {
        assert_eq!(Regime::for_state("ca"), Regime::CaliforniaEllisAct);
        assert_eq!(Regime::for_state("Or"), Regime::Oregon);
        assert_eq!(Regime::for_state("wa"), Regime::Washington);
    }

    #[test]
    fn only_ca_has_age_disability_extension() {
        // Same senior-disabled-long-tenancy scenario across all regimes.
        let ca = check(&input(Regime::CaliforniaEllisAct, 365, 70, true, 36));
        let or = check(&input(Regime::Oregon, 365, 70, true, 36));
        let wa = check(&input(Regime::Washington, 365, 70, true, 36));
        assert!(ca.ellis_365_day_extension_applied);
        assert!(!or.ellis_365_day_extension_applied);
        assert!(!wa.ellis_365_day_extension_applied);
    }

    #[test]
    fn only_or_has_first_year_prohibition() {
        let ca = check(&input(Regime::CaliforniaEllisAct, 120, 30, false, 6));
        let or = check(&input(Regime::Oregon, 90, 30, false, 6));
        let wa = check(&input(Regime::Washington, 120, 30, false, 6));
        assert_ne!(ca.violation, ViolationType::FirstYearProhibited);
        assert_eq!(or.violation, ViolationType::FirstYearProhibited);
        assert_ne!(wa.violation, ViolationType::FirstYearProhibited);
    }

    #[test]
    fn notice_period_ordering_or_lt_ca_eq_wa_for_standard() {
        let or = check(&input(Regime::Oregon, 90, 30, false, 24));
        let ca = check(&input(Regime::CaliforniaEllisAct, 120, 30, false, 24));
        let wa = check(&input(Regime::Washington, 120, 30, false, 24));
        assert!(or.required_notice_days < ca.required_notice_days);
        assert_eq!(ca.required_notice_days, wa.required_notice_days);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let ca = check(&input(Regime::CaliforniaEllisAct, 120, 30, false, 24));
        assert!(ca.citation.contains("Govt Code § 7060.4(a)"));

        let ca_ext = check(&input(Regime::CaliforniaEllisAct, 365, 70, false, 24));
        assert!(ca_ext.citation.contains("§ 7060.4(b)"));

        let or = check(&input(Regime::Oregon, 90, 30, false, 24));
        assert!(or.citation.contains("ORS 90.427"));

        let wa = check(&input(Regime::Washington, 120, 30, false, 24));
        assert!(wa.citation.contains("RCW 59.18.650"));
    }
}
