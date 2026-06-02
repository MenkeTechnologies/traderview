// IRC § 6050W — Third-Party Network Reporting (1099-K).
// Payment Settlement Entity (PSE) reports gross payments via TPSO or merchant card.
// ARPA 2021 threshold $600 — delayed to $20K + 200 txns (2024 partial $5K phasein, $2.5K 2025).
// Affects: PayPal, Venmo, Cash App, Etsy, eBay, Stripe, Square, Airbnb sellers.
// No expense deduction on form — taxpayer reconciles cost basis / business expenses.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    gross_payments_received: 0,
    transaction_count: 0,
    tax_year: 2024,
    payee_type: 'individual',
    is_business: false,
    is_personal_transfer: false,
    cost_of_goods: 0,
    business_expenses: 0,
    sale_was_loss: false,
    pse_type: 'tpso',
    backup_withholding_applied: false,
    state_threshold_lower: false,
    correction_filed: false,
};

export async function renderSection6050W(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6050W.h1.title">// § 6050W 1099-K REPORTING</span></h1>
        <p class="muted small" data-i18n="view.s6050W.hint.intro">
            <strong>PSE</strong> (Payment Settlement Entity) reports gross payments via <strong>Form 1099-K</strong>.
            <strong>ARPA 2021 threshold</strong> $600 — Notice 2023-74 + 2024-85 delayed phase-in: <strong>$20K
            + 200 txns</strong> (2023, 2024 baseline) → <strong>$5K</strong> (2024) → <strong>$2.5K</strong> (2025)
            → <strong>$600</strong> (2026). Affects: <strong>PayPal, Venmo, Cash App, Etsy, eBay, Stripe,
            Square, Airbnb sellers</strong>. Gross only — no expense deduction on form. Reconcile on
            Schedule C / Form 8949.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6050W.h2.inputs">Inputs</h2>
            <form id="s6050W-form" class="inline-form">
                <label><span data-i18n="view.s6050W.label.gross">Gross payments received ($)</span>
                    <input type="number" step="100" name="gross_payments_received" value="${state.gross_payments_received}"></label>
                <label><span data-i18n="view.s6050W.label.txns">Transaction count</span>
                    <input type="number" step="1" name="transaction_count" value="${state.transaction_count}"></label>
                <label><span data-i18n="view.s6050W.label.year">Tax year</span>
                    <input type="number" step="1" name="tax_year" value="${state.tax_year}"></label>
                <label><span data-i18n="view.s6050W.label.payee">Payee type</span>
                    <select name="payee_type">
                        <option value="individual" ${state.payee_type === 'individual' ? 'selected' : ''}>Individual</option>
                        <option value="sole_prop" ${state.payee_type === 'sole_prop' ? 'selected' : ''}>Sole proprietor</option>
                        <option value="partnership" ${state.payee_type === 'partnership' ? 'selected' : ''}>Partnership</option>
                        <option value="corp" ${state.payee_type === 'corp' ? 'selected' : ''}>Corporation</option>
                        <option value="exempt" ${state.payee_type === 'exempt' ? 'selected' : ''}>Tax-exempt</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6050W.label.biz">Business transactions?</span>
                    <input type="checkbox" name="is_business" ${state.is_business ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6050W.label.personal">Personal transfers (friends/family)?</span>
                    <input type="checkbox" name="is_personal_transfer" ${state.is_personal_transfer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6050W.label.cogs">Cost of goods sold ($)</span>
                    <input type="number" step="100" name="cost_of_goods" value="${state.cost_of_goods}"></label>
                <label><span data-i18n="view.s6050W.label.expenses">Business expenses ($)</span>
                    <input type="number" step="100" name="business_expenses" value="${state.business_expenses}"></label>
                <label><span data-i18n="view.s6050W.label.loss">Sale at LOSS (personal items)?</span>
                    <input type="checkbox" name="sale_was_loss" ${state.sale_was_loss ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6050W.label.pse">PSE type</span>
                    <select name="pse_type">
                        <option value="tpso" ${state.pse_type === 'tpso' ? 'selected' : ''}>TPSO (PayPal, Venmo, Cash App)</option>
                        <option value="merchant" ${state.pse_type === 'merchant' ? 'selected' : ''}>Merchant card (Stripe, Square)</option>
                        <option value="marketplace" ${state.pse_type === 'marketplace' ? 'selected' : ''}>Marketplace (eBay, Etsy, Amazon)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6050W.label.bw">Backup withholding (24%)?</span>
                    <input type="checkbox" name="backup_withholding_applied" ${state.backup_withholding_applied ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6050W.label.state_low">State lower threshold (CA $600, MA $600)?</span>
                    <input type="checkbox" name="state_threshold_lower" ${state.state_threshold_lower ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6050W.label.correction">Correction filed (CORR)?</span>
                    <input type="checkbox" name="correction_filed" ${state.correction_filed ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6050W.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6050W-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6050W.h2.phasein">Threshold phase-in timeline</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s6050W.th.year">Tax year</th>
                    <th data-i18n="view.s6050W.th.federal">Federal threshold</th>
                    <th data-i18n="view.s6050W.th.notes">Source</th>
                </tr></thead>
                <tbody>
                    <tr><td>≤ 2022</td><td>$20K + 200 transactions</td><td>Original 2008 statute</td></tr>
                    <tr><td>2023</td><td>$20K + 200 (delayed)</td><td>Notice 2023-10</td></tr>
                    <tr><td>2024</td><td>$5K (delayed phase-in)</td><td>Notice 2024-85</td></tr>
                    <tr><td>2025</td><td>$2.5K (delayed phase-in)</td><td>Notice 2024-85</td></tr>
                    <tr><td>2026+</td><td>$600 (ARPA 2021 final)</td><td>§ 9674 of ARPA 2021</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6050W.h2.reconcile">Reconciliation paths on tax return</h2>
            <ul class="muted small">
                <li data-i18n="view.s6050W.rec.schedule_c">Business income: Schedule C — gross 1099-K + COGS + expenses → net profit</li>
                <li data-i18n="view.s6050W.rec.personal_loss">Personal item LOSS (e.g., sold used couch): Schedule 1 → "Other Income +X" then "Other Adj -X"</li>
                <li data-i18n="view.s6050W.rec.personal_gain">Personal item GAIN: Schedule D — collectibles gain (Form 8949 capital gain)</li>
                <li data-i18n="view.s6050W.rec.friends">Friends + family Venmo: NOT income — but document and exclude on Schedule 1 adj</li>
                <li data-i18n="view.s6050W.rec.gig">Gig income: Schedule C if regular trade or Schedule SE for self-employment tax</li>
                <li data-i18n="view.s6050W.rec.airbnb">Airbnb / VRBO: Schedule E if not service business; Schedule C if substantial services</li>
                <li data-i18n="view.s6050W.rec.crypto">Crypto exchanges 1099-K: report on Form 8949 (specific identification by lot)</li>
                <li data-i18n="view.s6050W.rec.disagree">Disagree with PSE? Request CORR from PSE → Form 1040 reconciliation statement</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6050W.h2.exceptions">Exceptions + carve-outs</h2>
            <ul class="muted small">
                <li data-i18n="view.s6050W.exc.personal">Personal transfers (gifts, splitting bills): NOT reportable; PSE must distinguish</li>
                <li data-i18n="view.s6050W.exc.under_threshold">Below threshold: PSE may STILL report (de minimis) — many file anyway</li>
                <li data-i18n="view.s6050W.exc.foreign">Foreign payees: not reportable (different reporting under FATCA / Ch 3 / 4)</li>
                <li data-i18n="view.s6050W.exc.s139_disaster">Disaster relief payments (§ 139): excluded from gross income, may still appear</li>
                <li data-i18n="view.s6050W.exc.charity">Charitable contributions: excluded from PSE reporting (donor letter substantiates)</li>
                <li data-i18n="view.s6050W.exc.tax_exempt">Tax-exempt entities: still receive 1099-K but typically excluded from UBI</li>
                <li data-i18n="view.s6050W.exc.refunds">Refunds + chargebacks: GROSS — taxpayer reconciles to net on Schedule C</li>
                <li data-i18n="view.s6050W.exc.combined">Aggregation: PSE aggregates ALL transactions under SAME TIN/account</li>
            </ul>
        </div>
    `;
    document.getElementById('s6050W-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.gross_payments_received = Number(fd.get('gross_payments_received')) || 0;
        state.transaction_count = Number(fd.get('transaction_count')) || 0;
        state.tax_year = Number(fd.get('tax_year')) || 0;
        state.payee_type = fd.get('payee_type');
        state.is_business = !!fd.get('is_business');
        state.is_personal_transfer = !!fd.get('is_personal_transfer');
        state.cost_of_goods = Number(fd.get('cost_of_goods')) || 0;
        state.business_expenses = Number(fd.get('business_expenses')) || 0;
        state.sale_was_loss = !!fd.get('sale_was_loss');
        state.pse_type = fd.get('pse_type');
        state.backup_withholding_applied = !!fd.get('backup_withholding_applied');
        state.state_threshold_lower = !!fd.get('state_threshold_lower');
        state.correction_filed = !!fd.get('correction_filed');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6050W-output');
    if (!el) return;
    let federalThreshold = 600;
    if (state.tax_year <= 2023) federalThreshold = 20_000;
    else if (state.tax_year === 2024) federalThreshold = 5_000;
    else if (state.tax_year === 2025) federalThreshold = 2_500;
    const reportable = state.gross_payments_received >= federalThreshold || (state.tax_year <= 2023 && state.transaction_count >= 200);
    const stateReportable = state.state_threshold_lower && state.gross_payments_received >= 600;
    const formIssued = reportable || stateReportable;
    const taxableIncome = state.is_business ?
        Math.max(0, state.gross_payments_received - state.cost_of_goods - state.business_expenses) :
        state.is_personal_transfer ? 0 :
        state.sale_was_loss ? 0 :
        Math.max(0, state.gross_payments_received - state.cost_of_goods);
    const seTax = state.payee_type === 'sole_prop' ? taxableIncome * 0.153 : 0;
    const backupWHRate = state.backup_withholding_applied ? 0.24 : 0;
    const backupWH = state.gross_payments_received * backupWHRate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6050W.h2.result">§ 6050W reconciliation</h2>
            <div class="cards">
                <div class="card ${formIssued ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6050W.card.issued">1099-K issued?</div>
                    <div class="value">${formIssued ? esc(t('view.s6050W.status.yes')) : esc(t('view.s6050W.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6050W.card.threshold">Federal threshold</div>
                    <div class="value">$${federalThreshold.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6050W.card.gross">Gross amount</div>
                    <div class="value">$${state.gross_payments_received.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6050W.card.taxable">Taxable income</div>
                    <div class="value">$${taxableIncome.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6050W.card.se_tax">SE tax (15.3%)</div>
                    <div class="value">$${seTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6050W.card.backup">Backup withholding (24%)</div>
                    <div class="value">$${backupWH.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.is_personal_transfer ? `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s6050W.personal_note">
                    Personal transfers (friends + family): NOT taxable income. Reconcile on Schedule 1 — report
                    Other Income gross then reverse with Other Adjustment "Form 1099-K personal transfers
                    not reportable". PSE should have flagged as "Personal Payment" but often misclassified.
                </p>
            ` : ''}
        </div>
    `;
}
