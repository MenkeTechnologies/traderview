// Economic order quantity — Wilson EOQ, order cadence, ordering/holding cost
// split, and reorder point, via /calc/inventory-eoq. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

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
const VIEW = 'inventory-eoq';
let lastReport = null;
let lastBody = null;

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
            <div id="eoq-tools" class="ce-toolbar"></div>
            <button type="button" id="eoq-sens-btn" class="ce-tool" data-i18n="calc.enh.sens.run">▦ Sensitivity</button>
            <div id="eoq-sens" class="ce-sens"></div>
        </div>
        <div id="eoq-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#eoq-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        return body;
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcInventoryEoq(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r, body, tok);
        } catch (err) {
            showToast(err.message || t('view.eoq.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#eoq-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'inventory-eoq.csv' });
    mount.querySelector('#eoq-sens-btn').addEventListener('click', () => runSens(mount, readBody(), tok));
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['eoq_units', r.eoq_units],
        ['reorder_point_units', r.reorder_point_units],
        ['orders_per_year', r.orders_per_year],
        ['days_between_orders', r.days_between_orders],
        ['total_annual_cost_usd', r.total_annual_cost_usd],
        ['feasible', r.feasible],
    ];
}

async function renderResult(mount, r, body, tok) {
    const el = mount.querySelector('#eoq-result');
    // Line chart: EOQ as annual demand sweeps 0.5× → 2× (EOQ ∝ √demand).
    const base = body.annual_demand_units || 1000;
    const xs = enh.linspace(base * 0.5, base * 2, 13);
    const pts = await Promise.all(xs.map(async (d) => {
        const rr = await api.calcInventoryEoq({ ...body, annual_demand_units: d });
        return { x: d, y: rr && rr.feasible ? rr.eoq_units : NaN };
    }));
    if (!viewIsCurrent(tok)) return;
    const chart = enh.svgLineChart(pts, { xlabel: 'demand u', ylabel: 'EOQ u' });
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
            ${r.feasible ? chart : ''}
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

async function runSens(mount, base, tok) {
    const panel = mount.querySelector('#eoq-sens');
    panel.innerHTML = `<p class="muted small" data-i18n="calc.enh.sens.running">Computing…</p>`; applyUiI18n(panel);
    // x: cost per order 0.5× → 2×; y: holding cost / unit 0.5× → 2×. Output: EOQ units.
    const s = base.ordering_cost_per_order_usd || 10;
    const h = base.holding_cost_per_unit_year_usd || 2;
    const xVals = enh.linspace(s * 0.5, s * 2, 5);
    const yVals = enh.linspace(h * 0.5, h * 2, 5);
    const { cells } = await enh.runSensitivity({ base, xKey: 'ordering_cost_per_order_usd', yKey: 'holding_cost_per_unit_year_usd', xVals, yVals, compute: (b) => api.calcInventoryEoq(b), pick: (r) => (r && r.feasible ? r.eoq_units : null) });
    if (!viewIsCurrent(tok)) return;
    panel.innerHTML = enh.renderSensitivityTable({ xVals, yVals, cells, fmt: (v) => (v == null ? '—' : v.toFixed(0)), xfmt: (v) => '$' + v.toFixed(1), yfmt: (v) => '$' + v.toFixed(1), xName: t('view.eoq.label.ordering_cost_per_order_usd') || 'Order $', yName: t('view.eoq.label.holding_cost_per_unit_year_usd') || 'Hold $' });
}
