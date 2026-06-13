//! Bonus gross-up — the gross payment needed so the recipient nets a target
//! amount after tax. Used for grossing up bonuses, relocation, and other
//! make-whole payments.
//!
//! ```text
//! gross = net / (1 − combined withholding rate)
//! ```
//!
//! The federal supplemental-wage flat rate is 22% (37% above $1M); state and
//! the 7.65% FICA stack on top. The combined rate must be below 100% — there's
//! no gross-up if taxes would consume the entire payment.

use serde::{Deserialize, Serialize};

fn d_federal() -> f64 {
    22.0
}
fn d_fica() -> f64 {
    7.65
}

#[derive(Debug, Clone, Deserialize)]
pub struct GrossUpInput {
    /// Net amount the recipient should keep.
    pub desired_net_usd: f64,
    /// Federal withholding rate (22% IRS supplemental flat rate by default).
    #[serde(default = "d_federal")]
    pub federal_rate_pct: f64,
    #[serde(default)]
    pub state_rate_pct: f64,
    /// Include the 7.65% FICA (Social Security + Medicare) in the gross-up.
    #[serde(default)]
    pub include_fica: bool,
    #[serde(default = "d_fica")]
    pub fica_rate_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct GrossUpResult {
    /// Gross payment that nets the target; 0 if the combined rate ≥ 100%.
    pub gross_usd: f64,
    /// Combined withholding rate applied, percent.
    pub combined_rate_pct: f64,
    /// gross − net.
    pub total_tax_usd: f64,
    pub federal_withholding_usd: f64,
    pub state_withholding_usd: f64,
    pub fica_withholding_usd: f64,
}

pub fn analyze(input: &GrossUpInput) -> GrossUpResult {
    let fica = if input.include_fica {
        input.fica_rate_pct
    } else {
        0.0
    };
    let combined = input.federal_rate_pct + input.state_rate_pct + fica;

    let gross = if combined < 100.0 {
        input.desired_net_usd / (1.0 - combined / 100.0)
    } else {
        0.0
    };

    GrossUpResult {
        gross_usd: gross,
        combined_rate_pct: combined,
        total_tax_usd: gross - input.desired_net_usd,
        federal_withholding_usd: gross * input.federal_rate_pct / 100.0,
        state_withholding_usd: gross * input.state_rate_pct / 100.0,
        fica_withholding_usd: gross * fica / 100.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    fn run(net: f64, fed: f64, state: f64, fica: bool) -> GrossUpResult {
        analyze(&GrossUpInput {
            desired_net_usd: net,
            federal_rate_pct: fed,
            state_rate_pct: state,
            include_fica: fica,
            fica_rate_pct: 7.65,
        })
    }

    #[test]
    fn gross_with_fica() {
        // 1000 / (1 − 0.3465) = 1530.221882.
        let r = run(1000.0, 22.0, 5.0, true);
        assert!(close(r.gross_usd, 1530.221882));
        assert!(close(r.combined_rate_pct, 34.65));
    }

    #[test]
    fn gross_without_fica() {
        let r = run(1000.0, 22.0, 5.0, false);
        assert!(close(r.gross_usd, 1369.863014));
        assert!(close(r.combined_rate_pct, 27.0));
    }

    #[test]
    fn total_tax_is_gross_minus_net() {
        let r = run(1000.0, 22.0, 5.0, true);
        assert!(close(r.total_tax_usd, r.gross_usd - 1000.0));
    }

    #[test]
    fn withholdings_sum_to_total_tax() {
        let r = run(1000.0, 22.0, 5.0, true);
        let sum = r.federal_withholding_usd + r.state_withholding_usd + r.fica_withholding_usd;
        assert!(close(sum, r.total_tax_usd));
    }

    #[test]
    fn higher_state_raises_gross() {
        let low = run(1000.0, 22.0, 0.0, true);
        let high = run(1000.0, 22.0, 10.0, true);
        assert!(high.gross_usd > low.gross_usd);
    }

    #[test]
    fn fica_toggle_changes_gross() {
        let with = run(1000.0, 22.0, 5.0, true);
        let without = run(1000.0, 22.0, 5.0, false);
        assert!(with.gross_usd > without.gross_usd);
        assert!(close(without.fica_withholding_usd, 0.0));
    }

    #[test]
    fn nets_back_to_target() {
        // Gross minus all withholding equals the desired net.
        let r = run(1000.0, 22.0, 5.0, true);
        let net = r.gross_usd
            - r.federal_withholding_usd
            - r.state_withholding_usd
            - r.fica_withholding_usd;
        assert!(close(net, 1000.0));
    }

    #[test]
    fn combined_at_or_above_100_has_no_grossup() {
        let r = run(1000.0, 100.0, 5.0, true);
        assert!(close(r.gross_usd, 0.0));
    }
}
