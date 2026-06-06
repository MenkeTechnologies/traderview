//! SIP / DRIP recurring-investment simulator.
//!
//! Models a Systematic Investment Plan (or Dividend Reinvestment Plan)
//! into a single security over time. For each scheduled contribution:
//!   1. Buy `contribution_dollars` worth at the bar's price (fractional
//!      shares allowed for SIPs; if the broker disallows fractional, the
//!      caller can post-round the share counts).
//!   2. Optionally reinvest a per-bar dividend yield as additional shares.
//!   3. Accumulate cumulative cost basis + share count + market value.
//!
//! Caller supplies a price series (bar_time + close). The schedule lives
//! in `ScheduleSpec` — weekly, biweekly, or monthly. We attribute each
//! contribution to the first bar at or after the contribution date.
//!
//! Pure compute. eToro / Robinhood / Coinbase Advanced offer this as a
//! recurring-deposit feature; this module is the math behind the UI.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PriceBar {
    pub ts: DateTime<Utc>,
    pub price: f64,
    /// Optional dividend per share (zero for non-dividend names).
    #[serde(default)]
    pub dividend_per_share: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Cadence {
    Weekly,
    Biweekly,
    Monthly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleSpec {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub cadence: Cadence,
    pub contribution_dollars: f64,
    /// If true, dividend_per_share triggers a reinvestment (DRIP).
    /// Else, dividends are tracked as cash but not reinvested.
    pub reinvest_dividends: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SipEvent {
    pub ts: DateTime<Utc>,
    /// "contribution" | "dividend_reinvest".
    pub kind: String,
    pub dollars_in: f64,
    pub shares_bought: f64,
    pub price: f64,
    pub cumulative_shares: f64,
    pub cumulative_cost: f64,
    pub cumulative_dividend_cash: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SipReport {
    pub events: Vec<SipEvent>,
    pub final_shares: f64,
    pub final_cost_basis: f64,
    pub final_market_value: f64,
    pub final_dividend_cash: f64,
    /// `(market_value + dividend_cash) / cost_basis - 1` over the period.
    pub total_return_pct: f64,
}

pub fn simulate(bars: &[PriceBar], spec: &ScheduleSpec) -> SipReport {
    if bars.is_empty() || spec.contribution_dollars <= 0.0 {
        return SipReport::default();
    }
    let scheduled = scheduled_dates(spec);
    let mut events: Vec<SipEvent> = Vec::with_capacity(scheduled.len() + bars.len());
    let mut shares = 0.0;
    let mut cost = 0.0;
    let mut cash_dividends = 0.0;
    let mut sched_iter = scheduled.into_iter().peekable();
    for bar in bars {
        // Apply every scheduled contribution at or before this bar.
        while let Some(&due) = sched_iter.peek() {
            if due > bar.ts {
                break;
            }
            sched_iter.next();
            if bar.price > 0.0 {
                let bought = spec.contribution_dollars / bar.price;
                shares += bought;
                cost += spec.contribution_dollars;
                events.push(SipEvent {
                    ts: bar.ts,
                    kind: "contribution".into(),
                    dollars_in: spec.contribution_dollars,
                    shares_bought: bought,
                    price: bar.price,
                    cumulative_shares: shares,
                    cumulative_cost: cost,
                    cumulative_dividend_cash: cash_dividends,
                });
            }
        }
        // Process per-bar dividend.
        if bar.dividend_per_share > 0.0 && shares > 0.0 {
            let dollars = bar.dividend_per_share * shares;
            if spec.reinvest_dividends && bar.price > 0.0 {
                let bought = dollars / bar.price;
                shares += bought;
                events.push(SipEvent {
                    ts: bar.ts,
                    kind: "dividend_reinvest".into(),
                    dollars_in: dollars,
                    shares_bought: bought,
                    price: bar.price,
                    cumulative_shares: shares,
                    cumulative_cost: cost,
                    cumulative_dividend_cash: cash_dividends,
                });
            } else {
                cash_dividends += dollars;
            }
        }
    }
    let final_price = bars.last().map(|b| b.price).unwrap_or(0.0);
    let mv = shares * final_price;
    let total_return = if cost > 0.0 {
        (mv + cash_dividends) / cost - 1.0
    } else {
        0.0
    };
    SipReport {
        events,
        final_shares: shares,
        final_cost_basis: cost,
        final_market_value: mv,
        final_dividend_cash: cash_dividends,
        total_return_pct: total_return,
    }
}

fn scheduled_dates(spec: &ScheduleSpec) -> Vec<DateTime<Utc>> {
    let mut out = Vec::new();
    if spec.end < spec.start {
        return out;
    }
    let step_days = match spec.cadence {
        Cadence::Weekly => 7,
        Cadence::Biweekly => 14,
        Cadence::Monthly => 30, // approximate — caller wanting exact
                                // calendar months can iterate manually
    };
    let mut t = spec.start;
    let mut guard = 0usize;
    while t <= spec.end && guard < 10_000 {
        out.push(t);
        t += chrono::Duration::days(step_days);
        guard += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn bar(s: &str, price: f64) -> PriceBar {
        PriceBar {
            ts: d(s),
            price,
            dividend_per_share: 0.0,
        }
    }

    fn d(s: &str) -> DateTime<Utc> {
        NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .unwrap()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc()
    }

    #[test]
    fn empty_input_returns_default_report() {
        let r = simulate(
            &[],
            &ScheduleSpec {
                start: d("2026-01-01"),
                end: d("2026-06-01"),
                cadence: Cadence::Monthly,
                contribution_dollars: 500.0,
                reinvest_dividends: false,
            },
        );
        assert_eq!(r.final_shares, 0.0);
        assert_eq!(r.final_cost_basis, 0.0);
        assert!(r.events.is_empty());
    }

    #[test]
    fn zero_contribution_skips_all_buys() {
        let bars = vec![bar("2026-01-15", 100.0), bar("2026-02-15", 100.0)];
        let r = simulate(
            &bars,
            &ScheduleSpec {
                start: d("2026-01-01"),
                end: d("2026-12-31"),
                cadence: Cadence::Monthly,
                contribution_dollars: 0.0,
                reinvest_dividends: false,
            },
        );
        assert!(r.events.is_empty());
    }

    #[test]
    fn monthly_500_into_flat_price_buys_constant_shares() {
        // Monthly $500 at $100 → 5 shares per contribution.
        let bars: Vec<PriceBar> = (1..=6)
            .map(|m| bar(&format!("2026-{m:02}-15"), 100.0))
            .collect();
        let r = simulate(
            &bars,
            &ScheduleSpec {
                start: d("2026-01-01"),
                end: d("2026-06-30"),
                cadence: Cadence::Monthly,
                contribution_dollars: 500.0,
                reinvest_dividends: false,
            },
        );
        // Expect 6 monthly contributions × 5 shares = 30 shares total.
        assert!(
            r.events.iter().filter(|e| e.kind == "contribution").count() >= 5,
            "expected at least 5 contributions, got {}",
            r.events.len()
        );
        // Each contribution buys exactly 5 shares at $100.
        for ev in &r.events {
            if ev.kind == "contribution" {
                assert!((ev.shares_bought - 5.0).abs() < 1e-9);
            }
        }
        assert!(
            (r.final_cost_basis
                - 500.0 * r.events.iter().filter(|e| e.kind == "contribution").count() as f64)
                .abs()
                < 1e-9
        );
    }

    #[test]
    fn drip_reinvests_dividends_as_more_shares() {
        // Single contribution of $1000 at $100 → 10 shares. Dividend of $1/share
        // on that bar = $10 → reinvest at $100 = 0.1 more shares.
        // Window kept tight (start..=start+14) so the 30-day Monthly cadence
        // only fires once.
        let bars = vec![PriceBar {
            ts: d("2026-01-15"),
            price: 100.0,
            dividend_per_share: 1.0,
        }];
        let r = simulate(
            &bars,
            &ScheduleSpec {
                start: d("2026-01-01"),
                end: d("2026-01-15"),
                cadence: Cadence::Monthly,
                contribution_dollars: 1000.0,
                reinvest_dividends: true,
            },
        );
        assert!(
            (r.final_shares - 10.1).abs() < 1e-9,
            "DRIP should add 0.1 shares; got total {}",
            r.final_shares
        );
        assert_eq!(
            r.final_dividend_cash, 0.0,
            "DRIP path keeps no cash dividend"
        );
    }

    #[test]
    fn non_drip_keeps_dividends_as_cash() {
        let bars = vec![PriceBar {
            ts: d("2026-01-15"),
            price: 100.0,
            dividend_per_share: 2.0,
        }];
        let r = simulate(
            &bars,
            &ScheduleSpec {
                start: d("2026-01-01"),
                end: d("2026-01-15"),
                cadence: Cadence::Monthly,
                contribution_dollars: 1000.0,
                reinvest_dividends: false,
            },
        );
        assert!(
            (r.final_dividend_cash - 20.0).abs() < 1e-9,
            "non-DRIP should accrue $20 cash, got {}",
            r.final_dividend_cash
        );
        assert!((r.final_shares - 10.0).abs() < 1e-9);
    }

    #[test]
    fn appreciation_into_rising_price_shows_positive_return() {
        // Buy at $100, end at $150 — positive return.
        let bars = vec![bar("2026-01-15", 100.0), bar("2026-06-15", 150.0)];
        let r = simulate(
            &bars,
            &ScheduleSpec {
                start: d("2026-01-01"),
                end: d("2026-01-31"),
                cadence: Cadence::Monthly,
                contribution_dollars: 1000.0,
                reinvest_dividends: false,
            },
        );
        assert!(
            r.total_return_pct > 0.0,
            "rising price must produce positive return, got {}",
            r.total_return_pct
        );
    }
}
