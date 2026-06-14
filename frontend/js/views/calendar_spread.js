// Calendar spread — net debit, breakevens, max profit/loss, via /calc/calendar-spread.
import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
export async function renderCalendarSpread(mount, _s) {
    const tok = currentViewToken(); if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.cal.h1.title">// CALENDAR SPREAD</span></h1>
        <p class="muted small" data-i18n="view.cal.hint.intro">A calendar (horizontal) spread sells a near-term option and buys a longer-dated option at the same strike. P&L is evaluated at the front expiration (the back leg is repriced by Black-Scholes). It computes the net debit, breakevens, and max profit/loss across the spot grid.</p>
        <div class="lpv-split"><div class="chart-panel"><h2 data-i18n="view.cal.h2.inputs">Position</h2>
        <form id="cal-form" class="inline-form">
            <label><span data-i18n="view.cal.label.kind">Option kind</span><select name="kind"><option value="call" data-i18n="view.cal.opt.call">Call</option><option value="put" data-i18n="view.cal.opt.put">Put</option></select></label>
            <label><span data-i18n="view.cal.label.strike">Strike ($)</span><input type="number" step="0.5" min="0" name="strike" value="100" required></label>
            <label><span data-i18n="view.cal.label.front">Front premium ($)</span><input type="number" step="0.01" min="0" name="front_premium" value="2.5" required></label>
            <label><span data-i18n="view.cal.label.back">Back premium ($)</span><input type="number" step="0.01" min="0" name="back_premium" value="4.0" required></label>
            <label><span data-i18n="view.cal.label.backt">Back time after front (years)</span><input type="number" step="0.05" min="0" name="back_time_after_front_expiry" value="0.25" required></label>
            <label><span data-i18n="view.cal.label.rf">Risk-free (decimal)</span><input type="number" step="0.005" name="risk_free" value="0.04"></label>
            <label><span data-i18n="view.cal.label.q">Dividend yield (decimal)</span><input type="number" step="0.005" name="dividend_yield" value="0"></label>
            <label><span data-i18n="view.cal.label.sig">Volatility (decimal)</span><input type="number" step="0.01" min="0" name="sigma" value="0.22" required></label>
            <label><span data-i18n="view.cal.label.contracts">Contracts</span><input type="number" step="1" name="contracts" value="1" required></label>
            <label><span data-i18n="view.cal.label.mult">Multiplier</span><input type="number" step="1" min="1" name="multiplier" value="100"></label>
            <label><span data-i18n="view.cal.label.points">Grid points</span><input type="number" step="1" min="3" name="grid_points" value="41"></label>
        </form></div><div id="cal-result" class="chart-panel lpv-preview"></div></div>`;
    applyUiI18n(mount);
    const form = mount.querySelector('#cal-form'); const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const gen = async () => {
        const body = {
            spread: { strike: n('strike'), kind: form.querySelector('[name="kind"]').value, front_premium: n('front_premium'), back_premium: n('back_premium'), back_time_after_front_expiry: n('back_time_after_front_expiry'), risk_free: n('risk_free'), dividend_yield: n('dividend_yield'), sigma: n('sigma'), contracts: n('contracts'), multiplier: n('multiplier') || 100 },
            config: { grid_low_pct_of_strike: 0.5, grid_high_pct_of_strike: 1.5, grid_points: n('grid_points') || 41 },
        };
        try { const d = await api.calcCalendarSpread(body); if (!viewIsCurrent(tok)) return; res(mount, d); }
        catch (e) { showToast(e.message || t('view.cal.toast.error'), { level: 'error' }); }
    };
    const live = debounce(gen, 250); form.addEventListener('input', () => live()); form.addEventListener('submit', (e) => { e.preventDefault(); gen(); }); gen();
}
function res(mount, d) {
    const el = mount.querySelector('#cal-result');
    if (!d) { el.innerHTML = `<p class="muted" data-i18n="view.cal.invalid">Invalid inputs.</p>`; applyUiI18n(el); return; }
    const be = (d.breakevens || []).map(money).join(', ') || '—';
    el.innerHTML = `<div class="lpv-bar"><div class="cards">
        <div class="card neg"><div class="label" data-i18n="view.cal.card.debit">Net debit</div><div class="value">${money(d.net_debit)}</div></div>
        <div class="card pos"><div class="label" data-i18n="view.cal.card.maxp">Max profit</div><div class="value">${money(d.max_profit)}</div></div>
        <div class="card"><div class="label" data-i18n="view.cal.card.at">Max profit at</div><div class="value">${money(d.max_profit_at)}</div></div>
        <div class="card neg"><div class="label" data-i18n="view.cal.card.maxl">Max loss</div><div class="value">${money(d.max_loss)}</div></div>
        <div class="card"><div class="label" data-i18n="view.cal.card.be">Breakevens</div><div class="value">${be}</div></div>
    </div></div>`;
    applyUiI18n(el);
}
