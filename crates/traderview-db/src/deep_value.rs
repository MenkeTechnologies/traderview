//! Deep-value screen — three classic metrics from the latest annual
//! report (Finnhub `financials_reported`, same payload Beneish uses)
//! plus a live quote:
//!
//! * Graham NCAV / net-net — NCAV = current assets − total
//!   liabilities; a stock trading under 2/3 of NCAV per share is in
//!   Graham's net-net buy zone.
//! * Acquirer's Multiple (Carlisle) — enterprise value / operating
//!   earnings; lower = cheaper to a control buyer.
//! * Shareholder yield — (dividends + net buybacks + net debt
//!   paydown) / market cap, the cash actually returned to owners.
//!
//! Like Beneish, missing line items are surfaced in `missing` instead
//! of failing the whole report — only shares outstanding and a
//! positive price are hard requirements (market cap anchors all
//! three metrics).

use crate::beneish::find_concept;
use serde::Serialize;
use serde_json::Value;
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize)]
pub struct DeepValueReport {
    pub symbol: String,
    pub year: i64,
    pub price: f64,
    pub shares_outstanding: f64,
    pub market_cap: f64,
    // ── Graham NCAV ──
    pub ncav_total: Option<f64>,
    pub ncav_per_share: Option<f64>,
    /// Price ÷ NCAV/share (< 0.667 = net-net buy zone).
    pub price_to_ncav: Option<f64>,
    pub is_net_net: bool,
    /// 2/3 of NCAV per share — Graham's classic entry.
    pub net_net_buy_price: Option<f64>,
    // ── Acquirer's Multiple ──
    pub enterprise_value: Option<f64>,
    pub operating_income: Option<f64>,
    /// EV / operating income (only when operating income > 0).
    pub acquirers_multiple: Option<f64>,
    // ── Shareholder yield ──
    pub dividend_yield_pct: Option<f64>,
    pub net_buyback_yield_pct: Option<f64>,
    pub net_debt_paydown_yield_pct: Option<f64>,
    /// Sum of the available components.
    pub shareholder_yield_pct: Option<f64>,
    pub missing: Vec<&'static str>,
}

#[derive(Debug, thiserror::Error)]
pub enum DeepValueError {
    #[error("financials fetch failed: {0}")]
    Fetch(anyhow::Error),
    #[error("no annual report found for {symbol}")]
    NoAnnualReport { symbol: String },
    #[error("shares outstanding not found for {symbol}")]
    NoShares { symbol: String },
    #[error("no positive price for {symbol}")]
    NoPrice { symbol: String },
}

