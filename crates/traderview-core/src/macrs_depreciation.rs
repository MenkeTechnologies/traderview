//! MACRS depreciation — the U.S. Modified Accelerated Cost Recovery System tax
//! depreciation schedule (GDS, half-year convention, IRS Pub. 946). Given the
//! depreciable basis and a recovery period (3, 5, 7, 10, 15, or 20 years), it
//! applies the published per-year percentage table to produce the deduction,
//! accumulated depreciation, and remaining basis for each year. Distinct from the
//! book depreciation schedule (straight-line / declining-balance) and the §1250
//! recapture module — this is the tax schedule. Not tax advice.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct MacrsInput {
    pub asset_label: String,
    /// Depreciable basis (cost).
    pub basis_usd: f64,
    /// Recovery period in years: 3, 5, 7, 10, 15, or 20.
    pub recovery_years: u32,
    #[serde(default)]
    pub placed_in_service_year: i32,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MacrsRow {
    pub year: u32,
    pub rate_pct: f64,
    pub depreciation_usd: f64,
    pub accumulated_usd: f64,
    pub remaining_basis_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct MacrsReport {
    pub recovery_years: u32,
    pub total_depreciation_usd: f64,
    pub schedule: Vec<MacrsRow>,
    pub valid: bool,
}

/// GDS half-year-convention percentages by recovery period (IRS Pub. 946).
fn table(years: u32) -> Option<&'static [f64]> {
    match years {
        3 => Some(&[33.33, 44.45, 14.81, 7.41]),
        5 => Some(&[20.00, 32.00, 19.20, 11.52, 11.52, 5.76]),
        7 => Some(&[14.29, 24.49, 17.49, 12.49, 8.93, 8.92, 8.93, 4.46]),
        10 => Some(&[10.00, 18.00, 14.40, 11.52, 9.22, 7.37, 6.55, 6.55, 6.56, 6.55, 3.28]),
        15 => Some(&[
            5.00, 9.50, 8.55, 7.70, 6.93, 6.23, 5.90, 5.90, 5.91, 5.90, 5.91, 5.90, 5.91, 5.90,
            5.91, 2.95,
        ]),
        20 => Some(&[
            3.750, 7.219, 6.677, 6.177, 5.713, 5.285, 4.888, 4.522, 4.462, 4.461, 4.462, 4.461,
            4.462, 4.461, 4.462, 4.461, 4.462, 4.461, 4.462, 4.461, 2.231,
        ]),
        _ => None,
    }
}

fn cents(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}

pub fn generate(i: &MacrsInput) -> MacrsReport {
    let rates = match table(i.recovery_years) {
        Some(r) if i.basis_usd > 0.0 => r,
        _ => return MacrsReport { recovery_years: i.recovery_years, ..Default::default() },
    };
    let mut accumulated = 0.0;
    let schedule: Vec<MacrsRow> = rates
        .iter()
        .enumerate()
        .map(|(k, &rate)| {
            let dep = cents(i.basis_usd * rate / 100.0);
            accumulated = cents(accumulated + dep);
            MacrsRow {
                year: (k + 1) as u32,
                rate_pct: rate,
                depreciation_usd: dep,
                accumulated_usd: accumulated,
                remaining_basis_usd: cents(i.basis_usd - accumulated),
            }
        })
        .collect();

    MacrsReport {
        recovery_years: i.recovery_years,
        total_depreciation_usd: cents(accumulated),
        schedule,
        valid: true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 0.01
    }

    fn base() -> MacrsInput {
        MacrsInput {
            asset_label: "Equipment".into(),
            basis_usd: 10_000.0,
            recovery_years: 5,
            placed_in_service_year: 2026,
        }
    }

    #[test]
    fn five_year_schedule() {
        let d = generate(&base());
        assert!(d.valid);
        assert_eq!(d.schedule.len(), 6);
        assert!(close(d.schedule[0].depreciation_usd, 2_000.0));
        assert!(close(d.schedule[1].depreciation_usd, 3_200.0));
        assert!(close(d.schedule[5].depreciation_usd, 576.0));
        assert!(close(d.total_depreciation_usd, 10_000.0));
    }

    #[test]
    fn accumulates_to_basis() {
        for yrs in [3, 5, 7, 10, 15, 20] {
            let d = generate(&MacrsInput { recovery_years: yrs, ..base() });
            assert!(close(d.total_depreciation_usd, 10_000.0), "yrs {yrs}");
            assert!(close(d.schedule.last().unwrap().remaining_basis_usd, 0.0));
        }
    }

    #[test]
    fn remaining_basis_declines() {
        let d = generate(&base());
        for w in d.schedule.windows(2) {
            assert!(w[1].remaining_basis_usd <= w[0].remaining_basis_usd);
        }
    }

    #[test]
    fn unsupported_period_invalid() {
        let d = generate(&MacrsInput { recovery_years: 4, ..base() });
        assert!(!d.valid);
    }

    #[test]
    fn zero_basis_invalid() {
        let d = generate(&MacrsInput { basis_usd: 0.0, ..base() });
        assert!(!d.valid);
    }
}
