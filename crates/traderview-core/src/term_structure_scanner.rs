//! Options term-structure scanner.
//!
//! For each symbol, takes its current options-expiration IV curve
//! (near → far) and flags:
//!   - **Inverted** (backwardation): near IV ≥ far IV by ≥ `min_inversion`.
//!     Classic event-pricing signature — near-dated options are pricing
//!     more vol because of an upcoming catalyst (earnings, FOMC, etc).
//!   - **Steep contango**: far IV ≥ near IV by ≥ `min_steepness`.
//!     The "buy near, sell far" calendar setup looks attractive.
//!   - **Flat**: max IV − min IV < `flat_threshold`.
//!     No meaningful term-structure edge.
//!
//! Caller supplies per-symbol expiration curves (smallest DTE first).
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpiryIv {
    pub days_to_expiry: i64,
    /// ATM IV at this expiration (e.g. 0.30 = 30%).
    pub atm_iv: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolTermStructure {
    pub symbol: String,
    /// Expirations sorted ascending by DTE.
    pub curve: Vec<ExpiryIv>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TermStructureShape {
    Inverted,
    Contango,
    Flat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerConfig {
    pub min_inversion: f64,
    pub min_steepness: f64,
    pub flat_threshold: f64,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            min_inversion: 0.02,
            min_steepness: 0.02,
            flat_threshold: 0.01,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TermStructureEntry {
    pub symbol: String,
    pub shape: TermStructureShape,
    pub near_iv: f64,
    pub far_iv: f64,
    pub iv_spread: f64, // near − far (positive = inverted)
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TermStructureReport {
    pub entries: Vec<TermStructureEntry>,
    pub inverted: Vec<String>,
    pub contango: Vec<String>,
    pub flat: Vec<String>,
}

pub fn analyze(universe: &[SymbolTermStructure], cfg: &ScannerConfig) -> TermStructureReport {
    let mut report = TermStructureReport::default();
    for sym in universe {
        if sym.curve.len() < 2 {
            continue;
        }
        let near = &sym.curve[0];
        let far = sym.curve.last().expect("non-empty");
        if !near.atm_iv.is_finite() || !far.atm_iv.is_finite() {
            continue;
        }
        let spread = near.atm_iv - far.atm_iv;
        // Compute range across the curve for the flat check.
        let mut min_iv = f64::INFINITY;
        let mut max_iv = f64::NEG_INFINITY;
        for p in &sym.curve {
            if !p.atm_iv.is_finite() {
                continue;
            }
            if p.atm_iv < min_iv {
                min_iv = p.atm_iv;
            }
            if p.atm_iv > max_iv {
                max_iv = p.atm_iv;
            }
        }
        let range = max_iv - min_iv;
        let shape = if range < cfg.flat_threshold {
            TermStructureShape::Flat
        } else if spread >= cfg.min_inversion {
            TermStructureShape::Inverted
        } else if -spread >= cfg.min_steepness {
            TermStructureShape::Contango
        } else {
            TermStructureShape::Flat
        };
        let entry = TermStructureEntry {
            symbol: sym.symbol.clone(),
            shape,
            near_iv: near.atm_iv,
            far_iv: far.atm_iv,
            iv_spread: spread,
        };
        match shape {
            TermStructureShape::Inverted => report.inverted.push(sym.symbol.clone()),
            TermStructureShape::Contango => report.contango.push(sym.symbol.clone()),
            TermStructureShape::Flat => report.flat.push(sym.symbol.clone()),
        }
        report.entries.push(entry);
    }
    // Sort by absolute spread so most-inverted/most-steep float to top.
    report.entries.sort_by(|a, b| {
        b.iv_spread
            .abs()
            .partial_cmp(&a.iv_spread.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exp(dte: i64, iv: f64) -> ExpiryIv {
        ExpiryIv {
            days_to_expiry: dte,
            atm_iv: iv,
        }
    }

    fn s(name: &str, curve: Vec<ExpiryIv>) -> SymbolTermStructure {
        SymbolTermStructure {
            symbol: name.into(),
            curve,
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[], &ScannerConfig::default());
        assert!(r.entries.is_empty());
    }

    #[test]
    fn single_expiry_symbol_skipped() {
        let u = vec![s("AAPL", vec![exp(7, 0.3)])];
        let r = analyze(&u, &ScannerConfig::default());
        assert!(r.entries.is_empty());
    }

    #[test]
    fn inverted_curve_flagged() {
        // Near IV 40%, far IV 25% → inverted by 15pts.
        let u = vec![s("EVENT", vec![exp(7, 0.40), exp(30, 0.30), exp(90, 0.25)])];
        let r = analyze(&u, &ScannerConfig::default());
        assert_eq!(r.entries.len(), 1);
        assert_eq!(r.entries[0].shape, TermStructureShape::Inverted);
        assert!(r.inverted.contains(&"EVENT".to_string()));
    }

    #[test]
    fn contango_curve_flagged() {
        let u = vec![s("CALM", vec![exp(7, 0.20), exp(30, 0.25), exp(90, 0.35)])];
        let r = analyze(&u, &ScannerConfig::default());
        assert_eq!(r.entries[0].shape, TermStructureShape::Contango);
        assert!(r.contango.contains(&"CALM".to_string()));
    }

    #[test]
    fn flat_curve_flagged() {
        let u = vec![s("FLAT", vec![exp(7, 0.30), exp(30, 0.30), exp(90, 0.305)])];
        let r = analyze(&u, &ScannerConfig::default());
        assert_eq!(r.entries[0].shape, TermStructureShape::Flat);
        assert!(r.flat.contains(&"FLAT".to_string()));
    }

    #[test]
    fn nonfinite_iv_safely_skipped() {
        let u = vec![s("X", vec![exp(7, f64::NAN), exp(30, 0.3)])];
        let r = analyze(&u, &ScannerConfig::default());
        // Near IV is NaN → skip entry.
        assert!(r.entries.is_empty());
    }

    #[test]
    fn entries_sorted_by_absolute_spread_descending() {
        let u = vec![
            s("MILD", vec![exp(7, 0.32), exp(60, 0.30)]), // spread 0.02
            s("BIG", vec![exp(7, 0.50), exp(60, 0.20)]),  // spread 0.30
            s("SMALL", vec![exp(7, 0.31), exp(60, 0.30)]), // spread 0.01
        ];
        let r = analyze(&u, &ScannerConfig::default());
        assert_eq!(r.entries[0].symbol, "BIG");
        // Order of MILD vs SMALL: both small, but absolute spread sort means MILD before SMALL.
        let positions: Vec<&str> = r.entries.iter().map(|e| e.symbol.as_str()).collect();
        let big_pos = positions.iter().position(|s| *s == "BIG").unwrap();
        let mild_pos = positions.iter().position(|s| *s == "MILD").unwrap();
        let small_pos = positions.iter().position(|s| *s == "SMALL").unwrap();
        assert!(big_pos < mild_pos && mild_pos < small_pos);
    }
}
