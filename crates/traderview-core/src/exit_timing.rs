//! Exit-timing analysis ("left-on-table" report).
//!
//! Pairs each closed trade against its post-trade Maximum Favorable
//! Excursion (MFE) within the holding window to expose how much
//! additional profit was on offer after exit — i.e., the cost of
//! exiting too early. Critical for trend traders who systematically
//! cut winners short.
//!
//! Inputs per trade: realized R (or $ profit), MFE in the same unit,
//! and trade duration. Outputs per-trade left-on-table + aggregate
//! capture ratio (realized / MFE).
//!
//! Pure compute. Distinct from `excursion.rs` which is a generic
//! MFE/MAE accumulator — this module assumes you already have MFE
//! and asks the exit-quality question.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeExit {
    pub trade_id: String,
    pub realized: f64,
    /// MFE during the trade (always >= realized for a winner).
    pub mfe: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitRow {
    pub trade_id: String,
    pub realized: f64,
    pub mfe: f64,
    /// MFE - realized — the dollar/R amount left on the table.
    /// Zero or negative for losing trades / perfect exits.
    pub left_on_table: f64,
    /// realized / mfe (0..=1 for winners). None when mfe ≤ 0.
    pub capture_ratio: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ExitTimingReport {
    pub rows: Vec<ExitRow>,
    pub total_realized: f64,
    pub total_mfe: f64,
    /// Aggregate capture: sum(realized) / sum(mfe).
    pub aggregate_capture_ratio: f64,
    pub total_left_on_table: f64,
    /// How many trades exited inside the top 80% of MFE (i.e. capture ≥ 0.8).
    pub strong_exit_count: usize,
    /// Trades that exited below 30% of MFE (cut winners way short).
    pub weak_exit_count: usize,
}

pub fn evaluate(trades: &[TradeExit]) -> ExitTimingReport {
    let mut report = ExitTimingReport::default();
    if trades.is_empty() { return report; }
    for t in trades {
        // When MFE was zero or negative, the trade never showed a profit
        // — nothing was left "on the table" to leave. Clamp LOT to 0 in
        // that case rather than (mfe - realized) which would inflate when
        // realized is negative.
        let lot = if t.mfe <= 0.0 { 0.0 } else { (t.mfe - t.realized).max(0.0) };
        let capture = if t.mfe > 0.0 { Some(t.realized / t.mfe) } else { None };
        if let Some(c) = capture {
            if c >= 0.8 { report.strong_exit_count += 1; }
            else if c < 0.3 { report.weak_exit_count += 1; }
        }
        report.rows.push(ExitRow {
            trade_id: t.trade_id.clone(),
            realized: t.realized,
            mfe: t.mfe,
            left_on_table: lot,
            capture_ratio: capture,
        });
        report.total_realized += t.realized;
        report.total_mfe += t.mfe;
        report.total_left_on_table += lot;
    }
    report.aggregate_capture_ratio = if report.total_mfe > 0.0 {
        report.total_realized / report.total_mfe
    } else {
        0.0
    };
    // Sort by left-on-table descending — biggest "early exits" first.
    report.rows.sort_by(|a, b| b.left_on_table.partial_cmp(&a.left_on_table)
        .unwrap_or(std::cmp::Ordering::Equal));
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(id: &str, realized: f64, mfe: f64) -> TradeExit {
        TradeExit { trade_id: id.into(), realized, mfe }
    }

    #[test]
    fn empty_returns_default() {
        let r = evaluate(&[]);
        assert!(r.rows.is_empty());
        assert_eq!(r.aggregate_capture_ratio, 0.0);
    }

    #[test]
    fn perfect_exit_capture_ratio_is_one() {
        let r = evaluate(&[t("X", 100.0, 100.0)]);
        assert_eq!(r.rows[0].capture_ratio, Some(1.0));
        assert_eq!(r.rows[0].left_on_table, 0.0);
        assert_eq!(r.strong_exit_count, 1);
    }

    #[test]
    fn early_exit_records_left_on_table_and_flags_weak() {
        // Exited at 20, MFE was 100 → capture 0.20 → weak.
        let r = evaluate(&[t("X", 20.0, 100.0)]);
        assert_eq!(r.rows[0].left_on_table, 80.0);
        assert!((r.rows[0].capture_ratio.unwrap() - 0.20).abs() < 1e-9);
        assert_eq!(r.weak_exit_count, 1);
        assert_eq!(r.strong_exit_count, 0);
    }

    #[test]
    fn losing_trade_with_zero_mfe_has_none_capture() {
        let r = evaluate(&[t("LOSER", -50.0, 0.0)]);
        assert!(r.rows[0].capture_ratio.is_none());
        assert_eq!(r.rows[0].left_on_table, 0.0,
            "no left-on-table when MFE was zero (never showed a profit)");
    }

    #[test]
    fn capture_above_80pct_flagged_strong() {
        let r = evaluate(&[t("X", 85.0, 100.0)]);
        assert_eq!(r.strong_exit_count, 1);
        assert_eq!(r.weak_exit_count, 0);
    }

    #[test]
    fn aggregate_capture_is_sum_realized_over_sum_mfe() {
        let r = evaluate(&[
            t("A", 80.0,  100.0),
            t("B", 40.0,  100.0),
            t("C", 60.0,  100.0),
        ]);
        // Total realized = 180, total mfe = 300, ratio = 0.6.
        assert_eq!(r.total_realized, 180.0);
        assert_eq!(r.total_mfe, 300.0);
        assert!((r.aggregate_capture_ratio - 0.6).abs() < 1e-9);
    }

    #[test]
    fn rows_sorted_by_left_on_table_descending() {
        let r = evaluate(&[
            t("SMALL", 90.0, 100.0),    // 10 LOT
            t("BIG",   10.0, 100.0),    // 90 LOT
            t("MID",   50.0, 100.0),    // 50 LOT
        ]);
        assert_eq!(r.rows[0].trade_id, "BIG");
        assert_eq!(r.rows[1].trade_id, "MID");
        assert_eq!(r.rows[2].trade_id, "SMALL");
    }

    #[test]
    fn winner_turned_loser_left_on_table_reflects_mfe_minus_realized() {
        // Trade went +20 (MFE) then reversed to close at -50.
        // LOT = MFE - realized = 20 - (-50) = 70. The trader DID see +20 of
        // profit and rode it all the way to -50 — that's $70 of round-trip
        // erosion from MFE to exit.
        let r = evaluate(&[t("REVERSAL", -50.0, 20.0)]);
        assert_eq!(r.rows[0].left_on_table, 70.0);
    }

    #[test]
    fn over_exit_realized_greater_than_mfe_zeros_left_on_table() {
        // Shouldn't happen in clean data — realized > MFE means the exit
        // print was above the MFE record. Clamp to 0 LOT.
        let r = evaluate(&[t("WEIRD", 120.0, 100.0)]);
        assert_eq!(r.rows[0].left_on_table, 0.0);
        assert!(r.rows[0].capture_ratio.unwrap() > 1.0);
    }
}
