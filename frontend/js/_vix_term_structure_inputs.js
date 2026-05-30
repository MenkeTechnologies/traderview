// VIX term-structure analyzer helpers.
//
// Backend body: VixTermStructure flat: { vix9d, vix, vix3m, vix6m, vix1y }.
// Returns: TermStructureReport { vix_to_vix3m_ratio, slope, state, note }.
//
// State buckets (mirror crates/traderview-core/src/vix_term_structure.rs):
//   ratio < 0.80 → steep_contango
//   ratio < 1.00 → contango
//   ratio < 1.05 → flat
//   ratio < 1.20 → backwardation
//   ratio ≥ 1.20 → severe_backwardation
//
// Boundary semantics: each transition is strict `<` (so 0.80 exact → contango,
// 1.00 exact → flat, 1.05 exact → backwardation, 1.20 exact → severe).

import { t } from './i18n.js';

export const STATES = [
    'steep_contango', 'contango', 'flat', 'backwardation', 'severe_backwardation',
];

export const TENORS = ['vix9d', 'vix', 'vix3m', 'vix6m', 'vix1y'];

export const TENOR_DAYS = { vix9d: 9, vix: 30, vix3m: 90, vix6m: 180, vix1y: 365 };

export const DEFAULT_INPUTS = {
    vix9d: 13, vix: 15, vix3m: 18, vix6m: 19, vix1y: 20,
};

export function validateInputs(ts) {
    for (const k of TENORS) {
        if (!Number.isFinite(ts[k])) return t('common.validate.field_must_be_finite', { field: k });
        if (ts[k] < 0) return t('common.validate.field_must_be_non_neg', { field: k });
    }
    return null;
}

export function buildBody(ts) {
    return {
        vix9d: ts.vix9d, vix: ts.vix, vix3m: ts.vix3m,
        vix6m: ts.vix6m, vix1y: ts.vix1y,
    };
}

// Pure-JS mirror of analyze().
export function localAnalyze(ts) {
    const out = {
        vix_to_vix3m_ratio: 0, slope: 0, state: 'flat', note_key: 'view.vix_term_structure.note.flat',
    };
    if (ts.vix3m <= 0) return out;
    const ratio = ts.vix / ts.vix3m;
    out.vix_to_vix3m_ratio = ratio;
    out.slope = (ts.vix - ts.vix9d) + (ts.vix3m - ts.vix)
              + (ts.vix6m - ts.vix3m) + (ts.vix1y - ts.vix6m);
    if      (ratio < 0.80) out.state = 'steep_contango';
    else if (ratio < 1.00) out.state = 'contango';
    else if (ratio < 1.05) out.state = 'flat';
    else if (ratio < 1.20) out.state = 'backwardation';
    else                   out.state = 'severe_backwardation';
    out.note_key = `view.vix_term_structure.note.${out.state}`;
    return out;
}

// State → badge color class for the verdict card.
const STATE_BADGES = {
    steep_contango:       { key: 'view.vix_term_structure.badge.steep_contango',  cls: 'pos' },
    contango:             { key: 'view.vix_term_structure.badge.contango',        cls: 'pos' },
    flat:                 { key: 'view.vix_term_structure.badge.flat',            cls: '' },
    backwardation:        { key: 'view.vix_term_structure.badge.backwardation',   cls: 'neg' },
    severe_backwardation: { key: 'view.vix_term_structure.badge.severe',          cls: 'neg' },
};

export function stateBadge(state) {
    return STATE_BADGES[state] || { key: 'view.vix_term_structure.badge.unknown', cls: '' };
}

// Per-tenor slope contribution — what fraction of total slope each
// consecutive-tenor difference contributes.
export function tenorContributions(ts) {
    if (!ts) return [];
    return [
        { from: 'vix9d',  to: 'vix',   delta: ts.vix   - ts.vix9d },
        { from: 'vix',    to: 'vix3m', delta: ts.vix3m - ts.vix },
        { from: 'vix3m',  to: 'vix6m', delta: ts.vix6m - ts.vix3m },
        { from: 'vix6m',  to: 'vix1y', delta: ts.vix1y - ts.vix6m },
    ];
}

// Demo presets matching every Rust state branch + a corner case.
export function makeDemoInput(kind = 'normal-contango') {
    switch (kind) {
        case 'steep-contango':
            return { vix9d: 10, vix: 12, vix3m: 18, vix6m: 19, vix1y: 20 };
        case 'normal-contango':
            return { vix9d: 13, vix: 15, vix3m: 18, vix6m: 19, vix1y: 20 };
        case 'flat':
            return { vix9d: 20, vix: 20, vix3m: 20, vix6m: 20, vix1y: 20 };
        case 'backwardation':
            return { vix9d: 28, vix: 25, vix3m: 23, vix6m: 22, vix1y: 22 };
        case 'severe':
            return { vix9d: 45, vix: 40, vix3m: 30, vix6m: 28, vix1y: 26 };
        case 'covid-spike':
            return { vix9d: 90, vix: 82, vix3m: 60, vix6m: 50, vix1y: 40 };
        case 'gfc-bear':
            return { vix9d: 75, vix: 70, vix3m: 55, vix6m: 50, vix1y: 45 };
        case 'low-vol-regime':
            // Front 9 vs 3M 14 → ratio 0.643 → steep contango.
            return { vix9d: 8, vix: 9, vix3m: 14, vix6m: 16, vix1y: 18 };
        default:
            return makeDemoInput('normal-contango');
    }
}

export function fmtN(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtSigned(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toFixed(d);
}

export function fmtRatio(v) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(3);
}
