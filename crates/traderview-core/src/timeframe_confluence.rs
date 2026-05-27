//! Multi-timeframe confluence scorer.
//!
//! TrendSpider's "multi-timeframe analysis" lets the trader overlay an
//! indicator from a higher timeframe onto a lower-timeframe chart. The
//! practical goal: only take a 5m long if the 1h and 1d trend agree.
//! Confluence rolls each timeframe's bullish/bearish/neutral verdict
//! into a single composite score weighted by timeframe importance.
//!
//! Caller supplies a per-timeframe verdict + weight. Output is:
//!   - **net_score**: weighted sum in `[-1, 1]` (1 = unanimous bullish)
//!   - **bias**: discrete classification (StrongBull / Bull / Neutral / Bear / StrongBear)
//!   - **agreement_pct**: fraction of timeframes agreeing with the dominant side
//!
//! Pure compute. Distinct from `triple_screen.rs` (Elder's specific 3-step
//! rule) — this module handles arbitrary timeframe counts + weights.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Stance { Bullish, Bearish, Neutral }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeframeVerdict {
    /// E.g. "1m", "5m", "1h", "1d". Just a label for the report.
    pub timeframe: String,
    pub stance: Stance,
    /// Subjective weight; caller decides. Common: 1m=0.5, 5m=1, 1h=2, 1d=4.
    pub weight: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CompositeBias {
    StrongBull,
    Bull,
    #[default]
    Neutral,
    Bear,
    StrongBear,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfluenceReport {
    pub net_score: f64,
    pub bias: CompositeBias,
    pub agreement_pct: f64,
    /// Count of bullish vs bearish vs neutral verdicts (unweighted).
    pub bull_count: usize,
    pub bear_count: usize,
    pub neutral_count: usize,
    pub note: String,
}

pub fn analyze(verdicts: &[TimeframeVerdict]) -> ConfluenceReport {
    if verdicts.is_empty() {
        return ConfluenceReport { note: "no verdicts supplied".into(), ..Default::default() };
    }
    let mut bull = 0usize;
    let mut bear = 0usize;
    let mut neutral = 0usize;
    let mut total_w = 0.0_f64;
    let mut score = 0.0_f64;
    for v in verdicts {
        let w = v.weight.max(0.0);
        total_w += w;
        match v.stance {
            Stance::Bullish => { bull += 1; score += w; }
            Stance::Bearish => { bear += 1; score -= w; }
            Stance::Neutral => { neutral += 1; }
        }
    }
    let net_score = if total_w > 0.0 { score / total_w } else { 0.0 };
    let bias = if net_score >=  0.66 { CompositeBias::StrongBull }
               else if net_score >=  0.33 { CompositeBias::Bull }
               else if net_score <= -0.66 { CompositeBias::StrongBear }
               else if net_score <= -0.33 { CompositeBias::Bear }
               else                       { CompositeBias::Neutral };
    let dominant = bull.max(bear);
    let agreement_pct = if verdicts.is_empty() { 0.0 } else { dominant as f64 / verdicts.len() as f64 };
    let note = format!(
        "{} bull / {} bear / {} neutral — net score {:.2}",
        bull, bear, neutral, net_score
    );
    ConfluenceReport {
        net_score, bias, agreement_pct,
        bull_count: bull, bear_count: bear, neutral_count: neutral, note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tf(tag: &str, s: Stance, w: f64) -> TimeframeVerdict {
        TimeframeVerdict { timeframe: tag.into(), stance: s, weight: w }
    }

    #[test]
    fn empty_input_returns_neutral_with_note() {
        let r = analyze(&[]);
        assert!(matches!(r.bias, CompositeBias::Neutral));
        assert!(r.note.contains("no verdicts"));
    }

    #[test]
    fn all_bullish_unanimous_is_strong_bull() {
        let r = analyze(&[
            tf("5m", Stance::Bullish, 1.0),
            tf("1h", Stance::Bullish, 2.0),
            tf("1d", Stance::Bullish, 4.0),
        ]);
        assert!(matches!(r.bias, CompositeBias::StrongBull));
        assert!((r.net_score - 1.0).abs() < 1e-9);
        assert_eq!(r.bull_count, 3);
        assert_eq!(r.agreement_pct, 1.0);
    }

    #[test]
    fn all_bearish_unanimous_is_strong_bear() {
        let r = analyze(&[
            tf("5m", Stance::Bearish, 1.0),
            tf("1h", Stance::Bearish, 2.0),
            tf("1d", Stance::Bearish, 4.0),
        ]);
        assert!(matches!(r.bias, CompositeBias::StrongBear));
        assert!((r.net_score - (-1.0)).abs() < 1e-9);
    }

    #[test]
    fn weighted_higher_timeframe_dominates() {
        // 5m bull (weight 1) vs 1d bear (weight 4): score = (1 - 4) / 5 = -0.6 → Bear.
        let r = analyze(&[
            tf("5m", Stance::Bullish, 1.0),
            tf("1d", Stance::Bearish, 4.0),
        ]);
        assert!(matches!(r.bias, CompositeBias::Bear),
            "1d should dominate, got {:?} (score={})", r.bias, r.net_score);
        assert!(r.net_score < 0.0);
    }

    #[test]
    fn mixed_signals_with_neutral_classify_as_neutral() {
        // 5m bull (w=1), 1h neutral (w=2), 1d bear (w=1): score = (1 - 1) / 4 = 0 → Neutral.
        let r = analyze(&[
            tf("5m", Stance::Bullish, 1.0),
            tf("1h", Stance::Neutral, 2.0),
            tf("1d", Stance::Bearish, 1.0),
        ]);
        assert!(matches!(r.bias, CompositeBias::Neutral));
    }

    #[test]
    fn agreement_pct_picks_dominant_side() {
        // 4 bull / 1 bear / 0 neutral → agreement = 4/5 = 0.8.
        let verdicts = vec![
            tf("a", Stance::Bullish, 1.0),
            tf("b", Stance::Bullish, 1.0),
            tf("c", Stance::Bullish, 1.0),
            tf("d", Stance::Bullish, 1.0),
            tf("e", Stance::Bearish, 1.0),
        ];
        let r = analyze(&verdicts);
        assert_eq!(r.bull_count, 4);
        assert_eq!(r.bear_count, 1);
        assert!((r.agreement_pct - 0.8).abs() < 1e-9);
    }

    #[test]
    fn zero_weight_verdicts_dont_affect_score() {
        // Adding a zero-weight bearish verdict to two bullish ones shouldn't shift the score.
        let r = analyze(&[
            tf("a", Stance::Bullish, 1.0),
            tf("b", Stance::Bullish, 1.0),
            tf("c", Stance::Bearish, 0.0),
        ]);
        assert!((r.net_score - 1.0).abs() < 1e-9,
            "zero-weight should be silent, got {}", r.net_score);
    }

    #[test]
    fn negative_weights_clamped_to_zero() {
        // Defensive: caller may pass a negative weight; clamp it.
        let r = analyze(&[
            tf("a", Stance::Bullish, 2.0),
            tf("b", Stance::Bearish, -10.0),
        ]);
        // Bearish weight = 0 (clamped), so only the +2 bull counts.
        assert!(r.net_score > 0.5);
    }
}
