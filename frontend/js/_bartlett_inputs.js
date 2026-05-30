// Bartlett's Test for Equality of Variances (1937) helpers.
//
// Backend body: { groups: number[][] }
// Returns: {
//   chi_squared_statistic, degrees_of_freedom, p_value, pooled_variance,
//   n_groups, n_total, reject_at_5pct
// } | null

import { t } from './i18n.js';

export const MIN_GROUPS = 2;
export const MIN_PER_GROUP = 2;

export const DEFAULT_INPUTS = {
    groups: [],
};

export function validateInputs(input) {
    if (!Array.isArray(input.groups))                       return t('view.bartlett.validate.groups_array');
    if (input.groups.length < MIN_GROUPS)                   return t('view.bartlett.validate.groups_min', { n: MIN_GROUPS });
    let total = 0;
    for (let i = 0; i < input.groups.length; i++) {
        const g = input.groups[i];
        if (!Array.isArray(g))                              return t('view.bartlett.validate.group_array', { i });
        if (g.length < MIN_PER_GROUP)                       return t('view.bartlett.validate.group_obs_min', { i, n: MIN_PER_GROUP });
        for (let j = 0; j < g.length; j++) {
            if (!Number.isFinite(g[j]))                      return t('view.bartlett.validate.value_not_finite', { i, j });
        }
        total += g.length;
    }
    if (total <= input.groups.length)                       return t('view.bartlett.validate.total_obs_min');
    return null;
}

export function buildBody(input) {
    return { groups: input.groups.map(g => g.slice()) };
}

// Pure-JS mirror of crates/traderview-core/src/bartlett_variance_test.rs::test.
export function localTest(groups) {
    const k = groups.length;
    if (k < 2) return null;
    for (const g of groups) {
        if (g.length < 2) return null;
        for (const v of g) if (!Number.isFinite(v)) return null;
    }
    let n_total = 0;
    for (const g of groups) n_total += g.length;
    if (n_total <= k) return null;
    const variances = [];
    for (const g of groups) {
        const n_g = g.length;
        let sum = 0;
        for (const v of g) sum += v;
        const mean = sum / n_g;
        let var_acc = 0;
        for (const v of g) var_acc += (v - mean) ** 2;
        const variance = var_acc / (n_g - 1);
        if (variance <= 0) return null;
        variances.push(variance);
    }
    const n_total_f = n_total;
    const k_f = k;
    let pooled_num = 0;
    for (let i = 0; i < k; i++) {
        pooled_num += (groups[i].length - 1) * variances[i];
    }
    const pooled = pooled_num / (n_total_f - k_f);
    if (pooled <= 0) return null;
    let log_sum = 0;
    for (let i = 0; i < k; i++) {
        log_sum += (groups[i].length - 1) * Math.log(variances[i]);
    }
    const numerator = (n_total_f - k_f) * Math.log(pooled) - log_sum;
    let inv_sum = 0;
    for (const g of groups) inv_sum += 1 / (g.length - 1);
    const correction = 1 + (1 / (3 * (k_f - 1))) * (inv_sum - 1 / (n_total_f - k_f));
    const chi_sq = numerator / correction;
    const dof = k_f - 1;
    const p_value = chiSquaredUpperTail(chi_sq, dof);
    const crit_5pct = chiSquared5pctCritical(k - 1);
    return {
        chi_squared_statistic: chi_sq,
        degrees_of_freedom: dof,
        p_value,
        pooled_variance: pooled,
        n_groups: k,
        n_total,
        reject_at_5pct: chi_sq > crit_5pct,
    };
}

export function chiSquaredUpperTail(x, k) {
    if (x <= 0 || k <= 0) return 1;
    const z = (Math.pow(x / k, 1/3) - (1 - 2 / (9 * k))) / Math.sqrt(2 / (9 * k));
    return 1 - standardNormalCdf(z);
}

export function chiSquared5pctCritical(k) {
    const tbl = { 1: 3.841, 2: 5.991, 3: 7.815, 4: 9.488, 5: 11.070 };
    if (tbl[k] != null) return tbl[k];
    return k + 2 * Math.sqrt(2 * k);
}

export function standardNormalCdf(z) {
    return 0.5 * (1 + erf(z / Math.SQRT2));
}

