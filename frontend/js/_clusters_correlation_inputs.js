// Correlation-cluster helpers shared by view + vitest.
//
// Backend body: { positions: [{symbol, notional}, ...],
//   correlations: [{a, b, corr}, ...], threshold: f64 }.
// Returns: Cluster[] with {members, gross_exposure, net_exposure}, sorted
// by gross_exposure DESC. Union-find single-link agglomerative — two
// positions cluster if |corr| ≥ threshold (transitive via chain).

import { t } from './i18n.js';

const TOKEN_DELIM = /[\s,]+/;

// "<symbol> <notional>" per line. Notional may be negative (short).
export function parsePositionBlob(text) {
    const positions = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { positions, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    const seen = new Set();
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const noComment = stripComment(raw);
        const s = noComment.trim();
        if (!s) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 2) {
            errors.push({ line_no: i + 1, raw, message: `expected 2 tokens (symbol notional), got ${parts.length}` });
            continue;
        }
        const sym = parts[0].toUpperCase();
        const notional = Number(parts[1]);
        if (!Number.isFinite(notional)) {
            errors.push({ line_no: i + 1, raw, message: 'notional must be finite number' });
            continue;
        }
        if (seen.has(sym)) {
            errors.push({ line_no: i + 1, raw, message: `duplicate symbol ${sym}` });
            continue;
        }
        seen.add(sym);
        positions.push({ symbol: sym, notional });
    }
    return { positions, errors };
}

// "<a> <b> <corr>" per line. corr in [-1, 1]. Self-pairs allowed but ignored.
export function parseCorrelationBlob(text) {
    const correlations = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { correlations, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        const noComment = stripComment(raw);
        const s = noComment.trim();
        if (!s) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 3) {
            errors.push({ line_no: i + 1, raw, message: `expected 3 tokens (a b corr), got ${parts.length}` });
            continue;
        }
        const a = parts[0].toUpperCase();
        const b = parts[1].toUpperCase();
        const corr = Number(parts[2]);
        if (!Number.isFinite(corr) || corr < -1 || corr > 1) {
            errors.push({ line_no: i + 1, raw, message: 'corr must be in [-1, 1]' });
            continue;
        }
        correlations.push({ a, b, corr });
    }
    return { correlations, errors };
}

function stripComment(raw) {
    const i = raw.indexOf('#');
    return i >= 0 ? raw.slice(0, i) : raw;
}

export function validateInputs(positions, correlations, threshold) {
    if (!Array.isArray(positions) || positions.length === 0)
        return t('view.clusters_correlation.validate.need_position');
    if (!Array.isArray(correlations)) return t('view.clusters_correlation.validate.corr_array');
    if (!Number.isFinite(threshold) || threshold < 0 || threshold > 1)
        return t('view.clusters_correlation.validate.threshold');
    return null;
}

export function buildBody(positions, correlations, threshold) {
    return { positions, correlations, threshold };
}

// Pure-JS mirror of crates/traderview-core/src/correlation_clusters.rs::cluster.
// Union-find with |rho| ≥ threshold links + transitive chain + path
// compression + sort-by-gross-exposure-desc.
export function localCluster(positions, correlations, threshold) {
    const n = positions.length;
    if (n === 0) return [];
    const parent = Array.from({ length: n }, (_, i) => i);
    const idxOf = new Map();
    positions.forEach((p, i) => idxOf.set(p.symbol, i));
    // Build symmetric lookup so order doesn't matter.
    const corrMap = new Map();
    for (const e of correlations) {
        corrMap.set(`${e.a}|${e.b}`, e.corr);
        corrMap.set(`${e.b}|${e.a}`, e.corr);
    }
    for (let i = 0; i < n; i++) {
        for (let j = i + 1; j < n; j++) {
            const a = positions[i].symbol;
            const b = positions[j].symbol;
            const rho = corrMap.get(`${a}|${b}`) ?? 0;
            if (Math.abs(rho) >= threshold) {
                union(parent, idxOf.get(a), idxOf.get(b));
            }
        }
    }
    const groups = new Map();
    for (let i = 0; i < n; i++) {
        const r = find(parent, i);
        if (!groups.has(r)) groups.set(r, []);
        groups.get(r).push(i);
    }
    const out = [];
    for (const idxs of groups.values()) {
        let gross = 0, net = 0;
        const members = [];
        for (const i of idxs) {
            members.push(positions[i].symbol);
            gross += Math.abs(positions[i].notional);
            net   += positions[i].notional;
        }
        out.push({ members, gross_exposure: gross, net_exposure: net });
    }
    out.sort((a, b) => b.gross_exposure - a.gross_exposure);
    return out;
}

function find(p, i) {
    let r = i;
    while (p[r] !== r) r = p[r];
    let cur = i;
    while (p[cur] !== r) {
        const next = p[cur];
        p[cur] = r;
        cur = next;
    }
    return r;
}

function union(p, a, b) {
    const ra = find(p, a);
    const rb = find(p, b);
    if (ra !== rb) p[ra] = rb;
}

// Summary scalars: top cluster as % of total gross + concentration risk.
export function summarize(clusters) {
    const totalGross = clusters.reduce((a, c) => a + c.gross_exposure, 0);
    const totalNet   = clusters.reduce((a, c) => a + c.net_exposure, 0);
    const top = clusters[0] || null;
    const topPct = (totalGross > 0 && top) ? top.gross_exposure / totalGross : 0;
    const groupedPositions = clusters.reduce((a, c) => a + c.members.length, 0);
    return {
        nClusters: clusters.length, totalGross, totalNet, top, topPct,
        groupedPositions,
        maxClusterSize: clusters.reduce((m, c) => Math.max(m, c.members.length), 0),
        singletons: clusters.filter(c => c.members.length === 1).length,
    };
}

