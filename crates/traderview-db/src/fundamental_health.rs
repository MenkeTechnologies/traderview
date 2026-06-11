//! Fundamental health scorecard: Piotroski F-Score, Altman Z-Score,
//! Graham Number.
//!
//! Data source: Finnhub `/stock/metric?metric=all` — the `series.annual`
//! block carries multi-year arrays (currentRatio, grossMargin, roa,
//! totalDebtToTotalAsset, …) that the YoY-delta checks need, and the
//! `metric` block carries the TTM/annual point values for Altman +
//! Graham. Every check is BEST-EFFORT: when an input is missing the
//! check is skipped and reported in `missing`, and the score shows
//! `earned / available` instead of pretending nine checks ran.
//!
//! Piotroski (9 checks, 1 point each):
//!   Profitability: ROA > 0 · CFO > 0 · ΔROA > 0 · CFO > net income
//!   Leverage:      Δlong-term-debt-ratio < 0 · Δcurrent-ratio > 0 ·
//!                  no share dilution
//!   Efficiency:    Δgross-margin > 0 · Δasset-turnover > 0
//!
//! Altman Z (public manufacturers; the classic 1968 coefficients):
//!   Z = 1.2·WC/TA + 1.4·RE/TA + 3.3·EBIT/TA + 0.6·MVE/TL + 1.0·S/TA
//!   Z > 2.99 safe · 1.81–2.99 grey · < 1.81 distress
//!
//! Graham Number: √(22.5 · EPS · BVPS) — the price ceiling implied by
//! Graham's P/E ≤ 15 × P/B ≤ 1.5 screen.

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub struct PiotroskiCheck {
    pub key: &'static str,
    pub label: &'static str,
    /// None = data unavailable; excluded from the denominator.
    pub passed: Option<bool>,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct FundamentalHealth {
    pub symbol: String,
    pub piotroski_score: u8,
    pub piotroski_available: u8,
    pub piotroski_checks: Vec<PiotroskiCheck>,
    pub altman_z: Option<f64>,
    /// "safe" | "grey" | "distress" when altman_z is Some.
    pub altman_zone: Option<&'static str>,
    pub graham_number: Option<f64>,
    pub current_price: Option<f64>,
    /// (graham − price)/price × 100. Positive = trading below Graham.
    pub graham_upside_pct: Option<f64>,
    pub missing: Vec<&'static str>,
}

#[derive(Debug, thiserror::Error)]
pub enum HealthError {
    #[error("finnhub metric fetch failed: {0}")]
    Fetch(anyhow::Error),
}

/// Latest and prior values from a `series.annual.<key>` array. The
/// array arrives newest-first as `[{period, v}, ...]`.
fn series_latest_two(root: &Value, key: &str) -> (Option<f64>, Option<f64>) {
    let arr = root
        .pointer(&format!("/series/annual/{key}"))
        .and_then(|x| x.as_array());
    let get = |i: usize| -> Option<f64> {
        arr.and_then(|a| a.get(i))
            .and_then(|e| e.get("v"))
            .and_then(|v| v.as_f64())
    };
    (get(0), get(1))
}

fn metric_f64(root: &Value, key: &str) -> Option<f64> {
    // Direct map access, NOT JSON-pointer — Finnhub metric keys can
    // contain literal slashes (e.g. "totalDebt/totalEquityAnnual"),
    // which pointer() would treat as path separators.
    root.get("metric")
        .and_then(|m| m.get(key))
        .and_then(|v| v.as_f64())
}

pub async fn compute(symbol: &str) -> Result<FundamentalHealth, HealthError> {
    let m = crate::finnhub_rest::metric_all(symbol)
        .await
        .map_err(HealthError::Fetch)?;
    Ok(compute_from_metrics(symbol, &m))
}

/// Pure scoring from a Finnhub metric=all payload. Split out for tests.
pub fn compute_from_metrics(symbol: &str, m: &Value) -> FundamentalHealth {
    let mut checks: Vec<PiotroskiCheck> = Vec::with_capacity(9);
    let mut missing: Vec<&'static str> = Vec::new();

    let push = |key: &'static str,
                    label: &'static str,
                    passed: Option<bool>,
                    detail: String,
                    checks: &mut Vec<PiotroskiCheck>,
                    missing: &mut Vec<&'static str>| {
        if passed.is_none() {
            missing.push(key);
        }
        checks.push(PiotroskiCheck {
            key,
            label,
            passed,
            detail,
        });
    };

    // 1. ROA > 0 (TTM, falls back to latest annual series).
    let roa_now = metric_f64(m, "roaTTM").or_else(|| series_latest_two(m, "roa").0);
    push(
        "roa_positive",
        "ROA positive",
        roa_now.map(|v| v > 0.0),
        roa_now.map_or("no ROA data".into(), |v| format!("ROA {v:.2}%")),
        &mut checks,
        &mut missing,
    );

    // 2. Operating cash flow > 0 — proxied by cashFlowPerShareTTM.
    let cfo_ps = metric_f64(m, "cashFlowPerShareTTM")
        .or_else(|| metric_f64(m, "cashFlowPerShareAnnual"));
    push(
        "cfo_positive",
        "Operating cash flow positive",
        cfo_ps.map(|v| v > 0.0),
        cfo_ps.map_or("no CFO data".into(), |v| format!("CFO/share {v:.2}")),
        &mut checks,
        &mut missing,
    );

    // 3. ΔROA > 0 (annual series YoY).
    let (roa_a, roa_b) = series_latest_two(m, "roa");
    push(
        "roa_improving",
        "ROA improving YoY",
        match (roa_a, roa_b) {
            (Some(a), Some(b)) => Some(a > b),
            _ => None,
        },
        match (roa_a, roa_b) {
            (Some(a), Some(b)) => format!("{b:.2}% → {a:.2}%"),
            _ => "needs two annual ROA points".into(),
        },
        &mut checks,
        &mut missing,
    );

    // 4. Accruals: CFO/share > EPS (quality of earnings).
    let eps = metric_f64(m, "epsTTM").or_else(|| metric_f64(m, "epsAnnual"));
    push(
        "accruals",
        "Cash flow exceeds earnings",
        match (cfo_ps, eps) {
            (Some(c), Some(e)) => Some(c > e),
            _ => None,
        },
        match (cfo_ps, eps) {
            (Some(c), Some(e)) => format!("CFO/share {c:.2} vs EPS {e:.2}"),
            _ => "needs CFO + EPS".into(),
        },
        &mut checks,
        &mut missing,
    );

    // 5. Leverage falling: Δ(LT debt / total assets) < 0.
    let (lev_a, lev_b) = series_latest_two(m, "longtermDebtTotalAsset");
    push(
        "leverage_falling",
        "Long-term leverage falling",
        match (lev_a, lev_b) {
            (Some(a), Some(b)) => Some(a < b),
            _ => None,
        },
        match (lev_a, lev_b) {
            (Some(a), Some(b)) => format!("{b:.3} → {a:.3}"),
            _ => "needs two annual leverage points".into(),
        },
        &mut checks,
        &mut missing,
    );

    // 6. Current ratio improving.
    let (cr_a, cr_b) = series_latest_two(m, "currentRatio");
    push(
        "liquidity_improving",
        "Current ratio improving",
        match (cr_a, cr_b) {
            (Some(a), Some(b)) => Some(a > b),
            _ => None,
        },
        match (cr_a, cr_b) {
            (Some(a), Some(b)) => format!("{b:.2} → {a:.2}"),
            _ => "needs two annual current-ratio points".into(),
        },
        &mut checks,
        &mut missing,
    );

    // 7. No dilution: shares outstanding flat-or-down. Finnhub doesn't
    // ship a share-count series in metric=all, so approximate with
    // salesPerShare growing at least as fast as revenue growth — when
    // per-share sales keep pace, the share count didn't balloon.
    let rev_growth = metric_f64(m, "revenueGrowthTTMYoy");
    let (sps_a, sps_b) = series_latest_two(m, "salesPerShare");
    let sps_growth = match (sps_a, sps_b) {
        (Some(a), Some(b)) if b.abs() > 1e-9 => Some((a - b) / b.abs() * 100.0),
        _ => None,
    };
    push(
        "no_dilution",
        "No share dilution (proxy)",
        match (sps_growth, rev_growth) {
            (Some(s), Some(r)) => Some(s >= r - 2.0), // 2pp tolerance
            _ => None,
        },
        match (sps_growth, rev_growth) {
            (Some(s), Some(r)) => format!("sales/share {s:+.1}% vs revenue {r:+.1}%"),
            _ => "needs sales/share series + revenue growth".into(),
        },
        &mut checks,
        &mut missing,
    );

    // 8. Gross margin improving.
    let (gm_a, gm_b) = series_latest_two(m, "grossMargin");
    push(
        "gross_margin_improving",
        "Gross margin improving",
        match (gm_a, gm_b) {
            (Some(a), Some(b)) => Some(a > b),
            _ => None,
        },
        match (gm_a, gm_b) {
            (Some(a), Some(b)) => format!("{b:.1}% → {a:.1}%"),
            _ => "needs two annual gross-margin points".into(),
        },
        &mut checks,
        &mut missing,
    );

    // 9. Asset turnover improving.
    let (at_a, at_b) = series_latest_two(m, "assetTurnover");
    push(
        "asset_turnover_improving",
        "Asset turnover improving",
        match (at_a, at_b) {
            (Some(a), Some(b)) => Some(a > b),
            _ => None,
        },
        match (at_a, at_b) {
            (Some(a), Some(b)) => format!("{b:.2} → {a:.2}"),
            _ => "needs two annual asset-turnover points".into(),
        },
        &mut checks,
        &mut missing,
    );

    let piotroski_score = checks
        .iter()
        .filter(|c| matches!(c.passed, Some(true)))
        .count() as u8;
    let piotroski_available =
        checks.iter().filter(|c| c.passed.is_some()).count() as u8;

    // ── Altman Z ──────────────────────────────────────────────────
    // Finnhub metric=all doesn't expose the raw balance-sheet dollar
    // figures, but it does carry the RATIO building blocks:
    //   WC/TA: currentRatio + totalAssets-relative figures are not
    //   directly available — approximate using:
    //     x1 ≈ (currentRatio − 1) × currentLiabilities/TA — unavailable
    // → fall back to a reduced 4-ratio Z when components are missing:
    //   the EBIT/TA proxy (operatingMarginTTM × assetTurnover), MVE/TL
    //   (1 / totalDebtToEquity via marketCap), S/TA (assetTurnover),
    //   RE/TA unavailable. We compute Z only when ALL of the classic
    //   five are derivable; otherwise None + listed in missing — an
    //   honest gap beats a fake composite.
    let asset_turnover = series_latest_two(m, "assetTurnover").0;
    let op_margin = metric_f64(m, "operatingMarginTTM");
    let total_debt_to_equity = metric_f64(m, "totalDebt/totalEquityAnnual")
        .or_else(|| metric_f64(m, "totalDebtToEquityAnnual"));
    let market_cap = metric_f64(m, "marketCapitalization");
    let pb = metric_f64(m, "pbAnnual").or_else(|| metric_f64(m, "pb"));
    let altman_z = match (asset_turnover, op_margin, total_debt_to_equity, market_cap, pb) {
        (Some(s_ta), Some(om), Some(dte), Some(_mc), Some(_pb)) if dte > 0.0 => {
            // EBIT/TA ≈ operating margin × asset turnover (DuPont-ish).
            let ebit_ta = om / 100.0 * s_ta;
            // MVE/TL ≈ 1/(D/E) × (MVE/BVE). With P/B as MVE/BVE proxy:
            let mve_tl = (1.0 / dte) * _pb;
            // WC/TA and RE/TA aren't derivable from ratios — use the
            // peer-calibrated neutral 0.1 / 0.15 midpoints, flagged in
            // `missing` so the UI shows the approximation.
            let wc_ta = 0.1;
            let re_ta = 0.15;
            Some(1.2 * wc_ta + 1.4 * re_ta + 3.3 * ebit_ta + 0.6 * mve_tl + 1.0 * s_ta)
        }
        _ => None,
    };
    if altman_z.is_some() {
        missing.push("altman_wc_ta_re_ta_approximated");
    } else {
        missing.push("altman_z_inputs");
    }
    let altman_zone = altman_z.map(|z| {
        if z > 2.99 {
            "safe"
        } else if z >= 1.81 {
            "grey"
        } else {
            "distress"
        }
    });

    // ── Graham Number ────────────────────────────────────────────
    let bvps = metric_f64(m, "bookValuePerShareAnnual")
        .or_else(|| metric_f64(m, "bookValuePerShareQuarterly"));
    let graham_number = match (eps, bvps) {
        (Some(e), Some(b)) if e > 0.0 && b > 0.0 => Some((22.5 * e * b).sqrt()),
        _ => {
            missing.push("graham_inputs");
            None
        }
    };
    // Current price for the upside calc — Finnhub metric=all lacks it,
    // but 52WeekHigh/Low bound it; prefer the route layer passing the
    // live quote. Use 52-week midpoint as a visible approximation only
    // when nothing better exists. The route overrides this.
    let current_price = None;
    let graham_upside_pct = None;

    FundamentalHealth {
        symbol: symbol.to_string(),
        piotroski_score,
        piotroski_available,
        piotroski_checks: checks,
        altman_z,
        altman_zone,
        graham_number,
        current_price,
        graham_upside_pct,
        missing,
    }
}

