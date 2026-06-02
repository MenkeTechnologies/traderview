// IRC § 6041 — Information Reporting on Form 1099-MISC + Form 1099-NEC.
// Payers must report $600+ payments to non-corp persons for services (NEC), rent, prizes, royalties (MISC).
// Threshold: $600 / yr per recipient (NEC for services since 2020 PATH Act).
// Form 1099-NEC: Box 1 nonemployee compensation. Form 1099-MISC: Boxes 1-14 various.
// Failure to file: § 6721 ($60-$330 per failure + intentional disregard $660+).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    payment_type: 'nonemployee_compensation',
    total_payment_year: 0,
    recipient_type: 'individual',
    recipient_is_corp: false,
    recipient_provided_tin: true,
    backup_withholding_applied: false,
    backup_withholding_amount: 0,
    e_file_required: false,
    deadline_met: true,
    tax_year: 2024,
    payment_subject_to_se_tax: false,
    medical_provider_exception: false,
    attorney_fees: false,
    rent_box1: 0,
    royalties_box2: 0,
};

export async function renderSection6041(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6041.h1.title">// § 6041 INFO REPORTING (1099)</span></h1>
        <p class="muted small" data-i18n="view.s6041.hint.intro">
            Payers must report <strong>$600+ payments</strong> to non-corp persons for services (NEC), rent,
            prizes, royalties (MISC). <strong>Threshold:</strong> $600 / yr per recipient. <strong>Form
            1099-NEC</strong> (Box 1 nonemployee compensation) — since PATH Act 2020 separate from MISC.
            <strong>Form 1099-MISC</strong> (Boxes 1-14: rent, royalties, prizes, medical, attorney fees, etc.).
            <strong>Corporations exempt</strong> EXCEPT: attorneys, medical providers, federally-regulated.
            <strong>Backup withholding 24%</strong> if no TIN. <strong>Form 1096</strong> transmittal.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6041.h2.inputs">Inputs</h2>
            <form id="s6041-form" class="inline-form">
                <label><span data-i18n="view.s6041.label.type">Payment type</span>
                    <select name="payment_type">
                        <option value="nonemployee_compensation" ${state.payment_type === 'nonemployee_compensation' ? 'selected' : ''}>Nonemployee comp (1099-NEC Box 1)</option>
                        <option value="rents" ${state.payment_type === 'rents' ? 'selected' : ''}>Rents (1099-MISC Box 1)</option>
                        <option value="royalties" ${state.payment_type === 'royalties' ? 'selected' : ''}>Royalties (1099-MISC Box 2)</option>
                        <option value="other_income" ${state.payment_type === 'other_income' ? 'selected' : ''}>Other income (1099-MISC Box 3)</option>
                        <option value="medical_health" ${state.payment_type === 'medical_health' ? 'selected' : ''}>Medical / healthcare (1099-MISC Box 6)</option>
                        <option value="attorney_payments" ${state.payment_type === 'attorney_payments' ? 'selected' : ''}>Attorney payments (1099-MISC Box 10)</option>
                        <option value="prizes_awards" ${state.payment_type === 'prizes_awards' ? 'selected' : ''}>Prizes + awards (1099-MISC Box 3)</option>
                        <option value="crop_insurance" ${state.payment_type === 'crop_insurance' ? 'selected' : ''}>Crop insurance proceeds (1099-MISC Box 9)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6041.label.total">Total payment for year ($)</span>
                    <input type="number" step="100" name="total_payment_year" value="${state.total_payment_year}"></label>
                <label><span data-i18n="view.s6041.label.recipient">Recipient type</span>
                    <select name="recipient_type">
                        <option value="individual" ${state.recipient_type === 'individual' ? 'selected' : ''}>Individual</option>
                        <option value="sole_proprietorship" ${state.recipient_type === 'sole_proprietorship' ? 'selected' : ''}>Sole proprietorship</option>
                        <option value="partnership" ${state.recipient_type === 'partnership' ? 'selected' : ''}>Partnership</option>
                        <option value="llc_disregarded" ${state.recipient_type === 'llc_disregarded' ? 'selected' : ''}>LLC (disregarded)</option>
                        <option value="llc_partnership" ${state.recipient_type === 'llc_partnership' ? 'selected' : ''}>LLC (partnership)</option>
                        <option value="c_corp" ${state.recipient_type === 'c_corp' ? 'selected' : ''}>C-corp</option>
                        <option value="s_corp" ${state.recipient_type === 's_corp' ? 'selected' : ''}>S-corp</option>
                        <option value="trust" ${state.recipient_type === 'trust' ? 'selected' : ''}>Trust / estate</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6041.label.is_corp">Is corporation?</span>
                    <input type="checkbox" name="recipient_is_corp" ${state.recipient_is_corp ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6041.label.tin">Recipient provided W-9 / TIN?</span>
                    <input type="checkbox" name="recipient_provided_tin" ${state.recipient_provided_tin ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6041.label.backup">Backup withholding applied?</span>
                    <input type="checkbox" name="backup_withholding_applied" ${state.backup_withholding_applied ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6041.label.backup_amt">Backup withholding amount ($)</span>
                    <input type="number" step="10" name="backup_withholding_amount" value="${state.backup_withholding_amount}"></label>
                <label><span data-i18n="view.s6041.label.efile">E-file required?</span>
                    <input type="checkbox" name="e_file_required" ${state.e_file_required ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6041.label.deadline">Deadline met?</span>
                    <input type="checkbox" name="deadline_met" ${state.deadline_met ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6041.label.year">Tax year</span>
                    <input type="number" step="1" name="tax_year" value="${state.tax_year}"></label>
                <label><span data-i18n="view.s6041.label.se">Subject to SE tax?</span>
                    <input type="checkbox" name="payment_subject_to_se_tax" ${state.payment_subject_to_se_tax ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6041.label.medical_exc">Medical provider exception?</span>
                    <input type="checkbox" name="medical_provider_exception" ${state.medical_provider_exception ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6041.label.attorney_exc">Attorney fees exception?</span>
                    <input type="checkbox" name="attorney_fees" ${state.attorney_fees ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6041.label.rent">Rent box 1 ($)</span>
                    <input type="number" step="100" name="rent_box1" value="${state.rent_box1}"></label>
                <label><span data-i18n="view.s6041.label.royalty">Royalties box 2 ($)</span>
                    <input type="number" step="100" name="royalties_box2" value="${state.royalties_box2}"></label>
                <button class="primary" type="submit" data-i18n="view.s6041.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6041-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6041.h2.thresholds">Reporting thresholds by box</h2>
            <ul class="muted small">
                <li data-i18n="view.s6041.thr.nec">1099-NEC Box 1 (nonemployee comp): ≥ $600 (since 2020)</li>
                <li data-i18n="view.s6041.thr.rents">1099-MISC Box 1 (rents): ≥ $600</li>
                <li data-i18n="view.s6041.thr.royalties">1099-MISC Box 2 (royalties): ≥ $10</li>
                <li data-i18n="view.s6041.thr.other_income">1099-MISC Box 3 (other income / prizes): ≥ $600</li>
                <li data-i18n="view.s6041.thr.fishing">1099-MISC Box 5 (fishing boat proceeds): any amount</li>
                <li data-i18n="view.s6041.thr.medical">1099-MISC Box 6 (medical / healthcare): ≥ $600</li>
                <li data-i18n="view.s6041.thr.attorney">1099-MISC Box 10 (attorney payments): ≥ $600 (incl corp lawyers)</li>
                <li data-i18n="view.s6041.thr.crop">1099-MISC Box 9 (crop insurance): ≥ $600</li>
                <li data-i18n="view.s6041.thr.golden_parachute">1099-MISC Box 13 (golden parachute): no threshold</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6041.h2.exceptions">Corporation exceptions</h2>
            <ul class="muted small">
                <li data-i18n="view.s6041.exc.corp_default">Corporations generally NOT subject to § 6041 reporting</li>
                <li data-i18n="view.s6041.exc.attorneys">EXCEPT: attorney fees (always reportable, even to law firm corp)</li>
                <li data-i18n="view.s6041.exc.medical">EXCEPT: medical / healthcare payments (always reportable)</li>
                <li data-i18n="view.s6041.exc.fish_purchases">EXCEPT: cash fish purchases (always reportable)</li>
                <li data-i18n="view.s6041.exc.federal_govt">EXCEPT: federal executive agencies + federal contractors</li>
                <li data-i18n="view.s6041.exc.s_corp">S-corps + LLCs (electing corp status): generally treated as corps</li>
                <li data-i18n="view.s6041.exc.llc">LLCs: depends on TIN type (corp = exempt; partnership = reportable)</li>
                <li data-i18n="view.s6041.exc.foreign">Foreign persons: Form 1042 / 1042-S separate regime</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6041.h2.backup_withholding">Backup withholding (§ 3406)</h2>
            <ul class="muted small">
                <li data-i18n="view.s6041.bw.rate">Rate: 24% on REPORTABLE PAYMENT (since 2018 TCJA reduction from 28%)</li>
                <li data-i18n="view.s6041.bw.triggers">Triggers: (1) no TIN provided, (2) incorrect TIN, (3) IRS notification (B-Notice)</li>
                <li data-i18n="view.s6041.bw.w9">Solution: get Form W-9 (Request for Taxpayer Identification Number)</li>
                <li data-i18n="view.s6041.bw.b_notice">B-Notice / C-Notice: IRS sends to payer when discrepancy detected</li>
                <li data-i18n="view.s6041.bw.recipient_credit">Recipient credits BW against income tax on 1040 / 1120</li>
                <li data-i18n="view.s6041.bw.report_box4">Report on Form 1099 Box 4 (Federal Income Tax Withheld)</li>
                <li data-i18n="view.s6041.bw.remit_945">Remit via Form 945 (Annual Return of Withheld Federal Income Tax)</li>
                <li data-i18n="view.s6041.bw.penalty">Payer penalty: 100% of unwithheld amount + interest if failure</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6041.h2.deadlines">Filing deadlines</h2>
            <ul class="muted small">
                <li data-i18n="view.s6041.dl.nec_recipient">1099-NEC: to recipient by January 31</li>
                <li data-i18n="view.s6041.dl.nec_irs">1099-NEC: to IRS by January 31 (paper + electronic)</li>
                <li data-i18n="view.s6041.dl.misc_recipient">1099-MISC: to recipient by January 31 (Boxes 8, 10 — by February 15)</li>
                <li data-i18n="view.s6041.dl.misc_irs_paper">1099-MISC: to IRS by February 28 (paper) / March 31 (electronic)</li>
                <li data-i18n="view.s6041.dl.efile_threshold">E-file mandatory: 10+ returns aggregate (post-2024 IRS regs)</li>
                <li data-i18n="view.s6041.dl.extension">Extension: Form 8809 (30 days automatic, may extend 30 more)</li>
                <li data-i18n="view.s6041.dl.correction">Correction: file CORR 1099 promptly when error discovered</li>
                <li data-i18n="view.s6041.dl.weekend">Weekend / holiday: next business day</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6041.h2.penalties">§ 6721 + § 6722 penalties for failure</h2>
            <ul class="muted small">
                <li data-i18n="view.s6041.pen.tier1">Tier 1: $60 per form if corrected within 30 days (max $230K)</li>
                <li data-i18n="view.s6041.pen.tier2">Tier 2: $120 per form if corrected by August 1 (max $683K)</li>
                <li data-i18n="view.s6041.pen.tier3">Tier 3: $330 per form if NOT corrected by August 1 (max $1.37M)</li>
                <li data-i18n="view.s6041.pen.intentional">Intentional disregard: $660 per form + UNLIMITED (no cap)</li>
                <li data-i18n="view.s6041.pen.small_biz">Small biz ($5M or less): reduced caps ($79K / $239K / $479K)</li>
                <li data-i18n="view.s6041.pen.s6722">§ 6722 statement to payee: similar penalties for failure to furnish</li>
                <li data-i18n="view.s6041.pen.reasonable_cause">Reasonable cause exception: Boyle case strict (Form 1099 disputes rare)</li>
                <li data-i18n="view.s6041.pen.de_minimis">De minimis: 10 or less incorrect forms (or 1% of total) — no penalty</li>
            </ul>
        </div>
    `;
    document.getElementById('s6041-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.payment_type = fd.get('payment_type');
        state.total_payment_year = Number(fd.get('total_payment_year')) || 0;
        state.recipient_type = fd.get('recipient_type');
        state.recipient_is_corp = !!fd.get('recipient_is_corp');
        state.recipient_provided_tin = !!fd.get('recipient_provided_tin');
        state.backup_withholding_applied = !!fd.get('backup_withholding_applied');
        state.backup_withholding_amount = Number(fd.get('backup_withholding_amount')) || 0;
        state.e_file_required = !!fd.get('e_file_required');
        state.deadline_met = !!fd.get('deadline_met');
        state.tax_year = Number(fd.get('tax_year')) || 0;
        state.payment_subject_to_se_tax = !!fd.get('payment_subject_to_se_tax');
        state.medical_provider_exception = !!fd.get('medical_provider_exception');
        state.attorney_fees = !!fd.get('attorney_fees');
        state.rent_box1 = Number(fd.get('rent_box1')) || 0;
        state.royalties_box2 = Number(fd.get('royalties_box2')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6041-output');
    if (!el) return;
    const threshold = state.payment_type === 'royalties' ? 10 : 600;
    const corp_exception_applies = state.recipient_is_corp && !state.medical_provider_exception && !state.attorney_fees;
    const reportable = state.total_payment_year >= threshold && !corp_exception_applies;
    const backup_required = state.total_payment_year >= threshold && !state.recipient_provided_tin;
    const backup_amt = backup_required ? state.total_payment_year * 0.24 : 0;
    const penalty_per_form = !state.deadline_met ? 60 : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6041.h2.result">§ 6041 reporting determination</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s6041.card.threshold">Threshold</div>
                    <div class="value">$${threshold.toLocaleString()}</div>
                </div>
                <div class="card ${reportable ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6041.card.reportable">Reportable?</div>
                    <div class="value">${reportable ? esc(t('view.s6041.status.yes')) : esc(t('view.s6041.status.no'))}</div>
                </div>
                <div class="card ${corp_exception_applies ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s6041.card.corp_exc">Corp exception applies?</div>
                    <div class="value">${corp_exception_applies ? esc(t('view.s6041.status.yes')) : esc(t('view.s6041.status.no'))}</div>
                </div>
                <div class="card ${backup_required ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s6041.card.bw_required">Backup withholding required?</div>
                    <div class="value">${backup_required ? esc(t('view.s6041.status.yes')) : esc(t('view.s6041.status.no'))}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6041.card.bw_amount">Backup withholding amount</div>
                    <div class="value">$${backup_amt.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6041.card.penalty">Penalty per form</div>
                    <div class="value">$${penalty_per_form.toLocaleString()}</div>
                </div>
            </div>
            ${backup_required ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s6041.bw_note">
                    BACKUP WITHHOLDING REQUIRED — no W-9 / TIN from recipient. Withhold 24% from each
                    payment + remit via Form 945 + report Box 4 of 1099. Recipient claims credit on
                    income tax return. Failure to withhold: payer personally liable for entire amount.
                    Obtain W-9 IMMEDIATELY before next payment.
                </p>
            ` : ''}
        </div>
    `;
}
