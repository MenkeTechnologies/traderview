// Cup-and-Handle helpers shared by view + vitest.
//
// Backend body shape: { bars: [{high, low, close}, ...], config: {
// cup_min_bars, cup_max_bars, min_depth_pct, max_depth_pct,
// rim_tolerance_pct, handle_min_bars, handle_max_bars,
// max_handle_depth_pct } }.

import { t } from './i18n.js';

const TOKEN_DELIM = /[\s,]+/;

// Parses three-token-per-line OHLC bars: "high low close" (open is not
// needed by the detector). Lines starting with `#` and blank lines are
// skipped; per-line errors are tagged.
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
        const high = Number(parts[0]);
        const low  = Number(parts[1]);
        const close = Number(parts[2]);
        if (![high, low, close].every(Number.isFinite) || high <= 0 || low <= 0 || close <= 0) {
            errors.push({ line_no: i + 1, raw, message: `non-positive or non-finite OHLC` });
            continue;
        }
        if (low > high + 1e-9) {
            errors.push({ line_no: i + 1, raw, message: `low > high` });
            continue;
        }
        if (close < low - 1e-9 || close > high + 1e-9) {
            errors.push({ line_no: i + 1, raw, message: `close outside [low, high]` });
            continue;
        }
        bars.push({ high, low, close });
    }
    return { bars, errors };
}

export function validateInputs(bars, config) {
    if (!Array.isArray(bars)) return 'bars must be an array';
    if (config.cup_min_bars < 4) return 'cup_min_bars must be ≥ 4';
    if (config.cup_max_bars <= config.cup_min_bars) return 'cup_max_bars must be > cup_min_bars';
    if (config.handle_min_bars < 1) return 'handle_min_bars must be ≥ 1';
    if (config.handle_max_bars < config.handle_min_bars) return 'handle_max_bars must be ≥ handle_min_bars';
    if (config.min_depth_pct <= 0) return 'min_depth_pct must be > 0';
    if (config.max_depth_pct <= config.min_depth_pct) return 'max_depth_pct must be > min_depth_pct';
    if (config.rim_tolerance_pct < 0) return 'rim_tolerance_pct must be ≥ 0';
    if (config.max_handle_depth_pct <= 0) return 'max_handle_depth_pct must be > 0';
    const needed = config.cup_min_bars + config.handle_min_bars;
    if (bars.length < needed) return `need at least ${needed} bars (cup_min + handle_min)`;
    return null;
}

export function buildBody(bars, config) {
    return { bars, config };
}

// Synthesizes a deterministic cup-and-handle bar sequence. Used by the
// "Demo data" button so users see the detector flag something on the
// first visit. Shape: 30 pre-bars of drift, 80-bar cup (cosine to make
// a clean U), 12-bar handle that drifts down ~8%, then pivot.
export function makeDemoBars(seed = 1) {
    let s = seed;
    const rand = () => { s = (s * 1664525 + 1013904223) | 0; return ((s >>> 0) / 0xffffffff); };
    const noise = (amp) => (rand() * 2 - 1) * amp;
    const bars = [];
    let price = 100;
    const push = (target) => {
        const o = price;
        const c = target + noise(0.15);
        const hi = Math.max(o, c) + Math.abs(noise(0.4)) + 0.1;
        const lo = Math.min(o, c) - Math.abs(noise(0.4)) - 0.1;
        bars.push({ high: round2(hi), low: round2(lo), close: round2(c) });
        price = c;
    };
    // 30 bars of mild drift up to ~100.
    for (let i = 0; i < 30; i++) push(95 + i * 0.18 + noise(0.5));
    // 80-bar U cup from 100 → trough 78 → 100.
    const cupLen = 80;
    const left = price;
    const trough = left * 0.78;
    for (let i = 1; i <= cupLen; i++) {
        const phase = (i / cupLen) * Math.PI;        // 0 → π
        const target = (left + trough) / 2 + (left - trough) / 2 * Math.cos(phase);
        push(target);
    }
    // 12-bar handle: drift down to ~92% of right rim, then drift back near rim.
    const rightRim = price;
    const handleLow = rightRim * 0.92;
    for (let i = 1; i <= 7; i++) {
        const f = i / 7;
        push(rightRim * (1 - f * 0.08) + noise(0.2));
    }
    for (let i = 1; i <= 5; i++) {
        const f = i / 5;
        push(handleLow + (rightRim - handleLow) * f * 0.9 + noise(0.2));
    }
    return bars;
}

function round2(v) { return Math.round(v * 100) / 100; }

export function fmtN(v, digits = 2) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(digits);
}

export function fmtPct(v) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(2) + '%';
}

// Categorizes a depth in [0, 1] into the IBD-style narrative bucket. Used
// to give traders a one-glance "is this a textbook cup?" verdict.
export function depthQuality(depthPct) {
    if (!Number.isFinite(depthPct)) return { label: '—', cls: '' };
    if (depthPct < 0.12) return { label: t('view.cup_and_handle.depth.shallow'), cls: 'neg' };
    if (depthPct <= 0.33) return { label: t('view.cup_and_handle.depth.textbook'), cls: 'pos' };
    return { label: t('view.cup_and_handle.depth.deep'), cls: 'neg' };
}
