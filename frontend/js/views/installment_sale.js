// § 453 Installment Sale Tracker.
// Recognize gain pro-rata as payments are received (vs all up front).
// Gross profit % = (sale price - basis) / contract price.
// Each payment: gain portion = payment × gross profit %.
// Above $150k installment receivables: interest charge on deferred tax.
// Cannot use for inventory or marketable securities.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const INTEREST_CHARGE_THRESHOLD = 150_000;
const AFR_LONG_TERM = 0.045;  // approximate

let state = {
    sale_price: 500_000,
    basis: 100_000,
    selling_expenses: 10_000,
    down_payment: 100_000,
    payment_term_years: 5,
    annual_payment: 100_000,
    sale_year: new Date().getFullYear(),
    lt_cap_gains_rate: 0.20,
    niit: 0.038,
};

export async function renderInstallmentSale(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s453.h1.title">// § 453 INSTALLMENT SALE</span></h1>
        <p class="muted small" data-i18n="view.s453.hint.intro">
            <strong>§ 453:</strong> recognize gain pro-rata as payments come in.
            Gross profit % = (sale price − basis − selling expenses) / contract price.
            Each payment's gain = payment × gross profit %. Receivables &gt; $150k:
            interest-charge on deferred tax. NOT available for inventory or marketable
            securities. Useful for real estate + business sales.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s453.h2.inputs">Inputs</h2>
            <form id="is-form" class="inline-form">
                <label><span data-i18n="view.s453.label.sale_price">Sale price ($)</span>
                    <input type="number" step="1000" name="sale_price" value="${state.sale_price}"></label>
                <label><span data-i18n="view.s453.label.basis">Cost basis ($)</span>
                    <input type="number" step="1000" name="basis" value="${state.basis}"></label>
                <label><span data-i18n="view.s453.label.selling_expenses">Selling expenses ($)</span>
                    <input type="number" step="100" name="selling_expenses" value="${state.selling_expenses}"></label>
                <label><span data-i18n="view.s453.label.down_payment">Down payment ($)</span>
                    <input type="number" step="1000" name="down_payment" value="${state.down_payment}"></label>
                <label><span data-i18n="view.s453.label.term_years">Term (years)</span>
                    <input type="number" step="1" name="payment_term_years" value="${state.payment_term_years}" min="1" max="30"></label>
                <label><span data-i18n="view.s453.label.annual_payment">Annual payment ($)</span>
                    <input type="number" step="1000" name="annual_payment" value="${state.annual_payment}"></label>
                <label><span data-i18n="view.s453.label.sale_year">Sale year</span>
                    <input type="number" step="1" name="sale_year" value="${state.sale_year}"></label>
                <label><span data-i18n="view.s453.label.lt_cap_gains_rate">LT cap-gains rate %</span>
                    <input type="number" step="0.5" name="lt_cap_gains_rate" value="${(state.lt_cap_gains_rate * 100).toFixed(1)}"></label>
                <label><span data-i18n="view.s453.label.niit">NIIT %</span>
                    <input type="number" step="0.1" name="niit" value="${(state.niit * 100).toFixed(1)}"></label>
                <button class="primary" type="submit" data-i18n="view.s453.btn.compute">Compute</button>
            </form>
        </div>
        <div id="is-output"></div>
    `;
    document.getElementById('is-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.sale_price = Number(fd.get('sale_price')) || 0;
        state.basis = Number(fd.get('basis')) || 0;
        state.selling_expenses = Number(fd.get('selling_expenses')) || 0;
        state.down_payment = Number(fd.get('down_payment')) || 0;
        state.payment_term_years = Number(fd.get('payment_term_years')) || 1;
        state.annual_payment = Number(fd.get('annual_payment')) || 0;
        state.sale_year = Number(fd.get('sale_year'));
        state.lt_cap_gains_rate = (Number(fd.get('lt_cap_gains_rate')) || 20) / 100;
        state.niit = (Number(fd.get('niit')) || 3.8) / 100;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('is-output');
    if (!el) return;
    const adjustedBasis = state.basis + state.selling_expenses;
    const totalGain = state.sale_price - adjustedBasis;
    const grossProfitPct = state.sale_price > 0 ? totalGain / state.sale_price : 0;
    const fullPaymentTax = totalGain * (state.lt_cap_gains_rate + state.niit);

    // Build schedule
    const schedule = [];
    let remainingPrincipal = state.sale_price - state.down_payment;
    schedule.push({
        year: state.sale_year,
        payment: state.down_payment,
        gain_recognized: state.down_payment * grossProfitPct,
        tax: state.down_payment * grossProfitPct * (state.lt_cap_gains_rate + state.niit),
        principal_remaining: remainingPrincipal,
    });
    for (let y = 1; y <= state.payment_term_years; y++) {
        const payment = Math.min(state.annual_payment, remainingPrincipal);
        if (payment <= 0) break;
        remainingPrincipal -= payment;
        const gain = payment * grossProfitPct;
        schedule.push({
            year: state.sale_year + y,
            payment,
            gain_recognized: gain,
            tax: gain * (state.lt_cap_gains_rate + state.niit),
            principal_remaining: remainingPrincipal,
        });
    }
    const totalRecognized = schedule.reduce((s, r) => s + r.gain_recognized, 0);
    const totalTax = schedule.reduce((s, r) => s + r.tax, 0);
    const receivablesYr1End = state.sale_price - state.down_payment;
    const interestChargeApplies = receivablesYr1End > INTEREST_CHARGE_THRESHOLD;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s453.h2.summary">Tax outcome</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s453.card.total_gain">Total gain</div>
                    <div class="value">$${totalGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s453.card.gross_profit">Gross profit %</div>
                    <div class="value">${(grossProfitPct * 100).toFixed(1)}%</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s453.card.full_year_tax">If reported all up front</div>
                    <div class="value">$${fullPaymentTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s453.card.installment_tax">Installment total tax</div>
                    <div class="value">$${totalTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s453.card.deferred_pv">Deferral value (smoothing)</div>
                    <div class="value">$${(fullPaymentTax - schedule[0].tax).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${interestChargeApplies ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s453.card.interest_charge">Interest charge applies?</div>
                    <div class="value">${interestChargeApplies ? esc(t('view.s453.status.yes')) : esc(t('view.s453.status.no'))}</div>
                </div>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s453.h2.schedule">Payment schedule</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s453.th.year">Year</th>
                    <th data-i18n="view.s453.th.payment">Payment received</th>
                    <th data-i18n="view.s453.th.gain">Gain recognized</th>
                    <th data-i18n="view.s453.th.tax">Tax</th>
                    <th data-i18n="view.s453.th.principal_remaining">Principal remaining</th>
                </tr></thead>
                <tbody>${schedule.map(r => `
                    <tr>
                        <td>${r.year}</td>
                        <td>$${r.payment.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td class="neg">$${r.gain_recognized.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td class="neg">$${r.tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td>$${r.principal_remaining.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s453.h2.cautions">Cautions</h2>
            <ul class="muted small">
                <li data-i18n="view.s453.caution.depreciation_recap">Depreciation recapture (§ 1245/1250) recognized FULLY in year of sale, NOT installment</li>
                <li data-i18n="view.s453.caution.related_party">Related-party rule: buyer reselling within 2 years triggers full recognition</li>
                <li data-i18n="view.s453.caution.no_inventory">No installment treatment for inventory or marketable securities</li>
                <li data-i18n="view.s453.caution.election_out">Can elect OUT of installment (report all up front) — helpful in loss years</li>
                <li data-i18n="view.s453.caution.interest_charge">Receivables &gt; $150k: § 453A interest charge on deferred tax at AFR-ish rate</li>
            </ul>
        </div>
    `;
}
