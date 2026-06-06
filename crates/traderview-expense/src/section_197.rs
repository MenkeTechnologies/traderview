//! IRC § 197 — Amortization of goodwill and certain other
//! intangibles. Trader-relevant when a trading entity acquires
//! another trading business (asset purchase of customer list,
//! workforce in place, goodwill, non-compete with seller, etc.) —
//! § 197 provides the EXCLUSIVE deduction pathway for "amortizable
//! section 197 intangibles" acquired after August 10, 1993 (TRA-93
//! effective date) via 15-year straight-line amortization.
//!
//! § 197(a) GENERAL RULE — taxpayer entitled to amortization
//! deduction with respect to any amortizable section 197 intangible.
//! Amount of deduction determined by AMORTIZING THE ADJUSTED BASIS
//! (for purposes of determining gain) of such intangible RATABLY
//! OVER THE 15-YEAR PERIOD beginning with the month in which such
//! intangible was acquired.
//!
//! § 197(b) NO OTHER DEPRECIATION OR AMORTIZATION DEDUCTION
//! ALLOWABLE — except as provided in subsection (a), no depreciation
//! or amortization deduction shall be allowable with respect to any
//! amortizable section 197 intangible.
//!
//! § 197(c) AMORTIZABLE SECTION 197 INTANGIBLE — must be (1)
//! acquired by the taxpayer after the date of the enactment (August
//! 10, 1993) and (2) held in connection with the conduct of a trade
//! or business or an activity described in § 212.
//!
//! § 197(d) SECTION 197 INTANGIBLE — defined categories:
//!   (d)(1)(A) goodwill
//!   (d)(1)(B) going concern value
//!   (d)(1)(C) workforce in place (composition + terms and
//!     conditions of employment)
//!   (d)(1)(D) business books and records, operating systems,
//!     other information bases
//!   (d)(1)(E) any patent, copyright, formula, process, design,
//!     pattern, knowhow, format, or similar item
//!   (d)(1)(F) customer-based intangible OR supplier-based
//!     intangible
//!   (d)(1)(G) license, permit, or other right granted by a
//!     governmental unit or agency
//!   (d)(1)(H) covenant not to compete entered into in connection
//!     with an acquisition of a trade or business
//!   (d)(1)(I) franchise, trademark, or trade name
//!
//! § 197(e) EXCEPTIONS — section 197 does NOT apply to:
//!   (e)(1) financial interests in corporations / partnerships /
//!     trusts / estates
//!   (e)(2) interests in land
//!   (e)(3) computer software (separate § 167 rules apply for
//!     readily available software; § 197 applies for software
//!     acquired in connection with a trade or business)
//!   (e)(4) interests under leases of tangible property
//!   (e)(5) interests under existing contracts for use of patents
//!     or copyrights
//!   (e)(6) interests under existing contracts (other than § 197
//!     intangibles)
//!   (e)(7) sports franchises (subject to § 197 since 2004 amendment
//!     under American Jobs Creation Act of 2004 § 886)
//!
//! § 197(f)(9) ANTI-CHURNING RULES — prevent amortization of
//! § 197 intangibles held or used during the transition period
//! (July 25, 1991 to August 10, 1993) by the taxpayer or a related
//! party. "Related person" includes a corporation and an individual
//! who owns, directly or indirectly, MORE THAN 20% of the
//! corporation's outstanding stock. Purpose: prevent taxpayers from
//! converting pre-1993 non-amortizable intangibles into post-1993
//! § 197-amortizable intangibles via related-party transfers.
//!
//! § 197(g) DEPRECIATION OR AMORTIZATION OF § 197 INTANGIBLES —
//! intangibles are AMORTIZED under § 197(a), not depreciated under
//! § 167. Amortization deduction is § 162-character (ordinary
//! business expense) for a trade or business.
//!
//! Citations: IRC § 197(a) (15-year SL amortization beginning month
//! acquired); § 197(b) (exclusive deduction pathway); § 197(c)
//! (definition — post-8/10/1993 acquisition + trade-or-business
//! use); § 197(d)(1)(A)-(I) (intangible categories); § 197(e)(1)-(7)
//! (exceptions); § 197(f)(9) (anti-churning rules); § 197(g)
//! (§ 167 depreciation barred); Treas. Reg. § 1.197-2 (final
//! regulations); Rev. Rul. 2004-49 (anti-churning application to
//! covenants not to compete); American Jobs Creation Act of 2004
//! § 886 (sports franchise inclusion).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IntangibleType {
    /// § 197(d)(1)(A)
    Goodwill,
    /// § 197(d)(1)(B)
    GoingConcernValue,
    /// § 197(d)(1)(C)
    WorkforceInPlace,
    /// § 197(d)(1)(D)
    BusinessBooksAndRecords,
    /// § 197(d)(1)(E)
    PatentOrCopyright,
    /// § 197(d)(1)(F)
    CustomerOrSupplierBased,
    /// § 197(d)(1)(G)
    GovernmentLicense,
    /// § 197(d)(1)(H)
    CovenantNotToCompete,
    /// § 197(d)(1)(I)
    FranchiseTrademarkTradeName,
    /// § 197(e)(2) — land excluded
    Land,
    /// § 197(e)(1) — financial interests excluded
    FinancialInterest,
    /// § 197(e)(4) — tangible property lease excluded
    LeaseOfTangibleProperty,
}

