//! Elder's Triple Screen — Dr. Alexander Elder.
//!
//! Three-timeframe filter for trade entries:
//!   1. **Long-tide**: weekly MACD or trend filter — defines bias.
//!   2. **Intermediate-wave**: daily oscillator (RSI / stochastic) —
//!      pulls back AGAINST the long tide → entry zone.
//!   3. **Short-ripple**: intraday breakout in long-tide direction —
//!      pulls the trigger.
//!
//! This module formalizes the verdict: given the three inputs, emit
//! Buy / Sell / Wait. Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct TripleScreenInput {
    pub weekly_trend: TrendBias,
    pub daily_oscillator_value: f64,
    /// Oversold threshold for daily oscillator (e.g. 30 for RSI).
    pub oversold_threshold: f64,
    /// Overbought threshold (e.g. 70 for RSI).
    pub overbought_threshold: f64,
    pub intraday_breakout_up: bool,
    pub intraday_breakout_down: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrendBias { Up, Down, Neutral }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict { Buy, Sell, Wait }

pub fn evaluate(input: &TripleScreenInput) -> Verdict {
    match input.weekly_trend {
        TrendBias::Up => {
            if input.daily_oscillator_value < input.oversold_threshold
                && input.intraday_breakout_up
            {
                Verdict::Buy
            } else {
                Verdict::Wait
            }
        }
        TrendBias::Down => {
            if input.daily_oscillator_value > input.overbought_threshold
                && input.intraday_breakout_down
            {
                Verdict::Sell
            } else {
                Verdict::Wait
            }
        }
        TrendBias::Neutral => Verdict::Wait,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> TripleScreenInput {
        TripleScreenInput {
            weekly_trend: TrendBias::Neutral,
            daily_oscillator_value: 50.0,
            oversold_threshold: 30.0,
            overbought_threshold: 70.0,
            intraday_breakout_up: false,
            intraday_breakout_down: false,
        }
    }

    #[test]
    fn all_three_aligned_long_buy() {
        let i = TripleScreenInput {
            weekly_trend: TrendBias::Up,
            daily_oscillator_value: 25.0,    // oversold
            intraday_breakout_up: true,
            ..baseline()
        };
        assert_eq!(evaluate(&i), Verdict::Buy);
    }

    #[test]
    fn all_three_aligned_short_sell() {
        let i = TripleScreenInput {
            weekly_trend: TrendBias::Down,
            daily_oscillator_value: 75.0,    // overbought
            intraday_breakout_down: true,
            ..baseline()
        };
        assert_eq!(evaluate(&i), Verdict::Sell);
    }

    #[test]
    fn weekly_up_but_oscillator_not_oversold_wait() {
        let i = TripleScreenInput {
            weekly_trend: TrendBias::Up,
            daily_oscillator_value: 50.0,    // not oversold
            intraday_breakout_up: true,
            ..baseline()
        };
        assert_eq!(evaluate(&i), Verdict::Wait);
    }

    #[test]
    fn weekly_up_oversold_but_no_intraday_breakout_wait() {
        let i = TripleScreenInput {
            weekly_trend: TrendBias::Up,
            daily_oscillator_value: 25.0,
            intraday_breakout_up: false,
            ..baseline()
        };
        assert_eq!(evaluate(&i), Verdict::Wait);
    }

    #[test]
    fn weekly_neutral_always_wait() {
        let i = TripleScreenInput {
            weekly_trend: TrendBias::Neutral,
            daily_oscillator_value: 25.0,
            intraday_breakout_up: true,
            ..baseline()
        };
        assert_eq!(evaluate(&i), Verdict::Wait);
    }

    #[test]
    fn weekly_up_blocks_sell_signal_even_if_overbought() {
        let i = TripleScreenInput {
            weekly_trend: TrendBias::Up,
            daily_oscillator_value: 80.0,
            intraday_breakout_down: true,    // shorting setup
            ..baseline()
        };
        assert_eq!(evaluate(&i), Verdict::Wait);    // long-tide says no
    }

    #[test]
    fn weekly_down_blocks_buy_signal() {
        let i = TripleScreenInput {
            weekly_trend: TrendBias::Down,
            daily_oscillator_value: 20.0,
            intraday_breakout_up: true,
            ..baseline()
        };
        assert_eq!(evaluate(&i), Verdict::Wait);
    }

    #[test]
    fn custom_thresholds_change_result() {
        // Wide oversold/overbought thresholds — easier to fire.
        let i = TripleScreenInput {
            weekly_trend: TrendBias::Up,
            daily_oscillator_value: 45.0,
            oversold_threshold: 50.0,       // looser
            overbought_threshold: 60.0,
            intraday_breakout_up: true,
            ..baseline()
        };
        assert_eq!(evaluate(&i), Verdict::Buy);
    }
}
