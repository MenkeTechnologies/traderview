//! Position sizing for confluence autotrade.
//!
//! Three modes:
//!
//!   * `FixedNotional` — flat $ per fire (legacy 0065 behavior).
//!   * `HalfKelly`     — 0.5 · f* applied to current equity, clamped to
//!                       `max_fraction`. Conservative practitioner's choice.
//!   * `QuarterKelly`  — 0.25 · f*, same clamp.
//!
//! Kelly is computed from the scanner backtest stats:
//!
//!     μ = scanner mean_return_pct / 100         (per-horizon mean)
//!     σ = scanner stdev_pct / 100               (per-horizon stdev)
//!     f* = μ / σ²                               (continuous, r_f = 0)
//!
//! When no scanner stats are available (e.g. the scanner hasn't been
//! backtested yet, or n < min_n), the sizer falls back to
//! `FixedNotional` and reports the fall-back reason so the audit log
//! shows exactly why Kelly couldn't be applied this tick.

use serde::{Deserialize, Serialize};
use traderview_core::kelly_criterion;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SizingMode {
    FixedNotional,
    HalfKelly,
    QuarterKelly,
}

impl SizingMode {
    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "fixed_notional" => Some(Self::FixedNotional),
            "half_kelly" => Some(Self::HalfKelly),
            "quarter_kelly" => Some(Self::QuarterKelly),
            _ => None,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FixedNotional => "fixed_notional",
            Self::HalfKelly => "half_kelly",
            Self::QuarterKelly => "quarter_kelly",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ScannerStats {
    pub mean_return_pct: f64, // per-horizon, raw % (e.g. 1.5 = 1.5%)
    pub stdev_pct: f64,
    pub n: usize,
}

#[derive(Debug, Clone)]
pub struct SizingDecision {
    pub notional_usd: f64,
    pub sizing_used: &'static str,
    pub kelly_fraction: Option<f64>,
    pub fallback_reason: Option<String>,
}

const MIN_N_FOR_KELLY: usize = 30;

/// Pure compute: pick a notional given mode, equity, scanner stats, cap.
/// Always returns *some* notional — falls back to `fixed_notional` when
/// Kelly inputs are unusable, so the autopilot doesn't silently stop.
pub fn size_notional(
    mode: SizingMode,
    equity_usd: f64,
    fixed_notional_usd: f64,
    stats: Option<ScannerStats>,
    max_fraction: f64,
) -> SizingDecision {
    let fallback = |reason: &str| SizingDecision {
        notional_usd: fixed_notional_usd,
        sizing_used: "fixed_notional",
        kelly_fraction: None,
        fallback_reason: Some(reason.into()),
    };

    if mode == SizingMode::FixedNotional {
        return SizingDecision {
            notional_usd: fixed_notional_usd,
            sizing_used: "fixed_notional",
            kelly_fraction: None,
            fallback_reason: None,
        };
    }

    let Some(s) = stats else {
        return fallback("no scanner stats available");
    };
    if s.n < MIN_N_FOR_KELLY {
        return fallback(&format!(
            "scanner n={} below min for Kelly ({MIN_N_FOR_KELLY})",
            s.n
        ));
    }
    if s.stdev_pct <= 0.0 {
        return fallback("scanner stdev <= 0");
    }
    if s.mean_return_pct <= 0.0 {
        return fallback(&format!(
            "scanner mean_return {:.3}% non-positive — no edge to size",
            s.mean_return_pct
        ));
    }
    let mu = s.mean_return_pct / 100.0;
    let sigma = s.stdev_pct / 100.0;
    let Some(k) = kelly_criterion::continuous(mu, sigma, 0.0) else {
        return fallback("kelly_criterion::continuous rejected inputs");
    };
    let frac = match mode {
        SizingMode::HalfKelly => k.half_kelly,
        SizingMode::QuarterKelly => k.quarter_kelly,
        SizingMode::FixedNotional => unreachable!(),
    };
    if !frac.is_finite() || frac <= 0.0 {
        return fallback("kelly fraction non-positive");
    }
    let clamped = frac.min(max_fraction);
    let notional = clamped * equity_usd;
    if notional <= 0.0 {
        return fallback("kelly notional non-positive");
    }
    SizingDecision {
        notional_usd: notional,
        sizing_used: mode.as_str(),
        kelly_fraction: Some(clamped),
        fallback_reason: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good_stats() -> ScannerStats {
        ScannerStats {
            mean_return_pct: 1.0, // 1.0% per horizon
            stdev_pct: 5.0,       // 5.0% per horizon
            n: 100,
        }
    }

    #[test]
    fn fixed_notional_returns_input_unchanged() {
        let d = size_notional(
            SizingMode::FixedNotional,
            100_000.0,
            1000.0,
            Some(good_stats()),
            0.05,
        );
        assert_eq!(d.notional_usd, 1000.0);
        assert_eq!(d.sizing_used, "fixed_notional");
        assert!(d.kelly_fraction.is_none());
        assert!(d.fallback_reason.is_none());
    }

    #[test]
    fn half_kelly_uses_continuous_formula() {
        // μ=0.01, σ=0.05, r_f=0 → f* = 0.01/0.0025 = 4.0
        // half_kelly = 2.0 → clamps to max_fraction = 0.05
        // notional = 0.05 * 100k = 5000.
        let d = size_notional(
            SizingMode::HalfKelly,
            100_000.0,
            1000.0,
            Some(good_stats()),
            0.05,
        );
        assert!(d.fallback_reason.is_none(), "{:?}", d.fallback_reason);
        assert_eq!(d.notional_usd, 5000.0);
        assert_eq!(d.sizing_used, "half_kelly");
        assert_eq!(d.kelly_fraction, Some(0.05));
    }

    #[test]
    fn quarter_kelly_uses_quarter_factor() {
        // Need stats where 0.25·f* is under the cap to verify the factor
        // actually applied. f* = 0.01/0.04 = 0.25, quarter = 0.0625 → cap.
        let stats = ScannerStats {
            mean_return_pct: 1.0,
            stdev_pct: 20.0, // -> f* = 0.01/0.04 = 0.25
            n: 100,
        };
        let d = size_notional(
            SizingMode::QuarterKelly,
            100_000.0,
            1000.0,
            Some(stats),
            0.50,
        );
        // f* = 0.25, quarter = 0.0625 (under 0.50 cap).
        assert!((d.kelly_fraction.unwrap() - 0.0625).abs() < 1e-12);
        assert!((d.notional_usd - 6250.0).abs() < 1e-9);
    }

    #[test]
    fn falls_back_when_no_stats() {
        let d = size_notional(SizingMode::HalfKelly, 100_000.0, 1000.0, None, 0.05);
        assert_eq!(d.notional_usd, 1000.0);
        assert_eq!(d.sizing_used, "fixed_notional");
        assert!(d.fallback_reason.unwrap().contains("no scanner stats"));
    }

    #[test]
    fn falls_back_when_n_too_low() {
        let stats = ScannerStats {
            mean_return_pct: 1.0,
            stdev_pct: 5.0,
            n: 10,
        };
        let d = size_notional(SizingMode::HalfKelly, 100_000.0, 1000.0, Some(stats), 0.05);
        assert_eq!(d.sizing_used, "fixed_notional");
        let r = d.fallback_reason.unwrap();
        assert!(r.contains("below min"));
        assert!(r.contains("n=10"));
    }

    #[test]
    fn falls_back_when_mean_non_positive() {
        let stats = ScannerStats {
            mean_return_pct: -0.5,
            stdev_pct: 5.0,
            n: 100,
        };
        let d = size_notional(SizingMode::HalfKelly, 100_000.0, 1000.0, Some(stats), 0.05);
        assert_eq!(d.sizing_used, "fixed_notional");
        assert!(d.fallback_reason.unwrap().contains("non-positive"));
    }

    #[test]
    fn clamps_to_max_fraction() {
        // Stats produce huge f*; verify the cap holds.
        let stats = ScannerStats {
            mean_return_pct: 5.0, // 5%
            stdev_pct: 5.0,
            n: 200,
        };
        // f* = 0.05/0.0025 = 20, half = 10 → cap at 0.02.
        let d = size_notional(SizingMode::HalfKelly, 100_000.0, 1000.0, Some(stats), 0.02);
        assert_eq!(d.kelly_fraction, Some(0.02));
        assert_eq!(d.notional_usd, 2000.0);
    }

    #[test]
    fn parses_round_trip() {
        for m in [
            SizingMode::FixedNotional,
            SizingMode::HalfKelly,
            SizingMode::QuarterKelly,
        ] {
            assert_eq!(SizingMode::parse(m.as_str()), Some(m));
        }
        assert!(SizingMode::parse("nonsense").is_none());
    }
}
