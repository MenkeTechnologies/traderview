// Pyramid / Scale-In plan calculator helpers shared by view + vitest.
//
// Backend body shape (flat — no wrapper): PlanInput { kind: "pyramid_up"
// | "scale_in", side: "long" | "short", initial_qty: Decimal-string,
// initial_entry: Decimal-string, tranches: [{trigger_price, qty}, ...] }.
//
// Backend response uses Decimal-string scalars in every numeric field.

import { t } from './i18n.js';

const TOKEN_DELIM = /[\s,]+/;

// Two-token-per-line "trigger_price qty" for tranches.
export function parseTrancheBlob(text) {
    const tranches = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { tranches, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const s = raw.trim();
        if (!s || s.startsWith('#')) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 2) {
            errors.push({ line_no: i + 1, raw, message: `expected 2 tokens (trigger_price qty), got ${parts.length}` });
            continue;
        }
        const trigger = Number(parts[0]);
        const qty = Number(parts[1]);
        if (!Number.isFinite(trigger) || trigger <= 0) {
            errors.push({ line_no: i + 1, raw, message: `trigger_price must be > 0` });
            continue;
        }
        if (!Number.isFinite(qty) || qty <= 0) {
            errors.push({ line_no: i + 1, raw, message: `qty must be > 0` });
            continue;
        }
        tranches.push({ trigger_price: trigger, qty });
    }
    return { tranches, errors };
}

export function validateInputs(p) {
    if (p.kind !== 'pyramid_up' && p.kind !== 'scale_in') return t('view.pyramid.validate.kind');
    if (p.side !== 'long' && p.side !== 'short') return t('view.pyramid.validate.side');
    if (!Number.isFinite(p.initial_qty) || p.initial_qty <= 0) return t('view.pyramid.validate.initial_qty');
    if (!Number.isFinite(p.initial_entry) || p.initial_entry <= 0) return t('view.pyramid.validate.initial_entry');
    if (!Array.isArray(p.tranches) || p.tranches.length === 0) return t('view.pyramid.validate.need_tranche');
    if (!p.tranches.every(tr => Number.isFinite(tr.trigger_price) && tr.trigger_price > 0))
        return t('view.pyramid.validate.trigger_price');
    if (!p.tranches.every(tr => Number.isFinite(tr.qty) && tr.qty > 0))
        return t('view.pyramid.validate.tranche_qty');
    return null;
}

export function buildBody(p) {
    return {
        kind: p.kind,
        side: p.side,
        initial_qty:   String(p.initial_qty),
        initial_entry: String(p.initial_entry),
        tranches: p.tranches.map(t => ({
            trigger_price: String(t.trigger_price),
            qty:           String(t.qty),
        })),
    };
}

// Local pre-flight that mirrors the backend's direction check. Lets the
// view warn the user before round-tripping when a tranche obviously
// violates the kind's direction (e.g., PyramidUp Long with a tranche
// BELOW initial entry).
export function directionMisordered(kind, side, initialEntry, tranches) {
    if (!Array.isArray(tranches)) return false;
    for (const t of tranches) {
        const ok =
            (kind === 'pyramid_up' && side === 'long'  && t.trigger_price > initialEntry) ||
            (kind === 'pyramid_up' && side === 'short' && t.trigger_price < initialEntry) ||
            (kind === 'scale_in'   && side === 'long'  && t.trigger_price < initialEntry) ||
            (kind === 'scale_in'   && side === 'short' && t.trigger_price > initialEntry);
        if (!ok) return true;
    }
    return false;
}

// Coerces backend Decimal-string scalars to plain numbers for chart math.
export function decToNum(v) {
    if (v == null) return NaN;
    if (typeof v === 'number') return v;
    const n = Number(v);
    return Number.isFinite(n) ? n : NaN;
}

// Demo presets — one per (kind, side) combination. Each chosen so the
// direction-check passes (no misorder) and the avg-cost evolution chart
// shows the canonical curve for that strategy.
export function makeDemoData(kind = 'pyramid_up', side = 'long') {
    const initial_entry = 100;
    const initial_qty = 100;
    if (kind === 'pyramid_up' && side === 'long') {
        return { kind, side, initial_qty, initial_entry, tranches: [
            { trigger_price: 105, qty: 75 },
            { trigger_price: 110, qty: 50 },
            { trigger_price: 115, qty: 25 },
        ] };
    }
    if (kind === 'pyramid_up' && side === 'short') {
        return { kind, side, initial_qty, initial_entry, tranches: [
            { trigger_price: 95,  qty: 75 },
            { trigger_price: 90,  qty: 50 },
            { trigger_price: 85,  qty: 25 },
        ] };
    }
    if (kind === 'scale_in' && side === 'long') {
        return { kind, side, initial_qty, initial_entry, tranches: [
            { trigger_price: 95,  qty: 100 },
            { trigger_price: 90,  qty: 150 },
            { trigger_price: 85,  qty: 200 },
        ] };
    }
    return { kind, side, initial_qty, initial_entry, tranches: [
        { trigger_price: 105, qty: 100 },
        { trigger_price: 110, qty: 150 },
        { trigger_price: 115, qty: 200 },
    ] };
}

// Pulls per-state avg_cost evolution for the chart (decimal-string-safe).
export function avgCostSeries(report) {
    const xs = [], ys = [];
    if (!report || !Array.isArray(report.states)) return { xs, ys };
    for (let i = 0; i < report.states.length; i++) {
        xs.push(i);
        ys.push(decToNum(report.states[i].avg_cost));
    }
    return { xs, ys };
}

export function fmtN(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtInt(v) {
    if (!Number.isFinite(v)) return '—';
    return Math.round(v).toLocaleString('en-US');
}

export function fmtUSD(v) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-$' : '$';
    return sign + Math.abs(v).toFixed(2);
}
