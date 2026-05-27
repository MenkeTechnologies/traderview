//! Pre-trade risk gate.
//!
//! `discipline.rs` evaluates rules against *already-closed* trades — useful
//! for self-review, useless for prevention. This module is the missing
//! piece: given a proposed trade + recent state + user-configured rules,
//! decide whether to ALLOW, WARN, or BLOCK the trade *before* it goes to
//! the broker.
//!
//! Pure compute, no I/O, no DB. The route handler is responsible for
//! loading the user's `RiskRule` set + recent trades + current open
//! positions and assembling a `GateContext`.
//!
//! Each rule type below has its own variant + handler. Adding a new rule
//! type is one match arm + one test.

use crate::risk;
use crate::models::{AssetClass, TradeSide};
use chrono::{DateTime, Duration, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Inputs
// ---------------------------------------------------------------------------

/// What the user is trying to do, before any broker call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProposedTrade {
    pub symbol: String,
    pub side: TradeSide,
    pub qty: Decimal,
    pub entry_price: Decimal,
    pub stop_loss: Option<Decimal>,
    pub asset_class: AssetClass,
    pub multiplier: Decimal,
    pub tick_size: Option<Decimal>,
    pub tick_value: Option<Decimal>,
    /// User attached a written plan before clicking buy. Required by some rules.
    pub has_attached_plan: bool,
}

/// Snapshot of recent state used by the rules. The caller queries DB +
/// builds this; the gate itself never touches I/O.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GateContext {
    /// Current account equity in dollars. Used by % rules.
    pub account_equity: Decimal,
    /// Today's running net P&L (closed trades only) in dollars.
    pub today_realized_pnl: Decimal,
    /// Today's open position dollar exposure. Used by max_open_positions.
    pub open_position_count: usize,
    /// Trades closed today, oldest first. Used by streak rules.
    pub today_closed_trades: Vec<RecentTrade>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentTrade {
    pub closed_at: DateTime<Utc>,
    pub net_pnl: Decimal,
}

// ---------------------------------------------------------------------------
// Rules
// ---------------------------------------------------------------------------

/// Single configured rule. The serde-tagged enum lets the DB persist
/// arbitrary rules as JSONB rows that round-trip cleanly.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RiskRule {
    /// Block trade if its dollar risk exceeds `pct` of account equity.
    MaxLossPerTradePct { pct: Decimal },
    /// Block any new entry if today's realized loss already breached `pct`
    /// of account equity.
    MaxLossPerDayPct { pct: Decimal },
    /// Block if the user has had `n` losing trades in a row today.
    MaxConsecutiveLossesToday { n: usize },
    /// Block if a losing trade closed less than `minutes` ago — avoid
    /// revenge-trading.
    CoolDownAfterLossMinutes { minutes: i64 },
    /// Block if `open_position_count` already at or above `n`.
    MaxOpenPositions { n: usize },
    /// Block if `qty * entry * multiplier` exceeds `pct` of equity.
    MaxPositionSizePct { pct: Decimal },
    /// Block if the symbol matches (case-insensitive) any in this list.
    /// Use for self-imposed bans ("never touching GME again").
    BlockedSymbols { symbols: Vec<String> },
    /// Block if `has_attached_plan == false`.
    RequirePlanBeforeTrade,
    /// Block if `stop_loss` is missing.
    RequireStopLoss,
}

// ---------------------------------------------------------------------------
// Output
// ---------------------------------------------------------------------------

/// Severity of a rule firing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Surface to the user but allow the trade.
    Warning,
    /// Block the trade until the user overrides or fixes the input.
    Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub rule: String,
    pub severity: Severity,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateDecision {
    /// True iff zero `Severity::Block` violations fired. Warnings allowed.
    pub allow: bool,
    pub violations: Vec<Violation>,
}

impl GateDecision {
    pub fn warnings(&self) -> impl Iterator<Item = &Violation> {
        self.violations.iter().filter(|v| v.severity == Severity::Warning)
    }
    pub fn blocks(&self) -> impl Iterator<Item = &Violation> {
        self.violations.iter().filter(|v| v.severity == Severity::Block)
    }
}

