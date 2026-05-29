// Burke ratio helpers: parser, validator, localCompute parity (peak-trough DD), badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_RISK_FREE, DEFAULT_PERIODS_PER_YEAR, MIN_OBS,
    parseEquityBlob, equityToBlob, validateInputs, buildBody, localCompute,
    drawdownEpisodes, ratioBadge, ddBadge, excessBadge, summarizeEquity,
    makeDemoInput,
    fmtRatio, fmtRatioSigned, fmtPct, fmtPctSigned, fmtPrice, fmtInt,
} from '../js/_burke_inputs.js';

// ── parser ────────────────────────────────────────────────────────

test('parseEquityBlob: comma + whitespace', () => {
    const r = parseEquityBlob('100 100.5\n# noise\n101, 102');
    expect(r.errors).toEqual([]);
    expect(r.equity).toEqual([100, 100.5, 101, 102]);
});

test('parseEquityBlob: rejects non-positive', () => {
    expect(parseEquityBlob('100 -5 0 102').errors.length).toBe(2);
});

test('parseEquityBlob: non-string returns 1 error', () => {
    expect(parseEquityBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts valid', () => {
    expect(validateInputs({ equity: [100, 110, 105], risk_free_total: 0, periods_per_year: 252 })).toBe(null);
});

test('validate rejects: bad array / too short / bad rf / bad periods / NaN / non-positive', () => {
    const base = { equity: [100, 110, 105], risk_free_total: 0, periods_per_year: 252 };
    expect(validateInputs({ ...base, equity: 'no' })).toMatch(/equity/);
    expect(validateInputs({ ...base, equity: [100] })).toMatch(/2 equity/);
    expect(validateInputs({ ...base, risk_free_total: NaN })).toMatch(/risk_free_total/);
    expect(validateInputs({ ...base, periods_per_year: 0 })).toMatch(/periods_per_year/);
    expect(validateInputs({ ...base, equity: [100, NaN, 105] })).toMatch(/finite/);
    expect(validateInputs({ ...base, equity: [100, -10, 105] })).toMatch(/> 0/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody copies + passes through', () => {
    expect(buildBody({ equity: [100, 110], risk_free_total: 0.02, periods_per_year: 252 }))
        .toEqual({ equity: [100, 110], risk_free_total: 0.02, periods_per_year: 252 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: too-short returns null', () => {
    expect(localCompute([100], 0, 252)).toBe(null);
});

test('local: invalid inputs return null', () => {
    expect(localCompute([100, 110], 0, 0)).toBe(null);
    expect(localCompute([100, NaN], 0, 252)).toBe(null);
    expect(localCompute([100, 0], 0, 252)).toBe(null);
    expect(localCompute([100, -10], 0, 252)).toBe(null);
});

test('local: monotone uptrend yields zero drawdowns + burke = 0', () => {
    const eq = Array.from({ length: 20 }, (_, i) => 100 + i);
    const r = localCompute(eq, 0, 252);
    expect(r.n_drawdowns).toBe(0);
    expect(r.sum_squared_drawdowns).toBe(0);
    expect(r.burke_ratio).toBe(0);
});

test('local: single drawdown recorded', () => {
    const eq = [100, 110, 99, 120];
    const r = localCompute(eq, 0, 252);
    expect(r.n_drawdowns).toBe(1);
    const expected_dd = (110 - 99) / 110;
    expect(Math.abs(r.sum_squared_drawdowns - expected_dd ** 2)).toBeLessThan(1e-12);
});

test('local: multiple drawdowns summed', () => {
    const eq = [100, 110, 95, 115, 100, 120];
    const r = localCompute(eq, 0, 252);
    expect(r.n_drawdowns).toBe(2);
    const dd_1 = (110 - 95) / 110;
    const dd_2 = (115 - 100) / 115;
    const expected = dd_1 ** 2 + dd_2 ** 2;
    expect(Math.abs(r.sum_squared_drawdowns - expected)).toBeLessThan(1e-12);
});

test('local: risk_free offset subtracted', () => {
    const eq = [100, 110, 95, 120];
    const r0 = localCompute(eq, 0, 252);
    const r5 = localCompute(eq, 0.05, 252);
    expect(r5.burke_ratio).toBeLessThan(r0.burke_ratio);
});

test('local: modified burke scales with √periods_per_year', () => {
    const eq = [100, 110, 95, 120];
    const r12 = localCompute(eq, 0, 12);
    const r252 = localCompute(eq, 0, 252);
    const ratio = r252.modified_burke_ratio / r12.modified_burke_ratio;
    const expected = Math.sqrt(252 / 12);
    expect(Math.abs(ratio - expected)).toBeLessThan(1e-6);
});

test('local: total_return computed end/start − 1', () => {
    const r = localCompute([100, 150], 0, 252);
    expect(Math.abs(r.total_return - 0.5)).toBeLessThan(1e-12);
});

test('local: open-ended DD captured (final low not yet recovered)', () => {
    const eq = [100, 120, 110, 90];   // peaked at 120, ends at 90 — no recovery
    const r = localCompute(eq, 0, 252);
    expect(r.n_drawdowns).toBe(1);
    const expected_dd = (120 - 90) / 120;
    expect(Math.abs(r.sum_squared_drawdowns - expected_dd ** 2)).toBeLessThan(1e-12);
});

test('local: deterministic', () => {
    const eq = [100, 110, 95, 115, 100, 120];
    expect(localCompute(eq, 0.02, 252)).toEqual(localCompute(eq, 0.02, 252));
});

// ── drawdownEpisodes ─────────────────────────────────────────────

test('drawdownEpisodes: returns 2 episodes for multi-DD series', () => {
    const eq = [100, 110, 95, 115, 100, 120];
    const eps = drawdownEpisodes(eq);
    expect(eps.length).toBe(2);
    expect(eps[0].peak_value).toBe(110);
    expect(eps[0].trough_value).toBe(95);
    expect(eps[0].recovery_idx).toBe(3);
    expect(eps[1].peak_value).toBe(115);
    expect(eps[1].trough_value).toBe(100);
    expect(eps[1].recovery_idx).toBe(5);
});

test('drawdownEpisodes: open-ended DD has recovery_idx null', () => {
    const eq = [100, 120, 110, 90];
    const eps = drawdownEpisodes(eq);
    expect(eps.length).toBe(1);
    expect(eps[0].recovery_idx).toBe(null);
});

test('drawdownEpisodes: empty/single → []', () => {
    expect(drawdownEpisodes([])).toEqual([]);
    expect(drawdownEpisodes([100])).toEqual([]);
});

// ── badges ────────────────────────────────────────────────────────

test('ratioBadge: 5 tiers', () => {
    expect(ratioBadge(3.0).key).toMatch(/exceptional/);
    expect(ratioBadge(1.5).key).toMatch(/strong/);
    expect(ratioBadge(0.75).key).toMatch(/moderate/);
    expect(ratioBadge(0.1).key).toMatch(/weak/);
    expect(ratioBadge(-0.5).key).toMatch(/negative/);
    expect(ratioBadge(NaN).key).toMatch(/unknown/);
});

test('ddBadge: 6 tiers', () => {
    expect(ddBadge(0, 0).key).toMatch(/none/);
    expect(ddBadge(0.0001, 1).key).toMatch(/tiny/);
    expect(ddBadge(0.001, 1).key).toMatch(/mild/);   // rms ≈ 0.032
    expect(ddBadge(0.02, 1).key).toMatch(/notable/);   // rms ≈ 0.14
    expect(ddBadge(0.05, 1).key).toMatch(/severe/);   // rms ≈ 0.22
    expect(ddBadge(0.5, 1).key).toMatch(/catastrophic/); // rms ≈ 0.71
    expect(ddBadge(NaN, 1).key).toMatch(/unknown/);
});

test('excessBadge: 5 tiers', () => {
    expect(excessBadge(0.30, 0).key).toMatch(/strong_alpha/);
    expect(excessBadge(0.10, 0).key).toMatch(/alpha/);
    expect(excessBadge(0.00, 0).key).toMatch(/market/);
    expect(excessBadge(-0.10, 0).key).toMatch(/underperform/);
    expect(excessBadge(-0.30, 0).key).toMatch(/severe_underperform/);
    expect(excessBadge(NaN, 0).key).toMatch(/unknown/);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarizeEquity: count / start / end / extrema / peak-to-trough', () => {
    const s = summarizeEquity([100, 120, 90, 110]);
    expect(s.count).toBe(4);
    expect(s.start).toBe(100);
    expect(s.end).toBe(110);
    expect(s.min).toBe(90);
    expect(s.max).toBe(120);
    expect(s.peak_to_trough).toBeCloseTo((120 - 90) / 120, 9);
});

test('summarizeEquity: empty → NaN', () => {
    const s = summarizeEquity([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.start)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes', () => {
    for (const k of ['steady-growth','high-sharpe','volatile-uptrend','deep-drawdown',
                     'multi-drawdowns','losing-strategy','monthly','one-big-dd']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.equity, inp.risk_free_total, inp.periods_per_year);
        expect(r).not.toBe(null);
    }
});

test('demo deep-drawdown: at least 1 DD', () => {
    const inp = makeDemoInput('deep-drawdown');
    const r = localCompute(inp.equity, inp.risk_free_total, inp.periods_per_year);
    expect(r.n_drawdowns).toBeGreaterThan(0);
});

test('demo monthly uses periods_per_year=12', () => {
    const inp = makeDemoInput('monthly');
    expect(inp.periods_per_year).toBe(12);
});

// ── formatters ────────────────────────────────────────────────────

test('equityToBlob round-trips', () => {
    const eq = [100, 100.5, 101.25];
    const back = parseEquityBlob(equityToBlob(eq));
    expect(back.errors).toEqual([]);
    expect(back.equity).toEqual(eq);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtRatio(1.23456)).toBe('1.2346');
    expect(fmtRatioSigned(1.5)).toBe('+1.5000');
    expect(fmtRatioSigned(-1.5)).toBe('-1.5000');
    expect(fmtPct(0.0125)).toBe('1.25%');
    expect(fmtPctSigned(0.05)).toBe('+5.00%');
    expect(fmtPctSigned(-0.05)).toBe('-5.00%');
    expect(fmtPrice(100.456)).toBe('100.46');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtRatio(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(DEFAULT_INPUTS.equity).toEqual([]);
    expect(DEFAULT_INPUTS.risk_free_total).toBe(DEFAULT_RISK_FREE);
    expect(DEFAULT_INPUTS.periods_per_year).toBe(DEFAULT_PERIODS_PER_YEAR);
    expect(DEFAULT_RISK_FREE).toBe(0);
    expect(DEFAULT_PERIODS_PER_YEAR).toBe(252);
    expect(MIN_OBS).toBe(2);
});
