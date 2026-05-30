// Bollinger Squeeze (Bollinger) helpers.
//
// Backend body: { closes, bb_period, n_stdev, lookback, slack }
// Returns: { width_pct, squeeze_on, bb_period, n_stdev, lookback, slack }
//
// width_t  = 2·n_stdev·stdev(close, bb_period) / mean · 100   (% of midline)
// squeeze_t = width_t ≤ min(width over lookback) · (1 + slack)
//
// Defaults: bb_period=20, n_stdev=2.0, lookback=125, slack=0.05.

import { t } from './i18n.js';

export const DEFAULT_BB_PERIOD = 20;
export const DEFAULT_N_STDEV   = 2.0;
export const DEFAULT_LOOKBACK  = 125;
export const DEFAULT_SLACK     = 0.05;

export const DEFAULT_INPUTS = {
    closes: [],
    bb_period: DEFAULT_BB_PERIOD,
    n_stdev:   DEFAULT_N_STDEV,
    lookback:  DEFAULT_LOOKBACK,
    slack:     DEFAULT_SLACK,
};

export function validateInputs(input) {
    if (!Array.isArray(input.closes))                       return t('view.bollinger_squeeze.validate.closes_array');
    for (let i = 0; i < input.closes.length; i++) {
        if (!Number.isFinite(input.closes[i]))              return t('view.bollinger_squeeze.validate.closes_finite', { i });
    }
    if (!Number.isInteger(input.bb_period))                 return t('view.bollinger_squeeze.validate.bb_period_int');
    if (input.bb_period < 2)                                return t('view.bollinger_squeeze.validate.bb_period_min');
    if (!Number.isFinite(input.n_stdev) || input.n_stdev <= 0) return t('view.bollinger_squeeze.validate.n_stdev');
    if (!Number.isInteger(input.lookback))                  return t('view.bollinger_squeeze.validate.lookback_int');
    if (input.lookback < input.bb_period)                   return t('view.bollinger_squeeze.validate.lookback_min');
    if (!Number.isFinite(input.slack) || input.slack < 0)   return t('view.bollinger_squeeze.validate.slack');
    if (input.closes.length < input.lookback)               return t('view.bollinger_squeeze.validate.closes_min', { n: input.lookback });
    return null;
}

export function buildBody(input) {
    return {
        closes:    input.closes,
        bb_period: input.bb_period,
        n_stdev:   input.n_stdev,
        lookback:  input.lookback,
        slack:     input.slack,
    };
}

// Pure-JS mirror of crates/traderview-core/src/bollinger_squeeze.rs::compute.
export function localCompute(closes, bb_period, n_stdev, lookback, slack) {
    const n = closes.length;
    const report = {
        width_pct:  new Array(n).fill(null),
        squeeze_on: new Array(n).fill(null),
        bb_period, n_stdev, lookback, slack,
    };
    if (bb_period < 2 || lookback < bb_period
        || !Number.isFinite(n_stdev) || n_stdev <= 0
        || !Number.isFinite(slack) || slack < 0
        || n < lookback) return report;
    for (const v of closes) if (!Number.isFinite(v)) return report;
    const pf = bb_period;
    for (let i = bb_period - 1; i < n; i++) {
        let sum = 0;
        for (let k = i + 1 - bb_period; k <= i; k++) sum += closes[k];
        const mean = sum / pf;
        let varSum = 0;
        for (let k = i + 1 - bb_period; k <= i; k++) {
            const d = closes[k] - mean;
            varSum += d * d;
        }
        const std = Math.sqrt(Math.max(0, varSum / pf));
        if (Math.abs(mean) > 0) {
            report.width_pct[i] = 2 * n_stdev * std / Math.abs(mean) * 100;
        }
    }
    for (let i = lookback - 1; i < n; i++) {
        let mn = Infinity;
        let any_null = false;
        for (let k = i + 1 - lookback; k <= i; k++) {
            const v = report.width_pct[k];
            if (v == null) { any_null = true; break; }
            if (v < mn) mn = v;
        }
        if (any_null) continue;
        const cur = report.width_pct[i];
        if (cur == null) continue;
        report.squeeze_on[i] = cur <= mn * (1 + slack);
    }
    return report;
}

// Parse comma/whitespace-separated closes; ignores blanks + # comments.
export function parseClosesBlob(blob) {
    const out = { closes: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: t('common.parse.input_must_be_string') });
        return out;
    }
    const tokens = blob
        .split('\n')
        .map(l => l.split('#')[0])
        .join(' ')
        .split(/[\s,]+/)
        .filter(t => t.length > 0);
    for (let i = 0; i < tokens.length; i++) {
        const v = Number(tokens[i]);
        if (!Number.isFinite(v)) {
            out.errors.push({ line_no: i + 1, message: `token "${tokens[i]}" not finite` });
            continue;
        }
        out.closes.push(v);
    }
    return out;
}

export function closesToBlob(closes) {
    return closes.join('\n');
}

