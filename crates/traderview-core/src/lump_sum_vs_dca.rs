//! Lump-sum vs dollar-cost-averaging — the Vanguard "lump-sum beats DCA ~2/3 of
//! the time" comparison. Given a total to invest, a DCA cadence (N months), a
//! horizon (H months), an expected annual market return, and the cash-drag rate
//! the un-invested money earns: lump-sum puts the whole amount in at month 0 and
//! compounds it H months; DCA drips an equal slice each month for N months while
//! the waiting cash earns the (lower) cash rate. Reports both terminal values,
//! the gap, and the break-even market return below which DCA wins (cash drag
//! becomes less of a penalty than the market it missed). Faithful port of the
//! former client-side calculator. Pure compute, not advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct LumpSumVsDcaInput {
    pub total_usd: f64,
    /// DCA over this many months.
    pub dca_months: u32,
    /// Horizon in months (≥ dca_months).
    pub horizon_months: u32,
    pub expected_annual_return_pct: f64,
    pub cash_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct LumpSumVsDcaReport {
    pub lump_end_usd: f64,
    pub dca_end_usd: f64,
    pub gap_usd: f64,
    pub gap_pct: f64,
    /// "LUMP-SUM", "DCA", or "TIE".
    pub winner: String,
    /// Annual market return (%) where the two terminal values cross; below it DCA
    /// wins. None if it can't be bracketed.
    pub breakeven_return_pct: Option<f64>,
    pub valid: bool,
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

fn lump_end(total: f64, horizon: u32, r_ann: f64) -> f64 {
    let r = (1.0 + r_ann).powf(1.0 / 12.0) - 1.0;
    total * (1.0 + r).powi(horizon as i32)
}

fn simulate_dca(total: f64, dca_months: u32, horizon: u32, r_ann: f64, c_ann: f64) -> f64 {
    let r = (1.0 + r_ann).powf(1.0 / 12.0) - 1.0;
    let c = (1.0 + c_ann).powf(1.0 / 12.0) - 1.0;
    let per_month = total / dca_months as f64;
    let mut invested = 0.0;
    let mut cash = total;
    for m in 1..=horizon {
        invested *= 1.0 + r;
        cash *= 1.0 + c;
        if m <= dca_months {
            let drop = per_month.min(cash);
            cash -= drop;
            invested += drop;
        }
    }
    invested + cash
}

/// Binary search for the annual market return where DCA and lump-sum terminal
/// values meet. Both are monotonic in the return; lump grows faster, so above
/// the break-even lump wins. Mirrors the client's 60-iteration search over
/// [-30%, +50%].
fn find_breakeven(total: f64, dca_months: u32, horizon: u32, c_ann: f64) -> f64 {
    let mut lo = -0.30;
    let mut hi = 0.50;
    for _ in 0..60 {
        let mid = (lo + hi) / 2.0;
        let lump = lump_end(total, horizon, mid);
        let dca = simulate_dca(total, dca_months, horizon, mid, c_ann);
        if (lump - dca).abs() < 1.0 {
            return mid;
        }
        if lump > dca {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    (lo + hi) / 2.0
}

pub fn generate(i: &LumpSumVsDcaInput) -> LumpSumVsDcaReport {
    if i.total_usd <= 0.0 || i.dca_months < 2 || i.horizon_months < i.dca_months {
        return LumpSumVsDcaReport::default();
    }
    let r_ann = i.expected_annual_return_pct / 100.0;
    let c_ann = i.cash_rate_pct / 100.0;
    let le = lump_end(i.total_usd, i.horizon_months, r_ann);
    let de = simulate_dca(i.total_usd, i.dca_months, i.horizon_months, r_ann, c_ann);
    let gap = le - de;
    let winner = if gap > 0.0 {
        "LUMP-SUM"
    } else if gap < 0.0 {
        "DCA"
    } else {
        "TIE"
    };
    let breakeven = find_breakeven(i.total_usd, i.dca_months, i.horizon_months, c_ann);

    LumpSumVsDcaReport {
        lump_end_usd: round2(le),
        dca_end_usd: round2(de),
        gap_usd: round2(gap),
        gap_pct: round4(if de != 0.0 { gap / de * 100.0 } else { 0.0 }),
        winner: winner.to_string(),
        breakeven_return_pct: Some(round4(breakeven * 100.0)),
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> LumpSumVsDcaInput {
        LumpSumVsDcaInput {
            total_usd: 120_000.0,
            dca_months: 12,
            horizon_months: 120,
            expected_annual_return_pct: 7.0,
            cash_rate_pct: 4.0,
        }
    }

    // Pins cross-checked against the JS compute() in Python.
    #[test]
    fn default_lump_wins() {
        let d = generate(&base());
        assert!(d.valid);
        assert!(close(d.lump_end_usd, 236_058.16));
        assert!(close(d.dca_end_usd, 231_329.50));
        assert!(close(d.gap_usd, 4_728.67));
        assert!(close(d.gap_pct, 2.0441));
        assert_eq!(d.winner, "LUMP-SUM");
        // Break-even equals the cash rate: when the market matches cash, investing
        // early carries no advantage.
        assert!(close(d.breakeven_return_pct.unwrap(), 4.0002));
    }

    #[test]
    fn negative_return_dca_wins() {
        let d = generate(&LumpSumVsDcaInput { expected_annual_return_pct: -5.0, ..base() });
        assert_eq!(d.winner, "DCA");
        assert!(d.dca_end_usd > d.lump_end_usd);
    }

    #[test]
    fn invalid_when_horizon_below_dca() {
        let d = generate(&LumpSumVsDcaInput { horizon_months: 6, dca_months: 12, ..base() });
        assert!(!d.valid);
    }

    #[test]
    fn invalid_when_total_zero() {
        let d = generate(&LumpSumVsDcaInput { total_usd: 0.0, ..base() });
        assert!(!d.valid);
    }
}
