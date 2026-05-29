// Active Share helpers: parser, validator, body shape,
// localCompute Rust-mirror, badges, demos.

import { test, expect } from 'vitest';
import {
    OVER_TOL, DEFAULT_INPUTS,
    parseWeightsBlob, weightsToBlob, validateInputs, buildBody, localCompute,
    styleBadge, sumBadge, enrich, stanceLabelKey,
    makeDemoInput,
    fmtPct, fmtPctSigned, fmtNum, fmtInt,
} from '../js/_active_share_inputs.js';

const w = (sym, p, b) => ({ symbol: sym, portfolio_weight: p, benchmark_weight: b });

// ── constants ─────────────────────────────────────────────────────

test('OVER_TOL matches Rust 1e-12', () => {
    expect(OVER_TOL).toBe(1e-12);
});

// ── parser ────────────────────────────────────────────────────────

test('parseWeightsBlob: 3 tokens per line + decimal/pct mixed', () => {
    const r = parseWeightsBlob('AAPL 0.40 40%\nMSFT, 30%, 0.30');
    expect(r.errors).toEqual([]);
    expect(r.weights).toEqual([w('AAPL', 0.40, 0.40), w('MSFT', 0.30, 0.30)]);
});

test('parseWeightsBlob: comments + blanks ignored', () => {
    const r = parseWeightsBlob('# top\nAAPL 0.5 0.5\n\nMSFT 0.5 0.5');
    expect(r.errors).toEqual([]);
    expect(r.weights.length).toBe(2);
});

test('parseWeightsBlob: rejects wrong count / negative / NaN', () => {
    expect(parseWeightsBlob('AAPL 0.5').errors[0].message).toMatch(/3 tokens/);
    expect(parseWeightsBlob('AAPL -0.1 0.5').errors[0].message).toMatch(/portfolio_weight/);
    expect(parseWeightsBlob('AAPL 0.5 foo').errors[0].message).toMatch(/benchmark_weight/);
});

