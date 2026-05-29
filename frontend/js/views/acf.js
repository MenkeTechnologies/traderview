// ACF view — sample autocorrelation function + Bartlett confidence bands.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_MAX_LAG,
    parseSeriesBlob, seriesToBlob, validateInputs, buildBody, localCompute,
    autocorrelationBadge, ar1PhiEstimate, summarize,
    makeDemoInput,
    fmtAcf, fmtBand, fmtInt, fmtNum,
} from '../_acf_inputs.js';

let state = { ...makeDemoInput('ar1-0.8') };

export async function renderAcf(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.acf.h1.title" class="view-title">// AUTOCORRELATION (ACF)</h1>

        <div class="chart-panel" data-context-scope="acf">
            <h2 data-i18n="view.acf.h2.series">Series
                <small data-i18n="view.acf.h2.series_hint" class="muted">(one value per token; comments + blanks ignored)</small></h2>
            <textarea id="ac-blob" rows="6"
                      data-tip="view.acf.tip.series"
                      placeholder="0.01, -0.02, 0.005, ...">${esc(seriesToBlob(state.series))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.acf.label.max_lag">Max lag</span>
                    <input id="ac-lag" type="number" step="1" min="1" value="${state.max_lag}"></label>
                <button data-i18n="view.acf.btn.compute" id="ac-run" class="primary"
                        data-tip="view.acf.tip.compute" type="button">Compute ACF</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.acf.btn.demo_white"  id="ac-demo-white" class="secondary" type="button">Demo: white noise</button>
                <button data-i18n="view.acf.btn.demo_rw"     id="ac-demo-rw"    class="secondary" type="button">Demo: random walk (ρ→1)</button>
                <button data-i18n="view.acf.btn.demo_ar08"   id="ac-demo-ar08"  class="secondary" type="button">Demo: AR(1) φ=0.8</button>
                <button data-i18n="view.acf.btn.demo_arneg"  id="ac-demo-arneg" class="secondary" type="button">Demo: AR(1) φ=−0.6 (alternating)</button>
                <button data-i18n="view.acf.btn.demo_sine"   id="ac-demo-sine"  class="secondary" type="button">Demo: sinusoid</button>
                <button data-i18n="view.acf.btn.demo_trend"  id="ac-demo-trend" class="secondary" type="button">Demo: strong trend</button>
                <button data-i18n="view.acf.btn.demo_wide"   id="ac-demo-wide"  class="secondary" type="button">Demo: wide lags (50)</button>
                <button data-i18n="view.acf.btn.demo_short"  id="ac-demo-short" class="secondary" type="button">Demo: short series (20 bars)</button>
            </div>
            <p data-i18n="view.acf.hint.about" class="muted">ρ̂(k) = Σ(x_t−x̄)(x_{t−k}−x̄) / Σ(x_t−x̄)². ρ̂(0) = 1 by definition. Bartlett 95% bands = ±1.96/√n. Lags outside bands are statistically significant — evidence against white noise.</p>
        </div>

        <div id="ac-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.acf.h2.chart">ACF + Bartlett 95% bands</h2>
            <div id="ac-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.acf.h2.table">Per-lag ACF</h2>
            <div id="ac-table"></div>
        </div>

        <div id="ac-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('ac-blob').value = seriesToBlob(state.series);
        document.getElementById('ac-lag').value  = state.max_lag;
    };
    document.getElementById('ac-demo-white').addEventListener('click', () => { loadDemo('white-noise');    void compute(tok); });
    document.getElementById('ac-demo-rw').addEventListener('click',    () => { loadDemo('random-walk');    void compute(tok); });
    document.getElementById('ac-demo-ar08').addEventListener('click',  () => { loadDemo('ar1-0.8');        void compute(tok); });
    document.getElementById('ac-demo-arneg').addEventListener('click', () => { loadDemo('ar1-neg0.6');     void compute(tok); });
    document.getElementById('ac-demo-sine').addEventListener('click',  () => { loadDemo('sinusoid');       void compute(tok); });
    document.getElementById('ac-demo-trend').addEventListener('click', () => { loadDemo('trending');       void compute(tok); });
    document.getElementById('ac-demo-wide').addEventListener('click',  () => { loadDemo('wide-lags');      void compute(tok); });
    document.getElementById('ac-demo-short').addEventListener('click', () => { loadDemo('short-series');   void compute(tok); });
    document.getElementById('ac-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseSeriesBlob(document.getElementById('ac-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.acf.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.series = p.series;
    const lag = parseInt(document.getElementById('ac-lag').value, 10);
    state.max_lag = Number.isInteger(lag) && lag >= 1 ? lag : DEFAULT_MAX_LAG;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localCompute(state.series, state.max_lag);
    if (!local) { showErr(t('view.acf.err.degenerate')); return; }
    renderSummary(local, true);
    renderChart(local);
    renderTable(local);
    let resp;
    try {
        resp = await api.anlyAutocorrelationFunction(buildBody(state));
    } catch (e) {
        showErr(`${t('view.acf.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) { showErr(t('view.acf.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderTable(resp);
}

function renderSummary(report, pending) {
    const local = localCompute(state.series, state.max_lag);
    const parityOk = !!local
        && report.autocorrelations.length === local.autocorrelations.length
        && report.autocorrelations.every((v, i) => Math.abs(v - local.autocorrelations[i]) < 1e-9)
        && Math.abs(report.confidence_band - local.confidence_band) < 1e-9;
    const s = summarize(report);
    const badge = autocorrelationBadge(s.rho1, report.confidence_band);
    const phi = ar1PhiEstimate(report);
    const localTag = pending ? ` (${t('view.acf.tag.local')})` : '';
    document.getElementById('ac-summary').innerHTML = [
        card(t('view.acf.card.verdict'),     t(badge.key) + localTag, badge.cls),
        card(t('view.acf.card.nobs'),        fmtInt(report.n_observations)),
        card(t('view.acf.card.max_lag'),     fmtInt(state.max_lag)),
        card(t('view.acf.card.band'),        fmtBand(report.confidence_band)),
        card(t('view.acf.card.sig_count'),   fmtInt(s.sig_count),
             s.sig_count > 0 ? 'neg' : 'pos'),
        card(t('view.acf.card.max_lag_idx'), s.max_abs_lag > 0 ? '#' + s.max_abs_lag : '—'),
        card(t('view.acf.card.max_abs_acf'), fmtAcf(s.max_abs_acf)),
        card(t('view.acf.card.rho1'),        fmtAcf(s.rho1), badge.cls),
        card(t('view.acf.card.rho5'),        fmtAcf(s.rho5)),
        card(t('view.acf.card.rho10'),       fmtAcf(s.rho10)),
        card(t('view.acf.card.phi'),         fmtNum(phi, 4)),
        card(t('view.acf.card.parity'),
             parityOk ? t('view.acf.tag.ok') : t('view.acf.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(report) {
    if (!window.uPlot) return;
    const el = document.getElementById('ac-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!report.autocorrelations || report.autocorrelations.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.acf.empty">${esc(t('view.acf.empty'))}</div>`;
        return;
    }
    // Skip lag 0 (always +1) for visual clarity; plot lags 1..max.
    const xs = report.lags.slice(1);
    const ys = report.autocorrelations.slice(1);
    const bandPos = xs.map(() =>  report.confidence_band);
    const bandNeg = xs.map(() => -report.confidence_band);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 340,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('chart.series.lag') },
            { label: 'ρ̂(k)',  stroke: '#00e5ff', width: 1.5, points: { show: true, size: 5 } },
            { label: t('chart.series.plus_band'), stroke: '#ff3860', width: 1.0, dash: [4, 4], points: { show: false } },
            { label: t('chart.series.minus_band'), stroke: '#ff3860', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, ys, bandPos, bandNeg], el);
}

