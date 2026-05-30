// Stop-loss best-of helpers shared by view + vitest.
//
// Backend body shape: { trades: [{entry, mae, mfe, actual_exit}, ...],
// candidates: [{method, value, atr}, ...], side_long: bool }.
// Returns MethodResult[] (one per candidate) — sorted by caller.

import { t } from './i18n.js';

const TOKEN_DELIM = /[\s,]+/;

// Four-token-per-line `entry mae mfe actual_exit`.
//   MAE / MFE are stored as POSITIVE excursions from entry (per backend).
//   actual_exit is the exit price (not P&L).
export function parseTradeBlob(text) {
    const trades = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { trades, errors: [{ line_no: 0, raw: '', message: t('common.parse.input_must_be_string') }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.trim();
        if (!s || s.startsWith('#')) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 4) {
            errors.push({ line_no: i + 1, raw, message: `expected 4 tokens (entry mae mfe actual_exit), got ${parts.length}` });
            continue;
        }
        const entry = Number(parts[0]);
        const mae = Number(parts[1]);
        const mfe = Number(parts[2]);
        const actual_exit = Number(parts[3]);
        if (!Number.isFinite(entry) || entry <= 0) {
            errors.push({ line_no: i + 1, raw, message: `entry must be > 0` });
            continue;
        }
        if (!Number.isFinite(mae) || mae < 0) {
            errors.push({ line_no: i + 1, raw, message: `mae (excursion magnitude) must be ≥ 0` });
            continue;
        }
        if (!Number.isFinite(mfe) || mfe < 0) {
            errors.push({ line_no: i + 1, raw, message: `mfe (excursion magnitude) must be ≥ 0` });
            continue;
        }
        if (!Number.isFinite(actual_exit) || actual_exit <= 0) {
            errors.push({ line_no: i + 1, raw, message: `actual_exit must be > 0` });
            continue;
        }
        trades.push({ entry, mae, mfe, actual_exit });
    }
    return { trades, errors };
}

export const VALID_METHODS = new Set(['none', 'fixed_dollar', 'fixed_pct', 'atr_multiple']);

export function validateInputs(trades, candidates, sideLong) {
    if (!Array.isArray(trades) || trades.length === 0) return t('view.stop_loss_best_of.validate.trades_empty');
    if (!Array.isArray(candidates) || candidates.length === 0) return t('view.stop_loss_best_of.validate.candidates_empty');
    for (const c of candidates) {
        if (!c || !VALID_METHODS.has(c.method)) return t('view.stop_loss_best_of.validate.method', { m: c?.method });
        if (!Number.isFinite(c.value) || c.value < 0) return t('view.stop_loss_best_of.validate.value');
        if (!Number.isFinite(c.atr) || c.atr < 0) return t('view.stop_loss_best_of.validate.atr');
    }
    if (typeof sideLong !== 'boolean') return t('view.stop_loss_best_of.validate.side_long');
    return null;
}

export function buildBody(trades, candidates, sideLong) {
    return { trades, candidates, side_long: sideLong };
}

// Friendly per-method display label.
const METHOD_BADGES = {
    none:         { key: 'none',         cls: 'neg' },
    fixed_dollar: { key: 'fixed_dollar', cls: '' },
    fixed_pct:    { key: 'fixed_pct',    cls: '' },
    atr_multiple: { key: 'atr_multiple', cls: 'pos' },
};
export function methodBadge(m) {
    const x = METHOD_BADGES[m];
    if (!x) return { label: String(m || '—'), cls: '', desc: '' };
    return {
        label: t(`view.stop_loss_best_of.method.${x.key}.label`),
        cls: x.cls,
        desc: t(`view.stop_loss_best_of.method.${x.key}.desc`),
    };
}

// Renders a human-friendly candidate description combining method +
// value (e.g., "2.5 × ATR(1.0)", "0.5% fixed").
export function describeCandidate(c) {
    if (!c) return '—';
    const b = methodBadge(c.method);
    switch (c.method) {
        case 'none':         return b.label;
        case 'fixed_dollar': return t('view.stop_loss_best_of.summary.fixed_dollar', { value: c.value.toFixed(2) });
        case 'fixed_pct':    return t('view.stop_loss_best_of.summary.fixed_pct', { value: (c.value * 100).toFixed(2) });
        case 'atr_multiple': return `${c.value.toFixed(2)} × ATR(${c.atr.toFixed(2)})`;
        default:             return b.label;
    }
}

// Picks the row with the highest total_realized. Used by the "Best
// method" summary card.
export function bestByTotal(results) {
    if (!Array.isArray(results) || !results.length) return null;
    return results.reduce((best, r) =>
        (!best || (r.total_realized || 0) > (best.total_realized || 0)) ? r : best, null);
}

// Picks the row with the highest avg_realized — a complementary view
// (e.g., when one method had outsized wins but many small losses).
export function bestByAvg(results) {
    if (!Array.isArray(results) || !results.length) return null;
    return results.reduce((best, r) =>
        (!best || (r.avg_realized || 0) > (best.avg_realized || 0)) ? r : best, null);
}

// Default candidate ladder — covers all 4 methods with multiple values
// each so the comparison is meaningful out of the box.
export function defaultCandidates(atr = 1.0) {
    return [
        { method: 'none',         value: 0,     atr: 0 },
        { method: 'fixed_dollar', value: 1.0,   atr: 0 },
        { method: 'fixed_dollar', value: 2.0,   atr: 0 },
        { method: 'fixed_pct',    value: 0.005, atr: 0 },
        { method: 'fixed_pct',    value: 0.01,  atr: 0 },
        { method: 'fixed_pct',    value: 0.02,  atr: 0 },
        { method: 'atr_multiple', value: 1.0,   atr },
        { method: 'atr_multiple', value: 2.0,   atr },
        { method: 'atr_multiple', value: 3.0,   atr },
    ];
}

// 20-trade deterministic demo with varied MAE/MFE so the candidates
// genuinely differ. Engineered so a 2×ATR stop typically beats both
// tight 0.5% and loose unlimited.
export function makeDemoTrades(seed = 1) {
    let s = seed;
    const rand = () => { s = (s * 1664525 + 1013904223) | 0; return ((s >>> 0) / 0xffffffff); };
    const out = new Array(20);
    for (let i = 0; i < 20; i++) {
        const entry = 100 + i * 0.5;
        // Half winners, half losers, with realistic MAE/MFE excursions.
        const isWinner = i % 2 === 0;
        if (isWinner) {
            out[i] = {
                entry,
                mae: 0.4 + rand() * 0.6,    // 0.4-1.0 dollar drawdown
                mfe: 2.0 + rand() * 1.5,    // 2.0-3.5 dollar peak
                actual_exit: entry + 1.5 + rand() * 0.5,  // exited captured 1.5-2.0
            };
        } else {
            out[i] = {
                entry,
                mae: 1.5 + rand() * 1.5,    // 1.5-3.0 dollar drawdown
                mfe: 0.3 + rand() * 0.4,    // 0.3-0.7 small peak
                actual_exit: entry - 1.0 - rand() * 0.5,  // exited at -1.0 to -1.5
            };
        }
    }
    return out;
}

export function fmtN(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtSigned(v) {
    if (!Number.isFinite(v)) return '—';
    const sign = v >= 0 ? '+' : '';
    return sign + v.toFixed(2);
}
