// Market Impact view — participation-rate slippage analysis.
//
// "At what % of ADV does my slippage cliff?" Bucketed by participation
// rate, returns avg / median / max slippage per band + flags the first
// band where avg slippage exceeds the spike threshold.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseTradeBlob, validateInputs, buildBody,
    bucketParticipations, makeDemoTrades,
    BUCKET_LABELS, fmtBps, fmtN,
} from '../_market_impact_inputs.js';

import { t } from '../i18n.js';
import { showToast } from '../toast.js';
let state = { trades: '', spikeBps: 30 };

export async function renderMarketImpact(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.market_impact.h1.market_impact_participation_cliff" class="view-title">// MARKET IMPACT · PARTICIPATION CLIFF</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.market_impact.h2.trade_ledger">Trade ledger</h2>
            <p class="muted" data-i18n-html="view.market_impact.help">Paste <code>qty adv slippage_bps</code> per line. Negative
                slippage is a favorable fill. Demo loads 400 trades that visibly cliff
                past 1% ADV.</p>
            <textarea id="mi-trades" rows="8" placeholder="2500 5000000 2.1&#10;120000 5000000 12.5&#10;..." data-tip="view.market_impact.tip.trades"></textarea>
            <div class="inline-form">
                <button data-i18n="view.market_impact.btn.load_demo_400_trades" data-tip="view.market_impact.tip.demo" data-shortcut="market_impact_demo" id="mi-demo" class="secondary" type="button">Load demo (400 trades)</button>
                <button data-i18n="view.market_impact.btn.clear" data-tip="view.market_impact.tip.clear" id="mi-clear" class="secondary" type="button">Clear</button>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.market_impact.h2.threshold">Threshold</h2>
            <div class="inline-form">
                <label><span data-i18n="view.market_impact.label.spike_bps">Spike threshold (bps)</span>
                    <input id="mi-spike" type="number" step="0.01" min="0" value="${state.spikeBps}" data-tip="view.market_impact.tip.spike"></label>
                <button data-i18n="view.market_impact.btn.analyze" data-tip="view.market_impact.tip.analyze" data-shortcut="market_impact_analyze" id="mi-run" class="primary" type="button">Analyze</button>
            </div>
            <p data-i18n="view.market_impact.hint.the_first_adv_bucket_whose_avg_slippage_clears_the" class="muted">The first ADV bucket whose avg slippage clears the threshold
                is flagged as your "impact cliff" — size below it.</p>
        </div>

        <div id="mi-errors" class="boot" style="display:none"></div>
        <div id="mi-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.market_impact.h2.slippage_by_participation_bucket">Slippage by participation bucket</h2>
            <div id="mi-bars"></div>
            <p data-i18n="view.market_impact.hint.per_bucket_trade_count_avg_median_max_slippage_thr" class="muted">Per bucket: trade count · avg · median · max slippage.
                Threshold-busting buckets glow red.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.market_impact.h2.trade_distribution">Trade distribution</h2>
            <div id="mi-dist-chart" style="height:200px"></div>
            <p data-i18n="view.market_impact.hint.where_your_trades_actually_land_if_you_have_heavy_" class="muted">Where your trades actually land. If you have heavy mass past
                1% ADV you're paying the impact tax even if the average looks fine.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.market_impact.h2.scatter_chart">Per-trade participation % vs slippage (raw cliff)</h2>
            <div id="mi-scatter-chart" style="width:100%;height:260px"></div>
            <p data-i18n="view.market_impact.hint.scatter" class="muted small">Each trade as a point: qty/ADV on x, slippage bps on y. Continuous view of the cliff shape. Red dashed = spike threshold; trades above are paying the impact tax.</p>
        </div>

        <div id="mi-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('mi-demo').addEventListener('click', () => {
        const trades = makeDemoTrades(42);
        document.getElementById('mi-trades').value =
            trades.map(t => `${t.qty} ${t.adv} ${t.slippage_bps}`).join('\n');
    });
    document.getElementById('mi-clear').addEventListener('click', () => {
        document.getElementById('mi-trades').value = '';
    });
    document.getElementById('mi-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.trades = document.getElementById('mi-trades').value;
    state.spikeBps = Number(document.getElementById('mi-spike').value);
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('mi-errors');
    errs.style.display = 'none';
    const { trades, errors } = parseTradeBlob(state.trades);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            t('common.parse_error_inline', { line: e.line_no, msg: e.message, raw: e.raw.slice(0, 80) })).join('<br>');
        const more = errors.length > 8 ? `<br>${esc(t('common.and_n_more', { n: errors.length - 8 }))}` : '';
        errs.innerHTML = `<strong>${esc(t('common.parse_errors_lead', { n: errors.length }))}</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (trades.length < 5) return;
    }
    const err = validateInputs(trades, state.spikeBps);
    if (err) { showErr(err); showToast(err, { level: 'warning' }); return; }
    let res;
    try {
        res = await api.microMarketImpact(buildBody(trades, state.spikeBps));
    } catch (e) {
        const m = t("common.error.api", { msg: e.message || e });
        showErr(m); showToast(m, { level: 'error' }); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(res, trades);
    renderBars(res, state.spikeBps);
    renderDistribution(trades);
    renderScatterChart(trades, state.spikeBps);
    showToast(t('view.market_impact.toast.done', {
        trades: trades.length,
        cliff: res.impact_threshold_label || t('common.none'),
    }), { level: res.impact_threshold_label ? 'warning' : 'success' });
}

function renderSummary(report, trades) {
    const total = trades.length;
    const allSlips = trades.map(t => t.slippage_bps).filter(Number.isFinite);
    const avgAll = allSlips.length ? allSlips.reduce((a, b) => a + b, 0) / allSlips.length : NaN;
    const cliffLabel = report.impact_threshold_label || '—';
    const cliffCls = report.impact_threshold_label ? 'neg' : 'pos';
    document.getElementById('mi-summary').innerHTML = [
        card(t('view.market_impact.card.trades'),          fmtN(total)),
        card(t('view.market_impact.card.avg_slip_all'),  fmtBps(avgAll), avgAll > state.spikeBps ? 'neg' : 'pos'),
        card(t('view.market_impact.card.impact_cliff'),    cliffLabel,     cliffCls),
        card(t('view.market_impact.card.spike_thresh'),    fmtBps(state.spikeBps)),
        card(t('view.market_impact.card.active_buckets'),  String(report.buckets.filter(b => b.trade_count > 0).length)),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderBars(report, spikeBps) {
    const wrap = document.getElementById('mi-bars');
    const maxAbs = Math.max(...report.buckets.map(b => Math.abs(b.max_slippage_bps)), 1);
    wrap.innerHTML = report.buckets.map(b => {
        if (b.trade_count === 0) {
            return `
                <div class="is-bar-row">
                    <div class="is-bar-label">${esc(b.label)}</div>
                    <div class="is-bar-track"><div class="is-bar-midline mi-axis-zero"></div></div>
                    <div class="is-bar-value">${esc(t('common.no_trades'))}</div>
                </div>`;
        }
        const widthAvg = Math.max(0, Math.min(100, (Math.abs(b.avg_slippage_bps) / maxAbs) * 100)).toFixed(2);
        const widthMax = Math.max(0, Math.min(100, (Math.abs(b.max_slippage_bps) / maxAbs) * 100)).toFixed(2);
        const colorCls = Math.abs(b.avg_slippage_bps) > spikeBps ? 'mi-fill-bust' : 'mi-fill-ok';
        return `
            <div class="is-bar-row">
                <div class="is-bar-label">${esc(b.label)}</div>
                <div class="is-bar-track">
                    <div class="is-bar-midline mi-axis-zero"></div>
                    <div class="mi-bar-fill mi-fill-max" data-bar-pct="${widthMax}"></div>
                    <div class="mi-bar-fill ${colorCls}" data-bar-pct="${widthAvg}"></div>
                </div>
                <div class="is-bar-value">
                    n=${b.trade_count} · avg ${esc(fmtBps(b.avg_slippage_bps))} · med ${esc(fmtBps(b.median_slippage_bps))} · max ${esc(fmtBps(b.max_slippage_bps))}
                </div>
            </div>
        `;
    }).join('');
    requestAnimationFrame(() => {
        wrap.querySelectorAll('.mi-bar-fill').forEach(el => {
            const pct = Number(el.dataset.barPct);
            if (Number.isFinite(pct)) el.style.width = pct + '%';
        });
    });
}

function renderDistribution(trades) {
    if (!window.uPlot) return;
    const counts = bucketParticipations(trades);
    const xs = BUCKET_LABELS.map((_, i) => i);
    const el = document.getElementById('mi-dist-chart');
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: {} },
        series: [
            { label: t('chart.series.bucket') },
            { label: t('chart.series.trade_count'), stroke: '#00e5ff', width: 1.5,
              fill: '#00e5ff33', points: { show: true, size: 5 } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(s => BUCKET_LABELS[s] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: false },
    }, [xs, counts], el);
}

function renderScatterChart(trades, spikeBps) {
    if (!window.uPlot) return;
    const el = document.getElementById('mi-scatter-chart');
    if (!el) return;
    el.innerHTML = '';
    const valid = (trades || []).filter(tr =>
        Number.isFinite(Number(tr.qty)) && Number.isFinite(Number(tr.adv)) && Number(tr.adv) > 0
        && Number.isFinite(Number(tr.slippage_bps)));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.market_impact.empty_scatter_chart">${esc(t('view.market_impact.empty_scatter_chart'))}</div>`;
        return;
    }
    const sorted = valid.map(tr => ({
        x: Number(tr.qty) / Number(tr.adv) * 100,
        y: Number(tr.slippage_bps),
    })).sort((a, b) => a.x - b.x);
    const xs = sorted.map(p => p.x);
    const ys = sorted.map(p => p.y);
    const thresh = xs.map(() => Number(spikeBps) || 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 240,
        scales: { x: { time: false, auto: true }, y: { auto: true } },
        series: [
            { label: t('view.market_impact.chart.pct_adv') },
            { label: t('view.market_impact.chart.slippage'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 5, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.market_impact.chart.threshold'),
              stroke: '#ff3860', width: 1.0, dash: [4, 4], points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => v.toFixed(2) + '%') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ys, thresh], el);
}

function showErr(msg) {
    const el = document.getElementById('mi-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('mi-err').style.display = 'none'; }
