//! `traderview-core` — domain types, FIFO roll-up, per-asset P&L, statistics,
//! risk + R-multiple, MFE/MAE excursion, liquidity, slug helpers.
//!
//! Pure-Rust, no I/O. Consumed by `traderview-db` (persistence), `traderview-web`
//! (HTTP), `traderview-import` (broker parsers), and `traderview-desktop`.

pub mod adx;
pub mod alligator;
pub mod anchored_vwap;
pub mod aroon;
pub mod atr_cone;
pub mod awesome_oscillator;
pub mod backtest;
pub mod bb_squeeze;
pub mod beta;
pub mod beta_hedge;
pub mod bond_duration;
pub mod bracket_order;
pub mod buying_power;
pub mod cagr;
pub mod calendar_bias;
pub mod candlestick_patterns;
pub mod carry_score;
pub mod cci;
pub mod chaikin_volatility;
pub mod cluster_analysis;
pub mod commission_optimizer;
pub mod concentration;
pub mod coppock;
pub mod correlation;
pub mod correlation_clusters;
pub mod cost_basis;
pub mod currency_exposure;
pub mod daily_loss_limit;
pub mod discipline_score;
pub mod donchian;
pub mod dow_hour_heatmap;
pub mod dpo;
pub mod drawdown_duration;
pub mod drawdown_throttle;
pub mod dynamic_kelly;
pub mod earnings_calendar;
pub mod earnings_move;
pub mod emotion_tags;
pub mod equity_forecast;
pub mod equity_regime;
pub mod excursion;
pub mod exit_timing;
pub mod fibonacci;
pub mod force_index;
pub mod futures_roll;
pub mod gap_analysis;
pub mod gex;
pub mod goal_tracker;
pub mod greeks;
pub mod halt_risk;
pub mod hedge_ratio;
pub mod heikin_ashi;
pub mod high_water_mark;
pub mod holiday_calendar;
pub mod ichimoku;
pub mod indicators;
pub mod intraday_heatmap;
pub mod iv_backtest;
pub mod iv_rank;
pub mod iv_skew;
pub mod kelly;
pub mod keltner;
pub mod liquidity;
pub mod mae_stop_tuning;
pub mod margin_call;
pub mod margin_runway;
pub mod market_impact;
pub mod mass_index;
pub mod max_pain;
pub mod mfi;
pub mod models;
pub mod monte_carlo;
pub mod mtm_reconciliation;
pub mod news_event_handler;
pub mod oi_change;
pub mod open_type;
pub mod optimal_f;
pub mod options_margin;
pub mod order_book_imbalance;
pub mod order_flow;
pub mod order_staleness;
pub mod overtrading;
pub mod pair_trade;
pub mod parabolic_sar;
pub mod pead;
pub mod per_symbol_slippage;
pub mod pivots;
pub mod pnl;
pub mod portfolio_greeks;
pub mod position_aging;
pub mod position_irr;
pub mod position_size;
pub mod probability_of_touch;
pub mod profit_factor;
pub mod put_call_ratio;
pub mod pyramid;
pub mod pyramid_rules;
pub mod rebalance;
pub mod reconcile_1099b;
pub mod renko;
pub mod risk;
pub mod risk_gate;
pub mod risk_metrics;
pub mod risk_on_off;
pub mod risk_parity;
pub mod risk_reward;
pub mod roc;
pub mod rolling_zscore;
pub mod rollup;
pub mod rsi_divergence;
pub mod scan;
pub mod schaff_trend;
pub mod sector_exposure;
pub mod sentiment;
pub mod setup_catalog;
pub mod sharpe_by_window;
pub mod signals;
pub mod slug;
pub mod sortino;
pub mod spread_attribution;
pub mod spread_payoff;
pub mod stats;
pub mod stochastic;
pub mod stop_loss_backtest;
pub mod stops;
pub mod strategy_alert;
pub mod strategy_correlation;
pub mod streaks;
pub mod supertrend;
pub mod swing_points;
pub mod symbol_filter;
pub mod tax_loss_harvest;
pub mod tilt_detector;
pub mod time_in_force;
pub mod trade_plan_checklist;
pub mod trade_quality;
pub mod trend_channel;
pub mod treynor;
pub mod triple_screen;
pub mod trix;
pub mod twap;
pub mod var_estimator;
pub mod vix_term_structure;
pub mod volatility_stop;
pub mod volume_flow;
pub mod volume_profile;
pub mod footprint;
pub mod market_profile;
pub mod stress_test;
pub mod tilt_indicator;
pub mod strategy_decay;
pub mod volatility_regime;
pub mod sip_simulator;
pub mod portfolio_heat;
pub mod tax_lot_optimizer;
pub mod spread_tracker;
pub mod implementation_shortfall;
pub mod mean_reversion;
pub mod volume_burst;
pub mod round_levels;
pub mod timeframe_confluence;
pub mod crossover;
pub mod breakout_detector;
pub mod range_contraction;
pub mod stop_hunt;
pub mod fair_value_gap;
pub mod order_block;
pub mod break_of_structure;
pub mod change_of_character;
pub mod equal_levels;
pub mod cumulative_delta;
pub mod displacement;
pub mod opening_range;
pub mod vsa;
pub mod ulcer_index;
pub mod calmar_ratio;
pub mod wyckoff;
pub mod premium_discount;
pub mod cusum;
pub mod heikin_ashi_reversal;
pub mod three_bar_reversal;
pub mod range_expansion;
pub mod choppiness;
pub mod efficiency_ratio;
pub mod random_walk_index;
pub mod acceleration_deceleration;
pub mod liquidity_grab;
pub mod gap_fill_stats;
pub mod arms_index;
pub mod mcclellan_oscillator;
pub mod inside_bar_breakout;
pub mod vortex;
pub mod vwap_bands;
pub mod vwap_slippage;
pub mod vwmacd;
pub mod wash_sale;
pub mod williams_r;
pub mod winloss_asymmetry;
pub mod yield_curve;

