// Burn rate & runway — gross/net burn, months of cash, and months to
// break-even given revenue growth, via /calc/burn-rate. Updates live.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const FIELDS = [
    ['cash_on_hand_usd', 'Cash on hand ($)', 100000],
    ['monthly_revenue_usd', 'Monthly revenue ($)', 5000],
    ['monthly_expenses_usd', 'Monthly expenses ($)', 10000],
    ['monthly_revenue_growth_pct', 'Revenue growth (%/mo)', 0],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });

export async function renderBurnRate(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.burn.h1.title">// BURN RATE & RUNWAY</span></h1>
        <p class="muted small" data-i18n="view.burn.hint.intro">
            How long the cash lasts. Gross burn is monthly expenses; net burn is expenses minus
            revenue — what actually drains the bank. Runway is the months of cash at that burn,
            simulated month by month so growing revenue extends it. If revenue overtakes
            expenses before the cash runs out, you reach break-even and never deplete. Updates
            as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.burn.h2.inputs">The numbers</h2>
            <form id="burn-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.burn.label.${key}">${label}</span>
                        <input type="number" step="0.01" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="burn-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#burn-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcBurnRate(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.burn.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#burn-result');
    let runwayVal, runwayCls;
    if (r.already_profitable) {
        runwayVal = t('view.burn.profitable');
        runwayCls = 'pos';
    } else if (r.runway_months == null) {
        runwayVal = t('view.burn.reaches_breakeven');
        runwayCls = 'pos';
    } else {
        runwayVal = `${r.runway_months} ${t('view.burn.months')}`;
        runwayCls = r.runway_months <= 6 ? 'neg' : r.runway_months <= 12 ? '' : 'pos';
    }
    const breakeven = r.already_profitable
        ? t('view.burn.now')
        : r.months_to_breakeven == null
            ? '—'
            : `${r.months_to_breakeven} ${t('view.burn.months')}`;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.burn.h2.result">The runway</h2>
            <div class="cards">
                <div class="card ${runwayCls}"><div class="label" data-i18n="view.burn.card.runway">Runway</div>
                    <div class="value ${runwayCls}">${runwayVal}</div></div>
                <div class="card ${r.net_burn_usd > 0 ? 'neg' : 'pos'}"><div class="label" data-i18n="view.burn.card.net">Net burn / mo</div>
                    <div class="value">${money(r.net_burn_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.burn.card.gross">Gross burn / mo</div>
                    <div class="value">${money(r.gross_burn_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.burn.card.breakeven">Months to break-even</div>
                    <div class="value">${breakeven}</div></div>
            </div>
        </div>
    `;
    applyUiI18n(el);
}
