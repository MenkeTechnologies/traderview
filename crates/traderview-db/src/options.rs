//! Yahoo options chain fetcher.
//!
//! Endpoint (no auth):
//!   https://query2.finance.yahoo.com/v7/finance/options/SYMBOL
//!   https://query2.finance.yahoo.com/v7/finance/options/SYMBOL?date=EPOCH
//! The first call returns `expirationDates: [..]` listing available expiries.
//! Subsequent calls with `?date=` return the full call+put grid for one expiry.

use chrono::NaiveDate;
use serde::Serialize;

const UA: &str =
    "Mozilla/5.0 (compatible; traderview/0.1; +https://github.com/MenkeTechnologies/traderview)";

#[derive(Debug, Clone, Serialize)]
pub struct OptionContract {
    pub strike: f64,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub last_price: Option<f64>,
    pub implied_vol: Option<f64>,
    pub volume: Option<i64>,
    pub open_interest: Option<i64>,
    pub in_the_money: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Chain {
    pub symbol: String,
    pub spot: f64,
    pub expirations: Vec<NaiveDate>,
    pub expiration: NaiveDate,
    pub calls: Vec<OptionContract>,
    pub puts: Vec<OptionContract>,
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(UA)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap()
}

pub async fn chain(symbol: &str, expiration: Option<NaiveDate>) -> anyhow::Result<Chain> {
    let mut url = format!(
        "https://query2.finance.yahoo.com/v7/finance/options/{sym}",
        sym = symbol,
    );
    if let Some(d) = expiration {
        let epoch = d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
        url.push_str(&format!("?date={}", epoch));
    }
    let resp = client().get(&url).send().await?;
    if !resp.status().is_success() {
        anyhow::bail!("options HTTP {}", resp.status());
    }
    let raw: OptionsResp = resp.json().await?;
    let result = raw.option_chain.result.and_then(|mut v| v.pop())
        .ok_or_else(|| anyhow::anyhow!("empty options result"))?;
    let spot = result.quote.regular_market_price.unwrap_or(0.0);
    let expirations: Vec<NaiveDate> = result
        .expiration_dates.unwrap_or_default()
        .into_iter()
        .filter_map(|ts| chrono::DateTime::from_timestamp(ts, 0).map(|d| d.date_naive()))
        .collect();
    let one = result.options.into_iter().next()
        .ok_or_else(|| anyhow::anyhow!("no option strip"))?;
    let exp = chrono::DateTime::from_timestamp(one.expiration_date.unwrap_or(0), 0)
        .map(|d| d.date_naive())
        .unwrap_or_else(|| chrono::Utc::now().date_naive());

    let map_contract = |c: RawContract| OptionContract {
        strike: c.strike.unwrap_or(0.0),
        bid: c.bid,
        ask: c.ask,
        last_price: c.last_price,
        implied_vol: c.implied_volatility,
        volume: c.volume,
        open_interest: c.open_interest,
        in_the_money: c.in_the_money.unwrap_or(false),
    };

    Ok(Chain {
        symbol: symbol.into(),
        spot,
        expirations,
        expiration: exp,
        calls: one.calls.into_iter().map(map_contract).collect(),
        puts: one.puts.into_iter().map(map_contract).collect(),
    })
}

/// Pick the ATM call+put for an expiration: the contracts whose strikes
/// bracket the spot price, then average the two.
pub fn atm_straddle(chain: &Chain) -> Option<((OptionContract, f64), (OptionContract, f64), f64)> {
    let call = nearest_atm(&chain.calls, chain.spot)?;
    let put  = nearest_atm(&chain.puts,  chain.spot)?;
    let cm = mid(&call)?;
    let pm = mid(&put)?;
    let atm = (call.strike + put.strike) / 2.0;
    Some(((call, cm), (put, pm), atm))
}

fn nearest_atm(side: &[OptionContract], spot: f64) -> Option<OptionContract> {
    side.iter()
        .min_by(|a, b| (a.strike - spot).abs().partial_cmp(&(b.strike - spot).abs())
            .unwrap_or(std::cmp::Ordering::Equal))
        .cloned()
}

fn mid(c: &OptionContract) -> Option<f64> {
    match (c.bid, c.ask) {
        (Some(b), Some(a)) if a > 0.0 && b >= 0.0 => Some((a + b) / 2.0),
        _ => c.last_price.filter(|x| *x > 0.0),
    }
}

// ---- raw Yahoo shapes ----

#[derive(serde::Deserialize)]
struct OptionsResp { #[serde(rename = "optionChain")] option_chain: OuterChain }
#[derive(serde::Deserialize)]
struct OuterChain { result: Option<Vec<ChainResult>> }
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChainResult {
    expiration_dates: Option<Vec<i64>>,
    quote: ChainQuote,
    options: Vec<OneStrip>,
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct ChainQuote {
    regular_market_price: Option<f64>,
}
#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct OneStrip {
    expiration_date: Option<i64>,
    #[serde(default)]
    calls: Vec<RawContract>,
    #[serde(default)]
    puts: Vec<RawContract>,
}
#[derive(serde::Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct RawContract {
    strike: Option<f64>,
    bid: Option<f64>,
    ask: Option<f64>,
    last_price: Option<f64>,
    implied_volatility: Option<f64>,
    volume: Option<i64>,
    open_interest: Option<i64>,
    in_the_money: Option<bool>,
}
