// Value-at-Risk (VaR) helpers shared by view + vitest. Covers BOTH
// the historical + parametric-Gaussian endpoints (same body shape).
//
// Backend body: { daily_returns: f64[], position_value: f64, confidence: f64 }.
// Returns: { method, confidence, var_dollars, expected_shortfall_dollars, n }.

import { t as tr } from './i18n.js';

// Parse daily returns as a CSV / whitespace / newline mix. Returns
// always shown as decimals (0.01 = 1%, -0.02 = -2%). Optional %-suffix
// auto-divides by 100.
export function parseReturnsBlob(text) {
    const returns = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { returns, errors: [{ line: 0, message: 'expected string input' }] };
    }
    const cleaned = text.replace(/#[^\n]*/g, ' ');  // strip line comments
    const tokens = cleaned.split(/[\s,]+/).map(t => t.trim()).filter(Boolean);
    tokens.forEach((tok, i) => {
        let s = tok;
        let div = 1;
        if (s.endsWith('%')) { s = s.slice(0, -1); div = 100; }
        const n = Number(s);
        if (!Number.isFinite(n)) {
            errors.push({ line: i + 1, message: `"${tok}" is not finite` });
        } else {
            returns.push(n / div);
        }
    });
    return { returns, errors };
}

export function validateInputs(returns, positionValue, confidence) {
    if (!Array.isArray(returns)) return tr('view.var_estimator.validate.returns_array');
    if (returns.length < 10) return tr('view.var_estimator.validate.returns_min');
    if (returns.some(v => !Number.isFinite(v))) return tr('view.var_estimator.validate.returns_finite');
    if (!Number.isFinite(positionValue) || positionValue <= 0) return tr('view.var_estimator.validate.position');
    if (!Number.isFinite(confidence) || confidence <= 0 || confidence >= 1)
        return tr('view.var_estimator.validate.confidence');
    return null;
}

export function buildBody(returns, positionValue, confidence) {
    return { daily_returns: returns, position_value: positionValue, confidence };
}

// Pure-JS mirror of crates/traderview-core/src/var_estimator.rs::historical.
// Same fallback default for n<10 or bad confidence.
export function localHistorical(returns, positionValue, confidence) {
    const report = {
        method: 'historical', confidence, n: returns.length,
        var_dollars: 0, expected_shortfall_dollars: 0,
    };
    if (returns.length < 10 || confidence <= 0 || confidence >= 1) return report;
    const sorted = [...returns].sort((a, b) => a - b);
    const alpha = 1 - confidence;
    let idx = Math.floor(alpha * sorted.length) - 1;
    if (idx < 0) idx = 0;
    if (idx > sorted.length - 1) idx = sorted.length - 1;
    const varPct = -sorted[idx];
    report.var_dollars = varPct * positionValue;
    const tail = sorted.slice(0, idx + 1).map(r => -r);
    if (tail.length > 0) {
        const mean = tail.reduce((a, b) => a + b, 0) / tail.length;
        report.expected_shortfall_dollars = mean * positionValue;
    }
    return report;
}

// Pure-JS mirror of var_estimator::parametric_gaussian.
export function localParametricGaussian(returns, positionValue, confidence) {
    const report = {
        method: 'parametric_gaussian', confidence, n: returns.length,
        var_dollars: 0, expected_shortfall_dollars: 0,
    };
    if (returns.length < 2 || confidence <= 0 || confidence >= 1) return report;
    const n = returns.length;
    const mean = returns.reduce((a, b) => a + b, 0) / n;
    const variance = returns.reduce((a, r) => a + (r - mean) ** 2, 0) / n;
    const stdev = Math.sqrt(variance);
    const z = inverseNormal(confidence);
    const varPct = -(mean - z * stdev);
    report.var_dollars = Math.max(varPct, 0) * positionValue;
    const alpha = 1 - confidence;
    const phiZ = Math.exp(-0.5 * z * z) / Math.sqrt(2 * Math.PI);
    const esPct = -(mean - stdev * phiZ / alpha);
    report.expected_shortfall_dollars = Math.max(esPct, 0) * positionValue;
    return report;
}

// Mirror of Rust ::inverse_normal — Beasley-Springer-Moro w/ fast paths.
export function inverseNormal(confidence) {
    if (Math.abs(confidence - 0.90) < 1e-6)  return 1.282;
    if (Math.abs(confidence - 0.95) < 1e-6)  return 1.645;
    if (Math.abs(confidence - 0.99) < 1e-6)  return 2.326;
    if (Math.abs(confidence - 0.999) < 1e-6) return 3.090;
    if (confidence < 0.5) return -inverseNormal(1 - confidence);
    const p = confidence;
    const t = Math.sqrt(-2 * Math.log(1 - p));
    const c0 = 2.515517, c1 = 0.802853, c2 = 0.010328;
    const d1 = 1.432788, d2 = 0.189269, d3 = 0.001308;
    return t - (c0 + c1 * t + c2 * t * t) / (1 + d1 * t + d2 * t * t + d3 * t * t * t);
}

// Standard normal PDF (utility).
export function phi(z) {
    return Math.exp(-0.5 * z * z) / Math.sqrt(2 * Math.PI);
}

