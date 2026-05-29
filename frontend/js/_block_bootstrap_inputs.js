// Block Bootstrap (Künsch 1989) helpers.
//
// Backend body: { data: number[], block_size: usize, n_resamples: usize,
//                 statistic: 'mean' | 'stdev' | 'sharpe_ratio' | 'max_drawdown',
//                 seed: u64 }
// Returns: { original_statistic, bootstrap_mean, bootstrap_stdev,
//   ci_lower_2_5_pct, ci_upper_97_5_pct, n_resamples, block_size } | null
//
// Resamples blocks of consecutive observations with replacement to
// preserve serial dependence. Use for serially-correlated returns where
// the naive iid bootstrap underestimates variance.

import { t } from './i18n.js';

export const DEFAULT_BLOCK_SIZE = 20;
export const DEFAULT_RESAMPLES = 1000;
export const DEFAULT_SEED = 0n;
export const MIN_RESAMPLES = 50;
export const MAX_RESAMPLES = 10_000;

export const STATISTICS = ['mean', 'stdev', 'sharpe_ratio', 'max_drawdown'];

export const DEFAULT_INPUTS = {
    data: [],
    block_size: DEFAULT_BLOCK_SIZE,
    n_resamples: DEFAULT_RESAMPLES,
    statistic: 'mean',
    seed: DEFAULT_SEED,
};

export function validateInputs(input) {
    if (!Array.isArray(input.data))                                  return t('view.block_bootstrap.validate.data_array');
    if (!Number.isInteger(input.block_size) || input.block_size <= 0) return t('view.block_bootstrap.validate.block_size');
    if (input.data.length < input.block_size + 2)                     return t('view.block_bootstrap.validate.data_min', { n: input.block_size + 2 });
    for (let i = 0; i < input.data.length; i++) {
        if (!Number.isFinite(input.data[i]))                          return t('view.block_bootstrap.validate.data_finite', { i });
    }
    if (!Number.isInteger(input.n_resamples))                         return t('view.block_bootstrap.validate.resamples_int');
    if (input.n_resamples < MIN_RESAMPLES || input.n_resamples > MAX_RESAMPLES)
                                                                       return t('view.block_bootstrap.validate.resamples_range', { min: MIN_RESAMPLES, max: MAX_RESAMPLES });
    if (!STATISTICS.includes(input.statistic))                        return t('view.block_bootstrap.validate.statistic', { list: STATISTICS.join(', ') });
    if (typeof input.seed !== 'bigint' && !Number.isInteger(input.seed))
                                                                       return t('view.block_bootstrap.validate.seed');
    return null;
}

export function buildBody(input) {
    return {
        data:        input.data,
        block_size:  input.block_size,
        n_resamples: input.n_resamples,
        statistic:   input.statistic,
        seed:        typeof input.seed === 'bigint' ? Number(input.seed) : input.seed,
    };
}

// Pure-JS mirror of crates/traderview-core/src/block_bootstrap.rs::bootstrap.
export function localBootstrap(data, block_size, n_resamples, statistic, seed) {
    const n = data.length;
    if (n < block_size + 2 || block_size === 0) return null;
    if (n_resamples < MIN_RESAMPLES || n_resamples > MAX_RESAMPLES) return null;
    for (const v of data) if (!Number.isFinite(v)) return null;
    const original = computeStatistic(data, statistic);
    if (original == null) return null;
    const MASK = 0xFFFFFFFFFFFFFFFFn;
    const seedBig = typeof seed === 'bigint' ? (seed & MASK) : (BigInt(seed) & MASK);
    let state = (seedBig + 1n) & MASK;
    const max_start = n - block_size + 1;
    const max_start_big = BigInt(max_start);
    const n_blocks_per_resample = Math.ceil(n / block_size);
    const stats = [];
    const buffer = new Array(n_blocks_per_resample * block_size);
    for (let r = 0; r < n_resamples; r++) {
        let bi = 0;
        for (let b = 0; b < n_blocks_per_resample; b++) {
            state = (state * 6364136223846793005n + 1442695040888963407n) & MASK;
            const start = Number((state >> 32n) % max_start_big);
            for (let k = 0; k < block_size; k++) {
                buffer[bi++] = data[start + k];
            }
        }
        const truncated = buffer.slice(0, n);
        const s = computeStatistic(truncated, statistic);
        if (s != null && Number.isFinite(s)) stats.push(s);
    }
    if (stats.length === 0) return null;
    stats.sort((a, b) => a - b);
    const m = stats.length;
    let sum = 0;
    for (const v of stats) sum += v;
    const mean = sum / m;
    let var_acc = 0;
    for (const v of stats) var_acc += (v - mean) ** 2;
    const variance = m > 1 ? var_acc / (m - 1) : 0;
    const stdev = Math.sqrt(Math.max(0, variance));
    const lo_idx = Math.floor(m * 0.025);
    const hi_idx = Math.min(m - 1, Math.ceil(m * 0.975));
    return {
        original_statistic:  original,
        bootstrap_mean:      mean,
        bootstrap_stdev:     stdev,
        ci_lower_2_5_pct:    stats[lo_idx],
        ci_upper_97_5_pct:   stats[hi_idx],
        n_resamples:         m,
        block_size,
    };
}

