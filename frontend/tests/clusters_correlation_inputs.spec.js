// Correlation-cluster helpers: parsers, validator, body shape, local
// union-find parity, summarize, concentration badge, demos.

import { test, expect } from 'vitest';
import {
    parsePositionBlob, parseCorrelationBlob, validateInputs, buildBody,
    localCluster, summarize, concentrationBadge,
    makeDemoPositions, makeDemoCorrelations,
    fmtUSD, fmtUSDSigned, fmtPct, clusterColor, clusterRowClass,
} from '../js/_clusters_correlation_inputs.js';

const pos = (sym, n) => ({ symbol: sym, notional: n });
const edge = (a, b, c) => ({ a, b, corr: c });

// ── parsePositionBlob ─────────────────────────────────────────────

test('parsePositionBlob accepts 2-token rows + #-comments + uppercase symbols', () => {
    const r = parsePositionBlob('aapl 10000  # tech\n# pure comment\nxom 5000');
    expect(r.errors).toEqual([]);
    expect(r.positions).toEqual([pos('AAPL', 10000), pos('XOM', 5000)]);
});

test('parsePositionBlob rejects duplicate symbols (case-insensitive)', () => {
    const r = parsePositionBlob('AAPL 1\naapl 2');
    expect(r.errors[0].message).toMatch(/duplicate/);
    expect(r.positions.length).toBe(1);
});

test('parsePositionBlob accepts negative notional (short positions)', () => {
    const r = parsePositionBlob('SQQQ -5000');
    expect(r.errors).toEqual([]);
    expect(r.positions[0].notional).toBe(-5000);
});

test('parsePositionBlob rejects non-finite notional', () => {
    expect(parsePositionBlob('XYZ abc').errors[0].message).toMatch(/notional/);
});

test('parsePositionBlob rejects wrong token count', () => {
    expect(parsePositionBlob('AAPL').errors[0].message).toMatch(/expected 2 tokens/);
});

test('parsePositionBlob non-string returns 1 error', () => {
    expect(parsePositionBlob(null).errors.length).toBe(1);
});

// ── parseCorrelationBlob ──────────────────────────────────────────

test('parseCorrelationBlob accepts 3-token rows + uppercase + comments', () => {
    const r = parseCorrelationBlob('aapl msft 0.85\n# pair\nspy xom 0.45');
    expect(r.errors).toEqual([]);
    expect(r.correlations).toEqual([edge('AAPL', 'MSFT', 0.85), edge('SPY', 'XOM', 0.45)]);
});

test('parseCorrelationBlob rejects corr outside [-1, 1]', () => {
    expect(parseCorrelationBlob('A B 1.5').errors[0].message).toMatch(/\[-1, 1\]/);
    expect(parseCorrelationBlob('A B -1.5').errors[0].message).toMatch(/\[-1, 1\]/);
});

test('parseCorrelationBlob accepts boundary corr=1 and -1', () => {
    const r = parseCorrelationBlob('A B 1\nC D -1');
    expect(r.errors).toEqual([]);
    expect(r.correlations.length).toBe(2);
});

test('parseCorrelationBlob rejects bad token count', () => {
    expect(parseCorrelationBlob('A B').errors[0].message).toMatch(/expected 3 tokens/);
});

// ── validate / buildBody ──────────────────────────────────────────

test('validate accepts good inputs', () => {
    expect(validateInputs([pos('A', 1)], [edge('A', 'B', 0.5)], 0.7)).toBe(null);
});

test('validate rejects empty positions / bad correlations type / bad threshold', () => {
    expect(validateInputs([], [], 0.7)).toMatch(/≥ 1 position/);
    expect(validateInputs([pos('A', 1)], null, 0.7)).toMatch(/correlations/);
    expect(validateInputs([pos('A', 1)], [], -0.1)).toMatch(/threshold/);
    expect(validateInputs([pos('A', 1)], [], 1.5)).toMatch(/threshold/);
});

