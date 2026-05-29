// Black-Litterman view — combines equilibrium returns with subjective
// views to produce posterior returns + covariance.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_TAU,
    parseBlackLittermanBlob, blToBlob, validateInputs, buildBody, localSolve,
    confidenceBadge, tiltBadge,
    makeDemoInput,
    fmtPctSigned, fmtPct, fmtNum, fmtSci, fmtInt, assetLabel,
} from '../_black_litterman_inputs.js';

let state = { ...makeDemoInput('two-asset-view') };

export async function renderBlackLitterman(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.blit.h1.title" class="view-title">// BLACK-LITTERMAN</h1>

        <div class="chart-panel" data-context-scope="black-litterman">
            <h2 data-i18n="view.blit.h2.blob">Model inputs
                <small data-i18n="view.blit.h2.blob_hint" class="muted">(4 sections separated by blank lines: labels, equilibrium returns, n×n covariance, "tau X". Optional 5th: views as "name p_1 .. p_n q ω")</small></h2>
            <textarea id="bl-blob" rows="14"
                      data-tip="view.blit.tip.blob"
                      placeholder="A B&#10;&#10;0.05 0.07&#10;&#10;0.04, 0.01&#10;0.01, 0.09&#10;&#10;tau 0.05&#10;&#10;view_1 1 -1 0.02 0.001">${esc(blToBlob(state.inputs))}</textarea>

            <div class="inline-form">
                <button data-i18n="view.blit.btn.compute" id="bl-run" class="primary"
                        data-tip="view.blit.tip.compute" type="button">Solve posterior</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.blit.btn.demo_two"      id="bl-demo-two"     class="secondary" type="button">Demo: 2-asset basic view</button>
                <button data-i18n="view.blit.btn.demo_no_views" id="bl-demo-no"      class="secondary" type="button">Demo: no views (posterior = prior)</button>
                <button data-i18n="view.blit.btn.demo_conf"     id="bl-demo-conf"    class="secondary" type="button">Demo: very confident (ω=1e-8)</button>
                <button data-i18n="view.blit.btn.demo_loose"    id="bl-demo-loose"   class="secondary" type="button">Demo: very loose (ω=1e8)</button>
                <button data-i18n="view.blit.btn.demo_three"    id="bl-demo-three"   class="secondary" type="button">Demo: 3-asset, 2 views</button>
                <button data-i18n="view.blit.btn.demo_conflict" id="bl-demo-conflict" class="secondary" type="button">Demo: conflicting views</button>
                <button data-i18n="view.blit.btn.demo_low_tau"  id="bl-demo-low-tau" class="secondary" type="button">Demo: low τ (0.001)</button>
                <button data-i18n="view.blit.btn.demo_hi_tau"   id="bl-demo-hi-tau"  class="secondary" type="button">Demo: large τ (0.50)</button>
            </div>
            <p data-i18n="view.blit.hint.about" class="muted">μ_bl = [(τΣ)⁻¹ + PᵀΩ⁻¹P]⁻¹ · [(τΣ)⁻¹π + PᵀΩ⁻¹Q]. Σ_bl = Σ + A⁻¹. Low ω = high confidence (pulls posterior to view). High τ = weaker prior (more weight on views).</p>
        </div>

        <div id="bl-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.blit.h2.shifts">Posterior return shifts vs equilibrium</h2>
            <div id="bl-chart" style="width:100%;height:320px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.blit.h2.table">Per-asset breakdown</h2>
            <div id="bl-table"></div>
        </div>

        <div id="bl-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('bl-blob').value = blToBlob(state.inputs);
    };
    document.getElementById('bl-demo-two').addEventListener('click',      () => { loadDemo('two-asset-view');     void compute(tok); });
    document.getElementById('bl-demo-no').addEventListener('click',       () => { loadDemo('no-views');           void compute(tok); });
    document.getElementById('bl-demo-conf').addEventListener('click',     () => { loadDemo('very-confident');     void compute(tok); });
    document.getElementById('bl-demo-loose').addEventListener('click',    () => { loadDemo('very-loose');         void compute(tok); });
    document.getElementById('bl-demo-three').addEventListener('click',    () => { loadDemo('three-asset');        void compute(tok); });
    document.getElementById('bl-demo-conflict').addEventListener('click', () => { loadDemo('two-views-conflict'); void compute(tok); });
    document.getElementById('bl-demo-low-tau').addEventListener('click',  () => { loadDemo('low-tau');            void compute(tok); });
    document.getElementById('bl-demo-hi-tau').addEventListener('click',   () => { loadDemo('large-tau');          void compute(tok); });
    document.getElementById('bl-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseBlackLittermanBlob(document.getElementById('bl-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.blit.err.parse_prefix')}: ` + p.errors.slice(0, 3).map(e => e.message).join('; '));
        return;
    }
    hideErr();
    state.inputs = {
        labels:              p.labels,
        equilibrium_returns: p.equilibrium_returns,
        covariance:          p.covariance,
        view_loadings:       p.view_loadings,
        view_returns:        p.view_returns,
        view_confidence:     p.view_confidence,
        tau:                 Number.isFinite(p.tau) && p.tau > 0 ? p.tau : DEFAULT_TAU,
        view_labels:         p.view_labels,
    };
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localSolve(state.inputs);
    if (!local) { showErr(t('view.blit.err.degenerate')); return; }
    renderSummary(local, true);
    renderChart(local);
    renderTable(local);
    let resp;
    try {
        resp = await api.portfolioBlackLitterman(buildBody(state));
    } catch (e) {
        showErr(`${t('view.blit.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) { showErr(t('view.blit.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderTable(resp);
}

function renderSummary(report, pending) {
    const local = localSolve(state.inputs);
    const parityOk = !!local
        && local.posterior_returns.length === report.posterior_returns.length
        && local.posterior_returns.every((v, i) => Math.abs(v - report.posterior_returns[i]) < 1e-6);
    const cBadge = confidenceBadge(state.inputs.view_confidence);
    const localTag = pending ? ` (${t('view.blit.tag.local')})` : '';
    let maxShift = 0;
    let maxIdx = 0;
    for (let i = 0; i < report.posterior_returns.length; i++) {
        const d = Math.abs(report.posterior_returns[i] - state.inputs.equilibrium_returns[i]);
        if (d > maxShift) { maxShift = d; maxIdx = i; }
    }
    const n = state.inputs.equilibrium_returns.length;
    const k = state.inputs.view_returns.length;
    document.getElementById('bl-summary').innerHTML = [
        card(t('view.blit.card.verdict'),     t(cBadge.key) + localTag, cBadge.cls),
        card(t('view.blit.card.n'),           fmtInt(n)),
        card(t('view.blit.card.k'),           fmtInt(k)),
        card(t('view.blit.card.tau'),         fmtNum(state.inputs.tau, 4)),
        card(t('view.blit.card.max_shift_n'),
             assetLabel(state.inputs.labels, maxIdx),
             tiltBadge(report.posterior_returns[maxIdx] - state.inputs.equilibrium_returns[maxIdx]).cls),
        card(t('view.blit.card.max_shift'),
             fmtPctSigned(report.posterior_returns[maxIdx] - state.inputs.equilibrium_returns[maxIdx]),
             tiltBadge(report.posterior_returns[maxIdx] - state.inputs.equilibrium_returns[maxIdx]).cls),
        card(t('view.blit.card.mean_post'),
             fmtPctSigned(report.posterior_returns.reduce((s, v) => s + v, 0) / n)),
        card(t('view.blit.card.parity'),
             parityOk ? t('view.blit.tag.ok') : t('view.blit.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
    void fmtPct;
    void fmtSci;
}

function renderChart(report) {
    if (!window.uPlot) return;
    const el = document.getElementById('bl-chart');
    if (!el) return;
    el.innerHTML = '';
    const n = report.posterior_returns.length;
    if (n === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.blit.empty">${esc(t('view.blit.empty'))}</div>`;
        return;
    }
    const xs = Array.from({ length: n }, (_, i) => i);
    const priors = state.inputs.equilibrium_returns.map(v => v * 100);
    const posts = report.posterior_returns.map(v => v * 100);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 320,
        scales: { x: {}, y: {} },
        series: [
            { label: t('chart.series.asset') },
            { label: 'prior π (%)',     stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: true, size: 5 } },
            { label: 'posterior μ_bl (%)', stroke: '#00e5ff', width: 1.5, points: { show: true, size: 5 } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => {
                  const i = Math.trunc(v);
                  return i >= 0 && i < n ? assetLabel(state.inputs.labels, i) : '';
              }) },
            { stroke: '#aab', size: 60,
              values: (_u, splits) => splits.map(v => v.toFixed(1) + '%') },
        ],
        legend: { show: true },
    }, [xs, priors, posts], el);
}

