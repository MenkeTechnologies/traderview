// Profit First — splits real revenue (revenue − materials/subs) across
// Profit / Owner's Pay / Tax / OpEx by the target allocation band, or custom
// percentages, via /calc/profit-first. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const FIELDS = [
    ['annual_revenue_usd', 'Annual revenue ($)', 300000],
    ['materials_subcontractors_usd', 'Materials + subcontractors ($)', 0],
    ['profit_pct', 'Custom Profit % (0 = auto)', 0],
    ['owner_pay_pct', 'Custom Owner\'s Pay %', 0],
    ['tax_pct', 'Custom Tax %', 0],
    ['opex_pct', 'Custom OpEx %', 0],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });
const pct = (n) => Number(n).toFixed(0) + '%';

export async function renderProfitFirst(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.pf.h1.title">// PROFIT FIRST ALLOCATION</span></h1>
        <p class="muted small" data-i18n="view.pf.hint.intro">
            Take profit off the top: Sales − Profit = Expenses, not the other way around. Real
            revenue (revenue minus materials and subcontractors) is split across four accounts
            — Profit, Owner's Pay, Tax, Operating Expenses — by the target allocation band for
            your revenue size (under $250K starts at 5/50/15/30; the profit share rises and
            owner's pay falls as you grow). Leave the custom fields at 0 to auto-select the
            band, or enter four that sum to 100. The business-side savings waterfall.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.pf.h2.inputs">Your revenue</h2>
            <form id="pf-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.pf.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="pf-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#pf-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcProfitFirst(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.pf.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#pf-result');
    const row = (labelKey, p, usd, cls) => `
        <tr><td data-i18n="${labelKey}"></td><td>${pct(p)}</td><td class="${cls}">${money(usd)}</td></tr>`;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.pf.h2.result">The allocation</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.pf.card.real">Real revenue</div>
                    <div class="value">${money(r.real_revenue_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.pf.card.band">Allocation band</div>
                    <div class="value">${r.band === 'Custom' ? t('view.pf.custom') : r.band}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.pf.card.profit">Profit (off the top)</div>
                    <div class="value pos">${money(r.profit_usd)}</div></div>
            </div>
            <table class="data-table">
                <thead><tr>
                    <th data-i18n="view.pf.col.account">Account</th>
                    <th data-i18n="view.pf.col.pct">%</th>
                    <th data-i18n="view.pf.col.amount">Allocation</th>
                </tr></thead>
                <tbody>
                    ${row('view.pf.acct.profit', r.profit_pct, r.profit_usd, 'pos')}
                    ${row('view.pf.acct.owner', r.owner_pay_pct, r.owner_pay_usd, '')}
                    ${row('view.pf.acct.tax', r.tax_pct, r.tax_usd, '')}
                    ${row('view.pf.acct.opex', r.opex_pct, r.opex_usd, '')}
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
