// Portfolio Stress-Test helpers shared by view + vitest.
//
// Backend body shape (flat StressInput): { legs, price_shocks_pct,
// iv_shocks_pct, time_decay_days, risk_free_rate, dividend_yield }.
// Returns { grid: StressCell[], worst_case, best_case }.

import { t } from './i18n.js';

const TOKEN_DELIM = /[\s,]+/;
const VALID_KINDS = new Set(['call', 'put']);

// Eight-token-per-line "symbol kind spot strike days_to_expiry iv contracts entry_price".
//   kind = call|put (case-insensitive)
//   contracts = signed int (positive long, negative short)
export function parseLegBlob(text) {
    const legs = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { legs, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.trim();
        if (!s || s.startsWith('#')) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 8) {
            errors.push({ line_no: i + 1, raw, message: `expected 8 tokens (symbol kind spot strike dte iv contracts entry), got ${parts.length}` });
            continue;
        }
        const symbol = parts[0].toUpperCase();
        const kind = String(parts[1]).toLowerCase();
        const spot = Number(parts[2]);
        const strike = Number(parts[3]);
        const dte = Number(parts[4]);
        const iv = Number(parts[5]);
        const contractsNum = Number(parts[6]);
        const entry = Number(parts[7]);
        if (!/^[A-Z0-9._-]+$/.test(symbol)) {
            errors.push({ line_no: i + 1, raw, message: `bad symbol "${parts[0]}"` });
            continue;
        }
        if (!VALID_KINDS.has(kind)) {
            errors.push({ line_no: i + 1, raw, message: `kind must be call|put (got "${parts[1]}")` });
            continue;
        }
        if (!Number.isFinite(spot) || spot <= 0) {
            errors.push({ line_no: i + 1, raw, message: `spot must be > 0` });
            continue;
        }
        if (!Number.isFinite(strike) || strike <= 0) {
            errors.push({ line_no: i + 1, raw, message: `strike must be > 0` });
            continue;
        }
        if (!Number.isFinite(dte) || dte < 0) {
            errors.push({ line_no: i + 1, raw, message: `days_to_expiry must be ≥ 0` });
            continue;
        }
        if (!Number.isFinite(iv) || iv < 0) {
            errors.push({ line_no: i + 1, raw, message: `iv must be ≥ 0` });
            continue;
        }
        if (!Number.isFinite(contractsNum) || !Number.isInteger(contractsNum) || contractsNum === 0) {
            errors.push({ line_no: i + 1, raw, message: `contracts must be non-zero integer (signed; + long, − short)` });
            continue;
        }
        if (!Number.isFinite(entry) || entry < 0) {
            errors.push({ line_no: i + 1, raw, message: `entry_price must be ≥ 0` });
            continue;
        }
        legs.push({
            symbol, kind, spot, strike,
            days_to_expiry: dte, implied_vol: iv,
            contracts: contractsNum,
            multiplier: 100,
            entry_price: entry,
        });
    }
    return { legs, errors };
}

export function validateInputs(legs, priceShocks, ivShocks, timeDecay, rate, div) {
    if (!Array.isArray(legs) || legs.length === 0) return t('view.stress_test.validate.legs_empty');
    if (!Array.isArray(priceShocks) || priceShocks.length === 0) return t('view.stress_test.validate.price_shocks_empty');
    if (!Array.isArray(ivShocks) || ivShocks.length === 0) return t('view.stress_test.validate.iv_shocks_empty');
    if (!Number.isFinite(timeDecay) || timeDecay < 0) return t('view.stress_test.validate.time_decay');
    if (!Number.isFinite(rate)) return t('view.stress_test.validate.rate');
    if (!Number.isFinite(div) || div < 0) return t('view.stress_test.validate.div_yield');
    if (!priceShocks.every(Number.isFinite) || !ivShocks.every(Number.isFinite))
        return t('view.stress_test.validate.shocks_finite');
    return null;
}

export function buildBody(legs, priceShocks, ivShocks, timeDecay, rate, div) {
    return {
        legs, price_shocks_pct: priceShocks, iv_shocks_pct: ivShocks,
        time_decay_days: timeDecay, risk_free_rate: rate, dividend_yield: div,
    };
}

// Default shock ladders — symmetric around zero with reasonable steps.
export function defaultPriceShocks() {
    return [-0.10, -0.07, -0.05, -0.03, -0.01, 0, 0.01, 0.03, 0.05, 0.07, 0.10];
}

export function defaultIvShocks() {
    return [-0.30, -0.20, -0.10, 0, 0.10, 0.20, 0.30];
}

// Pivots the flat grid response into a 2D [priceIdx][ivIdx] matrix for
// heatmap rendering. Backend iterates priceShocks outer, ivShocks inner —
// so cell index = pIdx × ivShocks.length + ivIdx.
export function pivotGrid(grid, priceShocks, ivShocks) {
    const matrix = Array.from({ length: priceShocks.length },
        () => new Array(ivShocks.length).fill(null));
    if (!Array.isArray(grid)) return matrix;
    for (let pi = 0; pi < priceShocks.length; pi++) {
        for (let ii = 0; ii < ivShocks.length; ii++) {
            const idx = pi * ivShocks.length + ii;
            matrix[pi][ii] = grid[idx] || null;
        }
    }
    return matrix;
}

// Color-tier classifier for heatmap cells. Linear interpolation between
// max-loss (red) and max-gain (green), with a 5-tier discrete bucket so
// CSS-class assignment stays clean. Returns 'heat-pos-1..4' / 'heat-neg-1..4'
// / 'heat-empty' — same CSS scheme used by Intraday Heatmap.
export function heatStyleClass(pnl, maxAbs) {
    if (!Number.isFinite(pnl) || pnl === 0 || maxAbs <= 0) return 'heat-empty';
    const intensity = Math.min(1, Math.abs(pnl) / maxAbs);
    const tier = intensity < 0.25 ? 1
              : intensity < 0.50 ? 2
              : intensity < 0.75 ? 3
              : 4;
    return pnl > 0 ? `heat-pos-${tier}` : `heat-neg-${tier}`;
}

// Demo: short iron condor on a 100-strike underlying.
//   short put 95, long put 90, short call 105, long call 110
// At spot=100, expires in 30 days, 30% IV. Engineered so the demo grid
// shows the canonical "max-gain at center, losses on the wings"
// stress-test fingerprint of a condor.
export function makeDemoLegs() {
    const base = {
        symbol: 'SPY', spot: 100, days_to_expiry: 30, implied_vol: 0.30,
        multiplier: 100,
    };
    return [
        { ...base, kind: 'put',  strike: 95,  contracts: -1, entry_price: 1.20 },
        { ...base, kind: 'put',  strike: 90,  contracts:  1, entry_price: 0.40 },
        { ...base, kind: 'call', strike: 105, contracts: -1, entry_price: 1.20 },
        { ...base, kind: 'call', strike: 110, contracts:  1, entry_price: 0.40 },
    ];
}

export function fmtUSD(v) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-$' : '$';
    return sign + Math.abs(v).toFixed(0);
}

export function fmtUSDSigned(v) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-$' : '+$';
    return sign + Math.abs(v).toFixed(0);
}

export function fmtPct(v) {
    if (!Number.isFinite(v)) return '—';
    const sign = v >= 0 ? '+' : '';
    return sign + (v * 100).toFixed(1) + '%';
}
