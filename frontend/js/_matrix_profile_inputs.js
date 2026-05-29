// Pure helpers for the Pattern Discovery (Matrix Profile) view.
//
// Parse a 1-D price/return series. Validate vs the matrix-profile
// backend's window-size constraint (need 2·m samples and m ≥ 4).
//
// Generate the per-bar series used to render motif and discord markers
// on top of the price chart: an array the same length as the profile
// with the price values *only* where they intersect a flagged window,
// and `null` elsewhere. uPlot draws nothing for null points so this
// produces visible highlights without dropping into custom canvas
// painting.

import { parseFloatBlob } from './_paste_parser.js';

/** Parse a 1-D series. Same conventions as the other views' parsers. */
export function parseSeries(text) {
    return parseFloatBlob(text);
}

/** Validate the (series, m) combo against the backend's constraints:
 *  m ≥ 4 and n ≥ 2m. */
export function validateMatrixProfileInputs(series, m) {
    if (!Array.isArray(series) || series.length === 0) return 'series is empty';
    if (series.some(x => !Number.isFinite(x))) return 'series contains non-finite values';
    if (!Number.isInteger(m) || m < 4) return 'window m must be an integer ≥ 4';
    if (series.length < 2 * m) {
        return `series too short: need at least ${2 * m} samples for m=${m} (got ${series.length})`;
    }
    return null;
}

/** Build a per-bar overlay series with the original price at any index
 *  that falls inside a flagged window, and null elsewhere. `windows` is
 *  an array of `{ start }` objects (window length = `m`). */
export function overlaySeriesForWindows(series, windows, m) {
    const out = new Array(series.length).fill(null);
    if (!Array.isArray(windows) || windows.length === 0) return out;
    for (const w of windows) {
        const start = w.start;
        if (!Number.isInteger(start) || start < 0) continue;
        for (let k = 0; k < m && start + k < series.length; k++) {
            out[start + k] = series[start + k];
        }
    }
    return out;
}

/** Unpack the backend's tuple shapes for the motif pair and top
 *  discords. JSON serializes Rust tuples as arrays, so the wire shape
 *  is `[i, j, d]` and `[i, d]` — turn those into named objects so the
 *  view doesn't index by position. */
export function unpackMotifPair(pairOrNull) {
    if (!Array.isArray(pairOrNull) || pairOrNull.length < 3) return null;
    const [i, j, distance] = pairOrNull;
    if (!Number.isInteger(i) || !Number.isInteger(j) || !Number.isFinite(distance)) return null;
    return { i, j, distance };
}

export function unpackDiscords(raw) {
    if (!Array.isArray(raw)) return [];
    return raw
        .filter(t => Array.isArray(t) && t.length >= 2)
        .map(([start, distance]) => ({ start, distance }))
        .filter(d => Number.isInteger(d.start) && Number.isFinite(d.distance));
}

/** Generate `xs = 0..n-1` as the index axis used by uPlot. */
export function indexAxis(n) {
    return Array.from({ length: n }, (_, i) => i);
}
