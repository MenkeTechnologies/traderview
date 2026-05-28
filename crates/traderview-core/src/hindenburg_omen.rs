//! Hindenburg Omen — Jim Miekka's market-crash early-warning indicator.
//!
//! All five conditions must be true on the same day to "trigger" the omen:
//!   1. New 52-week highs ≥ `min_extreme_pct` of issues AND new lows ≥
//!      `min_extreme_pct` (both sides extreme — split market).
//!   2. The smaller of (new highs, new lows) is ≥ `min_smaller_count`
//!      (typically 79 absolute issues; calibrated to NYSE).
//!   3. NYSE Composite is above its `index_ma_period`-day SMA.
//!   4. McClellan Oscillator is negative (breadth deteriorating).
//!   5. New highs ≤ 2 × new lows (no clear leadership).
//!
//! Confirmed signal: ≥ 2 omens within 36 trading days. The full
//! confirmation logic is bundled in the report.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DailyBar {
    pub index_close: f64,
    pub issues_traded: i64,
    pub new_52w_highs: i64,
    pub new_52w_lows: i64,
    /// Pre-computed McClellan oscillator reading.
    pub mcclellan: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub min_extreme_pct: f64,        // typically 0.028 (2.8%)
    pub min_smaller_count: i64,      // typically 79 (NYSE classic)
    pub index_ma_period: usize,      // typically 50 (NYSE Comp)
    pub max_high_to_low_ratio: f64,  // typically 2.0
    pub confirmation_window: usize,  // typically 36 trading days
    pub min_confirmation_count: usize, // typically 2
}

