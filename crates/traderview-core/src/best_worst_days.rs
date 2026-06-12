//! Best/worst-days concentration — the "miss the 10 best days" study
//! on real closes.
//!
//! Recomputes the compounded return with the N best days replaced by
//! cash (0%), the N worst replaced, and both. Equity returns
//! concentrate brutally: a handful of days carry the decade, and they
//! cluster next to the worst ones — the standard argument against
//! market-timing exits AND the honest counterweight (missing the
//! worst days helps even more).
//!
//! Pure compute over daily closes. Companion to `equity_curve_filter`
//! (a rule that tries to dodge the bad days systematically).

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ConcentrationReport {
    pub days: usize,
    pub n_excluded: usize,
    pub total_return_pct: f64,
    pub missing_best_pct: f64,
    pub missing_worst_pct: f64,
    pub missing_both_pct: f64,
    /// The N best single-day returns, %.
    pub best_days_pct: Vec<f64>,
    pub worst_days_pct: Vec<f64>,
}

pub fn compute(closes: &[f64], n: usize) -> Option<ConcentrationReport> {
    if closes.len() < 3
        || n == 0
        || closes.iter().any(|c| !c.is_finite() || *c <= 0.0)
    {
        return None;
    }
    let rets: Vec<f64> = closes.windows(2).map(|w| w[1] / w[0] - 1.0).collect();
    if n * 2 >= rets.len() {
        return None; // excluding that many days leaves nothing
    }
    let mut sorted = rets.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let worst: Vec<f64> = sorted[..n].to_vec();
    let best: Vec<f64> = sorted[sorted.len() - n..].iter().rev().copied().collect();
    let best_cut = sorted[sorted.len() - n]; // smallest of the best
    let worst_cut = sorted[n - 1]; // largest of the worst
    let grow = |skip_best: bool, skip_worst: bool| -> f64 {
        // Replace qualifying days with 0% (cash), counting ties only
        // up to n occurrences.
        let mut best_left = n;
        let mut worst_left = n;
        let mut g = 1.0_f64;
        for &r in &rets {
            let skip = (skip_best && r >= best_cut && best_left > 0 && {
                best_left -= 1;
                true
            }) || (skip_worst && r <= worst_cut && worst_left > 0 && {
                worst_left -= 1;
                true
            });
            if !skip {
                g *= 1.0 + r;
            }
        }
        (g - 1.0) * 100.0
    };
    Some(ConcentrationReport {
        days: rets.len(),
        n_excluded: n,
        total_return_pct: grow(false, false),
        missing_best_pct: grow(true, false),
        missing_worst_pct: grow(false, true),
        missing_both_pct: grow(true, true),
        best_days_pct: best.iter().map(|r| r * 100.0).collect(),
        worst_days_pct: worst.iter().map(|r| r * 100.0).collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_huge_day_carries_the_whole_return() {
        // Flat tape except one +50% day: total +50%, missing it = 0%.
        let mut closes = vec![100.0; 20];
        for c in closes.iter_mut().skip(10) {
            *c = 150.0;
        }
        let r = compute(&closes, 1).unwrap();
        assert!((r.total_return_pct - 50.0).abs() < 1e-9);
        assert!(r.missing_best_pct.abs() < 1e-9);
        assert!((r.best_days_pct[0] - 50.0).abs() < 1e-9);
    }

    #[test]
    fn missing_worst_beats_buy_and_hold() {
        // Up 1% daily with two −10% crashes mixed in.
        let mut closes = vec![100.0_f64];
        for i in 1..40 {
            let prev = closes[i - 1];
            closes.push(if i == 15 || i == 30 { prev * 0.9 } else { prev * 1.01 });
        }
        let r = compute(&closes, 2).unwrap();
        assert!(r.missing_worst_pct > r.total_return_pct);
        assert!(r.missing_best_pct < r.total_return_pct);
        // Skipping both: the exact compounding identity — total growth
        // divided by the skipped days' growth factors.
        let g_total = 1.0 + r.total_return_pct / 100.0;
        let g_best: f64 = r.best_days_pct.iter().map(|b| 1.0 + b / 100.0).product();
        let g_worst: f64 = r.worst_days_pct.iter().map(|w| 1.0 + w / 100.0).product();
        let want_both = (g_total / g_best / g_worst - 1.0) * 100.0;
        assert!((r.missing_both_pct - want_both).abs() < 1e-9);
    }

    #[test]
    fn worst_days_sorted_ascending_best_descending() {
        let closes = vec![100.0, 110.0, 99.0, 104.0, 93.6, 103.0];
        let r = compute(&closes, 2).unwrap();
        assert!(r.best_days_pct[0] >= r.best_days_pct[1]);
        assert!(r.worst_days_pct[0] <= r.worst_days_pct[1]);
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(&[100.0, 101.0], 1).is_none()); // too short
        assert!(compute(&[100.0; 10], 5).is_none()); // excludes everything
        assert!(compute(&[100.0, 0.0, 101.0], 1).is_none());
        assert!(compute(&[100.0; 10], 0).is_none());
    }
}
