//! State / municipal eviction-diversion-program landlord pre-filing
//! mediation compliance check. Distinct from `eviction_notices`
//! (which addresses the NOTICE PERIOD landlord must give the tenant
//! prior to filing) and `pre_eviction_mediation` topics — this module
//! addresses the affirmative duty for the landlord to ENROLL in a
//! diversion program AND participate in good faith for a minimum
//! period BEFORE filing an eviction in court.
//!
//! Two jurisdictions impose mandatory pre-filing diversion duties on
//! landlords with sharply different minimum waiting periods + notice
//! mechanics:
//!
//! Philadelphia (Phila Code § 9-811 Eviction Diversion Program) —
//! landlords must apply to and be approved for the EDP, participate
//! in good faith for at least 30 days before filing, and provide
//! notice of diversion rights to the tenant. Applies to virtually
//! all eviction grounds since the 2022 amendments (nonpayment, lease
//! breach, term expiration). Carve-out: imminent physical harm or
//! harassment by tenant exempts the landlord. Enforcement: tenant
//! defense in court, plus court may sua sponte dismiss noncompliant
//! filings; non-waivable.
//!
//! New Jersey (court-rule mandatory diversion plus DCA program) —
//! landlords must enroll in the diversion program AND apply for
//! rental assistance if available, provide a 14-day notice to the
//! tenant of mediation right and send a copy to the local dispute
//! resolution center, wait at least 45 days from enrollment before
//! filing IF the tenant timely schedules mediation, and participate
//! in good faith. If the tenant does NOT schedule mediation within
//! 14 days, the landlord may proceed with filing.
//!
//! **Default** — no statewide mandatory pre-filing mediation;
//! court-level voluntary mediation may exist.
//!
//! Citations: Philadelphia Code § 9-811 (Eviction Diversion Program,
//! amended 2022-01 to apply to all eviction grounds); NJ Court Rule
//! 6:4 + DCA Eviction Diversion Program (2022+); LSNJ Legal Eviction
//! Process publication.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Regime {
    Philadelphia,
    NewJersey,
    Default,
}

