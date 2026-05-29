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

let state = {
    returns: makeDemoReturns('normal'),
    positionValue: 100_000,
    confidence: 0.95,
};

export async function renderVarEstimator(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// VALUE AT RISK</h1>

        <div class="chart-panel">
            <h2>Daily returns <small class="muted">(decimal or with % suffix; csv/space/newline mix)</small></h2>
            <textarea id="var-blob" rows="6" placeholder="0.005&#10;-0.012&#10;0.003&#10;-0.5%&#10;...">${esc(returnsToBlob(state.returns))}</textarea>
            <div class="inline-form">
                <label>Position value ($)
                    <input id="var-pv" type="number" step="any" min="0" value="${state.positionValue}"></label>
                <label>Confidence <small class="muted">(0.95 / 0.99 / 0.999)</small>
                    <select id="var-conf">
                        <option value="0.90"  ${state.confidence === 0.90  ? 'selected' : ''}>90%</option>
                        <option value="0.95"  ${state.confidence === 0.95  ? 'selected' : ''}>95%</option>
                        <option value="0.99"  ${state.confidence === 0.99  ? 'selected' : ''}>99%</option>
                        <option value="0.999" ${state.confidence === 0.999 ? 'selected' : ''}>99.9%</option>
                    </select></label>
                <button id="var-run" class="primary" type="button">Compute</button>
            </div>
            <div class="inline-form">
                <button id="var-demo-normal"    class="secondary" type="button">Demo: normal returns</button>
                <button id="var-demo-fat"       class="secondary" type="button">Demo: fat-tailed</button>
                <button id="var-demo-crisis"    class="secondary" type="button">Demo: 30-day crisis embedded</button>
                <button id="var-demo-low-vol"   class="secondary" type="button">Demo: low-vol drift</button>
                <button id="var-demo-walk"      class="secondary" type="button">Demo: random walk</button>
            </div>
            <p class="muted">Historical = empirical percentile of loss distribution. Parametric assumes Gaussian (μ ± z·σ). When historical &gt; parametric, fat tails are punishing your model. Expected Shortfall (CVaR) = mean loss BEYOND VaR — always ≥ VaR.</p>
        </div>

        <div id="var-summary" class="cards"></div>

        <div class="chart-panel">
            <h2>Method comparison</h2>
            <div id="var-compare"></div>
        </div>

        <div class="chart-panel">
            <h2>Loss distribution (positive numbers = $ losses)</h2>
            <div id="var-hist" style="height:340px"></div>
            <p class="muted">Histogram of daily losses. Yellow dashed = historical VaR, red dashed = historical ES, cyan dashed = parametric VaR.</p>
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
        showErr(`Parse errors: ${parsed.errors.slice(0, 3).map(e => `[${e.line}] ${e.message}`).join('; ')}`);
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
    if (err) { showErr(err); return; }
    const localHist  = localHistorical(state.returns, state.positionValue, state.confidence);
    const localGauss = localParametricGaussian(state.returns, state.positionValue, state.confidence);
    renderSummary(localHist, localGauss, true);
    renderCompare(localHist, localGauss);
    renderHistogram(state.returns, state.positionValue, localHist, localGauss);
    let hist, gauss;
    try {
        [hist, gauss] = await Promise.all([
            api.calcVarHistorical(buildBody(state.returns, state.positionValue, state.confidence)),
            api.calcVarGaussian(buildBody(state.returns, state.positionValue, state.confidence)),
        ]);
    } catch (e) {
        showErr(`API error: ${e.message || e}`); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(hist, gauss, false);
    renderCompare(hist, gauss);
    renderHistogram(state.returns, state.positionValue, hist, gauss);
}

function renderSummary(hist, gauss, pending) {
    const stats = distributionStats(state.returns);
    const localHist  = localHistorical(state.returns, state.positionValue, state.confidence);
    const localGauss = localParametricGaussian(state.returns, state.positionValue, state.confidence);
    const parityOk = Math.abs(hist.var_dollars  - localHist.var_dollars)  < 1e-6
                  && Math.abs(gauss.var_dollars - localGauss.var_dollars) < 1e-6;
    const fatTailFlag = hist.var_dollars > gauss.var_dollars * 1.05;
    document.getElementById('var-summary').innerHTML = [
        card('Historical VaR',     fmtUSD(hist.var_dollars) + (pending ? ' (local)' : ''), 'neg'),
        card('Parametric VaR',     fmtUSD(gauss.var_dollars), 'neg'),
        card('Historical ES',      fmtUSD(hist.expected_shortfall_dollars), 'neg'),
        card('Parametric ES',      fmtUSD(gauss.expected_shortfall_dollars), 'neg'),
        card('Confidence',         fmtPct(state.confidence, 1)),
        card('Position value',     fmtUSD(state.positionValue)),
        card('Sample size',        String(stats.n)),
        card('Mean return',        fmtPct(stats.mean, 3),
            stats.mean >= 0 ? 'pos' : 'neg'),
        card('Stdev (vol)',        fmtPct(stats.stdev, 3)),
        card('Skewness',           fmtN(stats.skewness, 3),
            stats.skewness < -0.5 ? 'neg' : ''),
        card('Excess kurtosis',    fmtN(stats.kurtosis, 3),
            stats.kurtosis > 1 ? 'neg' : ''),
        card('Worst day',          fmtPct(stats.min, 3), 'neg'),
        card('Worst-day σ-multiple', fmtN(stats.fattest_left_tail, 2) + 'σ',
            stats.fattest_left_tail < -3 ? 'neg' : ''),
        card('Fat-tail vs Gaussian', fatTailFlag ? 'FLAGGED' : 'within tolerance',
            fatTailFlag ? 'neg' : 'pos'),
        card('Local parity',       parityOk ? 'OK' : 'DIVERGED',
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
                <th>Method</th><th>VaR</th><th>Expected Shortfall</th>
                <th>ES − VaR (tail severity)</th><th>n</th>
            </tr></thead>
            <tbody>
                <tr>
                    <td><strong>Historical</strong></td>
                    <td class="neg">${esc(fmtUSD(hist.var_dollars))}</td>
                    <td class="neg">${esc(fmtUSD(hist.expected_shortfall_dollars))}</td>
                    <td>${esc(fmtUSD(histESminusVar))}</td>
                    <td>${hist.n}</td>
                </tr>
                <tr>
                    <td><strong>Parametric (Gaussian)</strong></td>
                    <td class="neg">${esc(fmtUSD(gauss.var_dollars))}</td>
                    <td class="neg">${esc(fmtUSD(gauss.expected_shortfall_dollars))}</td>
                    <td>${esc(fmtUSD(gaussESminusVar))}</td>
                    <td>${gauss.n}</td>
                </tr>
                <tr>
                    <td><strong>Hist − Gauss</strong></td>
                    <td class="${cmp.diff >= 0 ? 'neg' : 'pos'}">${esc(fmtUSDSigned(cmp.diff))}</td>
                    <td>${esc(fmtUSDSigned(hist.expected_shortfall_dollars - gauss.expected_shortfall_dollars))}</td>
                    <td colspan="2" class="muted">Positive Hist − Gauss = empirical tail is fatter than Gaussian assumes.</td>
                </tr>
            </tbody>
        </table>
    `;
}

function renderHistogram(returns, positionValue, hist, gauss) {
    if (!window.uPlot) return;
    const el = document.getElementById('var-hist');
    if (!el) return;
    el.innerHTML = '';
    const histo = lossHistogram(returns, positionValue, 30);
    if (!histo.edges.length) {
        el.innerHTML = '<div class="muted">No data to histogram.</div>';
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
            { label: 'loss $' },
            { label: 'count',     stroke: '#888', width: 1.5,
              fill: '#88888833', points: { show: false } },
            { label: 'Hist VaR',  stroke: '#ffd84a', width: 0,
              points: { show: true, size: 12, fill: '#ffd84a', stroke: '#ffd84a' } },
            { label: 'Hist ES',   stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, fill: '#ff3860', stroke: '#ff3860' } },
            { label: 'Gauss VaR', stroke: '#00e5ff', width: 0,
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
