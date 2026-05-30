// Pure helpers for the Forward Vol Curve view.
//
// Parse a pasted term-structure of "tenor iv" rows. The tenor accepts
// either a number (years) OR a shorthand like "1D", "1W", "1M", "3M",
// "1Y", "2Y" — what traders actually quote. The IV reuses the
// decimal/percent parser from the vol-smile view (≥ 1.0 OR trailing
// "%" → divide by 100).

import { parseIv } from './_vol_smile_inputs.js';
import { t } from './i18n.js';

// Tenor unit → years.
const TENOR_UNITS = {
    D: 1 / 365.0,
    W: 7 / 365.0,
    M: 1 / 12.0,
    Y: 1.0,
};

/** Convert a token like "1M" / "3M" / "1.5Y" / "0.25" to a year count.
 *  Bare numbers are interpreted as years directly. Returns NaN on
 *  unrecognized input. */
export function parseTenor(s) {
    if (typeof s !== 'string' && typeof s !== 'number') return NaN;
    const raw = String(s).trim();
    if (!raw) return NaN;
    // Try as a bare number first (interpreted as years).
    const asNumber = Number(raw);
    if (Number.isFinite(asNumber)) return asNumber;
    // Then try the "<n><unit>" form, units case-insensitive.
    const m = raw.match(/^(-?\d+(?:\.\d+)?)\s*([dwmyDWMY])$/);
    if (!m) return NaN;
    const n = Number(m[1]);
    const unit = m[2].toUpperCase();
    const yr = TENOR_UNITS[unit];
    if (!Number.isFinite(n) || yr == null) return NaN;
    return n * yr;
}

/** Parse the textarea blob. Each non-comment, non-blank line should be
 *  two tokens: tenor + iv. Whitespace OR comma separator. Returns
 *  `{ value, errors }`. The value is an array of `{ tenor_years, iv,
 *  raw_tenor, line_no }`. */
export function parseTermStructure(text) {
    const value = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { value, errors: [{ line_no: 0, raw: '', message: t('common.parse.input_must_be_string') }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const stripped = raw.trim();
        if (!stripped || stripped.startsWith('#')) continue;
        const parts = stripped.split(/[\s,]+/).filter(Boolean);
        if (parts.length < 2) {
            errors.push({ line_no: i + 1, raw, message: t('view.forward_vol.parse.expected_tenor_iv') });
            continue;
        }
        const tenor_years = parseTenor(parts[0]);
        const iv = parseIv(parts[1]);
        if (!Number.isFinite(tenor_years) || tenor_years <= 0) {
            errors.push({ line_no: i + 1, raw, message: `bad tenor "${parts[0]}"` });
            continue;
        }
        if (!Number.isFinite(iv) || iv < 0) {
            errors.push({ line_no: i + 1, raw, message: `bad IV "${parts[1]}"` });
            continue;
        }
        value.push({ tenor_years, iv, raw_tenor: parts[0], line_no: i + 1 });
    }
    return { value, errors };
}

/** Sort rows ascending by tenor (mandatory for the bootstrap to make
 *  sense; the backend enforces monotonic-increasing expiries too). */
export function sortRowsByTenor(rows) {
    return [...rows].sort((a, b) => a.tenor_years - b.tenor_years);
}

/** Reject duplicate tenors (would make Δt = 0 → division by zero in
 *  the bootstrap). Returns null on success, an error string with the
 *  duplicate value otherwise. */
export function checkUniqueTenors(rows) {
    for (let i = 1; i < rows.length; i++) {
        if (rows[i].tenor_years <= rows[i - 1].tenor_years) {
            return t('view.forward_vol.validate.tenors_increasing', { line: rows[i].line_no });
        }
    }
    return null;
}

/** Standard validation gate. */
export function validateTermStructure(rows) {
    if (!Array.isArray(rows) || rows.length < 2) {
        return t('view.forward_vol.validate.rows_min');
    }
    if (rows.some(r => !Number.isFinite(r.tenor_years) || r.tenor_years <= 0)) {
        return t('view.forward_vol.validate.tenors_positive');
    }
    if (rows.some(r => !Number.isFinite(r.iv) || r.iv < 0)) {
        return t('view.forward_vol.validate.ivs_non_negative');
    }
    return null;
}

/** Build the backend payload from the parsed + sorted rows. */
export function buildBody(rows) {
    return {
        expiries: rows.map(r => r.tenor_years),
        spot_iv: rows.map(r => r.iv),
    };
}

/** Produce a step-function (x, y) pair-of-arrays for the forward-vol
 *  chart series. Each forward vol applies to the interval
 *  [tenor_i, tenor_{i+1}], so a "step" between the two tenors visually
 *  represents the constant forward vol over that window. */
export function forwardVolStepSeries(rows, forwardVols) {
    const xs = [];
    const ys = [];
    if (!Array.isArray(rows) || !Array.isArray(forwardVols)) return { xs, ys };
    for (let i = 0; i < forwardVols.length && i + 1 < rows.length; i++) {
        const fv = forwardVols[i];
        const t0 = rows[i].tenor_years;
        const t1 = rows[i + 1].tenor_years;
        // Emit (t0, fv) and (t1, fv) — uPlot will draw the connecting
        // step. A null between consecutive intervals would create a
        // visible gap which we don't want (step changes happen at the
        // tenor points).
        xs.push(t0, t1);
        ys.push(fv, fv);
    }
    return { xs, ys };
}
