// Borrow rate indicator helpers: parser, validator, classify + localCompute parity, badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, DEFAULT_PERIOD, MIN_PERIOD, MAX_PERIOD, STRESS_LEVELS,
    parseRatesBlob, ratesToBlob, validateInputs, buildBody, localCompute, classify,
    stressBadge, trendBadge, escalationBadge, stressDistribution, summarizeRates,
    makeDemoInput,
    fmtPct, fmtPctSigned, fmtNum, fmtInt,
} from '../js/_borrow_rate_inputs.js';

// ── parser ────────────────────────────────────────────────────────

test('parseRatesBlob: comma + whitespace', () => {
    const r = parseRatesBlob('2.5 3.0\n# noise\n2.8, 3.2');
    expect(r.errors).toEqual([]);
    expect(r.rates_pct).toEqual([2.5, 3, 2.8, 3.2]);
});

test('parseRatesBlob: $/% prefix stripped', () => {
    const r = parseRatesBlob('5% 10% 25%');
    expect(r.errors).toEqual([]);
    expect(r.rates_pct).toEqual([5, 10, 25]);
});

test('parseRatesBlob: rejects negative', () => {
    expect(parseRatesBlob('2 -1 3').errors.length).toBe(1);
});

test('parseRatesBlob: non-string returns 1 error', () => {
    expect(parseRatesBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts valid', () => {
    expect(validateInputs({ rates_pct: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10], period: 5 })).toBe(null);
});

test('validate rejects: bad array / bad period / too short / non-finite / negative', () => {
    const base = { rates_pct: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10], period: 5 };
    expect(validateInputs({ ...base, rates_pct: 'no' })).toMatch(/rates_pct/);
    expect(validateInputs({ ...base, period: 0 })).toMatch(/period/);
    expect(validateInputs({ ...base, period: 9999 })).toMatch(/period/);
    expect(validateInputs({ ...base, rates_pct: [1, 2] })).toMatch(/period \+ 1/);
    expect(validateInputs({ ...base, rates_pct: [1, NaN, 3, 4, 5, 6, 7] })).toMatch(/finite/);
    expect(validateInputs({ ...base, rates_pct: [1, -1, 3, 4, 5, 6, 7] })).toMatch(/negative/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody copies + passes through', () => {
    expect(buildBody({ rates_pct: [1, 2, 3], period: 5 }))
        .toEqual({ rates_pct: [1, 2, 3], period: 5 });
});

// ── classify (mirrors every Rust branch) ──────────────────────────

test('classify: 5 tiers', () => {
    expect(classify(0.5, 0)).toBe('low_available');
    expect(classify(5, 0)).toBe('normal');
    expect(classify(25, 0)).toBe('tight');
    expect(classify(100, 0)).toBe('hard_to_borrow');
    expect(classify(300, 0)).toBe('extreme_squeeze');
    expect(classify(5, 200)).toBe('extreme_squeeze');   // change-driven
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: invalid inputs return all null', () => {
    const r = new Array(10).fill(0.5);
    expect(localCompute(r, 0).change_pct.every(x => x === null)).toBe(true);
    expect(localCompute(r.slice(0, 2), 5).change_pct.every(x => x === null)).toBe(true);
});

test('local: NaN or negative returns all null', () => {
    const r1 = new Array(10).fill(0.5);
    r1[3] = NaN;
    expect(localCompute(r1, 5).change_pct.every(x => x === null)).toBe(true);
    const r2 = new Array(10).fill(0.5);
    r2[3] = -1;
    expect(localCompute(r2, 5).change_pct.every(x => x === null)).toBe(true);
});

test('local: spike triggers extreme_squeeze via change_pct', () => {
    const r = [...new Array(10).fill(5), 12];   // 140% jump after 5 bars
    const rep = localCompute(r, 5);
    expect(rep.stress[10]).toBe('extreme_squeeze');
});

test('local: high rate alone triggers extreme_squeeze', () => {
    const r = new Array(10).fill(250);
    const rep = localCompute(r, 5);
    for (let i = 5; i < 10; i++) expect(rep.stress[i]).toBe('extreme_squeeze');
});

test('local: output lengths match input', () => {
    const r = new Array(10).fill(5);
    const rep = localCompute(r, 5);
    expect(rep.change_pct.length).toBe(10);
    expect(rep.stress.length).toBe(10);
});

test('local: leading change_pct null for first period bars', () => {
    const r = new Array(20).fill(5);
    const rep = localCompute(r, 5);
    for (let i = 0; i < 5; i++) expect(rep.change_pct[i]).toBe(null);
    expect(rep.change_pct[5]).not.toBe(null);
});

test('local: deterministic', () => {
    const r = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    expect(localCompute(r, 5)).toEqual(localCompute(r, 5));
});

test('local: prev = 0 → change_pct null (no division)', () => {
    const r = [0, 0, 0, 0, 0, 5];
    const rep = localCompute(r, 5);
    expect(rep.change_pct[5]).toBe(null);
});

test('local: stress always populated even when change null', () => {
    const r = new Array(10).fill(5);
    const rep = localCompute(r, 5);
    for (let i = 0; i < 10; i++) expect(rep.stress[i]).not.toBe(null);
});