// Concentration-risk traffic light: ≥70% of book in one cluster → red,
// ≥50% → amber, < 50% → green.
export function concentrationBadge(topPct) {
    if (!Number.isFinite(topPct)) return { label: '—', cls: '', hint: '—' };
    if (topPct >= 0.7) return { label: t('view.clusters_correlation.conc.concentrated.label'), cls: 'neg', hint: t('view.clusters_correlation.conc.concentrated.hint') };
    if (topPct >= 0.5) return { label: t('view.clusters_correlation.conc.tilted.label'),       cls: 'neg', hint: t('view.clusters_correlation.conc.tilted.hint') };
    if (topPct >= 0.3) return { label: t('view.clusters_correlation.conc.moderate.label'),     cls: '',    hint: t('view.clusters_correlation.conc.moderate.hint') };
    return { label: t('view.clusters_correlation.conc.diverse.label'), cls: 'pos', hint: t('view.clusters_correlation.conc.diverse.hint') };
}

// Demo presets with realistic correlation structures.
export function makeDemoPositions(kind = 'mega-cap-tech') {
    switch (kind) {
        case 'mega-cap-tech':
            // 4 tech mega-caps + 1 energy outlier. Tech cluster dominates.
            return [
                ['AAPL',  20_000], ['MSFT',  15_000],
                ['GOOGL', 18_000], ['META',  12_000],
                ['XOM',    5_000],
            ].map(([s, n]) => ({ symbol: s, notional: n }));
        case 'inverse-pair':
            return [
                ['QQQ',  10_000], ['SQQQ', -5_000],
                ['SPY',  20_000], ['XOM',   8_000],
            ].map(([s, n]) => ({ symbol: s, notional: n }));
        case 'sector-chain':
            // A-B-C chain via single-link transitivity, plus D solo.
            return [
                ['A',  5_000], ['B',  5_000], ['C',  5_000], ['D',  5_000],
            ].map(([s, n]) => ({ symbol: s, notional: n }));
        case 'all-singletons':
            return [
                ['AAPL', 5_000], ['XOM',  5_000], ['GLD',  5_000], ['TLT',  5_000],
            ].map(([s, n]) => ({ symbol: s, notional: n }));
        default:
            return makeDemoPositions('mega-cap-tech');
    }
}

export function makeDemoCorrelations(kind = 'mega-cap-tech') {
    switch (kind) {
        case 'mega-cap-tech':
            return [
                ['AAPL', 'MSFT',  0.85], ['AAPL', 'GOOGL', 0.82], ['AAPL', 'META',  0.78],
                ['MSFT', 'GOOGL', 0.80], ['MSFT', 'META',  0.75], ['GOOGL','META',  0.79],
                ['AAPL', 'XOM',   0.15], ['MSFT', 'XOM',   0.10], ['GOOGL','XOM',   0.12],
                ['META', 'XOM',   0.08],
            ].map(([a, b, c]) => ({ a, b, corr: c }));
        case 'inverse-pair':
            return [
                ['QQQ', 'SQQQ', -0.95], ['QQQ', 'SPY', 0.92], ['SQQQ','SPY',  -0.90],
                ['QQQ', 'XOM',   0.20], ['SPY', 'XOM',  0.45], ['SQQQ','XOM',  -0.15],
            ].map(([a, b, c]) => ({ a, b, corr: c }));
        case 'sector-chain':
            return [
                ['A', 'B', 0.80], ['B', 'C', 0.80], ['A', 'C', 0.20],
                ['A', 'D', 0.10], ['B', 'D', 0.05], ['C', 'D', 0.10],
            ].map(([a, b, c]) => ({ a, b, corr: c }));
        case 'all-singletons':
            return [
                ['AAPL', 'XOM', 0.10], ['AAPL', 'GLD', -0.05], ['AAPL', 'TLT', -0.10],
                ['XOM',  'GLD', 0.20], ['XOM',  'TLT', -0.15],
                ['GLD',  'TLT', 0.30],
            ].map(([a, b, c]) => ({ a, b, corr: c }));
        default:
            return makeDemoCorrelations('mega-cap-tech');
    }
}

export function fmtUSD(v, d = 0) {
    if (!Number.isFinite(v)) return '—';
    const sign = v < 0 ? '-' : '';
    return sign + '$' + Math.abs(v).toFixed(d);
}

export function fmtUSDSigned(v, d = 0) {
    if (!Number.isFinite(v)) return '—';
    return (v >= 0 ? '+' : '-') + '$' + Math.abs(v).toFixed(d);
}

export function fmtPct(v, d = 1) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(d) + '%';
}

// Used to dim the table row for singleton clusters (no hidden risk).
export function clusterRowClass(cluster) {
    if (!cluster) return '';
    if (cluster.members.length === 1) return 'muted';
    return '';
}

// Cluster color (cycles palette) for the per-cluster summary block.
const PALETTE = ['#00e5ff', '#ffd84a', '#ff3860', '#23d18b', '#c678dd', '#ffa657'];
export function clusterColor(idx) {
    if (!Number.isInteger(idx) || idx < 0) return '#aab';
    return PALETTE[idx % PALETTE.length];
}
