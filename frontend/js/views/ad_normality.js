// Anderson-Darling Normality Test view (Stephens 1986).
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseSampleBlob, sampleToBlob, validateInputs, buildBody, localTest,
    verdictBadge, approxPValue, summarizeSample,
    makeDemoInput, qqPlotData,
    fmtNum, fmtNumSigned, fmtPVal, fmtInt,
} from '../_ad_normality_inputs.js';

let state = { ...makeDemoInput('gaussian') };

export async function renderAdNormality(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.ad_norm.h1.title" class="view-title">// ANDERSON-DARLING NORMALITY</h1>

        <div class="chart-panel" data-context-scope="ad-normality">
            <h2 data-i18n="view.ad_norm.h2.sample">Sample
                <small data-i18n="view.ad_norm.h2.sample_hint" class="muted">(≥ 8 observations; whitespace/comma-separated)</small></h2>
            <textarea id="adn-blob" rows="6"
                      data-tip="view.ad_norm.tip.sample"
                      placeholder="-0.21, 0.05, 0.13, ...">${esc(sampleToBlob(state.sample))}</textarea>

            <div class="inline-form">
                <button data-i18n="view.ad_norm.btn.compute" id="adn-run" class="primary"
                        data-tip="view.ad_norm.tip.compute" data-shortcut="ad_normality_run" type="button">Test</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.ad_norm.btn.demo_gauss"  id="adn-d1" class="secondary" data-tip="view.ad_norm.tip.demo_gauss" type="button">Demo: Gaussian (n=500)</button>
                <button data-i18n="view.ad_norm.btn.demo_heavy"  id="adn-d2" class="secondary" data-tip="view.ad_norm.tip.demo_heavy" type="button">Demo: heavy tail mixture</button>
                <button data-i18n="view.ad_norm.btn.demo_right"  id="adn-d3" class="secondary" data-tip="view.ad_norm.tip.demo_right" type="button">Demo: right-skew (half-normal)</button>
                <button data-i18n="view.ad_norm.btn.demo_left"   id="adn-d4" class="secondary" data-tip="view.ad_norm.tip.demo_left"  type="button">Demo: left-skew</button>
                <button data-i18n="view.ad_norm.btn.demo_unif"   id="adn-d5" class="secondary" data-tip="view.ad_norm.tip.demo_unif"  type="button">Demo: uniform</button>
                <button data-i18n="view.ad_norm.btn.demo_bimod"  id="adn-d6" class="secondary" data-tip="view.ad_norm.tip.demo_bimod" type="button">Demo: bimodal</button>
                <button data-i18n="view.ad_norm.btn.demo_exp"    id="adn-d7" class="secondary" data-tip="view.ad_norm.tip.demo_exp"   type="button">Demo: exponential</button>
                <button data-i18n="view.ad_norm.btn.demo_small"  id="adn-d8" class="secondary" data-tip="view.ad_norm.tip.demo_small" type="button">Demo: small sample (n=12)</button>
            </div>
            <p data-i18n="view.ad_norm.hint.about" class="muted">A² uses the full empirical CDF — more powerful in the tails than Jarque-Bera. Stephens (1986) small-sample correction is applied. Critical values: α=0.10 → 0.631; α=0.05 → 0.752; α=0.025 → 0.873; α=0.01 → 1.035.</p>
        </div>

        <div id="adn-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.ad_norm.h2.crit">Critical-value comparison</h2>
            <div id="adn-crit"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.ad_norm.h2.stats">Sample distribution</h2>
            <div id="adn-stats"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.ad_norm.h2.qq">Q-Q plot vs standard normal</h2>
            <div id="adn-qq" style="width:100%;height:280px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.ad_norm.h2.ecdf">Empirical vs theoretical normal CDF</h2>
            <div id="adn-ecdf" style="width:100%;height:240px"></div>
        </div>

        <div id="adn-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('adn-blob').value = sampleToBlob(state.sample);
    };
    document.getElementById('adn-d1').addEventListener('click', () => { loadDemo('gaussian');    void compute(tok); });
    document.getElementById('adn-d2').addEventListener('click', () => { loadDemo('heavy-tail');  void compute(tok); });
    document.getElementById('adn-d3').addEventListener('click', () => { loadDemo('right-skew');  void compute(tok); });
    document.getElementById('adn-d4').addEventListener('click', () => { loadDemo('left-skew');   void compute(tok); });
    document.getElementById('adn-d5').addEventListener('click', () => { loadDemo('uniform');     void compute(tok); });
    document.getElementById('adn-d6').addEventListener('click', () => { loadDemo('bimodal');     void compute(tok); });
    document.getElementById('adn-d7').addEventListener('click', () => { loadDemo('exponential'); void compute(tok); });
    document.getElementById('adn-d8').addEventListener('click', () => { loadDemo('small-sample'); void compute(tok); });
    document.getElementById('adn-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseSampleBlob(document.getElementById('adn-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.ad_norm.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        showToast(t('view.ad_norm.toast.parse_error'), { level: 'error' });
        return;
    }
    hideErr();
    state.sample = p.sample;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); showToast(t('view.ad_norm.toast.invalid'), { level: 'warning' }); return; }
    const local = localTest(state.sample);
    if (!local) { showErr(t('view.ad_norm.err.degenerate')); showToast(t('view.ad_norm.toast.degenerate'), { level: 'warning' }); return; }
    renderSummary(local, true);
    renderCrit(local);
    renderStats();
    renderQQ();
    renderECdf();
    let resp;
    try {
        resp = await api.anlyAdNormality(buildBody(state));
    } catch (e) {
        showErr(`${t('view.ad_norm.err.api')}: ${e.message || e}`);
        showToast(t('view.ad_norm.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) { showErr(t('view.ad_norm.err.server_rejected')); showToast(t('view.ad_norm.toast.server_rejected'), { level: 'error' }); return; }
    renderSummary(resp, false);
    renderCrit(resp);
    renderStats();
    showToast(t('view.ad_norm.toast.tested'), { level: 'success' });
}

function renderSummary(report, pending) {
    const local = localTest(state.sample);
    const parityOk = !!local
        && Math.abs(local.a_squared - report.a_squared) < 1e-9
        && Math.abs(local.a_squared_adjusted - report.a_squared_adjusted) < 1e-9
        && local.reject_at_5pct === report.reject_at_5pct
        && local.reject_at_1pct === report.reject_at_1pct
        && local.n_observations === report.n_observations;
    const vBadge = verdictBadge(report);
    const localTag = pending ? ` (${t('view.ad_norm.tag.local')})` : '';
    const p = approxPValue(report.a_squared_adjusted);
    document.getElementById('adn-summary').innerHTML = [
        card(t('view.ad_norm.card.verdict'),  t(vBadge.key) + localTag, vBadge.cls),
        card(t('view.ad_norm.card.a_sq'),     fmtNum(report.a_squared)),
        card(t('view.ad_norm.card.a_sq_adj'), fmtNum(report.a_squared_adjusted),
             report.a_squared_adjusted > 1.035 ? 'neg' : report.a_squared_adjusted < 0.631 ? 'pos' : ''),
        card(t('view.ad_norm.card.p_value'),  fmtPVal(p),
             p < 0.01 ? 'neg' : p < 0.05 ? '' : 'pos'),
        card(t('view.ad_norm.card.reject_5'), report.reject_at_5pct ? t('view.ad_norm.tag.yes') : t('view.ad_norm.tag.no'),
             report.reject_at_5pct ? 'neg' : 'pos'),
        card(t('view.ad_norm.card.reject_1'), report.reject_at_1pct ? t('view.ad_norm.tag.yes') : t('view.ad_norm.tag.no'),
             report.reject_at_1pct ? 'neg' : 'pos'),
        card(t('view.ad_norm.card.n'),        fmtInt(report.n_observations)),
        card(t('view.ad_norm.card.parity'),
             parityOk ? t('view.ad_norm.tag.ok') : t('view.ad_norm.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderCrit(report) {
    const a = report.a_squared_adjusted;
    const row = (key, alpha, crit) => {
        const exceed = a > crit;
        return `<tr>
            <td data-i18n="${key}">α=${alpha}</td>
            <td>${esc(fmtNum(crit, 3))}</td>
            <td class="${exceed ? 'neg' : 'pos'}">${esc(exceed ? t('view.ad_norm.tag.reject') : t('view.ad_norm.tag.accept'))}</td>
        </tr>`;
    };
    document.getElementById('adn-crit').innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.ad_norm.col.alpha">Level α</th>
                <th data-i18n="view.ad_norm.col.crit">Critical A²*</th>
                <th data-i18n="view.ad_norm.col.decision">Decision</th>
            </tr></thead>
            <tbody>
                ${row('view.ad_norm.alpha.10',  '0.10',  0.631)}
                ${row('view.ad_norm.alpha.5',   '0.05',  0.752)}
                ${row('view.ad_norm.alpha.025', '0.025', 0.873)}
                ${row('view.ad_norm.alpha.1',   '0.01',  1.035)}
            </tbody>
        </table>
    `;
}

function renderStats() {
    const wrap = document.getElementById('adn-stats');
    const sample = state.sample;
    if (!sample.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.ad_norm.empty">${esc(t('view.ad_norm.empty'))}</div>`;
        return;
    }
    const s = summarizeSample(sample);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.ad_norm.col.metric">Metric</th>
                <th data-i18n="view.ad_norm.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.ad_norm.row.count">Observations</td><td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.ad_norm.row.mean">Mean</td>         <td>${esc(fmtNumSigned(s.mean))}</td></tr>
                <tr><td data-i18n="view.ad_norm.row.sd">Std dev</td>        <td>${esc(fmtNum(s.sd))}</td></tr>
                <tr><td data-i18n="view.ad_norm.row.skew">Skewness</td>     <td>${esc(fmtNumSigned(s.skew))}</td></tr>
                <tr><td data-i18n="view.ad_norm.row.kurt">Excess kurtosis</td><td>${esc(fmtNumSigned(s.kurt))}</td></tr>
                <tr><td data-i18n="view.ad_norm.row.min">Min</td>           <td>${esc(fmtNumSigned(s.min))}</td></tr>
                <tr><td data-i18n="view.ad_norm.row.max">Max</td>           <td>${esc(fmtNumSigned(s.max))}</td></tr>
            </tbody>
        </table>
    `;
}

function renderQQ() {
    const el = document.getElementById('adn-qq');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const { theoretical, empirical, n } = qqPlotData(state.sample);
    if (n < 2) {
        el.innerHTML = `<div class="muted" data-i18n="view.ad_norm.empty_qq">${esc(t('view.ad_norm.empty_qq'))}</div>`;
        return;
    }
    // Diagonal reference line y = x scaled to the empirical span.
    const tmin = theoretical[0], tmax = theoretical[n - 1];
    const emin = Math.min(...empirical), emax = Math.max(...empirical);
    // Use empirical mean and stdev to map theoretical z -> y_ref so the
    // line is meaningful in the same units as the data.
    const mean = empirical.reduce((a, b) => a + b, 0) / n;
    const variance = empirical.reduce((a, b) => a + (b - mean) ** 2, 0) / Math.max(1, n - 1);
    const sd = Math.sqrt(variance);
    const refY = theoretical.map(z => mean + sd * z);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 260,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.ad_norm.chart.x') },
            { label: t('view.ad_norm.chart.empirical'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 6, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.ad_norm.chart.normal_line'),
              stroke: '#ffd84a', width: 1.5, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [theoretical, empirical, refY], el);
    void tmin; void tmax; void emin; void emax;
}

function normCdf(x) {
    const a1 =  0.254829592, a2 = -0.284496736, a3 =  1.421413741;
    const a4 = -1.453152027, a5 =  1.061405429, p  =  0.3275911;
    const sign = x < 0 ? -1 : 1;
    const ax = Math.abs(x) / Math.sqrt(2);
    const tt = 1.0 / (1.0 + p * ax);
    const y = 1.0 - (((((a5 * tt + a4) * tt) + a3) * tt + a2) * tt + a1) * tt * Math.exp(-ax * ax);
    return 0.5 * (1.0 + sign * y);
}

function renderECdf() {
    const el = document.getElementById('adn-ecdf');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const sample = state.sample;
    const n = sample.length;
    if (n < 2) {
        el.innerHTML = `<div class="muted" data-i18n="view.ad_norm.empty_ecdf">${esc(t('view.ad_norm.empty_ecdf'))}</div>`;
        return;
    }
    const sorted = [...sample].sort((a, b) => a - b);
    const mean = sorted.reduce((a, b) => a + b, 0) / n;
    const variance = sorted.reduce((a, b) => a + (b - mean) ** 2, 0) / Math.max(1, n - 1);
    const sd = Math.sqrt(variance);
    if (!(sd > 0)) {
        el.innerHTML = `<div class="muted" data-i18n="view.ad_norm.empty_ecdf">${esc(t('view.ad_norm.empty_ecdf'))}</div>`;
        return;
    }
    const xs = sorted;
    const ecdf = sorted.map((_, i) => (i + 0.5) / n);
    const ncdf = sorted.map(v => normCdf((v - mean) / sd));
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.ad_norm.chart.x') },
            { label: t('view.ad_norm.chart.ecdf_emp'),
              stroke: '#00e5ff', width: 1.5,
              points: { show: true, size: 4, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.ad_norm.chart.ecdf_norm'),
              stroke: '#ffd84a', width: 1.5, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ecdf, ncdf], el);
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('adn-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('adn-err').style.display = 'none'; }
