// Paper-trading simulator — Warrior Trading SimTrader equivalent.
import { api } from '../api.js';
import { esc, fmt, fmtDateTime } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { t } from '../i18n.js';
import { showToast } from '../toast.js';
import { tConfirm } from '../dialog.js';

export async function renderPaper(mount) {
    const tok = currentViewToken();
    const accounts = await api.paperAccounts();
    if (!viewIsCurrent(tok)) return;
    if (!accounts.length) {
        await api.paperEnsure();
        if (!viewIsCurrent(tok)) return;
        return renderPaper(mount);
    }
    const savedId = localStorage.getItem('paper.acctId');
    const acct = accounts.find(a => a.id === savedId) || accounts[0];
    const [positions, orders] = await Promise.all([
        api.paperPositions(acct.id),
        api.paperOrders(acct.id, 50),
    ]);
    if (!viewIsCurrent(tok)) return;

    // Live unrealized P&L — fetch quotes for held symbols.
    const symList = positions.map(p => p.symbol);
    let quotes = {};
    if (symList.length) {
        try {
            const promises = symList.map(s => api.quote(s).catch(() => null));
            const qs = await Promise.all(promises);
            if (!viewIsCurrent(tok)) return;
            qs.forEach(q => { if (q) quotes[q.symbol] = q; });
        } catch (_) {}
    }
    let posValue = 0, unrealized = 0;
    positions.forEach(p => {
        const q = quotes[p.symbol];
        if (q) {
            const mark = Number(q.price);
            const qty = Number(p.qty);
            posValue += mark * qty;
            unrealized += (mark - Number(p.avg_price)) * qty;
        }
    });
    const cash = Number(acct.cash);
    const equity = cash + posValue;
    const total = equity - Number(acct.starting_cash);
    const totalPct = Number(acct.starting_cash) > 0 ? (total / Number(acct.starting_cash) * 100) : 0;

    mount.innerHTML = `
        <h1 class="view-title">// PAPER TRADING · ${esc(acct.name)}</h1>

        <div class="inline-form">
            <select id="acct-sel" data-tip="view.paper.tip.account_sel">
                ${accounts.map(a => `<option value="${esc(a.id)}"${a.id === acct.id ? ' selected' : ''}>${esc(a.name)}</option>`).join('')}
            </select>
            <input id="acct-name" placeholder="account name" data-i18n-placeholder="view.paper.placeholder.account_name" maxlength="60">
            <input id="acct-cash" type="number" min="1" step="1000" value="200000" data-tip="view.paper.tip.account_cash">
            <button id="acct-create" data-i18n="view.paper.btn.new_account" data-tip="view.paper.tip.account_create">NEW ACCOUNT</button>
            <button id="acct-rename" class="link" data-i18n="view.paper.btn.rename_account" data-tip="view.paper.tip.account_rename">Rename</button>
            <button id="acct-delete" class="link" data-i18n="view.paper.btn.delete_account" data-tip="view.paper.tip.account_delete">Delete</button>
            <label class="small"><input type="checkbox" id="acct-drip" ${acct.drip ? 'checked' : ''} data-tip="view.paper.tip.drip"> <span data-i18n="view.paper.label.drip">DRIP</span></label>
        </div>

        <div class="cards">
            <div class="card"><div class="label" data-i18n="view.paper.card.cash">Cash</div><div class="value">$${fmt(cash)}</div></div>
            <div class="card"><div class="label" data-i18n="view.paper.card.position_value">Position value</div><div class="value">$${fmt(posValue)}</div></div>
            <div class="card"><div class="label" data-i18n="view.paper.card.equity">Equity</div><div class="value">$${fmt(equity)}</div></div>
            <div class="card"><div class="label" data-i18n="view.paper.card.total_pnl">Total P&L</div>
                <div class="value ${total >= 0 ? 'pos' : 'neg'}">${total >= 0 ? '+' : ''}$${fmt(total)} (${totalPct.toFixed(2)}%)</div></div>
            <div class="card"><div class="label" data-i18n="view.paper.card.unrealized">Unrealized</div>
                <div class="value ${unrealized >= 0 ? 'pos' : 'neg'}">${unrealized >= 0 ? '+' : ''}$${fmt(unrealized)}</div></div>
            <div class="card"><div class="label" data-i18n="view.paper.card.starting_cash">Starting cash</div><div class="value">$${fmt(acct.starting_cash)}</div></div>
        </div>

        <div class="panel-grid">
            <div class="chart-panel">
                <h2 data-i18n="view.paper.h2.order_ticket">Order ticket</h2>
                <form id="ord-form" class="inline-form">
                    <input name="symbol" data-shortcut="focus_search" data-tip="view.paper.tip.symbol" placeholder="symbol" data-i18n-placeholder="common.placeholder.symbol" required style="text-transform:uppercase">
                    <select name="side" data-tip="view.paper.tip.side">
                        <option data-i18n="view.paper.opt.buy" value="buy">BUY</option>
                        <option data-i18n="view.paper.opt.sell" value="sell">SELL</option>
                        <option data-i18n="view.paper.opt.short" value="short">SHORT</option>
                        <option data-i18n="view.paper.opt.cover" value="cover">COVER</option>
                    </select>
                    <input name="qty" type="number" step="0.01" placeholder="qty" data-i18n-placeholder="common.placeholder.qty" data-tip="view.paper.tip.qty" required>
                    <select name="order_type" data-tip="view.paper.tip.order_type">
                        <option data-i18n="view.paper.opt.market" value="market">market</option>
                        <option data-i18n="view.paper.opt.limit" value="limit">limit</option>
                        <option data-i18n="view.paper.opt.stop" value="stop">stop</option>
                        <option data-i18n="view.paper.opt.trailing" value="trailing">trailing</option>
                    </select>
                    <input name="limit_price" type="number" step="0.01" placeholder="limit" data-i18n-placeholder="common.placeholder.limit" data-tip="view.paper.tip.limit">
                    <input name="stop_price"  type="number" step="0.01" placeholder="stop" data-i18n-placeholder="common.placeholder.stop" data-tip="view.paper.tip.stop">
                    <input name="trail_value" type="number" step="0.01" min="0" placeholder="trail" data-i18n-placeholder="common.placeholder.trail" data-tip="view.paper.tip.trail">
                    <select name="trail_unit" data-tip="view.paper.tip.trail_unit">
                        <option value="usd">$</option>
                        <option value="pct">%</option>
                    </select>
                    <select name="time_in_force" data-tip="view.paper.tip.tif">
                        <option value="gtc">GTC</option>
                        <option value="day">DAY</option>
                        <option value="gtd">GTD</option>
                    </select>
                    <input name="expire_at" type="datetime-local" data-tip="view.paper.tip.gtd_expiry">
                    <button data-i18n="view.paper.btn.submit" data-tip="view.paper.tip.submit" data-shortcut="paper_submit" class="primary" type="submit">SUBMIT</button>
                </form>
                <h2 data-i18n="view.paper.h2.spread_ticket">Option spread ticket</h2>
                <form id="spread-form" class="inline-form">
                    <input name="leg1" placeholder="buy leg OCC (e.g. AAPL260117C00190000)" data-i18n-placeholder="view.paper.placeholder.spread_buy" data-tip="view.paper.tip.spread_leg" required style="min-width:220px">
                    <input name="leg2" placeholder="sell leg OCC" data-i18n-placeholder="view.paper.placeholder.spread_sell" required style="min-width:220px">
                    <input name="qty" type="number" min="1" step="1" value="1" data-tip="view.paper.tip.spread_qty">
                    <button type="button" id="spread-preview" data-i18n="view.paper.btn.preview_spread" data-tip="view.paper.tip.preview_spread">PREVIEW</button>
                    <button data-i18n="view.paper.btn.submit_spread" class="primary" type="submit">SPREAD</button>
                </form>
                <div id="spread-preview-out" class="muted small"></div>

                <h2 data-i18n="view.paper.h2.bracket_ticket">Bracket (OCO) ticket</h2>
                <form id="bracket-form" class="inline-form">
                    <input name="symbol" placeholder="symbol" data-i18n-placeholder="common.placeholder.symbol" data-tip="view.paper.tip.symbol" required style="text-transform:uppercase">
                    <select name="side" data-tip="view.paper.tip.bracket_side">
                        <option data-i18n="view.paper.opt.buy" value="buy">BUY</option>
                        <option data-i18n="view.paper.opt.short" value="short">SHORT</option>
                    </select>
                    <input name="qty" type="number" step="0.01" placeholder="qty" data-i18n-placeholder="common.placeholder.qty" required>
                    <select name="entry_type" data-tip="view.paper.tip.bracket_entry">
                        <option data-i18n="view.paper.opt.market" value="market">market</option>
                        <option data-i18n="view.paper.opt.limit" value="limit">limit</option>
                    </select>
                    <input name="limit_price" type="number" step="0.01" placeholder="limit" data-i18n-placeholder="common.placeholder.limit">
                    <input name="stop_loss" type="number" step="0.01" placeholder="stop loss" data-i18n-placeholder="common.placeholder.stop_loss" data-tip="view.paper.tip.bracket_stop" required>
                    <input name="take_profit" type="number" step="0.01" placeholder="target" data-i18n-placeholder="common.placeholder.target" data-tip="view.paper.tip.bracket_target" required>
                    <button data-i18n="view.paper.btn.submit_bracket" class="primary" type="submit">BRACKET</button>
                </form>
                <button data-i18n="view.paper.btn.reset_account_200k" data-tip="view.paper.tip.reset" class="link" id="reset">Reset account ($200k)</button>
            </div>

            <div class="chart-panel">
                <h2 data-i18n="view.paper.h2.open_positions">Open positions</h2>
                ${positions.length ? `<table class="trades">
                    <thead><tr><th data-i18n="view.paper.th.sym">Sym</th><th data-i18n="view.paper.th.qty">Qty</th><th data-i18n="view.paper.th.avg">Avg</th><th data-i18n="view.paper.th.last">Last</th>
                    <th data-i18n="view.paper.th.unrealized">Unrealized</th><th data-i18n="view.paper.th.realized">Realized</th></tr></thead>
                    <tbody>${positions.map(p => {
                        const q = quotes[p.symbol];
                        const last = q ? Number(q.price) : null;
                        const u = last != null ? (last - Number(p.avg_price)) * Number(p.qty) : null;
                        const cls = u != null && u >= 0 ? 'pos' : 'neg';
                        return `<tr data-context-scope="symbol-row" data-symbol="${esc(p.symbol)}">
                            <td><a href="#research/${encodeURIComponent(p.symbol)}">${esc(p.symbol)}</a></td>
                            <td>${fmt(p.qty, 0)}</td>
                            <td>${fmt(p.avg_price)}</td>
                            <td>${last != null ? fmt(last) : '—'}</td>
                            <td class="${cls}">${u != null ? (u >= 0 ? '+' : '') + '$' + fmt(u) : '—'}</td>
                            <td class="${Number(p.realized_pnl) >= 0 ? 'pos' : 'neg'}">$${fmt(p.realized_pnl)}</td>
                        </tr>`;
                    }).join('')}</tbody></table>` : '<p data-i18n="view.paper.hint.no_open_positions" class="muted">No open positions.</p>'}
            </div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.paper.h2.auto_invest">Auto-invest</h2>
            <form id="recur-form" class="inline-form">
                <input name="symbol" placeholder="symbol — or blank with a target" data-i18n-placeholder="view.paper.placeholder.recur_symbol" style="text-transform:uppercase">
                <input name="target_id" placeholder="target id (cash-flow rebalance)" data-i18n-placeholder="view.paper.placeholder.recur_target" data-tip="view.paper.tip.recur_target" style="min-width:170px">
                <input name="notional_usd" type="number" min="1" step="50" value="500" data-tip="view.paper.tip.recur_notional">
                <select name="cadence">
                    <option value="weekly" selected>weekly</option>
                    <option value="daily">daily</option>
                    <option value="monthly">monthly</option>
                </select>
                <button class="primary" type="submit" data-i18n="view.paper.btn.recur_add">ADD</button>
            </form>
            <div id="recur-list" class="muted small"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.paper.h2.attribution">P&L attribution</h2>
            <div id="paper-attribution" class="muted small"></div>
        </div>

        <div class="chart-panel" id="paper-greeks-panel" style="display:none">
            <h2 data-i18n="view.paper.h2.greeks">Option greeks</h2>
            <div id="paper-greeks"></div>
        </div>

        ${accounts.length > 1 ? `
        <div class="chart-panel">
            <h2 data-i18n="view.paper.h2.leaderboard">Strategy leaderboard</h2>
            <div id="paper-leaderboard"></div>
        </div>` : ''}

        <div class="chart-panel">
            <h2 data-i18n="view.paper.h2.equity_curve">Equity curve</h2>
            <div id="paper-equity-summary" class="muted small"></div>
            <div id="paper-equity-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.paper.h2.dividends">Dividends received</h2>
            <div id="paper-dividends"></div>
        </div>

        <div class="chart-panel" id="paper-splits-panel" style="display:none">
            <h2 data-i18n="view.paper.h2.splits">Split adjustments</h2>
            <div id="paper-splits"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.paper.h2.unrealized_chart">Unrealized P&L per open position</h2>
            <div id="paper-chart" style="width:100%;height:240px"></div>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.paper.h2.notional_chart">Position notional per symbol</h2>
            <div id="paper-notional-chart" style="width:100%;height:220px"></div>
            <p data-i18n="view.paper.hint.notional" class="muted small">Per-symbol capital allocation (qty × last). Reveals concentration risk independent of P/L — a 60% notional in one name is concentration even if it's green today.</p>
        </div>

        <div class="chart-panel">
            <h2 data-i18n="view.paper.h2.order_history">Order history</h2>
            ${orders.length ? `<table class="trades">
                <thead><tr><th data-i18n="view.paper.th.submitted">Submitted</th><th data-i18n="view.paper.th.symbol">Symbol</th><th data-i18n="view.paper.th.side">Side</th><th data-i18n="view.paper.th.qty_2">Qty</th><th data-i18n="view.paper.th.type">Type</th>
                <th data-i18n="view.paper.th.status">Status</th><th data-i18n="view.paper.th.fill_price">Fill price</th><th data-i18n="view.paper.th.filled">Filled</th><th></th></tr></thead>
                <tbody>${orders.map(o => `
                    <tr data-context-scope="symbol-row" data-symbol="${esc(o.symbol)}">
                        <td>${fmtDateTime(o.submitted_at)}</td>
                        <td>${esc(o.symbol)}</td>
                        <td>${o.side}</td>
                        <td>${fmt(o.qty, 0)}</td>
                        <td>${o.order_type}${o.limit_price != null ? ' @' + fmt(o.limit_price) : ''}${o.stop_price != null ? ' stop ' + fmt(o.stop_price) : ''}${o.trail_value != null ? ' ' + (o.trail_is_pct ? (Number(o.trail_value) * 100).toFixed(1) + '%' : '$' + fmt(o.trail_value)) + (o.status === 'pending' && o.trail_extreme != null ? ' (hwm ' + fmt(o.trail_extreme) + ')' : '') : ''}</td>
                        <td class="${o.status === 'filled' ? 'pos' : (o.status === 'rejected' ? 'neg' : '')}">${o.status}${(o.status === 'pending' || o.status === 'held') && o.cancel_at ? ' · exp ' + fmtDateTime(o.cancel_at) : ''}</td>
                        <td>${o.filled_price != null ? fmt(o.filled_price) : '—'}</td>
                        <td>${o.filled_at ? fmtDateTime(o.filled_at) : '—'}</td>
                        <td>${(o.status === 'pending' || o.status === 'held') ? `<button class="ord-cancel" data-id="${esc(o.id)}" data-i18n="common.btn.cancel">${esc(t('common.btn.cancel'))}</button>` : ''}</td>
                    </tr>`).join('')}</tbody></table>` : '<p data-i18n="view.paper.hint.no_orders_yet" class="muted">No orders yet.</p>'}
        </div>
    `;

    renderUnrealizedChart(positions, quotes);
    renderNotionalChart(positions, quotes);
    api.paperEquityHistory(acct.id)
        .then(h => { if (viewIsCurrent(tok)) renderEquityCurve(h); })
        .catch(() => {});
    api.paperRecurringList()
        .then(rows => { if (viewIsCurrent(tok)) renderRecurring(mount, rows); })
        .catch(() => {});
    api.paperAttribution(acct.id)
        .then(a => { if (viewIsCurrent(tok)) renderAttribution(a); })
        .catch(() => {});
    mount.querySelector('#recur-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        try {
            await api.paperRecurringCreate(acct.id, {
                symbol: fd.get('symbol').trim().toUpperCase() || null,
                target_id: (fd.get('target_id') || '').trim() || null,
                notional_usd: String(fd.get('notional_usd')),
                cadence: fd.get('cadence'),
            });
            if (!viewIsCurrent(tok)) return;
            const rows = await api.paperRecurringList();
            renderRecurring(mount, rows);
        } catch (err) { showToast(t('common.error', { err: err.message }), { level: 'error' }); }
    });
    if (positions.some(p => p.symbol.length > 15)) {
        api.paperOptionGreeks(acct.id)
            .then(g => { if (viewIsCurrent(tok)) renderGreeks(g); })
            .catch(() => {});
    }
    if (accounts.length > 1) {
        api.paperAccountComparison()
            .then(rows => { if (viewIsCurrent(tok)) renderLeaderboard(rows, acct.id); })
            .catch(() => {});
    }
    api.paperDividends(acct.id)
        .then(d => { if (viewIsCurrent(tok)) renderDividends(d); })
        .catch(() => {});
    api.paperSplits(acct.id)
        .then(s => { if (viewIsCurrent(tok)) renderSplits(s); })
        .catch(() => {});

    mount.querySelector('#ord-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const body = {
            symbol: fd.get('symbol').trim().toUpperCase(),
            side: fd.get('side'),
            qty: Number(fd.get('qty')),
            order_type: fd.get('order_type'),
            limit_price: fd.get('limit_price') ? Number(fd.get('limit_price')) : null,
            stop_price: fd.get('stop_price') ? Number(fd.get('stop_price')) : null,
            // A "5" with % selected means 5% — the engine wants 0.05.
            trail_value: fd.get('trail_value')
                ? Number(fd.get('trail_value')) / (fd.get('trail_unit') === 'pct' ? 100 : 1)
                : null,
            trail_is_pct: fd.get('trail_value') ? fd.get('trail_unit') === 'pct' : null,
            time_in_force: fd.get('time_in_force'),
            // datetime-local has no zone; toISOString converts the
            // browser-local wall time to the UTC instant the API wants.
            expire_at: fd.get('time_in_force') === 'gtd' && fd.get('expire_at')
                ? new Date(fd.get('expire_at')).toISOString()
                : null,
        };
        try {
            const o = await api.paperSubmit(acct.id, body);
            if (!viewIsCurrent(tok)) return;
            if (o.status === 'rejected') {
                showToast(t('view.paper.alert.order_rejected', { reason: o.reject_reason || t('common.empty.unknown') }), { level: 'error' });
            } else if (o.status === 'filled') {
                showToast(t('view.paper.toast.filled', {
                    side: body.side, qty: body.qty, symbol: body.symbol,
                    price: o.filled_price != null ? fmt(o.filled_price) : '—',
                }), { level: 'success' });
            } else {
                showToast(t('view.paper.toast.submitted', { side: body.side, qty: body.qty, symbol: body.symbol }), { level: 'info' });
            }
            renderPaper(mount);
        } catch (err) { showToast(t('common.error', { err: err.message }), { level: 'error' }); }
    });
    mount.querySelector('#acct-drip').addEventListener('change', async (e) => {
        try {
            await api.paperSetDrip(acct.id, e.target.checked);
            showToast(t(e.target.checked ? 'view.paper.toast.drip_on' : 'view.paper.toast.drip_off'), { level: 'success' });
        } catch (err) { showToast(t('common.error', { err: err.message }), { level: 'error' }); }
    });
    mount.querySelector('#acct-sel').addEventListener('change', (e) => {
        localStorage.setItem('paper.acctId', e.target.value);
        renderPaper(mount);
    });
    mount.querySelector('#acct-create').addEventListener('click', async () => {
        const name = mount.querySelector('#acct-name').value.trim();
        if (!name) { showToast(t('view.paper.toast.name_required'), { level: 'error' }); return; }
        try {
            const a = await api.paperAccountCreate(name, Number(mount.querySelector('#acct-cash').value) || 200000);
            if (!viewIsCurrent(tok)) return;
            localStorage.setItem('paper.acctId', a.id);
            renderPaper(mount);
        } catch (err) { showToast(t('common.error', { err: err.message }), { level: 'error' }); }
    });
    mount.querySelector('#acct-rename').addEventListener('click', async () => {
        const name = mount.querySelector('#acct-name').value.trim();
        if (!name) { showToast(t('view.paper.toast.name_required'), { level: 'error' }); return; }
        try {
            await api.paperAccountRename(acct.id, name);
            if (!viewIsCurrent(tok)) return;
            renderPaper(mount);
        } catch (err) { showToast(t('common.error', { err: err.message }), { level: 'error' }); }
    });
    mount.querySelector('#acct-delete').addEventListener('click', async () => {
        if (!await tConfirm('view.paper.confirm.delete_account', { name: acct.name }, { level: 'danger' })) return;
        try {
            await api.paperAccountDelete(acct.id);
            if (!viewIsCurrent(tok)) return;
            localStorage.removeItem('paper.acctId');
            renderPaper(mount);
        } catch (err) { showToast(t('common.error', { err: err.message }), { level: 'error' }); }
    });
    mount.querySelector('#spread-preview').addEventListener('click', async () => {
        const form = mount.querySelector('#spread-form');
        const fd = new FormData(form);
        const out = mount.querySelector('#spread-preview-out');
        const leg1 = (fd.get('leg1') || '').trim().toUpperCase();
        const leg2 = (fd.get('leg2') || '').trim().toUpperCase();
        if (!leg1 || !leg2) { out.textContent = t('view.paper.err.spread_legs'); return; }
        out.textContent = t('common.loading');
        try {
            const r = await api.paperSpreadPreview({
                legs: [
                    { symbol: leg1, buy: true, ratio: 1 },
                    { symbol: leg2, buy: false, ratio: 1 },
                ],
                qty: Number(fd.get('qty')) || 1,
            });
            const p = r.payoff;
            out.innerHTML = `
                <strong>${r.net_premium_usd >= 0 ? 'Credit' : 'Debit'} $${Math.abs(r.net_premium_usd).toFixed(2)}</strong>
                \u00b7 max profit $${p.max_profit.toFixed(0)} \u00b7 max loss $${p.max_loss.toFixed(0)}
                \u00b7 breakeven${p.breakevens.length === 1 ? '' : 's'} ${p.breakevens.length ? p.breakevens.map(b => b.toFixed(2)).join(', ') : '\u2014'}
                \u00b7 legs @ ${r.legs.map(l => l.mid.toFixed(2)).join(' / ')}`;
        } catch (err) {
            out.textContent = t('common.error', { err: err.message });
        }
    });
    mount.querySelector('#spread-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        try {
            const r = await api.paperSpreadCreate(acct.id, {
                legs: [
                    { symbol: fd.get('leg1').trim().toUpperCase(), buy: true, ratio: 1 },
                    { symbol: fd.get('leg2').trim().toUpperCase(), buy: false, ratio: 1 },
                ],
                qty: Number(fd.get('qty')) || 1,
            });
            if (!viewIsCurrent(tok)) return;
            showToast(t('view.paper.toast.spread_filled', {
                premium: (r.net_premium_usd >= 0 ? '+' : '') + r.net_premium_usd.toFixed(2),
            }), { level: 'success' });
            renderPaper(mount);
        } catch (err) { showToast(t('common.error', { err: err.message }), { level: 'error' }); }
    });
    mount.querySelector('#bracket-form').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        try {
            await api.paperBracketCreate(acct.id, {
                symbol: fd.get('symbol').trim().toUpperCase(),
                side: fd.get('side'),
                qty: Number(fd.get('qty')),
                entry_type: fd.get('entry_type'),
                limit_price: fd.get('limit_price') ? Number(fd.get('limit_price')) : null,
                stop_loss: Number(fd.get('stop_loss')),
                take_profit: Number(fd.get('take_profit')),
            });
            if (!viewIsCurrent(tok)) return;
            showToast(t('view.paper.toast.bracket_submitted'), { level: 'success' });
            renderPaper(mount);
        } catch (err) { showToast(t('common.error', { err: err.message }), { level: 'error' }); }
    });
    mount.querySelectorAll('.ord-cancel').forEach(btn => btn.addEventListener('click', async () => {
        try {
            await api.paperOrderCancel(btn.dataset.id);
            if (!viewIsCurrent(tok)) return;
            renderPaper(mount);
        } catch (err) { showToast(t('common.error', { err: err.message }), { level: 'error' }); }
    }));
    mount.querySelector('#reset').addEventListener('click', async () => {
        if (!await tConfirm('view.paper.confirm.reset', {}, { level: 'danger' })) return;
        try {
            await api.paperReset(acct.id, 200000);
            if (!viewIsCurrent(tok)) return;
            showToast(t('view.paper.toast.reset'), { level: 'success' });
            renderPaper(mount);
        } catch (err) { showToast(t('toast.error.api', { err: err.message }), { level: 'error' }); }
    });
}

