//! Volatility-Targeting Position Sizer.
//!
//! Adjusts position size to maintain a target annualized portfolio
//! volatility, given a forecast of the asset's annualized volatility:
//!
//!   raw_leverage = target_vol / forecast_vol
//!   scaled_leverage = min(raw_leverage, max_leverage)
//!   position_size = scaled_leverage · base_capital / asset_price
//!
//! When forecast_vol is small (calm regime), leverage rises to hit the
//! target. When forecast_vol spikes (vol regime), leverage falls,
//! reducing realized portfolio vol toward the target.
//!
//! Used by:
//!   - Risk-parity strategies (target equal vol contribution per asset)
//!   - Managed-vol funds (target constant portfolio vol)
//!   - Single-strategy sizing (target constant strategy vol)
//!
//! Pure compute. Optional vol-of-vol smoothing applied as an EWMA
//! when prior leverage is supplied (reduces position whiplash during
//! transient vol spikes).
//!
//! Companion to `risk_parity_weights`, `ewma_volatility`,
//! `realized_volatility`, `garch_1_1`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VolTargetSizerReport {
    pub raw_leverage: f64,
    pub clipped_leverage: f64,
    pub smoothed_leverage: f64,
    pub position_size_shares: f64,
    pub position_notional: f64,
    pub implied_portfolio_vol: f64,
}

pub fn size(
    target_annualized_vol: f64,
    forecast_annualized_vol: f64,
    asset_price: f64,
    base_capital: f64,
    max_leverage: f64,
    prior_leverage: Option<f64>,
    smoothing_alpha: f64,
) -> Option<VolTargetSizerReport> {
    if !target_annualized_vol.is_finite()
        || target_annualized_vol <= 0.0
        || !forecast_annualized_vol.is_finite()
        || forecast_annualized_vol <= 0.0
        || !asset_price.is_finite()
        || asset_price <= 0.0
        || !base_capital.is_finite()
        || base_capital <= 0.0
        || !max_leverage.is_finite()
        || max_leverage <= 0.0
        || !smoothing_alpha.is_finite()
        || !(0.0..=1.0).contains(&smoothing_alpha)
    {
        return None;
    }
    let raw = target_annualized_vol / forecast_annualized_vol;
    let clipped = raw.min(max_leverage);
    let smoothed = match prior_leverage {
        Some(p) if p.is_finite() && p > 0.0 => {
            smoothing_alpha * clipped + (1.0 - smoothing_alpha) * p
        }
        _ => clipped,
    };
    let notional = smoothed * base_capital;
    let shares = notional / asset_price;
    let implied_vol = smoothed * forecast_annualized_vol;
    Some(VolTargetSizerReport {
        raw_leverage: raw,
        clipped_leverage: clipped,
        smoothed_leverage: smoothed,
        position_size_shares: shares,
        position_notional: notional,
        implied_portfolio_vol: implied_vol,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_none() {
        assert!(size(0.0, 0.20, 100.0, 1_000_000.0, 4.0, None, 0.5).is_none());
        assert!(size(0.15, 0.0, 100.0, 1_000_000.0, 4.0, None, 0.5).is_none());
        assert!(size(0.15, 0.20, 0.0, 1_000_000.0, 4.0, None, 0.5).is_none());
        assert!(size(0.15, 0.20, 100.0, 0.0, 4.0, None, 0.5).is_none());
        assert!(size(0.15, 0.20, 100.0, 1_000_000.0, 0.0, None, 0.5).is_none());
        assert!(size(0.15, 0.20, 100.0, 1_000_000.0, 4.0, None, 1.5).is_none());
        assert!(size(0.15, 0.20, 100.0, 1_000_000.0, 4.0, None, f64::NAN).is_none());
    }

    #[test]
    fn target_equals_forecast_yields_leverage_one() {
        let r = size(0.15, 0.15, 100.0, 1_000_000.0, 4.0, None, 0.0).unwrap();
        assert!((r.raw_leverage - 1.0).abs() < 1e-12);
        assert!((r.clipped_leverage - 1.0).abs() < 1e-12);
    }

    #[test]
    fn low_vol_regime_raises_leverage() {
        // Forecast 5% vol, target 15% → 3x leverage.
        let r = size(0.15, 0.05, 100.0, 1_000_000.0, 4.0, None, 0.0).unwrap();
        assert!((r.raw_leverage - 3.0).abs() < 1e-12);
    }

    #[test]
    fn high_vol_regime_lowers_leverage() {
        // Forecast 30% vol, target 15% → 0.5x leverage.
        let r = size(0.15, 0.30, 100.0, 1_000_000.0, 4.0, None, 0.0).unwrap();
        assert!((r.raw_leverage - 0.5).abs() < 1e-12);
    }

    #[test]
    fn max_leverage_caps_low_vol_regime() {
        // Forecast 3% vol, target 15% → raw 5x, capped to 4x.
        let r = size(0.15, 0.03, 100.0, 1_000_000.0, 4.0, None, 0.0).unwrap();
        assert!((r.raw_leverage - 5.0).abs() < 1e-12);
        assert!((r.clipped_leverage - 4.0).abs() < 1e-12);
    }

    #[test]
    fn smoothing_dampens_leverage_swings() {
        // Prior 1x, current 3x (low-vol regime).
        // With α = 0.3: smoothed = 0.3·3 + 0.7·1 = 1.6.
        let r = size(0.15, 0.05, 100.0, 1_000_000.0, 4.0, Some(1.0), 0.3).unwrap();
        assert!((r.smoothed_leverage - 1.6).abs() < 1e-12);
    }

    #[test]
    fn full_smoothing_yields_prior_unchanged() {
        // α = 0 → fully ignore current signal.
        let r = size(0.15, 0.05, 100.0, 1_000_000.0, 4.0, Some(1.0), 0.0).unwrap();
        assert!((r.smoothed_leverage - 1.0).abs() < 1e-12);
    }

    #[test]
    fn shares_consistent_with_notional() {
        let r = size(0.15, 0.20, 100.0, 1_000_000.0, 4.0, None, 0.0).unwrap();
        assert!((r.position_size_shares * 100.0 - r.position_notional).abs() < 1e-6);
    }

    #[test]
    fn implied_vol_matches_target_when_not_capped() {
        let r = size(0.15, 0.20, 100.0, 1_000_000.0, 4.0, None, 0.0).unwrap();
        assert!((r.implied_portfolio_vol - 0.15).abs() < 1e-12);
    }

    #[test]
    fn implied_vol_below_target_when_capped() {
        let r = size(0.15, 0.03, 100.0, 1_000_000.0, 4.0, None, 0.0).unwrap();
        // Capped at 4x → implied vol = 4 · 3% = 12% < 15% target.
        assert!(r.implied_portfolio_vol < 0.15);
    }
}
