// Yield Curve PCA view — Litterman-Scheinkman (1991) factor decomposition
// of a yield-curve history. Paste a T×N matrix of yields (T dates × N
// tenors) and the backend runs PCA on first differences. Empirical
// result for G7 sovereigns: the first 3 PCs explain >95% of variance,
// canonically labeled Level / Slope / Curvature.

import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';
import {
    parseCurves, parseTenorLabels, validatePcaInputs,
    factorName, normalizeTenors, buildBody, factorColor,
} from '../_yield_curve_pca_inputs.js';

const DEFAULT_CURVES = `# Yield curve history. One row = one date's curve at every tenor.
# Whitespace OR comma separator. # comments OK.
# Demo: 20 dates × 6 tenors with a level shift, slope steepening, and
# curvature wiggle baked in.
1.0  1.5  2.0  2.3  2.6  2.8
1.1  1.6  2.1  2.4  2.7  2.9
1.0  1.5  2.0  2.3  2.6  2.8
1.2  1.6  2.0  2.3  2.6  2.9
1.3  1.7  2.1  2.4  2.7  3.0
1.4  1.8  2.2  2.5  2.8  3.1
1.4  1.8  2.3  2.6  2.9  3.0
1.5  1.9  2.4  2.7  3.0  3.1
1.6  1.9  2.4  2.7  3.0  3.0
1.7  2.0  2.4  2.7  3.0  3.0
1.6  2.1  2.5  2.8  3.0  3.0
1.5  2.2  2.6  2.9  3.0  3.0
1.6  2.3  2.7  3.0  3.1  3.0
1.7  2.3  2.6  2.8  3.0  3.1
1.8  2.4  2.7  2.9  3.1  3.2
1.9  2.4  2.7  3.0  3.2  3.3
2.0  2.5  2.8  3.0  3.2  3.3
2.0  2.5  2.7  3.0  3.2  3.4
2.1  2.6  2.8  3.0  3.2  3.4
2.2  2.6  2.8  3.1  3.3  3.5
`;
const DEFAULT_TENORS = `# Tenor labels (one per line). Optional — defaults to T1..Tn.
3M
1Y
2Y
5Y
10Y
30Y
`;

let state = {
    curvesText: DEFAULT_CURVES,
    tenorsText: DEFAULT_TENORS,
    topK: 3,
};

export async function renderYieldCurvePca(mount, _appState) {
    const tok = currentViewToken();

    mount.innerHTML = `
        <h1 data-i18n="view.yield_curve_pca.h1.yield_curve_pca" class="view-title">// YIELD CURVE PCA</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.yield_curve_pca.h2.inputs">Inputs</h2>
            <div class="op-inputs-grid">
                <div>
                    <h3 data-i18n="view.yield_curve_pca.h3.yield_curves_t_n">Yield curves (T × N)</h3>
                    <textarea id="yp-curves" rows="14"
                        style="width:100%;font-family:monospace;font-size:13px">${esc(state.curvesText)}</textarea>
                </div>
                <div>
                    <h3 data-i18n="view.yield_curve_pca.h3.tenor_labels">Tenor labels</h3>
                    <textarea id="yp-tenors" rows="14"
                        style="width:100%;font-family:monospace;font-size:13px">${esc(state.tenorsText)}</textarea>
                </div>
            </div>
            <div class="inline-form" style="margin-top:10px">
                <label><span data-i18n="view.yield_curve_pca.label.topk">top_k factors</span>
                    <input id="yp-topk" type="number" step="1" min="1" max="20" value="${state.topK}"></label>
                <button data-i18n="view.yield_curve_pca.btn.decompose" id="yp-run" class="primary" type="button">Decompose</button>
            </div>
        </div>

        <div id="yp-parse-errors" class="boot" style="display:none;color:var(--red)"></div>

        <div id="yp-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.yield_curve_pca.h2.factor_loadings_vs_tenor">Factor loadings vs tenor</h2>
            <div id="yp-loadings-chart" style="width:100%;height:340px"></div>
            <p data-i18n="view.yield_curve_pca.hint.each_line_is_one_principal_component_s_loading_acr" class="muted">
                Each line is one principal component's loading across tenors. PC1 should be
                approximately flat (level shift), PC2 monotone (slope), PC3 bowed (curvature).
            </p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.yield_curve_pca.h2.variance_explained_per_factor">Variance explained per factor</h2>
            <div id="yp-variance-chart" style="width:100%;height:240px"></div>
            <p data-i18n="view.yield_curve_pca.hint.empirically_the_first_3_factors_carry_95_of_curve_" class="muted">
                Empirically the first 3 factors carry &gt;95% of curve-change variance.
            </p>
        </div>

        <div id="yp-err" class="boot" style="display:none;color:var(--red)"></div>
    `;

    document.getElementById('yp-run').addEventListener('click', () => {
        state.curvesText = document.getElementById('yp-curves').value;
        state.tenorsText = document.getElementById('yp-tenors').value;
        state.topK = parseInt(document.getElementById('yp-topk').value, 10);
        void run(mount, tok);
    });
    void fmt;
}

