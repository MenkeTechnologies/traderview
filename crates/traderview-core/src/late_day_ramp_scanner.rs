//! Late-Day Ramp Scanner — final-hour breakout detector. Filters
//! symbols whose closing range pushed through a daily high / low with
//! confirming volume. Catches end-of-day momentum that often
//! foreshadows next-day continuation.
//!
//! Per-symbol inputs: full-day high/low, the closing-hour
//! (3:00pm-4:00pm ET) OHLCV bar, and the 20-day average closing-hour
//! volume.
//!
//! Match criteria (long side):
//!   1. close ≥ closing_hour_open  (positive close)
//!   2. close ≥ day_high - tolerance · day_range
//!      (within tolerance of session high)
//!   3. closing_hour_high ≥ day_high      (broke or matched session high)
//!   4. closing_hour_volume ≥ volume_ratio_min · avg_close_hour_volume
//!   5. (close - closing_hour_open) / closing_hour_open · 100 ≥ min_move_pct
//!
//! Short side: symmetric with low / close ≤ closing_hour_open. Returns
//! both sides tagged in `side`.
//!
//! Score = move_pct · sqrt(volume_ratio) — favors big moves on big
//! volume, but volume effect is sub-linear (one fat tape print
//! shouldn't dominate).
//!
//! Pure compute. Companion to `gap_and_go_scanner`, `volume_burst`,
//! `power_bar`.

#[derive(Clone, Debug)]
pub struct Symbol {
    pub symbol: String,
    pub day_high: f64,
    pub day_low: f64,
    pub closing_hour_open: f64,
    pub closing_hour_high: f64,
    pub closing_hour_low: f64,
    pub close: f64,
    pub closing_hour_volume: f64,
    pub avg_close_hour_volume: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Long,
    Short,
}

#[derive(Clone, Copy, Debug)]
pub struct Config {
    pub tolerance_of_extreme: f64, // fraction of day range
    pub volume_ratio_min: f64,
    pub min_move_pct: f64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            tolerance_of_extreme: 0.05,
            volume_ratio_min: 1.5,
            min_move_pct: 0.5,
        }
    }
}

#[derive(Debug)]
pub struct Match {
    pub symbol: String,
    pub side: Side,
    pub move_pct: f64,
    pub volume_ratio: f64,
    pub score: f64,
}

