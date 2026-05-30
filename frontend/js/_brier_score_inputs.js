// Brier Score helpers — probabilistic forecast accuracy (Brier 1950).
//
// Backend body: { probabilities: number[], outcomes: 0/1[], n_bins }
// Returns: { brier_score, reliability, resolution, uncertainty,
//   base_rate, n_observations } | null
//
// BS = (1/N) · Σ (p_i − y_i)²,   range [0, 1] (0 = perfect).
// Murphy 1973 decomposition (exact only when each bin holds identical probs):
//   BS = reliability − resolution + uncertainty
//   reliability  = Σ (n_k/N) · (p̄_k − ō_k)²    (calibration)
//   resolution   = Σ (n_k/N) · (ō_k − ō)²        (discrimination)
//   uncertainty  = ō · (1 − ō)                    (base-rate variance)

import { t } from './i18n.js';

export const DEFAULT_BINS = 10;

export const DEFAULT_INPUTS = {
    probabilities: [],
    outcomes: [],
    n_bins: DEFAULT_BINS,
};

export function validateInputs(input) {
    if (!Array.isArray(input.probabilities))                      return t('view.brier_score.validate.probs_array');
    if (!Array.isArray(input.outcomes))                           return t('view.brier_score.validate.outcomes_array');
    if (input.probabilities.length === 0)                          return t('view.brier_score.validate.probs_empty');
    if (input.probabilities.length !== input.outcomes.length)     return t('view.brier_score.validate.length_mismatch');
    for (let i = 0; i < input.probabilities.length; i++) {
        const p = input.probabilities[i];
        if (!Number.isFinite(p))                                   return t('view.brier_score.validate.prob_finite', { i });
        if (p < 0 || p > 1)                                        return t('view.brier_score.validate.prob_range', { i });
    }
    for (let i = 0; i < input.outcomes.length; i++) {
        const y = input.outcomes[i];
        if (!Number.isInteger(y) || (y !== 0 && y !== 1))         return t('view.brier_score.validate.outcome_binary', { i });
    }
    if (!Number.isInteger(input.n_bins))                          return t('view.brier_score.validate.n_bins_int');
    if (input.n_bins < 1)                                         return t('view.brier_score.validate.n_bins_min');
    return null;
}

export function buildBody(input) {
    return {
        probabilities: input.probabilities,
        outcomes:      input.outcomes,
        n_bins:        input.n_bins,
    };
}

// Pure-JS mirror of crates/traderview-core/src/brier_score.rs::compute.
export function localCompute(probabilities, outcomes, n_bins) {
    const n = probabilities.length;
    if (n === 0 || outcomes.length !== n || n_bins === 0) return null;
    for (const p of probabilities) if (!Number.isFinite(p) || p < 0 || p > 1) return null;
    for (const y of outcomes) if (y !== 0 && y !== 1) return null;
    let brier = 0, base_sum = 0;
    for (let i = 0; i < n; i++) {
        const d = probabilities[i] - outcomes[i];
        brier += d * d;
        base_sum += outcomes[i];
    }
    brier /= n;
    const base_rate = base_sum / n;
    const uncertainty = base_rate * (1 - base_rate);
    const bin_p_sum = new Array(n_bins).fill(0);
    const bin_y_sum = new Array(n_bins).fill(0);
    const bin_n     = new Array(n_bins).fill(0);
    for (let i = 0; i < n; i++) {
        const p = probabilities[i];
        let bin = Math.floor(p * n_bins);
        if (bin >= n_bins) bin = n_bins - 1;
        bin_p_sum[bin] += p;
        bin_y_sum[bin] += outcomes[i];
        bin_n[bin]++;
    }
    let reliability = 0, resolution = 0;
    for (let k = 0; k < n_bins; k++) {
        if (bin_n[k] === 0) continue;
        const nk = bin_n[k];
        const p_bar = bin_p_sum[k] / nk;
        const o_bar = bin_y_sum[k] / nk;
        const wt = nk / n;
        const dp = p_bar - o_bar;
        const dr = o_bar - base_rate;
        reliability += wt * dp * dp;
        resolution  += wt * dr * dr;
    }
    return {
        brier_score:    brier,
        reliability,
        resolution,
        uncertainty,
        base_rate,
        n_observations: n,
    };
}

