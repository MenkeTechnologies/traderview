//! Pension survivor election — single-life vs joint-and-survivor.
//!
//! At retirement a pension forces a choice: a **single-life** annuity pays
//! the most but stops when the retiree dies (the surviving spouse gets
//! nothing), or a **joint-and-survivor** (J&S) annuity pays less now but
//! continues — often at a reduced percentage (50/75/100%) — to the
//! survivor. The reduction is the price of that survivor protection.
//!
//! The **pension-max** alternative: take the higher single-life payment and
//! buy life insurance to protect the spouse instead. If the single-life
//! payment minus the insurance premium still beats the J&S payment, pension
//! max gives more income now AND protects the survivor.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct PensionSurvivorInput {
    pub single_life_monthly_usd: f64,
    /// J&S monthly payment while both spouses are alive.
    pub joint_survivor_monthly_usd: f64,
    /// Percent the survivor continues to receive after the retiree dies.
    pub survivor_pct: f64,
    /// Monthly life-insurance premium for the pension-max comparison (0 = skip).
    #[serde(default)]
    pub life_insurance_premium_monthly_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct PensionSurvivorResult {
    /// Single-life − J&S: the monthly cost of survivor protection.
    pub monthly_reduction_usd: f64,
    pub annual_reduction_usd: f64,
    /// Reduction as a percent of the single-life payment.
    pub reduction_pct: f64,
    /// What the survivor receives after the retiree dies (J&S × survivor%).
    pub survivor_monthly_benefit_usd: f64,
    /// Single-life payment minus the insurance premium (pension-max net).
    pub pension_max_net_monthly_usd: f64,
    /// True when pension-max net income beats the J&S payment.
    pub pension_max_better: bool,
}

pub fn analyze(i: &PensionSurvivorInput) -> PensionSurvivorResult {
    let single = i.single_life_monthly_usd;
    let joint = i.joint_survivor_monthly_usd;

    let monthly_reduction = single - joint;
    let reduction_pct = if single > 0.0 { monthly_reduction / single * 100.0 } else { 0.0 };
    let survivor_benefit = joint * i.survivor_pct / 100.0;

    let pension_max_net = single - i.life_insurance_premium_monthly_usd;
    let pension_max_better = pension_max_net > joint;

    PensionSurvivorResult {
        monthly_reduction_usd: monthly_reduction,
        annual_reduction_usd: monthly_reduction * 12.0,
        reduction_pct,
        survivor_monthly_benefit_usd: survivor_benefit,
        pension_max_net_monthly_usd: pension_max_net,
        pension_max_better,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> PensionSurvivorInput {
        PensionSurvivorInput {
            single_life_monthly_usd: 3_000.0,
            joint_survivor_monthly_usd: 2_600.0,
            survivor_pct: 50.0,
            life_insurance_premium_monthly_usd: 0.0,
        }
    }

    #[test]
    fn monthly_reduction_is_single_minus_joint() {
        let r = analyze(&base());
        assert!((r.monthly_reduction_usd - 400.0).abs() < 1e-9);
        assert!((r.annual_reduction_usd - 4_800.0).abs() < 1e-9);
    }

    #[test]
    fn reduction_pct_of_single_life() {
        // 400 / 3000 = 13.33%.
        let r = analyze(&base());
        assert!((r.reduction_pct - (400.0 / 3_000.0 * 100.0)).abs() < 1e-9);
    }

    #[test]
    fn survivor_benefit_is_joint_times_pct() {
        // 2600 × 50% = 1300.
        let r = analyze(&base());
        assert!((r.survivor_monthly_benefit_usd - 1_300.0).abs() < 1e-9);
    }

    #[test]
    fn hundred_pct_survivor_continues_full_joint() {
        let r = analyze(&PensionSurvivorInput { survivor_pct: 100.0, ..base() });
        assert!((r.survivor_monthly_benefit_usd - 2_600.0).abs() < 1e-9);
    }

    #[test]
    fn pension_max_net_subtracts_premium() {
        // 3000 − 250 premium = 2750.
        let r = analyze(&PensionSurvivorInput { life_insurance_premium_monthly_usd: 250.0, ..base() });
        assert!((r.pension_max_net_monthly_usd - 2_750.0).abs() < 1e-9);
        assert!(r.pension_max_better); // 2750 > 2600 joint
    }

    #[test]
    fn expensive_insurance_flips_pension_max() {
        // Premium 500 → net 2500 < 2600 joint → J&S wins.
        let r = analyze(&PensionSurvivorInput { life_insurance_premium_monthly_usd: 500.0, ..base() });
        assert!((r.pension_max_net_monthly_usd - 2_500.0).abs() < 1e-9);
        assert!(!r.pension_max_better);
    }

    #[test]
    fn no_insurance_pension_max_always_beats_joint() {
        // Premium 0 → net = single 3000 > joint 2600.
        let r = analyze(&base());
        assert!(r.pension_max_better);
    }

    #[test]
    fn equal_payments_zero_reduction() {
        let r = analyze(&PensionSurvivorInput { joint_survivor_monthly_usd: 3_000.0, ..base() });
        assert!(r.monthly_reduction_usd.abs() < 1e-9);
        assert!(r.reduction_pct.abs() < 1e-9);
    }
}
