// Almgren-Chriss optimal-execution helpers shared by view + vitest.
//
// Backend body shape: { params: { total_shares, horizon_seconds,
// n_intervals, eta, gamma, lambda, sigma } }.
//
// Frontier helpers compute the (variance, expected-cost) curve over a
// λ sweep — the classical AC efficient frontier.

export function validateParams(p) {
    if (!Number.isFinite(p.total_shares) || p.total_shares === 0)
        return 'total_shares must be a non-zero finite number';
    if (!Number.isFinite(p.horizon_seconds) || p.horizon_seconds <= 0)
        return 'horizon_seconds must be > 0';
    if (!Number.isInteger(p.n_intervals) || p.n_intervals < 1 || p.n_intervals > 2000)
        return 'n_intervals must be integer in [1, 2000]';
    if (!Number.isFinite(p.eta) || p.eta <= 0)
        return 'eta (temporary impact) must be > 0';
    if (!Number.isFinite(p.gamma) || p.gamma < 0)
        return 'gamma (permanent impact) must be ≥ 0';
    if (!Number.isFinite(p.lambda) || p.lambda < 0)
        return 'lambda (risk aversion) must be ≥ 0';
    if (!Number.isFinite(p.sigma) || p.sigma < 0)
        return 'sigma (vol) must be ≥ 0';
    return null;
}

export function buildBody(p) {
    return {
        params: {
            total_shares:    p.total_shares,
            horizon_seconds: p.horizon_seconds,
            n_intervals:     p.n_intervals,
            eta:             p.eta,
            gamma:           p.gamma,
            lambda:          p.lambda,
            sigma:           p.sigma,
        },
    };
}

// Time grid for the trajectory chart. Inventory has n+1 points (t=0 to
// t=T); schedule has n points (one per slice).
export function timeAxis(horizonSeconds, nIntervals, kind = 'inventory') {
    if (!(nIntervals >= 1) || !(horizonSeconds > 0)) return [];
    const tau = horizonSeconds / nIntervals;
    const len = kind === 'schedule' ? nIntervals : nIntervals + 1;
    const xs = new Array(len);
    if (kind === 'schedule') {
        // Place each slice at the midpoint of its window.
        for (let k = 0; k < len; k++) xs[k] = (k + 0.5) * tau;
    } else {
        for (let k = 0; k < len; k++) xs[k] = k * tau;
    }
    return xs;
}

// Sweeps a geometric ladder of λ values and returns the (variance,
// expected-cost) pairs for the efficient frontier chart. Each point is
// produced by an independent backend call — caller controls concurrency.
export function lambdaSweep(baseLambda, points = 7) {
    if (!Number.isFinite(baseLambda) || baseLambda <= 0) baseLambda = 1e-6;
    if (!Number.isInteger(points) || points < 3) points = 3;
    if (points > 21) points = 21;
    const out = new Array(points);
    const half = (points - 1) / 2;
    for (let i = 0; i < points; i++) {
        const exp = i - half;
        out[i] = baseLambda * Math.pow(10, exp);
    }
    return out;
}

// Picks the index of `lambdas` whose value is closest to the user's
// chosen λ — used to highlight "you are here" on the frontier scatter.
export function nearestLambdaIndex(lambdas, target) {
    if (!Array.isArray(lambdas) || lambdas.length === 0) return -1;
    if (!Number.isFinite(target)) return -1;
    let bestI = 0, bestD = Math.abs(Math.log(lambdas[0]) - Math.log(target));
    for (let i = 1; i < lambdas.length; i++) {
        const d = Math.abs(Math.log(lambdas[i]) - Math.log(target));
        if (d < bestD) { bestI = i; bestD = d; }
    }
    return bestI;
}

// Compact human-readable formatter for cost / variance / shares.
export function fmtBig(v) {
    if (!Number.isFinite(v)) return '—';
    const a = Math.abs(v);
    if (a >= 1e9)  return (v / 1e9).toFixed(3) + 'B';
    if (a >= 1e6)  return (v / 1e6).toFixed(3) + 'M';
    if (a >= 1e3)  return (v / 1e3).toFixed(3) + 'k';
    return v.toFixed(3);
}

export function fmtSeconds(v) {
    if (!Number.isFinite(v) || v < 0) return '—';
    if (v < 60)    return `${v.toFixed(1)}s`;
    if (v < 3600)  return `${(v / 60).toFixed(2)}m`;
    if (v < 86400) return `${(v / 3600).toFixed(2)}h`;
    return `${(v / 86400).toFixed(2)}d`;
}

export function fmtPct(v) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(2) + '%';
}