// Parse "prob outcome" per line; blanks + # comments ignored.
export function parsePairsBlob(blob) {
    const out = { probabilities: [], outcomes: [], errors: [] };
    if (typeof blob !== 'string') {
        out.errors.push({ line_no: 0, message: t('common.parse.input_must_be_string') });
        return out;
    }
    const lines = blob.split('\n');
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i].split('#')[0].trim();
        if (!raw) continue;
        const toks = raw.split(/[\s,]+/).filter(t => t.length > 0);
        if (toks.length !== 2) {
            out.errors.push({ line_no: i + 1, message: 'expected 2 tokens (probability outcome)' });
            continue;
        }
        const p = Number(toks[0]);
        const y = Number(toks[1]);
        if (!Number.isFinite(p) || p < 0 || p > 1) {
            out.errors.push({ line_no: i + 1, message: 'probability must be in [0, 1]' });
            continue;
        }
        if (!Number.isInteger(y) || (y !== 0 && y !== 1)) {
            out.errors.push({ line_no: i + 1, message: 'outcome must be 0 or 1' });
            continue;
        }
        out.probabilities.push(p);
        out.outcomes.push(y);
    }
    return out;
}

export function pairsToBlob(probabilities, outcomes) {
    return probabilities.map((p, i) => `${p} ${outcomes[i]}`).join('\n');
}

// 5-tier verdict on Brier score (lower = better).
export function brierBadge(bs, uncertainty) {
    if (bs == null || !Number.isFinite(bs)) return { key: 'view.brier.badge.unknown', cls: '' };
    if (bs < 0.01)                          return { key: 'view.brier.badge.perfect',   cls: 'pos' };
    if (bs < 0.10)                          return { key: 'view.brier.badge.excellent', cls: 'pos' };
    if (bs < 0.20)                          return { key: 'view.brier.badge.good',      cls: 'pos' };
    if (Number.isFinite(uncertainty) && bs < uncertainty * 0.95)
                                              return { key: 'view.brier.badge.useful',    cls: '' };
    if (Number.isFinite(uncertainty) && bs <= uncertainty * 1.05)
                                              return { key: 'view.brier.badge.coin_flip', cls: 'neg' };
    return { key: 'view.brier.badge.worse_than_random', cls: 'neg' };
}

// Skill score: BS_skill = 1 − BS/BS_ref where BS_ref = uncertainty (climatology).
export function skillScore(bs, uncertainty) {
    if (!Number.isFinite(bs) || !Number.isFinite(uncertainty) || uncertainty <= 0) return NaN;
    return 1 - bs / uncertainty;
}

// Bin-level reliability (a.k.a. calibration table).
export function reliabilityBins(probabilities, outcomes, n_bins) {
    const bins = Array.from({ length: n_bins }, (_, k) => ({
        bin: k,
        lo: k / n_bins,
        hi: (k + 1) / n_bins,
        count: 0,
        mean_pred: NaN,
        mean_actual: NaN,
    }));
    if (!Array.isArray(probabilities) || probabilities.length === 0) return bins;
    const sums = Array.from({ length: n_bins }, () => ({ ps: 0, os: 0, n: 0 }));
    for (let i = 0; i < probabilities.length; i++) {
        const p = probabilities[i];
        let bin = Math.floor(p * n_bins);
        if (bin >= n_bins) bin = n_bins - 1;
        sums[bin].ps += p;
        sums[bin].os += outcomes[i];
        sums[bin].n++;
    }
    for (let k = 0; k < n_bins; k++) {
        bins[k].count = sums[k].n;
        if (sums[k].n > 0) {
            bins[k].mean_pred = sums[k].ps / sums[k].n;
            bins[k].mean_actual = sums[k].os / sums[k].n;
        }
    }
    return bins;
}

