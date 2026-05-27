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