function holdFmt(secs) {
    if (secs >= 86400) return (secs / 86400).toFixed(1) + 'd';
    if (secs >= 3600) return (secs / 3600).toFixed(1) + 'h';
    return Math.round(secs / 60) + 'm';
}

function renderAttribution(a) {
    const el = document.getElementById('paper-attribution');
    if (!el) return;
    if (!a.symbols.length) {
        el.innerHTML = `<p class="muted" data-i18n="view.paper.empty.attribution">${esc(t('view.paper.empty.attribution'))}</p>`;
        return;
    }
    const money = (v) => `<span class="${v >= 0 ? 'pos' : 'neg'}">${v >= 0 ? '+' : ''}$${fmt(v)}</span>`;
    const st = a.stats;
    el.innerHTML = `
        <p><strong>Trading:</strong> ${money(a.total_trading_pnl)} \u00b7 <strong>Dividends:</strong> ${money(a.total_dividends)} \u00b7 <strong>Fees:</strong> <span class="neg">\u2212$${fmt(a.total_fees)}</span></p>
        ${st ? `<div class="cards">
            <div class="card"><div class="label">Expectancy / trade</div>
                <div class="value ${st.expectancy >= 0 ? 'pos' : 'neg'}">${st.expectancy >= 0 ? '+' : ''}$${fmt(st.expectancy)}</div>
                <div class="small muted">${st.trades} closed trips</div></div>
            <div class="card"><div class="label">Win rate</div>
                <div class="value">${(st.win_rate * 100).toFixed(0)}%</div>
                <div class="small muted">avg win $${fmt(st.avg_win)} / avg loss $${fmt(st.avg_loss)}</div></div>
            <div class="card"><div class="label">Profit factor</div>
                <div class="value">${st.profit_factor != null ? st.profit_factor.toFixed(2) : '\u2014'}</div>
                <div class="small muted">best ${money(st.largest_win)} \u00b7 worst ${money(st.largest_loss)}</div></div>
            <div class="card"><div class="label">Kelly (record-implied)</div>
                <div class="value">${st.kelly_fraction != null ? (st.kelly_fraction * 100).toFixed(1) + '%' : '\u2014'}</div>
                <div class="small muted">${st.kelly_fraction != null ? 'size at half: ' + (st.kelly_fraction * 50).toFixed(1) + '%' : 'needs wins AND losses'} \u00b7 streak ${st.current_streak > 0 ? '+' + st.current_streak : st.current_streak} (best +${st.longest_win_streak} / worst \u2212${st.longest_loss_streak})</div></div>
            ${a.hold ? `<div class="card"><div class="label">Avg hold (win / loss)</div>
                <div class="value">${holdFmt(a.hold.avg_hold_secs_winners)} / ${holdFmt(a.hold.avg_hold_secs_losers)}</div>
                ${a.hold.behavioral_flag ? `<div class="small neg" data-i18n="view.paper.flag.riding_losers">${esc(t('view.paper.flag.riding_losers'))}</div>` : ''}</div>` : ''}
        </div>` : ''}
        <table class="trades">
            <thead><tr><th>Symbol</th><th>Trading P&L</th><th>Closed trips</th><th>Dividends</th><th>Fees</th></tr></thead>
            <tbody>${a.symbols.map(s => `
                <tr>
                    <td>${esc(s.symbol)}</td>
                    <td>${money(s.trading_pnl)}</td>
                    <td>${s.closed_trips}</td>
                    <td>${s.dividends ? money(s.dividends) : '\u2014'}</td>
                    <td>$${fmt(s.fees)}</td>
                </tr>`).join('')}
            </tbody>
        </table>
        ${a.months.length ? `
        <h3 class="small" data-i18n="view.paper.h3.monthly">By month</h3>
        <table class="trades">
            <thead><tr><th>Month</th><th>Trading</th><th>Dividends</th><th>Total</th><th>Trips</th></tr></thead>
            <tbody>${a.months.map(m => {
                const total = m.trading_pnl + m.dividends;
                return `<tr>
                    <td>${esc(m.month)}</td>
                    <td>${money(m.trading_pnl)}</td>
                    <td>${m.dividends ? money(m.dividends) : '\u2014'}</td>
                    <td>${money(total)}</td>
                    <td>${m.closed_trips}</td>
                </tr>`;
            }).join('')}
            </tbody>
        </table>` : ''}
        <p class="muted small" data-i18n="view.paper.hint.attribution">Realized record only — closed round trips (FIFO from fills, fees netted) plus dividends. Open positions\u2019 unrealized P&L lives in the positions table above.</p>`;
}

