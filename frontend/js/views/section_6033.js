// IRC § 6033 — Information Return Required of Exempt Organizations.
// 501(c) organizations must file annual return (Form 990, 990-EZ, 990-N, 990-PF).
// Form selection: based on gross receipts + total assets.
// Failure to file 3 consecutive years: automatic loss of exempt status (§ 6033(j)).
// Public inspection requirement: Forms 990 available to public via IRS + GuideStar.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    org_type: '501c3_public_charity',
    gross_receipts: 0,
    total_assets: 0,
    fiscal_year_end_month: 12,
    is_private_foundation: false,
    is_church: false,
    is_government_instrumentality: false,
    consecutive_years_failure: 0,
    form_filed: '990',
    is_first_year_org: false,
    requires_audit: false,
    has_unrelated_business_income: false,
    ubit_amount: 0,
    foreign_filings_5471: false,
    federal_grant_recipient: false,
    requires_schedule_b_donors: false,
};

export async function renderSection6033(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6033.h1.title">// § 6033 EXEMPT ORG FILING</span></h1>
        <p class="muted small" data-i18n="view.s6033.hint.intro">
            501(c) orgs must file <strong>annual return</strong>: Form 990, 990-EZ, 990-N, or 990-PF.
            <strong>Form selection</strong> based on gross receipts + total assets. <strong>Form 990-N
            (e-Postcard):</strong> ≤ $50K gross receipts. <strong>Form 990-EZ:</strong> &lt; $200K
            receipts + &lt; $500K assets. <strong>Form 990 (full):</strong> ≥ $200K OR ≥ $500K.
            <strong>Form 990-PF:</strong> ALL private foundations. <strong>Failure 3 consecutive years:</strong>
            <strong>AUTOMATIC LOSS</strong> of exempt status (§ 6033(j) — Pension Protection Act 2006).
            <strong>Public inspection:</strong> Forms 990 publicly available.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6033.h2.inputs">Inputs</h2>
            <form id="s6033-form" class="inline-form">
                <label><span data-i18n="view.s6033.label.type">Organization type</span>
                    <select name="org_type">
                        <option value="501c3_public_charity" ${state.org_type === '501c3_public_charity' ? 'selected' : ''}>§ 501(c)(3) public charity</option>
                        <option value="501c3_private_foundation" ${state.org_type === '501c3_private_foundation' ? 'selected' : ''}>§ 501(c)(3) private foundation</option>
                        <option value="501c4" ${state.org_type === '501c4' ? 'selected' : ''}>§ 501(c)(4) social welfare</option>
                        <option value="501c5" ${state.org_type === '501c5' ? 'selected' : ''}>§ 501(c)(5) labor / agriculture</option>
                        <option value="501c6" ${state.org_type === '501c6' ? 'selected' : ''}>§ 501(c)(6) business league</option>
                        <option value="501c7" ${state.org_type === '501c7' ? 'selected' : ''}>§ 501(c)(7) social club</option>
                        <option value="501c19" ${state.org_type === '501c19' ? 'selected' : ''}>§ 501(c)(19) veterans</option>
                        <option value="501c8" ${state.org_type === '501c8' ? 'selected' : ''}>§ 501(c)(8) fraternal</option>
                        <option value="church" ${state.org_type === 'church' ? 'selected' : ''}>Church (exempt from filing)</option>
                        <option value="govt_instrumentality" ${state.org_type === 'govt_instrumentality' ? 'selected' : ''}>Govt instrumentality</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6033.label.receipts">Gross receipts ($)</span>
                    <input type="number" step="0.01" name="gross_receipts" value="${state.gross_receipts}"></label>
                <label><span data-i18n="view.s6033.label.assets">Total assets ($)</span>
                    <input type="number" step="0.01" name="total_assets" value="${state.total_assets}"></label>
                <label><span data-i18n="view.s6033.label.fiscal">Fiscal year end month</span>
                    <input type="number" step="1" name="fiscal_year_end_month" value="${state.fiscal_year_end_month}"></label>
                <label><span data-i18n="view.s6033.label.pf">Private foundation?</span>
                    <input type="checkbox" name="is_private_foundation" ${state.is_private_foundation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6033.label.church">Church?</span>
                    <input type="checkbox" name="is_church" ${state.is_church ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6033.label.govt">Government instrumentality?</span>
                    <input type="checkbox" name="is_government_instrumentality" ${state.is_government_instrumentality ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6033.label.consec">Consecutive years failure</span>
                    <input type="number" step="1" name="consecutive_years_failure" value="${state.consecutive_years_failure}"></label>
                <label><span data-i18n="view.s6033.label.form">Form filed</span>
                    <select name="form_filed">
                        <option value="990" ${state.form_filed === '990' ? 'selected' : ''}>Form 990 (full)</option>
                        <option value="990ez" ${state.form_filed === '990ez' ? 'selected' : ''}>Form 990-EZ</option>
                        <option value="990n" ${state.form_filed === '990n' ? 'selected' : ''}>Form 990-N (e-Postcard)</option>
                        <option value="990pf" ${state.form_filed === '990pf' ? 'selected' : ''}>Form 990-PF</option>
                        <option value="990t" ${state.form_filed === '990t' ? 'selected' : ''}>Form 990-T (UBI)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6033.label.first_year">First year org?</span>
                    <input type="checkbox" name="is_first_year_org" ${state.is_first_year_org ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6033.label.audit">Required audit?</span>
                    <input type="checkbox" name="requires_audit" ${state.requires_audit ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6033.label.ubi">Has UBI (Unrelated Business Income)?</span>
                    <input type="checkbox" name="has_unrelated_business_income" ${state.has_unrelated_business_income ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6033.label.ubit">UBIT amount ($)</span>
                    <input type="number" step="0.01" name="ubit_amount" value="${state.ubit_amount}"></label>
                <label><span data-i18n="view.s6033.label.5471">Foreign filings (Form 5471)?</span>
                    <input type="checkbox" name="foreign_filings_5471" ${state.foreign_filings_5471 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6033.label.grant">Federal grant recipient?</span>
                    <input type="checkbox" name="federal_grant_recipient" ${state.federal_grant_recipient ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6033.label.schedule_b">Requires Schedule B (donors)?</span>
                    <input type="checkbox" name="requires_schedule_b_donors" ${state.requires_schedule_b_donors ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6033.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6033-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6033.h2.form_selection">Form selection by size</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s6033.th.size">Size</th>
                    <th data-i18n="view.s6033.th.form">Form</th>
                    <th data-i18n="view.s6033.th.criteria">Criteria</th>
                </tr></thead>
                <tbody>
                    <tr><td>Smallest</td><td>Form 990-N</td><td>Gross receipts ≤ $50K (annually)</td></tr>
                    <tr><td>Small</td><td>Form 990-EZ</td><td>Gross receipts &lt; $200K + Total assets &lt; $500K</td></tr>
                    <tr><td>Standard</td><td>Form 990</td><td>Gross receipts ≥ $200K OR Total assets ≥ $500K</td></tr>
                    <tr><td>Private foundation</td><td>Form 990-PF</td><td>ALL private foundations (any size)</td></tr>
                    <tr><td>UBI &gt; $1K</td><td>Form 990-T (additional)</td><td>Unrelated business taxable income</td></tr>
                    <tr><td>First year org</td><td>Pro-rated</td><td>Avg gross receipts over 3 yrs (1st year exception)</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6033.h2.deadlines">Filing deadlines + extensions</h2>
            <ul class="muted small">
                <li data-i18n="view.s6033.dl.standard">Standard deadline: 15th day of 5th month after fiscal year end (May 15 for calendar)</li>
                <li data-i18n="view.s6033.dl.extension">Form 8868 extension: 6-month automatic extension to November 15 (calendar)</li>
                <li data-i18n="view.s6033.dl.electronic">E-file mandatory (post-2020) — Taxpayer First Act 2019</li>
                <li data-i18n="view.s6033.dl.990n">Form 990-N: filed online via IRS e-Postcard portal (free)</li>
                <li data-i18n="view.s6033.dl.late_penalty">Late penalty: $20/day (up to $11K) or $100/day for large orgs (up to $56K)</li>
                <li data-i18n="view.s6033.dl.3_consecutive">3 consecutive years failure: AUTOMATIC LOSS of exempt status</li>
                <li data-i18n="view.s6033.dl.reinstatement">Reinstatement via Form 1023 + Rev. Proc. 2014-11 streamlined process</li>
                <li data-i18n="view.s6033.dl.public_inspection">Public inspection: Forms 990 + 1023 available 3 yrs free at IRS</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6033.h2.exceptions">Filing exceptions</h2>
            <ul class="muted small">
                <li data-i18n="view.s6033.exc.churches">Churches: NOT required to file (§ 6033(a)(3)(A)(i))</li>
                <li data-i18n="view.s6033.exc.religious_orders">Religious orders / mission societies: NOT required to file</li>
                <li data-i18n="view.s6033.exc.govt_instrumentalities">Governmental units / instrumentalities: NOT required to file</li>
                <li data-i18n="view.s6033.exc.exclusively_religious">Exclusively religious instructional orgs: NOT required to file</li>
                <li data-i18n="view.s6033.exc.us_possessions">US possession affiliated orgs: limited filing</li>
                <li data-i18n="view.s6033.exc.unincorporated">Unincorporated 501(c)(3) under religious affiliation: may not need to file</li>
                <li data-i18n="view.s6033.exc.affiliate_of_church">Integrated auxiliary of church (e.g., school): NOT required to file (Form 4720 exception)</li>
                <li data-i18n="view.s6033.exc.below_minimum">Below $50K + first year: may still file Form 990-N for transparency</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6033.h2.schedule_b">Schedule B donor disclosure</h2>
            <ul class="muted small">
                <li data-i18n="view.s6033.sb.purpose">Discloses substantial contributors (≥ $5K AND ≥ 2% of total)</li>
                <li data-i18n="view.s6033.sb.501c3">501(c)(3) public charity: only IRS sees names + addresses (post-2018 Rev. Proc. 2018-38)</li>
                <li data-i18n="view.s6033.sb.501c4">501(c)(4) + (5) + (6): NOT required (donor anonymity post-2018 Rev. Proc.)</li>
                <li data-i18n="view.s6033.sb.501c3_pi">Public inspection: amounts only (not donor names) for 501(c)(3) public charities</li>
                <li data-i18n="view.s6033.sb.pf">Private foundations: full disclosure even publicly (different rule)</li>
                <li data-i18n="view.s6033.sb.amat_v_irs">Americans for Prosperity v. Bonta (2021): SCOTUS struck down CA/NY donor disclosure</li>
                <li data-i18n="view.s6033.sb.state_rules">State-level: NY + CA stopped requiring after AFP v. Bonta</li>
                <li data-i18n="view.s6033.sb.federal_continue">Federal disclosure to IRS (not public): continues regardless</li>
            </ul>
        </div>
    `;
    document.getElementById('s6033-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.org_type = fd.get('org_type');
        state.gross_receipts = Number(fd.get('gross_receipts')) || 0;
        state.total_assets = Number(fd.get('total_assets')) || 0;
        state.fiscal_year_end_month = Number(fd.get('fiscal_year_end_month')) || 0;
        state.is_private_foundation = !!fd.get('is_private_foundation');
        state.is_church = !!fd.get('is_church');
        state.is_government_instrumentality = !!fd.get('is_government_instrumentality');
        state.consecutive_years_failure = Number(fd.get('consecutive_years_failure')) || 0;
        state.form_filed = fd.get('form_filed');
        state.is_first_year_org = !!fd.get('is_first_year_org');
        state.requires_audit = !!fd.get('requires_audit');
        state.has_unrelated_business_income = !!fd.get('has_unrelated_business_income');
        state.ubit_amount = Number(fd.get('ubit_amount')) || 0;
        state.foreign_filings_5471 = !!fd.get('foreign_filings_5471');
        state.federal_grant_recipient = !!fd.get('federal_grant_recipient');
        state.requires_schedule_b_donors = !!fd.get('requires_schedule_b_donors');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6033-output');
    if (!el) return;
    const filing_required = !(state.is_church || state.is_government_instrumentality);
    let appropriate_form = '990';
    if (state.is_private_foundation) appropriate_form = '990pf';
    else if (state.gross_receipts <= 50_000) appropriate_form = '990n';
    else if (state.gross_receipts < 200_000 && state.total_assets < 500_000) appropriate_form = '990ez';
    const automatic_revocation = state.consecutive_years_failure >= 3;
    const due_month = (state.fiscal_year_end_month + 5) % 12 || 12;
    const ubi_t_required = state.has_unrelated_business_income && state.ubit_amount > 1_000;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6033.h2.result">§ 6033 filing requirement</h2>
            <div class="cards">
                <div class="card ${filing_required ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6033.card.required">Filing required?</div>
                    <div class="value">${filing_required ? esc(t('view.s6033.status.yes')) : esc(t('view.s6033.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6033.card.form">Appropriate form</div>
                    <div class="value">Form ${esc(appropriate_form.toUpperCase())}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6033.card.due">Due (15th of month)</div>
                    <div class="value">${due_month}/15</div>
                </div>
                <div class="card ${automatic_revocation ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6033.card.revocation">Automatic revocation?</div>
                    <div class="value">${automatic_revocation ? esc(t('view.s6033.status.yes')) : esc(t('view.s6033.status.no'))}</div>
                </div>
                <div class="card ${ubi_t_required ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6033.card.ubi_t">Form 990-T required?</div>
                    <div class="value">${ubi_t_required ? esc(t('view.s6033.status.yes')) : esc(t('view.s6033.status.no'))}</div>
                </div>
                <div class="card ${state.requires_schedule_b_donors ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s6033.card.sb">Schedule B required?</div>
                    <div class="value">${state.requires_schedule_b_donors ? esc(t('view.s6033.status.yes')) : esc(t('view.s6033.status.no'))}</div>
                </div>
            </div>
            ${automatic_revocation ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s6033.revoked_note">
                    AUTOMATIC REVOCATION: 3 consecutive years of failure to file. Exempt status LOST.
                    Apply for reinstatement via Form 1023 + Rev. Proc. 2014-11 (streamlined available
                    if eligible). Retroactive reinstatement possible if applied within 15 months of
                    revocation notice. Public IRS list "Auto Revocation" shows revoked orgs.
                </p>
            ` : ''}
        </div>
    `;
}
