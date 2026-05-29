// Choppiness Index helpers shared by view + vitest.
//
// Backend body: { bars: [{high, low, close}, ...], period: usize }.
// Returns: { series: (f64 | null)[], latest: f64 | null,
//   regime: 'trending'|'mixed'|'choppy', note: string }.
//
// Formula (E.W. Dreiss): CI = 100 * log10(sum_TR_n / (max_H_n - min_L_n)) / log10(n).
// Reuse HLC parser from _chandelier_stop_inputs.js so this isn't duplicated.

import { t } from './i18n.js';
export { parseBarBlob } from './_chandelier_stop_inputs.js';

export function validateInputs(bars, period) {
    if (!Array.isArray(bars) || bars.length === 0) return t('view.choppiness.validate.need_bar');
    if (!Number.isInteger(period) || period < 2) return t('view.choppiness.validate.period_min');
    if (bars.length < period + 1) return t('view.choppiness.validate.bars_lt_period', { n: period + 1 });
    return null;
}

export function buildBody(bars, period) {
    return { bars, period };
}

// Pure-JS mirror of crates/traderview-core/src/choppiness.rs::compute.
// Same warmup-as-null semantics; degenerate (zero-range) windows → null.
export function localCompute(bars, period) {
    const n = bars.length;
    if (n === 0 || period < 2 || n < period + 1) {
        return {
            series: new Array(n).fill(null),
            latest: null, regime: 'mixed',
            note: `need ≥ ${period + 1} bars, got ${n}`,
        };
    }
    const series = new Array(n).fill(null);
    const log10n = Math.log10(period);
    for (let i = period; i < n; i++) {
        const winStart = i + 1 - period;
        let sumTr = 0;
        let hi = -Infinity, lo = Infinity;
        for (let j = winStart; j <= i; j++) {
            sumTr += trueRangeAt(bars, j);
            if (bars[j].high > hi) hi = bars[j].high;
            if (bars[j].low  < lo) lo = bars[j].low;
        }
        const env = hi - lo;
        if (env <= 0 || sumTr <= 0) continue;
        series[i] = 100 * Math.log10(sumTr / env) / log10n;
    }
    const latest = series.length > 0 ? series[series.length - 1] : null;
    const regime = latest == null ? 'mixed'
                 : latest > 61.8  ? 'choppy'
                 : latest < 38.2  ? 'trending'
                 :                  'mixed';
    const note = latest == null
        ? 'no value yet'
        : `CI = ${latest.toFixed(1)} → ${regimeRustName(regime)}`;
    return { series, latest, regime, note };
}

// True range at bar i: max(H-L, |H-prevClose|, |L-prevClose|).
// Bar 0 = H - L (no prev close).
export function trueRangeAt(bars, i) {
    if (i === 0) return bars[0].high - bars[0].low;
    const pc = bars[i - 1].close;
    const a = bars[i].high - bars[i].low;
    const b = Math.abs(bars[i].high - pc);
    const c = Math.abs(bars[i].low  - pc);
    return Math.max(a, b, c);
}

// Rust serializes the enum capitalized in the `note` string ("Trending"
// not "trending") via Debug — match that for parity.
function regimeRustName(r) {
    if (r === 'trending') return 'Trending';
    if (r === 'choppy')   return 'Choppy';
    return 'Mixed';
}

const REGIME_BADGES = {
    trending: { key: 'trending', cls: 'pos' },
    mixed:    { key: 'mixed',    cls: '' },
    choppy:   { key: 'choppy',   cls: 'neg' },
};

export function regimeBadge(r) {
    const x = REGIME_BADGES[r];
    if (!x) return { label: String(r || '—').toUpperCase(), cls: '', hint: '—' };
    return {
        label: t(`view.choppiness.regime.${x.key}.label`),
        cls: x.cls,
        hint: t(`view.choppiness.regime.${x.key}.hint`),
    };
}

// Time spent in each regime across the series (informative summary).
export function regimeBuckets(series) {
    const buckets = { trending: 0, mixed: 0, choppy: 0, warmup: 0 };
    for (const v of series) {
        if (v == null)         buckets.warmup++;
        else if (v > 61.8)     buckets.choppy++;
        else if (v < 38.2)     buckets.trending++;
        else                   buckets.mixed++;
    }
    return buckets;
}

// Find the most-recent regime switch (last index where regime differs
// from the latest regime). Returns null if no switch in the series.
export function lastRegimeSwitch(series) {
    if (!Array.isArray(series) || series.length === 0) return null;
    const cls = (v) => v == null ? null : (v > 61.8 ? 'choppy' : v < 38.2 ? 'trending' : 'mixed');
    let latestRegime = null;
    for (let i = series.length - 1; i >= 0; i--) {
        const r = cls(series[i]);
        if (r == null) continue;
        if (latestRegime == null) { latestRegime = r; continue; }
        if (r !== latestRegime) return { switchedAt: i + 1, fromRegime: r, toRegime: latestRegime };
    }
    return null;
}

// Demo presets matching each regime outcome. Each runs through localCompute
// and SHOULD classify into its named regime.
export function makeDemoBars(kind = 'mixed') {
    switch (kind) {
        case 'trending-up': {
            // Strong directional uptrend, narrow per-bar range.
            const out = [];
            let p = 100;
            for (let i = 0; i < 60; i++) { p += 0.6; out.push({ high: p + 0.3, low: p - 0.3, close: p + 0.2 }); }
            return out;
        }
        case 'trending-down': {
            const out = [];
            let p = 150;
            for (let i = 0; i < 60; i++) { p -= 0.7; out.push({ high: p + 0.3, low: p - 0.3, close: p - 0.2 }); }
            return out;
        }
        case 'choppy': {
            // Tight oscillation: sum-TR keeps growing while envelope stays narrow.
            const out = [];
            for (let i = 0; i < 60; i++) {
                const p = i % 2 === 0 ? 100.5 : 99.5;
                out.push({ high: p + 0.1, low: p - 0.1, close: p });
            }
            return out;
        }
        case 'mixed': {
            // Drift with moderate noise → CI ~38–62.
            const out = [];
            for (let i = 0; i < 60; i++) {
                const p = 100 + i * 0.2 + Math.sin(i * 0.7) * 0.8;
                out.push({ high: p + 0.4, low: p - 0.4, close: p + Math.cos(i * 0.5) * 0.2 });
            }
            return out;
        }
        case 'trend-then-chop': {
            // 30 bars trending → 30 bars chop. Regime switch happens.
            const out = [];
            let p = 100;
            for (let i = 0; i < 30; i++) { p += 0.6; out.push({ high: p + 0.3, low: p - 0.3, close: p + 0.2 }); }
            const flat = p;
            for (let i = 0; i < 30; i++) {
                const q = i % 2 === 0 ? flat + 0.5 : flat - 0.5;
                out.push({ high: q + 0.1, low: q - 0.1, close: q });
            }
            return out;
        }
        default:
            return makeDemoBars('mixed');
    }
}

export function fmtN(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtPct(v, d = 1) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}
