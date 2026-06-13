// Multi-product break-even — weighted-average contribution margin CVP across a
// product mix, via /calc/multi-product-breakeven. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }));
const units = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 1 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');

const SEED = [
    { name: 'A', price: 100, vc: 60, mix: 3 },
    { name: 'B', price: 50, vc: 30, mix: 2 },
];

function rowHtml(p, idx) {
    return `
        <div class="mpb-row" data-idx="${idx}">
            <input type="text" class="mpb-name" placeholder="${t('view.mpbe.ph.name')}" value="${p.name || ''}">
            <input type="number" step="0.01" class="mpb-price" placeholder="${t('view.mpbe.ph.price')}" value="${p.price}">
            <input type="number" step="0.01" class="mpb-vc" placeholder="${t('view.mpbe.ph.vc')}" value="${p.vc}">
            <input type="number" step="0.1" class="mpb-mix" placeholder="${t('view.mpbe.ph.mix')}" value="${p.mix}">
            <button type="button" class="mpb-del" data-i18n="view.mpbe.remove">Remove</button>
        </div>`;
}

export async function renderMultiProductBreakeven(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.mpbe.h1.title">// MULTI-PRODUCT BREAK-EVEN</span></h1>
        <p class="muted small" data-i18n="view.mpbe.hint.intro">
            When a business sells several products in a fixed mix, the break-even point depends on the
            blended margin. Each product contributes price minus variable cost per unit; weighting those
            by the unit sales mix gives the weighted-average contribution margin, and fixed costs over
            that is the total break-even volume. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.mpbe.h2.inputs">The product mix</h2>
            <form id="mpbe-form" class="inline-form">
                <label><span data-i18n="view.mpbe.label.fixed">Fixed costs ($)</span>
                    <input type="number" step="1000" min="0" name="fixed_costs_usd" value="60000" required></label>
            </form>
            <div class="mpb-head">
                <span data-i18n="view.mpbe.col.name">Product</span>
                <span data-i18n="view.mpbe.col.price">Price</span>
                <span data-i18n="view.mpbe.col.vc">Var. cost</span>
                <span data-i18n="view.mpbe.col.mix">Mix units</span>
                <span></span>
            </div>
            <div id="mpbe-rows">${SEED.map(rowHtml).join('')}</div>
            <button type="button" id="mpbe-add" class="secondary" data-i18n="view.mpbe.add">+ Add product</button>
        </div>
        <div id="mpbe-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const rowsEl = mount.querySelector('#mpbe-rows');
    const fixedInput = mount.querySelector('input[name="fixed_costs_usd"]');

    const generate = async () => {
        const products = [...rowsEl.querySelectorAll('.mpb-row')].map((row) => ({
            name: row.querySelector('.mpb-name').value || '',
            price_usd: Number(row.querySelector('.mpb-price').value) || 0,
            variable_cost_usd: Number(row.querySelector('.mpb-vc').value) || 0,
            mix_units: Number(row.querySelector('.mpb-mix').value) || 0,
        })).filter((p) => p.mix_units > 0);
        const body = { fixed_costs_usd: Number(fixedInput.value) || 0, products };
        if (!products.length) {
            mount.querySelector('#mpbe-result').innerHTML = '';
            return;
        }
        try {
            const r = await api.calcMultiProductBreakeven(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.mpbe.toast.error'), { level: 'error' });
        }
    };
    const debounced = debounce(generate, 250);

    mount.addEventListener('input', debounced);
    mount.querySelector('#mpbe-add').addEventListener('click', () => {
        rowsEl.insertAdjacentHTML('beforeend', rowHtml({ name: '', price: '', vc: '', mix: '' }, rowsEl.children.length));
        applyUiI18n(rowsEl.lastElementChild);
        generate();
    });
    rowsEl.addEventListener('click', (e) => {
        if (e.target.classList.contains('mpb-del')) {
            e.target.closest('.mpb-row').remove();
            generate();
        }
    });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#mpbe-result');
    if (!r.is_feasible) {
        el.innerHTML = `<div class="chart-panel"><p class="neg" data-i18n="view.mpbe.infeasible">No break-even — the blended contribution margin is not positive.</p></div>`;
        applyUiI18n(el);
        return;
    }
    const rows = r.products.map((p) => `
        <tr>
            <td>${p.name || '—'}</td>
            <td>${money(p.contribution_margin_usd)}</td>
            <td>${pct(p.mix_proportion_pct)}</td>
            <td>${units(p.breakeven_units)}</td>
            <td>${money(p.breakeven_revenue_usd)}</td>
        </tr>`).join('');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.mpbe.h2.result">The break-even</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.mpbe.card.units">Break-even units</div>
                    <div class="value pos">${units(r.breakeven_units_total)}</div></div>
                <div class="card"><div class="label" data-i18n="view.mpbe.card.revenue">Break-even revenue</div>
                    <div class="value">${money(r.breakeven_revenue_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.mpbe.card.wacm">Weighted-avg CM</div>
                    <div class="value">${money(r.weighted_avg_cm_usd)}</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.mpbe.row.wacm">Weighted-average CM / unit</td><td>${money(r.weighted_avg_cm_usd)}</td></tr>
                    <tr><td data-i18n="view.mpbe.row.wap">Weighted-average price</td><td>${money(r.weighted_avg_price_usd)}</td></tr>
                    <tr><td data-i18n="view.mpbe.row.ratio">Weighted CM ratio</td><td>${pct(r.weighted_cm_ratio_pct)}</td></tr>
                    <tr class="emph"><td data-i18n="view.mpbe.row.units">Total break-even units</td><td>${units(r.breakeven_units_total)}</td></tr>
                    <tr class="emph"><td data-i18n="view.mpbe.row.revenue">Break-even revenue</td><td>${money(r.breakeven_revenue_usd)}</td></tr>
                </tbody>
            </table>
            <table class="data-table" style="margin-top:10px">
                <thead><tr>
                    <th data-i18n="view.mpbe.th.name">Product</th>
                    <th data-i18n="view.mpbe.th.cm">CM/unit</th>
                    <th data-i18n="view.mpbe.th.mix">Mix</th>
                    <th data-i18n="view.mpbe.th.units">BE units</th>
                    <th data-i18n="view.mpbe.th.revenue">BE revenue</th>
                </tr></thead>
                <tbody>${rows}</tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