pub fn scan(symbols: &[Symbol], config: Config) -> Vec<Match> {
    let mut matches = Vec::new();
    if !config.tolerance_of_extreme.is_finite() || config.tolerance_of_extreme < 0.0 {
        return matches;
    }
    if !config.volume_ratio_min.is_finite() || config.volume_ratio_min <= 0.0 {
        return matches;
    }
    if !config.min_move_pct.is_finite() || config.min_move_pct < 0.0 {
        return matches;
    }
    for sym in symbols {
        let fields = [
            sym.day_high,
            sym.day_low,
            sym.closing_hour_open,
            sym.closing_hour_high,
            sym.closing_hour_low,
            sym.close,
            sym.closing_hour_volume,
            sym.avg_close_hour_volume,
        ];
        if fields.iter().any(|x| !x.is_finite()) {
            continue;
        }
        if sym.closing_hour_open <= 0.0 || sym.avg_close_hour_volume <= 0.0 {
            continue;
        }
        if sym.day_high <= sym.day_low {
            continue;
        }
        let day_range = sym.day_high - sym.day_low;
        let tolerance = config.tolerance_of_extreme * day_range;
        let volume_ratio = sym.closing_hour_volume / sym.avg_close_hour_volume;
        if volume_ratio < config.volume_ratio_min {
            continue;
        }
        // Long-side test.
        let long_match = sym.close >= sym.closing_hour_open
            && sym.close >= sym.day_high - tolerance
            && sym.closing_hour_high >= sym.day_high;
        let short_match = sym.close <= sym.closing_hour_open
            && sym.close <= sym.day_low + tolerance
            && sym.closing_hour_low <= sym.day_low;
        let (side, move_pct) = if long_match {
            (
                Some(Side::Long),
                (sym.close - sym.closing_hour_open) / sym.closing_hour_open * 100.0,
            )
        } else if short_match {
            (
                Some(Side::Short),
                (sym.closing_hour_open - sym.close) / sym.closing_hour_open * 100.0,
            )
        } else {
            (None, 0.0)
        };
        let Some(side) = side else { continue };
        if move_pct < config.min_move_pct {
            continue;
        }
        let score = move_pct * volume_ratio.sqrt();
        matches.push(Match {
            symbol: sym.symbol.clone(),
            side,
            move_pct,
            volume_ratio,
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
    fn sym(
        name: &str,
        dh: f64,
        dl: f64,
        cho: f64,
        chh: f64,
        chl: f64,
        c: f64,
        chv: f64,
        avg_chv: f64,
    ) -> Symbol {
        Symbol {
            symbol: name.into(),
            day_high: dh,
            day_low: dl,
            closing_hour_open: cho,
            closing_hour_high: chh,
            closing_hour_low: chl,
            close: c,
            closing_hour_volume: chv,
            avg_close_hour_volume: avg_chv,
        }
    }

    #[test]
    fn empty_input_yields_empty() {
        let r = scan(&[], Config::default());
        assert!(r.is_empty());
    }

    #[test]
    fn closing_breakout_to_session_high_matches_long() {
        // Day range 100-110, closing hour opens 105 and ramps to 110.
        // Closing-hour high matches day high, close near day high.
        let s = sym(
            "ABCD",
            110.0,
            100.0,
            105.0,
            110.5,
            104.0,
            110.0,
            1_500_000.0,
            1_000_000.0,
        );
        let r = scan(&[s], Config::default());
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].side, Side::Long);
        assert!(r[0].move_pct > 0.0);
    }

    #[test]
    fn closing_breakdown_to_session_low_matches_short() {
        let s = sym(
            "ABCD",
            110.0,
            100.0,
            105.0,
            105.5,
            99.5,
            100.0,
            1_500_000.0,
            1_000_000.0,
        );
        let r = scan(&[s], Config::default());
        assert_eq!(r.len(), 1);
        assert_eq!(r[0].side, Side::Short);
        assert!(r[0].move_pct > 0.0);
    }

    #[test]
    fn middle_of_range_close_rejected() {
        // Close at midpoint with no breakout — should not match.
        let s = sym(
            "ABCD",
            110.0,
            100.0,
            105.0,
            106.0,
            104.0,
            105.0,
            1_500_000.0,
            1_000_000.0,
        );
        let r = scan(&[s], Config::default());
        assert!(r.is_empty());
    }

    #[test]
    fn thin_volume_rejected() {
        let s = sym(
            "ABCD",
            110.0,
            100.0,
            105.0,
            110.5,
            104.0,
            110.0,
            1_000_000.0,
            1_000_000.0,
        );
        let r = scan(&[s], Config::default());
        assert!(r.is_empty());
    }

    #[test]
    fn small_move_rejected_by_min_move() {
        let cfg = Config {
            min_move_pct: 5.0,
            ..Config::default()
        };
        let s = sym(
            "ABCD",
            110.0,
            100.0,
            109.5,
            110.5,
            109.0,
            110.0,
            1_500_000.0,
            1_000_000.0,
        );
        let r = scan(&[s], cfg);
        assert!(r.is_empty());
    }

    #[test]
    fn invalid_fields_skip_symbol() {
        let mut s = sym(
            "ABCD",
            110.0,
            100.0,
            105.0,
            110.5,
            104.0,
            110.0,
            1_500_000.0,
            1_000_000.0,
        );
        s.closing_hour_open = 0.0;
        assert!(scan(&[s.clone()], Config::default()).is_empty());
        s.closing_hour_open = 105.0;
        s.day_high = 100.0; // day_high <= day_low
        assert!(scan(&[s.clone()], Config::default()).is_empty());
        s.day_high = 110.0;
        s.close = f64::NAN;
        assert!(scan(&[s], Config::default()).is_empty());
    }

    #[test]
    fn matches_sorted_by_score_descending() {
        let strong = sym(
            "STRONG",
            120.0,
            100.0,
            110.0,
            121.0,
            109.0,
            120.0,
            5_000_000.0,
            1_000_000.0,
        );
        let weak = sym(
            "WEAK",
            110.0,
            100.0,
            108.0,
            110.5,
            107.0,
            110.0,
            2_000_000.0,
            1_000_000.0,
        );
        let r = scan(&[weak, strong], Config::default());
        assert_eq!(r.len(), 2);
        assert!(r[0].score >= r[1].score);
        assert_eq!(r[0].symbol, "STRONG");
    }
}
