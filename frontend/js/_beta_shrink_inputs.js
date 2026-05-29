// Vasicek (1973) Bayesian Beta Shrinkage helpers.
//
// Backend body: {
//   assets: [{ symbol: string, asset_returns: number[] }, ...],
//   market_returns: number[]
// }
// Returns: {
//   prior_beta, cross_sectional_variance,
//   assets: [{ symbol, beta_ols, standard_error, shrinkage_weight, beta_shrunk }, ...]
// } | null

export const MIN_OBS = 5;

export const DEFAULT_INPUTS = {
    assets: [],
    market_returns: [],
};

export function validateInputs(input) {
    if (!Array.isArray(input.assets))                       return 'assets must be an array';
    if (input.assets.length === 0)                          return 'need at least one asset';
    if (!Array.isArray(input.market_returns))               return 'market_returns must be an array';
    if (input.market_returns.length < MIN_OBS)              return `market_returns needs ≥ ${MIN_OBS} obs`;
    for (let i = 0; i < input.market_returns.length; i++) {
        if (!Number.isFinite(input.market_returns[i]))      return `market_returns[${i}] not finite`;
    }
    for (let a = 0; a < input.assets.length; a++) {
        const x = input.assets[a];
        if (!x)                                              return `assets[${a}] missing`;
        if (typeof x.symbol !== 'string' || x.symbol.length === 0)
                                                              return `assets[${a}] symbol missing`;
        if (!Array.isArray(x.asset_returns))                 return `assets[${a}] asset_returns must be array`;
        // Length mismatch & non-finite are skipped server-side (per Rust impl); allow here too.
    }
    return null;
}

export function buildBody(input) {
    return {
        assets: input.assets.map(a => ({
            symbol: a.symbol,
            asset_returns: a.asset_returns.slice(),
        })),
        market_returns: input.market_returns.slice(),
    };
}

// Pure-JS mirror of crates/traderview-core/src/beta_shrinkage.rs::shrink.
export function localShrink(assets, market_returns) {
    if (!Array.isArray(assets) || assets.length === 0) return null;
    if (!Array.isArray(market_returns) || market_returns.length < MIN_OBS) return null;
    for (const v of market_returns) if (!Number.isFinite(v)) return null;
    const ols = [];
    for (const a of assets) {
        if (!Array.isArray(a.asset_returns)
            || a.asset_returns.length !== market_returns.length) continue;
        let allFinite = true;
        for (const v of a.asset_returns) if (!Number.isFinite(v)) { allFinite = false; break; }
        if (!allFinite) continue;
        const r = olsBeta(a.asset_returns, market_returns);
        if (r != null) ols.push({ symbol: a.symbol, beta: r.beta, se: r.se });
    }
    if (ols.length === 0) return null;
    const prior_beta = ols.reduce((s, o) => s + o.beta, 0) / ols.length;
    const cs_var = ols.length > 1
        ? ols.reduce((s, o) => s + (o.beta - prior_beta) ** 2, 0) / (ols.length - 1)
        : 0;
    const out_assets = ols.map(o => {
        const var_ols = o.se * o.se;
        const denom = cs_var + var_ols;
        const w = denom > 0 ? cs_var / denom : 0;
        const shrunk = w * o.beta + (1 - w) * prior_beta;
        return {
            symbol: o.symbol,
            beta_ols: o.beta,
            standard_error: o.se,
            shrinkage_weight: w,
            beta_shrunk: shrunk,
        };
    });
    return { prior_beta, cross_sectional_variance: cs_var, assets: out_assets };
}

export function olsBeta(y, x) {
    const n = y.length;
    if (n < MIN_OBS || x.length !== n) return null;
    const n_f = n;
    let xMean = 0, yMean = 0;
    for (let i = 0; i < n; i++) { xMean += x[i]; yMean += y[i]; }
    xMean /= n_f; yMean /= n_f;
    let s_xy = 0, s_xx = 0;
    for (let i = 0; i < n; i++) {
        const dx = x[i] - xMean;
        const dy = y[i] - yMean;
        s_xy += dx * dy;
        s_xx += dx * dx;
    }
    if (s_xx <= 0) return null;
    const beta = s_xy / s_xx;
    const alpha = yMean - beta * xMean;
    let ssr = 0;
    for (let i = 0; i < n; i++) {
        const resid = y[i] - alpha - beta * x[i];
        ssr += resid * resid;
    }
    const dof = n - 2;
    if (dof <= 0) return null;
    const sigma2 = ssr / dof;
    const se = Math.sqrt(Math.max(0, sigma2 / s_xx));
    return { beta, se };
}

