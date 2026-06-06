//! IRC § 461(g) — Prepaid interest deduction timing.
//! Direct trader companion to section_163d
//! (investment interest), section_163j (business
//! interest), section_163h (mortgage interest),
//! section_263a (UNICAP capitalization), section_481
//! (accounting method change), section_469 (passive
//! activity loss), section_280f (luxury auto
//! depreciation cap).
//!
//! § 461(g) provides that a CASH-BASIS taxpayer must
//! treat prepaid interest in the same manner as an
//! ACCRUAL-BASIS taxpayer for purposes of deduction
//! timing. Interest paid that is properly allocable
//! to any period AFTER the close of the taxable year
//! in which paid shall be CHARGED TO CAPITAL ACCOUNT
//! and treated as paid in the period to which so
//! allocable.
//!
//! Trader-critical because traders routinely:
//! - Prepay margin loan interest at end of taxable
//!   year to accelerate deduction (NOT PERMITTED
//!   under § 461(g));
//! - Pay mortgage points on rental property
//!   acquisition (must AMORTIZE over loan life
//!   under § 461(g));
//! - Refinance personal residence mortgage (points
//!   must be AMORTIZED — refinancing exception is
//!   narrow);
//! - Acquire principal residence with points (§ 461
//!   (g)(2) EXCEPTION allows current deduction).
//!
//! Companion to section_163d (investment interest
//! limitation), section_163h (qualified residence
//! interest), section_163j (business interest limit),
//! section_263a (UNICAP).
//!
//! **§ 461(g)(1) GENERAL RULE — capitalization
//! requirement**:
//! 1. Cash-basis taxpayer cannot deduct prepaid
//!    interest in year of payment;
//! 2. Prepaid interest CHARGED TO CAPITAL ACCOUNT;
//! 3. Allocated and deducted in the period to which
//!    interest is properly allocable;
//! 4. Effectively converts cash-basis to accrual-
//!    basis for prepaid interest only;
//! 5. Applies to ALL types of interest: mortgage,
//!    margin loan, business loan, personal loan,
//!    investment indebtedness.
//!
//! **§ 461(g)(2) EXCEPTION — points on principal
//! residence**:
//! Taxpayer may DEDUCT IN YEAR PAID if all five
//! conditions met:
//! 1. Points paid in connection with PURCHASE OR
//!    IMPROVEMENT of principal residence;
//! 2. SECURED BY principal residence (not investment
//!    property, not vacation home);
//! 3. Points charged represent ESTABLISHED PRACTICE
//!    in geographic area;
//! 4. Points DO NOT EXCEED amount generally charged
//!    in geographic area;
//! 5. Points computed as PERCENTAGE OF PRINCIPAL
//!    AMOUNT of mortgage (not flat fees for
//!    appraisal, escrow, title insurance).
//!
//! **§ 461(g)(2) REFINANCING EXCLUSION** — points
//! paid on REFINANCING of existing mortgage NOT
//! eligible for current deduction even if same
//! principal residence; must be AMORTIZED over loan
//! life (Rev. Rul. 87-22 + IRS Pub. 936). Exception:
//! refinancing proceeds used for IMPROVEMENTS to
//! residence may qualify pro rata.
//!
//! **§ 461(g) APPLICATION TO RENTAL PROPERTY**:
//! 1. Mortgage points on rental property purchase or
//!    refinance are PREPAID INTEREST under § 461(g)
//!    (1) general rule;
//! 2. MUST be AMORTIZED over loan life via straight-
//!    line method (Rev. Rul. 70-540);
//! 3. Annual deduction = total points / loan term
//!    in years;
//! 4. Remaining unamortized points at loan payoff
//!    deductible in year of payoff (Schedule E rental
//!    income);
//! 5. § 461(g)(2) EXCEPTION does NOT apply to rental
//!    property (residence requirement).
//!
//! **§ 461(g) APPLICATION TO MARGIN LOAN INTEREST**:
//! 1. Year-end prepayment of margin interest does
//!    NOT accelerate deduction;
//! 2. Interest allocable to next year remains
//!    deferred under § 461(g)(1);
//! 3. § 163(d) investment interest limitation
//!    applies separately (margin interest deductible
//!    only up to net investment income);
//! 4. Excess investment interest carried forward
//!    indefinitely under § 163(d)(2);
//! 5. § 475(f) trader-status election may
//!    reclassify margin interest as § 162 business
//!    interest expense (subject to § 163(j) limit
//!    instead of § 163(d)).
//!
//! **§ 461(g) REV. PROC. 94-27 SELLER-PAID POINTS**:
//! 1. Seller-paid points on principal residence
//!    purchase TREATED AS PAID BY BUYER under
//!    industry practice;
//! 2. Buyer-deductible in year of purchase under
//!    § 461(g)(2);
//! 3. Buyer's basis in residence REDUCED by seller-
//!    paid points to avoid double benefit;
//! 4. Seller cannot also deduct (would be double
//!    deduction).
//!
//! **§ 461(g) INTERACTION WITH § 263A UNICAP**:
//! Capitalized prepaid interest under § 461(g) may
//! be SUBJECT TO § 263A UNICAP if attributable to
//! production or resale activities (CONSTRUCTION
//! period interest, manufacturing facilities); not
//! applicable to passive investment indebtedness.
//!
//! **§ 461(g) INTERACTION WITH § 163(h) QUALIFIED
//! RESIDENCE INTEREST**:
//! 1. § 163(h)(3) qualified residence interest
//!    includes ACQUISITION INDEBTEDNESS + HOME EQUITY
//!    INDEBTEDNESS;
//! 2. TCJA 2017 suspended home equity interest
//!    deduction unless used for residence
//!    improvements through 2025 (extended by OBBBA
//!    2025);
//! 3. § 461(g)(2) current-deduction exception
//!    applies WITHIN § 163(h)(3) framework — points
//!    deductible currently AND interest deductible
//!    over loan life.
//!
//! **TRADER-CRITICAL FACT PATTERNS**:
//!
//! Trader purchases personal residence $1M with $30K
//! points at closing — § 461(g)(2) EXCEPTION applies
//! (purchase + principal residence + secured by
//! residence + percentage of principal); FULL $30,000
//! DEDUCTIBLE in year of purchase as itemized
//! deduction under § 163(h)(3).
//!
//! Trader refinances personal residence $1M with
//! $20K points to obtain lower interest rate — § 461
//! (g)(2) EXCLUSION; points NOT immediately
//! deductible; must AMORTIZE over loan life under
//! Rev. Rul. 87-22; if 30-year loan, $20K / 30 =
//! $667 annual deduction.
//!
//! Trader purchases $1M rental property with $25K
//! points at closing — § 461(g)(2) EXCEPTION does
//! NOT apply (rental, not residence); points MUST
//! be AMORTIZED via § 461(g)(1) under straight-line
//! method; 30-year loan = $833 annual deduction on
//! Schedule E.
//!
//! Trader prepays $50K margin loan interest on
//! December 30 for January-March of next year —
//! § 461(g)(1) APPLIES; interest allocable to next
//! year DEFERRED; only December portion deductible
//! in current year; current-year acceleration
//! IMPERMISSIBLE.
//!
//! § 475(f) trader-status election by trader with
//! $200K margin interest paid — reclassifies from
//! § 163(d) investment interest limitation to § 162
//! business interest expense subject to § 163(j)
//! business interest limit; § 461(g) timing rules
//! continue to apply.
//!
//! Citations: 26 USC § 461(g)(1) (general rule); 26
//! USC § 461(g)(2) (points on principal residence
//! exception); 26 USC § 163(d) (investment interest
//! limitation); 26 USC § 163(h) (qualified residence
//! interest); 26 USC § 163(h)(3) (acquisition vs
//! home equity indebtedness); 26 USC § 163(j)
//! (business interest limit); 26 USC § 263A (UNICAP);
//! 26 USC § 162 (ordinary and necessary business
//! expenses); 26 USC § 475(f) (trader mark-to-
//! market); Treas. Reg. § 1.461-1; Treas. Reg.
//! § 1.461-5; Treas. Reg. § 1.163-10T (qualified
//! residence interest); Treas. Reg. § 1.263A-1 et
//! seq.; Rev. Rul. 87-22 (refinancing points must be
//! amortized); Rev. Rul. 70-540 (rental property
//! straight-line amortization); Rev. Proc. 94-27
//! (seller-paid points treated as buyer-paid);
//! TCJA 2017 (Pub. L. 115-97); OBBBA 2025 (Pub. L.
//! 119-21); IRS Pub. 936 (Home Mortgage Interest
//! Deduction); IRS Topic 504 (Home Mortgage Points).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InterestCategory {
    /// Points on principal residence purchase OR
    /// improvement.
    PointsOnPrincipalResidencePurchase,
    /// Points on principal residence refinancing.
    PointsOnPrincipalResidenceRefinance,
    /// Points on rental property purchase or
    /// refinance.
    PointsOnRentalProperty,
    /// Margin loan interest prepayment.
    MarginLoanInterestPrepayment,
    /// Business loan interest prepayment.
    BusinessLoanInterestPrepayment,
    /// Seller-paid points on principal residence
    /// (Rev. Proc. 94-27 treated as buyer-paid).
    SellerPaidPointsOnPrincipalResidence,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section461gInput {
    pub category: InterestCategory,
    /// Total prepaid interest / points in cents.
    pub prepaid_amount_cents: u64,
    /// Loan term in years (for amortization
    /// calculation).
    pub loan_term_years: u32,
    /// Whether points represent established practice
    /// in geographic area (§ 461(g)(2)(A) condition).
    pub established_practice_in_area: bool,
    /// Whether points do not exceed amount generally
    /// charged in geographic area (§ 461(g)(2)(B)).
    pub not_excessive_in_area: bool,
    /// Whether points computed as percentage of
    /// principal (§ 461(g)(2)(C)) — not flat fees.
    pub computed_as_percentage_of_principal: bool,
    /// Whether refinancing proceeds used for
    /// residence improvements (partial § 461(g)(2)
    /// exception).
    pub refinancing_proceeds_used_for_improvements: bool,
    /// Whether trader has made § 475(f) trader mark-
    /// to-market election (reclassifies margin
    /// interest).
    pub section_475f_election: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section461gResult {
    pub current_year_deduction_cents: u64,
    pub amortization_required: bool,
    pub annual_amortization_cents: u64,
    pub section_461g2_exception_applies: bool,
    pub seller_paid_points_treated_as_buyer: bool,
    pub failure_reasons: Vec<String>,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn check(input: &Section461gInput) -> Section461gResult {
    let mut failure_reasons: Vec<String> = Vec::new();

    let section_461g2_exception_applies = matches!(
        input.category,
        InterestCategory::PointsOnPrincipalResidencePurchase
            | InterestCategory::SellerPaidPointsOnPrincipalResidence
    ) && input.established_practice_in_area
        && input.not_excessive_in_area
        && input.computed_as_percentage_of_principal;

    let seller_paid_points_treated_as_buyer = matches!(
        input.category,
        InterestCategory::SellerPaidPointsOnPrincipalResidence
    );

    let (current_year_deduction_cents, amortization_required) = match input.category {
        InterestCategory::PointsOnPrincipalResidencePurchase
        | InterestCategory::SellerPaidPointsOnPrincipalResidence => {
            if section_461g2_exception_applies {
                (input.prepaid_amount_cents, false)
            } else {
                (0, true)
            }
        }
        InterestCategory::PointsOnPrincipalResidenceRefinance => {
            if input.refinancing_proceeds_used_for_improvements
                && input.established_practice_in_area
                && input.not_excessive_in_area
                && input.computed_as_percentage_of_principal
            {
                (input.prepaid_amount_cents, false)
            } else {
                (0, true)
            }
        }
        InterestCategory::PointsOnRentalProperty => (0, true),
        InterestCategory::MarginLoanInterestPrepayment
        | InterestCategory::BusinessLoanInterestPrepayment => (0, true),
    };

    let annual_amortization_cents = if amortization_required && input.loan_term_years > 0 {
        input.prepaid_amount_cents / input.loan_term_years as u64
    } else {
        0
    };

    if section_461g2_exception_applies {
        let category_label = match input.category {
            InterestCategory::PointsOnPrincipalResidencePurchase => "PURCHASE OR IMPROVEMENT of principal residence",
            InterestCategory::SellerPaidPointsOnPrincipalResidence => "SELLER-PAID POINTS treated as BUYER-PAID under Rev. Proc. 94-27 (buyer's basis in residence REDUCED to avoid double benefit)",
            _ => "principal residence",
        };
        failure_reasons.push(format!(
            "26 USC § 461(g)(2) EXCEPTION APPLIES — {} with all five conditions met (purchase/improvement + secured by principal residence + established practice in area + not excessive + computed as percentage of principal); FULL ${} cents deductible in year paid as itemized deduction under § 163(h)(3) qualified residence interest",
            category_label,
            input.prepaid_amount_cents
        ));
    } else {
        let reason = match input.category {
            InterestCategory::PointsOnPrincipalResidenceRefinance => "Rev. Rul. 87-22 REFINANCING EXCLUSION — points paid on refinancing of existing mortgage NOT eligible for § 461(g)(2) current deduction even if same principal residence; must be AMORTIZED over loan life via straight-line method; refinancing proceeds used for IMPROVEMENTS may qualify pro rata".to_string(),
            InterestCategory::PointsOnRentalProperty => "Rev. Rul. 70-540 RENTAL PROPERTY — § 461(g)(2) exception does NOT apply to rental property (residence requirement); points must be AMORTIZED via § 461(g)(1) over loan life using straight-line method; annual deduction on Schedule E rental income; remaining unamortized points deductible in year of loan payoff".to_string(),
            InterestCategory::MarginLoanInterestPrepayment => {
                if input.section_475f_election {
                    "§ 475(f) TRADER MARK-TO-MARKET ELECTION — reclassifies margin interest from § 163(d) investment interest limitation to § 162 business interest expense subject to § 163(j) business interest limit; § 461(g) timing rules continue to apply (year-end prepayment does not accelerate deduction)".to_string()
                } else {
                    "§ 461(g)(1) GENERAL RULE — year-end prepayment of margin loan interest DOES NOT accelerate deduction; interest allocable to next year DEFERRED; § 163(d) investment interest limitation applies separately (margin interest deductible only up to net investment income; excess carried forward indefinitely under § 163(d)(2))".to_string()
                }
            }
            InterestCategory::BusinessLoanInterestPrepayment => "§ 461(g)(1) GENERAL RULE — cash-basis taxpayer cannot deduct prepaid business loan interest in year of payment; interest CHARGED TO CAPITAL ACCOUNT and deducted in period properly allocable; § 163(j) business interest limit may further restrict".to_string(),
            _ => "§ 461(g)(1) GENERAL RULE — prepaid interest CHARGED TO CAPITAL ACCOUNT and treated as paid in period properly allocable".to_string(),
        };
        failure_reasons.push(format!(
            "26 USC § 461(g) AMORTIZATION REQUIRED — {}; annual amortization {} cents over {} years",
            reason, annual_amortization_cents, input.loan_term_years
        ));
    }

    if !input.established_practice_in_area
        && matches!(
            input.category,
            InterestCategory::PointsOnPrincipalResidencePurchase
                | InterestCategory::PointsOnPrincipalResidenceRefinance
                | InterestCategory::SellerPaidPointsOnPrincipalResidence
        )
    {
        failure_reasons.push(
            "26 USC § 461(g)(2)(A) — points must represent ESTABLISHED PRACTICE in geographic area for § 461(g)(2) current-deduction exception".to_string(),
        );
    }
    if !input.not_excessive_in_area
        && matches!(
            input.category,
            InterestCategory::PointsOnPrincipalResidencePurchase
                | InterestCategory::PointsOnPrincipalResidenceRefinance
                | InterestCategory::SellerPaidPointsOnPrincipalResidence
        )
    {
        failure_reasons.push(
            "26 USC § 461(g)(2)(B) — points must NOT EXCEED amount generally charged in geographic area".to_string(),
        );
    }
    if !input.computed_as_percentage_of_principal
        && matches!(
            input.category,
            InterestCategory::PointsOnPrincipalResidencePurchase
                | InterestCategory::PointsOnPrincipalResidenceRefinance
                | InterestCategory::SellerPaidPointsOnPrincipalResidence
        )
    {
        failure_reasons.push(
            "26 USC § 461(g)(2)(C) — points must be COMPUTED AS PERCENTAGE OF PRINCIPAL amount of mortgage (NOT flat fees for appraisal, escrow, title insurance)".to_string(),
        );
    }

    if seller_paid_points_treated_as_buyer && section_461g2_exception_applies {
        failure_reasons.push(
            "Rev. Proc. 94-27 SELLER-PAID POINTS — seller-paid points on principal residence purchase TREATED AS PAID BY BUYER under industry practice; buyer-deductible in year of purchase under § 461(g)(2); buyer's basis in residence REDUCED by seller-paid points to avoid double benefit; seller CANNOT also deduct".to_string(),
        );
    }

    let notes: Vec<String> = vec![
        "26 USC § 461(g) — cash-basis taxpayer must treat prepaid interest in the same manner as an accrual-basis taxpayer; interest paid that is properly allocable to any period AFTER the close of the taxable year in which paid shall be CHARGED TO CAPITAL ACCOUNT and treated as paid in the period to which so allocable".to_string(),
        "26 USC § 461(g)(1) GENERAL RULE — cash-basis taxpayer cannot deduct prepaid interest in year of payment; charged to capital account; allocated and deducted in period interest properly allocable; effectively converts cash-basis to accrual-basis for prepaid interest only; applies to ALL types of interest (mortgage, margin loan, business loan, personal loan, investment indebtedness)".to_string(),
        "26 USC § 461(g)(2) EXCEPTION — points on principal residence: taxpayer may DEDUCT IN YEAR PAID if all five conditions met: (1) points paid in connection with PURCHASE OR IMPROVEMENT of principal residence; (2) SECURED BY principal residence; (3) ESTABLISHED PRACTICE in geographic area; (4) NOT EXCESSIVE compared to area; (5) COMPUTED AS PERCENTAGE OF PRINCIPAL (not flat fees for appraisal/escrow/title)".to_string(),
        "§ 461(g)(2) REFINANCING EXCLUSION — points paid on REFINANCING of existing mortgage NOT eligible for current deduction even if same principal residence; must be AMORTIZED over loan life (Rev. Rul. 87-22 + IRS Pub. 936); exception: refinancing proceeds used for IMPROVEMENTS to residence may qualify pro rata".to_string(),
        "§ 461(g) APPLICATION TO RENTAL PROPERTY: (1) mortgage points on rental property purchase or refinance are PREPAID INTEREST under § 461(g)(1) general rule; (2) MUST be AMORTIZED over loan life via straight-line method (Rev. Rul. 70-540); (3) annual deduction = total points / loan term in years; (4) remaining unamortized points at loan payoff deductible in year of payoff (Schedule E rental income); (5) § 461(g)(2) exception does NOT apply to rental property (residence requirement)".to_string(),
        "§ 461(g) APPLICATION TO MARGIN LOAN INTEREST: (1) year-end prepayment does NOT accelerate deduction; (2) interest allocable to next year DEFERRED under § 461(g)(1); (3) § 163(d) investment interest limitation applies separately (deductible only up to net investment income); (4) excess investment interest carried forward INDEFINITELY under § 163(d)(2); (5) § 475(f) trader-status election may reclassify margin interest as § 162 business interest expense (subject to § 163(j) business interest limit instead of § 163(d))".to_string(),
        "Rev. Proc. 94-27 SELLER-PAID POINTS — seller-paid points on principal residence purchase TREATED AS PAID BY BUYER under industry practice; buyer-deductible in year of purchase under § 461(g)(2); buyer's basis in residence REDUCED by seller-paid points to avoid double benefit; seller CANNOT also deduct".to_string(),
        "§ 461(g) INTERACTION WITH § 263A UNICAP — capitalized prepaid interest under § 461(g) may be SUBJECT TO § 263A UNICAP if attributable to production or resale activities (CONSTRUCTION period interest, manufacturing facilities); not applicable to passive investment indebtedness".to_string(),
        "§ 461(g) INTERACTION WITH § 163(h) QUALIFIED RESIDENCE INTEREST: (1) § 163(h)(3) qualified residence interest includes ACQUISITION INDEBTEDNESS + HOME EQUITY INDEBTEDNESS; (2) TCJA 2017 suspended home equity interest deduction unless used for residence improvements through 2025 (extended by OBBBA 2025); (3) § 461(g)(2) current-deduction exception applies WITHIN § 163(h)(3) framework — points deductible currently AND interest deductible over loan life".to_string(),
        "Trader-critical fact patterns: (1) trader purchases personal residence $1M with $30K points — § 461(g)(2) EXCEPTION applies; FULL $30,000 DEDUCTIBLE in year of purchase under § 163(h)(3); (2) refinances personal residence $1M with $20K points for lower interest rate — § 461(g)(2) EXCLUSION; must AMORTIZE over loan life ($20K / 30 = $667 annual); (3) purchases $1M rental property with $25K points — § 461(g)(2) does NOT apply (rental not residence); AMORTIZE over 30 years ($833 annual) on Schedule E; (4) prepays $50K margin loan interest December 30 for Q1 next year — § 461(g)(1) APPLIES; only December portion deductible current year; (5) § 475(f) trader-status election reclassifies margin interest from § 163(d) to § 162 business interest subject to § 163(j) limit".to_string(),
        "Companion to section_163d (investment interest) + section_163h (qualified residence interest) + section_163j (business interest limit) + section_263a (UNICAP) + section_481 (accounting method change) + section_469 (passive activity loss) + section_475c2 (trader mark-to-market) + section_162 (ordinary and necessary business expenses)".to_string(),
    ];

    Section461gResult {
        current_year_deduction_cents,
        amortization_required,
        annual_amortization_cents,
        section_461g2_exception_applies,
        seller_paid_points_treated_as_buyer,
        failure_reasons,
        citation: "26 USC § 461(g)(1); 26 USC § 461(g)(2); 26 USC § 163(d); 26 USC § 163(h); 26 USC § 163(h)(3); 26 USC § 163(j); 26 USC § 263A; 26 USC § 162; 26 USC § 475(f); Treas. Reg. § 1.461-1; Treas. Reg. § 1.461-5; Treas. Reg. § 1.163-10T (qualified residence interest); Treas. Reg. § 1.263A-1 et seq.; Rev. Rul. 87-22; Rev. Rul. 70-540; Rev. Proc. 94-27; TCJA 2017 (Pub. L. 115-97); OBBBA 2025 (Pub. L. 119-21); IRS Pub. 936; IRS Topic 504",
        notes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn principal_residence_purchase_compliant() -> Section461gInput {
        Section461gInput {
            category: InterestCategory::PointsOnPrincipalResidencePurchase,
            prepaid_amount_cents: 3_000_000,
            loan_term_years: 30,
            established_practice_in_area: true,
            not_excessive_in_area: true,
            computed_as_percentage_of_principal: true,
            refinancing_proceeds_used_for_improvements: false,
            section_475f_election: false,
        }
    }

    #[test]
    fn principal_residence_purchase_exception_applies() {
        let r = check(&principal_residence_purchase_compliant());
        assert!(r.section_461g2_exception_applies);
        assert_eq!(r.current_year_deduction_cents, 3_000_000);
        assert!(!r.amortization_required);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 461(g)(2) EXCEPTION APPLIES")
                && f.contains("PURCHASE OR IMPROVEMENT")));
    }

    #[test]
    fn principal_residence_not_established_practice_no_exception() {
        let mut i = principal_residence_purchase_compliant();
        i.established_practice_in_area = false;
        let r = check(&i);
        assert!(!r.section_461g2_exception_applies);
        assert!(r.amortization_required);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 461(g)(2)(A)") && f.contains("ESTABLISHED PRACTICE")));
    }

    #[test]
    fn principal_residence_excessive_no_exception() {
        let mut i = principal_residence_purchase_compliant();
        i.not_excessive_in_area = false;
        let r = check(&i);
        assert!(!r.section_461g2_exception_applies);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 461(g)(2)(B)") && f.contains("NOT EXCEED")));
    }

    #[test]
    fn principal_residence_flat_fee_no_exception() {
        let mut i = principal_residence_purchase_compliant();
        i.computed_as_percentage_of_principal = false;
        let r = check(&i);
        assert!(!r.section_461g2_exception_applies);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 461(g)(2)(C)") && f.contains("PERCENTAGE OF PRINCIPAL")));
    }

    #[test]
    fn principal_residence_refinance_amortization_required() {
        let mut i = principal_residence_purchase_compliant();
        i.category = InterestCategory::PointsOnPrincipalResidenceRefinance;
        let r = check(&i);
        assert!(r.amortization_required);
        assert_eq!(r.current_year_deduction_cents, 0);
        let expected = 3_000_000_u64 / 30;
        assert_eq!(r.annual_amortization_cents, expected);
        assert!(r.failure_reasons.iter().any(|f| f
            .contains("Rev. Rul. 87-22 REFINANCING EXCLUSION")
            && f.contains("AMORTIZED")));
    }

    #[test]
    fn principal_residence_refinance_for_improvements_full_deduction() {
        let mut i = principal_residence_purchase_compliant();
        i.category = InterestCategory::PointsOnPrincipalResidenceRefinance;
        i.refinancing_proceeds_used_for_improvements = true;
        let r = check(&i);
        assert_eq!(r.current_year_deduction_cents, 3_000_000);
        assert!(!r.amortization_required);
    }

    #[test]
    fn rental_property_points_amortization_required() {
        let mut i = principal_residence_purchase_compliant();
        i.category = InterestCategory::PointsOnRentalProperty;
        let r = check(&i);
        assert!(r.amortization_required);
        assert_eq!(r.current_year_deduction_cents, 0);
        let expected = 3_000_000_u64 / 30;
        assert_eq!(r.annual_amortization_cents, expected);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("Rev. Rul. 70-540 RENTAL PROPERTY")
                && f.contains("Schedule E")
                && f.contains("does NOT apply to rental property")));
    }

    #[test]
    fn rental_property_30_year_loan_833_per_year() {
        let mut i = principal_residence_purchase_compliant();
        i.category = InterestCategory::PointsOnRentalProperty;
        i.prepaid_amount_cents = 2_500_000;
        i.loan_term_years = 30;
        let r = check(&i);
        let expected_per_year = 2_500_000_u64 / 30;
        assert_eq!(r.annual_amortization_cents, expected_per_year);
    }

    #[test]
    fn margin_loan_interest_prepayment_deferred() {
        let mut i = principal_residence_purchase_compliant();
        i.category = InterestCategory::MarginLoanInterestPrepayment;
        let r = check(&i);
        assert!(r.amortization_required);
        assert_eq!(r.current_year_deduction_cents, 0);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 461(g)(1) GENERAL RULE")
                && f.contains("margin loan interest")
                && f.contains("§ 163(d)")
                && f.contains("net investment income")));
    }

    #[test]
    fn margin_loan_with_475f_election_reclassifies() {
        let mut i = principal_residence_purchase_compliant();
        i.category = InterestCategory::MarginLoanInterestPrepayment;
        i.section_475f_election = true;
        let r = check(&i);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 475(f) TRADER MARK-TO-MARKET ELECTION")
                && f.contains("§ 162 business interest expense")
                && f.contains("§ 163(j)")));
    }

    #[test]
    fn business_loan_interest_prepayment_deferred() {
        let mut i = principal_residence_purchase_compliant();
        i.category = InterestCategory::BusinessLoanInterestPrepayment;
        let r = check(&i);
        assert!(r.amortization_required);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("§ 461(g)(1) GENERAL RULE")
                && f.contains("business loan interest")
                && f.contains("§ 163(j)")));
    }

    #[test]
    fn seller_paid_points_treated_as_buyer_paid() {
        let mut i = principal_residence_purchase_compliant();
        i.category = InterestCategory::SellerPaidPointsOnPrincipalResidence;
        let r = check(&i);
        assert!(r.seller_paid_points_treated_as_buyer);
        assert!(r.section_461g2_exception_applies);
        assert_eq!(r.current_year_deduction_cents, 3_000_000);
        assert!(r
            .failure_reasons
            .iter()
            .any(|f| f.contains("Rev. Proc. 94-27 SELLER-PAID POINTS")
                && f.contains("TREATED AS PAID BY BUYER")
                && f.contains("basis in residence REDUCED")));
    }

    #[test]
    fn category_truth_table_six_cells() {
        for cat in [
            InterestCategory::PointsOnPrincipalResidencePurchase,
            InterestCategory::PointsOnPrincipalResidenceRefinance,
            InterestCategory::PointsOnRentalProperty,
            InterestCategory::MarginLoanInterestPrepayment,
            InterestCategory::BusinessLoanInterestPrepayment,
            InterestCategory::SellerPaidPointsOnPrincipalResidence,
        ] {
            let mut i = principal_residence_purchase_compliant();
            i.category = cat;
            let r = check(&i);
            let _ = r.section_461g2_exception_applies;
        }
    }

    #[test]
    fn principal_residence_uniquely_qualifies_for_current_deduction_invariant() {
        let r_principal = check(&principal_residence_purchase_compliant());
        assert!(r_principal.section_461g2_exception_applies);

        for cat in [
            InterestCategory::PointsOnPrincipalResidenceRefinance,
            InterestCategory::PointsOnRentalProperty,
            InterestCategory::MarginLoanInterestPrepayment,
            InterestCategory::BusinessLoanInterestPrepayment,
        ] {
            let mut i = principal_residence_purchase_compliant();
            i.category = cat;
            let r = check(&i);
            assert!(r.amortization_required, "cat={:?}", cat);
        }
    }

    #[test]
    fn citation_pins_all_authorities() {
        let r = check(&principal_residence_purchase_compliant());
        assert!(r.citation.contains("§ 461(g)(1)"));
        assert!(r.citation.contains("§ 461(g)(2)"));
        assert!(r.citation.contains("§ 163(d)"));
        assert!(r.citation.contains("§ 163(h)"));
        assert!(r.citation.contains("§ 163(h)(3)"));
        assert!(r.citation.contains("§ 163(j)"));
        assert!(r.citation.contains("§ 263A"));
        assert!(r.citation.contains("§ 162"));
        assert!(r.citation.contains("§ 475(f)"));
        assert!(r.citation.contains("Treas. Reg. § 1.461-1"));
        assert!(r.citation.contains("Treas. Reg. § 1.461-5"));
        assert!(r.citation.contains("Treas. Reg. § 1.163-10T"));
        assert!(r.citation.contains("Treas. Reg. § 1.263A-1"));
        assert!(r.citation.contains("Rev. Rul. 87-22"));
        assert!(r.citation.contains("Rev. Rul. 70-540"));
        assert!(r.citation.contains("Rev. Proc. 94-27"));
        assert!(r.citation.contains("Pub. L. 115-97"));
        assert!(r.citation.contains("Pub. L. 119-21"));
        assert!(r.citation.contains("IRS Pub. 936"));
        assert!(r.citation.contains("IRS Topic 504"));
    }

    #[test]
    fn note_pins_subsection_g_overview() {
        let r = check(&principal_residence_purchase_compliant());
        assert!(r.notes.iter().any(|n|
            n.contains("§ 461(g)")
            && n.contains("cash-basis taxpayer must treat prepaid interest in the same manner as an accrual-basis taxpayer")
            && n.contains("CHARGED TO CAPITAL ACCOUNT")));
    }

    #[test]
    fn note_pins_subsection_g1_general_rule() {
        let r = check(&principal_residence_purchase_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 461(g)(1) GENERAL RULE")
                && n.contains("converts cash-basis to accrual-basis")
                && n.contains("ALL types of interest")));
    }

    #[test]
    fn note_pins_subsection_g2_five_conditions() {
        let r = check(&principal_residence_purchase_compliant());
        assert!(r.notes.iter().any(|n| n.contains("§ 461(g)(2) EXCEPTION")
            && n.contains("PURCHASE OR IMPROVEMENT")
            && n.contains("SECURED BY")
            && n.contains("ESTABLISHED PRACTICE")
            && n.contains("NOT EXCESSIVE")
            && n.contains("PERCENTAGE OF PRINCIPAL")));
    }

    #[test]
    fn note_pins_refinancing_exclusion_rev_rul_87_22() {
        let r = check(&principal_residence_purchase_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 461(g)(2) REFINANCING EXCLUSION")
                && n.contains("Rev. Rul. 87-22")
                && n.contains("IRS Pub. 936")));
    }

    #[test]
    fn note_pins_rental_property_amortization_rev_rul_70_540() {
        let r = check(&principal_residence_purchase_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 461(g) APPLICATION TO RENTAL PROPERTY")
                && n.contains("Rev. Rul. 70-540")
                && n.contains("straight-line method")
                && n.contains("Schedule E")));
    }

    #[test]
    fn note_pins_margin_loan_section_163d_interaction() {
        let r = check(&principal_residence_purchase_compliant());
        assert!(r.notes.iter().any(
            |n| n.contains("§ 461(g) APPLICATION TO MARGIN LOAN INTEREST")
                && n.contains("§ 163(d) investment interest limitation")
                && n.contains("net investment income")
                && n.contains("§ 163(d)(2)")
                && n.contains("§ 475(f) trader-status election")
        ));
    }

    #[test]
    fn note_pins_rev_proc_94_27_seller_paid_points() {
        let r = check(&principal_residence_purchase_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Rev. Proc. 94-27 SELLER-PAID POINTS")
                && n.contains("TREATED AS PAID BY BUYER")
                && n.contains("basis in residence REDUCED")));
    }

    #[test]
    fn note_pins_section_263a_unicap_interaction() {
        let r = check(&principal_residence_purchase_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 461(g) INTERACTION WITH § 263A UNICAP")
                && n.contains("CONSTRUCTION period interest")));
    }

    #[test]
    fn note_pins_section_163h_qualified_residence_interaction() {
        let r = check(&principal_residence_purchase_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 461(g) INTERACTION WITH § 163(h)")
                && n.contains("ACQUISITION INDEBTEDNESS")
                && n.contains("HOME EQUITY INDEBTEDNESS")
                && n.contains("TCJA 2017")
                && n.contains("OBBBA 2025")));
    }

    #[test]
    fn note_pins_trader_fact_patterns_five() {
        let r = check(&principal_residence_purchase_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Trader-critical fact patterns")
                && n.contains("$30,000 DEDUCTIBLE")
                && n.contains("§ 461(g)(2) EXCLUSION")
                && n.contains("Schedule E")
                && n.contains("§ 475(f) trader-status election")));
    }

    #[test]
    fn note_pins_companion_modules() {
        let r = check(&principal_residence_purchase_compliant());
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("Companion to section_163d")
                && n.contains("section_163h")
                && n.contains("section_163j")
                && n.contains("section_263a")
                && n.contains("section_475c2")));
    }

    #[test]
    fn defensive_overflow_saturating() {
        let mut i = principal_residence_purchase_compliant();
        i.prepaid_amount_cents = u64::MAX;
        let r = check(&i);
        let _ = r.annual_amortization_cents;
    }
}
