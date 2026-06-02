// Cross-Broker Wash Sale Tracker.
// Wash sales are taxpayer-level, NOT account-level. Loss in IBKR + buy in
// Webull within ±30 days = STILL a wash sale, but neither broker's 1099-B
// catches it. Reg. § 1.1091 — substantially identical security. Spousal +
// IRA accounts also count. Log all sells/buys across brokers, auto-detect.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-cross-broker-wash-v1';
const WINDOW_DAYS = 30;

const BROKERS = ['IBKR', 'Webull', 'Schwab', 'Fidelity', 'Robinhood', 'TastyTrade', 'TradeStation', 'E*TRADE', 'Other'];

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(rows) { try { localStorage.setItem(LS_KEY, JSON.stringify(rows)); } catch { /* ignore */ } }

let state = { trades: load() };

export async function renderCrossBrokerWash(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.cbwash.h1.title">// CROSS-BROKER WASH SALES</span></h1>
        <p class="muted small" data-i18n="view.cbwash.hint.intro">
            Wash sale rules apply at the TAXPAYER level, not account level. Sell at
            IBKR + repurchase at Webull within ±30 days = wash sale. <strong>Neither
            broker's 1099-B catches this</strong> — you must self-report. Same applies
            across spousal accounts and IRAs. Log all sells + buys here, auto-detect.
            TTS + § 475(f) MTM EXEMPTS you from wash-sale rules entirely.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.cbwash.h2.add">Log trade</h2>
            <form id="cbw-form" class="inline-form">
                <label><span data-i18n="view.cbwash.label.date">Date</span>
                    <input type="date" name="date" required value="${new Date().toISOString().slice(0,10)}"></label>
                <label><span data-i18n="view.cbwash.label.broker">Broker</span>
                    <select name="broker">${BROKERS.map(b => `<option value="${esc(b)}">${esc(b)}</option>`).join('')}</select>
                </label>
                <label><span data-i18n="view.cbwash.label.symbol">Symbol</span>
                    <input type="text" name="symbol" required></label>
                <label><span data-i18n="view.cbwash.label.side">Side</span>
                    <select name="side">
                        <option value="buy">buy</option>
                        <option value="sell" selected>sell</option>
                    </select>
                </label>
                <label><span data-i18n="view.cbwash.label.qty">Quantity</span>
                    <input type="number" step="1" name="qty" required></label>
                <label><span data-i18n="view.cbwash.label.price">Price / share</span>
                    <input type="number" step="0.01" name="price" required></label>
                <label><span data-i18n="view.cbwash.label.basis">Basis / share (sells only)</span>
                    <input type="number" step="0.01" name="basis_per_share"></label>
                <button class="primary" type="submit" data-i18n="view.cbwash.btn.add">Add</button>
            </form>
        </div>
        <div id="cbw-summary"></div>
        <div id="cbw-table" class="chart-panel"></div>
    `;
    document.getElementById('cbw-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const tr = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            date: fd.get('date'),
            broker: fd.get('broker'),
            symbol: (fd.get('symbol') || '').toUpperCase(),
            side: fd.get('side'),
            qty: Number(fd.get('qty')),
            price: Number(fd.get('price')),
            basis_per_share: Number(fd.get('basis_per_share')) || 0,
        };
        state.trades.push(tr);
        save(state.trades);
        e.target.reset();
        e.target.querySelector('[name="date"]').value = new Date().toISOString().slice(0, 10);
        showToast(t('view.cbwash.toast.added'), { level: 'success' });
        render();
    });
    render();
}

function detectWashSales() {
    const bySymbol = new Map();
    for (const t of state.trades) {
        if (!bySymbol.has(t.symbol)) bySymbol.set(t.symbol, []);
        bySymbol.get(t.symbol).push(t);
    }
    const flagged = [];
    for (const [sym, trades] of bySymbol) {
        const sells = trades.filter(t => t.side === 'sell');
        for (const sell of sells) {
            const loss = (sell.basis_per_share - sell.price) * sell.qty;
            if (loss <= 0) continue;
            const sellMs = new Date(sell.date).getTime();
            const replacements = trades.filter(t =>
                t.side === 'buy' && t.id !== sell.id &&
                Math.abs(new Date(t.date).getTime() - sellMs) <= WINDOW_DAYS * 86_400_000);
            if (!replacements.length) continue;
            const totalReplaceQty = replacements.reduce((s, r) => s + r.qty, 0);
            const ratio = Math.min(1, totalReplaceQty / sell.qty);
            const disallowed = loss * ratio;
            const crossBroker = new Set(replacements.map(r => r.broker)).has(sell.broker) === false
                || replacements.some(r => r.broker !== sell.broker);
            flagged.push({ ...sell, loss, disallowed, allowed: loss - disallowed,
                replacement_count: replacements.length,
                cross_broker: crossBroker,
                replacements });
        }
    }
    return flagged;
}

function render() {
    const flagged = detectWashSales();
    renderSummary(flagged);
    renderTable(flagged);
}

function renderSummary(flagged) {
    const el = document.getElementById('cbw-summary');
    if (!el) return;
    const totalDisallowed = flagged.reduce((s, f) => s + f.disallowed, 0);
    const totalAllowed = flagged.reduce((s, f) => s + f.allowed, 0);
    const crossBrokerCount = flagged.filter(f => f.cross_broker).length;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.cbwash.h2.summary">Summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.cbwash.card.trades">Total trades</div>
                    <div class="value">${state.trades.length}</div>
                </div>
                <div class="card ${flagged.length ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.cbwash.card.flagged">Wash sales flagged</div>
                    <div class="value">${flagged.length}</div>
                </div>
                <div class="card ${crossBrokerCount ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.cbwash.card.cross_broker">Cross-broker (broker won't catch)</div>
                    <div class="value">${crossBrokerCount}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.cbwash.card.disallowed">Disallowed loss total</div>
                    <div class="value">$${totalDisallowed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.cbwash.card.allowed">Allowed loss total</div>
                    <div class="value">$${totalAllowed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable(flagged) {
    const el = document.getElementById('cbw-table');
    if (!el) return;
    if (!flagged.length) {
        el.innerHTML = `<h2 data-i18n="view.cbwash.h2.flagged">Flagged wash sales</h2>
            <p class="muted" data-i18n="view.cbwash.empty">No wash sales detected.</p>`;
        return;
    }
    const sorted = [...flagged].sort((a, b) => b.disallowed - a.disallowed);
    el.innerHTML = `
        <h2 data-i18n="view.cbwash.h2.flagged">Flagged wash sales</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.cbwash.th.date">Sale date</th>
                <th data-i18n="view.cbwash.th.broker">Broker</th>
                <th data-i18n="view.cbwash.th.symbol">Symbol</th>
                <th data-i18n="view.cbwash.th.qty">Qty</th>
                <th data-i18n="view.cbwash.th.loss">Loss</th>
                <th data-i18n="view.cbwash.th.disallowed">Disallowed</th>
                <th data-i18n="view.cbwash.th.allowed">Allowed</th>
                <th data-i18n="view.cbwash.th.cross_broker">Cross-broker?</th>
                <th data-i18n="view.cbwash.th.replacements">Replacements</th>
            </tr></thead>
            <tbody>${sorted.map(f => `
                <tr>
                    <td>${esc(f.date)}</td>
                    <td>${esc(f.broker)}</td>
                    <td>${esc(f.symbol)}</td>
                    <td>${f.qty}</td>
                    <td class="neg">$${f.loss.toFixed(2)}</td>
                    <td class="neg">$${f.disallowed.toFixed(2)}</td>
                    <td class="pos">$${f.allowed.toFixed(2)}</td>
                    <td class="${f.cross_broker ? 'neg' : 'muted'}">${f.cross_broker ? '⚠ YES' : 'no'}</td>
                    <td class="muted">${f.replacements.map(r => `${r.broker}(${r.qty}@$${r.price})`).join('; ')}</td>
                </tr>
            `).join('')}</tbody>
        </table>
    `;
}
