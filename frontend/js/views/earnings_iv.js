// Earnings-week IV scanner + per-symbol straddle backtest detail.
import { api } from '../api.js';
import { esc, fmt } from '../util.js';
import { t, applyUiI18n } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

export async function renderEarningsIv(mount, _state, symbol) {
    const tok = currentViewToken();
    if (symbol) return renderDetail(mount, symbol.toUpperCase());

    const lists = await api.watchlists();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 data-i18n="view.earnings_iv.h1.earnings_week_iv_scanner" class="view-title">// EARNINGS-WEEK IV SCANNER</h1>
        <p class="muted small">
            Ranks every symbol with earnings in the next N days by the gap between
            implied move (ATM straddle ÷ spot) and the 8-quarter median realized move.
            <code>edge &gt; 0</code> means premium is rich (sell); <code>edge &lt; 0</code> means cheap (buy).
        </p>

        <div class="chart-panel">
            <form id="ivf" class="inline-form">
                <label><span data-i18n="view.earnings_iv.label.universe">Universe</span>
                    <select name="watchlist_id">
                        <option data-i18n="view.earnings_iv.opt.all_my_watchlists" value="">all my watchlists</option>
                        ${lists.map(w => `<option value="${w.id}">${esc(w.name)}</option>`).join('')}
                    </select>
                </label>
                <label><span data-i18n="view.earnings_iv.label.horizon_days">Horizon (days)</span>
                    <input name="horizon_days" type="number" value="7"></label>
                <label><span data-i18n="view.earnings_iv.label.limit">Limit</span>
                    <input name="limit" type="number" value="50"></label>
                <button data-i18n="view.earnings_iv.btn.scan" class="primary" type="submit">Scan</button>
            </form>
        </div>

        <div id="iv-result"></div>
    `;
    mount.querySelector('#ivf').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const el = mount.querySelector('#iv-result');
        if (!el) return;
        el.innerHTML = '<div class="boot" data-i18n="view.earnings_iv.status.scanning_chains">scanning… options chains can take ~1s per symbol</div>';
        try {
            const hits = await api.ivScan(
                fd.get('watchlist_id') || null,
                Number(fd.get('horizon_days') || 7),
                Number(fd.get('limit') || 50),
            );
            if (!viewIsCurrent(tok)) return;
            const el2 = mount.querySelector('#iv-result');
            if (el2) el2.innerHTML = renderTable(hits);
        } catch (err) {
            if (!viewIsCurrent(tok)) return;
            const el2 = mount.querySelector('#iv-result');
            if (el2) el2.innerHTML = `<p class="boot">${esc(err.message)}</p>`;
        }
    });
}

function renderTable(hits) {
    if (!hits.length) {
        return '<p data-i18n="view.earnings_iv.hint.no_earnings_inside_the_horizon_for_your_universe_a" class="muted">No earnings inside the horizon for your universe. Add some watchlist symbols and rerun.</p>';
    }
    return `<div class="chart-panel">
        <h2>${hits.length} candidates</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.earnings_iv.th.symbol">Symbol</th><th data-i18n="view.earnings_iv.th.earnings">Earnings</th><th data-i18n="view.earnings_iv.th.days">Days</th>
                <th data-i18n="view.earnings_iv.th.implied_move">Implied Move</th><th data-i18n="view.earnings_iv.th.median_realized">Median Realized</th><th data-i18n="view.earnings_iv.th.edge">Edge</th>
                <th data-i18n="view.earnings_iv.th.reco">Reco</th><th data-i18n="view.earnings_iv.th.long_p_l">Long P&L</th><th data-i18n="view.earnings_iv.th.short_p_l">Short P&L</th><th>n</th>
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
        <p data-i18n="view.earnings_iv.hint.long_short_p_l_is_per_quarter_average_return_on_1_" class="muted small">Long/Short P&L is per-quarter average return on $1 of premium across the historical sample.</p>
    </div>`;
}

async function renderDetail(mount, sym) {
    const tok = currentViewToken();
    mount.innerHTML = `
        <h1 class="view-title">// EARNINGS IV · ${esc(sym)}
            <a class="link small" href="#earnings-iv">← back to scan</a>
        </h1>
        <div id="iv-detail"><div class="boot">computing…</div></div>
    `;
    try {
        const r = await api.ivSymbol(sym);
        if (!viewIsCurrent(tok)) return;
        const recCls = r.backtest.recommendation === 'long' ? 'pos' :
                       r.backtest.recommendation === 'short' ? 'neg' : '';
        const detailEl = mount.querySelector('#iv-detail');
        if (!detailEl) return;
        detailEl.innerHTML = `
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.earnings_iv.card.earnings">Earnings</div>
                    <div class="value">${esc(r.earnings_date)}</div></div>
                <div class="card"><div class="label" data-i18n="view.earnings_iv.card.days_until">Days until</div>
                    <div class="value">${r.days_until}</div></div>
                <div class="card"><div class="label" data-i18n="view.earnings_iv.card.spot">Spot</div>
                    <div class="value">${fmt(r.spot)}</div></div>
                <div class="card"><div class="label" data-i18n="view.earnings_iv.card.atm_strike">ATM strike</div>
                    <div class="value">${fmt(r.atm_strike)}</div></div>
                <div class="card"><div class="label" data-i18n="view.earnings_iv.card.implied_move">Implied move</div>
                    <div class="value">${fmt(r.implied_move_pct, 2)}%</div></div>
                <div class="card"><div class="label" data-i18n="view.earnings_iv.card.median_realized">Median realized</div>
                    <div class="value">${fmt(r.backtest.median_realized_pct, 2)}%</div></div>
                <div class="card"><div class="label" data-i18n="view.earnings_iv.card.edge">Edge</div>
                    <div class="value ${r.backtest.edge_pct >= 0 ? 'neg' : 'pos'}">${r.backtest.edge_pct >= 0 ? '+' : ''}${fmt(r.backtest.edge_pct, 2)}%</div></div>
                <div class="card"><div class="label" data-i18n="view.earnings_iv.card.recommendation">Recommendation</div>
                    <div class="value ${recCls}">${r.backtest.recommendation.toUpperCase()}</div></div>
            </div>

            <div class="chart-panel">
                <h2>${esc(t('view.earnings_iv.h2.straddle_backtest', { samples: r.backtest.samples }))}</h2>
                <table class="trades">
                    <thead><tr><th data-i18n="view.earnings_iv.th.strategy">Strategy</th><th data-i18n="view.earnings_iv.th.avg_p_l_1_premium">Avg P&L / $1 premium</th><th data-i18n="view.earnings_iv.th.win_rate">Win rate</th></tr></thead>
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
                <h2 data-i18n="view.earnings_iv.h2.historical_earnings_moves">Historical earnings moves</h2>
                ${r.historical.length ? `<table class="trades">
                    <thead><tr><th data-i18n="view.earnings_iv.th.date">Date</th><th data-i18n="view.earnings_iv.th.close_before">Close before</th><th data-i18n="view.earnings_iv.th.close_after">Close after</th>
                        <th data-i18n="view.earnings_iv.th.abs_move">Abs move %</th><th data-i18n="view.earnings_iv.th.direction">Direction</th></tr></thead>
                    <tbody>${r.historical.map(h => `
                        <tr><td>${esc(h.earnings_date)}</td>
                        <td>${fmt(h.close_before)}</td>
                        <td>${fmt(h.close_after)}</td>
                        <td class="${h.abs_move_pct > r.implied_move_pct ? 'pos' : 'neg'}">${fmt(h.abs_move_pct, 2)}%</td>
                        <td class="${h.direction === 'up' ? 'pos' : 'neg'}">${h.direction}</td></tr>
                    `).join('')}</tbody>
                </table>` : '<p data-i18n="view.earnings_iv.hint.no_historical_earnings_moves_in_cached_price_bars_" class="muted">No historical earnings moves in cached price bars yet — wait a moment for the prices fetcher.</p>'}
            </div>

            <div class="chart-panel">
                <h2>${esc(t('view.earnings_iv.h2.straddle_pricing', { expiration: r.expiration }))}</h2>
                <table class="trades">
                    <tbody>
                        <tr><td data-i18n="view.earnings_iv.row.call_mid">Call mid</td><td>${fmt(r.call_mid)}</td></tr>
                        <tr><td data-i18n="view.earnings_iv.row.put_mid">Put mid</td> <td>${fmt(r.put_mid)}</td></tr>
                        <tr><td><strong data-i18n="view.earnings_iv.row.total_premium">Total premium</strong></td>
                            <td><strong>${fmt(r.call_mid + r.put_mid)}</strong></td></tr>
                        <tr><td><strong data-i18n="view.earnings_iv.row.implied_move">Implied move</strong></td>
                            <td><strong>${fmt(r.implied_move_pct, 2)}%</strong></td></tr>
                    </tbody>
                </table>
            </div>
        `;
        try { applyUiI18n(detailEl); } catch (_) {}
    } catch (e) {
        if (!viewIsCurrent(tok)) return;
        const detailEl = mount.querySelector('#iv-detail');
        if (detailEl) detailEl.innerHTML = `<p class="boot">${esc(e.message)}</p>`;
    }
}
