//! IRC § 4945 — Taxes on taxable expenditures of private
//! foundations. PF must spend only on charitable purposes
//! within § 170(c)(2)(B); spending outside the narrow
//! permitted scope triggers four-tier excise tax. Direct
//! PF Chapter 42 companion to section_4940 (PF NII excise
//! — iter 470), section_4941 (PF self-dealing — iter 468),
//! section_4942 (PF minimum distribution — iter 472),
//! section_4943 (PF excess business holdings — iter 474),
//! section_4944 (PF jeopardizing investments — iter 476),
//! section_4958 (intermediate sanctions for public
//! charities — iter 466), section_4960 (ATEO executive
//! comp 21% — iter 464). Originally enacted by Tax Reform
//! Act of 1969, Pub. L. 91-172.
//!
//! Four-tier excise tax structure:
//! - § 4945(a)(1) TIER 1 PF: 20% of amount of taxable
//!   expenditure
//! - § 4945(a)(2) TIER 1 MANAGER (knowingly agreed to
//!   taxable expenditure): 5% of amount, capped at $10,000
//!   per expenditure per § 4945(c)(2)
//! - § 4945(b)(1) TIER 2 PF (not corrected within taxable
//!   period): 100% of amount of taxable expenditure
//! - § 4945(b)(2) TIER 2 MANAGER (refused to agree to
//!   correction): 50% of amount, capped at $20,000 per
//!   expenditure per § 4945(c)(2)
//!
//! Five categories of taxable expenditures per § 4945(d):
//! 1. § 4945(d)(1) — INFLUENCING LEGISLATION (lobbying):
//!    any amount paid or incurred to carry on propaganda
//!    or otherwise attempt to influence legislation;
//!    EXCEPTIONS under § 4945(e): (A) nonpartisan
//!    analysis, study, or research; (B) technical advice
//!    in response to written request from governmental
//!    body; (C) self-defense exception (legislation that
//!    would directly affect PF existence/powers/duties/
//!    tax status); (D) examination of broad social,
//!    economic, and similar problems
//! 2. § 4945(d)(2) — INFLUENCING ELECTIONS or CARRYING ON
//!    VOTER REGISTRATION DRIVES; § 4945(f) SAFE HARBOR
//!    for voter registration if FIVE conditions met:
//!    (i) PF is § 501(c)(3) and § 509(a)(1)/(2)/(3) or
//!    operating foundation; (ii) substantially all of
//!    income spent for activities described in (i);
//!    (iii) PF receives 85%+ of support from sources
//!    other than DPs and non-(c)(3)s; (iv) activities are
//!    nonpartisan and carried on in 5+ states; (v)
//!    activities are non-earmarked for any candidate
//! 3. § 4945(d)(3) — GRANTS TO INDIVIDUALS for travel,
//!    study, or other similar purposes; § 4945(g) ADVANCE
//!    IRS APPROVAL required: (i) procedure for awarding
//!    grants approved in advance by IRS; (ii) grant is
//!    scholarship/fellowship/grant for objective and
//!    nondiscriminatory selection; (iii) grant achieves
//!    specific charitable purpose
//! 4. § 4945(d)(4) — GRANTS TO ORGANIZATIONS that are
//!    NOT public charities under § 509(a)(1)/(2)/(3) OR
//!    operating foundations under § 4942(j)(3), UNLESS PF
//!    exercises § 4945(h) EXPENDITURE RESPONSIBILITY
//!    (four-prong)
//! 5. § 4945(d)(5) — NON-CHARITABLE EXPENDITURES: any
//!    amount paid or incurred for any purpose OTHER than
//!    one specified in § 170(c)(2)(B) religious/
//!    charitable/scientific/literary/educational/public
//!    purposes
//!
//! § 4945(h) EXPENDITURE RESPONSIBILITY four-prong:
//! 1. PRE-GRANT INQUIRY — reasonable investigation into
//!    grantee's ability to use funds for charitable
//!    purposes
//! 2. WRITTEN GRANT AGREEMENT — specifies charitable
//!    purpose + prohibits use for § 4945(d)(1)/(d)(2)
//!    lobbying or political activity
//! 3. REPORTS FROM GRANTEE — annual reports detailing
//!    use of funds + progress toward charitable goals
//! 4. REPORTS TO IRS — full and detailed reports
//!    regarding grants reported on Form 990-PF
//!
//! Trader-foundation critical because (1) § 4945(d)(1)
//! LOBBYING ban is among the most surprising compliance
//! traps — even social advocacy by family-foundation
//! board members or grant recipients can trigger; (2)
//! § 4945(d)(3) advance IRS approval for SCHOLARSHIP/
//! FELLOWSHIP PROGRAMS routinely takes 6-12 months —
//! must be in place before disbursement; (3) § 4945(d)(4)
//! GRANTS TO LLCs OR SOCIAL ENTERPRISES require
//! expenditure responsibility four-prong — failure to
//! document any prong triggers 20% + 100% tax; (4) Tier-2
//! 100% tax effectively confiscates the entire grant
//! amount and is non-deductible; (5) manager $10K + $20K
//! caps make individual board-member exposure manageable
//! but joint and several with PF tax.
//!
//! Distinction from § 4944 (iter 476): § 4944 evaluates
//! prudence of INVESTMENTS (asset side); § 4945 evaluates
//! propriety of EXPENDITURES (program side).
//!
//! Distinction from § 4941 (iter 468): § 4941 punishes
//! self-dealing TRANSACTIONS between PF and DP; § 4945
//! punishes expenditures outside permitted CHARITABLE
//! purposes regardless of recipient relationship.
//!
//! Authority: 26 U.S.C. § 4945; § 4945(a)(1); § 4945(a)(2);
//! § 4945(b)(1); § 4945(b)(2); § 4945(c)(1); § 4945(c)(2);
//! § 4945(d)(1); § 4945(d)(2); § 4945(d)(3); § 4945(d)(4);
//! § 4945(d)(5); § 4945(e); § 4945(f); § 4945(g); § 4945(h);
//! § 170(c)(2)(B); § 509(a)(1); § 509(a)(2); § 509(a)(3);
//! § 4942(j)(3); 26 C.F.R. § 53.4945-1 through § 53.4945-6;
//! Tax Reform Act of 1969, Pub. L. 91-172 (Dec. 30, 1969)
//! — original § 4945 enactment.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FoundationStatus {
    PrivateFoundation,
    PublicCharity,
    NonExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExpenditureCategory {
    CharitableProgramPayment,
    LobbyingLegislation,
    PoliticalCampaignOrVoterRegistration,
    GrantToIndividual,
    GrantToNonPublicCharityOrganization,
    NonCharitablePurpose,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Input {
    pub foundation_status: FoundationStatus,
    pub category: ExpenditureCategory,
    pub expenditure_amount_cents: u64,
    pub lobbying_section_4945e_exception_applies: bool,
    pub voter_registration_section_4945f_safe_harbor_satisfied: bool,
    pub grant_to_individual_section_4945g_advance_irs_approval: bool,
    pub grant_to_organization_section_4945h_expenditure_responsibility_satisfied: bool,
    pub corrected_within_taxable_period: bool,
    pub manager_knowingly_agreed: bool,
    pub manager_refused_correction: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    Compliant,
    Tier1TaxOwed,
    Tier2TaxOwed,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Output {
    pub severity: Severity,
    pub tier1_pf_tax_cents: u64,
    pub tier1_manager_tax_cents: u64,
    pub tier2_pf_tax_cents: u64,
    pub tier2_manager_tax_cents: u64,
    pub total_tax_cents: u64,
    pub notes: Vec<String>,
}

pub const TIER1_PF_RATE_PCT: u64 = 20;
pub const TIER1_MANAGER_RATE_PCT: u64 = 5;
pub const TIER2_PF_RATE_PCT: u64 = 100;
pub const TIER2_MANAGER_RATE_PCT: u64 = 50;
pub const TIER1_MANAGER_CAP_CENTS: u64 = 1_000_000;
pub const TIER2_MANAGER_CAP_CENTS: u64 = 2_000_000;

pub type Section4945Input = Input;
pub type Section4945Result = Output;

fn is_taxable_expenditure(input: &Input) -> bool {
    match input.category {
        ExpenditureCategory::CharitableProgramPayment => false,
        ExpenditureCategory::LobbyingLegislation => !input.lobbying_section_4945e_exception_applies,
        ExpenditureCategory::PoliticalCampaignOrVoterRegistration => {
            !input.voter_registration_section_4945f_safe_harbor_satisfied
        }
        ExpenditureCategory::GrantToIndividual => {
            !input.grant_to_individual_section_4945g_advance_irs_approval
        }
        ExpenditureCategory::GrantToNonPublicCharityOrganization => {
            !input.grant_to_organization_section_4945h_expenditure_responsibility_satisfied
        }
        ExpenditureCategory::NonCharitablePurpose => true,
    }
}

pub fn check(input: &Input) -> Output {
    let notes: Vec<String> = vec![
        "§ 4945(a)(1) TIER-1 PF 20% excise on amount of taxable expenditure; § 4945(a)(2) TIER-1 MANAGER 5% (knowingly agreed), capped at $10,000 per expenditure per § 4945(c)(2); § 4945(b)(1) TIER-2 PF 100% if not corrected within taxable period; § 4945(b)(2) TIER-2 MANAGER 50% (refused correction), capped at $20,000 per expenditure.".to_string(),
        "Five categories per § 4945(d): (1) § 4945(d)(1) INFLUENCING LEGISLATION (lobbying) with § 4945(e) exceptions for nonpartisan analysis + technical advice + self-defense + employee communications; (2) § 4945(d)(2) INFLUENCING ELECTIONS or VOTER REGISTRATION with § 4945(f) five-condition safe harbor; (3) § 4945(d)(3) GRANTS TO INDIVIDUALS without § 4945(g) advance IRS approval; (4) § 4945(d)(4) GRANTS TO ORGANIZATIONS not § 509(a)(1)/(2)/(3) or § 4942(j)(3) operating foundation without § 4945(h) expenditure responsibility; (5) § 4945(d)(5) NON-CHARITABLE EXPENDITURES outside § 170(c)(2)(B).".to_string(),
        "§ 4945(e) LOBBYING EXCEPTIONS — (A) nonpartisan analysis, study, or research; (B) technical advice in response to written request from governmental body; (C) self-defense (legislation directly affecting PF existence/powers/duties/tax status); (D) examination of broad social/economic problems.".to_string(),
        "§ 4945(f) VOTER REGISTRATION SAFE HARBOR — five conditions: (i) PF is § 501(c)(3) and § 509(a)(1)/(2)/(3) or operating foundation; (ii) substantially all income spent for charitable activities; (iii) 85%+ support from sources other than DPs and non-(c)(3)s; (iv) nonpartisan activities in 5+ states; (v) non-earmarked for any candidate.".to_string(),
        "§ 4945(h) EXPENDITURE RESPONSIBILITY four-prong for grants to non-public-charity organizations: (1) PRE-GRANT INQUIRY into grantee; (2) WRITTEN GRANT AGREEMENT specifying charitable purpose + prohibiting use for § 4945(d)(1)/(d)(2) lobbying or political activity; (3) REPORTS FROM GRANTEE detailing use of funds; (4) REPORTS TO IRS on Form 990-PF.".to_string(),
        "Distinction from § 4944 (iter 476): § 4944 evaluates prudence of INVESTMENTS (asset side); § 4945 evaluates propriety of EXPENDITURES (program side). Distinction from § 4941 (iter 468): § 4941 punishes self-dealing TRANSACTIONS between PF and DP; § 4945 punishes expenditures outside permitted charitable purposes regardless of recipient relationship.".to_string(),
        "Companion: section_4940 (iter 470), section_4941 (iter 468), section_4942 (iter 472), section_4943 (iter 474), section_4944 (iter 476), section_4958 (iter 466), section_4960 (iter 464).".to_string(),
    ];

    if !matches!(input.foundation_status, FoundationStatus::PrivateFoundation) {
        let mut n = notes;
        n.push("Organization is not a private foundation — § 4945 does not apply.".to_string());
        return Output {
            severity: Severity::NotApplicable,
            tier1_pf_tax_cents: 0,
            tier1_manager_tax_cents: 0,
            tier2_pf_tax_cents: 0,
            tier2_manager_tax_cents: 0,
            total_tax_cents: 0,
            notes: n,
        };
    }

    if !is_taxable_expenditure(input) {
        let mut n = notes;
        n.push("Expenditure is not a taxable expenditure under § 4945(d) — applicable exception, safe harbor, or charitable-purpose qualification satisfied.".to_string());
        return Output {
            severity: Severity::Compliant,
            tier1_pf_tax_cents: 0,
            tier1_manager_tax_cents: 0,
            tier2_pf_tax_cents: 0,
            tier2_manager_tax_cents: 0,
            total_tax_cents: 0,
            notes: n,
        };
    }

    let amount = input.expenditure_amount_cents;

    let tier1_pf = amount
        .saturating_mul(TIER1_PF_RATE_PCT)
        .checked_div(100)
        .unwrap_or(0);

    let tier1_manager = if input.manager_knowingly_agreed {
        let uncapped = amount
            .saturating_mul(TIER1_MANAGER_RATE_PCT)
            .checked_div(100)
            .unwrap_or(0);
        uncapped.min(TIER1_MANAGER_CAP_CENTS)
    } else {
        0
    };

    let corrected = input.corrected_within_taxable_period;

    let tier2_pf = if corrected {
        0
    } else {
        amount
            .saturating_mul(TIER2_PF_RATE_PCT)
            .checked_div(100)
            .unwrap_or(0)
    };

    let tier2_manager = if !corrected && input.manager_refused_correction {
        let uncapped = amount
            .saturating_mul(TIER2_MANAGER_RATE_PCT)
            .checked_div(100)
            .unwrap_or(0);
        uncapped.min(TIER2_MANAGER_CAP_CENTS)
    } else {
        0
    };

    let total = tier1_pf
        .saturating_add(tier1_manager)
        .saturating_add(tier2_pf)
        .saturating_add(tier2_manager);

    let severity = if tier2_pf > 0 || tier2_manager > 0 {
        Severity::Tier2TaxOwed
    } else {
        Severity::Tier1TaxOwed
    };

    let mut n = notes;
    n.push(format!(
        "Taxable expenditure § 4945 tax: Tier-1 PF 20% ${}.{:02} + Tier-1 manager 5% ${}.{:02} + Tier-2 PF 100% ${}.{:02} + Tier-2 manager 50% ${}.{:02} = Total ${}.{:02}.",
        tier1_pf / 100,
        tier1_pf % 100,
        tier1_manager / 100,
        tier1_manager % 100,
        tier2_pf / 100,
        tier2_pf % 100,
        tier2_manager / 100,
        tier2_manager % 100,
        total / 100,
        total % 100
    ));

    Output {
        severity,
        tier1_pf_tax_cents: tier1_pf,
        tier1_manager_tax_cents: tier1_manager,
        tier2_pf_tax_cents: tier2_pf,
        tier2_manager_tax_cents: tier2_manager,
        total_tax_cents: total,
        notes: n,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Input {
        Input {
            foundation_status: FoundationStatus::PrivateFoundation,
            category: ExpenditureCategory::LobbyingLegislation,
            expenditure_amount_cents: 100_000_00, // $100K
            lobbying_section_4945e_exception_applies: false,
            voter_registration_section_4945f_safe_harbor_satisfied: false,
            grant_to_individual_section_4945g_advance_irs_approval: false,
            grant_to_organization_section_4945h_expenditure_responsibility_satisfied: false,
            corrected_within_taxable_period: false,
            manager_knowingly_agreed: false,
            manager_refused_correction: false,
        }
    }

    #[test]
    fn public_charity_not_subject_to_4945() {
        let mut i = baseline();
        i.foundation_status = FoundationStatus::PublicCharity;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn non_exempt_not_applicable() {
        let mut i = baseline();
        i.foundation_status = FoundationStatus::NonExempt;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
    }

    #[test]
    fn charitable_program_payment_compliant() {
        let mut i = baseline();
        i.category = ExpenditureCategory::CharitableProgramPayment;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn lobbying_no_exception_tier_2() {
        let i = baseline(); // lobbying, no exception, uncorrected
        let out = check(&i);
        // Tier-1 PF 20% × $100K = $20K
        // Tier-2 PF 100% × $100K = $100K
        // Total: $120K
        assert_eq!(out.tier1_pf_tax_cents, 20_000_00);
        assert_eq!(out.tier2_pf_tax_cents, 100_000_00);
        assert_eq!(out.total_tax_cents, 120_000_00);
        assert_eq!(out.severity, Severity::Tier2TaxOwed);
    }

    #[test]
    fn lobbying_with_4945e_exception_compliant() {
        let mut i = baseline();
        i.lobbying_section_4945e_exception_applies = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn political_campaign_no_safe_harbor_tier_2() {
        let mut i = baseline();
        i.category = ExpenditureCategory::PoliticalCampaignOrVoterRegistration;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Tier2TaxOwed);
    }

    #[test]
    fn voter_registration_with_4945f_safe_harbor_compliant() {
        let mut i = baseline();
        i.category = ExpenditureCategory::PoliticalCampaignOrVoterRegistration;
        i.voter_registration_section_4945f_safe_harbor_satisfied = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn grant_to_individual_no_advance_irs_approval_tier_2() {
        let mut i = baseline();
        i.category = ExpenditureCategory::GrantToIndividual;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Tier2TaxOwed);
    }

    #[test]
    fn grant_to_individual_with_4945g_advance_irs_approval_compliant() {
        let mut i = baseline();
        i.category = ExpenditureCategory::GrantToIndividual;
        i.grant_to_individual_section_4945g_advance_irs_approval = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn grant_to_org_no_expenditure_responsibility_tier_2() {
        let mut i = baseline();
        i.category = ExpenditureCategory::GrantToNonPublicCharityOrganization;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Tier2TaxOwed);
    }

    #[test]
    fn grant_to_org_with_4945h_expenditure_responsibility_compliant() {
        let mut i = baseline();
        i.category = ExpenditureCategory::GrantToNonPublicCharityOrganization;
        i.grant_to_organization_section_4945h_expenditure_responsibility_satisfied = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Compliant);
    }

    #[test]
    fn non_charitable_expenditure_no_exception_available() {
        let mut i = baseline();
        i.category = ExpenditureCategory::NonCharitablePurpose;
        // No exception available for non-charitable purpose
        let out = check(&i);
        assert_eq!(out.severity, Severity::Tier2TaxOwed);
    }

    #[test]
    fn tier_1_only_when_corrected_within_taxable_period() {
        let mut i = baseline();
        i.corrected_within_taxable_period = true;
        let out = check(&i);
        assert_eq!(out.severity, Severity::Tier1TaxOwed);
        assert_eq!(out.tier1_pf_tax_cents, 20_000_00);
        assert_eq!(out.tier2_pf_tax_cents, 0);
    }

    #[test]
    fn manager_knowingly_agreed_5_pct_capped_at_10k() {
        let mut i = baseline();
        i.manager_knowingly_agreed = true;
        i.expenditure_amount_cents = 1_000_000_00; // $1M
                                                   // 5% × $1M = $50K, capped at $10K
        let out = check(&i);
        assert_eq!(out.tier1_manager_tax_cents, 10_000_00);
    }

    #[test]
    fn manager_knowingly_agreed_under_cap() {
        let mut i = baseline();
        i.manager_knowingly_agreed = true;
        i.expenditure_amount_cents = 100_000_00; // $100K
                                                 // 5% × $100K = $5K (under cap)
        let out = check(&i);
        assert_eq!(out.tier1_manager_tax_cents, 5_000_00);
    }

    #[test]
    fn manager_not_knowing_no_tier_1_manager_tax() {
        let i = baseline();
        let out = check(&i);
        assert_eq!(out.tier1_manager_tax_cents, 0);
    }

    #[test]
    fn manager_refused_correction_50_pct_capped_at_20k() {
        let mut i = baseline();
        i.manager_refused_correction = true;
        i.expenditure_amount_cents = 1_000_000_00;
        // 50% × $1M = $500K, capped at $20K
        let out = check(&i);
        assert_eq!(out.tier2_manager_tax_cents, 20_000_00);
    }

    #[test]
    fn manager_refused_correction_under_cap() {
        let mut i = baseline();
        i.manager_refused_correction = true;
        i.expenditure_amount_cents = 30_000_00;
        // 50% × $30K = $15K (under cap)
        let out = check(&i);
        assert_eq!(out.tier2_manager_tax_cents, 15_000_00);
    }

    #[test]
    fn manager_refused_but_corrected_no_tier_2_manager() {
        let mut i = baseline();
        i.corrected_within_taxable_period = true;
        i.manager_refused_correction = true; // moot when corrected
        let out = check(&i);
        assert_eq!(out.tier2_manager_tax_cents, 0);
    }

    #[test]
    fn all_four_tiers_stack() {
        let mut i = baseline();
        i.manager_knowingly_agreed = true;
        i.manager_refused_correction = true;
        i.expenditure_amount_cents = 100_000_00;
        let out = check(&i);
        // Tier-1 PF 20% × $100K = $20K
        // Tier-1 manager 5% × $100K = $5K
        // Tier-2 PF 100% × $100K = $100K
        // Tier-2 manager 50% × $100K = $50K capped at $20K
        // Total: $20K + $5K + $100K + $20K = $145K
        assert_eq!(out.tier1_pf_tax_cents, 20_000_00);
        assert_eq!(out.tier1_manager_tax_cents, 5_000_00);
        assert_eq!(out.tier2_pf_tax_cents, 100_000_00);
        assert_eq!(out.tier2_manager_tax_cents, 20_000_00);
        assert_eq!(out.total_tax_cents, 145_000_00);
    }

    #[test]
    fn citation_pins_all_authorities() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4945(a)(1)"));
        assert!(joined.contains("§ 4945(a)(2)"));
        assert!(joined.contains("§ 4945(b)(1)"));
        assert!(joined.contains("§ 4945(b)(2)"));
        assert!(joined.contains("§ 4945(c)(2)"));
        assert!(joined.contains("§ 4945(d)(1)"));
        assert!(joined.contains("§ 4945(d)(2)"));
        assert!(joined.contains("§ 4945(d)(3)"));
        assert!(joined.contains("§ 4945(d)(4)"));
        assert!(joined.contains("§ 4945(d)(5)"));
        assert!(joined.contains("§ 4945(e)"));
        assert!(joined.contains("§ 4945(f)"));
        assert!(joined.contains("§ 4945(g)"));
        assert!(joined.contains("§ 4945(h)"));
        assert!(joined.contains("§ 170(c)(2)(B)"));
        assert!(joined.contains("§ 509(a)(1)"));
        assert!(joined.contains("§ 4942(j)(3)"));
        assert!(joined.contains("§ 4944 (iter 476)"));
        assert!(joined.contains("§ 4941 (iter 468)"));
    }

    #[test]
    fn note_pins_four_tier_structure() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("20%"));
        assert!(joined.contains("100%"));
        assert!(joined.contains("5%"));
        assert!(joined.contains("50%"));
        assert!(joined.contains("$10,000"));
        assert!(joined.contains("$20,000"));
    }

    #[test]
    fn note_pins_five_categories() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("INFLUENCING LEGISLATION"));
        assert!(joined.contains("INFLUENCING ELECTIONS"));
        assert!(joined.contains("VOTER REGISTRATION"));
        assert!(joined.contains("GRANTS TO INDIVIDUALS"));
        assert!(joined.contains("GRANTS TO ORGANIZATIONS"));
        assert!(joined.contains("NON-CHARITABLE EXPENDITURES"));
    }

    #[test]
    fn note_pins_4945e_lobbying_exceptions() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("nonpartisan analysis"));
        assert!(joined.contains("technical advice"));
        assert!(joined.contains("self-defense"));
        assert!(joined.contains("broad social/economic problems"));
    }

    #[test]
    fn note_pins_4945f_voter_registration_safe_harbor() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("85%+ support"));
        assert!(joined.contains("nonpartisan activities"));
        assert!(joined.contains("5+ states"));
        assert!(joined.contains("non-earmarked"));
    }

    #[test]
    fn note_pins_4945h_expenditure_responsibility_four_prong() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("PRE-GRANT INQUIRY"));
        assert!(joined.contains("WRITTEN GRANT AGREEMENT"));
        assert!(joined.contains("REPORTS FROM GRANTEE"));
        assert!(joined.contains("REPORTS TO IRS"));
        assert!(joined.contains("Form 990-PF"));
    }

    #[test]
    fn note_pins_4944_4941_distinctions() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("§ 4944 (iter 476)"));
        assert!(joined.contains("INVESTMENTS"));
        assert!(joined.contains("EXPENDITURES"));
        assert!(joined.contains("§ 4941 (iter 468)"));
        assert!(joined.contains("self-dealing"));
    }

    #[test]
    fn note_pins_companion_modules() {
        let i = baseline();
        let out = check(&i);
        let joined = out.notes.join(" ");
        assert!(joined.contains("section_4940"));
        assert!(joined.contains("section_4941"));
        assert!(joined.contains("section_4942"));
        assert!(joined.contains("section_4943"));
        assert!(joined.contains("section_4944"));
        assert!(joined.contains("section_4958"));
        assert!(joined.contains("section_4960"));
    }

    #[test]
    fn truth_table_eight_cells() {
        // Public charity → NotApplicable
        let c1 = check(&Input {
            foundation_status: FoundationStatus::PublicCharity,
            ..baseline()
        });
        assert_eq!(c1.severity, Severity::NotApplicable);

        // Charitable program payment → Compliant
        let c2 = check(&Input {
            category: ExpenditureCategory::CharitableProgramPayment,
            ..baseline()
        });
        assert_eq!(c2.severity, Severity::Compliant);

        // Lobbying with exception → Compliant
        let c3 = check(&Input {
            lobbying_section_4945e_exception_applies: true,
            ..baseline()
        });
        assert_eq!(c3.severity, Severity::Compliant);

        // Voter reg with safe harbor → Compliant
        let c4 = check(&Input {
            category: ExpenditureCategory::PoliticalCampaignOrVoterRegistration,
            voter_registration_section_4945f_safe_harbor_satisfied: true,
            ..baseline()
        });
        assert_eq!(c4.severity, Severity::Compliant);

        // Individual grant with advance approval → Compliant
        let c5 = check(&Input {
            category: ExpenditureCategory::GrantToIndividual,
            grant_to_individual_section_4945g_advance_irs_approval: true,
            ..baseline()
        });
        assert_eq!(c5.severity, Severity::Compliant);

        // Org grant with expenditure responsibility → Compliant
        let c6 = check(&Input {
            category: ExpenditureCategory::GrantToNonPublicCharityOrganization,
            grant_to_organization_section_4945h_expenditure_responsibility_satisfied: true,
            ..baseline()
        });
        assert_eq!(c6.severity, Severity::Compliant);

        // Corrected within taxable period → Tier1TaxOwed
        let c7 = check(&Input {
            corrected_within_taxable_period: true,
            ..baseline()
        });
        assert_eq!(c7.severity, Severity::Tier1TaxOwed);

        // Uncorrected lobbying → Tier2TaxOwed
        let c8 = check(&baseline());
        assert_eq!(c8.severity, Severity::Tier2TaxOwed);
    }

    #[test]
    fn saturating_arithmetic_overflow_defense() {
        let i = Input {
            expenditure_amount_cents: u64::MAX,
            manager_knowingly_agreed: true,
            manager_refused_correction: true,
            ..baseline()
        };
        let out = check(&i);
        // No panic; manager caps bind
        assert_eq!(out.tier1_manager_tax_cents, TIER1_MANAGER_CAP_CENTS);
        assert_eq!(out.tier2_manager_tax_cents, TIER2_MANAGER_CAP_CENTS);
    }

    #[test]
    fn boundary_one_cent_expenditure() {
        let mut i = baseline();
        i.expenditure_amount_cents = 1;
        let out = check(&i);
        // 20% of 1 cent = 0; 100% of 1 cent = 1
        assert_eq!(out.tier1_pf_tax_cents, 0);
        assert_eq!(out.tier2_pf_tax_cents, 1);
    }

    #[test]
    fn boundary_zero_expenditure_no_tax() {
        let mut i = baseline();
        i.expenditure_amount_cents = 0;
        let out = check(&i);
        assert_eq!(out.tier1_pf_tax_cents, 0);
        assert_eq!(out.tier2_pf_tax_cents, 0);
    }

    #[test]
    fn realistic_50k_unauthorized_lobbying() {
        // PF makes $50K political donation, manager knowingly agreed, not corrected
        let i = Input {
            foundation_status: FoundationStatus::PrivateFoundation,
            category: ExpenditureCategory::LobbyingLegislation,
            expenditure_amount_cents: 50_000_00,
            lobbying_section_4945e_exception_applies: false,
            voter_registration_section_4945f_safe_harbor_satisfied: false,
            grant_to_individual_section_4945g_advance_irs_approval: false,
            grant_to_organization_section_4945h_expenditure_responsibility_satisfied: false,
            corrected_within_taxable_period: false,
            manager_knowingly_agreed: true,
            manager_refused_correction: false,
        };
        let out = check(&i);
        // Tier-1 PF 20% × $50K = $10K
        // Tier-1 manager 5% × $50K = $2.5K
        // Tier-2 PF 100% × $50K = $50K
        // Tier-2 manager 50% × $50K = $25K capped at $20K = $0 (no refusal)
        // Total = $10K + $2.5K + $50K + $0 = $62.5K
        assert_eq!(out.tier1_pf_tax_cents, 10_000_00);
        assert_eq!(out.tier1_manager_tax_cents, 2_500_00);
        assert_eq!(out.tier2_pf_tax_cents, 50_000_00);
        assert_eq!(out.tier2_manager_tax_cents, 0);
        assert_eq!(out.total_tax_cents, 62_500_00);
    }
}
