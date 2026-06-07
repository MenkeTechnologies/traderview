// Intraday Heatmap view — P&L by 15-minute bucket across the trading day.
//
// Visual: 24×4 grid (hours × quarter-hour). Cell color intensity scaled
// to global max-abs PnL: greens for winning windows, reds for losing.
// Tooltip on hover shows trade count + total/avg PnL + win rate.

import { api } from '../api.js';
import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import {
    parseTradeBlob, validateInputs, buildBody,
    gridify, heatStyleClass, makeDemoTrades,
    fmtUSD, fmtPct,
} from '../_intraday_heatmap_inputs.js';

import { t } from '../i18n.js';
import { showToast } from '../toast.js';
let state = { tradesText: '', sessionOnly: true };

export async function renderIntradayHeatmap(mount, _appState) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 data-i18n="view.intraday_heatmap.h1.intraday_heatmap_15_min_pandl" class="view-title">// INTRADAY HEATMAP · 15-MIN P&amp;L</h1>

        <div class="chart-panel">
            <h2 data-i18n="view.intraday_heatmap.h2.trade_ledger">Trade ledger</h2>
            <p class="muted" data-i18n="view.intraday_heatmap.hint.format">One line per trade: timestamp pnl. Timestamps accept ISO 8601 (2024-01-15T14:30:00Z) or bare HH:MM (anchored to a fixed epoch date).</p>
            <textarea id="ih-trades" rows="8" placeholder="09:35 125.50&#10;09:45 -42.00&#10;..." data-tip="view.intraday_heatmap.tip.trades"></textarea>
            <div class="inline-form">
                <button data-i18n="view.intraday_heatmap.btn.load_demo_200_trades" data-tip="view.intraday_heatmap.tip.demo" data-shortcut="intraday_heatmap_demo" id="ih-demo" class="secondary" type="button">Load demo (200 trades)</button>
                <button data-i18n="view.intraday_heatmap.btn.clear" data-tip="view.intraday_heatmap.tip.clear" id="ih-clear" class="secondary" type="button">Clear</button>
                <button data-i18n="view.intraday_heatmap.btn.build_heatmap" data-tip="view.intraday_heatmap.tip.build" data-shortcut="intraday_heatmap_build" id="ih-run" class="primary" type="button">Build heatmap</button>
                <label data-tip="view.intraday_heatmap.tip.session"><input id="ih-session" type="checkbox" checked> <span data-i18n="view.intraday_heatmap.label.session">Session hours only (09:00-16:00)</span></label>
            </div>
        </div>

        <div id="ih-errors" class="boot" style="display:none"></div>
        <div id="ih-summary" class="cards"></div>

        <div class="chart-panel">
            <h2 data-i18n="view.intraday_heatmap.h2.15_min_pandl_heatmap">15-min P&amp;L heatmap</h2>
            <div id="ih-grid" class="ih-grid"></div>
            <p data-i18n="view.intraday_heatmap.hint.color_intensity_scaled_to_the_global_max_abs_pnl_g" class="muted">Color intensity scaled to the global max-abs PnL.
                Green = winning bucket. Red = losing. Dark = empty.
                Hover any cell for trade count / avg / win-rate.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.intraday_heatmap.h2.hour_chart">P&L by hour (aggregated)</h2>
            <div id="ih-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.intraday_heatmap.h2.activity_chart">Trade count per 15-min bucket (activity profile)</h2>
            <div id="ih-activity-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.intraday_heatmap.hint.activity" class="muted small">When you actually trade vs where you make money. Opening drives, lunch lull, closing surge — overlay this against the P&L heatmap to see if your activity profile lines up with your profitability profile.</p>
        </div>

        <div id="ih-err" class="boot" style="display:none;color:var(--red)"></div>
    `;
    document.getElementById('ih-demo').addEventListener('click', () => {
        const t = makeDemoTrades(42);
        document.getElementById('ih-trades').value =
            t.map(x => `${x.when.slice(11, 16)} ${x.pnl}`).join('\n');
    });
    document.getElementById('ih-clear').addEventListener('click', () => {
        document.getElementById('ih-trades').value = '';
    });
    document.getElementById('ih-run').addEventListener('click', () => {
        readInputs();
        void compute(tok);
    });
}

function readInputs() {
    state.tradesText = document.getElementById('ih-trades').value;
    state.sessionOnly = document.getElementById('ih-session').checked;
}

async function compute(tok) {
    hideErr();
    const errs = document.getElementById('ih-errors');
    errs.style.display = 'none';
    const { trades, errors } = parseTradeBlob(state.tradesText);
    if (errors.length) {
        const head = errors.slice(0, 8).map(e =>
            t('common.parse_error_inline', { line: e.line_no, msg: e.message, raw: e.raw.slice(0, 80) })).join('<br>');
        const more = errors.length > 8 ? `<br>${esc(t('common.and_n_more', { n: errors.length - 8 }))}` : '';
        errs.innerHTML = `<strong>${esc(t('common.parse_errors_lead', { n: errors.length }))}</strong><br>${head}${more}`;
        errs.style.display = 'block';
        if (trades.length === 0) return;
    }
    const err = validateInputs(trades);
    if (err) { showErr(err); showToast(err, { level: 'warning' }); return; }
    let res;
    try {
        res = await api.microIntradayHeatmap(buildBody(trades));
    } catch (e) {
        const m = t("common.error.api", { msg: e.message || e });
        showErr(m); showToast(m, { level: 'error' }); return;
    }
    if (!viewIsCurrent(tok)) return;
    renderSummary(res, trades);
    renderGrid(res);
    renderHourChart(res);
    renderActivityChart(res);
    showToast(t('view.intraday_heatmap.toast.done', {
        trades: trades.length,
        buckets: (res.buckets || []).filter(b => Number(b.trade_count) > 0).length,
    }), { level: 'success' });
}

function renderHourChart(report) {
    const el = document.getElementById('ih-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const buckets = (report && report.buckets) || [];
    if (!buckets.length) {
        el.innerHTML = `<div class="muted" data-i18n="view.intraday_heatmap.empty_chart">${esc(t('view.intraday_heatmap.empty_chart'))}</div>`;
        return;
    }
    const startH = state.sessionOnly ? 9  : 0;
    const endH   = state.sessionOnly ? 16 : 24;
    const { grid } = gridify(buckets);
    const labels = [];
    const ys = [];
    for (let h = startH; h < endH; h++) {
        let total = 0;
        let any = false;
        for (let q = 0; q < 4; q++) {
            const b = grid[h][q];
            if (b && Number.isFinite(Number(b.total_pnl))) { total += Number(b.total_pnl); any = true; }
        }
        labels.push(h.toString().padStart(2, '0') + ':00');
        ys.push(any ? total : null);
    }
    if (!ys.some(v => v != null)) {
        el.innerHTML = `<div class="muted" data-i18n="view.intraday_heatmap.empty_chart">${esc(t('view.intraday_heatmap.empty_chart'))}</div>`;
        return;
    }
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.intraday_heatmap.chart.hour') },
            { label: t('view.intraday_heatmap.chart.pnl'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.intraday_heatmap.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, ys, zero], el);
}

function renderActivityChart(report) {
    const el = document.getElementById('ih-activity-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const buckets = (report && report.buckets) || [];
    if (!buckets.length) {
        el.innerHTML = `<div class="muted" data-i18n="view.intraday_heatmap.empty_activity_chart">${esc(t('view.intraday_heatmap.empty_activity_chart'))}</div>`;
        return;
    }
    const startH = state.sessionOnly ? 9  : 0;
    const endH   = state.sessionOnly ? 16 : 24;
    const { grid } = gridify(buckets);
    const labels = [];
    const ys = [];
    for (let h = startH; h < endH; h++) {
        for (let q = 0; q < 4; q++) {
            const b = grid[h][q];
            labels.push(h.toString().padStart(2, '0') + ':' + (q * 15).toString().padStart(2, '0'));
            ys.push(b ? Number(b.trade_count) || 0 : 0);
        }
    }
    if (!ys.some(v => v > 0)) {
        el.innerHTML = `<div class="muted" data-i18n="view.intraday_heatmap.empty_activity_chart">${esc(t('view.intraday_heatmap.empty_activity_chart'))}</div>`;
        return;
    }
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.intraday_heatmap.chart.bucket') },
            { label: t('view.intraday_heatmap.chart.trade_count'),
              stroke: '#b86bff', width: 1.5, points: { show: true, size: 4 } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 50 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}

function renderSummary(report, trades) {
    const totalPnl = report.buckets.reduce((a, b) => a + (b.total_pnl || 0), 0);
    const activeBuckets = report.buckets.filter(b => b.trade_count > 0).length;
    const totalTrades = report.buckets.reduce((a, b) => a + b.trade_count, 0);
    const totalWins   = report.buckets.reduce((a, b) => a + b.win_count, 0);
    const winRate = totalTrades > 0 ? totalWins / totalTrades : NaN;
    document.getElementById('ih-summary').innerHTML = [
        card(t('view.intraday_heatmap.card.trades'),          String(trades.length)),
        card(t('view.intraday_heatmap.card.total_p_l'),       fmtUSD(totalPnl), totalPnl >= 0 ? 'pos' : 'neg'),
        card(t('view.intraday_heatmap.card.win_rate'),        fmtPct(winRate)),
        card(t('view.intraday_heatmap.card.active_buckets'),  `${activeBuckets} / 96`),
        card(t('view.intraday_heatmap.card.best_15_min'),     report.best_bucket_label || '—', 'pos'),
        card(t('view.intraday_heatmap.card.worst_15_min'),    report.worst_bucket_label || '—', 'neg'),
    ].join('');
}

function card(label, value, cls = '') {
    return `<div class="card">
        <div class="label">${esc(label)}</div>
        <div class="value ${cls}">${esc(value)}</div>
    </div>`;
}

function renderGrid(report) {
    const wrap = document.getElementById('ih-grid');
    const { grid, maxAbs } = gridify(report.buckets);
    const startH = state.sessionOnly ? 9  : 0;
    const endH   = state.sessionOnly ? 16 : 24;
    let html = `<div class="ih-row ih-header">
        <div class="ih-hlabel">hr</div>
        <div class="ih-cell ih-col-label">:00</div>
        <div class="ih-cell ih-col-label">:15</div>
        <div class="ih-cell ih-col-label">:30</div>
        <div class="ih-cell ih-col-label">:45</div>
    </div>`;
    for (let h = startH; h < endH; h++) {
        html += `<div class="ih-row">
            <div class="ih-hlabel">${h.toString().padStart(2, '0')}</div>`;
        for (let q = 0; q < 4; q++) {
            const b = grid[h][q];
            if (!b || b.trade_count === 0) {
                html += `<div class="ih-cell heat-empty" title="${h.toString().padStart(2, '0')}:${(q*15).toString().padStart(2, '0')} — no trades"></div>`;
            } else {
                const cls = heatStyleClass(b.total_pnl, maxAbs);
                const tip = `${b.label} — n=${b.trade_count} · total ${fmtUSD(b.total_pnl)} · avg ${fmtUSD(b.avg_pnl)} · win ${fmtPct(b.win_rate)}`;
                html += `<div class="ih-cell ${cls}" title="${esc(tip)}">
                    <span class="ih-cell-pnl">${esc(fmtUSD(b.total_pnl))}</span>
                </div>`;
            }
        }
        html += `</div>`;
    }
    wrap.innerHTML = html;
}

function showErr(msg) {
    const el = document.getElementById('ih-err');
    el.textContent = msg;
    el.style.display = 'block';
}

function hideErr() { document.getElementById('ih-err').style.display = 'none'; }
