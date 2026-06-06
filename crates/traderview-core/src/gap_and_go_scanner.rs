//! Gap-and-Go Scanner — momentum-day-trader setup: prior-close → open
//! gap above threshold + first-N-minute volume / range confirmation +
//! held the gap (didn't fill). Filters out gaps that immediately reverse
//! or trade thin.
//!
//! Per-symbol inputs: prior close, opening N-minute OHLCV bar (the
//! "first bar"), and the 20-day average volume of that first-N-minute
//! window. Configurable thresholds in `Config`.
//!
//! Match criteria (all must hold):
//!   1. gap_pct = (open - prior_close) / prior_close · 100 ≥ min_gap_pct
//!   2. first_bar_volume ≥ volume_ratio_min · avg_first_bar_volume
//!   3. first_bar_close ≥ first_bar_open                    (held the gap)
//!   4. (first_bar_high - first_bar_low) / open · 100 ≥ min_range_pct
//!   5. first_bar_low ≥ prior_close                         (no gap fill)
//!
//! Returns a score = clamp((gap_pct - min_gap_pct) + (vol_ratio - 1.0)
//! · 5.0, 0.0, 100.0) for ranking match strength.
//!
//! Pure compute. Companion to `gap_classifier`, `gap_fill_stats`,
//! `late_day_ramp_scanner`, `vcp_pattern`, `volume_burst`.

#[derive(Clone, Debug)]
pub struct Symbol {
    pub symbol: String,
    pub prior_close: f64,
    pub first_bar_open: f64,
    pub first_bar_high: f64,
    pub first_bar_low: f64,
    pub first_bar_close: f64,
    pub first_bar_volume: f64,
    pub avg_first_bar_volume: f64,
}

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub min_gap_pct: f64,
    pub volume_ratio_min: f64,
    pub min_range_pct: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            min_gap_pct: 4.0,
            volume_ratio_min: 3.0,
            min_range_pct: 2.0,
        }
    }
}

#[derive(Debug)]
pub struct Match {
    pub symbol: String,
    pub gap_pct: f64,
    pub volume_ratio: f64,
    pub range_pct: f64,
    pub score: f64,
}

