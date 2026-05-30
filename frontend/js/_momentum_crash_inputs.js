// Momentum Crash Protection (Daniel & Moskowitz 2016) helpers.
//
// Backend body: { momentum_returns: number[], vol_lookback, target_annualized_vol,
//   periods_per_year, max_leverage, crash_filter_lookback, crash_filter_threshold_pct }
// Returns: { managed_returns, leverages, crash_filter_active, mean_leverage, n_observations } | null
//
// Model: w_t = min(target_vol / forecast_vol_t, max_leverage)
// Crash filter: if trailing-cumret over crash_lookback < threshold, w_t = 0.

import { t } from './i18n.js';

export const DEFAULT_INPUTS = {
    momentum_returns: [],
    vol_lookback: 60,
    target_annualized_vol: 0.15,
    periods_per_year: 252,
    max_leverage: 4.0,
    crash_filter_lookback: 22,
    crash_filter_threshold_pct: -0.20,
};

export function validateInputs(input) {
    if (!Array.isArray(input.momentum_returns))             return t('view.momentum_crash.validate.returns_array');
    for (let i = 0; i < input.momentum_returns.length; i++) {
        if (!Number.isFinite(input.momentum_returns[i]))    return t('view.momentum_crash.validate.return_finite', { i });
    }
    if (!Number.isInteger(input.vol_lookback) || input.vol_lookback < 5)
                                                              return t('view.momentum_crash.validate.vol_lookback');
    if (!Number.isFinite(input.target_annualized_vol) || input.target_annualized_vol <= 0)
                                                              return t('view.momentum_crash.validate.target_vol');
    if (!Number.isFinite(input.periods_per_year) || input.periods_per_year <= 0)
                                                              return t('view.momentum_crash.validate.periods_year');
    if (!Number.isFinite(input.max_leverage) || input.max_leverage <= 0)
                                                              return t('view.momentum_crash.validate.max_leverage');
    if (!Number.isInteger(input.crash_filter_lookback) || input.crash_filter_lookback < 1)
                                                              return t('view.momentum_crash.validate.crash_lookback');
    if (!Number.isFinite(input.crash_filter_threshold_pct))
                                                              return t('view.momentum_crash.validate.crash_threshold');
    const required = Math.max(input.vol_lookback, input.crash_filter_lookback) + 1;
    if (input.momentum_returns.length < required)
                                                              return t('view.momentum_crash.validate.returns_min', { n: required });
    return null;
}

export function buildBody(input) {
    return {
        momentum_returns:              input.momentum_returns,
        vol_lookback:                  input.vol_lookback,
        target_annualized_vol:         input.target_annualized_vol,
        periods_per_year:              input.periods_per_year,
        max_leverage:                  input.max_leverage,
        crash_filter_lookback:         input.crash_filter_lookback,
        crash_filter_threshold_pct:    input.crash_filter_threshold_pct,
    };
}

