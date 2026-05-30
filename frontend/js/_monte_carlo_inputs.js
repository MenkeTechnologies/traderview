// Monte Carlo helpers shared by view + vitest.
//
// Backend body: { historical_r: f64[], config: { n_curves, trades_per_curve,
//   start_equity, ruin_threshold, seed } }.
// Returns: McReport with 5 ending-equity percentiles + mean, 3 drawdown
// percentiles + mean, probability_of_ruin, probability_profitable.
//
// Local mirror uses the same LCG (Lemire's bounded-rand) as Rust so the
// JS pre-flight matches the backend bit-for-bit at a given seed.

import { t as tr } from './i18n.js';

export const DEFAULT_CONFIG = {
    n_curves: 1000,
    trades_per_curve: 100,
    start_equity: 10_000,
    ruin_threshold: 5_000,
    seed: 42,
};

export function parseRBlob(text) {
    const r = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { r, errors: [{ line: 0, message: tr('common.parse.input_must_be_string') }] };
    }
    const cleaned = text.replace(/#[^\n]*/g, ' ');
    const tokens = cleaned.split(/[\s,]+/).map(t => t.trim()).filter(Boolean);
    tokens.forEach((tok, i) => {
        const n = Number(tok);
        if (!Number.isFinite(n)) {
            errors.push({ line: i + 1, message: `"${tok}" is not finite` });
        } else {
            r.push(n);
        }
    });
    return { r, errors };
}

export function validateInputs(historical_r, cfg) {
    if (!Array.isArray(historical_r) || historical_r.length === 0)
        return tr('view.monte_carlo.validate.r_empty');
    if (historical_r.some(v => !Number.isFinite(v))) return tr('view.monte_carlo.validate.r_finite');
    if (!Number.isInteger(cfg.n_curves) || cfg.n_curves < 1)         return tr('view.monte_carlo.validate.n_curves');
    if (!Number.isInteger(cfg.trades_per_curve) || cfg.trades_per_curve < 1)
        return tr('view.monte_carlo.validate.trades_per_curve');
    if (!Number.isFinite(cfg.start_equity) || cfg.start_equity <= 0) return tr('view.monte_carlo.validate.start_equity');
    if (!Number.isFinite(cfg.ruin_threshold) || cfg.ruin_threshold < 0)
        return tr('view.monte_carlo.validate.ruin_threshold');
    if (cfg.n_curves > 50_000) return tr('view.monte_carlo.validate.n_curves_cap');
    if (cfg.trades_per_curve > 10_000) return tr('view.monte_carlo.validate.trades_cap');
    return null;
}

export function buildBody(historical_r, cfg) {
    return {
        historical_r: historical_r.slice(),
        config: { ...cfg },
    };
}

// Mirror of crates/traderview-core/src/monte_carlo.rs::simulate with the
// same LCG. Uses BigInt for the unsigned u64 / u128 multiplications since
// JS Number can't represent them losslessly.
export function localSimulate(historical_r, cfg) {
    if (!historical_r || historical_r.length === 0 || cfg.n_curves === 0 || cfg.trades_per_curve === 0) {
        return null;
    }
    const rng = new Lcg(BigInt(cfg.seed >>> 0));
    const ending = [];
    const maxDds = [];
    let ruinCount = 0;
    let profitableCount = 0;
    const len = BigInt(historical_r.length);
    for (let c = 0; c < cfg.n_curves; c++) {
        let equity = cfg.start_equity;
        let peak = equity;
        let maxDd = 0;
        let hitRuin = false;
        for (let t = 0; t < cfg.trades_per_curve; t++) {
            const idx = Number(rng.nextBounded(len));
            equity += historical_r[idx];
            if (equity > peak) peak = equity;
            if (equity <= cfg.ruin_threshold) hitRuin = true;
            const ddPct = peak > 0 ? (peak - equity) / peak : 0;
            if (ddPct > maxDd) maxDd = ddPct;
        }
        ending.push(equity);
        maxDds.push(maxDd);
        if (hitRuin) ruinCount++;
        if (equity > cfg.start_equity) profitableCount++;
    }
    ending.sort((a, b) => a - b);
    maxDds.sort((a, b) => a - b);
    const n = ending.length;
    return {
        n_curves: cfg.n_curves,
        trades_per_curve: cfg.trades_per_curve,
        start_equity: cfg.start_equity,
        ending_equity_p05: pct(ending, 0.05),
        ending_equity_p25: pct(ending, 0.25),
        ending_equity_p50: pct(ending, 0.50),
        ending_equity_p75: pct(ending, 0.75),
        ending_equity_p95: pct(ending, 0.95),
        mean_ending_equity: ending.reduce((a, b) => a + b, 0) / n,
        max_drawdown_p05: pct(maxDds, 0.05),
        max_drawdown_p50: pct(maxDds, 0.50),
        max_drawdown_p95: pct(maxDds, 0.95),
        mean_max_drawdown: maxDds.reduce((a, b) => a + b, 0) / n,
        probability_of_ruin: ruinCount / n,
        probability_profitable: profitableCount / n,
    };
}

