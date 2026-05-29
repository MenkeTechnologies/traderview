// Cohort-tilt helpers shared by view + vitest.
//
// Backend body: { positions: [{trader_id, symbol, net_contracts}, ...] }.
// Returns: { by_symbol: SymbolTilt[], active_traders, most_lopsided_symbol }
// where SymbolTilt = { symbol, long_traders, short_traders, flat_traders,
//   net_contracts, long_ratio: f64 | null, bias: TiltBias snake_case }.
//
// Bias thresholds match Rust ::classify exactly:
//   ≥ 0.75 strongly_long, ≥ 0.60 long, ≥ 0.40 balanced,
//   ≥ 0.25 short, else strongly_short.

const TOKEN_DELIM = /[\s,]+/;

// Per line: "<trader_id> <SYMBOL> <net_contracts>". Symbol uppercased.
// trader_id stays as-is (anonymized handle).
export function parsePositionBlob(text) {
    const positions = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { positions, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const noComment = stripComment(raw);
        const s = noComment.trim();
        if (!s) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 3) {
            errors.push({ line_no: i + 1, raw, message: `expected 3 tokens (trader_id symbol contracts), got ${parts.length}` });
            continue;
        }
        const trader_id = parts[0];
        const symbol    = parts[1].toUpperCase();
        const contracts = Number(parts[2]);
        if (!Number.isInteger(contracts)) {
            errors.push({ line_no: i + 1, raw, message: 'net_contracts must be integer (signed: + long, - short, 0 flat)' });
            continue;
        }
        positions.push({ trader_id, symbol, net_contracts: contracts });
    }
    return { positions, errors };
}

function stripComment(raw) {
    const i = raw.indexOf('#');
    return i >= 0 ? raw.slice(0, i) : raw;
}

export function validateInputs(positions) {
    if (!Array.isArray(positions) || positions.length === 0)
        return 'need ≥ 1 position';
    return null;
}

export function buildBody(positions) {
    return { positions };
}

// Pure-JS mirror of crates/traderview-core/src/tilt_indicator.rs::aggregate.
// Uses Map (string keys) for the by-symbol bucket; preserves insertion
// order but the final sort matches Rust (lopsidedness desc, ties stable).
export function localAggregate(positions) {
    if (!Array.isArray(positions) || positions.length === 0) {
        return { by_symbol: [], active_traders: 0, most_lopsided_symbol: null };
    }
    const bySym = new Map();
    const activeSet = new Set();
    for (const p of positions) {
        if (!bySym.has(p.symbol)) {
            bySym.set(p.symbol, { long_t: 0, short_t: 0, flat_t: 0, net: 0 });
        }
        const e = bySym.get(p.symbol);
        if (p.net_contracts > 0)      { e.long_t++;  activeSet.add(p.trader_id); }
        else if (p.net_contracts < 0) { e.short_t++; activeSet.add(p.trader_id); }
        else                          { e.flat_t++; }
        e.net += p.net_contracts;
    }
    const by_symbol = [];
    for (const [symbol, e] of bySym.entries()) {
        const totalPositioned = e.long_t + e.short_t;
        const long_ratio = totalPositioned === 0 ? null : e.long_t / totalPositioned;
        const bias = long_ratio == null ? 'balanced' : classify(long_ratio);
        by_symbol.push({
            symbol,
            long_traders: e.long_t, short_traders: e.short_t, flat_traders: e.flat_t,
            net_contracts: e.net, long_ratio, bias,
        });
    }
    by_symbol.sort((a, b) => {
        const aw = a.long_ratio == null ? 0 : Math.abs(a.long_ratio - 0.5);
        const bw = b.long_ratio == null ? 0 : Math.abs(b.long_ratio - 0.5);
        return bw - aw;
    });
    return {
        by_symbol,
        active_traders: activeSet.size,
        most_lopsided_symbol: by_symbol.length > 0 ? by_symbol[0].symbol : null,
    };
}

export function classify(longRatio) {
    if (!Number.isFinite(longRatio)) return 'balanced';
    if (longRatio >= 0.75) return 'strongly_long';
    if (longRatio >= 0.60) return 'long';
    if (longRatio >= 0.40) return 'balanced';
    if (longRatio >= 0.25) return 'short';
    return 'strongly_short';
}