function renderRecurring(mount, rows) {
    const el = mount.querySelector('#recur-list');
    if (!el) return;
    if (!rows.length) {
        el.innerHTML = `<p class="muted" data-i18n="view.paper.empty.recurring">${esc(t('view.paper.empty.recurring'))}</p>`;
        return;
    }
    el.innerHTML = `<table class="trades">
        <thead><tr><th>Symbol</th><th>Notional</th><th>Cadence</th><th>Next run</th><th>Last</th><th></th><th></th></tr></thead>
        <tbody>${rows.map(r => `
            <tr class="${r.enabled ? '' : 'muted'}">
                <td>${r.symbol ? esc(r.symbol) : `<em data-i18n="view.paper.label.target_mode">${esc(t('view.paper.label.target_mode'))}</em>`}</td>
                <td>$${fmt(r.notional_usd)}</td>
                <td>${esc(r.cadence)}</td>
                <td class="small">${new Date(r.next_run_at).toLocaleString()}</td>
                <td class="small">${esc(r.last_status || '—')}</td>
                <td><button class="link recur-toggle" data-id="${esc(r.id)}" data-on="${r.enabled ? 0 : 1}">${r.enabled ? 'pause' : 'resume'}</button></td>
                <td><button class="link recur-del" data-id="${esc(r.id)}" data-i18n="common.btn.cancel">${esc(t('common.btn.cancel'))}</button></td>
            </tr>`).join('')}
        </tbody></table>`;
    el.querySelectorAll('.recur-toggle').forEach(btn => btn.addEventListener('click', async () => {
        try {
            await api.paperRecurringToggle(btn.dataset.id, btn.dataset.on === '1');
            renderRecurring(mount, await api.paperRecurringList());
        } catch (err) { showToast(t('common.error', { err: err.message }), { level: 'error' }); }
    }));
    el.querySelectorAll('.recur-del').forEach(btn => btn.addEventListener('click', async () => {
        try {
            await api.paperRecurringDelete(btn.dataset.id);
            renderRecurring(mount, await api.paperRecurringList());
        } catch (err) { showToast(t('common.error', { err: err.message }), { level: 'error' }); }
    }));
}

