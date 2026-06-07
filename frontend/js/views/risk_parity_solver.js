// Risk-parity weights solver view — Spinu (2013) fixed-point ERC.
// Full covariance matrix input; finds weights that equalize each
// asset's contribution to portfolio variance.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_MAX_ITER, DEFAULT_TOLERANCE,
    parseMatrix, validateInputs, buildBody, localSolve,
    convergenceBadge, rcBadge,
    makeDemoInput, fmtPct, fmtNum, fmtInt, fmtSci, assetLabel, matrixToBlob,
} from '../_risk_parity_solver_inputs.js';

let state = { ...makeDemoInput('60-40-style') };

export async function renderRiskParitySolver(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.rp_solver.h1.title" class="view-title">// RISK-PARITY SOLVER (ERC)</h1>

        <div class="chart-panel" data-context-scope="risk-parity-solver">
            <h2 data-i18n="view.rp_solver.h2.cov">Covariance matrix
                <small data-i18n="view.rp_solver.h2.cov_hint" class="muted">(one row per line; comma/whitespace cells; must be square + symmetric)</small></h2>
            <textarea id="rps-cov" rows="6"
                      data-tip="view.rp_solver.tip.cov"
                      placeholder="0.04, 0.01, 0.005&#10;0.01, 0.09, 0.02&#10;0.005, 0.02, 0.16">${esc(matrixToBlob(state.covariance))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.rp_solver.label.max_iter">Max iterations</span>
                    <input id="rps-max" type="number" step="1" min="1" value="${state.max_iter}" data-tip="view.rp_solver.tip.max_iter"></label>
                <label><span data-i18n="view.rp_solver.label.tolerance">Tolerance</span>
                    <input id="rps-tol" type="number" step="0.01" min="0" value="${state.tolerance}" data-tip="view.rp_solver.tip.tolerance"></label>
                <button data-i18n="view.rp_solver.btn.solve" id="rps-run" class="primary"
                        data-tip="view.rp_solver.tip.solve" data-shortcut="risk_parity_solver_run" type="button">Solve</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.rp_solver.btn.demo_equal"     id="rps-demo-eq"    class="secondary" type="button" data-tip="view.rp_solver.tip.demo_eq">Demo: equal-vol uncorrelated</button>
                <button data-i18n="view.rp_solver.btn.demo_pair"      id="rps-demo-pair"  class="secondary" type="button" data-tip="view.rp_solver.tip.demo_pair">Demo: 2-asset high-vol pair</button>
                <button data-i18n="view.rp_solver.btn.demo_6040"      id="rps-demo-6040"  class="secondary" type="button" data-tip="view.rp_solver.tip.demo_6040">Demo: 60/40-style (3-asset)</button>
                <button data-i18n="view.rp_solver.btn.demo_corr"      id="rps-demo-corr"  class="secondary" type="button" data-tip="view.rp_solver.tip.demo_corr">Demo: high-correlation</button>
                <button data-i18n="view.rp_solver.btn.demo_diversifier" id="rps-demo-div" class="secondary" type="button" data-tip="view.rp_solver.tip.demo_div">Demo: 4-asset w/ diversifier</button>
                <button data-i18n="view.rp_solver.btn.demo_small"     id="rps-demo-small" class="secondary" type="button" data-tip="view.rp_solver.tip.demo_small">Demo: small 2-asset</button>
                <button data-i18n="view.rp_solver.btn.demo_tight"     id="rps-demo-tight" class="secondary" type="button" data-tip="view.rp_solver.tip.demo_tight">Demo: tight tolerance</button>
                <button data-i18n="view.rp_solver.btn.demo_loose"     id="rps-demo-loose" class="secondary" type="button" data-tip="view.rp_solver.tip.demo_loose">Demo: loose tolerance (may not converge)</button>
            </div>
            <p data-i18n="view.rp_solver.hint.about" class="muted">Spinu (2013) fixed-point ERC. Each asset's contribution to portfolio variance is equalized — concentration is risk-weighted, not dollar-weighted. Used by Bridgewater All Weather, AQR, others. Converges in O(n²) per iteration.</p>
        </div>

        <div id="rps-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.rp_solver.h2.weights">Solved weights vs equal-weight</h2>
            <div id="rps-table"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.rp_solver.h2.weights_chart">RP weight vs 1/N per asset</h2>
            <div id="rps-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.rp_solver.h2.rc_chart">Risk contribution share per asset (% of σ)</h2>
            <div id="rps-rc-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.rp_solver.hint.rc_chart" class="muted small">Each asset's risk contribution as a fraction of total portfolio volatility. At ERC convergence every bar sits on the yellow 1/N reference. Bars above the line = risk-overweight; below = risk-underweight.</p>
        </div>

        <div id="rps-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('rps-cov').value = matrixToBlob(state.covariance);
        document.getElementById('rps-max').value = state.max_iter;
        document.getElementById('rps-tol').value = state.tolerance;
    };
    document.getElementById('rps-demo-eq').addEventListener('click',    () => { loadDemo('equal-vol-uncorr'); void compute(tok); });
    document.getElementById('rps-demo-pair').addEventListener('click',  () => { loadDemo('high-vol-pair');    void compute(tok); });
    document.getElementById('rps-demo-6040').addEventListener('click',  () => { loadDemo('60-40-style');      void compute(tok); });
    document.getElementById('rps-demo-corr').addEventListener('click',  () => { loadDemo('high-correlation'); void compute(tok); });
    document.getElementById('rps-demo-div').addEventListener('click',   () => { loadDemo('diversifier');      void compute(tok); });
    document.getElementById('rps-demo-small').addEventListener('click', () => { loadDemo('small-pair');       void compute(tok); });
    document.getElementById('rps-demo-tight').addEventListener('click', () => { loadDemo('tight-tolerance');  void compute(tok); });
    document.getElementById('rps-demo-loose').addEventListener('click', () => { loadDemo('loose-tolerance');  void compute(tok); });
    document.getElementById('rps-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseMatrix(document.getElementById('rps-cov').value);
    if (p.errors.length) {
        showErr(`${t('view.rp_solver.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        showToast(t('view.rp_solver.toast.parse_error', { n: p.errors.length }), { level: 'warning' });
        return;
    }
    hideErr();
    state.covariance = p.matrix;
    const m = parseInt(document.getElementById('rps-max').value, 10);
    const tol = Number(document.getElementById('rps-tol').value);
    state.max_iter   = Number.isFinite(m) && m >= 1 ? m : DEFAULT_MAX_ITER;
    state.tolerance  = Number.isFinite(tol) && tol > 0 ? tol : DEFAULT_TOLERANCE;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); showToast(t('view.rp_solver.toast.invalid'), { level: 'warning' }); return; }
    const local = localSolve(state.covariance, state.max_iter, state.tolerance);
    if (!local) {
        showErr(t('view.rp_solver.err.degenerate'));
        showToast(t('view.rp_solver.toast.degenerate'), { level: 'warning' });
        return;
    }
    renderSummary(local, true);
    renderTable(local);
    let resp;
    try {
        resp = await api.portfolioRiskParityWeights(buildBody(state));
    } catch (e) {
        showErr(`${t('view.rp_solver.err.api')}: ${e.message || e}`);
        showToast(t('view.rp_solver.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) {
        showErr(t('view.rp_solver.err.server_rejected'));
        showToast(t('view.rp_solver.toast.rejected'), { level: 'error' });
        return;
    }
    renderSummary(resp, false);
    renderTable(resp);
    renderWeightsChart(resp);
    renderRcChart(resp);
    const n = (resp.weights || []).length;
    const iter = resp.iterations | 0;
    const conv = !!resp.converged;
    const level = conv ? 'success' : 'warning';
    showToast(t('view.rp_solver.toast.solved', {
        n, iter,
        status: conv ? 'CONVERGED' : 'no-convergence',
    }), { level });
}

function renderWeightsChart(report) {
    const el = document.getElementById('rps-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!report || !report.weights || !report.weights.length) {
        el.innerHTML = `<div class="muted" data-i18n="view.rp_solver.empty_chart">${esc(t('view.rp_solver.empty_chart'))}</div>`;
        return;
    }
    const n = report.weights.length;
    const eq = 1 / n;
    const labels = report.weights.map((_, i) => assetLabel(i));
    const rp = report.weights.map(w => Number(w) * 100);
    const eqs = labels.map(() => eq * 100);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.rp_solver.chart.asset_idx') },
            { label: t('view.rp_solver.chart.rp_weight'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 14, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.rp_solver.chart.eq_weight'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, rp, eqs], el);
}

function renderRcChart(report) {
    const el = document.getElementById('rps-rc-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!report || !report.risk_contributions || !report.risk_contributions.length || !(report.portfolio_volatility > 0)) {
        el.innerHTML = `<div class="muted" data-i18n="view.rp_solver.empty_rc_chart">${esc(t('view.rp_solver.empty_rc_chart'))}</div>`;
        return;
    }
    const n = report.risk_contributions.length;
    const eq = 100 / n;
    const labels = report.risk_contributions.map((_, i) => assetLabel(i));
    const rcs = report.risk_contributions.map(rc => (Number(rc) / report.portfolio_volatility) * 100);
    const eqs = labels.map(() => eq);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.rp_solver.chart.asset_idx') },
            { label: t('view.rp_solver.chart.rc_pct'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 14, fill: '#b86bff', stroke: '#b86bff' } },
            { label: t('view.rp_solver.chart.eq_rc'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40,
              values: (_u, splits) => splits.map(v => v.toFixed(1) + '%') },
        ],
        legend: { show: true },
    }, [xs, rcs, eqs], el);
}

function renderSummary(report, pending) {
    const local = localSolve(state.covariance, state.max_iter, state.tolerance);
    const n = state.covariance.length;
    const parityOk = !!local && local.weights.length === report.weights.length
        && local.weights.every((w, i) => Math.abs(w - report.weights[i]) < 1e-6)
        && local.converged === report.converged;
    const badge = convergenceBadge(report);
    const localTag = pending ? ` (${t('view.rp_solver.tag.local')})` : '';
    let mxW = -Infinity, mnW = Infinity;
    for (const w of report.weights) { if (w > mxW) mxW = w; if (w < mnW) mnW = w; }
    document.getElementById('rps-summary').innerHTML = [
        card(t('view.rp_solver.card.verdict'),    t(badge.key) + localTag, badge.cls),
        card(t('view.rp_solver.card.assets'),     fmtInt(n)),
        card(t('view.rp_solver.card.iterations'), fmtInt(report.iterations)),
        card(t('view.rp_solver.card.converged'),
             report.converged ? t('view.rp_solver.tag.yes') : t('view.rp_solver.tag.no'),
             report.converged ? 'pos' : 'neg'),
        card(t('view.rp_solver.card.max_dev'),    fmtSci(report.max_contribution_deviation),
             report.max_contribution_deviation < state.tolerance ? 'pos' : 'neg'),
        card(t('view.rp_solver.card.port_vol'),   fmtPct(report.portfolio_volatility, 4)),
        card(t('view.rp_solver.card.max_weight'), fmtPct(mxW)),
        card(t('view.rp_solver.card.min_weight'), fmtPct(mnW)),
        card(t('view.rp_solver.card.parity'),
             parityOk ? t('view.rp_solver.tag.ok') : t('view.rp_solver.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderTable(report) {
    const wrap = document.getElementById('rps-table');
    if (!report || !report.weights || report.weights.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.rp_solver.empty">${esc(t('view.rp_solver.empty'))}</div>`;
        return;
    }
    const n = report.weights.length;
    const eq = 1 / n;
    const portVol = report.portfolio_volatility;
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.rp_solver.col.asset">Asset</th>
                <th data-i18n="view.rp_solver.col.vol">σ (sqrt diag)</th>
                <th data-i18n="view.rp_solver.col.weight">RP weight</th>
                <th data-i18n="view.rp_solver.col.eq_weight">1/N weight</th>
                <th data-i18n="view.rp_solver.col.delta">Δ vs 1/N</th>
                <th data-i18n="view.rp_solver.col.rc">Risk contrib</th>
                <th data-i18n="view.rp_solver.col.rc_pct">% of port σ</th>
                <th data-i18n="view.rp_solver.col.rc_verdict">RC verdict</th>
            </tr></thead>
            <tbody>
                ${report.weights.map((w, i) => {
                    const sigma_i = Math.sqrt(Math.max(0, state.covariance[i][i]));
                    const delta = w - eq;
                    const rc = report.risk_contributions[i];
                    const rcPct = portVol > 0 ? rc / portVol : 0;
                    const verdict = rcBadge(rc, portVol, n);
                    return `<tr>
                        <td><strong>${esc(assetLabel(i))}</strong></td>
                        <td>${esc(fmtPct(sigma_i))}</td>
                        <td>${esc(fmtPct(w))}</td>
                        <td>${esc(fmtPct(eq))}</td>
                        <td class="${delta > 0 ? 'pos' : delta < 0 ? 'neg' : ''}">${esc(fmtPct(delta))}</td>
                        <td>${esc(fmtNum(rc))}</td>
                        <td>${esc(fmtPct(rcPct))}</td>
                        <td data-i18n="${esc(verdict.key)}" class="${verdict.cls}">${esc(t(verdict.key))}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('rps-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('rps-err').style.display = 'none'; }
