//! Daily-loss kill-switch checker.
//!
//! Enforces hard daily-loss limits — common discipline tool. Three tiers:
//!   - **Warning**: 50% of limit hit (yellow alert)
//!   - **CutSize**: 75% hit (halve position sizes)
//!   - **KillSwitch**: 100% hit (stop trading for the day)
//!
//! Configurable. Pure compute.

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
#[cfg(test)]
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LossLimitConfig {
    /// Hard daily-loss cap in account currency.
    pub max_daily_loss_dollars: Decimal,
    /// Cap as percent of equity (alternative; use whichever binds first).
    pub max_daily_loss_pct: Decimal,
    pub account_equity: Decimal,
    pub warning_threshold: Decimal,
    pub cut_size_threshold: Decimal,
    pub kill_threshold: Decimal,
}

impl LossLimitConfig {
    pub fn binding_limit(&self) -> Decimal {
        let pct_limit = self.account_equity * self.max_daily_loss_pct;
        if self.max_daily_loss_dollars > Decimal::ZERO && self.max_daily_loss_dollars < pct_limit {
            self.max_daily_loss_dollars
        } else {
            pct_limit
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LossState {
    #[default]
    OK,
    Warning,
    CutSize,
    KillSwitch,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LossLimitReport {
    pub today_realized_loss: Decimal,
    pub binding_limit: Decimal,
    pub pct_of_limit: Decimal,
    pub state: LossState,
    pub note: String,
}

pub fn evaluate(today_pnl: Decimal, cfg: &LossLimitConfig) -> LossLimitReport {
    let loss = if today_pnl < Decimal::ZERO {
        -today_pnl
    } else {
        Decimal::ZERO
    };
    let limit = cfg.binding_limit();
    let pct = if limit > Decimal::ZERO {
        loss / limit
    } else {
        Decimal::ZERO
    };
    let state = if pct >= cfg.kill_threshold {
        LossState::KillSwitch
    } else if pct >= cfg.cut_size_threshold {
        LossState::CutSize
    } else if pct >= cfg.warning_threshold {
        LossState::Warning
    } else {
        LossState::OK
    };
    let note = match state {
        LossState::OK => "within limits".into(),
        LossState::Warning => format!(
            "warning: {:.1}% of daily loss budget used",
            pct * Decimal::from(100)
        ),
        LossState::CutSize => "cut size: half-size positions only".into(),
        LossState::KillSwitch => "KILL SWITCH: stop trading for the day".into(),
    };
    LossLimitReport {
        today_realized_loss: loss,
        binding_limit: limit,
        pct_of_limit: pct,
        state,
        note,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    fn cfg() -> LossLimitConfig {
        LossLimitConfig {
            max_daily_loss_dollars: d("2000"),
            max_daily_loss_pct: d("0.02"),
            account_equity: d("100000"),
            warning_threshold: d("0.50"),
            cut_size_threshold: d("0.75"),
            kill_threshold: d("1.00"),
        }
    }

    #[test]
    fn positive_pnl_state_ok() {
        let r = evaluate(d("500"), &cfg());
        assert_eq!(r.state, LossState::OK);
    }

    #[test]
    fn small_loss_under_50pct_state_ok() {
        let r = evaluate(d("-500"), &cfg());
        assert_eq!(r.state, LossState::OK);
    }

    #[test]
    fn fifty_pct_of_limit_warning() {
        // Loss = $1000 = 50% of $2000 limit.
        let r = evaluate(d("-1000"), &cfg());
        assert_eq!(r.state, LossState::Warning);
    }

    #[test]
    fn seventy_five_pct_cut_size() {
        let r = evaluate(d("-1500"), &cfg());
        assert_eq!(r.state, LossState::CutSize);
    }

    #[test]
    fn hundred_pct_kill_switch() {
        let r = evaluate(d("-2000"), &cfg());
        assert_eq!(r.state, LossState::KillSwitch);
    }

    #[test]
    fn over_limit_still_kill_switch() {
        let r = evaluate(d("-3000"), &cfg());
        assert_eq!(r.state, LossState::KillSwitch);
    }

    #[test]
    fn binding_limit_picks_smaller_of_dollar_or_pct() {
        // $2000 cap vs 2% × $100k = $2000 → tie, dollar wins (lower or equal).
        let c = cfg();
        assert_eq!(c.binding_limit(), d("2000"));
        // Make pct limit smaller — switch.
        let strict = LossLimitConfig {
            max_daily_loss_pct: d("0.01"),
            ..c
        };
        // 1% × $100k = $1000 < $2000 → pct binds.
        assert_eq!(strict.binding_limit(), d("1000"));
    }

    #[test]
    fn pct_of_limit_correct_for_partial_loss() {
        let r = evaluate(d("-500"), &cfg());
        // 500 / 2000 = 0.25.
        assert_eq!(r.pct_of_limit, d("0.25"));
    }

    #[test]
    fn note_explains_state() {
        let r = evaluate(d("-2000"), &cfg());
        assert!(r.note.contains("KILL"));
    }
}
