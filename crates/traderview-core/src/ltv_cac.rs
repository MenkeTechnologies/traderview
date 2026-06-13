//! LTV:CAC — customer unit economics.
//!
//! Whether acquiring customers pays off. Two numbers and their ratio:
//!
//!   * **LTV** (lifetime value) = monthly gross profit per customer × the
//!     average lifetime, where lifetime (months) = 100 / monthly churn %.
//!     Gross profit, not revenue — you keep only the margin.
//!   * **CAC** (customer acquisition cost) = sales + marketing spend per new
//!     customer.
//!   * **LTV:CAC ratio** = LTV / CAC. The rule of thumb is **3:1** — below
//!     ~1:1 you lose money on every customer; far above 3:1 you're likely
//!     under-investing in growth.
//!   * **CAC payback** = CAC / monthly gross profit — months to recoup the
//!     acquisition cost.
//!
//! Pure compute (no discounting — the standard simple LTV).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct LtvCacInput {
    pub avg_monthly_revenue_usd: f64,
    pub gross_margin_pct: f64,
    /// Monthly churn rate (e.g. 5 for 5%/mo). Must be > 0 for a finite LTV.
    pub monthly_churn_rate_pct: f64,
    /// Fully-loaded cost to acquire one customer.
    pub cac_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct LtvCacResult {
    /// Average customer lifetime in months (100 / churn%).
    pub avg_lifetime_months: f64,
    /// Monthly gross profit per customer (revenue × margin).
    pub monthly_gross_profit_usd: f64,
    pub ltv_usd: f64,
    pub ltv_cac_ratio: f64,
    /// Months of gross profit to recoup the CAC.
    pub cac_payback_months: f64,
    /// "healthy" (≥3), "marginal" (1–3), or "unprofitable" (<1).
    pub rating: String,
}

pub fn analyze(i: &LtvCacInput) -> LtvCacResult {
    let lifetime = if i.monthly_churn_rate_pct > 0.0 {
        100.0 / i.monthly_churn_rate_pct
    } else {
        0.0
    };
    let monthly_gp = i.avg_monthly_revenue_usd * i.gross_margin_pct / 100.0;
    let ltv = monthly_gp * lifetime;
    let ratio = if i.cac_usd > 0.0 { ltv / i.cac_usd } else { 0.0 };
    let payback = if monthly_gp > 0.0 { i.cac_usd / monthly_gp } else { 0.0 };

    let rating = if i.cac_usd <= 0.0 {
        "n/a"
    } else if ratio >= 3.0 {
        "healthy"
    } else if ratio >= 1.0 {
        "marginal"
    } else {
        "unprofitable"
    };

    LtvCacResult {
        avg_lifetime_months: lifetime,
        monthly_gross_profit_usd: monthly_gp,
        ltv_usd: ltv,
        ltv_cac_ratio: ratio,
        cac_payback_months: payback,
        rating: rating.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> LtvCacInput {
        LtvCacInput {
            avg_monthly_revenue_usd: 100.0,
            gross_margin_pct: 80.0,
            monthly_churn_rate_pct: 5.0,
            cac_usd: 400.0,
        }
    }

    #[test]
    fn lifetime_is_inverse_of_churn() {
        // 100 / 5% = 20 months.
        let r = analyze(&base());
        assert!((r.avg_lifetime_months - 20.0).abs() < 1e-9);
    }

    #[test]
    fn monthly_gross_profit_applies_margin() {
        let r = analyze(&base());
        assert!((r.monthly_gross_profit_usd - 80.0).abs() < 1e-9); // 100 × 80%
    }

    #[test]
    fn ltv_is_gross_profit_times_lifetime() {
        let r = analyze(&base());
        assert!((r.ltv_usd - 1_600.0).abs() < 1e-9); // 80 × 20
    }

    #[test]
    fn ratio_and_healthy_rating() {
        // 1600 / 400 = 4.0 → healthy (≥3).
        let r = analyze(&base());
        assert!((r.ltv_cac_ratio - 4.0).abs() < 1e-9);
        assert_eq!(r.rating, "healthy");
    }

    #[test]
    fn cac_payback_months() {
        // 400 / 80 = 5 months.
        let r = analyze(&base());
        assert!((r.cac_payback_months - 5.0).abs() < 1e-9);
    }

    #[test]
    fn marginal_and_unprofitable_ratings() {
        // ratio 2 → marginal (cac 800).
        let marg = analyze(&LtvCacInput { cac_usd: 800.0, ..base() });
        assert_eq!(marg.rating, "marginal");
        // ratio 0.8 → unprofitable (cac 2000).
        let bad = analyze(&LtvCacInput { cac_usd: 2_000.0, ..base() });
        assert_eq!(bad.rating, "unprofitable");
    }

    #[test]
    fn zero_churn_guards_lifetime() {
        let r = analyze(&LtvCacInput { monthly_churn_rate_pct: 0.0, ..base() });
        assert!(r.avg_lifetime_months.abs() < 1e-9);
        assert!(r.ltv_usd.abs() < 1e-9);
    }

    #[test]
    fn zero_cac_guards_ratio() {
        let r = analyze(&LtvCacInput { cac_usd: 0.0, ..base() });
        assert!(r.ltv_cac_ratio.abs() < 1e-9);
        assert_eq!(r.rating, "n/a");
    }
}
