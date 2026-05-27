//! Time-in-force order validator.
//!
//! Validates an order's TIF semantics against current time + venue
//! hours + remaining fill quantity:
//!   - DAY: expires at session close.
//!   - GTC: lives until cancelled or 90-day broker timeout.
//!   - IOC: fill immediately what's available, cancel the rest.
//!   - FOK: fill the entire qty immediately or cancel.
//!   - GTD: valid until a specific date.
//!
//! Pure compute. Caller passes current state; engine emits decisions.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeInForce {
    Day,
    Gtc,
    Ioc,
    Fok,
    Gtd,    // requires good_until field
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderState {
    pub tif: TimeInForce,
    pub original_qty: f64,
    pub filled_qty: f64,
    pub placed_at: DateTime<Utc>,
    pub good_until: Option<NaiveDate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TifAction {
    /// Order remains live, awaiting fills.
    Keep,
    /// Cancel — TIF condition no longer satisfied.
    Cancel,
    /// Order fully filled.
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TifVerdict {
    pub action: TifAction,
    pub reason: String,
}

pub fn evaluate(order: &OrderState, now: DateTime<Utc>, session_open: NaiveDate)
    -> TifVerdict
{
    let remaining = order.original_qty - order.filled_qty;
    if remaining <= 0.0 {
        return TifVerdict {
            action: TifAction::Completed,
            reason: "fully filled".into(),
        };
    }
    match order.tif {
        TimeInForce::Day => {
            // DAY orders expire if today's session is different from placement.
            let placed_date = order.placed_at.date_naive();
            if session_open > placed_date {
                TifVerdict {
                    action: TifAction::Cancel,
                    reason: "DAY order rolled into new session — expire".into(),
                }
            } else {
                TifVerdict {
                    action: TifAction::Keep,
                    reason: "DAY order still in session".into(),
                }
            }
        }
        TimeInForce::Gtc => {
            // 90-day broker timeout convention.
            let age_days = (now - order.placed_at).num_days();
            if age_days > 90 {
                TifVerdict {
                    action: TifAction::Cancel,
                    reason: "GTC order exceeded 90-day broker timeout".into(),
                }
            } else {
                TifVerdict {
                    action: TifAction::Keep,
                    reason: format!("GTC order, age {} days", age_days),
                }
            }
        }
        TimeInForce::Ioc => {
            // IOC: cancel any unfilled portion immediately.
            TifVerdict {
                action: TifAction::Cancel,
                reason: format!("IOC: cancel {} unfilled qty", remaining),
            }
        }
        TimeInForce::Fok => {
            // FOK: if not fully filled at first chance, cancel.
            if order.filled_qty == 0.0 {
                TifVerdict {
                    action: TifAction::Cancel,
                    reason: "FOK: no fill available, cancel entire order".into(),
                }
            } else if remaining > 0.0 {
                // Partial fill on FOK shouldn't happen but defensive: cancel.
                TifVerdict {
                    action: TifAction::Cancel,
                    reason: "FOK: partial fill not allowed, cancel rest".into(),
                }
            } else {
                TifVerdict {
                    action: TifAction::Completed,
                    reason: "FOK: fully filled".into(),
                }
            }
        }
        TimeInForce::Gtd => {
            match order.good_until {
                Some(good) if session_open > good => TifVerdict {
                    action: TifAction::Cancel,
                    reason: format!("GTD order past good_until date {}", good),
                },
                Some(good) => TifVerdict {
                    action: TifAction::Keep,
                    reason: format!("GTD valid until {}", good),
                },
                None => TifVerdict {
                    action: TifAction::Cancel,
                    reason: "GTD missing good_until date".into(),
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(y: i32, m: u32, d: u32, h: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, h, 0, 0).unwrap()
    }
    fn day(y: i32, m: u32, d: u32) -> NaiveDate { NaiveDate::from_ymd_opt(y, m, d).unwrap() }

    fn ord(tif: TimeInForce, placed: DateTime<Utc>, filled: f64) -> OrderState {
        OrderState {
            tif,
            original_qty: 100.0,
            filled_qty: filled,
            placed_at: placed,
            good_until: None,
        }
    }

    #[test]
    fn fully_filled_order_completed() {
        let o = ord(TimeInForce::Gtc, at(2026, 5, 27, 10), 100.0);
        let v = evaluate(&o, at(2026, 5, 27, 11), day(2026, 5, 27));
        assert_eq!(v.action, TifAction::Completed);
    }

    #[test]
    fn day_order_within_session_keep() {
        let o = ord(TimeInForce::Day, at(2026, 5, 27, 10), 0.0);
        let v = evaluate(&o, at(2026, 5, 27, 14), day(2026, 5, 27));
        assert_eq!(v.action, TifAction::Keep);
    }

    #[test]
    fn day_order_next_session_cancel() {
        let o = ord(TimeInForce::Day, at(2026, 5, 27, 10), 0.0);
        let v = evaluate(&o, at(2026, 5, 28, 14), day(2026, 5, 28));
        assert_eq!(v.action, TifAction::Cancel);
    }

    #[test]
    fn gtc_within_90_days_keep() {
        let o = ord(TimeInForce::Gtc, at(2026, 5, 27, 10), 0.0);
        let v = evaluate(&o, at(2026, 6, 27, 10), day(2026, 6, 27));
        assert_eq!(v.action, TifAction::Keep);
    }

    #[test]
    fn gtc_past_90_days_cancel() {
        let o = ord(TimeInForce::Gtc, at(2026, 1, 1, 10), 0.0);
        let v = evaluate(&o, at(2026, 5, 27, 10), day(2026, 5, 27));
        assert_eq!(v.action, TifAction::Cancel);
    }

    #[test]
    fn ioc_with_remaining_qty_cancel() {
        let o = ord(TimeInForce::Ioc, at(2026, 5, 27, 10), 50.0);    // 50 still unfilled
        let v = evaluate(&o, at(2026, 5, 27, 10), day(2026, 5, 27));
        assert_eq!(v.action, TifAction::Cancel);
    }

    #[test]
    fn fok_no_fill_cancel() {
        let o = ord(TimeInForce::Fok, at(2026, 5, 27, 10), 0.0);
        let v = evaluate(&o, at(2026, 5, 27, 10), day(2026, 5, 27));
        assert_eq!(v.action, TifAction::Cancel);
    }

    #[test]
    fn fok_partial_fill_cancel() {
        let o = ord(TimeInForce::Fok, at(2026, 5, 27, 10), 50.0);
        let v = evaluate(&o, at(2026, 5, 27, 10), day(2026, 5, 27));
        assert_eq!(v.action, TifAction::Cancel);
    }

    #[test]
    fn gtd_within_date_keep() {
        let mut o = ord(TimeInForce::Gtd, at(2026, 5, 27, 10), 0.0);
        o.good_until = Some(day(2026, 6, 30));
        let v = evaluate(&o, at(2026, 6, 1, 10), day(2026, 6, 1));
        assert_eq!(v.action, TifAction::Keep);
    }

    #[test]
    fn gtd_past_date_cancel() {
        let mut o = ord(TimeInForce::Gtd, at(2026, 5, 27, 10), 0.0);
        o.good_until = Some(day(2026, 6, 1));
        let v = evaluate(&o, at(2026, 7, 1, 10), day(2026, 7, 1));
        assert_eq!(v.action, TifAction::Cancel);
    }

    #[test]
    fn gtd_missing_date_cancel() {
        let o = ord(TimeInForce::Gtd, at(2026, 5, 27, 10), 0.0);
        let v = evaluate(&o, at(2026, 5, 27, 10), day(2026, 5, 27));
        assert_eq!(v.action, TifAction::Cancel);
    }
}
