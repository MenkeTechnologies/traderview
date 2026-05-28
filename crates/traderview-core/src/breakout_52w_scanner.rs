//! 52-week High / Low Breakout Scanner.
//!
//! Donchian-style absolute-extremity screener that flags symbols whose
//! latest close penetrates the prior `period` bars' high or low. Adds
//! volume confirmation: breakout volume must be ≥ `min_volume_ratio`
//! times the trailing average volume to count.
//!
//! Classifications:
//!   - **NewHigh**: close > max(prior high) AND vol confirms
//!   - **NewLow**:  close < min(prior low) AND vol confirms
//!   - **Stalking**: close within `stalk_pct` of the extreme (e.g. < 2%)
//!     but hasn't crossed — pre-breakout watchlist
//!   - **None**:    no signal
//!
//! Pure compute. Default `period = 252` ≈ 1 year of trading days.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolSeries {
    pub symbol: String,
    /// Most recent bar last.
    pub bars: Vec<Bar>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar {
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub period: usize,
    pub min_volume_ratio: f64,
    pub stalk_pct: f64,
}

impl Default for Config {
    fn default() -> Self { Self { period: 252, min_volume_ratio: 1.5, stalk_pct: 0.02 } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BreakoutKind { NewHigh, NewLow, StalkingHigh, StalkingLow, None }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakoutHit {
    pub symbol: String,
    pub kind: BreakoutKind,
    pub last_close: f64,
    pub extreme_price: f64,
    pub volume_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ScannerReport {
    pub new_highs: Vec<BreakoutHit>,
    pub new_lows: Vec<BreakoutHit>,
    pub stalking: Vec<BreakoutHit>,
}

pub fn scan(symbols: &[SymbolSeries], cfg: &Config) -> ScannerReport {
    let mut report = ScannerReport::default();
    if cfg.period == 0
        || !cfg.min_volume_ratio.is_finite() || cfg.min_volume_ratio <= 0.0
        || !cfg.stalk_pct.is_finite() || cfg.stalk_pct < 0.0
    {
        return report;
    }
    for sym in symbols {
        let n = sym.bars.len();
        if n < cfg.period + 1 { continue; }
        let cur = sym.bars[n - 1];
        if !cur.close.is_finite() || !cur.volume.is_finite()
            || cur.close <= 0.0 || cur.volume < 0.0
        {
            continue;
        }
        let prior = &sym.bars[n - 1 - cfg.period..n - 1];
        // Validate prior window.
        if prior.iter().any(|b| !b.high.is_finite() || !b.low.is_finite() || !b.volume.is_finite()) {
            continue;
        }
        let prior_high = prior.iter().map(|b| b.high).fold(f64::NEG_INFINITY, f64::max);
        let prior_low  = prior.iter().map(|b| b.low).fold(f64::INFINITY, f64::min);
        let avg_volume = prior.iter().map(|b| b.volume).sum::<f64>() / prior.len() as f64;
        let vol_ratio = if avg_volume > 0.0 { cur.volume / avg_volume } else { f64::INFINITY };
        let mut emitted = false;
        if cur.close > prior_high && vol_ratio >= cfg.min_volume_ratio {
            report.new_highs.push(BreakoutHit {
                symbol: sym.symbol.clone(),
                kind: BreakoutKind::NewHigh,
                last_close: cur.close,
                extreme_price: prior_high,
                volume_ratio: vol_ratio,
            });
            emitted = true;
        } else if cur.close < prior_low && vol_ratio >= cfg.min_volume_ratio {
            report.new_lows.push(BreakoutHit {
                symbol: sym.symbol.clone(),
                kind: BreakoutKind::NewLow,
                last_close: cur.close,
                extreme_price: prior_low,
                volume_ratio: vol_ratio,
            });
            emitted = true;
        }
        if !emitted {
            // Stalking: within stalk_pct of extreme.
            let near_high = (prior_high - cur.close) / prior_high <= cfg.stalk_pct
                && cur.close <= prior_high;
            let near_low = (cur.close - prior_low) / prior_low <= cfg.stalk_pct
                && cur.close >= prior_low;
            if near_high {
                report.stalking.push(BreakoutHit {
                    symbol: sym.symbol.clone(),
                    kind: BreakoutKind::StalkingHigh,
                    last_close: cur.close,
                    extreme_price: prior_high,
                    volume_ratio: vol_ratio,
                });
            } else if near_low {
                report.stalking.push(BreakoutHit {
                    symbol: sym.symbol.clone(),
                    kind: BreakoutKind::StalkingLow,
                    last_close: cur.close,
                    extreme_price: prior_low,
                    volume_ratio: vol_ratio,
                });
            }
        }
    }
    // Sort each bucket: new_highs/lows by vol_ratio desc, stalking by proximity.
    report.new_highs.sort_by(|a, b| b.volume_ratio.partial_cmp(&a.volume_ratio)
        .unwrap_or(std::cmp::Ordering::Equal));
    report.new_lows.sort_by(|a, b| b.volume_ratio.partial_cmp(&a.volume_ratio)
        .unwrap_or(std::cmp::Ordering::Equal));
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(c: f64, v: f64) -> Bar {
        Bar { high: c + 0.5, low: c - 0.5, close: c, volume: v }
    }

    fn ser(sym: &str, bars: Vec<Bar>) -> SymbolSeries {
        SymbolSeries { symbol: sym.into(), bars }
    }

    #[test]
    fn empty_returns_default() {
        let r = scan(&[], &Config::default());
        assert!(r.new_highs.is_empty());
    }

    #[test]
    fn invalid_config_returns_default() {
        let bars: Vec<Bar> = (0..260).map(|i| b(100.0 + i as f64 * 0.1, 1_000.0)).collect();
        let series = vec![ser("X", bars)];
        for cfg in [
            Config { period: 0, ..Default::default() },
            Config { min_volume_ratio: 0.0, ..Default::default() },
            Config { stalk_pct: -1.0, ..Default::default() },
        ] {
            let r = scan(&series, &cfg);
            assert!(r.new_highs.is_empty() && r.new_lows.is_empty());
        }
    }

    #[test]
    fn too_few_bars_skipped() {
        let series = vec![ser("SHORT", vec![b(100.0, 1_000.0); 10])];
        let r = scan(&series, &Config::default());
        assert!(r.new_highs.is_empty());
    }

    #[test]
    fn new_high_detected_when_breaking_above_with_volume() {
        // 252 bars at ~100, then new bar at 110 with 2x volume.
        let mut bars = vec![b(100.0, 1_000.0); 252];
        bars.push(b(110.0, 2_500.0));
        let r = scan(&[ser("BREAKER", bars)], &Config::default());
        assert_eq!(r.new_highs.len(), 1);
        assert_eq!(r.new_highs[0].kind, BreakoutKind::NewHigh);
    }

    #[test]
    fn new_high_rejected_without_volume_confirmation() {
        let mut bars = vec![b(100.0, 1_000.0); 252];
        bars.push(b(110.0, 1_000.0));    // only 1x vol
        let r = scan(&[ser("WEAK", bars)], &Config::default());
        assert!(r.new_highs.is_empty());
    }

    #[test]
    fn new_low_detected() {
        let mut bars = vec![b(100.0, 1_000.0); 252];
        bars.push(b(85.0, 3_000.0));
        let r = scan(&[ser("BREAKDOWN", bars)], &Config::default());
        assert_eq!(r.new_lows.len(), 1);
        assert_eq!(r.new_lows[0].kind, BreakoutKind::NewLow);
    }

    #[test]
    fn stalking_high_detected_when_within_threshold() {
        // 252 bars with max high 100.5 → stalking @ 99.0 (1.5% below).
        let mut bars = vec![b(100.0, 1_000.0); 252];
        bars.push(b(99.0, 800.0));
        let r = scan(&[ser("STALK", bars)], &Config::default());
        assert!(r.stalking.iter().any(|h| h.kind == BreakoutKind::StalkingHigh));
    }

    #[test]
    fn nan_bars_skipped_safely() {
        let mut bars = vec![b(100.0, 1_000.0); 252];
        bars[100].high = f64::NAN;
        bars.push(b(110.0, 2_500.0));
        let r = scan(&[ser("NANSAFE", bars)], &Config::default());
        // Skipped without panic.
        assert!(r.new_highs.is_empty());
    }
}
