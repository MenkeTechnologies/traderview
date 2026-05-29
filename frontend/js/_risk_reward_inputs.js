// Risk-Reward helpers shared by view + vitest.
//
// Backend body: RrInput flat: { side, entry, stop, target, risk_budget,
//   multiplier } — Decimals as strings on the wire.
// Returns: RrReport { r_multiple, qty, dollar_risk, dollar_reward,
//   breakeven_win_rate, scale_outs: [{label, price, fraction}] }.
//
// Geometry errors:
//   - stop == entry → "stop equals entry"
//   - long with target ≤ entry → "long requires target > entry > stop"
//   - long with stop ≥ entry → same
//   - short with target ≥ entry → "short requires target < entry < stop"
//   - short with stop ≤ entry → same
// All emitted via Err<&str> on the backend; we mirror the exact strings.

import { t } from './i18n.js';

export const DEFAULT_INPUTS = {
    side: 'long',
    entry: 100,
    stop: 99,
    target: 103,
    risk_budget: 100,
    multiplier: 1,
};

// Shape-only validation (geometry is checked by compute / localCompute).
export function validateInputs(input) {
    if (input.side !== 'long' && input.side !== 'short')
        return 'side must be "long" or "short"';
    for (const k of ['entry', 'stop', 'target', 'risk_budget', 'multiplier']) {
        if (!Number.isFinite(input[k])) return `${k} must be finite`;
    }
    if (input.entry <= 0)       return 'entry must be > 0';
    if (input.stop <= 0)        return 'stop must be > 0';
    if (input.target <= 0)      return 'target must be > 0';
    if (input.risk_budget <= 0) return 'risk_budget must be > 0';
    if (input.multiplier <= 0)  return 'multiplier must be > 0';
    return null;
}

// Decimal fields go on the wire as strings per rust_decimal contract.
export function buildBody(input) {
    return {
        side: input.side,
        entry:        String(input.entry),
        stop:         String(input.stop),
        target:       String(input.target),
        risk_budget:  String(input.risk_budget),
        multiplier:   String(input.multiplier),
    };
}

// Coerce a Decimal-string-or-number field back to Number for chart math.
export function dec(v) {
    if (v == null) return 0;
    if (typeof v === 'number') return v;
    const n = Number(v);
    return Number.isFinite(n) ? n : 0;
}

// Mirror of crates/traderview-core/src/risk_reward.rs::compute. Returns
// { ok: true, report } OR { ok: false, error } with the EXACT same
// error strings the backend emits.
export function localCompute(input) {
    const stopDist = Math.abs(input.entry - input.stop);
    const targetDist = Math.abs(input.target - input.entry);
    if (stopDist === 0) return { ok: false, error: 'stop equals entry — risk is zero, cannot size' };
    const rMul = targetDist / stopDist;
    if (input.side === 'long' && (input.target <= input.entry || input.stop >= input.entry))
        return { ok: false, error: 'long requires target > entry > stop' };
    if (input.side === 'short' && (input.target >= input.entry || input.stop <= input.entry))
        return { ok: false, error: 'short requires target < entry < stop' };
    const perUnitRisk = stopDist * input.multiplier;
    if (perUnitRisk === 0) return { ok: false, error: 'multiplier × stop distance is zero' };
    const qty = input.risk_budget / perUnitRisk;
    const dollarRisk = qty * perUnitRisk;
    const dollarReward = qty * targetDist * input.multiplier;
    const breakevenWr = 1 / (1 + rMul);
    const oneR = input.side === 'long' ? input.entry + stopDist : input.entry - stopDist;
    const twoR = input.side === 'long' ? input.entry + 2 * stopDist : input.entry - 2 * stopDist;
    return {
        ok: true,
        report: {
            r_multiple: rMul,
            qty,
            dollar_risk: dollarRisk,
            dollar_reward: dollarReward,
            breakeven_win_rate: breakevenWr,
            scale_outs: [
                { label: '1R',                              price: oneR,         fraction: 1 / 3 },
                { label: '2R',                              price: twoR,         fraction: 1 / 3 },
                { label: t('view.risk_reward.scale_out.target'), price: input.target, fraction: 1 / 3 },
            ],
        },
    };
}

// R-multiple verdict.
const RR_BADGES = {
    none:       { key: 'view.risk_reward.badge.none',       cls: 'neg' },
    poor:       { key: 'view.risk_reward.badge.poor',       cls: 'neg' },
    fair:       { key: 'view.risk_reward.badge.fair',       cls: '' },
    good:       { key: 'view.risk_reward.badge.good',       cls: 'pos' },
    excellent:  { key: 'view.risk_reward.badge.excellent',  cls: 'pos' },
};

export function rrBadge(rMultiple) {
    if (!Number.isFinite(rMultiple) || rMultiple <= 0) return RR_BADGES.none;
    if (rMultiple < 1)   return RR_BADGES.poor;
    if (rMultiple < 2)   return RR_BADGES.fair;
    if (rMultiple < 3)   return RR_BADGES.good;
    return RR_BADGES.excellent;
}

// Demo presets — exercise every Rust geometry case + common trader setups.
export function makeDemoInput(kind = 'long-3r') {
    switch (kind) {
        case 'long-3r':
            // Classic 3:1 long. $100 → 100 sh stock at $100 with $1 stop.
            return { side: 'long', entry: 100, stop: 99,  target: 103, risk_budget: 100, multiplier: 1 };
        case 'long-1r':
            // 1:1 — coin flip, 50% breakeven required.
            return { side: 'long', entry: 100, stop: 99,  target: 101, risk_budget: 100, multiplier: 1 };
        case 'long-5r':
            return { side: 'long', entry: 100, stop: 99,  target: 105, risk_budget: 100, multiplier: 1 };
        case 'short-3r':
            return { side: 'short', entry: 100, stop: 101, target: 97, risk_budget: 100, multiplier: 1 };
        case 'options-1ct':
            // 1 option contract × 100 mult × $1 stop = $100 per-contract risk.
            return { side: 'long', entry: 5,   stop: 4,   target: 8,   risk_budget: 100, multiplier: 100 };
        case 'es-futures':
            // ES @ 4500, $10 stop, $30 target, $50 multiplier, $500 budget.
            // Per-unit risk = $10 × 50 = $500 → 1 contract.
            return { side: 'long', entry: 4500, stop: 4490, target: 4530, risk_budget: 500, multiplier: 50 };
        case 'broken-long':
            // Geometry error: long with target below entry.
            return { side: 'long', entry: 100, stop: 98, target: 95, risk_budget: 100, multiplier: 1 };
        case 'broken-short':
            return { side: 'short', entry: 100, stop: 101, target: 103, risk_budget: 100, multiplier: 1 };
        case 'zero-stop':
            return { side: 'long', entry: 100, stop: 100, target: 103, risk_budget: 100, multiplier: 1 };
        default:
            return makeDemoInput('long-3r');
    }
}

export function fmtUSD(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-' : '';
    return sign + '$' + Math.abs(v).toFixed(d);
}

export function fmtUSDSigned(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '-') + '$' + Math.abs(v).toFixed(d);
}

export function fmtNum(v, d = 4) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtPct(v, d = 1) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtR(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d) + 'R';
}

export function fmtFraction(v) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(1) + '%';
}