/// Find the first concept whose tag ENDS WITH one of `exact` (so
/// "Liabilities" can't match "LiabilitiesCurrent"); fall back to the
/// substring search shared with Beneish.
fn find_exact_or(report: &Value, exact: &[&str], fuzzy: &[&str]) -> Option<f64> {
    for section in ["bs", "ic", "cf"] {
        let Some(items) = report
            .pointer(&format!("/report/{section}"))
            .and_then(|x| x.as_array())
        else {
            continue;
        };
        for item in items {
            let concept = item
                .get("concept")
                .and_then(|c| c.as_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if exact.iter().any(|e| concept.ends_with(&e.to_ascii_lowercase())) {
                if let Some(v) = item.get("value").and_then(|v| v.as_f64()) {
                    return Some(v);
                }
                if let Some(v) = item
                    .get("value")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
                {
                    return Some(v);
                }
            }
        }
    }
    if fuzzy.is_empty() {
        None
    } else {
        find_concept(report, fuzzy)
    }
}

pub async fn compute(pool: &PgPool, symbol: &str) -> Result<DeepValueReport, DeepValueError> {
    let raw = crate::finnhub_rest::financials_reported(symbol)
        .await
        .map_err(DeepValueError::Fetch)?;
    let quote = crate::market_data::quote(pool, symbol)
        .await
        .map_err(DeepValueError::Fetch)?;
    compute_from_parts(symbol, &raw, quote.price)
}

/// Pure scoring from a financials_reported payload + price. Split for
/// tests, mirroring beneish::compute_from_reports.
pub fn compute_from_parts(
    symbol: &str,
    raw: &Value,
    price: f64,
) -> Result<DeepValueReport, DeepValueError> {
    if price <= 0.0 {
        return Err(DeepValueError::NoPrice {
            symbol: symbol.to_string(),
        });
    }
    let report = raw
        .get("data")
        .and_then(|d| d.as_array())
        .and_then(|arr| {
            arr.iter()
                .find(|r| r.get("quarter").and_then(|q| q.as_i64()) == Some(0))
        })
        .ok_or_else(|| DeepValueError::NoAnnualReport {
            symbol: symbol.to_string(),
        })?;
    let year = report.get("year").and_then(|y| y.as_i64()).unwrap_or(0);
    let shares = find_concept(
        report,
        &[
            "weightedaveragenumberofdilutedsharesoutstanding",
            "weightedaveragenumberofsharesoutstanding",
            "entitycommonstocksharesoutstanding",
        ],
    )
    .filter(|s| *s > 0.0)
    .ok_or_else(|| DeepValueError::NoShares {
        symbol: symbol.to_string(),
    })?;
    let market_cap = price * shares;
    let mut missing: Vec<&'static str> = Vec::new();
    let mut want = |name: &'static str, v: Option<f64>| -> Option<f64> {
        if v.is_none() {
            missing.push(name);
        }
        v
    };

    // ── NCAV ──
    let current_assets = want("current_assets", find_exact_or(report, &["assetscurrent"], &[]));
    let total_liabilities = want("total_liabilities", find_exact_or(report, &["liabilities"], &[]));
    let ncav_total = match (current_assets, total_liabilities) {
        (Some(ca), Some(tl)) => Some(ca - tl),
        _ => None,
    };
    let ncav_per_share = ncav_total.map(|n| n / shares);
    let price_to_ncav = ncav_per_share.filter(|n| *n > 0.0).map(|n| price / n);
    let is_net_net = price_to_ncav.map(|r| r < 2.0 / 3.0).unwrap_or(false);
    let net_net_buy_price = ncav_per_share.filter(|n| *n > 0.0).map(|n| n * 2.0 / 3.0);

    // ── Acquirer's Multiple ──
    let operating_income = want(
        "operating_income",
        find_exact_or(report, &["operatingincomeloss"], &[]),
    );
    let cash = want(
        "cash",
        find_exact_or(
            report,
            &["cashandcashequivalentsatcarryingvalue"],
            &["cashandcashequivalents", "cashcashequivalents"],
        ),
    );
    let lt_debt = find_concept(report, &["longtermdebtnoncurrent", "longtermdebt"]);
    let st_debt = find_concept(report, &["longtermdebtcurrent", "shorttermborrowings", "debtcurrent"]);
    let total_debt = want(
        "debt",
        match (lt_debt, st_debt) {
            (None, None) => None,
            (a, b) => Some(a.unwrap_or(0.0) + b.unwrap_or(0.0)),
        },
    );
    let enterprise_value = match (total_debt, cash) {
        (Some(d), Some(c)) => Some(market_cap + d - c),
        _ => None,
    };
    let acquirers_multiple = match (enterprise_value, operating_income) {
        (Some(ev), Some(oi)) if oi > 0.0 => Some(ev / oi),
        _ => None,
    };

    // ── Shareholder yield ── (XBRL payments/proceeds are positive)
    let dividends = want("dividends_paid", find_concept(report, &["paymentsofdividends"]));
    let buybacks = find_concept(report, &["paymentsforrepurchaseofcommonstock"]);
    let issuance = find_concept(report, &["proceedsfromissuanceofcommonstock"]);
    let debt_repaid = find_concept(report, &["repaymentsoflongtermdebt", "repaymentsofdebt"]);
    let debt_issued = find_concept(report, &["proceedsfromissuanceoflongtermdebt", "proceedsfromissuanceofdebt"]);
    let yield_pct = |v: f64| v / market_cap * 100.0;
    let dividend_yield_pct = dividends.map(yield_pct);
    let net_buyback_yield_pct = match (buybacks, issuance) {
        (None, None) => None,
        (b, i) => Some(yield_pct(b.unwrap_or(0.0) - i.unwrap_or(0.0))),
    };
    let net_debt_paydown_yield_pct = match (debt_repaid, debt_issued) {
        (None, None) => None,
        (r, i) => Some(yield_pct(r.unwrap_or(0.0) - i.unwrap_or(0.0))),
    };
    let components: Vec<f64> = [dividend_yield_pct, net_buyback_yield_pct, net_debt_paydown_yield_pct]
        .iter()
        .flatten()
        .copied()
        .collect();
    let shareholder_yield_pct = if components.is_empty() {
        None
    } else {
        Some(components.iter().sum())
    };

    Ok(DeepValueReport {
        symbol: symbol.to_string(),
        year,
        price,
        shares_outstanding: shares,
        market_cap,
        ncav_total,
        ncav_per_share,
        price_to_ncav,
        is_net_net,
        net_net_buy_price,
        enterprise_value,
        operating_income,
        acquirers_multiple,
        dividend_yield_pct,
        net_buyback_yield_pct,
        net_debt_paydown_yield_pct,
        shareholder_yield_pct,
        missing,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn payload(bs: Value, ic: Value, cf: Value) -> Value {
        json!({ "data": [{ "year": 2025, "quarter": 0,
                           "report": { "bs": bs, "ic": ic, "cf": cf } }] })
    }

    fn item(concept: &str, value: f64) -> Value {
        json!({ "concept": concept, "value": value })
    }

    fn full_payload() -> Value {
        payload(
            json!([
                item("us-gaap_AssetsCurrent", 900.0),
                item("us-gaap_LiabilitiesCurrent", 250.0),
                item("us-gaap_Liabilities", 400.0),
                item("us-gaap_CashAndCashEquivalentsAtCarryingValue", 100.0),
                item("us-gaap_LongTermDebtNoncurrent", 200.0),
                item("us-gaap_LongTermDebtCurrent", 50.0),
            ]),
            json!([
                item("us-gaap_OperatingIncomeLoss", 90.0),
                item("us-gaap_WeightedAverageNumberOfDilutedSharesOutstanding", 100.0),
            ]),
            json!([
                item("us-gaap_PaymentsOfDividends", 30.0),
                item("us-gaap_PaymentsForRepurchaseOfCommonStock", 20.0),
                item("us-gaap_ProceedsFromIssuanceOfCommonStock", 5.0),
                item("us-gaap_RepaymentsOfLongTermDebt", 60.0),
                item("us-gaap_ProceedsFromIssuanceOfLongTermDebt", 40.0),
            ]),
        )
    }

    #[test]
    fn net_net_detected_below_two_thirds_ncav() {
        // NCAV = 900 − 400 = 500 → $5/share; price $3 < $3.33 buy zone.
        let r = compute_from_parts("TEST", &full_payload(), 3.0).unwrap();
        assert_eq!(r.ncav_total, Some(500.0));
        assert_eq!(r.ncav_per_share, Some(5.0));
        assert!((r.price_to_ncav.unwrap() - 0.6).abs() < 1e-12);
        assert!(r.is_net_net);
        assert!((r.net_net_buy_price.unwrap() - 10.0 / 3.0).abs() < 1e-12);
        assert!(r.missing.is_empty(), "{:?}", r.missing);
    }

    #[test]
    fn total_liabilities_uses_exact_tag_not_liabilities_current() {
        // If substring matching grabbed LiabilitiesCurrent (250), NCAV
        // would read 650 — the ends-with rule must pick 400.
        let r = compute_from_parts("TEST", &full_payload(), 3.0).unwrap();
        assert_eq!(r.ncav_total, Some(500.0));
    }

    #[test]
    fn acquirers_multiple_from_ev_over_operating_income() {
        // mcap = 3 × 100 = 300; EV = 300 + (200+50) − 100 = 450; /90 = 5.
        let r = compute_from_parts("TEST", &full_payload(), 3.0).unwrap();
        assert_eq!(r.enterprise_value, Some(450.0));
        assert!((r.acquirers_multiple.unwrap() - 5.0).abs() < 1e-12);
    }

    #[test]
    fn shareholder_yield_sums_net_components() {
        // mcap 300: div 30 → 10%; buyback net 20−5=15 → 5%; debt net
        // 60−40=20 → 6.667%; total ≈ 21.667%.
        let r = compute_from_parts("TEST", &full_payload(), 3.0).unwrap();
        assert!((r.dividend_yield_pct.unwrap() - 10.0).abs() < 1e-9);
        assert!((r.net_buyback_yield_pct.unwrap() - 5.0).abs() < 1e-9);
        assert!((r.net_debt_paydown_yield_pct.unwrap() - 20.0 / 3.0).abs() < 1e-9);
        assert!((r.shareholder_yield_pct.unwrap() - (15.0 + 20.0 / 3.0)).abs() < 1e-9);
    }

    #[test]
    fn missing_items_are_reported_not_fatal() {
        // Only shares present: every metric degrades to None + missing.
        let raw = payload(
            json!([]),
            json!([item("us-gaap_WeightedAverageNumberOfSharesOutstanding", 100.0)]),
            json!([]),
        );
        let r = compute_from_parts("TEST", &raw, 10.0).unwrap();
        assert_eq!(r.ncav_total, None);
        assert_eq!(r.acquirers_multiple, None);
        assert_eq!(r.shareholder_yield_pct, None);
        assert!(!r.is_net_net);
        for key in ["current_assets", "total_liabilities", "operating_income", "cash", "debt", "dividends_paid"] {
            assert!(r.missing.contains(&key), "missing {key}: {:?}", r.missing);
        }
    }

    #[test]
    fn negative_operating_income_yields_no_multiple() {
        let raw = payload(
            json!([
                item("us-gaap_CashAndCashEquivalentsAtCarryingValue", 10.0),
                item("us-gaap_LongTermDebt", 5.0),
            ]),
            json!([
                item("us-gaap_OperatingIncomeLoss", -50.0),
                item("us-gaap_WeightedAverageNumberOfSharesOutstanding", 10.0),
            ]),
            json!([]),
        );
        let r = compute_from_parts("TEST", &raw, 10.0).unwrap();
        assert_eq!(r.operating_income, Some(-50.0));
        assert_eq!(r.acquirers_multiple, None);
    }

    #[test]
    fn hard_requirements_reject() {
        assert!(matches!(
            compute_from_parts("TEST", &full_payload(), 0.0),
            Err(DeepValueError::NoPrice { .. })
        ));
        let no_shares = payload(json!([]), json!([]), json!([]));
        assert!(matches!(
            compute_from_parts("TEST", &no_shares, 10.0),
            Err(DeepValueError::NoShares { .. })
        ));
        assert!(matches!(
            compute_from_parts("TEST", &json!({"data": []}), 10.0),
            Err(DeepValueError::NoAnnualReport { .. })
        ));
    }
}
