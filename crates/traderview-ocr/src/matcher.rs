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
