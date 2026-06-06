//! IRC §243 / §246 — Dividends Received Deduction (DRD).
//!
//! Allows a C-corporation receiving dividends from another domestic
//! corporation to deduct a percentage of those dividends from taxable
//! income. The percentage depends on the receiving corporation's
//! OWNERSHIP STAKE in the paying corporation; further reduced if the
//! stock was debt-financed (§246A) or held for too short a period
//! (§246(c)).
//!
//! **§243(a)(1) ownership tiers** (post-TCJA percentages):
//!
//! | Ownership stake | DRD percentage | Citation                           |
//! |-----------------|----------------|------------------------------------|
//! | < 20%           | 50%            | §243(a)(1)                         |
//! | 20% - 79%       | 65%            | §243(c) (20-percent owned corp)    |
//! | ≥ 80%           | 100%           | §243(b) (qualifying group)         |
//!
//! Pre-TCJA the percentages were 70% / 80% / 100%. TCJA (P.L.
//! 115-97 § 13002) lowered the 70% to 50% and the 80% to 65% to
//! preserve overall corporate tax rates after the §11 rate dropped
//! from 35% to 21%.
//!
//! **§246(c) anti-abuse holding period** — the receiving corporation
//! must hold the stock for more than:
//!
//! - **45 days** during the **91-day period** beginning 45 days
//!   BEFORE the ex-dividend date, for COMMON stock
//! - **90 days** during the **181-day period** beginning 90 days
//!   BEFORE the ex-dividend date, for PREFERRED stock if the
//!   dividends are attributable to periods aggregating MORE than
//!   **366 days**
//!
//! Failure to satisfy the holding period disallows the DRD ENTIRELY
//! for that dividend — not partial. Prevents traders from buying
//! stock just before ex-dividend, collecting the dividend with
//! preferential DRD treatment, then selling.
//!
//! **§246A debt-financed portfolio stock reduction** — if the stock
//! was debt-financed during the base period, the DRD percentage is
//! reduced by the average indebtedness percentage:
//!
//! ```text
//! effective DRD % = base DRD % × (100% − average indebtedness %)
//! ```
//!
//! So a 50%-tier dividend on 40%-debt-financed stock yields only
//! 50% × 60% = 30% DRD. Applies to "portfolio stock" — generally
//! stock that the corp does NOT hold for active business purposes.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StockType {
    CommonStock,
    /// Preferred stock with dividends attributable to ≤ 366 days —
    /// uses the COMMON-stock 46-day window.
    PreferredStockShortDividend,
    /// Preferred stock with dividends attributable to > 366 days —
    /// uses the extended 91-day window.
    PreferredStockLongDividend,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnershipTier {
    UnderTwentyPct,
    TwentyToSeventyNinePct,
    EightyPlusPct,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section243Input {
    pub dividend_received_dollars: i64,
    /// Ownership percentage of the dividend-paying corporation, in
    /// basis points (2000 = 20%, 8000 = 80%).
    pub ownership_pct_bp: u32,
    pub stock_type: StockType,
    /// Number of days the corp held the stock within the relevant
    /// statutory window (91-day for common / short-dividend
    /// preferred; 181-day for long-dividend preferred).
    pub holding_days_in_statutory_window: u32,
    /// True if any portion of the stock was debt-financed during the
    /// base period (§246A).
    pub stock_is_debt_financed_portfolio: bool,
    /// Average indebtedness percentage, in basis points (4000 = 40%).
    /// Only used when `stock_is_debt_financed_portfolio: true`.
    pub average_indebtedness_pct_bp: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section243Result {
    pub ownership_tier: OwnershipTier,
    pub base_drd_pct_bp: u32,
    pub holding_period_satisfied: bool,
    pub holding_period_required_days: u32,
    pub debt_financed_reduction_applies: bool,
    pub effective_drd_pct_bp: u32,
    pub drd_amount_dollars: i64,
    pub citation: String,
    pub note: String,
}

pub fn compute(input: &Section243Input) -> Section243Result {
    // §243 ownership tier.
    let tier = if input.ownership_pct_bp < 2000 {
        OwnershipTier::UnderTwentyPct
    } else if input.ownership_pct_bp < 8000 {
        OwnershipTier::TwentyToSeventyNinePct
    } else {
        OwnershipTier::EightyPlusPct
    };
    let base_pct_bp: u32 = match tier {
        OwnershipTier::UnderTwentyPct => 5000,
        OwnershipTier::TwentyToSeventyNinePct => 6500,
        OwnershipTier::EightyPlusPct => 10_000,
    };

    // §246(c) holding period: > 45 days for common / short-preferred;
    // > 90 days for long-preferred.
    let required_days = match input.stock_type {
        StockType::CommonStock | StockType::PreferredStockShortDividend => 45,
        StockType::PreferredStockLongDividend => 90,
    };
    let holding_satisfied = input.holding_days_in_statutory_window > required_days;

    // §246A debt-financed reduction.
    let debt_reduction =
        input.stock_is_debt_financed_portfolio && tier != OwnershipTier::EightyPlusPct;
    let effective_pct_bp = if !holding_satisfied {
        0 // §246(c) full disallowance
    } else if debt_reduction {
        // effective = base × (100% − indebtedness%)
        let indebt = input.average_indebtedness_pct_bp.min(10_000);
        let multiplier = 10_000u64 - indebt as u64; // basis points
        ((base_pct_bp as u64) * multiplier / 10_000) as u32
    } else {
        base_pct_bp
    };

    let drd_dollars =
        ((input.dividend_received_dollars as i128) * (effective_pct_bp as i128) / 10_000) as i64;

    let note = if !holding_satisfied {
        format!(
            "§246(c) HOLDING PERIOD FAILED: only {} days within statutory window (>{} required for {:?}); DRD ENTIRELY DISALLOWED.",
            input.holding_days_in_statutory_window,
            required_days,
            input.stock_type,
        )
    } else {
        let mut parts = vec![format!(
            "§243 {:?} tier: base DRD {}.{:02}%",
            tier,
            base_pct_bp / 100,
            base_pct_bp % 100,
        )];
        if debt_reduction {
            parts.push(format!(
                "§246A debt-financed reduction: average indebtedness {}.{:02}% → effective {}.{:02}%",
                input.average_indebtedness_pct_bp / 100,
                input.average_indebtedness_pct_bp % 100,
                effective_pct_bp / 100,
                effective_pct_bp % 100,
            ));
        }
        parts.push(format!(
            "DRD = {}.{:02}% × ${} = ${}",
            effective_pct_bp / 100,
            effective_pct_bp % 100,
            input.dividend_received_dollars,
            drd_dollars,
        ));
        parts.join("; ")
    };

    Section243Result {
        ownership_tier: tier,
        base_drd_pct_bp: base_pct_bp,
        holding_period_satisfied: holding_satisfied,
        holding_period_required_days: required_days,
        debt_financed_reduction_applies: debt_reduction,
        effective_drd_pct_bp: effective_pct_bp,
        drd_amount_dollars: drd_dollars,
        citation:
            "IRC §243(a)(1) 50% DRD baseline; §243(c) 65% DRD for 20%-owned; §243(b) 100% DRD qualifying group; §246(c) holding-period rule (>45 days/91-day window common; >90 days/181-day window long-preferred); §246A debt-financed portfolio stock reduction; TCJA (P.L. 115-97 §13002) lowered DRD percentages"
                .to_string(),
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Section243Input {
        Section243Input {
            dividend_received_dollars: 100_000,
            ownership_pct_bp: 500, // 5%
            stock_type: StockType::CommonStock,
            holding_days_in_statutory_window: 60,
            stock_is_debt_financed_portfolio: false,
            average_indebtedness_pct_bp: 0,
        }
    }

    // Ownership tiers.

    #[test]
    fn under_20_pct_50_drd() {
        let r = compute(&base());
        assert_eq!(r.ownership_tier, OwnershipTier::UnderTwentyPct);
        assert_eq!(r.base_drd_pct_bp, 5000);
        assert_eq!(r.effective_drd_pct_bp, 5000);
        assert_eq!(r.drd_amount_dollars, 50_000);
    }

    #[test]
    fn exactly_20_pct_65_drd() {
        let mut i = base();
        i.ownership_pct_bp = 2000;
        let r = compute(&i);
        assert_eq!(r.ownership_tier, OwnershipTier::TwentyToSeventyNinePct);
        assert_eq!(r.base_drd_pct_bp, 6500);
        assert_eq!(r.drd_amount_dollars, 65_000);
    }

    #[test]
    fn at_19_99_pct_still_under_20_tier() {
        // 1999 basis points = 19.99%.
        let mut i = base();
        i.ownership_pct_bp = 1999;
        let r = compute(&i);
        assert_eq!(r.ownership_tier, OwnershipTier::UnderTwentyPct);
    }

    #[test]
    fn at_50_pct_tier_2() {
        let mut i = base();
        i.ownership_pct_bp = 5000;
        let r = compute(&i);
        assert_eq!(r.ownership_tier, OwnershipTier::TwentyToSeventyNinePct);
    }

    #[test]
    fn at_79_99_pct_still_tier_2() {
        let mut i = base();
        i.ownership_pct_bp = 7999;
        let r = compute(&i);
        assert_eq!(r.ownership_tier, OwnershipTier::TwentyToSeventyNinePct);
    }

    #[test]
    fn at_80_pct_qualifying_group_100_drd() {
        let mut i = base();
        i.ownership_pct_bp = 8000;
        let r = compute(&i);
        assert_eq!(r.ownership_tier, OwnershipTier::EightyPlusPct);
        assert_eq!(r.base_drd_pct_bp, 10_000);
        assert_eq!(r.drd_amount_dollars, 100_000);
    }

    #[test]
    fn at_100_pct_full_drd() {
        let mut i = base();
        i.ownership_pct_bp = 10_000;
        let r = compute(&i);
        assert_eq!(r.drd_amount_dollars, 100_000);
    }

    // §246(c) holding period.

    #[test]
    fn common_45_days_exact_does_not_satisfy() {
        // Must be MORE THAN 45 days, not "at least 45".
        let mut i = base();
        i.holding_days_in_statutory_window = 45;
        let r = compute(&i);
        assert!(!r.holding_period_satisfied);
        assert_eq!(r.drd_amount_dollars, 0);
    }

    #[test]
    fn common_46_days_satisfies() {
        let mut i = base();
        i.holding_days_in_statutory_window = 46;
        let r = compute(&i);
        assert!(r.holding_period_satisfied);
        assert_eq!(r.drd_amount_dollars, 50_000);
    }

    #[test]
    fn preferred_long_dividend_90_days_does_not_satisfy() {
        let mut i = base();
        i.stock_type = StockType::PreferredStockLongDividend;
        i.holding_days_in_statutory_window = 90;
        let r = compute(&i);
        assert!(!r.holding_period_satisfied);
        assert_eq!(r.holding_period_required_days, 90);
    }

    #[test]
    fn preferred_long_dividend_91_days_satisfies() {
        let mut i = base();
        i.stock_type = StockType::PreferredStockLongDividend;
        i.holding_days_in_statutory_window = 91;
        let r = compute(&i);
        assert!(r.holding_period_satisfied);
    }

    #[test]
    fn preferred_short_dividend_uses_45_day_window() {
        let mut i = base();
        i.stock_type = StockType::PreferredStockShortDividend;
        i.holding_days_in_statutory_window = 46;
        let r = compute(&i);
        assert!(r.holding_period_satisfied);
        assert_eq!(r.holding_period_required_days, 45);
    }

    #[test]
    fn holding_period_failure_full_disallowance() {
        // Even with 100% ownership, if holding period fails → $0 DRD.
        let mut i = base();
        i.ownership_pct_bp = 10_000;
        i.holding_days_in_statutory_window = 30;
        let r = compute(&i);
        assert_eq!(r.drd_amount_dollars, 0);
        assert!(r.note.contains("HOLDING PERIOD FAILED"));
    }

    // §246A debt-financed reduction.

    #[test]
    fn debt_financed_under_20_50_drd_reduced_by_indebtedness() {
        // 50% × (100% − 40%) = 50% × 60% = 30%.
        let mut i = base();
        i.stock_is_debt_financed_portfolio = true;
        i.average_indebtedness_pct_bp = 4000; // 40%
        let r = compute(&i);
        assert!(r.debt_financed_reduction_applies);
        assert_eq!(r.effective_drd_pct_bp, 3000);
        assert_eq!(r.drd_amount_dollars, 30_000);
    }

    #[test]
    fn debt_financed_20_pct_tier_65_drd_reduced() {
        // 65% × (100% − 50%) = 65% × 50% = 32.5%.
        let mut i = base();
        i.ownership_pct_bp = 2500;
        i.stock_is_debt_financed_portfolio = true;
        i.average_indebtedness_pct_bp = 5000; // 50%
        let r = compute(&i);
        assert_eq!(r.effective_drd_pct_bp, 3250);
        assert_eq!(r.drd_amount_dollars, 32_500);
    }

    #[test]
    fn debt_financed_80_pct_tier_no_reduction() {
        // §246A does NOT apply to 80%+ owned qualifying group.
        let mut i = base();
        i.ownership_pct_bp = 8000;
        i.stock_is_debt_financed_portfolio = true;
        i.average_indebtedness_pct_bp = 5000;
        let r = compute(&i);
        assert!(!r.debt_financed_reduction_applies);
        assert_eq!(r.effective_drd_pct_bp, 10_000);
    }

    #[test]
    fn debt_financed_100_pct_indebtedness_clamps_to_zero_drd() {
        let mut i = base();
        i.stock_is_debt_financed_portfolio = true;
        i.average_indebtedness_pct_bp = 10_000;
        let r = compute(&i);
        assert_eq!(r.effective_drd_pct_bp, 0);
        assert_eq!(r.drd_amount_dollars, 0);
    }

    #[test]
    fn debt_financed_excess_indebtedness_capped() {
        // Input > 100% should be clamped to 100% for safety.
        let mut i = base();
        i.stock_is_debt_financed_portfolio = true;
        i.average_indebtedness_pct_bp = 15_000;
        let r = compute(&i);
        assert_eq!(r.effective_drd_pct_bp, 0);
    }

    // Combined paths.

    #[test]
    fn debt_financed_holding_period_failed_zero_drd() {
        let mut i = base();
        i.stock_is_debt_financed_portfolio = true;
        i.average_indebtedness_pct_bp = 3000;
        i.holding_days_in_statutory_window = 30;
        let r = compute(&i);
        assert_eq!(r.drd_amount_dollars, 0);
        assert!(r.note.contains("HOLDING PERIOD FAILED"));
    }

    // Precision / large gain.

    #[test]
    fn very_large_dividend_precision_path() {
        let mut i = base();
        i.dividend_received_dollars = 1_000_000_000;
        let r = compute(&i);
        // 50% × $1B = $500M.
        assert_eq!(r.drd_amount_dollars, 500_000_000);
    }

    #[test]
    fn zero_dividend_zero_drd() {
        let mut i = base();
        i.dividend_received_dollars = 0;
        let r = compute(&i);
        assert_eq!(r.drd_amount_dollars, 0);
    }

    // Note text.

    #[test]
    fn under_20_note_describes_50_pct_tier() {
        let r = compute(&base());
        assert!(r.note.contains("UnderTwentyPct"));
        assert!(r.note.contains("50.00%"));
    }

    #[test]
    fn debt_financed_note_describes_reduction() {
        let mut i = base();
        i.stock_is_debt_financed_portfolio = true;
        i.average_indebtedness_pct_bp = 4000;
        let r = compute(&i);
        assert!(r.note.contains("§246A debt-financed"));
        assert!(r.note.contains("30.00%"));
    }

    #[test]
    fn citation_mentions_all_relevant_authorities() {
        let r = compute(&base());
        assert!(r.citation.contains("§243(a)(1)"));
        assert!(r.citation.contains("§243(b)"));
        assert!(r.citation.contains("§243(c)"));
        assert!(r.citation.contains("§246(c)"));
        assert!(r.citation.contains("§246A"));
        assert!(r.citation.contains("TCJA"));
    }
}
