//! R-multiple distribution analyzer.
//!
//! R = net_pnl / risk_amount per trade. Only trades with both fields set are
//! included; the call surfaces a `skipped` count so the user knows how many
//! trades they can unlock by setting risk_amount on entry.
//!
//! Histogram: 21 bins from -5R to +5R in 0.5R steps; tails (<-5R, >+5R)
//! clamp to the edge bins.
//!
//! SQN (Van Tharp) = sqrt(N) × mean(R) / stdev(R). Conventional bands:
//!   < 1.6   poor
//!   1.6-1.9 below average
//!   2.0-2.4 average
//!   2.5-2.9 good
//!   3.0-5.0 excellent
//!   > 5.0   suspiciously curve-fit / unsustainable

use rust_decimal::Decimal;
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct Bin {
    pub low: f64,
    pub high: f64,
    pub label: String, // e.g. "-1.0..-0.5"
    pub count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct Stats {
    pub samples: usize,
    pub mean_r: f64, // expectancy
    pub stdev_r: f64,
    pub sqn: f64, // sqrt(N) * mean / stdev
    pub sqn_grade: &'static str,
    pub winners: usize,
    pub losers: usize,
    pub breakevens: usize,
    pub win_rate: f64,
    pub avg_winner_r: f64,
    pub avg_loser_r: f64,
    pub max_winner_r: f64,
    pub max_loser_r: f64,
    pub payoff_ratio: f64,  // |avg_win| / |avg_loss|
    pub profit_factor: f64, // sum(wins) / |sum(losses)|
}

#[derive(Debug, Clone, Serialize)]
pub struct TagBreakdown {
    pub tag_name: String,
    pub tag_color: String,
    pub samples: usize,
    pub mean_r: f64,
    pub sqn: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RDistReport {
    pub account_id: Uuid,
    pub bins: Vec<Bin>,
    pub stats: Stats,
    pub by_tag: Vec<TagBreakdown>,
    pub skipped_no_risk: usize,
    pub computed_at: chrono::DateTime<chrono::Utc>,
}

pub async fn report(
    pool: &PgPool,
    _user_id: Uuid,
    account_id: Uuid,
) -> anyhow::Result<RDistReport> {
    // Pull every closed trade with both net_pnl and risk_amount set.
    let rows: Vec<(Uuid, Decimal, Decimal)> = sqlx::query_as(
        "SELECT id, net_pnl, risk_amount FROM trades
          WHERE account_id = $1 AND status = 'closed'
            AND net_pnl IS NOT NULL AND risk_amount IS NOT NULL
            AND risk_amount > 0",
    )
    .bind(account_id)
    .fetch_all(pool)
    .await?;

    // Skip count for visibility.
    let skipped: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM trades
          WHERE account_id = $1 AND status = 'closed'
            AND net_pnl IS NOT NULL
            AND (risk_amount IS NULL OR risk_amount <= 0)",
    )
    .bind(account_id)
    .fetch_one(pool)
    .await
    .unwrap_or(0);

    let trade_ids: Vec<Uuid> = rows.iter().map(|(id, _, _)| *id).collect();
    let r_values: Vec<f64> = rows
        .iter()
        .map(|(_, pnl, risk)| dec(*pnl) / dec(*risk))
        .collect();

    // Tag-keyed R-vector. Single query for all trade↔tag pairs.
    let mut by_tag_r: HashMap<(Uuid, String, String), Vec<f64>> = HashMap::new();
    if !trade_ids.is_empty() {
        let tag_rows: Vec<(Uuid, Uuid, String, String)> = sqlx::query_as(
            "SELECT tt.trade_id, t.id, t.name, t.color
               FROM trade_tags tt
               JOIN tags t ON t.id = tt.tag_id
              WHERE tt.trade_id = ANY($1)",
        )
        .bind(&trade_ids)
        .fetch_all(pool)
        .await
        .unwrap_or_default();
        // Map trade_id -> R for fast lookup.
        let r_by_trade: HashMap<Uuid, f64> = trade_ids
            .iter()
            .copied()
            .zip(r_values.iter().copied())
            .collect();
        for (trade_id, tag_id, tag_name, tag_color) in tag_rows {
            if let Some(r) = r_by_trade.get(&trade_id) {
                by_tag_r
                    .entry((tag_id, tag_name, tag_color))
                    .or_default()
                    .push(*r);
            }
        }
    }

    // Build histogram: bins from -5R to +5R in 0.5R steps.
    let mut bins: Vec<Bin> = Vec::new();
    let mut lo = -5.0f64;
    while lo < 5.0 - 1e-9 {
        let hi = lo + 0.5;
        bins.push(Bin {
            low: lo,
            high: hi,
            label: format!("{:+.1}..{:+.1}", lo, hi),
            count: 0,
        });
        lo = hi;
    }
    for r in &r_values {
        let clamped = r.clamp(-5.0, 4.99);
        let idx = ((clamped - (-5.0)) / 0.5) as usize;
        let idx = idx.min(bins.len() - 1);
        bins[idx].count += 1;
    }

    let stats = compute_stats(&r_values);
    let by_tag: Vec<TagBreakdown> = by_tag_r
        .into_iter()
        .map(|((_id, name, color), rs)| {
            let s = compute_stats(&rs);
            TagBreakdown {
                tag_name: name,
                tag_color: color,
                samples: s.samples,
                mean_r: s.mean_r,
                sqn: s.sqn,
            }
        })
        .collect();

    let mut by_tag = by_tag;
    by_tag.sort_by(|a, b| {
        b.sqn
            .partial_cmp(&a.sqn)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    Ok(RDistReport {
        account_id,
        bins,
        stats,
        by_tag,
        skipped_no_risk: skipped as usize,
        computed_at: chrono::Utc::now(),
    })
}

fn compute_stats(rs: &[f64]) -> Stats {
    let n = rs.len();
    if n == 0 {
        return Stats {
            samples: 0,
            mean_r: 0.0,
            stdev_r: 0.0,
            sqn: 0.0,
            sqn_grade: "—",
            winners: 0,
            losers: 0,
            breakevens: 0,
            win_rate: 0.0,
            avg_winner_r: 0.0,
            avg_loser_r: 0.0,
            max_winner_r: 0.0,
            max_loser_r: 0.0,
            payoff_ratio: 0.0,
            profit_factor: 0.0,
        };
    }
    let nf = n as f64;
    let mean = rs.iter().sum::<f64>() / nf;
    let var = rs.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / nf;
    let stdev = var.sqrt();
    let sqn = if stdev > 1e-9 {
        nf.sqrt() * mean / stdev
    } else {
        0.0
    };
    let grade = if sqn < 1.6 {
        "poor"
    } else if sqn < 2.0 {
        "below avg"
    } else if sqn < 2.5 {
        "average"
    } else if sqn < 3.0 {
        "good"
    } else if sqn <= 5.0 {
        "excellent"
    } else {
        "suspect"
    };
    let winners_v: Vec<f64> = rs.iter().filter(|x| **x > 0.0).copied().collect();
    let losers_v: Vec<f64> = rs.iter().filter(|x| **x < 0.0).copied().collect();
    let breakevens = n - winners_v.len() - losers_v.len();
    let avg_w = if !winners_v.is_empty() {
        winners_v.iter().sum::<f64>() / winners_v.len() as f64
    } else {
        0.0
    };
    let avg_l = if !losers_v.is_empty() {
        losers_v.iter().sum::<f64>() / losers_v.len() as f64
    } else {
        0.0
    };
    let max_w = winners_v.iter().cloned().fold(0.0_f64, f64::max);
    let max_l = losers_v.iter().cloned().fold(0.0_f64, f64::min);
    let sum_w: f64 = winners_v.iter().sum();
    let sum_l: f64 = losers_v.iter().sum::<f64>().abs();
    Stats {
        samples: n,
        mean_r: mean,
        stdev_r: stdev,
        sqn,
        sqn_grade: grade,
        winners: winners_v.len(),
        losers: losers_v.len(),
        breakevens,
        win_rate: winners_v.len() as f64 / nf,
        avg_winner_r: avg_w,
        avg_loser_r: avg_l,
        max_winner_r: max_w,
        max_loser_r: max_l,
        payoff_ratio: if avg_l.abs() > 1e-9 {
            avg_w / avg_l.abs()
        } else {
            0.0
        },
        profit_factor: if sum_l > 1e-9 { sum_w / sum_l } else { 0.0 },
    }
}

fn dec(d: Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===========================================================================
    // compute_stats — empty input
    // ===========================================================================

    #[test]
    fn empty_input_yields_zeroed_stats_with_em_dash_grade() {
        let s = compute_stats(&[]);
        assert_eq!(s.samples, 0);
        assert_eq!(s.mean_r, 0.0);
        assert_eq!(s.stdev_r, 0.0);
        assert_eq!(s.sqn, 0.0);
        // No samples → no grade.
        assert_eq!(s.sqn_grade, "—");
        assert_eq!(s.winners, 0);
        assert_eq!(s.losers, 0);
        assert_eq!(s.breakevens, 0);
        assert_eq!(s.win_rate, 0.0);
        assert_eq!(s.payoff_ratio, 0.0);
        assert_eq!(s.profit_factor, 0.0);
    }

    // ===========================================================================
    // Winner / loser / breakeven classification
    // ===========================================================================

    #[test]
    fn breakevens_are_exactly_zero_r() {
        // 0.0 is neither winner (>0) nor loser (<0).
        let s = compute_stats(&[1.0, -1.0, 0.0, 0.0]);
        assert_eq!(s.winners, 1);
        assert_eq!(s.losers, 1);
        assert_eq!(s.breakevens, 2);
        assert_eq!(s.samples, 4);
    }

    #[test]
    fn win_rate_excludes_breakevens_from_numerator_only() {
        // 2 winners, 1 loser, 1 BE. win_rate = 2/4 = 0.5
        let s = compute_stats(&[1.0, 1.0, -1.0, 0.0]);
        assert_eq!(s.win_rate, 0.5);
    }

    #[test]
    fn mean_r_is_simple_average() {
        // (2 + -1 + 3) / 3 = 4/3
        let s = compute_stats(&[2.0, -1.0, 3.0]);
        assert!((s.mean_r - 4.0 / 3.0).abs() < 1e-9);
    }

    // ===========================================================================
    // SQN computation & grade bands
    // ===========================================================================

    #[test]
    fn sqn_zero_when_stdev_collapses_to_constant_series() {
        // All identical → stdev=0 → sqn=0 guard prevents divide-by-zero.
        let s = compute_stats(&[1.0, 1.0, 1.0, 1.0]);
        assert_eq!(s.stdev_r, 0.0);
        assert_eq!(s.sqn, 0.0);
        assert_eq!(s.sqn_grade, "poor");
    }

    #[test]
    fn sqn_grade_poor_band_below_1_6() {
        // Construct trades with mean 1.0 stdev 1.0 sqrt(2)≈1.41 → sqn≈1.41
        let s = compute_stats(&[2.0, 0.0]);
        assert!(s.sqn < 1.6);
        assert_eq!(s.sqn_grade, "poor");
    }

    #[test]
    fn sqn_grade_suspect_above_5() {
        // Strongly biased mean + tiny stdev × big n → SQN > 5.
        // mean=10, samples=10, stdev=1 → sqn = sqrt(10)*10/1 ≈ 31.6
        let r: Vec<f64> = (0..10).map(|i| 10.0 + (i % 2) as f64 - 0.5).collect();
        let s = compute_stats(&r);
        assert!(s.sqn > 5.0);
        assert_eq!(s.sqn_grade, "suspect");
    }

    #[test]
    fn sqn_grade_excellent_when_3_to_5() {
        // mean=1, stdev≈0.5, n=9 → sqn=sqrt(9)*1/0.5=6 (too high)
        // Use mean=1, stdev=1, n=16 → sqn=sqrt(16)*1/1=4 ∈ [3,5] → "excellent".
        let mut r: Vec<f64> = vec![0.0_f64; 8];
        r.extend(vec![2.0_f64; 8]);
        // mean = 1, var = 1, stdev = 1, n = 16, sqn = 4.0
        let s = compute_stats(&r);
        assert!((s.mean_r - 1.0).abs() < 1e-9);
        assert!((s.stdev_r - 1.0).abs() < 1e-9);
        assert!((s.sqn - 4.0).abs() < 1e-9);
        assert_eq!(s.sqn_grade, "excellent");
    }

    // ===========================================================================
    // Payoff & profit factor
    // ===========================================================================

    #[test]
    fn payoff_ratio_is_avg_win_over_abs_avg_loss() {
        // wins: 2, 4 → avg 3; losses: -1, -3 → avg -2 → |2|. payoff = 3/2 = 1.5
        let s = compute_stats(&[2.0, 4.0, -1.0, -3.0]);
        assert!((s.payoff_ratio - 1.5).abs() < 1e-9);
    }

    #[test]
    fn payoff_ratio_zero_when_no_losers() {
        // No losers: avg_loser_r stays 0.0, payoff guard returns 0.0.
        let s = compute_stats(&[1.0, 2.0, 3.0]);
        assert_eq!(s.payoff_ratio, 0.0);
    }

    #[test]
    fn profit_factor_is_sum_wins_over_abs_sum_losses() {
        // wins sum = 6; losses sum = -4, |abs|=4. PF = 1.5.
        let s = compute_stats(&[2.0, 4.0, -1.0, -3.0]);
        assert!((s.profit_factor - 1.5).abs() < 1e-9);
    }

    #[test]
    fn profit_factor_zero_when_no_losses() {
        // sum_l = 0 → guard returns 0.
        let s = compute_stats(&[1.0, 2.0, 3.0]);
        assert_eq!(s.profit_factor, 0.0);
    }

    // ===========================================================================
    // Max winner / max loser
    // ===========================================================================

    #[test]
    fn max_winner_and_max_loser_extremes() {
        let s = compute_stats(&[1.5, 4.0, -2.5, -0.3, 2.0]);
        assert_eq!(s.max_winner_r, 4.0);
        assert_eq!(s.max_loser_r, -2.5);
    }

    #[test]
    fn max_winner_zero_when_all_losses() {
        // No winners → fold start at 0.0 stays 0.
        let s = compute_stats(&[-1.0, -2.0, -3.0]);
        assert_eq!(s.max_winner_r, 0.0);
    }

    #[test]
    fn max_loser_zero_when_all_winners() {
        let s = compute_stats(&[1.0, 2.0, 3.0]);
        assert_eq!(s.max_loser_r, 0.0);
    }

    // ===========================================================================
    // dec helper
    // ===========================================================================

    #[test]
    fn dec_handles_zero_and_negative() {
        assert_eq!(dec(Decimal::ZERO), 0.0);
        assert_eq!(dec(Decimal::from(-7)), -7.0);
        assert!((dec(Decimal::new(125, 2)) - 1.25).abs() < 1e-9);
    }
}
