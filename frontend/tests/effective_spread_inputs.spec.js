// Effective+realized spread helpers: parser, validator, body shape,
// localAnalyze Rust-mirror, badges, demos.

import { test, expect } from 'vitest';
import {
    DIRECTIONS, DEFAULT_INPUTS,
    parseObsBlob, obsToBlob, validateInputs, buildBody, localAnalyze,
    executionBadge, adverseBadge, enrich,
    makeDemoInput,
    fmtUSD, fmtUSDSigned, fmtBps, fmtRatio, fmtInt, dirLabelKey,
} from '../js/_effective_spread_inputs.js';

const o = (trade, mid, dmid, qs, dir) => ({
    trade_price: trade, current_mid: mid, delayed_mid: dmid,
    quoted_spread: qs, direction: dir,
});

// ── constants ─────────────────────────────────────────────────────

test('DIRECTIONS = snake_case Rust enum strings', () => {
    expect(DIRECTIONS).toEqual(['buy', 'sell']);
});

// ── parser ────────────────────────────────────────────────────────

test('parseObsBlob: 5 tokens per line; blanks + comments ignored', () => {
    const r = parseObsBlob('100.05 100.00 100.00 0.10 buy\n# trade 2\n99.95, 100.00, 100.00, 0.10, sell');
    expect(r.errors).toEqual([]);
    expect(r.observations).toEqual([
        o(100.05, 100.00, 100.00, 0.10, 'buy'),
        o(99.95, 100.00, 100.00, 0.10, 'sell'),
    ]);
});

test('parseObsBlob: rejects wrong token count + bad direction + non-finite', () => {
    expect(parseObsBlob('100.05 100.00 100.00 0.10').errors[0].message).toMatch(/5 tokens/);
    expect(parseObsBlob('100.05 100.00 100.00 0.10 hold').errors[0].message).toMatch(/direction/);
    expect(parseObsBlob('foo 100.00 100.00 0.10 buy').errors[0].message).toMatch(/non-finite/);
});

test('parseObsBlob: non-string returns 1 error', () => {
    expect(parseObsBlob(null).errors.length).toBe(1);
});

test('parseObsBlob: direction case-insensitive', () => {
    expect(parseObsBlob('100.05 100.00 100.00 0.10 BUY').observations[0].direction).toBe('buy');
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts non-empty observations', () => {
    expect(validateInputs({ observations: [o(100.05, 100, 100, 0.1, 'buy')] })).toBe(null);
});

