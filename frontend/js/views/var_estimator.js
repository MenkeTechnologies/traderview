// Value-at-Risk (VaR) view — runs BOTH historical + parametric-Gaussian
// methods on the same return series, with side-by-side comparison +
// loss-distribution histogram annotated with VaR + ES lines.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseReturnsBlob, validateInputs, buildBody,
    localHistorical, localParametricGaussian, distributionStats,
    lossHistogram, compareMethods, makeDemoReturns,
    fmtUSD, fmtUSDSigned, fmtPct, fmtN,
} from '../_var_estimator_inputs.js';

import { t } from '../i18n.js';
import { showToast } from '../toast.js';
let state = {
    returns: makeDemoReturns('normal'),
    positionValue: 100_000,
    confidence: 0.95,
};

export async function renderVarEstimator(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.var_estimator.h1.value_at_risk" class="view-title">// VALUE AT RISK</h1>

        <div class="chart-panel">
            <h2><span data-i18n="view.var_estimator.h2.daily_returns">Daily returns</span> <small class="muted" data-i18n="view.var_estimator.h2.daily_returns_hint">(decimal or with % suffix; csv/space/newline mix)</small></h2>
            <textarea id="var-blob" rows="6" placeholder="0.005&#10;-0.012&#10;0.003&#10;-0.5%&#10;..." data-tip="view.var_estimator.tip.blob">${esc(returnsToBlob(state.returns))}</textarea>
            <div class="inline-form">
                <label><span data-i18n="view.var_estimator.label.position_value">Position value ($)</span>
                    <input id="var-pv" type="number" step="any" min="0" value="${state.positionValue}" data-tip="view.var_estimator.tip.pv"></label>
                <label><span data-i18n="view.var_estimator.label.confidence">Confidence</span>
                    <small class="muted" data-i18n="view.var_estimator.hint.confidence">(0.95 / 0.99 / 0.999)</small>
                    <select id="var-conf" data-tip="view.var_estimator.tip.conf">
                        <option value="0.90"  ${state.confidence === 0.90  ? 'selected' : ''}>90%</option>
                        <option value="0.95"  ${state.confidence === 0.95  ? 'selected' : ''}>95%</option>
                        <option value="0.99"  ${state.confidence === 0.99  ? 'selected' : ''}>99%</option>
                        <option value="0.999" ${state.confidence === 0.999 ? 'selected' : ''}>99.9%</option>
                    </select></label>
                <button data-i18n="view.var_estimator.btn.compute" id="var-run" class="primary" type="button" data-tip="view.var_estimator.tip.run" data-shortcut="var_estimator_run">Compute</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.var_estimator.btn.demo_normal_returns" id="var-demo-normal"    class="secondary" type="button" data-tip="view.var_estimator.tip.demo_normal">Demo: normal returns</button>
                <button data-i18n="view.var_estimator.btn.demo_fat_tailed" id="var-demo-fat"       class="secondary" type="button" data-tip="view.var_estimator.tip.demo_fat">Demo: fat-tailed</button>
                <button data-i18n="view.var_estimator.btn.demo_30_day_crisis_embedded" id="var-demo-crisis"    class="secondary" type="button" data-tip="view.var_estimator.tip.demo_crisis">Demo: 30-day crisis embedded</button>
                <button data-i18n="view.var_estimator.btn.demo_low_vol_drift" id="var-demo-low-vol"   class="secondary" type="button" data-tip="view.var_estimator.tip.demo_low_vol">Demo: low-vol drift</button>
                <button data-i18n="view.var_estimator.btn.demo_random_walk" id="var-demo-walk"      class="secondary" type="button" data-tip="view.var_estimator.tip.demo_walk">Demo: random walk</button>
            </div>
            <p data-i18n="view.var_estimator.hint.historical_empirical_percentile_of_loss_distributi" class="muted">Historical = empirical percentile of loss distribution. Parametric assumes Gaussian (μ ± z·σ). When historical &gt; parametric, fat tails are punishing your model. Expected Shortfall (CVaR) = mean loss BEYOND VaR — always ≥ VaR.</p>
        </div>

        <div id="var-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.var_estimator.h2.method_comparison">Method comparison</h2>
            <div id="var-compare"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.var_estimator.h2.loss_distribution_positive_numbers_losses">Loss distribution (positive numbers = $ losses)</h2>
            <div id="var-hist" style="height:340px"></div>
            <p data-i18n="view.var_estimator.hint.histogram_of_daily_losses_yellow_dashed_historical" class="muted">Histogram of daily losses. Yellow dashed = historical VaR, red dashed = historical ES, cyan dashed = parametric VaR.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.var_estimator.h2.rolling_vol_chart">Rolling 20-day volatility with mean-σ reference</h2>
            <div id="var-rollvol-chart" style="width:100%;height:240px"></div>
            <p data-i18n="view.var_estimator.hint.rolling_vol_chart" class="muted small">20-day rolling stdev of returns. The yellow dashed line is the full-sample σ used by the parametric Gaussian VaR. Spikes above the line are vol-regime shifts that the parametric estimate under-prices. Orthogonal to the binned distribution above.</p>
        </div>

        <div id="var-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (kind) => {
        state.returns = makeDemoReturns(kind);
        document.getElementById('var-blob').value = returnsToBlob(state.returns);
    };
    document.getElementById('var-demo-normal').addEventListener('click',  () => loadDemo('normal'));
    document.getElementById('var-demo-fat').addEventListener('click',     () => loadDemo('fat-tail'));
    document.getElementById('var-demo-crisis').addEventListener('click',  () => loadDemo('crisis'));
    document.getElementById('var-demo-low-vol').addEventListener('click', () => loadDemo('low-vol'));
    document.getElementById('var-demo-walk').addEventListener('click',    () => loadDemo('random-walk'));
    document.getElementById('var-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function returnsToBlob(returns) {
    return returns.map(r => r.toFixed(6)).join('\n');
}

