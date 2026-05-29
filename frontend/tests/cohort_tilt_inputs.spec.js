// Cohort-tilt helpers: parser, validator, body shape, local aggregate
// (parity with Rust ::aggregate + ::classify), bias-badge, demos.

import { test, expect } from 'vitest';
import {
    parsePositionBlob, validateInputs, buildBody, localAggregate, classify,
    biasBadge, lopsidedness, cohortLongRatio, makeDemoPositions,
    fmtPct, fmtSignedInt, symbolColor,
} from '../js/_cohort_tilt_inputs.js';

const pos = (id, sym, n) => ({ trader_id: id, symbol: sym, net_contracts: n });

// ── parser ────────────────────────────────────────────────────────

test('parser accepts 3 tokens + comments + uppercases symbol', () => {
    const r = parsePositionBlob('L0 es 3\n# comment\nS1 nq -2');
    expect(r.errors).toEqual([]);
    expect(r.positions).toEqual([pos('L0', 'ES', 3), pos('S1', 'NQ', -2)]);
});

test('parser keeps trader_id case-as-is (anonymized handle)', () => {
    expect(parsePositionBlob('Alice ES 1').positions[0].trader_id).toBe('Alice');
});

test('parser accepts 0 (flat) contracts', () => {
    expect(parsePositionBlob('a ES 0').errors).toEqual([]);
});

test('parser rejects non-integer contracts (must be ±N integer)', () => {
    expect(parsePositionBlob('a ES 1.5').errors[0].message).toMatch(/integer/);
    expect(parsePositionBlob('a ES abc').errors[0].message).toMatch(/integer/);
});

test('parser rejects wrong token count', () => {
    expect(parsePositionBlob('a ES').errors[0].message).toMatch(/3 tokens/);
});

test('parser non-string returns 1 error', () => {
    expect(parsePositionBlob(null).errors.length).toBe(1);
});

// ── validate / buildBody ──────────────────────────────────────────

test('validate rejects empty positions', () => {
    expect(validateInputs([])).toMatch(/≥ 1 position/);
});

test('buildBody wraps as { positions }', () => {
    const ps = [pos('a', 'ES', 1)];
    expect(buildBody(ps)).toEqual({ positions: ps });
});

// ── classify (Rust thresholds exactly) ────────────────────────────

test('classify: ≥ 0.75 strongly_long; 0.74 long', () => {
    expect(classify(0.75)).toBe('strongly_long');
    expect(classify(0.749)).toBe('long');
});

test('classify: ≥ 0.60 long; 0.59 balanced', () => {
    expect(classify(0.60)).toBe('long');
    expect(classify(0.599)).toBe('balanced');
});

test('classify: ≥ 0.40 balanced; 0.399 short', () => {
    expect(classify(0.40)).toBe('balanced');
    expect(classify(0.399)).toBe('short');
});

test('classify: ≥ 0.25 short; 0.249 strongly_short', () => {
    expect(classify(0.25)).toBe('short');
    expect(classify(0.249)).toBe('strongly_short');
});

test('classify: 1.0 / 0.0 / 0.5 → strongly_long / strongly_short / balanced', () => {
    expect(classify(1.0)).toBe('strongly_long');
    expect(classify(0.0)).toBe('strongly_short');
    expect(classify(0.5)).toBe('balanced');
});

test('classify: NaN / non-finite → balanced', () => {
    expect(classify(NaN)).toBe('balanced');
});

// ── localAggregate parity (one test per Rust test case + extras) ──

test('local: empty input → empty report', () => {
    expect(localAggregate([])).toEqual({
        by_symbol: [], active_traders: 0, most_lopsided_symbol: null,
    });
});

test('local: flat positions don\'t count as active traders', () => {
    const r = localAggregate([pos('a', 'ES', 0), pos('b', 'ES', 0), pos('c', 'ES', 0)]);
    expect(r.active_traders).toBe(0);
    expect(r.by_symbol[0].flat_traders).toBe(3);
    expect(r.by_symbol[0].long_ratio).toBeNull();
    expect(r.by_symbol[0].bias).toBe('balanced');
});

test('local: balanced room → 0.5 long_ratio + balanced bias', () => {
    const ps = [];
    for (let i = 0; i < 5; i++) ps.push(pos(`L${i}`, 'ES',  1));
    for (let i = 0; i < 5; i++) ps.push(pos(`S${i}`, 'ES', -1));
    const r = localAggregate(ps);
    expect(r.by_symbol[0].long_ratio).toBe(0.5);
    expect(r.by_symbol[0].bias).toBe('balanced');
});

test('local: heavy long room → strongly_long + sums net_contracts', () => {
    const ps = [];
    for (let i = 0; i < 8; i++) ps.push(pos(`L${i}`, 'ES',  3));
    for (let i = 0; i < 2; i++) ps.push(pos(`S${i}`, 'ES', -3));
    const r = localAggregate(ps);
    expect(r.by_symbol[0].long_ratio).toBe(0.8);
    expect(r.by_symbol[0].bias).toBe('strongly_long');
    expect(r.by_symbol[0].net_contracts).toBe(18); // 8*3 - 2*3
});

test('local: heavy short → strongly_short', () => {
    const r = localAggregate([
        pos('a', 'NQ',  1), pos('b', 'NQ', -2), pos('c', 'NQ', -2),
        pos('d', 'NQ', -2), pos('e', 'NQ', -2),
    ]);
    expect(r.by_symbol[0].long_ratio).toBe(0.2);
    expect(r.by_symbol[0].bias).toBe('strongly_short');
});

