// Pure helpers for the Yield Curve PCA view.
//
// Reuses the portfolio-allocator matrix parser (one row per date,
// columns = tenors). Adds tenor-label parsing (one per line) and PCA-
// specific validation + factor naming (PC1=Level, PC2=Slope,
// PC3=Curvature is the Litterman-Scheinkman convention).

import { parseMatrix } from './_portfolio_allocator_inputs.js';
import { parseLabelList } from './_portfolio_allocator_inputs.js';

/** Parse a multiline yield-curve history. One row per date, columns =
 *  tenors. Delegates to the shared matrix parser. */
export function parseCurves(text) {
    return parseMatrix(text);
}

/** Parse tenor labels (one per line). Delegates to the shared label
 *  parser. Returns `string[]`. */
export function parseTenorLabels(text) {
    return parseLabelList(text);
}

/** Validation gate. */
export function validatePcaInputs(curves, topK) {
    if (!Array.isArray(curves) || curves.length < 5) {
        return 'need at least 5 dated curves for a stable PCA';
    }
    if (!Array.isArray(curves[0]) || curves[0].length < 2) {
        return 'each curve must have at least 2 tenors';
    }
    const n = curves[0].length;
    for (let i = 0; i < curves.length; i++) {
        if (!Array.isArray(curves[i]) || curves[i].length !== n) {
            return `row ${i + 1} has ${curves[i]?.length ?? 0} columns, expected ${n}`;
        }
        if (curves[i].some(v => !Number.isFinite(v))) {
            return `row ${i + 1} contains non-finite values`;
        }
    }
    if (!Number.isInteger(topK) || topK < 1 || topK > n) {
        return `top_k must be an integer in [1, ${n}]`;
    }
    return null;
}

/** Canonical factor names per Litterman & Scheinkman (1991). */
export function factorName(idx) {
    const NAMED = ['Level', 'Slope', 'Curvature'];
    return idx < NAMED.length ? NAMED[idx] : `PC${idx + 1}`;
}

/** Normalize tenor labels: pad with defaults (T1..Tn) when too few,
 *  trim when too many. Mirrors the portfolio-allocator helper. */
export function normalizeTenors(labels, n) {
    if (!Array.isArray(labels) || labels.length === 0) {
        return Array.from({ length: n }, (_, i) => `T${i + 1}`);
    }
    const out = labels.slice(0, n);
    while (out.length < n) out.push(`T${out.length + 1}`);
    return out;
}

/** Build the JSON body for /analytics/principal-component-yield-curve. */
export function buildBody(curves, topK) {
    return { curves, top_k: topK };
}

/** Per-factor colors — keep stable across renders so the user learns
 *  "cyan = level, orange = slope, purple = curvature" muscle memory.
 *  Beyond PC3 we cycle through a fallback palette. */
const FACTOR_COLORS = ['#00e5ff', '#ff9f1a', '#a06bff', '#39ff14', '#ff3860', '#ffd84a'];

export function factorColor(idx) {
    return FACTOR_COLORS[idx % FACTOR_COLORS.length];
}
