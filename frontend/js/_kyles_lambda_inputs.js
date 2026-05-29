// Kyle's Lambda — rolling price-impact slope estimator helpers.
//
// Backend body: { price_changes: number[], signed_volumes: number[],
//                 window: number }.
// Returns: (number | null)[] — length = price_changes.length,
// warmup nulls for the first (window-1) bars.
//
// Model: Δp = λ · signed_volume + ε, closed-form OLS (no intercept):
//   λ = Σ(x · y) / Σ(x²)
//
// Interpretation: λ is price impact per unit of signed flow.
// LOW λ = deep, liquid book. HIGH λ = thin book, large moves on small flow.

export const DEFAULT_WINDOW = 30;

export const DEFAULT_INPUTS = {
    price_changes: [],
    signed_volumes: [],
    window: DEFAULT_WINDOW,
};

export function validateInputs(input) {
    if (!Array.isArray(input.price_changes))   return 'price_changes must be an array';
    if (!Array.isArray(input.signed_volumes))  return 'signed_volumes must be an array';
    if (!Number.isFinite(input.window))        return 'window must be finite';
    if (!Number.isInteger(input.window))       return 'window must be an integer';
    if (input.window < 2)                      return 'window must be ≥ 2';
    if (input.price_changes.length !== input.signed_volumes.length)
        return 'price_changes and signed_volumes must have the same length';
    return null;
}

export function buildBody(input) {
    return {
        price_changes:  input.price_changes,
        signed_volumes: input.signed_volumes,
        window:         input.window,
    };
}

// Pure-JS mirror of crates/traderview-core/src/kyles_lambda.rs::compute.
// Same warmup-nulls + sxx>0 guard + NaN-pair skip + valid<2 short-circuit.
export function localCompute(price_changes, signed_volumes, window) {
    const n = price_changes.length;
    const out = new Array(n).fill(null);
    if (window < 2 || price_changes.length !== signed_volumes.length || n < window) return out;
    for (let i = window - 1; i < n; i++) {
        const lo = i + 1 - window;
        let sxy = 0, sxx = 0, valid = 0;
        for (let j = lo; j <= i; j++) {
            const x = signed_volumes[j];
            const y = price_changes[j];
            if (!Number.isFinite(x) || !Number.isFinite(y)) continue;
            sxy += x * y;
            sxx += x * x;
            valid++;
        }
        if (valid < 2) continue;
        if (sxx > 0) {
            const lam = sxy / sxx;
            if (Number.isFinite(lam)) out[i] = lam;
        }
    }
    return out;
}

// Parse "price_change signed_volume" per line; comments + blank lines skipped.
export function parseFlowBlob(blob) {
    const out = { price_changes: [], signed_volumes: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: 'input must be a string' });
        return out;
    }
    const lines = blob.split('\n');
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i].trim();
        if (!raw || raw.startsWith('#')) continue;
        const toks = raw.split(/\s+/);
        if (toks.length !== 2) {
            out.errors.push({ line_no: i + 1, message: 'expected 2 tokens (price_change signed_volume)' });
            continue;
        }
        const pc = Number(toks[0]);
        const sv = Number(toks[1]);
        if (!Number.isFinite(pc)) {
            out.errors.push({ line_no: i + 1, message: 'price_change not finite' });
            continue;
        }
        if (!Number.isFinite(sv)) {
            out.errors.push({ line_no: i + 1, message: 'signed_volume not finite' });
            continue;
        }
        out.price_changes.push(pc);
        out.signed_volumes.push(sv);
    }
    return out;
}

// Aggregate stats over the non-null portion of a series.
export function summarize(series) {
    const valid = series.filter(v => v != null && Number.isFinite(v));
    if (valid.length === 0) return { count: 0, mean: NaN, min: NaN, max: NaN, last: NaN };
    let sum = 0, mn = Infinity, mx = -Infinity;
    for (const v of valid) { sum += v; if (v < mn) mn = v; if (v > mx) mx = v; }
    return { count: valid.length, mean: sum / valid.length, min: mn, max: mx, last: valid[valid.length - 1] };
}

