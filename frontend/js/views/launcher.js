// Launcher — single tile grid replacing the 77-tab strip. Categorized,
// searchable, keyboard-navigable. Becomes the default landing under "Home".

import { go } from '../app.js';
import { esc } from '../util.js';
import { t, applyUiI18n } from '../i18n.js';
import { upgradeTooltips } from '../tooltip.js';
import { matchesQuery } from '../_pure.js';
import * as favs from '../_favorites_storage.js';
import * as dashStore from '../_dashboards_storage.js';
import * as recents from '../_recents_storage.js';

// (view-id, label, glyph, description, badge | null)
// `badge` shows a small chip — "LIVE" for streaming tiles, etc.
// Exported so the Dashboards view can reuse this canonical view-id list
// for its "add tile" picker without duplicating registration.
export const TILES = [
    // — Live markets (always-on streams) ———————————————————————
    ['live-scanner', 'Live Scanner',  '⚡',  'Finnhub WS · 6-panel real-time scanner', 'LIVE'],
    ['halts',        'Halts',         '⏸',  'Nasdaq halt RSS + TTS voice alerts',     'LIVE'],
    ['catalysts',    'Catalysts',     '📰',  'SEC EDGAR + 4 PR wires, ticker NER',     'LIVE'],
    ['premarket',    'Pre-market',    '🌅',  'Pre-market gappers + opening drive',     'LIVE'],
    ['tape',         'Tape',          '🎞',  'Time & sales stream',                    'LIVE'],
    ['heatmap',      'Heatmap',       '🟩',  'Sector / S&P heatmap',                   null],

    // — Trading (your accounts + execution) ————————————————————
    ['webull',          'Webull',          '🪙',  'Live broker positions + orders',   'LIVE'],
    ['live',            'Live Positions',  '💰',  'Open positions w/ live P/L',       'LIVE'],
    ['paper',           'Paper Trade',     '📝',  'Simulated execution',              null],
    ['new-trade',       'New Trade',       '＋',  'Manual order entry',               null],
    ['plans',           'Pre-trade Plans', '🎯',  'Setups before fills — R:R, stop',  null],
    ['sizing',          'Position Size',   '🧮',  'Risk-based qty calculator',        null],
    ['optimal-f',       'Optimal-f',       '📐',  'Vince geometric-growth-optimal bet fraction', null],
    ['kelly',           'Kelly Sizer',     '🪙',  'Static + dynamic (rolling) Kelly fraction — full / half / quarter', 'NEW'],
    ['mc-trades',       'MC Trades',       '🎲',  'Bootstrap N synthetic equity curves from your R distribution; ending-equity / drawdown / ruin probability', 'NEW'],
    ['commission-optimizer', 'Commission Optimizer', '💸', 'Compare your real fee profile against IBKR / Lightspeed / Webull / custom tiers; projects annual savings', 'NEW'],
    ['margin-runway',   'Margin Runway',   '⚠️',  'How far can your position fall before margin call? Equity vs maintenance projection.', 'NEW'],
    ['risk-parity',     'Risk Parity',     '⚖️',  'Naive 1/σ allocator — each asset contributes the same portfolio variance. RP vs equal-weight side-by-side.', 'NEW'],
    ['risk-on-off',     'Risk On/Off',     '🚦',  'Cross-asset regime classifier — SPY / Gold / DXY / 10Y composite score', 'NEW'],
    ['risk-reward',     'Risk / Reward',   '⚖️',  'Per-trade R:R + auto-sized qty + dollar risk/reward + scale-out plan (1R/2R/target)', 'NEW'],
    ['hotkeys',         'Hotkeys',         '⌨️',  'Keyboard shortcuts editor',        null],
    ['risk-gate',       'Risk Gate',       '🛡',   'Pre-trade rules that block bad trades', null],

    // — Journal (record + learn) ——————————————————————————————
    ['journal',         'Journal',         '📓',  'Per-trade + daily + general notes', null],
    ['ai',              'AI Journal',      '🧠',  'GPT-assisted post-mortem',          null],
    ['reviews',         'Trade Reviews',   '🔁',  'Structured trade review forms',     null],
    ['trade-compare',   'Trade Compare',   '⚖️',  'Two-trade side-by-side',           null],
    ['replay',          'Replay',          '⏯',  'Step through historical fills',     null],
    ['tape-replay',     'Tape Replay',     '⏪',  'Re-stream historical ticks',        null],
    ['discipline',      'Discipline',      '🛡',  'Rule-violation tracker',            null],
    ['mood',            'Mood Analytics',  '🌡',  'Emotion ↔ P&L correlation',         null],
    ['goals',           'Goals',           '🏁',  'Daily / weekly P&L goals',          null],

    // — Charts & research ———————————————————————————————————
    ['charts',          'Charts',          '📈',  'OHLC + indicator overlays',        null],
    ['research',        'Research',        '🔎',  'Per-symbol research dossier',      null],
    ['watchlists',      'Watchlists',      '⭐',  'Symbol watchlists + quotes',        null],
    ['screener',        'Screener',        '🧪',  'Multi-criteria stock screener',    null],
    ['scanners',        'Scanners',        '🛰',  '24 Warrior/Zendoo preset filters',  null],
    ['top-signals',     'Top Signals',     '📡',  'Live signal leaderboard',          null],
    ['compare',         'Compare',         '⚔',   'Multi-symbol comparison',          null],
    ['pairs',           'Pairs',           '🔗',  'Pair-trade spread + Z-score',      null],
    ['correlation',     'Correlation',     '🧬',  'Cross-symbol correlation matrix',  null],
    ['sectors',         'Sectors',         '🏢',  'Sector breakdown + rotation',      null],
    ['sector-rotation', 'Sector Rotation', '🔄',  'Leader/laggard rotation map',      null],
    ['breadth',         'Breadth',         '🌐',  'Adv/dec, A/D line, McClellan',     null],
    ['fear-greed',      'Fear / Greed',    '😱',  'CNN-style sentiment index',        null],
    ['sentiment',       'Sentiment',       '🗯',  'WSB + Stocktwits tracker',         null],
    ['darkpool',        'Dark Pool',       '🕳',  'Off-exchange print scanner',       null],
    ['short-interest',  'Short Interest',  '🩳',  'SI %, days-to-cover, CTB',         null],
    ['vol',             'Volatility',      '📊',  'IV / HV / VRP scanner',            null],
    ['vol-surface',     'Vol Surface',     '🌋',  'Options IV surface plot',          null],
    ['options',         'Options Chain',   '⛓',  'Full chain + Greeks',              null],
    ['option-payoff',   'Option Payoff',   '📐',  'Multi-leg strategy P/L + live MTM', null],
    ['vol-smile',       'Vol Smile',       '🌗',  'SVI smile fit from pasted strike/IV', null],
    ['monte-carlo',     'Monte Carlo',     '🎲',  'GBM / Merton / Kou / fBm path simulator', null],
    ['portfolio-allocator', 'Portfolio Allocator', '⚖️', 'Min-Var · Max-Div · ERC weights side-by-side', null],
    ['cov-denoiser',    'Cov Denoiser',    '🧹',  'Marchenko-Pastur eigenvalue clipping', null],
    ['var-calculator',  'VaR Calculator',  '📉',  'HS · FHS · Cornish-Fisher comparison', null],
    ['var-estimator',   'VaR vs Gaussian', '📊',  'Historical vs parametric-Gaussian VaR + Expected Shortfall on the same returns', 'NEW'],
    ['series-smoother', 'Series Smoother', '∿',   'LOWESS · Kalman · Theil-Sen · Polynomial overlay', null],
    ['pattern-discovery', 'Pattern Discovery', '🧩', 'Matrix Profile motif + discord finder', null],
    ['execution-scheduler', 'Execution Scheduler', '🪡', 'TWAP / VWAP / POV slice schedule', null],
    ['almgren-chriss',  'Almgren-Chriss',   '🪐',  'Optimal-execution trajectory + efficient frontier', null],
    ['implementation-shortfall', 'Impl. Shortfall', '📉', 'TCA: spread + timing + impact + opportunity costs', null],
    ['deflated-sharpe', 'Deflated Sharpe', '🪞', 'Bailey-LdP backtest-overfitting deflator + PSR', null],
    ['vpin',            'VPIN',             '☣️', 'Volume-bucket toxic order-flow detector (Easley-LdP-O\'Hara)', null],
    ['cup-and-handle',  'Cup &amp; Handle', '🍵', 'IBD-style pattern detector + pivot buy-point', null],
    ['iv-rank',         'IV Rank',          '📏', 'IV rank + percentile vs 52w history + environment hint', null],
    ['market-impact',   'Market Impact',    '🌊', 'Slippage by % of ADV — find your participation cliff', null],
    ['liquidity',       'Liquidity',        '💧', 'Position vs ADV per symbol + P&amp;L by ADV bucket', null],
    ['spread-tracker',  'Spread Tracker',   '↔️', 'Bid/ask cost-of-immediacy regime detector', null],
    ['intraday-heatmap', 'Intraday Heatmap', '🗓️', 'P&amp;L by 15-min bucket — find your edge windows', null],
    ['heatmap-dow-hour', 'DOW × Hour Heatmap', '🗓', '7×24 grid: day-of-week × hour-of-day P&amp;L; surfaces best + worst time windows', 'NEW'],
    ['iv-backtest',     'IV Backtest',      '🎲', 'Earnings-straddle long/short backtester w/ recommendation', null],
    ['order-book-imbalance', 'OBI',         '⚖️', 'Level-2 bid/ask imbalance + directional bias', null],
    ['cusum',           'CUSUM',            '🔔', 'Page-Hinkley change-point detector for regime shifts', null],
    ['order-flow',      'Order Flow',       '🌀', 'Lee-Ready aggressor classification + cumulative flow', null],
    ['footprint',       'Footprint',        '🦶', 'Bid/ask volume + delta per price level (Sierra-style)', null],
    ['stress-test',     'Stress Test',      '🧨', 'Portfolio P&amp;L heatmap under price × IV × time shocks', null],
    ['chandelier-stop', 'Chandelier Stop',  '🪝', 'LeBeau ATR-based trailing stop (ratcheted, with trigger detection)', null],
    ['atr-cone',        'ATR Cone',         '🌪',  'Project ±1σ / ±2σ price bands forward N days using σ = ATR × √N (Brownian)', 'NEW'],
    ['round-levels',    'Round Levels',     '🎯',  'Psychological S/R: $1000/$500/$100 (major), $50/$25 (medium), $5/$1 (minor) in window', 'NEW'],
    ['kyles-lambda',    "Kyle's Lambda",    '🔬',  'Kyle (1985) rolling price-impact slope λ = Σxy / Σx². Liquidity depth + flow-sign regime detector', 'NEW'],
    ['hawkes',          'Hawkes Intensity', '💥',  'Self-exciting point process λ(t) = μ + Σ α·exp(−β·Δt). Trade-cluster bursts, branching ratio diagnostic', 'NEW'],
    ['kagi',            'Kagi Chart',       '🪜',  'Japanese price-only chart with directional lines + reversal threshold. Yang/yin (thick/thin) on peak/trough crosses', 'NEW'],
    ['risk-parity-solver', 'RP Solver',     '🧮',  'Spinu (2013) ERC fixed-point solver — equalizes each asset\'s contribution to portfolio variance. Full covariance input', 'NEW'],
    ['volume-at-price', 'Volume @ Price',   '📊',  'Volume Profile histogram — distributes bar volume across price bins; POC + Value-Area bracket (configurable %)', 'NEW'],
    ['herfindahl',      'HHI Concentration', '🏛',  'Herfindahl-Hirschman index — DOJ-scaled portfolio concentration + effective-N. Detects single-name & cluster risk', 'NEW'],
    ['roll-spread',     'Roll Spread',      '🪞',  'Roll (1984) implicit bid-ask spread from serial covariance of trade prints — no quote data required', 'NEW'],
    ['three-line-break', 'Three-Line Break', '🪒', 'Japanese TLB chart — N-line break reversal rule filters noise. N=2 sensitive, N=5 slow. Companion to Kagi', 'NEW'],
    ['momentum-crash',  'Crash Protection', '🛡',  'Daniel-Moskowitz (2016) inverse-vol scaling + trailing-cumret crash filter — cuts momentum tail-losses', 'NEW'],
    ['effective-spread', 'Eff. Spread',     '⚖',  'Lee-Ready/Bessembinder TCA — effective + realized spread + price-impact (adverse-selection cost decomposition)', 'NEW'],
    ['weighted-midprice', 'Microprice',     '⚓',  'Stoikov (2017) order-book-weighted midprice — top-of-book imbalance forecasts short-horizon mid movement', 'NEW'],
    ['marginal-var',    'Marginal VaR',     '💢',  'Risk-budgeting: marginal + component VaR decomposes portfolio tail risk by position. Σ contributions = total VaR', 'NEW'],
    ['range-bar',       'Range Bars',       '📏',  'Aggregate trade prints into fixed-range OHLC bars; time ignored. Volatile periods produce more bars', 'NEW'],
    ['tick-bar',        'Tick Bars',        '🎟',  'One OHLC bar per N prints — event-count normalization. Trailing partial bars dropped', 'NEW'],
    ['volume-bar',      'Volume Bars',      '🧱',  'One OHLC bar per N units of volume — activity-uniform sampling for futures + thin/lull markets', 'NEW'],
    ['dollar-bar',      'Dollar Bars',      '💵',  'One OHLC bar per N dollars of notional — López de Prado AFML preferred sampling for ML; normalizes activity AND price level', 'NEW'],
    ['equivolume',      'Equivolume',       '🟧',  'Richard Arms equivolume — bar WIDTH ∝ volume. Tags Narrow/Normal/Wide/Power (vol + range conviction)', 'NEW'],
    ['imbalance-bar',   'Imbalance Bars',   '⚖️', 'Tick-imbalance bars (López de Prado AFML) — bar closes when |Σ signed size| ≥ threshold. Best i.i.d. sampling', 'NEW'],
    ['active-share',    'Active Share',     '🎯',  'Cremers-Petajisto (2009) — ½·Σ|w_port − w_bench| measures distance from benchmark. AS≥0.60 = truly active', 'NEW'],
    ['brinson',         'Brinson Attrib',   '🥧',  'Brinson (1986) attribution — decomposes active return into allocation / selection / interaction effects', 'NEW'],
    ['black-litterman', 'Black-Litterman',  '🧠',  'Black-Litterman posterior solver — combines equilibrium returns with subjective views (P, Q, Ω, τ)', 'NEW'],
    ['adf-test',        'ADF Test',         '📐',  'Augmented Dickey-Fuller unit-root test — stationarity verdict at 1%/5%/10% critical values', 'NEW'],
    ['aroon',           'Aroon Indicator',  '🏹',  'Chande (1995) Aroon Up/Down/Oscillator — time-since-extreme trend strength + crossover signals', 'NEW'],
    ['amihud',          'Amihud Illiquidity', '💧', 'Amihud (2002) illiq = |r|/$vol·10⁶. Price-impact-per-dollar proxy. Rolling mean over period', 'NEW'],
    ['breadth-thrust',  'Breadth Thrust',   '🚀',  'Zweig (1986) classic bottom signal — EMA(adv/(adv+dec)) crosses 0.40 → 0.615 within 10 bars', 'NEW'],
    ['bb-squeeze',      'BB Squeeze',       '🌀',  'Bollinger squeeze — flags narrow-band % periods (precursor to volatility expansion)', 'NEW'],
    ['balance-of-power', 'Balance of Power', '⚔',  'Livshin BOP = (close−open)/(high−low). Per-bar buyer/seller dominance + SMA-smoothed signal', 'NEW'],
    ['anchored-momentum', 'Anchored Momentum', '⚓', 'ROC vs an anchor bar (earnings/FOMC/halt-resume) + linear-weight WMA smoothing', 'NEW'],
    ['acf',             'Autocorrelation',   '📡', 'Sample ACF + Bartlett 95% CI bands — mean-reversion vs random-walk diagnostic, AR(p) model order', 'NEW'],
    ['beta',            'Beta Regression',  '∫',   'Single-asset β/α/R² vs benchmark + beta-neutral hedge sizing helper', 'NEW'],
    ['brier-score',     'Brier Score',      '🎯',  'Probabilistic forecast accuracy + Murphy reliability/resolution/uncertainty decomposition', 'NEW'],
    ['bipower-variation', 'Bipower Var',    '🌋',  'Barndorff-Nielsen jump-robust IV estimator + Huang-Tauchen jump-detection z-stat', 'NEW'],
    ['bootstrap-pnl',   'Bootstrap P&L',    '🎰',  'Non-parametric trade-resample CIs + probability(positive total P&L)', 'NEW'],
    ['block-bootstrap', 'Block Bootstrap',  '🧱',  'Künsch (1989) block-resample for serially-dependent returns: 4 statistics, 95% CI, bias / significance verdicts', 'NEW'],
    ['ad-normality',    'AD Normality',     '🔔',  'Anderson-Darling normality test (Stephens 1986) — full empirical CDF + small-sample correction + 4 α-levels', 'NEW'],
    ['arch-lm',         'ARCH-LM',          '⚡',  'Engle (1982) Lagrange-Multiplier test for conditional heteroscedasticity — detects when GARCH modeling is warranted', 'NEW'],
    ['alma',            'ALMA',             '〰️',  'Arnaud Legoux Gaussian-kernel FIR moving average — adjustable offset/sigma for lag vs noise tradeoff', 'NEW'],
    ['alphatrend',      'AlphaTrend',       '🅰️',  'Ozbilgic (2021) ATR-trailing trend line gated by Wilder RSI — companion to Supertrend / Parabolic SAR', 'NEW'],
    ['atr-channel',     'ATR Channel',      '🛤️',  'EMA/SMA midline + Wilder-ATR upper/lower bands — volatility envelope for breakout / breakdown signals', 'NEW'],
    ['atr-trailing-stop','ATR Trail Stop',  '🪁',  'Long & short trailing stops at N×ATR from close, ratcheted in favorable direction — companion to Chandelier / Parabolic SAR', 'NEW'],
    ['adl',             'ADL (Chaikin)',    '📊',  'Accumulation/Distribution Line — cumulative Money Flow Volume; divergence detector + 5-tier trend verdict', 'NEW'],
    ['asi',             'ASI (Wilder)',     '🔁',  'Accumulation Swing Index — cumulative Wilder Swing Index with limit_move + breakout vs prior 20 bars', 'NEW'],
    ['ad-oscillator',   'A/D Oscillator',   '〽️',  'Per-bar CLV×Vol + EMA smoothing — current buying pressure (oscillates around 0, distinct from cumulative ADL)', 'NEW'],
    ['beta-shrinkage',  'β Shrinkage',      '🧲',  'Vasicek (1973) Bayesian beta shrinkage — pulls noisy per-asset OLS β toward cross-sectional mean β̄', 'NEW'],
    ['bartlett-variance','Bartlett σ² Test', '⚖️',  'Bartlett (1937) test for equality of variances across ≥ 2 groups — χ² statistic + Wilson-Hilferty p-value', 'NEW'],
    ['bid-ask-volume-ratio','Bid/Ask Vol',  '↔️',  'Rolling Σ bid / Σ ask order-flow imbalance — 7-tier flow verdict + trend + imbalance magnitude', 'NEW'],
    ['bollinger-band-width','BBW + %B',     '〰️',  'Bollinger Band Width + %B — squeeze/expansion detector + 7-tier %B position; dual uPlot (bands + width)', 'NEW'],
    ['bollinger-bandwidth-percentile','BBWP', '📐',  'Bollinger Bandwidth Percentile — vol-regime rank [0, 100] over 252-bar lookback; squeeze trigger detector', 'NEW'],
    ['bollinger-percent-b','Bollinger %B',  '🅱️',  'Standalone Bollinger %B — close position in band envelope; 7-tier zone + breakout/breakdown cross detector', 'NEW'],
    ['bollinger-band-distance','BB Distance', '📏',  'Distance to nearest Bollinger Band normalized by width — 0 = at band, 0.5 = midline, > 0.5 = outside', 'NEW'],
    ['bollinger-oscillators','BB Oscillators','🎚️',  'Combined %B + Bandwidth oscillators — 7-tier %B zone + 5-tier bandwidth-percentile vol regime + TTM-squeeze detector', 'NEW'],
    ['borrow-rate-indicator','Borrow Rate',  '🩸',  'Annualized securities-lending fee + N-bar Δ% — 5-tier hard-to-borrow stress classifier; squeeze-risk gauge', 'NEW'],
    ['breusch-pagan',    'Breusch-Pagan',   '📉',  'Breusch-Pagan (1979) heteroskedasticity test — LM = n·R²_aux ~ χ²(1); decide if White SEs needed', 'NEW'],
    ['burke-ratio',      'Burke Ratio',     '🌋',  'Burke (1994) drawdown-vol risk-adjusted return — (R − Rf) / √Σ DD²; per-trough DD episodes', 'NEW'],
    ['camarilla-pivots', 'Camarilla',       '🎯',  'Nick Stott Camarilla pivots — 8 intraday S/R levels (H4/H3/H2/H1/Pivot/L1/L2/L3/L4) + zone & rule verdicts', 'NEW'],
    ['breusch-godfrey',  'Breusch-Godfrey', '🌀',  'Breusch-Godfrey (1978) serial-correlation LM test — handles lagged regressors (vs Durbin-Watson)', 'NEW'],
    ['candle-strength-index','CSI',         '🕯️',  'Candle Strength Index — EMA of (close − open) / (high − low) body-to-range ratio; ±1 = marubozu', 'NEW'],
    ['carhart-4',       'Carhart 4-Factor', '4️⃣',   'Carhart (1997) Mkt + SMB + HML + WML factor regression — α, β loadings, t-stats, R²; style tilt verdict', 'NEW'],
    ['centered-smoothed-momentum','CSM',    '🎢',  'Ehlers Centered Smoothed Momentum — SuperSmoother-filtered momentum; zero-cross signals trend turn', 'NEW'],
    ['chaikin-oscillator','Chaikin Osc',    '📡',  'Chaikin Oscillator — MACD-style EMA(ADL, fast) − EMA(ADL, slow); zero-crosses + price divergences', 'NEW'],
    ['chande-dynamic-momentum','Chande DMI', '🌗',  'Chande Dynamic Momentum Index — volatility-adaptive RSI (period stretches in quiet markets, shrinks in vol)', 'NEW'],
    ['chande-kroll-stop','Chande-Kroll',     '⛔',  'Chande-Kroll Stop — two-pass volatility trailing stop (HH/LL ± x·ATR, smoothed); long+short bands + regime', 'NEW'],
    ['chande-momentum-oscillator','Chande CMO','🌡️', 'Chande Momentum Oscillator — unsmoothed RSI variant; ±100 range; 6-state cross detector + 7-tier zones', 'NEW'],
    ['chande-trend-index','Chande CTI',     '📐',  'Chande Trend Index — correlation of closes vs linear ramp; pure trend-strength metric ∈ [−1, +1]', 'NEW'],
    ['chande-volatility-index','Chande CVI', '🌪️',  'Chande Volatility Index — % change in EMA of high-low range; flags expanding/contracting volatility regimes', 'NEW'],
    ['chandelier-exit', 'Chandelier Exit',  '🪜',  'LeBeau Chandelier Exit — ATR trailing stop from HH/LL; ratchet + direction flip detector', 'NEW'],
    ['cholesky',        'Cholesky',         '🔢',  'Cholesky A = L·Lᵀ — symmetric PD factorization with conditioning + reconstruction verdict', 'NEW'],
    ['abc-pattern',     'ABC Pattern',      '🔻',  'Elliott-style ABC correction detector — bias / strength / C-extension on 3-pivot windows', 'NEW'],
    ['absorption',      'Absorption',       '🧲',  'Absorption detector — heavy-volume tight-range bars; bull/bear direction from close vs midpoint', 'NEW'],
    ['favorites',       'Favorites',        '⭐',  'Manage your saved favorites + bookmarks — rename, delete, navigate', 'NEW'],
    ['vol-stop-close',  'Vol-Stop (Close)',  '🪢', 'Chandelier variant referenced to highest CLOSE — ignores wicks. Side-by-side compare.', 'NEW'],
    ['time-in-force',   'Time-in-Force',     '⏰', 'DAY / GTC / IOC / FOK / GTD validator — single-order TIF verdict + cheat sheet', 'NEW'],
    ['clusters-trade-features', 'Trade Clusters', '🧬', 'k-means over (entry-time, hold-duration, R) — surfaces hidden trader cohorts', 'NEW'],
    ['clusters-correlation', 'Corr Clusters', '🕸',  'Single-link clustering of positions by |ρ| ≥ threshold — exposes disguised concentration', 'NEW'],
    ['setups-by-setup',  'Setup Stats',    '📒',  'Per-setup leaderboard (trades, win-rate, profit-factor, avg R, expectancy) — kill loser setups', 'NEW'],
    ['cohort-tilt',      'Cohort Tilt',    '🎚',  'TopstepX-style "The Tilt" — cohort positioning bias per symbol (5-tier classification)', 'NEW'],
    ['choppiness',       'Choppiness Idx', '⛵',  'E.W. Dreiss Choppiness Index — trend vs sideways oscillator (0-100, bands at 38.2/61.8)', 'NEW'],
    ['triple-screen',   'Triple Screen',    '🔭', 'Elder 3-timeframe entry filter (weekly + daily + intraday)', null],
    ['vwap-slippage',   'VWAP Slippage',    '🎯', 'TCA: fill vs VWAP — did you beat the benchmark?', null],
    ['twap',            'TWAP',             '⏱️', 'TCA: fill vs time-weighted mean (passive-limit benchmark)', null],
    ['news-event',      'News Event',       '📰', 'Pre-event auto-resize policy — trim by impact (Low/Med/High/Crit)', null],
    ['stop-loss-best-of', 'Stop Best-Of',   '🛑', 'Compete 9 stop strategies on your trades; rank by realized P&amp;L', null],
    ['stop-loss-backtest', 'Stop Backtest', '🎯', 'Single-method stop backtester — replay trades through one stop rule (none/$/% /ATR)', 'NEW'],
    ['futures-roll',    'Futures Roll',    '🔁',  'Roll schedule for open futures positions — NOW / SOON / COMFORTABLE / EXPIRED tiers', 'NEW'],
    ['squeeze-alerts',  'Squeeze Alerts',   '🔔', 'Audio bell + TTS when stocks squeeze (price spike + volume surge)', 'NEW'],
    ['alert-rules',     'Alert Rules',      '⚙️', 'Custom multi-rule alerts (squeeze / price / pct / volume) + per-rule sound + TTS', 'NEW'],
    ['daily-loss-limit', 'Daily Loss Limit', '🛑', 'Hard daily-loss kill-switch with warning / cut-size / kill tiers', null],
    ['drawdown-throttle', 'DD Throttle',    '📉', 'Auto-shrink size by drawdown tier (1.0× → 0.10× by 20%+)', null],
    ['goal-tracker',    'Goal Tracker',     '🎯', 'Period return target + max DD + on-pace classifier', null],
    ['trade-plan-checklist', 'Plan Checklist', '📝', 'Pre-trade gate enforcer: thesis, stop, target, R, direction, risk %', 'NEW'],
    ['per-symbol-slippage', 'Symbol TCA',   '📋', 'Per-symbol slippage roll-up + execution grades', null],
    ['order-staleness', 'Stale Orders',     '⏳', 'Resting-order freshness gauge with cancel-candidate flags', null],
    ['open-type',       'Open Type',        '🌅', 'Dalton AMT first-hour structural classifier', null],
    ['market-profile',  'Market Profile',   '📊', 'TPO histogram with POC, value area, single prints', null],
    ['oi-change',       'OI Change',        '🔥', 'Options OI surge alerter — where positioning is building', null],
    ['pyramid',         'Pyramid',          '🔺', 'Pyramid-up / scale-in entry plan + avg-cost evolution', null],
    ['ha-reversal',     'HA Reversal',      '🕯️', 'Heikin-Ashi color-flip reversal detector (strong/weak)', null],
    ['three-bar-reversal', '3-Bar Reversal', '🔄', 'Classic key-reversal pattern (down/small/up or up/small/down)', null],
    ['range-expansion', 'Range Expansion',  '💥', 'Wide-range bar after compression — Raschke spring-uncoil signal', null],
    ['alligator',       'Alligator',        '🐊', 'Williams jaw/teeth/lips trend gauge (sleeping vs hunting)', null],
    ['demarker',        'DeMarker',         '🎚️', 'Tom DeMark overbought/oversold oscillator (0.7/0.3 cuts)', null],
    ['murrey-math',     'Murrey Math',      '🎼', 'Octave-grid S/R levels (0/8 to 8/8 + extensions)', null],
    ['demark-pivots',   'DeMark Pivots',    '📐', 'Tom DeMark 3-level pivot (X-base switches on close-vs-open)', null],
    ['cypher-pattern',  'Cypher Pattern',   '🔱', 'Oglesbee XABCD harmonic — BC overshoots A; AD = 0.786·XA', null],
    ['regime-detector', 'Regime Detector', '🌗',  '2-state Markov-switching on returns', null],
    ['regime-equity',   'Equity Regime',   '📈',  'Classify your equity curve: trending / volatile / choppy (OLS slope + R² + residual stdev)', 'NEW'],
    ['american-option', 'American Option', '🇺🇸', 'LSMC pricer + European reference + EE premium', null],
    ['fx-option',       'FX Option',       '💱',  'Garman-Kohlhagen pricer + 6 greeks', null],
    ['forward-vol',     'Forward Vol',     '📏',  'Bootstrap forward vols from IV term structure', null],
    ['yield-curve-pca', 'Yield Curve PCA', '📈',  'Level / Slope / Curvature factor decomposition', null],
    ['dividend-calendar', 'Dividend Calendar', '💰', 'Upcoming ex-dates + indicated yields for symbol list', null],
    ['signal-decomposition', 'Signal Decomp', '🪞', 'EMD / Wavelet / SSA series decomposition', null],
    ['rr-butterfly',    'RR / BF Calc',    '🦋',  '25Δ risk-reversal & butterfly quote converter', null],
    ['microprice',      'Microprice',      '🪙',  'Stoikov quote-imbalance fair mid (L1 calculator)', null],
    ['dtw',             'DTW',             '↔',   'Dynamic time warping between 2 series', null],
    ['hurst',           'Hurst Exponent',  '∞',   'R/S long-memory / mean-reversion detector', null],
    ['bocpd',           'Change Points',   '🪓',  'Bayesian online change-point detector', null],
    ['vasicek',         'Vasicek Rates',   '🏦',  'Vasicek short-rate Monte Carlo simulator', null],
    ['kalman-beta',     'Kalman β',        '🎚',  'Time-varying pair hedge-ratio (Kalman filter)', null],
    ['pair-trade-calc', 'Pair Trade Calc', '🧪',  'Z-score + signal calculator from pasted prices', null],
    ['iv-solver',       'IV Solver',       '🔍',  'Implied vol from option market price (Newton-Raphson)', null],
    ['greeks-profile',  'Greeks Profile',  '🇬',  'Δ Γ Vega Θ ρ across a spot-price grid', null],
    ['second-order-greeks', '2nd-Order Greeks', '🇸', 'Vanna · Charm · Vomma · Veta — greeks of greeks', null],
    ['earnings-cal',    'Earnings Cal',    '📅',  'Earnings calendar w/ alerts',      null],
    ['earnings-iv',     'Earnings IV',     '💥',  'Pre-ER IV crush + straddle EV',    null],
    ['disclosures',     'Disclosures',     '📑',  'Form 4, 13D/G, 8-K stream',        null],
    ['economy',         'Economy',         '🏛',  'Economic calendar + macro',        null],
    ['news',            'News',            '📰',  'Per-symbol news + sentiment',      null],
    ['crypto',          'Crypto',          '🪙',  'Crypto markets snapshot',          null],

    // — Reports & analytics ——————————————————————————————————
    ['dashboard',       'Dashboard',       '🏠',  'Overview + equity + world markets', null],
    ['dashboards',      'My Dashboards',   '🗂️',  'Custom grids — compose any views into saved layouts', 'NEW'],
    ['reports',         'Reports',         '📊',  '17 TraderVue-style reports',       null],
    ['r-dist',          'R-Multiple',      '📐',  'R distribution histogram',         null],
    ['forecast',        'Equity Forecast', '🔮',  'Monte Carlo equity projection',    null],
    ['fill-quality',    'Fill Quality',    '🎯',  'TCA — slippage vs NBBO',           null],
    ['risk',            'Risk',            '🛟',  'Greeks / Beta / VaR',              null],
    ['rebalance',       'Rebalance',       '⚖️',  'Portfolio rebalance helper',       null],
    ['tax-lots',        'Tax Lots',        '💸',  'Lot-by-lot cost basis',            null],
    ['expenses',        'Expenses',        '🧾',  'Trading expenses tracking',        null],
    ['tax-workshop',    'Tax Workshop',    '💰',  'SE tax · home office · mileage · 1040-ES · subscriptions', null],
    ['tax-loss-harvest', 'Tax-Loss Harvest', '🧾', 'Year-end harvest suggester — ranks losers, flags wash-sale + $3k cap', 'NEW'],
    ['wash-sale',       'Wash Sale',       '🚨',  '§1091 detector — losing close + ±30-day replacement buy → disallowed-loss estimate', 'NEW'],
    ['buying-power',    'Buying Power',    '💪',  'Cash / Reg-T / portfolio-margin sizing — PDT 4× + sub-$5 rule + max notional/shares', 'NEW'],
    ['margin-call',     'Margin Call Dist', '🔻', 'Account-level dollar cushion before margin call — LMV vs trigger line', 'NEW'],
    ['vix-term-structure', 'VIX Term Structure', '📐', 'Contango / backwardation classifier from 5-point VIX curve (9D/30/3M/6M/1Y)', 'NEW'],
    ['currency-exposure', 'Currency Exposure', '💱', 'Aggregate positions by underlying CCY; flag overweight (>25%) vs home', 'NEW'],
    ['bond-duration',   'Bond Duration',   '📜',  'Macaulay + Modified duration calculator; ±Δy price sensitivity grid', 'NEW'],
    ['carry-score',     'Carry Score',     '🪙',  'FX carry attractiveness: (long − funding) / vol — strong / okay / poor / negative', 'NEW'],
    ['yield-curve',     'Yield Curve',     '🌊',  'UST 3M/2Y/5Y/10Y/30Y shape classifier: normal / flat / inverted / humped', 'NEW'],
    ['cost-basis',      'Cost Basis',      '📐',  'FIFO / LIFO / HIFO / LOFO lot selection + tax-optimal recommender', 'NEW'],
    ['calendar',        'Calendar',        '📆',  'Yearly P&L heatmap',               null],
    ['accounts-overview', 'Accounts Overview', '🗂', 'All accounts combined',        null],

    // — Strategy / automation ————————————————————————————————
    ['backtest',        'Backtest',        '🧷',  'stryke-JIT strategy backtester',   null],
    ['backtest-presets','BT Presets',      '📦',  'Saved backtest configs',           null],
    ['walk-forward',    'Walk-forward',    '🧱',  'Walk-forward optimization',        null],
    ['custom-indicators','Indicators',     '∇',   'Custom indicator editor',          null],
    ['strategy-alerts', 'Strategy Alerts', '🔔',  'Strategy-trigger alerts',          null],
    ['alerts',          'Alerts',          '🚨',  'Price / signal alert rules',       null],
    ['webhooks',        'Webhooks',        '🪝',  'Outbound webhook hub',             null],

    // — Community / sharing ————————————————————————————————
    ['shares',          'Shares',          '🔗',  'Public trade share links',         null],
    ['community',       'Community',       '💬',  'Forum + threads',                  null],
    ['mentorship',      'Mentorship',      '🎓',  'Mentor / mentee relationships',    null],
    ['boards',          'Boards',          '📌',  'Public/private symbol boards',     null],

    // — Admin / data ———————————————————————————————————————
    ['import',          'Import',          '⤴',  '12 broker CSV importers',          null],
    ['csv-wizard',      'CSV Wizard',      '🪄',  'Custom column-map importer',       null],
    ['exports',         'Exports',         '⤵',  'CSV / JSON / Schedule D export',   null],
    ['accounts',        'Accounts',        '🏦',  'Add / remove broker accounts',     null],
    ['tags',            'Tags',            '🏷',  'Tag management',                   null],
    ['search',          'Search',          '🔍',  'Full-text across trades/journal',  null],
    ['settings',        'Settings',        '⚙️',  'Profile, commissions, templates',  null],
    ['developer',       'Developer',       '🔧',  'API tokens + webhook tester',      null],
    ['tutorial',        'Tutorial',        '❓',  'In-app guide',                     null],
    ['keyboard-shortcuts', 'Shortcuts',    '⌨️',  'Cheat sheet of every keyboard shortcut (also `?` hotkey)', 'NEW'],
];

