//! Altman Z-Score — Edward Altman's 1968 bankruptcy-distress model for public
//! manufacturers. Five balance-sheet/income ratios, weighted into one score:
//!
//! ```text
//! Z = 1.2·X1 + 1.4·X2 + 3.3·X3 + 0.6·X4 + 1.0·X5
//!   X1 = working capital / total assets
//!   X2 = retained earnings / total assets
//!   X3 = EBIT / total assets
//!   X4 = market value of equity / total liabilities
//!   X5 = sales / total assets
//! ```
//!
//! Zones: Z > 2.99 is the safe zone, 1.81–2.99 is the grey zone, and below
//! 1.81 is the distress zone (elevated bankruptcy risk).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Zone {
    Safe,
    Grey,
    Distress,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AltmanInput {
    pub current_assets_usd: f64,
    pub current_liabilities_usd: f64,
    pub retained_earnings_usd: f64,
    /// Earnings before interest and taxes.
    pub ebit_usd: f64,
    /// Market value of equity (market cap).
    pub market_value_equity_usd: f64,
    pub total_liabilities_usd: f64,
    pub sales_usd: f64,
    pub total_assets_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AltmanResult {
    /// Working capital / total assets.
    pub x1_working_capital: f64,
    /// Retained earnings / total assets.
    pub x2_retained_earnings: f64,
    /// EBIT / total assets.
    pub x3_ebit: f64,
    /// Market value of equity / total liabilities.
    pub x4_equity_to_liabilities: f64,
    /// Sales / total assets.
    pub x5_sales: f64,
    /// The weighted Z-Score.
    pub z_score: f64,
    pub zone: Zone,
    /// Working capital (current assets − current liabilities), echoed.
    pub working_capital_usd: f64,
}

pub fn analyze(input: &AltmanInput) -> AltmanResult {
    let ta = input.total_assets_usd;
    let working_capital = input.current_assets_usd - input.current_liabilities_usd;

    let x1 = if ta > 0.0 { working_capital / ta } else { 0.0 };
    let x2 = if ta > 0.0 {
        input.retained_earnings_usd / ta
    } else {
        0.0
    };
    let x3 = if ta > 0.0 { input.ebit_usd / ta } else { 0.0 };
    let x4 = if input.total_liabilities_usd > 0.0 {
        input.market_value_equity_usd / input.total_liabilities_usd
    } else {
        0.0
    };
    let x5 = if ta > 0.0 { input.sales_usd / ta } else { 0.0 };

    let z = 1.2 * x1 + 1.4 * x2 + 3.3 * x3 + 0.6 * x4 + 1.0 * x5;

    let zone = if z > 2.99 {
        Zone::Safe
    } else if z >= 1.81 {
        Zone::Grey
    } else {
        Zone::Distress
    };

    AltmanResult {
        x1_working_capital: x1,
        x2_retained_earnings: x2,
        x3_ebit: x3,
        x4_equity_to_liabilities: x4,
        x5_sales: x5,
        z_score: z,
        zone,
        working_capital_usd: working_capital,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-4
    }

    fn safe() -> AltmanInput {
        AltmanInput {
            current_assets_usd: 500.0,
            current_liabilities_usd: 200.0,
            retained_earnings_usd: 400.0,
            ebit_usd: 150.0,
            market_value_equity_usd: 600.0,
            total_liabilities_usd: 400.0,
            sales_usd: 1200.0,
            total_assets_usd: 1000.0,
        }
    }

    #[test]
    fn ratios_computed() {
        let r = analyze(&safe());
        assert!(close(r.working_capital_usd, 300.0));
        assert!(close(r.x1_working_capital, 0.3));
        assert!(close(r.x2_retained_earnings, 0.4));
        assert!(close(r.x3_ebit, 0.15));
        assert!(close(r.x4_equity_to_liabilities, 1.5));
        assert!(close(r.x5_sales, 1.2));
    }

    #[test]
    fn z_score_weighted_sum() {
        // 0.36 + 0.56 + 0.495 + 0.9 + 1.2 = 3.515.
        let r = analyze(&safe());
        assert!(close(r.z_score, 3.515));
    }

    #[test]
    fn safe_zone() {
        let r = analyze(&safe());
        assert_eq!(r.zone, Zone::Safe);
    }

    #[test]
    fn grey_zone() {
        // Constructed for Z = 2.24.
        let r = analyze(&AltmanInput {
            current_assets_usd: 400.0,
            current_liabilities_usd: 250.0,
            retained_earnings_usd: 250.0,
            ebit_usd: 100.0,
            market_value_equity_usd: 400.0,
            total_liabilities_usd: 500.0,
            sales_usd: 900.0,
            total_assets_usd: 1000.0,
        });
        assert!(close(r.z_score, 2.24));
        assert_eq!(r.zone, Zone::Grey);
    }

    #[test]
    fn distress_zone() {
        let r = analyze(&AltmanInput {
            current_assets_usd: 200.0,
            current_liabilities_usd: 400.0,
            retained_earnings_usd: -100.0,
            ebit_usd: 10.0,
            market_value_equity_usd: 100.0,
            total_liabilities_usd: 900.0,
            sales_usd: 300.0,
            total_assets_usd: 1000.0,
        });
        assert!(r.z_score < 1.81);
        assert_eq!(r.zone, Zone::Distress);
    }

    #[test]
    fn boundary_181_is_grey() {
        // Z exactly 1.81 sits in the grey zone (inclusive lower bound).
        // Pure X5: sales/TA = 1.81 → all weight via the 1.0 coefficient.
        let r = analyze(&AltmanInput {
            current_assets_usd: 0.0,
            current_liabilities_usd: 0.0,
            retained_earnings_usd: 0.0,
            ebit_usd: 0.0,
            market_value_equity_usd: 0.0,
            total_liabilities_usd: 1.0,
            sales_usd: 1810.0,
            total_assets_usd: 1000.0,
        });
        assert!(close(r.z_score, 1.81));
        assert_eq!(r.zone, Zone::Grey);
    }

    #[test]
    fn zero_total_assets_guards() {
        let r = analyze(&AltmanInput {
            current_assets_usd: 100.0,
            current_liabilities_usd: 50.0,
            retained_earnings_usd: 100.0,
            ebit_usd: 50.0,
            market_value_equity_usd: 100.0,
            total_liabilities_usd: 100.0,
            sales_usd: 200.0,
            total_assets_usd: 0.0,
        });
        assert!(close(r.x1_working_capital, 0.0));
        assert!(close(r.x5_sales, 0.0));
        // Only X4 (equity/liabilities) survives: 0.6 × 1.0 = 0.6 → distress.
        assert!(close(r.z_score, 0.6));
        assert_eq!(r.zone, Zone::Distress);
    }

    #[test]
    fn zero_liabilities_guards_x4() {
        let r = analyze(&AltmanInput {
            current_assets_usd: 500.0,
            current_liabilities_usd: 200.0,
            retained_earnings_usd: 400.0,
            ebit_usd: 150.0,
            market_value_equity_usd: 600.0,
            total_liabilities_usd: 0.0,
            sales_usd: 1200.0,
            total_assets_usd: 1000.0,
        });
        assert!(close(r.x4_equity_to_liabilities, 0.0));
    }
}