// Parse blob: lines like "SYMBOL r1 r2 r3 ..." → 1 asset per line.
export function parseAssetsBlob(blob) {
    const out = { assets: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: 'input must be a string' });
        return out;
    }
    const lines = blob.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.split('#')[0].trim();
        if (!s) continue;
        const parts = s.split(/[\s,]+/).filter(Boolean);
        if (parts.length < 2) {
            out.errors.push({ line_no: i + 1, message: `expected SYMBOL plus ≥ 1 return value` });
            continue;
        }
        const symbol = parts[0];
        const asset_returns = [];
        let bad = false;
        for (let j = 1; j < parts.length; j++) {
            const v = Number(parts[j].replace(/[\$%]/g, ''));
            if (!Number.isFinite(v)) {
                out.errors.push({ line_no: i + 1, message: `token "${parts[j]}" not finite` });
                bad = true;
                break;
            }
            asset_returns.push(v);
        }
        if (!bad) out.assets.push({ symbol, asset_returns });
    }
    return out;
}

export function assetsToBlob(assets) {
    return assets.map(a => `${a.symbol} ${a.asset_returns.join(' ')}`).join('\n');
}

// Parse market series: 1 number per token.
export function parseMarketBlob(blob) {
    const out = { market_returns: [], errors: [] };
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
        const v = Number(tokens[i].replace(/[\$%]/g, ''));
        if (!Number.isFinite(v)) {
            out.errors.push({ line_no: i + 1, message: `token "${tokens[i]}" not finite` });
            continue;
        }
        out.market_returns.push(v);
    }
    return out;
}

export function marketToBlob(market_returns) {
    return market_returns.join('\n');
}

// Per-asset shrinkage weight verdict.
export function weightBadge(w) {
    if (w == null || !Number.isFinite(w)) return { key: 'view.beta_shrink.w.unknown', cls: '' };
    if (w >= 0.80) return { key: 'view.beta_shrink.w.high',     cls: 'pos' };
    if (w >= 0.50) return { key: 'view.beta_shrink.w.moderate', cls: '' };
    if (w >= 0.20) return { key: 'view.beta_shrink.w.low',      cls: 'neg' };
    return { key: 'view.beta_shrink.w.very_low', cls: 'neg' };
}

// Beta classification: low / market / high / leveraged / inverse.
export function betaBadge(b) {
    if (b == null || !Number.isFinite(b)) return { key: 'view.beta_shrink.beta.unknown', cls: '' };
    if (b < 0)         return { key: 'view.beta_shrink.beta.inverse',    cls: 'neg' };
    if (b < 0.5)       return { key: 'view.beta_shrink.beta.low',        cls: '' };
    if (b < 1.5)       return { key: 'view.beta_shrink.beta.market',     cls: '' };
    if (b < 2.5)       return { key: 'view.beta_shrink.beta.high',       cls: 'pos' };
    return { key: 'view.beta_shrink.beta.leveraged', cls: 'pos' };
}

