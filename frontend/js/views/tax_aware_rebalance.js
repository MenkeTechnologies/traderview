// Tax-Aware Rebalance — trade toward target weights while choosing sell
// lots to minimize realized gain (or harvest losses). Enter one row per
// tax lot; rows are grouped by symbol (price + target taken from the
// first lot of each symbol), then planned server-side via
// /calc/tax-aware-rebalance.

import { api } from '../api.js';
import { esc } from '../util.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const LS_KEY = 'tv-tax-rebalance-lots-v1';

function load() {
    try {
        const raw = localStorage.getItem(LS_KEY);
        return raw ? JSON.parse(raw) : [];
    } catch { return []; }
}
function save(lots) {
    try { localStorage.setItem(LS_KEY, JSON.stringify(lots)); } catch { /* private mode */ }
}

const state = { lots: load(), strategy: 'hifo', tax_rate: 25, band: 5 };

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });

export async function renderTaxAwareRebalance(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.tax_rebal.h1.title">// TAX-AWARE REBALANCE</span></h1>
        <p class="muted small" data-i18n="view.tax_rebal.hint.intro">
            Rebalancing to a target can realize a large capital gain it never tells you
            about. Enter one row per tax lot; the planner trades toward your targets but
            picks sell lots to minimize the gain (HIFO) or to harvest losses, and reports
            the gain and estimated tax the rebalance triggers. Holdings within the no-trade
            band are left alone — churning them realizes tax for a trivial correction.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.tax_rebal.h2.add_lot">Add tax lot</h2>
            <form id="tr-form" class="inline-form">
                <input name="symbol" placeholder="symbol" data-i18n-placeholder="common.placeholder.symbol" required style="text-transform:uppercase">
                <input name="qty" type="number" step="0.0001" min="0" placeholder="shares" data-i18n-placeholder="view.tax_rebal.ph.shares" required>
                <input name="cost" type="number" step="0.01" min="0" placeholder="cost/share" data-i18n-placeholder="view.tax_rebal.ph.cost" required>
                <input name="price" type="number" step="0.01" min="0" placeholder="price now" data-i18n-placeholder="view.tax_rebal.ph.price" required>
                <input name="target" type="number" step="0.1" min="0" max="100" placeholder="target %" data-i18n-placeholder="view.tax_rebal.ph.target" required>
                <button class="primary" type="submit" data-i18n="view.tax_rebal.btn.add">Add lot</button>
            </form>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.tax_rebal.h2.settings">Plan settings</h2>
            <form id="tr-settings" class="inline-form">
                <label><span data-i18n="view.tax_rebal.label.strategy">Sell-lot strategy</span>
                    <select name="strategy">
                        <option value="hifo" data-i18n="view.tax_rebal.opt.hifo">HIFO — minimize gain</option>
                        <option value="max_loss_harvest" data-i18n="view.tax_rebal.opt.harvest">Harvest losses only</option>
                        <option value="lifoust" data-i18n="view.tax_rebal.opt.lowcost">Lowest-cost — maximize gain</option>
                    </select>
                </label>
                <label><span data-i18n="view.tax_rebal.label.tax_rate">Tax rate %</span>
                    <input type="number" step="0.1" min="0" max="100" name="tax_rate" value="${state.tax_rate}"></label>
                <label><span data-i18n="view.tax_rebal.label.band">No-trade band %</span>
                    <input type="number" step="0.1" min="0" max="50" name="band" value="${state.band}"></label>
                <button class="primary" type="submit" data-i18n="view.tax_rebal.btn.plan">Plan rebalance</button>
            </form>
        </div>
        <div id="tr-result"></div>
        <div id="tr-table" class="chart-panel"></div>
    `;
    applyUiI18n(mount);

    mount.querySelector('#tr-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.lots.push({
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            symbol: (fd.get('symbol') || '').trim().toUpperCase(),
            qty: Number(fd.get('qty')),
            cost: Number(fd.get('cost')),
            price: Number(fd.get('price')),
            target: Number(fd.get('target')),
        });
        save(state.lots);
        e.target.reset();
        renderTable(mount);
    });

    mount.querySelector('#tr-settings').addEventListener('submit', async (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.strategy = fd.get('strategy');
        state.tax_rate = Number(fd.get('tax_rate')) || 0;
        state.band = Number(fd.get('band')) || 0;
        await runPlan(mount, tok);
    });

    renderTable(mount);
}

// Group flat lot rows into the holdings[] the API expects: price and
// target come from the first lot seen for each symbol.
function groupHoldings() {
    const bySym = new Map();
    for (const l of state.lots) {
        if (!bySym.has(l.symbol)) {
            bySym.set(l.symbol, {
                symbol: l.symbol,
                price: l.price,
                target_weight: l.target / 100,
                lots: [],
            });
        }
        bySym.get(l.symbol).lots.push({
            lot_id: l.id,
            qty_open: l.qty,
            cost_per_share: l.cost,
        });
    }
    return [...bySym.values()];
}

async function runPlan(mount, tok) {
    const holdings = groupHoldings();
    if (!holdings.length) {
        showToast(t('view.tax_rebal.toast.no_lots'), { level: 'warning' });
        return;
    }
    try {
        const plan = await api.calcTaxAwareRebalance({
            holdings,
            strategy: state.strategy,
            tax_rate: state.tax_rate / 100,
            band: state.band / 100,
        });
        if (!viewIsCurrent(tok)) return;
        renderResult(mount, plan);
    } catch (err) {
        showToast(err.message || t('view.tax_rebal.toast.error'), { level: 'error' });
    }
}

function renderResult(mount, plan) {
    const el = mount.querySelector('#tr-result');
    const gainCls = Number(plan.total_realized_gain) >= 0 ? 'neg' : 'pos'; // a gain is a tax cost; a loss is a benefit
    const rows = plan.actions.map((a) => {
        const tv = Number(a.trade_value);
        const verb = tv > 0 ? t('view.tax_rebal.act.buy') : tv < 0 ? t('view.tax_rebal.act.sell') : t('view.tax_rebal.act.hold');
        const cls = tv > 0 ? 'pos' : tv < 0 ? 'neg' : 'muted';
        const g = Number(a.realized_gain);
        return `<tr>
            <td>${esc(a.symbol)}</td>
            <td>${money(a.current_value)}</td>
            <td>${money(a.target_value)}</td>
            <td class="${cls}">${verb} ${tv === 0 ? '' : money(Math.abs(tv))}</td>
            <td class="${g > 0 ? 'neg' : g < 0 ? 'pos' : 'muted'}">${g === 0 ? '—' : money(g)}</td>
        </tr>`;
    }).join('');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.tax_rebal.h2.plan">Rebalance plan</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.tax_rebal.card.total">Portfolio value</div>
                    <div class="value">${money(plan.total_value)}</div></div>
                <div class="card"><div class="label" data-i18n="view.tax_rebal.card.gain">Realized gain</div>
                    <div class="value ${gainCls}">${money(plan.total_realized_gain)}</div></div>
                <div class="card"><div class="label" data-i18n="view.tax_rebal.card.tax">Estimated tax</div>
                    <div class="value neg">${money(plan.estimated_tax)}</div></div>
            </div>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.tax_rebal.th.symbol">Symbol</th>
                    <th data-i18n="view.tax_rebal.th.current">Current</th>
                    <th data-i18n="view.tax_rebal.th.target">Target</th>
                    <th data-i18n="view.tax_rebal.th.trade">Trade</th>
                    <th data-i18n="view.tax_rebal.th.gain">Realized gain</th>
                </tr></thead>
                <tbody>${rows}</tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}

function renderTable(mount) {
    const el = mount.querySelector('#tr-table');
    if (!el) return;
    if (!state.lots.length) {
        el.innerHTML = `<h2 data-i18n="view.tax_rebal.h2.lots">Tax lots</h2>
            <p class="muted" data-i18n="view.tax_rebal.empty">No lots yet.</p>`;
        applyUiI18n(el);
        return;
    }
    el.innerHTML = `
        <h2 data-i18n="view.tax_rebal.h2.lots">Tax lots</h2>
        <table class="trades">
            <thead><tr>
                <th data-i18n="view.tax_rebal.th.symbol">Symbol</th>
                <th data-i18n="view.tax_rebal.th.shares">Shares</th>
                <th data-i18n="view.tax_rebal.th.cost">Cost/sh</th>
                <th data-i18n="view.tax_rebal.th.price">Price</th>
                <th data-i18n="view.tax_rebal.th.target">Target %</th>
                <th data-i18n="view.tax_rebal.th.actions">Actions</th>
            </tr></thead>
            <tbody>${state.lots.map((l) => `<tr>
                <td>${esc(l.symbol)}</td>
                <td>${l.qty}</td>
                <td>$${l.cost}</td>
                <td>$${l.price}</td>
                <td>${l.target}%</td>
                <td><button class="link neg" data-del="${esc(l.id)}" data-i18n="view.tax_rebal.btn.delete">delete</button></td>
            </tr>`).join('')}</tbody>
        </table>
    `;
    applyUiI18n(el);
    el.querySelectorAll('[data-del]').forEach((btn) => {
        btn.addEventListener('click', () => {
            state.lots = state.lots.filter((l) => l.id !== btn.dataset.del);
            save(state.lots);
            renderTable(mount);
        });
    });
}