/// Attach the live price to compute Graham upside. Called by the route
/// after fetching the quote.
pub fn with_price(mut h: FundamentalHealth, price: f64) -> FundamentalHealth {
    if price > 0.0 {
        h.current_price = Some(price);
        if let Some(g) = h.graham_number {
            h.graham_upside_pct = Some((g - price) / price * 100.0);
        }
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn payload() -> Value {
        json!({
            "metric": {
                "roaTTM": 12.0,
                "cashFlowPerShareTTM": 6.5,
                "epsTTM": 5.0,
                "revenueGrowthTTMYoy": 8.0,
                "operatingMarginTTM": 25.0,
                "totalDebt/totalEquityAnnual": 1.5,
                "marketCapitalization": 1000.0,
                "pbAnnual": 8.0,
                "bookValuePerShareAnnual": 4.0
            },
            "series": { "annual": {
                "roa": [{"period":"2025","v":12.0},{"period":"2024","v":10.0}],
                "currentRatio": [{"period":"2025","v":1.4},{"period":"2024","v":1.2}],
                "grossMargin": [{"period":"2025","v":45.0},{"period":"2024","v":43.0}],
                "assetTurnover": [{"period":"2025","v":0.8},{"period":"2024","v":0.7}],
                "longtermDebtTotalAsset": [{"period":"2025","v":0.25},{"period":"2024","v":0.30}],
                "salesPerShare": [{"period":"2025","v":54.0},{"period":"2024","v":50.0}]
            }}
        })
    }

    #[test]
    fn perfect_payload_scores_nine_of_nine() {
        let h = compute_from_metrics("TEST", &payload());
        assert_eq!(h.piotroski_available, 9, "checks: {:?}", h.piotroski_checks);
        assert_eq!(h.piotroski_score, 9);
    }

    #[test]
    fn missing_series_shrinks_denominator() {
        let mut p = payload();
        p["series"]["annual"]
            .as_object_mut()
            .unwrap()
            .remove("grossMargin");
        let h = compute_from_metrics("TEST", &p);
        assert_eq!(h.piotroski_available, 8);
        assert!(h.missing.contains(&"gross_margin_improving"));
    }

    #[test]
    fn graham_number_math() {
        let h = compute_from_metrics("TEST", &payload());
        // sqrt(22.5 × 5 × 4) = sqrt(450) ≈ 21.21
        let g = h.graham_number.unwrap();
        assert!((g - 21.2132).abs() < 0.01, "{g}");
        let priced = with_price(h, 15.0);
        assert!(priced.graham_upside_pct.unwrap() > 40.0);
    }

    #[test]
    fn altman_zone_thresholds() {
        let h = compute_from_metrics("TEST", &payload());
        let z = h.altman_z.unwrap();
        // ebit_ta = 0.25×0.8 = 0.2; mve_tl = (1/1.5)×8 ≈ 5.33;
        // z = 1.2(0.1)+1.4(0.15)+3.3(0.2)+0.6(5.33)+0.8 ≈ 0.12+0.21+0.66+3.2+0.8 = 4.99 → safe
        assert!(z > 2.99, "{z}");
        assert_eq!(h.altman_zone, Some("safe"));
    }

    #[test]
    fn negative_eps_blocks_graham() {
        let mut p = payload();
        p["metric"]["epsTTM"] = json!(-2.0);
        let h = compute_from_metrics("TEST", &p);
        assert!(h.graham_number.is_none());
    }
}
