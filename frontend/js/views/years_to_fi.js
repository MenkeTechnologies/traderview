// Years to FI — savings rate, FI number (expenses / SWR), and the years for
// current savings + the annual surplus to reach it, via /calc/years-to-fi.
// Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const FIELDS = [
    ['current_savings_usd', 'Current savings ($)', 50000],
    ['annual_income_usd', 'Annual income ($)', 80000],
    ['annual_expenses_usd', 'Annual expenses ($)', 40000],
    ['annual_return_pct', 'Expected real return (%)', 5],
    ['safe_withdrawal_rate_pct', 'Safe withdrawal rate (%)', 4],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });

export async function renderYearsToFi(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ytf.h1.title">// YEARS TO FINANCIAL INDEPENDENCE</span></h1>
        <p class="muted small" data-i18n="view.ytf.hint.intro">
            FI is when your portfolio funds your spending indefinitely at a safe withdrawal rate
            — the FI number = annual expenses ÷ SWR (4% → 25× expenses). Starting from current
            savings and adding the income-minus-expenses gap each year (grown at your return),
            this finds the years to reach it. The dominant lever is the savings rate, far more
            than the return. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.ytf.h2.inputs">Your finances</h2>
            <form id="ytf-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.ytf.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="ytf-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#ytf-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcYearsToFi(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.ytf.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#ytf-result');
    let yearsVal, yearsCls;
    if (r.already_fi) {
        yearsVal = t('view.ytf.already');
        yearsCls = 'pos';
    } else if (r.years_to_fi == null) {
        yearsVal = t('view.ytf.never');
        yearsCls = 'neg';
    } else {
        yearsVal = `${r.years_to_fi} ${t('view.ytf.years')}`;
        yearsCls = r.years_to_fi <= 15 ? 'pos' : r.years_to_fi <= 30 ? '' : 'neg';
    }
    const srCls = r.savings_rate_pct >= 50 ? 'pos' : r.savings_rate_pct >= 20 ? '' : 'neg';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.ytf.h2.result">The timeline</h2>
            <div class="cards">
                <div class="card ${yearsCls}"><div class="label" data-i18n="view.ytf.card.years">Years to FI</div>
                    <div class="value ${yearsCls}">${yearsVal}</div></div>
                <div class="card ${srCls}"><div class="label" data-i18n="view.ytf.card.rate">Savings rate</div>
                    <div class="value ${srCls}">${Number(r.savings_rate_pct).toFixed(1)}%</div></div>
                <div class="card"><div class="label" data-i18n="view.ytf.card.finumber">FI number</div>
                    <div class="value">${money(r.fi_number_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ytf.card.savings">Annual savings</div>
                    <div class="value">${money(r.annual_savings_usd)}</div></div>
            </div>
        </div>
    `;
    applyUiI18n(el);
}
