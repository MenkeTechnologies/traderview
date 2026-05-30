// Herfindahl-Hirschman concentration index helpers.
//
// Backend body: { weights: number[] }
// Returns: { hhi, hhi_scaled, effective_n, n_positions, max_weight } | null
//
// HHI = Σ w_i² over positive weights. If weights don't sum to 1, the result
// is internally normalized: HHI = Σw² / (Σw)².
// Scaled HHI uses the regulatory 0–10_000 scale (DOJ antitrust threshold = 1500).

import { t as tr } from './i18n.js';

export const DEFAULT_INPUTS = {
    weights: [0.25, 0.25, 0.25, 0.25],
};

// Regulatory & analytic HHI thresholds (DOJ antitrust + portfolio convention).
export const DOJ_CONCENTRATED = 2500;     // scaled HHI ≥ 2500 = "highly concentrated"
export const DOJ_MODERATE = 1500;         // scaled HHI ≥ 1500 = "moderately concentrated"

export function validateInputs(input) {
    if (!Array.isArray(input.weights))                 return tr('view.hhi.validate.weights_array');
    if (input.weights.length === 0)                     return tr('view.hhi.validate.weights_empty');
    for (let i = 0; i < input.weights.length; i++) {
        const w = input.weights[i];
        if (!Number.isFinite(w))                       return tr('view.hhi.validate.weight_finite', { i });
        if (w < 0)                                     return tr('view.hhi.validate.weight_negative', { i });
    }
    let sum = 0;
    for (const w of input.weights) sum += w;
    if (sum <= 0)                                       return tr('view.hhi.validate.sum_positive');
    return null;
}

export function buildBody(input) {
    return { weights: input.weights };
}

// Pure-JS mirror of crates/traderview-core/src/herfindahl.rs::compute.
// Returns null on validation failure / all-zero.
export function localCompute(weights) {
    if (!Array.isArray(weights) || weights.length === 0) return null;
    let sum_w = 0, sum_w2 = 0, max_w = 0, n = 0;
    for (const w of weights) {
        if (!Number.isFinite(w)) return null;
        if (w < 0) return null;
        if (w > 0) {
            sum_w += w;
            sum_w2 += w * w;
            if (w > max_w) max_w = w;
            n++;
        }
    }
    if (sum_w <= 0) return null;
    const normalized = Math.abs(sum_w - 1) > 1e-9
        ? sum_w2 / (sum_w * sum_w)
        : sum_w2;
    if (!Number.isFinite(normalized) || normalized <= 0) return null;
    return {
        hhi: normalized,
        hhi_scaled: normalized * 10_000,
        effective_n: 1 / normalized,
        n_positions: n,
        max_weight: max_w / sum_w,
    };
}

// Parse "label weight" per line, or single "weight" tokens.
// Labels are optional — used purely for display in the table.
// # comments + blanks ignored.
export function parsePositionsBlob(blob) {
    const out = { weights: [], labels: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: tr('common.parse.input_must_be_string') });
        return out;
    }
    const lines = blob.split('\n');
    let idx = 0;
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i].split('#')[0].trim();
        if (!raw) continue;
        const toks = raw.split(/[\s,]+/).filter(t => t.length > 0);
        let label, weight;
        if (toks.length === 1) {
            label = `pos_${idx + 1}`;
            weight = Number(toks[0]);
        } else if (toks.length === 2) {
            label = toks[0];
            weight = Number(toks[1]);
        } else {
            out.errors.push({ line_no: i + 1, message: tr('view.herfindahl.parse.expected_1_or_2') });
            continue;
        }
        if (!Number.isFinite(weight)) {
            out.errors.push({ line_no: i + 1, message: tr('common.parse.weight_not_finite') });
            continue;
        }
        if (weight < 0) {
            out.errors.push({ line_no: i + 1, message: tr('view.herfindahl.parse.weight_non_neg') });
            continue;
        }
        out.weights.push(weight);
        out.labels.push(label);
        idx++;
    }
    return out;
}

export function positionsToBlob(labels, weights) {
    if (!labels || !weights) return '';
    return labels.map((l, i) => `${l} ${weights[i]}`).join('\n');
}

// 5-tier concentration verdict — combines DOJ scale + portfolio convention.
export function concentrationBadge(report) {
    if (!report || !Number.isFinite(report.hhi_scaled)) return { key: 'view.hhi.badge.unknown', cls: '' };
    const s = report.hhi_scaled;
    if (s >= 5000) return { key: 'view.hhi.badge.extreme',      cls: 'neg' };
    if (s >= DOJ_CONCENTRATED) return { key: 'view.hhi.badge.highly',  cls: 'neg' };
    if (s >= DOJ_MODERATE)     return { key: 'view.hhi.badge.moderate', cls: '' };
    if (s >= 500)              return { key: 'view.hhi.badge.diversified', cls: 'pos' };
    return { key: 'view.hhi.badge.well_diversified', cls: 'pos' };
}

// Effective-N efficiency: how close are you to using your positions?
export function efficiencyBadge(report) {
    if (!report || !Number.isFinite(report.effective_n) || report.n_positions === 0)
        return { key: 'view.hhi.eff.unknown', cls: '' };
    const ratio = report.effective_n / report.n_positions;
    if (ratio >= 0.95) return { key: 'view.hhi.eff.optimal',  cls: 'pos' };
    if (ratio >= 0.75) return { key: 'view.hhi.eff.good',     cls: 'pos' };
    if (ratio >= 0.50) return { key: 'view.hhi.eff.fair',     cls: '' };
    if (ratio >= 0.25) return { key: 'view.hhi.eff.poor',     cls: 'neg' };
    return { key: 'view.hhi.eff.wasted', cls: 'neg' };
}

export function makeDemoInput(kind = 'equal-4') {
    switch (kind) {
        case 'equal-4':       return { labels: ['SPY','QQQ','GLD','AGG'], weights: [0.25, 0.25, 0.25, 0.25] };
        case 'equal-10':      return { labels: alphaLabels(10), weights: Array(10).fill(0.1) };
        case 'concentrated':  return { labels: ['NVDA','SPY','AAPL','MSFT','GOOG'],
                                         weights: [0.80, 0.05, 0.05, 0.05, 0.05] };
        case 'single-name':   return { labels: ['NVDA'], weights: [1.0] };
        case 'pareto-80-20':  return { labels: ['A','B','C','D','E','F','G','H','I','J'],
                                         weights: [0.40, 0.20, 0.10, 0.07, 0.06, 0.05, 0.04, 0.03, 0.03, 0.02] };
        case 'unnormalized':  return { labels: ['A','B','C','D'], weights: [5, 5, 5, 5] };
        case 'with-zeroes':   return { labels: ['A','B','C','D','E'], weights: [0.5, 0.5, 0, 0, 0] };
        case '60-40-style':   return { labels: ['stocks','bonds'], weights: [0.60, 0.40] };
        default:              return makeDemoInput('equal-4');
    }
}

function alphaLabels(n) {
    const out = [];
    for (let i = 0; i < n; i++) out.push(String.fromCharCode(65 + i));
    return out;
}

export function fmtHhi(v) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(4);
}

export function fmtScaled(v) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(0);
}

export function fmtEffN(v) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(2);
}

export function fmtPct(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtInt(v) {
    if (!Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
