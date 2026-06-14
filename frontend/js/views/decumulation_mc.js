// Retirement decumulation Monte Carlo, via /calc/decumulation-mc.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
export async function renderDecumulationMc(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.decmc.h1.title">// DECUMULATION MONTE CARLO</span></h1>
        <p class="muted small" data-i18n="view.decmc.hint.intro">Simulates a retirement portfolio drawn down over the horizon to estimate the probability the money lasts. Each year the balance grows by a normally-distributed return and an inflation-adjusted withdrawal is taken. Fully deterministic (fixed-seed PRNG). Not financial advice.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.decmc.h2.inputs">Plan</h2>
        <form id="decmc-form" class="inline-form">
            <label><span data-i18n="view.decmc.label.bal">Initial balance ($)</span><input type="number" step="10000" min="0" name="initial_balance_usd" value="1000000" required></label>
            <label><span data-i18n="view.decmc.label.wd">Annual withdrawal ($)</span><input type="number" step="1000" min="0" name="annual_withdrawal_usd" value="40000" required></label>
            <label><span data-i18n="view.decmc.label.mean">Mean return (%)</span><input type="number" step="0.5" name="mean_return_pct" value="6"></label>
            <label><span data-i18n="view.decmc.label.vol">Volatility (%)</span><input type="number" step="0.5" min="0" name="volatility_pct" value="12"></label>
            <label><span data-i18n="view.decmc.label.infl">Inflation (%)</span><input type="number" step="0.1" name="inflation_pct" value="2.5"></label>
            <label><span data-i18n="view.decmc.label.years">Years</span><input type="number" step="1" min="1" name="years" value="30" required></label>
            <label><span data-i18n="view.decmc.label.sims">Simulations</span><input type="number" step="500" min="1" name="simulations" value="2000"></label>
        </form></div><div id="decmc-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#decmc-form'); const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const gen = async () => {
        const body = { initial_balance_usd: n('initial_balance_usd'), annual_withdrawal_usd: n('annual_withdrawal_usd'), mean_return_pct: n('mean_return_pct'), volatility_pct: n('volatility_pct'), inflation_pct: n('inflation_pct'), years: n('years'), simulations: n('simulations') || 2000 };
        try { const d = await api.calcDecumulationMc(body); if (!viewIsCurrent(tok)) return; res(mount, d); }
        catch (e) { showToast(e.message || t('view.decmc.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 300); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function res(mount, d) {
    const el = mount.querySelector('#decmc-result');
    const cls = d.success_rate_pct >= 85 ? 'pos' : d.success_rate_pct >= 70 ? '' : 'neg';
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card ${cls}"><div class="label" data-i18n="view.decmc.card.succ">Success rate</div><div class="value">${pct(d.success_rate_pct)}</div></div>
        <div class="card"><div class="label" data-i18n="view.decmc.card.p50">Median ending</div><div class="value">${money(d.median_ending_balance_usd)}</div></div>
        <div class="card neg"><div class="label" data-i18n="view.decmc.card.p10">10th pct ending</div><div class="value">${money(d.p10_ending_balance_usd)}</div></div>
        <div class="card pos"><div class="label" data-i18n="view.decmc.card.p90">90th pct ending</div><div class="value">${money(d.p90_ending_balance_usd)}</div></div>
        <div class="card"><div class="label" data-i18n="view.decmc.card.sims">Simulations</div><div class="value">${d.simulations}</div></div>
    </div></div>`;
    applyUiI18n(el);
}
