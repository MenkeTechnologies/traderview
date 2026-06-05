// Beta view — single-asset beta + alpha + R² + correlation vs benchmark.
//
// i18n throughout.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parsePairsBlob, pairsToBlob, validateInputs, buildBody, localEstimate,
    betaBadge, fitBadge, hedgeNotional, annualizeAlpha,
    makeDemoInput,
    fmtBeta, fmtAlpha, fmtR2, fmtPctSigned, fmtUSD, fmtInt,
} from '../_beta_inputs.js';

let state = { ...makeDemoInput('tech-stock'), notional: 100000, periods_per_year: 252 };

export async function renderBeta(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.beta.h1.title" class="view-title">// BETA REGRESSION</h1>

        <div class="chart-panel" data-context-scope="beta">
            <h2 data-i18n="view.beta.h2.pairs">Returns
                <small data-i18n="view.beta.h2.pairs_hint" class="muted">(per line: asset benchmark — decimal 0.012 or "1.2%")</small></h2>
            <textarea id="bt-blob" rows="6"
                      data-tip="view.beta.tip.pairs"
                      placeholder="0.012 0.008&#10;-0.005 -0.004">${esc(pairsToBlob(state.asset, state.benchmark))}</textarea>

            <div class="inline-form">
                <label><span data-i18n="view.beta.label.notional">Hedge notional ($)</span>
                    <input id="bt-notional" type="number" step="0.01" min="0" value="${state.notional}"
                           data-tip="view.beta.tip.notional"></label>
                <label><span data-i18n="view.beta.label.periods">Periods / yr</span>
                    <input id="bt-periods" type="number" step="0.01" min="1" value="${state.periods_per_year}"
                           data-tip="view.beta.tip.periods"></label>
                <button data-i18n="view.beta.btn.compute" id="bt-run" class="primary"
                        data-tip="view.beta.tip.compute" data-shortcut="beta_run" type="button">Estimate β</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.beta.btn.demo_tech"     id="bt-demo-tech"     class="secondary" data-tip="view.beta.tip.demo_tech"    type="button">Demo: tech stock (β≈1.3)</button>
                <button data-i18n="view.beta.btn.demo_utility"  id="bt-demo-util"     class="secondary" data-tip="view.beta.tip.demo_utility" type="button">Demo: utility (β≈0.3)</button>
                <button data-i18n="view.beta.btn.demo_inverse"  id="bt-demo-inv"      class="secondary" data-tip="view.beta.tip.demo_inverse" type="button">Demo: inverse ETF (β≈−1)</button>
                <button data-i18n="view.beta.btn.demo_neutral"  id="bt-demo-neutral"  class="secondary" data-tip="view.beta.tip.demo_neutral" type="button">Demo: market-neutral (β≈0.05)</button>
                <button data-i18n="view.beta.btn.demo_3x"       id="bt-demo-3x"       class="secondary" data-tip="view.beta.tip.demo_3x"      type="button">Demo: 3x leveraged (β≈3)</button>
                <button data-i18n="view.beta.btn.demo_perfect"  id="bt-demo-perfect"  class="secondary" data-tip="view.beta.tip.demo_perfect" type="button">Demo: perfect match (R²=1)</button>
                <button data-i18n="view.beta.btn.demo_nocorr"   id="bt-demo-noco"     class="secondary" data-tip="view.beta.tip.demo_nocorr"  type="button">Demo: no correlation</button>
                <button data-i18n="view.beta.btn.demo_flat"     id="bt-demo-flat"     class="secondary" data-tip="view.beta.tip.demo_flat"    type="button">Demo: flat benchmark (degenerate)</button>
            </div>
            <p data-i18n="view.beta.hint.about" class="muted">β = cov(asset, bench) / var(bench). α = mean(asset) − β·mean(bench). R² = corr². Beta-neutral hedge: short $X·β of benchmark per $X long. α is per-period return after removing market exposure.</p>
        </div>

        <div id="bt-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.beta.h2.chart">Asset vs benchmark scatter + regression line</h2>
            <div id="bt-chart" style="width:100%;height:340px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.beta.h2.resid_chart">Per-period residuals (asset − (α + β·bench))</h2>
            <div id="bt-resid-chart" style="width:100%;height:220px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.beta.h2.table">Paired returns (tail — last 30)</h2>
            <div id="bt-table"></div>
        </div>

        <div id="bt-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (k) => {
        const inp = makeDemoInput(k);
        state.asset = inp.asset;
        state.benchmark = inp.benchmark;
        document.getElementById('bt-blob').value = pairsToBlob(state.asset, state.benchmark);
    };
    document.getElementById('bt-demo-tech').addEventListener('click',    () => { loadDemo('tech-stock');        void compute(tok); });
    document.getElementById('bt-demo-util').addEventListener('click',    () => { loadDemo('utility-low-beta'); void compute(tok); });
    document.getElementById('bt-demo-inv').addEventListener('click',     () => { loadDemo('inverse-etf');       void compute(tok); });
    document.getElementById('bt-demo-neutral').addEventListener('click', () => { loadDemo('market-neutral');    void compute(tok); });
    document.getElementById('bt-demo-3x').addEventListener('click',      () => { loadDemo('high-beta-3x');      void compute(tok); });
    document.getElementById('bt-demo-perfect').addEventListener('click', () => { loadDemo('perfect-match');     void compute(tok); });
    document.getElementById('bt-demo-noco').addEventListener('click',    () => { loadDemo('no-correlation');    void compute(tok); });
    document.getElementById('bt-demo-flat').addEventListener('click',    () => { loadDemo('flat-bench');        void compute(tok); });
    document.getElementById('bt-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    void compute(tok);
}

