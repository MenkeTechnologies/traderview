//! Entry-correlation gate math — "don't stack the same trade."
//!
//! Five tech longs that move together are one position with five
//! commissions. Before a new entry, compare the candidate's daily
//! returns against every open position's; if any |ρ| exceeds the cap
//! the entry adds concentration, not diversification.
//!
//! Pure compute; Pearson reuses the shared correlation core. The
//! caller aligns the series by fetching the same lookback for every
//! symbol — series are compared over their common (trailing) overlap.

use crate::correlation::pearson;
use serde::Serialize;

/// Daily simple returns from a close series.
pub fn daily_returns(closes: &[f64]) -> Vec<f64> {
    closes
        .windows(2)
        .map(|w| if w[0] > 0.0 { w[1] / w[0] - 1.0 } else { 0.0 })
        .collect()
}

#[derive(Debug, Clone, Serialize)]
pub struct CorrelationHit {
    pub symbol: String,
    pub rho: f64,
}

/// The open position most correlated with the candidate, when its
/// |ρ| exceeds `cap`. Series shorter than `min_overlap` returns are
/// skipped — a correlation over a week of data is noise, and skipping
/// (rather than blocking) keeps a thin-history symbol tradeable.
pub fn worst_correlation(
    candidate_returns: &[f64],
    open_positions: &[(String, Vec<f64>)],
    cap: f64,
    min_overlap: usize,
) -> Option<CorrelationHit> {
    let mut worst: Option<CorrelationHit> = None;
    for (symbol, other) in open_positions {
        let n = candidate_returns.len().min(other.len());
        if n < min_overlap {
            continue;
        }
        // Compare the trailing overlap so both series cover the same days.
        let a = &candidate_returns[candidate_returns.len() - n..];
        let b = &other[other.len() - n..];
        let Some(rho) = pearson(a, b) else { continue };
        if rho.abs() > cap && worst.as_ref().is_none_or(|w| rho.abs() > w.rho.abs()) {
            worst = Some(CorrelationHit {
                symbol: symbol.clone(),
                rho,
            });
        }
    }
    worst
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_series_blocks_and_names_the_worst() {
        let base: Vec<f64> = (0..40).map(|i| (i as f64 * 0.7).sin() * 0.02).collect();
        let noisy: Vec<f64> = base.iter().map(|r| r * 0.5 + 0.001).collect(); // still ρ=1 shape
        let unrelated: Vec<f64> = (0..40).map(|i| (i as f64 * 2.3 + 9.0).cos() * 0.02).collect();
        let open = vec![
            ("WEAK".to_string(), unrelated),
            ("CLONE".to_string(), noisy),
        ];
        let hit = worst_correlation(&base, &open, 0.8, 20).expect("clone must trip the gate");
        assert_eq!(hit.symbol, "CLONE");
        assert!(hit.rho > 0.99);
    }

    #[test]
    fn anticorrelation_also_blocks() {
        // A perfect hedge is ALSO the same trade (mirrored) — |ρ| gates.
        let base: Vec<f64> = (0..40).map(|i| (i as f64 * 0.7).sin() * 0.02).collect();
        let inverse: Vec<f64> = base.iter().map(|r| -r).collect();
        let open = vec![("INV".to_string(), inverse)];
        let hit = worst_correlation(&base, &open, 0.8, 20).expect("inverse must trip");
        assert!(hit.rho < -0.99);
    }

    #[test]
    fn uncorrelated_and_thin_history_pass() {
        let base: Vec<f64> = (0..40).map(|i| (i as f64 * 0.7).sin() * 0.02).collect();
        let unrelated: Vec<f64> = (0..40).map(|i| (i as f64 * 2.3 + 9.0).cos() * 0.02).collect();
        assert!(worst_correlation(&base, &[("X".into(), unrelated)], 0.8, 20).is_none());
        // 5 days of overlap is noise — skipped, not blocked.
        let clone_short: Vec<f64> = base[..5].to_vec();
        assert!(worst_correlation(&base, &[("Y".into(), clone_short)], 0.8, 20).is_none());
    }

    #[test]
    fn returns_math_pins_simple_case() {
        let r = daily_returns(&[100.0, 110.0, 99.0]);
        assert!((r[0] - 0.10).abs() < 1e-12);
        assert!((r[1] + 0.10).abs() < 1e-12);
        assert_eq!(daily_returns(&[0.0, 5.0]), vec![0.0]);
    }
}
