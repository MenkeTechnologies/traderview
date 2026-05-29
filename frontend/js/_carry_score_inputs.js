// Carry-trade score helpers.
//
// Backend body: { long_rate, funding_rate, annualized_vol }.
// Returns: { long_rate, funding_rate, rate_differential, annualized_vol,
//   carry_score, tier: 'strong'|'okay'|'poor'|'negative' }.
//
// Score = (long_rate - funding_rate) / annualized_vol (0 when vol = 0).
// Tier priority:
//   1. diff < 0 → negative (regardless of score sign)
//   2. score ≥ 1.0 → strong   (strict ≥)
//   3. score ≥ 0.5 → okay     (strict ≥)
//   4. else        → poor

export const DEFAULT_INPUTS = {
    long_rate: 0.05,
    funding_rate: 0.01,
    annualized_vol: 0.10,
};

export const TIERS = ['strong', 'okay', 'poor', 'negative'];

export function validateInputs(input) {
    if (!Number.isFinite(input.long_rate))      return 'long_rate must be finite';
    if (!Number.isFinite(input.funding_rate))   return 'funding_rate must be finite';
    if (!Number.isFinite(input.annualized_vol)) return 'annualized_vol must be finite';
    if (input.annualized_vol < 0)               return 'annualized_vol must be ≥ 0';
    return null;
}

export function buildBody(input) {
    return {
        long_rate: input.long_rate,
        funding_rate: input.funding_rate,
        annualized_vol: input.annualized_vol,
    };
}

// Mirror of crates/traderview-core/src/carry_score.rs::score.
export function localScore(long_rate, funding_rate, annualized_vol) {
    const diff = long_rate - funding_rate;
    const sc = annualized_vol > 0 ? diff / annualized_vol : 0;
    let tier;
    if (diff < 0)        tier = 'negative';
    else if (sc >= 1.0)  tier = 'strong';
    else if (sc >= 0.5)  tier = 'okay';
    else                 tier = 'poor';
    return {
        long_rate, funding_rate,
        rate_differential: diff,
        annualized_vol,
        carry_score: sc,
        tier,
    };
}

const TIER_BADGES = {
    strong:   { key: 'view.carry_score.badge.strong',   cls: 'pos' },
    okay:     { key: 'view.carry_score.badge.okay',     cls: '' },
    poor:     { key: 'view.carry_score.badge.poor',     cls: 'neg' },
    negative: { key: 'view.carry_score.badge.negative', cls: 'neg' },
};

export function tierBadge(tier) {
    return TIER_BADGES[tier] || { key: 'view.carry_score.badge.unknown', cls: '' };
}

// Convenience for the rule-of-thumb note line.
export function noteKeyForTier(tier) {
    return `view.carry_score.note.${tier || 'unknown'}`;
}

// 8 demos — every tier + boundary + edge cases.
export function makeDemoInput(kind = 'strong-mxn-jpy') {
    switch (kind) {
        case 'strong-mxn-jpy':
            // Mexican Peso long (~10%) vs Japanese Yen funding (~0%), low FX vol (~7%).
            return { long_rate: 0.10, funding_rate: 0.005, annualized_vol: 0.07 };
        case 'okay-aud-jpy':
            // AUD ~4% / JPY ~0% / vol ~7% → score ~0.57.
            return { long_rate: 0.04, funding_rate: 0.005, annualized_vol: 0.07 };
        case 'poor-high-vol':
            // EM long but vol crushes score: 8% / 1% / 25% → 0.28.
            return { long_rate: 0.08, funding_rate: 0.01, annualized_vol: 0.25 };
        case 'negative-anti-carry':
            // Borrowing at high rate to buy low-yield asset.
            return { long_rate: 0.01, funding_rate: 0.05, annualized_vol: 0.10 };
        case 'boundary-strong':
            // Exactly 1.0 → strong (≥ boundary).
            return { long_rate: 0.05, funding_rate: 0, annualized_vol: 0.05 };
        case 'boundary-okay':
            // Exactly 0.5 → okay (≥ boundary).
            return { long_rate: 0.025, funding_rate: 0, annualized_vol: 0.05 };
        case 'zero-vol':
            return { long_rate: 0.05, funding_rate: 0.01, annualized_vol: 0 };
        case 'eur-vs-usd-2024':
            // Recent ECB ~3.75% vs Fed ~4.5% → negative carry, 8% vol.
            return { long_rate: 0.0375, funding_rate: 0.045, annualized_vol: 0.08 };
        default:
            return makeDemoInput('strong-mxn-jpy');
    }
}

export function fmtPct(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

export function fmtPctSigned(v, d = 2) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '') + (v * 100).toFixed(d) + '%';
}

export function fmtScore(v, d = 3) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}
