// Perpetual funding & basis, via /calc/perp-funding.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 4 }) + '%');
export async function renderPerpFunding(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.perpf.h1.title">// PERP FUNDING & BASIS</span></h1>
        <p class="muted small" data-i18n="view.perpf.hint.intro">The cost of carry on a perpetual-futures position and its premium to spot. Funding per interval = notional × funding rate (longs pay shorts when positive); the annualized rate scales by intervals/day over a year; basis = (perp − index) ÷ index. Not financial advice.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.perpf.h2.inputs">Inputs</h2>
        <form id="perpf-form" class="inline-form">
            <label><span data-i18n="view.perpf.label.side">Side</span><select name="side"><option value="long" data-i18n="view.perpf.opt.long">Long</option><option value="short" data-i18n="view.perpf.opt.short">Short</option></select></label>
            <label><span data-i18n="view.perpf.label.perp">Perp price ($)</span><input type="number" step="0.01" min="0" name="perp_price_usd" value="30050" required></label>
            <label><span data-i18n="view.perpf.label.index">Index price ($)</span><input type="number" step="0.01" min="0" name="index_price_usd" value="30000" required></label>
            <label><span data-i18n="view.perpf.label.rate">Funding rate/interval (decimal)</span><input type="number" step="0.00001" name="funding_rate" value="0.0001"></label>
            <label><span data-i18n="view.perpf.label.intervals">Intervals/day</span><input type="number" step="1" min="1" name="intervals_per_day" value="3"></label>
            <label><span data-i18n="view.perpf.label.notional">Position notional ($)</span><input type="number" step="100" min="0" name="position_notional_usd" value="100000" required></label>
        </form></div><div id="perpf-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#perpf-form'); const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const gen = async () => {
        const body = { side: form.querySelector('[name="side"]').value, perp_price_usd: n('perp_price_usd'), index_price_usd: n('index_price_usd'), funding_rate: n('funding_rate'), intervals_per_day: n('intervals_per_day') || 3, position_notional_usd: n('position_notional_usd') };
        try { const d = await api.calcPerpFunding(body); if (!viewIsCurrent(tok)) return; res(mount, d); }
        catch (e) { showToast(e.message || t('view.perpf.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 250); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function res(mount, d) {
    const el = mount.querySelector('#perpf-result');
    const dirKey = d.pays_funding ? 'view.perpf.pays' : 'view.perpf.receives';
    const cls = d.pays_funding ? 'neg' : 'pos';
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card ${cls}"><div class="label" data-i18n="view.perpf.card.dir">Funding</div><div class="value" data-i18n="${dirKey}">${d.pays_funding ? 'Pays' : 'Receives'}</div></div>
        <div class="card"><div class="label" data-i18n="view.perpf.card.interval">Per interval</div><div class="value">${money(d.funding_per_interval_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.perpf.card.day">Per day</div><div class="value">${money(d.funding_per_day_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.perpf.card.ann">Annualized</div><div class="value">${pct(d.annualized_funding_pct)}</div></div>
        <div class="card"><div class="label" data-i18n="view.perpf.card.basis">Basis</div><div class="value">${pct(d.basis_pct)}</div></div>
    </div></div>`;
    applyUiI18n(el);
}
