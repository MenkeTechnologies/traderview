//! CD (Certificate of Deposit) ladder builder.
//!
//! Standard CD-ladder strategy: split a lump sum equally across N
//! rungs, each maturing one year apart. As each rung matures, roll
//! the proceeds into a new N-year CD at the top of the ladder. This
//! gives you annual liquidity while still earning the (typically
//! higher) longer-term CD rate on most of the money.
//!
//! Inputs:
//!   - total_principal_usd
//!   - rungs        — number of CDs (e.g. 5 for a 1-5 year ladder)
//!   - term_years_per_rung — typically 1 (each rung extends by 1y)
//!   - per_rung_apy_pct[]  — published APY per rung, length = rungs
//!     (if a single rate is supplied as input.flat_apy_pct, all rungs
//!      use the same rate)
//!   - flat_apy_pct (optional) — overrides per_rung when set
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CdLadderInput {
    pub total_principal_usd: f64,
    pub rungs: u32,
    pub term_years_per_rung: u32,
    #[serde(default)]
    pub per_rung_apy_pct: Vec<f64>,
    #[serde(default)]
    pub flat_apy_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct RungReport {
    pub rung: u32,
    pub maturity_years: u32,
    pub principal_usd: f64,
    pub apy_pct: f64,
    pub interest_at_maturity_usd: f64,
    pub balance_at_maturity_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct CdLadderReport {
    pub rungs: Vec<RungReport>,
    pub blended_apy_pct: f64,
    pub total_principal_usd: f64,
    pub total_interest_full_ladder_usd: f64,
    pub annual_maturity_proceeds_usd: f64,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn compute(input: &CdLadderInput) -> CdLadderReport {
    let n = input.rungs.max(1);
    let per_rung_principal = if n > 0 {
        input.total_principal_usd / n as f64
    } else { 0.0 };
    let term = input.term_years_per_rung;

    let mut rungs: Vec<RungReport> = Vec::with_capacity(n as usize);
    let mut sum_weighted_apy = 0.0_f64;
    let mut sum_principal = 0.0_f64;
    let mut sum_interest = 0.0_f64;
    for i in 0..n {
        let maturity_years = (i + 1) * term;
        let apy = if let Some(flat) = input.flat_apy_pct {
            flat
        } else if let Some(p) = input.per_rung_apy_pct.get(i as usize) {
            *p
        } else {
            0.0
        };
        let r = apy / 100.0;
        // Compound annually at APY for `maturity_years` years.
        let balance = per_rung_principal * (1.0 + r).powi(maturity_years as i32);
        let interest = balance - per_rung_principal;
        sum_weighted_apy += apy * per_rung_principal;
        sum_principal += per_rung_principal;
        sum_interest += interest;
        rungs.push(RungReport {
            rung: i + 1,
            maturity_years,
            principal_usd: per_rung_principal,
            apy_pct: apy,
            interest_at_maturity_usd: interest,
            balance_at_maturity_usd: balance,
        });
    }
    let blended = if sum_principal > 0.0 {
        sum_weighted_apy / sum_principal
    } else { 0.0 };
    // After the ladder fully populates (year `n` × term), the user gets
    // one rung maturing per `term` years. Annual proceeds (ladder
    // running at full maturity, rolled into new N-yr CD each turn) =
    // per_rung_principal at compound interest.
    let annual_proceeds = if let Some(rep) = rungs.last() {
        rep.balance_at_maturity_usd
    } else { 0.0 };
    CdLadderReport {
        rungs,
        blended_apy_pct: blended,
        total_principal_usd: input.total_principal_usd,
        total_interest_full_ladder_usd: sum_interest,
        annual_maturity_proceeds_usd: annual_proceeds,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> CdLadderInput {
        CdLadderInput {
            total_principal_usd: 50_000.0,
            rungs: 5,
            term_years_per_rung: 1,
            per_rung_apy_pct: vec![4.5, 4.7, 4.8, 4.9, 5.0],
            flat_apy_pct: None,
        }
    }

    #[test]
    fn compute_rung_count_basic() {
        let r = compute(&input());
        assert_eq!(r.rungs.len(), 5);
    }

    #[test]
    fn compute_per_rung_principal_equal_split() {
        let r = compute(&input());
        // $50k / 5 = $10k per rung
        for rung in &r.rungs {
            assert_eq!(rung.principal_usd, 10_000.0);
        }
    }

    #[test]
    fn compute_maturity_years_increasing() {
        let r = compute(&input());
        for (i, rung) in r.rungs.iter().enumerate() {
            assert_eq!(rung.maturity_years, (i as u32 + 1));
        }
    }

    #[test]
    fn compute_apy_uses_per_rung() {
        let r = compute(&input());
        assert_eq!(r.rungs[0].apy_pct, 4.5);
        assert_eq!(r.rungs[4].apy_pct, 5.0);
    }

    #[test]
    fn compute_flat_apy_overrides() {
        let mut i = input();
        i.flat_apy_pct = Some(5.5);
        let r = compute(&i);
        for rung in &r.rungs {
            assert_eq!(rung.apy_pct, 5.5);
        }
    }

    #[test]
    fn compute_interest_higher_for_longer_term_at_same_apy() {
        let mut i = input();
        i.flat_apy_pct = Some(5.0);
        let r = compute(&i);
        for w in r.rungs.windows(2) {
            assert!(w[1].interest_at_maturity_usd > w[0].interest_at_maturity_usd);
        }
    }

    #[test]
    fn compute_balance_basic() {
        let mut i = input();
        i.flat_apy_pct = Some(5.0);
        let r = compute(&i);
        // Rung 1: $10k × 1.05^1 = $10,500
        assert!((r.rungs[0].balance_at_maturity_usd - 10_500.0).abs() < 0.5);
        // Rung 5: $10k × 1.05^5 ≈ $12,762.82
        assert!((r.rungs[4].balance_at_maturity_usd - 12_762.82).abs() < 1.0);
    }

    #[test]
    fn compute_blended_apy_basic() {
        let r = compute(&input());
        // Equal principal per rung → simple average APY.
        // (4.5 + 4.7 + 4.8 + 4.9 + 5.0) / 5 = 4.78
        assert!((r.blended_apy_pct - 4.78).abs() < 0.01);
    }

    #[test]
    fn compute_total_interest_positive() {
        let r = compute(&input());
        assert!(r.total_interest_full_ladder_usd > 0.0);
    }

    #[test]
    fn compute_zero_rungs_fallback() {
        let mut i = input();
        i.rungs = 0;
        let r = compute(&i);
        // rungs.max(1) ensures at least 1 rung created.
        assert_eq!(r.rungs.len(), 1);
    }

    #[test]
    fn compute_two_year_terms() {
        let mut i = input();
        i.term_years_per_rung = 2;
        let r = compute(&i);
        assert_eq!(r.rungs[0].maturity_years, 2);
        assert_eq!(r.rungs[4].maturity_years, 10);
    }

    #[test]
    fn compute_missing_per_rung_apy_zero() {
        let mut i = input();
        i.per_rung_apy_pct = vec![4.5];  // only one rung specified
        let r = compute(&i);
        // First rung uses 4.5, rest fall through to 0.
        assert_eq!(r.rungs[0].apy_pct, 4.5);
        assert_eq!(r.rungs[1].apy_pct, 0.0);
    }
}
