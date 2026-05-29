// Bipower Variation (Barndorff-Nielsen & Shephard 2004) helpers.
//
// Backend body: { returns: number[] }
// Returns: { realized_variance, bipower_variation, jump_variation,
//   tripower_quarticity, jump_test_z, jump_test_p_value, n_observations } | null
//
// BPV = (π/2) · Σ|r_i|·|r_{i−1}|   (jump-robust IV estimator)
// jump_variation = max(0, RV − BPV)
// Huang-Tauchen z = √n·(RV−BPV)/BPV/√(θ·max(1, TQ/BPV²))

export const MU1   = 0.7978845608028654;                       // √(2/π)
export const THETA = Math.PI * Math.PI / 4 + Math.PI - 5;
const GAMMA_7_6 = 0.9275537932833882;
const GAMMA_1_2 = Math.sqrt(Math.PI);

export const DEFAULT_INPUTS = {
    returns: [],
};

export function validateInputs(input) {
    if (!Array.isArray(input.returns))                       return 'returns must be an array';
    if (input.returns.length < 4)                            return 'need at least 4 returns';
    for (let i = 0; i < input.returns.length; i++) {
        if (!Number.isFinite(input.returns[i]))              return `returns[${i}] not finite`;
    }
    return null;
}

export function buildBody(input) {
    return { returns: input.returns };
}

// Pure-JS mirror of crates/traderview-core/src/bipower_variation.rs::compute.
export function localCompute(returns) {
    if (returns.length < 4) return null;
    for (const v of returns) if (!Number.isFinite(v)) return null;
    const n = returns.length;
    const n_f = n;
    let rv = 0;
    for (const r of returns) rv += r * r;
    let bpv_sum = 0;
    for (let i = 1; i < n; i++) bpv_sum += Math.abs(returns[i]) * Math.abs(returns[i - 1]);
    const bpv = (1 / (MU1 * MU1)) * bpv_sum;
    const jump = Math.max(0, rv - bpv);
    const mu43 = Math.pow(2, 2 / 3) * GAMMA_7_6 / GAMMA_1_2;
    const mu43_cubed_inv = 1 / Math.pow(mu43, 3);
    let tq_sum = 0;
    for (let i = 2; i < n; i++) {
        tq_sum += Math.pow(Math.abs(returns[i]),       4 / 3)
                * Math.pow(Math.abs(returns[i - 1]),   4 / 3)
                * Math.pow(Math.abs(returns[i - 2]),   4 / 3);
    }
    const tq = n_f * mu43_cubed_inv * tq_sum;
    const scale_denom = Math.sqrt(Math.max(0, THETA * Math.max(1, tq / (bpv * bpv))));
    const z = (bpv > 0 && scale_denom > 0)
        ? Math.sqrt(n_f) * (rv - bpv) / bpv / scale_denom
        : 0;
    const p_value = 1 - standardNormalCdf(z);
    return {
        realized_variance:    rv,
        bipower_variation:    bpv,
        jump_variation:       jump,
        tripower_quarticity:  tq,
        jump_test_z:          z,
        jump_test_p_value:    p_value,
        n_observations:       n,
    };
}

// Abramowitz & Stegun 7.1.26 series approximation (matches Rust impl).
function standardNormalCdf(z) {
    return 0.5 * (1 + erf(z / Math.SQRT2));
}
function erf(x) {
    const sign = x < 0 ? -1 : 1;
    x = Math.abs(x);
    const t = 1 / (1 + 0.3275911 * x);
    const y = 1 - (((((1.061405429 * t - 1.453152027) * t)
        + 1.421413741) * t - 0.284496736) * t + 0.254829592) * t * Math.exp(-x * x);
    return sign * y;
}

// Parse comma/whitespace-separated returns; blanks + # comments ignored.
// Accepts decimal (0.012) or pct-suffix (1.2%).
export function parseReturnsBlob(blob) {
    const out = { returns: [], errors: [] };
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
        const tok = tokens[i];
        const isPct = tok.endsWith('%');
        const stripped = isPct ? tok.slice(0, -1) : tok;
        const v = Number(stripped);
        if (!Number.isFinite(v)) {
            out.errors.push({ line_no: i + 1, message: `token "${tok}" not finite` });
            continue;
        }
        out.returns.push(isPct ? v / 100 : v);
    }
    return out;
}

