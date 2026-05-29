// Per-Symbol Slippage helpers shared by view + vitest.
//
// Backend body shape: { records: [{symbol, slippage_bps}, ...] }.
// Negative slippage = trader paid up (bad). Positive = trader captured
// liquidity / beat benchmark (good).

import { t } from './i18n.js';

const TOKEN_DELIM = /[\s,]+/;

export function parseRecordBlob(text) {
    const records = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { records, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.trim();
        if (!s || s.startsWith('#')) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 2) {
            errors.push({ line_no: i + 1, raw, message: `expected 2 tokens (symbol slippage_bps), got ${parts.length}` });
            continue;
        }
        const symbol = parts[0].toUpperCase();
        const bps = Number(parts[1]);
        if (!/^[A-Z0-9._-]+$/.test(symbol)) {
            errors.push({ line_no: i + 1, raw, message: `bad symbol "${parts[0]}"` });
            continue;
        }
        if (!Number.isFinite(bps)) {
            errors.push({ line_no: i + 1, raw, message: `slippage_bps must be finite` });
            continue;
        }
        records.push({ symbol, slippage_bps: bps });
    }
    return { records, errors };
}

export function validateInputs(records) {
    if (!Array.isArray(records) || records.length === 0) return 'need at least 1 record';
    return null;
}

export function buildBody(records) {
    return { records };
}

// Single-glance verdict per symbol — used for both the table row color
// and the summary's "best/worst symbol" cards.
export function executionGrade(meanBps) {
    if (!Number.isFinite(meanBps)) return { label: '—', cls: '' };
    if (meanBps > 5)   return { label: t('view.per_symbol_slippage.grade.excellent'), cls: 'pos' };
    if (meanBps > 0)   return { label: t('view.per_symbol_slippage.grade.good'),      cls: 'pos' };
    if (meanBps > -5)  return { label: t('view.per_symbol_slippage.grade.neutral'),   cls: '' };
    if (meanBps > -15) return { label: t('view.per_symbol_slippage.grade.poor'),      cls: 'neg' };
    return                  { label: t('view.per_symbol_slippage.grade.terrible'),  cls: 'neg' };
}

// Picks the worst-mean entry from a backend response. Backend already
// returns worst-first, but the view treats this as an explicit lookup
// to stay robust against backend ordering changes.
export function worstSymbol(report) {
    if (!Array.isArray(report) || report.length === 0) return null;
    let worst = report[0];
    for (const r of report) {
        if (Number.isFinite(r.mean_bps) && (!Number.isFinite(worst.mean_bps) || r.mean_bps < worst.mean_bps)) {
            worst = r;
        }
    }
    return worst;
}

export function bestSymbol(report) {
    if (!Array.isArray(report) || report.length === 0) return null;
    let best = report[0];
    for (const r of report) {
        if (Number.isFinite(r.mean_bps) && (!Number.isFinite(best.mean_bps) || r.mean_bps > best.mean_bps)) {
            best = r;
        }
    }
    return best;
}

// Deterministic seeded demo: 6 symbols spanning the execution-quality
// spectrum from "terrible" (penny stocks blowing through your size) to
// "excellent" (ETFs you got a discount on). Guarantees diverse grades.
export function makeDemoRecords(seed = 1) {
    let s = seed;
    const rand = () => { s = (s * 1664525 + 1013904223) | 0; return ((s >>> 0) / 0xffffffff); };
    const config = [
        // [symbol, mean_bps, stdev_bps, count]
        ['SPY',  +6.5,  3,   30],   // ETF, tight, slight edge → EXCELLENT
        ['QQQ',  +1.5,  4,   25],   // ETF, neutral → GOOD
        ['AAPL', -1.0,  6,   20],   // large cap, mild paying → NEUTRAL
        ['TSLA', -8.0,  10,  15],   // volatile, big size → POOR
        ['SMID', -18.0, 12,  10],   // small cap → TERRIBLE
        ['ILQD', -25.0, 18,  8],    // micro cap → TERRIBLE
    ];
    const out = [];
    for (const [sym, mean, sd, count] of config) {
        for (let i = 0; i < count; i++) {
            const bps = mean + (rand() - 0.5) * sd * 2;
            out.push({ symbol: sym, slippage_bps: Number(bps.toFixed(2)) });
        }
    }
    return out;
}

export function fmtBps(v) {
    if (!Number.isFinite(v)) return '—';
    const sign = v >= 0 ? '+' : '';
    return sign + v.toFixed(1) + ' bps';
}

export function fmtPct(v) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(0) + '%';
}

export function fmtN(v) {
    if (!Number.isFinite(v)) return '—';
    return v.toLocaleString('en-US');
}
