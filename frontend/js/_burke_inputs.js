// Burke Ratio (1994) helpers — return per unit of drawdown-vol.
//
// Backend body: { equity: number[], risk_free_total: f64, periods_per_year: f64 }
// Returns: {
//   burke_ratio, modified_burke_ratio, total_return,
//   n_drawdowns, sum_squared_drawdowns,
// } | null

export const DEFAULT_RISK_FREE = 0.0;
export const DEFAULT_PERIODS_PER_YEAR = 252.0;
import { t } from './i18n.js';

export const MIN_OBS = 2;

export const DEFAULT_INPUTS = {
    equity: [],
    risk_free_total: DEFAULT_RISK_FREE,
    periods_per_year: DEFAULT_PERIODS_PER_YEAR,
};

export function validateInputs(input) {
    if (!Array.isArray(input.equity))                       return t('view.burke.validate.equity_array');
    if (input.equity.length < MIN_OBS)                      return t('view.burke.validate.equity_min', { n: MIN_OBS });
    if (!Number.isFinite(input.risk_free_total))            return t('view.burke.validate.rf_finite');
    if (!Number.isFinite(input.periods_per_year) || input.periods_per_year <= 0)
                                                             return t('view.burke.validate.periods_pos');
    for (let i = 0; i < input.equity.length; i++) {
        if (!Number.isFinite(input.equity[i]))              return t('view.burke.validate.equity_not_finite', { i });
        if (input.equity[i] <= 0)                           return t('view.burke.validate.equity_pos', { i });
    }
    return null;
}

export function buildBody(input) {
    return {
        equity: input.equity.slice(),
        risk_free_total: input.risk_free_total,
        periods_per_year: input.periods_per_year,
    };
}

// Pure-JS mirror of crates/traderview-core/src/burke_ratio.rs::compute.
export function localCompute(equity, risk_free_total, periods_per_year) {
    if (equity.length < 2 || !Number.isFinite(risk_free_total)
        || !Number.isFinite(periods_per_year) || periods_per_year <= 0) return null;
    for (const v of equity) {
        if (!Number.isFinite(v) || v <= 0) return null;
    }
    const start = equity[0];
    const end = equity[equity.length - 1];
    const total_return = end / start - 1;
    const excess = total_return - risk_free_total;
    let hwm = start;
    let current_trough_dd = 0;
    const drawdowns = [];
    for (let i = 1; i < equity.length; i++) {
        const v = equity[i];
        if (v > hwm) {
            if (current_trough_dd > 0) drawdowns.push(current_trough_dd);
            hwm = v;
            current_trough_dd = 0;
        } else {
            const dd = (hwm - v) / hwm;
            if (dd > current_trough_dd) current_trough_dd = dd;
        }
    }
    if (current_trough_dd > 0) drawdowns.push(current_trough_dd);
    let sum_sq_dd = 0;
    for (const d of drawdowns) sum_sq_dd += d * d;
    const burke = sum_sq_dd > 0 ? excess / Math.sqrt(sum_sq_dd) : 0;
    const mod_burke = burke * Math.sqrt(periods_per_year);
    return {
        burke_ratio: burke,
        modified_burke_ratio: mod_burke,
        total_return,
        n_drawdowns: drawdowns.length,
        sum_squared_drawdowns: sum_sq_dd,
    };
}

