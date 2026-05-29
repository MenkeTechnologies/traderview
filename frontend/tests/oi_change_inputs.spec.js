// OI Change helpers: snapshot parser (5 tokens), validator, body shape,
// tier classifier, flow direction, summary, demo invariants, formatters.

import { test, expect } from 'vitest';
import {
    parseSnapshotBlob, validateInputs, buildBody,
    alertTier, flowDirection, summarize,
    makeDemoSnapshots,
    fmtN, fmtInt, fmtPct, fmtSignedInt,
} from '../js/_oi_change_inputs.js';

// ── parseSnapshotBlob ──────────────────────────────────────────────

test('parseSnapshotBlob accepts whitespace + commas + comments', () => {
    const r = parseSnapshotBlob('# header\n500 25000 6000 24000 6200\n510, 32000, 3000, 12000, 3100');
    expect(r.errors).toEqual([]);
    expect(r.snapshots).toEqual([
        { strike: 500, call_oi: 25000, put_oi: 6000, call_oi_baseline: 24000, put_oi_baseline: 6200 },
        { strike: 510, call_oi: 32000, put_oi: 3000, call_oi_baseline: 12000, put_oi_baseline: 3100 },
    ]);
});

test('parseSnapshotBlob rejects wrong token count', () => {
    expect(parseSnapshotBlob('500 1000').errors[0].message).toMatch(/expected 5 tokens/);
});

test('parseSnapshotBlob rejects non-positive strike', () => {
    expect(parseSnapshotBlob('0 100 100 100 100').errors[0].message).toMatch(/strike/);
});

test('parseSnapshotBlob rejects non-integer OI', () => {
    expect(parseSnapshotBlob('500 100.5 100 100 100').errors[0].message).toMatch(/call_oi/);
    expect(parseSnapshotBlob('500 100 -1 100 100').errors[0].message).toMatch(/put_oi/);
});

test('parseSnapshotBlob rejects negative baseline', () => {
    expect(parseSnapshotBlob('500 100 100 -1 100').errors[0].message).toMatch(/call_oi_baseline/);
    expect(parseSnapshotBlob('500 100 100 100 -1').errors[0].message).toMatch(/put_oi_baseline/);
});

test('parseSnapshotBlob accepts zero baseline (new strike with no history)', () => {
    const r = parseSnapshotBlob('500 100 100 0 0');
    expect(r.errors).toEqual([]);
    expect(r.snapshots).toHaveLength(1);
});

test('parseSnapshotBlob non-string returns 1 error', () => {
    expect(parseSnapshotBlob(null).errors.length).toBe(1);
});

// ── validateInputs / buildBody ────────────────────────────────────

const okSnap = [{ strike: 500, call_oi: 25000, put_oi: 6000, call_oi_baseline: 24000, put_oi_baseline: 6200 }];

test('validate accepts canonical', () => {
    expect(validateInputs(okSnap, 0.25, 1000)).toBe(null);
});

test('validate rejects empty snapshots', () => {
    expect(validateInputs([], 0.25, 1000)).toMatch(/at least 1 strike snapshot/);
});

test('validate rejects non-positive pct_threshold', () => {
    expect(validateInputs(okSnap, 0, 1000)).toMatch(/pct_threshold/);
    expect(validateInputs(okSnap, -0.1, 1000)).toMatch(/pct_threshold/);
});

test('validate rejects non-integer or negative min_oi', () => {
    expect(validateInputs(okSnap, 0.25, -1)).toMatch(/min_oi/);
    expect(validateInputs(okSnap, 0.25, 1.5)).toMatch(/min_oi/);
});

test('validate accepts min_oi = 0', () => {
    expect(validateInputs(okSnap, 0.25, 0)).toBe(null);
});

test('buildBody emits backend OiChangeBody shape', () => {
    expect(buildBody(okSnap, 0.25, 1000)).toEqual({
        snapshots: okSnap, pct_threshold: 0.25, min_oi: 1000,
    });
});

// ── alertTier ─────────────────────────────────────────────────────

