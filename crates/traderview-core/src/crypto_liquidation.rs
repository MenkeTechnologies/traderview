//! Crypto perpetual liquidation price (isolated margin, linear/USDT-settled) — the
//! mark price at which an isolated-margin perpetual position is force-liquidated.
//! For a long, liquidation is `entry × (1 − 1/leverage + maintenance_margin_rate)`;
//! for a short, `entry × (1 + 1/leverage − maintenance_margin_rate)`. It also
//! reports the distance to liquidation, notional, and the initial/maintenance
//! margin. Pure compute, not exchange-specific (cross-margin and inverse contracts
//! differ). Not financial advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct LiquidationInput {
    /// "long" or "short".
    pub side: String,
    pub entry_price_usd: f64,
    /// Leverage, e.g. 10 for 10×.
    pub leverage: f64,
    /// Maintenance margin rate, decimal (e.g. 0.005 = 0.5%).
    #[serde(default = "default_mmr")]
    pub maintenance_margin_rate: f64,
    /// Position size in coins/contracts (for margin amounts).
    #[serde(default)]
    pub position_size: f64,
}

fn default_mmr() -> f64 {
    0.005
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct LiquidationReport {
    pub is_long: bool,
    pub liquidation_price_usd: f64,
    /// Bankruptcy price (equity = 0, ignoring maintenance margin).
    pub bankruptcy_price_usd: f64,
    /// |liq − entry| ÷ entry, percent.
    pub distance_pct: f64,
    pub notional_usd: f64,
    pub initial_margin_usd: f64,
    pub maintenance_margin_usd: f64,
    pub valid: bool,
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

pub fn generate(i: &LiquidationInput) -> LiquidationReport {
    if i.leverage <= 0.0 || i.entry_price_usd <= 0.0 {
        return LiquidationReport::default();
    }
    let is_long = !i.side.trim().eq_ignore_ascii_case("short");
    let imr = 1.0 / i.leverage;
    let mmr = i.maintenance_margin_rate;
    let liq = if is_long {
        i.entry_price_usd * (1.0 - imr + mmr)
    } else {
        i.entry_price_usd * (1.0 + imr - mmr)
    };
    let bankruptcy = if is_long {
        i.entry_price_usd * (1.0 - imr)
    } else {
        i.entry_price_usd * (1.0 + imr)
    };
    let notional = i.entry_price_usd * i.position_size;
    LiquidationReport {
        is_long,
        liquidation_price_usd: round2(liq),
        bankruptcy_price_usd: round2(bankruptcy.max(0.0)),
        distance_pct: round2((liq - i.entry_price_usd).abs() / i.entry_price_usd * 100.0),
        notional_usd: round2(notional),
        initial_margin_usd: round2(notional * imr),
        maintenance_margin_usd: round2(notional * mmr),
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> LiquidationInput {
        LiquidationInput {
            side: "long".into(),
            entry_price_usd: 30_000.0,
            leverage: 10.0,
            maintenance_margin_rate: 0.005,
            position_size: 1.0,
        }
    }

    #[test]
    fn long_liquidation() {
        let d = generate(&base());
        assert!(d.is_long);
        assert!(close(d.liquidation_price_usd, 27_150.0));
        assert!(close(d.distance_pct, 9.5));
        assert!(close(d.bankruptcy_price_usd, 27_000.0));
        assert!(close(d.initial_margin_usd, 3_000.0));
        assert!(close(d.maintenance_margin_usd, 150.0));
    }

    #[test]
    fn short_liquidation() {
        let d = generate(&LiquidationInput { side: "short".into(), ..base() });
        assert!(!d.is_long);
        assert!(close(d.liquidation_price_usd, 32_850.0));
        assert!(close(d.distance_pct, 9.5));
    }

    #[test]
    fn higher_leverage_closer_liquidation() {
        let d = generate(&LiquidationInput { leverage: 50.0, ..base() });
        // 30000 × (1 − 0.02 + 0.005) = 29550.
        assert!(close(d.liquidation_price_usd, 29_550.0));
        assert!(d.distance_pct < 9.5);
    }

    #[test]
    fn invalid_leverage() {
        let d = generate(&LiquidationInput { leverage: 0.0, ..base() });
        assert!(!d.valid);
    }
}
