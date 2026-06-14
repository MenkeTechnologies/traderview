// Rule of 78s — precomputed-interest rebate, via /calc/rule-of-78.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
export async function renderRuleOf78(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.r78.h1.title">// RULE OF 78s</span></h1>
        <p class="muted small" data-i18n="view.r78.hint.intro">How a precomputed-interest loan allocates its finance charge and the rebate on early payoff. Interest is front-loaded by the sum-of-the-digits method (SOD = n(n+1)/2), so paying off early refunds less unearned interest than simple pro-rata — the penalty this quantifies. Not financial advice.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.r78.h2.inputs">Loan</h2>
        <form id="r78-form" class="inline-form">
            <label><span data-i18n="view.r78.label.charge">Total finance charge ($)</span><input type="number" step="10" min="0" name="total_finance_charge_usd" value="1200" required></label>
            <label><span data-i18n="view.r78.label.term">Original term (months)</span><input type="number" step="1" min="1" name="original_term_months" value="36" required></label>
            <label><span data-i18n="view.r78.label.made">Payments made</span><input type="number" step="1" min="0" name="payments_made" value="12"></label>
        </form></div><div id="r78-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#r78-form'); const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const gen = async () => {
        const body = { total_finance_charge_usd: n('total_finance_charge_usd'), original_term_months: n('original_term_months'), payments_made: n('payments_made') };
        try { const d = await api.calcRuleOf78(body); if (!viewIsCurrent(tok)) return; res(mount, d); }
        catch (e) { showToast(e.message || t('view.r78.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 250); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function res(mount, d) {
    const el = mount.querySelector('#r78-result');
    if (!d.valid) { el.innerHTML = `<p class="muted" data-i18n="view.r78.invalid">Payments made cannot exceed the term.</p>`; applyUiI18n(el); return; }
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card pos"><div class="label" data-i18n="view.r78.card.rebate">Rebate (unearned)</div><div class="value">${money(d.rebate_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.r78.card.earned">Earned interest</div><div class="value">${money(d.earned_interest_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.r78.card.straight">Pro-rata earned</div><div class="value">${money(d.straight_line_earned_usd)}</div></div>
        <div class="card neg"><div class="label" data-i18n="view.r78.card.penalty">Early-payoff penalty</div><div class="value">${money(d.early_payoff_penalty_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.r78.card.sod">Sum of digits</div><div class="value">${d.sum_of_digits}</div></div>
    </div></div>`;
    applyUiI18n(el);
}
