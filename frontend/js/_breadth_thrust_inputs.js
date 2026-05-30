// Zweig Breadth Thrust (1986) helpers.
//
// Backend body: { breadth: [{advancing, declining}, ...],
//   ema_period, max_window_bars, low_threshold, high_threshold }
// Returns: { ratio: (number|null)[], ema_ratio: (number|null)[],
//   thrust_triggered: boolean[], ema_period, max_window_bars,
//   low_threshold, high_threshold }
//
// ratio_t = adv / (adv + dec). EMA of ratio over ema_period bars.
// Thrust triggered when ema went from <low to >high within max_window_bars.

import { t } from './i18n.js';

export const DEFAULT_EMA_PERIOD = 10;
export const DEFAULT_MAX_WINDOW = 10;
export const DEFAULT_LOW  = 0.40;
export const DEFAULT_HIGH = 0.615;

export const DEFAULT_INPUTS = {
    breadth: [],
    ema_period:      DEFAULT_EMA_PERIOD,
    max_window_bars: DEFAULT_MAX_WINDOW,
    low_threshold:   DEFAULT_LOW,
    high_threshold:  DEFAULT_HIGH,
};

export function validateInputs(input) {
    if (!Array.isArray(input.breadth))                                  return t('view.breadth_thrust.validate.breadth_array');
    for (let i = 0; i < input.breadth.length; i++) {
        const b = input.breadth[i];
        if (!b || typeof b !== 'object')                                return t('view.breadth_thrust.validate.row_object', { i });
        if (!Number.isInteger(b.advancing) || b.advancing < 0)          return t('view.breadth_thrust.validate.advancing', { i });
        if (!Number.isInteger(b.declining) || b.declining < 0)          return t('view.breadth_thrust.validate.declining', { i });
    }
    if (!Number.isInteger(input.ema_period))                            return t('view.breadth_thrust.validate.ema_period_int');
    if (input.ema_period < 2)                                           return t('view.breadth_thrust.validate.ema_period_min');
    if (!Number.isInteger(input.max_window_bars))                       return t('view.breadth_thrust.validate.window_int');
    if (input.max_window_bars < 2)                                      return t('view.breadth_thrust.validate.window_min');
    if (!Number.isFinite(input.low_threshold) || input.low_threshold < 0 || input.low_threshold > 1)
                                                                          return t('view.breadth_thrust.validate.low_threshold');
    if (!Number.isFinite(input.high_threshold) || input.high_threshold < 0 || input.high_threshold > 1)
                                                                          return t('view.breadth_thrust.validate.high_threshold');
    if (input.low_threshold >= input.high_threshold)                    return t('view.breadth_thrust.validate.low_lt_high');
    return null;
}

export function buildBody(input) {
    return {
        breadth:         input.breadth.map(b => ({ advancing: b.advancing, declining: b.declining })),
        ema_period:      input.ema_period,
        max_window_bars: input.max_window_bars,
        low_threshold:   input.low_threshold,
        high_threshold:  input.high_threshold,
    };
}

