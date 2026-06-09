// Multi-broker position aggregation + kill-switch.
// Reads positions across every linked broker (alpaca/tradier wired;
// ibkr/schwab/tastytrade pending) and aggregates by symbol.
// Kill-switch is a destructive cancel-all-orders + close-all-positions
// trigger gated behind an explicit confirm token typed by the user.

import { api } from '../api.js';
import { esc, fmtDateTime } from '../util.js';
import { tConfirm } from '../dialog.js';
import { t } from '../i18n.js';

export async function renderMultiBroker(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.multi_broker.title">// MULTI-BROKER POSITIONS</span></h1>
        <p class="muted small" data-i18n-html="view.multi_broker.intro">
            Fans out positions calls across every linked broker, normalises into one
            row per (symbol, broker, account), and aggregates totals by symbol. Brokers
            without stored credentials are skipped silently; brokers whose call fails
            contribute zero positions and surface an error row. Currently wired:
            <code>alpaca</code> + <code>tradier</code>; <code>ibkr</code> /
            <code>schwab</code> / <code>tastytrade</code> add in follow-up commits.
            Below the totals table is a <strong>KILL-SWITCH</strong> that cancels every
            open order + closes every position across all linked brokers — destructive,
            gated by a typed confirmation token.
        </p>
        <div class="chart-panel">
            <div class="mb-controls" style="display:flex;gap:12px;align-items:center;flex-wrap:wrap;margin-bottom:8px">
                <button class="btn btn-sm primary" id="mb-refresh" data-shortcut="r" data-i18n="common.btn.refresh">⟳ Refresh</button>
                <span class="muted small" id="mb-meta"></span>
            </div>
            <h2 data-i18n="view.multi_broker.h2.totals">Totals by Symbol</h2>
            <table class="trades" id="mb-totals">
                <thead><tr>
                    <th data-i18n="view.multi_broker.th.symbol">Symbol</th>
                    <th data-i18n="view.multi_broker.th.qty">Total Qty</th>
                    <th data-i18n="view.multi_broker.th.mkt_value">Market Value</th>
                    <th data-i18n="view.multi_broker.th.unrl_pl">Unrealized P/L</th>
                    <th data-i18n="view.multi_broker.th.broker_count">Brokers</th>
                    <th data-i18n="view.multi_broker.th.broker_list">Per-Broker</th>
                </tr></thead>
                <tbody><tr><td colspan="6" class="muted" data-i18n="common.loading">loading…</td></tr></tbody>
            </table>
            <h2 data-i18n="view.multi_broker.h2.positions" style="margin-top:1rem">Per-Broker Detail</h2>
            <table class="trades" id="mb-detail">
                <thead><tr>
                    <th data-i18n="view.multi_broker.th.symbol">Symbol</th>
                    <th data-i18n="view.multi_broker.th.broker">Broker</th>
                    <th data-i18n="view.multi_broker.th.account">Account</th>
                    <th data-i18n="view.multi_broker.th.qty">Qty</th>
                    <th data-i18n="view.multi_broker.th.avg_cost">Avg Cost</th>
                    <th data-i18n="view.multi_broker.th.cur_price">Cur Price</th>
                    <th data-i18n="view.multi_broker.th.mkt_value">Market Value</th>
                    <th data-i18n="view.multi_broker.th.unrl_pl">Unrealized P/L</th>
                </tr></thead>
                <tbody><tr><td colspan="8" class="muted" data-i18n="common.loading">loading…</td></tr></tbody>
            </table>
            <div id="mb-errors" style="margin-top:1rem"></div>
        </div>
        <div class="chart-panel" style="margin-top:1rem;border:2px solid var(--red, #ff3860)">
            <h2 style="color:var(--red, #ff3860)" data-i18n="view.multi_broker.h2.kill_switch">🚨 KILL-SWITCH</h2>
            <p class="muted small" data-i18n-html="view.multi_broker.kill.intro">
                Cancels every open order and closes every position across all linked
                brokers. <strong>Destructive — no undo.</strong> Requires typing
                <code>KILL-ALL-NOW</code> in the confirmation dialog. Use only when
                you genuinely want everything flat right now (model malfunction,
                risk-gate trip, emergency exit).
            </p>
            <button class="btn primary" id="mb-kill" style="background:var(--red, #ff3860)" data-i18n="view.multi_broker.btn.kill">🚨 Trigger Kill-Switch</button>
            <div id="mb-kill-result" style="margin-top:1rem"></div>
        </div>
    `;
    mount.querySelector('#mb-refresh').addEventListener('click', () => fetchAndRender(mount));
    mount.querySelector('#mb-kill').addEventListener('click', () => triggerKill(mount));
    fetchAndRender(mount);
}

async function fetchAndRender(mount) {
    const totalsTbody = mount.querySelector('#mb-totals tbody');
    const detailTbody = mount.querySelector('#mb-detail tbody');
    const meta = mount.querySelector('#mb-meta');
    const errBox = mount.querySelector('#mb-errors');
    if (!totalsTbody || !detailTbody) return;
    totalsTbody.innerHTML = `<tr><td colspan="6" class="muted">${esc(t('common.loading'))}</td></tr>`;
    detailTbody.innerHTML = `<tr><td colspan="8" class="muted">${esc(t('common.loading'))}</td></tr>`;
    errBox.innerHTML = '';
    try {
        const r = await api('/multi-broker/positions');
        if (meta) meta.textContent = t('view.multi_broker.meta.summary')
            .replace('{n}', r.positions.length)
            .replace('{b}', r.broker_count);
        // Totals
        if (!r.totals.length) {
            totalsTbody.innerHTML = `<tr><td colspan="6" class="muted">${esc(t('view.multi_broker.empty.no_positions'))}</td></tr>`;
        } else {
            totalsTbody.innerHTML = r.totals.map(s => {
                const plCls = s.total_unrealized_pl >= 0 ? 'pos' : 'neg';
                return `<tr data-context-scope="symbol-row" data-symbol="${esc(s.symbol)}">
                    <td><strong style="color:var(--accent)">${esc(s.symbol)}</strong></td>
                    <td>${s.total_qty.toFixed(2)}</td>
                    <td>${fmtDollar(s.total_market_value)}</td>
                    <td class="${plCls}">${fmtDollar(s.total_unrealized_pl)}</td>
                    <td>${s.broker_count}</td>
                    <td class="muted small">${s.brokers.map(esc).join(', ')}</td>
                </tr>`;
            }).join('');
        }
        // Detail
        if (!r.positions.length) {
            detailTbody.innerHTML = `<tr><td colspan="8" class="muted">${esc(t('view.multi_broker.empty.no_positions'))}</td></tr>`;
        } else {
            detailTbody.innerHTML = r.positions.map(p => {
                const plCls = (p.unrealized_pl || 0) >= 0 ? 'pos' : 'neg';
                return `<tr data-context-scope="symbol-row" data-symbol="${esc(p.symbol)}">
                    <td><strong style="color:var(--accent)">${esc(p.symbol)}</strong></td>
                    <td class="muted small">${esc(p.broker)}</td>
                    <td class="muted small">${esc(p.account_label || '—')}</td>
                    <td>${p.qty.toFixed(2)}</td>
                    <td>${p.avg_cost == null ? '—' : fmtDollar(p.avg_cost)}</td>
                    <td>${p.current_price == null ? '—' : fmtDollar(p.current_price)}</td>
                    <td>${p.market_value == null ? '—' : fmtDollar(p.market_value)}</td>
                    <td class="${plCls}">${p.unrealized_pl == null ? '—' : fmtDollar(p.unrealized_pl)}</td>
                </tr>`;
            }).join('');
        }
        // Errors
        if (r.errors && r.errors.length) {
            errBox.innerHTML = `<h3 class="neg">${esc(t('view.multi_broker.h3.errors'))}</h3>
                <ul>${r.errors.map(e =>
                    `<li class="muted small"><strong>${esc(e.broker)}</strong>: ${esc(e.message)}</li>`
                ).join('')}</ul>`;
        }
    } catch (e) {
        totalsTbody.innerHTML = `<tr><td colspan="6" class="muted">${esc(String(e))}</td></tr>`;
        detailTbody.innerHTML = '';
    }
}

async function triggerKill(mount) {
    const result = mount.querySelector('#mb-kill-result');
    if (!result) return;
    let token = null;
    try {
        token = await tConfirm({
            title: t('view.multi_broker.kill.dialog.title'),
            message: t('view.multi_broker.kill.dialog.message'),
            level: 'danger',
            confirmText: t('view.multi_broker.kill.dialog.confirm'),
            cancelText: t('view.multi_broker.kill.dialog.cancel'),
            input: {
                placeholder: 'KILL-ALL-NOW',
                required: true,
            },
        });
    } catch (_) {
        return;
    }
    if (!token || token !== 'KILL-ALL-NOW') {
        result.innerHTML = `<p class="muted small">${esc(t('view.multi_broker.kill.cancelled'))}</p>`;
        return;
    }
    result.innerHTML = `<p class="muted small">${esc(t('view.multi_broker.kill.firing'))}</p>`;
    try {
        const r = await api('/multi-broker/kill-switch', {
            method: 'POST',
            body: { confirm_token: token },
        });
        result.innerHTML = `<div class="neg">
            <strong>${esc(t('view.multi_broker.kill.result_title'))}</strong>
            <ul>
                <li>${esc(t('view.multi_broker.kill.cancelled_count'))}: ${r.cancelled_orders}</li>
                <li>${esc(t('view.multi_broker.kill.closed_count'))}: ${r.closed_positions}</li>
            </ul>
            ${r.note ? `<p class="muted small">${esc(r.note)}</p>` : ''}
            ${r.errors && r.errors.length ? `<ul>${r.errors.map(e =>
                `<li class="muted small">${esc(e.broker)}: ${esc(e.message)}</li>`).join('')}</ul>` : ''}
        </div>`;
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}

function fmtDollar(n) {
    if (n == null) return '—';
    const abs = Math.abs(n);
    const sign = n < 0 ? '-' : '';
    if (abs >= 1_000_000) return `${sign}$${(abs / 1_000_000).toFixed(2)}M`;
    if (abs >= 1_000) return `${sign}$${(abs / 1_000).toFixed(2)}K`;
    return `${sign}$${abs.toFixed(2)}`;
}
