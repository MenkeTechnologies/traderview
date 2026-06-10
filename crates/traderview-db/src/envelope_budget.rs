//! Envelope budgeting — digital simulation of the cash-envelope method.
//!
//! Traditional cash envelope budgeting (Larry Burkett / Crown Financial,
//! 1970s): physically split monthly cash into per-category envelopes
//! at the start of the month; when an envelope is empty, that category
//! is done for the month. The digital version tracks the same idea
//! without the literal envelopes.
//!
//! Each input envelope has:
//!   - name
//!   - period_allotment_usd  — start-of-period budget
//!   - starting_balance_usd  — what's currently in the envelope
//!                              (may include rollover from prior period)
//!   - spent_this_period_usd
//!   - rollover               — true = leftover carries into next period
//!                              (sinking-fund style); false = leftover
//!                              resets (use-it-or-lose-it categories
//!                              like groceries)
//!
//! Compute returns per envelope:
//!   - remaining_usd       = starting_balance − spent_this_period
//!   - usage_pct           = spent / period_allotment × 100
//!   - status              = "ok" (usage < 75%) | "warning" (≥ 75 < 100%)
//!                            | "empty" (≥ 100%, may go negative)
//!   - next_period_balance = if rollover: max(remaining, 0) + period_allotment
//!                            else: period_allotment
//!
//! Plus aggregates: total_allotment / total_starting_balance /
//! total_spent / total_remaining / envelopes_empty_count /
//! envelopes_warning_count / overall_status.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct EnvelopeInput {
    pub name: String,
    pub period_allotment_usd: f64,
    pub starting_balance_usd: f64,
    pub spent_this_period_usd: f64,
    #[serde(default)]
    pub rollover: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EnvelopeBudgetInput {
    #[serde(default)]
    pub envelopes: Vec<EnvelopeInput>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvelopeResult {
    pub name: String,
    pub period_allotment_usd: f64,
    pub starting_balance_usd: f64,
    pub spent_this_period_usd: f64,
    pub remaining_usd: f64,
    pub usage_pct: f64,
    pub status: &'static str,
    pub rollover: bool,
    pub next_period_balance_usd: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct EnvelopeBudgetReport {
    pub envelopes: Vec<EnvelopeResult>,
    pub total_allotment_usd: f64,
    pub total_starting_balance_usd: f64,
    pub total_spent_usd: f64,
    pub total_remaining_usd: f64,
    pub envelopes_empty_count: usize,
    pub envelopes_warning_count: usize,
    pub overall_status: String,
}

// ─── Pure compute ─────────────────────────────────────────────────────────

pub fn envelope_status(usage_pct: f64) -> &'static str {
    if usage_pct >= 100.0 { "empty" }
    else if usage_pct >= 75.0 { "warning" }
    else { "ok" }
}

pub fn next_period_balance(remaining: f64, allotment: f64, rollover: bool) -> f64 {
    if rollover {
        remaining.max(0.0) + allotment
    } else {
        allotment
    }
}

pub fn evaluate_envelope(e: &EnvelopeInput) -> EnvelopeResult {
    let remaining = e.starting_balance_usd - e.spent_this_period_usd;
    let usage = if e.period_allotment_usd > 0.0 {
        e.spent_this_period_usd / e.period_allotment_usd * 100.0
    } else if e.spent_this_period_usd > 0.0 {
        100.0
    } else {
        0.0
    };
    let status = envelope_status(usage);
    EnvelopeResult {
        name: e.name.clone(),
        period_allotment_usd: e.period_allotment_usd,
        starting_balance_usd: e.starting_balance_usd,
        spent_this_period_usd: e.spent_this_period_usd,
        remaining_usd: remaining,
        usage_pct: usage,
        status,
        rollover: e.rollover,
        next_period_balance_usd: next_period_balance(remaining, e.period_allotment_usd, e.rollover),
    }
}

pub fn compute(input: &EnvelopeBudgetInput) -> EnvelopeBudgetReport {
    let envelopes: Vec<EnvelopeResult> = input.envelopes.iter().map(evaluate_envelope).collect();
    let total_allotment: f64 = envelopes.iter().map(|e| e.period_allotment_usd).sum();
    let total_starting: f64 = envelopes.iter().map(|e| e.starting_balance_usd).sum();
    let total_spent: f64 = envelopes.iter().map(|e| e.spent_this_period_usd).sum();
    let total_remaining: f64 = envelopes.iter().map(|e| e.remaining_usd).sum();
    let empty = envelopes.iter().filter(|e| e.status == "empty").count();
    let warning = envelopes.iter().filter(|e| e.status == "warning").count();
    let overall = if empty > 0 { "envelope_empty" }
                 else if warning > 0 { "watch" }
                 else { "healthy" }.to_string();
    EnvelopeBudgetReport {
        envelopes,
        total_allotment_usd: total_allotment,
        total_starting_balance_usd: total_starting,
        total_spent_usd: total_spent,
        total_remaining_usd: total_remaining,
        envelopes_empty_count: empty,
        envelopes_warning_count: warning,
        overall_status: overall,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn e(name: &str, alloc: f64, start: f64, spent: f64, ro: bool) -> EnvelopeInput {
        EnvelopeInput {
            name: name.into(),
            period_allotment_usd: alloc,
            starting_balance_usd: start,
            spent_this_period_usd: spent,
            rollover: ro,
        }
    }

    #[test]
    fn envelope_status_thresholds() {
        assert_eq!(envelope_status(50.0), "ok");
        assert_eq!(envelope_status(80.0), "warning");
        assert_eq!(envelope_status(99.9), "warning");
        assert_eq!(envelope_status(100.0), "empty");
        assert_eq!(envelope_status(120.0), "empty");
    }

    #[test]
    fn next_period_balance_rollover() {
        // $50 remaining + $200 allotment → $250 starting balance next period.
        assert_eq!(next_period_balance(50.0, 200.0, true), 250.0);
    }

    #[test]
    fn next_period_balance_rollover_clamps_negative_to_zero() {
        // -$10 remaining: doesn't carry a deficit forward; just resets to allotment.
        assert_eq!(next_period_balance(-10.0, 200.0, true), 200.0);
    }

    #[test]
    fn next_period_balance_no_rollover_resets() {
        assert_eq!(next_period_balance(50.0, 200.0, false), 200.0);
    }

    #[test]
    fn evaluate_envelope_ok_status() {
        let r = evaluate_envelope(&e("groc", 400.0, 400.0, 100.0, false));
        assert_eq!(r.remaining_usd, 300.0);
        assert_eq!(r.usage_pct, 25.0);
        assert_eq!(r.status, "ok");
    }

    #[test]
    fn evaluate_envelope_warning_status() {
        let r = evaluate_envelope(&e("groc", 400.0, 400.0, 320.0, false));
        assert_eq!(r.usage_pct, 80.0);
        assert_eq!(r.status, "warning");
    }

    #[test]
    fn evaluate_envelope_empty_negative_remaining() {
        let r = evaluate_envelope(&e("groc", 400.0, 400.0, 450.0, false));
        assert_eq!(r.remaining_usd, -50.0);
        assert!(r.usage_pct >= 100.0);
        assert_eq!(r.status, "empty");
    }

    #[test]
    fn evaluate_envelope_zero_allotment_with_spend_treated_full() {
        let r = evaluate_envelope(&e("none", 0.0, 0.0, 50.0, false));
        assert_eq!(r.usage_pct, 100.0);
        assert_eq!(r.status, "empty");
    }

    #[test]
    fn evaluate_envelope_rollover_carries_remainder() {
        let r = evaluate_envelope(&e("xmas", 100.0, 100.0, 0.0, true));
        // unspent → carry $100, plus $100 next period = $200.
        assert_eq!(r.next_period_balance_usd, 200.0);
    }

    #[test]
    fn evaluate_envelope_no_rollover_resets_to_allotment() {
        let r = evaluate_envelope(&e("groc", 400.0, 400.0, 100.0, false));
        assert_eq!(r.next_period_balance_usd, 400.0);
    }

    #[test]
    fn compute_aggregate_status_healthy_when_all_ok() {
        let r = compute(&EnvelopeBudgetInput {
            envelopes: vec![
                e("a", 400.0, 400.0, 100.0, false),
                e("b", 200.0, 200.0, 50.0, false),
            ],
        });
        assert_eq!(r.envelopes_empty_count, 0);
        assert_eq!(r.envelopes_warning_count, 0);
        assert_eq!(r.overall_status, "healthy");
    }

    #[test]
    fn compute_aggregate_status_watch_when_any_warning() {
        let r = compute(&EnvelopeBudgetInput {
            envelopes: vec![
                e("a", 400.0, 400.0, 320.0, false), // warning
                e("b", 200.0, 200.0, 50.0,  false), // ok
            ],
        });
        assert_eq!(r.envelopes_warning_count, 1);
        assert_eq!(r.overall_status, "watch");
    }

    #[test]
    fn compute_aggregate_status_empty_wins_over_warning() {
        let r = compute(&EnvelopeBudgetInput {
            envelopes: vec![
                e("a", 400.0, 400.0, 320.0, false), // warning
                e("b", 200.0, 200.0, 250.0, false), // empty
            ],
        });
        assert_eq!(r.envelopes_empty_count, 1);
        assert_eq!(r.envelopes_warning_count, 1);
        assert_eq!(r.overall_status, "envelope_empty");
    }

    #[test]
    fn compute_aggregate_totals_correct() {
        let r = compute(&EnvelopeBudgetInput {
            envelopes: vec![
                e("a", 400.0, 400.0, 100.0, false),
                e("b", 200.0, 250.0, 75.0,  true),
            ],
        });
        assert_eq!(r.total_allotment_usd, 600.0);
        assert_eq!(r.total_starting_balance_usd, 650.0);
        assert_eq!(r.total_spent_usd, 175.0);
        assert_eq!(r.total_remaining_usd, 475.0);
    }

    #[test]
    fn compute_empty_input_healthy_zero_counts() {
        let r = compute(&EnvelopeBudgetInput { envelopes: vec![] });
        assert_eq!(r.envelopes_empty_count, 0);
        assert_eq!(r.envelopes_warning_count, 0);
        assert_eq!(r.overall_status, "healthy");
    }
}
