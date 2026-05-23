// API client. Resolves base URL + token differently for desktop vs web.

let baseUrl = '';
let token = '';

export async function initApi() {
    if (window.__TAURI__) {
        // Desktop — fetch config from Tauri backend.
        const cfg = await window.__TAURI__.core.invoke('get_api_config');
        baseUrl = cfg.base_url;
        token = cfg.token;
    } else {
        // Web — same origin, token from localStorage.
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

export class ApiError extends Error {
    constructor(status, msg) { super(msg); this.status = status; }
}

export const api = {
    config: () => request('/config'),
    me: () => request('/auth/me'),
    login: (email, password) =>
        request('/auth/login', { method: 'POST', body: JSON.stringify({ email, password }) }),
    register: (email, password, display_name) =>
        request('/auth/register', {
            method: 'POST',
            body: JSON.stringify({ email, password, display_name }),
        }),
    accounts: () => request('/accounts'),
    trades: (account_id, limit = 50, offset = 0) =>
        request(`/trades?account_id=${account_id}&limit=${limit}&offset=${offset}`),
    summary: (account_id) => request(`/stats/summary?account_id=${account_id}`),
    equity: (account_id) => request(`/stats/equity?account_id=${account_id}`),
    journalForDay: (day) => request(`/journal/${day}`),
};