// Deterministic demos.
function lcg(seed) {
    let state = BigInt(7919) + seed;
    return () => {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        return Number(state >> 32n) / 0xFFFFFFFF;
    };
}

export function makeDemoInput(kind = 'well-calibrated') {
    switch (kind) {
        case 'perfect': {
            const probabilities = [1, 0, 1, 0, 1, 0, 1, 0, 1, 0];
            const outcomes      = [1, 0, 1, 0, 1, 0, 1, 0, 1, 0];
            return { probabilities, outcomes, n_bins: 10 };
        }
        case 'random-coin-flip': {
            // 50/50 forecasts on balanced sample → BS = 0.25.
            return {
                probabilities: new Array(10).fill(0.5),
                outcomes:      [1, 0, 1, 0, 1, 0, 1, 0, 1, 0],
                n_bins: 10,
            };
        }
        case 'well-calibrated': {
            // Forecasts deciles match actual incidence (R^2 fit on identity).
            const rand = lcg(42n);
            const probabilities = [], outcomes = [];
            for (let i = 0; i < 500; i++) {
                const p = rand();
                probabilities.push(p);
                outcomes.push(rand() < p ? 1 : 0);
            }
            return { probabilities, outcomes, n_bins: 10 };
        }
        case 'overconfident': {
            // Predicts 0.9/0.1 but actuals are 0.7/0.3 — too extreme.
            const rand = lcg(7n);
            const probabilities = [], outcomes = [];
            for (let i = 0; i < 500; i++) {
                const true_p = rand() > 0.5 ? 0.7 : 0.3;
                const pred = true_p > 0.5 ? 0.9 : 0.1;
                probabilities.push(pred);
                outcomes.push(rand() < true_p ? 1 : 0);
            }
            return { probabilities, outcomes, n_bins: 10 };
        }
        case 'underconfident': {
            // Predicts 0.6/0.4 but actuals are 0.85/0.15.
            const rand = lcg(99n);
            const probabilities = [], outcomes = [];
            for (let i = 0; i < 500; i++) {
                const true_p = rand() > 0.5 ? 0.85 : 0.15;
                const pred = true_p > 0.5 ? 0.6 : 0.4;
                probabilities.push(pred);
                outcomes.push(rand() < true_p ? 1 : 0);
            }
            return { probabilities, outcomes, n_bins: 10 };
        }
        case 'flipped-sign': {
            // Predictions are inverted — high p = low actual.
            const rand = lcg(13n);
            const probabilities = [], outcomes = [];
            for (let i = 0; i < 500; i++) {
                const p = rand();
                probabilities.push(p);
                outcomes.push(rand() > p ? 1 : 0);  // inverse correlation
            }
            return { probabilities, outcomes, n_bins: 10 };
        }
        case 'rare-event': {
            // Base rate ≈ 5% — only some forecasters notice.
            const rand = lcg(1n);
            const probabilities = [], outcomes = [];
            for (let i = 0; i < 500; i++) {
                const actual = rand() < 0.05;
                const p = actual ? 0.5 : 0.05;   // some skill
                probabilities.push(p);
                outcomes.push(actual ? 1 : 0);
            }
            return { probabilities, outcomes, n_bins: 10 };
        }
        case 'fine-bins': {
            const rand = lcg(2n);
            const probabilities = [], outcomes = [];
            for (let i = 0; i < 1000; i++) {
                const p = rand();
                probabilities.push(p);
                outcomes.push(rand() < p ? 1 : 0);
            }
            return { probabilities, outcomes, n_bins: 50 };
        }
        default: return makeDemoInput('well-calibrated');
    }
}

export function fmtBrier(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return v.toFixed(d);
}

export function fmtSkill(v, d = 4) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + v.toFixed(d);
}

export function fmtPct(v, d = 2) {
    if (v == null || !Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtInt(v) {
    if (v == null || !Number.isFinite(v)) return '—';
    return String(Math.trunc(v));
}
