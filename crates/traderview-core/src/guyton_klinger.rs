//! Guyton-Klinger guardrails — dynamic retirement withdrawals.
//!
//! A static 4% rule ignores how the portfolio actually does; Guyton-
//! Klinger adjusts the withdrawal each year against guardrails so a bad
//! sequence doesn't drain the account and a good one isn't under-spent.
//! Three rules drive one year's decision:
//!
//!   * **Withdrawal (inflation) rule** — raise the withdrawal with
//!     inflation, but FREEZE the raise in a year the portfolio lost
//!     money (don't compound a loss with a bigger draw).
//!   * **Capital-preservation rule** — if the resulting withdrawal rate
//!     rises above the upper guardrail (initial rate × 1.20 by default),
//!     CUT the withdrawal by the adjustment (10% default).
//!   * **Prosperity rule** — if it falls below the lower guardrail
//!     (initial × 0.80), RAISE the withdrawal by the adjustment.
//!
//! This is the single-year decision: the FIRE modules size the nest egg
//! (accumulation); this spends it down (decumulation). Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct GuardrailInput {
    pub portfolio_value_usd: f64,
    /// Last year's withdrawal in dollars.
    pub last_withdrawal_usd: f64,
    /// The withdrawal rate set at retirement (e.g. 5.0 for 5%).
    pub initial_withdrawal_rate_pct: f64,
    pub inflation_pct: f64,
    /// Guardrail band as a percent of the initial rate (default 20).
    pub guardrail_pct: f64,
    /// Cut/raise size when a guardrail is breached (default 10).
    pub adjustment_pct: f64,
    /// Did the portfolio gain this year? Drives the freeze rule.
    pub portfolio_gained: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Rule {
    /// Normal inflation raise, within the guardrails.
    InflationAdjusted,
    /// Down year — the inflation raise was skipped.
    FrozenDownYear,
    /// Rate too high — withdrawal cut to preserve capital.
    CapitalPreservation,
    /// Rate too low — withdrawal raised (you can afford more).
    Prosperity,
}

#[derive(Debug, Clone, Serialize)]
pub struct GuardrailDecision {
    /// Withdrawal after the inflation/freeze rule, before guardrails.
    pub base_withdrawal_usd: f64,
    /// Base withdrawal as a percent of the current portfolio.
    pub current_rate_pct: f64,
    pub upper_guardrail_pct: f64,
    pub lower_guardrail_pct: f64,
    pub rule: Rule,
    pub final_withdrawal_usd: f64,
    /// Final vs last year's withdrawal, percent.
    pub change_vs_last_pct: f64,
}

