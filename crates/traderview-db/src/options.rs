//! Yahoo options chain fetcher.
//!
//! Endpoints (cookie+crumb auth via `yahoo_auth` — v7 rejects anonymous
//! calls with 401 "Invalid Crumb"):
//!   <https://query2.finance.yahoo.com/v7/finance/options/SYMBOL>
//!   <https://query2.finance.yahoo.com/v7/finance/options/SYMBOL?date=EPOCH>
//! The first call returns `expirationDates: [..]` listing available expiries.
//! Subsequent calls with `?date=` return the full call+put grid for one expiry.

use chrono::NaiveDate;
use serde::Serialize;

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

pub async fn chain(symbol: &str, expiration: Option<NaiveDate>) -> anyhow::Result<Chain> {
    // One retry after invalidating the cached crumb — Yahoo expires
    // crumbs server-side, surfacing as a 401 on a previously-good pair.
    let mut resp = None;
    for attempt in 0..2 {
        let auth = crate::yahoo_auth::get().await?;
        let url = format!("https://query2.finance.yahoo.com/v7/finance/options/{symbol}");
        let mut req = auth.client.get(&url).query(&[("crumb", auth.crumb.as_str())]);
        if let Some(d) = expiration {
            let epoch = d.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp();
            req = req.query(&[("date", epoch.to_string().as_str())]);
        }
        let r = req.send().await?;
        if r.status() == reqwest::StatusCode::UNAUTHORIZED && attempt == 0 {
            crate::yahoo_auth::invalidate(&auth.crumb).await;
            continue;
        }
        resp = Some(r);
        break;
    }
    let resp = resp.expect("loop always sets resp on its final iteration");
    if !resp.status().is_success() {
        anyhow::bail!("options HTTP {}", resp.status());
    }
    let raw: OptionsResp = resp.json().await?;
    let result = raw
        .option_chain
        .result
        .and_then(|mut v| v.pop())
        .ok_or_else(|| anyhow::anyhow!("empty options result"))?;
    let spot = result.quote.regular_market_price.unwrap_or(0.0);
    let expirations: Vec<NaiveDate> = result
        .expiration_dates
        .unwrap_or_default()
        .into_iter()
        .filter_map(|ts| chrono::DateTime::from_timestamp(ts, 0).map(|d| d.date_naive()))
        .collect();
    let one = result
        .options
        .into_iter()
        .next()
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

/// Result of an ATM straddle pick: (call, call_mid), (put, put_mid), atm_strike.
pub type AtmStraddle = ((OptionContract, f64), (OptionContract, f64), f64);

/// Pick the ATM call+put for an expiration: the contracts whose strikes
/// bracket the spot price, then average the two.
pub fn atm_straddle(chain: &Chain) -> Option<AtmStraddle> {
    let call = nearest_atm(&chain.calls, chain.spot)?;
    let put = nearest_atm(&chain.puts, chain.spot)?;
    let cm = mid(&call)?;
    let pm = mid(&put)?;
    let atm = (call.strike + put.strike) / 2.0;
    Some(((call, cm), (put, pm), atm))
}

fn nearest_atm(side: &[OptionContract], spot: f64) -> Option<OptionContract> {
    side.iter()
        .min_by(|a, b| {
            (a.strike - spot)
                .abs()
                .partial_cmp(&(b.strike - spot).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
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
struct OptionsResp {
    #[serde(rename = "optionChain")]
    option_chain: OuterChain,
}
#[derive(serde::Deserialize)]
struct OuterChain {
    result: Option<Vec<ChainResult>>,
}
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

#[cfg(test)]
mod tests {
    use super::*;

    fn c(strike: f64, bid: Option<f64>, ask: Option<f64>) -> OptionContract {
        OptionContract {
            strike,
            bid,
            ask,
            last_price: None,
            implied_vol: None,
            volume: None,
            open_interest: None,
            in_the_money: false,
        }
    }

    // ─── mid() ────────────────────────────────────────────────────────────

    #[test]
    fn mid_averages_bid_and_ask() {
        // bid 1.00, ask 1.20 → mid 1.10
        let m = mid(&c(100.0, Some(1.00), Some(1.20)));
        assert_eq!(m, Some(1.10));
    }

    #[test]
    fn mid_falls_back_to_last_price_when_quote_missing() {
        // No bid/ask → use last_price.
        let mut ct = c(100.0, None, None);
        ct.last_price = Some(2.50);
        assert_eq!(mid(&ct), Some(2.50));
    }

    #[test]
    fn mid_falls_back_to_last_when_ask_is_zero() {
        // Stale or invalid ask (0) → don't compute (0+bid)/2; use last instead.
        let mut ct = c(100.0, Some(1.00), Some(0.0));
        ct.last_price = Some(1.50);
        assert_eq!(mid(&ct), Some(1.50));
    }

    #[test]
    fn mid_returns_none_when_no_data() {
        let ct = c(100.0, None, None);
        assert_eq!(mid(&ct), None);
    }

    #[test]
    fn mid_returns_none_when_last_price_is_zero_and_no_quote() {
        let mut ct = c(100.0, None, None);
        ct.last_price = Some(0.0);
        assert_eq!(
            mid(&ct),
            None,
            "zero last_price with no quotes shouldn't be treated as a real fill"
        );
    }

    // ─── nearest_atm() ────────────────────────────────────────────────────

    #[test]
    fn nearest_atm_picks_strike_closest_to_spot() {
        let strikes = vec![
            c(95.0, Some(5.0), Some(5.2)),
            c(100.0, Some(2.0), Some(2.2)),
            c(105.0, Some(0.5), Some(0.7)),
        ];
        // Spot 101 → 100 is closest (1 vs 4 vs 4).
        let n = nearest_atm(&strikes, 101.0).expect("non-empty");
        assert_eq!(n.strike, 100.0);
    }

    #[test]
    fn nearest_atm_picks_lower_strike_when_equidistant() {
        let strikes = vec![
            c(95.0, Some(5.0), Some(5.2)),
            c(105.0, Some(5.0), Some(5.2)),
        ];
        // Spot 100 — equidistant from 95 and 105. min_by returns first hit.
        let n = nearest_atm(&strikes, 100.0).expect("non-empty");
        assert_eq!(n.strike, 95.0);
    }

    #[test]
    fn nearest_atm_returns_none_for_empty_chain() {
        let n = nearest_atm(&[], 100.0);
        assert!(n.is_none());
    }

    // ─── atm_straddle() ───────────────────────────────────────────────────

    #[test]
    fn atm_straddle_combines_call_and_put() {
        let chain = Chain {
            symbol: "TEST".into(),
            spot: 100.0,
            expiration: chrono::NaiveDate::from_ymd_opt(2026, 6, 18).unwrap(),
            expirations: vec![],
            calls: vec![
                c(95.0, Some(6.0), Some(6.2)),
                c(100.0, Some(3.0), Some(3.2)),
                c(105.0, Some(1.0), Some(1.2)),
            ],
            puts: vec![
                c(95.0, Some(1.0), Some(1.2)),
                c(100.0, Some(3.0), Some(3.2)),
                c(105.0, Some(6.0), Some(6.2)),
            ],
        };
        let ((call, cm), (put, pm), atm) = atm_straddle(&chain).expect("straddle");
        assert_eq!(call.strike, 100.0);
        assert_eq!(put.strike, 100.0);
        assert_eq!(cm, 3.10); // (3.00 + 3.20) / 2
        assert_eq!(pm, 3.10);
        assert_eq!(atm, 100.0, "ATM strike = avg of call+put strikes");
    }

    #[test]
    fn atm_straddle_returns_none_when_either_side_empty() {
        let chain = Chain {
            symbol: "TEST".into(),
            spot: 100.0,
            expiration: chrono::NaiveDate::from_ymd_opt(2026, 6, 18).unwrap(),
            expirations: vec![],
            calls: vec![c(100.0, Some(3.0), Some(3.2))],
            puts: vec![], // no puts
        };
        assert!(
            atm_straddle(&chain).is_none(),
            "straddle requires both call and put"
        );
    }
}