// ---------------------------------------------------------------------------
// Evaluation
// ---------------------------------------------------------------------------

pub fn evaluate(
    proposed: &ProposedTrade,
    ctx: &GateContext,
    rules: &[RiskRule],
    now: DateTime<Utc>,
) -> GateDecision {
    let mut violations = Vec::new();
    for rule in rules {
        if let Some(v) = check_rule(proposed, ctx, rule, now) {
            violations.push(v);
        }
    }
    let blocked = violations.iter().any(|v| v.severity == Severity::Block);
    GateDecision { allow: !blocked, violations }
}

fn check_rule(
    p: &ProposedTrade,
    ctx: &GateContext,
    rule: &RiskRule,
    now: DateTime<Utc>,
) -> Option<Violation> {
    match rule {
        RiskRule::MaxLossPerTradePct { pct } => {
            let stop = p.stop_loss?;
            let risk_dollars = risk::risk_amount(
                p.asset_class, p.side, p.qty, p.entry_price, stop,
                p.multiplier, p.tick_size, p.tick_value,
            );
            let cap = ctx.account_equity * (*pct / Decimal::from(100));
            if risk_dollars > cap {
                Some(Violation {
                    rule: "max_loss_per_trade_pct".into(),
                    severity: Severity::Block,
                    message: format!(
                        "trade risk ${} exceeds {}% of equity (${} cap)",
                        risk_dollars, pct, cap,
                    ),
                })
            } else { None }
        }
        RiskRule::MaxLossPerDayPct { pct } => {
            // Use absolute value of today's loss; positive P&L can't trigger.
            if ctx.today_realized_pnl >= Decimal::ZERO { return None; }
            let lost = -ctx.today_realized_pnl;
            let cap = ctx.account_equity * (*pct / Decimal::from(100));
            if lost >= cap {
                Some(Violation {
                    rule: "max_loss_per_day_pct".into(),
                    severity: Severity::Block,
                    message: format!(
                        "today's realized loss ${} has hit the {}% daily cap (${})",
                        lost, pct, cap,
                    ),
                })
            } else { None }
        }
        RiskRule::MaxConsecutiveLossesToday { n } => {
            // Walk today's trades backwards counting losses; stop at first non-loss.
            let mut streak = 0usize;
            for t in ctx.today_closed_trades.iter().rev() {
                if t.net_pnl < Decimal::ZERO { streak += 1; } else { break; }
            }
            if streak >= *n {
                Some(Violation {
                    rule: "max_consecutive_losses_today".into(),
                    severity: Severity::Block,
                    message: format!(
                        "{streak} consecutive losses today — cool off; rule caps at {n}",
                    ),
                })
            } else { None }
        }
        RiskRule::CoolDownAfterLossMinutes { minutes } => {
            let cutoff = now - Duration::minutes(*minutes);
            let last_loss = ctx.today_closed_trades.iter().rev()
                .find(|t| t.net_pnl < Decimal::ZERO);
            if let Some(loss) = last_loss {
                if loss.closed_at > cutoff {
                    let remaining = (loss.closed_at + Duration::minutes(*minutes) - now)
                        .num_seconds().max(0);
                    return Some(Violation {
                        rule: "cool_down_after_loss".into(),
                        severity: Severity::Block,
                        message: format!(
                            "last loss was less than {minutes}m ago — wait {remaining}s",
                        ),
                    });
                }
            }
            None
        }
        RiskRule::MaxOpenPositions { n } => {
            if ctx.open_position_count >= *n {
                Some(Violation {
                    rule: "max_open_positions".into(),
                    severity: Severity::Block,
                    message: format!(
                        "{} open positions already; rule caps at {n}",
                        ctx.open_position_count,
                    ),
                })
            } else { None }
        }
        RiskRule::MaxPositionSizePct { pct } => {
            let notional = p.qty * p.entry_price * p.multiplier;
            let cap = ctx.account_equity * (*pct / Decimal::from(100));
            if notional > cap {
                Some(Violation {
                    rule: "max_position_size_pct".into(),
                    severity: Severity::Block,
                    message: format!(
                        "notional ${} exceeds {}% of equity (${} cap)",
                        notional, pct, cap,
                    ),
                })
            } else { None }
        }
        RiskRule::BlockedSymbols { symbols } => {
            let needle = p.symbol.to_ascii_uppercase();
            if symbols.iter().any(|s| s.to_ascii_uppercase() == needle) {
                Some(Violation {
                    rule: "blocked_symbol".into(),
                    severity: Severity::Block,
                    message: format!("{} is on your self-imposed block list", p.symbol),
                })
            } else { None }
        }
        RiskRule::RequirePlanBeforeTrade => {
            if !p.has_attached_plan {
                Some(Violation {
                    rule: "require_plan".into(),
                    severity: Severity::Block,
                    message: "no pre-trade plan attached — write the setup first".into(),
                })
            } else { None }
        }
        RiskRule::RequireStopLoss => {
            if p.stop_loss.is_none() {
                Some(Violation {
                    rule: "require_stop_loss".into(),
                    severity: Severity::Warning,
                    message: "no stop loss set — you're trading without a defined exit".into(),
                })
            } else { None }
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;
    use std::str::FromStr;

    fn d(s: &str) -> Decimal { Decimal::from_str(s).unwrap() }

    fn proposed() -> ProposedTrade {
        ProposedTrade {
            symbol: "AAPL".into(),
            side: TradeSide::Long,
            qty: d("100"),
            entry_price: d("150"),
            stop_loss: Some(d("149")),    // $1 stop × 100 sh = $100 risk
            asset_class: AssetClass::Stock,
            multiplier: Decimal::ONE,
            tick_size: None,
            tick_value: None,
            has_attached_plan: true,
        }
    }

    fn ctx() -> GateContext {
        GateContext {
            account_equity: d("100000"),     // $100k
            today_realized_pnl: Decimal::ZERO,
            open_position_count: 0,
            today_closed_trades: vec![],
        }
    }

    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 5, 27, 14, 30, 0).unwrap()
    }

    // ─── empty rules ─────────────────────────────────────────────────────

    #[test]
    fn empty_rules_allow_everything() {
        let d = evaluate(&proposed(), &ctx(), &[], now());
        assert!(d.allow);
        assert!(d.violations.is_empty());
    }

    // ─── MaxLossPerTradePct ──────────────────────────────────────────────

    #[test]
    fn max_loss_per_trade_pct_blocks_oversized_risk() {
        // 1% of $100k = $1000 cap. Risk = $100 → allow.
        let rules = vec![RiskRule::MaxLossPerTradePct { pct: d("1") }];
        let dec = evaluate(&proposed(), &ctx(), &rules, now());
        assert!(dec.allow, "$100 risk under $1k cap should allow");

        // Tighten to 0.05% cap = $50. Risk $100 > $50 → block.
        let rules = vec![RiskRule::MaxLossPerTradePct { pct: d("0.05") }];
        let dec = evaluate(&proposed(), &ctx(), &rules, now());
        assert!(!dec.allow);
        assert_eq!(dec.violations.len(), 1);
        assert_eq!(dec.violations[0].severity, Severity::Block);
    }

    #[test]
    fn max_loss_per_trade_skips_when_no_stop() {
        // Can't compute risk without a stop — rule must skip, not panic.
        let mut p = proposed();
        p.stop_loss = None;
        let rules = vec![RiskRule::MaxLossPerTradePct { pct: d("0.01") }];
        let dec = evaluate(&p, &ctx(), &rules, now());
        assert!(dec.allow);
    }

    // ─── MaxLossPerDayPct ────────────────────────────────────────────────

    #[test]
    fn max_loss_per_day_blocks_after_cap_hit() {
        let mut c = ctx();
        c.today_realized_pnl = d("-2500");  // already down $2,500
        let rules = vec![RiskRule::MaxLossPerDayPct { pct: d("2") }]; // 2% × 100k = 2k cap
        let dec = evaluate(&proposed(), &c, &rules, now());
        assert!(!dec.allow);
        assert!(dec.violations[0].message.contains("daily cap"));
    }

    #[test]
    fn max_loss_per_day_ignores_positive_pnl() {
        let mut c = ctx();
        c.today_realized_pnl = d("5000");  // up $5k
        let rules = vec![RiskRule::MaxLossPerDayPct { pct: d("2") }];
        let dec = evaluate(&proposed(), &c, &rules, now());
        assert!(dec.allow, "positive P&L must never trigger a loss-cap rule");
    }

    // ─── MaxConsecutiveLossesToday ───────────────────────────────────────

    #[test]
    fn consec_losses_blocks_at_threshold() {
        let mut c = ctx();
        c.today_closed_trades = vec![
            RecentTrade { closed_at: now(), net_pnl: d("-100") },
            RecentTrade { closed_at: now(), net_pnl: d("-50") },
            RecentTrade { closed_at: now(), net_pnl: d("-200") },
        ];
        let rules = vec![RiskRule::MaxConsecutiveLossesToday { n: 3 }];
        let dec = evaluate(&proposed(), &c, &rules, now());
        assert!(!dec.allow);
    }

    #[test]
    fn consec_losses_resets_on_a_win() {
        let mut c = ctx();
        c.today_closed_trades = vec![
            RecentTrade { closed_at: now(), net_pnl: d("-100") },
            RecentTrade { closed_at: now(), net_pnl: d("-50") },
            RecentTrade { closed_at: now(), net_pnl: d("200") },   // win breaks streak
            RecentTrade { closed_at: now(), net_pnl: d("-25") },
        ];
        let rules = vec![RiskRule::MaxConsecutiveLossesToday { n: 3 }];
        let dec = evaluate(&proposed(), &c, &rules, now());
        assert!(dec.allow, "win must reset the loss streak");
    }

    // ─── CoolDownAfterLossMinutes ────────────────────────────────────────

    #[test]
    fn cool_down_blocks_within_window() {
        let now_ = now();
        let mut c = ctx();
        c.today_closed_trades = vec![
            RecentTrade {
                closed_at: now_ - Duration::minutes(3),    // 3 min ago
                net_pnl: d("-50"),
            },
        ];
        let rules = vec![RiskRule::CoolDownAfterLossMinutes { minutes: 10 }];
        let dec = evaluate(&proposed(), &c, &rules, now_);
        assert!(!dec.allow);
    }

    #[test]
    fn cool_down_allows_after_window() {
        let now_ = now();
        let mut c = ctx();
        c.today_closed_trades = vec![
            RecentTrade {
                closed_at: now_ - Duration::minutes(15),   // 15 min ago — past
                net_pnl: d("-50"),
            },
        ];
        let rules = vec![RiskRule::CoolDownAfterLossMinutes { minutes: 10 }];
        let dec = evaluate(&proposed(), &c, &rules, now_);
        assert!(dec.allow);
    }

    #[test]
    fn cool_down_ignores_winning_trades_inside_window() {
        let now_ = now();
        let mut c = ctx();
        c.today_closed_trades = vec![
            RecentTrade {
                closed_at: now_ - Duration::minutes(1),
                net_pnl: d("500"),   // win — should NOT trigger cool-down
            },
        ];
        let rules = vec![RiskRule::CoolDownAfterLossMinutes { minutes: 10 }];
        let dec = evaluate(&proposed(), &c, &rules, now_);
        assert!(dec.allow, "cool-down must only apply after a LOSING trade");
    }

    // ─── MaxOpenPositions ────────────────────────────────────────────────

    #[test]
    fn max_open_positions_inclusive_at_threshold() {
        let mut c = ctx();
        c.open_position_count = 5;
        let rules = vec![RiskRule::MaxOpenPositions { n: 5 }];
        let dec = evaluate(&proposed(), &c, &rules, now());
        assert!(!dec.allow, "rule fires at >=, not >");
    }

    // ─── MaxPositionSizePct ──────────────────────────────────────────────

    #[test]
    fn max_position_size_pct_blocks_oversized_notional() {
        // Notional = 100 × $150 × 1 = $15,000. 10% of $100k = $10k cap.
        let rules = vec![RiskRule::MaxPositionSizePct { pct: d("10") }];
        let dec = evaluate(&proposed(), &ctx(), &rules, now());
        assert!(!dec.allow);
    }

    // ─── BlockedSymbols ──────────────────────────────────────────────────

    #[test]
    fn blocked_symbols_are_case_insensitive() {
        let rules = vec![RiskRule::BlockedSymbols { symbols: vec!["aapl".into()] }];
        let dec = evaluate(&proposed(), &ctx(), &rules, now());  // proposed.symbol = "AAPL"
        assert!(!dec.allow);
    }

    #[test]
    fn blocked_symbols_pass_through_non_matches() {
        let rules = vec![RiskRule::BlockedSymbols { symbols: vec!["GME".into(), "AMC".into()] }];
        let dec = evaluate(&proposed(), &ctx(), &rules, now());
        assert!(dec.allow);
    }

    // ─── RequirePlanBeforeTrade ──────────────────────────────────────────

    #[test]
    fn require_plan_blocks_when_missing() {
        let mut p = proposed();
        p.has_attached_plan = false;
        let rules = vec![RiskRule::RequirePlanBeforeTrade];
        let dec = evaluate(&p, &ctx(), &rules, now());
        assert!(!dec.allow);
    }

    // ─── RequireStopLoss ─────────────────────────────────────────────────

    #[test]
    fn require_stop_loss_warns_does_not_block() {
        let mut p = proposed();
        p.stop_loss = None;
        let rules = vec![RiskRule::RequireStopLoss];
        let dec = evaluate(&p, &ctx(), &rules, now());
        // The rule is a warning, not a block — allow should still be true.
        assert!(dec.allow, "missing stop is a warning, not a block");
        assert_eq!(dec.violations.len(), 1);
        assert_eq!(dec.violations[0].severity, Severity::Warning);
    }

    // ─── multiple rules stack ────────────────────────────────────────────

    #[test]
    fn multiple_violations_stack_and_block_wins() {
        let mut p = proposed();
        p.stop_loss = None;          // warning from RequireStopLoss
        p.has_attached_plan = false; // block from RequirePlanBeforeTrade
        let rules = vec![
            RiskRule::RequireStopLoss,
            RiskRule::RequirePlanBeforeTrade,
        ];
        let dec = evaluate(&p, &ctx(), &rules, now());
        assert!(!dec.allow, "any Block severity must veto the trade");
        assert_eq!(dec.violations.len(), 2);
        assert_eq!(dec.warnings().count(), 1);
        assert_eq!(dec.blocks().count(), 1);
    }

    // ─── serde roundtrip ─────────────────────────────────────────────────

    #[test]
    fn rule_serde_roundtrip_persists_each_variant() {
        // The DB stores rules as JSONB — every variant must roundtrip.
        for rule in [
            RiskRule::MaxLossPerTradePct { pct: d("0.5") },
            RiskRule::MaxLossPerDayPct { pct: d("2") },
            RiskRule::MaxConsecutiveLossesToday { n: 3 },
            RiskRule::CoolDownAfterLossMinutes { minutes: 15 },
            RiskRule::MaxOpenPositions { n: 5 },
            RiskRule::MaxPositionSizePct { pct: d("10") },
            RiskRule::BlockedSymbols { symbols: vec!["GME".into()] },
            RiskRule::RequirePlanBeforeTrade,
            RiskRule::RequireStopLoss,
        ] {
            let s = serde_json::to_string(&rule).unwrap();
            let back: RiskRule = serde_json::from_str(&s)
                .unwrap_or_else(|e| panic!("failed to roundtrip {s}: {e}"));
            // Re-serialize and compare — gives us value-equality without
            // implementing PartialEq on the enum.
            assert_eq!(serde_json::to_string(&back).unwrap(), s);
        }
    }
}