function renderGreeks(g) {
    const panel = document.getElementById('paper-greeks-panel');
    const el = document.getElementById('paper-greeks');
    if (!panel || !el || !g.positions.length) return;
    panel.style.display = '';
    const f = (v, d = 1) => v != null ? v.toFixed(d) : '\u2014';
    el.innerHTML = `
        <p class="small"><strong>Net:</strong> \u0394 ${f(g.net_delta)} \u00b7 \u0393 ${f(g.net_gamma, 3)} \u00b7 \u0398/day ${f(g.net_theta_per_day, 2)} \u00b7 vega ${f(g.net_vega, 1)}
        <span class="muted">(positions missing chain IV are listed but excluded from nets)</span></p>
        <table class="trades">
            <thead><tr><th>Contract</th><th>Qty</th><th>Spot</th><th>IV</th><th>\u0394</th><th>\u0393</th><th>\u0398/day</th><th>Vega</th></tr></thead>
            <tbody>${g.positions.map(p => `
                <tr>
                    <td class="small">${esc(p.symbol)}</td>
                    <td>${fmt(p.qty, 0)}</td>
                    <td>${p.spot.toFixed(2)}</td>
                    <td>${p.iv != null ? (p.iv * 100).toFixed(1) + '%' : '\u2014'}</td>
                    <td class="${(p.delta ?? 0) >= 0 ? 'pos' : 'neg'}">${f(p.delta)}</td>
                    <td>${f(p.gamma, 3)}</td>
                    <td class="${(p.theta_per_day ?? 0) >= 0 ? 'pos' : 'neg'}">${f(p.theta_per_day, 2)}</td>
                    <td>${f(p.vega)}</td>
                </tr>`).join('')}
            </tbody>
        </table>`;
}

