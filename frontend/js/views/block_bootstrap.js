// Block Bootstrap view — Künsch (1989) consecutive-block resampling.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_BLOCK_SIZE, DEFAULT_RESAMPLES, DEFAULT_SEED, STATISTICS,
    parseDataBlob, dataToBlob, validateInputs, buildBody, localBootstrap,
    ciBadge, biasBadge, signifBadge, summarizeData,
    makeDemoInput,
    fmtNum, fmtNumSigned, fmtInt,
} from '../_block_bootstrap_inputs.js';

let state = { ...makeDemoInput('mean-revert') };

export async function renderBlockBootstrap(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.block_boot.h1.title" class="view-title">// BLOCK BOOTSTRAP</h1>

        <div class="chart-panel" data-context-scope="block-bootstrap">
            <h2 data-i18n="view.block_boot.h2.data">Observation series
                <small data-i18n="view.block_boot.h2.data_hint" class="muted">(returns or P&amp;L increments — one per token)</small></h2>
            <textarea id="bb-blob" rows="6"
                      data-tip="view.block_boot.tip.data"
                      placeholder="0.012, -0.004, 0.008, ...">${esc(dataToBlob(state.data))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.block_boot.label.block">Block size</span>
                    <input id="bb-block" type="number" step="1" min="1" value="${state.block_size}"
                           data-tip="view.block_boot.tip.block"></label>
                <label><span data-i18n="view.block_boot.label.resamples">Resamples</span>
                    <input id="bb-resamples" type="number" step="1" min="50" max="10000" value="${state.n_resamples}"
                           data-tip="view.block_boot.tip.resamples"></label>
                <label><span data-i18n="view.block_boot.label.stat">Statistic</span>
                    <select id="bb-stat" data-tip="view.block_boot.tip.stat">
                        <option value="mean"           data-i18n="view.block_boot.stat.mean">mean</option>
                        <option value="stdev"          data-i18n="view.block_boot.stat.stdev">stdev</option>
                        <option value="sharpe_ratio"   data-i18n="view.block_boot.stat.sharpe">sharpe_ratio</option>
                        <option value="max_drawdown"   data-i18n="view.block_boot.stat.maxdd">max_drawdown</option>
                    </select></label>
                <label><span data-i18n="view.block_boot.label.seed">Seed</span>
                    <input id="bb-seed" type="number" step="1" value="${state.seed}"
                           data-tip="view.block_boot.tip.seed"></label>
                <button data-i18n="view.block_boot.btn.compute" id="bb-run" class="primary"
                        data-tip="view.block_boot.tip.compute" data-shortcut="block_bootstrap_run" type="button">Resample</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.block_boot.btn.demo_meanrevert" id="bb-d1" class="secondary" data-tip="view.block_boot.tip.demo_meanrevert" type="button">Demo: mean-revert AR(1)</button>
                <button data-i18n="view.block_boot.btn.demo_momentum"   id="bb-d2" class="secondary" data-tip="view.block_boot.tip.demo_momentum"   type="button">Demo: momentum AR(1)</button>
                <button data-i18n="view.block_boot.btn.demo_volclust"   id="bb-d3" class="secondary" data-tip="view.block_boot.tip.demo_volclust"   type="button">Demo: volatility clusters</button>
                <button data-i18n="view.block_boot.btn.demo_sharpe"     id="bb-d4" class="secondary" data-tip="view.block_boot.tip.demo_sharpe"     type="button">Demo: Sharpe strategy</button>
                <button data-i18n="view.block_boot.btn.demo_dd"         id="bb-d5" class="secondary" data-tip="view.block_boot.tip.demo_dd"         type="button">Demo: drawdown tail</button>
                <button data-i18n="view.block_boot.btn.demo_iid"        id="bb-d6" class="secondary" data-tip="view.block_boot.tip.demo_iid"        type="button">Demo: iid noise</button>
                <button data-i18n="view.block_boot.btn.demo_small"      id="bb-d7" class="secondary" data-tip="view.block_boot.tip.demo_small"      type="button">Demo: small sample</button>
                <button data-i18n="view.block_boot.btn.demo_fat"        id="bb-d8" class="secondary" data-tip="view.block_boot.tip.demo_fat"        type="button">Demo: fat tail</button>
            </div>
            <p data-i18n="view.block_boot.hint.about" class="muted">Draws blocks of consecutive observations with replacement to preserve serial dependence. Block size ≈ ⌊n^(1/3)⌋ for daily returns; larger blocks for stronger autocorrelation. Reports original statistic, bootstrap mean/stdev, and 95% percentile CI.</p>
        </div>

        <div id="bb-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.block_boot.h2.ci">95% confidence interval</h2>
            <div id="bb-ci"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.block_boot.h2.data_stats">Input series summary</h2>
            <div id="bb-table"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.block_boot.h2.series_chart">Input series</h2>
            <div id="bb-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.block_boot.h2.cum_chart">Cumulative sum (running equity / drawdown structure)</h2>
            <div id="bb-cum-chart" style="width:100%;height:220px"></div>
        </div>

        <div id="bb-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('bb-stat').value = state.statistic;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('bb-blob').value      = dataToBlob(state.data);
        document.getElementById('bb-block').value     = state.block_size;
        document.getElementById('bb-resamples').value = state.n_resamples;
        document.getElementById('bb-stat').value      = state.statistic;
        document.getElementById('bb-seed').value      = String(state.seed);
    };
    document.getElementById('bb-d1').addEventListener('click', () => { loadDemo('mean-revert');        void compute(tok); });
    document.getElementById('bb-d2').addEventListener('click', () => { loadDemo('momentum');           void compute(tok); });
    document.getElementById('bb-d3').addEventListener('click', () => { loadDemo('volatility-cluster'); void compute(tok); });
    document.getElementById('bb-d4').addEventListener('click', () => { loadDemo('sharpe-strategy');    void compute(tok); });
    document.getElementById('bb-d5').addEventListener('click', () => { loadDemo('drawdown-tail');      void compute(tok); });
    document.getElementById('bb-d6').addEventListener('click', () => { loadDemo('iid-noise');          void compute(tok); });
    document.getElementById('bb-d7').addEventListener('click', () => { loadDemo('small-sample');       void compute(tok); });
    document.getElementById('bb-d8').addEventListener('click', () => { loadDemo('fat-tail');           void compute(tok); });
    document.getElementById('bb-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseDataBlob(document.getElementById('bb-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.block_boot.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        showToast(t('view.block_boot.toast.parse_error'), { level: 'error' });
        return;
    }
    hideErr();
    state.data = p.data;
    const block = parseInt(document.getElementById('bb-block').value, 10);
    const resamples = parseInt(document.getElementById('bb-resamples').value, 10);
    const seedInput = document.getElementById('bb-seed').value;
    state.block_size = Number.isInteger(block) && block > 0 ? block : DEFAULT_BLOCK_SIZE;
    state.n_resamples = Number.isInteger(resamples) && resamples >= 50 ? resamples : DEFAULT_RESAMPLES;
    state.statistic = STATISTICS.includes(document.getElementById('bb-stat').value)
        ? document.getElementById('bb-stat').value : 'mean';
    let seed;
    try { seed = BigInt(seedInput); } catch { seed = DEFAULT_SEED; }
    state.seed = seed;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); showToast(t('view.block_boot.toast.invalid'), { level: 'warning' }); return; }
    const local = localBootstrap(state.data, state.block_size, state.n_resamples, state.statistic, state.seed);
    if (!local) { showErr(t('view.block_boot.err.degenerate')); showToast(t('view.block_boot.toast.degenerate'), { level: 'warning' }); return; }
    renderSummary(local, true);
    renderCi(local);
    renderTable();
    renderSeriesChart();
    renderCumChart();
    let resp;
    try {
        resp = await api.anlyBlockBootstrap(buildBody(state));
    } catch (e) {
        showErr(`${t('view.block_boot.err.api')}: ${e.message || e}`);
        showToast(t('view.block_boot.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) { showErr(t('view.block_boot.err.server_rejected')); showToast(t('view.block_boot.toast.server_rejected'), { level: 'error' }); return; }
    renderSummary(resp, false);
    renderCi(resp);
    renderTable();
    renderSeriesChart();
    renderCumChart();
    showToast(t('view.block_boot.toast.resampled'), { level: 'success' });
}

function renderSummary(report, pending) {
    const local = localBootstrap(state.data, state.block_size, state.n_resamples, state.statistic, state.seed);
    const parityOk = !!local
        && Math.abs(local.original_statistic - report.original_statistic) < 1e-9
        && Math.abs(local.bootstrap_mean - report.bootstrap_mean) < 1e-6
        && local.n_resamples === report.n_resamples
        && local.block_size  === report.block_size;
    const sBadge = signifBadge(report);
    const cBadge = ciBadge(report);
    const bBadge = biasBadge(report);
    const localTag = pending ? ` (${t('view.block_boot.tag.local')})` : '';
    document.getElementById('bb-summary').innerHTML = [
        card(t('view.block_boot.card.statistic'), t('view.block_boot.stat.' + state.statistic)),
        card(t('view.block_boot.card.verdict'),   t(sBadge.key) + localTag, sBadge.cls),
        card(t('view.block_boot.card.ci_width'),  t(cBadge.key), cBadge.cls),
        card(t('view.block_boot.card.bias'),      t(bBadge.key), bBadge.cls),
        card(t('view.block_boot.card.original'),  fmtNumSigned(report.original_statistic),
             report.original_statistic > 0 ? 'pos' : report.original_statistic < 0 ? 'neg' : ''),
        card(t('view.block_boot.card.mean'),      fmtNumSigned(report.bootstrap_mean)),
        card(t('view.block_boot.card.stdev'),     fmtNum(report.bootstrap_stdev)),
        card(t('view.block_boot.card.block'),     fmtInt(report.block_size)),
        card(t('view.block_boot.card.resamples'), fmtInt(report.n_resamples)),
        card(t('view.block_boot.card.parity'),
             parityOk ? t('view.block_boot.tag.ok') : t('view.block_boot.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderCi(report) {
    const wrap = document.getElementById('bb-ci');
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.block_boot.col.metric">Metric</th>
                <th data-i18n="view.block_boot.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.block_boot.row.lower">CI lower (2.5%)</td>
                    <td class="${report.ci_lower_2_5_pct < 0 ? 'neg' : 'pos'}">${esc(fmtNumSigned(report.ci_lower_2_5_pct))}</td></tr>
                <tr><td data-i18n="view.block_boot.row.upper">CI upper (97.5%)</td>
                    <td class="${report.ci_upper_97_5_pct < 0 ? 'neg' : 'pos'}">${esc(fmtNumSigned(report.ci_upper_97_5_pct))}</td></tr>
                <tr><td data-i18n="view.block_boot.row.width">CI width</td>
                    <td>${esc(fmtNum(report.ci_upper_97_5_pct - report.ci_lower_2_5_pct))}</td></tr>
                <tr><td data-i18n="view.block_boot.row.bias">Bias (boot − orig)</td>
                    <td>${esc(fmtNumSigned(report.bootstrap_mean - report.original_statistic))}</td></tr>
                <tr><td data-i18n="view.block_boot.row.se">Bootstrap SE</td>
                    <td>${esc(fmtNum(report.bootstrap_stdev))}</td></tr>
            </tbody>
        </table>
    `;
}

function renderTable() {
    const wrap = document.getElementById('bb-table');
    const data = state.data;
    if (!data.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.block_boot.empty">${esc(t('view.block_boot.empty'))}</div>`;
        return;
    }
    const s = summarizeData(data);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.block_boot.col.metric">Metric</th>
                <th data-i18n="view.block_boot.col.value">Value</th>
            </tr></thead>
            <tbody>
                <tr><td data-i18n="view.block_boot.row.count">Observations</td><td>${fmtInt(s.count)}</td></tr>
                <tr><td data-i18n="view.block_boot.row.sum">Sum</td>           <td class="${s.sum > 0 ? 'pos' : s.sum < 0 ? 'neg' : ''}">${esc(fmtNumSigned(s.sum))}</td></tr>
                <tr><td data-i18n="view.block_boot.row.mean">Mean</td>         <td class="${s.mean > 0 ? 'pos' : s.mean < 0 ? 'neg' : ''}">${esc(fmtNumSigned(s.mean))}</td></tr>
                <tr><td data-i18n="view.block_boot.row.max">Max</td>           <td class="pos">${esc(fmtNumSigned(s.max))}</td></tr>
                <tr><td data-i18n="view.block_boot.row.min">Min</td>           <td class="neg">${esc(fmtNumSigned(s.min))}</td></tr>
            </tbody>
        </table>
    `;
}

function renderSeriesChart() {
    const el = document.getElementById('bb-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const data = Array.isArray(state.data) ? state.data.filter(Number.isFinite) : [];
    if (data.length < 2) {
        el.innerHTML = `<div class="muted" data-i18n="view.block_boot.empty_chart">${esc(t('view.block_boot.empty_chart'))}</div>`;
        return;
    }
    const xs = data.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.block_boot.chart.obs_idx') },
            { label: t('view.block_boot.chart.value'),
              stroke: '#00e5ff', width: 1.2,
              points: { show: false } },
            { label: t('view.block_boot.chart.zero'),
              stroke: '#aab', width: 0.8, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, data, zero], el);
}

function renderCumChart() {
    const el = document.getElementById('bb-cum-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const data = Array.isArray(state.data) ? state.data.filter(Number.isFinite) : [];
    if (data.length < 2) {
        el.innerHTML = `<div class="muted" data-i18n="view.block_boot.empty_cum">${esc(t('view.block_boot.empty_cum'))}</div>`;
        return;
    }
    const xs = data.map((_, i) => i + 1);
    let acc = 0;
    let peak = -Infinity;
    const cum = new Array(data.length);
    const drawdown = new Array(data.length);
    for (let i = 0; i < data.length; i++) {
        acc += data[i];
        cum[i] = acc;
        if (acc > peak) peak = acc;
        drawdown[i] = acc - peak;
    }
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.block_boot.chart.obs_idx') },
            { label: t('view.block_boot.chart.cum'),
              stroke: '#7af0a8', width: 1.5, points: { show: false } },
            { label: t('view.block_boot.chart.dd'),
              stroke: '#ff3860', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, cum, drawdown], el);
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('bb-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('bb-err').style.display = 'none'; }