// Pure-JS mirror of crates/traderview-core/src/breadth_thrust.rs::compute.
export function localCompute(breadth, ema_period, max_window_bars, low_threshold, high_threshold) {
    const n = breadth.length;
    const report = {
        ratio:            new Array(n).fill(null),
        ema_ratio:        new Array(n).fill(null),
        thrust_triggered: new Array(n).fill(false),
        ema_period,
        max_window_bars,
        low_threshold,
        high_threshold,
    };
    if (ema_period < 2 || max_window_bars < 2
        || !Number.isFinite(low_threshold) || !Number.isFinite(high_threshold)
        || low_threshold >= high_threshold
        || low_threshold < 0 || low_threshold > 1 || high_threshold < 0 || high_threshold > 1
        || n < ema_period) {
        return report;
    }
    const raw = new Array(n);
    for (let i = 0; i < n; i++) {
        const denom = breadth[i].advancing + breadth[i].declining;
        raw[i] = denom > 0 ? breadth[i].advancing / denom : 0.5;
        report.ratio[i] = raw[i];
    }
    const k = 2 / (ema_period + 1);
    let seed = 0;
    for (let i = 0; i < ema_period; i++) seed += raw[i];
    seed /= ema_period;
    report.ema_ratio[ema_period - 1] = seed;
    let cur = seed;
    for (let i = ema_period; i < n; i++) {
        cur = raw[i] * k + cur * (1 - k);
        report.ema_ratio[i] = cur;
    }
    // Trigger when ema crossed from <low to >high within max_window_bars.
    for (let i = ema_period + max_window_bars - 1; i < n; i++) {
        const ema_now = report.ema_ratio[i];
        if (ema_now == null || ema_now <= high_threshold) continue;
        for (let back = 1; back <= max_window_bars; back++) {
            const ema_then = report.ema_ratio[i - back];
            if (ema_then == null) continue;
            if (ema_then < low_threshold) {
                report.thrust_triggered[i] = true;
                break;
            }
        }
    }
    return report;
}

// Parse "advancing declining" per line; comments + blanks ignored.
export function parseBreadthBlob(blob) {
    const out = { breadth: [], errors: [] };
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
            out.errors.push({ line_no: i + 1, message: 'expected 2 tokens (advancing declining)' });
            continue;
        }
        const adv = Number(toks[0]);
        const dec = Number(toks[1]);
        if (!Number.isInteger(adv) || adv < 0) {
            out.errors.push({ line_no: i + 1, message: 'advancing must be non-negative integer' });
            continue;
        }
        if (!Number.isInteger(dec) || dec < 0) {
            out.errors.push({ line_no: i + 1, message: 'declining must be non-negative integer' });
            continue;
        }
        out.breadth.push({ advancing: adv, declining: dec });
    }
    return out;
}

export function breadthToBlob(breadth) {
    return breadth.map(b => `${b.advancing} ${b.declining}`).join('\n');
}

// Verdict on the last EMA ratio.
export function regimeBadge(last_ema, low, high) {
    if (last_ema == null || !Number.isFinite(last_ema))   return { key: 'view.breadth.badge.unknown', cls: '' };
    if (last_ema > high)                                  return { key: 'view.breadth.badge.bullish_thrust', cls: 'pos' };
    if (last_ema > 0.55)                                  return { key: 'view.breadth.badge.strong',         cls: 'pos' };
    if (last_ema >= 0.45)                                 return { key: 'view.breadth.badge.neutral',        cls: '' };
    if (last_ema >= low)                                  return { key: 'view.breadth.badge.weak',           cls: 'neg' };
    return { key: 'view.breadth.badge.washout', cls: 'neg' };
    void low; void high;
}

// Did any thrust trigger in the series?
export function thrustBadge(thrusts) {
    if (!Array.isArray(thrusts) || thrusts.length === 0)  return { key: 'view.breadth.thrust.unknown', cls: '' };
    let count = 0;
    let lastIdx = -1;
    for (let i = 0; i < thrusts.length; i++) {
        if (thrusts[i]) { count++; lastIdx = i; }
    }
    if (count === 0) return { key: 'view.breadth.thrust.none', cls: '' };
    if (count === 1) return { key: 'view.breadth.thrust.fired',    cls: 'pos' };
    return { key: 'view.breadth.thrust.multiple', cls: 'pos' };
    void lastIdx;
}

// Index of the most recent thrust trigger, or null.
export function lastThrustIndex(thrusts) {
    if (!Array.isArray(thrusts)) return null;
    for (let i = thrusts.length - 1; i >= 0; i--) {
        if (thrusts[i]) return i;
    }
    return null;
}

