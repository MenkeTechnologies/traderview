// Trade-feature cluster view — k-means over (entry_minute, hold_minutes,
// R-multiple). Surfaces hidden cohorts in the trade journal — e.g. "your
// morning short-hold cluster has 70% win rate but afternoon long-hold
// cluster bleeds at -0.8R avg".

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseFeatureBlob, validateInputs, buildBody, localAnalyze,
    pointsByCluster, totalInertia, clusterColor, makeDemoFeatures,
    fmtMin, fmtR, fmtPct, fmtNum,
} from '../_clusters_trade_features_inputs.js';

import { t } from '../i18n.js';
let state = {
    features: makeDemoFeatures('morning-vs-afternoon'),
    k: 2,
    maxIters: 20,
};

export async function renderClustersTradeFeatures(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.clusters_trade_features.h1.trade_clusters_k_means" class="view-title">// TRADE CLUSTERS (K-MEANS)</h1>

        <div class="chart-panel">
            <h2>Paste trade features (per-line: <code>entry_min hold_min r_multiple</code>)</h2>
            <textarea id="cl-blob" rows="8" placeholder="540 30 1.5  # 9:00 entry, 30min hold, +1.5R&#10;...">${esc(featuresToBlob(state.features))}</textarea>
            <div class="inline-form">
                <label><span data-i18n="view.clusters_trade_features.label.k">k (clusters)</span>
                    <input id="cl-k" type="number" step="1" min="1" max="10" value="${state.k}"></label>
                <label><span data-i18n="view.clusters_trade_features.label.max_iters">Max iterations</span>
                    <input id="cl-iters" type="number" step="1" min="1" max="500" value="${state.maxIters}"></label>
                <button data-i18n="view.clusters_trade_features.btn.analyze" id="cl-run" class="primary" type="button">Analyze</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.clusters_trade_features.btn.demo_morning_vs_afternoon" id="cl-demo-morn-aft" class="secondary" type="button">Demo: morning vs afternoon</button>
                <button data-i18n="view.clusters_trade_features.btn.demo_3_trader_styles" id="cl-demo-three"    class="secondary" type="button">Demo: 3 trader styles</button>
                <button data-i18n="view.clusters_trade_features.btn.demo_single_cluster" id="cl-demo-single"   class="secondary" type="button">Demo: single cluster</button>
                <button data-i18n="view.clusters_trade_features.btn.demo_scattered_low_edge" id="cl-demo-scatter"  class="secondary" type="button">Demo: scattered (low-edge)</button>
            </div>
            <p data-i18n="view.clusters_trade_features.hint.entry_minute_is_minutes_from_midnight_utc_e_g_9_30" class="muted">Entry minute is minutes from midnight (UTC) — e.g. 9:30am ET ≈ 870. Distance is normalized: entry/1440, hold/1440, R/5 so each dim contributes ~comparably.</p>
        </div>

        <div id="cl-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.clusters_trade_features.h2.cluster_scatter_entry_time_hold_duration_color_clu">Cluster scatter — entry time × hold duration (color = cluster)</h2>
            <div id="cl-chart" style="height:360px"></div>
            <p data-i18n="view.clusters_trade_features.hint.each_dot_is_a_trade_color_cluster_id_x_axis_entry_" class="muted">Each dot is a trade. Color = cluster_id. X axis = entry minute of day (0–1440). Y = hold duration in minutes.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.clusters_trade_features.h2.per_cluster_stats">Per-cluster stats</h2>
            <div id="cl-clusters"></div>
        </div>

        <div id="cl-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (kind) => {
        state.features = makeDemoFeatures(kind);
        state.k = kind === 'three-style' ? 3 : kind === 'single' ? 1 : kind === 'scatter' ? 3 : 2;
        document.getElementById('cl-blob').value = featuresToBlob(state.features);
        document.getElementById('cl-k').value    = state.k;
    };
    document.getElementById('cl-demo-morn-aft').addEventListener('click',  () => loadDemo('morning-vs-afternoon'));
    document.getElementById('cl-demo-three').addEventListener('click',     () => loadDemo('three-style'));
    document.getElementById('cl-demo-single').addEventListener('click',    () => loadDemo('single'));
    document.getElementById('cl-demo-scatter').addEventListener('click',   () => loadDemo('scatter'));
    document.getElementById('cl-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function featuresToBlob(features) {
    return features.map(f => `${f.entry_minute_of_day} ${f.hold_duration_minutes} ${f.r_multiple}`).join('\n');
}

