//! 13F holdings diff — what a fund bought and sold between its last
//! two 13F-HR filings.
//!
//! Pipeline:
//!   1. `data.sec.gov/submissions/CIK##########.json` → the two most
//!      recent 13F-HR accessions.
//!   2. Each filing's index page → the infotable XML (href containing
//!      "infotable", the EDGAR convention).
//!   3. quick-xml parse of <infoTable> entries, aggregated per
//!      (cusip, put/call) — funds file multiple manager rows per issuer.
//!   4. Diff: new / exited / increased / decreased by SHARES.
//!
//! `value` is reported AS FILED, units untouched — EDGAR switched from
//! thousands to dollars in 2022 and older filings differ; shares are
//! the unambiguous diff axis. Same EDGAR client conventions as
//! `insider_stream` (Form 4).

use quick_xml::events::Event;
use quick_xml::Reader;
use serde::Serialize;
use std::collections::HashMap;
use std::time::Duration;

const UA: &str = "traderview admin@menketechnologies.com";

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Holding {
    pub issuer: String,
    pub cusip: String,
    /// "SH", "PRN", or with an option marker "Put"/"Call" appended.
    pub class: String,
    pub shares: f64,
    /// As filed — units vary by filing era.
    pub value_as_filed: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct DiffRow {
    pub issuer: String,
    pub cusip: String,
    pub class: String,
    pub shares_prior: f64,
    pub shares_latest: f64,
    pub shares_delta: f64,
    pub pct_change: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ThirteenFDiff {
    pub cik: String,
    pub latest_filed: String,
    pub prior_filed: String,
    pub latest_positions: usize,
    pub prior_positions: usize,
    pub new_positions: Vec<DiffRow>,
    pub exited: Vec<DiffRow>,
    pub increased: Vec<DiffRow>,
    pub decreased: Vec<DiffRow>,
    pub unchanged: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum ThirteenFError {
    #[error("EDGAR fetch failed: {0}")]
    Fetch(String),
    #[error("need two 13F-HR filings for CIK {cik}; found {found}")]
    NotEnoughFilings { cik: String, found: usize },
    #[error("infotable XML not found in filing {0}")]
    NoInfotable(String),
}

/// Parse a 13F infotable XML into aggregated holdings. Pure — fixture
/// tested.
pub fn parse_infotable(xml: &str) -> Vec<Holding> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut agg: HashMap<(String, String), Holding> = HashMap::new();
    let mut cur: Option<Holding> = None;
    let mut field: Option<String> = None;
    let mut put_call = String::new();
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                match name.as_str() {
                    "infoTable" => {
                        cur = Some(Holding {
                            issuer: String::new(),
                            cusip: String::new(),
                            class: String::new(),
                            shares: 0.0,
                            value_as_filed: 0.0,
                        });
                        put_call.clear();
                    }
                    "nameOfIssuer" | "cusip" | "value" | "sshPrnamt" | "sshPrnamtType"
                    | "putCall" => field = Some(name),
                    _ => {}
                }
            }
            Ok(Event::Text(t)) => {
                if let (Some(h), Some(f)) = (cur.as_mut(), field.as_deref()) {
                    let text = t.unescape().unwrap_or_default().trim().to_string();
                    match f {
                        "nameOfIssuer" => h.issuer = text,
                        "cusip" => h.cusip = text,
                        "value" => h.value_as_filed = text.replace(',', "").parse().unwrap_or(0.0),
                        "sshPrnamt" => h.shares = text.replace(',', "").parse().unwrap_or(0.0),
                        "sshPrnamtType" => h.class = text,
                        "putCall" => put_call = text,
                        _ => {}
                    }
                }
            }
            Ok(Event::End(e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                if name == "infoTable" {
                    if let Some(mut h) = cur.take() {
                        if !put_call.is_empty() {
                            h.class = format!("{} {}", h.class, put_call);
                        }
                        if !h.cusip.is_empty() {
                            let key = (h.cusip.clone(), h.class.clone());
                            agg.entry(key)
                                .and_modify(|e| {
                                    e.shares += h.shares;
                                    e.value_as_filed += h.value_as_filed;
                                })
                                .or_insert(h);
                        }
                    }
                } else if field.as_deref() == Some(name.as_str()) {
                    field = None;
                }
            }
            Ok(Event::Eof) => break,
            Err(_) => break,
            _ => {}
        }
        buf.clear();
    }
    let mut out: Vec<Holding> = agg.into_values().collect();
    out.sort_by(|a, b| {
        b.value_as_filed
            .partial_cmp(&a.value_as_filed)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    out
}

/// Diff two holding sets by (cusip, class). Pure — fixture tested.
pub fn diff_holdings(prior: &[Holding], latest: &[Holding]) -> (Vec<DiffRow>, Vec<DiffRow>, Vec<DiffRow>, Vec<DiffRow>, usize) {
    let key = |h: &Holding| (h.cusip.clone(), h.class.clone());
    let prior_map: HashMap<_, &Holding> = prior.iter().map(|h| (key(h), h)).collect();
    let latest_map: HashMap<_, &Holding> = latest.iter().map(|h| (key(h), h)).collect();
    let mut new_positions = Vec::new();
    let mut exited = Vec::new();
    let mut increased = Vec::new();
    let mut decreased = Vec::new();
    let mut unchanged = 0usize;
    let row = |h: &Holding, sp: f64, sl: f64| DiffRow {
        issuer: h.issuer.clone(),
        cusip: h.cusip.clone(),
        class: h.class.clone(),
        shares_prior: sp,
        shares_latest: sl,
        shares_delta: sl - sp,
        pct_change: (sp > 0.0).then(|| (sl / sp - 1.0) * 100.0),
    };
    for h in latest {
        match prior_map.get(&key(h)) {
            None => new_positions.push(row(h, 0.0, h.shares)),
            Some(p) if h.shares > p.shares => increased.push(row(h, p.shares, h.shares)),
            Some(p) if h.shares < p.shares => decreased.push(row(h, p.shares, h.shares)),
            Some(_) => unchanged += 1,
        }
    }
    for p in prior {
        if !latest_map.contains_key(&key(p)) {
            exited.push(row(p, p.shares, 0.0));
        }
    }
    let by_abs = |v: &mut Vec<DiffRow>| {
        v.sort_by(|a, b| {
            b.shares_delta
                .abs()
                .partial_cmp(&a.shares_delta.abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    };
    by_abs(&mut new_positions);
    by_abs(&mut exited);
    by_abs(&mut increased);
    by_abs(&mut decreased);
    (new_positions, exited, increased, decreased, unchanged)
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(UA)
        .timeout(Duration::from_secs(15))
        .build()
        .expect("reqwest client")
}

async fn fetch_text(url: &str) -> Result<String, ThirteenFError> {
    let resp = client()
        .get(url)
        .send()
        .await
        .map_err(|e| ThirteenFError::Fetch(e.to_string()))?;
    if !resp.status().is_success() {
        return Err(ThirteenFError::Fetch(format!("{} on {url}", resp.status())));
    }
    resp.text().await.map_err(|e| ThirteenFError::Fetch(e.to_string()))
}

/// Latest two 13F-HR (accession, filing_date) pairs for a CIK.
async fn latest_two_13f(cik: &str) -> Result<Vec<(String, String)>, ThirteenFError> {
    let padded = format!("{:0>10}", cik);
    let url = format!("https://data.sec.gov/submissions/CIK{padded}.json");
    let body = fetch_text(&url).await?;
    let v: serde_json::Value =
        serde_json::from_str(&body).map_err(|e| ThirteenFError::Fetch(e.to_string()))?;
    let recent = &v["filings"]["recent"];
    let forms = recent["form"].as_array().cloned().unwrap_or_default();
    let accessions = recent["accessionNumber"].as_array().cloned().unwrap_or_default();
    let dates = recent["filingDate"].as_array().cloned().unwrap_or_default();
    let mut out = Vec::new();
    for i in 0..forms.len().min(accessions.len()).min(dates.len()) {
        if forms[i].as_str() == Some("13F-HR") {
            if let (Some(a), Some(d)) = (accessions[i].as_str(), dates[i].as_str()) {
                out.push((a.to_string(), d.to_string()));
                if out.len() == 2 {
                    break;
                }
            }
        }
    }
    if out.len() < 2 {
        return Err(ThirteenFError::NotEnoughFilings {
            cik: cik.to_string(),
            found: out.len(),
        });
    }
    Ok(out)
}

async fn fetch_holdings(cik: &str, accession: &str) -> Result<Vec<Holding>, ThirteenFError> {
    let no_dash = accession.replace('-', "");
    let cik_trim = cik.trim_start_matches('0');
    let index_url =
        format!("https://www.sec.gov/Archives/edgar/data/{cik_trim}/{no_dash}/");
    let body = fetch_text(&index_url).await?;
    // Filename conventions vary by filer ("infotable.xml", bare
    // numeric names…) — collect every XML href except the cover page
    // (primary_doc), prefer infotable-named ones, and accept the first
    // candidate whose CONTENT parses to holdings.
    let mut candidates: Vec<String> = Vec::new();
    let lower = body.to_ascii_lowercase();
    let mut from = 0usize;
    while let Some(pos) = lower[from..].find("href=\"") {
        let start = from + pos + 6;
        let Some(end) = body[start..].find('"') else { break };
        let h = &body[start..start + end];
        let h_lc = h.to_ascii_lowercase();
        if h_lc.ends_with(".xml") && !h_lc.contains("primary_doc") {
            let url = crate::insider_stream::absolutize_href(&index_url, h);
            if !candidates.contains(&url) {
                if h_lc.contains("infotable") {
                    candidates.insert(0, url);
                } else {
                    candidates.push(url);
                }
            }
        }
        from = start + end + 1;
    }
    for url in candidates {
        if let Ok(xml) = fetch_text(&url).await {
            let holdings = parse_infotable(&xml);
            if !holdings.is_empty() {
                return Ok(holdings);
            }
        }
    }
    Err(ThirteenFError::NoInfotable(accession.to_string()))
}

pub async fn holdings_diff(cik: &str) -> Result<ThirteenFDiff, ThirteenFError> {
    let filings = latest_two_13f(cik).await?;
    let latest = fetch_holdings(cik, &filings[0].0).await?;
    let prior = fetch_holdings(cik, &filings[1].0).await?;
    let (new_positions, exited, increased, decreased, unchanged) =
        diff_holdings(&prior, &latest);
    Ok(ThirteenFDiff {
        cik: cik.to_string(),
        latest_filed: filings[0].1.clone(),
        prior_filed: filings[1].1.clone(),
        latest_positions: latest.len(),
        prior_positions: prior.len(),
        new_positions,
        exited,
        increased,
        decreased,
        unchanged,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"<?xml version="1.0"?>
<informationTable xmlns="http://www.sec.gov/edgar/document/thirteenf/informationtable">
  <infoTable>
    <nameOfIssuer>APPLE INC</nameOfIssuer>
    <cusip>037833100</cusip>
    <value>915560382</value>
    <shrsOrPrnAmt><sshPrnamt>915560382</sshPrnamt><sshPrnamtType>SH</sshPrnamtType></shrsOrPrnAmt>
  </infoTable>
  <infoTable>
    <nameOfIssuer>APPLE INC</nameOfIssuer>
    <cusip>037833100</cusip>
    <value>100</value>
    <shrsOrPrnAmt><sshPrnamt>50</sshPrnamt><sshPrnamtType>SH</sshPrnamtType></shrsOrPrnAmt>
  </infoTable>
  <infoTable>
    <nameOfIssuer>OCCIDENTAL PETE</nameOfIssuer>
    <cusip>674599105</cusip>
    <value>13000</value>
    <shrsOrPrnAmt><sshPrnamt>1000</sshPrnamt><sshPrnamtType>SH</sshPrnamtType></shrsOrPrnAmt>
    <putCall>Call</putCall>
  </infoTable>
</informationTable>"#;

    #[test]
    fn parse_aggregates_same_issuer_rows_and_tags_options() {
        let h = parse_infotable(FIXTURE);
        assert_eq!(h.len(), 2);
        let aapl = h.iter().find(|x| x.cusip == "037833100").expect("AAPL");
        // Two manager rows summed.
        assert!((aapl.shares - 915_560_432.0).abs() < 1e-3);
        assert!((aapl.value_as_filed - 915_560_482.0).abs() < 1e-3);
        assert_eq!(aapl.class, "SH");
        let oxy = h.iter().find(|x| x.cusip == "674599105").expect("OXY");
        assert_eq!(oxy.class, "SH Call");
        assert_eq!(oxy.issuer, "OCCIDENTAL PETE");
    }

    #[test]
    fn diff_classifies_all_four_buckets() {
        let mk = |cusip: &str, shares: f64| Holding {
            issuer: cusip.to_string(),
            cusip: cusip.to_string(),
            class: "SH".into(),
            shares,
            value_as_filed: 0.0,
        };
        let prior = vec![mk("AAA", 100.0), mk("BBB", 100.0), mk("CCC", 100.0), mk("DDD", 100.0)];
        let latest = vec![mk("AAA", 100.0), mk("BBB", 150.0), mk("CCC", 60.0), mk("EEE", 40.0)];
        let (newp, exited, inc, dec, unchanged) = diff_holdings(&prior, &latest);
        assert_eq!(unchanged, 1); // AAA
        assert_eq!(newp.len(), 1);
        assert_eq!(newp[0].cusip, "EEE");
        assert_eq!(newp[0].pct_change, None); // from zero — no fake %
        assert_eq!(exited.len(), 1);
        assert_eq!(exited[0].cusip, "DDD");
        assert!((exited[0].shares_delta + 100.0).abs() < 1e-12);
        assert_eq!(inc[0].cusip, "BBB");
        assert!((inc[0].pct_change.unwrap() - 50.0).abs() < 1e-12);
        assert_eq!(dec[0].cusip, "CCC");
        assert!((dec[0].pct_change.unwrap() + 40.0).abs() < 1e-12);
    }

    #[test]
    fn same_cusip_different_class_are_distinct_positions() {
        let sh = Holding {
            issuer: "X".into(),
            cusip: "111".into(),
            class: "SH".into(),
            shares: 100.0,
            value_as_filed: 0.0,
        };
        let call = Holding {
            class: "SH Call".into(),
            ..sh.clone()
        };
        let (newp, exited, _, _, unchanged) = diff_holdings(&[sh], &[call]);
        // The share line exited, the call line is new — not netted.
        assert_eq!(newp.len(), 1);
        assert_eq!(exited.len(), 1);
        assert_eq!(unchanged, 0);
    }

    #[test]
    fn malformed_xml_yields_empty_not_panic() {
        assert!(parse_infotable("not xml at all").is_empty());
        assert!(parse_infotable("<infoTable><cusip>abc").is_empty());
    }
}