export const CATEGORIES = [
    ['live',     '// LIVE MARKETS',     ['live-scanner','halts','catalysts','premarket','tape','heatmap']],
    ['trading',  '// TRADING',          ['dashboards','webull','live','paper','new-trade','plans','sizing','optimal-f','kelly','mc-trades','margin-runway','risk-reward','buying-power','margin-call','risk-gate','hotkeys','pyramid','news-event','stop-loss-best-of','stop-loss-backtest','futures-roll','squeeze-alerts','alert-rules','daily-loss-limit','drawdown-throttle','goal-tracker','trade-plan-checklist','time-in-force']],
    ['journal',  '// JOURNAL',          ['journal','ai','reviews','trade-compare','replay','tape-replay','discipline','mood','goals','clusters-trade-features','setups-by-setup']],
    ['research', '// CHARTS & RESEARCH',['charts','research','watchlists','screener','scanners','top-signals','compare','pairs','correlation','sectors','sector-rotation','breadth','fear-greed','sentiment','cohort-tilt','risk-on-off','darkpool','short-interest','vol','vol-surface','vix-term-structure','options','option-payoff','vol-smile','monte-carlo','series-smoother','pattern-discovery','regime-detector','regime-equity','american-option','fx-option','forward-vol','yield-curve-pca','yield-curve','bond-duration','carry-score','dividend-calendar','signal-decomposition','rr-butterfly','microprice','dtw','hurst','bocpd','vasicek','kalman-beta','pair-trade-calc','iv-solver','iv-rank','iv-backtest','greeks-profile','second-order-greeks','vpin','cup-and-handle','order-book-imbalance','cusum','order-flow','footprint','stress-test','chandelier-stop','vol-stop-close','atr-cone','round-levels','kyles-lambda','hawkes','kagi','three-line-break','volume-at-price','roll-spread','effective-spread','weighted-midprice','range-bar','tick-bar','volume-bar','dollar-bar','equivolume','imbalance-bar','adf-test','aroon','amihud','breadth-thrust','bb-squeeze','balance-of-power','anchored-momentum','acf','beta','brier-score','bipower-variation','bootstrap-pnl','block-bootstrap','ad-normality','arch-lm','alma','alphatrend','atr-channel','atr-trailing-stop','adl','asi','ad-oscillator','beta-shrinkage','bartlett-variance','bid-ask-volume-ratio','bollinger-band-width','bollinger-bandwidth-percentile','bollinger-percent-b','bollinger-band-distance','bollinger-oscillators','borrow-rate-indicator','breusch-pagan','burke-ratio','camarilla-pivots','breusch-godfrey','candle-strength-index','carhart-4','centered-smoothed-momentum','chaikin-oscillator','chande-dynamic-momentum','chande-kroll-stop','chande-momentum-oscillator','chande-trend-index','chande-volatility-index','chandelier-exit','cholesky','abc-pattern','absorption','favorites','triple-screen','open-type','market-profile','oi-change','ha-reversal','three-bar-reversal','range-expansion','alligator','choppiness','demarker','murrey-math','demark-pivots','cypher-pattern','earnings-cal','earnings-iv','disclosures','economy','news','crypto']],
    ['reports',  '// REPORTS',          ['dashboard','reports','r-dist','forecast','fill-quality','risk','rebalance','tax-lots','cost-basis','expenses','tax-workshop','tax-loss-harvest','wash-sale','calendar','accounts-overview','portfolio-allocator','risk-parity','risk-parity-solver','herfindahl','momentum-crash','marginal-var','active-share','brinson','black-litterman','currency-exposure','cov-denoiser','var-calculator','var-estimator','execution-scheduler','almgren-chriss','implementation-shortfall','deflated-sharpe','market-impact','liquidity','spread-tracker','intraday-heatmap','heatmap-dow-hour','vwap-slippage','twap','per-symbol-slippage','order-staleness','clusters-correlation','commission-optimizer']],
    ['strategy', '// STRATEGY & AUTOMATION', ['backtest','backtest-presets','walk-forward','custom-indicators','strategy-alerts','alerts','webhooks']],
    ['community','// COMMUNITY',        ['shares','community','mentorship','boards']],
    ['admin',    '// ADMIN & DATA',     ['import','csv-wizard','exports','accounts','tags','search','settings','developer','tutorial','keyboard-shortcuts']],
];

