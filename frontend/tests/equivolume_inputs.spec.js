// Equivolume bars helpers: parser, validator, body shape,
// localCompute Rust-mirror, badges, summarize, demos.

import { test, expect } from 'vitest';
import {
    KINDS, parseBarsBlob, barsToBlob, validateInputs, buildBody, localCompute,
    summarize, convictionBadge, lastBadge,
    makeDemoInput,
    fmtUSD, fmtNum, fmtInt, fmtVol, kindLabelKey, kindCls,
} from '../js/_equivolume_inputs.js';

const b = (h, l, v) => ({ high: h, low: l, volume: v });

// ── constants ─────────────────────────────────────────────────────

test('KINDS exposes the four Rust enum strings', () => {
    expect(KINDS).toEqual(['normal', 'narrow', 'wide', 'power']);
});

// ── parser ────────────────────────────────────────────────────────

test('parseBarsBlob: 3 tokens per line, comments + blanks ignored', () => {
    const r = parseBarsBlob('101 99 1000\n# spike\n102, 100, 1500');
    expect(r.errors).toEqual([]);
    expect(r.bars).toEqual([b(101, 99, 1000), b(102, 100, 1500)]);
});

test('parseBarsBlob: rejects wrong count / high<low / negative volume', () => {
    expect(parseBarsBlob('101 99').errors[0].message).toMatch(/3 tokens/);
    expect(parseBarsBlob('98 100 1000').errors[0].message).toMatch(/high < low/);
    expect(parseBarsBlob('101 99 -50').errors[0].message).toMatch(/volume/);
});

test('parseBarsBlob: non-string returns 1 error', () => {
    expect(parseBarsBlob(null).errors.length).toBe(1);
});

// ── validator ─────────────────────────────────────────────────────

test('validate accepts default-shape', () => {
    expect(validateInputs({ bars: [b(101, 99, 1000)], total_width: 1000 })).toBe(null);
});

test('validate rejects: bad array / NaN / high<low / negative vol / bad total_width', () => {
    expect(validateInputs({ bars: 'no', total_width: 1000 })).toMatch(/bars/);
    expect(validateInputs({ bars: [b(NaN, 99, 1000)], total_width: 1000 })).toMatch(/high/);
    expect(validateInputs({ bars: [b(98, 100, 1000)], total_width: 1000 })).toMatch(/high/);
    expect(validateInputs({ bars: [b(101, 99, -1)], total_width: 1000 })).toMatch(/volume/);
    expect(validateInputs({ bars: [b(101, 99, 1000)], total_width: 0 })).toMatch(/total_width/);
});

// ── buildBody ─────────────────────────────────────────────────────

test('buildBody: strips extras + preserves total_width', () => {
    const body = buildBody({ bars: [{ ...b(101, 99, 1000), extra: 'x' }], total_width: 500 });
    expect(body).toEqual({ bars: [b(101, 99, 1000)], total_width: 500 });
});

// ── localCompute parity (mirrors every Rust #[test]) ─────────────

test('local: empty input returns empty arrays', () => {
    const r = localCompute([], 100);
    expect(r.widths).toEqual([]);
    expect(r.kinds).toEqual([]);
});

test('local: total_width=0 → all-zero widths', () => {
    const r = localCompute(Array.from({ length: 5 }, () => b(101, 99, 1000)), 0);
    expect(r.widths.every(w => w === 0)).toBe(true);
});

test('local: NaN in bars → all-zero widths', () => {
    const r = localCompute([b(101, 99, 1000), b(NaN, 99, 1000)], 100);
    expect(r.widths.every(w => w === 0)).toBe(true);
});

test('local: widths sum to total_width', () => {
    const r = localCompute(Array.from({ length: 10 }, () => b(101, 99, 1000)), 100);
    const sum = r.widths.reduce((s, w) => s + w, 0);
    expect(sum).toBeCloseTo(100, 6);
});

