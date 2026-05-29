// Marginal / Component VaR view — risk-budgeting decomposition of a
// portfolio's tail risk at a chosen confidence level.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_Z_ALPHA, Z_CONFIDENCE_LEVELS,
    parsePortfolioBlob, portfolioToBlob, validateInputs, buildBody, localAnalyze,
    concentrationBadge, positionBadge,
    makeDemoInput,
    fmtPct, fmtPctNum, fmtNum, fmtInt, fmtSci, assetLabel,
} from '../_marginal_var_inputs.js';

let state = { ...makeDemoInput('mixed-3') };

export async function renderMarginalVar(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.mvar.h1.title" class="view-title">// MARGINAL / COMPONENT VaR</h1>

        <div class="chart-panel" data-context-scope="mvar">
            <h2 data-i18n="view.mvar.h2.portfolio">Portfolio
                <small data-i18n="view.mvar.h2.portfolio_hint" class="muted">(weights section, blank line, then k×k covariance matrix)</small></h2>
            <textarea id="mv-blob" rows="10"
                      data-tip="view.mvar.tip.portfolio"
                      placeholder="SPY 0.30&#10;EMB 0.40&#10;GLD 0.30&#10;&#10;0.04, 0.01, 0.005&#10;0.01, 0.09, 0.02&#10;0.005, 0.02, 0.16">${esc(portfolioToBlob(state.portfolio.labels, state.portfolio.weights, state.portfolio.covariance))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.mvar.label.confidence">Confidence</span>
                    <select id="mv-conf">
                        ${Z_CONFIDENCE_LEVELS.map(c => `<option value="${c.z}" ${Math.abs(c.z - state.z_alpha) < 1e-9 ? 'selected' : ''}>${esc(c.label)} (z=${c.z})</option>`).join('')}
                    </select></label>
                <label><span data-i18n="view.mvar.label.z_custom">… or custom z</span>
                    <input id="mv-z" type="number" step="any" min="0" value="${state.z_alpha}"></label>
                <button data-i18n="view.mvar.btn.compute" id="mv-run" class="primary"
                        data-tip="view.mvar.tip.compute" type="button">Analyze</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.mvar.btn.demo_mixed"   id="mv-demo-mixed"  class="secondary" type="button">Demo: mixed 3-asset (SPY/EMB/GLD)</button>
                <button data-i18n="view.mvar.btn.demo_equal"   id="mv-demo-equal"  class="secondary" type="button">Demo: 3 equal uncorrelated</button>
                <button data-i18n="view.mvar.btn.demo_conc"    id="mv-demo-conc"   class="secondary" type="button">Demo: concentrated (70% one name)</button>
                <button data-i18n="view.mvar.btn.demo_hedged"  id="mv-demo-hedged" class="secondary" type="button">Demo: fully-hedged pair (vol=0)</button>
                <button data-i18n="view.mvar.btn.demo_corr"    id="mv-demo-corr"   class="secondary" type="button">Demo: highly-correlated pair</button>
                <button data-i18n="view.mvar.btn.demo_div"     id="mv-demo-div"    class="secondary" type="button">Demo: 5-asset w/ diversifier</button>
                <button data-i18n="view.mvar.btn.demo_99"      id="mv-demo-99"     class="secondary" type="button">Demo: same, 99% z</button>
                <button data-i18n="view.mvar.btn.demo_99_9"    id="mv-demo-99_9"   class="secondary" type="button">Demo: same, 99.9% z</button>
            </div>
            <p data-i18n="view.mvar.hint.about" class="muted">marginal_i = z·(Σw)_i / vol. component_i = w_i · marginal_i. Σ components = portfolio VaR. Σ pct = 100%. Marginal answers "how does VaR change if I add one unit of position i?" Component splits total VaR by position.</p>
        </div>

        <div id="mv-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.mvar.h2.contrib">Contribution to portfolio VaR (% of total)</h2>
            <div id="mv-chart" style="width:100%;height:320px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.mvar.h2.table">Per-asset breakdown</h2>
            <div id="mv-table"></div>
        </div>

        <div id="mv-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('mv-blob').value = portfolioToBlob(state.portfolio.labels, state.portfolio.weights, state.portfolio.covariance);
        document.getElementById('mv-z').value = state.z_alpha;
        document.getElementById('mv-conf').value = state.z_alpha;
    };
    document.getElementById('mv-demo-mixed').addEventListener('click',  () => { loadDemo('mixed-3');         void compute(tok); });
    document.getElementById('mv-demo-equal').addEventListener('click',  () => { loadDemo('equal-uncorr');    void compute(tok); });
    document.getElementById('mv-demo-conc').addEventListener('click',   () => { loadDemo('concentrated');    void compute(tok); });
    document.getElementById('mv-demo-hedged').addEventListener('click', () => { loadDemo('hedged-pair');     void compute(tok); });
    document.getElementById('mv-demo-corr').addEventListener('click',   () => { loadDemo('two-asset-corr');  void compute(tok); });
    document.getElementById('mv-demo-div').addEventListener('click',    () => { loadDemo('diversifier');     void compute(tok); });
    document.getElementById('mv-demo-99').addEventListener('click',     () => { loadDemo('99-pct-vad');      void compute(tok); });
    document.getElementById('mv-demo-99_9').addEventListener('click',   () => { loadDemo('tight-99-9');      void compute(tok); });
    // Confidence dropdown updates the z input.
    document.getElementById('mv-conf').addEventListener('change', () => {
        document.getElementById('mv-z').value = document.getElementById('mv-conf').value;
    });
    document.getElementById('mv-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parsePortfolioBlob(document.getElementById('mv-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.mvar.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.portfolio.weights    = p.weights;
    state.portfolio.labels     = p.labels;
    state.portfolio.covariance = p.covariance;
    const z = Number(document.getElementById('mv-z').value);
    state.z_alpha = Number.isFinite(z) && z > 0 ? z : DEFAULT_Z_ALPHA;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localAnalyze(state.portfolio, state.z_alpha);
    if (!local) { showErr(t('view.mvar.err.degenerate')); return; }
    renderSummary(local, true);
    renderChart(local);
    renderTable(local);
    let resp;
    try {
        resp = await api.portfolioMarginalVar(buildBody(state));
    } catch (e) {
        showErr(`${t('view.mvar.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) { showErr(t('view.mvar.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderTable(resp);
}

function renderSummary(report, pending) {
    const local = localAnalyze(state.portfolio, state.z_alpha);
    const parityOk = !!local
        && Math.abs(local.portfolio_var - report.portfolio_var) < 1e-9
        && local.component_var.length === report.component_var.length
        && local.component_var.every((c, i) => Math.abs(c - report.component_var[i]) < 1e-9);
    const badge = concentrationBadge(report.pct_contribution);
    const localTag = pending ? ` (${t('view.mvar.tag.local')})` : '';
    let maxIdx = 0, maxPct = -Infinity;
    for (let i = 0; i < report.pct_contribution.length; i++) {
        if (Math.abs(report.pct_contribution[i]) > maxPct) {
            maxPct = Math.abs(report.pct_contribution[i]);
            maxIdx = i;
        }
    }
    const topName = assetLabel(state.portfolio.labels, maxIdx);
    const sumPct = report.pct_contribution.reduce((s, v) => s + v, 0);
    document.getElementById('mv-summary').innerHTML = [
        card(t('view.mvar.card.verdict'),     t(badge.key) + localTag, badge.cls),
        card(t('view.mvar.card.n'),           fmtInt(state.portfolio.weights.length)),
        card(t('view.mvar.card.z'),           fmtNum(state.z_alpha, 3)),
        card(t('view.mvar.card.port_vol'),    fmtPctNum(report.portfolio_vol, 4)),
        card(t('view.mvar.card.port_var'),    fmtPctNum(report.portfolio_var, 4),
             report.portfolio_var > 0.05 ? 'neg' : ''),
        card(t('view.mvar.card.top_name'),    topName, badge.cls),
        card(t('view.mvar.card.top_pct'),     fmtPct(report.pct_contribution[maxIdx]), badge.cls),
        card(t('view.mvar.card.sum_pct'),     fmtPct(sumPct)),
        card(t('view.mvar.card.parity'),
             parityOk ? t('view.mvar.tag.ok') : t('view.mvar.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(report) {
    if (!window.uPlot) return;
    const el = document.getElementById('mv-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!report.pct_contribution || report.pct_contribution.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.mvar.empty">${esc(t('view.mvar.empty'))}</div>`;
        return;
    }
    // Sort descending by absolute contribution for visual clarity.
    const indexed = report.pct_contribution.map((p, i) => ({ idx: i, pct: p,
                                                              label: assetLabel(state.portfolio.labels, i) }));
    indexed.sort((a, b) => Math.abs(b.pct) - Math.abs(a.pct));
    const xs = indexed.map((_, i) => i);
    const ys = indexed.map(r => r.pct);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 320,
        scales: { x: {}, y: {} },
        series: [
            { label: t('chart.series.rank') },
            { label: t('chart.series._contrib'), stroke: '#00e5ff', width: 1.5, points: { show: true, size: 5 } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => {
                  const i = Math.trunc(v);
                  return i >= 0 && i < indexed.length ? indexed[i].label : '';
              }) },
            { stroke: '#aab', size: 60,
              values: (_u, splits) => splits.map(v => v.toFixed(0) + '%') },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderTable(report) {
    const wrap = document.getElementById('mv-table');
    if (!report.pct_contribution || report.pct_contribution.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.mvar.empty">${esc(t('view.mvar.empty'))}</div>`;
        return;
    }
    const n = report.pct_contribution.length;
    const rows = report.pct_contribution.map((pct, i) => ({
        idx: i,
        label: assetLabel(state.portfolio.labels, i),
        weight: state.portfolio.weights[i],
        marginal: report.marginal_var[i],
        component: report.component_var[i],
        pct,
        verdict: positionBadge(pct, n),
    }));
    rows.sort((a, b) => Math.abs(b.pct) - Math.abs(a.pct));
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.mvar.col.rank">#</th>
                <th data-i18n="view.mvar.col.asset">Asset</th>
                <th data-i18n="view.mvar.col.weight">Weight</th>
                <th data-i18n="view.mvar.col.marginal">Marginal VaR</th>
                <th data-i18n="view.mvar.col.component">Component VaR</th>
                <th data-i18n="view.mvar.col.pct">% of Port VaR</th>
                <th data-i18n="view.mvar.col.verdict">Verdict</th>
            </tr></thead>
            <tbody>
                ${rows.map((r, i) => {
                    const wCls = r.weight > 0 ? 'pos' : r.weight < 0 ? 'neg' : '';
                    const pctCls = r.pct > 33 ? 'neg' : r.pct > 15 ? '' : 'pos';
                    return `<tr>
                        <td>${i + 1}</td>
                        <td><strong>${esc(r.label)}</strong></td>
                        <td class="${wCls}">${esc(fmtPctNum(r.weight))}</td>
                        <td>${esc(fmtSci(r.marginal))}</td>
                        <td>${esc(fmtSci(r.component))}</td>
                        <td class="${pctCls}">${esc(fmtPct(r.pct))}</td>
                        <td data-i18n="${esc(r.verdict.key)}" class="${r.verdict.cls}">${esc(t(r.verdict.key))}</td>
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
    const el = document.getElementById('mv-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('mv-err').style.display = 'none'; }
