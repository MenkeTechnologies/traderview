//! Open-interest change-rate alerter.
//!
//! For each strike, compare today's OI to a rolling baseline (e.g.,
//! 20-day average). Strikes where OI surged in the past 1-3 days
//! are likely the focus of institutional positioning — useful as a
//! "what does smart money see" signal.
//!
//! Reports the strikes with the largest absolute and relative OI
//! changes, separated by call/put.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrikeOiSnapshot {
    pub strike: f64,
    pub call_oi: u64,
    pub put_oi: u64,
    pub call_oi_baseline: f64,
    pub put_oi_baseline: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OiAlert {
    pub strike: f64,
    pub kind: OiSide,
    pub current_oi: u64,
    pub baseline_oi: f64,
    pub abs_change: f64,
    pub pct_change: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OiSide {
    Call,
    Put,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OiAlertReport {
    pub call_alerts: Vec<OiAlert>,
    pub put_alerts: Vec<OiAlert>,
}

/// Emit alerts where |pct_change| ≥ threshold AND absolute size ≥ min_oi.
pub fn analyze(snapshots: &[StrikeOiSnapshot], pct_threshold: f64, min_oi: u64) -> OiAlertReport {
    let mut report = OiAlertReport::default();
    for s in snapshots {
        for (kind, current, baseline) in [
            (OiSide::Call, s.call_oi, s.call_oi_baseline),
            (OiSide::Put, s.put_oi, s.put_oi_baseline),
        ] {
            if current < min_oi {
                continue;
            }
            let abs = current as f64 - baseline;
            let pct = if baseline > 0.0 { abs / baseline } else { 0.0 };
            if pct.abs() < pct_threshold {
                continue;
            }
            let alert = OiAlert {
                strike: s.strike,
                kind,
                current_oi: current,
                baseline_oi: baseline,
                abs_change: abs,
                pct_change: pct,
            };
            match kind {
                OiSide::Call => report.call_alerts.push(alert),
                OiSide::Put => report.put_alerts.push(alert),
            }
        }
    }
    // Sort biggest absolute change first.
    report.call_alerts.sort_by(|a, b| {
        b.abs_change
            .abs()
            .partial_cmp(&a.abs_change.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    report.put_alerts.sort_by(|a, b| {
        b.abs_change
            .abs()
            .partial_cmp(&a.abs_change.abs())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(strike: f64, c: u64, p: u64, cb: f64, pb: f64) -> StrikeOiSnapshot {
        StrikeOiSnapshot {
            strike,
            call_oi: c,
            put_oi: p,
            call_oi_baseline: cb,
            put_oi_baseline: pb,
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[], 0.5, 100);
        assert!(r.call_alerts.is_empty());
    }

    #[test]
    fn doubling_call_oi_triggers_alert_at_50pct_threshold() {
        // Baseline 1000, current 2500 → +150%.
        let snaps = vec![s(100.0, 2500, 0, 1000.0, 0.0)];
        let r = analyze(&snaps, 0.5, 100);
        assert_eq!(r.call_alerts.len(), 1);
        assert!((r.call_alerts[0].pct_change - 1.5).abs() < 1e-9);
    }

    #[test]
    fn under_threshold_no_alert() {
        // +20% from baseline at 50% threshold → no alert.
        let snaps = vec![s(100.0, 1200, 0, 1000.0, 0.0)];
        let r = analyze(&snaps, 0.5, 100);
        assert!(r.call_alerts.is_empty());
    }

    #[test]
    fn min_oi_filters_low_volume_strikes() {
        // OI of only 50 < 100 min → ignored even though pct change huge.
        let snaps = vec![s(100.0, 50, 0, 10.0, 0.0)];
        let r = analyze(&snaps, 0.5, 100);
        assert!(r.call_alerts.is_empty());
    }

    #[test]
    fn calls_and_puts_segregated() {
        let snaps = vec![s(100.0, 2000, 500, 1000.0, 1000.0)];
        let r = analyze(&snaps, 0.4, 100);
        // Calls: 2x baseline → +100%, alert. Puts: -50% → also alert at 40% threshold.
        assert_eq!(r.call_alerts.len(), 1);
        assert_eq!(r.put_alerts.len(), 1);
        assert!(r.put_alerts[0].pct_change < 0.0);
    }

    #[test]
    fn alerts_sorted_largest_abs_change_first() {
        let snaps = vec![
            s(100.0, 2000, 0, 1000.0, 0.0), // +1000 change
            s(105.0, 5000, 0, 1000.0, 0.0), // +4000 change
            s(110.0, 1700, 0, 1000.0, 0.0), // +700 change
        ];
        let r = analyze(&snaps, 0.5, 100);
        assert_eq!(r.call_alerts.len(), 3);
        assert_eq!(r.call_alerts[0].strike, 105.0);
        assert_eq!(r.call_alerts[1].strike, 100.0);
        assert_eq!(r.call_alerts[2].strike, 110.0);
    }

    #[test]
    fn zero_baseline_pct_change_zero_no_alert() {
        // Baseline 0 — can't compute meaningful % → filtered out.
        let snaps = vec![s(100.0, 5000, 0, 0.0, 0.0)];
        let r = analyze(&snaps, 0.5, 100);
        assert!(r.call_alerts.is_empty());
    }
}
