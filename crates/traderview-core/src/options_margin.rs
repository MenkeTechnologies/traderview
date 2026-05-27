//! Options margin estimator for retail Reg-T accounts.
//!
//! Per FINRA Rule 4210, common short-option / spread requirements:
//!   - Naked call:  greatest of [20% × underlying - OTM amount, 10% × underlying] × 100,
//!                  plus the premium received
//!   - Naked put:   greatest of [20% × underlying - OTM amount, 10% × strike] × 100,
//!                  plus the premium
//!   - Vertical debit spread: net debit paid (caller usually pre-computes)
//!   - Vertical credit spread: max loss = (strike width × 100) - premium received
//!   - Iron condor: max loss = greater of the two wings (long + short combined)
//!
//! This is the estimator broker margin systems use for initial requirement
//! at order entry. NOT portfolio-margin (which is risk-arrayed) and NOT
//! the exact intraday margin (which depends on real-time deltas).
//!
//! Pure compute.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionRight { Call, Put }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NakedShortOption {
    pub right: OptionRight,
    pub underlying_price: Decimal,
    pub strike: Decimal,
    pub premium: Decimal,    // received credit per share
    pub contracts: i64,      // short qty (positive)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerticalSpread {
    pub right: OptionRight,
    /// Strike width × multiplier (typically 100). Caller multiplies.
    pub strike_width: Decimal,
    pub net_premium: Decimal,    // positive = credit, negative = debit
    pub contracts: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MarginReport {
    /// Initial requirement (estimate). Caller adds this to existing
    /// account margin usage to see total exposure.
    pub initial_requirement: Decimal,
    pub max_loss: Option<Decimal>,
    pub max_gain: Option<Decimal>,
    pub explanation: String,
}

pub fn naked_short(opt: &NakedShortOption) -> MarginReport {
    let twenty_pct = pct("0.20");
    let ten_pct = pct("0.10");
    let mult = Decimal::from(100);
    let contracts_d = Decimal::from(opt.contracts);

    let otm = match opt.right {
        // Calls OTM amount: max(strike - underlying, 0).
        OptionRight::Call => (opt.strike - opt.underlying_price).max(Decimal::ZERO),
        // Puts OTM amount: max(underlying - strike, 0).
        OptionRight::Put  => (opt.underlying_price - opt.strike).max(Decimal::ZERO),
    };

    let twenty_minus_otm = (opt.underlying_price * twenty_pct - otm).max(Decimal::ZERO);
    let floor = match opt.right {
        OptionRight::Call => opt.underlying_price * ten_pct,
        OptionRight::Put  => opt.strike * ten_pct,
    };
    let per_share = twenty_minus_otm.max(floor);
    let per_contract = per_share * mult + opt.premium * mult;
    let total = per_contract * contracts_d;

    // Max loss: undefined for naked calls (∞), strike×100 for naked puts.
    let max_loss = match opt.right {
        OptionRight::Call => None,
        OptionRight::Put  => Some((opt.strike - opt.premium) * mult * contracts_d),
    };
    let max_gain = Some(opt.premium * mult * contracts_d);

    MarginReport {
        initial_requirement: total,
        max_loss,
        max_gain,
        explanation: format!(
            "naked short {} × {}: per-contract req ${} (greater of 20%-OTM × 100 vs 10%-floor × 100), plus premium",
            opt.contracts,
            match opt.right { OptionRight::Call => "call", OptionRight::Put => "put" },
            per_contract,
        ),
    }
}

pub fn vertical(spread: &VerticalSpread) -> MarginReport {
    let mult = Decimal::from(100);
    let contracts_d = Decimal::from(spread.contracts);
    if spread.net_premium >= Decimal::ZERO {
        // Credit spread — margin = max loss = (strike_width × 100) - net premium.
        let per = spread.strike_width * mult - spread.net_premium * mult;
        let total = per * contracts_d;
        let max_loss = total;
        let max_gain = spread.net_premium * mult * contracts_d;
        MarginReport {
            initial_requirement: total,
            max_loss: Some(max_loss),
            max_gain: Some(max_gain),
            explanation: format!(
                "credit vertical — margin = max loss = (width × 100 - credit) × {} contracts",
                spread.contracts,
            ),
        }
    } else {
        // Debit spread — margin = max loss = net debit paid (absolute value).
        let debit_per = -spread.net_premium * mult;
        let total = debit_per * contracts_d;
        let max_gain = spread.strike_width * mult * contracts_d + spread.net_premium * mult * contracts_d;
        MarginReport {
            initial_requirement: total,
            max_loss: Some(total),
            max_gain: Some(max_gain),
            explanation: format!(
                "debit vertical — margin = debit paid × {} contracts",
                spread.contracts,
            ),
        }
    }
}

