// Expense ratio drag — dollars a fund's expense ratio costs over a horizon
// vs a zero-fee fund (gross vs net-return future value), via
// /calc/expense-drag. Updates live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const FIELDS = [
    ['initial_investment_usd', 'Initial investment ($)', 100000],
    ['annual_contribution_usd', 'Annual contribution ($)', 6000],
    ['years', 'Years', 30],
    ['gross_return_pct', 'Gross return (%)', 7],
    ['expense_ratio_pct', 'Expense ratio (%)', 1],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { maximumFractionDigits: 0 });

export async function renderExpenseDrag(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.erd.h1.title">// EXPENSE RATIO DRAG</span></h1>
        <p class="muted small" data-i18n="view.erd.hint.intro">
            A fund's expense ratio looks tiny, but it's charged on assets every year and the fee
            dollars never compound — so over decades the gap versus a zero-fee fund is far larger
            than the headline percentage. This grows the same contributions at the gross return
            vs the net return (gross − expense ratio) and shows the dollars lost to fees. Updates
            as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.erd.h2.inputs">The fund</h2>
            <form id="erd-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.erd.label.${key}">${label}</span>
                        <input type="number" step="0.01" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
            </form>
        </div>
        <div id="erd-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#erd-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        try {
            const r = await api.calcExpenseDrag(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.erd.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#erd-result');
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.erd.h2.result">The cost of fees</h2>
            <div class="cards">
                <div class="card neg"><div class="label" data-i18n="view.erd.card.drag">Lost to fees</div>
                    <div class="value neg">${money(r.fee_drag_usd)}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.erd.card.dragpct">% of no-fee value</div>
                    <div class="value neg">${Number(r.fee_drag_pct).toFixed(1)}%</div></div>
                <div class="card"><div class="label" data-i18n="view.erd.card.net">Net return</div>
                    <div class="value">${Number(r.net_return_pct).toFixed(2)}%</div></div>
            </div>
            <table class="data-table">
                <thead><tr><th data-i18n="view.erd.col.line">Ending value</th><th data-i18n="view.erd.col.amount">Amount</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.erd.row.gross">No fees (gross return)</td><td class="pos">${money(r.gross_ending_usd)}</td></tr>
                    <tr><td data-i18n="view.erd.row.net">After expense ratio</td><td>${money(r.net_ending_usd)}</td></tr>
                    <tr class="emph"><td data-i18n="view.erd.row.drag">Fee drag</td><td class="neg">${money(r.fee_drag_usd)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