// Last-bar verdict + tightness scale.
export function squeezeBadge(squeeze_on, width_pct) {
    if (!Array.isArray(squeeze_on) || squeeze_on.length === 0)
        return { key: 'view.bbsq.badge.unknown', cls: '' };
    // Find last populated entry.
    let last = null;
    let lastWidth = NaN;
    for (let i = squeeze_on.length - 1; i >= 0; i--) {
        if (squeeze_on[i] != null) {
            last = squeeze_on[i];
            lastWidth = width_pct[i];
            break;
        }
    }
    if (last == null) return { key: 'view.bbsq.badge.unknown', cls: '' };
    if (last) {
        // Squeeze ON — tighter is more dramatic.
        if (Number.isFinite(lastWidth) && lastWidth < 1) return { key: 'view.bbsq.badge.coiled',  cls: 'pos' };
        if (Number.isFinite(lastWidth) && lastWidth < 3) return { key: 'view.bbsq.badge.tight',   cls: 'pos' };
        return { key: 'view.bbsq.badge.squeeze', cls: 'pos' };
    }
    // Squeeze OFF — wider = more volatility expansion.
    if (Number.isFinite(lastWidth) && lastWidth > 20) return { key: 'view.bbsq.badge.expansion', cls: 'neg' };
    return { key: 'view.bbsq.badge.normal', cls: '' };
}

// Aggregate stats.
export function summarize(report) {
    if (!report || !Array.isArray(report.width_pct) || report.width_pct.length === 0)
        return { count: 0, populated: 0, squeeze_count: 0,
                 last_width: NaN, min_width: NaN, max_width: NaN, last_state: null };
    let populated = 0, squeezes = 0, lastWidth = NaN, lastState = null;
    let mn = Infinity, mx = -Infinity;
    for (let i = 0; i < report.width_pct.length; i++) {
        const w = report.width_pct[i];
        if (w != null && Number.isFinite(w)) {
            populated++;
            lastWidth = w;
            if (w < mn) mn = w;
            if (w > mx) mx = w;
        }
        if (report.squeeze_on[i] === true) squeezes++;
        if (report.squeeze_on[i] != null) lastState = report.squeeze_on[i];
    }
    return {
        count: report.width_pct.length,
        populated,
        squeeze_count: squeezes,
        last_width: lastWidth,
        min_width: Number.isFinite(mn) ? mn : NaN,
        max_width: Number.isFinite(mx) ? mx : NaN,
        last_state: lastState,
    };
}

// Index of the most recent squeeze ON, or null.
export function lastSqueezeIndex(squeeze_on) {
    if (!Array.isArray(squeeze_on)) return null;
    for (let i = squeeze_on.length - 1; i >= 0; i--) {
        if (squeeze_on[i] === true) return i;
    }
    return null;
}

function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF - 0.5;
    };
}

export function makeDemoInput(kind = 'coiling') {
    switch (kind) {
        case 'flat-perpetual': {
            // Constant series → width=0 → always squeezed.
            const closes = new Array(200).fill(100);
            return { ...DEFAULT_INPUTS, closes };
        }
        case 'expansion-after-quiet': {
            // 130 quiet bars + 70 noisy → squeezes early, off late.
            const rand = lcg(42n);
            const closes = new Array(130).fill(100);
            for (let i = 0; i < 70; i++) closes.push(100 + rand() * 50);
            return { ...DEFAULT_INPUTS, closes };
        }
        case 'coiling': {
            // Volatile early, then quiet (squeeze should fire later).
            const rand = lcg(7n);
            const closes = [];
            for (let i = 0; i < 100; i++) closes.push(100 + rand() * 10);
            for (let i = 0; i < 100; i++) closes.push(100 + rand() * 0.2);
            return { ...DEFAULT_INPUTS, closes };
        }
        case 'noisy-walk': {
            const rand = lcg(99n);
            const closes = new Array(200);
            closes[0] = 100;
            for (let i = 1; i < 200; i++) closes[i] = Math.max(1, closes[i - 1] + rand() * 2);
            return { ...DEFAULT_INPUTS, closes };
        }
        case 'short-lookback': {
            const rand = lcg(3n);
            const closes = new Array(50);
            closes[0] = 100;
            for (let i = 1; i < 50; i++) closes[i] = Math.max(1, closes[i - 1] + rand() * 1);
            return { ...DEFAULT_INPUTS, closes, lookback: 40, bb_period: 10 };
        }
        case 'tight-slack': {
            // Same coiling demo but slack=0 → strict ≤ min comparison.
            const rand = lcg(7n);
            const closes = [];
            for (let i = 0; i < 100; i++) closes.push(100 + rand() * 10);
            for (let i = 0; i < 100; i++) closes.push(100 + rand() * 0.2);
            return { ...DEFAULT_INPUTS, closes, slack: 0 };
        }
        case 'loose-slack': {
            const rand = lcg(7n);
            const closes = [];
            for (let i = 0; i < 100; i++) closes.push(100 + rand() * 10);
            for (let i = 0; i < 100; i++) closes.push(100 + rand() * 0.5);
            return { ...DEFAULT_INPUTS, closes, slack: 0.5 };
        }
        case 'wide-bands': {
            // n_stdev=3 → 99.7% bands.
            const rand = lcg(11n);
            const closes = [];
            for (let i = 0; i < 200; i++) closes.push(100 + rand() * 2);
            return { ...DEFAULT_INPUTS, closes, n_stdev: 3.0 };
        }
        default: return makeDemoInput('coiling');
    }
}

export function fmtWidth(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d) + '%';
}

export function fmtNum(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}

export function fmtUSD(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return '$' + v.toFixed(d);
}
