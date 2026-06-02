// IRC § 6049 — Information Reporting on Interest Payments (Form 1099-INT).
// Payers must report $10+ interest paid + $600+ in trade/business interest.
// Form 1099-INT: Box 1 (interest income), Box 2 (early withdrawal penalty), Box 3 (US gov interest), Box 4 (federal withholding).
// Tax-exempt municipal interest: Box 8 (other Boxes).
// Form 1099-OID: separate form for original issue discount accruals.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    interest_paid_total: 0,
    interest_type: 'bank_deposits',
    is_us_government_obligation: false,
    is_tax_exempt_municipal: false,
    is_private_activity_bond: false,
    early_withdrawal_penalty: 0,
    federal_withholding_amount: 0,
    foreign_tax_paid: 0,
    is_foreign_payee: false,
    has_w8ben: false,
    is_corporation_payee: false,
    bank_deposit_exemption: false,
    tax_year: 2024,
    backup_withholding_pct: 24,
    backup_withholding_triggered: false,
    is_portfolio_interest_871h: false,
    recipient_tin_provided: true,
};

export async function renderSection6049(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6049.h1.title">// § 6049 INTEREST REPORTING (1099-INT)</span></h1>
        <p class="muted small" data-i18n="view.s6049.hint.intro">
            Payers must report <strong>$10+ interest paid</strong> + <strong>$600+ in trade/business interest</strong>.
            <strong>Form 1099-INT:</strong> Box 1 (interest income), Box 2 (early withdrawal penalty), Box 3
            (US gov interest), Box 4 (federal withholding), Box 8 (tax-exempt interest), Box 9 (private activity
            bond interest — AMT prefer item). <strong>Form 1099-OID</strong> separate for original issue discount.
            <strong>Foreign payees:</strong> Form 1042-S separate regime (with W-8BEN). <strong>Bank deposit
            interest to NRAs:</strong> § 871(i) exemption from withholding (since 2013).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6049.h2.inputs">Inputs</h2>
            <form id="s6049-form" class="inline-form">
                <label><span data-i18n="view.s6049.label.amount">Interest paid total ($)</span>
                    <input type="number" step="1" name="interest_paid_total" value="${state.interest_paid_total}"></label>
                <label><span data-i18n="view.s6049.label.type">Interest type</span>
                    <select name="interest_type">
                        <option value="bank_deposits" ${state.interest_type === 'bank_deposits' ? 'selected' : ''}>Bank deposits / CDs</option>
                        <option value="corporate_bonds" ${state.interest_type === 'corporate_bonds' ? 'selected' : ''}>Corporate bonds</option>
                        <option value="treasury" ${state.interest_type === 'treasury' ? 'selected' : ''}>US Treasury (Box 3)</option>
                        <option value="agency_bonds" ${state.interest_type === 'agency_bonds' ? 'selected' : ''}>Agency bonds (GNMA, FNMA)</option>
                        <option value="municipal_general" ${state.interest_type === 'municipal_general' ? 'selected' : ''}>Municipal general obligation (Box 8)</option>
                        <option value="municipal_private" ${state.interest_type === 'municipal_private' ? 'selected' : ''}>Municipal private activity (Box 9)</option>
                        <option value="private_loan" ${state.interest_type === 'private_loan' ? 'selected' : ''}>Private loan (1099-INT or business)</option>
                        <option value="brokered_cd" ${state.interest_type === 'brokered_cd' ? 'selected' : ''}>Brokered CD</option>
                        <option value="money_market" ${state.interest_type === 'money_market' ? 'selected' : ''}>Money market fund</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6049.label.us_gov">US Government obligation?</span>
                    <input type="checkbox" name="is_us_government_obligation" ${state.is_us_government_obligation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6049.label.tax_exempt">Tax-exempt municipal?</span>
                    <input type="checkbox" name="is_tax_exempt_municipal" ${state.is_tax_exempt_municipal ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6049.label.pab">Private activity bond (AMT)?</span>
                    <input type="checkbox" name="is_private_activity_bond" ${state.is_private_activity_bond ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6049.label.ewp">Early withdrawal penalty ($)</span>
                    <input type="number" step="1" name="early_withdrawal_penalty" value="${state.early_withdrawal_penalty}"></label>
                <label><span data-i18n="view.s6049.label.withholding">Federal withholding ($)</span>
                    <input type="number" step="1" name="federal_withholding_amount" value="${state.federal_withholding_amount}"></label>
                <label><span data-i18n="view.s6049.label.foreign">Foreign tax paid ($)</span>
                    <input type="number" step="1" name="foreign_tax_paid" value="${state.foreign_tax_paid}"></label>
                <label><span data-i18n="view.s6049.label.foreign_payee">Foreign payee (NRA)?</span>
                    <input type="checkbox" name="is_foreign_payee" ${state.is_foreign_payee ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6049.label.w8ben">Has W-8BEN?</span>
                    <input type="checkbox" name="has_w8ben" ${state.has_w8ben ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6049.label.corp">Corporation payee?</span>
                    <input type="checkbox" name="is_corporation_payee" ${state.is_corporation_payee ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6049.label.bank_exempt">§ 871(i) bank deposit exemption?</span>
                    <input type="checkbox" name="bank_deposit_exemption" ${state.bank_deposit_exemption ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6049.label.year">Tax year</span>
                    <input type="number" step="1" name="tax_year" value="${state.tax_year}"></label>
                <label><span data-i18n="view.s6049.label.bw_pct">Backup withholding %</span>
                    <input type="number" step="0.1" name="backup_withholding_pct" value="${state.backup_withholding_pct}"></label>
                <label><span data-i18n="view.s6049.label.bw_triggered">Backup withholding triggered?</span>
                    <input type="checkbox" name="backup_withholding_triggered" ${state.backup_withholding_triggered ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6049.label.s871h">§ 871(h) portfolio interest?</span>
                    <input type="checkbox" name="is_portfolio_interest_871h" ${state.is_portfolio_interest_871h ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6049.label.tin">Recipient TIN provided?</span>
                    <input type="checkbox" name="recipient_tin_provided" ${state.recipient_tin_provided ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6049.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6049-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6049.h2.boxes">Form 1099-INT box detail</h2>
            <ul class="muted small">
                <li data-i18n="view.s6049.box.1">Box 1: interest income (taxable)</li>
                <li data-i18n="view.s6049.box.2">Box 2: early withdrawal penalty (deductible on Schedule 1)</li>
                <li data-i18n="view.s6049.box.3">Box 3: US Treasury / agency interest (state-tax-exempt)</li>
                <li data-i18n="view.s6049.box.4">Box 4: federal income tax withheld</li>
                <li data-i18n="view.s6049.box.5">Box 5: investment expenses (REMICs only)</li>
                <li data-i18n="view.s6049.box.6">Box 6: foreign tax paid (FTC available)</li>
                <li data-i18n="view.s6049.box.7">Box 7: foreign country / US possession</li>
                <li data-i18n="view.s6049.box.8">Box 8: tax-exempt interest (municipal)</li>
                <li data-i18n="view.s6049.box.9">Box 9: specified private activity bond interest (AMT preference)</li>
                <li data-i18n="view.s6049.box.10">Box 10: market discount</li>
                <li data-i18n="view.s6049.box.11">Box 11: bond premium (amortizable taxable bonds)</li>
                <li data-i18n="view.s6049.box.12">Box 12: bond premium on Treasury</li>
                <li data-i18n="view.s6049.box.13">Box 13: bond premium on tax-exempt</li>
                <li data-i18n="view.s6049.box.14">Box 14: tax-exempt + tax credit bond CUSIP</li>
                <li data-i18n="view.s6049.box.15">Box 15: state name + tax withheld</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6049.h2.exemptions">§ 6049 reporting exemptions</h2>
            <ul class="muted small">
                <li data-i18n="view.s6049.exem.corporations">Corporations: generally no 1099-INT (interest payments)</li>
                <li data-i18n="view.s6049.exem.foreign">Foreign payees: Form 1042 / 1042-S separate regime</li>
                <li data-i18n="view.s6049.exem.tin">Less than $10: no reporting (Box 1 + Box 3 combined)</li>
                <li data-i18n="view.s6049.exem.muni_exempt">Tax-exempt entities: NOT EXEMPT from reporting (but they still get 1099-INT)</li>
                <li data-i18n="view.s6049.exem.gov_employees">Government / military: no exemption</li>
                <li data-i18n="view.s6049.exem.bank_dep_nra">Bank deposits to NRAs: § 871(i) exemption from 30% withholding (since 2013); still reported via 1042-S</li>
                <li data-i18n="view.s6049.exem.portfolio_int">§ 871(h) portfolio interest: 0% withholding for foreign payees</li>
                <li data-i18n="view.s6049.exem.under_10">Threshold: Box 1 + Box 3 combined ≥ $10 → must file</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6049.h2.tax_exempt_state">Tax-exempt + state tax interaction</h2>
            <ul class="muted small">
                <li data-i18n="view.s6049.te.federal">Federal: municipal bond interest (Box 8) excluded from gross income</li>
                <li data-i18n="view.s6049.te.amt_pab">AMT preference: private activity bond interest (Box 9) — adds back for AMT</li>
                <li data-i18n="view.s6049.te.state_in_state">In-state municipal bonds: usually state tax exempt too</li>
                <li data-i18n="view.s6049.te.state_out_state">Out-of-state municipal: federal exempt + state TAXABLE (most states)</li>
                <li data-i18n="view.s6049.te.treasury_state">US Treasury / agency: state tax exempt (Box 3)</li>
                <li data-i18n="view.s6049.te.aotc">Tax-exempt interest affects: Social Security taxation, AOTC phaseout, IRA contribution limits</li>
                <li data-i18n="view.s6049.te.modified_agi">Modified AGI for ACA premium credit: includes tax-exempt interest</li>
                <li data-i18n="view.s6049.te.alternative_minimum">Pre-2018 individual AMT preference; post-2025 sunset uncertain</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6049.h2.oid">§ 1273 OID / 1099-OID coordination</h2>
            <ul class="muted small">
                <li data-i18n="view.s6049.oid.separate">OID accrual reported on separate Form 1099-OID, not 1099-INT</li>
                <li data-i18n="view.s6049.oid.box1">1099-OID Box 1: OID amount accrued in current year</li>
                <li data-i18n="view.s6049.oid.box2">1099-OID Box 2: other periodic interest</li>
                <li data-i18n="view.s6049.oid.zero_coupon">Zero-coupon bonds: large OID accrual + no cash interest payments</li>
                <li data-i18n="view.s6049.oid.taxpayer_pays">Taxpayer pays tax on accrual EVEN WITHOUT cash interest (zero-coupon problem)</li>
                <li data-i18n="view.s6049.oid.tax_exempt">Tax-exempt municipal OID: excluded similar to regular muni interest</li>
                <li data-i18n="view.s6049.oid.short_term">Short-term obligations ≤ 1 year: no OID reporting</li>
                <li data-i18n="view.s6049.oid.de_minimis">De minimis OID: 0.25% × years to maturity → not reportable</li>
            </ul>
        </div>
    `;
    document.getElementById('s6049-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.interest_paid_total = Number(fd.get('interest_paid_total')) || 0;
        state.interest_type = fd.get('interest_type');
        state.is_us_government_obligation = !!fd.get('is_us_government_obligation');
        state.is_tax_exempt_municipal = !!fd.get('is_tax_exempt_municipal');
        state.is_private_activity_bond = !!fd.get('is_private_activity_bond');
        state.early_withdrawal_penalty = Number(fd.get('early_withdrawal_penalty')) || 0;
        state.federal_withholding_amount = Number(fd.get('federal_withholding_amount')) || 0;
        state.foreign_tax_paid = Number(fd.get('foreign_tax_paid')) || 0;
        state.is_foreign_payee = !!fd.get('is_foreign_payee');
        state.has_w8ben = !!fd.get('has_w8ben');
        state.is_corporation_payee = !!fd.get('is_corporation_payee');
        state.bank_deposit_exemption = !!fd.get('bank_deposit_exemption');
        state.tax_year = Number(fd.get('tax_year')) || 0;
        state.backup_withholding_pct = Number(fd.get('backup_withholding_pct')) || 0;
        state.backup_withholding_triggered = !!fd.get('backup_withholding_triggered');
        state.is_portfolio_interest_871h = !!fd.get('is_portfolio_interest_871h');
        state.recipient_tin_provided = !!fd.get('recipient_tin_provided');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6049-output');
    if (!el) return;
    const reportable = state.interest_paid_total >= 10 && !state.is_corporation_payee;
    const taxable_interest = state.is_tax_exempt_municipal ? 0 : state.interest_paid_total;
    const amt_preference = state.is_private_activity_bond ? state.interest_paid_total : 0;
    const backup_required = state.backup_withholding_triggered || !state.recipient_tin_provided;
    const backup_amt = backup_required ? state.interest_paid_total * (state.backup_withholding_pct / 100) : 0;
    const total_tax_savings_ewp = state.early_withdrawal_penalty * 0.37;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6049.h2.result">§ 6049 reporting determination</h2>
            <div class="cards">
                <div class="card ${reportable ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6049.card.reportable">Reportable on 1099-INT?</div>
                    <div class="value">${reportable ? esc(t('view.s6049.status.yes')) : esc(t('view.s6049.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6049.card.taxable">Taxable interest</div>
                    <div class="value">$${taxable_interest.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${amt_preference > 0 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s6049.card.amt">AMT preference (Box 9)</div>
                    <div class="value">$${amt_preference.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s6049.card.ewp_savings">EWP tax savings (37%)</div>
                    <div class="value">$${total_tax_savings_ewp.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${backup_required ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s6049.card.bw_required">Backup withholding required?</div>
                    <div class="value">${backup_required ? esc(t('view.s6049.status.yes')) : esc(t('view.s6049.status.no'))}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6049.card.bw_amount">Backup withholding amount</div>
                    <div class="value">$${backup_amt.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.is_tax_exempt_municipal && state.is_private_activity_bond ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s6049.pab_amt_note">
                    Private Activity Bond (Box 9): federal tax-exempt for regular tax BUT AMT preference
                    item adds back $${state.interest_paid_total.toLocaleString()} to AMTI. Watch out:
                    high-income taxpayers w/ tax-exempt PAB income may unexpectedly hit AMT. Consider
                    swapping to non-PAB muni bonds if AMT triggers consistently.
                </p>
            ` : ''}
        </div>
    `;
}