export function pct(sorted, q) {
    if (sorted.length === 0) return 0;
    const idx = Math.round((sorted.length - 1) * q);
    return sorted[Math.min(Math.max(idx, 0), sorted.length - 1)];
}

// LCG with MMIX constants + Lemire bounded-rand. Uses BigInt for the
// u128 multiply so the bit-pattern matches Rust's implementation.
const MMIX_MUL = 6_364_136_223_846_793_005n;
const MMIX_INC = 1_442_695_040_888_963_407n;
const MASK_64 = (1n << 64n) - 1n;
const SPLITMIX_NUDGE = 0x9E3779B97F4A7C15n;

export class Lcg {
    constructor(seed) {
        this.state = (BigInt.asUintN(64, BigInt(seed)) + SPLITMIX_NUDGE) & MASK_64;
    }
    nextU64() {
        this.state = (this.state * MMIX_MUL + MMIX_INC) & MASK_64;
        return this.state;
    }
    nextBounded(bound) {
        if (bound <= 0n) return 0n;
        let x = this.nextU64();
        let m = x * bound;
        let l = m & MASK_64;
        if (l < bound) {
            const t = ((1n << 64n) - bound) % bound;
            while (l < t) {
                x = this.nextU64();
                m = x * bound;
                l = m & MASK_64;
            }
        }
        return m >> 64n;
    }
}

// Compute a histogram of ending equity for the chart. Returns bin
// centers + counts. Reuses the local-simulate output's `ending` array
// by re-running just the per-curve loop — but for the post-network
// report we don't have the raw curves, so we accept a pre-computed
// `ending[]` if available.
export function endingHistogram(endingValues, nBuckets = 30) {
    if (!Array.isArray(endingValues) || endingValues.length === 0) {
        return { centers: [], counts: [] };
    }
    const sorted = [...endingValues].sort((a, b) => a - b);
    const lo = sorted[0];
    const hi = sorted[sorted.length - 1];
    if (hi <= lo) return { centers: [lo], counts: [sorted.length] };
    const w = (hi - lo) / nBuckets;
    const centers = Array.from({ length: nBuckets }, (_, i) => lo + (i + 0.5) * w);
    const counts = new Array(nBuckets).fill(0);
    for (const v of sorted) {
        let i = Math.floor((v - lo) / w);
        if (i < 0) i = 0;
        if (i >= nBuckets) i = nBuckets - 1;
        counts[i]++;
    }
    return { centers, counts };
}

