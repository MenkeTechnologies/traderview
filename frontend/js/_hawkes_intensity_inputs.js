// Hawkes self-exciting point-process intensity helpers.
//
// Backend body: { event_times: number[], query_times: number[],
//                 params: { baseline_mu, excitation_alpha, decay_beta } }
// Returns: { intensities: number[], unconditional_mean_intensity,
//   is_stable } | null  (null if any validation fails)
//
// Model: λ(t) = μ + Σ_{t_i < t} α · exp(−β · (t − t_i))
// Stable iff α < β; unconditional mean = μ / (1 − α/β).

import { t as tr } from './i18n.js';

export const DEFAULT_PARAMS = {
    baseline_mu: 0.5,
    excitation_alpha: 0.4,
    decay_beta: 1.0,
};

export const DEFAULT_INPUTS = {
    event_times: [1, 2, 3, 4, 5],
    query_times: [0.5, 1.5, 2.5, 3.5, 4.5, 5.5, 6.5],
    params: { ...DEFAULT_PARAMS },
};

export function validateInputs(input) {
    if (!Array.isArray(input.event_times))                       return tr('view.hawkes.validate.events_array');
    if (!Array.isArray(input.query_times))                       return tr('view.hawkes.validate.queries_array');
    if (input.event_times.some(t => !Number.isFinite(t)))        return tr('view.hawkes.validate.events_finite');
    if (input.query_times.some(t => !Number.isFinite(t)))        return tr('view.hawkes.validate.queries_finite');
    for (let i = 1; i < input.event_times.length; i++) {
        if (input.event_times[i] < input.event_times[i - 1]) return tr('view.hawkes.validate.events_sorted');
    }
    const p = input.params;
    if (!p)                                                       return tr('view.hawkes.validate.params_required');
    if (!Number.isFinite(p.baseline_mu) || p.baseline_mu < 0)     return tr('view.hawkes.validate.mu');
    if (!Number.isFinite(p.excitation_alpha) || p.excitation_alpha < 0) return tr('view.hawkes.validate.alpha');
    if (!Number.isFinite(p.decay_beta) || p.decay_beta <= 0)      return tr('view.hawkes.validate.beta');
    return null;
}

export function buildBody(input) {
    return {
        event_times: input.event_times,
        query_times: input.query_times,
        params: {
            baseline_mu:      input.params.baseline_mu,
            excitation_alpha: input.params.excitation_alpha,
            decay_beta:       input.params.decay_beta,
        },
    };
}

// Pure-JS mirror of crates/traderview-core/src/hawkes_intensity.rs::compute.
// Returns null on validation failure (same as Rust Option::None).
export function localCompute(event_times, query_times, params) {
    const err = validateInputs({ event_times, query_times, params });
    if (err) return null;
    const { baseline_mu: mu, excitation_alpha: alpha, decay_beta: beta } = params;
    const is_stable = alpha < beta;
    const unconditional = is_stable ? mu / (1 - alpha / beta) : Infinity;
    const intensities = query_times.map(t => {
        let lambda = mu;
        for (const ev of event_times) {
            if (ev >= t) break;
            lambda += alpha * Math.exp(-beta * (t - ev));
        }
        return lambda;
    });
    return { intensities, unconditional_mean_intensity: unconditional, is_stable };
}

// Mirror of intensity_after_each_event — peak just after each event.
export function localIntensityAfterEach(event_times, params) {
    const err = validateInputs({ event_times, query_times: [], params });
    if (err) return null;
    const { baseline_mu: mu, excitation_alpha: alpha, decay_beta: beta } = params;
    const out = [];
    for (let i = 0; i < event_times.length; i++) {
        const t = event_times[i];
        let lambda = mu;
        for (let j = 0; j < i; j++) {
            lambda += alpha * Math.exp(-beta * (t - event_times[j]));
        }
        lambda += alpha;
        out.push(lambda);
    }
    return out;
}

// Parse one timestamp per line; blank + # comments skipped.
export function parseTimesBlob(blob) {
    const out = { times: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: t('common.parse.input_must_be_string') });
        return out;
    }
    const lines = blob.split('\n');
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i].trim();
        if (!raw || raw.startsWith('#')) continue;
        const v = Number(raw);
        if (!Number.isFinite(v)) {
            out.errors.push({ line_no: i + 1, message: 'timestamp not finite' });
            continue;
        }
        out.times.push(v);
    }
    return out;
}

