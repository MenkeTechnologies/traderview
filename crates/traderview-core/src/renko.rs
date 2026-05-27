//! Renko brick generator.
//!
//! Filters noise by emitting a new "brick" only when price moves by at
//! least `brick_size` from the prior brick's close. Direction reversals
//! require 2× the brick size (to avoid one-tick whipsaws). Time is
//! ignored — Renko charts are price-only.
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy)]
pub struct PriceTick {
    pub price: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BrickDirection {
    Up,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Brick {
    pub open: f64,
    pub close: f64,
    pub direction: BrickDirection,
}

pub fn build(ticks: &[PriceTick], brick_size: f64) -> Vec<Brick> {
    let mut out = Vec::new();
    if ticks.is_empty() || brick_size <= 0.0 {
        return out;
    }
    let mut last_close = ticks[0].price;
    let mut last_dir: Option<BrickDirection> = None;
    for t in &ticks[1..] {
        loop {
            match last_dir {
                None => {
                    if t.price - last_close >= brick_size {
                        let open = last_close;
                        let close = open + brick_size;
                        out.push(Brick {
                            open,
                            close,
                            direction: BrickDirection::Up,
                        });
                        last_close = close;
                        last_dir = Some(BrickDirection::Up);
                    } else if last_close - t.price >= brick_size {
                        let open = last_close;
                        let close = open - brick_size;
                        out.push(Brick {
                            open,
                            close,
                            direction: BrickDirection::Down,
                        });
                        last_close = close;
                        last_dir = Some(BrickDirection::Down);
                    } else {
                        break;
                    }
                }
                Some(BrickDirection::Up) => {
                    if t.price - last_close >= brick_size {
                        let open = last_close;
                        let close = open + brick_size;
                        out.push(Brick {
                            open,
                            close,
                            direction: BrickDirection::Up,
                        });
                        last_close = close;
                    } else if last_close - t.price >= 2.0 * brick_size {
                        // Reversal requires 2× brick.
                        let open = last_close - brick_size;
                        let close = open - brick_size;
                        out.push(Brick {
                            open,
                            close,
                            direction: BrickDirection::Down,
                        });
                        last_close = close;
                        last_dir = Some(BrickDirection::Down);
                    } else {
                        break;
                    }
                }
                Some(BrickDirection::Down) => {
                    if last_close - t.price >= brick_size {
                        let open = last_close;
                        let close = open - brick_size;
                        out.push(Brick {
                            open,
                            close,
                            direction: BrickDirection::Down,
                        });
                        last_close = close;
                    } else if t.price - last_close >= 2.0 * brick_size {
                        let open = last_close + brick_size;
                        let close = open + brick_size;
                        out.push(Brick {
                            open,
                            close,
                            direction: BrickDirection::Up,
                        });
                        last_close = close;
                        last_dir = Some(BrickDirection::Up);
                    } else {
                        break;
                    }
                }
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(price: f64) -> PriceTick {
        PriceTick { price }
    }

    #[test]
    fn empty_returns_empty() {
        assert!(build(&[], 1.0).is_empty());
    }

    #[test]
    fn zero_brick_size_returns_empty() {
        assert!(build(&[t(100.0), t(105.0)], 0.0).is_empty());
    }

    #[test]
    fn small_move_no_brick() {
        // Move of 0.5 with brick size 1.0 → no brick yet.
        let out = build(&[t(100.0), t(100.5)], 1.0);
        assert!(out.is_empty());
    }

    #[test]
    fn one_brick_size_up_emits_one_up_brick() {
        let out = build(&[t(100.0), t(101.0)], 1.0);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].direction, BrickDirection::Up);
        assert_eq!(out[0].open, 100.0);
        assert_eq!(out[0].close, 101.0);
    }

    #[test]
    fn multiple_brick_moves_emit_chain_of_bricks() {
        let out = build(&[t(100.0), t(105.0)], 1.0);
        assert_eq!(out.len(), 5);
        for (i, brick) in out.iter().enumerate() {
            assert_eq!(brick.direction, BrickDirection::Up);
            assert_eq!(brick.open, 100.0 + i as f64);
            assert_eq!(brick.close, 101.0 + i as f64);
        }
    }

    #[test]
    fn down_brick_emitted_on_decline() {
        let out = build(&[t(100.0), t(99.0)], 1.0);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].direction, BrickDirection::Down);
        assert_eq!(out[0].close, 99.0);
    }

    #[test]
    fn reversal_requires_two_x_brick_size() {
        // First up brick, then need 2x reversal to flip.
        let out = build(&[t(100.0), t(101.0), t(100.5)], 1.0);
        // After +1 up to 101, then -0.5 → not 2× brick → no reversal brick yet.
        assert_eq!(out.len(), 1);

        // But -2 from 101 (i.e. price 99) → reversal: emit one down brick.
        let out2 = build(&[t(100.0), t(101.0), t(99.0)], 1.0);
        assert_eq!(out2.len(), 2);
        assert_eq!(out2[1].direction, BrickDirection::Down);
        assert_eq!(out2[1].open, 100.0);
        assert_eq!(out2[1].close, 99.0);
    }

    #[test]
    fn small_oscillations_filter_out() {
        // Tiny moves around 100 with brick=1 → no bricks.
        let out = build(&[t(100.0), t(100.5), t(99.5), t(100.3), t(99.8)], 1.0);
        assert!(out.is_empty());
    }

    #[test]
    fn continuous_trend_emits_only_aligned_bricks() {
        // Strong uptrend — should never emit a down brick.
        let out = build(&[t(100.0), t(110.0), t(120.0)], 2.0);
        assert!(out.iter().all(|b| b.direction == BrickDirection::Up));
    }
}
