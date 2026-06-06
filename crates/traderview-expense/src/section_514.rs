//! IRC § 514 — Unrelated Debt-Financed Income.
//!
//! § 514 expands the Unrelated Business Taxable Income (UBTI) framework of §
//! 511 to capture investment income that would otherwise be excluded under §
//! 512(b)(1)-(5) when the underlying property is debt-financed. The provision
//! exists to prevent tax-exempt organizations (pension funds, university
//! endowments, foundations, churches, IRAs) from using their tax-exempt status
//! to effectively guarantee leveraged investments without paying tax on the
//! debt-financed portion.
//!
//! § 514(a) general rule: a tax-exempt organization includes in UBTI a
//! percentage of the gross income from any debt-financed property, computed
//! as the "debt/basis percentage" — average acquisition indebtedness divided
//! by average adjusted basis. Deductions allocable to debt-financed property
//! are similarly multiplied by the debt/basis percentage.
//!
//! § 514(b)(1) "debt-financed property" means any property held to produce
//! income (including gain from disposition) for which there is "acquisition
//! indebtedness." Exclusions under § 514(b)(1)(A)-(E): (A) property
//! substantially all use of which is related to the organization's exempt
//! function; (B) property the income from which is already includible in UBTI
//! under § 512(a)(3) (recreational/social club); (C) property the income from
//! which is described in § 512(b)(7)-(9) (research income exception); (D)
//! property used in § 513 unrelated trade or business; (E) qualified §
//! 514(c)(9) real property (Section 514(c)(9) "fractions rule" exception for
//! qualified organizations holding qualified real property).
//!
//! § 514(c)(1) "acquisition indebtedness" means the unpaid amount of (A)
//! indebtedness incurred in acquiring or improving the property; (B) pre-
//! acquisition indebtedness that would not have been incurred but for the
//! acquisition; (C) post-acquisition indebtedness that would not have been
//! incurred but for the acquisition and was reasonably foreseeable at time
//! of acquisition.
//!
//! § 514(c)(9) "qualified real property" exception: real property acquired
//! by qualified organization (educational institution, qualified pension
//! plan trust, qualified title-holding company under § 501(c)(25), retirement
//! income account under § 403(b)(9)) is NOT debt-financed property if (A)
//! the price is fixed at closing, (B) no debt held by seller or related
//! person, (C) certain "fractions rule" requirements satisfied under §
//! 514(c)(9)(E). This exception is critical for university endowments and
//! private pension funds investing in leveraged real estate partnerships.
//!
//! Partnership flow-through: § 514 applies at the partnership level — a
//! tax-exempt partner's share of partnership debt-financed income is UBTI
//! to the extent of the partnership's debt/basis percentage on the
//! underlying assets. § 512(c) treats the partner's share of partnership
//! UBTI as the partner's own UBTI. Tax-exempt investors in hedge funds and
//! securities-trading partnerships that borrow funds must report § 514 UBTI
//! based on their share of the debt-financed securities income.
//!
//! Coordination: § 514 UBTI is taxed at corporate rates per § 511(a) on
//! § 501(c) organizations and at trust rates per § 511(b) on § 401(a)
//! qualified trusts (IRAs, 401(k)s, pension trusts).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExemptEntityType {
    /// § 501(c)(3) university, foundation, hospital, church.
    Section501c3PublicCharity,
    /// § 501(c)(3) private foundation — subject to § 4940 NII excise also.
    Section501c3PrivateFoundation,
    /// § 401(a) qualified pension / profit-sharing trust.
    Section401aQualifiedTrust,
    /// Traditional or Roth IRA / SEP-IRA / SIMPLE IRA.
    IndividualRetirementAccount,
    /// § 501(c)(25) qualified title-holding company.
    Section501c25TitleHoldingCompany,
    /// Not a tax-exempt entity — § 514 inapplicable.
    NotTaxExempt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    /// Real estate (rental property, hotel, office).
    RealEstate,
    /// Securities portfolio (stocks, bonds, derivatives) held with margin
    /// debt.
    SecuritiesWithMargin,
    /// Partnership interest in leveraged investment partnership.
    PartnershipInterestLeveraged,
    /// Property substantially used for exempt function (excluded).
    SubstantiallyExemptFunctionUse,
    /// No debt-financed property.
    NoDebtFinancedProperty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    NotTaxExemptNoUbti,
    NoDebtFinancedPropertyNoUbti,
    ExemptFunctionUseExcluded,
    QualifiedRealPropertyExceptionApplies,
    DebtFinancedUbtiSubjectToCorporateRate,
    DebtFinancedUbtiSubjectToTrustRate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section514Input {
    pub exempt_entity_type: ExemptEntityType,
    pub property_type: PropertyType,
    /// Whether qualified-organization exception under § 514(c)(9) applies
    /// (educational institution, qualified pension trust, etc.) — only
    /// relevant for real property.
    pub qualified_organization_for_514c9_exception: bool,
    /// Whether fractions rule satisfied per § 514(c)(9)(E).
    pub fractions_rule_satisfied: bool,
    /// Average acquisition indebtedness during the taxable year in cents.
    pub average_acquisition_indebtedness_cents: u64,
    /// Average adjusted basis of the property during the taxable year in
    /// cents.
    pub average_adjusted_basis_cents: u64,
    /// Gross income from the debt-financed property in cents.
    pub gross_income_from_property_cents: u64,
    /// Deductions allocable to the property in cents.
    pub deductions_allocable_to_property_cents: u64,
    /// Corporate income tax rate in basis points (21% = 2_100).
    pub corporate_tax_rate_bps: u32,
    /// Trust income tax rate in basis points (37% top marginal = 3_700).
    pub trust_tax_rate_bps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section514Result {
    pub severity: Severity,
    pub debt_basis_percentage_bps: u32,
    pub gross_ubti_cents: u64,
    pub allocable_deductions_cents: u64,
    pub net_ubti_cents: u64,
    pub applicable_tax_rate_bps: u32,
    pub tax_owed_cents: u64,
    pub recommended_actions: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const SECTION_514_INTRODUCED_YEAR: i32 = 1969;
pub const TAX_REFORM_ACT_1969_PUB_L: &str = "Pub. L. 91-172";

pub fn check(input: &Section514Input) -> Section514Result {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    if matches!(input.exempt_entity_type, ExemptEntityType::NotTaxExempt) {
        notes.push(
            "Entity is NOT a § 501(a) tax-exempt organization; § 514 UBTI framework \
             inapplicable. Investment income taxed under general individual / corporate / \
             partnership rules — debt-financed leverage is fully deductible as investment \
             interest under § 163(d) for individuals or general business interest under § \
             163(j) for corporations."
                .to_string(),
        );
        return empty_result(
            Severity::NotTaxExemptNoUbti,
            input,
            actions,
            notes,
            "26 U.S.C. § 501(a); § 514 (n/a)",
        );
    }

    if matches!(input.property_type, PropertyType::NoDebtFinancedProperty)
        || input.average_acquisition_indebtedness_cents == 0
    {
        notes.push(
            "No debt-financed property OR no acquisition indebtedness — no § 514 UBTI \
             arises. Investment income remains excluded from UBTI under § 512(b)(1)-(5) \
             investment-income exclusions (dividends, interest, royalties, rents, capital \
             gain)."
                .to_string(),
        );
        return empty_result(
            Severity::NoDebtFinancedPropertyNoUbti,
            input,
            actions,
            notes,
            "26 U.S.C. § 512(b)(1)-(5); § 514(c)(1)",
        );
    }

    if matches!(
        input.property_type,
        PropertyType::SubstantiallyExemptFunctionUse
    ) {
        notes.push(
            "Property substantially used for exempt function — excluded from § 514(b)(1)(A) \
             debt-financed property definition. University-owned dormitory or hospital \
             facility purchased with mortgage debt is NOT debt-financed property under § \
             514 because of the substantially-related-use exception, even though acquisition \
             indebtedness exists."
                .to_string(),
        );
        return empty_result(
            Severity::ExemptFunctionUseExcluded,
            input,
            actions,
            notes,
            "26 U.S.C. § 514(b)(1)(A)",
        );
    }

    let is_qualified_organization = input.qualified_organization_for_514c9_exception
        && matches!(input.property_type, PropertyType::RealEstate)
        && input.fractions_rule_satisfied
        && matches!(
            input.exempt_entity_type,
            ExemptEntityType::Section501c3PublicCharity
                | ExemptEntityType::Section401aQualifiedTrust
                | ExemptEntityType::Section501c25TitleHoldingCompany
        );

    if is_qualified_organization {
        notes.push(
            "§ 514(c)(9) qualified real property exception applies — qualified organization \
             (educational institution, qualified pension trust, § 501(c)(25) title-holding \
             company, or § 403(b)(9) retirement income account) holding qualified real \
             property satisfies (A) fixed price at closing, (B) no debt held by seller or \
             related person, and (C) § 514(c)(9)(E) fractions rule. Real property treated \
             as NOT debt-financed despite acquisition indebtedness; investment income \
             excluded from UBTI per § 514(c)(9)(A)."
                .to_string(),
        );
        return empty_result(
            Severity::QualifiedRealPropertyExceptionApplies,
            input,
            actions,
            notes,
            "26 U.S.C. § 514(c)(9)(A)-(E)",
        );
    }

    let debt_basis_bps: u32 = if input.average_adjusted_basis_cents == 0 {
        0
    } else {
        let pct: u128 = u128::from(input.average_acquisition_indebtedness_cents) * 10_000
            / u128::from(input.average_adjusted_basis_cents);
        pct.min(10_000) as u32
    };

    let gross_ubti: u64 = (u128::from(input.gross_income_from_property_cents)
        * u128::from(debt_basis_bps)
        / 10_000) as u64;
    let allocable_deductions: u64 = (u128::from(input.deductions_allocable_to_property_cents)
        * u128::from(debt_basis_bps)
        / 10_000) as u64;
    let net_ubti = gross_ubti.saturating_sub(allocable_deductions);

    let is_trust_taxed = matches!(
        input.exempt_entity_type,
        ExemptEntityType::Section401aQualifiedTrust | ExemptEntityType::IndividualRetirementAccount
    );

    let applicable_rate = if is_trust_taxed {
        input.trust_tax_rate_bps
    } else {
        input.corporate_tax_rate_bps
    };

    let tax_owed: u64 = (u128::from(net_ubti) * u128::from(applicable_rate) / 10_000) as u64;

    let severity = if is_trust_taxed {
        Severity::DebtFinancedUbtiSubjectToTrustRate
    } else {
        Severity::DebtFinancedUbtiSubjectToCorporateRate
    };

    actions.push(format!(
        "Compute § 514 UBTI: debt/basis percentage = avg acquisition indebtedness ({} cents) \
         / avg adjusted basis ({} cents) = {} bps. Gross UBTI = gross income {} × {} bps = \
         {} cents. Allocable deductions = deductions {} × {} bps = {} cents. Net UBTI = {} \
         cents. Apply {} rate {} bps = {} cents tax owed. File Form 990-T (Exempt \
         Organization Business Income Tax Return) under § 6012(a)(2). Coordinate with \
         [[section_512]] (UBTI computation) and [[section_511]] (tax imposition).",
        input.average_acquisition_indebtedness_cents,
        input.average_adjusted_basis_cents,
        debt_basis_bps,
        input.gross_income_from_property_cents,
        debt_basis_bps,
        gross_ubti,
        input.deductions_allocable_to_property_cents,
        debt_basis_bps,
        allocable_deductions,
        net_ubti,
        if is_trust_taxed { "trust" } else { "corporate" },
        applicable_rate,
        tax_owed
    ));

    if matches!(
        input.exempt_entity_type,
        ExemptEntityType::IndividualRetirementAccount
    ) {
        actions.push(
            "IRA UBTI above $1,000 annual exemption per § 512(b)(12) requires Form 990-T \
             filed by IRA custodian; tax is paid from IRA assets reducing retirement \
             savings. Common trigger: IRA invested in leveraged real estate partnership or \
             MLP with operating-trade income. Consider unleveraged real estate or non-MLP \
             securities to avoid § 514 UBTI."
                .to_string(),
        );
    }

    notes.push(
        "Coordination with [[section_511]] (tax imposition on UBTI at corporate or trust \
         rates), [[section_512]] (UBTI computation framework including § 512(b)(1)-(5) \
         investment income exclusions and § 512(c) partnership look-through), [[section_\
         513]] (unrelated trade or business definition), [[section_4940]] (PF NII excise \
         — separate regime for private foundation investment income), [[section_408]] (IRA \
         rules), [[section_401a9]] (RMD coordination), [[section_4944]] (PF jeopardy \
         investment regime — different leverage analysis)."
            .to_string(),
    );

    Section514Result {
        severity,
        debt_basis_percentage_bps: debt_basis_bps,
        gross_ubti_cents: gross_ubti,
        allocable_deductions_cents: allocable_deductions,
        net_ubti_cents: net_ubti,
        applicable_tax_rate_bps: applicable_rate,
        tax_owed_cents: tax_owed,
        recommended_actions: actions,
        citation: "26 U.S.C. § 514(a)-(c); § 511; § 512; Treas. Reg. § 1.514(b)-1; § 1.514(c)-1",
        notes,
    }
}

fn empty_result(
    severity: Severity,
    input: &Section514Input,
    recommended_actions: Vec<String>,
    mut notes: Vec<String>,
    citation: &'static str,
) -> Section514Result {
    notes.push(
        "Coordination with [[section_511]] (UBTI tax), [[section_512]] (UBTI computation), \
         [[section_513]] (unrelated trade or business), [[section_4940]] (PF NII excise), \
         [[section_408]] (IRA rules)."
            .to_string(),
    );
    let _ = input;
    Section514Result {
        severity,
        debt_basis_percentage_bps: 0,
        gross_ubti_cents: 0,
        allocable_deductions_cents: 0,
        net_ubti_cents: 0,
        applicable_tax_rate_bps: 0,
        tax_owed_cents: 0,
        recommended_actions,
        citation,
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Section514Input {
        Section514Input {
            exempt_entity_type: ExemptEntityType::Section401aQualifiedTrust,
            property_type: PropertyType::SecuritiesWithMargin,
            qualified_organization_for_514c9_exception: false,
            fractions_rule_satisfied: false,
            average_acquisition_indebtedness_cents: 50_000_000_00,
            average_adjusted_basis_cents: 100_000_000_00,
            gross_income_from_property_cents: 10_000_000_00,
            deductions_allocable_to_property_cents: 1_000_000_00,
            corporate_tax_rate_bps: 2_100,
            trust_tax_rate_bps: 3_700,
        }
    }

    #[test]
    fn not_tax_exempt_no_ubti() {
        let mut i = baseline();
        i.exempt_entity_type = ExemptEntityType::NotTaxExempt;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NotTaxExemptNoUbti));
        assert_eq!(r.tax_owed_cents, 0);
        assert!(r.notes.iter().any(|n| n.contains("§ 163(d)")));
    }

    #[test]
    fn no_debt_no_ubti() {
        let mut i = baseline();
        i.average_acquisition_indebtedness_cents = 0;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NoDebtFinancedPropertyNoUbti));
        assert!(r.notes.iter().any(|n| n.contains("§ 512(b)(1)-(5)")));
    }

    #[test]
    fn no_debt_financed_property_no_ubti() {
        let mut i = baseline();
        i.property_type = PropertyType::NoDebtFinancedProperty;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::NoDebtFinancedPropertyNoUbti));
    }

    #[test]
    fn substantially_exempt_function_use_excluded() {
        let mut i = baseline();
        i.property_type = PropertyType::SubstantiallyExemptFunctionUse;
        let r = check(&i);
        assert!(matches!(r.severity, Severity::ExemptFunctionUseExcluded));
        assert!(r.notes.iter().any(|n| n.contains("§ 514(b)(1)(A)")));
    }

    #[test]
    fn qualified_real_property_exception_applies() {
        let mut i = baseline();
        i.exempt_entity_type = ExemptEntityType::Section401aQualifiedTrust;
        i.property_type = PropertyType::RealEstate;
        i.qualified_organization_for_514c9_exception = true;
        i.fractions_rule_satisfied = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::QualifiedRealPropertyExceptionApplies
        ));
        assert!(r.notes.iter().any(|n| n.contains("§ 514(c)(9)")));
        assert!(r.notes.iter().any(|n| n.contains("fractions rule")));
    }

    #[test]
    fn qualified_real_property_without_fractions_rule_still_ubti() {
        let mut i = baseline();
        i.exempt_entity_type = ExemptEntityType::Section401aQualifiedTrust;
        i.property_type = PropertyType::RealEstate;
        i.qualified_organization_for_514c9_exception = true;
        i.fractions_rule_satisfied = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::DebtFinancedUbtiSubjectToTrustRate
        ));
    }

    #[test]
    fn ira_with_leveraged_securities_subject_to_trust_rate() {
        let mut i = baseline();
        i.exempt_entity_type = ExemptEntityType::IndividualRetirementAccount;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::DebtFinancedUbtiSubjectToTrustRate
        ));
        assert_eq!(r.applicable_tax_rate_bps, i.trust_tax_rate_bps);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("IRA UBTI above $1,000")));
    }

    #[test]
    fn pension_trust_with_leveraged_securities_subject_to_trust_rate() {
        let mut i = baseline();
        i.exempt_entity_type = ExemptEntityType::Section401aQualifiedTrust;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::DebtFinancedUbtiSubjectToTrustRate
        ));
        assert_eq!(r.applicable_tax_rate_bps, i.trust_tax_rate_bps);
    }

    #[test]
    fn public_charity_subject_to_corporate_rate() {
        let mut i = baseline();
        i.exempt_entity_type = ExemptEntityType::Section501c3PublicCharity;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::DebtFinancedUbtiSubjectToCorporateRate
        ));
        assert_eq!(r.applicable_tax_rate_bps, i.corporate_tax_rate_bps);
    }

    #[test]
    fn private_foundation_subject_to_corporate_rate() {
        let mut i = baseline();
        i.exempt_entity_type = ExemptEntityType::Section501c3PrivateFoundation;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::DebtFinancedUbtiSubjectToCorporateRate
        ));
    }

    #[test]
    fn debt_basis_percentage_50_pct_pinned() {
        let i = baseline();
        let r = check(&i);
        assert_eq!(r.debt_basis_percentage_bps, 5_000);
    }

    #[test]
    fn debt_basis_percentage_capped_at_100_pct() {
        let mut i = baseline();
        i.average_acquisition_indebtedness_cents = 200_000_000_00;
        i.average_adjusted_basis_cents = 100_000_000_00;
        let r = check(&i);
        assert_eq!(r.debt_basis_percentage_bps, 10_000);
    }

    #[test]
    fn debt_basis_percentage_zero_when_basis_zero() {
        let mut i = baseline();
        i.average_adjusted_basis_cents = 0;
        let r = check(&i);
        assert_eq!(r.debt_basis_percentage_bps, 0);
        assert_eq!(r.net_ubti_cents, 0);
    }

    #[test]
    fn gross_ubti_correctly_computed_at_50_pct() {
        let i = baseline();
        let r = check(&i);
        let expected_gross = i.gross_income_from_property_cents * 5_000 / 10_000;
        assert_eq!(r.gross_ubti_cents, expected_gross);
    }

    #[test]
    fn allocable_deductions_correctly_computed_at_50_pct() {
        let i = baseline();
        let r = check(&i);
        let expected_deductions = i.deductions_allocable_to_property_cents * 5_000 / 10_000;
        assert_eq!(r.allocable_deductions_cents, expected_deductions);
    }

    #[test]
    fn net_ubti_equals_gross_minus_deductions() {
        let i = baseline();
        let r = check(&i);
        assert_eq!(
            r.net_ubti_cents,
            r.gross_ubti_cents
                .saturating_sub(r.allocable_deductions_cents)
        );
    }

    #[test]
    fn tax_owed_correctly_computed_at_trust_rate() {
        let i = baseline();
        let r = check(&i);
        let expected = r.net_ubti_cents * u64::from(i.trust_tax_rate_bps) / 10_000;
        assert_eq!(r.tax_owed_cents, expected);
    }

    #[test]
    fn tax_owed_correctly_computed_at_corporate_rate() {
        let mut i = baseline();
        i.exempt_entity_type = ExemptEntityType::Section501c3PublicCharity;
        let r = check(&i);
        let expected = r.net_ubti_cents * u64::from(i.corporate_tax_rate_bps) / 10_000;
        assert_eq!(r.tax_owed_cents, expected);
    }

    #[test]
    fn action_references_form_990_t() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("Form 990-T")));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("§ 6012(a)(2)")));
    }

    #[test]
    fn coordination_note_references_511_512_513_4940() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("section_511")));
        assert!(r.notes.iter().any(|n| n.contains("section_512")));
        assert!(r.notes.iter().any(|n| n.contains("section_513")));
        assert!(r.notes.iter().any(|n| n.contains("section_4940")));
    }

    #[test]
    fn citation_pins_514_511_512_treas_reg() {
        let i = baseline();
        let r = check(&i);
        assert!(r.citation.contains("§ 514(a)-(c)"));
        assert!(r.citation.contains("§ 511"));
        assert!(r.citation.contains("§ 512"));
        assert!(r.citation.contains("Treas. Reg. § 1.514(b)-1"));
        assert!(r.citation.contains("§ 1.514(c)-1"));
    }

    #[test]
    fn section_514_introduced_year_pins_1969() {
        assert_eq!(SECTION_514_INTRODUCED_YEAR, 1969);
    }

    #[test]
    fn tax_reform_act_1969_pub_l_pins_91_172() {
        assert_eq!(TAX_REFORM_ACT_1969_PUB_L, "Pub. L. 91-172");
    }

    #[test]
    fn realistic_university_endowment_leveraged_real_estate_qualified_no_ubti() {
        let mut i = baseline();
        i.exempt_entity_type = ExemptEntityType::Section501c3PublicCharity;
        i.property_type = PropertyType::RealEstate;
        i.qualified_organization_for_514c9_exception = true;
        i.fractions_rule_satisfied = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::QualifiedRealPropertyExceptionApplies
        ));
        assert_eq!(r.tax_owed_cents, 0);
    }

    #[test]
    fn realistic_pension_fund_leveraged_partnership_ubti_pinned() {
        let mut i = baseline();
        i.exempt_entity_type = ExemptEntityType::Section401aQualifiedTrust;
        i.property_type = PropertyType::PartnershipInterestLeveraged;
        i.average_acquisition_indebtedness_cents = 100_000_000_00;
        i.average_adjusted_basis_cents = 200_000_000_00;
        i.gross_income_from_property_cents = 30_000_000_00;
        let r = check(&i);
        assert_eq!(r.debt_basis_percentage_bps, 5_000);
        let expected_gross = 30_000_000_00u64 * 5_000 / 10_000;
        assert_eq!(r.gross_ubti_cents, expected_gross);
    }

    #[test]
    fn extreme_value_does_not_overflow() {
        let mut i = baseline();
        i.average_acquisition_indebtedness_cents = u64::MAX / 100;
        i.average_adjusted_basis_cents = u64::MAX / 100;
        i.gross_income_from_property_cents = u64::MAX / 100;
        let r = check(&i);
        let _ = r.tax_owed_cents;
    }

    #[test]
    fn zero_income_zero_ubti() {
        let mut i = baseline();
        i.gross_income_from_property_cents = 0;
        i.deductions_allocable_to_property_cents = 0;
        let r = check(&i);
        assert_eq!(r.net_ubti_cents, 0);
        assert_eq!(r.tax_owed_cents, 0);
    }

    #[test]
    fn deductions_exceed_gross_saturating_no_negative_ubti() {
        let mut i = baseline();
        i.gross_income_from_property_cents = 1_000_000_00;
        i.deductions_allocable_to_property_cents = 5_000_000_00;
        let r = check(&i);
        assert_eq!(r.net_ubti_cents, 0);
        assert_eq!(r.tax_owed_cents, 0);
    }

    #[test]
    fn title_holding_company_qualified_real_property_exception() {
        let mut i = baseline();
        i.exempt_entity_type = ExemptEntityType::Section501c25TitleHoldingCompany;
        i.property_type = PropertyType::RealEstate;
        i.qualified_organization_for_514c9_exception = true;
        i.fractions_rule_satisfied = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::QualifiedRealPropertyExceptionApplies
        ));
    }

    #[test]
    fn ira_not_eligible_for_514_c9_exception() {
        let mut i = baseline();
        i.exempt_entity_type = ExemptEntityType::IndividualRetirementAccount;
        i.property_type = PropertyType::RealEstate;
        i.qualified_organization_for_514c9_exception = true;
        i.fractions_rule_satisfied = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            Severity::DebtFinancedUbtiSubjectToTrustRate
        ));
    }
}
