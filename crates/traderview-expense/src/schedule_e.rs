//! Schedule E Part I — Rental Real Estate income/loss roll-up.
//!
//! Aggregates the per-property `rental_income` / `rental_expenses` /
//! `rental_mileage` rows into the 22 line items that appear on Form 1040
//! Schedule E Part I, so the output maps 1:1 onto what the user (or tax
//! prep software) types onto the actual form.
//!
//! Line numbers in the comments below match the 2024 Schedule E form.
//!
//! Pure compute. Reads pre-fetched rows and a precomputed depreciation
//! number from `depreciation.rs`. Does NOT touch the database, does NOT
//! filter by tax year — callers slice rows to the year before calling.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Line 1b property type per Form 1040 Schedule E instructions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyType {
    SingleFamily,        // IRS code 1
    MultiFamily,         // IRS code 2
    VacationShortTerm,   // IRS code 3
    Commercial,          // IRS code 4
    Land,                // IRS code 5
    Royalties,           // IRS code 6
    SelfRental,          // IRS code 7
    Other,               // IRS code 8
}

impl PropertyType {
    pub fn irs_code(self) -> u8 {
        match self {
            PropertyType::SingleFamily      => 1,
            PropertyType::MultiFamily       => 2,
            PropertyType::VacationShortTerm => 3,
            PropertyType::Commercial        => 4,
            PropertyType::Land              => 5,
            PropertyType::Royalties         => 6,
            PropertyType::SelfRental        => 7,
            PropertyType::Other             => 8,
        }
    }
}

/// Stable category codes mirroring `schedule_e_categories.code` in 0032.
/// One entry per Schedule E expense line. New codes go on the end.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleECategory {
    Advertising,        // line 5
    AutoTravel,         // line 6
    CleaningMaint,      // line 7
    Commissions,        // line 8
    Insurance,          // line 9
    LegalProf,          // line 10
    MgmtFees,           // line 11
    MortgageInterest,   // line 12
    OtherInterest,      // line 13
    Repairs,            // line 14
    Supplies,           // line 15
    Taxes,              // line 16
    Utilities,          // line 17
    Depreciation,       // line 18 — usually computed, not entered as a raw expense
    Other,              // line 19
}

