// Effective + realized spread analyzer helpers (Lee-Ready / Bessembinder).
//
// Backend body: { observations: [{trade_price, current_mid, delayed_mid,
//   quoted_spread, direction: 'buy'|'sell'}, ...] }
// Returns: { avg_quoted_spread, avg_effective_spread, avg_realized_spread,
//   avg_price_impact, effective_to_quoted_ratio, n_observations } | null
//
// effective_spread  = 2 · D · (trade_price − current_mid)
// realized_spread   = 2 · D · (trade_price − delayed_mid)
// price_impact      = effective − realized          (adverse selection)
// effective/quoted ratio: < 1 = price improvement, > 1 = trades-through

import { t } from './i18n.js';

export const DIRECTIONS = ['buy', 'sell'];

export const DEFAULT_INPUTS = {
    observations: [],
};

export function validateInputs(input) {
    if (!Array.isArray(input.observations))                            return t('view.effective_spread.validate.obs_array');
    if (input.observations.length === 0)                                return t('view.effective_spread.validate.obs_empty');
    for (let i = 0; i < input.observations.length; i++) {
        const o = input.observations[i];
        if (!o || typeof o !== 'object')                               return t('view.effective_spread.validate.obs_object', { i });
        if (!DIRECTIONS.includes(o.direction))                         return t('view.effective_spread.validate.direction', { i });
        if (!Number.isFinite(o.trade_price))                           return t('view.effective_spread.validate.trade_finite', { i });
        if (!Number.isFinite(o.current_mid))                           return t('view.effective_spread.validate.cur_mid_finite', { i });
        if (!Number.isFinite(o.delayed_mid))                           return t('view.effective_spread.validate.del_mid_finite', { i });
        if (!Number.isFinite(o.quoted_spread))                         return t('view.effective_spread.validate.spread_finite', { i });
    }
    return null;
}

export function buildBody(input) {
    return {
        observations: input.observations.map(o => ({
            trade_price:   o.trade_price,
            current_mid:   o.current_mid,
            delayed_mid:   o.delayed_mid,
            quoted_spread: o.quoted_spread,
            direction:     o.direction,
        })),
    };
}

// Pure-JS mirror of crates/traderview-core/src/effective_spread.rs::analyze.
// Returns null when zero valid observations remain after per-row filtering.
export function localAnalyze(observations) {
    if (!Array.isArray(observations) || observations.length === 0) return null;
    let sum_q = 0, sum_eff = 0, sum_real = 0, count = 0;
    for (const o of observations) {
        if (!Number.isFinite(o.trade_price)   || o.trade_price   <= 0) continue;
        if (!Number.isFinite(o.current_mid)   || o.current_mid   <= 0) continue;
        if (!Number.isFinite(o.delayed_mid)   || o.delayed_mid   <= 0) continue;
        if (!Number.isFinite(o.quoted_spread) || o.quoted_spread <  0) continue;
        const d = o.direction === 'buy' ? 1 : o.direction === 'sell' ? -1 : 0;
        if (d === 0) continue;
        const eff = 2 * d * (o.trade_price - o.current_mid);
        const real = 2 * d * (o.trade_price - o.delayed_mid);
        sum_q += o.quoted_spread;
        sum_eff += eff;
        sum_real += real;
        count++;
    }
    if (count === 0) return null;
    const avg_q = sum_q / count;
    const avg_eff = sum_eff / count;
    const avg_real = sum_real / count;
    const ratio = avg_q > 0 ? avg_eff / avg_q : NaN;
    return {
        avg_quoted_spread:          avg_q,
        avg_effective_spread:       avg_eff,
        avg_realized_spread:        avg_real,
        avg_price_impact:           avg_eff - avg_real,
        effective_to_quoted_ratio:  ratio,
        n_observations:             count,
    };
}

