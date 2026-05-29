// ADF stationarity test helpers: parser, validator, body shape,
// localTest Rust-mirror, badges, demos.

import { test, expect } from 'vitest';
import {
    DEFAULT_LAGS, CRIT_1PCT, CRIT_5PCT, CRIT_10PCT, SIGNIFICANCES,
    parseSeriesBlob, seriesToBlob, validateInputs, buildBody, localTest,
    significanceBadge, strengthBadge, significanceLabelKey,
    makeDemoInput,
    fmtNum, fmtT, fmtInt,
} from '../js/_adf_test_inputs.js';

// ── constants ─────────────────────────────────────────────────────

test('Critical values match Fuller (1976) large-sample asymptotics', () => {
    expect(CRIT_1PCT).toBe(-3.43);
    expect(CRIT_5PCT).toBe(-2.86);
    expect(CRIT_10PCT).toBe(-2.57);
});

test('SIGNIFICANCES exposes snake_case Rust enum strings', () => {
    expect(SIGNIFICANCES).toEqual(['pct1', 'pct5', 'pct10', 'insignificant']);
});

// ── parser ────────────────────────────────────────────────────────

test('parseSeriesBlob: whitespace + commas; comments ignored', () => {
    const r = parseSeriesBlob('100.05, 100.10\n# midday\n99.95  100.05');
    expect(r.errors).toEqual([]);
    expect(r.series).toEqual([100.05, 100.10, 99.95, 100.05]);
});

test('parseSeriesBlob: rejects non-finite', () => {
    expect(parseSeriesBlob('100, foo').errors[0].message).toMatch(/foo/);
});