// 10 canonical indicators added in one batch — Hull MA / TEMA / DEMA /
// KAMA (moving averages), Stochastic RSI / Ultimate Oscillator / TSI /
// Chaikin Money Flow / PPO (oscillators), Elder Ray (bull/bear power).
// Wired into `traderview_web::routes::chart_indicators`.
pub mod chaikin_money_flow;
pub mod dema;
pub mod elder_ray;
pub mod hull_ma;
pub mod kama;
pub mod ppo;
pub mod stoch_rsi;
pub mod tema;
pub mod tsi;
pub mod ultimate_oscillator;

// Batch 2: 11 more canonical indicators + helpers (Connors RSI, Klinger,
// Ease of Movement, NVI/PVI/PVT bundle, ZLEMA, T3, Aroon Oscillator,
// Center of Gravity, Fisher Transform, WMA, QQE).
pub mod aroon_oscillator;
pub mod center_of_gravity;
pub mod connors_rsi;
pub mod ease_of_movement;
pub mod fisher_transform;
pub mod klinger_oscillator;
pub mod qqe;
pub mod t3_ma;
pub mod volume_indices;
pub mod wma;
pub mod zlema;

// Multi-symbol scanner orchestrator — walks a symbol universe and runs
// every scan preset in parallel via Rayon.
pub mod scan_orchestrator;

// Batch 3: harmonic + ABC + three-drive pattern detectors, squeeze-momentum,
// Bill Williams fractals, DeMarker, VHF, Welles Wilder Swing Index + ASI,
// and a backtest sweep orchestrator.
pub mod abc_pattern;
pub mod backtest_sweep;
pub mod demarker;
pub mod fractals;
pub mod harmonic_patterns;
pub mod squeeze_momentum;
pub mod swing_index;
pub mod three_drive_pattern;
pub mod vhf;

// Batch 4: order-flow primitives + market-internals + walk-forward.
pub mod depth_imbalance;
pub mod footprint_imbalance;
pub mod iv_rank_scanner;
pub mod sector_rotation;
pub mod tape_density;
pub mod tick_extreme;
pub mod walk_forward;

// Batch 6: adaptive MAs (McGinley, VIDYA, FRAMA), Ehlers SuperSmoother,
// Wolfe Wave pattern detector, options term-structure scanner, multi-
// venue sweep detector.
pub mod ehlers_super_smoother;
pub mod frama;
pub mod mcginley_dynamic;
pub mod sweep_detector;
pub mod term_structure_scanner;
pub mod vidya;
pub mod wolfe_wave;

// Batch 7: Relative Vigor Index, Chande Momentum Oscillator, breadth
// cumulative lines (AD-line / new-highs / volume), Ehlers Decycler,
// Post-Earnings Announcement Drift scanner, short-interest squeeze
// scanner, IBD-style relative strength, dark-pool index.
pub mod breadth_lines;
pub mod cmo;
pub mod dark_pool_index;
pub mod ehlers_decycler;
pub mod post_earnings_drift;
pub mod relative_strength;
pub mod relative_vigor_index;
pub mod short_interest_scanner;

// Batch 8: Elder Force Index (EMA-smoothed), Coppock-on-RSI variant,
// Zig-Zag pivot extractor, Relative Volume, Anchored OBV, TTM-style
// Keltner squeeze detector, generic price-vs-indicator divergence.
pub mod anchored_obv;
pub mod coppock_rsi;
pub mod divergence_detector;
pub mod elder_force;
pub mod keltner_squeeze;
pub mod relative_volume;
pub mod zigzag;

// Batch 9: cumulative TICK/TRIN integrator, McClellan Summation Index,
// Hindenburg Omen confirmation detector, premarket gap-percent scanner,
// halt-resume continuation/rejection monitor.
pub mod cumulative_tick_trin;
pub mod halt_resume_monitor;
pub mod hindenburg_omen;
pub mod premarket_gap_scanner;
pub mod summation_index;

// Batch 10: 2nd-order options Greeks (vanna/charm/vomma/veta), VPIN
// order-flow toxicity, Cup-and-Handle + Head-and-Shoulders pattern
// detectors, 52-week breakout scanner, EWMA volatility (RiskMetrics).
pub mod breakout_52w_scanner;
pub mod cup_and_handle;
pub mod ewma_volatility;
pub mod head_shoulders;
pub mod second_order_greeks;
pub mod vpin;

// Batch 11: COT report decoder, calendar-spread analyzer (BS-priced
// back leg), marginal/component VaR (risk budgeting), realized vol +
// bipower variation (Andersen-Bollerslev), Amihud illiquidity ratio,
// Kyle's lambda (price-impact slope), iron-condor P&L.
pub mod amihud_illiquidity;
pub mod calendar_spread;
pub mod cot_report;
pub mod iron_condor;
pub mod kyles_lambda;
pub mod marginal_var;
pub mod realized_volatility;

