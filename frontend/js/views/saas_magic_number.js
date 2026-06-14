// SaaS magic number (sales efficiency), via /calc/saas-magic-number.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const num = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 4 }));
export async function renderSaasMagicNumber(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.smn.h1.title">// SAAS MAGIC NUMBER</span></h1>
        <p class="muted small" data-i18n="view.smn.hint.intro">Sales efficiency: the quarter-over-quarter revenue increase, annualized (×4), divided by the prior quarter's sales & marketing spend. It measures how much new annual recurring revenue each dollar of S&M bought. Above ~0.75 is efficient enough to invest harder; below ~0.5 the go-to-market is not paying back. Not advice.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.smn.h2.inputs">Quarterly revenue & spend</h2>
        <form id="smn-form" class="inline-form">
            <label><span data-i18n="view.smn.label.cur">Current quarter revenue ($)</span><input type="number" step="10000" min="0" name="current_quarter_revenue_usd" value="1100000" required></label>
            <label><span data-i18n="view.smn.label.prior">Prior quarter revenue ($)</span><input type="number" step="10000" min="0" name="prior_quarter_revenue_usd" value="1000000" required></label>
            <label><span data-i18n="view.smn.label.sm">Prior quarter S&M spend ($)</span><input type="number" step="10000" min="0" name="prior_quarter_sm_spend_usd" value="400000" required></label>
        </form></div><div id="smn-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#smn-form'); const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const gen = async () => {
        const body = { current_quarter_revenue_usd: n('current_quarter_revenue_usd'), prior_quarter_revenue_usd: n('prior_quarter_revenue_usd'), prior_quarter_sm_spend_usd: n('prior_quarter_sm_spend_usd') };
        try { const d = await api.calcSaasMagicNumber(body); if (!viewIsCurrent(tok)) return; res(mount, d); }
        catch (e) { showToast(e.message || t('view.smn.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 250); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function res(mount, d) {
    const el = mount.querySelector('#smn-result');
    if (!d.valid) { el.innerHTML = `<p class="muted" data-i18n="view.smn.invalid">Prior quarter S&M spend must be positive.</p>`; applyUiI18n(el); return; }
    const cls = d.efficiency === 'efficient' ? 'pos' : (d.efficiency === 'poor' ? 'neg' : '');
    const effLabel = t('view.smn.eff.' + d.efficiency) || d.efficiency;
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card ${cls}"><div class="label" data-i18n="view.smn.card.magic">Magic number</div><div class="value">${num(d.magic_number)}</div></div>
        <div class="card ${cls}"><div class="label" data-i18n="view.smn.card.eff">Efficiency</div><div class="value">${effLabel}</div></div>
        <div class="card"><div class="label" data-i18n="view.smn.card.arr">Annualized net-new ARR</div><div class="value">${money(d.annualized_net_new_arr_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.smn.card.payback">S&M payback (months)</div><div class="value">${num(d.sm_payback_months)}</div></div>
    </div></div>`;
    applyUiI18n(el);
}
