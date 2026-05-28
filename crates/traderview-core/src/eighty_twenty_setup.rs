//! 80-20 Setup — Linda Bradford Raschke ("Street Smarts").
//!
//! Counter-trend setup based on the position of open and close inside
//! a bar's full range:
//!
//! Bullish 80-20 (long):
//!   - Bar i's open is in the top 20% of its own range
//!     (open ≥ low + 0.80·range)
//!   - Bar i's close is in the bottom 20% of its own range
//!     (close ≤ low + 0.20·range)
//!   - Bar i sets a new N-bar low
//!   - Next bar (i+1) closes above bar i's low (intrabar reclaim)
//!
//! Bearish 80-20 (short): mirror — open in bottom 20%, close in top 20%,
//! new N-bar high, next bar closes below bar i's high.
//!
//! Pure compute. Default lookback = 20.
//! Companion to `turtle_soup`, `pinball_setup`, `key_reversal_bar`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub open: f64, pub high: f64, pub low: f64, pub close: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EightyTwentyReport {
    pub long_signal: Vec<bool>,
    pub short_signal: Vec<bool>,
    pub lookback: usize,
}

pub fn compute(bars: &[Bar], lookback: usize) -> EightyTwentyReport {
    let n = bars.len();
    let mut report = EightyTwentyReport {
        long_signal: vec![false; n],
        short_signal: vec![false; n],
        lookback,
    };
    if lookback < 2 || n < lookback + 2 { return report; }
    if bars.iter().any(|b| !b.open.is_finite() || !b.high.is_finite()
        || !b.low.is_finite() || !b.close.is_finite()) {
        return report;
    }
    for i in lookback..(n - 1) {
        let bar = bars[i];
        let next = bars[i + 1];
        let range = bar.high - bar.low;
        if range <= 0.0 { continue; }
        let open_pct = (bar.open - bar.low) / range;
        let close_pct = (bar.close - bar.low) / range;
        let win = &bars[i - lookback..i];
        let prior_high = win.iter().fold(f64::NEG_INFINITY, |a, b| a.max(b.high));
        let prior_low = win.iter().fold(f64::INFINITY, |a, b| a.min(b.low));
        // Bullish: open near high (≥ 0.80), close near low (≤ 0.20),
        // new low broken, next bar reclaims.
        if open_pct >= 0.80 && close_pct <= 0.20
            && bar.low < prior_low && next.close > bar.low {
            report.long_signal[i + 1] = true;
        }
        // Bearish: open near low (≤ 0.20), close near high (≥ 0.80),
        // new high taken out, next bar rejects.
        if open_pct <= 0.20 && close_pct >= 0.80
            && bar.high > prior_high && next.close < bar.high {
            report.short_signal[i + 1] = true;
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(o: f64, h: f64, l: f64, c: f64) -> Bar {
        Bar { open: o, high: h, low: l, close: c }
    }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 30];
        let r = compute(&bars, 1);
        assert!(!r.long_signal.iter().any(|x| *x));
        let r2 = compute(&bars[..10], 20);
        assert!(!r2.long_signal.iter().any(|x| *x));
    }

    #[test]
    fn nan_returns_empty() {
        let mut bars = vec![bar(100.0, 101.0, 99.0, 100.5); 30];
        bars[5] = bar(f64::NAN, 101.0, 99.0, 100.5);
        let r = compute(&bars, 20);
        assert!(!r.long_signal.iter().any(|x| *x));
    }

    #[test]
    fn flat_market_yields_no_signals() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.0); 30];
        let r = compute(&bars, 20);
        assert!(!r.long_signal.iter().any(|x| *x));
        assert!(!r.short_signal.iter().any(|x| *x));
    }

    #[test]
    fn bullish_eighty_twenty_triggers_long() {
        // 20 quiet bars (high=101, low=99), then breakdown bar with
        // open near high, close near low, NEW LOW set; next bar
        // closes above breakdown bar's low.
        let mut bars = vec![bar(100.0, 101.0, 99.0, 100.0); 20];
        // Breakdown bar: open=99.8, low=90, high=100 → range=10.
        //   open_pct = (99.8 - 90)/10 = 0.98 ≥ 0.80 ✓
        //   close=90.5 → close_pct = 0.05 ≤ 0.20 ✓
        //   low=90 < prior_low=99 ✓
        bars.push(bar(99.8, 100.0, 90.0, 90.5));
        // Reclaim bar: close > 90.
        bars.push(bar(91.0, 95.0, 90.5, 95.0));
        let r = compute(&bars, 20);
        assert!(r.long_signal[21]);
    }

    #[test]
    fn bearish_eighty_twenty_triggers_short() {
        let mut bars = vec![bar(100.0, 101.0, 99.0, 100.0); 20];
        // Blow-off bar: open=90.2, high=110, low=90, close=109.5
        //   range = 20.
        //   open_pct = (90.2 - 90)/20 = 0.01 ≤ 0.20 ✓
        //   close_pct = (109.5 - 90)/20 = 0.975 ≥ 0.80 ✓
        //   high=110 > prior_high=101 ✓
        bars.push(bar(90.2, 110.0, 90.0, 109.5));
        // Rejection bar: close < 110.
        bars.push(bar(109.0, 110.0, 105.0, 105.0));
        let r = compute(&bars, 20);
        assert!(r.short_signal[21]);
    }

    #[test]
    fn no_signal_when_no_new_extreme() {
        let mut bars = vec![bar(100.0, 101.0, 99.0, 100.0); 20];
        // Geometry matches but bar doesn't make new low.
        bars.push(bar(100.8, 101.0, 99.5, 99.6));
        bars.push(bar(100.0, 100.5, 99.5, 100.0));
        let r = compute(&bars, 20);
        assert!(!r.long_signal.iter().any(|x| *x));
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![bar(100.0, 101.0, 99.0, 100.5); 30];
        let r = compute(&bars, 20);
        assert_eq!(r.long_signal.len(), 30);
        assert_eq!(r.short_signal.len(), 30);
    }
}
