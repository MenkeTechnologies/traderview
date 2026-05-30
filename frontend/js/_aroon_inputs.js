// Aroon Indicator (Chande 1995) helpers.
//
// Backend body: { bars: [{high, low}, ...], period }
// Returns: { aroon_up: (number|null)[], aroon_down: (number|null)[],
//   aroon_oscillator: (number|null)[] }
//
// AroonUp_t   = 100·(period − bars_since_period_high) / period
// AroonDown_t = 100·(period − bars_since_period_low)  / period
// AroonOsc_t  = AroonUp − AroonDown    ∈ [−100, +100]
//
// Window: (period + 1) bars [i − period .. i].
// Ties resolve to OLDEST bar (matches Rust strict `>` / `<` comparison).

import { t } from './i18n.js';

export const DEFAULT_PERIOD = 25;

export const DEFAULT_INPUTS = {
    bars: [],
    period: DEFAULT_PERIOD,
};

export function validateInputs(input) {
    if (!Array.isArray(input.bars))                              return t('view.aroon.validate.bars_array');
    for (let i = 0; i < input.bars.length; i++) {
        const b = input.bars[i];
        if (!b || typeof b !== 'object')                         return t('view.aroon.validate.bar_object', { i });
        if (!Number.isFinite(b.high))                            return t('view.aroon.validate.bar_high_finite', { i });
        if (!Number.isFinite(b.low))                             return t('view.aroon.validate.bar_low_finite', { i });
        if (b.high < b.low)                                      return t('view.aroon.validate.bar_high_low', { i });
    }
    if (!Number.isInteger(input.period))                         return t('view.aroon.validate.period_int');
    if (input.period < 2)                                        return t('view.aroon.validate.period_min');
    return null;
}

export function buildBody(input) {
    return {
        bars:   input.bars.map(b => ({ high: b.high, low: b.low })),
        period: input.period,
    };
}

// Pure-JS mirror of crates/traderview-core/src/aroon_indicator.rs::compute.
// Output arrays are input-length; first `period` slots are null (warmup).
export function localCompute(bars, period) {
    const n = bars.length;
    const up = new Array(n).fill(null);
    const dn = new Array(n).fill(null);
    const osc = new Array(n).fill(null);
    if (period < 2 || n < period + 1) return { aroon_up: up, aroon_down: dn, aroon_oscillator: osc };
    const pf = period;
    for (let i = period; i < n; i++) {
        // Validate window — skip if any NaN.
        let bad = false;
        for (let k = i - period; k <= i; k++) {
            const b = bars[k];
            if (!Number.isFinite(b.high) || !Number.isFinite(b.low)) { bad = true; break; }
        }
        if (bad) continue;
        let high_idx = 0;
        let low_idx = 0;
        // Window index 0 = bars[i - period] ... window index period = bars[i].
        // Most-recent bar (window index = period) → bars_since_extreme = 0 → Aroon = 100.
        for (let k = 0; k <= period; k++) {
            const w = bars[i - period + k];
            if (w.high > bars[i - period + high_idx].high) high_idx = k;
            if (w.low  < bars[i - period + low_idx].low)   low_idx  = k;
        }
        const bars_since_high = period - high_idx;
        const bars_since_low  = period - low_idx;
        const u = 100 * (pf - bars_since_high) / pf;
        const d = 100 * (pf - bars_since_low) / pf;
        up[i] = u;
        dn[i] = d;
        osc[i] = u - d;
    }
    return { aroon_up: up, aroon_down: dn, aroon_oscillator: osc };
}

// Parse "high low" per line; blanks + # comments ignored.
export function parseBarsBlob(blob) {
    const out = { bars: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: t('common.parse.input_must_be_string') });
        return out;
    }
    const lines = blob.split('\n');
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i].split('#')[0].trim();
        if (!raw) continue;
        const toks = raw.split(/[\s,]+/).filter(t => t.length > 0);
        if (toks.length !== 2) {
            out.errors.push({ line_no: i + 1, message: t('view.aroon.parse.expected_high_low') });
            continue;
        }
        const [high, low] = toks.map(Number);
        if (!Number.isFinite(high) || !Number.isFinite(low)) {
            out.errors.push({ line_no: i + 1, message: t('common.parse.non_finite_token') });
            continue;
        }
        if (high < low) {
            out.errors.push({ line_no: i + 1, message: t('common.parse.high_lt_low') });
            continue;
        }
        out.bars.push({ high, low });
    }
    return out;
}

export function barsToBlob(bars) {
    return bars.map(b => `${b.high} ${b.low}`).join('\n');
}

