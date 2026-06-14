// Self-employment tax — Schedule SE, via /calc/self-employment-tax.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
export async function renderSelfEmploymentTax(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.setax.h1.title">// SELF-EMPLOYMENT TAX</span></h1>
        <p class="muted small" data-i18n="view.setax.hint.intro">The Social Security and Medicare tax owed on self-employment income (Schedule SE). Net SE earnings are 92.35% of net profit; Social Security (12.4%) applies up to the wage base and Medicare (2.9%) to all of it. Half the SE tax is an above-the-line deduction. Not tax advice.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.setax.h2.inputs">Inputs</h2>
        <form id="setax-form" class="inline-form">
            <label><span data-i18n="view.setax.label.profit">Net profit ($)</span><input type="number" step="1000" min="0" name="net_profit_usd" value="100000" required></label>
            <label><span data-i18n="view.setax.label.base">SS wage base ($)</span><input type="number" step="100" min="0" name="ss_wage_base_usd" value="168600"></label>
            <label><span data-i18n="view.setax.label.ss">SS rate (%)</span><input type="number" step="0.1" min="0" name="ss_rate_pct" value="12.4"></label>
            <label><span data-i18n="view.setax.label.med">Medicare rate (%)</span><input type="number" step="0.1" min="0" name="medicare_rate_pct" value="2.9"></label>
        </form></div><div id="setax-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#setax-form'); const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const gen = async () => {
        const body = { net_profit_usd: n('net_profit_usd'), ss_wage_base_usd: n('ss_wage_base_usd') || 168600, ss_rate_pct: n('ss_rate_pct') || 12.4, medicare_rate_pct: n('medicare_rate_pct') || 2.9 };
        try { const d = await api.calcSelfEmploymentTax(body); if (!viewIsCurrent(tok)) return; res(mount, d); }
        catch (e) { showToast(e.message || t('view.setax.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 250); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function res(mount, d) {
    const el = mount.querySelector('#setax-result');
    const capNote = d.ss_capped ? ` <span class="muted small" data-i18n="view.setax.capped">(capped)</span>` : '';
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card neg"><div class="label" data-i18n="view.setax.card.se">SE tax</div><div class="value">${money(d.se_tax_usd)}</div></div>
        <div class="card pos"><div class="label" data-i18n="view.setax.card.ded">Deductible half</div><div class="value">${money(d.deductible_half_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.setax.card.ss">Social Security${capNote}</div><div class="value">${money(d.social_security_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.setax.card.med">Medicare</div><div class="value">${money(d.medicare_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.setax.card.nse">Net SE earnings</div><div class="value">${money(d.net_se_earnings_usd)}</div></div>
    </div></div>`;
    applyUiI18n(el);
}
