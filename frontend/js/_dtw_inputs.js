// Pure helpers for the Dynamic Time Warping calculator.
//
// Reuses the shared paste parser for the two series. The DTW
// algorithm matches index i of series A to index j of series B such
// that the cumulative L1 distance is minimized, subject to monotonic
// pairing — series B's index j is allowed to repeat or skip to match
// A's index i, which is what makes the comparison robust to non-
// linear time stretching.
//
// `band_radius` is the Sakoe-Chiba band width — a constraint that |i−j|
// cannot exceed this radius. 0 means unconstrained (full O(n·m) DP);
// small radii speed up the algorithm and prevent pathological matches.

import { parseFloatBlob } from './_paste_parser.js';

export function parseSeries(text) {
    return parseFloatBlob(text);
}

/** Build backend body. `band_radius` of 0 → backend treats as
 *  unconstrained DP. */
export function buildBody(a, b, bandRadius) {
    return {
        a, b,
        band_radius: Number.isInteger(bandRadius) && bandRadius >= 0 ? bandRadius : 0,
    };
}

/** Validate the two-series + band-radius input. */
export function validateInputs(a, b, bandRadius) {
    if (!Array.isArray(a) || a.length < 2) return 'series A needs ≥ 2 values';
    if (!Array.isArray(b) || b.length < 2) return 'series B needs ≥ 2 values';
    if (a.some(x => !Number.isFinite(x))) return 'series A contains non-finite values';
    if (b.some(x => !Number.isFinite(x))) return 'series B contains non-finite values';
    if (!Number.isInteger(bandRadius) || bandRadius < 0) {
        return 'band_radius must be a non-negative integer (0 = unconstrained)';
    }
    return null;
}

/** Normalized distance — distance per matched pair. Useful to compare
 *  DTW distances across pairs of different lengths. */
export function normalizedDistance(distance, pathLength) {
    if (!Number.isFinite(distance) || !Number.isInteger(pathLength) || pathLength <= 0) {
        return null;
    }
    return distance / pathLength;
}

/** Maximum time-stretch along the optimal path — `max |i - j|` across
 *  all pairs. A diagonal-only path returns 0 (no stretching); a path
 *  far from the diagonal indicates strong non-linear timing differences
 *  between the two series. */
export function maxStretch(path) {
    if (!Array.isArray(path) || path.length === 0) return 0;
    let m = 0;
    for (const pair of path) {
        if (!Array.isArray(pair) || pair.length < 2) continue;
        const stretch = Math.abs(pair[0] - pair[1]);
        if (stretch > m) m = stretch;
    }
    return m;
}

/** Unpack the path into parallel xs (i indices into A) and ys (j
 *  indices into B). Used directly by the chart series. */
export function pathToSeries(path) {
    const xs = [];
    const ys = [];
    if (!Array.isArray(path)) return { xs, ys };
    for (const pair of path) {
        if (!Array.isArray(pair) || pair.length < 2) continue;
        xs.push(pair[0]);
        ys.push(pair[1]);
    }
    return { xs, ys };
}
