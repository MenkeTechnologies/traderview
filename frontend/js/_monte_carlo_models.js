// Pure helpers for the Monte-Carlo path simulator view. Owns the model
// registry (id → label + parameter schema + payload builder), input
// validation, and the normal-approximation PDF used to render the
// terminal-price distribution. Lives outside the view so vitest can
// exercise every model without a DOM stub.

/** Parameter spec for one model. `fields` is the list of inputs the form
 *  must collect (label + key + default). `endpoint` is the api.js method
 *  name (e.g. `anlyGbmPathSimulator`). `buildBody` shapes the JSON
 *  request from a flat values map. `extractTerminalStats` maps the
 *  response into a canonical { mean, stdev, min?, max?, skew?, extra? }
 *  shape so the view renders uniformly. */
export const MODELS = {
    gbm: {
        label: 'Geometric Brownian Motion',
        endpoint: 'anlyGbmPathSimulator',
        fields: [
            { key: 's0', label: 'Spot S₀', default: 100, min: 0, step: 'any' },
            { key: 'mu', label: 'Drift μ (annual)', default: 0.05, step: 'any' },
            { key: 'sigma', label: 'Vol σ (annual)', default: 0.20, min: 0, step: 'any' },
            { key: 'dt', label: 'dt (years)', default: 1 / 252, min: 0, step: 'any' },
            { key: 'steps', label: 'Steps', default: 252, min: 1, integer: true },
            { key: 'paths', label: 'Paths', default: 5000, min: 1, integer: true },
            { key: 'seed', label: 'Seed (0 = auto)', default: 42, min: 0, integer: true },
        ],
        buildBody: (v) => ({
            s0: v.s0, mu: v.mu, sigma: v.sigma,
            dt: v.dt, steps: v.steps, paths: v.paths, seed: v.seed,
        }),
        extractTerminalStats: (r) => ({
            mean: r.mean_terminal,
            stdev: r.stdev_terminal,
            min: r.min_terminal,
            max: r.max_terminal,
            paths_run: r.paths_run,
        }),
    },

    merton_jump: {
        label: 'Merton Jump-Diffusion',
        endpoint: 'anlyJumpDiffusionSimulator',
        fields: [
            { key: 's0', label: 'Spot S₀', default: 100, min: 0, step: 'any' },
            { key: 'mu', label: 'Drift μ (annual)', default: 0.05, step: 'any' },
            { key: 'sigma', label: 'Vol σ (annual)', default: 0.20, min: 0, step: 'any' },
            { key: 'jump_lambda', label: 'Jump rate λ (per year)', default: 1.0, min: 0, step: 'any' },
            { key: 'jump_mean', label: 'Jump mean (log)', default: -0.05, step: 'any' },
            { key: 'jump_stdev', label: 'Jump stdev (log)', default: 0.10, min: 0, step: 'any' },
            { key: 'dt', label: 'dt (years)', default: 1 / 252, min: 0, step: 'any' },
            { key: 'steps', label: 'Steps', default: 252, min: 1, integer: true },
            { key: 'paths', label: 'Paths', default: 5000, min: 1, integer: true },
            { key: 'seed', label: 'Seed (0 = auto)', default: 42, min: 0, integer: true },
        ],
        buildBody: (v) => ({
            s0: v.s0, mu: v.mu, sigma: v.sigma,
            jump_lambda: v.jump_lambda, jump_mean: v.jump_mean, jump_stdev: v.jump_stdev,
            dt: v.dt, steps: v.steps, paths: v.paths, seed: v.seed,
        }),
        extractTerminalStats: (r) => ({
            mean: r.mean_terminal,
            stdev: r.stdev_terminal,
            skew: r.skew_log_return,
            extra: { 'Jump count (total)': r.jump_count_total },
            paths_run: r.paths_run,
        }),
    },

    kou_jump: {
        label: 'Kou Double-Exp Jump-Diffusion',
        endpoint: 'anlyKouJumpDiffusionSimulator',
        fields: [
            { key: 's0', label: 'Spot S₀', default: 100, min: 0, step: 'any' },
            { key: 'mu', label: 'Drift μ (annual)', default: 0.05, step: 'any' },
            { key: 'sigma', label: 'Vol σ (annual)', default: 0.20, min: 0, step: 'any' },
            { key: 'jump_lambda', label: 'Jump rate λ (per year)', default: 1.0, min: 0, step: 'any' },
            { key: 'up_prob', label: 'Up-jump probability', default: 0.4, min: 0, max: 1, step: 'any' },
            { key: 'eta_up', label: 'η up (up-tail rate; > 1)', default: 10, min: 1.0001, step: 'any' },
            { key: 'eta_down', label: 'η down (down-tail rate)', default: 5, min: 0, step: 'any' },
            { key: 'dt', label: 'dt (years)', default: 1 / 252, min: 0, step: 'any' },
            { key: 'steps', label: 'Steps', default: 252, min: 1, integer: true },
            { key: 'paths', label: 'Paths', default: 5000, min: 1, integer: true },
            { key: 'seed', label: 'Seed (0 = auto)', default: 42, min: 0, integer: true },
        ],
        buildBody: (v) => ({
            s0: v.s0, mu: v.mu, sigma: v.sigma,
            jump_lambda: v.jump_lambda, up_prob: v.up_prob,
            eta_up: v.eta_up, eta_down: v.eta_down,
            dt: v.dt, steps: v.steps, paths: v.paths, seed: v.seed,
        }),
        extractTerminalStats: (r) => ({
            mean: r.mean_terminal,
            stdev: r.stdev_terminal,
            skew: r.skew_log_return,
            extra: {
                'Up jumps':   r.up_jumps,
                'Down jumps': r.down_jumps,
            },
            paths_run: r.paths_run,
        }),
    },

    fbm: {
        label: 'Fractional Brownian Motion (path)',
        endpoint: 'anlyFbmGenerator',
        fields: [
            { key: 'hurst', label: 'Hurst H (0=anti, 1=trend)', default: 0.7, min: 0.01, max: 0.99, step: 'any' },
            { key: 'sigma0', label: 'σ₀ (initial scale)', default: 1.0, min: 0, step: 'any' },
            { key: 'levels', label: 'Levels (path length = 2^L+1)', default: 10, min: 1, max: 18, integer: true },
            { key: 'seed', label: 'Seed (0 = auto)', default: 42, min: 0, integer: true },
        ],
        buildBody: (v) => ({
            hurst: v.hurst, sigma0: v.sigma0, levels: v.levels, seed: v.seed,
        }),
        // fbm endpoint returns the raw path (Vec<f64>), not stats. Derive
        // them locally so the view's response handling stays uniform.
        extractTerminalStats: (path) => {
            if (!Array.isArray(path) || path.length === 0) return null;
            const n = path.length;
            const mean = path.reduce((a, b) => a + b, 0) / n;
            const variance = path.reduce((a, b) => a + (b - mean) * (b - mean), 0) / n;
            const stdev = Math.sqrt(Math.max(variance, 0));
            const min = path.reduce((a, b) => Math.min(a, b), Infinity);
            const max = path.reduce((a, b) => Math.max(a, b), -Infinity);
            return {
                mean, stdev, min, max,
                extra: { 'Path samples': n },
                paths_run: 1,
                // The view checks for `path` to render the trace instead
                // of the normal-approximation density.
                path,
            };
        },
    },
};