fn pct(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }

    // ─── naked options ────────────────────────────────────────────────

    #[test]
    fn naked_short_call_atm_uses_20pct_floor() {
        // SPY at $500, short the $500 call (ATM) — OTM = 0.
        // per-share = max(0.20 × 500 - 0, 0.10 × 500) = max(100, 50) = 100.
        // premium $5/sh × 100 = $500 added.
        // total per contract: 100 × 100 + 500 = $10,500.
        let r = naked_short(&NakedShortOption {
            right: OptionRight::Call,
            underlying_price: d("500"),
            strike: d("500"),
            premium: d("5"),
            contracts: 1,
        });
        assert_eq!(r.initial_requirement, d("10500"));
        assert!(r.max_loss.is_none(), "naked call has unlimited loss");
        assert_eq!(r.max_gain, Some(d("500")));
    }

    #[test]
    fn naked_short_call_far_otm_uses_10pct_floor() {
        // SPY $500, strike $600 (way OTM). OTM = 100.
        // 20%-otm: max(100 - 100, 0) = 0.
        // 10%-floor: 50.
        // per-share = max(0, 50) = 50.
        let r = naked_short(&NakedShortOption {
            right: OptionRight::Call,
            underlying_price: d("500"),
            strike: d("600"),
            premium: d("1"),
            contracts: 1,
        });
        // 50 × 100 + 1 × 100 = $5,100.
        assert_eq!(r.initial_requirement, d("5100"));
    }

    #[test]
    fn naked_short_put_uses_10pct_of_strike_as_floor() {
        // SPY $500, short the $480 put. OTM = 20.
        // 20%-otm: max(0.20 × 500 - 20, 0) = max(80, 0) = 80.
        // 10%-floor: 0.10 × 480 = 48.
        // per-share = max(80, 48) = 80.
        // premium $3 × 100 = 300. Total: 80 × 100 + 300 = $8,300.
        let r = naked_short(&NakedShortOption {
            right: OptionRight::Put,
            underlying_price: d("500"),
            strike: d("480"),
            premium: d("3"),
            contracts: 1,
        });
        assert_eq!(r.initial_requirement, d("8300"));
        // Max loss on a put: (strike - premium) × 100 = (480 - 3) × 100 = $47,700.
        assert_eq!(r.max_loss, Some(d("47700")));
    }

    #[test]
    fn naked_scales_linearly_with_contracts() {
        let one = naked_short(&NakedShortOption {
            right: OptionRight::Call,
            underlying_price: d("500"),
            strike: d("500"),
            premium: d("5"),
            contracts: 1,
        });
        let five = naked_short(&NakedShortOption {
            right: OptionRight::Call,
            underlying_price: d("500"),
            strike: d("500"),
            premium: d("5"),
            contracts: 5,
        });
        assert_eq!(five.initial_requirement, one.initial_requirement * d("5"));
    }

    // ─── verticals ────────────────────────────────────────────────────

    #[test]
    fn credit_vertical_margin_is_width_minus_credit() {
        // 5-wide bull put credit spread, $1 credit per share.
        // Per contract margin = (5 - 1) × 100 = $400.
        let r = vertical(&VerticalSpread {
            right: OptionRight::Put,
            strike_width: d("5"),
            net_premium: d("1"),    // credit
            contracts: 1,
        });
        assert_eq!(r.initial_requirement, d("400"));
        assert_eq!(r.max_loss, Some(d("400")));
        assert_eq!(r.max_gain, Some(d("100")));
    }

    #[test]
    fn debit_vertical_margin_equals_debit_paid() {
        // 5-wide bull call debit spread, -$2 net (paid 2 per share).
        let r = vertical(&VerticalSpread {
            right: OptionRight::Call,
            strike_width: d("5"),
            net_premium: d("-2"),
            contracts: 1,
        });
        // Margin = debit × 100 = $200.
        assert_eq!(r.initial_requirement, d("200"));
        assert_eq!(r.max_loss, Some(d("200")));
        // Max gain: (width - debit) × 100 = (5 - 2) × 100 = $300.
        assert_eq!(r.max_gain, Some(d("300")));
    }

    #[test]
    fn credit_vertical_scales_with_contracts() {
        let one = vertical(&VerticalSpread {
            right: OptionRight::Put,
            strike_width: d("5"),
            net_premium: d("1"),
            contracts: 1,
        });
        let ten = vertical(&VerticalSpread {
            right: OptionRight::Put,
            strike_width: d("5"),
            net_premium: d("1"),
            contracts: 10,
        });
        assert_eq!(ten.initial_requirement, one.initial_requirement * d("10"));
        assert_eq!(ten.max_loss, Some(d("4000")));
    }
}
