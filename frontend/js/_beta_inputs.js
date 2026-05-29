// Single-asset beta estimator helpers.
//
// Backend body: { asset: number[], benchmark: number[] }
// Returns: { beta, alpha, r_squared, correlation, n } | null
//
// beta = cov(asset, bench) / var(bench)
// alpha = mean(asset) − beta · mean(bench)
// r²    = correlation²
//
// Caller pre-aligns the two return series. NaN / non-finite inputs are
// rejected; zero-variance benchmark → null.

export const DEFAULT_INPUTS = {
    asset: [],
    benchmark: [],
};

export function validateInputs(input) {
    if (!Array.isArray(input.asset))                          return 'asset must be an array';
    if (!Array.isArray(input.benchmark))                      return 'benchmark must be an array';
    if (input.asset.length !== input.benchmark.length)        return 'asset and benchmark must have equal length';
    if (input.asset.length < 2)                                return 'need at least 2 paired observations';
    for (let i = 0; i < input.asset.length; i++) {
        if (!Number.isFinite(input.asset[i]))                 return `asset[${i}] not finite`;
        if (!Number.isFinite(input.benchmark[i]))             return `benchmark[${i}] not finite`;
    }
    return null;
}

export function buildBody(input) {
    return { asset: input.asset, benchmark: input.benchmark };
}

// Pure-JS mirror of crates/traderview-core/src/beta.rs::estimate.
export function localEstimate(asset, benchmark) {
    if (!Array.isArray(asset) || !Array.isArray(benchmark)) return null;
    if (asset.length !== benchmark.length || asset.length < 2) return null;
    const n = asset.length;
    let sumA = 0, sumB = 0;
    for (let i = 0; i < n; i++) { sumA += asset[i]; sumB += benchmark[i]; }
    const meanA = sumA / n, meanB = sumB / n;
    let cov = 0, varA = 0, varB = 0;
    for (let i = 0; i < n; i++) {
        const da = asset[i] - meanA;
        const db = benchmark[i] - meanB;
        cov += da * db;
        varB += db * db;
        varA += da * da;
    }
    if (varB === 0) return null;
    const beta = cov / varB;
    const alpha = meanA - beta * meanB;
    const correlation = (varA > 0 && varB > 0) ? cov / (Math.sqrt(varA) * Math.sqrt(varB)) : 0;
    const r_squared = correlation * correlation;
    return { beta, alpha, r_squared, correlation, n };
}

// Parse "asset benchmark" per line; comments + blanks ignored.
// Pct-suffix tokens ("1.2%") are converted to decimal.
export function parsePairsBlob(blob) {
    const out = { asset: [], benchmark: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: 'input must be a string' });
        return out;
    }
    const lines = blob.split('\n');
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i].split('#')[0].trim();
        if (!raw) continue;
        const toks = raw.split(/[\s,]+/).filter(t => t.length > 0);
        if (toks.length !== 2) {
            out.errors.push({ line_no: i + 1, message: 'expected 2 tokens (asset benchmark)' });
            continue;
        }
        const a = pctOrDec(toks[0]);
        const b = pctOrDec(toks[1]);
        if (!Number.isFinite(a) || !Number.isFinite(b)) {
            out.errors.push({ line_no: i + 1, message: 'non-finite token' });
            continue;
        }
        out.asset.push(a);
        out.benchmark.push(b);
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

export function pairsToBlob(asset, benchmark) {
    return asset.map((a, i) => `${a} ${benchmark[i]}`).join('\n');
}

// Verdict on beta magnitude.
export function betaBadge(b) {
    if (b == null || !Number.isFinite(b)) return { key: 'view.beta.badge.unknown', cls: '' };
    if (b >= 1.5)   return { key: 'view.beta.badge.high_beta',     cls: 'neg' };
    if (b >= 1.05)  return { key: 'view.beta.badge.above_market',  cls: '' };
    if (b >= 0.95)  return { key: 'view.beta.badge.market',        cls: '' };
    if (b >= 0.5)   return { key: 'view.beta.badge.low_beta',      cls: 'pos' };
    if (b >= -0.05) return { key: 'view.beta.badge.market_neutral', cls: 'pos' };
    if (b > -0.95)  return { key: 'view.beta.badge.negative_low',  cls: '' };
    return { key: 'view.beta.badge.negative_high', cls: 'neg' };
}

