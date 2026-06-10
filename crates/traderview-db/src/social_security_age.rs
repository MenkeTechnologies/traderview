//! Social Security claiming-age optimizer.
//!
//! Standard SS benefit reductions / credits per SSA rules:
//!
//!   - Full Retirement Age (FRA): 67 for those born 1960+
//!     (66+2/12 for 1955, 66+10/12 for 1959, etc. — we use 67 default).
//!   - Claim at 62 (earliest): permanent ~30% reduction from FRA benefit
//!     (5/9% per month × 36 months + 5/12% per month × 24 months).
//!   - Claim BEFORE FRA but after 62: linear-ish reduction.
//!   - Claim AFTER FRA up to 70: Delayed Retirement Credit (DRC)
//!     8% per year (= 2/3% per month).
//!   - Claim after 70: no further increase.
//!
//! Inputs:
//!   - fra_monthly_benefit_usd  — your projected monthly benefit AT
//!     Full Retirement Age (from the SSA statement)
//!   - fra_age                  — typically 67 (66 for 1943-1954 birth)
//!   - claim_age_a / claim_age_b — two ages to compare
//!   - life_expectancy_age      — typically 85 for breakeven analysis
//!
//! Compute returns per-claim-age: monthly benefit, annual benefit,
//! total lifetime benefits up to life expectancy. Plus breakeven age
//! between the two start ages.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SocialSecurityInput {
    pub fra_monthly_benefit_usd: f64,
    #[serde(default = "default_fra")]
    pub fra_age: u32,
    pub claim_age_a: u32,
    pub claim_age_b: u32,
    #[serde(default = "default_life")]
    pub life_expectancy_age: u32,
}

fn default_fra() -> u32 { 67 }
fn default_life() -> u32 { 85 }

