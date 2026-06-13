//! QBI deduction — the IRC § 199A qualified business income deduction for
//! pass-through owners (sole props, partnerships, S-corps).
//!
//! The deduction is 20% of qualified business income, but above a taxable-income
//! threshold it is capped by a W-2-wage / UBIA limit, and a specified service
//! trade or business (SSTB — law, health, consulting, etc.) is phased out
//! entirely. Between the threshold and the threshold + phase-in range, both the
//! wage limit and the SSTB cut apply on a sliding scale. The whole thing is then
//! capped at 20% of (taxable income − net capital gain).
//!
//! 2026 defaults (web-verified, OBBBA-updated, overridable for other years):
//!   - rate 20%
//!   - threshold $201,750 single / $403,500 married-joint
//!   - phase-in range $75,000 single / $150,000 married-joint
//!   - minimum deduction $400 when QBI ≥ $1,000 (new for 2026)
//!
//! Distinct from `section-1402` (the SE tax this deduction does *not* touch) and
//! the income-tax brackets (`capital-gains-tax`) it sits on top of.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FilingStatus {
    Single,
    MarriedJoint,
}

fn d_rate() -> f64 {
    20.0
}
fn d_min_deduction() -> f64 {
    400.0
}
fn d_min_qbi_floor() -> f64 {
    1_000.0
}

