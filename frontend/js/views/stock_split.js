// Stock-split position adjuster — scale shares, price, and basis by a new:old
// ratio (forward or reverse), via /calc/stock-split. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const sh = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 4 });

export async function renderStockSplit(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.split.h1.title">// STOCK SPLIT ADJUSTER</span></h1>
        <p class="muted small" data-i18n="view.split.hint.intro">
            A split changes your share count and per-share figures but not the total value or cost
            basis. Enter the ratio as new:old — 4:1 for a 4-for-1 forward split, 1:10 for a 1-for-10
            reverse split. Any leftover fractional share is usually paid out as cash in lieu.
            Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.split.h2.inputs">The position</h2>
            <form id="split-form" class="inline-form">
                <label><span data-i18n="view.split.label.shares">Shares held</span>
                    <input type="number" step="0.0001" min="0" name="shares" value="100" required></label>
                <label><span data-i18n="view.split.label.basis">Cost basis / share ($)</span>
                    <input type="number" step="0.01" min="0" name="cost_basis_per_share" value="50" required></label>
                <label><span data-i18n="view.split.label.price">Price / share ($)</span>
                    <input type="number" step="0.01" min="0" name="price_per_share" value="80" required></label>
                <label><span data-i18n="view.split.label.new">Ratio — new</span>
                    <input type="number" step="0.0001" min="0" name="split_new" value="2" required></label>
                <label><span data-i18n="view.split.label.old">Ratio — old</span>
                    <input type="number" step="0.0001" min="0" name="split_old" value="1" required></label>
            </form>
        </div>
        <div id="split-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#split-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            shares: Number(fd.get('shares')) || 0,
            cost_basis_per_share: Number(fd.get('cost_basis_per_share')) || 0,
            price_per_share: Number(fd.get('price_per_share')) || 0,
            split_new: Number(fd.get('split_new')) || 0,
            split_old: Number(fd.get('split_old')) || 0,
        };
        try {
            const r = await api.calcStockSplit(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.split.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#split-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.split.h2.result">After the split</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.split.card.shares">Shares</div>
                    <div class="value pos">${sh(r.post_shares)}</div></div>
                <div class="card"><div class="label" data-i18n="view.split.card.price">Price / share</div>
                    <div class="value">${money(r.post_price_per_share)}</div></div>
                <div class="card"><div class="label">${t('view.split.card.factor')}</div>
                    <div class="value">${sh(r.factor)}×</div></div>
            </div>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.split.row.basis">Cost basis / share</td><td>${money(r.post_basis_per_share)}</td></tr>
                    <tr><td data-i18n="view.split.row.whole">Whole shares</td><td>${sh(r.whole_shares)}</td></tr>
                    <tr><td data-i18n="view.split.row.fractional">Fractional share</td><td>${sh(r.fractional_shares)}</td></tr>
                    <tr><td data-i18n="view.split.row.cash">Cash in lieu</td><td>${money(r.cash_in_lieu)}</td></tr>
                    <tr class="emph"><td data-i18n="view.split.row.value">Total value (unchanged)</td><td>${money(r.total_value)}</td></tr>
                    <tr class="emph"><td data-i18n="view.split.row.cost">Total cost (unchanged)</td><td>${money(r.total_cost)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
