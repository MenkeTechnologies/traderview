//! Pension lump-sum vs annuity decision calculator.
//!
//! Many employer pensions offer the retiree a choice: take a one-time
//! lump-sum buyout today, or take a monthly annuity for life (often
//! with a joint-survivor option). The math comes down to:
//!
//!   - Lump sum: invest it at `expected_real_return_pct` and draw
//!     down `monthly_annuity` per month. Compute the year the lump
//!     would run out at that withdrawal rate (or never).
//!   - Annuity: guaranteed monthly for life. No portfolio risk but no
//!     inheritance left for heirs at death.
//!
//! Decision metrics:
//!   - lump_present_value_of_annuity = PV(annuity stream to life
//!     expectancy at expected_return)
//!   - implied_internal_rate         = solve for r where PV(annuity)
//!     = lump_sum
//!   - lump_runs_out_year_at_annuity = month at which lump invested
//!     at `r` drained at the annuity payout rate hits zero
//!   - leftover_at_death_if_lump     = lump invested then drawn at
//!     annuity-equivalent rate, balance remaining at life expectancy
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PensionInput {
    pub lump_sum_usd: f64,
    pub monthly_annuity_usd: f64,
    pub current_age: u32,
    pub life_expectancy_age: u32,
    pub expected_real_return_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PensionReport {
    pub lump_sum_usd: f64,
    pub annuity_present_value_usd: f64,
    pub implied_internal_rate_pct: Option<f64>,
    pub lump_runs_out_year: Option<u32>,
    pub leftover_at_life_expectancy_usd: f64,
    pub annuity_total_payments_usd: f64,
    pub net_winner: &'static str,
    pub winner_advantage_usd: f64,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

/// Present value of a monthly annuity stream for `years` at `annual_return_pct`.
pub fn annuity_present_value(monthly: f64, annual_return_pct: f64, years: u32) -> f64 {
    if monthly <= 0.0 || years == 0 { return 0.0; }
    let r = annual_return_pct / 100.0;
    let monthly_r = r / 12.0;
    let months = (years * 12) as f64;
    if monthly_r.abs() < 1e-12 {
        return monthly * months;
    }
    monthly * (1.0 - (1.0 + monthly_r).powf(-months)) / monthly_r
}

/// Solve for the internal rate where PV(annuity) = lump.
/// Bisection on r ∈ [-50%, 100%].
pub fn implied_internal_rate(lump: f64, monthly: f64, years: u32) -> Option<f64> {
    if lump <= 0.0 || monthly <= 0.0 || years == 0 { return None; }
    let total_undiscounted = monthly * 12.0 * years as f64;
    if total_undiscounted < lump { return None; }
    let mut lo = -50.0_f64;
    let mut hi = 100.0_f64;
    for _ in 0..200 {
        let mid = (lo + hi) / 2.0;
        let pv = annuity_present_value(monthly, mid, years);
        if (pv - lump).abs() < 0.01 { return Some(mid); }
        if pv > lump { lo = mid; } else { hi = mid; }
    }
    Some((lo + hi) / 2.0)
}

/// Year (relative to start) at which lump drained at the annuity payout
/// rate hits zero. None if the lump never runs out.
pub fn lump_runs_out_in_year(
    lump: f64, monthly_withdrawal: f64, annual_return_pct: f64,
) -> Option<u32> {
    if lump <= 0.0 { return Some(0); }
    if monthly_withdrawal <= 0.0 { return None; }
    let r = annual_return_pct / 100.0;
    let monthly_r = r / 12.0;
    let mut bal = lump;
    for m in 1..=1200u32 {
        bal *= 1.0 + monthly_r;
        bal -= monthly_withdrawal;
        if bal <= 0.005 { return Some(m / 12 + if m % 12 != 0 { 1 } else { 0 }); }
    }
    None
}

/// Balance remaining after `years` of drawing `monthly_withdrawal` from `lump`
/// at `annual_return_pct`. Floors at 0.
pub fn lump_balance_after(lump: f64, monthly_withdrawal: f64, annual_return_pct: f64, years: u32) -> f64 {
    let r = annual_return_pct / 100.0;
    let monthly_r = r / 12.0;
    let mut bal = lump;
    for _ in 0..(years * 12) {
        bal *= 1.0 + monthly_r;
        bal -= monthly_withdrawal;
        if bal < 0.0 { return 0.0; }
    }
    bal
}

pub fn compute(input: &PensionInput) -> PensionReport {
    let years = if input.life_expectancy_age > input.current_age {
        input.life_expectancy_age - input.current_age
    } else { 0 };
    let pv = annuity_present_value(
        input.monthly_annuity_usd,
        input.expected_real_return_pct,
        years,
    );
    let implied = implied_internal_rate(
        input.lump_sum_usd, input.monthly_annuity_usd, years,
    );
    let runs_out = lump_runs_out_in_year(
        input.lump_sum_usd,
        input.monthly_annuity_usd,
        input.expected_real_return_pct,
    );
    let leftover = lump_balance_after(
        input.lump_sum_usd,
        input.monthly_annuity_usd,
        input.expected_real_return_pct,
        years,
    );
    let annuity_total = input.monthly_annuity_usd * 12.0 * years as f64;

    let (winner, advantage): (&'static str, f64) = if pv > input.lump_sum_usd {
        ("annuity", pv - input.lump_sum_usd)
    } else if input.lump_sum_usd > pv {
        ("lump", input.lump_sum_usd - pv)
    } else {
        ("tied", 0.0)
    };

    PensionReport {
        lump_sum_usd: input.lump_sum_usd,
        annuity_present_value_usd: pv,
        implied_internal_rate_pct: implied,
        lump_runs_out_year: runs_out,
        leftover_at_life_expectancy_usd: leftover,
        annuity_total_payments_usd: annuity_total,
        net_winner: winner,
        winner_advantage_usd: advantage,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> PensionInput {
        PensionInput {
            lump_sum_usd: 500_000.0,
            monthly_annuity_usd: 3_000.0,
            current_age: 65,
            life_expectancy_age: 85,
            expected_real_return_pct: 5.0,
        }
    }

    #[test]
    fn annuity_pv_zero_inputs() {
        assert_eq!(annuity_present_value(0.0, 5.0, 20), 0.0);
        assert_eq!(annuity_present_value(1000.0, 5.0, 0), 0.0);
    }

    #[test]
    fn annuity_pv_zero_return_linear() {
        // $1000/mo for 10 years at 0% = $120,000.
        assert_eq!(annuity_present_value(1000.0, 0.0, 10), 120_000.0);
    }

    #[test]
    fn annuity_pv_basic() {
        // $3000/mo for 20y at 5% ≈ $454,565 (standard published value)
        let pv = annuity_present_value(3000.0, 5.0, 20);
        assert!((pv - 454_565.0).abs() < 500.0, "got {pv}");
    }

    #[test]
    fn implied_internal_rate_zero_lump() {
        assert!(implied_internal_rate(0.0, 1000.0, 20).is_none());
    }

    #[test]
    fn implied_internal_rate_basic() {
        // Lump $500k for $3000/mo × 240 months. Implied rate solves PV.
        let r = implied_internal_rate(500_000.0, 3000.0, 20).unwrap();
        assert!(r > 1.0 && r < 8.0, "expected ~3-4%, got {r}");
    }

    #[test]
    fn lump_runs_out_at_zero_balance() {
        assert_eq!(lump_runs_out_in_year(0.0, 1000.0, 5.0), Some(0));
    }

    #[test]
    fn lump_runs_out_none_when_return_exceeds_withdrawal() {
        // $1M with $100/mo at 10% — never runs out (return > withdrawal)
        let y = lump_runs_out_in_year(1_000_000.0, 100.0, 10.0);
        assert!(y.is_none() || y.unwrap() > 50);
    }

    #[test]
    fn lump_runs_out_basic() {
        // $100k drawn at $1000/mo @ 0% = exactly 100 months = 9 years (ceiled)
        let y = lump_runs_out_in_year(100_000.0, 1000.0, 0.0);
        assert_eq!(y, Some(9));
    }

    #[test]
    fn lump_balance_after_zero_return_basic() {
        // $100k − ($1000/mo × 12 × 5) = $40k
        let b = lump_balance_after(100_000.0, 1000.0, 0.0, 5);
        assert_eq!(b, 40_000.0);
    }

    #[test]
    fn lump_balance_after_floors_zero() {
        // Lump too small to last 10 years.
        let b = lump_balance_after(10_000.0, 1000.0, 0.0, 10);
        assert_eq!(b, 0.0);
    }

    #[test]
    fn compute_winner_basic() {
        let r = compute(&input());
        assert!(r.net_winner == "lump" || r.net_winner == "annuity");
        assert!(r.winner_advantage_usd >= 0.0);
    }

    #[test]
    fn compute_annuity_wins_when_lump_too_low() {
        let mut i = input();
        i.lump_sum_usd = 100_000.0;
        let r = compute(&i);
        assert_eq!(r.net_winner, "annuity");
    }

    #[test]
    fn compute_lump_wins_when_lump_too_high() {
        let mut i = input();
        i.lump_sum_usd = 2_000_000.0;
        let r = compute(&i);
        assert_eq!(r.net_winner, "lump");
    }

    #[test]
    fn compute_total_annuity_basic() {
        let r = compute(&input());
        // $3000 × 12 × 20 = $720k
        assert_eq!(r.annuity_total_payments_usd, 720_000.0);
    }
}