function renderLeaderboard(rows, currentId) {
    const el = document.getElementById('paper-leaderboard');
    if (!el || !rows || !rows.length) return;
    el.innerHTML = `<table class="trades">
        <thead><tr><th>#</th><th data-i18n="view.paper.th.account">Account</th><th data-i18n="view.paper.th.equity">Equity</th><th data-i18n="view.paper.th.return">Return</th><th data-i18n="view.paper.th.max_dd">Max DD</th></tr></thead>
        <tbody>${rows.map((r, i) => `
            <tr${r.account_id === currentId ? ' class="lf-current"' : ''}>
                <td>${i + 1}</td>
                <td>${esc(r.name)}${r.currently_underwater ? ' <span class="neg small">↓</span>' : ''}</td>
                <td>$${fmt(r.equity)}</td>
                <td class="${(r.return_pct ?? 0) >= 0 ? 'pos' : 'neg'}">${r.return_pct != null ? (r.return_pct >= 0 ? '+' : '') + r.return_pct.toFixed(2) + '%' : '—'}</td>
                <td>${r.max_drawdown_pct != null ? r.max_drawdown_pct.toFixed(2) + '%' : '—'}</td>
            </tr>`).join('')}
        </tbody></table>
        <p class="muted small" data-i18n="view.paper.hint.leaderboard">Return is measured against each account's starting cash — comparable across accounts created at different times. ↓ = currently below its high-water mark.</p>`;
}

