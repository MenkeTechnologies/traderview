//! Execution friction model.
//!
//! Paper accounts fill at the latest quote. Live execution doesn't —
//! the buyer pays the spread + impact (slippage), pays commission, and
//! on sell pays a per-transaction SEC fee. Without modelling this, paper
//! Sharpe overstates live Sharpe by 30-100bps per round trip on liquid
//! names and more on small caps. Kelly fractions derived from paper
//! stats over-size live positions by 1.5-2×.
//!
//! Two surfaces use this:
//!
//!   * `paper::submit` calls `apply_fill_friction` before recording the
//!     fill, so paper P&L tracks live P&L within the same model.
//!   * `scanner_backtest::backtest_with_history_with_friction` subtracts
//!     round-trip friction from every per-signal return before
//!     aggregating, so the backtested Sharpe is the same Sharpe the
//!     autopilot would have realized.
//!
//! Both paths take the same `FrictionConfig` so changing the model
//! changes both consistently.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FrictionConfig {
    /// One-way slippage in basis points (1 bps = 0.01%). A 5 bps config
    /// adds 0.05% to the buy fill and subtracts 0.05% from the sell.
    /// Round-trip cost = 2 × slippage_bps. Typical liquid US equity at
    /// retail market-order size: 2-10 bps each side.
    pub slippage_bps: f64,
    /// Flat $ per share commission. Alpaca + Tradier equity is $0 today
    /// but options are not; future broker dispatch may pay per-share.
    pub commission_per_share: f64,
    /// Floor commission per order. Tradier options: $0.35 base + $0.35
    /// per contract = $0.70 minimum on a 1-lot.
    pub commission_min_usd: f64,
    /// SEC Section 31 transaction fee on equity SELLS only. Rate as of
    /// 2024: $27.80 per $1M of dollar volume = 0.278 bps. FINRA TAF is
    /// ~$0.000166/share — bundled into this config for simplicity.
    pub sec_fee_bps: f64,
}

impl FrictionConfig {
    /// Sensible baseline for retail equity on Alpaca/Tradier in 2026:
    /// 5 bps slippage each side, no commission, 0.278 bps SEC fee on
    /// sells. Tuned so backtest results approximate realistic live P&L
    /// without requiring a per-symbol microstructure model.
    pub fn baseline_equity() -> Self {
        Self {
            slippage_bps: 5.0,
            commission_per_share: 0.0,
            commission_min_usd: 0.0,
            sec_fee_bps: 0.278,
        }
    }

    pub fn frictionless() -> Self {
        Self {
            slippage_bps: 0.0,
            commission_per_share: 0.0,
            commission_min_usd: 0.0,
            sec_fee_bps: 0.0,
        }
    }

    /// Round-trip friction in percent — what a backtest must subtract
    /// from each signal's gross return to get a friction-adjusted net.
    ///
    /// Breakdown:
    ///   * 2 × slippage_bps / 100 → percent (entry slip + exit slip)
    ///   * sec_fee_bps / 100      → percent (sell side only)
    ///   * commission as % of notional — sized as 100 shares at
    ///     $50/share (i.e. $5,000 notional) so the per-order $1 floor
    ///     doesn't bleed into the per-share rate. Caller can override
    ///     via `round_trip_pct_at_size`.
    pub fn round_trip_pct(self) -> f64 {
        self.round_trip_pct_at_size(50.0, 100)
    }

    /// `round_trip_pct` at a caller-supplied price (assumes 100 shares).
    /// Kept for backwards compat; new callers should use
    /// `round_trip_pct_at_size` which lets them pass typical lot size.
    pub fn round_trip_pct_at_price(self, avg_share_price: f64) -> f64 {
        self.round_trip_pct_at_size(avg_share_price, 100)
    }

    /// Round-trip friction at a specific notional. `avg_share_price`
    /// is per share and `typical_shares` is the lot size used to gate
    /// the per-order $ floor. The old formula `max(per_share, min_usd)`
    /// silently mixed units — for a $50 stock and a $1 floor it claimed
    /// the floor was a per-share rate, overstating round-trip friction
    /// by ~100× on typical lots. The corrected math:
    ///   leg_commission = max(per_share * typical_shares, min_usd)
    ///   notional       = avg_share_price * typical_shares
    ///   commission_pct = 2 * leg_commission / notional * 100
    pub fn round_trip_pct_at_size(self, avg_share_price: f64, typical_shares: u32) -> f64 {
        let slip_pct = 2.0 * self.slippage_bps / 100.0;
        let sec_pct = self.sec_fee_bps / 100.0;
        let shares = typical_shares.max(1) as f64;
        let notional = avg_share_price * shares;
        let comm_pct = if notional > 0.0 {
            let per_share_leg = self.commission_per_share * shares;
            let leg = per_share_leg.max(self.commission_min_usd);
            2.0 * leg / notional * 100.0
        } else {
            0.0
        };
        slip_pct + sec_pct + comm_pct
    }
}

