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
    /// Block if `now` is outside US regular trading hours (09:30-16:00 ET,
    /// Mon-Fri). Catches accidental after-hours entries when the user only
    /// wants RTH liquidity. NOT a holiday calendar — for "is the market
    /// closed today" use a separate calendar check.
    RegularTradingHoursOnly,
    /// Block if the proposed trade's notional dollar exposure (qty × entry
    /// × multiplier) is BELOW `min_dollars` — protects against fat-finger
    /// entries that are too small to be meaningful and would just burn fees.
    MinPositionSizeDollars { min_dollars: Decimal },
    /// Hard kill switch — always blocks. Install + enable to halt all
    /// trading without uninstalling other rules. Toggle off when ready
    /// to resume.
    KillSwitch,
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
        RiskRule::RegularTradingHoursOnly => {
            if !is_us_rth(now) {
                Some(Violation {
                    rule: "regular_trading_hours_only".into(),
                    severity: Severity::Block,
                    message: "outside US regular trading hours (09:30-16:00 ET, Mon-Fri)".into(),
                })
            } else { None }
        }
        RiskRule::MinPositionSizeDollars { min_dollars } => {
            let notional = p.qty * p.entry_price * p.multiplier;
            if notional < *min_dollars {
                Some(Violation {
                    rule: "min_position_size_dollars".into(),
                    severity: Severity::Block,
                    message: format!(
                        "notional ${} below ${} minimum — fat-finger guard",
                        notional, min_dollars,
                    ),
                })
            } else { None }
        }
        RiskRule::KillSwitch => Some(Violation {
            rule: "kill_switch".into(),
            severity: Severity::Block,
            message: "kill switch enabled — disable in Risk Gate to resume trading".into(),
        }),
    }
}

/// Is `now` inside US regular trading hours (09:30-16:00 America/New_York,
/// Mon-Fri)? Does NOT consult a holiday calendar — that's the caller's job
/// if needed. Implemented via chrono-tz-free UTC-offset math so we don't
/// pull a TZ database into the workspace for one check.
fn is_us_rth(now: DateTime<Utc>) -> bool {
    use chrono::{Datelike, Timelike, Weekday};
    // US Eastern is UTC-5 (EST) or UTC-4 (EDT). Approximation: DST runs
    // from second Sunday of March to first Sunday of November. This is a
    // best-effort guess; perfect handling requires chrono-tz. The error
    // window is at most one hour twice a year — acceptable for a gate
    // rule users can override.
    let month = now.month();
    let day   = now.day();
    let in_dst = match month {
        1 | 2 | 12 => false,
        3 => {
            // Second Sunday of March.
            let mar = chrono::NaiveDate::from_ymd_opt(now.year(), 3, 1).unwrap();
            let first_sun = (8 - (mar.weekday().num_days_from_sunday() as i32)).rem_euclid(7) + 1;
            let second_sun = first_sun + 7;
            day as i32 >= second_sun
        }
        11 => {
            // First Sunday of November.
            let nov = chrono::NaiveDate::from_ymd_opt(now.year(), 11, 1).unwrap();
            let first_sun = (8 - (nov.weekday().num_days_from_sunday() as i32)).rem_euclid(7) + 1;
            (day as i32) < first_sun
        }
        4..=10 => true,
        _ => false,
    };
    let offset_hours: i64 = if in_dst { -4 } else { -5 };
    let local = now + chrono::Duration::hours(offset_hours);
    match local.weekday() {
        Weekday::Sat | Weekday::Sun => return false,
        _ => {}
    }
    // 09:30 → 16:00 inclusive of 09:30, exclusive of 16:00.
    let h = local.hour();
    let m = local.minute();
    let minutes = h * 60 + m;
    minutes >= 9 * 60 + 30 && minutes < 16 * 60
}

// ---------------------------------------------------------------------------
// Rule presets — curated rule packs the user can install with one click.
// ---------------------------------------------------------------------------

/// Predefined rule pack identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Preset {
    /// Strict beginner ruleset. Forces plan + stop, hard daily cap,
    /// short streak threshold, cool-down after every loss.
    Beginner,
    /// Moderate ruleset for an experienced trader: 1% per trade, 4 streak,
    /// shorter cool-down, requires stop (not plan).
    Intermediate,
    /// Aggressive day-trader minimum: daily-loss cap + cool-down only.
    /// Assumes the user is disciplined enough to manage per-trade risk
    /// themselves.
    Aggressive,
}

