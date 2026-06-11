//! Beneish M-Score — earnings-manipulation probability from two
//! consecutive ANNUAL reports (Finnhub `financials_reported`).
//!
//! ```text
//! M = −4.84 + 0.920·DSRI + 0.528·GMI + 0.404·AQI + 0.892·SGI
//!     + 0.115·DEPI − 0.172·SGAI + 4.679·TATA − 0.327·LVGI
//! M > −1.78 → flagged as a likely manipulator (Beneish 1999 cutoff)
//! ```
//!
//! Filer concept tags vary, so each line item is located by searching
//! a list of common us-gaap tags. Missing items surface in `missing`
//! and their index defaults to the neutral 1.0 (0.0 for TATA) — the
//! score still computes but the UI shows exactly which inputs were
//! approximated.

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub struct BeneishIndex {
    pub key: &'static str,
    pub value: f64,
    pub approximated: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BeneishReport {
    pub symbol: String,
    pub m_score: f64,
    /// True when M > −1.78.
    pub likely_manipulator: bool,
    pub year_current: i64,
    pub year_prior: i64,
    pub indexes: Vec<BeneishIndex>,
    pub missing: Vec<&'static str>,
}

#[derive(Debug, thiserror::Error)]
pub enum BeneishError {
    #[error("financials fetch failed: {0}")]
    Fetch(anyhow::Error),
    #[error("need two annual reports for {symbol}; found {found}")]
    NotEnoughReports { symbol: String, found: usize },
}

/// Search every statement section of a report for the first concept
/// whose tag contains any of `needles` (case-insensitive). Shared with
/// deep_value, which screens the same financials_reported payloads.
pub(crate) fn find_concept(report: &Value, needles: &[&str]) -> Option<f64> {
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
            if needles.iter().any(|n| concept.contains(&n.to_ascii_lowercase())) {
                if let Some(v) = item.get("value").and_then(|v| v.as_f64()) {
                    return Some(v);
                }
                // Some filers ship values as strings.
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
    None
}

/// All Beneish line items for one annual report.
#[derive(Debug, Default, Clone, Copy)]
struct Items {
    sales: Option<f64>,
    cogs: Option<f64>,
    receivables: Option<f64>,
    current_assets: Option<f64>,
    ppe: Option<f64>,
    total_assets: Option<f64>,
    depreciation: Option<f64>,
    sga: Option<f64>,
    long_term_debt: Option<f64>,
    current_liabilities: Option<f64>,
    income_continuing: Option<f64>,
    cfo: Option<f64>,
}

fn extract(report: &Value) -> Items {
    Items {
        sales: find_concept(report, &["revenuefromcontract", "revenues", "salesrevenuenet"]),
        cogs: find_concept(report, &["costofgoodssold", "costofrevenue", "costofsales"]),
        receivables: find_concept(report, &["accountsreceivablenet", "receivablesnetcurrent"]),
        current_assets: find_concept(report, &["assetscurrent"]),
        ppe: find_concept(report, &["propertyplantandequipmentnet"]),
        total_assets: find_concept(report, &["assets\u{0}", "assets"]),
        depreciation: find_concept(report, &["depreciationdepletionandamortization", "depreciationandamortization", "depreciation"]),
        sga: find_concept(report, &["sellinggeneralandadministrative"]),
        long_term_debt: find_concept(report, &["longtermdebtnoncurrent", "longtermdebt"]),
        current_liabilities: find_concept(report, &["liabilitiescurrent"]),
        income_continuing: find_concept(report, &["incomelossfromcontinuingoperations", "netincomeloss"]),
        cfo: find_concept(report, &["netcashprovidedbyusedinoperatingactivities"]),
    }
}

pub async fn compute(symbol: &str) -> Result<BeneishReport, BeneishError> {
    let raw = crate::finnhub_rest::financials_reported(symbol)
        .await
        .map_err(BeneishError::Fetch)?;
    compute_from_reports(symbol, &raw)
}

/// Pure scoring from a financials_reported payload. Split for tests.
pub fn compute_from_reports(symbol: &str, raw: &Value) -> Result<BeneishReport, BeneishError> {
    // Keep ANNUAL reports (quarter == 0), newest first.
    let annuals: Vec<&Value> = raw
        .get("data")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter(|r| r.get("quarter").and_then(|q| q.as_i64()) == Some(0))
                .collect()
        })
        .unwrap_or_default();
    if annuals.len() < 2 {
        return Err(BeneishError::NotEnoughReports {
            symbol: symbol.to_string(),
            found: annuals.len(),
        });
    }
    let year = |r: &Value| r.get("year").and_then(|y| y.as_i64()).unwrap_or(0);
    let cur = extract(annuals[0]);
    let pri = extract(annuals[1]);
    let mut missing: Vec<&'static str> = Vec::new();

    // Ratio helper: a/b when both present and b != 0.
    let ratio = |a: Option<f64>, b: Option<f64>| -> Option<f64> {
        match (a, b) {
            (Some(x), Some(y)) if y.abs() > 1e-9 => Some(x / y),
            _ => None,
        }
    };
    // Index helper: cur_ratio / prior_ratio.
    let index = |c: Option<f64>, p: Option<f64>| -> Option<f64> {
        match (c, p) {
            (Some(x), Some(y)) if y.abs() > 1e-9 => Some(x / y),
            _ => None,
        }
    };

    // DSRI
    let dsri = index(
        ratio(cur.receivables, cur.sales),
        ratio(pri.receivables, pri.sales),
    );
    // GMI = GM_prior / GM_cur.
    let gm = |i: &Items| -> Option<f64> {
        match (i.sales, i.cogs) {
            (Some(s), Some(c)) if s.abs() > 1e-9 => Some((s - c) / s),
            _ => None,
        }
    };
    let gmi = index(gm(&pri), gm(&cur));
    // AQI = (1 − (CA+PPE)/TA) ratio.
    let aq = |i: &Items| -> Option<f64> {
        match (i.current_assets, i.ppe, i.total_assets) {
            (Some(ca), Some(pp), Some(ta)) if ta.abs() > 1e-9 => Some(1.0 - (ca + pp) / ta),
            _ => None,
        }
    };
    let aqi = index(aq(&cur), aq(&pri));
    // SGI
    let sgi = index(cur.sales, pri.sales);
    // DEPI = dep-rate_prior / dep-rate_cur.
    let dep_rate = |i: &Items| -> Option<f64> {
        match (i.depreciation, i.ppe) {
            (Some(d), Some(pp)) if (d + pp).abs() > 1e-9 => Some(d / (d + pp)),
            _ => None,
        }
    };
    let depi = index(dep_rate(&pri), dep_rate(&cur));
    // SGAI
    let sgai = index(ratio(cur.sga, cur.sales), ratio(pri.sga, pri.sales));
    // LVGI = leverage_cur / leverage_prior.
    let lev = |i: &Items| -> Option<f64> {
        match (i.long_term_debt, i.current_liabilities, i.total_assets) {
            (Some(l), Some(c), Some(ta)) if ta.abs() > 1e-9 => Some((l + c) / ta),
            _ => None,
        }
    };
    let lvgi = index(lev(&cur), lev(&pri));
    // TATA = (income − CFO) / TA, current year only.
    let tata = match (cur.income_continuing, cur.cfo, cur.total_assets) {
        (Some(inc), Some(cf), Some(ta)) if ta.abs() > 1e-9 => Some((inc - cf) / ta),
        _ => None,
    };

    let mut idx = |key: &'static str, v: Option<f64>, neutral: f64| -> f64 {
        match v {
            Some(x) if x.is_finite() => {
                // Clamp pathological ratios so one bad parse can't blow
                // up the whole score.
                x.clamp(-10.0, 10.0)
            }
            _ => {
                missing.push(key);
                neutral
            }
        }
    };
    let dsri_v = idx("dsri", dsri, 1.0);
    let gmi_v = idx("gmi", gmi, 1.0);
    let aqi_v = idx("aqi", aqi, 1.0);
    let sgi_v = idx("sgi", sgi, 1.0);
    let depi_v = idx("depi", depi, 1.0);
    let sgai_v = idx("sgai", sgai, 1.0);
    let lvgi_v = idx("lvgi", lvgi, 1.0);
    let tata_v = idx("tata", tata, 0.0);

    let m_score = -4.84 + 0.920 * dsri_v + 0.528 * gmi_v + 0.404 * aqi_v + 0.892 * sgi_v
        + 0.115 * depi_v
        - 0.172 * sgai_v
        + 4.679 * tata_v
        - 0.327 * lvgi_v;

    let mk = |key: &'static str, value: f64| BeneishIndex {
        key,
        value,
        approximated: missing.contains(&key),
    };
    Ok(BeneishReport {
        symbol: symbol.to_string(),
        m_score,
        likely_manipulator: m_score > -1.78,
        year_current: year(annuals[0]),
        year_prior: year(annuals[1]),
        indexes: vec![
            mk("dsri", dsri_v),
            mk("gmi", gmi_v),
            mk("aqi", aqi_v),
            mk("sgi", sgi_v),
            mk("depi", depi_v),
            mk("sgai", sgai_v),
            mk("lvgi", lvgi_v),
            mk("tata", tata_v),
        ],
        missing,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn report(year: i64, sales: f64, recv: f64, cfo: f64, income: f64) -> Value {
        json!({
            "year": year,
            "quarter": 0,
            "report": {
                "ic": [
                    {"concept": "us-gaap_Revenues", "value": sales},
                    {"concept": "us-gaap_CostOfRevenue", "value": sales * 0.6},
                    {"concept": "us-gaap_SellingGeneralAndAdministrativeExpense", "value": sales * 0.2},
                    {"concept": "us-gaap_NetIncomeLoss", "value": income},
                ],
                "bs": [
                    {"concept": "us-gaap_AccountsReceivableNetCurrent", "value": recv},
                    {"concept": "us-gaap_AssetsCurrent", "value": 500.0},
                    {"concept": "us-gaap_PropertyPlantAndEquipmentNet", "value": 300.0},
                    {"concept": "us-gaap_Assets", "value": 1000.0},
                    {"concept": "us-gaap_LongTermDebtNoncurrent", "value": 200.0},
                    {"concept": "us-gaap_LiabilitiesCurrent", "value": 150.0},
                ],
                "cf": [
                    {"concept": "us-gaap_DepreciationDepletionAndAmortization", "value": 50.0},
                    {"concept": "us-gaap_NetCashProvidedByUsedInOperatingActivities", "value": cfo},
                ]
            }
        })
    }

    #[test]
    fn steady_company_scores_below_cutoff() {
        // Identical YoY → every index ≈ 1, TATA slightly negative
        // (income 90 < CFO 120 → TATA = −0.03). Sum of ratio terms at
        // 1.0 ≈ −2.477; plus 4.679×(−0.03) ≈ −2.62 → well below −1.78.
        let raw = json!({ "data": [
            report(2025, 1000.0, 100.0, 120.0, 90.0),
            report(2024, 1000.0, 100.0, 120.0, 90.0),
        ]});
        let r = compute_from_reports("STEADY", &raw).unwrap();
        assert!(r.missing.is_empty(), "missing: {:?}", r.missing);
        assert!(!r.likely_manipulator, "m_score {}", r.m_score);
    }

    #[test]
    fn receivables_spike_plus_accruals_flags_manipulator() {
        // Receivables triple while sales flat (DSRI=3) AND income far
        // above CFO (big positive accruals) → M crosses −1.78.
        let raw = json!({ "data": [
            report(2025, 1000.0, 300.0, 20.0, 400.0),
            report(2024, 1000.0, 100.0, 120.0, 90.0),
        ]});
        let r = compute_from_reports("SUS", &raw).unwrap();
        assert!(r.likely_manipulator, "m_score {}", r.m_score);
    }

    #[test]
    fn single_report_errors() {
        let raw = json!({ "data": [ report(2025, 1000.0, 100.0, 120.0, 90.0) ]});
        assert!(matches!(
            compute_from_reports("ONE", &raw),
            Err(BeneishError::NotEnoughReports { found: 1, .. })
        ));
    }

    #[test]
    fn missing_concept_reports_and_neutralizes() {
        // Strip SGA from the current year → sgai missing, defaults 1.0.
        let mut cur = report(2025, 1000.0, 100.0, 120.0, 90.0);
        let ic = cur["report"]["ic"].as_array_mut().unwrap();
        ic.retain(|i| !i["concept"].as_str().unwrap().contains("SellingGeneral"));
        let raw = json!({ "data": [ cur, report(2024, 1000.0, 100.0, 120.0, 90.0) ]});
        let r = compute_from_reports("GAPPY", &raw).unwrap();
        assert!(r.missing.contains(&"sgai"));
        let sgai = r.indexes.iter().find(|i| i.key == "sgai").unwrap();
        assert!(sgai.approximated);
        assert!((sgai.value - 1.0).abs() < 1e-9);
    }
}
