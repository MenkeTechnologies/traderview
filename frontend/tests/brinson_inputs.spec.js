// Brinson attribution helpers: parser, validator, body shape,
// localAnalyze Rust-mirror, badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS,
    parseInputsBlob, inputsToBlob, validateInputs, buildBody, localAnalyze,
    activeBadge, driverBadge, enrichSector,
    makeDemoInput,
    fmtPct, fmtPctSigned, fmtBps, fmtNum, fmtInt,
} from '../js/_brinson_inputs.js';

const s = (sec, pw, bw, pr, br) => ({
    sector: sec, portfolio_weight: pw, benchmark_weight: bw,
    portfolio_return: pr, benchmark_return: br,
});

// ── parser ────────────────────────────────────────────────────────

test('parseInputsBlob: 5 tokens; decimals + pct-suffix accepted', () => {
    const r = parseInputsBlob('Tech 0.30 0.20 12% 0.08\nEnergy 0.15 0.25 -3% 0.01');
    expect(r.errors).toEqual([]);
    expect(r.inputs).toEqual([
        s('Tech', 0.30, 0.20, 0.12, 0.08),
        s('Energy', 0.15, 0.25, -0.03, 0.01),
    ]);
});

test('parseInputsBlob: rejects wrong count + negative weight + NaN return', () => {
    expect(parseInputsBlob('Tech 0.5').errors[0].message).toMatch(/5 tokens/);
    expect(parseInputsBlob('Tech -0.1 0.5 0.05 0.04').errors[0].message).toMatch(/portfolio_weight/);
    expect(parseInputsBlob('Tech 0.5 0.5 foo 0.04').errors[0].message).toMatch(/finite/);
});

