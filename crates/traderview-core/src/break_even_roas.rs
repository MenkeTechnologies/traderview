//! Break-even ROAS (advertising profitability) — for a marketing campaign, the
//! return on ad spend (revenue ÷ ad spend) and the break-even ROAS at which the
//! gross profit on the revenue exactly covers the ad spend (1 ÷ gross-margin
//! ratio). It also reports gross profit, the contribution left after ad spend, and
//! the profit per advertising dollar. A campaign is profitable when its ROAS
//! exceeds the break-even ROAS. Distinct from the LTV/CAC module. Not advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct RoasInput {
    pub campaign_label: String,
    pub revenue_usd: f64,
    pub ad_spend_usd: f64,
    /// Gross (variable) margin on revenue, percent.
    pub gross_margin_pct: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct RoasReport {
    /// Revenue ÷ ad spend.
    pub roas: f64,
    /// 1 ÷ gross-margin ratio — the ROAS where gross profit = ad spend.
    pub break_even_roas: f64,
    pub gross_profit_usd: f64,
    /// Gross profit − ad spend.
    pub contribution_after_ads_usd: f64,
    /// Contribution after ads ÷ ad spend.
    pub profit_per_ad_dollar: f64,
    pub profitable: bool,
    pub valid: bool,
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

fn round4(x: f64) -> f64 {
    (x * 10_000.0).round() / 10_000.0
}

pub fn generate(i: &RoasInput) -> RoasReport {
    if i.ad_spend_usd <= 0.0 || i.gross_margin_pct <= 0.0 {
        return RoasReport::default();
    }
    let roas = i.revenue_usd / i.ad_spend_usd;
    let break_even = 100.0 / i.gross_margin_pct;
    let gross_profit = i.revenue_usd * i.gross_margin_pct / 100.0;
    let contribution = gross_profit - i.ad_spend_usd;
    RoasReport {
        roas: round4(roas),
        break_even_roas: round4(break_even),
        gross_profit_usd: cents(gross_profit),
        contribution_after_ads_usd: cents(contribution),
        profit_per_ad_dollar: round4(contribution / i.ad_spend_usd),
        profitable: roas > break_even,
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> RoasInput {
        RoasInput {
            campaign_label: "Spring sale".into(),
            revenue_usd: 10_000.0,
            ad_spend_usd: 2_500.0,
            gross_margin_pct: 40.0,
        }
    }

    #[test]
    fn profitable_campaign() {
        let d = generate(&base());
        assert!(close(d.roas, 4.0));
        assert!(close(d.break_even_roas, 2.5));
        assert!(close(d.gross_profit_usd, 4_000.0));
        assert!(close(d.contribution_after_ads_usd, 1_500.0));
        assert!(close(d.profit_per_ad_dollar, 0.6));
        assert!(d.profitable);
    }

    #[test]
    fn unprofitable_below_breakeven() {
        let d = generate(&RoasInput { ad_spend_usd: 5_000.0, ..base() });
        assert!(close(d.roas, 2.0));
        assert!(close(d.contribution_after_ads_usd, -1_000.0));
        assert!(!d.profitable);
    }

    #[test]
    fn break_even_is_inverse_margin() {
        let d = generate(&base());
        assert!(close(d.break_even_roas, 100.0 / 40.0));
    }

    #[test]
    fn at_breakeven_zero_contribution() {
        // ad spend = gross profit → contribution ≈ 0, roas == break-even.
        let d = generate(&RoasInput { ad_spend_usd: 4_000.0, ..base() });
        assert!(close(d.contribution_after_ads_usd, 0.0));
        assert!(!d.profitable);
    }

    #[test]
    fn invalid_inputs() {
        assert!(!generate(&RoasInput { ad_spend_usd: 0.0, ..base() }).valid);
        assert!(!generate(&RoasInput { gross_margin_pct: 0.0, ..base() }).valid);
    }
}
