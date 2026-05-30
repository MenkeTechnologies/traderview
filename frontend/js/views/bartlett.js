// Bartlett's Test for Equality of Variances view.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseGroupsBlob, groupsToBlob,
    validateInputs, buildBody, localTest,
    verdictBadge, ratioBadge, groupStats,
    makeDemoInput,
    fmtNum, fmtNumSigned, fmtPVal, fmtInt,
} from '../_bartlett_inputs.js';

let state = { ...makeDemoInput('equal') };

export async function renderBartlett(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.bartlett.h1.title" class="view-title">// BARTLETT VARIANCE TEST</h1>

        <div class="chart-panel" data-context-scope="bartlett">
            <h2 data-i18n="view.bartlett.h2.groups">Groups
                <small data-i18n="view.bartlett.h2.groups_hint" class="muted">(one group per line: LABEL v1 v2 v3 ...)</small></h2>
            <textarea id="bt-blob" rows="8"
                      data-tip="view.bartlett.tip.groups"
                      placeholder="A 1.1 0.9 1.0 ...\nB 0.95 1.05 ...">${esc(groupsToBlob(state.groups, state.labels))}</textarea>

            <div class="inline-form">
                <button data-i18n="view.bartlett.btn.compute" id="bt-run" class="primary"
                        data-tip="view.bartlett.tip.compute" type="button">Test</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.bartlett.btn.demo_equal"    id="bt-d1" class="secondary" type="button">Demo: equal variance</button>
                <button data-i18n="view.bartlett.btn.demo_mild"     id="bt-d2" class="secondary" type="button">Demo: mild diff (1.5×)</button>
                <button data-i18n="view.bartlett.btn.demo_strong"   id="bt-d3" class="secondary" type="button">Demo: strong diff (5×)</button>
                <button data-i18n="view.bartlett.btn.demo_3eq"      id="bt-d4" class="secondary" type="button">Demo: 3 equal groups</button>
                <button data-i18n="view.bartlett.btn.demo_3mix"     id="bt-d5" class="secondary" type="button">Demo: 3 mixed groups</button>
                <button data-i18n="view.bartlett.btn.demo_4reg"     id="bt-d6" class="secondary" type="button">Demo: 4 vol regimes</button>
                <button data-i18n="view.bartlett.btn.demo_small"    id="bt-d7" class="secondary" type="button">Demo: small groups (n=5)</button>
                <button data-i18n="view.bartlett.btn.demo_asym"     id="bt-d8" class="secondary" type="button">Demo: asymmetric sizes</button>
            </div>
            <p data-i18n="view.bartlett.hint.about" class="muted">Tests H₀: all k groups share the same variance. χ² = ((N−k)·ln(σ²_pooled) − Σ(n_i−1)·ln(σ²_i)) / correction. Under H₀, χ² ~ χ²(k−1). More powerful than Levene if data is normal — sensitive to non-normality.</p>
        </div>

        <div id="bt-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.bartlett.h2.groups_table">Per-group statistics</h2>
            <div id="bt-table"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.bartlett.h2.variance_chart">Group variances</h2>
            <div id="bt-chart" style="width:100%;height:280px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.bartlett.h2.mean_chart">Group means vs grand mean</h2>
            <div id="bt-mean-chart" style="width:100%;height:220px"></div>
        </div>

        <div id="bt-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        state = makeDemoInput(k);
        document.getElementById('bt-blob').value = groupsToBlob(state.groups, state.labels);
    };
    document.getElementById('bt-d1').addEventListener('click', () => { loadDemo('equal');          void compute(tok); });
    document.getElementById('bt-d2').addEventListener('click', () => { loadDemo('mild-diff');      void compute(tok); });
    document.getElementById('bt-d3').addEventListener('click', () => { loadDemo('strong-diff');    void compute(tok); });
    document.getElementById('bt-d4').addEventListener('click', () => { loadDemo('three-equal');    void compute(tok); });
    document.getElementById('bt-d5').addEventListener('click', () => { loadDemo('three-mixed');    void compute(tok); });
    document.getElementById('bt-d6').addEventListener('click', () => { loadDemo('four-volregime'); void compute(tok); });
    document.getElementById('bt-d7').addEventListener('click', () => { loadDemo('small-groups');   void compute(tok); });
    document.getElementById('bt-d8').addEventListener('click', () => { loadDemo('asymmetric-sizes');void compute(tok); });
    document.getElementById('bt-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parseGroupsBlob(document.getElementById('bt-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.bartlett.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        return;
    }
    hideErr();
    state.groups = p.groups;
    state.labels = p.labels;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); return; }
    const local = localTest(state.groups);
    if (!local) { showErr(t('view.bartlett.err.degenerate')); return; }
    renderSummary(local, true);
    renderTable();
    renderChart(local);
    renderMeanChart();
    let resp;
    try {
        resp = await api.anlyBartlettVariance(buildBody(state));
    } catch (e) {
        showErr(`${t('view.bartlett.err.api')}: ${e.message || e}`);
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) { showErr(t('view.bartlett.err.server_rejected')); return; }
    renderSummary(resp, false);
    renderTable();
    renderChart(resp);
    renderMeanChart();
}

