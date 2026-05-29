// Pure helpers for the Greeks Profile view.
//
// Backend returns an array of GreeksPoint { spot, price, delta, gamma,
// vega, theta, rho } across a spot grid. We split that into parallel
// per-metric series so each mini-chart can render independently.

const METRICS = ['price', 'delta', 'gamma', 'vega', 'theta', 'rho'];

/** Build backend body. */
export function buildBody(p) {
    return {
        kind:           p.kind,
        strike:         p.strike,
        time_to_expiry: p.time_to_expiry,
        risk_free:      p.risk_free,
        dividend_yield: p.dividend_yield,
        sigma:          p.sigma,
        spot_grid_low:  p.spot_grid_low,
        spot_grid_high: p.spot_grid_high,
        n_points:       p.n_points,
    };
}

/** Validate inputs. */
export function validateParams(p) {
    if (p.kind !== 'call' && p.kind !== 'put') return 'kind must be "call" or "put"';
    if (!Number.isFinite(p.strike) || p.strike <= 0)         return 'strike must be > 0';
    if (!Number.isFinite(p.time_to_expiry) || p.time_to_expiry <= 0) return 'time_to_expiry must be > 0';
    if (!Number.isFinite(p.risk_free))                       return 'risk_free must be finite';
    if (!Number.isFinite(p.dividend_yield) || p.dividend_yield < 0)  return 'dividend_yield must be ≥ 0';
    if (!Number.isFinite(p.sigma) || p.sigma <= 0)           return 'sigma must be > 0';
    if (!Number.isFinite(p.spot_grid_low) || p.spot_grid_low <= 0)   return 'spot_grid_low must be > 0';
    if (!Number.isFinite(p.spot_grid_high) || p.spot_grid_high <= 0) return 'spot_grid_high must be > 0';
    if (p.spot_grid_high <= p.spot_grid_low) {
        return 'spot_grid_high must be > spot_grid_low';
    }
    if (!Number.isInteger(p.n_points) || p.n_points < 5 || p.n_points > 501) {
        return 'n_points must be an integer in [5, 501]';
    }
    return null;
}

/** Split a GreeksProfileReport's `points` array into the per-metric
 *  series used by the mini-charts. Returns:
 *    { spots: number[], price: number[], delta: number[], ... }
 *  Each series is null-safe (non-finite values mapped to null). */
export function splitMetricSeries(points) {
    const out = { spots: [] };
    for (const m of METRICS) out[m] = [];
    if (!Array.isArray(points)) return out;
    for (const pt of points) {
        if (!pt || typeof pt !== 'object') continue;
        out.spots.push(Number.isFinite(pt.spot) ? pt.spot : null);
        for (const m of METRICS) {
            const v = pt[m];
            out[m].push(Number.isFinite(v) ? v : null);
        }
    }
    return out;
}

export { METRICS };

/** Format a small floating-point value. */
export function fmtN(x, digits = 4) {
    if (!Number.isFinite(x)) return '—';
    return x.toFixed(digits);
}

/** Suggest a default spot-grid range from strike: ±50% by default. */
export function defaultSpotGrid(strike) {
    if (!Number.isFinite(strike) || strike <= 0) {
        return { low: 50, high: 150 };
    }
    return { low: strike * 0.5, high: strike * 1.5 };
}
