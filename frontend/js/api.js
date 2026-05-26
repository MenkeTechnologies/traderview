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
        token = localStorage.getItem('tv-token') || '';
    }
    return { baseUrl, hasToken: !!token };
}

export function setToken(t) {
    token = t;
    if (!window.__TAURI__) localStorage.setItem('tv-token', t);
}

export function clearToken() {
    token = '';
    if (!window.__TAURI__) localStorage.removeItem('tv-token');
}

async function request(path, opts = {}) {
    const headers = Object.assign({}, opts.headers || {});
    if (token) headers['Authorization'] = `Bearer ${token}`;
    if (opts.body && !(opts.body instanceof FormData) && !headers['Content-Type']) {
        headers['Content-Type'] = 'application/json';
    }
    const res = await fetch(`${baseUrl}/api${path}`, Object.assign({}, opts, { headers }));
    if (res.status === 401) {
        clearToken();
        throw new ApiError(401, 'unauthorized');
    }
    if (!res.ok) {
        let msg = res.statusText;
        try { msg = (await res.json()).error || msg; } catch (_) { /* not json */ }
        throw new ApiError(res.status, msg);
    }
    if (res.status === 204) return null;
    const ct = res.headers.get('content-type') || '';
    return ct.includes('application/json') ? res.json() : res.text();
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
};