// Batch 12: TPO/Market Profile (POC + value area), Omega ratio,
// Hurst exponent (R/S analysis), GARCH(1,1) vol model + forecast,
// Engle-Granger cointegration (OLS + ADF), Treynor-Mazuy market-timing
// test, Ornstein-Uhlenbeck mean-reversion fit.
pub mod cointegration;
pub mod garch_1_1;
pub mod hurst_exponent;
pub mod omega_ratio;
pub mod ornstein_uhlenbeck;
pub mod tpo_profile;
pub mod treynor_mazuy;

// Batch 13: range-based volatility (Parkinson/Garman-Klass/Rogers-
// Satchell/Yang-Zhang), Roll (1984) effective spread, Lee-Ready trade
// classifier, variance-swap fair-strike (Carr-Madan), TD Sequential,
// Andrews' Pitchfork projection, anchored momentum (WMA-smoothed).
pub mod anchored_momentum;
pub mod andrews_pitchfork;
pub mod lee_ready;
pub mod range_volatility;
pub mod roll_spread;
pub mod td_sequential;
pub mod variance_swap;

// Batch 14: Information Ratio (active return / tracking error),
// Schwager's Gain-Pain Ratio, Henriksson-Merton market-timing dummy
// regression, Black-Scholes IV solver (Brent's method), Black-76
// futures option pricing + greeks, Deflated Sharpe Ratio (Bailey &
// López de Prado), Murrey Math octave price levels.
pub mod black76;
pub mod deflated_sharpe;
pub mod gain_pain_ratio;
pub mod henriksson_merton;
pub mod information_ratio;
pub mod iv_solver;
pub mod murrey_math;

// Batch 15: Conditional VaR / Expected Shortfall (historical +
// parametric), Fama-French 3-factor + Carhart 4-factor regressions,
// pair-trade z-score signal generator, butterfly P&L (call + put),
// jade-lizard 3-leg P&L (no-upside-risk variant), rolling realized
// correlation matrix with regime-detection mean off-diagonal.
pub mod butterfly_spread;
pub mod conditional_var;
pub mod factor_models;
pub mod jade_lizard;
pub mod pair_trade_zscore;
pub mod realized_correlation;

// Batch 16: Cornish-Fisher VaR (skew/kurt-adjusted parametric VaR),
// Macaulay/modified duration + DV01 + convexity, yield-curve bootstrap
// from coupon bond quotes, Herfindahl-Hirschman concentration index,
// Treynor/Jensen/Modigliani performance trio, risk-parity weights
// solver (Spinu fixed point), Brinson performance attribution.
pub mod brinson_attribution;
pub mod cornish_fisher;
pub mod herfindahl;
pub mod macaulay_duration;
pub mod risk_parity_weights;
pub mod treynor_jensen;
pub mod yield_curve_bootstrap;

// Batch 17: Nelson-Siegel-Svensson yield-curve parametric fit,
// Margrabe spread option (exchange option), geometric Asian option,
// continuous-monitoring barrier option (DI/DO call, UI/UO put),
// Vasicek short-rate zero-coupon bond pricer, Black-Litterman
// portfolio combiner, liquidity-adjusted VaR (Bangia-Diebold).
pub mod asian_option;
pub mod barrier_option;
pub mod black_litterman;
pub mod liquidity_adjusted_var;
pub mod margrabe_spread_option;
pub mod nelson_siegel;
pub mod vasicek;

// Batch 18: CIR (Cox-Ingersoll-Ross) short-rate ZCB pricer with
// Feller condition, Hagan SABR implied-vol approximation,
// Conze-Viswanathan floating-strike lookback option, digital
// (cash/asset-or-nothing) options, Granger causality F-test,
// Ledoit-Wolf covariance shrinkage, Almgren-Chriss optimal execution.
pub mod almgren_chriss;
pub mod cir;
pub mod digital_option;
pub mod granger_causality;
pub mod ledoit_wolf;
pub mod lookback_option;
pub mod sabr;

// Batch 19: Hull-White extended-Vasicek ZCB with curve calibration,
// Geske compound option (option-on-option) with bisection critical
// spot, quanto option (foreign asset, domestic payoff), cliquet
// forward-start ratchet option, Spearman/Kendall rank correlations,
// empirical tail dependence coefficients, vector autoregression VAR(p).
pub mod cliquet_option;
pub mod compound_option;
pub mod hull_white;
pub mod quanto_option;
pub mod rank_correlation;
pub mod tail_dependence;
pub mod vector_autoregression;

// Batch 20: Cholesky decomposition (with correlated-draw multiply),
// PCA via Jacobi eigendecomposition, power option, gap option, FRA
// forward-rate + PV, Black-76 caplet/floorlet, trade-quality stats
// (win rate, profit factor, expectancy, MAE/MFE capture).
pub mod caplet_black76;
pub mod cholesky;
pub mod fra;
pub mod gap_option;
pub mod pca;
pub mod power_option;
pub mod trade_quality_stats;