test('validate rejects: bad array / empty / bad fields', () => {
    expect(validateInputs({ observations: 'no' })).toMatch(/observations/);
    expect(validateInputs({ observations: [] })).toMatch(/non-empty/);
    expect(validateInputs({ observations: [{ trade_price: 100, current_mid: 100, delayed_mid: 100, quoted_spread: 0.1, direction: 'hold' }] }))
        .toMatch(/direction/);
    expect(validateInputs({ observations: [{ trade_price: NaN, current_mid: 100, delayed_mid: 100, quoted_spread: 0.1, direction: 'buy' }] }))
        .toMatch(/trade_price/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: emits plain JSON-shaped observations (strips extras)', () => {
    const body = buildBody({ observations: [{ ...o(100.05, 100, 100, 0.1, 'buy'), extra: 'x' }] });
    expect(body.observations[0]).toEqual({
        trade_price: 100.05, current_mid: 100, delayed_mid: 100, quoted_spread: 0.1, direction: 'buy',
    });
});

// ── localAnalyze parity (mirrors every Rust #[test]) ─────────────

test('local: empty → null', () => {
    expect(localAnalyze([])).toBeNull();
});

test('local: all-invalid rows filtered → null', () => {
    const bad = [
        o(0, 100, 100, 0.1, 'buy'),       // bad trade_price
        o(100.10, -1, 100, 0.1, 'buy'),   // bad mid
        o(100.10, 100, NaN, 0.1, 'buy'),  // bad delayed_mid
    ];
    expect(localAnalyze(bad)).toBeNull();
});

test('local: buy at ask → effective = quoted spread', () => {
    const r = localAnalyze([o(100.05, 100.00, 100.00, 0.10, 'buy')]);
    expect(r.avg_effective_spread).toBeCloseTo(0.10, 9);
});

test('local: sell at bid → effective positive (sign flips)', () => {
    const r = localAnalyze([o(99.95, 100.00, 100.00, 0.10, 'sell')]);
    expect(r.avg_effective_spread).toBeCloseTo(0.10, 9);
});

test('local: no adverse selection → realized == effective, impact == 0', () => {
    const r = localAnalyze([
        o(100.05, 100.00, 100.00, 0.10, 'buy'),
        o(99.95,  100.00, 100.00, 0.10, 'sell'),
    ]);
    expect(r.avg_realized_spread).toBeCloseTo(r.avg_effective_spread, 9);
    expect(Math.abs(r.avg_price_impact)).toBeLessThan(1e-9);
});

test('local: adverse selection (informed buy) → positive price impact', () => {
    const r = localAnalyze([o(100.05, 100.00, 100.10, 0.10, 'buy')]);
    expect(r.avg_realized_spread).toBeCloseTo(-0.10, 9);
    expect(r.avg_price_impact).toBeGreaterThan(0);
});

test('local: at-quote trades → effective/quoted ratio ≈ 1', () => {
    const r = localAnalyze([
        o(100.05, 100.00, 100.00, 0.10, 'buy'),
        o(99.95,  100.00, 100.00, 0.10, 'sell'),
    ]);
    expect(r.effective_to_quoted_ratio).toBeCloseTo(1, 9);
});

test('local: inside-quote trade (price improvement) → ratio < 1', () => {
    const r = localAnalyze([o(100.02, 100.00, 100.00, 0.10, 'buy')]);
    expect(r.effective_to_quoted_ratio).toBeLessThan(1);
});

test('local: trade-through (beyond ask) → ratio > 1', () => {
    const r = localAnalyze([o(100.10, 100.00, 100.00, 0.10, 'buy')]);
    expect(r.effective_to_quoted_ratio).toBeGreaterThan(1);
});

test('local: n_observations counts only valid rows', () => {
    const r = localAnalyze([
        o(100.05, 100.00, 100.00, 0.10, 'buy'),
        o(0,      100.00, 100.00, 0.10, 'buy'),       // filtered
        o(99.95,  100.00, 100.00, 0.10, 'sell'),
    ]);
    expect(r.n_observations).toBe(2);
});

test('local: zero quoted_spread → ratio is NaN (avg_q guard)', () => {
    const r = localAnalyze([
        o(100, 100, 100, 0, 'buy'),
        o(100, 100, 100, 0, 'sell'),
    ]);
    expect(Number.isNaN(r.effective_to_quoted_ratio)).toBe(true);
});

test('local: LP wins case (mid drifts opposite to direction) → negative price impact', () => {
    // Buy then mid drops → realized > effective → impact = eff − real < 0.
    const r = localAnalyze([o(100.05, 100.00, 99.95, 0.10, 'buy')]);
    expect(r.avg_price_impact).toBeLessThan(0);
});

// ── executionBadge ───────────────────────────────────────────────

test('executionBadge: 5-tier on effective/quoted ratio', () => {
    const mk = (r) => ({ effective_to_quoted_ratio: r });
    expect(executionBadge(mk(0.3)).key).toMatch(/great_improvement/);
    expect(executionBadge(mk(0.7)).key).toMatch(/improvement/);
    expect(executionBadge(mk(1.0)).key).toMatch(/at_quote/);
    expect(executionBadge(mk(1.2)).key).toMatch(/adverse/);
    expect(executionBadge(mk(2.0)).key).toMatch(/trade_through/);
    expect(executionBadge(null).key).toMatch(/unknown/);
});

// ── adverseBadge ──────────────────────────────────────────────────

test('adverseBadge: 5-tier on impact/effective ratio', () => {
    const mk = (i, e) => ({ avg_price_impact: i, avg_effective_spread: e });
    expect(adverseBadge(mk(-0.01, 0.10)).key).toMatch(/lp_wins/);
    expect(adverseBadge(mk(0.01,  0.10)).key).toMatch(/low/);
    expect(adverseBadge(mk(0.04,  0.10)).key).toMatch(/moderate/);
    expect(adverseBadge(mk(0.08,  0.10)).key).toMatch(/high/);
    expect(adverseBadge(mk(0.15,  0.10)).key).toMatch(/extreme/);
    expect(adverseBadge(null).key).toMatch(/unknown/);
});

// ── enrich ────────────────────────────────────────────────────────

test('enrich: adds effective/realized/impact to an observation', () => {
    const e = enrich(o(100.05, 100.00, 100.10, 0.10, 'buy'));
    expect(e.effective_spread).toBeCloseTo(0.10, 9);
    expect(e.realized_spread).toBeCloseTo(-0.10, 9);
    expect(e.price_impact).toBeCloseTo(0.20, 9);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + analyzes to a non-null report', () => {
    for (const k of ['at-quote','price-improvement','adverse-selection','lp-wins',
                     'trade-through','mixed-quality','tight-market','large-tick']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localAnalyze(inp.observations);
        expect(r).not.toBeNull();
        expect(r.n_observations).toBe(inp.observations.length);
    }
});

test('demo at-quote: effective/quoted ratio = 1.0', () => {
    const r = localAnalyze(makeDemoInput('at-quote').observations);
    expect(r.effective_to_quoted_ratio).toBeCloseTo(1, 9);
});

test('demo price-improvement: effective/quoted ratio < 1', () => {
    const r = localAnalyze(makeDemoInput('price-improvement').observations);
    expect(r.effective_to_quoted_ratio).toBeLessThan(1);
});

test('demo trade-through: effective/quoted ratio > 1', () => {
    const r = localAnalyze(makeDemoInput('trade-through').observations);
    expect(r.effective_to_quoted_ratio).toBeGreaterThan(1);
});

test('demo adverse-selection: avg_price_impact > 0', () => {
    const r = localAnalyze(makeDemoInput('adverse-selection').observations);
    expect(r.avg_price_impact).toBeGreaterThan(0);
});

test('demo lp-wins: avg_price_impact < 0', () => {
    const r = localAnalyze(makeDemoInput('lp-wins').observations);
    expect(r.avg_price_impact).toBeLessThan(0);
});

// ── round-trip + formatters ───────────────────────────────────────

test('obsToBlob round-trips through parseObsBlob', () => {
    const obs = [o(100.05, 100, 100, 0.10, 'buy'), o(99.95, 100, 100, 0.10, 'sell')];
    const back = parseObsBlob(obsToBlob(obs));
    expect(back.errors).toEqual([]);
    expect(back.observations).toEqual(obs);
});

test('dirLabelKey: i18n keys for buy/sell/unknown', () => {
    expect(dirLabelKey('buy')).toBe('view.eff_spread.dir.buy');
    expect(dirLabelKey('sell')).toBe('view.eff_spread.dir.sell');
    expect(dirLabelKey()).toBe('view.eff_spread.dir.unknown');
});

test('fmt helpers + non-finite guards', () => {
    expect(fmtUSD(0.1234)).toBe('$0.1234');
    expect(fmtUSDSigned(0.05)).toBe('+$0.0500');
    expect(fmtUSDSigned(-0.05)).toBe('-$0.0500');
    expect(fmtBps(0.10, 100)).toBe('10.00 bps');
    expect(fmtRatio(0.7345)).toBe('0.735');    // toFixed-safe (avoid x.x5 IEEE tie)
    expect(fmtInt(7.9)).toBe('7');
    expect(fmtUSD(NaN)).toBe('—');
    expect(fmtBps(NaN, 100)).toBe('—');
});
