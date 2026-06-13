//! Forex desk math: pip value, risk-based position sizing, and the
//! 24/5 trading-session clock.
//!
//! Carry and covered-interest-parity forward pricing live in
//! [`crate::fx_carry`]; this module is the position-desk companion that
//! turns a risk budget into a lot size and tells you which centers are
//! open. Pure functions — the caller supplies the clock.

use serde::{Deserialize, Serialize};

/// Pip size for a canonical pair: 0.01 for JPY-quoted pairs (USDJPY,
/// EURJPY, …), 0.0001 otherwise. The fourth decimal is the pip for most
/// pairs; JPY pairs quote to two decimals, so the pip is the second.
/// This is the single source of truth — `traderview_db::forex::pip_size`
/// delegates here so the fill engine and the calculators can't diverge.
pub fn pip_size(pair: &str) -> f64 {
    if pair.ends_with("JPY") {
        0.01
    } else {
        0.0001
    }
}

/// Value of one pip on `units` of the base currency, in the QUOTE
/// currency: `pip_size × |units|`. For USD-quoted majors (EURUSD,
/// GBPUSD) this is USD directly; for other quotes it is denominated in
/// that currency.
pub fn pip_value(pair: &str, units: f64) -> f64 {
    pip_size(pair) * units.abs()
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct PositionSize {
    /// Base-currency units the risk budget supports.
    pub units: f64,
    /// Units expressed as standard (100k), mini (10k), micro (1k) lots.
    pub standard_lots: f64,
    pub mini_lots: f64,
    pub micro_lots: f64,
    /// The risked amount = equity × risk_pct / 100.
    pub risk_amount: f64,
    /// Pip value of the sized position, in the quote currency.
    pub pip_value: f64,
}

/// Risk-based FX sizing. Given account equity, the fraction of it to
/// risk, the stop distance in pips, and the pair, return the position
/// whose loss at the stop equals the risked amount:
///
/// `units = (equity × risk_pct/100) / (stop_pips × pip_size)`
///
/// Exact when the quote currency is the account currency (USD-quoted
/// majors); for other quotes the units are correct and the loss is in
/// the quote currency. `None` on non-positive inputs.
pub fn position_size(
    equity: f64,
    risk_pct: f64,
    stop_pips: f64,
    pair: &str,
) -> Option<PositionSize> {
    if equity <= 0.0 || risk_pct <= 0.0 || stop_pips <= 0.0 {
        return None;
    }
    let risk_amount = equity * risk_pct / 100.0;
    let loss_per_unit = stop_pips * pip_size(pair);
    if loss_per_unit <= 0.0 {
        return None;
    }
    let units = risk_amount / loss_per_unit;
    Some(PositionSize {
        units,
        standard_lots: units / 100_000.0,
        mini_lots: units / 10_000.0,
        micro_lots: units / 1_000.0,
        risk_amount,
        pip_value: pip_value(pair, units),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Session {
    Sydney,
    Tokyo,
    London,
    #[serde(rename = "newyork")]
    NewYork,
}

impl Session {
    pub const ALL: [Session; 4] =
        [Session::Sydney, Session::Tokyo, Session::London, Session::NewYork];

    /// Standard UTC window `[open, close)` for the session. Sydney wraps
    /// midnight (21:00→06:00). Windows are fixed-UTC approximations of
    /// the conventional desk hours; real centers shift ±1h with their
    /// own DST, which this clock does not model.
    fn window(self) -> (u32, u32) {
        match self {
            Session::Sydney => (21, 6),
            Session::Tokyo => (0, 9),
            Session::London => (8, 17),
            Session::NewYork => (13, 22),
        }
    }

    fn contains(self, hour: u32) -> bool {
        let (open, close) = self.window();
        if open <= close {
            (open..close).contains(&hour)
        } else {
            hour >= open || hour < close
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SessionStatus {
    /// True Sun 21:00 UTC → Fri 21:00 UTC; false over the weekend.
    pub market_open: bool,
    /// Centers currently trading (empty when the market is closed).
    pub active: Vec<Session>,
    /// London/New York overlap (≈13:00–16:00 UTC) — the deepest
    /// liquidity window of the FX day.
    pub london_ny_overlap: bool,
}

/// FX session state for a UTC `weekday` (chrono's days-from-Monday:
/// 0=Mon … 6=Sun) and `hour` (0–23). The cash market runs continuously
/// from Sunday 21:00 UTC (Sydney open) to Friday 21:00 UTC.
pub fn session_status(weekday: u32, hour: u32) -> SessionStatus {
    let market_open = match weekday {
        5 => false,      // Saturday — closed all day
        6 => hour >= 21, // Sunday — opens 21:00 UTC at the Sydney bell
        4 => hour < 21,  // Friday — closes 21:00 UTC
        _ => true,       // Mon–Thu — open around the clock
    };
    let active: Vec<Session> = if market_open {
        Session::ALL.into_iter().filter(|s| s.contains(hour)).collect()
    } else {
        Vec::new()
    };
    let london_ny_overlap =
        market_open && Session::London.contains(hour) && Session::NewYork.contains(hour);
    SessionStatus { market_open, active, london_ny_overlap }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pip_value_usd_quoted_major() {
        // One standard lot of EURUSD: 1 pip = $10.
        assert!((pip_value("EURUSD", 100_000.0) - 10.0).abs() < 1e-9);
    }

    #[test]
    fn pip_value_jpy_pair() {
        // USDJPY pip is 0.01: 100k units → 1000 JPY per pip.
        assert!((pip_value("USDJPY", 100_000.0) - 1000.0).abs() < 1e-9);
    }

    #[test]
    fn position_size_matches_risk_at_stop() {
        // $10k equity, 1% risk = $100; 20-pip stop on EURUSD.
        // units = 100 / (20 × 0.0001) = 50,000 (half a standard lot).
        let p = position_size(10_000.0, 1.0, 20.0, "EURUSD").unwrap();
        assert!((p.units - 50_000.0).abs() < 1e-6);
        assert!((p.standard_lots - 0.5).abs() < 1e-9);
        assert!((p.risk_amount - 100.0).abs() < 1e-9);
        // Loss at the stop must equal the risked amount.
        let loss = 20.0 * pip_size("EURUSD") * p.units;
        assert!((loss - p.risk_amount).abs() < 1e-6);
    }

    #[test]
    fn position_size_rejects_nonpositive() {
        assert!(position_size(0.0, 1.0, 20.0, "EURUSD").is_none());
        assert!(position_size(10_000.0, 0.0, 20.0, "EURUSD").is_none());
        assert!(position_size(10_000.0, 1.0, 0.0, "EURUSD").is_none());
    }

    #[test]
    fn weekend_is_closed() {
        assert!(!session_status(5, 12).market_open); // Saturday noon
        assert!(session_status(5, 12).active.is_empty());
        assert!(!session_status(6, 20).market_open); // Sunday 20:00, pre-open
    }

    #[test]
    fn sydney_opens_sunday_2100_utc() {
        let s = session_status(6, 22); // Sunday 22:00 UTC
        assert!(s.market_open);
        assert!(s.active.contains(&Session::Sydney));
    }

    #[test]
    fn friday_closes_at_2100_utc() {
        assert!(session_status(4, 20).market_open); // Fri 20:00 — still open
        assert!(!session_status(4, 21).market_open); // Fri 21:00 — closed
    }

    #[test]
    fn london_ny_overlap_window() {
        // Wednesday 14:00 UTC: London (08–17) and NY (13–22) both open.
        let s = session_status(2, 14);
        assert!(s.london_ny_overlap);
        assert!(s.active.contains(&Session::London));
        assert!(s.active.contains(&Session::NewYork));
        // 10:00 UTC: London open, NY not yet → no overlap.
        assert!(!session_status(2, 10).london_ny_overlap);
    }

    #[test]
    fn asian_session_overlap() {
        // Wednesday 03:00 UTC: Sydney (wraps, <06) and Tokyo (00–09).
        let s = session_status(2, 3);
        assert!(s.active.contains(&Session::Sydney));
        assert!(s.active.contains(&Session::Tokyo));
        assert!(!s.london_ny_overlap);
    }
}
