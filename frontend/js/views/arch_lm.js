// Engle's ARCH-LM test view (conditional heteroscedasticity).
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_LAGS, MIN_LAGS, MAX_LAGS,
    parseReturnsBlob, returnsToBlob, validateInputs, buildBody, localTest,
    chi2Critical, chi2PValue, verdictBadge, r2Badge, summarizeReturns,
    makeDemoInput,
    fmtNum, fmtNumSigned, fmtPct, fmtPVal, fmtInt,
} from '../_arch_lm_inputs.js';

let state = { ...makeDemoInput('arch-strong') };

export async function renderArchLm(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.arch_lm.h1.title" class="view-title">// ARCH-LM TEST</h1>

        <div class="chart-panel" data-context-scope="arch-lm">
            <h2 data-i18n="view.arch_lm.h2.returns">Returns
                <small data-i18n="view.arch_lm.h2.returns_hint" class="muted">(≥ 3·lags + 2 obs; whitespace/comma-separated)</small></h2>
            <textarea id="arl-blob" rows="6"
                      data-tip="view.arch_lm.tip.returns"
                      placeholder="0.012, -0.004, 0.008, ...">${esc(returnsToBlob(state.returns))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.arch_lm.label.lags">Lags (q)</span>
                    <input id="arl-lags" type="number" step="1" min="${MIN_LAGS}" max="${MAX_LAGS}" value="${state.lags}"
                           data-tip="view.arch_lm.tip.lags"></label>
                <button data-i18n="view.arch_lm.btn.compute" id="arl-run" class="primary"
                        data-tip="view.arch_lm.tip.compute" data-shortcut="arch_lm_run" type="button">Test</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.arch_lm.btn.demo_strong" id="arl-d1" class="secondary" data-tip="view.arch_lm.tip.demo_strong" type="button">Demo: ARCH(1) strong</button>
                <button data-i18n="view.arch_lm.btn.demo_mild"   id="arl-d2" class="secondary" data-tip="view.arch_lm.tip.demo_mild"   type="button">Demo: ARCH(1) mild</button>
                <button data-i18n="view.arch_lm.btn.demo_garch"  id="arl-d3" class="secondary" data-tip="view.arch_lm.tip.demo_garch"  type="button">Demo: GARCH(1,1)-like</button>
                <button data-i18n="view.arch_lm.btn.demo_iid"    id="arl-d4" class="secondary" data-tip="view.arch_lm.tip.demo_iid"    type="button">Demo: iid Gaussian</button>
                <button data-i18n="view.arch_lm.btn.demo_lap"    id="arl-d5" class="secondary" data-tip="view.arch_lm.tip.demo_lap"    type="button">Demo: iid Laplace</button>
                <button data-i18n="view.arch_lm.btn.demo_regime" id="arl-d6" class="secondary" data-tip="view.arch_lm.tip.demo_regime" type="button">Demo: short-memory regime</button>
                <button data-i18n="view.arch_lm.btn.demo_few"    id="arl-d7" class="secondary" data-tip="view.arch_lm.tip.demo_few"    type="button">Demo: few obs (n=25)</button>
                <button data-i18n="view.arch_lm.btn.demo_high"   id="arl-d8" class="secondary" data-tip="view.arch_lm.tip.demo_high"   type="button">Demo: high lags (q=10)</button>
            </div>
            <p data-i18n="view.arch_lm.hint.about" class="muted">Regresses ê²ₜ on q lagged squared residuals. LM = (n−q)·R² ~ χ²(q) under H₀ (homoscedasticity). Reject when LM exceeds tabulated χ²(q) critical value — implies GARCH-style modeling is warranted.</p>
        </div>

        <div id="arl-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.arch_lm.h2.crit">χ²(q) critical comparison</h2>
            <div id="arl-crit"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.arch_lm.h2.stats">Returns summary</h2>
            <div id="arl-stats"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.arch_lm.h2.sq_returns">Squared returns (volatility clustering)</h2>
            <div id="arl-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.arch_lm.h2.roll_chart">Rolling volatility (window σ)</h2>
            <div id="arl-roll-chart" style="width:100%;height:220px"></div>
        </div>

        <div id="arl-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('arl-blob').value = returnsToBlob(state.returns);
        document.getElementById('arl-lags').value = state.lags;
    };
    document.getElementById('arl-d1').addEventListener('click', () => { loadDemo('arch-strong');        void compute(tok); });
    document.getElementById('arl-d2').addEventListener('click', () => { loadDemo('arch-mild');          void compute(tok); });
    document.getElementById('arl-d3').addEventListener('click', () => { loadDemo('garch-like');         void compute(tok); });
    document.getElementById('arl-d4').addEventListener('click', () => { loadDemo('iid-gauss');          void compute(tok); });
    document.getElementById('arl-d5').addEventListener('click', () => { loadDemo('iid-laplace');        void compute(tok); });
    document.getElementById('arl-d6').addEventListener('click', () => { loadDemo('short-memory-vol');   void compute(tok); });
    document.getElementById('arl-d7').addEventListener('click', () => { loadDemo('few-obs');            void compute(tok); });
    document.getElementById('arl-d8').addEventListener('click', () => { loadDemo('high-lags');          void compute(tok); });
    document.getElementById('arl-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseReturnsBlob(document.getElementById('arl-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.arch_lm.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        showToast(t('view.arch_lm.toast.parse_error'), { level: 'error' });
        return;
    }
    hideErr();
    state.returns = p.returns;
    const lagsV = parseInt(document.getElementById('arl-lags').value, 10);
    state.lags = Number.isInteger(lagsV) && lagsV >= MIN_LAGS && lagsV <= MAX_LAGS ? lagsV : DEFAULT_LAGS;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); showToast(t('view.arch_lm.toast.invalid'), { level: 'warning' }); return; }
    const local = localTest(state.returns, state.lags);
    if (!local) { showErr(t('view.arch_lm.err.degenerate')); showToast(t('view.arch_lm.toast.degenerate'), { level: 'warning' }); return; }
    renderSummary(local, true);
    renderCrit(local);
    renderStats();
    renderSqChart();
    renderRollChart();
    let resp;
    try {
        resp = await api.anlyArchLm(buildBody(state));
    } catch (e) {
        showErr(`${t('view.arch_lm.err.api')}: ${e.message || e}`);
        showToast(t('view.arch_lm.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) { showErr(t('view.arch_lm.err.server_rejected')); showToast(t('view.arch_lm.toast.server_rejected'), { level: 'error' }); return; }
    renderSummary(resp, false);
    renderCrit(resp);
    renderStats();
    renderSqChart();
    renderRollChart();
    showToast(t('view.arch_lm.toast.tested'), { level: 'success' });
}

function renderSummary(report, pending) {
    const local = localTest(state.returns, state.lags);
    const parityOk = !!local
        && Math.abs(local.lm_statistic - report.lm_statistic) < 1e-6
        && Math.abs(local.r_squared - report.r_squared) < 1e-9
        && local.lags === report.lags
        && local.n_observations === report.n_observations;
    const vBadge = verdictBadge(report);
    const rBadge = r2Badge(report.r_squared);
    const pVal = chi2PValue(report.lm_statistic, report.lags);
    const localTag = pending ? ` (${t('view.arch_lm.tag.local')})` : '';
    document.getElementById('arl-summary').innerHTML = [
        card(t('view.arch_lm.card.verdict'),  t(vBadge.key) + localTag, vBadge.cls),
        card(t('view.arch_lm.card.lm'),       fmtNum(report.lm_statistic),
             vBadge.cls),
        card(t('view.arch_lm.card.p_value'),  fmtPVal(pVal),
             pVal < 0.01 ? 'neg' : pVal < 0.05 ? '' : 'pos'),
        card(t('view.arch_lm.card.r2'),       fmtPct(report.r_squared), rBadge.cls),
        card(t('view.arch_lm.card.r2_strength'), t(rBadge.key), rBadge.cls),
        card(t('view.arch_lm.card.lags'),     fmtInt(report.lags)),
        card(t('view.arch_lm.card.n'),        fmtInt(report.n_observations)),
        card(t('view.arch_lm.card.parity'),
             parityOk ? t('view.arch_lm.tag.ok') : t('view.arch_lm.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderCrit(report) {
    const c = chi2Critical(report.lags);
    const lm = report.lm_statistic;
    const row = (key, alpha, crit) => {
        const exceed = lm > crit;
        return `<tr>
            <td data-i18n="${key}">α=${alpha}</td>
            <td>${esc(fmtNum(crit, 3))}</td>
            <td class="${exceed ? 'neg' : 'pos'}">${esc(exceed ? t('view.arch_lm.tag.reject') : t('view.arch_lm.tag.accept'))}</td>
        </tr>`;
    };
    document.getElementById('arl-crit').innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.arch_lm.col.alpha">Level α</th>
                <th data-i18n="view.arch_lm.col.crit">Critical χ²(${report.lags})</th>
                <th data-i18n="view.arch_lm.col.decision">Decision</th>
            </tr></thead>
            <tbody>
                ${row('view.arch_lm.alpha.10', '0.10', c.a10)}
                ${row('view.arch_lm.alpha.5',  '0.05', c.a5)}
                ${row('view.arch_lm.alpha.1',  '0.01', c.a1)}
            </tbody>
        </table>
    `;
}

function renderStats() {
    const wrap = document.getElementById('arl-stats');
    const ret = state.returns;
    if (!ret.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.arch_lm.empty">${esc(t('view.arch_lm.empty'))}</div>`;
        return;
    }
    const s = summarizeReturns(ret);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.arch_lm.col.metric">Metric</th>
                <th data-i18n="view.arch_lm.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.arch_lm.row.count">Observations</td><td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.arch_lm.row.mean">Mean</td>         <td>${esc(fmtNumSigned(s.mean))}</td></tr>
                <tr><td data-i18n="view.arch_lm.row.sd">Std dev</td>        <td>${esc(fmtNum(s.sd))}</td></tr>
                <tr><td data-i18n="view.arch_lm.row.min">Min</td>           <td>${esc(fmtNumSigned(s.min))}</td></tr>
                <tr><td data-i18n="view.arch_lm.row.max">Max</td>           <td>${esc(fmtNumSigned(s.max))}</td></tr>
            </tbody>
        </table>
    `;
}

function renderRollChart() {
    const el = document.getElementById('arl-roll-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const series = Array.isArray(state.returns) ? state.returns.filter(Number.isFinite) : [];
    if (series.length < 4) {
        el.innerHTML = `<div class="muted" data-i18n="view.arch_lm.empty_roll_chart">${esc(t('view.arch_lm.empty_roll_chart'))}</div>`;
        return;
    }
    const win = Math.min(20, Math.max(3, Math.floor(series.length / 5)));
    const xs = series.map((_, i) => i + 1);
    const roll = series.map((_, i) => {
        if (i + 1 < win) return null;
        let sum = 0;
        for (let j = i - win + 1; j <= i; j++) sum += series[j];
        const m = sum / win;
        let varSum = 0;
        for (let j = i - win + 1; j <= i; j++) varSum += (series[j] - m) ** 2;
        return Math.sqrt(varSum / Math.max(1, win - 1));
    });
    const mean = series.reduce((a, b) => a + b, 0) / series.length;
    let totalVar = 0;
    for (const v of series) totalVar += (v - mean) ** 2;
    const globalSd = Math.sqrt(totalVar / Math.max(1, series.length - 1));
    const globalLine = xs.map(() => globalSd);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.arch_lm.chart.bar_idx') },
            { label: t('view.arch_lm.chart.roll_sd'),
              stroke: '#7af0a8', width: 1.5, points: { show: false } },
            { label: t('view.arch_lm.chart.global_sd'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, roll, globalLine], el);
}

function renderSqChart() {
    const el = document.getElementById('arl-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const series = Array.isArray(state.returns) ? state.returns.filter(Number.isFinite) : [];
    if (series.length < 2) {
        el.innerHTML = `<div class="muted" data-i18n="view.arch_lm.empty_chart">${esc(t('view.arch_lm.empty_chart'))}</div>`;
        return;
    }
    const mean = series.reduce((a, b) => a + b, 0) / series.length;
    const sqResiduals = series.map(r => (r - mean) ** 2);
    const xs = sqResiduals.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.arch_lm.chart.bar_idx') },
            { label: t('view.arch_lm.chart.sq_residual'),
              stroke: '#00e5ff', width: 1.0,
              fill: 'rgba(0,229,255,0.12)',
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, sqResiduals], el);
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('arl-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('arl-err').style.display = 'none'; }
