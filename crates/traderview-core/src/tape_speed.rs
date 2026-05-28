//! Tape Speed — print rate (events per second) and rate-of-change
//! used to detect sudden bursts of activity (news, halt-resume, etc.).
//!
//!   speed_t = events_in_bar_t / bar_seconds
//!   ema_t   = EMA(speed, period)
//!   spike_t = speed_t / ema_t                  (>1 = current bar faster than average)
//!
//! Caller supplies `events` (trade count per bar) and the duration of
//! each bar in seconds (constant or variable).
//!
//! Pure compute. Default period = 20.
//! Companion to `tape_density`, `volume_burst`, `halt_resume_monitor`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Bar { pub events: u64, pub bar_seconds: f64 }

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TapeSpeedReport {
    pub speed: Vec<Option<f64>>,
    pub ema_speed: Vec<Option<f64>>,
    pub spike_ratio: Vec<Option<f64>>,
    pub period: usize,
}

pub fn compute(bars: &[Bar], period: usize) -> TapeSpeedReport {
    let n = bars.len();
    let mut report = TapeSpeedReport {
        speed: vec![None; n],
        ema_speed: vec![None; n],
        spike_ratio: vec![None; n],
        period,
    };
    if period < 2 || n < period { return report; }
    if bars.iter().any(|b| !b.bar_seconds.is_finite() || b.bar_seconds <= 0.0) {
        return report;
    }
    let speeds: Vec<f64> = bars.iter().map(|b| b.events as f64 / b.bar_seconds).collect();
    for (i, &s) in speeds.iter().enumerate() {
        report.speed[i] = Some(s);
    }
    let p_f = period as f64;
    let k = 2.0 / (p_f + 1.0);
    let seed: f64 = speeds[..period].iter().sum::<f64>() / p_f;
    report.ema_speed[period - 1] = Some(seed);
    let mut cur = seed;
    for (i, &s) in speeds.iter().enumerate().skip(period) {
        cur = s * k + cur * (1.0 - k);
        report.ema_speed[i] = Some(cur);
    }
    for i in 0..n {
        if let (Some(s), Some(e)) = (report.speed[i], report.ema_speed[i]) {
            if e > 0.0 {
                report.spike_ratio[i] = Some(s / e);
            }
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(events: u64, secs: f64) -> Bar { Bar { events, bar_seconds: secs } }

    #[test]
    fn invalid_inputs_return_empty() {
        let bars = vec![b(100, 60.0); 30];
        let r = compute(&bars, 1);
        assert!(r.speed.iter().all(|x| x.is_none()));
        let bars2 = vec![b(100, 0.0); 30];
        let r2 = compute(&bars2, 14);
        assert!(r2.speed.iter().all(|x| x.is_none()));
    }

    #[test]
    fn constant_tape_yields_unit_spike() {
        // 100 events / 60 sec = 1.667 events/sec, constant.
        let bars = vec![b(100, 60.0); 30];
        let r = compute(&bars, 14);
        let last = 29;
        assert!((r.spike_ratio[last].unwrap() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn surge_yields_spike_above_one() {
        let mut bars = vec![b(100, 60.0); 30];
        bars.push(b(500, 60.0));
        let r = compute(&bars, 14);
        let last = bars.len() - 1;
        // Speed jumps from 1.67 to 8.33 → ratio > 1.
        assert!(r.spike_ratio[last].unwrap() > 1.5);
    }

    #[test]
    fn drop_yields_spike_below_one() {
        let mut bars = vec![b(100, 60.0); 30];
        bars.push(b(10, 60.0));
        let r = compute(&bars, 14);
        let last = bars.len() - 1;
        assert!(r.spike_ratio[last].unwrap() < 0.5);
    }

    #[test]
    fn output_lengths_match_input() {
        let bars = vec![b(100, 60.0); 30];
        let r = compute(&bars, 14);
        assert_eq!(r.speed.len(), 30);
        assert_eq!(r.ema_speed.len(), 30);
        assert_eq!(r.spike_ratio.len(), 30);
    }
}
