//! Key-Rate Duration (Ho 1992).
//!
//! Decomposes total duration into sensitivity to each tenor on the
//! yield curve. For each key rate t_k:
//!
//!   KRD_k = − (∂PV/∂y_k) / PV
//!
//! computed by triangular bumping: shift the spot curve at tenor t_k
//! by ±δ (linearly tapered to zero at neighboring keys), reprice the
//! bond, and finite-difference.
//!
//! Sum of KRDs ≈ effective duration (parallel-shift sensitivity).
//! Used in immunization, risk decomposition, asset-liability matching.
//!
//! Pure compute on a vector of (cash-flow time, cash-flow amount)
//! pairs + a piecewise-linear spot-rate curve at provided key tenors.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CashFlow {
    pub time_years: f64,
    pub amount: f64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct KeyTenor {
    pub time_years: f64,
    pub spot_rate: f64, // continuously-compounded
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KrdReport {
    pub present_value: f64,
    /// Per-tenor KRD; aligned positionally with the input `tenors`.
    pub krd_per_tenor: Vec<f64>,
    /// Sum of KRDs (approx. effective duration).
    pub total_duration: f64,
}

pub fn compute(
    cash_flows: &[CashFlow],
    tenors: &[KeyTenor],
    bump_basis_points: f64,
) -> Option<KrdReport> {
    if cash_flows.is_empty() || tenors.len() < 2 || bump_basis_points <= 0.0 {
        return None;
    }
    if cash_flows
        .iter()
        .any(|c| !c.time_years.is_finite() || c.time_years < 0.0 || !c.amount.is_finite())
    {
        return None;
    }
    if tenors
        .iter()
        .any(|t| !t.time_years.is_finite() || !t.spot_rate.is_finite() || t.time_years < 0.0)
    {
        return None;
    }
    // Tenors must be strictly increasing in time.
    for w in tenors.windows(2) {
        if w[1].time_years <= w[0].time_years {
            return None;
        }
    }
    let pv0 = price(cash_flows, tenors)?;
    if pv0 <= 0.0 {
        return None;
    }
    let delta = bump_basis_points / 10_000.0;
    let mut krds = Vec::with_capacity(tenors.len());
    for k in 0..tenors.len() {
        let up = price_with_triangular_bump(cash_flows, tenors, k, delta)?;
        let dn = price_with_triangular_bump(cash_flows, tenors, k, -delta)?;
        // Effective KRD via central difference: KRD = (P_dn − P_up)/(2δ·P).
        let krd = (dn - up) / (2.0 * delta * pv0);
        krds.push(krd);
    }
    let total: f64 = krds.iter().sum();
    Some(KrdReport {
        present_value: pv0,
        krd_per_tenor: krds,
        total_duration: total,
    })
}

fn price(cf: &[CashFlow], tenors: &[KeyTenor]) -> Option<f64> {
    let mut pv = 0.0;
    for c in cf {
        let r = interp_rate(c.time_years, tenors)?;
        pv += c.amount * (-r * c.time_years).exp();
    }
    Some(pv)
}

fn price_with_triangular_bump(
    cf: &[CashFlow],
    tenors: &[KeyTenor],
    k: usize,
    delta: f64,
) -> Option<f64> {
    let mut bumped: Vec<KeyTenor> = tenors.to_vec();
    bumped[k].spot_rate += delta; // peak bump
                                  // Triangular taper: bump tapers linearly to zero at k−1 and k+1.
                                  // Implemented by *also* bumping intermediate tenors? In the "key-
                                  // rate" method, only the keys themselves are bumped — the linear
                                  // interpolation between keys generates the triangular shape
                                  // automatically when re-interpolating to the cash-flow times.
    price(cf, &bumped)
}

fn interp_rate(t: f64, tenors: &[KeyTenor]) -> Option<f64> {
    if tenors.is_empty() {
        return None;
    }
    if t <= tenors[0].time_years {
        return Some(tenors[0].spot_rate);
    }
    if t >= tenors[tenors.len() - 1].time_years {
        return Some(tenors[tenors.len() - 1].spot_rate);
    }
    for w in tenors.windows(2) {
        if t >= w[0].time_years && t <= w[1].time_years {
            let span = w[1].time_years - w[0].time_years;
            if span <= 0.0 {
                return Some(w[0].spot_rate);
            }
            let frac = (t - w[0].time_years) / span;
            return Some(w[0].spot_rate + frac * (w[1].spot_rate - w[0].spot_rate));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_curve(rate: f64) -> Vec<KeyTenor> {
        vec![
            KeyTenor {
                time_years: 0.5,
                spot_rate: rate,
            },
            KeyTenor {
                time_years: 2.0,
                spot_rate: rate,
            },
            KeyTenor {
                time_years: 5.0,
                spot_rate: rate,
            },
            KeyTenor {
                time_years: 10.0,
                spot_rate: rate,
            },
        ]
    }

    fn ten_year_bond() -> Vec<CashFlow> {
        // 10-year 5% annual-coupon bond, face 100.
        let mut cf = Vec::new();
        for t in 1..=10 {
            cf.push(CashFlow {
                time_years: t as f64,
                amount: 5.0,
            });
        }
        cf.last_mut().unwrap().amount = 105.0;
        cf
    }

    #[test]
    fn empty_or_invalid_returns_none() {
        assert!(compute(&[], &flat_curve(0.05), 1.0).is_none());
        let cf = ten_year_bond();
        assert!(compute(
            &cf,
            &[KeyTenor {
                time_years: 1.0,
                spot_rate: 0.05
            }],
            1.0
        )
        .is_none());
        assert!(compute(&cf, &flat_curve(0.05), 0.0).is_none());
        assert!(compute(&cf, &flat_curve(0.05), -1.0).is_none());
    }

    #[test]
    fn non_monotonic_tenors_rejected() {
        let bad = vec![
            KeyTenor {
                time_years: 5.0,
                spot_rate: 0.05,
            },
            KeyTenor {
                time_years: 2.0,
                spot_rate: 0.05,
            },
        ];
        let cf = ten_year_bond();
        assert!(compute(&cf, &bad, 1.0).is_none());
    }

    #[test]
    fn nan_tenor_rejected() {
        let bad = vec![
            KeyTenor {
                time_years: 0.5,
                spot_rate: f64::NAN,
            },
            KeyTenor {
                time_years: 2.0,
                spot_rate: 0.05,
            },
        ];
        let cf = ten_year_bond();
        assert!(compute(&cf, &bad, 1.0).is_none());
    }

    #[test]
    fn total_duration_approximates_macaulay() {
        // Flat 5% curve, 10y bond at par; Macaulay ≈ 8.1 years.
        // Sum of KRDs should approximate effective duration ≈ 8 years.
        let cf = ten_year_bond();
        let r = compute(&cf, &flat_curve(0.05), 1.0).unwrap();
        assert!(
            r.total_duration > 7.0 && r.total_duration < 9.0,
            "total duration {} outside expected ~8y for 10y bond",
            r.total_duration
        );
    }

    #[test]
    fn all_krds_non_negative_for_vanilla_bond() {
        let cf = ten_year_bond();
        let r = compute(&cf, &flat_curve(0.05), 1.0).unwrap();
        for (i, k) in r.krd_per_tenor.iter().enumerate() {
            assert!(
                *k >= -1e-6,
                "KRD[{i}] = {k} unexpectedly negative for vanilla bond"
            );
        }
    }

    #[test]
    fn longest_tenor_dominates_for_10y_bond() {
        // For a 10y bond, the 10y key rate KRD should dominate (large CF at 10y).
        let cf = ten_year_bond();
        let r = compute(&cf, &flat_curve(0.05), 1.0).unwrap();
        let max_idx = r
            .krd_per_tenor
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .unwrap();
        assert_eq!(
            max_idx, 3,
            "10y bond should have highest KRD at 10y key tenor"
        );
    }

    #[test]
    fn zero_coupon_concentrates_krd() {
        let cf = vec![CashFlow {
            time_years: 5.0,
            amount: 100.0,
        }];
        let r = compute(&cf, &flat_curve(0.05), 1.0).unwrap();
        // 5y zero between 2y and 10y keys → mostly 2y + 5y KRD contributions.
        // Sum should be ~ 5y duration.
        assert!(r.total_duration > 4.0 && r.total_duration < 6.0);
    }
}