#[cfg(test)]
mod round_trip_tests {
    use super::*;

    #[test]
    fn per_share_only_no_floor() {
        let f = FrictionConfig {
            slippage_bps: 0.0,
            commission_per_share: 0.005, // half-cent/share
            commission_min_usd: 0.0,
            sec_fee_bps: 0.0,
        };
        // $50 × 100 shares = $5000 notional. Round-trip commission:
        // 2 × (0.005 × 100) = $1. As % of $5000: 0.02%.
        let pct = f.round_trip_pct();
        assert!((pct - 0.02).abs() < 1e-6, "got {pct}");
    }

    #[test]
    fn floor_kicks_in_for_tiny_orders() {
        let f = FrictionConfig {
            slippage_bps: 0.0,
            commission_per_share: 0.005,
            commission_min_usd: 1.0,
            sec_fee_bps: 0.0,
        };
        // 10 shares × $50 = $500 notional. per_share leg = 0.005×10 = $0.05;
        // floor = $1. So leg = $1, round-trip = $2. Pct = 2/500*100 = 0.4%.
        let pct = f.round_trip_pct_at_size(50.0, 10);
        assert!((pct - 0.4).abs() < 1e-6, "got {pct}");
    }

    #[test]
    fn does_not_overstate_on_typical_lot() {
        // Regression for the old `max(per_share, min_usd / 1.0)` bug
        // that mixed $/share with $/order. With per_share=0.005 and
        // min=$1, the old formula treated the $1 as per-share → leg=$1
        // → round-trip commission ≈ 4% at $50/share. The fixed math
        // returns 0.02% (a $1 floor distributed over a 100-share, $5000
        // notional order is negligible).
        let f = FrictionConfig {
            slippage_bps: 0.0,
            commission_per_share: 0.005,
            commission_min_usd: 1.0,
            sec_fee_bps: 0.0,
        };
        let pct = f.round_trip_pct();
        assert!(pct < 0.1, "old bug returned ~4%, fixed math should be <0.1%, got {pct}");
    }
}

