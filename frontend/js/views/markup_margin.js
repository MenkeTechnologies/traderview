// Markup vs margin — from cost + one of {price, markup%, margin%}, shows
// price, profit, and both markup% (of cost) and margin% (of price), via
// /calc/markup-margin. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const pct = (n) => Number(n).toFixed(2) + '%';

export async function renderMarkupMargin(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.mm.h1.title">// MARKUP vs MARGIN</span></h1>
        <p class="muted small" data-i18n="view.mm.hint.intro">
            Markup is profit as a percent of cost; margin is the same profit as a percent of
            price. They are never equal — a 50% markup is only a 33% margin, and doubling cost
            (100% markup, "keystone") is a 50% margin. Pricing off the wrong one erodes profit.
            Pick what you know — price, markup%, or margin% — and see all four. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.mm.h2.inputs">What you know</h2>
            <form id="mm-form" class="inline-form">
                <label><span data-i18n="view.mm.label.mode">I'm entering</span>
                    <select name="mode" id="mm-mode">
                        <option value="price" data-i18n="view.mm.mode.price">Cost + price</option>
                        <option value="markup" data-i18n="view.mm.mode.markup">Cost + markup %</option>
                        <option value="margin" data-i18n="view.mm.mode.margin">Cost + margin %</option>
                    </select></label>
                <label><span data-i18n="view.mm.label.cost">Unit cost ($)</span>
                    <input type="number" step="0.01" min="0" name="cost_usd" value="60" required></label>
                <label id="mm-price-wrap"><span data-i18n="view.mm.label.price">Selling price ($)</span>
                    <input type="number" step="0.01" min="0" name="price_usd" value="100"></label>
                <label id="mm-markup-wrap"><span data-i18n="view.mm.label.markup">Markup (%)</span>
                    <input type="number" step="0.01" name="markup_pct" value="66.67"></label>
                <label id="mm-margin-wrap"><span data-i18n="view.mm.label.margin">Margin (%)</span>
                    <input type="number" step="0.01" name="margin_pct" value="40"></label>
            </form>
        </div>
        <div id="mm-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#mm-form');
    const modeSel = mount.querySelector('#mm-mode');
    const wraps = {
        price: mount.querySelector('#mm-price-wrap'),
        markup: mount.querySelector('#mm-markup-wrap'),
        margin: mount.querySelector('#mm-margin-wrap'),
    };
    const syncMode = () => {
        for (const [k, el] of Object.entries(wraps)) el.style.display = modeSel.value === k ? '' : 'none';
    };

    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            mode: fd.get('mode'),
            cost_usd: Number(fd.get('cost_usd')) || 0,
            price_usd: Number(fd.get('price_usd')) || 0,
            markup_pct: Number(fd.get('markup_pct')) || 0,
            margin_pct: Number(fd.get('margin_pct')) || 0,
        };
        try {
            const r = await api.calcMarkupMargin(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.mm.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);
    modeSel.addEventListener('change', () => { syncMode(); generate(); });
    form.addEventListener('input', live);
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    syncMode();
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#mm-result');
    if (!r.feasible) {
        el.innerHTML = `<div class="chart-panel"><h2 data-i18n="view.mm.h2.result">The numbers</h2>
            <p class="muted small neg" data-i18n="view.mm.warn.infeasible">A margin of 100% or more is impossible — price would be infinite. Lower the margin.</p></div>`;
        applyUiI18n(el);
        return;
    }
    const profitCls = r.profit_usd >= 0 ? 'pos' : 'neg';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.mm.h2.result">The numbers</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.mm.card.price">Price</div>
                    <div class="value">${money(r.price_usd)}</div></div>
                <div class="card ${profitCls}"><div class="label" data-i18n="view.mm.card.profit">Profit / unit</div>
                    <div class="value ${profitCls}">${money(r.profit_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.mm.card.markup">Markup (of cost)</div>
                    <div class="value">${pct(r.markup_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.mm.card.margin">Margin (of price)</div>
                    <div class="value">${pct(r.margin_pct)}</div></div>
            </div>
        </div>
    `;
    applyUiI18n(el);
}
