//! 401(k) per-paycheck maximizer — and the front-loading match trap.
//!
//! Two questions every year: how much per paycheck do I defer to hit the
//! annual limit by December, and am I about to forfeit free money? The
//! trap: most plans match per paycheck, so if you front-load and hit the
//! limit early, you contribute $0 on the remaining checks and the
//! employer matches $0 on them too — unless the plan has a year-end
//! "true-up" that pays the missed match. Spreading evenly captures the
//! full match every period.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct Paycheck401kInput {
    /// Employee elective-deferral limit for the year.
    pub annual_limit_usd: f64,
    /// Already deferred year-to-date.
    pub ytd_contributed_usd: f64,
    pub pay_periods_remaining: u32,
    pub gross_per_period_usd: f64,
    /// Employer match rate, cents per dollar (e.g. 50 = $0.50 per $1).
    pub employer_match_pct: f64,
    /// Employer matches deferrals up to this percent of pay per period.
    pub match_limit_pct_of_pay: f64,
    /// Does the plan true-up missed match at year-end?
    pub plan_has_true_up: bool,
    /// What you're currently set to defer per period — for the
    /// front-loading check (0 to skip it).
    pub planned_per_period_usd: f64,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct Paycheck401kResult {
    pub remaining_to_limit_usd: f64,
    /// Defer this per period to hit the limit exactly at year-end.
    pub even_per_period_usd: f64,
    pub even_pct_of_pay: f64,
    /// Defer at least this per period (pay × match-limit%) for full match.
    pub match_threshold_per_period_usd: f64,
    /// Employer dollars per period at or above the threshold.
    pub full_match_per_period_usd: f64,
    /// Does the even-spread deferral also clear the match threshold?
    pub captures_full_match_evenly: bool,
    /// Periods until the planned deferral hits the limit (`None` if no
    /// planned amount given).
    pub periods_to_limit_at_planned: Option<u32>,
    /// Periods you'd contribute $0 after hitting the limit early.
    pub empty_periods: u32,
    /// Match forfeited by front-loading without a true-up (else 0).
    pub forfeited_match_usd: f64,
}

pub fn compute(i: &Paycheck401kInput) -> Paycheck401kResult {
    let remaining = (i.annual_limit_usd - i.ytd_contributed_usd).max(0.0);
    let periods = i.pay_periods_remaining;

    let even_per_period = if periods > 0 { remaining / periods as f64 } else { 0.0 };
    let even_pct_of_pay = if i.gross_per_period_usd > 0.0 {
        even_per_period / i.gross_per_period_usd * 100.0
    } else {
        0.0
    };

    let threshold = i.gross_per_period_usd * (i.match_limit_pct_of_pay / 100.0);
    let full_match_per_period = (i.employer_match_pct / 100.0) * threshold;
    let captures_full_match_evenly = even_per_period >= threshold;

    let (periods_to_limit, empty_periods) = if i.planned_per_period_usd > 0.0 && remaining > 0.0 {
        let k = (remaining / i.planned_per_period_usd).ceil() as u32;
        let k = k.min(periods);
        (Some(k), periods.saturating_sub(k))
    } else {
        (None, 0)
    };

    let forfeited_match = if i.plan_has_true_up {
        0.0
    } else {
        full_match_per_period * empty_periods as f64
    };

    Paycheck401kResult {
        remaining_to_limit_usd: remaining,
        even_per_period_usd: even_per_period,
        even_pct_of_pay,
        match_threshold_per_period_usd: threshold,
        full_match_per_period_usd: full_match_per_period,
        captures_full_match_evenly,
        periods_to_limit_at_planned: periods_to_limit,
        empty_periods,
        forfeited_match_usd: forfeited_match,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Paycheck401kInput {
        Paycheck401kInput {
            annual_limit_usd: 23_000.0,
            ytd_contributed_usd: 0.0,
            pay_periods_remaining: 24,
            gross_per_period_usd: 4_000.0,
            employer_match_pct: 50.0,      // $0.50 per $1
            match_limit_pct_of_pay: 5.0,   // up to 5% of pay
            plan_has_true_up: false,
            planned_per_period_usd: 0.0,
        }
    }

    #[test]
    fn even_spread_hits_limit() {
        let r = compute(&base());
        // $23,000 / 24 ≈ $958.33 per period.
        assert!((r.even_per_period_usd - 23_000.0 / 24.0).abs() < 1e-6);
        assert!((r.even_pct_of_pay - (23_000.0 / 24.0) / 4_000.0 * 100.0).abs() < 1e-6);
    }

    #[test]
    fn match_threshold_and_full_match() {
        let r = compute(&base());
        // 5% of $4,000 = $200 to earn full match; 50% of $200 = $100.
        assert!((r.match_threshold_per_period_usd - 200.0).abs() < 1e-9);
        assert!((r.full_match_per_period_usd - 100.0).abs() < 1e-9);
        // Even spread ($958) clears the $200 threshold → full match all year.
        assert!(r.captures_full_match_evenly);
    }

    #[test]
    fn front_loading_forfeits_match_without_true_up() {
        // Defer $2,000/period → hits $23k after 12 periods, idle for 12,
        // forfeiting $100 match × 12 = $1,200.
        let r = compute(&Paycheck401kInput { planned_per_period_usd: 2_000.0, ..base() });
        assert_eq!(r.periods_to_limit_at_planned, Some(12));
        assert_eq!(r.empty_periods, 12);
        assert!((r.forfeited_match_usd - 1_200.0).abs() < 1e-9);
    }

    #[test]
    fn true_up_plan_forfeits_nothing() {
        let r = compute(&Paycheck401kInput {
            planned_per_period_usd: 2_000.0,
            plan_has_true_up: true,
            ..base()
        });
        assert_eq!(r.empty_periods, 12); // still hits early...
        assert_eq!(r.forfeited_match_usd, 0.0); // ...but the true-up pays it back
    }

    #[test]
    fn planned_within_limit_no_front_load() {
        // $958/period over 24 periods exactly hits the limit — no idle periods.
        let r = compute(&Paycheck401kInput { planned_per_period_usd: 958.34, ..base() });
        assert_eq!(r.empty_periods, 0);
        assert!((r.forfeited_match_usd - 0.0).abs() < 1e-9);
    }

    #[test]
    fn already_maxed_leaves_nothing() {
        let r = compute(&Paycheck401kInput { ytd_contributed_usd: 23_000.0, ..base() });
        assert_eq!(r.remaining_to_limit_usd, 0.0);
        assert_eq!(r.even_per_period_usd, 0.0);
    }

    #[test]
    fn no_planned_skips_front_load_check() {
        let r = compute(&base());
        assert_eq!(r.periods_to_limit_at_planned, None);
        assert_eq!(r.empty_periods, 0);
        assert_eq!(r.forfeited_match_usd, 0.0);
    }
}
