// Revenue retention (NRR & GRR), via /calc/revenue-retention.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
export async function renderRevenueRetention(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.nrr.h1.title">// REVENUE RETENTION</span></h1>
        <p class="muted small" data-i18n="view.nrr.hint.intro">The core subscription-revenue health metrics. From a cohort's starting recurring revenue, expansion adds revenue while contraction and churn remove it. Net revenue retention (NRR) can exceed 100% when expansion outruns losses; gross revenue retention (GRR) ignores expansion and caps at 100%. New-logo revenue is excluded. Not advice.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.nrr.h2.inputs">Cohort movement</h2>
        <form id="nrr-form" class="inline-form">
            <label><span data-i18n="view.nrr.label.period">Period</span><input type="text" name="period_label" value="Q2"></label>
            <label><span data-i18n="view.nrr.label.start">Starting MRR ($)</span><input type="number" step="1000" min="0" name="starting_mrr_usd" value="100000" required></label>
            <label><span data-i18n="view.nrr.label.exp">Expansion MRR ($)</span><input type="number" step="500" min="0" name="expansion_mrr_usd" value="15000"></label>
            <label><span data-i18n="view.nrr.label.contr">Contraction MRR ($)</span><input type="number" step="500" min="0" name="contraction_mrr_usd" value="5000"></label>
            <label><span data-i18n="view.nrr.label.churn">Churned MRR ($)</span><input type="number" step="500" min="0" name="churned_mrr_usd" value="8000"></label>
        </form></div><div id="nrr-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#nrr-form'); const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const gen = async () => {
        const body = { period_label: (form.querySelector('[name="period_label"]').value || '').trim(), starting_mrr_usd: n('starting_mrr_usd'), expansion_mrr_usd: n('expansion_mrr_usd'), contraction_mrr_usd: n('contraction_mrr_usd'), churned_mrr_usd: n('churned_mrr_usd') };
        try { const d = await api.calcRevenueRetention(body); if (!viewIsCurrent(tok)) return; res(mount, d); }
        catch (e) { showToast(e.message || t('view.nrr.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 250); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function res(mount, d) {
    const el = mount.querySelector('#nrr-result');
    if (!d.valid) { el.innerHTML = `<p class="muted" data-i18n="view.nrr.invalid">Starting MRR must be positive.</p>`; applyUiI18n(el); return; }
    const cls = d.net_expanding ? 'pos' : 'neg';
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card ${cls}"><div class="label" data-i18n="view.nrr.card.nrr">Net retention (NRR)</div><div class="value">${pct(d.nrr_pct)}</div></div>
        <div class="card"><div class="label" data-i18n="view.nrr.card.grr">Gross retention (GRR)</div><div class="value">${pct(d.grr_pct)}</div></div>
        <div class="card ${cls}"><div class="label" data-i18n="view.nrr.card.net">Net change</div><div class="value">${money(d.net_change_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.nrr.card.end">Ending MRR</div><div class="value">${money(d.ending_mrr_usd)}</div></div>
    </div></div>`;
    applyUiI18n(el);
}
