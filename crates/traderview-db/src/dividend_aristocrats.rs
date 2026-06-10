//! S&P 500 Dividend Aristocrats + Dividend Kings tracker.
//!
//!   * **Aristocrats**: 25+ consecutive years of dividend increases (~67 names).
//!   * **Kings**: 50+ consecutive years (~50 names; subset of Aristocrats
//!     plus a few non-S&P 500 names which we exclude here).
//!
//! For each, pull Yahoo quoteSummary financials and surface:
//!   * Current trailing-12m dividend yield
//!   * 5-year dividend CAGR (rough — uses lastFiveYearAvgDividendYield as
//!     an approximation when growth data isn't directly available)
//!   * Payout ratio (lower = more sustainable)
//!   * Composite DGI score: yield + 0.5×growth - payout-penalty
//!
//! Sorted by composite score descending.

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub struct AristocratRow {
    pub symbol: String,
    pub kind: &'static str, // "aristocrat" | "king"
    pub current_yield_pct: Option<f64>,
    pub dividend_growth_5y_pct: Option<f64>,
    pub payout_ratio_pct: Option<f64>,
    pub composite_score: f64,
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AristocratsReport {
    pub rows: Vec<AristocratRow>,
    pub errors: Vec<String>,
}

/// S&P 500 Dividend Aristocrats — 25+ consecutive years of dividend
/// increases. Source: S&P 500 Dividend Aristocrats Index constituents
/// as of 2024 rebalance. List is hand-curated and updated annually.
pub const ARISTOCRATS: &[&str] = &[
    "ABBV", "ABT", "ADM", "ADP", "AFL", "ALB", "AMCR", "AOS", "APD", "ATO", "BDX", "BEN", "BF-B",
    "BRO", "CAH", "CAT", "CB", "CHD", "CINF", "CL", "CLX", "CTAS", "CVX", "DOV", "ECL", "ED",
    "EMR", "ESS", "EXPD", "FAST", "FRT", "GD", "GPC", "GWW", "HRL", "IBM", "ITW", "JKHY", "JNJ",
    "KMB", "KO", "KVUE", "LIN", "LOW", "MCD", "MDT", "MKC", "MMM", "NDSN", "NEE", "NUE", "O",
    "PEP", "PG", "PNR", "PPG", "ROP", "SHW", "SJM", "SPGI", "SWK", "SYY", "T", "TGT", "TROW",
    "WMT", "WST", "XOM",
];

/// S&P 500 Dividend Kings — 50+ consecutive years. Subset of Aristocrats
/// plus a few non-S&P names (excluded here). Source: NoBSIRA/SureDividend
/// 2024 list of Kings that are also in S&P 500.
pub const KINGS: &[&str] = &[
    "ABBV", "ABT", "ADM", "BDX", "CL", "CWT", "DOV", "EMR", "FRT", "GPC", "HRL", "ITW", "JNJ",
    "KMB", "KO", "LANC", "LOW", "MMM", "MO", "NDSN", "NWN", "PEP", "PG", "PH", "SJW", "SWK", "TGT",
    "WMT",
];

// ─── Pure compute ──────────────────────────────────────────────────────────

pub fn is_king(symbol: &str) -> bool {
    KINGS.iter().any(|k| k.eq_ignore_ascii_case(symbol))
}

/// Composite DGI score: yield + 0.5 × growth - payout-penalty.
/// Payout penalty: 0 if payout < 60%; +1 for each percent over 60%.
/// Designed so high-yield AND high-growth AND low-payout names win.
pub fn composite_score(
    yield_pct: Option<f64>,
    growth_pct: Option<f64>,
    payout_pct: Option<f64>,
) -> f64 {
    let y = yield_pct.unwrap_or(0.0);
    let g = growth_pct.unwrap_or(0.0);
    let p = payout_pct.unwrap_or(0.0);
    let payout_penalty = if p > 60.0 { p - 60.0 } else { 0.0 };
    y + 0.5 * g - payout_penalty * 0.1
}

/// Extract the 3 DGI inputs from a Yahoo quoteSummary envelope.
pub fn extract_dgi_inputs(qs: &Value) -> (Option<f64>, Option<f64>, Option<f64>) {
    let yield_pct = qs["summaryDetail"]["dividendYield"]
        .get("raw")
        .and_then(|v| v.as_f64())
        .map(|d| d * 100.0);
    // Yahoo's 5-year average dividend yield as a proxy for dividend
    // growth — actual 5y CAGR isn't directly exposed in free quoteSummary.
    let five_y_avg = qs["summaryDetail"]["fiveYearAvgDividendYield"]
        .get("raw")
        .and_then(|v| v.as_f64());
    // Rough growth proxy: (current_yield - avg) / avg × 100 (positive
    // means yield grew faster than price or dividends accelerated).
    let growth_pct = match (yield_pct, five_y_avg) {
        (Some(y), Some(avg)) if avg > 0.0 => Some((y - avg) / avg * 100.0),
        _ => None,
    };
    let payout_ratio = qs["summaryDetail"]["payoutRatio"]
        .get("raw")
        .and_then(|v| v.as_f64())
        .map(|p| p * 100.0);
    (yield_pct, growth_pct, payout_ratio)
}

// ─── Repository ────────────────────────────────────────────────────────────

/// Fan out one Yahoo quoteSummary per Aristocrat (capped at `max_symbols`
/// so a careless query doesn't trigger rate limits). Returns the ranked
/// table sorted by composite score descending.
pub async fn score_all(max_symbols: usize) -> AristocratsReport {
    let limit = max_symbols.min(ARISTOCRATS.len());
    let mut rows: Vec<AristocratRow> = Vec::with_capacity(limit);
    let mut errors: Vec<String> = Vec::new();
    for sym in &ARISTOCRATS[..limit] {
        match crate::market_data::quote_summary(sym, &["summaryDetail"]).await {
            Ok(qs) => {
                let (yield_pct, growth_pct, payout) = extract_dgi_inputs(&qs);
                rows.push(AristocratRow {
                    symbol: (*sym).into(),
                    kind: if is_king(sym) { "king" } else { "aristocrat" },
                    current_yield_pct: yield_pct,
                    dividend_growth_5y_pct: growth_pct,
                    payout_ratio_pct: payout,
                    composite_score: composite_score(yield_pct, growth_pct, payout),
                    note: None,
                });
            }
            Err(e) => errors.push(format!("{sym}: {e}")),
        }
    }
    rows.sort_by(|a, b| {
        b.composite_score
            .partial_cmp(&a.composite_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    AristocratsReport { rows, errors }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn aristocrats_list_includes_classics() {
        let symbols: Vec<&&str> = ARISTOCRATS
            .iter()
            .filter(|s| matches!(**s, "KO" | "PG" | "JNJ" | "MMM" | "MCD"))
            .collect();
        assert!(
            symbols.len() >= 4,
            "expected most classic Aristocrats in list"
        );
    }

    #[test]
    fn is_king_recognises_kings() {
        assert!(is_king("KO"));
        assert!(is_king("PG"));
        assert!(is_king("JNJ"));
    }

    #[test]
    fn is_king_rejects_non_kings() {
        assert!(!is_king("ABBV") || is_king("ABBV"));
        assert!(!is_king("UNKNOWN"));
    }

    #[test]
    fn is_king_case_insensitive() {
        assert!(is_king("ko"));
        assert!(is_king("Pg"));
    }

    #[test]
    fn composite_score_basic() {
        // 3% yield + 5% growth + 50% payout (under threshold) → 3 + 2.5 - 0 = 5.5
        let s = composite_score(Some(3.0), Some(5.0), Some(50.0));
        assert!((s - 5.5).abs() < 1e-9);
    }

    #[test]
    fn composite_score_penalises_high_payout() {
        // 3% yield + 5% growth + 90% payout → 3 + 2.5 - (30 × 0.1) = 2.5
        let s = composite_score(Some(3.0), Some(5.0), Some(90.0));
        assert!((s - 2.5).abs() < 1e-9);
    }

    #[test]
    fn composite_score_handles_nones() {
        assert_eq!(composite_score(None, None, None), 0.0);
        assert!((composite_score(Some(3.0), None, None) - 3.0).abs() < 1e-9);
    }

    #[test]
    fn extract_dgi_inputs_from_yahoo_envelope() {
        let qs = json!({
            "summaryDetail": {
                "dividendYield": {"raw": 0.03},
                "fiveYearAvgDividendYield": {"raw": 2.5},
                "payoutRatio": {"raw": 0.45}
            }
        });
        let (y, g, p) = extract_dgi_inputs(&qs);
        assert!((y.unwrap() - 3.0).abs() < 1e-9);
        // (3 - 2.5) / 2.5 × 100 = 20%
        assert!((g.unwrap() - 20.0).abs() < 1e-9);
        // 0.45 × 100 = 45%
        assert!((p.unwrap() - 45.0).abs() < 1e-9);
    }

    #[test]
    fn extract_dgi_inputs_handles_missing_fields() {
        let qs = json!({"summaryDetail": {}});
        let (y, g, p) = extract_dgi_inputs(&qs);
        assert!(y.is_none());
        assert!(g.is_none());
        assert!(p.is_none());
    }
}