test('buildBody emits backend CorrelationClustersBody shape', () => {
    const p = [pos('A', 1)];
    const c = [edge('A', 'B', 0.5)];
    expect(buildBody(p, c, 0.7)).toEqual({ positions: p, correlations: c, threshold: 0.7 });
});

// ── localCluster parity (one test per Rust test case) ─────────────

test('local: empty returns empty', () => {
    expect(localCluster([], [], 0.7)).toEqual([]);
});

test('local: isolated positions become singletons', () => {
    const out = localCluster(
        [pos('AAPL', 10_000), pos('XOM', 5_000)],
        [edge('AAPL', 'XOM', 0.10)], 0.7);
    expect(out.length).toBe(2);
});

test('local: high-correlation pair groups, gross + net match', () => {
    const out = localCluster(
        [pos('AAPL', 10_000), pos('MSFT', 8_000)],
        [edge('AAPL', 'MSFT', 0.85)], 0.7);
    expect(out.length).toBe(1);
    expect(out[0].members.length).toBe(2);
    expect(out[0].gross_exposure).toBe(18_000);
    expect(out[0].net_exposure).toBe(18_000);
});

test('local: transitive chain A-B-C in one cluster (single-link)', () => {
    const out = localCluster(
        [pos('A', 1000), pos('B', 1000), pos('C', 1000)],
        [edge('A', 'B', 0.8), edge('B', 'C', 0.8), edge('A', 'C', 0.0)],
        0.7);
    expect(out.length).toBe(1);
    expect(out[0].members.length).toBe(3);
});

test('local: negative correlation above threshold (|rho|) clusters', () => {
    const out = localCluster(
        [pos('QQQ', 10_000), pos('SQQQ', -5_000)],
        [edge('QQQ', 'SQQQ', -0.95)], 0.7);
    expect(out.length).toBe(1);
    expect(out[0].gross_exposure).toBe(15_000);
    expect(out[0].net_exposure).toBe(5_000);
});

test('local: pair lookup is order-independent', () => {
    const out = localCluster(
        [pos('X', 1), pos('Y', 1)],
        [edge('Y', 'X', 0.9)], 0.7);
    expect(out.length).toBe(1);
});

test('local: missing pair defaults to rho=0 → no cluster', () => {
    const out = localCluster([pos('X', 1), pos('Y', 1)], [], 0.7);
    expect(out.length).toBe(2);
});

test('local: clusters sorted by gross_exposure DESC', () => {
    const out = localCluster(
        [pos('XOM', 5_000), pos('AAPL', 10_000), pos('MSFT', 8_000)],
        [edge('AAPL', 'MSFT', 0.9)], 0.7);
    expect(out[0].gross_exposure).toBe(18_000);
    expect(out[1].gross_exposure).toBe(5_000);
});

test('local: threshold is INCLUSIVE (≥ not >)', () => {
    const out = localCluster([pos('A', 1), pos('B', 1)], [edge('A', 'B', 0.7)], 0.7);
    expect(out.length).toBe(1);
});

test('local: each position appears in exactly one cluster (partition)', () => {
    const positions = makeDemoPositions('mega-cap-tech');
    const out = localCluster(positions, makeDemoCorrelations('mega-cap-tech'), 0.7);
    const seen = new Set();
    for (const c of out) for (const m of c.members) {
        expect(seen.has(m)).toBe(false);
        seen.add(m);
    }
    expect(seen.size).toBe(positions.length);
});

// ── summarize / concentrationBadge ────────────────────────────────

test('summarize: empty clusters returns zeros + null top', () => {
    const s = summarize([]);
    expect(s.nClusters).toBe(0);
    expect(s.totalGross).toBe(0);
    expect(s.top).toBe(null);
    expect(s.topPct).toBe(0);
});