function readInputs() {
    const p = parsePairsBlob(document.getElementById('bt-blob').value);
    if (p.errors.length) {
        showErr(`${t('view.beta.err.parse_prefix')}: `
            + p.errors.slice(0, 3).map(e => `[${e.line_no}] ${e.message}`).join('; '));
        showToast(t('view.beta.toast.parse_error'), { level: 'error' });
        return;
    }
    hideErr();
    state.asset = p.asset;
    state.benchmark = p.benchmark;
    const notional = Number(document.getElementById('bt-notional').value);
    const periods = Number(document.getElementById('bt-periods').value);
    state.notional = Number.isFinite(notional) && notional >= 0 ? notional : 100000;
    state.periods_per_year = Number.isFinite(periods) && periods > 0 ? periods : 252;
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state);
    if (err) { showErr(err); showToast(t('view.beta.toast.invalid'), { level: 'warning' }); return; }
    const local = localEstimate(state.asset, state.benchmark);
    if (!local) { showErr(t('view.beta.err.degenerate')); showToast(t('view.beta.toast.degenerate'), { level: 'warning' }); return; }
    renderSummary(local, true);
    renderChart(local);
    renderResidChart(local);
    renderTable();
    let resp;
    try {
        resp = await api.anlyBeta(buildBody(state));
    } catch (e) {
        showErr(`${t('view.beta.err.api')}: ${e.message || e}`);
        showToast(t('view.beta.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    if (!resp) { showErr(t('view.beta.err.server_rejected')); showToast(t('view.beta.toast.server_rejected'), { level: 'error' }); return; }
    renderSummary(resp, false);
    renderChart(resp);
    renderResidChart(resp);
    renderTable();
    showToast(t('view.beta.toast.estimated'), { level: 'success' });
}

function renderSummary(report, pending) {
    const local = localEstimate(state.asset, state.benchmark);
    const parityOk = !!local
        && Math.abs(local.beta - report.beta) < 1e-9
        && Math.abs(local.alpha - report.alpha) < 1e-9
        && Math.abs(local.r_squared - report.r_squared) < 1e-9;
    const bBadge = betaBadge(report.beta);
    const fBadge = fitBadge(report.r_squared);
    const hedge = hedgeNotional(state.notional, report.beta);
    const annAlpha = annualizeAlpha(report.alpha, state.periods_per_year);
    const localTag = pending ? ` (${t('view.beta.tag.local')})` : '';
    document.getElementById('bt-summary').innerHTML = [
        card(t('view.beta.card.verdict'),     t(bBadge.key) + localTag, bBadge.cls),
        card(t('view.beta.card.fit'),         t(fBadge.key), fBadge.cls),
        card(t('view.beta.card.beta'),        fmtBeta(report.beta), bBadge.cls),
        card(t('view.beta.card.alpha'),       fmtAlpha(report.alpha)),
        card(t('view.beta.card.alpha_ann'),   fmtPctSigned(annAlpha),
             annAlpha > 0 ? 'pos' : annAlpha < 0 ? 'neg' : ''),
        card(t('view.beta.card.r2'),          fmtR2(report.r_squared), fBadge.cls),
        card(t('view.beta.card.correlation'), fmtBeta(report.correlation)),
        card(t('view.beta.card.n'),           fmtInt(report.n)),
        card(t('view.beta.card.hedge'),       fmtUSD(hedge),
             Math.abs(hedge) > Math.abs(state.notional) ? 'neg' : ''),
        card(t('view.beta.card.parity'),
             parityOk ? t('view.beta.tag.ok') : t('view.beta.tag.diverged'),
             parityOk ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(report) {
    if (!window.uPlot) return;
    const el = document.getElementById('bt-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!state.benchmark.length) {
        el.innerHTML = `<div class="muted" data-i18n="view.beta.empty">${esc(t('view.beta.empty'))}</div>`;
        return;
    }
    // X = benchmark return %, Y = asset return %. Regression line: y = α + β·x.
    const bench = state.benchmark.map(v => v * 100);
    const asset = state.asset.map(v => v * 100);
    let mnB = Infinity, mxB = -Infinity;
    for (const v of bench) { if (v < mnB) mnB = v; if (v > mxB) mxB = v; }
    // Plot regression line as a separate aligned series on the same x grid.
    // Build a sorted-by-x sequence: scatter points + a paired (xs, fitted) line.
    const idx = bench.map((_, i) => i).sort((a, b) => bench[a] - bench[b]);
    const xsSorted = idx.map(i => bench[i]);
    const fitSorted = idx.map(i => (report.alpha + report.beta * (bench[i] / 100)) * 100);
    const assetSorted = idx.map(i => asset[i]);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 340,
        scales: { x: {}, y: {} },
        series: [
            { label: t('view.beta.series.bench_pct') },
            { label: t('view.beta.series.asset_pct'), stroke: '#00e5ff', width: 0, points: { show: true, size: 4 } },
            { label: t('view.beta.series.fit', { alpha: report.alpha.toFixed(5), beta: report.beta.toFixed(3) }),
              stroke: '#ff3860', width: 1.5, points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => v.toFixed(2) + '%') },
            { stroke: '#aab', size: 60,
              values: (_u, splits) => splits.map(v => v.toFixed(2) + '%') },
        ],
        legend: { show: true },
    }, [xsSorted, assetSorted, fitSorted], el);
    void mnB; void mxB;
}

function renderResidChart(report) {
    if (!window.uPlot) return;
    const el = document.getElementById('bt-resid-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!state.benchmark.length) {
        el.innerHTML = `<div class="muted" data-i18n="view.beta.empty_resid">${esc(t('view.beta.empty_resid'))}</div>`;
        return;
    }
    const xs = state.benchmark.map((_, i) => i + 1);
    const resid = state.asset.map((a, i) => {
        const b = state.benchmark[i];
        if (!Number.isFinite(a) || !Number.isFinite(b)) return null;
        return (a - (report.alpha + report.beta * b)) * 100;
    });
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('chart.series.i') },
            { label: t('view.beta.series.resid_pct'),
              stroke: '#7af0a8', width: 1.2,
              points: { show: true, size: 4, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.beta.series.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28 },
            { stroke: '#aab', size: 60,
              values: (_u, splits) => splits.map(v => v.toFixed(2) + '%') },
        ],
        legend: { show: true },
    }, [xs, resid, zero], el);
}

