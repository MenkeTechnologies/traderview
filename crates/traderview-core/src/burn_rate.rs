//! Burn rate & runway — how long the cash lasts.
//!
//! Gross burn is monthly expenses; net burn is expenses minus revenue (what
//! actually drains the bank each month). Runway is how many months the cash
//! on hand survives that net burn. With growing revenue the net burn shrinks
//! each month, so the runway is simulated rather than a simple division —
//! and if revenue overtakes expenses before the cash runs out, the business
//! reaches break-even and never depletes.
//!
//!   * gross burn = monthly expenses
//!   * net burn (month 1) = expenses − revenue
//!   * runway = months survived before cash goes negative
//!   * months to break-even = first month revenue ≥ expenses
//!
//! Pure compute (month-by-month, capped at 600 months / 50 years).

use serde::{Deserialize, Serialize};

const HORIZON_MONTHS: u32 = 600;

#[derive(Debug, Clone, Deserialize)]
pub struct BurnInput {
    pub cash_on_hand_usd: f64,
    pub monthly_revenue_usd: f64,
    pub monthly_expenses_usd: f64,
    /// Monthly revenue growth rate (e.g. 10 for 10%/mo); 0 = flat.
    #[serde(default)]
    pub monthly_revenue_growth_pct: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct BurnResult {
    pub gross_burn_usd: f64,
    /// Expenses − revenue in month 1 (negative ⇒ already profitable).
    pub net_burn_usd: f64,
    /// Months of cash before it goes negative; `None` if it never runs out.
    pub runway_months: Option<u32>,
    /// First month revenue ≥ expenses; `None` if already profitable or never.
    pub months_to_breakeven: Option<u32>,
    /// True when month-1 revenue already covers expenses.
    pub already_profitable: bool,
}

pub fn analyze(i: &BurnInput) -> BurnResult {
    let g = i.monthly_revenue_growth_pct / 100.0;
    let gross_burn = i.monthly_expenses_usd;
    let net_burn = i.monthly_expenses_usd - i.monthly_revenue_usd;
    let already_profitable = i.monthly_revenue_usd >= i.monthly_expenses_usd;

    // Find months-to-breakeven (first month revenue covers expenses).
    let mut months_to_breakeven = None;
    if already_profitable {
        months_to_breakeven = None; // already there
    } else if g > 0.0 {
        for m in 1..=HORIZON_MONTHS {
            let rev = i.monthly_revenue_usd * (1.0 + g).powi((m - 1) as i32);
            if rev >= i.monthly_expenses_usd {
                months_to_breakeven = Some(m);
                break;
            }
        }
    }

    // Simulate cash month by month.
    let mut cash = i.cash_on_hand_usd;
    let mut runway = None;
    if !already_profitable {
        for m in 1..=HORIZON_MONTHS {
            let rev = i.monthly_revenue_usd * (1.0 + g).powi((m - 1) as i32);
            cash += rev - i.monthly_expenses_usd;
            if cash < 0.0 {
                runway = Some(m - 1); // full months survived before going negative
                break;
            }
        }
    }
    // already_profitable or never depleted within the horizon ⇒ runway None.

    BurnResult {
        gross_burn_usd: gross_burn,
        net_burn_usd: net_burn,
        runway_months: runway,
        months_to_breakeven,
        already_profitable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> BurnInput {
        BurnInput {
            cash_on_hand_usd: 100_000.0,
            monthly_revenue_usd: 0.0,
            monthly_expenses_usd: 10_000.0,
            monthly_revenue_growth_pct: 0.0,
        }
    }

    #[test]
    fn flat_burn_runway_is_cash_over_burn() {
        // 100k / 10k = 10 months.
        let r = analyze(&base());
        assert!((r.gross_burn_usd - 10_000.0).abs() < 1e-9);
        assert!((r.net_burn_usd - 10_000.0).abs() < 1e-9);
        assert_eq!(r.runway_months, Some(10));
    }

    #[test]
    fn revenue_reduces_net_burn_and_extends_runway() {
        // rev 5k, exp 10k → net burn 5k → 100k/5k = 20 months.
        let r = analyze(&BurnInput { monthly_revenue_usd: 5_000.0, ..base() });
        assert!((r.net_burn_usd - 5_000.0).abs() < 1e-9);
        assert_eq!(r.runway_months, Some(20));
    }

    #[test]
    fn profitable_has_no_runway_limit() {
        // rev 12k > exp 10k → net burn negative, already profitable.
        let r = analyze(&BurnInput { monthly_revenue_usd: 12_000.0, ..base() });
        assert!(r.already_profitable);
        assert!(r.net_burn_usd < 0.0);
        assert_eq!(r.runway_months, None);
        assert_eq!(r.months_to_breakeven, None);
    }

    #[test]
    fn revenue_growth_reaches_breakeven() {
        // rev 5k growing 10%/mo, exp 10k → 5k×1.1^(n-1) ≥ 10k at n=9.
        let r = analyze(&BurnInput {
            monthly_revenue_usd: 5_000.0,
            monthly_revenue_growth_pct: 10.0,
            ..base()
        });
        assert_eq!(r.months_to_breakeven, Some(9));
    }

    #[test]
    fn strong_growth_never_depletes_cash() {
        // Reaches breakeven fast with ample cash → survives the horizon.
        let r = analyze(&BurnInput {
            monthly_revenue_usd: 8_000.0,
            monthly_revenue_growth_pct: 15.0,
            ..base()
        });
        assert_eq!(r.runway_months, None);
        assert!(r.months_to_breakeven.is_some());
    }

    #[test]
    fn already_profitable_has_no_breakeven_month() {
        let r = analyze(&BurnInput { monthly_revenue_usd: 15_000.0, ..base() });
        assert!(r.already_profitable);
        assert_eq!(r.months_to_breakeven, None);
    }

    #[test]
    fn zero_cash_runs_out_immediately() {
        let r = analyze(&BurnInput { cash_on_hand_usd: 0.0, ..base() });
        assert_eq!(r.runway_months, Some(0));
    }

    #[test]
    fn gross_burn_equals_expenses_regardless_of_revenue() {
        let r = analyze(&BurnInput { monthly_revenue_usd: 9_000.0, ..base() });
        assert!((r.gross_burn_usd - 10_000.0).abs() < 1e-9);
        assert!((r.net_burn_usd - 1_000.0).abs() < 1e-9);
    }
}
