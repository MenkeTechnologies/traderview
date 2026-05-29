// Murrey Math Levels (T. Henning Murrey 1995) helpers shared by view +
// vitest.
//
// Backend body shape: { bars: [{high, low, close}, ...], lookback_bars
// }. Returns { levels: [(label, value), ...], current_price, nearest_level,
// distance_to_nearest_pct } — or null when invalid.

import { t } from './i18n.js';

const TOKEN_DELIM = /[\s,]+/;

// Three-token-per-line "high low close".
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
        if (parts.length !== 3) {
            errors.push({ line_no: i + 1, raw, message: `expected 3 tokens (high low close), got ${parts.length}` });
            continue;
        }
        const h = Number(parts[0]);
        const l = Number(parts[1]);
        const c = Number(parts[2]);
        if (![h, l, c].every(Number.isFinite) || h <= 0 || l <= 0 || c <= 0) {
            errors.push({ line_no: i + 1, raw, message: `HLC must be positive finite` });
            continue;
        }
        if (l > h + 1e-9) {
            errors.push({ line_no: i + 1, raw, message: `low > high` });
            continue;
        }
        if (c < l - 1e-9 || c > h + 1e-9) {
            errors.push({ line_no: i + 1, raw, message: `close outside [low, high]` });
            continue;
        }
        bars.push({ high: h, low: l, close: c });
    }
    return { bars, errors };
}

export function validateInputs(bars, lookback) {
    if (!Array.isArray(bars) || bars.length === 0) return t('view.murrey_math.validate.bars_empty');
    if (!Number.isInteger(lookback) || lookback < 1) return t('view.murrey_math.validate.lookback');
    return null;
}

export function buildBody(bars, lookback) {
    return { bars, lookback_bars: lookback };
}

// Murrey's classic per-level significance labels. Used as tooltip + table
// annotations so traders read the level's tactical meaning at a glance.
const LEVEL_SIGNIFICANCE_META = {
    '-2/8':  { key: 'extension_low',       cls: '',    rank: 'extended' },
    '-1/8':  { key: 'extension_low',       cls: '',    rank: 'extended' },
    '0/8':   { key: 'ultimate_support',    cls: 'pos', rank: 'critical' },
    '1/8':   { key: 'weak_stall',          cls: '',    rank: 'minor' },
    '2/8':   { key: 'pivot_reverse',       cls: '',    rank: 'major' },
    '3/8':   { key: 'lower_range_edge',    cls: '',    rank: 'minor' },
    '4/8':   { key: 'major_sr_mid',        cls: '',    rank: 'critical' },
    '5/8':   { key: 'upper_range_edge',    cls: '',    rank: 'minor' },
    '6/8':   { key: 'pivot_reverse',       cls: '',    rank: 'major' },
    '7/8':   { key: 'weak_stall',          cls: '',    rank: 'minor' },
    '8/8':   { key: 'ultimate_resistance', cls: 'neg', rank: 'critical' },
    '9/8':   { key: 'extension_high',      cls: '',    rank: 'extended' },
    '10/8':  { key: 'extension_high',      cls: '',    rank: 'extended' },
};

// Exposed for tests/consumers that want the raw map without resolving label.
export const LEVEL_SIGNIFICANCE = LEVEL_SIGNIFICANCE_META;

export function significanceOf(label) {
    const m = LEVEL_SIGNIFICANCE_META[label];
    if (!m) return { label: '—', cls: '', rank: 'unknown' };
    return { label: t(`view.murrey_math.sig.${m.key}`), cls: m.cls, rank: m.rank };
}

// Classifies the current price's position within the octave. Octave is
// 0/8 to 8/8; -2/-1 and 9/10 are extensions (breakout territory).
export function pricePosition(current, levels) {
    if (!Number.isFinite(current) || !Array.isArray(levels) || levels.length === 0)
        return 'unknown';
    const get = (lbl) => levels.find(([l]) => l === lbl)?.[1];
    const v0 = get('0/8');
    const v4 = get('4/8');
    const v8 = get('8/8');
    if (!Number.isFinite(v0) || !Number.isFinite(v4) || !Number.isFinite(v8))
        return 'unknown';
    if (current < v0) return 'below octave';
    if (current > v8) return 'above octave';
    if (current <= v4) return 'lower half';
    return 'upper half';
}

// Translate a pricePosition() return value into a display string.
export function pricePositionLabel(pos) {
    switch (pos) {
        case 'below octave': return t('view.murrey_math.pos.below_octave');
        case 'above octave': return t('view.murrey_math.pos.above_octave');
        case 'lower half':   return t('view.murrey_math.pos.lower_half');
        case 'upper half':   return t('view.murrey_math.pos.upper_half');
        default:             return pos;
    }
}

// Auto-detect the bracketing levels for the current price — useful for
// the "trading between X/8 and Y/8" summary card.
export function bracketingLevels(current, levels) {
    if (!Number.isFinite(current) || !Array.isArray(levels) || levels.length === 0)
        return { below: null, above: null };
    let below = null, above = null;
    for (const [lbl, v] of levels) {
        if (!Number.isFinite(v)) continue;
        if (v <= current && (below == null || v > below[1])) below = [lbl, v];
        if (v >= current && (above == null || v < above[1])) above = [lbl, v];
    }
    return { below, above };
}

// Deterministic demo: 80-bar HLC series with a clear ~10-point range
// that lands cleanly on a Murrey octave at default lookback 64.
export function makeDemoBars() {
    const out = [];
    let price = 100;
    for (let i = 0; i < 80; i++) {
        // Oscillate inside a 95-105 range with one near-top spike.
        const phase = Math.sin(i / 6);
        price = 100 + phase * 4 + (i % 7 === 0 ? 1 : 0);
        out.push({
            high:  Number((price + 0.6).toFixed(2)),
            low:   Number((price - 0.6).toFixed(2)),
            close: Number(price.toFixed(2)),
        });
    }
    return out;
}

export function fmtN(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtPct(v) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(2) + '%';
}
