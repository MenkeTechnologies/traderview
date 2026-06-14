// Iron butterfly — max profit/loss, breakevens, wing width, via /calc/iron-butterfly.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
export async function renderIronButterfly(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ironbf.h1.title">// IRON BUTTERFLY</span></h1>
        <p class="muted small" data-i18n="view.ironbf.hint.intro">A short iron butterfly sells an at-the-money straddle (the body) and buys protective wings. It computes the max profit (the net credit), max loss, breakevens, and wing width.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.ironbf.h2.inputs">Position</h2>
        <form id="ironbf-form" class="inline-form">
            <label><span data-i18n="view.ironbf.label.putlong">Long put strike ($)</span><input type="number" step="0.5" min="0" name="put_long_strike" value="90" required></label>
            <label><span data-i18n="view.ironbf.label.body">Body strike ($)</span><input type="number" step="0.5" min="0" name="body_strike" value="100" required></label>
            <label><span data-i18n="view.ironbf.label.calllong">Long call strike ($)</span><input type="number" step="0.5" min="0" name="call_long_strike" value="110" required></label>
            <label><span data-i18n="view.ironbf.label.credit">Net credit/contract ($)</span><input type="number" step="0.01" min="0" name="net_credit_per_contract" value="6" required></label>
            <label><span data-i18n="view.ironbf.label.contracts">Contracts</span><input type="number" step="1" name="contracts" value="1" required></label>
            <label><span data-i18n="view.ironbf.label.mult">Multiplier</span><input type="number" step="1" min="1" name="multiplier" value="100"></label>
        </form></div><div id="ironbf-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#ironbf-form'); const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const gen = async () => {
        const body = { put_long_strike: n('put_long_strike'), body_strike: n('body_strike'), call_long_strike: n('call_long_strike'), net_credit_per_contract: n('net_credit_per_contract'), contracts: n('contracts'), multiplier: n('multiplier') || 100 };
        try { const d = await api.calcIronButterfly(body); if (!viewIsCurrent(tok)) return; res(mount, d); }
        catch (e) { showToast(e.message || t('view.ironbf.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 250); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function res(mount, d) {
    const el = mount.querySelector('#ironbf-result');
    if (!d) { el.innerHTML = `<p class="muted" data-i18n="view.ironbf.invalid">Invalid inputs.</p>`; applyUiI18n(el); return; }
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card pos"><div class="label" data-i18n="view.ironbf.card.maxp">Max profit</div><div class="value">${money(d.max_profit)}</div></div>
        <div class="card neg"><div class="label" data-i18n="view.ironbf.card.maxl">Max loss</div><div class="value">${money(d.max_loss)}</div></div>
        <div class="card"><div class="label" data-i18n="view.ironbf.card.be">Breakevens</div><div class="value">${money(d.lower_breakeven)} / ${money(d.upper_breakeven)}</div></div>
        <div class="card"><div class="label" data-i18n="view.ironbf.card.wing">Wing width</div><div class="value">${money(d.wing_width)}</div></div>
    </div></div>`;
    applyUiI18n(el);
}