test('local: lopsided symbol surfaces first', () => {
    const ps = [
        pos('a', 'ES',  1), pos('b', 'ES', -1),
    ];
    for (let i = 0; i < 4; i++) ps.push(pos(`L${i}`, 'NQ',  1));
    ps.push(pos('S0', 'NQ', -1));
    const r = localAggregate(ps);
    expect(r.by_symbol[0].symbol).toBe('NQ');
    expect(r.most_lopsided_symbol).toBe('NQ');
});

test('local: same trader in two symbols counted once in active', () => {
    const r = localAggregate([pos('a', 'ES', 1), pos('a', 'NQ', -1)]);
    expect(r.active_traders).toBe(1);
});

test('local: trader counts per-symbol independent (1 long ES + 1 short NQ → both buckets non-empty)', () => {
    const r = localAggregate([pos('a', 'ES', 1), pos('a', 'NQ', -1)]);
    const es = r.by_symbol.find(s => s.symbol === 'ES');
    const nq = r.by_symbol.find(s => s.symbol === 'NQ');
    expect(es.long_traders).toBe(1);
    expect(es.short_traders).toBe(0);
    expect(nq.short_traders).toBe(1);
});

test('local: net_contracts sums all contracts (incl. zero/flat positions)', () => {
    const r = localAggregate([
        pos('a', 'ES', 5), pos('b', 'ES', -2), pos('c', 'ES', 0),
    ]);
    expect(r.by_symbol[0].net_contracts).toBe(3);
});

// ── helpers ───────────────────────────────────────────────────────

test('biasBadge maps every known bias to label + css class', () => {
    expect(biasBadge('strongly_long').cls).toBe('pos');
    expect(biasBadge('long').cls).toBe('pos');
    expect(biasBadge('balanced').cls).toBe('');
    expect(biasBadge('short').cls).toBe('neg');
    expect(biasBadge('strongly_short').cls).toBe('neg');
    expect(biasBadge('unknown').label).toBe('UNKNOWN');
});

test('lopsidedness: 0 when null; |r - 0.5| otherwise', () => {
    expect(lopsidedness(null)).toBe(0);
    expect(lopsidedness({ long_ratio: null })).toBe(0);
    expect(lopsidedness({ long_ratio: 0.8 })).toBeCloseTo(0.3, 10);
    expect(lopsidedness({ long_ratio: 0.5 })).toBe(0);
});

test('cohortLongRatio: weighted avg by positioned traders, ignores all-flat', () => {
    // ES: 8L/2S → 0.8 with 10 positioned. NQ: 5L/5S → 0.5 with 10.
    // Weighted = (0.8*10 + 0.5*10) / 20 = 0.65.
    const r = {
        by_symbol: [
            { long_traders: 8, short_traders: 2, long_ratio: 0.8 },
            { long_traders: 5, short_traders: 5, long_ratio: 0.5 },
            { long_traders: 0, short_traders: 0, long_ratio: null },
        ],
    };
    expect(cohortLongRatio(r)).toBeCloseTo(0.65, 10);
});

test('cohortLongRatio: null when nobody positioned', () => {
    expect(cohortLongRatio({ by_symbol: [{ long_traders: 0, short_traders: 0, long_ratio: null }] })).toBeNull();
    expect(cohortLongRatio(null)).toBeNull();
});

// ── demos invariants ──────────────────────────────────────────────

test('demo mixed: 4 symbols, ES strongly_long, NQ balanced, CL strongly_short, GC all-flat', () => {
    const r = localAggregate(makeDemoPositions('mixed'));
    const es = r.by_symbol.find(s => s.symbol === 'ES');
    const nq = r.by_symbol.find(s => s.symbol === 'NQ');
    const cl = r.by_symbol.find(s => s.symbol === 'CL');
    const gc = r.by_symbol.find(s => s.symbol === 'GC');
    expect(es.bias).toBe('strongly_long');
    expect(nq.bias).toBe('balanced');
    expect(cl.bias).toBe('strongly_short');
    expect(gc.long_ratio).toBeNull();
    expect(gc.flat_traders).toBe(3);
});

test('demo strongly-long: bias=strongly_long', () => {
    const r = localAggregate(makeDemoPositions('strongly-long'));
    expect(r.by_symbol[0].bias).toBe('strongly_long');
});

test('demo strongly-short: bias=strongly_short', () => {
    const r = localAggregate(makeDemoPositions('strongly-short'));
    expect(r.by_symbol[0].bias).toBe('strongly_short');
});

test('demo all-flat: 0 active traders, long_ratio null', () => {
    const r = localAggregate(makeDemoPositions('all-flat'));
    expect(r.active_traders).toBe(0);
    expect(r.by_symbol[0].long_ratio).toBeNull();
});

test('demo cross-symbol: 1 trader appears in 2 symbols, active=2 (a + b)', () => {
    const r = localAggregate(makeDemoPositions('cross-symbol'));
    expect(r.active_traders).toBe(2);
});

// ── formatters ────────────────────────────────────────────────────

test('fmtPct guards non-finite + rounds to d places', () => {
    expect(fmtPct(0.6543, 2)).toBe('65.43%');
    expect(fmtPct(null)).toBe('—');
    expect(fmtPct(NaN)).toBe('—');
});

test('fmtSignedInt: signs positive but not negative (negative gets its own sign)', () => {
    expect(fmtSignedInt(5)).toBe('+5');
    expect(fmtSignedInt(-3)).toBe('-3');
    expect(fmtSignedInt(0)).toBe('+0');
});

test('symbolColor cycles palette, neg id → muted', () => {
    expect(symbolColor(0)).toBe('#00e5ff');
    expect(symbolColor(6)).toBe('#00e5ff');
    expect(symbolColor(-1)).toBe('#aab');
});
