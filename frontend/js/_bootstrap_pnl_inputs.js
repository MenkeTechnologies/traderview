// Bootstrap P&L Confidence Intervals helpers.
//
// Backend body: { trade_pnls: number[], n_resamples: number, seed: u64 }
// Returns: { mean_total_pnl, median_total_pnl, pnl_5th_percentile,
//   pnl_95th_percentile, pnl_2_5th_percentile, pnl_97_5th_percentile,
//   probability_positive, n_resamples, n_trades } | null
//
// Resamples trade-level P&L with replacement, sums per-resample to total,
// then reports quantiles + probability(total > 0).

import { t } from './i18n.js';

export const DEFAULT_RESAMPLES = 5000;
export const DEFAULT_SEED = 0n;
export const MIN_TRADES = 5;
export const MIN_RESAMPLES = 100;

export const DEFAULT_INPUTS = {
    trade_pnls: [],
    n_resamples: DEFAULT_RESAMPLES,
    seed: DEFAULT_SEED,
};

export function validateInputs(input) {
    if (!Array.isArray(input.trade_pnls))                          return t('view.bootstrap_pnl.validate.pnls_array');
    if (input.trade_pnls.length < MIN_TRADES)                       return t('view.bootstrap_pnl.validate.trades_min', { min: MIN_TRADES });
    for (let i = 0; i < input.trade_pnls.length; i++) {
        if (!Number.isFinite(input.trade_pnls[i]))                  return t('view.bootstrap_pnl.validate.pnls_finite', { i });
    }
    if (!Number.isInteger(input.n_resamples))                       return t('view.bootstrap_pnl.validate.resamples_int');
    if (input.n_resamples < MIN_RESAMPLES)                          return t('view.bootstrap_pnl.validate.resamples_min', { min: MIN_RESAMPLES });
    if (typeof input.seed !== 'bigint' && !Number.isInteger(input.seed))
                                                                      return t('view.bootstrap_pnl.validate.seed');
    return null;
}

export function buildBody(input) {
    return {
        trade_pnls:  input.trade_pnls,
        n_resamples: input.n_resamples,
        // Backend takes u64; JS Number is safe up to 2^53. Convert bigint → Number for wire.
        seed: typeof input.seed === 'bigint' ? Number(input.seed) : input.seed,
    };
}

// Pure-JS mirror of crates/traderview-core/src/bootstrap_pnl.rs::bootstrap.
// Uses BigInt-based LCG matching Rust's wrapping u64 multiplication.
export function localBootstrap(trade_pnls, n_resamples, seed) {
    const n = trade_pnls.length;
    if (n < MIN_TRADES || n_resamples < MIN_RESAMPLES) return null;
    for (const v of trade_pnls) if (!Number.isFinite(v)) return null;
    const MASK = 0xFFFFFFFFFFFFFFFFn;
    let state = typeof seed === 'bigint' ? (seed & MASK) : (BigInt(seed) & MASK);
    const resampled = new Array(n_resamples);
    for (let r = 0; r < n_resamples; r++) {
        let total = 0;
        for (let i = 0; i < n; i++) {
            state = (state * 6364136223846793005n + 1442695040888963407n) & MASK;
            const idx = Number(state >> 32n) % n;
            total += trade_pnls[idx];
        }
        resampled[r] = total;
    }
    resampled.sort((a, b) => a - b);
    const n_f = n_resamples;
    let sum = 0;
    for (const v of resampled) sum += v;
    const q = (p) => resampled[Math.min(n_resamples - 1, Math.floor(p * n_f))];
    let positive_count = 0;
    for (const v of resampled) if (v > 0) positive_count++;
    return {
        mean_total_pnl:        sum / n_f,
        median_total_pnl:      q(0.50),
        pnl_5th_percentile:    q(0.05),
        pnl_95th_percentile:   q(0.95),
        pnl_2_5th_percentile:  q(0.025),
        pnl_97_5th_percentile: q(0.975),
        probability_positive:  positive_count / n_f,
        n_resamples,
        n_trades: n,
    };
}

// Parse comma/whitespace-separated trade P&L values; comments + blanks ignored.
// Supports $-prefix and ()-wrapped negatives ("($50)") for accounting notation.
export function parseTradesBlob(blob) {
    const out = { trade_pnls: [], errors: [] };
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
        let tok = tokens[i];
        let neg = false;
        if (tok.startsWith('(') && tok.endsWith(')')) {
            neg = true;
            tok = tok.slice(1, -1);
        }
        // Strip $ anywhere so "-$30" and "$-30" both parse cleanly.
        tok = tok.replace(/\$/g, '');
        const v = Number(tok);
        if (!Number.isFinite(v)) {
            out.errors.push({ line_no: i + 1, message: `token "${tokens[i]}" not finite` });
            continue;
        }
        out.trade_pnls.push(neg ? -v : v);
    }
    return out;
}

export function tradesToBlob(trade_pnls) {
    return trade_pnls.join('\n');
}

