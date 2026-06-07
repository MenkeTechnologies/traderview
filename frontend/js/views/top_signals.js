import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

export async function renderTopSignals(mount) {
    const tok = currentViewToken();
    const lists = await api.watchlists();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 data-i18n="view.top_signals.h1.top_signals" class="view-title">// TOP SIGNALS</h1>
        <p data-i18n="view.top_signals.hint.highest_and_lowest_scoring_symbols_across_your_wat" class="muted small">Highest- and lowest-scoring symbols across your watchlists (StockInvest-style ranking).</p>

        <div class="chart-panel">
            <form id="top-form" class="inline-form">
                <label><span data-i18n="view.top_signals.label.watchlist">Watchlist</span>
                    <select name="watchlist_id" data-tip="view.top_signals.tip.watchlist">
                        <option data-i18n="view.top_signals.opt.all_my_watchlists" value="">all my watchlists</option>
                        ${lists.map(w => `<option value="${w.id}">${esc(w.name)}</option>`).join('')}
                    </select>
                </label>
                <label><span data-i18n="view.top_signals.label.limit">Limit</span>
                    <input name="limit" type="number" value="25" data-tip="view.top_signals.tip.limit"></label>
                <button data-i18n="view.top_signals.btn.refresh" data-tip="view.top_signals.tip.refresh" data-shortcut="top_signals_refresh" class="primary" type="submit">Refresh</button>
            </form>
        </div>

        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.top_signals.h2.top_buy_signals">Top BUY signals</h2>
                <div id="buys"><span class="tv-spinner-inline" role="status" aria-label="loading"></span></div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.top_signals.h2.top_sell_signals">Top SELL signals</h2>
                <div id="sells"><span class="tv-spinner-inline" role="status" aria-label="loading"></span></div>
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.top_signals.h2.score_chart">Score: buys (+) vs sells (-)</h2>
            <div id="ts-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.top_signals.h2.rsi_chart">RSI(14) per ranked symbol</h2>
            <div id="ts-rsi-chart" style="width:100%;height:220px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.top_signals.h2.signals_chart">Signal count per ranked symbol</h2>
            <div id="ts-signals-chart" style="width:100%;height:220px"></div>
        </div>
    `;
    const refresh = async () => {
        const form = mount.querySelector('#top-form');
        if (!form) return;
        const fd = new FormData(form);
        const wid = fd.get('watchlist_id') || null;
        const limit = Number(fd.get('limit') || 25);
        const buysEl0 = mount.querySelector('#buys');
        const sellsEl0 = mount.querySelector('#sells');
        if (buysEl0) buysEl0.innerHTML = '<div class="boot" data-i18n="common.status.scoring">scoring…</div>';
        if (sellsEl0) sellsEl0.innerHTML = '<div class="boot" data-i18n="common.status.scoring">scoring…</div>';
        try {
            const [buys, sells] = await Promise.all([
                api.topSignals('buy', wid, limit),
                api.topSignals('sell', wid, limit),
            ]);
            if (!viewIsCurrent(tok)) return;
            const buysEl = mount.querySelector('#buys');
            const sellsEl = mount.querySelector('#sells');
            if (buysEl) buysEl.innerHTML  = renderList(buys, 'buy');
            if (sellsEl) sellsEl.innerHTML = renderList(sells, 'sell');
            renderScoreChart(buys, sells);
            renderRsiChart(buys, sells);
            renderSignalsChart(buys, sells);
            const total = (buys.hits?.length || 0) + (sells.hits?.length || 0);
            showToast(t('view.top_signals.toast.done', {
                buys: buys.hits?.length || 0, sells: sells.hits?.length || 0,
            }), { level: total > 0 ? 'success' : 'info' });
        } catch (e) {
            if (!viewIsCurrent(tok)) return;
            const buysEl = mount.querySelector('#buys');
            const sellsEl = mount.querySelector('#sells');
            if (buysEl) buysEl.innerHTML  = `<p class="boot">${esc(e.message)}</p>`;
            if (sellsEl) sellsEl.innerHTML = '';
            showToast(t('toast.error.api', { err: e.message }), { level: 'error' });
        }
    };
    mount.querySelector('#top-form').addEventListener('submit', (e) => { e.preventDefault(); refresh(); });
    refresh();
}

function renderSignalsChart(buys, sells) {
    const el = document.getElementById('ts-signals-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const bRows = (buys?.hits  || []).filter(h => Number.isFinite(Number(h.signal_count)));
    const sRows = (sells?.hits || []).filter(h => Number.isFinite(Number(h.signal_count)));
    if (bRows.length + sRows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.top_signals.empty_signals_chart">${esc(t('view.top_signals.empty_signals_chart'))}</div>`;
        return;
    }
    bRows.sort((a, b) => Number(b.signal_count) - Number(a.signal_count));
    sRows.sort((a, b) => Number(b.signal_count) - Number(a.signal_count));
    const rows = [...bRows, ...sRows];
    const labels = rows.map(h => h.symbol);
    const xs = labels.map((_, i) => i + 1);
    const buyY  = rows.map((h, i) => i < bRows.length ?  Number(h.signal_count) : null);
    const sellY = rows.map((h, i) => i < bRows.length ? null : Number(h.signal_count));
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.top_signals.chart.symbol') },
            { label: t('view.top_signals.chart.buy_signals'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 12, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.top_signals.chart.sell_signals'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 12, fill: '#ff3860', stroke: '#ff3860' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, buyY, sellY], el);
}

function renderRsiChart(buys, sells) {
    const el = document.getElementById('ts-rsi-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const bRows = (buys?.hits  || []).filter(h => Number.isFinite(Number(h.rsi14)));
    const sRows = (sells?.hits || []).filter(h => Number.isFinite(Number(h.rsi14)));
    if (bRows.length + sRows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.top_signals.empty_rsi_chart">${esc(t('view.top_signals.empty_rsi_chart'))}</div>`;
        return;
    }
    const rows = [...bRows, ...sRows];
    const labels = rows.map(h => h.symbol);
    const xs = labels.map((_, i) => i + 1);
    const buyY  = rows.map((h, i) => i < bRows.length ?  Number(h.rsi14) : null);
    const sellY = rows.map((h, i) => i < bRows.length ? null : Number(h.rsi14));
    const ob    = xs.map(() => 70);
    const os    = xs.map(() => 30);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { range: () => [0, 100] } },
        series: [
            { label: t('view.top_signals.chart.symbol') },
            { label: t('view.top_signals.chart.buy_rsi'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 10, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.top_signals.chart.sell_rsi'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 10, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.top_signals.chart.ob_70'),
              stroke: '#ff3860', width: 1.0, dash: [4, 4],
              points: { show: false } },
            { label: t('view.top_signals.chart.os_30'),
              stroke: '#7af0a8', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, buyY, sellY, ob, os], el);
}

function renderScoreChart(buys, sells) {
    const el = document.getElementById('ts-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const bRows = (buys?.hits  || []).filter(h => Number.isFinite(Number(h.score)));
    const sRows = (sells?.hits || []).filter(h => Number.isFinite(Number(h.score)));
    if (bRows.length + sRows.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.top_signals.empty_chart">${esc(t('view.top_signals.empty_chart'))}</div>`;
        return;
    }
    bRows.sort((a, b) => Number(b.score) - Number(a.score));
    sRows.sort((a, b) => Number(a.score) - Number(b.score));
    const rows = [...bRows, ...sRows];
    const labels = rows.map(h => h.symbol);
    const xs = labels.map((_, i) => i + 1);
    const buyY  = rows.map((h, i) => i < bRows.length ?  Number(h.score) : null);
    const sellY = rows.map((h, i) => i < bRows.length ? null : Number(h.score));
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.top_signals.chart.symbol') },
            { label: t('view.top_signals.chart.buy'),
              stroke: '#7af0a8', width: 0,
              points: { show: true, size: 10, fill: '#7af0a8', stroke: '#7af0a8' } },
            { label: t('view.top_signals.chart.sell'),
              stroke: '#ff3860', width: 0,
              points: { show: true, size: 10, fill: '#ff3860', stroke: '#ff3860' } },
            { label: t('view.top_signals.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, buyY, sellY, zero], el);
}

function renderList(r, side) {
    if (!r.hits.length) return `<p class="muted">${esc(t('view.top_signals.empty', { side }))}</p>`;
    return `<table class="trades">
        <thead><tr><th>#</th><th data-i18n="view.top_signals.th.symbol">Symbol</th><th data-i18n="view.top_signals.th.score">Score</th><th data-i18n="view.top_signals.th.summary">Summary</th>
            <th data-i18n="view.top_signals.th.close">Close</th><th data-i18n="view.top_signals.th.rsi">RSI</th><th data-i18n="view.top_signals.th.signals">Signals</th></tr></thead>
        <tbody>${r.hits.map((h, i) => {
            const cls = h.score >= 3 ? 'pos' : h.score <= -3 ? 'neg' : '';
            return `<tr data-context-scope="symbol-row" data-symbol="${esc(h.symbol)}">
                <td>${i + 1}</td>
                <td><a href="#research/${encodeURIComponent(h.symbol)}">${esc(h.symbol)}</a></td>
                <td class="${cls}">${h.score >= 0 ? '+' : ''}${h.score}</td>
                <td class="${cls}">${h.summary}</td>
                <td>${fmt(h.last_close)}</td>
                <td>${h.rsi14 != null ? fmt(h.rsi14, 1) : '—'}</td>
                <td>${h.signal_count}</td>
            </tr>`;
        }).join('')}</tbody></table>`;
}
