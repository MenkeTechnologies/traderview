// ATR-cone projection helpers.
//
// Backend body: { entry, daily_atr, horizon_days }.
// Returns: ConePoint[] with { days_forward, upper_2sd, upper_1sd, center, lower_1sd, lower_2sd }.
//
// Math: σ_N = daily_atr × √N (Brownian scaling). Bands symmetric around entry.
// Horizon capped at 1000 days server-side (MAX_HORIZON_DAYS) — local mirror
// applies the same cap so chart x-range matches.

export const MAX_HORIZON_DAYS = 1000;

export const DEFAULT_INPUTS = {
    entry: 100,
    daily_atr: 2,
    horizon_days: 20,
};

export function validateInputs(input) {
    if (!Number.isFinite(input.entry))     return 'entry must be finite';
    if (input.entry <= 0)                  return 'entry must be > 0';
    if (!Number.isFinite(input.daily_atr)) return 'daily_atr must be finite';
    if (input.daily_atr < 0)               return 'daily_atr must be ≥ 0';
    if (!Number.isInteger(input.horizon_days) || input.horizon_days < 0)
        return 'horizon_days must be non-negative integer';
    return null;
}

export function buildBody(input) {
    return {
        entry: input.entry,
        daily_atr: input.daily_atr,
        horizon_days: input.horizon_days,
    };
}

// Pure-JS mirror of crates/traderview-core/src/atr_cone.rs::project.
// Returns horizon+1 points (inclusive of day 0). Caps at MAX_HORIZON_DAYS.
export function localProject(entry, daily_atr, horizon_days) {
    const horizon = Math.min(horizon_days, MAX_HORIZON_DAYS);
    const out = new Array(horizon + 1);
    for (let d = 0; d <= horizon; d++) {
        const sigma = daily_atr * Math.sqrt(d);
        out[d] = {
            days_forward: d,
            upper_2sd: entry + 2 * sigma,
            upper_1sd: entry + sigma,
            center: entry,
            lower_1sd: entry - sigma,
            lower_2sd: entry - 2 * sigma,
        };
    }
    return out;
}

// Width-at-horizon: 2 × sigma_N. Useful for the summary card.
export function widthAtHorizon(daily_atr, horizon_days) {
    if (!Number.isFinite(daily_atr) || !Number.isInteger(horizon_days) || horizon_days < 0) return NaN;
    return 2 * daily_atr * Math.sqrt(Math.min(horizon_days, MAX_HORIZON_DAYS));
}

// Width as a % of entry — comparable across symbols.
export function widthPctAtHorizon(entry, daily_atr, horizon_days) {
    if (!Number.isFinite(entry) || entry <= 0) return NaN;
    const w = widthAtHorizon(daily_atr, horizon_days);
    return Number.isFinite(w) ? w / entry : NaN;
}

// "Risk-noise" tier — how loud the band is relative to entry, helps the
// trader judge if their stop/target placement makes sense vs the cone.
export function noiseBadge(entry, daily_atr, horizon_days) {
    const pct = widthPctAtHorizon(entry, daily_atr, horizon_days);
    if (!Number.isFinite(pct)) return { key: 'view.atr_cone.badge.unknown', cls: '' };
    if (pct === 0)             return { key: 'view.atr_cone.badge.flat',    cls: '' };
    if (pct < 0.02)            return { key: 'view.atr_cone.badge.quiet',   cls: 'pos' };
    if (pct < 0.05)            return { key: 'view.atr_cone.badge.normal',  cls: '' };
    if (pct < 0.10)            return { key: 'view.atr_cone.badge.loud',    cls: 'neg' };
    return { key: 'view.atr_cone.badge.extreme', cls: 'neg' };
}

// Number of days needed for the ±1σ band to reach a target offset (in $).
// Inverts σ_N = ATR × √N → N = (offset / ATR)^2.
export function daysToReachOffset(daily_atr, offset_dollars) {
    if (!Number.isFinite(daily_atr) || daily_atr <= 0) return Infinity;
    if (!Number.isFinite(offset_dollars) || offset_dollars <= 0) return 0;
    return Math.pow(offset_dollars / daily_atr, 2);
}

// Demo presets across asset-class volatility regimes.
export function makeDemoInput(kind = 'spy-normal') {
    switch (kind) {
        case 'spy-normal':
            // SPY ~$500, ATR ~$5, 5-day cone.
            return { entry: 500, daily_atr: 5, horizon_days: 5 };
        case 'aapl-medium':
            return { entry: 180, daily_atr: 3.5, horizon_days: 10 };
        case 'tsla-loud':
            // Tesla — 4% daily ATR.
            return { entry: 250, daily_atr: 10, horizon_days: 20 };
        case 'penny-extreme':
            // $5 stock, $1 ATR = 20% daily move.
            return { entry: 5, daily_atr: 1, horizon_days: 5 };
        case 'long-horizon':
            // 60-day swing — cone widens to ±2σ × √60.
            return { entry: 100, daily_atr: 2, horizon_days: 60 };
        case 'zero-atr':
            // Edge case — flat cone.
            return { entry: 100, daily_atr: 0, horizon_days: 10 };
        case 'huge-horizon':
            // Over the MAX_HORIZON_DAYS cap.
            return { entry: 100, daily_atr: 1, horizon_days: 2000 };
        case 'es-futures':
            // ES @ 5000, ATR ~50, weekly cone.
            return { entry: 5000, daily_atr: 50, horizon_days: 5 };
        default:
            return makeDemoInput('spy-normal');
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

export function fmtPct(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtDays(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d) + ' d';
}