// Batch 21: Rubinstein chooser option, Conditional Drawdown at Risk
// (CDaR), Sterling/Burke/Ulcer-Performance risk-adjusted ratios,
// Pain Index (Becker mean-abs-drawdown), Stoikov microprice / weighted-
// midprice from L1 quotes, quoted/effective/realized spread analysis
// with adverse-selection decomposition, Asness 12-1 momentum scanner.
pub mod chooser_option;
pub mod conditional_drawdown;
pub mod effective_spread;
pub mod momentum_12_1;
pub mod pain_index;
pub mod risk_adjusted_ratios;
pub mod weighted_midprice;

// Batch 22: Bachelier normal-Black-Scholes (negative-rate options),
// Black swaption pricer, CDS par-spread / PV under standard hazard-rate
// + recovery model, asset-swap spread (bond vs par swap), Holt-Winters
// double-exponential smoother with forecast, Volume-Weighted EMA,
// Stochastic Momentum Index (Blau 1993).
pub mod asset_swap_spread;
pub mod bachelier;
pub mod cds_pricing;
pub mod holt_winters;
pub mod stochastic_momentum_index;
pub mod swaption_black;
pub mod vwema;

// Batch 23: CRR American + Bermudan binomial option pricers,
// convertible-bond binomial with call/put rights, López de Prado
// Hierarchical Risk Parity portfolio allocator, Hawkes-process
// intensity for trade-arrival modeling, ARIMA(1,1,1) iterative-CLS
// estimator with one-step forecast, greeks-vs-spot profile.
pub mod american_binomial;
pub mod arima_111;
pub mod bermudan_binomial;
pub mod convertible_bond;
pub mod greeks_profile;
pub mod hawkes_intensity;
pub mod hierarchical_risk_parity;

// Batch 24: Boyle trinomial-tree American/European pricer, Engle
// ARCH-LM heteroscedasticity test, Ljung-Box Q-test for serial
// autocorrelation, Markowitz min-variance + tangency portfolios,
// 6-pattern candlestick scanner, standalone ADF unit-root test,
// Bollinger %B + Bandwidth oscillators.
//
// (Heston stochastic-vol module was prototyped but pulled — the
// Little-Heston-Trap characteristic-function integral had a sign error
// that would mislead deep-ITM pricing. A correct port requires careful
// complex-arithmetic verification and is queued for a future batch.)
pub mod adf_standalone;
pub mod arch_lm_test;
pub mod bollinger_oscillators;
pub mod candle_patterns;
pub mod ljung_box;
pub mod min_variance_portfolio;
pub mod trinomial_tree;

// Batch 25: Kupiec POF + Christoffersen independence + conditional
// coverage VaR backtests, cross-sectional value/quality/low-vol
// factor scanners, composite multi-factor scoring combiner, survival-
// probability curve from a piecewise-constant hazard schedule.
pub mod composite_factor_scoring;
pub mod low_vol_factor;
pub mod quality_factor;
pub mod survival_probability;
pub mod value_factor;
pub mod var_backtest_christoffersen;
pub mod var_backtest_kupiec;

// Batch 26: straddle / strangle / iron-butterfly / collar P&L
// analyzers, Hodrick-Prescott trend filter, 1-D Kalman filter,
// Künsch block bootstrap for serially-dependent time series.
pub mod block_bootstrap;
pub mod collar;
pub mod hodrick_prescott;
pub mod iron_butterfly;
pub mod kalman_filter_1d;
pub mod straddle;
pub mod strangle;

// Batch 27: realized higher moments (rolling skew/kurt), Bawa-Lindenberg
// lower/upper partial moments, Peng et al. Detrended Fluctuation
// Analysis, Pincus sample entropy, Bandt-Pompe permutation entropy,
// triple-top/bottom pattern detector, Choueifaty-Coignard maximum-
// diversification portfolio solver.
pub mod dfa;
pub mod lower_partial_moments;
pub mod max_diversification;
pub mod permutation_entropy;
pub mod realized_higher_moments;
pub mod sample_entropy;
pub mod triple_top_bottom;

// Batch 28: Patton-Sheppard realized semivariance (downside/upside RV
// decomposition), Barndorff-Nielsen-Shephard bipower variation + jump
// test, Morningstar up/down market capture ratios, Modigliani-Modigliani
// M² risk-adjusted performance, Vasicek Bayesian beta shrinkage,
// Ho key-rate duration vector, Treynor-Black active-portfolio model.
pub mod beta_shrinkage;
pub mod bipower_variation;
pub mod key_rate_duration;
pub mod modigliani_m2;
pub mod realized_semivariance;
pub mod treynor_black;
pub mod up_down_capture;

// Batch 29: Botes-Siepman vortex indicator (VI+/VI-), Floor-Trader /
// Fibonacci / Camarilla pivot points, Tushar Chande Aroon indicator,
// Donchian Channels (Turtle-system breakout bands), Stochastic RSI
// (Chande-Kroll combined oscillator), Bollinger Band Width + %B,
// closed-form + numerical bond convexity.
pub mod aroon_indicator;
pub mod bollinger_band_width;
pub mod bond_convexity;
pub mod donchian_channels;
pub mod pivot_points;
pub mod stochastic_rsi;
pub mod vortex_indicator;

