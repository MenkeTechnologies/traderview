// Three-Bar Reversal helpers shared by view + vitest.
//
// Backend body shape: { bars: OhlcBar[] }. No config — detection rule
// is fixed: bar1 trend, small middle, bar3 closes past bar1's extreme.

const TOKEN_DELIM = /[\s,]+/;

// Four-token-per-line "open high low close" with per-bar OHLC sanity
// (positivity, low ≤ high, open/close in [low, high]).
export function parseBarBlob(text) {
    const bars = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { bars, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.trim();
        if (!s || s.startsWith('#')) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 4) {
            errors.push({ line_no: i + 1, raw, message: `expected 4 tokens (open high low close), got ${parts.length}` });
            continue;
        }
        const o = Number(parts[0]);
        const h = Number(parts[1]);
        const l = Number(parts[2]);
        const c = Number(parts[3]);
        if (![o, h, l, c].every(Number.isFinite) || o <= 0 || h <= 0 || l <= 0 || c <= 0) {
            errors.push({ line_no: i + 1, raw, message: `OHLC must be positive finite` });
            continue;
        }
        if (l > h + 1e-9) {
            errors.push({ line_no: i + 1, raw, message: `low > high` });
            continue;
        }
        if (o < l - 1e-9 || o > h + 1e-9 || c < l - 1e-9 || c > h + 1e-9) {
            errors.push({ line_no: i + 1, raw, message: `open / close outside [low, high]` });
            continue;
        }
        bars.push({ open: o, high: h, low: l, close: c });
    }
    return { bars, errors };
}

export function validateInputs(bars) {
    if (!Array.isArray(bars) || bars.length < 3) return 'need at least 3 bars (pattern requires bar i-2, i-1, i)';
    return null;
}

export function buildBody(bars) {
    return { bars };
}

const KIND_BADGES = {
    bullish: { label: 'BULLISH 3-BAR', cls: 'pos', hint: 'down → small → up; closes above bar 1 high' },
    bearish: { label: 'BEARISH 3-BAR', cls: 'neg', hint: 'up → small → down; closes below bar 1 low' },
};
export function kindBadge(k) { return KIND_BADGES[k] || { label: String(k || '—'), cls: '', hint: '' }; }

// Splits events into parallel up/down null-padded series for uPlot marker
// rendering. Bullish markers go below the bar; bearish above.
export function eventMarkers(events, bars) {
    const up = new Array(bars.length).fill(null);
    const dn = new Array(bars.length).fill(null);
    if (!Array.isArray(events)) return { up, dn };
    for (const e of events) {
        if (!Number.isInteger(e.bar_index) || e.bar_index < 0 || e.bar_index >= bars.length) continue;
        const bar = bars[e.bar_index];
        if (!bar) continue;
        if (e.kind === 'bullish') up[e.bar_index] = bar.low * 0.998;
        if (e.kind === 'bearish') dn[e.bar_index] = bar.high * 1.002;
    }
    return { up, dn };
}

// Deterministic 14-bar demo containing one classic bullish 3-bar at
// indices 2-3-4 (down → inside → up closing above the down-bar's high)
// and one classic bearish 3-bar at indices 11-12-13.
export function makeDemoBars() {
    return [
        { open: 100,  high: 100.5, low: 99.5, close: 100.2 },   // 0 — preamble
        { open: 100.2, high: 100.5, low: 99.8, close: 100.0 },  // 1 — preamble
        { open: 100.0, high: 100.2, low: 96.5, close: 96.8 },   // 2 — strong DOWN bar1 (close < open)
        { open: 96.8,  high: 97.5, low: 96.5, close: 97.0 },    // 3 — small inside middle
        { open: 97.0,  high: 101.0, low: 96.9, close: 100.8 },  // 4 — UP bar3, close > bar1.high (100.2)
        { open: 100.8, high: 102.0, low: 100.5, close: 101.5 }, // 5 — rally continuation
        { open: 101.5, high: 102.5, low: 101.0, close: 102.0 }, // 6
        { open: 102.0, high: 103.0, low: 101.5, close: 102.5 }, // 7
        { open: 102.5, high: 103.5, low: 102.0, close: 103.0 }, // 8
        { open: 103.0, high: 104.0, low: 102.5, close: 103.5 }, // 9
        { open: 103.5, high: 104.5, low: 103.0, close: 104.0 }, // 10
        { open: 104.0, high: 107.5, low: 103.8, close: 107.0 }, // 11 — strong UP bar1 (close > open)
        { open: 107.0, high: 107.3, low: 106.5, close: 106.8 }, // 12 — small inside middle
        { open: 106.8, high: 107.0, low: 103.0, close: 103.5 }, // 13 — DOWN bar3, close < bar1.low (103.8)
    ];
}

export function fmtN(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}
