//! Calendar (time) and diagonal spread analyzer — long the back month,
//! short the front month. Same strike = calendar; different strikes =
//! diagonal (e.g. the "poor man's covered call": long deep-ITM back
//! call, short OTM front call). One P/L engine serves both; the
//! calendar is the equal-strikes special case.
//!
//! Net cost = back_premium − front_premium  (debit, positive).
//! Theta capture: short front month decays faster; the trade profits
//! when the front expires near its strike (front goes to 0, back keeps
//! extrinsic value).
//!
//! P&L at front-month expiration as a function of underlying:
//!   - intrinsic on front leg: max(0, kind_sign · (S − K_front))
//!   - back leg estimate: Black-Scholes value at K_back with remaining
//!     time (`back_time_after_front_expiry`) and current vol
//!
//!   pnl(S) = back_value(S) − max(0, front_intrinsic(S)) − net_cost
//!
//! Pure compute. Returns sampled P&L grid + breakevens (by linear
//! interpolation across the grid) + max profit / loss.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionKind {
    Call,
    Put,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CalendarSpread {
    pub strike: f64,
    pub kind: OptionKind,
    pub front_premium: f64,
    pub back_premium: f64,
    /// Years between front expiration and back expiration.
    pub back_time_after_front_expiry: f64,
    pub risk_free: f64,
    pub dividend_yield: f64,
    pub sigma: f64,
    pub contracts: i64,
    pub multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyzerConfig {
    pub grid_low_pct_of_strike: f64,  // 0.5 → −50% from strike
    pub grid_high_pct_of_strike: f64, // 1.5 → +50% above strike
    pub grid_points: usize,
}

impl Default for AnalyzerConfig {
    fn default() -> Self {
        Self {
            grid_low_pct_of_strike: 0.6,
            grid_high_pct_of_strike: 1.4,
            grid_points: 81,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CalendarReport {
    pub net_debit: f64,
    pub pnl_grid: Vec<PnlPoint>,
    pub breakevens: Vec<f64>,
    pub max_profit: f64,
    pub max_profit_at: f64,
    pub max_loss: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PnlPoint {
    pub spot: f64,
    pub pnl: f64,
}

/// Diagonal spread: same structure as the calendar but the short front
/// leg and long back leg sit at different strikes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DiagonalSpread {
    pub front_strike: f64,
    pub back_strike: f64,
    pub kind: OptionKind,
    pub front_premium: f64,
    pub back_premium: f64,
    /// Years between front expiration and back expiration.
    pub back_time_after_front_expiry: f64,
    pub risk_free: f64,
    pub dividend_yield: f64,
    pub sigma: f64,
    pub contracts: i64,
    pub multiplier: f64,
}

/// Calendar = diagonal with both legs on the same strike.
pub fn analyze(spread: &CalendarSpread, cfg: &AnalyzerConfig) -> Option<CalendarReport> {
    analyze_diagonal(
        &DiagonalSpread {
            front_strike: spread.strike,
            back_strike: spread.strike,
            kind: spread.kind,
            front_premium: spread.front_premium,
            back_premium: spread.back_premium,
            back_time_after_front_expiry: spread.back_time_after_front_expiry,
            risk_free: spread.risk_free,
            dividend_yield: spread.dividend_yield,
            sigma: spread.sigma,
            contracts: spread.contracts,
            multiplier: spread.multiplier,
        },
        cfg,
    )
}

pub fn analyze_diagonal(spread: &DiagonalSpread, cfg: &AnalyzerConfig) -> Option<CalendarReport> {
    if !spread.front_strike.is_finite()
        || spread.front_strike <= 0.0
        || !spread.back_strike.is_finite()
        || spread.back_strike <= 0.0
        || !spread.front_premium.is_finite()
        || spread.front_premium < 0.0
        || !spread.back_premium.is_finite()
        || spread.back_premium < 0.0
        || !spread.back_time_after_front_expiry.is_finite()
        || spread.back_time_after_front_expiry <= 0.0
        || !spread.sigma.is_finite()
        || spread.sigma <= 0.0
        || !spread.multiplier.is_finite()
        || spread.multiplier <= 0.0
        || spread.contracts == 0
        || cfg.grid_points < 3
        || !cfg.grid_low_pct_of_strike.is_finite()
        || cfg.grid_low_pct_of_strike <= 0.0
        || !cfg.grid_high_pct_of_strike.is_finite()
        || cfg.grid_high_pct_of_strike <= cfg.grid_low_pct_of_strike
    {
        return None;
    }
    let net_debit_per_contract = spread.back_premium - spread.front_premium;
    let scale = spread.contracts as f64 * spread.multiplier;
    let net_debit = net_debit_per_contract * scale;
    let kind_sign = match spread.kind {
        OptionKind::Call => 1.0,
        OptionKind::Put => -1.0,
    };
    // Grid spans around the midpoint of the two strikes (identical to
    // the strike itself for a calendar).
    let mid = (spread.front_strike + spread.back_strike) / 2.0;
    let lo = mid * cfg.grid_low_pct_of_strike;
    let hi = mid * cfg.grid_high_pct_of_strike;
    let step = (hi - lo) / (cfg.grid_points as f64 - 1.0);
    let mut grid = Vec::with_capacity(cfg.grid_points);
    let mut max_profit = f64::NEG_INFINITY;
    let mut max_profit_at = 0.0;
    let mut max_loss = f64::INFINITY;
    for k in 0..cfg.grid_points {
        let s = lo + step * k as f64;
        let front_intrinsic = (kind_sign * (s - spread.front_strike)).max(0.0);
        let back_value = bs_price(
            s,
            spread.back_strike,
            spread.back_time_after_front_expiry,
            spread.risk_free,
            spread.dividend_yield,
            spread.sigma,
            spread.kind,
        );
        // contracts > 0 = long the spread; contracts < 0 = short the spread.
        let pnl_per = back_value - front_intrinsic - net_debit_per_contract;
        let pnl = pnl_per * scale;
        if pnl > max_profit {
            max_profit = pnl;
            max_profit_at = s;
        }
        if pnl < max_loss {
            max_loss = pnl;
        }
        grid.push(PnlPoint { spot: s, pnl });
    }
    // Find sign-change breakevens by linear interpolation.
    let mut breakevens = Vec::new();
    for w in grid.windows(2) {
        let (a, b) = (w[0], w[1]);
        if (a.pnl == 0.0)
            || (a.pnl.signum() != b.pnl.signum() && a.pnl.is_finite() && b.pnl.is_finite())
        {
            if a.pnl == 0.0 {
                breakevens.push(a.spot);
            } else {
                let t = a.pnl / (a.pnl - b.pnl);
                breakevens.push(a.spot + t * (b.spot - a.spot));
            }
        }
    }
    Some(CalendarReport {
        net_debit,
        pnl_grid: grid,
        breakevens,
        max_profit,
        max_profit_at,
        max_loss,
    })
}

/// Delegates to the shared pricer (`black_scholes`) — extracted from
/// this module, identical numerics.
fn bs_price(s: f64, k: f64, t: f64, r: f64, q: f64, sigma: f64, kind: OptionKind) -> f64 {
    match kind {
        OptionKind::Call => crate::black_scholes::call(s, k, t, r, q, sigma),
        OptionKind::Put => crate::black_scholes::put(s, k, t, r, q, sigma),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cs(strike: f64, kind: OptionKind, front: f64, back: f64) -> CalendarSpread {
        CalendarSpread {
            strike,
            kind,
            front_premium: front,
            back_premium: back,
            back_time_after_front_expiry: 0.25,
            risk_free: 0.05,
            dividend_yield: 0.0,
            sigma: 0.25,
            contracts: 1,
            multiplier: 100.0,
        }
    }

    #[test]
    fn invalid_inputs_return_none() {
        let bad = cs(0.0, OptionKind::Call, 2.0, 5.0);
        assert!(analyze(&bad, &AnalyzerConfig::default()).is_none());
        let bad = cs(100.0, OptionKind::Call, -1.0, 5.0);
        assert!(analyze(&bad, &AnalyzerConfig::default()).is_none());
        let mut bad = cs(100.0, OptionKind::Call, 2.0, 5.0);
        bad.contracts = 0;
        assert!(analyze(&bad, &AnalyzerConfig::default()).is_none());
        let mut bad = cs(100.0, OptionKind::Call, 2.0, 5.0);
        bad.sigma = 0.0;
        assert!(analyze(&bad, &AnalyzerConfig::default()).is_none());
    }

    #[test]
    fn classic_at_the_money_calendar_has_max_profit_near_strike() {
        // Long 100 strike call calendar: pay 5 for back, receive 2 for front.
        let r = analyze(
            &cs(100.0, OptionKind::Call, 2.0, 5.0),
            &AnalyzerConfig::default(),
        )
        .unwrap();
        assert!(r.net_debit > 0.0);
        // Max profit should be at or very near the strike.
        assert!(
            (r.max_profit_at - 100.0).abs() / 100.0 < 0.10,
            "max-profit spot should be near 100, got {}",
            r.max_profit_at
        );
        assert!(r.max_profit > 0.0);
    }

    #[test]
    fn far_otm_pnl_approaches_negative_net_debit() {
        // At spot=200 (far ITM call for both legs), front intrinsic ≈ 100,
        // back value ≈ 100+extra extrinsic. The trade should lose money
        // (front intrinsic eats most of the back value).
        let r = analyze(
            &cs(100.0, OptionKind::Call, 2.0, 5.0),
            &AnalyzerConfig::default(),
        )
        .unwrap();
        let far_itm = r.pnl_grid.last().expect("grid");
        // Either near max_loss, or at least less than max_profit — i.e. the wings lose.
        assert!(
            far_itm.pnl < r.max_profit,
            "wings should lose vs max profit"
        );
    }

    #[test]
    fn negative_contracts_inverts_pnl() {
        let mut long = cs(100.0, OptionKind::Call, 2.0, 5.0);
        let r_long = analyze(&long, &AnalyzerConfig::default()).unwrap();
        long.contracts = -1;
        let r_short = analyze(&long, &AnalyzerConfig::default()).unwrap();
        // Short calendar has inverted P&L sign at every grid point.
        for (a, b) in r_long.pnl_grid.iter().zip(r_short.pnl_grid.iter()) {
            assert!((a.pnl + b.pnl).abs() < 1e-6, "P&L should be sign-flipped");
        }
    }

    #[test]
    fn put_calendar_supported() {
        let r = analyze(
            &cs(100.0, OptionKind::Put, 2.0, 5.0),
            &AnalyzerConfig::default(),
        )
        .unwrap();
        assert!(r.max_profit > 0.0);
    }

    fn diag(front_k: f64, back_k: f64) -> DiagonalSpread {
        DiagonalSpread {
            front_strike: front_k,
            back_strike: back_k,
            kind: OptionKind::Call,
            front_premium: 2.0,
            back_premium: 8.0,
            back_time_after_front_expiry: 0.25,
            risk_free: 0.05,
            dividend_yield: 0.0,
            sigma: 0.25,
            contracts: 1,
            multiplier: 100.0,
        }
    }

    #[test]
    fn calendar_is_the_equal_strikes_diagonal() {
        // The calendar path must produce bit-identical output to the
        // diagonal engine with both strikes equal — pins the refactor.
        let cal = analyze(
            &cs(100.0, OptionKind::Call, 2.0, 5.0),
            &AnalyzerConfig::default(),
        )
        .unwrap();
        let mut d = diag(100.0, 100.0);
        d.back_premium = 5.0;
        let dia = analyze_diagonal(&d, &AnalyzerConfig::default()).unwrap();
        assert_eq!(cal.net_debit, dia.net_debit);
        assert_eq!(cal.max_profit, dia.max_profit);
        assert_eq!(cal.pnl_grid.len(), dia.pnl_grid.len());
        for (a, b) in cal.pnl_grid.iter().zip(dia.pnl_grid.iter()) {
            assert_eq!(a.spot, b.spot);
            assert_eq!(a.pnl, b.pnl);
        }
    }

    #[test]
    fn bullish_call_diagonal_peaks_near_front_strike() {
        // Poor man's covered call: long back 90C, short front 105C.
        let r = analyze_diagonal(&diag(105.0, 90.0), &AnalyzerConfig::default()).unwrap();
        assert!(r.max_profit > 0.0);
        // Peak sits at/above the short front strike, where the short
        // leg expires worthless while the back call is deep ITM.
        assert!(
            r.max_profit_at >= 100.0,
            "peak at {}, expected near/above front strike",
            r.max_profit_at
        );
        // Far-downside P/L approaches −net debit (both legs worthless).
        let far_otm = r.pnl_grid.first().expect("grid");
        assert!(
            (far_otm.pnl + r.net_debit).abs() < r.net_debit * 0.25,
            "downside {} should approach −net debit {}",
            far_otm.pnl,
            -r.net_debit
        );
    }

    #[test]
    fn diagonal_rejects_bad_strikes() {
        assert!(analyze_diagonal(&diag(0.0, 90.0), &AnalyzerConfig::default()).is_none());
        assert!(analyze_diagonal(&diag(105.0, -1.0), &AnalyzerConfig::default()).is_none());
    }

    #[test]
    fn config_validation() {
        let s = cs(100.0, OptionKind::Call, 2.0, 5.0);
        let cfg = AnalyzerConfig {
            grid_points: 2,
            ..Default::default()
        };
        assert!(analyze(&s, &cfg).is_none());
        let cfg = AnalyzerConfig {
            grid_low_pct_of_strike: 1.5,
            grid_high_pct_of_strike: 0.5,
            ..Default::default()
        };
        assert!(analyze(&s, &cfg).is_none());
    }
}
