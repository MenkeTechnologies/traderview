// Herfindahl-Hirschman concentration view — analyzes a weight vector
// for portfolio concentration risk + DOJ-style scaled score.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DOJ_CONCENTRATED, DOJ_MODERATE,
    parsePositionsBlob, positionsToBlob, validateInputs, buildBody, localCompute,
    concentrationBadge, efficiencyBadge,
    makeDemoInput,
    fmtHhi, fmtScaled, fmtEffN, fmtPct, fmtInt,
} from '../_herfindahl_inputs.js';

let state = { ...makeDemoInput('pareto-80-20') };

export async function renderHerfindahl(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.hhi.h1.title" class="view-title">// HERFINDAHL (HHI)</h1>

        <div class="chart-panel" data-context-scope="hhi">
            <h2 data-i18n="view.hhi.h2.positions">Positions
                <small data-i18n="view.hhi.h2.positions_hint" class="muted">(per line: "weight" or "label weight"; absolute or normalized weights)</small></h2>
            <textarea id="hh-blob" rows="8"
                      data-tip="view.hhi.tip.positions"
                      placeholder="SPY 0.30&#10;QQQ 0.20&#10;GLD 0.50">${esc(positionsToBlob(state.labels, state.weights))}</textarea>

            <div class="inline-form">
                <button data-i18n="view.hhi.btn.compute" id="hh-run" class="primary"
                        data-tip="view.hhi.tip.compute" type="button">Compute HHI</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.hhi.btn.demo_eq4"    id="hh-demo-eq4"   class="secondary" type="button">Demo: equal 4</button>
                <button data-i18n="view.hhi.btn.demo_eq10"   id="hh-demo-eq10"  class="secondary" type="button">Demo: equal 10</button>
                <button data-i18n="view.hhi.btn.demo_conc"   id="hh-demo-conc"  class="secondary" type="button">Demo: concentrated (80% one name)</button>
                <button data-i18n="view.hhi.btn.demo_single" id="hh-demo-single" class="secondary" type="button">Demo: single name</button>
                <button data-i18n="view.hhi.btn.demo_pareto" id="hh-demo-pareto" class="secondary" type="button">Demo: Pareto 80/20</button>
                <button data-i18n="view.hhi.btn.demo_unnorm" id="hh-demo-unnorm" class="secondary" type="button">Demo: un-normalized</button>
                <button data-i18n="view.hhi.btn.demo_zeroes" id="hh-demo-zeroes" class="secondary" type="button">Demo: with zeroes</button>
                <button data-i18n="view.hhi.btn.demo_6040"   id="hh-demo-6040"  class="secondary" type="button">Demo: 60/40</button>
            </div>
            <p data-i18n="view.hhi.hint.about" class="muted">HHI = Σwᵢ². Scaled 0–10,000 (DOJ scale). Effective N = 1/HHI = "equivalent equal-weight positions." DOJ antitrust thresholds: ≥ ${DOJ_MODERATE} moderate · ≥ ${DOJ_CONCENTRATED} highly concentrated.</p>
        </div>

        <div id="hh-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.hhi.h2.bars">Weight contribution to HHI</h2>
            <div id="hh-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.hhi.h2.table">Position breakdown</h2>
            <div id="hh-table"></div>
        </div>

        <div id="hh-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('hh-blob').value = positionsToBlob(state.labels, state.weights);
    };
    document.getElementById('hh-demo-eq4').addEventListener('click',   () => { loadDemo('equal-4');      void compute(tok); });
    document.getElementById('hh-demo-eq10').addEventListener('click',  () => { loadDemo('equal-10');     void compute(tok); });
    document.getElementById('hh-demo-conc').addEventListener('click',  () => { loadDemo('concentrated'); void compute(tok); });
    document.getElementById('hh-demo-single').addEventListener('click', () => { loadDemo('single-name'); void compute(tok); });
    document.getElementById('hh-demo-pareto').addEventListener('click', () => { loadDemo('pareto-80-20'); void compute(tok); });
    document.getElementById('hh-demo-unnorm').addEventListener('click', () => { loadDemo('unnormalized'); void compute(tok); });
    document.getElementById('hh-demo-zeroes').addEventListener('click', () => { loadDemo('with-zeroes'); void compute(tok); });
    document.getElementById('hh-demo-6040').addEventListener('click',  () => { loadDemo('60-40-style');  void compute(tok); });
    document.getElementById('hh-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parsePositionsBlob(document.getElementById('hh-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.hhi.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.weights = p.weights;
    state.labels  = p.labels;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.weights);
    if (!local) { showErr(t('view.hhi.err.degenerate')); return; }
    renderSummary(local, true);
    renderChart(local);
    renderTable(local);
    let resp;
    try {
        resp = await api.portfolioHerfindahl(buildBody(state));
    } catch (e) {
        showErr(`${t('view.hhi.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) { showErr(t('view.hhi.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderTable(resp);
}

function renderSummary(report, pending) {
    const local = localCompute(state.weights);
    const parityOk = !!local
        && Math.abs(local.hhi - report.hhi) < 1e-9
        && local.n_positions === report.n_positions
        && Math.abs(local.effective_n - report.effective_n) < 1e-9;
    const badge = concentrationBadge(report);
    const eff = efficiencyBadge(report);
    const localTag = pending ? ` (${t('view.hhi.tag.local')})` : '';
    document.getElementById('hh-summary').innerHTML = [
        card(t('view.hhi.card.verdict'),     t(badge.key) + localTag, badge.cls),
        card(t('view.hhi.card.efficiency'),  t(eff.key), eff.cls),
        card(t('view.hhi.card.hhi'),         fmtHhi(report.hhi)),
        card(t('view.hhi.card.scaled'),      fmtScaled(report.hhi_scaled), badge.cls),
        card(t('view.hhi.card.effective_n'), fmtEffN(report.effective_n)),
        card(t('view.hhi.card.n_positions'), fmtInt(report.n_positions)),
        card(t('view.hhi.card.max_weight'),  fmtPct(report.max_weight), report.max_weight > 0.5 ? 'neg' : ''),
        card(t('view.hhi.card.parity'),
             parityOk ? t('view.hhi.tag.ok') : t('view.hhi.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(report) {
    if (!window.uPlot) return;
    const el = document.getElementById('hh-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!state.weights || state.weights.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.hhi.empty">${esc(t('view.hhi.empty'))}</div>`;
        return;
    }
    // Sort positions descending by weight for visual clarity.
    const sumW = state.weights.reduce((s, w) => s + w, 0);
    const indexed = state.weights.map((w, i) => ({
        idx: i,
        label: state.labels[i],
        norm: sumW > 0 ? w / sumW : 0,
    }));
    indexed.sort((a, b) => b.norm - a.norm);
    const xs = indexed.map((_, i) => i);
    const weightPct = indexed.map(p => p.norm * 100);
    // Each position's contribution to HHI (squared share, scaled to 10_000).
    const contribScaled = indexed.map(p => p.norm * p.norm * 10_000);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 340,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('chart.series.rank') },
            { label: t('chart.series.weight_'),     stroke: '#00e5ff', width: 1.5, points: { show: true, size: 5 } },
            { label: t('chart.series.hhi_contrib'),  stroke: '#ff3860', width: 1.5, points: { show: true, size: 5 } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => {
                  const i = Math.trunc(v);
                  return i >= 0 && i < indexed.length ? indexed[i].label : '';
              })
            },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, weightPct, contribScaled], el);
}

