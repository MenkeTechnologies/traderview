//! Implied-volatility surface.
//!
//! Builds an IV grid for a symbol across N nearest expirations × a fixed set
//! of moneyness buckets (`-15%, -10%, -5%, -2%, ATM, +2%, +5%, +10%, +15%`).
//!
//! For each cell we:
//!   1. Pull both the call and put chain for the expiration via `options::chain`.
//!   2. Interpolate linearly between the two nearest strikes' `impliedVolatility`.
//!   3. If Yahoo didn't return IV, solve Black–Scholes Newton–Raphson on the
//!      mid-quote using `traderview_core::greeks::implied_vol`.
//!
//! Companion slices:
//!   * `term_structure`  — ATM IV vs days-to-expiry
//!   * `front_skew`      — IV vs moneyness for the front month
//!
//! All IVs are returned as decimal (0.32 = 32%).

use chrono::{NaiveDate, Utc};
use serde::Serialize;
use traderview_core::greeks::{implied_vol, OptKind};

use crate::options::{self, OptionContract};

const MONEYNESS: &[f64] = &[-0.15, -0.10, -0.05, -0.02, 0.0, 0.02, 0.05, 0.10, 0.15];
const RISK_FREE: f64 = 0.045; // approximate; surface shape is insensitive.
const DIVIDEND: f64 = 0.0;

