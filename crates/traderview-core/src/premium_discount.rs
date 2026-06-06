//! Premium/discount zone classifier — SMC range-position helper.
//!
//! Given an established range (swing high to swing low), price location
//! within that range tells the SMC trader whether they're buying at
//! "premium" (top 50%) or "discount" (bottom 50%). Combined with the
//! prevailing trend, the rule is: long only at discount in uptrend,
//! short only at premium in downtrend.
//!
//! Caller supplies the range high/low and the current price (plus an
//! optional current trend bias for full verdict). Outputs:
//!   - Zone classification (Premium / Equilibrium / Discount)
//!   - Position-in-range as fraction
//!   - Verdict (Buy / Sell / Wait) when trend bias is supplied
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendBias {
    Up,
    Down,
    Neutral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Zone {
    Premium,
    #[default]
    Equilibrium,
    Discount,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    /// Long-only edge: discount price in uptrend.
    Buy,
    /// Short-only edge: premium price in downtrend.
    Sell,
    #[default]
    Wait,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ZoneReport {
    pub zone: Zone,
    pub position_pct: f64,
    pub verdict: Verdict,
    pub midpoint: f64,
    pub note: String,
}

pub fn classify(range_high: f64, range_low: f64, price: f64, trend: TrendBias) -> ZoneReport {
    if !range_high.is_finite() || !range_low.is_finite() || range_high <= range_low {
        return ZoneReport {
            note: "invalid range".into(),
            ..Default::default()
        };
    }
    let midpoint = (range_high + range_low) / 2.0;
    let position = (price - range_low) / (range_high - range_low);
    // Strict bands: top 30% = premium, bottom 30% = discount, middle = equilibrium.
    let zone = if position >= 0.70 {
        Zone::Premium
    } else if position <= 0.30 {
        Zone::Discount
    } else {
        Zone::Equilibrium
    };
    let verdict = match (trend, zone) {
        (TrendBias::Up, Zone::Discount) => Verdict::Buy,
        (TrendBias::Down, Zone::Premium) => Verdict::Sell,
        _ => Verdict::Wait,
    };
    let note = match (zone, verdict) {
        (Zone::Discount, Verdict::Buy) => format!("discount {position:.0}% in uptrend — long bias"),
        (Zone::Premium, Verdict::Sell) => {
            format!("premium {position:.0}% in downtrend — short bias")
        }
        (Zone::Equilibrium, _) => format!("equilibrium {position:.0}% — wait for premium/discount"),
        _ => format!("{:?} zone in {:?} trend — no edge", zone, trend),
    };
    ZoneReport {
        zone,
        position_pct: position * 100.0,
        verdict,
        midpoint,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_range_returns_default() {
        let r = classify(100.0, 110.0, 105.0, TrendBias::Up);
        assert!(matches!(r.zone, Zone::Equilibrium));
        assert!(r.note.contains("invalid"));
    }

    #[test]
    fn discount_zone_in_uptrend_signals_buy() {
        // Range 100..110, price 102 (20% in range) = discount.
        let r = classify(110.0, 100.0, 102.0, TrendBias::Up);
        assert!(matches!(r.zone, Zone::Discount));
        assert!(matches!(r.verdict, Verdict::Buy));
    }

    #[test]
    fn premium_zone_in_downtrend_signals_sell() {
        let r = classify(110.0, 100.0, 108.0, TrendBias::Down);
        assert!(matches!(r.zone, Zone::Premium));
        assert!(matches!(r.verdict, Verdict::Sell));
    }

    #[test]
    fn premium_in_uptrend_is_wait() {
        let r = classify(110.0, 100.0, 108.0, TrendBias::Up);
        assert!(matches!(r.zone, Zone::Premium));
        assert!(matches!(r.verdict, Verdict::Wait));
    }

    #[test]
    fn discount_in_downtrend_is_wait() {
        let r = classify(110.0, 100.0, 102.0, TrendBias::Down);
        assert!(matches!(r.zone, Zone::Discount));
        assert!(matches!(r.verdict, Verdict::Wait));
    }

    #[test]
    fn equilibrium_zone_always_waits() {
        let r = classify(110.0, 100.0, 105.0, TrendBias::Up);
        assert!(matches!(r.zone, Zone::Equilibrium));
        assert!(matches!(r.verdict, Verdict::Wait));
    }

    #[test]
    fn midpoint_computed_correctly() {
        let r = classify(110.0, 100.0, 105.0, TrendBias::Neutral);
        assert!((r.midpoint - 105.0).abs() < 1e-9);
    }
}