// Compute per-trough drawdown list (for the per-bar table).
export function drawdownEpisodes(equity) {
    const episodes = [];
    if (!Array.isArray(equity) || equity.length < 2) return episodes;
    let hwm = equity[0];
    let hwm_idx = 0;
    let trough = equity[0];
    let trough_idx = 0;
    let current_dd = 0;
    for (let i = 1; i < equity.length; i++) {
        const v = equity[i];
        if (v > hwm) {
            if (current_dd > 0) {
                episodes.push({
                    peak_idx: hwm_idx,
                    trough_idx,
                    peak_value: hwm,
                    trough_value: trough,
                    drawdown_pct: current_dd,
                    recovery_idx: i,
                });
            }
            hwm = v;
            hwm_idx = i;
            trough = v;
            trough_idx = i;
            current_dd = 0;
        } else {
            const dd = (hwm - v) / hwm;
            if (dd > current_dd) {
                current_dd = dd;
                trough = v;
                trough_idx = i;
            }
        }
    }
    if (current_dd > 0) {
        episodes.push({
            peak_idx: hwm_idx,
            trough_idx,
            peak_value: hwm,
            trough_value: trough,
            drawdown_pct: current_dd,
            recovery_idx: null,
        });
    }
    return episodes;
}

// Parse positive-only equity series.
export function parseEquityBlob(blob) {
    const out = { equity: [], errors: [] };
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
        const v = Number(tokens[i].replace(/[\$,]/g, ''));
        if (!Number.isFinite(v) || v <= 0) {
            out.errors.push({ line_no: i + 1, message: `token "${tokens[i]}" must be positive finite` });
            continue;
        }
        out.equity.push(v);
    }
    return out;
}

export function equityToBlob(equity) {
    return equity.join('\n');
}

// 5-tier modified-Burke verdict — standardized industry benchmark.
export function ratioBadge(mod_burke) {
    if (mod_burke == null || !Number.isFinite(mod_burke)) {
        return { key: 'view.burke.ratio.unknown', cls: '' };
    }
    if (mod_burke >= 2)   return { key: 'view.burke.ratio.exceptional', cls: 'pos' };
    if (mod_burke >= 1)   return { key: 'view.burke.ratio.strong',      cls: 'pos' };
    if (mod_burke >= 0.5) return { key: 'view.burke.ratio.moderate',    cls: '' };
    if (mod_burke >= 0)   return { key: 'view.burke.ratio.weak',        cls: 'neg' };
    return { key: 'view.burke.ratio.negative', cls: 'neg' };
}

// Drawdown intensity from sum of squared DDs.
export function ddBadge(sum_sq_dd, n_drawdowns) {
    if (sum_sq_dd == null || !Number.isFinite(sum_sq_dd) || n_drawdowns == null) {
        return { key: 'view.burke.dd.unknown', cls: '' };
    }
    if (sum_sq_dd === 0)          return { key: 'view.burke.dd.none',    cls: 'pos' };
    const rms = Math.sqrt(sum_sq_dd / Math.max(1, n_drawdowns));
    if (rms < 0.02)               return { key: 'view.burke.dd.tiny',    cls: 'pos' };
    if (rms < 0.05)               return { key: 'view.burke.dd.mild',    cls: '' };
    if (rms < 0.15)               return { key: 'view.burke.dd.notable', cls: 'neg' };
    if (rms < 0.30)               return { key: 'view.burke.dd.severe',  cls: 'neg' };
    return { key: 'view.burke.dd.catastrophic', cls: 'neg' };
}

// Return-vs-rf verdict.
export function excessBadge(total_return, risk_free_total) {
    if (!Number.isFinite(total_return) || !Number.isFinite(risk_free_total)) {
        return { key: 'view.burke.excess.unknown', cls: '' };
    }
    const excess = total_return - risk_free_total;
    if (excess >= 0.20)  return { key: 'view.burke.excess.strong_alpha', cls: 'pos' };
    if (excess >= 0.05)  return { key: 'view.burke.excess.alpha',        cls: 'pos' };
    if (excess >= -0.05) return { key: 'view.burke.excess.market',       cls: '' };
    if (excess >= -0.20) return { key: 'view.burke.excess.underperform', cls: 'neg' };
    return { key: 'view.burke.excess.severe_underperform', cls: 'neg' };
}

