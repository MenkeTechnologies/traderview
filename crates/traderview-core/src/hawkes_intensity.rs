//! Hawkes (1971) process intensity — self-exciting point process for
//! modeling clustered trade arrivals, news bursts, and order-flow
//! clumping.
//!
//!   λ(t) = μ + Σ_{t_i < t} α · e^{−β·(t − t_i)}
//!
//! Each event boosts the future arrival intensity by α, decaying
//! exponentially at rate β. Used in HFT for short-horizon trade-rate
//! forecasting and in market-microstructure research on clustering.
//!
//! This module:
//!   - Given event timestamps + (μ, α, β), computes intensity at each
//!     query time (or per-event).
//!   - Reports the unconditional mean intensity μ / (1 − α/β) (valid
//!     when α < β; otherwise the process is explosive).
//!
//! Pure compute.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HawkesParams {
    pub baseline_mu: f64,
    pub excitation_alpha: f64,
    pub decay_beta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HawkesReport {
    /// Intensity at each query timestamp.
    pub intensities: Vec<f64>,
    pub unconditional_mean_intensity: f64,
    pub is_stable: bool,
}

pub fn compute(
    event_times: &[f64],
    query_times: &[f64],
    params: HawkesParams,
) -> Option<HawkesReport> {
    if !params.baseline_mu.is_finite() || params.baseline_mu < 0.0
        || !params.excitation_alpha.is_finite() || params.excitation_alpha < 0.0
        || !params.decay_beta.is_finite() || params.decay_beta <= 0.0
        || event_times.iter().any(|t| !t.is_finite())
        || query_times.iter().any(|t| !t.is_finite())
    {
        return None;
    }
    // Verify event times are sorted ascending.
    for w in event_times.windows(2) {
        if w[1] < w[0] { return None; }
    }
    let is_stable = params.excitation_alpha < params.decay_beta;
    let unconditional = if is_stable {
        params.baseline_mu / (1.0 - params.excitation_alpha / params.decay_beta)
    } else {
        f64::INFINITY
    };
    let intensities: Vec<f64> = query_times.iter().map(|&t| {
        // Sum excitation from all events strictly before t.
        let mut lambda = params.baseline_mu;
        for &ev in event_times {
            if ev >= t { break; }
            lambda += params.excitation_alpha * (-params.decay_beta * (t - ev)).exp();
        }
        lambda
    }).collect();
    Some(HawkesReport {
        intensities,
        unconditional_mean_intensity: unconditional,
        is_stable,
    })
}

/// Compute intensity JUST AFTER each event time (the "self-excited" peak).
pub fn intensity_after_each_event(event_times: &[f64], params: HawkesParams) -> Option<Vec<f64>> {
    if !params.baseline_mu.is_finite() || params.baseline_mu < 0.0
        || !params.excitation_alpha.is_finite() || params.excitation_alpha < 0.0
        || !params.decay_beta.is_finite() || params.decay_beta <= 0.0
        || event_times.iter().any(|t| !t.is_finite())
    {
        return None;
    }
    for w in event_times.windows(2) {
        if w[1] < w[0] { return None; }
    }
    let mut out = Vec::with_capacity(event_times.len());
    for (i, &t) in event_times.iter().enumerate() {
        let mut lambda = params.baseline_mu;
        // Past events.
        for &prev in &event_times[..i] {
            lambda += params.excitation_alpha * (-params.decay_beta * (t - prev)).exp();
        }
        // Plus the self-excitation kick from THIS event.
        lambda += params.excitation_alpha;
        out.push(lambda);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(mu: f64, alpha: f64, beta: f64) -> HawkesParams {
        HawkesParams { baseline_mu: mu, excitation_alpha: alpha, decay_beta: beta }
    }

    #[test]
    fn invalid_params_return_none() {
        assert!(compute(&[], &[], p(-0.1, 0.5, 1.0)).is_none());
        assert!(compute(&[], &[], p(0.1, -0.1, 1.0)).is_none());
        assert!(compute(&[], &[], p(0.1, 0.5, 0.0)).is_none());
        assert!(compute(&[], &[], p(f64::NAN, 0.5, 1.0)).is_none());
    }

    #[test]
    fn unsorted_events_rejected() {
        let evs = vec![1.0, 0.5];
        assert!(compute(&evs, &[2.0], p(0.1, 0.5, 1.0)).is_none());
    }

    #[test]
    fn no_events_yields_baseline_intensity() {
        let r = compute(&[], &[1.0, 2.0, 3.0], p(0.5, 0.0, 1.0)).unwrap();
        assert!(r.intensities.iter().all(|x| (x - 0.5).abs() < 1e-12));
    }

    #[test]
    fn event_burst_inflates_local_intensity() {
        // 5 events at t=1..5, query at t=5.5.
        let evs = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let r = compute(&evs, &[0.5, 5.5], p(0.1, 0.5, 1.0)).unwrap();
        assert!(r.intensities[1] > r.intensities[0]);
        assert!(r.intensities[1] > 0.1);
    }

    #[test]
    fn stable_when_alpha_below_beta() {
        let r = compute(&[], &[1.0], p(0.1, 0.3, 1.0)).unwrap();
        assert!(r.is_stable);
        assert!(r.unconditional_mean_intensity.is_finite());
    }

    #[test]
    fn unstable_when_alpha_at_or_above_beta() {
        let r = compute(&[], &[1.0], p(0.1, 1.0, 1.0)).unwrap();
        assert!(!r.is_stable);
        assert!(r.unconditional_mean_intensity.is_infinite());
    }

    #[test]
    fn intensity_after_event_includes_self_kick() {
        let evs = vec![1.0];
        let r = intensity_after_each_event(&evs, p(0.1, 0.5, 1.0)).unwrap();
        // Just after t=1: μ + α (since no prior events).
        assert!((r[0] - (0.1 + 0.5)).abs() < 1e-9);
    }

    #[test]
    fn decay_back_toward_baseline() {
        // After a single event at t=0, intensity at t=10 with β=1 should
        // be very close to μ.
        let evs = vec![0.0];
        let r = compute(&evs, &[10.0], p(0.1, 0.5, 1.0)).unwrap();
        assert!((r.intensities[0] - 0.1).abs() < 0.001);
    }

    #[test]
    fn unconditional_mean_formula() {
        let r = compute(&[], &[1.0], p(1.0, 0.5, 1.0)).unwrap();
        // μ / (1 − α/β) = 1.0 / (1 − 0.5) = 2.0.
        assert!((r.unconditional_mean_intensity - 2.0).abs() < 1e-9);
    }
}