#[derive(Debug, Clone, Serialize)]
pub struct VolSurface {
    pub symbol: String,
    pub spot: f64,
    pub moneyness: Vec<f64>, // x-axis labels (decimal fractions of spot)
    pub expirations: Vec<ExpirationRow>,
    pub term_structure: Vec<TermPoint>, // ATM IV per expiration
    pub front_skew: Vec<SkewPoint>,     // front-month: IV by moneyness
    pub fetched_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExpirationRow {
    pub expiration: NaiveDate,
    pub days_to_expiry: i64,
    pub iv_by_moneyness: Vec<Option<f64>>, // length == MONEYNESS.len()
    pub atm_iv: Option<f64>,
    pub atm_call_mid: Option<f64>,
    pub atm_put_mid: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TermPoint {
    pub days_to_expiry: i64,
    pub atm_iv: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SkewPoint {
    pub moneyness: f64,
    pub strike: f64,
    pub call_iv: Option<f64>,
    pub put_iv: Option<f64>,
}

pub async fn surface(symbol: &str, n_expirations: usize) -> anyhow::Result<VolSurface> {
    let head = options::chain(symbol, None).await?;
    let spot = head.spot;
    if spot <= 0.0 {
        anyhow::bail!("no spot for {}", symbol);
    }
    let today = Utc::now().date_naive();

    let mut rows = Vec::new();
    let exps: Vec<NaiveDate> = head
        .expirations
        .iter()
        .filter(|d| **d >= today)
        .take(n_expirations.max(1))
        .copied()
        .collect();

    for exp in &exps {
        // First expiration is already loaded in `head`; refetch for consistency
        // when not the first.
        let chain = if *exp == head.expiration {
            head.clone()
        } else {
            match options::chain(symbol, Some(*exp)).await {
                Ok(c) => c,
                Err(_) => continue,
            }
        };
        let dte = (*exp - today).num_days().max(0);
        let t_years = (dte as f64 / 365.0).max(1.0 / 365.0);

        let mut iv_row = Vec::with_capacity(MONEYNESS.len());
        for m in MONEYNESS {
            let target = spot * (1.0 + m);
            // Out-of-the-money convention: OTM calls above spot, OTM puts below.
            // For IV stability we prefer the OTM side.
            let iv = if *m >= 0.0 {
                iv_at_strike(&chain.calls, OptKind::Call, target, spot, t_years)
            } else {
                iv_at_strike(&chain.puts, OptKind::Put, target, spot, t_years)
            };
            iv_row.push(iv);
        }

        // ATM = average of call & put at nearest strike.
        let atm_c = nearest_strike(&chain.calls, spot);
        let atm_p = nearest_strike(&chain.puts, spot);
        let cv = atm_c
            .as_ref()
            .and_then(|c| resolve_iv(c, OptKind::Call, spot, t_years));
        let pv = atm_p
            .as_ref()
            .and_then(|c| resolve_iv(c, OptKind::Put, spot, t_years));
        let atm_iv = match (cv, pv) {
            (Some(a), Some(b)) => Some((a + b) / 2.0),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            _ => None,
        };

        rows.push(ExpirationRow {
            expiration: *exp,
            days_to_expiry: dte,
            iv_by_moneyness: iv_row,
            atm_iv,
            atm_call_mid: atm_c.and_then(|c| mid(&c)),
            atm_put_mid: atm_p.and_then(|c| mid(&c)),
        });
    }

    // Term structure = filter rows where atm_iv resolved.
    let term_structure: Vec<TermPoint> = rows
        .iter()
        .filter_map(|r| {
            r.atm_iv.map(|iv| TermPoint {
                days_to_expiry: r.days_to_expiry,
                atm_iv: iv,
            })
        })
        .collect();

    // Front-month skew = the first expiration with > 0 strikes.
    let front_skew = if let Some(front) = exps.first() {
        let chain = if *front == head.expiration {
            head.clone()
        } else {
            options::chain(symbol, Some(*front))
                .await
                .unwrap_or(head.clone())
        };
        let dte = (*front - today).num_days().max(0);
        let t_years = (dte as f64 / 365.0).max(1.0 / 365.0);
        MONEYNESS
            .iter()
            .map(|m| {
                let target = spot * (1.0 + m);
                let call_iv = nearest_strike(&chain.calls, target)
                    .and_then(|c| resolve_iv(&c, OptKind::Call, spot, t_years));
                let put_iv = nearest_strike(&chain.puts, target)
                    .and_then(|c| resolve_iv(&c, OptKind::Put, spot, t_years));
                SkewPoint {
                    moneyness: *m,
                    strike: target,
                    call_iv,
                    put_iv,
                }
            })
            .collect()
    } else {
        Vec::new()
    };

    Ok(VolSurface {
        symbol: symbol.to_string(),
        spot,
        moneyness: MONEYNESS.to_vec(),
        expirations: rows,
        term_structure,
        front_skew,
        fetched_at: Utc::now(),
    })
}

fn iv_at_strike(
    contracts: &[OptionContract],
    kind: OptKind,
    target: f64,
    spot: f64,
    t_years: f64,
) -> Option<f64> {
    if contracts.is_empty() {
        return None;
    }
    // Find the two adjacent strikes bracketing target.
    let mut sorted: Vec<&OptionContract> = contracts.iter().collect();
    sorted.sort_by(|a, b| {
        a.strike
            .partial_cmp(&b.strike)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let mut lo: Option<&OptionContract> = None;
    let mut hi: Option<&OptionContract> = None;
    for c in &sorted {
        if c.strike <= target {
            lo = Some(c);
        }
        if c.strike >= target && hi.is_none() {
            hi = Some(c);
            break;
        }
    }
    let (lo, hi) = match (lo, hi) {
        (Some(l), Some(h)) => (l, h),
        (Some(l), None) => (l, l),
        (None, Some(h)) => (h, h),
        (None, None) => return None,
    };
    let lo_iv = resolve_iv(lo, kind, spot, t_years)?;
    let hi_iv = resolve_iv(hi, kind, spot, t_years)?;
    if (hi.strike - lo.strike).abs() < f64::EPSILON {
        return Some(lo_iv);
    }
    let w = (target - lo.strike) / (hi.strike - lo.strike);
    Some(lo_iv + w * (hi_iv - lo_iv))
}

fn resolve_iv(c: &OptionContract, kind: OptKind, spot: f64, t_years: f64) -> Option<f64> {
    if let Some(iv) = c.implied_vol {
        if iv > 0.0 && iv.is_finite() {
            return Some(iv);
        }
    }
    let m = mid(c)?;
    if m <= 0.0 {
        return None;
    }
    implied_vol(kind, m, spot, c.strike, t_years, RISK_FREE, DIVIDEND)
}

fn mid(c: &OptionContract) -> Option<f64> {
    match (c.bid, c.ask) {
        (Some(b), Some(a)) if b > 0.0 && a > 0.0 => Some((a + b) / 2.0),
        _ => c.last_price.filter(|p| *p > 0.0),
    }
}

fn nearest_strike(contracts: &[OptionContract], target: f64) -> Option<OptionContract> {
    contracts
        .iter()
        .min_by(|a, b| {
            (a.strike - target)
                .abs()
                .partial_cmp(&(b.strike - target).abs())
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===========================================================================
    // Fixture builder
    // ===========================================================================

    fn c(strike: f64, iv: Option<f64>) -> OptionContract {
        OptionContract {
            strike,
            bid: None,
            ask: None,
            last_price: None,
            implied_vol: iv,
            volume: None,
            open_interest: None,
            in_the_money: false,
        }
    }

    fn c_with_quotes(strike: f64, bid: f64, ask: f64) -> OptionContract {
        OptionContract {
            strike,
            bid: Some(bid),
            ask: Some(ask),
            last_price: None,
            implied_vol: None,
            volume: None,
            open_interest: None,
            in_the_money: false,
        }
    }

    // ===========================================================================
    // mid — quote priority
    // ===========================================================================

    #[test]
    fn mid_averages_bid_and_ask_when_both_positive() {
        let q = c_with_quotes(100.0, 2.0, 4.0);
        assert_eq!(mid(&q), Some(3.0));
    }

    #[test]
    fn mid_falls_back_to_last_price_when_quotes_missing() {
        let mut q = c(100.0, None);
        q.last_price = Some(5.5);
        assert_eq!(mid(&q), Some(5.5));
    }

    #[test]
    fn mid_returns_none_when_bid_or_ask_is_zero() {
        // bid=0 → not both positive → fall through to last_price (also None).
        let q = c_with_quotes(100.0, 0.0, 4.0);
        assert_eq!(mid(&q), None);
    }

    #[test]
    fn mid_returns_none_when_last_price_is_zero() {
        let mut q = c(100.0, None);
        q.last_price = Some(0.0);
        assert_eq!(mid(&q), None);
    }

    #[test]
    fn mid_returns_none_when_no_quotes_or_last_price() {
        assert_eq!(mid(&c(100.0, None)), None);
    }

    // ===========================================================================
    // nearest_strike — minimum absolute distance
    // ===========================================================================

    #[test]
    fn nearest_strike_returns_none_on_empty_input() {
        assert!(nearest_strike(&[], 100.0).is_none());
    }

    #[test]
    fn nearest_strike_picks_closest_to_target() {
        let contracts = vec![c(90.0, None), c(95.0, None), c(105.0, None), c(120.0, None)];
        let n = nearest_strike(&contracts, 100.0).unwrap();
        // 95 and 105 are equidistant; min_by returns the first equal element (95).
        assert_eq!(n.strike, 95.0);
    }

    #[test]
    fn nearest_strike_handles_exact_match() {
        let contracts = vec![c(95.0, None), c(100.0, None), c(105.0, None)];
        let n = nearest_strike(&contracts, 100.0).unwrap();
        assert_eq!(n.strike, 100.0);
    }

    #[test]
    fn nearest_strike_handles_target_below_min() {
        let contracts = vec![c(100.0, None), c(110.0, None)];
        let n = nearest_strike(&contracts, 50.0).unwrap();
        assert_eq!(n.strike, 100.0);
    }

    #[test]
    fn nearest_strike_handles_target_above_max() {
        let contracts = vec![c(100.0, None), c(110.0, None)];
        let n = nearest_strike(&contracts, 999.0).unwrap();
        assert_eq!(n.strike, 110.0);
    }

    // ===========================================================================
    // iv_at_strike — interpolation between brackets
    // ===========================================================================

    #[test]
    fn iv_at_strike_returns_none_on_empty_chain() {
        let r = iv_at_strike(&[], OptKind::Call, 100.0, 100.0, 0.25);
        assert!(r.is_none());
    }

    #[test]
    fn iv_at_strike_returns_exact_iv_when_target_matches_strike() {
        // Single contract → lo == hi → no interpolation, returns lo_iv directly.
        let contracts = vec![c(100.0, Some(0.30))];
        let r = iv_at_strike(&contracts, OptKind::Call, 100.0, 100.0, 0.25);
        assert!((r.unwrap() - 0.30).abs() < 1e-9);
    }

    #[test]
    fn iv_at_strike_linearly_interpolates_between_two_strikes() {
        // strikes: 90 (IV 0.20), 110 (IV 0.40). Target 100 → IV = 0.30.
        let contracts = vec![c(90.0, Some(0.20)), c(110.0, Some(0.40))];
        let r = iv_at_strike(&contracts, OptKind::Call, 100.0, 100.0, 0.25);
        assert!((r.unwrap() - 0.30).abs() < 1e-9);
    }

    #[test]
    fn iv_at_strike_quarter_weight_on_lower_strike() {
        // strikes: 90 (IV 0.20), 110 (IV 0.40). Target 95 → w=0.25 → IV=0.25.
        let contracts = vec![c(90.0, Some(0.20)), c(110.0, Some(0.40))];
        let r = iv_at_strike(&contracts, OptKind::Call, 95.0, 100.0, 0.25);
        assert!((r.unwrap() - 0.25).abs() < 1e-9);
    }

    #[test]
    fn iv_at_strike_target_above_all_strikes_uses_highest_strike_iv() {
        // No bracket above; lo = highest, hi = lo via (Some, None) branch → returns lo_iv.
        let contracts = vec![c(90.0, Some(0.20)), c(100.0, Some(0.25))];
        let r = iv_at_strike(&contracts, OptKind::Call, 200.0, 100.0, 0.25);
        assert!((r.unwrap() - 0.25).abs() < 1e-9);
    }

    #[test]
    fn iv_at_strike_target_below_all_strikes_uses_lowest_strike_iv() {
        let contracts = vec![c(100.0, Some(0.25)), c(110.0, Some(0.30))];
        let r = iv_at_strike(&contracts, OptKind::Call, 50.0, 100.0, 0.25);
        assert!((r.unwrap() - 0.25).abs() < 1e-9);
    }

    #[test]
    fn iv_at_strike_sorts_contracts_so_input_order_does_not_matter() {
        // Same input as the interpolation test, but reversed order.
        let contracts = vec![c(110.0, Some(0.40)), c(90.0, Some(0.20))];
        let r = iv_at_strike(&contracts, OptKind::Call, 100.0, 100.0, 0.25);
        assert!((r.unwrap() - 0.30).abs() < 1e-9);
    }
}
