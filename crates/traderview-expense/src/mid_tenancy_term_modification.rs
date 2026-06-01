//! Mid-tenancy modification of NON-RENT lease terms — landlord
//! compliance check for unilateral changes to pet policy, parking
//! rules, smoking policy, late-fee structure, utility allocation,
//! house rules, etc. during an existing tenancy.
//!
//! Distinct from `rent_increase_notice_period` (rent-amount changes
//! across CA/WA/OR), `lease_termination_notice` (ending tenancy or
//! non-renewal across NY/CA/OR/WA/NJ + default), and
//! `lease_waiver_enforceability` (enforceability of lease waivers
//! of statutory tenant rights). This module addresses ONLY mid-
//! tenancy non-rent term modifications during an ongoing tenancy.
//!
//! Four regimes:
//!
//! **California — Cal. Civ. Code § 827(a)**. UNILATERAL
//! MODIFICATION PERMITTED in PERIODIC tenancies (month-to-month,
//! week-to-week, etc.) with prescribed written notice. Default
//! notice period equals "at least one period as fixed by the
//! agreement," with a floor of 30 days for monthly periodic
//! tenancies and 7 days for shorter periodic intervals. The
//! statute permits the rental agreement to set a shorter notice
//! period, but not below 7 days. Notice must be in writing,
//! served per Code Civ. Proc. § 1162. Service by mail within
//! California adds 5 calendar days under Code Civ. Proc. §
//! 1013(a). § 827(a) does NOT apply to fixed-term leases —
//! bilateral consent of both parties is required to modify a
//! fixed-term lease mid-term.
//!
//! **New York — common-law contract rule**. No statute permits
//! unilateral mid-tenancy modification of non-rent terms. RPP
//! § 226-c (HSTPA 2019) addresses only rent increases and non-
//! renewal of residential tenancy; it does NOT authorize
//! unilateral non-rent term modification. Bilateral written
//! consent of landlord and tenant is required to modify the
//! lease mid-tenancy regardless of tenancy type. A landlord
//! seeking to impose new non-rent terms during a month-to-month
//! tenancy must terminate per § 226-c (30/60/90-day notice
//! tiered by occupancy length) and offer new terms for a new
//! tenancy.
//!
//! **Texas — common-law contract rule + lease controls**. Tex.
//! Prop. Code Ch. 92 does not generally authorize unilateral
//! mid-tenancy modification of non-rent terms; § 91.001
//! addresses termination only. A lease may expressly grant the
//! landlord a right to modify non-rent terms on prescribed
//! notice (e.g., updated house rules, parking assignments). If
//! the lease lacks an express modification clause, bilateral
//! written consent is required for non-rent term modifications.
//! Lease-granted modification rights cannot override mandatory
//! statutory tenant protections under § 92.006 (anti-waiver of
//! certain warranties).
//!
//! **Default — common-law contract rule + Statute of Frauds**.
//! Bilateral consent of landlord and tenant required to modify
//! the lease mid-tenancy. Material modifications to leases
//! falling under the Statute of Frauds (one-year-or-more leases)
//! must be in writing under Restatement (Second) of Contracts §
//! 149. Periodic tenancy modifications historically require
//! termination plus new-tenancy offer at common law, absent an
//! express lease modification clause.
//!
//! Citations: Cal. Civ. Code § 827(a) (notice for term
//! modification of periodic tenancy); Cal. Civ. Code § 827(a)(1)
//! (30-day floor for monthly periodic); Cal. Civ. Code §
//! 827(a)(2) (7-day floor for shorter periodic); Cal. Code Civ.
//! Proc. § 1162 (service method); Cal. Code Civ. Proc. § 1013(a)
//! (5-day mail extension within CA); N.Y. Real Prop. Law §
//! 226-c (HSTPA 2019 — rent increase and non-renewal notice
//! only); Tex. Prop. Code § 91.001 (termination notice only);
//! Tex. Prop. Code § 92.006 (anti-waiver of certain warranties);
//! Restatement (Second) of Contracts § 149 (statute of frauds
//! application to material lease modifications).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    California,
    NewYork,
    Texas,
    Default,
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TenancyType {
    MonthToMonth,
    WeekToWeek,
    FixedTerm,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModificationInput {
    pub regime: Regime,
    pub tenancy_type: TenancyType,
    /// Whether the landlord delivered the modification notice in
    /// writing. Oral / verbal notice never satisfies any regime.
    pub notice_in_writing: bool,
    /// Number of days between notice delivery and modification
    /// effective date.
    pub notice_days_provided: u32,
    /// CA-only: § 1013(a) adds 5 calendar days when notice is
    /// served by mail within California. Outside CA or non-mail
    /// service: set false.
    pub served_by_mail_within_ca: bool,
    /// CA-only: rental agreement clause reducing the § 827(a)
    /// notice period. Statutory floor is 7 days; clauses below 7
    /// are void as against the statute.
    pub agreement_shortened_to_days: Option<u32>,
    /// Whether the tenant signed a written consent to the
    /// modification. Bilateral consent satisfies all four regimes
    /// for fixed-term modifications and avoids the § 827(a) notice
    /// analysis entirely.
    pub tenant_signed_consent: bool,
    /// Texas-only: lease expressly grants landlord unilateral
    /// right to modify non-rent terms on prescribed notice.
    pub tx_lease_grants_modification_right: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ModificationResult {
    pub enforceable: bool,
    pub required_notice_days: u32,
    pub effective_notice_days: u32,
    pub bilateral_consent_required: bool,
    pub violations: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &ModificationInput) -> ModificationResult {
    let mut violations: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if input.tenant_signed_consent {
        return ModificationResult {
            enforceable: true,
            required_notice_days: 0,
            effective_notice_days: input.notice_days_provided,
            bilateral_consent_required: false,
            violations,
            citation: citation_for(input.regime),
            notes: vec![
                "tenant signed written consent — bilateral modification valid under contract law in all regimes"
                    .to_string(),
            ],
        };
    }

    match input.regime {
        Regime::California => check_california(input, &mut violations, &mut notes),
        Regime::NewYork => check_new_york(input, &mut violations, &mut notes),
        Regime::Texas => check_texas(input, &mut violations, &mut notes),
        Regime::Default => check_default(input, &mut violations, &mut notes),
    }
}

fn check_california(
    input: &ModificationInput,
    violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> ModificationResult {
    let bilateral_consent_required;
    let required_notice_days: u32;

    match input.tenancy_type {
        TenancyType::FixedTerm => {
            bilateral_consent_required = true;
            required_notice_days = 0;
            violations.push(
                "Cal. Civ. Code § 827(a) governs periodic tenancies only — fixed-term lease modification requires bilateral consent"
                    .to_string(),
            );
        }
        TenancyType::MonthToMonth => {
            bilateral_consent_required = false;
            let floor = 30u32;
            required_notice_days = match input.agreement_shortened_to_days {
                Some(d) if d >= 7 && d < floor => d,
                Some(d) if d < 7 => {
                    notes.push(
                        "agreement shortening below 7 days is void — falling back to statutory floor"
                            .to_string(),
                    );
                    7
                }
                _ => floor,
            };
        }
        TenancyType::WeekToWeek => {
            bilateral_consent_required = false;
            required_notice_days = match input.agreement_shortened_to_days {
                Some(d) if d >= 7 => d,
                Some(_) => {
                    notes.push(
                        "agreement shortening below 7 days is void — falling back to statutory floor"
                            .to_string(),
                    );
                    7
                }
                None => 7,
            };
        }
    }

    let mail_extension: u32 = if input.served_by_mail_within_ca { 5 } else { 0 };
    let effective_required = required_notice_days + mail_extension;
    if input.served_by_mail_within_ca {
        notes.push("CCP § 1013(a) adds 5 days for service by mail within California".to_string());
    }

    if !input.notice_in_writing && !bilateral_consent_required {
        violations.push("notice not in writing — § 827(a) requires written notice".to_string());
    }

    if !bilateral_consent_required && input.notice_days_provided < effective_required {
        violations.push(format!(
            "notice short — provided {} days, required {} days",
            input.notice_days_provided, effective_required
        ));
    }

    ModificationResult {
        enforceable: violations.is_empty() && !bilateral_consent_required,
        required_notice_days,
        effective_notice_days: effective_required,
        bilateral_consent_required,
        violations: violations.clone(),
        citation: citation_for(Regime::California),
        notes: notes.clone(),
    }
}

fn check_new_york(
    input: &ModificationInput,
    violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> ModificationResult {
    violations.push(
        "no NY statute authorizes unilateral mid-tenancy non-rent term modification — bilateral written consent required"
            .to_string(),
    );
    notes.push(
        "to impose new terms during M2M tenancy, landlord must terminate per RPL § 226-c (30/60/90-day notice tiered by occupancy) and offer new tenancy"
            .to_string(),
    );

    ModificationResult {
        enforceable: false,
        required_notice_days: 0,
        effective_notice_days: input.notice_days_provided,
        bilateral_consent_required: true,
        violations: violations.clone(),
        citation: citation_for(Regime::NewYork),
        notes: notes.clone(),
    }
}

fn check_texas(
    input: &ModificationInput,
    violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> ModificationResult {
    if input.tx_lease_grants_modification_right {
        if !input.notice_in_writing {
            violations.push(
                "lease-granted modification right requires written notice under standard Texas lease form".to_string(),
            );
        }
        if input.notice_days_provided == 0 {
            violations.push("lease modification clause requires prescribed advance notice".to_string());
        }
        notes.push(
            "lease grants modification right — TX § 92.006 anti-waiver of certain warranties still applies to mandatory tenant protections"
                .to_string(),
        );
        return ModificationResult {
            enforceable: violations.is_empty(),
            required_notice_days: 0,
            effective_notice_days: input.notice_days_provided,
            bilateral_consent_required: false,
            violations: violations.clone(),
            citation: citation_for(Regime::Texas),
            notes: notes.clone(),
        };
    }

    violations.push(
        "no TX statute authorizes unilateral mid-tenancy non-rent term modification absent express lease clause — bilateral written consent required"
            .to_string(),
    );
    ModificationResult {
        enforceable: false,
        required_notice_days: 0,
        effective_notice_days: input.notice_days_provided,
        bilateral_consent_required: true,
        violations: violations.clone(),
        citation: citation_for(Regime::Texas),
        notes: notes.clone(),
    }
}

fn check_default(
    input: &ModificationInput,
    violations: &mut Vec<String>,
    notes: &mut Vec<String>,
) -> ModificationResult {
    violations.push(
        "default common-law rule — bilateral consent of landlord and tenant required to modify lease mid-tenancy"
            .to_string(),
    );
    notes.push(
        "Restatement (Second) of Contracts § 149 applies statute of frauds to material lease modifications for leases 1 year or more"
            .to_string(),
    );

    ModificationResult {
        enforceable: false,
        required_notice_days: 0,
        effective_notice_days: input.notice_days_provided,
        bilateral_consent_required: true,
        violations: violations.clone(),
        citation: citation_for(Regime::Default),
        notes: notes.clone(),
    }
}

fn citation_for(regime: Regime) -> &'static str {
    match regime {
        Regime::California => "Cal. Civ. Code § 827(a); Cal. Code Civ. Proc. §§ 1162, 1013(a)",
        Regime::NewYork => "N.Y. Real Prop. Law § 226-c (HSTPA 2019); common-law contract rule",
        Regime::Texas => "Tex. Prop. Code §§ 91.001, 92.006; common-law contract rule",
        Regime::Default => "Restatement (Second) of Contracts § 149; common-law contract rule",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(regime: Regime, tenancy_type: TenancyType) -> ModificationInput {
        ModificationInput {
            regime,
            tenancy_type,
            notice_in_writing: true,
            notice_days_provided: 30,
            served_by_mail_within_ca: false,
            agreement_shortened_to_days: None,
            tenant_signed_consent: false,
            tx_lease_grants_modification_right: false,
        }
    }

    #[test]
    fn ca_monthly_thirty_day_personal_service_valid() {
        let r = check(&base(Regime::California, TenancyType::MonthToMonth));
        assert!(r.enforceable);
        assert!(r.violations.is_empty());
        assert_eq!(r.required_notice_days, 30);
        assert_eq!(r.effective_notice_days, 30);
        assert!(!r.bilateral_consent_required);
    }

    #[test]
    fn ca_monthly_short_notice_violation() {
        let mut i = base(Regime::California, TenancyType::MonthToMonth);
        i.notice_days_provided = 25;
        let r = check(&i);
        assert!(!r.enforceable);
        assert!(r.violations.iter().any(|v| v.contains("notice short")));
    }

    #[test]
    fn ca_monthly_mail_service_requires_extra_five_days() {
        let mut i = base(Regime::California, TenancyType::MonthToMonth);
        i.served_by_mail_within_ca = true;
        i.notice_days_provided = 30;
        let r = check(&i);
        assert!(!r.enforceable, "30 days insufficient when served by mail");
        assert_eq!(r.effective_notice_days, 35);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("CCP § 1013(a) adds 5 days")));
    }

    #[test]
    fn ca_monthly_mail_service_thirty_five_days_valid() {
        let mut i = base(Regime::California, TenancyType::MonthToMonth);
        i.served_by_mail_within_ca = true;
        i.notice_days_provided = 35;
        let r = check(&i);
        assert!(r.enforceable);
        assert!(r.violations.is_empty());
        assert_eq!(r.effective_notice_days, 35);
    }

    #[test]
    fn ca_monthly_agreement_reduces_to_ten_days_valid() {
        let mut i = base(Regime::California, TenancyType::MonthToMonth);
        i.agreement_shortened_to_days = Some(10);
        i.notice_days_provided = 10;
        let r = check(&i);
        assert!(r.enforceable);
        assert_eq!(r.required_notice_days, 10);
    }

    #[test]
    fn ca_agreement_reduces_below_seven_void_falls_to_floor() {
        let mut i = base(Regime::California, TenancyType::MonthToMonth);
        i.agreement_shortened_to_days = Some(3);
        i.notice_days_provided = 3;
        let r = check(&i);
        assert!(!r.enforceable);
        assert_eq!(r.required_notice_days, 7);
        assert!(r.notes.iter().any(|n| n.contains("void")));
    }

    #[test]
    fn ca_week_to_week_seven_days_valid() {
        let r = check(&base(Regime::California, TenancyType::WeekToWeek));
        assert!(r.enforceable);
        assert_eq!(r.required_notice_days, 7);
    }

    #[test]
    fn ca_fixed_term_requires_bilateral_consent() {
        let r = check(&base(Regime::California, TenancyType::FixedTerm));
        assert!(!r.enforceable);
        assert!(r.bilateral_consent_required);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("fixed-term lease modification requires bilateral consent")));
    }

    #[test]
    fn ca_oral_notice_invalid_for_periodic() {
        let mut i = base(Regime::California, TenancyType::MonthToMonth);
        i.notice_in_writing = false;
        let r = check(&i);
        assert!(!r.enforceable);
        assert!(r.violations.iter().any(|v| v.contains("not in writing")));
    }

    #[test]
    fn ca_fixed_term_tenant_consent_satisfies_modification() {
        let mut i = base(Regime::California, TenancyType::FixedTerm);
        i.tenant_signed_consent = true;
        let r = check(&i);
        assert!(r.enforceable);
        assert!(!r.bilateral_consent_required);
        assert!(r.violations.is_empty());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("bilateral modification valid")));
    }

    #[test]
    fn ny_monthly_unilateral_modification_void() {
        let r = check(&base(Regime::NewYork, TenancyType::MonthToMonth));
        assert!(!r.enforceable);
        assert!(r.bilateral_consent_required);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("no NY statute authorizes")));
    }

    #[test]
    fn ny_fixed_term_unilateral_modification_void() {
        let r = check(&base(Regime::NewYork, TenancyType::FixedTerm));
        assert!(!r.enforceable);
        assert!(r.bilateral_consent_required);
    }

    #[test]
    fn ny_tenant_consent_satisfies_bilateral_requirement() {
        let mut i = base(Regime::NewYork, TenancyType::MonthToMonth);
        i.tenant_signed_consent = true;
        let r = check(&i);
        assert!(r.enforceable);
        assert!(r.violations.is_empty());
        assert!(!r.bilateral_consent_required);
    }

    #[test]
    fn ny_notes_termination_alternative() {
        let r = check(&base(Regime::NewYork, TenancyType::MonthToMonth));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 226-c") && n.contains("30/60/90")));
    }

    #[test]
    fn tx_unilateral_without_lease_clause_void() {
        let r = check(&base(Regime::Texas, TenancyType::MonthToMonth));
        assert!(!r.enforceable);
        assert!(r.bilateral_consent_required);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("no TX statute")));
    }

    #[test]
    fn tx_lease_clause_with_written_notice_valid() {
        let mut i = base(Regime::Texas, TenancyType::MonthToMonth);
        i.tx_lease_grants_modification_right = true;
        i.notice_days_provided = 30;
        let r = check(&i);
        assert!(r.enforceable);
        assert!(!r.bilateral_consent_required);
        assert!(r.notes.iter().any(|n| n.contains("§ 92.006")));
    }

    #[test]
    fn tx_lease_clause_oral_notice_invalid() {
        let mut i = base(Regime::Texas, TenancyType::MonthToMonth);
        i.tx_lease_grants_modification_right = true;
        i.notice_in_writing = false;
        let r = check(&i);
        assert!(!r.enforceable);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("requires written notice")));
    }

    #[test]
    fn tx_lease_clause_zero_days_notice_invalid() {
        let mut i = base(Regime::Texas, TenancyType::MonthToMonth);
        i.tx_lease_grants_modification_right = true;
        i.notice_days_provided = 0;
        let r = check(&i);
        assert!(!r.enforceable);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("prescribed advance notice")));
    }

    #[test]
    fn default_unilateral_modification_void() {
        let r = check(&base(Regime::Default, TenancyType::MonthToMonth));
        assert!(!r.enforceable);
        assert!(r.bilateral_consent_required);
        assert!(r
            .violations
            .iter()
            .any(|v| v.contains("default common-law rule")));
    }

    #[test]
    fn default_consent_satisfies_modification() {
        let mut i = base(Regime::Default, TenancyType::FixedTerm);
        i.tenant_signed_consent = true;
        let r = check(&i);
        assert!(r.enforceable);
        assert!(r.violations.is_empty());
    }

    #[test]
    fn default_notes_statute_of_frauds() {
        let r = check(&base(Regime::Default, TenancyType::MonthToMonth));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Restatement (Second) of Contracts § 149")));
    }

    #[test]
    fn citation_california_pins_827a_and_service_statutes() {
        let r = check(&base(Regime::California, TenancyType::MonthToMonth));
        assert!(r.citation.contains("§ 827(a)"));
        assert!(r.citation.contains("§§ 1162, 1013(a)"));
    }

    #[test]
    fn citation_newyork_pins_226c_hstpa() {
        let r = check(&base(Regime::NewYork, TenancyType::MonthToMonth));
        assert!(r.citation.contains("§ 226-c"));
        assert!(r.citation.contains("HSTPA 2019"));
    }

    #[test]
    fn citation_texas_pins_91_001_and_92_006() {
        let r = check(&base(Regime::Texas, TenancyType::MonthToMonth));
        assert!(r.citation.contains("§§ 91.001, 92.006"));
    }

    #[test]
    fn ca_seven_day_floor_boundary_with_mail_extension() {
        let mut i = base(Regime::California, TenancyType::WeekToWeek);
        i.agreement_shortened_to_days = Some(7);
        i.served_by_mail_within_ca = true;
        i.notice_days_provided = 11;
        let r = check(&i);
        assert!(!r.enforceable, "11 days short of 7 + 5 mail = 12");
        assert_eq!(r.effective_notice_days, 12);
    }

    #[test]
    fn ca_thirty_day_with_mail_one_day_short_invalid() {
        let mut i = base(Regime::California, TenancyType::MonthToMonth);
        i.served_by_mail_within_ca = true;
        i.notice_days_provided = 34;
        let r = check(&i);
        assert!(!r.enforceable);
        assert!(r.violations.iter().any(|v| v.contains("notice short")));
    }
}