function renderSplits(rows) {
    const panel = document.getElementById('paper-splits-panel');
    const el = document.getElementById('paper-splits');
    // Splits are rare — the panel only appears once one has been applied.
    if (!panel || !el || !rows || !rows.length) return;
    panel.style.display = '';
    el.innerHTML = `
        <table class="trades">
            <thead><tr><th data-i18n="view.paper.th.split_date">Split date</th><th data-i18n="view.paper.th.sym">Sym</th><th data-i18n="view.paper.th.ratio">Ratio</th>
            <th data-i18n="view.paper.th.qty_before">Qty before</th><th data-i18n="view.paper.th.qty_after">Qty after</th></tr></thead>
            <tbody>${rows.map(r => `
                <tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
                    <td>${esc(r.split_date)}</td>
                    <td><a href="#research/${encodeURIComponent(r.symbol)}">${esc(r.symbol)}</a></td>
                    <td>${fmt(r.numerator, 0)}:${fmt(r.denominator, 0)}</td>
                    <td>${fmt(r.qty_before, 0)}</td>
                    <td>${fmt(r.qty_after, 0)}</td>
                </tr>`).join('')}</tbody></table>`;
}

function renderDividends(rows) {
    const el = document.getElementById('paper-dividends');
    if (!el) return;
    if (!rows || !rows.length) {
        el.innerHTML = `<p class="muted" data-i18n="view.paper.empty_dividends">${esc(t('view.paper.empty_dividends'))}</p>`;
        return;
    }
    const total = rows.reduce((s, r) => s + Number(r.cash_credited), 0);
    el.innerHTML = `
        <div class="muted small"><span data-i18n="view.paper.dividends.total">${esc(t('view.paper.dividends.total'))}</span>:
            <span class="${total >= 0 ? 'pos' : 'neg'}">${total >= 0 ? '+' : ''}$${fmt(total)}</span></div>
        <table class="trades">
            <thead><tr><th data-i18n="view.paper.th.ex_date">Ex-date</th><th data-i18n="view.paper.th.sym">Sym</th><th data-i18n="view.paper.th.qty">Qty</th>
            <th data-i18n="view.paper.th.per_share">Per share</th><th data-i18n="view.paper.th.cash">Cash</th></tr></thead>
            <tbody>${rows.map(r => {
                const cash = Number(r.cash_credited);
                return `<tr data-context-scope="symbol-row" data-symbol="${esc(r.symbol)}">
                    <td>${esc(r.ex_date)}</td>
                    <td><a href="#research/${encodeURIComponent(r.symbol)}">${esc(r.symbol)}</a></td>
                    <td>${fmt(r.qty, 0)}</td>
                    <td>$${fmt(r.amount_per_share)}</td>
                    <td class="${cash >= 0 ? 'pos' : 'neg'}">${cash >= 0 ? '+' : ''}$${fmt(cash)}</td>
                </tr>`;
            }).join('')}</tbody></table>`;
}