export function computeStatistic(data, stat) {
    if (!data || data.length === 0) return null;
    const n = data.length;
    let sum = 0;
    for (const v of data) sum += v;
    const mean = sum / n;
    if (stat === 'mean') return mean;
    if (stat === 'stdev' || stat === 'sharpe_ratio') {
        if (n < 2) return null;
        let v_acc = 0;
        for (const v of data) v_acc += (v - mean) ** 2;
        const variance = v_acc / (n - 1);
        const sd = Math.sqrt(Math.max(0, variance));
        if (stat === 'stdev') return sd;
        return sd > 0 ? mean / sd : null;
    }
    if (stat === 'max_drawdown') {
        let equity = 0, peak = 0, max_dd = 0;
        for (const r of data) {
            equity += r;
            if (equity > peak) peak = equity;
            const dd = peak - equity;
            if (dd > max_dd) max_dd = dd;
        }
        return max_dd;
    }
    return null;
}

// Parse whitespace/comma-separated number list (returns, P&L increments).
export function parseDataBlob(blob) {
    const out = { data: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: 'input must be a string' });
        return out;
    }
    const tokens = blob
        .split('\n')
        .map(l => l.split('#')[0])
        .join(' ')
        .split(/[\s,]+/)
        .filter(t => t.length > 0);
    for (let i = 0; i < tokens.length; i++) {
        const raw = tokens[i];
        let tok = raw;
        let neg = false;
        if (tok.startsWith('(') && tok.endsWith(')')) {
            neg = true;
            tok = tok.slice(1, -1);
        }
        tok = tok.replace(/[\$%,]/g, '');
        const v = Number(tok);
        if (!Number.isFinite(v)) {
            out.errors.push({ line_no: i + 1, message: `token "${raw}" not finite` });
            continue;
        }
        out.data.push(neg ? -v : v);
    }
    return out;
}

export function dataToBlob(data) {
    return data.join('\n');
}

// CI-width vs |original| ratio — narrower means more certain.
export function ciBadge(report) {
    if (!report) return { key: 'view.block_boot.ci.unknown', cls: '' };
    const width = report.ci_upper_97_5_pct - report.ci_lower_2_5_pct;
    const orig = Math.abs(report.original_statistic);
    if (!Number.isFinite(width) || orig === 0) return { key: 'view.block_boot.ci.unknown', cls: '' };
    const ratio = width / orig;
    if (ratio < 0.5) return { key: 'view.block_boot.ci.tight',    cls: 'pos' };
    if (ratio < 2)   return { key: 'view.block_boot.ci.moderate', cls: '' };
    if (ratio < 10)  return { key: 'view.block_boot.ci.wide',     cls: 'neg' };
    return { key: 'view.block_boot.ci.extreme', cls: 'neg' };
}

// Bias verdict: bootstrap_mean vs original_statistic.
export function biasBadge(report) {
    if (!report) return { key: 'view.block_boot.bias.unknown', cls: '' };
    const diff = report.bootstrap_mean - report.original_statistic;
    const orig = Math.abs(report.original_statistic);
    if (orig === 0 || !Number.isFinite(diff)) return { key: 'view.block_boot.bias.unknown', cls: '' };
    const rel = Math.abs(diff) / orig;
    if (rel < 0.05) return { key: 'view.block_boot.bias.negligible', cls: 'pos' };
    if (rel < 0.20) return { key: 'view.block_boot.bias.small',      cls: '' };
    if (rel < 0.50) return { key: 'view.block_boot.bias.notable',    cls: 'neg' };
    return { key: 'view.block_boot.bias.large', cls: 'neg' };
}