pub fn scan(symbols: &[Symbol], config: Config) -> Vec<Match> {
    let mut matches = Vec::new();
    if !config.min_gap_pct.is_finite() || config.min_gap_pct <= 0.0 {
        return matches;
    }
    if !config.volume_ratio_min.is_finite() || config.volume_ratio_min <= 0.0 {
        return matches;
    }
    if !config.min_range_pct.is_finite() || config.min_range_pct < 0.0 {
        return matches;
    }
    for sym in symbols {
        let fields = [
            sym.prior_close,
            sym.first_bar_open,
            sym.first_bar_high,
            sym.first_bar_low,
            sym.first_bar_close,
            sym.first_bar_volume,
            sym.avg_first_bar_volume,
        ];
        if fields.iter().any(|x| !x.is_finite()) {
            continue;
        }
        if sym.prior_close <= 0.0 || sym.first_bar_open <= 0.0 {
            continue;
        }
        if sym.avg_first_bar_volume <= 0.0 {
            continue;
        }
        let gap_pct = (sym.first_bar_open - sym.prior_close) / sym.prior_close * 100.0;
        if gap_pct < config.min_gap_pct {
            continue;
        }
        let volume_ratio = sym.first_bar_volume / sym.avg_first_bar_volume;
        if volume_ratio < config.volume_ratio_min {
            continue;
        }
        if sym.first_bar_close < sym.first_bar_open {
            continue;
        }
        let range_pct = (sym.first_bar_high - sym.first_bar_low) / sym.first_bar_open * 100.0;
        if range_pct < config.min_range_pct {
            continue;
        }
        if sym.first_bar_low < sym.prior_close {
            continue;
        }
        let score = ((gap_pct - config.min_gap_pct) + (volume_ratio - 1.0) * 5.0).clamp(0.0, 100.0);
        matches.push(Match {
            symbol: sym.symbol.clone(),
            gap_pct,
            volume_ratio,
            range_pct,
            score,
        });
    }
    matches.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    matches
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::too_many_arguments)]
    fn sym(name: &str, prior: f64, o: f64, h: f64, l: f64, c: f64, v: f64, avg_v: f64) -> Symbol {
        Symbol {
            symbol: name.into(),
            prior_close: prior,
            first_bar_open: o,
            first_bar_high: h,
            first_bar_low: l,
            first_bar_close: c,
            first_bar_volume: v,
            avg_first_bar_volume: avg_v,
        }
    }

    #[test]
    fn empty_input_yields_empty_matches() {
        let r = scan(&[], Config::default());
        assert!(r.is_empty());
    }

    #[test]
    fn classic_setup_matches() {
        // Prior 100, opens 110 (+10% gap), strong volume, holds gap.
        let s = sym(
            "ABCD",
            100.0,
            110.0,
            115.0,
            110.5,
            114.0,
            5_000_000.0,
            1_000_000.0,
        );
        let r = scan(&[s], Config::default());
        assert_eq!(r.len(), 1);
        assert!((r[0].gap_pct - 10.0).abs() < 1e-9);
        assert!((r[0].volume_ratio - 5.0).abs() < 1e-9);
        assert!(r[0].score > 0.0);
    }

    #[test]
    fn small_gap_rejected() {
        let s = sym(
            "ABCD",
            100.0,
            101.0,
            102.0,
            101.0,
            101.5,
            5_000_000.0,
            1_000_000.0,
        );
        let r = scan(&[s], Config::default());
        assert!(r.is_empty());
    }

    #[test]
    fn thin_volume_rejected() {
        let s = sym(
            "ABCD",
            100.0,
            110.0,
            115.0,
            110.5,
            114.0,
            1_500_000.0,
            1_000_000.0,
        );
        let r = scan(&[s], Config::default());
        assert!(r.is_empty());
    }

    #[test]
    fn reversal_first_bar_rejected() {
        // Close < open → didn't hold the gap.
        let s = sym(
            "ABCD",
            100.0,
            110.0,
            115.0,
            105.0,
            106.0,
            5_000_000.0,
            1_000_000.0,
        );
        let r = scan(&[s], Config::default());
        assert!(r.is_empty());
    }

    #[test]
    fn gap_fill_rejected() {
        // First-bar low < prior close → gap got filled.
        let s = sym(
            "ABCD",
            100.0,
            110.0,
            115.0,
            99.0,
            112.0,
            5_000_000.0,
            1_000_000.0,
        );
        let r = scan(&[s], Config::default());
        assert!(r.is_empty());
    }

    #[test]
    fn invalid_fields_skip_symbol() {
        let mut s = sym(
            "ABCD",
            100.0,
            110.0,
            115.0,
            110.5,
            114.0,
            5_000_000.0,
            1_000_000.0,
        );
        s.prior_close = f64::NAN;
        assert!(scan(&[s.clone()], Config::default()).is_empty());
        s.prior_close = 0.0;
        assert!(scan(&[s.clone()], Config::default()).is_empty());
        s.prior_close = 100.0;
        s.avg_first_bar_volume = 0.0;
        assert!(scan(&[s], Config::default()).is_empty());
    }

    #[test]
    fn matches_sorted_by_score_descending() {
        let strong = sym(
            "STRONG",
            100.0,
            115.0,
            120.0,
            115.5,
            119.0,
            10_000_000.0,
            1_000_000.0,
        );
        let weak = sym(
            "WEAK",
            100.0,
            105.0,
            108.0,
            105.0,
            107.0,
            4_000_000.0,
            1_000_000.0,
        );
        let r = scan(&[weak, strong], Config::default());
        assert_eq!(r.len(), 2);
        assert!(r[0].score >= r[1].score);
        assert_eq!(r[0].symbol, "STRONG");
    }
}
