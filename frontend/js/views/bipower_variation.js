// Bipower Variation view — jump-robust volatility + Huang-Tauchen jump test.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseReturnsBlob, returnsToBlob, validateInputs, buildBody, localCompute,
    jumpBadge, jumpFractionBadge, jumpRatio,
    makeDemoInput,
    fmtVar, fmtZ, fmtP, fmtPct, fmtInt,
} from '../_bipower_variation_inputs.js';

let state = { ...makeDemoInput('no-jumps') };

export async function renderBipowerVariation(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.bpv.h1.title" class="view-title">// BIPOWER VARIATION</h1>

        <div class="chart-panel" data-context-scope="bpv">
            <h2 data-i18n="view.bpv.h2.returns">Returns
                <small data-i18n="view.bpv.h2.returns_hint" class="muted">(one per token; comma/whitespace; decimal 0.012 or "1.2%")</small></h2>
            <textarea id="bv-blob" rows="6"
                      data-tip="view.bpv.tip.returns"
                      placeholder="0.012&#10;-0.005&#10;0.50  # jump">${esc(returnsToBlob(state.returns))}</textarea>

            <div class="inline-form">
                <button data-i18n="view.bpv.btn.compute" id="bv-run" class="primary"
                        data-tip="view.bpv.tip.compute" type="button">Compute BPV</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.bpv.btn.demo_smooth"  id="bv-demo-smooth"  class="secondary" type="button">Demo: smooth no-jumps</button>
                <button data-i18n="view.bpv.btn.demo_big"     id="bv-demo-big"     class="secondary" type="button">Demo: single big jump (50%)</button>
                <button data-i18n="view.bpv.btn.demo_multi"   id="bv-demo-multi"   class="secondary" type="button">Demo: multiple small jumps</button>
                <button data-i18n="view.bpv.btn.demo_flat"    id="bv-demo-flat"    class="secondary" type="button">Demo: flat zero returns</button>
                <button data-i18n="view.bpv.btn.demo_highvol" id="bv-demo-hi"      class="secondary" type="button">Demo: high vol, no jumps</button>
                <button data-i18n="view.bpv.btn.demo_crash"   id="bv-demo-crash"   class="secondary" type="button">Demo: −30% crash bar</button>
                <button data-i18n="view.bpv.btn.demo_short"   id="bv-demo-short"   class="secondary" type="button">Demo: short series (5 bars)</button>
                <button data-i18n="view.bpv.btn.demo_persist" id="bv-demo-persist" class="secondary" type="button">Demo: persistent vol</button>
            </div>
            <p data-i18n="view.bpv.hint.about" class="muted">BPV = (π/2)·Σ|r_i|·|r_{i−1}|. Jump-robust IV estimator. Jump variation = max(0, RV − BPV). Huang-Tauchen z = √n·(RV−BPV)/BPV/√(θ·max(1, TQ/BPV²)) ~ N(0, 1) under H₀ (no jumps).</p>
        </div>

        <div id="bv-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.bpv.h2.chart">|r_t| × |r_{t−1}| series (BPV summands)</h2>
            <div id="bv-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.bpv.h2.table">Component breakdown</h2>
            <div id="bv-table"></div>
        </div>

        <div id="bv-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('bv-blob').value = returnsToBlob(state.returns);
    };
    document.getElementById('bv-demo-smooth').addEventListener('click',  () => { loadDemo('no-jumps');           void compute(tok); });
    document.getElementById('bv-demo-big').addEventListener('click',     () => { loadDemo('single-big-jump');   void compute(tok); });
    document.getElementById('bv-demo-multi').addEventListener('click',   () => { loadDemo('multi-small-jumps'); void compute(tok); });
    document.getElementById('bv-demo-flat').addEventListener('click',    () => { loadDemo('flat-zero');          void compute(tok); });
    document.getElementById('bv-demo-hi').addEventListener('click',      () => { loadDemo('high-vol-no-jumps'); void compute(tok); });
    document.getElementById('bv-demo-crash').addEventListener('click',   () => { loadDemo('crash-down');         void compute(tok); });
    document.getElementById('bv-demo-short').addEventListener('click',   () => { loadDemo('short-series');       void compute(tok); });
    document.getElementById('bv-demo-persist').addEventListener('click', () => { loadDemo('persistent-vol');     void compute(tok); });
    document.getElementById('bv-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseReturnsBlob(document.getElementById('bv-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.bpv.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.returns = p.returns;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.returns);
    if (!local) { showErr(t('view.bpv.err.degenerate')); return; }
    renderSummary(local, true);
    renderChart();
    renderTable(local);
    let resp;
    try {
        resp = await api.anlyBipowerVariation(buildBody(state));
    } catch (e) {
        showErr(`${t('view.bpv.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) { showErr(t('view.bpv.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart();
    renderTable(resp);
}

function renderSummary(report, pending) {
    const local = localCompute(state.returns);
    const parityOk = !!local
        && Math.abs(local.realized_variance - report.realized_variance) < 1e-9
        && Math.abs(local.bipower_variation - report.bipower_variation) < 1e-9
        && Math.abs(local.jump_test_z - report.jump_test_z) < 1e-6;
    const jBadge = jumpBadge(report.jump_test_p_value);
    const fBadge = jumpFractionBadge(report.realized_variance, report.jump_variation);
    const frac = jumpRatio(report.realized_variance, report.jump_variation);
    const localTag = pending ? ` (${t('view.bpv.tag.local')})` : '';
    document.getElementById('bv-summary').innerHTML = [
        card(t('view.bpv.card.verdict'),     t(jBadge.key) + localTag, jBadge.cls),
        card(t('view.bpv.card.fraction'),    t(fBadge.key), fBadge.cls),
        card(t('view.bpv.card.n'),           fmtInt(report.n_observations)),
        card(t('view.bpv.card.rv'),          fmtVar(report.realized_variance)),
        card(t('view.bpv.card.bpv'),         fmtVar(report.bipower_variation)),
        card(t('view.bpv.card.jump_var'),    fmtVar(report.jump_variation),
             report.jump_variation > 0 ? 'neg' : ''),
        card(t('view.bpv.card.jump_pct'),    fmtPct(frac), fBadge.cls),
        card(t('view.bpv.card.tq'),          fmtVar(report.tripower_quarticity)),
        card(t('view.bpv.card.z'),           fmtZ(report.jump_test_z), jBadge.cls),
        card(t('view.bpv.card.p'),           fmtP(report.jump_test_p_value), jBadge.cls),
        card(t('view.bpv.card.parity'),
             parityOk ? t('view.bpv.tag.ok') : t('view.bpv.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart() {
    if (!window.uPlot) return;
    const el = document.getElementById('bv-chart');
    if (!el) return;
    el.innerHTML = '';
    if (state.returns.length < 2) {
        el.innerHTML = `<div class="muted" data-i18n="view.bpv.empty">${esc(t('view.bpv.empty'))}</div>`;
        return;
    }
    // Per-bar BPV summand |r_t| × |r_{t-1}| (the visible jump-robust contribution).
    const xs = state.returns.map((_, i) => i);
    const summand = state.returns.map((r, i) => i === 0
        ? null
        : Math.abs(r) * Math.abs(state.returns[i - 1]));
    // RV per-bar = r².
    const rv = state.returns.map(r => r * r);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 340,
        scales: { x: {}, y: {} },
        series: [
            { label: 'bar' },
            { label: 'r_t²',          stroke: '#ff3860', width: 1.0, dash: [4, 4], points: { show: false } },
            { label: '|r_t|·|r_{t-1}|', stroke: '#00e5ff', width: 1.5, points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 70,
              values: (_u, splits) => splits.map(v => Math.abs(v) < 1e-4 ? v.toExponential(1) : v.toFixed(4)) },
        ],
        legend: { show: true },
    }, [xs, rv, summand], el);
}

function renderTable(report) {
    const wrap = document.getElementById('bv-table');
    if (!report) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.bpv.empty">${esc(t('view.bpv.empty'))}</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.bpv.col.metric">Metric</th>
                <th data-i18n="view.bpv.col.value">Value</th>
                <th data-i18n="view.bpv.col.note">Note</th>
            </tr></thead>
            <tbody>
                <tr><td><strong data-i18n="view.bpv.row.rv">RV (realized variance)</strong></td>
                    <td>${esc(fmtVar(report.realized_variance))}</td>
                    <td data-i18n="view.bpv.row.rv_note">${esc(t('view.bpv.row.rv_note'))}</td></tr>
                <tr><td><strong data-i18n="view.bpv.row.bpv">BPV (bipower variation)</strong></td>
                    <td>${esc(fmtVar(report.bipower_variation))}</td>
                    <td data-i18n="view.bpv.row.bpv_note">${esc(t('view.bpv.row.bpv_note'))}</td></tr>
                <tr><td><strong data-i18n="view.bpv.row.jump">Jump variation</strong></td>
                    <td class="${report.jump_variation > 0 ? 'neg' : ''}">${esc(fmtVar(report.jump_variation))}</td>
                    <td data-i18n="view.bpv.row.jump_note">${esc(t('view.bpv.row.jump_note'))}</td></tr>
                <tr><td><strong data-i18n="view.bpv.row.tq">TQ (tripower quarticity)</strong></td>
                    <td>${esc(fmtVar(report.tripower_quarticity))}</td>
                    <td data-i18n="view.bpv.row.tq_note">${esc(t('view.bpv.row.tq_note'))}</td></tr>
                <tr><td><strong data-i18n="view.bpv.row.z">Huang-Tauchen z</strong></td>
                    <td>${esc(fmtZ(report.jump_test_z))}</td>
                    <td data-i18n="view.bpv.row.z_note">${esc(t('view.bpv.row.z_note'))}</td></tr>
                <tr><td><strong data-i18n="view.bpv.row.p">One-sided p-value</strong></td>
                    <td>${esc(fmtP(report.jump_test_p_value))}</td>
                    <td data-i18n="view.bpv.row.p_note">${esc(t('view.bpv.row.p_note'))}</td></tr>
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
    const el = document.getElementById('bv-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('bv-err').style.display = 'none'; }