function renderTable(report) {
    const wrap = document.getElementById('bl-table');
    const n = report.posterior_returns.length;
    if (n === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.blit.empty">${esc(t('view.blit.empty'))}</div>`;
        return;
    }
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.blit.col.idx">#</th>
                <th data-i18n="view.blit.col.asset">Asset</th>
                <th data-i18n="view.blit.col.prior">Prior π</th>
                <th data-i18n="view.blit.col.posterior">Posterior μ_bl</th>
                <th data-i18n="view.blit.col.shift">Shift</th>
                <th data-i18n="view.blit.col.prior_vol">Prior σ</th>
                <th data-i18n="view.blit.col.posterior_vol">Posterior σ_bl</th>
                <th data-i18n="view.blit.col.verdict">Verdict</th>
            </tr></thead>
            <tbody>
                ${Array.from({ length: n }, (_, i) => {
                    const prior = state.inputs.equilibrium_returns[i];
                    const post = report.posterior_returns[i];
                    const shift = post - prior;
                    const priorSig = Math.sqrt(Math.max(0, state.inputs.covariance[i][i]));
                    const postSig = Math.sqrt(Math.max(0, report.posterior_covariance[i][i]));
                    const tBadge = tiltBadge(shift);
                    return `<tr>
                        <td>${i + 1}</td>
                        <td><strong>${esc(assetLabel(state.inputs.labels, i))}</strong></td>
                        <td>${esc(fmtPctSigned(prior))}</td>
                        <td>${esc(fmtPctSigned(post))}</td>
                        <td class="${tBadge.cls}">${esc(fmtPctSigned(shift))}</td>
                        <td>${esc(fmtPct(priorSig))}</td>
                        <td>${esc(fmtPct(postSig))}</td>
                        <td data-i18n="${esc(tBadge.key)}" class="${tBadge.cls}">${esc(t(tBadge.key))}</td>
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
    const el = document.getElementById('bl-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('bl-err').style.display = 'none'; }