test('parseWeightsBlob: non-string returns 1 error', () => {
    expect(parseWeightsBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts non-empty input', () => {
    expect(validateInputs({ weights: [w('A', 0.5, 0.5)] })).toBe(null);
});

test('validate rejects: bad array / empty / bad fields', () => {
    expect(validateInputs({ weights: 'no' })).toMatch(/weights/);
    expect(validateInputs({ weights: [] })).toMatch(/non-empty/);
    expect(validateInputs({ weights: [{ symbol: '', portfolio_weight: 0.5, benchmark_weight: 0.5 }] })).toMatch(/symbol/);
    expect(validateInputs({ weights: [{ symbol: 'X', portfolio_weight: NaN, benchmark_weight: 0.5 }] })).toMatch(/portfolio_weight/);
    expect(validateInputs({ weights: [{ symbol: 'X', portfolio_weight: 0.5, benchmark_weight: -0.1 }] })).toMatch(/benchmark_weight/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: emits plain-shape weights (strips extras)', () => {
    const body = buildBody({ weights: [{ ...w('A', 0.5, 0.5), extra: 'x' }] });
    expect(body).toEqual({ weights: [w('A', 0.5, 0.5)] });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: empty → null', () => {
    expect(localCompute([])).toBeNull();
});

test('local: NaN / negative weight → null', () => {
    expect(localCompute([w('X', NaN, 0.5)])).toBeNull();
    expect(localCompute([w('X', -0.1, 0.5)])).toBeNull();
    expect(localCompute([w('X', 0.5, -0.1)])).toBeNull();
});

test('local: identical portfolio → AS = 0, 0 over, 0 under', () => {
    const r = localCompute([w('A', 0.4, 0.4), w('B', 0.3, 0.3), w('C', 0.3, 0.3)]);
    expect(r.active_share).toBeCloseTo(0, 12);
    expect(r.n_overweights).toBe(0);
    expect(r.n_underweights).toBe(0);
});

test('local: disjoint portfolios → AS = 1', () => {
    const r = localCompute([w('A', 1, 0), w('B', 0, 1)]);
    expect(r.active_share).toBeCloseTo(1, 12);
});

test('local: Cremers-Petajisto canonical → AS = 0.50', () => {
    const r = localCompute([w('A', 0.5, 0.5), w('B', 0.5, 0), w('C', 0, 0.5)]);
    expect(r.active_share).toBeCloseTo(0.5, 12);
});

test('local: over/under counts correct', () => {
    const r = localCompute([
        w('A', 0.5, 0.3),    // over
        w('B', 0.2, 0.3),    // under
        w('C', 0.3, 0.3),    // equal
        w('D', 0.0, 0.1),    // under
    ]);
    expect(r.n_overweights).toBe(1);
    expect(r.n_underweights).toBe(2);
});

test('local: weight sums reported', () => {
    const r = localCompute([w('A', 0.4, 0.5), w('B', 0.6, 0.5)]);
    expect(r.portfolio_weight_sum).toBeCloseTo(1, 12);
    expect(r.benchmark_weight_sum).toBeCloseTo(1, 12);
});

test('local: AS clamped to [0, 1]', () => {
    const r = localCompute([w('A', 0.7, 0.3), w('B', 0.3, 0.7)]);
    expect(r.active_share).toBeGreaterThanOrEqual(0);
    expect(r.active_share).toBeLessThanOrEqual(1);
});

test('local: n_names = input length', () => {
    const r = localCompute([w('A', 0.5, 0.5), w('B', 0.5, 0.5)]);
    expect(r.n_names).toBe(2);
});

test('local: tiny differences below OVER_TOL count as equal', () => {
    const r = localCompute([w('A', 0.5 + 1e-15, 0.5), w('B', 0.5 - 1e-15, 0.5)]);
    expect(r.n_overweights).toBe(0);
    expect(r.n_underweights).toBe(0);
});

// ── styleBadge / sumBadge ────────────────────────────────────────

test('styleBadge: 5-tier (closet / semi-closet / moderate / active / very-active)', () => {
    expect(styleBadge(0.10).key).toMatch(/closet/);
    expect(styleBadge(0.30).key).toMatch(/semi_closet/);
    expect(styleBadge(0.50).key).toMatch(/moderate/);
    expect(styleBadge(0.70).key).toMatch(/active/);
    expect(styleBadge(0.90).key).toMatch(/very_active/);
    expect(styleBadge(NaN).key).toMatch(/unknown/);
});

test('sumBadge: normalized / close / unnormalized', () => {
    expect(sumBadge(1.0).key).toMatch(/normalized/);
    expect(sumBadge(1.02).key).toMatch(/close/);
    expect(sumBadge(0.80).key).toMatch(/unnormalized/);
    expect(sumBadge(NaN).key).toMatch(/unknown/);
});

// ── enrich + stanceLabelKey ──────────────────────────────────────

test('enrich: adds diff/abs_diff/stance', () => {
    const a = enrich(w('A', 0.5, 0.3));
    expect(a.diff).toBeCloseTo(0.2, 9);
    expect(a.abs_diff).toBeCloseTo(0.2, 9);
    expect(a.stance).toBe('over');

    const b = enrich(w('B', 0.3, 0.5));
    expect(b.stance).toBe('under');

    const c = enrich(w('C', 0.5, 0.5));
    expect(c.stance).toBe('equal');
});

test('stanceLabelKey: i18n keys for over/under/equal/unknown', () => {
    expect(stanceLabelKey('over')).toBe('view.act_share.stance.over');
    expect(stanceLabelKey('under')).toBe('view.act_share.stance.under');
    expect(stanceLabelKey('equal')).toBe('view.act_share.stance.equal');
    expect(stanceLabelKey()).toBe('view.act_share.stance.unknown');
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes to a non-null report', () => {
    for (const k of ['identical','disjoint','cremers-canonical','closet-indexer',
                     'highly-active','sector-bet','short-bet','unnormalized']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.weights);
        expect(r).not.toBeNull();
        expect(r.active_share).toBeGreaterThanOrEqual(0);
        expect(r.active_share).toBeLessThanOrEqual(1);
    }
});

test('demo identical: AS = 0', () => {
    const r = localCompute(makeDemoInput('identical').weights);
    expect(r.active_share).toBeCloseTo(0, 12);
});

test('demo disjoint: AS = 1', () => {
    const r = localCompute(makeDemoInput('disjoint').weights);
    expect(r.active_share).toBeCloseTo(1, 12);
});

test('demo cremers-canonical: AS = 0.50', () => {
    const r = localCompute(makeDemoInput('cremers-canonical').weights);
    expect(r.active_share).toBeCloseTo(0.5, 12);
});

test('demo closet-indexer: AS < 0.20 → closet badge', () => {
    const r = localCompute(makeDemoInput('closet-indexer').weights);
    expect(r.active_share).toBeLessThan(0.20);
    expect(styleBadge(r.active_share).key).toMatch(/closet/);
});

test('demo highly-active: AS ≥ 0.60 → active or very_active badge', () => {
    const r = localCompute(makeDemoInput('highly-active').weights);
    expect(r.active_share).toBeGreaterThanOrEqual(0.60);
    expect(styleBadge(r.active_share).key).toMatch(/active|very_active/);
});

test('demo sector-bet: produces high AS (concentrated tech bet vs broad bench)', () => {
    const r = localCompute(makeDemoInput('sector-bet').weights);
    expect(r.active_share).toBeGreaterThan(0.60);
});

// ── round-trip + formatters ──────────────────────────────────────

test('weightsToBlob round-trips through parseWeightsBlob', () => {
    const weights = [w('AAPL', 0.30, 0.07), w('MSFT', 0.25, 0.07)];
    const back = parseWeightsBlob(weightsToBlob(weights));
    expect(back.errors).toEqual([]);
    expect(back.weights).toEqual(weights);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtPct(0.4)).toBe('40.00%');
    expect(fmtPctSigned(0.05)).toBe('+5.00%');
    expect(fmtPctSigned(-0.05)).toBe('-5.00%');
    expect(fmtNum(0.5)).toBe('0.5000');
    expect(fmtInt(42.7)).toBe('42');
    expect(fmtPct(NaN)).toBe('—');
});