// Summary stats.
export function summarize(report) {
    if (!report || !Array.isArray(report.ema_ratio) || report.ema_ratio.length === 0)
        return { count: 0, populated: 0, thrust_count: 0,
                 last_ratio: NaN, last_ema: NaN, min_ema: NaN, max_ema: NaN };
    let populated = 0, lastE = NaN, mn = Infinity, mx = -Infinity;
    let lastR = NaN, count_thrust = 0;
    for (let i = 0; i < report.ema_ratio.length; i++) {
        const v = report.ema_ratio[i];
        if (v != null && Number.isFinite(v)) {
            populated++;
            lastE = v;
            if (v < mn) mn = v;
            if (v > mx) mx = v;
        }
        const r = report.ratio[i];
        if (r != null && Number.isFinite(r)) lastR = r;
        if (report.thrust_triggered[i]) count_thrust++;
    }
    return {
        count: report.ema_ratio.length,
        populated,
        thrust_count: count_thrust,
        last_ratio: lastR,
        last_ema: lastE,
        min_ema: Number.isFinite(mn) ? mn : NaN,
        max_ema: Number.isFinite(mx) ? mx : NaN,
    };
}

// Demos.
function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF;
    };
}

export function makeDemoInput(kind = 'classic-thrust') {
    switch (kind) {
        case 'classic-thrust': {
            const b = [];
            for (let i = 0; i < 30; i++) b.push({ advancing: 30, declining: 70 });
            for (let i = 0; i < 15; i++) b.push({ advancing: 90, declining: 10 });
            return { ...DEFAULT_INPUTS, breadth: b };
        }
        case 'flat-balanced': {
            const b = [];
            for (let i = 0; i < 60; i++) b.push({ advancing: 50, declining: 50 });
            return { ...DEFAULT_INPUTS, breadth: b };
        }
        case 'slow-recovery': {
            const b = [];
            for (let i = 0; i < 60; i++) {
                const adv = 30 + i;
                const dec = Math.max(1, 70 - i);
                b.push({ advancing: adv, declining: dec });
            }
            return { ...DEFAULT_INPUTS, breadth: b, max_window_bars: 5 };
        }
        case 'multi-thrust': {
            const b = [];
            for (let i = 0; i < 30; i++) b.push({ advancing: 30, declining: 70 });
            for (let i = 0; i < 15; i++) b.push({ advancing: 90, declining: 10 });
            for (let i = 0; i < 20; i++) b.push({ advancing: 25, declining: 75 });
            for (let i = 0; i < 15; i++) b.push({ advancing: 95, declining: 5 });
            return { ...DEFAULT_INPUTS, breadth: b };
        }
        case 'washout-only': {
            // Very weak breadth, never recovers.
            const b = [];
            for (let i = 0; i < 50; i++) b.push({ advancing: 10, declining: 90 });
            return { ...DEFAULT_INPUTS, breadth: b };
        }
        case 'noisy-walk': {
            const rand = lcg(42n);
            const b = [];
            for (let i = 0; i < 100; i++) {
                const u = rand();
                b.push({ advancing: Math.trunc(20 + u * 80), declining: Math.trunc(20 + (1 - u) * 80) });
            }
            return { ...DEFAULT_INPUTS, breadth: b };
        }
        case 'tight-window': {
            // Classic thrust but max_window=3 → no trigger (rise too slow).
            const b = [];
            for (let i = 0; i < 30; i++) b.push({ advancing: 30, declining: 70 });
            for (let i = 0; i < 15; i++) b.push({ advancing: 90, declining: 10 });
            return { ...DEFAULT_INPUTS, breadth: b, max_window_bars: 3 };
        }
        case 'custom-thresholds': {
            // Less strict 0.45/0.55 thresholds catch weaker rallies.
            const b = [];
            for (let i = 0; i < 30; i++) b.push({ advancing: 40, declining: 60 });
            for (let i = 0; i < 15; i++) b.push({ advancing: 65, declining: 35 });
            return { ...DEFAULT_INPUTS, breadth: b, low_threshold: 0.45, high_threshold: 0.55 };
        }
        default: return makeDemoInput('classic-thrust');
    }
}

export function fmtRatio(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtPct(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
