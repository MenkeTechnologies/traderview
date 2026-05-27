// Launcher — single tile grid replacing the 77-tab strip. Categorized,
// searchable, keyboard-navigable. Becomes the default landing under "Home".

import { go } from '../app.js';
import { esc } from '../util.js';

// (view-id, label, glyph, description, badge | null)
// `badge` shows a small chip — "LIVE" for streaming tiles, etc.
const TILES = [
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
    ['hotkeys',         'Hotkeys',         '⌨️',  'Keyboard shortcuts editor',        null],

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
    ['earnings-cal',    'Earnings Cal',    '📅',  'Earnings calendar w/ alerts',      null],
    ['earnings-iv',     'Earnings IV',     '💥',  'Pre-ER IV crush + straddle EV',    null],
    ['disclosures',     'Disclosures',     '📑',  'Form 4, 13D/G, 8-K stream',        null],
    ['economy',         'Economy',         '🏛',  'Economic calendar + macro',        null],
    ['news',            'News',            '📰',  'Per-symbol news + sentiment',      null],
    ['crypto',          'Crypto',          '🪙',  'Crypto markets snapshot',          null],

    // — Reports & analytics ——————————————————————————————————
    ['dashboard',       'Dashboard',       '🏠',  'Overview + equity + world markets', null],
    ['reports',         'Reports',         '📊',  '17 TraderVue-style reports',       null],
    ['r-dist',          'R-Multiple',      '📐',  'R distribution histogram',         null],
    ['forecast',        'Equity Forecast', '🔮',  'Monte Carlo equity projection',    null],
    ['fill-quality',    'Fill Quality',    '🎯',  'TCA — slippage vs NBBO',           null],
    ['risk',            'Risk',            '🛟',  'Greeks / Beta / VaR',              null],
    ['rebalance',       'Rebalance',       '⚖️',  'Portfolio rebalance helper',       null],
    ['tax-lots',        'Tax Lots',        '💸',  'Lot-by-lot cost basis',            null],
    ['expenses',        'Expenses',        '🧾',  'Trading expenses tracking',        null],
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
];

const CATEGORIES = [
    ['live',     '// LIVE MARKETS',     ['live-scanner','halts','catalysts','premarket','tape','heatmap']],
    ['trading',  '// TRADING',          ['webull','live','paper','new-trade','plans','sizing','hotkeys']],
    ['journal',  '// JOURNAL',          ['journal','ai','reviews','trade-compare','replay','tape-replay','discipline','mood','goals']],
    ['research', '// CHARTS & RESEARCH',['charts','research','watchlists','screener','scanners','top-signals','compare','pairs','correlation','sectors','sector-rotation','breadth','fear-greed','sentiment','darkpool','short-interest','vol','vol-surface','options','earnings-cal','earnings-iv','disclosures','economy','news','crypto']],
    ['reports',  '// REPORTS',          ['dashboard','reports','r-dist','forecast','fill-quality','risk','rebalance','tax-lots','expenses','calendar','accounts-overview']],
    ['strategy', '// STRATEGY & AUTOMATION', ['backtest','backtest-presets','walk-forward','custom-indicators','strategy-alerts','alerts','webhooks']],
    ['community','// COMMUNITY',        ['shares','community','mentorship','boards']],
    ['admin',    '// ADMIN & DATA',     ['import','csv-wizard','exports','accounts','tags','search','settings','developer']],
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

    grid.querySelectorAll('.tile[data-view]').forEach(el => {
        el.addEventListener('click', () => go(el.dataset.view));
        el.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' || e.key === ' ') {
                e.preventDefault();
                go(el.dataset.view);
            }
        });
    });
}

function matchesQuery(t, q) {
    return t[1].toLowerCase().includes(q)
        || t[0].toLowerCase().includes(q)
        || (t[3] || '').toLowerCase().includes(q);
}

function renderTile([id, label, glyph, desc, badge]) {
    return `<button class="tile" data-view="${esc(id)}" tabindex="0">
        <span class="tile-glyph">${glyph}</span>
        <span class="tile-body">
            <span class="tile-label">${esc(label)}${badge ? ` <span class="tile-badge">${esc(badge)}</span>` : ''}</span>
            <span class="tile-desc">${esc(desc)}</span>
        </span>
    </button>`;
}
