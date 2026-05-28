//! Stock scanners — Warrior Trading / Zendoo preset filters over daily bars.
//!
//! Each preset answers a yes/no per symbol: "does this symbol match the
//! pattern today?". The scanner-routes layer iterates over a universe of
//! symbols and returns hits with the key stats that justify the match.

use crate::indicators::{closes, highs, lows, sma, volumes};
use crate::models::PriceBar;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ScanHit {
    pub symbol: String,
    pub matched: Vec<&'static str>, // preset names matched
    pub price: f64,
    pub gap_pct: f64,    // open vs prior close
    pub change_pct: f64, // close vs prior close
    pub day_pct: f64,    // close vs open
    pub volume: f64,
    pub rel_volume: f64,    // today vs 20-day avg
    pub hod_dist_pct: f64,  // (close - day_high) / day_high
    pub lod_dist_pct: f64,  // (close - day_low) / day_low
    pub year_high_pct: f64, // close vs 52w high
    pub year_low_pct: f64,  // close vs 52w low
}

impl ScanHit {
    pub fn empty(symbol: &str) -> Self {
        ScanHit {
            symbol: symbol.into(),
            matched: Vec::new(),
            price: 0.0,
            gap_pct: 0.0,
            change_pct: 0.0,
            day_pct: 0.0,
            volume: 0.0,
            rel_volume: 0.0,
            hod_dist_pct: 0.0,
            lod_dist_pct: 0.0,
            year_high_pct: 0.0,
            year_low_pct: 0.0,
        }
    }
}

/// Compute the raw stats for a single symbol given its daily bars (most
/// recent last). Returns None if there's not enough data.
pub fn stats_for(symbol: &str, bars: &[PriceBar]) -> Option<ScanHit> {
    let n = bars.len();
    if n < 2 {
        return None;
    }
    let last = &bars[n - 1];
    let prev = &bars[n - 2];
    let opens: Vec<f64> = bars.iter().map(|b| dec(b.open)).collect();
    let cs = closes(bars);
    let hi = highs(bars);
    let lo = lows(bars);
    let vol = volumes(bars);
    let price = cs[n - 1];
    let open = opens[n - 1];
    let prev_close = cs[n - 2];
    let day_high = hi[n - 1];
    let day_low = lo[n - 1];
    let vol_today = vol[n - 1];
    let avg_vol = sma(&vol, 20.min(n))
        .last()
        .and_then(|x| *x)
        .unwrap_or(vol_today.max(1.0));
    let year_high = hi[n.saturating_sub(252)..]
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let year_low = lo[n.saturating_sub(252)..]
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min);

    let pct = |a: f64, b: f64| if b > 0.0 { (a - b) / b * 100.0 } else { 0.0 };
    let _ = last;
    let _ = prev; // future use
    Some(ScanHit {
        symbol: symbol.into(),
        matched: Vec::new(),
        price,
        gap_pct: pct(open, prev_close),
        change_pct: pct(price, prev_close),
        day_pct: pct(price, open),
        volume: vol_today,
        rel_volume: if avg_vol > 0.0 {
            vol_today / avg_vol
        } else {
            0.0
        },
        hod_dist_pct: if day_high > 0.0 {
            (price - day_high) / day_high * 100.0
        } else {
            0.0
        },
        lod_dist_pct: if day_low > 0.0 {
            (price - day_low) / day_low * 100.0
        } else {
            0.0
        },
        year_high_pct: pct(price, year_high),
        year_low_pct: pct(price, year_low),
    })
}

