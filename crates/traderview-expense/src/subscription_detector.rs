//! Recurring-subscription detector.
//!
//! Walks a transaction history, groups by merchant, and flags merchants
//! that show a regular cadence (monthly / quarterly / yearly) with stable
//! amounts (within `amount_tolerance_pct` of the median). Used to surface
//! the SaaS / subscription tail that quietly compounds — most users
//! seriously underestimate it.
//!
//! Pure compute — no I/O. Input is the same `ParsedTransaction` shape used
//! by the broker importers.

use crate::ParsedTransaction;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cadence {
    Weekly,
    Monthly,
    Quarterly,
    SemiAnnual,
    Annual,
    Irregular,
}

impl Cadence {
    pub fn from_avg_days(days: f64) -> Self {
        // Tolerance windows around each cadence midpoint.
        match days {
            d if (5.0..=10.0).contains(&d) => Cadence::Weekly,
            d if (25.0..=35.0).contains(&d) => Cadence::Monthly,
            d if (80.0..=100.0).contains(&d) => Cadence::Quarterly,
            d if (170.0..=190.0).contains(&d) => Cadence::SemiAnnual,
            d if (350.0..=380.0).contains(&d) => Cadence::Annual,
            _ => Cadence::Irregular,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Cadence::Weekly => "weekly",
            Cadence::Monthly => "monthly",
            Cadence::Quarterly => "quarterly",
            Cadence::SemiAnnual => "semi-annual",
            Cadence::Annual => "annual",
            Cadence::Irregular => "irregular",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub merchant: String,
    pub cadence: Cadence,
    pub avg_gap_days: f64,
    pub samples: usize,
    pub median_amount: Decimal,
    pub amount_variation_pct: f64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    /// Projected annual cost = median_amount × periods_per_year.
    pub projected_annual_cost: Decimal,
}

#[derive(Debug, Clone, Copy)]
pub struct DetectOptions {
    /// Minimum repeats to qualify (default 3 — two gaps).
    pub min_samples: usize,
    /// Maximum coefficient-of-variation in gap days (default 0.20).
    pub max_gap_cv: f64,
    /// Maximum (max-min)/median amount allowed (default 0.05 = 5%).
    pub max_amount_variation: f64,
}

impl Default for DetectOptions {
    fn default() -> Self {
        Self {
            min_samples: 3,
            max_gap_cv: 0.20,
            max_amount_variation: 0.05,
        }
    }
}

/// Group transactions by `merchant_normalized` and detect subscriptions.
pub fn detect(txns: &[ParsedTransaction], opts: DetectOptions) -> Vec<Subscription> {
    // Group by normalized merchant (already lowercased + processor-stripped
    // by the parser).
    let mut by_merch: HashMap<String, Vec<&ParsedTransaction>> = HashMap::new();
    for t in txns {
        // Only expenses (negative amounts) qualify. Refunds skew the median.
        if t.amount >= Decimal::ZERO {
            continue;
        }
        by_merch
            .entry(t.merchant_normalized.clone())
            .or_default()
            .push(t);
    }

    let mut out = Vec::new();
    for (merch, mut rows) in by_merch {
        if rows.len() < opts.min_samples {
            continue;
        }
        rows.sort_by_key(|t| t.posted_at);

        // Gap days between consecutive postings.
        let mut gaps: Vec<f64> = Vec::with_capacity(rows.len() - 1);
        for w in rows.windows(2) {
            let d = (w[1].posted_at - w[0].posted_at).num_days() as f64;
            if d > 0.0 {
                gaps.push(d);
            }
        }
        if gaps.is_empty() {
            continue;
        }

        let avg_gap = gaps.iter().sum::<f64>() / gaps.len() as f64;
        let var = gaps.iter().map(|d| (d - avg_gap).powi(2)).sum::<f64>() / gaps.len() as f64;
        let cv = if avg_gap > 0.0 {
            var.sqrt() / avg_gap
        } else {
            f64::INFINITY
        };
        if cv > opts.max_gap_cv {
            continue;
        }

        // Amount stability — abs() each since expenses are negative.
        let mut amounts: Vec<Decimal> = rows.iter().map(|t| t.amount.abs()).collect();
        amounts.sort();
        let median = amounts[amounts.len() / 2];
        let min = *amounts.first().unwrap();
        let max = *amounts.last().unwrap();
        let variation = if median.is_zero() {
            f64::INFINITY
        } else {
            decimal_to_f64(max - min) / decimal_to_f64(median)
        };
        if variation > opts.max_amount_variation {
            continue;
        }

        let cadence = Cadence::from_avg_days(avg_gap);
        // Periods per year = 365.25 / avg_gap; multiplied by median = projected.
        let periods_per_year = if avg_gap > 0.0 { 365.25 / avg_gap } else { 0.0 };
        let projected = median * Decimal::try_from(periods_per_year).unwrap_or(Decimal::ZERO);

        out.push(Subscription {
            merchant: merch,
            cadence,
            avg_gap_days: avg_gap,
            samples: rows.len(),
            median_amount: median,
            amount_variation_pct: variation * 100.0,
            first_seen: rows.first().unwrap().posted_at,
            last_seen: rows.last().unwrap().posted_at,
            projected_annual_cost: projected,
        });
    }
    // Largest projected annual first so the user sees the biggest leaks at top.
    out.sort_by_key(|a| std::cmp::Reverse(a.projected_annual_cost));
    out
}

fn decimal_to_f64(d: Decimal) -> f64 {
    d.to_string().parse().unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn tx(merchant: &str, amount: &str, days_offset: i64) -> ParsedTransaction {
        let base = Utc::now() - Duration::days(365);
        ParsedTransaction {
            posted_at: base + Duration::days(days_offset),
            amount: d(amount),
            currency: "USD".into(),
            merchant_raw: merchant.into(),
            merchant_normalized: merchant.into(),
            description: merchant.into(),
            raw: serde_json::json!({}),
        }
    }

    #[test]
    fn empty_input_returns_no_subscriptions() {
        let out = detect(&[], DetectOptions::default());
        assert!(out.is_empty());
    }

    #[test]
    fn detects_monthly_netflix() {
        // 12 monthly hits at -$15.99 — textbook monthly subscription.
        let txns: Vec<_> = (0..12)
            .map(|i| tx("netflix", "-15.99", (i * 30) as i64))
            .collect();
        let out = detect(&txns, DetectOptions::default());
        assert_eq!(out.len(), 1);
        let s = &out[0];
        assert_eq!(s.merchant, "netflix");
        assert_eq!(s.cadence, Cadence::Monthly);
        assert_eq!(s.median_amount, d("15.99"));
        assert_eq!(s.samples, 12);
        // Projected ≈ 15.99 × (365.25/30) ≈ $194.69.
        assert!(s.projected_annual_cost > d("190"));
        assert!(s.projected_annual_cost < d("200"));
    }

    #[test]
    fn detects_annual_subscription() {
        // 3 yearly hits — minimum samples to qualify.
        let txns: Vec<_> = (0..3)
            .map(|i| tx("dropbox annual", "-99.00", (i * 365) as i64))
            .collect();
        let out = detect(&txns, DetectOptions::default());
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].cadence, Cadence::Annual);
        // 99.00 × 365.25/365 = 99.068 — small rounding from the
        // 365.25-day year. Project is within 1¢ of one billing.
        assert!(out[0].projected_annual_cost > d("99.00"));
        assert!(out[0].projected_annual_cost < d("99.10"));
    }

