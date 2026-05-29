// Futures-roll schedule helpers.
//
// Backend body: { positions: FuturesPosition[], today: 'YYYY-MM-DD',
//   roll_window_days: i64 }.
// FuturesPosition = { symbol, contracts: i64, expiration: 'YYYY-MM-DD' }.
// Returns: { rows: RollRow[], now_count, expired_count }.
// RollRow  = { symbol, contracts, expiration, days_to_expiry, urgency }.
//
// Urgency rules (mirror crates/traderview-core/src/futures_roll.rs):
//   days < 0                         → expired
//   days ≤ roll_window_days          → now
//   days ≤ 2 × roll_window_days      → soon
//   else                             → comfortable
// Rows sorted by days_to_expiry ASC (most urgent first).

const TOKEN_DELIM = /[\s,]+/;
export const URGENCIES = ['now', 'soon', 'comfortable', 'expired'];

// "<symbol> <contracts> <YYYY-MM-DD>" per line. Contracts is integer
// (positive = long, negative = short).
export function parsePositionBlob(text) {
    const positions = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { positions, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = stripComment(raw).trim();
        if (!s) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 3) {
            errors.push({ line_no: i + 1, raw, message: `expected 3 tokens (symbol contracts YYYY-MM-DD), got ${parts.length}` });
            continue;
        }
        const symbol = parts[0];
        const contracts = Number(parts[1]);
        const expiration = parts[2];
        if (!Number.isInteger(contracts) || contracts === 0) {
            errors.push({ line_no: i + 1, raw, message: 'contracts must be non-zero integer (+ long / - short)' });
            continue;
        }
        if (!isValidDate(expiration)) {
            errors.push({ line_no: i + 1, raw, message: 'expiration must be YYYY-MM-DD' });
            continue;
        }
        positions.push({ symbol, contracts, expiration });
    }
    return { positions, errors };
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

// Whole-days b - a (matches chrono::Duration::num_days).
export function daysBetween(a, b) {
    if (!isValidDate(a) || !isValidDate(b)) return NaN;
    const da = Date.parse(a + 'T00:00:00Z');
    const db = Date.parse(b + 'T00:00:00Z');
    return Math.round((db - da) / 86_400_000);
}

export function validateInputs(positions, today, roll_window_days) {
    if (!Array.isArray(positions)) return 'positions must be an array';
    if (!isValidDate(today))       return 'today must be YYYY-MM-DD';
    if (!Number.isInteger(roll_window_days) || roll_window_days < 0)
        return 'roll_window_days must be non-negative integer';
    return null;
}

export function buildBody(positions, today, roll_window_days) {
    return {
        positions: positions.map(p => ({
            symbol: p.symbol, contracts: p.contracts, expiration: p.expiration,
        })),
        today,
        roll_window_days,
    };
}

// Mirror of futures_roll::schedule. Returns same shape; rows sorted ASC.
export function localSchedule(positions, today, roll_window_days) {
    const out = { rows: [], now_count: 0, expired_count: 0 };
    if (!Array.isArray(positions)) return out;
    for (const p of positions) {
        const days = daysBetween(today, p.expiration);
        const soonThreshold = roll_window_days * 2;
        let urgency;
        if (days < 0)                     urgency = 'expired';
        else if (days <= roll_window_days) urgency = 'now';
        else if (days <= soonThreshold)    urgency = 'soon';
        else                                urgency = 'comfortable';
        out.rows.push({
            symbol: p.symbol,
            contracts: p.contracts,
            expiration: p.expiration,
            days_to_expiry: days,
            urgency,
        });
        if (urgency === 'now')     out.now_count++;
        if (urgency === 'expired') out.expired_count++;
    }
    out.rows.sort((a, b) => a.days_to_expiry - b.days_to_expiry);
    return out;
}

const URGENCY_BADGES = {
    now:         { key: 'view.futures_roll.badge.now',         cls: 'neg' },
    soon:        { key: 'view.futures_roll.badge.soon',        cls: '' },
    comfortable: { key: 'view.futures_roll.badge.comfortable', cls: 'pos' },
    expired:     { key: 'view.futures_roll.badge.expired',     cls: 'neg' },
};

export function urgencyBadge(urgency) {
    return URGENCY_BADGES[urgency] || { key: 'view.futures_roll.badge.unknown', cls: '' };
}

// Top-of-page verdict based on counts.
export function overallBadge(report) {
    if (!report) return { key: 'view.futures_roll.verdict.unknown', cls: '' };
    if (report.expired_count > 0) return { key: 'view.futures_roll.verdict.emergency', cls: 'neg' };
    if (report.now_count > 0)     return { key: 'view.futures_roll.verdict.action',    cls: 'neg' };
    if (report.rows.length === 0) return { key: 'view.futures_roll.verdict.empty',     cls: '' };
    return { key: 'view.futures_roll.verdict.clean', cls: 'pos' };
}

// Build a YYYY-MM-DD relative to an anchor date by adding/subtracting days.
export function dateOffset(anchorIso, deltaDays) {
    if (!isValidDate(anchorIso)) return anchorIso;
    const d = new Date(anchorIso + 'T00:00:00Z');
    d.setUTCDate(d.getUTCDate() + deltaDays);
    const y = d.getUTCFullYear();
    const m = String(d.getUTCMonth() + 1).padStart(2, '0');
    const dd = String(d.getUTCDate()).padStart(2, '0');
    return `${y}-${m}-${dd}`;
}

export function todayIso() {
    return dateOffset(new Date().toISOString().slice(0, 10), 0);
}

// 5 demo presets (today-relative so they exercise the same urgency buckets
// regardless of when the user opens the view).
export function makeDemoPositions(kind, today) {
    const t = today || todayIso();
    switch (kind) {
        case 'mixed': {
            return [
                { symbol: '/ES', contracts:  1, expiration: dateOffset(t, 3) },  // now
                { symbol: '/NQ', contracts:  2, expiration: dateOffset(t, 10) }, // soon
                { symbol: '/CL', contracts: -1, expiration: dateOffset(t, 30) }, // comfortable
                { symbol: '/GC', contracts:  1, expiration: dateOffset(t, -5) }, // expired
            ];
        }
        case 'all-now':
            return Array.from({ length: 4 }, (_, i) => ({
                symbol: `/X${i + 1}`, contracts: 1, expiration: dateOffset(t, i + 1),
            }));
        case 'all-soon':
            return Array.from({ length: 4 }, (_, i) => ({
                symbol: `/Y${i + 1}`, contracts: 1, expiration: dateOffset(t, 10 + i),
            }));
        case 'comfortable':
            return Array.from({ length: 4 }, (_, i) => ({
                symbol: `/Z${i + 1}`, contracts: 1, expiration: dateOffset(t, 60 + i * 30),
            }));
        case 'emergency':
            // 2 expired + 1 now → emergency verdict.
            return [
                { symbol: '/ES', contracts:  1, expiration: dateOffset(t, -3) },
                { symbol: '/NQ', contracts: -1, expiration: dateOffset(t, -1) },
                { symbol: '/CL', contracts:  1, expiration: dateOffset(t, 2) },
            ];
        default:
            return makeDemoPositions('mixed', t);
    }
}

export function fmtDays(v) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v + 'd';
}

export function fmtContracts(v) {
    if (!Number.isFinite(v)) return '—';
    return (v > 0 ? '+' : '') + Math.trunc(v);
}

export function urgencyLabelKey(u) {
    return `view.futures_roll.urgency.${u || 'unknown'}`;
}
