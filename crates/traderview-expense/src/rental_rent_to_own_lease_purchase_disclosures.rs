//! Multi-State Residential Rent-to-Own / Lease-Purchase
//! Disclosure Requirements Compliance Module.
//!
//! Pure-compute check for landlord-seller compliance with state
//! executory-contract / lease-purchase disclosure statutes when
//! offering a residential rent-to-own, lease-option, contract-
//! for-deed, or installment land contract arrangement. Trader-
//! landlord critical because executory contracts longer than
//! 180 days (Texas) trigger extensive disclosure regimes,
//! purchaser cancellation rights, and Residential Mortgage Loan
//! Originator (RMLO) licensing under the federal SAFE Act +
//! Dodd-Frank owner-financing exceptions.
//!
//! Web research (verified 2026-06-03):
//! - **Texas Property Code Subchapter D §§ 5.061-5.085**
//!   (Executory Contract for Conveyance): applies to executory
//!   contracts for residential property OR lots ≤ 1 acre used or
//!   to be used as purchaser's residence. Contract for deed,
//!   lease option, or purchase option longer than **180 days** =
//!   executory contract. An option to purchase combined or
//!   executed concurrently with a residential lease = executory
//!   contract for conveyance ([Texas Property Code § 5.061
//!   public.law](https://texas.public.law/statutes/tex._prop._code_section_5.061);
//!   Drew Shirley — Texas Property Code Chapter 5 Subchapter D).
//! - **Texas § 5.069 REQUIRED DISCLOSURES**: recent survey or
//!   current plat; copies of liens, restrictive covenants, and
//!   easements; statutory disclosure; non-subdivision property
//!   notice; tax certificates; copy of insurance policy; **7-day
//!   notice letter** waiting period before signing; **annual
//!   accounting** statement under § 5.077.
//! - **Texas § 5.072 RIGHT OF RESCISSION**: purchaser has 14-day
//!   right of rescission after receiving disclosures.
//! - **Texas § 5.074 RIGHT OF CANCELLATION**: purchaser has
//!   unilateral right to cancel within 14 days.
//! - **Texas § 5.079 RECORDING**: executory contract must be
//!   recorded within **30 days**.
//! - **SAFE Act** (S.A.F.E. Mortgage Licensing Act of 2008;
//!   12 U.S.C. § 5101 et seq.) and **T-SAFE** (Texas equivalent):
//!   sellers of NON-HOMESTEAD property to NON-FAMILY MEMBERS
//!   require a **Residential Mortgage Loan Originator (RMLO)
//!   license** ([LoneStarLandLaw — Seller Finance in Texas
//!   Residential Sales Transactions](https://lonestarlandlaw.com/owner-finance-in-texas-residential-sales-transactions/)).
//! - **Dodd-Frank §§ 1402-1403 OWNER-FINANCING EXCEPTION**: 3 or
//!   fewer properties per year + no negative amortization +
//!   maximum interest rate increase + fixed-rate for at least
//!   5 years.
//! - **California Civ. Code § 2985** (installment land contract /
//!   contract of sale): basic statute requiring disclosures.
//! - **Maryland Real Property Code § 10-101**: executory contract
//!   requirements analog.
//! - **Illinois Residential Real Property Lease-Purchase Act**
//!   (765 ILCS 71/) — major NEW statute effective Jan 1, 2025.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const TEXAS_5_061_EXECUTORY_CONTRACT_THRESHOLD_DAYS: u32 = 180;
pub const TEXAS_5_072_RIGHT_OF_RESCISSION_DAYS: u32 = 14;
pub const TEXAS_5_074_RIGHT_OF_CANCELLATION_DAYS: u32 = 14;
pub const TEXAS_5_079_RECORDING_DAYS: u32 = 30;
pub const TEXAS_5_069_NOTICE_LETTER_WAITING_DAYS: u32 = 7;
pub const TEXAS_5_077_ANNUAL_ACCOUNTING_REQUIRED: bool = true;
pub const TEXAS_5_061_MAX_LOT_SIZE_ACRES: u32 = 1;
pub const DODD_FRANK_OWNER_FINANCING_MAX_PROPERTIES_PER_YEAR: u32 = 3;
pub const DODD_FRANK_OWNER_FINANCING_MIN_FIXED_RATE_YEARS: u32 = 5;
pub const SAFE_ACT_ENACTMENT_YEAR: u32 = 2008;
pub const ILLINOIS_765_71_EFFECTIVE_YEAR: u32 = 2025;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LeasePurchaseJurisdiction {
    Texas5_061ExecutoryContract,
    California2985InstallmentLandContract,
    Maryland10_101ExecutoryContract,
    Illinois765_71_1ResidentialLeasePurchase,
    OtherStateCommonLawOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractType {
    ContractForDeedOver180Days,
    LeaseOptionOver180Days,
    PurchaseOptionConcurrentWithLease,
    ShortTermLeaseOptionUnder180Days,
    ResidentialLeaseOnlyNoOption,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SellerType {
    HomesteadSellerToFamilyMemberSafeActExempt,
    NonHomesteadSellerToNonFamilyMemberRmloRequired,
    DoddFrankSmallOwnerFinancingExempt3OrFewerProperties,
    InstitutionalSellerLargePortfolio,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LeasePurchaseDisclosureMode {
    NotApplicableShortTermLeaseUnder180Days,
    NotApplicableNoLeaseToOwnComponent,
    CompliantTexasAllExecutoryContractDisclosuresProvided,
    CompliantTexas14DayCancellationRespected,
    CompliantTexasAnnualAccountingProvided,
    CompliantTexasRmloLicenseObtainedForNonHomesteadNonFamily,
    CompliantDoddFrankOwnerFinancingExceptionSatisfied,
    CompliantSafeActHomesteadFamilyExemption,
    CompliantCaliforniaCiv2985DisclosuresProvided,
    CompliantIllinois765_71_1DisclosuresProvided,
    ViolationTexasExecutoryContractDisclosuresMissing,
    ViolationTexasSevenDayNoticeLetterNotProvided,
    ViolationTexasAnnualAccountingNotProvided,
    ViolationTexasContractNotRecordedWithin30Days,
    ViolationSafeActRmloLicenseNotObtained,
    ViolationDoddFrankOwnerFinancingExceptionExceeded,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub jurisdiction: LeasePurchaseJurisdiction,
    pub contract_type: ContractType,
    pub seller_type: SellerType,
    pub all_disclosures_under_texas_5_069_provided: bool,
    pub seven_day_notice_letter_provided: bool,
    pub annual_accounting_provided: bool,
    pub contract_recorded_within_30_days: bool,
    pub purchaser_cancellation_respected_within_14_days: bool,
    pub safe_act_rmlo_license_obtained: bool,
    pub properties_owner_financed_in_calendar_year: u32,
    pub fixed_rate_term_years: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: LeasePurchaseDisclosureMode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
}

pub type RentalRentToOwnLeasePurchaseDisclosuresInput = Input;
pub type RentalRentToOwnLeasePurchaseDisclosuresOutput = Output;
pub type RentalRentToOwnLeasePurchaseDisclosuresResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Tex. Prop. Code Subchapter D §§ 5.061-5.085 — executory contract for conveyance applies to residential property + lots ≤ 1 acre; contract for deed, lease option, or purchase option > 180 days = executory contract".to_string(),
        "Tex. Prop. Code § 5.069 — required disclosures: survey/plat + liens + covenants + easements + statutory disclosure + non-subdivision notice + tax certificates + insurance policy + 7-day notice letter + annual accounting".to_string(),
        "Tex. Prop. Code § 5.072 — 14-day right of rescission after receiving disclosures".to_string(),
        "Tex. Prop. Code § 5.074 — 14-day unilateral right of cancellation".to_string(),
        "Tex. Prop. Code § 5.077 — annual accounting statement required".to_string(),
        "Tex. Prop. Code § 5.079 — executory contract must be recorded within 30 days".to_string(),
        "SAFE Act (S.A.F.E. Mortgage Licensing Act of 2008; 12 U.S.C. § 5101 et seq.) + T-SAFE — Residential Mortgage Loan Originator (RMLO) license required for sellers of non-homestead property to non-family members".to_string(),
        "Dodd-Frank §§ 1402-1403 owner-financing exception — 3 or fewer properties per year + no negative amortization + max interest rate increase + fixed-rate ≥ 5 years".to_string(),
        "Cal. Civ. Code § 2985 — installment land contract / contract of sale; basic disclosure statute".to_string(),
        "Md. Real Prop. Code § 10-101 — executory contract requirements analog".to_string(),
        "Illinois Residential Real Property Lease-Purchase Act (765 ILCS 71/) — effective Jan 1, 2025 — new statute".to_string(),
        "CFPB Reg Z (Truth in Lending Act implementation) — treats lease-purchase contracts as 'consumer credit' subject to disclosure requirements".to_string(),
    ];

    if input.contract_type == ContractType::ResidentialLeaseOnlyNoOption {
        return Output {
            mode: LeasePurchaseDisclosureMode::NotApplicableNoLeaseToOwnComponent,
            statutory_basis: "Pure residential lease without purchase option not subject to executory contract statutes".to_string(),
            notes: "Pure residential lease with no purchase option / contract-for-deed component; executory contract disclosure statutes inapplicable.".to_string(),
            citations,
        };
    }

    if input.contract_type == ContractType::ShortTermLeaseOptionUnder180Days {
        return Output {
            mode: LeasePurchaseDisclosureMode::NotApplicableShortTermLeaseUnder180Days,
            statutory_basis: "Tex. Prop. Code § 5.062 — executory contract trigger only at > 180 days".to_string(),
            notes: "Lease option ≤ 180 days; below Texas § 5.062 executory contract threshold; standard lease law applies.".to_string(),
            citations,
        };
    }

    match input.jurisdiction {
        LeasePurchaseJurisdiction::Texas5_061ExecutoryContract => {
            if !input.all_disclosures_under_texas_5_069_provided {
                return Output {
                    mode: LeasePurchaseDisclosureMode::ViolationTexasExecutoryContractDisclosuresMissing,
                    statutory_basis: "Tex. Prop. Code § 5.069 — all required disclosures must be provided".to_string(),
                    notes: "VIOLATION § 5.069: not all required disclosures (survey + liens + covenants + easements + statutory disclosure + non-subdivision notice + tax certificates + insurance + 7-day notice + annual accounting) provided to purchaser.".to_string(),
                    citations,
                };
            }
            if !input.seven_day_notice_letter_provided {
                return Output {
                    mode: LeasePurchaseDisclosureMode::ViolationTexasSevenDayNoticeLetterNotProvided,
                    statutory_basis: "Tex. Prop. Code § 5.069 — 7-day notice letter waiting period required".to_string(),
                    notes: "VIOLATION § 5.069: 7-day notice letter waiting period before signing executory contract not provided.".to_string(),
                    citations,
                };
            }
            if !input.contract_recorded_within_30_days {
                return Output {
                    mode: LeasePurchaseDisclosureMode::ViolationTexasContractNotRecordedWithin30Days,
                    statutory_basis: "Tex. Prop. Code § 5.079 — executory contract must be recorded within 30 days".to_string(),
                    notes: "VIOLATION § 5.079: executory contract not recorded within 30 days.".to_string(),
                    citations,
                };
            }
            if !input.annual_accounting_provided {
                return Output {
                    mode: LeasePurchaseDisclosureMode::ViolationTexasAnnualAccountingNotProvided,
                    statutory_basis: "Tex. Prop. Code § 5.077 — annual accounting statement required".to_string(),
                    notes: "VIOLATION § 5.077: annual accounting statement not provided to purchaser.".to_string(),
                    citations,
                };
            }
            if input.seller_type == SellerType::NonHomesteadSellerToNonFamilyMemberRmloRequired
                && !input.safe_act_rmlo_license_obtained
            {
                return Output {
                    mode: LeasePurchaseDisclosureMode::ViolationSafeActRmloLicenseNotObtained,
                    statutory_basis: "SAFE Act + T-SAFE — RMLO license required for non-homestead non-family owner finance".to_string(),
                    notes: "VIOLATION SAFE Act: seller of non-homestead property to non-family member did not obtain Residential Mortgage Loan Originator (RMLO) license.".to_string(),
                    citations,
                };
            }
            if input.seller_type == SellerType::DoddFrankSmallOwnerFinancingExempt3OrFewerProperties
                && (input.properties_owner_financed_in_calendar_year
                    > DODD_FRANK_OWNER_FINANCING_MAX_PROPERTIES_PER_YEAR
                    || input.fixed_rate_term_years < DODD_FRANK_OWNER_FINANCING_MIN_FIXED_RATE_YEARS)
            {
                return Output {
                    mode: LeasePurchaseDisclosureMode::ViolationDoddFrankOwnerFinancingExceptionExceeded,
                    statutory_basis: "Dodd-Frank §§ 1402-1403 — owner-financing exception 3 properties/year max + fixed-rate ≥ 5 years".to_string(),
                    notes: format!(
                        "VIOLATION Dodd-Frank: claimed small-owner-financing exception but financed {} properties (max 3/year) OR fixed-rate term {} years (min 5).",
                        input.properties_owner_financed_in_calendar_year, input.fixed_rate_term_years
                    ),
                    citations,
                };
            }
            if input.seller_type == SellerType::DoddFrankSmallOwnerFinancingExempt3OrFewerProperties {
                return Output {
                    mode: LeasePurchaseDisclosureMode::CompliantDoddFrankOwnerFinancingExceptionSatisfied,
                    statutory_basis: "Dodd-Frank §§ 1402-1403 — small-owner-financing exception satisfied".to_string(),
                    notes: format!(
                        "COMPLIANT Dodd-Frank: {} properties owner-financed (≤ 3); fixed-rate term {} years (≥ 5); SAFE Act exception applies.",
                        input.properties_owner_financed_in_calendar_year, input.fixed_rate_term_years
                    ),
                    citations,
                };
            }
            if input.seller_type == SellerType::HomesteadSellerToFamilyMemberSafeActExempt {
                return Output {
                    mode: LeasePurchaseDisclosureMode::CompliantSafeActHomesteadFamilyExemption,
                    statutory_basis: "SAFE Act — homestead-to-family exemption applies".to_string(),
                    notes: "COMPLIANT SAFE Act: homestead seller to family member exempt from RMLO licensing.".to_string(),
                    citations,
                };
            }
            if input.purchaser_cancellation_respected_within_14_days {
                return Output {
                    mode: LeasePurchaseDisclosureMode::CompliantTexas14DayCancellationRespected,
                    statutory_basis: "Tex. Prop. Code § 5.072 + § 5.074 — 14-day cancellation rights respected".to_string(),
                    notes: "COMPLIANT § 5.072 + § 5.074: 14-day right of rescission and cancellation rights honored.".to_string(),
                    citations,
                };
            }
            Output {
                mode: LeasePurchaseDisclosureMode::CompliantTexasAllExecutoryContractDisclosuresProvided,
                statutory_basis: "Tex. Prop. Code §§ 5.061-5.085 — all executory contract requirements satisfied".to_string(),
                notes: "COMPLIANT Tex. Subchapter D: all § 5.069 disclosures + 7-day notice + 30-day recording + annual accounting + RMLO license obtained.".to_string(),
                citations,
            }
        }
        LeasePurchaseJurisdiction::California2985InstallmentLandContract => {
            if !input.all_disclosures_under_texas_5_069_provided {
                return Output {
                    mode: LeasePurchaseDisclosureMode::ViolationTexasExecutoryContractDisclosuresMissing,
                    statutory_basis: "Cal. Civ. Code § 2985 — installment land contract disclosures required".to_string(),
                    notes: "VIOLATION Cal. Civ. Code § 2985: installment land contract disclosure requirements not met.".to_string(),
                    citations,
                };
            }
            Output {
                mode: LeasePurchaseDisclosureMode::CompliantCaliforniaCiv2985DisclosuresProvided,
                statutory_basis: "Cal. Civ. Code § 2985 disclosures satisfied".to_string(),
                notes: "COMPLIANT Cal. Civ. Code § 2985: installment land contract disclosures provided.".to_string(),
                citations,
            }
        }
        LeasePurchaseJurisdiction::Illinois765_71_1ResidentialLeasePurchase => {
            if !input.all_disclosures_under_texas_5_069_provided {
                return Output {
                    mode: LeasePurchaseDisclosureMode::ViolationTexasExecutoryContractDisclosuresMissing,
                    statutory_basis: "Illinois Residential Real Property Lease-Purchase Act 765 ILCS 71/ — disclosures required".to_string(),
                    notes: "VIOLATION 765 ILCS 71/: Illinois Lease-Purchase Act disclosures not provided (effective Jan 1, 2025).".to_string(),
                    citations,
                };
            }
            Output {
                mode: LeasePurchaseDisclosureMode::CompliantIllinois765_71_1DisclosuresProvided,
                statutory_basis: "Illinois 765 ILCS 71/ disclosures satisfied".to_string(),
                notes: "COMPLIANT Illinois 765 ILCS 71/: Residential Real Property Lease-Purchase Act disclosures provided (effective Jan 1, 2025).".to_string(),
                citations,
            }
        }
        LeasePurchaseJurisdiction::Maryland10_101ExecutoryContract => {
            if !input.all_disclosures_under_texas_5_069_provided {
                return Output {
                    mode: LeasePurchaseDisclosureMode::ViolationTexasExecutoryContractDisclosuresMissing,
                    statutory_basis: "Md. Real Prop. Code § 10-101 — executory contract disclosures required".to_string(),
                    notes: "VIOLATION Md. Real Prop. Code § 10-101: Maryland executory contract disclosures not provided.".to_string(),
                    citations,
                };
            }
            Output {
                mode: LeasePurchaseDisclosureMode::CompliantTexasAllExecutoryContractDisclosuresProvided,
                statutory_basis: "Md. Real Prop. Code § 10-101 disclosures satisfied".to_string(),
                notes: "COMPLIANT Md. Real Prop. Code § 10-101: executory contract disclosures provided.".to_string(),
                citations,
            }
        }
        LeasePurchaseJurisdiction::OtherStateCommonLawOnly => Output {
            mode: LeasePurchaseDisclosureMode::CompliantTexasAllExecutoryContractDisclosuresProvided,
            statutory_basis: "Other state — common law applies; no codified executory contract regime".to_string(),
            notes: "Jurisdiction has no codified executory contract / lease-purchase disclosure regime; common law applies. Federal SAFE Act + Dodd-Frank still applicable.".to_string(),
            citations,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_texas_compliant() -> Input {
        Input {
            jurisdiction: LeasePurchaseJurisdiction::Texas5_061ExecutoryContract,
            contract_type: ContractType::ContractForDeedOver180Days,
            seller_type: SellerType::NonHomesteadSellerToNonFamilyMemberRmloRequired,
            all_disclosures_under_texas_5_069_provided: true,
            seven_day_notice_letter_provided: true,
            annual_accounting_provided: true,
            contract_recorded_within_30_days: true,
            purchaser_cancellation_respected_within_14_days: true,
            safe_act_rmlo_license_obtained: true,
            properties_owner_financed_in_calendar_year: 2,
            fixed_rate_term_years: 7,
        }
    }

    #[test]
    fn residential_lease_only_not_applicable() {
        let input = Input {
            contract_type: ContractType::ResidentialLeaseOnlyNoOption,
            ..baseline_texas_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            LeasePurchaseDisclosureMode::NotApplicableNoLeaseToOwnComponent
        );
    }

    #[test]
    fn short_term_lease_under_180_days_not_applicable() {
        let input = Input {
            contract_type: ContractType::ShortTermLeaseOptionUnder180Days,
            ..baseline_texas_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            LeasePurchaseDisclosureMode::NotApplicableShortTermLeaseUnder180Days
        );
    }

    #[test]
    fn texas_baseline_compliant() {
        let result = check(&baseline_texas_compliant());
        assert_eq!(
            result.mode,
            LeasePurchaseDisclosureMode::CompliantTexas14DayCancellationRespected
        );
    }

    #[test]
    fn texas_all_disclosures_missing_violation() {
        let input = Input {
            all_disclosures_under_texas_5_069_provided: false,
            ..baseline_texas_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            LeasePurchaseDisclosureMode::ViolationTexasExecutoryContractDisclosuresMissing
        );
    }

    #[test]
    fn texas_seven_day_notice_missing_violation() {
        let input = Input {
            seven_day_notice_letter_provided: false,
            ..baseline_texas_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            LeasePurchaseDisclosureMode::ViolationTexasSevenDayNoticeLetterNotProvided
        );
    }

    #[test]
    fn texas_contract_not_recorded_30_days_violation() {
        let input = Input {
            contract_recorded_within_30_days: false,
            ..baseline_texas_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            LeasePurchaseDisclosureMode::ViolationTexasContractNotRecordedWithin30Days
        );
    }

    #[test]
    fn texas_annual_accounting_missing_violation() {
        let input = Input {
            annual_accounting_provided: false,
            ..baseline_texas_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            LeasePurchaseDisclosureMode::ViolationTexasAnnualAccountingNotProvided
        );
    }

    #[test]
    fn safe_act_rmlo_license_missing_violation() {
        let input = Input {
            safe_act_rmlo_license_obtained: false,
            ..baseline_texas_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            LeasePurchaseDisclosureMode::ViolationSafeActRmloLicenseNotObtained
        );
    }

    #[test]
    fn dodd_frank_3_or_fewer_properties_compliant() {
        let input = Input {
            seller_type: SellerType::DoddFrankSmallOwnerFinancingExempt3OrFewerProperties,
            properties_owner_financed_in_calendar_year: 3,
            ..baseline_texas_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            LeasePurchaseDisclosureMode::CompliantDoddFrankOwnerFinancingExceptionSatisfied
        );
    }

    #[test]
    fn dodd_frank_4_properties_violation() {
        let input = Input {
            seller_type: SellerType::DoddFrankSmallOwnerFinancingExempt3OrFewerProperties,
            properties_owner_financed_in_calendar_year: 4,
            ..baseline_texas_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            LeasePurchaseDisclosureMode::ViolationDoddFrankOwnerFinancingExceptionExceeded
        );
    }

    #[test]
    fn dodd_frank_fixed_rate_under_5_years_violation() {
        let input = Input {
            seller_type: SellerType::DoddFrankSmallOwnerFinancingExempt3OrFewerProperties,
            properties_owner_financed_in_calendar_year: 2,
            fixed_rate_term_years: 4,
            ..baseline_texas_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            LeasePurchaseDisclosureMode::ViolationDoddFrankOwnerFinancingExceptionExceeded
        );
    }

    #[test]
    fn homestead_to_family_safe_act_exemption_compliant() {
        let input = Input {
            seller_type: SellerType::HomesteadSellerToFamilyMemberSafeActExempt,
            safe_act_rmlo_license_obtained: false,
            ..baseline_texas_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            LeasePurchaseDisclosureMode::CompliantSafeActHomesteadFamilyExemption
        );
    }

    #[test]
    fn california_2985_disclosures_compliant() {
        let input = Input {
            jurisdiction: LeasePurchaseJurisdiction::California2985InstallmentLandContract,
            ..baseline_texas_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            LeasePurchaseDisclosureMode::CompliantCaliforniaCiv2985DisclosuresProvided
        );
    }

    #[test]
    fn california_2985_disclosures_missing_violation() {
        let input = Input {
            jurisdiction: LeasePurchaseJurisdiction::California2985InstallmentLandContract,
            all_disclosures_under_texas_5_069_provided: false,
            ..baseline_texas_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            LeasePurchaseDisclosureMode::ViolationTexasExecutoryContractDisclosuresMissing
        );
    }

    #[test]
    fn illinois_765_71_disclosures_compliant() {
        let input = Input {
            jurisdiction: LeasePurchaseJurisdiction::Illinois765_71_1ResidentialLeasePurchase,
            ..baseline_texas_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            LeasePurchaseDisclosureMode::CompliantIllinois765_71_1DisclosuresProvided
        );
    }

    #[test]
    fn maryland_10_101_disclosures_compliant() {
        let input = Input {
            jurisdiction: LeasePurchaseJurisdiction::Maryland10_101ExecutoryContract,
            ..baseline_texas_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            LeasePurchaseDisclosureMode::CompliantTexasAllExecutoryContractDisclosuresProvided
        );
    }

    #[test]
    fn other_state_common_law_default() {
        let input = Input {
            jurisdiction: LeasePurchaseJurisdiction::OtherStateCommonLawOnly,
            ..baseline_texas_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            LeasePurchaseDisclosureMode::CompliantTexasAllExecutoryContractDisclosuresProvided
        );
    }

    #[test]
    fn lease_option_over_180_days_subject_to_texas_5_061() {
        let input = Input {
            contract_type: ContractType::LeaseOptionOver180Days,
            ..baseline_texas_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            LeasePurchaseDisclosureMode::CompliantTexas14DayCancellationRespected
        );
    }

    #[test]
    fn purchase_option_concurrent_with_lease_executory_contract() {
        let input = Input {
            contract_type: ContractType::PurchaseOptionConcurrentWithLease,
            ..baseline_texas_compliant()
        };
        let result = check(&input);
        assert_eq!(
            result.mode,
            LeasePurchaseDisclosureMode::CompliantTexas14DayCancellationRespected
        );
    }

    #[test]
    fn citations_pin_texas_subchapter_d_and_safe_act() {
        let result = check(&baseline_texas_compliant());
        let joined = result.citations.join(" | ");
        assert!(joined.contains("Tex. Prop. Code Subchapter D"));
        assert!(joined.contains("§§ 5.061-5.085"));
        assert!(joined.contains("180 days"));
        assert!(joined.contains("Tex. Prop. Code § 5.069"));
        assert!(joined.contains("Tex. Prop. Code § 5.072"));
        assert!(joined.contains("Tex. Prop. Code § 5.074"));
        assert!(joined.contains("Tex. Prop. Code § 5.077"));
        assert!(joined.contains("Tex. Prop. Code § 5.079"));
        assert!(joined.contains("SAFE Act"));
        assert!(joined.contains("12 U.S.C. § 5101"));
        assert!(joined.contains("Dodd-Frank"));
        assert!(joined.contains("3 or fewer properties"));
        assert!(joined.contains("Cal. Civ. Code § 2985"));
        assert!(joined.contains("Md. Real Prop. Code § 10-101"));
        assert!(joined.contains("765 ILCS 71/"));
        assert!(joined.contains("Jan 1, 2025"));
        assert!(joined.contains("CFPB Reg Z"));
    }

    #[test]
    fn constant_pin_texas_thresholds_and_safe_act_dates() {
        assert_eq!(TEXAS_5_061_EXECUTORY_CONTRACT_THRESHOLD_DAYS, 180);
        assert_eq!(TEXAS_5_072_RIGHT_OF_RESCISSION_DAYS, 14);
        assert_eq!(TEXAS_5_074_RIGHT_OF_CANCELLATION_DAYS, 14);
        assert_eq!(TEXAS_5_079_RECORDING_DAYS, 30);
        assert_eq!(TEXAS_5_069_NOTICE_LETTER_WAITING_DAYS, 7);
        assert!(TEXAS_5_077_ANNUAL_ACCOUNTING_REQUIRED);
        assert_eq!(TEXAS_5_061_MAX_LOT_SIZE_ACRES, 1);
        assert_eq!(DODD_FRANK_OWNER_FINANCING_MAX_PROPERTIES_PER_YEAR, 3);
        assert_eq!(DODD_FRANK_OWNER_FINANCING_MIN_FIXED_RATE_YEARS, 5);
        assert_eq!(SAFE_ACT_ENACTMENT_YEAR, 2008);
        assert_eq!(ILLINOIS_765_71_EFFECTIVE_YEAR, 2025);
    }
}