test('summarize: top cluster is first (clusters arrive sorted-by-gross)', () => {
    const clusters = [
        { members: ['A', 'B'], gross_exposure: 30, net_exposure: 30 },
        { members: ['C'],      gross_exposure: 10, net_exposure: -10 },
    ];
    const s = summarize(clusters);
    expect(s.top.members.length).toBe(2);
    expect(s.topPct).toBeCloseTo(30 / 40, 10);
    expect(s.totalNet).toBe(20);
    expect(s.maxClusterSize).toBe(2);
    expect(s.singletons).toBe(1);
});

test('concentrationBadge: thresholds 30/50/70', () => {
    expect(concentrationBadge(0.75).label).toBe('CONCENTRATED');
    expect(concentrationBadge(0.5).label).toBe('TILTED');
    expect(concentrationBadge(0.35).label).toBe('MODERATE');
    expect(concentrationBadge(0.20).label).toBe('DIVERSE');
    expect(concentrationBadge(NaN).label).toBe('—');
});

test('concentrationBadge: 0.7 hits CONCENTRATED bucket exactly', () => {
    expect(concentrationBadge(0.7).cls).toBe('neg');
});

// ── demos invariants ──────────────────────────────────────────────

test('demo mega-cap-tech: 4 tech symbols cluster, XOM singleton', () => {
    const out = localCluster(makeDemoPositions('mega-cap-tech'),
                             makeDemoCorrelations('mega-cap-tech'), 0.7);
    // Expect 2 clusters: the 4-mega-tech + XOM solo.
    expect(out.length).toBe(2);
    expect(out[0].members.length).toBe(4);
    expect(out[0].members.sort()).toEqual(['AAPL', 'GOOGL', 'META', 'MSFT']);
    expect(out[1].members).toEqual(['XOM']);
});

test('demo inverse-pair: QQQ + SQQQ + SPY all link via 0.92/0.95/0.90', () => {
    const out = localCluster(makeDemoPositions('inverse-pair'),
                             makeDemoCorrelations('inverse-pair'), 0.7);
    expect(out[0].members.length).toBeGreaterThanOrEqual(3);
});

test('demo sector-chain: A-B-C transitive chain, D solo', () => {
    const out = localCluster(makeDemoPositions('sector-chain'),
                             makeDemoCorrelations('sector-chain'), 0.7);
    expect(out.length).toBe(2);
    const tri = out.find(c => c.members.length === 3);
    expect(tri.members.sort()).toEqual(['A', 'B', 'C']);
});

test('demo all-singletons: every position is its own cluster', () => {
    const out = localCluster(makeDemoPositions('all-singletons'),
                             makeDemoCorrelations('all-singletons'), 0.7);
    expect(out.length).toBe(4);
    expect(out.every(c => c.members.length === 1)).toBe(true);
});

// ── formatters + helpers ──────────────────────────────────────────

test('fmtUSD / fmtUSDSigned / fmtPct + non-finite guards', () => {
    expect(fmtUSD(1234)).toBe('$1234');
    expect(fmtUSD(-100)).toBe('-$100');
    expect(fmtUSDSigned(100)).toBe('+$100');
    expect(fmtUSDSigned(-100)).toBe('-$100');
    expect(fmtPct(0.05)).toBe('5.0%');
    expect(fmtUSD(NaN)).toBe('—');
    expect(fmtPct(NaN)).toBe('—');
});

test('clusterColor cycles palette, neg id → muted', () => {
    expect(clusterColor(0)).toBe('#00e5ff');
    expect(clusterColor(6)).toBe('#00e5ff');
    expect(clusterColor(-1)).toBe('#aab');
});

test('clusterRowClass: singletons get muted; multi-member empty class', () => {
    expect(clusterRowClass({ members: ['A'] })).toBe('muted');
    expect(clusterRowClass({ members: ['A', 'B'] })).toBe('');
    expect(clusterRowClass(null)).toBe('');
});
