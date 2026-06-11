//! Stock-based-compensation dilution — the buyback that isn't.
//!
//!   sbc_yield      = annual SBC / market cap
//!   gross buyback  = buybacks / market cap
//!   net buyback    = gross − sbc_yield
//!
//! Tech screens love "shareholder yield" built on gross buybacks while
//! SBC quietly re-issues the shares out the back door. The report nets
//! the two and shows how much of the buyback is just SBC mop-up.
//!
//! Pure compute. Companion to `deep_value` (gross shareholder yield),
//! `buyback_accretion`.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct SbcInput {
    pub market_cap: f64,
    /// Annual stock-based compensation expense, $.
    pub annual_sbc: f64,
    /// Annual gross buybacks, $ (0 = none).
    #[serde(default)]
    pub annual_buybacks: f64,
    /// Annual revenue, $ — optional SBC-intensity row (0 = skip).
    #[serde(default)]
    pub annual_revenue: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SbcReport {
    /// SBC as % of market cap — the implicit annual dilution.
    pub sbc_dilution_pct: f64,
    pub gross_buyback_yield_pct: f64,
    /// Buyback yield net of SBC — the real return of capital.
    pub net_buyback_yield_pct: f64,
    /// Share of the buyback that merely offsets SBC (capped at 100).
    pub buyback_consumed_by_sbc_pct: Option<f64>,
    /// SBC / revenue, % — the intensity screeners use.
    pub sbc_to_revenue_pct: Option<f64>,
}

pub fn compute(inp: &SbcInput) -> Option<SbcReport> {
    if !inp.market_cap.is_finite()
        || inp.market_cap <= 0.0
        || !inp.annual_sbc.is_finite()
        || inp.annual_sbc < 0.0
        || !inp.annual_buybacks.is_finite()
        || inp.annual_buybacks < 0.0
        || !inp.annual_revenue.is_finite()
        || inp.annual_revenue < 0.0
    {
        return None;
    }
    let sbc = inp.annual_sbc / inp.market_cap * 100.0;
    let gross = inp.annual_buybacks / inp.market_cap * 100.0;
    Some(SbcReport {
        sbc_dilution_pct: sbc,
        gross_buyback_yield_pct: gross,
        net_buyback_yield_pct: gross - sbc,
        buyback_consumed_by_sbc_pct: (inp.annual_buybacks > 0.0)
            .then(|| (inp.annual_sbc / inp.annual_buybacks * 100.0).min(100.0)),
        sbc_to_revenue_pct: (inp.annual_revenue > 0.0)
            .then(|| inp.annual_sbc / inp.annual_revenue * 100.0),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nets_the_buyback_against_sbc() {
        // $100B cap, $3B SBC, $5B buybacks, $40B revenue:
        // 3% dilution, 5% gross, 2% net, 60% of the buyback is mop-up,
        // SBC 7.5% of revenue.
        let r = compute(&SbcInput {
            market_cap: 100_000.0,
            annual_sbc: 3_000.0,
            annual_buybacks: 5_000.0,
            annual_revenue: 40_000.0,
        })
        .unwrap();
        assert!((r.sbc_dilution_pct - 3.0).abs() < 1e-12);
        assert!((r.gross_buyback_yield_pct - 5.0).abs() < 1e-12);
        assert!((r.net_buyback_yield_pct - 2.0).abs() < 1e-12);
        assert!((r.buyback_consumed_by_sbc_pct.unwrap() - 60.0).abs() < 1e-12);
        assert!((r.sbc_to_revenue_pct.unwrap() - 7.5).abs() < 1e-12);
    }

    #[test]
    fn sbc_exceeding_buybacks_goes_net_negative() {
        let r = compute(&SbcInput {
            market_cap: 100_000.0,
            annual_sbc: 6_000.0,
            annual_buybacks: 5_000.0,
            annual_revenue: 0.0,
        })
        .unwrap();
        assert!(r.net_buyback_yield_pct < 0.0);
        // Mop-up share capped at 100%.
        assert_eq!(r.buyback_consumed_by_sbc_pct, Some(100.0));
        assert!(r.sbc_to_revenue_pct.is_none());
    }

    #[test]
    fn no_buybacks_is_pure_dilution() {
        let r = compute(&SbcInput {
            market_cap: 50_000.0,
            annual_sbc: 1_000.0,
            annual_buybacks: 0.0,
            annual_revenue: 0.0,
        })
        .unwrap();
        assert!((r.net_buyback_yield_pct + 2.0).abs() < 1e-12);
        assert!(r.buyback_consumed_by_sbc_pct.is_none());
    }

    #[test]
    fn hostile_inputs_return_none() {
        assert!(compute(&SbcInput {
            market_cap: 0.0,
            annual_sbc: 1.0,
            annual_buybacks: 0.0,
            annual_revenue: 0.0,
        })
        .is_none());
        assert!(compute(&SbcInput {
            market_cap: 100.0,
            annual_sbc: -1.0,
            annual_buybacks: 0.0,
            annual_revenue: 0.0,
        })
        .is_none());
        assert!(compute(&SbcInput {
            market_cap: 100.0,
            annual_sbc: f64::NAN,
            annual_buybacks: 0.0,
            annual_revenue: 0.0,
        })
        .is_none());
    }
}