#[derive(Debug, Clone, Serialize)]
pub struct ClaimResult {
    pub claim_age: u32,
    pub monthly_benefit_usd: f64,
    pub annual_benefit_usd: f64,
    pub lifetime_total_to_life_expectancy_usd: f64,
    pub pct_of_fra: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SocialSecurityReport {
    pub fra_monthly_benefit_usd: f64,
    pub claim_a: ClaimResult,
    pub claim_b: ClaimResult,
    pub breakeven_age: Option<u32>,
    pub net_winner_at_life_expectancy: &'static str,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

/// Monthly benefit at `claim_age` as multiple of FRA benefit, per SSA rules.
pub fn benefit_multiplier(claim_age: u32, fra_age: u32) -> f64 {
    if claim_age == fra_age { return 1.0; }
    if claim_age > fra_age {
        // Delayed Retirement Credit: 8% per year up to 70, capped at 70.
        let capped = claim_age.min(70);
        let extra_years = (capped - fra_age) as f64;
        return 1.0 + extra_years * 0.08;
    }
    // Early reduction. SSA formula:
    //   First 36 months early: 5/9% per month = 6.667%/yr
    //   Beyond 36 months early: 5/12% per month = 5%/yr
    let months_early = (fra_age - claim_age) * 12;
    let first36 = months_early.min(36) as f64;
    let beyond = months_early.saturating_sub(36) as f64;
    let reduction = first36 * (5.0 / 9.0) / 100.0 + beyond * (5.0 / 12.0) / 100.0;
    (1.0 - reduction).max(0.0)
}

pub fn benefit_at(claim_age: u32, fra_monthly: f64, fra_age: u32) -> f64 {
    fra_monthly * benefit_multiplier(claim_age, fra_age)
}

pub fn lifetime_total(claim_age: u32, monthly: f64, life_expectancy_age: u32) -> f64 {
    if life_expectancy_age <= claim_age { return 0.0; }
    let years = (life_expectancy_age - claim_age) as f64;
    monthly * 12.0 * years
}

/// Year (age) at which cumulative B catches up with cumulative A
/// (or vice versa). None if they never cross within reasonable range.
pub fn breakeven_age(
    age_a: u32, monthly_a: f64,
    age_b: u32, monthly_b: f64,
) -> Option<u32> {
    // Higher monthly benefit is the "later claim", lower is "earlier claim".
    // The earlier claim has a head start. The later claim catches up at
    // the breakeven age.
    let (early_age, early_monthly, late_age, late_monthly) =
        if age_a < age_b { (age_a, monthly_a, age_b, monthly_b) }
        else { (age_b, monthly_b, age_a, monthly_a) };
    if late_monthly <= early_monthly + 1e-9 { return None; }
    // At late_age, early has accumulated (late − early) years × 12 months × early_monthly.
    // After that, the late starts; each subsequent month the late earns
    // (late_monthly − early_monthly) net catch-up.
    let head_start = (late_age - early_age) as f64 * 12.0 * early_monthly;
    let net_monthly = late_monthly - early_monthly;
    let months_to_catch = head_start / net_monthly;
    let years = (months_to_catch / 12.0).ceil() as u32;
    Some(late_age + years)
}

pub fn compute(input: &SocialSecurityInput) -> SocialSecurityReport {
    let mk = |age: u32| -> ClaimResult {
        let mult = benefit_multiplier(age, input.fra_age);
        let m = input.fra_monthly_benefit_usd * mult;
        let a = m * 12.0;
        let total = lifetime_total(age, m, input.life_expectancy_age);
        ClaimResult {
            claim_age: age,
            monthly_benefit_usd: m,
            annual_benefit_usd: a,
            lifetime_total_to_life_expectancy_usd: total,
            pct_of_fra: mult * 100.0,
        }
    };
    let ca = mk(input.claim_age_a);
    let cb = mk(input.claim_age_b);
    let breakeven = breakeven_age(
        ca.claim_age, ca.monthly_benefit_usd,
        cb.claim_age, cb.monthly_benefit_usd,
    );
    let winner: &'static str =
        if ca.lifetime_total_to_life_expectancy_usd > cb.lifetime_total_to_life_expectancy_usd {
            "claim_a"
        } else if cb.lifetime_total_to_life_expectancy_usd > ca.lifetime_total_to_life_expectancy_usd {
            "claim_b"
        } else {
            "tied"
        };
    SocialSecurityReport {
        fra_monthly_benefit_usd: input.fra_monthly_benefit_usd,
        claim_a: ca,
        claim_b: cb,
        breakeven_age: breakeven,
        net_winner_at_life_expectancy: winner,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn benefit_multiplier_at_fra() {
        assert_eq!(benefit_multiplier(67, 67), 1.0);
    }

    #[test]
    fn benefit_multiplier_age_62_drops_30pct() {
        // 5 years early = 60 months. First 36 @ 5/9% + 24 @ 5/12% = 30%
        let m = benefit_multiplier(62, 67);
        assert!((m - 0.70).abs() < 1e-3, "got {m}");
    }

    #[test]
    fn benefit_multiplier_age_70_adds_24pct() {
        // 3 years × 8% = 24%
        let m = benefit_multiplier(70, 67);
        assert!((m - 1.24).abs() < 1e-9);
    }

    #[test]
    fn benefit_multiplier_age_75_caps_at_70_credit() {
        // Same as 70 — no further DRC after 70.
        assert_eq!(benefit_multiplier(75, 67), benefit_multiplier(70, 67));
    }

    #[test]
    fn benefit_multiplier_age_66_one_year_early() {
        // 12 months early × 5/9% = 6.667%
        let m = benefit_multiplier(66, 67);
        assert!((m - (1.0 - 12.0 * 5.0 / 9.0 / 100.0)).abs() < 1e-9);
    }

    #[test]
    fn benefit_at_fra_basic() {
        assert_eq!(benefit_at(67, 2_500.0, 67), 2_500.0);
    }

    #[test]
    fn benefit_at_age_70_higher_than_fra() {
        let fra = benefit_at(67, 2_500.0, 67);
        let seventy = benefit_at(70, 2_500.0, 67);
        assert!(seventy > fra);
    }

    #[test]
    fn lifetime_total_basic() {
        // Claim at 65, $1000/mo, live to 85 = 20 years × 12k = $240k
        assert_eq!(lifetime_total(65, 1_000.0, 85), 240_000.0);
    }

    #[test]
    fn lifetime_total_zero_when_life_below_claim() {
        assert_eq!(lifetime_total(65, 1_000.0, 60), 0.0);
    }

    #[test]
    fn breakeven_age_known_62_vs_70() {
        // Classic published number: 62 vs 70 breakeven ~80-81.
        // FRA $2000 → 62 = $1400, 70 = $2480.
        let be = breakeven_age(62, 1_400.0, 70, 2_480.0);
        assert!(be.is_some());
        let age = be.unwrap();
        assert!(age >= 79 && age <= 82, "expected ~80, got {age}");
    }

    #[test]
    fn breakeven_age_none_when_equal_benefits() {
        assert!(breakeven_age(62, 1000.0, 65, 1000.0).is_none());
    }

    #[test]
    fn compute_full_report_62_vs_70_with_85_life() {
        let r = compute(&SocialSecurityInput {
            fra_monthly_benefit_usd: 2_000.0,
            fra_age: 67,
            claim_age_a: 62,
            claim_age_b: 70,
            life_expectancy_age: 85,
        });
        assert!((r.claim_a.monthly_benefit_usd - 1_400.0).abs() < 1.0);
        assert!((r.claim_b.monthly_benefit_usd - 2_480.0).abs() < 1.0);
        assert!(r.breakeven_age.is_some());
        // With life to 85, claim_b (age 70) should win.
        assert_eq!(r.net_winner_at_life_expectancy, "claim_b");
    }

    #[test]
    fn compute_winner_flips_with_early_death() {
        let r = compute(&SocialSecurityInput {
            fra_monthly_benefit_usd: 2_000.0,
            fra_age: 67,
            claim_age_a: 62,
            claim_age_b: 70,
            life_expectancy_age: 75,  // earlier than breakeven
        });
        assert_eq!(r.net_winner_at_life_expectancy, "claim_a");
    }
}
