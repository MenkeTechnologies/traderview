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
import { showToast } from '../toast.js';
import { initDragReorder } from '../drag_reorder.js';

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
    ['after-hours',  'After-hours',   '🌙',  'PRE + POST movers vs RTH close — TTS at ±5%', 'LIVE'],
    ['catalyst-correlations', 'Catalyst Impact', '🎯', 'Catalysts that produced ≥2% move within 60s — sentiment-scored', 'NEW'],
    ['uoa-stream',   'UOA Stream',    '🎰',  'Live unusual options activity — rotating poll of top-20 movers', 'NEW'],
    ['gamma-squeeze', 'Gamma Squeeze', '💥',  'Negative dealer GEX ≥ $250M + spot within 2% of pin strike', 'NEW'],
    ['htb-ranker',   'HTB Ranker',    '🔒',  'Squeeze-pressure score = % float + DTC + MoM change + inverse float', 'NEW'],
    ['breadth-divergence', 'Breadth Divergence', '⚖️', 'SPY vs market breadth — bullish or bearish non-confirmation', 'NEW'],
    ['rvol-accel',   'RVOL Accel',    '🚀',  'Per-minute volume acceleration — 3 strictly-rising bars + ≥5× baseline', 'NEW'],
    ['insider-stream', 'Insider Form 4', '👁️', 'Real-time SEC Form 4 — parses insider buys/sells from XML, ranks cluster buys', 'NEW'],
    ['insider-clusters', 'Insider Clusters', '👥', 'Cohen Malloy Pomorski 2012 — clusters of ≥3 distinct insider buys in 30d', 'NEW'],
    ['earnings-revisions', 'Earnings Revisions', '✏️', 'Analyst EPS revision velocity — 30d / 90d / accelerating composite per Womack 1996', 'NEW'],
    ['sector-timing', 'Sector Timing', '🔄', 'Sector RS rotation timing — MA crossover + slope accel + breakout proximity', 'NEW'],
    ['market-gamma', 'Market Gamma',  '🌐',  'SPY total dealer GEX regime — positive=mean-revert / negative=amplify / flip moments', 'NEW'],
    ['scanner-backtest', 'Scanner Backtest', '📑', 'Backtest each scanner against historical bars — Sharpe, hit rate, max DD per horizon', 'NEW'],
    ['confluence-autotrade', 'Confluence Autotrade', '🤖', 'Auto-submit paper-market buys when a symbol crosses your confluence-score gate', 'NEW'],
    ['portfolio-exposure', 'Portfolio Exposure', '🧭', 'Total β to SPY, sector concentration, single-name HHI, parametric 1-day 95% VaR', 'NEW'],
    ['dividend-tracker', 'Dividend Tracker', '💵', 'Price + DRIP total return per position, yield-on-cost, forward 12-month income estimate', 'NEW'],
    ['magic-formula', 'Magic Formula', '🪄', 'Greenblatt value scorer: rank S&P by combined (EBIT/EV + ROIC) ranking', 'NEW'],
    ['paper-rebalance', 'Paper Rebalancer', '⚖️', 'Named target weight sets, drift detection, suggested rebalance trades for the paper account', 'NEW'],
    ['paper-tax-loss-harvest', 'Paper TLH', '📉', 'Paper account tax-loss harvest: find losses ≥ threshold, suggest non-substantially-identical replacement, flag wash-sale risk', 'NEW'],
    ['sector-rotation-strategy', 'Sector Rotation', '🔄', 'Faber-style monthly sector momentum rotation — Sharpe, CI, max DD on 11 sector ETFs', 'NEW'],
    ['dca-simulator', 'DCA Simulator', '📅', 'Dollar-cost-averaging scheduler with lump-sum comparison — does buying $N/month beat lump-summing?', 'NEW'],
    ['dividend-aristocrats', 'Aristocrats', '🎖️', '67 S&P 500 Dividend Aristocrats + Kings ranked by composite DGI score (yield + growth - payout penalty)', 'NEW'],
    ['permanent-portfolio', 'Permanent Portfolio', '🪨', 'All-Weather + Permanent Portfolio + 60/40 + 100% S&P backtested side-by-side — return, vol, Sharpe CI, max DD', 'NEW'],
    ['cape-indicator', 'CAPE / Shiller P/E', '📊', 'Shiller CAPE ratio with percentile rank in 1881-2024 historical distribution + regime interpretation', 'NEW'],
    ['fire-calculator', 'FIRE Calculator', '🔥', 'Years-to-target, required savings, year-by-year projection, return×contribution sensitivity table', 'NEW'],
    ['emergency-fund', 'Emergency Fund', '🛟', 'Months covered + 3/6/9/12-month target gaps + months-to-target at your contribution rate', 'NEW'],
    ['savings-waterfall', 'Savings Waterfall', '🪣', 'Financial order of operations — where the next dollar goes: match, debt, emergency, HSA/Roth, max retirement, then taxable', 'NEW'],
    ['net-worth-tracker', 'Net Worth Tracker', '📒', 'Assets − Liabilities across categories + M/M + Y/Y delta + debt-to-asset %', 'NEW'],
    ['personal-balance-sheet', 'Balance Sheet', '⚖️', 'GAAP-style current/non-current split + equity + working capital + current/quick ratios + D/E', 'NEW'],
    ['personal-cash-flow', 'Cash Flow Statement', '💵', 'GAAP operating / investing / financing split + net change in cash + savings rate', 'NEW'],
    ['financial-ratios', 'Financial Ratios', '📊', '7 CFP/Bogleheads ratios (savings, DTI, 28/36, liquidity, solvency, EF, retirement) + composite score', 'NEW'],
    ['savings-rate', 'Savings Rate / FI', '🎯', 'MMM math: years to FI from savings rate at 5% real + 4% SWR, FI number, 10-70% sensitivity', 'NEW'],
    ['sinking-fund', 'Sinking Fund', '🪣', 'Multi-goal monthly allocator — required/mo, months-to-target, shortfall, on-track per goal + aggregate', 'NEW'],
    ['zero-based-budget', 'Zero-Based Budget', '🟰', 'Dave Ramsey / YNAB — every dollar assigned, leftover = 0, per-category planned vs actual variance', 'NEW'],
    ['fifty-thirty-twenty', '50/30/20 Rule', '🥧', 'Warren 50/30/20 needs/wants/savings — bucket actual vs ideal $ + delta + status', 'NEW'],
    ['envelope-budget', 'Envelope Budget', '✉️', 'Digital cash-envelope method — per-envelope ok/warning/empty + rollover vs reset', 'NEW'],
    ['debt-avalanche', 'Debt Avalanche', '💳', 'Highest-APR-first payoff sim — optimal for total interest minimisation; rolls minimums on payoff', 'NEW'],
    ['debt-snowball', 'Debt Snowball', '⛄', 'Dave Ramsey smallest-balance-first payoff — adherence wins from psychological early payoffs', 'NEW'],
    ['credit-utilization', 'Credit Utilization', '💳', 'Per-card + aggregate util % vs FICO thresholds (10/30%) + paydown needed to land under 30%', 'NEW'],
    ['auto-loan', 'Auto Loan', '🚗', 'Vehicle price + down + trade + tax + APR + term → monthly P+I + total interest + full schedule', 'NEW'],
    ['mortgage-amortization', 'Mortgage', '🏠', 'PITI + PMI (LTV > 80) + extra-payment what-if — months saved + interest saved + schedule head', 'NEW'],
    ['mortgage-refinance', 'Mortgage Refi', '🔄', 'Closing costs / monthly savings breakeven — refi wins vs breakeven too long vs no savings', 'NEW'],
    ['rent-vs-buy', 'Rent vs Buy', '⚖️', 'NYT-style year-by-year NPV — breakeven year + net winner across N-year horizon', 'NEW'],
    ['heloc', 'HELOC', '🏘️', 'Variable-rate revolving home-equity line — draw + repayment phases, utilization, lifetime interest', 'NEW'],
    ['home-maintenance', 'Home Maintenance', '🛠️', '1% rule + per-system replacement schedule with monthly set-aside and overdue flags', 'NEW'],
    ['student-loan-payoff', 'Student Loan', '🎓', 'Compare Standard 10y vs IBR vs PAYE vs SAVE — monthly + total paid + months + forgiven balance', 'NEW'],
    ['pslf-tracker', 'PSLF Tracker', '🏛️', '120 qualifying payments → tax-free federal loan forgiveness for public-service / 501(c)(3) employees', 'NEW'],
    ['college-529', '529 College', '🎒', '4-year college cost inflated to enrollment + monthly contribution needed at expected return', 'NEW'],
    ['fafsa-efc', 'FAFSA SAI', '📋', 'Simplified Student Aid Index estimator — parent+student income+asset graduated contributions', 'NEW'],
    ['car-tco', 'Car TCO', '🚙', 'Total cost of ownership — depreciation + fuel + insurance + maintenance + financing + cost-per-mile', 'NEW'],
    ['lease-vs-buy-car', 'Lease vs Buy Car', '🔁', 'NPV lease vs buy over N-year horizon with breakeven monthly lease payment', 'NEW'],
    ['ev-vs-ice', 'EV vs ICE', '⚡', 'EV vs gas total cost — electricity vs fuel, EV credits, maintenance delta, battery replacement', 'NEW'],
    ['coast-fire', 'Coast FIRE', '🏖️', 'NW needed today to compound to FI with NO more contributions by target retirement age', 'NEW'],
    ['barista-fire', 'Barista FIRE', '☕', 'Portfolio covers the gap between expenses and part-time income — smaller FI number', 'NEW'],
    ['lean-fire', 'Lean FIRE', '🥗', 'Minimalist FIRE (≤ $40k/yr expenses) — FI number + years to target + expense tier classification', 'NEW'],
    ['fat-fire', 'Fat FIRE', '🍷', 'High-spend FIRE (≥ $100k/yr expenses) — conservative 3.5% SWR, bigger target, longer timeline', 'NEW'],
    ['rmd-calculator', 'RMD Calculator', '📅', 'IRS Uniform Lifetime Table — current RMD + N-year projection, SECURE 2.0 age 73/75 start', 'NEW'],
    ['social-security-age', 'Social Security Age', '🧓', 'Claim 62 vs FRA vs 70 — SSA reduction/DRC + lifetime totals + breakeven age', 'NEW'],
    ['roth-vs-trad-401k', 'Roth vs Trad 401k', '💸', 'Apples-to-apples Roth vs Traditional with side-account tax-savings model + breakeven retire rate', 'NEW'],
    ['pension-lump-vs-annuity', 'Pension Lump vs Annuity', '🏦', 'PV(annuity) vs lump + implied IRR + runs-out year + leftover at life expectancy', 'NEW'],
    ['three-fund-portfolio', 'Three-Fund Portfolio', '🧮', 'Boglehead US + Intl + Bonds allocation by age + risk tolerance + drift + rebalance', 'NEW'],
    ['bond-tent', 'Bond Tent', '⛺', 'Kitces/Pfau rising-equity glide — ramp bonds up into retirement to dampen SORR', 'NEW'],
    ['glide-path', 'Glide Path', '🛬', 'Target-Date-Fund-style 3-phase linear stock/bond glide', 'NEW'],
    ['annuity-pv-fv', 'Annuity PV / FV', '🪙', 'Time-value annuity: PV + FV + total payments + interest; ordinary or due', 'NEW'],
    ['cd-ladder', 'CD Ladder', '🪜', 'N-rung Certificate of Deposit ladder + blended APY + per-rung maturity schedule', 'NEW'],
    ['i-bond', 'I-Bond', '💵', 'Series I savings bond — composite rate + holding rules + 3mo penalty in 1-5y window', 'NEW'],
    ['tips-bond', 'TIPS Bond', '🛡', 'Treasury Inflation-Protected Securities — semi-annual CPI accretion + real coupons + deflation floor', 'NEW'],
    ['hysa-compare', 'HYSA Compare', '🏧', 'Per-bank effective APY (with fees + min-balance) + winner by net gain', 'NEW'],
    ['tax-bracket-optimizer', 'Tax Brackets', '🧾', '2026 IRS marginal brackets — current bracket + room remaining + effective rate', 'NEW'],
    ['drawdown-cutoff', 'Drawdown Cutoff', '🛑', 'Auto-fire kill-switch when live broker equity drops below your max-drawdown threshold', 'NEW'],
    ['inflation-calculator', 'Inflation Calculator', '💵', 'Future-dollar / real-value calculator — preserves purchasing power across N-year horizon at target CPI rate', 'NEW'],
    ['lump-sum-vs-dca', 'Lump-Sum vs DCA', '⚖️', 'Vanguard 2012 comparison — lump-sum end value vs DCA cadence end value + break-even market return', 'NEW'],
    ['bill-calendar', 'Bill Calendar', '🗓️', 'Recurring bill due-dates — 12-month projected timeline, monthly equivalent, annual roll-up; persists locally', 'NEW'],
    ['cash-flow-forecast', 'Cash-Flow Forecast', '📈', 'Bank balance projection N months out — income + bills + one-offs + cash crunch warning', 'NEW'],
    ['income-tax-estimator', 'Income Tax Estimator', '🧾', 'TY2025 IRS brackets — ordinary + LTCG stacked + FICA + Add\'l Medicare; effective + marginal breakdown', 'NEW'],
    ['compound-interest', 'Compound Interest', '🔁', 'PV + PMT → FV with daily/weekly/monthly/quarterly/annual compounding + contribution escalator', 'NEW'],
    ['time-value-money', 'Time Value of Money', '🧮', 'Generic financial solver — solve for FV / PV / PMT / N / R given the other four (HP-12C / TI-BA-II engine)', 'NEW'],
    ['roth-conversion-ladder', 'Roth Conversion Ladder', '🪜', 'Early-retirement bridge — laddered Trad→Roth conversions with 5yr season-out + per-year tax cost + access age', 'NEW'],
    ['mortgage-payoff-vs-invest', 'Payoff vs Invest', '⚖️', 'Extra cash/mo against mortgage vs invested in market — net wealth at horizon + after-tax rate adjustment', 'NEW'],
    ['ibond-calculator', 'I-Bonds', '🪙', 'TreasuryDirect Series I — composite fixed + inflation, 1yr lockup, 3mo penalty, recent rate history', 'NEW'],
    ['bond-ladder', 'Bond Ladder', '🪜', 'Staggered Treasury / CD / muni ladder — per-rung yield + after-tax income, tax-equivalent yield for muni mode', 'NEW'],
    ['ltcg-harvesting', 'LTCG Harvesting', '🌾', '0% bracket fill calculator — free basis step-up via long-term cap gain harvest + immediate repurchase (no wash-sale on gains)', 'NEW'],
    ['sequence-of-returns', 'Sequence Risk', '📉', 'Bengen 4% pressure-tested across forward / reversed / worst-first / best-first orderings of the same return sequence', 'NEW'],
    ['rule-of-72', 'Rule of 72', '⏳', 'Doubling / tripling / quadrupling time shortcuts vs exact closed-form — error %, $-becomes value', 'NEW'],
    ['goal-funding', 'Goal Funding', '🎯', 'Required monthly $ to reach $X in N years + lump-sum equivalent + rate×horizon sensitivity grid', 'NEW'],
    ['reverse-mortgage', 'Reverse Mortgage', '🏠', 'HECM calculator — PLF, MIP, origination fees, lump/tenure/line payouts, balance-vs-home-value timeline', 'NEW'],
    ['niit-calculator', 'NIIT 3.8%', '🩺', 'Net Investment Income Tax surtax — investment income vs MAGI threshold breakdown + per-component detail', 'NEW'],
    ['drip-simulator', 'DRIP Simulator', '🔁', 'Dividend reinvestment compounding vs cash dividends — year-by-year share accumulation + CAGR comparison', 'NEW'],
    ['vertical-spread', 'Vertical Spread', '↕️', 'Bull call / bear put / bull put / bear call — max profit, max loss, breakeven, R/R, return on collateral', 'NEW'],
    ['iron-condor', 'Iron Condor', '🦅', 'Sell OTM put spread + sell OTM call spread — profit zone, collateral, return %, P&L curve across spot', 'NEW'],
    ['stretch-ira', 'Stretch IRA', '🧬', 'SECURE Act 10-year rule for inherited IRAs — 4 distribution strategies compared by after-tax received', 'NEW'],
    ['tips-breakeven', 'TIPS Breakeven', '📊', 'Nominal − TIPS yield = market-implied inflation; year-N terminal-value comparison vs your CPI forecast', 'NEW'],
    ['yield-to-call', 'Yield to Call', '📞', 'YTC vs YTM vs YTW for callable bonds — premium/discount risk + cash flow trace if called', 'NEW'],
    ['covered-call', 'Covered Call', '📞', 'Write OTM calls against long stock — premium yield, breakeven cushion, scenario P&L, opportunity cost', 'NEW'],
    ['real-estate-cap-rate', 'Rental Underwriting', '🏘️', 'Cap rate, cash-on-cash, 1% rule, GRM, DSCR + full P&L breakdown for rental property analysis', 'NEW'],
    ['house-hacking', 'House Hacking', '🏠', 'Live in one unit, rent the rest — net housing cost, savings vs renting, and cash flow once you move out', 'NEW'],
    ['live-dashboard', 'Live Dashboard', '📡', 'Live broker state — equity, BP, positions, orders. Pulls directly from Alpaca every 15s. No CSV, no closed-trade dependency.', 'LIVE'],
    ['pead',         'PEAD Drift',    '📈',  'Post-earnings drift — surprise + 5/20/60d returns + drift score', 'NEW'],
    ['sentiment-velocity', 'Sentiment Velocity', '🔊', 'WSB+StockTwits mention acceleration — ≥3× hour-over-hour + ≥2 consecutive', 'NEW'],
    ['confluence',   'Confluence',    '🎯',  'Meta-scanner — symbols hitting the most independent edges, weighted', 'TOP'],
    ['vrp',          'VRP Scanner',   '📐',  'Vol risk premium: IV÷RV ranking — overpriced vs underpriced option premium', 'NEW'],
    ['pairs-coint',  'Pairs Cointeg', '🔗',  'Stat-arb pairs: OLS β + AR(1) ρ + half-life + z-score, |z|≥2 entry', 'NEW'],
    ['ipo-lockups',  'IPO Lockups',   '🔓',  'Forced supply pressure — IPO+180d lockup expirations in next 60d', 'NEW'],
    ['iv-term',      'IV Term',       '📊',  'IV term-structure inversion — calendar-spread candidates ranked by front-back spread', 'NEW'],
    ['sp500-predict','S&P 500 Predict', '🏆','S&P 500 inclusion predictor — score symbols against methodology criteria', 'NEW'],
    ['dividend-capture','Dividend Capture','💰','Dividend capture / arb — ranks long-capture vs short-arb edge per name', 'NEW'],
    ['multi-broker', 'Multi-Broker',  '🔀',  'Positions aggregated across alpaca/tradier (ibkr/schwab/tasty pending) + kill-switch', 'NEW'],
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
    ['multichart',      'Multi-Chart',     '📊',  '4-up multi-timeframe grid — shared symbol, per-pane indicators', 'NEW'],
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
    ['squeeze-scanner', 'Squeeze Scanner',  '⚡', 'LIVE catalyst-driven squeeze detector — WS-fed by SEC + halts + PR; alerts to the second', 'NEW'],
    ['ipo-calendar',    'IPO Calendar',     '🚀', 'Upcoming IPOs via Finnhub — low-float runners on IPO day are classic squeeze candidates', 'NEW'],
    ['top-news',        'Top News',         '🗞️', 'Broad market news strip — general / forex / crypto / M&A categories', 'NEW'],
    ['finnhub-pattern',       'Pattern Recognition', '🔍', 'Finnhub /scan/pattern — head & shoulders, double tops, triangles, flags', 'NEW'],
    ['finnhub-sr',            'S/R Levels (Finnhub)', '📍', 'Finnhub /scan/support-resistance — algorithmic price levels by resolution', 'NEW'],
    ['finnhub-aggregate',     'Tech Aggregate',      '🎯', 'Finnhub composite buy/sell/neutral signal across MA + ADX + RSI + Stoch + MACD + CCI', 'NEW'],
    ['forex-rates',           'Forex Rates',         '💱', 'Real-time spot rates for any base currency', 'NEW'],
    ['economic-calendar',     'Economic Calendar',   '🏛', 'Macro events (CPI, NFP, FOMC, GDP) — Finnhub premium endpoint', 'NEW'],
    ['symbol-changes',        'Symbol Changes',      '🔁', 'Recent ticker renames + ISIN changes — audit your watchlists', 'NEW'],
    ['etf-profile',           'ETF Profile',         '🧺', 'ETF profile + holdings + sector + country mix', 'NEW'],
    ['lobbying',              'Lobbying',            '🏢', 'Per-symbol Senate lobbying disclosures + spend totals', 'NEW'],
    ['congressional-trading', 'Congressional Trades', '🏛', 'STOCK Act disclosures by US Senators / Reps in any ticker', 'NEW'],
    ['finnhub-search',        'Symbol Search',       '🔎', 'Universal ticker lookup across global exchanges', 'NEW'],
    ['fda-calendar',          'FDA Calendar',        '💊', 'Upcoming FDA advisory + PDUFA dates — biotech catalyst gold', 'NEW'],
    ['market-status',         'Market Status',       '🚦', 'Live exchange status + upcoming holidays / half-days', 'NEW'],
    ['index-constituents',    'Index Constituents',  '📋', 'S&P 500, Nasdaq 100, Dow, Russell, FTSE, Nikkei members with weights', 'NEW'],
    ['insider-finnhub',       'Insiders (Finnhub)',  '🕵', 'Form 4 insider buys/sells — Finnhub side-by-side w/ SEC EDGAR', 'NEW'],
    ['news-sentiment',        'News Sentiment',      '🧠', 'Bullish/bearish %, buzz score, sector-relative ranking', 'NEW'],
    ['price-target',          'Price Target',        '🎯', 'Wall Street PT consensus — high/median/low + implied upside vs current', 'NEW'],
    ['estimates-dashboard',   'Estimates Dashboard', '📊', '8 analyst estimates side-by-side: Revenue / EBITDA / EBIT / EPS / Net inc / Pretax / Gross / DPS', 'NEW'],
    ['crypto-markets',        'Crypto Markets',      '₿',  'Crypto exchanges + supported pairs + per-coin fundamentals (Finnhub)', 'NEW'],
    ['historical-market-cap', 'Mkt Cap History',     '📈', 'Market cap over time — spot dilution vs growth at a glance', 'NEW'],
    ['earnings-call-live',    'Earnings Call Live',  '🎙', "Today's earnings calls + per-symbol transcript history", 'NEW'],
    ['supply-chain',          'Supply Chain',        '🔗', 'Customers + suppliers per symbol — sympathy-play hunting ground', 'NEW'],
    ['esg',                   'ESG Scores',          '🌿', 'Environmental / Social / Governance scores — current + historical', 'NEW'],
    ['sector-heatmap',        'Sector Heatmap',      '🟦', 'Sector-level P/E, P/B, dividend yield, ROE, margin — rotation visualizer', 'NEW'],
    ['bond-yield-curve',      'Yield Curve',         '📐', 'Sovereign yield curves + inversion detector (US 2s10s recession signal)', 'NEW'],
    ['unusual-options',       'Unusual Options',     '⚡', 'Vol/OI &gt; 3× scanner — fresh institutional positioning signal', 'NEW'],
    ['subscriptions',         'Subscriptions',       '💳', 'Auto-detect recurring charges from your expense history — audit data-feed creep', 'NEW'],
    ['revenue-breakdown',     'Revenue Breakdown',   '🥧', 'Segment + geographic revenue mix over time — spots secular shifts', 'NEW'],
    ['earnings-quality',      'Earnings Quality',    '✅', 'Finnhub 1-10 score: profitability + growth + cash + capital + leverage', 'NEW'],
    ['quarterly-tax',         'Quarterly Tax',       '🧾', '1040-ES estimator: SE + fed income tax, safe-harbor floor, per-quarter target', 'NEW'],
    ['mileage-log',           'Mileage Log',         '🚗', 'Log business trips, compute IRS-rate deductible (67¢/mi 2024, 70¢/mi 2025)', 'NEW'],
    ['filings-browser',       'SEC Filings',         '📜', '10-K / 10-Q / 8-K / 13F / Form 4 filings browser with form-type filter', 'NEW'],
    ['insider-sentiment',     'Insider Sentiment',   '🌡', 'Monthly Share Purchase Ratio (MSPR) — net bullish vs net bearish trend chart', 'NEW'],
    ['institutional-13f',     '13F Portfolio',       '🏦', 'Hedge fund holdings (Berkshire, Renaissance, Bridgewater, Citadel, Ackman, Einhorn)', 'NEW'],
    ['section-179',           'Section 179',         '🛠', 'IRC § 179 equipment expensing calculator — year-1 deduction up to $1.16M', 'NEW'],
    ['retirement-max',        'Retirement Max',      '🏖', 'SEP IRA vs Solo 401(k) shelter comparator — maximize tax-deferred for SE traders', 'NEW'],
    ['splits-history',        'Splits History',      '🔀', 'Forward + reverse splits per symbol with cumulative multiplier calculation', 'NEW'],
    ['mutual-fund',           'Mutual Fund',         '🏛', 'Profile + top holdings + sector + country mix for any open-end mutual fund', 'NEW'],
    ['uspto-patents',         'USPTO Patents',       '⚗', 'Per-symbol patent filings — R&D velocity catalyst signal (semis, biotech, EVs)', 'NEW'],
    ['home-office',           'Home Office',         '🏠', 'IRC § 280A simplified ($5/sqft) vs actual expense method side-by-side', 'NEW'],
    ['income-1099',           '1099 Tracker',        '📥', 'Log every 1099 received (NEC/MISC/K/INT/DIV/B/R) + match against books', 'NEW'],
    ['meal-deduction',        'Meal Deductions',     '🍽', 'IRC § 274(n) 50% rule meal log with audit-ready substantiation fields', 'NEW'],
    ['biz-categorizer',       'Biz Categorizer',     '🏷', 'Regex-based auto-tag of expense transactions: business / personal / ambiguous', 'NEW'],
    ['depreciation',          'Depreciation',        '📉', 'MACRS GDS 3/5/7/10/15-year schedules with year-1 + accumulated totals', 'NEW'],
    ['travel-per-diem',       'Travel + Per Diem',   '✈️', 'GSA per-diem rates × business-trip log with lodging actual vs per-diem comparison', 'NEW'],
    ['qbi-199a',              'QBI § 199A',          '🎁', '20% pass-through deduction calculator with SSTB phase-out (TTS traders only)', 'NEW'],
    ['state-tax',             'State Tax',           '🗺', '17-state side-by-side effective tax burden — trader-relocation hot list', 'NEW'],
    ['scorp-calc',            'S-Corp Election',     '🏢', 'Sole-prop vs S-corp tax comparison — SE tax savings minus payroll + filing overhead', 'NEW'],
    ['nol-tracker',           'NOL Tracker',         '📉', 'Net Operating Loss carryforward log with TCJA 80% taxable-income limit', 'NEW'],
    ['augusta-rule',          'Augusta Rule',        '🏠', 'IRC § 280A(g) rent-your-home-to-business 14-day tax-free tracker', 'NEW'],
    ['charitable-planner',    'Charitable Planner',  '🎁', 'Cash vs appreciated stock vs DAF bunching strategy — minimize after-tax giving cost', 'NEW'],
    ['fbar-8938',             'FBAR + 8938',         '🌐', 'Foreign account reporting compliance — IBKR-UK, Saxo, IG Markets, etc.', 'NEW'],
    ['sec-1256',              '§ 1256 60/40',        '⚖', 'Futures + SPX/NDX options: 60% LT / 40% ST regardless of holding period, MTM at year-end', 'NEW'],
    ['wash-sale-tracker',     'Wash Sales',          '🌀', 'IRC § 1091 — auto-detect ±30-day loss-disallowance with basis-adjustment math', 'NEW'],
    ['hsa-max',               'HSA Maximizer',       '🩺', 'Triple tax advantage stealth retirement — contribution limits, FV projection, CA/NJ warnings', 'NEW'],
    ['forex-988',             '§ 988 Forex',         '💱', 'Spot forex ordinary income/loss tracking — § 988 default vs § 1256 election', 'NEW'],
    ['rd-credit',             'R&D Credit § 41',     '⚗', 'Alternative Simplified Credit: 14% on incremental QREs — trader algos qualify', 'NEW'],
    ['mtm-election',          '§ 475(f) MTM',        '📝', 'April 15 election deadline tracker + MTM benefits/drawbacks + filing checklist', 'NEW'],
    ['tts-qualification',     'TTS Qualification',   '✅', 'Endicott/Holsinger 11-factor checklist with score + case-law backgrounders', 'NEW'],
    ['qsbs-1202',             'QSBS § 1202',         '💎', 'Qualified Small Business Stock 100% exclusion tracker — $10M or 10× basis cap', 'NEW'],
    ['education-credits',     'Education Credits',   '🎓', 'AOC + LLC calculator: per-student credits, MAGI phase-outs, refundable portion', 'NEW'],
    ['accountable-plan',      'Accountable Plan',    '📋', 'S-corp § 62(c) reimbursement log — replaces dead 2106 employee deduction', 'NEW'],
    ['dcfsa',                 'DCFSA',               '👶', 'Dependent Care FSA + CDCC optimizer — combined shelter for child-care costs', 'NEW'],
    ['ev-credit',             'EV Tax Credit',       '🔋', 'IRC § 30D: $7,500 new / $4,000 used — MSRP + MAGI cap checker with point-of-sale transfer note', 'NEW'],
    ['foreign-tax-credit',    'Foreign Tax Credit',  '🌍', 'IRC § 901: dollar-for-dollar US tax reduction for foreign WHT on dividends / cap gains', 'NEW'],
    ['roth-ladder',           'Roth Conversion',     '🪜', 'FIRE Roth conversion ladder — fill lower brackets in low-income years (5-yr seasoning)', 'NEW'],
    ['gift-tax',              'Gift Tax',            '🎁', '$18k/$36k annual exclusion + $13.6M lifetime exemption tracker — Form 709 trigger detection', 'NEW'],
    ['clean-energy-25d',      'Clean Energy 30%',    '☀️', '§ 25D residential 30% credit: solar / heat pump / battery / geothermal — uncapped, lifetime carryforward', 'NEW'],
    ['savers-credit',         "Saver's Credit",      '🐷', "§ 25B 10-50% credit on retirement contributions — under $76,500 MFJ 2024", 'NEW'],
    ['inherited-ira-rmd',     'Inherited IRA RMD',   '🪦', 'Post-SECURE 10-year rule + EDB life-expectancy + spouse options', 'NEW'],
    ['qcd-tracker',           'QCD Tracker',         '⛪', 'Qualified Charitable Distribution — IRA → charity, counts toward RMD, NOT taxable', 'NEW'],
    ['nua-strategy',          'NUA Strategy',        '📦', 'Employer stock 401(k) in-kind: basis ordinary, appreciation LT cap-gains', 'NEW'],
    ['kiddie-tax',            'Kiddie Tax § 1(g)',   '👶', 'Unearned income > $2,600 taxed at parent rate — gift-to-kid breakeven calc', 'NEW'],
    ['qoz-tracker',           'Opportunity Zone',    '🌆', 'QOF investment ladder: 10/15/100% basis step-ups + Dec 2026 recognition cliff', 'NEW'],
    ['529-roth',              '529 → Roth',          '🎓', 'SECURE 2.0 $35k lifetime rollover ladder, 15-yr 529 minimum, earned-income test', 'NEW'],
    ['se-health-deduction',   'SE Health Insurance', '🩻', 'IRC § 162(l) above-the-line health/dental/LTC deduction with age-based LTC cap', 'NEW'],
    ['mega-backdoor-roth',    'Mega Backdoor Roth',  '🥏', 'After-tax 401(k) → Roth, up to $46k/yr extra Roth space (FAANG plans typically allow)', 'NEW'],
    ['cost-seg',              'Cost Segregation',    '🏗', 'Reclassify 20-40% of real-estate basis to 5/7/15-yr classes + bonus depreciation', 'NEW'],
    ['passive-loss',          'PAL § 469',           '🔻', '$25k allowance phase-out + REP status + suspended-loss carryforward tracker', 'NEW'],
    ['section-1031',          '§ 1031 Exchange',     '🔄', 'Like-kind real estate: 45-day ID / 180-day close countdown + boot + basis carry', 'NEW'],
    ['installment-sale',      '§ 453 Installment',   '📅', 'Gain recognition pro-rata to payments + $150k interest-charge trigger', 'NEW'],
    ['str-loophole',          'STR Loophole',        '🏖', "Avg stay ≤7d + material participation = ordinary loss against W-2 income", 'NEW'],
    ['amt-calc',              'AMT § 55',            '⚠️', 'Alternative Minimum Tax: AMTI + exemption phase-out + 26/28% brackets — ISO trigger', 'NEW'],
    ['iso-exercise',          'ISO Exercise',        '📈', 'Incentive Stock Option bargain element, AMT preference, qualifying disposition check', 'NEW'],
    ['nso-exercise',          'NSO Exercise',        '📊', 'Non-qualified Option: W-2 income at exercise + supplemental W/H + FICA at high income', 'NEW'],
    ['rsu-vest-tracker',      'RSU Vest Tracker',    '💎', 'Per-vest log with FMV income + W/H + federal shortfall warning', 'NEW'],
    ['espp-calc',             'ESPP § 423',          '🧾', 'ESPP qualifying vs disqualifying disposition + W-2 basis-adjustment warning', 'NEW'],
    ['backdoor-roth',         'Backdoor Roth',       '🪟', 'High-earner Roth contribution workaround + pro-rata trap detector + Form 8606 basis', 'NEW'],
    ['cross-broker-wash',     'Cross-Broker Wash',   '🌐', 'Multi-broker wash sale detector — IBKR + Webull + Schwab + Fidelity reconciliation', 'NEW'],
    ['able-account',          'ABLE § 529A',         '♿', 'Tax-advantaged disability savings — $18k/yr + ABLE-to-Work + $100k SSI exclusion', 'NEW'],
    ['conservation-easement', 'Conservation Easement','🌳', 'IRC § 170(h) charitable deduction + Notice 2017-10 syndicated audit warning', 'NEW'],
    ['lihtc',                 'LIHTC § 42',          '🏘', 'Low Income Housing Credit 10-yr stream + 9% / 4% allocation + compliance period', 'NEW'],
    ['mlp-k1',                'MLP K-1 Tracker',     '🛢', 'EPD MPLX ET MMP K-1s + UBTI alert in IRA + 990-T + § 199A QBI + passive losses', 'NEW'],
    ['historic-rehab',        'Historic Rehab § 47', '🏛', '20% credit on certified historic structure rehab + 5-yr spread + state stack', 'NEW'],
    ['disabled-access',       'ADA Access § 44',     '♿', '$5k small biz credit (50% of $250-$10,250) + § 190 barrier removal $15k deduction', 'NEW'],
    ['film-181',              'Film § 181',          '🎬', '100% expensing of film / TV / theater (75% US comp + $15M / $20M low-income cap)', 'NEW'],
    ['partial-disposition',   'Partial Disposition', '🏚', '§ 1.168(i)-8 election: dispose old roof / HVAC + Form 4797 loss + Form 3115 catch-up', 'NEW'],
    ['tts-scorer',            'TTS Scorer',          '🎖', 'Trader Tax Status qualifier — Holsinger Endicott Vines case-law factors + 100-pt score', 'NEW'],
    ['section-475f',          '§ 475(f) MTM',        '📑', 'Mark-to-market election: ordinary, no wash sales, unlimited losses, April 15 deadline', 'NEW'],
    ['section-195',           '§ 195 Start-up',      '🚀', '$5k immediate + 180-mo amort for start-up + § 248 org costs + phase-out at $50k', 'NEW'],
    ['section-1244',          '§ 1244 Ordinary Loss',  '🪦', 'Failed startup stock: $50k/$100k MFJ converted from capital to ORDINARY loss', 'NEW'],
    ['section-280f',          '§ 280F Luxury Auto',  '🚙', 'Passenger auto deprec caps + heavy-SUV § 179 $30.5k + bonus + recapture trap', 'NEW'],
    ['section-197',           '§ 197 Intangibles',   '📂', '15-yr amortization on goodwill + customer list + non-compete + purchased software', 'NEW'],
    ['section-274',           '§ 274 Meals & Gifts', '🍽', 'Meals 50% + entertainment 0% + client gift $25 cap + de-minimis branded items', 'NEW'],
    ['solo-401k',             'Solo 401(k)',         '🏦', '2024 $69k cap / $76.5k catch-up + SE 20% employer share + Roth + Mega Backdoor', 'NEW'],
    ['grat',                  'GRAT',                '🌱', 'Walton zeroed-out grantor retained annuity trust — appreciation transfer gift-free', 'NEW'],
    ['section-469',           '§ 469 PAL + REP',     '🏠', 'Passive loss limits + Real Estate Professional 750-hr/&gt;50% test + material participation', 'NEW'],
    ['sep-ira',               'SEP IRA',             '💼', '25% W-2 / 20% SE up to $69k cap + mandatory employee coverage + SECURE 2.0 Roth', 'NEW'],
    ['section-72t',           '§ 72(t) SEPP',        '⏳', 'Pre-59½ early withdrawal exception — RMD / Amort / Annuit + 5-yr lock + modification trap', 'NEW'],
    ['daf',                   'DAF',                 '🎁', 'Donor Advised Fund — front-load FMV deduction + LTCG avoidance + 5-yr carryforward', 'NEW'],
    ['slat',                  'SLAT',                '👫', 'Spousal Lifetime Access Trust — lock $13.6M exemption pre-2026 + reciprocal-trust trap', 'NEW'],
    ['section-121',           '§ 121 Home Sale',     '🏡', 'Principal residence $250k/$500k exclusion + non-qual-use rentals + dep recapture', 'NEW'],
    ['section-1361',          '§ 1361 S-corp + RC',  '🏢', 'Form 2553 deadline + reasonable comp risk + Watson 8th Cir + SE tax savings', 'NEW'],
    ['section-162l',          '§ 162(l) SEHI',       '🩺', 'Above-the-line SE health insurance + age-tiered LTC + S-corp W-2 box rules', 'NEW'],
    ['section-7872',          '§ 7872 AFR Loan',     '🤝', 'Intra-family loan AFR rates + imputed interest + $10k/$100k safe harbors', 'NEW'],
    ['crut',                  'CRUT',                '🌿', 'Charitable Remainder Unitrust — 5-50% payout + 10% remainder test + 4-tier ordering', 'NEW'],
    ['ilit',                  'ILIT',                '🛡', 'Irrev. Life Insurance Trust — Crummey letters + 3-yr lookback + 5×5 power', 'NEW'],
    ['section-168k',          '§ 168(k) Bonus',      '⚡', 'Bonus depreciation phase-down 100→80→60→40→20→0% by year + § 179 stack', 'NEW'],
    ['section-168',           '§ 168 MACRS',         '📅', 'MACRS depreciation schedule generator — 3/5/7/15/20/27.5/39-yr classes', 'NEW'],
    ['section-263a',          '§ 263A UNICAP',       '📦', 'Producer / reseller capitalization rules + $30M small biz exemption', 'NEW'],
    ['section-6654',          '§ 6654 Safe Harbor',  '🛡', 'Estimated tax penalty — 90% / 100% / 110% safe harbors + quarterly Form 2210', 'NEW'],
    ['section-911',           '§ 911 FEIE',          '🌍', 'Foreign Earned Income Exclusion $126.5k + housing + 330-day physical presence', 'NEW'],
    ['section-1411',          '§ 1411 NIIT',         '📈', '3.8% surtax on investment income — § 475(f) MTM exempts trader, Form 8960', 'NEW'],
    ['section-280e',          '§ 280E Cannabis',     '🌿', 'Schedule I/II trafficker expense disallowance — COGS only + Champ caregiving carve-out', 'NEW'],
    ['section-165g',          '§ 165(g) Worthless',  '🪦', 'Year-end worthless securities cap loss + § 1234A terminated contracts + ordinary § 165(g)(3)', 'NEW'],
    ['section-2010c',         '§ 2010(c) DSUE',      '👰', 'Portability of deceased spouse exemption — 9-mo Form 706 + Rev. Proc. 2022-32 5-yr relief', 'NEW'],
    ['section-174',           '§ 174 R&D Cap',       '🧪', 'Mandatory 5-yr US / 15-yr foreign R&D capitalization + 10% year-1 + repeal watch', 'NEW'],
    ['section-691',           '§ 691 IRD',           '⚰', 'Income in Respect of Decedent — no step-up + § 691(c) deduction for estate tax', 'NEW'],
    ['section-25a',           '§ 25A AOTC/LLC',      '🎓', 'American Opportunity Credit $2.5k + Lifetime Learning $2k + MAGI phase-out', 'NEW'],
    ['section-221',           '§ 221 Student Loan',  '📚', '$2,500 above-the-line interest deduction + MAGI phase-out + SECURE 2.0 401(k) match', 'NEW'],
    ['crat',                  'CRAT',                '📊', 'Charitable Remainder Annuity Trust — fixed $ + 5% exhaustion test + Notice 2008-90', 'NEW'],
    ['residency-daycount',    'State Residency',     '🗺', 'Day-count tracker + NY/CA 183-day stat resident + domicile change checklist', 'NEW'],
    ['section-36b',           '§ 36B ACA PTC',       '🏥', 'ACA Premium Tax Credit + 8.5% MAGI cap + 400% FPL 2026 cliff + Form 8962', 'NEW'],
    ['simple-ira',            'SIMPLE IRA',          '💼', '$16k deferral + 3% match / 2% non-elective + 25% early-W trap + ≤100 employees', 'NEW'],
    ['defined-benefit',       'Defined Benefit',     '🏦', '$275k annual benefit + Cash Balance + actuary + Form 5500 + age-stacked 401(k)', 'NEW'],
    ['section-213',           '§ 213 Medical',       '🩺', 'Schedule A medical above 7.5% AGI + 21¢/mi mileage + LTC + capital expenditures', 'NEW'],
    ['section-6038',          '§ 6038 Form 5471',    '🌐', 'CFC reporting + 5 categories + $10k/mo penalty + Streamlined Domestic Offshore', 'NEW'],
    ['section-408d3',         '§ 408(d)(3) Bobrow',  '🔁', 'Once-per-12-mo IRA rollover (per-taxpayer Bobrow 2014) + trustee-to-trustee unlimited', 'NEW'],
    ['section-162m',          '§ 162(m) Exec Comp',  '💰', '$1M public C-corp exec comp deduction cap + TCJA sticky list + ARPA top-5 2027', 'NEW'],
    ['section-4975',          '§ 4975 PT',           '🚫', 'Prohibited Transactions — SDIRA traps + 15%/100% excise + auto deemed distribution', 'NEW'],
    ['section-4980h',         '§ 4980H Mandate',     '📋', 'ACA Employer Mandate — ALE 50+ FTE + (a) $2,970 + (b) $4,460 affordability 8.39%', 'NEW'],
    ['section-263-tpr',       '§ 263 TPR Repair',    '🔧', 'Tangible Property Regs BAR test + de-minimis + routine + small TP safe harbors', 'NEW'],
    ['section-6048',          '§ 6048 Form 3520',    '📨', 'Foreign trust + $100k foreign gift reporting + 35% penalty + DIIRSP relief', 'NEW'],
    ['section-6038a',         '§ 6038A Form 5472',   '🌎', '25% foreign-owned US corp / disregarded LLC + $25k/year per failure', 'NEW'],
    ['section-7702a',         '§ 7702A MEC Test',    '💸', 'Modified Endowment Contract 7-pay test + LIFO MEC tax + non-MEC benefits', 'NEW'],
    ['crypto-staking',        'Crypto Staking',      '⛓', 'Rev. Rul. 2023-14 staking + airdrop FMV ordinary + Jarrett case + basis tracking', 'NEW'],
    ['section-1042',          '§ 1042 ESOP Defer',   '🏭', 'C-corp founder ESOP sale gain deferral + QRP + step-up-at-death exit hack', 'NEW'],
    ['section-1259',          '§ 1259 Constructive', '📦', 'Short-against-the-box anti-deferral + 30-day/60-day safe harbor + VPF workaround', 'NEW'],
    ['section-1296-pfic',     '§ 1296 PFIC MTM',     '🌍', 'PFIC § 1291 default / § 1296 MTM / § 1295 QEF + Form 8621 + foreign fund traps', 'NEW'],
    ['section-1245-1250',     '§ 1245/1250 Recap',   '↩', 'Depreciation recapture — § 1245 ordinary + § 1250 unrecap 25% + § 291 corp add-on', 'NEW'],
    ['section-351-721',       '§ 351/721 Formation', '🏗', 'Tax-free corp/partnership formation + 80% control + § 357(c) liability boot trap', 'NEW'],
    ['section-172',           '§ 172 NOL',           '📉', 'Net Operating Loss carryforward + 80% cap post-TCJA + § 461(l) EBL $305k/$610k', 'NEW'],
    ['section-1035',          '§ 1035 Ins. Exch.',   '🔄', 'Tax-free annuity / life / LTC contract exchange + boot trap + MEC carryover', 'NEW'],
    ['section-6707a',         '§ 6707A Reportable',  '🚨', 'Listed transactions + 75% penalty (min $10k/$50k) + SCE + captive + Form 8886', 'NEW'],
    ['section-280g',          '§ 280G Parachute',    '🪂', 'Change-of-control 3× base safe harbor + 20% excise + private cleansing vote 75%', 'NEW'],
    ['section-1374',          '§ 1374 S-corp BIG',   '🔁', 'Built-In Gains tax on C → S conversion + 5-yr recognition + NUBIG tracking', 'NEW'],
    ['section-871m',          '§ 871(m) Swap WH',    '📐', 'Foreign div-equivalent on TRS / swaps + delta-1 / 0.80 + QDD + Notice 2024-44', 'NEW'],
    ['section-1402',          '§ 1402 SE Tax',       '👤', 'Self-Employment tax 15.3% + SS base + Add Medicare 0.9% + half-SE deduction', 'NEW'],
    ['section-1276',          '§ 1276 Mkt Discount', '📉', 'Bond market discount ordinary income + de minimis + § 1278(b) current inclusion', 'NEW'],
    ['section-1233',          '§ 1233 Short Hold',   '🔻', 'Short sale holding period rules + covered short + ST/LT character determination', 'NEW'],
    ['section-6166',          '§ 6166 Estate Inst.', '🏛', '14-yr estate tax installment for closely-held biz + 2% special rate + 35% test', 'NEW'],
    ['section-4941',          '§ 4941 PF Self-Deal', '🏛', 'Private foundation self-dealing strict-liability 10%/200% + Form 4720', 'NEW'],
    ['section-1092',          '§ 1092 Straddle',     '⚖', 'Straddle loss deferral + § 263(g) cap interest + mixed-straddle election', 'NEW'],
    ['section-6038b',         '§ 6038B Form 926',    '✈', 'Foreign corp transfer + § 367 gain + Form 926 + 10% penalty up to $100k', 'NEW'],
    ['section-2056',          '§ 2056 Marital QTIP', '💑', 'Marital deduction + QTIP trust + QDOT non-citizen + portability interplay', 'NEW'],
    ['section-4940',          '§ 4940 PF NIIT',      '📊', 'Private foundation 1.39% net investment income tax + Form 990-PF Part XI', 'NEW'],
    ['section-7345',          '§ 7345 Passport',     '🛂', 'Passport revocation for $62k+ seriously delinquent tax + CP508C + reversal', 'NEW'],
    ['section-1212',          '§ 1212 Cap Loss CF',  '📈', 'Capital loss carryforward + $3k ordinary offset + C-corp 3-yr carryback / 5-yr CF', 'NEW'],
    ['section-4980d',         '§ 4980D HRA Excise',  '⚕', '$100/day per employee + QSEHRA / ICHRA safe harbors + Form 8928 + Notice 2013-54', 'NEW'],
    ['section-6663',          '§ 6663 Civil Fraud',  '⚖', '75% civil fraud + clear+convincing + Spies badges + unlimited SOL + Form 4549', 'NEW'],
    ['section-6694',          '§ 6694 Preparer',     '👨‍💼', 'Return preparer penalty (a) $1k/50% + (b) $5k/75% + reasonable basis standards', 'NEW'],
    ['section-6045b',         '§ 6045B Form 8937',   '📋', 'Issuer corporate action reporting + 45-day deadline + broker basis 1099-B chain', 'NEW'],
    ['section-529',           '§ 529 Plan',          '🎓', '529 contributions + state deductions + 5-yr super-fund + K-12 + SECURE 2.0 Roth rollover', 'NEW'],
    ['section-530',           '§ 530 Coverdell ESA', '📚', 'Coverdell ESA $2k cap + $95-220k MAGI phase-out + broader K-12 expenses than 529', 'NEW'],
    ['section-401k-hardship', '§ 401(k) Hardship',   '🆘', 'Hardship withdrawal safe harbors + SECURE 2.0 disasters + birth/adoption + $1k emergency', 'NEW'],
    ['section-72p',           '§ 72(p) Plan Loan',   '💳', '50% or $50k cap + 5-yr term (30-yr home) + opportunity cost + default trap', 'NEW'],
    ['section-401a9',         '§ 401(a)(9) RMD',     '⏰', 'RMD calculator + age 75 SECURE 2.0 + QCD $105k + 25% missed-RMD penalty', 'NEW'],
    ['section-6015',          '§ 6015 Innocent Sp',  '💔', 'Innocent spouse relief — (b) Traditional + (c) Separation + (f) Equitable + Form 8857', 'NEW'],
    ['section-6651',          '§ 6651 FTF / FTP',    '📅', 'Failure to file 5%/mo + failure to pay 0.5%/mo + FTA + reasonable cause + min $485', 'NEW'],
    ['section-1014',          '§ 1014 Step-up',      '⚰', 'Basis step-up at death + community property + IRD exception + JTWROS half-rule', 'NEW'],
    ['section-23',            '§ 23 Adoption Credit','👶', '$16,810 (2024) adoption credit + § 137 employer exclusion + special needs full credit', 'NEW'],
    ['section-32-eic',        '§ 32 EITC',           '💰', 'Earned Income Tax Credit — refundable $632-$7,830 + investment income $11,600 cap', 'NEW'],
    ['section-4942',          '§ 4942 PF Undist.',   '🏛', 'PF 5% minimum payout + 30%/100% excise + 5-yr carryforward + set-asides', 'NEW'],
    ['section-4960',          '§ 4960 TE Comp 21%',  '🎓', 'Tax-exempt $1M comp 21% excise + top-5 sticky list + medical exclusion', 'NEW'],
    ['section-6213',          '§ 6213 90-Day Letter','📨', 'Notice of Deficiency + Tax Court 90-day deadline + forum comparison', 'NEW'],
    ['section-6321',          '§ 6321 Federal Lien', '🔒', 'Federal tax lien arises at assessment + NFTL priority + § 6325 release options', 'NEW'],
    ['section-6331',          '§ 6331 Levy',         '⚖', 'Wage / bank levy + 30-day CDP hearing + exemptions ($5.5k tools + basic living)', 'NEW'],
    ['section-4943',          '§ 4943 Excess Hold.', '🏢', 'PF excess business holdings 20%/35% + 5-yr divestiture + 10%/200% excise', 'NEW'],
    ['section-4944',          '§ 4944 Jeopardize',   '🎲', 'PF jeopardizing investments + PRI exception + prudent investor rule + 10%/25%', 'NEW'],
    ['section-4945',          '§ 4945 Taxable Exp.', '📤', 'PF taxable expenditures + expenditure responsibility + 20%/100% PF + 5%/50% mgr', 'NEW'],
    ['section-6664',          '§ 6664 Reasonable',   '🛡', 'Reasonable cause defense vs penalties + Neonatology factors + Boyle limits', 'NEW'],
    ['section-7430',          '§ 7430 Atty Fees',    '💵', 'Recover litigation costs from IRS + $230/hr cap + $2M / $7M net worth limits', 'NEW'],
    ['section-6502',          '§ 6502 CSED 10-yr',   '⏳', 'Collection Statute Expiration Date + tolling events + Form 4340 verify', 'NEW'],
    ['section-7122',          '§ 7122 OIC',          '🤝', 'Offer in Compromise: DATC / DATL / ETA + RCP formula + Form 656', 'NEW'],
    ['section-6159',          '§ 6159 IA',           '📅', 'Installment Agreements — Guaranteed / Streamlined / Routine / Non-streamlined / PPIA', 'NEW'],
    ['section-7811',          '§ 7811 TAO',          '🆘', 'Taxpayer Advocate Service Assistance Order — Form 911 / hardship qualification', 'NEW'],
    ['section-6724',          '§ 6724 Info Penalty', '📄', 'Info return penalty tiers $60/$130/$310 + de minimis + reasonable cause waiver', 'NEW'],
    ['section-24-ctc',        '§ 24 CTC',            '👶', 'Child Tax Credit $2k + ACTC refundable $1,700 + $500 other dependent + MAGI phase-out', 'NEW'],
    ['section-21-cdcc',       '§ 21 CDCC',           '🧒', 'Child + Dependent Care Credit 20-35% of $3k/$6k + § 129 FSA coordination', 'NEW'],
    ['section-71-alimony',    '§ 71 Alimony / 1041', '💔', 'TCJA 2018 alimony deduction repeal + § 1041 spousal transfer carryover basis', 'NEW'],
    ['section-152',           '§ 152 Dependents',    '👨‍👩‍👧', 'Qualifying Child vs Qualifying Relative + tiebreaker + $5,050 income test', 'NEW'],
    ['section-7508a',         '§ 7508A Disaster',    '🌪', 'IRS disaster postponement up to 1 yr + FEMA county + 2024 hurricane / wildfire list', 'NEW'],
    ['section-132',           '§ 132 Fringe',        '🎁', '8 statutory fringe benefits + qualified transportation $315/mo + de minimis examples', 'NEW'],
    ['section-127',           '§ 127 Ed Assist.',    '📖', 'Employer educational assistance $5,250/yr + SECURE 2.0 student loan payments', 'NEW'],
    ['section-125',           '§ 125 Cafeteria',     '🥗', 'Section 125 cafeteria plan + Health FSA $3,200 + DCFSA $5k + HSA + nondiscrim tests', 'NEW'],
    ['section-165c3',         '§ 165(c)(3) Casualty','⚡', 'Federally-declared disaster casualty loss + $100/10% AGI floor + § 165(i) prior year', 'NEW'],
    ['section-119',           '§ 119 Meals/Lodging', '🍽', 'Convenience of employer meals + lodging exclusion + business premises + TCJA 50%', 'NEW'],
    ['section-25c',           '§ 25C Energy Home',   '🔧', 'Energy efficient home improvement credit 30% + heat pump $2k + windows + audit', 'NEW'],
    ['section-45l',           '§ 45L Builder Home',  '🏗', 'New energy efficient home credit $2.5k-$5k builder + prevailing wage multiplier', 'NEW'],
    ['section-179d',          '§ 179D Commercial',   '🏢', 'Commercial building energy deduction $0.50-$5.65/sqft + ASHRAE 90.1 + designer alloc', 'NEW'],
    ['section-86',            '§ 86 SS Taxability',  '👴', 'Social Security 0/50/85% taxability + provisional income + retirement planning', 'NEW'],
    ['section-219',           '§ 219 IRA Deduction', '💰', 'Traditional IRA $7k/$8k + active participant phase-out + Form 8606 nondeductible', 'NEW'],
    ['section-415',           '§ 415 Combined',      '🧮', 'Combined 401(k) limits $69k + § 415(b) DB $275k + comp cap $345k + HCE $155k', 'NEW'],
    ['section-414v',          '§ 414(v) Catch-up',   '👴', 'Catch-up $7.5k / $11.25k super + SECURE 2.0 mandatory Roth for high earners 2026', 'NEW'],
    ['section-481a',          '§ 481(a) Method Chg', '📋', 'Accounting method change Form 3115 + 4-yr spread + automatic DCN + de minimis $50k', 'NEW'],
    ['section-168g',          '§ 168(g) ADS',        '⏲', 'Alternative Depreciation System + straight-line + 50% business use + AMT minimization', 'NEW'],
    ['section-416',           '§ 416 Top-Heavy',     '⚖', 'Top-heavy 60% test + 3% min contribution + 3-yr cliff vesting + Safe Harbor relief', 'NEW'],
    ['section-446',           '§ 446 Acctg Methods', '📊', 'Cash vs accrual + hybrid + $30M small biz threshold + Form 3115 method change', 'NEW'],
    ['section-451',           '§ 451 Advance Pay',   '💵', 'Income inclusion + § 451(c) advance payment 1-yr deferral + Rev. Proc. 2004-34', 'NEW'],
    ['section-461',           '§ 461 Econ Perf',     '⏱', 'All-events test + economic performance + recurring item 8.5-mo + § 461(l) EBL', 'NEW'],
    ['section-471',           '§ 471 Inventory',     '📦', 'FIFO / LIFO / Specific ID + § 471(c) small biz non-incidental + LIFO conformity', 'NEW'],
    ['section-482',           '§ 482 Transfer Pri.', '🌐', 'Arm\'s-length + CUP / RPM / CPM / PSM / TNMM + 6 documents + § 6662(e) 20%/40%', 'NEW'],
    ['section-901',           '§ 901 Foreign Tax Cr.', '🌍', 'Direct FTC + § 904 limit + baskets + 1-yr back / 10-yr fwd carry', 'NEW'],
    ['section-951',           '§ 951 Subpart F',  '🌐', 'CFC anti-deferral + FBC Sales / Services / FPHC + § 962 election', 'NEW'],
    ['section-951a',          '§ 951A GILTI',     '🌍', 'Tested income − 10% QBAI + 50% § 250 + 80% § 960(d) credit', 'NEW'],
    ['section-250',           '§ 250 FDII / GILTI', '🌐', '37.5% FDII + 50% GILTI deduction → 13.125% / 10.5% effective', 'NEW'],
    ['section-163j',          '§ 163(j) Int. Limit', '💰', 'BIE ≤ BII + 30% ATI + floor-plan; $30M small biz + elect-out', 'NEW'],
    ['section-59a',           '§ 59A BEAT',       '🌐', 'Base Erosion Anti-Abuse: 10%/12.5% MTI − reg. tax; $500M + 3% threshold', 'NEW'],
    ['section-245a',          '§ 245A 100% DRD',  '🌍', '100% deduction foreign-source dividend from 10%-owned + § 1248 + holding period', 'NEW'],
    ['section-7874',          '§ 7874 Anti-Inv.', '🌐', '80% / 60-79% inversion tests + SBA 25% × 3 + serial inversion lookback', 'NEW'],
    ['section-4501',          '§ 4501 Buyback Tax', '💰', '1% (proposed 4%) excise on stock repurchases − net issuances; $1M de minimis', 'NEW'],
    ['section-877a',          '§ 877A Exit Tax',  '🌍', 'Covered expatriate MTM + $2M / $201K triggers + $866K exemption + § 2801', 'NEW'],
    ['section-1291',          '§ 1291 PFIC Excess', '🌐', 'Default punitive PFIC: top ordinary rate + interest charge over holding pd', 'NEW'],
    ['section-1295',          '§ 1295 PFIC QEF',  '🌍', 'QEF election: current inclusion ord. + cap gain, no interest, requires AIS', 'NEW'],
    ['section-897',           '§ 897 FIRPTA',     '🏠', 'USRPI gain ECI + USRPHC 50% test + 15%/10%/0% withholding + QFP § 897(l)', 'NEW'],
    ['section-1445',          '§ 1445 FIRPTA WH', '🏠', '15% buyer withholding + 10% personal $1M + 0% $300K + Form 8288-B reduction', 'NEW'],
    ['section-754',           '§ 754 PS Step-Up', '🤝', '§ 743(b) on transfer + § 734(b) on distribution + mandatory SBIL $250K', 'NEW'],
    ['section-302',           '§ 302 Stock Redem.', '💱', 'Sale vs dividend tests (b)(1)/(2)/(3)/(4) + § 318 attribution + waiver', 'NEW'],
    ['section-332',           '§ 332 Sub Liquid.', '🏢', 'Tax-free 80% sub liquidation + § 337 + § 381 carryover + insolvent fail', 'NEW'],
    ['section-338',           '§ 338(g)/(h)(10)', '🔄', 'Deemed asset sale election + AGUB step-up + Class IV-VII allocation', 'NEW'],
    ['section-368',           '§ 368 Reorgs',     '🔀', 'Types A/B/C/D/E/F/G + COI 40% + COBE + business purpose + § 382 NOL', 'NEW'],
    ['section-1248',          '§ 1248 CFC Sale',  '🌍', 'CFC stock gain → dividend recharacterization + § 245A path + PTEP excl', 'NEW'],
    ['section-355',           '§ 355 Spin-Off',   '🔀', 'Spin / split / split-up + 5-yr active + device + § 355(d) / (e) anti-abuse', 'NEW'],
    ['section-311',           '§ 311 Corp Dist.', '💸', '§ 311(b) gain on appreciation; no loss; § 301 dividend tier ordering', 'NEW'],
    ['section-1366',          '§ 1366 S-Corp PT', '📊', 'Pass-through w/ character preservation + § 1366(d) basis loss limit', 'NEW'],
    ['section-1368',          '§ 1368 S-Corp Dist', '💰', 'AAA → PTI → E&P → OAA → basis → cap gain + bypass election', 'NEW'],
    ['section-41',            '§ 41 R&D Credit',  '🔬', 'ASC 14% / Regular 20% + $500K payroll tax election + 4-part test QRE', 'NEW'],
    ['section-1033',          '§ 1033 Involuntary', '🌪️', 'Casualty/theft/condemnation deferral 2/3/4-yr window + § 121 combo', 'NEW'],
    ['section-1400z',         '§ 1400Z OZ',       '🌆', 'Opportunity Zones: defer + step-up + 10-yr exclusion through 2028', 'NEW'],
    ['section-357',           '§ 357 § 351 Liab.', '📋', '§ 357(b) avoidance + § 357(c) excess liab gain on incorporation', 'NEW'],
    ['section-305',           '§ 305 Stock Div.', '📈', 'Tax-free split + 5 § 305(b) exceptions + § 305(c) deemed PIK / arrear', 'NEW'],
    ['section-336',           '§ 336 Corp Liquid.', '🏢', 'General gain/loss + § 336(d) disqualified prop + § 336(e) sister 338', 'NEW'],
    ['section-30d',           '§ 30D Clean Vehicle', '⚡', 'Up to $7,500 EV credit + $80K/$55K MSRP + MAGI limits + POS transfer', 'NEW'],
    ['section-45q',           '§ 45Q Carbon',     '🌫️', '$85 geologic / $60 utiliz / $180 DAC + 5× wage bonus + transferability', 'NEW'],
    ['section-48',            '§ 48 ITC',         '☀️', '30% solar/energy ITC + adders (domestic 10% + community 10% + LMI 10%)', 'NEW'],
    ['section-38',            '§ 38 GBC',         '🧮', 'Sum of all business credits — TMT limit + 1-yr back / 20-yr forward', 'NEW'],
    ['section-6050w',         '§ 6050W 1099-K',   '💳', 'TPSO threshold $20K/$5K/$2.5K/$600 phasein + personal carve-out', 'NEW'],
    ['section-45x',           '§ 45X Adv. Mfg',   '🏭', 'Per-unit solar/wind/battery + critical minerals 10% PERMANENT', 'NEW'],
    ['section-45v',           '§ 45V Hydrogen',   '⚛️', 'Up to $3/kg by GHG tier + 10-yr PTC + three pillars + § 48 election', 'NEW'],
    ['section-48c',           '§ 48C Adv. Energy', '🏗️', '30% DOE competitive allocation $10B + 40% energy community reserve', 'NEW'],
    ['section-25d',           '§ 25D Resid. Solar', '☀️', '30% home solar + battery ≥ 3kWh + unlimited carryforward through 2034', 'NEW'],
    ['section-1446f',         '§ 1446(f) PS Int.',  '🌍', '10% withholding on foreign sale of US PS interest + ECI deemed sale', 'NEW'],
    ['section-42',            '§ 42 LIHTC',       '🏘️', '9% / 4% credit + 10-yr period + 20-50 / 40-60 / income avg + QCT 30%', 'NEW'],
    ['section-83',            '§ 83 / § 83(b)',   '📜', 'Restricted property: 30-day election locks grant FMV + starts LTCG clock', 'NEW'],
    ['section-409a',          '§ 409A NQDC',      '⏰', 'Nonqual deferred comp: 6 events + 20% penalty + interest if violation', 'NEW'],
    ['section-457',           '§ 457 Gov / NP',   '🏛️', '457(b) $23,500 + triple catch-up + 457(f) ineligible top-hat', 'NEW'],
    ['section-30c',           '§ 30C EV Charger', '🔌', '30% / $100K business or $1K personal + non-urban / low-income only', 'NEW'],
    ['section-45w',           '§ 45W Comm. EV',   '🚛', '$7,500 / $40K heavy EV — NO income / MSRP / NA limits ("lease loophole")', 'NEW'],
    ['section-47',            '§ 47 Historic Reh.', '🏛️', '20% QRE × 5-yr vesting + NPS / SHPO 3-part + substantial rehab test', 'NEW'],
    ['section-51',            '§ 51 WOTC',        '👷', 'Up to $9,600 per qualified hire + 9 target groups + 28-day Form 8850', 'NEW'],
    ['section-25e',           '§ 25E Used EV',    '🚙', '30% × $25K cap or $4K — MAGI $75/112.5/150K + 2yr age + qual. dealer', 'NEW'],
    ['section-460',           '§ 460 LT Contracts', '🏗️', 'PCM cost-to-cost + $30M small biz CCM + Form 8697 look-back interest', 'NEW'],
    ['section-467',           '§ 467 Deferred Rent', '🏢', '$250K + stepped → CRA level + AFR imputed interest + leaseback CRA', 'NEW'],
    ['section-79',            '§ 79 Group Life',  '💀', '$50K excl + Table I rate × excess + discrim test + W-2 Box 12 code C', 'NEW'],
    ['section-105',           '§ 105 Health Pay', '🏥', '§ 105(a)/(b)/(c)/(h) exclusion + § 162(l) SE health above-the-line', 'NEW'],
    ['section-269',           '§ 269 Anti-Abuse', '🚫', 'Disallow NOL/credits/deductions on acquisition w/ tax-avoidance purpose', 'NEW'],
    ['section-1239',          '§ 1239 Related',   '🤝', 'Sale to related w/ depreciable = ordinary income (not LTCG)', 'NEW'],
    ['section-7701o',         '§ 7701(o) Econ Sub.', '⚖️', 'Codified 2010 — 20%/40% strict-liability penalty if no economic substance', 'NEW'],
    ['section-106',           '§ 106 Employer Hlth', '🏥', 'Premiums + HSA + HRA excl from W-2; 2% S-corp owner excluded', 'NEW'],
    ['section-1015',          '§ 1015 Gift Basis', '🎁', 'Carryover basis + dual basis rule + § 1015(d)(6) gift tax adjustment', 'NEW'],
    ['section-444',           '§ 444 Fiscal Yr',  '📅', 'PSC + PS + S-corp 3-mo deferral + § 7519 deposit + NBY 25% excep', 'NEW'],
    ['section-904',           '§ 904 FTC Limit',  '🌍', '(For TI / WW TI) × US tax + baskets + 1-yr back / 10-yr fwd carry', 'NEW'],
    ['section-7701b',         '§ 7701(b) Resident', '🛂', 'Green card OR SPT 31+183 weighted + closer connection + treaty TB', 'NEW'],
    ['section-1041',          '§ 1041 Spouse/Divorce', '💍', 'No gain/loss + carryover basis + 1-yr / pursuant to divorce', 'NEW'],
    ['section-871a',          '§ 871(a) NRA FDAP',  '🌐', '30% withholding on US-source FDAP + treaty rates + portfolio int. excl', 'NEW'],
    ['section-367d',          '§ 367(d) IP Outb.',  '🌍', 'Deemed annual royalty on IP outbound + TCJA workforce + goodwill incl', 'NEW'],
    ['section-165d',          '§ 165(d) Gambling',  '🎲', 'Losses ≤ winnings only + Schedule A itemize + W-2G + Mayo decision', 'NEW'],
    ['section-1377',          '§ 1377 Closing Bks', '📚', '§ 1377(a)(2) close-books + § 1377(b) PTTP 1yr+120d tax-free dist', 'NEW'],
    ['section-162f',          '§ 162(f) Fines',   '🚫', 'Gov fines/penalties non-deductible + 1098-F identification of restitution', 'NEW'],
    ['section-162c',          '§ 162(c) Illegal', '⚖️', 'Bribes/kickbacks/Medicare fraud non-deductible + FCPA + Stark Law', 'NEW'],
    ['section-1297',          '§ 1297 PFIC Def',  '🌐', '75% income / 50% asset test + lookthrough + startup + CFC overlap', 'NEW'],
    ['section-6655',          '§ 6655 Corp Est',  '📅', 'Quarterly installments + 100% prior / current safe harbor + 2220 penalty', 'NEW'],
    ['section-6700',          '§ 6700 Promoter',  '🛡️', 'Abusive shelter promoter penalty: $1K or 100% gross income + § 6701', 'NEW'],
    ['section-6325',          '§ 6325 Lien Release', '🔓', 'Full release + § 6325(b) discharge + subordination + § 6326 appeal', 'NEW'],
    ['section-1298',          '§ 1298 PFIC Rules',  '🌐', 'Attribution + once-PFIC taint + startup + CFC overlap + Form 8621', 'NEW'],
    ['section-6072',          '§ 6072 Due Dates',   '📅', 'Apr 15 / Mar 15 / 7004 extension + combat zone + disaster relief', 'NEW'],
    ['section-162a1',         '§ 162(a)(1) Reas Comp', '⚖️', '5-factor Mayson + Indep. Investor + S-corp reverse problem + Watson', 'NEW'],
    ['section-367a',          '§ 367(a) Outb Tang',  '🌍', 'Gain on tangible outbound + § 367(a)(3) active + GRA + § 6038B 926', 'NEW'],
    ['section-882',           '§ 882 Foreign ECI', '🌐', 'Foreign corp 21% on ECI + § 882(c) deduction filing + treaty PE', 'NEW'],
    ['section-884',           '§ 884 Branch Profits', '🌍', '30% BPT on DEA + treaty rates + § 884(f) branch interest', 'NEW'],
    ['section-6045',          '§ 6045 Broker Rpt', '📊', '1099-B basis reporting + covered securities + FIFO/HIFO/spec ID', 'NEW'],
    ['section-263c',          '§ 263(c) IDC',     '🛢️', '100% indep / 70% integrated IDC current + AMT pref + § 1254 recap', 'NEW'],
    ['section-412',           '§ 412 Pension Fund', '🏦', 'DB min funding § 430/431 + PBGC premiums + § 4971 10%/100% excise', 'NEW'],
    ['section-1362',          '§ 1362 S Election',  '📝', 'Form 2553 by Mar 15 + consents + termination triggers + Rev Proc 2013-30', 'NEW'],
    ['section-414',           '§ 414 Aggregation',  '🔗', 'Parent-sub / brother-sister 80% + ASG + benefit plan + ACA aggregation', 'NEW'],
    ['section-472',           '§ 472 LIFO Method',  '📦', 'LIFO election + book conformity + dollar-value + § 1363(d) recapture', 'NEW'],
    ['section-901j',          '§ 901(j) Sanctioned', '🚫', 'NO FTC for taxes paid to sanctioned countries (Iran/NK/Syria/Cuba+)', 'NEW'],
    ['section-248',           '§ 248 Org. Exp.',   '🏛️', '$5K immediate + 180-mo amortization + $50K phaseout + § 195/709 parallel', 'NEW'],
    ['section-6041',          '§ 6041 1099-NEC',   '📋', '$600+ nonemp comp + corp exceptions + 24% backup withholding', 'NEW'],
    ['section-6049',          '§ 6049 1099-INT',   '🏦', '$10+ interest + Boxes 1-15 + tax-exempt + PAB AMT + § 871(i)', 'NEW'],
    ['section-6051',          '§ 6051 W-2 Wages',  '💼', 'Box 1 wages + Box 12 codes + Jan 31 deadline + 2% S-corp owner', 'NEW'],
    ['section-1273',          '§ 1273 OID',       '📈', 'Original Issue Discount + de minimis + zero coupon + 1099-OID', 'NEW'],
    ['section-7702',          '§ 7702 Life Insur.', '🛡️', 'CVAT or GPT+CVCT + 2021 lower 2% floor + MEC § 7702A interaction', 'NEW'],
    ['section-6033',          '§ 6033 Form 990',  '🤝', '501(c) annual filing + 3-yr fail = auto revocation + Schedule B', 'NEW'],
    ['section-7491',          '§ 7491 Burden',    '⚖️', 'IRS burden shift if records + cooperation + § 7491(c) penalty burden', 'NEW'],
    ['section-483',           '§ 483 Imputed Int.', '💸', 'AFR test for installment + below-AFR → imputed interest portion', 'NEW'],
    ['section-4973',          '§ 4973 Excess IRA', '⚠️', '6% annual excise on excess IRA/HSA + cumulative + NIA computation', 'NEW'],
    ['section-6695',          '§ 6695 Preparer',  '📝', '$60 procedural + $635 EITC/CTC due diligence + Form 8867', 'NEW'],
    ['section-2503',          '§ 2503 Gift Excl', '🎁', '$19K annual excl + Crummey + unlim education/medical + 529 5-yr', 'NEW'],
    ['section-2055',          '§ 2055 Estate Char', '🏛️', 'Unlimited bequest deduction + CRT/CLT + actuarial split-interest', 'NEW'],
    ['section-511',           '§ 511 UBIT',       '💼', '21% UBIT + 3-part test + § 512(b) passive excl + § 512(a)(6) silos', 'NEW'],
    ['section-2032',          '§ 2032 AVD',       '📅', 'Alt valuation 6-month date + reduce estate + tax both required', 'NEW'],
    ['section-4974',          '§ 4974 RMD Penalty', '⏰', '25%/10% (SECURE 2.0) on RMD shortfall + 2-yr cure + QCD offset', 'NEW'],
    ['section-6111',          '§ 6111 Material Adv', '🔎', 'Form 8918 material advisor + $200K/75% income + § 6112 list', 'NEW'],
    ['section-4972',          '§ 4972 Excess Plan', '⚖️', '10% on nondeduct contrib + § 415(c)/§ 404 limits + cumulative', 'NEW'],
    ['section-6039',          '§ 6039 ISO/ESPP',  '📑', 'Form 3921/3922 + § 422 ISO + § 423 ESPP + AMT spread trap', 'NEW'],
    ['section-894',           '§ 894 Treaty Pos', '🌐', 'Form 8833 + § 6114 disclosure + LOB Art 22 + conduit § 7701(l)', 'NEW'],
    ['section-2518',          '§ 2518 Disclaimer', '🚫', '9-mo qualified disclaimer + spouse exception + GST + UDPIA', 'NEW'],
    ['section-199a',          '§ 199A QBI',       '💰', '20% pass-through deduction + SSTB phase-out + W2/UBIA limit', 'NEW'],
    ['section-461l',          '§ 461(l) Excess Loss', '📉', '$305K/$610K cap on non-corp business loss + NOL carryforward', 'NEW'],
    ['section-165',           '§ 165 Losses',     '💥', 'Casualty + theft + worthless securities + Ponzi (Rev Proc 2009-20)', 'NEW'],
    ['section-408a',          '§ 408A Roth IRA',  '🌳', '$7K limit + MAGI phase-out + backdoor + mega backdoor + 5-yr rule', 'NEW'],
    ['section-962',           '§ 962 Election',   '🏢', 'Individual CFC shareholder corp rate + § 250 + § 960 80% FTC', 'NEW'],
    ['section-280c',          '§ 280C Credit Dedn', '🔻', 'R&D credit § 174 basis reduction OR § 280C(c)(2) 79% election', 'NEW'],
    ['section-1202',          '§ 1202 QSBS',      '🚀', 'Up to 100% gain exclusion + $10M/10× basis cap + § 1045 rollover', 'NEW'],
    ['section-170',           '§ 170 Charitable', '🤲', '60% AGI cash + 30% LTCG property + DAF + QCD + substantiation', 'NEW'],
    ['section-351',           '§ 351 Tax-Free TX', '↪️', '80% control + carryover basis + § 357 liab + § 358/362', 'NEW'],
    ['section-6038d',         '§ 6038D Form 8938', '🌍', 'SFFA reporting $50K-$600K threshold + $10K base + § 6662(j) 40%', 'NEW'],
    ['section-752',           '§ 752 Partnership Liab', '🤝', 'Recourse vs nonrecourse 3-tier + outside basis + § 465 at-risk', 'NEW'],
    ['section-1245',          '§ 1245 Recapture', '🔄', 'Personal property depreciation recapture as ordinary + § 197 intangibles', 'NEW'],
    ['section-164',           '§ 164 SALT Cap',   '🧾', '$10K SALT cap 2018-25 + PTET workaround + § 27 FTC choice', 'NEW'],
    ['section-24',            '§ 24 CTC',         '👶', '$2K CTC + $1.7K ACTC refundable + $500 ODC + $200/400K phase', 'NEW'],
    ['section-6045a',         '§ 6045A Basis TX', '↔️', 'Covered security transfer + § 1012 method + § 1091 wash sale', 'NEW'],
    ['section-743',           '§ 743 Basis Adj',  '⚖️', '§ 754 step-up + § 743(d) SBIL $250K + § 755 allocation', 'NEW'],
    ['section-1250',          '§ 1250 Real Recap', '🏢', 'Unrecaptured § 1250 25% + § 291 corp + cost seg interaction', 'NEW'],
    ['section-1231',          '§ 1231 Hotchpot',  '🎲', 'Net gain LTCG / net loss ordinary + 5-yr look-back § 1231(c)', 'NEW'],
    ['section-871',           '§ 871 NRA Tax',    '🌐', '30% FDAP + ECI + § 871(m) div equiv + portfolio interest exempt', 'NEW'],
    ['section-731',           '§ 731 PS Distrib', '🪙', '§ 731(a)(1) cash > basis + § 731(c) securities + § 737 7-yr look', 'NEW'],
    ['section-6011',          '§ 6011 Form 8886', '📋', 'RT disclosure + § 6707A 75% $100K/$200K + § 6501(c)(10) SOL', 'NEW'],
    ['section-382',           '§ 382 NOL Limit',  '🚧', 'NOL annual limit after ownership change + § 383 + § 382(h) RBIG', 'NEW'],
    ['section-1234',          '§ 1234 Options',   '🎯', 'Holder capital + writer STCG + § 1234A termination + § 1256 60/40', 'NEW'],
    ['section-32',            '§ 32 EITC',        '💸', '2024 $7.8K max (3+ kids) + 40% phase-in + $11.6K disqualified inc', 'NEW'],
    ['section-707',           '§ 707 Partner TX', '🔁', '§ 707(a) disguised sale 2-yr + § 707(b) related + § 707(c) GP', 'NEW'],
    ['section-736',           '§ 736 Retire Pmt', '👋', '§ 736(b) capital + § 736(a) ordinary + svc PS unrealized/goodwill', 'NEW'],
    ['section-1058',          '§ 1058 Sec Loan',  '🔄', 'Nonrecognition + substitute pmts ordinary + § 871(m) + § 263(g)', 'NEW'],
    ['section-269a',          '§ 269A PSC',       '👔', 'IRS reallocation if 95% to one entity + tax avoidance + § 444 5-5-5', 'NEW'],
    ['section-318',           '§ 318 Constructive', '🔗', '5 attribution rules: family + entity + option + reattribution', 'NEW'],
    ['section-481',           '§ 481 Method Chg', '📈', 'Form 3115 + 4-yr unfav / 1-yr fav spread + Rev Proc 2022-14', 'NEW'],
    ['section-6221',          '§ 6221 BBA Audit', '🏛️', 'Entity-level audit + 37% imputed + § 6226 push-out + § 6223 PR', 'NEW'],
    ['section-956',           '§ 956 CFC US Prop', '🌎', 'CFC investing in US property = inclusion + § 245A hybrid relief', 'NEW'],
    ['section-1296',          '§ 1296 PFIC MTM',  '📊', 'PFIC mark-to-market election + ordinary + § 1295 QEF alternative', 'NEW'],
    ['section-1059',          '§ 1059 Extra Div', '💎', '5% common / 10% preferred basis reduction + 85-day aggregation', 'NEW'],
    ['section-304',           '§ 304 Related Red', '🔁', 'Brother-sister + parent-sub redemption + § 301 dividend treatment', 'NEW'],
    ['section-6112',          '§ 6112 Advisor List', '📒', '7-yr retention + 20-biz day production + § 6708 $10K/day no cap', 'NEW'],
    ['section-6601',          '§ 6601 Interest',  '📈', 'ST + 3pp under / +5pp hot corp + daily compound + § 6404(e) abate', 'NEW'],
    ['section-475',           '§ 475 Trader MTM', '⚡', 'Trader/dealer MTM + ordinary + no wash sale + Sch C above-line', 'NEW'],
    ['section-988',           '§ 988 FX',         '💱', 'FX ordinary character + § 988(a)(1)(B) capital elect + § 1256 60/40', 'NEW'],
    ['section-303',           '§ 303 Est Redeem', '⚰️', '35% AGE threshold + tax + funeral + admin expenses + § 6166 pair', 'NEW'],
    ['section-129',           '§ 129 DCAP',       '🍼', '$5K DCAP exclusion + § 21 credit coordination + § 125 cafeteria', 'NEW'],
    ['section-134',           '§ 134 Military',   '🎖️', '§ 112 combat + BAH/BAS + § 121(d)(9) 10-yr home + § 7508 ext', 'NEW'],
    ['section-6672',          '§ 6672 TFRP',      '🚨', '100% trust fund recovery + responsible person + willful test', 'NEW'],
    ['section-6662',          '§ 6662 Accuracy',  '🎯', '20% accuracy / 40% gross val / 40% foreign + § 6664 RC defense', 'NEW'],
    ['section-67',            '§ 67 2% Misc',     '📐', 'TCJA § 67(g) suspended 2018-25 + § 67(b) exceptions + § 67(e) trust', 'NEW'],
    ['section-421',           '§ 421 ISO/ESPP',   '🔐', '§ 422 ISO 2yr/1yr + § 423 ESPP 15% + § 6039 Form 3921/3922 + AMT', 'NEW'],
    ['section-989',           '§ 989 FX QBU',     '🌐', 'Functional currency + QBU + § 987 2024 regs + DASTM high-infl', 'NEW'],
    ['section-4958',          '§ 4958 Excess Ben', '🏛️', '25%/200% excise + DP + rebuttable presumption + § 4960 21% exec', 'NEW'],
    ['section-102',           '§ 102 Gift Excl',  '🎁', '§ 1015 carryover + § 1014 step-up + Duberstein + § 102(c) emp', 'NEW'],
    ['section-362',           '§ 362 Corp Basis', '🏢', '§ 351 carryover + § 362(e)(2) NBIL anti-loss + § 1223(2) tacking', 'NEW'],
    ['section-6330',          '§ 6330 CDP Levy',  '🛡️', '30-day Form 12153 + Appeals + § 6502 SOL tolled + Tax Court', 'NEW'],
    ['section-6404',          '§ 6404 Abatement', '❌', '§ 6404(e) IRS delay + § 6404(f) advice + § 6404(g) 36-mo + FTA', 'NEW'],
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
    ['forex',           'Forex Desk',      '💱',  'FX majors, session clock, pip + position sizing', 'NEW'],

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
    ['receipts',        'Receipts',        '📂',  'Paginated, filterable receipts library — bulk auto-attach to transactions', 'NEW'],
    ['purchases',       'Purchases',       '🛒',  'Unified line-item ledger — every receipt item ∪ every CSV transaction, one row each, click back to source', 'NEW'],
    ['tax-workshop',    'Tax Workshop',    '💰',  'SE tax · home office · mileage · 1040-ES · subscriptions', null],
    ['tax-loss-harvest', 'Tax-Loss Harvest', '🧾', 'Year-end harvest suggester — ranks losers, flags wash-sale + $3k cap', 'NEW'],
    ['tax-aware-rebalance', 'Tax-Aware Rebalance', '⚖️', 'Rebalance to targets while minimizing realized gain (HIFO) or harvesting losses; reports the tax the trade triggers', 'NEW'],
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
    ['algo',            'Algo Trading',    '🤖',  '5-strategy engine: momentum / mean-reversion / ORB / Donchian / BB-squeeze. Native bracket orders, kill switch, 30-day paper lock.', 'NEW'],
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

    // — Tabs that ship in the topbar but were previously missing
    //   from TILES (so they didn't appear in the launcher OR the
    //   command palette, since both source from this array).
    ['trades',            'Trades',              '📈',  'Trade list, filtering, manual entry', null],
    ['brokers',           'Manage Brokers',      '🏦',  'CRUD on broker entities — list, edit, set default, delete', 'NEW'],
    ['businesses',        'Manage Businesses',   '🏢',  'CRUD on business entities (Schedule C) — list, edit, set default, delete', 'NEW'],
    ['broker-compare',    'Broker Comparison',   '⚔',   'Side-by-side broker P&L / KPIs', null],
    ['business-compare',  'Business Comparison', '⚔',   'Side-by-side business expense totals', null],
    ['budget',            'Budget',              '💰',  'Monthly budget targets + savings rate', null],
    ['categorize',        'Categorize',          '🗂',  'Bulk receipt category assignment queue', null],
    ['expense-dashboard', 'Expense Dashboard',   '📊',  'Expense totals + category breakdown', null],
    ['expense-calendar',  'Expense Calendar',    '🗓',  'Daily expense spend heatmap', null],
    ['file-taxes',        'File Taxes',          '🧾',  'Tax wizard — 1040 / Schedule C / Schedule E', null],
    ['note-templates',    'Note Templates',      '📝',  'Pre-canned trade journal templates', null],
    ['toast-history',     'Toast History',       '🔔',  'Persistent log of every toast notification this session (mirrored to localStorage)', 'NEW'],
    ['log-viewer',        'Log Viewer',          '📜',  'Tail the backend log file in-app — auto-refresh, level filter, free-text search', 'NEW'],
    ['about',             'About',               'ℹ️',  'App identity card — version, PID, RSS, uptime, tile/renderer/locale counts, providers, links', 'NEW'],
    ['live-feed',         'Live Feed',           '📡',  'WS firehose — every tick (Alpaca IEX/SIP, Polygon, Finnhub) + news + sentiment + disclosures + algo signals in one scrolling stream', 'NEW'],
    ['golden-stars',      'Golden Stars',        '⭐',  'stockinvest.us-style ranked recommendation leaderboard — most-recent score per symbol, click to drill into research', null],
    ['recommendation-sectors', 'Sector Heatmap', '🗺️',  '11 SPDR sector ETFs color-coded by aggregated buy/sell recommendation — spot rotating sectors at a glance', null],
    ['dcf',               'DCF Valuation',       '🧮',  'Two-stage discounted cash-flow intrinsic-value calculator — pre-fills FCF/shares from fundamentals when given a symbol', null],
    ['valuation-tools',   'Valuation Tools',     '🧰',  'Reverse DCF, Dividend Discount, Earnings Power Value, and options Wheel calculator behind one tabbed view', null],
    ['strategy-tools',    'Strategy Tools',      '🎛️',  '71 quant calculators behind one filterable tabbed view — sizing (grid/fixed-ratio/Kelly-context/futures), options desk math (Heston, POP, parity arbs, dailies), event studies (OpEx, Santa, ex-div, splits), screeners (seasonality/risk/momentum/mean-reversion), macro + valuation gauges', null],
    ['rrg',               'RRG Rotation',        '🌀',  'Relative Rotation Graph — 11 sector ETFs vs SPY on RS-Ratio / RS-Momentum axes with comet tails', null],
];

