//! Portfolio concentration metrics.
//!
//! Standard measures of how concentrated (or diversified) an account is:
//!   - HHI (Herfindahl-Hirschman Index): sum of squared portfolio weights.
//!     1.0 = single position, 1/N for equal-weighted N positions.
//!   - Top-N share: % of gross exposure in the largest N positions.
//!   - Effective number of holdings: 1/HHI — the equivalent count of
//!     equal-weighted positions. A 10-position portfolio with HHI 0.40
//!     has effective N ≈ 2.5 (acts like 2-3 equal positions concentration-wise).
//!
//! Pure compute. Caller passes per-position absolute notional values
//! (signed positions don't make sense for concentration — a long $50k
//! and a short $50k are TWO bets, not zero).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Holding {
    pub symbol: String,
    /// Absolute exposure in dollars (longs and shorts both positive).
    pub abs_notional: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConcentrationReport {
    pub n_positions: usize,
    pub total_gross_exposure: f64,
    /// Herfindahl-Hirschman Index, 0..=1. Higher = more concentrated.
    pub hhi: f64,
    /// 1/HHI — equivalent equal-weighted N.
    pub effective_n: f64,
    /// Largest position as fraction of gross.
    pub top_1_pct: f64,
    pub top_3_pct: f64,
    pub top_5_pct: f64,
    pub top_10_pct: f64,
    /// True if any position > 25% of gross (Reg-T-style concentration flag).
    pub flag_single_position_over_25pct: bool,
}

pub fn evaluate(holdings: &[Holding]) -> ConcentrationReport {
    let total: f64 = holdings.iter().map(|h| h.abs_notional).sum();
    if total <= 0.0 || holdings.is_empty() {
        return ConcentrationReport::default();
    }
    let weights: Vec<f64> = holdings.iter().map(|h| h.abs_notional / total).collect();
    let hhi: f64 = weights.iter().map(|w| w * w).sum();
    let effective_n = if hhi > 0.0 { 1.0 / hhi } else { 0.0 };
    let mut sorted = weights.clone();
    sorted.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    ConcentrationReport {
        n_positions: holdings.len(),
        total_gross_exposure: total,
        hhi,
        effective_n,
        top_1_pct: sorted.iter().take(1).sum(),
        top_3_pct: sorted.iter().take(3).sum(),
        top_5_pct: sorted.iter().take(5).sum(),
        top_10_pct: sorted.iter().take(10).sum(),
        flag_single_position_over_25pct: sorted.first().is_some_and(|w| *w > 0.25),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn h(sym: &str, n: f64) -> Holding {
        Holding {
            symbol: sym.into(),
            abs_notional: n,
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = evaluate(&[]);
        assert_eq!(r.n_positions, 0);
        assert_eq!(r.hhi, 0.0);
        assert_eq!(r.effective_n, 0.0);
    }

    #[test]
    fn single_position_hhi_is_one() {
        let r = evaluate(&[h("AAPL", 10_000.0)]);
        assert_eq!(r.hhi, 1.0);
        assert_eq!(r.effective_n, 1.0);
        assert_eq!(r.top_1_pct, 1.0);
        assert!(r.flag_single_position_over_25pct);
    }

    #[test]
    fn ten_equal_positions_hhi_is_one_tenth() {
        let holdings: Vec<_> = (0..10).map(|i| h(&format!("S{i}"), 1000.0)).collect();
        let r = evaluate(&holdings);
        assert!((r.hhi - 0.1).abs() < 1e-12);
        assert!((r.effective_n - 10.0).abs() < 1e-9);
        assert!((r.top_1_pct - 0.1).abs() < 1e-12);
        assert!((r.top_3_pct - 0.3).abs() < 1e-12);
        assert!(!r.flag_single_position_over_25pct);
    }

    #[test]
    fn concentrated_portfolio_flags_25pct() {
        // 40% in one name + 6 equal-weighted at 10% each = 100%.
        let mut holdings = vec![h("BIG", 4_000.0)];
        for i in 0..6 {
            holdings.push(h(&format!("S{i}"), 1000.0));
        }
        let r = evaluate(&holdings);
        assert!(r.flag_single_position_over_25pct);
        assert!((r.top_1_pct - 0.4).abs() < 1e-9);
    }

    #[test]
    fn top_n_caps_at_position_count() {
        // 3 positions, top_5 is just the 3 we have.
        let holdings = vec![h("A", 5000.0), h("B", 3000.0), h("C", 2000.0)];
        let r = evaluate(&holdings);
        assert!((r.top_5_pct - 1.0).abs() < 1e-12);
        assert!((r.top_10_pct - 1.0).abs() < 1e-12);
    }

    #[test]
    fn effective_n_inverse_of_hhi() {
        // 50/50 split → HHI = 0.5, effective_n = 2.
        let r = evaluate(&[h("A", 1.0), h("B", 1.0)]);
        assert!((r.hhi - 0.5).abs() < 1e-12);
        assert!((r.effective_n - 2.0).abs() < 1e-9);
    }

    #[test]
    fn top_positions_sorted_largest_first() {
        // Out-of-order input.
        let holdings = vec![h("SMALL", 1000.0), h("BIG", 9000.0)];
        let r = evaluate(&holdings);
        assert!((r.top_1_pct - 0.9).abs() < 1e-12);
    }

    #[test]
    fn zero_notional_holdings_return_default() {
        let holdings = vec![h("A", 0.0), h("B", 0.0)];
        let r = evaluate(&holdings);
        assert_eq!(r.hhi, 0.0);
    }
}