// 4-tier liquidity badge from |λ| — calibrated against typical equity flow
// (signed volumes in shares, price changes in dollars).
// thresholds in absolute log scale: λ < 1e-5 deep | < 1e-4 normal | < 1e-3 thin | ≥ 1e-3 illiquid
export function liquidityBadge(lambda) {
    if (lambda == null || !Number.isFinite(lambda)) return { key: 'view.kyles_lambda.badge.unknown', cls: '' };
    const a = Math.abs(lambda);
    if (a < 1e-5) return { key: 'view.kyles_lambda.badge.deep',     cls: 'pos' };
    if (a < 1e-4) return { key: 'view.kyles_lambda.badge.normal',   cls: '' };
    if (a < 1e-3) return { key: 'view.kyles_lambda.badge.thin',     cls: 'neg' };
    return { key: 'view.kyles_lambda.badge.illiquid', cls: 'neg' };
}

// Sign badge — positive λ = momentum (flow follows price); negative λ = mean-reverting.
export function signBadge(lambda) {
    if (lambda == null || !Number.isFinite(lambda)) return { key: 'view.kyles_lambda.sign.unknown', cls: '' };
    if (lambda > 0)  return { key: 'view.kyles_lambda.sign.momentum',  cls: '' };
    if (lambda < 0)  return { key: 'view.kyles_lambda.sign.reversion', cls: '' };
    return { key: 'view.kyles_lambda.sign.flat', cls: '' };
}

// Deterministic synthetic demos.
export function makeDemoInput(kind = 'deep-mm') {
    switch (kind) {
        case 'deep-mm':           return synth({ n: 200, lambda: 5e-6,  noise: 1e-4,  window: 30 });
        case 'normal-mid-cap':    return synth({ n: 200, lambda: 5e-5,  noise: 5e-4,  window: 30 });
        case 'thin-small-cap':    return synth({ n: 200, lambda: 5e-4,  noise: 1e-3,  window: 30 });
        case 'illiquid-penny':    return synth({ n: 200, lambda: 5e-3,  noise: 5e-3,  window: 30 });
        case 'reversion':         return synth({ n: 200, lambda: -3e-4, noise: 1e-3,  window: 30 });
        case 'regime-shift':      return regimeShift();
        case 'zero-flow':         return { price_changes: Array(60).fill(0.01),
                                           signed_volumes: Array(60).fill(0), window: 30 };
        case 'nan-spotty':        return spotty();
        default:                  return makeDemoInput('normal-mid-cap');
    }
}

// Deterministic sine-noise to keep tests stable.
function synth({ n, lambda, noise, window }) {
    const price_changes = [];
    const signed_volumes = [];
    for (let i = 0; i < n; i++) {
        const v = (i % 21) * 1000 - 10_000;
        const eps = Math.sin(i * 0.31) * noise;
        price_changes.push(lambda * v + eps);
        signed_volumes.push(v);
    }
    return { price_changes, signed_volumes, window };
}

function regimeShift() {
    // First 100 bars: deep (λ=5e-6); next 100 bars: thin (λ=5e-4).
    const a = synth({ n: 100, lambda: 5e-6, noise: 1e-4, window: 30 });
    const b = synth({ n: 100, lambda: 5e-4, noise: 1e-3, window: 30 });
    return {
        price_changes:  [...a.price_changes,  ...b.price_changes],
        signed_volumes: [...a.signed_volumes, ...b.signed_volumes],
        window: 30,
    };
}

function spotty() {
    const base = synth({ n: 100, lambda: 1e-4, noise: 5e-4, window: 30 });
    base.price_changes[10]  = NaN;
    base.signed_volumes[40] = NaN;
    return base;
}

export function fmtLambda(v, d = 7) {
    if (v == null || !Number.isFinite(v)) return '—';
    if (Math.abs(v) >= 1e-3) return v.toFixed(6);
    return v.toExponential(d - 4);
}

export function fmtSci(v) {
    if (!Number.isFinite(v)) return '—';
    if (Math.abs(v) >= 1e-3 || v === 0) return v.toFixed(6);
    return v.toExponential(3);
}

export function fmtInt(v) {
    if (!Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
