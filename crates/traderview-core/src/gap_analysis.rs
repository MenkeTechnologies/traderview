//! Gap classifier + gap-fill probability estimator.
//!
//! Gap = today's open vs yesterday's close. Categorize:
//!   - **Common gap**: < 0.5% — likely fills same day.
//!   - **Breakaway gap**: 0.5-2%, in trend direction.
//!   - **Runaway gap**: 2-4%, mid-trend continuation.
//!   - **Exhaustion gap**: > 4%, late-trend climax.
//!
//! Gap-fill probability heuristic — historically common gaps fill ~80%
//! same session; exhaustion gaps fill < 20% same session. Per-tier
//! probability lookup.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GapKind {
    NoGap,
    CommonUp,
    CommonDown,
    BreakawayUp,
    BreakawayDown,
    RunawayUp,
    RunawayDown,
    ExhaustionUp,
    ExhaustionDown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapReport {
    pub prior_close: f64,
    pub open: f64,
    pub gap_pct: f64,
    pub kind: GapKind,
    /// Historical probability the gap fills same session.
    pub same_session_fill_probability: f64,
}

pub fn classify(prior_close: f64, today_open: f64) -> GapReport {
    if prior_close <= 0.0 {
        return GapReport {
            prior_close,
            open: today_open,
            gap_pct: 0.0,
            kind: GapKind::NoGap,
            same_session_fill_probability: 0.0,
        };
    }
    let gap_pct = (today_open - prior_close) / prior_close;
    let abs_pct = gap_pct.abs();
    let kind = if abs_pct < 0.005 {
        if gap_pct.abs() < 0.001 {
            GapKind::NoGap
        } else if gap_pct > 0.0 {
            GapKind::CommonUp
        } else {
            GapKind::CommonDown
        }
    } else if abs_pct < 0.02 {
        if gap_pct > 0.0 {
            GapKind::BreakawayUp
        } else {
            GapKind::BreakawayDown
        }
    } else if abs_pct < 0.04 {
        if gap_pct > 0.0 {
            GapKind::RunawayUp
        } else {
            GapKind::RunawayDown
        }
    } else {
        if gap_pct > 0.0 {
            GapKind::ExhaustionUp
        } else {
            GapKind::ExhaustionDown
        }
    };
    let fill_prob = match kind {
        GapKind::NoGap => 1.0,
        GapKind::CommonUp | GapKind::CommonDown => 0.80,
        GapKind::BreakawayUp | GapKind::BreakawayDown => 0.50,
        GapKind::RunawayUp | GapKind::RunawayDown => 0.30,
        GapKind::ExhaustionUp | GapKind::ExhaustionDown => 0.15,
    };
    GapReport {
        prior_close,
        open: today_open,
        gap_pct,
        kind,
        same_session_fill_probability: fill_prob,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_prior_close_returns_no_gap() {
        let r = classify(0.0, 100.0);
        assert_eq!(r.kind, GapKind::NoGap);
        assert_eq!(r.same_session_fill_probability, 0.0);
    }

    #[test]
    fn tiny_open_change_classified_no_gap() {
        // 0.05% change → below 0.1% → NoGap.
        let r = classify(100.0, 100.05);
        assert_eq!(r.kind, GapKind::NoGap);
    }

    #[test]
    fn small_gap_up_classified_common_up() {
        // 0.3% gap up.
        let r = classify(100.0, 100.3);
        assert_eq!(r.kind, GapKind::CommonUp);
        assert_eq!(r.same_session_fill_probability, 0.80);
    }

    #[test]
    fn one_pct_gap_up_classified_breakaway() {
        let r = classify(100.0, 101.0);
        assert_eq!(r.kind, GapKind::BreakawayUp);
        assert_eq!(r.same_session_fill_probability, 0.50);
    }

    #[test]
    fn three_pct_gap_classified_runaway() {
        let r = classify(100.0, 103.0);
        assert_eq!(r.kind, GapKind::RunawayUp);
        assert_eq!(r.same_session_fill_probability, 0.30);
    }

    #[test]
    fn five_pct_gap_classified_exhaustion() {
        let r = classify(100.0, 105.0);
        assert_eq!(r.kind, GapKind::ExhaustionUp);
        assert_eq!(r.same_session_fill_probability, 0.15);
    }

    #[test]
    fn negative_gap_classified_down_variants() {
        assert_eq!(classify(100.0, 99.7).kind, GapKind::CommonDown);
        assert_eq!(classify(100.0, 99.0).kind, GapKind::BreakawayDown);
        assert_eq!(classify(100.0, 97.0).kind, GapKind::RunawayDown);
        assert_eq!(classify(100.0, 95.0).kind, GapKind::ExhaustionDown);
    }

    #[test]
    fn fill_probability_inversely_correlated_with_gap_size() {
        let common = classify(100.0, 100.3);
        let runaway = classify(100.0, 103.0);
        let exhaustion = classify(100.0, 110.0);
        assert!(common.same_session_fill_probability > runaway.same_session_fill_probability);
        assert!(runaway.same_session_fill_probability > exhaustion.same_session_fill_probability);
    }
}
