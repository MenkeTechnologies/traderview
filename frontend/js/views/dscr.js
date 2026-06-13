// Debt-Service Coverage Ratio — NOI over annual debt service, with the max
// loan that still clears a target DSCR, via /calc/dscr. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const ratio = (n) => Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 3 });
const pct = (n) => (Number(n) * 100).toLocaleString(undefined, { maximumFractionDigits: 1 }) + '%';

export async function renderDscr(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.dscr.h1.title">// DSCR</span></h1>
        <p class="muted small" data-i18n="view.dscr.hint.intro">
            The debt-service coverage ratio is how rental and commercial lenders size a loan: net
            operating income divided by annual debt service. At or above the lender's minimum
            (commonly 1.20–1.25) the income covers the payments with a cushion. This also sizes the
            largest loan that still clears your target DSCR. Updates as you type.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.dscr.h2.inputs">The deal</h2>
            <form id="dscr-form" class="inline-form">
                <label><span data-i18n="view.dscr.label.noi">Net operating income / yr ($)</span>
                    <input type="number" step="0.01" min="0" name="noi_usd" value="60000" required></label>
                <label><span data-i18n="view.dscr.label.loan">Loan amount ($)</span>
                    <input type="number" step="0.01" min="0" name="loan_amount_usd" value="600000" required></label>
                <label><span data-i18n="view.dscr.label.rate">Interest rate (%)</span>
                    <input type="number" step="0.001" min="0" name="interest_rate_pct" value="7" required></label>
                <label><span data-i18n="view.dscr.label.amort">Amortization (years)</span>
                    <input type="number" step="1" min="1" name="amortization_years" value="30" required></label>
                <label><span data-i18n="view.dscr.label.freq">Payments / year</span>
                    <input type="number" step="1" min="1" name="payments_per_year" value="12" required></label>
                <label><span data-i18n="view.dscr.label.target">Target DSCR</span>
                    <input type="number" step="0.01" min="0" name="target_dscr" value="1.25" required></label>
            </form>
        </div>
        <div id="dscr-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#dscr-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            noi_usd: Number(fd.get('noi_usd')) || 0,
            loan_amount_usd: Number(fd.get('loan_amount_usd')) || 0,
            interest_rate_pct: Number(fd.get('interest_rate_pct')) || 0,
            amortization_years: Number(fd.get('amortization_years')) || 0,
            payments_per_year: Number(fd.get('payments_per_year')) || 12,
            target_dscr: Number(fd.get('target_dscr')) || 0,
        };
        try {
            const r = await api.calcDscr(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, r);
        } catch (err) {
            showToast(err.message || t('view.dscr.toast.error'), { level: 'error' });
        }
    };
    form.addEventListener('input', debounce(generate, 250));
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function renderResult(mount, r) {
    const el = mount.querySelector('#dscr-result');
    const verdictClass = r.meets_target ? 'pos' : 'neg';
    const verdictKey = r.meets_target ? 'view.dscr.verdict.pass' : 'view.dscr.verdict.fail';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.dscr.h2.result">The coverage</h2>
            <div class="cards">
                <div class="card ${verdictClass}"><div class="label" data-i18n="view.dscr.card.dscr">DSCR</div>
                    <div class="value ${verdictClass}">${ratio(r.dscr)}</div></div>
                <div class="card"><div class="label" data-i18n="view.dscr.card.cashflow">Annual cash flow</div>
                    <div class="value">${money(r.annual_cash_flow_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.dscr.card.maxloan">Max loan at target</div>
                    <div class="value">${money(r.max_loan_at_target_usd)}</div></div>
            </div>
            <p class="${verdictClass}" data-i18n="${verdictKey}">${r.meets_target ? 'Clears the target DSCR.' : 'Below the target DSCR.'}</p>
            <table class="data-table">
                <tbody>
                    <tr><td data-i18n="view.dscr.row.payment">Periodic payment</td><td>${money(r.periodic_payment_usd)}</td></tr>
                    <tr><td data-i18n="view.dscr.row.annualds">Annual debt service</td><td>${money(r.annual_debt_service_usd)}</td></tr>
                    <tr><td data-i18n="view.dscr.row.maxds">Max debt service at target</td><td>${money(r.max_annual_debt_service_usd)}</td></tr>
                    <tr><td data-i18n="view.dscr.row.cushion">NOI cushion to target</td><td>${pct(r.noi_cushion_fraction)}</td></tr>
                    <tr class="emph"><td data-i18n="view.dscr.row.dscr">DSCR</td><td>${ratio(r.dscr)}</td></tr>
                </tbody>
            </table>
        </div>
    `;
    applyUiI18n(el);
}
