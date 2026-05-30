// Pure helpers for the Portfolio Allocator view.
//
// Parse user-pasted inputs into the shapes the allocator endpoints
// expect:
//   * covariance matrix (NxN, whitespace OR comma rows)
//   * asset labels (one per line, optional — defaults to A1..An)
//   * expected excess returns (one per line, optional — used only by
//     the min-variance/tangency solver)
//
// All parsers return `{ value, errors }` and never throw.

import { t } from './i18n.js';

/** Parse a multi-line N×N matrix. Tokens within a row separated by
 *  whitespace OR commas; rows by newlines. Lines starting with `#`
 *  and blank lines are skipped. Returns the matrix plus a list of
 *  line-anchored error strings.
 */
export function parseMatrix(text) {
    const value = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { value, errors: [{ line_no: 0, message: t('common.parse.input_must_be_string'), raw: '' }] };
    }
    const lines = text.split(/\r?\n/);
    let width = null;
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const stripped = raw.trim();
        if (!stripped || stripped.startsWith('#')) continue;
        const parts = stripped.split(/[\s,]+/).filter(Boolean);
        const row = parts.map(Number);
        if (row.some(x => !Number.isFinite(x))) {
            errors.push({ line_no: i + 1, raw, message: 'non-numeric token in row' });
            continue;
        }
        if (width == null) width = row.length;
        else if (row.length !== width) {
            errors.push({ line_no: i + 1, raw, message:
                `expected ${width} columns, got ${row.length}` });
            continue;
        }
        value.push(row);
    }
    return { value, errors };
}

/** Same parser, but for a single column of floats (excess returns). */
export function parseFloatList(text) {
    const value = [];
    const errors = [];
    if (typeof text !== 'string' || !text.trim()) return { value, errors };
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const stripped = raw.trim();
        if (!stripped || stripped.startsWith('#')) continue;
        const parts = stripped.split(/[\s,]+/).filter(Boolean);
        for (const p of parts) {
            const n = Number(p);
            if (!Number.isFinite(n)) {
                errors.push({ line_no: i + 1, raw, message: `bad number "${p}"` });
                continue;
            }
            value.push(n);
        }
    }
    return { value, errors };
}

/** Labels: one per line. Blank/# lines skipped. Returns trimmed strings. */
export function parseLabelList(text) {
    if (typeof text !== 'string') return [];
    return text.split(/\r?\n/)
        .map(s => s.trim())
        .filter(s => s && !s.startsWith('#'));
}

/** Generate fallback labels A1..An when the user doesn't supply any. */
export function defaultLabels(n) {
    return Array.from({ length: n }, (_, i) => `A${i + 1}`);
}

/** Validate that a parsed matrix can be sent to the allocators.
 *  Returns null on success, an error string otherwise. */
export function validateCovariance(cov) {
    if (!Array.isArray(cov) || cov.length < 2) return t('view.portfolio_allocator.validate.assets_min');
    const n = cov.length;
    for (let i = 0; i < n; i++) {
        if (!Array.isArray(cov[i]) || cov[i].length !== n) {
            return t('view.portfolio_allocator.validate.row_cols', { row: i + 1, n });
        }
        if (cov[i][i] <= 0) return t('view.portfolio_allocator.validate.diagonal', { i });
    }
    // Symmetry check: allow small float drift.
    for (let i = 0; i < n; i++) {
        for (let j = i + 1; j < n; j++) {
            if (Math.abs(cov[i][j] - cov[j][i]) > 1e-9 * Math.max(1, Math.abs(cov[i][j]))) {
                return t('view.portfolio_allocator.validate.not_symmetric', { i: i + 1, j: j + 1 });
            }
        }
    }
    return null;
}

/** Pretty-print a matrix as space-separated rows (for the textarea
 *  default placeholder, and for round-trip tests). */
export function formatMatrix(m, digits = 4) {
    return m.map(row => row.map(v => v.toFixed(digits)).join(' ')).join('\n');
}

/** Normalize labels to match cov dimension. Right-pads with defaults if
 *  the user supplied fewer; trims if they supplied more. */
export function normalizeLabels(labels, n) {
    if (!Array.isArray(labels) || labels.length === 0) return defaultLabels(n);
    const out = labels.slice(0, n);
    while (out.length < n) out.push(`A${out.length + 1}`);
    return out;
}

/** Default excess-returns vector (uniform 5%) for the min-variance
 *  endpoint when the user doesn't provide one. Lets the MV solver also
 *  compute its tangency portfolio without us forcing the user to
 *  specify returns. */
export function defaultExcessReturns(n) {
    return Array.from({ length: n }, () => 0.05);
}
