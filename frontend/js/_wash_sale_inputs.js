// Wash-sale (§1091) detector helpers shared by view + vitest.
//
// Backend body: { closings: ClosingTrade[], openings: OpeningExecution[] }.
//   ClosingTrade   = { trade_id: UUID, symbol, closed_at: YYYY-MM-DD,
//                      net_pnl: Decimal-as-string, qty: Decimal-as-string }
//   OpeningExecution = { execution_id: UUID, symbol, executed_at,
//                        qty: Decimal-as-string }
// Returns: { hits: WashHit[], total_disallowed: Decimal-as-string }.
//
// Bidirectional ±30-day window. Boundary 30 days exactly = INSIDE.

import { t } from './i18n.js';

const TOKEN_DELIM = /[\s,]+/;
const WASH_WINDOW_DAYS = 30;

// "<symbol> <YYYY-MM-DD> <net_pnl> <qty>" per line. net_pnl negative = loss.
export function parseClosingBlob(text) {
    return parseBlob(text, ['symbol', 'closed_at', 'net_pnl', 'qty'], (sym, date, pnl, qty) => {
        const id = makeDeterministicUuid(hashStr(`${sym}|${date}|${pnl}|${qty}`));
        return { trade_id: id, symbol: sym, closed_at: date,
                 net_pnl: pnl, qty };
    });
}

// "<symbol> <YYYY-MM-DD> <qty>" per line.
export function parseOpeningBlob(text) {
    return parseBlob(text, ['symbol', 'executed_at', 'qty'], (sym, date, qty) => {
        const id = makeDeterministicUuid(hashStr(`${sym}|${date}|${qty}`) + 0x10000);
        return { execution_id: id, symbol: sym, executed_at: date, qty };
    });
}

function parseBlob(text, schema, build) {
    const rows = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { rows, errors: [{ line_no: 0, raw: '', message: t('common.parse.input_must_be_string') }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = stripComment(raw).trim();
        if (!s) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== schema.length) {
            errors.push({ line_no: i + 1, raw, message: `expected ${schema.length} tokens (${schema.join(' ')}), got ${parts.length}` });
            continue;
        }
        const sym = parts[0].toUpperCase();
        const date = parts[1];
        if (!isValidDate(date)) {
            errors.push({ line_no: i + 1, raw, message: `${schema[1]} must be YYYY-MM-DD` });
            continue;
        }
        const numerics = parts.slice(2).map(Number);
        if (numerics.some(n => !Number.isFinite(n))) {
            errors.push({ line_no: i + 1, raw, message: t('view.wash_sale.parse.numeric_tokens_finite') });
            continue;
        }
        // For openings the last field is qty; for closings it's pnl + qty.
        if (schema[schema.length - 1] === 'qty' && numerics[numerics.length - 1] <= 0) {
            errors.push({ line_no: i + 1, raw, message: t('common.parse.qty_must_be_positive') });
            continue;
        }
        rows.push(build(sym, date, ...numerics));
    }
    return { rows, errors };
}

function stripComment(raw) {
    const i = raw.indexOf('#');
    return i >= 0 ? raw.slice(0, i) : raw;
}

export function isValidDate(s) {
    if (typeof s !== 'string' || !/^\d{4}-\d{2}-\d{2}$/.test(s)) return false;
    const [y, m, d] = s.split('-').map(Number);
    const dt = new Date(Date.UTC(y, m - 1, d));
    return dt.getUTCFullYear() === y && dt.getUTCMonth() === m - 1 && dt.getUTCDate() === d;
}

export function daysBetween(a, b) {
    if (!isValidDate(a) || !isValidDate(b)) return Infinity;
    const da = Date.parse(a + 'T00:00:00Z');
    const db = Date.parse(b + 'T00:00:00Z');
    return Math.round((db - da) / 86_400_000);
}

