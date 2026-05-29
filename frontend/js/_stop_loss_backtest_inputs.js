// Single-method stop-loss backtester helpers.
//
// Backend body: { trades: TradeOutcome[], params: StopParams, side_long: bool }.
// TradeOutcome = { entry, mae, mfe, actual_exit } (all f64).
// StopParams   = { method, value, atr } (method snake_case enum).
// Returns: MethodResult { method, value, total_realized,
//                         stopped_out_count, winning_trades, avg_realized }.
//
// Stop-price formulas (long side, mirror = sign-flip for shorts):
//   none         → -∞ (never hits)
//   fixed_dollar → entry − value
//   fixed_pct    → entry × (1 − value)
//   atr_multiple → entry − value × atr
//
// Hit detection: stop hit iff trade's MAE breaches the stop level.
// Realized when stopped: stop − entry (long) / entry − stop (short).
// Realized when not stopped: actual_exit − entry (long) / entry − actual_exit (short).

import { t as tr } from './i18n.js';

const TOKEN_DELIM = /[\s,]+/;
export const METHODS = ['none', 'fixed_dollar', 'fixed_pct', 'atr_multiple'];

export const DEFAULT_PARAMS = { method: 'fixed_pct', value: 0.02, atr: 0 };
export const DEFAULT_SIDE_LONG = true;

// "<entry> <mae> <mfe> <actual_exit>" per line. mae + mfe are POSITIVE
// magnitudes of adverse / favorable excursion (in dollars-from-entry).
export function parseTradeBlob(text) {
    const trades = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { trades, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = stripComment(raw).trim();
        if (!s) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 4) {
            errors.push({ line_no: i + 1, raw, message: `expected 4 tokens (entry mae mfe actual_exit), got ${parts.length}` });
            continue;
        }
        const [entry, mae, mfe, exit] = parts.map(Number);
        if ([entry, mae, mfe, exit].some(n => !Number.isFinite(n))) {
            errors.push({ line_no: i + 1, raw, message: 'tokens must be finite' });
            continue;
        }
        if (entry <= 0) {
            errors.push({ line_no: i + 1, raw, message: 'entry must be > 0' });
            continue;
        }
        if (mae < 0 || mfe < 0) {
            errors.push({ line_no: i + 1, raw, message: 'mae + mfe are positive magnitudes (≥ 0)' });
            continue;
        }
        trades.push({ entry, mae, mfe, actual_exit: exit });
    }
    return { trades, errors };
}

function stripComment(raw) {
    const i = raw.indexOf('#');
    return i >= 0 ? raw.slice(0, i) : raw;
}

export function validateInputs(trades, params, side_long) {
    if (!Array.isArray(trades)) return tr('view.stop_loss_backtest.validate.trades_array');
    if (!params || !METHODS.includes(params.method))
        return tr('view.stop_loss_backtest.validate.method', { list: METHODS.join(', ') });
    if (!Number.isFinite(params.value)) return tr('view.stop_loss_backtest.validate.value_finite');
    if (params.method === 'fixed_pct' && (params.value < 0 || params.value > 1))
        return tr('view.stop_loss_backtest.validate.pct_range');
    if (params.method === 'fixed_dollar' && params.value < 0)
        return tr('view.stop_loss_backtest.validate.dollar_negative');
    if (!Number.isFinite(params.atr) || params.atr < 0)
        return tr('view.stop_loss_backtest.validate.atr_negative');
    if (typeof side_long !== 'boolean') return tr('view.stop_loss_backtest.validate.side_long');
    return null;
}

export function buildBody(trades, params, side_long) {
    return {
        trades: trades.map(t => ({
            entry: t.entry, mae: t.mae, mfe: t.mfe, actual_exit: t.actual_exit,
        })),
        params: { method: params.method, value: params.value, atr: params.atr },
        side_long,
    };
}

// Compute stop price for a single trade given side + params.
export function stopPriceFor(trade, params, side_long) {
    switch (params.method) {
        case 'none':         return side_long ? -Infinity : Infinity;
        case 'fixed_dollar': return side_long ? trade.entry - params.value : trade.entry + params.value;
        case 'fixed_pct':    return side_long ? trade.entry * (1 - params.value) : trade.entry * (1 + params.value);
        case 'atr_multiple': {
            const off = params.value * params.atr;
            return side_long ? trade.entry - off : trade.entry + off;
        }
        default: return side_long ? -Infinity : Infinity;
    }
}

// Pure-JS mirror of crates/traderview-core/src/stop_loss_backtest.rs::simulate.
export function localSimulate(trades, params, side_long) {
    let total = 0, stopped = 0, wins = 0;
    for (const t of trades) {
        const stop = stopPriceFor(t, params, side_long);
        const maePrice = side_long ? t.entry - t.mae : t.entry + t.mae;
        const hit = side_long ? maePrice <= stop : maePrice >= stop;
        let realized;
        if (hit) {
            stopped++;
            realized = side_long ? stop - t.entry : t.entry - stop;
        } else {
            realized = side_long ? t.actual_exit - t.entry : t.entry - t.actual_exit;
        }
        if (realized > 0) wins++;
        total += realized;
    }
    const n = trades.length;
    return {
        method: params.method,
        value: params.value,
        total_realized: total,
        stopped_out_count: stopped,
        winning_trades: wins,
        avg_realized: n > 0 ? total / n : 0,
    };
}

