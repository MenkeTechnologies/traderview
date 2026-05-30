// Amihud (2002) Illiquidity Ratio helpers.
//
// Backend body: { returns: number[], dollar_volumes: number[], period }
// Returns: (number | null)[] of length returns.length.
//
// Per-bar illiq_t = |return_t| / dollar_volume_t · 10^6   (skipped when dv ≤ 0 or non-finite)
// amihud_t = rolling mean of per-bar illiq over `period` bars (skipping nulls).
//
// Higher = less liquid (more price impact per dollar traded).

import { t } from './i18n.js';

export const DEFAULT_PERIOD = 21;

export const DEFAULT_INPUTS = {
    returns: [],
    dollar_volumes: [],
    period: DEFAULT_PERIOD,
};

export function validateInputs(input) {
    if (!Array.isArray(input.returns))                          return t('view.amihud.validate.returns_array');
    if (!Array.isArray(input.dollar_volumes))                   return t('view.amihud.validate.dollar_volumes_array');
    if (input.returns.length !== input.dollar_volumes.length)   return t('view.amihud.validate.length_mismatch');
    for (let i = 0; i < input.returns.length; i++) {
        // NaN tolerated in series (matches Rust skip behavior); only reject non-numeric.
        if (typeof input.returns[i] !== 'number' && input.returns[i] != null)
                                                                  return t('view.amihud.validate.return_type', { i });
        if (typeof input.dollar_volumes[i] !== 'number' && input.dollar_volumes[i] != null)
                                                                  return t('view.amihud.validate.dollar_volume_type', { i });
    }
    if (!Number.isInteger(input.period))                        return t('view.amihud.validate.period_int');
    if (input.period < 1)                                       return t('view.amihud.validate.period_min');
    return null;
}

export function buildBody(input) {
    return {
        returns:        input.returns,
        dollar_volumes: input.dollar_volumes,
        period:         input.period,
    };
}

// Pure-JS mirror of crates/traderview-core/src/amihud_illiquidity.rs::compute.
// Returns same warmup-nulls + finite-only guards.
export function localCompute(returns, dollar_volumes, period) {
    const n = returns.length;
    const out = new Array(n).fill(null);
    if (period === 0 || returns.length !== dollar_volumes.length || n < period) return out;
    const per_bar = new Array(n).fill(null);
    for (let i = 0; i < n; i++) {
        const r = returns[i];
        const dv = dollar_volumes[i];
        if (!Number.isFinite(r) || !Number.isFinite(dv) || dv <= 0) continue;
        const v = Math.abs(r) / dv * 1_000_000;
        if (Number.isFinite(v)) per_bar[i] = v;
    }
    for (let i = period - 1; i < n; i++) {
        let sum = 0, count = 0;
        for (let k = i + 1 - period; k <= i; k++) {
            if (per_bar[k] != null) { sum += per_bar[k]; count++; }
        }
        if (count > 0) {
            const mean = sum / count;
            if (Number.isFinite(mean)) out[i] = mean;
        }
    }
    return out;
}

// Parse "return dollar_volume" per line (pct-suffix accepted on return).
export function parsePairsBlob(blob) {
    const out = { returns: [], dollar_volumes: [], errors: [] };
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
            out.errors.push({ line_no: i + 1, message: 'expected 2 tokens (return dollar_volume)' });
            continue;
        }
        const r = pctOrDec(toks[0]);
        const dv = Number(toks[1]);
        if (!Number.isFinite(r) && raw.toLowerCase() !== 'nan nan') {
            out.errors.push({ line_no: i + 1, message: 'return must be finite or pct' });
            continue;
        }
        if (!Number.isFinite(dv) && raw.toLowerCase() !== 'nan nan') {
            out.errors.push({ line_no: i + 1, message: 'dollar_volume must be finite' });
            continue;
        }
        out.returns.push(r);
        out.dollar_volumes.push(dv);
    }
    return out;
}

function pctOrDec(tok) {
    if (typeof tok === 'string' && tok.endsWith('%')) {
        const v = Number(tok.slice(0, -1));
        return Number.isFinite(v) ? v / 100 : NaN;
    }
    return Number(tok);
}

export function pairsToBlob(returns, dollar_volumes) {
    return returns.map((r, i) => `${r} ${dollar_volumes[i]}`).join('\n');
}

// 5-tier liquidity verdict — calibrated for the canonical Amihud scale
// (return × 10^6 / dollar_vol). 0.0001 ≈ S&P 500 large-caps; 1.0 ≈ illiquid penny.
export function liquidityBadge(last) {
    if (!Number.isFinite(last))    return { key: 'view.amihud.badge.unknown', cls: '' };
    if (last < 0.001)              return { key: 'view.amihud.badge.deep',     cls: 'pos' };
    if (last < 0.01)               return { key: 'view.amihud.badge.liquid',   cls: 'pos' };
    if (last < 0.1)                return { key: 'view.amihud.badge.normal',   cls: '' };
    if (last < 1.0)                return { key: 'view.amihud.badge.thin',     cls: 'neg' };
    return { key: 'view.amihud.badge.illiquid', cls: 'neg' };
}