function readInputs() {
    const parsed = parseReturnsBlob(document.getElementById('var-blob').value);
    if (parsed.errors.length) {
        showErr(t("common.error.parse_errors", { summary: parsed.errors.slice(0, 3).map(e => `[] `).join("; ") }));
        showToast(t('view.var_estimator.toast.parse_error', { n: parsed.errors.length }), { level: 'warning' });
        return;
    }
    hideErr();
    state.returns       = parsed.returns;
    state.positionValue = Number(document.getElementById('var-pv').value);
    state.confidence    = Number(document.getElementById('var-conf').value);
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.returns, state.positionValue, state.confidence);
    if (err) { showErr(err); showToast(t('view.var_estimator.toast.invalid'), { level: 'warning' }); return; }
    const localHist  = localHistorical(state.returns, state.positionValue, state.confidence);
    const localGauss = localParametricGaussian(state.returns, state.positionValue, state.confidence);
    renderSummary(localHist, localGauss, true);
    renderCompare(localHist, localGauss);
    renderHistogram(state.returns, state.positionValue, localHist, localGauss);
    renderRollingVolChart(state.returns);
    let hist, gauss;
    try {
        [hist, gauss] = await Promise.all([
            api.calcVarHistorical(buildBody(state.returns, state.positionValue, state.confidence)),
            api.calcVarGaussian(buildBody(state.returns, state.positionValue, state.confidence)),
        ]);
    } catch (e) {
        showErr(t("common.error.api", { msg: e.message || e }));
        showToast(t('view.var_estimator.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(hist, gauss, false);
    renderCompare(hist, gauss);
    renderHistogram(state.returns, state.positionValue, hist, gauss);
    const fatTail = hist.var_dollars > gauss.var_dollars * 1.05;
    showToast(t('view.var_estimator.toast.computed', {
        hist: Math.round(hist.var_dollars).toLocaleString(),
        gauss: Math.round(gauss.var_dollars).toLocaleString(),
    }), { level: fatTail ? 'warning' : 'success' });
}

function renderSummary(hist, gauss, pending) {
    const stats = distributionStats(state.returns);
    const localHist  = localHistorical(state.returns, state.positionValue, state.confidence);
    const localGauss = localParametricGaussian(state.returns, state.positionValue, state.confidence);
    const parityOk = Math.abs(hist.var_dollars  - localHist.var_dollars)  < 1e-6
                  && Math.abs(gauss.var_dollars - localGauss.var_dollars) < 1e-6;
    const fatTailFlag = hist.var_dollars > gauss.var_dollars * 1.05;
    document.getElementById('var-summary').innerHTML = [
        card(t('view.var_estimator.card.historical_var'),     fmtUSD(hist.var_dollars) + (pending ? t('common.suffix.local') : ''), 'neg'),
        card(t('view.var_estimator.card.parametric_var'),     fmtUSD(gauss.var_dollars), 'neg'),
        card(t('view.var_estimator.card.historical_es'),      fmtUSD(hist.expected_shortfall_dollars), 'neg'),
        card(t('view.var_estimator.card.parametric_es'),      fmtUSD(gauss.expected_shortfall_dollars), 'neg'),
        card(t('view.var_estimator.card.confidence'),         fmtPct(state.confidence, 1)),
        card(t('view.var_estimator.card.position_value'),     fmtUSD(state.positionValue)),
        card(t('view.var_estimator.card.sample_size'),        String(stats.n)),
        card(t('view.var_estimator.card.mean_return'),        fmtPct(stats.mean, 3),
            stats.mean >= 0 ? 'pos' : 'neg'),
        card(t('view.var_estimator.card.stdev_vol'),        fmtPct(stats.stdev, 3)),
        card(t('view.var_estimator.card.skewness'),           fmtN(stats.skewness, 3),
            stats.skewness < -0.5 ? 'neg' : ''),
        card(t('view.var_estimator.card.excess_kurtosis'),    fmtN(stats.kurtosis, 3),
            stats.kurtosis > 1 ? 'neg' : ''),
        card(t('view.var_estimator.card.worst_day'),          fmtPct(stats.min, 3), 'neg'),
        card(t('view.var_estimator.card.worst_day_multiple'), fmtN(stats.fattest_left_tail, 2) + 'σ',
            stats.fattest_left_tail < -3 ? 'neg' : ''),
        card(t('view.var_estimator.card.fat_tail_vs_gaussian'), fatTailFlag ? 'FLAGGED' : 'within tolerance',
            fatTailFlag ? 'neg' : 'pos'),
        card(t('view.var_estimator.card.local_parity'),       parityOk ? t('common.ok') : t('common.diverged'),
            parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderCompare(hist, gauss) {
    const cmp = compareMethods(hist, gauss);
    const histESminusVar  = hist.expected_shortfall_dollars  - hist.var_dollars;
    const gaussESminusVar = gauss.expected_shortfall_dollars - gauss.var_dollars;
    document.getElementById('var-compare').innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.var_estimator.th.method">Method</th><th data-i18n="view.var_estimator.th.var">VaR</th><th data-i18n="view.var_estimator.th.expected_shortfall">Expected Shortfall</th>
                <th data-i18n="view.var_estimator.th.es_var_tail_severity">ES − VaR (tail severity)</th><th>n</th>
            </tr></thead>
            <tbody>
                <tr>
                    <td><strong data-i18n="view.var_estimator.row.historical">Historical</strong></td>
                    <td class="neg">${esc(fmtUSD(hist.var_dollars))}</td>
                    <td class="neg">${esc(fmtUSD(hist.expected_shortfall_dollars))}</td>
                    <td>${esc(fmtUSD(histESminusVar))}</td>
                    <td>${hist.n}</td>
                </tr>
                <tr>
                    <td><strong data-i18n="view.var_estimator.row.parametric">Parametric (Gaussian)</strong></td>
                    <td class="neg">${esc(fmtUSD(gauss.var_dollars))}</td>
                    <td class="neg">${esc(fmtUSD(gauss.expected_shortfall_dollars))}</td>
                    <td>${esc(fmtUSD(gaussESminusVar))}</td>
                    <td>${gauss.n}</td>
                </tr>
                <tr>
                    <td><strong data-i18n="view.var_estimator.row.hist_minus_gauss">Hist − Gauss</strong></td>
                    <td class="${cmp.diff >= 0 ? 'neg' : 'pos'}">${esc(fmtUSDSigned(cmp.diff))}</td>
                    <td>${esc(fmtUSDSigned(hist.expected_shortfall_dollars - gauss.expected_shortfall_dollars))}</td>
                    <td colspan="2" class="muted" data-i18n="view.var_estimator.hint.fat_tail">Positive Hist − Gauss = empirical tail is fatter than Gaussian assumes.</td>
                </tr>
            </tbody>
        </table>
    `;
}

function renderRollingVolChart(returns) {
    const el = document.getElementById('var-rollvol-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const arr = (returns || []).map(Number).filter(Number.isFinite);
    const W = 20;
    if (arr.length < W + 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.var_estimator.empty_rollvol_chart">${esc(t('view.var_estimator.empty_rollvol_chart'))}</div>`;
        return;
    }
    const mu = arr.reduce((a, b) => a + b, 0) / arr.length;
    const sigma = Math.sqrt(arr.reduce((s, v) => s + (v - mu) * (v - mu), 0) / Math.max(arr.length - 1, 1));
    const rollVol = new Array(arr.length).fill(null);
    let sum = 0, sumSq = 0;
    for (let i = 0; i < W; i++) { sum += arr[i]; sumSq += arr[i] * arr[i]; }
    rollVol[W - 1] = Math.sqrt(Math.max(0, (sumSq - sum * sum / W) / Math.max(W - 1, 1)));
    for (let i = W; i < arr.length; i++) {
        sum += arr[i] - arr[i - W];
        sumSq += arr[i] * arr[i] - arr[i - W] * arr[i - W];
        rollVol[i] = Math.sqrt(Math.max(0, (sumSq - sum * sum / W) / Math.max(W - 1, 1)));
    }
    const xs = arr.map((_, i) => i + 1);
    const meanRef = xs.map(() => sigma);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.var_estimator.chart.day_idx') },
            { label: t('view.var_estimator.chart.roll_vol'),
              stroke: '#b86bff', width: 1.6, points: { show: false } },
            { label: t('view.var_estimator.chart.mean_vol'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 56,
              values: (_u, splits) => splits.map(v => (v * 100).toFixed(2) + '%') },
        ],
        legend: { show: true },
    }, [xs, rollVol, meanRef], el);
}

function renderHistogram(returns, positionValue, hist, gauss) {
    if (!window.uPlot) return;
    const el = document.getElementById('var-hist');
    if (!el) return;
    el.innerHTML = '';
    const histo = lossHistogram(returns, positionValue, 30);
    if (!histo.edges.length) {
        el.innerHTML = `<div class="muted" data-i18n="view.var_estimator.empty.histogram">No data to histogram.</div>`;
        return;
    }
    const xs = histo.edges.slice(0, -1).map((e, i) => (e + histo.edges[i + 1]) / 2);
    const counts = histo.counts;
    // Reference lines at each VaR / ES level.
    const varH = new Array(xs.length).fill(null);
    const esH  = new Array(xs.length).fill(null);
    const varG = new Array(xs.length).fill(null);
    const targets = [
        { val: hist.var_dollars,                    arr: varH },
        { val: hist.expected_shortfall_dollars,     arr: esH  },
        { val: gauss.var_dollars,                   arr: varG },
    ];
    for (const t of targets) {
        if (!Number.isFinite(t.val)) continue;
        let bestI = -1, bestD = Infinity;
        for (let i = 0; i < xs.length; i++) {
            const d = Math.abs(xs[i] - t.val);
            if (d < bestD) { bestD = d; bestI = i; }
        }
        if (bestI >= 0) t.arr[bestI] = Math.max(...counts);
    }
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 320,
        scales: { x: {}, y: {} },
        series: [
            { label: t('chart.series.loss_') },
            { label: t('chart.series.count'),     stroke: '#888', width: 1.5,
              fill: '#88888833', points: { show: false } },
            { label: t('chart.series.hist_var'),  stroke: '#ffd84a', width: 0,
              points: { show: true, size: 12, fill: '#ffd84a', stroke: '#ffd84a' } },
            { label: t('chart.series.hist_es'),   stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('chart.series.gauss_var'), stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 32,
              values: (_u, splits) => splits.map(v => fmtUSD(v, 0)) },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, counts, varH, esH, varG], el);
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('var-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('var-err').style.display = 'none'; }