pub const AMORTIZATION_MONTHS: i64 = 180;

#[derive(Debug, Clone, Deserialize)]
pub struct Section197Input {
    pub intangible_type: IntangibleType,
    /// Adjusted basis at time of acquisition (cents).
    pub adjusted_basis_cents: i64,
    /// Months elapsed since the month the intangible was acquired.
    /// 0 = acquired this month; 180 = fully amortized.
    pub months_held_since_acquisition: u32,
    /// § 197(c)(2) — intangible must be held in connection with the
    /// conduct of a trade or business or § 212 activity.
    pub held_in_trade_or_business: bool,
    /// § 197(f)(9) anti-churning — was the intangible held or used
    /// during the transition period (July 25, 1991 - August 10,
    /// 1993) by the taxpayer or a related party? If true, § 197(a)
    /// amortization is barred.
    pub anti_churning_transition_period_held_by_related: bool,
    /// § 197(f)(9) anti-churning — acquired from a related party
    /// (> 20% ownership) AND transferring party continues to use
    /// the intangible? If both true, amortization barred.
    pub acquired_from_related_party_with_continued_use: bool,
    /// § 197(c)(1) effective date — was the intangible acquired
    /// after August 10, 1993?
    pub acquired_after_august_10_1993: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Section197Result {
    pub amortizable: bool,
    pub monthly_amortization_cents: i64,
    pub annual_amortization_cents: i64,
    pub cumulative_amortization_cents: i64,
    pub remaining_basis_cents: i64,
    pub citation: &'static str,
    pub notes: Vec<String>,
}

pub fn compute(input: &Section197Input) -> Section197Result {
    let mut notes: Vec<String> = Vec::new();

    if !input.held_in_trade_or_business {
        notes.push(
            "§ 197(c)(2) — intangible NOT held in connection with trade or business or § 212 activity; § 197 amortization unavailable"
                .to_string(),
        );
        return zero_amortization_result(input, notes, false);
    }

    if !input.acquired_after_august_10_1993 {
        notes.push(
            "§ 197(c)(1) — intangible acquired ON OR BEFORE August 10, 1993; § 197 amortization unavailable (pre-TRA-93 acquisition)"
                .to_string(),
        );
        return zero_amortization_result(input, notes, false);
    }

    if matches!(
        input.intangible_type,
        IntangibleType::Land
            | IntangibleType::FinancialInterest
            | IntangibleType::LeaseOfTangibleProperty
    ) {
        let exception_cite = match input.intangible_type {
            IntangibleType::Land => "§ 197(e)(2)",
            IntangibleType::FinancialInterest => "§ 197(e)(1)",
            IntangibleType::LeaseOfTangibleProperty => "§ 197(e)(4)",
            _ => unreachable!(),
        };
        notes.push(format!(
            "{} — intangible type excluded from § 197 amortization pathway",
            exception_cite
        ));
        return zero_amortization_result(input, notes, false);
    }

    if input.anti_churning_transition_period_held_by_related
        || input.acquired_from_related_party_with_continued_use
    {
        notes.push(
            "§ 197(f)(9) anti-churning — § 197 amortization barred when intangible was held by taxpayer or related (> 20%) party during 7/25/1991-8/10/1993 transition period OR acquired from related party with continued use"
                .to_string(),
        );
        return zero_amortization_result(input, notes, false);
    }

    let basis = input.adjusted_basis_cents.max(0);
    let monthly = basis / AMORTIZATION_MONTHS;
    let months_active = (input.months_held_since_acquisition as i64).min(AMORTIZATION_MONTHS);
    let cumulative = monthly.saturating_mul(months_active);
    let remaining = basis - cumulative;
    let annual = monthly.saturating_mul(12);

    notes.push(format!(
        "§ 197(a) — 15-year (180-month) straight-line amortization beginning month acquired; ${} / 180 = ${} monthly",
        basis, monthly
    ));

    if months_active >= AMORTIZATION_MONTHS {
        notes.push("180-month amortization period exhausted — basis fully amortized".to_string());
    }

    let category_note = category_description(input.intangible_type);
    notes.push(category_note.to_string());

    Section197Result {
        amortizable: true,
        monthly_amortization_cents: monthly,
        annual_amortization_cents: annual,
        cumulative_amortization_cents: cumulative,
        remaining_basis_cents: remaining,
        citation: citation(),
        notes,
    }
}

fn zero_amortization_result(
    input: &Section197Input,
    notes: Vec<String>,
    amortizable: bool,
) -> Section197Result {
    Section197Result {
        amortizable,
        monthly_amortization_cents: 0,
        annual_amortization_cents: 0,
        cumulative_amortization_cents: 0,
        remaining_basis_cents: input.adjusted_basis_cents.max(0),
        citation: citation(),
        notes,
    }
}

fn category_description(t: IntangibleType) -> &'static str {
    match t {
        IntangibleType::Goodwill => "§ 197(d)(1)(A) — goodwill",
        IntangibleType::GoingConcernValue => "§ 197(d)(1)(B) — going concern value",
        IntangibleType::WorkforceInPlace => "§ 197(d)(1)(C) — workforce in place",
        IntangibleType::BusinessBooksAndRecords => "§ 197(d)(1)(D) — business books and records / operating systems",
        IntangibleType::PatentOrCopyright => "§ 197(d)(1)(E) — patent / copyright / formula / process / design",
        IntangibleType::CustomerOrSupplierBased => "§ 197(d)(1)(F) — customer-based or supplier-based intangible",
        IntangibleType::GovernmentLicense => "§ 197(d)(1)(G) — governmental license / permit / right",
        IntangibleType::CovenantNotToCompete => "§ 197(d)(1)(H) — covenant not to compete entered in connection with trade or business acquisition",
        IntangibleType::FranchiseTrademarkTradeName => "§ 197(d)(1)(I) — franchise / trademark / trade name",
        IntangibleType::Land => "§ 197(e)(2) excluded — land",
        IntangibleType::FinancialInterest => "§ 197(e)(1) excluded — financial interest",
        IntangibleType::LeaseOfTangibleProperty => "§ 197(e)(4) excluded — lease of tangible property",
    }
}

