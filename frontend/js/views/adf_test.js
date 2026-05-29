// ADF test view — Augmented Dickey-Fuller unit-root / stationarity test.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_LAGS, CRIT_1PCT, CRIT_5PCT, CRIT_10PCT,
    parseSeriesBlob, seriesToBlob, validateInputs, buildBody, localTest,
    significanceBadge, strengthBadge, significanceLabelKey,
    makeDemoInput,
    fmtNum, fmtT, fmtInt,
} from '../_adf_test_inputs.js';

let state = { ...makeDemoInput('mean-reverting-strong') };

export async function renderAdfTest(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.adf.h1.title" class="view-title">// ADF STATIONARITY TEST</h1>

        <div class="chart-panel" data-context-scope="adf">
            <h2 data-i18n="view.adf.h2.series">Series
                <small data-i18n="view.adf.h2.series_hint" class="muted">(one value per token; whitespace + commas separated; # comments)</small></h2>
            <textarea id="adf-blob" rows="6"
                      data-tip="view.adf.tip.series"
                      placeholder="100.05, 100.10, 99.95, ...">${esc(seriesToBlob(state.series))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.adf.label.lags">Augmentation lags (p)</span>
                    <input id="adf-lags" type="number" step="1" min="0" value="${state.lags}"></label>
                <button data-i18n="view.adf.btn.compute" id="adf-run" class="primary"
                        data-tip="view.adf.tip.compute" type="button">Run ADF</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.adf.btn.demo_walk"     id="adf-demo-walk"   class="secondary" type="button">Demo: random walk (unit root)</button>
                <button data-i18n="view.adf.btn.demo_mr_strong" id="adf-demo-mrs"   class="secondary" type="button">Demo: strong mean-rev (φ=0.3)</button>
                <button data-i18n="view.adf.btn.demo_mr_weak"  id="adf-demo-mrw"    class="secondary" type="button">Demo: weak mean-rev (φ=0.85)</button>
                <button data-i18n="view.adf.btn.demo_trend"    id="adf-demo-trend"  class="secondary" type="button">Demo: trend (drift+noise)</button>
                <button data-i18n="view.adf.btn.demo_noise"    id="adf-demo-noise"  class="secondary" type="button">Demo: pure noise</button>
                <button data-i18n="view.adf.btn.demo_lags"     id="adf-demo-lags"   class="secondary" type="button">Demo: same series, lags=5</button>
                <button data-i18n="view.adf.btn.demo_short"    id="adf-demo-short"  class="secondary" type="button">Demo: short series (lags=0)</button>
                <button data-i18n="view.adf.btn.demo_flat"     id="adf-demo-flat"   class="secondary" type="button">Demo: flat (degenerate)</button>
            </div>
            <p data-i18n="view.adf.hint.about" class="muted">Δy_t = α + γ·y_{t-1} + Σφ·Δy_{t-i} + ε. H₀: unit root (non-stationary). Reject when t-stat &lt; critical (1% = −3.43, 5% = −2.86, 10% = −2.57). More-negative = more stationary.</p>
        </div>

        <div id="adf-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.adf.h2.chart">Series + first-differences</h2>
            <div id="adf-chart" style="width:100%;height:320px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.adf.h2.thresholds">Critical-value comparison</h2>
            <div id="adf-thresh"></div>
        </div>

        <div id="adf-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('adf-blob').value = seriesToBlob(state.series);
        document.getElementById('adf-lags').value = state.lags;
    };
    document.getElementById('adf-demo-walk').addEventListener('click',  () => { loadDemo('random-walk');           void compute(tok); });
    document.getElementById('adf-demo-mrs').addEventListener('click',   () => { loadDemo('mean-reverting-strong'); void compute(tok); });
    document.getElementById('adf-demo-mrw').addEventListener('click',   () => { loadDemo('mean-reverting-weak');   void compute(tok); });
    document.getElementById('adf-demo-trend').addEventListener('click', () => { loadDemo('trend-stationary');      void compute(tok); });
    document.getElementById('adf-demo-noise').addEventListener('click', () => { loadDemo('pure-noise');            void compute(tok); });
    document.getElementById('adf-demo-lags').addEventListener('click',  () => { loadDemo('high-lags');             void compute(tok); });
    document.getElementById('adf-demo-short').addEventListener('click', () => { loadDemo('short-series');          void compute(tok); });
    document.getElementById('adf-demo-flat').addEventListener('click',  () => { loadDemo('flat');                  void compute(tok); });
    document.getElementById('adf-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseSeriesBlob(document.getElementById('adf-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.adf.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.series = p.series;
    const lg = parseInt(document.getElementById('adf-lags').value, 10);
    state.lags = Number.isInteger(lg) && lg >= 0 ? lg : DEFAULT_LAGS;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localTest(state.series, state.lags);
    if (!local) { showErr(t('view.adf.err.degenerate')); return; }
    renderSummary(local, true);
    renderChart();
    renderThresholds(local);
    let resp;
    try {
        resp = await api.anlyAdfTest(buildBody(state));
    } catch (e) {
        showErr(`${t('view.adf.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) { showErr(t('view.adf.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart();
    renderThresholds(resp);
}

function renderSummary(report, pending) {
    const local = localTest(state.series, state.lags);
    const parityOk = !!local
        && Math.abs(local.t_statistic - report.t_statistic) < 1e-6
        && local.significance === report.significance
        && local.n_observations === report.n_observations;
    const sBadge = significanceBadge(report.significance);
    const stBadge = strengthBadge(report.t_statistic);
    const localTag = pending ? ` (${t('view.adf.tag.local')})` : '';
    document.getElementById('adf-summary').innerHTML = [
        card(t('view.adf.card.verdict'),    t(sBadge.key) + localTag, sBadge.cls),
        card(t('view.adf.card.strength'),   t(stBadge.key), stBadge.cls),
        card(t('view.adf.card.t_stat'),     fmtT(report.t_statistic), stBadge.cls),
        card(t('view.adf.card.gamma'),      fmtNum(report.gamma)),
        card(t('view.adf.card.gamma_se'),   fmtNum(report.gamma_se)),
        card(t('view.adf.card.significance'), t(significanceLabelKey(report.significance)), sBadge.cls),
        card(t('view.adf.card.lags'),       fmtInt(report.lags)),
        card(t('view.adf.card.nobs'),       fmtInt(report.n_observations)),
        card(t('view.adf.card.input_len'),  fmtInt(state.series.length)),
        card(t('view.adf.card.parity'),
             parityOk ? t('view.adf.tag.ok') : t('view.adf.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart() {
    if (!window.uPlot) return;
    const el = document.getElementById('adf-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!state.series || state.series.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.adf.empty">${esc(t('view.adf.empty'))}</div>`;
        return;
    }
    const xs = state.series.map((_, i) => i);
    const diffs = state.series.map((v, i) => i === 0 ? null : v - state.series[i - 1]);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 320,
        scales: { x: {}, y: {}, y2: { auto: true } },
        series: [
            { label: 't' },
            { label: 'y',  stroke: '#00e5ff', width: 1.5, points: { show: false } },
            { label: 'Δy', stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false }, scale: 'y2' },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 60 },
            { side: 1, stroke: '#aab', size: 60, scale: 'y2' },
        ],
        legend: { show: true },
    }, [xs, state.series, diffs], el);
}

