// Crypto perpetual liquidation price (isolated margin), via /calc/crypto-liquidation.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
export async function renderCryptoLiquidation(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.cliq.h1.title">// PERP LIQUIDATION PRICE</span></h1>
        <p class="muted small" data-i18n="view.cliq.hint.intro">The mark price at which an isolated-margin linear (USDT) perpetual position is force-liquidated. Long liquidation = entry × (1 − 1/leverage + maintenance margin rate); short flips the signs. Cross-margin and inverse contracts differ. Not financial advice.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.cliq.h2.inputs">Position</h2>
        <form id="cliq-form" class="inline-form">
            <label><span data-i18n="view.cliq.label.side">Side</span><select name="side"><option value="long" data-i18n="view.cliq.opt.long">Long</option><option value="short" data-i18n="view.cliq.opt.short">Short</option></select></label>
            <label><span data-i18n="view.cliq.label.entry">Entry price ($)</span><input type="number" step="0.01" min="0" name="entry_price_usd" value="30000" required></label>
            <label><span data-i18n="view.cliq.label.lev">Leverage (×)</span><input type="number" step="1" min="1" name="leverage" value="10" required></label>
            <label><span data-i18n="view.cliq.label.mmr">Maint. margin rate (decimal)</span><input type="number" step="0.001" min="0" name="maintenance_margin_rate" value="0.005"></label>
            <label><span data-i18n="view.cliq.label.size">Position size (coins)</span><input type="number" step="0.1" min="0" name="position_size" value="1"></label>
        </form></div><div id="cliq-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#cliq-form'); const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const gen = async () => {
        const body = { side: form.querySelector('[name="side"]').value, entry_price_usd: n('entry_price_usd'), leverage: n('leverage'), maintenance_margin_rate: n('maintenance_margin_rate'), position_size: n('position_size') };
        try { const d = await api.calcCryptoLiquidation(body); if (!viewIsCurrent(tok)) return; res(mount, d); }
        catch (e) { showToast(e.message || t('view.cliq.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 250); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function res(mount, d) {
    const el = mount.querySelector('#cliq-result');
    if (!d.valid) { el.innerHTML = `<p class="muted" data-i18n="view.cliq.invalid">Invalid inputs (leverage and entry must be positive).</p>`; applyUiI18n(el); return; }
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card neg"><div class="label" data-i18n="view.cliq.card.liq">Liquidation price</div><div class="value">${money(d.liquidation_price_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.cliq.card.dist">Distance to liq</div><div class="value">${pct(d.distance_pct)}</div></div>
        <div class="card"><div class="label" data-i18n="view.cliq.card.bank">Bankruptcy price</div><div class="value">${money(d.bankruptcy_price_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.cliq.card.im">Initial margin</div><div class="value">${money(d.initial_margin_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.cliq.card.mm">Maint. margin</div><div class="value">${money(d.maintenance_margin_usd)}</div></div>
    </div></div>`;
    applyUiI18n(el);
}
