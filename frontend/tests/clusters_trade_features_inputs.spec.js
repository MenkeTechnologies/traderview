// Trade-feature cluster helpers: parser, validator, body shape, local
// k-means parity, points-by-cluster slicing, inertia, palette, demos.

import { test, expect } from 'vitest';
import {
    parseFeatureBlob, validateInputs, buildBody, localAnalyze,
    pointsByCluster, totalInertia, clusterColor, makeDemoFeatures,
    fmtMin, fmtR, fmtPct, fmtNum,
} from '../js/_clusters_trade_features_inputs.js';

const f = (em, hd, r) => ({ entry_minute_of_day: em, hold_duration_minutes: hd, r_multiple: r });

// ── parseFeatureBlob ──────────────────────────────────────────────

test('parseFeatureBlob accepts 3-token rows + comments', () => {
    const r = parseFeatureBlob('540 30 1.5  # morning win\n# pure comment\n900 240 -1.0');
    expect(r.errors).toEqual([]);
    expect(r.features).toEqual([f(540, 30, 1.5), f(900, 240, -1.0)]);
});

test('parseFeatureBlob rejects wrong token count', () => {
    expect(parseFeatureBlob('540 30').errors[0].message).toMatch(/expected 3 tokens/);
});

test('parseFeatureBlob rejects non-finite tokens', () => {
    expect(parseFeatureBlob('abc 30 1').errors[0].message).toMatch(/finite/);
});

test('parseFeatureBlob rejects entry_minute outside [0, 1440]', () => {
    expect(parseFeatureBlob('-1 30 1').errors[0].message).toMatch(/entry_minute/);
    expect(parseFeatureBlob('1500 30 1').errors[0].message).toMatch(/entry_minute/);
});

test('parseFeatureBlob rejects negative hold_minutes', () => {
    expect(parseFeatureBlob('540 -1 1').errors[0].message).toMatch(/hold_duration/);
});

test('parseFeatureBlob accepts boundary 0 and 1440', () => {
    const r = parseFeatureBlob('0 0 0\n1440 0 0');
    expect(r.errors).toEqual([]);
    expect(r.features.length).toBe(2);
});

test('parseFeatureBlob non-string returns 1 error', () => {
    expect(parseFeatureBlob(null).errors.length).toBe(1);
});

// ── validateInputs / buildBody ────────────────────────────────────

test('validate accepts good inputs', () => {
    expect(validateInputs([f(1, 2, 3)], 1, 10)).toBe(null);
});

test('validate rejects empty features / bad k / bad iters / k > n', () => {
    expect(validateInputs([], 1, 10)).toMatch(/≥ 1 trade/);
    expect(validateInputs([f(1, 2, 3)], 0, 10)).toMatch(/k must be/);
    expect(validateInputs([f(1, 2, 3)], 1.5, 10)).toMatch(/k must be/);
    expect(validateInputs([f(1, 2, 3)], 1, 0)).toMatch(/max_iters/);
    expect(validateInputs([f(1, 2, 3)], 5, 10)).toMatch(/cannot exceed/);
});

test('buildBody emits backend ClusterAnalysisBody shape', () => {
    expect(buildBody([f(1, 2, 3)], 1, 10)).toEqual({
        features: [f(1, 2, 3)], k: 1, max_iters: 10,
    });
});

// ── localAnalyze parity (mirror of cluster_analysis::analyze) ─────

test('local: empty features returns empty report', () => {
    expect(localAnalyze([], 3, 10)).toEqual({ assignments: [], clusters: [] });
});

test('local: k=0 returns empty report', () => {
    expect(localAnalyze([f(1, 2, 3)], 0, 10)).toEqual({ assignments: [], clusters: [] });
});

test('local: k=1 puts every trade in cluster 0', () => {
    const fs = [f(540, 30, 1), f(575, 35, 1.5), f(580, 28, 0.9)];
    const r = localAnalyze(fs, 1, 10);
    expect(r.assignments).toEqual([0, 0, 0]);
    expect(r.clusters.length).toBe(1);
    expect(r.clusters[0].size).toBe(3);
});