// Batch 30: classic volume-flow indicators + tail/component risk.
// Chaikin A/D line (cumulative MFV), Granville OBV (sign-of-close vol
// tally), Chaikin Oscillator (MACD on ADL), Klinger Volume Oscillator,
// Chande Momentum Oscillator (raw-sum RSI variant), Hill Pareto
// tail-index estimator, Jorion Component VaR (Euler exact decomp).
pub mod accumulation_distribution_line;
pub mod chaikin_oscillator;
pub mod chande_momentum_oscillator;
pub mod component_var;
pub mod hill_estimator;
pub mod klinger_volume_oscillator;
pub mod on_balance_volume;

// Batch 31: adaptive moving averages, classic momentum curves, swing
// auto-leveling, ES contribution. Arnaud Legoux MA (Gaussian-kernel
// FIR), Tim Tillson T3 (6-EMA cascade), Ehlers FRAMA (fractal-
// dimension-driven α), Coppock long-term momentum curve, Detrended
// Price Oscillator (cycle isolation), Fibonacci retracement +
// extension level generator, Acerbi-Tasche Euler-allocation
// component ES.
pub mod alma_legoux;
pub mod coppock_curve;
pub mod detrended_price_oscillator;
pub mod expected_shortfall_contribution;
pub mod fibonacci_retracements;
pub mod frama_fractal;
pub mod t3_moving_average;

// Batch 32: fixed-income spread/swap toolkit, options-flow scanners,
// asymmetric GARCH, equity multi-factor regression, CIP-implied basis.
// Brent-bisected zero-vol spread on bond cash flows, dealer-net Gamma
// Exposure (GEX) per strike, Unusual Options Activity scanner with
// fill-side classification, GJR-GARCH(1,1) ML fit via Nelder-Mead,
// vanilla fixed-vs-float IRS valuation + par rate, Fama-French 3-factor
// OLS with HC-style t-stats, cross-currency basis from FX forward parity.
pub mod cross_currency_basis;
pub mod fama_french_3factor;
pub mod gex_scanner;
pub mod gjr_garch;
pub mod swap_valuation;
pub mod unusual_options_activity;
pub mod z_spread;

// Batch 33: PSA mortgage cash flow generator, Nadaraya-Watson kernel
// smoother, insider cluster-buy scanner, analyst earnings-revision
// scanner, Bulkowski Bump-and-Run reversal pattern, Diamond top/bottom
// reversal pattern, EKOP Probability of Informed Trading (method-of-
// moments approximation).
pub mod bump_and_run;
pub mod diamond_pattern;
pub mod earnings_revision_scanner;
pub mod insider_buying_scanner;
pub mod mortgage_psa;
pub mod nadaraya_watson;
pub mod probability_of_informed_trading;

// Batch 34: multivariate distance + time-series diagnostics + chart
// quantization + adaptive overlays. Mahalanobis distance for outlier /
// regime detection, sample ACF with Bartlett bands, Yule-Walker PACF
// for AR-order ID, Point & Figure column generator, Darvas Box
// breakout system, dual-band SuperTrend regime overlay, Ehlers
// Hilbert-Transform FIR with dominant-cycle estimation.
pub mod autocorrelation_function;
pub mod darvas_box;
pub mod hilbert_transform;
pub mod mahalanobis_distance;
pub mod partial_autocorrelation;
pub mod point_and_figure;
pub mod supertrend_dual;

// Batch 35: statistical tests + advanced vol modeling + copula fits.
// Jarque-Bera normality moment test, Spearman rank correlation with
// t-test, Corsi HAR-RV realized-variance forecaster, Carr-Madan fair
// variance-swap strike from option chain, rank-transform Gaussian
// copula fitter, Chow F-test for structural break in OLS regression,
// Breusch-Godfrey LM test for serial correlation in regression residuals.
pub mod breusch_godfrey;
pub mod chow_test;
pub mod gaussian_copula;
pub mod har_volatility;
pub mod jarque_bera;
pub mod spearman_correlation;
pub mod variance_swap_strike;

// Batch 36: randomness tests + microstructure spread + robust filter +
// fixed-income carry/inflation + position sizing. Lo-MacKinlay variance
// ratio (robust), Wald-Wolfowitz runs test, Corwin-Schultz high-low
// spread estimator, Hampel MAD-based outlier filter, TIPS-vs-Treasury
// breakeven inflation, bond carry + roll-down decomposition, vol-
// targeting position sizer with EWMA smoothing.
pub mod breakeven_inflation;
pub mod carry_roll_decomposition;
pub mod corwin_schultz_spread;
pub mod hampel_filter;
pub mod runs_test;
pub mod variance_ratio_test;
pub mod vol_targeting_sizer;

// Batch 37: distributional comparison + stationarity / heteroskedasticity
// diagnostics + IV-skew screening. Kolmogorov-Smirnov 2-sample test,
// Anderson-Darling normality, KPSS stationarity (complement of ADF),
// Breusch-Pagan heteroskedasticity LM test, KL divergence with JS +
// Hellinger companions, 1D Wasserstein (earth-mover) distance, IV-skew
// scanner emitting 25Δ risk-reversal / butterfly / put-wing slope.
pub mod anderson_darling_normality;
pub mod breusch_pagan_test;
pub mod iv_skew_scanner;
pub mod kolmogorov_smirnov_2sample;
pub mod kpss_test;
pub mod kullback_leibler_divergence;
pub mod wasserstein_1d;

