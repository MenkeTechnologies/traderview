// Risk-parity allocator helpers shared by view + vitest.
//
// Backend body: { assets: [{symbol, vol}, ...] }.
// Returns: { allocations: [{symbol, weight, risk_contribution}], total_weight }.
//
// Naive risk parity: w_i ∝ 1/σ_i, normalize → Σw = 1. Each asset's
// risk_contribution = weight × vol = 1 / Σ(1/σ) — constant across assets.

import { t } from './i18n.js';

const TOKEN_DELIM = /[\s,]+/;

// Per line: "<symbol> <vol>"; vol > 0; %-suffix supported (auto / 100).
export function parseAssetBlob(text) {
    const assets = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { assets, errors: [{ line_no: 0, raw: '', message: t('view.risk_parity.parse.input_not_string') }] };
    }
    const lines = text.split(/\r?\n/);
    const seen = new Set();
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const noComment = stripComment(raw);
        const s = noComment.trim();
        if (!s) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 2) {
            errors.push({ line_no: i + 1, raw, message: t('view.risk_parity.parse.token_count', { n: parts.length }) });
            continue;
        }
        const sym = parts[0].toUpperCase();
        let volStr = parts[1];
        let div = 1;
        if (volStr.endsWith('%')) { volStr = volStr.slice(0, -1); div = 100; }
        const vol = Number(volStr) / div;
        if (!Number.isFinite(vol) || vol < 0) {
            errors.push({ line_no: i + 1, raw, message: t('view.risk_parity.parse.vol_invalid') });
            continue;
        }
        if (seen.has(sym)) {
            errors.push({ line_no: i + 1, raw, message: t('view.risk_parity.parse.duplicate_symbol', { sym }) });
            continue;
        }
        seen.add(sym);
        assets.push({ symbol: sym, vol });
    }
    return { assets, errors };
}

function stripComment(raw) {
    const i = raw.indexOf('#');
    return i >= 0 ? raw.slice(0, i) : raw;
}

export function validateInputs(assets) {
    if (!Array.isArray(assets) || assets.length === 0) return t('view.risk_parity.validate.assets_min');
    return null;
}

export function buildBody(assets) {
    return { assets: assets.map(a => ({ symbol: a.symbol, vol: a.vol })) };
}

// Pure-JS mirror of crates/traderview-core/src/risk_parity.rs::allocate.
// Zero-vol assets get zero weight; if all vols are zero or assets is
// empty, return default zeroed report.
export function localAllocate(assets) {
    const out = { allocations: [], total_weight: 0 };
    if (!Array.isArray(assets) || assets.length === 0) return out;
    const invVol = assets.map(a => (a.vol > 0 ? 1 / a.vol : 0));
    const total  = invVol.reduce((acc, v) => acc + v, 0);
    if (total <= 0) return out;
    for (let i = 0; i < assets.length; i++) {
        const w = invVol[i] / total;
        out.allocations.push({
            symbol: assets[i].symbol,
            weight: w,
            risk_contribution: w * assets[i].vol,
        });
    }
    out.total_weight = out.allocations.reduce((acc, a) => acc + a.weight, 0);
    return out;
}

// What the equal-weight allocation would be (1/n per asset) — useful
// to contrast with risk-parity in the chart.
export function equalWeightAllocation(assets) {
    if (!Array.isArray(assets) || assets.length === 0) return [];
    const w = 1 / assets.length;
    return assets.map(a => ({
        symbol: a.symbol, weight: w, risk_contribution: w * a.vol,
    }));
}

// Risk-contribution dispersion: max - min across assets (should be ~0
// for true risk parity). Diagnostic.
export function riskContribDispersion(allocations) {
    if (!Array.isArray(allocations) || allocations.length === 0) return 0;
    let lo = Infinity, hi = -Infinity;
    for (const a of allocations) {
        if (a.risk_contribution < lo) lo = a.risk_contribution;
        if (a.risk_contribution > hi) hi = a.risk_contribution;
    }
    return hi - lo;
}

// Concentration: max weight as a fraction of the total. Higher = more
// concentrated. Useful diagnostic next to plain weights.
export function maxConcentration(allocations) {
    if (!Array.isArray(allocations) || allocations.length === 0) return 0;
    let m = 0;
    for (const a of allocations) if (a.weight > m) m = a.weight;
    return m;
}

// Demo presets that exercise common allocation shapes.
export function makeDemoAssets(kind = 'classic-60-40') {
    switch (kind) {
        case 'classic-60-40':
            // SPY-vs-AGG. Stocks ~15% vol, bonds ~5% vol → bonds get ~75% RP weight.
            return [
                { symbol: 'SPY', vol: 0.15 },
                { symbol: 'AGG', vol: 0.05 },
            ];
        case 'five-asset':
            // Diversified across stocks, bonds, gold, EM, REITs.
            return [
                { symbol: 'SPY', vol: 0.15 },
                { symbol: 'AGG', vol: 0.05 },
                { symbol: 'GLD', vol: 0.16 },
                { symbol: 'EEM', vol: 0.22 },
                { symbol: 'VNQ', vol: 0.20 },
            ];
        case 'equal-vol':
            // 3 assets at identical vol → equal-weighted by RP too.
            return [
                { symbol: 'A', vol: 0.20 },
                { symbol: 'B', vol: 0.20 },
                { symbol: 'C', vol: 0.20 },
            ];
        case 'extreme-vol':
            // One steady asset, one highly volatile → big weight skew.
            return [
                { symbol: 'STEADY', vol: 0.05 },
                { symbol: 'VOLATILE', vol: 0.50 },
            ];
        case 'single':
            return [{ symbol: 'ONLY', vol: 0.20 }];
        case 'zero-vol-mixed':
            // Demonstrates zero-vol guard: cash-equivalent gets 0 weight.
            return [
                { symbol: 'CASH', vol: 0.00 },
                { symbol: 'SPY',  vol: 0.15 },
                { symbol: 'AGG',  vol: 0.05 },
            ];
        default:
            return makeDemoAssets('classic-60-40');
    }
}

export function fmtPct(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtVol(v, d = 1) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtNum(v, d = 6) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

const PALETTE = ['#00e5ff', '#ffd84a', '#ff3860', '#23d18b', '#c678dd', '#ffa657', '#5fd0ff'];
export function symbolColor(i) {
    if (!Number.isInteger(i) || i < 0) return '#aab';
    return PALETTE[i % PALETTE.length];
}