// Tiny hash to seed deterministic UUIDs from blob content (so id stability
// matches what the user typed).
export function hashStr(s) {
    let h = 0x811c9dc5;
    for (let i = 0; i < s.length; i++) {
        h ^= s.charCodeAt(i);
        h = (h + ((h << 1) + (h << 4) + (h << 7) + (h << 8) + (h << 24))) >>> 0;
    }
    return h >>> 0;
}

export function makeDeterministicUuid(n) {
    const hex = (BigInt(n) & 0xffffffffffffffffffffffffffffffffn).toString(16).padStart(32, '0');
    return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20, 32)}`;
}

export function validateInputs(closings, openings) {
    if (!Array.isArray(closings)) return t('view.wash_sale.validate.closings_array');
    if (!Array.isArray(openings)) return t('view.wash_sale.validate.openings_array');
    return null;
}

// Stringify Decimal fields per rust_decimal contract.
export function buildBody(closings, openings) {
    return {
        closings: closings.map(c => ({
            trade_id: c.trade_id, symbol: c.symbol, closed_at: c.closed_at,
            net_pnl: String(c.net_pnl), qty: String(c.qty),
        })),
        openings: openings.map(o => ({
            execution_id: o.execution_id, symbol: o.symbol,
            executed_at: o.executed_at, qty: String(o.qty),
        })),
    };
}

export function dec(v) {
    if (v == null) return 0;
    if (typeof v === 'number') return v;
    const n = Number(v);
    return Number.isFinite(n) ? n : 0;
}

// Pure-JS mirror of crates/traderview-core/src/wash_sale.rs::detect_hits.
// Emits one hit per (losing-close, replacement-buy) pair within ±30 days.
export function localDetectHits(closings, openings) {
    const hits = [];
    for (const c of closings) {
        if (c.net_pnl >= 0) continue;
        const loss = -c.net_pnl;
        for (const o of openings) {
            if (o.symbol !== c.symbol) continue;
            const days = daysBetween(c.closed_at, o.executed_at);
            if (Math.abs(days) > WASH_WINDOW_DAYS) continue;
            let qtyRatio = 0;
            if (c.qty !== 0) {
                qtyRatio = Math.min(o.qty, c.qty) / c.qty;
                if (qtyRatio < 0) qtyRatio = 0;
                if (qtyRatio > 1) qtyRatio = 1;
            }
            hits.push({
                losing_trade_id: c.trade_id,
                symbol: c.symbol,
                loss_amount: loss,
                days_offset: days,
                replacement_execution_id: o.execution_id,
                disallowed_loss_estimate: loss * qtyRatio,
            });
        }
    }
    return hits;
}

export function localTotalDisallowed(hits) {
    if (!Array.isArray(hits)) return 0;
    return hits.reduce((s, h) => s + dec(h.disallowed_loss_estimate), 0);
}

// Verdict badge by total disallowed loss.
export function washBadge(totalDisallowed, totalLoss) {
    if (!Number.isFinite(totalDisallowed)) return { key: 'view.wash_sale.badge.unknown', cls: '' };
    if (totalDisallowed === 0) return { key: 'view.wash_sale.badge.clean', cls: 'pos' };
    const pct = totalLoss > 0 ? totalDisallowed / totalLoss : 1;
    if (pct >= 0.75) return { key: 'view.wash_sale.badge.severe',   cls: 'neg' };
    if (pct >= 0.25) return { key: 'view.wash_sale.badge.material', cls: 'neg' };
    return { key: 'view.wash_sale.badge.minor', cls: '' };
}

// Sum loss across all losing closings (independent of hits).
export function totalRealizedLoss(closings) {
    return (closings || []).filter(c => c.net_pnl < 0).reduce((s, c) => s + (-c.net_pnl), 0);
}

// Demo presets exercising the Rust branches.
export function makeDemoClosings(kind = 'classic-trap') {
    switch (kind) {
        case 'classic-trap':
            // Sold AAPL at $500 loss; bought back 14 days later.
            return [{
                trade_id: makeDeterministicUuid(1),
                symbol: 'AAPL', closed_at: '2026-06-01',
                net_pnl: -500, qty: 100,
            }];
        case 'winning-trade-no-flag':
            return [{
                trade_id: makeDeterministicUuid(2),
                symbol: 'AAPL', closed_at: '2026-06-01',
                net_pnl: 500, qty: 100,
            }];
        case 'outside-window':
            return [{
                trade_id: makeDeterministicUuid(3),
                symbol: 'AAPL', closed_at: '2026-06-01',
                net_pnl: -500, qty: 100,
            }];
        case 'partial-replacement':
            return [{
                trade_id: makeDeterministicUuid(4),
                symbol: 'AAPL', closed_at: '2026-06-01',
                net_pnl: -500, qty: 100,
            }];
        case 'multi-hit':
            return [{
                trade_id: makeDeterministicUuid(5),
                symbol: 'AAPL', closed_at: '2026-06-01',
                net_pnl: -500, qty: 100,
            }];
        case 'mixed':
            return [
                { trade_id: makeDeterministicUuid(6),
                  symbol: 'AAPL', closed_at: '2026-06-01', net_pnl: -1000, qty: 100 },
                { trade_id: makeDeterministicUuid(7),
                  symbol: 'TSLA', closed_at: '2026-06-15', net_pnl: -500, qty: 50 },
                { trade_id: makeDeterministicUuid(8),
                  symbol: 'NVDA', closed_at: '2026-06-20', net_pnl: 800, qty: 25 },
            ];
        default:
            return makeDemoClosings('classic-trap');
    }
}

export function makeDemoOpenings(kind = 'classic-trap') {
    switch (kind) {
        case 'classic-trap':
            // Bought back 14 days later inside window.
            return [{
                execution_id: makeDeterministicUuid(0x100 + 1),
                symbol: 'AAPL', executed_at: '2026-06-15', qty: 100,
            }];
        case 'winning-trade-no-flag':
            return [{
                execution_id: makeDeterministicUuid(0x100 + 2),
                symbol: 'AAPL', executed_at: '2026-06-15', qty: 100,
            }];
        case 'outside-window':
            // 34 days after — outside the ±30-day window.
            return [{
                execution_id: makeDeterministicUuid(0x100 + 3),
                symbol: 'AAPL', executed_at: '2026-07-05', qty: 100,
            }];
        case 'partial-replacement':
            // Only bought back 30 of 100 sold → 30% disallowed.
            return [{
                execution_id: makeDeterministicUuid(0x100 + 4),
                symbol: 'AAPL', executed_at: '2026-06-05', qty: 30,
            }];
        case 'multi-hit':
            // TWO replacement buys, both inside window.
            return [
                { execution_id: makeDeterministicUuid(0x100 + 5),
                  symbol: 'AAPL', executed_at: '2026-06-05', qty: 100 },
                { execution_id: makeDeterministicUuid(0x100 + 6),
                  symbol: 'AAPL', executed_at: '2026-06-20', qty: 100 },
            ];
        case 'mixed':
            return [
                { execution_id: makeDeterministicUuid(0x100 + 7),
                  symbol: 'AAPL', executed_at: '2026-06-10', qty: 100 },
                { execution_id: makeDeterministicUuid(0x100 + 8),
                  symbol: 'TSLA', executed_at: '2026-07-20', qty: 50 }, // outside window
            ];
        default:
            return makeDemoOpenings('classic-trap');
    }
}

export function fmtUSD(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-' : '';
    return sign + '$' + Math.abs(v).toFixed(d);
}

export function fmtUSDSigned(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '-') + '$' + Math.abs(v).toFixed(d);
}

export function fmtDays(v) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v + 'd';
}

export function fmtPct(v, d = 1) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtNum(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function shortUuid(u) {
    if (typeof u !== 'string') return '—';
    return u.slice(0, 8);
}
