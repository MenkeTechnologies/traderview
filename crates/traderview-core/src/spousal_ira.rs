//! Spousal IRA — contribution eligibility for a non-working spouse.
//!
//! Normally an IRA contribution requires earned income. The spousal-IRA rule
//! lets a married couple filing jointly fund an IRA for a spouse with little
//! or no income, as long as the **couple's combined earned income covers the
//! total of both spouses' contributions**. Each spouse is still capped at the
//! annual per-person limit (plus a catch-up at age 50+).
//!
//!   * per-spouse limit = base limit + (age 50+ ? catch-up : 0)
//!   * each contribution is capped at that spouse's limit
//!   * the two contributions together can't exceed combined earned income
//!
//! 2026 limits (web-verified, irs.gov): base $7,500, catch-up $1,100. Both
//! are inputs so the calc survives future indexing. Pure compute.

use serde::{Deserialize, Serialize};

/// 2026 IRA base contribution limit.
pub const DEFAULT_BASE_LIMIT_USD: f64 = 7_500.0;
/// 2026 age-50+ catch-up.
pub const DEFAULT_CATCH_UP_USD: f64 = 1_100.0;

#[derive(Debug, Clone, Deserialize)]
pub struct SpousalIraInput {
    /// The couple's combined earned income (often the working spouse's).
    pub combined_earned_income_usd: f64,
    pub working_spouse_contribution_usd: f64,
    pub nonworking_spouse_contribution_usd: f64,
    #[serde(default)]
    pub working_spouse_50plus: bool,
    #[serde(default)]
    pub nonworking_spouse_50plus: bool,
    /// Per-person base limit (defaults to the current $7,500 if 0).
    #[serde(default)]
    pub base_limit_usd: f64,
    /// Age-50+ catch-up (defaults to $1,100 if 0).
    #[serde(default)]
    pub catch_up_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpousalIraResult {
    pub working_spouse_limit_usd: f64,
    pub nonworking_spouse_limit_usd: f64,
    /// Working contribution after the per-person cap.
    pub working_allowed_usd: f64,
    /// Non-working contribution after the per-person cap.
    pub nonworking_allowed_usd: f64,
    /// The two capped contributions summed.
    pub combined_contribution_usd: f64,
    /// Most the couple could contribute = min(sum of limits, earned income).
    pub max_combined_usd: f64,
    /// True when earned income covers the (capped) contributions.
    pub income_sufficient: bool,
}

pub fn analyze(i: &SpousalIraInput) -> SpousalIraResult {
    let base = if i.base_limit_usd > 0.0 { i.base_limit_usd } else { DEFAULT_BASE_LIMIT_USD };
    let catch_up = if i.catch_up_usd > 0.0 { i.catch_up_usd } else { DEFAULT_CATCH_UP_USD };

    let working_limit = base + if i.working_spouse_50plus { catch_up } else { 0.0 };
    let nonworking_limit = base + if i.nonworking_spouse_50plus { catch_up } else { 0.0 };

    let working_allowed = i.working_spouse_contribution_usd.max(0.0).min(working_limit);
    let nonworking_allowed = i.nonworking_spouse_contribution_usd.max(0.0).min(nonworking_limit);
    let combined = working_allowed + nonworking_allowed;

    let income = i.combined_earned_income_usd.max(0.0);
    let max_combined = (working_limit + nonworking_limit).min(income);
    let income_sufficient = income >= combined;

    SpousalIraResult {
        working_spouse_limit_usd: working_limit,
        nonworking_spouse_limit_usd: nonworking_limit,
        working_allowed_usd: working_allowed,
        nonworking_allowed_usd: nonworking_allowed,
        combined_contribution_usd: combined,
        max_combined_usd: max_combined,
        income_sufficient,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> SpousalIraInput {
        SpousalIraInput {
            combined_earned_income_usd: 80_000.0,
            working_spouse_contribution_usd: 7_500.0,
            nonworking_spouse_contribution_usd: 7_500.0,
            working_spouse_50plus: false,
            nonworking_spouse_50plus: false,
            base_limit_usd: 0.0,
            catch_up_usd: 0.0,
        }
    }

    #[test]
    fn default_limit_under_50() {
        let r = analyze(&base());
        assert!((r.working_spouse_limit_usd - 7_500.0).abs() < 1e-9);
        assert!((r.nonworking_spouse_limit_usd - 7_500.0).abs() < 1e-9);
    }

    #[test]
    fn catch_up_only_for_50plus_spouse() {
        let r = analyze(&SpousalIraInput { working_spouse_50plus: true, ..base() });
        assert!((r.working_spouse_limit_usd - 8_600.0).abs() < 1e-9); // 7500 + 1100
        assert!((r.nonworking_spouse_limit_usd - 7_500.0).abs() < 1e-9); // unchanged
    }

    #[test]
    fn contribution_capped_at_limit() {
        let r = analyze(&SpousalIraInput { working_spouse_contribution_usd: 10_000.0, ..base() });
        assert!((r.working_allowed_usd - 7_500.0).abs() < 1e-9);
    }

    #[test]
    fn nonworking_spouse_funded_on_working_income() {
        // The whole point: nonworking spouse contributes on the couple's income.
        let r = analyze(&base());
        assert!((r.nonworking_allowed_usd - 7_500.0).abs() < 1e-9);
        assert!(r.income_sufficient); // 80k >> 15k combined
    }

    #[test]
    fn combined_is_sum_of_capped_contributions() {
        let r = analyze(&base());
        assert!((r.combined_contribution_usd - 15_000.0).abs() < 1e-9);
    }

    #[test]
    fn income_insufficient_when_below_contributions() {
        // Only $5k earned income → can't fund $15k of contributions.
        let r = analyze(&SpousalIraInput { combined_earned_income_usd: 5_000.0, ..base() });
        assert!(!r.income_sufficient);
        assert!((r.max_combined_usd - 5_000.0).abs() < 1e-9); // capped by income
    }

    #[test]
    fn max_combined_is_min_of_limits_and_income() {
        // Ample income → max = sum of limits (15k).
        let r = analyze(&base());
        assert!((r.max_combined_usd - 15_000.0).abs() < 1e-9);
    }

    #[test]
    fn both_50plus_doubles_catch_up() {
        let r = analyze(&SpousalIraInput {
            working_spouse_50plus: true,
            nonworking_spouse_50plus: true,
            ..base()
        });
        assert!((r.working_spouse_limit_usd - 8_600.0).abs() < 1e-9);
        assert!((r.nonworking_spouse_limit_usd - 8_600.0).abs() < 1e-9);
    }
}
