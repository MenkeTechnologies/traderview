// Bartlett's variance test helpers: parser, validator, localTest parity, badges.

import { test, expect } from 'vitest';
import {
    DEFAULT_INPUTS, MIN_GROUPS, MIN_PER_GROUP,
    parseGroupsBlob, groupsToBlob,
    validateInputs, buildBody, localTest,
    chiSquaredUpperTail, chiSquared5pctCritical, standardNormalCdf, erf,
    verdictBadge, ratioBadge, groupStats,
    makeDemoInput,
    fmtNum, fmtNumSigned, fmtPVal, fmtInt,
} from '../js/_bartlett_inputs.js';

// ── parser ────────────────────────────────────────────────────────

test('parseGroupsBlob: LABEL v1 v2 v3 (≥ 2 obs)', () => {
    const r = parseGroupsBlob('A 1 2 3 4\n# midline\nB 5, 6, 7, 8');
    expect(r.errors).toEqual([]);
    expect(r.labels).toEqual(['A', 'B']);
    expect(r.groups).toEqual([[1, 2, 3, 4], [5, 6, 7, 8]]);
});

test('parseGroupsBlob: rejects line with < 2 obs', () => {
    expect(parseGroupsBlob('A 1').errors[0].message).toMatch(/LABEL/);
});

test('parseGroupsBlob: rejects non-finite token', () => {
    expect(parseGroupsBlob('A 1 zzz 3').errors[0].message).toMatch(/finite/);
});

