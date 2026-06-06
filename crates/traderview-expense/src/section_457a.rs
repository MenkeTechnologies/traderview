//! IRC § 457A — Nonqualified Deferred Compensation From Certain Tax
//! Indifferent Parties (enacted October 3, 2008 in EESA § 801; effective for
//! amounts attributable to services performed after 2008-12-31).
//!
//! § 457A(a) general rule: compensation deferred under a nonqualified deferred
//! compensation (NQDC) plan of a "nonqualified entity" is includible in gross
//! income of the service provider when there is no substantial risk of
//! forfeiture (SROF) of the rights to such compensation.
//!
//! § 457A(b) "nonqualified entity" means (1) any foreign corporation UNLESS
//! substantially all of its income is either (A) effectively connected with
//! the conduct of a trade or business in the United States, OR (B) subject to
//! a comprehensive foreign income tax; OR (2) any partnership UNLESS
//! substantially all of its income is allocated to persons other than (A)
//! foreign persons with respect to which such income is not subject to a
//! comprehensive foreign income tax, and (B) US tax-exempt organizations.
//!
//! § 457A(c)(1)(B) penalty if deferred amount not determinable when ceases to
//! be subject to SROF: NOT includible until determinable, then includes (i)
//! 20% additional tax on amount required to be included PLUS (ii) interest at
//! AFR + 1% from year of vesting to year of inclusion (underpayment-rate
//! style).
//!
//! § 457A(d)(2) SROF definition: rights to compensation are treated as subject
//! to SROF ONLY if conditioned on future performance of substantial services.
//! Performance metrics, profit hurdles, etc. do NOT qualify (this is stricter
//! than the § 409A SROF definition).
//!
//! § 457A(d)(4) coordination with § 409A: § 457A applies IN ADDITION to §
//! 409A; both can simultaneously trigger.
//!
//! § 457A(e) transition rule: amounts attributable to services performed
//! BEFORE January 1, 2009 must be includible in gross income no later than
//! the LATER of (1) the last taxable year beginning before 2017, OR (2) the
//! taxable year in which there is no SROF (Notice 2009-8 transition guidance).
//!
//! Stock right exemption (Notice 2009-8 + Rev. Rul. 2014-18): nonstatutory
//! stock options and stock-settled SARs that meet § 409A exempt requirements
//! (exercise price at or above FMV at grant, on stock of the service
//! recipient) are NOT subject to § 457A.
//!
//! Hedge-fund context: US fund managers operating master-feeder structures
//! with Cayman Islands master fund + Cayman/BVI offshore feeder receive
//! management plus incentive fees from the offshore feeder; § 457A bars
//! traditional carry-deferral structures where compensation is deferred at
//! the offshore-feeder level. Post-2008 industry shift: managers either
//! accept current taxation OR restructure through US partnership management
//! company that recognizes fees currently (and partnership-allocates to
//! taxable US partners) rather than offshore deferral.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    /// Foreign corp with substantially all income subject to comprehensive
    /// foreign income tax → NOT a nonqualified entity.
    ForeignCorpComprehensiveForeignTax,
    /// Foreign corp with substantially all income ECI to US trade/business
    /// → NOT a nonqualified entity.
    ForeignCorpEffectivelyConnectedUSTrade,
    /// Foreign corp without comprehensive foreign tax or ECI → IS a
    /// nonqualified entity (typical Cayman/BVI master-feeder structure).
    ForeignCorpTaxIndifferent,
    /// Partnership with substantially all income allocated to taxable US
    /// persons (other than tax-exempts) → NOT a nonqualified entity.
    PartnershipMostlyTaxableUSPartners,
    /// Partnership with substantially all income allocated to foreign
    /// non-taxed persons or US tax-exempts → IS a nonqualified entity.
    PartnershipTaxIndifferentAllocations,
    /// US domestic C corporation paying tax at full corporate rates →
    /// NOT a nonqualified entity.
    DomesticCCorporation,
    /// US S corporation pass-through taxed at shareholder level → NOT a
    /// nonqualified entity.
    DomesticSCorporation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeferralType {
    /// Cash compensation deferred (carry, management fee, bonus).
    CashCompensationDeferral,
    /// Nonstatutory stock option meeting § 409A exempt requirements
    /// (strike at or above FMV at grant, on stock of service recipient).
    Section409aExemptStockOption,
    /// Stock-settled SAR meeting § 409A exempt requirements.
    Section409aExemptStockSettledSar,
    /// Phantom equity or profit-participating interest not meeting § 409A
    /// exemption.
    NonexemptPhantomEquityOrProfit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeferralSeverity {
    NotApplicable,
    NotANonqualifiedEntity,
    StockRightExemptUnder409a,
    SrofPresentDeferralAllowed,
    NoSrofIncomeImmediatelyIncludible,
    AmountNotDeterminablePending20PctPlusInterest,
    Pre2009TransitionViolationMissed2017Window,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section457aInput {
    pub entity_type: EntityType,
    pub deferral_type: DeferralType,
    /// Whether substantial-risk-of-forfeiture present per § 457A(d)(2)
    /// (future-performance-of-substantial-services standard).
    pub substantial_risk_of_forfeiture_present: bool,
    /// Whether amount of deferred compensation is determinable when SROF
    /// lapses per § 457A(c)(1)(B).
    pub amount_determinable_when_srof_lapses: bool,
    /// Deferred compensation amount in cents (gross).
    pub deferred_amount_cents: u64,
    /// Services-performed year (calendar). Pre-2009 services trigger
    /// transition rule under § 457A(e).
    pub services_performed_year: i32,
    /// Whether pre-2009 deferrals were included in gross income by last tax
    /// year beginning before 2017 (transition compliance).
    pub pre_2009_deferral_included_by_2017: bool,
    /// Year SROF lapses (services completed and rights vest).
    pub year_srof_lapses: i32,
    /// Year deferred amount becomes determinable (≥ year_srof_lapses if
    /// amount_determinable_when_srof_lapses is false).
    pub year_amount_determinable: i32,
    /// Federal marginal ordinary income rate applied to includible
    /// compensation, basis points (e.g., 3700 = 37.00%).
    pub federal_marginal_rate_bps: u32,
    /// Applicable federal underpayment rate (AFR) in basis points for the
    /// year SROF lapses (used as AFR + 1% per § 457A(c)(1)(B)(ii)).
    pub afr_bps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section457aResult {
    pub severity: DeferralSeverity,
    pub is_nonqualified_entity: bool,
    pub income_inclusion_year: i32,
    pub income_inclusion_amount_cents: u64,
    pub federal_income_tax_cents: u64,
    pub section_457a_20pct_additional_tax_cents: u64,
    pub interest_cents: u64,
    pub total_tax_burden_cents: u64,
    pub recommended_actions: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub const SECTION_457A_ADDITIONAL_TAX_BPS: u32 = 2_000;
pub const SECTION_457A_AFR_PLUS_BPS: u32 = 100;
pub const SECTION_457A_TRANSITION_LAST_YEAR: i32 = 2017;
pub const SECTION_457A_EFFECTIVE_YEAR: i32 = 2009;

pub fn check(input: &Section457aInput) -> Section457aResult {
    let mut actions: Vec<String> = Vec::new();
    let mut notes: Vec<String> = Vec::new();

    let is_nonqualified_entity = matches!(
        input.entity_type,
        EntityType::ForeignCorpTaxIndifferent | EntityType::PartnershipTaxIndifferentAllocations
    );

    if !is_nonqualified_entity {
        notes.push(format!(
            "Entity type {:?} is NOT a nonqualified entity under § 457A(b); plan governed by \
             § 409A general NQDC rules only. § 457A 20% additional tax inapplicable.",
            input.entity_type
        ));
        actions.push(
            "Confirm § 409A documentation, election timing, and distribution-event rules \
             remain satisfied; § 457A does not separately apply."
                .to_string(),
        );
        return Section457aResult {
            severity: DeferralSeverity::NotANonqualifiedEntity,
            is_nonqualified_entity: false,
            income_inclusion_year: input.year_srof_lapses,
            income_inclusion_amount_cents: 0,
            federal_income_tax_cents: 0,
            section_457a_20pct_additional_tax_cents: 0,
            interest_cents: 0,
            total_tax_burden_cents: 0,
            recommended_actions: actions,
            citation: "26 U.S.C. § 457A(b); coord. § 409A",
            notes,
        };
    }

    if matches!(
        input.deferral_type,
        DeferralType::Section409aExemptStockOption | DeferralType::Section409aExemptStockSettledSar
    ) {
        notes.push(
            "Nonstatutory stock option or stock-settled SAR meeting § 409A exempt requirements \
             (strike ≥ FMV at grant, on service-recipient stock) is NOT subject to § 457A \
             per Notice 2009-8 Q&A 2 and Rev. Rul. 2014-18 (fund-manager stock-right safe \
             harbor)."
                .to_string(),
        );
        return Section457aResult {
            severity: DeferralSeverity::StockRightExemptUnder409a,
            is_nonqualified_entity: true,
            income_inclusion_year: input.year_srof_lapses,
            income_inclusion_amount_cents: 0,
            federal_income_tax_cents: 0,
            section_457a_20pct_additional_tax_cents: 0,
            interest_cents: 0,
            total_tax_burden_cents: 0,
            recommended_actions: actions,
            citation: "26 U.S.C. § 457A; Notice 2009-8 Q&A 2; Rev. Rul. 2014-18",
            notes,
        };
    }

    let pre_2009_service = input.services_performed_year < SECTION_457A_EFFECTIVE_YEAR;
    if pre_2009_service {
        if input.pre_2009_deferral_included_by_2017 {
            notes.push(
                "Pre-2009 deferred amounts properly included in gross income by last tax year \
                 beginning before 2017 per § 457A(e) transition rule; no current-year § \
                 457A inclusion or 20% additional tax."
                    .to_string(),
            );
            return Section457aResult {
                severity: DeferralSeverity::NotApplicable,
                is_nonqualified_entity: true,
                income_inclusion_year: SECTION_457A_TRANSITION_LAST_YEAR - 1,
                income_inclusion_amount_cents: 0,
                federal_income_tax_cents: 0,
                section_457a_20pct_additional_tax_cents: 0,
                interest_cents: 0,
                total_tax_burden_cents: 0,
                recommended_actions: actions,
                citation: "26 U.S.C. § 457A(e); Notice 2009-8 transition guidance",
                notes,
            };
        }
        actions.push(
            "Pre-2009 deferral NOT included in income by last tax year beginning before 2017; \
             file amended returns for affected open tax years to include amounts plus \
             interest. § 457A(e) transition deadline missed."
                .to_string(),
        );
        let fed_tax: u64 = (u128::from(input.deferred_amount_cents)
            * u128::from(input.federal_marginal_rate_bps)
            / 10_000) as u64;
        return Section457aResult {
            severity: DeferralSeverity::Pre2009TransitionViolationMissed2017Window,
            is_nonqualified_entity: true,
            income_inclusion_year: SECTION_457A_TRANSITION_LAST_YEAR - 1,
            income_inclusion_amount_cents: input.deferred_amount_cents,
            federal_income_tax_cents: fed_tax,
            section_457a_20pct_additional_tax_cents: 0,
            interest_cents: 0,
            total_tax_burden_cents: fed_tax,
            recommended_actions: actions,
            citation: "26 U.S.C. § 457A(e); Notice 2009-8",
            notes,
        };
    }

    if input.substantial_risk_of_forfeiture_present {
        notes.push(
            "SROF currently present (future performance of substantial services required per \
             § 457A(d)(2)); § 457A defers income inclusion until SROF lapses. Reconfirm \
             SROF each tax year; performance-metric or profit-hurdle conditions do NOT \
             qualify as SROF under the § 457A standard."
                .to_string(),
        );
        actions.push(
            "Monitor SROF status annually. Upon SROF lapse, immediate inclusion required for \
             entire deferred amount unless amount-not-determinable rule of § 457A(c)(1)(B) \
             applies (which itself imposes 20% additional tax plus AFR + 1% interest when \
             amount eventually determinable)."
                .to_string(),
        );
        return Section457aResult {
            severity: DeferralSeverity::SrofPresentDeferralAllowed,
            is_nonqualified_entity: true,
            income_inclusion_year: input.year_srof_lapses,
            income_inclusion_amount_cents: 0,
            federal_income_tax_cents: 0,
            section_457a_20pct_additional_tax_cents: 0,
            interest_cents: 0,
            total_tax_burden_cents: 0,
            recommended_actions: actions,
            citation: "26 U.S.C. § 457A(a), (d)(2)",
            notes,
        };
    }

    if !input.amount_determinable_when_srof_lapses {
        let years_of_interest = input
            .year_amount_determinable
            .saturating_sub(input.year_srof_lapses)
            .max(0) as u64;
        let additional_tax_20pct: u64 = (u128::from(input.deferred_amount_cents)
            * u128::from(SECTION_457A_ADDITIONAL_TAX_BPS)
            / 10_000) as u64;
        let interest_rate_bps = input.afr_bps.saturating_add(SECTION_457A_AFR_PLUS_BPS);
        let interest_cents: u64 = (u128::from(input.deferred_amount_cents)
            * u128::from(interest_rate_bps)
            * u128::from(years_of_interest)
            / 10_000) as u64;
        let fed_tax: u64 = (u128::from(input.deferred_amount_cents)
            * u128::from(input.federal_marginal_rate_bps)
            / 10_000) as u64;
        let total = fed_tax
            .saturating_add(additional_tax_20pct)
            .saturating_add(interest_cents);
        actions.push(format!(
            "Amount not determinable when SROF lapsed in {}; § 457A(c)(1)(B) defers inclusion \
             until {} (determinable year) but imposes 20% additional tax on amount required \
             to be included PLUS interest at AFR + 1% from year of SROF lapse to year of \
             inclusion. Years of interest accrual: {}.",
            input.year_srof_lapses, input.year_amount_determinable, years_of_interest
        ));
        actions.push(
            "Consider restructuring deferral to determinable form (fixed dollar amount or \
             formula yielding determinable amount at SROF lapse) to avoid 20% additional \
             tax. Existing structure exposes participant to 20% additional tax plus AFR + 1% \
             interest."
                .to_string(),
        );
        return Section457aResult {
            severity: DeferralSeverity::AmountNotDeterminablePending20PctPlusInterest,
            is_nonqualified_entity: true,
            income_inclusion_year: input.year_amount_determinable,
            income_inclusion_amount_cents: input.deferred_amount_cents,
            federal_income_tax_cents: fed_tax,
            section_457a_20pct_additional_tax_cents: additional_tax_20pct,
            interest_cents,
            total_tax_burden_cents: total,
            recommended_actions: actions,
            citation: "26 U.S.C. § 457A(c)(1)(B)(i)–(ii)",
            notes,
        };
    }

    let fed_tax: u64 = (u128::from(input.deferred_amount_cents)
        * u128::from(input.federal_marginal_rate_bps)
        / 10_000) as u64;
    actions.push(format!(
        "No SROF; entire deferred amount of {} cents includible in gross income in year SROF \
         lapses ({}); ordinary-income-rate federal tax applies. Coordinate state-tax \
         inclusion separately.",
        input.deferred_amount_cents, input.year_srof_lapses
    ));
    actions.push(
        "Document § 457A income inclusion on Form W-2 (if employee) or Form 1099-NEC (if \
         contractor); employer should report on Box 11 of W-2 for NQDC. Confirm § 409A \
         documentation also satisfied per § 457A(d)(4) coordination rule."
            .to_string(),
    );
    notes.push(
        "Coordination with [[section_409a]] (§ 457A applies IN ADDITION to § 409A per § \
         457A(d)(4); both regimes can simultaneously trigger). Distinct from [[section_457b]] \
         (tax-exempt governmental NQDC plans) and [[section_280g]] (golden parachutes on \
         change in control)."
            .to_string(),
    );

    Section457aResult {
        severity: DeferralSeverity::NoSrofIncomeImmediatelyIncludible,
        is_nonqualified_entity: true,
        income_inclusion_year: input.year_srof_lapses,
        income_inclusion_amount_cents: input.deferred_amount_cents,
        federal_income_tax_cents: fed_tax,
        section_457a_20pct_additional_tax_cents: 0,
        interest_cents: 0,
        total_tax_burden_cents: fed_tax,
        recommended_actions: actions,
        citation: "26 U.S.C. § 457A(a), (d)(2), (d)(4)",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> Section457aInput {
        Section457aInput {
            entity_type: EntityType::ForeignCorpTaxIndifferent,
            deferral_type: DeferralType::CashCompensationDeferral,
            substantial_risk_of_forfeiture_present: false,
            amount_determinable_when_srof_lapses: true,
            deferred_amount_cents: 10_000_000_00,
            services_performed_year: 2023,
            pre_2009_deferral_included_by_2017: false,
            year_srof_lapses: 2024,
            year_amount_determinable: 2024,
            federal_marginal_rate_bps: 3_700,
            afr_bps: 500,
        }
    }

    #[test]
    fn domestic_c_corp_not_nonqualified_entity() {
        let mut i = baseline();
        i.entity_type = EntityType::DomesticCCorporation;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            DeferralSeverity::NotANonqualifiedEntity
        ));
        assert!(!r.is_nonqualified_entity);
        assert_eq!(r.section_457a_20pct_additional_tax_cents, 0);
        assert!(r.notes.iter().any(|n| n.contains("§ 409A")));
    }

    #[test]
    fn domestic_s_corp_not_nonqualified_entity() {
        let mut i = baseline();
        i.entity_type = EntityType::DomesticSCorporation;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            DeferralSeverity::NotANonqualifiedEntity
        ));
    }

    #[test]
    fn foreign_corp_with_comprehensive_foreign_tax_not_nonqualified() {
        let mut i = baseline();
        i.entity_type = EntityType::ForeignCorpComprehensiveForeignTax;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            DeferralSeverity::NotANonqualifiedEntity
        ));
        assert!(!r.is_nonqualified_entity);
    }

    #[test]
    fn foreign_corp_eci_not_nonqualified() {
        let mut i = baseline();
        i.entity_type = EntityType::ForeignCorpEffectivelyConnectedUSTrade;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            DeferralSeverity::NotANonqualifiedEntity
        ));
    }

    #[test]
    fn partnership_mostly_taxable_us_partners_not_nonqualified() {
        let mut i = baseline();
        i.entity_type = EntityType::PartnershipMostlyTaxableUSPartners;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            DeferralSeverity::NotANonqualifiedEntity
        ));
    }

    #[test]
    fn cayman_offshore_feeder_is_nonqualified_entity() {
        let i = baseline();
        let r = check(&i);
        assert!(r.is_nonqualified_entity);
        assert!(matches!(
            r.severity,
            DeferralSeverity::NoSrofIncomeImmediatelyIncludible
        ));
    }

    #[test]
    fn partnership_tax_indifferent_allocations_is_nonqualified() {
        let mut i = baseline();
        i.entity_type = EntityType::PartnershipTaxIndifferentAllocations;
        let r = check(&i);
        assert!(r.is_nonqualified_entity);
    }

    #[test]
    fn section_409a_exempt_stock_option_safe_harbor() {
        let mut i = baseline();
        i.deferral_type = DeferralType::Section409aExemptStockOption;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            DeferralSeverity::StockRightExemptUnder409a
        ));
        assert_eq!(r.income_inclusion_amount_cents, 0);
        assert!(r.notes.iter().any(|n| n.contains("Notice 2009-8 Q&A 2")));
        assert!(r.notes.iter().any(|n| n.contains("Rev. Rul. 2014-18")));
    }

    #[test]
    fn section_409a_exempt_stock_settled_sar_safe_harbor() {
        let mut i = baseline();
        i.deferral_type = DeferralType::Section409aExemptStockSettledSar;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            DeferralSeverity::StockRightExemptUnder409a
        ));
    }

    #[test]
    fn pre_2009_service_included_by_2017_compliant() {
        let mut i = baseline();
        i.services_performed_year = 2007;
        i.pre_2009_deferral_included_by_2017 = true;
        let r = check(&i);
        assert!(matches!(r.severity, DeferralSeverity::NotApplicable));
        assert!(r.notes.iter().any(|n| n.contains("before 2017")));
        assert_eq!(r.section_457a_20pct_additional_tax_cents, 0);
    }

    #[test]
    fn pre_2009_service_not_included_by_2017_transition_violation() {
        let mut i = baseline();
        i.services_performed_year = 2007;
        i.pre_2009_deferral_included_by_2017 = false;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            DeferralSeverity::Pre2009TransitionViolationMissed2017Window
        ));
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("amended returns")));
        assert_eq!(r.income_inclusion_amount_cents, i.deferred_amount_cents);
    }

    #[test]
    fn pre_2009_2008_boundary_still_treated_as_pre_2009() {
        let mut i = baseline();
        i.services_performed_year = 2008;
        i.pre_2009_deferral_included_by_2017 = true;
        let r = check(&i);
        assert!(matches!(r.severity, DeferralSeverity::NotApplicable));
    }

    #[test]
    fn srof_present_defers_inclusion() {
        let mut i = baseline();
        i.substantial_risk_of_forfeiture_present = true;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            DeferralSeverity::SrofPresentDeferralAllowed
        ));
        assert_eq!(r.income_inclusion_amount_cents, 0);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("future performance of substantial services")));
    }

    #[test]
    fn no_srof_immediate_inclusion() {
        let i = baseline();
        let r = check(&i);
        assert!(matches!(
            r.severity,
            DeferralSeverity::NoSrofIncomeImmediatelyIncludible
        ));
        assert_eq!(r.income_inclusion_amount_cents, i.deferred_amount_cents);
        let expected_fed_tax =
            i.deferred_amount_cents * u64::from(i.federal_marginal_rate_bps) / 10_000;
        assert_eq!(r.federal_income_tax_cents, expected_fed_tax);
        assert_eq!(r.section_457a_20pct_additional_tax_cents, 0);
        assert_eq!(r.interest_cents, 0);
    }

    #[test]
    fn amount_not_determinable_triggers_20_pct_plus_interest() {
        let mut i = baseline();
        i.amount_determinable_when_srof_lapses = false;
        i.year_srof_lapses = 2024;
        i.year_amount_determinable = 2027;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            DeferralSeverity::AmountNotDeterminablePending20PctPlusInterest
        ));
        let expected_20pct =
            i.deferred_amount_cents * u64::from(SECTION_457A_ADDITIONAL_TAX_BPS) / 10_000;
        assert_eq!(r.section_457a_20pct_additional_tax_cents, expected_20pct);
        let years = 3u64;
        let expected_interest = i.deferred_amount_cents
            * u64::from(i.afr_bps.saturating_add(SECTION_457A_AFR_PLUS_BPS))
            * years
            / 10_000;
        assert_eq!(r.interest_cents, expected_interest);
        assert_eq!(r.income_inclusion_year, 2027);
    }

    #[test]
    fn amount_not_determinable_same_year_zero_interest() {
        let mut i = baseline();
        i.amount_determinable_when_srof_lapses = false;
        i.year_srof_lapses = 2024;
        i.year_amount_determinable = 2024;
        let r = check(&i);
        assert_eq!(r.interest_cents, 0);
    }

    #[test]
    fn additional_tax_constant_is_20_percent() {
        assert_eq!(SECTION_457A_ADDITIONAL_TAX_BPS, 2_000);
    }

    #[test]
    fn interest_rate_addition_is_one_percent() {
        assert_eq!(SECTION_457A_AFR_PLUS_BPS, 100);
    }

    #[test]
    fn effective_year_pins_2009() {
        assert_eq!(SECTION_457A_EFFECTIVE_YEAR, 2009);
    }

    #[test]
    fn transition_last_year_pins_2017() {
        assert_eq!(SECTION_457A_TRANSITION_LAST_YEAR, 2017);
    }

    #[test]
    fn coordination_with_409a_pinned_in_notes() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("section_409a")));
        assert!(r.notes.iter().any(|n| n.contains("§ 457A(d)(4)")));
    }

    #[test]
    fn coordination_references_section_457b_and_section_280g() {
        let i = baseline();
        let r = check(&i);
        assert!(r.notes.iter().any(|n| n.contains("section_457b")));
        assert!(r.notes.iter().any(|n| n.contains("section_280g")));
    }

    #[test]
    fn citation_pins_457a_subsections() {
        let i = baseline();
        let r = check(&i);
        assert!(r.citation.contains("§ 457A"));
        assert!(r.citation.contains("(a)"));
        assert!(r.citation.contains("(d)(2)"));
    }

    #[test]
    fn amount_not_determinable_citation_pins_457a_c_1_b() {
        let mut i = baseline();
        i.amount_determinable_when_srof_lapses = false;
        let r = check(&i);
        assert!(r.citation.contains("§ 457A(c)(1)(B)"));
    }

    #[test]
    fn pre_2009_transition_citation_pins_notice_2009_8() {
        let mut i = baseline();
        i.services_performed_year = 2005;
        i.pre_2009_deferral_included_by_2017 = false;
        let r = check(&i);
        assert!(r.citation.contains("Notice 2009-8"));
        assert!(r.citation.contains("§ 457A(e)"));
    }

    #[test]
    fn stock_option_safe_harbor_citation_pins_rev_rul_2014_18() {
        let mut i = baseline();
        i.deferral_type = DeferralType::Section409aExemptStockOption;
        let r = check(&i);
        assert!(r.citation.contains("Rev. Rul. 2014-18"));
        assert!(r.citation.contains("Notice 2009-8 Q&A 2"));
    }

    #[test]
    fn full_tax_burden_sums_components() {
        let mut i = baseline();
        i.amount_determinable_when_srof_lapses = false;
        i.year_srof_lapses = 2024;
        i.year_amount_determinable = 2026;
        let r = check(&i);
        let sum = r.federal_income_tax_cents
            + r.section_457a_20pct_additional_tax_cents
            + r.interest_cents;
        assert_eq!(r.total_tax_burden_cents, sum);
    }

    #[test]
    fn high_deferral_amount_does_not_panic_overflow_defense() {
        let mut i = baseline();
        i.deferred_amount_cents = u64::MAX / 4;
        i.amount_determinable_when_srof_lapses = false;
        i.year_srof_lapses = 2024;
        i.year_amount_determinable = 2030;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            DeferralSeverity::AmountNotDeterminablePending20PctPlusInterest
        ));
    }

    #[test]
    fn zero_deferral_zero_tax() {
        let mut i = baseline();
        i.deferred_amount_cents = 0;
        let r = check(&i);
        assert_eq!(r.income_inclusion_amount_cents, 0);
        assert_eq!(r.federal_income_tax_cents, 0);
        assert_eq!(r.total_tax_burden_cents, 0);
    }

    #[test]
    fn cayman_master_feeder_hedge_fund_classic_case() {
        let mut i = baseline();
        i.entity_type = EntityType::ForeignCorpTaxIndifferent;
        i.deferral_type = DeferralType::CashCompensationDeferral;
        i.substantial_risk_of_forfeiture_present = false;
        i.deferred_amount_cents = 50_000_000_00;
        i.federal_marginal_rate_bps = 3_700;
        let r = check(&i);
        assert!(matches!(
            r.severity,
            DeferralSeverity::NoSrofIncomeImmediatelyIncludible
        ));
        let expected_fed_tax = 50_000_000_00u64 * 3_700 / 10_000;
        assert_eq!(r.federal_income_tax_cents, expected_fed_tax);
    }

    #[test]
    fn partnership_tax_indifferent_with_phantom_equity_triggers_457a() {
        let mut i = baseline();
        i.entity_type = EntityType::PartnershipTaxIndifferentAllocations;
        i.deferral_type = DeferralType::NonexemptPhantomEquityOrProfit;
        let r = check(&i);
        assert!(r.is_nonqualified_entity);
        assert!(matches!(
            r.severity,
            DeferralSeverity::NoSrofIncomeImmediatelyIncludible
        ));
    }

    #[test]
    fn srof_failed_metric_only_treated_as_no_srof() {
        let i = baseline();
        let r = check(&i);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("entire deferred amount")));
    }

    #[test]
    fn amount_not_determinable_pending_recommends_restructure() {
        let mut i = baseline();
        i.amount_determinable_when_srof_lapses = false;
        let r = check(&i);
        assert!(r
            .recommended_actions
            .iter()
            .any(|a| a.contains("restructuring deferral to determinable form")));
    }
}
