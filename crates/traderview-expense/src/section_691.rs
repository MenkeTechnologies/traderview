//! IRC §691 — Recipients of income in respect of decedents (IRD).
//!
//! Pairs with **`section_1014`** (stepped-up basis at death). Where
//! §1014(a) wipes out embedded gains at death, §1014(c) explicitly
//! denies the step-up for IRD assets — and §691 governs the income-tax
//! consequences for the heir who receives that IRD.
//!
//! **§691(a)** — IRD is included in the gross income of the recipient
//! (heir / estate / beneficiary) in the year received. Character is
//! preserved — IRA distributions are ordinary income, installment-sale
//! gain is capital, accrued bond interest is ordinary, etc.
//!
//! **§691(c)** — to mitigate the "double tax" of estate tax PLUS
//! income tax on the same dollar, the recipient gets an itemized
//! deduction (above-the-2%-AGI-floor) equal to the federal estate tax
//! attributable to the IRD share. Two-step computation per Treas. Reg.
//! § 1.691(c)-1(a)(2):
//!
//!   1. Compute the decedent's federal estate tax twice — once
//!      INCLUDING the IRD assets (actual), once EXCLUDING them.
//!   2. The difference is the estate tax attributable to IRD.
//!   3. Each recipient's deduction = their share of total IRD ×
//!      estate tax attributable to total IRD.
//!
//! The deduction is **per-recipient pro-rata** — if two beneficiaries
//! each receive half a $1M IRA, they each get half of the §691(c)
//! deduction.
//!
//! **Common IRD items** (preserved character):
//!   - Traditional IRA / 401(k) / qualified plan distributions (ord.)
//!   - Accrued bond interest (ord.)
//!   - Accrued royalties (ord.)
//!   - Installment-sale gain not yet recognized (cap.)
//!   - Deferred compensation (ord.)
//!   - Accrued but unpaid salary / commissions (ord.)
//!   - Renewal commissions on insurance contracts (ord.)
//!
//! Caller passes the federal estate tax attributable to total IRD as
//! pre-computed (Treas. Reg. § 1.691(c)-1 two-step). Module applies the
//! pro-rata allocation and reports the recipient's net taxable income.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IrdType {
    TraditionalIraDistribution,
    QualifiedPlanDistribution,
    AccruedBondInterest,
    AccruedRoyalties,
    InstallmentSaleGainNotYetRecognized,
    DeferredCompensation,
    AccruedSalaryOrCommissions,
    RenewalCommissionsLifeInsurance,
    Other,
}