// Pure-JS mirror of crates/traderview-core/src/momentum_crash_protection.rs::manage.
export function localManage(
    momentum_returns, vol_lookback, target_annualized_vol, periods_per_year,
    max_leverage, crash_filter_lookback, crash_filter_threshold_pct,
) {
    const n = momentum_returns.length;
    const lookback = Math.max(vol_lookback, crash_filter_lookback);
    if (n < lookback + 1) return null;
    if (vol_lookback < 5) return null;
    if (crash_filter_lookback === 0) return null;
    if (!Number.isFinite(target_annualized_vol) || target_annualized_vol <= 0) return null;
    if (!Number.isFinite(periods_per_year) || periods_per_year <= 0) return null;
    if (!Number.isFinite(max_leverage) || max_leverage <= 0) return null;
    if (!Number.isFinite(crash_filter_threshold_pct)) return null;
    for (const v of momentum_returns) if (!Number.isFinite(v)) return null;
    const target_vol_period = target_annualized_vol / Math.sqrt(periods_per_year);
    const managed = new Array(n).fill(null);
    const leverages = new Array(n).fill(null);
    const crash_active = new Array(n).fill(null);
    let lev_sum = 0, lev_count = 0;
    for (let i = lookback; i < n; i++) {
        // vol window: i - vol_lookback .. i (exclusive of i)
        let sum = 0;
        for (let k = i - vol_lookback; k < i; k++) sum += momentum_returns[k];
        const mean = sum / vol_lookback;
        let var_ = 0;
        for (let k = i - vol_lookback; k < i; k++) {
            const d = momentum_returns[k] - mean;
            var_ += d * d;
        }
        var_ /= vol_lookback;
        const vol = Math.sqrt(Math.max(0, var_));
        const raw_lev = vol > 0 ? Math.min(target_vol_period / vol, max_leverage) : max_leverage;
        // Crash filter window: i - crash_filter_lookback .. i (exclusive)
        let cum = 1;
        for (let k = i - crash_filter_lookback; k < i; k++) cum *= (1 + momentum_returns[k]);
        const cum_ret = cum - 1;
        const crash_on = cum_ret < crash_filter_threshold_pct;
        const lev = crash_on ? 0 : raw_lev;
        managed[i] = lev * momentum_returns[i];
        leverages[i] = lev;
        crash_active[i] = crash_on;
        if (lev > 0) { lev_sum += lev; lev_count++; }
    }
    return {
        managed_returns:     managed,
        leverages,
        crash_filter_active: crash_active,
        mean_leverage:       lev_count > 0 ? lev_sum / lev_count : 0,
        n_observations:      n,
    };
}

// Parse one return per line/token; blanks + # comments + commas + whitespace handled.
// Tokens can be raw decimal (0.012) or pct-suffixed (1.2%); pct is converted to decimal.
export function parseReturnsBlob(blob) {
    const out = { returns: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: t('common.parse.input_must_be_string') });
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
        const isPct = raw.endsWith('%');
        const stripped = isPct ? raw.slice(0, -1) : raw;
        const n = Number(stripped);
        if (!Number.isFinite(n)) {
            out.errors.push({ line_no: i + 1, message: `token "${raw}" not finite` });
            continue;
        }
        out.returns.push(isPct ? n / 100 : n);
    }
    return out;
}

export function returnsToBlob(returns) {
    return returns.join('\n');
}

// Aggregate stats from the report.
export function summarize(report) {
    if (!report) return { populated: 0, crash_bars: 0, crash_frac: NaN, max_lev: NaN, total_managed: NaN, total_raw: NaN };
    let populated = 0, crash_bars = 0, max_lev = -Infinity;
    let cum_managed = 1, cum_raw = 1;
    for (let i = 0; i < report.leverages.length; i++) {
        const lev = report.leverages[i];
        if (lev != null) {
            populated++;
            if (lev > max_lev) max_lev = lev;
        }
        if (report.crash_filter_active[i] === true) crash_bars++;
        const m = report.managed_returns[i];
        if (m != null) cum_managed *= (1 + m);
    }
    // Raw cum return uses ALL bars (including warmup) — informative comparison.
    for (let i = 0; i < report.n_observations; i++) {
        // No way to access input from here — caller will compute total_raw separately.
    }
    return {
        populated,
        crash_bars,
        crash_frac: populated > 0 ? crash_bars / populated : NaN,
        max_lev: Number.isFinite(max_lev) ? max_lev : NaN,
        total_managed: cum_managed - 1,
        total_raw: NaN,
    };
}

// Compute raw cum return from the input returns (used for managed-vs-raw comparison).
export function cumReturn(returns) {
    if (!Array.isArray(returns) || returns.length === 0) return NaN;
    let cum = 1;
    for (const r of returns) cum *= (1 + r);
    return cum - 1;
}

// Drawdown of a cumulative return series.
export function maxDrawdown(returns) {
    if (!Array.isArray(returns) || returns.length === 0) return NaN;
    let cum = 1, peak = 1, mdd = 0;
    for (const r of returns) {
        cum *= (1 + r);
        if (cum > peak) peak = cum;
        const dd = (cum - peak) / peak;
        if (dd < mdd) mdd = dd;
    }
    return mdd;
}

