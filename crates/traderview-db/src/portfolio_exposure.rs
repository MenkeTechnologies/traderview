//! Portfolio factor-exposure dashboard.
//!
//! Correlation gate already prevents a single candidate from stacking
//! on the same factor as existing positions. But it doesn't show *what*
//! factor you're loaded on. This module decomposes the current paper
//! portfolio:
//!
//!   * **Total β to SPY** — OLS slope of weighted portfolio returns
//!     vs SPY returns over the trailing 60d. >1.2 means amplified market
//!     exposure; <0.8 means dampened.
//!   * **Sector concentration** — bucket positions by GICS sector
//!     (uses the existing heatmap UNIVERSE mapping for the top S&P names);
//!     surface "you're 80% tech."
//!   * **Single-name HHI** — Herfindahl-Hirschman concentration on the
//!     position weights. 1.0 = 1 position, 1/N = perfectly diversified
//!     across N positions, > 0.25 = highly concentrated.
//!   * **Portfolio realized vol** — weighted-sum approximation; ignores
//!     cross-correlations for simplicity (overstates vol when positions
//!     are correlated, understates when they're anti-correlated).
//!   * **1-day 95% VaR** — parametric: 1.645 × portfolio_vol_pct × MV.

use chrono::{Duration, NaiveDate, Utc};
use rust_decimal::prelude::ToPrimitive;
use serde::Serialize;
use sqlx::PgPool;
use std::collections::HashMap;
use traderview_core::BarInterval;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct PositionExposure {
    pub symbol: String,
    pub qty: f64,
    pub mark_price: f64,
    pub market_value: f64,
    pub weight_pct: f64,
    pub sector: Option<String>,
    pub beta_to_spy: Option<f64>,
    pub realized_vol_pct_annualised: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SectorWeight {
    pub sector: String,
    pub weight_pct: f64,
    pub position_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExposureReport {
    pub total_market_value: f64,
    pub positions: Vec<PositionExposure>,
    pub portfolio_beta_to_spy: Option<f64>,
    pub sector_weights: Vec<SectorWeight>,
    /// Herfindahl-Hirschman on weight squares. 1.0 = single position,
    /// 1/N = perfectly diversified across N equal positions, ≥ 0.25 = high.
    pub single_name_hhi: f64,
    pub portfolio_vol_pct_annualised: Option<f64>,
    pub var_95_1day_usd: Option<f64>,
}

// ─── Pure compute ──────────────────────────────────────────────────────────

/// OLS slope (beta): cov(asset, market) / var(market). Inputs are aligned
/// per-period returns. Returns `None` when N < 2 or market variance is zero.
pub fn ols_beta(asset_returns: &[f64], market_returns: &[f64]) -> Option<f64> {
    let n = asset_returns.len();
    if n < 2 || n != market_returns.len() {
        return None;
    }
    let mean_a = asset_returns.iter().sum::<f64>() / n as f64;
    let mean_m = market_returns.iter().sum::<f64>() / n as f64;
    let mut cov = 0.0;
    let mut var_m = 0.0;
    for (a, m) in asset_returns.iter().zip(market_returns.iter()) {
        let da = a - mean_a;
        let dm = m - mean_m;
        cov += da * dm;
        var_m += dm * dm;
    }
    if var_m <= 0.0 {
        return None;
    }
    Some(cov / var_m)
}

/// Herfindahl-Hirschman concentration on a weight vector. Each weight
/// is a fraction in [0, 1]; HHI = Σ w². 1 = single position; 1/N for N
/// equal-weighted positions.
pub fn hhi(weights: &[f64]) -> f64 {
    weights.iter().map(|w| w * w).sum()
}

/// Annualised stdev of daily % returns. Returns `None` when N < 2.
pub fn annualised_vol_pct(returns_pct: &[f64]) -> Option<f64> {
    let n = returns_pct.len();
    if n < 2 {
        return None;
    }
    let mean = returns_pct.iter().sum::<f64>() / n as f64;
    let var = returns_pct.iter().map(|r| (r - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
    Some(var.sqrt() * (252.0_f64).sqrt())
}

/// Weighted portfolio return series given per-asset returns + weights.
/// Assumes per-asset return arrays are date-aligned; caller's job to
/// build them via inner join on dates.
pub fn weighted_portfolio_returns(
    weights: &[f64],
    asset_returns: &[Vec<f64>],
    n_days: usize,
) -> Vec<f64> {
    if weights.is_empty() || asset_returns.is_empty() {
        return Vec::new();
    }
    let mut out = vec![0.0; n_days];
    for (i, ret_series) in asset_returns.iter().enumerate() {
        if i >= weights.len() {
            break;
        }
        for (j, r) in ret_series.iter().enumerate() {
            if j < n_days {
                out[j] += weights[i] * r;
            }
        }
    }
    out
}

/// 1-day 95% parametric VaR: 1.645 × daily_vol × portfolio_mv.
/// Inputs: annualised vol % (e.g. 25 = 25%) and total MV in USD.
pub fn var_95_1day(annualised_vol_pct: f64, mv_usd: f64) -> f64 {
    if !(annualised_vol_pct.is_finite() && annualised_vol_pct > 0.0 && mv_usd > 0.0) {
        return 0.0;
    }
    let daily_vol_pct = annualised_vol_pct / (252.0_f64).sqrt();
    1.645 * daily_vol_pct / 100.0 * mv_usd
}

// ─── Repository ────────────────────────────────────────────────────────────

async fn fetch_closes(pool: &PgPool, symbol: &str, days: i64) -> Vec<(NaiveDate, f64)> {
    let to = Utc::now();
    let from = to - Duration::days(days);
    let bars = crate::prices::get_bars(pool, symbol, BarInterval::D1, from, to)
        .await
        .unwrap_or_default();
    bars.into_iter()
        .filter_map(|b| b.close.to_f64().map(|c| (b.bar_time.date_naive(), c)))
        .collect()
}

/// Build the exposure report for the user's default paper account.
/// Best-effort: positions whose price_bars are empty get beta/vol = None
/// and contribute MV using avg_price as the mark.
pub async fn compute_exposure(pool: &PgPool, user_id: Uuid) -> anyhow::Result<ExposureReport> {
    let account = crate::paper::ensure_default(pool, user_id).await?;
    let positions = crate::paper::positions(pool, account.id).await?;
    if positions.is_empty() {
        return Ok(ExposureReport {
            total_market_value: 0.0,
            positions: Vec::new(),
            portfolio_beta_to_spy: None,
            sector_weights: Vec::new(),
            single_name_hhi: 0.0,
            portfolio_vol_pct_annualised: None,
            var_95_1day_usd: None,
        });
    }

    let spy_closes = fetch_closes(pool, "SPY", 90).await;
    let spy_returns = crate::correlation::pct_returns(&spy_closes);

    let mut exposures: Vec<PositionExposure> = Vec::new();
    let mut per_position_returns: Vec<Vec<f64>> = Vec::new();
    for p in &positions {
        let qty = p.qty.to_f64().unwrap_or(0.0);
        if qty == 0.0 {
            continue;
        }
        let avg_price = p.avg_price.to_f64().unwrap_or(0.0);
        let closes = fetch_closes(pool, &p.symbol, 90).await;
        let last_close = closes.last().map(|(_, c)| *c).unwrap_or(avg_price);
        let mv = qty * last_close;
        let asset_returns = crate::correlation::pct_returns(&closes);
        let beta = align_then_ols(&closes, &spy_closes);
        let vol = annualised_vol_pct(&asset_returns);
        exposures.push(PositionExposure {
            symbol: p.symbol.clone(),
            qty,
            mark_price: last_close,
            market_value: mv,
            weight_pct: 0.0,
            sector: crate::heatmap::sector_for(&p.symbol).map(|s| s.to_string()),
            beta_to_spy: beta,
            realized_vol_pct_annualised: vol,
        });
        per_position_returns.push(asset_returns);
    }

    let total_mv: f64 = exposures.iter().map(|e| e.market_value.abs()).sum();
    if total_mv > 0.0 {
        for e in exposures.iter_mut() {
            e.weight_pct = e.market_value.abs() / total_mv * 100.0;
        }
    }

    // Sector buckets.
    let mut by_sector: HashMap<String, (f64, usize)> = HashMap::new();
    for e in &exposures {
        let sector = e.sector.clone().unwrap_or_else(|| "Unclassified".into());
        let entry = by_sector.entry(sector).or_insert((0.0, 0));
        entry.0 += e.weight_pct;
        entry.1 += 1;
    }
    let mut sector_weights: Vec<SectorWeight> = by_sector
        .into_iter()
        .map(|(sector, (w, n))| SectorWeight {
            sector,
            weight_pct: w,
            position_count: n,
        })
        .collect();
    sector_weights.sort_by(|a, b| {
        b.weight_pct
            .partial_cmp(&a.weight_pct)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // HHI on position-level weights (as fractions in [0, 1]).
    let weight_fracs: Vec<f64> = exposures.iter().map(|e| e.weight_pct / 100.0).collect();
    let single_name_hhi = hhi(&weight_fracs);

    // Portfolio beta = Σ weight × per-position beta where defined.
    // Falls back to None when no position has a defined β.
    let portfolio_beta = {
        let mut sum = 0.0;
        let mut covered = 0.0;
        for e in &exposures {
            if let Some(b) = e.beta_to_spy {
                sum += e.weight_pct / 100.0 * b;
                covered += e.weight_pct / 100.0;
            }
        }
        if covered > 0.0 {
            Some(sum / covered)
        } else {
            None
        }
    };

    // Portfolio vol: build a weighted return series from per-position
    // returns (rough — ignores cross-correlations). The signed weight
    // matters (short positions get negative weight).
    let portfolio_vol = {
        // Align all per-position return series to the SPY length.
        let n_days = spy_returns.len();
        if n_days < 2 {
            None
        } else {
            let signed_weights: Vec<f64> = exposures
                .iter()
                .map(|e| {
                    let frac = e.weight_pct / 100.0;
                    if e.qty < 0.0 {
                        -frac
                    } else {
                        frac
                    }
                })
                .collect();
            let aligned: Vec<Vec<f64>> = per_position_returns
                .iter()
                .map(|s| {
                    if s.len() >= n_days {
                        s[s.len() - n_days..].to_vec()
                    } else {
                        let mut padded = vec![0.0; n_days - s.len()];
                        padded.extend_from_slice(s);
                        padded
                    }
                })
                .collect();
            let port_returns = weighted_portfolio_returns(&signed_weights, &aligned, n_days);
            annualised_vol_pct(&port_returns)
        }
    };

    let var_95 = portfolio_vol.map(|v| var_95_1day(v, total_mv));

    Ok(ExposureReport {
        total_market_value: total_mv,
        positions: exposures,
        portfolio_beta_to_spy: portfolio_beta,
        sector_weights,
        single_name_hhi,
        portfolio_vol_pct_annualised: portfolio_vol,
        var_95_1day_usd: var_95,
    })
}

fn align_then_ols(
    asset_closes: &[(NaiveDate, f64)],
    market_closes: &[(NaiveDate, f64)],
) -> Option<f64> {
    use std::collections::HashMap;
    let m_map: HashMap<NaiveDate, f64> = market_closes.iter().copied().collect();
    let mut aligned_a: Vec<(NaiveDate, f64)> = Vec::new();
    let mut aligned_m: Vec<(NaiveDate, f64)> = Vec::new();
    for (date, va) in asset_closes {
        if let Some(vm) = m_map.get(date).copied() {
            aligned_a.push((*date, *va));
            aligned_m.push((*date, vm));
        }
    }
    if aligned_a.len() < 5 {
        return None;
    }
    let asset_returns = crate::correlation::pct_returns(&aligned_a);
    let market_returns = crate::correlation::pct_returns(&aligned_m);
    ols_beta(&asset_returns, &market_returns)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ols_beta_identity_series_is_one() {
        // Asset = market exactly → beta = 1.
        let returns: Vec<f64> = (0..20).map(|i| (i % 3) as f64 - 1.0).collect();
        let b = ols_beta(&returns, &returns).unwrap();
        assert!((b - 1.0).abs() < 1e-9);
    }

    #[test]
    fn ols_beta_2x_amplified_is_two() {
        let market: Vec<f64> = (0..20).map(|i| (i % 5) as f64 - 2.0).collect();
        let asset: Vec<f64> = market.iter().map(|m| 2.0 * m).collect();
        let b = ols_beta(&asset, &market).unwrap();
        assert!((b - 2.0).abs() < 1e-9);
    }

    #[test]
    fn ols_beta_inverse_series_is_minus_one() {
        let market: Vec<f64> = (0..20).map(|i| (i % 4) as f64 - 1.5).collect();
        let asset: Vec<f64> = market.iter().map(|m| -m).collect();
        let b = ols_beta(&asset, &market).unwrap();
        assert!((b - (-1.0)).abs() < 1e-9);
    }

    #[test]
    fn ols_beta_none_on_zero_variance_market() {
        let market = vec![1.0, 1.0, 1.0, 1.0];
        let asset = vec![1.0, 2.0, 3.0, 4.0];
        assert!(ols_beta(&asset, &market).is_none());
    }

    #[test]
    fn ols_beta_none_on_length_mismatch() {
        let a = vec![1.0, 2.0];
        let m = vec![1.0, 2.0, 3.0];
        assert!(ols_beta(&a, &m).is_none());
    }

    #[test]
    fn hhi_single_position_is_one() {
        assert!((hhi(&[1.0]) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn hhi_n_equal_positions_is_one_over_n() {
        let w = vec![0.25; 4];
        assert!((hhi(&w) - 0.25).abs() < 1e-9);
        let w = vec![0.10; 10];
        assert!((hhi(&w) - 0.10).abs() < 1e-9);
    }

    #[test]
    fn hhi_concentrated_portfolio_high() {
        // 80% one name + 4 × 5% — HHI = 0.64 + 4 × 0.0025 = 0.65
        let w = vec![0.80, 0.05, 0.05, 0.05, 0.05];
        let h = hhi(&w);
        assert!((h - 0.65).abs() < 1e-9);
    }

    #[test]
    fn annualised_vol_pct_returns_none_below_two_samples() {
        assert!(annualised_vol_pct(&[]).is_none());
        assert!(annualised_vol_pct(&[1.0]).is_none());
    }

    #[test]
    fn annualised_vol_pct_scales_by_sqrt_252() {
        // Daily stdev = 1.0% → annualised should be 1.0 × sqrt(252) ≈ 15.87%.
        let returns: Vec<f64> = (0..50)
            .map(|i| if i % 2 == 0 { 1.0 } else { -1.0 })
            .collect();
        let v = annualised_vol_pct(&returns).unwrap();
        // Stdev of [1, -1, 1, -1, ...] is sqrt((sum (r - 0)^2) / (n-1)) ≈ 1.01
        // Annualised ≈ 1.01 × √252 ≈ 16.03
        assert!(v > 15.0 && v < 17.0, "expected ~16, got {v}");
    }

    #[test]
    fn var_95_uses_1645_factor() {
        // Vol 30% annual, MV $100k → daily vol = 30/sqrt(252) ≈ 1.89% → VaR ≈ 3.11% × 100k = $3112
        let v = var_95_1day(30.0, 100_000.0);
        assert!((v - 3112.0).abs() < 50.0, "VaR ≈ 3112, got {v}");
    }

    #[test]
    fn var_95_zero_on_invalid_inputs() {
        assert_eq!(var_95_1day(0.0, 1000.0), 0.0);
        assert_eq!(var_95_1day(-5.0, 1000.0), 0.0);
        assert_eq!(var_95_1day(10.0, 0.0), 0.0);
        assert_eq!(var_95_1day(f64::NAN, 1000.0), 0.0);
    }

    #[test]
    fn weighted_portfolio_returns_blends_per_asset() {
        // Two assets at equal weight, opposite returns → portfolio = 0.
        let returns_a = vec![1.0, 2.0, 3.0];
        let returns_b = vec![-1.0, -2.0, -3.0];
        let port = weighted_portfolio_returns(&[0.5, 0.5], &[returns_a, returns_b], 3);
        for p in &port {
            assert!(p.abs() < 1e-9);
        }
    }
}