let lastQuery = '';
let _wired = false;

export async function renderLauncher(mount, _state) {
    if (!_wired) {
        _wired = true;
        // Repaint when favorites or bookmarks change anywhere in the app
        // (Cmd+D star toggle, Cmd+B bookmark, manager view, etc.) so the
        // tile stars and any future favorite-aware UI stay live.
        window.addEventListener('tv:favorites-changed', () => {
            if ((window.location.hash || '').replace(/^#/, '').split('/')[0] === 'launcher') {
                renderGrid();
            }
        });
    }
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.launcher.title">// HOME</span>
            <input id="launcher-q" type="search"
                   placeholder="filter tiles…  (Esc to clear)"
                   data-i18n-placeholder="view.launcher.search_placeholder"
                   data-tip="view.launcher.search_tip"
                   autocomplete="off">
        </h1>
        <div id="launcher-grid"></div>
    `;
    const q = document.getElementById('launcher-q');
    q.value = lastQuery;
    q.focus();
    q.addEventListener('input', () => { lastQuery = q.value; renderGrid(); });
    q.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') { q.value = ''; lastQuery = ''; renderGrid(); }
        if (e.key === 'Enter') {
            // Jump to first visible tile.
            const first = document.querySelector('.tile[data-view]');
            if (first) go(first.dataset.view);
        }
    });
    renderGrid();
}

function renderGrid() {
    const q = (lastQuery || '').trim().toLowerCase();
    const byId = new Map(TILES.map(t => [t[0], t]));
    const grid = document.getElementById('launcher-grid');
    if (!grid) return;

    // Recents block — newest at front, filtered by query, capped to MAX_RECENTS.
    let recentSection = '';
    try {
        const r = recents.loadState();
        const recentTiles = recents.listRecents(r)
            .map(rc => byId.get(rc.viewId))
            .filter(Boolean)
            .filter(tt => !q || matchesQuery(tt, q));
        if (recentTiles.length) {
            const heading = t('view.launcher.section.recent') || '// RECENT';
            recentSection = `<section class="launcher-cat" data-context-scope="launcher-recents">
                <h2>${esc(heading)}</h2>
                <div class="launcher-tiles">${recentTiles.map(renderTile).join('')}</div>
            </section>`;
        }
    } catch (_) { /* recents storage unavailable; fine */ }

    grid.innerHTML = recentSection + CATEGORIES.map(([cat, label, ids]) => {
        const tiles = ids
            .map(id => byId.get(id))
            .filter(Boolean)
            .filter(t => !q || matchesQuery(t, q));
        if (!tiles.length) return '';
        // Translate the category header — tile.<id>.label-style key with
        // English literal fallback when key is missing.
        const catKey = `view.launcher.category.${cat}`;
        const catLabel = (() => { const v = t(catKey); return (v && v !== catKey) ? v : label; })();
        return `<section class="launcher-cat">
            <h2>${esc(catLabel)}</h2>
            <div class="launcher-tiles">${tiles.map(renderTile).join('')}</div>
        </section>`;
    }).join('');

    // Tile click navigates; favorites + pin sub-buttons stop propagation.
    grid.querySelectorAll('.tile[data-view]').forEach(el => {
        el.addEventListener('click', (e) => {
            if (e.target instanceof HTMLElement && e.target.closest('.tile-action')) return;
            go(el.dataset.view);
        });
        el.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                go(el.dataset.view);
            }
        });
    });
    grid.querySelectorAll('.tile-action[data-fav]').forEach(btn => {
        btn.addEventListener('click', (e) => {
            e.stopPropagation();
            const id = btn.dataset.fav;
            let f = favs.loadState();
            f = favs.toggleFavorite(f, id);
            favs.saveState(f);
            renderGrid();
            window.dispatchEvent(new CustomEvent('tv:favorites-changed'));
        });
    });
    grid.querySelectorAll('.tile-action[data-pin]').forEach(btn => {
        btn.addEventListener('click', (e) => {
            e.stopPropagation();
            const id = btn.dataset.pin;
            void (async () => {
                const toast = await import('../toast.js');
                let s = dashStore.loadState();
                const active = dashStore.getActiveDashboard(s);
                if (!active) {
                    toast.showToast(t('toast.no_active_dashboard'), { level: 'warning' });
                    return;
                }
                s = dashStore.addTile(s, active.id, id);
                dashStore.saveState(s);
                btn.textContent = '✓';
                setTimeout(() => { btn.textContent = '📌'; }, 1200);
                toast.showToast(
                    t('toast.tile_pinned', { view: id, dashboard: active.name || active.id }),
                    { level: 'success' });
                window.dispatchEvent(new CustomEvent('tv:dashboards-changed'));
            })();
        });
    });
    // Re-translate + upgrade tooltips on the freshly-rendered grid so
    // toggle-induced rebuilds (favorite star, recents repaint) carry
    // the same a11y / hover-help surface as the initial dispatch.
    try { applyUiI18n(grid); } catch (_) {}
    try { upgradeTooltips(grid); } catch (_) {}
}

function renderTile([id, label, glyph, desc, badge]) {
    const fState = favs.loadState();
    const fav = favs.isFavorite(fState, id);
    // Convention: tile.<id>.label / tile.<id>.desc translate the literal
    // strings in TILES. Missing keys fall back to the English literal so
    // partial locale catalogs degrade cleanly.
    const labelKey = `tile.${id}.label`;
    const descKey  = `tile.${id}.desc`;
    const tLabel = (() => { const v = t(labelKey); return (v && v !== labelKey) ? v : label; })();
    const tDesc  = (() => { const v = t(descKey);  return (v && v !== descKey)  ? v : desc;  })();
    return `<button class="tile" data-view="${esc(id)}" tabindex="0">
        <span class="tile-glyph">${glyph}</span>
        <span class="tile-body">
            <span class="tile-label"><span data-i18n="${esc(labelKey)}">${esc(tLabel)}</span>${badge ? ` <span class="tile-badge">${esc(badge)}</span>` : ''}</span>
            <span class="tile-desc" data-i18n="${esc(descKey)}">${esc(tDesc)}</span>
        </span>
        <span class="tile-actions">
            <span class="tile-action ${fav ? 'tile-fav-on' : 'tile-fav-off'}" data-fav="${esc(id)}"
                  data-tip="${fav ? 'view.launcher.tile.unfavorite' : 'view.launcher.tile.favorite'}"
                  data-i18n-aria-label="${fav ? 'view.launcher.tile.unfavorite' : 'view.launcher.tile.favorite'}">${fav ? '★' : '☆'}</span>
            <span class="tile-action" data-pin="${esc(id)}"
                  data-tip="view.launcher.tile.pin"
                  data-i18n-aria-label="view.launcher.tile.pin">📌</span>
        </span>
    </button>`;
}
