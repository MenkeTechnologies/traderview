//! Emotion-tag P&L correlation.
//!
//! Many traders journal each trade with a mood tag ("calm", "FOMO",
//! "rushed", "confident", "revenge", "tilt"). This module groups
//! per-tag P&L stats so the trader can spot which moods are profitable
//! and which leak money.
//!
//! Pure compute. Trader supplies the tag taxonomy — engine just buckets.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaggedTrade {
    /// One or more mood tags from the trader's journal.
    pub tags: Vec<String>,
    pub pnl: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagStats {
    pub tag: String,
    pub trade_count: usize,
    pub total_pnl: f64,
    pub avg_pnl: f64,
    pub win_rate: f64,
    pub best: f64,
    pub worst: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmotionReport {
    pub by_tag: Vec<TagStats>,
    /// Tags ranked by total P&L — most profitable first.
    pub most_profitable: Vec<String>,
    /// Tags with negative avg P&L AND at least 5 trades.
    pub problem_tags: Vec<String>,
}

pub fn analyze(trades: &[TaggedTrade]) -> EmotionReport {
    let mut by_tag: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for t in trades {
        for tag in &t.tags {
            by_tag.entry(tag.clone()).or_default().push(t.pnl);
        }
    }
    let mut report = EmotionReport::default();
    for (tag, pnls) in by_tag {
        let n = pnls.len();
        let total: f64 = pnls.iter().sum();
        let avg = total / n as f64;
        let wins = pnls.iter().filter(|p| **p > 0.0).count();
        let best = pnls.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let worst = pnls.iter().cloned().fold(f64::INFINITY, f64::min);
        report.by_tag.push(TagStats {
            tag,
            trade_count: n,
            total_pnl: total,
            avg_pnl: avg,
            win_rate: wins as f64 / n as f64,
            best,
            worst,
        });
    }
    // Most profitable first.
    let mut sorted_by_total = report.by_tag.clone();
    sorted_by_total.sort_by(|a, b| {
        b.total_pnl
            .partial_cmp(&a.total_pnl)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    report.most_profitable = sorted_by_total.iter().map(|t| t.tag.clone()).collect();
    for t in &report.by_tag {
        if t.trade_count >= 5 && t.avg_pnl < 0.0 {
            report.problem_tags.push(t.tag.clone());
        }
    }
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(tags: &[&str], pnl: f64) -> TaggedTrade {
        TaggedTrade {
            tags: tags.iter().map(|s| s.to_string()).collect(),
            pnl,
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[]);
        assert!(r.by_tag.is_empty());
    }

    #[test]
    fn single_tag_single_trade() {
        let r = analyze(&[t(&["calm"], 100.0)]);
        assert_eq!(r.by_tag[0].tag, "calm");
        assert_eq!(r.by_tag[0].total_pnl, 100.0);
        assert_eq!(r.by_tag[0].avg_pnl, 100.0);
        assert_eq!(r.by_tag[0].win_rate, 1.0);
    }

    #[test]
    fn problem_tag_flagged_when_5plus_trades_negative_avg() {
        let trades: Vec<_> = (0..5).map(|_| t(&["tilt"], -100.0)).collect();
        let r = analyze(&trades);
        assert!(r.problem_tags.contains(&"tilt".to_string()));
    }

    #[test]
    fn problem_tag_not_flagged_under_five_trades() {
        let trades: Vec<_> = (0..3).map(|_| t(&["tilt"], -100.0)).collect();
        let r = analyze(&trades);
        assert!(!r.problem_tags.contains(&"tilt".to_string()));
    }

    #[test]
    fn most_profitable_sorted_descending() {
        let trades = vec![
            t(&["calm"], 500.0),
            t(&["FOMO"], -100.0),
            t(&["confident"], 200.0),
        ];
        let r = analyze(&trades);
        assert_eq!(r.most_profitable[0], "calm");
        assert_eq!(r.most_profitable[1], "confident");
        assert_eq!(r.most_profitable[2], "FOMO");
    }

    #[test]
    fn trade_with_multiple_tags_counted_in_each() {
        let trades = vec![t(&["calm", "confident"], 100.0)];
        let r = analyze(&trades);
        // Both "calm" and "confident" should have a single trade.
        assert_eq!(
            r.by_tag
                .iter()
                .find(|t| t.tag == "calm")
                .unwrap()
                .trade_count,
            1
        );
        assert_eq!(
            r.by_tag
                .iter()
                .find(|t| t.tag == "confident")
                .unwrap()
                .trade_count,
            1
        );
    }

    #[test]
    fn best_and_worst_extracted_correctly() {
        let trades = vec![
            t(&["FOMO"], -50.0),
            t(&["FOMO"], 200.0),
            t(&["FOMO"], -100.0),
        ];
        let r = analyze(&trades);
        let fomo = r.by_tag.iter().find(|t| t.tag == "FOMO").unwrap();
        assert_eq!(fomo.best, 200.0);
        assert_eq!(fomo.worst, -100.0);
    }

    #[test]
    fn win_rate_computed_per_tag() {
        let trades = vec![
            t(&["tag"], 100.0),
            t(&["tag"], -50.0),
            t(&["tag"], 100.0),
            t(&["tag"], 100.0),
        ];
        let r = analyze(&trades);
        let tag = &r.by_tag[0];
        assert!((tag.win_rate - 0.75).abs() < 1e-9);
    }
}