// ── badges ────────────────────────────────────────────────────────

test('stressBadge: each level mapped', () => {
    expect(stressBadge('low_available').key).toMatch(/low_available/);
    expect(stressBadge('normal').key).toMatch(/normal/);
    expect(stressBadge('tight').key).toMatch(/tight/);
    expect(stressBadge('hard_to_borrow').key).toMatch(/hard_to_borrow/);
    expect(stressBadge('extreme_squeeze').key).toMatch(/extreme_squeeze/);
    expect(stressBadge(null).key).toMatch(/unknown/);
});

test('trendBadge: tiers', () => {
    expect(trendBadge([null, null, 60]).key).toMatch(/spiking/);
    expect(trendBadge([null, null, 15]).key).toMatch(/rising/);
    expect(trendBadge([null, null, 0]).key).toMatch(/steady/);
    expect(trendBadge([null, null, -15]).key).toMatch(/easing/);
    expect(trendBadge([null, null, -60]).key).toMatch(/collapsing/);
    expect(trendBadge([]).key).toMatch(/unknown/);
});

test('escalationBadge: stable / escalating / sharp_escalation / easing / sharp_relief / unknown', () => {
    expect(escalationBadge(['normal', 'normal', 'normal']).key).toMatch(/stable/);
    expect(escalationBadge(['normal', 'normal', 'tight']).key).toMatch(/escalating/);
    expect(escalationBadge(['normal', 'tight', 'extreme_squeeze']).key).toMatch(/sharp_escalation/);
    expect(escalationBadge(['tight', 'normal', 'low_available']).key).toMatch(/sharp_relief/);
    expect(escalationBadge(['tight', 'tight', 'normal']).key).toMatch(/easing/);
    expect(escalationBadge([]).key).toMatch(/unknown/);
});

test('stressDistribution: counts each level', () => {
    const d = stressDistribution(['normal', 'normal', 'tight', 'extreme_squeeze', 'low_available']);
    expect(d.low_available).toBe(1);
    expect(d.normal).toBe(2);
    expect(d.tight).toBe(1);
    expect(d.extreme_squeeze).toBe(1);
    expect(d.hard_to_borrow).toBe(0);
});

test('stressDistribution: nulls ignored', () => {
    const d = stressDistribution([null, 'normal', null, 'tight']);
    expect(d.normal).toBe(1);
    expect(d.tight).toBe(1);
});

// ── summarize ─────────────────────────────────────────────────────

test('summarizeRates: count / last / extrema / mean', () => {
    const s = summarizeRates([2, 5, 8, 3]);
    expect(s.count).toBe(4);
    expect(s.last).toBe(3);
    expect(s.min).toBe(2);
    expect(s.max).toBe(8);
    expect(s.mean).toBe(4.5);
});

test('summarizeRates: empty → NaN', () => {
    const s = summarizeRates([]);
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.last)).toBe(true);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes', () => {
    for (const k of ['normal','gradually-escalating','sudden-spike','extreme-squeeze',
                     'easy-borrow','oscillating','spike-and-relax','short-period']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.rates_pct, inp.period);
        expect(r.change_pct.length).toBe(inp.rates_pct.length);
        expect(r.stress.length).toBe(inp.rates_pct.length);
    }
});

test('demo extreme-squeeze ends in hard-to-borrow or extreme', () => {
    const inp = makeDemoInput('extreme-squeeze');
    const r = localCompute(inp.rates_pct, inp.period);
    const last = r.stress[r.stress.length - 1];
    expect(['hard_to_borrow', 'extreme_squeeze']).toContain(last);
});

test('demo easy-borrow stays in low_available', () => {
    const inp = makeDemoInput('easy-borrow');
    const r = localCompute(inp.rates_pct, inp.period);
    for (let i = 5; i < r.stress.length; i++) {
        expect(r.stress[i]).toBe('low_available');
    }
});

test('demo short-period uses period=2', () => {
    const inp = makeDemoInput('short-period');
    expect(inp.period).toBe(2);
});

// ── formatters ────────────────────────────────────────────────────

test('ratesToBlob round-trips', () => {
    const r = [2.5, 3, 2.8];
    const back = parseRatesBlob(ratesToBlob(r));
    expect(back.errors).toEqual([]);
    expect(back.rates_pct).toEqual(r);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtPct(2.5)).toBe('2.50%');
    expect(fmtPctSigned(2.5)).toBe('+2.50%');
    expect(fmtPctSigned(-2.5)).toBe('-2.50%');
    expect(fmtNum(1.23456)).toBe('1.23');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtPct(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(DEFAULT_INPUTS.rates_pct).toEqual([]);
    expect(DEFAULT_INPUTS.period).toBe(DEFAULT_PERIOD);
    expect(DEFAULT_PERIOD).toBe(5);
    expect(MIN_PERIOD).toBe(1);
    expect(MAX_PERIOD).toBe(500);
    expect(STRESS_LEVELS).toEqual(['low_available', 'normal', 'tight', 'hard_to_borrow', 'extreme_squeeze']);
});
