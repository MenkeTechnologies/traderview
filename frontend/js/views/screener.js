import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';

export async function renderScreener(mount) {
    const tok = currentViewToken();
    const lists = await api.watchlists();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 data-i18n="view.screener.h1.screener" class="view-title">// SCREENER</h1>
        <p data-i18n="view.screener.hint.runs_technical_signals_across_your_watchlists_retu" class="muted small">Runs technical signals across your watchlists, returns ranked hits. Score range -10..+10.</p>

        <div class="chart-panel">
            <form id="sc-form" class="inline-form">
                <label><span data-i18n="view.screener.label.watchlist">Watchlist</span>
                    <select name="watchlist_id" data-tip="view.screener.tip.watchlist">
                        <option data-i18n="view.screener.opt.all_my_watchlists" value="">all my watchlists</option>
                        ${lists.map(w => `<option value="${w.id}">${esc(w.name)}</option>`).join('')}
                    </select>
                </label>
                <label><span data-i18n="view.screener.label.min_score">Min score</span>
                    <input name="min_score" type="number" value="3" data-tip="view.screener.tip.min_score"></label>
                <label><span data-i18n="view.screener.label.max_score">Max score</span>
                    <input name="max_score" type="number" data-tip="view.screener.tip.max_score"></label>
                <label><span data-i18n="view.screener.label.summary">Summary</span>
                    <select name="summary" data-tip="view.screener.tip.summary">
                        <option data-i18n="view.screener.opt.any" value="">any</option>
                        <option data-i18n="view.screener.opt.buy" value="buy">buy</option>
                        <option data-i18n="view.screener.opt.hold" value="hold">hold</option>
                        <option data-i18n="view.screener.opt.sell" value="sell">sell</option>
                    </select>
                </label>
                <label><span data-i18n="view.screener.label.history_days">History days</span>
                    <input name="days" type="number" value="365" data-tip="view.screener.tip.days"></label>
                <label><span data-i18n="view.screener.label.limit">Limit</span>
                    <input name="limit" type="number" value="50" data-tip="view.screener.tip.limit"></label>
                <button data-i18n="view.screener.btn.run" data-tip="view.screener.tip.run" data-shortcut="screener_run" class="primary" type="submit">Run</button>
            </form>
        </div>

        <div id="sc-result"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.screener.h2.score_chart">Score per hit symbol</h2>
            <div id="sc-chart" style="width:100%;height:240px"></div>
        </div>
    `;
    mount.querySelector('#sc-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const opts = {};
        for (const [k, v] of fd.entries()) if (v !== '') opts[k] = v;
        const el = mount.querySelector('#sc-result');
        if (!el) return;
        el.innerHTML = '<div class="boot" data-i18n="common.status.scanning">scanning…</div>';
        try {
            const r = await api.screenerRun(opts);
            if (!viewIsCurrent(tok)) return;
            const elNow = mount.querySelector('#sc-result');
            if (elNow) elNow.innerHTML = renderResult(r);
            renderScoreChart(r.hits);
            showToast(t('view.screener.toast.done', {
                hits: r.hits.length, universe: r.universe_size,
            }), { level: r.hits.length > 0 ? 'success' : 'info' });
        } catch (err) {
            if (!viewIsCurrent(tok)) return;
            const elNow = mount.querySelector('#sc-result');
            if (elNow) elNow.innerHTML = `<p class="boot">${esc(err.message)}</p>`;
            showToast(t('toast.error.api', { err: err.message }), { level: 'error' });
        }
    });
}

function renderScoreChart(hits) {
    const el = document.getElementById('sc-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const valid = (hits || []).filter(h => Number.isFinite(Number(h.score)));
    if (valid.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.screener.empty_chart">${esc(t('view.screener.empty_chart'))}</div>`;
        return;
    }
    valid.sort((a, b) => Number(b.score) - Number(a.score));
    const labels = valid.map(h => h.symbol);
    const ys = valid.map(h => Number(h.score));
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: {}, y: { auto: true } },
        series: [
            { label: t('view.screener.chart.symbol_idx') },
            { label: t('view.screener.chart.score'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 10, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.screener.chart.zero'),
              stroke: '#ffd84a', width: 1.0, dash: [4, 4],
              points: { show: false } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 40 },
        ],
        legend: { show: true },
    }, [xs, ys, zero], el);
}

function renderResult(r) {
    return `<div class="chart-panel">
        <h2>${esc(t('view.screener.h2.hits_summary', { hits: r.hits.length, universe: r.universe_size }))}</h2>
        ${r.hits.length ? `<table class="trades">
            <thead><tr><th data-i18n="view.screener.th.symbol">Symbol</th><th data-i18n="view.screener.th.score">Score</th><th data-i18n="view.screener.th.summary">Summary</th>
            <th data-i18n="view.screener.th.close">Close</th><th data-i18n="view.screener.th.rsi_14">RSI(14)</th><th data-i18n="view.screener.th.sma_50">SMA(50)</th><th data-i18n="view.screener.th.sma_200">SMA(200)</th>
            <th data-i18n="view.screener.th.macd_hist">MACD hist</th><th data-i18n="view.screener.th.signals">Signals</th></tr></thead>
            <tbody>${r.hits.map(h => {
                const cls = h.score >= 3 ? 'pos' : h.score <= -3 ? 'neg' : '';
                return `<tr data-context-scope="symbol-row" data-symbol="${esc(h.symbol)}">
                    <td><a href="#research/${encodeURIComponent(h.symbol)}">${esc(h.symbol)}</a></td>
                    <td class="${cls}">${h.score >= 0 ? '+' : ''}${h.score}</td>
                    <td class="${cls}">${h.summary}</td>
                    <td>${fmt(h.last_close)}</td>
                    <td>${h.rsi14 != null ? fmt(h.rsi14, 1) : '—'}</td>
                    <td>${h.sma50 != null ? fmt(h.sma50) : '—'}</td>
                    <td>${h.sma200 != null ? fmt(h.sma200) : '—'}</td>
                    <td>${h.macd_hist != null ? fmt(h.macd_hist, 3) : '—'}</td>
                    <td>${h.signal_count}</td>
                </tr>`;
            }).join('')}</tbody>
        </table>` : '<p data-i18n="view.screener.hint.no_hits" class="muted">No hits.</p>'}
    </div>`;
}