export function summarizeEquity(equity) {
    if (!Array.isArray(equity) || equity.length === 0) {
        return { count: 0, start: NaN, end: NaN, min: NaN, max: NaN, peak_to_trough: NaN };
    }
    let mx = -Infinity, mn = Infinity;
    for (const v of equity) {
        if (v > mx) mx = v;
        if (v < mn) mn = v;
    }
    return {
        count: equity.length,
        start: equity[0],
        end: equity[equity.length - 1],
        min: Number.isFinite(mn) ? mn : NaN,
        max: Number.isFinite(mx) ? mx : NaN,
        peak_to_trough: (mx - mn) / mx,
    };
}

function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF;
    };
}

export function makeDemoInput(kind = 'steady-growth') {
    switch (kind) {
        case 'steady-growth': {
            const rand = lcg(42n);
            const eq = [100];
            for (let i = 1; i < 252; i++) eq.push(eq[i - 1] * (1 + 0.0005 + (rand() - 0.5) * 0.01));
            return { equity: eq, risk_free_total: 0.02, periods_per_year: 252 };
        }
        case 'high-sharpe': {
            const rand = lcg(7n);
            const eq = [100];
            for (let i = 1; i < 252; i++) eq.push(eq[i - 1] * (1 + 0.001 + (rand() - 0.5) * 0.005));
            return { equity: eq, risk_free_total: 0.02, periods_per_year: 252 };
        }
        case 'volatile-uptrend': {
            const rand = lcg(11n);
            const eq = [100];
            for (let i = 1; i < 252; i++) eq.push(eq[i - 1] * (1 + 0.0005 + (rand() - 0.5) * 0.03));
            return { equity: eq, risk_free_total: 0.02, periods_per_year: 252 };
        }
        case 'deep-drawdown': {
            // 50% drawdown mid-period then full recovery.
            const eq = [100];
            for (let i = 1; i < 100; i++) eq.push(eq[i - 1] * 1.005);
            for (let i = 100; i < 150; i++) eq.push(eq[i - 1] * 0.98);
            for (let i = 150; i < 252; i++) eq.push(eq[i - 1] * 1.012);
            return { equity: eq, risk_free_total: 0.02, periods_per_year: 252 };
        }
        case 'multi-drawdowns': {
            // Several distinct DD episodes.
            const rand = lcg(13n);
            const eq = [100];
            for (let i = 1; i < 252; i++) {
                const r = (rand() - 0.4) * 0.025;
                eq.push(Math.max(0.01, eq[i - 1] * (1 + r)));
            }
            return { equity: eq, risk_free_total: 0.02, periods_per_year: 252 };
        }
        case 'losing-strategy': {
            const rand = lcg(21n);
            const eq = [100];
            for (let i = 1; i < 252; i++) eq.push(Math.max(0.01, eq[i - 1] * (1 - 0.001 + (rand() - 0.5) * 0.01)));
            return { equity: eq, risk_free_total: 0.02, periods_per_year: 252 };
        }
        case 'monthly': {
            // Monthly bars instead of daily.
            const rand = lcg(33n);
            const eq = [100];
            for (let i = 1; i < 60; i++) eq.push(eq[i - 1] * (1 + 0.005 + (rand() - 0.5) * 0.03));
            return { equity: eq, risk_free_total: 0.02, periods_per_year: 12 };
        }
        case 'one-big-dd': {
            // Single deep DD that hasn't recovered (open-ended).
            const eq = [100];
            for (let i = 1; i < 200; i++) eq.push(eq[i - 1] * 1.003);
            for (let i = 200; i < 252; i++) eq.push(eq[i - 1] * 0.99);
            return { equity: eq, risk_free_total: 0.02, periods_per_year: 252 };
        }
        default: return makeDemoInput('steady-growth');
    }
}

export function fmtRatio(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtRatioSigned(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toFixed(d);
}

export function fmtPct(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtPctSigned(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + (v * 100).toFixed(d) + '%';
}

export function fmtPrice(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
