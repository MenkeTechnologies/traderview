// News Event Handler helpers shared by view + vitest.
//
// Backend body shape: { positions: [{symbol, current_qty}, ...],
// events: [{event_name, impact, affected_symbols}, ...] }.
// Returns { actions: [{symbol, current_qty, recommended_qty,
// trim_amount, reason}] }. Trim % by impact: Low=0, Medium=25,
// High=50, Critical=100.

const TOKEN_DELIM = /[\s,]+/;

// Two-token-per-line `symbol qty` for positions.
export function parsePositions(text) {
    const positions = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { positions, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.trim();
        if (!s || s.startsWith('#')) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 2) {
            errors.push({ line_no: i + 1, raw, message: `expected 2 tokens (symbol qty), got ${parts.length}` });
            continue;
        }
        const sym = parts[0].toUpperCase();
        const qty = Number(parts[1]);
        if (!/^[A-Z0-9._-]+$/.test(sym)) {
            errors.push({ line_no: i + 1, raw, message: `bad symbol "${parts[0]}"` });
            continue;
        }
        if (!Number.isFinite(qty) || qty <= 0) {
            errors.push({ line_no: i + 1, raw, message: `qty must be > 0` });
            continue;
        }
        positions.push({ symbol: sym, current_qty: qty });
    }
    return { positions, errors };
}

const VALID_IMPACTS = new Set(['low', 'medium', 'high', 'critical']);

// Per-line event format:
//   "event_name impact"                    — market-wide event
//   "event_name impact AAPL,TSLA,SPY"      — symbol-specific event
// Event name CAN contain spaces — we split on the LAST whitespace before
// the impact token so e.g. "Retail sales medium" parses as ("Retail sales", medium, []).
export function parseEvents(text) {
    const events = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { events, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.trim();
        if (!s || s.startsWith('#')) continue;
        // Find the impact token (one of low/medium/high/critical) — name
        // is everything before, affected-symbols are everything after.
        const tokens = s.split(/\s+/);
        let impactIdx = -1;
        let impact = null;
        for (let j = 0; j < tokens.length; j++) {
            const lower = tokens[j].toLowerCase();
            if (VALID_IMPACTS.has(lower)) {
                impactIdx = j;
                impact = lower;
                break;
            }
        }
        if (impactIdx < 0 || impactIdx === 0) {
            errors.push({ line_no: i + 1, raw, message: `expected "event_name <low|medium|high|critical> [comma,sep,symbols]"` });
            continue;
        }
        const event_name = tokens.slice(0, impactIdx).join(' ').trim();
        if (!event_name) {
            errors.push({ line_no: i + 1, raw, message: `event_name cannot be empty` });
            continue;
        }
        // Affected symbols: everything after the impact token, joined and
        // re-split on comma OR whitespace so both "AAPL,TSLA" and "AAPL TSLA" work.
        const rest = tokens.slice(impactIdx + 1).join(' ').trim();
        const affected_symbols = rest
            ? rest.split(TOKEN_DELIM).filter(Boolean).map(s => s.toUpperCase())
            : [];
        if (affected_symbols.length > 0 && !affected_symbols.every(s => /^[A-Z0-9._-]+$/.test(s))) {
            errors.push({ line_no: i + 1, raw, message: `bad symbol in affected list` });
            continue;
        }
        events.push({ event_name, impact, affected_symbols });
    }
    return { events, errors };
}

export function validateInputs(positions, events) {
    if (!Array.isArray(positions) || positions.length === 0) return 'need at least 1 position';
    if (!Array.isArray(events)) return 'events must be an array';
    return null;
}

export function buildBody(positions, events) {
    return { positions, events };
}

// Trim percentage by impact — mirrors backend exactly. Returned as a
// fraction (0.0 to 1.0).
export function trimFractionFor(impact) {
    switch (impact) {
        case 'low':      return 0.0;
        case 'medium':   return 0.25;
        case 'high':     return 0.50;
        case 'critical': return 1.0;
        default:         return 0.0;
    }
}

// Impact-to-UI badge.
const IMPACT_BADGES = {
    low:      { label: 'LOW',      cls: 'pos' },
    medium:   { label: 'MEDIUM',   cls: '' },
    high:     { label: 'HIGH',     cls: 'neg' },
    critical: { label: 'CRITICAL', cls: 'neg' },
};
export function impactBadge(i) { return IMPACT_BADGES[i] || { label: String(i || '—'), cls: '' }; }

// Aggregate counts for the summary panel.
export function summarize(report, positions) {
    const actions = (report && report.actions) || [];
    let totalTrim = 0;
    let critical = 0;
    for (const a of actions) {
        if (Number.isFinite(a.trim_amount)) totalTrim += Math.abs(a.trim_amount);
        if (/critical/i.test(a.reason || '')) critical++;
    }
    return {
        positionCount: positions.length,
        actionCount:   actions.length,
        unchanged:     positions.length - actions.length,
        totalTrim,
        critical,
    };
}

// 4-position + 3-event deterministic demo spanning every impact tier:
//   AAPL → critical (FOMC market-wide) → full close
//   TSLA → high (CPI market-wide)      → 50% trim
//   MSFT → medium (Retail sales)        → 25% trim
//   SPY  → critical (FOMC) wins over medium (Retail sales)
//   ILQD → low (Fed minutes)            → no action
export function makeDemoData() {
    return {
        positions: [
            { symbol: 'AAPL', current_qty: 100 },
            { symbol: 'TSLA', current_qty: 50 },
            { symbol: 'MSFT', current_qty: 200 },
            { symbol: 'SPY',  current_qty: 1000 },
            { symbol: 'ILQD', current_qty: 500 },
        ],
        events: [
            { event_name: 'FOMC',         impact: 'critical', affected_symbols: [] },
            { event_name: 'CPI',          impact: 'high',     affected_symbols: ['TSLA'] },
            { event_name: 'Retail sales', impact: 'medium',   affected_symbols: ['MSFT'] },
            { event_name: 'Fed minutes',  impact: 'low',      affected_symbols: ['ILQD'] },
        ],
    };
}

export function fmtN(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtInt(v) {
    if (!Number.isFinite(v)) return '—';
    return Math.round(v).toLocaleString('en-US');
}

export function fmtPct(v) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(0) + '%';
}
