//! Series I savings bond calculator. The composite rate combines a fixed rate
//! (locked at purchase for the life of the bond) with a semiannual inflation
//! rate via Treasury's official formula: composite = fixed + 2·semi + fixed·semi.
//! Holding the (constant-rate) assumption, value compounds monthly. I-bonds lock
//! for 12 months and forfeit the last 3 months of interest if redeemed before 5
//! years. Reports the composite rate, value and interest at the hold, the
//! redeemable amount today (after any early-redemption penalty), and milestone
//! rows at the 1/5/10/20/30-year marks. Faithful port of the former client-side
//! calculator. Pure compute, not advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct IbondInput {
    pub amount_usd: f64,
    pub fixed_rate_pct: f64,
    /// Semiannual inflation rate (%), used directly in Treasury's formula.
    pub semi_inflation_pct: f64,
    pub hold_months: u32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct IbondMilestone {
    pub month: u32,
    /// Stable label key: "y1", "y5", "y10", "y20", "y30".
    pub label_key: String,
    pub value_usd: f64,
    pub cum_interest_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct IbondReport {
    pub composite_annual_pct: f64,
    pub value_at_hold_usd: f64,
    pub interest_earned_usd: f64,
    pub total_return_pct: f64,
    pub redeemable_today_usd: f64,
    /// True when held < 5 years (3-month interest penalty applies).
    pub penalty_applies: bool,
    /// True when held < 1 year (not redeemable at all).
    pub locked: bool,
    pub last_three_months_interest_usd: f64,
    pub rows: Vec<IbondMilestone>,
    pub valid: bool,
}

fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

fn milestone_key(month: u32) -> Option<&'static str> {
    match month {
        12 => Some("y1"),
        60 => Some("y5"),
        120 => Some("y10"),
        240 => Some("y20"),
        360 => Some("y30"),
        _ => None,
    }
}

pub fn generate(i: &IbondInput) -> IbondReport {
    if i.amount_usd <= 0.0 || i.hold_months == 0 {
        return IbondReport::default();
    }
    let fixed_ann = i.fixed_rate_pct / 100.0;
    let semi_infl = i.semi_inflation_pct / 100.0;
    // Treasury composite uses the semiannual inflation rate directly.
    let composite_ann = fixed_ann + 2.0 * semi_infl + fixed_ann * semi_infl;
    let r_month = (1.0 + composite_ann).powf(1.0 / 12.0) - 1.0;

    let months = i.hold_months;
    let mut value = i.amount_usd;
    let mut cum_interest = 0.0;
    let mut last_three = 0.0;
    let mut rows = Vec::new();
    for m in 1..=months {
        let interest = value * r_month;
        value += interest;
        cum_interest += interest;
        if m >= months.saturating_sub(2) {
            last_three += interest;
        }
        if let Some(key) = milestone_key(m) {
            rows.push(IbondMilestone {
                month: m,
                label_key: key.to_string(),
                value_usd: round2(value),
                cum_interest_usd: round2(cum_interest),
            });
        }
    }

    let penalty_applies = months < 60;
    let locked = months < 12;
    let redeemable = if penalty_applies && months >= 12 {
        value - last_three
    } else if months >= 60 {
        value
    } else {
        0.0
    };

    IbondReport {
        composite_annual_pct: round4(composite_ann * 100.0),
        value_at_hold_usd: round2(value),
        interest_earned_usd: round2(cum_interest),
        total_return_pct: round4(cum_interest / i.amount_usd * 100.0),
        redeemable_today_usd: round2(redeemable),
        penalty_applies,
        locked,
        last_three_months_interest_usd: round2(last_three),
        rows,
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> IbondInput {
        IbondInput {
            amount_usd: 10_000.0,
            fixed_rate_pct: 1.20,
            semi_inflation_pct: 0.95,
            hold_months: 60,
        }
    }

    // Pins cross-checked against the JS compute() in Python.
    #[test]
    fn default_five_year_hold() {
        let d = generate(&base());
        assert!(d.valid);
        assert!(close(d.composite_annual_pct, 3.1114));
        assert!(close(d.value_at_hold_usd, 11_655.57));
        assert!(close(d.interest_earned_usd, 1_655.57));
        assert!(close(d.total_return_pct, 16.5557));
        assert!(close(d.last_three_months_interest_usd, 88.94));
        assert!(close(d.redeemable_today_usd, 11_655.57)); // ≥5yr → no penalty
        assert!(!d.penalty_applies);
        assert!(!d.locked);
        // Milestones at month 12 and 60 → 2 rows.
        assert_eq!(d.rows.len(), 2);
        assert_eq!(d.rows[0].label_key, "y1");
        assert!(close(d.rows[0].value_usd, 10_311.14));
        assert_eq!(d.rows[1].label_key, "y5");
    }

    #[test]
    fn early_redemption_penalty() {
        let d = generate(&IbondInput { hold_months: 36, ..base() });
        assert!(d.penalty_applies);
        assert!(close(d.value_at_hold_usd, 10_962.76));
        assert!(close(d.redeemable_today_usd, 10_879.11)); // value − last 3 months
        assert!(close(d.last_three_months_interest_usd, 83.65));
    }

    #[test]
    fn locked_under_one_year() {
        let d = generate(&IbondInput { hold_months: 6, ..base() });
        assert!(d.locked);
        assert!(close(d.redeemable_today_usd, 0.0));
        assert!(d.rows.is_empty());
    }

    #[test]
    fn invalid_when_amount_zero() {
        let d = generate(&IbondInput { amount_usd: 0.0, ..base() });
        assert!(!d.valid);
    }
}