fn citation() -> &'static str {
    "IRC § 197(a)/(b)/(c)/(d)(1)/(e)/(f)(9)/(g); Treas. Reg. § 1.197-2; Rev. Rul. 2004-49; American Jobs Creation Act of 2004 § 886"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base(intangible_type: IntangibleType, basis_dollars: i64, months: u32) -> Section197Input {
        Section197Input {
            intangible_type,
            adjusted_basis_cents: basis_dollars * 100,
            months_held_since_acquisition: months,
            held_in_trade_or_business: true,
            anti_churning_transition_period_held_by_related: false,
            acquired_from_related_party_with_continued_use: false,
            acquired_after_august_10_1993: true,
        }
    }

    #[test]
    fn goodwill_180000_basis_1000_monthly_amortization() {
        // $180,000 / 180 months = $1,000/month
        let r = compute(&base(IntangibleType::Goodwill, 180_000, 12));
        assert!(r.amortizable);
        assert_eq!(r.monthly_amortization_cents, 100_000);
        assert_eq!(r.annual_amortization_cents, 1_200_000);
        assert_eq!(r.cumulative_amortization_cents, 1_200_000);
        assert_eq!(r.remaining_basis_cents, 180_000_00 - 1_200_000);
    }

    #[test]
    fn customer_list_15_years_fully_amortized() {
        let r = compute(&base(IntangibleType::CustomerOrSupplierBased, 180_000, 180));
        assert!(r.amortizable);
        assert_eq!(r.cumulative_amortization_cents, 180_000_00);
        assert_eq!(r.remaining_basis_cents, 0);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("180-month amortization period exhausted")));
    }

    #[test]
    fn workforce_in_place_first_month_amortization() {
        let r = compute(&base(IntangibleType::WorkforceInPlace, 360_000, 1));
        assert!(r.amortizable);
        assert_eq!(r.monthly_amortization_cents, 200_000);
        assert_eq!(r.cumulative_amortization_cents, 200_000);
    }

    #[test]
    fn covenant_not_to_compete_categorized_as_section_197_intangible() {
        let r = compute(&base(IntangibleType::CovenantNotToCompete, 90_000, 12));
        assert!(r.amortizable);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 197(d)(1)(H) — covenant not to compete")));
    }

    #[test]
    fn franchise_trademark_trade_name_amortizable() {
        let r = compute(&base(
            IntangibleType::FranchiseTrademarkTradeName,
            540_000,
            60,
        ));
        assert!(r.amortizable);
        assert_eq!(r.monthly_amortization_cents, 300_000);
    }

    #[test]
    fn government_license_amortizable() {
        let r = compute(&base(IntangibleType::GovernmentLicense, 180_000, 24));
        assert!(r.amortizable);
        assert!(r.notes.iter().any(|n| n.contains("§ 197(d)(1)(G)")));
    }

    #[test]
    fn going_concern_value_amortizable() {
        let r = compute(&base(IntangibleType::GoingConcernValue, 180_000, 36));
        assert!(r.amortizable);
        assert!(r.notes.iter().any(|n| n.contains("§ 197(d)(1)(B)")));
    }

    #[test]
    fn business_books_and_records_amortizable() {
        let r = compute(&base(IntangibleType::BusinessBooksAndRecords, 36_000, 12));
        assert!(r.amortizable);
        assert!(r.notes.iter().any(|n| n.contains("§ 197(d)(1)(D)")));
    }

    #[test]
    fn patent_or_copyright_amortizable() {
        let r = compute(&base(IntangibleType::PatentOrCopyright, 360_000, 60));
        assert!(r.amortizable);
        assert!(r.notes.iter().any(|n| n.contains("§ 197(d)(1)(E)")));
    }

    #[test]
    fn land_excluded_no_amortization() {
        let r = compute(&base(IntangibleType::Land, 1_000_000, 60));
        assert!(!r.amortizable);
        assert_eq!(r.monthly_amortization_cents, 0);
        assert_eq!(r.remaining_basis_cents, 1_000_000_00);
        assert!(r.notes.iter().any(|n| n.contains("§ 197(e)(2)")));
    }

    #[test]
    fn financial_interest_excluded_no_amortization() {
        let r = compute(&base(IntangibleType::FinancialInterest, 100_000, 60));
        assert!(!r.amortizable);
        assert!(r.notes.iter().any(|n| n.contains("§ 197(e)(1)")));
    }

    #[test]
    fn lease_of_tangible_property_excluded() {
        let r = compute(&base(IntangibleType::LeaseOfTangibleProperty, 100_000, 60));
        assert!(!r.amortizable);
        assert!(r.notes.iter().any(|n| n.contains("§ 197(e)(4)")));
    }

    #[test]
    fn not_held_in_trade_or_business_no_amortization() {
        let mut i = base(IntangibleType::Goodwill, 180_000, 12);
        i.held_in_trade_or_business = false;
        let r = compute(&i);
        assert!(!r.amortizable);
        assert!(r.notes.iter().any(|n| n.contains("§ 197(c)(2)")));
    }

    #[test]
    fn pre_august_10_1993_acquisition_no_amortization() {
        let mut i = base(IntangibleType::Goodwill, 180_000, 12);
        i.acquired_after_august_10_1993 = false;
        let r = compute(&i);
        assert!(!r.amortizable);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 197(c)(1)") && n.contains("August 10, 1993")));
    }

    #[test]
    fn anti_churning_transition_period_blocks_amortization() {
        let mut i = base(IntangibleType::Goodwill, 180_000, 12);
        i.anti_churning_transition_period_held_by_related = true;
        let r = compute(&i);
        assert!(!r.amortizable);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 197(f)(9) anti-churning")));
    }

    #[test]
    fn anti_churning_related_party_continued_use_blocks_amortization() {
        let mut i = base(IntangibleType::Goodwill, 180_000, 12);
        i.acquired_from_related_party_with_continued_use = true;
        let r = compute(&i);
        assert!(!r.amortizable);
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("§ 197(f)(9) anti-churning")));
    }

    #[test]
    fn negative_basis_clamped_to_zero() {
        let mut i = base(IntangibleType::Goodwill, 0, 12);
        i.adjusted_basis_cents = -100_000;
        let r = compute(&i);
        assert_eq!(r.monthly_amortization_cents, 0);
        assert_eq!(r.remaining_basis_cents, 0);
    }

    #[test]
    fn months_exceed_180_caps_cumulative_at_basis() {
        let r = compute(&base(IntangibleType::Goodwill, 180_000, 300));
        assert_eq!(r.cumulative_amortization_cents, 180_000_00);
        assert_eq!(r.remaining_basis_cents, 0);
    }

    #[test]
    fn citation_pins_subsections_and_treasury_regs() {
        let r = compute(&base(IntangibleType::Goodwill, 180_000, 12));
        assert!(r.citation.contains("§ 197(a)"));
        assert!(r.citation.contains("(b)"));
        assert!(r.citation.contains("(c)"));
        assert!(r.citation.contains("(d)(1)"));
        assert!(r.citation.contains("(e)"));
        assert!(r.citation.contains("(f)(9)"));
        assert!(r.citation.contains("(g)"));
        assert!(r.citation.contains("§ 1.197-2"));
        assert!(r.citation.contains("Rev. Rul. 2004-49"));
        assert!(r.citation.contains("American Jobs Creation Act of 2004"));
    }

    #[test]
    fn amortization_months_constant_pins_180() {
        assert_eq!(AMORTIZATION_MONTHS, 180);
    }

    #[test]
    fn annual_amortization_equals_monthly_times_twelve() {
        let r = compute(&base(IntangibleType::Goodwill, 360_000, 12));
        assert_eq!(
            r.annual_amortization_cents,
            r.monthly_amortization_cents * 12
        );
    }

    #[test]
    fn note_describes_180_month_straight_line() {
        let r = compute(&base(IntangibleType::Goodwill, 180_000, 12));
        assert!(r
            .notes
            .iter()
            .any(|n| n.contains("15-year") && n.contains("(180-month)")));
    }

    #[test]
    fn nine_intangible_categories_d1_a_to_i_all_amortizable() {
        let categories = [
            IntangibleType::Goodwill,
            IntangibleType::GoingConcernValue,
            IntangibleType::WorkforceInPlace,
            IntangibleType::BusinessBooksAndRecords,
            IntangibleType::PatentOrCopyright,
            IntangibleType::CustomerOrSupplierBased,
            IntangibleType::GovernmentLicense,
            IntangibleType::CovenantNotToCompete,
            IntangibleType::FranchiseTrademarkTradeName,
        ];
        for cat in categories {
            let r = compute(&base(cat, 180_000, 12));
            assert!(r.amortizable, "category {:?} should be amortizable", cat);
        }
    }

    #[test]
    fn three_exception_categories_e_all_non_amortizable() {
        let exceptions = [
            IntangibleType::Land,
            IntangibleType::FinancialInterest,
            IntangibleType::LeaseOfTangibleProperty,
        ];
        for cat in exceptions {
            let r = compute(&base(cat, 180_000, 12));
            assert!(
                !r.amortizable,
                "exception {:?} should NOT be amortizable",
                cat
            );
        }
    }

    #[test]
    fn cumulative_at_60_months_one_third_of_basis() {
        let r = compute(&base(IntangibleType::Goodwill, 180_000, 60));
        let expected = (180_000_00i64 / 180) * 60;
        assert_eq!(r.cumulative_amortization_cents, expected);
    }
}
