//! Residential Tenant Estoppel Certificate Compliance Module.
//!
//! Pure-compute check for whether a landlord-issued tenant
//! estoppel certificate demand is enforceable, whether the tenant
//! has properly responded, and whether the resulting estoppel
//! certificate is binding. Trader-landlord critical because
//! estoppel certificates are demanded during refinancing and
//! sale transactions; defective demands and false statements
//! create transaction-failure and litigation exposure.
//!
//! Web research (verified 2026-06-03):
//! - **General rule**: a tenant is required to sign an estoppel
//!   certificate ONLY when the written lease contains a provision
//!   requiring the tenant to do so. Absent a lease provision, a
//!   tenant is NOT required to complete and sign an estoppel
//!   certificate. (Tobener Ravenscroft LLP Estoppel Certificates;
//!   California Lawyers Association Tenant Estoppel Certificates;
//!   Tenant Law Group SF.)
//! - **Refusal to sign when lease requires**: breach of lease.
//!   (Wolford Wayne LLP — Oakland tenant attorneys.)
//! - **Typical response timeframe**: a tenant typically has 10
//!   days to complete an estoppel certificate after landlord
//!   request, as provided by the lease provision. Lease commonly
//!   states "Tenant agrees, from time to time, within 10 days
//!   after request of Landlord, to execute and deliver to
//!   Landlord ... any estoppel certificate requested by Landlord."
//! - **California Evidence Code § 622** — the facts recited in
//!   a written instrument are conclusively presumed to be true
//!   as between the parties thereto, or their successors in
//!   interest. Statements made in an estoppel certificate are
//!   BINDING; tenants cannot later contradict them.
//! - **Content of estoppel**: rent amount, lease terms, protected
//!   tenancy status (rent stabilization / rent control / good
//!   cause), oral agreements with landlord, amendments to written
//!   lease, promises made by landlord, utility payment
//!   arrangements.
//! - **Common landlord transactional triggers**: property sale,
//!   refinancing, line-of-credit application, mortgage loan
//!   modification.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const ESTOPPEL_TYPICAL_RESPONSE_DAYS: u32 = 10;
pub const ESTOPPEL_MINIMUM_REASONABLE_DAYS: u32 = 7;
pub const ESTOPPEL_CALIFORNIA_EVIDENCE_CODE_SECTION: u32 = 622;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LandlordTransactionalTrigger {
    PropertySale,
    Refinancing,
    LineOfCreditApplication,
    MortgageLoanModification,
    NoTransactionPending,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LeaseEstoppelClauseType {
    NoEstoppelClauseInLease,
    TenDayEstoppelClause,
    SevenDayEstoppelClause,
    StatutoryReasonableTimeNoExplicitDays,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantResponse {
    TenantSignedAsDrafted,
    TenantSignedWithCorrectionsNoted,
    TenantRefusedToSign,
    TenantNotYetResponded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EstoppelDraftContentDefect {
    NoDefects,
    OmittedProtectedTenancyStatus,
    UnderstatedOralAgreements,
    OverstatedRentAmount,
    OmittedLandlordPromises,
    OmittedUtilityPaymentArrangements,
    WaivedTenantRentStabilizationStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenantEstoppelCertificateMode {
    NotApplicableNoTransactionPending,
    CompliantTenantSignedEstoppelWithinTimeframe,
    CompliantTenantSignedWithCorrectionsNoted,
    CompliantTenantRefusedAbsentLeaseClause,
    CompliantTenantNotYetWithinResponseWindow,
    ViolationTenantRefusedToSignDespiteLeaseClause,
    ViolationLandlordDemandedEstoppelWithoutLeaseAuthority,
    ViolationEstoppelTimeframeShorterThanLeaseProvision,
    ViolationLandlordIncludedFalseStatementsInEstoppelDraft,
    ViolationLandlordWaivedTenantProtectedTenancyStatus,
    ViolationTenantMissedResponseDeadline,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub transactional_trigger: LandlordTransactionalTrigger,
    pub lease_estoppel_clause: LeaseEstoppelClauseType,
    pub landlord_demanded_response_in_days: u32,
    pub days_elapsed_since_landlord_demand: u32,
    pub tenant_response: TenantResponse,
    pub estoppel_draft_content_defect: EstoppelDraftContentDefect,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: TenantEstoppelCertificateMode,
    pub allowed_response_days: u32,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalTenantEstoppelCertificateInput = Input;
pub type RentalTenantEstoppelCertificateOutput = Output;
pub type RentalTenantEstoppelCertificateResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

fn lease_required_response_days(c: LeaseEstoppelClauseType) -> u32 {
    match c {
        LeaseEstoppelClauseType::NoEstoppelClauseInLease => 0,
        LeaseEstoppelClauseType::TenDayEstoppelClause => 10,
        LeaseEstoppelClauseType::SevenDayEstoppelClause => 7,
        LeaseEstoppelClauseType::StatutoryReasonableTimeNoExplicitDays => {
            ESTOPPEL_TYPICAL_RESPONSE_DAYS
        }
    }
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Tobener Ravenscroft LLP Estoppel Certificates — tenant required to sign ONLY when lease contains estoppel provision".to_string(),
        "California Lawyers Association Tenant Estoppel Certificates — refusal to sign when lease requires is breach of lease".to_string(),
        "California Evidence Code § 622 — facts recited in written instrument conclusively presumed true between parties or successors in interest".to_string(),
        "Wolford Wayne LLP Oakland — typical 10-day response timeframe under standard lease clause".to_string(),
        "Tenant Law Group SF — absent lease provision, tenant NOT required to complete and sign estoppel certificate".to_string(),
        "Standard lease clause: 'Tenant agrees, from time to time, within 10 days after request of Landlord, to execute and deliver to Landlord ... any estoppel certificate requested by Landlord'".to_string(),
        "Required content: rent amount + lease terms + protected tenancy status + oral agreements + lease amendments + landlord promises + utility payment arrangements".to_string(),
        "Common transactional triggers: property sale, refinancing, line-of-credit application, mortgage loan modification".to_string(),
    ];

    if input.transactional_trigger == LandlordTransactionalTrigger::NoTransactionPending {
        return Output {
            mode: TenantEstoppelCertificateMode::NotApplicableNoTransactionPending,
            allowed_response_days: 0,
            statutory_basis: "No landlord transaction triggering estoppel demand".to_string(),
            notes: "No pending sale, refinancing, line-of-credit, or mortgage modification; landlord has no legitimate basis to demand estoppel certificate.".to_string(),
            citations,
        };
    }

    if input.lease_estoppel_clause == LeaseEstoppelClauseType::NoEstoppelClauseInLease {
        if input.tenant_response == TenantResponse::TenantRefusedToSign {
            return Output {
                mode: TenantEstoppelCertificateMode::CompliantTenantRefusedAbsentLeaseClause,
                allowed_response_days: 0,
                statutory_basis: "No lease estoppel clause; tenant not required to sign".to_string(),
                notes: "COMPLIANT: lease lacks estoppel clause; tenant lawfully refused to sign. Absent lease provision, tenant NOT required to complete or sign estoppel certificate.".to_string(),
                citations,
            };
        }
        return Output {
            mode: TenantEstoppelCertificateMode::ViolationLandlordDemandedEstoppelWithoutLeaseAuthority,
            allowed_response_days: 0,
            statutory_basis: "Landlord demand for estoppel without lease authority unenforceable".to_string(),
            notes: "VIOLATION: lease contains no estoppel clause; landlord lacks contractual authority to demand estoppel certificate. Tenant under no obligation to sign.".to_string(),
            citations,
        };
    }

    let allowed_days = lease_required_response_days(input.lease_estoppel_clause);

    if input.landlord_demanded_response_in_days < allowed_days {
        return Output {
            mode: TenantEstoppelCertificateMode::ViolationEstoppelTimeframeShorterThanLeaseProvision,
            allowed_response_days: allowed_days,
            statutory_basis: format!(
                "Landlord demand for response in {} days shorter than lease-provided {} days",
                input.landlord_demanded_response_in_days, allowed_days
            ),
            notes: format!(
                "VIOLATION: landlord demanded response in {} days; lease provides {} days. Landlord cannot unilaterally shorten the lease-provided window.",
                input.landlord_demanded_response_in_days, allowed_days
            ),
            citations,
        };
    }

    if input.estoppel_draft_content_defect
        == EstoppelDraftContentDefect::WaivedTenantRentStabilizationStatus
    {
        return Output {
            mode: TenantEstoppelCertificateMode::ViolationLandlordWaivedTenantProtectedTenancyStatus,
            allowed_response_days: allowed_days,
            statutory_basis: "Estoppel draft purports to waive tenant rent stabilization / rent control status".to_string(),
            notes: "VIOLATION: landlord-drafted estoppel certificate omits or affirmatively waives tenant's rent stabilization or rent control status. Tenant must NOT sign as drafted; protected tenancy status is non-waivable by statute and any estoppel statement to the contrary is voidable.".to_string(),
            citations,
        };
    }

    if input.estoppel_draft_content_defect != EstoppelDraftContentDefect::NoDefects {
        return Output {
            mode: TenantEstoppelCertificateMode::ViolationLandlordIncludedFalseStatementsInEstoppelDraft,
            allowed_response_days: allowed_days,
            statutory_basis: format!(
                "Estoppel draft contains content defect {:?}",
                input.estoppel_draft_content_defect
            ),
            notes: format!(
                "VIOLATION: landlord-drafted estoppel certificate contains content defect {:?}. Tenant should sign with corrections noted to preserve true statement of the landlord-tenant relationship.",
                input.estoppel_draft_content_defect
            ),
            citations,
        };
    }

    match input.tenant_response {
        TenantResponse::TenantSignedAsDrafted => Output {
            mode: TenantEstoppelCertificateMode::CompliantTenantSignedEstoppelWithinTimeframe,
            allowed_response_days: allowed_days,
            statutory_basis: format!(
                "Tenant signed compliant estoppel within {} day response window per lease",
                allowed_days
            ),
            notes: format!(
                "COMPLIANT: tenant signed estoppel certificate as drafted within {} day response window. Cal. Evidence Code § 622: facts recited conclusively presumed true between parties.",
                allowed_days
            ),
            citations,
        },
        TenantResponse::TenantSignedWithCorrectionsNoted => Output {
            mode: TenantEstoppelCertificateMode::CompliantTenantSignedWithCorrectionsNoted,
            allowed_response_days: allowed_days,
            statutory_basis: "Tenant signed estoppel with corrections noted to preserve accuracy".to_string(),
            notes: "COMPLIANT: tenant signed estoppel certificate with corrections noted; preserves accurate statement of landlord-tenant relationship including rent amount, protected tenancy status, oral agreements, and landlord promises.".to_string(),
            citations,
        },
        TenantResponse::TenantRefusedToSign => Output {
            mode: TenantEstoppelCertificateMode::ViolationTenantRefusedToSignDespiteLeaseClause,
            allowed_response_days: allowed_days,
            statutory_basis: format!(
                "Lease contains estoppel clause ({} day window); tenant refusal = breach of lease",
                allowed_days
            ),
            notes: format!(
                "VIOLATION: lease contains estoppel clause requiring tenant response within {} days; tenant refused to sign. Refusal is breach of lease and may give rise to landlord remedies including monetary damages (transaction failure / financing repricing).",
                allowed_days
            ),
            citations,
        },
        TenantResponse::TenantNotYetResponded => {
            if input.days_elapsed_since_landlord_demand <= allowed_days {
                Output {
                    mode: TenantEstoppelCertificateMode::CompliantTenantNotYetWithinResponseWindow,
                    allowed_response_days: allowed_days,
                    statutory_basis: format!(
                        "Tenant within {} day response window ({} days elapsed)",
                        allowed_days, input.days_elapsed_since_landlord_demand
                    ),
                    notes: format!(
                        "COMPLIANT: tenant has not yet responded but is within the {} day response window ({} days elapsed).",
                        allowed_days, input.days_elapsed_since_landlord_demand
                    ),
                    citations,
                }
            } else {
                Output {
                    mode: TenantEstoppelCertificateMode::ViolationTenantMissedResponseDeadline,
                    allowed_response_days: allowed_days,
                    statutory_basis: format!(
                        "Tenant exceeded {} day response window ({} days elapsed)",
                        allowed_days, input.days_elapsed_since_landlord_demand
                    ),
                    notes: format!(
                        "VIOLATION: tenant has not responded within the {} day response window; {} days elapsed. Tenant in breach of lease.",
                        allowed_days, input.days_elapsed_since_landlord_demand
                    ),
                    citations,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_california_sale_with_10_day_clause_compliant() -> Input {
        Input {
            transactional_trigger: LandlordTransactionalTrigger::PropertySale,
            lease_estoppel_clause: LeaseEstoppelClauseType::TenDayEstoppelClause,
            landlord_demanded_response_in_days: 10,
            days_elapsed_since_landlord_demand: 5,
            tenant_response: TenantResponse::TenantSignedAsDrafted,
            estoppel_draft_content_defect: EstoppelDraftContentDefect::NoDefects,
        }
    }

    #[test]
    fn no_transaction_pending_not_applicable() {
        let input = Input {
            transactional_trigger: LandlordTransactionalTrigger::NoTransactionPending,
            ..baseline_california_sale_with_10_day_clause_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantEstoppelCertificateMode::NotApplicableNoTransactionPending
        );
    }

    #[test]
    fn no_lease_clause_tenant_refused_compliant() {
        let input = Input {
            lease_estoppel_clause: LeaseEstoppelClauseType::NoEstoppelClauseInLease,
            tenant_response: TenantResponse::TenantRefusedToSign,
            ..baseline_california_sale_with_10_day_clause_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantEstoppelCertificateMode::CompliantTenantRefusedAbsentLeaseClause
        );
    }

    #[test]
    fn no_lease_clause_landlord_demand_violation() {
        let input = Input {
            lease_estoppel_clause: LeaseEstoppelClauseType::NoEstoppelClauseInLease,
            tenant_response: TenantResponse::TenantSignedAsDrafted,
            ..baseline_california_sale_with_10_day_clause_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantEstoppelCertificateMode::ViolationLandlordDemandedEstoppelWithoutLeaseAuthority
        );
    }

    #[test]
    fn ten_day_clause_signed_compliant() {
        let result = check(&baseline_california_sale_with_10_day_clause_compliant());
        assert_eq!(
            result.mode,
            TenantEstoppelCertificateMode::CompliantTenantSignedEstoppelWithinTimeframe
        );
        assert_eq!(result.allowed_response_days, 10);
    }

    #[test]
    fn signed_with_corrections_noted_compliant() {
        let input = Input {
            tenant_response: TenantResponse::TenantSignedWithCorrectionsNoted,
            ..baseline_california_sale_with_10_day_clause_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantEstoppelCertificateMode::CompliantTenantSignedWithCorrectionsNoted
        );
    }

    #[test]
    fn tenant_refused_to_sign_despite_lease_clause_violation() {
        let input = Input {
            tenant_response: TenantResponse::TenantRefusedToSign,
            ..baseline_california_sale_with_10_day_clause_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantEstoppelCertificateMode::ViolationTenantRefusedToSignDespiteLeaseClause
        );
    }

    #[test]
    fn landlord_shortened_timeframe_violation() {
        let input = Input {
            landlord_demanded_response_in_days: 3,
            ..baseline_california_sale_with_10_day_clause_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantEstoppelCertificateMode::ViolationEstoppelTimeframeShorterThanLeaseProvision
        );
    }

    #[test]
    fn waived_rent_stabilization_status_violation() {
        let input = Input {
            estoppel_draft_content_defect:
                EstoppelDraftContentDefect::WaivedTenantRentStabilizationStatus,
            ..baseline_california_sale_with_10_day_clause_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantEstoppelCertificateMode::ViolationLandlordWaivedTenantProtectedTenancyStatus
        );
    }

    #[test]
    fn overstated_rent_amount_violation() {
        let input = Input {
            estoppel_draft_content_defect: EstoppelDraftContentDefect::OverstatedRentAmount,
            ..baseline_california_sale_with_10_day_clause_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantEstoppelCertificateMode::ViolationLandlordIncludedFalseStatementsInEstoppelDraft
        );
    }

    #[test]
    fn omitted_oral_agreements_violation() {
        let input = Input {
            estoppel_draft_content_defect: EstoppelDraftContentDefect::UnderstatedOralAgreements,
            ..baseline_california_sale_with_10_day_clause_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantEstoppelCertificateMode::ViolationLandlordIncludedFalseStatementsInEstoppelDraft
        );
    }

    #[test]
    fn omitted_landlord_promises_violation() {
        let input = Input {
            estoppel_draft_content_defect: EstoppelDraftContentDefect::OmittedLandlordPromises,
            ..baseline_california_sale_with_10_day_clause_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantEstoppelCertificateMode::ViolationLandlordIncludedFalseStatementsInEstoppelDraft
        );
    }

    #[test]
    fn tenant_not_yet_responded_within_window_compliant() {
        let input = Input {
            tenant_response: TenantResponse::TenantNotYetResponded,
            days_elapsed_since_landlord_demand: 5,
            ..baseline_california_sale_with_10_day_clause_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantEstoppelCertificateMode::CompliantTenantNotYetWithinResponseWindow
        );
    }

    #[test]
    fn tenant_at_exactly_10_days_still_within_window() {
        let input = Input {
            tenant_response: TenantResponse::TenantNotYetResponded,
            days_elapsed_since_landlord_demand: 10,
            ..baseline_california_sale_with_10_day_clause_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantEstoppelCertificateMode::CompliantTenantNotYetWithinResponseWindow
        );
    }

    #[test]
    fn tenant_missed_response_deadline_at_11_days_violation() {
        let input = Input {
            tenant_response: TenantResponse::TenantNotYetResponded,
            days_elapsed_since_landlord_demand: 11,
            ..baseline_california_sale_with_10_day_clause_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantEstoppelCertificateMode::ViolationTenantMissedResponseDeadline
        );
    }

    #[test]
    fn seven_day_clause_compliant() {
        let input = Input {
            lease_estoppel_clause: LeaseEstoppelClauseType::SevenDayEstoppelClause,
            landlord_demanded_response_in_days: 7,
            ..baseline_california_sale_with_10_day_clause_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantEstoppelCertificateMode::CompliantTenantSignedEstoppelWithinTimeframe
        );
        assert_eq!(result.allowed_response_days, 7);
    }

    #[test]
    fn statutory_reasonable_time_defaults_to_10_days() {
        let input = Input {
            lease_estoppel_clause: LeaseEstoppelClauseType::StatutoryReasonableTimeNoExplicitDays,
            landlord_demanded_response_in_days: 10,
            ..baseline_california_sale_with_10_day_clause_compliant()
        };
        let result = check(&input);
        assert_eq!(result.allowed_response_days, 10);
    }

    #[test]
    fn refinancing_trigger_treated_same_as_sale() {
        let input = Input {
            transactional_trigger: LandlordTransactionalTrigger::Refinancing,
            ..baseline_california_sale_with_10_day_clause_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            TenantEstoppelCertificateMode::CompliantTenantSignedEstoppelWithinTimeframe
        );
    }

    #[test]
    fn citations_pin_evidence_code_622_and_typical_clause() {
        let result = check(&baseline_california_sale_with_10_day_clause_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("California Evidence Code § 622"));
        assert!(joined.contains("conclusively presumed true"));
        assert!(joined.contains("within 10 days after request"));
        assert!(joined.contains("absent lease provision"));
        assert!(joined.contains("breach of lease"));
    }

    #[test]
    fn constant_pin_response_window_and_evidence_code() {
        assert_eq!(ESTOPPEL_TYPICAL_RESPONSE_DAYS, 10);
        assert_eq!(ESTOPPEL_MINIMUM_REASONABLE_DAYS, 7);
        assert_eq!(ESTOPPEL_CALIFORNIA_EVIDENCE_CODE_SECTION, 622);
    }
}