/** Validate the values map against the model spec. Returns null on
 *  success, an error string with the offending field. Universal checks
 *  (required, finite, integer, range) only — model-specific physics
 *  (e.g. Kou's η_up > 1) is enforced by the `min` constraint here AND
 *  the backend's compute() rejects it on top. */
export function validateValues(modelId, values) {
    const model = MODELS[modelId];
    if (!model) return `unknown model "${modelId}"`;
    for (const f of model.fields) {
        const v = values[f.key];
        if (v == null) return `${f.label}: missing`;
        if (!Number.isFinite(v)) return `${f.label}: must be finite`;
        if (f.integer && !Number.isInteger(v)) return `${f.label}: must be an integer`;
        if (f.min != null && v < f.min) return `${f.label}: must be ≥ ${f.min}`;
        if (f.max != null && v > f.max) return `${f.label}: must be ≤ ${f.max}`;
    }
    return null;
}

/** Default-values map keyed by field id. Returned fresh each call so the
 *  caller can mutate without poisoning the registry. */
export function defaultValues(modelId) {
    const out = {};
    for (const f of MODELS[modelId].fields) out[f.key] = f.default;
    return out;
}

/** Generate a normal-approximation PDF over [mean − 4σ, mean + 4σ]. The
 *  caller passes (mean, stdev, points). Returns parallel `xs` and `ys`
 *  arrays — feed directly to uPlot. Useful as a sanity-visualization of
 *  where the simulated terminal distribution sits; NOT a real histogram
 *  (the backend doesn't expose per-path terminals). */
export function normalDensityCurve(mean, stdev, points = 161) {
    if (!Number.isFinite(mean) || !Number.isFinite(stdev) || stdev <= 0) {
        return { xs: [], ys: [] };
    }
    const xs = [];
    const ys = [];
    const span = 8 * stdev;
    const start = mean - 4 * stdev;
    const step = span / (points - 1);
    const norm = 1 / (stdev * Math.sqrt(2 * Math.PI));
    for (let i = 0; i < points; i++) {
        const x = start + i * step;
        const z = (x - mean) / stdev;
        xs.push(x);
        ys.push(norm * Math.exp(-0.5 * z * z));
    }
    return { xs, ys };
}
