// Shared paste-parser used by every view that accepts a textarea-pasted
// list of floats (return series, price series, volume curve, etc.).
//
// Format:
//   * One value per token, separated by whitespace OR commas OR newlines.
//   * Lines starting with `#` are skipped (free-form comments).
//   * Blank lines are skipped.
//   * Each bad token is reported as its own error tagged with line number.
//
// Returns `{ value, errors }`:
//   value:  flat Number[] of successfully-parsed values.
//   errors: { line_no, raw, message }[] — one per offending token.
//           Caller decides whether to render or ignore.
//
// Options:
//   * nonNegative: when true, negative values are reported as errors.
//     Used by the volume-curve parser (negative volume is meaningless).

import { t } from './i18n.js';

const TOKEN_DELIM = /[\s,]+/;

export function parseFloatBlob(text, opts = {}) {
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
        const parts = stripped.split(TOKEN_DELIM).filter(Boolean);
        for (const p of parts) {
            const n = Number(p);
            if (!Number.isFinite(n)) {
                errors.push({ line_no: i + 1, raw, message: `non-numeric "${p}"` });
                continue;
            }
            if (opts.nonNegative && n < 0) {
                errors.push({ line_no: i + 1, raw, message: `negative value "${p}"` });
                continue;
            }
            value.push(n);
        }
    }
    return { value, errors };
}