/// Single-fill friction application. Returns the adjusted fill price +
/// commission for that leg. Slippage moves the price *against* the
/// trader: buyer pays MORE, seller receives LESS.
#[derive(Debug, Clone, Copy)]
pub struct FillFriction {
    pub fill_price: f64,
    pub commission_usd: f64,
    pub sec_fee_usd: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillSide {
    BuyOpen,
    SellClose,
    SellOpen, // short open
    BuyClose, // cover
}

impl FillSide {
    /// `true` when the leg pays the SEC transaction fee (US equity sells).
    pub fn pays_sec_fee(self) -> bool {
        matches!(self, FillSide::SellClose | FillSide::SellOpen)
    }
    /// Slippage sign: +1 means quote price moves UP against us (buys),
    /// -1 means quote price moves DOWN against us (sells).
    pub fn slippage_sign(self) -> f64 {
        match self {
            FillSide::BuyOpen | FillSide::BuyClose => 1.0,
            FillSide::SellClose | FillSide::SellOpen => -1.0,
        }
    }
}

pub fn apply_fill_friction(
    quote_price: f64,
    qty: f64,
    side: FillSide,
    cfg: FrictionConfig,
) -> FillFriction {
    if !(quote_price > 0.0 && qty > 0.0) {
        return FillFriction {
            fill_price: quote_price,
            commission_usd: 0.0,
            sec_fee_usd: 0.0,
        };
    }
    let slip_pct = cfg.slippage_bps / 10_000.0 * side.slippage_sign();
    let fill_price = quote_price * (1.0 + slip_pct);
    let notional = fill_price * qty;
    let raw_commission = (cfg.commission_per_share * qty).max(cfg.commission_min_usd);
    let commission_usd = if raw_commission < 0.0 {
        0.0
    } else {
        raw_commission
    };
    let sec_fee_usd = if side.pays_sec_fee() {
        notional * (cfg.sec_fee_bps / 10_000.0)
    } else {
        0.0
    };
    FillFriction {
        fill_price,
        commission_usd,
        sec_fee_usd,
    }
}

/// Subtract round-trip friction from each per-signal return. The
/// scanner backtest collects gross log-percent returns; live trading
/// would pay friction on entry + exit, so we deduct the friction.
pub fn friction_adjusted_returns(returns_gross_pct: &[f64], cfg: FrictionConfig) -> Vec<f64> {
    let cost_pct = cfg.round_trip_pct();
    returns_gross_pct.iter().map(|r| r - cost_pct).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_round_trip_subtracts_slip_plus_sec() {
        let cfg = FrictionConfig::baseline_equity();
        let rt = cfg.round_trip_pct();
        // 2 × 5 bps slip + 0.278 bps SEC + zero commission at $50/share
        // = 0.10% + 0.00278% = 0.10278%
        assert!((rt - 0.10278).abs() < 1e-3, "rt = {rt}");
    }

    #[test]
    fn frictionless_config_no_cost() {
        let cfg = FrictionConfig::frictionless();
        assert_eq!(cfg.round_trip_pct(), 0.0);
    }

    #[test]
    fn buyer_pays_slippage_up() {
        let cfg = FrictionConfig::baseline_equity();
        let f = apply_fill_friction(100.0, 10.0, FillSide::BuyOpen, cfg);
        // 100 × (1 + 5/10000) = 100.05
        assert!((f.fill_price - 100.05).abs() < 1e-9);
        assert_eq!(f.sec_fee_usd, 0.0, "buys pay no SEC fee");
    }

    #[test]
    fn seller_receives_slippage_down_and_pays_sec_fee() {
        let cfg = FrictionConfig::baseline_equity();
        let f = apply_fill_friction(100.0, 10.0, FillSide::SellClose, cfg);
        // 100 × (1 - 5/10000) = 99.95
        assert!((f.fill_price - 99.95).abs() < 1e-9);
        // SEC: notional × 0.278 / 10000 = 999.5 × 0.0000278 ≈ 0.0278
        assert!((f.sec_fee_usd - 0.0278).abs() < 1e-3);
    }

    #[test]
    fn short_open_treated_as_sell_for_sec_purposes() {
        let cfg = FrictionConfig::baseline_equity();
        let f = apply_fill_friction(100.0, 10.0, FillSide::SellOpen, cfg);
        assert!(f.sec_fee_usd > 0.0);
        assert!((f.fill_price - 99.95).abs() < 1e-9);
    }

    #[test]
    fn buy_close_cover_pays_no_sec_fee() {
        let cfg = FrictionConfig::baseline_equity();
        let f = apply_fill_friction(100.0, 10.0, FillSide::BuyClose, cfg);
        assert_eq!(f.sec_fee_usd, 0.0);
        assert!(f.fill_price > 100.0, "cover still pays slippage UP");
    }

    #[test]
    fn commission_min_floor_applies() {
        let cfg = FrictionConfig {
            slippage_bps: 0.0,
            commission_per_share: 0.001,
            commission_min_usd: 1.0,
            sec_fee_bps: 0.0,
        };
        // 10 shares × $0.001 = $0.01 raw, floor $1 → $1.
        let f = apply_fill_friction(100.0, 10.0, FillSide::BuyOpen, cfg);
        assert_eq!(f.commission_usd, 1.0);
    }

    #[test]
    fn invalid_inputs_return_unchanged_quote_zero_costs() {
        let cfg = FrictionConfig::baseline_equity();
        let f = apply_fill_friction(0.0, 10.0, FillSide::BuyOpen, cfg);
        assert_eq!(f.fill_price, 0.0);
        assert_eq!(f.commission_usd, 0.0);
        assert_eq!(f.sec_fee_usd, 0.0);
        let f2 = apply_fill_friction(100.0, -5.0, FillSide::BuyOpen, cfg);
        assert_eq!(f2.fill_price, 100.0);
    }

    #[test]
    fn friction_adjusted_returns_subtracts_round_trip_from_each() {
        let cfg = FrictionConfig::baseline_equity();
        let rt = cfg.round_trip_pct();
        let gross = vec![1.0, 2.0, -0.5];
        let net = friction_adjusted_returns(&gross, cfg);
        assert!((net[0] - (1.0 - rt)).abs() < 1e-9);
        assert!((net[1] - (2.0 - rt)).abs() < 1e-9);
        assert!((net[2] - (-0.5 - rt)).abs() < 1e-9);
    }

    #[test]
    fn friction_adjusted_with_frictionless_passes_through() {
        let cfg = FrictionConfig::frictionless();
        let gross = vec![1.0, 2.0];
        let net = friction_adjusted_returns(&gross, cfg);
        assert_eq!(net, gross);
    }

    #[test]
    fn pays_sec_fee_only_for_sells() {
        assert!(FillSide::SellClose.pays_sec_fee());
        assert!(FillSide::SellOpen.pays_sec_fee());
        assert!(!FillSide::BuyOpen.pays_sec_fee());
        assert!(!FillSide::BuyClose.pays_sec_fee());
    }
}
