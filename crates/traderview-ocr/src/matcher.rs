//! Score OCR-extracted receipt fields against candidate transactions.
//!
//! Scoring weights:
//!   * amount_match     0.50  — exact within $0.01, else dropoff with absolute diff.
//!   * date_proximity   0.30  — 1.0 if same day; linear falloff to 0 at ±7 days.
//!   * merchant_overlap 0.20  — Jaccard over word-tokens of normalized merchant.
//!
//! Returns candidates with `score >= threshold` sorted descending.

use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::collections::HashSet;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ReceiptFields {
    pub merchant: Option<String>,
    pub total: Option<Decimal>,
    pub date: Option<NaiveDate>,
}

#[derive(Debug, Clone)]
pub struct TxCandidate {
    pub id: Uuid,
    pub posted_date: NaiveDate,
    pub amount: Decimal,
    pub merchant_normalized: String,
}

#[derive(Debug, Clone)]
pub struct ScoredMatch {
    pub id: Uuid,
    pub score: f32,
}

pub fn score(
    receipt: &ReceiptFields,
    candidates: &[TxCandidate],
    threshold: f32,
) -> Vec<ScoredMatch> {
    let mut out: Vec<ScoredMatch> = candidates
        .iter()
        .map(|c| ScoredMatch {
            id: c.id,
            score: score_one(receipt, c),
        })
        .filter(|m| m.score >= threshold)
        .collect();
    out.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}

fn score_one(receipt: &ReceiptFields, cand: &TxCandidate) -> f32 {
    let amount_match = match receipt.total {
        // Compare absolute values — transactions are typically negative for expenses,
        // receipts only show positive totals.
        Some(total) => {
            let diff = (total - cand.amount.abs()).abs();
            if diff <= Decimal::new(1, 2) {
                1.0
            } else if diff <= Decimal::new(50, 2) {
                // Within $0.50, linear falloff.
                1.0 - (decimal_to_f32(diff) / 0.5) * 0.5
            } else {
                0.0
            }
        }
        None => 0.0,
    };

    let date_proximity = match receipt.date {
        Some(d) => {
            let delta = (d - cand.posted_date).num_days().abs() as f32;
            if delta == 0.0 {
                1.0
            } else if delta <= 7.0 {
                1.0 - (delta / 7.0)
            } else {
                0.0
            }
        }
        None => 0.0,
    };

    let merchant_overlap = match &receipt.merchant {
        Some(m) => jaccard(&tokens(m), &tokens(&cand.merchant_normalized)),
        None => 0.0,
    };

    0.50 * amount_match + 0.30 * date_proximity + 0.20 * merchant_overlap
}

fn tokens(s: &str) -> HashSet<String> {
    s.split(|c: char| !c.is_alphanumeric())
        .filter(|t| t.len() >= 2)
        .map(|t| t.to_ascii_lowercase())
        .collect()
}

fn jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f32 {
    if a.is_empty() && b.is_empty() {
        return 0.0;
    }
    let inter = a.intersection(b).count() as f32;
    let union = a.union(b).count() as f32;
    if union == 0.0 {
        0.0
    } else {
        inter / union
    }
}

