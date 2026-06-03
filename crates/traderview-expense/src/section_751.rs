//! IRC § 751 unrealized receivables and inventory items
//! ("hot assets") recharacterization for partnership transactions.
//!
//! § 751 overrides the default capital-character treatment of
//! Subchapter K in two scenarios:
//!
//! **§ 751(a) — Sale or exchange of a partnership interest**: the
//! portion of the amount realized attributable to the partner's share
//! of unrealized receivables and inventory items is treated as
//! ORDINARY income (gain or loss), and the remainder receives § 741
//! capital character. For § 751(a) purposes, ALL inventory is hot
//! regardless of appreciation — the "substantially appreciated"
//! threshold applies only to § 751(b) distributions.
//!
//! **§ 751(b) — Disproportionate distributions**: a partnership
//! current or liquidating distribution that ALTERS the distributee
//! partner's share of unrealized receivables or SUBSTANTIALLY
//! APPRECIATED inventory items is recast as a deemed sale or exchange
//! between the distributee and the partnership of the disproportionate
//! portion. For § 751(b), inventory is "substantially appreciated"
//! ONLY if fair market value exceeds 120% of adjusted basis
//! (§ 751(b)(3)(A)).
//!
//! Unrealized receivables under § 751(c) include rights to payment
//! for goods delivered or to be delivered + services rendered or to
//! be rendered, plus the potential ordinary recapture income from
//! § 1245, § 1250, § 1252, § 1254, § 617(d), and similar recapture
//! provisions on partnership-owned recapture property.
//!
//! Inventory items under § 751(d) include partnership inventory plus
//! any other property held by the partnership which, on sale or
//! exchange, would be considered property other than a capital asset
//! and other than § 1231 property.

#![allow(dead_code)]

use serde::{Deserialize, Serialize};

