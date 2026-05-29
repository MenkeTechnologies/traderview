// Pure helpers for the Execution Scheduler view.
//
// Parse the user-pasted volume curve (one bar's expected volume per
// token), validate inputs against the 3 algos' constraints, and shape
// the per-algo payloads.
//
// Algo conventions:
//   * POV  — slice_i = participation_rate · expected_volume_i; can fall
//            short of total if rate × curve < total.
//   * TWAP — equal slice across num_slices bars; volume curve is
//            optional and only used to flag a max participation rate.
//   * VWAP — slice_i ∝ expected_volume_i / sum(curve); always fills
//            the full order.

import { parseFloatBlob } from './_paste_parser.js';

/** Parse a 1-D volume curve from text. Negative volumes are rejected
 *  with line-anchored errors. */
export function parseVolumeCurve(text) {
    return parseFloatBlob(text, { nonNegative: true });
}

/** Validation: combined inputs check before sending. */
export function validateExecInputs(totalOrder, volumeCurve, participationRate) {
    if (!Number.isFinite(totalOrder) || totalOrder <= 0) {
        return 'total order size must be > 0';
    }
    if (!Array.isArray(volumeCurve) || volumeCurve.length === 0) {
        return 'volume curve must have at least one bar';
    }
    if (volumeCurve.some(v => !Number.isFinite(v) || v < 0)) {
        return 'volume curve contains invalid (negative / non-finite) values';
    }
    const sum = volumeCurve.reduce((a, b) => a + b, 0);
    if (sum <= 0) return 'volume curve sums to 0 — at least one bar must have volume';
    if (!Number.isFinite(participationRate)
        || participationRate <= 0 || participationRate > 1) {
        return 'participation rate must be in (0, 1]';
    }
    return null;
}

/** Per-algo payload builders. Each returns { endpoint, body }. */
export function buildPovBody(totalOrder, volumeCurve, participationRate) {
    return {
        total_order_size: totalOrder,
        volume_curve: volumeCurve,
        participation_rate: participationRate,
    };
}

export function buildTwapBody(totalOrder, numSlices, volumeCurve) {
    return {
        total_order_size: totalOrder,
        num_slices: numSlices,
        // Backend's Option<Vec<f64>> serde: send the array unconditionally
        // (the route reads it for the participation rate). If it's empty
        // the server's None path runs; either way fine.
        ...(Array.isArray(volumeCurve) && volumeCurve.length === numSlices
            ? { volume_curve: volumeCurve }
            : {}),
    };
}

export function buildVwapBody(totalOrder, volumeCurve) {
    return { total_order_size: totalOrder, volume_curve: volumeCurve };
}

/** Compute a summary row per algo from a uniform response shape.
 *  Returns { totalFilled, lastFillBar, shortfall, maxParticipation }.
 *  All algos return slices + cumulative_fill; only POV returns
 *  shortfall + completion_bar; only TWAP/VWAP return
 *  max_participation_rate. The fields we don't get back are null. */
export function summarizeSchedule(res) {
    if (!res || !Array.isArray(res.slices)) return null;
    const slices = res.slices;
    const cum = Array.isArray(res.cumulative_fill) ? res.cumulative_fill : [];
    const totalFilled = cum.length ? cum[cum.length - 1] : slices.reduce((a, b) => a + b, 0);
    let lastFillBar = null;
    for (let i = slices.length - 1; i >= 0; i--) {
        if (slices[i] > 0) { lastFillBar = i; break; }
    }
    return {
        totalFilled,
        lastFillBar,
        shortfall: typeof res.shortfall === 'number' ? res.shortfall : null,
        completionBar: typeof res.completion_bar === 'number' ? res.completion_bar : null,
        maxParticipation: typeof res.max_participation_rate === 'number'
            ? res.max_participation_rate : null,
    };
}
