// Trade-feature cluster (k-means) helpers shared by view + vitest.
//
// Backend body: { features: [{entry_minute_of_day, hold_duration_minutes,
//   r_multiple}, ...], k: usize, max_iters: usize }.
// Returns: { assignments: number[], clusters: ClusterStat[] }.

const TOKEN_DELIM = /[\s,]+/;

// "<entry_minute> <hold_min> <r_multiple>" per line. Blank + #-comments ok.
export function parseFeatureBlob(text) {
    const features = [];
    const errors = [];
    if (typeof text !== 'string') {
        return { features, errors: [{ line_no: 0, raw: '', message: 'input not a string' }] };
    }
    const lines = text.split(/\r?\n/);
    for (let i = 0; i < lines.length; i++) {
        const raw = lines[i];
        // Strip inline #-comments before tokenizing so trailing "# note"
        // text doesn't get counted as data tokens.
        const hashIdx = raw.indexOf('#');
        const noComment = hashIdx >= 0 ? raw.slice(0, hashIdx) : raw;
        const s = noComment.trim();
        if (!s) continue;
        const parts = s.split(TOKEN_DELIM).filter(Boolean);
        if (parts.length !== 3) {
            errors.push({ line_no: i + 1, raw, message: `expected 3 tokens (entry_min hold_min r_multiple), got ${parts.length}` });
            continue;
        }
        const em = Number(parts[0]);
        const hd = Number(parts[1]);
        const r  = Number(parts[2]);
        if (![em, hd, r].every(Number.isFinite)) {
            errors.push({ line_no: i + 1, raw, message: 'tokens must be finite numbers' });
            continue;
        }
        if (em < 0 || em > 1440) {
            errors.push({ line_no: i + 1, raw, message: 'entry_minute_of_day must be 0..1440' });
            continue;
        }
        if (hd < 0) {
            errors.push({ line_no: i + 1, raw, message: 'hold_duration_minutes must be ≥ 0' });
            continue;
        }
        features.push({ entry_minute_of_day: em, hold_duration_minutes: hd, r_multiple: r });
    }
    return { features, errors };
}

export function validateInputs(features, k, maxIters) {
    if (!Array.isArray(features) || features.length === 0)
        return 'need ≥ 1 trade feature';
    if (!Number.isInteger(k) || k < 1) return 'k must be integer ≥ 1';
    if (!Number.isInteger(maxIters) || maxIters < 1) return 'max_iters must be integer ≥ 1';
    if (k > features.length) return `k (${k}) cannot exceed feature count (${features.length})`;
    return null;
}

export function buildBody(features, k, maxIters) {
    return { features, k, max_iters: maxIters };
}

// Backend-parity Lloyd's k-means with stride seeding.
// Mirrors crates/traderview-core/src/cluster_analysis.rs::analyze.
export function localAnalyze(features, k, maxIters) {
    if (!Array.isArray(features) || features.length === 0 || k === 0) {
        return { assignments: [], clusters: [] };
    }
    const kk = Math.min(k, features.length);
    const stride = Math.floor(features.length / kk);
    const centroids = [];
    for (let i = 0; i < kk; i++) {
        const f = features[i * stride];
        centroids.push({
            entry_minute: f.entry_minute_of_day,
            hold_minutes: f.hold_duration_minutes,
            r_multiple: f.r_multiple,
        });
    }
    const assignments = new Array(features.length).fill(0);
    for (let it = 0; it < maxIters; it++) {
        let changed = false;
        for (let i = 0; i < features.length; i++) {
            const f = features[i];
            let best = 0, bestD = Infinity;
            for (let j = 0; j < centroids.length; j++) {
                const d = sqDist(f, centroids[j]);
                if (d < bestD) { bestD = d; best = j; }
            }
            if (assignments[i] !== best) { changed = true; assignments[i] = best; }
        }
        if (!changed) break;
        for (let j = 0; j < kk; j++) {
            let n = 0, em = 0, hd = 0, r = 0;
            for (let i = 0; i < features.length; i++) {
                if (assignments[i] !== j) continue;
                n++; em += features[i].entry_minute_of_day;
                hd += features[i].hold_duration_minutes;
                r  += features[i].r_multiple;
            }
            if (n === 0) continue;
            centroids[j] = { entry_minute: em / n, hold_minutes: hd / n, r_multiple: r / n };
        }
    }
    const clusters = [];
    for (let j = 0; j < kk; j++) {
        let size = 0, sumR = 0, wins = 0;
        for (let i = 0; i < features.length; i++) {
            if (assignments[i] !== j) continue;
            size++;
            sumR += features[i].r_multiple;
            if (features[i].r_multiple > 0) wins++;
        }
        clusters.push({
            cluster_id: j, size,
            centroid: centroids[j],
            mean_r: size > 0 ? sumR / size : 0,
            win_rate: size > 0 ? wins / size : 0,
        });
    }
    return { assignments, clusters };
}

