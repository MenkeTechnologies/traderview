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

/**
 * Normalize report-query args. Each report endpoint accepts the same
 * filter shape; we want callers to pass either a numeric days value
 * (legacy 30/60/90 toggle) or a filter object. Returns a plain object
 * ready to feed into qs().
 */
// Lazy import to avoid a circular dep — broker_context imports api.js
// for `brokersList`. The active broker is read from a global symbol the
// broker_context module installs at module-eval time.
function activeBrokerIdSync() {
    try { return globalThis.__tvActiveBroker?.() || null; } catch { return null; }
}

const rq = (account_id, f) => {
    const broker_id = activeBrokerIdSync();
    const base = broker_id ? { account_id, broker_id } : { account_id };
    if (f == null) return base;
    if (typeof f === 'number') return { ...base, days: f };
    return { ...base, ...f };
};

export class ApiError extends Error {
    constructor(status, msg) { super(msg); this.status = status; }
}

export const api = {
    // Generic passthrough — views written as `api.request('/path', opts)`
    // route here. New endpoints SHOULD get a dedicated method on this
    // object (`api.fooBar(...)`); this passthrough is for ad-hoc /
    // experimental routes that haven't been promoted yet.
    request,

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
    patchAccount: (id, body) =>
        request(`/accounts/${id}`, { method: 'PATCH', body: JSON.stringify(body) }),
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

    rebuildTrades: (account_id) =>
        request(`/accounts/${account_id}/rebuild-trades`, { method: 'POST' }),

    // reports
    // Each method accepts either a numeric `days` arg (legacy 30/60/90 toggle)
    // or a filter object: { days, symbol, side, asset_class, duration,
    // date_from, date_to, tag_id, starting_cash }. The same object is used by
    // the new global filter bar on /reports.
    overview: (account_id, f) => request(`/reports/overview${qs(rq(account_id, f))}`),
    bySymbol: (account_id, f) => request(`/reports/by-symbol${qs(rq(account_id, f))}`),
    byBroker: (account_id, f) => request(`/reports/by-broker${qs(rq(account_id, f))}`),
    bySide: (account_id, f) => request(`/reports/by-side${qs(rq(account_id, f))}`),
    byAssetClass: (account_id, f) => request(`/reports/by-asset-class${qs(rq(account_id, f))}`),
    byDow: (account_id, f) => request(`/reports/by-day-of-week${qs(rq(account_id, f))}`),
    byHour: (account_id, f) => request(`/reports/by-hour${qs(rq(account_id, f))}`),
    byHold: (account_id, f) => request(`/reports/by-hold${qs(rq(account_id, f))}`),
    byMonth: (account_id, f) => request(`/reports/by-month${qs(rq(account_id, f))}`),
    byPrice: (account_id, f) => request(`/reports/by-price${qs(rq(account_id, f))}`),
    byTag: (account_id, f) => request(`/reports/by-tag${qs(rq(account_id, f))}`),
    byDurationCoarse:   (account_id, f) => request(`/reports/by-duration-coarse${qs(rq(account_id, f))}`),
    byRBucket:          (account_id, f) => request(`/reports/by-r-bucket${qs(rq(account_id, f))}`),
    byOpeningGap:       (account_id, f) => request(`/reports/by-opening-gap${qs(rq(account_id, f))}`),
    byInstrumentVolume: (account_id, f) => request(`/reports/by-instrument-volume${qs(rq(account_id, f))}`),
    byMovement:         (account_id, f) => request(`/reports/by-movement${qs(rq(account_id, f))}`),
    dailySeries: (account_id, f) => request(`/reports/daily-series${qs(rq(account_id, f))}`),
    winLossDays: (account_id, f) => request(`/reports/win-loss-days${qs(rq(account_id, f))}`),
    rDist: (account_id, f) => request(`/reports/r-distribution${qs(rq(account_id, f))}`),
    streaks: (account_id, f) => request(`/reports/streaks${qs(rq(account_id, f))}`),
    comparison: (account_id, f) => request(`/reports/comparison${qs(rq(account_id, f))}`),
    exitEff: (account_id, f) => request(`/reports/exit-efficiency${qs(rq(account_id, f))}`),
    commissions: (account_id, f) => request(`/reports/commissions${qs(rq(account_id, f))}`),
    liquidity: (account_id, adv = '') => request(`/reports/liquidity${qs({ account_id, adv })}`),
    risk: (account_id, f) => request(`/reports/risk${qs(rq(account_id, f))}`),
    drawdown: (account_id, starting_cash, f) =>
        request(`/reports/drawdown${qs({ ...rq(account_id, f), starting_cash })}`),
    riskAdjusted: (account_id, starting_cash, f) =>
        request(`/reports/risk-adjusted${qs({ ...rq(account_id, f), starting_cash })}`),
    calendar: (account_id, f) => request(`/reports/calendar${qs(rq(account_id, f))}`),
    advanced: (account_id, starting_cash, f) =>
        request(`/reports/advanced${qs({ ...rq(account_id, f), starting_cash })}`),
    summary: (account_id, f) => request(`/stats/summary${qs(rq(account_id, f))}`),
    // Skips the global broker filter — callers (e.g., broker-compare)
    // supply broker_id / account_id explicitly and want the server to
    // respect THAT, not whichever broker the topbar selector points at.
    summaryRaw: (params = {}) => request(`/stats/summary${qs(params)}`),
    equity: (account_id, starting_cash, f) =>
        request(`/stats/equity${qs({ ...rq(account_id, f), starting_cash })}`),

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
    importExpense: (account_id, source, file, mapping) => {
        const fd = new FormData();
        fd.append('account_id', account_id);
        fd.append('source', source);
        fd.append('file', file, file.name);
        if (mapping) fd.append('mapping', JSON.stringify(mapping));
        return request('/expense/import', { method: 'POST', body: fd });
    },
    expenseRules: () => request('/expense/rules'),
    createExpenseRule: (body) =>
        request('/expense/rules', { method: 'POST', body: JSON.stringify(body) }),
    deleteExpenseRule: (id) => request(`/expense/rules/${id}`, { method: 'DELETE' }),
    seedExpenseRules: () => request('/expense/rules/seed', { method: 'POST' }),

    // --- receipts -------------------------------------------------------
    receipts: (filters = {}) => {
        const s = qs(filters);
        return request(`/expense/receipts${s ? '?' + s : ''}`);
    },
    bulkAttachReceipts: (body = {}) =>
        request('/expense/receipts/bulk-attach', {
            method: 'POST',
            body: JSON.stringify(body),
        }),
    bulkDeleteReceipts: (ids) =>
        request('/expense/receipts/bulk-delete', {
            method: 'POST',
            body: JSON.stringify({ ids }),
        }),
    bulkPatchReceiptItems: (body) =>
        request('/expense/receipts/bulk-patch-items', {
            method: 'POST',
            body: JSON.stringify(body),
        }),
    bulkReocr: (filter = 'non_vision') =>
        request('/expense/receipts/bulk-reocr', {
            method: 'POST',
            body: JSON.stringify({ filter }),
        }),
    reocrProgress: () => request('/expense/receipts/bulk-reocr/progress'),
    receiptsByMerchant: (params = {}) => {
        const s = qs(params);
        return request(`/expense/receipts/by-merchant${s ? '?' + s : ''}`);
    },
    topMerchants: (params = {}) => {
        const s = qs(params);
        return request(`/tax/merchants${s ? '?' + s : ''}`);
    },
    searchReceipts: (q, limit = 50) => {
        const s = qs({ q, limit });
        return request(`/expense/receipts/search?${s}`);
    },
    receiptDuplicates: (params = {}) => {
        const s = qs(params);
        return request(`/expense/receipts/duplicates${s ? '?' + s : ''}`);
    },

    // --- tax filing wizard ----------------------------------------------
    taxReturn:        (year) => request(`/tax-filing/returns/${year}`),
    saveTaxReturn:    (year, draft, status, change_label) => request(`/tax-filing/returns/${year}`, {
        method: 'PUT',
        body: JSON.stringify({ draft, status, change_label }),
    }),
    autopopulateTaxReturn: (year) => request(`/tax-filing/returns/${year}/autopopulate`, { method: 'POST' }),
    computeTaxReturn:      (year) => request(`/tax-filing/returns/${year}/compute`),
    taxReturnPdfUrl:       (year) => `${baseUrl}/api/tax-filing/returns/${year}/pdf`,
    uploadTaxForm: (file, tax_year) => {
        const fd = new FormData();
        fd.append('file', file, file.name);
        if (tax_year != null) fd.append('tax_year', String(tax_year));
        return request('/tax-filing/forms/upload', { method: 'POST', body: fd });
    },
    listTaxForms: (year) => request(`/tax-filing/forms/${year}`),
    taxSafeHarbor: (year, params = {}) => {
        const s = qs(params);
        return request(`/tax-filing/returns/${year}/safe-harbor${s ? '?' + s : ''}`);
    },
    taxWhatIf: (year, scenario) => request(`/tax-filing/returns/${year}/what-if`, {
        method: 'POST',
        body: JSON.stringify({ scenario }),
    }),
    taxLatePenalty: (year, input) => request(`/tax-filing/returns/${year}/late-penalty`, {
        method: 'POST',
        body: JSON.stringify(input),
    }),
    taxPlannerAotc: (input) => request('/tax-filing/planner/aotc', {
        method: 'POST',
        body: JSON.stringify(input),
    }),
    taxPlannerLlc: (input) => request('/tax-filing/planner/llc', {
        method: 'POST',
        body: JSON.stringify(input),
    }),
    taxPlannerIra: (input) => request('/tax-filing/planner/ira', {
        method: 'POST',
        body: JSON.stringify(input),
    }),
    taxPlannerRothIra: (input) => request('/tax-filing/planner/roth-ira', {
        method: 'POST',
        body: JSON.stringify(input),
    }),
    taxPlannerHsa: (input) => request('/tax-filing/planner/hsa', {
        method: 'POST',
        body: JSON.stringify(input),
    }),
    taxPlannerMileage: (input) => request('/tax-filing/planner/mileage', {
        method: 'POST',
        body: JSON.stringify(input),
    }),
    taxPlannerMileageCompare: (input) => request('/tax-filing/planner/mileage-compare', {
        method: 'POST',
        body: JSON.stringify(input),
    }),
    taxPlannerHomeOffice: (input) => request('/tax-filing/planner/home-office', {
        method: 'POST',
        body: JSON.stringify(input),
    }),
    taxPlannerSection179: (input) => request('/tax-filing/planner/section-179', {
        method: 'POST',
        body: JSON.stringify(input),
    }),
    receiptsRecurring: (params = {}) => {
        const s = qs(params);
        return request(`/expense/receipts/recurring${s ? '?' + s : ''}`);
    },
    receiptsSpendCalendar: (year) => {
        const s = year ? `?year=${year}` : '';
        return request(`/expense/receipts/spend-calendar${s}`);
    },
    receiptsDow: (params = {}) => {
        const s = qs(params);
        return request(`/expense/receipts/dow${s ? '?' + s : ''}`);
    },
    receiptsCumulative: (params = {}) => {
        const s = qs(params);
        return request(`/expense/receipts/cumulative${s ? '?' + s : ''}`);
    },
    expenseDashboardBundle: (year, businessId) => {
        const parts = [];
        if (year) parts.push(`year=${year}`);
        if (businessId) parts.push(`business_id=${encodeURIComponent(businessId)}`);
        return request(`/expense/receipts/dashboard-bundle${parts.length ? '?' + parts.join('&') : ''}`);
    },
    receiptsMonthCalendar: (year, month, businessId) => {
        const q = businessId ? `?business_id=${encodeURIComponent(businessId)}` : '';
        return request(`/expense/receipts/calendar/${year}/${month}${q}`);
    },
    receiptsYoyMonthly: (year, businessId) => {
        const parts = [];
        if (year) parts.push(`year=${year}`);
        if (businessId) parts.push(`business_id=${encodeURIComponent(businessId)}`);
        return request(`/expense/receipts/yoy-monthly${parts.length ? '?' + parts.join('&') : ''}`);
    },
    receiptsAging: (businessId) => {
        const q = businessId ? `?business_id=${encodeURIComponent(businessId)}` : '';
        return request(`/expense/receipts/aging${q}`);
    },
    receiptsByProperty: (year, businessId) => {
        const parts = [];
        if (year) parts.push(`year=${year}`);
        if (businessId) parts.push(`business_id=${encodeURIComponent(businessId)}`);
        return request(`/expense/receipts/by-property${parts.length ? '?' + parts.join('&') : ''}`);
    },
    receiptsAnomalies: (businessId) => {
        const q = businessId ? `?business_id=${encodeURIComponent(businessId)}` : '';
        return request(`/expense/receipts/anomalies${q}`);
    },
    receiptsCategoryDistribution: (year, businessId) => {
        const parts = [];
        if (year) parts.push(`year=${year}`);
        if (businessId) parts.push(`business_id=${encodeURIComponent(businessId)}`);
        return request(`/expense/receipts/category-distribution${parts.length ? '?' + parts.join('&') : ''}`);
    },
    // Multi-business CRUD
    businessesList: () => request('/businesses/'),
    businessCreate: (body) => request('/businesses/', {
        method: 'POST', body: JSON.stringify(body),
    }),
    businessPatch: (id, body) => request(`/businesses/${encodeURIComponent(id)}`, {
        method: 'PATCH', body: JSON.stringify(body),
    }),
    businessDelete: (id) => request(`/businesses/${encodeURIComponent(id)}`, {
        method: 'DELETE',
    }),
    businessSetDefault: (id) => request(`/businesses/${encodeURIComponent(id)}/set-default`, {
        method: 'PATCH', body: '{}',
    }),
    // Multi-broker CRUD
    brokersList: () => request('/brokers/'),
    brokerCreate: (body) => request('/brokers/', {
        method: 'POST', body: JSON.stringify(body),
    }),
    brokerPatch: (id, body) => request(`/brokers/${encodeURIComponent(id)}`, {
        method: 'PATCH', body: JSON.stringify(body),
    }),
    brokerDelete: (id) => request(`/brokers/${encodeURIComponent(id)}`, {
        method: 'DELETE',
    }),
    brokerSetDefault: (id) => request(`/brokers/${encodeURIComponent(id)}/set-default`, {
        method: 'PATCH', body: '{}',
    }),

    // --- budgeting ------------------------------------------------------
    listBudgets: () => request('/budget/'),
    upsertBudget: (code, body) => request(`/budget/categories/${encodeURIComponent(code)}`, {
        method: 'PUT', body: JSON.stringify(body),
    }),
    deleteBudget: (code) => request(`/budget/categories/${encodeURIComponent(code)}`, { method: 'DELETE' }),
    setSavingsGoal: (monthly_target) => request('/budget/savings-goal', {
        method: 'PUT', body: JSON.stringify({ monthly_target }),
    }),
    budgetSnapshot: (params = {}) => {
        const s = qs(params);
        return request(`/budget/snapshot${s ? '?' + s : ''}`);
    },
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
    ocrModelsStatus: () => request('/expense/receipts/ocr-models/status'),
    ocrModelsDownload: (opts = {}) => {
        const path = opts.force
            ? '/expense/receipts/ocr-models/download?force=1'
            : '/expense/receipts/ocr-models/download';
        return request(path, { method: 'POST' });
    },
    retryReceiptOcr: (id) => request(`/expense/receipts/${id}/retry-ocr`, { method: 'POST' }),
    patchReceiptMeta: (id, patch) =>
        request(`/expense/receipts/${id}/meta`, {
            method: 'PATCH',
            body: JSON.stringify(patch),
        }),
    patchReceiptItem: (id, idx, patch) =>
        request(`/expense/receipts/${id}/items/${idx}`, {
            method: 'PATCH',
            body: JSON.stringify(patch),
        }),
    addReceiptItem: (id, item) =>
        request(`/expense/receipts/${id}/items`, {
            method: 'POST',
            body: JSON.stringify(item),
        }),
    deleteReceiptItem: (id, idx) =>
        request(`/expense/receipts/${id}/items/${idx}`, { method: 'DELETE' }),
    taxRollup: (params = {}) => {
        const s = qs(params);
        return request(`/expense/receipts/tax-rollup${s ? '?' + s : ''}`);
    },
    taxRollupCsvUrl: (params = {}) => {
        const s = qs(params);
        return `${baseUrl}/api/expense/receipts/tax-rollup.csv${s ? '?' + s : ''}`;
    },
    taxRollupPdfUrl: (params = {}) => {
        const s = qs(params);
        return `${baseUrl}/api/expense/receipts/tax-rollup.pdf${s ? '?' + s : ''}`;
    },
    listEstimatedPayments: (params = {}) => {
        const s = qs(params);
        return request(`/tax/estimated-payments${s ? '?' + s : ''}`);
    },
    createEstimatedPayment: (body) =>
        request('/tax/estimated-payments', {
            method: 'POST', body: JSON.stringify(body),
        }),
    updateEstimatedPayment: (id, body) =>
        request(`/tax/estimated-payments/${id}`, {
            method: 'PATCH', body: JSON.stringify(body),
        }),
    deleteEstimatedPayment: (id) =>
        request(`/tax/estimated-payments/${id}`, { method: 'DELETE' }),
    setCategoryKind: (id, kind) =>
        request(`/tax/categories/${id}/kind`, {
            method: 'PATCH', body: JSON.stringify({ kind }),
        }),
    listPurchases: (filters = {}) => {
        const s = qs(filters);
        return request(`/tax/purchases${s ? '?' + s : ''}`);
    },
    monthlyTotals: (year) => request(`/tax/monthly-totals?year=${year}`),
    yoyTrend: (years = 5) => request(`/tax/yoy?years=${years}`),
    rentalProperties: () => request('/rental/properties'),
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

    // dashboards (per-user mirror of localStorage layouts)
    dashboards:        () => request('/dashboards'),
    createDashboard:   (body) => request('/dashboards', { method: 'POST', body: JSON.stringify(body) }),
    updateDashboard:   (id, body) => request(`/dashboards/${id}`, { method: 'PUT', body: JSON.stringify(body) }),
    deleteDashboard:   (id) => request(`/dashboards/${id}`, { method: 'DELETE' }),

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
    dividendsCalendar: (days = 14) => request(`/dividends/calendar${qs({ days })}`),
    symbolRecs:      (sym) => request(`/symbols/${encodeURIComponent(sym)}/recommendations`),
    // Internal composite Buy/Sell/Hold + stars + score + target. Mirrors
    // stockinvest.us's per-ticker surface but the algorithm is ours
    // (see crates/traderview-db/src/stock_recommendation.rs).
    symbolRecommendation:        (sym) => request(`/symbols/${encodeURIComponent(sym)}/recommendation`),
    symbolRecommendationBacktest:(sym, horizon = 22) => request(`/symbols/${encodeURIComponent(sym)}/recommendation/backtest${qs({ horizon })}`),
    recommendationLeaderboard:   ({ limit = 100, min_score = 0 } = {}) =>
        request(`/recommendations/golden-stars${qs({ limit, min_score })}`),
    recommendationSectors:       () => request('/recommendations/sectors'),
    recommendationWatchers:      () => request('/recommendations/watchers'),
    recommendationWatcherUpsert: (body) => request('/recommendations/watchers', { method: 'POST', body: JSON.stringify(body) }),
    recommendationWatcherDelete: (id) => request(`/recommendations/watchers/${encodeURIComponent(id)}`, { method: 'DELETE' }),
    // Classic valuation extras — Piotroski/Altman/Graham, DCF, gap
    // fill-rates, monthly seasonality.
    symbolFundamentalHealth: (sym) => request(`/symbols/${encodeURIComponent(sym)}/fundamental-health`),
    symbolGapStats:          (sym, threshold = 0.5) => request(`/symbols/${encodeURIComponent(sym)}/gap-stats${qs({ threshold })}`),
    symbolSeasonality:       (sym) => request(`/symbols/${encodeURIComponent(sym)}/seasonality`),
    calcDcf:                 (body) => request('/calc/dcf', { method: 'POST', body: JSON.stringify(body) }),
    calcReverseDcf:          (body) => request('/calc/reverse-dcf', { method: 'POST', body: JSON.stringify(body) }),
    calcDdm:                 (body) => request('/calc/ddm', { method: 'POST', body: JSON.stringify(body) }),
    calcEpv:                 (body) => request('/calc/epv', { method: 'POST', body: JSON.stringify(body) }),
    calcWheel:               (body) => request('/calc/wheel', { method: 'POST', body: JSON.stringify(body) }),
    symbolRuleOf40:          (sym) => request(`/symbols/${encodeURIComponent(sym)}/rule-of-40`),
    rrg:                     () => request('/rrg'),
    symbolBeneish:           (sym) => request(`/symbols/${encodeURIComponent(sym)}/beneish`),
    symbolDeepValue:         (sym) => request(`/symbols/${encodeURIComponent(sym)}/deep-value`),
    symbolChowder:           (sym) => request(`/symbols/${encodeURIComponent(sym)}/chowder`),
    marketFedModel:          () => request('/market/fed-model'),
    marketNhNl:              () => request('/market/nh-nl'),
    simValueAveraging:       (body) => request('/sim/value-averaging', { method: 'POST', body: JSON.stringify(body) }),
    simCppi:                 (body) => request('/sim/cppi', { method: 'POST', body: JSON.stringify(body) }),
    // --- strategy tools (grid / fixed-ratio / anti-martingale / GEM / TOM)
    calcGridTrading:         (body) => request('/calc/grid-trading', { method: 'POST', body: JSON.stringify(body) }),
    calcFixedRatio:          (body) => request('/calc/fixed-ratio', { method: 'POST', body: JSON.stringify(body) }),
    calcAntiMartingale:      (body) => request('/calc/anti-martingale', { method: 'POST', body: JSON.stringify(body) }),
    calcRiskOfRuin:          (body) => request('/calc/risk-of-ruin', { method: 'POST', body: JSON.stringify(body) }),
    calcTaylorRule:          (body) => request('/calc/taylor-rule', { method: 'POST', body: JSON.stringify(body) }),
    calcSahmRule:            (body) => request('/calc/sahm-rule', { method: 'POST', body: JSON.stringify(body) }),
    calcMiseryIndex:         (body) => request('/calc/misery-index', { method: 'POST', body: JSON.stringify(body) }),
    calcValuationGauges:     (body) => request('/calc/valuation-gauges', { method: 'POST', body: JSON.stringify(body) }),
    calcVarianceRiskPremium: (body) => request('/calc/variance-risk-premium', { method: 'POST', body: JSON.stringify(body) }),
    calcScaleOut:            (body) => request('/calc/scale-out', { method: 'POST', body: JSON.stringify(body) }),
    calcTaxAwareRebalance:   (body) => request('/calc/tax-aware-rebalance', { method: 'POST', body: JSON.stringify(body) }),
    calcSavingsWaterfall:    (body) => request('/calc/savings-waterfall', { method: 'POST', body: JSON.stringify(body) }),
    calcHouseHacking:        (body) => request('/calc/house-hacking', { method: 'POST', body: JSON.stringify(body) }),
    calcBrrrr:               (body) => request('/calc/brrrr', { method: 'POST', body: JSON.stringify(body) }),
    calcPaycheck401k:        (body) => request('/calc/paycheck-401k', { method: 'POST', body: JSON.stringify(body) }),
    calcGuytonKlinger:       (body) => request('/calc/guyton-klinger', { method: 'POST', body: JSON.stringify(body) }),
    calcIrmaa:               (body) => request('/calc/irmaa', { method: 'POST', body: JSON.stringify(body) }),
    calcBreakEven:           (body) => request('/calc/break-even', { method: 'POST', body: JSON.stringify(body) }),
    calcLeaseGenerator:      (body) => request('/calc/lease-generator', { method: 'POST', body: JSON.stringify(body) }),
    calcInvoiceGenerator:    (body) => request('/calc/invoice-generator', { method: 'POST', body: JSON.stringify(body) }),
    calcLandlordNotice:      (body) => request('/calc/landlord-notice', { method: 'POST', body: JSON.stringify(body) }),
    calcFixAndFlip:          (body) => request('/calc/fix-and-flip', { method: 'POST', body: JSON.stringify(body) }),
    calcCashConversionCycle: (body) => request('/calc/cash-conversion-cycle', { method: 'POST', body: JSON.stringify(body) }),
    calcProfitFirst:         (body) => request('/calc/profit-first', { method: 'POST', body: JSON.stringify(body) }),
    calcMarkupMargin:        (body) => request('/calc/markup-margin', { method: 'POST', body: JSON.stringify(body) }),
    calcInventoryEoq:        (body) => request('/calc/inventory-eoq', { method: 'POST', body: JSON.stringify(body) }),
    calcRentVsSell:          (body) => request('/calc/rent-vs-sell', { method: 'POST', body: JSON.stringify(body) }),
    calcDepreciationRecapture: (body) => request('/calc/depreciation-recapture', { method: 'POST', body: JSON.stringify(body) }),
    calcLikeKindExchange:    (body) => request('/calc/like-kind-exchange', { method: 'POST', body: JSON.stringify(body) }),
    calcCostOfHire:          (body) => request('/calc/cost-of-hire', { method: 'POST', body: JSON.stringify(body) }),
    calcInvoiceFactoring:    (body) => request('/calc/invoice-factoring', { method: 'POST', body: JSON.stringify(body) }),
    calcLtvCac:              (body) => request('/calc/ltv-cac', { method: 'POST', body: JSON.stringify(body) }),
    calcBurnRate:            (body) => request('/calc/burn-rate', { method: 'POST', body: JSON.stringify(body) }),
    calcQlac:                (body) => request('/calc/qlac', { method: 'POST', body: JSON.stringify(body) }),
    calcSpousalIra:          (body) => request('/calc/spousal-ira', { method: 'POST', body: JSON.stringify(body) }),
    calcPensionSurvivor:     (body) => request('/calc/pension-survivor', { method: 'POST', body: JSON.stringify(body) }),
    calcSsPia:               (body) => request('/calc/ss-pia', { method: 'POST', body: JSON.stringify(body) }),
    calcHsaTripleTax:        (body) => request('/calc/hsa-triple-tax', { method: 'POST', body: JSON.stringify(body) }),
    calcAgeAllocation:       (body) => request('/calc/age-allocation', { method: 'POST', body: JSON.stringify(body) }),
    calcRothBracketFill:     (body) => request('/calc/roth-bracket-fill', { method: 'POST', body: JSON.stringify(body) }),
    calcMortgagePoints:      (body) => request('/calc/mortgage-points', { method: 'POST', body: JSON.stringify(body) }),
    calcAprApy:              (body) => request('/calc/apr-apy', { method: 'POST', body: JSON.stringify(body) }),
    calcBlendedDebt:         (body) => request('/calc/blended-debt', { method: 'POST', body: JSON.stringify(body) }),
    calcDividendCoverage:    (body) => request('/calc/dividend-coverage', { method: 'POST', body: JSON.stringify(body) }),
    calcSpia:                (body) => request('/calc/spia', { method: 'POST', body: JSON.stringify(body) }),
    calcDebtYield:           (body) => request('/calc/debt-yield', { method: 'POST', body: JSON.stringify(body) }),
    calcPriceToRent:         (body) => request('/calc/price-to-rent', { method: 'POST', body: JSON.stringify(body) }),
    calcYearsToFi:           (body) => request('/calc/years-to-fi', { method: 'POST', body: JSON.stringify(body) }),
    calcGrm:                 (body) => request('/calc/grm', { method: 'POST', body: JSON.stringify(body) }),
    calcSellerFinancing:     (body) => request('/calc/seller-financing', { method: 'POST', body: JSON.stringify(body) }),
    calcExpenseDrag:         (body) => request('/calc/expense-drag', { method: 'POST', body: JSON.stringify(body) }),
    calcLeasePayment:        (body) => request('/calc/lease-payment', { method: 'POST', body: JSON.stringify(body) }),
    calcRealReturn:          (body) => request('/calc/real-return', { method: 'POST', body: JSON.stringify(body) }),
    calcCdPenalty:           (body) => request('/calc/cd-penalty', { method: 'POST', body: JSON.stringify(body) }),
    calcYieldOnCost:         (body) => request('/calc/yield-on-cost', { method: 'POST', body: JSON.stringify(body) }),
    calcTradeExpectancy:     (body) => request('/calc/trade-expectancy', { method: 'POST', body: JSON.stringify(body) }),
    calcWageConverter:       (body) => request('/calc/wage-converter', { method: 'POST', body: JSON.stringify(body) }),
    calcSalesTax:            (body) => request('/calc/sales-tax', { method: 'POST', body: JSON.stringify(body) }),
    calcAccruedInterest:     (body) => request('/calc/accrued-interest', { method: 'POST', body: JSON.stringify(body) }),
    calcStockSplit:          (body) => request('/calc/stock-split', { method: 'POST', body: JSON.stringify(body) }),
    calcTbillYield:          (body) => request('/calc/tbill-yield', { method: 'POST', body: JSON.stringify(body) }),
    calcDscr:                (body) => request('/calc/dscr', { method: 'POST', body: JSON.stringify(body) }),
    calcGrahamNumber:        (body) => request('/calc/graham-number', { method: 'POST', body: JSON.stringify(body) }),
    calcTakeHomePaycheck:    (body) => request('/calc/take-home-paycheck', { method: 'POST', body: JSON.stringify(body) }),
    calcEvEbitda:            (body) => request('/calc/ev-ebitda', { method: 'POST', body: JSON.stringify(body) }),
    calcHoldingPeriodReturn: (body) => request('/calc/holding-period-return', { method: 'POST', body: JSON.stringify(body) }),
    calcAltmanZScore:        (body) => request('/calc/altman-z-score', { method: 'POST', body: JSON.stringify(body) }),
    calcPiotroskiFScore:     (body) => request('/calc/piotroski-f-score', { method: 'POST', body: JSON.stringify(body) }),
    calcGmroi:               (body) => request('/calc/gmroi', { method: 'POST', body: JSON.stringify(body) }),
    calcRothContribution:    (body) => request('/calc/roth-contribution', { method: 'POST', body: JSON.stringify(body) }),
    calcInterestCoverage:    (body) => request('/calc/interest-coverage', { method: 'POST', body: JSON.stringify(body) }),
    calcCapitalGainsTax:     (body) => request('/calc/capital-gains-tax', { method: 'POST', body: JSON.stringify(body) }),
    calcTraditionalIraDeduction: (body) => request('/calc/traditional-ira-deduction', { method: 'POST', body: JSON.stringify(body) }),
    calcRuleOf40:            (body) => request('/calc/rule-of-40', { method: 'POST', body: JSON.stringify(body) }),
    calcWacc:                (body) => request('/calc/wacc', { method: 'POST', body: JSON.stringify(body) }),
    calcDupontRoe:           (body) => request('/calc/dupont-roe', { method: 'POST', body: JSON.stringify(body) }),
    calcSsTaxation:          (body) => request('/calc/ss-taxation', { method: 'POST', body: JSON.stringify(body) }),
    calcNpvIrr:              (body) => request('/calc/npv-irr', { method: 'POST', body: JSON.stringify(body) }),
    calcLeverage:            (body) => request('/calc/leverage', { method: 'POST', body: JSON.stringify(body) }),
    calcTwoAssetPortfolio:   (body) => request('/calc/two-asset-portfolio', { method: 'POST', body: JSON.stringify(body) }),
    calcMortgageRecast:      (body) => request('/calc/mortgage-recast', { method: 'POST', body: JSON.stringify(body) }),
    calcTaxEquivalentYield:  (body) => request('/calc/tax-equivalent-yield', { method: 'POST', body: JSON.stringify(body) }),
    calcPmiRemoval:          (body) => request('/calc/pmi-removal', { method: 'POST', body: JSON.stringify(body) }),
    calcFreeCashFlow:        (body) => request('/calc/free-cash-flow', { method: 'POST', body: JSON.stringify(body) }),
    calcCreditCardPayoff:    (body) => request('/calc/credit-card-payoff', { method: 'POST', body: JSON.stringify(body) }),
    calcBondPricing:         (body) => request('/calc/bond-pricing', { method: 'POST', body: JSON.stringify(body) }),
    calcCashOutRefinance:    (body) => request('/calc/cash-out-refinance', { method: 'POST', body: JSON.stringify(body) }),
    calcMarginAnalysis:      (body) => request('/calc/margin-analysis', { method: 'POST', body: JSON.stringify(body) }),
    calcBonusGrossup:        (body) => request('/calc/bonus-grossup', { method: 'POST', body: JSON.stringify(body) }),
    calcRentEscalation:      (body) => request('/calc/rent-escalation', { method: 'POST', body: JSON.stringify(body) }),
    calcLoanApr:             (body) => request('/calc/loan-apr', { method: 'POST', body: JSON.stringify(body) }),
    calcHomeSaleExclusion:   (body) => request('/calc/home-sale-exclusion', { method: 'POST', body: JSON.stringify(body) }),
    calcLifeInsuranceNeeds:  (body) => request('/calc/life-insurance-needs', { method: 'POST', body: JSON.stringify(body) }),
    calcCarAffordability:    (body) => request('/calc/car-affordability', { method: 'POST', body: JSON.stringify(body) }),
    calcDisabilityInsuranceNeeds: (body) => request('/calc/disability-insurance-needs', { method: 'POST', body: JSON.stringify(body) }),
    calcTrueHourlyWage:      (body) => request('/calc/true-hourly-wage', { method: 'POST', body: JSON.stringify(body) }),
    calcPropertyTax:         (body) => request('/calc/property-tax', { method: 'POST', body: JSON.stringify(body) }),
    calcRentalNoi:           (body) => request('/calc/rental-noi', { method: 'POST', body: JSON.stringify(body) }),
    calcMortgageAffordability: (body) => request('/calc/mortgage-affordability', { method: 'POST', body: JSON.stringify(body) }),
    calcOvertimePay:         (body) => request('/calc/overtime-pay', { method: 'POST', body: JSON.stringify(body) }),
    calcSolarPayback:        (body) => request('/calc/solar-payback', { method: 'POST', body: JSON.stringify(body) }),
    calcPortfolioLongevity:  (body) => request('/calc/portfolio-longevity', { method: 'POST', body: JSON.stringify(body) }),
    calcSecondIncome:        (body) => request('/calc/second-income', { method: 'POST', body: JSON.stringify(body) }),
    calcBreakevenOccupancy:  (body) => request('/calc/breakeven-occupancy', { method: 'POST', body: JSON.stringify(body) }),
    calcRentAffordability:   (body) => request('/calc/rent-affordability', { method: 'POST', body: JSON.stringify(body) }),
    calcRealRaise:           (body) => request('/calc/real-raise', { method: 'POST', body: JSON.stringify(body) }),
    calcSdeValuation:        (body) => request('/calc/sde-valuation', { method: 'POST', body: JSON.stringify(body) }),
    calcFreelanceRate:       (body) => request('/calc/freelance-rate', { method: 'POST', body: JSON.stringify(body) }),
    calcPreferredStock:      (body) => request('/calc/preferred-stock', { method: 'POST', body: JSON.stringify(body) }),
    calcMarginInterest:      (body) => request('/calc/margin-interest', { method: 'POST', body: JSON.stringify(body) }),
    calcQbiDeduction:        (body) => request('/calc/qbi-deduction', { method: 'POST', body: JSON.stringify(body) }),
    calcEstateTax:           (body) => request('/calc/estate-tax', { method: 'POST', body: JSON.stringify(body) }),
    calcMarriagePenalty:     (body) => request('/calc/marriage-penalty', { method: 'POST', body: JSON.stringify(body) }),
    calcStdVsItemized:       (body) => request('/calc/standard-vs-itemized', { method: 'POST', body: JSON.stringify(body) }),
    calcCaptureRatio:        (body) => request('/calc/capture-ratio', { method: 'POST', body: JSON.stringify(body) }),
    calcMergerArb:           (body) => request('/calc/merger-arb', { method: 'POST', body: JSON.stringify(body) }),
    calcBuybackAccretion:    (body) => request('/calc/buyback-accretion', { method: 'POST', body: JSON.stringify(body) }),
    calcCefDiscount:         (body) => request('/calc/cef-discount', { method: 'POST', body: JSON.stringify(body) }),
    calcAdrPremium:          (body) => request('/calc/adr-premium', { method: 'POST', body: JSON.stringify(body) }),
    calcSbcDilution:         (body) => request('/calc/sbc-dilution', { method: 'POST', body: JSON.stringify(body) }),
    calcSumOfParts:          (body) => request('/calc/sum-of-parts', { method: 'POST', body: JSON.stringify(body) }),
    calcOddLotTender:        (body) => request('/calc/odd-lot-tender', { method: 'POST', body: JSON.stringify(body) }),
    calcCrackSpread:         (body) => request('/calc/crack-spread', { method: 'POST', body: JSON.stringify(body) }),
    calcCrushSpread:         (body) => request('/calc/crush-spread', { method: 'POST', body: JSON.stringify(body) }),
    calcSparkSpread:         (body) => request('/calc/spark-spread', { method: 'POST', body: JSON.stringify(body) }),
    calcCurveTrade:          (body) => request('/calc/curve-trade', { method: 'POST', body: JSON.stringify(body) }),
    calcCheapestToDeliver:   (body) => request('/calc/cheapest-to-deliver', { method: 'POST', body: JSON.stringify(body) }),
    calcRebalanceBands:      (body) => request('/calc/rebalance-bands', { method: 'POST', body: JSON.stringify(body) }),
    calcIvCone:              (body) => request('/calc/iv-cone', { method: 'POST', body: JSON.stringify(body) }),
    calcFundFees:            (body) => request('/calc/fund-fees', { method: 'POST', body: JSON.stringify(body) }),
    calcWinRateConfidence:   (body) => request('/calc/win-rate-confidence', { method: 'POST', body: JSON.stringify(body) }),
    calcEquityCurveFilter:   (body) => request('/calc/equity-curve-filter', { method: 'POST', body: JSON.stringify(body) }),
    tradeReportCard:         (body) => request('/analytics/report-card', { method: 'POST', body: JSON.stringify(body) }),
    thirteenFDiff:           (cik) => request(`/13f/${encodeURIComponent(cik)}/diff`),
    futuresCurve:            (root, exchange = 'NYM', months = 8) => request(`/futures/${encodeURIComponent(root)}/curve${qs({ exchange, months })}`),
    carryScreen:             (months = 6) => request(`/futures/carry-screen${qs({ months })}`),
    screenerSnapshot:        (name) => request(`/screeners/snapshots/${encodeURIComponent(name)}`),
    paperParentOrderCreate:  (accountId, body) => request(`/paper/accounts/${encodeURIComponent(accountId)}/parent-orders`, { method: 'POST', body: JSON.stringify(body) }),
    paperParentOrders:       () => request('/paper/parent-orders'),
    paperParentOrderCancel:  (id) => request(`/paper/parent-orders/${encodeURIComponent(id)}/cancel`, { method: 'POST' }),
    calcDoubleBarrier:       (body) => request('/calc/double-barrier', { method: 'POST', body: JSON.stringify(body) }),
    calcFuturesSizing:       (body) => request('/calc/futures-sizing', { method: 'POST', body: JSON.stringify(body) }),
    calcImpermanentLoss:     (body) => request('/calc/impermanent-loss', { method: 'POST', body: JSON.stringify(body) }),
    cryptoFundingArb:        (body) => request('/crypto/calc/funding-arb', { method: 'POST', body: JSON.stringify(body) }),
    cryptoFundingArbLive:    (body) => request('/crypto/calc/funding-arb-live', { method: 'POST', body: JSON.stringify(body) }),
    cryptoFundingScan:       (bases) => request(`/crypto/funding-scan${bases ? `?bases=${encodeURIComponent(bases)}` : ''}`),
    cryptoPositioning: (body) => request('/crypto/positioning', { method: 'POST', body }),
    cryptoCarryBasis: (body) => request('/crypto/carry-basis', { method: 'POST', body }),
    cryptoVolSurface: (body) => request('/crypto/vol-surface', { method: 'POST', body }),
    cryptoVrp: (body) => request('/crypto/vrp', { method: 'POST', body }),
    cryptoBookDepth: (body) => request('/crypto/book-depth', { method: 'POST', body }),
    calcFxCarry:             (body) => request('/calc/fx-carry', { method: 'POST', body: JSON.stringify(body) }),
    calcAverageDown:         (body) => request('/calc/average-down', { method: 'POST', body: JSON.stringify(body) }),
    calcLeveragedEtfDecay:   (body) => request('/calc/leveraged-etf-decay', { method: 'POST', body: JSON.stringify(body) }),
    calcShortCarry:          (body) => request('/calc/short-carry', { method: 'POST', body: JSON.stringify(body) }),
    calcAssetLocation:       (body) => request('/calc/asset-location', { method: 'POST', body: JSON.stringify(body) }),
    calcAlphaHorizon:        (body) => request('/calc/alpha-horizon', { method: 'POST', body: JSON.stringify(body) }),
    calcOptionsQuickMath:    (body) => request('/calc/options-quick-math', { method: 'POST', body: JSON.stringify(body) }),
    calcLynchFairValue:      (body) => request('/calc/lynch-fair-value', { method: 'POST', body: JSON.stringify(body) }),
    overnightSplit:          (sym, years = 10) => request(`/symbols/${encodeURIComponent(sym)}/overnight-split${qs({ years })}`),
    bestDays:                (sym, years = 10, n = 10) => request(`/symbols/${encodeURIComponent(sym)}/best-days${qs({ years, n })}`),
    drawdownEpisodes:        (sym, years = 10, n = 5) => request(`/symbols/${encodeURIComponent(sym)}/drawdown-episodes${qs({ years, n })}`),
    opexWeek:                (sym, years = 10, quarterly = false) => request(`/symbols/${encodeURIComponent(sym)}/opex-week${qs({ years, quarterly })}`),
    exDivStudy:              (sym, years = 10) => request(`/symbols/${encodeURIComponent(sym)}/ex-div-study${qs({ years })}`),
    splitStudy:              (sym, years = 15) => request(`/symbols/${encodeURIComponent(sym)}/split-study${qs({ years })}`),
    volRichCheap:            (sym, body) => request(`/symbols/${encodeURIComponent(sym)}/vol-rich-cheap`, { method: 'POST', body: JSON.stringify(body) }),
    characterSheet:          (sym, years = 10) => request(`/symbols/${encodeURIComponent(sym)}/character-sheet${qs({ years })}`),
    seasonalityScreen:       (body) => request('/screeners/seasonality', { method: 'POST', body: JSON.stringify(body) }),
    riskScreen:              (body) => request('/screeners/risk', { method: 'POST', body: JSON.stringify(body) }),
    momentumScreen:          (body) => request('/screeners/momentum', { method: 'POST', body: JSON.stringify(body) }),
    meanReversionScreen:     (body) => request('/screeners/mean-reversion', { method: 'POST', body: JSON.stringify(body) }),
    preHoliday:              (sym, years = 5) => request(`/symbols/${encodeURIComponent(sym)}/pre-holiday${qs({ years })}`),
    eventStudy:              (sym, body) => request(`/symbols/${encodeURIComponent(sym)}/event-study`, { method: 'POST', body: JSON.stringify(body) }),
    calcImpliedDividend:     (body) => request('/options/calc/implied-dividend', { method: 'POST', body: JSON.stringify(body) }),
    calcWarrant:             (body) => request('/options/calc/warrant', { method: 'POST', body: JSON.stringify(body) }),
    calcEarlyAssignment:     (body) => request('/options/calc/early-assignment', { method: 'POST', body: JSON.stringify(body) }),
    calcEventVol:            (body) => request('/options/calc/event-vol', { method: 'POST', body: JSON.stringify(body) }),
    calcGammaThetaBreakeven: (body) => request('/options/calc/gamma-theta-breakeven', { method: 'POST', body: JSON.stringify(body) }),
    simDualMomentum:         (body) => request('/sim/dual-momentum', { method: 'POST', body: JSON.stringify(body) }),
    turnOfMonth:             (sym, years = 10) => request(`/symbols/${encodeURIComponent(sym)}/turn-of-month${qs({ years })}`),
    volCone:                 (sym, years = 5) => request(`/symbols/${encodeURIComponent(sym)}/vol-cone${qs({ years })}`),
    dayOfWeekSeasonality:    (sym, years = 10) => request(`/symbols/${encodeURIComponent(sym)}/day-of-week${qs({ years })}`),
    santaRally:              (sym, years = 15) => request(`/symbols/${encodeURIComponent(sym)}/santa-rally${qs({ years })}`),
    correlationRegime:       (a, b, window = 63, years = 5) => request(`/correlation/regime${qs({ a, b, window, years })}`),
    pairSheet:               (a, b, years = 5) => request(`/correlation/pair-sheet${qs({ a, b, years })}`),
    calcConversionReversal:  (body) => request('/options/calc/conversion-reversal', { method: 'POST', body: JSON.stringify(body) }),
    calcSeagull:             (body) => request('/options/calc/seagull', { method: 'POST', body: JSON.stringify(body) }),
    calcHeston:              (body) => request('/options/calc/heston', { method: 'POST', body: JSON.stringify(body) }),
    calcHestonCalibrate:     (body) => request('/options/calc/heston-calibrate', { method: 'POST', body: JSON.stringify(body) }),
    calcDiagonalSpread:      (body) => request('/options/calc/diagonal-spread', { method: 'POST', body: JSON.stringify(body) }),
    calcProbabilityOfProfit: (body) => request('/options/calc/probability-of-profit', { method: 'POST', body: JSON.stringify(body) }),
    calcBootstrapZeroCurve:  (body) => request('/bonds/calc/bootstrap-zero-curve', { method: 'POST', body: JSON.stringify(body) }),
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
    paperOrderCancel: (id) => request(`/paper/orders/${encodeURIComponent(id)}/cancel`, { method: 'POST' }),
    paperBracketCreate: (id, req) =>
        request(`/paper/accounts/${id}/brackets`, { method: 'POST', body: JSON.stringify(req) }),
    paperScaleCreate: (id, req) =>
        request(`/paper/accounts/${id}/scale-orders`, { method: 'POST', body: JSON.stringify(req) }),
    paperSpreadCreate: (id, req) =>
        request(`/paper/accounts/${id}/spreads`, { method: 'POST', body: JSON.stringify(req) }),
    paperOptionGreeks: (id) => request(`/paper/accounts/${id}/option-greeks`),
    paperSpreadPreview: (req) =>
        request('/paper/spreads/preview', { method: 'POST', body: JSON.stringify(req) }),
    paperRecurringCreate: (id, req) =>
        request(`/paper/accounts/${id}/recurring`, { method: 'POST', body: JSON.stringify(req) }),
    paperRecurringList: () => request('/paper/recurring'),
    paperRecurringToggle: (id, enabled) =>
        request(`/paper/recurring/${id}/toggle`, { method: 'POST', body: JSON.stringify({ enabled }) }),
    paperRecurringDelete: (id) => request(`/paper/recurring/${id}`, { method: 'DELETE' }),
    paperSetDrip: (id, enabled) =>
        request(`/paper/accounts/${id}/drip`, { method: 'POST', body: JSON.stringify({ enabled }) }),
    paperSetCashApy: (id, apy_pct) =>
        request(`/paper/accounts/${id}/cash-apy`, { method: 'POST', body: JSON.stringify({ apy_pct }) }),
    paperSetBorrowApy: (id, apy_pct) =>
        request(`/paper/accounts/${id}/borrow-apy`, { method: 'POST', body: JSON.stringify({ apy_pct }) }),
    paperSetMargin: (id, multiplier) =>
        request(`/paper/accounts/${id}/margin`, { method: 'POST', body: JSON.stringify({ multiplier }) }),
    paperSetMarginApy: (id, apy_pct) =>
        request(`/paper/accounts/${id}/margin-apy`, { method: 'POST', body: JSON.stringify({ apy_pct }) }),
    paperSetAutoLiquidate: (id, enabled) =>
        request(`/paper/accounts/${id}/auto-liquidate`, { method: 'POST', body: JSON.stringify({ enabled }) }),
    paperInterest: (id) => request(`/paper/accounts/${id}/interest`),
    paperStatement: (id, month) => request(`/paper/accounts/${id}/statement?month=${encodeURIComponent(month)}`),
    paperHoldings: () => request('/paper/holdings'),
    paperAccountCorrelations: () => request('/paper/account-correlations'),
    paperPdt: (id) => request(`/paper/accounts/${id}/pdt`),
    paperStopSuggestion: (symbol) => request(`/paper/stop-suggestion?symbol=${encodeURIComponent(symbol)}`),
    paperCashFlows: (id) => request(`/paper/accounts/${id}/cash-flows`),
    paperTransfer: (from, to, amount) =>
        request('/paper/transfers', { method: 'POST', body: JSON.stringify({ from, to, amount }) }),
    paperTransferPosition: (from, to, symbol, qty) =>
        request('/paper/transfers/position', { method: 'POST', body: JSON.stringify({ from, to, symbol, qty }) }),
    paperCashFlow: (id, amount, note) =>
        request(`/paper/accounts/${id}/cash-flows`, { method: 'POST', body: JSON.stringify({ amount, note }) }),
    paperAttribution: (id) => request(`/paper/accounts/${id}/attribution`),
    paperWashSales: (id) => request(`/paper/accounts/${id}/wash-sales`),
    paperProtect: (id, body) => request(`/paper/accounts/${id}/protect`, { method: 'POST', body }),
    paperReplace: (orderId, body) => request(`/paper/orders/${orderId}/replace`, { method: 'POST', body }),
    paperRoll: (id, body) => request(`/paper/accounts/${id}/roll`, { method: 'POST', body }),
    paperCoveredCall: (id, body) => request(`/paper/accounts/${id}/covered-call`, { method: 'POST', body }),
    paperExercise: (id, body) => request(`/paper/accounts/${id}/exercise`, { method: 'POST', body }),
    paperAssign: (id, body) => request(`/paper/accounts/${id}/assign`, { method: 'POST', body }),
    paperCorrelations: (id, lookback = 90) => request(`/paper/accounts/${id}/correlations?lookback_days=${lookback}`),
    paperVar: (id, lookback = 365) => request(`/paper/accounts/${id}/var?lookback_days=${lookback}`),
    paperStress: (id, lookback = 365, benchmark) => request(`/paper/accounts/${id}/stress?lookback_days=${lookback}${benchmark ? `&benchmark=${encodeURIComponent(benchmark)}` : ''}`),
    paperEquityHistory: (id, benchmark) => request(`/paper/accounts/${id}/equity-history${benchmark ? `?benchmark=${encodeURIComponent(benchmark)}` : ''}`),
    paperAccountCreate: (name, starting_cash) =>
        request('/paper/accounts/create', { method: 'POST', body: JSON.stringify({ name, starting_cash }) }),
    paperAccountRename: (id, name) =>
        request(`/paper/accounts/${id}/rename`, { method: 'POST', body: JSON.stringify({ name }) }),
    paperAccountDelete: (id) => request(`/paper/accounts/${id}/delete`, { method: 'POST' }),
    paperAccountComparison: () => request('/paper/accounts/comparison'),
    paperDividends: (id) => request(`/paper/accounts/${id}/dividends`),
    paperSplits: (id) => request(`/paper/accounts/${id}/splits`),

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

    // Forex (shared Yahoo quote seam + forex_calc desk math)
    forexPairs:        () => request('/forex/pairs'),
    forexSessions:     () => request('/forex/sessions'),
    forexPipValue:     (body) => request('/forex/pip-value', { method: 'POST', body: JSON.stringify(body) }),
    forexPositionSize: (body) => request('/forex/position-size', { method: 'POST', body: JSON.stringify(body) }),

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
    // Finnhub /stock/short-interest — semimonthly settlement series.
    // Use this alongside `shortFinra` (daily Reg SHO volume) for the
    // long-tail + short-tail view of shorting activity.
    finnhubShortInterest: (sym, from, to) =>
        request(`/symbols/${encodeURIComponent(sym)}/short-interest${qs({ from, to })}`),

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

    // Algo momentum strategies (CRUD + run lifecycle + kill switch)
    listAlgoStrategies: () => request('/algo/strategies'),
    createAlgoStrategy: (body) =>
        request('/algo/strategies', { method: 'POST', body: JSON.stringify(body) }),
    updateAlgoStrategy: (id, body) =>
        request(`/algo/strategies/${id}`, { method: 'PUT', body: JSON.stringify(body) }),
    deleteAlgoStrategy: (id) =>
        request(`/algo/strategies/${id}`, { method: 'DELETE' }),
    setAlgoKillSwitch: (id, engaged, reason = null) =>
        request(`/algo/strategies/${id}/kill-switch`, {
            method: 'POST',
            body: JSON.stringify({ engaged, reason }),
        }),
    algoKillHistory: (id) => request(`/algo/strategies/${id}/kill-history`),
    listAlgoRuns: (id, limit = 25) =>
        request(`/algo/strategies/${id}/runs?limit=${limit}`),
    startAlgoRun: (id) =>
        request(`/algo/strategies/${id}/runs`, { method: 'POST' }),
    stopAlgoRun: (id, reason = 'user') =>
        request(`/algo/strategies/${id}/stop`, {
            method: 'POST',
            body: JSON.stringify({ reason }),
        }),
    listAlgoOrders: (runId, limit = 100) =>
        request(`/algo/runs/${runId}/orders?limit=${limit}`),
    listAlgoFills: (orderId) => request(`/algo/orders/${orderId}/fills`),
    backtestAlgoStrategy: (id, body) =>
        request(`/algo/strategies/${id}/backtest`, {
            method: 'POST',
            body: JSON.stringify(body || {}),
        }),
    algoStrategyMetrics: (id) => request(`/algo/strategies/${id}/metrics`),
    listAlgoBacktests: (id, limit = 50) =>
        request(`/algo/strategies/${id}/backtests?limit=${limit}`),
    deleteAlgoBacktest: (id) =>
        request(`/algo/backtests/${id}`, { method: 'DELETE' }),
    optimizeAlgoStrategy: (id, body) =>
        request(`/algo/strategies/${id}/optimize`, {
            method: 'POST',
            body: JSON.stringify(body || {}),
        }),
    algoTournament: (body) =>
        request('/algo/tournament', {
            method: 'POST',
            body: JSON.stringify(body || {}),
        }),
    algoPortfolio: (body) =>
        request('/algo/portfolio', {
            method: 'POST',
            body: JSON.stringify(body || {}),
        }),
    algoTournamentMatrix: (body) =>
        request('/algo/tournament-matrix', {
            method: 'POST',
            body: JSON.stringify(body || {}),
        }),
    algoWalkForward: (id, body) =>
        request(`/algo/strategies/${id}/walk-forward`, {
            method: 'POST',
            body: JSON.stringify(body || {}),
        }),
    algoBacktestMc: (id, body) =>
        request(`/algo/strategies/${id}/backtest-mc`, {
            method: 'POST',
            body: JSON.stringify(body || {}),
        }),
    algoBacktestRegimes: (id, body) =>
        request(`/algo/strategies/${id}/backtest-regimes`, {
            method: 'POST',
            body: JSON.stringify(body || {}),
        }),
    algoLiveVsBacktest: (id) => request(`/algo/strategies/${id}/live-vs-backtest`),
    algoGateFires: (id, windowDays = 7) => request(`/algo/strategies/${id}/gate-fires?window_days=${windowDays}`),
    algoPnlCurve: (id) => request(`/algo/strategies/${id}/pnl-curve`),
    algoRevisions: (id) => request(`/algo/strategies/${id}/revisions`),
    algoRestoreRevision: (id, revId) =>
        request(`/algo/strategies/${id}/revisions/${revId}/restore`, { method: 'POST' }),
    algoForkStrategy: (id, name) =>
        request(`/algo/strategies/${id}/fork`, { method: 'POST', body: JSON.stringify(name ? { name } : {}) }),
    algoExportStrategy: (id) => request(`/algo/strategies/${id}/export`),
    algoImportStrategy: (body) => request('/algo/strategies/import', { method: 'POST', body }),

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

    // data-source provider keys (Finnhub, Alpaca, …). Persisted in user_settings.
    // GET returns secrets masked as "***"; POST treats "***" / empty as "leave alone".
    dataSources: () => request('/data-sources'),
    /// Returns the unmasked secrets so the Settings UI can show them
    /// on demand when the user clicks the reveal toggle next to a
    /// password field. Same auth as `dataSources()`.
    dataSourcesReveal: () => request('/data-sources/reveal'),
    updateDataSources: (body) =>
        request('/data-sources', { method: 'POST', body: JSON.stringify(body) }),
    // Verify Alpaca creds without saving. Opens a WS, auths, returns
    // `{ok, feed, detail}` where `detail` is the raw Alpaca frame so
    // failures surface their own diagnostic.
    testAlpaca: (body) =>
        request('/data-sources/test-alpaca', { method: 'POST', body: JSON.stringify(body || {}) }),
    testFinnhub: (body) =>
        request('/data-sources/test-finnhub', { method: 'POST', body: JSON.stringify(body || {}) }),
    testTradier: (body) =>
        request('/data-sources/test-tradier', { method: 'POST', body: JSON.stringify(body || {}) }),
    testTastytrade: (body) =>
        request('/data-sources/test-tastytrade', { method: 'POST', body: JSON.stringify(body || {}) }),
    testIbkr: (body) =>
        request('/data-sources/test-ibkr', { method: 'POST', body: JSON.stringify(body || {}) }),
    testSchwab: (body) =>
        request('/data-sources/test-schwab', { method: 'POST', body: JSON.stringify(body || {}) }),

    // Live squeeze scanner — candidate aggregator + rolling-window detector.
    // /ws/squeeze emits {type:'snapshot',events:[…]} on connect, then
    // {type:'event',event:{…}} per fire. Reconnect on close.
    squeezeCandidates: () => request('/squeeze/candidates'),
    squeezeEvents: (limit = 50) => request(`/squeeze/events?limit=${limit}`),
    squeezeConfig: () => request('/squeeze/config'),
    updateSqueezeConfig: (body) =>
        request('/squeeze/config', { method: 'POST', body: JSON.stringify(body) }),

    // Direct Finnhub-backed routes (Finnhub-shaped responses, not Yahoo-shimmed).
    symbolProfile:              (sym) => request(`/symbols/${encodeURIComponent(sym)}/profile`),
    symbolPeers:                (sym) => request(`/symbols/${encodeURIComponent(sym)}/peers`),
    symbolUpgrades:             (sym) => request(`/symbols/${encodeURIComponent(sym)}/upgrades`),
    symbolFinancialsReported:   (sym) => request(`/symbols/${encodeURIComponent(sym)}/financials-reported`),
    symbolFinnhubQuote:         (sym) => request(`/symbols/${encodeURIComponent(sym)}/finnhub-quote`),
    symbolFinnhubNews:          (sym, days = 7) => request(`/symbols/${encodeURIComponent(sym)}/finnhub-news${qs({ days })}`),
    finnhubEarningsCalendar:    (from, to, sym) => request(`/finnhub/calendar/earnings${qs({ from, to, symbol: sym })}`),
    finnhubIpoCalendar:         (from, to) => request(`/finnhub/calendar/ipo${qs({ from, to })}`),
    finnhubGeneralNews:         (category = 'general') => request(`/finnhub/news${qs({ category })}`),

    // ── Comprehensive Finnhub coverage ──────────────────────────────
    // Per-symbol (free + premium routes — premium will 500 on free key).
    symbolProfileLegacy:        (sym) => request(`/symbols/${encodeURIComponent(sym)}/profile-legacy`),
    symbolExecutives:           (sym) => request(`/symbols/${encodeURIComponent(sym)}/executives`),
    symbolFinancials:           (sym, statement = 'ic', freq = 'annual') => request(`/symbols/${encodeURIComponent(sym)}/financials${qs({ statement, freq })}`),
    symbolMetric:               (sym) => request(`/symbols/${encodeURIComponent(sym)}/metric`),
    symbolNewsSentiment:        (sym) => request(`/symbols/${encodeURIComponent(sym)}/news-sentiment`),
    symbolPressReleases:        (sym, from, to) => request(`/symbols/${encodeURIComponent(sym)}/press-releases${qs({ from, to })}`),
    symbolEpsSurprise:          (sym) => request(`/symbols/${encodeURIComponent(sym)}/eps-surprise`),
    symbolRevenueEstimate:      (sym, freq = 'annual') => request(`/symbols/${encodeURIComponent(sym)}/revenue-estimate${qs({ freq })}`),
    symbolEbitdaEstimate:       (sym, freq = 'annual') => request(`/symbols/${encodeURIComponent(sym)}/ebitda-estimate${qs({ freq })}`),
    symbolEbitEstimate:         (sym, freq = 'annual') => request(`/symbols/${encodeURIComponent(sym)}/ebit-estimate${qs({ freq })}`),
    symbolEpsEstimate:          (sym, freq = 'annual') => request(`/symbols/${encodeURIComponent(sym)}/eps-estimate${qs({ freq })}`),
    symbolNetIncomeEstimate:    (sym, freq = 'annual') => request(`/symbols/${encodeURIComponent(sym)}/net-income-estimate${qs({ freq })}`),
    symbolPretaxIncomeEstimate: (sym, freq = 'annual') => request(`/symbols/${encodeURIComponent(sym)}/pretax-income-estimate${qs({ freq })}`),
    symbolGrossIncomeEstimate:  (sym, freq = 'annual') => request(`/symbols/${encodeURIComponent(sym)}/gross-income-estimate${qs({ freq })}`),
    symbolDpsEstimate:          (sym, freq = 'annual') => request(`/symbols/${encodeURIComponent(sym)}/dps-estimate${qs({ freq })}`),
    symbolPriceTarget:          (sym) => request(`/symbols/${encodeURIComponent(sym)}/price-target`),
    symbolOptionChain:          (sym) => request(`/symbols/${encodeURIComponent(sym)}/option-chain`),
    symbolFundOwnership:        (sym, limit = 20) => request(`/symbols/${encodeURIComponent(sym)}/fund-ownership${qs({ limit })}`),
    symbolOwnership:            (sym, limit = 20) => request(`/symbols/${encodeURIComponent(sym)}/ownership${qs({ limit })}`),
    symbolCompanyEarnings:      (sym, limit = 20) => request(`/symbols/${encodeURIComponent(sym)}/company-earnings${qs({ limit })}`),
    symbolFinnhubDividends:     (sym, from, to) => request(`/symbols/${encodeURIComponent(sym)}/finnhub-dividends${qs({ from, to })}`),
    symbolDividendsBasic:       (sym) => request(`/symbols/${encodeURIComponent(sym)}/dividends-basic`),
    symbolSplits:               (sym, from, to) => request(`/symbols/${encodeURIComponent(sym)}/splits${qs({ from, to })}`),
    symbolFinnhubCandles:       (sym, resolution = 'D', from, to) => request(`/symbols/${encodeURIComponent(sym)}/finnhub-candles${qs({ resolution, from, to })}`),
    symbolTick:                 (sym, date, limit = 500, skip = 0) => request(`/symbols/${encodeURIComponent(sym)}/tick${qs({ date, limit, skip })}`),
    symbolNbbo:                 (sym, date, limit = 500, skip = 0) => request(`/symbols/${encodeURIComponent(sym)}/nbbo${qs({ date, limit, skip })}`),
    symbolBidAsk:               (sym) => request(`/symbols/${encodeURIComponent(sym)}/bidask`),
    symbolFilings:              (sym, from, to, form) => request(`/symbols/${encodeURIComponent(sym)}/filings${qs({ from, to, form })}`),
    symbolTranscriptsList:      (sym) => request(`/symbols/${encodeURIComponent(sym)}/transcripts-list`),
    symbolSimilarityIndex:      (sym, freq = 'annual') => request(`/symbols/${encodeURIComponent(sym)}/similarity-index${qs({ freq })}`),
    symbolFinnhubInsiders:      (sym, from, to) => request(`/symbols/${encodeURIComponent(sym)}/finnhub-insiders${qs({ from, to })}`),
    symbolInsiderSentiment:     (sym, from, to) => request(`/symbols/${encodeURIComponent(sym)}/insider-sentiment${qs({ from, to })}`),
    symbolLobbying:             (sym, from, to) => request(`/symbols/${encodeURIComponent(sym)}/lobbying${qs({ from, to })}`),
    symbolUsaSpending:          (sym, from, to) => request(`/symbols/${encodeURIComponent(sym)}/usa-spending${qs({ from, to })}`),
    symbolVisaApplication:      (sym, from, to) => request(`/symbols/${encodeURIComponent(sym)}/visa-application${qs({ from, to })}`),
    symbolUsptoPatent:          (sym, from, to) => request(`/symbols/${encodeURIComponent(sym)}/uspto-patent${qs({ from, to })}`),
    symbolSupplyChain:          (sym) => request(`/symbols/${encodeURIComponent(sym)}/supply-chain`),
    symbolSocialSentiment:      (sym, from, to) => request(`/symbols/${encodeURIComponent(sym)}/social-sentiment${qs({ from, to })}`),
    symbolEsg:                  (sym) => request(`/symbols/${encodeURIComponent(sym)}/esg`),
    symbolEsgHistorical:        (sym) => request(`/symbols/${encodeURIComponent(sym)}/esg-historical`),
    symbolHistoricalMarketCap:  (sym, from, to) => request(`/symbols/${encodeURIComponent(sym)}/historical-market-cap${qs({ from, to })}`),
    symbolHistoricalEmployeeCount: (sym, from, to) => request(`/symbols/${encodeURIComponent(sym)}/historical-employee-count${qs({ from, to })}`),
    symbolEarningsQualityScore: (sym, freq = 'annual') => request(`/symbols/${encodeURIComponent(sym)}/earnings-quality-score${qs({ freq })}`),
    symbolRevenueBreakdown:     (sym) => request(`/symbols/${encodeURIComponent(sym)}/revenue-breakdown`),
    symbolRevenueBreakdown2:    (sym) => request(`/symbols/${encodeURIComponent(sym)}/revenue-breakdown2`),
    symbolPresentation:         (sym) => request(`/symbols/${encodeURIComponent(sym)}/presentation`),
    symbolNewsroom:             (sym, from, to) => request(`/symbols/${encodeURIComponent(sym)}/newsroom${qs({ from, to })}`),
    symbolCongressionalTrading: (sym, from, to) => request(`/symbols/${encodeURIComponent(sym)}/congressional-trading${qs({ from, to })}`),
    symbolPriceMetric:          (sym, date) => request(`/symbols/${encodeURIComponent(sym)}/price-metric${qs({ date })}`),
    symbolBankBranch:           (sym) => request(`/symbols/${encodeURIComponent(sym)}/bank-branch`),
    symbolScanPattern:          (sym, resolution = 'D') => request(`/symbols/${encodeURIComponent(sym)}/scan/pattern${qs({ resolution })}`),
    symbolScanSr:               (sym, resolution = 'D') => request(`/symbols/${encodeURIComponent(sym)}/scan/sr${qs({ resolution })}`),
    symbolScanAggregate:        (sym, resolution = 'D') => request(`/symbols/${encodeURIComponent(sym)}/scan/aggregate${qs({ resolution })}`),
    symbolIndicator:            (sym, resolution = 'D', from, to, indicator = 'sma') => request(`/symbols/${encodeURIComponent(sym)}/indicator${qs({ resolution, from, to, indicator })}`),

    // Calendars
    finnhubEconomicCalendar:    (from, to) => request(`/finnhub/calendar/economic${qs({ from, to })}`),
    finnhubFdaCalendar:         () => request(`/finnhub/calendar/fda`),
    finnhubEarningsCallLive:    (from, to, symbol) => request(`/finnhub/calendar/earnings-call-live${qs({ from, to, symbol })}`),

    // Forex
    finnhubForexExchanges:      () => request(`/finnhub/forex/exchanges`),
    finnhubForexSymbols:        (exchange = 'oanda') => request(`/finnhub/forex/symbols${qs({ exchange })}`),
    finnhubForexRates:          (base = 'USD') => request(`/finnhub/forex/rates${qs({ base })}`),
    finnhubForexCandle:         (symbol, resolution = 'D', from, to) => request(`/finnhub/forex/candle${qs({ symbol, resolution, from, to })}`),

    // Crypto
    finnhubCryptoExchanges:     () => request(`/finnhub/crypto/exchanges`),
    finnhubCryptoSymbols:       (exchange = 'binance') => request(`/finnhub/crypto/symbols${qs({ exchange })}`),
    finnhubCryptoCandle:        (symbol, resolution = 'D', from, to) => request(`/finnhub/crypto/candle${qs({ symbol, resolution, from, to })}`),
    finnhubCryptoProfile:       (symbol) => request(`/finnhub/crypto/profile${qs({ symbol })}`),

    // Indices / ETF / Mutual fund / Bond
    finnhubIndexConstituents:   (symbol) => request(`/finnhub/index/${encodeURIComponent(symbol)}/constituents`),
    finnhubIndexHistorical:     (symbol) => request(`/finnhub/index/${encodeURIComponent(symbol)}/historical-constituents`),
    finnhubEtfProfile:          (symbol) => request(`/finnhub/etf/${encodeURIComponent(symbol)}/profile`),
    finnhubEtfHoldings:         (symbol, skip = 0) => request(`/finnhub/etf/${encodeURIComponent(symbol)}/holdings${qs({ skip })}`),
    finnhubEtfSector:           (symbol) => request(`/finnhub/etf/${encodeURIComponent(symbol)}/sector`),
    finnhubEtfCountry:          (symbol) => request(`/finnhub/etf/${encodeURIComponent(symbol)}/country`),
    finnhubEtfAllocation:       (symbol) => request(`/finnhub/etf/${encodeURIComponent(symbol)}/allocation`),
    finnhubMfProfile:           (symbol) => request(`/finnhub/mutual-fund/${encodeURIComponent(symbol)}/profile`),
    finnhubMfHoldings:          (symbol, skip = 0) => request(`/finnhub/mutual-fund/${encodeURIComponent(symbol)}/holdings${qs({ skip })}`),
    finnhubMfSector:            (symbol) => request(`/finnhub/mutual-fund/${encodeURIComponent(symbol)}/sector`),
    finnhubMfCountry:           (symbol) => request(`/finnhub/mutual-fund/${encodeURIComponent(symbol)}/country`),
    finnhubMfEet:               (isin) => request(`/finnhub/mutual-fund/eet/${encodeURIComponent(isin)}`),
    finnhubBondProfile:         (isin) => request(`/finnhub/bond/${encodeURIComponent(isin)}/profile`),
    finnhubBondPrice:           (isin, from, to) => request(`/finnhub/bond/${encodeURIComponent(isin)}/price${qs({ from, to })}`),
    finnhubBondYieldCurve:      (code) => request(`/finnhub/bond/yield-curve${qs({ code })}`),

    // Economic / market / institutional
    finnhubEconomicCodes:       () => request(`/finnhub/economic/codes`),
    finnhubEconomicData:        (code) => request(`/finnhub/economic/data${qs({ code })}`),
    finnhubCountryList:         () => request(`/finnhub/country-list`),
    finnhubMarketStatus:        (exchange = 'US') => request(`/finnhub/market/status${qs({ exchange })}`),
    finnhubMarketHoliday:       (exchange = 'US') => request(`/finnhub/market/holiday${qs({ exchange })}`),
    finnhubStockExchanges:      () => request(`/finnhub/stock-exchanges`),
    finnhubSectorMetrics:       (region = 'NA') => request(`/finnhub/sector-metrics${qs({ category: region })}`),
    finnhubInstProfile:         (cik) => request(`/finnhub/institutional/${encodeURIComponent(cik)}/profile`),
    finnhubInstPortfolio:       (cik, from, to) => request(`/finnhub/institutional/${encodeURIComponent(cik)}/portfolio${qs({ from, to })}`),
    finnhubInstOwnership:       (symbol, from, to) => request(`/finnhub/institutional/${encodeURIComponent(symbol)}/ownership${qs({ from, to })}`),

    // Discovery / specialty
    finnhubSymbolLookup:        (q) => request(`/finnhub/search${qs({ q })}`),
    // Global symbol catalog — every listed ticker (US by default).
    // Used by the global autocomplete datalist; backend seeds from
    // Finnhub on first call when the table is empty.
    symbolsList: (seedIfEmpty = true) =>
        request(`/symbols/list${qs({ seed_if_empty: seedIfEmpty })}`),
    symbolsSeed: (exchange = 'US') =>
        request(`/symbols/seed${qs({ exchange })}`, { method: 'POST' }),
    finnhubStockSymbols:        (exchange = 'US') => request(`/finnhub/stock-symbols${qs({ exchange })}`),
    finnhubSymbolChange:        (from, to) => request(`/finnhub/symbol-change${qs({ from, to })}`),
    finnhubIsinChange:          (from, to) => request(`/finnhub/isin-change${qs({ from, to })}`),
    finnhubCovid19:             () => request(`/finnhub/covid19`),
    finnhubInvestmentTheme:     (theme) => request(`/finnhub/investment-theme${qs({ theme })}`),
    finnhubAirlinePriceIndex:   (airline, from, to) => request(`/finnhub/airline-price-index${qs({ airline, from, to })}`),
    listFilters: () => request('/filter-sets'),
    saveFilter: (name, payload, is_default = false) =>
        request('/filter-sets', { method: 'POST', body: JSON.stringify({ name, payload, is_default }) }),
    deleteFilter: (id) => request(`/filter-sets/${id}`, { method: 'DELETE' }),

    // Direct URL for CSV/HTML downloads. Browser navigates here for file
    // download — Authorization header isn't sent on <a download> by default,
    // so a query-param token is appended when present.
    exportTradesUrl: (account_id) => {
        const base = `${baseUrl}/api/export/trades/${account_id}.csv`;
        return token ? `${base}?token=${encodeURIComponent(token)}` : base;
    },
    exportExecutionsUrl: (account_id) => {
        const base = `${baseUrl}/api/export/executions/${account_id}.csv`;
        return token ? `${base}?token=${encodeURIComponent(token)}` : base;
    },

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
    indWma:           (sym, q) => request(`/bars/${encodeURIComponent(sym)}/wma${qs(q)}`),
    indHull:          (sym, q) => request(`/bars/${encodeURIComponent(sym)}/hull-ma${qs(q)}`),
    indRsi:           (sym, q) => request(`/bars/${encodeURIComponent(sym)}/rsi${qs(q)}`),
    indMacd:          (sym, q) => request(`/bars/${encodeURIComponent(sym)}/macd${qs(q)}`),
    indBollinger:     (sym, q) => request(`/bars/${encodeURIComponent(sym)}/bollinger${qs(q)}`),
    indAtr:           (sym, q) => request(`/bars/${encodeURIComponent(sym)}/atr${qs(q)}`),
    indRoc:           (sym, q) => request(`/bars/${encodeURIComponent(sym)}/roc${qs(q)}`),
    indTrix:          (sym, q) => request(`/bars/${encodeURIComponent(sym)}/trix${qs(q)}`),
    indDpo:           (sym, q) => request(`/bars/${encodeURIComponent(sym)}/dpo${qs(q)}`),
    indCoppock:       (sym, q) => request(`/bars/${encodeURIComponent(sym)}/coppock${qs(q)}`),
    indVixFix:        (sym, q) => request(`/bars/${encodeURIComponent(sym)}/vix-fix${qs(q)}`),
    indLaguerreRsi:   (sym, q) => request(`/bars/${encodeURIComponent(sym)}/laguerre-rsi${qs(q)}`),
    indCoppockRsi:    (sym, q) => request(`/bars/${encodeURIComponent(sym)}/coppock-rsi${qs(q)}`),
    indAnchoredObv:   (sym, q) => request(`/bars/${encodeURIComponent(sym)}/anchored-obv${qs(q)}`),
    indWolfeWave:     (sym, q) => request(`/bars/${encodeURIComponent(sym)}/wolfe-wave${qs(q)}`),
    indRsLine:        (sym, q) => request(`/bars/${encodeURIComponent(sym)}/rs-line${qs(q)}`),
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
    indAroonOscillator: (sym, q) => request(`/bars/${encodeURIComponent(sym)}/aroon-oscillator${qs(q)}`),
    indBbSqueezeMomentum: (sym, q) => request(`/bars/${encodeURIComponent(sym)}/squeeze-momentum${qs(q)}`),
    indCenterOfGravity: (sym, q) => request(`/bars/${encodeURIComponent(sym)}/center-of-gravity${qs(q)}`),
    indChaikinMoneyFlow: (sym, q) => request(`/bars/${encodeURIComponent(sym)}/chaikin-money-flow${qs(q)}`),
    indCmo:           (sym, q) => request(`/bars/${encodeURIComponent(sym)}/cmo${qs(q)}`),
    indConnorsRsi:    (sym, q) => request(`/bars/${encodeURIComponent(sym)}/connors-rsi${qs(q)}`),
    indDema:          (sym, q) => request(`/bars/${encodeURIComponent(sym)}/dema${qs(q)}`),
    indDemarker:      (sym, q) => request(`/bars/${encodeURIComponent(sym)}/demarker${qs(q)}`),
    indEaseOfMovement: (sym, q) => request(`/bars/${encodeURIComponent(sym)}/ease-of-movement${qs(q)}`),
    indEhlersDecycler: (sym, q) => request(`/bars/${encodeURIComponent(sym)}/ehlers-decycler${qs(q)}`),
    indElderForce:    (sym, q) => request(`/bars/${encodeURIComponent(sym)}/elder-force${qs(q)}`),
    indElderRay:      (sym, q) => request(`/bars/${encodeURIComponent(sym)}/elder-ray${qs(q)}`),
    indFisherTransform: (sym, q) => request(`/bars/${encodeURIComponent(sym)}/fisher-transform${qs(q)}`),
    indFractals:      (sym, q) => request(`/bars/${encodeURIComponent(sym)}/fractals${qs(q)}`),
    indFrama:         (sym, q) => request(`/bars/${encodeURIComponent(sym)}/frama${qs(q)}`),
    indKama:          (sym, q) => request(`/bars/${encodeURIComponent(sym)}/kama${qs(q)}`),
    indKlingerOscillator: (sym, q) => request(`/bars/${encodeURIComponent(sym)}/klinger-oscillator${qs(q)}`),
    indMcGinleyDynamic: (sym, q) => request(`/bars/${encodeURIComponent(sym)}/mcginley-dynamic${qs(q)}`),
    indNvi:           (sym, q) => request(`/bars/${encodeURIComponent(sym)}/nvi${qs(q)}`),
    indPpo:           (sym, q) => request(`/bars/${encodeURIComponent(sym)}/ppo${qs(q)}`),
    indPvi:           (sym, q) => request(`/bars/${encodeURIComponent(sym)}/pvi${qs(q)}`),
    indPvt:           (sym, q) => request(`/bars/${encodeURIComponent(sym)}/pvt${qs(q)}`),
    indQqe:           (sym, q) => request(`/bars/${encodeURIComponent(sym)}/qqe${qs(q)}`),
    indRelativeVolume: (sym, q) => request(`/bars/${encodeURIComponent(sym)}/relative-volume${qs(q)}`),
    indRvi:           (sym, q) => request(`/bars/${encodeURIComponent(sym)}/rvi${qs(q)}`),
    indStochRsi:      (sym, q) => request(`/bars/${encodeURIComponent(sym)}/stoch-rsi${qs(q)}`),
    indSuperSmoother: (sym, q) => request(`/bars/${encodeURIComponent(sym)}/super-smoother${qs(q)}`),
    indSwingIndex:    (sym, q) => request(`/bars/${encodeURIComponent(sym)}/swing-index${qs(q)}`),
    indTema:          (sym, q) => request(`/bars/${encodeURIComponent(sym)}/tema${qs(q)}`),
    indTsi:           (sym, q) => request(`/bars/${encodeURIComponent(sym)}/tsi${qs(q)}`),
    indUltimateOscillator: (sym, q) => request(`/bars/${encodeURIComponent(sym)}/ultimate-oscillator${qs(q)}`),
    indVhf:           (sym, q) => request(`/bars/${encodeURIComponent(sym)}/vhf${qs(q)}`),
    indVidya:         (sym, q) => request(`/bars/${encodeURIComponent(sym)}/vidya${qs(q)}`),
    indZigzag:        (sym, q) => request(`/bars/${encodeURIComponent(sym)}/zigzag${qs(q)}`),
    indZlema:         (sym, q) => request(`/bars/${encodeURIComponent(sym)}/zlema${qs(q)}`),
    // Composite / chart-overlay primitives — share the same `/bars/`
    // namespace but live in `charts.rs` rather than `chart_indicators.rs`.
    indIchimoku:      (sym, q) => request(`/bars/${encodeURIComponent(sym)}/ichimoku${qs(q)}`),
    indSupertrend:    (sym, q) => request(`/bars/${encodeURIComponent(sym)}/supertrend${qs(q)}`),

    // ============================================================
    // Options analytics (chain-derived)
    // ============================================================
    optionsMaxPain: (sym, q) => request(`/options/${encodeURIComponent(sym)}/max-pain${qs(q)}`),
    optionsGex:     (sym, q) => request(`/options/${encodeURIComponent(sym)}/gex${qs(q)}`),
    optionsIvSkew:  (sym, q) => request(`/options/${encodeURIComponent(sym)}/iv-skew${qs(q)}`),

    // Portfolio-level dealer GEX regime — see views/market_gamma_regime.js.
    marketGammaReport:  () => request('/market-gamma/report'),
    marketGammaRefresh: () => request('/market-gamma/refresh', { method: 'POST' }),

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
    microKylesLambda:        (b) => request('/microstructure/kyles-lambda',         { method: 'POST', body: JSON.stringify(b) }),
    microHawkesIntensity:    (b) => request('/microstructure/hawkes-intensity',     { method: 'POST', body: JSON.stringify(b) }),
    chartsKagi:              (b) => request('/charts/kagi',                         { method: 'POST', body: JSON.stringify(b) }),
    portfolioRiskParityWeights: (b) => request('/portfolio/risk-parity-weights',    { method: 'POST', body: JSON.stringify(b) }),
    chartsVolumeAtPrice:     (b) => request('/charts/volume-at-price',              { method: 'POST', body: JSON.stringify(b) }),
    portfolioHerfindahl:     (b) => request('/portfolio/herfindahl',                { method: 'POST', body: JSON.stringify(b) }),
    microRollSpread:         (b) => request('/microstructure/roll-spread',          { method: 'POST', body: JSON.stringify(b) }),
    chartsThreeLineBreak:    (b) => request('/charts/three-line-break',             { method: 'POST', body: JSON.stringify(b) }),
    portfolioMomentumCrashProtection: (b) => request('/portfolio/momentum-crash-protection', { method: 'POST', body: JSON.stringify(b) }),
    microEffectiveSpread:    (b) => request('/microstructure/effective-spread',     { method: 'POST', body: JSON.stringify(b) }),
    microWeightedMidprice:   (b) => request('/microstructure/weighted-midprice',    { method: 'POST', body: JSON.stringify(b) }),
    portfolioMarginalVar:    (b) => request('/portfolio/marginal-var',              { method: 'POST', body: JSON.stringify(b) }),
    chartsRangeBar:          (b) => request('/charts/range-bar',                    { method: 'POST', body: JSON.stringify(b) }),
    chartsTickBar:           (b) => request('/charts/tick-bar',                     { method: 'POST', body: JSON.stringify(b) }),
    chartsVolumeBar:         (b) => request('/charts/volume-bar',                   { method: 'POST', body: JSON.stringify(b) }),
    chartsDollarBar:         (b) => request('/charts/dollar-bar',                   { method: 'POST', body: JSON.stringify(b) }),
    portfolioActiveShare:    (b) => request('/portfolio/active-share',              { method: 'POST', body: JSON.stringify(b) }),
    portfolioBrinsonAttribution: (b) => request('/portfolio/brinson-attribution',   { method: 'POST', body: JSON.stringify(b) }),
    chartsEquivolumeBars:    (b) => request('/charts/equivolume-bars',              { method: 'POST', body: JSON.stringify(b) }),
    chartsImbalanceBar:      (b) => request('/charts/imbalance-bar',                { method: 'POST', body: JSON.stringify(b) }),
    portfolioBlackLitterman: (b) => request('/portfolio/black-litterman',           { method: 'POST', body: JSON.stringify(b) }),
    anlyAdfTest:             (b) => request('/analytics/adf-test',                  { method: 'POST', body: JSON.stringify(b) }),
    anlyAroonIndicator:      (b) => request('/analytics/aroon-indicator',           { method: 'POST', body: JSON.stringify(b) }),
    anlyAmihudIlliquidity:   (b) => request('/analytics/amihud-illiquidity',        { method: 'POST', body: JSON.stringify(b) }),
    anlyBreadthThrust:       (b) => request('/analytics/breadth-thrust',            { method: 'POST', body: JSON.stringify(b) }),
    anlyBollingerSqueeze:    (b) => request('/analytics/bollinger-squeeze',         { method: 'POST', body: JSON.stringify(b) }),
    anlyBalanceOfPower:      (b) => request('/analytics/balance-of-power',          { method: 'POST', body: JSON.stringify(b) }),
    anlyAnchoredMomentum:    (b) => request('/analytics/anchored-momentum',         { method: 'POST', body: JSON.stringify(b) }),
    anlyAutocorrelationFunction: (b) => request('/analytics/autocorrelation-function', { method: 'POST', body: JSON.stringify(b) }),
    anlyBrierScore:          (b) => request('/analytics/brier-score',               { method: 'POST', body: JSON.stringify(b) }),
    anlyBipowerVariation:    (b) => request('/analytics/bipower-variation',         { method: 'POST', body: JSON.stringify(b) }),
    anlyBootstrapPnl:        (b) => request('/analytics/bootstrap-pnl',             { method: 'POST', body: JSON.stringify(b) }),
    anlyBlockBootstrap:      (b) => request('/analytics/block-bootstrap',            { method: 'POST', body: JSON.stringify(b) }),
    anlyAdNormality:         (b) => request('/analytics/anderson-darling-normality', { method: 'POST', body: JSON.stringify(b) }),
    anlyArchLm:              (b) => request('/analytics/arch-lm-test',               { method: 'POST', body: JSON.stringify(b) }),
    anlyAlma:                (b) => request('/analytics/alma',                       { method: 'POST', body: JSON.stringify(b) }),
    anlyAlphatrend:          (b) => request('/analytics/alphatrend',                 { method: 'POST', body: JSON.stringify(b) }),
    anlyAtrChannel:          (b) => request('/analytics/atr-channel',                { method: 'POST', body: JSON.stringify(b) }),
    anlyAtrTrailingStop:     (b) => request('/analytics/atr-trailing-stop',          { method: 'POST', body: JSON.stringify(b) }),
    anlyAdl:                 (b) => request('/analytics/accumulation-distribution-line', { method: 'POST', body: JSON.stringify(b) }),
    anlyAccumulationSwingIndex: (b) => request('/analytics/accumulation-swing-index',    { method: 'POST', body: JSON.stringify(b) }),
    anlyAdOscillator:        (b) => request('/analytics/ad-oscillator',                  { method: 'POST', body: JSON.stringify(b) }),
    anlyBetaShrinkage:       (b) => request('/analytics/beta-shrinkage',                 { method: 'POST', body: JSON.stringify(b) }),
    anlyBartlettVariance:    (b) => request('/analytics/bartlett-variance-test',         { method: 'POST', body: JSON.stringify(b) }),
    anlyBidAskVolumeRatio:   (b) => request('/analytics/bid-ask-volume-ratio',           { method: 'POST', body: JSON.stringify(b) }),
    anlyBollingerBandWidth:  (b) => request('/analytics/bollinger-band-width',           { method: 'POST', body: JSON.stringify(b) }),
    anlyBollingerBandwidthPercentile: (b) => request('/analytics/bollinger-bandwidth-percentile', { method: 'POST', body: JSON.stringify(b) }),
    anlyBollingerPercentB:   (b) => request('/analytics/bollinger-percent-b',            { method: 'POST', body: JSON.stringify(b) }),
    anlyBollingerBandDistance: (b) => request('/analytics/bollinger-band-distance',      { method: 'POST', body: JSON.stringify(b) }),
    anlyBollingerOscillators: (b) => request('/analytics/bollinger-oscillators',         { method: 'POST', body: JSON.stringify(b) }),
    anlyBorrowRateIndicator: (b) => request('/analytics/borrow-rate-indicator',          { method: 'POST', body: JSON.stringify(b) }),
    anlyBreuschPagan:        (b) => request('/analytics/breusch-pagan-test',             { method: 'POST', body: JSON.stringify(b) }),
    anlyBurkeRatio:          (b) => request('/analytics/burke-ratio',                    { method: 'POST', body: JSON.stringify(b) }),
    anlyCamarillaPivots:     (b) => request('/analytics/camarilla-pivots',               { method: 'POST', body: JSON.stringify(b) }),
    anlyBreuschGodfrey:      (b) => request('/analytics/breusch-godfrey',                { method: 'POST', body: JSON.stringify(b) }),
    anlyCandleStrengthIndex: (b) => request('/analytics/candle-strength-index',          { method: 'POST', body: JSON.stringify(b) }),
    anlyCarhart4:            (b) => request('/analytics/carhart-4',                      { method: 'POST', body: JSON.stringify(b) }),
    anlyCenteredSmoothedMomentum: (b) => request('/analytics/centered-smoothed-momentum', { method: 'POST', body: JSON.stringify(b) }),
    anlyChaikinOscillator:   (b) => request('/analytics/chaikin-oscillator',             { method: 'POST', body: JSON.stringify(b) }),
    anlyChandeDynamicMomentum: (b) => request('/analytics/chande-dynamic-momentum',      { method: 'POST', body: JSON.stringify(b) }),
    anlyChandeKrollStop:     (b) => request('/analytics/chande-kroll-stop',              { method: 'POST', body: JSON.stringify(b) }),
    anlyChandeMomentumOscillator: (b) => request('/analytics/chande-momentum-oscillator', { method: 'POST', body: JSON.stringify(b) }),
    anlyChandeTrendIndex:    (b) => request('/analytics/chande-trend-index',             { method: 'POST', body: JSON.stringify(b) }),
    anlyChandeVolatilityIndex: (b) => request('/analytics/chande-volatility-index',      { method: 'POST', body: JSON.stringify(b) }),
    anlyChandelierExit:      (b) => request('/analytics/chandelier-exit',                { method: 'POST', body: JSON.stringify(b) }),
    anlyCholesky:            (b) => request('/analytics/cholesky',                       { method: 'POST', body: JSON.stringify(b) }),
    anlyAbcPattern:          (b) => request('/analytics/abc-pattern',                    { method: 'POST', body: JSON.stringify(b) }),
    anlyAbsorptionDetector:  (b) => request('/analytics/absorption-detector',            { method: 'POST', body: JSON.stringify(b) }),
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