test('local: k=2 separates morning-winners and afternoon-losers', () => {
    const fs = [
        f(540, 30, 1.5), f(545, 25, 2.0), f(550, 35, 1.0),
        f(840, 240, -0.8), f(850, 250, -1.0), f(860, 220, -0.5),
    ];
    const r = localAnalyze(fs, 2, 20);
    expect(r.assignments[0]).toBe(r.assignments[1]);
    expect(r.assignments[1]).toBe(r.assignments[2]);
    expect(r.assignments[3]).toBe(r.assignments[4]);
    expect(r.assignments[4]).toBe(r.assignments[5]);
    expect(r.assignments[0]).not.toBe(r.assignments[3]);
});

test('local: k > n is capped to n', () => {
    const fs = [f(100, 30, 1), f(200, 60, 2)];
    const r = localAnalyze(fs, 10, 5);
    expect(r.clusters.length).toBeLessThanOrEqual(2);
});

test('local: deterministic seeding ⇒ repeated calls match', () => {
    const fs = [f(540, 30, 1.5), f(545, 25, 2.0), f(840, 240, -0.8), f(850, 250, -1.0)];
    const r1 = localAnalyze(fs, 2, 20);
    const r2 = localAnalyze(fs, 2, 20);
    expect(r1.assignments).toEqual(r2.assignments);
});

test('local: cluster sizes sum to total features', () => {
    const fs = makeDemoFeatures('three-style');
    const r = localAnalyze(fs, 3, 30);
    const sum = r.clusters.reduce((a, c) => a + c.size, 0);
    expect(sum).toBe(fs.length);
});

test('local: per-cluster mean_r equals direct avg of member r-multiples', () => {
    const fs = [f(540, 30, 1.5), f(545, 25, 2.0), f(840, 240, -0.8)];
    const r = localAnalyze(fs, 2, 20);
    for (const c of r.clusters) {
        const members = fs.filter((_, i) => r.assignments[i] === c.cluster_id);
        const expected = members.length > 0
            ? members.reduce((a, b) => a + b.r_multiple, 0) / members.length
            : 0;
        expect(c.mean_r).toBeCloseTo(expected, 10);
    }
});

test('local: win_rate counts strictly positive R only', () => {
    const fs = [f(540, 30, 1), f(540, 30, 0), f(540, 30, -1)];
    const r = localAnalyze(fs, 1, 5);
    expect(r.clusters[0].win_rate).toBeCloseTo(1 / 3, 10);
});

test('local: centroid components are mean of assigned features (after reassign)', () => {
    // Two distinct halves force assignment changes on iter 1 so the
    // centroid-update step runs (single-cluster trivial seed wouldn't).
    const fs = [f(540, 30, 1), f(560, 50, 2), f(900, 240, -1), f(910, 250, -1.2)];
    const r = localAnalyze(fs, 2, 20);
    for (const c of r.clusters) {
        const members = fs.filter((_, i) => r.assignments[i] === c.cluster_id);
        if (!members.length) continue;
        const mE = members.reduce((a, b) => a + b.entry_minute_of_day, 0) / members.length;
        const mH = members.reduce((a, b) => a + b.hold_duration_minutes, 0) / members.length;
        const mR = members.reduce((a, b) => a + b.r_multiple, 0) / members.length;
        expect(c.centroid.entry_minute).toBeCloseTo(mE, 9);
        expect(c.centroid.hold_minutes).toBeCloseTo(mH, 9);
        expect(c.centroid.r_multiple).toBeCloseTo(mR, 9);
    }
});

// ── pointsByCluster, totalInertia, palette ────────────────────────

test('pointsByCluster groups by assignment id, parallel x/y/r arrays per cluster', () => {
    const fs = [f(100, 10, 0.5), f(200, 20, -0.5)];
    const r = pointsByCluster(fs, [0, 1], 2);
    expect(r.xs[0]).toEqual([100]);
    expect(r.ys[0]).toEqual([10]);
    expect(r.rs[0]).toEqual([0.5]);
    expect(r.xs[1]).toEqual([200]);
});

