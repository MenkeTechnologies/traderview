//! Dalton's Open Type classifier (Auction Market Theory).
//!
//! James Dalton's four "open types" — predictive day structure based on
//! how the first 30-60 minutes auctioned vs prior day's range:
//!
//!   - **OpenDrive**: opens AT or near prior-day extreme and drives
//!     through it without retracement. Strong directional day expected.
//!   - **OpenTestDrive**: opens, tests prior range briefly, then drives.
//!     Slightly less aggressive than Open-Drive.
//!   - **OpenRejectionReverse**: opens, makes brief move, gets rejected
//!     and reverses. Trend-day in opposite direction often follows.
//!   - **OpenAuction**: range-bound chop within prior-day value area.
//!     No directional conviction — fade extremes.
//!
//! Pure compute. Caller supplies the first-N-minute bar of today + the
//! prior-day's high/low/value-area.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct OpenInput {
    pub open_price: f64,
    /// First 30-60 minute bar — high and low of that opening range.
    pub opening_range_high: f64,
    pub opening_range_low: f64,
    pub opening_range_close: f64,
    pub prior_day_high: f64,
    pub prior_day_low: f64,
    pub prior_day_vah: f64,
    pub prior_day_val: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenType {
    OpenDrive,
    OpenTestDrive,
    OpenRejectionReverse,
    OpenAuction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenTypeReport {
    pub open_type: OpenType,
    pub above_prior_high: bool,
    pub below_prior_low: bool,
    pub inside_prior_value: bool,
    pub note: String,
}

pub fn classify(input: &OpenInput) -> OpenTypeReport {
    let above = input.opening_range_high > input.prior_day_high;
    let below = input.opening_range_low < input.prior_day_low;
    let inside = input.open_price >= input.prior_day_val && input.open_price <= input.prior_day_vah;

    // OpenDrive: opened AT prior extreme + drove past it AND closed near opposite end.
    let drive_up = input.open_price >= input.prior_day_high
        && above
        && input.opening_range_close >= input.opening_range_high * 0.95;
    let drive_down = input.open_price <= input.prior_day_low
        && below
        && input.opening_range_close <= input.opening_range_low * 1.05;
    if drive_up || drive_down {
        return OpenTypeReport {
            open_type: OpenType::OpenDrive,
            above_prior_high: above,
            below_prior_low: below,
            inside_prior_value: inside,
            note: "open drove through prior extreme — strong trend day expected".into(),
        };
    }

    // OpenTestDrive: opened in range, tested an extreme, then drove past.
    let test_drive_up = above
        && input.open_price < input.prior_day_high
        && input.opening_range_close > input.prior_day_high;
    let test_drive_down = below
        && input.open_price > input.prior_day_low
        && input.opening_range_close < input.prior_day_low;
    if test_drive_up || test_drive_down {
        return OpenTypeReport {
            open_type: OpenType::OpenTestDrive,
            above_prior_high: above,
            below_prior_low: below,
            inside_prior_value: inside,
            note: "tested prior range then broke out — moderate trend".into(),
        };
    }

    // OpenRejectionReverse: opened above (or below) prior, tested past
    // an extreme, then closed back inside.
    let reject_up = input.opening_range_high > input.prior_day_high
        && input.opening_range_close < input.prior_day_high;
    let reject_down = input.opening_range_low < input.prior_day_low
        && input.opening_range_close > input.prior_day_low;
    if reject_up || reject_down {
        return OpenTypeReport {
            open_type: OpenType::OpenRejectionReverse,
            above_prior_high: above,
            below_prior_low: below,
            inside_prior_value: inside,
            note: "tested past prior extreme and was rejected — trend reversal candidate".into(),
        };
    }

    // OpenAuction: range-bound chop, no clear conviction.
    OpenTypeReport {
        open_type: OpenType::OpenAuction,
        above_prior_high: above,
        below_prior_low: below,
        inside_prior_value: inside,
        note: "range-bound open — no conviction, fade extremes".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn baseline() -> OpenInput {
        OpenInput {
            open_price: 100.0,
            opening_range_high: 101.0,
            opening_range_low: 99.0,
            opening_range_close: 100.5,
            prior_day_high: 102.0,
            prior_day_low: 98.0,
            prior_day_vah: 101.0,
            prior_day_val: 99.0,
        }
    }

    #[test]
    fn neutral_open_inside_range_classified_as_auction() {
        let r = classify(&baseline());
        assert_eq!(r.open_type, OpenType::OpenAuction);
    }

    #[test]
    fn open_drive_up_when_opens_above_prior_high_and_drives() {
        let mut i = baseline();
        i.open_price = 103.0;
        i.opening_range_high = 105.0;
        i.opening_range_low = 102.5;
        i.opening_range_close = 105.0; // closed at high of OR
        let r = classify(&i);
        assert_eq!(r.open_type, OpenType::OpenDrive);
        assert!(r.above_prior_high);
    }

    #[test]
    fn open_drive_down_when_opens_below_prior_low_and_drives() {
        let mut i = baseline();
        i.open_price = 97.0;
        i.opening_range_high = 98.0;
        i.opening_range_low = 95.0;
        i.opening_range_close = 95.0; // closed at low of OR
        let r = classify(&i);
        assert_eq!(r.open_type, OpenType::OpenDrive);
        assert!(r.below_prior_low);
    }

    #[test]
    fn open_test_drive_up_when_opens_in_range_breaks_above() {
        let mut i = baseline();
        i.open_price = 101.0;
        i.opening_range_high = 103.0; // above prior high 102
        i.opening_range_close = 103.0;
        let r = classify(&i);
        assert_eq!(r.open_type, OpenType::OpenTestDrive);
    }

    #[test]
    fn open_rejection_reverse_up() {
        // OR pokes above prior high but closes back below.
        let mut i = baseline();
        i.opening_range_high = 103.0; // above prior 102
        i.opening_range_close = 100.0; // back below prior high
        let r = classify(&i);
        assert_eq!(r.open_type, OpenType::OpenRejectionReverse);
    }

    #[test]
    fn open_rejection_reverse_down() {
        let mut i = baseline();
        i.opening_range_low = 97.0; // below prior 98
        i.opening_range_close = 100.0; // back above prior low
        let r = classify(&i);
        assert_eq!(r.open_type, OpenType::OpenRejectionReverse);
    }

    #[test]
    fn inside_value_flag_correctly_set() {
        let r = classify(&baseline());
        assert!(r.inside_prior_value);
    }

    #[test]
    fn outside_value_flag_correctly_set() {
        let mut i = baseline();
        i.open_price = 103.0; // above VAH 101
        let r = classify(&i);
        assert!(!r.inside_prior_value);
    }
}