function renderEquityCurve(hist) {
    const el = document.getElementById('paper-equity-chart');
    const sumEl = document.getElementById('paper-equity-summary');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const snaps = (hist && hist.snapshots) || [];
    if (snaps.length < 2) {
        el.innerHTML = `<div class="muted" data-i18n="view.paper.empty_equity_curve">${esc(t('view.paper.empty_equity_curve'))}</div>`;
        return;
    }
    if (sumEl && hist.summary) {
        const s = hist.summary;
        const b = hist.benchmark;
        const alpha = b && b.summary ? s.return_pct - b.summary.return_pct : null;
        sumEl.innerHTML = `<span class="${s.return_pct >= 0 ? 'pos' : 'neg'}">${s.return_pct >= 0 ? '+' : ''}${s.return_pct.toFixed(2)}%</span>
            · max DD ${s.max_drawdown_pct.toFixed(2)}%${s.currently_underwater ? ' · ' + esc(t('view.paper.label.underwater')) : ''}${
            b && b.summary ? ` · ${esc(b.symbol)} ${b.summary.return_pct >= 0 ? '+' : ''}${b.summary.return_pct.toFixed(2)}% · <strong class="${alpha >= 0 ? 'pos' : 'neg'}">${alpha >= 0 ? '+' : ''}${alpha.toFixed(2)}% vs ${esc(b.symbol)}</strong>` : ''}`;
    }
    const xs = snaps.map(p => Math.floor(new Date(p.taken_at).getTime() / 1000));
    const ys = snaps.map(p => Number(p.equity));
    const series = [
        {},
        { label: t('view.paper.chart.equity'), stroke: '#00e5ff', width: 1.5, points: { show: false } },
    ];
    const data = [xs, ys];
    if (hist.benchmark && hist.benchmark.values.some(v => v != null)) {
        series.push({ label: hist.benchmark.symbol, stroke: '#888', width: 1, dash: [4, 4], points: { show: false } });
        data.push(hist.benchmark.values.map(v => v != null ? v : null));
    }
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        series,
        axes: [
            { stroke: '#aab' },
            { stroke: '#aab', size: 70 },
        ],
        legend: { show: false },
    }, data, el);
}

