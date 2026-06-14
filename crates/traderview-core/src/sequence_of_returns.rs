//! Sequence-of-returns risk — the same set of annual returns, in four different
//! orderings, run against an inflation-adjusted withdrawal. Forward (as given),
//! reversed, worst-years-first, and best-years-first all share the identical
//! arithmetic mean return yet can end with wildly different balances, because
//! withdrawing from a depleted portfolio compounds early losses. Faithful port
//! of the former client-side simulator. Pure compute, not advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SequenceInput {
    /// Annual returns in percent (e.g. -9.1 = −9.1%).
    pub returns_pct: Vec<f64>,
    pub start_balance_usd: f64,
    pub annual_withdrawal_usd: f64,
    /// Annual inflation adjustment to the withdrawal, percent.
    #[serde(default)]
    pub inflation_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PathYear {
    pub year: u32,
    pub return_pct: f64,
    pub close_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Scenario {
    pub name: String,
    pub end_balance_usd: f64,
    /// First year the balance hit zero, if it failed.
    pub failed_at_year: Option<u32>,
    pub total_withdrawn_usd: f64,
    pub path: Vec<PathYear>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct SequenceReport {
    pub mean_return_pct: f64,
    pub years: u32,
    pub scenarios: Vec<Scenario>,
    pub valid: bool,
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

// Simulate one ordering: grow by the return, then take the (inflating) withdrawal.
// Records the first year the balance goes non-positive (it is clamped to 0 once,
// matching the original), then keeps running.
fn run(seq: &[f64], start: f64, wd: f64, infl: f64, name: &str) -> Scenario {
    let mut bal = start;
    let mut real_wd = wd;
    let mut failed: Option<u32> = None;
    let mut total = 0.0;
    let mut path = Vec::with_capacity(seq.len());
    for (i, &ret) in seq.iter().enumerate() {
        bal *= 1.0 + ret;
        bal -= real_wd;
        total += real_wd;
        if bal <= 0.0 && failed.is_none() {
            failed = Some((i + 1) as u32);
            bal = 0.0;
        }
        path.push(PathYear { year: (i + 1) as u32, return_pct: round4(ret * 100.0), close_usd: round2(bal) });
        real_wd *= 1.0 + infl;
    }
    Scenario {
        name: name.to_string(),
        end_balance_usd: round2(bal),
        failed_at_year: failed,
        total_withdrawn_usd: round2(total),
        path,
    }
}

pub fn generate(i: &SequenceInput) -> SequenceReport {
    let rs: Vec<f64> = i.returns_pct.iter().filter(|x| x.is_finite()).map(|x| x / 100.0).collect();
    if rs.len() < 5 {
        return SequenceReport::default();
    }
    let mean = rs.iter().sum::<f64>() / rs.len() as f64;
    let mut sorted = rs.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mut reversed = rs.clone();
    reversed.reverse();
    let mut best = sorted.clone();
    best.reverse();

    let infl = i.inflation_pct / 100.0;
    let start = i.start_balance_usd;
    let wd = i.annual_withdrawal_usd;
    let scenarios = vec![
        run(&rs, start, wd, infl, "Forward (actual)"),
        run(&reversed, start, wd, infl, "Reversed"),
        run(&sorted, start, wd, infl, "Worst-years-first"),
        run(&best, start, wd, infl, "Best-years-first"),
    ];

    SequenceReport {
        mean_return_pct: round4(mean * 100.0),
        years: rs.len() as u32,
        scenarios,
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.5
    }

    // SPY 2000-2024 (approx), the "lost decade" sequence used as the default.
    fn spy() -> Vec<f64> {
        vec![-9.1, -11.9, -22.1, 28.7, 10.9, 4.9, 15.8, 5.5, -37.0, 26.5, 15.1, 2.1, 16.0, 32.4, 13.7, 1.4, 12.0, 21.8, -4.4, 31.5, 18.4, 28.7, -18.1, 26.3, 25.0]
    }

    fn base() -> SequenceInput {
        SequenceInput { returns_pct: spy(), start_balance_usd: 1_000_000.0, annual_withdrawal_usd: 40_000.0, inflation_pct: 3.0 }
    }

    // Pins cross-checked against the original JS compute() in Python.
    #[test]
    fn four_orderings_same_mean_different_ends() {
        let d = generate(&base());
        assert_eq!(d.scenarios.len(), 4);
        assert_eq!(d.years, 25);
        assert!(close(d.mean_return_pct, 9.364));
        let fwd = &d.scenarios[0];
        let worst = &d.scenarios[2];
        let best = &d.scenarios[3];
        assert!(close(fwd.end_balance_usd, 194_347.58));
        assert!(fwd.failed_at_year.is_none());
        // Worst-years-first fails partway through.
        assert_eq!(worst.failed_at_year, Some(9));
        // Best-years-first ends far higher than forward — same mean, opposite luck.
        assert!(best.end_balance_usd > fwd.end_balance_usd);
        assert!(close(best.end_balance_usd, 5_059_559.95));
    }

    #[test]
    fn path_length_matches_years() {
        let d = generate(&base());
        for s in &d.scenarios {
            assert_eq!(s.path.len(), 25);
        }
    }

    #[test]
    fn too_few_returns_invalid() {
        let d = generate(&SequenceInput { returns_pct: vec![5.0, 6.0, 7.0], ..base() });
        assert!(!d.valid);
    }
}
