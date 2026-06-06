//! Brinson (1986) performance attribution — decomposes the portfolio's
//! active return vs benchmark into:
//!
//!   - **Allocation effect**: did we overweight outperforming sectors?
//!     A_i = (w_p,i − w_b,i) · (r_b,i − r_b,total)
//!
//!   - **Selection effect**: within each sector, did we pick better stocks?
//!     S_i = w_b,i · (r_p,i − r_b,i)
//!
//!   - **Interaction effect**: the cross term.
//!     I_i = (w_p,i − w_b,i) · (r_p,i − r_b,i)
//!
//!   total_active = Σ(A_i + S_i + I_i) = r_p,total − r_b,total
//!
//! Where w_p,i / w_b,i are the portfolio / benchmark weights to sector i,
//! and r_p,i / r_b,i are the realized period returns within each sector.
//!
//! Pure compute. Caller supplies aligned per-sector vectors. The total
//! benchmark and portfolio returns are computed internally as
//! Σ w_b · r_b and Σ w_p · r_p.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrinsonInput {
    pub sector: String,
    pub portfolio_weight: f64,
    pub benchmark_weight: f64,
    pub portfolio_return: f64,
    pub benchmark_return: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectorEffect {
    pub sector: String,
    pub allocation_effect: f64,
    pub selection_effect: f64,
    pub interaction_effect: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BrinsonReport {
    pub per_sector: Vec<SectorEffect>,
    pub total_allocation: f64,
    pub total_selection: f64,
    pub total_interaction: f64,
    pub portfolio_total_return: f64,
    pub benchmark_total_return: f64,
    pub total_active_return: f64,
}

pub fn analyze(inputs: &[BrinsonInput]) -> Option<BrinsonReport> {
    if inputs.is_empty() {
        return None;
    }
    if inputs.iter().any(|s| {
        !s.portfolio_weight.is_finite()
            || !s.benchmark_weight.is_finite()
            || !s.portfolio_return.is_finite()
            || !s.benchmark_return.is_finite()
            || s.portfolio_weight < 0.0
            || s.benchmark_weight < 0.0
    }) {
        return None;
    }
    // Total benchmark return = Σ w_b · r_b (uses provided weights as-is).
    let benchmark_total: f64 = inputs
        .iter()
        .map(|s| s.benchmark_weight * s.benchmark_return)
        .sum();
    let portfolio_total: f64 = inputs
        .iter()
        .map(|s| s.portfolio_weight * s.portfolio_return)
        .sum();
    let mut per_sector = Vec::with_capacity(inputs.len());
    let mut total_a = 0.0_f64;
    let mut total_s = 0.0_f64;
    let mut total_i = 0.0_f64;
    for s in inputs {
        let dw = s.portfolio_weight - s.benchmark_weight;
        let dr = s.portfolio_return - s.benchmark_return;
        let allocation = dw * (s.benchmark_return - benchmark_total);
        let selection = s.benchmark_weight * dr;
        let interaction = dw * dr;
        total_a += allocation;
        total_s += selection;
        total_i += interaction;
        per_sector.push(SectorEffect {
            sector: s.sector.clone(),
            allocation_effect: allocation,
            selection_effect: selection,
            interaction_effect: interaction,
        });
    }
    Some(BrinsonReport {
        per_sector,
        total_allocation: total_a,
        total_selection: total_s,
        total_interaction: total_i,
        portfolio_total_return: portfolio_total,
        benchmark_total_return: benchmark_total,
        total_active_return: portfolio_total - benchmark_total,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(name: &str, pw: f64, bw: f64, pr: f64, br: f64) -> BrinsonInput {
        BrinsonInput {
            sector: name.into(),
            portfolio_weight: pw,
            benchmark_weight: bw,
            portfolio_return: pr,
            benchmark_return: br,
        }
    }

    #[test]
    fn empty_returns_none() {
        assert!(analyze(&[]).is_none());
    }

    #[test]
    fn nan_input_returns_none() {
        assert!(analyze(&[s("A", f64::NAN, 0.5, 0.01, 0.01)]).is_none());
    }

    #[test]
    fn negative_weights_rejected() {
        assert!(analyze(&[s("A", -0.5, 0.5, 0.01, 0.01)]).is_none());
    }

    #[test]
    fn portfolio_identical_to_benchmark_yields_zero_active() {
        let inputs = vec![
            s("Tech", 0.4, 0.4, 0.05, 0.05),
            s("Energy", 0.3, 0.3, -0.02, -0.02),
            s("Health", 0.3, 0.3, 0.01, 0.01),
        ];
        let r = analyze(&inputs).unwrap();
        assert!(r.total_active_return.abs() < 1e-12);
        assert!(r.total_allocation.abs() < 1e-12);
        assert!(r.total_selection.abs() < 1e-12);
        assert!(r.total_interaction.abs() < 1e-12);
    }

    #[test]
    fn allocation_effect_positive_when_overweighting_winners() {
        // Tech beat benchmark mean → overweight Tech → positive A.
        // Benchmark total = 0.4·0.05 + 0.6·0.01 = 0.026
        // (Tech ret 0.05 vs benchmark 0.026 → above-mean sector.)
        let inputs = vec![
            s("Tech", 0.6, 0.4, 0.05, 0.05), // overweight a winner
            s("Other", 0.4, 0.6, 0.01, 0.01),
        ];
        let r = analyze(&inputs).unwrap();
        assert!(r.total_allocation > 0.0);
    }

    #[test]
    fn selection_effect_positive_when_picking_better_stocks() {
        // Within Tech: 0.10 portfolio vs 0.05 benchmark → +5% selection.
        let inputs = vec![
            s("Tech", 0.4, 0.4, 0.10, 0.05),
            s("Other", 0.6, 0.6, 0.02, 0.02),
        ];
        let r = analyze(&inputs).unwrap();
        assert!(r.total_selection > 0.0);
        assert!(r.total_allocation.abs() < 1e-12); // same weights → no allocation effect
    }

    #[test]
    fn interaction_effect_nonzero_when_both_active() {
        let inputs = vec![
            s("Tech", 0.6, 0.4, 0.10, 0.05), // overweight AND outperformer
            s("Other", 0.4, 0.6, 0.02, 0.02),
        ];
        let r = analyze(&inputs).unwrap();
        assert!(r.total_interaction.abs() > 1e-9);
    }

    #[test]
    fn sum_of_effects_equals_total_active_return() {
        let inputs = vec![
            s("Tech", 0.30, 0.20, 0.12, 0.08),
            s("Energy", 0.15, 0.25, -0.03, 0.01),
            s("Health", 0.25, 0.20, 0.05, 0.04),
            s("Fin", 0.30, 0.35, 0.02, 0.03),
        ];
        let r = analyze(&inputs).unwrap();
        let sum_effects = r.total_allocation + r.total_selection + r.total_interaction;
        assert!(
            (sum_effects - r.total_active_return).abs() < 1e-9,
            "A+S+I should equal total active: A+S+I={sum_effects} active={}",
            r.total_active_return
        );
    }

    #[test]
    fn per_sector_count_matches_input() {
        let inputs = vec![s("A", 0.5, 0.5, 0.01, 0.01), s("B", 0.5, 0.5, 0.01, 0.01)];
        let r = analyze(&inputs).unwrap();
        assert_eq!(r.per_sector.len(), 2);
    }
}
