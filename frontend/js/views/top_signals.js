import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

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
                    <select name="watchlist_id">
                        <option data-i18n="view.top_signals.opt.all_my_watchlists" value="">all my watchlists</option>
                        ${lists.map(w => `<option value="${w.id}">${esc(w.name)}</option>`).join('')}
                    </select>
                </label>
                <label><span data-i18n="view.top_signals.label.limit">Limit</span>
                    <input name="limit" type="number" value="25"></label>
                <button data-i18n="view.top_signals.btn.refresh" class="primary" type="submit">Refresh</button>
            </form>
        </div>

        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.top_signals.h2.top_buy_signals">Top BUY signals</h2>
                <div id="buys">loading…</div>
            </div>
            <div class="chart-panel">
                <h2 data-i18n="view.top_signals.h2.top_sell_signals">Top SELL signals</h2>
                <div id="sells">loading…</div>
            </div>
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
        if (buysEl0) buysEl0.innerHTML = '<div class="boot">scoring…</div>';
        if (sellsEl0) sellsEl0.innerHTML = '<div class="boot">scoring…</div>';
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
        } catch (e) {
            if (!viewIsCurrent(tok)) return;
            const buysEl = mount.querySelector('#buys');
            const sellsEl = mount.querySelector('#sells');
            if (buysEl) buysEl.innerHTML  = `<p class="boot">${esc(e.message)}</p>`;
            if (sellsEl) sellsEl.innerHTML = '';
        }
    };
    mount.querySelector('#top-form').addEventListener('submit', (e) => { e.preventDefault(); refresh(); });
    refresh();
}

function renderList(r, side) {
    if (!r.hits.length) return `<p class="muted">No ${side} candidates in this universe.</p>`;
    return `<table class="trades">
        <thead><tr><th>#</th><th data-i18n="view.top_signals.th.symbol">Symbol</th><th data-i18n="view.top_signals.th.score">Score</th><th data-i18n="view.top_signals.th.summary">Summary</th>
            <th data-i18n="view.top_signals.th.close">Close</th><th data-i18n="view.top_signals.th.rsi">RSI</th><th data-i18n="view.top_signals.th.signals">Signals</th></tr></thead>
        <tbody>${r.hits.map((h, i) => {
            const cls = h.score >= 3 ? 'pos' : h.score <= -3 ? 'neg' : '';
            return `<tr>
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
