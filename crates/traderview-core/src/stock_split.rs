//! Stock-split position adjuster.
//!
//! A split changes the share count and per-share figures but not the total
//! value or total cost basis. Given a `new:old` ratio — `4:1` for a 4-for-1
//! forward split, `1:10` for a 1-for-10 reverse split — this scales a holding:
//!
//! ```text
//! factor          = new / old
//! post_shares     = shares × factor
//! post_price      = price / factor
//! post_basis/sh   = basis/sh / factor
//! ```
//!
//! Reverse splits and odd forward ratios can leave a fractional share; brokers
//! typically pay cash in lieu of it, so the whole/fractional split and the
//! cash-in-lieu value are reported too.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SplitInput {
    /// Shares held before the split.
    pub shares: f64,
    /// Cost basis per share before the split.
    pub cost_basis_per_share: f64,
    /// Market price per share before the split.
    pub price_per_share: f64,
    /// Ratio numerator: shares received per `split_old` held (e.g. 4 in 4:1).
    pub split_new: f64,
    /// Ratio denominator: shares given up (e.g. 1 in 4:1, 10 in 1:10).
    pub split_old: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct SplitResult {
    /// new / old.
    pub factor: f64,
    /// Shares after the split (may be fractional).
    pub post_shares: f64,
    /// Price per share after the split.
    pub post_price_per_share: f64,
    /// Cost basis per share after the split.
    pub post_basis_per_share: f64,
    /// Position market value — unchanged by the split.
    pub total_value: f64,
    /// Total cost basis — unchanged by the split.
    pub total_cost: f64,
    /// Whole shares the holder keeps.
    pub whole_shares: f64,
    /// Fractional share left over (0 when it divides evenly).
    pub fractional_shares: f64,
    /// Cash typically paid in lieu of the fractional share.
    pub cash_in_lieu: f64,
}

pub fn analyze(input: &SplitInput) -> SplitResult {
    // Guard a zero/negative denominator that would make the ratio meaningless.
    let factor = if input.split_old > 0.0 && input.split_new > 0.0 {
        input.split_new / input.split_old
    } else {
        1.0
    };

    let post_shares = input.shares * factor;
    let post_price = if factor != 0.0 {
        input.price_per_share / factor
    } else {
        input.price_per_share
    };
    let post_basis = if factor != 0.0 {
        input.cost_basis_per_share / factor
    } else {
        input.cost_basis_per_share
    };

    let whole = post_shares.floor();
    let fractional = post_shares - whole;

    SplitResult {
        factor,
        post_shares,
        post_price_per_share: post_price,
        post_basis_per_share: post_basis,
        total_value: input.shares * input.price_per_share,
        total_cost: input.shares * input.cost_basis_per_share,
        whole_shares: whole,
        fractional_shares: fractional,
        cash_in_lieu: fractional * post_price,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    fn run(shares: f64, basis: f64, price: f64, new: f64, old: f64) -> SplitResult {
        analyze(&SplitInput {
            shares,
            cost_basis_per_share: basis,
            price_per_share: price,
            split_new: new,
            split_old: old,
        })
    }

    #[test]
    fn forward_two_for_one_doubles_shares_halves_price() {
        let r = run(100.0, 50.0, 80.0, 2.0, 1.0);
        assert!(close(r.factor, 2.0));
        assert!(close(r.post_shares, 200.0));
        assert!(close(r.post_price_per_share, 40.0));
    }

    #[test]
    fn forward_four_for_one() {
        let r = run(10.0, 100.0, 600.0, 4.0, 1.0);
        assert!(close(r.post_shares, 40.0));
        assert!(close(r.post_price_per_share, 150.0));
        assert!(close(r.post_basis_per_share, 25.0));
    }

    #[test]
    fn reverse_one_for_ten() {
        let r = run(1000.0, 2.0, 0.5, 1.0, 10.0);
        assert!(close(r.factor, 0.1));
        assert!(close(r.post_shares, 100.0));
        assert!(close(r.post_price_per_share, 5.0));
    }

    #[test]
    fn total_value_unchanged() {
        let r = run(100.0, 50.0, 80.0, 3.0, 2.0);
        assert!(close(r.total_value, 8000.0));
        assert!(close(r.post_shares * r.post_price_per_share, 8000.0));
    }

    #[test]
    fn total_cost_unchanged() {
        let r = run(100.0, 50.0, 80.0, 3.0, 2.0);
        assert!(close(r.total_cost, 5000.0));
        assert!(close(r.post_shares * r.post_basis_per_share, 5000.0));
    }

    #[test]
    fn basis_per_share_adjusts() {
        let r = run(100.0, 60.0, 80.0, 2.0, 1.0);
        assert!(close(r.post_basis_per_share, 30.0));
    }

    #[test]
    fn fractional_share_and_cash_in_lieu() {
        // 10 shares reverse 1:3 → 3.3333 shares: 3 whole + 0.3333 frac.
        let r = run(10.0, 9.0, 3.0, 1.0, 3.0);
        assert!(close(r.whole_shares, 3.0));
        assert!(close(r.fractional_shares, 10.0 / 3.0 - 3.0));
        // post price = 3 / (1/3) = 9; cash = frac × 9.
        assert!(close(r.post_price_per_share, 9.0));
        assert!(close(r.cash_in_lieu, (10.0 / 3.0 - 3.0) * 9.0));
    }

    #[test]
    fn zero_denominator_guards_to_identity() {
        let r = run(100.0, 50.0, 80.0, 2.0, 0.0);
        assert!(close(r.factor, 1.0));
        assert!(close(r.post_shares, 100.0));
        assert!(close(r.post_price_per_share, 80.0));
    }
}