#[allow(dead_code)]
pub const SUBSTANTIALLY_APPRECIATED_INVENTORY_FMV_OVER_BASIS_PERCENT: u64 = 120;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    NotApplicable,
    SaleOrExchangeOfPartnershipInterest,
    CurrentDistribution,
    LiquidatingDistribution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    NotApplicable,
    SaleAllCapitalNoHotAssetsAtRisk,
    SaleBifurcatedOrdinaryHotAssetsPlusCapitalRemainder,
    DistributionProportionateNoSection751bRecastNeeded,
    DistributionDisproportionateAlterShareOfHotAssetsRecast,
    ViolationFailedToBifurcateReportedAllCapital,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Input {
    pub transaction_type: TransactionType,
    pub partner_outside_basis_cents: u64,
    pub amount_realized_cents: u64,
    pub partner_share_of_unrealized_receivables_cents: u64,
    pub partner_share_of_inventory_fmv_cents: u64,
    pub partner_share_of_inventory_adjusted_basis_cents: u64,
    pub distribution_alters_share_of_hot_assets: bool,
    pub taxpayer_reported_all_capital: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Output {
    pub severity: Severity,
    pub ordinary_income_from_hot_assets_cents: u64,
    pub capital_gain_cents: i128,
    pub substantially_appreciated_inventory: bool,
    pub bifurcation_required: bool,
    pub notes: Vec<String>,
    pub citations: Vec<String>,
}

pub type Section751Input = Input;
pub type Section751Output = Output;
pub type Section751Result = Output;

pub fn check(input: &Input) -> Output {
    let mut notes: Vec<String> = Vec::new();
    let citations: Vec<String> = vec![
        "IRC § 751(a) (sale or exchange of partnership interest)".to_string(),
        "IRC § 751(b) (disproportionate distributions)".to_string(),
        "IRC § 751(c) (unrealized receivables definition)".to_string(),
        "IRC § 751(d) (inventory items definition)".to_string(),
        "IRC § 751(b)(3)(A) (substantially appreciated 120% threshold)".to_string(),
        "IRC § 741 (capital character default rule)".to_string(),
        "IRC § 731 (partnership distribution nonrecognition default rule)".to_string(),
        "Treas. Reg. § 1.751-1".to_string(),
        "IRS Notice 2006-14".to_string(),
    ];

    let inventory_fmv = input.partner_share_of_inventory_fmv_cents;
    let inventory_basis = input.partner_share_of_inventory_adjusted_basis_cents;
    let substantially_appreciated = inventory_basis > 0
        && inventory_fmv.saturating_mul(100)
            > inventory_basis
                .saturating_mul(SUBSTANTIALLY_APPRECIATED_INVENTORY_FMV_OVER_BASIS_PERCENT);

    match input.transaction_type {
        TransactionType::NotApplicable => {
            notes.push("No partnership interest sale or distribution recorded.".to_string());
            Output {
                severity: Severity::NotApplicable,
                ordinary_income_from_hot_assets_cents: 0,
                capital_gain_cents: 0,
                substantially_appreciated_inventory: substantially_appreciated,
                bifurcation_required: false,
                notes,
                citations,
            }
        }
        TransactionType::SaleOrExchangeOfPartnershipInterest => {
            let hot_assets_total = input
                .partner_share_of_unrealized_receivables_cents
                .saturating_add(inventory_fmv);

            if hot_assets_total == 0 {
                let total_gain = (input.amount_realized_cents as i128)
                    - (input.partner_outside_basis_cents as i128);
                notes.push(format!(
                    "Sale of partnership interest with $0 hot-asset share — full ${} treated as § 741 capital gain/loss.",
                    total_gain / 100
                ));
                return Output {
                    severity: Severity::SaleAllCapitalNoHotAssetsAtRisk,
                    ordinary_income_from_hot_assets_cents: 0,
                    capital_gain_cents: total_gain,
                    substantially_appreciated_inventory: substantially_appreciated,
                    bifurcation_required: false,
                    notes,
                    citations,
                };
            }

            let ordinary_portion = hot_assets_total
                .saturating_sub(inventory_basis);
            let capital_portion: i128 = (input.amount_realized_cents as i128)
                - (input.partner_outside_basis_cents as i128)
                - (ordinary_portion as i128);

            if input.taxpayer_reported_all_capital {
                notes.push(format!(
                    "Taxpayer treated entire ${} as capital, but § 751(a) requires bifurcation: ${} ordinary (hot assets) + ${} capital.",
                    input.amount_realized_cents / 100,
                    ordinary_portion / 100,
                    capital_portion / 100
                ));
                return Output {
                    severity: Severity::ViolationFailedToBifurcateReportedAllCapital,
                    ordinary_income_from_hot_assets_cents: ordinary_portion,
                    capital_gain_cents: capital_portion,
                    substantially_appreciated_inventory: substantially_appreciated,
                    bifurcation_required: true,
                    notes,
                    citations,
                };
            }

            notes.push(format!(
                "§ 751(a) bifurcation: ${} ordinary (hot assets: ${} unrealized receivables + ${} inventory FMV − ${} inventory basis) + ${} § 741 capital.",
                ordinary_portion / 100,
                input.partner_share_of_unrealized_receivables_cents / 100,
                inventory_fmv / 100,
                inventory_basis / 100,
                capital_portion / 100
            ));
            Output {
                severity: Severity::SaleBifurcatedOrdinaryHotAssetsPlusCapitalRemainder,
                ordinary_income_from_hot_assets_cents: ordinary_portion,
                capital_gain_cents: capital_portion,
                substantially_appreciated_inventory: substantially_appreciated,
                bifurcation_required: true,
                notes,
                citations,
            }
        }
        TransactionType::CurrentDistribution | TransactionType::LiquidatingDistribution => {
            if !input.distribution_alters_share_of_hot_assets {
                notes.push("Proportionate distribution: § 751(b) recast not triggered; § 731 nonrecognition default applies.".to_string());
                return Output {
                    severity: Severity::DistributionProportionateNoSection751bRecastNeeded,
                    ordinary_income_from_hot_assets_cents: 0,
                    capital_gain_cents: 0,
                    substantially_appreciated_inventory: substantially_appreciated,
                    bifurcation_required: false,
                    notes,
                    citations,
                };
            }

            let inventory_hot_for_751b = if substantially_appreciated {
                inventory_fmv
            } else {
                0
            };
            let hot_assets_total = input
                .partner_share_of_unrealized_receivables_cents
                .saturating_add(inventory_hot_for_751b);
            let inventory_hot_for_751b_basis = if substantially_appreciated {
                inventory_basis
            } else {
                0
            };
            let ordinary_portion = hot_assets_total.saturating_sub(inventory_hot_for_751b_basis);

            notes.push(format!(
                "§ 751(b) disproportionate distribution recast: deemed sale/exchange of hot-asset portion = ${} ordinary income (inventory substantially appreciated: {} — FMV/basis = {}%).",
                ordinary_portion / 100,
                substantially_appreciated,
                inventory_fmv
                    .saturating_mul(100)
                    .checked_div(inventory_basis)
                    .unwrap_or(0)
            ));
            Output {
                severity: Severity::DistributionDisproportionateAlterShareOfHotAssetsRecast,
                ordinary_income_from_hot_assets_cents: ordinary_portion,
                capital_gain_cents: 0,
                substantially_appreciated_inventory: substantially_appreciated,
                bifurcation_required: true,
                notes,
                citations,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_sale() -> Input {
        Input {
            transaction_type: TransactionType::SaleOrExchangeOfPartnershipInterest,
            partner_outside_basis_cents: 5_000_000,
            amount_realized_cents: 10_000_000,
            partner_share_of_unrealized_receivables_cents: 1_000_000,
            partner_share_of_inventory_fmv_cents: 2_000_000,
            partner_share_of_inventory_adjusted_basis_cents: 1_500_000,
            distribution_alters_share_of_hot_assets: false,
            taxpayer_reported_all_capital: false,
        }
    }

    #[test]
    fn sale_with_hot_assets_bifurcates_correctly() {
        let out = check(&base_sale());
        assert_eq!(
            out.severity,
            Severity::SaleBifurcatedOrdinaryHotAssetsPlusCapitalRemainder
        );
        assert_eq!(out.ordinary_income_from_hot_assets_cents, 1_500_000);
        assert_eq!(out.capital_gain_cents, 3_500_000);
        assert!(out.bifurcation_required);
    }

    #[test]
    fn sale_with_zero_hot_assets_all_capital() {
        let mut i = base_sale();
        i.partner_share_of_unrealized_receivables_cents = 0;
        i.partner_share_of_inventory_fmv_cents = 0;
        i.partner_share_of_inventory_adjusted_basis_cents = 0;
        let out = check(&i);
        assert_eq!(out.severity, Severity::SaleAllCapitalNoHotAssetsAtRisk);
        assert_eq!(out.ordinary_income_from_hot_assets_cents, 0);
        assert_eq!(out.capital_gain_cents, 5_000_000);
        assert!(!out.bifurcation_required);
    }

    #[test]
    fn sale_taxpayer_reported_all_capital_is_violation() {
        let mut i = base_sale();
        i.taxpayer_reported_all_capital = true;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::ViolationFailedToBifurcateReportedAllCapital
        );
        assert_eq!(out.ordinary_income_from_hot_assets_cents, 1_500_000);
    }

    #[test]
    fn sale_inventory_substantially_appreciated_flag_set_at_121_percent() {
        let mut i = base_sale();
        i.partner_share_of_inventory_fmv_cents = 1_210_000;
        i.partner_share_of_inventory_adjusted_basis_cents = 1_000_000;
        let out = check(&i);
        assert!(out.substantially_appreciated_inventory);
    }

    #[test]
    fn sale_inventory_at_120_percent_not_substantially_appreciated() {
        let mut i = base_sale();
        i.partner_share_of_inventory_fmv_cents = 1_200_000;
        i.partner_share_of_inventory_adjusted_basis_cents = 1_000_000;
        let out = check(&i);
        assert!(!out.substantially_appreciated_inventory);
    }

    #[test]
    fn sale_inventory_at_119_percent_not_substantially_appreciated() {
        let mut i = base_sale();
        i.partner_share_of_inventory_fmv_cents = 1_190_000;
        i.partner_share_of_inventory_adjusted_basis_cents = 1_000_000;
        let out = check(&i);
        assert!(!out.substantially_appreciated_inventory);
    }

    #[test]
    fn sale_all_inventory_is_hot_regardless_of_appreciation_under_751a() {
        let mut i = base_sale();
        i.partner_share_of_inventory_fmv_cents = 800_000;
        i.partner_share_of_inventory_adjusted_basis_cents = 1_000_000;
        i.partner_share_of_unrealized_receivables_cents = 0;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::SaleBifurcatedOrdinaryHotAssetsPlusCapitalRemainder
        );
        assert!(!out.substantially_appreciated_inventory);
        assert!(out.bifurcation_required);
    }

    #[test]
    fn proportionate_distribution_no_751b_recast() {
        let mut i = base_sale();
        i.transaction_type = TransactionType::CurrentDistribution;
        i.distribution_alters_share_of_hot_assets = false;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::DistributionProportionateNoSection751bRecastNeeded
        );
    }

    #[test]
    fn disproportionate_distribution_with_substantially_appreciated_recast() {
        let mut i = base_sale();
        i.transaction_type = TransactionType::CurrentDistribution;
        i.distribution_alters_share_of_hot_assets = true;
        i.partner_share_of_inventory_fmv_cents = 1_250_000;
        i.partner_share_of_inventory_adjusted_basis_cents = 1_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::DistributionDisproportionateAlterShareOfHotAssetsRecast
        );
        assert!(out.substantially_appreciated_inventory);
        assert_eq!(out.ordinary_income_from_hot_assets_cents, 1_250_000);
    }

    #[test]
    fn disproportionate_distribution_inventory_not_substantially_appreciated_only_receivables_hot() {
        let mut i = base_sale();
        i.transaction_type = TransactionType::CurrentDistribution;
        i.distribution_alters_share_of_hot_assets = true;
        i.partner_share_of_inventory_fmv_cents = 1_100_000;
        i.partner_share_of_inventory_adjusted_basis_cents = 1_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::DistributionDisproportionateAlterShareOfHotAssetsRecast
        );
        assert!(!out.substantially_appreciated_inventory);
        assert_eq!(
            out.ordinary_income_from_hot_assets_cents,
            i.partner_share_of_unrealized_receivables_cents
        );
    }

    #[test]
    fn liquidating_distribution_treated_same_as_current_for_751b() {
        let mut i = base_sale();
        i.transaction_type = TransactionType::LiquidatingDistribution;
        i.distribution_alters_share_of_hot_assets = true;
        i.partner_share_of_inventory_fmv_cents = 1_250_000;
        i.partner_share_of_inventory_adjusted_basis_cents = 1_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::DistributionDisproportionateAlterShareOfHotAssetsRecast
        );
    }

    #[test]
    fn not_applicable_transaction_returns_zero() {
        let mut i = base_sale();
        i.transaction_type = TransactionType::NotApplicable;
        let out = check(&i);
        assert_eq!(out.severity, Severity::NotApplicable);
        assert_eq!(out.ordinary_income_from_hot_assets_cents, 0);
        assert_eq!(out.capital_gain_cents, 0);
    }

    #[test]
    fn negative_capital_loss_recorded_correctly() {
        let mut i = base_sale();
        i.amount_realized_cents = 3_000_000;
        let out = check(&i);
        assert_eq!(
            out.severity,
            Severity::SaleBifurcatedOrdinaryHotAssetsPlusCapitalRemainder
        );
        assert_eq!(out.ordinary_income_from_hot_assets_cents, 1_500_000);
        assert_eq!(out.capital_gain_cents, -3_500_000);
    }

    #[test]
    fn citations_pin_751a_751b_741_731() {
        let out = check(&base_sale());
        assert!(out.citations.iter().any(|c| c.contains("§ 751(a)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 751(b)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 741")));
        assert!(out.citations.iter().any(|c| c.contains("§ 731")));
    }

    #[test]
    fn citations_pin_751c_751d_751b3a_treas_reg() {
        let out = check(&base_sale());
        assert!(out.citations.iter().any(|c| c.contains("§ 751(c)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 751(d)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 751(b)(3)(A)")));
        assert!(out.citations.iter().any(|c| c.contains("§ 1.751-1")));
    }

    #[test]
    fn citations_pin_irs_notice_2006_14() {
        let out = check(&base_sale());
        assert!(out.citations.iter().any(|c| c.contains("Notice 2006-14")));
    }

    #[test]
    fn constant_pin_substantially_appreciated_120_percent_threshold() {
        assert_eq!(SUBSTANTIALLY_APPRECIATED_INVENTORY_FMV_OVER_BASIS_PERCENT, 120);
    }

    #[test]
    fn very_large_hot_assets_saturating_does_not_overflow() {
        let mut i = base_sale();
        i.partner_share_of_unrealized_receivables_cents = u64::MAX;
        i.partner_share_of_inventory_fmv_cents = u64::MAX;
        i.partner_share_of_inventory_adjusted_basis_cents = 0;
        let out = check(&i);
        assert_eq!(out.ordinary_income_from_hot_assets_cents, u64::MAX);
    }

    #[test]
    fn zero_inventory_basis_avoids_division_by_zero() {
        let mut i = base_sale();
        i.partner_share_of_inventory_adjusted_basis_cents = 0;
        let out = check(&i);
        assert!(!out.substantially_appreciated_inventory);
    }
}
