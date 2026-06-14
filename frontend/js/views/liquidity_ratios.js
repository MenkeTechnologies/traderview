// Business liquidity ratios, via /calc/liquidity-ratios.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const x = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '×');
export async function renderLiquidityRatios(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.liqr.h1.title">// LIQUIDITY RATIOS</span></h1>
        <p class="muted small" data-i18n="view.liqr.hint.intro">The short-term solvency measures from a company balance sheet: net working capital (current assets − current liabilities), the current ratio, the quick / acid-test ratio (excludes inventory), and the cash ratio. Business-entity ratios, not personal finance. Not advice.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.liqr.h2.inputs">Balance sheet</h2>
        <form id="liqr-form" class="inline-form">
            <label><span data-i18n="view.liqr.label.company">Company</span><input type="text" name="company_label" value="Acme"></label>
            <label><span data-i18n="view.liqr.label.ca">Current assets ($)</span><input type="number" step="1000" min="0" name="current_assets_usd" value="200000" required></label>
            <label><span data-i18n="view.liqr.label.inv">Inventory ($)</span><input type="number" step="1000" min="0" name="inventory_usd" value="80000"></label>
            <label><span data-i18n="view.liqr.label.cash">Cash & equivalents ($)</span><input type="number" step="1000" min="0" name="cash_and_equivalents_usd" value="50000"></label>
            <label><span data-i18n="view.liqr.label.cl">Current liabilities ($)</span><input type="number" step="1000" min="0" name="current_liabilities_usd" value="100000" required></label>
        </form></div><div id="liqr-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#liqr-form'); const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const gen = async () => {
        const body = { company_label: (form.querySelector('[name="company_label"]').value || '').trim(), current_assets_usd: n('current_assets_usd'), inventory_usd: n('inventory_usd'), cash_and_equivalents_usd: n('cash_and_equivalents_usd'), current_liabilities_usd: n('current_liabilities_usd') };
        try { const d = await api.calcLiquidityRatios(body); if (!viewIsCurrent(tok)) return; res(mount, d); }
        catch (e) { showToast(e.message || t('view.liqr.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 250); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function res(mount, d) {
    const el = mount.querySelector('#liqr-result');
    if (!d.valid) { el.innerHTML = `<p class="muted" data-i18n="view.liqr.invalid">Current liabilities must be positive.</p>`; applyUiI18n(el); return; }
    const statusKey = d.solvent_short_term ? 'view.liqr.solvent' : 'view.liqr.tight';
    const cls = d.solvent_short_term ? 'pos' : 'neg';
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card ${cls}"><div class="label" data-i18n="view.liqr.card.nwc">Net working capital</div><div class="value">${money(d.net_working_capital_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.liqr.card.current">Current ratio</div><div class="value">${x(d.current_ratio)}</div></div>
        <div class="card"><div class="label" data-i18n="view.liqr.card.quick">Quick / acid-test</div><div class="value">${x(d.quick_ratio)}</div></div>
        <div class="card"><div class="label" data-i18n="view.liqr.card.cash">Cash ratio</div><div class="value">${x(d.cash_ratio)}</div></div>
        <div class="card ${cls}"><div class="label" data-i18n="view.liqr.card.status">Short-term</div><div class="value" data-i18n="${statusKey}">${d.solvent_short_term ? 'Solvent' : 'Tight'}</div></div>
    </div></div>`;
    applyUiI18n(el);
}