    #[test]
    fn skips_irregular_one_offs() {
        // Two random restaurant hits — not enough samples, irregular gaps.
        let txns = vec![
            tx("random diner", "-32.10", 0),
            tx("random diner", "-87.50", 17),
        ];
        let out = detect(&txns, DetectOptions::default());
        assert!(out.is_empty(), "only 2 samples → cannot detect cadence");
    }

    #[test]
    fn rejects_high_amount_variation() {
        // Monthly cadence but amounts swing 50% — not a subscription.
        let amounts = ["-10", "-20", "-15", "-8", "-25"];
        let txns: Vec<_> = amounts
            .iter()
            .enumerate()
            .map(|(i, a)| tx("variable charge", a, (i * 30) as i64))
            .collect();
        let out = detect(&txns, DetectOptions::default());
        assert!(
            out.is_empty(),
            "amount swings should disqualify the recurring detector"
        );
    }

    #[test]
    fn rejects_high_gap_variance() {
        // Same amount but irregular gaps (5, 60, 5, 60 days).
        let offsets = [0, 5, 65, 70, 130];
        let txns: Vec<_> = offsets
            .iter()
            .map(|o| tx("bursty merchant", "-10.00", *o))
            .collect();
        let out = detect(&txns, DetectOptions::default());
        assert!(out.is_empty(), "high gap CV must reject as irregular");
    }

    #[test]
    fn excludes_refund_only_merchants() {
        // Positive amounts are refunds, not expenses — should be skipped.
        let txns: Vec<_> = (0..6)
            .map(|i| tx("refunds", "10.00", (i * 30) as i64))
            .collect();
        let out = detect(&txns, DetectOptions::default());
        assert!(out.is_empty());
    }

    #[test]
    fn sorts_results_by_projected_cost_descending() {
        let mut txns = Vec::new();
        // Cheap monthly $5.
        for i in 0..6 {
            txns.push(tx("cheap", "-5.00", (i * 30) as i64));
        }
        // Expensive monthly $99.
        for i in 0..6 {
            txns.push(tx("expensive", "-99.00", (i * 30) as i64));
        }
        let out = detect(&txns, DetectOptions::default());
        assert_eq!(out.len(), 2);
        assert_eq!(out[0].merchant, "expensive");
        assert_eq!(out[1].merchant, "cheap");
    }

    #[test]
    fn cadence_classification_boundaries() {
        assert_eq!(Cadence::from_avg_days(7.0), Cadence::Weekly);
        assert_eq!(Cadence::from_avg_days(30.0), Cadence::Monthly);
        assert_eq!(Cadence::from_avg_days(90.0), Cadence::Quarterly);
        assert_eq!(Cadence::from_avg_days(180.0), Cadence::SemiAnnual);
        assert_eq!(Cadence::from_avg_days(365.0), Cadence::Annual);
        assert_eq!(Cadence::from_avg_days(45.0), Cadence::Irregular);
    }
}