#[derive(Debug, Clone, Deserialize)]
pub struct QbiInput {
    /// Qualified business income (net of the deductible half of SE tax, etc.).
    pub qbi_usd: f64,
    /// Taxable income before the QBI deduction.
    pub taxable_income_usd: f64,
    /// Net capital gain + qualified dividends (caps the deduction).
    #[serde(default)]
    pub net_capital_gain_usd: f64,
    pub filing_status: FilingStatus,
    /// W-2 wages paid by the business (for the wage limit).
    #[serde(default)]
    pub w2_wages_usd: f64,
    /// Unadjusted basis immediately after acquisition of qualified property.
    #[serde(default)]
    pub ubia_usd: f64,
    /// Specified service trade or business (phased out entirely above the range).
    #[serde(default)]
    pub is_sstb: bool,
    /// Deduction rate, percent.
    #[serde(default = "d_rate")]
    pub rate_pct: f64,
    /// Taxable-income threshold where limits begin. None → status default.
    #[serde(default)]
    pub threshold_usd: Option<f64>,
    /// Phase-in range above the threshold. None → status default.
    #[serde(default)]
    pub phase_in_usd: Option<f64>,
    /// Minimum deduction for active QBI (2026+).
    #[serde(default = "d_min_deduction")]
    pub min_deduction_usd: f64,
    /// QBI floor that unlocks the minimum deduction.
    #[serde(default = "d_min_qbi_floor")]
    pub min_qbi_floor_usd: f64,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct QbiResult {
    /// rate × QBI, before any limit.
    pub tentative_deduction_usd: f64,
    /// max(50% × W-2, 25% × W-2 + 2.5% × UBIA).
    pub wage_ubia_limit_usd: f64,
    /// 20% × (taxable income − net capital gain), the overall ceiling.
    pub overall_limit_usd: f64,
    /// QBI component after wage limit / SSTB / phase-in / minimum.
    pub qbi_component_usd: f64,
    /// Final allowed deduction.
    pub deduction_usd: f64,
    /// below_threshold | in_phase_in | fully_phased_in.
    pub phase_in_status: &'static str,
    /// Fraction (0..1) of the way through the phase-in range.
    pub phase_in_fraction: f64,
    /// True when any limit cut the deduction below rate × QBI.
    pub is_limited: bool,
}

pub fn analyze(input: &QbiInput) -> QbiResult {
    let rate = input.rate_pct / 100.0;
    let (def_threshold, def_phase_in) = match input.filing_status {
        FilingStatus::Single => (201_750.0, 75_000.0),
        FilingStatus::MarriedJoint => (403_500.0, 150_000.0),
    };
    let threshold = input.threshold_usd.unwrap_or(def_threshold);
    let phase_in = input.phase_in_usd.unwrap_or(def_phase_in);

    let tentative = rate * input.qbi_usd;
    let full_wage_limit =
        (0.50 * input.w2_wages_usd).max(0.25 * input.w2_wages_usd + 0.025 * input.ubia_usd);

    let over = input.taxable_income_usd - threshold;
    // qualified_qbi is the QBI that actually counts as qualified after the SSTB
    // treatment — it drives the 2026 minimum-deduction floor.
    let (status, fraction, component_before_cap, qualified_qbi): (&'static str, f64, f64, f64) =
        if over <= 0.0 {
            // Below the threshold: full 20%, no wage limit, SSTB fully allowed.
            ("below_threshold", 0.0, tentative, input.qbi_usd)
        } else if phase_in > 0.0 && over < phase_in {
            // Within the phase-in range: sliding application.
            let ratio = over / phase_in;
            let (q, w, u) = if input.is_sstb {
                let applicable = 1.0 - ratio;
                (
                    input.qbi_usd * applicable,
                    input.w2_wages_usd * applicable,
                    input.ubia_usd * applicable,
                )
            } else {
                (input.qbi_usd, input.w2_wages_usd, input.ubia_usd)
            };
            let tent = rate * q;
            let wage_limit = (0.50 * w).max(0.25 * w + 0.025 * u);
            let comp = if tent <= wage_limit {
                tent
            } else {
                tent - (tent - wage_limit) * ratio
            };
            ("in_phase_in", ratio, comp, q)
        } else {
            // Fully phased in: SSTB → 0 (income no longer qualified), else
            // capped at the wage/UBIA limit.
            if input.is_sstb {
                ("fully_phased_in", 1.0, 0.0, 0.0)
            } else {
                ("fully_phased_in", 1.0, tentative.min(full_wage_limit), input.qbi_usd)
            }
        };

    // 2026 minimum deduction for active *qualified* QBI above the floor.
    let mut component = component_before_cap;
    if qualified_qbi >= input.min_qbi_floor_usd {
        component = component.max(input.min_deduction_usd);
    }

    let overall_limit = (rate * (input.taxable_income_usd - input.net_capital_gain_usd)).max(0.0);
    let deduction = component.min(overall_limit).max(0.0);

    QbiResult {
        tentative_deduction_usd: tentative,
        wage_ubia_limit_usd: full_wage_limit,
        overall_limit_usd: overall_limit,
        qbi_component_usd: component,
        deduction_usd: deduction,
        phase_in_status: status,
        phase_in_fraction: fraction,
        is_limited: (tentative - deduction).abs() > 1e-6,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-2
    }

    fn base() -> QbiInput {
        QbiInput {
            qbi_usd: 100_000.0,
            taxable_income_usd: 150_000.0,
            net_capital_gain_usd: 0.0,
            filing_status: FilingStatus::Single,
            w2_wages_usd: 0.0,
            ubia_usd: 0.0,
            is_sstb: false,
            rate_pct: 20.0,
            threshold_usd: None,
            phase_in_usd: None,
            min_deduction_usd: 400.0,
            min_qbi_floor_usd: 1_000.0,
        }
    }

    #[test]
    fn below_threshold_full_twenty_percent() {
        let r = analyze(&base());
        assert!(close(r.deduction_usd, 20_000.0));
        assert_eq!(r.phase_in_status, "below_threshold");
        assert!(!r.is_limited);
    }

    #[test]
    fn overall_income_cap_binds() {
        let r = analyze(&QbiInput {
            taxable_income_usd: 80_000.0,
            net_capital_gain_usd: 10_000.0,
            ..base()
        });
        // 20% × (80,000 − 10,000) = 14,000 < 20,000.
        assert!(close(r.deduction_usd, 14_000.0));
        assert!(r.is_limited);
    }

    #[test]
    fn above_threshold_wage_limit_binds() {
        let r = analyze(&QbiInput {
            qbi_usd: 300_000.0,
            taxable_income_usd: 350_000.0,
            w2_wages_usd: 50_000.0,
            ..base()
        });
        // tentative 60,000; wage limit 25,000.
        assert!(close(r.wage_ubia_limit_usd, 25_000.0));
        assert!(close(r.deduction_usd, 25_000.0));
        assert_eq!(r.phase_in_status, "fully_phased_in");
    }

    #[test]
    fn sstb_fully_disallowed_above_range() {
        let r = analyze(&QbiInput {
            qbi_usd: 300_000.0,
            taxable_income_usd: 350_000.0,
            is_sstb: true,
            ..base()
        });
        assert!(close(r.deduction_usd, 0.0));
    }

    #[test]
    fn within_phase_in_partial_wage_reduction() {
        let r = analyze(&QbiInput {
            taxable_income_usd: 239_250.0, // over = 37,500, ratio 0.5
            w2_wages_usd: 20_000.0,
            ..base()
        });
        // tentative 20,000; wage limit 10,000; excess 10,000 × 0.5 = 5,000 reduction.
        assert_eq!(r.phase_in_status, "in_phase_in");
        assert!(close(r.phase_in_fraction, 0.5));
        assert!(close(r.deduction_usd, 15_000.0));
    }

    #[test]
    fn minimum_deduction_floor() {
        let r = analyze(&QbiInput {
            qbi_usd: 1_500.0,
            taxable_income_usd: 10_000.0,
            ..base()
        });
        // 20% × 1,500 = 300, but QBI ≥ 1,000 → floored to 400.
        assert!(close(r.deduction_usd, 400.0));
    }

    #[test]
    fn below_floor_no_minimum() {
        let r = analyze(&QbiInput {
            qbi_usd: 800.0,
            taxable_income_usd: 10_000.0,
            ..base()
        });
        // QBI < 1,000 → no floor; 20% × 800 = 160.
        assert!(close(r.deduction_usd, 160.0));
    }

    #[test]
    fn ubia_path_of_wage_limit() {
        let r = analyze(&QbiInput {
            qbi_usd: 200_000.0,
            taxable_income_usd: 350_000.0,
            w2_wages_usd: 20_000.0,
            ubia_usd: 1_000_000.0,
            ..base()
        });
        // 25% × 20,000 + 2.5% × 1,000,000 = 5,000 + 25,000 = 30,000 > 10,000.
        assert!(close(r.wage_ubia_limit_usd, 30_000.0));
        assert!(close(r.deduction_usd, 30_000.0));
    }
}
