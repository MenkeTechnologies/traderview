// Economic order quantity — Wilson EOQ, order cadence, ordering/holding cost
// split, and reorder point, via /calc/inventory-eoq. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const FIELDS = [
    ['annual_demand_units', 'Annual demand (units)', 1000],
    ['ordering_cost_per_order_usd', 'Cost per order ($)', 10],
    ['holding_cost_per_unit_year_usd', 'Holding cost / unit / yr ($)', 2],
    ['lead_time_days', 'Lead time (days)', 7],
    ['safety_stock_units', 'Safety stock (units)', 0],
    ['period_days', 'Demand period (days)', 365],
];

const num = (n, d = 1) => Number(n).toLocaleString(undefined, { maximumFractionDigits: d });
const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 });

export async function renderInventoryEoq(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.eoq.h1.title">// ECONOMIC ORDER QUANTITY</span></h1>
        <p class="muted small" data-i18n="view.eoq.hint.intro">
            How much to order at a time, and when to reorder, to minimize total inventory cost.
            Big batches cut ordering cost but raise holding cost; EOQ = √(2·D·S/H) is the batch
            size where the two balance (and are equal). The reorder point — daily demand × lead
            time + safety stock — is the on-hand level that triggers the next order so it arrives
            before you run out. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.eoq.h2.inputs">The item</h2>
            <form id="eoq-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.eoq.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="eoq-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#eoq-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcInventoryEoq(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.eoq.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#eoq-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.eoq.h2.result">The plan</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.eoq.card.eoq">Order quantity (EOQ)</div>
                    <div class="value">${r.feasible ? num(r.eoq_units) : '—'}</div></div>
                <div class="card"><div class="label" data-i18n="view.eoq.card.rop">Reorder point</div>
                    <div class="value">${num(r.reorder_point_units)}</div></div>
                <div class="card"><div class="label" data-i18n="view.eoq.card.orders">Orders / year</div>
                    <div class="value">${r.feasible ? num(r.orders_per_year, 2) : '—'}</div></div>
                <div class="card"><div class="label" data-i18n="view.eoq.card.spacing">Days between orders</div>
                    <div class="value">${r.feasible ? num(r.days_between_orders) : '—'}</div></div>
                <div class="card"><div class="label" data-i18n="view.eoq.card.total">Total annual cost</div>
                    <div class="value">${r.feasible ? money(r.total_annual_cost_usd) : '—'}</div></div>
            </div>
            ${r.feasible ? `
            <table class="data-table">
                <thead><tr><th data-i18n="view.eoq.col.line">Line</th><th data-i18n="view.eoq.col.amount">Amount</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.eoq.row.ordering">Annual ordering cost</td><td>${money(r.annual_ordering_cost_usd)}</td></tr>
                    <tr><td data-i18n="view.eoq.row.holding">Annual holding cost</td><td>${money(r.annual_holding_cost_usd)}</td></tr>
                    <tr><td data-i18n="view.eoq.row.daily">Daily demand</td><td>${num(r.daily_demand_units, 2)}</td></tr>
                </tbody>
            </table>` : `<p class="muted small neg" data-i18n="view.eoq.warn.infeasible">EOQ needs a positive holding cost and demand. The reorder point still applies.</p>`}
        </div>
    `;
    applyUiI18n(el);
}
