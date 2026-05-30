// Monte Carlo trade-sequence simulator view. Draws N synthetic equity
// curves from a historical R-multiple distribution; reports ending-
// equity percentiles + max-drawdown percentiles + probability of ruin.
//
// Distinct from views/monte_carlo.js — that one is a stochastic-process
// path simulator (fBm / GBM / jump diffusion). This one bootstraps from
// an empirical R distribution.
//
// All user-facing strings flow through t() / data-i18n keys.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    DEFAULT_CONFIG, parseRBlob, validateInputs, buildBody,
    localSimulateWithCurves, endingHistogram, makeDemoR,
    ruinBadge, fmtUSD, fmtPct,
} from '../_monte_carlo_inputs.js';

let state = {
    historical_r: makeDemoR('positive-edge'),
    cfg: { ...DEFAULT_CONFIG },
    lastEnding: null,
};

export async function renderMcTrades(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.mc_trades.h1.title" class="view-title">// MC TRADES (BOOTSTRAP)</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.mc_trades.h2.historical_r">Historical R-multiples
                <small data-i18n="view.mc_trades.h2.historical_r_hint" class="muted">(one per token, csv/space/newline)</small></h2>
            <textarea id="mct-blob" rows="5" placeholder="1.0 -1.0 0.5 -0.5 ..." data-tip="view.mc_trades.tip.blob">${esc(rToBlob(state.historical_r))}</textarea>
            <div class="inline-form">
                <label><span data-i18n="view.mc_trades.label.n_curves">N curves</span>
                    <input id="mct-n" type="number" step="1" min="1" max="50000" value="${state.cfg.n_curves}" data-tip="view.mc_trades.tip.n_curves"></label>
                <label><span data-i18n="view.mc_trades.label.trades_per_curve">Trades / curve</span>
                    <input id="mct-tpc" type="number" step="1" min="1" max="10000" value="${state.cfg.trades_per_curve}" data-tip="view.mc_trades.tip.trades_per_curve"></label>
                <label><span data-i18n="view.mc_trades.label.start_equity">Start equity ($)</span>
                    <input id="mct-eq" type="number" step="any" min="0" value="${state.cfg.start_equity}" data-tip="view.mc_trades.tip.start_equity"></label>
                <label><span data-i18n="view.mc_trades.label.ruin_threshold">Ruin threshold ($)</span>
                    <input id="mct-ruin" type="number" step="any" min="0" value="${state.cfg.ruin_threshold}" data-tip="view.mc_trades.tip.ruin_threshold"></label>
                <label><span data-i18n="view.mc_trades.label.seed">RNG seed</span>
                    <input id="mct-seed" type="number" step="1" min="0" value="${state.cfg.seed}" data-tip="view.mc_trades.tip.seed"></label>
                <button data-i18n="view.mc_trades.btn.simulate" id="mct-run" class="primary" type="button" data-tip="view.mc_trades.tip.run" data-shortcut="mc_trades_run">Simulate</button>
            </div>
            <div class="inline-form">
                <button data-i18n="view.mc_trades.btn.demo_positive" id="mct-demo-pos"   class="secondary" type="button" data-tip="view.mc_trades.tip.demo_pos">Demo: positive edge</button>
                <button data-i18n="view.mc_trades.btn.demo_negative" id="mct-demo-neg"   class="secondary" type="button" data-tip="view.mc_trades.tip.demo_neg">Demo: negative edge</button>
                <button data-i18n="view.mc_trades.btn.demo_fat_tail" id="mct-demo-fat"   class="secondary" type="button" data-tip="view.mc_trades.tip.demo_fat">Demo: fat-tail</button>
                <button data-i18n="view.mc_trades.btn.demo_lumpy"    id="mct-demo-lumpy" class="secondary" type="button" data-tip="view.mc_trades.tip.demo_lumpy">Demo: lumpy winners</button>
                <button data-i18n="view.mc_trades.btn.demo_random"   id="mct-demo-rand"  class="secondary" type="button" data-tip="view.mc_trades.tip.demo_rand">Demo: random walk</button>
            </div>
            <p data-i18n="view.mc_trades.hint.about" class="muted">Draws N independent synthetic equity curves of length L by sampling R-multiples with replacement. Surfaces percentile + ruin-probability distribution so you can see realistic worst cases, not just expectancy. Deterministic given seed.</p>
        </div>

        <div id="mct-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.mc_trades.h2.ending_dist">Ending-equity distribution</h2>
            <div id="mct-chart" style="height:340px"></div>
            <p data-i18n="view.mc_trades.hint.histogram" class="muted">Histogram of ending equity across all curves. Cyan dot = start equity. Red = 5th percentile (downside). Yellow = median. Green = 95th percentile.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.mc_trades.h2.cdf_chart">Ending-equity CDF (cumulative % below value)</h2>
            <div id="mct-cdf-chart" style="width:100%;height:240px"></div>
            <p data-i18n="view.mc_trades.hint.cdf" class="muted small">Cumulative share of curves with ending equity ≤ x. Cyan dashed = start equity (read off "% of paths losing money"). Easier than the histogram for tail-probability questions.</p>
        </div>

        <div id="mct-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    const loadDemo = (kind) => {
        state.historical_r = makeDemoR(kind);
        document.getElementById('mct-blob').value = rToBlob(state.historical_r);
    };
    document.getElementById('mct-demo-pos').addEventListener('click',   () => loadDemo('positive-edge'));
    document.getElementById('mct-demo-neg').addEventListener('click',   () => loadDemo('negative-edge'));
    document.getElementById('mct-demo-fat').addEventListener('click',   () => loadDemo('fat-tail'));
    document.getElementById('mct-demo-lumpy').addEventListener('click', () => loadDemo('lumpy-winner'));
    document.getElementById('mct-demo-rand').addEventListener('click',  () => loadDemo('random'));
    document.getElementById('mct-run').addEventListener('click', () => { readInputs(); void compute(tok); });
    readInputs(); void compute(tok);
}

