// Wash Sale Tracker — IRC § 1091.
// Loss disallowed if substantially identical security bought within
// 30 days before OR after the loss sale. Disallowed loss adds to basis
// of replacement shares (holding period continues from original).
// Active day-traders accumulate wash sales fast — TTS + § 475(f) MTM
// election EXEMPTS you from wash-sale rules entirely.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-wash-sales-v1';
const WINDOW_DAYS = 30;

function load() {
    try { return JSON.parse(localStorage.getItem(LS_KEY) || '[]'); }
    catch { return []; }
}
function save(trades) { try { localStorage.setItem(LS_KEY, JSON.stringify(trades)); } catch { /* ignore */ } }

let state = { trades: load() };

export async function renderWashSaleTracker(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.wash.h1.title">// WASH SALE TRACKER</span></h1>
        <p class="muted small" data-i18n="view.wash.hint.intro">
            IRC § 1091 — loss disallowed if you buy the same (or substantially identical)
            security within 30 days before or after the loss sale. Disallowed loss adds to
            basis of the replacement shares. <strong>TTS + § 475(f) election EXEMPTS you</strong>
            from wash sales entirely — major reason traders elect MTM.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.wash.h2.add">Add trade</h2>
            <form id="ws-form" class="inline-form">
                <label><span data-i18n="view.wash.label.date">Date</span>
                    <input type="date" name="date" required value="${new Date().toISOString().slice(0,10)}"></label>
                <label><span data-i18n="view.wash.label.symbol">Symbol</span>
                    <input type="text" name="symbol" placeholder="NVDA" required></label>
                <label><span data-i18n="view.wash.label.side">Side</span>
                    <select name="side">
                        <option value="buy">buy</option>
                        <option value="sell" selected>sell</option>
                    </select>
                </label>
                <label><span data-i18n="view.wash.label.qty">Quantity</span>
                    <input type="number" step="1" name="qty" required></label>
                <label><span data-i18n="view.wash.label.proceeds_or_cost">Proceeds (sell) / Cost (buy)</span>
                    <input type="number" step="0.01" name="proceeds_or_cost" required></label>
                <label><span data-i18n="view.wash.label.basis_sold">Basis (sells only)</span>
                    <input type="number" step="0.01" name="basis"></label>
                <button class="primary" type="submit" data-i18n="view.wash.btn.add">Add</button>
            </form>
        </div>
        <div id="ws-summary"></div>
        <div id="ws-table" class="chart-panel"></div>
    `;
    document.getElementById('ws-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        const tr = {
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            date: fd.get('date'),
            symbol: (fd.get('symbol') || '').toUpperCase(),
            side: fd.get('side'),
            qty: Number(fd.get('qty')),
            proceeds_or_cost: Number(fd.get('proceeds_or_cost')),
            basis: Number(fd.get('basis')) || 0,
        };
        state.trades.push(tr);
        save(state.trades);
        e.target.reset();
        e.target.querySelector('[name="date"]').value = new Date().toISOString().slice(0, 10);
        showToast(t('view.wash.toast.added'), { level: 'success' });
        render();
    });
    render();
}

function detectWashSales() {
    // Group by symbol.
    const bySymbol = new Map();
    for (const t of state.trades) {
        if (!bySymbol.has(t.symbol)) bySymbol.set(t.symbol, []);
        bySymbol.get(t.symbol).push(t);
    }
    const flagged = [];
    for (const [sym, trades] of bySymbol) {
        const sortedByDate = [...trades].sort((a, b) =>
            String(a.date).localeCompare(String(b.date)));
        for (const sell of sortedByDate.filter(t => t.side === 'sell')) {
            const loss = sell.basis - sell.proceeds_or_cost;
            if (loss <= 0) continue;  // gain, not a wash candidate
            const sellMs = new Date(sell.date).getTime();
            // Look for buys within ±30 days.
            const replacements = sortedByDate.filter(t =>
                t.side === 'buy' && t.id !== sell.id &&
                Math.abs(new Date(t.date).getTime() - sellMs) <= WINDOW_DAYS * 86_400_000);
            if (!replacements.length) continue;
            // Naive: assume FIFO replacement; disallowed loss = loss × (replacement qty / sell qty), capped at loss.
            const totalReplaceQty = replacements.reduce((s, r) => s + r.qty, 0);
            const ratio = Math.min(1, totalReplaceQty / sell.qty);
            const disallowed = loss * ratio;
            flagged.push({
                ...sell,
                loss,
                disallowed,
                allowed: loss - disallowed,
                replacement_count: replacements.length,
                first_replacement: replacements[0].date,
            });
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
    const el = document.getElementById('ws-summary');
    if (!el) return;
    const totalLosses = flagged.reduce((s, f) => s + f.loss, 0);
    const totalDisallowed = flagged.reduce((s, f) => s + f.disallowed, 0);
    const totalAllowed = flagged.reduce((s, f) => s + f.allowed, 0);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.wash.h2.summary">Summary</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.wash.card.trades">Trades tracked</div>
                    <div class="value">${state.trades.length}</div>
                </div>
                <div class="card ${flagged.length ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.wash.card.flagged">Wash sales flagged</div>
                    <div class="value">${flagged.length}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.wash.card.losses">Loss-side total</div>
                    <div class="value">$${totalLosses.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.wash.card.disallowed">Disallowed loss</div>
                    <div class="value">$${totalDisallowed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.wash.card.allowed">Allowed loss</div>
                    <div class="value">$${totalAllowed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}

function renderTable(flagged) {
    const el = document.getElementById('ws-table');
    if (!el) return;
    if (!flagged.length) {
        el.innerHTML = `<h2 data-i18n="view.wash.h2.flagged">Wash sales flagged</h2>
            <p class="muted" data-i18n="view.wash.empty">No wash sales detected.</p>`;
        return;
    }
    const sorted = [...flagged].sort((a, b) => b.disallowed - a.disallowed);
    el.innerHTML = `
        <h2 data-i18n="view.wash.h2.flagged">Wash sales flagged</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.wash.th.date">Sale date</th>
                <th data-i18n="view.wash.th.symbol">Symbol</th>
                <th data-i18n="view.wash.th.qty">Qty</th>
                <th data-i18n="view.wash.th.loss">Loss</th>
                <th data-i18n="view.wash.th.disallowed">Disallowed</th>
                <th data-i18n="view.wash.th.allowed">Allowed</th>
                <th data-i18n="view.wash.th.replacements"># replacements</th>
                <th data-i18n="view.wash.th.first_rep">First rep</th>
            </tr></thead>
            <tbody>${sorted.map(f => `
                <tr>
                    <td>${esc(f.date)}</td>
                    <td>${esc(f.symbol)}</td>
                    <td>${f.qty}</td>
                    <td class="neg">$${f.loss.toFixed(2)}</td>
                    <td class="neg">$${f.disallowed.toFixed(2)}</td>
                    <td class="pos">$${f.allowed.toFixed(2)}</td>
                    <td>${f.replacement_count}</td>
                    <td class="muted">${esc(f.first_replacement)}</td>
                </tr>
            `).join('')}</tbody>
        </table>
    `;
}