// Parse "trade current_mid delayed_mid quoted_spread direction" per line.
// # comments + blanks ignored.
export function parseObsBlob(blob) {
    const out = { observations: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: t('common.parse.input_must_be_string') });
        return out;
    }
    const lines = blob.split('\n');
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i].split('#')[0].trim();
        if (!raw) continue;
        const toks = raw.split(/[\s,]+/).filter(t => t.length > 0);
        if (toks.length !== 5) {
            out.errors.push({ line_no: i + 1, message: 'expected 5 tokens (trade current_mid delayed_mid quoted_spread direction)' });
            continue;
        }
        const trade_price = Number(toks[0]);
        const current_mid = Number(toks[1]);
        const delayed_mid = Number(toks[2]);
        const quoted_spread = Number(toks[3]);
        const direction = toks[4].toLowerCase();
        if (![trade_price, current_mid, delayed_mid, quoted_spread].every(Number.isFinite)) {
            out.errors.push({ line_no: i + 1, message: 'non-finite token' });
            continue;
        }
        if (!DIRECTIONS.includes(direction)) {
            out.errors.push({ line_no: i + 1, message: 'direction must be buy or sell' });
            continue;
        }
        out.observations.push({ trade_price, current_mid, delayed_mid, quoted_spread, direction });
    }
    return out;
}

export function obsToBlob(obs) {
    return obs.map(o =>
        `${o.trade_price} ${o.current_mid} ${o.delayed_mid} ${o.quoted_spread} ${o.direction}`
    ).join('\n');
}

// Effective-to-quoted ratio verdict.
export function executionBadge(report) {
    if (!report || !Number.isFinite(report.effective_to_quoted_ratio))
        return { key: 'view.eff_spread.badge.unknown', cls: '' };
    const r = report.effective_to_quoted_ratio;
    if (r < 0.5)  return { key: 'view.eff_spread.badge.great_improvement', cls: 'pos' };
    if (r < 0.9)  return { key: 'view.eff_spread.badge.improvement',       cls: 'pos' };
    if (r <= 1.05) return { key: 'view.eff_spread.badge.at_quote',         cls: '' };
    if (r <= 1.5)  return { key: 'view.eff_spread.badge.adverse',          cls: 'neg' };
    return { key: 'view.eff_spread.badge.trade_through', cls: 'neg' };
}

// Adverse-selection verdict from price-impact / effective ratio.
export function adverseBadge(report) {
    if (!report || !Number.isFinite(report.avg_price_impact)
        || !Number.isFinite(report.avg_effective_spread) || report.avg_effective_spread <= 0)
        return { key: 'view.eff_spread.adv.unknown', cls: '' };
    const r = report.avg_price_impact / report.avg_effective_spread;
    if (r <= 0)   return { key: 'view.eff_spread.adv.lp_wins',     cls: 'pos' };
    if (r < 0.25) return { key: 'view.eff_spread.adv.low',         cls: 'pos' };
    if (r < 0.6)  return { key: 'view.eff_spread.adv.moderate',    cls: '' };
    if (r < 1.0)  return { key: 'view.eff_spread.adv.high',        cls: 'neg' };
    return { key: 'view.eff_spread.adv.extreme', cls: 'neg' };
}

// Per-observation enrichment (used for the table view).
export function enrich(o) {
    const d = o.direction === 'buy' ? 1 : -1;
    const eff = 2 * d * (o.trade_price - o.current_mid);
    const real = 2 * d * (o.trade_price - o.delayed_mid);
    const impact = eff - real;
    return { ...o, effective_spread: eff, realized_spread: real, price_impact: impact };
}

