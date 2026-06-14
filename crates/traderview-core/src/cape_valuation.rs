//! CAPE valuation & CAPE-adjusted safe withdrawal rate — uses the cyclically-
//! adjusted price/earnings ratio (Shiller CAPE) to gauge market valuation and to
//! temper a retirement withdrawal rate. The CAPE earnings yield (1 ÷ CAPE) is a
//! mean-reversion proxy for the expected real return; comparing CAPE to its
//! historical mean flags over- or under-valuation; and a CAPE-adjusted safe
//! withdrawal rate lowers the starting draw when valuations are stretched
//! (heuristic: SWR ≈ 1.0% + 0.5 × CAPE earnings yield, after Pfau-style work).
//! Pure compute. Not financial advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct CapeInput {
    /// Current CAPE (Shiller P/E10).
    pub cape: f64,
    /// Historical mean CAPE for the valuation comparison (long-run ≈ 16.4).
    #[serde(default = "default_mean")]
    pub historical_mean_cape: f64,
}

fn default_mean() -> f64 {
    16.4
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct CapeReport {
    /// 1 ÷ CAPE, percent — the CAPE earnings yield / expected real return proxy.
    pub cape_earnings_yield_pct: f64,
    /// CAPE ÷ historical mean (>1 = richer than average).
    pub valuation_ratio: f64,
    /// "undervalued", "fair", or "overvalued" vs the historical mean.
    pub valuation: String,
    /// Heuristic CAPE-adjusted safe withdrawal rate, percent.
    pub cape_adjusted_swr_pct: f64,
    pub valid: bool,
}

fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

pub fn generate(i: &CapeInput) -> CapeReport {
    if i.cape <= 0.0 {
        return CapeReport::default();
    }
    let caey = 1.0 / i.cape * 100.0;
    let ratio = if i.historical_mean_cape > 0.0 {
        i.cape / i.historical_mean_cape
    } else {
        1.0
    };
    let valuation = if ratio > 1.15 {
        "overvalued"
    } else if ratio < 0.85 {
        "undervalued"
    } else {
        "fair"
    };
    let swr = 1.0 + 0.5 * caey;
    CapeReport {
        cape_earnings_yield_pct: round3(caey),
        valuation_ratio: round3(ratio),
        valuation: valuation.to_string(),
        cape_adjusted_swr_pct: round3(swr),
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    #[test]
    fn rich_market_low_swr() {
        let d = generate(&CapeInput { cape: 30.0, historical_mean_cape: 16.4 });
        assert!(d.valid);
        assert!(close(d.cape_earnings_yield_pct, 3.333));
        assert!(close(d.cape_adjusted_swr_pct, 2.667));
        assert!(close(d.valuation_ratio, 1.829));
        assert_eq!(d.valuation, "overvalued");
    }

    #[test]
    fn cheap_market_higher_swr() {
        // CAPE 14 sits just inside the "fair" band (14/16.4 = 0.854 > 0.85) but
        // still gives a higher withdrawal rate than a rich market.
        let mid = generate(&CapeInput { cape: 14.0, historical_mean_cape: 16.4 });
        assert!(close(mid.cape_adjusted_swr_pct, 4.571));
        // A clearly cheap market (CAPE 12 → 0.73× mean) reads as undervalued.
        let cheap = generate(&CapeInput { cape: 12.0, historical_mean_cape: 16.4 });
        assert!(close(cheap.cape_adjusted_swr_pct, 5.167));
        assert_eq!(cheap.valuation, "undervalued");
    }

    #[test]
    fn near_mean_is_fair() {
        let d = generate(&CapeInput { cape: 17.0, historical_mean_cape: 16.4 });
        assert_eq!(d.valuation, "fair");
    }

    #[test]
    fn earnings_yield_inverse_of_cape() {
        let d = generate(&CapeInput { cape: 25.0, historical_mean_cape: 16.4 });
        assert!(close(d.cape_earnings_yield_pct, 100.0 / 25.0));
    }

    #[test]
    fn invalid_cape() {
        assert!(!generate(&CapeInput { cape: 0.0, historical_mean_cape: 16.4 }).valid);
    }
}
