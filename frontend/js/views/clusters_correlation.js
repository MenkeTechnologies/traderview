// Correlation-cluster view — single-link agglomerative clustering of
// positions by |pairwise correlation| ≥ threshold. Exposes disguised
// concentration: AAPL+MSFT+GOOGL+META isn't "4 positions", it's 1
// mega-cap-tech bet.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parsePositionBlob, parseCorrelationBlob, validateInputs, buildBody,
    localCluster, summarize, concentrationBadge,
    makeDemoPositions, makeDemoCorrelations,
    fmtUSD, fmtUSDSigned, fmtPct, clusterColor,
} from '../_clusters_correlation_inputs.js';

import { t } from '../i18n.js';
import { showToast } from '../toast.js';
let state = {
    positions: makeDemoPositions('mega-cap-tech'),
    correlations: makeDemoCorrelations('mega-cap-tech'),
    threshold: 0.7,
};

export async function renderClustersCorrelation(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.clusters_correlation.h1.correlation_clusters" class="view-title">// CORRELATION CLUSTERS</h1>

        <div class="chart-panel">
            <h2><span data-i18n="view.clusters_correlation.h2.positions">Positions</span> <small class="muted"><span data-i18n="view.clusters_correlation.h2.positions_hint">(per line: </span><code>SYMBOL notional</code><span data-i18n="view.clusters_correlation.h2.positions_hint2">, neg = short)</span></small></h2>
            <textarea id="cc-pos" rows="5" placeholder="AAPL 20000&#10;MSFT 15000" data-tip="view.clusters_correlation.tip.positions">${esc(positionsToBlob(state.positions))}</textarea>
            <h2><span data-i18n="view.clusters_correlation.h2.correlations">Correlations</span> <small class="muted"><span data-i18n="view.clusters_correlation.h2.correlations_hint">(per line: </span><code>A B corr</code><span data-i18n="view.clusters_correlation.h2.correlations_hint2">, order doesn't matter)</span></small></h2>
            <textarea id="cc-corr" rows="6" placeholder="AAPL MSFT 0.85" data-tip="view.clusters_correlation.tip.correlations">${esc(correlationsToBlob(state.correlations))}</textarea>
            <div class="inline-form">
                <label><span data-i18n="view.clusters_correlation.label.threshold">Threshold |ρ|</span>
                    <input id="cc-thr" type="number" step="any" min="0" max="1" value="${state.threshold}" data-tip="view.clusters_correlation.tip.threshold"></label>
                <button data-i18n="view.clusters_correlation.btn.cluster" id="cc-run" class="primary" type="button" data-tip="view.clusters_correlation.tip.run" data-shortcut="clusters_correlation_run">Cluster</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.clusters_correlation.btn.demo_mega_cap_tech_cluster" id="cc-demo-tech"     class="secondary" type="button" data-tip="view.clusters_correlation.tip.demo_tech">Demo: mega-cap-tech cluster</button>
                <button data-i18n="view.clusters_correlation.btn.demo_inverse_pair_qqq_sqqq" id="cc-demo-inverse"  class="secondary" type="button" data-tip="view.clusters_correlation.tip.demo_inverse">Demo: inverse pair (QQQ/SQQQ)</button>
                <button data-i18n="view.clusters_correlation.btn.demo_a_b_c_transitive_chain" id="cc-demo-chain"    class="secondary" type="button" data-tip="view.clusters_correlation.tip.demo_chain">Demo: A-B-C transitive chain</button>
                <button data-i18n="view.clusters_correlation.btn.demo_all_singletons" id="cc-demo-diverse"  class="secondary" type="button" data-tip="view.clusters_correlation.tip.demo_diverse">Demo: all singletons</button>
            </div>
            <p data-i18n="view.clusters_correlation.hint.positions_cluster_if_there_s_a_chain_of_pairs_each" class="muted">Positions cluster if there's a chain of pairs each with |ρ| ≥ threshold. Threshold is inclusive (≥). Missing pairs default to ρ = 0.</p>
        </div>

        <div id="cc-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.clusters_correlation.h2.clusters_sorted_by_gross_exposure">Clusters (sorted by gross exposure)</h2>
            <div id="cc-clusters"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.clusters_correlation.h2.net_exposure_chart">Net exposure per cluster</h2>
            <div id="cc-chart" style="width:100%;height:240px"></div>
        </div>

        <div id="cc-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (kind) => {
        state.positions    = makeDemoPositions(kind);
        state.correlations = makeDemoCorrelations(kind);
        state.threshold    = kind === 'sector-chain' ? 0.7 : 0.7;
        document.getElementById('cc-pos').value  = positionsToBlob(state.positions);
        document.getElementById('cc-corr').value = correlationsToBlob(state.correlations);
        document.getElementById('cc-thr').value  = state.threshold;
    };
    document.getElementById('cc-demo-tech').addEventListener('click',    () => loadDemo('mega-cap-tech'));
    document.getElementById('cc-demo-inverse').addEventListener('click', () => loadDemo('inverse-pair'));
    document.getElementById('cc-demo-chain').addEventListener('click',   () => loadDemo('sector-chain'));
    document.getElementById('cc-demo-diverse').addEventListener('click', () => loadDemo('all-singletons'));
    document.getElementById('cc-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function positionsToBlob(positions) {
    return positions.map(p => `${p.symbol} ${p.notional}`).join('\n');
}

function correlationsToBlob(correlations) {
    return correlations.map(e => `${e.a} ${e.b} ${e.corr}`).join('\n');
}

function readInputs() {
    const pPos = parsePositionBlob(document.getElementById('cc-pos').value);
    const pCorr = parseCorrelationBlob(document.getElementById('cc-corr').value);
    const errs = [...pPos.errors.map(e => `pos[${e.line_no}] ${e.message}`),
                  ...pCorr.errors.map(e => `corr[${e.line_no}] ${e.message}`)];
    if (errs.length) {
        showErr(errs.slice(0, 4).join('; '));
        showToast(t('view.clusters_correlation.toast.parse_error', { n: errs.length }), { level: 'warning' });
        return;
    }
    hideErr();
    state.positions    = pPos.positions;
    state.correlations = pCorr.correlations;
    state.threshold    = Number(document.getElementById('cc-thr').value);
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.positions, state.correlations, state.threshold);
    if (err) { showErr(err); showToast(t('view.clusters_correlation.toast.invalid'), { level: 'warning' }); return; }
    const local = localCluster(state.positions, state.correlations, state.threshold);
    renderSummary(local, true);
    renderClusters(local);
    renderExposureChart(local);
    let resp;
    try {
        resp = await api.clustersCorrelation(buildBody(state.positions, state.correlations, state.threshold));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        showToast(t('view.clusters_correlation.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderClusters(resp);
    renderExposureChart(resp);
    const s = summarize(resp);
    const level = s.topPct >= 0.5 ? 'warning' : 'success';
    showToast(t('view.clusters_correlation.toast.clustered', { n: s.nClusters, top: (s.topPct * 100).toFixed(0) }), { level });
}

function renderSummary(clusters, pending) {
    const s = summarize(clusters);
    const local = localCluster(state.positions, state.correlations, state.threshold);
    const parity = clustersEq(clusters, local);
    const badge = concentrationBadge(s.topPct);
    document.getElementById('cc-summary').innerHTML = [
        card(t('view.clusters_correlation.card.concentration'),  badge.label + (pending ? t('common.suffix.local') : ''), badge.cls),
        card(t('view.clusters_correlation.card.action'),         badge.hint),
        card(t('view.clusters_correlation.card.positions'),      String(state.positions.length)),
        card(t('view.clusters_correlation.card.clusters'),       String(s.nClusters)),
        card(t('view.clusters_correlation.card.singletons'),     String(s.singletons),
            s.singletons === state.positions.length ? 'pos' : ''),
        card(t('view.clusters_correlation.card.largest_cluster_size'), String(s.maxClusterSize),
            s.maxClusterSize >= 4 ? 'neg' : ''),
        card(t('view.clusters_correlation.card.top_cluster'),  fmtPct(s.topPct), badge.cls),
        card(t('view.clusters_correlation.card.total_gross'),    fmtUSD(s.totalGross)),
        card(t('view.clusters_correlation.card.total_net'),      fmtUSDSigned(s.totalNet),
            s.totalNet >= 0 ? 'pos' : 'neg'),
        card(t('view.clusters_correlation.card.local_parity'),   parity ? t('common.ok') : t('common.diverged'), parity ? 'pos' : 'neg'),
    ].join('');
}

function clustersEq(a, b) {
    if (!Array.isArray(a) || !Array.isArray(b) || a.length !== b.length) return false;
    for (let i = 0; i < a.length; i++) {
        if (a[i].gross_exposure !== b[i].gross_exposure) return false;
        if (a[i].net_exposure   !== b[i].net_exposure)   return false;
        if (a[i].members.length !== b[i].members.length) return false;
        const A = [...a[i].members].sort();
        const B = [...b[i].members].sort();
        for (let j = 0; j < A.length; j++) if (A[j] !== B[j]) return false;
    }
    return true;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderClusters(clusters) {
    const wrap = document.getElementById('cc-clusters');
    if (!clusters.length) { wrap.innerHTML = `<div class="muted" data-i18n="view.clusters_correlation.empty.clusters">No clusters.</div>`; return; }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>#</th><th data-i18n="view.clusters_correlation.th.members">Members</th><th data-i18n="view.clusters_correlation.th.size">Size</th>
                <th data-i18n="view.clusters_correlation.th.gross_exposure">Gross exposure</th><th data-i18n="view.clusters_correlation.th.net_exposure">Net exposure</th><th data-i18n="view.clusters_correlation.th.direction">Direction</th>
            </tr></thead>
            <tbody>
                ${clusters.map((c, i) => `<tr>
                    <td><span style="color:${esc(clusterColor(i))};font-weight:bold">●</span> c${i}</td>
                    <td><strong>${c.members.map(esc).join(', ')}</strong></td>
                    <td>${c.members.length}</td>
                    <td>${esc(fmtUSD(c.gross_exposure))}</td>
                    <td class="${c.net_exposure >= 0 ? 'pos' : 'neg'}">${esc(fmtUSDSigned(c.net_exposure))}</td>
                    <td class="${c.net_exposure >= 0 ? 'pos' : 'neg'}">${c.net_exposure >= 0 ? 'LONG' : 'SHORT'}${Math.abs(c.net_exposure) < c.gross_exposure ? ' (hedged)' : ''}</td>
                </tr>`).join('')}
            </tbody>
        </table>
    `;
}

function renderExposureChart(clusters) {
    const el = document.getElementById('cc-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!Array.isArray(clusters) || clusters.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.clusters_correlation.empty_chart">${esc(t('view.clusters_correlation.empty_chart'))}</div>`;
        return;
    }
    const xs = clusters.map((_, i) => i + 1);
    const net = clusters.map(c => Number.isFinite(c.net_exposure) ? c.net_exposure : null);
    const labels = clusters.map((_, i) => `c${i}`);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.clusters_correlation.chart.cluster_idx') },
            { label: t('view.clusters_correlation.chart.net_exposure'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.clusters_correlation.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, net, zero], el);
}

function showErr(msg) {
    const el = document.getElementById('cc-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('cc-err').style.display = 'none'; }
