// Butterfly spread — max profit/loss, breakevens, wing width, via /calc/butterfly-spread.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const num = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 4 }));
export async function renderButterflySpread(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.bfly.h1.title">// BUTTERFLY SPREAD</span></h1>
        <p class="muted small" data-i18n="view.bfly.hint.intro">A long butterfly buys one lower wing, sells two body options, and buys one upper wing (all calls or all puts). It computes the max profit at the body, max loss (the debit), breakevens, and the debit-to-max-profit ratio.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.bfly.h2.inputs">Position</h2>
        <form id="bfly-form" class="inline-form">
            <label><span data-i18n="view.bfly.label.kind">Option kind</span><select name="kind"><option value="call" data-i18n="view.bfly.opt.call">Call</option><option value="put" data-i18n="view.bfly.opt.put">Put</option></select></label>
            <label><span data-i18n="view.bfly.label.lower">Lower wing strike ($)</span><input type="number" step="0.5" min="0" name="lower_wing_strike" value="95" required></label>
            <label><span data-i18n="view.bfly.label.body">Body strike ($)</span><input type="number" step="0.5" min="0" name="body_strike" value="100" required></label>
            <label><span data-i18n="view.bfly.label.upper">Upper wing strike ($)</span><input type="number" step="0.5" min="0" name="upper_wing_strike" value="105" required></label>
            <label><span data-i18n="view.bfly.label.debit">Net debit/contract ($)</span><input type="number" step="0.01" min="0" name="net_debit_per_contract" value="1.50" required></label>
            <label><span data-i18n="view.bfly.label.contracts">Contracts</span><input type="number" step="1" name="contracts" value="1" required></label>
            <label><span data-i18n="view.bfly.label.mult">Multiplier</span><input type="number" step="1" min="1" name="multiplier" value="100"></label>
        </form></div><div id="bfly-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#bfly-form'); const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const gen = async () => {
        const body = { kind: form.querySelector('[name="kind"]').value, lower_wing_strike: n('lower_wing_strike'), body_strike: n('body_strike'), upper_wing_strike: n('upper_wing_strike'), net_debit_per_contract: n('net_debit_per_contract'), contracts: n('contracts'), multiplier: n('multiplier') || 100 };
        try { const d = await api.calcButterflySpread(body); if (!viewIsCurrent(tok)) return; res(mount, d); }
        catch (e) { showToast(e.message || t('view.bfly.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 250); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function res(mount, d) {
    const el = mount.querySelector('#bfly-result');
    if (!d) { el.innerHTML = `<p class="muted" data-i18n="view.bfly.invalid">Invalid inputs.</p>`; applyUiI18n(el); return; }
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card pos"><div class="label" data-i18n="view.bfly.card.maxp">Max profit</div><div class="value">${money(d.max_profit)}</div></div>
        <div class="card neg"><div class="label" data-i18n="view.bfly.card.maxl">Max loss</div><div class="value">${money(d.max_loss)}</div></div>
        <div class="card"><div class="label" data-i18n="view.bfly.card.be">Breakevens</div><div class="value">${money(d.lower_breakeven)} / ${money(d.upper_breakeven)}</div></div>
        <div class="card"><div class="label" data-i18n="view.bfly.card.wing">Wing width</div><div class="value">${money(d.wing_width)}</div></div>
        <div class="card"><div class="label" data-i18n="view.bfly.card.ratio">Debit/max-profit</div><div class="value">${num(d.debit_to_max_profit_ratio)}</div></div>
    </div></div>`;
    applyUiI18n(el);
}