// Statistics summary helpful for diagnosing the underlying distribution
// (which informs whether historical or parametric VaR is more trustworthy).
export function distributionStats(returns) {
    const out = { n: 0, mean: NaN, stdev: NaN, min: NaN, max: NaN, skewness: NaN, kurtosis: NaN, fattest_left_tail: NaN };
    if (!Array.isArray(returns) || returns.length === 0) return out;
    out.n = returns.length;
    const n = returns.length;
    let sum = 0, min = Infinity, max = -Infinity;
    for (const r of returns) { sum += r; if (r < min) min = r; if (r > max) max = r; }
    out.mean = sum / n;
    out.min = min; out.max = max;
    let var2 = 0, m3 = 0, m4 = 0;
    for (const r of returns) {
        const d = r - out.mean;
        var2 += d * d;
        m3 += d * d * d;
        m4 += d * d * d * d;
    }
    var2 /= n;
    out.stdev = Math.sqrt(var2);
    if (out.stdev > 0) {
        out.skewness = (m3 / n) / out.stdev ** 3;
        out.kurtosis = (m4 / n) / out.stdev ** 4 - 3; // excess kurtosis
    }
    // Worst left-tail return as a multiple of stdev.
    if (out.stdev > 0) out.fattest_left_tail = (min - out.mean) / out.stdev;
    return out;
}

// Bin the loss distribution for a histogram. Buckets are LOSS dollars
// (positive numbers = losses), nBuckets evenly across [0, max_loss].
export function lossHistogram(returns, positionValue, nBuckets = 30) {
    if (!Array.isArray(returns) || returns.length === 0 || positionValue <= 0) {
        return { edges: [], counts: [] };
    }
    const losses = returns.map(r => -r * positionValue);
    const maxLoss = Math.max(...losses, 0);
    const minLoss = Math.min(...losses, 0);
    if (maxLoss <= minLoss) return { edges: [], counts: [] };
    const range = maxLoss - minLoss;
    const w = range / nBuckets;
    const edges = Array.from({ length: nBuckets + 1 }, (_, i) => minLoss + i * w);
    const counts = new Array(nBuckets).fill(0);
    for (const loss of losses) {
        let i = Math.floor((loss - minLoss) / w);
        if (i < 0) i = 0;
        if (i >= nBuckets) i = nBuckets - 1;
        counts[i]++;
    }
    return { edges, counts };
}

// Compare historical vs parametric VAR for "Gaussian vs reality" insight.
export function compareMethods(hist, gauss) {
    const diff = hist.var_dollars - gauss.var_dollars;
    const pct = gauss.var_dollars > 0 ? diff / gauss.var_dollars : 0;
    return { diff, pct };
}

const PALETTE = ['#00e5ff', '#ffd84a', '#ff3860', '#23d18b'];

// 5 demo presets that exercise common distribution shapes.
export function makeDemoReturns(kind = 'normal') {
    const out = [];
    switch (kind) {
        case 'normal': {
            // ~250 days of N(0, 0.01) — bell-shaped, Gaussian VaR ≈ historical.
            for (let i = 0; i < 250; i++) {
                const u = (i * 0.131) % 1, v = (i * 0.371) % 1;
                const z = Math.sqrt(-2 * Math.log(Math.max(1e-9, u))) * Math.cos(2 * Math.PI * v);
                out.push(z * 0.01);
            }
            return out;
        }
        case 'fat-tail': {
            // Normal-ish 95% of the time but occasional 5-sigma blowups.
            // Historical VaR > Gaussian VaR.
            for (let i = 0; i < 250; i++) {
                const u = (i * 0.131) % 1, v = (i * 0.371) % 1;
                const z = Math.sqrt(-2 * Math.log(Math.max(1e-9, u))) * Math.cos(2 * Math.PI * v);
                if (i % 22 === 0) out.push(-0.05);  // 11 fat-tail negatives
                else out.push(z * 0.01);
            }
            return out;
        }
        case 'crisis': {
            // 30 days of -3% to -5% drops embedded in noise (2008/COVID-style).
            for (let i = 0; i < 250; i++) {
                if (i >= 100 && i < 130) out.push(-0.03 - (i % 5) * 0.005);
                else out.push(((i * 17) % 7 - 3) * 0.002);
            }
            return out;
        }
        case 'low-vol': {
            // Tight gainer: returns clustered around +0.05% with σ ≈ 0.3%.
            for (let i = 0; i < 250; i++) {
                out.push(0.0005 + ((i * 13) % 5 - 2) * 0.0015);
            }
            return out;
        }
        case 'random-walk': {
            // Trending random walk with bursts — closer to real intraday data.
            let p = 0;
            for (let i = 0; i < 250; i++) {
                p += ((i * 41) % 11 - 5) / 100;
                if (i > 0) out.push((p - (i - 1) * 0.0001) / Math.max(1, i) * 0.1);
                else out.push(0);
            }
            return out.map(r => Math.max(-0.05, Math.min(0.05, r)));
        }
        default:
            return makeDemoReturns('normal');
    }
}

export function fmtUSD(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-' : '';
    return sign + '$' + Math.abs(v).toFixed(d);
}

export function fmtUSDSigned(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '-') + '$' + Math.abs(v).toFixed(d);
}

export function fmtPct(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtN(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function methodColor(method) {
    return method === 'historical' ? PALETTE[0] : PALETTE[1];
}
