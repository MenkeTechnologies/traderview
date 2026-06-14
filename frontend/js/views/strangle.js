// Strangle — max profit/loss, breakevens, profit-zone width, via /calc/strangle.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => (n == null ? '∞' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));

export async function renderStrangle(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.strangle.h1.title">// STRANGLE</span></h1>
        <p class="muted small" data-i18n="view.strangle.hint.intro">
            A strangle buys (or sells) an out-of-the-money put and call at different strikes. It computes the
            breakevens, the profit-zone width, and the max profit/loss — unbounded for a long strangle, capped
            at the credit for a short strangle. Use a positive contract count for long, negative for short.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.strangle.h2.inputs">Position</h2>
            <form id="strangle-form" class="inline-form">
                <label><span data-i18n="view.strangle.label.put">Put strike ($)</span>
                    <input type="number" step="0.5" min="0" name="put_strike" value="95" required></label>
                <label><span data-i18n="view.strangle.label.call">Call strike ($)</span>
                    <input type="number" step="0.5" min="0" name="call_strike" value="105" required></label>
                <label><span data-i18n="view.strangle.label.premium">Net premium/contract ($)</span>
                    <input type="number" step="0.01" min="0" name="net_premium_per_contract" value="4" required></label>
                <label><span data-i18n="view.strangle.label.contracts">Contracts (+long/−short)</span>
                    <input type="number" step="1" name="contracts" value="1" required></label>
                <label><span data-i18n="view.strangle.label.mult">Multiplier</span>
                    <input type="number" step="1" min="1" name="multiplier" value="100"></label>
            </form>
        </div>
        <div id="strangle-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);
    const form = mount.querySelector('#strangle-form');
    const n = (k) => Number(form.querySelector(`[name="${k}"]`).value) || 0;
    const generate = async () => {
        const body = { put_strike: n('put_strike'), call_strike: n('call_strike'), net_premium_per_contract: n('net_premium_per_contract'), contracts: n('contracts'), multiplier: n('multiplier') || 100 };
        try {
            const doc = await api.calcStrangle(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) { showToast(err.message || t('view.strangle.toast.error'), { level: 'error' }); }
    };
    const live = debounce(generate, 250);
    form.addEventListener('input', () => { live(); });
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, doc) {
    const el = mount.querySelector('#strangle-result');
    if (!doc) { el.innerHTML = `<p class="muted" data-i18n="view.strangle.invalid">Invalid inputs.</p>`; applyUiI18n(el); return; }
    const dirKey = doc.is_long ? 'view.strangle.long' : 'view.strangle.short';
    el.innerHTML = `
        <div class="lpv-bar"><div class="cards">
            <div class="card"><div class="label" data-i18n="view.strangle.card.dir">Direction</div>
                <div class="value" data-i18n="${dirKey}">${doc.is_long ? 'Long' : 'Short'}</div></div>
            <div class="card pos"><div class="label" data-i18n="view.strangle.card.maxp">Max profit</div>
                <div class="value">${money(doc.max_profit)}</div></div>
            <div class="card neg"><div class="label" data-i18n="view.strangle.card.maxl">Max loss</div>
                <div class="value">${money(doc.max_loss)}</div></div>
            <div class="card"><div class="label" data-i18n="view.strangle.card.be">Breakevens</div>
                <div class="value">${money(doc.lower_breakeven)} / ${money(doc.upper_breakeven)}</div></div>
            <div class="card"><div class="label" data-i18n="view.strangle.card.width">Profit-zone width</div>
                <div class="value">${money(doc.profit_zone_width)}</div></div>
        </div></div>
    `;
    applyUiI18n(el);
}
