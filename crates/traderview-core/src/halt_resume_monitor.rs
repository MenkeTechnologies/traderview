//! Halt-Resume Monitor — track LULD / news-halt / volatility-pause events
//! and classify the resume-bar pattern for "halt-go" trading setups.
//!
//! Real-world workflow: an exchange halt resumes after N minutes. The
//! reopen auction sets a new price; the next 1-5 bars of trading often
//! gap further in the halt direction (continuation) or sharply reverse
//! (rejection). This module classifies the post-resume bar sequence to
//! flag continuation candidates for the "halt continuation" play.
//!
//!   Resume bar = first trading bar after the halt.
//!   Direction  = sign(resume_close − halt_pause_price).
//!   Confirmation window = next `confirm_bars` bars.
//!
//! Classification:
//!   - **Continuation**: post-resume bars extend direction by ≥ `min_followthrough_pct`
//!   - **Rejection**: post-resume bars reverse by ≥ `min_followthrough_pct`
//!   - **Indecision**: neither side hits the threshold
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HaltEvent {
    pub symbol: String,
    pub halt_pause_price: f64,
    /// Bars after the halt resumed, oldest first.
    pub resume_bars: Vec<ResumeBar>,
    pub halt_reason: HaltReason,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ResumeBar {
    pub close: f64,
    pub volume: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HaltReason {
    Luld,        // limit-up / limit-down
    News,
    Volatility,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub confirm_bars: usize,
    pub min_followthrough_pct: f64,
}

impl Default for Config {
    fn default() -> Self { Self { confirm_bars: 3, min_followthrough_pct: 0.02 } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Continuation,
    Rejection,
    Indecision,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedHalt {
    pub symbol: String,
    pub halt_reason: HaltReason,
    pub direction_pct: f64,
    pub followthrough_pct: f64,
    pub verdict: Verdict,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MonitorReport {
    pub classified: Vec<ClassifiedHalt>,
    pub continuations: Vec<String>,
    pub rejections: Vec<String>,
}

pub fn analyze(events: &[HaltEvent], cfg: &Config) -> MonitorReport {
    let mut report = MonitorReport::default();
    if cfg.confirm_bars == 0
        || !cfg.min_followthrough_pct.is_finite()
        || cfg.min_followthrough_pct <= 0.0
    {
        return report;
    }
    for e in events {
        if !e.halt_pause_price.is_finite() || e.halt_pause_price <= 0.0 {
            continue;
        }
        if e.resume_bars.is_empty() {
            continue;
        }
        let resume_close = e.resume_bars[0].close;
        if !resume_close.is_finite() {
            continue;
        }
        let direction_pct = (resume_close - e.halt_pause_price) / e.halt_pause_price;
        if !direction_pct.is_finite() { continue; }
        // Follow-through = highest extension in direction over next N bars.
        let look_end = (1 + cfg.confirm_bars).min(e.resume_bars.len());
        let mut max_with_dir = 0.0_f64;
        let mut max_against_dir = 0.0_f64;
        for bar in &e.resume_bars[1..look_end] {
            if !bar.close.is_finite() { continue; }
            let move_pct = (bar.close - resume_close) / resume_close;
            if !move_pct.is_finite() { continue; }
            if direction_pct >= 0.0 {
                if move_pct > max_with_dir { max_with_dir = move_pct; }
                if -move_pct > max_against_dir { max_against_dir = -move_pct; }
            } else {
                if -move_pct > max_with_dir { max_with_dir = -move_pct; }
                if move_pct > max_against_dir { max_against_dir = move_pct; }
            }
        }
        let verdict = if max_with_dir >= cfg.min_followthrough_pct {
            Verdict::Continuation
        } else if max_against_dir >= cfg.min_followthrough_pct {
            Verdict::Rejection
        } else {
            Verdict::Indecision
        };
        let followthrough_pct = if direction_pct >= 0.0 { max_with_dir } else { -max_with_dir };
        report.classified.push(ClassifiedHalt {
            symbol: e.symbol.clone(),
            halt_reason: e.halt_reason,
            direction_pct,
            followthrough_pct,
            verdict,
        });
        match verdict {
            Verdict::Continuation => report.continuations.push(e.symbol.clone()),
            Verdict::Rejection => report.rejections.push(e.symbol.clone()),
            Verdict::Indecision => {}
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(c: f64, v: f64) -> ResumeBar { ResumeBar { close: c, volume: v } }

    fn ev(sym: &str, pause: f64, bars: Vec<ResumeBar>) -> HaltEvent {
        HaltEvent {
            symbol: sym.into(),
            halt_pause_price: pause,
            resume_bars: bars,
            halt_reason: HaltReason::Luld,
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[], &Config::default());
        assert!(r.classified.is_empty());
    }

    #[test]
    fn invalid_config_returns_default() {
        let events = vec![ev("X", 100.0, vec![bar(105.0, 1_000.0); 5])];
        for cfg in [
            Config { confirm_bars: 0, ..Default::default() },
            Config { min_followthrough_pct: 0.0, ..Default::default() },
            Config { min_followthrough_pct: f64::NAN, ..Default::default() },
        ] {
            assert!(analyze(&events, &cfg).classified.is_empty());
        }
    }

    #[test]
    fn missing_resume_bars_skipped() {
        let r = analyze(&[ev("X", 100.0, vec![])], &Config::default());
        assert!(r.classified.is_empty());
    }

    #[test]
    fn upward_halt_with_followthrough_classified_continuation() {
        // Pause @ 100, resume @ 105 (+5%), then 107, 110, 112.
        let bars = vec![bar(105.0, 100.0), bar(107.0, 100.0), bar(110.0, 100.0), bar(112.0, 100.0)];
        let r = analyze(&[ev("UP", 100.0, bars)], &Config::default());
        assert_eq!(r.classified[0].verdict, Verdict::Continuation);
        assert!(r.continuations.contains(&"UP".to_string()));
    }

    #[test]
    fn upward_halt_with_reversal_classified_rejection() {
        // Pause @ 100, resume @ 105, then 102, 100, 99 — gives back.
        let bars = vec![bar(105.0, 100.0), bar(102.0, 100.0), bar(100.0, 100.0), bar(99.0, 100.0)];
        let r = analyze(&[ev("REV", 100.0, bars)], &Config::default());
        assert_eq!(r.classified[0].verdict, Verdict::Rejection);
        assert!(r.rejections.contains(&"REV".to_string()));
    }

    #[test]
    fn small_followthrough_yields_indecision() {
        // 105 → 105.5 → 105.3 → 104.9 (all < 2% from 105).
        let bars = vec![bar(105.0, 100.0), bar(105.5, 100.0), bar(105.3, 100.0), bar(104.9, 100.0)];
        let r = analyze(&[ev("CHOP", 100.0, bars)], &Config::default());
        assert_eq!(r.classified[0].verdict, Verdict::Indecision);
    }

    #[test]
    fn downward_halt_continuation_detected() {
        // Pause @ 100, resume @ 95, then 92, 90, 88.
        let bars = vec![bar(95.0, 100.0), bar(92.0, 100.0), bar(90.0, 100.0), bar(88.0, 100.0)];
        let r = analyze(&[ev("DROP", 100.0, bars)], &Config::default());
        assert_eq!(r.classified[0].verdict, Verdict::Continuation);
        assert!(r.classified[0].direction_pct < 0.0);
    }

    #[test]
    fn zero_pause_price_skipped() {
        let bars = vec![bar(95.0, 100.0); 4];
        let r = analyze(&[ev("X", 0.0, bars)], &Config::default());
        assert!(r.classified.is_empty());
    }

    #[test]
    fn nan_resume_close_skipped() {
        let bars = vec![bar(f64::NAN, 100.0), bar(92.0, 100.0)];
        let r = analyze(&[ev("X", 100.0, bars)], &Config::default());
        assert!(r.classified.is_empty());
    }
}