function renderTable(report) {
    const wrap = document.getElementById('ac-table');
    const n = report.autocorrelations?.length || 0;
    if (n === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.acf.empty">${esc(t('view.acf.empty'))}</div>`;
        return;
    }
    const band = report.confidence_band;
    const sigSet = new Set(report.significant_lags);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.acf.col.lag">Lag</th>
                <th data-i18n="view.acf.col.acf">ρ̂(k)</th>
                <th data-i18n="view.acf.col.abs">|ρ̂(k)|</th>
                <th data-i18n="view.acf.col.band">vs band</th>
                <th data-i18n="view.acf.col.tag">Tag</th>
            </tr></thead>
            <tbody>
                ${report.autocorrelations.map((v, k) => {
                    const sig = sigSet.has(k);
                    const cls = k === 0 ? '' : sig ? 'neg' : 'pos';
                    const key = k === 0 ? 'view.acf.cell.zero_lag'
                              : sig ? 'view.acf.cell.significant'
                              : 'view.acf.cell.in_band';
                    return `<tr>
                        <td><strong>${k}</strong></td>
                        <td class="${cls}">${esc(fmtAcf(v))}</td>
                        <td>${esc(fmtNum(Math.abs(v)))}</td>
                        <td>${esc(fmtNum(Math.abs(v) / band))}×</td>
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
    const el = document.getElementById('ac-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ac-err').style.display = 'none'; }
