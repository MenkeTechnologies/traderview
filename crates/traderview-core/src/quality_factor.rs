//! Cross-sectional Quality factor — composite of ROE, low debt, low
//! earnings volatility.
//!
//! Per-symbol quality score = weighted average of three z-scored
//! components (positive scores = higher quality):
//!   - **ROE z-score**: higher return-on-equity = better
//!   - **negative debt-to-equity z-score**: lower leverage = better
//!   - **negative earnings-volatility z-score**: more stable earnings = better
//!
//! Cross-sectionally rank into deciles → QualityDecile (top 10%) /
//! JunkDecile (bottom 10%). Used in Asness-Frazzini-Pedersen "Quality
//! Minus Junk" (QMJ) factor construction.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolQualityInputs {
    pub symbol: String,
    pub return_on_equity: f64,    // e.g. 0.15 = 15%
    pub debt_to_equity: f64,      // 0.0 = no debt
    pub earnings_volatility: f64, // stdev of trailing EPS
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QualityBucket {
    QualityDecile,
    JunkDecile,
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityHit {
    pub symbol: String,
    pub quality_score: f64,
    pub percentile: f64,
    pub bucket: QualityBucket,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QualityFactorReport {
    pub quality_decile: Vec<QualityHit>,
    pub junk_decile: Vec<QualityHit>,
    pub all_ranked: Vec<QualityHit>,
}

pub fn scan(symbols: &[SymbolQualityInputs]) -> Option<QualityFactorReport> {
    if symbols.is_empty() {
        return None;
    }
    let valid: Vec<&SymbolQualityInputs> = symbols
        .iter()
        .filter(|s| {
            s.return_on_equity.is_finite()
                && s.debt_to_equity.is_finite()
                && s.earnings_volatility.is_finite()
                && s.debt_to_equity >= 0.0
                && s.earnings_volatility >= 0.0
        })
        .collect();
    if valid.is_empty() {
        return None;
    }
    let n = valid.len();
    // Compute mean + stdev for each component.
    let mean_stdev = |getter: &dyn Fn(&SymbolQualityInputs) -> f64| -> Option<(f64, f64)> {
        let vals: Vec<f64> = valid.iter().map(|s| getter(s)).collect();
        let mean: f64 = vals.iter().sum::<f64>() / n as f64;
        let var: f64 = vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
        let stdev = var.max(0.0).sqrt();
        if stdev > 0.0 {
            Some((mean, stdev))
        } else {
            None
        }
    };
    let (roe_mean, roe_sd) = mean_stdev(&|s| s.return_on_equity)?;
    let (dte_mean, dte_sd) = mean_stdev(&|s| s.debt_to_equity)?;
    let (ev_mean, ev_sd) = mean_stdev(&|s| s.earnings_volatility)?;
    let z = |x: f64, mu: f64, sd: f64| (x - mu) / sd;
    let mut all_ranked: Vec<QualityHit> = valid
        .iter()
        .map(|s| {
            let z_roe = z(s.return_on_equity, roe_mean, roe_sd);
            let z_dte = z(s.debt_to_equity, dte_mean, dte_sd);
            let z_ev = z(s.earnings_volatility, ev_mean, ev_sd);
            // Higher score = better. ROE z positive is good; low leverage and
            // low earnings vol are good (so we subtract).
            let score = z_roe - z_dte - z_ev;
            QualityHit {
                symbol: s.symbol.clone(),
                quality_score: score,
                percentile: 0.0,
                bucket: QualityBucket::Neutral,
            }
        })
        .collect();
    all_ranked.sort_by(|a, b| {
        a.quality_score
            .partial_cmp(&b.quality_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    for (i, h) in all_ranked.iter_mut().enumerate() {
        h.percentile = (i + 1) as f64 / n as f64 * 100.0;
        h.bucket = if h.percentile >= 90.0 {
            QualityBucket::QualityDecile
        } else if h.percentile <= 10.0 {
            QualityBucket::JunkDecile
        } else {
            QualityBucket::Neutral
        };
    }
    all_ranked.sort_by(|a, b| {
        b.quality_score
            .partial_cmp(&a.quality_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let quality_decile: Vec<QualityHit> = all_ranked
        .iter()
        .filter(|h| h.bucket == QualityBucket::QualityDecile)
        .cloned()
        .collect();
    let junk_decile: Vec<QualityHit> = all_ranked
        .iter()
        .filter(|h| h.bucket == QualityBucket::JunkDecile)
        .cloned()
        .collect();
    Some(QualityFactorReport {
        quality_decile,
        junk_decile,
        all_ranked,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(sym: &str, roe: f64, dte: f64, ev: f64) -> SymbolQualityInputs {
        SymbolQualityInputs {
            symbol: sym.into(),
            return_on_equity: roe,
            debt_to_equity: dte,
            earnings_volatility: ev,
        }
    }

    #[test]
    fn empty_returns_none() {
        assert!(scan(&[]).is_none());
    }

    #[test]
    fn invalid_inputs_filtered() {
        let symbols = vec![
            s("BAD_DTE", 0.15, -1.0, 0.05),
            s("BAD_EV", 0.15, 0.5, -0.05),
            s("NAN_ROE", f64::NAN, 0.5, 0.05),
        ];
        assert!(scan(&symbols).is_none());
    }

    #[test]
    fn high_quality_company_ranked_first() {
        // 10 symbols with steadily increasing quality across all 3 metrics.
        let symbols: Vec<_> = (1..=10)
            .map(|i| {
                let roe = 0.05 + (i as f64) * 0.02; // 7% → 25%
                let dte = 1.5 - (i as f64) * 0.10; // 1.4 → 0.5
                let ev = 0.20 - (i as f64) * 0.015; // 0.185 → 0.05
                s(&format!("Q{i}"), roe, dte, ev)
            })
            .collect();
        let r = scan(&symbols).unwrap();
        // Q10 has highest ROE, lowest debt, lowest earnings vol → top quality.
        assert_eq!(r.all_ranked[0].symbol, "Q10");
        assert!(r.quality_decile.iter().any(|h| h.symbol == "Q10"));
    }

    #[test]
    fn low_quality_company_in_junk_decile() {
        let symbols: Vec<_> = (1..=10)
            .map(|i| {
                let roe = 0.05 + (i as f64) * 0.02;
                let dte = 1.5 - (i as f64) * 0.10;
                let ev = 0.20 - (i as f64) * 0.015;
                s(&format!("Q{i}"), roe, dte, ev)
            })
            .collect();
        let r = scan(&symbols).unwrap();
        assert!(r.junk_decile.iter().any(|h| h.symbol == "Q1"));
    }

    #[test]
    fn percentiles_in_unit_range() {
        // All three components must vary across the universe; otherwise
        // a per-component stdev = 0 makes z-scores undefined and the
        // scanner correctly returns None (covered by the all-same test
        // below).
        let symbols: Vec<_> = (1..=20)
            .map(|i| {
                s(
                    &format!("S{i:02}"),
                    0.10 + i as f64 * 0.01,
                    1.5 - i as f64 * 0.05,
                    0.20 - i as f64 * 0.005,
                )
            })
            .collect();
        let r = scan(&symbols).unwrap();
        for h in &r.all_ranked {
            assert!((0.0..=100.0).contains(&h.percentile));
        }
    }

    #[test]
    fn all_same_quality_uniform_returns_none() {
        // Every symbol identical → all stdevs = 0 → can't compute z-scores.
        let symbols = vec![s("X", 0.15, 0.5, 0.05); 10];
        assert!(scan(&symbols).is_none());
    }

    #[test]
    fn ranked_descending_by_quality_score() {
        let symbols: Vec<_> = (1..=15)
            .map(|i| {
                s(
                    &format!("S{i:02}"),
                    0.05 + i as f64 * 0.02,
                    1.0 - i as f64 * 0.05,
                    0.1,
                )
            })
            .collect();
        let r = scan(&symbols).unwrap();
        for w in r.all_ranked.windows(2) {
            assert!(w[0].quality_score >= w[1].quality_score - 1e-9);
        }
    }
}
