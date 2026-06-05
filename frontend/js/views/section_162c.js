// IRC § 162(c) — Illegal Bribes, Kickbacks, Other Illegal Payments.
// NO DEDUCTION for any amount that constitutes illegal bribe / kickback / other illegal payment.
// § 162(c)(1): payments to government officials (federal, state, local, foreign).
// § 162(c)(2): payments to non-government persons / entities — kickbacks.
// § 162(c)(3): kickbacks paid to enable Medicare / Medicaid fraud.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    payment_amount: 0,
    payment_type: 'kickback_general',
    payment_to: 'private_party',
    is_illegal_under_us: true,
    is_illegal_under_foreign: false,
    fcpa_violation: false,
    sherman_act_violation: false,
    rico_violation: false,
    medicare_medicaid_kickback: false,
    cooperative_in_investigation: false,
    is_corporation: true,
    is_individual: false,
    proven_via_doj: false,
    settled_civil_only: false,
};

export async function renderSection162C(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s162c.h1.title">// § 162(c) ILLEGAL PAYMENTS</span></h1>
        <p class="muted small" data-i18n="view.s162c.hint.intro">
            <strong>NO DEDUCTION</strong> for amounts constituting <strong>illegal bribes</strong>, kickbacks,
            other illegal payments. <strong>§ 162(c)(1):</strong> payments to government officials — federal,
            state, local, FOREIGN. <strong>§ 162(c)(2):</strong> kickbacks to non-government persons / entities.
            <strong>§ 162(c)(3):</strong> kickbacks tied to Medicare / Medicaid fraud. <strong>FCPA</strong>
            (Foreign Corrupt Practices Act) violations: nondeductible. <strong>Burden:</strong> taxpayer must
            show payment NOT illegal. <strong>Conviction:</strong> not required — IRS can disallow based on
            facts + law analysis.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s162c.h2.inputs">Inputs</h2>
            <form id="s162c-form" class="inline-form">
                <label><span data-i18n="view.s162c.label.amount">Payment amount ($)</span>
                    <input type="number" step="0.01" name="payment_amount" value="${state.payment_amount}"></label>
                <label><span data-i18n="view.s162c.label.type">Payment type</span>
                    <select name="payment_type">
                        <option value="kickback_general" ${state.payment_type === 'kickback_general' ? 'selected' : ''}>General kickback</option>
                        <option value="bribe_government" ${state.payment_type === 'bribe_government' ? 'selected' : ''}>Bribe to government official</option>
                        <option value="bribe_foreign" ${state.payment_type === 'bribe_foreign' ? 'selected' : ''}>Bribe to foreign official (FCPA)</option>
                        <option value="medicare_kickback" ${state.payment_type === 'medicare_kickback' ? 'selected' : ''}>Medicare / Medicaid kickback</option>
                        <option value="referral_fee" ${state.payment_type === 'referral_fee' ? 'selected' : ''}>Referral fee (potentially legal)</option>
                        <option value="commission" ${state.payment_type === 'commission' ? 'selected' : ''}>Commission</option>
                        <option value="finders_fee" ${state.payment_type === 'finders_fee' ? 'selected' : ''}>Finder's fee</option>
                        <option value="grease_payment" ${state.payment_type === 'grease_payment' ? 'selected' : ''}>Facilitating ("grease") payment</option>
                    </select>
                </label>
                <label><span data-i18n="view.s162c.label.to">Payment to</span>
                    <select name="payment_to">
                        <option value="us_gov_federal" ${state.payment_to === 'us_gov_federal' ? 'selected' : ''}>US federal official</option>
                        <option value="us_gov_state" ${state.payment_to === 'us_gov_state' ? 'selected' : ''}>US state / local official</option>
                        <option value="foreign_gov" ${state.payment_to === 'foreign_gov' ? 'selected' : ''}>Foreign government official</option>
                        <option value="private_party" ${state.payment_to === 'private_party' ? 'selected' : ''}>Private party</option>
                        <option value="medical_provider" ${state.payment_to === 'medical_provider' ? 'selected' : ''}>Medical provider</option>
                        <option value="employee" ${state.payment_to === 'employee' ? 'selected' : ''}>Employee</option>
                    </select>
                </label>
                <label><span data-i18n="view.s162c.label.illegal_us">Illegal under US law?</span>
                    <input type="checkbox" name="is_illegal_under_us" ${state.is_illegal_under_us ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162c.label.illegal_foreign">Illegal under foreign law?</span>
                    <input type="checkbox" name="is_illegal_under_foreign" ${state.is_illegal_under_foreign ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162c.label.fcpa">FCPA violation?</span>
                    <input type="checkbox" name="fcpa_violation" ${state.fcpa_violation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162c.label.sherman">Sherman Act violation?</span>
                    <input type="checkbox" name="sherman_act_violation" ${state.sherman_act_violation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162c.label.rico">RICO predicate?</span>
                    <input type="checkbox" name="rico_violation" ${state.rico_violation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162c.label.medicare">Medicare/Medicaid fraud kickback?</span>
                    <input type="checkbox" name="medicare_medicaid_kickback" ${state.medicare_medicaid_kickback ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162c.label.cooperative">Cooperating in investigation?</span>
                    <input type="checkbox" name="cooperative_in_investigation" ${state.cooperative_in_investigation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162c.label.corporation">Payor corporation?</span>
                    <input type="checkbox" name="is_corporation" ${state.is_corporation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162c.label.individual">Payor individual?</span>
                    <input type="checkbox" name="is_individual" ${state.is_individual ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162c.label.doj">DOJ prosecution / conviction?</span>
                    <input type="checkbox" name="proven_via_doj" ${state.proven_via_doj ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162c.label.civil_only">Civil settlement only?</span>
                    <input type="checkbox" name="settled_civil_only" ${state.settled_civil_only ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s162c.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s162c-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s162c.h2.subsections">§ 162(c) subsections</h2>
            <ul class="muted small">
                <li data-i18n="view.s162c.sub.1">§ 162(c)(1): payments to government officials (any govt) — NEVER deductible</li>
                <li data-i18n="view.s162c.sub.1_fcpa">FCPA violations specifically nondeductible under § 162(c)(1)</li>
                <li data-i18n="view.s162c.sub.2">§ 162(c)(2): non-government illegal payments — kickbacks</li>
                <li data-i18n="view.s162c.sub.2_burden">Burden: taxpayer must prove payment is NOT illegal</li>
                <li data-i18n="view.s162c.sub.3">§ 162(c)(3): Medicare/Medicaid kickbacks (specific federal statutes)</li>
                <li data-i18n="view.s162c.sub.3_safe_harbor">Anti-kickback statute safe harbors: legitimate payment structures</li>
                <li data-i18n="view.s162c.sub.legal_referrals">Legal referrals (with Stark Law compliance): deductible</li>
                <li data-i18n="view.s162c.sub.proof">No conviction required — IRS analyzes legal substance</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s162c.h2.fcpa">FCPA (Foreign Corrupt Practices Act) — § 162(c)(1)(B)</h2>
            <ul class="muted small">
                <li data-i18n="view.s162c.fcpa.basic">FCPA prohibits bribery of foreign officials to obtain / retain business</li>
                <li data-i18n="view.s162c.fcpa.payor">Applies to: US persons + entities + foreign issuers (15 U.S.C. § 78dd-1)</li>
                <li data-i18n="view.s162c.fcpa.cdr">Books + records provisions require accurate financial reporting</li>
                <li data-i18n="view.s162c.fcpa.facilitating">Facilitating ("grease") payments: limited exception under FCPA, STILL NONDEDUCTIBLE under § 162(c)</li>
                <li data-i18n="view.s162c.fcpa.uk_bribery">UK Bribery Act 2010: more strict (no facilitating payment exception)</li>
                <li data-i18n="view.s162c.fcpa.enforcement">SEC + DOJ joint enforcement</li>
                <li data-i18n="view.s162c.fcpa.successor_liability">Successor liability: acquiring corp may inherit FCPA exposure</li>
                <li data-i18n="view.s162c.fcpa.due_diligence">Due diligence + compliance program defense</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s162c.h2.medicare_kickbacks">§ 162(c)(3) Medicare / Medicaid kickbacks</h2>
            <ul class="muted small">
                <li data-i18n="view.s162c.med.anti_kickback">Anti-Kickback Statute (42 U.S.C. § 1320a-7b): broad prohibition</li>
                <li data-i18n="view.s162c.med.stark">Stark Law (42 U.S.C. § 1395nn): physician self-referral</li>
                <li data-i18n="view.s162c.med.safe_harbors">Safe harbors: bona fide employee, GPO (Group Purchasing), space rental, equipment rental</li>
                <li data-i18n="view.s162c.med.intent">Intent: anything of value INTENDED to induce referrals</li>
                <li data-i18n="view.s162c.med.fee_splitting">Fee-splitting with non-physicians: NONDEDUCTIBLE (and illegal)</li>
                <li data-i18n="view.s162c.med.consulting_fees">"Consulting fees" disguised as referral compensation: scrutinized</li>
                <li data-i18n="view.s162c.med.compliance_program">Compliance program: Office of Inspector General (OIG) recommendations</li>
                <li data-i18n="view.s162c.med.qui_tam">Qui tam (False Claims Act): whistleblower 15-30% reward share</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s162c.h2.notable_cases">Notable § 162(c) cases</h2>
            <ul class="muted small">
                <li data-i18n="view.s162c.cases.commissioner">Commissioner v. Tellier (1966): legal fees in defense of business deductible (NOT § 162(c))</li>
                <li data-i18n="view.s162c.cases.smith_wessel">Smith v. Commissioner: kickback definition fact-intensive</li>
                <li data-i18n="view.s162c.cases.alcan">Alcan Aluminium (1998): Canadian tax law factor in foreign legality</li>
                <li data-i18n="view.s162c.cases.dichter">Dichter v. Commissioner: legitimate vs disguised commissions</li>
                <li data-i18n="view.s162c.cases.ucasco">U-CASCO (S.D.N.Y. 1992): Medicare anti-kickback enforcement</li>
                <li data-i18n="view.s162c.cases.siemens">Siemens FCPA $800M (2008): largest FCPA settlement — all nondeductible</li>
                <li data-i18n="view.s162c.cases.goldman_1mdb">Goldman Sachs / 1MDB $3.3B settlement (2020): partial restitution / penalty allocation</li>
                <li data-i18n="view.s162c.cases.hca">HCA $1.7B Medicare fraud settlement: anti-kickback dimension</li>
            </ul>
        </div>
    `;
    document.getElementById('s162c-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.payment_amount = Number(fd.get('payment_amount')) || 0;
        state.payment_type = fd.get('payment_type');
        state.payment_to = fd.get('payment_to');
        state.is_illegal_under_us = !!fd.get('is_illegal_under_us');
        state.is_illegal_under_foreign = !!fd.get('is_illegal_under_foreign');
        state.fcpa_violation = !!fd.get('fcpa_violation');
        state.sherman_act_violation = !!fd.get('sherman_act_violation');
        state.rico_violation = !!fd.get('rico_violation');
        state.medicare_medicaid_kickback = !!fd.get('medicare_medicaid_kickback');
        state.cooperative_in_investigation = !!fd.get('cooperative_in_investigation');
        state.is_corporation = !!fd.get('is_corporation');
        state.is_individual = !!fd.get('is_individual');
        state.proven_via_doj = !!fd.get('proven_via_doj');
        state.settled_civil_only = !!fd.get('settled_civil_only');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s162c-output');
    if (!el) return;
    const nondeductible = state.is_illegal_under_us || state.fcpa_violation || state.medicare_medicaid_kickback;
    const disallowed = nondeductible ? state.payment_amount : 0;
    const taxRate = state.is_corporation ? 0.21 : 0.37;
    const lostBenefit = disallowed * taxRate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s162c.h2.result">§ 162(c) determination</h2>
            <div class="cards">
                <div class="card ${nondeductible ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s162c.card.illegal">Nondeductible?</div>
                    <div class="value">${nondeductible ? esc(t('view.s162c.status.yes')) : esc(t('view.s162c.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s162c.card.amount">Payment amount</div>
                    <div class="value">$${state.payment_amount.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s162c.card.disallowed">Disallowed deduction</div>
                    <div class="value">$${disallowed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s162c.card.lost_tax">Lost tax benefit</div>
                    <div class="value">$${lostBenefit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${nondeductible ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s162c.illegal_note">
                    Payment NONDEDUCTIBLE under § 162(c). Even without DOJ prosecution / conviction, IRS may
                    disallow based on legal substance. FCPA + Anti-Kickback Statute violations particularly
                    high risk. Consider self-disclosure to DOJ for cooperation credit + reduce penalty exposure.
                    Sec/SEC + DOJ + IRS exchange information.
                </p>
            ` : ''}
        </div>
    `;
}
