// Intraday Heatmap helpers shared by view + vitest.
//
// Backend body shape: { trades: [{when: ISO-8601, pnl: f64}, ...] }.
// The view accepts either:
//   * Full ISO timestamps  ("2024-01-15T14:30:00Z 125.50")
//   * Bare time-of-day     ("14:30 125.50")  — fills the date with a
//     stable epoch (2024-01-01) so all samples land on one day.

import { t } from './i18n.js';

const TOKEN_DELIM = /[\s,]+/;
const EPOCH_DATE = '2024-01-01';

// Parses two-token-per-line: "<timestamp> <pnl>".
//
// First token can be either:
//   - Full ISO 8601 ("2024-01-15T14:30:00Z")
//   - Time-of-day "HH:MM" or "HH:MM:SS" — anchored to EPOCH_DATE.
//
// Returns ISO-8601 strings the backend chrono can deserialize directly.
export function parseTradeBlob(text) {
    const trades = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { trades, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.trim();
        if (!s || s.startsWith('#')) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 2) {
            errors.push({ line_no: i + 1, raw, message: `expected 2 tokens (when pnl), got ${parts.length}` });
            continue;
        }
        const whenStr = normalizeTimestamp(parts[0]);
        if (!whenStr) {
            errors.push({ line_no: i + 1, raw, message: `bad timestamp "${parts[0]}" (need ISO 8601 or HH:MM)` });
            continue;
        }
        const pnl = Number(parts[1]);
        if (!Number.isFinite(pnl)) {
            errors.push({ line_no: i + 1, raw, message: `pnl must be finite` });
            continue;
        }
        trades.push({ when: whenStr, pnl });
    }
    return { trades, errors };
}

// Returns null when the token can't be coerced into an ISO-8601 string.
export function normalizeTimestamp(tok) {
    if (typeof tok !== 'string' || !tok) return null;
    // HH:MM or HH:MM:SS — anchor to EPOCH_DATE.
    const timeMatch = /^(\d{1,2}):(\d{2})(?::(\d{2}))?$/.exec(tok);
    if (timeMatch) {
        const h = parseInt(timeMatch[1], 10);
        const m = parseInt(timeMatch[2], 10);
        const s = timeMatch[3] ? parseInt(timeMatch[3], 10) : 0;
        if (h > 23 || m > 59 || s > 59) return null;
        return `${EPOCH_DATE}T${pad2(h)}:${pad2(m)}:${pad2(s)}Z`;
    }
    // Full ISO 8601 — validate by round-tripping Date.
    const d = new Date(tok);
    if (Number.isNaN(d.getTime())) return null;
    return d.toISOString();
}

function pad2(n) { return n.toString().padStart(2, '0'); }

export function validateInputs(trades) {
    if (!Array.isArray(trades) || trades.length === 0) return t('view.intraday_heatmap.validate.trades_empty');
    return null;
}

export function buildBody(trades) {
    return { trades };
}

// Re-aggregates the 96 backend buckets into hourly rows × 4 quarter-hour
// columns, returning a 24×4 grid plus the global max-abs for color scaling.
export function gridify(buckets) {
    const grid = Array.from({ length: 24 }, () => new Array(4).fill(null));
    let maxAbs = 0;
    for (const b of buckets || []) {
        if (b.hour < 24 && b.minute % 15 === 0) {
            grid[b.hour][b.minute / 15] = b;
            if (Math.abs(b.total_pnl) > maxAbs) maxAbs = Math.abs(b.total_pnl);
        }
    }
    return { grid, maxAbs };
}

// Picks a heat color for a bucket given the global max-abs scale. Empty
// buckets get a flat dark cell. Greens for positive, reds for negative,
// intensity proportional to |pnl| / maxAbs.
export function heatStyleClass(pnl, maxAbs) {
    if (!Number.isFinite(pnl) || pnl === 0 || maxAbs <= 0) return 'heat-empty';
    const intensity = Math.min(1, Math.abs(pnl) / maxAbs);
    const tier = intensity < 0.25 ? 1
              : intensity < 0.50 ? 2
              : intensity < 0.75 ? 3
              : 4;
    return pnl > 0 ? `heat-pos-${tier}` : `heat-neg-${tier}`;
}

// Deterministic 200-trade demo clustered around US session hours with
// a profitable 09:30-10:00 momo window and a losing 11:30-12:00 chop window.
export function makeDemoTrades(seed = 1) {
    let s = seed;
    const rand = () => { s = (s * 1664525 + 1013904223) | 0; return ((s >>> 0) / 0xffffffff); };
    const out = [];
    const sessionMinutes = [];
    // Build a weighted minute distribution: heavy at 09:30 and 11:30, light at 14:00-15:00, very light pre/post.
    for (let h = 9; h <= 15; h++) {
        for (let m = 0; m < 60; m += 5) {
            const weight = (h === 9 && m >= 30) || (h === 11 && m >= 30 && m < 60) ? 4
                : h === 10 || h === 12 ? 2 : 1;
            for (let w = 0; w < weight; w++) sessionMinutes.push(h * 60 + m);
        }
    }
    for (let i = 0; i < 200; i++) {
        const slot = sessionMinutes[Math.floor(rand() * sessionMinutes.length)];
        const h = Math.floor(slot / 60);
        const m = slot % 60;
        // Edge by window:
        let pnlMean;
        if (h === 9 && m >= 30) pnlMean =  85;   // 09:30 momo window: edge
        else if (h === 11 && m >= 30) pnlMean = -110;  // 11:30 chop: anti-edge
        else if (h === 15 && m >= 30) pnlMean =  45;   // closing run
        else                       pnlMean =   5;
        const pnl = Number((pnlMean + (rand() - 0.5) * 200).toFixed(2));
        out.push({ when: `${EPOCH_DATE}T${pad2(h)}:${pad2(m)}:00Z`, pnl });
    }
    return out;
}

export function fmtUSD(v) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-$' : '$';
    return sign + Math.abs(v).toFixed(2);
}

export function fmtPct(v) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(0) + '%';
}
