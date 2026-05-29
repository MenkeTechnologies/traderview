// VIX term-structure view. Reads the 5-point implied-vol curve and
// classifies contango / backwardation regime.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_INPUTS, TENORS, TENOR_DAYS,
    validateInputs, buildBody, localAnalyze,
    stateBadge, tenorContributions, makeDemoInput,
    fmtN, fmtSigned, fmtRatio,
} from '../_vix_term_structure_inputs.js';

let state = { ...DEFAULT_INPUTS };

export async function renderVixTermStructure(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.vix_term_structure.h1.title" class="view-title">// VIX TERM STRUCTURE</h1>

        <div class="chart-panel" data-context-scope="vix-term-structure">
            <h2 data-i18n="view.vix_term_structure.h2.curve">5-point curve</h2>
            <div class="inline-form">
                ${TENORS.map(k => `
                    <label><span data-i18n="view.vix_term_structure.label.${k}">${k.toUpperCase()}</span>
                        <input id="vix-${k}" type="number" step="any" min="0" value="${state[k]}"></label>
                `).join('')}
                <button data-i18n="view.vix_term_structure.btn.analyze" id="vix-run" class="primary"
                        data-tip="view.vix_term_structure.tip.analyze" type="button">Analyze</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.vix_term_structure.btn.demo_steep"    id="vix-demo-steep"   class="secondary" type="button">Demo: steep contango</button>
                <button data-i18n="view.vix_term_structure.btn.demo_normal"   id="vix-demo-normal"  class="secondary" type="button">Demo: normal contango</button>
                <button data-i18n="view.vix_term_structure.btn.demo_flat"     id="vix-demo-flat"    class="secondary" type="button">Demo: flat</button>
                <button data-i18n="view.vix_term_structure.btn.demo_backward" id="vix-demo-back"    class="secondary" type="button">Demo: backwardation</button>
                <button data-i18n="view.vix_term_structure.btn.demo_severe"   id="vix-demo-severe"  class="secondary" type="button">Demo: severe backwardation</button>
                <button data-i18n="view.vix_term_structure.btn.demo_covid"    id="vix-demo-covid"   class="secondary" type="button">Demo: COVID spike (Mar 2020)</button>
                <button data-i18n="view.vix_term_structure.btn.demo_gfc"      id="vix-demo-gfc"     class="secondary" type="button">Demo: GFC bear (Oct 2008)</button>
                <button data-i18n="view.vix_term_structure.btn.demo_lowvol"   id="vix-demo-lowvol"  class="secondary" type="button">Demo: low-vol regime</button>
            </div>
            <p data-i18n="view.vix_term_structure.hint.about" class="muted">VIX/VIX3M ratio &lt; 0.80 steep contango · &lt; 1.00 contango · &lt; 1.05 flat · &lt; 1.20 backwardation · ≥ 1.20 severe. Slope = sum of consecutive-tenor diffs. Boundaries are strict &lt;; e.g. ratio = 1.05 exactly → backwardation.</p>
        </div>

        <div id="vix-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.vix_term_structure.h2.chart">Term-structure curve</h2>
            <div id="vix-chart" style="height:320px"></div>
            <p data-i18n="view.vix_term_structure.hint.chart" class="muted">X axis: tenor in days. Y axis: implied vol. Upward slope = contango (normal). Downward = backwardation (fear).</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.vix_term_structure.h2.contrib">Per-tenor slope contribution</h2>
            <div id="vix-contrib"></div>
        </div>

        <div id="vix-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        for (const tnr of TENORS) document.getElementById(`vix-${tnr}`).value = state[tnr];
    };
    document.getElementById('vix-demo-steep').addEventListener('click',  () => loadDemo('steep-contango'));
    document.getElementById('vix-demo-normal').addEventListener('click', () => loadDemo('normal-contango'));
    document.getElementById('vix-demo-flat').addEventListener('click',   () => loadDemo('flat'));
    document.getElementById('vix-demo-back').addEventListener('click',   () => loadDemo('backwardation'));
    document.getElementById('vix-demo-severe').addEventListener('click', () => loadDemo('severe'));
    document.getElementById('vix-demo-covid').addEventListener('click',  () => loadDemo('covid-spike'));
    document.getElementById('vix-demo-gfc').addEventListener('click',    () => loadDemo('gfc-bear'));
    document.getElementById('vix-demo-lowvol').addEventListener('click', () => loadDemo('low-vol-regime'));
    document.getElementById('vix-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function readInputs() {
    const out = {};
    for (const k of TENORS) out[k] = Number(document.getElementById(`vix-${k}`).value);
    state = out;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localAnalyze(state);
    renderSummary(local, true);
    renderChart();
    renderContrib();
    let resp;
    try {
        resp = await api.calcVixTermStructure(buildBody(state));
    } catch (e) {
        showErr(`${t('view.vix_term_structure.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart();
    renderContrib();
}

function renderSummary(report, pending) {
    const badge = stateBadge(report.state);
    const local = localAnalyze(state);
    const parityOk = report.state === local.state
                  && Math.abs(report.vix_to_vix3m_ratio - local.vix_to_vix3m_ratio) < 1e-9;
    const localTag = pending ? ` (${t('view.vix_term_structure.tag.local')})` : '';
    document.getElementById('vix-summary').innerHTML = [
        card(t('view.vix_term_structure.card.regime'),
             t(badge.key) + localTag, badge.cls),
        card(t('view.vix_term_structure.card.ratio'),
             fmtRatio(report.vix_to_vix3m_ratio), badge.cls),
        card(t('view.vix_term_structure.card.slope'),
             fmtSigned(report.slope),
             report.slope >= 0 ? 'pos' : 'neg'),
        card(t('view.vix_term_structure.card.note'),
             t(local.note_key)),
        card(t('view.vix_term_structure.card.vix_minus_vix3m'),
             fmtSigned(state.vix - state.vix3m),
             state.vix < state.vix3m ? 'pos' : 'neg'),
        card(t('view.vix_term_structure.card.front_lead'),
             fmtSigned(state.vix - state.vix9d),
             state.vix > state.vix9d ? 'pos' : 'neg'),
        card(t('view.vix_term_structure.card.parity'),
             parityOk ? t('view.vix_term_structure.tag.ok') : t('view.vix_term_structure.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart() {
    if (!window.uPlot) return;
    const el = document.getElementById('vix-chart');
    if (!el) return;
    el.innerHTML = '';
    const xs = TENORS.map(k => TENOR_DAYS[k]);
    const ys = TENORS.map(k => state[k]);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 300,
        scales: { x: {}, y: {} },
        series: [
            { label: 'days' },
            { label: 'IV',
              stroke: '#00e5ff', width: 2,
              fill: '#00e5ff14',
              points: { show: true, size: 8, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => v + 'd') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: false },
    }, [xs, ys], el);
}

function renderContrib() {
    const wrap = document.getElementById('vix-contrib');
    if (!wrap) return;
    const rows = tenorContributions(state);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.vix_term_structure.col.from">From</th>
                <th data-i18n="view.vix_term_structure.col.to">To</th>
                <th data-i18n="view.vix_term_structure.col.delta">Δ Vol</th>
                <th data-i18n="view.vix_term_structure.col.dir">Direction</th>
            </tr></thead>
            <tbody>
                ${rows.map(r => `<tr>
                    <td><strong data-i18n="view.vix_term_structure.label.${esc(r.from)}">${esc(r.from.toUpperCase())}</strong></td>
                    <td><strong data-i18n="view.vix_term_structure.label.${esc(r.to)}">${esc(r.to.toUpperCase())}</strong></td>
                    <td class="${r.delta >= 0 ? 'pos' : 'neg'}">${esc(fmtSigned(r.delta))}</td>
                    <td class="${r.delta >= 0 ? 'pos' : 'neg'}">${r.delta > 0 ? '▲' : r.delta < 0 ? '▼' : '·'}</td>
                </tr>`).join('')}
            </tbody>
        </table>
    `;
    void fmtN;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('vix-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('vix-err').style.display = 'none'; }