function readInputs() {
    const parsed = parseFeatureBlob(document.getElementById('cl-blob').value);
    if (parsed.errors.length) {
        showErr(`Parse errors: ${parsed.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; ')}`);
        return;
    }
    hideErr();
    state.features = parsed.features;
    state.k        = parseInt(document.getElementById('cl-k').value, 10);
    state.maxIters = parseInt(document.getElementById('cl-iters').value, 10);
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.features, state.k, state.maxIters);
    if (err) { showErr(err); return; }
    const local = localAnalyze(state.features, state.k, state.maxIters);
    renderSummary(local, true);
    renderChart(state.features, local);
    renderClusters(local);
    let resp;
    try {
        resp = await api.clustersTradeFeatures(buildBody(state.features, state.k, state.maxIters));
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(state.features, resp);
    renderClusters(resp);
}

function renderSummary(report, pending) {
    const local = localAnalyze(state.features, state.k, state.maxIters);
    const parity = arrEq(report.assignments, local.assignments);
    const wins = report.clusters.reduce((acc, c) => acc + c.win_rate * c.size, 0);
    const totalSize = report.clusters.reduce((acc, c) => acc + c.size, 0);
    const meanR = totalSize > 0
        ? report.clusters.reduce((acc, c) => acc + c.mean_r * c.size, 0) / totalSize
        : NaN;
    const best = pickBest(report.clusters, c => c.mean_r);
    const worst = pickBest(report.clusters, c => -c.mean_r);
    const inertia = totalInertia(state.features, report.assignments, report.clusters);
    document.getElementById('cl-summary').innerHTML = [
        card(t('view.clusters_trade_features.card.trades'),          String(state.features.length)),
        card(t('view.clusters_trade_features.card.k_clusters'),    String(report.clusters.length) + (pending ? ' (local)' : '')),
        card(t('view.clusters_trade_features.card.total_win_rate'),  fmtPct(totalSize > 0 ? wins / totalSize : 0),
            totalSize > 0 && wins / totalSize >= 0.5 ? 'pos' : 'neg'),
        card(t('view.clusters_trade_features.card.total_mean_r'),    fmtR(meanR),    meanR >= 0 ? 'pos' : 'neg'),
        card(t('view.clusters_trade_features.card.best_cluster'),    best ? `#${best.cluster_id}: ${fmtR(best.mean_r)} (n=${best.size})` : '—',
            best && best.mean_r >= 0 ? 'pos' : 'neg'),
        card(t('view.clusters_trade_features.card.worst_cluster'),   worst ? `#${worst.cluster_id}: ${fmtR(worst.mean_r)} (n=${worst.size})` : '—',
            worst && worst.mean_r <= 0 ? 'neg' : 'pos'),
        card(t('view.clusters_trade_features.card.inertia_wss'),   fmtNum(inertia, 4),
            inertia < 1 ? 'pos' : ''),
        card(t('view.clusters_trade_features.card.local_parity'),    parity ? 'OK' : 'DIVERGED', parity ? 'pos' : 'neg'),
    ].join('');
}

function pickBest(arr, scorer) {
    if (!arr.length) return null;
    return arr.reduce((best, c) => (best == null || scorer(c) > scorer(best)) ? c : best, null);
}