test('alertTier escalates with pct or absolute change', () => {
    expect(alertTier({ pct_change: 0.15, abs_change: 2000 }).label).toBe('MILD');
    expect(alertTier({ pct_change: 0.30, abs_change: 6000 }).label).toBe('NOTABLE');
    expect(alertTier({ pct_change: 0.60, abs_change: 25000 }).label).toBe('STRONG');
    expect(alertTier({ pct_change: 1.20, abs_change: 60000 }).label).toBe('SURGE');
});

test('alertTier null returns em-dash', () => {
    expect(alertTier(null).label).toBe('—');
});

test('alertTier escalation respects either-or threshold (pct alone or abs alone)', () => {
    // High pct alone: 60% with tiny abs → STRONG by pct alone.
    expect(alertTier({ pct_change: 0.60, abs_change: 10 }).label).toBe('STRONG');
    // High abs alone: 60k surge on a tiny pct change → SURGE by abs alone.
    expect(alertTier({ pct_change: 0.02, abs_change: 60000 }).label).toBe('SURGE');
});

// ── flowDirection ─────────────────────────────────────────────────

test('flowDirection maps positive → BUILDING, negative → UNWIND, zero → FLAT', () => {
    expect(flowDirection(5000).label).toBe('BUILDING');
    expect(flowDirection(-5000).label).toBe('UNWIND');
    expect(flowDirection(0).label).toBe('FLAT');
    expect(flowDirection(NaN).label).toBe('FLAT');
});

test('flowDirection: BUILDING is neg-color (concern), UNWIND is pos-color (relief)', () => {
    expect(flowDirection(5000).cls).toBe('neg');
    expect(flowDirection(-5000).cls).toBe('pos');
});

// ── summarize ─────────────────────────────────────────────────────

test('summarize aggregates across both alert lists', () => {
    const r = {
        call_alerts: [
            { strike: 510, abs_change: 20000 },
            { strike: 520, abs_change: 3000 },
        ],
        put_alerts:  [
            { strike: 470, abs_change: 13000 },
        ],
    };
    const s = summarize(r);
    expect(s.totalCallAlerts).toBe(2);
    expect(s.totalPutAlerts).toBe(1);
    expect(s.netCallChange).toBe(23000);
    expect(s.netPutChange).toBe(13000);
    expect(s.maxCallStrike).toBe(510);    // first = biggest backend-sorted
    expect(s.maxPutStrike).toBe(470);
});

test('summarize handles null report', () => {
    const s = summarize(null);
    expect(s.totalCallAlerts).toBe(0);
    expect(s.maxCallStrike).toBe(null);
});

// ── makeDemoSnapshots ─────────────────────────────────────────────

test('makeDemoSnapshots returns 8 strikes with valid shape', () => {
    const snaps = makeDemoSnapshots();
    expect(snaps.length).toBe(8);
    expect(snaps.every(s =>
        s.strike > 0 &&
        Number.isInteger(s.call_oi) && s.call_oi >= 0 &&
        Number.isInteger(s.put_oi)  && s.put_oi  >= 0 &&
        s.call_oi_baseline >= 0 && s.put_oi_baseline >= 0
    )).toBe(true);
});

test('makeDemoSnapshots 510 call has ≥ 100% pct change (engineered surge)', () => {
    const s510 = makeDemoSnapshots().find(x => x.strike === 510);
    const pct = (s510.call_oi - s510.call_oi_baseline) / s510.call_oi_baseline;
    expect(pct).toBeGreaterThanOrEqual(1.0);
});

test('makeDemoSnapshots 470 put has ≥ 50% pct change (engineered hedge build)', () => {
    const s470 = makeDemoSnapshots().find(x => x.strike === 470);
    const pct = (s470.put_oi - s470.put_oi_baseline) / s470.put_oi_baseline;
    expect(pct).toBeGreaterThanOrEqual(0.5);
});

// ── formatters ────────────────────────────────────────────────────

test('formatters handle non-finite + sign correctly', () => {
    expect(fmtN(500.5)).toBe('500.50');
    expect(fmtN(NaN)).toBe('—');
    expect(fmtInt(1234567)).toBe('1,234,567');
    expect(fmtInt(NaN)).toBe('—');
    expect(fmtPct(0.234)).toBe('+23.4%');
    expect(fmtPct(-0.05)).toBe('-5.0%');
    expect(fmtSignedInt(5000)).toBe('+5,000');
    expect(fmtSignedInt(-2500)).toBe('-2,500');
});