impl IrdType {
    /// True if the IRD item is ordinary in character (the common case);
    /// false for the installment-sale gain path which is capital.
    pub fn is_ordinary_character(&self) -> bool {
        !matches!(self, IrdType::InstallmentSaleGainNotYetRecognized)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section691Input {
    pub ird_received_by_heir: Decimal,
    /// Total IRD across all beneficiaries / heirs of the decedent.
    pub total_ird_in_estate: Decimal,
    /// Federal estate tax attributable to IRD per Treas. Reg.
    /// § 1.691(c)-1(a)(2) two-step computation. Caller pre-computes by
    /// running the estate tax twice (with and without IRD).
    pub federal_estate_tax_attributable_to_total_ird: Decimal,
    pub ird_type: IrdType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section691Result {
    /// Full IRD amount the heir reports as gross income (character per
    /// `ird_type`).
    pub includible_gross_income: Decimal,
    /// Character of the income (ordinary or capital).
    pub character_is_ordinary: bool,
    /// Heir's share of the §691(c) deduction (pro-rata by IRD share).
    pub section_691c_deduction: Decimal,
    /// Net taxable amount after the §691(c) deduction.
    pub net_taxable_after_deduction: Decimal,
    /// Effective tax-rate relief = §691(c) deduction / IRD received.
    /// Demonstrates the proportion of "estate tax money" that comes
    /// back via the income-tax deduction.
    pub effective_relief_ratio: Decimal,
    pub note: String,
}

pub fn compute(input: &Section691Input) -> Section691Result {
    let includible = input.ird_received_by_heir;
    let character_ordinary = input.ird_type.is_ordinary_character();

    // Heir's share of total IRD (guard division by zero).
    let heir_share = if input.total_ird_in_estate > Decimal::ZERO {
        input.ird_received_by_heir / input.total_ird_in_estate
    } else {
        Decimal::ZERO
    };

    // §691(c) deduction = pro-rata share of estate tax attributable to IRD.
    let deduction = heir_share * input.federal_estate_tax_attributable_to_total_ird;

    let net_taxable = (includible - deduction).max(Decimal::ZERO);

    let relief_ratio = if includible > Decimal::ZERO {
        deduction / includible
    } else {
        Decimal::ZERO
    };

    let type_label = match input.ird_type {
        IrdType::TraditionalIraDistribution => "traditional IRA distribution",
        IrdType::QualifiedPlanDistribution => "qualified plan distribution",
        IrdType::AccruedBondInterest => "accrued bond interest",
        IrdType::AccruedRoyalties => "accrued royalties",
        IrdType::InstallmentSaleGainNotYetRecognized => {
            "installment sale gain (capital character preserved)"
        }
        IrdType::DeferredCompensation => "deferred compensation",
        IrdType::AccruedSalaryOrCommissions => "accrued salary/commissions",
        IrdType::RenewalCommissionsLifeInsurance => "life insurance renewal commissions",
        IrdType::Other => "other IRD",
    };

    let note = if deduction > Decimal::ZERO {
        format!(
            "§691(a) IRD: ${} {} income; §691(c) deduction ${} (heir share {} of total IRD × ${} estate tax); net taxable ${}",
            includible.round_dp(2),
            type_label,
            deduction.round_dp(2),
            heir_share.round_dp(4),
            input
                .federal_estate_tax_attributable_to_total_ird
                .round_dp(2),
            net_taxable.round_dp(2),
        )
    } else {
        format!(
            "§691(a) IRD: ${} {} income; no §691(c) deduction (no federal estate tax attributable or no IRD in estate)",
            includible.round_dp(2),
            type_label,
        )
    };

    Section691Result {
        includible_gross_income: includible,
        character_is_ordinary: character_ordinary,
        section_691c_deduction: deduction,
        net_taxable_after_deduction: net_taxable,
        effective_relief_ratio: relief_ratio,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    fn base() -> Section691Input {
        Section691Input {
            ird_received_by_heir: dec!(500_000),
            total_ird_in_estate: dec!(2_000_000),
            federal_estate_tax_attributable_to_total_ird: dec!(800_000),
            ird_type: IrdType::TraditionalIraDistribution,
        }
    }

    #[test]
    fn canonical_kitces_example_pro_rata_deduction() {
        // From Kitces / Treas. Reg. example: $1M IRA, 2 beneficiaries
        // each get half; estate tax attributable to IRD = $400k → each
        // gets $200k §691(c) deduction.
        let mut i = base();
        i.ird_received_by_heir = dec!(500_000);
        i.total_ird_in_estate = dec!(1_000_000);
        i.federal_estate_tax_attributable_to_total_ird = dec!(400_000);
        let r = compute(&i);
        assert_eq!(r.section_691c_deduction, dec!(200_000));
        assert_eq!(r.net_taxable_after_deduction, dec!(300_000));
        assert_eq!(r.effective_relief_ratio, dec!(0.4));
    }

    #[test]
    fn full_ird_recipient_gets_full_deduction() {
        // Heir receives all $2M of IRD → $800k deduction.
        let mut i = base();
        i.ird_received_by_heir = dec!(2_000_000);
        let r = compute(&i);
        assert_eq!(r.section_691c_deduction, dec!(800_000));
        assert_eq!(r.net_taxable_after_deduction, dec!(1_200_000));
    }

    #[test]
    fn partial_share_proportional_deduction() {
        // Heir gets 25% of IRD → 25% of attributable estate tax.
        let mut i = base();
        i.ird_received_by_heir = dec!(500_000); // 25% of $2M
        let r = compute(&i);
        assert_eq!(r.section_691c_deduction, dec!(200_000)); // 25% × $800k
        assert_eq!(r.net_taxable_after_deduction, dec!(300_000));
    }

    #[test]
    fn zero_estate_tax_attributable_no_deduction() {
        // Estate below filing threshold (no federal estate tax due) →
        // no §691(c) deduction available. All IRD is fully taxable.
        let mut i = base();
        i.federal_estate_tax_attributable_to_total_ird = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.section_691c_deduction, Decimal::ZERO);
        assert_eq!(r.net_taxable_after_deduction, dec!(500_000));
        assert!(r.note.contains("no §691(c) deduction"));
    }

    #[test]
    fn zero_total_ird_no_deduction_no_panic() {
        // Pathological: total IRD is zero (somehow heir received IRD
        // but tot reports zero). Division-by-zero guard returns 0
        // deduction.
        let mut i = base();
        i.total_ird_in_estate = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.section_691c_deduction, Decimal::ZERO);
    }

    #[test]
    fn traditional_ira_is_ordinary_character() {
        let r = compute(&base());
        assert!(r.character_is_ordinary);
        assert!(r.note.contains("traditional IRA"));
    }

    #[test]
    fn installment_sale_gain_preserves_capital_character() {
        // §691(a) preserves character. Installment sale → capital.
        let mut i = base();
        i.ird_type = IrdType::InstallmentSaleGainNotYetRecognized;
        let r = compute(&i);
        assert!(!r.character_is_ordinary);
        assert!(r.note.contains("capital character"));
    }

    #[test]
    fn deferred_comp_is_ordinary_character() {
        let mut i = base();
        i.ird_type = IrdType::DeferredCompensation;
        let r = compute(&i);
        assert!(r.character_is_ordinary);
    }

    #[test]
    fn accrued_bond_interest_is_ordinary() {
        let mut i = base();
        i.ird_type = IrdType::AccruedBondInterest;
        let r = compute(&i);
        assert!(r.character_is_ordinary);
    }

    #[test]
    fn renewal_commissions_is_ordinary() {
        let mut i = base();
        i.ird_type = IrdType::RenewalCommissionsLifeInsurance;
        let r = compute(&i);
        assert!(r.character_is_ordinary);
    }

    #[test]
    fn deduction_never_exceeds_includible() {
        // Even pathological inputs (huge estate tax, small IRD share)
        // shouldn't produce negative net taxable. The compute clamps at
        // zero.
        let mut i = base();
        i.ird_received_by_heir = dec!(100);
        i.total_ird_in_estate = dec!(100);
        i.federal_estate_tax_attributable_to_total_ird = dec!(1_000_000);
        let r = compute(&i);
        assert_eq!(r.section_691c_deduction, dec!(1_000_000));
        assert_eq!(r.net_taxable_after_deduction, Decimal::ZERO);
    }

    #[test]
    fn effective_relief_ratio_50_percent_example() {
        // $500k IRD with $250k effective deduction → 50% relief ratio.
        let mut i = base();
        i.ird_received_by_heir = dec!(500_000);
        i.total_ird_in_estate = dec!(1_000_000);
        i.federal_estate_tax_attributable_to_total_ird = dec!(500_000);
        let r = compute(&i);
        assert_eq!(r.section_691c_deduction, dec!(250_000));
        assert_eq!(r.effective_relief_ratio, dec!(0.5));
    }

    #[test]
    fn zero_ird_received_no_income_no_deduction() {
        let mut i = base();
        i.ird_received_by_heir = Decimal::ZERO;
        let r = compute(&i);
        assert_eq!(r.includible_gross_income, Decimal::ZERO);
        assert_eq!(r.section_691c_deduction, Decimal::ZERO);
        assert_eq!(r.effective_relief_ratio, Decimal::ZERO);
    }

    #[test]
    fn very_large_ira_no_precision_loss() {
        // $50M IRA, $20M total IRD, $8M estate tax. Heir receives
        // entire $50M (treating each heir's IRD share with appropriate
        // total). Math stays exact.
        let mut i = base();
        i.ird_received_by_heir = dec!(50_000_000);
        i.total_ird_in_estate = dec!(50_000_000);
        i.federal_estate_tax_attributable_to_total_ird = dec!(20_000_000);
        let r = compute(&i);
        assert_eq!(r.section_691c_deduction, dec!(20_000_000));
        assert_eq!(r.net_taxable_after_deduction, dec!(30_000_000));
    }

    #[test]
    fn three_beneficiary_equal_split_each_gets_one_third_deduction() {
        // $3M IRD, $1.2M estate tax attributable. Three beneficiaries
        // each get $1M → each gets ~$400k §691(c) deduction.
        // 1/3 doesn't divide cleanly in Decimal (limited precision), so
        // we check to two-decimal-rounded equality.
        let mut i = base();
        i.ird_received_by_heir = dec!(1_000_000);
        i.total_ird_in_estate = dec!(3_000_000);
        i.federal_estate_tax_attributable_to_total_ird = dec!(1_200_000);
        let r = compute(&i);
        assert_eq!(r.section_691c_deduction.round_dp(2), dec!(400_000.00));
    }

    #[test]
    fn unequal_beneficiary_share_proportional() {
        // Heir A gets 70% of $1M = $700k IRD. Tot IRD $1M, estate tax
        // $400k → heir A gets 70% × $400k = $280k deduction.
        let mut i = base();
        i.ird_received_by_heir = dec!(700_000);
        i.total_ird_in_estate = dec!(1_000_000);
        i.federal_estate_tax_attributable_to_total_ird = dec!(400_000);
        let r = compute(&i);
        assert_eq!(r.section_691c_deduction, dec!(280_000));
        assert_eq!(r.net_taxable_after_deduction, dec!(420_000));
    }

    #[test]
    fn note_describes_pro_rata_with_dollar_figures() {
        let r = compute(&base());
        assert!(r.note.contains("§691(a)"));
        assert!(r.note.contains("§691(c)"));
        assert!(r.note.contains("200000")); // deduction amount
    }

    #[test]
    fn note_for_zero_deduction_path_explains() {
        let mut i = base();
        i.federal_estate_tax_attributable_to_total_ird = Decimal::ZERO;
        let r = compute(&i);
        assert!(r.note.contains("no §691(c) deduction"));
    }

    #[test]
    fn ird_type_method_classifies_correctly() {
        assert!(IrdType::TraditionalIraDistribution.is_ordinary_character());
        assert!(IrdType::AccruedBondInterest.is_ordinary_character());
        assert!(IrdType::DeferredCompensation.is_ordinary_character());
        assert!(!IrdType::InstallmentSaleGainNotYetRecognized.is_ordinary_character());
    }
}