test('parseInputsBlob: non-string returns 1 error', () => {
    expect(parseInputsBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts non-empty inputs', () => {
    expect(validateInputs({ inputs: [s('A', 0.5, 0.5, 0.05, 0.05)] })).toBe(null);
});

test('validate rejects: bad array / empty / bad fields', () => {
    expect(validateInputs({ inputs: 'no' })).toMatch(/inputs/);
    expect(validateInputs({ inputs: [] })).toMatch(/non-empty/);
    expect(validateInputs({ inputs: [{ ...s('', 0.5, 0.5, 0.05, 0.05) }] })).toMatch(/sector/);
    expect(validateInputs({ inputs: [{ ...s('X', NaN, 0.5, 0.05, 0.05) }] })).toMatch(/portfolio_weight/);
    expect(validateInputs({ inputs: [{ ...s('X', 0.5, -0.1, 0.05, 0.05) }] })).toMatch(/benchmark_weight/);
    expect(validateInputs({ inputs: [{ ...s('X', 0.5, 0.5, NaN, 0.05) }] })).toMatch(/portfolio_return/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: emits plain-shape rows (strips extras)', () => {
    const body = buildBody({ inputs: [{ ...s('A', 0.5, 0.5, 0.05, 0.05), extra: 'x' }] });
    expect(body.inputs[0]).toEqual(s('A', 0.5, 0.5, 0.05, 0.05));
});

// ── localAnalyze parity (mirrors every Rust #[test]) ─────────────

test('local: empty → null', () => {
    expect(localAnalyze([])).toBeNull();
});

test('local: NaN / negative weight → null', () => {
    expect(localAnalyze([s('A', NaN, 0.5, 0.05, 0.05)])).toBeNull();
    expect(localAnalyze([s('A', -0.1, 0.5, 0.05, 0.05)])).toBeNull();
    expect(localAnalyze([s('A', 0.5, -0.1, 0.05, 0.05)])).toBeNull();
});

test('local: identical portfolio → zero active + zero on all three effects', () => {
    const r = localAnalyze([
        s('Tech', 0.4, 0.4, 0.05, 0.05),
        s('Energy', 0.3, 0.3, -0.02, -0.02),
        s('Health', 0.3, 0.3, 0.01, 0.01),
    ]);
    expect(r.total_active_return).toBeCloseTo(0, 12);
    expect(r.total_allocation).toBeCloseTo(0, 12);
    expect(r.total_selection).toBeCloseTo(0, 12);
    expect(r.total_interaction).toBeCloseTo(0, 12);
});

test('local: allocation effect positive when overweighting above-mean sector', () => {
    const r = localAnalyze([
        s('Tech',  0.6, 0.4, 0.05, 0.05),
        s('Other', 0.4, 0.6, 0.01, 0.01),
    ]);
    expect(r.total_allocation).toBeGreaterThan(0);
});

test('local: selection effect positive when picking better stocks; allocation = 0 with same weights', () => {
    const r = localAnalyze([
        s('Tech',  0.4, 0.4, 0.10, 0.05),
        s('Other', 0.6, 0.6, 0.02, 0.02),
    ]);
    expect(r.total_selection).toBeGreaterThan(0);
    expect(Math.abs(r.total_allocation)).toBeLessThan(1e-12);
});

test('local: interaction effect non-zero when both weight + return differ', () => {
    const r = localAnalyze([
        s('Tech',  0.6, 0.4, 0.10, 0.05),
        s('Other', 0.4, 0.6, 0.02, 0.02),
    ]);
    expect(Math.abs(r.total_interaction)).toBeGreaterThan(1e-9);
});

test('local: A + S + I = total_active_return (decomposition identity)', () => {
    const r = localAnalyze([
        s('Tech',   0.30, 0.20,  0.12, 0.08),
        s('Energy', 0.15, 0.25, -0.03, 0.01),
        s('Health', 0.25, 0.20,  0.05, 0.04),
        s('Fin',    0.30, 0.35,  0.02, 0.03),
    ]);
    const sum = r.total_allocation + r.total_selection + r.total_interaction;
    expect(sum).toBeCloseTo(r.total_active_return, 9);
});

test('local: per_sector count matches input', () => {
    const r = localAnalyze([
        s('A', 0.5, 0.5, 0.01, 0.01),
        s('B', 0.5, 0.5, 0.01, 0.01),
    ]);
    expect(r.per_sector.length).toBe(2);
});

test('local: portfolio_total_return = Σ w_p · r_p; benchmark_total_return = Σ w_b · r_b', () => {
    const r = localAnalyze([
        s('A', 0.3, 0.4, 0.10, 0.08),
        s('B', 0.7, 0.6, 0.05, 0.03),
    ]);
    expect(r.portfolio_total_return).toBeCloseTo(0.3 * 0.10 + 0.7 * 0.05, 9);
    expect(r.benchmark_total_return).toBeCloseTo(0.4 * 0.08 + 0.6 * 0.03, 9);
});

test('local: total_active = portfolio_total - benchmark_total', () => {
    const r = localAnalyze([s('A', 0.5, 0.5, 0.10, 0.05)]);
    expect(r.total_active_return).toBeCloseTo(r.portfolio_total_return - r.benchmark_total_return, 9);
});

// ── activeBadge ──────────────────────────────────────────────────

test('activeBadge: 5-tier on total_active', () => {
    expect(activeBadge(0.03).key).toMatch(/strong_alpha/);
    expect(activeBadge(0.01).key).toMatch(/alpha/);
    expect(activeBadge(0.001).key).toMatch(/flat/);
    expect(activeBadge(-0.01).key).toMatch(/lagging/);
    expect(activeBadge(-0.05).key).toMatch(/deep_lag/);
    expect(activeBadge(NaN).key).toMatch(/unknown/);
});

// ── driverBadge ──────────────────────────────────────────────────

test('driverBadge: which effect dominates', () => {
    expect(driverBadge({ total_allocation: 0.05, total_selection: 0.01, total_interaction: 0 }).key).toMatch(/allocation/);
    expect(driverBadge({ total_allocation: 0.01, total_selection: 0.05, total_interaction: 0 }).key).toMatch(/selection/);
    expect(driverBadge({ total_allocation: 0.01, total_selection: 0.01, total_interaction: 0.05 }).key).toMatch(/interaction/);
    expect(driverBadge({ total_allocation: 0, total_selection: 0, total_interaction: 0 }).key).toMatch(/none/);
    expect(driverBadge(null).key).toMatch(/unknown/);
});

// ── enrichSector ─────────────────────────────────────────────────

test('enrichSector: adds weight/return + diffs + total_effect', () => {
    const input = s('A', 0.5, 0.3, 0.10, 0.08);
    const eff = { sector: 'A', allocation_effect: 0.01, selection_effect: 0.005, interaction_effect: 0.002 };
    const r = enrichSector(input, eff);
    expect(r.weight_diff).toBeCloseTo(0.20, 9);
    expect(r.return_diff).toBeCloseTo(0.02, 9);
    expect(r.total_effect).toBeCloseTo(0.017, 9);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes to a non-null report', () => {
    for (const k of ['identical','allocation-win','selection-win','mixed',
                     'losing-overweight','cash-heavy','sector-bet','all-effects']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localAnalyze(inp.inputs);
        expect(r).not.toBeNull();
        expect(r.per_sector.length).toBe(inp.inputs.length);
    }
});

test('demo identical: zero active return + zero on every effect', () => {
    const r = localAnalyze(makeDemoInput('identical').inputs);
    expect(r.total_active_return).toBeCloseTo(0, 12);
    expect(r.total_allocation).toBeCloseTo(0, 12);
    expect(r.total_selection).toBeCloseTo(0, 12);
});

test('demo allocation-win: positive total_allocation', () => {
    const r = localAnalyze(makeDemoInput('allocation-win').inputs);
    expect(r.total_allocation).toBeGreaterThan(0);
});

test('demo selection-win: positive total_selection + ~zero allocation', () => {
    const r = localAnalyze(makeDemoInput('selection-win').inputs);
    expect(r.total_selection).toBeGreaterThan(0);
    expect(Math.abs(r.total_allocation)).toBeLessThan(1e-12);
});

test('demo cash-heavy: portfolio_total < benchmark_total (cash drag)', () => {
    const r = localAnalyze(makeDemoInput('cash-heavy').inputs);
    expect(r.portfolio_total_return).toBeLessThan(r.benchmark_total_return);
});

test('demo all-effects: all three totals non-zero', () => {
    const r = localAnalyze(makeDemoInput('all-effects').inputs);
    expect(Math.abs(r.total_allocation)).toBeGreaterThan(1e-9);
    expect(Math.abs(r.total_selection)).toBeGreaterThan(1e-9);
    expect(Math.abs(r.total_interaction)).toBeGreaterThan(1e-9);
});

// ── round-trip + formatters ──────────────────────────────────────

test('inputsToBlob round-trips through parseInputsBlob', () => {
    const inputs = [s('Tech', 0.3, 0.2, 0.12, 0.08), s('Energy', 0.15, 0.25, -0.03, 0.01)];
    const back = parseInputsBlob(inputsToBlob(inputs));
    expect(back.errors).toEqual([]);
    expect(back.inputs).toEqual(inputs);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtPct(0.4)).toBe('40.00%');
    expect(fmtPctSigned(0.05)).toBe('+5.00%');
    expect(fmtPctSigned(-0.05)).toBe('-5.00%');
    expect(fmtBps(0.0123)).toBe('+123.0 bps');
    expect(fmtBps(-0.005)).toBe('-50.0 bps');
    expect(fmtNum(0.5)).toBe('0.5000');
    expect(fmtInt(42.7)).toBe('42');
    expect(fmtPct(NaN)).toBe('—');
});
