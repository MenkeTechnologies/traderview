// Kyle's Lambda view — rolling price-impact slope estimator.
// Reads paired (Δp, signed_volume) flow → emits per-bar λ time-series
// with liquidity badges and a uPlot chart.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_WINDOW,
    parseFlowBlob, validateInputs, buildBody, localCompute,
    summarize, liquidityBadge, signBadge,
    makeDemoInput, fmtLambda, fmtSci, fmtInt,
} from '../_kyles_lambda_inputs.js';

let state = { ...makeDemoInput('normal-mid-cap') };

export async function renderKylesLambda(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.kyles_lambda.h1.title" class="view-title">// KYLE'S LAMBDA</h1>

        <div class="chart-panel" data-context-scope="kyles-lambda">
            <h2 data-i18n="view.kyles_lambda.h2.flow">Order flow
                <small data-i18n="view.kyles_lambda.h2.flow_hint" class="muted">(per line: price_change signed_volume; + = net buy, − = net sell)</small></h2>
            <textarea id="kl-blob" rows="8"
                      data-tip="view.kyles_lambda.tip.flow"
                      placeholder="0.005 1500&#10;-0.003 -800&#10;0.011 2200">${esc(flowToBlob(state))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.kyles_lambda.label.window">Window (bars)</span>
                    <input id="kl-window" type="number" step="1" min="2" value="${state.window}"></label>
                <button data-i18n="view.kyles_lambda.btn.compute" id="kl-run" class="primary"
                        data-tip="view.kyles_lambda.tip.compute" type="button">Compute λ</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.kyles_lambda.btn.demo_deep"     id="kl-demo-deep"     class="secondary" type="button">Demo: deep market-maker</button>
                <button data-i18n="view.kyles_lambda.btn.demo_normal"   id="kl-demo-normal"   class="secondary" type="button">Demo: normal mid-cap</button>
                <button data-i18n="view.kyles_lambda.btn.demo_thin"     id="kl-demo-thin"     class="secondary" type="button">Demo: thin small-cap</button>
                <button data-i18n="view.kyles_lambda.btn.demo_illiquid" id="kl-demo-illiquid" class="secondary" type="button">Demo: illiquid penny</button>
                <button data-i18n="view.kyles_lambda.btn.demo_reversion" id="kl-demo-rev"     class="secondary" type="button">Demo: mean-reversion (λ&lt;0)</button>
                <button data-i18n="view.kyles_lambda.btn.demo_regime"    id="kl-demo-regime"  class="secondary" type="button">Demo: regime shift mid-series</button>
                <button data-i18n="view.kyles_lambda.btn.demo_zero"      id="kl-demo-zero"    class="secondary" type="button">Demo: zero flow</button>
                <button data-i18n="view.kyles_lambda.btn.demo_spotty"    id="kl-demo-spotty"  class="secondary" type="button">Demo: spotty NaN</button>
            </div>
            <p data-i18n="view.kyles_lambda.hint.about" class="muted">Kyle (1985): Δp = λ · signed_volume + ε. Rolling closed-form OLS, no intercept. LOW |λ| = deep liquid book. HIGH |λ| = thin book. Sign flip → flow-vs-price regime change.</p>
        </div>

        <div id="kl-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.kyles_lambda.h2.chart">Rolling λ time-series</h2>
            <div id="kl-chart" style="width:100%;height:300px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.kyles_lambda.h2.table">Per-bar λ (tail — last 30 bars)</h2>
            <div id="kl-table"></div>
        </div>

        <div id="kl-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('kl-blob').value   = flowToBlob(state);
        document.getElementById('kl-window').value = state.window;
    };
    document.getElementById('kl-demo-deep').addEventListener('click',     () => { loadDemo('deep-mm');        void compute(tok); });
    document.getElementById('kl-demo-normal').addEventListener('click',   () => { loadDemo('normal-mid-cap'); void compute(tok); });
    document.getElementById('kl-demo-thin').addEventListener('click',     () => { loadDemo('thin-small-cap'); void compute(tok); });
    document.getElementById('kl-demo-illiquid').addEventListener('click', () => { loadDemo('illiquid-penny'); void compute(tok); });
    document.getElementById('kl-demo-rev').addEventListener('click',      () => { loadDemo('reversion');     void compute(tok); });
    document.getElementById('kl-demo-regime').addEventListener('click',   () => { loadDemo('regime-shift');  void compute(tok); });
    document.getElementById('kl-demo-zero').addEventListener('click',     () => { loadDemo('zero-flow');     void compute(tok); });
    document.getElementById('kl-demo-spotty').addEventListener('click',   () => { loadDemo('nan-spotty');    void compute(tok); });
    document.getElementById('kl-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function flowToBlob(s) {
    return s.price_changes.map((pc, i) => `${pc} ${s.signed_volumes[i]}`).join('\n');
}

function readInputs() {
    const parsed = parseFlowBlob(document.getElementById('kl-blob').value);
    if (parsed.errors.length) {
        showErr(`${t('view.kyles_lambda.err.parse_prefix')}: `
            + parsed.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.price_changes  = parsed.price_changes;
    state.signed_volumes = parsed.signed_volumes;
    const w = parseInt(document.getElementById('kl-window').value, 10);
    state.window = Number.isFinite(w) ? w : DEFAULT_WINDOW;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.price_changes, state.signed_volumes, state.window);
    renderSummary(local, true);
    renderChart(local);
    renderTable(local);
    let resp;
    try {
        resp = await api.microKylesLambda(buildBody(state));
    } catch (e) {
        showErr(`${t('view.kyles_lambda.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(resp);
    renderTable(resp);
}

function renderSummary(series, pending) {
    const local = localCompute(state.price_changes, state.signed_volumes, state.window);
    const parityOk = series.length === local.length
        && series.every((v, i) => {
            if (v == null && local[i] == null) return true;
            if (v == null || local[i] == null) return false;
            return Math.abs(v - local[i]) < 1e-9;
        });
    const s = summarize(series);
    const localTag = pending ? ` (${t('view.kyles_lambda.tag.local')})` : '';
    const lastBadge = liquidityBadge(s.last);
    const lastSign  = signBadge(s.last);
    document.getElementById('kl-summary').innerHTML = [
        card(t('view.kyles_lambda.card.verdict'),    t(lastBadge.key) + localTag, lastBadge.cls),
        card(t('view.kyles_lambda.card.sign'),       t(lastSign.key)),
        card(t('view.kyles_lambda.card.bars'),       fmtInt(state.price_changes.length)),
        card(t('view.kyles_lambda.card.window'),     fmtInt(state.window)),
        card(t('view.kyles_lambda.card.populated'),  fmtInt(s.count)),
        card(t('view.kyles_lambda.card.last'),       fmtLambda(s.last), lastBadge.cls),
        card(t('view.kyles_lambda.card.mean'),       fmtLambda(s.mean)),
        card(t('view.kyles_lambda.card.min'),        fmtLambda(s.min)),
        card(t('view.kyles_lambda.card.max'),        fmtLambda(s.max)),
        card(t('view.kyles_lambda.card.parity'),
             parityOk ? t('view.kyles_lambda.tag.ok') : t('view.kyles_lambda.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
    void fmtSci;
}

function renderChart(series) {
    if (!window.uPlot) return;
    const el = document.getElementById('kl-chart');
    if (!el) return;
    el.innerHTML = '';
    const xs = series.map((_, i) => i);
    const ys = series.map(v => v == null ? null : v);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 300,
        scales: { x: {}, y: {} },
        series: [
            { label: 'bar' },
            { label: 'λ', stroke: '#00e5ff', width: 1.5, points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 80,
              values: (_u, splits) => splits.map(v => v.toExponential(1)) },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderTable(series) {
    const wrap = document.getElementById('kl-table');
    if (!series || series.length === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.kyles_lambda.empty">${esc(t('view.kyles_lambda.empty'))}</div>`;
        return;
    }
    const start = Math.max(0, series.length - 30);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.kyles_lambda.col.bar">Bar</th>
                <th data-i18n="view.kyles_lambda.col.lambda">λ</th>
                <th data-i18n="view.kyles_lambda.col.depth">Depth</th>
                <th data-i18n="view.kyles_lambda.col.sign">Sign</th>
            </tr></thead>
            <tbody>
                ${series.slice(start).map((v, idx) => {
                    const i = start + idx;
                    const b = liquidityBadge(v);
                    const sgn = signBadge(v);
                    return `<tr>
                        <td>${i}</td>
                        <td class="${b.cls}">${esc(fmtLambda(v))}</td>
                        <td data-i18n="${esc(b.key)}" class="${b.cls}">${esc(t(b.key))}</td>
                        <td data-i18n="${esc(sgn.key)}">${esc(t(sgn.key))}</td>
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
    const el = document.getElementById('kl-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('kl-err').style.display = 'none'; }