export function leverageBadge(meanLev, maxLeverage) {
    if (!Number.isFinite(meanLev) || meanLev <= 0) return { key: 'view.mcp.badge.off',    cls: 'neg' };
    const cap = Number.isFinite(maxLeverage) && maxLeverage > 0 ? maxLeverage : 1;
    if (meanLev < 0.5)            return { key: 'view.mcp.badge.defensive', cls: '' };
    if (meanLev < cap * 0.5)      return { key: 'view.mcp.badge.balanced',  cls: 'pos' };
    if (meanLev < cap * 0.9)      return { key: 'view.mcp.badge.aggressive', cls: '' };
    return { key: 'view.mcp.badge.maxed', cls: 'neg' };
}

export function crashBadge(crash_frac) {
    if (!Number.isFinite(crash_frac)) return { key: 'view.mcp.crash.unknown', cls: '' };
    if (crash_frac === 0)              return { key: 'view.mcp.crash.none',    cls: 'pos' };
    if (crash_frac < 0.05)             return { key: 'view.mcp.crash.brief',   cls: '' };
    if (crash_frac < 0.20)             return { key: 'view.mcp.crash.frequent', cls: 'neg' };
    return { key: 'view.mcp.crash.dominant', cls: 'neg' };
}

// Synthetic demos. Deterministic LCG so tests are stable.
export function makeDemoInput(kind = 'normal-regime') {
    switch (kind) {
        case 'normal-regime':   return synth({ n: 200, mu: 0.0005, sigma: 0.005, seed: 42n });
        case 'low-vol':         return synth({ n: 200, mu: 0.0001, sigma: 0.0005, seed: 11n });
        case 'crash-event':     return crashSeries();
        case 'persistent-crash': return persistentCrash();
        case 'high-vol':        return synth({ n: 200, mu: 0.0001, sigma: 0.02, seed: 99n });
        case 'mixed-regime':    return mixedRegime();
        case 'short-lookback':  return { ...synth({ n: 80, mu: 0.0003, sigma: 0.004, seed: 7n }),
                                          vol_lookback: 10, crash_filter_lookback: 5 };
        case 'tight-target':    return { ...synth({ n: 200, mu: 0.0005, sigma: 0.005, seed: 4n }),
                                          target_annualized_vol: 0.05 };
        default:                return makeDemoInput('normal-regime');
    }
}

function synth({ n, mu, sigma, seed }) {
    const r = [];
    let state = BigInt(7919) + BigInt(seed);
    for (let i = 0; i < n; i++) {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        const u = Number(state >> 32n) / 0xFFFFFFFF;
        r.push(mu + (u - 0.5) * sigma * 2);
    }
    return { ...DEFAULT_INPUTS, momentum_returns: r };
}

function crashSeries() {
    // 50 calm bars + 22 crash bars + 30 recovery bars.
    const calm = synth({ n: 50, mu: 0.001, sigma: 0.003, seed: 1n }).momentum_returns;
    const crash = new Array(22).fill(-0.01);
    const recovery = new Array(30).fill(0.001);
    return { ...DEFAULT_INPUTS,
             momentum_returns: [...calm, ...crash, ...recovery],
             crash_filter_threshold_pct: -0.10 };
}

function persistentCrash() {
    // 80 bars of −0.5%/bar.
    const calm = synth({ n: 30, mu: 0.0005, sigma: 0.002, seed: 2n }).momentum_returns;
    const crash = new Array(80).fill(-0.005);
    return { ...DEFAULT_INPUTS,
             momentum_returns: [...calm, ...crash],
             crash_filter_threshold_pct: -0.05 };
}

function mixedRegime() {
    const a = synth({ n: 80, mu: 0.0008, sigma: 0.005, seed: 3n }).momentum_returns;
    const b = synth({ n: 80, mu: -0.0005, sigma: 0.015, seed: 5n }).momentum_returns;
    return { ...DEFAULT_INPUTS,
             momentum_returns: [...a, ...b] };
}

export function fmtPct(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtPctSigned(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + (v * 100).toFixed(d) + '%';
}

export function fmtLev(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d) + 'x';
}

export function fmtNum(v, d = 6) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtInt(v) {
    if (!Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
