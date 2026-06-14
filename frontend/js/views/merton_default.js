// Merton structural default — distance to default + PD, via /calc/merton-default.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const num = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 4 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 3 }) + '%');
export async function renderMertonDefault(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.merton.h1.title">// MERTON DEFAULT MODEL</span></h1>
        <p class="muted small" data-i18n="view.merton.hint.intro">Treats equity as a call option on the firm's assets to derive the distance to default and the probability of default. DD = (ln(V/D) + (r − ½σ²)T) / (σ√T); PD = N(−DD). A structural approximation, not a credit rating.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.merton.h2.inputs">Firm inputs</h2>
        <form id="merton-form" class="inline-form">
            <label><span data-i18n="view.merton.label.v">Asset value ($)</span><input type="number" step="1" min="0" name="asset_value_usd" value="100" required></label>
            <label><span data-i18n="view.merton.label.d">Debt face ($)</span><input type="number" step="1" min="0" name="debt_face_usd" value="80" required></label>
            <label><span data-i18n="view.merton.label.sig">Asset volatility (decimal)</span><input type="number" step="0.01" min="0" name="asset_volatility" value="0.30" required></label>
            <label><span data-i18n="view.merton.label.r">Risk-free rate (decimal)</span><input type="number" step="0.005" name="risk_free_rate" value="0.05"></label>
            <label><span data-i18n="view.merton.label.t">Horizon (years)</span><input type="number" step="0.25" min="0" name="horizon_years" value="1"></label>
        </form></div><div id="merton-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#merton-form'); const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const gen = async () => {
        const body = { asset_value_usd: n('asset_value_usd'), debt_face_usd: n('debt_face_usd'), asset_volatility: n('asset_volatility'), risk_free_rate: n('risk_free_rate'), horizon_years: n('horizon_years') };
        try { const d = await api.calcMertonDefault(body); if (!viewIsCurrent(tok)) return; res(mount, d); }
        catch (e) { showToast(e.message || t('view.merton.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 250); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function res(mount, d) {
    const el = mount.querySelector('#merton-result');
    if (!d.valid) { el.innerHTML = `<p class="muted" data-i18n="view.merton.invalid">Invalid inputs (value, debt, vol, horizon must be positive).</p>`; applyUiI18n(el); return; }
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card neg"><div class="label" data-i18n="view.merton.card.pd">Probability of default</div><div class="value">${pct(d.probability_of_default_pct)}</div></div>
        <div class="card pos"><div class="label" data-i18n="view.merton.card.dd">Distance to default</div><div class="value">${num(d.distance_to_default)}σ</div></div>
        <div class="card"><div class="label" data-i18n="view.merton.card.lev">Leverage</div><div class="value">${pct(d.leverage_pct)}</div></div>
    </div></div>`;
    applyUiI18n(el);
}