fn decimal_to_f32(d: Decimal) -> f32 {
    use std::str::FromStr;
    f32::from_str(&d.to_string()).unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Datelike;

    fn cand(id_byte: u8, day: u32, amount_cents: i64, merch: &str) -> TxCandidate {
        TxCandidate {
            id: Uuid::from_bytes([id_byte; 16]),
            posted_date: NaiveDate::from_ymd_opt(2026, 5, day).unwrap(),
            amount: Decimal::new(amount_cents, 2),
            merchant_normalized: merch.into(),
        }
    }

    /// Receipt with ALL fields None — no merchant, no total, no date.
    /// score_one returns 0.0 for each component → total 0.0. With a
    /// non-zero threshold, the candidate gets filtered out → empty
    /// result. Critical: no panic, no NaN, no Inf.
    #[test]
    fn all_none_receipt_returns_empty_matches() {
        let receipt = ReceiptFields {
            merchant: None,
            total: None,
            date: None,
        };
        let cands = vec![cand(1, 15, -450, "blue bottle coffee")];
        let matches = score(&receipt, &cands, 0.5);
        assert!(
            matches.is_empty(),
            "all-None receipt with threshold 0.5 must produce no matches"
        );
    }

    /// Receipt with only merchant set (no total, no date) — partial
    /// credit on merchant overlap only. Should produce a low score
    /// but a non-panic, well-defined result.
    #[test]
    fn merchant_only_receipt_gives_partial_credit() {
        let receipt = ReceiptFields {
            merchant: Some("Blue Bottle Coffee".into()),
            total: None,
            date: None,
        };
        let cands = vec![cand(1, 15, -450, "blue bottle coffee")];
        // amount_match = 0, date_proximity = 0, merchant_overlap = 1.0
        // Total = 0.50*0 + 0.30*0 + 0.20*1.0 = 0.20.
        // Below default threshold 0.6 → filtered out.
        let none = score(&receipt, &cands, 0.5);
        assert!(none.is_empty(), "score 0.20 < threshold 0.5 → filtered");
        // Lower threshold 0.1 → included.
        let some = score(&receipt, &cands, 0.1);
        assert_eq!(some.len(), 1);
        assert!(
            (some[0].score - 0.20).abs() < 0.01,
            "merchant-only match should score ~0.20, got {}",
            some[0].score
        );
    }

    /// Empty candidates Vec → empty results, no panic.
    #[test]
    fn empty_candidates_yields_empty_matches() {
        let receipt = ReceiptFields {
            merchant: Some("Anything".into()),
            total: Some(Decimal::new(100, 2)),
            date: Some(NaiveDate::from_ymd_opt(2026, 5, 15).unwrap()),
        };
        let matches = score(&receipt, &[], 0.0);
        assert!(matches.is_empty());
    }

    /// Threshold of 1.0 only accepts a PERFECT match. With anything
    /// less than perfect on any axis, score < 1.0 → filtered.
    #[test]
    fn threshold_1_0_only_accepts_perfect_match() {
        let receipt = ReceiptFields {
            merchant: Some("Uber".into()),
            total: Some(Decimal::new(2500, 2)),
            date: Some(NaiveDate::from_ymd_opt(2026, 5, 15).unwrap()),
        };
        // Same merchant + exact amount + exact date → score should
        // approach 1.0. But not quite — Jaccard of "Uber" vs "Uber" is
        // 1.0 ONLY if tokens match exactly. Let's pick a case where
        // it's definitively below 1.0: date off by 1.
        let cands = vec![cand(1, 16, -2500, "uber")];
        let matches = score(&receipt, &cands, 1.0);
        assert!(
            matches.is_empty(),
            "threshold 1.0 must reject day-off-by-1 (score < 1.0)"
        );
    }

    /// Threshold of 0.0 accepts everything — even zero-scoring
    /// candidates. Defensive against `>= 0.0` vs `> 0.0` comparisons.
    #[test]
    fn threshold_0_accepts_all_candidates() {
        let receipt = ReceiptFields {
            merchant: None,
            total: None,
            date: None,
        };
        let cands = vec![cand(1, 15, -450, "anything"), cand(2, 16, -100, "else")];
        let matches = score(&receipt, &cands, 0.0);
        assert_eq!(
            matches.len(),
            2,
            "threshold 0.0 must include every candidate"
        );
        // All should score exactly 0.0 (all None receipt fields).
        for m in &matches {
            assert_eq!(m.score, 0.0);
        }
    }

    #[test]
    fn exact_match_scores_high() {
        let receipt = ReceiptFields {
            merchant: Some("Blue Bottle Coffee".into()),
            total: Some(Decimal::new(450, 2)),
            date: Some(NaiveDate::from_ymd_opt(2026, 5, 15).unwrap()),
        };
        let cands = vec![cand(1, 15, -450, "blue bottle coffee")];
        let matches = score(&receipt, &cands, 0.6);
        assert_eq!(matches.len(), 1);
        assert!(matches[0].score > 0.95);
    }

    #[test]
    fn wrong_amount_loses() {
        let receipt = ReceiptFields {
            merchant: Some("Blue Bottle Coffee".into()),
            total: Some(Decimal::new(450, 2)),
            date: Some(NaiveDate::from_ymd_opt(2026, 5, 15).unwrap()),
        };
        let cands = vec![cand(1, 15, -100000, "blue bottle coffee")];
        let matches = score(&receipt, &cands, 0.6);
        // Off by $1000 — amount score = 0, date = 1.0, merchant = ~1.0
        // Total = 0.5*0 + 0.3*1.0 + 0.2*1.0 = 0.5 < 0.6 → filtered out.
        assert_eq!(matches.len(), 0);
    }

    #[test]
    fn date_falloff() {
        let receipt = ReceiptFields {
            merchant: Some("Blue Bottle".into()),
            total: Some(Decimal::new(450, 2)),
            date: Some(NaiveDate::from_ymd_opt(2026, 5, 15).unwrap()),
        };
        // 3 days off — date score ~0.57, amount 1.0, merchant 1.0 → ~0.84
        let cands = vec![cand(1, 18, -450, "blue bottle")];
        let matches = score(&receipt, &cands, 0.6);
        assert_eq!(matches.len(), 1);
        assert!(matches[0].score > 0.8 && matches[0].score < 0.9);
    }

    #[test]
    fn sorts_descending_by_score() {
        let receipt = ReceiptFields {
            merchant: Some("Uber".into()),
            total: Some(Decimal::new(2500, 2)),
            date: Some(NaiveDate::from_ymd_opt(2026, 5, 15).unwrap()),
        };
        let cands = vec![
            cand(1, 18, -2500, "uber trip"), // date off by 3, but exact amount + merchant overlap
            cand(2, 15, -2500, "uber trip"), // perfect date
            cand(3, 15, -2400, "lyft"),      // off amount, no merchant overlap
        ];
        let matches = score(&receipt, &cands, 0.5);
        assert!(matches.len() >= 2);
        assert_eq!(matches[0].id, Uuid::from_bytes([2; 16]));
        // second-best is candidate 1, since 3 doesn't overlap merchant.
        // Verify ordering — first is highest.
        for w in matches.windows(2) {
            assert!(w[0].score >= w[1].score);
        }
        // Sanity on the year being correct so chrono::Datelike pulls in.
        let _ = matches[0].id.as_bytes()[0]
            + NaiveDate::from_ymd_opt(2026, 5, 15).unwrap().year() as u8;
    }
}
