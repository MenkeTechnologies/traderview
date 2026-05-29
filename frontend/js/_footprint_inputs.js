// Footprint chart helpers shared by view + vitest.
//
// Backend body shape: { ticks: BarTick[], tick_size: f64 }, where
// BarTick = { bar_id, price, classified: { volume, side } }.
// The user pastes simple 4-token rows; we synthesize the ClassifiedTick
// shape required by the backend.

import { t } from './i18n.js';

const TOKEN_DELIM = /[\s,]+/;
const VALID_SIDES = new Set(['buy', 'sell', 'uncertain']);

// Four-token-per-line `bar_id price volume side`.
// `side` ∈ {buy, sell, uncertain} — case-insensitive.
export function parseTickBlob(text) {
    const ticks = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { ticks, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.trim();
        if (!s || s.startsWith('#')) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 4) {
            errors.push({ line_no: i + 1, raw, message: `expected 4 tokens (bar_id price volume side), got ${parts.length}` });
            continue;
        }
        const barIdNum = Number(parts[0]);
        const price = Number(parts[1]);
        const volume = Number(parts[2]);
        const side = String(parts[3]).toLowerCase();
        if (!Number.isFinite(barIdNum) || !Number.isInteger(barIdNum) || barIdNum < 0) {
            errors.push({ line_no: i + 1, raw, message: `bar_id must be non-negative integer` });
            continue;
        }
        if (!Number.isFinite(price) || price <= 0) {
            errors.push({ line_no: i + 1, raw, message: `price must be > 0` });
            continue;
        }
        if (!Number.isFinite(volume) || volume <= 0) {
            errors.push({ line_no: i + 1, raw, message: `volume must be > 0` });
            continue;
        }
        if (!VALID_SIDES.has(side)) {
            errors.push({ line_no: i + 1, raw, message: `side must be buy/sell/uncertain (got "${parts[3]}")` });
            continue;
        }
        ticks.push({
            bar_id: barIdNum,
            price,
            classified: { volume, side },
        });
    }
    return { ticks, errors };
}

export function validateInputs(ticks, tickSize) {
    if (!Array.isArray(ticks) || ticks.length === 0) return t('view.footprint.validate.ticks_empty');
    if (!Number.isFinite(tickSize) || tickSize <= 0) return t('view.footprint.validate.tick_size');
    return null;
}

export function buildBody(ticks, tickSize) {
    return { ticks, tick_size: tickSize };
}

// Delta CSS-class picker matching the divergent-bar pattern used
// elsewhere — positive delta = bullish absorption (pos), negative =
// bearish rejection (neg). Zero stays neutral.
export function deltaCls(delta) {
    if (!Number.isFinite(delta) || delta === 0) return '';
    return delta > 0 ? 'pos' : 'neg';
}

// Aggregate scalars for the summary cards.
export function summarize(report) {
    if (!report || !Array.isArray(report.bars)) {
        return { barCount: 0, totalVolume: 0, totalDelta: 0,
                 maxAbsDelta: 0, lastPoc: null };
    }
    let totalVolume = 0;
    let totalDelta = 0;
    let maxAbsDelta = 0;
    for (const b of report.bars) {
        totalVolume += (b.total_volume || 0);
        totalDelta  += (b.total_delta  || 0);
        const a = Math.abs(b.total_delta || 0);
        if (a > maxAbsDelta) maxAbsDelta = a;
    }
    const last = report.bars[report.bars.length - 1];
    return {
        barCount: report.bars.length,
        totalVolume, totalDelta, maxAbsDelta,
        lastPoc: last ? last.poc_price : null,
    };
}

// Walks each bar's cells to find the cell with the largest abs delta —
// the "imbalance hot-spot." Used to surface absorption / rejection
// candles where one side dominated a single price level.
export function imbalanceHotspots(report, topN = 5) {
    if (!report || !Array.isArray(report.bars)) return [];
    const all = [];
    for (const b of report.bars) {
        for (const c of (b.cells || [])) {
            all.push({
                bar_id: b.bar_id,
                price:  c.price,
                bid:    c.bid_volume,
                ask:    c.ask_volume,
                delta:  c.delta,
            });
        }
    }
    all.sort((a, b) => Math.abs(b.delta) - Math.abs(a.delta));
    return all.slice(0, topN);
}

// Deterministic 4-bar demo with engineered patterns:
//   Bar 0: balanced churn
//   Bar 1: absorption at the low (heavy bid volume but price held)
//   Bar 2: drive up (consistent ask dominance across levels)
//   Bar 3: rejection at the high (heavy ask volume but price reversed)
export function makeDemoTicks() {
    const out = [];
    const push = (bar, price, volume, side) =>
        out.push({ bar_id: bar, price, classified: { volume, side } });

    // Bar 0 — balanced.
    push(0, 100.00,  50, 'buy');
    push(0, 100.00,  50, 'sell');
    push(0, 100.05, 100, 'buy');
    push(0, 100.05, 100, 'sell');
    push(0,  99.95,  75, 'buy');
    push(0,  99.95,  75, 'sell');

    // Bar 1 — absorption at the low (lots of bid hitting at 99.85 but price doesn't break).
    push(1, 100.00, 100, 'sell');
    push(1,  99.95, 200, 'sell');
    push(1,  99.90, 300, 'sell');
    push(1,  99.85, 500, 'buy');   // huge buy absorption at the bottom
    push(1,  99.85, 500, 'buy');
    push(1,  99.90, 150, 'buy');

    // Bar 2 — drive up (consistent ask dominance).
    push(2,  99.95,  20, 'sell');
    push(2, 100.00, 200, 'buy');
    push(2, 100.05, 350, 'buy');
    push(2, 100.10, 400, 'buy');
    push(2, 100.15, 500, 'buy');

    // Bar 3 — rejection at the high (heavy ask volume but price returns to start).
    push(3, 100.15, 300, 'buy');
    push(3, 100.20, 700, 'sell');  // big sell wall at top
    push(3, 100.20, 600, 'sell');
    push(3, 100.15, 200, 'sell');
    push(3, 100.10, 100, 'sell');
    push(3, 100.05,  50, 'sell');
    return out;
}

export function fmtN(v, d = 0) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtPrice(v, tickSize) {
    if (!Number.isFinite(v)) return '—';
    // Pick decimals matching tick_size magnitude (0.01 → 2, 0.001 → 3, etc).
    let d = 2;
    if (Number.isFinite(tickSize) && tickSize > 0) {
        d = Math.max(0, -Math.floor(Math.log10(tickSize)));
    }
    return v.toFixed(d);
}

export function fmtSigned(v) {
    if (!Number.isFinite(v)) return '—';
    const sign = v >= 0 ? '+' : '';
    return sign + v.toFixed(0);
}
