import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderScreener(mount) {
    const tok = currentViewToken();
    const lists = await api.watchlists();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title">// SCREENER</h1>
        <p class="muted small">Runs technical signals across your watchlists, returns ranked hits. Score range -10..+10.</p>

        <div class="chart-panel">
            <form id="sc-form" class="inline-form">
                <label>Watchlist
                    <select name="watchlist_id">
                        <option value="">all my watchlists</option>
                        ${lists.map(w => `<option value="${w.id}">${esc(w.name)}</option>`).join('')}
                    </select>
                </label>
                <label>Min score <input name="min_score" type="number" value="3"></label>
                <label>Max score <input name="max_score" type="number"></label>
                <label>Summary
                    <select name="summary">
                        <option value="">any</option>
                        <option value="buy">buy</option>
                        <option value="hold">hold</option>
                        <option value="sell">sell</option>
                    </select>
                </label>
                <label>History days <input name="days" type="number" value="365"></label>
                <label>Limit <input name="limit" type="number" value="50"></label>
                <button class="primary" type="submit">Run</button>
            </form>
        </div>

        <div id="sc-result"></div>
    `;
    mount.querySelector('#sc-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const opts = {};
        for (const [k, v] of fd.entries()) if (v !== '') opts[k] = v;
        const el = mount.querySelector('#sc-result');
        if (!el) return;
        el.innerHTML = '<div class="boot">scanning…</div>';
        try {
            const r = await api.screenerRun(opts);
            if (!viewIsCurrent(tok)) return;
            const elNow = mount.querySelector('#sc-result');
            if (elNow) elNow.innerHTML = renderResult(r);
        } catch (err) {
            if (!viewIsCurrent(tok)) return;
            const elNow = mount.querySelector('#sc-result');
            if (elNow) elNow.innerHTML = `<p class="boot">${esc(err.message)}</p>`;
        }
    });
}

function renderResult(r) {
    return `<div class="chart-panel">
        <h2>${r.hits.length} hits of ${r.universe_size} symbols scanned</h2>
        ${r.hits.length ? `<table class="trades">
            <thead><tr><th>Symbol</th><th>Score</th><th>Summary</th>
            <th>Close</th><th>RSI(14)</th><th>SMA(50)</th><th>SMA(200)</th>
            <th>MACD hist</th><th>Signals</th></tr></thead>
            <tbody>${r.hits.map(h => {
                const cls = h.score >= 3 ? 'pos' : h.score <= -3 ? 'neg' : '';
                return `<tr>
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
        </table>` : '<p class="muted">No hits.</p>'}
    </div>`;
}