// Run the full simulation client-side and surface BOTH the McReport and
// the raw ending-equity series (so the chart can plot the distribution).
// The backend doesn't return the raw curves — we always need to compute
// them locally for visualization.
export function localSimulateWithCurves(historical_r, cfg) {
    if (!historical_r || historical_r.length === 0 || cfg.n_curves === 0 || cfg.trades_per_curve === 0) {
        return { report: null, ending: [], maxDds: [] };
    }
    const rng = new Lcg(BigInt(cfg.seed >>> 0));
    const ending = [];
    const maxDds = [];
    let ruinCount = 0;
    let profitableCount = 0;
    const len = BigInt(historical_r.length);
    for (let c = 0; c < cfg.n_curves; c++) {
        let equity = cfg.start_equity;
        let peak = equity;
        let maxDd = 0;
        let hitRuin = false;
        for (let t = 0; t < cfg.trades_per_curve; t++) {
            const idx = Number(rng.nextBounded(len));
            equity += historical_r[idx];
            if (equity > peak) peak = equity;
            if (equity <= cfg.ruin_threshold) hitRuin = true;
            const ddPct = peak > 0 ? (peak - equity) / peak : 0;
            if (ddPct > maxDd) maxDd = ddPct;
        }
        ending.push(equity);
        maxDds.push(maxDd);
        if (hitRuin) ruinCount++;
        if (equity > cfg.start_equity) profitableCount++;
    }
    const sortedEnding = [...ending].sort((a, b) => a - b);
    const sortedDds    = [...maxDds].sort((a, b) => a - b);
    const n = ending.length;
    const report = {
        n_curves: cfg.n_curves,
        trades_per_curve: cfg.trades_per_curve,
        start_equity: cfg.start_equity,
        ending_equity_p05: pct(sortedEnding, 0.05),
        ending_equity_p25: pct(sortedEnding, 0.25),
        ending_equity_p50: pct(sortedEnding, 0.50),
        ending_equity_p75: pct(sortedEnding, 0.75),
        ending_equity_p95: pct(sortedEnding, 0.95),
        mean_ending_equity: sortedEnding.reduce((a, b) => a + b, 0) / n,
        max_drawdown_p05: pct(sortedDds, 0.05),
        max_drawdown_p50: pct(sortedDds, 0.50),
        max_drawdown_p95: pct(sortedDds, 0.95),
        mean_max_drawdown: sortedDds.reduce((a, b) => a + b, 0) / n,
        probability_of_ruin: ruinCount / n,
        probability_profitable: profitableCount / n,
    };
    return { report, ending, maxDds };
}

// Demo presets — small enough to run interactively yet show distinct shapes.
export function makeDemoR(kind = 'positive-edge') {
    switch (kind) {
        case 'positive-edge':
            // 60% +1R, 40% -1R → mean +0.2R, clear edge.
            return Array.from({ length: 100 }, (_, i) => i % 5 < 3 ? 1 : -1);
        case 'negative-edge':
            return Array.from({ length: 100 }, (_, i) => i % 5 < 2 ? 1 : -1);
        case 'fat-tail':
            // Mostly small wins, occasional -5R disaster.
            return Array.from({ length: 100 }, (_, i) =>
                i % 25 === 0 ? -5 : i % 3 === 0 ? -0.5 : 0.5);
        case 'lumpy-winner':
            // Mean positive but driven by a few big winners — distribution skewed right.
            return Array.from({ length: 100 }, (_, i) =>
                i % 10 === 0 ? 3 : i % 4 === 0 ? -0.5 : 0.2);
        case 'random':
            // Symmetric ±2R with tiny drift — break-even on average.
            return Array.from({ length: 100 }, (_, i) => (((i * 23) % 9) - 4) / 2);
        default:
            return makeDemoR('positive-edge');
    }
}

export function ruinBadge(probabilityOfRuin) {
    if (!Number.isFinite(probabilityOfRuin)) return { key: 'view.monte_carlo.badge.unknown' };
    if (probabilityOfRuin >= 0.10) return { key: 'view.monte_carlo.badge.high_ruin',     cls: 'neg' };
    if (probabilityOfRuin >= 0.02) return { key: 'view.monte_carlo.badge.moderate_ruin', cls: 'neg' };
    if (probabilityOfRuin > 0)     return { key: 'view.monte_carlo.badge.low_ruin',      cls: '' };
    return { key: 'view.monte_carlo.badge.no_ruin', cls: 'pos' };
}

export function fmtUSD(v, d = 0) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-' : '';
    return sign + '$' + Math.abs(v).toFixed(d);
}

export function fmtUSDSigned(v, d = 0) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '-') + '$' + Math.abs(v).toFixed(d);
}

export function fmtPct(v, d = 1) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtNum(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}
