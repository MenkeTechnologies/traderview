// Bollinger Squeeze view — detects unusually narrow BB-width periods.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_BB_PERIOD, DEFAULT_N_STDEV, DEFAULT_LOOKBACK, DEFAULT_SLACK,
    parseClosesBlob, closesToBlob, validateInputs, buildBody, localCompute,
    squeezeBadge, summarize, lastSqueezeIndex,
    makeDemoInput,
    fmtWidth, fmtNum, fmtInt, fmtUSD,
} from '../_bollinger_squeeze_inputs.js';

let state = { ...makeDemoInput('coiling') };

export async function renderBollingerSqueeze(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.bbsq.h1.title" class="view-title">// BOLLINGER SQUEEZE</h1>

        <div class="chart-panel" data-context-scope="bb-squeeze">
            <h2 data-i18n="view.bbsq.h2.closes">Closes
                <small data-i18n="view.bbsq.h2.closes_hint" class="muted">(one value per token; whitespace + commas separated)</small></h2>
            <textarea id="bs-blob" rows="6"
                      data-tip="view.bbsq.tip.closes"
                      placeholder="100.0, 100.1, 99.9, ...">${esc(closesToBlob(state.closes))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.bbsq.label.period">BB period</span>
                    <input id="bs-period" type="number" step="1" min="2" value="${state.bb_period}"></label>
                <label><span data-i18n="view.bbsq.label.stdev">σ multiplier</span>
                    <input id="bs-stdev" type="number" step="any" min="0.1" value="${state.n_stdev}"></label>
                <label><span data-i18n="view.bbsq.label.lookback">Lookback</span>
                    <input id="bs-lookback" type="number" step="1" min="2" value="${state.lookback}"></label>
                <label><span data-i18n="view.bbsq.label.slack">Slack</span>
                    <input id="bs-slack" type="number" step="any" min="0" value="${state.slack}"></label>
                <button data-i18n="view.bbsq.btn.compute" id="bs-run" class="primary"
                        data-tip="view.bbsq.tip.compute" type="button">Detect squeeze</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.bbsq.btn.demo_flat"      id="bs-demo-flat"     class="secondary" type="button">Demo: flat (perpetual squeeze)</button>
                <button data-i18n="view.bbsq.btn.demo_coiling"   id="bs-demo-coiling"  class="secondary" type="button">Demo: coiling (volatile → quiet)</button>
                <button data-i18n="view.bbsq.btn.demo_expansion" id="bs-demo-expand"   class="secondary" type="button">Demo: expansion after quiet</button>
                <button data-i18n="view.bbsq.btn.demo_noisy"     id="bs-demo-noisy"    class="secondary" type="button">Demo: noisy walk</button>
                <button data-i18n="view.bbsq.btn.demo_short"     id="bs-demo-short"    class="secondary" type="button">Demo: short lookback (40)</button>
                <button data-i18n="view.bbsq.btn.demo_tight"     id="bs-demo-tight"    class="secondary" type="button">Demo: zero slack (strict)</button>
                <button data-i18n="view.bbsq.btn.demo_loose"     id="bs-demo-loose"    class="secondary" type="button">Demo: loose slack (50%)</button>
                <button data-i18n="view.bbsq.btn.demo_wide"      id="bs-demo-wide"     class="secondary" type="button">Demo: wide bands (3σ)</button>
            </div>
            <p data-i18n="view.bbsq.hint.about" class="muted">width = 2·n_stdev·σ/mean·100. squeeze_on if width ≤ min(width over lookback)·(1+slack). Defaults: 20 / 2.0 / 125 / 0.05. Squeezes precede volatility expansion.</p>
        </div>

        <div id="bs-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.bbsq.h2.chart">BB width % + squeeze markers</h2>
            <div id="bs-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.bbsq.h2.table">Per-bar squeeze status (tail — last 30)</h2>
            <div id="bs-table"></div>
        </div>

        <div id="bs-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('bs-blob').value     = closesToBlob(state.closes);
        document.getElementById('bs-period').value   = state.bb_period;
        document.getElementById('bs-stdev').value    = state.n_stdev;
        document.getElementById('bs-lookback').value = state.lookback;
        document.getElementById('bs-slack').value    = state.slack;
    };
    document.getElementById('bs-demo-flat').addEventListener('click',    () => { loadDemo('flat-perpetual');         void compute(tok); });
    document.getElementById('bs-demo-coiling').addEventListener('click', () => { loadDemo('coiling');                void compute(tok); });
    document.getElementById('bs-demo-expand').addEventListener('click',  () => { loadDemo('expansion-after-quiet'); void compute(tok); });
    document.getElementById('bs-demo-noisy').addEventListener('click',   () => { loadDemo('noisy-walk');            void compute(tok); });
    document.getElementById('bs-demo-short').addEventListener('click',   () => { loadDemo('short-lookback');        void compute(tok); });
    document.getElementById('bs-demo-tight').addEventListener('click',   () => { loadDemo('tight-slack');           void compute(tok); });
    document.getElementById('bs-demo-loose').addEventListener('click',   () => { loadDemo('loose-slack');           void compute(tok); });
    document.getElementById('bs-demo-wide').addEventListener('click',    () => { loadDemo('wide-bands');            void compute(tok); });
    document.getElementById('bs-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseClosesBlob(document.getElementById('bs-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.bbsq.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.closes = p.closes;
    const period = parseInt(document.getElementById('bs-period').value, 10);
    const stdev = Number(document.getElementById('bs-stdev').value);
    const lookback = parseInt(document.getElementById('bs-lookback').value, 10);
    const slack = Number(document.getElementById('bs-slack').value);
    state.bb_period = Number.isInteger(period) && period >= 2 ? period : DEFAULT_BB_PERIOD;
    state.n_stdev   = Number.isFinite(stdev) && stdev > 0 ? stdev : DEFAULT_N_STDEV;
    state.lookback  = Number.isInteger(lookback) && lookback >= state.bb_period ? lookback : DEFAULT_LOOKBACK;
    state.slack     = Number.isFinite(slack) && slack >= 0 ? slack : DEFAULT_SLACK;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.closes, state.bb_period, state.n_stdev, state.lookback, state.slack);
    renderSummary(local, true);
    renderChart(local);
    renderTable(local);
    let resp;
    try {
        resp = await api.anlyBollingerSqueeze(buildBody(state));
    } catch (e) {
        showErr(`${t('view.bbsq.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(resp);
    renderTable(resp);
}

function renderSummary(report, pending) {
    const local = localCompute(state.closes, state.bb_period, state.n_stdev, state.lookback, state.slack);
    const parityOk = report.width_pct.length === local.width_pct.length
        && report.squeeze_on.every((v, i) => v === local.squeeze_on[i])
        && report.width_pct.every((v, i) => {
            if (v == null && local.width_pct[i] == null) return true;
            if (v == null || local.width_pct[i] == null) return false;
            return Math.abs(v - local.width_pct[i]) < 1e-9;
        });
    const s = summarize(report);
    const badge = squeezeBadge(report.squeeze_on, report.width_pct);
    const lastIdx = lastSqueezeIndex(report.squeeze_on);
    const localTag = pending ? ` (${t('view.bbsq.tag.local')})` : '';
    const lastIdxLabel = lastIdx == null ? t('view.bbsq.tag.no_squeeze') : '#' + (lastIdx + 1);
    document.getElementById('bs-summary').innerHTML = [
        card(t('view.bbsq.card.verdict'),       t(badge.key) + localTag, badge.cls),
        card(t('view.bbsq.card.bars'),          fmtInt(s.count)),
        card(t('view.bbsq.card.populated'),     fmtInt(s.populated)),
        card(t('view.bbsq.card.squeeze_count'), fmtInt(s.squeeze_count),
             s.squeeze_count > 0 ? 'pos' : ''),
        card(t('view.bbsq.card.last_state'),
             s.last_state === true  ? t('view.bbsq.tag.squeeze_on')
             : s.last_state === false ? t('view.bbsq.tag.squeeze_off')
             : '—',
             s.last_state === true ? 'pos' : s.last_state === false ? '' : ''),
        card(t('view.bbsq.card.last_width'),    fmtWidth(s.last_width)),
        card(t('view.bbsq.card.min_width'),     fmtWidth(s.min_width)),
        card(t('view.bbsq.card.max_width'),     fmtWidth(s.max_width)),
        card(t('view.bbsq.card.last_squeeze'),  lastIdxLabel,
             lastIdx != null ? 'pos' : ''),
        card(t('view.bbsq.card.parity'),
             parityOk ? t('view.bbsq.tag.ok') : t('view.bbsq.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(report) {
    if (!window.uPlot) return;
    const el = document.getElementById('bs-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!report.width_pct || report.width_pct.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.bbsq.empty">${esc(t('view.bbsq.empty'))}</div>`;
        return;
    }
    const xs = report.width_pct.map((_, i) => i);
    // Marker series: at squeezed bars, set to width (so dot appears on the line).
    const markers = report.width_pct.map((w, i) => report.squeeze_on[i] === true ? w : null);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 340,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: 'bar' },
            { label: 'width %', stroke: '#00e5ff', width: 1.5, points: { show: false } },
            { label: 'squeeze', stroke: '#ffd84a', width: 0,   points: { show: true, size: 6 } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 70,
              values: (_u, splits) => splits.map(v => v.toFixed(2) + '%') },
        ],
        legend: { show: true },
    }, [xs, report.width_pct, markers], el);
}

