// Collar — max profit/loss, upside cap, downside floor, breakeven, via /calc/collar.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
export async function renderCollar(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.collar.h1.title">// COLLAR</span></h1>
        <p class="muted small" data-i18n="view.collar.hint.intro">A collar holds stock, buys a protective put, and sells a covered call. It computes the capped upside, the protected downside floor, the breakeven, and max profit/loss.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.collar.h2.inputs">Position</h2>
        <form id="collar-form" class="inline-form">
            <label><span data-i18n="view.collar.label.basis">Stock basis ($)</span><input type="number" step="0.01" min="0" name="stock_basis" value="100" required></label>
            <label><span data-i18n="view.collar.label.put">Put strike ($)</span><input type="number" step="0.5" min="0" name="put_strike" value="95" required></label>
            <label><span data-i18n="view.collar.label.call">Call strike ($)</span><input type="number" step="0.5" min="0" name="call_strike" value="110" required></label>
            <label><span data-i18n="view.collar.label.debit">Net option debit/share ($)</span><input type="number" step="0.01" name="net_option_debit_per_share" value="0.50"></label>
            <label><span data-i18n="view.collar.label.shares">Shares</span><input type="number" step="100" min="0" name="shares" value="100" required></label>
        </form></div><div id="collar-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#collar-form'); const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const gen = async () => {
        const body = { stock_basis: n('stock_basis'), put_strike: n('put_strike'), call_strike: n('call_strike'), net_option_debit_per_share: n('net_option_debit_per_share'), shares: n('shares') };
        try { const d = await api.calcCollar(body); if (!viewIsCurrent(tok)) return; res(mount, d); }
        catch (e) { showToast(e.message || t('view.collar.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 250); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function res(mount, d) {
    const el = mount.querySelector('#collar-result');
    if (!d) { el.innerHTML = `<p class="muted" data-i18n="view.collar.invalid">Invalid inputs.</p>`; applyUiI18n(el); return; }
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card pos"><div class="label" data-i18n="view.collar.card.maxp">Max profit</div><div class="value">${money(d.max_profit)}</div></div>
        <div class="card neg"><div class="label" data-i18n="view.collar.card.maxl">Max loss</div><div class="value">${money(d.max_loss)}</div></div>
        <div class="card"><div class="label" data-i18n="view.collar.card.cap">Upside cap</div><div class="value">${money(d.upside_cap_at_call_strike)}</div></div>
        <div class="card"><div class="label" data-i18n="view.collar.card.floor">Downside floor</div><div class="value">${money(d.downside_floor_at_put_strike)}</div></div>
        <div class="card"><div class="label" data-i18n="view.collar.card.be">Breakeven spot</div><div class="value">${money(d.breakeven_spot)}</div></div>
    </div></div>`;
    applyUiI18n(el);
}
