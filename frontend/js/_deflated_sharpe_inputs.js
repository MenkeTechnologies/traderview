// Deflated Sharpe Ratio (Bailey & López de Prado 2014) — helpers shared
// by view + vitest.
//
// Backend body shape: { observed_sharpe, n_observations, skewness,
// kurtosis, n_trials }.

import { t } from './i18n.js';

export function validateInputs(p) {
    if (!Number.isFinite(p.observed_sharpe)) return t('view.deflated_sharpe.validate.observed_sharpe');
    if (!Number.isInteger(p.n_observations) || p.n_observations < 4)
        return t('view.deflated_sharpe.validate.n_observations');
    if (!Number.isFinite(p.skewness)) return t('view.deflated_sharpe.validate.skewness');
    if (!Number.isFinite(p.kurtosis)) return t('view.deflated_sharpe.validate.kurtosis');
    if (!Number.isInteger(p.n_trials) || p.n_trials < 1) return t('view.deflated_sharpe.validate.n_trials');
    // Mertens denominator validity check — same gate the backend uses.
    const sr = p.observed_sharpe;
    const denom = 1.0 - p.skewness * sr + ((p.kurtosis - 1.0) / 4.0) * sr * sr;
    if (!Number.isFinite(denom) || denom <= 0) return t('view.deflated_sharpe.validate.mertens_denom');
    return null;
}

export function buildBody(p) {
    return {
        observed_sharpe: p.observed_sharpe,
        n_observations: p.n_observations,
        skewness:       p.skewness,
        kurtosis:       p.kurtosis,
        n_trials:       p.n_trials,
    };
}

// Translates a probability into a confidence-tier label that traders read
// at a glance. Buckets follow standard academic/CFA-style cutoffs.
export function confidenceTier(prob) {
    if (!Number.isFinite(prob)) return { label: '—', cls: '' };
    if (prob >= 0.99) return { label: t('view.deflated_sharpe.tier.very_high'), cls: 'pos' };
    if (prob >= 0.95) return { label: t('view.deflated_sharpe.tier.high'),      cls: 'pos' };
    if (prob >= 0.90) return { label: t('view.deflated_sharpe.tier.moderate'),  cls: '' };
    if (prob >= 0.50) return { label: t('view.deflated_sharpe.tier.weak'),      cls: 'neg' };
    return { label: t('view.deflated_sharpe.tier.overfit'), cls: 'neg' };
}

// Builds a geometric n_trials ladder for the sensitivity sweep — answers
// "if I tested K strategies instead of N, what's my real PSR?"
export function trialsSweep(base) {
    if (!Number.isInteger(base) || base < 1) base = 10;
    const out = [1, 5, 10, 25, 50, 100, 250, 1000];
    if (!out.includes(base)) out.push(base);
    return Array.from(new Set(out)).sort((a, b) => a - b);
}

export function fmtSR(v) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(3);
}

export function fmtProb(v) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(2) + '%';
}

export function fmtZ(v) {
    if (!Number.isFinite(v)) return '—';
    const sign = v >= 0 ? '+' : '';
    return sign + v.toFixed(2) + 'σ';
}