// Cross-sectional dispersion verdict.
export function dispersionBadge(cs_var, n_assets) {
    if (cs_var == null || !Number.isFinite(cs_var)) return { key: 'view.beta_shrink.disp.unknown', cls: '' };
    if (n_assets < 2)  return { key: 'view.beta_shrink.disp.unknown', cls: '' };
    const sd = Math.sqrt(cs_var);
    if (sd < 0.10) return { key: 'view.beta_shrink.disp.tight',   cls: '' };
    if (sd < 0.30) return { key: 'view.beta_shrink.disp.moderate', cls: '' };
    if (sd < 0.60) return { key: 'view.beta_shrink.disp.wide',     cls: '' };
    return { key: 'view.beta_shrink.disp.very_wide', cls: '' };
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

export function makeDemoInput(kind = 'mixed') {
    switch (kind) {
        case 'mixed': {
            const rand = lcg(42n);
            const n = 100;
            const market = Array.from({ length: n }, () => 0.01 * gaussian(rand));
            const assets = [
                { symbol: 'LOW',     asset_returns: market.map(x => x * 0.5 + 0.002 * gaussian(rand)) },
                { symbol: 'MARKET',  asset_returns: market.map(x => x * 1.0 + 0.002 * gaussian(rand)) },
                { symbol: 'HIGH',    asset_returns: market.map(x => x * 1.5 + 0.002 * gaussian(rand)) },
                { symbol: 'LEVERED', asset_returns: market.map(x => x * 2.5 + 0.005 * gaussian(rand)) },
            ];
            return { assets, market_returns: market };
        }
        case 'tight-vs-noisy': {
            const rand = lcg(7n);
            const n = 100;
            const market = Array.from({ length: n }, () => 0.01 * gaussian(rand));
            const assets = [
                { symbol: 'TIGHT',  asset_returns: market.map(x => x * 1.5 + 0.0005 * gaussian(rand)) },
                { symbol: 'NOISY',  asset_returns: market.map(x => x * 1.5 + 0.05 * gaussian(rand)) },
            ];
            return { assets, market_returns: market };
        }
        case 'all-similar': {
            // All betas near 1.0 → cs_var small, w small for everyone.
            const rand = lcg(11n);
            const n = 100;
            const market = Array.from({ length: n }, () => 0.01 * gaussian(rand));
            const assets = [
                { symbol: 'A', asset_returns: market.map(x => x * 1.0 + 0.002 * gaussian(rand)) },
                { symbol: 'B', asset_returns: market.map(x => x * 1.02 + 0.002 * gaussian(rand)) },
                { symbol: 'C', asset_returns: market.map(x => x * 0.98 + 0.002 * gaussian(rand)) },
                { symbol: 'D', asset_returns: market.map(x => x * 1.01 + 0.002 * gaussian(rand)) },
            ];
            return { assets, market_returns: market };
        }
        case 'sector-mix': {
            const rand = lcg(13n);
            const n = 80;
            const market = Array.from({ length: n }, () => 0.012 * gaussian(rand));
            const assets = [
                { symbol: 'UTIL',  asset_returns: market.map(x => x * 0.4 + 0.003 * gaussian(rand)) },
                { symbol: 'STAPL', asset_returns: market.map(x => x * 0.6 + 0.003 * gaussian(rand)) },
                { symbol: 'INDUS', asset_returns: market.map(x => x * 1.0 + 0.004 * gaussian(rand)) },
                { symbol: 'TECH',  asset_returns: market.map(x => x * 1.4 + 0.005 * gaussian(rand)) },
                { symbol: 'CRYPTO',asset_returns: market.map(x => x * 2.5 + 0.015 * gaussian(rand)) },
            ];
            return { assets, market_returns: market };
        }
        case 'inverse': {
            // One asset with negative beta (e.g. inverse ETF).
            const rand = lcg(21n);
            const n = 80;
            const market = Array.from({ length: n }, () => 0.012 * gaussian(rand));
            const assets = [
                { symbol: 'SPY',  asset_returns: market.map(x => x * 1.0 + 0.002 * gaussian(rand)) },
                { symbol: 'SH',   asset_returns: market.map(x => x * -1.0 + 0.002 * gaussian(rand)) },
                { symbol: 'SDS',  asset_returns: market.map(x => x * -2.0 + 0.005 * gaussian(rand)) },
            ];
            return { assets, market_returns: market };
        }
        case 'short-series': {
            const rand = lcg(33n);
            const n = 10;    // just above MIN_OBS=5
            const market = Array.from({ length: n }, () => 0.01 * gaussian(rand));
            const assets = [
                { symbol: 'A', asset_returns: market.map(x => x * 1.0 + 0.005 * gaussian(rand)) },
                { symbol: 'B', asset_returns: market.map(x => x * 1.3 + 0.005 * gaussian(rand)) },
            ];
            return { assets, market_returns: market };
        }
        case 'mismatched': {
            // One asset has wrong length → server skips it; we mirror that.
            const rand = lcg(57n);
            const n = 30;
            const market = Array.from({ length: n }, () => 0.01 * gaussian(rand));
            const assets = [
                { symbol: 'OK',  asset_returns: market.map(x => x * 1.0 + 0.002 * gaussian(rand)) },
                { symbol: 'BAD', asset_returns: market.slice(0, 10).map(x => x * 1.0) },
            ];
            return { assets, market_returns: market };
        }
        case 'single': {
            // Single asset → cs_var = 0 → w = 0 → all shrunk = prior (= beta_ols).
            const rand = lcg(99n);
            const n = 50;
            const market = Array.from({ length: n }, () => 0.01 * gaussian(rand));
            const assets = [
                { symbol: 'ONLY', asset_returns: market.map(x => x * 1.2 + 0.003 * gaussian(rand)) },
            ];
            return { assets, market_returns: market };
        }
        default: return makeDemoInput('mixed');
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
