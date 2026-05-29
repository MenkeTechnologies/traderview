// Pure helpers for the option-payoff view: preset strategies + form/legs
// conversion. Lives outside the view file so vitest can exercise them
// without a DOM stub.
//
// Leg shape (UI):   { kind: 'call'|'put'|'underlying', strike, premium, qty }
// Backend payload:  same fields; the server's enum is matched by lower-case
//                   string.

/** Built-in strategies keyed by id. Each returns a fresh array of legs
 * sized to the given `spot` so the preset opens at-the-money by default
 * with sensible premiums (heuristic, not market-accurate).
 */
export const PRESETS = {
    long_call: (spot) => [
        { kind: 'call', strike: round5(spot), premium: spot * 0.03, qty: 1 },
    ],
    long_put: (spot) => [
        { kind: 'put',  strike: round5(spot), premium: spot * 0.03, qty: 1 },
    ],
    long_straddle: (spot) => [
        { kind: 'call', strike: round5(spot), premium: spot * 0.03, qty: 1 },
        { kind: 'put',  strike: round5(spot), premium: spot * 0.03, qty: 1 },
    ],
    long_strangle: (spot) => [
        { kind: 'call', strike: round5(spot * 1.05), premium: spot * 0.02, qty: 1 },
        { kind: 'put',  strike: round5(spot * 0.95), premium: spot * 0.02, qty: 1 },
    ],
    bull_call_spread: (spot) => [
        { kind: 'call', strike: round5(spot),        premium: spot * 0.03,  qty:  1 },
        { kind: 'call', strike: round5(spot * 1.05), premium: spot * 0.015, qty: -1 },
    ],
    bear_put_spread: (spot) => [
        { kind: 'put',  strike: round5(spot),        premium: spot * 0.03,  qty:  1 },
        { kind: 'put',  strike: round5(spot * 0.95), premium: spot * 0.015, qty: -1 },
    ],
    iron_condor: (spot) => [
        { kind: 'put',  strike: round5(spot * 0.90), premium: spot * 0.005, qty:  1 },
        { kind: 'put',  strike: round5(spot * 0.95), premium: spot * 0.015, qty: -1 },
        { kind: 'call', strike: round5(spot * 1.05), premium: spot * 0.015, qty: -1 },
        { kind: 'call', strike: round5(spot * 1.10), premium: spot * 0.005, qty:  1 },
    ],
    iron_butterfly: (spot) => [
        { kind: 'put',  strike: round5(spot * 0.95), premium: spot * 0.005, qty:  1 },
        { kind: 'put',  strike: round5(spot),        premium: spot * 0.03,  qty: -1 },
        { kind: 'call', strike: round5(spot),        premium: spot * 0.03,  qty: -1 },
        { kind: 'call', strike: round5(spot * 1.05), premium: spot * 0.005, qty:  1 },
    ],
    covered_call: (spot) => [
        { kind: 'underlying', strike: round5(spot),        premium: 0,             qty:  1 },
        { kind: 'call',       strike: round5(spot * 1.05), premium: spot * 0.015,  qty: -1 },
    ],
};

/** Round to nearest 5 for cleaner default strikes. Preserves negatives. */
function round5(x) {
    return Math.round(x / 5) * 5;
}

/** Validate a single leg. Returns null if valid, an error string otherwise. */
export function validateLeg(leg) {
    if (!leg || typeof leg !== 'object') return 'invalid leg';
    if (!['call', 'put', 'underlying'].includes(leg.kind)) return `bad kind: ${leg.kind}`;
    if (!Number.isFinite(leg.strike) || leg.strike <= 0) return 'strike must be > 0';
    if (!Number.isFinite(leg.premium)) return 'premium must be a number';
    if (!Number.isFinite(leg.qty) || leg.qty === 0) return 'qty must be non-zero';
    return null;
}

/** Validate the whole leg list. Returns null if every leg is valid, otherwise
 *  the first error tagged with leg index. */
export function validateLegs(legs) {
    if (!Array.isArray(legs) || legs.length === 0) return 'add at least one leg';
    for (let i = 0; i < legs.length; i++) {
        const err = validateLeg(legs[i]);
        if (err) return `leg ${i + 1}: ${err}`;
    }
    return null;
}

/** Build the JSON body for `/analytics/option-payoff-diagram`. */
export function buildPayoffBody(legs, spotMin, spotMax, steps) {
    return {
        legs: legs.map(l => ({
            kind: l.kind, strike: l.strike, premium: l.premium, qty: l.qty,
        })),
        spot_min: spotMin,
        spot_max: spotMax,
        steps,
    };
}

/** Build the JSON body for `/analytics/multi-leg-option-pricer`. */
export function buildPricerBody(legs, spot, tToExpiry, rate, divYield, sigma) {
    return {
        legs: legs.map(l => ({
            kind: l.kind, strike: l.strike, premium: l.premium, qty: l.qty,
        })),
        spot, t_to_expiry: tToExpiry, rate, div_yield: divYield, sigma,
    };
}

/** Pick a sensible spot-range around the current spot that's wide enough to
 *  see the strategy's wings. ±50% by default. */
export function defaultSpotRange(spot) {
    if (!Number.isFinite(spot) || spot <= 0) return { min: 0, max: 1 };
    return { min: spot * 0.5, max: spot * 1.5 };
}