test('local: proportional widths (1k vs 3k → 10/40 vs 30/40 of total_width 40)', () => {
    const r = localCompute([b(101, 99, 1000), b(101, 99, 3000)], 40);
    expect(r.widths[0]).toBeCloseTo(10, 6);
    expect(r.widths[1]).toBeCloseTo(30, 6);
});

test('local: high-volume bar classified Wide or Power', () => {
    const bars = Array.from({ length: 9 }, () => b(101, 99, 1000));
    bars.push(b(115, 95, 5000));    // 5× vol AND wide range
    const r = localCompute(bars, 100);
    expect(['wide', 'power']).toContain(r.kinds[9]);
});

test('local: low-volume bar classified Narrow', () => {
    const bars = Array.from({ length: 9 }, () => b(101, 99, 2000));
    bars.push(b(101, 99, 100));
    const r = localCompute(bars, 100);
    expect(r.kinds[9]).toBe('narrow');
});

test('local: output lengths match input', () => {
    const r = localCompute(Array.from({ length: 10 }, () => b(101, 99, 1000)), 100);
    expect(r.widths.length).toBe(10);
    expect(r.kinds.length).toBe(10);
});

test('local: avg_volume + avg_range reported correctly', () => {
    const r = localCompute([b(102, 100, 1000), b(105, 100, 3000)], 100);
    // avg vol = (1000 + 3000)/2 = 2000. avg range = (2 + 5)/2 = 3.5.
    expect(r.avg_volume).toBeCloseTo(2000, 9);
    expect(r.avg_range).toBeCloseTo(3.5, 9);
});

test('local: total_vol = 0 → all-zero widths even with valid bars', () => {
    const r = localCompute(Array.from({ length: 5 }, () => b(101, 99, 0)), 100);
    expect(r.widths.every(w => w === 0)).toBe(true);
});

test('local: Power requires both big_vol AND big_range (not just one)', () => {
    // 9 normal + 1 with 5× vol but SAME range as baseline → Wide, not Power.
    const bars = Array.from({ length: 9 }, () => b(101, 99, 1000));
    bars.push(b(101, 99, 5000));
    const r = localCompute(bars, 100);
    expect(r.kinds[9]).toBe('wide');
});

// ── summarize ─────────────────────────────────────────────────────

test('summarize: tallies kinds + extrema', () => {
    const report = {
        widths: [10, 30, 60],
        kinds: ['narrow', 'normal', 'power'],
        avg_volume: 1000, avg_range: 2, total_width: 100,
    };
    const s = summarize(report);
    expect(s.count).toBe(3);
    expect(s.narrow).toBe(1);
    expect(s.normal).toBe(1);
    expect(s.power).toBe(1);
    expect(s.max_width).toBe(60);
    expect(s.min_width).toBe(10);
});

test('summarize: empty → count 0, NaN extrema', () => {
    const s = summarize({ widths: [], kinds: [] });
    expect(s.count).toBe(0);
    expect(Number.isNaN(s.max_width)).toBe(true);
});

// ── convictionBadge / lastBadge ──────────────────────────────────

test('convictionBadge: power_run > heavy > normal > quiet', () => {
    // 10 bars, 2 power → power/count = 0.20 ≥ 0.10 → power_run
    expect(convictionBadge({ count: 10, narrow: 0, normal: 8, wide: 0, power: 2 }).key).toMatch(/power_run/);
    // 10 bars, 0 power, 4 wide → wide/count = 0.40 → heavy
    expect(convictionBadge({ count: 10, narrow: 0, normal: 6, wide: 4, power: 0 }).key).toMatch(/heavy/);
    // 10 bars, 0 power, 2 wide → 0.20 → normal (between 10% and 30%)
    expect(convictionBadge({ count: 10, narrow: 0, normal: 8, wide: 2, power: 0 }).key).toMatch(/normal/);
    expect(convictionBadge({ count: 10, narrow: 0, normal: 10, wide: 0, power: 0 }).key).toMatch(/quiet/);
    expect(convictionBadge(null).key).toMatch(/unknown/);
});

