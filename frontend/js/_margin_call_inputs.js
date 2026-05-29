// Margin-call distance calculator helpers.
//
// Backend body: AccountSnapshot { long_market_value, margin_debt,
//   maintenance_pct } — all Decimal-as-string on the wire.
// Returns: MarginCallReport { current_equity, current_equity_pct,
//   dollar_cushion, pct_cushion, in_call }.
//
// Trigger LMV = debt / (1 - maint_pct). Cushion = LMV - trigger_LMV.
// Boundary: cushion == 0 → NOT in call (Rust uses strict <).
// 100% maintenance: any debt → in_call.

export const DEFAULT_INPUTS = {
    long_market_value: 100_000,
    margin_debt: 60_000,
    maintenance_pct: 0.25,
};

export function validateInputs(snap) {
    if (!Number.isFinite(snap.long_market_value)) return 'long_market_value must be finite';
    if (snap.long_market_value < 0)               return 'long_market_value must be ≥ 0';
    if (!Number.isFinite(snap.margin_debt))       return 'margin_debt must be finite';
    if (snap.margin_debt < 0)                     return 'margin_debt must be ≥ 0';
    if (!Number.isFinite(snap.maintenance_pct))   return 'maintenance_pct must be finite';
    if (snap.maintenance_pct < 0 || snap.maintenance_pct > 1)
        return 'maintenance_pct must be in [0, 1] (decimal — 0.25 = 25%)';
    return null;
}

export function buildBody(snap) {
    return {
        long_market_value: String(snap.long_market_value),
        margin_debt:       String(snap.margin_debt),
        maintenance_pct:   String(snap.maintenance_pct),
    };
}

export function dec(v) {
    if (v == null) return 0;
    if (typeof v === 'number') return v;
    const n = Number(v);
    return Number.isFinite(n) ? n : 0;
}

// Pure-JS mirror of crates/traderview-core/src/margin_call.rs::evaluate.
export function localEvaluate(snap) {
    const out = {
        current_equity: snap.long_market_value - snap.margin_debt,
        current_equity_pct: 0,
        dollar_cushion: 0,
        pct_cushion: 0,
        in_call: false,
    };
    if (snap.long_market_value === 0) return out;
    const oneMinusMaint = 1 - snap.maintenance_pct;
    if (oneMinusMaint === 0) {
        // 100% maintenance.
        out.dollar_cushion = -snap.margin_debt;
        out.in_call = snap.margin_debt > 0;
        return out;
    }
    const triggerLmv = snap.margin_debt / oneMinusMaint;
    out.dollar_cushion = snap.long_market_value - triggerLmv;
    out.pct_cushion = snap.long_market_value > 0
        ? out.dollar_cushion / snap.long_market_value : 0;
    out.current_equity_pct = out.current_equity / snap.long_market_value;
    out.in_call = out.dollar_cushion < 0;
    return out;
}

// Trigger price for a single-position account (informative addition for the UI).
export function triggerLmv(snap) {
    if (!Number.isFinite(snap.margin_debt) || snap.maintenance_pct >= 1) return Infinity;
    return snap.margin_debt / (1 - snap.maintenance_pct);
}

// Cushion-level traffic light. Drives the headline badge.
export function cushionBadge(report) {
    if (!report) return { key: 'view.margin_call.badge.unknown', cls: '' };
    if (report.in_call) return { key: 'view.margin_call.badge.in_call', cls: 'neg' };
    if (!Number.isFinite(report.pct_cushion)) return { key: 'view.margin_call.badge.unknown', cls: '' };
    if (report.pct_cushion < 0.05) return { key: 'view.margin_call.badge.critical', cls: 'neg' };
    if (report.pct_cushion < 0.15) return { key: 'view.margin_call.badge.tight',    cls: 'neg' };
    if (report.pct_cushion < 0.30) return { key: 'view.margin_call.badge.moderate', cls: '' };
    return { key: 'view.margin_call.badge.safe', cls: 'pos' };
}

// Demo presets — one per Rust test branch + a few real-world setups.
export function makeDemoInput(kind = 'standard') {
    switch (kind) {
        case 'fully-cash':
            return { long_market_value: 50_000, margin_debt: 0, maintenance_pct: 0.25 };
        case 'standard':
            // $100k LMV, $60k debt, 25% maint → cushion $20k.
            return { long_market_value: 100_000, margin_debt: 60_000, maintenance_pct: 0.25 };
        case 'in-call':
            // $100k LMV, $80k debt → 20% equity < 25% maint → in call.
            return { long_market_value: 100_000, margin_debt: 80_000, maintenance_pct: 0.25 };
        case 'at-line':
            // Exactly at maintenance (cushion = 0).
            return { long_market_value: 100_000, margin_debt: 75_000, maintenance_pct: 0.25 };
        case 'high-maint':
            // Small-cap 40% maint with same numbers → cushion = 0.
            return { long_market_value: 100_000, margin_debt: 60_000, maintenance_pct: 0.40 };
        case 'cash-only-with-debt':
            // 100% maint → any debt = in call.
            return { long_market_value: 50_000, margin_debt: 1, maintenance_pct: 1.0 };
        case 'no-positions':
            return { long_market_value: 0, margin_debt: 0, maintenance_pct: 0.25 };
        case 'leveraged-bull':
            // $500k LMV, $300k debt, 25% maint → cushion $100k = 20%.
            return { long_market_value: 500_000, margin_debt: 300_000, maintenance_pct: 0.25 };
        default:
            return makeDemoInput('standard');
    }
}

export function fmtUSD(v, d = 0) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-' : '';
    return sign + '$' + Math.abs(v).toFixed(d);
}

export function fmtUSDSigned(v, d = 0) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '-') + '$' + Math.abs(v).toFixed(d);
}

export function fmtPct(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtMaintPct(v) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(0) + '%';
}
