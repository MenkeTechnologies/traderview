// Pure helpers for the Dividend Yield Calendar view.
//
// Parse a textarea-pasted symbol list, then extract upcoming dividend
// info from the Yahoo `quoteSummary` blob the backend returns from
// `/symbols/:symbol/dividends`. Yahoo wraps every field as
// `{ raw: number|null, fmt: string|null }` — `extractDividend` reaches
// past that shape into UI-friendly scalars.

const TOKEN_DELIM = /[\s,]+/;

/** Parse a list of symbols (one per line/comma/space). Uppercases each
 *  and de-duplicates while preserving first-seen order. */
export function parseSymbolList(text) {
    if (typeof text !== 'string') return [];
    const seen = new Set();
    const out = [];
    for (const line of text.split(/\r?\n/)) {
        const stripped = line.trim();
        if (!stripped || stripped.startsWith('#')) continue;
        for (const tok of stripped.split(TOKEN_DELIM)) {
            const s = tok.trim().toUpperCase();
            if (!s) continue;
            // Symbols are letters + digits + `.-` (BRK.B, RDS-A).
            if (!/^[A-Z0-9.\-]+$/.test(s)) continue;
            if (seen.has(s)) continue;
            seen.add(s);
            out.push(s);
        }
    }
    return out;
}

/** Pull a `.raw` numeric value from a Yahoo `{ raw, fmt }` field, or
 *  return null if the field is missing / null / non-numeric. */
function rawNum(field) {
    if (field == null || typeof field !== 'object') return null;
    const v = field.raw;
    return Number.isFinite(v) ? v : null;
}

/** Yahoo timestamps come as Unix seconds. Convert to JS Date or null. */
function unixToDate(ts) {
    if (!Number.isFinite(ts)) return null;
    return new Date(ts * 1000);
}

/** Extract a clean dividend-event record from one symbol's
 *  /symbols/:symbol/dividends response. Returns null if no actionable
 *  dividend data (non-payers, ETFs without dividends, etc.). */
export function extractDividend(symbol, payload) {
    if (!payload || typeof payload !== 'object') return null;
    // The endpoint pipes Yahoo quoteSummary; the modules of interest are
    // summaryDetail + calendarEvents. Either may be missing for non-
    // payers, so guard each access.
    const sd = payload.summaryDetail || {};
    const ce = payload.calendarEvents || {};
    const yieldDecimal = rawNum(sd.dividendYield);
    const amount       = rawNum(sd.dividendRate);
    const exTs         = rawNum(ce.exDividendDate) ?? rawNum(sd.exDividendDate);
    const payTs        = rawNum(ce.dividendDate);
    const lastDivAmt   = rawNum(sd.lastDividendValue);
    const lastDivTs    = rawNum(sd.lastDividendDate);
    const payoutRatio  = rawNum(sd.payoutRatio);
    // Need at least an ex-date OR an amount to surface a row.
    if (exTs == null && amount == null && yieldDecimal == null) return null;
    return {
        symbol,
        ex_date: unixToDate(exTs),
        pay_date: unixToDate(payTs),
        amount,
        yield: yieldDecimal,
        payout_ratio: payoutRatio,
        last_div_amount: lastDivAmt,
        last_div_date: unixToDate(lastDivTs),
    };
}

/** Days between `from` and `to` (both Date), or null if either missing.
 *  Rounded to nearest whole day. */
export function daysBetween(from, to) {
    if (!(from instanceof Date) || !(to instanceof Date)) return null;
    const MS_PER_DAY = 1000 * 60 * 60 * 24;
    return Math.round((to.getTime() - from.getTime()) / MS_PER_DAY);
}

/** Sort by ex-date ascending. Rows with no ex-date sort to the end. */
export function sortByExDate(rows) {
    return [...rows].sort((a, b) => {
        const at = a.ex_date ? a.ex_date.getTime() : Infinity;
        const bt = b.ex_date ? b.ex_date.getTime() : Infinity;
        return at - bt;
    });
}

/** Keep only rows whose ex-date falls in [today, today + horizonDays].
 *  Past-dated ex-dates are dropped (already happened). horizonDays = 0
 *  returns only today's ex-dividends; horizonDays = Infinity keeps all
 *  future rows. */
export function filterByHorizon(rows, today, horizonDays) {
    if (!(today instanceof Date)) return rows;
    const todayMidnight = new Date(today.getFullYear(), today.getMonth(), today.getDate());
    const startMs = todayMidnight.getTime();
    const endMs = Number.isFinite(horizonDays)
        ? startMs + horizonDays * 1000 * 60 * 60 * 24 + 1
        : Infinity;
    return rows.filter(r => {
        if (!r.ex_date) return false;
        const t = r.ex_date.getTime();
        return t >= startMs && t <= endMs;
    });
}

/** Format a Date as `YYYY-MM-DD` in local time. Returns "—" for null. */
export function fmtDate(d) {
    if (!(d instanceof Date) || isNaN(d.getTime())) return '—';
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${y}-${m}-${day}`;
}

/** Format a decimal yield (0.025) as "2.50%". Null → "—". */
export function fmtYield(y) {
    if (!Number.isFinite(y)) return '—';
    return `${(y * 100).toFixed(2)}%`;
}

/** Format a dividend amount with $ prefix and 4 decimals (penny stocks
 *  pay 0.0050 dividends). Null → "—". */
export function fmtAmount(a) {
    if (!Number.isFinite(a)) return '—';
    return `$${a.toFixed(4)}`;
}
