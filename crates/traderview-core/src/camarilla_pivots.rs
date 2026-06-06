//! Camarilla Pivot Points — Nick Stott.
//!
//! Eight intraday support/resistance levels derived from the prior
//! session's high/low/close, with constants tuned for tight
//! mean-reversion bands:
//!
//!   range = high - low
//!   H4 = close + range · 1.1 / 2
//!   H3 = close + range · 1.1 / 4
//!   H2 = close + range · 1.1 / 6
//!   H1 = close + range · 1.1 / 12
//!   L1 = close - range · 1.1 / 12
//!   L2 = close - range · 1.1 / 6
//!   L3 = close - range · 1.1 / 4
//!   L4 = close - range · 1.1 / 2
//!
//! Trading rules (Camarilla equation):
//!   - L3/H3 are common reversal levels (long L3, short H3)
//!   - L4/H4 are breakout levels (long break of H4, short break of L4)
//!
//! Pure compute. Companion to `pivot_points`, `murrey_math`, `round_levels`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PriorSession {
    pub high: f64,
    pub low: f64,
    pub close: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub struct CamarillaLevels {
    pub h4: f64,
    pub h3: f64,
    pub h2: f64,
    pub h1: f64,
    pub pivot: f64,
    pub l1: f64,
    pub l2: f64,
    pub l3: f64,
    pub l4: f64,
}

pub fn compute(session: PriorSession) -> Option<CamarillaLevels> {
    if !session.high.is_finite()
        || !session.low.is_finite()
        || !session.close.is_finite()
        || session.high < session.low
    {
        return None;
    }
    let range = session.high - session.low;
    let k = range * 1.1;
    let pivot = (session.high + session.low + session.close) / 3.0;
    Some(CamarillaLevels {
        h4: session.close + k / 2.0,
        h3: session.close + k / 4.0,
        h2: session.close + k / 6.0,
        h1: session.close + k / 12.0,
        pivot,
        l1: session.close - k / 12.0,
        l2: session.close - k / 6.0,
        l3: session.close - k / 4.0,
        l4: session.close - k / 2.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_session_returns_none() {
        assert!(compute(PriorSession {
            high: f64::NAN,
            low: 99.0,
            close: 100.0
        })
        .is_none());
        assert!(compute(PriorSession {
            high: 99.0,
            low: 101.0,
            close: 100.0
        })
        .is_none());
    }

    #[test]
    fn levels_centered_around_close() {
        let r = compute(PriorSession {
            high: 110.0,
            low: 100.0,
            close: 105.0,
        })
        .unwrap();
        // Symmetric around close.
        assert!((r.h1 - 105.0 - (105.0 - r.l1)).abs() < 1e-9);
        assert!((r.h4 - 105.0 - (105.0 - r.l4)).abs() < 1e-9);
        // H1 < H2 < H3 < H4.
        assert!(r.h1 < r.h2 && r.h2 < r.h3 && r.h3 < r.h4);
        assert!(r.l1 > r.l2 && r.l2 > r.l3 && r.l3 > r.l4);
    }

    #[test]
    fn exact_formula_values() {
        // range = 10, k = 11.
        let r = compute(PriorSession {
            high: 110.0,
            low: 100.0,
            close: 105.0,
        })
        .unwrap();
        assert!((r.h4 - (105.0 + 11.0 / 2.0)).abs() < 1e-9);
        assert!((r.h3 - (105.0 + 11.0 / 4.0)).abs() < 1e-9);
        assert!((r.l3 - (105.0 - 11.0 / 4.0)).abs() < 1e-9);
        assert!((r.l4 - (105.0 - 11.0 / 2.0)).abs() < 1e-9);
    }

    #[test]
    fn pivot_is_typical_price() {
        let r = compute(PriorSession {
            high: 110.0,
            low: 100.0,
            close: 108.0,
        })
        .unwrap();
        assert!((r.pivot - (110.0 + 100.0 + 108.0) / 3.0).abs() < 1e-9);
    }

    #[test]
    fn zero_range_collapses_to_close() {
        let r = compute(PriorSession {
            high: 100.0,
            low: 100.0,
            close: 100.0,
        })
        .unwrap();
        for lvl in [r.h4, r.h3, r.h2, r.h1, r.l1, r.l2, r.l3, r.l4] {
            assert!((lvl - 100.0).abs() < 1e-9);
        }
    }
}
