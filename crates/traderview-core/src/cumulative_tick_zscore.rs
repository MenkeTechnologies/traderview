//! Cumulative TICK Z-Score — running sum of NYSE TICK normalized to a
//! z-score of its own rolling history.
//!
//! Builds on the raw TICK series:
//!   cum_tick_t = Σ tick                                  (cumulative since session)
//!   z_t        = (cum_tick_t - SMA(cum_tick, period))
//!                / stdev(cum_tick, period)
//!
//! Useful for spotting cumulative breadth divergence — if cum-TICK is
//! at a relative high but underlying index is making a new high,
//! breadth is confirming. If z drops while index makes new high,
//! breadth is diverging.
//!
//! Pure compute. Default period = 60. Companion to `nyse_tick`,
//! `cumulative_tick_trin`, `breadth_lines`, `arms_high_low_index`.

pub fn compute(tick: &[f64], period: usize) -> Vec<Option<f64>> {
    let n = tick.len();
    let mut out = vec![None; n];
    if period < 3 || n < period { return out; }
    if tick.iter().any(|x| !x.is_finite()) { return out; }
    let mut cum = vec![0.0_f64; n];
    cum[0] = tick[0];
    for i in 1..n {
        cum[i] = cum[i - 1] + tick[i];
    }
    let p_f = period as f64;
    for (i, slot) in out.iter_mut().enumerate().skip(period - 1) {
        let win = &cum[i + 1 - period..=i];
        let mean: f64 = win.iter().sum::<f64>() / p_f;
        let var: f64 = win.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / p_f;
        let std = var.max(0.0).sqrt();
        if std > 0.0 {
            *slot = Some((cum[i] - mean) / std);
        } else {
            *slot = Some(0.0);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_inputs_return_empty() {
        let t = vec![100.0_f64; 100];
        assert!(compute(&t, 1).iter().all(|x| x.is_none()));
        assert!(compute(&t[..5], 60).iter().all(|x| x.is_none()));
    }

    #[test]
    fn nan_returns_empty() {
        let mut t = vec![100.0_f64; 100];
        t[5] = f64::NAN;
        assert!(compute(&t, 60).iter().all(|x| x.is_none()));
    }

    #[test]
    fn flat_tick_yields_high_z_due_to_linear_drift() {
        // Constant tick → cum grows linearly. Linear sequence has
        // non-zero stdev → z is well-defined and grows linearly itself.
        // Verify it doesn't panic and stays finite.
        let t = vec![100.0_f64; 100];
        let r = compute(&t, 60);
        for v in r.iter().flatten() {
            assert!(v.is_finite());
        }
    }

    #[test]
    fn zero_tick_yields_zero_z() {
        // All zeros → cum stays 0 → mean=0, stdev=0 → z=0.
        let t = vec![0.0_f64; 100];
        let r = compute(&t, 60);
        for v in r.iter().flatten() {
            assert!(v.abs() < 1e-9);
        }
    }

    #[test]
    fn final_burst_yields_high_z() {
        // 90 zeros then 10 large positives → cum spikes → z high.
        let mut t = vec![0.0_f64; 90];
        t.extend(vec![500.0; 10]);
        let r = compute(&t, 60);
        let last = t.len() - 1;
        assert!(r[last].unwrap() > 0.0);
    }

    #[test]
    fn output_length_matches_input() {
        let t = vec![100.0_f64; 100];
        assert_eq!(compute(&t, 60).len(), 100);
    }
}