function sqDist(f, c) {
    const de = (f.entry_minute_of_day - c.entry_minute) / 1440;
    const dh = (f.hold_duration_minutes - c.hold_minutes) / 1440;
    const dr = (f.r_multiple - c.r_multiple) / 5;
    return de * de + dh * dh + dr * dr;
}

// Group features by cluster id for per-series uPlot scatter rendering.
// Returns parallel x[], y[] arrays per cluster, indexed by cluster_id.
export function pointsByCluster(features, assignments, k) {
    const xs = Array.from({ length: k }, () => []);
    const ys = Array.from({ length: k }, () => []);
    const rs = Array.from({ length: k }, () => []);
    for (let i = 0; i < features.length; i++) {
        const j = assignments[i];
        if (j < 0 || j >= k) continue;
        xs[j].push(features[i].entry_minute_of_day);
        ys[j].push(features[i].hold_duration_minutes);
        rs[j].push(features[i].r_multiple);
    }
    return { xs, ys, rs };
}

// Total within-cluster sum of squares (inertia) — useful summary scalar.
export function totalInertia(features, assignments, clusters) {
    if (!Array.isArray(features) || !Array.isArray(clusters) || clusters.length === 0) return 0;
    let total = 0;
    for (let i = 0; i < features.length; i++) {
        const j = assignments[i];
        if (j == null || j < 0 || j >= clusters.length) continue;
        const c = clusters[j].centroid;
        total += sqDist(features[i], c);
    }
    return total;
}

const PALETTE = ['#00e5ff', '#ffd84a', '#ff3860', '#23d18b', '#c678dd', '#ffa657'];

export function clusterColor(id) {
    if (!Number.isInteger(id) || id < 0) return '#aab';
    return PALETTE[id % PALETTE.length];
}

// Deterministic demo presets that produce visually distinct cluster
// outcomes. Each `kind` should classify as roughly its named topology.
export function makeDemoFeatures(kind = 'morning-vs-afternoon') {
    switch (kind) {
        case 'morning-vs-afternoon': {
            // Cluster A: morning short-hold winners. Cluster B: afternoon long-hold losers.
            const out = [];
            for (let i = 0; i < 12; i++) {
                out.push({ entry_minute_of_day: 540 + i * 3, hold_duration_minutes: 25 + (i % 5) * 2, r_multiple: 1.2 + (i % 4) * 0.3 });
            }
            for (let i = 0; i < 12; i++) {
                out.push({ entry_minute_of_day: 840 + i * 4, hold_duration_minutes: 200 + (i % 6) * 10, r_multiple: -0.8 - (i % 4) * 0.2 });
            }
            return out;
        }
        case 'three-style': {
            // 3 clusters: scalpers, swings, momentum-fades.
            const out = [];
            for (let i = 0; i < 10; i++) out.push({ entry_minute_of_day: 540 + i * 2, hold_duration_minutes: 5 + (i % 3), r_multiple: 0.5 + (i % 4) * 0.3 });
            for (let i = 0; i < 10; i++) out.push({ entry_minute_of_day: 600 + i * 4, hold_duration_minutes: 90 + (i % 5) * 10, r_multiple: 1.5 + (i % 3) * 0.5 });
            for (let i = 0; i < 10; i++) out.push({ entry_minute_of_day: 800 + i * 5, hold_duration_minutes: 240 + (i % 4) * 20, r_multiple: -1.0 - (i % 5) * 0.2 });
            return out;
        }
        case 'single': {
            // Tight single-cluster: ~all the same trade profile.
            const out = [];
            for (let i = 0; i < 20; i++) {
                out.push({ entry_minute_of_day: 570 + (i % 5), hold_duration_minutes: 30 + (i % 3), r_multiple: 1.2 + (i % 4) * 0.1 });
            }
            return out;
        }
        case 'scatter': {
            // Spread evenly across the day with random R.
            const out = [];
            for (let i = 0; i < 30; i++) {
                out.push({ entry_minute_of_day: 540 + i * 12, hold_duration_minutes: 30 + (i * 17) % 200, r_multiple: ((i * 23) % 11) / 3 - 1.5 });
            }
            return out;
        }
        default:
            return makeDemoFeatures('morning-vs-afternoon');
    }
}

export function fmtMin(v) {
    if (!Number.isFinite(v)) return '—';
    const h = Math.floor(v / 60);
    const m = Math.round(v % 60);
    return `${String(h).padStart(2, '0')}:${String(m).padStart(2, '0')}`;
}

export function fmtR(v) {
    if (!Number.isFinite(v)) return '—';
    const s = v >= 0 ? '+' : '';
    return s + v.toFixed(2) + 'R';
}

export function fmtPct(v) {
    if (!Number.isFinite(v)) return '—';
    return (v * 100).toFixed(1) + '%';
}

export function fmtNum(v, d = 1) {
    if (!Number.isFinite(v)) return '—';
    return v.toFixed(d);
}