// Verdict on r-squared (fit quality).
export function fitBadge(r2) {
    if (r2 == null || !Number.isFinite(r2)) return { key: 'view.beta.fit.unknown', cls: '' };
    if (r2 >= 0.8)  return { key: 'view.beta.fit.strong',  cls: 'pos' };
    if (r2 >= 0.5)  return { key: 'view.beta.fit.good',    cls: 'pos' };
    if (r2 >= 0.2)  return { key: 'view.beta.fit.moderate', cls: '' };
    if (r2 >= 0.05) return { key: 'view.beta.fit.weak',    cls: 'neg' };
    return { key: 'view.beta.fit.noise', cls: 'neg' };
}

// Beta-neutral hedge sizing: short benchmark $X = beta × asset $.
export function hedgeNotional(asset_notional, beta) {
    if (!Number.isFinite(asset_notional) || !Number.isFinite(beta)) return NaN;
    return asset_notional * beta;
}

// Annualize alpha when caller knows periods/year (252 daily, 12 monthly, etc.).
export function annualizeAlpha(alpha, periods_per_year) {
    if (!Number.isFinite(alpha) || !Number.isFinite(periods_per_year) || periods_per_year <= 0) return NaN;
    return alpha * periods_per_year;
}

function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF - 0.5;
    };
}

export function makeDemoInput(kind = 'tech-stock') {
    switch (kind) {
        case 'tech-stock': {
            // β ≈ 1.3 with alpha = +0.0003/day and noise.
            const rand = lcg(42n);
            const benchmark = [], asset = [];
            for (let i = 0; i < 252; i++) {
                const b = rand() * 0.02;
                benchmark.push(b);
                asset.push(0.0003 + 1.3 * b + rand() * 0.005);
            }
            return { asset, benchmark };
        }
        case 'utility-low-beta': {
            const rand = lcg(7n);
            const benchmark = [], asset = [];
            for (let i = 0; i < 252; i++) {
                const b = rand() * 0.02;
                benchmark.push(b);
                asset.push(0.0001 + 0.3 * b + rand() * 0.002);
            }
            return { asset, benchmark };
        }
        case 'inverse-etf': {
            // β ≈ -1.0 (inverse ETF), tight fit.
            const rand = lcg(31n);
            const benchmark = [], asset = [];
            for (let i = 0; i < 252; i++) {
                const b = rand() * 0.02;
                benchmark.push(b);
                asset.push(-1.0 * b + rand() * 0.001);
            }
            return { asset, benchmark };
        }
        case 'market-neutral': {
            const rand = lcg(99n);
            const benchmark = [], asset = [];
            for (let i = 0; i < 252; i++) {
                const b = rand() * 0.02;
                benchmark.push(b);
                asset.push(0.05 * b + rand() * 0.01);
            }
            return { asset, benchmark };
        }
        case 'high-beta-3x': {
            const rand = lcg(11n);
            const benchmark = [], asset = [];
            for (let i = 0; i < 252; i++) {
                const b = rand() * 0.02;
                benchmark.push(b);
                asset.push(3.0 * b + rand() * 0.001);
            }
            return { asset, benchmark };
        }
        case 'perfect-match': {
            // Asset = benchmark → β=1, R²=1, α=0.
            const benchmark = [];
            for (let i = 0; i < 100; i++) benchmark.push(Math.sin(i * 0.1));
            return { asset: benchmark.slice(), benchmark };
        }
        case 'no-correlation': {
            const randA = lcg(1n);
            const randB = lcg(2n);
            const asset = [], benchmark = [];
            for (let i = 0; i < 252; i++) {
                asset.push(randA() * 0.02);
                benchmark.push(randB() * 0.02);
            }
            return { asset, benchmark };
        }
        case 'flat-bench': {
            const asset = [];
            const benchmark = new Array(50).fill(0);
            for (let i = 0; i < 50; i++) asset.push(Math.sin(i * 0.2) * 0.01);
            return { asset, benchmark };
        }
        default: return makeDemoInput('tech-stock');
    }
}

export function fmtBeta(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toFixed(d);
}

export function fmtAlpha(v, d = 6) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toFixed(d);
}

export function fmtR2(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtPctSigned(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + (v * 100).toFixed(d) + '%';
}

export function fmtUSD(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return '$' + v.toFixed(d);
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