function rToBlob(r) { return r.map(v => v.toString()).join(' '); }

function readInputs() {
    const parsed = parseRBlob(document.getElementById('mct-blob').value);
    if (parsed.errors.length) {
        showErr(`${t('view.mc_trades.err.parse_prefix')}: `
            + parsed.errors.slice(0, 3).map(e => `[${e.line}] ${e.message}`).join('; '));
        showToast(t('view.mc_trades.toast.parse_error', { n: parsed.errors.length }), { level: 'warning' });
        return;
    }
    hideErr();
    state.historical_r = parsed.r;
    state.cfg = {
        n_curves:         parseInt(document.getElementById('mct-n').value, 10),
        trades_per_curve: parseInt(document.getElementById('mct-tpc').value, 10),
        start_equity:     Number(document.getElementById('mct-eq').value),
        ruin_threshold:   Number(document.getElementById('mct-ruin').value),
        seed:             parseInt(document.getElementById('mct-seed').value, 10),
    };
}

async function compute(tok) {
    hideErr();
    const err = validateInputs(state.historical_r, state.cfg);
    if (err) { showErr(err); showToast(t('view.mc_trades.toast.invalid'), { level: 'warning' }); return; }
    const local = localSimulateWithCurves(state.historical_r, state.cfg);
    if (!local.report) {
        showErr(t('view.mc_trades.err.invalid'));
        showToast(t('view.mc_trades.toast.invalid'), { level: 'warning' });
        return;
    }
    state.lastEnding = local.ending;
    renderSummary(local.report, true);
    renderChart(local.ending, local.report);
    renderCdfChart(local.ending, local.report);
    let resp;
    try {
        resp = await api.calcMonteCarlo(buildBody(state.historical_r, state.cfg));
    } catch (e) {
        showErr(`${t('view.mc_trades.err.api')}: ${e.message || e}`);
        showToast(t('view.mc_trades.toast.api_error'), { level: 'error' });
        return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(resp, false);
    renderChart(state.lastEnding, resp);
    renderCdfChart(state.lastEnding, resp);
    const pRuin = (Number(resp.probability_of_ruin) * 100).toFixed(2);
    const pProf = (Number(resp.probability_profitable) * 100).toFixed(1);
    const level = resp.probability_of_ruin > 0.02 ? 'warning' : 'success';
    showToast(t('view.mc_trades.toast.simulated', { n: resp.n_curves, ruin: pRuin, profit: pProf }), { level });
}

function renderSummary(report, pending) {
    const badge = ruinBadge(report.probability_of_ruin);
    const local = localSimulateWithCurves(state.historical_r, state.cfg);
    const localR = local.report;
    const parity = Math.abs(report.probability_of_ruin - (localR ? localR.probability_of_ruin : 0)) < 1e-9
                && Math.abs(report.mean_ending_equity - (localR ? localR.mean_ending_equity : 0)) < 1e-6;
    const localTag = pending ? ` (${t('view.mc_trades.tag.local')})` : '';
    document.getElementById('mct-summary').innerHTML = [
        card(t('view.mc_trades.card.ruin'),         t(badge.key) + localTag, badge.cls || 'pos'),
        card(t('view.mc_trades.card.p_ruin'),       fmtPct(report.probability_of_ruin),
             report.probability_of_ruin > 0.02 ? 'neg' : 'pos'),
        card(t('view.mc_trades.card.p_profit'),     fmtPct(report.probability_profitable),
             report.probability_profitable >= 0.5 ? 'pos' : 'neg'),
        card(t('view.mc_trades.card.n_curves'),     String(report.n_curves)),
        card(t('view.mc_trades.card.trades'),       String(report.trades_per_curve)),
        card(t('view.mc_trades.card.start_equity'), fmtUSD(report.start_equity)),
        card(t('view.mc_trades.card.mean_ending'),  fmtUSD(report.mean_ending_equity),
             report.mean_ending_equity > report.start_equity ? 'pos' : 'neg'),
        card(t('view.mc_trades.card.ending_p05'),   fmtUSD(report.ending_equity_p05), 'neg'),
        card(t('view.mc_trades.card.ending_p25'),   fmtUSD(report.ending_equity_p25)),
        card(t('view.mc_trades.card.ending_p50'),   fmtUSD(report.ending_equity_p50),
             report.ending_equity_p50 > report.start_equity ? 'pos' : 'neg'),
        card(t('view.mc_trades.card.ending_p75'),   fmtUSD(report.ending_equity_p75)),
        card(t('view.mc_trades.card.ending_p95'),   fmtUSD(report.ending_equity_p95), 'pos'),
        card(t('view.mc_trades.card.dd_mean'),      fmtPct(report.mean_max_drawdown), 'neg'),
        card(t('view.mc_trades.card.dd_p05'),       fmtPct(report.max_drawdown_p05)),
        card(t('view.mc_trades.card.dd_p50'),       fmtPct(report.max_drawdown_p50), 'neg'),
        card(t('view.mc_trades.card.dd_p95'),       fmtPct(report.max_drawdown_p95), 'neg'),
        card(t('view.mc_trades.card.parity'),       parity ? t('view.mc_trades.tag.ok') : t('view.mc_trades.tag.diverged'),
             parity ? 'pos' : 'neg'),
    ].join('');
}

function renderChart(ending, report) {
    if (!window.uPlot || !ending || ending.length === 0) return;
    const el = document.getElementById('mct-chart');
    if (!el) return;
    el.innerHTML = '';
    const h = endingHistogram(ending, 30);
    if (!h.centers.length) return;
    const counts = h.counts;
    const maxC = Math.max(...counts);
    const lines = [
        { val: report.start_equity,       color: '#00e5ff' },
        { val: report.ending_equity_p05,  color: '#ff3860' },
        { val: report.ending_equity_p50,  color: '#ffd84a' },
        { val: report.ending_equity_p95,  color: '#23d18b' },
    ];
    const dashSeries = lines.map(L => {
        const arr = new Array(h.centers.length).fill(null);
        let bestI = -1, bestD = Infinity;
        for (let i = 0; i < h.centers.length; i++) {
            const d = Math.abs(h.centers[i] - L.val);
            if (d < bestD) { bestD = d; bestI = i; }
        }
        if (bestI >= 0) arr[bestI] = maxC;
        return arr;
    });
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 320,
        scales: { x: {}, y: {} },
        series: [
            { label: t('chart.series.equity_') },
            { label: t('chart.series.count'), stroke: '#888', width: 1.5,
              fill: '#88888833', points: { show: false } },
            { label: t('chart.series.start'),  stroke: '#00e5ff', width: 0,
              points: { show: true, size: 11, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: 'p05',    stroke: '#ff3860', width: 0,
              points: { show: true, size: 11, fill: '#ff3860', stroke: '#ff3860' } },
            { label: 'p50',    stroke: '#ffd84a', width: 0,
              points: { show: true, size: 11, fill: '#ffd84a', stroke: '#ffd84a' } },
            { label: 'p95',    stroke: '#23d18b', width: 0,
              points: { show: true, size: 11, fill: '#23d18b', stroke: '#23d18b' } },
        ],
        axes: [
            { stroke: '#aab', size: 32,
              values: (_u, splits) => splits.map(v => fmtUSD(v, 0)) },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [h.centers, counts, ...dashSeries], el);
}

function renderCdfChart(ending, report) {
    if (!window.uPlot) return;
    const el = document.getElementById('mct-cdf-chart');
    if (!el) return;
    el.innerHTML = '';
    if (!ending || ending.length === 0) {
        el.innerHTML = `<div class="muted" data-i18n="view.mc_trades.empty_cdf_chart">${esc(t('view.mc_trades.empty_cdf_chart'))}</div>`;
        return;
    }
    const sorted = ending.slice().sort((a, b) => a - b);
    const n = sorted.length;
    const xs = sorted;
    const ys = sorted.map((_, i) => (i + 1) / n * 100);
    const startLine = xs.map(() => null);
    let bestI = 0, bestD = Infinity;
    for (let i = 0; i < xs.length; i++) {
        const d = Math.abs(xs[i] - Number(report.start_equity));
        if (d < bestD) { bestD = d; bestI = i; }
    }
    if (Number.isFinite(Number(report.start_equity))) startLine[bestI] = ys[bestI];
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { range: [0, 100] } },
        series: [
            { label: t('chart.series.equity_') },
            { label: t('view.mc_trades.chart.cdf'),
              stroke: '#b86bff', width: 1.6, points: { show: false } },
            { label: t('view.mc_trades.chart.start_marker'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
        ],
        axes: [
            { stroke: '#aab', size: 32,
              values: (_u, splits) => splits.map(v => fmtUSD(v, 0)) },
            { stroke: '#aab', size: 50,
              values: (_u, splits) => splits.map(v => v.toFixed(0) + '%') },
        ],
        legend: { show: true },
    }, [xs, ys, startLine], el);
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function showErr(msg) {
    const el = document.getElementById('mct-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('mct-err').style.display = 'none'; }
