// CD early-withdrawal penalty — interest earned minus the months-of-interest
// penalty, net proceeds, annualized yield, and principal-loss flag, via
// /calc/cd-penalty. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const FIELDS = [
    ['principal_usd', 'Principal ($)', 10000],
    ['apy_pct', 'CD rate / APY (%)', 5],
    ['months_held', 'Months held before withdrawal', 12],
    ['penalty_months', 'Penalty (months of interest)', 6],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 });

export async function renderCdPenalty(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.cdp.h1.title">// CD EARLY WITHDRAWAL</span></h1>
        <p class="muted small" data-i18n="view.cdp.hint.intro">
            Cashing out a CD before maturity triggers a penalty — usually a set number of months
            of interest. If you haven't held it long enough to earn that much, the penalty eats
            into principal. This shows the interest earned, the penalty, the net proceeds, the
            annualized yield you actually got, and whether you'd lose principal. Updates as you
            type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.cdp.h2.inputs">The CD</h2>
            <form id="cdp-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.cdp.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="cdp-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#cdp-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcCdPenalty(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.cdp.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#cdp-result');
    const netCls = r.principal_loss ? 'neg' : 'pos';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.cdp.h2.result">If you break it now</h2>
            <div class="cards">
                <div class="card ${netCls}"><div class="label" data-i18n="view.cdp.card.net">Net interest</div>
                    <div class="value ${netCls}">${money(r.net_interest_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.cdp.card.proceeds">Net proceeds</div>
                    <div class="value">${money(r.net_proceeds_usd)}</div></div>
                <div class="card ${netCls}"><div class="label" data-i18n="view.cdp.card.yield">Net annualized yield</div>
                    <div class="value ${netCls}">${Number(r.net_annualized_yield_pct).toFixed(2)}%</div></div>
            </div>
            ${r.principal_loss ? `<p class="muted small neg" data-i18n="view.cdp.warn.loss">The penalty exceeds the interest earned — withdrawing now dips into principal.</p>` : ''}
            <table class="data-table">
                <thead><tr><th data-i18n="view.cdp.col.line">Line</th><th data-i18n="view.cdp.col.amount">Amount</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.cdp.row.earned">Interest earned</td><td class="pos">${money(r.interest_earned_usd)}</td></tr>
                    <tr><td data-i18n="view.cdp.row.penalty">Early-withdrawal penalty</td><td class="neg">-${money(r.penalty_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.cdp.row.net">Net interest</td><td class="${netCls}">${money(r.net_interest_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
