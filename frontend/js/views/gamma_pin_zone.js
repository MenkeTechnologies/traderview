// Gamma pin zone — gamma-flip level + pinning strike, via /calc/gamma-pin-zone.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const gx = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 }));
export async function renderGammaPinZone(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.gpin.h1.title">// GAMMA PIN ZONE</span></h1>
        <p class="muted small" data-i18n="view.gpin.hint.intro">Finds the dealer gamma-flip level (where net GEX crosses zero) and the strike most likely to pin price near expiration. Enter strikes as "strike:gex" pairs (gex in dealer-gamma units; sign matters).</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.gpin.h2.inputs">Inputs</h2>
        <form id="gpin-form" class="inline-form">
            <label><span data-i18n="view.gpin.label.spot">Spot ($)</span><input type="number" step="0.01" min="0" name="spot" value="100" required></label>
            <label><span data-i18n="view.gpin.label.radius">Pin radius (%)</span><input type="number" step="0.5" min="0" name="pin_radius_pct" value="2"></label>
            <label class="full"><span data-i18n="view.gpin.label.gex">Strikes (strike:gex)</span><textarea name="gex" rows="4">90:-3e8, 95:-1e8, 100:5e8, 105:2e8, 110:-2e8</textarea></label>
        </form></div><div id="gpin-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#gpin-form');
    const gen = async () => {
        const strike_gex = (form.querySelector('[name="gex"]').value || '').split(/[,\n]+/).map((p) => p.split(':').map((x) => Number(x.trim()))).filter((a) => a.length === 2 && a[0] > 0 && Number.isFinite(a[1])).map(([strike, gex]) => ({ strike, gex }));
        const body = { spot: Number(form.querySelector('[name="spot"]').value) || 0, pin_radius_pct: Number(form.querySelector('[name="pin_radius_pct"]').value) || 2, strike_gex };
        if (!strike_gex.length) { mount.querySelector('#gpin-result').innerHTML = ''; return; }
        try { const d = await api.calcGammaPinZone(body); if (!viewIsCurrent(tok)) return; res(mount, d); }
        catch (e) { showToast(e.message || t('view.gpin.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 250); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function res(mount, d) {
    const el = mount.querySelector('#gpin-result');
    if (!d) { el.innerHTML = `<p class="muted" data-i18n="view.gpin.invalid">Enter valid strike GEX data.</p>`; applyUiI18n(el); return; }
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card pos"><div class="label" data-i18n="view.gpin.card.pin">Pin strike</div><div class="value">${d.pin_strike == null ? '—' : money(d.pin_strike)}</div></div>
        <div class="card"><div class="label" data-i18n="view.gpin.card.flip">Gamma flip</div><div class="value">${d.gamma_flip == null ? '—' : money(d.gamma_flip)}</div></div>
        <div class="card"><div class="label" data-i18n="view.gpin.card.strength">Pin strength</div><div class="value">${d.pin_strength == null ? '—' : gx(d.pin_strength)}</div></div>
        <div class="card"><div class="label" data-i18n="view.gpin.card.total">Total GEX</div><div class="value">${gx(d.total_gex)}</div></div>
        <div class="card"><div class="label" data-i18n="view.gpin.card.n">Strikes</div><div class="value">${d.n_strikes}</div></div>
    </div></div>`;
    applyUiI18n(el);
}
