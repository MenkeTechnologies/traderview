// Active Share (Cremers & Petajisto 2009) helpers.
//
// Backend body: { weights: [{symbol, portfolio_weight, benchmark_weight}, ...] }
// Returns: { active_share, portfolio_weight_sum, benchmark_weight_sum,
//   n_names, n_overweights, n_underweights } | null
//
// AS = ½ · Σ |w_port_i − w_bench_i|.  Range [0, 1].
// 0   = closet indexer; 0.6+ commonly called "active"; 1.0 = disjoint.

// 1e-12 over/under tolerance matches Rust impl.
import { t } from './i18n.js';

export const OVER_TOL = 1e-12;

export const DEFAULT_INPUTS = {
    weights: [],
};

export function validateInputs(input) {
    if (!Array.isArray(input.weights))                          return t('view.active_share.validate.weights_array');
    if (input.weights.length === 0)                              return t('view.active_share.validate.weights_non_empty');
    for (let i = 0; i < input.weights.length; i++) {
        const w = input.weights[i];
        if (!w || typeof w !== 'object')                         return t('view.active_share.validate.weight_object', { i });
        if (typeof w.symbol !== 'string' || w.symbol.length === 0) return t('view.active_share.validate.weight_symbol', { i });
        if (!Number.isFinite(w.portfolio_weight) || w.portfolio_weight < 0)
                                                                  return t('view.active_share.validate.weight_portfolio', { i });
        if (!Number.isFinite(w.benchmark_weight) || w.benchmark_weight < 0)
                                                                  return t('view.active_share.validate.weight_benchmark', { i });
    }
    return null;
}

export function buildBody(input) {
    return {
        weights: input.weights.map(w => ({
            symbol: w.symbol,
            portfolio_weight: w.portfolio_weight,
            benchmark_weight: w.benchmark_weight,
        })),
    };
}

// Pure-JS mirror of crates/traderview-core/src/active_share.rs::compute.
// Returns null on validation failure.
export function localCompute(weights) {
    if (!Array.isArray(weights) || weights.length === 0) return null;
    for (const w of weights) {
        if (!Number.isFinite(w.portfolio_weight) || !Number.isFinite(w.benchmark_weight)) return null;
        if (w.portfolio_weight < 0 || w.benchmark_weight < 0) return null;
    }
    let p_sum = 0, b_sum = 0, abs_diff_sum = 0;
    let n_over = 0, n_under = 0;
    for (const w of weights) {
        p_sum += w.portfolio_weight;
        b_sum += w.benchmark_weight;
        const diff = w.portfolio_weight - w.benchmark_weight;
        abs_diff_sum += Math.abs(diff);
        if (diff >  OVER_TOL) n_over++;
        else if (diff < -OVER_TOL) n_under++;
    }
    return {
        active_share:           Math.min(1, Math.max(0, 0.5 * abs_diff_sum)),
        portfolio_weight_sum:   p_sum,
        benchmark_weight_sum:   b_sum,
        n_names:                weights.length,
        n_overweights:          n_over,
        n_underweights:         n_under,
    };
}

// Parse "symbol portfolio_weight benchmark_weight" per line.
// Weights can be raw decimal (0.4) or pct-suffix ("40%"); pct → decimal.
export function parseWeightsBlob(blob) {
    const out = { weights: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: 'input must be a string' });
        return out;
    }
    const lines = blob.split('\n');
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i].split('#')[0].trim();
        if (!raw) continue;
        const toks = raw.split(/[\s,]+/).filter(t => t.length > 0);
        if (toks.length !== 3) {
            out.errors.push({ line_no: i + 1, message: 'expected 3 tokens (symbol portfolio_w benchmark_w)' });
            continue;
        }
        const symbol = toks[0];
        const p = pctOrDec(toks[1]);
        const b = pctOrDec(toks[2]);
        if (!Number.isFinite(p) || p < 0) {
            out.errors.push({ line_no: i + 1, message: 'portfolio_weight must be ≥ 0 finite' });
            continue;
        }
        if (!Number.isFinite(b) || b < 0) {
            out.errors.push({ line_no: i + 1, message: 'benchmark_weight must be ≥ 0 finite' });
            continue;
        }
        out.weights.push({ symbol, portfolio_weight: p, benchmark_weight: b });
    }
    return out;
}

function pctOrDec(tok) {
    if (tok.endsWith('%')) {
        const v = Number(tok.slice(0, -1));
        return Number.isFinite(v) ? v / 100 : NaN;
    }
    return Number(tok);
}

export function weightsToBlob(weights) {
    return weights.map(w => `${w.symbol} ${w.portfolio_weight} ${w.benchmark_weight}`).join('\n');
}