const BIAS_BADGES = {
    strongly_long:  { label: 'STRONGLY LONG',  cls: 'pos', hint: '≥75% of positioned traders are long — squeeze risk elevated.' },
    long:           { label: 'LONG',           cls: 'pos', hint: '60–75% long.' },
    balanced:       { label: 'BALANCED',       cls: '',    hint: '40–60% long — no consensus.' },
    short:          { label: 'SHORT',          cls: 'neg', hint: '25–40% long (i.e., 60–75% short).' },
    strongly_short: { label: 'STRONGLY SHORT', cls: 'neg', hint: '≤25% long — short-squeeze fuel.' },
};

export function biasBadge(bias) {
    return BIAS_BADGES[bias] || { label: String(bias || '—').toUpperCase(), cls: '', hint: '—' };
}

// Lopsidedness = |long_ratio - 0.5|; null → 0.
export function lopsidedness(sym) {
    if (!sym || sym.long_ratio == null) return 0;
    return Math.abs(sym.long_ratio - 0.5);
}

// Cohort-wide bias from a weighted average of long_ratios (weighted by
// positioned traders). Useful summary scalar.
export function cohortLongRatio(report) {
    if (!report || !Array.isArray(report.by_symbol)) return null;
    let num = 0, den = 0;
    for (const s of report.by_symbol) {
        if (s.long_ratio == null) continue;
        const positioned = s.long_traders + s.short_traders;
        num += s.long_ratio * positioned;
        den += positioned;
    }
    return den > 0 ? num / den : null;
}

// Demo presets exercising each bias bucket.
export function makeDemoPositions(kind = 'mixed') {
    switch (kind) {
        case 'mixed': {
            // ES strongly long (8L/2S → 0.80), NQ balanced (5L/5S → 0.50),
            // CL strongly short (1L/4S → 0.20), GC has all flat traders.
            const out = [];
            for (let i = 0; i < 8; i++) out.push({ trader_id: `L${i}`, symbol: 'ES', net_contracts:  3 });
            for (let i = 0; i < 2; i++) out.push({ trader_id: `S${i}`, symbol: 'ES', net_contracts: -3 });
            for (let i = 0; i < 5; i++) out.push({ trader_id: `LN${i}`, symbol: 'NQ', net_contracts: 2 });
            for (let i = 0; i < 5; i++) out.push({ trader_id: `SN${i}`, symbol: 'NQ', net_contracts: -2 });
            out.push({ trader_id: 'a', symbol: 'CL', net_contracts:  1 });
            for (let i = 0; i < 4; i++) out.push({ trader_id: `b${i}`, symbol: 'CL', net_contracts: -2 });
            for (let i = 0; i < 3; i++) out.push({ trader_id: `f${i}`, symbol: 'GC', net_contracts:  0 });
            return out;
        }
        case 'strongly-long': {
            const out = [];
            for (let i = 0; i < 8; i++) out.push({ trader_id: `L${i}`, symbol: 'ES', net_contracts:  3 });
            for (let i = 0; i < 2; i++) out.push({ trader_id: `S${i}`, symbol: 'ES', net_contracts: -3 });
            return out;
        }
        case 'strongly-short': {
            return [
                { trader_id: 'a', symbol: 'NQ', net_contracts:  1 },
                { trader_id: 'b', symbol: 'NQ', net_contracts: -2 },
                { trader_id: 'c', symbol: 'NQ', net_contracts: -2 },
                { trader_id: 'd', symbol: 'NQ', net_contracts: -2 },
                { trader_id: 'e', symbol: 'NQ', net_contracts: -2 },
            ];
        }
        case 'all-flat': {
            return [
                { trader_id: 'a', symbol: 'ES', net_contracts: 0 },
                { trader_id: 'b', symbol: 'ES', net_contracts: 0 },
                { trader_id: 'c', symbol: 'ES', net_contracts: 0 },
            ];
        }
        case 'cross-symbol': {
            // 1 trader long ES + same trader short NQ. Active = 1.
            return [
                { trader_id: 'a', symbol: 'ES', net_contracts:  1 },
                { trader_id: 'a', symbol: 'NQ', net_contracts: -1 },
                { trader_id: 'b', symbol: 'ES', net_contracts: -1 },
            ];
        }
        default:
            return makeDemoPositions('mixed');
    }
}

export function fmtPct(v, d = 1) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtSignedInt(v) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + Math.trunc(v).toString();
}

const PALETTE = ['#00e5ff', '#ffd84a', '#ff3860', '#23d18b', '#c678dd', '#ffa657'];
export function symbolColor(idx) {
    if (!Number.isInteger(idx) || idx < 0) return '#aab';
    return PALETTE[idx % PALETTE.length];
}
