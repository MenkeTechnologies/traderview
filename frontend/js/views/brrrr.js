// BRRRR — Buy, Rehab, Rent, Refinance, Repeat. Nets the cash-out refi
// against total cash invested to show the cash left in the deal and the
// post-refi cash flow, via /calc/brrrr.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import * as enh from '../calc_enhance.js';

const VIEW = 'brrrr';
let lastReport = null;
let lastBody = null;

const FIELDS = [
    ['purchase_price_usd', 'Purchase price ($)', 100000],
    ['rehab_cost_usd', 'Rehab cost ($)', 30000],
    ['purchase_closing_usd', 'Acquisition closing ($)', 5000],
    ['after_repair_value_usd', 'After-repair value / ARV ($)', 200000],
    ['refi_ltv_pct', 'Refi LTV (%)', 75],
    ['refi_apr_pct', 'Refi APR (%)', 7],
    ['refi_term_months', 'Refi term (months)', 360],
    ['refi_closing_usd', 'Refi closing ($)', 4000],
    ['monthly_rent_usd', 'Monthly rent ($)', 1800],
    ['monthly_operating_usd', 'Monthly operating ($, ex-mortgage)', 600],
];
const INT_FIELDS = new Set(['refi_term_months']);

const money = (n) => (n < 0 ? '-$' : '$') + Math.abs(Number(n)).toLocaleString(undefined, { maximumFractionDigits: 0 });

export async function renderBrrrr(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.brrrr.h1.title">// BRRRR</span></h1>
        <p class="muted small" data-i18n="view.brrrr.hint.intro">
            Buy a distressed property cheap, rehab it to force appreciation, rent it, then
            refinance against the higher after-repair value and pull your cash back out to
            do it again. The deal succeeds when the cash-out refi recovers everything you
            put in — the cash left in the deal still earns rent, so the cash-on-cash return
            goes infinite.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.brrrr.h2.inputs">The deal</h2>
            <form id="br-form" class="inline-form">
                ${FIELDS.map(([key, label, def]) => `
                    <label><span data-i18n="view.brrrr.label.${key}">${label}</span>
                        <input type="number" step="${INT_FIELDS.has(key) ? '1' : '0.01'}" min="0" name="${key}" value="${def}" required></label>
                `).join('')}
                <button class="primary" type="submit" data-i18n="view.brrrr.btn.run">Run the deal</button>
            </form>
            <div id="br-tools" class="ce-toolbar"></div>
        </div>
        <div id="br-result"></div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#br-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        const body = {};
        for (const [key] of FIELDS) body[key] = Number(fd.get(key)) || 0;
        return body;
    };
    enh.mountToolbar(mount.querySelector('#br-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'brrrr.csv' });
    form.addEventListener('submit', async (e) => {
        e.preventDefault();
        const body = readBody();
        try {
            const r = await api.calcBrrrr(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.brrrr.toast.error'), { level: 'error' });
        }
    });
    form.dispatchEvent(new Event('submit'));
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['total_cash_invested_usd', r.total_cash_invested_usd],
        ['refi_loan_usd', r.refi_loan_usd],
        ['cash_out_usd', r.cash_out_usd],
        ['cash_left_in_deal_usd', r.cash_left_in_deal_usd],
        ['equity_after_refi_usd', r.equity_after_refi_usd],
        ['monthly_cash_flow_usd', r.monthly_cash_flow_usd],
    ];
}

function renderResult(mount, r) {
    const el = mount.querySelector('#br-result');
    const leftCls = Number(r.cash_left_in_deal_usd) <= 0 ? 'pos' : '';
    // Cash invested vs cash pulled at refi vs cash left in the deal.
    const chart = enh.svgBarChart([
        { label: 'Cash in', value: r.total_cash_invested_usd },
        { label: 'Cash out', value: r.cash_out_usd },
        { label: 'Left in', value: r.cash_left_in_deal_usd },
    ]);
    const cfCls = Number(r.monthly_cash_flow_usd) >= 0 ? 'pos' : 'neg';
    const coc = r.cash_on_cash_pct == null ? t('view.brrrr.infinite') : Number(r.cash_on_cash_pct).toFixed(1) + '%';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.brrrr.h2.result">The numbers</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.brrrr.card.invested">Total cash in</div>
                    <div class="value">${money(r.total_cash_invested_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.brrrr.card.loan">Refi loan</div>
                    <div class="value">${money(r.refi_loan_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.brrrr.card.cashout">Cash pulled at refi</div>
                    <div class="value pos">${money(r.cash_out_usd)}</div></div>
                <div class="card ${leftCls ? 'pos' : ''}"><div class="label" data-i18n="view.brrrr.card.left">Cash left in deal</div>
                    <div class="value ${leftCls}">${money(r.cash_left_in_deal_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.brrrr.card.equity">Equity after refi</div>
                    <div class="value">${money(r.equity_after_refi_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.brrrr.card.pi">Refi P&amp;I</div>
                    <div class="value">${money(r.monthly_pi_usd)}/mo</div></div>
                <div class="card"><div class="label" data-i18n="view.brrrr.card.cashflow">Monthly cash flow</div>
                    <div class="value ${cfCls}">${money(r.monthly_cash_flow_usd)}/mo</div></div>
                <div class="card"><div class="label" data-i18n="view.brrrr.card.coc">Cash-on-cash</div>
                    <div class="value ${r.cash_on_cash_pct == null ? 'pos' : ''}">${coc}</div></div>
            </div>
            ${chart}
            <p class="muted small">${r.all_cash_recovered
                ? `<span class="pos" data-i18n="view.brrrr.note.recovered">The refinance recovered all your cash — an infinite-return rental you can repeat.</span>`
                : `<span data-i18n="view.brrrr.note.partial">Some cash stays in the deal; the cash-on-cash return measures what it earns.</span>`}</p>
        </div>
    `;
    applyUiI18n(el);
}
