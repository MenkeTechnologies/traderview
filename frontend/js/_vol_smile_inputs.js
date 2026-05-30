// Pure helpers for the vol-smile fitter view: parse pasted strike/IV
// chains, convert to the backend's log-moneyness + total-variance shape.
// Kept out of the view file so vitest can exercise them headless.
//
// Input format: one quote per line, two whitespace-OR-comma-separated
// fields per line: strike, then IV. IV may be a decimal (0.25) or a
// percent (25, 25%, 25.0%). Lines starting with `#` and blank lines are
// ignored.
//
// Forward = spot · exp((rate − div_yield) · t). For equities without
// dividends, set div_yield = 0; for currencies use rate_dom − rate_for.
// (Garman-Kohlhagen convention.)

import { t } from './i18n.js';

/** Parse a multiline strike/IV blob. Returns { rows, errors } where
 *  rows is an array of `{ strike, iv, line_no }` and errors is an
 *  array of `{ line_no, raw, message }`. A successful parse has
 *  `errors.length === 0`. Always returns both; the caller decides how
 *  strict to be.
 */
export function parseStrikeIvText(text) {
    const rows = [];
    const errors = [];
    if (typeof text !== 'string') return { rows, errors: [{ line_no: 0, raw: '', message: t('view.vol_smile.parse.input_not_string') }] };
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const stripped = raw.trim();
        if (!stripped || stripped.startsWith('#')) continue;
        // Split on comma OR whitespace.
        const parts = stripped.split(/[\s,]+/).filter(Boolean);
        if (parts.length < 2) {
            errors.push({ line_no: i + 1, raw, message: t('view.vol_smile.parse.expected_strike_iv') });
            continue;
        }
        const strike = Number(parts[0]);
        const iv = parseIv(parts[1]);
        if (!Number.isFinite(strike) || strike <= 0) {
            errors.push({ line_no: i + 1, raw, message: `bad strike "${parts[0]}"` });
            continue;
        }
        if (!Number.isFinite(iv) || iv < 0) {
            errors.push({ line_no: i + 1, raw, message: `bad IV "${parts[1]}"` });
            continue;
        }
        rows.push({ strike, iv, line_no: i + 1 });
    }
    return { rows, errors };
}

/** Coerce IV from a string. Accepts "0.25", "25", "25%", "25.0%". An
 *  ambiguity exists between "0.25" (decimal) and "25" (percent). The
 *  rule: any value ≥ 1.0 (or with a trailing %) is treated as percent
 *  and divided by 100. Values < 1.0 are decimal. This matches how
 *  traders quote IV ("25 vol" vs "0.25 vol"). */
export function parseIv(s) {
    if (typeof s !== 'string' && typeof s !== 'number') return NaN;
    let raw = String(s).trim();
    let isPct = false;
    if (raw.endsWith('%')) {
        isPct = true;
        raw = raw.slice(0, -1).trim();
    }
    const n = Number(raw);
    if (!Number.isFinite(n)) return NaN;
    if (n < 0) return NaN;
    if (isPct || n >= 1.0) return n / 100;
    return n;
}

/** Convert parsed rows + (spot, rate, div_yield, t_years) into the
 *  backend's `{ log_moneyness, total_variance, expiry_years }` payload. */
export function buildSviBody(rows, spot, rate, divYield, tYears) {
    const fwd = spot * Math.exp((rate - divYield) * tYears);
    const log_moneyness = rows.map(r => Math.log(r.strike / fwd));
    const total_variance = rows.map(r => r.iv * r.iv * tYears);
    return { log_moneyness, total_variance, expiry_years: tYears };
}

/** Validate the inputs to buildSviBody. Returns null on success,
 *  an error string otherwise. */
export function validateSmileInputs(rows, spot, tYears) {
    if (!Array.isArray(rows) || rows.length < 5) {
        return t('view.vol_smile.validate.rows_min');
    }
    if (!Number.isFinite(spot) || spot <= 0) return t('view.vol_smile.validate.spot');
    if (!Number.isFinite(tYears) || tYears <= 0) return t('view.vol_smile.validate.expiry');
    for (const r of rows) {
        if (!Number.isFinite(r.strike) || r.strike <= 0) {
            return t('view.vol_smile.validate.strike', { line: r.line_no });
        }
        if (!Number.isFinite(r.iv) || r.iv < 0) {
            return t('view.vol_smile.validate.iv', { line: r.line_no });
        }
    }
    return null;
}

/** Sort rows ascending by strike (the chart looks weird otherwise). */
export function sortRowsByStrike(rows) {
    return [...rows].sort((a, b) => a.strike - b.strike);
}

/** Compute the local skew (∂σ/∂k) at ATM (k = 0) from raw SVI params.
 *  Useful as a sanity-check number on the fit:
 *    w'(k) = b · (ρ + (k - m) / sqrt((k - m)² + σ²))
 *    σ_IV(k) = sqrt(w(k) / T)
 *    σ_IV'(k) = w'(k) / (2 · T · σ_IV(k))
 */
export function atmSkewSlope(params, expiryYears) {
    const { a, b, rho, m, sigma } = params;
    const k = 0;
    const dist = Math.sqrt((k - m) * (k - m) + sigma * sigma);
    const w_prime = b * (rho + (k - m) / dist);
    const w = a + b * (rho * (k - m) + dist);
    if (w <= 0 || expiryYears <= 0) return 0;
    const iv = Math.sqrt(w / expiryYears);
    if (iv <= 0) return 0;
    return w_prime / (2 * expiryYears * iv);
}