test('parseGroupsBlob: non-string returns 1 error', () => {
    expect(parseGroupsBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts valid', () => {
    expect(validateInputs({ groups: [[1, 2, 3], [4, 5, 6]] })).toBe(null);
});

test('validate rejects: bad array / < 2 groups / short group / non-finite / total ≤ k', () => {
    expect(validateInputs({ groups: 'no' })).toMatch(/groups/);
    expect(validateInputs({ groups: [[1, 2, 3]] })).toMatch(/2 groups/);
    expect(validateInputs({ groups: [[1, 2, 3], [4]] })).toMatch(/2 obs/);
    expect(validateInputs({ groups: [[1, NaN, 3], [4, 5, 6]] })).toMatch(/finite/);
    expect(validateInputs({ groups: [[1, 2], [3, 4]] })).toBe(null);  // total 4 > k 2 ✓
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody copies (defensive slice)', () => {
    const inp = { groups: [[1, 2], [3, 4]] };
    const body = buildBody(inp);
    expect(body).toEqual({ groups: [[1, 2], [3, 4]] });
    expect(body.groups[0]).not.toBe(inp.groups[0]);
});

// ── chi² helpers ──────────────────────────────────────────────────

test('chiSquared5pctCritical: hardcoded 1..5 match table', () => {
    expect(chiSquared5pctCritical(1)).toBe(3.841);
    expect(chiSquared5pctCritical(2)).toBe(5.991);
    expect(chiSquared5pctCritical(5)).toBe(11.070);
});

test('chiSquared5pctCritical: Wilson-Hilferty fallback for k > 5', () => {
    expect(chiSquared5pctCritical(10)).toBeGreaterThan(10);
    expect(chiSquared5pctCritical(10)).toBeLessThan(20);
});

test('chiSquaredUpperTail: monotone decreasing in x', () => {
    expect(chiSquaredUpperTail(1, 2)).toBeGreaterThan(chiSquaredUpperTail(5, 2));
    expect(chiSquaredUpperTail(5, 2)).toBeGreaterThan(chiSquaredUpperTail(10, 2));
});

test('chiSquaredUpperTail: returns 1 for x ≤ 0', () => {
    expect(chiSquaredUpperTail(0, 2)).toBe(1);
    expect(chiSquaredUpperTail(-1, 2)).toBe(1);
});

test('erf sanity', () => {
    expect(erf(0)).toBeCloseTo(0, 6);
    expect(erf(1)).toBeCloseTo(0.8427, 3);
});

test('standardNormalCdf(0) = 0.5', () => {
    expect(standardNormalCdf(0)).toBeCloseTo(0.5, 6);
});

// ── localTest parity (mirrors every Rust #[test]) ────────────────

test('local: too-few or small groups → null', () => {
    expect(localTest([[1, 2, 3]])).toBe(null);
    expect(localTest([[1], [2, 3]])).toBe(null);
});

test('local: NaN returns null', () => {
    expect(localTest([[1, NaN, 3], [1, 2, 3]])).toBe(null);
});

function boxMuller(n, seed, scale) {
    let state = BigInt(seed);
    const MASK = 0xFFFFFFFFFFFFFFFFn;
    const out = [];
    for (let i = 0; i < n; i++) {
        state = (state * 6364136223846793005n + 1442695040888963407n) & MASK;
        const u1 = Math.max(1e-12, Number(state >> 32n) / 0xFFFFFFFF);
        state = (state * 6364136223846793005n + 1442695040888963407n) & MASK;
        const u2 = Number(state >> 32n) / 0xFFFFFFFF;
        out.push(scale * Math.sqrt(-2 * Math.log(u1)) * Math.cos(2 * Math.PI * u2));
    }
    return out;
}

test('local: equal variance does not reject at 5%', () => {
    const g1 = boxMuller(200, 42, 1.0);
    const g2 = boxMuller(200, 13, 1.0);
    const r = localTest([g1, g2]);
    expect(r).not.toBe(null);
    expect(r.reject_at_5pct).toBe(false);
});

test('local: 5x variance difference rejects at 5%', () => {
    const g1 = boxMuller(200, 42, 1.0);
    const g2 = boxMuller(200, 13, 5.0);
    const r = localTest([g1, g2]);
    expect(r.reject_at_5pct).toBe(true);
});

test('local: 3 groups supported with correct meta', () => {
    const g1 = boxMuller(100, 1, 1.0);
    const g2 = boxMuller(100, 2, 1.0);
    const g3 = boxMuller(100, 3, 1.0);
    const r = localTest([g1, g2, g3]);
    expect(r.n_groups).toBe(3);
    expect(r.n_total).toBe(300);
    expect(r.degrees_of_freedom).toBe(2);
});

test('local: p_value in [0, 1]', () => {
    const g1 = boxMuller(50, 1, 1.0);
    const g2 = boxMuller(50, 2, 1.0);
    const r = localTest([g1, g2]);
    expect(r.p_value).toBeGreaterThanOrEqual(0);
    expect(r.p_value).toBeLessThanOrEqual(1);
});

test('local: zero-variance group returns null (log(0))', () => {
    expect(localTest([[1, 1, 1, 1], [1, 2, 3, 4]])).toBe(null);
});

test('local: deterministic for same input', () => {
    const g1 = [1.0, 2.0, 3.0, 4.0, 5.0];
    const g2 = [2.0, 3.0, 4.0, 5.0, 6.0];
    expect(localTest([g1, g2])).toEqual(localTest([g1, g2]));
});

// ── badges ────────────────────────────────────────────────────────

test('verdictBadge: tiers', () => {
    const mk = (p) => ({ p_value: p, chi_squared_statistic: 5, degrees_of_freedom: 1,
                          pooled_variance: 1, n_groups: 2, n_total: 100, reject_at_5pct: p < 0.05 });
    expect(verdictBadge(mk(0.005)).key).toMatch(/strong_reject/);
    expect(verdictBadge(mk(0.03)).key).toMatch(/reject/);
    expect(verdictBadge(mk(0.07)).key).toMatch(/borderline/);
    expect(verdictBadge(mk(0.5)).key).toMatch(/equal_variance/);
    expect(verdictBadge(null).key).toMatch(/unknown/);
});

test('ratioBadge: tiers', () => {
    // ratios 1.0, 1.5, 3, 6, 20
    expect(ratioBadge([[1, 2, 3, 4, 5], [1, 2, 3, 4, 5]]).key).toMatch(/tiny/);
    expect(ratioBadge([[1, 2, 3, 4, 5], [1, 2, 3, 4, 7]]).key).toMatch(/mild|moderate/);
    expect(ratioBadge([[1, 2, 3], [1, 5, 10]]).key).toMatch(/large|moderate|severe/);
    expect(ratioBadge([[1, 1.0001, 0.9999], [1, 100, 200]]).key).toMatch(/severe/);
    expect(ratioBadge([]).key).toMatch(/unknown/);
    expect(ratioBadge([[1, 1, 1], [2, 2, 2]]).key).toMatch(/unknown/);  // zero-variance → null
});

// ── groupStats ────────────────────────────────────────────────────

test('groupStats: per-group n / mean / sd / extrema', () => {
    const s = groupStats([[1, 2, 3, 4, 5]], ['G']);
    expect(s[0].n).toBe(5);
    expect(s[0].mean).toBe(3);
    expect(s[0].variance).toBeCloseTo(2.5, 9);
    expect(s[0].sd).toBeCloseTo(Math.sqrt(2.5), 9);
    expect(s[0].min).toBe(1);
    expect(s[0].max).toBe(5);
});

test('groupStats: empty group → 0 / NaN', () => {
    const s = groupStats([[]], ['E']);
    expect(s[0].n).toBe(0);
    expect(Number.isNaN(s[0].mean)).toBe(true);
});

test('groupStats: default labels G1, G2 if missing', () => {
    const s = groupStats([[1, 2], [3, 4]]);
    expect(s[0].label).toBe('G1');
    expect(s[1].label).toBe('G2');
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + tests (or null for non-finite cases)', () => {
    for (const k of ['equal','mild-diff','strong-diff','three-equal','three-mixed',
                     'four-volregime','small-groups','asymmetric-sizes']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localTest(inp.groups);
        expect(r).not.toBe(null);
        expect(r.n_groups).toBe(inp.groups.length);
    }
});

test('demo strong-diff rejects at 5%', () => {
    const inp = makeDemoInput('strong-diff');
    const r = localTest(inp.groups);
    expect(r.reject_at_5pct).toBe(true);
});

test('demo equal does not reject at 5%', () => {
    const inp = makeDemoInput('equal');
    const r = localTest(inp.groups);
    expect(r.reject_at_5pct).toBe(false);
});

test('demo four-volregime: n_groups = 4, degrees_of_freedom = 3', () => {
    const inp = makeDemoInput('four-volregime');
    const r = localTest(inp.groups);
    expect(r.n_groups).toBe(4);
    expect(r.degrees_of_freedom).toBe(3);
});

// ── formatters ────────────────────────────────────────────────────

test('groupsToBlob round-trips through parseGroupsBlob', () => {
    const groups = [[1, 2, 3], [4, 5, 6]];
    const labels = ['A', 'B'];
    const back = parseGroupsBlob(groupsToBlob(groups, labels));
    expect(back.errors).toEqual([]);
    expect(back.groups).toEqual(groups);
    expect(back.labels).toEqual(labels);
});

test('groupsToBlob: default labels when missing', () => {
    const blob = groupsToBlob([[1, 2], [3, 4]]);
    expect(blob).toContain('G1');
    expect(blob).toContain('G2');
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtNum(1.23456)).toBe('1.2346');
    expect(fmtNumSigned(1.5)).toBe('+1.5000');
    expect(fmtNumSigned(-1.5)).toBe('-1.5000');
    expect(fmtPVal(0.000001)).toBe('< 0.0001');
    expect(fmtPVal(0.04)).toBe('0.0400');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtNum(NaN)).toBe('—');
});

test('DEFAULTS sanity', () => {
    expect(DEFAULT_INPUTS.groups).toEqual([]);
    expect(MIN_GROUPS).toBe(2);
    expect(MIN_PER_GROUP).toBe(2);
});