async function run(mount, tok) {
    hideErrs();
    const parsed = parseCurves(state.curvesText);
    if (parsed.errors.length) renderParseErrors(parsed.errors);

    const err = validatePcaInputs(parsed.value, state.topK);
    if (err) { showErr(err); return; }

    const tenors = normalizeTenors(parseTenorLabels(state.tenorsText), parsed.value[0].length);
    let res;
    try {
        res = await api.anlyPrincipalComponentYieldCurve(buildBody(parsed.value, state.topK));
        if (!res) throw new Error(t('view.yield_curve_pca.error.null_result'));
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        return;
    }
    if (!viewIsCurrent(tok)) return;

    renderSummary(res);
    renderLoadingsChart(tenors, res);
    renderVarianceChart(res);
}

function renderSummary(res) {
    const cards = res.variance_explained.map((vexp, i) => {
        return `<div class="card">
            <div class="label"><span class="yp-swatch yp-pc-${i % 6}">▮</span> ${esc(factorName(i))}</div>
            <div class="value">${(vexp * 100).toFixed(1)}%</div>
            <div class="value yp-summary-value">
                <div class="vc-row"><span class="muted" data-i18n="view.yield_curve_pca.row.cumulative">cumulative</span>
                    <strong>${(res.cumulative_variance[i] * 100).toFixed(1)}%</strong></div>
                <div class="vc-row"><span class="muted" data-i18n="view.yield_curve_pca.row.eigenvalue">eigenvalue</span>
                    <strong>${res.eigenvalues[i].toFixed(6)}</strong></div>
            </div>
        </div>`;
    });
    document.getElementById('yp-summary').innerHTML = cards.join('');
}

function renderLoadingsChart(tenors, res) {
    const el = document.getElementById('yp-loadings-chart');
    if (!window.uPlot) { el.textContent = t('common.error.uplot_not_loaded'); return; }
    el.innerHTML = '';
    // X axis = tenor index (0..N-1). Plot one line per factor.
    const xs = tenors.map((_, i) => i);
    const data = [xs];
    const series = [{ label: 'tenor idx' }];
    for (let i = 0; i < res.loadings.length; i++) {
        series.push({
            label: factorName(i),
            stroke: factorColor(i),
            width: 2,
            points: { show: true, size: 6, stroke: factorColor(i), fill: factorColor(i) },
        });
        data.push(res.loadings[i]);
    }
    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 340,
        scales: { x: {}, y: {} },
        series,
        axes: [
            { stroke: '#aab',
              values: (_, ticks) => ticks.map(t => tenors[Math.round(t)] ?? '') },
            { stroke: '#aab' },
        ],
    }, data, el);
}

function renderVarianceChart(res) {
    const el = document.getElementById('yp-variance-chart');
    if (!window.uPlot) { el.textContent = t('common.error.uplot_not_loaded'); return; }
    el.innerHTML = '';
    const xs = res.variance_explained.map((_, i) => i);
    // Render as line + points instead of bar chart (uPlot doesn't ship a
    // bar primitive without the bars plugin and we want zero deps).
    const ys = res.variance_explained;
    const cumYs = res.cumulative_variance;
    new window.uPlot({
        title: '', width: el.clientWidth || 800, height: 240,
        scales: { x: {}, y: {} },
        series: [
            { label: 'factor idx' },
            { label: 'per-factor', stroke: '#00e5ff', width: 2,
              points: { show: true, size: 8, stroke: '#00e5ff', fill: '#00e5ff' } },
            { label: 'cumulative', stroke: '#ff9f1a', width: 2,
              points: { show: true, size: 6, stroke: '#ff9f1a', fill: '#ff9f1a' } },
        ],
        axes: [
            { stroke: '#aab',
              values: (_, ticks) => ticks.map(t => factorName(Math.round(t))) },
            { stroke: '#aab',
              values: (_, ticks) => ticks.map(t => `${(t * 100).toFixed(0)}%`) },
        ],
    }, [xs, ys, cumYs], el);
}

function renderParseErrors(errors) {
    const el = document.getElementById('yp-parse-errors');
    el.innerHTML = errors.slice(0, 20).map(e =>
        `<div>line ${e.line_no}: ${esc(e.message)} <span class="muted">→ <code>${esc(e.raw || '')}</code></span></div>`
    ).join('');
    if (errors.length > 20) {
        el.innerHTML += `<div class="muted">… (+${errors.length - 20} more)</div>`;
    }
    el.style.display = 'block';
}

function showErr(msg) {
    const el = document.getElementById('yp-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErrs() {
    document.getElementById('yp-parse-errors').style.display = 'none';
    document.getElementById('yp-err').style.display = 'none';
}