function renderTable(report) {
    const wrap = document.getElementById('hh-table');
    if (!state.weights || state.weights.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.hhi.empty">${esc(t('view.hhi.empty'))}</div>`;
        return;
    }
    const sumW = state.weights.reduce((s, w) => s + w, 0);
    const rows = state.weights.map((w, i) => {
        const norm = sumW > 0 ? w / sumW : 0;
        return {
            label: state.labels[i],
            raw: w,
            norm,
            contrib: norm * norm,
            contrib_scaled: norm * norm * 10_000,
        };
    });
    rows.sort((a, b) => b.norm - a.norm);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.hhi.col.rank">#</th>
                <th data-i18n="view.hhi.col.label">Position</th>
                <th data-i18n="view.hhi.col.raw">Raw</th>
                <th data-i18n="view.hhi.col.norm">Normalized</th>
                <th data-i18n="view.hhi.col.contrib">w²</th>
                <th data-i18n="view.hhi.col.contrib_scaled">w² × 10k</th>
            </tr></thead>
            <tbody>
                ${rows.map((r, i) => {
                    const cls = r.norm > 0.5 ? 'neg' : r.norm > 0.25 ? '' : 'pos';
                    return `<tr>
                        <td>${i + 1}</td>
                        <td><strong>${esc(r.label)}</strong></td>
                        <td>${esc(fmtHhi(r.raw))}</td>
                        <td class="${cls}">${esc(fmtPct(r.norm))}</td>
                        <td>${esc(fmtHhi(r.contrib))}</td>
                        <td class="${cls}">${esc(fmtScaled(r.contrib_scaled))}</td>
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
    const el = document.getElementById('hh-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('hh-err').style.display = 'none'; }