// Trend verdict: is liquidity improving or deteriorating over the series?
export function trendBadge(series) {
    if (!Array.isArray(series)) return { key: 'view.amihud.trend.unknown', cls: '' };
    const valid = [];
    for (const v of series) if (v != null && Number.isFinite(v)) valid.push(v);
    if (valid.length < 2) return { key: 'view.amihud.trend.unknown', cls: '' };
    const halfIdx = Math.floor(valid.length / 2);
    const earlier = valid.slice(0, halfIdx);
    const later = valid.slice(halfIdx);
    const meanE = earlier.reduce((s, v) => s + v, 0) / earlier.length;
    const meanL = later.reduce((s, v) => s + v, 0) / later.length;
    if (meanL < meanE * 0.5)  return { key: 'view.amihud.trend.improving_fast', cls: 'pos' };
    if (meanL < meanE * 0.9)  return { key: 'view.amihud.trend.improving',      cls: 'pos' };
    if (meanL < meanE * 1.1)  return { key: 'view.amihud.trend.stable',         cls: '' };
    if (meanL < meanE * 2.0)  return { key: 'view.amihud.trend.deteriorating',  cls: 'neg' };
    return { key: 'view.amihud.trend.crashing', cls: 'neg' };
}

export function summarize(series) {
    if (!Array.isArray(series) || series.length === 0)
        return { count: 0, populated: 0, mean: NaN, min: NaN, max: NaN, last: NaN };
    let sum = 0, mn = Infinity, mx = -Infinity, populated = 0, last = NaN;
    for (const v of series) {
        if (v != null && Number.isFinite(v)) {
            populated++;
            sum += v;
            if (v < mn) mn = v;
            if (v > mx) mx = v;
            last = v;
        }
    }
    return {
        count: series.length,
        populated,
        mean: populated > 0 ? sum / populated : NaN,
        min: Number.isFinite(mn) ? mn : NaN,
        max: Number.isFinite(mx) ? mx : NaN,
        last,
    };
}

// LCG for stable demos.
function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF - 0.5;
    };
}

export function makeDemoInput(kind = 'mid-cap') {
    switch (kind) {
        case 'large-cap': {
            // Big dollar volumes (mega-cap mean ~$10B / day) → very low Amihud.
            const rand = lcg(42n);
            const returns = [], dv = [];
            for (let i = 0; i < 60; i++) {
                returns.push(rand() * 0.02);
                dv.push(1e10 + rand() * 1e9);
            }
            return { returns, dollar_volumes: dv, period: 21 };
        }
        case 'mid-cap': {
            const rand = lcg(7n);
            const returns = [], dv = [];
            for (let i = 0; i < 60; i++) {
                returns.push(rand() * 0.03);
                dv.push(1e8 + rand() * 1e7);
            }
            return { returns, dollar_volumes: dv, period: 21 };
        }
        case 'small-cap': {
            const rand = lcg(123n);
            const returns = [], dv = [];
            for (let i = 0; i < 60; i++) {
                returns.push(rand() * 0.05);
                dv.push(1e6 + rand() * 1e5);
            }
            return { returns, dollar_volumes: dv, period: 21 };
        }
        case 'penny-illiquid': {
            const rand = lcg(99n);
            const returns = [], dv = [];
            for (let i = 0; i < 60; i++) {
                returns.push(rand() * 0.1);
                dv.push(5e4 + rand() * 1e4);
            }
            return { returns, dollar_volumes: dv, period: 21 };
        }
        case 'liquidity-shock': {
            // Mid-cap deteriorating into a liquidity crisis.
            const rand = lcg(31n);
            const returns = [], dv = [];
            for (let i = 0; i < 30; i++) {
                returns.push(rand() * 0.03);
                dv.push(1e8 + rand() * 1e7);
            }
            for (let i = 0; i < 30; i++) {
                returns.push(rand() * 0.06);
                dv.push(1e6 + rand() * 1e5);
            }
            return { returns, dollar_volumes: dv, period: 21 };
        }
        case 'recovery': {
            // Illiquid → liquid recovery.
            const rand = lcg(57n);
            const returns = [], dv = [];
            for (let i = 0; i < 30; i++) {
                returns.push(rand() * 0.05);
                dv.push(5e5 + rand() * 1e5);
            }
            for (let i = 0; i < 30; i++) {
                returns.push(rand() * 0.02);
                dv.push(1e9 + rand() * 1e8);
            }
            return { returns, dollar_volumes: dv, period: 21 };
        }
        case 'spotty-volume': {
            // Sprinkle some 0-volume / NaN bars.
            const rand = lcg(11n);
            const returns = [], dv = [];
            for (let i = 0; i < 60; i++) {
                returns.push(rand() * 0.03);
                dv.push(i % 7 === 0 ? 0 : 1e8 + rand() * 1e7);
            }
            returns[10] = NaN;
            dv[20] = NaN;
            return { returns, dollar_volumes: dv, period: 21 };
        }
        case 'short-period': {
            const rand = lcg(3n);
            const returns = [], dv = [];
            for (let i = 0; i < 20; i++) {
                returns.push(rand() * 0.03);
                dv.push(1e7 + rand() * 1e6);
            }
            return { returns, dollar_volumes: dv, period: 5 };
        }
        default: return makeDemoInput('mid-cap');
    }
}

export function fmtAmihud(v, d = 6) {
    if (v == null || !Number.isFinite(v)) return '—';
    if (v >= 0.01) return v.toFixed(4);
    return v.toExponential(3);
}

export function fmtPct(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtDV(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    if (Math.abs(v) >= 1e9) return '$' + (v / 1e9).toFixed(2) + 'B';
    if (Math.abs(v) >= 1e6) return '$' + (v / 1e6).toFixed(2) + 'M';
    if (Math.abs(v) >= 1e3) return '$' + (v / 1e3).toFixed(2) + 'k';
    return '$' + v.toFixed(0);
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
