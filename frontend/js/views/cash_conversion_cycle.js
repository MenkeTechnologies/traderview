// Cash conversion cycle — DSO + DIO − DPO and the operating cycle, the days
// cash is tied up between paying suppliers and collecting from customers, via
// /calc/cash-conversion-cycle. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const FIELDS = [
    ['accounts_receivable_usd', 'Accounts receivable ($)', 50000],
    ['annual_revenue_usd', 'Annual revenue ($)', 365000],
    ['inventory_usd', 'Inventory ($)', 30000],
    ['annual_cogs_usd', 'Annual COGS ($)', 219000],
    ['accounts_payable_usd', 'Accounts payable ($)', 24000],
    ['period_days', 'Period (days)', 365],
];

const days = (n) => Number(n).toLocaleString(undefined, { maximumFractionDigits: 1 }) + ' days';

export async function renderCashConversionCycle(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ccc.h1.title">// CASH CONVERSION CYCLE</span></h1>
        <p class="muted small" data-i18n="view.ccc.hint.intro">
            How many days a dollar is tied up in operations. DSO is how long customers take
            to pay (receivables ÷ revenue × period); DIO is how long inventory sits (inventory
            ÷ COGS × period); DPO is how long you take to pay suppliers (payables ÷ COGS ×
            period). CCC = DSO + DIO − DPO. A negative cycle is excellent — you collect before
            you pay, so growth funds itself. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.ccc.h2.inputs">The balance sheet</h2>
            <form id="ccc-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.ccc.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="ccc-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#ccc-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcCashConversionCycle(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.ccc.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#ccc-result');
    const cccCls = r.self_financing ? 'pos' : r.cash_conversion_cycle_days > 0 ? 'neg' : '';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.ccc.h2.result">The cycle</h2>
            <div class="cards">
                <div class="card ${cccCls}"><div class="label" data-i18n="view.ccc.card.ccc">Cash conversion cycle</div>
                    <div class="value ${cccCls}">${days(r.cash_conversion_cycle_days)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ccc.card.operating">Operating cycle</div>
                    <div class="value">${days(r.operating_cycle_days)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ccc.card.dso">DSO (collect)</div>
                    <div class="value">${days(r.dso_days)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ccc.card.dio">DIO (inventory)</div>
                    <div class="value">${days(r.dio_days)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ccc.card.dpo">DPO (pay)</div>
                    <div class="value">${days(r.dpo_days)}</div></div>
            </div>
            ${r.self_financing ? `<p class="muted small pos" data-i18n="view.ccc.note.self">Negative cycle — suppliers finance your operations; growth is self-funding.</p>` : ''}
        </div>
    `;
    applyUiI18n(el);
}
