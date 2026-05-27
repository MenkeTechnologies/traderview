//! Buying power calculator for margin / Reg-T / PDT scenarios.
//!
//! The user wants to know "if I have $25k equity, how big a position
//! can I take?" The answer depends on:
//!   * account type (cash, margin, portfolio margin)
//!   * Reg-T initial requirement (50% for stocks > $5, 100% for under)
//!   * day-trading multiplier (4× equity if PDT-flagged)
//!   * security-specific maintenance margin (50% standard, 25% maint)
//!
//! Pure compute. Conservative — uses Reg-T initial req for entries,
//! doesn't model exotic instruments (LEAPs, futures with span margin).

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
#[cfg(test)]
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccountType {
    /// 100% cash up front. No margin, no day-trade multiplier.
    Cash,
    /// Standard Reg-T margin. 2× equity overnight, 4× day-trading if
    /// PDT-flagged (≥ $25k + 4 day-trades in 5 days).
    RegT,
    /// Portfolio margin — higher leverage tied to portfolio risk model.
    /// We don't model the SPAN-style calc here; treat as Reg-T × 1.5.
    PortfolioMargin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BpInput {
    pub account_type: AccountType,
    pub equity: Decimal,
    /// True if account has the PDT flag (FINRA Rule 4210 — 4× intraday
    /// leverage). Requires ≥ $25k equity.
    pub is_pdt: bool,
    /// True if the trade will be closed same day.
    pub is_day_trade: bool,
    pub share_price: Decimal,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BpReport {
    /// Maximum dollar notional the user can put up.
    pub max_notional: Decimal,
    /// Maximum share count at the requested price.
    pub max_shares: Decimal,
    /// Effective leverage applied (1.0 cash, 2.0 margin overnight, 4.0 PDT).
    pub leverage: f64,
    /// Reg-T initial requirement % for this share price (1.00 for sub-$5,
    /// 0.50 above).
    pub initial_requirement_pct: f64,
    /// Reason note for the UI (`"cash account: 1× equity, no margin"` etc).
    pub note: String,
}

const PDT_MIN_EQUITY: i64 = 25_000;

pub fn compute(input: &BpInput) -> BpReport {
    // FINRA Rule 4210: stocks under $5 require 100% initial margin (no
    // leverage on sub-$5 stocks regardless of account type).
    let initial_req: f64 = if input.share_price < Decimal::from(5) {
        1.00
    } else {
        0.50
    };

    let pdt_qualified =
        input.is_pdt && input.equity >= Decimal::from(PDT_MIN_EQUITY) && input.is_day_trade;

    let (leverage, note) = match input.account_type {
        AccountType::Cash => (1.0, "cash account: 1× equity, no margin".to_string()),
        AccountType::RegT if pdt_qualified => {
            (4.0, "PDT day-trade: 4× equity intraday".to_string())
        }
        AccountType::RegT if input.share_price < Decimal::from(5) => (
            1.0,
            "sub-$5 stock — Reg-T requires 100% initial margin".to_string(),
        ),
        AccountType::RegT => (2.0, "Reg-T margin: 2× equity overnight".to_string()),
        AccountType::PortfolioMargin if pdt_qualified => (
            6.0,
            "portfolio margin + PDT: 6× equity intraday".to_string(),
        ),
        AccountType::PortfolioMargin => (3.0, "portfolio margin: ~3× equity overnight".to_string()),
    };

    let max_notional = input.equity * Decimal::try_from(leverage).unwrap_or(Decimal::ONE);
    let max_shares = if input.share_price > Decimal::ZERO {
        max_notional / input.share_price
    } else {
        Decimal::ZERO
    };

    BpReport {
        max_notional,
        max_shares,
        leverage,
        initial_requirement_pct: initial_req,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    #[test]
    fn cash_account_is_one_to_one() {
        let r = compute(&BpInput {
            account_type: AccountType::Cash,
            equity: d("10000"),
            is_pdt: false,
            is_day_trade: false,
            share_price: d("50"),
        });
        assert_eq!(r.leverage, 1.0);
        assert_eq!(r.max_notional, d("10000"));
        assert_eq!(r.max_shares, d("200"));
    }

    #[test]
    fn reg_t_overnight_is_two_to_one() {
        let r = compute(&BpInput {
            account_type: AccountType::RegT,
            equity: d("10000"),
            is_pdt: false,
            is_day_trade: false,
            share_price: d("50"),
        });
        assert_eq!(r.leverage, 2.0);
        assert_eq!(r.max_notional, d("20000"));
    }

    #[test]
    fn pdt_day_trade_is_four_to_one_above_25k() {
        let r = compute(&BpInput {
            account_type: AccountType::RegT,
            equity: d("30000"),
            is_pdt: true,
            is_day_trade: true,
            share_price: d("50"),
        });
        assert_eq!(r.leverage, 4.0);
        assert_eq!(r.max_notional, d("120000"));
    }

    #[test]
    fn pdt_below_25k_falls_back_to_2x() {
        // PDT flag without the $25k minimum doesn't grant 4× leverage.
        let r = compute(&BpInput {
            account_type: AccountType::RegT,
            equity: d("20000"),
            is_pdt: true,
            is_day_trade: true,
            share_price: d("50"),
        });
        assert_eq!(r.leverage, 2.0, "PDT requires ≥ $25k equity");
    }

    #[test]
    fn pdt_overnight_loses_the_four_x_multiplier() {
        // Day-trade flag was false — should NOT get 4×, just 2× Reg-T.
        let r = compute(&BpInput {
            account_type: AccountType::RegT,
            equity: d("30000"),
            is_pdt: true,
            is_day_trade: false,
            share_price: d("50"),
        });
        assert_eq!(r.leverage, 2.0);
    }

    #[test]
    fn sub_5_stock_in_reg_t_forces_100_percent_initial() {
        // FINRA Rule 4210 — penny stocks get no margin.
        let r = compute(&BpInput {
            account_type: AccountType::RegT,
            equity: d("10000"),
            is_pdt: false,
            is_day_trade: false,
            share_price: d("3"),
        });
        assert_eq!(r.leverage, 1.0);
        assert!(r.note.contains("sub-$5"));
        assert_eq!(r.initial_requirement_pct, 1.00);
    }

    #[test]
    fn initial_requirement_drops_to_half_above_5_dollars() {
        let r = compute(&BpInput {
            account_type: AccountType::Cash,
            equity: d("10000"),
            is_pdt: false,
            is_day_trade: false,
            share_price: d("5.01"),
        });
        assert_eq!(r.initial_requirement_pct, 0.50);
    }

    #[test]
    fn zero_share_price_yields_zero_shares_not_divide_by_zero() {
        let r = compute(&BpInput {
            account_type: AccountType::Cash,
            equity: d("10000"),
            is_pdt: false,
            is_day_trade: false,
            share_price: Decimal::ZERO,
        });
        assert_eq!(r.max_shares, Decimal::ZERO);
    }

    #[test]
    fn portfolio_margin_overnight_is_3x() {
        let r = compute(&BpInput {
            account_type: AccountType::PortfolioMargin,
            equity: d("100000"),
            is_pdt: false,
            is_day_trade: false,
            share_price: d("100"),
        });
        assert_eq!(r.leverage, 3.0);
        assert_eq!(r.max_notional, d("300000"));
    }

    #[test]
    fn portfolio_margin_pdt_day_trade_is_6x() {
        let r = compute(&BpInput {
            account_type: AccountType::PortfolioMargin,
            equity: d("100000"),
            is_pdt: true,
            is_day_trade: true,
            share_price: d("100"),
        });
        assert_eq!(r.leverage, 6.0);
    }
}