// Batch 38: noise-robust realized-volatility estimator family +
// higher-moment realized statistics. Zhang-Mykland-Aït-Sahalia TSRV,
// subsampled RV averaged over K offsets, Barndorff-Nielsen-Hansen-
// Lunde-Shephard realized kernel (Bartlett/Parzen/Tukey), Hansen-Lunde
// noise-to-signal diagnostic, Amaya-Christoffersen-Jacobs-Vasquez
// realized skewness, BNS realized quarticity + tripower variant,
// Andersen-Dobrev-Schaumburg median realized variance.
pub mod median_realized_variance;
pub mod noise_to_signal_ratio;
pub mod realized_kernel;
pub mod realized_quarticity;
pub mod realized_skewness;
pub mod subsampled_realized_variance;
pub mod two_scales_realized_variance;

// Batch 39: nonparametric 2-sample + paired-sample tests, group-vol
// test, RVOL & IV-term-structure scanners, model-spec test, Székely
// distance correlation. Mann-Whitney U (rank-sum), Wilcoxon signed-
// rank, Brown-Forsythe Levene variance equality, relative-volume
// scanner, IV term-structure slope/convexity analyzer, Ramsey RESET
// functional-form test, Székely distance correlation.
pub mod distance_correlation;
pub mod iv_term_structure;
pub mod levene_test;
pub mod mann_whitney_u;
pub mod ramsey_reset;
pub mod relative_volume_scanner;
pub mod wilcoxon_signed_rank;

// Batch 40: quant signal-quality + options-arbitrage + probabilistic-
// forecast scoring + cross-sectional sorting. Cross-sectional IC +
// Information Ratio, box-spread synthetic-rate arbitrage, jelly-roll
// calendar-arbitrage detector, factor-neutralization orthogonalizer,
// CRPS for ensemble forecasts, Brier score + Murphy decomposition,
// decile long-short portfolio constructor.
pub mod box_spread;
pub mod brier_score;
pub mod continuous_ranked_probability_score;
pub mod decile_long_short_signal;
pub mod factor_neutralization;
pub mod information_coefficient;
pub mod jelly_roll_arbitrage;

// Batch 41: HAC/HC robust standard errors + forecast accuracy + gamma
// scalping P&L + Breeden-Litzenberger implied density + ECE + vol risk
// premium. Newey-West HAC SEs for serially-correlated residuals,
// Diebold-Mariano forecast equality test, gamma-scalping discretized
// P&L sim, Breeden-Litzenberger risk-neutral density extraction,
// White HC0/1/2/3 robust SEs, Naeini-Cooper-Hauskrecht ECE + MCE,
// implied-vs-realized volatility risk premium analyzer.
pub mod breeden_litzenberger;
pub mod diebold_mariano;
pub mod expected_calibration_error;
pub mod gamma_scalping_pnl;
pub mod newey_west;
pub mod vol_risk_premium;
pub mod white_robust_se;

// Batch 42: bank-funding stress + classical variance equality +
// non-param repeated measures + monotone fit + multiple changepoints +
// permanent/transitory decomposition + MC VaR. LIBOR-OIS spread,
// Bartlett variance test, Friedman rank test, PAVA isotonic regression,
// PELT changepoint detection, Gonzalo-Granger decomposition, Monte
// Carlo VaR with ES.
pub mod bartlett_variance_test;
pub mod friedman_test;
pub mod gonzalo_granger_decomposition;
pub mod isotonic_regression;
pub mod libor_ois_spread;
pub mod monte_carlo_var;
pub mod pelt_segmentation;

// Batch 43: extreme-value-theory toolkit + quantile regression +
// distribution-free CDF + megaphone pattern. Hosking-Wallis GPD fit
// via PWM, POT methodology with mean-residual-life diagnostic, EVT
// VaR + ES extrapolation, Pickands shape estimator, ECDF with DKW
// confidence band, Koenker-Bassett IRLS quantile regression,
// megaphone (broadening) top/bottom pattern detector.
pub mod empirical_distribution_function;
pub mod evt_value_at_risk;
pub mod gpd_tail_fit;
pub mod megaphone_pattern;
pub mod peaks_over_threshold;
pub mod pickands_estimator;
pub mod quantile_regression;

// Batch 44: rolling risk-adjusted statistics + cointegration test +
// E[MDD] simulator + VCP pattern. Rolling max drawdown, rolling
// Sharpe, rolling Sortino, rolling beta + α + R² vs benchmark,
// Monte-Carlo Expected Drawdown with quantiles, Engle-Granger 2-step
// cointegration test, Minervini Volatility Contraction Pattern.
pub mod engle_granger_2step;
pub mod expected_drawdown;
pub mod rolling_beta;
pub mod rolling_drawdown;
pub mod rolling_sharpe;
pub mod rolling_sortino;
pub mod vcp_pattern;

// Batch 45: drawdown-adjusted performance ratios + Schwager gain/pain
// + tail asymmetry + Weinstein stage classifier + per-trade expectancy.
// Burke (sum-sq DD), Sterling (mean top-K DD), Recovery Factor + MAR,
// Schwager Gain-to-Pain ratio + index, tail ratio + common-sense ratio,
// Weinstein 4-stage trend classification, per-trade expectancy + R.
pub mod burke_ratio;
pub mod expectancy_per_trade;
pub mod gain_to_pain_ratio;
pub mod recovery_factor;
pub mod sterling_ratio;
pub mod tail_ratio;
pub mod weinstein_stages;

