// TWAP (Time-Weighted Average Price) helpers shared by view + vitest.
//
// Backend body shape: TwapInput { side: "long"|"short", fill_price:
// Decimal-string, typical_prices: Decimal-string[] }. Returns wrapper
// { result: Option<TwapResult> }.
//
// Distinct from VwapSlippage: weights each bar EQUALLY (arithmetic mean
// of typical prices), no volume consideration. Used when the trader was
// working a passive limit and cares about time-in-market rather than
// volume-participation rate.

import { parseFloatBlob } from './_paste_parser.js';

// Single-column paste — one typical price per line. Reuse shared parser
// with nonNegative gate (prices can't go below zero).
export function parseTypicals(text) {
    return parseFloatBlob(text, { nonNegative: true });
}

export function validateInputs(side, fillPrice, typicals) {
    if (side !== 'long' && side !== 'short') return 'side must be long or short';
    if (!Number.isFinite(fillPrice) || fillPrice <= 0) return 'fill_price must be > 0';
    if (!Array.isArray(typicals) || typicals.length === 0) return 'need at least 1 typical price';
    if (!typicals.every(v => Number.isFinite(v) && v > 0))
        return 'typical_prices must all be > 0';
    return null;
}

export function buildBody(side, fillPrice, typicals) {
    return {
        side,
        fill_price: String(fillPrice),
        typical_prices: typicals.map(p => String(p)),
    };
}

// Mirrors backend's arithmetic-mean formula for local parity check.
export function localTwap(typicals) {
    const xs = (typicals || []).filter(Number.isFinite);
    if (xs.length === 0) return NaN;
    return xs.reduce((a, b) => a + b, 0) / xs.length;
}

// Per-bar rolling TWAP for chart overlay: bar i's value = mean of
// typicals[0..=i]. Lets the trader trace where TWAP was at any
// snapshot during the exposure window.
export function rollingTwap(typicals) {
    const out = [];
    if (!Array.isArray(typicals)) return out;
    let sum = 0, n = 0;
    for (const v of typicals) {
        if (!Number.isFinite(v)) { out.push(null); continue; }
        sum += v;
        n += 1;
        out.push(sum / n);
    }
    return out;
}

// Backend Decimal scalars come back as strings.
export function decToNum(v) {
    if (v == null) return NaN;
    if (typeof v === 'number') return v;
    const n = Number(v);
    return Number.isFinite(n) ? n : NaN;
}

// Unwraps backend's TwapResp { result: Option<TwapResult> } envelope.
export function unwrapResponse(resp) {
    if (!resp || typeof resp !== 'object') return { ok: false, reason: 'malformed response' };
    if (resp.result === null || resp.result === undefined)
        return { ok: false, reason: 'backend returned null (empty typicals?)' };
    return { ok: true, result: resp.result };
}

// Deterministic 200-bar typical series with a long fill engineered at
// a ~12 bps discount to the arithmetic mean — guarantees beat_twap=true
// across all tested LCG seeds.
export function makeDemoData(seed = 1) {
    let s = seed;
    const rand = () => { s = (s * 1664525 + 1013904223) | 0; return ((s >>> 0) / 0xffffffff); };
    const typicals = new Array(200);
    let price = 100;
    for (let i = 0; i < 200; i++) {
        price = Math.max(0.01, price + (rand() - 0.45) * 0.08);
        typicals[i] = Number(price.toFixed(4));
    }
    const avg = typicals.reduce((a, b) => a + b, 0) / typicals.length;
    const fill_price = Number((avg * 0.9988).toFixed(4));
    return { side: 'long', fill_price, typicals };
}

export function fmtN(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtBps(v) {
    if (!Number.isFinite(v)) return '—';
    const sign = v >= 0 ? '+' : '';
    return sign + v.toFixed(1) + ' bps';
}
