// DeMark Pivots helpers shared by view + vitest.
//
// Backend body shape: { session: { open, high, low, close } }. Returns
// { r1, pivot, s1 } — DeMark's tight three-level system. Distinct from
// floor/Camarilla/Woodie/Fibonacci pivots in that the X-base formula
// depends on close-vs-open direction, NOT (H+L+C)/3.

export function validateInputs(p) {
    for (const k of ['open', 'high', 'low', 'close']) {
        if (!Number.isFinite(p[k]) || p[k] <= 0) return `${k} must be > 0`;
    }
    if (p.high < p.low) return 'high must be ≥ low';
    if (p.open < p.low - 1e-9 || p.open > p.high + 1e-9) return 'open must be in [low, high]';
    if (p.close < p.low - 1e-9 || p.close > p.high + 1e-9) return 'close must be in [low, high]';
    return null;
}

export function buildBody(p) {
    return {
        session: {
            open: p.open, high: p.high, low: p.low, close: p.close,
        },
    };
}

// Mirrors backend's X-base direction logic exactly. Returned label
// + multiplier explains which formula was used so the trader sees the
// math, not just the output.
export function xBaseInfo(p) {
    if (p.close < p.open) {
        return {
            label: 'BEARISH X — low-heavy',
            cls: 'neg',
            formula: 'X = H + 2·L + C',
            hint: 'prior session sold off; X weighted toward low → pivot biases down',
        };
    }
    if (p.close > p.open) {
        return {
            label: 'BULLISH X — high-heavy',
            cls: 'pos',
            formula: 'X = 2·H + L + C',
            hint: 'prior session rallied; X weighted toward high → pivot biases up',
        };
    }
    return {
        label: 'NEUTRAL X — close-heavy',
        cls: '',
        formula: 'X = H + L + 2·C',
        hint: 'doji close = open; X weighted toward close → pivot at center',
    };
}

// Compute X locally (parity-check vs backend output). Useful both for
// sanity-validation and for the "shown math" panel.
export function computeX(p) {
    if (!Number.isFinite(p.open) || !Number.isFinite(p.high) ||
        !Number.isFinite(p.low) || !Number.isFinite(p.close)) return NaN;
    if (p.close < p.open) return p.high + 2 * p.low + p.close;
    if (p.close > p.open) return 2 * p.high + p.low + p.close;
    return p.high + p.low + 2 * p.close;
}

// Classifies the relationship between today's intended trade price and
// the pivot system. Used for the "trade bias" hint.
export function tradeBias(spotNow, levels) {
    if (!Number.isFinite(spotNow) || !levels) return { label: '—', cls: '', hint: '' };
    if (spotNow >= levels.r1) return { label: 'ABOVE R1', cls: 'neg',
        hint: 'breakout above DeMark resistance — momentum long or wait for retest' };
    if (spotNow <= levels.s1) return { label: 'BELOW S1', cls: 'pos',
        hint: 'breakdown below DeMark support — momentum short or wait for retest' };
    if (spotNow >= levels.pivot) return { label: 'PIVOT → R1', cls: '',
        hint: 'upper band — long bias targeting R1' };
    return { label: 'S1 → PIVOT', cls: '',
        hint: 'lower band — short bias targeting S1' };
}

// 4 deterministic demo presets matching the X-base formula branches +
// an "inside day" preset that's close-near-mid (neutral with tighter range).
export function makeDemoSession(kind) {
    switch (kind) {
        case 'bearish':  return { open: 108, high: 110, low: 100, close: 102 };
        case 'bullish':  return { open: 102, high: 110, low: 100, close: 108 };
        case 'doji':     return { open: 105, high: 110, low: 100, close: 105 };
        case 'inside':
        default:         return { open: 103, high: 106, low: 102, close: 104 };
    }
}

// Computes a sensible chart y-range that fits all of the pivots + the
// session HLC with a 5% pad.
export function chartSpan(session, levels) {
    const vals = [session.open, session.high, session.low, session.close];
    if (levels) vals.push(levels.r1, levels.pivot, levels.s1);
    const finite = vals.filter(Number.isFinite);
    if (!finite.length) return { min: 0, max: 1 };
    const min = Math.min(...finite);
    const max = Math.max(...finite);
    const pad = (max - min) * 0.05 || 1;
    return { min: min - pad, max: max + pad };
}

export function fmtN(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}
