// Earnings-week IV scanner + per-symbol straddle backtest detail.
import { api } from '../api.js';
import { esc, fmt } from '../util.js';

export async function renderEarningsIv(mount, _state, symbol) {
    if (symbol) return renderDetail(mount, symbol.toUpperCase());

    const lists = await api.watchlists();
    mount.innerHTML = `
        <h1 class="view-title">// EARNINGS-WEEK IV SCANNER</h1>
        <p class="muted small">
            Ranks every symbol with earnings in the next N days by the gap between
            implied move (ATM straddle ÷ spot) and the 8-quarter median realized move.
            <code>edge &gt; 0</code> means premium is rich (sell); <code>edge &lt; 0</code> means cheap (buy).
        </p>

        <div class="chart-panel">
            <form id="ivf" class="inline-form">
                <label>Universe
                    <select name="watchlist_id">
                        <option value="">all my watchlists</option>
                        ${lists.map(w => `<option value="${w.id}">${esc(w.name)}</option>`).join('')}
                    </select>
                </label>
                <label>Horizon (days) <input name="horizon_days" type="number" value="7"></label>
                <label>Limit <input name="limit" type="number" value="50"></label>
                <button class="primary" type="submit">Scan</button>
            </form>
        </div>

        <div id="iv-result"></div>
    `;
    document.getElementById('ivf').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const el = document.getElementById('iv-result');
        el.innerHTML = '<div class="boot">scanning… options chains can take ~1s per symbol</div>';
        try {
            const hits = await api.ivScan(
                fd.get('watchlist_id') || null,
                Number(fd.get('horizon_days') || 7),
                Number(fd.get('limit') || 50),
            );
            el.innerHTML = renderTable(hits);
        } catch (err) {
            el.innerHTML = `<p class="boot">${esc(err.message)}</p>`;
        }
    });
}

function renderTable(hits) {
    if (!hits.length) {
        return '<p class="muted">No earnings inside the horizon for your universe. Add some watchlist symbols and rerun.</p>';
    }
    return `<div class="chart-panel">
        <h2>${hits.length} candidates</h2>
        <table class="trades">
            <thead><tr>
                <th>Symbol</th><th>Earnings</th><th>Days</th>
                <th>Implied Move</th><th>Median Realized</th><th>Edge</th>
                <th>Reco</th><th>Long P&L</th><th>Short P&L</th><th>n</th>
            </tr></thead>
            <tbody>${hits.map(h => {
                const rec = h.recommendation;
                const recCls = rec === 'long' ? 'pos' : rec === 'short' ? 'neg' : '';
                const edgeCls = h.edge_pct >= 0 ? 'neg' : 'pos'; // positive edge = sell
                return `<tr>
                    <td><a href="#earnings-iv/${encodeURIComponent(h.symbol)}">${esc(h.symbol)}</a></td>
                    <td>${esc(h.earnings_date)}</td>
                    <td>${h.days_until}</td>
                    <td>${fmt(h.implied_move_pct, 2)}%</td>
                    <td>${fmt(h.median_realized_pct, 2)}%</td>
                    <td class="${edgeCls}">${h.edge_pct >= 0 ? '+' : ''}${fmt(h.edge_pct, 2)}%</td>
                    <td class="${recCls}"><strong>${rec.toUpperCase()}</strong></td>
                    <td class="${h.long_avg_pnl >= 0 ? 'pos' : 'neg'}">${(h.long_avg_pnl * 100).toFixed(0)}%</td>
                    <td class="${h.short_avg_pnl >= 0 ? 'pos' : 'neg'}">${(h.short_avg_pnl * 100).toFixed(0)}%</td>
                    <td>${h.samples}</td>
                </tr>`;
            }).join('')}</tbody>
        </table>
        <p class="muted small">Long/Short P&L is per-quarter average return on $1 of premium across the historical sample.</p>
    </div>`;
}

