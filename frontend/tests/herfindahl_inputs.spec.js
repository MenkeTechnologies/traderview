// Herfindahl-Hirschman helpers: parser, validator, body shape,
// localCompute Rust-mirror, badges, demos.

import { test, expect } from 'vitest';
import {
    DOJ_CONCENTRATED, DOJ_MODERATE,
    parsePositionsBlob, positionsToBlob, validateInputs, buildBody, localCompute,
    concentrationBadge, efficiencyBadge,
    makeDemoInput,
    fmtHhi, fmtScaled, fmtEffN, fmtPct, fmtInt,
} from '../js/_herfindahl_inputs.js';

// ── constants ─────────────────────────────────────────────────────

test('DOJ thresholds match regulatory convention', () => {
    expect(DOJ_MODERATE).toBe(1500);
    expect(DOJ_CONCENTRATED).toBe(2500);
});

// ── parser ────────────────────────────────────────────────────────

test('parsePositionsBlob: single-token form auto-labels pos_N', () => {
    const r = parsePositionsBlob('0.5\n0.3\n0.2');
    expect(r.errors).toEqual([]);
    expect(r.weights).toEqual([0.5, 0.3, 0.2]);
    expect(r.labels).toEqual(['pos_1', 'pos_2', 'pos_3']);
});

test('parsePositionsBlob: label+weight form preserves label', () => {
    const r = parsePositionsBlob('SPY 0.6\nQQQ 0.4');
    expect(r.errors).toEqual([]);
    expect(r.labels).toEqual(['SPY', 'QQQ']);
    expect(r.weights).toEqual([0.6, 0.4]);
});

test('parsePositionsBlob: blanks and # comments ignored', () => {
    const r = parsePositionsBlob('# header\n\nSPY 0.5\n# mid\nQQQ 0.5');
    expect(r.errors).toEqual([]);
    expect(r.labels).toEqual(['SPY', 'QQQ']);
});

test('parsePositionsBlob: rejects negative + non-finite weight + 3-token row', () => {
    expect(parsePositionsBlob('A -0.5').errors[0].message).toMatch(/weight/);
    expect(parsePositionsBlob('A foo').errors[0].message).toMatch(/finite/);
    expect(parsePositionsBlob('A B C').errors[0].message).toMatch(/1 or 2/);
});