test('parseSeriesBlob: non-string returns 1 error', () => {
    expect(parseSeriesBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts ≥ 3·lags + 4 observations', () => {
    expect(validateInputs({ series: new Array(50).fill(1), lags: 1 })).toBe(null);
});

test('validate rejects: bad array / NaN / non-integer lags / negative lags / too short', () => {
    expect(validateInputs({ series: 'no', lags: 1 })).toMatch(/series/);
    expect(validateInputs({ series: [1, NaN], lags: 1 })).toMatch(/finite/);
    expect(validateInputs({ series: new Array(50).fill(1), lags: 1.5 })).toMatch(/integer/);
    expect(validateInputs({ series: new Array(50).fill(1), lags: -1 })).toMatch(/≥ 0/);
    expect(validateInputs({ series: [1, 2, 3], lags: 1 })).toMatch(/observations/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: passes through series + lags', () => {
    const b = buildBody({ series: [1, 2, 3], lags: 2 });
    expect(b).toEqual({ series: [1, 2, 3], lags: 2 });
});

// ── localTest parity (mirrors every Rust #[test]) ────────────────

test('local: too short → null', () => {
    expect(localTest(new Array(5).fill(1), 2)).toBeNull();
});

test('local: NaN → null', () => {
    const s = new Array(50).fill(1);
    s[5] = NaN;
    expect(localTest(s, 1)).toBeNull();
});

test('local: random walk fails to reject H₀ (insignificant or pct10)', () => {
    // Use the same LCG seed as Rust test.
    let state = 42n;
    const s = new Array(500).fill(0);
    for (let i = 1; i < s.length; i++) {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        const u = Number(state >> 32n) / 0xFFFFFFFF - 0.5;
        s[i] = s[i - 1] + u;
    }
    const r = localTest(s, 1);
    expect(['insignificant', 'pct10']).toContain(r.significance);
});

test('local: strongly mean-reverting (AR(1) φ=0.3) rejects H₀ at 5%', () => {
    let state = 999n;
    const s = new Array(500).fill(0);
    for (let i = 1; i < s.length; i++) {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        const u = Number(state >> 32n) / 0xFFFFFFFF - 0.5;
        s[i] = 0.3 * s[i - 1] + u;
    }
    const r = localTest(s, 2);
    expect(r.t_statistic).toBeLessThan(CRIT_5PCT);
    expect(r.significance).not.toBe('insignificant');
});

test('local: flat series → null or zero gamma_se (degenerate)', () => {
    const s = new Array(50).fill(100);
    const r = localTest(s, 1);
    expect(r === null || r.gamma_se === 0).toBe(true);
});

test('local: zero lags runs simple DF (non-augmented) without error', () => {
    let state = 7n;
    const n = 300;
    const s = new Array(n).fill(0);
    for (let i = 1; i < n; i++) {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        const u = Number(state >> 32n) / 0xFFFFFFFF - 0.5;
        s[i] = 0.5 * s[i - 1] + u;
    }
    const r = localTest(s, 0);
    expect(Number.isFinite(r.t_statistic)).toBe(true);
    expect(r.lags).toBe(0);
});

test('local: n_observations = N − lags − 1', () => {
    const r = localTest(new Array(100).fill(0).map((_, i) => 100 + i * 0.01 + Math.sin(i)), 3);
    expect(r.n_observations).toBe(100 - 3 - 1);
});

test('local: lags echoed back', () => {
    const r = localTest(new Array(100).fill(0).map((_, i) => i + Math.sin(i)), 5);
    expect(r.lags).toBe(5);
});

test('local: t_statistic = gamma / gamma_se', () => {
    let state = 123n;
    const s = new Array(300).fill(0);
    for (let i = 1; i < s.length; i++) {
        state = (state * 6364136223846793005n + 1442695040888963407n) & 0xFFFFFFFFFFFFFFFFn;
        const u = Number(state >> 32n) / 0xFFFFFFFF - 0.5;
        s[i] = 0.5 * s[i - 1] + u;
    }
    const r = localTest(s, 1);
    expect(r.t_statistic).toBeCloseTo(r.gamma / r.gamma_se, 9);
});

// ── significance / strength badges ───────────────────────────────

test('significanceBadge: pct1 / pct5 / pct10 / insignificant', () => {
    expect(significanceBadge('pct1').cls).toBe('pos');
    expect(significanceBadge('pct5').cls).toBe('pos');
    expect(significanceBadge('pct10').cls).toBe('');
    expect(significanceBadge('insignificant').cls).toBe('neg');
});

test('strengthBadge: very_strong / strong / moderate / weak / weak_trend / unit_root', () => {
    expect(strengthBadge(-6).key).toMatch(/very_strong/);
    expect(strengthBadge(-3.5).key).toMatch(/strong/);
    expect(strengthBadge(-3).key).toMatch(/moderate/);
    expect(strengthBadge(-2.6).key).toMatch(/weak/);    // between 10% and 5%
    expect(strengthBadge(-1.5).key).toMatch(/weak_trend/);
    expect(strengthBadge(0.5).key).toMatch(/unit_root/);
});

test('significanceLabelKey: maps every enum to view.adf.sig.<name>', () => {
    expect(significanceLabelKey('pct1')).toBe('view.adf.sig.pct1');
    expect(significanceLabelKey('insignificant')).toBe('view.adf.sig.insignificant');
    expect(significanceLabelKey()).toBe('view.adf.sig.unknown');
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + computes without throwing', () => {
    for (const k of ['random-walk','mean-reverting-strong','mean-reverting-weak',
                     'trend-stationary','pure-noise','high-lags','short-series','flat']) {
        const inp = makeDemoInput(k);
        // Note: flat / short-series may fail validation or return null from localTest;
        // they should still go through without runtime error.
        const err = validateInputs(inp);
        if (err === null) {
            const r = localTest(inp.series, inp.lags);
            expect(r === null || Number.isFinite(r.t_statistic)).toBe(true);
        }
    }
});

test('demo random-walk: significance ∈ {insignificant, pct10}', () => {
    const inp = makeDemoInput('random-walk');
    const r = localTest(inp.series, inp.lags);
    expect(['insignificant', 'pct10']).toContain(r.significance);
});

test('demo mean-reverting-strong: t-stat < CRIT_5PCT', () => {
    const inp = makeDemoInput('mean-reverting-strong');
    const r = localTest(inp.series, inp.lags);
    expect(r.t_statistic).toBeLessThan(CRIT_5PCT);
});

test('demo pure-noise: stationary by construction → rejects H₀', () => {
    const inp = makeDemoInput('pure-noise');
    const r = localTest(inp.series, inp.lags);
    expect(r.significance).not.toBe('insignificant');
});

test('demo high-lags: lags reported as 5', () => {
    const inp = makeDemoInput('high-lags');
    expect(inp.lags).toBe(5);
    const r = localTest(inp.series, inp.lags);
    expect(r.lags).toBe(5);
});

test('demo flat: regression is degenerate → null or zero SE', () => {
    const inp = makeDemoInput('flat');
    const r = localTest(inp.series, inp.lags);
    expect(r === null || r.gamma_se === 0).toBe(true);
});

// ── round-trip + formatters ──────────────────────────────────────

test('seriesToBlob round-trips through parseSeriesBlob', () => {
    const s = [100.05, 100.10, 99.95, 100.05];
    const back = parseSeriesBlob(seriesToBlob(s));
    expect(back.errors).toEqual([]);
    expect(back.series).toEqual(s);
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtNum(1.234567)).toBe('1.2346');
    expect(fmtT(-3.4321)).toBe('-3.432');
    expect(fmtInt(42.7)).toBe('42');
    expect(fmtNum(NaN)).toBe('—');
    expect(fmtT(NaN)).toBe('—');
});
