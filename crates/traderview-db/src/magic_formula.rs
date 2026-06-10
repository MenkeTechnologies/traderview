//! Greenblatt magic formula value scorer.
//!
//! Two simple rankings, combined:
//!
//!   1. **Earnings yield** = EBIT / Enterprise Value
//!      How much pre-tax profit you get per dollar of total firm value
//!      (debt + equity - cash). Higher = cheaper.
//!
//!   2. **Return on invested capital** = EBIT / Invested Capital
//!      How efficiently each dollar of capital is turned into profit.
//!      Higher = better business quality.
//!
//! Combined rank = sum of the two individual ranks. Lower combined rank
//! = better candidate (cheap + high-quality). The book ranks across the
//! S&P 500 universe and recommends holding the top 20-30 for one year,
//! rebalancing annually.
//!
//! This module does the per-symbol math; the route layer fans out across
//! a configurable universe (defaults to the heatmap UNIVERSE top-S&P
//! names) and returns the combined ranking.

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub struct MagicFormulaScore {
    pub symbol: String,
    pub ebit_usd: Option<f64>,
    pub enterprise_value_usd: Option<f64>,
    pub invested_capital_usd: Option<f64>,
    pub earnings_yield_pct: Option<f64>,
    pub roic_pct: Option<f64>,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MagicFormulaRow {
    pub symbol: String,
    pub earnings_yield_pct: f64,
    pub roic_pct: f64,
    pub earnings_yield_rank: usize,
    pub roic_rank: usize,
    pub combined_rank: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct MagicFormulaReport {
    pub universe_size: usize,
    pub scored: Vec<MagicFormulaRow>,
    pub errors: Vec<String>,
}

// ─── Pure compute ──────────────────────────────────────────────────────────

/// Extract Greenblatt inputs from a Yahoo quoteSummary JSON envelope.
/// Returns Nones for the parts we can't find rather than failing — the
/// route layer decides what to do with partial data.
pub fn extract_inputs(symbol: &str, qs: &Value) -> MagicFormulaScore {
    let ebit_usd = qs["financialData"]["ebitda"]
        .get("raw")
        .and_then(|v| v.as_f64())
        .or_else(|| {
            pluck(
                &qs["incomeStatementHistory"]["incomeStatementHistory"],
                0,
                "operatingIncome",
            )
        });
    let enterprise_value_usd = qs["defaultKeyStatistics"]["enterpriseValue"]
        .get("raw")
        .and_then(|v| v.as_f64());
    // Invested capital ≈ total equity + total debt - cash.
    // From balanceSheetHistory[0]:
    //   totalStockholderEquity, totalLiab (no debt-only field), cash
    // Use longTermDebt + shortLongTermDebt when present, else totalLiab.
    let bs = &qs["balanceSheetHistory"]["balanceSheetStatements"];
    let total_equity = pluck(bs, 0, "totalStockholderEquity");
    let long_term_debt = pluck(bs, 0, "longTermDebt").unwrap_or(0.0);
    let short_long_debt = pluck(bs, 0, "shortLongTermDebt").unwrap_or(0.0);
    let cash = pluck(bs, 0, "cash").unwrap_or(0.0);
    let invested_capital_usd = total_equity.map(|eq| eq + long_term_debt + short_long_debt - cash);

    let earnings_yield_pct = match (ebit_usd, enterprise_value_usd) {
        (Some(e), Some(ev)) if ev > 0.0 => Some(e / ev * 100.0),
        _ => None,
    };
    let roic_pct = match (ebit_usd, invested_capital_usd) {
        (Some(e), Some(ic)) if ic > 0.0 => Some(e / ic * 100.0),
        _ => None,
    };

    MagicFormulaScore {
        symbol: symbol.into(),
        ebit_usd,
        enterprise_value_usd,
        invested_capital_usd,
        earnings_yield_pct,
        roic_pct,
        note: None,
    }
}

fn pluck(arr: &Value, index: usize, key: &str) -> Option<f64> {
    arr.get(index)?.get(key)?.get("raw")?.as_f64()
}

/// Combine per-symbol scores into the final ranked output. Drops symbols
/// missing either input. Rank 1 is best (highest earnings yield AND
/// highest ROIC); combined_rank is the sum of the two individual ranks.
pub fn rank(scores: &[MagicFormulaScore]) -> Vec<MagicFormulaRow> {
    let usable: Vec<&MagicFormulaScore> = scores
        .iter()
        .filter(|s| s.earnings_yield_pct.is_some() && s.roic_pct.is_some())
        .collect();
    if usable.is_empty() {
        return Vec::new();
    }
    // Sort by earnings yield descending, assign rank.
    let mut by_yield: Vec<&MagicFormulaScore> = usable.clone();
    by_yield.sort_by(|a, b| {
        b.earnings_yield_pct
            .partial_cmp(&a.earnings_yield_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let yield_rank: std::collections::HashMap<&str, usize> = by_yield
        .iter()
        .enumerate()
        .map(|(i, s)| (s.symbol.as_str(), i + 1))
        .collect();

    let mut by_roic: Vec<&MagicFormulaScore> = usable.clone();
    by_roic.sort_by(|a, b| {
        b.roic_pct
            .partial_cmp(&a.roic_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let roic_rank: std::collections::HashMap<&str, usize> = by_roic
        .iter()
        .enumerate()
        .map(|(i, s)| (s.symbol.as_str(), i + 1))
        .collect();

    let mut rows: Vec<MagicFormulaRow> = usable
        .into_iter()
        .map(|s| {
            let yr = yield_rank[s.symbol.as_str()];
            let rr = roic_rank[s.symbol.as_str()];
            MagicFormulaRow {
                symbol: s.symbol.clone(),
                earnings_yield_pct: s.earnings_yield_pct.unwrap_or(0.0),
                roic_pct: s.roic_pct.unwrap_or(0.0),
                earnings_yield_rank: yr,
                roic_rank: rr,
                combined_rank: yr + rr,
            }
        })
        .collect();
    rows.sort_by_key(|r| r.combined_rank);
    rows
}

// ─── Repository ────────────────────────────────────────────────────────────

/// Default universe: top S&P names from the existing heatmap mapping
/// (deduped, ~210 symbols). Trades sector breadth for coverage.
pub fn default_universe() -> Vec<&'static str> {
    use std::collections::HashSet;
    let mut seen: HashSet<&'static str> = HashSet::new();
    let mut out: Vec<&'static str> = Vec::new();
    for (_, names) in crate::heatmap::UNIVERSE_EXPORT.iter() {
        for n in *names {
            if seen.insert(*n) {
                out.push(*n);
            }
        }
    }
    out
}

/// Score the requested universe. Fans out one Yahoo quoteSummary call per
/// symbol; capped at `max_symbols` so a careless query doesn't trigger
/// rate limits.
pub async fn score_universe(symbols: &[&str], max_symbols: usize) -> MagicFormulaReport {
    let limit = max_symbols.min(symbols.len());
    let mut scores: Vec<MagicFormulaScore> = Vec::with_capacity(limit);
    let mut errors: Vec<String> = Vec::new();
    for s in &symbols[..limit] {
        match crate::market_data::quote_summary(
            s,
            &[
                "financialData",
                "defaultKeyStatistics",
                "incomeStatementHistory",
                "balanceSheetHistory",
            ],
        )
        .await
        {
            Ok(qs) => {
                let score = extract_inputs(s, &qs);
                scores.push(score);
            }
            Err(e) => errors.push(format!("{s}: {e}")),
        }
    }
    let scored = rank(&scores);
    MagicFormulaReport {
        universe_size: limit,
        scored,
        errors,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn score(sym: &str, yield_pct: f64, roic_pct: f64) -> MagicFormulaScore {
        MagicFormulaScore {
            symbol: sym.into(),
            ebit_usd: Some(1.0),
            enterprise_value_usd: Some(1.0),
            invested_capital_usd: Some(1.0),
            earnings_yield_pct: Some(yield_pct),
            roic_pct: Some(roic_pct),
            note: None,
        }
    }

    #[test]
    fn rank_combines_two_ranks() {
        // A: best yield (1st), worst ROIC (3rd) → combined 4
        // B: 2nd yield, 2nd ROIC → combined 4
        // C: worst yield (3rd), best ROIC (1st) → combined 4
        // All tied at 4; order is implementation-defined but combined ranks must equal.
        let scores = vec![
            score("A", 15.0, 5.0),
            score("B", 10.0, 10.0),
            score("C", 5.0, 15.0),
        ];
        let ranked = rank(&scores);
        assert_eq!(ranked.len(), 3);
        for r in &ranked {
            assert_eq!(r.combined_rank, 4, "all tied at 4 in this symmetric case");
        }
    }

    #[test]
    fn rank_winner_dominates_on_both() {
        // A: 1st yield + 1st ROIC = 2 (best)
        // B: 2nd + 2nd = 4
        // C: 3rd + 3rd = 6 (worst)
        let scores = vec![
            score("A", 15.0, 15.0),
            score("B", 10.0, 10.0),
            score("C", 5.0, 5.0),
        ];
        let ranked = rank(&scores);
        assert_eq!(ranked[0].symbol, "A");
        assert_eq!(ranked[0].combined_rank, 2);
        assert_eq!(ranked[2].symbol, "C");
        assert_eq!(ranked[2].combined_rank, 6);
    }

    #[test]
    fn rank_drops_symbols_missing_inputs() {
        let mut scores = vec![score("A", 15.0, 10.0), score("B", 10.0, 5.0)];
        scores.push(MagicFormulaScore {
            symbol: "INCOMPLETE".into(),
            ebit_usd: Some(1.0),
            enterprise_value_usd: None,
            invested_capital_usd: Some(1.0),
            earnings_yield_pct: None,
            roic_pct: Some(5.0),
            note: None,
        });
        let ranked = rank(&scores);
        let syms: Vec<&str> = ranked.iter().map(|r| r.symbol.as_str()).collect();
        assert_eq!(syms, vec!["A", "B"], "INCOMPLETE dropped");
    }

    #[test]
    fn rank_empty_input_empty_output() {
        assert!(rank(&[]).is_empty());
    }

    #[test]
    fn extract_inputs_handles_missing_fields_gracefully() {
        let qs = json!({"financialData": {}, "defaultKeyStatistics": {}, "balanceSheetHistory": {}, "incomeStatementHistory": {}});
        let s = extract_inputs("AAA", &qs);
        assert_eq!(s.symbol, "AAA");
        assert!(s.earnings_yield_pct.is_none());
        assert!(s.roic_pct.is_none());
    }

    #[test]
    fn extract_inputs_computes_yield_when_inputs_present() {
        let qs = json!({
            "financialData": {"ebitda": {"raw": 1_000_000_000_u64}},
            "defaultKeyStatistics": {"enterpriseValue": {"raw": 10_000_000_000_u64}},
            "balanceSheetHistory": {
                "balanceSheetStatements": [{
                    "totalStockholderEquity": {"raw": 5_000_000_000_u64},
                    "longTermDebt": {"raw": 2_000_000_000_u64},
                    "cash": {"raw": 1_000_000_000_u64},
                }]
            },
            "incomeStatementHistory": {"incomeStatementHistory": []},
        });
        let s = extract_inputs("AAA", &qs);
        // EBIT/EV = 1B / 10B = 10%
        assert!((s.earnings_yield_pct.unwrap() - 10.0).abs() < 1e-9);
        // Invested capital = 5B + 2B + 0 - 1B = 6B
        // ROIC = 1B / 6B ≈ 16.67%
        assert!((s.roic_pct.unwrap() - 16.666666666666664).abs() < 1e-6);
    }

    #[test]
    fn extract_inputs_falls_back_to_income_statement_ebit() {
        let qs = json!({
            "financialData": {},
            "defaultKeyStatistics": {"enterpriseValue": {"raw": 10_000_000_000_u64}},
            "balanceSheetHistory": {
                "balanceSheetStatements": [{
                    "totalStockholderEquity": {"raw": 10_000_000_000_u64},
                }]
            },
            "incomeStatementHistory": {
                "incomeStatementHistory": [{
                    "operatingIncome": {"raw": 2_000_000_000_u64}
                }]
            },
        });
        let s = extract_inputs("BBB", &qs);
        // 2B / 10B = 20% yield
        assert!((s.earnings_yield_pct.unwrap() - 20.0).abs() < 1e-9);
    }
}
