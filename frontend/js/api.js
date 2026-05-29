// API client. Resolves base URL + token differently for desktop vs web.

let baseUrl = '';
let token = '';

export async function initApi() {
    if (window.__TAURI__) {
        const cfg = await window.__TAURI__.core.invoke('get_api_config');
        baseUrl = cfg.base_url;
        token = cfg.token;
    } else {
        baseUrl = '';
        try { token = localStorage.getItem('tv-token') || ''; }
        catch (_) { token = ''; /* SecurityError (private mode etc.) */ }
    }
    // Expose for the error reporter (loaded as a non-importing script) and any
    // other module that needs to build WebSocket URLs synchronously.
    window.__tvApiBase = baseUrl;
    window.__tvApiToken = token;
    return { baseUrl, hasToken: !!token };
}

/// Return the absolute API base URL (e.g. `http://127.0.0.1:54321` in Tauri,
/// `` in web mode). Always usable for fetch and as the source for wsBase().
export function httpBase() { return baseUrl; }

/// Return the WebSocket base derived from baseUrl. Handles both http→ws and
/// https→wss; for web mode (empty base) it falls back to the page's origin.
export function wsBase() {
    if (baseUrl) return baseUrl.replace(/^http/i, 'ws');
    return location.origin.replace(/^http/i, 'ws');
}

/// Build a full WebSocket URL: `wsUrl('/api/ws/halts')` →
/// `ws://127.0.0.1:54321/api/ws/halts` (Tauri) or `ws://host/api/ws/halts` (web).
export function wsUrl(path) {
    const b = wsBase();
    return `${b}${path.startsWith('/') ? '' : '/'}${path}`;
}

export function setToken(t) {
    token = t;
    window.__tvApiToken = t;
    if (!window.__TAURI__) {
        try { localStorage.setItem('tv-token', t); } catch (_) {}
    }
}

export function clearToken() {
    token = '';
    window.__tvApiToken = '';
    if (!window.__TAURI__) {
        try { localStorage.removeItem('tv-token'); } catch (_) {}
    }
}

async function request(path, opts = {}) {
    const method = (opts.method || 'GET').toUpperCase();
    const headers = Object.assign({}, opts.headers || {});
    if (token) headers['Authorization'] = `Bearer ${token}`;
    if (opts.body && !(opts.body instanceof FormData) && !headers['Content-Type']) {
        headers['Content-Type'] = 'application/json';
    }
    let res;
    try {
        res = await fetch(`${baseUrl}/api${path}`, Object.assign({}, opts, { headers }));
    } catch (netErr) {
        // Network-level failure (server down, CORS, DNS) — also report.
        try {
            const m = await import('./error_reporter.js');
            m.reportApiFail(method, path, 0, String(netErr));
        } catch (_) {}
        throw netErr;
    }
    if (res.status === 401) {
        clearToken();
        throw new ApiError(401, 'unauthorized');
    }
    if (!res.ok) {
        // Read body once so we can both report it and surface a useful Error.
        let bodyText = '';
        try { bodyText = await res.text(); } catch (_) {}
        let msg = res.statusText;
        try { msg = JSON.parse(bodyText).error || msg; } catch (_) {}
        // Don't recursively report errors from /client-errors itself.
        if (path !== '/client-errors') {
            try {
                const m = await import('./error_reporter.js');
                m.reportApiFail(method, path, res.status, bodyText);
            } catch (_) {}
        }
        throw new ApiError(res.status, msg);
    }
    if (res.status === 204) return null;
    const ct = res.headers.get('content-type') || '';
    return ct.includes('application/json') ? res.json() : res.text();
}

/// Fetch a path that returns binary/text content (CSV, HTML) and return a Blob.
/// Honors the bearer token and 401 handling identically to request().
export async function apiFetchBlob(path) {
    const headers = {};
    if (token) headers['Authorization'] = `Bearer ${token}`;
    const res = await fetch(`${baseUrl}/api${path}`, { headers });
    if (res.status === 401) { clearToken(); throw new ApiError(401, 'unauthorized'); }
    if (!res.ok) {
        let msg = res.statusText;
        try { msg = (await res.json()).error || msg; } catch (_) { /* not json */ }
        throw new ApiError(res.status, msg);
    }
    return res.blob();
}

const qs = (obj) => {
    const parts = [];
    for (const [k, v] of Object.entries(obj || {})) {
        if (v === undefined || v === null || v === '') continue;
        parts.push(`${encodeURIComponent(k)}=${encodeURIComponent(v)}`);
    }
    return parts.length ? '?' + parts.join('&') : '';
};

export class ApiError extends Error {
    constructor(status, msg) { super(msg); this.status = status; }
}

