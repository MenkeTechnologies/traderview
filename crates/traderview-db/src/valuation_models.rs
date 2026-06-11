//! Tier-2 valuation models: reverse DCF, dividend discount (Gordon +
//! two-stage), Greenwald earnings-power value, Rule of 40, and the
//! options-wheel cycle calculator. All pure compute — the route layer
//! feeds user inputs (or metric_all-derived defaults) straight in.

use serde::{Deserialize, Serialize};

// ===========================================================================
// Reverse DCF — what growth does today's price imply?
// ===========================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct ReverseDcfInput {
    pub fcf_usd: f64,
    pub growth_years: u32,
    pub terminal_growth_pct: f64,
    pub discount_rate_pct: f64,
    pub net_debt_usd: f64,
    pub shares_outstanding: f64,
    pub current_price: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReverseDcfReport {
    /// Stage-1 growth (%/yr) that makes intrinsic == price.
    pub implied_growth_pct: f64,
    /// Intrinsic value at the implied growth (≈ current price; sanity).
    pub intrinsic_at_implied: f64,
    pub iterations: u32,
}

/// Bisection over the stage-1 growth rate in [-50%, +100%]. Returns
/// None when the price isn't reachable inside the bracket (e.g. price
/// below the zero-growth value AND negative growth still overshoots —
/// pathological inputs).
pub fn reverse_dcf(input: &ReverseDcfInput) -> Option<ReverseDcfReport> {
    if input.current_price <= 0.0 || input.shares_outstanding <= 0.0 {
        return None;
    }
    let intrinsic_at = |growth_pct: f64| -> Option<f64> {
        let r = crate::dcf_valuation::compute(&crate::dcf_valuation::DcfInput {
            fcf_usd: input.fcf_usd,
            growth_pct,
            growth_years: input.growth_years,
            terminal_growth_pct: input.terminal_growth_pct,
            discount_rate_pct: input.discount_rate_pct,
            net_debt_usd: input.net_debt_usd,
            shares_outstanding: input.shares_outstanding,
            current_price: None,
        })
        .ok()?;
        Some(r.intrinsic_per_share)
    };
    let (mut lo, mut hi) = (-50.0_f64, 100.0_f64);
    // Intrinsic is monotonic in growth, so check bracket endpoints.
    let v_lo = intrinsic_at(lo)?;
    let v_hi = intrinsic_at(hi)?;
    if !(v_lo <= input.current_price && input.current_price <= v_hi) {
        return None;
    }
    let mut iterations = 0;
    for _ in 0..200 {
        iterations += 1;
        let mid = (lo + hi) / 2.0;
        let v = intrinsic_at(mid)?;
        if (v - input.current_price).abs() < 0.01 {
            return Some(ReverseDcfReport {
                implied_growth_pct: mid,
                intrinsic_at_implied: v,
                iterations,
            });
        }
        if v < input.current_price {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let mid = (lo + hi) / 2.0;
    Some(ReverseDcfReport {
        implied_growth_pct: mid,
        intrinsic_at_implied: intrinsic_at(mid)?,
        iterations,
    })
}

// ===========================================================================
// Dividend Discount Model — Gordon + two-stage
// ===========================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct DdmInput {
    /// Forward annual dividend per share (D1).
    pub annual_dividend: f64,
    /// Stage-1 dividend growth %/yr (two-stage); ignored when
    /// `growth_years == 0` (pure Gordon).
    pub growth_pct: f64,
    pub growth_years: u32,
    /// Terminal growth %/yr. Must be < required return.
    pub terminal_growth_pct: f64,
    /// Required rate of return %/yr.
    pub required_return_pct: f64,
    pub current_price: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DdmReport {
    pub fair_value: f64,
    pub upside_pct: Option<f64>,
    pub model: &'static str, // "gordon" | "two_stage"
}

pub fn ddm(input: &DdmInput) -> Option<DdmReport> {
    let r = input.required_return_pct / 100.0;
    let g = input.growth_pct / 100.0;
    let gt = input.terminal_growth_pct / 100.0;
    if r <= gt || input.annual_dividend <= 0.0 || !r.is_finite() {
        return None;
    }
    let (fair, model) = if input.growth_years == 0 {
        // Pure Gordon: P = D1 / (r − g_term).
        (input.annual_dividend / (r - gt), "gordon")
    } else {
        // Two-stage: PV of growing dividends + discounted Gordon tail.
        let mut pv = 0.0;
        let mut d = input.annual_dividend;
        for t in 1..=input.growth_years {
            if t > 1 {
                d *= 1.0 + g;
            }
            pv += d / (1.0 + r).powi(t as i32);
        }
        let d_next = d * (1.0 + gt);
        let tail = d_next / (r - gt) / (1.0 + r).powi(input.growth_years as i32);
        (pv + tail, "two_stage")
    };
    let upside_pct = input
        .current_price
        .filter(|p| *p > 0.0)
        .map(|p| (fair - p) / p * 100.0);
    Some(DdmReport {
        fair_value: fair,
        upside_pct,
        model,
    })
}

// ===========================================================================
// Earnings Power Value (Greenwald)
// ===========================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct EpvInput {
    /// Normalized (cycle-average) operating earnings, $.
    pub normalized_ebit_usd: f64,
    /// Effective tax rate % (e.g. 21).
    pub tax_rate_pct: f64,
    /// WACC %.
    pub wacc_pct: f64,
    /// Excess cash − debt adjustment, $ (positive = net cash added).
    pub net_cash_adjustment_usd: f64,
    pub shares_outstanding: f64,
    pub current_price: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct EpvReport {
    pub nopat_usd: f64,
    pub epv_operations_usd: f64,
    pub epv_equity_usd: f64,
    pub epv_per_share: f64,
    pub upside_pct: Option<f64>,
}

pub fn epv(input: &EpvInput) -> Option<EpvReport> {
    let wacc = input.wacc_pct / 100.0;
    if wacc <= 0.0 || input.shares_outstanding <= 0.0 {
        return None;
    }
    let nopat = input.normalized_ebit_usd * (1.0 - input.tax_rate_pct / 100.0);
    let epv_ops = nopat / wacc;
    let equity = epv_ops + input.net_cash_adjustment_usd;
    let per_share = equity / input.shares_outstanding;
    let upside_pct = input
        .current_price
        .filter(|p| *p > 0.0)
        .map(|p| (per_share - p) / p * 100.0);
    Some(EpvReport {
        nopat_usd: nopat,
        epv_operations_usd: epv_ops,
        epv_equity_usd: equity,
        epv_per_share: per_share,
        upside_pct,
    })
}

// ===========================================================================
// Rule of 40 (growth + profitability for growth companies)
// ===========================================================================

#[derive(Debug, Clone, Serialize)]
pub struct RuleOf40Report {
    pub revenue_growth_pct: f64,
    pub fcf_margin_pct: f64,
    pub score: f64,
    pub passes: bool,
}

pub fn rule_of_40(revenue_growth_pct: f64, fcf_margin_pct: f64) -> RuleOf40Report {
    let score = revenue_growth_pct + fcf_margin_pct;
    RuleOf40Report {
        revenue_growth_pct,
        fcf_margin_pct,
        score,
        passes: score >= 40.0,
    }
}

// ===========================================================================
// Options wheel cycle calculator
// ===========================================================================

#[derive(Debug, Clone, Deserialize)]
pub struct WheelInput {
    /// Underlying price now.
    pub stock_price: f64,
    /// Cash-secured put: strike + premium received + days to expiry.
    pub put_strike: f64,
    pub put_premium: f64,
    pub put_dte: u32,
    /// Covered call sold after assignment: strike + premium + DTE.
    pub call_strike: f64,
    pub call_premium: f64,
    pub call_dte: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct WheelReport {
    /// Return if the put expires worthless: premium / collateral, annualized.
    pub put_side_return_pct: f64,
    pub put_side_annualized_pct: f64,
    /// Full-cycle return if assigned then called away:
    /// (put_prem + call_prem + (call_strike − put_strike)) / put_strike.
    pub full_cycle_return_pct: f64,
    pub full_cycle_annualized_pct: f64,
    pub full_cycle_days: u32,
    /// Effective cost basis if assigned: put_strike − put_premium.
    pub assigned_cost_basis: f64,
    /// Downside cushion: how far the stock can fall before the
    /// assigned basis is underwater, as % of current price.
    pub breakeven_drop_pct: f64,
}

pub fn wheel(input: &WheelInput) -> Option<WheelReport> {
    if input.put_strike <= 0.0
        || input.stock_price <= 0.0
        || input.put_dte == 0
        || input.call_dte == 0
    {
        return None;
    }
    let collateral = input.put_strike;
    let put_ret = input.put_premium / collateral * 100.0;
    let put_ann = put_ret * 365.0 / input.put_dte as f64;
    let cycle_days = input.put_dte + input.call_dte;
    let cycle_gain =
        input.put_premium + input.call_premium + (input.call_strike - input.put_strike).max(0.0);
    let cycle_ret = cycle_gain / collateral * 100.0;
    let cycle_ann = cycle_ret * 365.0 / cycle_days as f64;
    let basis = input.put_strike - input.put_premium;
    let breakeven_drop = (input.stock_price - basis) / input.stock_price * 100.0;
    Some(WheelReport {
        put_side_return_pct: put_ret,
        put_side_annualized_pct: put_ann,
        full_cycle_return_pct: cycle_ret,
        full_cycle_annualized_pct: cycle_ann,
        full_cycle_days: cycle_days,
        assigned_cost_basis: basis,
        breakeven_drop_pct: breakeven_drop,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reverse_dcf_recovers_forward_growth() {
        // Forward: FCF=100, g=8%, 5y, gt=2.5%, r=10%, shares=10.
        let fwd = crate::dcf_valuation::compute(&crate::dcf_valuation::DcfInput {
            fcf_usd: 100.0,
            growth_pct: 8.0,
            growth_years: 5,
            terminal_growth_pct: 2.5,
            discount_rate_pct: 10.0,
            net_debt_usd: 0.0,
            shares_outstanding: 10.0,
            current_price: None,
        })
        .unwrap();
        // Reverse: feed the intrinsic as the price; should recover ≈8%.
        let rev = reverse_dcf(&ReverseDcfInput {
            fcf_usd: 100.0,
            growth_years: 5,
            terminal_growth_pct: 2.5,
            discount_rate_pct: 10.0,
            net_debt_usd: 0.0,
            shares_outstanding: 10.0,
            current_price: fwd.intrinsic_per_share,
        })
        .unwrap();
        assert!(
            (rev.implied_growth_pct - 8.0).abs() < 0.1,
            "{}",
            rev.implied_growth_pct
        );
    }

    #[test]
    fn gordon_ddm_identity() {
        // D1=4, r=8%, g=3% → P = 4 / 0.05 = 80.
        let r = ddm(&DdmInput {
            annual_dividend: 4.0,
            growth_pct: 0.0,
            growth_years: 0,
            terminal_growth_pct: 3.0,
            required_return_pct: 8.0,
            current_price: Some(60.0),
        })
        .unwrap();
        assert!((r.fair_value - 80.0).abs() < 0.01);
        assert_eq!(r.model, "gordon");
        assert!(r.upside_pct.unwrap() > 30.0);
    }

    #[test]
    fn two_stage_ddm_exceeds_gordon_when_stage1_growth_higher() {
        let gordon = ddm(&DdmInput {
            annual_dividend: 4.0,
            growth_pct: 0.0,
            growth_years: 0,
            terminal_growth_pct: 3.0,
            required_return_pct: 8.0,
            current_price: None,
        })
        .unwrap();
        let two = ddm(&DdmInput {
            annual_dividend: 4.0,
            growth_pct: 10.0, // stage-1 growth > terminal
            growth_years: 5,
            terminal_growth_pct: 3.0,
            required_return_pct: 8.0,
            current_price: None,
        })
        .unwrap();
        assert!(two.fair_value > gordon.fair_value);
    }

    #[test]
    fn ddm_rejects_rate_inversion() {
        assert!(ddm(&DdmInput {
            annual_dividend: 4.0,
            growth_pct: 0.0,
            growth_years: 0,
            terminal_growth_pct: 9.0,
            required_return_pct: 8.0,
            current_price: None,
        })
        .is_none());
    }

    #[test]
    fn epv_math() {
        // EBIT=100, tax 21% → NOPAT 79; WACC 10% → ops 790; +10 cash,
        // 10 shares → $80/share.
        let r = epv(&EpvInput {
            normalized_ebit_usd: 100.0,
            tax_rate_pct: 21.0,
            wacc_pct: 10.0,
            net_cash_adjustment_usd: 10.0,
            shares_outstanding: 10.0,
            current_price: Some(40.0),
        })
        .unwrap();
        assert!((r.epv_per_share - 80.0).abs() < 0.01);
        assert!(r.upside_pct.unwrap() > 99.0);
    }

    #[test]
    fn rule_of_40_threshold() {
        assert!(rule_of_40(30.0, 15.0).passes);
        assert!(!rule_of_40(25.0, 10.0).passes);
        assert!((rule_of_40(25.0, 10.0).score - 35.0).abs() < 1e-9);
    }

    #[test]
    fn wheel_cycle_math() {
        // Stock $100; CSP 95 strike, $2 prem, 30 DTE; CC 100 strike,
        // $2.50 prem, 30 DTE.
        let r = wheel(&WheelInput {
            stock_price: 100.0,
            put_strike: 95.0,
            put_premium: 2.0,
            put_dte: 30,
            call_strike: 100.0,
            call_premium: 2.5,
            call_dte: 30,
        })
        .unwrap();
        // Put side: 2/95 ≈ 2.105% per 30d → ~25.6% annualized.
        assert!((r.put_side_return_pct - 2.105).abs() < 0.01);
        assert!((r.put_side_annualized_pct - 25.6).abs() < 0.3);
        // Full cycle: (2 + 2.5 + 5)/95 = 10% per 60d → ~60.8% annualized.
        assert!((r.full_cycle_return_pct - 10.0).abs() < 0.01);
        assert_eq!(r.full_cycle_days, 60);
        // Basis 93; cushion (100−93)/100 = 7%.
        assert!((r.assigned_cost_basis - 93.0).abs() < 1e-9);
        assert!((r.breakeven_drop_pct - 7.0).abs() < 0.01);
    }
}
