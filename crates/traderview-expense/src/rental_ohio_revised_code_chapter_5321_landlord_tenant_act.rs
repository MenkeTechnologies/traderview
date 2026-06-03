//! Ohio Revised Code Chapter 5321 — Ohio Landlord-Tenant
//! Act Compliance Module — codified at O.R.C. §§ 5321.01
//! through 5321.21. Pure-compute check for trader-landlord
//! compliance with Ohio's foundational statewide residential
//! tenancy regime.
//!
//! Enacted as **1974 Am. H.B. 144** and made effective on
//! **NOVEMBER 4, 1974**. Ohio is a major Midwestern rental
//! market covering metropolitan Columbus, Cleveland,
//! Cincinnati, Toledo, Akron, Dayton, and Youngstown. The
//! Act is supplemented by **Ohio R.C. Chapter 1923 Forcible
//! Entry and Detainer** for eviction procedure.
//!
//! Web research (verified 2026-06-03):
//! - **Enactment**: Ohio Landlord-Tenant Act enacted as **1974 Am. H.B. 144** ("An Act to Enact Chapter 5321 of the Revised Code"); effective **November 4, 1974**; codified at **O.R.C. §§ 5321.01 through 5321.21** (Title 53 Real Property, Chapter 5321 Landlords and Tenants) ([Ohio Laws — Chapter 5321 Revised Code](https://codes.ohio.gov/ohio-revised-code/chapter-5321); [Ohio Laws — Section 5321.16 Procedures for Security Deposits](https://codes.ohio.gov/ohio-revised-code/section-5321.16); [Ohio Laws — Section 5321.07 Tenant Remedies for Landlord Non-Compliance](https://codes.ohio.gov/ohio-revised-code/section-5321.07); [Ohio Laws — Section 5321.09 Landlord Application for Release of Rent](https://codes.ohio.gov/ohio-revised-code/section-5321.09); [Justia — 2024 Ohio R.C. § 5321.16 Procedures for Security Deposits](https://law.justia.com/codes/ohio/title-53/chapter-5321/section-5321-16/); [Justia — 2024 Ohio R.C. § 5321.04 Landlord Obligations](https://law.justia.com/codes/ohio/title-53/chapter-5321/section-5321-04/); [Hamilton County Law Library — Evictions Ohio Landlord/Tenant Law](https://libguides.hamilton-co.org/c.php?g=392898&p=2668993); [Hamilton County Law Library — Conditions/Rent Escrow Ohio Landlord/Tenant Law](https://libguides.hamilton-co.org/c.php?g=392898&p=2668910); [Franklin County Law Library — Tenant Resources Ohio Landlord/Tenant Law](https://fclawlib.libguides.com/ohiolandlordtenantlaw/tenant); [Newark, Ohio — ORC 5321 Ohio's Landlord and Tenant Law An Overview PDF](https://www.newarkohio.gov/wp-content/uploads/2022/08/FH-OhioLandlordTenantLaw-AnOverview.pdf); [COHHIO — Ohio Landlord Tenant Law PDF](https://cohhio.org/wp-content/uploads/2023/04/Ohio-Landlord-Tenant-Law.pdf); [MPC Law — Know Your Tenant Rights in Ohio: A Guide to ORC 5321](https://mpclawllc.com/know-your-tenant-rights-in-ohio-a-guide-to-orc-5321/); [Tenant Rights — Ohio Security Deposit Rules Limits and Return Deadlines](https://tenant-rights.com/ohio/ohio-security-deposit-rules-limits-and-return-deadlines); [Deposit Forensics — Ohio Security Deposit Laws 2026 30-Day Return Rule](https://www.depositforensics.com/ohio-security-deposit-laws); [FindLaw — Ohio Revised Code Title LIII Real Property § 5321.16](https://codes.findlaw.com/oh/title-liii-real-property/oh-rev-code-sect-5321-16/)).
//! - **§ 5321.04 Landlord Obligations**: landlord must (1) comply with building, housing, health, and safety codes; (2) keep all common areas safe and sanitary; (3) keep all electrical, plumbing, sanitary, heating, ventilating, and air-conditioning fixtures and appliances supplied by landlord in good and safe working order; (4) maintain all appliances and equipment supplied by landlord; (5) provide and maintain appropriate receptacles for ashes, garbage, rubbish; (6) supply running water + reasonable amount of hot water at all times + reasonable heat at all times; (7) not abuse the landlord's right of entry; (8) give the tenant at least **24 HOURS' notice** before entering the premises and enter only at reasonable times; (9) promptly commence an action under R.C. Chapter 1923 to remove the tenant after compliance with § 5321.17(C); (10) deliver the rented premises to the tenant in compliance with the rental agreement and in a fit and habitable condition.
//! - **§ 5321.05 Tenant Obligations**: tenant must (1) keep premises safe and sanitary; (2) dispose of rubbish; (3) keep plumbing fixtures clean; (4) use electrical and plumbing fixtures properly; (5) comply with building, housing, health, and safety codes; (6) maintain appliances supplied by tenant; (7) conduct themselves in a manner that will not disturb neighbors; (8) not damage premises; (9) not deny landlord reasonable access.
//! - **§ 5321.07 Tenant Remedies for Landlord Non-Compliance**: tenant may give written notice to landlord specifying acts / omissions / code violations + provide reasonable time (presumption 30 days) for landlord to remedy; if landlord fails to remedy, tenant may **(1) DEPOSIT RENT IN ESCROW with municipal or county court clerk**; **(2) APPLY TO COURT FOR REDUCTION OF RENT**; or **(3) TERMINATE RENTAL AGREEMENT**.
//! - **§ 5321.09 Landlord Application for Release of Rent**: landlord may apply to court for release of escrowed rent upon proof that landlord has remedied the condition.
//! - **§ 5321.12 Wrongful Retention Treble Damages**: landlord who wrongfully retains rent, security deposit, or last month's rent in bad faith is liable for treble damages plus reasonable attorney fees.
//! - **§ 5321.13 Retaliation Prohibited**: landlord may NOT retaliate against tenant for complaint to government agency about code violations, complaint to landlord about breach of obligation, organization of tenants, or otherwise asserting Chapter 5321 rights.
//! - **§ 5321.16 Security Deposit Procedures**: (1) any security deposit in excess of **$50 OR ONE MONTH'S RENT (whichever greater)** must bear interest on the excess at **5 PERCENT PER ANNUM** if tenant remains in possession for **6 MONTHS OR MORE** (computed and paid annually); (2) any deduction must be **ITEMIZED IN WRITING** and identified in a notice delivered to tenant together with amount due **WITHIN 30 DAYS** after termination of rental agreement and delivery of possession; (3) if landlord fails to comply, tenant may recover deposit and money due **PLUS DAMAGES EQUAL TO AMOUNT WRONGFULLY WITHHELD (DOUBLE DAMAGES)** AND reasonable attorney fees.
//! - **§ 5321.17 Termination Notice Requirements**: (A) **MONTH-TO-MONTH tenancy** = **30-DAY NOTICE**; (B) **WEEK-TO-WEEK tenancy** = **7-DAY NOTICE**; (C) landlord must give **3-DAY NOTICE TO LEAVE PREMISES** before commencing R.C. Chapter 1923 forcible entry and detainer action.
//! - **§ 5321.18 Written Rental Agreement Required for 90+ Days (Ohio Statute of Frauds)**: rental agreement for term exceeding 90 days must be in writing (R.C. § 1335.05 Statute of Frauds applies).
//! - **R.C. § 1923.02 Forcible Entry and Detainer Jurisdiction**: municipal and county courts have jurisdiction over eviction actions.
//! - **R.C. § 1923.04 Notice for Eviction**: landlord must serve **3-DAY NOTICE TO LEAVE PREMISES** before commencing forcible entry and detainer action; notice must be served by certified mail OR personal delivery OR posting on premises.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

