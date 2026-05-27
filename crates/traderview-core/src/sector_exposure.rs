//! Sector-exposure aggregator.
//!
//! Groups holdings by GICS sector tag and emits per-sector gross/net
//! exposure + share-of-portfolio. Surfaces sector concentration that
//! correlation_clusters can't catch (e.g. AAPL + EQR — no return
//! correlation but both heavily tied to US equities sentiment).
//!
//! Pure compute. Caller provides the (symbol → sector) mapping. Engine
//! does not embed a sector lookup table — that's reference data the
//! caller refreshes on its own cadence.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionWithSector {
    pub symbol: String,
    /// Signed dollar notional. Positive = long, negative = short.
    pub notional: f64,
    /// GICS sector or other taxonomy. Unknown → caller passes "Unknown".
    pub sector: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorStats {
    pub sector: String,
    pub long_notional: f64,
    pub short_notional: f64,
    pub gross_notional: f64,
    pub net_notional: f64,
    /// Share of total gross exposure.
    pub pct_of_gross: f64,
    pub position_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SectorReport {
    pub total_gross: f64,
    pub total_net: f64,
    pub by_sector: Vec<SectorStats>,
    /// Sectors with > 33% of gross — concentration flag.
    pub overweight_sectors: Vec<String>,
}

pub fn analyze(positions: &[PositionWithSector]) -> SectorReport {
    let mut report = SectorReport::default();
    if positions.is_empty() {
        return report;
    }
    let mut by_sector: BTreeMap<String, (f64, f64, usize)> = BTreeMap::new();
    let mut total_gross = 0.0;
    let mut total_net = 0.0;
    for p in positions {
        let entry = by_sector.entry(p.sector.clone()).or_insert((0.0, 0.0, 0));
        if p.notional >= 0.0 {
            entry.0 += p.notional;
        } else {
            entry.1 += -p.notional;
        } // store as positive magnitude
        entry.2 += 1;
        total_gross += p.notional.abs();
        total_net += p.notional;
    }
    report.total_gross = total_gross;
    report.total_net = total_net;
    for (sector, (long, short, count)) in by_sector {
        let gross = long + short;
        let net = long - short;
        let pct = if total_gross > 0.0 {
            gross / total_gross
        } else {
            0.0
        };
        if pct > 0.33 {
            report.overweight_sectors.push(sector.clone());
        }
        report.by_sector.push(SectorStats {
            sector,
            long_notional: long,
            short_notional: short,
            gross_notional: gross,
            net_notional: net,
            pct_of_gross: pct,
            position_count: count,
        });
    }
    // Largest sector first.
    report.by_sector.sort_by(|a, b| {
        b.gross_notional
            .partial_cmp(&a.gross_notional)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    report.overweight_sectors.sort();
    report
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(sym: &str, n: f64, sector: &str) -> PositionWithSector {
        PositionWithSector {
            symbol: sym.into(),
            notional: n,
            sector: sector.into(),
        }
    }

    #[test]
    fn empty_returns_default() {
        let r = analyze(&[]);
        assert_eq!(r.total_gross, 0.0);
        assert!(r.by_sector.is_empty());
    }

    #[test]
    fn single_position_sector_full_weight() {
        let r = analyze(&[p("AAPL", 10_000.0, "Tech")]);
        assert_eq!(r.by_sector.len(), 1);
        assert_eq!(r.by_sector[0].pct_of_gross, 1.0);
        assert!(r.overweight_sectors.contains(&"Tech".into()));
    }

    #[test]
    fn long_and_short_in_same_sector_track_separately() {
        let positions = vec![p("AAPL", 10_000.0, "Tech"), p("META", -4_000.0, "Tech")];
        let r = analyze(&positions);
        let tech = &r.by_sector[0];
        assert_eq!(tech.long_notional, 10_000.0);
        assert_eq!(tech.short_notional, 4_000.0);
        assert_eq!(tech.gross_notional, 14_000.0);
        assert_eq!(tech.net_notional, 6_000.0);
    }

    #[test]
    fn overweight_flag_quiet_when_no_sector_above_33_percent() {
        // 30/30/40 split — only Finance (40%) crosses the 33% bar.
        let positions = vec![
            p("AAPL", 3_000.0, "Tech"),
            p("XOM", 3_000.0, "Energy"),
            p("JPM", 4_000.0, "Finance"),
        ];
        let r = analyze(&positions);
        assert_eq!(r.overweight_sectors, vec!["Finance"]);
    }

    #[test]
    fn overweight_flag_triggers_when_one_sector_dominates() {
        // 70% Tech, 30% Energy.
        let positions = vec![p("AAPL", 7_000.0, "Tech"), p("XOM", 3_000.0, "Energy")];
        let r = analyze(&positions);
        assert_eq!(r.overweight_sectors, vec!["Tech"]);
    }

    #[test]
    fn sectors_sorted_by_gross_descending() {
        let positions = vec![
            p("SMALL", 1_000.0, "Small"),
            p("BIG", 10_000.0, "Big"),
            p("MID", 5_000.0, "Mid"),
        ];
        let r = analyze(&positions);
        assert_eq!(r.by_sector[0].sector, "Big");
        assert_eq!(r.by_sector[1].sector, "Mid");
        assert_eq!(r.by_sector[2].sector, "Small");
    }

    #[test]
    fn position_count_per_sector_tracked() {
        let positions = vec![
            p("AAPL", 1.0, "Tech"),
            p("MSFT", 1.0, "Tech"),
            p("GOOGL", 1.0, "Tech"),
            p("XOM", 1.0, "Energy"),
        ];
        let r = analyze(&positions);
        let tech = r.by_sector.iter().find(|s| s.sector == "Tech").unwrap();
        let energy = r.by_sector.iter().find(|s| s.sector == "Energy").unwrap();
        assert_eq!(tech.position_count, 3);
        assert_eq!(energy.position_count, 1);
    }

    #[test]
    fn total_net_signed_sum() {
        // Net should reflect long minus short across all positions.
        let positions = vec![p("LONG", 10_000.0, "X"), p("SHORT", -10_000.0, "Y")];
        let r = analyze(&positions);
        assert_eq!(r.total_net, 0.0);
        assert_eq!(r.total_gross, 20_000.0);
    }
}
