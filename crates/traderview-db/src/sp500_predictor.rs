//! S&P 500 inclusion predictor.
//!
//! The S&P 500 is methodologically rules-based: a name added on the
//! next quarterly rebalance can be predicted from public fundamentals
//! several weeks ahead. Index inclusion is one of the cleanest
//! mechanical edges in equity markets — passive funds tracking the
//! index must buy on the effective date, producing the ~6%
//! announcement-day pop and ~3% effective-day move that have shown
//! up consistently since 1989 (Lynch & Mendenhall, Chen Noronha
//! Singal 2004, and follow-ups through the 2020s).
//!
//! Criteria implemented (from the official S&P DJI methodology, as
//! published; thresholds are tunable via `Criteria` for when the
//! committee raises them):
//!
//!   1. Unadjusted market cap ≥ MIN_MARKET_CAP_USD (default
//!      $22.7B — current as of mid-2025; rises every few years).
//!   2. Public float ≥ 50% of shares outstanding.
//!   3. Annualised dollar volume ≥ FFMC × LIQUIDITY_RATIO_MIN
//!      (default 1.00 — was 0.75 historically).
//!   4. US-domiciled (country == US OR primary exchange in
//!      {NYSE, NASDAQ}).
//!   5. Trailing-4-quarter aggregate GAAP net income > 0 AND
//!      most recent quarter > 0 (the profitability gate).
//!
//! The composite 0-100 score is the equal-weighted average of the
//! per-criterion booleans, with partial credit for being close to a
//! threshold on the numeric ones (e.g., market cap at $20B scores
//! ~88% on criterion 1 if threshold is $22.7B). A symbol passes when
//! every criterion is ≥1.0 weight (i.e. all booleans true).

use serde::Serialize;

/// Tunable thresholds — separated from the per-symbol facts so the
/// committee's periodic threshold raises don't require code changes
/// at the call site. Defaults match the methodology as of mid-2025.
#[derive(Debug, Clone, Copy)]
pub struct Criteria {
    pub min_market_cap_usd: f64,
    pub min_float_ratio: f64,
    pub liquidity_ratio_min: f64,
}

impl Default for Criteria {
    fn default() -> Self {
        Self {
            min_market_cap_usd: 22_700_000_000.0,
            min_float_ratio: 0.50,
            liquidity_ratio_min: 1.00,
        }
    }
}

/// Raw facts about a candidate symbol. Every field is `Option` because
/// the upstream feeds (Yahoo quoteSummary + Finnhub metric_all) are
/// notorious for omitting individual sub-fields without warning.
#[derive(Debug, Clone)]
pub struct Candidate {
    pub symbol: String,
    pub market_cap_usd: Option<f64>,
    pub shares_outstanding: Option<f64>,
    pub float_shares: Option<f64>,
    pub avg_daily_volume: Option<f64>,
    pub price: Option<f64>,
    pub country: Option<String>,
    pub exchange: Option<String>,
    /// Trailing 4 quarters of GAAP net income, oldest first. Each entry
    /// can be None when the issuer hasn't reported that quarter yet.
    pub net_income_4q: Vec<Option<f64>>,
}

