// Price-to-rent ratio — home price ÷ annual rent and gross rental yield,
// with the buy/borderline/rent verdict, via /calc/price-to-rent. Live.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const FIELDS = [
    ['home_price_usd', 'Home price ($)', 400000],
    ['monthly_rent_usd', 'Comparable monthly rent ($)', 2000],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });
const VERDICT = {
    'favors buying': ['Favors buying (<15)', 'pos'],
    borderline: ['Borderline (15–20)', ''],
    'favors renting': ['Favors renting (>20)', 'neg'],
    'n/a': ['—', ''],
};

export async function renderPriceToRent(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ptr.h1.title">// PRICE-TO-RENT RATIO</span></h1>
        <p class="muted small" data-i18n="view.ptr.hint.intro">
            A quick read on whether a market favors buying or renting: home price ÷ annual rent
            for a comparable home. Under 15 generally favors buying, 15–20 is borderline, over
            20 favors renting (the home is pricey relative to what it rents for). The gross
            rental yield is the inverse. A screen, not a full rent-vs-buy NPV. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.ptr.h2.inputs">The market</h2>
            <form id="ptr-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.ptr.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="ptr-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#ptr-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcPriceToRent(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.ptr.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#ptr-result');
    const [verdictLabel, verdictCls] = VERDICT[r.verdict] || [r.verdict, ''];
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.ptr.h2.result">The verdict</h2>
            <div class="cards">
                <div class="card ${verdictCls}"><div class="label" data-i18n="view.ptr.card.ratio">Price-to-rent</div>
                    <div class="value ${verdictCls}">${Number(r.price_to_rent_ratio).toFixed(1)}</div></div>
                <div class="card ${verdictCls}"><div class="label" data-i18n="view.ptr.card.verdict">Verdict</div>
                    <div class="value ${verdictCls}">${verdictLabel}</div></div>
                <div class="card"><div class="label" data-i18n="view.ptr.card.yield">Gross rental yield</div>
                    <div class="value">${Number(r.gross_rental_yield_pct).toFixed(2)}%</div></div>
                <div class="card"><div class="label" data-i18n="view.ptr.card.annual">Annual rent</div>
                    <div class="value">${money(r.annual_rent_usd)}</div></div>
            </div>
        </div>
    `;
    applyUiI18n(el);
}
