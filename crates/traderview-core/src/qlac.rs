//! QLAC — Qualified Longevity Annuity Contract RMD deferral.
//!
//! A QLAC lets you move a capped amount out of an IRA/401(k) into a deferred
//! income annuity that starts paying later (up to age 85). The money in the
//! QLAC is **excluded from the RMD base** until the annuity starts, so it
//! lowers the required minimum distribution (and the tax) during the
//! deferral years.
//!
//!   * The premium is capped at the SECURE 2.0 limit — a flat $200,000 base,
//!     inflation-indexed to **$210,000 for 2025–2026** (lifetime, per person,
//!     across all accounts). It can't exceed the account balance either.
//!   * RMD = (account balance − QLAC premium) / the Uniform Lifetime Table
//!     divisor for your age (age 73 = 26.5).
//!   * The RMD reduction = QLAC premium / divisor.
//!
//! Pure compute. The premium limit and the divisor are inputs so the calc
//! survives future inflation adjustments and any age.

use serde::{Deserialize, Serialize};

/// SECURE 2.0 QLAC premium cap for 2025–2026 (base $200,000 indexed).
pub const DEFAULT_PREMIUM_LIMIT_USD: f64 = 210_000.0;

#[derive(Debug, Clone, Deserialize)]
pub struct QlacInput {
    pub account_balance_usd: f64,
    /// Amount you want to move into the QLAC.
    pub qlac_premium_usd: f64,
    /// Premium cap (defaults to the current $210,000 if 0 is passed).
    #[serde(default)]
    pub premium_limit_usd: f64,
    /// Uniform Lifetime Table divisor for your age (age 73 = 26.5).
    pub rmd_divisor: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct QlacResult {
    pub premium_limit_usd: f64,
    /// Premium actually allowed = min(requested, limit, balance).
    pub premium_allowed_usd: f64,
    /// True when the requested premium exceeds the limit.
    pub over_limit: bool,
    pub rmd_without_qlac_usd: f64,
    pub rmd_with_qlac_usd: f64,
    /// Annual RMD reduction from the QLAC = premium / divisor.
    pub annual_rmd_reduction_usd: f64,
}

pub fn analyze(i: &QlacInput) -> QlacResult {
    let balance = i.account_balance_usd.max(0.0);
    let limit = if i.premium_limit_usd > 0.0 { i.premium_limit_usd } else { DEFAULT_PREMIUM_LIMIT_USD };
    let requested = i.qlac_premium_usd.max(0.0);

    let over_limit = requested > limit;
    // Can't move more than the limit, nor more than the account holds.
    let allowed = requested.min(limit).min(balance);

    let (rmd_without, rmd_with) = if i.rmd_divisor > 0.0 {
        (balance / i.rmd_divisor, (balance - allowed) / i.rmd_divisor)
    } else {
        (0.0, 0.0)
    };
    let reduction = rmd_without - rmd_with;

    QlacResult {
        premium_limit_usd: limit,
        premium_allowed_usd: allowed,
        over_limit,
        rmd_without_qlac_usd: rmd_without,
        rmd_with_qlac_usd: rmd_with,
        annual_rmd_reduction_usd: reduction,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> QlacInput {
        QlacInput {
            account_balance_usd: 1_000_000.0,
            qlac_premium_usd: 210_000.0,
            premium_limit_usd: 0.0, // use default
            rmd_divisor: 26.5, // age 73
        }
    }

    #[test]
    fn default_limit_is_210k() {
        let r = analyze(&base());
        assert!((r.premium_limit_usd - 210_000.0).abs() < 1e-9);
    }

    #[test]
    fn premium_capped_at_limit() {
        let r = analyze(&QlacInput { qlac_premium_usd: 250_000.0, ..base() });
        assert!(r.over_limit);
        assert!((r.premium_allowed_usd - 210_000.0).abs() < 1e-9);
    }

    #[test]
    fn under_limit_allows_full_request() {
        let r = analyze(&QlacInput { qlac_premium_usd: 150_000.0, ..base() });
        assert!(!r.over_limit);
        assert!((r.premium_allowed_usd - 150_000.0).abs() < 1e-9);
    }

    #[test]
    fn rmd_without_qlac_is_balance_over_divisor() {
        // 1,000,000 / 26.5 = 37,735.849…
        let r = analyze(&base());
        assert!((r.rmd_without_qlac_usd - 1_000_000.0 / 26.5).abs() < 1e-6);
    }

    #[test]
    fn rmd_with_qlac_excludes_the_premium() {
        // (1,000,000 − 210,000) / 26.5 = 790,000 / 26.5.
        let r = analyze(&base());
        assert!((r.rmd_with_qlac_usd - 790_000.0 / 26.5).abs() < 1e-6);
    }

    #[test]
    fn reduction_is_premium_over_divisor_and_consistent() {
        let r = analyze(&base());
        assert!((r.annual_rmd_reduction_usd - 210_000.0 / 26.5).abs() < 1e-6);
        assert!((r.annual_rmd_reduction_usd - (r.rmd_without_qlac_usd - r.rmd_with_qlac_usd)).abs() < 1e-6);
    }

    #[test]
    fn premium_cannot_exceed_balance() {
        // Tiny account, big request → allowed capped at the balance.
        let r = analyze(&QlacInput {
            account_balance_usd: 50_000.0,
            qlac_premium_usd: 210_000.0,
            ..base()
        });
        assert!((r.premium_allowed_usd - 50_000.0).abs() < 1e-9);
    }

    #[test]
    fn custom_limit_overrides_default() {
        let r = analyze(&QlacInput { premium_limit_usd: 200_000.0, qlac_premium_usd: 205_000.0, ..base() });
        assert!((r.premium_limit_usd - 200_000.0).abs() < 1e-9);
        assert!(r.over_limit);
        assert!((r.premium_allowed_usd - 200_000.0).abs() < 1e-9);
    }
}