// Verdict badge by avg_realized + win_rate combo.
export function methodBadge(report, n) {
    if (!report || n === 0) return { key: 'view.stop_loss_backtest.badge.empty', cls: '' };
    const winRate = n > 0 ? report.winning_trades / n : 0;
    if (report.avg_realized > 0 && winRate >= 0.5) return { key: 'view.stop_loss_backtest.badge.profitable', cls: 'pos' };
    if (report.avg_realized > 0)                   return { key: 'view.stop_loss_backtest.badge.marginal',   cls: '' };
    if (report.avg_realized < 0 && winRate < 0.3)  return { key: 'view.stop_loss_backtest.badge.disastrous', cls: 'neg' };
    return { key: 'view.stop_loss_backtest.badge.losing', cls: 'neg' };
}

// 6 demo presets covering each method + edge cases.
export function makeDemoTrades(kind = 'mixed') {
    switch (kind) {
        case 'mixed':
            // 10 long trades with varied MAE/MFE/exit.
            return [
                { entry: 100, mae: 3,  mfe: 8,  actual_exit: 106 },
                { entry: 102, mae: 5,  mfe: 4,  actual_exit: 99 },
                { entry: 98,  mae: 2,  mfe: 12, actual_exit: 108 },
                { entry: 101, mae: 8,  mfe: 3,  actual_exit: 96 },
                { entry: 100, mae: 1,  mfe: 15, actual_exit: 112 },
                { entry: 99,  mae: 4,  mfe: 6,  actual_exit: 103 },
                { entry: 100, mae: 6,  mfe: 4,  actual_exit: 100 },
                { entry: 103, mae: 3,  mfe: 10, actual_exit: 110 },
                { entry: 100, mae: 10, mfe: 2,  actual_exit: 92 },
                { entry: 101, mae: 2,  mfe: 7,  actual_exit: 106 },
            ];
        case 'high-mae':
            // All trades have big MAE → tight stops crush.
            return [
                { entry: 100, mae: 10, mfe: 15, actual_exit: 108 },
                { entry: 100, mae: 12, mfe: 8,  actual_exit: 105 },
                { entry: 100, mae: 15, mfe: 20, actual_exit: 115 },
                { entry: 100, mae: 8,  mfe: 25, actual_exit: 120 },
            ];
        case 'low-mae':
            // Tight MAE → tight stops don't trigger.
            return [
                { entry: 100, mae: 1, mfe: 10, actual_exit: 108 },
                { entry: 100, mae: 0.5, mfe: 8, actual_exit: 105 },
                { entry: 100, mae: 0.8, mfe: 12, actual_exit: 110 },
                { entry: 100, mae: 1.5, mfe: 6, actual_exit: 103 },
            ];
        case 'short-only':
            // For shorts the geometry inverts; mae/mfe still positive magnitudes.
            return [
                { entry: 100, mae: 4, mfe: 8, actual_exit: 92 },
                { entry: 102, mae: 6, mfe: 3, actual_exit: 105 },
                { entry: 98, mae: 2, mfe: 10, actual_exit: 88 },
                { entry: 100, mae: 5, mfe: 7, actual_exit: 93 },
            ];
        case 'all-losers':
            return Array.from({ length: 8 }, (_, i) => ({
                entry: 100, mae: 8 + (i % 3), mfe: 1, actual_exit: 92 - (i % 4),
            }));
        case 'all-winners':
            return Array.from({ length: 8 }, (_, i) => ({
                entry: 100, mae: 1 + (i % 2), mfe: 10 + i, actual_exit: 108 + (i % 3),
            }));
        default:
            return makeDemoTrades('mixed');
    }
}

export function makeDemoParams(kind = 'tight-pct') {
    switch (kind) {
        case 'none':         return { method: 'none',         value: 0,    atr: 0 };
        case 'tight-pct':    return { method: 'fixed_pct',    value: 0.02, atr: 0 };
        case 'loose-pct':    return { method: 'fixed_pct',    value: 0.05, atr: 0 };
        case 'dollar-1':     return { method: 'fixed_dollar', value: 1.0,  atr: 0 };
        case 'dollar-3':     return { method: 'fixed_dollar', value: 3.0,  atr: 0 };
        case 'atr-2x':       return { method: 'atr_multiple', value: 2.0,  atr: 1.5 };
        case 'atr-3x':       return { method: 'atr_multiple', value: 3.0,  atr: 1.5 };
        default:             return { method: 'fixed_pct',    value: 0.02, atr: 0 };
    }
}

export function fmtN(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtSigned(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toFixed(d);
}

export function fmtPct(v, d = 1) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function methodLabelKey(m) {
    return `view.stop_loss_backtest.method.${m || 'unknown'}`;
}