// Generate evenly-spaced query times bracketing event series ± padding.
export function makeQueryGrid(event_times, count = 100, padFrac = 0.1) {
    if (!Array.isArray(event_times) || event_times.length === 0) {
        return Array.from({ length: count }, (_, i) => i);
    }
    const lo = event_times[0];
    const hi = event_times[event_times.length - 1];
    const span = Math.max(hi - lo, 1);
    const pad = span * padFrac;
    const start = lo - pad;
    const end = hi + pad;
    const out = [];
    for (let i = 0; i < count; i++) {
        out.push(start + (end - start) * (i / (count - 1)));
    }
    return out;
}

// Stability badge — alpha/beta is the branching ratio.
export function stabilityBadge(params) {
    if (!params) return { key: 'view.hawkes.badge.unknown', cls: '' };
    const { excitation_alpha: a, decay_beta: b } = params;
    if (!Number.isFinite(a) || !Number.isFinite(b) || b <= 0)
        return { key: 'view.hawkes.badge.unknown', cls: '' };
    const ratio = a / b;
    if (ratio >= 1)     return { key: 'view.hawkes.badge.explosive',  cls: 'neg' };
    if (ratio >= 0.9)   return { key: 'view.hawkes.badge.critical',   cls: 'neg' };
    if (ratio >= 0.5)   return { key: 'view.hawkes.badge.clustered',  cls: '' };
    if (ratio > 0)      return { key: 'view.hawkes.badge.weak',       cls: 'pos' };
    return { key: 'view.hawkes.badge.poisson', cls: 'pos' };
}

// Peak-vs-baseline ratio — how much does flow cluster?
export function clusteringRatio(intensities, baseline_mu) {
    if (!Array.isArray(intensities) || intensities.length === 0 || !Number.isFinite(baseline_mu) || baseline_mu <= 0)
        return NaN;
    let mx = -Infinity;
    for (const v of intensities) if (Number.isFinite(v) && v > mx) mx = v;
    if (!Number.isFinite(mx)) return NaN;
    return mx / baseline_mu;
}

export function makeDemoInput(kind = 'cluster-burst') {
    switch (kind) {
        case 'poisson-baseline': {
            const events = Array.from({ length: 10 }, (_, i) => i + 1);
            return { event_times: events, query_times: makeQueryGrid(events),
                     params: { baseline_mu: 0.5, excitation_alpha: 0, decay_beta: 1.0 } };
        }
        case 'cluster-burst': {
            // 5 close-spaced events then quiet.
            const events = [1, 1.2, 1.4, 1.6, 1.8, 5, 8];
            return { event_times: events, query_times: makeQueryGrid(events),
                     params: { baseline_mu: 0.3, excitation_alpha: 0.6, decay_beta: 2.0 } };
        }
        case 'news-burst': {
            // Earnings spike: 8 events in 30 seconds, then 3 isolated.
            const burst = Array.from({ length: 8 }, (_, i) => 10 + i * 0.05);
            const events = [...burst, 15, 20, 25];
            return { event_times: events, query_times: makeQueryGrid(events, 200),
                     params: { baseline_mu: 0.2, excitation_alpha: 0.8, decay_beta: 3.0 } };
        }
        case 'critical': {
            // α/β = 0.95 — near explosion.
            const events = Array.from({ length: 8 }, (_, i) => i + 1);
            return { event_times: events, query_times: makeQueryGrid(events),
                     params: { baseline_mu: 0.1, excitation_alpha: 0.95, decay_beta: 1.0 } };
        }
        case 'explosive': {
            // α >= β — unstable.
            const events = [1, 2, 3];
            return { event_times: events, query_times: makeQueryGrid(events),
                     params: { baseline_mu: 0.1, excitation_alpha: 1.5, decay_beta: 1.0 } };
        }
        case 'no-events': {
            return { event_times: [], query_times: [0, 1, 2, 3, 4, 5],
                     params: { ...DEFAULT_PARAMS } };
        }
        case 'long-decay': {
            // β small → excitation persists.
            const events = [0, 1, 2];
            return { event_times: events, query_times: makeQueryGrid(events, 100, 1),
                     params: { baseline_mu: 0.1, excitation_alpha: 0.3, decay_beta: 0.1 } };
        }
        case 'fast-decay': {
            // β large → spike+immediate fade.
            const events = [1, 2, 3, 4, 5];
            return { event_times: events, query_times: makeQueryGrid(events, 200),
                     params: { baseline_mu: 0.2, excitation_alpha: 0.4, decay_beta: 10 } };
        }
        default:
            return makeDemoInput('cluster-burst');
    }
}

export function fmtNum(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return v === Infinity ? '∞' : '—';
    return v.toFixed(d);
}

export function fmtInt(v) {
    if (!Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}

export function fmtRatio(v) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(3);
}