function arrEq(a, b) {
    if (!Array.isArray(a) || !Array.isArray(b) || a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) if (a[i] !== b[i]) return false;
    return true;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderChart(features, report) {
    if (!window.uPlot) return;
    const el = document.getElementById('cl-chart');
    if (!el) return;
    el.innerHTML = '';
    const k = report.clusters.length;
    const grouped = pointsByCluster(features, report.assignments, k);
    // For a scatter, uPlot needs one X array per series — we use the
    // per-cluster x slice. Build the full per-series data: first series
    // is the x-axis "all bars" (unused; uPlot needs x[]). We fake a global
    // x range so all series can share it via separate series x[]+y[] pairs.
    // Simplest: emit one series per cluster, each as a separate two-row
    // uPlot data row where everyone shares the longest x array.
    // Workaround: union all xs, then per-cluster y arrays line up by null-padding.
    const allXSet = new Set();
    for (const xs of grouped.xs) for (const x of xs) allXSet.add(x);
    const xs = Array.from(allXSet).sort((a, b) => a - b);
    const seriesData = [xs];
    const seriesDefs = [{ label: 'entry_min' }];
    for (let j = 0; j < k; j++) {
        const idxByX = new Map();
        for (let i = 0; i < grouped.xs[j].length; i++) {
            // If two trades share entry_min, ensure both points render —
            // bump with a tiny epsilon. uPlot will draw both points.
            let key = grouped.xs[j][i];
            while (idxByX.has(key)) key += 1e-9;
            idxByX.set(key, grouped.ys[j][i]);
        }
        const yArr = xs.map(x => idxByX.has(x) ? idxByX.get(x) : null);
        seriesData.push(yArr);
        const c = clusterColor(j);
        const stat = report.clusters[j] || { mean_r: 0, size: 0, win_rate: 0 };
        seriesDefs.push({
            label: `c${j} n=${stat.size} ${fmtR(stat.mean_r)} ${fmtPct(stat.win_rate)}`,
            stroke: c, width: 0,
            points: { show: true, size: 6, fill: c, stroke: c },
        });
    }
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 340,
        scales: { x: { range: [0, 1440] }, y: {} },
        series: seriesDefs,
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => fmtMin(v)) },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, seriesData, el);
}

function renderClusters(report) {
    const wrap = document.getElementById('cl-clusters');
    if (!report.clusters.length) { wrap.innerHTML = '<div class="muted">No clusters.</div>'; return; }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.clusters_trade_features.th.cluster">Cluster</th><th data-i18n="view.clusters_trade_features.th.size">Size</th><th data-i18n="view.clusters_trade_features.th.centroid_entry">Centroid entry</th><th data-i18n="view.clusters_trade_features.th.centroid_hold">Centroid hold</th>
                <th data-i18n="view.clusters_trade_features.th.centroid_r">Centroid R</th><th data-i18n="view.clusters_trade_features.th.mean_r">Mean R</th><th data-i18n="view.clusters_trade_features.th.win_rate">Win rate</th>
            </tr></thead>
            <tbody>
                ${report.clusters.map(c => `<tr>
                    <td><span style="color:${esc(clusterColor(c.cluster_id))};font-weight:bold">●</span> c${c.cluster_id}</td>
                    <td>${c.size}</td>
                    <td>${esc(fmtMin(c.centroid.entry_minute))}</td>
                    <td>${esc(fmtNum(c.centroid.hold_minutes, 0))} min</td>
                    <td>${esc(fmtR(c.centroid.r_multiple))}</td>
                    <td class="${c.mean_r >= 0 ? 'pos' : 'neg'}">${esc(fmtR(c.mean_r))}</td>
                    <td class="${c.win_rate >= 0.5 ? 'pos' : 'neg'}">${esc(fmtPct(c.win_rate))}</td>
                </tr>`).join('')}
            </tbody>
        </table>
    `;
}

function showErr(msg) {
    const el = document.getElementById('cl-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('cl-err').style.display = 'none'; }