export function returnsToBlob(returns) {
    return returns.join('\n');
}

// Verdict on jump test p-value.
export function jumpBadge(p) {
    if (p == null || !Number.isFinite(p)) return { key: 'view.bpv.badge.unknown', cls: '' };
    if (p < 0.001) return { key: 'view.bpv.badge.strong_jumps',   cls: 'neg' };
    if (p < 0.01)  return { key: 'view.bpv.badge.significant',    cls: 'neg' };
    if (p < 0.05)  return { key: 'view.bpv.badge.weak',           cls: '' };
    if (p < 0.10)  return { key: 'view.bpv.badge.marginal',       cls: '' };
    return { key: 'view.bpv.badge.no_jumps', cls: 'pos' };
}

// Jump-to-RV fraction badge.
export function jumpFractionBadge(rv, jump) {
    if (!Number.isFinite(rv) || rv <= 0) return { key: 'view.bpv.frac.unknown', cls: '' };
    const f = jump / rv;
    if (f >= 0.5)  return { key: 'view.bpv.frac.dominant', cls: 'neg' };
    if (f >= 0.2)  return { key: 'view.bpv.frac.substantial', cls: 'neg' };
    if (f >= 0.05) return { key: 'view.bpv.frac.moderate', cls: '' };
    if (f > 0)     return { key: 'view.bpv.frac.minor', cls: 'pos' };
    return { key: 'view.bpv.frac.none', cls: 'pos' };
}

// Compute Q3 (third quartile) of returns abs values — quick summary.
export function jumpRatio(rv, jump) {
    if (!Number.isFinite(rv) || rv <= 0) return 0;
    return jump / rv;
}

// LCG for stable demo generation.
function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF;
    };
}

function gaussianPair(rand) {
    const u1 = Math.max(1e-12, rand());
    const u2 = rand();
    return Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * Math.PI * u2);
}

export function makeDemoInput(kind = 'no-jumps') {
    switch (kind) {
        case 'no-jumps': {
            // Smooth Gaussian-ish returns; BPV should track RV.
            const rand = lcg(12345n);
            const r = [];
            for (let i = 0; i < 500; i++) r.push(gaussianPair(rand) * 0.01);
            return { returns: r };
        }
        case 'single-big-jump': {
            // 200 small returns with one 50% jump.
            const r = new Array(200).fill(0.001);
            r[100] = 0.50;
            return { returns: r };
        }
        case 'multi-small-jumps': {
            const r = new Array(300).fill(0.001);
            r[50] = 0.05;
            r[120] = -0.04;
            r[200] = 0.06;
            return { returns: r };
        }
        case 'flat-zero': {
            return { returns: new Array(100).fill(0) };
        }
        case 'high-vol-no-jumps': {
            const rand = lcg(42n);
            const r = [];
            for (let i = 0; i < 500; i++) r.push(gaussianPair(rand) * 0.03);
            return { returns: r };
        }
        case 'crash-down': {
            const r = new Array(150).fill(0.001);
            r[75] = -0.30;
            return { returns: r };
        }
        case 'short-series': {
            // Just over the minimum length.
            return { returns: [0.01, -0.02, 0.005, 0.012, -0.008] };
        }
        case 'persistent-vol': {
            // AR(1)-like volatility process.
            const rand = lcg(77n);
            let sigma = 0.01;
            const r = [];
            for (let i = 0; i < 400; i++) {
                sigma = 0.95 * sigma + 0.05 * (rand() * 0.02);
                r.push(gaussianPair(rand) * sigma);
            }
            return { returns: r };
        }
        default: return makeDemoInput('no-jumps');
    }
}

export function fmtVar(v, d = 6) {
    if (v == null || !Number.isFinite(v)) return '—';
    if (Math.abs(v) >= 1e-3) return v.toFixed(d);
    return v.toExponential(3);
}

export function fmtZ(v, d = 3) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toFixed(d);
}

export function fmtP(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    if (v < 1e-4) return v.toExponential(2);
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
