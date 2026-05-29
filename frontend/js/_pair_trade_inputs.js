// Pure helpers for the Pair Trade Z-Score view.
//
// Backend returns a snapshot (β, spread mean/stdev, current z, signal).
// We compute the FULL z-score time series locally using the backend's β
// — gives the user a chart of historical z to see when the strategy
// would have triggered + how often it mean-reverted.
//
// Spread formula: spread_t = y_t - β · x_t
// Z-score:       z_t = (spread_t - mean(spread)) / stdev(spread)

import { parseFloatBlob } from './_paste_parser.js';
import { t } from './i18n.js';

/** Parse a price-series textarea. */
export function parseSeries(text) {
    return parseFloatBlob(text);
}

/** Validate the y/x series + threshold inputs. */
export function validateInputs(y, x, config) {
    if (!Array.isArray(y) || y.length < 10) return t('view.pair_trade.validate.y_min');
    if (!Array.isArray(x) || x.length < 10) return t('view.pair_trade.validate.x_min');
    if (y.length !== x.length) {
        return t('view.pair_trade.validate.length_mismatch', { yLen: y.length, xLen: x.length });
    }
    if (y.some(v => !Number.isFinite(v))) return t('view.pair_trade.validate.y_non_finite');
    if (x.some(v => !Number.isFinite(v))) return t('view.pair_trade.validate.x_non_finite');
    if (!Number.isFinite(config.entry_z) || config.entry_z <= 0) {
        return t('view.pair_trade.validate.entry_z');
    }
    if (!Number.isFinite(config.exit_z) || config.exit_z <= 0) {
        return t('view.pair_trade.validate.exit_z');
    }
    if (!Number.isFinite(config.stop_z) || config.stop_z <= 0) {
        return t('view.pair_trade.validate.stop_z');
    }
    if (config.exit_z >= config.entry_z) {
        return t('view.pair_trade.validate.exit_lt_entry');
    }
    if (config.stop_z <= config.entry_z) {
        return t('view.pair_trade.validate.stop_gt_entry');
    }
    return null;
}

/** Build the backend payload. */
export function buildBody(y, x, config) {
    return { y, x, config };
}

/** Compute the full spread + z-score time series locally using a fixed
 *  hedge ratio β. Returns parallel arrays so the caller can chart them.
 *  Spread stats (mean, stdev) are computed over the WHOLE series, not
 *  rolling — matches the backend's PairReport.spread_mean/stdev. */
export function spreadAndZSeries(y, x, beta) {
    if (!Array.isArray(y) || !Array.isArray(x) || y.length !== x.length) {
        return { spreads: [], zs: [], spread_mean: NaN, spread_stdev: NaN };
    }
    const n = y.length;
    if (n === 0 || !Number.isFinite(beta)) {
        return { spreads: [], zs: [], spread_mean: NaN, spread_stdev: NaN };
    }
    const spreads = new Array(n);
    let sum = 0;
    for (let i = 0; i < n; i++) {
        spreads[i] = y[i] - beta * x[i];
        sum += spreads[i];
    }
    const mean = sum / n;
    let sse = 0;
    for (let i = 0; i < n; i++) sse += (spreads[i] - mean) ** 2;
    const stdev = Math.sqrt(sse / n);
    const zs = spreads.map(s => stdev > 1e-15 ? (s - mean) / stdev : 0);
    return { spreads, zs, spread_mean: mean, spread_stdev: stdev };
}

/** Count bars whose |z| crossed a threshold (one signal per crossing,
 *  not per bar — only count the first bar of each excursion outside
 *  the band). */
export function countCrossings(zs, threshold) {
    if (!Array.isArray(zs) || !Number.isFinite(threshold) || threshold <= 0) return 0;
    let count = 0;
    let outside = false;
    for (const z of zs) {
        if (!Number.isFinite(z)) continue;
        const isOutside = Math.abs(z) > threshold;
        if (isOutside && !outside) count++;
        outside = isOutside;
    }
    return count;
}

/** Human-friendly label for the backend signal enum (snake_case from
 *  serde rename_all). */
export function fmtSignal(signal) {
    if (typeof signal !== 'string') return 'unknown';
    switch (signal) {
        case 'long_spread':  return 'LONG SPREAD (buy y, sell β·x)';
        case 'short_spread': return 'SHORT SPREAD (sell y, buy β·x)';
        case 'exit_spread':  return 'EXIT (mean-reverted to band)';
        case 'stop_out':     return 'STOP OUT (|z| blew through stop)';
        case 'hold':         return 'HOLD (no signal)';
        default: return signal.toUpperCase();
    }
}

/** Color class for the signal card. */
export function signalCssClass(signal) {
    switch (signal) {
        case 'long_spread': case 'short_spread': return 'pos';
        case 'stop_out':                         return 'neg';
        default:                                 return '';
    }
}