export const CATEGORIES = [
    ['live',     '// LIVE MARKETS',     ['live-dashboard','confluence','live-feed','market-gamma','live-scanner','squeeze-scanner','halts','catalysts','catalyst-correlations','uoa-stream','gamma-squeeze','vrp','iv-term','pairs-coint','htb-ranker','breadth-divergence','rvol-accel','insider-stream','insider-clusters','pead','earnings-revisions','sentiment-velocity','ipo-lockups','sp500-predict','dividend-capture','sector-timing','multi-broker','premarket','after-hours','tape','heatmap','top-news','ipo-calendar','economic-calendar','fda-calendar','market-status','earnings-call-live','symbol-changes','finnhub-search','index-constituents','crypto-markets']],
    ['trading',  '// TRADING',          ['dashboards','webull','live','paper','new-trade','plans','sizing','optimal-f','kelly','mc-trades','margin-runway','risk-reward','buying-power','margin-call','risk-gate','hotkeys','pyramid','news-event','stop-loss-best-of','stop-loss-backtest','futures-roll','squeeze-alerts','alert-rules','daily-loss-limit','drawdown-throttle','goal-tracker','trade-plan-checklist','time-in-force']],
    ['journal',  '// JOURNAL',          ['journal','ai','reviews','trade-compare','replay','tape-replay','discipline','mood','goals','clusters-trade-features','setups-by-setup','note-templates']],
    ['research', '// CHARTS & RESEARCH',['charts','multichart','research','golden-stars','recommendation-sectors','rrg','watchlists','screener','scanners','top-signals','compare','pairs','correlation','sectors','sector-rotation','breadth','fear-greed','sentiment','cohort-tilt','risk-on-off','darkpool','short-interest','vol','vol-surface','vix-term-structure','options','option-payoff','vol-smile','monte-carlo','series-smoother','pattern-discovery','regime-detector','regime-equity','american-option','fx-option','forward-vol','yield-curve-pca','yield-curve','bond-duration','carry-score','dividend-calendar','signal-decomposition','rr-butterfly','microprice','dtw','hurst','bocpd','vasicek','kalman-beta','pair-trade-calc','iv-solver','iv-rank','iv-backtest','greeks-profile','second-order-greeks','vpin','cup-and-handle','order-book-imbalance','cusum','order-flow','footprint','stress-test','chandelier-stop','vol-stop-close','atr-cone','round-levels','kyles-lambda','hawkes','kagi','three-line-break','volume-at-price','roll-spread','effective-spread','weighted-midprice','range-bar','tick-bar','volume-bar','dollar-bar','equivolume','imbalance-bar','adf-test','aroon','amihud','breadth-thrust','bb-squeeze','balance-of-power','anchored-momentum','acf','beta','brier-score','bipower-variation','bootstrap-pnl','block-bootstrap','ad-normality','arch-lm','alma','alphatrend','atr-channel','atr-trailing-stop','adl','asi','ad-oscillator','beta-shrinkage','bartlett-variance','bid-ask-volume-ratio','bollinger-band-width','bollinger-bandwidth-percentile','bollinger-percent-b','bollinger-band-distance','bollinger-oscillators','borrow-rate-indicator','breusch-pagan','burke-ratio','camarilla-pivots','breusch-godfrey','candle-strength-index','carhart-4','centered-smoothed-momentum','chaikin-oscillator','chande-dynamic-momentum','chande-kroll-stop','chande-momentum-oscillator','chande-trend-index','chande-volatility-index','chandelier-exit','cholesky','abc-pattern','absorption','favorites','triple-screen','open-type','market-profile','oi-change','ha-reversal','three-bar-reversal','range-expansion','alligator','choppiness','demarker','murrey-math','demark-pivots','cypher-pattern','earnings-cal','earnings-iv','disclosures','economy','news','crypto','forex']],
    ['reports',  '// REPORTS',          ['dashboard','trades','dcf','valuation-tools','strategy-tools','reports','r-dist','forecast','fill-quality','risk','rebalance','tax-lots','cost-basis','expenses','receipts','purchases','expense-dashboard','expense-calendar','budget','categorize','file-taxes','broker-compare','business-compare','tax-workshop','tax-loss-harvest','tax-aware-rebalance','wash-sale','calendar','accounts-overview','portfolio-allocator','risk-parity','risk-parity-solver','herfindahl','momentum-crash','marginal-var','active-share','brinson','black-litterman','currency-exposure','cov-denoiser','var-calculator','var-estimator','execution-scheduler','almgren-chriss','implementation-shortfall','deflated-sharpe','market-impact','liquidity','spread-tracker','intraday-heatmap','heatmap-dow-hour','vwap-slippage','twap','per-symbol-slippage','order-staleness','clusters-correlation','commission-optimizer','portfolio-exposure','dividend-tracker','inflation-calculator','lump-sum-vs-dca','bill-calendar','cash-flow-forecast','income-tax-estimator','compound-interest','time-value-money','roth-conversion-ladder','mortgage-payoff-vs-invest','ibond-calculator','bond-ladder','ltcg-harvesting','sequence-of-returns','rule-of-72','goal-funding','reverse-mortgage','niit-calculator','drip-simulator','vertical-spread','iron-condor','stretch-ira','tips-breakeven','yield-to-call','covered-call','real-estate-cap-rate','house-hacking','paper-tax-loss-harvest','sector-rotation-strategy','dca-simulator','dividend-aristocrats','permanent-portfolio','cape-indicator','fire-calculator','emergency-fund','savings-waterfall','net-worth-tracker','personal-balance-sheet','personal-cash-flow','financial-ratios','savings-rate','sinking-fund','zero-based-budget','fifty-thirty-twenty','envelope-budget','debt-avalanche','debt-snowball','credit-utilization','auto-loan','mortgage-amortization','mortgage-refinance','rent-vs-buy','heloc','home-maintenance','student-loan-payoff','pslf-tracker','college-529','fafsa-efc','car-tco','lease-vs-buy-car','ev-vs-ice','coast-fire','barista-fire','lean-fire','fat-fire','rmd-calculator','social-security-age','roth-vs-trad-401k','pension-lump-vs-annuity','three-fund-portfolio','bond-tent','glide-path','annuity-pv-fv','cd-ladder','i-bond','tips-bond','hysa-compare','tax-bracket-optimizer']],
    ['strategy', '// STRATEGY & AUTOMATION', ['backtest','backtest-presets','walk-forward','custom-indicators','strategy-alerts','algo','alerts','webhooks','scanner-backtest','confluence-autotrade','paper-rebalance','drawdown-cutoff','magic-formula']],
    ['community','// COMMUNITY',        ['shares','community','mentorship','boards']],
    ['admin',    '// ADMIN & DATA',     ['import','csv-wizard','exports','accounts','brokers','businesses','tags','search','settings','developer','tutorial','keyboard-shortcuts','toast-history','log-viewer','about']],
    ['tax-code','// TAX CODE',['section-1014','section-1015','section-102','section-1031','section-1033','section-1035','section-1041','section-1042','section-105','section-1058','section-1059','section-106','section-1092','section-119','section-1202','section-121','section-1212','section-1231','section-1233','section-1234','section-1239','section-1244','section-1245','section-1245-1250','section-1248','section-125','section-1250','section-1259','section-127','section-1273','section-1276','section-129','section-1291','section-1295','section-1296','section-1296-pfic','section-1297','section-1298','section-132','section-134','section-1361','section-1362','section-1366','section-1368','section-1374','section-1377','section-1400z','section-1402','section-1411','section-1445','section-1446f','section-152','section-162a1','section-162c','section-162f','section-162l','section-162m','section-163j','section-164','section-165','section-165c3','section-165d','section-165g','section-168','section-168g','section-168k','section-170','section-172','section-174','section-179','section-179d','section-195','section-197','section-199a','section-2010c','section-2032','section-2055','section-2056','section-21-cdcc','section-213','section-219','section-221','section-23','section-24','section-24-ctc','section-245a','section-248','section-250','section-2503','section-2518','section-25a','section-25c','section-25d','section-25e','section-263-tpr','section-263a','section-263c','section-269','section-269a','section-274','section-280c','section-280e','section-280f','section-280g','section-302','section-303','section-304','section-305','section-30c','section-30d','section-311','section-318','section-32','section-32-eic','section-332','section-336','section-338','section-351','section-351-721','section-355','section-357','section-362','section-367a','section-367d','section-368','section-36b','section-38','section-382','section-401a9','section-401k-hardship','section-408a','section-408d3','section-409a','section-41','section-412','section-414','section-414v','section-415','section-416','section-42','section-421','section-444','section-446','section-4501','section-451','section-457','section-45l','section-45q','section-45v','section-45w','section-45x','section-460','section-461','section-461l','section-467','section-469','section-47','section-471','section-472','section-475','section-475f','section-48','section-481','section-481a','section-482','section-483','section-48c','section-4940','section-4941','section-4942','section-4943','section-4944','section-4945','section-4958','section-4960','section-4972','section-4973','section-4974','section-4975','section-4980d','section-4980h','section-51','section-511','section-529','section-530','section-59a','section-6011','section-6015','section-6033','section-6038','section-6038a','section-6038b','section-6038d','section-6039','section-6041','section-6045','section-6045a','section-6045b','section-6048','section-6049','section-6050w','section-6051','section-6072','section-6111','section-6112','section-6159','section-6166','section-6213','section-6221','section-6321','section-6325','section-6330','section-6331','section-6404','section-6502','section-6601','section-6651','section-6654','section-6655','section-6662','section-6663','section-6664','section-6672','section-6694','section-6695','section-67','section-6700','section-6707a','section-6724','section-691','section-707','section-71-alimony','section-7122','section-72p','section-72t','section-731','section-7345','section-736','section-743','section-7430','section-7491','section-7508a','section-752','section-754','section-7701b','section-7701o','section-7702','section-7702a','section-7811','section-7872','section-7874','section-79','section-83','section-86','section-871','section-871a','section-871m','section-877a','section-882','section-884','section-894','section-897','section-901','section-901j','section-904','section-911','section-951','section-951a','section-956','section-962','section-988','section-989']],
    ['misc','// MISC',['529-roth','able-account','accountable-plan','amt-calc','augusta-rule','backdoor-roth','biz-categorizer','bond-yield-curve','charitable-planner','clean-energy-25d','congressional-trading','conservation-easement','cost-seg','crat','cross-broker-wash','crut','crypto-staking','daf','dcfsa','defined-benefit','depreciation','disabled-access','earnings-quality','education-credits','esg','espp-calc','estimates-dashboard','etf-profile','ev-credit','fbar-8938','filings-browser','film-181','finnhub-aggregate','finnhub-pattern','finnhub-sr','foreign-tax-credit','forex-988','forex-rates','gift-tax','grat','historic-rehab','historical-market-cap','home-office','hsa-max','ilit','income-1099','inherited-ira-rmd','insider-finnhub','insider-sentiment','installment-sale','institutional-13f','iso-exercise','kiddie-tax','lihtc','lobbying','meal-deduction','mega-backdoor-roth','mileage-log','mlp-k1','mtm-election','mutual-fund','news-sentiment','nol-tracker','nso-exercise','nua-strategy','partial-disposition','passive-loss','price-target','qbi-199a','qcd-tracker','qoz-tracker','qsbs-1202','quarterly-tax','rd-credit','residency-daycount','retirement-max','revenue-breakdown','roth-ladder','rsu-vest-tracker','savers-credit','scorp-calc','se-health-deduction','sec-1256','sector-heatmap','sep-ira','simple-ira','slat','solo-401k','splits-history','state-tax','str-loophole','subscriptions','supply-chain','travel-per-diem','tts-qualification','tts-scorer','unusual-options','uspto-patents','wash-sale-tracker']],
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
                   data-shortcut="focus_search"
                   autocomplete="off"
                   autocorrect="off"
                   autocapitalize="off"
                   spellcheck="false">
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

// Wire each per-category .launcher-tiles container so its child tiles can
// be drag-reordered (Trello-style, pointer-driven — see drag_reorder.js).
// Saved order lives in `tv:launcherOrder:<cat>` and is restored on every
// renderGrid() rebuild, so it survives view re-renders + reloads. New
// tiles released later append at the end of the saved sequence.
function wireLauncherDrag() {
    const grid = document.getElementById('launcher-grid');
    if (!grid) return;
    grid.querySelectorAll('.launcher-cat[data-cat] > .launcher-tiles').forEach(container => {
        const sec = container.closest('.launcher-cat');
        const cat = sec && sec.dataset.cat;
        if (!cat) return;
        initDragReorder(container, '.tile', `tv:launcherOrder:${cat}`, {
            direction: 'horizontal',
            // Only initiate drag from the tile body — clicking the
            // favorite/pin/popout action buttons stays a click.
            handleSelector: '.tile-glyph, .tile-body',
            getKey: (el) => el.dataset.view,
            toastKey: 'toast.reordered',
        });
    });
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
        return `<section class="launcher-cat" data-cat="${esc(cat)}">
            <h2>${esc(catLabel)}</h2>
            <div class="launcher-tiles">${tiles.map(renderTile).join('')}</div>
        </section>`;
    }).join('');

    // Trello-style drag-to-reorder per category, persisted to localStorage.
    // Disabled while a query is active — reordering a filtered subset would
    // silently truncate the saved order to only-visible tiles.
    if (!q) wireLauncherDrag();

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
            const nowFav = favs.isFavorite(f, id);
            showToast(
                t(nowFav ? 'toast.favorite_added' : 'toast.favorite_removed', { view: id }),
                { level: 'success' });
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
    grid.querySelectorAll('.tile-action[data-popout]').forEach(btn => {
        btn.addEventListener('click', (e) => {
            e.stopPropagation();
            const id = btn.dataset.popout;
            popoutTile(id);
        });
    });
    // Re-translate + upgrade tooltips on the freshly-rendered grid so
    // toggle-induced rebuilds (favorite star, recents repaint) carry
    // the same a11y / hover-help surface as the initial dispatch.
    try { applyUiI18n(grid); } catch (_) {}
    try { upgradeTooltips(grid); } catch (_) {}
}