async function renderDetail(mount, sym) {
    mount.innerHTML = `
        <h1 class="view-title">// EARNINGS IV · ${esc(sym)}
            <a class="link small" href="#earnings-iv">← back to scan</a>
        </h1>
        <div id="iv-detail"><div class="boot">computing…</div></div>
    `;
    try {
        const r = await api.ivSymbol(sym);
        const recCls = r.backtest.recommendation === 'long' ? 'pos' :
                       r.backtest.recommendation === 'short' ? 'neg' : '';
        document.getElementById('iv-detail').innerHTML = `
            <div class="cards">
                <div class="card"><div class="label">Earnings</div>
                    <div class="value">${esc(r.earnings_date)}</div></div>
                <div class="card"><div class="label">Days until</div>
                    <div class="value">${r.days_until}</div></div>
                <div class="card"><div class="label">Spot</div>
                    <div class="value">${fmt(r.spot)}</div></div>
                <div class="card"><div class="label">ATM strike</div>
                    <div class="value">${fmt(r.atm_strike)}</div></div>
                <div class="card"><div class="label">Implied move</div>
                    <div class="value">${fmt(r.implied_move_pct, 2)}%</div></div>
                <div class="card"><div class="label">Median realized</div>
                    <div class="value">${fmt(r.backtest.median_realized_pct, 2)}%</div></div>
                <div class="card"><div class="label">Edge</div>
                    <div class="value ${r.backtest.edge_pct >= 0 ? 'neg' : 'pos'}">${r.backtest.edge_pct >= 0 ? '+' : ''}${fmt(r.backtest.edge_pct, 2)}%</div></div>
                <div class="card"><div class="label">Recommendation</div>
                    <div class="value ${recCls}">${r.backtest.recommendation.toUpperCase()}</div></div>
            </div>

            <div class="chart-panel">
                <h2>Straddle backtest — ${r.backtest.samples} historical earnings</h2>
                <table class="trades">
                    <thead><tr><th>Strategy</th><th>Avg P&L / $1 premium</th><th>Win rate</th></tr></thead>
                    <tbody>
                        <tr><td>LONG straddle (buy call + put)</td>
                            <td class="${r.backtest.long_avg_pnl >= 0 ? 'pos' : 'neg'}">${(r.backtest.long_avg_pnl * 100).toFixed(1)}%</td>
                            <td>${(r.backtest.long_win_rate * 100).toFixed(0)}%</td></tr>
                        <tr><td>SHORT straddle (sell call + put)</td>
                            <td class="${r.backtest.short_avg_pnl >= 0 ? 'pos' : 'neg'}">${(r.backtest.short_avg_pnl * 100).toFixed(1)}%</td>
                            <td>${(r.backtest.short_win_rate * 100).toFixed(0)}%</td></tr>
                    </tbody>
                </table>
            </div>

            <div class="chart-panel">
                <h2>Historical earnings moves</h2>
                ${r.historical.length ? `<table class="trades">
                    <thead><tr><th>Date</th><th>Close before</th><th>Close after</th>
                        <th>Abs move %</th><th>Direction</th></tr></thead>
                    <tbody>${r.historical.map(h => `
                        <tr><td>${esc(h.earnings_date)}</td>
                        <td>${fmt(h.close_before)}</td>
                        <td>${fmt(h.close_after)}</td>
                        <td class="${h.abs_move_pct > r.implied_move_pct ? 'pos' : 'neg'}">${fmt(h.abs_move_pct, 2)}%</td>
                        <td class="${h.direction === 'up' ? 'pos' : 'neg'}">${h.direction}</td></tr>
                    `).join('')}</tbody>
                </table>` : '<p class="muted">No historical earnings moves in cached price bars yet — wait a moment for the prices fetcher.</p>'}
            </div>

            <div class="chart-panel">
                <h2>Straddle pricing — expiration ${esc(r.expiration)}</h2>
                <table class="trades">
                    <tbody>
                        <tr><td>Call mid</td><td>${fmt(r.call_mid)}</td></tr>
                        <tr><td>Put mid</td> <td>${fmt(r.put_mid)}</td></tr>
                        <tr><td><strong>Total premium</strong></td>
                            <td><strong>${fmt(r.call_mid + r.put_mid)}</strong></td></tr>
                        <tr><td><strong>Implied move</strong></td>
                            <td><strong>${fmt(r.implied_move_pct, 2)}%</strong></td></tr>
                    </tbody>
                </table>
            </div>
        `;
    } catch (e) {
        document.getElementById('iv-detail').innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}
