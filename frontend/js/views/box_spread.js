// Box spread — synthetic-loan implied rate and arbitrage check, via /calc/box-spread.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const ratep = (n) => (n == null ? '—' : (Number(n) * 100).toLocaleString(undefined, { maximumFractionDigits: 3 }) + '%');
export async function renderBoxSpread(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.box.h1.title">// BOX SPREAD</span></h1>
        <p class="muted small" data-i18n="view.box.hint.intro">A box spread (long lower vertical + short upper vertical with calls and puts) locks a fixed payoff equal to the strike width. It implies a synthetic financing rate; comparing it to the market rate flags an arbitrage when the gap exceeds the threshold.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.box.h2.inputs">Quotes</h2>
        <form id="box-form" class="inline-form">
            <label><span data-i18n="view.box.label.klo">Strike low ($)</span><input type="number" step="0.5" min="0" name="strike_low" value="100" required></label>
            <label><span data-i18n="view.box.label.khi">Strike high ($)</span><input type="number" step="0.5" min="0" name="strike_high" value="110" required></label>
            <label><span data-i18n="view.box.label.clo">Call @ low ($)</span><input type="number" step="0.01" min="0" name="call_low_price" value="12.50" required></label>
            <label><span data-i18n="view.box.label.chi">Call @ high ($)</span><input type="number" step="0.01" min="0" name="call_high_price" value="4.20" required></label>
            <label><span data-i18n="view.box.label.plo">Put @ low ($)</span><input type="number" step="0.01" min="0" name="put_low_price" value="2.10" required></label>
            <label><span data-i18n="view.box.label.phi">Put @ high ($)</span><input type="number" step="0.01" min="0" name="put_high_price" value="3.60" required></label>
            <label><span data-i18n="view.box.label.t">Time to expiry (years)</span><input type="number" step="0.01" min="0" name="time_to_expiry_years" value="0.5" required></label>
            <label><span data-i18n="view.box.label.rf">Market rate (decimal)</span><input type="number" step="0.005" name="market_risk_free_rate" value="0.05"></label>
            <label><span data-i18n="view.box.label.thr">Arb threshold (bps)</span><input type="number" step="1" min="0" name="arbitrage_threshold_bps" value="25"></label>
        </form></div><div id="box-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#box-form'); const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const gen = async () => {
        const body = { strike_low: n('strike_low'), strike_high: n('strike_high'), call_low_price: n('call_low_price'), call_high_price: n('call_high_price'), put_low_price: n('put_low_price'), put_high_price: n('put_high_price'), time_to_expiry_years: n('time_to_expiry_years'), market_risk_free_rate: n('market_risk_free_rate'), arbitrage_threshold_bps: n('arbitrage_threshold_bps') };
        try { const d = await api.calcBoxSpread(body); if (!viewIsCurrent(tok)) return; res(mount, d); }
        catch (e) { showToast(e.message || t('view.box.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 250); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function res(mount, d) {
    const el = mount.querySelector('#box-result');
    if (!d) { el.innerHTML = `<p class="muted" data-i18n="view.box.invalid">Invalid quotes (need high strike > low strike).</p>`; applyUiI18n(el); return; }
    const arbKey = d.is_arbitrage_opportunity ? 'view.box.arb' : 'view.box.noarb';
    const cls = d.is_arbitrage_opportunity ? 'pos' : '';
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card ${cls}"><div class="label" data-i18n="view.box.card.arb">Arbitrage</div><div class="value" data-i18n="${arbKey}">${d.is_arbitrage_opportunity ? 'Yes' : 'No'}</div></div>
        <div class="card"><div class="label" data-i18n="view.box.card.implied">Implied rate</div><div class="value">${ratep(d.implied_continuous_rate)}</div></div>
        <div class="card"><div class="label" data-i18n="view.box.card.market">Market rate</div><div class="value">${ratep(d.market_rate)}</div></div>
        <div class="card"><div class="label" data-i18n="view.box.card.basis">Basis (bps)</div><div class="value">${Number(d.arbitrage_basis_points).toFixed(1)}</div></div>
        <div class="card"><div class="label" data-i18n="view.box.card.payoff">Locked payoff</div><div class="value">${money(d.locked_payoff)}</div></div>
        <div class="card"><div class="label" data-i18n="view.box.card.debit">Net debit</div><div class="value">${money(d.net_premium_debit)}</div></div>
    </div></div>`;
    applyUiI18n(el);
}