// Significance verdict: does the 95% CI exclude zero?
export function signifBadge(report) {
    if (!report) return { key: 'view.block_boot.sig.unknown', cls: '' };
    const lo = report.ci_lower_2_5_pct;
    const hi = report.ci_upper_97_5_pct;
    if (!Number.isFinite(lo) || !Number.isFinite(hi)) return { key: 'view.block_boot.sig.unknown', cls: '' };
    if (lo > 0)        return { key: 'view.block_boot.sig.positive', cls: 'pos' };
    if (hi < 0)        return { key: 'view.block_boot.sig.negative', cls: 'neg' };
    return { key: 'view.block_boot.sig.spans_zero', cls: '' };
}

export function summarizeData(data) {
    if (!Array.isArray(data) || data.length === 0) {
        return { count: 0, mean: NaN, sum: NaN, max: NaN, min: NaN };
    }
    let sum = 0, mx = -Infinity, mn = Infinity;
    for (const v of data) {
        sum += v;
        if (v > mx) mx = v;
        if (v < mn) mn = v;
    }
    return {
        count: data.length,
        sum,
        mean: sum / data.length,
        max: Number.isFinite(mx) ? mx : NaN,
        min: Number.isFinite(mn) ? mn : NaN,
    };
}

function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF;
    };
}

export function makeDemoInput(kind = 'mean-revert') {
    switch (kind) {
        case 'mean-revert': {
            // AR(1) negative coefficient — strong serial dependence.
            const rand = lcg(42n);
            const data = [];
            let prev = 0;
            for (let i = 0; i < 500; i++) {
                const eps = (rand() - 0.5) * 0.04;
                const v = -0.6 * prev + eps;
                data.push(v);
                prev = v;
            }
            return { data, block_size: 20, n_resamples: 1000, statistic: 'mean', seed: 42n };
        }
        case 'momentum': {
            // AR(1) positive coefficient — trending.
            const rand = lcg(7n);
            const data = [];
            let prev = 0;
            for (let i = 0; i < 500; i++) {
                const eps = (rand() - 0.5) * 0.04;
                const v = 0.5 * prev + 0.001 + eps;
                data.push(v);
                prev = v;
            }
            return { data, block_size: 20, n_resamples: 1000, statistic: 'mean', seed: 42n };
        }
        case 'volatility-cluster': {
            // GARCH-ish: volatility shocks persist.
            const rand = lcg(99n);
            const data = [];
            let sigma = 0.01;
            for (let i = 0; i < 500; i++) {
                const eps = (rand() - 0.5) * 2;
                const ret = sigma * eps;
                data.push(ret);
                sigma = 0.7 * sigma + 0.3 * Math.abs(ret) + 0.001;
            }
            return { data, block_size: 25, n_resamples: 1000, statistic: 'stdev', seed: 99n };
        }
        case 'sharpe-strategy': {
            // Strategy with small positive edge, mild noise — Sharpe CI test.
            const rand = lcg(11n);
            const data = [];
            for (let i = 0; i < 500; i++) data.push((rand() - 0.45) * 0.03);
            return { data, block_size: 15, n_resamples: 1500, statistic: 'sharpe_ratio', seed: 11n };
        }
        case 'drawdown-tail': {
            // Equity returns with occasional large drawdowns.
            const rand = lcg(13n);
            const data = [];
            for (let i = 0; i < 500; i++) {
                const u = rand();
                data.push(u < 0.02 ? -0.10 + (rand() - 0.5) * 0.02 : (rand() - 0.48) * 0.02);
            }
            return { data, block_size: 30, n_resamples: 500, statistic: 'max_drawdown', seed: 13n };
        }
        case 'iid-noise': {
            const rand = lcg(21n);
            const data = [];
            for (let i = 0; i < 500; i++) data.push((rand() - 0.5) * 0.02);
            return { data, block_size: 10, n_resamples: 1000, statistic: 'mean', seed: 21n };
        }
        case 'small-sample': {
            // Just above min size, big block, fewer resamples.
            const rand = lcg(33n);
            const data = [];
            for (let i = 0; i < 50; i++) data.push((rand() - 0.5) * 0.05);
            return { data, block_size: 10, n_resamples: 500, statistic: 'mean', seed: 33n };
        }
        case 'fat-tail': {
            // Cauchy-ish: tail-heavy returns with serial dependence.
            const rand = lcg(57n);
            const data = [];
            for (let i = 0; i < 500; i++) {
                const u = rand();
                const v = Math.tan(Math.PI * (u - 0.5)) * 0.005;
                const clipped = Math.max(-0.30, Math.min(0.30, v));
                data.push(clipped);
            }
            return { data, block_size: 20, n_resamples: 800, statistic: 'stdev', seed: 57n };
        }
        default: return makeDemoInput('mean-revert');
    }
}

export function fmtNum(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtNumSigned(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toFixed(d);
}

export function fmtPct(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