test('parsePositionsBlob: non-string returns 1 error', () => {
    expect(parsePositionsBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts non-empty positive weights', () => {
    expect(validateInputs({ weights: [0.5, 0.5] })).toBe(null);
});

test('validate rejects: bad array / empty / non-finite / negative / all-zero', () => {
    expect(validateInputs({ weights: 'nope' })).toMatch(/array/);
    expect(validateInputs({ weights: [] })).toMatch(/non-empty/);
    expect(validateInputs({ weights: [0.5, NaN] })).toMatch(/finite/);
    expect(validateInputs({ weights: [0.5, -0.1] })).toMatch(/≥ 0/);
    expect(validateInputs({ weights: [0, 0] })).toMatch(/sum/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: forwards weights only (route doesn\'t accept labels)', () => {
    const body = buildBody({ weights: [0.5, 0.5], labels: ['SPY', 'QQQ'] });
    expect(body).toEqual({ weights: [0.5, 0.5] });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: empty → null', () => {
    expect(localCompute([])).toBeNull();
});

test('local: NaN or negative → null', () => {
    expect(localCompute([0.5, NaN])).toBeNull();
    expect(localCompute([0.5, -0.5])).toBeNull();
});

test('local: all-zero → null', () => {
    expect(localCompute([0, 0, 0])).toBeNull();
});

test('local: single full weight → HHI=1, effective_n=1', () => {
    const r = localCompute([1.0]);
    expect(r.hhi).toBeCloseTo(1, 12);
    expect(r.effective_n).toBe(1);
    expect(r.n_positions).toBe(1);
});

test('local: 4 equal → HHI=0.25, effective_n=4, scaled=2500', () => {
    const r = localCompute([0.25, 0.25, 0.25, 0.25]);
    expect(r.hhi).toBeCloseTo(0.25, 12);
    expect(r.effective_n).toBeCloseTo(4, 12);
    expect(r.hhi_scaled).toBeCloseTo(2500, 9);
});

test('local: unnormalized weights internally normalized (5,5,5,5 → HHI=0.25)', () => {
    const r = localCompute([5, 5, 5, 5]);
    expect(r.hhi).toBeCloseTo(0.25, 12);
});

test('local: scaled HHI for 4-equal hits DOJ 2500 threshold exactly', () => {
    const r = localCompute([0.25, 0.25, 0.25, 0.25]);
    expect(r.hhi_scaled).toBeCloseTo(2500, 9);
});

test('local: concentrated (0.80 + 4×0.05) → HHI=0.65, effective_n<2', () => {
    const r = localCompute([0.80, 0.05, 0.05, 0.05, 0.05]);
    expect(r.hhi).toBeCloseTo(0.65, 9);
    expect(r.effective_n).toBeLessThan(2);
});

test('local: max_weight normalized to sum (10,20,70 → 0.70)', () => {
    const r = localCompute([10, 20, 70]);
    expect(r.max_weight).toBeCloseTo(0.70, 9);
});

test('local: zero weights excluded from n_positions', () => {
    const r = localCompute([0.5, 0.5, 0, 0]);
    expect(r.n_positions).toBe(2);
});

test('local: effective_n bounded by n_positions when uniformly equal', () => {
    const r = localCompute([0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1, 0.1]);
    expect(r.effective_n).toBeCloseTo(10, 9);
    expect(r.n_positions).toBe(10);
});

test('local: effective_n always ≤ n_positions (inequality of concentration)', () => {
    const r = localCompute([0.50, 0.30, 0.20]);
    expect(r.effective_n).toBeLessThanOrEqual(r.n_positions);
});

// ── concentrationBadge ───────────────────────────────────────────

test('concentrationBadge: 5-tier ladder using scaled HHI', () => {
    const mk = (s) => ({ hhi_scaled: s });
    expect(concentrationBadge(mk(100)).key).toMatch(/well_diversified/);
    expect(concentrationBadge(mk(800)).key).toMatch(/diversified/);
    expect(concentrationBadge(mk(2000)).key).toMatch(/moderate/);
    expect(concentrationBadge(mk(3000)).key).toMatch(/highly/);
    expect(concentrationBadge(mk(6000)).key).toMatch(/extreme/);
    expect(concentrationBadge(null).key).toMatch(/unknown/);
});

// ── efficiencyBadge ───────────────────────────────────────────────

test('efficiencyBadge: 5-tier using effective_n / n_positions ratio', () => {
    const mk = (e, n) => ({ effective_n: e, n_positions: n });
    expect(efficiencyBadge(mk(10, 10)).key).toMatch(/optimal/);
    expect(efficiencyBadge(mk(8, 10)).key).toMatch(/good/);
    expect(efficiencyBadge(mk(6, 10)).key).toMatch(/fair/);
    expect(efficiencyBadge(mk(3, 10)).key).toMatch(/poor/);
    expect(efficiencyBadge(mk(1, 10)).key).toMatch(/wasted/);
    expect(efficiencyBadge(null).key).toMatch(/unknown/);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes a non-null report', () => {
    for (const k of ['equal-4','equal-10','concentrated','single-name',
                     'pareto-80-20','unnormalized','with-zeroes','60-40-style']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.weights);
        expect(r).not.toBeNull();
        expect(r.effective_n).toBeGreaterThan(0);
    }
});

test('demo single-name: HHI=1, effective_n=1', () => {
    const inp = makeDemoInput('single-name');
    const r = localCompute(inp.weights);
    expect(r.hhi).toBe(1);
    expect(r.effective_n).toBe(1);
});

test('demo equal-10: HHI=0.10, scaled=1000', () => {
    const inp = makeDemoInput('equal-10');
    const r = localCompute(inp.weights);
    expect(r.hhi).toBeCloseTo(0.10, 9);
    expect(r.hhi_scaled).toBeCloseTo(1000, 9);
});

test('demo concentrated: highly verdict (scaled > 2500)', () => {
    const inp = makeDemoInput('concentrated');
    const r = localCompute(inp.weights);
    expect(concentrationBadge(r).key).toMatch(/highly|extreme/);
});

test('demo unnormalized: produces same HHI as equal-4 (internal normalize)', () => {
    const a = localCompute(makeDemoInput('unnormalized').weights);
    const b = localCompute(makeDemoInput('equal-4').weights);
    expect(a.hhi).toBeCloseTo(b.hhi, 12);
});

test('demo with-zeroes: n_positions=2 even though array length is 5', () => {
    const inp = makeDemoInput('with-zeroes');
    const r = localCompute(inp.weights);
    expect(r.n_positions).toBe(2);
});

// ── positionsToBlob round-trip ───────────────────────────────────

test('positionsToBlob round-trips through parsePositionsBlob', () => {
    const labels = ['SPY', 'QQQ', 'GLD'];
    const weights = [0.5, 0.3, 0.2];
    const back = parsePositionsBlob(positionsToBlob(labels, weights));
    expect(back.errors).toEqual([]);
    expect(back.labels).toEqual(labels);
    expect(back.weights).toEqual(weights);
});

// ── formatters ────────────────────────────────────────────────────

test('fmt helpers + non-finite guards', () => {
    expect(fmtHhi(0.25)).toBe('0.2500');
    expect(fmtScaled(2500)).toBe('2500');
    expect(fmtEffN(4.2)).toBe('4.20');
    expect(fmtPct(0.3)).toBe('30.00%');
    expect(fmtInt(7.9)).toBe('7');
    expect(fmtHhi(NaN)).toBe('—');
    expect(fmtPct(NaN)).toBe('—');
});