export function erf(x) {
    const sign = x < 0 ? -1 : 1;
    x = Math.abs(x);
    const t = 1 / (1 + 0.327_591_1 * x);
    const y = 1 - (((((1.061_405_429 * t - 1.453_152_027) * t)
        + 1.421_413_741) * t - 0.284_496_736) * t + 0.254_829_592) * t * Math.exp(-x * x);
    return sign * y;
}

// Parse blob: lines like "LABEL v1 v2 v3 ..." → 1 group per line; LABEL kept for display.
export function parseGroupsBlob(blob) {
    const out = { groups: [], labels: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: t('common.parse.input_must_be_string') });
        return out;
    }
    const lines = blob.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.split('#')[0].trim();
        if (!s) continue;
        const parts = s.split(/[\s,]+/).filter(Boolean);
        if (parts.length < 3) {
            out.errors.push({ line_no: i + 1, message: `expected LABEL plus ≥ 2 obs, got ${parts.length} token(s)` });
            continue;
        }
        const label = parts[0];
        const obs = [];
        let bad = false;
        for (let j = 1; j < parts.length; j++) {
            const v = Number(parts[j].replace(/[\$%]/g, ''));
            if (!Number.isFinite(v)) {
                out.errors.push({ line_no: i + 1, message: `token "${parts[j]}" not finite` });
                bad = true;
                break;
            }
            obs.push(v);
        }
        if (!bad) {
            out.labels.push(label);
            out.groups.push(obs);
        }
    }
    return out;
}

export function groupsToBlob(groups, labels) {
    if (!labels || labels.length !== groups.length) {
        labels = groups.map((_, i) => `G${i + 1}`);
    }
    return groups.map((g, i) => `${labels[i]} ${g.join(' ')}`).join('\n');
}

// Verdict on χ² + p-value tiers.
export function verdictBadge(report) {
    if (!report) return { key: 'view.bartlett.verdict.unknown', cls: '' };
    const p = report.p_value;
    if (!Number.isFinite(p)) return { key: 'view.bartlett.verdict.unknown', cls: '' };
    if (p < 0.01)  return { key: 'view.bartlett.verdict.strong_reject', cls: 'neg' };
    if (p < 0.05)  return { key: 'view.bartlett.verdict.reject',        cls: 'neg' };
    if (p < 0.10)  return { key: 'view.bartlett.verdict.borderline',    cls: '' };
    return { key: 'view.bartlett.verdict.equal_variance', cls: 'pos' };
}

// Variance ratio verdict — max/min group variance.
export function ratioBadge(groups) {
    if (!Array.isArray(groups) || groups.length < 2) {
        return { key: 'view.bartlett.ratio.unknown', cls: '' };
    }
    const vars_ = [];
    for (const g of groups) {
        const n_g = g.length;
        if (n_g < 2) return { key: 'view.bartlett.ratio.unknown', cls: '' };
        let sum = 0;
        for (const v of g) sum += v;
        const mean = sum / n_g;
        let var_acc = 0;
        for (const v of g) var_acc += (v - mean) ** 2;
        const variance = var_acc / (n_g - 1);
        if (!Number.isFinite(variance) || variance <= 0) return { key: 'view.bartlett.ratio.unknown', cls: '' };
        vars_.push(variance);
    }
    const mx = Math.max(...vars_);
    const mn = Math.min(...vars_);
    const ratio = mx / mn;
    if (ratio > 10)  return { key: 'view.bartlett.ratio.severe',  cls: 'neg' };
    if (ratio > 4)   return { key: 'view.bartlett.ratio.large',   cls: 'neg' };
    if (ratio > 2)   return { key: 'view.bartlett.ratio.moderate', cls: '' };
    if (ratio > 1.2) return { key: 'view.bartlett.ratio.mild',    cls: '' };
    return { key: 'view.bartlett.ratio.tiny', cls: 'pos' };
}