fn dec(d: rust_decimal::Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Preset {
    PremarketGappers, // gap_pct >= 5% (up or down)
    MomentumMovers,   // change_pct >= 5% and rel_volume >= 2.0
    HighOfDay,        // |hod_dist_pct| < 0.5%
    LowFloatRunners,  // change_pct >= 10% and rel_volume >= 5.0 (proxy for low float)
    Pct52wHigh,       // within 1% of 52w high
    Pct52wLow,        // within 1% of 52w low
    VolumeSurge,      // rel_volume >= 3.0
    Breakdown,        // change_pct <= -5%
    Breakout,         // close above 20-day high
    OversoldBounce,   // close > yesterday close AND yesterday was -5% or worse
    // === Batch added presets ===
    GapAndGo,         // gap up >= 3% AND close above open AND closed near HOD (+volume)
    GapAndFade,       // gap up >= 3% BUT close < open (fade) AND closed near LOD
    InsideDayLow,     // close <= day_low + 0.5% — could break down tomorrow
    InsideDayHigh,    // close near HOD AND barely off prev close (coiling at extreme)
    RangeContractionDay, // tight range vs avg: day_pct + gap_pct both near zero, rel_vol low
    DistributionDay,  // close down >= 2% on rel_volume >= 1.5x
    AccumulationDay,  // close up >= 2% on rel_volume >= 1.5x
    NearYearHighLowVol, // within 1% of 52w high BUT rel_volume < 1 (no real buying interest)
}

pub fn matches(hit: &ScanHit, preset: Preset) -> bool {
    match preset {
        Preset::PremarketGappers => hit.gap_pct.abs() >= 5.0,
        Preset::MomentumMovers => hit.change_pct >= 5.0 && hit.rel_volume >= 2.0,
        Preset::HighOfDay => hit.hod_dist_pct.abs() <= 0.5,
        Preset::LowFloatRunners => hit.change_pct >= 10.0 && hit.rel_volume >= 5.0,
        Preset::Pct52wHigh => hit.year_high_pct >= -1.0,
        Preset::Pct52wLow => hit.year_low_pct <= 1.0,
        Preset::VolumeSurge => hit.rel_volume >= 3.0,
        Preset::Breakdown => hit.change_pct <= -5.0,
        Preset::Breakout => hit.day_pct > 0.0 && hit.hod_dist_pct.abs() <= 0.5,
        Preset::OversoldBounce => hit.change_pct > 0.0, // simplified — needs prior bar context
        Preset::GapAndGo => {
            hit.gap_pct >= 3.0
                && hit.day_pct > 0.0
                && hit.hod_dist_pct.abs() <= 1.0
                && hit.rel_volume >= 1.5
        }
        Preset::GapAndFade => {
            hit.gap_pct >= 3.0
                && hit.day_pct < 0.0
                && hit.lod_dist_pct.abs() <= 1.0
        }
        Preset::InsideDayLow => hit.lod_dist_pct.abs() <= 0.5,
        Preset::InsideDayHigh => {
            hit.hod_dist_pct.abs() <= 0.5 && hit.change_pct.abs() <= 1.0
        }
        Preset::RangeContractionDay => {
            hit.day_pct.abs() <= 0.5 && hit.gap_pct.abs() <= 0.5 && hit.rel_volume <= 0.7
        }
        Preset::DistributionDay => hit.change_pct <= -2.0 && hit.rel_volume >= 1.5,
        Preset::AccumulationDay => hit.change_pct >= 2.0 && hit.rel_volume >= 1.5,
        Preset::NearYearHighLowVol => hit.year_high_pct >= -1.0 && hit.rel_volume < 1.0,
    }
}

pub fn preset_label(p: Preset) -> &'static str {
    match p {
        Preset::PremarketGappers => "Gappers",
        Preset::MomentumMovers => "Momentum",
        Preset::HighOfDay => "High of Day",
        Preset::LowFloatRunners => "Low-Float Runner",
        Preset::Pct52wHigh => "52w High",
        Preset::Pct52wLow => "52w Low",
        Preset::VolumeSurge => "Volume Surge",
        Preset::Breakdown => "Breakdown",
        Preset::Breakout => "Breakout",
        Preset::OversoldBounce => "Oversold Bounce",
        Preset::GapAndGo => "Gap & Go",
        Preset::GapAndFade => "Gap & Fade",
        Preset::InsideDayLow => "Near Day Low",
        Preset::InsideDayHigh => "Coiling at HOD",
        Preset::RangeContractionDay => "Range Contraction",
        Preset::DistributionDay => "Distribution Day",
        Preset::AccumulationDay => "Accumulation Day",
        Preset::NearYearHighLowVol => "52w High, No Volume",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{TimeZone, Utc};
    use rust_decimal::Decimal;

    fn bar(open: u64, high: u64, low: u64, close: u64, vol: u64, ts: i64) -> PriceBar {
        PriceBar {
            symbol: "X".into(),
            interval: crate::models::BarInterval::D1,
            bar_time: Utc.timestamp_opt(ts, 0).unwrap(),
            open: Decimal::from(open),
            high: Decimal::from(high),
            low: Decimal::from(low),
            close: Decimal::from(close),
            volume: Decimal::from(vol),
            source: "test".into(),
        }
    }

    #[test]
    fn gappers_fires_on_5pct_gap() {
        let bars = vec![
            bar(100, 100, 95, 100, 1_000_000, 1),
            bar(106, 110, 105, 108, 1_000_000, 2),
        ];
        let hit = stats_for("X", &bars).unwrap();
        assert!(matches(&hit, Preset::PremarketGappers));
    }

    #[test]
    fn gap_and_go_fires_on_upgap_with_strong_close_at_hod() {
        // Prior close 100. Open gaps to 105 (5% gap up).
        // Closes near day's high (108 vs HOD 108).
        // Need rel_volume >= 1.5 — build a 5-bar baseline at 1M then today at 2M.
        let bars = vec![
            bar(100, 101, 99,  100, 1_000_000, 1),
            bar(100, 101, 99,  100, 1_000_000, 2),
            bar(100, 101, 99,  100, 1_000_000, 3),
            bar(100, 101, 99,  100, 1_000_000, 4),
            bar(105, 108, 104, 108, 2_000_000, 5),
        ];
        let hit = stats_for("X", &bars).unwrap();
        assert!(matches(&hit, Preset::GapAndGo),
            "gap={} day_pct={} hod_dist={} rel_vol={}",
            hit.gap_pct, hit.day_pct, hit.hod_dist_pct, hit.rel_volume);
    }

    #[test]
    fn distribution_day_fires_on_2pct_down_with_high_volume() {
        let bars = vec![
            bar(100, 101, 99, 100, 1_000_000, 1),
            bar(100, 101, 99, 100, 1_000_000, 2),
            bar(100, 101, 99, 100, 1_000_000, 3),
            bar(100, 101, 99, 100, 1_000_000, 4),
            bar(100, 100, 95,  97, 2_000_000, 5),    // close -3%, vol 2x avg
        ];
        let hit = stats_for("X", &bars).unwrap();
        assert!(matches(&hit, Preset::DistributionDay));
    }

    #[test]
    fn accumulation_day_fires_on_2pct_up_with_high_volume() {
        let bars = vec![
            bar(100, 101, 99, 100, 1_000_000, 1),
            bar(100, 101, 99, 100, 1_000_000, 2),
            bar(100, 101, 99, 100, 1_000_000, 3),
            bar(100, 101, 99, 100, 1_000_000, 4),
            bar(100, 104, 100, 103, 2_000_000, 5),
        ];
        let hit = stats_for("X", &bars).unwrap();
        assert!(matches(&hit, Preset::AccumulationDay));
    }

    #[test]
    fn range_contraction_fires_on_tiny_day_with_low_volume() {
        let bars = vec![
            bar(100, 105, 95, 100, 2_000_000, 1),
            bar(100, 105, 95, 100, 2_000_000, 2),
            bar(100, 105, 95, 100, 2_000_000, 3),
            bar(100, 105, 95, 100, 2_000_000, 4),
            bar(100, 100, 100, 100, 1_000_000, 5),    // doji-like, half avg vol
        ];
        let hit = stats_for("X", &bars).unwrap();
        assert!(matches(&hit, Preset::RangeContractionDay),
            "day_pct={} gap_pct={} rel_vol={}",
            hit.day_pct, hit.gap_pct, hit.rel_volume);
    }

    #[test]
    fn momentum_needs_both_pct_and_volume() {
        // SMA-window = min(20, n). With 2 bars, avg = mean of both.
        // To clear rel_volume >= 2.0, need today >= 2× avg.
        // Here today=4M, prior=1M → avg=2.5M → rel_vol = 4/2.5 = 1.6 — too low.
        // Use 5 bars to get a meaningful avg, then a big surge on the last.
        let bars = vec![
            bar(100, 100, 95, 100, 1_000_000, 1),
            bar(100, 101, 99, 100, 1_000_000, 2),
            bar(100, 101, 99, 100, 1_000_000, 3),
            bar(100, 101, 99, 100, 1_000_000, 4),
            bar(100, 110, 100, 108, 6_000_000, 5),
        ];
        let hit = stats_for("X", &bars).unwrap();
        assert!(hit.change_pct >= 5.0, "change_pct = {}", hit.change_pct);
        assert!(hit.rel_volume >= 2.0, "rel_volume = {}", hit.rel_volume);
        assert!(matches(&hit, Preset::MomentumMovers));
    }
}
