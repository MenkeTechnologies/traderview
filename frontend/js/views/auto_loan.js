// Auto loan calculator. Vehicle price + down + trade + tax + APR + term →
// monthly P+I + total interest + full amortization schedule.

import { api } from '../api.js';
import { esc } from '../util.js';
import { t } from '../i18n.js';
import * as enh from '../calc_enhance.js';

export async function renderAutoLoan(mount, _state) {
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.auto_loan.title">// AUTO LOAN CALCULATOR</span></h1>
        <p class="muted small" data-i18n-html="view.auto_loan.intro">
            Standard fixed-rate amortization for a vehicle loan: monthly P+I from
            closed-form annuity equation, plus full month-by-month amortization schedule.
            Sales tax can be rolled into the loan or paid at signing. Principal financed
            = (vehicle price + tax if rolled) − down payment − trade-in credit.
        </p>
        <div class="chart-panel">
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:8px;margin-bottom:12px">
                <label>
                    <span class="muted small" data-i18n="view.auto_loan.field.price">Vehicle price $</span>
                    <input type="number" id="al-price" step="500" min="0" value="35000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.auto_loan.field.down">Down payment $</span>
                    <input type="number" id="al-down" step="500" min="0" value="5000" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.auto_loan.field.trade">Trade-in credit $</span>
                    <input type="number" id="al-trade" step="500" min="0" value="0" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.auto_loan.field.tax">Sales tax %</span>
                    <input type="number" id="al-tax" step="0.25" min="0" max="30" value="8" style="width:100%">
                </label>
                <label style="display:flex;align-items:center;gap:6px">
                    <input type="checkbox" id="al-tax-paid"> <span data-i18n="view.auto_loan.field.tax_paid">Tax paid at signing</span>
                </label>
                <label>
                    <span class="muted small" data-i18n="view.auto_loan.field.apr">APR %</span>
                    <input type="number" id="al-apr" step="0.25" min="0" max="50" value="6.5" style="width:100%">
                </label>
                <label>
                    <span class="muted small" data-i18n="view.auto_loan.field.term">Term months</span>
                    <input type="number" id="al-term" step="12" min="1" max="120" value="60" style="width:100%">
                </label>
            </div>
            <button class="btn btn-sm primary" id="al-run" data-shortcut="r" data-i18n="view.auto_loan.btn.run">⚡ Compute Loan</button>
            <div id="al-result"></div>
        </div>
    `;
    mount.querySelector('#al-run').addEventListener('click', () => runCompute(mount));
    await runCompute(mount);
}

async function runCompute(mount) {
    const result = mount.querySelector('#al-result');
    const input = {
        vehicle_price_usd: parseFloat(mount.querySelector('#al-price').value) || 0,
        down_payment_usd: parseFloat(mount.querySelector('#al-down').value) || 0,
        trade_in_credit_usd: parseFloat(mount.querySelector('#al-trade').value) || 0,
        sales_tax_pct: parseFloat(mount.querySelector('#al-tax').value) || 0,
        tax_paid_at_signing: mount.querySelector('#al-tax-paid').checked,
        apr_pct: parseFloat(mount.querySelector('#al-apr').value) || 0,
        term_months: parseInt(mount.querySelector('#al-term').value, 10) || 60,
    };
    result.innerHTML = `<p class="muted">${esc(t('view.auto_loan.status.computing'))}</p>`;
    try {
        const r = await api.request('/auto-loan/compute', { method: 'POST', body: JSON.stringify(input) });
        // Balance-over-time curve straight from the returned amortization schedule.
        const chart = enh.svgLineChart((r.schedule || []).map(m => ({ x: m.month, y: m.balance_usd })), { xlabel: 'month', ylabel: 'balance $' });
        result.innerHTML = `
            <div style="display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px;margin-top:1rem">
                <div><div class="muted small">${esc(t('view.auto_loan.field.principal'))}</div>
                    <strong>$${r.principal_financed_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.auto_loan.field.monthly'))}</div>
                    <strong style="font-size:1.4em">$${r.monthly_payment_usd.toFixed(2)}/mo</strong></div>
                <div><div class="muted small">${esc(t('view.auto_loan.field.total_payments'))}</div>
                    <strong>$${r.total_payments_usd.toFixed(0)}</strong></div>
                <div><div class="muted small">${esc(t('view.auto_loan.field.total_interest'))}</div>
                    <strong class="neg">$${r.total_interest_usd.toFixed(0)}</strong></div>
            </div>
            ${chart}
            <div id="al-tools" class="ce-toolbar"></div>
            <h2 style="margin-top:1rem">${esc(t('view.auto_loan.h2.schedule'))}</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.auto_loan.th.month">Month</th>
                    <th data-i18n="view.auto_loan.th.payment">Payment $</th>
                    <th data-i18n="view.auto_loan.th.principal">Principal $</th>
                    <th data-i18n="view.auto_loan.th.interest">Interest $</th>
                    <th data-i18n="view.auto_loan.th.balance">Balance $</th>
                </tr></thead>
                <tbody>${(r.schedule || []).map(m => `
                    <tr>
                        <td>${m.month}</td>
                        <td>$${m.payment_usd.toFixed(2)}</td>
                        <td class="pos">$${m.principal_usd.toFixed(2)}</td>
                        <td class="neg">$${m.interest_usd.toFixed(2)}</td>
                        <td>$${m.balance_usd.toFixed(2)}</td>
                    </tr>
                `).join('')}</tbody>
            </table>
        `;
        // Export the full amortization schedule (Copy / CSV). No permalink — this
        // view uses id-based inputs that the hash-prefill helper cannot target.
        enh.mountToolbar(mount.querySelector('#al-tools'), {
            viewId: 'auto-loan',
            link: false,
            filename: 'auto-loan-schedule.csv',
            getRows: () => [['month', 'payment', 'principal', 'interest', 'balance'],
                ...(r.schedule || []).map(m => [m.month, m.payment_usd, m.principal_usd, m.interest_usd, m.balance_usd])],
        });
    } catch (e) {
        result.innerHTML = `<p class="neg">${esc(String(e))}</p>`;
    }
}