pub const OHIO_RC_5321_CHAPTER_NUMBER: u32 = 5321;
pub const OHIO_RC_5321_ENACTMENT_YEAR: u32 = 1974;
pub const OHIO_RC_5321_ENACTMENT_MONTH: u32 = 11;
pub const OHIO_RC_5321_ENACTMENT_DAY: u32 = 4;
pub const OHIO_RC_5321_ENABLING_AM_HB_NUMBER: u32 = 144;
pub const OHIO_RC_5321_SECURITY_DEPOSIT_INTEREST_THRESHOLD_DOLLARS: u64 = 50;
pub const OHIO_RC_5321_SECURITY_DEPOSIT_INTEREST_THRESHOLD_MONTHS_OF_RENT: u32 = 1;
pub const OHIO_RC_5321_SECURITY_DEPOSIT_INTEREST_RATE_BPS: u64 = 500;
pub const OHIO_RC_5321_SECURITY_DEPOSIT_INTEREST_MIN_HOLDING_MONTHS: u32 = 6;
pub const OHIO_RC_5321_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS: u32 = 30;
pub const OHIO_RC_5321_SECURITY_DEPOSIT_DOUBLE_DAMAGES_MULTIPLIER: u64 = 2;
pub const OHIO_RC_5321_LANDLORD_ENTRY_NOTICE_HOURS: u32 = 24;
pub const OHIO_RC_5321_MONTH_TO_MONTH_TERMINATION_NOTICE_DAYS: u32 = 30;
pub const OHIO_RC_5321_WEEK_TO_WEEK_TERMINATION_NOTICE_DAYS: u32 = 7;
pub const OHIO_RC_5321_LEASE_WRITING_REQUIRED_THRESHOLD_DAYS: u32 = 90;
pub const OHIO_RC_1923_PAY_OR_QUIT_NOTICE_DAYS: u32 = 3;
pub const OHIO_RC_5321_TREBLE_DAMAGES_MULTIPLIER: u64 = 3;
pub const OHIO_RC_5321_BASIS_POINT_DENOMINATOR: u64 = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyType {
    ResidentialRentalCoveredByChapter5321,
    CommercialRentalExempt,
    HotelMotelTransientLodgingExempt,
    InstitutionalCareFacilityExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TenancyTermLength {
    MonthToMonth,
    WeekToWeek,
    FixedTermUnderNinetyDays,
    FixedTermNinetyDaysOrMore,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceAspect {
    LandlordObligationsUnderSection5321_04,
    SecurityDepositInterestUnderSection5321_16A,
    SecurityDepositReturnAndItemizedDeductionsUnderSection5321_16B,
    SecurityDepositDoubleDamagesUnderSection5321_16C,
    LandlordEntryTwentyFourHourNoticeUnderSection5321_04A8,
    RentEscrowRemedyUnderSection5321_07,
    RetaliationProhibitedUnderSection5321_13,
    TerminationNoticeBasedOnTenancyTermUnderSection5321_17,
    ThreeDayPayOrQuitNoticeUnderSection1923_04,
    WrittenRentalAgreementRequiredForOverNinetyDaysUnderSection5321_18,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OhioRc5321Mode {
    NotApplicableTenancyExemptFromChapter5321,
    CompliantLandlordObligationsMet,
    CompliantSecurityDepositInterestPaidOrNotRequired,
    CompliantSecurityDepositReturnedAndItemizedDeductionsWithinThirtyDays,
    CompliantSecurityDepositNoWrongfulRetention,
    CompliantLandlordEntryTwentyFourHourNoticeProvided,
    CompliantRentEscrowAvailableUnderSection5321_07,
    CompliantNoRetaliation,
    CompliantTerminationNoticeMeetsStatutoryLengthForTenancyTerm,
    CompliantThreeDayPayOrQuitNoticeProvided,
    CompliantWrittenRentalAgreementForOverNinetyDays,
    ViolationLandlordObligationsBreached,
    ViolationSecurityDepositInterestNotPaid,
    ViolationSecurityDepositReturnedPastThirtyDayDeadlineDoubleDamages,
    ViolationSecurityDepositWrongfulRetentionDoubleDamagesPlusAttorneyFees,
    ViolationLandlordEntryWithoutTwentyFourHourNotice,
    ViolationRetaliatoryConduct,
    ViolationTerminationNoticeShorterThanStatutoryLengthForTenancyTerm,
    ViolationPayOrQuitNoticeShorterThanThreeDays,
    ViolationOralRentalAgreementOverNinetyDaysUnenforceable,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub tenancy_type: TenancyType,
    pub compliance_aspect: ComplianceAspect,
    pub tenancy_term_length: TenancyTermLength,
    pub monthly_rent_dollars: u64,
    pub security_deposit_dollars: u64,
    pub portion_of_deposit_wrongfully_withheld_dollars: u64,
    pub months_tenant_in_possession: u32,
    pub interest_paid_on_excess_deposit: bool,
    pub deposit_returned_and_itemized_within_window: bool,
    pub days_since_lease_termination_for_deposit_return: u32,
    pub landlord_obligations_met: bool,
    pub landlord_entry_notice_hours_given: u32,
    pub retaliatory_conduct_occurred: bool,
    pub termination_notice_days_given: u32,
    pub pay_or_quit_notice_days_given: u32,
    pub rental_agreement_in_writing: bool,
    pub rental_agreement_term_days: u32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub mode: OhioRc5321Mode,
    pub statutory_basis: String,
    pub notes: String,
    pub citations: Vec<String>,
    pub double_damages_remedy_dollars: u64,
}

pub type RentalOhioRevisedCodeChapter5321LandlordTenantActInput = Input;
pub type RentalOhioRevisedCodeChapter5321LandlordTenantActOutput = Output;
pub type RentalOhioRevisedCodeChapter5321LandlordTenantActResult = Output;

pub fn check(input: &Input) -> Output {
    compute(input)
}

pub fn compute(input: &Input) -> Output {
    let citations: Vec<String> = vec![
        "Ohio Landlord-Tenant Act — enacted as 1974 Am. H.B. 144 ('An Act to Enact Chapter 5321 of the Revised Code'); effective November 4, 1974; codified at Ohio Revised Code §§ 5321.01 through 5321.21 (Title 53 Real Property, Chapter 5321 Landlords and Tenants)".to_string(),
        "O.R.C. § 5321.04 Landlord Obligations — landlord must comply with building / housing / health / safety codes; keep common areas safe and sanitary; keep electrical / plumbing / sanitary / heating / ventilating / air-conditioning fixtures and appliances in good and safe working order; maintain appliances supplied by landlord; provide receptacles for ashes / garbage / rubbish; supply running water + reasonable hot water + reasonable heat; not abuse right of entry; give 24 HOURS' notice before entering; promptly commence R.C. Chapter 1923 action to remove tenant after § 5321.17(C) compliance; deliver premises in fit and habitable condition".to_string(),
        "O.R.C. § 5321.05 Tenant Obligations — tenant must keep premises safe and sanitary; dispose of rubbish; keep plumbing fixtures clean; use electrical and plumbing fixtures properly; comply with building / housing / health / safety codes; maintain tenant-supplied appliances; not disturb neighbors; not damage premises; not deny landlord reasonable access".to_string(),
        "O.R.C. § 5321.07 Tenant Remedies for Landlord Non-Compliance — tenant may give written notice + provide reasonable time (presumption 30 days) for landlord to remedy; if landlord fails to remedy, tenant may (1) DEPOSIT RENT IN ESCROW with municipal or county court clerk; (2) APPLY TO COURT FOR REDUCTION OF RENT; or (3) TERMINATE RENTAL AGREEMENT".to_string(),
        "O.R.C. § 5321.09 Landlord Application for Release of Rent — landlord may apply to court for release of escrowed rent upon proof that landlord has remedied the condition".to_string(),
        "O.R.C. § 5321.12 Wrongful Retention Treble Damages — landlord who wrongfully retains rent, security deposit, or last month's rent in bad faith is liable for treble damages plus reasonable attorney fees".to_string(),
        "O.R.C. § 5321.13 Retaliation Prohibited — landlord may NOT retaliate against tenant for complaint to government agency about code violations; complaint to landlord about breach of obligation; organization of tenants; or otherwise asserting Chapter 5321 rights".to_string(),
        "O.R.C. § 5321.16(A) Security Deposit Interest Procedure — any security deposit in excess of $50 OR ONE MONTH'S RENT (whichever greater) must bear interest on the excess at 5 PERCENT PER ANNUM if tenant remains in possession for 6 MONTHS OR MORE (computed and paid annually)".to_string(),
        "O.R.C. § 5321.16(B) Security Deposit Return Procedure — any deduction must be ITEMIZED IN WRITING and identified in a notice delivered to tenant together with amount due WITHIN 30 DAYS after termination of rental agreement and delivery of possession".to_string(),
        "O.R.C. § 5321.16(C) Double Damages Procedure — if landlord fails to comply with § 5321.16(B), tenant may recover deposit and money due PLUS DAMAGES EQUAL TO AMOUNT WRONGFULLY WITHHELD (DOUBLE DAMAGES) AND reasonable attorney fees".to_string(),
        "O.R.C. § 5321.17(A) Month-to-Month Tenancy Termination — 30-DAY notice required".to_string(),
        "O.R.C. § 5321.17(B) Week-to-Week Tenancy Termination — 7-DAY notice required".to_string(),
        "O.R.C. § 5321.17(C) Pre-Eviction Notice — landlord must give 3-DAY NOTICE TO LEAVE PREMISES before commencing R.C. Chapter 1923 forcible entry and detainer action".to_string(),
        "O.R.C. § 5321.18 Written Rental Agreement Required — rental agreement for term exceeding 90 days must be in writing (Ohio Statute of Frauds R.C. § 1335.05 applies)".to_string(),
        "O.R.C. § 1923.02 Forcible Entry and Detainer Jurisdiction — municipal and county courts have jurisdiction over eviction actions".to_string(),
        "O.R.C. § 1923.04 Notice for Eviction — landlord must serve 3-DAY NOTICE TO LEAVE PREMISES before commencing forcible entry and detainer action; notice must be served by certified mail OR personal delivery OR posting on premises".to_string(),
        "Ohio Laws + Justia + Hamilton County Law Library + Franklin County Law Library + Newark Ohio + COHHIO + MPC Law + Tenant Rights + Deposit Forensics + FindLaw — primary statutory text and practitioner guides".to_string(),
    ];

    if input.tenancy_type != TenancyType::ResidentialRentalCoveredByChapter5321 {
        return Output {
            mode: OhioRc5321Mode::NotApplicableTenancyExemptFromChapter5321,
            statutory_basis: "O.R.C. § 5321.01 — Chapter 5321 applies only to residential leaseholds; commercial / hotel-motel / institutional exempt".to_string(),
            notes: "NOT APPLICABLE: tenancy type exempt from O.R.C. Chapter 5321 (commercial rental; hotel/motel transient lodging; institutional care facility).".to_string(),
            citations,
            double_damages_remedy_dollars: 0,
        };
    }

    let double_damages = input
        .portion_of_deposit_wrongfully_withheld_dollars
        .saturating_mul(OHIO_RC_5321_SECURITY_DEPOSIT_DOUBLE_DAMAGES_MULTIPLIER);

    match input.compliance_aspect {
        ComplianceAspect::LandlordObligationsUnderSection5321_04 => {
            if input.landlord_obligations_met {
                Output {
                    mode: OhioRc5321Mode::CompliantLandlordObligationsMet,
                    statutory_basis: "O.R.C. § 5321.04 — landlord obligations met".to_string(),
                    notes: "COMPLIANT: landlord met § 5321.04 obligations (codes + common areas + fixtures + appliances + receptacles + water + heat + entry + habitable condition).".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            } else {
                Output {
                    mode: OhioRc5321Mode::ViolationLandlordObligationsBreached,
                    statutory_basis: "O.R.C. § 5321.04 — landlord obligations breached".to_string(),
                    notes: "VIOLATION: landlord breached § 5321.04 obligations; tenant remedies under § 5321.07 (rent escrow + court reduction + termination).".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            }
        }
        ComplianceAspect::SecurityDepositInterestUnderSection5321_16A => {
            let threshold = input
                .monthly_rent_dollars
                .max(OHIO_RC_5321_SECURITY_DEPOSIT_INTEREST_THRESHOLD_DOLLARS);
            if input.security_deposit_dollars <= threshold
                || input.months_tenant_in_possession < OHIO_RC_5321_SECURITY_DEPOSIT_INTEREST_MIN_HOLDING_MONTHS
            {
                Output {
                    mode: OhioRc5321Mode::CompliantSecurityDepositInterestPaidOrNotRequired,
                    statutory_basis: "O.R.C. § 5321.16(A) — interest requirement not triggered (deposit at or below threshold OR holding period under 6 months)".to_string(),
                    notes: "NOT TRIGGERED: § 5321.16(A) interest requirement does not attach (deposit at or below greater of $50 or one month's rent OR tenant in possession less than 6 months).".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            } else if input.interest_paid_on_excess_deposit {
                Output {
                    mode: OhioRc5321Mode::CompliantSecurityDepositInterestPaidOrNotRequired,
                    statutory_basis: "O.R.C. § 5321.16(A) — 5 % per annum interest paid on excess deposit".to_string(),
                    notes: "COMPLIANT: landlord paid 5 % per annum interest on portion of security deposit exceeding greater of $50 or one month's rent for tenant in possession at least 6 months.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            } else {
                Output {
                    mode: OhioRc5321Mode::ViolationSecurityDepositInterestNotPaid,
                    statutory_basis: "O.R.C. § 5321.16(A) — 5 % per annum interest not paid on excess deposit".to_string(),
                    notes: "VIOLATION: landlord failed to pay 5 % per annum interest on portion of security deposit exceeding greater of $50 or one month's rent; tenant in possession 6+ months entitled to annual interest.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            }
        }
        ComplianceAspect::SecurityDepositReturnAndItemizedDeductionsUnderSection5321_16B => {
            if input.deposit_returned_and_itemized_within_window
                && input.days_since_lease_termination_for_deposit_return
                    <= OHIO_RC_5321_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS
            {
                Output {
                    mode: OhioRc5321Mode::CompliantSecurityDepositReturnedAndItemizedDeductionsWithinThirtyDays,
                    statutory_basis: "O.R.C. § 5321.16(B) — deposit returned with itemized statement within 30-day statutory deadline".to_string(),
                    notes: "COMPLIANT: landlord delivered itemized statement of deductions AND remaining portion of security deposit within 30 days after termination of rental agreement and delivery of possession.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            } else {
                Output {
                    mode: OhioRc5321Mode::ViolationSecurityDepositReturnedPastThirtyDayDeadlineDoubleDamages,
                    statutory_basis: "O.R.C. § 5321.16(B) + § 5321.16(C) — deposit not returned within 30-day deadline; double damages remedy".to_string(),
                    notes: "VIOLATION: landlord missed 30-day deposit return / itemized statement deadline under § 5321.16(B); § 5321.16(C) entitles tenant to recover amount wrongfully withheld PLUS damages equal to amount wrongfully withheld (DOUBLE DAMAGES) AND reasonable attorney fees.".to_string(),
                    citations,
                    double_damages_remedy_dollars: double_damages,
                }
            }
        }
        ComplianceAspect::SecurityDepositDoubleDamagesUnderSection5321_16C => {
            if input.portion_of_deposit_wrongfully_withheld_dollars > 0 {
                Output {
                    mode: OhioRc5321Mode::ViolationSecurityDepositWrongfulRetentionDoubleDamagesPlusAttorneyFees,
                    statutory_basis: "O.R.C. § 5321.16(C) — wrongful retention triggers double damages + attorney fees".to_string(),
                    notes: "VIOLATION: landlord wrongfully retained portion of security deposit; § 5321.16(C) remedy = amount wrongfully withheld PLUS damages equal to amount wrongfully withheld (DOUBLE DAMAGES) AND reasonable attorney fees.".to_string(),
                    citations,
                    double_damages_remedy_dollars: double_damages,
                }
            } else {
                Output {
                    mode: OhioRc5321Mode::CompliantSecurityDepositNoWrongfulRetention,
                    statutory_basis: "O.R.C. § 5321.16(C) — no wrongful retention".to_string(),
                    notes: "COMPLIANT: no wrongful retention of security deposit; § 5321.16(C) double-damages remedy does not attach.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            }
        }
        ComplianceAspect::LandlordEntryTwentyFourHourNoticeUnderSection5321_04A8 => {
            if input.landlord_entry_notice_hours_given >= OHIO_RC_5321_LANDLORD_ENTRY_NOTICE_HOURS {
                Output {
                    mode: OhioRc5321Mode::CompliantLandlordEntryTwentyFourHourNoticeProvided,
                    statutory_basis: "O.R.C. § 5321.04(A)(8) — landlord entry with at least 24-hour notice".to_string(),
                    notes: "COMPLIANT: landlord provided at least 24-hour notice before entering premises under § 5321.04(A)(8); entry at reasonable times.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            } else {
                Output {
                    mode: OhioRc5321Mode::ViolationLandlordEntryWithoutTwentyFourHourNotice,
                    statutory_basis: "O.R.C. § 5321.04(A)(8) — landlord entry without 24-hour notice".to_string(),
                    notes: "VIOLATION: landlord entered without 24-hour notice under § 5321.04(A)(8); tenant may seek damages or injunctive relief.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            }
        }
        ComplianceAspect::RentEscrowRemedyUnderSection5321_07 => Output {
            mode: OhioRc5321Mode::CompliantRentEscrowAvailableUnderSection5321_07,
            statutory_basis: "O.R.C. § 5321.07 — rent escrow remedy available to tenant for landlord non-compliance".to_string(),
            notes: "INFORMATIONAL: § 5321.07 tenant remedies for landlord non-compliance with § 5321.04 obligations = (1) deposit rent in escrow with municipal or county court clerk; (2) apply to court for reduction of rent; (3) terminate rental agreement.".to_string(),
            citations,
            double_damages_remedy_dollars: 0,
        },
        ComplianceAspect::RetaliationProhibitedUnderSection5321_13 => {
            if input.retaliatory_conduct_occurred {
                Output {
                    mode: OhioRc5321Mode::ViolationRetaliatoryConduct,
                    statutory_basis: "O.R.C. § 5321.13 — retaliatory conduct prohibited".to_string(),
                    notes: "VIOLATION: landlord engaged in retaliatory conduct against tenant for complaint to government agency / complaint to landlord / tenant organization / Chapter 5321 rights assertion.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            } else {
                Output {
                    mode: OhioRc5321Mode::CompliantNoRetaliation,
                    statutory_basis: "O.R.C. § 5321.13 — no retaliatory conduct".to_string(),
                    notes: "COMPLIANT: no retaliatory conduct under § 5321.13.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            }
        }
        ComplianceAspect::TerminationNoticeBasedOnTenancyTermUnderSection5321_17 => {
            let required_days = match input.tenancy_term_length {
                TenancyTermLength::MonthToMonth => OHIO_RC_5321_MONTH_TO_MONTH_TERMINATION_NOTICE_DAYS,
                TenancyTermLength::WeekToWeek => OHIO_RC_5321_WEEK_TO_WEEK_TERMINATION_NOTICE_DAYS,
                TenancyTermLength::FixedTermUnderNinetyDays | TenancyTermLength::FixedTermNinetyDaysOrMore => {
                    OHIO_RC_5321_MONTH_TO_MONTH_TERMINATION_NOTICE_DAYS
                }
            };
            if input.termination_notice_days_given >= required_days {
                Output {
                    mode: OhioRc5321Mode::CompliantTerminationNoticeMeetsStatutoryLengthForTenancyTerm,
                    statutory_basis: "O.R.C. § 5321.17 — termination notice meets statutory length".to_string(),
                    notes: "COMPLIANT: termination notice meets § 5321.17 statutory length for tenancy term (30 days for month-to-month or fixed term; 7 days for week-to-week).".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            } else {
                Output {
                    mode: OhioRc5321Mode::ViolationTerminationNoticeShorterThanStatutoryLengthForTenancyTerm,
                    statutory_basis: "O.R.C. § 5321.17 — termination notice shorter than statutory length".to_string(),
                    notes: "VIOLATION: termination notice shorter than § 5321.17 statutory length for tenancy term; termination invalid.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            }
        }
        ComplianceAspect::ThreeDayPayOrQuitNoticeUnderSection1923_04 => {
            if input.pay_or_quit_notice_days_given >= OHIO_RC_1923_PAY_OR_QUIT_NOTICE_DAYS {
                Output {
                    mode: OhioRc5321Mode::CompliantThreeDayPayOrQuitNoticeProvided,
                    statutory_basis: "O.R.C. § 1923.04 + § 5321.17(C) — 3-day pay or quit notice provided".to_string(),
                    notes: "COMPLIANT: landlord provided 3-day notice to leave premises before commencing R.C. Chapter 1923 forcible entry and detainer action under § 5321.17(C) and § 1923.04.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            } else {
                Output {
                    mode: OhioRc5321Mode::ViolationPayOrQuitNoticeShorterThanThreeDays,
                    statutory_basis: "O.R.C. § 1923.04 + § 5321.17(C) — pay or quit notice shorter than 3-day statutory minimum".to_string(),
                    notes: "VIOLATION: pay or quit notice shorter than 3-day statutory minimum under § 1923.04 and § 5321.17(C); forcible entry and detainer action subject to dismissal.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            }
        }
        ComplianceAspect::WrittenRentalAgreementRequiredForOverNinetyDaysUnderSection5321_18 => {
            if input.rental_agreement_term_days > OHIO_RC_5321_LEASE_WRITING_REQUIRED_THRESHOLD_DAYS
                && !input.rental_agreement_in_writing
            {
                Output {
                    mode: OhioRc5321Mode::ViolationOralRentalAgreementOverNinetyDaysUnenforceable,
                    statutory_basis: "O.R.C. § 5321.18 + R.C. § 1335.05 Statute of Frauds — oral rental agreement for term exceeding 90 days unenforceable".to_string(),
                    notes: "VIOLATION: rental agreement term exceeds 90 days but is not in writing; § 5321.18 + R.C. § 1335.05 (Ohio Statute of Frauds) render the agreement unenforceable beyond 90-day term.".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            } else {
                Output {
                    mode: OhioRc5321Mode::CompliantWrittenRentalAgreementForOverNinetyDays,
                    statutory_basis: "O.R.C. § 5321.18 — written rental agreement requirement satisfied".to_string(),
                    notes: "COMPLIANT: written rental agreement for term exceeding 90 days OR rental agreement term at or below 90 days (no writing required).".to_string(),
                    citations,
                    double_damages_remedy_dollars: 0,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline_input() -> Input {
        Input {
            tenancy_type: TenancyType::ResidentialRentalCoveredByChapter5321,
            compliance_aspect: ComplianceAspect::SecurityDepositReturnAndItemizedDeductionsUnderSection5321_16B,
            tenancy_term_length: TenancyTermLength::MonthToMonth,
            monthly_rent_dollars: 1_200,
            security_deposit_dollars: 1_200,
            portion_of_deposit_wrongfully_withheld_dollars: 0,
            months_tenant_in_possession: 12,
            interest_paid_on_excess_deposit: true,
            deposit_returned_and_itemized_within_window: true,
            days_since_lease_termination_for_deposit_return: 25,
            landlord_obligations_met: true,
            landlord_entry_notice_hours_given: 24,
            retaliatory_conduct_occurred: false,
            termination_notice_days_given: 30,
            pay_or_quit_notice_days_given: 3,
            rental_agreement_in_writing: true,
            rental_agreement_term_days: 365,
        }
    }

    #[test]
    fn exempt_tenancy_not_applicable() {
        let mut input = baseline_input();
        input.tenancy_type = TenancyType::CommercialRentalExempt;
        let output = check(&input);
        assert_eq!(output.mode, OhioRc5321Mode::NotApplicableTenancyExemptFromChapter5321);
    }

    #[test]
    fn deposit_return_within_thirty_days_compliant() {
        let output = check(&baseline_input());
        assert_eq!(
            output.mode,
            OhioRc5321Mode::CompliantSecurityDepositReturnedAndItemizedDeductionsWithinThirtyDays
        );
    }

    #[test]
    fn deposit_return_at_exactly_thirty_day_boundary_compliant() {
        let mut input = baseline_input();
        input.days_since_lease_termination_for_deposit_return = 30;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OhioRc5321Mode::CompliantSecurityDepositReturnedAndItemizedDeductionsWithinThirtyDays
        );
    }

    #[test]
    fn deposit_return_past_thirty_days_violation_double_damages() {
        let mut input = baseline_input();
        input.days_since_lease_termination_for_deposit_return = 35;
        input.deposit_returned_and_itemized_within_window = false;
        input.portion_of_deposit_wrongfully_withheld_dollars = 500;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OhioRc5321Mode::ViolationSecurityDepositReturnedPastThirtyDayDeadlineDoubleDamages
        );
        assert_eq!(output.double_damages_remedy_dollars, 500 * 2);
    }

    #[test]
    fn security_deposit_interest_not_required_under_threshold() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositInterestUnderSection5321_16A;
        input.security_deposit_dollars = 1_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OhioRc5321Mode::CompliantSecurityDepositInterestPaidOrNotRequired
        );
    }

    #[test]
    fn security_deposit_interest_not_required_under_six_months_holding() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositInterestUnderSection5321_16A;
        input.security_deposit_dollars = 5_000;
        input.months_tenant_in_possession = 5;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OhioRc5321Mode::CompliantSecurityDepositInterestPaidOrNotRequired
        );
    }

    #[test]
    fn security_deposit_interest_paid_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositInterestUnderSection5321_16A;
        input.security_deposit_dollars = 5_000;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OhioRc5321Mode::CompliantSecurityDepositInterestPaidOrNotRequired
        );
    }

    #[test]
    fn security_deposit_interest_not_paid_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositInterestUnderSection5321_16A;
        input.security_deposit_dollars = 5_000;
        input.interest_paid_on_excess_deposit = false;
        let output = check(&input);
        assert_eq!(output.mode, OhioRc5321Mode::ViolationSecurityDepositInterestNotPaid);
    }

    #[test]
    fn double_damages_wrongful_retention_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositDoubleDamagesUnderSection5321_16C;
        input.portion_of_deposit_wrongfully_withheld_dollars = 800;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OhioRc5321Mode::ViolationSecurityDepositWrongfulRetentionDoubleDamagesPlusAttorneyFees
        );
        assert_eq!(output.double_damages_remedy_dollars, 800 * 2);
    }

    #[test]
    fn landlord_obligations_met_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordObligationsUnderSection5321_04;
        let output = check(&input);
        assert_eq!(output.mode, OhioRc5321Mode::CompliantLandlordObligationsMet);
    }

    #[test]
    fn landlord_obligations_breached_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordObligationsUnderSection5321_04;
        input.landlord_obligations_met = false;
        let output = check(&input);
        assert_eq!(output.mode, OhioRc5321Mode::ViolationLandlordObligationsBreached);
    }

    #[test]
    fn landlord_entry_twenty_four_hours_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordEntryTwentyFourHourNoticeUnderSection5321_04A8;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OhioRc5321Mode::CompliantLandlordEntryTwentyFourHourNoticeProvided
        );
    }

    #[test]
    fn landlord_entry_under_twenty_four_hours_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::LandlordEntryTwentyFourHourNoticeUnderSection5321_04A8;
        input.landlord_entry_notice_hours_given = 12;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OhioRc5321Mode::ViolationLandlordEntryWithoutTwentyFourHourNotice
        );
    }

    #[test]
    fn rent_escrow_remedy_available_under_section_5321_07() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RentEscrowRemedyUnderSection5321_07;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OhioRc5321Mode::CompliantRentEscrowAvailableUnderSection5321_07
        );
    }

    #[test]
    fn no_retaliation_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RetaliationProhibitedUnderSection5321_13;
        let output = check(&input);
        assert_eq!(output.mode, OhioRc5321Mode::CompliantNoRetaliation);
    }

    #[test]
    fn retaliatory_conduct_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::RetaliationProhibitedUnderSection5321_13;
        input.retaliatory_conduct_occurred = true;
        let output = check(&input);
        assert_eq!(output.mode, OhioRc5321Mode::ViolationRetaliatoryConduct);
    }

    #[test]
    fn termination_notice_month_to_month_thirty_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TerminationNoticeBasedOnTenancyTermUnderSection5321_17;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OhioRc5321Mode::CompliantTerminationNoticeMeetsStatutoryLengthForTenancyTerm
        );
    }

    #[test]
    fn termination_notice_week_to_week_seven_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TerminationNoticeBasedOnTenancyTermUnderSection5321_17;
        input.tenancy_term_length = TenancyTermLength::WeekToWeek;
        input.termination_notice_days_given = 7;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OhioRc5321Mode::CompliantTerminationNoticeMeetsStatutoryLengthForTenancyTerm
        );
    }

    #[test]
    fn termination_notice_month_to_month_twenty_nine_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::TerminationNoticeBasedOnTenancyTermUnderSection5321_17;
        input.termination_notice_days_given = 29;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OhioRc5321Mode::ViolationTerminationNoticeShorterThanStatutoryLengthForTenancyTerm
        );
    }

    #[test]
    fn three_day_pay_or_quit_notice_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ThreeDayPayOrQuitNoticeUnderSection1923_04;
        let output = check(&input);
        assert_eq!(output.mode, OhioRc5321Mode::CompliantThreeDayPayOrQuitNoticeProvided);
    }

    #[test]
    fn pay_or_quit_under_three_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::ThreeDayPayOrQuitNoticeUnderSection1923_04;
        input.pay_or_quit_notice_days_given = 2;
        let output = check(&input);
        assert_eq!(output.mode, OhioRc5321Mode::ViolationPayOrQuitNoticeShorterThanThreeDays);
    }

    #[test]
    fn written_rental_agreement_required_over_ninety_days_compliant() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::WrittenRentalAgreementRequiredForOverNinetyDaysUnderSection5321_18;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OhioRc5321Mode::CompliantWrittenRentalAgreementForOverNinetyDays
        );
    }

    #[test]
    fn oral_rental_agreement_over_ninety_days_violation() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::WrittenRentalAgreementRequiredForOverNinetyDaysUnderSection5321_18;
        input.rental_agreement_in_writing = false;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OhioRc5321Mode::ViolationOralRentalAgreementOverNinetyDaysUnenforceable
        );
    }

    #[test]
    fn constants_pin_statutory_facts() {
        assert_eq!(OHIO_RC_5321_CHAPTER_NUMBER, 5321);
        assert_eq!(OHIO_RC_5321_ENACTMENT_YEAR, 1974);
        assert_eq!(OHIO_RC_5321_ENACTMENT_MONTH, 11);
        assert_eq!(OHIO_RC_5321_ENACTMENT_DAY, 4);
        assert_eq!(OHIO_RC_5321_ENABLING_AM_HB_NUMBER, 144);
        assert_eq!(OHIO_RC_5321_SECURITY_DEPOSIT_INTEREST_THRESHOLD_DOLLARS, 50);
        assert_eq!(OHIO_RC_5321_SECURITY_DEPOSIT_INTEREST_THRESHOLD_MONTHS_OF_RENT, 1);
        assert_eq!(OHIO_RC_5321_SECURITY_DEPOSIT_INTEREST_RATE_BPS, 500);
        assert_eq!(OHIO_RC_5321_SECURITY_DEPOSIT_INTEREST_MIN_HOLDING_MONTHS, 6);
        assert_eq!(OHIO_RC_5321_SECURITY_DEPOSIT_RETURN_DEADLINE_DAYS, 30);
        assert_eq!(OHIO_RC_5321_SECURITY_DEPOSIT_DOUBLE_DAMAGES_MULTIPLIER, 2);
        assert_eq!(OHIO_RC_5321_LANDLORD_ENTRY_NOTICE_HOURS, 24);
        assert_eq!(OHIO_RC_5321_MONTH_TO_MONTH_TERMINATION_NOTICE_DAYS, 30);
        assert_eq!(OHIO_RC_5321_WEEK_TO_WEEK_TERMINATION_NOTICE_DAYS, 7);
        assert_eq!(OHIO_RC_5321_LEASE_WRITING_REQUIRED_THRESHOLD_DAYS, 90);
        assert_eq!(OHIO_RC_1923_PAY_OR_QUIT_NOTICE_DAYS, 3);
        assert_eq!(OHIO_RC_5321_TREBLE_DAMAGES_MULTIPLIER, 3);
        assert_eq!(OHIO_RC_5321_BASIS_POINT_DENOMINATOR, 10_000);
    }

    #[test]
    fn citations_contain_landmarks() {
        let output = check(&baseline_input());
        let joined = output.citations.join("\n");
        assert!(joined.contains("Ohio Landlord-Tenant Act"));
        assert!(joined.contains("1974 Am. H.B. 144"));
        assert!(joined.contains("November 4, 1974"));
        assert!(joined.contains("Ohio Revised Code §§ 5321.01 through 5321.21"));
        assert!(joined.contains("§ 5321.04"));
        assert!(joined.contains("§ 5321.05"));
        assert!(joined.contains("§ 5321.07"));
        assert!(joined.contains("§ 5321.09"));
        assert!(joined.contains("§ 5321.12"));
        assert!(joined.contains("§ 5321.13"));
        assert!(joined.contains("§ 5321.16(A)"));
        assert!(joined.contains("§ 5321.16(B)"));
        assert!(joined.contains("§ 5321.16(C)"));
        assert!(joined.contains("§ 5321.17"));
        assert!(joined.contains("§ 5321.18"));
        assert!(joined.contains("§ 1923.02"));
        assert!(joined.contains("§ 1923.04"));
        assert!(joined.contains("24 HOURS"));
        assert!(joined.contains("5 PERCENT"));
        assert!(joined.contains("6 MONTHS"));
        assert!(joined.contains("30 DAYS"));
        assert!(joined.contains("DOUBLE DAMAGES"));
        assert!(joined.contains("30-DAY"));
        assert!(joined.contains("7-DAY"));
        assert!(joined.contains("3-DAY"));
        assert!(joined.contains("90 days"));
    }

    #[test]
    fn double_damages_saturating_overflow_defense() {
        let mut input = baseline_input();
        input.compliance_aspect = ComplianceAspect::SecurityDepositDoubleDamagesUnderSection5321_16C;
        input.portion_of_deposit_wrongfully_withheld_dollars = u64::MAX;
        let output = check(&input);
        assert_eq!(
            output.mode,
            OhioRc5321Mode::ViolationSecurityDepositWrongfulRetentionDoubleDamagesPlusAttorneyFees
        );
        assert_eq!(output.double_damages_remedy_dollars, u64::MAX);
    }
}
