// Margin-call runway helpers shared by view + vitest.
//
// Backend body: { account_equity, position_value, maintenance_req_pct }.
// Returns: MarginRunwayReport with {account_equity, position_value,
//   maintenance_req_pct, runway_pct, already_in_margin_call, equity_buffer_dollars}.
//
// Formula (Reg-T style single-leg):
//   maint_dollars = position × maint_pct
//   equity_buffer = equity - maint_dollars
//   if buffer < 0 → already in call, runway = 0
//   else runway% = buffer / (position × (1 - maint_pct))
//
// Pure compute; matches crates/traderview-core/src/margin_runway.rs::compute.

export const DEFAULT_INPUTS = {
    account_equity: 50_000,
    position_value: 100_000,
    maintenance_req_pct: 0.25,
};

export function validateInputs(equity, position, maintPct) {
    if (!Number.isFinite(equity))            return 'account_equity must be finite';
    if (!Number.isFinite(position))          return 'position_value must be finite';
    if (position < 0)                        return 'position_value must be ≥ 0';
    if (!Number.isFinite(maintPct))          return 'maintenance_req_pct must be finite';
    if (maintPct < 0 || maintPct >= 1)
        return 'maintenance_req_pct must be in [0, 1) — e.g. 0.25 for 25%';
    return null;
}

export function buildBody(equity, position, maintPct) {
    return {
        account_equity: equity,
        position_value: position,
        maintenance_req_pct: maintPct,
    };
}

// Local Rust mirror.
export function localCompute(equity, position, maintPct) {
    const out = {
        account_equity: equity,
        position_value: position,
        maintenance_req_pct: maintPct,
        runway_pct: 0,
        already_in_margin_call: false,
        equity_buffer_dollars: 0,
    };
    if (position <= 0 || maintPct >= 1) return out;
    const maintReq = position * maintPct;
    const buffer = equity - maintReq;
    out.equity_buffer_dollars = buffer;
    out.already_in_margin_call = buffer < 0;
    if (out.already_in_margin_call) {
        out.runway_pct = 0;
    } else {
        out.runway_pct = buffer / (position * (1 - maintPct));
    }
    return out;
}

// Risk traffic-light. Driven by runway_pct + already_in_margin_call.
export function runwayBadge(report) {
    if (!report || !Number.isFinite(report.runway_pct)) {
        return { key: 'view.margin_runway.badge.unknown', cls: '' };
    }
    if (report.already_in_margin_call) return { key: 'view.margin_runway.badge.in_call',  cls: 'neg' };
    if (report.runway_pct < 0.05)       return { key: 'view.margin_runway.badge.critical', cls: 'neg' };
    if (report.runway_pct < 0.15)       return { key: 'view.margin_runway.badge.tight',    cls: 'neg' };
    if (report.runway_pct < 0.30)       return { key: 'view.margin_runway.badge.moderate', cls: '' };
    return { key: 'view.margin_runway.badge.safe', cls: 'pos' };
}

// Build a projection of equity vs maintenance requirement across a
// range of hypothetical price declines. Returns parallel arrays for
// the uPlot chart.
export function projectionCurves(equity, position, maintPct, steps = 41, maxDeclinePct = 0.5) {
    const xs = [], equityCurve = [], maintCurve = [], bufferCurve = [];
    if (!Number.isFinite(equity) || !Number.isFinite(position) || position <= 0) {
        return { xs, equityCurve, maintCurve, bufferCurve };
    }
    for (let i = 0; i <= steps; i++) {
        const d = (i / steps) * maxDeclinePct;
        xs.push(d);
        const newPos = position * (1 - d);
        const newEquity = equity - position * d;
        const newMaint = newPos * maintPct;
        equityCurve.push(newEquity);
        maintCurve.push(newMaint);
        bufferCurve.push(newEquity - newMaint);
    }
    return { xs, equityCurve, maintCurve, bufferCurve };
}

// Demo presets — exercise every badge bucket + the already-in-call branch.
export function makeDemoInputs(kind = 'safe') {
    switch (kind) {
        case 'cash-only':
            // $50k cash, no leveraged position → no margin risk.
            return { account_equity: 50_000, position_value: 0,      maintenance_req_pct: 0.25 };
        case 'safe':
            // $100k equity, $100k position → runway 100%.
            return { account_equity: 100_000, position_value: 100_000, maintenance_req_pct: 0.25 };
        case 'moderate':
            // $50k equity, $100k position → runway 33%.
            return { account_equity: 50_000,  position_value: 100_000, maintenance_req_pct: 0.25 };
        case 'tight':
            // $30k equity, $100k position → runway ~6.7%.
            return { account_equity: 30_000,  position_value: 100_000, maintenance_req_pct: 0.25 };
        case 'critical':
            // $26k equity, $100k position → runway ~1.3%.
            return { account_equity: 26_000,  position_value: 100_000, maintenance_req_pct: 0.25 };
        case 'in-call':
            // $20k equity, $100k position → already in call.
            return { account_equity: 20_000,  position_value: 100_000, maintenance_req_pct: 0.25 };
        case 'pdt-leveraged':
            // Pattern day trader with 4× buying power. $25k equity, $100k position, 25% maint.
            return { account_equity: 25_000,  position_value: 100_000, maintenance_req_pct: 0.25 };
        case 'concentrated':
            // Concentrated position w/ broker bumping maint to 50%.
            return { account_equity: 50_000,  position_value: 100_000, maintenance_req_pct: 0.50 };
        default:
            return makeDemoInputs('safe');
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
    // e.g. 0.25 → "25%" without decimals.
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(0) + '%';
}
