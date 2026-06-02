// IRC § 162(f) — Fines + Penalties Paid to Government.
// General rule: NO deduction for fines / similar penalties paid to government / governmental entity.
// TCJA 2017: expanded to ALL governmental + clarified disallowance.
// Exceptions: (1) restitution, (2) amounts paid for governmental investigation/inquiry compliance.
// § 6050X reporting: government MUST report payments to IRS (Form 1098-F).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    total_payment: 0,
    payment_type: 'fine',
    payor_type: 'corporation',
    identified_restitution: 0,
    identified_compliance: 0,
    settlement_with_admission: false,
    settlement_without_admission: false,
    court_order: false,
    consent_decree: false,
    pre_2018_tcja: false,
    form_1098f_received: false,
    paid_to_government: true,
    paid_to_private_party: false,
    cooperating_witness: false,
    treble_damages_antitrust: 0,
};

export async function renderSection162F(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s162f.h1.title">// § 162(f) FINES + PENALTIES</span></h1>
        <p class="muted small" data-i18n="view.s162f.hint.intro">
            <strong>NO DEDUCTION</strong> for fines / similar penalties paid to government / governmental
            entity. <strong>TCJA 2017 expanded:</strong> all government, even foreign. <strong>Exceptions:</strong>
            (1) IDENTIFIED <strong>RESTITUTION</strong> (to victim), (2) IDENTIFIED <strong>COMPLIANCE</strong>
            costs. <strong>Settlement language CRITICAL:</strong> must specifically IDENTIFY which portion is
            restitution vs penalty. <strong>§ 6050X reporting:</strong> government MUST report payments to
            IRS via Form 1098-F. <strong>§ 162(g):</strong> 2/3 of antitrust treble damages also nondeductible.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s162f.h2.inputs">Inputs</h2>
            <form id="s162f-form" class="inline-form">
                <label><span data-i18n="view.s162f.label.payment">Total payment ($)</span>
                    <input type="number" step="10000" name="total_payment" value="${state.total_payment}"></label>
                <label><span data-i18n="view.s162f.label.type">Payment type</span>
                    <select name="payment_type">
                        <option value="fine" ${state.payment_type === 'fine' ? 'selected' : ''}>Fine (penalty)</option>
                        <option value="civil_penalty" ${state.payment_type === 'civil_penalty' ? 'selected' : ''}>Civil penalty</option>
                        <option value="restitution" ${state.payment_type === 'restitution' ? 'selected' : ''}>Restitution to victims</option>
                        <option value="compliance" ${state.payment_type === 'compliance' ? 'selected' : ''}>Compliance / remediation</option>
                        <option value="mixed" ${state.payment_type === 'mixed' ? 'selected' : ''}>Mixed (settlement)</option>
                        <option value="treble_antitrust" ${state.payment_type === 'treble_antitrust' ? 'selected' : ''}>Antitrust treble damages</option>
                    </select>
                </label>
                <label><span data-i18n="view.s162f.label.payor">Payor type</span>
                    <select name="payor_type">
                        <option value="corporation" ${state.payor_type === 'corporation' ? 'selected' : ''}>Corporation</option>
                        <option value="individual" ${state.payor_type === 'individual' ? 'selected' : ''}>Individual</option>
                        <option value="partnership" ${state.payor_type === 'partnership' ? 'selected' : ''}>Partnership</option>
                    </select>
                </label>
                <label><span data-i18n="view.s162f.label.restitution">Identified restitution ($)</span>
                    <input type="number" step="1000" name="identified_restitution" value="${state.identified_restitution}"></label>
                <label><span data-i18n="view.s162f.label.compliance">Identified compliance ($)</span>
                    <input type="number" step="1000" name="identified_compliance" value="${state.identified_compliance}"></label>
                <label><span data-i18n="view.s162f.label.admit">Settlement w/ admission?</span>
                    <input type="checkbox" name="settlement_with_admission" ${state.settlement_with_admission ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162f.label.no_admit">Settlement w/o admission?</span>
                    <input type="checkbox" name="settlement_without_admission" ${state.settlement_without_admission ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162f.label.court">Court order?</span>
                    <input type="checkbox" name="court_order" ${state.court_order ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162f.label.consent">Consent decree?</span>
                    <input type="checkbox" name="consent_decree" ${state.consent_decree ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162f.label.pre_tcja">Pre-2018 TCJA?</span>
                    <input type="checkbox" name="pre_2018_tcja" ${state.pre_2018_tcja ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162f.label.form_1098f">Form 1098-F received?</span>
                    <input type="checkbox" name="form_1098f_received" ${state.form_1098f_received ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162f.label.gov">Paid to government?</span>
                    <input type="checkbox" name="paid_to_government" ${state.paid_to_government ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162f.label.private">Paid to private party?</span>
                    <input type="checkbox" name="paid_to_private_party" ${state.paid_to_private_party ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162f.label.witness">Cooperating witness?</span>
                    <input type="checkbox" name="cooperating_witness" ${state.cooperating_witness ? 'checked' : ''}></label>
                <label><span data-i18n="view.s162f.label.treble">Treble damages antitrust ($)</span>
                    <input type="number" step="10000" name="treble_damages_antitrust" value="${state.treble_damages_antitrust}"></label>
                <button class="primary" type="submit" data-i18n="view.s162f.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s162f-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s162f.h2.scope">§ 162(f) scope</h2>
            <ul class="muted small">
                <li data-i18n="view.s162f.scope.fine">Fines + penalties paid to government for violation of law</li>
                <li data-i18n="view.s162f.scope.civil">Civil penalties (DOJ, EPA, SEC, CFTC, FTC, state attorneys general)</li>
                <li data-i18n="view.s162f.scope.regulatory">Regulatory penalties (FDIC, OCC, FERC, NRC)</li>
                <li data-i18n="view.s162f.scope.foreign_gov">Foreign government penalties</li>
                <li data-i18n="view.s162f.scope.consumer_protection">Consumer protection penalties (FTC, state AGs)</li>
                <li data-i18n="view.s162f.scope.environmental">Environmental penalties (EPA Clean Air, Clean Water, RCRA)</li>
                <li data-i18n="view.s162f.scope.banking">Banking + securities penalties</li>
                <li data-i18n="view.s162f.scope.opioid">Opioid settlements (recent wave): mostly NONDEDUCTIBLE under § 162(f)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s162f.h2.identified_test">"Identified restitution / compliance" — 2017 narrow exception</h2>
            <ul class="muted small">
                <li data-i18n="view.s162f.id.specific">Settlement / court order must SPECIFICALLY IDENTIFY portion as restitution / compliance</li>
                <li data-i18n="view.s162f.id.amount">Amount specifically identified (not generic / pro-rata after the fact)</li>
                <li data-i18n="view.s162f.id.allocation">Cannot ALLOCATE after the fact — must be PRE-SET in settlement document</li>
                <li data-i18n="view.s162f.id.regulation">Reg § 1.162-21 (2021 final regs) clarifies identification standard</li>
                <li data-i18n="view.s162f.id.allocation_rare">Specific dollar amount + clear restitution / compliance label required</li>
                <li data-i18n="view.s162f.id.reasonable_basis">"Reasonable basis to expect" amounts identified would be paid</li>
                <li data-i18n="view.s162f.id.preparer_burden">Document settlement carefully — IRS examiners challenge allocations</li>
                <li data-i18n="view.s162f.id.both_required">Both identification + actual payment required</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s162f.h2.1098f">§ 6050X Form 1098-F reporting</h2>
            <ul class="muted small">
                <li data-i18n="view.s162f.f.gov_files">Government MUST file Form 1098-F if total ≥ $50K</li>
                <li data-i18n="view.s162f.f.copy_to_taxpayer">Copy to taxpayer; copy to IRS</li>
                <li data-i18n="view.s162f.f.boxes">Form has separate boxes: (a) penalty, (b) restitution, (c) compliance, (d) other</li>
                <li data-i18n="view.s162f.f.consistent">Taxpayer must REPORT CONSISTENTLY with 1098-F amounts</li>
                <li data-i18n="view.s162f.f.dispute">Dispute: file amended 1098-F or claim different on 1040 with explanation</li>
                <li data-i18n="view.s162f.f.no_form_no_deduction">No 1098-F received: presumption is no restitution / compliance identified</li>
                <li data-i18n="view.s162f.f.timing">Filed in year payment made (cash basis for both parties)</li>
                <li data-i18n="view.s162f.f.exception">Exception: settlement payments to private plaintiffs not subject to 1098-F</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s162f.h2.related">§ 162(c), (g), (k), (l) related nondeductibles</h2>
            <ul class="muted small">
                <li data-i18n="view.s162f.rel.c">§ 162(c): illegal payments / kickbacks NONDEDUCTIBLE (foreign bribes too)</li>
                <li data-i18n="view.s162f.rel.g">§ 162(g): 2/3 of antitrust treble damages NONDEDUCTIBLE</li>
                <li data-i18n="view.s162f.rel.k">§ 162(k): healthcare premium for retirees subject to limits</li>
                <li data-i18n="view.s162f.rel.l">§ 162(l): SE health deduction above-the-line</li>
                <li data-i18n="view.s162f.rel.m">§ 162(m): $1M deduction limit on public co exec comp</li>
                <li data-i18n="view.s162f.rel.n">§ 162(n): catastrophic care payments</li>
                <li data-i18n="view.s162f.rel.r">§ 162(r): bank deposit insurance premiums limited</li>
                <li data-i18n="view.s162f.rel.s">Smith / Jenkins / Westmoreland cases established narrow restitution interpretation</li>
            </ul>
        </div>
    `;
    document.getElementById('s162f-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.total_payment = Number(fd.get('total_payment')) || 0;
        state.payment_type = fd.get('payment_type');
        state.payor_type = fd.get('payor_type');
        state.identified_restitution = Number(fd.get('identified_restitution')) || 0;
        state.identified_compliance = Number(fd.get('identified_compliance')) || 0;
        state.settlement_with_admission = !!fd.get('settlement_with_admission');
        state.settlement_without_admission = !!fd.get('settlement_without_admission');
        state.court_order = !!fd.get('court_order');
        state.consent_decree = !!fd.get('consent_decree');
        state.pre_2018_tcja = !!fd.get('pre_2018_tcja');
        state.form_1098f_received = !!fd.get('form_1098f_received');
        state.paid_to_government = !!fd.get('paid_to_government');
        state.paid_to_private_party = !!fd.get('paid_to_private_party');
        state.cooperating_witness = !!fd.get('cooperating_witness');
        state.treble_damages_antitrust = Number(fd.get('treble_damages_antitrust')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s162f-output');
    if (!el) return;
    const baseNonDeductible = state.paid_to_government ? state.total_payment - state.identified_restitution - state.identified_compliance : 0;
    const trebleNonDeductible = state.treble_damages_antitrust * (2 / 3);
    const totalNonDeductible = baseNonDeductible + trebleNonDeductible;
    const totalDeductible = state.total_payment + state.treble_damages_antitrust - totalNonDeductible;
    const taxRate = state.payor_type === 'corporation' ? 0.21 : 0.37;
    const lostTaxBenefit = totalNonDeductible * taxRate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s162f.h2.result">§ 162(f) deductibility</h2>
            <div class="cards">
                <div class="card neg">
                    <div class="label" data-i18n="view.s162f.card.total">Total payment + treble</div>
                    <div class="value">$${(state.total_payment + state.treble_damages_antitrust).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s162f.card.restitution_id">Restitution identified</div>
                    <div class="value">$${state.identified_restitution.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s162f.card.compliance_id">Compliance identified</div>
                    <div class="value">$${state.identified_compliance.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s162f.card.nondeductible">Nondeductible</div>
                    <div class="value">$${totalNonDeductible.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s162f.card.deductible">Deductible</div>
                    <div class="value">$${totalDeductible.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s162f.card.lost_benefit">Lost tax benefit</div>
                    <div class="value">$${lostTaxBenefit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${totalNonDeductible > 0 && !state.form_1098f_received ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s162f.no_form_note">
                    No Form 1098-F received: IRS presumes ALL payment to government is nondeductible.
                    Critical pre-settlement step: NEGOTIATE settlement language identifying specific dollar
                    amounts as restitution / compliance. Post-settlement allocation usually too late.
                    Consult tax counsel during settlement negotiations.
                </p>
            ` : ''}
        </div>
    `;
}
