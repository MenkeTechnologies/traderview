// Beta regression helpers: parser, validator, body shape,
// localEstimate Rust-mirror, badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS,
    parsePairsBlob, pairsToBlob, validateInputs, buildBody, localEstimate,
    betaBadge, fitBadge, hedgeNotional, annualizeAlpha,
    makeDemoInput,
    fmtBeta, fmtAlpha, fmtR2, fmtPctSigned, fmtUSD, fmtInt,
} from '../js/_beta_inputs.js';

// ── parser ────────────────────────────────────────────────────────

test('parsePairsBlob: 2 tokens per line, comments + pct-suffix', () => {
    const r = parsePairsBlob('0.012 0.008\n# day 2\n1.5% 0.8%');
    expect(r.errors).toEqual([]);
    expect(r.asset).toEqual([0.012, 0.015]);
    expect(r.benchmark).toEqual([0.008, 0.008]);
});

test('parsePairsBlob: rejects wrong count + non-finite', () => {
    expect(parsePairsBlob('0.01').errors[0].message).toMatch(/2 tokens/);
    expect(parsePairsBlob('0.01 foo').errors[0].message).toMatch(/non-finite/);
});

test('parsePairsBlob: non-string returns 1 error', () => {
    expect(parsePairsBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts equal-length series ≥ 2', () => {
    expect(validateInputs({ asset: [0.01, 0.02], benchmark: [0.01, 0.02] })).toBe(null);
});

test('validate rejects: bad array / length mismatch / < 2 / NaN', () => {
    expect(validateInputs({ asset: 'no', benchmark: [] })).toMatch(/asset/);
    expect(validateInputs({ asset: [0.01], benchmark: [0.01, 0.02] })).toMatch(/length/);
    expect(validateInputs({ asset: [0.01], benchmark: [0.01] })).toMatch(/2 paired/);
    expect(validateInputs({ asset: [0.01, NaN], benchmark: [0.01, 0.02] })).toMatch(/asset/);
    expect(validateInputs({ asset: [0.01, 0.02], benchmark: [0.01, NaN] })).toMatch(/benchmark/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: forwards asset + benchmark verbatim', () => {
    const b = buildBody({ asset: [0.01], benchmark: [0.008] });
    expect(b).toEqual({ asset: [0.01], benchmark: [0.008] });
});

// ── localEstimate parity (mirrors every Rust #[test]) ────────────

test('local: empty → null', () => {
    expect(localEstimate([], [])).toBeNull();
});

test('local: length mismatch → null', () => {
    expect(localEstimate([1, 2], [1])).toBeNull();
});

test('local: zero-variance benchmark → null', () => {
    expect(localEstimate([1, 2, 3], [5, 5, 5])).toBeNull();
});

test('local: asset == benchmark → β=1, R²=1, α=0', () => {
    const r = localEstimate([1, 2, 3, 4, 5], [1, 2, 3, 4, 5]);
    expect(r.beta).toBeCloseTo(1, 12);
    expect(r.r_squared).toBeCloseTo(1, 12);
    expect(r.alpha).toBeCloseTo(0, 12);
});

test('local: asset = 2·benchmark → β=2, R²=1', () => {
    const r = localEstimate([2, 4, 6, 8, 10], [1, 2, 3, 4, 5]);
    expect(r.beta).toBeCloseTo(2, 9);
    expect(r.r_squared).toBeCloseTo(1, 12);
});

test('local: negatively correlated → β<0, corr<0', () => {
    const r = localEstimate([5, 4, 3, 2, 1], [1, 2, 3, 4, 5]);
    expect(r.beta).toBeCloseTo(-1, 9);
    expect(r.correlation).toBeLessThan(0);
});

test('local: alpha captures constant offset (asset = bench + 10)', () => {
    const bench = [1, 2, 3, 4, 5];
    const asset = bench.map(x => x + 10);
    const r = localEstimate(asset, bench);
    expect(r.beta).toBeCloseTo(1, 9);
    expect(r.alpha).toBeCloseTo(10, 9);
});

test('local: zero-variance asset → R²=0, corr=0', () => {
    const r = localEstimate([1, 1, 1, 1, 1, 1], [1, -1, 1, -1, 1, -1]);
    expect(r.r_squared).toBe(0);
    expect(r.correlation).toBe(0);
});

test('local: low-beta stock (asset = 10 + 0.3·(bench-10)) → β≈0.3', () => {
    const bench = [10, 11, 9, 12, 8];
    const asset = bench.map(x => 10 + (x - 10) * 0.3);
    const r = localEstimate(asset, bench);
    expect(r.beta).toBeCloseTo(0.3, 9);
});

test('local: n field matches input length', () => {
    const r = localEstimate([1, 2, 3], [1, 2, 4]);
    expect(r.n).toBe(3);
});

// ── badges ────────────────────────────────────────────────────────

test('betaBadge: 7-tier on beta', () => {
    expect(betaBadge(2.0).key).toMatch(/high_beta/);
    expect(betaBadge(1.2).key).toMatch(/above_market/);
    expect(betaBadge(1.0).key).toMatch(/market/);
    expect(betaBadge(0.7).key).toMatch(/low_beta/);
    expect(betaBadge(0.02).key).toMatch(/market_neutral/);
    expect(betaBadge(-0.5).key).toMatch(/negative_low/);
    expect(betaBadge(-2).key).toMatch(/negative_high/);
    expect(betaBadge(null).key).toMatch(/unknown/);
});

test('fitBadge: 5-tier on R²', () => {
    expect(fitBadge(0.9).key).toMatch(/strong/);
    expect(fitBadge(0.6).key).toMatch(/good/);
    expect(fitBadge(0.3).key).toMatch(/moderate/);
    expect(fitBadge(0.1).key).toMatch(/weak/);
    expect(fitBadge(0.02).key).toMatch(/noise/);
    expect(fitBadge(null).key).toMatch(/unknown/);
});

// ── hedge / annualize ────────────────────────────────────────────

test('hedgeNotional: notional × β', () => {
    expect(hedgeNotional(100000, 1.3)).toBeCloseTo(130000, 9);
    expect(hedgeNotional(100000, -1)).toBeCloseTo(-100000, 9);
    expect(Number.isNaN(hedgeNotional(NaN, 1))).toBe(true);
});

test('annualizeAlpha: α × periods_per_year', () => {
    expect(annualizeAlpha(0.0003, 252)).toBeCloseTo(0.0756, 6);
    expect(Number.isNaN(annualizeAlpha(0.001, 0))).toBe(true);
    expect(Number.isNaN(annualizeAlpha(NaN, 252))).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + estimates a non-null report', () => {
    for (const k of ['tech-stock','utility-low-beta','inverse-etf','market-neutral',
                     'high-beta-3x','perfect-match','no-correlation','flat-bench']) {
        const inp = makeDemoInput(k);
        const err = validateInputs(inp);
        if (err === null) {
            const r = localEstimate(inp.asset, inp.benchmark);
            // flat-bench is designed to return null (zero variance benchmark).
            if (k !== 'flat-bench') {
                expect(r).not.toBeNull();
            }
        }
    }
});

test('demo tech-stock: β within 0.1 of 1.3', () => {
    const inp = makeDemoInput('tech-stock');
    const r = localEstimate(inp.asset, inp.benchmark);
    expect(Math.abs(r.beta - 1.3)).toBeLessThan(0.1);
});

test('demo utility-low-beta: β within 0.1 of 0.3', () => {
    const inp = makeDemoInput('utility-low-beta');
    const r = localEstimate(inp.asset, inp.benchmark);
    expect(Math.abs(r.beta - 0.3)).toBeLessThan(0.1);
});

test('demo inverse-etf: β ≈ −1 (within 0.05)', () => {
    const inp = makeDemoInput('inverse-etf');
    const r = localEstimate(inp.asset, inp.benchmark);
    expect(Math.abs(r.beta - (-1))).toBeLessThan(0.05);
});

test('demo high-beta-3x: β ≈ 3', () => {
    const inp = makeDemoInput('high-beta-3x');
    const r = localEstimate(inp.asset, inp.benchmark);
    expect(Math.abs(r.beta - 3)).toBeLessThan(0.1);
});

test('demo perfect-match: β=1, R²=1', () => {
    const inp = makeDemoInput('perfect-match');
    const r = localEstimate(inp.asset, inp.benchmark);
    expect(r.beta).toBeCloseTo(1, 12);
    expect(r.r_squared).toBeCloseTo(1, 12);
});

test('demo no-correlation: |R²| < 0.05', () => {
    const inp = makeDemoInput('no-correlation');
    const r = localEstimate(inp.asset, inp.benchmark);
    expect(Math.abs(r.r_squared)).toBeLessThan(0.05);
});

test('demo flat-bench: returns null (degenerate variance)', () => {
    const inp = makeDemoInput('flat-bench');
    const r = localEstimate(inp.asset, inp.benchmark);
    expect(r).toBeNull();
});

// ── round-trip + formatters ──────────────────────────────────────

test('pairsToBlob round-trips through parsePairsBlob', () => {
    const a = [0.012, -0.005];
    const b = [0.008, -0.004];
    const back = parsePairsBlob(pairsToBlob(a, b));
    expect(back.errors).toEqual([]);
    expect(back.asset).toEqual(a);
    expect(back.benchmark).toEqual(b);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtBeta(1.234)).toBe('+1.2340');
    expect(fmtBeta(-1.234)).toBe('-1.2340');
    expect(fmtAlpha(0.000123)).toBe('+0.000123');
    expect(fmtR2(0.95)).toBe('0.9500');
    expect(fmtPctSigned(0.012)).toBe('+1.20%');
    expect(fmtUSD(1234)).toBe('$1234.00');
    expect(fmtInt(7.9)).toBe('7');
    expect(fmtBeta(NaN)).toBe('—');
});