impl ScheduleECategory {
    pub fn line(self) -> &'static str {
        match self {
            ScheduleECategory::Advertising      => "5",
            ScheduleECategory::AutoTravel       => "6",
            ScheduleECategory::CleaningMaint    => "7",
            ScheduleECategory::Commissions      => "8",
            ScheduleECategory::Insurance        => "9",
            ScheduleECategory::LegalProf        => "10",
            ScheduleECategory::MgmtFees         => "11",
            ScheduleECategory::MortgageInterest => "12",
            ScheduleECategory::OtherInterest    => "13",
            ScheduleECategory::Repairs          => "14",
            ScheduleECategory::Supplies         => "15",
            ScheduleECategory::Taxes            => "16",
            ScheduleECategory::Utilities        => "17",
            ScheduleECategory::Depreciation     => "18",
            ScheduleECategory::Other            => "19",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "e_advertising"                                 => Some(Self::Advertising),
            "e_auto_travel"                                 => Some(Self::AutoTravel),
            "e_cleaning_maint"                              => Some(Self::CleaningMaint),
            "e_commissions"                                 => Some(Self::Commissions),
            "e_insurance"                                   => Some(Self::Insurance),
            "e_legal_prof"                                  => Some(Self::LegalProf),
            "e_mgmt_fees"                                   => Some(Self::MgmtFees),
            "e_mortgage_interest"                           => Some(Self::MortgageInterest),
            "e_other_interest"                              => Some(Self::OtherInterest),
            "e_repairs"                                     => Some(Self::Repairs),
            "e_supplies"                                    => Some(Self::Supplies),
            "e_taxes"                                       => Some(Self::Taxes),
            "e_utilities"                                   => Some(Self::Utilities),
            "e_depreciation"                                => Some(Self::Depreciation),
            "e_other"      | "e_hoa"         | "e_landscaping"
            | "e_pest_control" | "e_permit_license" | "e_appliance"
            | "e_software" | "e_bank_fee"    | "e_eviction"
            | "e_security"                                   => Some(Self::Other),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomeRow {
    /// Schedule E income kinds — rent (line 3a) vs royalty (line 3b) vs
    /// "Other" miscellany (late fees, deposit forfeitures, parking).
    pub kind: IncomeKind,
    pub amount: Decimal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncomeKind {
    Rent,
    Royalty,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseRow {
    pub category: ScheduleECategory,
    pub amount: Decimal,
    /// Improvement / capitalized cost — excluded from Schedule E line totals
    /// (recovered via depreciation instead, per Reg. §1.263(a)-3).
    pub is_capitalized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MileageRow {
    pub miles: Decimal,
    pub rate_per_mile: Decimal,
}

/// Final per-property totals corresponding to Schedule E Part I lines.
/// Fields are named after the line number (lXX) so they read as the form
/// in code reviews.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScheduleELine {
    pub property_id: String,
    pub property_type_code: u8,        // line 1b
    pub fair_rental_days: u32,         // line 2 (column A/B/C)
    pub personal_use_days: u32,        // line 2

    pub l3a_rents_received: Decimal,
    pub l3b_royalties: Decimal,
    pub l5_advertising: Decimal,
    pub l6_auto_travel: Decimal,
    pub l7_cleaning_maint: Decimal,
    pub l8_commissions: Decimal,
    pub l9_insurance: Decimal,
    pub l10_legal_prof: Decimal,
    pub l11_mgmt_fees: Decimal,
    pub l12_mortgage_interest: Decimal,
    pub l13_other_interest: Decimal,
    pub l14_repairs: Decimal,
    pub l15_supplies: Decimal,
    pub l16_taxes: Decimal,
    pub l17_utilities: Decimal,
    pub l18_depreciation: Decimal,
    pub l19_other: Decimal,

    pub l20_total_expenses: Decimal,   // sum 5-19
    pub l21_income_or_loss: Decimal,   // 3a + 3b - 20
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScheduleEReport {
    pub properties: Vec<ScheduleELine>,
    pub l23a_total_rents: Decimal,            // sum of all l3a
    pub l23b_total_royalties: Decimal,        // sum of all l3b
    pub l23c_total_mortgage_interest: Decimal,// sum of all l12
    pub l23d_total_depreciation: Decimal,     // sum of all l18
    pub l23e_total_expenses: Decimal,         // sum of all l20
    pub l24_income_total: Decimal,            // sum of positive l21
    pub l25_loss_total: Decimal,              // sum of negative l21 (as positive)
    pub l26_total_real_estate_income: Decimal,// l24 - l25
}

#[derive(Debug, Clone)]
pub struct PropertyInput<'a> {
    pub property_id: &'a str,
    pub property_type: PropertyType,
    pub fair_rental_days: u32,
    pub personal_use_days: u32,
    pub income: &'a [IncomeRow],
    pub expenses: &'a [ExpenseRow],
    pub mileage: &'a [MileageRow],
    /// Precomputed via `depreciation::macrs_residential` (or commercial
    /// 39yr SL) for THIS tax year. Passed in to keep schedule_e a pure
    /// roll-up — the depreciation algorithm has its own module.
    pub depreciation_for_year: Decimal,
}

/// Roll one property's rows into the Schedule E line items.
pub fn roll_property(p: &PropertyInput<'_>) -> ScheduleELine {
    let mut out = ScheduleELine {
        property_id: p.property_id.to_string(),
        property_type_code: p.property_type.irs_code(),
        fair_rental_days: p.fair_rental_days,
        personal_use_days: p.personal_use_days,
        ..ScheduleELine::default()
    };

    for r in p.income {
        match r.kind {
            IncomeKind::Rent    => out.l3a_rents_received += r.amount,
            IncomeKind::Royalty => out.l3b_royalties     += r.amount,
            IncomeKind::Other   => out.l19_other         += r.amount, // miscellaneous receipts
        }
    }

    for r in p.expenses {
        if r.is_capitalized {
            continue;
        }
        let slot = match r.category {
            ScheduleECategory::Advertising      => &mut out.l5_advertising,
            ScheduleECategory::AutoTravel       => &mut out.l6_auto_travel,
            ScheduleECategory::CleaningMaint    => &mut out.l7_cleaning_maint,
            ScheduleECategory::Commissions      => &mut out.l8_commissions,
            ScheduleECategory::Insurance        => &mut out.l9_insurance,
            ScheduleECategory::LegalProf        => &mut out.l10_legal_prof,
            ScheduleECategory::MgmtFees         => &mut out.l11_mgmt_fees,
            ScheduleECategory::MortgageInterest => &mut out.l12_mortgage_interest,
            ScheduleECategory::OtherInterest    => &mut out.l13_other_interest,
            ScheduleECategory::Repairs          => &mut out.l14_repairs,
            ScheduleECategory::Supplies         => &mut out.l15_supplies,
            ScheduleECategory::Taxes            => &mut out.l16_taxes,
            ScheduleECategory::Utilities        => &mut out.l17_utilities,
            ScheduleECategory::Depreciation     => &mut out.l18_depreciation,
            ScheduleECategory::Other            => &mut out.l19_other,
        };
        *slot += r.amount;
    }

    for m in p.mileage {
        out.l6_auto_travel += m.miles * m.rate_per_mile;
    }

    out.l18_depreciation += p.depreciation_for_year;

    out.l20_total_expenses = out.l5_advertising
        + out.l6_auto_travel
        + out.l7_cleaning_maint
        + out.l8_commissions
        + out.l9_insurance
        + out.l10_legal_prof
        + out.l11_mgmt_fees
        + out.l12_mortgage_interest
        + out.l13_other_interest
        + out.l14_repairs
        + out.l15_supplies
        + out.l16_taxes
        + out.l17_utilities
        + out.l18_depreciation
        + out.l19_other;

    out.l21_income_or_loss = out.l3a_rents_received + out.l3b_royalties - out.l20_total_expenses;
    out
}

/// Roll all properties into the form's totals section (lines 23-26).
pub fn roll_report(lines: Vec<ScheduleELine>) -> ScheduleEReport {
    let mut r = ScheduleEReport {
        properties: lines,
        ..ScheduleEReport::default()
    };
    let zero = Decimal::ZERO;
    for p in &r.properties {
        r.l23a_total_rents             += p.l3a_rents_received;
        r.l23b_total_royalties         += p.l3b_royalties;
        r.l23c_total_mortgage_interest += p.l12_mortgage_interest;
        r.l23d_total_depreciation      += p.l18_depreciation;
        r.l23e_total_expenses          += p.l20_total_expenses;
        if p.l21_income_or_loss >= zero {
            r.l24_income_total += p.l21_income_or_loss;
        } else {
            r.l25_loss_total -= p.l21_income_or_loss; // negative → positive magnitude
        }
    }
    r.l26_total_real_estate_income = r.l24_income_total - r.l25_loss_total;
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn rent(n: Decimal) -> IncomeRow {
        IncomeRow { kind: IncomeKind::Rent, amount: n }
    }
    fn exp(cat: ScheduleECategory, amt: Decimal) -> ExpenseRow {
        ExpenseRow { category: cat, amount: amt, is_capitalized: false }
    }
    fn cap(cat: ScheduleECategory, amt: Decimal) -> ExpenseRow {
        ExpenseRow { category: cat, amount: amt, is_capitalized: true }
    }

    #[test]
    fn single_property_breakeven_after_depreciation() {
        // 12 months × $2,000 rent = $24,000.
        // $4k repairs + $3k insurance + $5k taxes + $8k mortgage interest = $20k.
        // $4,000 depreciation pushes Schedule E to $0.
        let income = vec![rent(dec!(24000))];
        let expenses = vec![
            exp(ScheduleECategory::Repairs,          dec!(4000)),
            exp(ScheduleECategory::Insurance,        dec!(3000)),
            exp(ScheduleECategory::Taxes,            dec!(5000)),
            exp(ScheduleECategory::MortgageInterest, dec!(8000)),
        ];
        let p = PropertyInput {
            property_id: "p1",
            property_type: PropertyType::SingleFamily,
            fair_rental_days: 365,
            personal_use_days: 0,
            income: &income,
            expenses: &expenses,
            mileage: &[],
            depreciation_for_year: dec!(4000),
        };
        let line = roll_property(&p);
        assert_eq!(line.l3a_rents_received, dec!(24000));
        assert_eq!(line.l20_total_expenses, dec!(24000));
        assert_eq!(line.l21_income_or_loss, dec!(0));
        assert_eq!(line.property_type_code, 1);
    }

    #[test]
    fn capitalized_improvement_does_not_hit_line_14() {
        // A $15k roof replacement is capitalized — it should NOT flow into
        // line 14 (Repairs). The IRS gets it back via depreciation.
        let income = vec![rent(dec!(12000))];
        let expenses = vec![
            exp(ScheduleECategory::Repairs, dec!(500)),    // ordinary repair
            cap(ScheduleECategory::Repairs, dec!(15000)),  // improvement
        ];
        let p = PropertyInput {
            property_id: "p1",
            property_type: PropertyType::SingleFamily,
            fair_rental_days: 365,
            personal_use_days: 0,
            income: &income,
            expenses: &expenses,
            mileage: &[],
            depreciation_for_year: Decimal::ZERO,
        };
        let line = roll_property(&p);
        assert_eq!(line.l14_repairs, dec!(500), "capitalized cost must be excluded from line 14");
        assert_eq!(line.l20_total_expenses, dec!(500));
    }

    #[test]
    fn mileage_folds_into_line_6_auto_travel() {
        let income = vec![rent(dec!(0))];
        let mileage = vec![MileageRow { miles: dec!(1000), rate_per_mile: dec!(0.67) }];
        let p = PropertyInput {
            property_id: "p1",
            property_type: PropertyType::SingleFamily,
            fair_rental_days: 365,
            personal_use_days: 0,
            income: &income,
            expenses: &[],
            mileage: &mileage,
            depreciation_for_year: Decimal::ZERO,
        };
        let line = roll_property(&p);
        assert_eq!(line.l6_auto_travel, dec!(670));
    }

    #[test]
    fn report_splits_winners_and_losers_into_l24_l25() {
        // p1 profitable +$3k, p2 losing -$2k → l24=3000, l25=2000, l26=1000.
        let p1_income = vec![rent(dec!(3000))];
        let p2_income = vec![rent(dec!(0))];
        let p2_expenses = vec![exp(ScheduleECategory::Repairs, dec!(2000))];
        let l1 = roll_property(&PropertyInput {
            property_id: "p1",
            property_type: PropertyType::SingleFamily,
            fair_rental_days: 365,
            personal_use_days: 0,
            income: &p1_income,
            expenses: &[],
            mileage: &[],
            depreciation_for_year: Decimal::ZERO,
        });
        let l2 = roll_property(&PropertyInput {
            property_id: "p2",
            property_type: PropertyType::Commercial,
            fair_rental_days: 365,
            personal_use_days: 0,
            income: &p2_income,
            expenses: &p2_expenses,
            mileage: &[],
            depreciation_for_year: Decimal::ZERO,
        });
        let r = roll_report(vec![l1, l2]);
        assert_eq!(r.l24_income_total, dec!(3000));
        assert_eq!(r.l25_loss_total, dec!(2000));
        assert_eq!(r.l26_total_real_estate_income, dec!(1000));
    }

    #[test]
    fn category_code_round_trip_matches_migration() {
        // Codes here must mirror schedule_e_categories.code in 0032.sql —
        // a typo means the SQL roll-up will mis-bucket expenses.
        assert_eq!(ScheduleECategory::from_code("e_repairs"),   Some(ScheduleECategory::Repairs));
        assert_eq!(ScheduleECategory::from_code("e_utilities"), Some(ScheduleECategory::Utilities));
        assert_eq!(ScheduleECategory::from_code("e_mortgage_interest"), Some(ScheduleECategory::MortgageInterest));
        // All line-19 detail codes collapse to ::Other.
        assert_eq!(ScheduleECategory::from_code("e_hoa"),       Some(ScheduleECategory::Other));
        assert_eq!(ScheduleECategory::from_code("e_landscaping"), Some(ScheduleECategory::Other));
        assert_eq!(ScheduleECategory::from_code("e_eviction"),  Some(ScheduleECategory::Other));
        assert_eq!(ScheduleECategory::from_code("not_a_code"),  None);
    }

    #[test]
    fn property_type_codes_match_irs_1_through_8() {
        assert_eq!(PropertyType::SingleFamily.irs_code(),      1);
        assert_eq!(PropertyType::MultiFamily.irs_code(),       2);
        assert_eq!(PropertyType::VacationShortTerm.irs_code(), 3);
        assert_eq!(PropertyType::Commercial.irs_code(),        4);
        assert_eq!(PropertyType::Land.irs_code(),              5);
        assert_eq!(PropertyType::Royalties.irs_code(),         6);
        assert_eq!(PropertyType::SelfRental.irs_code(),        7);
        assert_eq!(PropertyType::Other.irs_code(),             8);
    }
}