// 5-tier classification on last AroonOsc.
export function regimeBadge(last_osc) {
    if (!Number.isFinite(last_osc))    return { key: 'view.aroon.badge.unknown', cls: '' };
    if (last_osc >= 80)                return { key: 'view.aroon.badge.strong_up',   cls: 'pos' };
    if (last_osc >= 20)                return { key: 'view.aroon.badge.up',          cls: 'pos' };
    if (last_osc > -20)                return { key: 'view.aroon.badge.consolidate', cls: '' };
    if (last_osc > -80)                return { key: 'view.aroon.badge.down',        cls: 'neg' };
    return { key: 'view.aroon.badge.strong_down', cls: 'neg' };
}

// Crossover detection — index of last Up/Down crossover.
export function lastCrossover(report) {
    if (!report || !Array.isArray(report.aroon_up)) return null;
    const u = report.aroon_up;
    const d = report.aroon_down;
    let lastIdx = -1;
    let kind = null;
    for (let i = 1; i < u.length; i++) {
        if (u[i] == null || u[i - 1] == null || d[i] == null || d[i - 1] == null) continue;
        const prev = u[i - 1] - d[i - 1];
        const cur  = u[i] - d[i];
        if (prev <= 0 && cur > 0)      { lastIdx = i; kind = 'bull'; }
        else if (prev >= 0 && cur < 0) { lastIdx = i; kind = 'bear'; }
    }
    return lastIdx >= 0 ? { idx: lastIdx, kind } : null;
}

// Aggregate stats from the report.
export function summarize(report) {
    if (!report || !Array.isArray(report.aroon_up))
        return { count: 0, populated: 0, last_up: NaN, last_down: NaN, last_osc: NaN };
    const u = report.aroon_up;
    const d = report.aroon_down;
    const o = report.aroon_oscillator;
    let populated = 0;
    let lastU = NaN, lastD = NaN, lastO = NaN;
    for (let i = 0; i < u.length; i++) {
        if (u[i] != null) {
            populated++;
            lastU = u[i];
            lastD = d[i];
            lastO = o[i];
        }
    }
    return { count: u.length, populated, last_up: lastU, last_down: lastD, last_osc: lastO };
}

// Synthetic demos — deterministic LCG for stable test outcomes.
function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF;
    };
}

export function makeDemoInput(kind = 'strong-uptrend') {
    switch (kind) {
        case 'strong-uptrend': {
            // Strictly rising → AroonUp=100, AroonDown=0 at the tail.
            const bars = [];
            for (let i = 0; i < 60; i++) bars.push({ high: 100 + i, low: 99 + i });
            return { bars, period: 25 };
        }
        case 'strong-downtrend': {
            const bars = [];
            for (let i = 0; i < 60; i++) bars.push({ high: 200 - i, low: 199 - i });
            return { bars, period: 25 };
        }
        case 'flat': {
            const bars = [];
            for (let i = 0; i < 60; i++) bars.push({ high: 101, low: 99 });
            return { bars, period: 25 };
        }
        case 'consolidation': {
            const bars = [];
            for (let i = 0; i < 60; i++) {
                const m = 100 + Math.sin(i * 0.3) * 0.5;
                bars.push({ high: m + 1, low: m - 1 });
            }
            return { bars, period: 25 };
        }
        case 'bull-cross': {
            // Down then up so the most recent extreme flips.
            const bars = [];
            for (let i = 0; i < 30; i++) bars.push({ high: 150 - i, low: 149 - i });
            for (let i = 0; i < 40; i++) bars.push({ high: 121 + i, low: 120 + i });
            return { bars, period: 25 };
        }
        case 'bear-cross': {
            const bars = [];
            for (let i = 0; i < 30; i++) bars.push({ high: 100 + i, low: 99 + i });
            for (let i = 0; i < 40; i++) bars.push({ high: 130 - i, low: 129 - i });
            return { bars, period: 25 };
        }
        case 'noisy': {
            const rand = lcg(42n);
            const bars = [];
            let p = 100;
            for (let i = 0; i < 200; i++) {
                p += (rand() - 0.5) * 2;
                bars.push({ high: p + 0.5, low: p - 0.5 });
            }
            return { bars, period: 25 };
        }
        case 'short-period': {
            // Period=10 on a short series.
            const bars = [];
            for (let i = 0; i < 30; i++) bars.push({ high: 100 + (i % 7), low: 99 + (i % 7) });
            return { bars, period: 10 };
        }
        default: return makeDemoInput('strong-uptrend');
    }
}

export function fmtNum(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtPct(v, d = 1) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtOsc(v, d = 1) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toFixed(d);
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}

export function fmtUSD(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return '$' + v.toFixed(d);
}
