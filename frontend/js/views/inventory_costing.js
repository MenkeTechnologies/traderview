// Inventory costing FIFO/LIFO/WAC — COGS and ending inventory, via /calc/inventory-costing.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const SEED = [{ q: 100, c: 10 }, { q: 100, c: 12 }, { q: 100, c: 15 }];
function rowHtml(l) {
    return `<div class="mpb-row inv-row">
        <input type="number" step="1" min="0" class="inv-q" value="${l.q}">
        <input type="number" step="0.01" min="0" class="inv-c" value="${l.c}">
        <button type="button" class="inv-del" data-i18n="view.invc.remove">Remove</button></div>`;
}
export async function renderInventoryCosting(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.invc.h1.title">// INVENTORY COSTING</span></h1>
        <p class="muted small" data-i18n="view.invc.hint.intro">Computes cost of goods sold and ending inventory under the three cost-flow assumptions. FIFO expenses the oldest layers first, LIFO the newest, and weighted-average blends all layers. Enter purchase layers oldest-first. Not tax advice.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.invc.h2.inputs">Purchases & sales</h2>
        <form id="invc-form" class="inline-form">
            <label><span data-i18n="view.invc.label.sold">Units sold</span><input type="number" step="1" min="0" name="units_sold" value="150" required></label>
        </form>
        <div class="mpb-head inv-head"><span data-i18n="view.invc.col.q">Quantity</span><span data-i18n="view.invc.col.c">Unit cost ($)</span><span></span></div>
        <div id="invc-rows">${SEED.map(rowHtml).join('')}</div>
        <button type="button" id="invc-add" class="secondary" data-i18n="view.invc.add">+ Add layer</button>
        </div><div id="invc-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#invc-form'); const rowsEl = mount.querySelector('#invc-rows');
    const gen = async () => {
        const layers = [...rowsEl.querySelectorAll('.inv-row')].map((r) => ({ quantity: Number(r.querySelector('.inv-q').value) || 0, unit_cost_usd: Number(r.querySelector('.inv-c').value) || 0 })).filter((l) => l.quantity > 0);
        const body = { layers, units_sold: Number(form.querySelector('[name="units_sold"]').value) || 0 };
        if (!layers.length) { mount.querySelector('#invc-result').innerHTML = ''; return; }
        try { const d = await api.calcInventoryCosting(body); if (!viewIsCurrent(tok)) return; res(mount, d); }
        catch (e) { showToast(e.message || t('view.invc.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 250);
    mount.querySelector('#invc-add').addEventListener('click', () => { rowsEl.insertAdjacentHTML('beforeend', rowHtml({ q: 100, c: 12 })); applyUiI18n(rowsEl.lastElementChild); gen(); });
    rowsEl.addEventListener('click', (e) => { if (e.target.classList.contains('inv-del')) { e.target.closest('.inv-row').remove(); gen(); } });
    form.addEventListener('input', () => live()); rowsEl.addEventListener('input', () => live()); gen();
}
function res(mount, d) {
    const el = mount.querySelector('#invc-result');
    if (!d.valid) { el.innerHTML = `<p class="muted" data-i18n="view.invc.invalid">Units sold cannot exceed units purchased.</p>`; applyUiI18n(el); return; }
    const row = (lbl, m) => `<tr><td>${esc(lbl)}</td><td>${money(m.cogs_usd)}</td><td>${money(m.ending_inventory_usd)}</td></tr>`;
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card"><div class="label" data-i18n="view.invc.card.units">Total units</div><div class="value">${d.total_units}</div></div>
        <div class="card"><div class="label" data-i18n="view.invc.card.cost">Total cost</div><div class="value">${money(d.total_cost_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.invc.card.avg">Avg unit cost</div><div class="value">${money(d.weighted_avg_unit_cost_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.invc.card.end">Ending units</div><div class="value">${d.ending_units}</div></div>
    </div></div>
    <table class="data-table"><thead><tr><th data-i18n="view.invc.th.method">Method</th><th data-i18n="view.invc.th.cogs">COGS</th><th data-i18n="view.invc.th.endinv">Ending inventory</th></tr></thead>
    <tbody>${row('FIFO', d.fifo)}${row('LIFO', d.lifo)}${row(t('view.invc.wac'), d.weighted_average)}</tbody></table>`;
    applyUiI18n(el);
}