// Batch 46: Kelly sizing + tracking-error / active-share benchmark
// stats + SG smoothing + Minervini & O'Neil trend setups + bootstrap.
// Kelly (discrete + continuous), tracking error + IR, Cremers-Petajisto
// active share, Savitzky-Golay polynomial smoother, Minervini 8-criterion
// trend template, bootstrap P&L CI, O'Neil pocket-pivot detector.
pub mod active_share;
pub mod bootstrap_pnl;
pub mod kelly_criterion;
pub mod minervini_trend_template;
pub mod pocket_pivot_buy;
pub mod savitzky_golay;
pub mod tracking_error;

// Batch 47: PDE/MC options + forward-start + saucer/island patterns +
// TED spread + Moreira-Muir vol-managed portfolio. Crank-Nicolson FD
// European pricer with delta/gamma, GBM MC with antithetic, Rubinstein
// forward-start, quadratic-fit saucer top/bottom, gap-island reversal,
// TED stress indicator, vol-managed portfolio scaler.
pub mod finite_difference_option;
pub mod forward_start_option;
pub mod island_reversal;
pub mod monte_carlo_option;
pub mod rounding_pattern;
pub mod ted_spread;
pub mod volatility_managed_portfolio;

// Batch 48: vol swap + NSS yield curve + Phillips-Perron + key-reversal
// bar + Daniel-Moskowitz momentum crash protection + Student-t copula
// + Welch PSD. DDKZ-style convex-adjusted vol-swap strike, Nelson-Siegel-
// Svensson 6-parameter yield curve, PP HAC-corrected unit-root test,
// classic single-bar key reversal, Daniel-Moskowitz vol-scaled momentum
// with crash filter, Student-t copula with inv-t marginals, Welch
// periodogram for power spectral density.
pub mod key_reversal_bar;
pub mod momentum_crash_protection;
pub mod nelson_siegel_svensson;
pub mod pp_test;
pub mod t_copula;
pub mod volatility_swap;
pub mod welch_periodogram;

// Batch 49: classic technical oscillators + line studies. Williams
// accumulation/distribution, Chande trend index, balance of power,
// Donald Dorsey RVI, DeMarker bounded oscillator, Woodies CCI (turbo
// + standard + TLB), Lee Leibfarth Premier Stochastic.
pub mod balance_of_power;
pub mod chande_trend_index;
pub mod demarker_oscillator;
pub mod premier_stochastic;
pub mod relative_volatility_index;
pub mod williams_accumulation_distribution;
pub mod woodies_cci;

// Batch 50: classic indicator catalogue gap-fillers. Chande Q-Stick,
// Pring Know-Sure-Thing, Nison Disparity Index, Camarilla pivot levels,
// Raff linear-regression channel, Bill Williams Gator Oscillator,
// triangular moving average.
pub mod camarilla_pivots;
pub mod disparity_index;
pub mod gator_oscillator;
pub mod know_sure_thing;
pub mod linear_regression_channel;
pub mod qstick;
pub mod triangular_ma;

// Batch 51: volume-confirmation & adaptive-stop indicators. Granville
// PVT, Dysart/Fosback NVI + PVI, Stoller STARC bands, Guppy GMMA
// ribbons, Wilder ASI, Twiggs Money Flow, Elder Safe-Zone Stop.
pub mod accumulation_swing_index;
pub mod elder_safezone_stop;
pub mod guppy_mma;
pub mod negative_volume_index;
pub mod positive_volume_index;
pub mod price_volume_trend;
pub mod starc_bands;
pub mod twiggs_money_flow;

// Batch 52: adaptive smoothers + volatility stops + composite trend
// indicators. Mark Jurik JMA, Chande-Kroll dynamic stop, Elder Market
// Thermometer, classic Floor pivots, Dean Malone TDI, John Carter
// TTM Squeeze, Bill Williams Elliott Wave Oscillator.
pub mod chande_kroll_stop;
pub mod elder_thermometer;
pub mod elliott_wave_oscillator;
pub mod floor_pivots;
pub mod jurik_ma;
pub mod traders_dynamic_index;
pub mod ttm_squeeze;

// Batch 53: pivot variants + filter banks + volume-wave / volatility-quality.
// Ken Wood Woodie pivots, Fibonacci pivots, Mark Johnson PGO, Ehlers
// Roofing Filter, David Weis Wave volume, John Carter TTM Trend bars,
// Thomas Stridsman Volatility Quality Index.
pub mod fibonacci_pivots;
pub mod pretty_good_oscillator;
pub mod roofing_filter;
pub mod ttm_trend;
pub mod volatility_quality_index;
pub mod weiss_wave;
pub mod woodie_pivots;

// Batch 54: pivot + regime + classic-floor synthesis. DeMark pivots,
// Krausz Gann high-low activator, Elder Impulse System, Damiani
// volatmeter regime gauge, Ehlers Instant Trendline + trigger,
// Donovan Wall Range Filter, Linda Raschke 3-10 oscillator.
pub mod damiani_volatmeter;
pub mod demark_pivots;
pub mod ehlers_instant_trendline;
pub mod gann_high_low_activator;
pub mod impulse_system;
pub mod linda_raschke_3_10;
pub mod range_filter;

