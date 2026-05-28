//! Deterministic fuzz harness for the breadth / pattern primitives added
//! by commit 43015081f3 plus their immediate neighbors (gap_fill_stats,
//! liquidity_grab, acceleration_deceleration).
//!
//! Each suite throws ~20k pseudo-random inputs at one function and asserts:
//!   1. no panic (i64 / usize overflow, OOB index, etc.)
//!   2. every produced Some(f64) is finite (no NaN, no Inf)
//!   3. structural invariants (vec lengths, event counts non-negative, etc.)
//!
//! Uses a tiny LCG so no extra crates and reproducible failures.

#![allow(clippy::unusual_byte_groupings, clippy::manual_is_multiple_of)]

use traderview_core::{
    acceleration_deceleration, anchored_vwap, aroon, arms_index, atr_cone,
    awesome_oscillator, bond_duration, breakout_detector, chaikin_money_flow,
    choppiness, coppock, correlation, cusum, dema, displacement, dynamic_kelly,
    elder_ray, equity_regime, fair_value_gap, futures_roll, gap_fill_stats,
    hull_ma, indicators, inside_bar_breakout, kama, liquidity_grab, mass_index,
    mcclellan_oscillator, monte_carlo, opening_range, order_block, pair_trade,
    portfolio_heat, ppo, premium_discount, put_call_ratio, random_walk_index,
    range_contraction, range_expansion, rolling_zscore, round_levels,
    sharpe_by_window, sortino, stoch_rsi, stochastic, stop_hunt, supertrend,
    swing_points, tema, three_bar_reversal, treynor, tsi, ulcer_index,
    ultimate_oscillator, volatility_stop, volume_burst, vortex, vsa, wyckoff,
};
use traderview_core::models::TradeSide;
use chrono::{NaiveDate, TimeZone, Utc};

const ITERS: usize = 20_000;

struct Lcg(u64);
impl Lcg {
    fn new(seed: u64) -> Self { Self(seed) }
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.0
    }
    fn range_usize(&mut self, n: usize) -> usize { (self.next_u64() as usize) % n.max(1) }
    fn f64_range(&mut self, lo: f64, hi: f64) -> f64 {
        let r = (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
        lo + (hi - lo) * r
    }
    /// Pick a "bad" f64 — NaN / Inf / huge / zero — to exercise guards.
    fn pick_pathological_f64(&mut self) -> f64 {
        match self.next_u64() % 8 {
            0 => f64::NAN,
            1 => f64::INFINITY,
            2 => f64::NEG_INFINITY,
            3 => -1.0e9,
            4 => 0.0,
            5 => f64::MIN_POSITIVE,
            6 => f64::MAX / 2.0,
            _ => self.f64_range(-1.0e6, 1.0e6),
        }
    }
    fn pick_i64(&mut self) -> i64 {
        match self.next_u64() % 6 {
            0 => i64::MIN,
            1 => i64::MAX,
            2 => 0,
            3 => -1,
            4 => 1,
            _ => self.next_u64() as i64,
        }
    }
    fn pick_u64_small(&mut self) -> u64 { self.next_u64() % 1_000_000 }
}

fn assert_finite_opt(v: &Option<f64>, msg: &str) {
    if let Some(x) = v {
        assert!(x.is_finite(), "{msg}: expected finite, got {x}");
    }
}

// ---------------------------------------------------------------------------
// arms_index
// ---------------------------------------------------------------------------
#[test]
fn fuzz_arms_index() {
    let mut rng = Lcg::new(0xA11_BAD_DA7A);
    for it in 0..ITERS {
        let n = rng.range_usize(64);
        let bars: Vec<arms_index::BreadthBar> = (0..n).map(|_| arms_index::BreadthBar {
            advancing_issues: rng.pick_u64_small(),
            declining_issues: rng.pick_u64_small(),
            advancing_volume: rng.pick_pathological_f64(),
            declining_volume: rng.pick_pathological_f64(),
        }).collect();
        let r = arms_index::compute(&bars);
        assert_eq!(r.series.len(), bars.len(), "iter {it}");
        for v in &r.series { assert_finite_opt(v, "series elem"); }
        assert_finite_opt(&r.latest, "latest");
    }
}

// ---------------------------------------------------------------------------
// mcclellan_oscillator
// ---------------------------------------------------------------------------
#[test]
fn fuzz_mcclellan_oscillator() {
    let mut rng = Lcg::new(0xC0FFEE_F00D);
    for it in 0..ITERS {
        let n = rng.range_usize(128);
        let bars: Vec<mcclellan_oscillator::BreadthBar> = (0..n).map(|_| {
            mcclellan_oscillator::BreadthBar {
                advancing_issues: rng.pick_i64(),
                declining_issues: rng.pick_i64(),
            }
        }).collect();
        let r = mcclellan_oscillator::compute(&bars);
        assert_eq!(r.oscillator.len(), bars.len(), "iter {it} osc len");
        assert_eq!(r.summation_index.len(), bars.len(), "iter {it} sum len");
        // EMAs of f64-cast net-advances should stay finite for any realistic
        // i64 magnitude that fits in f64. i64::{MIN,MAX} casts to ~±9.2e18
        // which is well below f64::MAX, and the EMA recursion is a convex
        // combination so it can't escape the input range.
        for (i, v) in r.oscillator.iter().enumerate() {
            assert_finite_opt(v, &format!("iter {it} osc[{i}]"));
        }
        for (i, v) in r.summation_index.iter().enumerate() {
            assert_finite_opt(v, &format!("iter {it} sum[{i}]"));
        }
        let _ = r.regime;
    }
}

