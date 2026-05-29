// Launcher — single tile grid replacing the 77-tab strip. Categorized,
// searchable, keyboard-navigable. Becomes the default landing under "Home".

import { go } from '../app.js';
import { esc } from '../util.js';
import { matchesQuery } from '../_pure.js';
import * as favs from '../_favorites_storage.js';
import * as dashStore from '../_dashboards_storage.js';

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
    ['iv-backtest',     'IV Backtest',      '🎲', 'Earnings-straddle long/short backtester w/ recommendation', null],
    ['order-book-imbalance', 'OBI',         '⚖️', 'Level-2 bid/ask imbalance + directional bias', null],
    ['cusum',           'CUSUM',            '🔔', 'Page-Hinkley change-point detector for regime shifts', null],
    ['order-flow',      'Order Flow',       '🌀', 'Lee-Ready aggressor classification + cumulative flow', null],
    ['footprint',       'Footprint',        '🦶', 'Bid/ask volume + delta per price level (Sierra-style)', null],
    ['stress-test',     'Stress Test',      '🧨', 'Portfolio P&amp;L heatmap under price × IV × time shocks', null],
    ['chandelier-stop', 'Chandelier Stop',  '🪝', 'LeBeau ATR-based trailing stop (ratcheted, with trigger detection)', null],
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
    ['tutorial',        'Tutorial',        '❓',  'In-app guide (also `?` hotkey)',   null],
];

export const CATEGORIES = [
    ['live',     '// LIVE MARKETS',     ['live-scanner','halts','catalysts','premarket','tape','heatmap']],
    ['trading',  '// TRADING',          ['dashboards','webull','live','paper','new-trade','plans','sizing','optimal-f','kelly','risk-gate','hotkeys','pyramid','news-event','stop-loss-best-of','squeeze-alerts','alert-rules','daily-loss-limit','drawdown-throttle','goal-tracker','trade-plan-checklist','time-in-force']],
    ['journal',  '// JOURNAL',          ['journal','ai','reviews','trade-compare','replay','tape-replay','discipline','mood','goals','clusters-trade-features','setups-by-setup']],
    ['research', '// CHARTS & RESEARCH',['charts','research','watchlists','screener','scanners','top-signals','compare','pairs','correlation','sectors','sector-rotation','breadth','fear-greed','sentiment','cohort-tilt','darkpool','short-interest','vol','vol-surface','options','option-payoff','vol-smile','monte-carlo','series-smoother','pattern-discovery','regime-detector','regime-equity','american-option','fx-option','forward-vol','yield-curve-pca','dividend-calendar','signal-decomposition','rr-butterfly','microprice','dtw','hurst','bocpd','vasicek','kalman-beta','pair-trade-calc','iv-solver','iv-rank','iv-backtest','greeks-profile','second-order-greeks','vpin','cup-and-handle','order-book-imbalance','cusum','order-flow','footprint','stress-test','chandelier-stop','vol-stop-close','triple-screen','open-type','market-profile','oi-change','ha-reversal','three-bar-reversal','range-expansion','alligator','choppiness','demarker','murrey-math','demark-pivots','cypher-pattern','earnings-cal','earnings-iv','disclosures','economy','news','crypto']],
    ['reports',  '// REPORTS',          ['dashboard','reports','r-dist','forecast','fill-quality','risk','rebalance','tax-lots','expenses','tax-workshop','calendar','accounts-overview','portfolio-allocator','cov-denoiser','var-calculator','var-estimator','execution-scheduler','almgren-chriss','implementation-shortfall','deflated-sharpe','market-impact','liquidity','spread-tracker','intraday-heatmap','vwap-slippage','twap','per-symbol-slippage','order-staleness','clusters-correlation']],
    ['strategy', '// STRATEGY & AUTOMATION', ['backtest','backtest-presets','walk-forward','custom-indicators','strategy-alerts','alerts','webhooks']],
    ['community','// COMMUNITY',        ['shares','community','mentorship','boards']],
    ['admin',    '// ADMIN & DATA',     ['import','csv-wizard','exports','accounts','tags','search','settings','developer','tutorial']],
];

let lastQuery = '';

export async function renderLauncher(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title">// HOME
            <input id="launcher-q" type="search" placeholder="filter tiles…  (Esc to clear)" autocomplete="off">
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
    grid.innerHTML = CATEGORIES.map(([cat, label, ids]) => {
        const tiles = ids
            .map(id => byId.get(id))
            .filter(Boolean)
            .filter(t => !q || matchesQuery(t, q));
        if (!tiles.length) return '';
        return `<section class="launcher-cat">
            <h2>${esc(label)}</h2>
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
        });
    });
    grid.querySelectorAll('.tile-action[data-pin]').forEach(btn => {
        btn.addEventListener('click', (e) => {
            e.stopPropagation();
            const id = btn.dataset.pin;
            let s = dashStore.loadState();
            const active = dashStore.getActiveDashboard(s);
            if (!active) return;
            s = dashStore.addTile(s, active.id, id);
            dashStore.saveState(s);
            btn.textContent = '✓';
            setTimeout(() => { btn.textContent = '📌'; }, 1200);
        });
    });
}

function renderTile([id, label, glyph, desc, badge]) {
    const fState = favs.loadState();
    const fav = favs.isFavorite(fState, id);
    return `<button class="tile" data-view="${esc(id)}" tabindex="0">
        <span class="tile-glyph">${glyph}</span>
        <span class="tile-body">
            <span class="tile-label">${esc(label)}${badge ? ` <span class="tile-badge">${esc(badge)}</span>` : ''}</span>
            <span class="tile-desc">${esc(desc)}</span>
        </span>
        <span class="tile-actions">
            <span class="tile-action ${fav ? 'tile-fav-on' : 'tile-fav-off'}" data-fav="${esc(id)}"
                  title="${fav ? 'Unfavorite' : 'Favorite'}">${fav ? '★' : '☆'}</span>
            <span class="tile-action" data-pin="${esc(id)}"
                  title="Pin to active dashboard">📌</span>
        </span>
    </button>`;
}
