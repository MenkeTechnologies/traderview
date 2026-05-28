//! Asness 12-1 (Jegadeesh-Titman) cross-sectional momentum scanner.
//!
//! For each symbol, compute the trailing 12-month total return EXCLUDING
//! the most recent month (the "skip-1" convention skips short-term
//! reversal noise). Then cross-sectionally rank into deciles and flag:
//!
//!   - **WinnerDecile**:   rank ≥ 9th decile (top 10%) — momentum long
//!   - **LoserDecile**:    rank ≤ 1st decile (bottom 10%) — momentum short
//!   - **Neutral**:        middle deciles
//!
//! Inputs are per-symbol monthly close price series (most recent last).
//! Must have at least 13 months of data per symbol; symbols with less
//! are excluded.
//!
//! Pure compute. Returns ranked candidates suitable for a long-short
//! momentum portfolio.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolMonthlyCloses {
    pub symbol: String,
    pub monthly_closes: Vec<f64>,    // most recent last
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MomentumBucket {
    WinnerDecile,
    LoserDecile,
    Neutral,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MomentumHit {
    pub symbol: String,
    pub trailing_12_1_return: f64,
    pub cross_sectional_percentile: f64,
    pub bucket: MomentumBucket,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MomentumReport {
    pub winners: Vec<MomentumHit>,
    pub losers: Vec<MomentumHit>,
    pub all_ranked: Vec<MomentumHit>,
}

pub fn scan(symbols: &[SymbolMonthlyCloses]) -> Option<MomentumReport> {
    if symbols.is_empty() { return None; }
    let mut returns: Vec<(String, f64)> = Vec::new();
    for s in symbols {
        let n = s.monthly_closes.len();
        if n < 13 { continue; }
        // Use the close 12 months ago vs the close from 1 month ago.
        let start_idx = n - 13;    // 13 months back from current = 12 months back from last-month
        let end_idx = n - 2;       // skip the most recent month
        let start = s.monthly_closes[start_idx];
        let end = s.monthly_closes[end_idx];
        if !start.is_finite() || start <= 0.0
            || !end.is_finite() || end <= 0.0
        {
            continue;
        }
        let ret_12_1 = (end - start) / start;
        if !ret_12_1.is_finite() { continue; }
        returns.push((s.symbol.clone(), ret_12_1));
    }
    if returns.is_empty() { return None; }
    // Sort ascending by return, then assign percentile = rank / n.
    returns.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    let n = returns.len();
    let mut all_ranked: Vec<MomentumHit> = returns.iter().enumerate()
        .map(|(i, (sym, ret))| {
            let pct = (i + 1) as f64 / n as f64 * 100.0;
            let bucket = if pct >= 90.0 { MomentumBucket::WinnerDecile }
                else if pct <= 10.0 { MomentumBucket::LoserDecile }
                else { MomentumBucket::Neutral };
            MomentumHit {
                symbol: sym.clone(),
                trailing_12_1_return: *ret,
                cross_sectional_percentile: pct,
                bucket,
            }
        })
        .collect();
    // Reorder so winners come first, losers second, neutrals last.
    all_ranked.sort_by(|a, b| b.trailing_12_1_return.partial_cmp(&a.trailing_12_1_return)
        .unwrap_or(std::cmp::Ordering::Equal));
    let winners: Vec<MomentumHit> = all_ranked.iter()
        .filter(|h| h.bucket == MomentumBucket::WinnerDecile)
        .cloned().collect();
    let losers: Vec<MomentumHit> = all_ranked.iter()
        .filter(|h| h.bucket == MomentumBucket::LoserDecile)
        .cloned().collect();
    Some(MomentumReport { winners, losers, all_ranked })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(sym: &str, closes: Vec<f64>) -> SymbolMonthlyCloses {
        SymbolMonthlyCloses { symbol: sym.into(), monthly_closes: closes }
    }

    #[test]
    fn empty_returns_none() {
        assert!(scan(&[]).is_none());
    }

    #[test]
    fn fewer_than_13_months_excluded() {
        let r = scan(&[s("SHORT", vec![100.0; 10])]);
        // Only insufficient symbols → no usable returns.
        assert!(r.is_none());
    }

    #[test]
    fn invalid_closes_skipped() {
        let r = scan(&[s("BAD", vec![0.0; 13])]);
        // Zero start price → ret undefined → skipped.
        assert!(r.is_none());
    }

    #[test]
    fn winner_decile_identifies_top_returners() {
        // 10 symbols with returns 1%–10%; top one should be WinnerDecile.
        let mut syms = Vec::new();
        for i in 1..=10 {
            let mut closes = vec![100.0; 13];
            // 13 months back = closes[0]. 1 month back = closes[11]. Ret = (closes[11] / 100 − 1).
            closes[11] = 100.0 * (1.0 + i as f64 * 0.02);
            syms.push(s(&format!("S{i}"), closes));
        }
        let r = scan(&syms).unwrap();
        // With n=10, top 10% = top 1 symbol.
        assert!(!r.winners.is_empty());
        assert_eq!(r.winners[0].symbol, "S10");
    }

    #[test]
    fn loser_decile_identifies_bottom_returners() {
        let mut syms = Vec::new();
        for i in 1..=10 {
            let mut closes = vec![100.0; 13];
            closes[11] = 100.0 * (1.0 - i as f64 * 0.02);
            syms.push(s(&format!("DOG{i}"), closes));
        }
        let r = scan(&syms).unwrap();
        assert!(!r.losers.is_empty());
        // Worst is DOG10.
        assert_eq!(r.losers[0].symbol, "DOG10");
    }

    #[test]
    fn skip_one_excludes_recent_month_from_calculation() {
        // Closes: 100 for 11 months, then 200 in month 12 (most recent),
        // then 100 in current month. The 12-1 return should be (last-month
        // close / 12-months-back close) − 1 = (200 / 100) − 1 = 1.0,
        // NOT including the current-month 100.
        let closes = vec![
            100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0, 100.0,
            200.0,    // 1 month back
            100.0,    // current (skipped)
        ];
        let r = scan(&[s("SKIP", closes)]).unwrap();
        assert!((r.all_ranked[0].trailing_12_1_return - 1.0).abs() < 1e-9);
    }

    #[test]
    fn nan_close_skipped() {
        let mut closes = vec![100.0; 13];
        closes[0] = f64::NAN;
        let r = scan(&[s("X", closes)]);
        assert!(r.is_none());
    }

    #[test]
    fn ranked_output_sorted_by_return_descending() {
        let mut syms = Vec::new();
        for i in 1..=20 {
            let mut closes = vec![100.0; 13];
            closes[11] = 100.0 + (i as f64);
            syms.push(s(&format!("S{i:02}"), closes));
        }
        let r = scan(&syms).unwrap();
        for w in r.all_ranked.windows(2) {
            assert!(w[0].trailing_12_1_return >= w[1].trailing_12_1_return);
        }
    }

    #[test]
    fn percentiles_in_unit_range() {
        let mut syms = Vec::new();
        for i in 1..=10 {
            let mut closes = vec![100.0; 13];
            closes[11] = 100.0 + i as f64;
            syms.push(s(&format!("S{i}"), closes));
        }
        let r = scan(&syms).unwrap();
        for h in &r.all_ranked {
            assert!((0.0..=100.0).contains(&h.cross_sectional_percentile));
        }
    }
}
