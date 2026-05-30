// Cholesky decomposition view — A = L · Lᵀ for SPD matrices.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseMatrixBlob, matrixToBlob, validateInputs, buildBody, localDecompose,
    statusBadge, conditionBadge, offDiagScale, reconstructionError, summarizeMatrix,
    makeDemoInput,
    fmtNum, fmtNumSigned, fmtInt, fmtSci,
} from '../_cholesky_inputs.js';

let state = { ...makeDemoInput('kershaw') };

export async function renderCholesky(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.chol.h1.title" class="view-title">// CHOLESKY DECOMPOSITION</h1>

        <div class="chart-panel" data-context-scope="cholesky">
            <h2 data-i18n="view.chol.h2.matrix">Matrix A (symmetric positive-definite)
                <small data-i18n="view.chol.h2.matrix_hint" class="muted">(one row per line; n×n with n ≤ 50)</small></h2>
            <textarea id="ch-blob" rows="8"
                      data-tip="view.chol.tip.matrix"
                      placeholder="4 12 -16\n12 37 -43\n-16 -43 98">${esc(matrixToBlob(state.matrix))}</textarea>

            <div class="inline-form">
                <button data-i18n="view.chol.btn.compute" id="ch-run" class="primary"
                        data-tip="view.chol.tip.compute" type="button">Decompose</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.chol.btn.demo_kershaw" id="ch-d1" class="secondary" type="button">Demo: Kershaw 3×3</button>
                <button data-i18n="view.chol.btn.demo_identity" id="ch-d2" class="secondary" type="button">Demo: identity 4×4</button>
                <button data-i18n="view.chol.btn.demo_diag" id="ch-d3" class="secondary" type="button">Demo: diagonal 3×3</button>
                <button data-i18n="view.chol.btn.demo_corr2" id="ch-d4" class="secondary" type="button">Demo: corr 2×2</button>
                <button data-i18n="view.chol.btn.demo_cov3" id="ch-d5" class="secondary" type="button">Demo: cov 3×3</button>
                <button data-i18n="view.chol.btn.demo_npd" id="ch-d6" class="secondary" type="button">Demo: not PD</button>
                <button data-i18n="view.chol.btn.demo_asym" id="ch-d7" class="secondary" type="button">Demo: asymmetric</button>
                <button data-i18n="view.chol.btn.demo_large" id="ch-d8" class="secondary" type="button">Demo: cov 5×5</button>
            </div>
            <p data-i18n="view.chol.hint.about" class="muted">Lower-triangular factorization A = L · Lᵀ for symmetric positive-definite matrices. Used in correlated Monte Carlo (L·z transforms uncorrelated draws to a target covariance) and stable matrix inversion. Returns None if A is not symmetric or not positive-definite.</p>
        </div>

        <div id="ch-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.chol.h2.factor">Factor L (lower-triangular)</h2>
            <div id="ch-factor"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.chol.h2.stats">Matrix A summary</h2>
            <div id="ch-stats"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.chol.h2.diag_chart">L diagonal (conditional std devs)</h2>
            <div id="ch-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.chol.h2.coupling_chart">Per-row max |L[i][j]| for j&lt;i (coupling strength to prior variables)</h2>
            <div id="ch-coupling-chart" style="width:100%;height:220px"></div>
        </div>

        <div id="ch-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('ch-blob').value = matrixToBlob(state.matrix);
    };
    document.getElementById('ch-d1').addEventListener('click', () => { loadDemo('kershaw');         void compute(tok); });
    document.getElementById('ch-d2').addEventListener('click', () => { loadDemo('identity-4');     void compute(tok); });
    document.getElementById('ch-d3').addEventListener('click', () => { loadDemo('diagonal-3');     void compute(tok); });
    document.getElementById('ch-d4').addEventListener('click', () => { loadDemo('correlation-2x2'); void compute(tok); });
    document.getElementById('ch-d5').addEventListener('click', () => { loadDemo('covariance-3x3'); void compute(tok); });
    document.getElementById('ch-d6').addEventListener('click', () => { loadDemo('not-pd');          void compute(tok); });
    document.getElementById('ch-d7').addEventListener('click', () => { loadDemo('asymmetric');     void compute(tok); });
    document.getElementById('ch-d8').addEventListener('click', () => { loadDemo('large-cov-5');     void compute(tok); });
    document.getElementById('ch-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseMatrixBlob(document.getElementById('ch-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.chol.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.matrix = p.matrix;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localDecompose(state.matrix);
    renderSummary(local, true);
    renderFactor(local);
    renderStats();
    renderDiagChart(local);
    renderCouplingChart(local);
    let resp;
    try {
        resp = await api.anlyCholesky(buildBody(state));
    } catch (e) {
        showErr(`${t('view.chol.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    // resp may be null when A is not PD — that's a valid response, render the summary anyway.
    renderSummary(resp, false);
    renderFactor(resp);
    renderStats();
    renderDiagChart(resp);
    renderCouplingChart(resp);
}

function renderSummary(report, pending) {
    const local = localDecompose(state.matrix);
    const parityOk = (local == null && report == null)
        || (!!local && !!report
            && local.l.length === report.l.length
            && Math.abs(local.sqrt_determinant - report.sqrt_determinant) < 1e-9
            && local.l.every((row, i) => row.every((v, j) =>
                (v == null && report.l[i][j] == null) || Math.abs(v - report.l[i][j]) < 1e-9)));
    const sBadge = statusBadge(report);
    const cBadge = conditionBadge(report);
    const offScale = report ? offDiagScale(report.l) : NaN;
    const recErr = report ? reconstructionError(state.matrix, report.l) : NaN;
    const n = state.matrix.length;
    const det = report ? report.sqrt_determinant * report.sqrt_determinant : NaN;
    const localTag = pending ? ` (${t('view.chol.tag.local')})` : '';
    document.getElementById('ch-summary').innerHTML = [
        card(t('view.chol.card.status'),    t(sBadge.key) + localTag, sBadge.cls),
        card(t('view.chol.card.condition'), t(cBadge.key), cBadge.cls),
        card(t('view.chol.card.dim'),       `${n} × ${n}`),
        card(t('view.chol.card.sqrt_det'),  fmtNum(report ? report.sqrt_determinant : NaN)),
        card(t('view.chol.card.det'),       fmtNum(det)),
        card(t('view.chol.card.off_scale'), fmtNum(offScale)),
        card(t('view.chol.card.rec_error'), fmtSci(recErr)),
        card(t('view.chol.card.parity'),
             parityOk ? t('view.chol.tag.ok') : t('view.chol.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderFactor(report) {
    const wrap = document.getElementById('ch-factor');
    if (!report) {
        wrap.innerHTML = `<div class="muted neg" data-i18n="view.chol.no_factor">${esc(t('view.chol.no_factor'))}</div>`;
        return;
    }
    const n = report.l.length;
    const rows = report.l.map((row, i) => {
        const cells = row.map((v, j) => {
            if (j > i) return `<td class="muted">—</td>`;
            return `<td>${esc(fmtNumSigned(v))}</td>`;
        }).join('');
        return `<tr><td><strong>L<sub>${i + 1}</sub></strong></td>${cells}</tr>`;
    }).join('');
    const header = `<th></th>` + Array.from({ length: n }, (_, j) =>
        `<th>${j + 1}</th>`).join('');
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>${header}</tr></thead>
            <tbody>${rows}</tbody>
        </table>
    `;
}

function renderStats() {
    const wrap = document.getElementById('ch-stats');
    if (!state.matrix.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.chol.empty">${esc(t('view.chol.empty'))}</div>`;
        return;
    }
    const s = summarizeMatrix(state.matrix);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.chol.col.metric">Metric</th>
                <th data-i18n="view.chol.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.chol.row.n">Dimension</td>      <td>${fmtInt(s.n)}</td></tr>
                <tr><td data-i18n="view.chol.row.trace">Trace</td>     <td>${esc(fmtNum(s.trace))}</td></tr>
                <tr><td data-i18n="view.chol.row.max_diag">Max diag</td><td>${esc(fmtNum(s.max_diag))}</td></tr>
                <tr><td data-i18n="view.chol.row.min_diag">Min diag</td><td>${esc(fmtNum(s.min_diag))}</td></tr>
                <tr><td data-i18n="view.chol.row.max_off">Max abs off-diag</td><td>${esc(fmtNum(s.max_abs_off))}</td></tr>
            </tbody>
        </table>
    `;
}

function renderDiagChart(report) {
    const el = document.getElementById('ch-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!report || !Array.isArray(report.l) || !report.l.length) {
        el.innerHTML = `<div class="muted" data-i18n="view.chol.empty_chart">${esc(t('view.chol.empty_chart'))}</div>`;
        return;
    }
    const diag = report.l.map((row, i) => Number.isFinite(row[i]) ? row[i] : null);
    const xs = diag.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.chol.chart.row_idx') },
            { label: t('view.chol.chart.diag'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 10, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => String(Math.round(v))) },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, diag], el);
}

function renderCouplingChart(report) {
    const el = document.getElementById('ch-coupling-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    if (!report || !Array.isArray(report.l) || !report.l.length) {
        el.innerHTML = `<div class="muted" data-i18n="view.chol.empty_coupling_chart">${esc(t('view.chol.empty_coupling_chart'))}</div>`;
        return;
    }
    const coupling = report.l.map((row, i) => {
        if (i === 0) return 0;
        let m = 0;
        for (let j = 0; j < i; j++) {
            const v = row[j];
            if (Number.isFinite(v) && Math.abs(v) > m) m = Math.abs(v);
        }
        return m;
    });
    const xs = coupling.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.chol.chart.row_idx') },
            { label: t('view.chol.chart.coupling'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 10, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => String(Math.round(v))) },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, coupling], el);
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('ch-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ch-err').style.display = 'none'; }