/// Per-criterion outcome — the booleans + partial-credit ratios.
#[derive(Debug, Clone, Serialize)]
pub struct CriterionResult {
    pub name: &'static str,
    pub passed: bool,
    /// 0.0–1.0. 1.0 = passes outright; lower = partial credit.
    pub partial_score: f64,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Sp500Score {
    pub symbol: String,
    /// 0–100 composite. 100 = passes every criterion outright.
    pub composite: f64,
    pub passes_all: bool,
    pub criteria: Vec<CriterionResult>,
}

/// Pure: score one candidate against the criteria.
pub fn score(candidate: &Candidate, criteria: &Criteria) -> Sp500Score {
    let mc = score_market_cap(candidate.market_cap_usd, criteria.min_market_cap_usd);
    let fl = score_float(
        candidate.float_shares,
        candidate.shares_outstanding,
        criteria.min_float_ratio,
    );
    let lq = score_liquidity(
        candidate.avg_daily_volume,
        candidate.price,
        candidate.float_shares.or(candidate.shares_outstanding),
        criteria.liquidity_ratio_min,
    );
    let dm = score_domicile(candidate.country.as_deref(), candidate.exchange.as_deref());
    let pr = score_profitability(&candidate.net_income_4q);

    let results = vec![mc, fl, lq, dm, pr];
    let composite =
        results.iter().map(|r| r.partial_score).sum::<f64>() / results.len() as f64 * 100.0;
    let passes_all = results.iter().all(|r| r.passed);

    Sp500Score {
        symbol: candidate.symbol.to_ascii_uppercase(),
        composite,
        passes_all,
        criteria: results,
    }
}

// ─── Per-criterion scoring ────────────────────────────────────────────────

fn score_market_cap(mc: Option<f64>, threshold: f64) -> CriterionResult {
    match mc {
        Some(v) if v.is_finite() && v > 0.0 => {
            let passed = v >= threshold;
            // Partial credit: linear ramp from 50% of threshold (0.0
            // score) to threshold (1.0 score). Above threshold pins to 1.
            let partial = if passed {
                1.0
            } else {
                ((v / threshold) - 0.5).max(0.0) / 0.5
            };
            CriterionResult {
                name: "market_cap",
                passed,
                partial_score: partial.clamp(0.0, 1.0),
                detail: format!("${:.1}B vs threshold ${:.1}B", v / 1e9, threshold / 1e9),
            }
        }
        _ => CriterionResult {
            name: "market_cap",
            passed: false,
            partial_score: 0.0,
            detail: "market_cap unknown".into(),
        },
    }
}

fn score_float(float: Option<f64>, shares_out: Option<f64>, threshold: f64) -> CriterionResult {
    match (float, shares_out) {
        (Some(f), Some(o)) if o > 0.0 && f.is_finite() && o.is_finite() => {
            let ratio = f / o;
            let passed = ratio >= threshold;
            // Partial credit: linear from 0 (0.0) to threshold (1.0).
            let partial = if passed { 1.0 } else { ratio / threshold };
            CriterionResult {
                name: "public_float",
                passed,
                partial_score: partial.clamp(0.0, 1.0),
                detail: format!(
                    "float {:.1}% vs threshold {:.0}%",
                    ratio * 100.0,
                    threshold * 100.0
                ),
            }
        }
        _ => CriterionResult {
            name: "public_float",
            passed: false,
            partial_score: 0.0,
            detail: "float / shares_outstanding unknown".into(),
        },
    }
}

fn score_liquidity(
    avg_volume: Option<f64>,
    price: Option<f64>,
    ffmc_shares: Option<f64>,
    threshold: f64,
) -> CriterionResult {
    match (avg_volume, price, ffmc_shares) {
        (Some(v), Some(p), Some(s)) if v > 0.0 && p > 0.0 && s > 0.0 => {
            // Annualised $-volume = avg_daily_$_volume × 252.
            let annual_dollar_volume = v * p * 252.0;
            let ffmc = s * p;
            if ffmc <= 0.0 {
                return CriterionResult {
                    name: "liquidity",
                    passed: false,
                    partial_score: 0.0,
                    detail: "FFMC zero (degenerate)".into(),
                };
            }
            let ratio = annual_dollar_volume / ffmc;
            let passed = ratio >= threshold;
            let partial = if passed { 1.0 } else { ratio / threshold };
            CriterionResult {
                name: "liquidity",
                passed,
                partial_score: partial.clamp(0.0, 1.0),
                detail: format!(
                    "annual $-vol / FFMC = {:.2} vs threshold {:.2}",
                    ratio, threshold
                ),
            }
        }
        _ => CriterionResult {
            name: "liquidity",
            passed: false,
            partial_score: 0.0,
            detail: "avg_volume / price / shares unknown".into(),
        },
    }
}

fn score_domicile(country: Option<&str>, exchange: Option<&str>) -> CriterionResult {
    let is_us_country = country
        .map(|c| {
            let c = c.to_ascii_uppercase();
            c == "US" || c == "USA" || c == "UNITED STATES"
        })
        .unwrap_or(false);
    let is_us_exchange = exchange
        .map(|e| {
            let e = e.to_ascii_uppercase();
            e.contains("NYSE") || e.contains("NASDAQ") || e == "NMS" || e == "NGM" || e == "NCM"
        })
        .unwrap_or(false);
    let passed = is_us_country || is_us_exchange;
    CriterionResult {
        name: "us_domicile",
        passed,
        partial_score: if passed { 1.0 } else { 0.0 },
        detail: format!(
            "country={} exchange={}",
            country.unwrap_or("?"),
            exchange.unwrap_or("?")
        ),
    }
}

fn score_profitability(net_income_4q: &[Option<f64>]) -> CriterionResult {
    let known: Vec<f64> = net_income_4q.iter().filter_map(|q| *q).collect();
    if known.is_empty() {
        return CriterionResult {
            name: "profitability",
            passed: false,
            partial_score: 0.0,
            detail: "no quarterly net income reported".into(),
        };
    }
    let aggregate: f64 = known.iter().sum();
    let last_quarter = *known.last().unwrap();
    let passed = aggregate > 0.0 && last_quarter > 0.0;
    // Partial credit: 0.5 for aggregate positive, 0.5 for last quarter positive.
    let partial =
        (if aggregate > 0.0 { 0.5 } else { 0.0 }) + (if last_quarter > 0.0 { 0.5 } else { 0.0 });
    CriterionResult {
        name: "profitability",
        passed,
        partial_score: partial,
        detail: format!(
            "agg ${:.0}M, last_q ${:.0}M",
            aggregate / 1e6,
            last_quarter / 1e6
        ),
    }
}

// ─── Repository: fetch + score ─────────────────────────────────────────────

/// Best-effort: pull a `Candidate` from Yahoo + Finnhub. Missing
/// sub-fields are filled with `None`; the caller's score() will
/// degrade gracefully (failing the affected criterion).
pub async fn fetch_candidate(symbol: &str) -> Candidate {
    let symbol = symbol.to_ascii_uppercase();
    let yahoo = crate::market_data::quote_summary(
        &symbol,
        &[
            "defaultKeyStatistics",
            "summaryDetail",
            "summaryProfile",
            "earnings",
            "price",
        ],
    )
    .await
    .unwrap_or(serde_json::Value::Null);

    let f64_at = |path: &[&str]| -> Option<f64> {
        let mut cur = &yahoo;
        for k in path {
            cur = cur.get(k)?;
        }
        cur.as_f64()
            .or_else(|| cur.get("raw").and_then(|r| r.as_f64()))
    };
    let str_at = |path: &[&str]| -> Option<String> {
        let mut cur = &yahoo;
        for k in path {
            cur = cur.get(k)?;
        }
        cur.as_str().map(|s| s.to_string())
    };

    // Earnings — quarterly net income from earnings.earningsChart.quarterly is
    // EPS, not net income. financialData.netIncomeToCommon is trailing 12mo.
    // Best effort: use the trailing 12-month + most-recent-quarter from
    // earnings.financialsChart.quarterly.earnings (Yahoo's nested structure).
    let mut net_income_4q: Vec<Option<f64>> = Vec::new();
    if let Some(arr) = yahoo
        .pointer("/earnings/financialsChart/quarterly")
        .and_then(|v| v.as_array())
    {
        for q in arr.iter().take(4) {
            let ni = q
                .get("earnings")
                .and_then(|e| e.get("raw").and_then(|r| r.as_f64()).or_else(|| e.as_f64()));
            net_income_4q.push(ni);
        }
    }
    Candidate {
        symbol: symbol.clone(),
        market_cap_usd: f64_at(&["price", "marketCap"])
            .or_else(|| f64_at(&["summaryDetail", "marketCap"])),
        shares_outstanding: f64_at(&["defaultKeyStatistics", "sharesOutstanding"]),
        float_shares: f64_at(&["defaultKeyStatistics", "floatShares"]),
        avg_daily_volume: f64_at(&["summaryDetail", "averageVolume10days"])
            .or_else(|| f64_at(&["summaryDetail", "averageDailyVolume3Month"]))
            .or_else(|| f64_at(&["summaryDetail", "averageVolume"])),
        price: f64_at(&["price", "regularMarketPrice"])
            .or_else(|| f64_at(&["summaryDetail", "regularMarketPrice"])),
        country: str_at(&["summaryProfile", "country"]),
        exchange: str_at(&["price", "exchange"]).or_else(|| str_at(&["price", "exchangeName"])),
        net_income_4q,
    }
}

pub async fn scan(symbols: &[String], criteria: &Criteria) -> Vec<Sp500Score> {
    let mut rows: Vec<Sp500Score> = Vec::with_capacity(symbols.len());
    for sym in symbols {
        let c = fetch_candidate(sym).await;
        let s = score(&c, criteria);
        rows.push(s);
        // Light pacing so we don't burst Yahoo when scanning ~30 names.
        tokio::time::sleep(std::time::Duration::from_millis(120)).await;
    }
    rows.sort_by(|a, b| {
        b.composite
            .partial_cmp(&a.composite)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good() -> Candidate {
        Candidate {
            symbol: "GOOD".into(),
            market_cap_usd: Some(50_000_000_000.0), // $50B
            shares_outstanding: Some(1_000_000_000.0),
            float_shares: Some(800_000_000.0), // 80% float
            avg_daily_volume: Some(5_000_000.0),
            price: Some(50.0),
            country: Some("United States".into()),
            exchange: Some("NMS".into()),
            net_income_4q: vec![Some(1e9), Some(1.1e9), Some(1.2e9), Some(1.3e9)],
        }
    }

    #[test]
    fn score_passing_candidate_returns_100() {
        let s = score(&good(), &Criteria::default());
        assert!(s.passes_all);
        assert!((s.composite - 100.0).abs() < 1e-6, "got {}", s.composite);
        assert_eq!(s.criteria.len(), 5);
    }

    #[test]
    fn market_cap_partial_credit_below_threshold() {
        // $15B vs $22.7B threshold → ratio 0.66 → partial = (0.66 - 0.5)/0.5 ≈ 0.32
        let mut c = good();
        c.market_cap_usd = Some(15_000_000_000.0);
        let s = score(&c, &Criteria::default());
        let mc = s.criteria.iter().find(|r| r.name == "market_cap").unwrap();
        assert!(!mc.passed);
        assert!(
            (mc.partial_score - 0.32).abs() < 0.02,
            "got {}",
            mc.partial_score
        );
    }

    #[test]
    fn float_fails_when_ratio_below_50pct() {
        let mut c = good();
        c.float_shares = Some(300_000_000.0); // 30% float
        let s = score(&c, &Criteria::default());
        let fl = s
            .criteria
            .iter()
            .find(|r| r.name == "public_float")
            .unwrap();
        assert!(!fl.passed);
        // Partial = 0.30 / 0.50 = 0.6
        assert!((fl.partial_score - 0.6).abs() < 0.01);
    }

    #[test]
    fn liquidity_passes_when_dollar_volume_high() {
        let s = score(&good(), &Criteria::default());
        let lq = s.criteria.iter().find(|r| r.name == "liquidity").unwrap();
        // annual $-vol = 5M * $50 * 252 = $63B
        // FFMC = 800M * $50 = $40B
        // ratio = 1.575 ≥ 1.00 → passes
        assert!(lq.passed, "{}", lq.detail);
    }

    #[test]
    fn domicile_passes_on_us_country() {
        let mut c = good();
        c.exchange = Some("Unknown".into());
        let s = score(&c, &Criteria::default());
        let dm = s.criteria.iter().find(|r| r.name == "us_domicile").unwrap();
        assert!(dm.passed);
    }

    #[test]
    fn domicile_passes_on_us_exchange_when_country_missing() {
        let mut c = good();
        c.country = None;
        c.exchange = Some("NasdaqGS".into());
        let s = score(&c, &Criteria::default());
        let dm = s.criteria.iter().find(|r| r.name == "us_domicile").unwrap();
        assert!(dm.passed);
    }

    #[test]
    fn domicile_fails_when_non_us() {
        let mut c = good();
        c.country = Some("Ireland".into());
        c.exchange = Some("LSE".into());
        let s = score(&c, &Criteria::default());
        let dm = s.criteria.iter().find(|r| r.name == "us_domicile").unwrap();
        assert!(!dm.passed);
    }

    #[test]
    fn profitability_fails_when_last_quarter_loss() {
        let mut c = good();
        // Aggregate positive but last quarter negative → must fail.
        c.net_income_4q = vec![Some(2e9), Some(2e9), Some(2e9), Some(-1e9)];
        let s = score(&c, &Criteria::default());
        let pr = s
            .criteria
            .iter()
            .find(|r| r.name == "profitability")
            .unwrap();
        assert!(!pr.passed);
        // Aggregate +ve (0.5) + last_q -ve (0.0) = 0.5 partial
        assert!((pr.partial_score - 0.5).abs() < 1e-9);
    }

    #[test]
    fn profitability_fails_when_aggregate_negative() {
        let mut c = good();
        c.net_income_4q = vec![Some(-5e9), Some(1e8), Some(1e8), Some(1e8)];
        let s = score(&c, &Criteria::default());
        let pr = s
            .criteria
            .iter()
            .find(|r| r.name == "profitability")
            .unwrap();
        assert!(!pr.passed);
    }

    #[test]
    fn profitability_zero_when_no_quarters_reported() {
        let mut c = good();
        c.net_income_4q = vec![None, None, None, None];
        let s = score(&c, &Criteria::default());
        let pr = s
            .criteria
            .iter()
            .find(|r| r.name == "profitability")
            .unwrap();
        assert!(!pr.passed);
        assert_eq!(pr.partial_score, 0.0);
    }

    #[test]
    fn fully_missing_candidate_scores_zero() {
        let blank = Candidate {
            symbol: "BLANK".into(),
            market_cap_usd: None,
            shares_outstanding: None,
            float_shares: None,
            avg_daily_volume: None,
            price: None,
            country: None,
            exchange: None,
            net_income_4q: vec![],
        };
        let s = score(&blank, &Criteria::default());
        assert_eq!(s.composite, 0.0);
        assert!(!s.passes_all);
    }

    #[test]
    fn composite_is_average_of_partial_scores() {
        // Construct a candidate that passes 3 of 5 criteria and scores
        // 0.5 / 0.5 on the other two.
        let c = Candidate {
            symbol: "MIX".into(),
            market_cap_usd: Some(15_000_000_000.0), // ~0.32 partial
            shares_outstanding: Some(1_000_000_000.0),
            float_shares: Some(800_000_000.0), // 80% → passes
            avg_daily_volume: Some(5_000_000.0),
            price: Some(50.0),
            country: Some("US".into()),
            exchange: Some("NMS".into()),
            net_income_4q: vec![Some(1e9), Some(1e9), Some(1e9), Some(1e9)],
        };
        let s = score(&c, &Criteria::default());
        // Pass: float (1.0), liquidity (1.0), domicile (1.0), profitability (1.0)
        // Partial: market_cap ≈ 0.32
        // Composite = (0.32 + 1 + 1 + 1 + 1) / 5 × 100 ≈ 86.4
        assert!(
            (s.composite - 86.4).abs() < 1.5,
            "expected ~86.4, got {}",
            s.composite
        );
        assert!(
            !s.passes_all,
            "market_cap below threshold should fail passes_all"
        );
    }
}