// Per-group descriptive stats for the table.
export function groupStats(groups, labels) {
    if (!Array.isArray(groups)) return [];
    return groups.map((g, i) => {
        if (!Array.isArray(g) || g.length === 0) {
            return { label: labels?.[i] || `G${i + 1}`, n: 0, mean: NaN, variance: NaN, sd: NaN, min: NaN, max: NaN };
        }
        const n_g = g.length;
        let sum = 0, mx = -Infinity, mn = Infinity;
        for (const v of g) { sum += v; if (v > mx) mx = v; if (v < mn) mn = v; }
        const mean = sum / n_g;
        let var_acc = 0;
        for (const v of g) var_acc += (v - mean) ** 2;
        const variance = n_g > 1 ? var_acc / (n_g - 1) : NaN;
        const sd = Number.isFinite(variance) ? Math.sqrt(Math.max(0, variance)) : NaN;
        return {
            label: labels?.[i] || `G${i + 1}`,
            n: n_g, mean, variance, sd,
            min: Number.isFinite(mn) ? mn : NaN,
            max: Number.isFinite(mx) ? mx : NaN,
        };
    });
}

function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF;
    };
}

function gaussian(rand) {
    const u1 = Math.max(1e-12, rand());
    const u2 = rand();
    return Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
}

export function makeDemoInput(kind = 'equal') {
    switch (kind) {
        case 'equal': {
            const r1 = lcg(42n), r2 = lcg(7n);
            return {
                groups: [
                    Array.from({ length: 100 }, () => gaussian(r1)),
                    Array.from({ length: 100 }, () => gaussian(r2)),
                ],
                labels: ['A', 'B'],
            };
        }
        case 'mild-diff': {
            // 1.5x ratio.
            const r1 = lcg(11n), r2 = lcg(13n);
            return {
                groups: [
                    Array.from({ length: 100 }, () => gaussian(r1)),
                    Array.from({ length: 100 }, () => gaussian(r2) * 1.5),
                ],
                labels: ['Tight', 'Loose'],
            };
        }
        case 'strong-diff': {
            // 5x ratio.
            const r1 = lcg(21n), r2 = lcg(33n);
            return {
                groups: [
                    Array.from({ length: 100 }, () => gaussian(r1)),
                    Array.from({ length: 100 }, () => gaussian(r2) * 5.0),
                ],
                labels: ['Calm', 'Wild'],
            };
        }
        case 'three-equal': {
            const r1 = lcg(1n), r2 = lcg(2n), r3 = lcg(3n);
            return {
                groups: [
                    Array.from({ length: 80 }, () => gaussian(r1)),
                    Array.from({ length: 80 }, () => gaussian(r2)),
                    Array.from({ length: 80 }, () => gaussian(r3)),
                ],
                labels: ['G1', 'G2', 'G3'],
            };
        }
        case 'three-mixed': {
            // Mix of variances across 3 groups.
            const r1 = lcg(4n), r2 = lcg(5n), r3 = lcg(6n);
            return {
                groups: [
                    Array.from({ length: 80 }, () => gaussian(r1)),
                    Array.from({ length: 80 }, () => gaussian(r2) * 2.0),
                    Array.from({ length: 80 }, () => gaussian(r3) * 4.0),
                ],
                labels: ['Low', 'Med', 'High'],
            };
        }
        case 'four-volregime': {
            const r1 = lcg(57n), r2 = lcg(58n), r3 = lcg(59n), r4 = lcg(60n);
            return {
                groups: [
                    Array.from({ length: 60 }, () => gaussian(r1) * 1.0),
                    Array.from({ length: 60 }, () => gaussian(r2) * 1.5),
                    Array.from({ length: 60 }, () => gaussian(r3) * 0.7),
                    Array.from({ length: 60 }, () => gaussian(r4) * 3.0),
                ],
                labels: ['Q1', 'Q2', 'Q3', 'Q4'],
            };
        }
        case 'small-groups': {
            // Just above MIN_PER_GROUP=2.
            const r1 = lcg(99n);
            const g = (n) => Array.from({ length: n }, () => gaussian(r1));
            return { groups: [g(5), g(5), g(5)], labels: ['A', 'B', 'C'] };
        }
        case 'asymmetric-sizes': {
            const r1 = lcg(77n), r2 = lcg(88n);
            return {
                groups: [
                    Array.from({ length: 200 }, () => gaussian(r1)),
                    Array.from({ length: 30 },  () => gaussian(r2)),
                ],
                labels: ['Large', 'Small'],
            };
        }
        default: return makeDemoInput('equal');
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

export function fmtPVal(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    if (v < 0.0001) return '< 0.0001';
    return v.toFixed(4);
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