function renderSummary(report, pending) {
    const local = localTest(state.groups);
    const parityOk = !!local
        && Math.abs(local.chi_squared_statistic - report.chi_squared_statistic) < 1e-6
        && Math.abs(local.p_value - report.p_value) < 1e-6
        && Math.abs(local.pooled_variance - report.pooled_variance) < 1e-9
        && local.n_groups === report.n_groups
        && local.n_total === report.n_total
        && local.reject_at_5pct === report.reject_at_5pct;
    const vBadge = verdictBadge(report);
    const rBadge = ratioBadge(state.groups);
    const localTag = pending ? ` (${t('view.bartlett.tag.local')})` : '';
    document.getElementById('bt-summary').innerHTML = [
        card(t('view.bartlett.card.verdict'),  t(vBadge.key) + localTag, vBadge.cls),
        card(t('view.bartlett.card.ratio'),    t(rBadge.key), rBadge.cls),
        card(t('view.bartlett.card.chi'),      fmtNum(report.chi_squared_statistic)),
        card(t('view.bartlett.card.dof'),      fmtNum(report.degrees_of_freedom, 1)),
        card(t('view.bartlett.card.p_value'),  fmtPVal(report.p_value),
             report.p_value < 0.01 ? 'neg' : report.p_value < 0.05 ? 'neg' : 'pos'),
        card(t('view.bartlett.card.reject'),
             report.reject_at_5pct ? t('view.bartlett.tag.yes') : t('view.bartlett.tag.no'),
             report.reject_at_5pct ? 'neg' : 'pos'),
        card(t('view.bartlett.card.pooled'),   fmtNum(report.pooled_variance)),
        card(t('view.bartlett.card.n_groups'), fmtInt(report.n_groups)),
        card(t('view.bartlett.card.n_total'),  fmtInt(report.n_total)),
        card(t('view.bartlett.card.parity'),
             parityOk ? t('view.bartlett.tag.ok') : t('view.bartlett.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderTable() {
    const wrap = document.getElementById('bt-table');
    const stats = groupStats(state.groups, state.labels);
    if (!stats.length) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.bartlett.empty">${esc(t('view.bartlett.empty'))}</div>`;
        return;
    }
    const variances = stats.map(s => s.variance).filter(v => Number.isFinite(v));
    const minVar = variances.length ? Math.min(...variances) : NaN;
    const rows = stats.map(s => {
        const ratioToMin = Number.isFinite(s.variance) && Number.isFinite(minVar) && minVar > 0
            ? s.variance / minVar : NaN;
        return `<tr>
            <td><strong>${esc(s.label)}</strong></td>
            <td>${fmtInt(s.n)}</td>
            <td>${esc(fmtNumSigned(s.mean))}</td>
            <td>${esc(fmtNum(s.variance))}</td>
            <td>${esc(fmtNum(s.sd))}</td>
            <td>${esc(fmtNumSigned(s.min))}</td>
            <td>${esc(fmtNumSigned(s.max))}</td>
            <td>${esc(fmtNum(ratioToMin, 2))}×</td>
        </tr>`;
    }).join('');
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.bartlett.col.label">Group</th>
                <th data-i18n="view.bartlett.col.n">n</th>
                <th data-i18n="view.bartlett.col.mean">Mean</th>
                <th data-i18n="view.bartlett.col.var">Variance</th>
                <th data-i18n="view.bartlett.col.sd">Std dev</th>
                <th data-i18n="view.bartlett.col.min">Min</th>
                <th data-i18n="view.bartlett.col.max">Max</th>
                <th data-i18n="view.bartlett.col.ratio">σ²/min(σ²)</th>
            </tr></thead>
            <tbody>${rows}</tbody>
        </table>
    `;
}

function renderMeanChart() {
    const el = document.getElementById('bt-mean-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const stats = groupStats(state.groups, state.labels);
    const means = stats.map(s => s.mean).filter(Number.isFinite);
    if (!means.length) {
        el.innerHTML = `<div class="muted" data-i18n="view.bartlett.empty_mean_chart">${esc(t('view.bartlett.empty_mean_chart'))}</div>`;
        return;
    }
    const xs = stats.map((_, i) => i + 1);
    const ys = stats.map(s => Number.isFinite(s.mean) ? s.mean : null);
    let totalN = 0, totalSum = 0;
    for (const s of stats) {
        if (Number.isFinite(s.mean) && Number.isFinite(s.n)) { totalSum += s.mean * s.n; totalN += s.n; }
    }
    const grand = totalN > 0 ? totalSum / totalN : NaN;
    const grandLine = Number.isFinite(grand) ? xs.map(() => grand) : null;
    const series = [
        { label: t('view.bartlett.chart.series.group_idx') },
        { label: t('view.bartlett.chart.series.mean'),
          stroke: '#7af0a8', width: 0,
          points: { show: true, size: 10, fill: '#7af0a8', stroke: '#7af0a8' } },
    ];
    const data = [xs, ys];
    if (grandLine) {
        series.push({ label: t('view.bartlett.chart.series.grand_mean'),
                      stroke: '#ffd84a', width: 1.5, dash: [4, 4],
                      points: { show: false } });
        data.push(grandLine);
    }
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series,
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => {
                  const i = Math.round(v) - 1;
                  return i >= 0 && i < stats.length ? stats[i].label : '';
              }) },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, data, el);
}

function renderChart(report) {
    const el = document.getElementById('bt-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const stats = groupStats(state.groups, state.labels);
    const variances = stats.map(s => s.variance).filter(Number.isFinite);
    if (!variances.length) {
        el.innerHTML = `<div class="muted" data-i18n="view.bartlett.empty_chart">${esc(t('view.bartlett.empty_chart'))}</div>`;
        return;
    }
    const xs = stats.map((_, i) => i + 1);
    const ys = stats.map(s => Number.isFinite(s.variance) ? s.variance : null);
    const pooled = (report && Number.isFinite(report.pooled_variance)) ? report.pooled_variance : null;
    const pooledYs = pooled != null ? xs.map(() => pooled) : null;
    const series = [
        { label: t('view.bartlett.chart.series.group_idx') },
        { label: t('view.bartlett.chart.series.variance'),
          stroke: '#00e5ff', width: 0,
          points: { show: true, size: 10, fill: '#00e5ff', stroke: '#00e5ff' } },
    ];
    const data = [xs, ys];
    if (pooledYs) {
        series.push({ label: t('view.bartlett.chart.series.pooled'),
                      stroke: '#ffd84a', width: 1.5, dash: [4, 4],
                      points: { show: false } });
        data.push(pooledYs);
    }
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 260,
        scales: { x: {}, y: { auto: true } },
        series,
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => stats[Math.round(v) - 1]?.label || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, data, el);
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('bt-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('bt-err').style.display = 'none'; }