/// Open a tile in a new window. In Tauri desktop builds, spawns a real
/// native WebviewWindow so the user can drag it to another monitor; in
/// browser/web mode falls back to `window.open()`. Each popout window
/// gets its own label (id + epoch) so multiple pops of the same tile
/// don't collide. The new window navigates to `#popout/<id>` which the
/// app.js dispatcher recognizes (strips chrome via `body.popout-mode`).
function popoutTile(id) {
    const route = `${location.origin}${location.pathname}#popout/${encodeURIComponent(id)}`;
    const tauri = window.__TAURI__;
    if (tauri && tauri.webviewWindow && tauri.webviewWindow.WebviewWindow) {
        const label = `tile-${id}-${Date.now()}`;
        try {
            new tauri.webviewWindow.WebviewWindow(label, {
                url: `${location.pathname}#popout/${encodeURIComponent(id)}`,
                title: id,
                width: 1100,
                height: 720,
                focus: true,
            });
            return;
        } catch (e) {
            console.warn('[popout] tauri create failed, falling back to window.open:', e);
        }
    }
    window.open(route, '_blank', 'width=1100,height=720,menubar=no,toolbar=no,location=no');
}

// Map of tile view-id → registered shortcut id. When present, the tile
// gets `data-shortcut="<id>"` so the tooltip augmenter appends the
// keyboard chip (e.g. "Trades  (⌘⌥T)") on hover automatically.
const TILE_SHORTCUTS = {
    'trades':       'nav_trades',
    'journal':      'nav_journal',
    'dashboard':    'nav_dashboard',
    'watchlists':   'nav_watchlists',
    'charts':       'nav_charts',
    'live':         'nav_live',
    'reports':      'nav_reports',
    'live-scanner': 'nav_scanner',
    'after-hours':  'nav_after_hours',
    'launcher':     'go_home',
};

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
    const shortcutAttr = TILE_SHORTCUTS[id] ? ` data-shortcut="${esc(TILE_SHORTCUTS[id])}"` : '';
    return `<button class="tile" data-view="${esc(id)}"${shortcutAttr} tabindex="0">
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
            <span class="tile-action" data-popout="${esc(id)}"
                  data-tip="view.launcher.tile.popout"
                  data-i18n-aria-label="view.launcher.tile.popout">↗</span>
        </span>
    </button>`;
}
