//! Gamma Pin Zone — strikes where dealer gamma exposure (GEX) reverses
//! sign, plus the consolidated "pin" strike most likely to anchor price
//! into expiration.
//!
//! Dealer GEX per strike = Σ (option gamma · OI · contract_multiplier),
//! sign-flipped for puts (dealers are short gamma in puts unless they
//! hedge). Total exposure as price moves up or down across strikes
//! determines whether dealers must BUY (positive gamma) or SELL
//! (negative gamma) to hedge, suppressing or amplifying moves.
//!
//! Key levels returned:
//!   - gamma_flip: the strike at which cumulative GEX crosses zero
//!     (price above → suppressive; below → amplifying)
//!   - pin_strike: the strike with the highest absolute gamma magnitude
//!     and current spot within `pin_radius_pct` of it
//!
//! Pure compute. Companion to `gex`, `gex_scanner`,
//! `unusual_options_activity`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StrikeGex {
    pub strike: f64,
    /// Net dealer gamma exposure at this strike (positive or negative).
    pub gex: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GammaPinReport {
    pub gamma_flip: Option<f64>,
    pub pin_strike: Option<f64>,
    pub pin_strength: Option<f64>,
    pub total_gex: f64,
    pub n_strikes: usize,
}

pub fn compute(
    strike_gex: &[StrikeGex],
    spot: f64,
    pin_radius_pct: f64,
) -> Option<GammaPinReport> {
    if strike_gex.is_empty() || !spot.is_finite() || spot <= 0.0
        || !pin_radius_pct.is_finite() || pin_radius_pct <= 0.0 {
        return None;
    }
    if strike_gex.iter().any(|s| !s.strike.is_finite() || s.strike <= 0.0 || !s.gex.is_finite()) {
        return None;
    }
    let mut sorted: Vec<StrikeGex> = strike_gex.to_vec();
    sorted.sort_by(|a, b| a.strike.partial_cmp(&b.strike).unwrap_or(std::cmp::Ordering::Equal));
    let total_gex: f64 = sorted.iter().map(|s| s.gex).sum();
    // Gamma flip: strike where cumulative GEX from low to high crosses zero.
    let mut cum = 0.0_f64;
    let mut flip: Option<f64> = None;
    let mut prev_strike: Option<f64> = None;
    let mut prev_cum;
    for s in &sorted {
        prev_cum = cum;
        cum += s.gex;
        if let Some(ps) = prev_strike {
            if prev_cum.signum() != cum.signum() && prev_cum != 0.0 && cum != 0.0 {
                // Linear interpolation between ps and s.strike.
                let span = s.strike - ps;
                let cross_offset = -prev_cum / (cum - prev_cum);
                flip = Some(ps + cross_offset * span);
                break;
            }
        }
        prev_strike = Some(s.strike);
    }
    // Pin strike: largest |gex| near spot (within pin_radius_pct of spot).
    let radius = spot * pin_radius_pct / 100.0;
    let near_spot: Vec<&StrikeGex> = sorted.iter()
        .filter(|s| (s.strike - spot).abs() <= radius)
        .collect();
    let (pin_strike, pin_strength) = if let Some(top) = near_spot.iter()
        .max_by(|a, b| a.gex.abs().partial_cmp(&b.gex.abs())
            .unwrap_or(std::cmp::Ordering::Equal))
    {
        (Some(top.strike), Some(top.gex.abs()))
    } else {
        (None, None)
    };
    Some(GammaPinReport {
        gamma_flip: flip,
        pin_strike,
        pin_strength,
        total_gex,
        n_strikes: sorted.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(strike: f64, gex: f64) -> StrikeGex { StrikeGex { strike, gex } }

    #[test]
    fn empty_or_invalid_returns_none() {
        assert!(compute(&[], 100.0, 5.0).is_none());
        assert!(compute(&[s(100.0, 1.0)], 0.0, 5.0).is_none());
        assert!(compute(&[s(100.0, 1.0)], 100.0, 0.0).is_none());
    }

    #[test]
    fn nan_returns_none() {
        let strikes = vec![s(100.0, f64::NAN)];
        assert!(compute(&strikes, 100.0, 5.0).is_none());
    }

    #[test]
    fn flip_strike_detected() {
        // Cumulative GEX must actually cross zero. Sequence:
        //   90: -3, 95: -5, 100: -4, 105: +1 → flip between 100 and 105.
        let strikes = vec![
            s(90.0, -3.0),
            s(95.0, -2.0),
            s(100.0, 1.0),
            s(105.0, 5.0),
        ];
        let r = compute(&strikes, 100.0, 5.0).unwrap();
        assert!(r.gamma_flip.is_some());
        let flip = r.gamma_flip.unwrap();
        assert!((100.0..=105.0).contains(&flip),
            "flip strike {flip} should be between 100 and 105");
    }

    #[test]
    fn pin_strike_is_largest_magnitude_near_spot() {
        let strikes = vec![
            s(95.0, -2.0),
            s(100.0, 5.0),     // largest |gex| within radius
            s(105.0, -1.0),
            s(150.0, 100.0),   // largest overall but FAR from spot
        ];
        let r = compute(&strikes, 100.0, 10.0).unwrap();
        assert_eq!(r.pin_strike, Some(100.0));
        assert!((r.pin_strength.unwrap() - 5.0).abs() < 1e-9);
    }

    #[test]
    fn no_pin_when_no_strikes_within_radius() {
        let strikes = vec![s(50.0, 10.0), s(200.0, 10.0)];
        let r = compute(&strikes, 100.0, 5.0).unwrap();
        assert!(r.pin_strike.is_none());
    }

    #[test]
    fn total_gex_sums_all_strikes() {
        let strikes = vec![s(95.0, -2.0), s(100.0, 5.0), s(105.0, -1.0)];
        let r = compute(&strikes, 100.0, 10.0).unwrap();
        assert!((r.total_gex - 2.0).abs() < 1e-9);
        assert_eq!(r.n_strikes, 3);
    }
}