impl Default for Config {
    fn default() -> Self {
        Self {
            min_extreme_pct: 0.028,
            min_smaller_count: 79,
            index_ma_period: 50,
            max_high_to_low_ratio: 2.0,
            confirmation_window: 36,
            min_confirmation_count: 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Report {
    /// Per-bar boolean: did the omen trigger on this day?
    pub triggers: Vec<bool>,
    /// Per-bar boolean: did a confirmed omen complete on this day?
    pub confirmations: Vec<bool>,
    pub trigger_indices: Vec<usize>,
    pub confirmation_indices: Vec<usize>,
}

pub fn analyze(bars: &[DailyBar], cfg: &Config) -> Report {
    let n = bars.len();
    let mut report = Report {
        triggers: vec![false; n],
        confirmations: vec![false; n],
        trigger_indices: Vec::new(),
        confirmation_indices: Vec::new(),
    };
    if cfg.index_ma_period == 0
        || cfg.confirmation_window == 0
        || cfg.min_confirmation_count == 0
        || !cfg.min_extreme_pct.is_finite()
        || cfg.min_extreme_pct <= 0.0
        || cfg.min_extreme_pct >= 1.0
        || cfg.min_smaller_count < 0
        || !cfg.max_high_to_low_ratio.is_finite()
        || cfg.max_high_to_low_ratio <= 0.0
        || n < cfg.index_ma_period
    {
        return report;
    }
    // Pre-compute rolling SMA of index_close for condition 3.
    let p = cfg.index_ma_period;
    let mut sma = vec![None::<f64>; n];
    let mut sum = 0.0;
    for (i, b) in bars.iter().enumerate() {
        if !b.index_close.is_finite() {
            // Reset accumulator to avoid NaN propagation.
            sum = 0.0;
            // Re-warm by walking back: easier to just continue and
            // accept None until window valid again.
            continue;
        }
        sum += b.index_close;
        if i + 1 > p {
            sum -= bars[i - p].index_close;
        }
        if i + 1 >= p {
            sma[i] = Some(sum / p as f64);
        }
    }
    // Pass over bars: evaluate each condition.
    for (i, b) in bars.iter().enumerate() {
        let issues = b.issues_traded as f64;
        if issues <= 0.0 { continue; }
        let high_pct = b.new_52w_highs as f64 / issues;
        let low_pct = b.new_52w_lows as f64 / issues;
        let cond1 = high_pct >= cfg.min_extreme_pct && low_pct >= cfg.min_extreme_pct;
        let smaller = b.new_52w_highs.min(b.new_52w_lows);
        let cond2 = smaller >= cfg.min_smaller_count;
        let cond3 = matches!(sma[i], Some(m) if b.index_close > m);
        let cond4 = b.mcclellan < 0.0;
        let cond5 = b.new_52w_highs as f64 <= cfg.max_high_to_low_ratio * b.new_52w_lows as f64;
        if cond1 && cond2 && cond3 && cond4 && cond5 {
            report.triggers[i] = true;
            report.trigger_indices.push(i);
        }
    }
    // Confirmation pass: ≥ `min_confirmation_count` triggers within
    // `confirmation_window` days mark a confirmed omen on the day the
    // count is reached.
    let win = cfg.confirmation_window;
    let need = cfg.min_confirmation_count;
    for i in 0..n {
        if !report.triggers[i] { continue; }
        // Count triggers in [i - win + 1, i] inclusive.
        let lo = i.saturating_sub(win - 1);
        let cnt = report.triggers[lo..=i].iter().filter(|t| **t).count();
        if cnt >= need {
            report.confirmations[i] = true;
            report.confirmation_indices.push(i);
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn b(index: f64, issues: i64, nh: i64, nl: i64, mc: f64) -> DailyBar {
        DailyBar {
            index_close: index,
            issues_traded: issues,
            new_52w_highs: nh,
            new_52w_lows: nl,
            mcclellan: mc,
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[], &Config::default());
        assert!(r.trigger_indices.is_empty());
    }

    #[test]
    fn invalid_config_returns_default() {
        let bars = vec![b(100.0, 3000, 100, 100, -5.0); 60];
        for cfg in [
            Config { min_extreme_pct: 0.0, ..Default::default() },
            Config { min_extreme_pct: 1.5, ..Default::default() },
            Config { max_high_to_low_ratio: 0.0, ..Default::default() },
            Config { index_ma_period: 0, ..Default::default() },
            Config { confirmation_window: 0, ..Default::default() },
            Config { min_confirmation_count: 0, ..Default::default() },
        ] {
            let r = analyze(&bars, &cfg);
            assert!(r.trigger_indices.is_empty());
        }
    }

    #[test]
    fn too_few_bars_for_sma_returns_default() {
        let bars = vec![b(100.0, 3000, 100, 100, -5.0); 10];
        let r = analyze(&bars, &Config::default());
        assert!(r.trigger_indices.is_empty());
    }

    #[test]
    fn all_conditions_satisfied_yields_trigger() {
        // 60 days flat at index=100, then a day with extreme breadth.
        let mut bars = vec![b(100.0, 3000, 5, 5, 5.0); 60];
        bars.push(b(110.0, 3000, 100, 95, -10.0));    // satisfies all 5
        let r = analyze(&bars, &Config::default());
        assert!(r.triggers[60], "day 60 should trigger");
    }

    #[test]
    fn confirmation_requires_second_trigger_within_window() {
        let mut bars = vec![b(100.0, 3000, 5, 5, 5.0); 60];
        bars.push(b(110.0, 3000, 100, 95, -10.0));    // trigger 1
        for _ in 0..10 { bars.push(b(105.0, 3000, 5, 5, -2.0)); }    // no trigger
        bars.push(b(110.0, 3000, 100, 95, -10.0));    // trigger 2 within window
        let r = analyze(&bars, &Config::default());
        assert!(r.confirmation_indices.contains(&71), "should confirm on second trigger");
    }

    #[test]
    fn single_trigger_does_not_confirm() {
        let mut bars = vec![b(100.0, 3000, 5, 5, 5.0); 60];
        bars.push(b(110.0, 3000, 100, 95, -10.0));
        let r = analyze(&bars, &Config::default());
        assert!(r.trigger_indices.contains(&60));
        assert!(r.confirmation_indices.is_empty(), "single trigger should not confirm");
    }

    #[test]
    fn positive_mcclellan_blocks_trigger() {
        let mut bars = vec![b(100.0, 3000, 5, 5, 5.0); 60];
        bars.push(b(110.0, 3000, 100, 95, 5.0));    // positive McClellan
        let r = analyze(&bars, &Config::default());
        assert!(r.trigger_indices.is_empty());
    }

    #[test]
    fn imbalanced_highs_dominate_blocks_trigger() {
        // 200 highs vs 50 lows: 200 > 2 × 50 → cond5 fails.
        let mut bars = vec![b(100.0, 3000, 5, 5, 5.0); 60];
        bars.push(b(110.0, 3000, 200, 50, -10.0));
        let r = analyze(&bars, &Config::default());
        assert!(r.trigger_indices.is_empty());
    }
}