export const api = {
    // auth
    config: () => request('/config'),
    me: () => request('/auth/me'),
    login: (email, password) =>
        request('/auth/login', { method: 'POST', body: JSON.stringify({ email, password }) }),
    register: (email, password, display_name) =>
        request('/auth/register', { method: 'POST', body: JSON.stringify({ email, password, display_name }) }),

    // accounts
    accounts: () => request('/accounts'),
    createAccount: (broker, name, base_currency = 'USD') =>
        request('/accounts', { method: 'POST', body: JSON.stringify({ broker, name, base_currency }) }),
    deleteAccount: (id) => request(`/accounts/${id}`, { method: 'DELETE' }),

    // trades
    trades: (account_id, filter = {}) =>
        request(`/trades${qs(Object.assign({ account_id, limit: 200 }, filter))}`),
    trade: (id) => request(`/trades/${id}`),
    deleteTrade: (id) => request(`/trades/${id}`, { method: 'DELETE' }),
    rollupTrades: (account_id) => request(`/trades/rollup?account_id=${account_id}`, { method: 'POST' }),
    setRisk: (trade_id, body) =>
        request(`/trades/${trade_id}/risk`, { method: 'POST', body: JSON.stringify(body) }),
    splitTrade: (id) => request(`/trades/${id}/split`, { method: 'POST' }),
    mergeTrades: (trade_ids) =>
        request('/trades/merge', { method: 'POST', body: JSON.stringify({ trade_ids }) }),
    bulkTrades: (trade_ids, action, extras = {}) =>
        request('/trades/bulk', {
            method: 'POST',
            body: JSON.stringify(Object.assign({ trade_ids, action }, extras)),
        }),
    closeExpiredOptions: (account_id) =>
        request(`/trades/close-expired-options?account_id=${account_id}`, { method: 'POST' }),

    // executions
    executions: (account_id) => request(`/executions?account_id=${account_id}`),
    executionsForTrade: (trade_id) => request(`/trades/${trade_id}/executions`),
    createExecution: (body) =>
        request('/executions', { method: 'POST', body: JSON.stringify(body) }),
    updateExecution: (id, body) =>
        request(`/executions/${id}`, { method: 'PATCH', body: JSON.stringify(body) }),
    addExecutionToTrade: (trade_id, body) =>
        request(`/trades/${trade_id}/executions`, { method: 'POST', body: JSON.stringify(body) }),
    deleteExecution: (id) => request(`/executions/${id}`, { method: 'DELETE' }),

    // tags
    tags: () => request('/tags'),
    createTag: (name, color) =>
        request('/tags', { method: 'POST', body: JSON.stringify({ name, color }) }),
    deleteTag: (id) => request(`/tags/${id}`, { method: 'DELETE' }),
    tagsForTrade: (trade_id) => request(`/trades/${trade_id}/tags`),
    attachTag: (trade_id, tag_id) =>
        request(`/trades/${trade_id}/tags`, { method: 'POST', body: JSON.stringify({ tag_id }) }),
    detachTag: (trade_id, tag_id) =>
        request(`/trades/${trade_id}/tags/${tag_id}`, { method: 'DELETE' }),

    // journal
    journalForDay: (day) => request(`/journal/day/${day}`),
    journalForTrade: (trade_id) => request(`/journal/trade/${trade_id}`),
    journalGeneral: () => request('/journal/general'),
    createJournal: (body) => request('/journal', { method: 'POST', body: JSON.stringify(body) }),
    updateJournal: (id, body) =>
        request(`/journal/${id}`, { method: 'POST', body: JSON.stringify(body) }),
    deleteJournal: (id) => request(`/journal/${id}`, { method: 'DELETE' }),

    // screenshots (multipart)
    screenshotsForTrade: (trade_id) => request(`/trades/${trade_id}/screenshots`),
    uploadScreenshot: (trade_id, file, caption = '') => {
        const fd = new FormData();
        fd.append('file', file);
        fd.append('caption', caption);
        return request(`/trades/${trade_id}/screenshots`, { method: 'POST', body: fd });
    },
    screenshotUrl: (id) => `${baseUrl}/api/screenshots/${id}/bytes`,
    deleteScreenshot: (id) => request(`/screenshots/${id}`, { method: 'DELETE' }),

    // imports
    importList: (account_id) => request(`/imports?account_id=${account_id}`),
    importSources: () => request('/imports/sources'),
    upload: (account_id, source, file) => {
        const fd = new FormData();
        fd.append('account_id', account_id);
        fd.append('source', source);
        fd.append('file', file);
        return request('/imports', { method: 'POST', body: fd });
    },

    // reports
    overview: (account_id) => request(`/reports/overview?account_id=${account_id}`),
    bySymbol: (account_id) => request(`/reports/by-symbol?account_id=${account_id}`),
    bySide: (account_id) => request(`/reports/by-side?account_id=${account_id}`),
    byAssetClass: (account_id) => request(`/reports/by-asset-class?account_id=${account_id}`),
    byDow: (account_id) => request(`/reports/by-day-of-week?account_id=${account_id}`),
    byHour: (account_id) => request(`/reports/by-hour?account_id=${account_id}`),
    byHold: (account_id) => request(`/reports/by-hold?account_id=${account_id}`),
    byMonth: (account_id) => request(`/reports/by-month?account_id=${account_id}`),
    rDist: (account_id) => request(`/reports/r-distribution?account_id=${account_id}`),
    streaks: (account_id) => request(`/reports/streaks?account_id=${account_id}`),
    comparison: (account_id) => request(`/reports/comparison?account_id=${account_id}`),
    exitEff: (account_id) => request(`/reports/exit-efficiency?account_id=${account_id}`),
    commissions: (account_id) => request(`/reports/commissions?account_id=${account_id}`),
    liquidity: (account_id, adv = '') => request(`/reports/liquidity${qs({ account_id, adv })}`),
    risk: (account_id) => request(`/reports/risk?account_id=${account_id}`),
    drawdown: (account_id, starting_cash) =>
        request(`/reports/drawdown${qs({ account_id, starting_cash })}`),
    riskAdjusted: (account_id, starting_cash) =>
        request(`/reports/risk-adjusted${qs({ account_id, starting_cash })}`),
    calendar: (account_id) => request(`/reports/calendar?account_id=${account_id}`),
    summary: (account_id) => request(`/stats/summary?account_id=${account_id}`),
    equity: (account_id, starting_cash) =>
        request(`/stats/equity${qs({ account_id, starting_cash })}`),

    // --- expenses -------------------------------------------------------
    expenseAccounts: () => request('/expense/accounts'),
    createExpenseAccount: (body) =>
        request('/expense/accounts', { method: 'POST', body: JSON.stringify(body) }),
    expenseCategories: () => request('/expense/categories'),
    expenseTransactions: (params = {}) => {
        const q = new URLSearchParams();
        Object.entries(params).forEach(([k, v]) => {
            if (v !== null && v !== undefined && v !== '') q.set(k, v);
        });
        const s = q.toString();
        return request(`/expense/transactions${s ? '?' + s : ''}`);
    },
    updateExpenseTransaction: (id, patch) =>
        request(`/expense/transactions/${id}`, { method: 'PATCH', body: JSON.stringify(patch) }),
    importExpense: (account_id, source, file) => {
        const fd = new FormData();
        fd.append('account_id', account_id);
        fd.append('source', source);
        fd.append('file', file, file.name);
        return request('/expense/import', { method: 'POST', body: fd });
    },
    expenseRules: () => request('/expense/rules'),
    createExpenseRule: (body) =>
        request('/expense/rules', { method: 'POST', body: JSON.stringify(body) }),
    deleteExpenseRule: (id) => request(`/expense/rules/${id}`, { method: 'DELETE' }),
    seedExpenseRules: () => request('/expense/rules/seed', { method: 'POST' }),

    // --- receipts -------------------------------------------------------
    receipts: () => request('/expense/receipts'),
    uploadReceipt: (file) => {
        const fd = new FormData();
        fd.append('file', file, file.name);
        return request('/expense/receipts', { method: 'POST', body: fd });
    },
    receiptMeta: (id) => request(`/expense/receipts/${id}/meta`),
    receiptMatches: (id) => request(`/expense/receipts/${id}/matches`),
    attachReceipt: (id, transaction_id) =>
        request(`/expense/receipts/${id}/attach`, {
            method: 'POST',
            body: JSON.stringify({ transaction_id }),
        }),
    receiptBlobUrl: (id) => `${baseUrl}/api/expense/receipts/${id}`,

    // --- schedule C report ---------------------------------------------
    scheduleC: (year) => request(`/expense/report/schedule_c${year ? `?year=${year}` : ''}`),

    // --- tax workshop calculators (pure compute, no DB writes) ---------
    calcSelfEmploymentTax: (body) =>
        request('/expense/calc/self-employment-tax', { method: 'POST', body: JSON.stringify(body) }),
    calcHomeOffice: (body) =>
        request('/expense/calc/home-office', { method: 'POST', body: JSON.stringify(body) }),
    calcMileage: (trips) =>
        request('/expense/calc/mileage', { method: 'POST', body: JSON.stringify({ trips }) }),
    calcQuarterlyTax: (body) =>
        request('/expense/calc/quarterly-tax', { method: 'POST', body: JSON.stringify(body) }),
    detectSubscriptions: () => request('/expense/subscriptions/detect'),

    // --- pre-trade risk gate -------------------------------------------
    riskRules: (account_id) =>
        request(`/risk-gate/rules${account_id ? `?account_id=${account_id}` : ''}`),
    createRiskRule: (body) =>
        request('/risk-gate/rules', { method: 'POST', body: JSON.stringify(body) }),
    deleteRiskRule: (id) => request(`/risk-gate/rules/${id}`, { method: 'DELETE' }),
    toggleRiskRule: (id, enabled) =>
        request(`/risk-gate/rules/${id}/toggle`, { method: 'POST', body: JSON.stringify({ enabled }) }),
    evaluateProposedTrade: (account_id, proposed) =>
        request('/risk-gate/evaluate', {
            method: 'POST',
            body: JSON.stringify({ account_id, proposed }),
        }),
    installRiskPreset: (preset, account_id = null) =>
        request('/risk-gate/rules/install-preset', {
            method: 'POST',
            body: JSON.stringify({ preset, account_id }),
        }),
    riskFires: (limit = 100) => request(`/risk-gate/fires?limit=${limit}`),
    riskFiresByRule: (days = 30) => request(`/risk-gate/fires/by-rule?days=${days}`),
    riskKillSwitchState: () => request('/risk-gate/kill-switch'),

    // mentorships
    mentorshipRequest: (mentor_id, scope = 'read') =>
        request('/mentorships', { method: 'POST', body: JSON.stringify({ mentor_id, scope }) }),
    mentors: () => request('/mentorships/mentors'),
    mentees: () => request('/mentorships/mentees'),
    acceptMentorship: (id) => request(`/mentorships/${id}/accept`, { method: 'POST' }),
    revokeMentorship: (id) => request(`/mentorships/${id}`, { method: 'DELETE' }),

    // shares
    sharesPublic: () => request('/shares/public'),
    sharesMine: () => request('/shares'),
    createShare: (body) => request('/shares', { method: 'POST', body: JSON.stringify(body) }),
    deleteShare: (id) => request(`/shares/${id}`, { method: 'DELETE' }),
    viewShared: (slug) => request(`/shared/${slug}`),
    comments: (slug) => request(`/shared/${slug}/comments`),
    postComment: (slug, body_md, parent_id = null) =>
        request(`/shared/${slug}/comments`, {
            method: 'POST',
            body: JSON.stringify({ body_md, parent_id }),
        }),
    deleteComment: (id) => request(`/comments/${id}`, { method: 'DELETE' }),

    // forum
    forumCategories: () => request('/forum/categories'),
    forumThreadsIn: (slug) => request(`/forum/threads/category/${slug}`),
    forumThreadBySlug: (cat_slug, thread_slug) =>
        request(`/forum/by-slug/${cat_slug}/${thread_slug}`),
    forumPosts: (thread_id) => request(`/forum/threads/${thread_id}/posts`),
    forumCreateThread: (category_id, title, body_md) =>
        request('/forum/threads', { method: 'POST', body: JSON.stringify({ category_id, title, body_md }) }),
    forumCreatePost: (thread_id, body_md) =>
        request(`/forum/threads/${thread_id}/posts`, { method: 'POST', body: JSON.stringify({ body_md }) }),
    forumBumpView: (thread_id) =>
        request(`/forum/threads/${thread_id}/view`, { method: 'POST' }),

    // charts (price bars)
    bars: (symbol, interval, from, to) =>
        request(`/bars/${encodeURIComponent(symbol)}${qs({ interval, from, to })}`),

    // global markets snapshot
    marketsSnapshot: () => request('/markets/snapshot'),

    // live ticks (Finnhub-backed real-time scanner)
    configureLiveTicks: (body) =>
        request('/ticks/configure', { method: 'POST', body: JSON.stringify(body) }),
    liveTicksSnapshot: () => request('/ticks/snapshot'),

    // webull broker (read-only)
    connectWebull: (body) =>
        request('/webull/connect', { method: 'POST', body: JSON.stringify(body) }),
    webullSnapshot: () => request('/webull/snapshot'),

    // watchlists
    watchlists:        () => request('/watchlists'),
    createWatchlist:   (name) => request('/watchlists', { method: 'POST', body: JSON.stringify({ name }) }),
    renameWatchlist:   (id, name) => request(`/watchlists/${id}`, { method: 'POST', body: JSON.stringify({ name }) }),
    deleteWatchlist:   (id) => request(`/watchlists/${id}`, { method: 'DELETE' }),
    watchlistSymbols:  (id) => request(`/watchlists/${id}/symbols`),
    addWatchlistSym:   (id, symbol) => request(`/watchlists/${id}/symbols`, { method: 'POST', body: JSON.stringify({ symbol }) }),
    removeWatchlistSym:(id, symbol) => request(`/watchlists/${id}/symbols/${encodeURIComponent(symbol)}`, { method: 'DELETE' }),
    watchlistQuotes:   (id) => request(`/watchlists/${id}/quotes`),

    // research (per symbol)
    quote:           (sym) => request(`/symbols/${encodeURIComponent(sym)}/quote`),
    symbolSignals:   (sym, days = 365) => request(`/symbols/${encodeURIComponent(sym)}/signals${qs({ days })}`),
    symbolNews:      (sym, count = 20) => request(`/symbols/${encodeURIComponent(sym)}/news${qs({ count })}`),
    symbolEarnings:  (sym) => request(`/symbols/${encodeURIComponent(sym)}/earnings`),
    symbolDividends: (sym) => request(`/symbols/${encodeURIComponent(sym)}/dividends`),
    symbolRecs:      (sym) => request(`/symbols/${encodeURIComponent(sym)}/recommendations`),
    symbolInsiders:  (sym) => request(`/symbols/${encodeURIComponent(sym)}/insiders`),
    symbolFundamentals: (sym) => request(`/symbols/${encodeURIComponent(sym)}/fundamentals`),
    symbolHolders:   (sym) => request(`/symbols/${encodeURIComponent(sym)}/holders`),

    // screener / top signals
    screenerRun: (opts = {}) => request(`/screener/run${qs(opts)}`),
    topSignals:  (side = 'buy', watchlist_id = null, limit = 25) =>
        request(`/screener/top${qs({ side, watchlist_id, limit })}`),

    // scanners (Warrior/Zendoo presets)
    scanRun: (preset, watchlist_id = null, limit = 50) =>
        request(`/scans/run${qs({ preset, watchlist_id, limit })}`),

    // sectors
    sectors: () => request('/sectors'),

    // paper trading
    paperAccounts:  () => request('/paper/accounts'),
    paperEnsure:    () => request('/paper/accounts', { method: 'POST' }),
    paperReset:     (id, starting_cash) =>
        request(`/paper/accounts/${id}/reset`, { method: 'POST', body: JSON.stringify({ starting_cash }) }),
    paperOrders:    (id, limit = 100) => request(`/paper/accounts/${id}/orders${qs({ limit })}`),
    paperSubmit:    (id, req) =>
        request(`/paper/accounts/${id}/orders`, { method: 'POST', body: JSON.stringify(req) }),
    paperPositions: (id) => request(`/paper/accounts/${id}/positions`),

    // alerts
    alerts:        () => request('/alerts'),
    createAlert:   (body) => request('/alerts', { method: 'POST', body: JSON.stringify(body) }),
    deleteAlert:   (id) => request(`/alerts/${id}`, { method: 'DELETE' }),
    toggleAlert:   (id, enabled) =>
        request(`/alerts/${id}/toggle`, { method: 'POST', body: JSON.stringify({ enabled }) }),
    markAlertFired:(id) => request(`/alerts/${id}/fired`, { method: 'POST' }),

    // hotkeys
    hotkeys:       () => request('/hotkeys'),
    upsertHotkey:  (body) => request('/hotkeys', { method: 'POST', body: JSON.stringify(body) }),
    deleteHotkey:  (id) => request(`/hotkeys/${id}`, { method: 'DELETE' }),

    // earnings IV scanner
    ivScan:        (watchlist_id = null, horizon_days = 7, limit = 50) =>
        request(`/iv/scan${qs({ watchlist_id, horizon_days, limit })}`),
    ivSymbol:      (sym) => request(`/iv/symbols/${encodeURIComponent(sym)}`),

    // disclosures (insider Form 4 + Senate / House STOCK Act)
    disclosures:        (kind = null, symbol = null, limit = 200) =>
        request(`/disclosures${qs({ kind, symbol, limit })}`),
    disclosuresPollNow: () => request('/disclosures/poll', { method: 'POST' }),
    disclosureWatchers: () => request('/disclosures/watchers'),
    createDisclosureWatcher: (body) =>
        request('/disclosures/watchers', { method: 'POST', body: JSON.stringify(body) }),
    deleteDisclosureWatcher: (id) =>
        request(`/disclosures/watchers/${id}`, { method: 'DELETE' }),

    // sentiment-as-a-feed (WSB + StockTwits, optional X)
    sentimentFeed:        (limit = 200)            => request(`/sentiment/feed${qs({ limit })}`),
    sentimentRanked:      (hours = 1, limit = 50)  => request(`/sentiment/ranked${qs({ hours, limit })}`),
    sentimentPollNow:     ()                       => request('/sentiment/poll', { method: 'POST' }),
    sentimentForSymbol:   (sym, hours = 24, limit = 100) =>
        request(`/sentiment/symbol/${encodeURIComponent(sym)}${qs({ hours, limit })}`),
    sentimentSeries:      (sym, hours = 168) =>
        request(`/sentiment/series/${encodeURIComponent(sym)}${qs({ hours })}`),

    // Finviz-style sector heatmap
    heatmap: () => request('/heatmap'),

    // Options chain + Greeks
    options:     (sym, expiration = null) => request(`/options/${encodeURIComponent(sym)}${qs({ expiration })}`),
    greeksCalc:  (params) => request(`/greeks${qs(params)}`),

    // Crypto (CoinGecko + blockchain.com)
    cryptoMarkets:  (n = 100) => request(`/crypto/markets${qs({ n })}`),
    cryptoGlobal:   () => request('/crypto/global'),
    cryptoBtcChain: () => request('/crypto/btc/chain'),

    // Strategy backtest engine
    backtestRun: (body) => request('/backtest/run', { method: 'POST', body: JSON.stringify(body) }),

    // Economic calendar
    economyCalendar: (days = 60, importance = 'medium') =>
        request(`/economy/calendar${qs({ days, importance })}`),

    // Pairs / correlation
    correlationMatrix: (symbols, days = 90) =>
        request(`/analysis/correlation${qs({ symbols, days })}`),
    pairAnalysis: (a, b, days = 180) =>
        request(`/analysis/pair${qs({ a, b, days })}`),

    // Short interest
    shortSymbol: (sym) => request(`/short/symbol/${encodeURIComponent(sym)}`),
    shortFinra:  (sym, days = 30) => request(`/short/finra/${encodeURIComponent(sym)}${qs({ days })}`),
    shortRanked: (watchlist_id = null) => request(`/short/ranked${qs({ watchlist_id })}`),

    // Dark pool / off-exchange volume
    darkpoolSymbol: (sym, days = 30) => request(`/darkpool/symbol/${encodeURIComponent(sym)}${qs({ days })}`),
    darkpoolRanked: (watchlist_id = null, days = 30) => request(`/darkpool/ranked${qs({ watchlist_id, days })}`),

    // Volatility / yields / DXY
    volVix:    () => request('/vol/vix'),
    volYields: () => request('/vol/yields'),
    volDollar: () => request('/vol/dollar'),

    // Webhooks (Discord/Slack/generic)
    webhooks:        () => request('/webhooks'),
    createWebhook:   (body) => request('/webhooks', { method: 'POST', body: JSON.stringify(body) }),
    deleteWebhook:   (id) => request(`/webhooks/${id}`, { method: 'DELETE' }),
    toggleWebhook:   (id, enabled) =>
        request(`/webhooks/${id}/toggle`, { method: 'POST', body: JSON.stringify({ enabled }) }),
    testWebhook:     (id) => request(`/webhooks/${id}/test`, { method: 'POST' }),

    // Market breadth (TICK/TRIN/A-D/Up-Dn vol/P-C)
    breadthSnapshot: () => request('/breadth/snapshot'),

    // Fear & Greed gauge (7-component CNN-style composite)
    fearGreed: () => request('/fear-greed'),

    // Pre-market / overnight cross-asset dashboard
    premarketSnapshot: () => request('/premarket/snapshot'),

    // Implied-vol surface (IV grid + term structure + skew)
    volSurface: (sym, n = 8) => request(`/vol-surface/${encodeURIComponent(sym)}?n=${n}`),

    // Walk-forward optimization (rolling IS/OOS sweep)
    walkForward: (body) =>
        request('/backtest/walk-forward', { method: 'POST', body: JSON.stringify(body) }),

    // Tax-lot tracker (FIFO/LIFO, ST/LT, wash-sale)
    taxLots: (accountId, year, method) =>
        request(`/tax-lots/${accountId}?year=${year}&method=${method}`),

    // Stock comparison (2-4 symbols side-by-side)
    compare: (symbolsCsv) => request(`/compare?symbols=${encodeURIComponent(symbolsCsv)}`),

    // Live P/L tracker (snapshot of open positions with fresh quotes)
    livePositions: (accountId) => request(`/live-positions/${accountId}`),

    // Sector ETF rotation heatmap
    sectorRotation: () => request('/sector-rotation'),

    // Mood-vs-PnL analytics
    moodAnalytics: (accountId) => request(`/mood-analytics/${accountId}`),

    // Streaks + discipline scorecard
    discipline: (accountId) => request(`/discipline/${accountId}`),
    disciplineScore: (accountId, days = 7) =>
        request(`/discipline/${accountId}/score?days=${days}`),

    // R-multiple distribution + SQN + per-tag breakdown
    rDistribution: (accountId) => request(`/r-distribution/${accountId}`),

    // Monte Carlo equity forecast
    equityForecast: (body) =>
        request('/equity-forecast', { method: 'POST', body: JSON.stringify(body) }),

    // Fill quality (bar-level slippage approximation)
    fillQuality: (accountId) => request(`/fill-quality/${accountId}`),

    // Multi-account overview
    accountsOverview: () => request('/accounts/overview'),

    // Trade comparison (2-4 trades, side-by-side + normalized P/L overlay)
    tradeCompare: (tradeIds) =>
        request(`/trade-compare?trade_ids=${tradeIds.join(',')}`),

    // Custom indicator registry
    listCustomIndicators:  () => request('/custom-indicators'),
    createCustomIndicator: (body) =>
        request('/custom-indicators', { method: 'POST', body: JSON.stringify(body) }),
    deleteCustomIndicator: (id) =>
        request(`/custom-indicators/${id}`, { method: 'DELETE' }),
    evalCustomIndicators:  (sym, interval, days, indicator_ids) =>
        request(`/custom-indicators/eval/${encodeURIComponent(sym)}?interval=${interval}&days=${days}`,
                { method: 'POST', body: JSON.stringify({ indicator_ids }) }),

    // Trade reviews (forced reflection on |R|>=2 trades)
    listReviews:    (limit = 50) => request(`/trade-reviews?limit=${limit}`),
    reviewsNeeded:  (accountId, limit = 50) =>
        request(`/trade-reviews/needed/${accountId}?limit=${limit}`),
    reviewStats:    (accountId) => request(`/trade-reviews/stats/${accountId}`),
    reviewForTrade: (tradeId)   => request(`/trade-reviews/trade/${tradeId}`),
    saveReview:     (body) =>
        request('/trade-reviews', { method: 'POST', body: JSON.stringify(body) }),
    deleteReview:   (tradeId) =>
        request(`/trade-reviews/trade/${tradeId}/delete`, { method: 'POST' }),

    // Trading goals
    listGoals:   () => request('/goals'),
    createGoal:  (body) => request('/goals', { method: 'POST', body: JSON.stringify(body) }),
    updateGoal:  (id, body) => request(`/goals/${id}`, { method: 'PUT', body: JSON.stringify(body) }),
    deleteGoal:  (id) => request(`/goals/${id}`, { method: 'DELETE' }),
    goalProgress:(id) => request(`/goals/${id}/progress`),

    // Per-trade tape replay (bars + execs timeline)
    tapeReplay: (tradeId) => request(`/tape-replay/${tradeId}`),

    // Backtest preset library (save / share / fork)
    listMyBacktestPresets: () => request('/backtest-presets'),
    listPublicBacktestPresets: (limit = 50) =>
        request(`/backtest-presets/public?limit=${limit}`),
    getBacktestPresetBySlug: (slug) => request(`/backtest-presets/slug/${slug}`),
    createBacktestPreset: (body) =>
        request('/backtest-presets', { method: 'POST', body: JSON.stringify(body) }),
    updateBacktestPreset: (id, body) =>
        request(`/backtest-presets/${id}`, { method: 'PUT', body: JSON.stringify(body) }),
    deleteBacktestPreset: (id) =>
        request(`/backtest-presets/${id}`, { method: 'DELETE' }),
    forkBacktestPreset: (slug) =>
        request(`/backtest-presets/slug/${slug}/fork`, { method: 'POST' }),

    // Portfolio rebalancing
    rebalanceTargetsList: () => request('/rebalance/targets'),
    rebalanceTargetSave: (body) =>
        request('/rebalance/targets', { method: 'POST', body: JSON.stringify(body) }),
    rebalanceTargetDelete: (id) =>
        request(`/rebalance/targets/${id}`, { method: 'DELETE' }),
    rebalanceRun: (body) =>
        request('/rebalance/run', { method: 'POST', body: JSON.stringify(body) }),

    // Strategy alerts (compound AND/OR/NOT rules)
    listStrategyAlerts: () => request('/strategy-alerts'),
    createStrategyAlert: (body) =>
        request('/strategy-alerts', { method: 'POST', body: JSON.stringify(body) }),
    updateStrategyAlert: (id, body) =>
        request(`/strategy-alerts/${id}`, { method: 'PUT', body: JSON.stringify(body) }),
    deleteStrategyAlert: (id) =>
        request(`/strategy-alerts/${id}`, { method: 'DELETE' }),
    strategyAlertFires: () => request('/strategy-alerts/fires'),
    strategyAlertsEvaluateNow: () =>
        request('/strategy-alerts/evaluate-now', { method: 'POST' }),

    // Correlation matrix (pairwise Pearson on cached daily-bar log-returns)
    corrWatchlist: (wid, days = 90) =>
        request(`/correlation/watchlist/${wid}?days=${days}`),
    corrSymbols: (csv, days = 90) =>
        request(`/correlation/symbols?symbols=${encodeURIComponent(csv)}&days=${days}`),

    // Position sizing (Kelly / fixed-fractional / R-based, correlation-aware)
    positionSize: (body) =>
        request('/position-size', { method: 'POST', body: JSON.stringify(body) }),
    positionSizeWinRate: (accountId) =>
        request(`/position-size/account/${accountId}/winrate`),

    // Earnings calendar + surprise tracking
    earningsCalendar: (days = 7) => request(`/earnings/calendar?days=${days}`),
    earningsSurprises: (days = 30) => request(`/earnings/surprises?days=${days}`),
    earningsPollNow: () => request('/earnings/poll-now', { method: 'POST' }),
    earningsRefreshSymbol: (sym) =>
        request(`/earnings/symbol/${encodeURIComponent(sym)}/refresh`, { method: 'POST' }),

    // News (sentiment-tagged history + FTS)
    newsBySymbol: (sym, limit = 20) =>
        request(`/news/symbol/${encodeURIComponent(sym)}?limit=${limit}`),
    newsRecent: (limit = 40) => request(`/news/recent?limit=${limit}`),
    newsSearch: (q, limit = 50) =>
        request(`/news/search?q=${encodeURIComponent(q)}&limit=${limit}`),
    newsPollNow: () => request('/news/poll-now', { method: 'POST' }),
    newsRefreshSymbol: (sym) =>
        request(`/news/symbol/${encodeURIComponent(sym)}/refresh`, { method: 'POST' }),

    // Dashboards (multi-monitor / per-workflow custom boards)
    listDashboards: () => request('/dashboards'),
    getDashboard: (id) => request(`/dashboards/${id}`),
    createDashboard: (body) =>
        request('/dashboards', { method: 'POST', body: JSON.stringify(body) }),
    updateDashboard: (id, body) =>
        request(`/dashboards/${id}`, { method: 'PUT', body: JSON.stringify(body) }),
    deleteDashboard: (id) =>
        request(`/dashboards/${id}`, { method: 'DELETE' }),

    // Personal Access Tokens (public API auth)
    listApiTokens: () => request('/api-tokens'),
    createApiToken: (body) =>
        request('/api-tokens', { method: 'POST', body: JSON.stringify(body) }),
    revokeApiToken: (id) => request(`/api-tokens/${id}`, { method: 'DELETE' }),
    setApiTokenRateLimit: (id, rate_limit_per_min) =>
        request(`/api-tokens/${id}/rate-limit`,
                { method: 'PATCH', body: JSON.stringify({ rate_limit_per_min }) }),

    // AI journal analysis
    getAiSettings: () => request('/journal-ai/settings'),
    setAiSettings: (body) =>
        request('/journal-ai/settings', { method: 'POST', body: JSON.stringify(body) }),
    runAiAnalysis: (tradeId) =>
        request(`/journal-ai/${tradeId}/analyze`, { method: 'POST' }),
    getAiCached: (tradeId) =>
        request(`/journal-ai/${tradeId}/cached`),

    // Chart drawings (per-user, per-symbol persisted overlays)
    listChartDrawings: (sym) => request(`/chart-drawings/${encodeURIComponent(sym)}`),
    createChartDrawing: (sym, draft) =>
        request(`/chart-drawings/${encodeURIComponent(sym)}`,
                { method: 'POST', body: JSON.stringify(draft) }),
    deleteChartDrawing: (id) =>
        request(`/chart-drawings/by-id/${id}`, { method: 'DELETE' }),
    deleteChartDrawings: (sym) =>
        request(`/chart-drawings/${encodeURIComponent(sym)}`, { method: 'DELETE' }),

    // settings
    settings: () => request('/settings'),
    updateSettings: (body) => request('/settings', { method: 'POST', body: JSON.stringify(body) }),
    listFilters: () => request('/filter-sets'),
    saveFilter: (name, payload, is_default = false) =>
        request('/filter-sets', { method: 'POST', body: JSON.stringify({ name, payload, is_default }) }),
    deleteFilter: (id) => request(`/filter-sets/${id}`, { method: 'DELETE' }),

    // search
    search: (q, scope = 'all', limit = 50) => request(`/search${qs({ q, scope, limit })}`),

    // note templates
    noteTemplates: (scope) => request(`/note-templates${qs({ scope })}`),
    upsertNoteTemplate: (name, scope, body_md, is_default) =>
        request('/note-templates', { method: 'POST', body: JSON.stringify({ name, scope, body_md, is_default }) }),
    deleteNoteTemplate: (id) => request(`/note-templates/${id}`, { method: 'DELETE' }),
    defaultNoteTemplate: (scope) => request(`/note-templates/default${qs({ scope })}`),

    // plans
    plans: () => request('/plans'),
    createPlan: (body) => request('/plans', { method: 'POST', body: JSON.stringify(body) }),
    linkPlan: (plan_id, trade_id) =>
        request(`/plans/${plan_id}/link/${trade_id}`, { method: 'POST' }),
    abandonPlan: (id) => request(`/plans/${id}`, { method: 'DELETE' }),

    // ============================================================
    // Chart transformations — alt bar series + auto-drawing overlays
    // ============================================================
    barsHeikinAshi:    (sym, q) => request(`/bars/${encodeURIComponent(sym)}/heikin-ashi${qs(q)}`),
    barsRenko:         (sym, q) => request(`/bars/${encodeURIComponent(sym)}/renko${qs(q)}`),
    barsVolumeProfile: (sym, q) => request(`/bars/${encodeURIComponent(sym)}/volume-profile${qs(q)}`),
    barsIchimoku:      (sym, q) => request(`/bars/${encodeURIComponent(sym)}/ichimoku${qs(q)}`),
    barsFibonacci:     (sym, q) => request(`/bars/${encodeURIComponent(sym)}/fibonacci${qs(q)}`),
    barsSupertrend:    (sym, q) => request(`/bars/${encodeURIComponent(sym)}/supertrend${qs(q)}`),
    barsSwingPoints:   (sym, q) => request(`/bars/${encodeURIComponent(sym)}/swing-points${qs(q)}`),
    barsCandlestickPatterns: (sym, q) => request(`/bars/${encodeURIComponent(sym)}/candlestick-patterns${qs(q)}`),
    barsPivotsFloor:     (sym, q) => request(`/bars/${encodeURIComponent(sym)}/pivots/floor${qs(q)}`),
    barsPivotsCamarilla: (sym, q) => request(`/bars/${encodeURIComponent(sym)}/pivots/camarilla${qs(q)}`),
    barsPivotsWoodie:    (sym, q) => request(`/bars/${encodeURIComponent(sym)}/pivots/woodie${qs(q)}`),
    barsPivotsDemark:    (sym, q) => request(`/bars/${encodeURIComponent(sym)}/pivots/demark${qs(q)}`),

    // ============================================================
    // Technical indicators — series-out, aligned with bar_time
    // ============================================================
    indSma:           (sym, q) => request(`/bars/${encodeURIComponent(sym)}/sma${qs(q)}`),
    indEma:           (sym, q) => request(`/bars/${encodeURIComponent(sym)}/ema${qs(q)}`),
    indRsi:           (sym, q) => request(`/bars/${encodeURIComponent(sym)}/rsi${qs(q)}`),
    indMacd:          (sym, q) => request(`/bars/${encodeURIComponent(sym)}/macd${qs(q)}`),
    indBollinger:     (sym, q) => request(`/bars/${encodeURIComponent(sym)}/bollinger${qs(q)}`),
    indAtr:           (sym, q) => request(`/bars/${encodeURIComponent(sym)}/atr${qs(q)}`),
    indRoc:           (sym, q) => request(`/bars/${encodeURIComponent(sym)}/roc${qs(q)}`),
    indTrix:          (sym, q) => request(`/bars/${encodeURIComponent(sym)}/trix${qs(q)}`),
    indDpo:           (sym, q) => request(`/bars/${encodeURIComponent(sym)}/dpo${qs(q)}`),
    indCoppock:       (sym, q) => request(`/bars/${encodeURIComponent(sym)}/coppock${qs(q)}`),
    indSchaffTrend:   (sym, q) => request(`/bars/${encodeURIComponent(sym)}/schaff-trend${qs(q)}`),
    indMassIndex:     (sym, q) => request(`/bars/${encodeURIComponent(sym)}/mass-index${qs(q)}`),
    indAdx:           (sym, q) => request(`/bars/${encodeURIComponent(sym)}/adx${qs(q)}`),
    indStochastic:    (sym, q) => request(`/bars/${encodeURIComponent(sym)}/stochastic${qs(q)}`),
    indWilliamsR:     (sym, q) => request(`/bars/${encodeURIComponent(sym)}/williams-r${qs(q)}`),
    indCci:           (sym, q) => request(`/bars/${encodeURIComponent(sym)}/cci${qs(q)}`),
    indMfi:           (sym, q) => request(`/bars/${encodeURIComponent(sym)}/mfi${qs(q)}`),
    indDonchian:      (sym, q) => request(`/bars/${encodeURIComponent(sym)}/donchian${qs(q)}`),
    indParabolicSar:  (sym, q) => request(`/bars/${encodeURIComponent(sym)}/parabolic-sar${qs(q)}`),
    indAnchoredVwap:  (sym, q) => request(`/bars/${encodeURIComponent(sym)}/anchored-vwap${qs(q)}`),
    indAroon:         (sym, q) => request(`/bars/${encodeURIComponent(sym)}/aroon${qs(q)}`),
    indAwesomeOscillator: (sym, q) => request(`/bars/${encodeURIComponent(sym)}/awesome-oscillator${qs(q)}`),
    indVortex:        (sym, q) => request(`/bars/${encodeURIComponent(sym)}/vortex${qs(q)}`),
    indChaikinVolatility: (sym, q) => request(`/bars/${encodeURIComponent(sym)}/chaikin-volatility${qs(q)}`),
    indObv:           (sym, q) => request(`/bars/${encodeURIComponent(sym)}/obv${qs(q)}`),
    indAccumulationDistribution: (sym, q) => request(`/bars/${encodeURIComponent(sym)}/accumulation-distribution${qs(q)}`),
    indForceIndex:    (sym, q) => request(`/bars/${encodeURIComponent(sym)}/force-index${qs(q)}`),
    indKeltner:       (sym, q) => request(`/bars/${encodeURIComponent(sym)}/keltner${qs(q)}`),
    indVwapBands:     (sym, q) => request(`/bars/${encodeURIComponent(sym)}/vwap-bands${qs(q)}`),
    indBbSqueeze:     (sym, q) => request(`/bars/${encodeURIComponent(sym)}/bb-squeeze${qs(q)}`),
    indRsiDivergence: (sym, q) => request(`/bars/${encodeURIComponent(sym)}/rsi-divergence${qs(q)}`),
    indTrendChannel:  (sym, q) => request(`/bars/${encodeURIComponent(sym)}/trend-channel${qs(q)}`),

    // ============================================================
    // Options analytics (chain-derived)
    // ============================================================
    optionsMaxPain: (sym, q) => request(`/options/${encodeURIComponent(sym)}/max-pain${qs(q)}`),
    optionsGex:     (sym, q) => request(`/options/${encodeURIComponent(sym)}/gex${qs(q)}`),
    optionsIvSkew:  (sym, q) => request(`/options/${encodeURIComponent(sym)}/iv-skew${qs(q)}`),

    // ============================================================
    // Stateless calculators
    // ============================================================
    calcKelly:               (b) => request('/calc/kelly',                { method: 'POST', body: JSON.stringify(b) }),
    calcDynamicKelly:        (b) => request('/calc/dynamic-kelly',        { method: 'POST', body: JSON.stringify(b) }),
    calcOptimalF:            (b) => request('/calc/optimal-f',            { method: 'POST', body: JSON.stringify(b) }),
    calcVarHistorical:       (b) => request('/calc/var-historical',       { method: 'POST', body: JSON.stringify(b) }),
    calcVarGaussian:         (b) => request('/calc/var-gaussian',         { method: 'POST', body: JSON.stringify(b) }),
    calcMonteCarlo:          (b) => request('/calc/monte-carlo',          { method: 'POST', body: JSON.stringify(b) }),
    calcRiskParity:          (b) => request('/calc/risk-parity',          { method: 'POST', body: JSON.stringify(b) }),
    calcRiskOnOff:           (b) => request('/calc/risk-on-off',          { method: 'POST', body: JSON.stringify(b) }),
    calcMarginCall:          (b) => request('/calc/margin-call',          { method: 'POST', body: JSON.stringify(b) }),
    calcMarginRunway:        (b) => request('/calc/margin-runway',        { method: 'POST', body: JSON.stringify(b) }),
    calcBuyingPower:         (b) => request('/calc/buying-power',         { method: 'POST', body: JSON.stringify(b) }),
    calcTaxLossHarvest:      (b) => request('/calc/tax-loss-harvest',     { method: 'POST', body: JSON.stringify(b) }),
    calcWashSale:            (b) => request('/calc/wash-sale',            { method: 'POST', body: JSON.stringify(b) }),
    calcCostBasis:           (b) => request('/calc/cost-basis',           { method: 'POST', body: JSON.stringify(b) }),
    calcCommissionOptimizer: (b) => request('/calc/commission-optimizer', { method: 'POST', body: JSON.stringify(b) }),
    calcYieldCurve:          (b) => request('/calc/yield-curve',          { method: 'POST', body: JSON.stringify(b) }),
    calcBondDuration:        (b) => request('/calc/bond-duration',        { method: 'POST', body: JSON.stringify(b) }),
    calcCarryScore:          (b) => request('/calc/carry-score',          { method: 'POST', body: JSON.stringify(b) }),
    calcCurrencyExposure:    (b) => request('/calc/currency-exposure',    { method: 'POST', body: JSON.stringify(b) }),
    calcVixTermStructure:    (b) => request('/calc/vix-term-structure',   { method: 'POST', body: JSON.stringify(b) }),
    calcRiskReward:          (b) => request('/calc/risk-reward',          { method: 'POST', body: JSON.stringify(b) }),

    // ============================================================
    // Trade analytics — psychology, performance, event, quality, portfolio
    // ============================================================
    anlyTiltDetector:        (b) => request('/analytics/tilt-detector',         { method: 'POST', body: JSON.stringify(b) }),
    anlyDisciplineScore:     (b) => request('/analytics/discipline-score',      { method: 'POST', body: JSON.stringify(b) }),
    anlyEmotionTags:         (b) => request('/analytics/emotion-tags',          { method: 'POST', body: JSON.stringify(b) }),
    anlyOvertrading:         (b) => request('/analytics/overtrading',           { method: 'POST', body: JSON.stringify(b) }),
    anlyStreaks:             (b) => request('/analytics/streaks',               { method: 'POST', body: JSON.stringify(b) }),
    anlyLosingStreakProbability: (b) => request('/analytics/losing-streak-probability', { method: 'POST', body: JSON.stringify(b) }),
    anlyWinLossAsymmetry:    (b) => request('/analytics/winloss-asymmetry',     { method: 'POST', body: JSON.stringify(b) }),
    anlyPyramidRules:        (b) => request('/analytics/pyramid-rules',         { method: 'POST', body: JSON.stringify(b) }),
    anlyCagrSimple:          (b) => request('/analytics/cagr-simple',           { method: 'POST', body: JSON.stringify(b) }),
    anlyCagrRolling:         (b) => request('/analytics/cagr-rolling',          { method: 'POST', body: JSON.stringify(b) }),
    anlyProfitFactor:        (b) => request('/analytics/profit-factor',         { method: 'POST', body: JSON.stringify(b) }),
    anlySortino:             (b) => request('/analytics/sortino',               { method: 'POST', body: JSON.stringify(b) }),
    anlyTreynor:             (b) => request('/analytics/treynor',               { method: 'POST', body: JSON.stringify(b) }),
    anlyInformationRatio:    (b) => request('/analytics/information-ratio',     { method: 'POST', body: JSON.stringify(b) }),
    anlySharpeByWindow:      (b) => request('/analytics/sharpe-by-window',      { method: 'POST', body: JSON.stringify(b) }),
    anlyHighWaterMark:       (b) => request('/analytics/high-water-mark',       { method: 'POST', body: JSON.stringify(b) }),
    anlyDrawdownDuration:    (b) => request('/analytics/drawdown-duration',     { method: 'POST', body: JSON.stringify(b) }),
    anlyEarningsMoveStraddle:(b) => request('/analytics/earnings-move-straddle',{ method: 'POST', body: JSON.stringify(b) }),
    anlyEarningsMoveIv:      (b) => request('/analytics/earnings-move-iv',      { method: 'POST', body: JSON.stringify(b) }),
    anlyPead:                (b) => request('/analytics/pead',                  { method: 'POST', body: JSON.stringify(b) }),
    anlyGapAnalysis:         (b) => request('/analytics/gap-analysis',          { method: 'POST', body: JSON.stringify(b) }),
    anlyCalendarBias:        (b) => request('/analytics/calendar-bias',         { method: 'POST', body: JSON.stringify(b) }),
    anlyHaltRisk:            (b) => request('/analytics/halt-risk',             { method: 'POST', body: JSON.stringify(b) }),
    anlyTradeQuality:        (b) => request('/analytics/trade-quality',         { method: 'POST', body: JSON.stringify(b) }),
    anlyExitTiming:          (b) => request('/analytics/exit-timing',           { method: 'POST', body: JSON.stringify(b) }),
    anlyMaeStopTuning:       (b) => request('/analytics/mae-stop-tuning',       { method: 'POST', body: JSON.stringify(b) }),
    anlyBracketOrder:        (b) => request('/analytics/bracket-order',         { method: 'POST', body: JSON.stringify(b) }),
    anlyProbabilityOfTouch:  (b) => request('/analytics/probability-of-touch',  { method: 'POST', body: JSON.stringify(b) }),
    anlyPortfolioGreeks:     (b) => request('/analytics/portfolio-greeks',      { method: 'POST', body: JSON.stringify(b) }),
    anlyConcentration:       (b) => request('/analytics/concentration',         { method: 'POST', body: JSON.stringify(b) }),
    anlySectorExposure:      (b) => request('/analytics/sector-exposure',       { method: 'POST', body: JSON.stringify(b) }),
    anlyBeta:                (b) => request('/analytics/beta',                  { method: 'POST', body: JSON.stringify(b) }),
    anlyBetaHedge:           (b) => request('/analytics/beta-hedge',            { method: 'POST', body: JSON.stringify(b) }),
    anlyHedgeRatio:          (b) => request('/analytics/hedge-ratio',           { method: 'POST', body: JSON.stringify(b) }),
    anlySpreadPayoff:        (b) => request('/analytics/spread-payoff',         { method: 'POST', body: JSON.stringify(b) }),
    anlyOptionPayoffDiagram: (b) => request('/analytics/option-payoff-diagram', { method: 'POST', body: JSON.stringify(b) }),
    anlyMultiLegOptionPricer:(b) => request('/analytics/multi-leg-option-pricer',{ method: 'POST', body: JSON.stringify(b) }),
    anlySviVolatilitySmile:  (b) => request('/analytics/svi-volatility-smile',  { method: 'POST', body: JSON.stringify(b) }),
    anlyGbmPathSimulator:    (b) => request('/analytics/gbm-simulator',         { method: 'POST', body: JSON.stringify(b) }),
    anlyJumpDiffusionSimulator: (b) => request('/analytics/jump-diffusion-simulator', { method: 'POST', body: JSON.stringify(b) }),
    anlyKouJumpDiffusionSimulator: (b) => request('/analytics/kou-jump-diffusion-simulator', { method: 'POST', body: JSON.stringify(b) }),
    anlyFbmGenerator:        (b) => request('/analytics/fbm-generator',         { method: 'POST', body: JSON.stringify(b) }),
    minVariancePortfolio:    (b) => request('/portfolio/min-variance',          { method: 'POST', body: JSON.stringify(b) }),
    maxDiversification:      (b) => request('/portfolio/max-diversification',   { method: 'POST', body: JSON.stringify(b) }),
    equalRiskContributionPortfolio: (b) => request('/analytics/equal-risk-contribution-portfolio', { method: 'POST', body: JSON.stringify(b) }),
    anlyValueAtRiskHistorical: (b) => request('/analytics/var-historical',          { method: 'POST', body: JSON.stringify(b) }),
    anlyValueAtRiskFilteredHistorical: (b) => request('/analytics/var-filtered-historical', { method: 'POST', body: JSON.stringify(b) }),
    anlyCornishFisherVar:    (b) => request('/analytics/cornish-fisher-var',       { method: 'POST', body: JSON.stringify(b) }),
    anlyLowessSmoother:      (b) => request('/analytics/lowess-smoother',          { method: 'POST', body: JSON.stringify(b) }),
    anlyKalmanSmootherRts:   (b) => request('/analytics/kalman-smoother-rts',      { method: 'POST', body: JSON.stringify(b) }),
    anlyTheilSenEstimator:   (b) => request('/analytics/theil-sen-estimator',      { method: 'POST', body: JSON.stringify(b) }),
    anlyPolynomialRegression:(b) => request('/analytics/polynomial-regression',    { method: 'POST', body: JSON.stringify(b) }),
    anlyMatrixProfile:       (b) => request('/analytics/matrix-profile',           { method: 'POST', body: JSON.stringify(b) }),
    anlyOptimalExecutionPov: (b) => request('/analytics/optimal-execution-pov',    { method: 'POST', body: JSON.stringify(b) }),
    anlyOptimalExecutionTwap:(b) => request('/analytics/optimal-execution-twap',   { method: 'POST', body: JSON.stringify(b) }),
    anlyOptimalExecutionVwap:(b) => request('/analytics/optimal-execution-vwap',   { method: 'POST', body: JSON.stringify(b) }),
    anlyMarkovSwitching2State:(b) => request('/analytics/markov-switching-2state',  { method: 'POST', body: JSON.stringify(b) }),
    anlyAmericanOptionLsmc:  (b) => request('/analytics/american-option-lsmc',     { method: 'POST', body: JSON.stringify(b) }),
    anlyGarmanKohlhagenFxOption: (b) => request('/analytics/garman-kohlhagen-fx-option', { method: 'POST', body: JSON.stringify(b) }),
    anlyForwardVolatilityBootstrap: (b) => request('/analytics/forward-volatility-bootstrap', { method: 'POST', body: JSON.stringify(b) }),
    anlyPrincipalComponentYieldCurve: (b) => request('/analytics/principal-component-yield-curve', { method: 'POST', body: JSON.stringify(b) }),
    anlyEmpiricalModeDecomposition: (b) => request('/analytics/empirical-mode-decomposition', { method: 'POST', body: JSON.stringify(b) }),
    anlyWaveletDecompositionHaar: (b) => request('/analytics/wavelet-decomposition-haar', { method: 'POST', body: JSON.stringify(b) }),
    anlySingularSpectrumAnalysis: (b) => request('/analytics/singular-spectrum-analysis', { method: 'POST', body: JSON.stringify(b) }),
    anlyRiskReversalBfCalc:  (b) => request('/analytics/risk-reversal-25-delta-butterfly', { method: 'POST', body: JSON.stringify(b) }),
    anlyMarchenkoPasturCleaning: (b) => request('/analytics/marchenko-pastur-cleaning', { method: 'POST', body: JSON.stringify(b) }),
    anlyMicropriceStoikov:   (b) => request('/analytics/microprice-stoikov',       { method: 'POST', body: JSON.stringify(b) }),
    anlyDynamicTimeWarping:  (b) => request('/analytics/dynamic-time-warping',     { method: 'POST', body: JSON.stringify(b) }),
    anlyHurstExponent:       (b) => request('/analytics/hurst-exponent',           { method: 'POST', body: JSON.stringify(b) }),
    anlyBayesianChangePoint: (b) => request('/analytics/bayesian-change-point',    { method: 'POST', body: JSON.stringify(b) }),
    anlyVasicekShortRateSimulator: (b) => request('/analytics/vasicek-short-rate-simulator', { method: 'POST', body: JSON.stringify(b) }),
    anlyKalmanDynamicBeta:   (b) => request('/analytics/kalman-dynamic-beta',      { method: 'POST', body: JSON.stringify(b) }),
    optsIvSolver:            (b) => request('/options/calc/iv-solver',             { method: 'POST', body: JSON.stringify(b) }),
    optsGreeksProfile:       (b) => request('/options/calc/greeks-profile',        { method: 'POST', body: JSON.stringify(b) }),
    anlySecondOrderGreeks:   (b) => request('/options/calc/second-order-greeks',   { method: 'POST', body: JSON.stringify(b) }),
    anlyRollingZscore:       (b) => request('/analytics/rolling-zscore',        { method: 'POST', body: JSON.stringify(b) }),
    anlyStrategyCorrelation: (b) => request('/analytics/strategy-correlation',  { method: 'POST', body: JSON.stringify(b) }),
    anlySpreadAttribution:   (b) => request('/analytics/spread-attribution',    { method: 'POST', body: JSON.stringify(b) }),
    anlyPairTradeSignal:     (b) => request('/analytics/pair-trade-signal',     { method: 'POST', body: JSON.stringify(b) }),

    // ============================================================
    // Microstructure / order flow / heatmaps / regime
    // ============================================================
    microOrderBookImbalance: (b) => request('/microstructure/order-book-imbalance', { method: 'POST', body: JSON.stringify(b) }),
    microOrderFlowClassify:  (b) => request('/microstructure/order-flow-classify',  { method: 'POST', body: JSON.stringify(b) }),
    microOrderFlowAggregate: (b) => request('/microstructure/order-flow-aggregate', { method: 'POST', body: JSON.stringify(b) }),
    microLiquidity:          (b) => request('/microstructure/liquidity',            { method: 'POST', body: JSON.stringify(b) }),
    microMarketImpact:       (b) => request('/microstructure/market-impact',        { method: 'POST', body: JSON.stringify(b) }),
    microPerSymbolSlippage:  (b) => request('/microstructure/per-symbol-slippage',  { method: 'POST', body: JSON.stringify(b) }),
    microVwapSlippage:       (b) => request('/microstructure/vwap-slippage',        { method: 'POST', body: JSON.stringify(b) }),
    microOrderStaleness:     (b) => request('/microstructure/order-staleness',      { method: 'POST', body: JSON.stringify(b) }),
    microAlmgrenChriss:      (b) => request('/microstructure/almgren-chriss',       { method: 'POST', body: JSON.stringify(b) }),
    microImplementationShortfall: (b) => request('/microstructure/implementation-shortfall', { method: 'POST', body: JSON.stringify(b) }),
    anlyDeflatedSharpe:      (b) => request('/analytics/deflated-sharpe',           { method: 'POST', body: JSON.stringify(b) }),
    microVpin:               (b) => request('/microstructure/vpin',                 { method: 'POST', body: JSON.stringify(b) }),
    anlyCupAndHandle:        (b) => request('/analytics/cup-and-handle',            { method: 'POST', body: JSON.stringify(b) }),
    microSpreadTracker:      (b) => request('/microstructure/spread-tracker',       { method: 'POST', body: JSON.stringify(b) }),
    microIntradayHeatmap:    (b) => request('/heatmaps/intraday',                   { method: 'POST', body: JSON.stringify(b) }),
    microTwap:               (b) => request('/microstructure/twap',                 { method: 'POST', body: JSON.stringify(b) }),
    heatmapIntraday:         (b) => request('/heatmaps/intraday',                   { method: 'POST', body: JSON.stringify(b) }),
    heatmapDowHour:          (b) => request('/heatmaps/dow-hour',                   { method: 'POST', body: JSON.stringify(b) }),
    regimeEquity:            (b) => request('/regime/equity',                       { method: 'POST', body: JSON.stringify(b) }),
    regimeNewsEvent:         (b) => request('/regime/news-event',                   { method: 'POST', body: JSON.stringify(b) }),

    // ============================================================
    // Discipline / risk-gates / pre-trade
    // ============================================================
    discTimeInForce:         (b) => request('/discipline/time-in-force',            { method: 'POST', body: JSON.stringify(b) }),
    discOpenType:            (b) => request('/discipline/open-type',                { method: 'POST', body: JSON.stringify(b) }),
    discTradePlanChecklist:  (b) => request('/discipline/trade-plan-checklist',     { method: 'POST', body: JSON.stringify(b) }),
    discStopLossBacktest:    (b) => request('/discipline/stop-loss-backtest',       { method: 'POST', body: JSON.stringify(b) }),
    discStopLossBestOf:      (b) => request('/discipline/stop-loss-best-of',        { method: 'POST', body: JSON.stringify(b) }),
    discPyramidPlan:         (b) => request('/discipline/pyramid-plan',             { method: 'POST', body: JSON.stringify(b) }),
    discDailyLossLimit:      (b) => request('/discipline/daily-loss-limit',         { method: 'POST', body: JSON.stringify(b) }),
    discDrawdownThrottle:    (b) => request('/discipline/drawdown-throttle',        { method: 'POST', body: JSON.stringify(b) }),
    discGoalTracker:         (b) => request('/discipline/goal-tracker',             { method: 'POST', body: JSON.stringify(b) }),
    discTripleScreen:        (b) => request('/discipline/triple-screen',            { method: 'POST', body: JSON.stringify(b) }),
    discChandelierStop:      (b) => request('/discipline/chandelier-stop',          { method: 'POST', body: JSON.stringify(b) }),
    discVolStopClose:        (b) => request('/discipline/vol-stop-close',           { method: 'POST', body: JSON.stringify(b) }),

    // ============================================================
    // Options calc (IV/OI history + margin)
    // ============================================================
    optCalcIvRank:           (b) => request('/options/calc/iv-rank',                { method: 'POST', body: JSON.stringify(b) }),
    optCalcIvBacktest:       (b) => request('/options/calc/iv-backtest',            { method: 'POST', body: JSON.stringify(b) }),
    optCalcOiChange:         (b) => request('/options/calc/oi-change',              { method: 'POST', body: JSON.stringify(b) }),
    optCalcMarginNakedShort: (b) => request('/options/calc/margin-naked-short',     { method: 'POST', body: JSON.stringify(b) }),
    optCalcMarginVertical:   (b) => request('/options/calc/margin-vertical',        { method: 'POST', body: JSON.stringify(b) }),

    // ============================================================
    // Clustering + setup tracking
    // ============================================================
    clustersTradeFeatures:   (b) => request('/clusters/trade-features',             { method: 'POST', body: JSON.stringify(b) }),
    clustersCorrelation:     (b) => request('/clusters/correlation',                { method: 'POST', body: JSON.stringify(b) }),
    setupsBySetup:           (b) => request('/setups/by-setup',                     { method: 'POST', body: JSON.stringify(b) }),

    // ============================================================
    // Portfolio reporting (POST)
    // ============================================================
    portPositionAging:       (b) => request('/portfolio/position-aging',            { method: 'POST', body: JSON.stringify(b) }),
    portPositionIrr:         (b) => request('/portfolio/position-irr',              { method: 'POST', body: JSON.stringify(b) }),
    portMtmReconciliation:   (b) => request('/portfolio/mtm-reconciliation',        { method: 'POST', body: JSON.stringify(b) }),

    // ============================================================
    // Sentiment + tax + filter
    // ============================================================
    sentPutCallRatio:        (b) => request('/sentiment/calc/put-call-ratio',       { method: 'POST', body: JSON.stringify(b) }),
    taxReconcile1099b:       (b) => request('/tax/reconcile-1099b',                 { method: 'POST', body: JSON.stringify(b) }),
    filterSymbols:           (b) => request('/filter/symbols',                      { method: 'POST', body: JSON.stringify(b) }),

    // ============================================================
    // Misc charts/bars
    // ============================================================
    chartsAtrCone:           (b) => request('/charts/atr-cone',                     { method: 'POST', body: JSON.stringify(b) }),
    barsAlligator:           (b) => request('/bars/alligator',                      { method: 'POST', body: JSON.stringify(b) }),

    // ============================================================
    // Calendar helpers
    // ============================================================
    calIsTradingDay:         (b) => request('/calendar/is-trading-day',             { method: 'POST', body: JSON.stringify(b) }),
    calNextTradingDay:       (b) => request('/calendar/next-trading-day',           { method: 'POST', body: JSON.stringify(b) }),
    calPriorTradingDay:      (b) => request('/calendar/prior-trading-day',          { method: 'POST', body: JSON.stringify(b) }),
    calAddTradingDays:       (b) => request('/calendar/add-trading-days',           { method: 'POST', body: JSON.stringify(b) }),
    calTradingDaysBetween:   (b) => request('/calendar/trading-days-between',       { method: 'POST', body: JSON.stringify(b) }),
    calEarningsWindow:       (b) => request('/calendar/earnings-window',            { method: 'POST', body: JSON.stringify(b) }),
    calEarningsAnalysis:     (b) => request('/calendar/earnings-analysis',          { method: 'POST', body: JSON.stringify(b) }),
    futuresRollSchedule:     (b) => request('/futures/roll-schedule',               { method: 'POST', body: JSON.stringify(b) }),
    microFootprint:          (b) => request('/microstructure/footprint',            { method: 'POST', body: JSON.stringify(b) }),
    microMarketProfile:      (b) => request('/microstructure/market-profile',       { method: 'POST', body: JSON.stringify(b) }),
    microStressTest:         (b) => request('/microstructure/stress-test',          { method: 'POST', body: JSON.stringify(b) }),
    cohortTilt:              (b) => request('/sentiment/cohort-tilt',                { method: 'POST', body: JSON.stringify(b) }),
    anlyStrategyDecay:       (b) => request('/analytics/strategy-decay',             { method: 'POST', body: JSON.stringify(b) }),
    anlyVolatilityRegime:    (b) => request('/analytics/volatility-regime',          { method: 'POST', body: JSON.stringify(b) }),
    sipSimulator:            (b) => request('/portfolio/sip-simulator',              { method: 'POST', body: JSON.stringify(b) }),
    portfolioHeat:           (b) => request('/portfolio/heat',                       { method: 'POST', body: JSON.stringify(b) }),
    taxLotOptimizer:         (b) => request('/tax/lot-optimizer',                    { method: 'POST', body: JSON.stringify(b) }),
    microSpreadTracker:      (b) => request('/microstructure/spread-tracker',        { method: 'POST', body: JSON.stringify(b) }),
    microImplShortfall:      (b) => request('/microstructure/implementation-shortfall', { method: 'POST', body: JSON.stringify(b) }),
    anlyMeanReversion:       (b) => request('/analytics/mean-reversion',             { method: 'POST', body: JSON.stringify(b) }),
    anlyVolumeBurst:         (b) => request('/analytics/volume-burst',               { method: 'POST', body: JSON.stringify(b) }),
    chartsRoundLevels:       (b) => request('/charts/round-levels',                  { method: 'POST', body: JSON.stringify(b) }),
    anlyTimeframeConfluence: (b) => request('/analytics/timeframe-confluence',       { method: 'POST', body: JSON.stringify(b) }),
    anlyCrossover:           (b) => request('/analytics/crossover',                  { method: 'POST', body: JSON.stringify(b) }),
    anlyBreakout:            (b) => request('/analytics/breakout',                   { method: 'POST', body: JSON.stringify(b) }),
    anlyRangeContraction:    (b) => request('/analytics/range-contraction',          { method: 'POST', body: JSON.stringify(b) }),
    anlyStopHunt:            (b) => request('/analytics/stop-hunt',                  { method: 'POST', body: JSON.stringify(b) }),
    anlyFairValueGap:        (b) => request('/analytics/fair-value-gap',             { method: 'POST', body: JSON.stringify(b) }),
    anlyOrderBlock:          (b) => request('/analytics/order-block',                { method: 'POST', body: JSON.stringify(b) }),
    anlyBreakOfStructure:    (b) => request('/analytics/break-of-structure',         { method: 'POST', body: JSON.stringify(b) }),
    anlyChangeOfCharacter:   (b) => request('/analytics/change-of-character',        { method: 'POST', body: JSON.stringify(b) }),
    anlyEqualLevels:         (b) => request('/analytics/equal-levels',               { method: 'POST', body: JSON.stringify(b) }),
    microCumulativeDelta:    (b) => request('/microstructure/cumulative-delta',      { method: 'POST', body: JSON.stringify(b) }),
    anlyDisplacement:        (b) => request('/analytics/displacement',               { method: 'POST', body: JSON.stringify(b) }),
    anlyOpeningRange:        (b) => request('/analytics/opening-range',              { method: 'POST', body: JSON.stringify(b) }),
    anlyVsa:                 (b) => request('/analytics/vsa',                        { method: 'POST', body: JSON.stringify(b) }),
    anlyUlcerIndex:          (b) => request('/analytics/ulcer-index',                { method: 'POST', body: JSON.stringify(b) }),
    anlyCalmarRatio:         (b) => request('/analytics/calmar-ratio',               { method: 'POST', body: JSON.stringify(b) }),
    anlyWyckoff:             (b) => request('/analytics/wyckoff',                    { method: 'POST', body: JSON.stringify(b) }),
    anlyPremiumDiscount:     (b) => request('/analytics/premium-discount',           { method: 'POST', body: JSON.stringify(b) }),
    anlyCusum:               (b) => request('/analytics/cusum',                      { method: 'POST', body: JSON.stringify(b) }),
    anlyHaReversal:          (b) => request('/analytics/heikin-ashi-reversal',       { method: 'POST', body: JSON.stringify(b) }),
    anlyThreeBarReversal:    (b) => request('/analytics/three-bar-reversal',         { method: 'POST', body: JSON.stringify(b) }),
    anlyRangeExpansion:      (b) => request('/analytics/range-expansion',            { method: 'POST', body: JSON.stringify(b) }),
    anlyDemarker:            (b) => request('/analytics/demarker-oscillator',        { method: 'POST', body: JSON.stringify(b) }),
    anlyMurreyMath:          (b) => request('/charts/murrey-math',                   { method: 'POST', body: JSON.stringify(b) }),
    anlyDemarkPivots:        (b) => request('/analytics/demark-pivots',              { method: 'POST', body: JSON.stringify(b) }),
    anlyCypherPattern:       (b) => request('/patterns/cypher',                      { method: 'POST', body: JSON.stringify(b) }),
    anlyChoppiness:          (b) => request('/analytics/choppiness',                 { method: 'POST', body: JSON.stringify(b) }),
    anlyEfficiencyRatio:     (b) => request('/analytics/efficiency-ratio',           { method: 'POST', body: JSON.stringify(b) }),
    anlyRandomWalkIndex:     (b) => request('/analytics/random-walk-index',          { method: 'POST', body: JSON.stringify(b) }),
    anlyAccelerationDecel:   (b) => request('/analytics/acceleration-deceleration',  { method: 'POST', body: JSON.stringify(b) }),
    anlyLiquidityGrab:       (b) => request('/analytics/liquidity-grab',             { method: 'POST', body: JSON.stringify(b) }),
    anlyGapFillStats:        (b) => request('/analytics/gap-fill-stats',             { method: 'POST', body: JSON.stringify(b) }),
    anlyArmsIndex:           (b) => request('/analytics/arms-index',                 { method: 'POST', body: JSON.stringify(b) }),
    anlyMcclellanOsc:        (b) => request('/analytics/mcclellan-oscillator',       { method: 'POST', body: JSON.stringify(b) }),
    anlyInsideBarBreakout:   (b) => request('/analytics/inside-bar-breakout',        { method: 'POST', body: JSON.stringify(b) }),
};