impl Regime {
    pub fn for_jurisdiction(state: &str, city: &str) -> Self {
        let st = state.trim().to_ascii_uppercase();
        let ct = city.trim().to_ascii_lowercase();
        match (st.as_str(), ct.as_str()) {
            ("PA", "philadelphia") | ("PA", "phila") => Self::Philadelphia,
            ("NJ", _) => Self::NewJersey,
            _ => Self::Default,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvictionGround {
    Nonpayment,
    LeaseBreach,
    TermExpiration,
    /// Imminent physical harm or harassment by tenant — Philadelphia §
    /// 9-811(2)(b) carve-out exempts the landlord from EDP requirements.
    ImminentHarmHarassment,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiversionProgramInput {
    pub regime: Regime,
    pub eviction_ground: EvictionGround,
    /// Whether the landlord enrolled in the applicable diversion program.
    pub diversion_program_enrolled: bool,
    /// Days since landlord enrolled in the program. Used to test the
    /// minimum 30-day (PHL) / 45-day (NJ) waiting periods.
    pub days_since_enrollment: u32,
    /// Whether the landlord provided the required notice of diversion
    /// rights to the tenant.
    pub diversion_notice_to_tenant_provided: bool,
    /// NJ-specific: whether the tenant scheduled mediation within the
    /// 14-day notice window. If not, landlord may proceed to filing
    /// without the 45-day wait.
    pub tenant_scheduled_mediation_within_14_days: bool,
    /// Whether the landlord participated in mediation in good faith
    /// (required for both PHL and NJ).
    pub good_faith_participation: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ViolationType {
    None,
    NotEnrolledInDiversionProgram,
    InsufficientWaitingPeriod,
    MissingNoticeToTenant,
    NotGoodFaithParticipation,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct DiversionProgramResult {
    pub regime: Regime,
    pub pre_filing_diversion_required: bool,
    pub minimum_waiting_period_days: u32,
    pub imminent_harm_carve_out_applied: bool,
    pub can_proceed_to_filing: bool,
    pub violation: ViolationType,
    pub landlord_compliant: bool,
    pub citation: &'static str,
    pub note: String,
}

pub fn check(input: &DiversionProgramInput) -> DiversionProgramResult {
    match input.regime {
        Regime::Philadelphia => phl_check(input),
        Regime::NewJersey => nj_check(input),
        Regime::Default => default_check(input),
    }
}

fn phl_check(input: &DiversionProgramInput) -> DiversionProgramResult {
    // § 9-811(2)(b): imminent physical harm or harassment carve-out
    // exempts the landlord from EDP requirements.
    if input.eviction_ground == EvictionGround::ImminentHarmHarassment {
        return DiversionProgramResult {
            regime: Regime::Philadelphia,
            pre_filing_diversion_required: false,
            minimum_waiting_period_days: 0,
            imminent_harm_carve_out_applied: true,
            can_proceed_to_filing: true,
            violation: ViolationType::None,
            landlord_compliant: true,
            citation: "Philadelphia Code § 9-811(2)(b) — imminent physical harm or harassment carve-out exempts landlord from EDP requirements",
            note: "Eviction necessary to cease or prevent imminent threat of harm — § 9-811 carve-out applies. Landlord may file directly without EDP enrollment.".to_string(),
        };
    }

    if !input.diversion_program_enrolled {
        return DiversionProgramResult {
            regime: Regime::Philadelphia,
            pre_filing_diversion_required: true,
            minimum_waiting_period_days: 30,
            imminent_harm_carve_out_applied: false,
            can_proceed_to_filing: false,
            violation: ViolationType::NotEnrolledInDiversionProgram,
            landlord_compliant: false,
            citation: "Philadelphia Code § 9-811 — landlord must apply for and be approved for the Eviction Diversion Program before filing",
            note: "Landlord has not enrolled in the EDP. Filing is not permitted; tenant may assert noncompliance as a defense and the court may sua sponte dismiss.".to_string(),
        };
    }

    if !input.diversion_notice_to_tenant_provided {
        return DiversionProgramResult {
            regime: Regime::Philadelphia,
            pre_filing_diversion_required: true,
            minimum_waiting_period_days: 30,
            imminent_harm_carve_out_applied: false,
            can_proceed_to_filing: false,
            violation: ViolationType::MissingNoticeToTenant,
            landlord_compliant: false,
            citation: "Philadelphia Code § 9-811 — landlord must provide notice of diversion rights to tenant",
            note: "Notice of diversion rights not provided to tenant.".to_string(),
        };
    }

    if input.days_since_enrollment < 30 {
        return DiversionProgramResult {
            regime: Regime::Philadelphia,
            pre_filing_diversion_required: true,
            minimum_waiting_period_days: 30,
            imminent_harm_carve_out_applied: false,
            can_proceed_to_filing: false,
            violation: ViolationType::InsufficientWaitingPeriod,
            landlord_compliant: false,
            citation: "Philadelphia Code § 9-811 — landlord must participate in EDP in good faith for at least 30 days before filing",
            note: format!(
                "Only {} days since EDP enrollment — Philadelphia EDP requires at least 30 days of good-faith participation before filing.",
                input.days_since_enrollment
            ),
        };
    }

    if !input.good_faith_participation {
        return DiversionProgramResult {
            regime: Regime::Philadelphia,
            pre_filing_diversion_required: true,
            minimum_waiting_period_days: 30,
            imminent_harm_carve_out_applied: false,
            can_proceed_to_filing: false,
            violation: ViolationType::NotGoodFaithParticipation,
            landlord_compliant: false,
            citation: "Philadelphia Code § 9-811 — landlord must participate in EDP in GOOD FAITH",
            note: "Landlord did not participate in EDP in good faith.".to_string(),
        };
    }

    DiversionProgramResult {
        regime: Regime::Philadelphia,
        pre_filing_diversion_required: true,
        minimum_waiting_period_days: 30,
        imminent_harm_carve_out_applied: false,
        can_proceed_to_filing: true,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation: "Philadelphia Code § 9-811 — Eviction Diversion Program compliance OK",
        note: format!(
            "EDP requirements satisfied: enrolled + {} days of good-faith participation + tenant notice provided. Landlord may file.",
            input.days_since_enrollment
        ),
    }
}

fn nj_check(input: &DiversionProgramInput) -> DiversionProgramResult {
    if !input.diversion_program_enrolled {
        return DiversionProgramResult {
            regime: Regime::NewJersey,
            pre_filing_diversion_required: true,
            minimum_waiting_period_days: 45,
            imminent_harm_carve_out_applied: false,
            can_proceed_to_filing: false,
            violation: ViolationType::NotEnrolledInDiversionProgram,
            landlord_compliant: false,
            citation: "NJ DCA Eviction Diversion Program — landlord must enroll in the program and apply for rental assistance before filing",
            note: "Landlord has not enrolled in the NJ DCA diversion program.".to_string(),
        };
    }

    if !input.diversion_notice_to_tenant_provided {
        return DiversionProgramResult {
            regime: Regime::NewJersey,
            pre_filing_diversion_required: true,
            minimum_waiting_period_days: 45,
            imminent_harm_carve_out_applied: false,
            can_proceed_to_filing: false,
            violation: ViolationType::MissingNoticeToTenant,
            landlord_compliant: false,
            citation: "NJ DCA Eviction Diversion Program — landlord must provide 14-day notice of mediation right to tenant + send copy to local dispute resolution center",
            note: "Required 14-day notice of mediation right not provided to tenant + dispute resolution center.".to_string(),
        };
    }

    // If tenant did NOT timely schedule mediation, landlord may proceed
    // without the 45-day wait. This is the NJ-specific tenant-default
    // carve-out.
    if !input.tenant_scheduled_mediation_within_14_days {
        return DiversionProgramResult {
            regime: Regime::NewJersey,
            pre_filing_diversion_required: true,
            minimum_waiting_period_days: 45,
            imminent_harm_carve_out_applied: false,
            can_proceed_to_filing: true,
            violation: ViolationType::None,
            landlord_compliant: true,
            citation: "NJ DCA Eviction Diversion Program — landlord may proceed after 14-day notice when tenant fails to schedule mediation",
            note: "Tenant did not schedule mediation within the 14-day notice window. Landlord may proceed to filing without the 45-day diversion wait.".to_string(),
        };
    }

    if input.days_since_enrollment < 45 {
        return DiversionProgramResult {
            regime: Regime::NewJersey,
            pre_filing_diversion_required: true,
            minimum_waiting_period_days: 45,
            imminent_harm_carve_out_applied: false,
            can_proceed_to_filing: false,
            violation: ViolationType::InsufficientWaitingPeriod,
            landlord_compliant: false,
            citation: "NJ DCA Eviction Diversion Program — landlord must wait at least 45 days before filing when tenant timely schedules mediation",
            note: format!(
                "Only {} days since enrollment — NJ requires at least 45 days when tenant schedules mediation.",
                input.days_since_enrollment
            ),
        };
    }

    if !input.good_faith_participation {
        return DiversionProgramResult {
            regime: Regime::NewJersey,
            pre_filing_diversion_required: true,
            minimum_waiting_period_days: 45,
            imminent_harm_carve_out_applied: false,
            can_proceed_to_filing: false,
            violation: ViolationType::NotGoodFaithParticipation,
            landlord_compliant: false,
            citation: "NJ DCA Eviction Diversion Program — good-faith participation required",
            note: "Landlord did not participate in diversion mediation in good faith.".to_string(),
        };
    }

    DiversionProgramResult {
        regime: Regime::NewJersey,
        pre_filing_diversion_required: true,
        minimum_waiting_period_days: 45,
        imminent_harm_carve_out_applied: false,
        can_proceed_to_filing: true,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation: "NJ DCA Eviction Diversion Program — compliance OK",
        note: format!(
            "NJ diversion requirements satisfied: enrolled + 14-day notice + tenant scheduled mediation + {} days good-faith participation. Landlord may file.",
            input.days_since_enrollment
        ),
    }
}

fn default_check(_input: &DiversionProgramInput) -> DiversionProgramResult {
    DiversionProgramResult {
        regime: Regime::Default,
        pre_filing_diversion_required: false,
        minimum_waiting_period_days: 0,
        imminent_harm_carve_out_applied: false,
        can_proceed_to_filing: true,
        violation: ViolationType::None,
        landlord_compliant: true,
        citation: "No mandatory pre-filing eviction-diversion program identified for this jurisdiction",
        note: "Default regime: no mandatory pre-filing mediation. Court-level voluntary mediation may exist; check local court rules.".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input(
        regime: Regime,
        ground: EvictionGround,
        enrolled: bool,
        days: u32,
        notice: bool,
        scheduled: bool,
        good_faith: bool,
    ) -> DiversionProgramInput {
        DiversionProgramInput {
            regime,
            eviction_ground: ground,
            diversion_program_enrolled: enrolled,
            days_since_enrollment: days,
            diversion_notice_to_tenant_provided: notice,
            tenant_scheduled_mediation_within_14_days: scheduled,
            good_faith_participation: good_faith,
        }
    }

    #[test]
    fn phl_imminent_harm_carve_out_skips_edp() {
        let r = check(&input(
            Regime::Philadelphia,
            EvictionGround::ImminentHarmHarassment,
            false, 0, false, false, false,
        ));
        assert!(r.imminent_harm_carve_out_applied);
        assert!(r.can_proceed_to_filing);
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.citation.contains("§ 9-811(2)(b)"));
        assert!(!r.pre_filing_diversion_required);
    }

    #[test]
    fn phl_nonpayment_not_enrolled_blocks_filing() {
        let r = check(&input(
            Regime::Philadelphia,
            EvictionGround::Nonpayment,
            false, 0, false, false, false,
        ));
        assert_eq!(r.violation, ViolationType::NotEnrolledInDiversionProgram);
        assert!(!r.can_proceed_to_filing);
        assert!(r.citation.contains("apply for and be approved"));
    }

    #[test]
    fn phl_enrolled_no_notice_blocks() {
        let r = check(&input(
            Regime::Philadelphia,
            EvictionGround::Nonpayment,
            true, 35, false, false, true,
        ));
        assert_eq!(r.violation, ViolationType::MissingNoticeToTenant);
        assert!(!r.can_proceed_to_filing);
    }

    #[test]
    fn phl_29_days_insufficient() {
        let r = check(&input(
            Regime::Philadelphia,
            EvictionGround::Nonpayment,
            true, 29, true, false, true,
        ));
        assert_eq!(r.violation, ViolationType::InsufficientWaitingPeriod);
        assert!(r.note.contains("29 days"));
        assert!(r.note.contains("30 days"));
    }

    #[test]
    fn phl_at_30_day_boundary_compliant() {
        let r = check(&input(
            Regime::Philadelphia,
            EvictionGround::Nonpayment,
            true, 30, true, false, true,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.can_proceed_to_filing);
    }

    #[test]
    fn phl_no_good_faith_blocks() {
        let r = check(&input(
            Regime::Philadelphia,
            EvictionGround::Nonpayment,
            true, 35, true, false, false,
        ));
        assert_eq!(r.violation, ViolationType::NotGoodFaithParticipation);
    }

    #[test]
    fn phl_all_requirements_met_compliant() {
        let r = check(&input(
            Regime::Philadelphia,
            EvictionGround::Nonpayment,
            true, 45, true, false, true,
        ));
        assert!(r.landlord_compliant);
        assert!(r.can_proceed_to_filing);
    }

    #[test]
    fn phl_applies_to_lease_breach_post_2022_amendment() {
        let r = check(&input(
            Regime::Philadelphia,
            EvictionGround::LeaseBreach,
            false, 0, false, false, false,
        ));
        // Post-2022 amendment: lease breach is covered.
        assert_eq!(r.violation, ViolationType::NotEnrolledInDiversionProgram);
    }

    #[test]
    fn phl_applies_to_term_expiration_post_2022() {
        let r = check(&input(
            Regime::Philadelphia,
            EvictionGround::TermExpiration,
            false, 0, false, false, false,
        ));
        assert_eq!(r.violation, ViolationType::NotEnrolledInDiversionProgram);
    }

    #[test]
    fn nj_not_enrolled_blocks() {
        let r = check(&input(
            Regime::NewJersey,
            EvictionGround::Nonpayment,
            false, 0, false, false, false,
        ));
        assert_eq!(r.violation, ViolationType::NotEnrolledInDiversionProgram);
        assert!(r.citation.contains("NJ DCA"));
    }

    #[test]
    fn nj_enrolled_no_notice_blocks() {
        let r = check(&input(
            Regime::NewJersey,
            EvictionGround::Nonpayment,
            true, 50, false, true, true,
        ));
        assert_eq!(r.violation, ViolationType::MissingNoticeToTenant);
        assert!(r.citation.contains("14-day notice"));
    }

    #[test]
    fn nj_tenant_did_not_schedule_landlord_may_proceed() {
        // NJ-specific tenant-default carve-out: if tenant doesn't schedule
        // within 14 days, landlord can file immediately.
        let r = check(&input(
            Regime::NewJersey,
            EvictionGround::Nonpayment,
            true, 14, true, false, true,
        ));
        assert!(r.can_proceed_to_filing);
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.note.contains("did not schedule mediation"));
    }

    #[test]
    fn nj_tenant_scheduled_44_days_insufficient() {
        let r = check(&input(
            Regime::NewJersey,
            EvictionGround::Nonpayment,
            true, 44, true, true, true,
        ));
        assert_eq!(r.violation, ViolationType::InsufficientWaitingPeriod);
        assert!(r.note.contains("44 days"));
        assert!(r.note.contains("45 days"));
    }

    #[test]
    fn nj_at_45_day_boundary_compliant() {
        let r = check(&input(
            Regime::NewJersey,
            EvictionGround::Nonpayment,
            true, 45, true, true, true,
        ));
        assert_eq!(r.violation, ViolationType::None);
        assert!(r.can_proceed_to_filing);
    }

    #[test]
    fn nj_no_good_faith_blocks() {
        let r = check(&input(
            Regime::NewJersey,
            EvictionGround::Nonpayment,
            true, 60, true, true, false,
        ));
        assert_eq!(r.violation, ViolationType::NotGoodFaithParticipation);
    }

    #[test]
    fn default_no_diversion_required() {
        let r = check(&input(
            Regime::Default,
            EvictionGround::Nonpayment,
            false, 0, false, false, false,
        ));
        assert!(r.can_proceed_to_filing);
        assert!(!r.pre_filing_diversion_required);
        assert!(r.landlord_compliant);
    }

    #[test]
    fn jurisdiction_routing_phila_nj_default() {
        assert_eq!(
            Regime::for_jurisdiction("PA", "Philadelphia"),
            Regime::Philadelphia
        );
        assert_eq!(
            Regime::for_jurisdiction("PA", "Phila"),
            Regime::Philadelphia
        );
        assert_eq!(
            Regime::for_jurisdiction("PA", "Pittsburgh"),
            Regime::Default
        );
        assert_eq!(
            Regime::for_jurisdiction("NJ", "Newark"),
            Regime::NewJersey
        );
        assert_eq!(
            Regime::for_jurisdiction("CA", "Los Angeles"),
            Regime::Default
        );
    }

    #[test]
    fn jurisdiction_routing_case_insensitive() {
        assert_eq!(
            Regime::for_jurisdiction("pa", "philadelphia"),
            Regime::Philadelphia
        );
        assert_eq!(
            Regime::for_jurisdiction("nj", "any"),
            Regime::NewJersey
        );
    }

    #[test]
    fn only_phl_has_imminent_harm_carve_out() {
        let phl = check(&input(
            Regime::Philadelphia,
            EvictionGround::ImminentHarmHarassment,
            false, 0, false, false, false,
        ));
        let nj = check(&input(
            Regime::NewJersey,
            EvictionGround::ImminentHarmHarassment,
            false, 0, false, false, false,
        ));
        assert!(phl.imminent_harm_carve_out_applied);
        assert!(phl.can_proceed_to_filing);
        // NJ has no equivalent carve-out — same input still requires enrollment.
        assert!(!nj.imminent_harm_carve_out_applied);
        assert_eq!(nj.violation, ViolationType::NotEnrolledInDiversionProgram);
    }

    #[test]
    fn only_nj_has_tenant_default_carve_out() {
        // PHL has no "tenant didn't schedule" carve-out — landlord still
        // must wait 30 days regardless.
        let phl = check(&input(
            Regime::Philadelphia,
            EvictionGround::Nonpayment,
            true, 5, true, false, true,
        ));
        // NJ: tenant didn't schedule → landlord may proceed.
        let nj = check(&input(
            Regime::NewJersey,
            EvictionGround::Nonpayment,
            true, 5, true, false, true,
        ));
        assert_eq!(phl.violation, ViolationType::InsufficientWaitingPeriod);
        assert!(nj.can_proceed_to_filing);
    }

    #[test]
    fn waiting_period_ordering_phl_30_lt_nj_45() {
        let phl = check(&input(
            Regime::Philadelphia,
            EvictionGround::Nonpayment,
            true, 40, true, false, true,
        ));
        let nj = check(&input(
            Regime::NewJersey,
            EvictionGround::Nonpayment,
            true, 40, true, true, true,
        ));
        assert!(phl.minimum_waiting_period_days < nj.minimum_waiting_period_days);
        assert_eq!(phl.minimum_waiting_period_days, 30);
        assert_eq!(nj.minimum_waiting_period_days, 45);
    }

    #[test]
    fn citations_pin_correct_authorities() {
        let phl = check(&input(
            Regime::Philadelphia,
            EvictionGround::Nonpayment,
            true, 30, true, false, true,
        ));
        assert!(phl.citation.contains("Philadelphia Code § 9-811"));

        let nj = check(&input(
            Regime::NewJersey,
            EvictionGround::Nonpayment,
            true, 45, true, true, true,
        ));
        assert!(nj.citation.contains("NJ DCA"));
    }
}