// Batch 55: Ehlers MAMA/FAMA + DSS + TAZ + III + DMI + SE bands + CTI.
// John Ehlers MAMA/FAMA adaptive pair, Walter Bressert double-smoothed
// stochastic, Vince Wagner Trader's Action Zone, David Bostian
// Intraday Intensity Index, Chande Dynamic Momentum Index, Andersen
// Standard Error Bands, Ehlers Correlation Trend Indicator.
pub mod bressert_dss;
pub mod chande_dynamic_momentum_index;
pub mod ehlers_correlation_trend_indicator;
pub mod ehlers_mama_fama;
pub mod intraday_intensity_index;
pub mod standard_error_bands;
pub mod traders_action_zone;

// Batch 56: trailing stops + breadth + pattern + slope. Chuck LeBeau
// Chandelier Exit, Raschke Holy Grail setup, Pring volume oscillator,
// Elder chop zone, Kivanc Ozbilgic AlphaTrend, OLS linear regression
// slope-only, Martin Ulcer Performance Index.
pub mod alphatrend;
pub mod chandelier_exit;
pub mod chop_zone_indicator;
pub mod holy_grail;
pub mod linear_regression_slope;
pub mod ulcer_performance_index;
pub mod volume_oscillator;

// Batch 57: %B + VZO + Gartley harmonic + Pinball + AVP + RMA + DSP.
// John Bollinger %B, Walid Khalil VZO, Gartley 222 harmonic detector,
// Linda Raschke Pinball reversal, Don Cassidy Accumulation Volume
// Pattern, Roy Larsen Recursive MA (Wilder), Ehlers Detrended Synthetic
// Price.
pub mod accumulation_volume_pattern;
pub mod bollinger_percent_b;
pub mod detrended_synthetic_price;
pub mod gartley_pattern;
pub mod pinball_setup;
pub mod recursive_ma;
pub mod volume_zone_oscillator;

// Batch 58: full harmonic-pattern set + Raschke pattern catalogue
// completions. Scott Carney Bat / Crab / Shark, Gilmore Butterfly,
// Oglesbee Cypher, Raschke Turtle Soup false-breakout, Raschke
// 80-20 reversal setup.
pub mod bat_pattern;
pub mod butterfly_pattern;
pub mod crab_pattern;
pub mod cypher_pattern;
pub mod eighty_twenty_setup;
pub mod shark_pattern;
pub mod turtle_soup;

// Batch 59: bands + regression + flow + Raschke Anti + Chande CMI.
// Rolling z-score, Markos Katsanos VFI, linear regression curve,
// percent-offset MA envelope, Bollinger squeeze (rolling-min width),
// Raschke Anti pullback setup, Chande Choppy Market Index.
pub mod anti_setup;
pub mod bollinger_squeeze;
pub mod choppy_market_index;
pub mod katsanos_vfi;
pub mod linear_regression_curve;
pub mod moving_average_envelope;
pub mod z_score_indicator;

// Batch 60: Madrid ribbon + ATR-velocity + Schwartz VBS + DVO + AD
// oscillator + CSI + Hawkins dynamic zones. 8-EMA Madrid color-coded
// ribbon, ATR-normalized momentum, Larry Williams / Schwartz volatility
// breakout, percent-rank-based detrended-volatility oscillator,
// per-bar Chaikin AD oscillator, candle body-strength EMA, Zamansky-
// Stendahl adaptive overbought/oversold zones.
pub mod accumulation_distribution_oscillator;
pub mod candle_strength_index;
pub mod detrended_volatility_oscillator;
pub mod hawkins_dynamic_zones;
pub mod madrid_moving_average_ribbon;
pub mod velocity_indicator;
pub mod volatility_breakout_system;

// Batch 61: Ehlers DSO + Pesavento ABCD + Gann fan + price-ratio /
// price-spread charts + BBWP + RS-vs-market.  Ehlers decycler-
// oscillator high-pass, ABCD 4-pivot harmonic, 9-line Gann angle fan,
// asset-A / asset-B ratio + spread charts, BB-width percentile,
// IBD-style relative-strength line.
pub mod abcd_pattern;
pub mod bollinger_bandwidth_percentile;
pub mod ehlers_decycler_oscillator;
pub mod gann_fan;
pub mod ratio_chart;
pub mod relative_strength_vs_market;
pub mod spread_chart;

// Batch 62: Ehlers Ultimate Smoother + CSM + Carney 5-0 harmonic +
// TP/WC primitives + engulfing scanner + 52w high-low scanner.
pub mod ehlers_centered_smoothed_momentum;
pub mod engulfing_pattern_scanner;
pub mod fifty_two_week_high_low_scanner;
pub mod five_o_pattern;
pub mod typical_price;
pub mod ultimate_smoother;
pub mod weighted_close;

// Batch 63: microstructure + breadth fillers. Session VWAP with bands,
// per-bar tape speed, pivot-cluster liquidity pools, absorption-bar
// detector, iceberg-print detector, median-price primitive, Arms
// high-low breadth index.
pub mod absorption_detector;
pub mod arms_high_low_index;
pub mod iceberg_detector;
pub mod liquidity_pool_detector;
pub mod median_price;
pub mod session_vwap;
pub mod tape_speed;

pub use models::*;
