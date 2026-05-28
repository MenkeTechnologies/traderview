//! Final batch: discipline circuit-breakers, margin calculators, calendar
//! helpers, sentiment indicators, and execution-TCA endpoints.
//!
//! Provenance from this round's research:
//!   - **tastytrade** — vertical-spread margin (Reg-T short-option model).
//!   - **MT5** — strategy correlation matrix (Expert Advisor portfolio fit).
//!   - **eToro / Robinhood** — recurring-investment scaffolding via the
//!     position_irr (XIRR) calculator for SIPs and DRIP analysis.
//!   - **TradeStation** — TWAP execution-quality (institutional TCA).

use crate::auth::AuthUser;
use crate::error::ApiError;
use crate::state::AppState;
use axum::routing::post;
use axum::{Json, Router};
use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use traderview_core::{
    abandoned_baby, abc_pattern, abcd_pattern, absorption_detector,
    acceleration_deceleration,
    accumulation_distribution_line,
    accumulation_distribution_oscillator, accumulation_swing_index, accumulation_volume_pattern,
    active_share,
    adf_standalone,
    alligator, alma_legoux, almgren_chriss, alphatrend,
    american_binomial, amihud_illiquidity, anchored_momentum, andrews_pitchfork,
    anderson_darling_normality, anti_setup,
    arch_lm_test, arima_111, arms_high_low_index, arms_index, aroon_indicator, asian_option,
    asset_swap_spread, atr_channel, atr_trailing_stop,
    atr_cone, autocorrelation_function,
    bachelier, backtest_sweep, balance_of_power, barrier_option, bartlett_variance_test,
    bat_pattern, bayesian_change_point_detector, belkhayate_timing, belt_hold_pattern,
    bermudan_binomial,
    beta_shrinkage, bid_ask_volume_ratio, bipower_variation,
    black76, black_litterman, breeden_litzenberger,
    block_bootstrap, bollinger_band_distance, bollinger_band_width,
    bollinger_bandwidth_percentile,
    bollinger_oscillators, bollinger_percent_b,
    bollinger_squeeze, bond_convexity, bootstrap_pnl, borrow_rate_indicator,
    box_spread,
    break_of_structure, breadth_lines, breadth_thrust, breakaway_pattern, breakeven_inflation,
    breakout_52w_scanner,
    breusch_godfrey, breusch_pagan_test, brier_score, bump_and_run, burke_ratio,
    breakout_detector, bressert_dss, brinson_attribution, butterfly_pattern, butterfly_spread,
    calendar_spread, calmar_ratio,
    camarilla_pivots,
    candle_patterns, candle_strength_index, caplet_black76, carry_roll_decomposition,
    cds_pricing, chaikin_oscillator,
    chande_dynamic_momentum_index, chande_kroll_stop, chande_momentum_oscillator,
    chande_trend_index, chande_volatility_index, chandelier_exit,
    change_of_character, chop_zone_indicator, choppiness, choppy_market_index, cholesky, chow_test, corwin_schultz_spread,
    chooser_option, cir, cliquet_option, cointegration, collar, component_var,
    composite_factor_scoring, compound_pivots,
    compound_option, conditional_drawdown, conditional_var,
    continuous_ranked_probability_score, convertible_bond, coppock_curve,
    cornish_fisher,
    cot_report, counter_attack_lines, crab_pattern, crossover, cross_currency_basis,
    cumulative_delta,
    cumulative_tick_trin, cumulative_tick_zscore, cumulative_volume_delta_zscore,
    cup_and_handle, cusum, cypher_pattern,
    daily_loss_limit, damiani_volatmeter, dark_cloud_piercing, dark_pool_index, darvas_box,
    day_of_week_seasonality,
    decile_long_short_signal,
    deflated_sharpe, demark_pivots, demarker_oscillator, depth_imbalance,
    developing_poc, developing_value_area, dividend_yield_curve,
    detrended_price_oscillator, detrended_synthetic_price, detrended_volatility_oscillator,
    dfa, diamond_pattern, diebold_mariano, digital_option,
    disparity_index, displacement, distance_correlation, divergence_detector, doji_variants,
    dollar_bar_chart, donchian_channels, donchian_oscillator,
    drawdown_throttle, dupire_local_volatility, dynamic_time_warping,
    earnings_calendar,
    earnings_revision_scanner, earnings_surprise_indicator, effective_spread, efficiency_ratio,
    ehlers_centered_smoothed_momentum, ehlers_correlation_trend_indicator,
    ehlers_decycler_oscillator, ehlers_fisher_cybernetic, ehlers_instant_trendline,
    ehlers_mama_fama, ehrlich_filter,
    eighty_twenty_setup, elder_safezone_stop,
    elder_thermometer, elliott_wave_oscillator, engulfing_pattern_scanner, equivolume_bars,
    empirical_distribution_function,
    engle_granger_2step,
    equal_levels, ergodic_oscillator, evt_value_at_risk, ewma_volatility,
    expectancy_per_trade, expected_calibration_error, expected_drawdown,
    expected_shortfall_contribution, factor_models, factor_neutralization,
    fama_french_3factor,
    fair_value_gap, fibonacci_extensions, fibonacci_pivots, fibonacci_retracements,
    fifty_two_week_high_low_scanner,
    finite_difference_option,
    five_o_pattern, floor_pivots,
    footprint_imbalance, fra,
    forward_start_option, fractional_brownian_motion_generator, frama_fractal,
    friedman_test,
    futures_roll, gain_pain_ratio, gain_to_pain_ratio, gamma_pin_zone, gamma_scalping_pnl,
    gann_fan, gann_high_low_activator, gann_swing_chart, gap_classifier, gap_option,
    gartley_pattern,
    garch_1_1, gap_fill_stats, gator_oscillator, gaussian_copula, gbm_path_simulator,
    gex_scanner, gjr_garch,
    goal_tracker,
    gonzalo_granger_decomposition, gpd_tail_fit, granger_causality,
    greeks_profile, guppy_mma,
    halt_resume_monitor, hampel_filter, hanging_man_shooting_star, har_volatility,
    harami_pattern, harmonic_patterns, hawkes_intensity,
    hawkins_dynamic_zones, head_shoulders, hikkake_pattern,
    heikin_ashi_reversal, henriksson_merton, herfindahl, hierarchical_risk_parity, hill_estimator,
    hilbert_transform, hindenburg_omen, hodrick_prescott, holiday_calendar, holiday_seasonality,
    holt_winters, hull_white,
    holy_grail, hurst_exponent, iceberg_detector, imbalance_bar_chart, impulse_system,
    information_coefficient,
    information_ratio,
    inside_bar_breakout,
    insider_buying_scanner, intraday_intensity_index, intraday_seasonality,
    iron_butterfly, iron_condor,
    island_reversal, isotonic_regression,
    iv_rank_scanner, iv_skew_scanner, iv_solver, iv_term_structure, jade_lizard, jarque_bera,
    jelly_roll_arbitrage, jump_diffusion_simulator, jurik_ma,
    kagi_chart, kalman_dynamic_beta, kalman_filter_1d, katsanos_vfi, kelly_criterion,
    kicker_pattern,
    keltner_squeeze,
    key_rate_duration, key_reversal_bar, klinger_volume_oscillator, know_sure_thing,
    kolmogorov_smirnov_2sample, kpss_test,
    kullback_leibler_divergence, kyles_lambda, ladder_bottom_top, ledoit_wolf, lee_ready,
    levene_test,
    libor_ois_spread, linda_raschke_3_10, linear_regression_channel,
    linear_regression_curve, linear_regression_r_squared, linear_regression_slope,
    liquidity_pool_detector, liquidity_void_detector,
    liquidity_adjusted_var, liquidity_grab, ljung_box, lookback_option, low_vol_factor,
    lower_partial_moments, macaulay_duration, madrid_moving_average_ribbon,
    mahalanobis_distance, mann_whitney_u, marginal_var,
    margrabe_spread_option, markov_switching_2state, mat_hold_pattern, max_diversification,
    mcclellan_oscillator,
    median_price,
    median_realized_variance, meeting_lines, mesa_sine_wave,
    megaphone_pattern, microprice_stoikov,
    min_variance_portfolio, minervini_trend_template, models::{PriceBar, Trade, TradeSide},
    modigliani_m2, momentum_12_1, momentum_crash_protection, momentum_persistence_index,
    momentum_quality_score, monte_carlo_option,
    monthly_seasonality, morning_evening_star, moving_correlation,
    monte_carlo_var, mortgage_psa, moving_average_envelope, mtm_reconciliation,
    multi_leg_option_pricer, murrey_math,
    nadaraya_watson,
    negative_volume_index, nelson_siegel, nelson_siegel_svensson,
    newey_west, noise_to_signal_ratio, nyse_tick, omega_ratio, on_balance_volume, on_neck_in_neck,
    opening_range,
    option_open_interest_distribution, option_payoff_diagram, options_margin, order_block, ornstein_uhlenbeck,
    pain_index, pair_trade,
    pair_trade_zscore, partial_autocorrelation, pca, peaks_over_threshold, pelt_segmentation,
    permutation_entropy, pickands_estimator, pin_bar, pinball_setup, pivot_points,
    pocket_pivot_buy, point_and_figure, portfolio_heat, position_aging,
    position_irr, positive_volume_index, post_earnings_drift, power_bar, power_option, pp_test,
    premarket_gap_scanner,
    premier_stochastic, premium_discount, pretty_good_oscillator, price_volume_oscillator,
    price_volume_trend,
    probability_of_informed_trading, put_call_ratio,
    qstick, quality_factor,
    quantile_regression, quanto_option, ramsey_reset, random_walk_index,
    range_bar_chart, range_contraction, range_expansion, range_filter, range_volatility,
    rank_correlation, ratio_chart,
    realized_correlation, realized_higher_moments, realized_kernel, realized_quarticity,
    realized_semivariance, realized_skewness, realized_volatility, recursive_ma,
    recovery_factor, reconcile_1099b, regime_classifier, relative_strength,
    relative_strength_vs_market,
    relative_volatility_index,
    relative_volume_scanner, relative_volume_zscore, repo_rate_spread,
    rising_falling_three_methods,
    risk_adjusted_ratios,
    risk_parity_weights, risk_reward, roll_spread, rolling_beta, rolling_drawdown,
    ross_hook,
    rolling_quantile, rolling_sharpe, rolling_sortino, rolling_zscore, roofing_filter,
    round_levels,
    rounding_pattern, runs_test,
    sabr,
    sample_entropy, savitzky_golay, scan_orchestrator, second_order_greeks, sector_rotation,
    separating_lines, session_vwap, side_by_side_white_lines, sperandeo_1_2_3,
    spinning_top_marubozu,
    shark_pattern, short_interest_scanner, sip_simulator, spearman_correlation,
    spread_attribution, spread_chart,
    standard_error_bands, starc_bands, stick_sandwich, stochastic_momentum_index,
    sterling_ratio, stochastic_rsi, stop_hunt, straddle, strangle, strategy_correlation,
    subsampled_realized_variance, swing_failure_pattern,
    summation_index,
    supertrend_dual, survival_probability, svi_volatility_smile,
    swap_rate_curve, swap_valuation, swaption_black,
    swing_points, swing_strength_index, symbol_filter,
    t3_moving_average, t_copula, tail_dependence, tail_ratio, tasuki_gap,
    tape_density, tape_speed, tax_lot_optimizer, td_sequential, ted_spread, term_premium_estimator,
    term_spread,
    three_bar_reversal,
    three_drive_pattern, three_inside_up_down, three_line_break, three_outside_up_down,
    three_white_soldiers_crows, thrusting_pattern, traders_action_zone,
    traders_dynamic_index,
    tick_bar_chart, tick_extreme, timeframe_confluence, tpo_profile, trade_quality_stats,
    tracking_error,
    treynor_black,
    treynor_jensen, treynor_mazuy, triangular_ma, trinomial_tree, triple_screen,
    triple_top_bottom, ttm_squeeze, ttm_trend, turtle_soup, tweezer_top_bottom, twap,
    twiggs_money_flow,
    typical_price,
    two_scales_realized_variance,
    ulcer_index, ulcer_performance_index, ultimate_smoother, unusual_options_activity,
    up_down_capture, upside_gap_two_crows, value_at_risk_filtered_historical,
    value_at_risk_historical, value_factor,
    var_backtest_christoffersen,
    var_backtest_kupiec, variance_ratio_test, variance_swap, variance_swap_strike, vasicek,
    vcp_pattern, vector_autoregression, velocity_indicator, vix_basis, vix_skew_smirk,
    vol_risk_premium, vol_targeting_sizer, volatility_smile,
    volatility_breakout_system, volatility_managed_portfolio, volatility_quality_index,
    volatility_stop, volatility_swap,
    volume_at_price, volume_bar_chart, volume_burst, volume_climax_pattern,
    volume_force_index,
    volume_oscillator, volume_weighted_macd,
    volume_zone_oscillator, vortex_indicator, vpin, vsa,
    vwema, walk_forward, wasserstein_1d, wavelet_decomposition_haar,
    weighted_close, weighted_midprice, weinstein_stages, weiss_wave, welch_periodogram,
    white_robust_se,
    wilcoxon_signed_rank, williams_accumulation_distribution, woodie_pivots, woodies_cci,
    wyckoff,
    yield_curve_bootstrap, z_score_indicator, z_spread, zero_lag_macd,
};

pub fn router() -> Router<AppState> {
    Router::new()
        // ── Discipline circuit breakers ────────────────────────────────
        .route("/discipline/daily-loss-limit",    post(daily_loss_limit_route))
        .route("/discipline/drawdown-throttle",   post(drawdown_throttle_route))
        .route("/discipline/goal-tracker",        post(goal_tracker_route))
        // ── Options margin (Reg-T) ─────────────────────────────────────
        .route("/options/calc/margin-naked-short", post(margin_naked_short_route))
        .route("/options/calc/margin-vertical",    post(margin_vertical_route))
        // ── Portfolio reporting ────────────────────────────────────────
        .route("/portfolio/position-aging",       post(position_aging_route))
        .route("/portfolio/position-irr",         post(position_irr_route))
        // ── Sentiment indicator ────────────────────────────────────────
        .route("/sentiment/calc/put-call-ratio",  post(put_call_ratio_route))
        // ── Tax reconciliation ─────────────────────────────────────────
        .route("/tax/reconcile-1099b",            post(reconcile_1099b_route))
        // ── Risk:reward planning ───────────────────────────────────────
        .route("/calc/risk-reward",               post(risk_reward_route))
        // ── Rolling-window analytics ───────────────────────────────────
        .route("/analytics/rolling-zscore",       post(rolling_zscore_route))
        // ── Strategy + spread analytics ────────────────────────────────
        .route("/analytics/strategy-correlation", post(strategy_correlation_route))
        .route("/analytics/spread-attribution",   post(spread_attribution_route))
        .route("/analytics/pair-trade-signal",    post(pair_trade_signal_route))
        // ── Decision systems ───────────────────────────────────────────
        .route("/discipline/triple-screen",       post(triple_screen_route))
        // ── Execution-quality TCA ──────────────────────────────────────
        .route("/microstructure/twap",            post(twap_route))
        // ── Volatility-based stops ─────────────────────────────────────
        .route("/discipline/chandelier-stop",     post(chandelier_stop_route))
        .route("/discipline/vol-stop-close",      post(vol_stop_close_route))
        // ── Broker reconciliation ──────────────────────────────────────
        .route("/portfolio/mtm-reconciliation",   post(mtm_reconciliation_route))
        // ── Forecasting cones ──────────────────────────────────────────
        .route("/charts/atr-cone",                post(atr_cone_route))
        // ── Alligator indicator ────────────────────────────────────────
        .route("/bars/alligator",                 post(alligator_route))
        // ── Calendar helpers ───────────────────────────────────────────
        .route("/calendar/is-trading-day",        post(is_trading_day_route))
        .route("/calendar/next-trading-day",      post(next_trading_day_route))
        .route("/calendar/prior-trading-day",     post(prior_trading_day_route))
        .route("/calendar/add-trading-days",      post(add_trading_days_route))
        .route("/calendar/trading-days-between",  post(trading_days_between_route))
        .route("/calendar/earnings-window",       post(earnings_window_route))
        .route("/calendar/earnings-analysis",     post(earnings_analysis_route))
        // ── Symbol filter ──────────────────────────────────────────────
        .route("/filter/symbols",                 post(symbol_filter_route))
        // ── Futures roll schedule ──────────────────────────────────────
        .route("/futures/roll-schedule",          post(futures_roll_route))
        // ── New: SIP/DRIP + portfolio heat + HIFO lot optimizer ────────
        .route("/portfolio/sip-simulator",        post(sip_simulator_route))
        .route("/portfolio/heat",                 post(portfolio_heat_route))
        .route("/tax/lot-optimizer",              post(tax_lot_optimizer_route))
        // ── New: Volume burst + round levels + timeframe confluence ────
        .route("/analytics/volume-burst",         post(volume_burst_route))
        .route("/charts/round-levels",            post(round_levels_route))
        .route("/analytics/timeframe-confluence", post(timeframe_confluence_route))
        // ── Pattern primitives: crossover + breakout + range-contraction
        .route("/analytics/crossover",            post(crossover_route))
        .route("/analytics/breakout",             post(breakout_route))
        .route("/analytics/range-contraction",    post(range_contraction_route))
        // ── SMC primitives: stop hunt + FVG + order block ──────────────
        .route("/analytics/stop-hunt",            post(stop_hunt_route))
        .route("/analytics/fair-value-gap",       post(fair_value_gap_route))
        .route("/analytics/order-block",          post(order_block_route))
        // ── More SMC: BOS + CHoCH + equal levels ───────────────────────
        .route("/analytics/break-of-structure",   post(break_of_structure_route))
        .route("/analytics/change-of-character",  post(change_of_character_route))
        .route("/analytics/equal-levels",         post(equal_levels_route))
        // ── Order flow + ORB + displacement ────────────────────────────
        .route("/microstructure/cumulative-delta",post(cumulative_delta_route))
        .route("/analytics/displacement",         post(displacement_route))
        .route("/analytics/opening-range",        post(opening_range_route))
        // ── VSA + risk-adjusted metrics ────────────────────────────────
        .route("/analytics/vsa",                  post(vsa_route))
        .route("/analytics/ulcer-index",          post(ulcer_index_route))
        .route("/analytics/calmar-ratio",         post(calmar_ratio_route))
        // ── Wyckoff + premium/discount + CUSUM ─────────────────────────
        .route("/analytics/wyckoff",              post(wyckoff_route))
        .route("/analytics/premium-discount",     post(premium_discount_route))
        .route("/analytics/cusum",                post(cusum_route))
        // ── HA reversal + 3-bar reversal + range expansion ─────────────
        .route("/analytics/heikin-ashi-reversal", post(heikin_ashi_reversal_route))
        .route("/analytics/three-bar-reversal",   post(three_bar_reversal_route))
        .route("/analytics/range-expansion",      post(range_expansion_route))
        // ── Trend-efficiency primitives ────────────────────────────────
        .route("/analytics/choppiness",           post(choppiness_route))
        .route("/analytics/efficiency-ratio",     post(efficiency_ratio_route))
        .route("/analytics/random-walk-index",    post(random_walk_index_route))
        // ── Bill Williams AC + ICT liquidity grab + gap stats ──────────
        .route("/analytics/acceleration-deceleration", post(ac_route))
        .route("/analytics/liquidity-grab",            post(liquidity_grab_route))
        .route("/analytics/gap-fill-stats",            post(gap_fill_stats_route))
        // ── Market breadth + inside-bar pattern ────────────────────────
        .route("/analytics/arms-index",                post(arms_index_route))
        .route("/analytics/mcclellan-oscillator",      post(mcclellan_oscillator_route))
        .route("/analytics/inside-bar-breakout",       post(inside_bar_breakout_route))
        // ── Pattern detectors (XABCD harmonics, ABC, three-drive) ────
        .route("/analytics/harmonic-patterns",         post(harmonic_patterns_route))
        .route("/analytics/abc-pattern",               post(abc_pattern_route))
        .route("/analytics/three-drive-pattern",       post(three_drive_pattern_route))
        // ── Order flow + market internals ────────────────────────────
        .route("/orderflow/footprint-imbalance",       post(footprint_imbalance_route))
        .route("/orderflow/tape-density",              post(tape_density_route))
        .route("/orderflow/depth-imbalance",           post(depth_imbalance_route))
        .route("/internals/tick-extreme",              post(tick_extreme_route))
        .route("/internals/sector-rotation",           post(sector_rotation_route))
        // ── Options scanner (IV-rank universe ranking) ───────────────
        .route("/options/iv-rank-scanner",             post(iv_rank_scanner_route))
        // ── Universe scanner orchestrator ────────────────────────────
        .route("/scanner/run-universe",                post(scan_orchestrator_route))
        // ── Backtest sweep + walk-forward validation ─────────────────
        .route("/backtest/sweep-sma-cross",            post(sweep_sma_cross_route))
        .route("/backtest/sweep-bb-breakout",          post(sweep_bb_breakout_route))
        .route("/backtest/walk-forward-sma-cross",     post(walk_forward_sma_cross_route))
        .route("/internals/breadth-lines",             post(breadth_lines_route))
        .route("/internals/dark-pool-index",           post(dark_pool_index_route))
        .route("/scanner/post-earnings-drift",         post(post_earnings_drift_route))
        .route("/scanner/short-interest",              post(short_interest_route))
        .route("/scanner/relative-strength",           post(relative_strength_route))
        .route("/analytics/keltner-squeeze",           post(keltner_squeeze_route))
        .route("/analytics/divergence-detect",         post(divergence_detect_route))
        .route("/internals/cumulative-tick-trin",      post(cumulative_tick_trin_route))
        .route("/internals/summation-index",           post(summation_index_route))
        .route("/internals/hindenburg-omen",           post(hindenburg_omen_route))
        .route("/scanner/premarket-gap",               post(premarket_gap_route))
        .route("/scanner/halt-resume",                 post(halt_resume_route))
        .route("/options/calc/second-order-greeks",    post(second_order_greeks_route))
        .route("/microstructure/vpin",                 post(vpin_route))
        .route("/analytics/cup-and-handle",            post(cup_and_handle_route))
        .route("/analytics/head-shoulders",            post(head_shoulders_route))
        .route("/scanner/breakout-52w",                post(breakout_52w_route))
        .route("/analytics/ewma-volatility",           post(ewma_volatility_route))
        .route("/futures/cot-report",                  post(cot_report_route))
        .route("/options/calc/calendar-spread",        post(calendar_spread_route))
        .route("/options/calc/iron-condor",            post(iron_condor_route))
        .route("/portfolio/marginal-var",              post(marginal_var_route))
        .route("/analytics/realized-volatility",       post(realized_volatility_route))
        .route("/analytics/amihud-illiquidity",        post(amihud_illiquidity_route))
        .route("/microstructure/kyles-lambda",         post(kyles_lambda_route))
        .route("/analytics/tpo-profile",               post(tpo_profile_route))
        .route("/analytics/omega-ratio",               post(omega_ratio_route))
        .route("/analytics/hurst-exponent",            post(hurst_exponent_route))
        .route("/analytics/garch-1-1",                 post(garch_1_1_route))
        .route("/analytics/cointegration",             post(cointegration_route))
        .route("/analytics/treynor-mazuy",             post(treynor_mazuy_route))
        .route("/analytics/ornstein-uhlenbeck",        post(ornstein_uhlenbeck_route))
        .route("/analytics/range-volatility",          post(range_volatility_route))
        .route("/microstructure/roll-spread",          post(roll_spread_route))
        .route("/microstructure/lee-ready",            post(lee_ready_route))
        .route("/options/calc/variance-swap",          post(variance_swap_route))
        .route("/analytics/td-sequential",             post(td_sequential_route))
        .route("/analytics/andrews-pitchfork",         post(andrews_pitchfork_route))
        .route("/analytics/anchored-momentum",         post(anchored_momentum_route))
        .route("/analytics/information-ratio",         post(information_ratio_route))
        .route("/analytics/gain-pain-ratio",           post(gain_pain_ratio_route))
        .route("/analytics/henriksson-merton",         post(henriksson_merton_route))
        .route("/options/calc/iv-solver",              post(iv_solver_route))
        .route("/options/calc/black-76",               post(black76_route))
        .route("/analytics/deflated-sharpe",           post(deflated_sharpe_route))
        .route("/charts/murrey-math",                  post(murrey_math_route))
        .route("/analytics/conditional-var",           post(conditional_var_route))
        .route("/analytics/fama-french-3",             post(fama_french_3_route))
        .route("/analytics/carhart-4",                 post(carhart_4_route))
        .route("/analytics/pair-trade-zscore",         post(pair_trade_zscore_route))
        .route("/options/calc/butterfly-spread",       post(butterfly_spread_route))
        .route("/options/calc/jade-lizard",            post(jade_lizard_route))
        .route("/analytics/realized-correlation",      post(realized_correlation_route))
        .route("/analytics/cornish-fisher-var",        post(cornish_fisher_var_route))
        .route("/bonds/calc/macaulay-duration",        post(macaulay_duration_route))
        .route("/bonds/calc/yield-curve-bootstrap",    post(yield_curve_bootstrap_route))
        .route("/portfolio/herfindahl",                post(herfindahl_route))
        .route("/analytics/treynor-jensen",            post(treynor_jensen_route))
        .route("/portfolio/risk-parity-weights",       post(risk_parity_weights_route))
        .route("/portfolio/brinson-attribution",       post(brinson_attribution_route))
        .route("/bonds/calc/nelson-siegel-fit",        post(nelson_siegel_fit_route))
        .route("/bonds/calc/svensson-fit",             post(svensson_fit_route))
        .route("/options/calc/margrabe-spread",        post(margrabe_spread_route))
        .route("/options/calc/asian-geometric",        post(asian_option_route))
        .route("/options/calc/barrier",                post(barrier_option_route))
        .route("/bonds/calc/vasicek-zcb",              post(vasicek_zcb_route))
        .route("/portfolio/black-litterman",           post(black_litterman_route))
        .route("/analytics/liquidity-adjusted-var",    post(liquidity_adjusted_var_route))
        .route("/bonds/calc/cir-zcb",                  post(cir_zcb_route))
        .route("/options/calc/sabr-vol",               post(sabr_vol_route))
        .route("/options/calc/lookback",               post(lookback_option_route))
        .route("/options/calc/digital",                post(digital_option_route))
        .route("/analytics/granger-causality",         post(granger_causality_route))
        .route("/portfolio/ledoit-wolf-shrinkage",     post(ledoit_wolf_route))
        .route("/microstructure/almgren-chriss",       post(almgren_chriss_route))
        .route("/bonds/calc/hull-white-zcb",           post(hull_white_zcb_route))
        .route("/options/calc/compound",               post(compound_option_route))
        .route("/options/calc/quanto",                 post(quanto_option_route))
        .route("/options/calc/cliquet",                post(cliquet_option_route))
        .route("/analytics/rank-correlation",          post(rank_correlation_route))
        .route("/analytics/tail-dependence",           post(tail_dependence_route))
        .route("/analytics/vector-autoregression",     post(vector_autoregression_route))
        .route("/analytics/cholesky",                  post(cholesky_route))
        .route("/analytics/pca",                       post(pca_route))
        .route("/options/calc/power",                  post(power_option_route))
        .route("/options/calc/gap",                    post(gap_option_route))
        .route("/rates/calc/fra",                      post(fra_route))
        .route("/rates/calc/caplet-black76",           post(caplet_black76_route))
        .route("/portfolio/trade-quality-stats",       post(trade_quality_stats_route))
        .route("/options/calc/chooser",                post(chooser_option_route))
        .route("/analytics/conditional-drawdown",      post(conditional_drawdown_route))
        .route("/analytics/risk-adjusted-ratios",      post(risk_adjusted_ratios_route))
        .route("/analytics/pain-index",                post(pain_index_route))
        .route("/microstructure/weighted-midprice",    post(weighted_midprice_route))
        .route("/microstructure/effective-spread",     post(effective_spread_route))
        .route("/scanner/momentum-12-1",               post(momentum_12_1_route))
        .route("/options/calc/bachelier",              post(bachelier_route))
        .route("/options/calc/swaption-black",         post(swaption_black_route))
        .route("/credit/calc/cds",                     post(cds_pricing_route))
        .route("/bonds/calc/asset-swap-spread",        post(asset_swap_spread_route))
        .route("/analytics/holt-winters",              post(holt_winters_route))
        .route("/analytics/vwema",                     post(vwema_route))
        .route("/analytics/smi",                       post(stochastic_momentum_index_route))
        .route("/options/calc/american",               post(american_binomial_route))
        .route("/options/calc/bermudan",               post(bermudan_binomial_route))
        .route("/options/calc/convertible-bond",       post(convertible_bond_route))
        .route("/portfolio/hierarchical-risk-parity",  post(hierarchical_risk_parity_route))
        .route("/microstructure/hawkes-intensity",     post(hawkes_intensity_route))
        .route("/analytics/arima-111",                 post(arima_111_route))
        .route("/options/calc/greeks-profile",         post(greeks_profile_route))
        .route("/options/calc/trinomial",              post(trinomial_tree_route))
        .route("/analytics/arch-lm-test",              post(arch_lm_test_route))
        .route("/analytics/ljung-box",                 post(ljung_box_route))
        .route("/portfolio/min-variance",              post(min_variance_portfolio_route))
        .route("/analytics/candle-patterns",           post(candle_patterns_route))
        .route("/analytics/adf-test",                  post(adf_standalone_route))
        .route("/analytics/bollinger-oscillators",     post(bollinger_oscillators_route))
        .route("/analytics/var-backtest-kupiec",       post(var_backtest_kupiec_route))
        .route("/analytics/var-backtest-christoffersen", post(var_backtest_christoffersen_route))
        .route("/scanner/value-factor",                post(value_factor_route))
        .route("/scanner/quality-factor",              post(quality_factor_route))
        .route("/scanner/low-vol-factor",              post(low_vol_factor_route))
        .route("/scanner/composite-factor-scoring",    post(composite_factor_scoring_route))
        .route("/credit/calc/survival-probability",    post(survival_probability_route))
        .route("/options/calc/straddle",               post(straddle_route))
        .route("/options/calc/strangle",               post(strangle_route))
        .route("/options/calc/iron-butterfly",         post(iron_butterfly_route))
        .route("/options/calc/collar",                 post(collar_route))
        .route("/analytics/hodrick-prescott",          post(hodrick_prescott_route))
        .route("/analytics/kalman-filter-1d",          post(kalman_filter_1d_route))
        .route("/analytics/block-bootstrap",           post(block_bootstrap_route))
        .route("/analytics/realized-higher-moments",   post(realized_higher_moments_route))
        .route("/analytics/lower-partial-moments",     post(lower_partial_moments_route))
        .route("/analytics/dfa",                       post(dfa_route))
        .route("/analytics/sample-entropy",            post(sample_entropy_route))
        .route("/analytics/permutation-entropy",       post(permutation_entropy_route))
        .route("/patterns/triple-top-bottom",          post(triple_top_bottom_route))
        .route("/portfolio/max-diversification",       post(max_diversification_route))
        .route("/analytics/realized-semivariance",     post(realized_semivariance_route))
        .route("/analytics/bipower-variation",         post(bipower_variation_route))
        .route("/analytics/up-down-capture",           post(up_down_capture_route))
        .route("/analytics/modigliani-m2",             post(modigliani_m2_route))
        .route("/analytics/beta-shrinkage",            post(beta_shrinkage_route))
        .route("/credit/calc/key-rate-duration",       post(key_rate_duration_route))
        .route("/portfolio/treynor-black",             post(treynor_black_route))
        .route("/analytics/vortex-indicator",          post(vortex_indicator_route))
        .route("/analytics/pivot-points",              post(pivot_points_route))
        .route("/analytics/aroon-indicator",           post(aroon_indicator_route))
        .route("/analytics/donchian-channels",         post(donchian_channels_route))
        .route("/analytics/stochastic-rsi",            post(stochastic_rsi_route))
        .route("/analytics/bollinger-band-width",      post(bollinger_band_width_route))
        .route("/credit/calc/bond-convexity",          post(bond_convexity_route))
        .route("/analytics/accumulation-distribution-line", post(adl_route))
        .route("/analytics/on-balance-volume",         post(obv_route))
        .route("/analytics/chaikin-oscillator",        post(chaikin_oscillator_route))
        .route("/analytics/klinger-volume-oscillator", post(klinger_volume_oscillator_route))
        .route("/analytics/chande-momentum-oscillator", post(chande_momentum_oscillator_route))
        .route("/analytics/hill-estimator",            post(hill_estimator_route))
        .route("/analytics/component-var",             post(component_var_route))
        .route("/analytics/alma",                      post(alma_route))
        .route("/analytics/t3-moving-average",         post(t3_route))
        .route("/analytics/frama",                     post(frama_route))
        .route("/analytics/coppock-curve",             post(coppock_curve_route))
        .route("/analytics/detrended-price-oscillator", post(dpo_route))
        .route("/analytics/fibonacci-retracements",    post(fibonacci_retracements_route))
        .route("/analytics/expected-shortfall-contribution", post(es_contribution_route))
        .route("/credit/calc/z-spread",                post(z_spread_route))
        .route("/credit/calc/swap-valuation",          post(swap_valuation_route))
        .route("/credit/calc/cross-currency-basis",    post(cross_currency_basis_route))
        .route("/scanner/gex",                         post(gex_scanner_route))
        .route("/scanner/unusual-options-activity",    post(unusual_options_activity_route))
        .route("/analytics/gjr-garch",                 post(gjr_garch_route))
        .route("/analytics/fama-french-3factor",       post(fama_french_3factor_route))
        .route("/credit/calc/mortgage-psa",            post(mortgage_psa_route))
        .route("/analytics/nadaraya-watson",           post(nadaraya_watson_route))
        .route("/scanner/insider-buying",              post(insider_buying_route))
        .route("/scanner/earnings-revision",           post(earnings_revision_route))
        .route("/patterns/bump-and-run",               post(bump_and_run_route))
        .route("/patterns/diamond",                    post(diamond_pattern_route))
        .route("/analytics/probability-of-informed-trading", post(pin_route))
        .route("/analytics/mahalanobis-distance",      post(mahalanobis_distance_route))
        .route("/analytics/autocorrelation-function",  post(acf_route))
        .route("/analytics/partial-autocorrelation",   post(pacf_route))
        .route("/analytics/point-and-figure",          post(point_and_figure_route))
        .route("/patterns/darvas-box",                 post(darvas_box_route))
        .route("/analytics/supertrend-dual",           post(supertrend_dual_route))
        .route("/analytics/hilbert-transform",         post(hilbert_transform_route))
        .route("/analytics/jarque-bera",               post(jarque_bera_route))
        .route("/analytics/spearman-correlation",      post(spearman_correlation_route))
        .route("/analytics/har-volatility",            post(har_volatility_route))
        .route("/analytics/variance-swap-strike",      post(variance_swap_strike_route))
        .route("/analytics/gaussian-copula",           post(gaussian_copula_route))
        .route("/analytics/chow-test",                 post(chow_test_route))
        .route("/analytics/breusch-godfrey",           post(breusch_godfrey_route))
        .route("/analytics/variance-ratio-test",       post(variance_ratio_test_route))
        .route("/analytics/runs-test",                 post(runs_test_route))
        .route("/analytics/corwin-schultz-spread",     post(corwin_schultz_spread_route))
        .route("/analytics/hampel-filter",             post(hampel_filter_route))
        .route("/credit/calc/breakeven-inflation",     post(breakeven_inflation_route))
        .route("/credit/calc/carry-roll-decomposition", post(carry_roll_decomposition_route))
        .route("/portfolio/vol-targeting-sizer",       post(vol_targeting_sizer_route))
        .route("/analytics/kolmogorov-smirnov-2sample", post(ks_2sample_route))
        .route("/analytics/anderson-darling-normality", post(ad_normality_route))
        .route("/analytics/kpss-test",                 post(kpss_test_route))
        .route("/analytics/breusch-pagan-test",        post(breusch_pagan_test_route))
        .route("/analytics/kullback-leibler-divergence", post(kl_divergence_route))
        .route("/analytics/wasserstein-1d",            post(wasserstein_1d_route))
        .route("/scanner/iv-skew",                     post(iv_skew_scanner_route))
        .route("/analytics/two-scales-realized-variance", post(tsrv_route))
        .route("/analytics/subsampled-realized-variance", post(subsampled_rv_route))
        .route("/analytics/realized-kernel",           post(realized_kernel_route))
        .route("/analytics/noise-to-signal-ratio",     post(nsr_route))
        .route("/analytics/realized-skewness",         post(realized_skewness_route))
        .route("/analytics/realized-quarticity",       post(realized_quarticity_route))
        .route("/analytics/median-realized-variance",  post(median_rv_route))
        .route("/analytics/mann-whitney-u",            post(mann_whitney_u_route))
        .route("/analytics/wilcoxon-signed-rank",      post(wilcoxon_signed_rank_route))
        .route("/analytics/levene-test",               post(levene_test_route))
        .route("/scanner/relative-volume",             post(relative_volume_scanner_route))
        .route("/analytics/iv-term-structure",         post(iv_term_structure_route))
        .route("/analytics/ramsey-reset",              post(ramsey_reset_route))
        .route("/analytics/distance-correlation",      post(distance_correlation_route))
        .route("/analytics/information-coefficient",   post(information_coefficient_route))
        .route("/options/calc/box-spread",             post(box_spread_route))
        .route("/options/calc/jelly-roll-arbitrage",   post(jelly_roll_arbitrage_route))
        .route("/analytics/factor-neutralization",     post(factor_neutralization_route))
        .route("/analytics/crps",                      post(crps_route))
        .route("/analytics/brier-score",               post(brier_score_route))
        .route("/analytics/decile-long-short",         post(decile_long_short_route))
        .route("/analytics/newey-west",                post(newey_west_route))
        .route("/analytics/diebold-mariano",           post(diebold_mariano_route))
        .route("/options/calc/gamma-scalping-pnl",     post(gamma_scalping_pnl_route))
        .route("/options/calc/breeden-litzenberger",   post(breeden_litzenberger_route))
        .route("/analytics/white-robust-se",           post(white_robust_se_route))
        .route("/analytics/expected-calibration-error", post(ece_route))
        .route("/analytics/vol-risk-premium",          post(vol_risk_premium_route))
        .route("/credit/calc/libor-ois-spread",        post(libor_ois_spread_route))
        .route("/analytics/bartlett-variance-test",    post(bartlett_variance_test_route))
        .route("/analytics/friedman-test",             post(friedman_test_route))
        .route("/analytics/isotonic-regression",       post(isotonic_regression_route))
        .route("/analytics/pelt-segmentation",         post(pelt_segmentation_route))
        .route("/analytics/gonzalo-granger-decomposition", post(gonzalo_granger_route))
        .route("/analytics/monte-carlo-var",           post(monte_carlo_var_route))
        .route("/analytics/gpd-tail-fit",              post(gpd_tail_fit_route))
        .route("/analytics/peaks-over-threshold",      post(peaks_over_threshold_route))
        .route("/analytics/evt-value-at-risk",         post(evt_value_at_risk_route))
        .route("/analytics/pickands-estimator",        post(pickands_estimator_route))
        .route("/analytics/empirical-distribution-function", post(ecdf_route))
        .route("/analytics/quantile-regression",       post(quantile_regression_route))
        .route("/patterns/megaphone",                  post(megaphone_pattern_route))
        .route("/analytics/rolling-drawdown",          post(rolling_drawdown_route))
        .route("/analytics/rolling-sharpe",            post(rolling_sharpe_route))
        .route("/analytics/rolling-sortino",           post(rolling_sortino_route))
        .route("/analytics/rolling-beta",              post(rolling_beta_route))
        .route("/analytics/expected-drawdown",         post(expected_drawdown_route))
        .route("/analytics/engle-granger-2step",       post(engle_granger_2step_route))
        .route("/patterns/vcp",                        post(vcp_pattern_route))
        .route("/analytics/burke-ratio",               post(burke_ratio_route))
        .route("/analytics/sterling-ratio",            post(sterling_ratio_route))
        .route("/analytics/recovery-factor",           post(recovery_factor_route))
        .route("/analytics/gain-to-pain-ratio",        post(gain_to_pain_ratio_route))
        .route("/analytics/tail-ratio",                post(tail_ratio_route))
        .route("/analytics/weinstein-stages",          post(weinstein_stages_route))
        .route("/analytics/expectancy-per-trade",      post(expectancy_per_trade_route))
        .route("/portfolio/kelly-criterion-discrete",  post(kelly_discrete_route))
        .route("/portfolio/kelly-criterion-continuous", post(kelly_continuous_route))
        .route("/analytics/tracking-error",            post(tracking_error_route))
        .route("/portfolio/active-share",              post(active_share_route))
        .route("/analytics/savitzky-golay",            post(savitzky_golay_route))
        .route("/scanner/minervini-trend-template",    post(minervini_route))
        .route("/scanner/pocket-pivot-buy",            post(pocket_pivot_route))
        .route("/analytics/bootstrap-pnl",             post(bootstrap_pnl_route))
        .route("/options/calc/finite-difference",      post(finite_difference_option_route))
        .route("/options/calc/monte-carlo",            post(monte_carlo_option_route))
        .route("/options/calc/forward-start",          post(forward_start_option_route))
        .route("/patterns/rounding",                   post(rounding_pattern_route))
        .route("/patterns/island-reversal",            post(island_reversal_route))
        .route("/credit/calc/ted-spread",              post(ted_spread_route))
        .route("/portfolio/volatility-managed",        post(volatility_managed_portfolio_route))
        .route("/options/calc/volatility-swap",        post(volatility_swap_route))
        .route("/credit/calc/nelson-siegel-svensson",  post(nelson_siegel_svensson_route))
        .route("/analytics/pp-test",                   post(pp_test_route))
        .route("/patterns/key-reversal-bar",           post(key_reversal_bar_route))
        .route("/portfolio/momentum-crash-protection", post(momentum_crash_protection_route))
        .route("/analytics/t-copula",                  post(t_copula_route))
        .route("/analytics/welch-periodogram",         post(welch_periodogram_route))
        .route("/analytics/williams-ad",               post(williams_accumulation_distribution_route))
        .route("/analytics/chande-trend-index",        post(chande_trend_index_route))
        .route("/analytics/balance-of-power",          post(balance_of_power_route))
        .route("/analytics/relative-volatility-index", post(relative_volatility_index_route))
        .route("/analytics/demarker-oscillator",       post(demarker_oscillator_route))
        .route("/analytics/woodies-cci",               post(woodies_cci_route))
        .route("/analytics/premier-stochastic",        post(premier_stochastic_route))
        .route("/analytics/qstick",                    post(qstick_route))
        .route("/analytics/know-sure-thing",           post(know_sure_thing_route))
        .route("/analytics/disparity-index",           post(disparity_index_route))
        .route("/analytics/camarilla-pivots",          post(camarilla_pivots_route))
        .route("/analytics/linear-regression-channel", post(linear_regression_channel_route))
        .route("/analytics/gator-oscillator",          post(gator_oscillator_route))
        .route("/analytics/triangular-ma",             post(triangular_ma_route))
        .route("/analytics/price-volume-trend",        post(price_volume_trend_route))
        .route("/analytics/negative-volume-index",     post(negative_volume_index_route))
        .route("/analytics/positive-volume-index",     post(positive_volume_index_route))
        .route("/analytics/starc-bands",               post(starc_bands_route))
        .route("/analytics/guppy-mma",                 post(guppy_mma_route))
        .route("/analytics/accumulation-swing-index",  post(accumulation_swing_index_route))
        .route("/analytics/twiggs-money-flow",         post(twiggs_money_flow_route))
        .route("/analytics/elder-safezone-stop",       post(elder_safezone_stop_route))
        .route("/analytics/jurik-ma",                  post(jurik_ma_route))
        .route("/analytics/chande-kroll-stop",         post(chande_kroll_stop_route))
        .route("/analytics/elder-thermometer",         post(elder_thermometer_route))
        .route("/analytics/floor-pivots",              post(floor_pivots_route))
        .route("/analytics/traders-dynamic-index",     post(traders_dynamic_index_route))
        .route("/analytics/ttm-squeeze",               post(ttm_squeeze_route))
        .route("/analytics/elliott-wave-oscillator",   post(elliott_wave_oscillator_route))
        .route("/analytics/woodie-pivots",             post(woodie_pivots_route))
        .route("/analytics/fibonacci-pivots",          post(fibonacci_pivots_route))
        .route("/analytics/pretty-good-oscillator",    post(pretty_good_oscillator_route))
        .route("/analytics/roofing-filter",            post(roofing_filter_route))
        .route("/analytics/weiss-wave",                post(weiss_wave_route))
        .route("/analytics/ttm-trend",                 post(ttm_trend_route))
        .route("/analytics/volatility-quality-index",  post(volatility_quality_index_route))
        .route("/analytics/demark-pivots",             post(demark_pivots_route))
        .route("/analytics/gann-high-low-activator",   post(gann_high_low_activator_route))
        .route("/analytics/impulse-system",            post(impulse_system_route))
        .route("/analytics/damiani-volatmeter",        post(damiani_volatmeter_route))
        .route("/analytics/ehlers-instant-trendline",  post(ehlers_instant_trendline_route))
        .route("/analytics/range-filter",              post(range_filter_route))
        .route("/analytics/linda-raschke-3-10",        post(linda_raschke_3_10_route))
        .route("/analytics/ehlers-mama-fama",          post(ehlers_mama_fama_route))
        .route("/analytics/bressert-dss",              post(bressert_dss_route))
        .route("/analytics/traders-action-zone",       post(traders_action_zone_route))
        .route("/analytics/intraday-intensity-index",  post(intraday_intensity_index_route))
        .route("/analytics/chande-dynamic-momentum",   post(chande_dynamic_momentum_index_route))
        .route("/analytics/standard-error-bands",      post(standard_error_bands_route))
        .route("/analytics/ehlers-cti",                post(ehlers_correlation_trend_indicator_route))
        .route("/analytics/chandelier-exit",           post(chandelier_exit_route))
        .route("/patterns/holy-grail",                 post(holy_grail_route))
        .route("/analytics/volume-oscillator",         post(volume_oscillator_route))
        .route("/analytics/chop-zone",                 post(chop_zone_indicator_route))
        .route("/analytics/alphatrend",                post(alphatrend_route))
        .route("/analytics/linear-regression-slope",   post(linear_regression_slope_route))
        .route("/analytics/ulcer-performance-index",   post(ulcer_performance_index_route))
        .route("/analytics/bollinger-percent-b",       post(bollinger_percent_b_route))
        .route("/analytics/volume-zone-oscillator",    post(volume_zone_oscillator_route))
        .route("/patterns/gartley",                    post(gartley_pattern_route))
        .route("/patterns/pinball-setup",              post(pinball_setup_route))
        .route("/analytics/accumulation-volume-pattern", post(accumulation_volume_pattern_route))
        .route("/analytics/recursive-ma",              post(recursive_ma_route))
        .route("/analytics/detrended-synthetic-price", post(detrended_synthetic_price_route))
        .route("/patterns/bat",                        post(bat_pattern_route))
        .route("/patterns/butterfly",                  post(butterfly_pattern_route))
        .route("/patterns/crab",                       post(crab_pattern_route))
        .route("/patterns/cypher",                     post(cypher_pattern_route))
        .route("/patterns/shark",                      post(shark_pattern_route))
        .route("/patterns/turtle-soup",                post(turtle_soup_route))
        .route("/patterns/eighty-twenty",              post(eighty_twenty_setup_route))
        .route("/analytics/z-score-indicator",         post(z_score_indicator_route))
        .route("/analytics/katsanos-vfi",              post(katsanos_vfi_route))
        .route("/analytics/linear-regression-curve",   post(linear_regression_curve_route))
        .route("/analytics/moving-average-envelope",   post(moving_average_envelope_route))
        .route("/analytics/bollinger-squeeze",         post(bollinger_squeeze_route))
        .route("/patterns/anti-setup",                 post(anti_setup_route))
        .route("/analytics/choppy-market-index",       post(choppy_market_index_route))
        .route("/analytics/madrid-ribbon",             post(madrid_moving_average_ribbon_route))
        .route("/analytics/velocity-indicator",        post(velocity_indicator_route))
        .route("/analytics/volatility-breakout-system", post(volatility_breakout_system_route))
        .route("/analytics/detrended-volatility-oscillator", post(detrended_volatility_oscillator_route))
        .route("/analytics/ad-oscillator",             post(accumulation_distribution_oscillator_route))
        .route("/analytics/candle-strength-index",     post(candle_strength_index_route))
        .route("/analytics/hawkins-dynamic-zones",     post(hawkins_dynamic_zones_route))
        .route("/analytics/ehlers-decycler-oscillator", post(ehlers_decycler_oscillator_route))
        .route("/patterns/abcd",                       post(abcd_pattern_route))
        .route("/analytics/gann-fan",                  post(gann_fan_route))
        .route("/analytics/ratio-chart",               post(ratio_chart_route))
        .route("/analytics/spread-chart",              post(spread_chart_route))
        .route("/analytics/bollinger-bandwidth-percentile", post(bollinger_bandwidth_percentile_route))
        .route("/analytics/relative-strength-vs-market", post(relative_strength_vs_market_route))
        .route("/analytics/ultimate-smoother",         post(ultimate_smoother_route))
        .route("/analytics/centered-smoothed-momentum", post(ehlers_centered_smoothed_momentum_route))
        .route("/patterns/five-o",                     post(five_o_pattern_route))
        .route("/analytics/typical-price",             post(typical_price_route))
        .route("/analytics/weighted-close",            post(weighted_close_route))
        .route("/patterns/engulfing-scanner",          post(engulfing_pattern_scanner_route))
        .route("/scanner/fifty-two-week-high-low",     post(fifty_two_week_high_low_scanner_route))
        .route("/analytics/session-vwap",              post(session_vwap_route))
        .route("/analytics/tape-speed",                post(tape_speed_route))
        .route("/analytics/liquidity-pools",           post(liquidity_pool_detector_route))
        .route("/analytics/absorption-detector",       post(absorption_detector_route))
        .route("/analytics/iceberg-detector",          post(iceberg_detector_route))
        .route("/analytics/median-price",              post(median_price_route))
        .route("/analytics/arms-high-low-index",       post(arms_high_low_index_route))
        .route("/analytics/fibonacci-extensions",      post(fibonacci_extensions_route))
        .route("/analytics/donchian-oscillator",       post(donchian_oscillator_route))
        .route("/analytics/mesa-sine-wave",            post(mesa_sine_wave_route))
        .route("/analytics/cybernetic-fisher",         post(ehlers_fisher_cybernetic_route))
        .route("/analytics/bid-ask-volume-ratio",      post(bid_ask_volume_ratio_route))
        .route("/analytics/nyse-tick",                 post(nyse_tick_route))
        .route("/analytics/chande-volatility-index",   post(chande_volatility_index_route))
        .route("/patterns/gann-swing-chart",           post(gann_swing_chart_route))
        .route("/analytics/vix-basis",                 post(vix_basis_route))
        .route("/analytics/vix-skew-smirk",            post(vix_skew_smirk_route))
        .route("/options/calc/gamma-pin-zone",         post(gamma_pin_zone_route))
        .route("/analytics/borrow-rate-indicator",     post(borrow_rate_indicator_route))
        .route("/analytics/earnings-surprise-indicator", post(earnings_surprise_indicator_route))
        .route("/analytics/liquidity-void-detector",   post(liquidity_void_detector_route))
        .route("/patterns/three-white-soldiers-crows", post(three_white_soldiers_crows_route))
        .route("/patterns/morning-evening-star",       post(morning_evening_star_route))
        .route("/patterns/dark-cloud-piercing",        post(dark_cloud_piercing_route))
        .route("/patterns/hanging-man-shooting-star",  post(hanging_man_shooting_star_route))
        .route("/analytics/term-spread",               post(term_spread_route))
        .route("/analytics/volume-weighted-macd",      post(volume_weighted_macd_route))
        .route("/analytics/cvd-zscore",                post(cumulative_volume_delta_zscore_route))
        .route("/patterns/doji-variants",              post(doji_variants_route))
        .route("/patterns/harami",                     post(harami_pattern_route))
        .route("/patterns/tweezer",                    post(tweezer_top_bottom_route))
        .route("/patterns/hikkake",                    post(hikkake_pattern_route))
        .route("/patterns/sperandeo-1-2-3",            post(sperandeo_1_2_3_route))
        .route("/analytics/breadth-thrust",            post(breadth_thrust_route))
        .route("/patterns/abandoned-baby",             post(abandoned_baby_route))
        .route("/patterns/spinning-top-marubozu",      post(spinning_top_marubozu_route))
        .route("/patterns/kicker",                     post(kicker_pattern_route))
        .route("/patterns/three-inside-up-down",       post(three_inside_up_down_route))
        .route("/patterns/three-outside-up-down",      post(three_outside_up_down_route))
        .route("/patterns/belt-hold",                  post(belt_hold_pattern_route))
        .route("/analytics/gap-classifier",            post(gap_classifier_route))
        .route("/analytics/linear-regression-r-squared", post(linear_regression_r_squared_route))
        .route("/analytics/price-volume-oscillator",   post(price_volume_oscillator_route))
        .route("/patterns/rising-falling-three-methods", post(rising_falling_three_methods_route))
        .route("/patterns/mat-hold",                   post(mat_hold_pattern_route))
        .route("/patterns/on-neck-in-neck",            post(on_neck_in_neck_route))
        .route("/patterns/tasuki-gap",                 post(tasuki_gap_route))
        .route("/patterns/breakaway",                  post(breakaway_pattern_route))
        .route("/patterns/volume-climax",              post(volume_climax_pattern_route))
        .route("/patterns/ladder-bottom-top",          post(ladder_bottom_top_route))
        .route("/patterns/counter-attack",             post(counter_attack_lines_route))
        .route("/patterns/thrusting",                  post(thrusting_pattern_route))
        .route("/patterns/stick-sandwich",             post(stick_sandwich_route))
        .route("/patterns/side-by-side-white-lines",   post(side_by_side_white_lines_route))
        .route("/patterns/upside-gap-two-crows",       post(upside_gap_two_crows_route))
        .route("/patterns/swing-failure",              post(swing_failure_pattern_route))
        .route("/patterns/ross-hook",                  post(ross_hook_route))
        .route("/patterns/separating-lines",           post(separating_lines_route))
        .route("/charts/kagi",                         post(kagi_chart_route))
        .route("/charts/three-line-break",             post(three_line_break_route))
        .route("/patterns/meeting-lines",              post(meeting_lines_route))
        .route("/charts/volume-at-price",              post(volume_at_price_route))
        .route("/charts/equivolume-bars",              post(equivolume_bars_route))
        .route("/analytics/ehrlich-filter",            post(ehrlich_filter_route))
        .route("/analytics/belkhayate-timing",         post(belkhayate_timing_route))
        .route("/charts/range-bar",                    post(range_bar_chart_route))
        .route("/charts/tick-bar",                     post(tick_bar_chart_route))
        .route("/charts/volume-bar",                   post(volume_bar_chart_route))
        .route("/charts/dollar-bar",                   post(dollar_bar_chart_route))
        .route("/options/volatility-smile",            post(volatility_smile_route))
        .route("/analytics/monthly-seasonality",       post(monthly_seasonality_route))
        .route("/analytics/intraday-seasonality",      post(intraday_seasonality_route))
        .route("/analytics/holiday-seasonality",       post(holiday_seasonality_route))
        .route("/analytics/day-of-week-seasonality",   post(day_of_week_seasonality_route))
        .route("/analytics/developing-value-area",     post(developing_value_area_route))
        .route("/analytics/developing-poc",            post(developing_poc_route))
        .route("/options/oi-distribution",             post(option_open_interest_distribution_route))
        .route("/analytics/moving-correlation",        post(moving_correlation_route))
        .route("/charts/imbalance-bar",                post(imbalance_bar_chart_route))
        .route("/patterns/pin-bar",                    post(pin_bar_route))
        .route("/analytics/atr-trailing-stop",         post(atr_trailing_stop_route))
        .route("/analytics/atr-channel",               post(atr_channel_route))
        .route("/analytics/relative-volume-zscore",    post(relative_volume_zscore_route))
        .route("/patterns/power-bar",                  post(power_bar_route))
        .route("/analytics/bollinger-band-distance",   post(bollinger_band_distance_route))
        .route("/analytics/volume-force-index",        post(volume_force_index_route))
        .route("/analytics/zero-lag-macd",             post(zero_lag_macd_route))
        .route("/analytics/compound-pivots",           post(compound_pivots_route))
        .route("/analytics/ergodic-oscillator",        post(ergodic_oscillator_route))
        .route("/analytics/momentum-quality-score",    post(momentum_quality_score_route))
        .route("/analytics/swing-strength-index",      post(swing_strength_index_route))
        .route("/analytics/momentum-persistence-index", post(momentum_persistence_index_route))
        .route("/analytics/cumulative-tick-zscore",    post(cumulative_tick_zscore_route))
        .route("/analytics/regime-classifier",         post(regime_classifier_route))
        .route("/analytics/bayesian-change-point",     post(bayesian_change_point_detector_route))
        .route("/analytics/dividend-yield-curve",      post(dividend_yield_curve_route))
        .route("/analytics/repo-rate-spread",          post(repo_rate_spread_route))
        .route("/analytics/term-premium",              post(term_premium_estimator_route))
        .route("/analytics/swap-rate-curve",           post(swap_rate_curve_route))
        .route("/analytics/rolling-quantile",          post(rolling_quantile_route))
        .route("/analytics/option-payoff-diagram",     post(option_payoff_diagram_route))
        .route("/analytics/var-historical",            post(value_at_risk_historical_route))
        .route("/analytics/var-filtered-historical",   post(value_at_risk_filtered_historical_route))
        .route("/analytics/kalman-dynamic-beta",       post(kalman_dynamic_beta_route))
        .route("/analytics/gbm-simulator",             post(gbm_path_simulator_route))
        .route("/analytics/jump-diffusion-simulator",  post(jump_diffusion_simulator_route))
        .route("/analytics/fbm-generator",             post(fractional_brownian_motion_generator_route))
        .route("/analytics/multi-leg-option-pricer",   post(multi_leg_option_pricer_route))
        .route("/analytics/markov-switching-2state",   post(markov_switching_2state_route))
        .route("/analytics/microprice-stoikov",        post(microprice_stoikov_route))
        .route("/analytics/svi-volatility-smile",      post(svi_volatility_smile_route))
        .route("/analytics/dupire-local-volatility",   post(dupire_local_volatility_route))
        .route("/analytics/dynamic-time-warping",      post(dynamic_time_warping_route))
        .route("/analytics/wavelet-decomposition-haar", post(wavelet_decomposition_haar_route))
}

// ──────────────────────────────────────────────────────────────────────
// Discipline
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct LossLimitBody {
    today_pnl: Decimal,
    config: daily_loss_limit::LossLimitConfig,
}

async fn daily_loss_limit_route(
    _u: AuthUser, Json(b): Json<LossLimitBody>,
) -> Json<daily_loss_limit::LossLimitReport> {
    Json(daily_loss_limit::evaluate(b.today_pnl, &b.config))
}

#[derive(Deserialize)]
struct DdThrottleBody {
    equity_history: Vec<f64>,
    config: drawdown_throttle::ThrottleConfig,
}

async fn drawdown_throttle_route(
    _u: AuthUser, Json(b): Json<DdThrottleBody>,
) -> Json<drawdown_throttle::ThrottleReport> {
    Json(drawdown_throttle::evaluate(&b.equity_history, &b.config))
}

#[derive(Deserialize)]
struct GoalTrackerBody {
    goals: goal_tracker::Goals,
    equity_history: Vec<f64>,
    today: NaiveDate,
}

async fn goal_tracker_route(
    _u: AuthUser, Json(b): Json<GoalTrackerBody>,
) -> Json<goal_tracker::ProgressReport> {
    Json(goal_tracker::evaluate(&b.goals, &b.equity_history, b.today))
}

// ──────────────────────────────────────────────────────────────────────
// Options margin
// ──────────────────────────────────────────────────────────────────────

async fn margin_naked_short_route(
    _u: AuthUser, Json(opt): Json<options_margin::NakedShortOption>,
) -> Json<options_margin::MarginReport> {
    Json(options_margin::naked_short(&opt))
}

async fn margin_vertical_route(
    _u: AuthUser, Json(spread): Json<options_margin::VerticalSpread>,
) -> Json<options_margin::MarginReport> {
    Json(options_margin::vertical(&spread))
}

// ──────────────────────────────────────────────────────────────────────
// Portfolio
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct PositionAgingBody {
    positions: Vec<position_aging::OpenPosition>,
    now: DateTime<Utc>,
    /// Position is flagged as stale after holding this many days.
    stale_threshold_days: i64,
}

async fn position_aging_route(
    _u: AuthUser, Json(b): Json<PositionAgingBody>,
) -> Json<position_aging::AgingReport> {
    Json(position_aging::evaluate(&b.positions, b.now, b.stale_threshold_days))
}

#[derive(Deserialize)]
struct PositionIrrBody { flows: Vec<position_irr::CashFlow> }

#[derive(Serialize)]
struct PositionIrrResp { irr: Option<f64> }

async fn position_irr_route(
    _u: AuthUser, Json(b): Json<PositionIrrBody>,
) -> Json<PositionIrrResp> {
    Json(PositionIrrResp { irr: position_irr::annualized_irr(&b.flows) })
}

// ──────────────────────────────────────────────────────────────────────
// Sentiment
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct PutCallRatioBody {
    input: put_call_ratio::PutCallInput,
    thresholds: put_call_ratio::Thresholds,
}

async fn put_call_ratio_route(
    _u: AuthUser, Json(b): Json<PutCallRatioBody>,
) -> Json<put_call_ratio::PutCallReport> {
    Json(put_call_ratio::compute(&b.input, &b.thresholds))
}

// ──────────────────────────────────────────────────────────────────────
// Tax reconciliation
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct Reconcile1099bBody {
    year: i32,
    trades: Vec<Trade>,
    rows: Vec<reconcile_1099b::B1099Row>,
}

async fn reconcile_1099b_route(
    _u: AuthUser, Json(b): Json<Reconcile1099bBody>,
) -> Json<reconcile_1099b::ReconReport> {
    Json(reconcile_1099b::reconcile(b.year, &b.trades, &b.rows))
}

// ──────────────────────────────────────────────────────────────────────
// Risk:reward
// ──────────────────────────────────────────────────────────────────────

async fn risk_reward_route(
    _u: AuthUser, Json(input): Json<risk_reward::RrInput>,
) -> Result<Json<risk_reward::RrReport>, ApiError> {
    risk_reward::compute(&input)
        .map(Json)
        .map_err(|e| ApiError::BadRequest(e.into()))
}

// ──────────────────────────────────────────────────────────────────────
// Rolling window analytics
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RollingZBody { series: Vec<f64>, window: usize }

async fn rolling_zscore_route(
    _u: AuthUser, Json(b): Json<RollingZBody>,
) -> Json<Vec<rolling_zscore::ZPoint>> {
    Json(rolling_zscore::compute(&b.series, b.window))
}

// ──────────────────────────────────────────────────────────────────────
// Strategy + spread analytics
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct StrategyCorrelationBody {
    strategies: Vec<strategy_correlation::StrategyReturns>,
    high_threshold: f64,
}

async fn strategy_correlation_route(
    _u: AuthUser, Json(b): Json<StrategyCorrelationBody>,
) -> Json<strategy_correlation::CorrReport> {
    Json(strategy_correlation::analyze(&b.strategies, b.high_threshold))
}

async fn spread_attribution_route(
    _u: AuthUser, Json(t): Json<spread_attribution::PairTrade>,
) -> Json<spread_attribution::AttributionReport> {
    Json(spread_attribution::attribute(&t))
}

#[derive(Deserialize)]
struct PairTradeBody {
    /// y-leg price series (regression dependent).
    y: Vec<f64>,
    /// x-leg price series (regression independent).
    x: Vec<f64>,
    config: pair_trade::PairConfig,
}

async fn pair_trade_signal_route(
    _u: AuthUser, Json(b): Json<PairTradeBody>,
) -> Result<Json<pair_trade::PairReport>, ApiError> {
    pair_trade::analyze(&b.y, &b.x, &b.config)
        .ok_or_else(|| ApiError::BadRequest(
            "y and x must be the same length, at least 3 long, with non-zero x-variance".into()
        ))
        .map(Json)
}

// ──────────────────────────────────────────────────────────────────────
// Decision systems
// ──────────────────────────────────────────────────────────────────────

async fn triple_screen_route(
    _u: AuthUser, Json(input): Json<triple_screen::TripleScreenInput>,
) -> Json<TripleScreenResp> {
    Json(TripleScreenResp { verdict: triple_screen::evaluate(&input) })
}

#[derive(Serialize)]
struct TripleScreenResp { verdict: triple_screen::Verdict }

// ──────────────────────────────────────────────────────────────────────
// TWAP TCA
// ──────────────────────────────────────────────────────────────────────

async fn twap_route(
    _u: AuthUser, Json(input): Json<twap::TwapInput>,
) -> Json<TwapResp> {
    Json(TwapResp { result: twap::compute(&input) })
}

#[derive(Serialize)]
struct TwapResp { result: Option<twap::TwapResult> }

// ──────────────────────────────────────────────────────────────────────
// Volatility-based stops
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ChandelierBody {
    bars: Vec<volatility_stop::Bar>,
    atr: Vec<f64>,
    side: TradeSide,
    config: volatility_stop::StopConfig,
}

async fn chandelier_stop_route(
    _u: AuthUser, Json(b): Json<ChandelierBody>,
) -> Json<Vec<volatility_stop::StopPoint>> {
    Json(volatility_stop::chandelier(&b.bars, &b.atr, b.side, &b.config))
}

async fn vol_stop_close_route(
    _u: AuthUser, Json(b): Json<ChandelierBody>,
) -> Json<Vec<volatility_stop::StopPoint>> {
    Json(volatility_stop::vol_stop_close(&b.bars, &b.atr, b.side, &b.config))
}

// ──────────────────────────────────────────────────────────────────────
// Broker reconciliation
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct MtmReconciliationBody {
    broker: Vec<mtm_reconciliation::BrokerPosition>,
    internal: Vec<mtm_reconciliation::InternalPosition>,
    threshold_dollars: Decimal,
}

async fn mtm_reconciliation_route(
    _u: AuthUser, Json(b): Json<MtmReconciliationBody>,
) -> Json<mtm_reconciliation::ReconciliationReport> {
    Json(mtm_reconciliation::reconcile(&b.broker, &b.internal, b.threshold_dollars))
}

// ──────────────────────────────────────────────────────────────────────
// ATR cone projection
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct AtrConeBody { entry: f64, daily_atr: f64, horizon_days: usize }

async fn atr_cone_route(
    _u: AuthUser, Json(b): Json<AtrConeBody>,
) -> Json<Vec<atr_cone::ConePoint>> {
    Json(atr_cone::project(b.entry, b.daily_atr, b.horizon_days))
}

// ──────────────────────────────────────────────────────────────────────
// Alligator
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct AlligatorBody { bars: Vec<alligator::Bar> }

async fn alligator_route(
    _u: AuthUser, Json(b): Json<AlligatorBody>,
) -> Json<Vec<alligator::AlligatorPoint>> {
    Json(alligator::compute(&b.bars))
}

// ──────────────────────────────────────────────────────────────────────
// Calendar helpers
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct DateBody { date: NaiveDate }

#[derive(Serialize)]
struct BoolResp { value: bool }

#[derive(Serialize)]
struct DateResp { date: NaiveDate }

#[derive(Serialize)]
struct CountResp { count: i32 }

async fn is_trading_day_route(_u: AuthUser, Json(b): Json<DateBody>) -> Json<BoolResp> {
    Json(BoolResp { value: holiday_calendar::is_trading_day(b.date) })
}

async fn next_trading_day_route(_u: AuthUser, Json(b): Json<DateBody>) -> Json<DateResp> {
    Json(DateResp { date: holiday_calendar::next_trading_day(b.date) })
}

async fn prior_trading_day_route(_u: AuthUser, Json(b): Json<DateBody>) -> Json<DateResp> {
    Json(DateResp { date: holiday_calendar::prior_trading_day(b.date) })
}

#[derive(Deserialize)]
struct AddTradingDaysBody { date: NaiveDate, days: i32 }

async fn add_trading_days_route(
    _u: AuthUser, Json(b): Json<AddTradingDaysBody>,
) -> Json<DateResp> {
    Json(DateResp { date: holiday_calendar::add_trading_days(b.date, b.days) })
}

#[derive(Deserialize)]
struct TradingDaysBetweenBody { start: NaiveDate, end: NaiveDate }

async fn trading_days_between_route(
    _u: AuthUser, Json(b): Json<TradingDaysBetweenBody>,
) -> Json<CountResp> {
    Json(CountResp { count: holiday_calendar::trading_days_between(b.start, b.end) })
}

#[derive(Deserialize)]
struct EarningsWindowBody {
    events: Vec<earnings_calendar::EarningsEvent>,
    today: NaiveDate,
    /// Look-ahead window in days when checking whether earnings fall inside
    /// the trade-holding period.
    hold_days: i64,
}

async fn earnings_window_route(
    _u: AuthUser, Json(b): Json<EarningsWindowBody>,
) -> Json<Vec<String>> {
    Json(earnings_calendar::earnings_within_window(&b.events, b.today, b.hold_days))
}

#[derive(Deserialize)]
struct EarningsAnalysisBody {
    events: Vec<earnings_calendar::EarningsEvent>,
    today: NaiveDate,
}

async fn earnings_analysis_route(
    _u: AuthUser, Json(b): Json<EarningsAnalysisBody>,
) -> Json<earnings_calendar::EarningsReport> {
    Json(earnings_calendar::analyze(&b.events, b.today))
}

// ──────────────────────────────────────────────────────────────────────
// Symbol filter
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct SymbolFilterBody {
    filter: symbol_filter::SymbolFilter,
    symbol: String,
}

#[derive(Serialize)]
struct SymbolFilterResp { decision: symbol_filter::FilterDecision }

async fn symbol_filter_route(
    _u: AuthUser, Json(b): Json<SymbolFilterBody>,
) -> Json<SymbolFilterResp> {
    Json(SymbolFilterResp { decision: b.filter.check(&b.symbol) })
}

// ──────────────────────────────────────────────────────────────────────
// Futures roll schedule — surfaces contracts approaching expiration so
// the trader can roll forward before liquidity dries up. tastytrade /
// IBKR / NinjaTrader-class feature.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct FuturesRollBody {
    positions: Vec<futures_roll::FuturesPosition>,
    today: NaiveDate,
    /// Days-out window to surface upcoming rolls.
    roll_window_days: i64,
}

async fn futures_roll_route(
    _u: AuthUser, Json(b): Json<FuturesRollBody>,
) -> Json<futures_roll::RollReport> {
    Json(futures_roll::schedule(&b.positions, b.today, b.roll_window_days))
}

// ──────────────────────────────────────────────────────────────────────
// SIP/DRIP simulator — eToro/Robinhood/Coinbase recurring-deposit math.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct SipBody {
    bars: Vec<sip_simulator::PriceBar>,
    spec: sip_simulator::ScheduleSpec,
}

async fn sip_simulator_route(
    _u: AuthUser, Json(b): Json<SipBody>,
) -> Json<sip_simulator::SipReport> {
    Json(sip_simulator::simulate(&b.bars, &b.spec))
}

// ──────────────────────────────────────────────────────────────────────
// Portfolio heat — correlated-position budget enforcement.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct PortfolioHeatBody {
    open_positions: Vec<portfolio_heat::OpenRiskPosition>,
    correlations: Vec<portfolio_heat::CorrEdge>,
    candidate: portfolio_heat::CandidateTrade,
    config: portfolio_heat::HeatConfig,
}

async fn portfolio_heat_route(
    _u: AuthUser, Json(b): Json<PortfolioHeatBody>,
) -> Json<portfolio_heat::HeatReport> {
    Json(portfolio_heat::evaluate(&b.open_positions, &b.correlations, &b.candidate, &b.config))
}

// ──────────────────────────────────────────────────────────────────────
// Tax-lot optimizer — HIFO / Lifoust / MaxLossHarvest selection.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct TaxLotOptimizerBody {
    lots: Vec<tax_lot_optimizer::CostLot>,
    qty_to_close: Decimal,
    sell_price: Decimal,
    strategy: tax_lot_optimizer::LotStrategy,
}

async fn tax_lot_optimizer_route(
    _u: AuthUser, Json(b): Json<TaxLotOptimizerBody>,
) -> Json<tax_lot_optimizer::CloseReport> {
    Json(tax_lot_optimizer::close(&b.lots, b.qty_to_close, b.sell_price, b.strategy))
}

// ──────────────────────────────────────────────────────────────────────
// Volume burst + round levels + timeframe confluence.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct VolumeBurstBody {
    bars: Vec<volume_burst::VolumeBar>,
    config: volume_burst::BurstConfig,
}

async fn volume_burst_route(
    _u: AuthUser, Json(b): Json<VolumeBurstBody>,
) -> Json<volume_burst::BurstReport> {
    Json(volume_burst::detect(&b.bars, &b.config))
}

#[derive(Deserialize)]
struct RoundLevelsBody {
    current_price: f64,
    /// Optional ATR for distance-in-ATRs annotations.
    atr: Option<f64>,
    config: round_levels::LevelsConfig,
}

async fn round_levels_route(
    _u: AuthUser, Json(b): Json<RoundLevelsBody>,
) -> Json<round_levels::LevelsReport> {
    Json(round_levels::detect(b.current_price, b.atr, &b.config))
}

#[derive(Deserialize)]
struct ConfluenceBody { verdicts: Vec<timeframe_confluence::TimeframeVerdict> }

async fn timeframe_confluence_route(
    _u: AuthUser, Json(b): Json<ConfluenceBody>,
) -> Json<timeframe_confluence::ConfluenceReport> {
    Json(timeframe_confluence::analyze(&b.verdicts))
}

// ──────────────────────────────────────────────────────────────────────
// Crossover + breakout + range-contraction primitives.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CrossoverBody { a: Vec<Option<f64>>, b: Vec<Option<f64>> }

async fn crossover_route(
    _u: AuthUser, Json(body): Json<CrossoverBody>,
) -> Json<crossover::CrossReport> {
    Json(crossover::detect(&body.a, &body.b))
}

#[derive(Deserialize)]
struct BreakoutBody {
    bars: Vec<breakout_detector::OhlcBar>,
    config: breakout_detector::BreakoutConfig,
}

async fn breakout_route(
    _u: AuthUser, Json(b): Json<BreakoutBody>,
) -> Json<breakout_detector::BreakoutReport> {
    Json(breakout_detector::detect(&b.bars, &b.config))
}

#[derive(Deserialize)]
struct RangeContractionBody { bars: Vec<range_contraction::OhlcBar> }

async fn range_contraction_route(
    _u: AuthUser, Json(b): Json<RangeContractionBody>,
) -> Json<range_contraction::PatternReport> {
    Json(range_contraction::detect(&b.bars))
}

// ──────────────────────────────────────────────────────────────────────
// Smart-money concepts: stop hunt + fair value gap + order block.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct StopHuntBody {
    bars: Vec<stop_hunt::OhlcBar>,
    config: stop_hunt::StopHuntConfig,
}

async fn stop_hunt_route(
    _u: AuthUser, Json(b): Json<StopHuntBody>,
) -> Json<stop_hunt::SweepReport> {
    Json(stop_hunt::detect(&b.bars, &b.config))
}

#[derive(Deserialize)]
struct FairValueGapBody { bars: Vec<fair_value_gap::OhlcBar> }

async fn fair_value_gap_route(
    _u: AuthUser, Json(b): Json<FairValueGapBody>,
) -> Json<fair_value_gap::FvgReport> {
    Json(fair_value_gap::detect(&b.bars))
}

#[derive(Deserialize)]
struct OrderBlockBody {
    bars: Vec<order_block::OhlcBar>,
    config: order_block::OrderBlockConfig,
}

async fn order_block_route(
    _u: AuthUser, Json(b): Json<OrderBlockBody>,
) -> Json<order_block::OrderBlockReport> {
    Json(order_block::detect(&b.bars, &b.config))
}

// ──────────────────────────────────────────────────────────────────────
// Break of Structure + Change of Character + Equal levels.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct BosBody {
    closes: Vec<f64>,
    swings: Vec<swing_points::SwingPoint>,
}

async fn break_of_structure_route(
    _u: AuthUser, Json(b): Json<BosBody>,
) -> Json<break_of_structure::BosReport> {
    Json(break_of_structure::detect(&b.closes, &b.swings))
}

#[derive(Deserialize)]
struct ChochBody {
    closes: Vec<f64>,
    swings: Vec<swing_points::SwingPoint>,
    initial_trend: change_of_character::TrendDirection,
}

async fn change_of_character_route(
    _u: AuthUser, Json(b): Json<ChochBody>,
) -> Json<change_of_character::ChochReport> {
    Json(change_of_character::detect(&b.closes, &b.swings, b.initial_trend))
}

#[derive(Deserialize)]
struct EqualLevelsBody {
    swings: Vec<swing_points::SwingPoint>,
    config: equal_levels::EqualLevelsConfig,
}

async fn equal_levels_route(
    _u: AuthUser, Json(b): Json<EqualLevelsBody>,
) -> Json<equal_levels::LevelsReport> {
    Json(equal_levels::detect(&b.swings, &b.config))
}

// ──────────────────────────────────────────────────────────────────────
// Cumulative delta + displacement + ORB.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CumulativeDeltaBody { ticks: Vec<cumulative_delta::TickWithPrice> }

async fn cumulative_delta_route(
    _u: AuthUser, Json(b): Json<CumulativeDeltaBody>,
) -> Json<cumulative_delta::CdReport> {
    Json(cumulative_delta::compute(&b.ticks))
}

#[derive(Deserialize)]
struct DisplacementBody {
    bars: Vec<displacement::OhlcBar>,
    atr: Vec<f64>,
    config: displacement::DisplacementConfig,
}

async fn displacement_route(
    _u: AuthUser, Json(b): Json<DisplacementBody>,
) -> Json<displacement::DisplacementReport> {
    Json(displacement::detect(&b.bars, &b.atr, &b.config))
}

#[derive(Deserialize)]
struct OrbBody {
    bars: Vec<opening_range::OhlcBar>,
    config: opening_range::OrbConfig,
}

async fn opening_range_route(
    _u: AuthUser, Json(b): Json<OrbBody>,
) -> Json<opening_range::OrbReport> {
    Json(opening_range::detect(&b.bars, &b.config))
}

// ──────────────────────────────────────────────────────────────────────
// VSA + Ulcer Index + Calmar ratio.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct VsaBody {
    bars: Vec<vsa::VsaBar>,
    avg_volume: Vec<f64>,
}

async fn vsa_route(
    _u: AuthUser, Json(b): Json<VsaBody>,
) -> Json<vsa::VsaReport> {
    Json(vsa::classify(&b.bars, &b.avg_volume))
}

#[derive(Deserialize)]
struct UlcerBody {
    equity: Vec<f64>,
    /// Optional annual return (in %) for Ulcer Performance Index calc.
    risk_free_rate: Option<f64>,
}

async fn ulcer_index_route(
    _u: AuthUser, Json(b): Json<UlcerBody>,
) -> Json<ulcer_index::UlcerReport> {
    Json(ulcer_index::compute(&b.equity, b.risk_free_rate))
}

#[derive(Deserialize)]
struct CalmarBody {
    equity: Vec<f64>,
    years: f64,
}

async fn calmar_ratio_route(
    _u: AuthUser, Json(b): Json<CalmarBody>,
) -> Json<calmar_ratio::CalmarReport> {
    Json(calmar_ratio::compute(&b.equity, b.years))
}

// ──────────────────────────────────────────────────────────────────────
// Wyckoff + premium/discount + CUSUM.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct WyckoffBody {
    closes: Vec<f64>,
    config: wyckoff::WyckoffConfig,
}

async fn wyckoff_route(
    _u: AuthUser, Json(b): Json<WyckoffBody>,
) -> Json<wyckoff::WyckoffReport> {
    Json(wyckoff::classify(&b.closes, &b.config))
}

#[derive(Deserialize)]
struct PremiumDiscountBody {
    range_high: f64,
    range_low: f64,
    price: f64,
    trend: premium_discount::TrendBias,
}

async fn premium_discount_route(
    _u: AuthUser, Json(b): Json<PremiumDiscountBody>,
) -> Json<premium_discount::ZoneReport> {
    Json(premium_discount::classify(b.range_high, b.range_low, b.price, b.trend))
}

#[derive(Deserialize)]
struct CusumBody {
    series: Vec<f64>,
    config: cusum::CusumConfig,
}

async fn cusum_route(
    _u: AuthUser, Json(b): Json<CusumBody>,
) -> Json<cusum::CusumReport> {
    Json(cusum::detect(&b.series, &b.config))
}

// ──────────────────────────────────────────────────────────────────────
// HA reversal + three-bar reversal + range expansion.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct HaReversalBody {
    bars: Vec<traderview_core::heikin_ashi::HaBar>,
    config: heikin_ashi_reversal::FlipConfig,
}

async fn heikin_ashi_reversal_route(
    _u: AuthUser, Json(b): Json<HaReversalBody>,
) -> Json<heikin_ashi_reversal::FlipReport> {
    Json(heikin_ashi_reversal::detect(&b.bars, &b.config))
}

#[derive(Deserialize)]
struct ThreeBarReversalBody { bars: Vec<three_bar_reversal::OhlcBar> }

async fn three_bar_reversal_route(
    _u: AuthUser, Json(b): Json<ThreeBarReversalBody>,
) -> Json<three_bar_reversal::ReversalReport> {
    Json(three_bar_reversal::detect(&b.bars))
}

#[derive(Deserialize)]
struct RangeExpansionBody {
    bars: Vec<range_expansion::OhlcBar>,
    atr: Vec<f64>,
    config: range_expansion::ExpansionConfig,
}

async fn range_expansion_route(
    _u: AuthUser, Json(b): Json<RangeExpansionBody>,
) -> Json<range_expansion::ExpansionReport> {
    Json(range_expansion::detect(&b.bars, &b.atr, &b.config))
}

// ──────────────────────────────────────────────────────────────────────
// Trend-efficiency primitives: Choppiness + KER + RWI.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ChoppinessBody {
    bars: Vec<choppiness::OhlcBar>,
    period: usize,
}

async fn choppiness_route(
    _u: AuthUser, Json(b): Json<ChoppinessBody>,
) -> Json<choppiness::ChopReport> {
    Json(choppiness::compute(&b.bars, b.period))
}

#[derive(Deserialize)]
struct EfficiencyBody {
    closes: Vec<f64>,
    lookback: usize,
}

async fn efficiency_ratio_route(
    _u: AuthUser, Json(b): Json<EfficiencyBody>,
) -> Json<efficiency_ratio::EfficiencyReport> {
    Json(efficiency_ratio::compute(&b.closes, b.lookback))
}

#[derive(Deserialize)]
struct RwiBody {
    bars: Vec<random_walk_index::OhlcBar>,
    max_n: usize,
}

async fn random_walk_index_route(
    _u: AuthUser, Json(b): Json<RwiBody>,
) -> Json<random_walk_index::RwiReport> {
    Json(random_walk_index::compute(&b.bars, b.max_n))
}

// ──────────────────────────────────────────────────────────────────────
// Bill Williams AC + ICT liquidity grab + gap-fill statistics.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct AcBody {
    bars: Vec<acceleration_deceleration::HlBar>,
}

async fn ac_route(
    _u: AuthUser, Json(b): Json<AcBody>,
) -> Json<acceleration_deceleration::AcReport> {
    Json(acceleration_deceleration::compute(&b.bars))
}

#[derive(Deserialize)]
struct LiquidityGrabBody {
    bars: Vec<liquidity_grab::OhlcBar>,
    atr: Vec<f64>,
    swings: Vec<swing_points::SwingPoint>,
    #[serde(default)]
    config: liquidity_grab::GrabConfig,
}

async fn liquidity_grab_route(
    _u: AuthUser, Json(b): Json<LiquidityGrabBody>,
) -> Json<liquidity_grab::GrabReport> {
    Json(liquidity_grab::detect(&b.bars, &b.atr, &b.swings, &b.config))
}

#[derive(Deserialize)]
struct GapFillBody {
    bars: Vec<gap_fill_stats::OhlcBar>,
    atr: Vec<f64>,
}

async fn gap_fill_stats_route(
    _u: AuthUser, Json(b): Json<GapFillBody>,
) -> Json<gap_fill_stats::GapStatsReport> {
    Json(gap_fill_stats::analyze(&b.bars, &b.atr))
}

// ──────────────────────────────────────────────────────────────────────
// Market breadth (TRIN, McClellan) + inside-bar breakout pattern.
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ArmsBody {
    bars: Vec<arms_index::BreadthBar>,
}

async fn arms_index_route(
    _u: AuthUser, Json(b): Json<ArmsBody>,
) -> Json<arms_index::TrinReport> {
    Json(arms_index::compute(&b.bars))
}

#[derive(Deserialize)]
struct McClellanBody {
    bars: Vec<mcclellan_oscillator::BreadthBar>,
}

async fn mcclellan_oscillator_route(
    _u: AuthUser, Json(b): Json<McClellanBody>,
) -> Json<mcclellan_oscillator::McClellanReport> {
    Json(mcclellan_oscillator::compute(&b.bars))
}

#[derive(Deserialize)]
struct InsideBarBody {
    bars: Vec<inside_bar_breakout::OhlcBar>,
    #[serde(default)]
    config: inside_bar_breakout::IbConfig,
}

async fn inside_bar_breakout_route(
    _u: AuthUser, Json(b): Json<InsideBarBody>,
) -> Json<inside_bar_breakout::IbReport> {
    Json(inside_bar_breakout::detect(&b.bars, &b.config))
}

// ──────────────────────────────────────────────────────────────────────
// Pattern detectors
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct HarmonicBody {
    swings: Vec<swing_points::SwingPoint>,
    #[serde(default)]
    config: harmonic_patterns::DetectorConfig,
}

async fn harmonic_patterns_route(
    _u: AuthUser, Json(b): Json<HarmonicBody>,
) -> Json<harmonic_patterns::HarmonicReport> {
    Json(harmonic_patterns::detect(&b.swings, &b.config))
}

#[derive(Deserialize)]
struct AbcBody {
    swings: Vec<swing_points::SwingPoint>,
    #[serde(default)]
    config: abc_pattern::AbcConfig,
}

async fn abc_pattern_route(
    _u: AuthUser, Json(b): Json<AbcBody>,
) -> Json<abc_pattern::AbcReport> {
    Json(abc_pattern::detect(&b.swings, &b.config))
}

#[derive(Deserialize)]
struct ThreeDriveBody {
    swings: Vec<swing_points::SwingPoint>,
    #[serde(default)]
    config: three_drive_pattern::ThreeDriveConfig,
}

async fn three_drive_pattern_route(
    _u: AuthUser, Json(b): Json<ThreeDriveBody>,
) -> Json<three_drive_pattern::ThreeDriveReport> {
    Json(three_drive_pattern::detect(&b.swings, &b.config))
}

// ──────────────────────────────────────────────────────────────────────
// Order flow + market internals
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct FootprintBody {
    bars: Vec<footprint_imbalance::FootprintBar>,
    #[serde(default)]
    config: footprint_imbalance::ImbalanceConfig,
}

async fn footprint_imbalance_route(
    _u: AuthUser, Json(b): Json<FootprintBody>,
) -> Json<footprint_imbalance::ImbalanceReport> {
    Json(footprint_imbalance::detect(&b.bars, &b.config))
}

#[derive(Deserialize)]
struct TapeDensityBody {
    ticks: Vec<tape_density::Tick>,
    #[serde(default)]
    config: tape_density::DensityConfig,
}

async fn tape_density_route(
    _u: AuthUser, Json(b): Json<TapeDensityBody>,
) -> Json<tape_density::DensityReport> {
    Json(tape_density::analyze(&b.ticks, &b.config))
}

#[derive(Deserialize)]
struct DepthImbalanceBody {
    snapshots: Vec<depth_imbalance::DepthSnapshot>,
    #[serde(default)]
    config: depth_imbalance::DepthConfig,
}

async fn depth_imbalance_route(
    _u: AuthUser, Json(b): Json<DepthImbalanceBody>,
) -> Json<depth_imbalance::DepthReport> {
    Json(depth_imbalance::analyze(&b.snapshots, &b.config))
}

#[derive(Deserialize)]
struct TickExtremeBody {
    tick_series: Vec<f64>,
    #[serde(default)]
    config: tick_extreme::TickConfig,
}

async fn tick_extreme_route(
    _u: AuthUser, Json(b): Json<TickExtremeBody>,
) -> Json<tick_extreme::TickReport> {
    Json(tick_extreme::analyze(&b.tick_series, &b.config))
}

#[derive(Deserialize)]
struct SectorRotationBody {
    returns: Vec<sector_rotation::SectorReturn>,
    #[serde(default)]
    prior_ranks: std::collections::HashMap<String, usize>,
    #[serde(default)]
    config: sector_rotation::RotationConfig,
}

async fn sector_rotation_route(
    _u: AuthUser, Json(b): Json<SectorRotationBody>,
) -> Json<sector_rotation::RotationReport> {
    let prior = if b.prior_ranks.is_empty() { None } else { Some(&b.prior_ranks) };
    Json(sector_rotation::analyze(&b.returns, prior, &b.config))
}

// ──────────────────────────────────────────────────────────────────────
// Options IV-rank universe scanner
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct IvRankScannerBody {
    symbols: Vec<iv_rank_scanner::SymbolIv>,
}

async fn iv_rank_scanner_route(
    _u: AuthUser, Json(b): Json<IvRankScannerBody>,
) -> Json<iv_rank_scanner::IvRankReport> {
    Json(iv_rank_scanner::analyze(&b.symbols))
}

// ──────────────────────────────────────────────────────────────────────
// Universe scanner orchestrator
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ScanOrchestratorBody {
    universe: Vec<(String, Vec<PriceBar>)>,
}

async fn scan_orchestrator_route(
    _u: AuthUser, Json(b): Json<ScanOrchestratorBody>,
) -> Json<scan_orchestrator::UniverseReport> {
    Json(scan_orchestrator::scan_universe(&b.universe))
}

// ──────────────────────────────────────────────────────────────────────
// Backtest sweep + walk-forward
// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct SweepSmaBody {
    bars: Vec<PriceBar>,
    grid: backtest_sweep::SmaCrossGrid,
    #[serde(default = "default_initial_capital")] initial_capital: f64,
    #[serde(default = "default_fee_per_trade")]   fee_per_trade: f64,
}
fn default_initial_capital() -> f64 { 10_000.0 }
fn default_fee_per_trade()   -> f64 { 1.0 }

async fn sweep_sma_cross_route(
    _u: AuthUser, Json(b): Json<SweepSmaBody>,
) -> Json<backtest_sweep::SweepReport> {
    Json(backtest_sweep::sweep_sma_cross(&b.bars, &b.grid, b.initial_capital, b.fee_per_trade))
}

#[derive(Deserialize)]
struct SweepBbBody {
    bars: Vec<PriceBar>,
    grid: backtest_sweep::BbBreakoutGrid,
    #[serde(default = "default_initial_capital")] initial_capital: f64,
    #[serde(default = "default_fee_per_trade")]   fee_per_trade: f64,
}

async fn sweep_bb_breakout_route(
    _u: AuthUser, Json(b): Json<SweepBbBody>,
) -> Json<backtest_sweep::SweepReport> {
    Json(backtest_sweep::sweep_bb_breakout(&b.bars, &b.grid, b.initial_capital, b.fee_per_trade))
}

#[derive(Deserialize)]
struct WalkForwardSmaBody {
    bars: Vec<PriceBar>,
    grid: backtest_sweep::SmaCrossGrid,
    #[serde(default)]
    config: walk_forward::WalkForwardConfig,
}

async fn walk_forward_sma_cross_route(
    _u: AuthUser, Json(b): Json<WalkForwardSmaBody>,
) -> Json<walk_forward::WalkForwardReport> {
    Json(walk_forward::run_sma_cross(&b.bars, &b.grid, &b.config))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct BreadthLinesBody {
    bars: Vec<breadth_lines::BreadthBar>,
}

async fn breadth_lines_route(
    _u: AuthUser, Json(b): Json<BreadthLinesBody>,
) -> Json<breadth_lines::BreadthReport> {
    Json(breadth_lines::compute(&b.bars))
}

#[derive(Deserialize)]
struct DpiBody {
    bars: Vec<dark_pool_index::DarkPoolBar>,
    #[serde(default)]
    config: dark_pool_index::DpiConfig,
}

async fn dark_pool_index_route(
    _u: AuthUser, Json(b): Json<DpiBody>,
) -> Json<dark_pool_index::DpiReport> {
    Json(dark_pool_index::compute(&b.bars, &b.config))
}

#[derive(Deserialize)]
struct PeadBody {
    events: Vec<post_earnings_drift::EarningsEvent>,
    #[serde(default)]
    config: post_earnings_drift::PeadConfig,
}

async fn post_earnings_drift_route(
    _u: AuthUser, Json(b): Json<PeadBody>,
) -> Json<post_earnings_drift::PeadReport> {
    Json(post_earnings_drift::analyze(&b.events, &b.config))
}

#[derive(Deserialize)]
struct ShortInterestBody {
    entries: Vec<short_interest_scanner::ShortInterestEntry>,
    #[serde(default)]
    config: short_interest_scanner::ScannerConfig,
}

async fn short_interest_route(
    _u: AuthUser, Json(b): Json<ShortInterestBody>,
) -> Json<short_interest_scanner::ScannerReport> {
    Json(short_interest_scanner::analyze(&b.entries, &b.config))
}

#[derive(Deserialize)]
struct RelativeStrengthBody {
    universe: Vec<relative_strength::SymbolPrices>,
    benchmark: relative_strength::SymbolPrices,
    #[serde(default)]
    config: relative_strength::RsConfig,
}

async fn relative_strength_route(
    _u: AuthUser, Json(b): Json<RelativeStrengthBody>,
) -> Json<relative_strength::RsReport> {
    Json(relative_strength::analyze(&b.universe, &b.benchmark, &b.config))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct KeltnerSqueezeBody {
    bars: Vec<keltner_squeeze::Bar>,
    #[serde(default)]
    config: keltner_squeeze::SqueezeConfig,
}

async fn keltner_squeeze_route(
    _u: AuthUser, Json(b): Json<KeltnerSqueezeBody>,
) -> Json<keltner_squeeze::SqueezeReport> {
    Json(keltner_squeeze::compute(&b.bars, &b.config))
}

#[derive(Deserialize)]
struct DivergenceBody {
    prices: Vec<f64>,
    indicator: Vec<Option<f64>>,
    #[serde(default = "default_div_lookback")]
    lookback: usize,
}
fn default_div_lookback() -> usize { 5 }

async fn divergence_detect_route(
    _u: AuthUser, Json(b): Json<DivergenceBody>,
) -> Json<Vec<divergence_detector::DivergenceEvent>> {
    Json(divergence_detector::detect(&b.prices, &b.indicator, b.lookback))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CumulativeTickTrinBody {
    samples: Vec<cumulative_tick_trin::Sample>,
    #[serde(default)]
    config: cumulative_tick_trin::Config,
}

async fn cumulative_tick_trin_route(
    _u: AuthUser, Json(b): Json<CumulativeTickTrinBody>,
) -> Json<cumulative_tick_trin::Report> {
    Json(cumulative_tick_trin::compute(&b.samples, &b.config))
}

#[derive(Deserialize)]
struct SummationIndexBody {
    mcclellan: Vec<Option<f64>>,
}

async fn summation_index_route(
    _u: AuthUser, Json(b): Json<SummationIndexBody>,
) -> Json<Vec<Option<f64>>> {
    Json(summation_index::compute(&b.mcclellan))
}

#[derive(Deserialize)]
struct HindenburgBody {
    bars: Vec<hindenburg_omen::DailyBar>,
    #[serde(default)]
    config: hindenburg_omen::Config,
}

async fn hindenburg_omen_route(
    _u: AuthUser, Json(b): Json<HindenburgBody>,
) -> Json<hindenburg_omen::Report> {
    Json(hindenburg_omen::analyze(&b.bars, &b.config))
}

#[derive(Deserialize)]
struct PremarketGapBody {
    snapshots: Vec<premarket_gap_scanner::PremarketSnapshot>,
    #[serde(default)]
    config: premarket_gap_scanner::ScannerConfig,
}

async fn premarket_gap_route(
    _u: AuthUser, Json(b): Json<PremarketGapBody>,
) -> Json<premarket_gap_scanner::ScannerReport> {
    Json(premarket_gap_scanner::scan(&b.snapshots, &b.config))
}

#[derive(Deserialize)]
struct HaltResumeBody {
    events: Vec<halt_resume_monitor::HaltEvent>,
    #[serde(default)]
    config: halt_resume_monitor::Config,
}

async fn halt_resume_route(
    _u: AuthUser, Json(b): Json<HaltResumeBody>,
) -> Json<halt_resume_monitor::MonitorReport> {
    Json(halt_resume_monitor::analyze(&b.events, &b.config))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct SecondOrderGreeksBody {
    spot: f64,
    strike: f64,
    time_to_expiry: f64,
    risk_free: f64,
    dividend_yield: f64,
    sigma: f64,
    kind: second_order_greeks::OptionKind,
}

async fn second_order_greeks_route(
    _u: AuthUser, Json(b): Json<SecondOrderGreeksBody>,
) -> Json<Option<second_order_greeks::Greeks2>> {
    Json(second_order_greeks::compute(
        b.spot, b.strike, b.time_to_expiry, b.risk_free, b.dividend_yield, b.sigma, b.kind,
    ))
}

#[derive(Deserialize)]
struct VpinBody {
    ticks: Vec<vpin::Tick>,
    #[serde(default)]
    config: vpin::Config,
}

async fn vpin_route(
    _u: AuthUser, Json(b): Json<VpinBody>,
) -> Json<vpin::VpinReport> {
    Json(vpin::compute(&b.ticks, &b.config))
}

#[derive(Deserialize)]
struct CupHandleBody {
    bars: Vec<cup_and_handle::Bar>,
    #[serde(default)]
    config: cup_and_handle::Config,
}

async fn cup_and_handle_route(
    _u: AuthUser, Json(b): Json<CupHandleBody>,
) -> Json<Option<cup_and_handle::CupHandleCandidate>> {
    Json(cup_and_handle::detect(&b.bars, &b.config))
}

#[derive(Deserialize)]
struct HeadShouldersBody {
    bars: Vec<head_shoulders::Bar>,
    #[serde(default)]
    config: head_shoulders::Config,
}

async fn head_shoulders_route(
    _u: AuthUser, Json(b): Json<HeadShouldersBody>,
) -> Json<Vec<head_shoulders::HsCandidate>> {
    Json(head_shoulders::detect(&b.bars, &b.config))
}

#[derive(Deserialize)]
struct Breakout52wBody {
    symbols: Vec<breakout_52w_scanner::SymbolSeries>,
    #[serde(default)]
    config: breakout_52w_scanner::Config,
}

async fn breakout_52w_route(
    _u: AuthUser, Json(b): Json<Breakout52wBody>,
) -> Json<breakout_52w_scanner::ScannerReport> {
    Json(breakout_52w_scanner::scan(&b.symbols, &b.config))
}

#[derive(Deserialize)]
struct EwmaVolBody {
    closes: Vec<f64>,
    #[serde(default = "default_ewma_lambda")] lambda: f64,
}
fn default_ewma_lambda() -> f64 { 0.94 }

async fn ewma_volatility_route(
    _u: AuthUser, Json(b): Json<EwmaVolBody>,
) -> Json<Vec<Option<f64>>> {
    Json(ewma_volatility::compute(&b.closes, b.lambda))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CotReportBody {
    entries: Vec<cot_report::WeeklyEntry>,
    #[serde(default)]
    config: cot_report::CotConfig,
}

async fn cot_report_route(
    _u: AuthUser, Json(b): Json<CotReportBody>,
) -> Json<cot_report::CotReport> {
    Json(cot_report::analyze(&b.entries, &b.config))
}

#[derive(Deserialize)]
struct CalendarSpreadBody {
    spread: calendar_spread::CalendarSpread,
    #[serde(default)]
    config: calendar_spread::AnalyzerConfig,
}

async fn calendar_spread_route(
    _u: AuthUser, Json(b): Json<CalendarSpreadBody>,
) -> Json<Option<calendar_spread::CalendarReport>> {
    Json(calendar_spread::analyze(&b.spread, &b.config))
}

#[derive(Deserialize)]
struct IronCondorBody {
    condor: iron_condor::IronCondor,
    /// Optional explicit spot to also compute P&L at expiration.
    spot_at_expiry: Option<f64>,
}

#[derive(Serialize)]
struct IronCondorResponse {
    report: Option<iron_condor::IronCondorReport>,
    pnl_at_spot: Option<f64>,
}

async fn iron_condor_route(
    _u: AuthUser, Json(b): Json<IronCondorBody>,
) -> Json<IronCondorResponse> {
    let report = iron_condor::analyze(&b.condor);
    let pnl_at_spot = b.spot_at_expiry.and_then(|s| iron_condor::pnl_at_expiration(&b.condor, s));
    Json(IronCondorResponse { report, pnl_at_spot })
}

#[derive(Deserialize)]
struct MarginalVarBody {
    portfolio: marginal_var::Portfolio,
    #[serde(default = "default_z_alpha")] z_alpha: f64,
}
fn default_z_alpha() -> f64 { 1.645 }    // 95%

async fn marginal_var_route(
    _u: AuthUser, Json(b): Json<MarginalVarBody>,
) -> Json<Option<marginal_var::VarReport>> {
    Json(marginal_var::analyze(&b.portfolio, b.z_alpha))
}

#[derive(Deserialize)]
struct RealizedVolBody {
    intraday_returns: Vec<Vec<f64>>,
}

async fn realized_volatility_route(
    _u: AuthUser, Json(b): Json<RealizedVolBody>,
) -> Json<Vec<Option<realized_volatility::WindowMetrics>>> {
    Json(realized_volatility::compute(&b.intraday_returns))
}

#[derive(Deserialize)]
struct AmihudBody {
    returns: Vec<f64>,
    dollar_volumes: Vec<f64>,
    #[serde(default = "default_amihud_period")] period: usize,
}
fn default_amihud_period() -> usize { 21 }

async fn amihud_illiquidity_route(
    _u: AuthUser, Json(b): Json<AmihudBody>,
) -> Json<Vec<Option<f64>>> {
    Json(amihud_illiquidity::compute(&b.returns, &b.dollar_volumes, b.period))
}

#[derive(Deserialize)]
struct KylesLambdaBody {
    price_changes: Vec<f64>,
    signed_volumes: Vec<f64>,
    #[serde(default = "default_kyles_window")] window: usize,
}
fn default_kyles_window() -> usize { 30 }

async fn kyles_lambda_route(
    _u: AuthUser, Json(b): Json<KylesLambdaBody>,
) -> Json<Vec<Option<f64>>> {
    Json(kyles_lambda::compute(&b.price_changes, &b.signed_volumes, b.window))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct TpoBody {
    bars: Vec<tpo_profile::Bar>,
    #[serde(default)]
    config: tpo_profile::Config,
}

async fn tpo_profile_route(
    _u: AuthUser, Json(b): Json<TpoBody>,
) -> Json<Option<tpo_profile::TpoReport>> {
    Json(tpo_profile::build(&b.bars, &b.config))
}

#[derive(Deserialize)]
struct OmegaBody {
    returns: Vec<f64>,
    #[serde(default)] threshold: f64,
}

async fn omega_ratio_route(
    _u: AuthUser, Json(b): Json<OmegaBody>,
) -> Json<Option<omega_ratio::OmegaReport>> {
    Json(omega_ratio::compute(&b.returns, b.threshold))
}

#[derive(Deserialize)]
struct HurstBody {
    returns: Vec<f64>,
    #[serde(default = "default_hurst_chunks")] chunk_sizes: Vec<usize>,
}
fn default_hurst_chunks() -> Vec<usize> { vec![10, 20, 50, 100, 250] }

async fn hurst_exponent_route(
    _u: AuthUser, Json(b): Json<HurstBody>,
) -> Json<Option<hurst_exponent::HurstReport>> {
    Json(hurst_exponent::compute(&b.returns, &b.chunk_sizes))
}

#[derive(Deserialize)]
struct Garch11Body {
    log_returns: Vec<f64>,
    params: garch_1_1::Garch11,
    #[serde(default = "default_garch_horizon")] forecast_horizon: usize,
}
fn default_garch_horizon() -> usize { 10 }

async fn garch_1_1_route(
    _u: AuthUser, Json(b): Json<Garch11Body>,
) -> Json<Option<garch_1_1::Garch11Report>> {
    Json(garch_1_1::compute(&b.log_returns, b.params, b.forecast_horizon))
}

#[derive(Deserialize)]
struct CointegrationBody {
    y: Vec<f64>,
    x: Vec<f64>,
    #[serde(default = "default_adf_lags")] adf_lags: usize,
}
fn default_adf_lags() -> usize { 1 }

async fn cointegration_route(
    _u: AuthUser, Json(b): Json<CointegrationBody>,
) -> Json<Option<cointegration::CointegrationReport>> {
    Json(cointegration::test(&b.y, &b.x, b.adf_lags))
}

#[derive(Deserialize)]
struct TreynorMazuyBody {
    portfolio_returns: Vec<f64>,
    market_returns: Vec<f64>,
    risk_free_returns: Vec<f64>,
}

async fn treynor_mazuy_route(
    _u: AuthUser, Json(b): Json<TreynorMazuyBody>,
) -> Json<Option<treynor_mazuy::TmReport>> {
    Json(treynor_mazuy::analyze(&b.portfolio_returns, &b.market_returns, &b.risk_free_returns))
}

#[derive(Deserialize)]
struct OuBody {
    series: Vec<f64>,
}

async fn ornstein_uhlenbeck_route(
    _u: AuthUser, Json(b): Json<OuBody>,
) -> Json<Option<ornstein_uhlenbeck::OuReport>> {
    Json(ornstein_uhlenbeck::fit(&b.series))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RangeVolBody {
    bars: Vec<range_volatility::Bar>,
    #[serde(default = "default_range_vol_period")] period: usize,
}
fn default_range_vol_period() -> usize { 20 }

async fn range_volatility_route(
    _u: AuthUser, Json(b): Json<RangeVolBody>,
) -> Json<range_volatility::RangeVolReport> {
    Json(range_volatility::compute(&b.bars, b.period))
}

#[derive(Deserialize)]
struct RollSpreadBody {
    prices: Vec<f64>,
    #[serde(default = "default_roll_window")] window: usize,
}
fn default_roll_window() -> usize { 50 }

async fn roll_spread_route(
    _u: AuthUser, Json(b): Json<RollSpreadBody>,
) -> Json<Vec<Option<f64>>> {
    Json(roll_spread::compute(&b.prices, b.window))
}

#[derive(Deserialize)]
struct LeeReadyBody {
    trades: Vec<lee_ready::TradeWithQuote>,
    /// Optional volume series for summary aggregation. When provided
    /// (matching length), response includes the classification summary.
    volumes: Option<Vec<f64>>,
}

#[derive(Serialize)]
struct LeeReadyResponse {
    directions: Vec<lee_ready::Direction>,
    summary: Option<lee_ready::ClassificationSummary>,
}

async fn lee_ready_route(
    _u: AuthUser, Json(b): Json<LeeReadyBody>,
) -> Json<LeeReadyResponse> {
    let dirs = lee_ready::classify(&b.trades);
    let summary = b.volumes.as_ref().map(|v| lee_ready::summarize(&b.trades, v, &dirs));
    Json(LeeReadyResponse { directions: dirs, summary })
}

#[derive(Deserialize)]
struct VarSwapBody {
    spot: f64,
    risk_free: f64,
    time_to_expiry: f64,
    quotes: Vec<variance_swap::OptionQuote>,
}

async fn variance_swap_route(
    _u: AuthUser, Json(b): Json<VarSwapBody>,
) -> Json<Option<variance_swap::VarianceSwapReport>> {
    Json(variance_swap::fair_strike(b.spot, b.risk_free, b.time_to_expiry, &b.quotes))
}

#[derive(Deserialize)]
struct TdSequentialBody {
    bars: Vec<td_sequential::Bar>,
    #[serde(default)]
    config: td_sequential::Config,
}

async fn td_sequential_route(
    _u: AuthUser, Json(b): Json<TdSequentialBody>,
) -> Json<td_sequential::TdReport> {
    Json(td_sequential::analyze(&b.bars, &b.config))
}

#[derive(Deserialize)]
struct AndrewsBody {
    p1: andrews_pitchfork::Pivot,
    p2: andrews_pitchfork::Pivot,
    p3: andrews_pitchfork::Pivot,
    xs: Vec<f64>,
}

async fn andrews_pitchfork_route(
    _u: AuthUser, Json(b): Json<AndrewsBody>,
) -> Json<Vec<Option<andrews_pitchfork::PitchforkLines>>> {
    Json(andrews_pitchfork::series(b.p1, b.p2, b.p3, &b.xs))
}

#[derive(Deserialize)]
struct AnchoredMomentumBody {
    closes: Vec<f64>,
    anchor: usize,
    #[serde(default = "default_anchored_momentum_smooth")] smooth_period: usize,
}
fn default_anchored_momentum_smooth() -> usize { 5 }

async fn anchored_momentum_route(
    _u: AuthUser, Json(b): Json<AnchoredMomentumBody>,
) -> Json<Vec<Option<f64>>> {
    Json(anchored_momentum::compute(&b.closes, b.anchor, b.smooth_period))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct InformationRatioBody {
    portfolio_returns: Vec<f64>,
    benchmark_returns: Vec<f64>,
    #[serde(default = "default_periods_per_year")] periods_per_year: f64,
}
fn default_periods_per_year() -> f64 { 252.0 }

async fn information_ratio_route(
    _u: AuthUser, Json(b): Json<InformationRatioBody>,
) -> Json<Option<information_ratio::InformationReport>> {
    Json(information_ratio::compute(&b.portfolio_returns, &b.benchmark_returns, b.periods_per_year))
}

#[derive(Deserialize)]
struct GainPainBody {
    returns: Vec<f64>,
    /// Optional rolling window — if set, returns the rolling series in
    /// `rolling`. The `summary` field is always populated from the full series.
    window: Option<usize>,
}

#[derive(Serialize)]
struct GainPainResponse {
    summary: Option<gain_pain_ratio::GpReport>,
    rolling: Option<Vec<Option<f64>>>,
}

async fn gain_pain_ratio_route(
    _u: AuthUser, Json(b): Json<GainPainBody>,
) -> Json<GainPainResponse> {
    Json(GainPainResponse {
        summary: gain_pain_ratio::compute(&b.returns),
        rolling: b.window.map(|w| gain_pain_ratio::rolling(&b.returns, w)),
    })
}

#[derive(Deserialize)]
struct HenrikssonMertonBody {
    portfolio_returns: Vec<f64>,
    market_returns: Vec<f64>,
    risk_free_returns: Vec<f64>,
}

async fn henriksson_merton_route(
    _u: AuthUser, Json(b): Json<HenrikssonMertonBody>,
) -> Json<Option<henriksson_merton::HmReport>> {
    Json(henriksson_merton::analyze(&b.portfolio_returns, &b.market_returns, &b.risk_free_returns))
}

#[derive(Deserialize)]
struct IvSolverBody {
    market_price: f64,
    spot: f64,
    strike: f64,
    time_to_expiry: f64,
    risk_free: f64,
    dividend_yield: f64,
    kind: iv_solver::OptionKind,
}

async fn iv_solver_route(
    _u: AuthUser, Json(b): Json<IvSolverBody>,
) -> Json<Option<iv_solver::IvReport>> {
    Json(iv_solver::solve(
        b.market_price, b.spot, b.strike, b.time_to_expiry,
        b.risk_free, b.dividend_yield, b.kind,
    ))
}

#[derive(Deserialize)]
struct Black76Body {
    forward: f64,
    strike: f64,
    time_to_expiry: f64,
    risk_free: f64,
    sigma: f64,
    kind: black76::OptionKind,
}

async fn black76_route(
    _u: AuthUser, Json(b): Json<Black76Body>,
) -> Json<Option<black76::Black76Output>> {
    Json(black76::price(b.forward, b.strike, b.time_to_expiry, b.risk_free, b.sigma, b.kind))
}

#[derive(Deserialize)]
struct DeflatedSharpeBody {
    observed_sharpe: f64,
    n_observations: usize,
    skewness: f64,
    kurtosis: f64,
    n_trials: usize,
}

async fn deflated_sharpe_route(
    _u: AuthUser, Json(b): Json<DeflatedSharpeBody>,
) -> Json<Option<deflated_sharpe::DsrReport>> {
    Json(deflated_sharpe::compute(
        b.observed_sharpe, b.n_observations, b.skewness, b.kurtosis, b.n_trials,
    ))
}

#[derive(Deserialize)]
struct MurreyMathBody {
    bars: Vec<murrey_math::Bar>,
    #[serde(default = "default_murrey_lookback")] lookback_bars: usize,
}
fn default_murrey_lookback() -> usize { 64 }

async fn murrey_math_route(
    _u: AuthUser, Json(b): Json<MurreyMathBody>,
) -> Json<Option<murrey_math::MurreyLevels>> {
    Json(murrey_math::compute(&b.bars, b.lookback_bars))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CVaRBody {
    returns: Vec<f64>,
    #[serde(default = "default_cvar_alpha")] alpha: f64,
}
fn default_cvar_alpha() -> f64 { 0.05 }

async fn conditional_var_route(
    _u: AuthUser, Json(b): Json<CVaRBody>,
) -> Json<Option<conditional_var::CvarReport>> {
    Json(conditional_var::compute(&b.returns, b.alpha))
}

#[derive(Deserialize)]
struct Ff3Body {
    portfolio_returns: Vec<f64>,
    market_excess: Vec<f64>,
    smb: Vec<f64>,
    hml: Vec<f64>,
    risk_free: Vec<f64>,
}

async fn fama_french_3_route(
    _u: AuthUser, Json(b): Json<Ff3Body>,
) -> Json<Option<factor_models::Ff3Report>> {
    let inputs = factor_models::Ff3Inputs {
        portfolio_returns: &b.portfolio_returns,
        market_excess: &b.market_excess,
        smb: &b.smb,
        hml: &b.hml,
        risk_free: &b.risk_free,
    };
    Json(factor_models::ff3(&inputs))
}

#[derive(Deserialize)]
struct Carhart4Body {
    portfolio_returns: Vec<f64>,
    market_excess: Vec<f64>,
    smb: Vec<f64>,
    hml: Vec<f64>,
    wml: Vec<f64>,
    risk_free: Vec<f64>,
}

async fn carhart_4_route(
    _u: AuthUser, Json(b): Json<Carhart4Body>,
) -> Json<Option<factor_models::Carhart4Report>> {
    let inputs = factor_models::Carhart4Inputs {
        portfolio_returns: &b.portfolio_returns,
        market_excess: &b.market_excess,
        smb: &b.smb,
        hml: &b.hml,
        wml: &b.wml,
        risk_free: &b.risk_free,
    };
    Json(factor_models::carhart4(&inputs))
}

#[derive(Deserialize)]
struct PairZBody {
    y: Vec<f64>,
    x: Vec<f64>,
    hedge_ratio: f64,
    #[serde(default)] intercept: f64,
    #[serde(default)]
    config: pair_trade_zscore::Config,
}

async fn pair_trade_zscore_route(
    _u: AuthUser, Json(b): Json<PairZBody>,
) -> Json<Option<pair_trade_zscore::PairReport>> {
    Json(pair_trade_zscore::compute(&b.y, &b.x, b.hedge_ratio, b.intercept, &b.config))
}

#[derive(Deserialize)]
struct ButterflyBody {
    butterfly: butterfly_spread::Butterfly,
    spot_at_expiry: Option<f64>,
}

#[derive(Serialize)]
struct ButterflyResponse {
    report: Option<butterfly_spread::ButterflyReport>,
    pnl_at_spot: Option<f64>,
}

async fn butterfly_spread_route(
    _u: AuthUser, Json(b): Json<ButterflyBody>,
) -> Json<ButterflyResponse> {
    let report = butterfly_spread::analyze(&b.butterfly);
    let pnl_at_spot = b.spot_at_expiry.and_then(|s| butterfly_spread::pnl_at_expiration(&b.butterfly, s));
    Json(ButterflyResponse { report, pnl_at_spot })
}

#[derive(Deserialize)]
struct JadeLizardBody {
    jade_lizard: jade_lizard::JadeLizard,
    spot_at_expiry: Option<f64>,
}

#[derive(Serialize)]
struct JadeLizardResponse {
    report: Option<jade_lizard::JadeLizardReport>,
    pnl_at_spot: Option<f64>,
}

async fn jade_lizard_route(
    _u: AuthUser, Json(b): Json<JadeLizardBody>,
) -> Json<JadeLizardResponse> {
    let report = jade_lizard::analyze(&b.jade_lizard);
    let pnl_at_spot = b.spot_at_expiry.and_then(|s| jade_lizard::pnl_at_expiration(&b.jade_lizard, s));
    Json(JadeLizardResponse { report, pnl_at_spot })
}

#[derive(Deserialize)]
struct RealizedCorrBody {
    series: Vec<realized_correlation::SymbolReturns>,
    #[serde(default = "default_corr_window")] window: usize,
}
fn default_corr_window() -> usize { 30 }

async fn realized_correlation_route(
    _u: AuthUser, Json(b): Json<RealizedCorrBody>,
) -> Json<Option<realized_correlation::CorrelationReport>> {
    Json(realized_correlation::compute(&b.series, b.window))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CornishFisherBody {
    returns: Vec<f64>,
    #[serde(default = "default_cvar_alpha")] alpha: f64,
}

async fn cornish_fisher_var_route(
    _u: AuthUser, Json(b): Json<CornishFisherBody>,
) -> Json<Option<cornish_fisher::CornishFisherReport>> {
    Json(cornish_fisher::compute(&b.returns, b.alpha))
}

#[derive(Deserialize)]
struct MacaulayDurationBody {
    cash_flows: Vec<macaulay_duration::CashFlow>,
    yield_to_maturity: f64,
    #[serde(default = "default_freq")] compounding_freq: u32,
}
fn default_freq() -> u32 { 2 }

async fn macaulay_duration_route(
    _u: AuthUser, Json(b): Json<MacaulayDurationBody>,
) -> Json<Option<macaulay_duration::DurationReport>> {
    Json(macaulay_duration::compute(&b.cash_flows, b.yield_to_maturity, b.compounding_freq))
}

#[derive(Deserialize)]
struct YieldCurveBootstrapBody {
    bonds: Vec<yield_curve_bootstrap::CouponBond>,
}

async fn yield_curve_bootstrap_route(
    _u: AuthUser, Json(b): Json<YieldCurveBootstrapBody>,
) -> Json<Option<yield_curve_bootstrap::YieldCurveReport>> {
    Json(yield_curve_bootstrap::bootstrap(&b.bonds))
}

#[derive(Deserialize)]
struct HerfindahlBody {
    weights: Vec<f64>,
}

async fn herfindahl_route(
    _u: AuthUser, Json(b): Json<HerfindahlBody>,
) -> Json<Option<herfindahl::HhiReport>> {
    Json(herfindahl::compute(&b.weights))
}

#[derive(Deserialize)]
struct TreynorJensenBody {
    portfolio_returns: Vec<f64>,
    market_returns: Vec<f64>,
    risk_free_returns: Vec<f64>,
}

async fn treynor_jensen_route(
    _u: AuthUser, Json(b): Json<TreynorJensenBody>,
) -> Json<Option<treynor_jensen::PerformanceReport>> {
    Json(treynor_jensen::compute(&b.portfolio_returns, &b.market_returns, &b.risk_free_returns))
}

#[derive(Deserialize)]
struct RiskParityBody {
    covariance: Vec<Vec<f64>>,
    #[serde(default = "default_rp_max_iter")] max_iter: usize,
    #[serde(default = "default_rp_tol")] tolerance: f64,
}
fn default_rp_max_iter() -> usize { 500 }
fn default_rp_tol() -> f64 { 1e-8 }

async fn risk_parity_weights_route(
    _u: AuthUser, Json(b): Json<RiskParityBody>,
) -> Json<Option<risk_parity_weights::RiskParityReport>> {
    Json(risk_parity_weights::solve(&b.covariance, b.max_iter, b.tolerance))
}

#[derive(Deserialize)]
struct BrinsonBody {
    inputs: Vec<brinson_attribution::BrinsonInput>,
}

async fn brinson_attribution_route(
    _u: AuthUser, Json(b): Json<BrinsonBody>,
) -> Json<Option<brinson_attribution::BrinsonReport>> {
    Json(brinson_attribution::analyze(&b.inputs))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct NsFitBody {
    tenors: Vec<f64>,
    yields: Vec<f64>,
    #[serde(default = "default_ns_tau")] tau: f64,
}
fn default_ns_tau() -> f64 { 1.5 }

async fn nelson_siegel_fit_route(
    _u: AuthUser, Json(b): Json<NsFitBody>,
) -> Json<Option<nelson_siegel::NsParams>> {
    Json(nelson_siegel::fit_nelson_siegel(&b.tenors, &b.yields, b.tau))
}

#[derive(Deserialize)]
struct NssFitBody {
    tenors: Vec<f64>,
    yields: Vec<f64>,
    #[serde(default = "default_ns_tau")] tau1: f64,
    #[serde(default = "default_nss_tau2")] tau2: f64,
}
fn default_nss_tau2() -> f64 { 5.0 }

async fn svensson_fit_route(
    _u: AuthUser, Json(b): Json<NssFitBody>,
) -> Json<Option<nelson_siegel::NssParams>> {
    Json(nelson_siegel::fit_svensson(&b.tenors, &b.yields, b.tau1, b.tau2))
}

#[derive(Deserialize)]
struct MargrabeBody {
    s1: f64, s2: f64,
    sigma1: f64, sigma2: f64,
    correlation: f64,
    #[serde(default)] q1: f64,
    #[serde(default)] q2: f64,
    time_to_expiry: f64,
}

async fn margrabe_spread_route(
    _u: AuthUser, Json(b): Json<MargrabeBody>,
) -> Json<Option<margrabe_spread_option::MargrabeReport>> {
    Json(margrabe_spread_option::price(
        b.s1, b.s2, b.sigma1, b.sigma2, b.correlation, b.q1, b.q2, b.time_to_expiry,
    ))
}

#[derive(Deserialize)]
struct AsianOptionBody {
    spot: f64, strike: f64,
    time_to_expiry: f64,
    risk_free: f64,
    #[serde(default)] dividend_yield: f64,
    sigma: f64,
    kind: asian_option::OptionKind,
}

async fn asian_option_route(
    _u: AuthUser, Json(b): Json<AsianOptionBody>,
) -> Json<Option<asian_option::AsianReport>> {
    Json(asian_option::price(
        b.spot, b.strike, b.time_to_expiry, b.risk_free, b.dividend_yield, b.sigma, b.kind,
    ))
}

#[derive(Deserialize)]
struct BarrierBody {
    spot: f64, strike: f64, barrier: f64,
    time_to_expiry: f64,
    risk_free: f64,
    #[serde(default)] dividend_yield: f64,
    sigma: f64,
    #[serde(default)] rebate: f64,
    kind: barrier_option::BarrierKind,
}

async fn barrier_option_route(
    _u: AuthUser, Json(b): Json<BarrierBody>,
) -> Json<Option<barrier_option::BarrierReport>> {
    Json(barrier_option::price(
        b.spot, b.strike, b.barrier, b.time_to_expiry, b.risk_free,
        b.dividend_yield, b.sigma, b.rebate, b.kind,
    ))
}

#[derive(Deserialize)]
struct VasicekZcbBody {
    params: vasicek::VasicekParams,
    /// Single tenor (set `tenors` instead for a curve).
    tenor: Option<f64>,
    /// Multiple tenors for a curve.
    tenors: Option<Vec<f64>>,
}

#[derive(Serialize)]
struct VasicekZcbResponse {
    single: Option<vasicek::VasicekZcb>,
    curve: Option<Vec<Option<vasicek::VasicekZcb>>>,
}

async fn vasicek_zcb_route(
    _u: AuthUser, Json(b): Json<VasicekZcbBody>,
) -> Json<VasicekZcbResponse> {
    let single = b.tenor.and_then(|t| vasicek::zero_coupon_bond(&b.params, t));
    let curve = b.tenors.as_ref().map(|ts| vasicek::zero_curve(&b.params, ts));
    Json(VasicekZcbResponse { single, curve })
}

#[derive(Deserialize)]
struct BlackLittermanBody {
    inputs: black_litterman::BlackLittermanInputs,
}

async fn black_litterman_route(
    _u: AuthUser, Json(b): Json<BlackLittermanBody>,
) -> Json<Option<black_litterman::BlackLittermanReport>> {
    Json(black_litterman::solve(&b.inputs))
}

#[derive(Deserialize)]
struct LvarBody {
    price_var_fraction: f64,
    notional: f64,
    spreads_as_fraction_of_mid: Vec<f64>,
    #[serde(default = "default_cvar_alpha")] alpha: f64,
}

async fn liquidity_adjusted_var_route(
    _u: AuthUser, Json(b): Json<LvarBody>,
) -> Json<Option<liquidity_adjusted_var::LvarReport>> {
    Json(liquidity_adjusted_var::compute(
        b.price_var_fraction, b.notional, &b.spreads_as_fraction_of_mid, b.alpha,
    ))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CirZcbBody {
    params: cir::CirParams,
    tenor: Option<f64>,
    tenors: Option<Vec<f64>>,
}

#[derive(Serialize)]
struct CirZcbResponse {
    single: Option<cir::CirZcb>,
    curve: Option<Vec<Option<cir::CirZcb>>>,
}

async fn cir_zcb_route(
    _u: AuthUser, Json(b): Json<CirZcbBody>,
) -> Json<CirZcbResponse> {
    let single = b.tenor.and_then(|t| cir::zero_coupon_bond(&b.params, t));
    let curve = b.tenors.as_ref().map(|ts| cir::zero_curve(&b.params, ts));
    Json(CirZcbResponse { single, curve })
}

#[derive(Deserialize)]
struct SabrBody {
    forward: f64,
    strike: f64,
    time_to_expiry: f64,
    params: sabr::SabrParams,
}

async fn sabr_vol_route(
    _u: AuthUser, Json(b): Json<SabrBody>,
) -> Json<Option<f64>> {
    Json(sabr::implied_lognormal_vol(b.forward, b.strike, b.time_to_expiry, &b.params))
}

#[derive(Deserialize)]
struct LookbackBody {
    spot: f64,
    observed_extreme: f64,
    time_to_expiry: f64,
    risk_free: f64,
    #[serde(default)] dividend_yield: f64,
    sigma: f64,
    kind: lookback_option::OptionKind,
}

async fn lookback_option_route(
    _u: AuthUser, Json(b): Json<LookbackBody>,
) -> Json<Option<lookback_option::LookbackReport>> {
    Json(lookback_option::price(
        b.spot, b.observed_extreme, b.time_to_expiry, b.risk_free,
        b.dividend_yield, b.sigma, b.kind,
    ))
}

#[derive(Deserialize)]
struct DigitalBody {
    spot: f64,
    strike: f64,
    time_to_expiry: f64,
    risk_free: f64,
    #[serde(default)] dividend_yield: f64,
    sigma: f64,
    #[serde(default = "default_cash")] cash: f64,
    kind: digital_option::DigitalKind,
}
fn default_cash() -> f64 { 1.0 }

async fn digital_option_route(
    _u: AuthUser, Json(b): Json<DigitalBody>,
) -> Json<Option<digital_option::DigitalReport>> {
    Json(digital_option::price(
        b.spot, b.strike, b.time_to_expiry, b.risk_free,
        b.dividend_yield, b.sigma, b.cash, b.kind,
    ))
}

#[derive(Deserialize)]
struct GrangerBody {
    x: Vec<f64>,
    y: Vec<f64>,
    #[serde(default = "default_granger_lags")] lags: usize,
}
fn default_granger_lags() -> usize { 2 }

async fn granger_causality_route(
    _u: AuthUser, Json(b): Json<GrangerBody>,
) -> Json<Option<granger_causality::GrangerReport>> {
    Json(granger_causality::test(&b.x, &b.y, b.lags))
}

#[derive(Deserialize)]
struct LedoitWolfBody {
    returns: Vec<Vec<f64>>,
}

async fn ledoit_wolf_route(
    _u: AuthUser, Json(b): Json<LedoitWolfBody>,
) -> Json<Option<ledoit_wolf::LedoitWolfReport>> {
    Json(ledoit_wolf::estimate(&b.returns))
}

#[derive(Deserialize)]
struct AlmgrenChrissBody {
    params: almgren_chriss::AlmgrenChrissParams,
}

async fn almgren_chriss_route(
    _u: AuthUser, Json(b): Json<AlmgrenChrissBody>,
) -> Json<Option<almgren_chriss::AlmgrenChrissReport>> {
    Json(almgren_chriss::solve(&b.params))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct HullWhiteZcbBody {
    params: hull_white::HullWhiteParams,
    #[serde(default)] t: f64,
    tau: f64,
    flat_forward_rate: f64,
}

async fn hull_white_zcb_route(
    _u: AuthUser, Json(b): Json<HullWhiteZcbBody>,
) -> Json<Option<hull_white::HullWhiteZcb>> {
    Json(hull_white::zero_coupon_bond_flat_forward(&b.params, b.t, b.tau, b.flat_forward_rate))
}

#[derive(Deserialize)]
struct CompoundOptionBody {
    spot: f64,
    strike_outer: f64,
    strike_inner: f64,
    t1: f64,
    t2: f64,
    risk_free: f64,
    #[serde(default)] dividend_yield: f64,
    sigma: f64,
    kind: compound_option::CompoundKind,
}

async fn compound_option_route(
    _u: AuthUser, Json(b): Json<CompoundOptionBody>,
) -> Json<Option<compound_option::CompoundReport>> {
    Json(compound_option::price(
        b.spot, b.strike_outer, b.strike_inner, b.t1, b.t2,
        b.risk_free, b.dividend_yield, b.sigma, b.kind,
    ))
}

#[derive(Deserialize)]
struct QuantoOptionBody {
    spot: f64,
    strike: f64,
    time_to_expiry: f64,
    rate_domestic: f64,
    rate_foreign: f64,
    #[serde(default)] dividend_yield: f64,
    sigma_asset: f64,
    sigma_fx: f64,
    correlation_asset_fx: f64,
    kind: quanto_option::OptionKind,
}

async fn quanto_option_route(
    _u: AuthUser, Json(b): Json<QuantoOptionBody>,
) -> Json<Option<quanto_option::QuantoReport>> {
    Json(quanto_option::price(
        b.spot, b.strike, b.time_to_expiry, b.rate_domestic, b.rate_foreign,
        b.dividend_yield, b.sigma_asset, b.sigma_fx, b.correlation_asset_fx, b.kind,
    ))
}

#[derive(Deserialize)]
struct CliquetOptionBody {
    spot: f64,
    reset_dates: Vec<f64>,
    risk_free: f64,
    #[serde(default)] dividend_yield: f64,
    sigma: f64,
    #[serde(default = "default_cliquet_multiplier")] reset_multiplier: f64,
}
fn default_cliquet_multiplier() -> f64 { 1.0 }

async fn cliquet_option_route(
    _u: AuthUser, Json(b): Json<CliquetOptionBody>,
) -> Json<Option<cliquet_option::CliquetReport>> {
    Json(cliquet_option::price(
        b.spot, &b.reset_dates, b.risk_free, b.dividend_yield, b.sigma, b.reset_multiplier,
    ))
}

#[derive(Deserialize)]
struct RankCorrBody {
    x: Vec<f64>,
    y: Vec<f64>,
}

async fn rank_correlation_route(
    _u: AuthUser, Json(b): Json<RankCorrBody>,
) -> Json<Option<rank_correlation::RankCorrReport>> {
    Json(rank_correlation::compute(&b.x, &b.y))
}

#[derive(Deserialize)]
struct TailDepBody {
    x: Vec<f64>,
    y: Vec<f64>,
    #[serde(default = "default_tail_alpha")] alpha: f64,
}
fn default_tail_alpha() -> f64 { 0.10 }

async fn tail_dependence_route(
    _u: AuthUser, Json(b): Json<TailDepBody>,
) -> Json<Option<tail_dependence::TailDependenceReport>> {
    Json(tail_dependence::compute(&b.x, &b.y, b.alpha))
}

#[derive(Deserialize)]
struct VarModelBody {
    series: Vec<Vec<f64>>,
    #[serde(default = "default_var_lags")] lags: usize,
}
fn default_var_lags() -> usize { 1 }

async fn vector_autoregression_route(
    _u: AuthUser, Json(b): Json<VarModelBody>,
) -> Json<Option<vector_autoregression::VarReport>> {
    Json(vector_autoregression::estimate(&b.series, b.lags))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct CholeskyBody {
    matrix: Vec<Vec<f64>>,
}

async fn cholesky_route(
    _u: AuthUser, Json(b): Json<CholeskyBody>,
) -> Json<Option<cholesky::CholeskyReport>> {
    Json(cholesky::decompose(&b.matrix))
}

#[derive(Deserialize)]
struct PcaBody {
    matrix: Vec<Vec<f64>>,
    #[serde(default = "default_pca_iter")] max_iter: usize,
    #[serde(default = "default_pca_tol")] tolerance: f64,
}
fn default_pca_iter() -> usize { 200 }
fn default_pca_tol() -> f64 { 1e-10 }

async fn pca_route(
    _u: AuthUser, Json(b): Json<PcaBody>,
) -> Json<Option<pca::PcaReport>> {
    Json(pca::decompose(&b.matrix, b.max_iter, b.tolerance))
}

#[derive(Deserialize)]
struct PowerOptionBody {
    spot: f64,
    strike: f64,
    time_to_expiry: f64,
    risk_free: f64,
    #[serde(default)] dividend_yield: f64,
    sigma: f64,
    power: f64,
    kind: power_option::OptionKind,
}

async fn power_option_route(
    _u: AuthUser, Json(b): Json<PowerOptionBody>,
) -> Json<Option<power_option::PowerOptionReport>> {
    Json(power_option::price(
        b.spot, b.strike, b.time_to_expiry, b.risk_free,
        b.dividend_yield, b.sigma, b.power, b.kind,
    ))
}

#[derive(Deserialize)]
struct GapOptionBody {
    spot: f64,
    strike_trigger: f64,
    strike_settlement: f64,
    time_to_expiry: f64,
    risk_free: f64,
    #[serde(default)] dividend_yield: f64,
    sigma: f64,
    kind: gap_option::OptionKind,
}

async fn gap_option_route(
    _u: AuthUser, Json(b): Json<GapOptionBody>,
) -> Json<Option<gap_option::GapOptionReport>> {
    Json(gap_option::price(
        b.spot, b.strike_trigger, b.strike_settlement, b.time_to_expiry,
        b.risk_free, b.dividend_yield, b.sigma, b.kind,
    ))
}

#[derive(Deserialize)]
struct FraBody {
    discount_start: f64,
    discount_end: f64,
    t_start: f64,
    t_end: f64,
    contract_rate: f64,
    notional: f64,
}

async fn fra_route(
    _u: AuthUser, Json(b): Json<FraBody>,
) -> Json<Option<fra::FraReport>> {
    Json(fra::analyze(
        b.discount_start, b.discount_end, b.t_start, b.t_end,
        b.contract_rate, b.notional,
    ))
}

#[derive(Deserialize)]
struct CapletBlack76Body {
    forward_rate: f64,
    strike_rate: f64,
    sigma: f64,
    t_expiry: f64,
    t_end: f64,
    discount_factor_t_end: f64,
    accrual: f64,
    notional: f64,
    kind: caplet_black76::OptionKind,
}

async fn caplet_black76_route(
    _u: AuthUser, Json(b): Json<CapletBlack76Body>,
) -> Json<Option<caplet_black76::CapletReport>> {
    Json(caplet_black76::price(
        b.forward_rate, b.strike_rate, b.sigma, b.t_expiry, b.t_end,
        b.discount_factor_t_end, b.accrual, b.notional, b.kind,
    ))
}

#[derive(Deserialize)]
struct TradeQualityBody {
    trades: Vec<trade_quality_stats::Trade>,
}

async fn trade_quality_stats_route(
    _u: AuthUser, Json(b): Json<TradeQualityBody>,
) -> Json<Option<trade_quality_stats::TradeQualityReport>> {
    Json(trade_quality_stats::analyze(&b.trades))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ChooserOptionBody {
    spot: f64,
    strike: f64,
    time_to_choice: f64,
    time_to_expiry: f64,
    risk_free: f64,
    #[serde(default)] dividend_yield: f64,
    sigma: f64,
}

async fn chooser_option_route(
    _u: AuthUser, Json(b): Json<ChooserOptionBody>,
) -> Json<Option<chooser_option::ChooserReport>> {
    Json(chooser_option::price(
        b.spot, b.strike, b.time_to_choice, b.time_to_expiry,
        b.risk_free, b.dividend_yield, b.sigma,
    ))
}

#[derive(Deserialize)]
struct ConditionalDrawdownBody {
    equity_curve: Vec<f64>,
    #[serde(default = "default_cvar_alpha")] alpha: f64,
}

async fn conditional_drawdown_route(
    _u: AuthUser, Json(b): Json<ConditionalDrawdownBody>,
) -> Json<Option<conditional_drawdown::CdarReport>> {
    Json(conditional_drawdown::compute(&b.equity_curve, b.alpha))
}

#[derive(Deserialize)]
struct RiskAdjustedRatiosBody {
    equity_curve: Vec<f64>,
    period_returns: Vec<f64>,
    #[serde(default)] risk_free_annual: f64,
    #[serde(default = "default_periods_per_year")] periods_per_year: f64,
    #[serde(default = "default_n_worst")] n_worst_drawdowns: usize,
}
fn default_n_worst() -> usize { 5 }

async fn risk_adjusted_ratios_route(
    _u: AuthUser, Json(b): Json<RiskAdjustedRatiosBody>,
) -> Json<Option<risk_adjusted_ratios::RiskAdjustedReport>> {
    Json(risk_adjusted_ratios::compute(
        &b.equity_curve, &b.period_returns, b.risk_free_annual,
        b.periods_per_year, b.n_worst_drawdowns,
    ))
}

#[derive(Deserialize)]
struct PainIndexBody {
    equity_curve: Vec<f64>,
    period_returns: Vec<f64>,
    #[serde(default)] risk_free_annual: f64,
    #[serde(default = "default_periods_per_year")] periods_per_year: f64,
}

async fn pain_index_route(
    _u: AuthUser, Json(b): Json<PainIndexBody>,
) -> Json<Option<pain_index::PainReport>> {
    Json(pain_index::compute(
        &b.equity_curve, &b.period_returns, b.risk_free_annual, b.periods_per_year,
    ))
}

#[derive(Deserialize)]
struct WeightedMidpriceBody {
    quote: Option<weighted_midprice::Quote>,
    quotes: Option<Vec<weighted_midprice::Quote>>,
}

#[derive(Serialize)]
struct WeightedMidpriceResponse {
    single: Option<weighted_midprice::MicropriceReport>,
    series: Option<Vec<Option<weighted_midprice::MicropriceReport>>>,
}

async fn weighted_midprice_route(
    _u: AuthUser, Json(b): Json<WeightedMidpriceBody>,
) -> Json<WeightedMidpriceResponse> {
    let single = b.quote.as_ref().and_then(weighted_midprice::compute);
    let series = b.quotes.as_ref().map(|qs| weighted_midprice::series(qs));
    Json(WeightedMidpriceResponse { single, series })
}

#[derive(Deserialize)]
struct EffectiveSpreadBody {
    observations: Vec<effective_spread::SpreadObservation>,
}

async fn effective_spread_route(
    _u: AuthUser, Json(b): Json<EffectiveSpreadBody>,
) -> Json<Option<effective_spread::SpreadReport>> {
    Json(effective_spread::analyze(&b.observations))
}

#[derive(Deserialize)]
struct Momentum121Body {
    symbols: Vec<momentum_12_1::SymbolMonthlyCloses>,
}

async fn momentum_12_1_route(
    _u: AuthUser, Json(b): Json<Momentum121Body>,
) -> Json<Option<momentum_12_1::MomentumReport>> {
    Json(momentum_12_1::scan(&b.symbols))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct BachelierBody {
    forward: f64,
    strike: f64,
    time_to_expiry: f64,
    #[serde(default)] risk_free: f64,
    normal_sigma: f64,
    kind: bachelier::OptionKind,
}

async fn bachelier_route(
    _u: AuthUser, Json(b): Json<BachelierBody>,
) -> Json<Option<bachelier::BachelierReport>> {
    Json(bachelier::price(b.forward, b.strike, b.time_to_expiry, b.risk_free,
        b.normal_sigma, b.kind))
}

#[derive(Deserialize)]
struct SwaptionBlackBody {
    forward_swap_rate: f64,
    strike_rate: f64,
    sigma: f64,
    time_to_expiry: f64,
    annuity_pv01: f64,
    notional: f64,
    kind: swaption_black::SwaptionKind,
}

async fn swaption_black_route(
    _u: AuthUser, Json(b): Json<SwaptionBlackBody>,
) -> Json<Option<swaption_black::SwaptionReport>> {
    Json(swaption_black::price(b.forward_swap_rate, b.strike_rate, b.sigma,
        b.time_to_expiry, b.annuity_pv01, b.notional, b.kind))
}

#[derive(Deserialize)]
struct CdsPricingBody {
    knots: Vec<cds_pricing::CurvePoint>,
    coupons: Vec<cds_pricing::CouponDate>,
    #[serde(default = "default_recovery")] recovery_rate: f64,
    notional: f64,
    existing_spread_bps: Option<f64>,
}
fn default_recovery() -> f64 { 0.40 }

async fn cds_pricing_route(
    _u: AuthUser, Json(b): Json<CdsPricingBody>,
) -> Json<Option<cds_pricing::CdsReport>> {
    Json(cds_pricing::analyze(&b.knots, &b.coupons, b.recovery_rate, b.notional,
        b.existing_spread_bps))
}

#[derive(Deserialize)]
struct AssetSwapBody {
    bond_clean_price: f64,
    par_value: f64,
    fixed_coupon_rate: f64,
    par_swap_rate: f64,
    cash_flows: Vec<asset_swap_spread::CashFlow>,
}

async fn asset_swap_spread_route(
    _u: AuthUser, Json(b): Json<AssetSwapBody>,
) -> Json<Option<asset_swap_spread::AssetSwapReport>> {
    Json(asset_swap_spread::analyze(b.bond_clean_price, b.par_value,
        b.fixed_coupon_rate, b.par_swap_rate, &b.cash_flows))
}

#[derive(Deserialize)]
struct HoltWintersBody {
    series: Vec<f64>,
    #[serde(default = "default_hw_alpha")] alpha: f64,
    #[serde(default = "default_hw_beta")] beta: f64,
    #[serde(default = "default_hw_horizon")] forecast_horizon: usize,
}
fn default_hw_alpha() -> f64 { 0.3 }
fn default_hw_beta() -> f64 { 0.1 }
fn default_hw_horizon() -> usize { 10 }

async fn holt_winters_route(
    _u: AuthUser, Json(b): Json<HoltWintersBody>,
) -> Json<Option<holt_winters::HoltWintersReport>> {
    Json(holt_winters::compute(&b.series, b.alpha, b.beta, b.forecast_horizon))
}

#[derive(Deserialize)]
struct VwemaBody {
    prices: Vec<f64>,
    volumes: Vec<f64>,
    #[serde(default = "default_vwema_period")] period: usize,
    #[serde(default)] volume_weighted: bool,
}
fn default_vwema_period() -> usize { 20 }

async fn vwema_route(_u: AuthUser, Json(b): Json<VwemaBody>) -> Json<Vec<Option<f64>>> {
    Json(if b.volume_weighted {
        vwema::compute_volume_weighted(&b.prices, &b.volumes, b.period)
    } else {
        vwema::compute(&b.prices, &b.volumes, b.period)
    })
}

#[derive(Deserialize)]
struct SmiBody {
    highs: Vec<f64>,
    lows: Vec<f64>,
    closes: Vec<f64>,
    #[serde(default = "default_smi_period")] period: usize,
    #[serde(default = "default_smi_smooth")] smooth: usize,
    #[serde(default = "default_smi_signal")] signal: usize,
}
fn default_smi_period() -> usize { 14 }
fn default_smi_smooth() -> usize { 3 }
fn default_smi_signal() -> usize { 3 }

async fn stochastic_momentum_index_route(
    _u: AuthUser, Json(b): Json<SmiBody>,
) -> Json<stochastic_momentum_index::SmiReport> {
    Json(stochastic_momentum_index::compute(&b.highs, &b.lows, &b.closes,
        b.period, b.smooth, b.signal))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct AmericanBinomialBody {
    spot: f64,
    strike: f64,
    time_to_expiry: f64,
    risk_free: f64,
    #[serde(default)] dividend_yield: f64,
    sigma: f64,
    #[serde(default = "default_n_steps")] n_steps: usize,
    kind: american_binomial::OptionKind,
}
fn default_n_steps() -> usize { 200 }

async fn american_binomial_route(
    _u: AuthUser, Json(b): Json<AmericanBinomialBody>,
) -> Json<Option<american_binomial::AmericanReport>> {
    Json(american_binomial::price(b.spot, b.strike, b.time_to_expiry, b.risk_free,
        b.dividend_yield, b.sigma, b.n_steps, b.kind))
}

#[derive(Deserialize)]
struct BermudanBinomialBody {
    spot: f64,
    strike: f64,
    time_to_expiry: f64,
    risk_free: f64,
    #[serde(default)] dividend_yield: f64,
    sigma: f64,
    #[serde(default = "default_n_steps")] n_steps: usize,
    exercise_dates_years: Vec<f64>,
    kind: bermudan_binomial::OptionKind,
}

async fn bermudan_binomial_route(
    _u: AuthUser, Json(b): Json<BermudanBinomialBody>,
) -> Json<Option<bermudan_binomial::BermudanReport>> {
    Json(bermudan_binomial::price(b.spot, b.strike, b.time_to_expiry, b.risk_free,
        b.dividend_yield, b.sigma, b.n_steps, &b.exercise_dates_years, b.kind))
}

#[derive(Deserialize)]
struct ConvertibleBondBody {
    inputs: convertible_bond::ConvertibleBondInputs,
}

async fn convertible_bond_route(
    _u: AuthUser, Json(b): Json<ConvertibleBondBody>,
) -> Json<Option<convertible_bond::ConvertibleBondReport>> {
    Json(convertible_bond::price(&b.inputs))
}

#[derive(Deserialize)]
struct HrpBody {
    covariance: Vec<Vec<f64>>,
}

async fn hierarchical_risk_parity_route(
    _u: AuthUser, Json(b): Json<HrpBody>,
) -> Json<Option<hierarchical_risk_parity::HrpReport>> {
    Json(hierarchical_risk_parity::solve(&b.covariance))
}

#[derive(Deserialize)]
struct HawkesBody {
    event_times: Vec<f64>,
    query_times: Vec<f64>,
    params: hawkes_intensity::HawkesParams,
}

async fn hawkes_intensity_route(
    _u: AuthUser, Json(b): Json<HawkesBody>,
) -> Json<Option<hawkes_intensity::HawkesReport>> {
    Json(hawkes_intensity::compute(&b.event_times, &b.query_times, b.params))
}

#[derive(Deserialize)]
struct Arima111Body {
    series: Vec<f64>,
}

async fn arima_111_route(
    _u: AuthUser, Json(b): Json<Arima111Body>,
) -> Json<Option<arima_111::ArimaReport>> {
    Json(arima_111::fit(&b.series))
}

#[derive(Deserialize)]
struct GreeksProfileBody {
    strike: f64,
    time_to_expiry: f64,
    risk_free: f64,
    #[serde(default)] dividend_yield: f64,
    sigma: f64,
    spot_grid_low: f64,
    spot_grid_high: f64,
    #[serde(default = "default_grid_points")] n_points: usize,
    kind: greeks_profile::OptionKind,
}
fn default_grid_points() -> usize { 41 }

async fn greeks_profile_route(
    _u: AuthUser, Json(b): Json<GreeksProfileBody>,
) -> Json<Option<greeks_profile::GreeksProfileReport>> {
    Json(greeks_profile::compute(b.strike, b.time_to_expiry, b.risk_free,
        b.dividend_yield, b.sigma, b.spot_grid_low, b.spot_grid_high,
        b.n_points, b.kind))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct TrinomialBody {
    spot: f64,
    strike: f64,
    time_to_expiry: f64,
    risk_free: f64,
    #[serde(default)] dividend_yield: f64,
    sigma: f64,
    #[serde(default = "default_n_steps")] n_steps: usize,
    kind: trinomial_tree::OptionKind,
    #[serde(default)] american: bool,
}

async fn trinomial_tree_route(
    _u: AuthUser, Json(b): Json<TrinomialBody>,
) -> Json<Option<trinomial_tree::TrinomialReport>> {
    Json(trinomial_tree::price(b.spot, b.strike, b.time_to_expiry, b.risk_free,
        b.dividend_yield, b.sigma, b.n_steps, b.kind, b.american))
}

#[derive(Deserialize)]
struct ArchLmBody {
    returns: Vec<f64>,
    #[serde(default = "default_arch_lags")] lags: usize,
}
fn default_arch_lags() -> usize { 5 }

async fn arch_lm_test_route(
    _u: AuthUser, Json(b): Json<ArchLmBody>,
) -> Json<Option<arch_lm_test::ArchLmReport>> {
    Json(arch_lm_test::test(&b.returns, b.lags))
}

#[derive(Deserialize)]
struct LjungBoxBody {
    series: Vec<f64>,
    #[serde(default = "default_ljung_lags")] lags: usize,
}
fn default_ljung_lags() -> usize { 10 }

async fn ljung_box_route(
    _u: AuthUser, Json(b): Json<LjungBoxBody>,
) -> Json<Option<ljung_box::LjungBoxReport>> {
    Json(ljung_box::test(&b.series, b.lags))
}

#[derive(Deserialize)]
struct MinVarianceBody {
    covariance: Vec<Vec<f64>>,
    expected_excess_returns: Vec<f64>,
}

async fn min_variance_portfolio_route(
    _u: AuthUser, Json(b): Json<MinVarianceBody>,
) -> Json<Option<min_variance_portfolio::MvReport>> {
    Json(min_variance_portfolio::solve(&b.covariance, &b.expected_excess_returns))
}

#[derive(Deserialize)]
struct CandlePatternBody {
    bars: Vec<candle_patterns::Bar>,
}

async fn candle_patterns_route(
    _u: AuthUser, Json(b): Json<CandlePatternBody>,
) -> Json<candle_patterns::PatternReport> {
    Json(candle_patterns::scan(&b.bars))
}

#[derive(Deserialize)]
struct AdfStandaloneBody {
    series: Vec<f64>,
    #[serde(default = "default_adf_standalone_lags")] lags: usize,
}
fn default_adf_standalone_lags() -> usize { 1 }

async fn adf_standalone_route(
    _u: AuthUser, Json(b): Json<AdfStandaloneBody>,
) -> Json<Option<adf_standalone::AdfReport>> {
    Json(adf_standalone::test(&b.series, b.lags))
}

#[derive(Deserialize)]
struct BollingerOscBody {
    closes: Vec<f64>,
    #[serde(default = "default_bb_period")] period: usize,
    #[serde(default = "default_bb_k")] k: f64,
}
fn default_bb_period() -> usize { 20 }
fn default_bb_k() -> f64 { 2.0 }

async fn bollinger_oscillators_route(
    _u: AuthUser, Json(b): Json<BollingerOscBody>,
) -> Json<bollinger_oscillators::BollingerOscReport> {
    Json(bollinger_oscillators::compute(&b.closes, b.period, b.k))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct VarBacktestBody {
    realized_returns: Vec<f64>,
    var_forecasts: Vec<f64>,
    #[serde(default = "default_var_alpha")] alpha: f64,
}
fn default_var_alpha() -> f64 { 0.05 }

async fn var_backtest_kupiec_route(
    _u: AuthUser, Json(b): Json<VarBacktestBody>,
) -> Json<Option<var_backtest_kupiec::KupiecReport>> {
    Json(var_backtest_kupiec::test(&b.realized_returns, &b.var_forecasts, b.alpha))
}

async fn var_backtest_christoffersen_route(
    _u: AuthUser, Json(b): Json<VarBacktestBody>,
) -> Json<Option<var_backtest_christoffersen::ChristoffersenReport>> {
    Json(var_backtest_christoffersen::test(&b.realized_returns, &b.var_forecasts, b.alpha))
}

#[derive(Deserialize)]
struct ValueFactorBody {
    symbols: Vec<value_factor::SymbolFundamentals>,
}

async fn value_factor_route(
    _u: AuthUser, Json(b): Json<ValueFactorBody>,
) -> Json<Option<value_factor::ValueFactorReport>> {
    Json(value_factor::scan(&b.symbols))
}

#[derive(Deserialize)]
struct QualityFactorBody {
    symbols: Vec<quality_factor::SymbolQualityInputs>,
}

async fn quality_factor_route(
    _u: AuthUser, Json(b): Json<QualityFactorBody>,
) -> Json<Option<quality_factor::QualityFactorReport>> {
    Json(quality_factor::scan(&b.symbols))
}

#[derive(Deserialize)]
struct LowVolFactorBody {
    symbols: Vec<low_vol_factor::SymbolPriceHistory>,
    #[serde(default = "default_lookback_days")] lookback_days: usize,
}
fn default_lookback_days() -> usize { 63 }

async fn low_vol_factor_route(
    _u: AuthUser, Json(b): Json<LowVolFactorBody>,
) -> Json<Option<low_vol_factor::LowVolFactorReport>> {
    Json(low_vol_factor::scan(&b.symbols, b.lookback_days))
}

#[derive(Deserialize)]
struct CompositeFactorBody {
    symbols: Vec<composite_factor_scoring::SymbolFactorScores>,
    factor_weights: Vec<f64>,
}

async fn composite_factor_scoring_route(
    _u: AuthUser, Json(b): Json<CompositeFactorBody>,
) -> Json<Option<composite_factor_scoring::CompositeReport>> {
    Json(composite_factor_scoring::score(&b.symbols, &b.factor_weights))
}

#[derive(Deserialize)]
struct SurvivalProbabilityBody {
    segments: Vec<survival_probability::HazardSegment>,
    query_times: Vec<f64>,
}

async fn survival_probability_route(
    _u: AuthUser, Json(b): Json<SurvivalProbabilityBody>,
) -> Json<Option<survival_probability::SurvivalCurveReport>> {
    Json(survival_probability::build_curve(&b.segments, &b.query_times))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct StraddleBody {
    straddle: straddle::Straddle,
    spot_at_expiry: Option<f64>,
}

#[derive(Serialize)]
struct StraddleResponse {
    report: Option<straddle::StraddleReport>,
    pnl_at_spot: Option<f64>,
}

async fn straddle_route(
    _u: AuthUser, Json(b): Json<StraddleBody>,
) -> Json<StraddleResponse> {
    let report = straddle::analyze(&b.straddle);
    let pnl_at_spot = b.spot_at_expiry.and_then(|s| straddle::pnl_at_expiration(&b.straddle, s));
    Json(StraddleResponse { report, pnl_at_spot })
}

#[derive(Deserialize)]
struct StrangleBody {
    strangle: strangle::Strangle,
    spot_at_expiry: Option<f64>,
}

#[derive(Serialize)]
struct StrangleResponse {
    report: Option<strangle::StrangleReport>,
    pnl_at_spot: Option<f64>,
}

async fn strangle_route(
    _u: AuthUser, Json(b): Json<StrangleBody>,
) -> Json<StrangleResponse> {
    let report = strangle::analyze(&b.strangle);
    let pnl_at_spot = b.spot_at_expiry.and_then(|s| strangle::pnl_at_expiration(&b.strangle, s));
    Json(StrangleResponse { report, pnl_at_spot })
}

#[derive(Deserialize)]
struct IronButterflyBody {
    iron_butterfly: iron_butterfly::IronButterfly,
    spot_at_expiry: Option<f64>,
}

#[derive(Serialize)]
struct IronButterflyResponse {
    report: Option<iron_butterfly::IronButterflyReport>,
    pnl_at_spot: Option<f64>,
}

async fn iron_butterfly_route(
    _u: AuthUser, Json(b): Json<IronButterflyBody>,
) -> Json<IronButterflyResponse> {
    let report = iron_butterfly::analyze(&b.iron_butterfly);
    let pnl_at_spot = b.spot_at_expiry.and_then(|s| iron_butterfly::pnl_at_expiration(&b.iron_butterfly, s));
    Json(IronButterflyResponse { report, pnl_at_spot })
}

#[derive(Deserialize)]
struct CollarBody {
    collar: collar::Collar,
    spot_at_expiry: Option<f64>,
}

#[derive(Serialize)]
struct CollarResponse {
    report: Option<collar::CollarReport>,
    pnl_at_spot: Option<f64>,
}

async fn collar_route(
    _u: AuthUser, Json(b): Json<CollarBody>,
) -> Json<CollarResponse> {
    let report = collar::analyze(&b.collar);
    let pnl_at_spot = b.spot_at_expiry.and_then(|s| collar::pnl_at_expiration(&b.collar, s));
    Json(CollarResponse { report, pnl_at_spot })
}

#[derive(Deserialize)]
struct HodrickPrescottBody {
    series: Vec<f64>,
    #[serde(default = "default_hp_lambda")] lambda: f64,
}
fn default_hp_lambda() -> f64 { 1600.0 }

async fn hodrick_prescott_route(
    _u: AuthUser, Json(b): Json<HodrickPrescottBody>,
) -> Json<Option<hodrick_prescott::HpReport>> {
    Json(hodrick_prescott::compute(&b.series, b.lambda))
}

#[derive(Deserialize)]
struct KalmanFilter1dBody {
    observations: Vec<f64>,
    params: kalman_filter_1d::KalmanParams1d,
}

async fn kalman_filter_1d_route(
    _u: AuthUser, Json(b): Json<KalmanFilter1dBody>,
) -> Json<Option<kalman_filter_1d::KalmanReport1d>> {
    Json(kalman_filter_1d::filter(&b.observations, &b.params))
}

#[derive(Deserialize)]
struct BlockBootstrapBody {
    data: Vec<f64>,
    #[serde(default = "default_block_size")] block_size: usize,
    #[serde(default = "default_n_resamples")] n_resamples: usize,
    statistic: block_bootstrap::Statistic,
    #[serde(default)] seed: u64,
}
fn default_block_size() -> usize { 20 }
fn default_n_resamples() -> usize { 1000 }

async fn block_bootstrap_route(
    _u: AuthUser, Json(b): Json<BlockBootstrapBody>,
) -> Json<Option<block_bootstrap::BootstrapReport>> {
    Json(block_bootstrap::bootstrap(&b.data, b.block_size, b.n_resamples, b.statistic, b.seed))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RealizedHigherMomentsBody {
    returns: Vec<f64>,
    #[serde(default = "default_hm_window")] window: usize,
}
fn default_hm_window() -> usize { 60 }

async fn realized_higher_moments_route(
    _u: AuthUser, Json(b): Json<RealizedHigherMomentsBody>,
) -> Json<Option<realized_higher_moments::HigherMomentsReport>> {
    Json(realized_higher_moments::compute(&b.returns, b.window))
}

#[derive(Deserialize)]
struct LowerPartialMomentsBody {
    returns: Vec<f64>,
    #[serde(default)] threshold: f64,
}

async fn lower_partial_moments_route(
    _u: AuthUser, Json(b): Json<LowerPartialMomentsBody>,
) -> Json<Option<lower_partial_moments::PartialMomentReport>> {
    Json(lower_partial_moments::compute(&b.returns, b.threshold))
}

#[derive(Deserialize)]
struct DfaBody {
    series: Vec<f64>,
    #[serde(default = "default_dfa_scales")] scales: Vec<usize>,
}
fn default_dfa_scales() -> Vec<usize> { vec![4, 8, 16, 32, 64, 128] }

async fn dfa_route(
    _u: AuthUser, Json(b): Json<DfaBody>,
) -> Json<Option<dfa::DfaReport>> {
    Json(dfa::compute(&b.series, &b.scales))
}

#[derive(Deserialize)]
struct SampleEntropyBody {
    series: Vec<f64>,
    #[serde(default)] tolerance: Option<f64>,
    #[serde(default = "default_se_order")] order: usize,
}
fn default_se_order() -> usize { 2 }

async fn sample_entropy_route(
    _u: AuthUser, Json(b): Json<SampleEntropyBody>,
) -> Json<Option<sample_entropy::SampleEntropyReport>> {
    let r = match b.tolerance {
        Some(r) => sample_entropy::compute(&b.series, b.order, r),
        None => sample_entropy::compute_default(&b.series),
    };
    Json(r)
}

#[derive(Deserialize)]
struct PermutationEntropyBody {
    series: Vec<f64>,
    #[serde(default = "default_pe_order")] order: usize,
}
fn default_pe_order() -> usize { 3 }

async fn permutation_entropy_route(
    _u: AuthUser, Json(b): Json<PermutationEntropyBody>,
) -> Json<Option<permutation_entropy::PermutationEntropyReport>> {
    Json(permutation_entropy::compute(&b.series, b.order))
}

#[derive(Deserialize)]
struct TripleTopBottomBody {
    bars: Vec<triple_top_bottom::Bar>,
    #[serde(default)] config: Option<triple_top_bottom::Config>,
}

async fn triple_top_bottom_route(
    _u: AuthUser, Json(b): Json<TripleTopBottomBody>,
) -> Json<Vec<triple_top_bottom::TripleCandidate>> {
    let cfg = b.config.unwrap_or_default();
    Json(triple_top_bottom::detect(&b.bars, &cfg))
}

#[derive(Deserialize)]
struct MaxDiversificationBody {
    covariance: Vec<Vec<f64>>,
}

async fn max_diversification_route(
    _u: AuthUser, Json(b): Json<MaxDiversificationBody>,
) -> Json<Option<max_diversification::MaxDivReport>> {
    Json(max_diversification::solve(&b.covariance))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RealizedSemivarianceBody {
    returns: Vec<f64>,
    #[serde(default = "default_periods_per_year")] periods_per_year: f64,
}

async fn realized_semivariance_route(
    _u: AuthUser, Json(b): Json<RealizedSemivarianceBody>,
) -> Json<Option<realized_semivariance::SemivarianceReport>> {
    Json(realized_semivariance::compute(&b.returns, b.periods_per_year))
}

#[derive(Deserialize)]
struct BipowerVariationBody {
    returns: Vec<f64>,
}

async fn bipower_variation_route(
    _u: AuthUser, Json(b): Json<BipowerVariationBody>,
) -> Json<Option<bipower_variation::BipowerReport>> {
    Json(bipower_variation::compute(&b.returns))
}

#[derive(Deserialize)]
struct UpDownCaptureBody {
    portfolio: Vec<f64>,
    benchmark: Vec<f64>,
}

async fn up_down_capture_route(
    _u: AuthUser, Json(b): Json<UpDownCaptureBody>,
) -> Json<Option<up_down_capture::CaptureReport>> {
    Json(up_down_capture::compute(&b.portfolio, &b.benchmark))
}

#[derive(Deserialize)]
struct ModiglianiM2Body {
    portfolio: Vec<f64>,
    benchmark: Vec<f64>,
    #[serde(default)] risk_free_rate: f64,
}

async fn modigliani_m2_route(
    _u: AuthUser, Json(b): Json<ModiglianiM2Body>,
) -> Json<Option<modigliani_m2::M2Report>> {
    Json(modigliani_m2::compute(&b.portfolio, &b.benchmark, b.risk_free_rate))
}

#[derive(Deserialize)]
struct BetaShrinkageBody {
    assets: Vec<beta_shrinkage::AssetReturns>,
    market_returns: Vec<f64>,
}

async fn beta_shrinkage_route(
    _u: AuthUser, Json(b): Json<BetaShrinkageBody>,
) -> Json<Option<beta_shrinkage::ShrinkageReport>> {
    Json(beta_shrinkage::shrink(&b.assets, &b.market_returns))
}

#[derive(Deserialize)]
struct KeyRateDurationBody {
    cash_flows: Vec<key_rate_duration::CashFlow>,
    tenors: Vec<key_rate_duration::KeyTenor>,
    #[serde(default = "default_krd_bump_bps")] bump_basis_points: f64,
}
fn default_krd_bump_bps() -> f64 { 1.0 }

async fn key_rate_duration_route(
    _u: AuthUser, Json(b): Json<KeyRateDurationBody>,
) -> Json<Option<key_rate_duration::KrdReport>> {
    Json(key_rate_duration::compute(&b.cash_flows, &b.tenors, b.bump_basis_points))
}

#[derive(Deserialize)]
struct TreynorBlackBody {
    securities: Vec<treynor_black::ActiveSecurity>,
    market_excess_return: f64,
    market_variance: f64,
}

async fn treynor_black_route(
    _u: AuthUser, Json(b): Json<TreynorBlackBody>,
) -> Json<Option<treynor_black::TreynorBlackReport>> {
    Json(treynor_black::solve(&b.securities, b.market_excess_return, b.market_variance))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct VortexBody {
    bars: Vec<vortex_indicator::Bar>,
    #[serde(default = "default_vortex_period")] period: usize,
}
fn default_vortex_period() -> usize { 14 }

async fn vortex_indicator_route(
    _u: AuthUser, Json(b): Json<VortexBody>,
) -> Json<vortex_indicator::VortexReport> {
    Json(vortex_indicator::compute(&b.bars, b.period))
}

#[derive(Deserialize)]
struct PivotPointsBody {
    prior_session: pivot_points::SessionOhlc,
}

async fn pivot_points_route(
    _u: AuthUser, Json(b): Json<PivotPointsBody>,
) -> Json<Option<pivot_points::PivotReport>> {
    Json(pivot_points::compute(b.prior_session))
}

#[derive(Deserialize)]
struct AroonBody {
    bars: Vec<aroon_indicator::Bar>,
    #[serde(default = "default_aroon_period")] period: usize,
}
fn default_aroon_period() -> usize { 25 }

async fn aroon_indicator_route(
    _u: AuthUser, Json(b): Json<AroonBody>,
) -> Json<aroon_indicator::AroonReport> {
    Json(aroon_indicator::compute(&b.bars, b.period))
}

#[derive(Deserialize)]
struct DonchianBody {
    bars: Vec<donchian_channels::Bar>,
    #[serde(default = "default_donchian_period")] period: usize,
}
fn default_donchian_period() -> usize { 20 }

async fn donchian_channels_route(
    _u: AuthUser, Json(b): Json<DonchianBody>,
) -> Json<donchian_channels::DonchianReport> {
    Json(donchian_channels::compute(&b.bars, b.period))
}

#[derive(Deserialize)]
struct StochasticRsiBody {
    closes: Vec<f64>,
    #[serde(default = "default_srsi_rsi_period")] rsi_period: usize,
    #[serde(default = "default_srsi_k_period")] k_period: usize,
    #[serde(default = "default_srsi_k_smooth")] k_smooth: usize,
    #[serde(default = "default_srsi_d_smooth")] d_smooth: usize,
}
fn default_srsi_rsi_period() -> usize { 14 }
fn default_srsi_k_period() -> usize { 14 }
fn default_srsi_k_smooth() -> usize { 3 }
fn default_srsi_d_smooth() -> usize { 3 }

async fn stochastic_rsi_route(
    _u: AuthUser, Json(b): Json<StochasticRsiBody>,
) -> Json<stochastic_rsi::StochasticRsiReport> {
    Json(stochastic_rsi::compute(&b.closes, b.rsi_period, b.k_period, b.k_smooth, b.d_smooth))
}

#[derive(Deserialize)]
struct BollingerBandWidthBody {
    closes: Vec<f64>,
    #[serde(default = "default_bbw_period")] period: usize,
    #[serde(default = "default_bbw_k")] k: f64,
}
fn default_bbw_period() -> usize { 20 }
fn default_bbw_k() -> f64 { 2.0 }

async fn bollinger_band_width_route(
    _u: AuthUser, Json(b): Json<BollingerBandWidthBody>,
) -> Json<bollinger_band_width::BbwReport> {
    Json(bollinger_band_width::compute(&b.closes, b.period, b.k))
}

#[derive(Deserialize)]
struct BondConvexityBody {
    cash_flows: Vec<bond_convexity::CashFlow>,
    ytm: f64,
    #[serde(default = "default_compounding_freq")] compounding_periods_per_year: u32,
}
fn default_compounding_freq() -> u32 { 2 }

async fn bond_convexity_route(
    _u: AuthUser, Json(b): Json<BondConvexityBody>,
) -> Json<Option<bond_convexity::ConvexityReport>> {
    Json(bond_convexity::compute(&b.cash_flows, b.ytm, b.compounding_periods_per_year))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct AdlBody { bars: Vec<accumulation_distribution_line::Bar> }

async fn adl_route(
    _u: AuthUser, Json(b): Json<AdlBody>,
) -> Json<Vec<Option<f64>>> {
    Json(accumulation_distribution_line::compute(&b.bars))
}

#[derive(Deserialize)]
struct ObvBody { bars: Vec<on_balance_volume::Bar> }

async fn obv_route(
    _u: AuthUser, Json(b): Json<ObvBody>,
) -> Json<Vec<Option<f64>>> {
    Json(on_balance_volume::compute(&b.bars))
}

#[derive(Deserialize)]
struct ChaikinOscBody {
    bars: Vec<chaikin_oscillator::Bar>,
    #[serde(default = "default_co_fast")] fast: usize,
    #[serde(default = "default_co_slow")] slow: usize,
}
fn default_co_fast() -> usize { 3 }
fn default_co_slow() -> usize { 10 }

async fn chaikin_oscillator_route(
    _u: AuthUser, Json(b): Json<ChaikinOscBody>,
) -> Json<Vec<Option<f64>>> {
    Json(chaikin_oscillator::compute(&b.bars, b.fast, b.slow))
}

#[derive(Deserialize)]
struct KvoBody {
    bars: Vec<klinger_volume_oscillator::Bar>,
    #[serde(default = "default_kvo_fast")] fast: usize,
    #[serde(default = "default_kvo_slow")] slow: usize,
    #[serde(default = "default_kvo_signal")] signal_period: usize,
}
fn default_kvo_fast() -> usize { 34 }
fn default_kvo_slow() -> usize { 55 }
fn default_kvo_signal() -> usize { 13 }

async fn klinger_volume_oscillator_route(
    _u: AuthUser, Json(b): Json<KvoBody>,
) -> Json<klinger_volume_oscillator::KvoReport> {
    Json(klinger_volume_oscillator::compute(&b.bars, b.fast, b.slow, b.signal_period))
}

#[derive(Deserialize)]
struct CmoBody {
    closes: Vec<f64>,
    #[serde(default = "default_cmo_period")] period: usize,
}
fn default_cmo_period() -> usize { 14 }

async fn chande_momentum_oscillator_route(
    _u: AuthUser, Json(b): Json<CmoBody>,
) -> Json<Vec<Option<f64>>> {
    Json(chande_momentum_oscillator::compute(&b.closes, b.period))
}

#[derive(Deserialize)]
struct HillBody {
    losses: Vec<f64>,
    #[serde(default = "default_hill_ks")] k_values: Vec<usize>,
}
fn default_hill_ks() -> Vec<usize> { vec![25, 50, 100, 200] }

async fn hill_estimator_route(
    _u: AuthUser, Json(b): Json<HillBody>,
) -> Json<Option<hill_estimator::HillReport>> {
    Json(hill_estimator::compute(&b.losses, &b.k_values))
}

#[derive(Deserialize)]
struct ComponentVarBody {
    weights: Vec<f64>,
    covariance: Vec<Vec<f64>>,
    #[serde(default = "default_var_confidence")] confidence: f64,
}
fn default_var_confidence() -> f64 { 0.95 }

async fn component_var_route(
    _u: AuthUser, Json(b): Json<ComponentVarBody>,
) -> Json<Option<component_var::ComponentVarReport>> {
    Json(component_var::compute(&b.weights, &b.covariance, b.confidence))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct AlmaBody {
    closes: Vec<f64>,
    #[serde(default = "default_alma_period")] period: usize,
    #[serde(default = "default_alma_offset")] offset: f64,
    #[serde(default = "default_alma_sigma")] sigma: f64,
}
fn default_alma_period() -> usize { 9 }
fn default_alma_offset() -> f64 { 0.85 }
fn default_alma_sigma() -> f64 { 6.0 }

async fn alma_route(
    _u: AuthUser, Json(b): Json<AlmaBody>,
) -> Json<Vec<Option<f64>>> {
    Json(alma_legoux::compute(&b.closes, b.period, b.offset, b.sigma))
}

#[derive(Deserialize)]
struct T3Body {
    closes: Vec<f64>,
    #[serde(default = "default_t3_period")] period: usize,
    #[serde(default = "default_t3_volume_factor")] volume_factor: f64,
}
fn default_t3_period() -> usize { 5 }
fn default_t3_volume_factor() -> f64 { 0.7 }

async fn t3_route(
    _u: AuthUser, Json(b): Json<T3Body>,
) -> Json<Vec<Option<f64>>> {
    Json(t3_moving_average::compute(&b.closes, b.period, b.volume_factor))
}

#[derive(Deserialize)]
struct FramaBody {
    bars: Vec<frama_fractal::Bar>,
    #[serde(default = "default_frama_period")] period: usize,
}
fn default_frama_period() -> usize { 16 }

async fn frama_route(
    _u: AuthUser, Json(b): Json<FramaBody>,
) -> Json<Vec<Option<f64>>> {
    Json(frama_fractal::compute(&b.bars, b.period))
}

#[derive(Deserialize)]
struct CoppockBody {
    closes: Vec<f64>,
    #[serde(default = "default_coppock_long")] long_period: usize,
    #[serde(default = "default_coppock_short")] short_period: usize,
    #[serde(default = "default_coppock_wma")] wma_period: usize,
}
fn default_coppock_long() -> usize { 14 }
fn default_coppock_short() -> usize { 11 }
fn default_coppock_wma() -> usize { 10 }

async fn coppock_curve_route(
    _u: AuthUser, Json(b): Json<CoppockBody>,
) -> Json<Vec<Option<f64>>> {
    Json(coppock_curve::compute(&b.closes, b.long_period, b.short_period, b.wma_period))
}

#[derive(Deserialize)]
struct DpoBody {
    closes: Vec<f64>,
    #[serde(default = "default_dpo_period")] period: usize,
}
fn default_dpo_period() -> usize { 20 }

async fn dpo_route(
    _u: AuthUser, Json(b): Json<DpoBody>,
) -> Json<Vec<Option<f64>>> {
    Json(detrended_price_oscillator::compute(&b.closes, b.period))
}

#[derive(Deserialize)]
struct FibBody { leg_start: f64, leg_end: f64 }

async fn fibonacci_retracements_route(
    _u: AuthUser, Json(b): Json<FibBody>,
) -> Json<Option<fibonacci_retracements::FibonacciLevels>> {
    Json(fibonacci_retracements::compute(b.leg_start, b.leg_end))
}

#[derive(Deserialize)]
struct EsContributionBody {
    weights: Vec<f64>,
    covariance: Vec<Vec<f64>>,
    #[serde(default = "default_es_confidence")] confidence: f64,
}
fn default_es_confidence() -> f64 { 0.975 }

async fn es_contribution_route(
    _u: AuthUser, Json(b): Json<EsContributionBody>,
) -> Json<Option<expected_shortfall_contribution::ComponentEsReport>> {
    Json(expected_shortfall_contribution::compute(&b.weights, &b.covariance, b.confidence))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ZSpreadBody {
    cash_flows: Vec<z_spread::CashFlow>,
    spot_curve: Vec<z_spread::SpotPoint>,
    market_price: f64,
    #[serde(default = "default_zspread_tolerance")] tolerance: f64,
    #[serde(default = "default_zspread_iter")] max_iter: usize,
}
fn default_zspread_tolerance() -> f64 { 1e-8 }
fn default_zspread_iter() -> usize { 200 }

async fn z_spread_route(
    _u: AuthUser, Json(b): Json<ZSpreadBody>,
) -> Json<Option<z_spread::ZSpreadReport>> {
    Json(z_spread::solve(&b.cash_flows, &b.spot_curve, b.market_price, b.tolerance, b.max_iter))
}

#[derive(Deserialize)]
struct SwapValuationBody {
    notional: f64,
    fixed_rate: f64,
    schedule_times: Vec<f64>,
    next_reset_time: f64,
    curve: Vec<swap_valuation::SpotPoint>,
}

async fn swap_valuation_route(
    _u: AuthUser, Json(b): Json<SwapValuationBody>,
) -> Json<Option<swap_valuation::SwapValuationReport>> {
    Json(swap_valuation::value(
        b.notional, b.fixed_rate, &b.schedule_times, b.next_reset_time, &b.curve,
    ))
}

#[derive(Deserialize)]
struct CrossCurrencyBasisBody {
    spot: f64,
    forward: f64,
    domestic_rate: f64,
    foreign_rate: f64,
    time_years: f64,
}

async fn cross_currency_basis_route(
    _u: AuthUser, Json(b): Json<CrossCurrencyBasisBody>,
) -> Json<Option<cross_currency_basis::BasisReport>> {
    Json(cross_currency_basis::compute(
        b.spot, b.forward, b.domestic_rate, b.foreign_rate, b.time_years,
    ))
}

#[derive(Deserialize)]
struct GexBody { chain: Vec<gex_scanner::OptionStrike> }

async fn gex_scanner_route(
    _u: AuthUser, Json(b): Json<GexBody>,
) -> Json<Option<gex_scanner::GexReport>> {
    Json(gex_scanner::scan(&b.chain))
}

#[derive(Deserialize)]
struct UoaBody {
    contracts: Vec<unusual_options_activity::OptionContract>,
    #[serde(default)] config: Option<unusual_options_activity::Config>,
}

async fn unusual_options_activity_route(
    _u: AuthUser, Json(b): Json<UoaBody>,
) -> Json<Vec<unusual_options_activity::UoaHit>> {
    let cfg = b.config.unwrap_or_default();
    Json(unusual_options_activity::scan(&b.contracts, &cfg))
}

#[derive(Deserialize)]
struct GjrGarchBody { returns: Vec<f64> }

async fn gjr_garch_route(
    _u: AuthUser, Json(b): Json<GjrGarchBody>,
) -> Json<Option<gjr_garch::GjrGarchReport>> {
    Json(gjr_garch::estimate(&b.returns))
}

#[derive(Deserialize)]
struct FamaFrenchBody {
    excess_returns: Vec<f64>,
    mkt: Vec<f64>,
    smb: Vec<f64>,
    hml: Vec<f64>,
}

async fn fama_french_3factor_route(
    _u: AuthUser, Json(b): Json<FamaFrenchBody>,
) -> Json<Option<fama_french_3factor::FamaFrenchReport>> {
    Json(fama_french_3factor::estimate(&b.excess_returns, &b.mkt, &b.smb, &b.hml))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct MortgagePsaBody {
    original_balance: f64,
    gross_coupon: f64,
    original_term_months: usize,
    #[serde(default = "default_psa_speed")] psa_speed_pct: f64,
}
fn default_psa_speed() -> f64 { 100.0 }

async fn mortgage_psa_route(
    _u: AuthUser, Json(b): Json<MortgagePsaBody>,
) -> Json<Option<mortgage_psa::PsaScheduleReport>> {
    Json(mortgage_psa::schedule(
        b.original_balance, b.gross_coupon, b.original_term_months, b.psa_speed_pct,
    ))
}

#[derive(Deserialize)]
struct NadarayaWatsonBody {
    y: Vec<f64>,
    #[serde(default = "default_nw_bandwidth")] bandwidth: f64,
    #[serde(default)] grid: Option<Vec<f64>>,
}
fn default_nw_bandwidth() -> f64 { 5.0 }

async fn nadaraya_watson_route(
    _u: AuthUser, Json(b): Json<NadarayaWatsonBody>,
) -> Json<Vec<Option<f64>>> {
    if let Some(g) = b.grid {
        Json(nadaraya_watson::evaluate_at_grid(&b.y, &g, b.bandwidth))
    } else {
        Json(nadaraya_watson::evaluate_at_indices(&b.y, b.bandwidth))
    }
}

#[derive(Deserialize)]
struct InsiderBuyingBody {
    transactions: Vec<insider_buying_scanner::InsiderTransaction>,
    #[serde(default)] config: Option<insider_buying_scanner::Config>,
}

async fn insider_buying_route(
    _u: AuthUser, Json(b): Json<InsiderBuyingBody>,
) -> Json<Vec<insider_buying_scanner::InsiderClusterHit>> {
    let cfg = b.config.unwrap_or_default();
    Json(insider_buying_scanner::scan(&b.transactions, &cfg))
}

#[derive(Deserialize)]
struct EarningsRevisionBody {
    symbols: Vec<earnings_revision_scanner::SymbolRevisions>,
    #[serde(default)] config: Option<earnings_revision_scanner::Config>,
}

async fn earnings_revision_route(
    _u: AuthUser, Json(b): Json<EarningsRevisionBody>,
) -> Json<Vec<earnings_revision_scanner::RevisionHit>> {
    let cfg = b.config.unwrap_or_default();
    Json(earnings_revision_scanner::scan(&b.symbols, &cfg))
}

#[derive(Deserialize)]
struct BumpAndRunBody {
    bars: Vec<bump_and_run::Bar>,
    #[serde(default)] config: Option<bump_and_run::Config>,
}

async fn bump_and_run_route(
    _u: AuthUser, Json(b): Json<BumpAndRunBody>,
) -> Json<Vec<bump_and_run::BarrCandidate>> {
    let cfg = b.config.unwrap_or_default();
    Json(bump_and_run::detect(&b.bars, &cfg))
}

#[derive(Deserialize)]
struct DiamondBody {
    bars: Vec<diamond_pattern::Bar>,
    #[serde(default)] config: Option<diamond_pattern::Config>,
}

async fn diamond_pattern_route(
    _u: AuthUser, Json(b): Json<DiamondBody>,
) -> Json<Vec<diamond_pattern::DiamondCandidate>> {
    let cfg = b.config.unwrap_or_default();
    Json(diamond_pattern::detect(&b.bars, &cfg))
}

#[derive(Deserialize)]
struct PinBody { flow: Vec<probability_of_informed_trading::DailyOrderFlow> }

async fn pin_route(
    _u: AuthUser, Json(b): Json<PinBody>,
) -> Json<Option<probability_of_informed_trading::PinReport>> {
    Json(probability_of_informed_trading::estimate(&b.flow))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct MahalanobisBody { observations: Vec<Vec<f64>> }

async fn mahalanobis_distance_route(
    _u: AuthUser, Json(b): Json<MahalanobisBody>,
) -> Json<Option<mahalanobis_distance::MahalanobisReport>> {
    Json(mahalanobis_distance::compute(&b.observations))
}

#[derive(Deserialize)]
struct AcfBody {
    series: Vec<f64>,
    #[serde(default = "default_acf_max_lag")] max_lag: usize,
}
fn default_acf_max_lag() -> usize { 20 }

async fn acf_route(
    _u: AuthUser, Json(b): Json<AcfBody>,
) -> Json<Option<autocorrelation_function::AcfReport>> {
    Json(autocorrelation_function::compute(&b.series, b.max_lag))
}

#[derive(Deserialize)]
struct PacfBody {
    series: Vec<f64>,
    #[serde(default = "default_pacf_max_lag")] max_lag: usize,
}
fn default_pacf_max_lag() -> usize { 20 }

async fn pacf_route(
    _u: AuthUser, Json(b): Json<PacfBody>,
) -> Json<Option<partial_autocorrelation::PacfReport>> {
    Json(partial_autocorrelation::compute(&b.series, b.max_lag))
}

#[derive(Deserialize)]
struct PointAndFigureBody {
    prices: Vec<f64>,
    box_size: f64,
    #[serde(default = "default_pf_reversal")] reversal_boxes: usize,
}
fn default_pf_reversal() -> usize { 3 }

async fn point_and_figure_route(
    _u: AuthUser, Json(b): Json<PointAndFigureBody>,
) -> Json<Option<point_and_figure::PfReport>> {
    Json(point_and_figure::compute(&b.prices, b.box_size, b.reversal_boxes))
}

#[derive(Deserialize)]
struct DarvasBody {
    bars: Vec<darvas_box::Bar>,
    #[serde(default)] config: Option<darvas_box::Config>,
}

async fn darvas_box_route(
    _u: AuthUser, Json(b): Json<DarvasBody>,
) -> Json<Vec<darvas_box::DarvasBoxEvent>> {
    let cfg = b.config.unwrap_or_default();
    Json(darvas_box::detect(&b.bars, &cfg))
}

#[derive(Deserialize)]
struct SupertrendDualBody {
    bars: Vec<supertrend_dual::Bar>,
    #[serde(default = "default_st_fast_atr")] fast_atr_period: usize,
    #[serde(default = "default_st_fast_mult")] fast_multiplier: f64,
    #[serde(default = "default_st_slow_atr")] slow_atr_period: usize,
    #[serde(default = "default_st_slow_mult")] slow_multiplier: f64,
}
fn default_st_fast_atr() -> usize { 7 }
fn default_st_fast_mult() -> f64 { 1.5 }
fn default_st_slow_atr() -> usize { 21 }
fn default_st_slow_mult() -> f64 { 3.0 }

async fn supertrend_dual_route(
    _u: AuthUser, Json(b): Json<SupertrendDualBody>,
) -> Json<supertrend_dual::DualSupertrendReport> {
    Json(supertrend_dual::compute(
        &b.bars, b.fast_atr_period, b.fast_multiplier,
        b.slow_atr_period, b.slow_multiplier,
    ))
}

#[derive(Deserialize)]
struct HilbertBody { smoothed_price: Vec<f64> }

async fn hilbert_transform_route(
    _u: AuthUser, Json(b): Json<HilbertBody>,
) -> Json<hilbert_transform::HilbertReport> {
    Json(hilbert_transform::compute(&b.smoothed_price))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct JarqueBeraBody { sample: Vec<f64> }

async fn jarque_bera_route(
    _u: AuthUser, Json(b): Json<JarqueBeraBody>,
) -> Json<Option<jarque_bera::JarqueBeraReport>> {
    Json(jarque_bera::test(&b.sample))
}

#[derive(Deserialize)]
struct SpearmanBody { x: Vec<f64>, y: Vec<f64> }

async fn spearman_correlation_route(
    _u: AuthUser, Json(b): Json<SpearmanBody>,
) -> Json<Option<spearman_correlation::SpearmanReport>> {
    Json(spearman_correlation::compute(&b.x, &b.y))
}

#[derive(Deserialize)]
struct HarVolBody { realized_variance: Vec<f64> }

async fn har_volatility_route(
    _u: AuthUser, Json(b): Json<HarVolBody>,
) -> Json<Option<har_volatility::HarVolatilityReport>> {
    Json(har_volatility::estimate(&b.realized_variance))
}

#[derive(Deserialize)]
struct VarSwapStrikeBody {
    spot: f64,
    risk_free_rate: f64,
    time_to_expiry: f64,
    chain: Vec<variance_swap_strike::OptionQuote>,
}

async fn variance_swap_strike_route(
    _u: AuthUser, Json(b): Json<VarSwapStrikeBody>,
) -> Json<Option<variance_swap_strike::VarSwapStrikeReport>> {
    Json(variance_swap_strike::compute(
        b.spot, b.risk_free_rate, b.time_to_expiry, &b.chain,
    ))
}

#[derive(Deserialize)]
struct GaussianCopulaBody { observations: Vec<Vec<f64>> }

async fn gaussian_copula_route(
    _u: AuthUser, Json(b): Json<GaussianCopulaBody>,
) -> Json<Option<gaussian_copula::GaussianCopulaReport>> {
    Json(gaussian_copula::fit(&b.observations))
}

#[derive(Deserialize)]
struct ChowTestBody { x: Vec<f64>, y: Vec<f64>, break_index: usize }

async fn chow_test_route(
    _u: AuthUser, Json(b): Json<ChowTestBody>,
) -> Json<Option<chow_test::ChowTestReport>> {
    Json(chow_test::univariate(&b.x, &b.y, b.break_index))
}

#[derive(Deserialize)]
struct BreuschGodfreyBody {
    x: Vec<f64>,
    y: Vec<f64>,
    #[serde(default = "default_bg_lag")] lag_order: usize,
}
fn default_bg_lag() -> usize { 4 }

async fn breusch_godfrey_route(
    _u: AuthUser, Json(b): Json<BreuschGodfreyBody>,
) -> Json<Option<breusch_godfrey::BreuschGodfreyReport>> {
    Json(breusch_godfrey::test(&b.x, &b.y, b.lag_order))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct VarianceRatioBody {
    returns: Vec<f64>,
    #[serde(default = "default_vr_k")] k: usize,
}
fn default_vr_k() -> usize { 4 }

async fn variance_ratio_test_route(
    _u: AuthUser, Json(b): Json<VarianceRatioBody>,
) -> Json<Option<variance_ratio_test::VarianceRatioReport>> {
    Json(variance_ratio_test::test(&b.returns, b.k))
}

#[derive(Deserialize)]
struct RunsTestBody {
    values: Vec<f64>,
    #[serde(default)] threshold: f64,
}

async fn runs_test_route(
    _u: AuthUser, Json(b): Json<RunsTestBody>,
) -> Json<Option<runs_test::RunsTestReport>> {
    Json(runs_test::test(&b.values, b.threshold))
}

#[derive(Deserialize)]
struct CorwinSchultzBody { bars: Vec<corwin_schultz_spread::Bar> }

async fn corwin_schultz_spread_route(
    _u: AuthUser, Json(b): Json<CorwinSchultzBody>,
) -> Json<Option<corwin_schultz_spread::CorwinSchultzReport>> {
    Json(corwin_schultz_spread::compute(&b.bars))
}

#[derive(Deserialize)]
struct HampelBody {
    series: Vec<f64>,
    #[serde(default = "default_hampel_half_window")] half_window: usize,
    #[serde(default = "default_hampel_k_sigma")] k_sigma: f64,
}
fn default_hampel_half_window() -> usize { 5 }
fn default_hampel_k_sigma() -> f64 { 3.0 }

async fn hampel_filter_route(
    _u: AuthUser, Json(b): Json<HampelBody>,
) -> Json<Option<hampel_filter::HampelReport>> {
    Json(hampel_filter::compute(&b.series, b.half_window, b.k_sigma))
}

#[derive(Deserialize)]
struct BreakevenInflationBody {
    nominal_yield: f64,
    real_yield: f64,
    #[serde(default)] inflation_risk_premium: f64,
    #[serde(default)] liquidity_premium: f64,
}

async fn breakeven_inflation_route(
    _u: AuthUser, Json(b): Json<BreakevenInflationBody>,
) -> Json<Option<breakeven_inflation::BreakevenInflationReport>> {
    Json(breakeven_inflation::compute(
        b.nominal_yield, b.real_yield, b.inflation_risk_premium, b.liquidity_premium,
    ))
}

#[derive(Deserialize)]
struct CarryRollBody {
    coupon_rate_annual: f64,
    modified_duration_years: f64,
    yield_now_at_maturity_t: f64,
    yield_at_shorter_maturity_t_minus_horizon: f64,
    horizon_years: f64,
}

async fn carry_roll_decomposition_route(
    _u: AuthUser, Json(b): Json<CarryRollBody>,
) -> Json<Option<carry_roll_decomposition::CarryRollReport>> {
    Json(carry_roll_decomposition::compute(
        b.coupon_rate_annual,
        b.modified_duration_years,
        b.yield_now_at_maturity_t,
        b.yield_at_shorter_maturity_t_minus_horizon,
        b.horizon_years,
    ))
}

#[derive(Deserialize)]
struct VolTargetSizerBody {
    target_annualized_vol: f64,
    forecast_annualized_vol: f64,
    asset_price: f64,
    base_capital: f64,
    #[serde(default = "default_vts_max_leverage")] max_leverage: f64,
    #[serde(default)] prior_leverage: Option<f64>,
    #[serde(default = "default_vts_smoothing")] smoothing_alpha: f64,
}
fn default_vts_max_leverage() -> f64 { 4.0 }
fn default_vts_smoothing() -> f64 { 0.3 }

async fn vol_targeting_sizer_route(
    _u: AuthUser, Json(b): Json<VolTargetSizerBody>,
) -> Json<Option<vol_targeting_sizer::VolTargetSizerReport>> {
    Json(vol_targeting_sizer::size(
        b.target_annualized_vol,
        b.forecast_annualized_vol,
        b.asset_price,
        b.base_capital,
        b.max_leverage,
        b.prior_leverage,
        b.smoothing_alpha,
    ))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct Ks2SampleBody { sample_a: Vec<f64>, sample_b: Vec<f64> }

async fn ks_2sample_route(
    _u: AuthUser, Json(b): Json<Ks2SampleBody>,
) -> Json<Option<kolmogorov_smirnov_2sample::Ks2SampleReport>> {
    Json(kolmogorov_smirnov_2sample::test(&b.sample_a, &b.sample_b))
}

#[derive(Deserialize)]
struct AdNormalityBody { sample: Vec<f64> }

async fn ad_normality_route(
    _u: AuthUser, Json(b): Json<AdNormalityBody>,
) -> Json<Option<anderson_darling_normality::AndersonDarlingReport>> {
    Json(anderson_darling_normality::test(&b.sample))
}

#[derive(Deserialize)]
struct KpssBody {
    series: Vec<f64>,
    #[serde(default)] truncation_lag: Option<usize>,
}

async fn kpss_test_route(
    _u: AuthUser, Json(b): Json<KpssBody>,
) -> Json<Option<kpss_test::KpssReport>> {
    Json(kpss_test::test(&b.series, b.truncation_lag))
}

#[derive(Deserialize)]
struct BreuschPaganBody { x: Vec<f64>, y: Vec<f64> }

async fn breusch_pagan_test_route(
    _u: AuthUser, Json(b): Json<BreuschPaganBody>,
) -> Json<Option<breusch_pagan_test::BreuschPaganReport>> {
    Json(breusch_pagan_test::test(&b.x, &b.y))
}

#[derive(Deserialize)]
struct KlDivergenceBody { p: Vec<f64>, q: Vec<f64> }

async fn kl_divergence_route(
    _u: AuthUser, Json(b): Json<KlDivergenceBody>,
) -> Json<Option<kullback_leibler_divergence::DivergenceReport>> {
    Json(kullback_leibler_divergence::compute(&b.p, &b.q))
}

#[derive(Deserialize)]
struct Wasserstein1dBody { sample_a: Vec<f64>, sample_b: Vec<f64> }

async fn wasserstein_1d_route(
    _u: AuthUser, Json(b): Json<Wasserstein1dBody>,
) -> Json<Option<wasserstein_1d::WassersteinReport>> {
    Json(wasserstein_1d::compute(&b.sample_a, &b.sample_b))
}

#[derive(Deserialize)]
struct IvSkewBody {
    strips: Vec<iv_skew_scanner::SymbolIvStrip>,
    #[serde(default)] config: Option<iv_skew_scanner::Config>,
}

async fn iv_skew_scanner_route(
    _u: AuthUser, Json(b): Json<IvSkewBody>,
) -> Json<Vec<iv_skew_scanner::SkewHit>> {
    let cfg = b.config.unwrap_or_default();
    Json(iv_skew_scanner::scan(&b.strips, &cfg))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct TsrvBody {
    returns: Vec<f64>,
    #[serde(default)] k_subsamples: Option<usize>,
}

async fn tsrv_route(
    _u: AuthUser, Json(b): Json<TsrvBody>,
) -> Json<Option<two_scales_realized_variance::TsrvReport>> {
    Json(two_scales_realized_variance::compute(&b.returns, b.k_subsamples))
}

#[derive(Deserialize)]
struct SubsampledRvBody {
    returns: Vec<f64>,
    #[serde(default = "default_subsampled_k")] k: usize,
}
fn default_subsampled_k() -> usize { 5 }

async fn subsampled_rv_route(
    _u: AuthUser, Json(b): Json<SubsampledRvBody>,
) -> Json<Option<subsampled_realized_variance::SubsampledRvReport>> {
    Json(subsampled_realized_variance::compute(&b.returns, b.k))
}

#[derive(Deserialize)]
struct RealizedKernelBody {
    returns: Vec<f64>,
    #[serde(default)] bandwidth: Option<usize>,
    #[serde(default = "default_kernel_kind")] kernel: realized_kernel::KernelKind,
}
fn default_kernel_kind() -> realized_kernel::KernelKind { realized_kernel::KernelKind::Bartlett }

async fn realized_kernel_route(
    _u: AuthUser, Json(b): Json<RealizedKernelBody>,
) -> Json<Option<realized_kernel::RealizedKernelReport>> {
    Json(realized_kernel::compute(&b.returns, b.bandwidth, b.kernel))
}

#[derive(Deserialize)]
struct NsrBody { returns: Vec<f64> }

async fn nsr_route(
    _u: AuthUser, Json(b): Json<NsrBody>,
) -> Json<Option<noise_to_signal_ratio::NoiseToSignalReport>> {
    Json(noise_to_signal_ratio::compute(&b.returns))
}

#[derive(Deserialize)]
struct RealizedSkewnessBody { returns: Vec<f64> }

async fn realized_skewness_route(
    _u: AuthUser, Json(b): Json<RealizedSkewnessBody>,
) -> Json<Option<realized_skewness::RealizedSkewnessReport>> {
    Json(realized_skewness::compute(&b.returns))
}

#[derive(Deserialize)]
struct RealizedQuarticityBody { returns: Vec<f64> }

async fn realized_quarticity_route(
    _u: AuthUser, Json(b): Json<RealizedQuarticityBody>,
) -> Json<Option<realized_quarticity::RealizedQuarticityReport>> {
    Json(realized_quarticity::compute(&b.returns))
}

#[derive(Deserialize)]
struct MedianRvBody { returns: Vec<f64> }

async fn median_rv_route(
    _u: AuthUser, Json(b): Json<MedianRvBody>,
) -> Json<Option<median_realized_variance::MedianRvReport>> {
    Json(median_realized_variance::compute(&b.returns))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct MannWhitneyBody { sample_a: Vec<f64>, sample_b: Vec<f64> }

async fn mann_whitney_u_route(
    _u: AuthUser, Json(b): Json<MannWhitneyBody>,
) -> Json<Option<mann_whitney_u::MannWhitneyReport>> {
    Json(mann_whitney_u::test(&b.sample_a, &b.sample_b))
}

#[derive(Deserialize)]
struct WilcoxonBody { sample_x: Vec<f64>, sample_y: Vec<f64> }

async fn wilcoxon_signed_rank_route(
    _u: AuthUser, Json(b): Json<WilcoxonBody>,
) -> Json<Option<wilcoxon_signed_rank::WilcoxonReport>> {
    Json(wilcoxon_signed_rank::test(&b.sample_x, &b.sample_y))
}

#[derive(Deserialize)]
struct LeveneBody { groups: Vec<Vec<f64>> }

async fn levene_test_route(
    _u: AuthUser, Json(b): Json<LeveneBody>,
) -> Json<Option<levene_test::LeveneReport>> {
    Json(levene_test::test(&b.groups))
}

#[derive(Deserialize)]
struct RvolScanBody {
    symbols: Vec<relative_volume_scanner::SymbolVolume>,
    #[serde(default)] config: Option<relative_volume_scanner::Config>,
}

async fn relative_volume_scanner_route(
    _u: AuthUser, Json(b): Json<RvolScanBody>,
) -> Json<Vec<relative_volume_scanner::RvolHit>> {
    let cfg = b.config.unwrap_or_default();
    Json(relative_volume_scanner::scan(&b.symbols, &cfg))
}

#[derive(Deserialize)]
struct IvTermStructureBody { expiries: Vec<iv_term_structure::ExpiryIv> }

async fn iv_term_structure_route(
    _u: AuthUser, Json(b): Json<IvTermStructureBody>,
) -> Json<Option<iv_term_structure::IvTermStructureReport>> {
    Json(iv_term_structure::compute(&b.expiries))
}

#[derive(Deserialize)]
struct RamseyResetBody { x: Vec<f64>, y: Vec<f64> }

async fn ramsey_reset_route(
    _u: AuthUser, Json(b): Json<RamseyResetBody>,
) -> Json<Option<ramsey_reset::RamseyResetReport>> {
    Json(ramsey_reset::test(&b.x, &b.y))
}

#[derive(Deserialize)]
struct DistanceCorrelationBody { x: Vec<f64>, y: Vec<f64> }

async fn distance_correlation_route(
    _u: AuthUser, Json(b): Json<DistanceCorrelationBody>,
) -> Json<Option<distance_correlation::DistanceCorrelationReport>> {
    Json(distance_correlation::compute(&b.x, &b.y))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct InformationCoefficientBody {
    cross_sections: Vec<information_coefficient::CrossSection>,
    #[serde(default = "default_ic_periods_per_year")] periods_per_year: f64,
}
fn default_ic_periods_per_year() -> f64 { 252.0 }

async fn information_coefficient_route(
    _u: AuthUser, Json(b): Json<InformationCoefficientBody>,
) -> Json<Option<information_coefficient::InformationCoefficientReport>> {
    Json(information_coefficient::compute(&b.cross_sections, b.periods_per_year))
}

#[derive(Deserialize)]
struct BoxSpreadBody {
    strike_low: f64,
    strike_high: f64,
    call_low_price: f64,
    call_high_price: f64,
    put_low_price: f64,
    put_high_price: f64,
    time_to_expiry_years: f64,
    market_risk_free_rate: f64,
    #[serde(default = "default_box_arb_bps")] arbitrage_threshold_bps: f64,
}
fn default_box_arb_bps() -> f64 { 50.0 }

async fn box_spread_route(
    _u: AuthUser, Json(b): Json<BoxSpreadBody>,
) -> Json<Option<box_spread::BoxSpreadReport>> {
    Json(box_spread::compute(
        b.strike_low, b.strike_high,
        b.call_low_price, b.call_high_price,
        b.put_low_price, b.put_high_price,
        b.time_to_expiry_years, b.market_risk_free_rate,
        b.arbitrage_threshold_bps,
    ))
}

#[derive(Deserialize)]
struct JellyRollBody {
    strike: f64,
    risk_free_rate: f64,
    time_short_years: f64,
    time_long_years: f64,
    call_short_price: f64,
    call_long_price: f64,
    put_short_price: f64,
    put_long_price: f64,
    #[serde(default = "default_jr_arb_bps")] arbitrage_threshold_bps: f64,
}
fn default_jr_arb_bps() -> f64 { 25.0 }

async fn jelly_roll_arbitrage_route(
    _u: AuthUser, Json(b): Json<JellyRollBody>,
) -> Json<Option<jelly_roll_arbitrage::JellyRollReport>> {
    Json(jelly_roll_arbitrage::compute(
        b.strike, b.risk_free_rate, b.time_short_years, b.time_long_years,
        b.call_short_price, b.call_long_price, b.put_short_price, b.put_long_price,
        b.arbitrage_threshold_bps,
    ))
}

#[derive(Deserialize)]
struct FactorNeutralizationBody {
    factor_names: Vec<String>,
    inputs: Vec<factor_neutralization::NameInputs>,
}

async fn factor_neutralization_route(
    _u: AuthUser, Json(b): Json<FactorNeutralizationBody>,
) -> Json<Option<factor_neutralization::FactorNeutralizationReport>> {
    Json(factor_neutralization::neutralize(&b.factor_names, &b.inputs))
}

#[derive(Deserialize)]
struct CrpsBody { ensembles: Vec<Vec<f64>>, observations: Vec<f64> }

async fn crps_route(
    _u: AuthUser, Json(b): Json<CrpsBody>,
) -> Json<Option<continuous_ranked_probability_score::CrpsReport>> {
    Json(continuous_ranked_probability_score::ensemble(&b.ensembles, &b.observations))
}

#[derive(Deserialize)]
struct BrierBody {
    probabilities: Vec<f64>,
    outcomes: Vec<u8>,
    #[serde(default = "default_brier_bins")] n_bins: usize,
}
fn default_brier_bins() -> usize { 10 }

async fn brier_score_route(
    _u: AuthUser, Json(b): Json<BrierBody>,
) -> Json<Option<brier_score::BrierReport>> {
    Json(brier_score::compute(&b.probabilities, &b.outcomes, b.n_bins))
}

#[derive(Deserialize)]
struct DecileLongShortBody {
    names: Vec<decile_long_short_signal::NameRecord>,
    #[serde(default = "default_n_buckets")] n_buckets: usize,
}
fn default_n_buckets() -> usize { 10 }

async fn decile_long_short_route(
    _u: AuthUser, Json(b): Json<DecileLongShortBody>,
) -> Json<Option<decile_long_short_signal::DecileLongShortReport>> {
    Json(decile_long_short_signal::build(&b.names, b.n_buckets))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct NeweyWestBody {
    x: Vec<f64>,
    y: Vec<f64>,
    #[serde(default)] lag: Option<usize>,
}

async fn newey_west_route(
    _u: AuthUser, Json(b): Json<NeweyWestBody>,
) -> Json<Option<newey_west::NeweyWestReport>> {
    Json(newey_west::estimate(&b.x, &b.y, b.lag))
}

#[derive(Deserialize)]
struct DieboldMarianoBody {
    forecast_errors_1: Vec<f64>,
    forecast_errors_2: Vec<f64>,
    #[serde(default = "default_dm_loss")] loss: diebold_mariano::LossFunction,
    #[serde(default = "default_dm_horizon")] horizon: usize,
}
fn default_dm_loss() -> diebold_mariano::LossFunction { diebold_mariano::LossFunction::SquaredError }
fn default_dm_horizon() -> usize { 1 }

async fn diebold_mariano_route(
    _u: AuthUser, Json(b): Json<DieboldMarianoBody>,
) -> Json<Option<diebold_mariano::DieboldMarianoReport>> {
    Json(diebold_mariano::test(&b.forecast_errors_1, &b.forecast_errors_2, b.loss, b.horizon))
}

#[derive(Deserialize)]
struct GammaScalpingBody {
    spot_path: Vec<f64>,
    gamma: f64,
    theta_per_step: f64,
    #[serde(default)] transaction_cost_pct: f64,
    #[serde(default = "default_gs_steps_per_year")] steps_per_year: f64,
}
fn default_gs_steps_per_year() -> f64 { 252.0 }

async fn gamma_scalping_pnl_route(
    _u: AuthUser, Json(b): Json<GammaScalpingBody>,
) -> Json<Option<gamma_scalping_pnl::GammaScalpingReport>> {
    Json(gamma_scalping_pnl::simulate(
        &b.spot_path, b.gamma, b.theta_per_step, b.transaction_cost_pct, b.steps_per_year,
    ))
}

#[derive(Deserialize)]
struct BreedenLitzenbergerBody {
    call_strip: Vec<breeden_litzenberger::StrikeCall>,
    risk_free_rate: f64,
    time_to_expiry_years: f64,
}

async fn breeden_litzenberger_route(
    _u: AuthUser, Json(b): Json<BreedenLitzenbergerBody>,
) -> Json<Option<breeden_litzenberger::BreedenLitzenbergerReport>> {
    Json(breeden_litzenberger::extract(
        &b.call_strip, b.risk_free_rate, b.time_to_expiry_years,
    ))
}

#[derive(Deserialize)]
struct WhiteRobustBody {
    x: Vec<f64>,
    y: Vec<f64>,
    #[serde(default)] variant: white_robust_se::HcVariant,
}

async fn white_robust_se_route(
    _u: AuthUser, Json(b): Json<WhiteRobustBody>,
) -> Json<Option<white_robust_se::WhiteRobustReport>> {
    Json(white_robust_se::estimate(&b.x, &b.y, b.variant))
}

#[derive(Deserialize)]
struct EceBody {
    probabilities: Vec<f64>,
    outcomes: Vec<u8>,
    #[serde(default = "default_ece_bins")] n_bins: usize,
}
fn default_ece_bins() -> usize { 10 }

async fn ece_route(
    _u: AuthUser, Json(b): Json<EceBody>,
) -> Json<Option<expected_calibration_error::EceReport>> {
    Json(expected_calibration_error::compute(&b.probabilities, &b.outcomes, b.n_bins))
}

#[derive(Deserialize)]
struct VrpBody { observations: Vec<vol_risk_premium::VrpObservation> }

async fn vol_risk_premium_route(
    _u: AuthUser, Json(b): Json<VrpBody>,
) -> Json<Option<vol_risk_premium::VrpReport>> {
    Json(vol_risk_premium::compute(&b.observations))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct LiborOisBody { daily_rates: Vec<libor_ois_spread::DailyRate> }

async fn libor_ois_spread_route(
    _u: AuthUser, Json(b): Json<LiborOisBody>,
) -> Json<Option<libor_ois_spread::LiborOisReport>> {
    Json(libor_ois_spread::compute(&b.daily_rates))
}

#[derive(Deserialize)]
struct BartlettBody { groups: Vec<Vec<f64>> }

async fn bartlett_variance_test_route(
    _u: AuthUser, Json(b): Json<BartlettBody>,
) -> Json<Option<bartlett_variance_test::BartlettReport>> {
    Json(bartlett_variance_test::test(&b.groups))
}

#[derive(Deserialize)]
struct FriedmanBody { data: Vec<Vec<f64>> }

async fn friedman_test_route(
    _u: AuthUser, Json(b): Json<FriedmanBody>,
) -> Json<Option<friedman_test::FriedmanReport>> {
    Json(friedman_test::test(&b.data))
}

#[derive(Deserialize)]
struct IsotonicBody {
    x: Vec<f64>,
    y: Vec<f64>,
    #[serde(default)] decreasing: bool,
}

async fn isotonic_regression_route(
    _u: AuthUser, Json(b): Json<IsotonicBody>,
) -> Json<Option<isotonic_regression::IsotonicReport>> {
    Json(isotonic_regression::fit(&b.x, &b.y, b.decreasing))
}

#[derive(Deserialize)]
struct PeltBody {
    series: Vec<f64>,
    #[serde(default)] penalty: Option<f64>,
}

async fn pelt_segmentation_route(
    _u: AuthUser, Json(b): Json<PeltBody>,
) -> Json<Option<pelt_segmentation::PeltReport>> {
    Json(pelt_segmentation::detect(&b.series, b.penalty))
}

#[derive(Deserialize)]
struct GonzaloGrangerBody { price_1: Vec<f64>, price_2: Vec<f64> }

async fn gonzalo_granger_route(
    _u: AuthUser, Json(b): Json<GonzaloGrangerBody>,
) -> Json<Option<gonzalo_granger_decomposition::GonzaloGrangerReport>> {
    Json(gonzalo_granger_decomposition::decompose(&b.price_1, &b.price_2))
}

#[derive(Deserialize)]
struct MonteCarloVarBody {
    weights: Vec<f64>,
    mean_returns: Vec<f64>,
    cholesky_lower: Vec<Vec<f64>>,
    #[serde(default = "default_mc_confidence")] confidence: f64,
    #[serde(default = "default_mc_n_sim")] n_simulations: usize,
    #[serde(default)] seed: u64,
}
fn default_mc_confidence() -> f64 { 0.95 }
fn default_mc_n_sim() -> usize { 10_000 }

async fn monte_carlo_var_route(
    _u: AuthUser, Json(b): Json<MonteCarloVarBody>,
) -> Json<Option<monte_carlo_var::MonteCarloVarReport>> {
    Json(monte_carlo_var::simulate(
        &b.weights, &b.mean_returns, &b.cholesky_lower,
        b.confidence, b.n_simulations, b.seed,
    ))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct GpdTailFitBody { losses: Vec<f64>, threshold: f64 }

async fn gpd_tail_fit_route(
    _u: AuthUser, Json(b): Json<GpdTailFitBody>,
) -> Json<Option<gpd_tail_fit::GpdFitReport>> {
    Json(gpd_tail_fit::fit(&b.losses, b.threshold))
}

#[derive(Deserialize)]
struct PotBody {
    losses: Vec<f64>,
    #[serde(default = "default_pot_quantile")] quantile: f64,
    #[serde(default)] mrl_grid: Vec<f64>,
}
fn default_pot_quantile() -> f64 { 0.95 }

async fn peaks_over_threshold_route(
    _u: AuthUser, Json(b): Json<PotBody>,
) -> Json<Option<peaks_over_threshold::PotReport>> {
    Json(peaks_over_threshold::run(&b.losses, b.quantile, &b.mrl_grid))
}

#[derive(Deserialize)]
struct EvtVarBody {
    threshold: f64,
    n_exceedances: usize,
    n_total: usize,
    shape_xi: f64,
    scale_beta: f64,
    #[serde(default = "default_evt_confidence")] confidence: f64,
}
fn default_evt_confidence() -> f64 { 0.99 }

async fn evt_value_at_risk_route(
    _u: AuthUser, Json(b): Json<EvtVarBody>,
) -> Json<Option<evt_value_at_risk::EvtVarReport>> {
    Json(evt_value_at_risk::compute(
        b.threshold, b.n_exceedances, b.n_total,
        b.shape_xi, b.scale_beta, b.confidence,
    ))
}

#[derive(Deserialize)]
struct PickandsBody {
    losses: Vec<f64>,
    #[serde(default = "default_pickands_ks")] k_values: Vec<usize>,
}
fn default_pickands_ks() -> Vec<usize> { vec![10, 25, 50, 100] }

async fn pickands_estimator_route(
    _u: AuthUser, Json(b): Json<PickandsBody>,
) -> Json<Option<pickands_estimator::PickandsReport>> {
    Json(pickands_estimator::compute(&b.losses, &b.k_values))
}

#[derive(Deserialize)]
struct EcdfBody {
    sample: Vec<f64>,
    #[serde(default = "default_ecdf_confidence")] confidence: f64,
}
fn default_ecdf_confidence() -> f64 { 0.95 }

async fn ecdf_route(
    _u: AuthUser, Json(b): Json<EcdfBody>,
) -> Json<Option<empirical_distribution_function::EcdfReport>> {
    Json(empirical_distribution_function::compute(&b.sample, b.confidence))
}

#[derive(Deserialize)]
struct QuantileRegressionBody {
    x: Vec<f64>,
    y: Vec<f64>,
    #[serde(default = "default_qreg_tau")] tau: f64,
}
fn default_qreg_tau() -> f64 { 0.5 }

async fn quantile_regression_route(
    _u: AuthUser, Json(b): Json<QuantileRegressionBody>,
) -> Json<Option<quantile_regression::QuantileRegressionReport>> {
    Json(quantile_regression::fit(&b.x, &b.y, b.tau))
}

#[derive(Deserialize)]
struct MegaphoneBody {
    bars: Vec<megaphone_pattern::Bar>,
    #[serde(default)] config: Option<megaphone_pattern::Config>,
}

async fn megaphone_pattern_route(
    _u: AuthUser, Json(b): Json<MegaphoneBody>,
) -> Json<Vec<megaphone_pattern::MegaphoneCandidate>> {
    let cfg = b.config.unwrap_or_default();
    Json(megaphone_pattern::detect(&b.bars, &cfg))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct RollingDrawdownBody {
    equity: Vec<f64>,
    #[serde(default = "default_dd_window")] window: usize,
}
fn default_dd_window() -> usize { 60 }

async fn rolling_drawdown_route(
    _u: AuthUser, Json(b): Json<RollingDrawdownBody>,
) -> Json<Option<rolling_drawdown::RollingDrawdownReport>> {
    Json(rolling_drawdown::compute(&b.equity, b.window))
}

#[derive(Deserialize)]
struct RollingSharpeBody {
    returns: Vec<f64>,
    #[serde(default = "default_sharpe_window")] window: usize,
    #[serde(default = "default_periods_per_year_252")] periods_per_year: f64,
    #[serde(default)] risk_free_per_period: f64,
}
fn default_sharpe_window() -> usize { 60 }
fn default_periods_per_year_252() -> f64 { 252.0 }

async fn rolling_sharpe_route(
    _u: AuthUser, Json(b): Json<RollingSharpeBody>,
) -> Json<Vec<Option<f64>>> {
    Json(rolling_sharpe::compute(
        &b.returns, b.window, b.periods_per_year, b.risk_free_per_period,
    ))
}

#[derive(Deserialize)]
struct RollingSortinoBody {
    returns: Vec<f64>,
    #[serde(default = "default_sharpe_window")] window: usize,
    #[serde(default = "default_periods_per_year_252")] periods_per_year: f64,
    #[serde(default)] minimum_acceptable_return: f64,
}

async fn rolling_sortino_route(
    _u: AuthUser, Json(b): Json<RollingSortinoBody>,
) -> Json<Vec<Option<f64>>> {
    Json(rolling_sortino::compute(
        &b.returns, b.window, b.periods_per_year, b.minimum_acceptable_return,
    ))
}

#[derive(Deserialize)]
struct RollingBetaBody {
    asset_returns: Vec<f64>,
    benchmark_returns: Vec<f64>,
    #[serde(default = "default_sharpe_window")] window: usize,
    #[serde(default = "default_periods_per_year_252")] periods_per_year: f64,
}

async fn rolling_beta_route(
    _u: AuthUser, Json(b): Json<RollingBetaBody>,
) -> Json<Option<rolling_beta::RollingBetaReport>> {
    Json(rolling_beta::compute(
        &b.asset_returns, &b.benchmark_returns, b.window, b.periods_per_year,
    ))
}

#[derive(Deserialize)]
struct ExpectedDrawdownBody {
    drift_per_period: f64,
    vol_per_period: f64,
    horizon: usize,
    #[serde(default = "default_edd_n_paths")] n_paths: usize,
    #[serde(default)] seed: u64,
}
fn default_edd_n_paths() -> usize { 5_000 }

async fn expected_drawdown_route(
    _u: AuthUser, Json(b): Json<ExpectedDrawdownBody>,
) -> Json<Option<expected_drawdown::ExpectedDrawdownReport>> {
    Json(expected_drawdown::simulate(
        b.drift_per_period, b.vol_per_period, b.horizon, b.n_paths, b.seed,
    ))
}

#[derive(Deserialize)]
struct EngleGrangerBody {
    y: Vec<f64>,
    x: Vec<f64>,
    #[serde(default = "default_eg_lags")] adf_lags: usize,
}
fn default_eg_lags() -> usize { 2 }

async fn engle_granger_2step_route(
    _u: AuthUser, Json(b): Json<EngleGrangerBody>,
) -> Json<Option<engle_granger_2step::EngleGrangerReport>> {
    Json(engle_granger_2step::test(&b.y, &b.x, b.adf_lags))
}

#[derive(Deserialize)]
struct VcpBody {
    bars: Vec<vcp_pattern::Bar>,
    #[serde(default)] config: Option<vcp_pattern::Config>,
}

async fn vcp_pattern_route(
    _u: AuthUser, Json(b): Json<VcpBody>,
) -> Json<Vec<vcp_pattern::VcpCandidate>> {
    let cfg = b.config.unwrap_or_default();
    Json(vcp_pattern::detect(&b.bars, &cfg))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct BurkeBody {
    equity: Vec<f64>,
    #[serde(default)] risk_free_total: f64,
    #[serde(default = "default_periods_per_year_252")] periods_per_year: f64,
}

async fn burke_ratio_route(
    _u: AuthUser, Json(b): Json<BurkeBody>,
) -> Json<Option<burke_ratio::BurkeReport>> {
    Json(burke_ratio::compute(&b.equity, b.risk_free_total, b.periods_per_year))
}

#[derive(Deserialize)]
struct SterlingBody {
    equity: Vec<f64>,
    #[serde(default = "default_periods_per_year_252")] periods_per_year: f64,
    #[serde(default)] risk_free_annualized: f64,
    #[serde(default = "default_sterling_k")] top_k: usize,
}
fn default_sterling_k() -> usize { 3 }

async fn sterling_ratio_route(
    _u: AuthUser, Json(b): Json<SterlingBody>,
) -> Json<Option<sterling_ratio::SterlingReport>> {
    Json(sterling_ratio::compute(
        &b.equity, b.periods_per_year, b.risk_free_annualized, b.top_k,
    ))
}

#[derive(Deserialize)]
struct RecoveryFactorBody {
    equity: Vec<f64>,
    #[serde(default = "default_periods_per_year_252")] periods_per_year: f64,
}

async fn recovery_factor_route(
    _u: AuthUser, Json(b): Json<RecoveryFactorBody>,
) -> Json<Option<recovery_factor::RecoveryFactorReport>> {
    Json(recovery_factor::compute(&b.equity, b.periods_per_year))
}

#[derive(Deserialize)]
struct GainToPainBody { returns: Vec<f64> }

async fn gain_to_pain_ratio_route(
    _u: AuthUser, Json(b): Json<GainToPainBody>,
) -> Json<Option<gain_to_pain_ratio::GainToPainReport>> {
    Json(gain_to_pain_ratio::compute(&b.returns))
}

#[derive(Deserialize)]
struct TailRatioBody {
    returns: Vec<f64>,
    #[serde(default = "default_tail_upper")] upper_q: f64,
    #[serde(default = "default_tail_lower")] lower_q: f64,
}
fn default_tail_upper() -> f64 { 0.95 }
fn default_tail_lower() -> f64 { 0.05 }

async fn tail_ratio_route(
    _u: AuthUser, Json(b): Json<TailRatioBody>,
) -> Json<Option<tail_ratio::TailRatioReport>> {
    Json(tail_ratio::compute(&b.returns, b.upper_q, b.lower_q))
}

#[derive(Deserialize)]
struct WeinsteinStagesBody {
    closes: Vec<f64>,
    #[serde(default = "default_weinstein_ma")] ma_period: usize,
    #[serde(default = "default_weinstein_slope_window")] ma_slope_window: usize,
    #[serde(default = "default_weinstein_band")] band_pct: f64,
}
fn default_weinstein_ma() -> usize { 150 }
fn default_weinstein_slope_window() -> usize { 25 }
fn default_weinstein_band() -> f64 { 0.01 }

async fn weinstein_stages_route(
    _u: AuthUser, Json(b): Json<WeinsteinStagesBody>,
) -> Json<Option<weinstein_stages::WeinsteinStagesReport>> {
    Json(weinstein_stages::classify(
        &b.closes, b.ma_period, b.ma_slope_window, b.band_pct,
    ))
}

#[derive(Deserialize)]
struct ExpectancyBody { trade_pnls: Vec<f64> }

async fn expectancy_per_trade_route(
    _u: AuthUser, Json(b): Json<ExpectancyBody>,
) -> Json<Option<expectancy_per_trade::ExpectancyReport>> {
    Json(expectancy_per_trade::compute(&b.trade_pnls))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct KellyDiscreteBody { win_probability: f64, win_loss_payoff_ratio: f64 }

async fn kelly_discrete_route(
    _u: AuthUser, Json(b): Json<KellyDiscreteBody>,
) -> Json<Option<kelly_criterion::KellyReport>> {
    Json(kelly_criterion::discrete(b.win_probability, b.win_loss_payoff_ratio))
}

#[derive(Deserialize)]
struct KellyContinuousBody {
    expected_return: f64,
    return_volatility: f64,
    #[serde(default)] risk_free_rate: f64,
}

async fn kelly_continuous_route(
    _u: AuthUser, Json(b): Json<KellyContinuousBody>,
) -> Json<Option<kelly_criterion::KellyReport>> {
    Json(kelly_criterion::continuous(b.expected_return, b.return_volatility, b.risk_free_rate))
}

#[derive(Deserialize)]
struct TrackingErrorBody {
    portfolio_returns: Vec<f64>,
    benchmark_returns: Vec<f64>,
    #[serde(default = "default_periods_per_year_252")] periods_per_year: f64,
}

async fn tracking_error_route(
    _u: AuthUser, Json(b): Json<TrackingErrorBody>,
) -> Json<Option<tracking_error::TrackingErrorReport>> {
    Json(tracking_error::compute(
        &b.portfolio_returns, &b.benchmark_returns, b.periods_per_year,
    ))
}

#[derive(Deserialize)]
struct ActiveShareBody { weights: Vec<active_share::WeightPair> }

async fn active_share_route(
    _u: AuthUser, Json(b): Json<ActiveShareBody>,
) -> Json<Option<active_share::ActiveShareReport>> {
    Json(active_share::compute(&b.weights))
}

#[derive(Deserialize)]
struct SavitzkyGolayBody {
    series: Vec<f64>,
    #[serde(default = "default_sg_window")] window: usize,
    #[serde(default = "default_sg_order")] polynomial_order: usize,
}
fn default_sg_window() -> usize { 7 }
fn default_sg_order() -> usize { 3 }

async fn savitzky_golay_route(
    _u: AuthUser, Json(b): Json<SavitzkyGolayBody>,
) -> Json<Option<savitzky_golay::SavitzkyGolayReport>> {
    Json(savitzky_golay::compute(&b.series, b.window, b.polynomial_order))
}

#[derive(Deserialize)]
struct MinerviniBody {
    closes: Vec<f64>,
    #[serde(default = "default_minervini_rs")] relative_strength_rank: f64,
}
fn default_minervini_rs() -> f64 { 80.0 }

async fn minervini_route(
    _u: AuthUser, Json(b): Json<MinerviniBody>,
) -> Json<Option<minervini_trend_template::MinerviniReport>> {
    Json(minervini_trend_template::classify(&b.closes, b.relative_strength_rank))
}

#[derive(Deserialize)]
struct PocketPivotBody {
    bars: Vec<pocket_pivot_buy::Bar>,
    #[serde(default)] config: Option<pocket_pivot_buy::Config>,
}

async fn pocket_pivot_route(
    _u: AuthUser, Json(b): Json<PocketPivotBody>,
) -> Json<Vec<pocket_pivot_buy::PocketPivotEvent>> {
    let cfg = b.config.unwrap_or_default();
    Json(pocket_pivot_buy::detect(&b.bars, &cfg))
}

#[derive(Deserialize)]
struct BootstrapPnlBody {
    trade_pnls: Vec<f64>,
    #[serde(default = "default_boot_n")] n_resamples: usize,
    #[serde(default)] seed: u64,
}
fn default_boot_n() -> usize { 5000 }

async fn bootstrap_pnl_route(
    _u: AuthUser, Json(b): Json<BootstrapPnlBody>,
) -> Json<Option<bootstrap_pnl::BootstrapPnlReport>> {
    Json(bootstrap_pnl::bootstrap(&b.trade_pnls, b.n_resamples, b.seed))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct FdOptionBody {
    spot: f64,
    strike: f64,
    time_to_expiry: f64,
    risk_free_rate: f64,
    #[serde(default)] dividend_yield: f64,
    volatility: f64,
    option_type: finite_difference_option::OptionType,
    #[serde(default = "default_fd_s_steps")] n_s_steps: usize,
    #[serde(default = "default_fd_t_steps")] n_t_steps: usize,
    #[serde(default = "default_fd_s_max_mult")] s_max_multiplier: f64,
}
fn default_fd_s_steps() -> usize { 200 }
fn default_fd_t_steps() -> usize { 100 }
fn default_fd_s_max_mult() -> f64 { 4.0 }

async fn finite_difference_option_route(
    _u: AuthUser, Json(b): Json<FdOptionBody>,
) -> Json<Option<finite_difference_option::FdOptionReport>> {
    Json(finite_difference_option::price(
        b.spot, b.strike, b.time_to_expiry, b.risk_free_rate, b.dividend_yield,
        b.volatility, b.option_type, b.n_s_steps, b.n_t_steps, b.s_max_multiplier,
    ))
}

#[derive(Deserialize)]
struct McOptionBody {
    spot: f64,
    strike: f64,
    time_to_expiry: f64,
    risk_free_rate: f64,
    #[serde(default)] dividend_yield: f64,
    volatility: f64,
    option_type: monte_carlo_option::OptionType,
    #[serde(default = "default_mc_n_paths")] n_paths: usize,
    #[serde(default)] seed: u64,
    #[serde(default = "default_mc_antithetic")] use_antithetic: bool,
}
fn default_mc_n_paths() -> usize { 50_000 }
fn default_mc_antithetic() -> bool { true }

async fn monte_carlo_option_route(
    _u: AuthUser, Json(b): Json<McOptionBody>,
) -> Json<Option<monte_carlo_option::McOptionReport>> {
    Json(monte_carlo_option::price(
        b.spot, b.strike, b.time_to_expiry, b.risk_free_rate, b.dividend_yield,
        b.volatility, b.option_type, b.n_paths, b.seed, b.use_antithetic,
    ))
}

#[derive(Deserialize)]
struct ForwardStartBody {
    spot: f64,
    moneyness_factor_alpha: f64,
    time_to_strike_set_years: f64,
    time_strike_to_expiry_years: f64,
    risk_free_rate: f64,
    #[serde(default)] dividend_yield: f64,
    volatility: f64,
    option_type: forward_start_option::OptionType,
}

async fn forward_start_option_route(
    _u: AuthUser, Json(b): Json<ForwardStartBody>,
) -> Json<Option<forward_start_option::ForwardStartReport>> {
    Json(forward_start_option::price(
        b.spot, b.moneyness_factor_alpha,
        b.time_to_strike_set_years, b.time_strike_to_expiry_years,
        b.risk_free_rate, b.dividend_yield, b.volatility, b.option_type,
    ))
}

#[derive(Deserialize)]
struct RoundingBody {
    closes: Vec<f64>,
    #[serde(default)] config: Option<rounding_pattern::Config>,
}

async fn rounding_pattern_route(
    _u: AuthUser, Json(b): Json<RoundingBody>,
) -> Json<Vec<rounding_pattern::SaucerCandidate>> {
    let cfg = b.config.unwrap_or_default();
    Json(rounding_pattern::detect(&b.closes, &cfg))
}

#[derive(Deserialize)]
struct IslandBody {
    bars: Vec<island_reversal::Bar>,
    #[serde(default)] config: Option<island_reversal::Config>,
}

async fn island_reversal_route(
    _u: AuthUser, Json(b): Json<IslandBody>,
) -> Json<Vec<island_reversal::IslandCandidate>> {
    let cfg = b.config.unwrap_or_default();
    Json(island_reversal::detect(&b.bars, &cfg))
}

#[derive(Deserialize)]
struct TedSpreadBody { daily_rates: Vec<ted_spread::DailyRate> }

async fn ted_spread_route(
    _u: AuthUser, Json(b): Json<TedSpreadBody>,
) -> Json<Option<ted_spread::TedSpreadReport>> {
    Json(ted_spread::compute(&b.daily_rates))
}

#[derive(Deserialize)]
struct VolManagedBody {
    returns: Vec<f64>,
    #[serde(default = "default_vm_lookback")] lookback: usize,
    target_annualized_vol: f64,
    #[serde(default = "default_periods_per_year_252")] periods_per_year: f64,
    #[serde(default = "default_vm_max_leverage")] max_leverage: f64,
}
fn default_vm_lookback() -> usize { 60 }
fn default_vm_max_leverage() -> f64 { 4.0 }

async fn volatility_managed_portfolio_route(
    _u: AuthUser, Json(b): Json<VolManagedBody>,
) -> Json<Option<volatility_managed_portfolio::VolManagedReport>> {
    Json(volatility_managed_portfolio::compute(
        &b.returns, b.lookback, b.target_annualized_vol, b.periods_per_year, b.max_leverage,
    ))
}

// ──────────────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct VolSwapBody {
    variance_strike: f64,
    vol_of_vol_annualized: f64,
    time_to_expiry_years: f64,
}

async fn volatility_swap_route(
    _u: AuthUser, Json(b): Json<VolSwapBody>,
) -> Json<Option<volatility_swap::VolatilitySwapReport>> {
    Json(volatility_swap::fair_strike(
        b.variance_strike, b.vol_of_vol_annualized, b.time_to_expiry_years,
    ))
}

#[derive(Deserialize)]
struct NssBody {
    points: Vec<nelson_siegel_svensson::CurvePoint>,
    #[serde(default = "default_nss_svensson_tau1")] tau_1: f64,
    #[serde(default = "default_nss_svensson_tau2")] tau_2: f64,
}
fn default_nss_svensson_tau1() -> f64 { 1.0 }
fn default_nss_svensson_tau2() -> f64 { 5.0 }

async fn nelson_siegel_svensson_route(
    _u: AuthUser, Json(b): Json<NssBody>,
) -> Json<Option<nelson_siegel_svensson::NssFitReport>> {
    Json(nelson_siegel_svensson::fit(&b.points, b.tau_1, b.tau_2))
}

#[derive(Deserialize)]
struct PpTestBody {
    series: Vec<f64>,
    #[serde(default)] bandwidth_lag: Option<usize>,
}

async fn pp_test_route(
    _u: AuthUser, Json(b): Json<PpTestBody>,
) -> Json<Option<pp_test::PpTestReport>> {
    Json(pp_test::test(&b.series, b.bandwidth_lag))
}

#[derive(Deserialize)]
struct KeyReversalBody {
    bars: Vec<key_reversal_bar::Bar>,
    #[serde(default)] config: Option<key_reversal_bar::Config>,
}

async fn key_reversal_bar_route(
    _u: AuthUser, Json(b): Json<KeyReversalBody>,
) -> Json<Vec<key_reversal_bar::KeyReversalEvent>> {
    let cfg = b.config.unwrap_or_default();
    Json(key_reversal_bar::detect(&b.bars, &cfg))
}

#[derive(Deserialize)]
struct MomentumCrashBody {
    momentum_returns: Vec<f64>,
    #[serde(default = "default_mcp_vol_lookback")] vol_lookback: usize,
    target_annualized_vol: f64,
    #[serde(default = "default_periods_per_year_252")] periods_per_year: f64,
    #[serde(default = "default_mcp_max_leverage")] max_leverage: f64,
    #[serde(default = "default_mcp_crash_lookback")] crash_filter_lookback: usize,
    #[serde(default = "default_mcp_crash_threshold")] crash_filter_threshold_pct: f64,
}
fn default_mcp_vol_lookback() -> usize { 60 }
fn default_mcp_max_leverage() -> f64 { 4.0 }
fn default_mcp_crash_lookback() -> usize { 22 }
fn default_mcp_crash_threshold() -> f64 { -0.20 }

async fn momentum_crash_protection_route(
    _u: AuthUser, Json(b): Json<MomentumCrashBody>,
) -> Json<Option<momentum_crash_protection::CrashProtectionReport>> {
    Json(momentum_crash_protection::manage(
        &b.momentum_returns, b.vol_lookback, b.target_annualized_vol,
        b.periods_per_year, b.max_leverage,
        b.crash_filter_lookback, b.crash_filter_threshold_pct,
    ))
}

#[derive(Deserialize)]
struct TCopulaBody {
    observations: Vec<Vec<f64>>,
    #[serde(default = "default_t_copula_dof")] degrees_of_freedom: f64,
}
fn default_t_copula_dof() -> f64 { 5.0 }

async fn t_copula_route(
    _u: AuthUser, Json(b): Json<TCopulaBody>,
) -> Json<Option<t_copula::TCopulaReport>> {
    Json(t_copula::fit(&b.observations, b.degrees_of_freedom))
}

#[derive(Deserialize)]
struct WelchPeriodogramBody {
    series: Vec<f64>,
    #[serde(default = "default_welch_seg")] segment_length: usize,
    #[serde(default = "default_welch_overlap")] overlap_fraction: f64,
}
fn default_welch_seg() -> usize { 64 }
fn default_welch_overlap() -> f64 { 0.5 }

async fn welch_periodogram_route(
    _u: AuthUser, Json(b): Json<WelchPeriodogramBody>,
) -> Json<Option<welch_periodogram::WelchPeriodogramReport>> {
    Json(welch_periodogram::compute(&b.series, b.segment_length, b.overlap_fraction))
}


#[derive(Deserialize)]
struct WilliamsAdBody { bars: Vec<williams_accumulation_distribution::Bar> }

async fn williams_accumulation_distribution_route(
    _u: AuthUser, Json(b): Json<WilliamsAdBody>,
) -> Json<Vec<Option<f64>>> {
    Json(williams_accumulation_distribution::compute(&b.bars))
}

#[derive(Deserialize)]
struct ChandeTrendBody {
    closes: Vec<f64>,
    #[serde(default = "default_cti_period")] period: usize,
}
fn default_cti_period() -> usize { 14 }

async fn chande_trend_index_route(
    _u: AuthUser, Json(b): Json<ChandeTrendBody>,
) -> Json<Vec<Option<f64>>> {
    Json(chande_trend_index::compute(&b.closes, b.period))
}

#[derive(Deserialize)]
struct BalanceOfPowerBody {
    bars: Vec<balance_of_power::Bar>,
    #[serde(default = "default_bop_smoothing")] smoothing_period: usize,
}
fn default_bop_smoothing() -> usize { 14 }

async fn balance_of_power_route(
    _u: AuthUser, Json(b): Json<BalanceOfPowerBody>,
) -> Json<balance_of_power::BalanceOfPowerReport> {
    Json(balance_of_power::compute(&b.bars, b.smoothing_period))
}

#[derive(Deserialize)]
struct RelativeVolatilityBody {
    highs: Vec<f64>,
    lows: Vec<f64>,
    closes: Vec<f64>,
    #[serde(default = "default_rvi_period")] period: usize,
}
fn default_rvi_period() -> usize { 14 }

async fn relative_volatility_index_route(
    _u: AuthUser, Json(b): Json<RelativeVolatilityBody>,
) -> Json<Vec<Option<f64>>> {
    Json(relative_volatility_index::compute(&b.highs, &b.lows, &b.closes, b.period))
}

#[derive(Deserialize)]
struct DemarkerBody {
    highs: Vec<f64>,
    lows: Vec<f64>,
    #[serde(default = "default_demarker_period")] period: usize,
}
fn default_demarker_period() -> usize { 14 }

async fn demarker_oscillator_route(
    _u: AuthUser, Json(b): Json<DemarkerBody>,
) -> Json<Vec<Option<f64>>> {
    Json(demarker_oscillator::compute(&b.highs, &b.lows, b.period))
}

#[derive(Deserialize)]
struct WoodiesCciBody {
    bars: Vec<woodies_cci::Bar>,
    #[serde(default = "default_woodies_turbo")] turbo_period: usize,
    #[serde(default = "default_woodies_standard")] standard_period: usize,
    #[serde(default = "default_woodies_tlb")] tlb_period: usize,
}
fn default_woodies_turbo() -> usize { 6 }
fn default_woodies_standard() -> usize { 14 }
fn default_woodies_tlb() -> usize { 25 }

async fn woodies_cci_route(
    _u: AuthUser, Json(b): Json<WoodiesCciBody>,
) -> Json<woodies_cci::WoodiesCciReport> {
    Json(woodies_cci::compute(&b.bars, b.turbo_period, b.standard_period, b.tlb_period))
}

#[derive(Deserialize)]
struct PremierStochasticBody {
    bars: Vec<premier_stochastic::Bar>,
    #[serde(default = "default_pso_stoch")] stoch_period: usize,
    #[serde(default = "default_pso_s1")] smoothing_1: usize,
    #[serde(default = "default_pso_s2")] smoothing_2: usize,
}
fn default_pso_stoch() -> usize { 8 }
fn default_pso_s1() -> usize { 5 }
fn default_pso_s2() -> usize { 3 }

async fn premier_stochastic_route(
    _u: AuthUser, Json(b): Json<PremierStochasticBody>,
) -> Json<Vec<Option<f64>>> {
    Json(premier_stochastic::compute(&b.bars, b.stoch_period, b.smoothing_1, b.smoothing_2))
}


#[derive(Deserialize)]
struct QStickBody {
    bars: Vec<qstick::Bar>,
    #[serde(default = "default_qstick_period")] period: usize,
}
fn default_qstick_period() -> usize { 8 }

async fn qstick_route(
    _u: AuthUser, Json(b): Json<QStickBody>,
) -> Json<Vec<Option<f64>>> {
    Json(qstick::compute(&b.bars, b.period))
}

#[derive(Deserialize)]
struct KstBody { closes: Vec<f64> }

async fn know_sure_thing_route(
    _u: AuthUser, Json(b): Json<KstBody>,
) -> Json<know_sure_thing::KstReport> {
    Json(know_sure_thing::compute(&b.closes))
}

#[derive(Deserialize)]
struct DisparityBody {
    closes: Vec<f64>,
    #[serde(default = "default_disparity_period")] period: usize,
}
fn default_disparity_period() -> usize { 14 }

async fn disparity_index_route(
    _u: AuthUser, Json(b): Json<DisparityBody>,
) -> Json<Vec<Option<f64>>> {
    Json(disparity_index::compute(&b.closes, b.period))
}

#[derive(Deserialize)]
struct CamarillaBody { session: camarilla_pivots::PriorSession }

async fn camarilla_pivots_route(
    _u: AuthUser, Json(b): Json<CamarillaBody>,
) -> Json<Option<camarilla_pivots::CamarillaLevels>> {
    Json(camarilla_pivots::compute(b.session))
}

#[derive(Deserialize)]
struct LrChannelBody {
    closes: Vec<f64>,
    #[serde(default = "default_lr_period")] period: usize,
    #[serde(default = "default_lr_stdev")] n_stdev: f64,
}
fn default_lr_period() -> usize { 20 }
fn default_lr_stdev() -> f64 { 2.0 }

async fn linear_regression_channel_route(
    _u: AuthUser, Json(b): Json<LrChannelBody>,
) -> Json<linear_regression_channel::LinearRegressionChannelReport> {
    Json(linear_regression_channel::compute(&b.closes, b.period, b.n_stdev))
}

#[derive(Deserialize)]
struct GatorBody {
    highs: Vec<f64>,
    lows: Vec<f64>,
    #[serde(default = "default_gator_jaw_p")] jaw_period: usize,
    #[serde(default = "default_gator_jaw_s")] jaw_shift: usize,
    #[serde(default = "default_gator_teeth_p")] teeth_period: usize,
    #[serde(default = "default_gator_teeth_s")] teeth_shift: usize,
    #[serde(default = "default_gator_lips_p")] lips_period: usize,
    #[serde(default = "default_gator_lips_s")] lips_shift: usize,
}
fn default_gator_jaw_p() -> usize { 13 }
fn default_gator_jaw_s() -> usize { 8 }
fn default_gator_teeth_p() -> usize { 8 }
fn default_gator_teeth_s() -> usize { 5 }
fn default_gator_lips_p() -> usize { 5 }
fn default_gator_lips_s() -> usize { 3 }

async fn gator_oscillator_route(
    _u: AuthUser, Json(b): Json<GatorBody>,
) -> Json<gator_oscillator::GatorOscillatorReport> {
    Json(gator_oscillator::compute(&b.highs, &b.lows,
        b.jaw_period, b.jaw_shift,
        b.teeth_period, b.teeth_shift,
        b.lips_period, b.lips_shift))
}

#[derive(Deserialize)]
struct TriangularMaBody {
    closes: Vec<f64>,
    #[serde(default = "default_tma_period")] period: usize,
}
fn default_tma_period() -> usize { 20 }

async fn triangular_ma_route(
    _u: AuthUser, Json(b): Json<TriangularMaBody>,
) -> Json<Vec<Option<f64>>> {
    Json(triangular_ma::compute(&b.closes, b.period))
}


#[derive(Deserialize)]
struct PvtBody { bars: Vec<price_volume_trend::Bar> }

async fn price_volume_trend_route(
    _u: AuthUser, Json(b): Json<PvtBody>,
) -> Json<Vec<Option<f64>>> {
    Json(price_volume_trend::compute(&b.bars))
}

#[derive(Deserialize)]
struct NviBody { bars: Vec<negative_volume_index::Bar> }

async fn negative_volume_index_route(
    _u: AuthUser, Json(b): Json<NviBody>,
) -> Json<Vec<Option<f64>>> {
    Json(negative_volume_index::compute(&b.bars))
}

#[derive(Deserialize)]
struct PviBody { bars: Vec<positive_volume_index::Bar> }

async fn positive_volume_index_route(
    _u: AuthUser, Json(b): Json<PviBody>,
) -> Json<Vec<Option<f64>>> {
    Json(positive_volume_index::compute(&b.bars))
}

#[derive(Deserialize)]
struct StarcBody {
    bars: Vec<starc_bands::Bar>,
    #[serde(default = "default_starc_sma")] sma_period: usize,
    #[serde(default = "default_starc_atr")] atr_period: usize,
    #[serde(default = "default_starc_mul")] multiplier: f64,
}
fn default_starc_sma() -> usize { 5 }
fn default_starc_atr() -> usize { 15 }
fn default_starc_mul() -> f64 { 2.0 }

async fn starc_bands_route(
    _u: AuthUser, Json(b): Json<StarcBody>,
) -> Json<starc_bands::StarcBandsReport> {
    Json(starc_bands::compute(&b.bars, b.sma_period, b.atr_period, b.multiplier))
}

#[derive(Deserialize)]
struct GuppyBody { closes: Vec<f64> }

async fn guppy_mma_route(
    _u: AuthUser, Json(b): Json<GuppyBody>,
) -> Json<guppy_mma::GuppyMmaReport> {
    Json(guppy_mma::compute(&b.closes))
}

#[derive(Deserialize)]
struct AsiBody {
    bars: Vec<accumulation_swing_index::Bar>,
    limit_move: f64,
}

async fn accumulation_swing_index_route(
    _u: AuthUser, Json(b): Json<AsiBody>,
) -> Json<Vec<Option<f64>>> {
    Json(accumulation_swing_index::compute(&b.bars, b.limit_move))
}

#[derive(Deserialize)]
struct TwiggsBody {
    bars: Vec<twiggs_money_flow::Bar>,
    #[serde(default = "default_tmf_period")] period: usize,
}
fn default_tmf_period() -> usize { 21 }

async fn twiggs_money_flow_route(
    _u: AuthUser, Json(b): Json<TwiggsBody>,
) -> Json<Vec<Option<f64>>> {
    Json(twiggs_money_flow::compute(&b.bars, b.period))
}

#[derive(Deserialize)]
struct ElderSafeZoneBody {
    bars: Vec<elder_safezone_stop::Bar>,
    #[serde(default = "default_elder_period")] period: usize,
    #[serde(default = "default_elder_k")] k_multiplier: f64,
}
fn default_elder_period() -> usize { 22 }
fn default_elder_k() -> f64 { 3.0 }

async fn elder_safezone_stop_route(
    _u: AuthUser, Json(b): Json<ElderSafeZoneBody>,
) -> Json<elder_safezone_stop::ElderSafeZoneReport> {
    Json(elder_safezone_stop::compute(&b.bars, b.period, b.k_multiplier))
}


#[derive(Deserialize)]
struct JmaBody {
    series: Vec<f64>,
    #[serde(default = "default_jma_length")] length: usize,
    #[serde(default = "default_jma_phase")] phase: f64,
    #[serde(default = "default_jma_power")] power: f64,
}
fn default_jma_length() -> usize { 14 }
fn default_jma_phase() -> f64 { 0.0 }
fn default_jma_power() -> f64 { 1.0 }

async fn jurik_ma_route(
    _u: AuthUser, Json(b): Json<JmaBody>,
) -> Json<Vec<Option<f64>>> {
    Json(jurik_ma::compute(&b.series, b.length, b.phase, b.power))
}

#[derive(Deserialize)]
struct ChandeKrollBody {
    bars: Vec<chande_kroll_stop::Bar>,
    #[serde(default = "default_ck_p")] p: usize,
    #[serde(default = "default_ck_x")] x: f64,
    #[serde(default = "default_ck_q")] q: usize,
}
fn default_ck_p() -> usize { 10 }
fn default_ck_x() -> f64 { 1.0 }
fn default_ck_q() -> usize { 9 }

async fn chande_kroll_stop_route(
    _u: AuthUser, Json(b): Json<ChandeKrollBody>,
) -> Json<chande_kroll_stop::ChandeKrollReport> {
    Json(chande_kroll_stop::compute(&b.bars, b.p, b.x, b.q))
}

#[derive(Deserialize)]
struct ElderThermoBody {
    bars: Vec<elder_thermometer::Bar>,
    #[serde(default = "default_thermo_period")] period: usize,
}
fn default_thermo_period() -> usize { 22 }

async fn elder_thermometer_route(
    _u: AuthUser, Json(b): Json<ElderThermoBody>,
) -> Json<elder_thermometer::ElderThermometerReport> {
    Json(elder_thermometer::compute(&b.bars, b.period))
}

#[derive(Deserialize)]
struct FloorPivotsBody { session: floor_pivots::PriorSession }

async fn floor_pivots_route(
    _u: AuthUser, Json(b): Json<FloorPivotsBody>,
) -> Json<Option<floor_pivots::FloorPivotLevels>> {
    Json(floor_pivots::compute(b.session))
}

#[derive(Deserialize)]
struct TdiBody {
    closes: Vec<f64>,
    #[serde(default = "default_tdi_rsi")] rsi_period: usize,
    #[serde(default = "default_tdi_price")] price_period: usize,
    #[serde(default = "default_tdi_signal")] signal_period: usize,
    #[serde(default = "default_tdi_band")] band_period: usize,
    #[serde(default = "default_tdi_stdev")] n_stdev: f64,
}
fn default_tdi_rsi() -> usize { 14 }
fn default_tdi_price() -> usize { 2 }
fn default_tdi_signal() -> usize { 7 }
fn default_tdi_band() -> usize { 34 }
fn default_tdi_stdev() -> f64 { 1.6185 }

async fn traders_dynamic_index_route(
    _u: AuthUser, Json(b): Json<TdiBody>,
) -> Json<traders_dynamic_index::TdiReport> {
    Json(traders_dynamic_index::compute(&b.closes, b.rsi_period, b.price_period,
        b.signal_period, b.band_period, b.n_stdev))
}

#[derive(Deserialize)]
struct TtmSqueezeBody {
    bars: Vec<ttm_squeeze::Bar>,
    #[serde(default = "default_ttm_period")] period: usize,
    #[serde(default = "default_ttm_bb")] bb_mult: f64,
    #[serde(default = "default_ttm_kc")] kc_mult: f64,
}
fn default_ttm_period() -> usize { 20 }
fn default_ttm_bb() -> f64 { 2.0 }
fn default_ttm_kc() -> f64 { 1.5 }

async fn ttm_squeeze_route(
    _u: AuthUser, Json(b): Json<TtmSqueezeBody>,
) -> Json<ttm_squeeze::TtmSqueezeReport> {
    Json(ttm_squeeze::compute(&b.bars, b.period, b.bb_mult, b.kc_mult))
}

#[derive(Deserialize)]
struct EwoBody {
    bars: Vec<elliott_wave_oscillator::Bar>,
    #[serde(default = "default_ewo_fast")] fast: usize,
    #[serde(default = "default_ewo_slow")] slow: usize,
}
fn default_ewo_fast() -> usize { 5 }
fn default_ewo_slow() -> usize { 35 }

async fn elliott_wave_oscillator_route(
    _u: AuthUser, Json(b): Json<EwoBody>,
) -> Json<Vec<Option<f64>>> {
    Json(elliott_wave_oscillator::compute(&b.bars, b.fast, b.slow))
}


#[derive(Deserialize)]
struct WoodieBody { session: woodie_pivots::PriorSession }

async fn woodie_pivots_route(
    _u: AuthUser, Json(b): Json<WoodieBody>,
) -> Json<Option<woodie_pivots::WoodiePivotLevels>> {
    Json(woodie_pivots::compute(b.session))
}

#[derive(Deserialize)]
struct FibPivotsBody { session: fibonacci_pivots::PriorSession }

async fn fibonacci_pivots_route(
    _u: AuthUser, Json(b): Json<FibPivotsBody>,
) -> Json<Option<fibonacci_pivots::FibPivotLevels>> {
    Json(fibonacci_pivots::compute(b.session))
}

#[derive(Deserialize)]
struct PgoBody {
    bars: Vec<pretty_good_oscillator::Bar>,
    #[serde(default = "default_pgo_period")] period: usize,
}
fn default_pgo_period() -> usize { 14 }

async fn pretty_good_oscillator_route(
    _u: AuthUser, Json(b): Json<PgoBody>,
) -> Json<Vec<Option<f64>>> {
    Json(pretty_good_oscillator::compute(&b.bars, b.period))
}

#[derive(Deserialize)]
struct RoofingBody {
    series: Vec<f64>,
    #[serde(default = "default_roofing_hp")] hp_period: usize,
    #[serde(default = "default_roofing_ss")] ss_period: usize,
}
fn default_roofing_hp() -> usize { 48 }
fn default_roofing_ss() -> usize { 10 }

async fn roofing_filter_route(
    _u: AuthUser, Json(b): Json<RoofingBody>,
) -> Json<Vec<Option<f64>>> {
    Json(roofing_filter::compute(&b.series, b.hp_period, b.ss_period))
}

#[derive(Deserialize)]
struct WeissWaveBody {
    bars: Vec<weiss_wave::Bar>,
    #[serde(default = "default_weiss_pct")] reversal_pct: f64,
}
fn default_weiss_pct() -> f64 { 2.0 }

async fn weiss_wave_route(
    _u: AuthUser, Json(b): Json<WeissWaveBody>,
) -> Json<Vec<Option<f64>>> {
    Json(weiss_wave::compute(&b.bars, b.reversal_pct))
}

#[derive(Deserialize)]
struct TtmTrendBody {
    bars: Vec<ttm_trend::Bar>,
    #[serde(default = "default_ttm_trend_lookback")] lookback: usize,
}
fn default_ttm_trend_lookback() -> usize { 5 }

async fn ttm_trend_route(
    _u: AuthUser, Json(b): Json<TtmTrendBody>,
) -> Json<Vec<Option<ttm_trend::TtmTrendState>>> {
    Json(ttm_trend::compute(&b.bars, b.lookback))
}

#[derive(Deserialize)]
struct VqiBody {
    bars: Vec<volatility_quality_index::Bar>,
    #[serde(default = "default_vqi_norm")] normalization_period: usize,
}
fn default_vqi_norm() -> usize { 14 }

async fn volatility_quality_index_route(
    _u: AuthUser, Json(b): Json<VqiBody>,
) -> Json<volatility_quality_index::VqiReport> {
    Json(volatility_quality_index::compute(&b.bars, b.normalization_period))
}


#[derive(Deserialize)]
struct DemarkPivotsBody { session: demark_pivots::PriorSession }

async fn demark_pivots_route(
    _u: AuthUser, Json(b): Json<DemarkPivotsBody>,
) -> Json<Option<demark_pivots::DemarkPivotLevels>> {
    Json(demark_pivots::compute(b.session))
}

#[derive(Deserialize)]
struct GannHlaBody {
    bars: Vec<gann_high_low_activator::Bar>,
    #[serde(default = "default_gann_hla_period")] period: usize,
}
fn default_gann_hla_period() -> usize { 5 }

async fn gann_high_low_activator_route(
    _u: AuthUser, Json(b): Json<GannHlaBody>,
) -> Json<gann_high_low_activator::GannHlaReport> {
    Json(gann_high_low_activator::compute(&b.bars, b.period))
}

#[derive(Deserialize)]
struct ImpulseBody {
    closes: Vec<f64>,
    #[serde(default = "default_impulse_fast")] fast_period: usize,
    #[serde(default = "default_impulse_macd_fast")] macd_fast: usize,
    #[serde(default = "default_impulse_macd_slow")] macd_slow: usize,
    #[serde(default = "default_impulse_macd_signal")] macd_signal: usize,
}
fn default_impulse_fast() -> usize { 13 }
fn default_impulse_macd_fast() -> usize { 12 }
fn default_impulse_macd_slow() -> usize { 26 }
fn default_impulse_macd_signal() -> usize { 9 }

async fn impulse_system_route(
    _u: AuthUser, Json(b): Json<ImpulseBody>,
) -> Json<Vec<Option<impulse_system::ImpulseColor>>> {
    Json(impulse_system::compute(&b.closes, b.fast_period,
        b.macd_fast, b.macd_slow, b.macd_signal))
}

#[derive(Deserialize)]
struct DamianiBody {
    bars: Vec<damiani_volatmeter::Bar>,
    #[serde(default = "default_damiani_fast")] fast_period: usize,
    #[serde(default = "default_damiani_slow")] slow_period: usize,
    #[serde(default = "default_damiani_thr")] threshold: f64,
}
fn default_damiani_fast() -> usize { 3 }
fn default_damiani_slow() -> usize { 13 }
fn default_damiani_thr() -> f64 { 1.4 }

async fn damiani_volatmeter_route(
    _u: AuthUser, Json(b): Json<DamianiBody>,
) -> Json<damiani_volatmeter::DamianiReport> {
    Json(damiani_volatmeter::compute(&b.bars, b.fast_period, b.slow_period, b.threshold))
}

#[derive(Deserialize)]
struct ITrendBody {
    series: Vec<f64>,
    #[serde(default = "default_itrend_period")] period: usize,
}
fn default_itrend_period() -> usize { 14 }

async fn ehlers_instant_trendline_route(
    _u: AuthUser, Json(b): Json<ITrendBody>,
) -> Json<ehlers_instant_trendline::InstantTrendlineReport> {
    Json(ehlers_instant_trendline::compute(&b.series, b.period))
}

#[derive(Deserialize)]
struct RangeFilterBody {
    closes: Vec<f64>,
    #[serde(default = "default_rf_period")] n_range: usize,
    #[serde(default = "default_rf_mult")] multiplier: f64,
}
fn default_rf_period() -> usize { 20 }
fn default_rf_mult() -> f64 { 3.5 }

async fn range_filter_route(
    _u: AuthUser, Json(b): Json<RangeFilterBody>,
) -> Json<range_filter::RangeFilterReport> {
    Json(range_filter::compute(&b.closes, b.n_range, b.multiplier))
}

#[derive(Deserialize)]
struct LindaRaschkeBody {
    closes: Vec<f64>,
    #[serde(default = "default_lr_fast")] fast_period: usize,
    #[serde(default = "default_lr_slow")] slow_period: usize,
    #[serde(default = "default_lr_signal")] signal_period: usize,
}
fn default_lr_fast() -> usize { 3 }
fn default_lr_slow() -> usize { 10 }
fn default_lr_signal() -> usize { 16 }

async fn linda_raschke_3_10_route(
    _u: AuthUser, Json(b): Json<LindaRaschkeBody>,
) -> Json<linda_raschke_3_10::ThreeTenReport> {
    Json(linda_raschke_3_10::compute(&b.closes, b.fast_period,
        b.slow_period, b.signal_period))
}


#[derive(Deserialize)]
struct MamaFamaBody {
    series: Vec<f64>,
    #[serde(default = "default_mama_fast")] fast_limit: f64,
    #[serde(default = "default_mama_slow")] slow_limit: f64,
}
fn default_mama_fast() -> f64 { 0.5 }
fn default_mama_slow() -> f64 { 0.05 }

async fn ehlers_mama_fama_route(
    _u: AuthUser, Json(b): Json<MamaFamaBody>,
) -> Json<ehlers_mama_fama::MamaFamaReport> {
    Json(ehlers_mama_fama::compute(&b.series, b.fast_limit, b.slow_limit))
}

#[derive(Deserialize)]
struct DssBody {
    bars: Vec<bressert_dss::Bar>,
    #[serde(default = "default_dss_stoch")] stoch_period: usize,
    #[serde(default = "default_dss_ema")] ema_period: usize,
}
fn default_dss_stoch() -> usize { 13 }
fn default_dss_ema() -> usize { 8 }

async fn bressert_dss_route(
    _u: AuthUser, Json(b): Json<DssBody>,
) -> Json<Vec<Option<f64>>> {
    Json(bressert_dss::compute(&b.bars, b.stoch_period, b.ema_period))
}

#[derive(Deserialize)]
struct TazBody {
    closes: Vec<f64>,
    #[serde(default = "default_taz_fast")] fast_period: usize,
    #[serde(default = "default_taz_slow")] slow_period: usize,
}
fn default_taz_fast() -> usize { 10 }
fn default_taz_slow() -> usize { 30 }

async fn traders_action_zone_route(
    _u: AuthUser, Json(b): Json<TazBody>,
) -> Json<traders_action_zone::TazReport> {
    Json(traders_action_zone::compute(&b.closes, b.fast_period, b.slow_period))
}

#[derive(Deserialize)]
struct IiiBody { bars: Vec<intraday_intensity_index::Bar> }

async fn intraday_intensity_index_route(
    _u: AuthUser, Json(b): Json<IiiBody>,
) -> Json<intraday_intensity_index::IiiReport> {
    Json(intraday_intensity_index::compute(&b.bars))
}

#[derive(Deserialize)]
struct CdmiBody {
    closes: Vec<f64>,
    #[serde(default = "default_cdmi_const")] td_const: usize,
    #[serde(default = "default_cdmi_std")] std_period: usize,
    #[serde(default = "default_cdmi_min")] td_min: usize,
    #[serde(default = "default_cdmi_max")] td_max: usize,
}
fn default_cdmi_const() -> usize { 14 }
fn default_cdmi_std() -> usize { 5 }
fn default_cdmi_min() -> usize { 5 }
fn default_cdmi_max() -> usize { 30 }

async fn chande_dynamic_momentum_index_route(
    _u: AuthUser, Json(b): Json<CdmiBody>,
) -> Json<Vec<Option<f64>>> {
    Json(chande_dynamic_momentum_index::compute(&b.closes,
        b.td_const, b.std_period, b.td_min, b.td_max))
}

#[derive(Deserialize)]
struct SebBody {
    closes: Vec<f64>,
    #[serde(default = "default_seb_period")] period: usize,
    #[serde(default = "default_seb_smooth")] smoothing: usize,
    #[serde(default = "default_seb_mult")] multiplier: f64,
}
fn default_seb_period() -> usize { 21 }
fn default_seb_smooth() -> usize { 3 }
fn default_seb_mult() -> f64 { 2.0 }

async fn standard_error_bands_route(
    _u: AuthUser, Json(b): Json<SebBody>,
) -> Json<standard_error_bands::StandardErrorBandsReport> {
    Json(standard_error_bands::compute(&b.closes, b.period, b.smoothing, b.multiplier))
}

#[derive(Deserialize)]
struct EhlersCtiBody {
    closes: Vec<f64>,
    #[serde(default = "default_ehlers_cti_period")] period: usize,
}
fn default_ehlers_cti_period() -> usize { 20 }

async fn ehlers_correlation_trend_indicator_route(
    _u: AuthUser, Json(b): Json<EhlersCtiBody>,
) -> Json<Vec<Option<f64>>> {
    Json(ehlers_correlation_trend_indicator::compute(&b.closes, b.period))
}


#[derive(Deserialize)]
struct ChandelierExitBody {
    bars: Vec<chandelier_exit::Bar>,
    #[serde(default = "default_chandelier_exit_period")] period: usize,
    #[serde(default = "default_chandelier_exit_mult")] multiplier: f64,
}
fn default_chandelier_exit_period() -> usize { 22 }
fn default_chandelier_exit_mult() -> f64 { 3.0 }

async fn chandelier_exit_route(
    _u: AuthUser, Json(b): Json<ChandelierExitBody>,
) -> Json<chandelier_exit::ChandelierReport> {
    Json(chandelier_exit::compute(&b.bars, b.period, b.multiplier))
}

#[derive(Deserialize)]
struct HolyGrailBody {
    bars: Vec<holy_grail::Bar>,
    #[serde(default = "default_hg_ema")] ema_period: usize,
    #[serde(default = "default_hg_adx")] adx_period: usize,
    #[serde(default = "default_hg_threshold")] adx_threshold: f64,
}
fn default_hg_ema() -> usize { 20 }
fn default_hg_adx() -> usize { 14 }
fn default_hg_threshold() -> f64 { 30.0 }

async fn holy_grail_route(
    _u: AuthUser, Json(b): Json<HolyGrailBody>,
) -> Json<holy_grail::HolyGrailReport> {
    Json(holy_grail::compute(&b.bars, b.ema_period, b.adx_period, b.adx_threshold))
}

#[derive(Deserialize)]
struct VolumeOscillatorBody {
    volumes: Vec<f64>,
    #[serde(default = "default_vo_fast")] fast_period: usize,
    #[serde(default = "default_vo_slow")] slow_period: usize,
}
fn default_vo_fast() -> usize { 14 }
fn default_vo_slow() -> usize { 28 }

async fn volume_oscillator_route(
    _u: AuthUser, Json(b): Json<VolumeOscillatorBody>,
) -> Json<Vec<Option<f64>>> {
    Json(volume_oscillator::compute(&b.volumes, b.fast_period, b.slow_period))
}

#[derive(Deserialize)]
struct ChopZoneBody {
    bars: Vec<chop_zone_indicator::Bar>,
    #[serde(default = "default_chop_zone_period")] period: usize,
}
fn default_chop_zone_period() -> usize { 30 }

async fn chop_zone_indicator_route(
    _u: AuthUser, Json(b): Json<ChopZoneBody>,
) -> Json<chop_zone_indicator::ChopZoneReport> {
    Json(chop_zone_indicator::compute(&b.bars, b.period))
}

#[derive(Deserialize)]
struct AlphaTrendBody {
    bars: Vec<alphatrend::Bar>,
    #[serde(default = "default_alphatrend_period")] period: usize,
    #[serde(default = "default_alphatrend_mult")] multiplier: f64,
}
fn default_alphatrend_period() -> usize { 14 }
fn default_alphatrend_mult() -> f64 { 1.0 }

async fn alphatrend_route(
    _u: AuthUser, Json(b): Json<AlphaTrendBody>,
) -> Json<alphatrend::AlphaTrendReport> {
    Json(alphatrend::compute(&b.bars, b.period, b.multiplier))
}

#[derive(Deserialize)]
struct LrsBody {
    closes: Vec<f64>,
    #[serde(default = "default_lrs_period")] period: usize,
}
fn default_lrs_period() -> usize { 14 }

async fn linear_regression_slope_route(
    _u: AuthUser, Json(b): Json<LrsBody>,
) -> Json<Vec<Option<f64>>> {
    Json(linear_regression_slope::compute(&b.closes, b.period))
}

#[derive(Deserialize)]
struct UpiBody {
    equity_curve: Vec<f64>,
    #[serde(default)] risk_free_rate: f64,
    #[serde(default = "default_upi_ppy")] periods_per_year: f64,
}
fn default_upi_ppy() -> f64 { 252.0 }

async fn ulcer_performance_index_route(
    _u: AuthUser, Json(b): Json<UpiBody>,
) -> Json<Option<ulcer_performance_index::UlcerPerformanceReport>> {
    Json(ulcer_performance_index::compute(&b.equity_curve, b.risk_free_rate, b.periods_per_year))
}


#[derive(Deserialize)]
struct BbPercentBBody {
    closes: Vec<f64>,
    #[serde(default = "default_bbb_period")] period: usize,
    #[serde(default = "default_bbb_stdev")] n_stdev: f64,
}
fn default_bbb_period() -> usize { 20 }
fn default_bbb_stdev() -> f64 { 2.0 }

async fn bollinger_percent_b_route(
    _u: AuthUser, Json(b): Json<BbPercentBBody>,
) -> Json<Vec<Option<f64>>> {
    Json(bollinger_percent_b::compute(&b.closes, b.period, b.n_stdev))
}

#[derive(Deserialize)]
struct VzoBody {
    bars: Vec<volume_zone_oscillator::Bar>,
    #[serde(default = "default_vzo_period")] period: usize,
}
fn default_vzo_period() -> usize { 14 }

async fn volume_zone_oscillator_route(
    _u: AuthUser, Json(b): Json<VzoBody>,
) -> Json<Vec<Option<f64>>> {
    Json(volume_zone_oscillator::compute(&b.bars, b.period))
}

#[derive(Deserialize)]
struct GartleyBody {
    pivots: Vec<gartley_pattern::Pivot>,
    #[serde(default = "default_gartley_tol")] tolerance: f64,
}
fn default_gartley_tol() -> f64 { 0.05 }

async fn gartley_pattern_route(
    _u: AuthUser, Json(b): Json<GartleyBody>,
) -> Json<Vec<gartley_pattern::GartleyMatch>> {
    Json(gartley_pattern::detect(&b.pivots, b.tolerance))
}

#[derive(Deserialize)]
struct PinballBody {
    bars: Vec<pinball_setup::Bar>,
    #[serde(default = "default_pinball_sma")] sma_period: usize,
    #[serde(default = "default_pinball_lookback")] lookback: usize,
}
fn default_pinball_sma() -> usize { 50 }
fn default_pinball_lookback() -> usize { 20 }

async fn pinball_setup_route(
    _u: AuthUser, Json(b): Json<PinballBody>,
) -> Json<pinball_setup::PinballReport> {
    Json(pinball_setup::compute(&b.bars, b.sma_period, b.lookback))
}

#[derive(Deserialize)]
struct AvpBody {
    bars: Vec<accumulation_volume_pattern::Bar>,
    #[serde(default = "default_avp_lookback")] lookback: usize,
}
fn default_avp_lookback() -> usize { 50 }

async fn accumulation_volume_pattern_route(
    _u: AuthUser, Json(b): Json<AvpBody>,
) -> Json<accumulation_volume_pattern::AvpReport> {
    Json(accumulation_volume_pattern::compute(&b.bars, b.lookback))
}

#[derive(Deserialize)]
struct RmaBody {
    series: Vec<f64>,
    #[serde(default = "default_rma_period")] period: usize,
}
fn default_rma_period() -> usize { 14 }

async fn recursive_ma_route(
    _u: AuthUser, Json(b): Json<RmaBody>,
) -> Json<Vec<Option<f64>>> {
    Json(recursive_ma::compute(&b.series, b.period))
}

#[derive(Deserialize)]
struct DspBody {
    closes: Vec<f64>,
    #[serde(default = "default_dsp_period")] period: usize,
}
fn default_dsp_period() -> usize { 14 }

async fn detrended_synthetic_price_route(
    _u: AuthUser, Json(b): Json<DspBody>,
) -> Json<Vec<Option<f64>>> {
    Json(detrended_synthetic_price::compute(&b.closes, b.period))
}


#[derive(Deserialize)]
struct HarmonicPatternBody {
    pivots: Vec<gartley_pattern::Pivot>,
    #[serde(default = "default_harmonic_pattern_tol")] tolerance: f64,
}
fn default_harmonic_pattern_tol() -> f64 { 0.05 }

async fn bat_pattern_route(
    _u: AuthUser, Json(b): Json<HarmonicPatternBody>,
) -> Json<Vec<bat_pattern::BatMatch>> {
    Json(bat_pattern::detect(&b.pivots, b.tolerance))
}

async fn butterfly_pattern_route(
    _u: AuthUser, Json(b): Json<HarmonicPatternBody>,
) -> Json<Vec<butterfly_pattern::ButterflyMatch>> {
    Json(butterfly_pattern::detect(&b.pivots, b.tolerance))
}

async fn crab_pattern_route(
    _u: AuthUser, Json(b): Json<HarmonicPatternBody>,
) -> Json<Vec<crab_pattern::CrabMatch>> {
    Json(crab_pattern::detect(&b.pivots, b.tolerance))
}

async fn cypher_pattern_route(
    _u: AuthUser, Json(b): Json<HarmonicPatternBody>,
) -> Json<Vec<cypher_pattern::CypherMatch>> {
    Json(cypher_pattern::detect(&b.pivots, b.tolerance))
}

async fn shark_pattern_route(
    _u: AuthUser, Json(b): Json<HarmonicPatternBody>,
) -> Json<Vec<shark_pattern::SharkMatch>> {
    Json(shark_pattern::detect(&b.pivots, b.tolerance))
}

#[derive(Deserialize)]
struct TurtleSoupBody {
    bars: Vec<turtle_soup::Bar>,
    #[serde(default = "default_turtle_lookback")] lookback: usize,
    #[serde(default = "default_turtle_confirm")] confirm_bars: usize,
}
fn default_turtle_lookback() -> usize { 20 }
fn default_turtle_confirm() -> usize { 2 }

async fn turtle_soup_route(
    _u: AuthUser, Json(b): Json<TurtleSoupBody>,
) -> Json<turtle_soup::TurtleSoupReport> {
    Json(turtle_soup::compute(&b.bars, b.lookback, b.confirm_bars))
}

#[derive(Deserialize)]
struct EightyTwentyBody {
    bars: Vec<eighty_twenty_setup::Bar>,
    #[serde(default = "default_eighty_twenty_lookback")] lookback: usize,
}
fn default_eighty_twenty_lookback() -> usize { 20 }

async fn eighty_twenty_setup_route(
    _u: AuthUser, Json(b): Json<EightyTwentyBody>,
) -> Json<eighty_twenty_setup::EightyTwentyReport> {
    Json(eighty_twenty_setup::compute(&b.bars, b.lookback))
}


#[derive(Deserialize)]
struct ZScoreBody {
    closes: Vec<f64>,
    #[serde(default = "default_zscore_period")] period: usize,
}
fn default_zscore_period() -> usize { 20 }

async fn z_score_indicator_route(
    _u: AuthUser, Json(b): Json<ZScoreBody>,
) -> Json<Vec<Option<f64>>> {
    Json(z_score_indicator::compute(&b.closes, b.period))
}

#[derive(Deserialize)]
struct VfiBody {
    bars: Vec<katsanos_vfi::Bar>,
    #[serde(default = "default_vfi_period")] period: usize,
    #[serde(default = "default_vfi_smooth")] smoothing: usize,
}
fn default_vfi_period() -> usize { 130 }
fn default_vfi_smooth() -> usize { 3 }

async fn katsanos_vfi_route(
    _u: AuthUser, Json(b): Json<VfiBody>,
) -> Json<Vec<Option<f64>>> {
    Json(katsanos_vfi::compute(&b.bars, b.period, b.smoothing))
}

#[derive(Deserialize)]
struct LrcBody {
    closes: Vec<f64>,
    #[serde(default = "default_lrc_period")] period: usize,
}
fn default_lrc_period() -> usize { 14 }

async fn linear_regression_curve_route(
    _u: AuthUser, Json(b): Json<LrcBody>,
) -> Json<Vec<Option<f64>>> {
    Json(linear_regression_curve::compute(&b.closes, b.period))
}

#[derive(Deserialize)]
struct MaEnvelopeBody {
    closes: Vec<f64>,
    #[serde(default = "default_mae_period")] period: usize,
    #[serde(default = "default_mae_pct")] pct: f64,
    #[serde(default)] use_ema: bool,
}
fn default_mae_period() -> usize { 20 }
fn default_mae_pct() -> f64 { 2.5 }

async fn moving_average_envelope_route(
    _u: AuthUser, Json(b): Json<MaEnvelopeBody>,
) -> Json<moving_average_envelope::MaEnvelopeReport> {
    Json(moving_average_envelope::compute(&b.closes, b.period, b.pct, b.use_ema))
}

#[derive(Deserialize)]
struct BbSqueezeBody {
    closes: Vec<f64>,
    #[serde(default = "default_bbsq_period")] bb_period: usize,
    #[serde(default = "default_bbsq_stdev")] n_stdev: f64,
    #[serde(default = "default_bbsq_lookback")] lookback: usize,
    #[serde(default = "default_bbsq_slack")] slack: f64,
}
fn default_bbsq_period() -> usize { 20 }
fn default_bbsq_stdev() -> f64 { 2.0 }
fn default_bbsq_lookback() -> usize { 125 }
fn default_bbsq_slack() -> f64 { 0.05 }

async fn bollinger_squeeze_route(
    _u: AuthUser, Json(b): Json<BbSqueezeBody>,
) -> Json<bollinger_squeeze::BollingerSqueezeReport> {
    Json(bollinger_squeeze::compute(&b.closes, b.bb_period, b.n_stdev, b.lookback, b.slack))
}

#[derive(Deserialize)]
struct AntiSetupBody {
    bars: Vec<anti_setup::Bar>,
    #[serde(default = "default_anti_stoch")] stoch_period: usize,
    #[serde(default = "default_anti_d")] d_period: usize,
    #[serde(default = "default_anti_short")] trend_short: usize,
    #[serde(default = "default_anti_long")] trend_long: usize,
    #[serde(default = "default_anti_lookback")] lookback: usize,
}
fn default_anti_stoch() -> usize { 14 }
fn default_anti_d() -> usize { 3 }
fn default_anti_short() -> usize { 20 }
fn default_anti_long() -> usize { 50 }
fn default_anti_lookback() -> usize { 5 }

async fn anti_setup_route(
    _u: AuthUser, Json(b): Json<AntiSetupBody>,
) -> Json<anti_setup::AntiReport> {
    Json(anti_setup::compute(&b.bars, b.stoch_period, b.d_period,
        b.trend_short, b.trend_long, b.lookback))
}

#[derive(Deserialize)]
struct CmiBody {
    bars: Vec<choppy_market_index::Bar>,
    #[serde(default = "default_cmi_period")] period: usize,
}
fn default_cmi_period() -> usize { 14 }

async fn choppy_market_index_route(
    _u: AuthUser, Json(b): Json<CmiBody>,
) -> Json<Vec<Option<f64>>> {
    Json(choppy_market_index::compute(&b.bars, b.period))
}


#[derive(Deserialize)]
struct MadridBody { closes: Vec<f64> }

async fn madrid_moving_average_ribbon_route(
    _u: AuthUser, Json(b): Json<MadridBody>,
) -> Json<madrid_moving_average_ribbon::MadridRibbonReport> {
    Json(madrid_moving_average_ribbon::compute(&b.closes))
}

#[derive(Deserialize)]
struct VelocityBody {
    bars: Vec<velocity_indicator::Bar>,
    #[serde(default = "default_velocity_period")] period: usize,
}
fn default_velocity_period() -> usize { 14 }

async fn velocity_indicator_route(
    _u: AuthUser, Json(b): Json<VelocityBody>,
) -> Json<Vec<Option<f64>>> {
    Json(velocity_indicator::compute(&b.bars, b.period))
}

#[derive(Deserialize)]
struct VbsBody {
    bars: Vec<volatility_breakout_system::Bar>,
    #[serde(default = "default_vbs_period")] period: usize,
    #[serde(default = "default_vbs_mult")] multiplier: f64,
}
fn default_vbs_period() -> usize { 5 }
fn default_vbs_mult() -> f64 { 0.5 }

async fn volatility_breakout_system_route(
    _u: AuthUser, Json(b): Json<VbsBody>,
) -> Json<volatility_breakout_system::VolatilityBreakoutReport> {
    Json(volatility_breakout_system::compute(&b.bars, b.period, b.multiplier))
}

#[derive(Deserialize)]
struct DvoBody {
    bars: Vec<detrended_volatility_oscillator::Bar>,
    #[serde(default = "default_dvo_mean")] mean_period: usize,
    #[serde(default = "default_dvo_rank")] rank_period: usize,
}
fn default_dvo_mean() -> usize { 5 }
fn default_dvo_rank() -> usize { 252 }

async fn detrended_volatility_oscillator_route(
    _u: AuthUser, Json(b): Json<DvoBody>,
) -> Json<Vec<Option<f64>>> {
    Json(detrended_volatility_oscillator::compute(&b.bars, b.mean_period, b.rank_period))
}

#[derive(Deserialize)]
struct AdOscBody {
    bars: Vec<accumulation_distribution_oscillator::Bar>,
    #[serde(default = "default_ad_osc_period")] period: usize,
}
fn default_ad_osc_period() -> usize { 14 }

async fn accumulation_distribution_oscillator_route(
    _u: AuthUser, Json(b): Json<AdOscBody>,
) -> Json<accumulation_distribution_oscillator::AdOscillatorReport> {
    Json(accumulation_distribution_oscillator::compute(&b.bars, b.period))
}

#[derive(Deserialize)]
struct CsiBody {
    bars: Vec<candle_strength_index::Bar>,
    #[serde(default = "default_csi_period")] period: usize,
}
fn default_csi_period() -> usize { 14 }

async fn candle_strength_index_route(
    _u: AuthUser, Json(b): Json<CsiBody>,
) -> Json<Vec<Option<f64>>> {
    Json(candle_strength_index::compute(&b.bars, b.period))
}

#[derive(Deserialize)]
struct HawkinsZonesBody {
    oscillator: Vec<Option<f64>>,
    #[serde(default = "default_hawkins_period")] period: usize,
    #[serde(default = "default_hawkins_pct")] pct: f64,
}
fn default_hawkins_period() -> usize { 70 }
fn default_hawkins_pct() -> f64 { 90.0 }

async fn hawkins_dynamic_zones_route(
    _u: AuthUser, Json(b): Json<HawkinsZonesBody>,
) -> Json<hawkins_dynamic_zones::HawkinsZonesReport> {
    Json(hawkins_dynamic_zones::compute(&b.oscillator, b.period, b.pct))
}


#[derive(Deserialize)]
struct DsoBody {
    series: Vec<f64>,
    #[serde(default = "default_dso_period")] hp_period: usize,
}
fn default_dso_period() -> usize { 30 }

async fn ehlers_decycler_oscillator_route(
    _u: AuthUser, Json(b): Json<DsoBody>,
) -> Json<Vec<Option<f64>>> {
    Json(ehlers_decycler_oscillator::compute(&b.series, b.hp_period))
}

#[derive(Deserialize)]
struct AbcdBody {
    pivots: Vec<gartley_pattern::Pivot>,
    #[serde(default = "default_abcd_tol")] tolerance: f64,
}
fn default_abcd_tol() -> f64 { 0.05 }

async fn abcd_pattern_route(
    _u: AuthUser, Json(b): Json<AbcdBody>,
) -> Json<Vec<abcd_pattern::AbcdMatch>> {
    Json(abcd_pattern::detect(&b.pivots, b.tolerance))
}

#[derive(Deserialize)]
struct GannFanBody {
    anchor_price: f64,
    bars_from_anchor: usize,
    #[serde(default = "default_gann_fan_unit")] unit_per_bar: f64,
    #[serde(default)] up: bool,
}
fn default_gann_fan_unit() -> f64 { 1.0 }

async fn gann_fan_route(
    _u: AuthUser, Json(b): Json<GannFanBody>,
) -> Json<Option<gann_fan::GannFanLevels>> {
    Json(gann_fan::compute(b.anchor_price, b.bars_from_anchor, b.unit_per_bar, b.up))
}

#[derive(Deserialize)]
struct RatioChartBody { closes_a: Vec<f64>, closes_b: Vec<f64> }

async fn ratio_chart_route(
    _u: AuthUser, Json(b): Json<RatioChartBody>,
) -> Json<ratio_chart::RatioChartReport> {
    Json(ratio_chart::compute(&b.closes_a, &b.closes_b))
}

#[derive(Deserialize)]
struct SpreadChartBody {
    closes_a: Vec<f64>,
    closes_b: Vec<f64>,
    #[serde(default = "default_spread_hedge")] hedge_ratio: f64,
    #[serde(default = "default_spread_period")] zscore_period: usize,
}
fn default_spread_hedge() -> f64 { 1.0 }
fn default_spread_period() -> usize { 20 }

async fn spread_chart_route(
    _u: AuthUser, Json(b): Json<SpreadChartBody>,
) -> Json<spread_chart::SpreadChartReport> {
    Json(spread_chart::compute(&b.closes_a, &b.closes_b, b.hedge_ratio, b.zscore_period))
}

#[derive(Deserialize)]
struct BbwpBody {
    closes: Vec<f64>,
    #[serde(default = "default_bbwp_period")] bb_period: usize,
    #[serde(default = "default_bbwp_stdev")] n_stdev: f64,
    #[serde(default = "default_bbwp_lookback")] lookback: usize,
}
fn default_bbwp_period() -> usize { 20 }
fn default_bbwp_stdev() -> f64 { 2.0 }
fn default_bbwp_lookback() -> usize { 252 }

async fn bollinger_bandwidth_percentile_route(
    _u: AuthUser, Json(b): Json<BbwpBody>,
) -> Json<Vec<Option<f64>>> {
    Json(bollinger_bandwidth_percentile::compute(&b.closes, b.bb_period, b.n_stdev, b.lookback))
}

#[derive(Deserialize)]
struct RsVsMarketBody {
    stock_closes: Vec<f64>,
    benchmark_closes: Vec<f64>,
    #[serde(default = "default_rs_vs_period")] period: usize,
}
fn default_rs_vs_period() -> usize { 63 }

async fn relative_strength_vs_market_route(
    _u: AuthUser, Json(b): Json<RsVsMarketBody>,
) -> Json<relative_strength_vs_market::RsVsMarketReport> {
    Json(relative_strength_vs_market::compute(&b.stock_closes, &b.benchmark_closes, b.period))
}


#[derive(Deserialize)]
struct UltimateSmootherBody {
    series: Vec<f64>,
    #[serde(default = "default_us_period")] period: usize,
}
fn default_us_period() -> usize { 10 }

async fn ultimate_smoother_route(
    _u: AuthUser, Json(b): Json<UltimateSmootherBody>,
) -> Json<Vec<Option<f64>>> {
    Json(ultimate_smoother::compute(&b.series, b.period))
}

#[derive(Deserialize)]
struct CsmBody {
    closes: Vec<f64>,
    #[serde(default = "default_csm_mom")] momentum_period: usize,
    #[serde(default = "default_csm_smooth")] smooth_period: usize,
}
fn default_csm_mom() -> usize { 10 }
fn default_csm_smooth() -> usize { 8 }

async fn ehlers_centered_smoothed_momentum_route(
    _u: AuthUser, Json(b): Json<CsmBody>,
) -> Json<Vec<Option<f64>>> {
    Json(ehlers_centered_smoothed_momentum::compute(&b.closes,
        b.momentum_period, b.smooth_period))
}

#[derive(Deserialize)]
struct FiveOBody {
    pivots: Vec<gartley_pattern::Pivot>,
    #[serde(default = "default_five_o_tol")] tolerance: f64,
}
fn default_five_o_tol() -> f64 { 0.05 }

async fn five_o_pattern_route(
    _u: AuthUser, Json(b): Json<FiveOBody>,
) -> Json<Vec<five_o_pattern::FiveOMatch>> {
    Json(five_o_pattern::detect(&b.pivots, b.tolerance))
}

#[derive(Deserialize)]
struct TpBody { bars: Vec<typical_price::Bar> }

async fn typical_price_route(
    _u: AuthUser, Json(b): Json<TpBody>,
) -> Json<Vec<Option<f64>>> {
    Json(typical_price::compute(&b.bars))
}

#[derive(Deserialize)]
struct WcBody { bars: Vec<weighted_close::Bar> }

async fn weighted_close_route(
    _u: AuthUser, Json(b): Json<WcBody>,
) -> Json<Vec<Option<f64>>> {
    Json(weighted_close::compute(&b.bars))
}

#[derive(Deserialize)]
struct EngulfingScanBody {
    bars: Vec<engulfing_pattern_scanner::Bar>,
    #[serde(default = "default_engulf_trend")] trend_period: usize,
}
fn default_engulf_trend() -> usize { 20 }

async fn engulfing_pattern_scanner_route(
    _u: AuthUser, Json(b): Json<EngulfingScanBody>,
) -> Json<engulfing_pattern_scanner::EngulfingReport> {
    Json(engulfing_pattern_scanner::compute(&b.bars, b.trend_period))
}

#[derive(Deserialize)]
struct FiftyTwoWeekBody {
    bars: Vec<fifty_two_week_high_low_scanner::Bar>,
    #[serde(default = "default_fifty_two_week_lookback")] lookback: usize,
}
fn default_fifty_two_week_lookback() -> usize { 252 }

async fn fifty_two_week_high_low_scanner_route(
    _u: AuthUser, Json(b): Json<FiftyTwoWeekBody>,
) -> Json<fifty_two_week_high_low_scanner::FiftyTwoWeekReport> {
    Json(fifty_two_week_high_low_scanner::compute(&b.bars, b.lookback))
}


#[derive(Deserialize)]
struct SessionVwapBody { bars: Vec<session_vwap::Bar> }

async fn session_vwap_route(
    _u: AuthUser, Json(b): Json<SessionVwapBody>,
) -> Json<session_vwap::SessionVwapReport> {
    Json(session_vwap::compute(&b.bars))
}

#[derive(Deserialize)]
struct TapeSpeedBody {
    bars: Vec<tape_speed::Bar>,
    #[serde(default = "default_tape_speed_period")] period: usize,
}
fn default_tape_speed_period() -> usize { 20 }

async fn tape_speed_route(
    _u: AuthUser, Json(b): Json<TapeSpeedBody>,
) -> Json<tape_speed::TapeSpeedReport> {
    Json(tape_speed::compute(&b.bars, b.period))
}

#[derive(Deserialize)]
struct LiquidityPoolsBody {
    pivots: Vec<gartley_pattern::Pivot>,
    #[serde(default = "default_liq_tol")] tolerance_pct: f64,
    #[serde(default = "default_liq_min")] min_count: usize,
}
fn default_liq_tol() -> f64 { 0.5 }
fn default_liq_min() -> usize { 3 }

async fn liquidity_pool_detector_route(
    _u: AuthUser, Json(b): Json<LiquidityPoolsBody>,
) -> Json<Vec<liquidity_pool_detector::LiquidityPool>> {
    Json(liquidity_pool_detector::detect(&b.pivots, b.tolerance_pct, b.min_count))
}

#[derive(Deserialize)]
struct AbsorptionBody {
    bars: Vec<absorption_detector::Bar>,
    #[serde(default = "default_absorption_period")] period: usize,
    #[serde(default = "default_absorption_threshold")] threshold: f64,
    #[serde(default = "default_absorption_vol")] vol_multiplier: f64,
}
fn default_absorption_period() -> usize { 20 }
fn default_absorption_threshold() -> f64 { 0.5 }
fn default_absorption_vol() -> f64 { 1.5 }

async fn absorption_detector_route(
    _u: AuthUser, Json(b): Json<AbsorptionBody>,
) -> Json<absorption_detector::AbsorptionReport> {
    Json(absorption_detector::compute(&b.bars, b.period, b.threshold, b.vol_multiplier))
}

#[derive(Deserialize)]
struct IcebergBody {
    prints: Vec<iceberg_detector::Print>,
    #[serde(default = "default_iceberg_tol")] price_tolerance_pct: f64,
    #[serde(default = "default_iceberg_window")] max_window_sec: f64,
    #[serde(default = "default_iceberg_fills")] min_fills: usize,
    #[serde(default = "default_iceberg_vol")] vol_threshold: f64,
}
fn default_iceberg_tol() -> f64 { 0.01 }
fn default_iceberg_window() -> f64 { 60.0 }
fn default_iceberg_fills() -> usize { 5 }
fn default_iceberg_vol() -> f64 { 1000.0 }

async fn iceberg_detector_route(
    _u: AuthUser, Json(b): Json<IcebergBody>,
) -> Json<Vec<iceberg_detector::IcebergMatch>> {
    Json(iceberg_detector::detect(&b.prints, b.price_tolerance_pct,
        b.max_window_sec, b.min_fills, b.vol_threshold))
}

#[derive(Deserialize)]
struct MedianPriceBody { bars: Vec<median_price::Bar> }

async fn median_price_route(
    _u: AuthUser, Json(b): Json<MedianPriceBody>,
) -> Json<Vec<Option<f64>>> {
    Json(median_price::compute(&b.bars))
}

#[derive(Deserialize)]
struct ArmsHighLowBody {
    breadth: Vec<arms_high_low_index::DailyBreadth>,
    #[serde(default = "default_ahli_period")] period: usize,
}
fn default_ahli_period() -> usize { 10 }

async fn arms_high_low_index_route(
    _u: AuthUser, Json(b): Json<ArmsHighLowBody>,
) -> Json<arms_high_low_index::ArmsHighLowReport> {
    Json(arms_high_low_index::compute(&b.breadth, b.period))
}


#[derive(Deserialize)]
struct FibExtBody {
    a: f64,
    b: f64,
    retracement_base: f64,
    #[serde(default)] bullish: bool,
}

async fn fibonacci_extensions_route(
    _u: AuthUser, Json(b): Json<FibExtBody>,
) -> Json<Option<fibonacci_extensions::FibonacciExtensionLevels>> {
    Json(fibonacci_extensions::compute(b.a, b.b, b.retracement_base, b.bullish))
}

#[derive(Deserialize)]
struct DonchianOscBody {
    bars: Vec<donchian_oscillator::Bar>,
    #[serde(default = "default_donchian_osc_period")] period: usize,
}
fn default_donchian_osc_period() -> usize { 20 }

async fn donchian_oscillator_route(
    _u: AuthUser, Json(b): Json<DonchianOscBody>,
) -> Json<Vec<Option<f64>>> {
    Json(donchian_oscillator::compute(&b.bars, b.period))
}

#[derive(Deserialize)]
struct MesaSineBody { series: Vec<f64> }

async fn mesa_sine_wave_route(
    _u: AuthUser, Json(b): Json<MesaSineBody>,
) -> Json<mesa_sine_wave::SineWaveReport> {
    Json(mesa_sine_wave::compute(&b.series))
}

#[derive(Deserialize)]
struct CyberneticFisherBody {
    bars: Vec<ehlers_fisher_cybernetic::Bar>,
    #[serde(default = "default_cyber_fisher_period")] period: usize,
}
fn default_cyber_fisher_period() -> usize { 10 }

async fn ehlers_fisher_cybernetic_route(
    _u: AuthUser, Json(b): Json<CyberneticFisherBody>,
) -> Json<ehlers_fisher_cybernetic::CyberneticFisherReport> {
    Json(ehlers_fisher_cybernetic::compute(&b.bars, b.period))
}

#[derive(Deserialize)]
struct BidAskVolBody {
    bars: Vec<bid_ask_volume_ratio::Bar>,
    #[serde(default = "default_bav_period")] period: usize,
}
fn default_bav_period() -> usize { 60 }

async fn bid_ask_volume_ratio_route(
    _u: AuthUser, Json(b): Json<BidAskVolBody>,
) -> Json<Vec<Option<f64>>> {
    Json(bid_ask_volume_ratio::compute(&b.bars, b.period))
}

#[derive(Deserialize)]
struct NyseTickBody {
    tick: Vec<f64>,
    #[serde(default = "default_tick_ema")] ema_period: usize,
    #[serde(default = "default_tick_ext")] ext_threshold: f64,
}
fn default_tick_ema() -> usize { 14 }
fn default_tick_ext() -> f64 { 1000.0 }

async fn nyse_tick_route(
    _u: AuthUser, Json(b): Json<NyseTickBody>,
) -> Json<nyse_tick::NyseTickReport> {
    Json(nyse_tick::compute(&b.tick, b.ema_period, b.ext_threshold))
}

#[derive(Deserialize)]
struct CviBarBody {
    bars: Vec<chande_volatility_index::Bar>,
    #[serde(default = "default_cvi_ema")] ema_period: usize,
    #[serde(default = "default_cvi_roc")] roc_period: usize,
}
fn default_cvi_ema() -> usize { 10 }
fn default_cvi_roc() -> usize { 10 }

async fn chande_volatility_index_route(
    _u: AuthUser, Json(b): Json<CviBarBody>,
) -> Json<Vec<Option<f64>>> {
    Json(chande_volatility_index::compute(&b.bars, b.ema_period, b.roc_period))
}


#[derive(Deserialize)]
struct GannSwingBody {
    bars: Vec<gann_swing_chart::Bar>,
    #[serde(default = "default_gann_swing_bars")] reversal_bars: usize,
}
fn default_gann_swing_bars() -> usize { 2 }

async fn gann_swing_chart_route(
    _u: AuthUser, Json(b): Json<GannSwingBody>,
) -> Json<gann_swing_chart::GannSwingReport> {
    Json(gann_swing_chart::compute(&b.bars, b.reversal_bars))
}

#[derive(Deserialize)]
struct VixBasisBody { vix_spot: Vec<f64>, vx_front: Vec<f64> }

async fn vix_basis_route(
    _u: AuthUser, Json(b): Json<VixBasisBody>,
) -> Json<vix_basis::VixBasisReport> {
    Json(vix_basis::compute(&b.vix_spot, &b.vx_front))
}

#[derive(Deserialize)]
struct VixSkewBody {
    iv_25d_put: Vec<f64>,
    iv_25d_call: Vec<f64>,
    iv_atm: Vec<f64>,
    #[serde(default = "default_vix_skew_threshold")] smirk_threshold: f64,
}
fn default_vix_skew_threshold() -> f64 { 5.0 }

async fn vix_skew_smirk_route(
    _u: AuthUser, Json(b): Json<VixSkewBody>,
) -> Json<vix_skew_smirk::VixSkewReport> {
    Json(vix_skew_smirk::compute(&b.iv_25d_put, &b.iv_25d_call, &b.iv_atm, b.smirk_threshold))
}

#[derive(Deserialize)]
struct GammaPinBody {
    strike_gex: Vec<gamma_pin_zone::StrikeGex>,
    spot: f64,
    #[serde(default = "default_pin_radius")] pin_radius_pct: f64,
}
fn default_pin_radius() -> f64 { 5.0 }

async fn gamma_pin_zone_route(
    _u: AuthUser, Json(b): Json<GammaPinBody>,
) -> Json<Option<gamma_pin_zone::GammaPinReport>> {
    Json(gamma_pin_zone::compute(&b.strike_gex, b.spot, b.pin_radius_pct))
}

#[derive(Deserialize)]
struct BorrowRateBody {
    rates_pct: Vec<f64>,
    #[serde(default = "default_borrow_period")] period: usize,
}
fn default_borrow_period() -> usize { 5 }

async fn borrow_rate_indicator_route(
    _u: AuthUser, Json(b): Json<BorrowRateBody>,
) -> Json<borrow_rate_indicator::BorrowRateReport> {
    Json(borrow_rate_indicator::compute(&b.rates_pct, b.period))
}

#[derive(Deserialize)]
struct EarningsSurpriseBody { report: earnings_surprise_indicator::EarningsReport }

async fn earnings_surprise_indicator_route(
    _u: AuthUser, Json(b): Json<EarningsSurpriseBody>,
) -> Json<Option<earnings_surprise_indicator::EarningsSurpriseReport>> {
    Json(earnings_surprise_indicator::compute(b.report))
}

#[derive(Deserialize)]
struct LiquidityVoidBody {
    bars: Vec<liquidity_void_detector::Bar>,
    #[serde(default = "default_lv_period")] period: usize,
    #[serde(default = "default_lv_range_mult")] range_multiplier: f64,
    #[serde(default = "default_lv_vol_mult")] vol_multiplier: f64,
}
fn default_lv_period() -> usize { 20 }
fn default_lv_range_mult() -> f64 { 2.0 }
fn default_lv_vol_mult() -> f64 { 0.5 }

async fn liquidity_void_detector_route(
    _u: AuthUser, Json(b): Json<LiquidityVoidBody>,
) -> Json<Vec<liquidity_void_detector::LiquidityVoid>> {
    Json(liquidity_void_detector::detect(&b.bars, b.period, b.range_multiplier, b.vol_multiplier))
}


#[derive(Deserialize)]
struct ThreeBarBody { bars: Vec<three_white_soldiers_crows::Bar> }

async fn three_white_soldiers_crows_route(
    _u: AuthUser, Json(b): Json<ThreeBarBody>,
) -> Json<three_white_soldiers_crows::ThreeWhiteSoldiersReport> {
    Json(three_white_soldiers_crows::compute(&b.bars))
}

#[derive(Deserialize)]
struct MorningStarBody { bars: Vec<morning_evening_star::Bar> }

async fn morning_evening_star_route(
    _u: AuthUser, Json(b): Json<MorningStarBody>,
) -> Json<morning_evening_star::StarPatternReport> {
    Json(morning_evening_star::compute(&b.bars))
}

#[derive(Deserialize)]
struct DarkCloudBody { bars: Vec<dark_cloud_piercing::Bar> }

async fn dark_cloud_piercing_route(
    _u: AuthUser, Json(b): Json<DarkCloudBody>,
) -> Json<dark_cloud_piercing::DarkCloudPiercingReport> {
    Json(dark_cloud_piercing::compute(&b.bars))
}

#[derive(Deserialize)]
struct HangingManBody {
    bars: Vec<hanging_man_shooting_star::Bar>,
    #[serde(default = "default_hms_trend")] trend_period: usize,
}
fn default_hms_trend() -> usize { 5 }

async fn hanging_man_shooting_star_route(
    _u: AuthUser, Json(b): Json<HangingManBody>,
) -> Json<hanging_man_shooting_star::HangingShootingReport> {
    Json(hanging_man_shooting_star::compute(&b.bars, b.trend_period))
}

#[derive(Deserialize)]
struct TermSpreadBody { short_yield_pct: Vec<f64>, long_yield_pct: Vec<f64> }

async fn term_spread_route(
    _u: AuthUser, Json(b): Json<TermSpreadBody>,
) -> Json<term_spread::TermSpreadReport> {
    Json(term_spread::compute(&b.short_yield_pct, &b.long_yield_pct))
}

#[derive(Deserialize)]
struct VwMacdBody {
    bars: Vec<volume_weighted_macd::Bar>,
    #[serde(default = "default_vwmacd_fast")] fast_period: usize,
    #[serde(default = "default_vwmacd_slow")] slow_period: usize,
    #[serde(default = "default_vwmacd_signal")] signal_period: usize,
}
fn default_vwmacd_fast() -> usize { 12 }
fn default_vwmacd_slow() -> usize { 26 }
fn default_vwmacd_signal() -> usize { 9 }

async fn volume_weighted_macd_route(
    _u: AuthUser, Json(b): Json<VwMacdBody>,
) -> Json<volume_weighted_macd::VolumeWeightedMacdReport> {
    Json(volume_weighted_macd::compute(&b.bars, b.fast_period, b.slow_period, b.signal_period))
}

#[derive(Deserialize)]
struct CvdZScoreBody {
    cvd: Vec<f64>,
    #[serde(default = "default_cvd_zscore_period")] period: usize,
}
fn default_cvd_zscore_period() -> usize { 60 }

async fn cumulative_volume_delta_zscore_route(
    _u: AuthUser, Json(b): Json<CvdZScoreBody>,
) -> Json<Vec<Option<f64>>> {
    Json(cumulative_volume_delta_zscore::compute(&b.cvd, b.period))
}


#[derive(Deserialize)]
struct DojiVariantsBody {
    bars: Vec<doji_variants::Bar>,
    #[serde(default = "default_doji_pct")] doji_pct: f64,
}
fn default_doji_pct() -> f64 { 0.1 }

async fn doji_variants_route(
    _u: AuthUser, Json(b): Json<DojiVariantsBody>,
) -> Json<doji_variants::DojiVariantsReport> {
    Json(doji_variants::compute(&b.bars, b.doji_pct))
}

#[derive(Deserialize)]
struct HaramiBody { bars: Vec<harami_pattern::Bar> }

async fn harami_pattern_route(
    _u: AuthUser, Json(b): Json<HaramiBody>,
) -> Json<harami_pattern::HaramiReport> {
    Json(harami_pattern::compute(&b.bars))
}

#[derive(Deserialize)]
struct TweezerBody {
    bars: Vec<tweezer_top_bottom::Bar>,
    #[serde(default = "default_tweezer_tol")] tolerance_pct: f64,
}
fn default_tweezer_tol() -> f64 { 0.05 }

async fn tweezer_top_bottom_route(
    _u: AuthUser, Json(b): Json<TweezerBody>,
) -> Json<tweezer_top_bottom::TweezerReport> {
    Json(tweezer_top_bottom::compute(&b.bars, b.tolerance_pct))
}

#[derive(Deserialize)]
struct HikkakeBody {
    bars: Vec<hikkake_pattern::Bar>,
    #[serde(default = "default_hikkake_confirm")] confirm_bars: usize,
}
fn default_hikkake_confirm() -> usize { 3 }

async fn hikkake_pattern_route(
    _u: AuthUser, Json(b): Json<HikkakeBody>,
) -> Json<hikkake_pattern::HikkakeReport> {
    Json(hikkake_pattern::compute(&b.bars, b.confirm_bars))
}

#[derive(Deserialize)]
struct Sperandeo123Body {
    pivots: Vec<gartley_pattern::Pivot>,
    closes: Vec<f64>,
    #[serde(default = "default_sperandeo_lookahead")] max_confirmation_lookahead: usize,
}
fn default_sperandeo_lookahead() -> usize { 20 }

async fn sperandeo_1_2_3_route(
    _u: AuthUser, Json(b): Json<Sperandeo123Body>,
) -> Json<Vec<sperandeo_1_2_3::SperandoMatch>> {
    Json(sperandeo_1_2_3::detect(&b.pivots, &b.closes, b.max_confirmation_lookahead))
}

#[derive(Deserialize)]
struct BreadthThrustBody {
    breadth: Vec<breadth_thrust::DailyBreadth>,
    #[serde(default = "default_thrust_ema")] ema_period: usize,
    #[serde(default = "default_thrust_window")] max_window_bars: usize,
    #[serde(default = "default_thrust_low")] low_threshold: f64,
    #[serde(default = "default_thrust_high")] high_threshold: f64,
}
fn default_thrust_ema() -> usize { 10 }
fn default_thrust_window() -> usize { 10 }
fn default_thrust_low() -> f64 { 0.40 }
fn default_thrust_high() -> f64 { 0.615 }

async fn breadth_thrust_route(
    _u: AuthUser, Json(b): Json<BreadthThrustBody>,
) -> Json<breadth_thrust::BreadthThrustReport> {
    Json(breadth_thrust::compute(&b.breadth, b.ema_period, b.max_window_bars,
        b.low_threshold, b.high_threshold))
}

#[derive(Deserialize)]
struct AbandonedBabyBody {
    bars: Vec<abandoned_baby::Bar>,
    #[serde(default = "default_ab_doji_pct")] doji_pct: f64,
}
fn default_ab_doji_pct() -> f64 { 0.1 }

async fn abandoned_baby_route(
    _u: AuthUser, Json(b): Json<AbandonedBabyBody>,
) -> Json<abandoned_baby::AbandonedBabyReport> {
    Json(abandoned_baby::compute(&b.bars, b.doji_pct))
}

#[derive(Deserialize)]
struct SpinningMarubozuBody { bars: Vec<spinning_top_marubozu::Bar> }

async fn spinning_top_marubozu_route(
    _u: AuthUser, Json(b): Json<SpinningMarubozuBody>,
) -> Json<spinning_top_marubozu::SpinningTopMarubozuReport> {
    Json(spinning_top_marubozu::compute(&b.bars))
}


#[derive(Deserialize)]
struct KickerBody { bars: Vec<kicker_pattern::Bar> }

async fn kicker_pattern_route(
    _u: AuthUser, Json(b): Json<KickerBody>,
) -> Json<kicker_pattern::KickerReport> {
    Json(kicker_pattern::compute(&b.bars))
}

#[derive(Deserialize)]
struct ThreeInsideBody { bars: Vec<three_inside_up_down::Bar> }

async fn three_inside_up_down_route(
    _u: AuthUser, Json(b): Json<ThreeInsideBody>,
) -> Json<three_inside_up_down::ThreeInsideReport> {
    Json(three_inside_up_down::compute(&b.bars))
}

#[derive(Deserialize)]
struct ThreeOutsideBody { bars: Vec<three_outside_up_down::Bar> }

async fn three_outside_up_down_route(
    _u: AuthUser, Json(b): Json<ThreeOutsideBody>,
) -> Json<three_outside_up_down::ThreeOutsideReport> {
    Json(three_outside_up_down::compute(&b.bars))
}

#[derive(Deserialize)]
struct BeltHoldBody {
    bars: Vec<belt_hold_pattern::Bar>,
    #[serde(default = "default_belt_hold_trend")] trend_period: usize,
    #[serde(default = "default_belt_hold_wick")] wick_pct: f64,
}
fn default_belt_hold_trend() -> usize { 5 }
fn default_belt_hold_wick() -> f64 { 0.05 }

async fn belt_hold_pattern_route(
    _u: AuthUser, Json(b): Json<BeltHoldBody>,
) -> Json<belt_hold_pattern::BeltHoldReport> {
    Json(belt_hold_pattern::compute(&b.bars, b.trend_period, b.wick_pct))
}

#[derive(Deserialize)]
struct GapClassifierBody {
    bars: Vec<gap_classifier::Bar>,
    #[serde(default = "default_gap_lookback")] lookback: usize,
    #[serde(default = "default_gap_fill_bars")] fill_bars: usize,
}
fn default_gap_lookback() -> usize { 20 }
fn default_gap_fill_bars() -> usize { 5 }

async fn gap_classifier_route(
    _u: AuthUser, Json(b): Json<GapClassifierBody>,
) -> Json<gap_classifier::GapClassifierReport> {
    Json(gap_classifier::compute(&b.bars, b.lookback, b.fill_bars))
}

#[derive(Deserialize)]
struct LrR2Body {
    closes: Vec<f64>,
    #[serde(default = "default_lr_r2_period")] period: usize,
}
fn default_lr_r2_period() -> usize { 14 }

async fn linear_regression_r_squared_route(
    _u: AuthUser, Json(b): Json<LrR2Body>,
) -> Json<Vec<Option<f64>>> {
    Json(linear_regression_r_squared::compute(&b.closes, b.period))
}

#[derive(Deserialize)]
struct PvoBody {
    bars: Vec<price_volume_oscillator::Bar>,
    #[serde(default = "default_pvo_fast")] fast_period: usize,
    #[serde(default = "default_pvo_slow")] slow_period: usize,
    #[serde(default = "default_pvo_signal")] signal_period: usize,
}
fn default_pvo_fast() -> usize { 12 }
fn default_pvo_slow() -> usize { 26 }
fn default_pvo_signal() -> usize { 9 }

async fn price_volume_oscillator_route(
    _u: AuthUser, Json(b): Json<PvoBody>,
) -> Json<price_volume_oscillator::PvoReport> {
    Json(price_volume_oscillator::compute(&b.bars, b.fast_period, b.slow_period, b.signal_period))
}


#[derive(Deserialize)]
struct ThreeMethodsBody { bars: Vec<rising_falling_three_methods::Bar> }

async fn rising_falling_three_methods_route(
    _u: AuthUser, Json(b): Json<ThreeMethodsBody>,
) -> Json<rising_falling_three_methods::ThreeMethodsReport> {
    Json(rising_falling_three_methods::compute(&b.bars))
}

#[derive(Deserialize)]
struct MatHoldBody { bars: Vec<mat_hold_pattern::Bar> }

async fn mat_hold_pattern_route(
    _u: AuthUser, Json(b): Json<MatHoldBody>,
) -> Json<mat_hold_pattern::MatHoldReport> {
    Json(mat_hold_pattern::compute(&b.bars))
}

#[derive(Deserialize)]
struct OnNeckBody {
    bars: Vec<on_neck_in_neck::Bar>,
    #[serde(default = "default_on_neck_tol")] tolerance_pct: f64,
}
fn default_on_neck_tol() -> f64 { 0.5 }

async fn on_neck_in_neck_route(
    _u: AuthUser, Json(b): Json<OnNeckBody>,
) -> Json<on_neck_in_neck::NeckPatternReport> {
    Json(on_neck_in_neck::compute(&b.bars, b.tolerance_pct))
}

#[derive(Deserialize)]
struct TasukiBody { bars: Vec<tasuki_gap::Bar> }

async fn tasuki_gap_route(
    _u: AuthUser, Json(b): Json<TasukiBody>,
) -> Json<tasuki_gap::TasukiGapReport> {
    Json(tasuki_gap::compute(&b.bars))
}

#[derive(Deserialize)]
struct BreakawayBody { bars: Vec<breakaway_pattern::Bar> }

async fn breakaway_pattern_route(
    _u: AuthUser, Json(b): Json<BreakawayBody>,
) -> Json<breakaway_pattern::BreakawayReport> {
    Json(breakaway_pattern::compute(&b.bars))
}

#[derive(Deserialize)]
struct VolumeClimaxBody {
    bars: Vec<volume_climax_pattern::Bar>,
    #[serde(default = "default_vclimax_period")] period: usize,
    #[serde(default = "default_vclimax_lookback")] lookback: usize,
    #[serde(default = "default_vclimax_vol")] vol_multiplier: f64,
    #[serde(default = "default_vclimax_range")] range_multiplier: f64,
}
fn default_vclimax_period() -> usize { 20 }
fn default_vclimax_lookback() -> usize { 20 }
fn default_vclimax_vol() -> f64 { 2.0 }
fn default_vclimax_range() -> f64 { 1.5 }

async fn volume_climax_pattern_route(
    _u: AuthUser, Json(b): Json<VolumeClimaxBody>,
) -> Json<volume_climax_pattern::VolumeClimaxReport> {
    Json(volume_climax_pattern::compute(&b.bars, b.period, b.lookback,
        b.vol_multiplier, b.range_multiplier))
}

#[derive(Deserialize)]
struct LadderBody { bars: Vec<ladder_bottom_top::Bar> }

async fn ladder_bottom_top_route(
    _u: AuthUser, Json(b): Json<LadderBody>,
) -> Json<ladder_bottom_top::LadderReport> {
    Json(ladder_bottom_top::compute(&b.bars))
}

#[derive(Deserialize)]
struct CounterAttackBody {
    bars: Vec<counter_attack_lines::Bar>,
    #[serde(default = "default_counter_attack_tol")] tolerance_pct: f64,
}
fn default_counter_attack_tol() -> f64 { 0.3 }

async fn counter_attack_lines_route(
    _u: AuthUser, Json(b): Json<CounterAttackBody>,
) -> Json<counter_attack_lines::CounterAttackReport> {
    Json(counter_attack_lines::compute(&b.bars, b.tolerance_pct))
}


#[derive(Deserialize)]
struct ThrustingBody { bars: Vec<thrusting_pattern::Bar> }

async fn thrusting_pattern_route(
    _u: AuthUser, Json(b): Json<ThrustingBody>,
) -> Json<thrusting_pattern::ThrustingReport> {
    Json(thrusting_pattern::compute(&b.bars))
}

#[derive(Deserialize)]
struct StickSandwichBody {
    bars: Vec<stick_sandwich::Bar>,
    #[serde(default = "default_stick_tol")] tolerance_pct: f64,
}
fn default_stick_tol() -> f64 { 0.3 }

async fn stick_sandwich_route(
    _u: AuthUser, Json(b): Json<StickSandwichBody>,
) -> Json<stick_sandwich::StickSandwichReport> {
    Json(stick_sandwich::compute(&b.bars, b.tolerance_pct))
}

#[derive(Deserialize)]
struct SideBySideBody {
    bars: Vec<side_by_side_white_lines::Bar>,
    #[serde(default = "default_side_by_side_tol")] tolerance_pct: f64,
}
fn default_side_by_side_tol() -> f64 { 0.3 }

async fn side_by_side_white_lines_route(
    _u: AuthUser, Json(b): Json<SideBySideBody>,
) -> Json<side_by_side_white_lines::SideBySideReport> {
    Json(side_by_side_white_lines::compute(&b.bars, b.tolerance_pct))
}

#[derive(Deserialize)]
struct UpsideGap2CrowsBody { bars: Vec<upside_gap_two_crows::Bar> }

async fn upside_gap_two_crows_route(
    _u: AuthUser, Json(b): Json<UpsideGap2CrowsBody>,
) -> Json<Vec<bool>> {
    Json(upside_gap_two_crows::compute(&b.bars))
}

#[derive(Deserialize)]
struct SfpBody {
    bars: Vec<swing_failure_pattern::Bar>,
    #[serde(default = "default_sfp_lookback")] lookback: usize,
}
fn default_sfp_lookback() -> usize { 20 }

async fn swing_failure_pattern_route(
    _u: AuthUser, Json(b): Json<SfpBody>,
) -> Json<swing_failure_pattern::SfpReport> {
    Json(swing_failure_pattern::compute(&b.bars, b.lookback))
}

#[derive(Deserialize)]
struct RossHookBody {
    bars: Vec<ross_hook::Bar>,
    #[serde(default = "default_ross_lookback")] trend_lookback: usize,
}
fn default_ross_lookback() -> usize { 5 }

async fn ross_hook_route(
    _u: AuthUser, Json(b): Json<RossHookBody>,
) -> Json<ross_hook::RossHookReport> {
    Json(ross_hook::compute(&b.bars, b.trend_lookback))
}

#[derive(Deserialize)]
struct SeparatingLinesBody {
    bars: Vec<separating_lines::Bar>,
    #[serde(default = "default_separating_trend")] trend_period: usize,
    #[serde(default = "default_separating_tol")] tolerance_pct: f64,
}
fn default_separating_trend() -> usize { 5 }
fn default_separating_tol() -> f64 { 0.3 }

async fn separating_lines_route(
    _u: AuthUser, Json(b): Json<SeparatingLinesBody>,
) -> Json<separating_lines::SeparatingLinesReport> {
    Json(separating_lines::compute(&b.bars, b.trend_period, b.tolerance_pct))
}


#[derive(Deserialize)]
struct KagiBody {
    closes: Vec<f64>,
    reversal: f64,
    #[serde(default = "default_kagi_kind")] kind: kagi_chart::AmountKind,
}
fn default_kagi_kind() -> kagi_chart::AmountKind { kagi_chart::AmountKind::Absolute }

async fn kagi_chart_route(
    _u: AuthUser, Json(b): Json<KagiBody>,
) -> Json<Vec<kagi_chart::KagiLine>> {
    Json(kagi_chart::compute(&b.closes, b.reversal, b.kind))
}

#[derive(Deserialize)]
struct TlbBody {
    closes: Vec<f64>,
    #[serde(default = "default_tlb_lines")] num_lines: usize,
}
fn default_tlb_lines() -> usize { 3 }

async fn three_line_break_route(
    _u: AuthUser, Json(b): Json<TlbBody>,
) -> Json<Vec<three_line_break::TlbLine>> {
    Json(three_line_break::compute(&b.closes, b.num_lines))
}

#[derive(Deserialize)]
struct MeetingLinesBody {
    bars: Vec<meeting_lines::Bar>,
    #[serde(default = "default_meeting_tol")] tolerance_pct: f64,
}
fn default_meeting_tol() -> f64 { 0.3 }

async fn meeting_lines_route(
    _u: AuthUser, Json(b): Json<MeetingLinesBody>,
) -> Json<meeting_lines::MeetingLinesReport> {
    Json(meeting_lines::compute(&b.bars, b.tolerance_pct))
}

#[derive(Deserialize)]
struct VolumeAtPriceBody {
    bars: Vec<volume_at_price::Bar>,
    #[serde(default = "default_vap_bins")] num_bins: usize,
    #[serde(default = "default_vap_va")] value_area_pct: f64,
}
fn default_vap_bins() -> usize { 50 }
fn default_vap_va() -> f64 { 70.0 }

async fn volume_at_price_route(
    _u: AuthUser, Json(b): Json<VolumeAtPriceBody>,
) -> Json<volume_at_price::VolumeAtPriceReport> {
    Json(volume_at_price::compute(&b.bars, b.num_bins, b.value_area_pct))
}

#[derive(Deserialize)]
struct EquivolumeBody {
    bars: Vec<equivolume_bars::Bar>,
    #[serde(default = "default_equivolume_width")] total_width: f64,
}
fn default_equivolume_width() -> f64 { 1000.0 }

async fn equivolume_bars_route(
    _u: AuthUser, Json(b): Json<EquivolumeBody>,
) -> Json<equivolume_bars::EquivolumeReport> {
    Json(equivolume_bars::compute(&b.bars, b.total_width))
}

#[derive(Deserialize)]
struct EhrlichBody {
    bars: Vec<ehrlich_filter::Bar>,
    #[serde(default = "default_ehrlich_period")] period: usize,
}
fn default_ehrlich_period() -> usize { 14 }

async fn ehrlich_filter_route(
    _u: AuthUser, Json(b): Json<EhrlichBody>,
) -> Json<Vec<Option<f64>>> {
    Json(ehrlich_filter::compute(&b.bars, b.period))
}

#[derive(Deserialize)]
struct BelkhayateBody {
    closes: Vec<f64>,
    #[serde(default = "default_belkhayate_period")] period: usize,
}
fn default_belkhayate_period() -> usize { 30 }

async fn belkhayate_timing_route(
    _u: AuthUser, Json(b): Json<BelkhayateBody>,
) -> Json<Vec<Option<f64>>> {
    Json(belkhayate_timing::compute(&b.closes, b.period))
}


#[derive(Deserialize)]
struct RangeBarBody { prints: Vec<range_bar_chart::Print>, target_range: f64 }

async fn range_bar_chart_route(
    _u: AuthUser, Json(b): Json<RangeBarBody>,
) -> Json<Vec<range_bar_chart::RangeBar>> {
    Json(range_bar_chart::compute(&b.prints, b.target_range))
}

#[derive(Deserialize)]
struct TickBarBody { prints: Vec<tick_bar_chart::Print>, ticks_per_bar: u32 }

async fn tick_bar_chart_route(
    _u: AuthUser, Json(b): Json<TickBarBody>,
) -> Json<Vec<tick_bar_chart::TickBar>> {
    Json(tick_bar_chart::compute(&b.prints, b.ticks_per_bar))
}

#[derive(Deserialize)]
struct VolumeBarBody { prints: Vec<volume_bar_chart::Print>, volume_per_bar: f64 }

async fn volume_bar_chart_route(
    _u: AuthUser, Json(b): Json<VolumeBarBody>,
) -> Json<Vec<volume_bar_chart::VolumeBar>> {
    Json(volume_bar_chart::compute(&b.prints, b.volume_per_bar))
}

#[derive(Deserialize)]
struct DollarBarBody { prints: Vec<dollar_bar_chart::Print>, dollars_per_bar: f64 }

async fn dollar_bar_chart_route(
    _u: AuthUser, Json(b): Json<DollarBarBody>,
) -> Json<Vec<dollar_bar_chart::DollarBar>> {
    Json(dollar_bar_chart::compute(&b.prints, b.dollars_per_bar))
}

#[derive(Deserialize)]
struct VolSmileBody {
    strike_iv: Vec<volatility_smile::StrikeIv>,
    spot: f64,
    put_25d_strike: f64,
    call_25d_strike: f64,
}

async fn volatility_smile_route(
    _u: AuthUser, Json(b): Json<VolSmileBody>,
) -> Json<Option<volatility_smile::VolatilitySmileReport>> {
    Json(volatility_smile::compute(&b.strike_iv, b.spot,
        b.put_25d_strike, b.call_25d_strike))
}

#[derive(Deserialize)]
struct MonthlySeasonalityBody { closes: Vec<monthly_seasonality::DailyClose> }

async fn monthly_seasonality_route(
    _u: AuthUser, Json(b): Json<MonthlySeasonalityBody>,
) -> Json<Option<monthly_seasonality::MonthlySeasonalityReport>> {
    Json(monthly_seasonality::compute(&b.closes))
}

#[derive(Deserialize)]
struct IntradaySeasonalityBody { bars: Vec<intraday_seasonality::IntradayBar> }

async fn intraday_seasonality_route(
    _u: AuthUser, Json(b): Json<IntradaySeasonalityBody>,
) -> Json<Option<intraday_seasonality::IntradaySeasonalityReport>> {
    Json(intraday_seasonality::compute(&b.bars))
}


#[derive(Deserialize)]
struct HolidaySeasonalityBody {
    days: Vec<holiday_seasonality::TradingDay>,
    holiday_indices: Vec<u32>,
    #[serde(default = "default_holiday_window")] window_before: u32,
    #[serde(default = "default_holiday_window")] window_after: u32,
}
fn default_holiday_window() -> u32 { 5 }

async fn holiday_seasonality_route(
    _u: AuthUser, Json(b): Json<HolidaySeasonalityBody>,
) -> Json<Option<holiday_seasonality::HolidaySeasonalityReport>> {
    Json(holiday_seasonality::compute(&b.days, &b.holiday_indices,
        b.window_before, b.window_after))
}

#[derive(Deserialize)]
struct DowSeasonalityBody { closes: Vec<day_of_week_seasonality::DailyClose> }

async fn day_of_week_seasonality_route(
    _u: AuthUser, Json(b): Json<DowSeasonalityBody>,
) -> Json<Option<day_of_week_seasonality::DayOfWeekSeasonalityReport>> {
    Json(day_of_week_seasonality::compute(&b.closes))
}

#[derive(Deserialize)]
struct DevelopingValueAreaBody {
    bars: Vec<developing_value_area::Bar>,
    #[serde(default = "default_dev_va_bins")] num_bins: usize,
    #[serde(default = "default_dev_va_va")] value_area_pct: f64,
}
fn default_dev_va_bins() -> usize { 50 }
fn default_dev_va_va() -> f64 { 70.0 }

async fn developing_value_area_route(
    _u: AuthUser, Json(b): Json<DevelopingValueAreaBody>,
) -> Json<developing_value_area::DevelopingValueAreaReport> {
    Json(developing_value_area::compute(&b.bars, b.num_bins, b.value_area_pct))
}

#[derive(Deserialize)]
struct DevelopingPocBody {
    bars: Vec<developing_poc::Bar>,
    #[serde(default = "default_dev_poc_bins")] num_bins: usize,
}
fn default_dev_poc_bins() -> usize { 50 }

async fn developing_poc_route(
    _u: AuthUser, Json(b): Json<DevelopingPocBody>,
) -> Json<developing_poc::DevelopingPocReport> {
    Json(developing_poc::compute(&b.bars, b.num_bins))
}

#[derive(Deserialize)]
struct OiDistBody { strikes: Vec<option_open_interest_distribution::StrikeOi> }

async fn option_open_interest_distribution_route(
    _u: AuthUser, Json(b): Json<OiDistBody>,
) -> Json<Option<option_open_interest_distribution::OptionOiReport>> {
    Json(option_open_interest_distribution::compute(&b.strikes))
}

#[derive(Deserialize)]
struct MovingCorrelationBody {
    series_x: Vec<f64>,
    series_y: Vec<f64>,
    #[serde(default = "default_corr_period")] period: usize,
}
fn default_corr_period() -> usize { 30 }

async fn moving_correlation_route(
    _u: AuthUser, Json(b): Json<MovingCorrelationBody>,
) -> Json<Vec<Option<f64>>> {
    Json(moving_correlation::compute(&b.series_x, &b.series_y, b.period))
}

#[derive(Deserialize)]
struct ImbalanceBarBody {
    prints: Vec<imbalance_bar_chart::Print>,
    imbalance_threshold: f64,
}

async fn imbalance_bar_chart_route(
    _u: AuthUser, Json(b): Json<ImbalanceBarBody>,
) -> Json<Vec<imbalance_bar_chart::ImbalanceBar>> {
    Json(imbalance_bar_chart::compute(&b.prints, b.imbalance_threshold))
}


#[derive(Deserialize)]
struct PinBarBody { bars: Vec<pin_bar::Bar> }

async fn pin_bar_route(
    _u: AuthUser, Json(b): Json<PinBarBody>,
) -> Json<pin_bar::PinBarReport> {
    Json(pin_bar::compute(&b.bars))
}

#[derive(Deserialize)]
struct AtrTrailingStopBody {
    bars: Vec<atr_trailing_stop::Bar>,
    #[serde(default = "default_atrts_period")] period: usize,
    #[serde(default = "default_atrts_mult")] multiplier: f64,
}
fn default_atrts_period() -> usize { 14 }
fn default_atrts_mult() -> f64 { 3.0 }

async fn atr_trailing_stop_route(
    _u: AuthUser, Json(b): Json<AtrTrailingStopBody>,
) -> Json<atr_trailing_stop::AtrTrailingStopReport> {
    Json(atr_trailing_stop::compute(&b.bars, b.period, b.multiplier))
}

#[derive(Deserialize)]
struct AtrChannelBody {
    bars: Vec<atr_channel::Bar>,
    #[serde(default = "default_atrc_period")] period: usize,
    #[serde(default = "default_atrc_mult")] multiplier: f64,
    #[serde(default = "default_atrc_use_ema")] use_ema: bool,
}
fn default_atrc_period() -> usize { 20 }
fn default_atrc_mult() -> f64 { 2.0 }
fn default_atrc_use_ema() -> bool { true }

async fn atr_channel_route(
    _u: AuthUser, Json(b): Json<AtrChannelBody>,
) -> Json<atr_channel::AtrChannelReport> {
    Json(atr_channel::compute(&b.bars, b.period, b.multiplier, b.use_ema))
}

#[derive(Deserialize)]
struct RvolZBody {
    volumes: Vec<f64>,
    #[serde(default = "default_rvolz_p")] rvol_period: usize,
    #[serde(default = "default_rvolz_z")] zscore_period: usize,
}
fn default_rvolz_p() -> usize { 20 }
fn default_rvolz_z() -> usize { 60 }

async fn relative_volume_zscore_route(
    _u: AuthUser, Json(b): Json<RvolZBody>,
) -> Json<Vec<Option<f64>>> {
    Json(relative_volume_zscore::compute(&b.volumes, b.rvol_period, b.zscore_period))
}

#[derive(Deserialize)]
struct PowerBarBody {
    bars: Vec<power_bar::Bar>,
    #[serde(default = "default_powerbar_body")] body_threshold: f64,
    #[serde(default = "default_powerbar_close")] close_at_extreme_threshold: f64,
}
fn default_powerbar_body() -> f64 { 0.7 }
fn default_powerbar_close() -> f64 { 0.85 }

async fn power_bar_route(
    _u: AuthUser, Json(b): Json<PowerBarBody>,
) -> Json<power_bar::PowerBarReport> {
    Json(power_bar::compute(&b.bars, b.body_threshold, b.close_at_extreme_threshold))
}

#[derive(Deserialize)]
struct BbDistBody {
    closes: Vec<f64>,
    #[serde(default = "default_bbdist_period")] period: usize,
    #[serde(default = "default_bbdist_stdev")] n_stdev: f64,
}
fn default_bbdist_period() -> usize { 20 }
fn default_bbdist_stdev() -> f64 { 2.0 }

async fn bollinger_band_distance_route(
    _u: AuthUser, Json(b): Json<BbDistBody>,
) -> Json<Vec<Option<f64>>> {
    Json(bollinger_band_distance::compute(&b.closes, b.period, b.n_stdev))
}

#[derive(Deserialize)]
struct VfiBarsBody {
    bars: Vec<volume_force_index::Bar>,
    #[serde(default = "default_vfi_short")] short_period: usize,
    #[serde(default = "default_vfi_long")] long_period: usize,
}
fn default_vfi_short() -> usize { 2 }
fn default_vfi_long() -> usize { 13 }

async fn volume_force_index_route(
    _u: AuthUser, Json(b): Json<VfiBarsBody>,
) -> Json<volume_force_index::VolumeForceIndexReport> {
    Json(volume_force_index::compute(&b.bars, b.short_period, b.long_period))
}


#[derive(Deserialize)]
struct ZeroLagMacdBody {
    closes: Vec<f64>,
    #[serde(default = "default_zlmacd_fast")] fast_period: usize,
    #[serde(default = "default_zlmacd_slow")] slow_period: usize,
    #[serde(default = "default_zlmacd_signal")] signal_period: usize,
}
fn default_zlmacd_fast() -> usize { 12 }
fn default_zlmacd_slow() -> usize { 26 }
fn default_zlmacd_signal() -> usize { 9 }

async fn zero_lag_macd_route(
    _u: AuthUser, Json(b): Json<ZeroLagMacdBody>,
) -> Json<zero_lag_macd::ZeroLagMacdReport> {
    Json(zero_lag_macd::compute(&b.closes, b.fast_period, b.slow_period, b.signal_period))
}

#[derive(Deserialize)]
struct CompoundPivotsBody {
    daily: Option<compound_pivots::PriorSession>,
    weekly: Option<compound_pivots::PriorSession>,
    monthly: Option<compound_pivots::PriorSession>,
}

async fn compound_pivots_route(
    _u: AuthUser, Json(b): Json<CompoundPivotsBody>,
) -> Json<compound_pivots::CompoundPivotsReport> {
    Json(compound_pivots::compute(b.daily, b.weekly, b.monthly))
}

#[derive(Deserialize)]
struct ErgodicBody {
    closes: Vec<f64>,
    #[serde(default = "default_ergodic_r")] r_period: usize,
    #[serde(default = "default_ergodic_s")] s_period: usize,
    #[serde(default = "default_ergodic_sig")] signal_period: usize,
}
fn default_ergodic_r() -> usize { 25 }
fn default_ergodic_s() -> usize { 13 }
fn default_ergodic_sig() -> usize { 7 }

async fn ergodic_oscillator_route(
    _u: AuthUser, Json(b): Json<ErgodicBody>,
) -> Json<ergodic_oscillator::ErgodicReport> {
    Json(ergodic_oscillator::compute(&b.closes, b.r_period, b.s_period, b.signal_period))
}

#[derive(Deserialize)]
struct MomentumQualityBody {
    bars: Vec<momentum_quality_score::Bar>,
    #[serde(default = "default_mq_period")] period: usize,
    #[serde(default = "default_mq_z")] z_period: usize,
}
fn default_mq_period() -> usize { 14 }
fn default_mq_z() -> usize { 60 }

async fn momentum_quality_score_route(
    _u: AuthUser, Json(b): Json<MomentumQualityBody>,
) -> Json<Vec<Option<f64>>> {
    Json(momentum_quality_score::compute(&b.bars, b.period, b.z_period))
}

#[derive(Deserialize)]
struct SwingStrengthBody {
    pivots: Vec<gartley_pattern::Pivot>,
    #[serde(default = "default_ss_z")] z_period: usize,
}
fn default_ss_z() -> usize { 10 }

async fn swing_strength_index_route(
    _u: AuthUser, Json(b): Json<SwingStrengthBody>,
) -> Json<swing_strength_index::SwingStrengthReport> {
    Json(swing_strength_index::compute(&b.pivots, b.z_period))
}

#[derive(Deserialize)]
struct MomentumPersistenceBody {
    closes: Vec<f64>,
    #[serde(default = "default_mp_period")] period: usize,
}
fn default_mp_period() -> usize { 20 }

async fn momentum_persistence_index_route(
    _u: AuthUser, Json(b): Json<MomentumPersistenceBody>,
) -> Json<momentum_persistence_index::MomentumPersistenceReport> {
    Json(momentum_persistence_index::compute(&b.closes, b.period))
}

#[derive(Deserialize)]
struct CumTickZBody {
    tick: Vec<f64>,
    #[serde(default = "default_ctz_period")] period: usize,
}
fn default_ctz_period() -> usize { 60 }

async fn cumulative_tick_zscore_route(
    _u: AuthUser, Json(b): Json<CumTickZBody>,
) -> Json<Vec<Option<f64>>> {
    Json(cumulative_tick_zscore::compute(&b.tick, b.period))
}


#[derive(Deserialize)]
struct RegimeClassifierBody {
    closes: Vec<f64>,
    #[serde(default = "default_regime_period")] period: usize,
}
fn default_regime_period() -> usize { 20 }

async fn regime_classifier_route(
    _u: AuthUser, Json(b): Json<RegimeClassifierBody>,
) -> Json<regime_classifier::RegimeClassifierReport> {
    Json(regime_classifier::compute(&b.closes, b.period))
}

#[derive(Deserialize)]
struct BocpdBody {
    returns: Vec<f64>,
    #[serde(default = "default_bocpd_hazard")] hazard: f64,
}
fn default_bocpd_hazard() -> f64 { 0.01 }

async fn bayesian_change_point_detector_route(
    _u: AuthUser, Json(b): Json<BocpdBody>,
) -> Json<bayesian_change_point_detector::BocpdReport> {
    Json(bayesian_change_point_detector::compute(&b.returns, b.hazard))
}

#[derive(Deserialize)]
struct DividendYieldBody { observations: Vec<dividend_yield_curve::DividendObservation> }

async fn dividend_yield_curve_route(
    _u: AuthUser, Json(b): Json<DividendYieldBody>,
) -> Json<Option<dividend_yield_curve::DividendYieldCurveReport>> {
    Json(dividend_yield_curve::compute(&b.observations))
}

#[derive(Deserialize)]
struct RepoSpreadBody { repo_rate: Vec<f64>, target_rate: Vec<f64> }

async fn repo_rate_spread_route(
    _u: AuthUser, Json(b): Json<RepoSpreadBody>,
) -> Json<repo_rate_spread::RepoRateSpreadReport> {
    Json(repo_rate_spread::compute(&b.repo_rate, &b.target_rate))
}

#[derive(Deserialize)]
struct TermPremiumBody {
    long_yield_pct: Vec<f64>,
    short_yield_pct: Vec<f64>,
    #[serde(default = "default_term_premium_lookback")] lookback: usize,
}
fn default_term_premium_lookback() -> usize { 60 }

async fn term_premium_estimator_route(
    _u: AuthUser, Json(b): Json<TermPremiumBody>,
) -> Json<Vec<Option<f64>>> {
    Json(term_premium_estimator::compute(&b.long_yield_pct, &b.short_yield_pct, b.lookback))
}

#[derive(Deserialize)]
struct SwapCurveBody { points: Vec<swap_rate_curve::SwapPoint> }

async fn swap_rate_curve_route(
    _u: AuthUser, Json(b): Json<SwapCurveBody>,
) -> Json<Option<swap_rate_curve::SwapRateCurveReport>> {
    Json(swap_rate_curve::compute(&b.points))
}

#[derive(Deserialize)]
struct RollingQuantileBody {
    series: Vec<f64>,
    period: usize,
    q: f64,
}

async fn rolling_quantile_route(
    _u: AuthUser, Json(b): Json<RollingQuantileBody>,
) -> Json<Vec<Option<f64>>> {
    Json(rolling_quantile::compute(&b.series, b.period, b.q))
}

#[derive(Deserialize)]
struct OptionPayoffDiagramBody {
    legs: Vec<OptionLeg>,
    spot_min: f64,
    spot_max: f64,
    steps: usize,
}

#[derive(Deserialize)]
struct OptionLeg {
    kind: String,
    strike: f64,
    premium: f64,
    qty: f64,
}

async fn option_payoff_diagram_route(
    _u: AuthUser, Json(b): Json<OptionPayoffDiagramBody>,
) -> Json<Option<OptionPayoffDiagramResponse>> {
    let legs: Vec<option_payoff_diagram::Leg> = b.legs.iter().map(|l| {
        let kind = match l.kind.to_lowercase().as_str() {
            "call" => option_payoff_diagram::LegKind::Call,
            "put" => option_payoff_diagram::LegKind::Put,
            _ => option_payoff_diagram::LegKind::Underlying,
        };
        option_payoff_diagram::Leg { kind, strike: l.strike, premium: l.premium, qty: l.qty }
    }).collect();
    Json(option_payoff_diagram::compute(&legs, b.spot_min, b.spot_max, b.steps)
        .map(|r| OptionPayoffDiagramResponse {
            spots: r.points.iter().map(|p| p.spot).collect(),
            pnls: r.points.iter().map(|p| p.pnl).collect(),
            max_profit: r.max_profit,
            max_loss: r.max_loss,
            breakevens: r.breakevens,
        }))
}

#[derive(serde::Serialize)]
struct OptionPayoffDiagramResponse {
    spots: Vec<f64>,
    pnls: Vec<f64>,
    max_profit: f64,
    max_loss: f64,
    breakevens: Vec<f64>,
}

#[derive(Deserialize)]
struct ValueAtRiskHistoricalBody {
    returns: Vec<f64>,
    #[serde(default = "default_hs_var_confidence")]
    confidence: f64,
}

fn default_hs_var_confidence() -> f64 { 0.95 }

async fn value_at_risk_historical_route(
    _u: AuthUser, Json(b): Json<ValueAtRiskHistoricalBody>,
) -> Json<Option<ValueAtRiskHistoricalResponse>> {
    Json(value_at_risk_historical::compute(&b.returns, b.confidence)
        .map(|r| ValueAtRiskHistoricalResponse {
            var: r.var,
            expected_shortfall: r.expected_shortfall,
            n: r.n,
        }))
}

#[derive(serde::Serialize)]
struct ValueAtRiskHistoricalResponse {
    var: f64,
    expected_shortfall: f64,
    n: usize,
}

#[derive(Deserialize)]
struct ValueAtRiskFilteredHistoricalBody {
    returns: Vec<f64>,
    #[serde(default = "default_fhs_var_confidence")]
    confidence: f64,
    #[serde(default = "default_fhs_ewma_lambda")]
    lambda: f64,
}

fn default_fhs_var_confidence() -> f64 { 0.95 }
fn default_fhs_ewma_lambda() -> f64 { 0.94 }

async fn value_at_risk_filtered_historical_route(
    _u: AuthUser, Json(b): Json<ValueAtRiskFilteredHistoricalBody>,
) -> Json<Option<ValueAtRiskFilteredHistoricalResponse>> {
    Json(value_at_risk_filtered_historical::compute(&b.returns, b.confidence, b.lambda)
        .map(|r| ValueAtRiskFilteredHistoricalResponse {
            var: r.var,
            expected_shortfall: r.expected_shortfall,
            current_sigma: r.current_sigma,
            n: r.n,
        }))
}

#[derive(serde::Serialize)]
struct ValueAtRiskFilteredHistoricalResponse {
    var: f64,
    expected_shortfall: f64,
    current_sigma: f64,
    n: usize,
}

#[derive(Deserialize)]
struct KalmanDynamicBetaBody {
    asset: Vec<f64>,
    bench: Vec<f64>,
    #[serde(default = "default_kalman_q")]
    process_noise_q: f64,
    #[serde(default = "default_kalman_r")]
    obs_noise_r: f64,
    #[serde(default = "default_kalman_beta0")]
    beta0: f64,
    #[serde(default = "default_kalman_p0")]
    p0: f64,
}

fn default_kalman_q() -> f64 { 1e-6 }
fn default_kalman_r() -> f64 { 1e-4 }
fn default_kalman_beta0() -> f64 { 1.0 }
fn default_kalman_p0() -> f64 { 1.0 }

async fn kalman_dynamic_beta_route(
    _u: AuthUser, Json(b): Json<KalmanDynamicBetaBody>,
) -> Json<Vec<Option<f64>>> {
    Json(kalman_dynamic_beta::compute(
        &b.asset, &b.bench,
        b.process_noise_q, b.obs_noise_r,
        b.beta0, b.p0,
    ))
}

#[derive(Deserialize)]
struct GbmPathSimulatorBody {
    s0: f64,
    mu: f64,
    sigma: f64,
    dt: f64,
    steps: usize,
    paths: usize,
    #[serde(default)]
    seed: u64,
}

async fn gbm_path_simulator_route(
    _u: AuthUser, Json(b): Json<GbmPathSimulatorBody>,
) -> Json<Option<GbmPathSimulatorResponse>> {
    Json(gbm_path_simulator::compute(b.s0, b.mu, b.sigma, b.dt, b.steps, b.paths, b.seed)
        .map(|r| GbmPathSimulatorResponse {
            mean_terminal: r.mean_terminal,
            stdev_terminal: r.stdev_terminal,
            min_terminal: r.min_terminal,
            max_terminal: r.max_terminal,
            paths_run: r.paths_run,
        }))
}

#[derive(serde::Serialize)]
struct GbmPathSimulatorResponse {
    mean_terminal: f64,
    stdev_terminal: f64,
    min_terminal: f64,
    max_terminal: f64,
    paths_run: usize,
}

#[derive(Deserialize)]
struct JumpDiffusionSimulatorBody {
    s0: f64,
    mu: f64,
    sigma: f64,
    jump_lambda: f64,
    jump_mean: f64,
    jump_stdev: f64,
    dt: f64,
    steps: usize,
    paths: usize,
    #[serde(default)]
    seed: u64,
}

async fn jump_diffusion_simulator_route(
    _u: AuthUser, Json(b): Json<JumpDiffusionSimulatorBody>,
) -> Json<Option<JumpDiffusionSimulatorResponse>> {
    Json(jump_diffusion_simulator::compute(
        b.s0, b.mu, b.sigma,
        b.jump_lambda, b.jump_mean, b.jump_stdev,
        b.dt, b.steps, b.paths, b.seed,
    ).map(|r| JumpDiffusionSimulatorResponse {
        mean_terminal: r.mean_terminal,
        stdev_terminal: r.stdev_terminal,
        mean_log_return: r.mean_log_return,
        skew_log_return: r.skew_log_return,
        jump_count_total: r.jump_count_total,
        paths_run: r.paths_run,
    }))
}

#[derive(serde::Serialize)]
struct JumpDiffusionSimulatorResponse {
    mean_terminal: f64,
    stdev_terminal: f64,
    mean_log_return: f64,
    skew_log_return: f64,
    jump_count_total: u64,
    paths_run: usize,
}

#[derive(Deserialize)]
struct FbmGeneratorBody {
    hurst: f64,
    #[serde(default = "default_fbm_sigma")]
    sigma0: f64,
    #[serde(default = "default_fbm_levels")]
    levels: u32,
    #[serde(default)]
    seed: u64,
}

fn default_fbm_sigma() -> f64 { 1.0 }
fn default_fbm_levels() -> u32 { 10 }

async fn fractional_brownian_motion_generator_route(
    _u: AuthUser, Json(b): Json<FbmGeneratorBody>,
) -> Json<Option<Vec<f64>>> {
    Json(fractional_brownian_motion_generator::compute(b.hurst, b.sigma0, b.levels, b.seed))
}

#[derive(Deserialize)]
struct MultiLegOptionPricerBody {
    legs: Vec<MultiLegOptionLeg>,
    spot: f64,
    t_to_expiry: f64,
    rate: f64,
    div_yield: f64,
    sigma: f64,
}

#[derive(Deserialize)]
struct MultiLegOptionLeg {
    kind: String,
    strike: f64,
    premium: f64,
    qty: f64,
}

async fn multi_leg_option_pricer_route(
    _u: AuthUser, Json(b): Json<MultiLegOptionPricerBody>,
) -> Json<Option<MultiLegOptionPricerResponse>> {
    let legs: Vec<multi_leg_option_pricer::Leg> = b.legs.iter().map(|l| {
        let kind = match l.kind.to_lowercase().as_str() {
            "call" => multi_leg_option_pricer::LegKind::Call,
            "put" => multi_leg_option_pricer::LegKind::Put,
            _ => multi_leg_option_pricer::LegKind::Underlying,
        };
        multi_leg_option_pricer::Leg { kind, strike: l.strike, premium: l.premium, qty: l.qty }
    }).collect();
    Json(multi_leg_option_pricer::compute(
        &legs, b.spot, b.t_to_expiry, b.rate, b.div_yield, b.sigma,
    ).map(|r| MultiLegOptionPricerResponse {
        strategy_value: r.strategy_value,
        strategy_pnl: r.strategy_pnl,
        leg_values: r.leg_values,
        leg_pnls: r.leg_pnls,
    }))
}

#[derive(serde::Serialize)]
struct MultiLegOptionPricerResponse {
    strategy_value: f64,
    strategy_pnl: f64,
    leg_values: Vec<f64>,
    leg_pnls: Vec<f64>,
}

#[derive(Deserialize)]
struct MarkovSwitching2StateBody {
    returns: Vec<f64>,
}

async fn markov_switching_2state_route(
    _u: AuthUser, Json(b): Json<MarkovSwitching2StateBody>,
) -> Json<Option<MarkovSwitching2StateResponse>> {
    Json(markov_switching_2state::compute(&b.returns).map(|r| MarkovSwitching2StateResponse {
        prob_state1: r.prob_state1,
        mu0: r.mu0, mu1: r.mu1,
        sigma0: r.sigma0, sigma1: r.sigma1,
        p00: r.p00, p11: r.p11,
        log_likelihood: r.log_likelihood,
        iterations: r.iterations,
    }))
}

#[derive(serde::Serialize)]
struct MarkovSwitching2StateResponse {
    prob_state1: Vec<f64>,
    mu0: f64, mu1: f64,
    sigma0: f64, sigma1: f64,
    p00: f64, p11: f64,
    log_likelihood: f64,
    iterations: u32,
}

#[derive(Deserialize)]
struct MicropriceStoikovBody {
    quotes: Vec<MicropriceStoikovQuote>,
}

#[derive(Deserialize)]
struct MicropriceStoikovQuote {
    bid: f64,
    ask: f64,
    bid_size: f64,
    ask_size: f64,
}

async fn microprice_stoikov_route(
    _u: AuthUser, Json(b): Json<MicropriceStoikovBody>,
) -> Json<Vec<Option<MicropriceStoikovBar>>> {
    let quotes: Vec<microprice_stoikov::L1Quote> = b.quotes.iter().map(|q| {
        microprice_stoikov::L1Quote {
            bid: q.bid, ask: q.ask, bid_size: q.bid_size, ask_size: q.ask_size,
        }
    }).collect();
    let out = microprice_stoikov::compute(&quotes);
    Json(out.into_iter().map(|opt| opt.map(|b| MicropriceStoikovBar {
        microprice: b.microprice,
        midpoint: b.midpoint,
        imbalance: b.imbalance,
        bias_bps: b.bias_bps,
    })).collect())
}

#[derive(serde::Serialize)]
struct MicropriceStoikovBar {
    microprice: f64,
    midpoint: f64,
    imbalance: f64,
    bias_bps: f64,
}

#[derive(Deserialize)]
struct SviVolatilitySmileBody {
    log_moneyness: Vec<f64>,
    total_variance: Vec<f64>,
    expiry_years: f64,
}

async fn svi_volatility_smile_route(
    _u: AuthUser, Json(b): Json<SviVolatilitySmileBody>,
) -> Json<Option<SviVolatilitySmileResponse>> {
    Json(svi_volatility_smile::compute(&b.log_moneyness, &b.total_variance, b.expiry_years)
        .map(|r| SviVolatilitySmileResponse {
            a: r.params.a, b: r.params.b, rho: r.params.rho, m: r.params.m, sigma: r.params.sigma,
            fitted_total_var: r.fitted_total_var,
            fitted_iv: r.fitted_iv,
            rmse_total_var: r.rmse_total_var,
            arbitrage_ok: r.arbitrage_ok,
        }))
}

#[derive(serde::Serialize)]
struct SviVolatilitySmileResponse {
    a: f64, b: f64, rho: f64, m: f64, sigma: f64,
    fitted_total_var: Vec<f64>,
    fitted_iv: Vec<f64>,
    rmse_total_var: f64,
    arbitrage_ok: bool,
}

#[derive(Deserialize)]
struct DupireLocalVolatilityBody {
    call_prices: Vec<Vec<f64>>,
    strikes: Vec<f64>,
    expiries: Vec<f64>,
    risk_free_rate: f64,
}

async fn dupire_local_volatility_route(
    _u: AuthUser, Json(b): Json<DupireLocalVolatilityBody>,
) -> Json<Option<DupireLocalVolatilityResponse>> {
    Json(dupire_local_volatility::compute(&b.call_prices, &b.strikes, &b.expiries, b.risk_free_rate)
        .map(|r| DupireLocalVolatilityResponse {
            local_var: r.local_var,
            local_vol: r.local_vol,
        }))
}

#[derive(serde::Serialize)]
struct DupireLocalVolatilityResponse {
    local_var: Vec<Vec<Option<f64>>>,
    local_vol: Vec<Vec<Option<f64>>>,
}

#[derive(Deserialize)]
struct DynamicTimeWarpingBody {
    a: Vec<f64>,
    b: Vec<f64>,
    #[serde(default)]
    band_radius: usize,
}

async fn dynamic_time_warping_route(
    _u: AuthUser, Json(req): Json<DynamicTimeWarpingBody>,
) -> Json<Option<DynamicTimeWarpingResponse>> {
    Json(dynamic_time_warping::compute(&req.a, &req.b, req.band_radius)
        .map(|r| DynamicTimeWarpingResponse {
            distance: r.distance,
            path: r.path,
        }))
}

#[derive(serde::Serialize)]
struct DynamicTimeWarpingResponse {
    distance: f64,
    path: Vec<(usize, usize)>,
}

#[derive(Deserialize)]
struct WaveletDecompositionHaarBody {
    series: Vec<f64>,
    #[serde(default = "default_haar_levels")]
    levels: u32,
}

fn default_haar_levels() -> u32 { 4 }

async fn wavelet_decomposition_haar_route(
    _u: AuthUser, Json(b): Json<WaveletDecompositionHaarBody>,
) -> Json<Option<WaveletDecompositionHaarResponse>> {
    Json(wavelet_decomposition_haar::compute(&b.series, b.levels)
        .map(|r| WaveletDecompositionHaarResponse {
            approximation: r.approximation,
            details: r.details,
            levels: r.levels,
            used_length: r.used_length,
        }))
}

#[derive(serde::Serialize)]
struct WaveletDecompositionHaarResponse {
    approximation: Vec<f64>,
    details: Vec<Vec<f64>>,
    levels: u32,
    used_length: usize,
}