function renderUnrealizedChart(positions, quotes) {
    const el = document.getElementById('paper-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const pts = (positions || []).map(p => {
        const q = quotes[p.symbol];
        if (!q) return null;
        const u = (Number(q.price) - Number(p.avg_price)) * Number(p.qty);
        return Number.isFinite(u) ? { symbol: p.symbol, u } : null;
    }).filter(Boolean);
    if (pts.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.paper.empty_chart">${esc(t('view.paper.empty_chart'))}</div>`;
        return;
    }
    pts.sort((a, b) => b.u - a.u);
    const labels = pts.map(p => p.symbol);
    const ys = pts.map(p => p.u);
    const xs = labels.map((_, i) => i + 1);
    const zero = xs.map(() => 0);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 220,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.paper.chart.symbol_idx') },
            { label: t('view.paper.chart.unrealized'),
              stroke: '#00e5ff', width: 0,
              points: { show: true, size: 12, fill: '#00e5ff', stroke: '#00e5ff' } },
            { label: t('view.paper.chart.zero'),
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

function renderNotionalChart(positions, quotes) {
    const el = document.getElementById('paper-notional-chart');
    if (!el || !window.uPlot) return;
    el.innerHTML = '';
    const pts = (positions || []).map(p => {
        const q = quotes[p.symbol];
        if (!q) return null;
        const n = Number(q.price) * Number(p.qty);
        return Number.isFinite(n) ? { symbol: p.symbol, n } : null;
    }).filter(Boolean);
    if (pts.length < 1) {
        el.innerHTML = `<div class="muted" data-i18n="view.paper.empty_notional_chart">${esc(t('view.paper.empty_notional_chart'))}</div>`;
        return;
    }
    pts.sort((a, b) => b.n - a.n);
    const labels = pts.map(p => p.symbol);
    const ys = pts.map(p => p.n);
    const xs = labels.map((_, i) => i + 1);
    new window.uPlot({
        title: '', width: el.clientWidth || 600, height: 200,
        scales: { x: { time: false,}, y: { auto: true } },
        series: [
            { label: t('view.paper.chart.symbol_idx') },
            { label: t('view.paper.chart.notional'),
              stroke: '#b86bff', width: 0,
              points: { show: true, size: 12, fill: '#b86bff', stroke: '#b86bff' } },
        ],
        axes: [
            { stroke: '#aab', size: 28,
              splits: () => xs,
              incrs: [1],
              values: (_u, splits) => splits.map(v => labels[Math.round(v) - 1] || '') },
            { stroke: '#aab', size: 60 },
        ],
        legend: { show: true },
    }, [xs, ys], el);
}