test('pointsByCluster: out-of-bounds id is dropped silently', () => {
    const fs = [f(100, 10, 0.5)];
    const r = pointsByCluster(fs, [5], 2);
    expect(r.xs[0]).toEqual([]); expect(r.xs[1]).toEqual([]);
});

test('totalInertia: zero when every feature == its centroid', () => {
    const fs = [f(100, 10, 0.5)];
    const r = localAnalyze(fs, 1, 5);
    expect(totalInertia(fs, r.assignments, r.clusters)).toBeCloseTo(0, 10);
});

test('totalInertia: increases with cluster spread', () => {
    const tight  = [f(100, 10, 0.5), f(101, 10, 0.5)];
    const spread = [f(100, 10, 0.5), f(900, 240, -1.5)];
    const rT = localAnalyze(tight, 1, 5);
    const rS = localAnalyze(spread, 1, 5);
    expect(totalInertia(spread, rS.assignments, rS.clusters))
        .toBeGreaterThan(totalInertia(tight, rT.assignments, rT.clusters));
});

test('clusterColor cycles through palette, neg id returns muted color', () => {
    expect(clusterColor(0)).toBe('#00e5ff');
    expect(clusterColor(6)).toBe('#00e5ff');   // wraps after 6
    expect(clusterColor(-1)).toBe('#aab');
    expect(clusterColor(NaN)).toBe('#aab');
});

// ── demos invariants ──────────────────────────────────────────────

test('demos: each preset returns ≥ 20 features with valid ranges', () => {
    for (const kind of ['morning-vs-afternoon', 'three-style', 'single', 'scatter']) {
        const fs = makeDemoFeatures(kind);
        expect(fs.length).toBeGreaterThanOrEqual(20);
        for (const ft of fs) {
            expect(ft.entry_minute_of_day).toBeGreaterThanOrEqual(0);
            expect(ft.entry_minute_of_day).toBeLessThanOrEqual(1440);
            expect(ft.hold_duration_minutes).toBeGreaterThanOrEqual(0);
            expect(Number.isFinite(ft.r_multiple)).toBe(true);
        }
    }
});

test('demo morning-vs-afternoon: k=2 perfectly separates by index half', () => {
    const fs = makeDemoFeatures('morning-vs-afternoon');
    const r = localAnalyze(fs, 2, 30);
    const firstHalfCluster = r.assignments[0];
    const secondHalfCluster = r.assignments[12];
    expect(firstHalfCluster).not.toBe(secondHalfCluster);
    for (let i = 0; i < 12; i++) expect(r.assignments[i]).toBe(firstHalfCluster);
    for (let i = 12; i < 24; i++) expect(r.assignments[i]).toBe(secondHalfCluster);
});

test('demo three-style: k=3 produces 3 clusters with distinct mean-R signs', () => {
    const fs = makeDemoFeatures('three-style');
    const r = localAnalyze(fs, 3, 30);
    expect(r.clusters.length).toBe(3);
    const meanRs = r.clusters.map(c => c.mean_r).sort((a, b) => a - b);
    expect(meanRs[0]).toBeLessThan(0);  // losers
    expect(meanRs[2]).toBeGreaterThan(0); // winners
});

// ── formatters ────────────────────────────────────────────────────

test('fmtMin: minutes-of-day → HH:MM', () => {
    expect(fmtMin(0)).toBe('00:00');
    expect(fmtMin(570)).toBe('09:30');
    expect(fmtMin(1439)).toBe('23:59');
    expect(fmtMin(NaN)).toBe('—');
});

test('fmtR / fmtPct / fmtNum', () => {
    expect(fmtR(1.5)).toBe('+1.50R');
    expect(fmtR(-0.8)).toBe('-0.80R');
    expect(fmtPct(0.65)).toBe('65.0%');
    expect(fmtNum(3.14159, 2)).toBe('3.14');
    expect(fmtR(NaN)).toBe('—');
});
