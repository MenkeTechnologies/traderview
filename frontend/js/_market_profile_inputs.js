// Market Profile (TPO) helpers shared by view + vitest.
//
// Backend body shape: { brackets: [{bracket_index, high, low}, ...],
// tick_size: f64 }. Backend returns levels[] + POC + VAH/VAL + single_prints.

const TOKEN_DELIM = /[\s,]+/;

// Three-token-per-line "bracket_index high low".
export function parseBracketBlob(text) {
    const brackets = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { brackets, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.trim();
        if (!s || s.startsWith('#')) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 3) {
            errors.push({ line_no: i + 1, raw, message: `expected 3 tokens (bracket_index high low), got ${parts.length}` });
            continue;
        }
        const idxNum = Number(parts[0]);
        const high = Number(parts[1]);
        const low = Number(parts[2]);
        if (!Number.isFinite(idxNum) || !Number.isInteger(idxNum) || idxNum < 0) {
            errors.push({ line_no: i + 1, raw, message: `bracket_index must be non-negative integer` });
            continue;
        }
        const bracket_index = idxNum;
        if (!Number.isFinite(high) || high <= 0) {
            errors.push({ line_no: i + 1, raw, message: `high must be > 0` });
            continue;
        }
        if (!Number.isFinite(low) || low <= 0) {
            errors.push({ line_no: i + 1, raw, message: `low must be > 0` });
            continue;
        }
        if (high < low) {
            errors.push({ line_no: i + 1, raw, message: `high must be ≥ low` });
            continue;
        }
        brackets.push({ bracket_index, high, low });
    }
    return { brackets, errors };
}

export function validateInputs(brackets, tickSize) {
    if (!Array.isArray(brackets) || brackets.length === 0)
        return 'need at least 1 bracket';
    if (!Number.isFinite(tickSize) || tickSize <= 0)
        return 'tick_size must be > 0';
    return null;
}

export function buildBody(brackets, tickSize) {
    return { brackets, tick_size: tickSize };
}

// Letter labels for the TPO histogram, A-Z then a-z. Bracket index N
// maps to TPO_LETTERS[N % 52]. Mirrors Sierra Chart convention.
export const TPO_LETTERS = (() => {
    const letters = [];
    for (let i = 0; i < 26; i++) letters.push(String.fromCharCode(65 + i));   // A-Z
    for (let i = 0; i < 26; i++) letters.push(String.fromCharCode(97 + i));   // a-z
    return letters;
})();

export function bracketLetter(idx) {
    if (!Number.isInteger(idx) || idx < 0) return '?';
    return TPO_LETTERS[idx % 52];
}

// Maps a price level to its UI tier — POC / VA / single-print / normal.
// Drives the per-row color in the TPO histogram.
export function levelTier(level, report) {
    if (!level || !report) return 'normal';
    if (level.price === report.poc_price) return 'poc';
    if (level.single_print) return 'single';
    if (level.price >= report.value_area_low && level.price <= report.value_area_high)
        return 'value';
    return 'normal';
}

// Renders the per-level letter row (e.g. "ABDFG" for 5 brackets) — the
// visual "bar" of the TPO histogram is built from these letter strings.
export function levelLetters(level) {
    if (!level || !Array.isArray(level.brackets)) return '';
    return level.brackets.map(bracketLetter).join('');
}

// Counts each tier across the levels — used for the summary cards.
export function tierCounts(report) {
    if (!report || !Array.isArray(report.levels)) return { poc: 0, value: 0, single: 0, normal: 0 };
    let poc = 0, value = 0, single = 0, normal = 0;
    for (const l of report.levels) {
        const t = levelTier(l, report);
        if      (t === 'poc')    poc++;
        else if (t === 'single') single++;
        else if (t === 'value')  value++;
        else                     normal++;
    }
    return { poc, value, single, normal };
}

// Deterministic 13-bracket demo (matches a typical RTH session with A-M
// brackets). Engineered to produce a "normal day" profile with a clear
// POC near the middle, a defined value area, and a few single-print tails.
export function makeDemoBrackets() {
    return [
        { bracket_index: 0,  high: 102.5, low: 101.0 },   // A — single print up high
        { bracket_index: 1,  high: 101.5, low: 100.0 },   // B
        { bracket_index: 2,  high: 100.5, low:  99.5 },   // C
        { bracket_index: 3,  high: 100.5, low:  99.0 },   // D
        { bracket_index: 4,  high: 100.0, low:  99.0 },   // E
        { bracket_index: 5,  high:  99.5, low:  98.5 },   // F
        { bracket_index: 6,  high:  99.5, low:  98.5 },   // G   ← POC band
        { bracket_index: 7,  high: 100.0, low:  98.5 },   // H   ← POC band
        { bracket_index: 8,  high: 100.0, low:  99.0 },   // I
        { bracket_index: 9,  high:  99.5, low:  98.0 },   // J
        { bracket_index: 10, high:  99.0, low:  97.5 },   // K
        { bracket_index: 11, high:  98.5, low:  97.0 },   // L
        { bracket_index: 12, high:  98.0, low:  96.5 },   // M — single print down low
    ];
}

export function fmtN(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtInt(v) {
    if (!Number.isFinite(v)) return '—';
    return Math.round(v).toLocaleString('en-US');
}
