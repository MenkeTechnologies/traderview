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

let state = {
    positions: makeDemoPositions('mega-cap-tech'),
    correlations: makeDemoCorrelations('mega-cap-tech'),
    threshold: 0.7,
};

export async function renderClustersCorrelation(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// CORRELATION CLUSTERS</h1>

        <div class="chart-panel">
            <h2>Positions <small class="muted">(per line: <code>SYMBOL notional</code>, neg = short)</small></h2>
            <textarea id="cc-pos" rows="5" placeholder="AAPL 20000&#10;MSFT 15000">${esc(positionsToBlob(state.positions))}</textarea>
            <h2>Correlations <small class="muted">(per line: <code>A B corr</code>, order doesn't matter)</small></h2>
            <textarea id="cc-corr" rows="6" placeholder="AAPL MSFT 0.85">${esc(correlationsToBlob(state.correlations))}</textarea>
            <div class="inline-form">
                <label>Threshold |ρ|
                    <input id="cc-thr" type="number" step="any" min="0" max="1" value="${state.threshold}"></label>
                <button id="cc-run" class="primary" type="button">Cluster</button>
            </div>
            <div class="inline-form">
                <button id="cc-demo-tech"     class="secondary" type="button">Demo: mega-cap-tech cluster</button>
                <button id="cc-demo-inverse"  class="secondary" type="button">Demo: inverse pair (QQQ/SQQQ)</button>
                <button id="cc-demo-chain"    class="secondary" type="button">Demo: A-B-C transitive chain</button>
                <button id="cc-demo-diverse"  class="secondary" type="button">Demo: all singletons</button>
            </div>
            <p class="muted">Positions cluster if there's a chain of pairs each with |ρ| ≥ threshold. Threshold is inclusive (≥). Missing pairs default to ρ = 0.</p>
        </div>

        <div id="cc-summary" class="cards"></div>

        <div class="chart-panel">
            <h2>Clusters (sorted by gross exposure)</h2>
            <div id="cc-clusters"></div>
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
    if (errs.length) { showErr(errs.slice(0, 4).join('; ')); return; }
    hideErr();
    state.positions    = pPos.positions;
    state.correlations = pCorr.correlations;
    state.threshold    = Number(document.getElementById('cc-thr').value);
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.positions, state.correlations, state.threshold);
    if (err) { showErr(err); return; }
    const local = localCluster(state.positions, state.correlations, state.threshold);
    renderSummary(local, true);
    renderClusters(local);
    let resp;
    try {
        resp = await api.clustersCorrelation(buildBody(state.positions, state.correlations, state.threshold));
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderClusters(resp);
}

function renderSummary(clusters, pending) {
    const s = summarize(clusters);
    const local = localCluster(state.positions, state.correlations, state.threshold);
    const parity = clustersEq(clusters, local);
    const badge = concentrationBadge(s.topPct);
    document.getElementById('cc-summary').innerHTML = [
        card('Concentration',  badge.label + (pending ? ' (local)' : ''), badge.cls),
        card('Action',         badge.hint),
        card('Positions',      String(state.positions.length)),
        card('Clusters',       String(s.nClusters)),
        card('Singletons',     String(s.singletons),
            s.singletons === state.positions.length ? 'pos' : ''),
        card('Largest cluster size', String(s.maxClusterSize),
            s.maxClusterSize >= 4 ? 'neg' : ''),
        card('Top cluster %',  fmtPct(s.topPct), badge.cls),
        card('Total gross',    fmtUSD(s.totalGross)),
        card('Total net',      fmtUSDSigned(s.totalNet),
            s.totalNet >= 0 ? 'pos' : 'neg'),
        card('Local parity',   parity ? 'OK' : 'DIVERGED', parity ? 'pos' : 'neg'),
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
    if (!clusters.length) { wrap.innerHTML = '<div class="muted">No clusters.</div>'; return; }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th>#</th><th>Members</th><th>Size</th>
                <th>Gross exposure</th><th>Net exposure</th><th>Direction</th>
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

function showErr(msg) {
    const el = document.getElementById('cc-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('cc-err').style.display = 'none'; }
