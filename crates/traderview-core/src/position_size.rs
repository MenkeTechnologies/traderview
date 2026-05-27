//! Position sizing — three industry-standard methods + correlation drag.
//!
//! All methods take (entry, stop) and return:
//!   * shares to buy (rounded down to whole shares for stocks; the caller
//!     can post-round to lot/contract size)
//!   * dollar risk (= shares * risk_per_share)
//!   * position notional (= shares * entry)
//!   * position % of equity
//!
//! Methods:
//!   * Fixed-fractional — risk a fixed % of equity per trade. Industry
//!     default 0.5%–2%. Independent of expectancy.
//!   * R-based — risk a fixed dollar amount per trade. Useful for
//!     consistent R-multiple journaling.
//!   * Kelly — optimal-growth fraction given win_rate p, payoff ratio
//!     b = avg_win / |avg_loss|. f* = (p*b - q) / b where q = 1-p.
//!     We clamp f* to 0 when negative (no edge) and apply an optional
//!     fractional-Kelly multiplier (default 0.5 since full Kelly is
//!     famously brutal in real drawdowns).
//!
//! Correlation drag: when a `correlations` vector is supplied (one entry
//! per open position symbol vs the candidate), the final share count is
//! multiplied by 1 / (1 + sum(max(0, corr))). Negatively-correlated
//! positions don't penalize sizing — they hedge.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Long,
    Short,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Inputs {
    pub side: Side,
    pub entry: f64,
    pub stop: f64,
    pub equity: f64,
    /// Sum of |corr| with open positions; engine multiplies share count by
    /// 1 / (1 + this). 0 = no other positions.
    #[serde(default)]
    pub correlation_drag: f64,
    /// Hard cap as % of equity any single position can occupy. 0 = no cap.
    #[serde(default)]
    pub max_position_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct Sizing {
    pub method: &'static str,
    pub shares: i64,
    pub risk_per_share: f64,
    pub risk_dollars: f64,
    pub notional: f64,
    pub position_pct_of_equity: f64,
    pub correlation_multiplier: f64,
    /// Capped by max_position_pct if it triggered.
    pub capped_by_position_pct: bool,
    /// Method-specific notes for the user.
    pub note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Report {
    pub inputs: Inputs,
    pub risk_per_share: f64,
    pub fixed_fractional: Option<Sizing>,
    pub r_based: Option<Sizing>,
    pub kelly: Option<Sizing>,
    /// Whatever the user passed as their preferred default method.
    pub recommended: Option<Sizing>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct FixedFractionalParams {
    pub risk_pct: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RBasedParams {
    pub risk_dollars: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct KellyParams {
    pub win_rate: f64,         // 0..1
    pub avg_win: f64,          // dollars (or R, must match avg_loss unit)
    pub avg_loss: f64,         // positive magnitude
    pub fractional_kelly: f64, // 0.25 / 0.5 / 1.0 typical
}

pub fn risk_per_share(side: Side, entry: f64, stop: f64) -> f64 {
    match side {
        Side::Long => (entry - stop).abs(),
        Side::Short => (stop - entry).abs(),
    }
}

pub fn fixed_fractional(i: Inputs, p: FixedFractionalParams) -> Option<Sizing> {
    let rps = risk_per_share(i.side, i.entry, i.stop);
    if rps <= 0.0 || i.equity <= 0.0 {
        return None;
    }
    let dollar_risk = i.equity * p.risk_pct;
    // Small epsilon nudge so e.g. 1999.9999... from 0.6*2.0-0.4 rounds to 2000
    // rather than truncating away the user's intended whole-share count.
    let raw = (dollar_risk / rps + 1e-6).floor() as i64;
    Some(finalize(
        "fixed_fractional",
        i,
        rps,
        raw,
        format!(
            "risking {:.2}% of equity (${:.2})",
            p.risk_pct * 100.0,
            dollar_risk
        ),
    ))
}

pub fn r_based(i: Inputs, p: RBasedParams) -> Option<Sizing> {
    let rps = risk_per_share(i.side, i.entry, i.stop);
    if rps <= 0.0 || p.risk_dollars <= 0.0 {
        return None;
    }
    let raw = (p.risk_dollars / rps).floor() as i64;
    Some(finalize(
        "r_based",
        i,
        rps,
        raw,
        format!("fixed ${:.2} R per trade", p.risk_dollars),
    ))
}

pub fn kelly(i: Inputs, p: KellyParams) -> Option<Sizing> {
    let rps = risk_per_share(i.side, i.entry, i.stop);
    if rps <= 0.0 || i.equity <= 0.0 {
        return None;
    }
    if !(0.0..=1.0).contains(&p.win_rate) {
        return None;
    }
    if p.avg_loss <= 0.0 {
        return None;
    }
    let b = p.avg_win / p.avg_loss;
    let f_star = (p.win_rate * b - (1.0 - p.win_rate)) / b;
    let f = (f_star * p.fractional_kelly).clamp(0.0, 1.0);
    let dollar_risk = i.equity * f;
    // Small epsilon nudge so e.g. 1999.9999... from 0.6*2.0-0.4 rounds to 2000
    // rather than truncating away the user's intended whole-share count.
    let raw = (dollar_risk / rps + 1e-6).floor() as i64;
    let note = if f_star <= 0.0 {
        "Kelly = 0 (no edge — skip)".to_string()
    } else {
        format!(
            "Kelly f* = {:.3}, applying {:.0}%-Kelly → risking ${:.2}",
            f_star,
            p.fractional_kelly * 100.0,
            dollar_risk
        )
    };
    Some(finalize("kelly", i, rps, raw, note))
}

fn finalize(
    method: &'static str,
    i: Inputs,
    rps: f64,
    raw_shares: i64,
    mut note: String,
) -> Sizing {
    let drag_mul = if i.correlation_drag > 0.0 {
        1.0 / (1.0 + i.correlation_drag)
    } else {
        1.0
    };
    let mut shares = ((raw_shares as f64) * drag_mul).floor() as i64;
    if shares < 0 {
        shares = 0;
    }

    let mut capped = false;
    if i.max_position_pct > 0.0 && i.equity > 0.0 && i.entry > 0.0 {
        let max_notional = i.equity * i.max_position_pct;
        let max_shares = (max_notional / i.entry).floor() as i64;
        if shares > max_shares {
            shares = max_shares.max(0);
            capped = true;
            note.push_str(&format!(
                " · capped at {:.1}% of equity",
                i.max_position_pct * 100.0
            ));
        }
    }

    let risk_dollars = shares as f64 * rps;
    let notional = shares as f64 * i.entry;
    let pct = if i.equity > 0.0 {
        notional / i.equity
    } else {
        0.0
    };
    Sizing {
        method,
        shares,
        risk_per_share: rps,
        risk_dollars,
        notional,
        position_pct_of_equity: pct,
        correlation_multiplier: drag_mul,
        capped_by_position_pct: capped,
        note,
    }
}

/// Run all three methods. `kelly_params` is optional — when None, the Kelly
/// sizing slot is filled with None.
pub fn report(
    i: Inputs,
    ff: Option<FixedFractionalParams>,
    rb: Option<RBasedParams>,
    kp: Option<KellyParams>,
    recommended_method: Option<&str>,
) -> Report {
    let rps = risk_per_share(i.side, i.entry, i.stop);
    let mut warnings = Vec::new();
    if rps <= 0.0 {
        warnings.push("stop equals entry — risk per share is zero; sizing is undefined".into());
    }
    if i.equity <= 0.0 {
        warnings.push("equity is zero or negative — supply a positive account balance".into());
    }
    if i.correlation_drag > 0.0 {
        warnings.push(format!(
            "correlation drag {:.2} reduces share count by {:.0}%",
            i.correlation_drag,
            (1.0 - 1.0 / (1.0 + i.correlation_drag)) * 100.0
        ));
    }
    let fixed_fractional = ff.and_then(|p| fixed_fractional(i, p));
    let r_based = rb.and_then(|p| r_based(i, p));
    let kelly = kp.and_then(|p| kelly(i, p));
    let recommended = match recommended_method {
        Some("fixed_fractional") => fixed_fractional.clone(),
        Some("r_based") => r_based.clone(),
        Some("kelly") => kelly.clone(),
        _ => fixed_fractional.clone(),
    };
    Report {
        inputs: i,
        risk_per_share: rps,
        fixed_fractional,
        r_based,
        kelly,
        recommended,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_fractional_long_basic() {
        // $100k equity, 1% risk, entry 50, stop 48 → rps=2, dollar_risk=1000,
        // shares = 500.
        let i = Inputs {
            side: Side::Long,
            entry: 50.0,
            stop: 48.0,
            equity: 100_000.0,
            correlation_drag: 0.0,
            max_position_pct: 0.0,
        };
        let s = fixed_fractional(i, FixedFractionalParams { risk_pct: 0.01 }).unwrap();
        assert_eq!(s.shares, 500);
        assert!((s.risk_dollars - 1000.0).abs() < 1e-6);
        assert!((s.notional - 25_000.0).abs() < 1e-6);
    }

    #[test]
    fn correlation_drag_halves_size_at_correlation_one() {
        // drag = 1 → multiplier = 1/2 → shares should halve.
        let i = Inputs {
            side: Side::Long,
            entry: 50.0,
            stop: 48.0,
            equity: 100_000.0,
            correlation_drag: 1.0,
            max_position_pct: 0.0,
        };
        let s = fixed_fractional(i, FixedFractionalParams { risk_pct: 0.01 }).unwrap();
        assert_eq!(s.shares, 250, "1.0 drag should halve from 500 → 250");
    }

    #[test]
    fn position_cap_clamps() {
        // 100k equity, entry 50, 20% cap → 10k notional → 200 shares cap.
        // Without cap, fixed_fractional@5% gives 5000/2 = 2500 shares.
        let i = Inputs {
            side: Side::Long,
            entry: 50.0,
            stop: 48.0,
            equity: 100_000.0,
            correlation_drag: 0.0,
            max_position_pct: 0.20,
        };
        let s = fixed_fractional(i, FixedFractionalParams { risk_pct: 0.05 }).unwrap();
        // 100k * 20% = 20k notional, entry 50 → 400 shares cap.
        assert_eq!(s.shares, 400);
        assert!(s.capped_by_position_pct);
    }

    #[test]
    fn kelly_no_edge_returns_zero() {
        // win 40%, avg_win=1, avg_loss=1 → f* = -.2; clamp to 0.
        let i = Inputs {
            side: Side::Long,
            entry: 10.0,
            stop: 9.0,
            equity: 10_000.0,
            correlation_drag: 0.0,
            max_position_pct: 0.0,
        };
        let k = KellyParams {
            win_rate: 0.4,
            avg_win: 1.0,
            avg_loss: 1.0,
            fractional_kelly: 1.0,
        };
        let s = kelly(i, k).unwrap();
        assert_eq!(s.shares, 0);
        assert!(s.note.contains("no edge"));
    }

    #[test]
    fn kelly_with_edge_sizes_positive() {
        // win 60%, win=2, loss=1 → b=2, f* = (1.2 - .4)/2 = .4; half-Kelly = .2
        // → dollar_risk = 2000; rps = 1 → 2000 shares.
        let i = Inputs {
            side: Side::Long,
            entry: 10.0,
            stop: 9.0,
            equity: 10_000.0,
            correlation_drag: 0.0,
            max_position_pct: 0.0,
        };
        let k = KellyParams {
            win_rate: 0.6,
            avg_win: 2.0,
            avg_loss: 1.0,
            fractional_kelly: 0.5,
        };
        let s = kelly(i, k).unwrap();
        assert_eq!(s.shares, 2000);
    }
}
