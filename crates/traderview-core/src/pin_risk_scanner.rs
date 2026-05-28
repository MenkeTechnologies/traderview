//! Pin-Risk Scanner — flags option contracts near expiry whose strike
//! sits inside an ATM band where assignment risk is highest.
//!
//! "Pin risk" = the risk that an option finishes the session at-or-
//! very-near its strike, leaving the holder/writer uncertain whether
//! they'll get auto-exercised. Most acute in the final hour of expiry
//! day for short positions in deep open-interest strikes.
//!
//! Match criteria:
//!   - Days to expiry  ≤ max_days_to_expiry
//!   - |spot - strike| / spot · 100  ≤ atm_band_pct
//!   - open interest    ≥ min_open_interest
//!
//! Score = (1 - |spot - strike| / (spot · atm_band_pct/100))
//!         · log10(open_interest + 1)
//! Higher = closer to pin + thicker OI.
//!
//! Pure compute. Companion to `gamma_pin_zone`, `gamma_squeeze`,
//! `option_open_interest_distribution`.

#[derive(Clone, Debug)]
pub struct Contract {
    pub symbol: String,
    pub strike: f64,
    pub spot: f64,
    pub days_to_expiry: f64,
    pub open_interest: u64,
    pub kind: String,           // "call" / "put" — free-form, passed through
}

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub max_days_to_expiry: f64,
    pub atm_band_pct: f64,
    pub min_open_interest: u64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_days_to_expiry: 1.0,
            atm_band_pct: 1.0,
            min_open_interest: 500,
        }
    }
}

#[derive(Debug)]
pub struct Match {
    pub symbol: String,
    pub strike: f64,
    pub kind: String,
    pub distance_pct: f64,
    pub open_interest: u64,
    pub days_to_expiry: f64,
    pub score: f64,
}

pub fn scan(contracts: &[Contract], config: Config) -> Vec<Match> {
    let mut matches = Vec::new();
    if !config.max_days_to_expiry.is_finite() || config.max_days_to_expiry < 0.0 {
        return matches;
    }
    if !config.atm_band_pct.is_finite() || config.atm_band_pct <= 0.0 {
        return matches;
    }
    for c in contracts {
        if !c.strike.is_finite() || !c.spot.is_finite() || !c.days_to_expiry.is_finite() {
            continue;
        }
        if c.spot <= 0.0 || c.strike <= 0.0 { continue; }
        if c.days_to_expiry < 0.0 || c.days_to_expiry > config.max_days_to_expiry { continue; }
        if c.open_interest < config.min_open_interest { continue; }
        let distance_pct = (c.spot - c.strike).abs() / c.spot * 100.0;
        if distance_pct > config.atm_band_pct { continue; }
        let band_norm = distance_pct / config.atm_band_pct;
        let proximity = 1.0 - band_norm;
        let oi_factor = ((c.open_interest as f64) + 1.0).log10();
        let score = proximity * oi_factor;
        matches.push(Match {
            symbol: c.symbol.clone(),
            strike: c.strike,
            kind: c.kind.clone(),
            distance_pct,
            open_interest: c.open_interest,
            days_to_expiry: c.days_to_expiry,
            score,
        });
    }
    matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    fn c(sym: &str, k: f64, s: f64, dte: f64, oi: u64, kind: &str) -> Contract {
        Contract {
            symbol: sym.into(), strike: k, spot: s,
            days_to_expiry: dte, open_interest: oi, kind: kind.into(),
        }
    }

    #[test]
    fn empty_input_yields_no_matches() {
        let r = scan(&[], Config::default());
        assert!(r.is_empty());
    }

    #[test]
    fn at_money_with_high_oi_matches() {
        // spot 100, strike 100, expires today, 5k OI → strong pin candidate.
        let x = c("ABCD", 100.0, 100.0, 0.5, 5_000, "call");
        let r = scan(&[x], Config::default());
        assert_eq!(r.len(), 1);
        assert!(r[0].score > 0.0);
        assert!((r[0].distance_pct).abs() < 1e-9);
    }

    #[test]
    fn far_from_money_rejected() {
        // 5% away from spot at 1% band → reject.
        let x = c("ABCD", 105.0, 100.0, 0.5, 5_000, "call");
        let r = scan(&[x], Config::default());
        assert!(r.is_empty());
    }

    #[test]
    fn too_far_dte_rejected() {
        let x = c("ABCD", 100.0, 100.0, 30.0, 5_000, "call");
        let r = scan(&[x], Config::default());
        assert!(r.is_empty());
    }

    #[test]
    fn thin_oi_rejected() {
        let x = c("ABCD", 100.0, 100.0, 0.5, 100, "call");
        let r = scan(&[x], Config::default());
        assert!(r.is_empty());
    }

    #[test]
    fn invalid_fields_skip_contract() {
        let x1 = c("X", 0.0, 100.0, 0.5, 5_000, "call");
        let x2 = c("X", 100.0, 0.0, 0.5, 5_000, "call");
        let x3 = c("X", 100.0, 100.0, -1.0, 5_000, "call");
        let mut x4 = c("X", 100.0, 100.0, 0.5, 5_000, "call");
        x4.strike = f64::NAN;
        let r = scan(&[x1, x2, x3, x4], Config::default());
        assert!(r.is_empty());
    }

    #[test]
    fn matches_sorted_by_score_descending() {
        // Two pin candidates: one perfectly at strike + high OI, one slightly off + lower OI.
        let strong = c("STRONG", 100.0, 100.0, 0.1, 10_000, "call");
        let weak   = c("WEAK",   100.5, 100.0, 0.5, 1_000, "call");
        let r = scan(&[weak, strong], Config::default());
        assert_eq!(r.len(), 2);
        assert!(r[0].score >= r[1].score);
        assert_eq!(r[0].symbol, "STRONG");
    }

    #[test]
    fn match_carries_kind_string_through() {
        let put = c("ABCD", 100.0, 100.0, 0.5, 5_000, "put");
        let r = scan(&[put], Config::default());
        assert_eq!(r[0].kind, "put");
    }
}