// 5-tier verdict on probability(positive total P&L).
export function probBadge(p) {
    if (p == null || !Number.isFinite(p)) return { key: 'view.boot_pnl.badge.unknown', cls: '' };
    if (p >= 0.95)                          return { key: 'view.boot_pnl.badge.almost_certain', cls: 'pos' };
    if (p >= 0.75)                          return { key: 'view.boot_pnl.badge.profitable',     cls: 'pos' };
    if (p >= 0.55)                          return { key: 'view.boot_pnl.badge.edge',           cls: '' };
    if (p >= 0.45)                          return { key: 'view.boot_pnl.badge.coin_flip',      cls: '' };
    if (p >= 0.25)                          return { key: 'view.boot_pnl.badge.unfavorable',    cls: 'neg' };
    return { key: 'view.boot_pnl.badge.disaster', cls: 'neg' };
}

// CI-width interpretation — tighter is more certain.
export function ciBadge(report) {
    if (!report) return { key: 'view.boot_pnl.ci.unknown', cls: '' };
    const width95 = report.pnl_97_5th_percentile - report.pnl_2_5th_percentile;
    const mean = Math.abs(report.mean_total_pnl);
    if (!Number.isFinite(width95) || mean === 0) return { key: 'view.boot_pnl.ci.unknown', cls: '' };
    const ratio = width95 / mean;
    if (ratio < 1)   return { key: 'view.boot_pnl.ci.tight', cls: 'pos' };
    if (ratio < 3)   return { key: 'view.boot_pnl.ci.moderate', cls: '' };
    if (ratio < 10)  return { key: 'view.boot_pnl.ci.wide', cls: 'neg' };
    return { key: 'view.boot_pnl.ci.extreme', cls: 'neg' };
}

// Per-trade stats (no resampling — describes the raw distribution).
export function summarizeTrades(trade_pnls) {
    if (!Array.isArray(trade_pnls) || trade_pnls.length === 0) {
        return { count: 0, mean: NaN, sum: NaN, wins: 0, losses: 0, win_rate: NaN,
                 max_win: NaN, max_loss: NaN };
    }
    let sum = 0, wins = 0, losses = 0, mx = -Infinity, mn = Infinity;
    for (const v of trade_pnls) {
        sum += v;
        if (v > 0) wins++;
        else if (v < 0) losses++;
        if (v > mx) mx = v;
        if (v < mn) mn = v;
    }
    return {
        count: trade_pnls.length,
        sum,
        mean: sum / trade_pnls.length,
        wins, losses,
        win_rate: trade_pnls.length > 0 ? wins / trade_pnls.length : NaN,
        max_win: Number.isFinite(mx) ? mx : NaN,
        max_loss: Number.isFinite(mn) ? mn : NaN,
    };
}

function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF;
    };
}

export function makeDemoInput(kind = 'winning-strategy') {
    switch (kind) {
        case 'winning-strategy': {
            // Positive expectancy: 60% wins of $50, 40% losses of $30.
            const rand = lcg(42n);
            const trades = [];
            for (let i = 0; i < 100; i++) trades.push(rand() > 0.4 ? 50 : -30);
            return { trade_pnls: trades, n_resamples: 5000, seed: 42n };
        }
        case 'losing-strategy': {
            const rand = lcg(7n);
            const trades = [];
            for (let i = 0; i < 100; i++) trades.push(rand() > 0.55 ? 30 : -50);
            return { trade_pnls: trades, n_resamples: 5000, seed: 42n };
        }
        case 'high-variance': {
            // High-variance system: big wins, big losses, slight edge.
            const rand = lcg(99n);
            const trades = [];
            for (let i = 0; i < 100; i++) trades.push(rand() > 0.5 ? 200 : -180);
            return { trade_pnls: trades, n_resamples: 5000, seed: 42n };
        }
        case 'low-variance': {
            // Tight grid: small consistent wins.
            const rand = lcg(11n);
            const trades = [];
            for (let i = 0; i < 100; i++) trades.push(rand() > 0.45 ? 5 : -3);
            return { trade_pnls: trades, n_resamples: 5000, seed: 42n };
        }
        case 'all-winners': {
            return { trade_pnls: [10, 5, 20, 8, 15, 12, 25, 30, 18, 22],
                     n_resamples: 1000, seed: 42n };
        }
        case 'all-losers': {
            return { trade_pnls: [-10, -5, -20, -8, -15, -12, -25, -30],
                     n_resamples: 1000, seed: 42n };
        }
        case 'lumpy-tail': {
            // 95% small wins, 5% catastrophic losses (martingale-style).
            const trades = [];
            for (let i = 0; i < 95; i++) trades.push(10);
            for (let i = 0; i < 5; i++) trades.push(-500);
            return { trade_pnls: trades, n_resamples: 5000, seed: 13n };
        }
        case 'few-trades': {
            // Just above the minimum to show degenerate-ish CIs.
            return { trade_pnls: [10, -5, 15, -10, 20, -8, 12, 25],
                     n_resamples: 2000, seed: 42n };
        }
        default: return makeDemoInput('winning-strategy');
    }
}

export function fmtUSD(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-' : '';
    return sign + '$' + Math.abs(v).toFixed(d);
}

export function fmtUSDSigned(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '-') + '$' + Math.abs(v).toFixed(d);
}

export function fmtPct(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