pub fn decide(i: &GuardrailInput) -> GuardrailDecision {
    // Withdrawal rule: freeze the inflation raise after a down year (only
    // when inflation would have raised it).
    let frozen = !i.portfolio_gained && i.inflation_pct > 0.0;
    let base = if frozen {
        i.last_withdrawal_usd
    } else {
        i.last_withdrawal_usd * (1.0 + i.inflation_pct / 100.0)
    };

    let portfolio = i.portfolio_value_usd.max(0.0);
    let current_rate = if portfolio > 0.0 { base / portfolio * 100.0 } else { 0.0 };
    let upper = i.initial_withdrawal_rate_pct * (1.0 + i.guardrail_pct / 100.0);
    let lower = i.initial_withdrawal_rate_pct * (1.0 - i.guardrail_pct / 100.0);

    let (rule, final_withdrawal) = if portfolio > 0.0 && current_rate > upper {
        (Rule::CapitalPreservation, base * (1.0 - i.adjustment_pct / 100.0))
    } else if portfolio > 0.0 && current_rate < lower {
        (Rule::Prosperity, base * (1.0 + i.adjustment_pct / 100.0))
    } else if frozen {
        (Rule::FrozenDownYear, base)
    } else {
        (Rule::InflationAdjusted, base)
    };

    let change_vs_last_pct = if i.last_withdrawal_usd > 0.0 {
        (final_withdrawal / i.last_withdrawal_usd - 1.0) * 100.0
    } else {
        0.0
    };

    GuardrailDecision {
        base_withdrawal_usd: base,
        current_rate_pct: current_rate,
        upper_guardrail_pct: upper,
        lower_guardrail_pct: lower,
        rule,
        final_withdrawal_usd: final_withdrawal,
        change_vs_last_pct,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> GuardrailInput {
        GuardrailInput {
            portfolio_value_usd: 1_000_000.0,
            last_withdrawal_usd: 50_000.0,
            initial_withdrawal_rate_pct: 5.0,
            inflation_pct: 3.0,
            guardrail_pct: 20.0,
            adjustment_pct: 10.0,
            portfolio_gained: true,
        }
    }

    #[test]
    fn normal_year_takes_inflation_raise() {
        // $50k × 1.03 = $51,500; rate 5.15% sits inside [4%, 6%].
        let r = decide(&base());
        assert!((r.base_withdrawal_usd - 51_500.0).abs() < 1e-6);
        assert!((r.current_rate_pct - 5.15).abs() < 1e-9);
        assert_eq!(r.rule, Rule::InflationAdjusted);
        assert!((r.final_withdrawal_usd - 51_500.0).abs() < 1e-6);
    }

    #[test]
    fn guardrails_are_initial_rate_plus_minus_band() {
        let r = decide(&base());
        assert!((r.upper_guardrail_pct - 6.0).abs() < 1e-9); // 5 × 1.20
        assert!((r.lower_guardrail_pct - 4.0).abs() < 1e-9); // 5 × 0.80
    }

    #[test]
    fn high_rate_triggers_capital_preservation_cut() {
        // Portfolio fell to $760k; even the frozen $50k draw is 6.58% > 6%.
        let r = decide(&GuardrailInput {
            portfolio_value_usd: 760_000.0,
            portfolio_gained: false,
            ..base()
        });
        // Down year freezes the raise → base stays $50k.
        assert!((r.base_withdrawal_usd - 50_000.0).abs() < 1e-6);
        assert_eq!(r.rule, Rule::CapitalPreservation);
        // Cut 10% → $45,000.
        assert!((r.final_withdrawal_usd - 45_000.0).abs() < 1e-6);
        assert!((r.change_vs_last_pct - (-10.0)).abs() < 1e-9);
    }

    #[test]
    fn low_rate_triggers_prosperity_raise() {
        // Portfolio soared to $1.5M; $51,500 is 3.43% < 4%.
        let r = decide(&GuardrailInput { portfolio_value_usd: 1_500_000.0, ..base() });
        assert_eq!(r.rule, Rule::Prosperity);
        // Raise 10% off the inflation-adjusted base.
        assert!((r.final_withdrawal_usd - 51_500.0 * 1.10).abs() < 1e-6);
    }

    #[test]
    fn down_year_freezes_raise_when_within_guardrails() {
        // Portfolio at $1.0M, down year: freeze at $50k, rate 5% in band.
        let r = decide(&GuardrailInput { portfolio_gained: false, ..base() });
        assert!((r.base_withdrawal_usd - 50_000.0).abs() < 1e-6);
        assert_eq!(r.rule, Rule::FrozenDownYear);
        assert!((r.final_withdrawal_usd - 50_000.0).abs() < 1e-6);
    }

    #[test]
    fn down_year_with_deflation_still_takes_the_cut() {
        // Negative inflation isn't a raise, so the freeze doesn't apply.
        let r = decide(&GuardrailInput {
            inflation_pct: -2.0,
            portfolio_gained: false,
            ..base()
        });
        // base = 50000 × 0.98 = 49,000; rate 4.9% in band.
        assert!((r.base_withdrawal_usd - 49_000.0).abs() < 1e-6);
        assert_eq!(r.rule, Rule::InflationAdjusted);
    }
}
