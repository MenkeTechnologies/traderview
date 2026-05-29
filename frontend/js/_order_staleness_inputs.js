// Order Staleness helpers shared by view + vitest.
//
// Backend body shape: { orders: RestingOrder[], now: ISO-8601,
// thresholds: { warn_hours, stale_hours, forgotten_hours } }.
//
// Resting orders that sit too long become liabilities — a forgotten
// limit can fire when price comes back. This module classifies orders
// into 4 freshness tiers using the most-recent-touch as the clock.

import { t } from './i18n.js';

const TOKEN_DELIM = /[\s,]+/;
const ALLOWED_SIDES = new Set(['buy', 'sell', 'buy_stop', 'sell_stop']);

// Parses lines of the form:
//   "order_id symbol placed_at side"                   (4 tokens, no last_modified)
//   "order_id symbol placed_at last_modified_at side"  (5 tokens)
//
// Timestamps must be parseable by `Date`. `side` must be one of the
// 4 allowed strings.
export function parseOrderBlob(text) {
    const orders = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { orders, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.trim();
        if (!s || s.startsWith('#')) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 4 && parts.length !== 5) {
            errors.push({ line_no: i + 1, raw, message: `expected 4 or 5 tokens (id symbol placed_at [last_modified] side), got ${parts.length}` });
            continue;
        }
        const [order_id, symbol, placedTok, ...rest] = parts;
        const side = rest[rest.length - 1];
        const lastModifiedTok = rest.length === 2 ? rest[0] : null;
        if (!/^[A-Z0-9._-]+$/i.test(order_id)) {
            errors.push({ line_no: i + 1, raw, message: `bad order_id "${order_id}"` });
            continue;
        }
        if (!/^[A-Z0-9._-]+$/i.test(symbol)) {
            errors.push({ line_no: i + 1, raw, message: `bad symbol "${symbol}"` });
            continue;
        }
        const placed = toIso(placedTok);
        if (!placed) {
            errors.push({ line_no: i + 1, raw, message: `bad placed_at "${placedTok}"` });
            continue;
        }
        let lastModified = null;
        if (lastModifiedTok) {
            lastModified = toIso(lastModifiedTok);
            if (!lastModified) {
                errors.push({ line_no: i + 1, raw, message: `bad last_modified_at "${lastModifiedTok}"` });
                continue;
            }
        }
        if (!ALLOWED_SIDES.has(side)) {
            errors.push({ line_no: i + 1, raw, message: `side must be one of buy/sell/buy_stop/sell_stop, got "${side}"` });
            continue;
        }
        orders.push({
            order_id,
            symbol: symbol.toUpperCase(),
            placed_at: placed,
            last_modified_at: lastModified,
            side,
        });
    }
    return { orders, errors };
}

function toIso(tok) {
    const d = new Date(tok);
    if (Number.isNaN(d.getTime())) return null;
    return d.toISOString();
}

export function validateInputs(orders, nowIso, thresholds) {
    if (!Array.isArray(orders) || orders.length === 0) return 'need at least 1 order';
    if (typeof nowIso !== 'string' || !nowIso) return 'now must be an ISO 8601 string';
    if (Number.isNaN(new Date(nowIso).getTime())) return 'now is not a valid timestamp';
    if (!Number.isFinite(thresholds.warn_hours) || thresholds.warn_hours <= 0)
        return 'warn_hours must be > 0';
    if (!Number.isFinite(thresholds.stale_hours) || thresholds.stale_hours <= thresholds.warn_hours)
        return 'stale_hours must be > warn_hours';
    if (!Number.isFinite(thresholds.forgotten_hours) || thresholds.forgotten_hours <= thresholds.stale_hours)
        return 'forgotten_hours must be > stale_hours';
    return null;
}

export function buildBody(orders, nowIso, thresholds) {
    return { orders, now: nowIso, thresholds };
}

// Maps backend snake_case tier enum to badge label + color class.
const TIER_BADGES = {
    fresh:     { key: 'fresh',     cls: 'pos' },
    aging:     { key: 'aging',     cls: '' },
    stale:     { key: 'stale',     cls: 'neg' },
    forgotten: { key: 'forgotten', cls: 'neg' },
};
export function tierBadge(tier) {
    const x = TIER_BADGES[tier];
    if (!x) return { label: String(tier || '—'), cls: '' };
    return { label: t(`view.order_staleness.tier.${x.key}`), cls: x.cls };
}

// Pretty-prints hours into "X.Yh" / "X.Yd" depending on magnitude.
export function fmtHours(v) {
    if (!Number.isFinite(v)) return '—';
    if (v < 24)  return v.toFixed(1) + 'h';
    return (v / 24).toFixed(1) + 'd';
}

// Deterministic demo: 12 orders with engineered ages spanning all 4 tiers.
// Uses a fixed "now" reference and computes back-dated placed_at so the
// tier each order lands in is predictable.
export function makeDemoData(nowIso = '2024-06-15T15:00:00Z') {
    const now = new Date(nowIso);
    const at = (hoursAgo) => new Date(now.getTime() - hoursAgo * 3600_000).toISOString();
    const orders = [
        // Fresh (< 24h)
        { order_id: 'A1', symbol: 'AAPL', placed_at: at(0.5),  last_modified_at: null, side: 'buy' },
        { order_id: 'A2', symbol: 'MSFT', placed_at: at(2),    last_modified_at: null, side: 'sell' },
        { order_id: 'A3', symbol: 'SPY',  placed_at: at(10),   last_modified_at: null, side: 'buy_stop' },
        // Aging (24-72h)
        { order_id: 'B1', symbol: 'TSLA', placed_at: at(30),   last_modified_at: null, side: 'sell_stop' },
        { order_id: 'B2', symbol: 'NVDA', placed_at: at(50),   last_modified_at: null, side: 'sell' },
        { order_id: 'B3', symbol: 'AMD',  placed_at: at(100),  last_modified_at: at(48), side: 'buy' }, // touch resets clock
        // Stale (72-168h)
        { order_id: 'C1', symbol: 'META', placed_at: at(80),   last_modified_at: null, side: 'buy' },
        { order_id: 'C2', symbol: 'AMZN', placed_at: at(120),  last_modified_at: null, side: 'sell' },
        // Forgotten (> 168h)
        { order_id: 'D1', symbol: 'GOOG', placed_at: at(200),  last_modified_at: null, side: 'buy_stop' },
        { order_id: 'D2', symbol: 'NFLX', placed_at: at(400),  last_modified_at: null, side: 'sell' },
        { order_id: 'D3', symbol: 'IBM',  placed_at: at(720),  last_modified_at: null, side: 'buy' },
        { order_id: 'D4', symbol: 'KO',   placed_at: at(1500), last_modified_at: null, side: 'sell' },
    ];
    return { orders, now: nowIso };
}

// Current-time helper for the "now = now" button. Lives here for testability.
export function nowIso() {
    return new Date().toISOString();
}