function renderTable(report) {
    const wrap = document.getElementById('bs-table');
    const n = report.width_pct?.length || 0;
    if (n === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.bbsq.empty">${esc(t('view.bbsq.empty'))}</div>`;
        return;
    }
    const start = Math.max(0, n - 30);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.bbsq.col.idx">#</th>
                <th data-i18n="view.bbsq.col.close">Close</th>
                <th data-i18n="view.bbsq.col.width">Width</th>
                <th data-i18n="view.bbsq.col.state">State</th>
            </tr></thead>
            <tbody>
                ${Array.from({ length: n - start }, (_, k) => {
                    const i = start + k;
                    const w = report.width_pct[i];
                    const s = report.squeeze_on[i];
                    const cls = s === true ? 'pos' : '';
                    const key = s === true  ? 'view.bbsq.cell.on'
                              : s === false ? 'view.bbsq.cell.off'
                              : 'view.bbsq.cell.warmup';
                    return `<tr>
                        <td>${i + 1}</td>
                        <td>${esc(fmtUSD(state.closes[i]))}</td>
                        <td>${esc(fmtWidth(w))}</td>
                        <td data-i18n="${esc(key)}" class="${cls}">${esc(t(key))}</td>
                    </tr>`;
                }).join('')}
            </tbody>
        </table>
    `;
    void fmtNum;
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('bs-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('bs-err').style.display = 'none'; }
