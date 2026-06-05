// IRC § 6695 — Other Tax Return Preparer Penalties.
// Various flat penalties for procedural failures by tax return preparers.
// § 6695(a): failure to furnish copy of return to taxpayer ($60 / return).
// § 6695(b): failure to sign as preparer ($60 / return).
// § 6695(c): failure to maintain list of returns prepared ($60 / return).
// § 6695(g): failure to perform due diligence on EITC, CTC, AOTC, HoH ($635 / failure).

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    preparer_type: 'paid_preparer',
    failures: {
        '6695a_no_copy': 0,
        '6695b_no_sign': 0,
        '6695c_no_list': 0,
        '6695d_no_id': 0,
        '6695e_no_records': 0,
        '6695f_check_neg': 0,
        '6695g_due_diligence': 0,
    },
    has_ptin: true,
    diligence_documented: false,
    e_file_provider: false,
    cur_max_per_return: 600,
    intentional_disregard: false,
    years_in_practice: 0,
    state_licensed: false,
    enrolled_agent: false,
    cpa: false,
    attorney: false,
};

export async function renderSection6695(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6695.h1.title">// § 6695 PREPARER PENALTIES</span></h1>
        <p class="muted small" data-i18n="view.s6695.hint.intro">
            Various <strong>FLAT PENALTIES</strong> for procedural failures by tax return preparers.
            <strong>§ 6695(a):</strong> failure to furnish copy of return to taxpayer — $60 / return.
            <strong>§ 6695(b):</strong> failure to sign as preparer — $60. <strong>§ 6695(c):</strong>
            failure to maintain list of returns prepared — $60. <strong>§ 6695(g):</strong> failure to
            perform due diligence on <strong>EITC, CTC, AOTC, HoH</strong> — <strong>$635 / failure</strong>
            (2024). <strong>Form 8867</strong> required for each refundable credit return. <strong>Annual
            penalty caps</strong> by failure type. <strong>Reasonable cause exception</strong> limited.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6695.h2.inputs">Inputs</h2>
            <form id="s6695-form" class="inline-form">
                <label><span data-i18n="view.s6695.label.type">Preparer type</span>
                    <select name="preparer_type">
                        <option value="paid_preparer" ${state.preparer_type === 'paid_preparer' ? 'selected' : ''}>Paid preparer (generic)</option>
                        <option value="cpa" ${state.preparer_type === 'cpa' ? 'selected' : ''}>CPA</option>
                        <option value="enrolled_agent" ${state.preparer_type === 'enrolled_agent' ? 'selected' : ''}>Enrolled Agent</option>
                        <option value="attorney" ${state.preparer_type === 'attorney' ? 'selected' : ''}>Attorney</option>
                        <option value="non_licensed" ${state.preparer_type === 'non_licensed' ? 'selected' : ''}>Non-licensed paid preparer</option>
                        <option value="volunteer" ${state.preparer_type === 'volunteer' ? 'selected' : ''}>Volunteer (VITA / TCE — exempt)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6695.label.6695a">§ 6695(a) failures (no copy)</span>
                    <input type="number" step="1" name="6695a_no_copy" value="${state.failures['6695a_no_copy']}"></label>
                <label><span data-i18n="view.s6695.label.6695b">§ 6695(b) failures (no sign)</span>
                    <input type="number" step="1" name="6695b_no_sign" value="${state.failures['6695b_no_sign']}"></label>
                <label><span data-i18n="view.s6695.label.6695c">§ 6695(c) failures (no list)</span>
                    <input type="number" step="1" name="6695c_no_list" value="${state.failures['6695c_no_list']}"></label>
                <label><span data-i18n="view.s6695.label.6695d">§ 6695(d) failures (no PTIN)</span>
                    <input type="number" step="1" name="6695d_no_id" value="${state.failures['6695d_no_id']}"></label>
                <label><span data-i18n="view.s6695.label.6695e">§ 6695(e) failures (no records)</span>
                    <input type="number" step="1" name="6695e_no_records" value="${state.failures['6695e_no_records']}"></label>
                <label><span data-i18n="view.s6695.label.6695f">§ 6695(f) check negotiation</span>
                    <input type="number" step="1" name="6695f_check_neg" value="${state.failures['6695f_check_neg']}"></label>
                <label><span data-i18n="view.s6695.label.6695g">§ 6695(g) DD failures</span>
                    <input type="number" step="1" name="6695g_due_diligence" value="${state.failures['6695g_due_diligence']}"></label>
                <label><span data-i18n="view.s6695.label.ptin">Has PTIN?</span>
                    <input type="checkbox" name="has_ptin" ${state.has_ptin ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6695.label.diligence">Diligence documented (Form 8867)?</span>
                    <input type="checkbox" name="diligence_documented" ${state.diligence_documented ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6695.label.efile">E-file provider?</span>
                    <input type="checkbox" name="e_file_provider" ${state.e_file_provider ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6695.label.max">Current max per return ($)</span>
                    <input type="number" step="0.01" name="cur_max_per_return" value="${state.cur_max_per_return}"></label>
                <label><span data-i18n="view.s6695.label.intentional">Intentional disregard?</span>
                    <input type="checkbox" name="intentional_disregard" ${state.intentional_disregard ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6695.label.years">Years in practice</span>
                    <input type="number" step="1" name="years_in_practice" value="${state.years_in_practice}"></label>
                <label><span data-i18n="view.s6695.label.licensed">State licensed?</span>
                    <input type="checkbox" name="state_licensed" ${state.state_licensed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6695.label.ea">Enrolled Agent?</span>
                    <input type="checkbox" name="enrolled_agent" ${state.enrolled_agent ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6695.label.cpa">CPA?</span>
                    <input type="checkbox" name="cpa" ${state.cpa ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6695.label.attorney">Attorney?</span>
                    <input type="checkbox" name="attorney" ${state.attorney ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6695.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6695-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6695.h2.subsections">§ 6695 subsections + penalties (2024 amounts)</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s6695.th.section">Section</th>
                    <th data-i18n="view.s6695.th.failure">Failure</th>
                    <th data-i18n="view.s6695.th.penalty">Penalty</th>
                    <th data-i18n="view.s6695.th.cap">Annual cap</th>
                </tr></thead>
                <tbody>
                    <tr><td>§ 6695(a)</td><td>Failure to furnish copy of return</td><td>$60 / return</td><td>$30,000</td></tr>
                    <tr><td>§ 6695(b)</td><td>Failure to sign return as preparer</td><td>$60 / return</td><td>$30,000</td></tr>
                    <tr><td>§ 6695(c)</td><td>Failure to maintain list of returns</td><td>$60 / return</td><td>$30,000</td></tr>
                    <tr><td>§ 6695(d)</td><td>Failure to furnish ID # (PTIN)</td><td>$60 / return</td><td>$30,000</td></tr>
                    <tr><td>§ 6695(e)</td><td>Failure to maintain records / due diligence backup</td><td>$60 / return</td><td>$30,000</td></tr>
                    <tr><td>§ 6695(f)</td><td>Negotiate refund check on taxpayer's behalf</td><td>$635 / check</td><td>NONE</td></tr>
                    <tr><td>§ 6695(g)</td><td>EITC / CTC / AOTC / HoH due diligence failure</td><td>$635 / failure</td><td>NONE</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6695.h2.s6695g_diligence">§ 6695(g) — EITC + CTC + AOTC + HoH due diligence</h2>
            <ul class="muted small">
                <li data-i18n="view.s6695.gg.scope">Refundable credits: EITC, Child Tax Credit (refundable), American Opportunity Tax Credit, Head of Household status</li>
                <li data-i18n="view.s6695.gg.form_8867">Form 8867 (Paid Preparer's Due Diligence Checklist) required EACH RETURN</li>
                <li data-i18n="view.s6695.gg.records">Retain ALL records 3 years from filing date</li>
                <li data-i18n="view.s6695.gg.questions">Required: ask appropriate questions + verify taxpayer eligibility</li>
                <li data-i18n="view.s6695.gg.documentation">Document: SSN cards, dependent records, qualifying expense receipts</li>
                <li data-i18n="view.s6695.gg.cant_solely_rely">Cannot rely solely on taxpayer's word — verify documentation</li>
                <li data-i18n="view.s6695.gg.penalty_per_credit">PENALTY PER CREDIT per return — multiple credits → multiple penalties</li>
                <li data-i18n="view.s6695.gg.cumulative">EITC + CTC + AOTC + HoH on same return = up to $635 × 4 = $2,540 per return</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6695.h2.reasonable_cause">§ 6694(b) reasonable cause exception</h2>
            <ul class="muted small">
                <li data-i18n="view.s6695.rc.standard">Limited reasonable cause exception (Boyle case strict)</li>
                <li data-i18n="view.s6695.rc.reliance">Reliance on substantial authority + adequate disclosure</li>
                <li data-i18n="view.s6695.rc.good_faith">Good faith effort to comply</li>
                <li data-i18n="view.s6695.rc.factors">Factors: experience, complexity of return, supervisor approval</li>
                <li data-i18n="view.s6695.rc.no_reliance">Cannot rely on taxpayer's representations without verification</li>
                <li data-i18n="view.s6695.rc.documentation">Document reasonable steps taken to verify</li>
                <li data-i18n="view.s6695.rc.first_time">First-time penalty: harder to abate (no automatic abatement)</li>
                <li data-i18n="view.s6695.rc.appeals">Appeals: petition Tax Court within 30 days of denial</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6695.h2.related_circular230">Related: Circular 230 ethics rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s6695.c230.suspension">Circular 230 § 10.50: suspension / disbarment for violations</li>
                <li data-i18n="view.s6695.c230.due_diligence">§ 10.22 due diligence as to accuracy</li>
                <li data-i18n="view.s6695.c230.conflicts">§ 10.29 conflicts of interest</li>
                <li data-i18n="view.s6695.c230.standards">§ 10.34 standards for tax returns + documents</li>
                <li data-i18n="view.s6695.c230.advertising">§ 10.30 advertising / solicitation restrictions</li>
                <li data-i18n="view.s6695.c230.fees">§ 10.27 fees: no contingent fees on original returns</li>
                <li data-i18n="view.s6695.c230.client_records">§ 10.28 returning client records on request</li>
                <li data-i18n="view.s6695.c230.opr">Office of Professional Responsibility enforces</li>
            </ul>
        </div>
    `;
    document.getElementById('s6695-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.preparer_type = fd.get('preparer_type');
        state.failures['6695a_no_copy'] = Number(fd.get('6695a_no_copy')) || 0;
        state.failures['6695b_no_sign'] = Number(fd.get('6695b_no_sign')) || 0;
        state.failures['6695c_no_list'] = Number(fd.get('6695c_no_list')) || 0;
        state.failures['6695d_no_id'] = Number(fd.get('6695d_no_id')) || 0;
        state.failures['6695e_no_records'] = Number(fd.get('6695e_no_records')) || 0;
        state.failures['6695f_check_neg'] = Number(fd.get('6695f_check_neg')) || 0;
        state.failures['6695g_due_diligence'] = Number(fd.get('6695g_due_diligence')) || 0;
        state.has_ptin = !!fd.get('has_ptin');
        state.diligence_documented = !!fd.get('diligence_documented');
        state.e_file_provider = !!fd.get('e_file_provider');
        state.cur_max_per_return = Number(fd.get('cur_max_per_return')) || 0;
        state.intentional_disregard = !!fd.get('intentional_disregard');
        state.years_in_practice = Number(fd.get('years_in_practice')) || 0;
        state.state_licensed = !!fd.get('state_licensed');
        state.enrolled_agent = !!fd.get('enrolled_agent');
        state.cpa = !!fd.get('cpa');
        state.attorney = !!fd.get('attorney');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6695-output');
    if (!el) return;
    const pen_60_subsections = ['6695a_no_copy', '6695b_no_sign', '6695c_no_list', '6695d_no_id', '6695e_no_records'];
    let pen_60_total = 0;
    pen_60_subsections.forEach(s => {
        pen_60_total += Math.min(state.failures[s] * 60, 30_000);
    });
    const pen_f = state.failures['6695f_check_neg'] * 635;
    const pen_g = state.failures['6695g_due_diligence'] * 635;
    const total_penalty = pen_60_total + pen_f + pen_g;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6695.h2.result">§ 6695 penalty computation</h2>
            <div class="cards">
                <div class="card neg">
                    <div class="label" data-i18n="view.s6695.card.pen60">$60 subsections total</div>
                    <div class="value">$${pen_60_total.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6695.card.pen_f">§ 6695(f) check negotiation</div>
                    <div class="value">$${pen_f.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6695.card.pen_g">§ 6695(g) due diligence</div>
                    <div class="value">$${pen_g.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6695.card.total">TOTAL PENALTY</div>
                    <div class="value">$${total_penalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${pen_g > 0 && !state.diligence_documented ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s6695.dd_note">
                    § 6695(g) due diligence FAILURES. Required: Form 8867 each return + retain 3-year records.
                    Penalty PER CREDIT per return — EITC + CTC + AOTC + HoH on same return = up to $635 × 4
                    = $2,540 per return. IRS preparer compliance unit (CDX) audits high-volume EITC preparers.
                    Implement systematic checklist + documentation procedures.
                </p>
            ` : ''}
        </div>
    `;
}