pub fn preset_rules(preset: Preset) -> Vec<RiskRule> {
    use std::str::FromStr;
    let d = |s: &str| Decimal::from_str(s).unwrap();
    match preset {
        Preset::Beginner => vec![
            RiskRule::MaxLossPerTradePct { pct: d("1.0") },
            RiskRule::MaxLossPerDayPct   { pct: d("3.0") },
            RiskRule::MaxConsecutiveLossesToday { n: 3 },
            RiskRule::CoolDownAfterLossMinutes  { minutes: 15 },
            RiskRule::MaxPositionSizePct { pct: d("25") },
            RiskRule::RequirePlanBeforeTrade,
            RiskRule::RequireStopLoss,
        ],
        Preset::Intermediate => vec![
            RiskRule::MaxLossPerTradePct { pct: d("1.0") },
            RiskRule::MaxLossPerDayPct   { pct: d("5.0") },
            RiskRule::MaxConsecutiveLossesToday { n: 4 },
            RiskRule::CoolDownAfterLossMinutes  { minutes: 5 },
            RiskRule::RequireStopLoss,
        ],
        Preset::Aggressive => vec![
            RiskRule::MaxLossPerDayPct  { pct: d("8.0") },
            RiskRule::CoolDownAfterLossMinutes { minutes: 2 },
        ],
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
            RiskRule::RegularTradingHoursOnly,
            RiskRule::MinPositionSizeDollars { min_dollars: d("100") },
            RiskRule::KillSwitch,
        ] {
            let s = serde_json::to_string(&rule).unwrap();
            let back: RiskRule = serde_json::from_str(&s)
                .unwrap_or_else(|e| panic!("failed to roundtrip {s}: {e}"));
            // Re-serialize and compare — gives us value-equality without
            // implementing PartialEq on the enum.
            assert_eq!(serde_json::to_string(&back).unwrap(), s);
        }
    }

    // ─── RegularTradingHoursOnly + RTH detector ──────────────────────────

    /// Convert ET local clock to a `DateTime<Utc>` for testing the gate at
    /// a specific Eastern wall-clock time. Uses the same approximate DST
    /// rule the detector uses so tests + production stay in sync.
    fn et_to_utc(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
        use chrono::{Datelike, Weekday};
        let in_dst = match month {
            1 | 2 | 12 => false,
            3 => {
                let mar = chrono::NaiveDate::from_ymd_opt(year, 3, 1).unwrap();
                let first_sun = (8 - (mar.weekday().num_days_from_sunday() as i32)).rem_euclid(7) + 1;
                let second_sun = first_sun + 7;
                day as i32 >= second_sun
            }
            11 => {
                let nov = chrono::NaiveDate::from_ymd_opt(year, 11, 1).unwrap();
                let first_sun = (8 - (nov.weekday().num_days_from_sunday() as i32)).rem_euclid(7) + 1;
                (day as i32) < first_sun
            }
            4..=10 => true,
            _ => false,
        };
        let _ = Weekday::Mon; // silence unused-import warning if any
        let offset_hours: i64 = if in_dst { -4 } else { -5 };
        Utc.with_ymd_and_hms(year, month, day, hour, minute, 0).unwrap()
            - chrono::Duration::hours(offset_hours)
    }

    #[test]
    fn rth_rule_allows_tuesday_10am_et() {
        // Tuesday 2026-05-26 10:00 ET — RTH open.
        let now_ = et_to_utc(2026, 5, 26, 10, 0);
        let rules = vec![RiskRule::RegularTradingHoursOnly];
        let dec = evaluate(&proposed(), &ctx(), &rules, now_);
        assert!(dec.allow);
    }

    #[test]
    fn rth_rule_blocks_at_8am_et() {
        // Pre-market — before 09:30.
        let now_ = et_to_utc(2026, 5, 26, 8, 30);
        let rules = vec![RiskRule::RegularTradingHoursOnly];
        let dec = evaluate(&proposed(), &ctx(), &rules, now_);
        assert!(!dec.allow);
    }

    #[test]
    fn rth_rule_blocks_at_4pm_et() {
        // RTH is exclusive of 16:00 — first after-hours tick is at 4:00.
        let now_ = et_to_utc(2026, 5, 26, 16, 0);
        let rules = vec![RiskRule::RegularTradingHoursOnly];
        let dec = evaluate(&proposed(), &ctx(), &rules, now_);
        assert!(!dec.allow);
    }

    #[test]
    fn rth_rule_blocks_saturday() {
        // 2026-05-30 is a Saturday.
        let now_ = et_to_utc(2026, 5, 30, 12, 0);
        let rules = vec![RiskRule::RegularTradingHoursOnly];
        let dec = evaluate(&proposed(), &ctx(), &rules, now_);
        assert!(!dec.allow);
    }

    #[test]
    fn rth_rule_allows_exactly_at_open() {
        // 09:30:00 ET is the first allowed minute.
        let now_ = et_to_utc(2026, 5, 26, 9, 30);
        let rules = vec![RiskRule::RegularTradingHoursOnly];
        let dec = evaluate(&proposed(), &ctx(), &rules, now_);
        assert!(dec.allow, "09:30 ET must be inclusive open");
    }

    // ─── MinPositionSizeDollars (fat-finger guard) ───────────────────────

    #[test]
    fn min_position_size_blocks_tiny_notional() {
        // Notional = 100 × $150 = $15,000. Min $100k → block.
        let rules = vec![RiskRule::MinPositionSizeDollars { min_dollars: d("100000") }];
        let dec = evaluate(&proposed(), &ctx(), &rules, now());
        assert!(!dec.allow);
        assert!(dec.violations[0].message.contains("fat-finger"));
    }

    #[test]
    fn min_position_size_allows_at_or_above_minimum() {
        let rules = vec![RiskRule::MinPositionSizeDollars { min_dollars: d("15000") }];
        let dec = evaluate(&proposed(), &ctx(), &rules, now());
        assert!(dec.allow, "$15,000 notional at $15,000 floor should allow (>=)");
    }

    // ─── KillSwitch ──────────────────────────────────────────────────────

    #[test]
    fn kill_switch_blocks_unconditionally() {
        let rules = vec![RiskRule::KillSwitch];
        let dec = evaluate(&proposed(), &ctx(), &rules, now());
        assert!(!dec.allow);
        assert_eq!(dec.violations.len(), 1);
        assert_eq!(dec.violations[0].rule, "kill_switch");
    }

    #[test]
    fn kill_switch_blocks_even_with_perfect_setup() {
        // Even a textbook valid trade — plan attached, stop set, well
        // under every cap — must still be blocked by the kill switch.
        let mut p = proposed();
        p.has_attached_plan = true;
        p.stop_loss = Some(d("149.95"));   // tiny $5 risk
        let rules = vec![
            RiskRule::MaxLossPerTradePct { pct: d("10") },
            RiskRule::MaxLossPerDayPct   { pct: d("10") },
            RiskRule::RequirePlanBeforeTrade,
            RiskRule::RequireStopLoss,
            RiskRule::KillSwitch,
        ];
        let dec = evaluate(&p, &ctx(), &rules, now());
        assert!(!dec.allow, "kill switch must veto regardless of other rules");
    }

    // ─── Presets ─────────────────────────────────────────────────────────

    #[test]
    fn beginner_preset_is_strictest() {
        let b = preset_rules(Preset::Beginner);
        let i = preset_rules(Preset::Intermediate);
        let a = preset_rules(Preset::Aggressive);
        // More rules = more restrictions in this ruleset family.
        assert!(b.len() > i.len(),  "beginner ({}) should restrict more than intermediate ({})", b.len(), i.len());
        assert!(i.len() > a.len(), "intermediate ({}) should restrict more than aggressive ({})", i.len(), a.len());
    }

    #[test]
    fn beginner_preset_includes_plan_and_stop_requirements() {
        let rules = preset_rules(Preset::Beginner);
        let has_plan = rules.iter().any(|r| matches!(r, RiskRule::RequirePlanBeforeTrade));
        let has_stop = rules.iter().any(|r| matches!(r, RiskRule::RequireStopLoss));
        assert!(has_plan, "beginner must require a written plan");
        assert!(has_stop, "beginner must require a stop loss");
    }

    #[test]
    fn intermediate_preset_drops_plan_requirement_but_keeps_stop() {
        let rules = preset_rules(Preset::Intermediate);
        assert!(!rules.iter().any(|r| matches!(r, RiskRule::RequirePlanBeforeTrade)));
        assert!( rules.iter().any(|r| matches!(r, RiskRule::RequireStopLoss)));
    }

    #[test]
    fn aggressive_preset_is_daily_cap_plus_cool_down_only() {
        let rules = preset_rules(Preset::Aggressive);
        assert_eq!(rules.len(), 2,
            "aggressive must stay minimal — adding more defeats the purpose");
        assert!(rules.iter().any(|r| matches!(r, RiskRule::MaxLossPerDayPct { .. })));
        assert!(rules.iter().any(|r| matches!(r, RiskRule::CoolDownAfterLossMinutes { .. })));
    }

    #[test]
    fn presets_are_actually_usable_by_evaluate() {
        // Smoke test — every preset must successfully evaluate against the
        // sample proposed trade (no panic, no overflow).
        for p in [Preset::Beginner, Preset::Intermediate, Preset::Aggressive] {
            let rules = preset_rules(p);
            let _ = evaluate(&proposed(), &ctx(), &rules, now());
        }
    }

    #[test]
    fn preset_daily_caps_are_progressively_more_lenient() {
        let extract_daily = |p: Preset| -> Option<Decimal> {
            preset_rules(p).into_iter().find_map(|r| match r {
                RiskRule::MaxLossPerDayPct { pct } => Some(pct),
                _ => None,
            })
        };
        let b = extract_daily(Preset::Beginner).expect("beginner has daily cap");
        let i = extract_daily(Preset::Intermediate).expect("intermediate has daily cap");
        let a = extract_daily(Preset::Aggressive).expect("aggressive has daily cap");
        assert!(b < i, "beginner daily cap ({b}) must be tighter than intermediate ({i})");
        assert!(i < a, "intermediate cap ({i}) must be tighter than aggressive ({a})");
    }
}