function renderThresholds(report) {
    const wrap = document.getElementById('adf-thresh');
    const rows = [
        { label: '1%',  crit: CRIT_1PCT,  passes: report.t_statistic < CRIT_1PCT },
        { label: '5%',  crit: CRIT_5PCT,  passes: report.t_statistic < CRIT_5PCT },
        { label: '10%', crit: CRIT_10PCT, passes: report.t_statistic < CRIT_10PCT },
    ];
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.adf.col.level">Level</th>
                <th data-i18n="view.adf.col.crit">Critical t</th>
                <th data-i18n="view.adf.col.tstat">Our t</th>
                <th data-i18n="view.adf.col.margin">Margin</th>
                <th data-i18n="view.adf.col.reject">Reject H₀?</th>
            </tr></thead>
            <tbody>
                ${rows.map(r => {
                    const margin = r.crit - report.t_statistic;
                    const cls = r.passes ? 'pos' : 'neg';
                    const key = r.passes ? 'view.adf.cell.yes' : 'view.adf.cell.no';
                    return `<tr>
                        <td><strong>${r.label}</strong></td>
                        <td>${esc(fmtT(r.crit))}</td>
                        <td>${esc(fmtT(report.t_statistic))}</td>
                        <td>${esc(fmtT(margin))}</td>
                        <td data-i18n="${esc(key)}" class="${cls}">${esc(t(key))}</td>
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
    const el = document.getElementById('adf-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('adf-err').style.display = 'none'; }