// ---------------------------------------------------------------------------
// inside_bar_breakout
// ---------------------------------------------------------------------------
#[test]
fn fuzz_inside_bar_breakout() {
    let mut rng = Lcg::new(0xDEAD_BAD_BEEF);
    for it in 0..ITERS {
        let n = rng.range_usize(64);
        // Produce mostly-valid OHLC plus the occasional pathological bar.
        let bars: Vec<inside_bar_breakout::OhlcBar> = (0..n).map(|_| {
            let h = rng.f64_range(50.0, 200.0);
            let l = rng.f64_range(50.0, h);
            let o = rng.f64_range(l, h);
            let c = rng.f64_range(l, h);
            // 10% chance of NaN/Inf to exercise edges.
            if rng.next_u64() % 10 == 0 {
                inside_bar_breakout::OhlcBar {
                    open: rng.pick_pathological_f64(),
                    high: rng.pick_pathological_f64(),
                    low:  rng.pick_pathological_f64(),
                    close: rng.pick_pathological_f64(),
                }
            } else {
                inside_bar_breakout::OhlcBar { open: o, high: h, low: l, close: c }
            }
        }).collect();
        let cfg = inside_bar_breakout::IbConfig {
            confirm_within: if rng.next_u64() % 20 == 0 { usize::MAX } else { rng.range_usize(10) },
            max_range_ratio: rng.f64_range(-0.1, 1.5),
        };
        let r = inside_bar_breakout::detect(&bars, &cfg);
        assert_eq!(r.n_events, r.events.len(), "iter {it} count mismatch");
        // Event indices must be in bounds.
        for e in &r.events {
            assert!(e.mother_bar < n);
            assert!(e.inside_bar < n);
            if let Some(b) = e.breakout_bar {
                assert!(b < n, "breakout_bar {b} OOB for n={n}");
            }
        }
        assert_eq!(
            r.n_resolved_up + r.n_resolved_down + r.n_unresolved,
            r.n_events,
            "iter {it} bucket sum != n_events",
        );
        // range_ratio is cur_range / mother_range — both built from
        // user-supplied OHLC. With pathological NaN/Inf input the ratio
        // may itself be NaN/Inf — that's OK for now, but verify NO finite
        // ratio ever exceeds the configured max (would mean the guard
        // dropped through).
        for e in &r.events {
            if e.range_ratio.is_finite() {
                assert!(e.range_ratio <= cfg.max_range_ratio + 1e-12,
                    "iter {it} ratio {} > max {}", e.range_ratio, cfg.max_range_ratio);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// gap_fill_stats
// ---------------------------------------------------------------------------
#[test]
fn fuzz_gap_fill_stats() {
    let mut rng = Lcg::new(0x6AB_F177);
    for it in 0..ITERS {
        let n = rng.range_usize(64);
        let bars: Vec<gap_fill_stats::OhlcBar> = (0..n).map(|_| {
            if rng.next_u64() % 10 == 0 {
                gap_fill_stats::OhlcBar {
                    open: rng.pick_pathological_f64(),
                    high: rng.pick_pathological_f64(),
                    low:  rng.pick_pathological_f64(),
                    close: rng.pick_pathological_f64(),
                }
            } else {
                let h = rng.f64_range(50.0, 200.0);
                let l = rng.f64_range(50.0, h);
                let o = rng.f64_range(l, h);
                let c = rng.f64_range(l, h);
                gap_fill_stats::OhlcBar { open: o, high: h, low: l, close: c }
            }
        }).collect();
        let atr: Vec<f64> = (0..n).map(|_| rng.pick_pathological_f64()).collect();
        let r = gap_fill_stats::analyze(&bars, &atr);
        assert!(r.n_gaps == r.n_up_gaps + r.n_down_gaps, "iter {it}");
        // Fill rates are k/n with both finite small counts → always finite.
        assert!(r.up_fill_rate.is_finite(),   "iter {it} up_fill_rate {}", r.up_fill_rate);
        assert!(r.down_fill_rate.is_finite(), "iter {it} down_fill_rate {}", r.down_fill_rate);
        for e in &r.events {
            assert!(e.bar_index < n);
            assert!(e.gap_size.is_finite() || e.gap_size.is_nan(),
                "iter {it} gap_size {}", e.gap_size);
            // gap_size_atrs may be NaN/Inf if the user-supplied ATR is
            // pathological — the analyze() contract just passes that
            // through. But it must NOT panic.
        }
    }
}

// ---------------------------------------------------------------------------
// liquidity_grab
// ---------------------------------------------------------------------------
#[test]
fn fuzz_liquidity_grab() {
    let mut rng = Lcg::new(0xF1_5CA1);
    for it in 0..ITERS {
        let n = rng.range_usize(64);
        let bars: Vec<liquidity_grab::OhlcBar> = (0..n).map(|_| {
            let h = rng.f64_range(50.0, 200.0);
            let l = rng.f64_range(50.0, h);
            let o = rng.f64_range(l, h);
            let c = rng.f64_range(l, h);
            liquidity_grab::OhlcBar { open: o, high: h, low: l, close: c }
        }).collect();
        let atr: Vec<f64> = (0..n).map(|_| rng.f64_range(0.1, 5.0)).collect();
        let swings_n = if n == 0 { 0 } else { rng.range_usize(n.min(8) + 1) };
        let swings: Vec<swing_points::SwingPoint> = (0..swings_n).map(|_| {
            let idx = if n == 0 { 0 } else { rng.range_usize(n) };
            let kind = if rng.next_u64() & 1 == 0 {
                swing_points::SwingKind::High
            } else {
                swing_points::SwingKind::Low
            };
            swing_points::SwingPoint { index: idx, price: rng.f64_range(50.0, 200.0), kind }
        }).collect();
        let cfg = liquidity_grab::GrabConfig {
            min_sweep_atrs: rng.f64_range(-0.5, 2.0),
            confirm_within: if rng.next_u64() % 20 == 0 { usize::MAX } else { rng.range_usize(8) },
            min_followthrough: rng.range_usize(8),
        };
        let r = liquidity_grab::detect(&bars, &atr, &swings, &cfg);
        assert_eq!(r.n_events, r.events.len(), "iter {it}");
        for e in &r.events {
            assert!(e.sweep_bar < n);
            assert!(e.confirm_bar < n);
            assert!(e.swing_index < swings.len());
        }
    }
}

// ---------------------------------------------------------------------------
// acceleration_deceleration
// ---------------------------------------------------------------------------
#[test]
fn fuzz_acceleration_deceleration() {
    let mut rng = Lcg::new(0xACDC_ACDC);
    for it in 0..ITERS {
        let n = rng.range_usize(120);
        let bars: Vec<acceleration_deceleration::HlBar> = (0..n).map(|_| {
            let h = rng.f64_range(50.0, 200.0);
            let l = rng.f64_range(50.0, h);
            acceleration_deceleration::HlBar { high: h, low: l }
        }).collect();
        let r = acceleration_deceleration::compute(&bars);
        assert_eq!(r.ao.len(), bars.len(), "iter {it} ao len");
        assert_eq!(r.ac.len(), bars.len(), "iter {it} ac len");
        for v in &r.ao { assert_finite_opt(v, "ao"); }
        for v in &r.ac { assert_finite_opt(v, "ac"); }
    }
}

// ---------------------------------------------------------------------------
// atr_cone — DoS-cap + length invariant
// ---------------------------------------------------------------------------
#[test]
fn fuzz_atr_cone() {
    let mut rng = Lcg::new(0x4E_5C_0A_E1);
    for it in 0..ITERS / 4 {  // smaller suite — each call alloc up to 1000 entries
        let entry = rng.pick_pathological_f64();
        let atr = rng.pick_pathological_f64();
        // Mostly bounded, occasionally hostile.
        let horizon = match rng.next_u64() % 10 {
            0 => usize::MAX,
            1 => usize::MAX - 1,
            2 => 10_000,
            _ => rng.range_usize(50),
        };
        let out = atr_cone::project(entry, atr, horizon);
        assert!(out.len() <= 1001, "iter {it} unbounded growth: {}", out.len());
        for p in &out { assert!(p.days_forward < out.len()); }
    }
}

// ---------------------------------------------------------------------------
// breakout_detector — fuzz NaN/Inf bar inputs (used to spuriously fire)
// ---------------------------------------------------------------------------
#[test]
fn fuzz_breakout_detector() {
    let mut rng = Lcg::new(0xB7E4_007);
    for it in 0..ITERS {
        let n = rng.range_usize(64);
        let bars: Vec<breakout_detector::OhlcBar> = (0..n).map(|_| {
            if rng.next_u64() % 5 == 0 {
                breakout_detector::OhlcBar {
                    high: rng.pick_pathological_f64(),
                    low:  rng.pick_pathological_f64(),
                    close: rng.pick_pathological_f64(),
                }
            } else {
                let h = rng.f64_range(50.0, 200.0);
                let l = rng.f64_range(50.0, h);
                let c = rng.f64_range(l, h);
                breakout_detector::OhlcBar { high: h, low: l, close: c }
            }
        }).collect();
        let cfg = breakout_detector::BreakoutConfig {
            lookback: if rng.next_u64() % 20 == 0 { usize::MAX } else { rng.range_usize(20) },
            buffer: rng.f64_range(-1.0, 5.0),
            close_only: rng.next_u64() & 1 == 0,
        };
        let r = breakout_detector::detect(&bars, &cfg);
        assert_eq!(r.n_events, r.events.len(), "iter {it}");
        for e in &r.events {
            assert!(e.bar_index < n);
            // After the NaN-window fix, reference_level must always be finite.
            assert!(e.reference_level.is_finite(),
                "iter {it} event ref_level non-finite: {}", e.reference_level);
        }
    }
}

// ---------------------------------------------------------------------------
// choppiness
// ---------------------------------------------------------------------------
#[test]
fn fuzz_choppiness() {
    let mut rng = Lcg::new(0xC4_0BB);
    for it in 0..ITERS {
        let n = rng.range_usize(64);
        let bars: Vec<choppiness::OhlcBar> = (0..n).map(|_| {
            let h = rng.f64_range(50.0, 200.0);
            let l = rng.f64_range(50.0, h);
            let c = rng.f64_range(l, h);
            choppiness::OhlcBar { high: h, low: l, close: c }
        }).collect();
        let period = match rng.next_u64() % 20 {
            0 => usize::MAX,
            1 => 0,
            2 => 1,
            _ => rng.range_usize(20),
        };
        let r = choppiness::compute(&bars, period);
        assert_eq!(r.series.len(), bars.len(), "iter {it}");
        for v in &r.series { assert_finite_opt(v, "ci"); }
    }
}

// ---------------------------------------------------------------------------
// dynamic_kelly
// ---------------------------------------------------------------------------
#[test]
fn fuzz_dynamic_kelly() {
    let mut rng = Lcg::new(0xDE10_001);
    for it in 0..ITERS {
        let n = rng.range_usize(64);
        let trades: Vec<f64> = (0..n).map(|_| rng.pick_pathological_f64()).collect();
        let window = match rng.next_u64() % 12 {
            0 => 0,
            1 => usize::MAX,
            _ => rng.range_usize(20),
        };
        let r = dynamic_kelly::compute(&trades, window);
        // Either empty (window=0) or aligned to input length.
        assert!(r.is_empty() || r.len() == trades.len(),
            "iter {it} weird length {} for n={n}", r.len());
        for p in &r {
            // window_win_rate is unconditionally written — must be finite.
            assert!(p.window_win_rate.is_finite(),
                "iter {it} win_rate non-finite: {}", p.window_win_rate);
            assert_finite_opt(&p.window_payoff_ratio, "payoff");
            assert_finite_opt(&p.kelly_fraction, "kelly");
            assert_finite_opt(&p.half_kelly_fraction, "half_kelly");
            // Kelly must be clamped to [-1, 1].
            if let Some(k) = p.kelly_fraction {
                assert!((-1.0..=1.0).contains(&k),
                    "iter {it} kelly outside [-1,1]: {k}");
            }
        }
    }
}

// ---------------------------------------------------------------------------
// equity_regime
// ---------------------------------------------------------------------------
#[test]
fn fuzz_equity_regime() {
    let mut rng = Lcg::new(0xE21_7E61);
    for it in 0..ITERS {
        let n = rng.range_usize(64);
        let equity: Vec<f64> = (0..n).map(|_| rng.f64_range(5_000.0, 50_000.0)).collect();
        let cfg = equity_regime::DetectorConfig {
            trend_slope_pct: rng.f64_range(-0.01, 0.1),
            clean_trend_rel_stdev: rng.f64_range(-0.01, 0.5),
        };
        let r = equity_regime::analyze(&equity, &cfg);
        assert_eq!(r.n, equity.len(), "iter {it}");
        assert!(r.r_squared.is_finite(), "iter {it} r²={}", r.r_squared);
        assert!(r.slope_per_period.is_finite(), "iter {it}");
        assert!(r.residual_stdev.is_finite(), "iter {it}");
    }
}

// ---------------------------------------------------------------------------
// futures_roll
// ---------------------------------------------------------------------------
#[test]
fn fuzz_futures_roll() {
    let mut rng = Lcg::new(0xF07_E5_E0);
    let today = NaiveDate::from_ymd_opt(2026, 5, 27).unwrap();
    for it in 0..ITERS {
        let m = rng.range_usize(16);
        let positions: Vec<futures_roll::FuturesPosition> = (0..m).map(|i| {
            // Spread expiry over a wide range to exercise sort + edge cases.
            let day_offset = (rng.next_u64() % 1000) as i64 - 100;
            let exp = today.checked_add_signed(chrono::Duration::days(day_offset))
                .unwrap_or(today);
            futures_roll::FuturesPosition {
                symbol: format!("/X{i}"),
                contracts: rng.pick_i64() / 1000,
                expiration: exp,
            }
        }).collect();
        // Hostile window values to exercise saturating_mul fix.
        let window = match rng.next_u64() % 10 {
            0 => i64::MAX,
            1 => i64::MAX - 1,
            2 => 0,
            3 => -1,
            _ => (rng.next_u64() % 60) as i64,
        };
        let r = futures_roll::schedule(&positions, today, window);
        assert_eq!(r.rows.len(), m, "iter {it}");
        // Rows must be sorted ascending by days_to_expiry.
        for w in r.rows.windows(2) {
            assert!(w[0].days_to_expiry <= w[1].days_to_expiry,
                "iter {it} unsorted: {} then {}", w[0].days_to_expiry, w[1].days_to_expiry);
        }
    }
}

// ---------------------------------------------------------------------------
// correlation::pearson + beta
// ---------------------------------------------------------------------------
#[test]
fn fuzz_correlation_pearson_beta() {
    let mut rng = Lcg::new(0xC0_FFEE);
    for it in 0..ITERS {
        let n = rng.range_usize(32);
        let a: Vec<f64> = (0..n).map(|_| rng.f64_range(-1.0, 1.0)).collect();
        let b: Vec<f64> = (0..n).map(|_| rng.f64_range(-1.0, 1.0)).collect();
        if let Some(p) = correlation::pearson(&a, &b) {
            assert!(p.is_finite(), "iter {it} pearson non-finite: {p}");
            assert!((-1.0..=1.0).contains(&p) || p.abs() <= 1.0 + 1e-9,
                "iter {it} pearson outside [-1,1]: {p}");
        }
        if let Some(b_v) = correlation::beta(&a, &b) {
            assert!(b_v.is_finite(), "iter {it} beta non-finite");
        }
    }
}

// ---------------------------------------------------------------------------
// pair_trade
// ---------------------------------------------------------------------------
#[test]
fn fuzz_pair_trade() {
    let mut rng = Lcg::new(0x9A1_07);
    for it in 0..ITERS {
        let n = rng.range_usize(32);
        let x: Vec<f64> = (0..n).map(|_| rng.f64_range(10.0, 200.0)).collect();
        let y: Vec<f64> = (0..n).map(|_| rng.f64_range(10.0, 200.0)).collect();
        let cfg = pair_trade::PairConfig::default();
        if let Some(r) = pair_trade::analyze(&y, &x, &cfg) {
            assert!(r.hedge_ratio.is_finite(), "iter {it} beta");
            assert!(r.spread_mean.is_finite(), "iter {it} mean");
            assert!(r.spread_stdev.is_finite(), "iter {it} stdev");
            assert!(r.current_spread.is_finite(), "iter {it} spread");
            assert!(r.current_z.is_finite(), "iter {it} z");
        }
    }
}

// ---------------------------------------------------------------------------
// portfolio_heat
// ---------------------------------------------------------------------------
#[test]
fn fuzz_portfolio_heat() {
    let mut rng = Lcg::new(0x4E_A7);
    for it in 0..ITERS {
        let np = rng.range_usize(8);
        let positions: Vec<portfolio_heat::OpenRiskPosition> = (0..np).map(|i| {
            portfolio_heat::OpenRiskPosition {
                symbol: format!("S{i}"),
                dollar_risk: rng.f64_range(0.0, 10_000.0),
            }
        }).collect();
        let nc = rng.range_usize(8);
        let corrs: Vec<portfolio_heat::CorrEdge> = (0..nc).map(|_| {
            let a = rng.range_usize(np.max(1));
            let b = rng.range_usize(np.max(1));
            portfolio_heat::CorrEdge {
                a: format!("S{a}"),
                b: format!("S{b}"),
                corr: rng.f64_range(-1.0, 1.0),
            }
        }).collect();
        let candidate = portfolio_heat::CandidateTrade {
            symbol: format!("S{}", rng.range_usize(np.max(1) + 1)),
            dollar_risk: rng.f64_range(0.0, 5_000.0),
        };
        let cfg = portfolio_heat::HeatConfig {
            bundle_threshold: rng.f64_range(0.0, 1.0),
            bundle_budget: rng.f64_range(0.0, 50_000.0),
            total_budget: rng.f64_range(0.0, 100_000.0),
        };
        let r = portfolio_heat::evaluate(&positions, &corrs, &candidate, &cfg);
        assert!(r.bundle_existing_heat.is_finite(), "iter {it}");
        assert!(r.bundle_projected_heat.is_finite(), "iter {it}");
        assert!(r.portfolio_existing_heat.is_finite(), "iter {it}");
        assert!(r.portfolio_projected_heat.is_finite(), "iter {it}");
        // Bundle members must always include the candidate symbol.
        assert!(r.bundle_members.contains(&candidate.symbol));
    }
}

// ---------------------------------------------------------------------------
// rolling_zscore
// ---------------------------------------------------------------------------
#[test]
fn fuzz_rolling_zscore() {
    let mut rng = Lcg::new(0x20_5C0);
    for it in 0..ITERS {
        let n = rng.range_usize(64);
        let series: Vec<f64> = (0..n).map(|_| rng.f64_range(-100.0, 100.0)).collect();
        let window = match rng.next_u64() % 12 {
            0 => 0,
            1 => usize::MAX,
            _ => rng.range_usize(20),
        };
        let out = rolling_zscore::compute(&series, window);
        assert_eq!(out.len(), n, "iter {it}");
        for p in &out {
            assert!(p.window_mean.is_finite(), "iter {it} mean");
            assert!(p.window_stdev.is_finite(), "iter {it} stdev");
            assert!(p.z_score.is_finite(), "iter {it} z");
            assert!(p.window_stdev >= 0.0, "iter {it} stdev neg");
        }
    }
}

// ---------------------------------------------------------------------------
// ulcer_index
// ---------------------------------------------------------------------------
#[test]
fn fuzz_ulcer_index() {
    let mut rng = Lcg::new(0x1C7E5);
    for it in 0..ITERS {
        let n = rng.range_usize(64);
        let equity: Vec<f64> = (0..n).map(|_| rng.f64_range(1_000.0, 50_000.0)).collect();
        let rf = if rng.next_u64() & 1 == 0 { None } else { Some(rng.f64_range(-10.0, 20.0)) };
        let r = ulcer_index::compute(&equity, rf);
        assert!(r.ulcer_index.is_finite(), "iter {it} UI");
        assert!(r.ulcer_index >= 0.0, "iter {it} UI negative: {}", r.ulcer_index);
        assert!(r.max_drawdown_pct.is_finite(), "iter {it} maxdd");
        assert!(r.max_drawdown_pct >= 0.0, "iter {it} maxdd negative: {}", r.max_drawdown_pct);
        assert_finite_opt(&r.upi, "upi");
    }
}

// ---------------------------------------------------------------------------
// monte_carlo
// ---------------------------------------------------------------------------
#[test]
fn fuzz_monte_carlo() {
    let mut rng = Lcg::new(0x_70_8C);
    for it in 0..ITERS / 20 {  // each call iterates a lot
        let nh = rng.range_usize(16) + 1;
        let history: Vec<f64> = (0..nh).map(|_| rng.f64_range(-500.0, 500.0)).collect();
        let cfg = monte_carlo::McConfig {
            n_curves: rng.range_usize(50) + 1,
            trades_per_curve: rng.range_usize(50) + 1,
            start_equity: rng.f64_range(1_000.0, 100_000.0),
            ruin_threshold: rng.f64_range(0.0, 5_000.0),
            seed: rng.next_u64(),
        };
        if let Some(r) = monte_carlo::simulate(&history, &cfg) {
            assert!(r.mean_ending_equity.is_finite(), "iter {it} mean");
            assert!(r.probability_of_ruin.is_finite(), "iter {it} ruin");
            assert!((0.0..=1.0).contains(&r.probability_of_ruin),
                "iter {it} ruin out of range: {}", r.probability_of_ruin);
            assert!((0.0..=1.0).contains(&r.probability_profitable),
                "iter {it} profit out of range: {}", r.probability_profitable);
            // Percentiles must be monotonic.
            assert!(r.ending_equity_p05 <= r.ending_equity_p25);
            assert!(r.ending_equity_p25 <= r.ending_equity_p50);
            assert!(r.ending_equity_p50 <= r.ending_equity_p75);
            assert!(r.ending_equity_p75 <= r.ending_equity_p95);
        }
    }
}

// ---------------------------------------------------------------------------
// put_call_ratio
// ---------------------------------------------------------------------------
#[test]
fn fuzz_put_call_ratio() {
    let mut rng = Lcg::new(0x_9C_4A_7);
    for _ in 0..ITERS {
        let input = put_call_ratio::PutCallInput {
            put_volume: rng.next_u64() % 1_000_000_000,
            call_volume: rng.next_u64() % 1_000_000_000,
            put_oi: rng.next_u64() % 1_000_000_000,
            call_oi: rng.next_u64() % 1_000_000_000,
        };
        let thresh = put_call_ratio::Thresholds {
            bullish_extreme_below: rng.f64_range(0.0, 5.0),
            bearish_extreme_above: rng.f64_range(0.0, 5.0),
        };
        let r = put_call_ratio::compute(&input, &thresh);
        if let Some(v) = r.volume_pc_ratio {
            assert!(v.is_finite() && v >= 0.0, "vol ratio {v}");
        }
        if let Some(v) = r.oi_pc_ratio {
            assert!(v.is_finite() && v >= 0.0, "oi ratio {v}");
        }
    }
}

// ---------------------------------------------------------------------------
// random_walk_index
// ---------------------------------------------------------------------------
#[test]
fn fuzz_random_walk_index() {
    let mut rng = Lcg::new(0x_4B7A);
    for it in 0..ITERS / 4 {  // RWI has O(n*N) cost — keep iters modest
        let n = rng.range_usize(40);
        let bars: Vec<random_walk_index::OhlcBar> = (0..n).map(|_| {
            let h = rng.f64_range(50.0, 200.0);
            let l = rng.f64_range(50.0, h);
            let c = rng.f64_range(l, h);
            random_walk_index::OhlcBar { high: h, low: l, close: c }
        }).collect();
        let max_n = match rng.next_u64() % 10 {
            0 => 0,
            1 => 1,
            2 => usize::MAX,
            _ => rng.range_usize(15),
        };
        let r = random_walk_index::compute(&bars, max_n);
        assert_eq!(r.rwi_high.len(), n, "iter {it} high len");
        assert_eq!(r.rwi_low.len(), n, "iter {it} low len");
        for v in &r.rwi_high { assert_finite_opt(v, "rwi_high"); }
        for v in &r.rwi_low  { assert_finite_opt(v, "rwi_low"); }
    }
}

// ---------------------------------------------------------------------------
// order_block — hostile expansion_window to trigger the slice-panic guard
// ---------------------------------------------------------------------------
#[test]
fn fuzz_order_block() {
    let mut rng = Lcg::new(0x_0_8_DE_F);
    for it in 0..ITERS {
        let n = rng.range_usize(32);
        let bars: Vec<order_block::OhlcBar> = (0..n).map(|_| {
            let h = rng.f64_range(50.0, 200.0);
            let l = rng.f64_range(50.0, h);
            let o = rng.f64_range(l, h);
            let c = rng.f64_range(l, h);
            order_block::OhlcBar { open: o, high: h, low: l, close: c }
        }).collect();
        let cfg = order_block::OrderBlockConfig {
            expansion_window: match rng.next_u64() % 12 {
                0 => usize::MAX,
                1 => usize::MAX - 1,
                2 => 0,
                _ => rng.range_usize(8),
            },
            expansion_multiple: rng.f64_range(-1.0, 5.0),
        };
        let r = order_block::detect(&bars, &cfg);
        for b in &r.blocks {
            assert!(b.bar_index < n);
            assert!(b.expansion_magnitude.is_finite());
            assert!(b.zone_high >= b.zone_low,
                "iter {it} zone inverted: high={} low={}", b.zone_high, b.zone_low);
        }
    }
}

// ---------------------------------------------------------------------------
// displacement
// ---------------------------------------------------------------------------
#[test]
fn fuzz_displacement() {
    let mut rng = Lcg::new(0x_D15_AA);
    for it in 0..ITERS {
        let n = rng.range_usize(48);
        let bars: Vec<displacement::OhlcBar> = (0..n).map(|_| {
            if rng.next_u64() % 8 == 0 {
                displacement::OhlcBar {
                    open: rng.pick_pathological_f64(),
                    high: rng.pick_pathological_f64(),
                    low:  rng.pick_pathological_f64(),
                    close: rng.pick_pathological_f64(),
                }
            } else {
                let h = rng.f64_range(50.0, 200.0);
                let l = rng.f64_range(50.0, h);
                let o = rng.f64_range(l, h);
                let c = rng.f64_range(l, h);
                displacement::OhlcBar { open: o, high: h, low: l, close: c }
            }
        }).collect();
        let atr: Vec<f64> = (0..n).map(|_| rng.pick_pathological_f64()).collect();
        let cfg = displacement::DisplacementConfig {
            min_atrs: rng.f64_range(-1.0, 5.0),
            min_close_position: rng.f64_range(-0.5, 1.5),
        };
        let r = displacement::detect(&bars, &atr, &cfg);
        for e in &r.events {
            assert!(e.bar_index < n);
            assert!(e.body_atrs.is_finite(), "iter {it} body_atrs");
            assert!(e.close_position.is_finite(), "iter {it} close_pos");
        }
    }
}

// ---------------------------------------------------------------------------
// fair_value_gap
// ---------------------------------------------------------------------------
#[test]
fn fuzz_fair_value_gap() {
    let mut rng = Lcg::new(0x_F4_06_AB);
    for it in 0..ITERS {
        let n = rng.range_usize(48);
        let bars: Vec<fair_value_gap::OhlcBar> = (0..n).map(|_| {
            let h = rng.f64_range(50.0, 200.0);
            let l = rng.f64_range(50.0, h);
            let c = rng.f64_range(l, h);
            fair_value_gap::OhlcBar { high: h, low: l, close: c }
        }).collect();
        let r = fair_value_gap::detect(&bars);
        for g in &r.gaps {
            assert!(g.formed_at < n, "iter {it}");
            assert!(g.gap_high >= g.gap_low,
                "iter {it} gap inverted: high={} low={}", g.gap_high, g.gap_low);
            if let Some(f) = g.filled_at {
                assert!(f < n);
                assert!(f > g.formed_at, "iter {it} filled before formed");
            }
        }
        for &i in &r.open_gaps {
            assert!(i < r.gaps.len());
            assert!(r.gaps[i].filled_at.is_none());
        }
    }
}

// ---------------------------------------------------------------------------
// vsa
// ---------------------------------------------------------------------------
#[test]
fn fuzz_vsa() {
    let mut rng = Lcg::new(0x_DE_AD_F0);
    for it in 0..ITERS {
        let n = rng.range_usize(64);
        let bars: Vec<vsa::VsaBar> = (0..n).map(|_| {
            let h = rng.f64_range(50.0, 200.0);
            let l = rng.f64_range(50.0, h);
            let c = rng.f64_range(l, h);
            let o = rng.f64_range(l, h);
            let v = rng.f64_range(0.0, 1_000_000.0);
            vsa::VsaBar { open: o, high: h, low: l, close: c, volume: v }
        }).collect();
        let avg_volume: Vec<f64> = (0..n).map(|_| rng.f64_range(0.0, 500_000.0)).collect();
        let r = vsa::classify(&bars, &avg_volume);
        assert_eq!(r.n_events, r.events.len(), "iter {it}");
        for e in &r.events {
            assert!(e.bar_index < n);
            assert!(e.spread.is_finite() && e.spread >= 0.0, "iter {it} spread {}", e.spread);
            assert!(e.volume_ratio.is_finite() && e.volume_ratio >= 0.0,
                "iter {it} vol_ratio {}", e.volume_ratio);
            assert!(e.close_position.is_finite(), "iter {it} close_pos");
            // close_position should be in [0, 1] for valid bars (close in [low, high]).
            // Bars we generate always satisfy this.
            assert!((0.0..=1.0 + 1e-9).contains(&e.close_position),
                "iter {it} close_position out of range: {}", e.close_position);
        }
    }
}

// ---------------------------------------------------------------------------
// wyckoff
// ---------------------------------------------------------------------------
#[test]
fn fuzz_wyckoff() {
    let mut rng = Lcg::new(0x_4_7_C_0_FF);
    for it in 0..ITERS {
        let n = rng.range_usize(80);
        let closes: Vec<f64> = (0..n).map(|_| rng.f64_range(50.0, 200.0)).collect();
        let cfg = wyckoff::WyckoffConfig {
            lookback: match rng.next_u64() % 10 {
                0 => 0,
                1 => usize::MAX,
                _ => rng.range_usize(50),
            },
            flat_slope_pct: rng.f64_range(-0.01, 0.05),
            tight_range_pct: rng.f64_range(-0.01, 0.5),
        };
        let r = wyckoff::classify(&closes, &cfg);
        assert!(r.slope_pct.is_finite(), "iter {it}");
        assert!(r.range_pct.is_finite(), "iter {it}");
        assert!(r.price_position_in_range.is_finite(), "iter {it}");
    }
}

// ---------------------------------------------------------------------------
// stop_hunt
// ---------------------------------------------------------------------------
#[test]
fn fuzz_stop_hunt() {
    let mut rng = Lcg::new(0x_5_70_9_4A);
    for it in 0..ITERS {
        let n = rng.range_usize(64);
        let bars: Vec<stop_hunt::OhlcBar> = (0..n).map(|_| {
            let h = rng.f64_range(50.0, 200.0);
            let l = rng.f64_range(50.0, h);
            let c = rng.f64_range(l, h);
            stop_hunt::OhlcBar { high: h, low: l, close: c }
        }).collect();
        let cfg = stop_hunt::StopHuntConfig {
            lookback: match rng.next_u64() % 20 {
                0 => usize::MAX,
                1 => 0,
                _ => rng.range_usize(15),
            },
            min_pierce: rng.f64_range(-1.0, 5.0),
            min_reversal_pct: rng.f64_range(-0.5, 1.5),
        };
        let r = stop_hunt::detect(&bars, &cfg);
        for e in &r.events {
            assert!(e.bar_index < n, "iter {it}");
            assert!(e.reversal_pct.is_finite() && (0.0..=1.0 + 1e-9).contains(&e.reversal_pct),
                "iter {it} reversal_pct out of range: {}", e.reversal_pct);
        }
    }
}

// ---------------------------------------------------------------------------
// range_expansion
// ---------------------------------------------------------------------------
#[test]
fn fuzz_range_expansion() {
    let mut rng = Lcg::new(0x_4A_6_8E_C);
    for it in 0..ITERS {
        let n = rng.range_usize(64);
        let bars: Vec<range_expansion::OhlcBar> = (0..n).map(|_| {
            let h = rng.f64_range(50.0, 200.0);
            let l = rng.f64_range(50.0, h);
            let c = rng.f64_range(l, h);
            range_expansion::OhlcBar { high: h, low: l, close: c }
        }).collect();
        let atr: Vec<f64> = (0..n).map(|_| rng.f64_range(0.1, 10.0)).collect();
        let cfg = range_expansion::ExpansionConfig {
            lookback: match rng.next_u64() % 20 {
                0 => usize::MAX,
                1 => 0,
                _ => rng.range_usize(10),
            },
            min_expansion_atrs: rng.f64_range(-1.0, 5.0),
            prior_atr_max: rng.f64_range(-0.5, 2.0),
        };
        let r = range_expansion::detect(&bars, &atr, &cfg);
        for e in &r.events {
            assert!(e.bar_index < n, "iter {it}");
            assert!(e.range_atrs.is_finite(), "iter {it}");
        }
    }
}

// ---------------------------------------------------------------------------
// cusum
// ---------------------------------------------------------------------------
#[test]
fn fuzz_cusum() {
    let mut rng = Lcg::new(0x_C_5_88_AA);
    for it in 0..ITERS {
        let n = rng.range_usize(128);
        let series: Vec<f64> = (0..n).map(|_| rng.pick_pathological_f64()).collect();
        let cfg = cusum::CusumConfig {
            reference_mean: rng.f64_range(-10.0, 10.0),
            reference_stdev: rng.f64_range(-1.0, 5.0),
            threshold_stdevs: rng.f64_range(0.0, 10.0),
            slack: rng.f64_range(-1.0, 2.0),
        };
        let r = cusum::detect(&series, &cfg);
        for e in &r.events {
            assert!(e.bar_index < n, "iter {it}");
        }
    }
}

// ---------------------------------------------------------------------------
// volume_burst
// ---------------------------------------------------------------------------
#[test]
fn fuzz_volume_burst() {
    let mut rng = Lcg::new(0x_B0_1BA);
    for it in 0..ITERS {
        let n = rng.range_usize(64);
        let bars: Vec<volume_burst::VolumeBar> = (0..n).map(|_| {
            volume_burst::VolumeBar {
                volume: rng.f64_range(0.0, 100_000.0),
                close: rng.f64_range(10.0, 200.0),
            }
        }).collect();
        let cfg = volume_burst::BurstConfig {
            lookback: match rng.next_u64() % 12 {
                0 => 0,
                1 => usize::MAX,
                _ => rng.range_usize(20),
            },
            min_ratio: rng.f64_range(-1.0, 10.0),
        };
        let r = volume_burst::detect(&bars, &cfg);
        for e in &r.events {
            assert!(e.bar_index < n, "iter {it}");
            assert!(e.ratio.is_finite(), "iter {it} ratio {}", e.ratio);
            assert!(e.avg_volume.is_finite(), "iter {it} avg");
        }
        assert!(r.max_ratio.is_finite() || r.max_ratio == 0.0);
    }
}

// ---------------------------------------------------------------------------
// coppock
// ---------------------------------------------------------------------------
#[test]
fn fuzz_coppock() {
    let mut rng = Lcg::new(0x_C0_99_AC);
    for it in 0..ITERS {
        let n = rng.range_usize(80);
        // Mostly-positive prices; occasionally inject zeros to exercise the
        // (price[i]-price[i-roc])/price[i-roc] division-by-zero risk.
        let closes: Vec<f64> = (0..n).map(|_| {
            if rng.next_u64() % 10 == 0 { 0.0 } else { rng.f64_range(10.0, 200.0) }
        }).collect();
        let roc1 = match rng.next_u64() % 10 {
            0 => 0,
            1 => usize::MAX,
            _ => rng.range_usize(15) + 1,
        };
        let roc2 = match rng.next_u64() % 10 {
            0 => 0,
            1 => usize::MAX,
            _ => rng.range_usize(15) + 1,
        };
        let wma = match rng.next_u64() % 10 {
            0 => 0,
            1 => usize::MAX,
            _ => rng.range_usize(15) + 1,
        };
        let out = coppock::compute(&closes, roc1, roc2, wma);
        assert_eq!(out.len(), n, "iter {it}");
        for v in &out {
            assert!(v.is_finite(), "iter {it} coppock {} (price hit 0 → div-by-zero)", v);
        }
    }
}

// ---------------------------------------------------------------------------
// three_bar_reversal
// ---------------------------------------------------------------------------
#[test]
fn fuzz_three_bar_reversal() {
    let mut rng = Lcg::new(0x_38_AA_4A);
    for it in 0..ITERS {
        let n = rng.range_usize(32);
        let bars: Vec<three_bar_reversal::OhlcBar> = (0..n).map(|_| {
            let h = rng.f64_range(50.0, 200.0);
            let l = rng.f64_range(50.0, h);
            let o = rng.f64_range(l, h);
            let c = rng.f64_range(l, h);
            three_bar_reversal::OhlcBar { open: o, high: h, low: l, close: c }
        }).collect();
        let r = three_bar_reversal::detect(&bars);
        for e in &r.events {
            assert!(e.bar_index < n, "iter {it}");
            assert!(e.bar1_open.is_finite() && e.bar3_close.is_finite(), "iter {it}");
        }
    }
}

// ---------------------------------------------------------------------------
// sortino — verify negative annualization no longer poisons output with NaN
// ---------------------------------------------------------------------------
#[test]
fn fuzz_sortino() {
    let mut rng = Lcg::new(0x_50_47_1A);
    for it in 0..ITERS {
        let n = rng.range_usize(64);
        let returns: Vec<f64> = (0..n).map(|_| rng.f64_range(-5.0, 5.0)).collect();
        let mar = rng.f64_range(-2.0, 2.0);
        // Include negatives — used to produce NaN via sqrt(-x).
        let annualization = rng.f64_range(-100.0, 252.0);
        let r = sortino::compute(&returns, mar, annualization);
        assert!(r.mean_return.is_finite(), "iter {it} mean");
        assert!(r.downside_deviation.is_finite() && r.downside_deviation >= 0.0,
            "iter {it} dd {}", r.downside_deviation);
        // Infinite sortino is acceptable (all-positive series); NaN is not.
        assert!(!r.sortino_ratio.is_nan(),
            "iter {it} NaN sortino with ann={annualization}");
    }
}

// ---------------------------------------------------------------------------
// sharpe_by_window — same negative-annualization concern
// ---------------------------------------------------------------------------
#[test]
fn fuzz_sharpe_by_window() {
    let mut rng = Lcg::new(0x_5_A_E_8E);
    for it in 0..ITERS / 4 {
        let n = rng.range_usize(50);
        let returns: Vec<sharpe_by_window::TradeReturn> = (0..n).map(|i| {
            sharpe_by_window::TradeReturn {
                when: Utc.with_ymd_and_hms(2026, 1, 1, (i % 24) as u32, 0, 0).unwrap(),
                r: rng.f64_range(-5.0, 5.0),
            }
        }).collect();
        let annualization = rng.f64_range(-100.0, 252.0);
        let out = sharpe_by_window::by(&returns, sharpe_by_window::Bucket::HourOfDay, annualization);
        for s in &out {
            assert!(s.mean_r.is_finite(), "iter {it} mean");
            assert!(s.stdev_r.is_finite() && s.stdev_r >= 0.0, "iter {it} stdev");
            assert!(!s.sharpe.is_nan(), "iter {it} NaN sharpe");
        }
    }
}

// ---------------------------------------------------------------------------
// treynor — information_ratio annualization
// ---------------------------------------------------------------------------
#[test]
fn fuzz_treynor_information_ratio() {
    let mut rng = Lcg::new(0x_4_5E_88);
    for it in 0..ITERS {
        let n = rng.range_usize(48);
        let p: Vec<f64> = (0..n).map(|_| rng.f64_range(-1.0, 1.0)).collect();
        let b: Vec<f64> = (0..n).map(|_| rng.f64_range(-1.0, 1.0)).collect();
        let annualization = rng.f64_range(-100.0, 252.0);
        if let Some(r) = treynor::information_ratio(&p, &b, annualization) {
            assert!(r.mean_active_return.is_finite(), "iter {it} mean");
            assert!(r.tracking_error.is_finite() && r.tracking_error >= 0.0,
                "iter {it} te");
            assert!(!r.information_ratio.is_nan(),
                "iter {it} NaN IR with ann={annualization}");
        }
    }
}

// ---------------------------------------------------------------------------
// premium_discount
// ---------------------------------------------------------------------------
#[test]
fn fuzz_premium_discount() {
    let mut rng = Lcg::new(0x_8E_4D_77);
    for it in 0..ITERS {
        let lo = rng.pick_pathological_f64();
        let hi = rng.pick_pathological_f64();
        let price = rng.pick_pathological_f64();
        let trend = match rng.next_u64() % 3 {
            0 => premium_discount::TrendBias::Up,
            1 => premium_discount::TrendBias::Down,
            _ => premium_discount::TrendBias::Neutral,
        };
        let r = premium_discount::classify(hi, lo, price, trend);
        // midpoint may be NaN/Inf if the early-return missed (e.g. only one
        // side non-finite). Verify the early-return guard catches all bad
        // ranges so the report is always trustworthy.
        if r.note != "invalid range" {
            assert!(r.midpoint.is_finite(), "iter {it} midpoint non-finite but range not flagged");
        }
    }
}

// ---------------------------------------------------------------------------
// range_contraction
// ---------------------------------------------------------------------------
#[test]
fn fuzz_range_contraction() {
    let mut rng = Lcg::new(0x_4_C_0_A_77);
    for it in 0..ITERS {
        let n = rng.range_usize(64);
        let bars: Vec<range_contraction::OhlcBar> = (0..n).map(|_| {
            let h = rng.f64_range(50.0, 200.0);
            let l = rng.f64_range(50.0, h);
            let c = rng.f64_range(l, h);
            range_contraction::OhlcBar { high: h, low: l, close: c }
        }).collect();
        let r = range_contraction::detect(&bars);
        for h in &r.hits {
            assert!(h.bar_index < n, "iter {it}");
            assert!(h.range.is_finite() && h.range >= 0.0, "iter {it} range {}", h.range);
        }
    }
}

// ---------------------------------------------------------------------------
// volatility_stop — fuzz the just-added mismatched-ATR length panic guard
// ---------------------------------------------------------------------------
#[test]
fn fuzz_volatility_stop() {
    let mut rng = Lcg::new(0x_5_70_44_70);
    for it in 0..ITERS {
        let n = rng.range_usize(64);
        let bars: Vec<volatility_stop::Bar> = (0..n).map(|_| {
            let h = rng.f64_range(50.0, 200.0);
            let l = rng.f64_range(50.0, h);
            let c = rng.f64_range(l, h);
            volatility_stop::Bar { high: h, low: l, close: c }
        }).collect();
        // Deliberately desync ATR length from bars half the time to make
        // sure the new guard catches mismatched arrays without panicking.
        let atr: Vec<f64> = if rng.next_u64() & 1 == 0 {
            (0..n).map(|_| rng.f64_range(0.1, 5.0)).collect()
        } else {
            (0..rng.range_usize(n.max(1) * 2)).map(|_| rng.f64_range(0.1, 5.0)).collect()
        };
        let cfg = volatility_stop::StopConfig {
            lookback: match rng.next_u64() % 12 {
                0 => 0,
                1 => usize::MAX,
                _ => rng.range_usize(20),
            },
            atr_multiplier: rng.f64_range(-1.0, 5.0),
        };
        let side = if rng.next_u64() & 1 == 0 { TradeSide::Long } else { TradeSide::Short };
        let _ = volatility_stop::chandelier(&bars, &atr, side, &cfg);
        let _ = volatility_stop::vol_stop_close(&bars, &atr, side, &cfg);
        let _ = it;
    }
}

// ---------------------------------------------------------------------------
// anchored_vwap
// ---------------------------------------------------------------------------
#[test]
fn fuzz_anchored_vwap() {
    let mut rng = Lcg::new(0x_A_57_D7);
    for it in 0..ITERS {
        let n = rng.range_usize(64);
        let bars: Vec<anchored_vwap::Bar> = (0..n).map(|_| {
            anchored_vwap::Bar {
                typical: rng.f64_range(10.0, 500.0),
                volume: rng.f64_range(-1000.0, 100_000.0),
            }
        }).collect();
        let anchor = match rng.next_u64() % 10 {
            0 => usize::MAX,
            1 => 0,
            _ => rng.range_usize(n.max(1) * 2),
        };
        let out = anchored_vwap::compute(&bars, anchor);
        assert_eq!(out.len(), bars.len(), "iter {it}");
        for p in &out {
            assert!(p.vwap.is_finite(), "iter {it} vwap {}", p.vwap);
            assert!(p.upper_1sd.is_finite(), "iter {it} upper");
            assert!(p.lower_1sd.is_finite(), "iter {it} lower");
        }
    }
}

// ---------------------------------------------------------------------------
// awesome_oscillator
// ---------------------------------------------------------------------------
#[test]
fn fuzz_awesome_oscillator() {
    let mut rng = Lcg::new(0x_A_05_C);
    for it in 0..ITERS {
        let n = rng.range_usize(80);
        let bars: Vec<awesome_oscillator::Bar> = (0..n).map(|_| {
            let h = rng.f64_range(50.0, 200.0);
            let l = rng.f64_range(50.0, h);
            awesome_oscillator::Bar { high: h, low: l }
        }).collect();
        let short_p = match rng.next_u64() % 8 {
            0 => 0, 1 => usize::MAX, _ => rng.range_usize(10) + 1,
        };
        let long_p = match rng.next_u64() % 8 {
            0 => 0, 1 => usize::MAX, _ => rng.range_usize(40) + 1,
        };
        let out = awesome_oscillator::compute(&bars, short_p, long_p);
        assert_eq!(out.len(), n, "iter {it}");
        for v in &out {
            assert!(v.is_finite(), "iter {it} ao {}", v);
        }
    }
}

// ---------------------------------------------------------------------------
// opening_range — usize::MAX opening_bars + ATR
// ---------------------------------------------------------------------------
#[test]
fn fuzz_opening_range() {
    let mut rng = Lcg::new(0x_0_5B_07);
    for it in 0..ITERS {
        let n = rng.range_usize(64);
        let bars: Vec<opening_range::OhlcBar> = (0..n).map(|_| {
            let h = rng.f64_range(50.0, 200.0);
            let l = rng.f64_range(50.0, h);
            let c = rng.f64_range(l, h);
            opening_range::OhlcBar { high: h, low: l, close: c }
        }).collect();
        let cfg = opening_range::OrbConfig {
            opening_bars: match rng.next_u64() % 12 {
                0 => 0, 1 => usize::MAX, _ => rng.range_usize(20),
            },
            atr: rng.pick_pathological_f64(),
            close_only: rng.next_u64() & 1 == 0,
        };
        let r = opening_range::detect(&bars, &cfg);
        if let Some(b) = r.upper_break { assert!(b.bar_index < n, "iter {it}"); }
        if let Some(b) = r.lower_break { assert!(b.bar_index < n, "iter {it}"); }
    }
}

// ---------------------------------------------------------------------------
// supertrend — has atr.len() guard; verify no leaks
// ---------------------------------------------------------------------------
#[test]
fn fuzz_supertrend() {
    let mut rng = Lcg::new(0x_5_F0_E4);
    for it in 0..ITERS {
        let n = rng.range_usize(64);
        let bars: Vec<supertrend::Bar> = (0..n).map(|_| {
            let h = rng.f64_range(50.0, 200.0);
            let l = rng.f64_range(50.0, h);
            let c = rng.f64_range(l, h);
            supertrend::Bar { high: h, low: l, close: c }
        }).collect();
        let atr: Vec<f64> = if rng.next_u64() & 1 == 0 {
            (0..n).map(|_| rng.f64_range(0.1, 5.0)).collect()
        } else {
            (0..rng.range_usize(n.max(1) * 2)).map(|_| rng.f64_range(0.1, 5.0)).collect()
        };
        let mult = rng.f64_range(-1.0, 5.0);
        let out = supertrend::compute(&bars, &atr, mult);
        assert_eq!(out.len(), n, "iter {it}");
        for p in &out {
            assert!(p.upper_band.is_finite(), "iter {it} upper");
            assert!(p.lower_band.is_finite(), "iter {it} lower");
            assert!(p.super_trend.is_finite(), "iter {it} st");
        }
    }
}

// ---------------------------------------------------------------------------
// round_levels
// ---------------------------------------------------------------------------
#[test]
fn fuzz_round_levels() {
    let mut rng = Lcg::new(0x_4_0_F1_5);
    for it in 0..ITERS {
        let cur = rng.pick_pathological_f64();
        let atr = if rng.next_u64() & 1 == 0 { None } else { Some(rng.pick_pathological_f64()) };
        let cfg = round_levels::LevelsConfig {
            window: rng.f64_range(-100.0, 200_000.0),
            min_weight: match rng.next_u64() % 3 {
                0 => round_levels::LevelWeight::Major,
                1 => round_levels::LevelWeight::Medium,
                _ => round_levels::LevelWeight::Minor,
            },
        };
        let r = round_levels::detect(cur, atr, &cfg);
        // No panic; levels must be finite when present.
        for l in &r.levels {
            assert!(l.price.is_finite(), "iter {it} price");
            assert!(l.distance.is_finite(), "iter {it} distance");
        }
    }
}

// ---------------------------------------------------------------------------
// bond_duration — negative-yield + extreme YTM safety
// ---------------------------------------------------------------------------
#[test]
fn fuzz_bond_duration() {
    let mut rng = Lcg::new(0x_B_0_07_D);
    for it in 0..ITERS {
        let n = rng.range_usize(20);
        let cfs: Vec<bond_duration::CashFlow> = (0..n).map(|i| {
            bond_duration::CashFlow {
                time_years: (i as f64 + 1.0) + rng.f64_range(-0.5, 0.5),
                amount: rng.f64_range(1.0, 200.0),
            }
        }).collect();
        // Include adversarial yields: -m (which would have given NaN via
        // 0^positive before the fix), extreme positive, etc.
        let ytm = match rng.next_u64() % 8 {
            0 => -2.0,
            1 => -0.99,
            2 => 100.0,
            3 => 1e9,
            4 => 0.0,
            _ => rng.f64_range(-0.5, 0.20),
        };
        let m = match rng.next_u64() % 5 {
            0 => 0, 1 => 1, 2 => 2, 3 => 12, _ => 4,
        };
        let r = bond_duration::compute(&cfs, ytm, m);
        // After the guard, all numeric fields must be finite.
        assert!(r.price.is_finite(),
            "iter {it} price non-finite (ytm={ytm}, m={m}): {}", r.price);
        assert!(r.macaulay_duration.is_finite(),
            "iter {it} mac dur non-finite: {}", r.macaulay_duration);
        assert!(r.modified_duration.is_finite(),
            "iter {it} mod dur non-finite: {}", r.modified_duration);
    }
}

// ---------------------------------------------------------------------------
// indicators::adx — exercises the wilder_smooth saturating-add fix
// (adx forwards `period` to wilder_smooth; without the fix
// `period = usize::MAX` panicked on `values[1..=usize::MAX]`)
// ---------------------------------------------------------------------------
#[test]
fn fuzz_indicators_adx() {
    let mut rng = Lcg::new(0x_A_DC_8E);
    for it in 0..ITERS / 4 {
        let n = rng.range_usize(64);
        let highs: Vec<f64>  = (0..n).map(|_| rng.f64_range(50.0, 200.0)).collect();
        let lows: Vec<f64>   = (0..n).map(|i| highs[i] - rng.f64_range(0.0, 5.0)).collect();
        let closes: Vec<f64> = (0..n).map(|i| rng.f64_range(lows[i], highs[i])).collect();
        let period = match rng.next_u64() % 10 {
            0 => 0, 1 => usize::MAX, 2 => usize::MAX - 1, _ => rng.range_usize(20) + 1,
        };
        let r = indicators::adx(&highs, &lows, &closes, period);
        assert_eq!(r.adx.len(), n, "iter {it} adx len");
        for v in &r.adx { assert_finite_opt(v, "adx"); }
        for v in &r.plus_di { assert_finite_opt(v, "+di"); }
        for v in &r.minus_di { assert_finite_opt(v, "-di"); }
    }
}

// ---------------------------------------------------------------------------
// indicators::atr — verify the period+1 saturating guard
// ---------------------------------------------------------------------------
#[test]
fn fuzz_indicators_atr() {
    let mut rng = Lcg::new(0x_A7_8B);
    for it in 0..ITERS / 2 {
        let n = rng.range_usize(64);
        let highs: Vec<f64>  = (0..n).map(|_| rng.f64_range(50.0, 200.0)).collect();
        let lows: Vec<f64>   = (0..n).map(|i| highs[i] - rng.f64_range(0.0, 5.0)).collect();
        let closes: Vec<f64> = (0..n).map(|i| rng.f64_range(lows[i], highs[i])).collect();
        let period = match rng.next_u64() % 10 {
            0 => 0, 1 => usize::MAX, 2 => usize::MAX - 1, _ => rng.range_usize(20) + 1,
        };
        let out = indicators::atr(&highs, &lows, &closes, period);
        assert_eq!(out.len(), n, "iter {it}");
        for v in &out { assert_finite_opt(v, "atr"); }
    }
}

// ---------------------------------------------------------------------------
// vortex — n < period.saturating_add(1) guard
// ---------------------------------------------------------------------------
#[test]
fn fuzz_vortex() {
    let mut rng = Lcg::new(0x_4_07_AB);
    for it in 0..ITERS / 2 {
        let n = rng.range_usize(64);
        let bars: Vec<vortex::Bar> = (0..n).map(|_| {
            let h = rng.f64_range(50.0, 200.0);
            let l = rng.f64_range(50.0, h);
            let c = rng.f64_range(l, h);
            vortex::Bar { high: h, low: l, close: c }
        }).collect();
        let period = match rng.next_u64() % 10 {
            0 => 0, 1 => usize::MAX, 2 => usize::MAX - 1, _ => rng.range_usize(20) + 1,
        };
        let out = vortex::compute(&bars, period);
        assert_eq!(out.len(), n, "iter {it}");
        for p in &out {
            assert!(p.vi_plus.is_finite(), "iter {it} vi+");
            assert!(p.vi_minus.is_finite(), "iter {it} vi-");
        }
    }
}

// ---------------------------------------------------------------------------
// aroon — n < period.saturating_add(1) guard
// ---------------------------------------------------------------------------
#[test]
fn fuzz_aroon() {
    let mut rng = Lcg::new(0x_AA_5E_E);
    for it in 0..ITERS / 2 {
        let n = rng.range_usize(64);
        let bars: Vec<aroon::Bar> = (0..n).map(|_| {
            let h = rng.f64_range(50.0, 200.0);
            let l = rng.f64_range(50.0, h);
            aroon::Bar { high: h, low: l }
        }).collect();
        let period = match rng.next_u64() % 10 {
            0 => 0, 1 => usize::MAX, 2 => usize::MAX - 1, _ => rng.range_usize(20) + 1,
        };
        let out = aroon::compute(&bars, period);
        assert_eq!(out.len(), n, "iter {it}");
        for p in &out {
            assert!(p.up.is_finite(), "iter {it} up");
            assert!(p.down.is_finite(), "iter {it} down");
            assert!(p.oscillator.is_finite(), "iter {it} osc");
        }
    }
}

// ---------------------------------------------------------------------------
// mass_index — sum_period==0 and ema_period==0 guards
// ---------------------------------------------------------------------------
#[test]
fn fuzz_mass_index() {
    let mut rng = Lcg::new(0x_4_4_55);
    for it in 0..ITERS / 4 {
        let n = rng.range_usize(64);
        let highs: Vec<f64> = (0..n).map(|_| rng.f64_range(50.0, 200.0)).collect();
        let lows: Vec<f64>  = (0..n).map(|i| highs[i] - rng.f64_range(0.0, 5.0)).collect();
        let ema_p = match rng.next_u64() % 5 {
            0 => 0, 1 => usize::MAX, _ => rng.range_usize(20) + 1,
        };
        let sum_p = match rng.next_u64() % 5 {
            0 => 0, 1 => usize::MAX, _ => rng.range_usize(30) + 1,
        };
        let out = mass_index::compute(&highs, &lows, ema_p, sum_p);
        assert_eq!(out.len(), n, "iter {it}");
        for v in &out { assert!(v.is_finite(), "iter {it} mi {}", v); }
    }
}

// ---------------------------------------------------------------------------
// stochastic — verify the k_period + d_period overflow guard
// ---------------------------------------------------------------------------
#[test]
fn fuzz_stochastic_compute() {
    let mut rng = Lcg::new(0x_57_0C_C);
    for it in 0..ITERS {
        let n = rng.range_usize(64);
        let bars: Vec<stochastic::Bar> = (0..n).map(|_| {
            let h = rng.f64_range(50.0, 200.0);
            let l = rng.f64_range(50.0, h);
            let c = rng.f64_range(l, h);
            stochastic::Bar { high: h, low: l, close: c }
        }).collect();
        // Adversarial periods to exercise the saturating_add/saturating_sub fix.
        let k_period = match rng.next_u64() % 10 {
            0 => 0, 1 => usize::MAX, 2 => usize::MAX - 1, _ => rng.range_usize(15) + 1,
        };
        let d_period = match rng.next_u64() % 10 {
            0 => 0, 1 => usize::MAX, 2 => usize::MAX - 1, _ => rng.range_usize(10) + 1,
        };
        let out = stochastic::compute(&bars, k_period, d_period);
        assert_eq!(out.len(), n, "iter {it} length invariant");
        for p in &out {
            assert!(p.fast_k.is_finite() || p.fast_k == 0.0, "iter {it} fast_k");
            assert!(p.fast_d.is_finite() || p.fast_d == 0.0, "iter {it} fast_d");
            assert!(p.slow_k.is_finite() || p.slow_k == 0.0, "iter {it} slow_k");
            assert!(p.slow_d.is_finite() || p.slow_d == 0.0, "iter {it} slow_d");
        }
    }
}

// ---------------------------------------------------------------------------
// Batch fuzz coverage for the 10 newly-added indicators
// ---------------------------------------------------------------------------
#[test]
fn fuzz_new_moving_averages() {
    let mut rng = Lcg::new(0x_4A_4A_4A);
    for it in 0..ITERS / 2 {
        let n = rng.range_usize(80);
        let closes: Vec<f64> = (0..n).map(|_| rng.f64_range(10.0, 500.0)).collect();
        let period = match rng.next_u64() % 10 {
            0 => 0, 1 => 1, 2 => usize::MAX, _ => rng.range_usize(20) + 2,
        };
        for v in &hull_ma::compute(&closes, period) { assert_finite_opt(v, "hull"); }
        for v in &tema::compute(&closes, period)    { assert_finite_opt(v, "tema"); }
        for v in &dema::compute(&closes, period)    { assert_finite_opt(v, "dema"); }
        let _ = it;
    }
}

#[test]
fn fuzz_kama() {
    let mut rng = Lcg::new(0x_4A_AA);
    for it in 0..ITERS / 2 {
        let n = rng.range_usize(80);
        let closes: Vec<f64> = (0..n).map(|_| rng.f64_range(10.0, 500.0)).collect();
        let er = match rng.next_u64() % 8 {
            0 => 0, 1 => usize::MAX, _ => rng.range_usize(15) + 1,
        };
        let fp = match rng.next_u64() % 8 {
            0 => 0, 1 => usize::MAX, _ => rng.range_usize(5) + 1,
        };
        let sp = match rng.next_u64() % 8 {
            0 => 0, 1 => usize::MAX, _ => rng.range_usize(40) + 1,
        };
        let out = kama::compute(&closes, er, fp, sp);
        for v in &out { assert_finite_opt(v, "kama"); }
        let _ = it;
    }
}

#[test]
fn fuzz_stoch_rsi() {
    let mut rng = Lcg::new(0x_57_4F_45);
    for it in 0..ITERS / 4 {
        let n = rng.range_usize(80);
        let closes: Vec<f64> = (0..n).map(|_| rng.f64_range(10.0, 500.0)).collect();
        let rp = match rng.next_u64() % 8 { 0 => 0, 1 => usize::MAX, _ => rng.range_usize(20)+1 };
        let sp = match rng.next_u64() % 8 { 0 => 0, 1 => usize::MAX, _ => rng.range_usize(20)+1 };
        let sk = match rng.next_u64() % 8 { 0 => 0, _ => rng.range_usize(5) + 1 };
        let sd = match rng.next_u64() % 8 { 0 => 0, _ => rng.range_usize(5) + 1 };
        let r = stoch_rsi::compute(&closes, rp, sp, sk, sd);
        for v in &r.raw { assert_finite_opt(v, "raw"); }
        for v in &r.k   { assert_finite_opt(v, "k"); }
        for v in &r.d   { assert_finite_opt(v, "d"); }
        let _ = it;
    }
}

#[test]
fn fuzz_ultimate_oscillator() {
    let mut rng = Lcg::new(0x_007_F1);
    for it in 0..ITERS / 4 {
        let n = rng.range_usize(80);
        let bars: Vec<ultimate_oscillator::OhlcBar> = (0..n).map(|_| {
            let h = rng.f64_range(50.0, 200.0);
            let l = rng.f64_range(50.0, h);
            let c = rng.f64_range(l, h);
            ultimate_oscillator::OhlcBar { high: h, low: l, close: c }
        }).collect();
        let s = match rng.next_u64() % 8 { 0 => 0, 1 => usize::MAX, _ => rng.range_usize(15)+1 };
        let m = match rng.next_u64() % 8 { 0 => 0, 1 => usize::MAX, _ => rng.range_usize(20)+1 };
        let l = match rng.next_u64() % 8 { 0 => 0, 1 => usize::MAX, _ => rng.range_usize(40)+1 };
        let out = ultimate_oscillator::compute(&bars, s, m, l);
        for v in &out { assert_finite_opt(v, "uo"); }
        let _ = it;
    }
}

#[test]
fn fuzz_tsi() {
    let mut rng = Lcg::new(0x_751_E);
    for it in 0..ITERS / 4 {
        let n = rng.range_usize(80);
        let closes: Vec<f64> = (0..n).map(|_| rng.f64_range(10.0, 500.0)).collect();
        let r = match rng.next_u64() % 8 { 0 => 0, 1 => usize::MAX, _ => rng.range_usize(40)+1 };
        let s = match rng.next_u64() % 8 { 0 => 0, 1 => usize::MAX, _ => rng.range_usize(20)+1 };
        let out = tsi::compute(&closes, r, s);
        for v in &out { assert_finite_opt(v, "tsi"); }
        let _ = it;
    }
}

#[test]
fn fuzz_chaikin_money_flow() {
    let mut rng = Lcg::new(0x_C_4_F_4);
    for it in 0..ITERS / 4 {
        let n = rng.range_usize(64);
        let bars: Vec<chaikin_money_flow::Bar> = (0..n).map(|_| {
            let h = rng.f64_range(50.0, 200.0);
            let l = rng.f64_range(50.0, h);
            let c = rng.f64_range(l, h);
            chaikin_money_flow::Bar { high: h, low: l, close: c, volume: rng.f64_range(0.0, 1e6) }
        }).collect();
        let period = match rng.next_u64() % 8 {
            0 => 0, 1 => usize::MAX, _ => rng.range_usize(30) + 1,
        };
        let out = chaikin_money_flow::compute(&bars, period);
        for x in out.iter().flatten() {
            assert!(x.is_finite() && (-1.0 - 1e-9..=1.0 + 1e-9).contains(x),
                "iter {it} cmf out of [-1,1]: {x}");
        }
    }
}

#[test]
fn fuzz_ppo() {
    let mut rng = Lcg::new(0x_88_0F);
    for it in 0..ITERS / 4 {
        let n = rng.range_usize(120);
        let closes: Vec<f64> = (0..n).map(|_| rng.f64_range(10.0, 500.0)).collect();
        let f = match rng.next_u64() % 8 { 0 => 0, 1 => usize::MAX, _ => rng.range_usize(15)+1 };
        let s = match rng.next_u64() % 8 { 0 => 0, 1 => usize::MAX, _ => rng.range_usize(40)+1 };
        let g = match rng.next_u64() % 8 { 0 => 0, _ => rng.range_usize(15)+1 };
        let r = ppo::compute(&closes, f, s, g);
        for v in &r.line      { assert_finite_opt(v, "ppo"); }
        for v in &r.signal    { assert_finite_opt(v, "ppo signal"); }
        for v in &r.histogram { assert_finite_opt(v, "ppo hist"); }
        let _ = it;
    }
}

#[test]
fn fuzz_elder_ray() {
    let mut rng = Lcg::new(0x_E_1_DE);
    for it in 0..ITERS / 4 {
        let n = rng.range_usize(80);
        let bars: Vec<elder_ray::Bar> = (0..n).map(|_| {
            let h = rng.f64_range(50.0, 200.0);
            let l = rng.f64_range(50.0, h);
            let c = rng.f64_range(l, h);
            elder_ray::Bar { high: h, low: l, close: c }
        }).collect();
        let period = match rng.next_u64() % 8 {
            0 => 0, 1 => usize::MAX, _ => rng.range_usize(30) + 1,
        };
        let r = elder_ray::compute(&bars, period);
        for v in &r.bull_power { assert_finite_opt(v, "bull"); }
        for v in &r.bear_power { assert_finite_opt(v, "bear"); }
        let _ = it;
    }
}

// ---------------------------------------------------------------------------
// indicators::stochastic + adx + rsi + ema + sma
// ---------------------------------------------------------------------------
#[test]
fn fuzz_indicators_core() {
    let mut rng = Lcg::new(0x_1_5DC4);
    for it in 0..ITERS / 4 {
        let n = rng.range_usize(64);
        let closes: Vec<f64> = (0..n).map(|_| rng.f64_range(10.0, 200.0)).collect();
        let highs: Vec<f64> = (0..n).map(|i| closes[i] + rng.f64_range(0.0, 5.0)).collect();
        let lows: Vec<f64>  = (0..n).map(|i| closes[i] - rng.f64_range(0.0, 5.0)).collect();
        let period = match rng.next_u64() % 10 {
            0 => 0,
            1 => usize::MAX,
            _ => rng.range_usize(20),
        };
        // sma
        let sma_out = indicators::sma(&closes, period);
        assert_eq!(sma_out.len(), n, "iter {it} sma len");
        for v in &sma_out { assert_finite_opt(v, "sma"); }
        // ema
        let ema_out = indicators::ema(&closes, period);
        assert_eq!(ema_out.len(), n, "iter {it} ema len");
        for v in &ema_out { assert_finite_opt(v, "ema"); }
        // rsi
        let rsi_out = indicators::rsi(&closes, period);
        assert_eq!(rsi_out.len(), n, "iter {it} rsi len");
        for v in &rsi_out { assert_finite_opt(v, "rsi"); }
        // stochastic
        let stoch = indicators::stochastic(&highs, &lows, &closes, period.max(2), 3);
        assert_eq!(stoch.k.len(), n, "iter {it} k len");
        assert_eq!(stoch.d.len(), n, "iter {it} d len");
        for v in &stoch.k { assert_finite_opt(v, "k"); }
        for v in &stoch.d { assert_finite_opt(v, "d"); }
    }
}