test('lastBadge: maps each kind to its key', () => {
    expect(lastBadge('power').key).toMatch(/power/);
    expect(lastBadge('wide').key).toMatch(/wide/);
    expect(lastBadge('narrow').key).toMatch(/narrow/);
    expect(lastBadge('normal').key).toMatch(/normal/);
    expect(lastBadge(null).key).toMatch(/unknown/);
});

// ── demos ─────────────────────────────────────────────────────────

test('demos: each preset validates + produces a non-empty kinds array', () => {
    for (const k of ['normal-mix','power-spike','wide-only','narrow-spike',
                     'flat-volume','climax-day','mixed-kinds','noisy-walk']) {
        const inp = makeDemoInput(k);
        expect(validateInputs(inp)).toBe(null);
        const r = localCompute(inp.bars, inp.total_width);
        expect(r.kinds.length).toBe(inp.bars.length);
    }
});

test('demo power-spike: last bar is Power (high vol AND high range)', () => {
    const inp = makeDemoInput('power-spike');
    const r = localCompute(inp.bars, inp.total_width);
    expect(r.kinds[r.kinds.length - 1]).toBe('power');
});

test('demo wide-only: last bar is Wide (high vol but same range)', () => {
    const inp = makeDemoInput('wide-only');
    const r = localCompute(inp.bars, inp.total_width);
    expect(r.kinds[r.kinds.length - 1]).toBe('wide');
});

test('demo narrow-spike: last bar is Narrow', () => {
    const inp = makeDemoInput('narrow-spike');
    const r = localCompute(inp.bars, inp.total_width);
    expect(r.kinds[r.kinds.length - 1]).toBe('narrow');
});

test('demo flat-volume: every kind is Normal', () => {
    const inp = makeDemoInput('flat-volume');
    const r = localCompute(inp.bars, inp.total_width);
    for (const k of r.kinds) expect(k).toBe('normal');
});

test('demo mixed-kinds: contains at least 1 of each (narrow/normal/wide/power)', () => {
    const inp = makeDemoInput('mixed-kinds');
    const r = localCompute(inp.bars, inp.total_width);
    expect(r.kinds).toContain('narrow');
    expect(r.kinds).toContain('normal');
    expect(r.kinds).toContain('wide');
    expect(r.kinds).toContain('power');
});

// ── round-trip + label helpers ───────────────────────────────────

test('barsToBlob round-trips through parseBarsBlob', () => {
    const bars = [b(101, 99, 1000), b(102, 100, 1500)];
    const back = parseBarsBlob(barsToBlob(bars));
    expect(back.errors).toEqual([]);
    expect(back.bars).toEqual(bars);
});

test('kindLabelKey: i18n keys for all 4 + unknown', () => {
    expect(kindLabelKey('power')).toBe('view.equivol.kind.power');
    expect(kindLabelKey('wide')).toBe('view.equivol.kind.wide');
    expect(kindLabelKey('narrow')).toBe('view.equivol.kind.narrow');
    expect(kindLabelKey('normal')).toBe('view.equivol.kind.normal');
    expect(kindLabelKey()).toBe('view.equivol.kind.unknown');
});

test('kindCls: power + wide → neg; narrow/normal/unknown → empty', () => {
    expect(kindCls('power')).toBe('neg');
    expect(kindCls('wide')).toBe('neg');
    expect(kindCls('narrow')).toBe('');
    expect(kindCls('normal')).toBe('');
});

// ── formatters ────────────────────────────────────────────────────

test('fmt helpers + non-finite guards', () => {
    expect(fmtUSD(100.5)).toBe('$100.50');
    expect(fmtNum(1.234, 1)).toBe('1.2');
    expect(fmtInt(42.9)).toBe('42');
    expect(fmtVol(1_500_000)).toBe('1.50M');
    expect(fmtVol(15_500)).toBe('15.50k');
    expect(fmtVol(42)).toBe('42');
    expect(fmtUSD(NaN)).toBe('—');
});