// 5-tier classification per Cremers-Petajisto convention.
export function styleBadge(active_share) {
    if (!Number.isFinite(active_share)) return { key: 'view.act_share.badge.unknown', cls: '' };
    if (active_share < 0.20) return { key: 'view.act_share.badge.closet',      cls: 'neg' };
    if (active_share < 0.40) return { key: 'view.act_share.badge.semi_closet', cls: 'neg' };
    if (active_share < 0.60) return { key: 'view.act_share.badge.moderate',    cls: '' };
    if (active_share < 0.80) return { key: 'view.act_share.badge.active',      cls: 'pos' };
    return { key: 'view.act_share.badge.very_active', cls: 'pos' };
}

// Sum-quality verdict (do weights add to ~1.0?).
export function sumBadge(sum) {
    if (!Number.isFinite(sum)) return { key: 'view.act_share.sum.unknown', cls: '' };
    if (Math.abs(sum - 1) < 0.001) return { key: 'view.act_share.sum.normalized', cls: 'pos' };
    if (Math.abs(sum - 1) < 0.05)  return { key: 'view.act_share.sum.close',      cls: '' };
    return { key: 'view.act_share.sum.unnormalized', cls: 'neg' };
}

// Per-row enrichment used for the breakdown table.
export function enrich(w) {
    const diff = w.portfolio_weight - w.benchmark_weight;
    let stance;
    if (diff > OVER_TOL) stance = 'over';
    else if (diff < -OVER_TOL) stance = 'under';
    else stance = 'equal';
    return { ...w, diff, abs_diff: Math.abs(diff), stance };
}

export function stanceLabelKey(stance) {
    if (stance === 'over')  return 'view.act_share.stance.over';
    if (stance === 'under') return 'view.act_share.stance.under';
    if (stance === 'equal') return 'view.act_share.stance.equal';
    return 'view.act_share.stance.unknown';
}

// Synthetic demos.
export function makeDemoInput(kind = 'cremers-canonical') {
    switch (kind) {
        case 'identical': {
            return { weights: [
                w('AAPL', 0.4, 0.4),
                w('MSFT', 0.3, 0.3),
                w('GOOG', 0.3, 0.3),
            ]};
        }
        case 'disjoint': {
            return { weights: [
                w('AAPL', 1.0, 0.0),
                w('XOM',  0.0, 1.0),
            ]};
        }
        case 'cremers-canonical': {
            // 50% overlap → AS = 0.50 (textbook example).
            return { weights: [
                w('A', 0.5, 0.5),
                w('B', 0.5, 0.0),
                w('C', 0.0, 0.5),
            ]};
        }
        case 'closet-indexer': {
            // Tiny tilt off the benchmark.
            return { weights: [
                w('AAPL', 0.205, 0.20),
                w('MSFT', 0.195, 0.20),
                w('NVDA', 0.198, 0.20),
                w('AMZN', 0.202, 0.20),
                w('GOOG', 0.200, 0.20),
            ]};
        }
        case 'highly-active': {
            // Concentrated tilts in just 4 of 6 names.
            return { weights: [
                w('PLTR',  0.35, 0.05),
                w('NVDA',  0.30, 0.15),
                w('SOFI',  0.20, 0.0),
                w('TSLA',  0.15, 0.10),
                w('AAPL',  0.0, 0.40),
                w('MSFT',  0.0, 0.30),
            ]};
        }
        case 'sector-bet': {
            // All-tech vs broad-market benchmark.
            return { weights: [
                w('AAPL', 0.30, 0.07),
                w('MSFT', 0.25, 0.07),
                w('NVDA', 0.25, 0.06),
                w('GOOG', 0.20, 0.04),
                w('JPM',  0.0,  0.04),
                w('JNJ',  0.0,  0.03),
                w('XOM',  0.0,  0.03),
                w('UNH',  0.0,  0.03),
                w('OTHR', 0.0,  0.63),
            ]};
        }
        case 'short-bet': {
            // Long-only Active Share (benchmark has positions portfolio doesn't).
            return { weights: [
                w('META', 0.5, 0.10),
                w('NVDA', 0.5, 0.15),
                w('AAPL', 0.0, 0.20),
                w('MSFT', 0.0, 0.20),
                w('OTHR', 0.0, 0.35),
            ]};
        }
        case 'unnormalized': {
            // Both sides sum to 0.95 (slightly off) — view should still compute.
            return { weights: [
                w('A', 0.5, 0.45),
                w('B', 0.45, 0.50),
            ]};
        }
        default:
            return makeDemoInput('cremers-canonical');
    }
}

function w(symbol, portfolio_weight, benchmark_weight) {
    return { symbol, portfolio_weight, benchmark_weight };
}

export function fmtPct(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtPctSigned(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + (v * 100).toFixed(d) + '%';
}

export function fmtNum(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtInt(v) {
    if (!Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