// Synthetic demos.
export function makeDemoInput(kind = 'at-quote') {
    switch (kind) {
        case 'at-quote': {
            // 8 trades at exactly bid or ask, no adverse selection.
            return { observations: [
                obs(100.05, 100.00, 100.00, 0.10, 'buy'),
                obs(99.95,  100.00, 100.00, 0.10, 'sell'),
                obs(100.05, 100.00, 100.00, 0.10, 'buy'),
                obs(99.95,  100.00, 100.00, 0.10, 'sell'),
                obs(100.05, 100.00, 100.00, 0.10, 'buy'),
                obs(99.95,  100.00, 100.00, 0.10, 'sell'),
                obs(100.05, 100.00, 100.00, 0.10, 'buy'),
                obs(99.95,  100.00, 100.00, 0.10, 'sell'),
            ]};
        }
        case 'price-improvement': {
            // Trades INSIDE the spread (effective < quoted).
            return { observations: [
                obs(100.02, 100.00, 100.00, 0.10, 'buy'),
                obs(99.98,  100.00, 100.00, 0.10, 'sell'),
                obs(100.03, 100.00, 100.00, 0.10, 'buy'),
                obs(99.97,  100.00, 100.00, 0.10, 'sell'),
            ]};
        }
        case 'adverse-selection': {
            // Buys followed by upward mid drift (informed flow).
            return { observations: [
                obs(100.05, 100.00, 100.10, 0.10, 'buy'),
                obs(100.05, 100.00, 100.08, 0.10, 'buy'),
                obs(99.95,  100.00, 99.90,  0.10, 'sell'),
                obs(99.95,  100.00, 99.92,  0.10, 'sell'),
            ]};
        }
        case 'lp-wins': {
            // Mid drifts AGAINST the trade direction (uninformed flow → LP profits).
            return { observations: [
                obs(100.05, 100.00, 99.95, 0.10, 'buy'),     // buy → mid drops → LP wins
                obs(99.95,  100.00, 100.05, 0.10, 'sell'),
            ]};
        }
        case 'trade-through': {
            // Trade prices BEYOND the quote — effective > quoted.
            return { observations: [
                obs(100.10, 100.00, 100.00, 0.10, 'buy'),
                obs(99.90,  100.00, 100.00, 0.10, 'sell'),
            ]};
        }
        case 'mixed-quality': {
            // Mix of price-improvement + at-quote + trade-through.
            return { observations: [
                obs(100.02, 100.00, 100.01, 0.10, 'buy'),
                obs(100.05, 100.00, 100.03, 0.10, 'buy'),
                obs(100.08, 100.00, 100.06, 0.10, 'buy'),
                obs(99.98,  100.00, 99.99,  0.10, 'sell'),
                obs(99.95,  100.00, 99.97,  0.10, 'sell'),
                obs(99.92,  100.00, 99.94,  0.10, 'sell'),
            ]};
        }
        case 'tight-market': {
            // Penny-spread environment (1 cent on $100).
            return { observations: [
                obs(100.005, 100.000, 100.000, 0.01, 'buy'),
                obs(99.995,  100.000, 100.000, 0.01, 'sell'),
                obs(100.005, 100.000, 100.000, 0.01, 'buy'),
                obs(99.995,  100.000, 100.000, 0.01, 'sell'),
            ]};
        }
        case 'large-tick': {
            // Wide-spread instrument (50 cent on $100).
            return { observations: [
                obs(100.25, 100.00, 100.00, 0.50, 'buy'),
                obs(99.75,  100.00, 100.00, 0.50, 'sell'),
                obs(100.20, 100.00, 100.05, 0.50, 'buy'),
                obs(99.80,  100.00, 99.95,  0.50, 'sell'),
            ]};
        }
        default:
            return makeDemoInput('at-quote');
    }
}

function obs(trade_price, current_mid, delayed_mid, quoted_spread, direction) {
    return { trade_price, current_mid, delayed_mid, quoted_spread, direction };
}

export function fmtUSD(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-' : '';
    return sign + '$' + Math.abs(v).toFixed(d);
}

export function fmtUSDSigned(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '-') + '$' + Math.abs(v).toFixed(d);
}

export function fmtBps(spread, ref_price, d = 2) {
    if (!Number.isFinite(spread) || !Number.isFinite(ref_price) || ref_price <= 0) return '—';
    return ((spread / ref_price) * 10_000).toFixed(d) + ' bps';
}

export function fmtRatio(v, d = 3) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtInt(v) {
    if (!Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}

export function dirLabelKey(dir) {
    if (dir === 'buy')  return 'view.eff_spread.dir.buy';
    if (dir === 'sell') return 'view.eff_spread.dir.sell';
    return 'view.eff_spread.dir.unknown';
}
