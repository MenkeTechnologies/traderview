// Rental total return — the four-component decomposition (cash flow,
// appreciation, loan paydown, depreciation tax shield) on cash invested, via
// /calc/rental-total-return. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';
import * as enh from '../calc_enhance.js';

const money = (n) => (n == null ? '—' : (n < 0 ? '−$' : '$') + Math.abs(Number(n)).toLocaleString(undefined, { maximumFractionDigits: 0 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
const VIEW = 'rental-total-return';
let lastReport = null;
let lastBody = null;

export async function renderRentalTotalReturn(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.rentret.h1.title">// RENTAL TOTAL RETURN</span></h1>
        <p class="muted small" data-i18n="view.rentret.hint.intro">
            A leveraged rental builds wealth four ways in a year: cash flow (NOI less debt service),
            appreciation, the loan principal the rent retires, and the depreciation tax shield. Summed
            over the cash invested, the total return runs well above the cap rate. 27.5-year straight-line
            depreciation on the building. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.rentret.h2.inputs">The deal</h2>
            <form id="rentret-form" class="inline-form">
                <label><span data-i18n="view.rentret.label.price">Purchase price ($)</span>
                    <input type="number" step="1000" min="0" name="purchase_price_usd" value="300000" required></label>
                <label><span data-i18n="view.rentret.label.down">Down payment ($)</span>
                    <input type="number" step="1000" min="0" name="down_payment_usd" value="60000" required></label>
                <label><span data-i18n="view.rentret.label.closing">Closing costs ($)</span>
                    <input type="number" step="500" min="0" name="closing_costs_usd" value="6000"></label>
                <label><span data-i18n="view.rentret.label.rate">Loan rate (%)</span>
                    <input type="number" step="0.01" min="0" name="loan_rate_pct" value="7" required></label>
                <label><span data-i18n="view.rentret.label.term">Loan term (years)</span>
                    <input type="number" step="1" min="1" name="loan_term_years" value="30"></label>
                <label><span data-i18n="view.rentret.label.noi">Annual NOI ($)</span>
                    <input type="number" step="500" name="annual_noi_usd" value="24000" required></label>
                <label><span data-i18n="view.rentret.label.appr">Appreciation rate (%)</span>
                    <input type="number" step="0.1" name="appreciation_rate_pct" value="3"></label>
                <label><span data-i18n="view.rentret.label.tax">Marginal tax rate (%)</span>
                    <input type="number" step="1" min="0" name="marginal_tax_rate_pct" value="24"></label>
                <label><span data-i18n="view.rentret.label.land">Land value (% of price)</span>
                    <input type="number" step="1" min="0" max="100" name="land_value_pct" value="20"></label>
            </form>
        </div>
        <div id="rentret-tools" class="ce-toolbar"></div>
        <div id="rentret-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#rentret-form');
    enh.prefillForm(form, enh.readHashInputs());
    const readBody = () => {
        const fd = new FormData(form);
        return {
            purchase_price_usd: Number(fd.get('purchase_price_usd')) || 0,
            down_payment_usd: Number(fd.get('down_payment_usd')) || 0,
            closing_costs_usd: Number(fd.get('closing_costs_usd')) || 0,
            loan_rate_pct: Number(fd.get('loan_rate_pct')) || 0,
            loan_term_years: Number(fd.get('loan_term_years')) || 30,
            annual_noi_usd: Number(fd.get('annual_noi_usd')) || 0,
            appreciation_rate_pct: Number(fd.get('appreciation_rate_pct')) || 0,
            marginal_tax_rate_pct: Number(fd.get('marginal_tax_rate_pct')) || 0,
            land_value_pct: Number(fd.get('land_value_pct')) || 0,
        };
    };
    const generate = async () => {
        const body = readBody();
        try {
            const r = await api.calcRentalTotalReturn(body);
            if (!viewIsCurrent(tok)) return;
            lastReport = r; lastBody = body;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.rentret.toast.error'), { level: 'error' });
        }
    };
    enh.mountToolbar(mount.querySelector('#rentret-tools'), { viewId: VIEW, getInputs: () => lastBody || readBody(), getRows: () => reportRows(lastReport), filename: 'rental-total-return.csv' });
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function reportRows(r) {
    if (!r) return [];
    return [
        ['metric', 'value'],
        ['total_return_pct', r.total_return_pct],
        ['total_return_usd', r.total_return_usd],
        ['cash_flow_usd', r.cash_flow_usd],
        ['appreciation_gain_usd', r.appreciation_gain_usd],
        ['principal_paydown_usd', r.principal_paydown_usd],
        ['tax_shield_usd', r.tax_shield_usd],
    ];
}

function renderResult(mount, r) {
    const el = mount.querySelector('#rentret-result');
    const cfCls = r.cash_flow_usd >= 0 ? 'pos' : 'neg';
    // Four-component return decomposition (where the total return comes from).
    const chart = enh.svgBarChart([
        { label: 'CashFlow', value: r.cash_flow_usd },
        { label: 'Apprec', value: r.appreciation_gain_usd },
        { label: 'Paydown', value: r.principal_paydown_usd },
        { label: 'TaxShield', value: r.tax_shield_usd },
    ]);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.rentret.h2.result">The return</h2>
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.rentret.card.total">Total return</div>
                    <div class="value pos">${pct(r.total_return_pct)}</div></div>
                <div class="card"><div class="label" data-i18n="view.rentret.card.totaldollars">Total return ($)</div>
                    <div class="value">${money(r.total_return_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.rentret.card.cash">Cash invested</div>
                    <div class="value">${money(r.cash_invested_usd)}</div></div>
            </div>
            ${chart}
            <table class="data-table">
                <tbody>
                    <tr class="${cfCls}"><td data-i18n="view.rentret.row.cashflow">1. Cash flow</td><td>${money(r.cash_flow_usd)} (${pct(r.cash_on_cash_pct)})</td></tr>
                    <tr><td data-i18n="view.rentret.row.appreciation">2. Appreciation</td><td>${money(r.appreciation_gain_usd)} (${pct(r.appreciation_return_pct)})</td></tr>
                    <tr><td data-i18n="view.rentret.row.paydown">3. Loan paydown</td><td>${money(r.principal_paydown_usd)} (${pct(r.paydown_return_pct)})</td></tr>
                    <tr><td data-i18n="view.rentret.row.shield">4. Depreciation tax shield</td><td>${money(r.tax_shield_usd)} (${pct(r.tax_shield_return_pct)})</td></tr>
                    <tr><td data-i18n="view.rentret.row.loan">Loan amount</td><td>${money(r.loan_amount_usd)}</td></tr>
                    <tr><td data-i18n="view.rentret.row.debt">Annual debt service</td><td>${money(r.annual_debt_service_usd)}</td></tr>
                    <tr><td data-i18n="view.rentret.row.depreciation">Annual depreciation</td><td>${money(r.annual_depreciation_usd)}</td></tr>
                    <tr class="emph pos"><td data-i18n="view.rentret.row.total">Total return on cash</td><td>${money(r.total_return_usd)} (${pct(r.total_return_pct)})</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
