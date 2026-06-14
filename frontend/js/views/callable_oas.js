// Callable-bond option-adjusted spread, via /calc/callable-oas.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const num = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 4 }));
export async function renderCallableOas(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.coas.h1.title">// CALLABLE BOND OAS</span></h1>
        <p class="muted small" data-i18n="view.coas.hint.intro">Values a callable bond on a binomial short-rate lattice and solves for the constant spread (the OAS) that matches the market price. The straight (option-free) value minus the callable value is the embedded call's cost. Single-factor model, not a calibrated curve.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.coas.h2.inputs">Bond</h2>
        <form id="coas-form" class="inline-form">
            <label><span data-i18n="view.coas.label.face">Face value</span><input type="number" step="1" min="0" name="face_value" value="100"></label>
            <label><span data-i18n="view.coas.label.coupon">Coupon rate (%)</span><input type="number" step="0.25" min="0" name="coupon_rate_pct" value="5" required></label>
            <label><span data-i18n="view.coas.label.mat">Maturity (years)</span><input type="number" step="1" min="1" name="maturity_years" value="5" required></label>
            <label><span data-i18n="view.coas.label.steps">Lattice steps</span><input type="number" step="1" min="1" name="steps" value="5"></label>
            <label><span data-i18n="view.coas.label.rate">Short rate (%)</span><input type="number" step="0.1" name="short_rate_pct" value="4" required></label>
            <label><span data-i18n="view.coas.label.vol">Rate vol (%)</span><input type="number" step="1" min="0" name="rate_vol_pct" value="20" required></label>
            <label><span data-i18n="view.coas.label.call">Call price</span><input type="number" step="1" min="0" name="call_price" value="100"></label>
            <label><span data-i18n="view.coas.label.lock">Lockout (years)</span><input type="number" step="1" min="0" name="lockout_years" value="2"></label>
            <label><span data-i18n="view.coas.label.mkt">Market price</span><input type="number" step="0.1" min="0" name="market_price" value="99" required></label>
        </form></div><div id="coas-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#coas-form'); const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const gen = async () => {
        const body = { face_value: n('face_value') || 100, coupon_rate_pct: n('coupon_rate_pct'), maturity_years: n('maturity_years'), steps: n('steps') || 5, short_rate_pct: n('short_rate_pct'), rate_vol_pct: n('rate_vol_pct'), call_price: n('call_price') || 100, lockout_years: n('lockout_years'), market_price: n('market_price') };
        try { const d = await api.calcCallableOas(body); if (!viewIsCurrent(tok)) return; res(mount, d); }
        catch (e) { showToast(e.message || t('view.coas.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 250); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function res(mount, d) {
    const el = mount.querySelector('#coas-result');
    if (!d.valid) { el.innerHTML = `<p class="muted" data-i18n="view.coas.invalid">Invalid inputs.</p>`; applyUiI18n(el); return; }
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card pos"><div class="label" data-i18n="view.coas.card.oas">OAS</div><div class="value">${num(d.oas_bps)} bps</div></div>
        <div class="card"><div class="label" data-i18n="view.coas.card.straight">Straight price</div><div class="value">${num(d.straight_price)}</div></div>
        <div class="card"><div class="label" data-i18n="view.coas.card.callable">Callable price</div><div class="value">${num(d.callable_price)}</div></div>
        <div class="card neg"><div class="label" data-i18n="view.coas.card.opt">Option cost</div><div class="value">${num(d.option_cost)}</div></div>
    </div></div>`;
    applyUiI18n(el);
}
