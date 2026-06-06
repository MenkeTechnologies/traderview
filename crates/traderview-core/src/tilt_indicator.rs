//! Cohort tilt indicator — aggregate long/short positioning bias.
//!
//! TopstepX surfaces "The Tilt" — a real-time read of how all funded
//! traders at the firm are positioned across the major instruments
//! (ES, NQ, CL, GC). The cohort's net long/short bias is a sentiment
//! signal: when the room is heavily long ES, a contrarian read says
//! squeeze risk is elevated.
//!
//! This module is the pure-compute core. The caller supplies per-trader
//! positions (symbol + signed contract count); we emit per-symbol bias
//! plus the cohort-wide aggregate. Sized in contracts, not dollars —
//! the input is already directionally normalized so adding contracts
//! across traders is meaningful.
//!
//! Distinct from `crate::tilt_detector`, which detects an INDIVIDUAL
//! trader's emotional tilt over time. Same word, different concept.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraderPosition {
    /// Anonymized trader id — only the aggregate matters; this just
    /// prevents one big position from being counted twice.
    pub trader_id: String,
    pub symbol: String,
    /// Signed contracts: + long, - short, 0 = flat.
    pub net_contracts: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TiltBias {
    StronglyLong,
    Long,
    #[default]
    Balanced,
    Short,
    StronglyShort,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SymbolTilt {
    pub symbol: String,
    /// Traders net-long this symbol.
    pub long_traders: u32,
    /// Traders net-short this symbol.
    pub short_traders: u32,
    /// Traders flat (net_contracts == 0).
    pub flat_traders: u32,
    /// Sum of signed contracts across all traders.
    pub net_contracts: i64,
    /// long_traders / (long_traders + short_traders) as a fraction in
    /// `[0,1]`. None when no one is positioned (all flat).
    pub long_ratio: Option<f64>,
    /// Discrete classification of `long_ratio`.
    pub bias: TiltBias,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TiltReport {
    pub by_symbol: Vec<SymbolTilt>,
    /// Total unique traders with at least one non-flat position.
    pub active_traders: u32,
    /// Symbol with the most extreme `long_ratio` distance from 0.5.
    pub most_lopsided_symbol: Option<String>,
}

/// Discrete bias thresholds. Match TopstepX-style five-tier classification.
fn classify(long_ratio: f64) -> TiltBias {
    if long_ratio >= 0.75 {
        TiltBias::StronglyLong
    } else if long_ratio >= 0.60 {
        TiltBias::Long
    } else if long_ratio >= 0.40 {
        TiltBias::Balanced
    } else if long_ratio >= 0.25 {
        TiltBias::Short
    } else {
        TiltBias::StronglyShort
    }
}

pub fn aggregate(positions: &[TraderPosition]) -> TiltReport {
    if positions.is_empty() {
        return TiltReport::default();
    }
    let mut by_sym: BTreeMap<String, (u32, u32, u32, i64)> = BTreeMap::new();
    let mut active_set: std::collections::BTreeSet<&str> = Default::default();
    for p in positions {
        let entry = by_sym.entry(p.symbol.clone()).or_default();
        if p.net_contracts > 0 {
            entry.0 += 1;
            active_set.insert(&p.trader_id);
        } else if p.net_contracts < 0 {
            entry.1 += 1;
            active_set.insert(&p.trader_id);
        } else {
            entry.2 += 1;
        }
        entry.3 += p.net_contracts;
    }
    let mut by_symbol: Vec<SymbolTilt> = by_sym
        .into_iter()
        .map(|(symbol, (long_t, short_t, flat_t, net))| {
            let total_positioned = long_t + short_t;
            let long_ratio = if total_positioned == 0 {
                None
            } else {
                Some(long_t as f64 / total_positioned as f64)
            };
            let bias = long_ratio.map(classify).unwrap_or_default();
            SymbolTilt {
                symbol,
                long_traders: long_t,
                short_traders: short_t,
                flat_traders: flat_t,
                net_contracts: net,
                long_ratio,
                bias,
            }
        })
        .collect();
    // Sort symbols by lopsidedness so the UI surfaces the most-skewed names first.
    by_symbol.sort_by(|a, b| {
        let aw = a.long_ratio.map(|r| (r - 0.5).abs()).unwrap_or(0.0);
        let bw = b.long_ratio.map(|r| (r - 0.5).abs()).unwrap_or(0.0);
        bw.partial_cmp(&aw).unwrap_or(std::cmp::Ordering::Equal)
    });
    let most_lopsided = by_symbol.first().map(|s| s.symbol.clone());
    TiltReport {
        by_symbol,
        active_traders: active_set.len() as u32,
        most_lopsided_symbol: most_lopsided,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(id: &str, sym: &str, n: i64) -> TraderPosition {
        TraderPosition {
            trader_id: id.into(),
            symbol: sym.into(),
            net_contracts: n,
        }
    }

    #[test]
    fn empty_returns_empty_report() {
        let r = aggregate(&[]);
        assert!(r.by_symbol.is_empty());
        assert_eq!(r.active_traders, 0);
        assert!(r.most_lopsided_symbol.is_none());
    }

    #[test]
    fn flat_positions_dont_count_as_active_traders() {
        // 3 traders, all flat. None of them are "positioned".
        let r = aggregate(&[pos("a", "ES", 0), pos("b", "ES", 0), pos("c", "ES", 0)]);
        assert_eq!(r.active_traders, 0);
        assert_eq!(r.by_symbol[0].flat_traders, 3);
        assert!(
            r.by_symbol[0].long_ratio.is_none(),
            "long_ratio undefined when nobody's positioned"
        );
    }

    #[test]
    fn balanced_room_is_classified_balanced() {
        // 5 long + 5 short = 0.50 long_ratio → Balanced.
        let mut ps = Vec::new();
        for i in 0..5 {
            ps.push(pos(&format!("L{i}"), "ES", 1));
        }
        for i in 0..5 {
            ps.push(pos(&format!("S{i}"), "ES", -1));
        }
        let r = aggregate(&ps);
        let es = &r.by_symbol[0];
        assert_eq!(es.long_ratio, Some(0.5));
        assert!(matches!(es.bias, TiltBias::Balanced));
    }

    #[test]
    fn heavy_long_room_is_strongly_long() {
        // 8 long + 2 short = 0.80 → StronglyLong.
        let mut ps = Vec::new();
        for i in 0..8 {
            ps.push(pos(&format!("L{i}"), "ES", 3));
        }
        for i in 0..2 {
            ps.push(pos(&format!("S{i}"), "ES", -3));
        }
        let r = aggregate(&ps);
        let es = &r.by_symbol[0];
        assert_eq!(es.long_ratio, Some(0.8));
        assert!(matches!(es.bias, TiltBias::StronglyLong));
        // Net contracts: 8×3 - 2×3 = 24 - 6 = 18 long.
        assert_eq!(es.net_contracts, 18);
    }

    #[test]
    fn heavy_short_room_is_strongly_short() {
        // 1 long + 4 short = 0.20 → StronglyShort.
        let r = aggregate(&[
            pos("a", "NQ", 1),
            pos("b", "NQ", -2),
            pos("c", "NQ", -2),
            pos("d", "NQ", -2),
            pos("e", "NQ", -2),
        ]);
        let nq = &r.by_symbol[0];
        assert_eq!(nq.long_ratio, Some(0.2));
        assert!(matches!(nq.bias, TiltBias::StronglyShort));
    }

    #[test]
    fn lopsided_symbol_surfaces_first() {
        // ES is 0.50 (Balanced), NQ is 0.80 (StronglyLong).
        // NQ should bubble to the top of the report.
        let mut ps = Vec::new();
        ps.push(pos("a", "ES", 1));
        ps.push(pos("b", "ES", -1));
        for i in 0..4 {
            ps.push(pos(&format!("L{i}"), "NQ", 1));
        }
        ps.push(pos("S0", "NQ", -1));
        let r = aggregate(&ps);
        assert_eq!(r.by_symbol[0].symbol, "NQ");
        assert_eq!(r.most_lopsided_symbol, Some("NQ".into()));
    }

    #[test]
    fn same_trader_in_two_symbols_counted_once_in_active() {
        // Trader "a" is long ES AND short NQ → counts once as active.
        let r = aggregate(&[pos("a", "ES", 1), pos("a", "NQ", -1)]);
        assert_eq!(r.active_traders, 1);
    }
}