function renderTable() {
    const wrap = document.getElementById('bt-table');
    const n = state.asset.length;
    if (n === 0) {
        wrap.innerHTML = `<div class="muted" data-i18n="view.beta.empty">${esc(t('view.beta.empty'))}</div>`;
        return;
    }
    const start = Math.max(0, n - 30);
    wrap.innerHTML = `
        <table class="lq-table">
            <thead><tr>
                <th data-i18n="view.beta.col.idx">#</th>
                <th data-i18n="view.beta.col.asset">Asset</th>
                <th data-i18n="view.beta.col.bench">Benchmark</th>
                <th data-i18n="view.beta.col.diff">Δ</th>
            </tr></thead>
            <tbody>
                ${Array.from({ length: n - start }, (_, k) => {
                    const i = start + k;
                    const a = state.asset[i];
                    const b = state.benchmark[i];
                    const d = a - b;
                    const acls = a > 0 ? 'pos' : a < 0 ? 'neg' : '';
                    const bcls = b > 0 ? 'pos' : b < 0 ? 'neg' : '';
                    const dcls = d > 0 ? 'pos' : d < 0 ? 'neg' : '';
                    return `<tr>
                        <td>${i + 1}</td>
                        <td class="${acls}">${esc(fmtPctSigned(a, 4))}</td>
                        <td class="${bcls}">${esc(fmtPctSigned(b, 4))}</td>
                        <td class="${dcls}">${esc(fmtPctSigned(d, 4))}</td>
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
    const el = document.getElementById('bt-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('bt-err').style.display = 'none'; }
