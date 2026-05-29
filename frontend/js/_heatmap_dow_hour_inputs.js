// Day-of-week × hour-of-day P&L heatmap helpers.
//
// Backend body: { trades: Trade[] }. Trade is the full backend struct
// but the heatmap only uses {status, opened_at, net_pnl}. We synthesize
// minimal-but-deserialize-valid Trades from a simple input blob.
//
// Returns: { cells: HeatCell[7][24], total_trades, total_pnl }.
// HeatCell = { trades, wins, net_pnl }. dow: 0=Sun..6=Sat. hour: 0..23.

const TOKEN_DELIM = /[\s,]+/;

export const DOW_LABELS = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];

// "<YYYY-MM-DDThh:mm or 'YYYY-MM-DD hh'> <net_pnl>" — accept either:
//   2026-05-26T10:30 100
//   2026-05-26 10 100
//
// We split first on whitespace, then if the first token looks like a date
// we look at whether the next is hour+min combined (T form) or separate.
export function parseTradeBlob(text) {
    const rows = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { rows, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = stripComment(raw).trim();
        if (!s) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        // Iso form: "<YYYY-MM-DDThh:mm> <net_pnl>" → 2 tokens.
        // Spaced  : "<YYYY-MM-DD> <hour> <net_pnl>" → 3 tokens.
        let dateStr, hour, pnlStr;
        if (parts.length === 2 && parts[0].includes('T')) {
            dateStr = parts[0].slice(0, 10);
            const timePart = parts[0].slice(11);
            hour = parseInt(timePart.slice(0, 2), 10);
            pnlStr = parts[1];
        } else if (parts.length === 3) {
            dateStr = parts[0];
            hour = parseInt(parts[1], 10);
            pnlStr = parts[2];
        } else {
            errors.push({ line_no: i + 1, raw, message: `expected "<date>T<hh:mm> <pnl>" OR "<date> <hour> <pnl>", got ${parts.length} tokens` });
            continue;
        }
        if (!isValidDate(dateStr)) {
            errors.push({ line_no: i + 1, raw, message: 'date must be YYYY-MM-DD' });
            continue;
        }
        if (!Number.isInteger(hour) || hour < 0 || hour > 23) {
            errors.push({ line_no: i + 1, raw, message: 'hour must be integer 0..23' });
            continue;
        }
        const net_pnl = Number(pnlStr);
        if (!Number.isFinite(net_pnl)) {
            errors.push({ line_no: i + 1, raw, message: 'net_pnl must be finite' });
            continue;
        }
        rows.push({ date: dateStr, hour, net_pnl });
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

// 0=Sun..6=Sat (matches chrono::Weekday::num_days_from_sunday).
export function dowFromIsoDate(s) {
    if (!isValidDate(s)) return -1;
    const [y, m, d] = s.split('-').map(Number);
    const dt = new Date(Date.UTC(y, m - 1, d));
    return dt.getUTCDay();
}

export function validateInputs(rows) {
    if (!Array.isArray(rows)) return 'rows must be an array';
    return null;
}

// Build the backend body. Synthesizes a minimal Trade per row (same
// approach as setups_by_setup view — only fields stats_by_setup /
// dow_hour_heatmap actually read are populated meaningfully).
export function buildBody(rows) {
    return {
        trades: rows.map((r, i) => syntheticTrade(r.date, r.hour, r.net_pnl, i)),
    };
}

function syntheticTrade(date, hour, net_pnl, idx) {
    const hh = String(hour).padStart(2, '0');
    const opened = `${date}T${hh}:00:00Z`;
    const closed = `${date}T${hh}:30:00Z`;
    const id = makeDeterministicUuid(idx + 1);
    return {
        id,
        account_id: '00000000-0000-0000-0000-000000000000',
        symbol: 'X',
        side: 'long',
        status: 'closed',
        opened_at: opened,
        closed_at: closed,
        qty: '1',
        entry_avg: '100',
        exit_avg: '101',
        gross_pnl: String(net_pnl),
        fees: '0',
        net_pnl: String(net_pnl),
        asset_class: 'stock',
        option_type: null,
        strike: null,
        expiration: null,
        multiplier: '1',
        tick_size: null,
        tick_value: null,
        base_ccy: null,
        quote_ccy: null,
        pip_size: null,
        stop_loss: null,
        risk_amount: null,
        initial_target: null,
        mfe: null,
        mae: null,
        best_exit_pnl: null,
        exit_efficiency: null,
    };
}

export function makeDeterministicUuid(n) {
    const hex = String(n).padStart(32, '0');
    return `${hex.slice(0, 8)}-${hex.slice(8, 12)}-${hex.slice(12, 16)}-${hex.slice(16, 20)}-${hex.slice(20, 32)}`;
}

// Decimal-string-or-number safe coercion.
export function dec(v) {
    if (v == null) return 0;
    if (typeof v === 'number') return v;
    const n = Number(v);
    return Number.isFinite(n) ? n : 0;
}

// Empty 7×24 grid of zero cells.
export function emptyCells() {
    return Array.from({ length: 7 }, () =>
        Array.from({ length: 24 }, () => ({ trades: 0, wins: 0, net_pnl: 0 })));
}

// Pure-JS mirror of crates/traderview-core/src/dow_hour_heatmap.rs::build.
export function localBuild(rows) {
    const h = { cells: emptyCells(), total_trades: 0, total_pnl: 0 };
    for (const r of rows) {
        const dow = dowFromIsoDate(r.date);
        const hour = r.hour;
        if (dow < 0 || dow >= 7 || hour < 0 || hour >= 24) continue;
        const cell = h.cells[dow][hour];
        cell.trades++;
        cell.net_pnl += r.net_pnl;
        if (r.net_pnl > 0) cell.wins++;
        h.total_trades++;
        h.total_pnl += r.net_pnl;
    }
    return h;
}

export function winRate(cell) {
    return cell.trades > 0 ? cell.wins / cell.trades : 0;
}

// Per-cell heat class — 8 tiers (4 pos + 4 neg). Caller drives by net_pnl
// relative to maxAbs across the whole grid.
export function heatClass(net_pnl, maxAbs) {
    if (!Number.isFinite(net_pnl) || net_pnl === 0 || maxAbs <= 0) return 'heat-empty';
    const intensity = Math.min(1, Math.abs(net_pnl) / maxAbs);
    const tier = intensity >= 0.75 ? 4
               : intensity >= 0.50 ? 3
               : intensity >= 0.25 ? 2
               : 1;
    return net_pnl > 0 ? `heat-pos-${tier}` : `heat-neg-${tier}`;
}

// Largest absolute net_pnl across all 7×24 cells — used to color-scale.
export function maxCellAbs(report) {
    let m = 0;
    if (!report || !Array.isArray(report.cells)) return 0;
    for (const row of report.cells) for (const c of row) {
        const a = Math.abs(dec(c.net_pnl));
        if (a > m) m = a;
    }
    return m;
}

// Find best + worst cells across the whole grid (most positive / negative).
export function extremeCells(report) {
    let best = null, worst = null;
    if (!report || !Array.isArray(report.cells)) return { best, worst };
    for (let d = 0; d < 7; d++) for (let h = 0; h < 24; h++) {
        const cell = report.cells[d][h];
        if (!cell || cell.trades === 0) continue;
        const pnl = dec(cell.net_pnl);
        if (best == null || pnl > best.net_pnl) best  = { dow: d, hour: h, net_pnl: pnl, trades: cell.trades };
        if (worst == null || pnl < worst.net_pnl) worst = { dow: d, hour: h, net_pnl: pnl, trades: cell.trades };
    }
    return { best, worst };
}

// Demo presets.
export function makeDemoRows(kind = 'mixed') {
    switch (kind) {
        case 'mixed':
            return [
                { date: '2026-05-25', hour: 9,  net_pnl: 100 },  // Mon 9am
                { date: '2026-05-25', hour: 10, net_pnl: 250 },  // Mon 10am
                { date: '2026-05-26', hour: 9,  net_pnl: -150 }, // Tue 9am
                { date: '2026-05-26', hour: 14, net_pnl: 200 },  // Tue 2pm
                { date: '2026-05-27', hour: 10, net_pnl: 75 },   // Wed 10am
                { date: '2026-05-28', hour: 11, net_pnl: -300 }, // Thu 11am
                { date: '2026-05-29', hour: 15, net_pnl: 175 },  // Fri 3pm
            ];
        case 'monday-disaster':
            // Every trade Monday 9:30 is a loser — classic "don't trade Monday open" pattern.
            return Array.from({ length: 8 }, (_, i) => ({
                date: '2026-05-25', hour: 9, net_pnl: -50 - i * 25,
            }));
        case 'sweet-spot':
            // Tue/Wed 10am is the trader's edge.
            return Array.from({ length: 10 }, (_, i) => ({
                date: i % 2 === 0 ? '2026-05-26' : '2026-05-27', hour: 10, net_pnl: 100 + i * 20,
            }));
        case 'weekend-crypto':
            // Sat/Sun crypto trades — most asset views don't have this.
            return [
                { date: '2026-05-30', hour: 12, net_pnl: 250 },  // Sat
                { date: '2026-05-31', hour: 14, net_pnl: -100 }, // Sun
                { date: '2026-05-30', hour: 22, net_pnl: 500 },  // Sat night
            ];
        case 'all-week':
            // One trade per (dow,hour) intersection 9-16 for the whole week.
            const out = [];
            const dates = ['2026-05-25', '2026-05-26', '2026-05-27', '2026-05-28', '2026-05-29'];
            for (const d of dates) {
                for (let h = 9; h <= 16; h++) {
                    out.push({ date: d, hour: h, net_pnl: ((h + d.length) % 3 - 1) * 100 });
                }
            }
            return out;
        default:
            return makeDemoRows('mixed');
    }
}

export function fmtUSD(v, d = 0) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-' : '';
    return sign + '$' + Math.abs(v).toFixed(d);
}

export function fmtUSDSigned(v, d = 0) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '-') + '$' + Math.abs(v).toFixed(d);
}

export function fmtPct(v, d = 0) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtHour(h) {
    if (!Number.isInteger(h)) return '—';
    return String(h).padStart(2, '0') + ':00';
}
