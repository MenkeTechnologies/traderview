// IRC § 2518 — Qualified Disclaimer (estate planning rejection of inherited interest).
// Disclaimer effective if: (1) WRITTEN, (2) RECEIVED by transferor within 9 MONTHS of transfer,
// (3) UNCONDITIONAL/IRREVOCABLE, (4) DISCLAIMANT did NOT accept any benefit + (5) interest
// passes to spouse or other person without disclaimant's direction. Result: as if disclaimant
// predeceased decedent — interest passes to next-in-line beneficiary.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    is_written: false,
    days_since_transfer: 0,
    is_unconditional_irrevocable: false,
    accepted_benefit: false,
    passes_to_spouse: false,
    passes_to_other: true,
    disclaimer_direction: false,
    disclaimant_age: 0,
    disclaimed_amount: 0,
    transferor_type: 'decedent',
    transferred_property_type: 'cash',
    is_partial_disclaimer: false,
    partial_amount: 0,
    received_state_law_compliant: false,
    state_of_disclaimer: 'NY',
    spouse_disclaimer: false,
    gst_implications: false,
    is_qtip_disclaimer: false,
    multiple_beneficiaries: false,
};

export async function renderSection2518(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s2518.h1.title">// § 2518 QUALIFIED DISCLAIMER</span></h1>
        <p class="muted small" data-i18n="view.s2518.hint.intro">
            <strong>"Qualified disclaimer"</strong> = refusal to accept inherited interest. <strong>5
            requirements</strong>: (1) WRITTEN + signed, (2) RECEIVED by transferor (estate / trustee /
            holder) within <strong>9 months</strong> of transfer (or age 21 for minor disclaimant),
            (3) UNCONDITIONAL + irrevocable, (4) disclaimant did NOT accept any benefit (income / use /
            distribution), (5) interest passes <strong>without disclaimant's direction</strong> either
            to surviving spouse OR to person other than disclaimant. <strong>Tax result:</strong>
            treated as IF disclaimant PREDECEASED — disclaimed interest passes to next-in-line under
            governing instrument. <strong>NOT</strong> a gift from disclaimant under § 2511. State law
            disclaimer compliance separately required (UDPIA + state-specific).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s2518.h2.inputs">Inputs</h2>
            <form id="s2518-form" class="inline-form">
                <label><span data-i18n="view.s2518.label.written">Written + signed?</span>
                    <input type="checkbox" name="is_written" ${state.is_written ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2518.label.days">Days since transfer</span>
                    <input type="number" step="1" name="days_since_transfer" value="${state.days_since_transfer}"></label>
                <label><span data-i18n="view.s2518.label.unconditional">Unconditional + irrevocable?</span>
                    <input type="checkbox" name="is_unconditional_irrevocable" ${state.is_unconditional_irrevocable ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2518.label.benefit">Disclaimant accepted benefit?</span>
                    <input type="checkbox" name="accepted_benefit" ${state.accepted_benefit ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2518.label.spouse">Passes to spouse?</span>
                    <input type="checkbox" name="passes_to_spouse" ${state.passes_to_spouse ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2518.label.other">Passes to other?</span>
                    <input type="checkbox" name="passes_to_other" ${state.passes_to_other ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2518.label.direction">Disclaimant directed?</span>
                    <input type="checkbox" name="disclaimer_direction" ${state.disclaimer_direction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2518.label.age">Disclaimant age</span>
                    <input type="number" step="1" name="disclaimant_age" value="${state.disclaimant_age}"></label>
                <label><span data-i18n="view.s2518.label.amount">Disclaimed amount ($)</span>
                    <input type="number" step="10000" name="disclaimed_amount" value="${state.disclaimed_amount}"></label>
                <label><span data-i18n="view.s2518.label.transferor">Transferor</span>
                    <select name="transferor_type">
                        <option value="decedent" ${state.transferor_type === 'decedent' ? 'selected' : ''}>Decedent (estate)</option>
                        <option value="donor" ${state.transferor_type === 'donor' ? 'selected' : ''}>Donor (inter vivos)</option>
                        <option value="trust" ${state.transferor_type === 'trust' ? 'selected' : ''}>Trust (distribution)</option>
                        <option value="joint_tenancy" ${state.transferor_type === 'joint_tenancy' ? 'selected' : ''}>Joint tenancy (death of co-T)</option>
                        <option value="retirement_account" ${state.transferor_type === 'retirement_account' ? 'selected' : ''}>Retirement account (IRA/401(k))</option>
                    </select>
                </label>
                <label><span data-i18n="view.s2518.label.prop_type">Property type</span>
                    <select name="transferred_property_type">
                        <option value="cash" ${state.transferred_property_type === 'cash' ? 'selected' : ''}>Cash</option>
                        <option value="securities" ${state.transferred_property_type === 'securities' ? 'selected' : ''}>Securities</option>
                        <option value="real_estate" ${state.transferred_property_type === 'real_estate' ? 'selected' : ''}>Real estate</option>
                        <option value="business_interest" ${state.transferred_property_type === 'business_interest' ? 'selected' : ''}>Business interest</option>
                        <option value="life_insurance" ${state.transferred_property_type === 'life_insurance' ? 'selected' : ''}>Life insurance proceeds</option>
                        <option value="ira" ${state.transferred_property_type === 'ira' ? 'selected' : ''}>IRA / 401(k)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s2518.label.partial">Partial disclaimer?</span>
                    <input type="checkbox" name="is_partial_disclaimer" ${state.is_partial_disclaimer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2518.label.partial_amt">Partial amount ($)</span>
                    <input type="number" step="10000" name="partial_amount" value="${state.partial_amount}"></label>
                <label><span data-i18n="view.s2518.label.state_compliant">State law compliant?</span>
                    <input type="checkbox" name="received_state_law_compliant" ${state.received_state_law_compliant ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2518.label.state">State of disclaimer</span>
                    <input type="text" name="state_of_disclaimer" value="${esc(state.state_of_disclaimer)}"></label>
                <label><span data-i18n="view.s2518.label.spouse_disclaim">Spouse disclaiming?</span>
                    <input type="checkbox" name="spouse_disclaimer" ${state.spouse_disclaimer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2518.label.gst">GST implications?</span>
                    <input type="checkbox" name="gst_implications" ${state.gst_implications ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2518.label.qtip">QTIP disclaimer?</span>
                    <input type="checkbox" name="is_qtip_disclaimer" ${state.is_qtip_disclaimer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s2518.label.multi">Multiple beneficiaries?</span>
                    <input type="checkbox" name="multiple_beneficiaries" ${state.multiple_beneficiaries ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s2518.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s2518-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s2518.h2.five">5-requirement test (§ 2518(b))</h2>
            <ol class="muted small">
                <li data-i18n="view.s2518.five.written">WRITTEN refusal — signed by disclaimant or legal rep</li>
                <li data-i18n="view.s2518.five.timing">9 months from later of: (a) date of transfer creating interest, OR (b) age 21 for minor</li>
                <li data-i18n="view.s2518.five.unconditional">UNCONDITIONAL + irrevocable — no conditions, no power to revoke</li>
                <li data-i18n="view.s2518.five.benefit">No acceptance of benefit (income/principal/use)</li>
                <li data-i18n="view.s2518.five.passes">Passes WITHOUT disclaimant direction (governing instrument controls)</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s2518.h2.spouse">Special rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s2518.special.spouse_redirect">§ 2518(b)(4)(A) — surviving spouse may disclaim AND interest passes to spouse</li>
                <li data-i18n="view.s2518.special.spouse_exception">Otherwise, disclaimed interest CANNOT pass to disclaimant</li>
                <li data-i18n="view.s2518.special.minor">Minor (under 21): 9-month period runs from age 21, not date of transfer</li>
                <li data-i18n="view.s2518.special.partial_undivided">Partial disclaimer: must be UNDIVIDED interest (severable share)</li>
                <li data-i18n="view.s2518.special.formula">Formula clause disclaimer permitted (e.g., "amount equal to applicable exclusion")</li>
                <li data-i18n="view.s2518.special.ira_separate">IRA / retirement: each beneficiary disclaims separately</li>
                <li data-i18n="view.s2518.special.fmv_date">FMV at date of disclaimer (not date of original transfer)</li>
                <li data-i18n="view.s2518.special.power_of_appointment">General power of appointment may be disclaimed</li>
                <li data-i18n="view.s2518.special.no_post_mortem">NO post-mortem election after 9-month deadline</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s2518.h2.use_cases">Disclaimer use cases</h2>
            <ul class="muted small">
                <li data-i18n="view.s2518.uc.applicable_exclusion">Capture decedent's applicable exclusion (formerly $5M, now $13.99M 2024)</li>
                <li data-i18n="view.s2518.uc.bypass_trust">Disclaimer to bypass trust funding (reduces surviving spouse's estate)</li>
                <li data-i18n="view.s2518.uc.skip_generation">Disclaimer to skip generation (children disclaim → grandchildren — GST issues)</li>
                <li data-i18n="view.s2518.uc.poor_planning">Post-mortem correction of pre-mortem planning errors</li>
                <li data-i18n="view.s2518.uc.gst_planning">GST allocation optimization (formulaic disclaimer)</li>
                <li data-i18n="view.s2518.uc.creditor">Creditor protection: disclaimed property never owned by disclaimant</li>
                <li data-i18n="view.s2518.uc.elderly">Elderly child / financially independent — disclaim in favor of next gen</li>
                <li data-i18n="view.s2518.uc.qtip_election">Disclaim to enable QTIP election + reverse direction</li>
                <li data-i18n="view.s2518.uc.charitable">Disclaim residue → charity (creates § 2055 deduction)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s2518.h2.failed">Failed disclaimer consequences</h2>
            <ul class="muted small">
                <li data-i18n="view.s2518.fail.deemed_gift">Treated as accepted + new gift from disclaimant to next beneficiary</li>
                <li data-i18n="view.s2518.fail.gift_tax">§ 2511 gift tax applies — file Form 709 + use lifetime exemption</li>
                <li data-i18n="view.s2518.fail.cost_basis">Disclaimant becomes property owner — § 1014 step-up still applies (was decedent's)</li>
                <li data-i18n="view.s2518.fail.income_tax">Disclaimant taxable on subsequent income</li>
                <li data-i18n="view.s2518.fail.state_law_only">State-law disclaimer may still effective for state purposes</li>
                <li data-i18n="view.s2518.fail.late_filing">9-month deadline is strict — no equitable tolling</li>
                <li data-i18n="view.s2518.fail.partial_consequences">Partial disclaimer with conditions = total failure (not partial)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s2518.h2.state_law">State law (UDPIA + common law)</h2>
            <ul class="muted small">
                <li data-i18n="view.s2518.state.udpia">Uniform Disclaimer of Property Interests Act (UDPIA, 2002)</li>
                <li data-i18n="view.s2518.state.adoption">Adopted in 40+ states (CA, NY, IL, TX vary)</li>
                <li data-i18n="view.s2518.state.no_time_limit">UDPIA: NO state law time limit (federal 9-month controls for tax)</li>
                <li data-i18n="view.s2518.state.fiduciary">Permits fiduciary (executor / trustee / guardian) disclaimer</li>
                <li data-i18n="view.s2518.state.unborn">Unborn beneficiary: virtual representation may apply</li>
                <li data-i18n="view.s2518.state.recording">Real property: recording requirement (varies state)</li>
                <li data-i18n="view.s2518.state.notice">Personal notice to remaining beneficiaries (UDPIA)</li>
                <li data-i18n="view.s2518.state.medicaid">Medicaid: disclaimer = uncompensated transfer + 5-yr lookback</li>
            </ul>
        </div>
    `;
    document.getElementById('s2518-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.is_written = !!fd.get('is_written');
        state.days_since_transfer = Number(fd.get('days_since_transfer')) || 0;
        state.is_unconditional_irrevocable = !!fd.get('is_unconditional_irrevocable');
        state.accepted_benefit = !!fd.get('accepted_benefit');
        state.passes_to_spouse = !!fd.get('passes_to_spouse');
        state.passes_to_other = !!fd.get('passes_to_other');
        state.disclaimer_direction = !!fd.get('disclaimer_direction');
        state.disclaimant_age = Number(fd.get('disclaimant_age')) || 0;
        state.disclaimed_amount = Number(fd.get('disclaimed_amount')) || 0;
        state.transferor_type = fd.get('transferor_type');
        state.transferred_property_type = fd.get('transferred_property_type');
        state.is_partial_disclaimer = !!fd.get('is_partial_disclaimer');
        state.partial_amount = Number(fd.get('partial_amount')) || 0;
        state.received_state_law_compliant = !!fd.get('received_state_law_compliant');
        state.state_of_disclaimer = fd.get('state_of_disclaimer') || '';
        state.spouse_disclaimer = !!fd.get('spouse_disclaimer');
        state.gst_implications = !!fd.get('gst_implications');
        state.is_qtip_disclaimer = !!fd.get('is_qtip_disclaimer');
        state.multiple_beneficiaries = !!fd.get('multiple_beneficiaries');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s2518-output');
    if (!el) return;
    const timing_satisfied = state.days_since_transfer > 0 && state.days_since_transfer <= 273;  // ~9 months
    const directs_check = !state.disclaimer_direction;
    const passes_correctly = state.passes_to_spouse || state.passes_to_other;
    const all_satisfied =
        state.is_written && timing_satisfied && state.is_unconditional_irrevocable
        && !state.accepted_benefit && passes_correctly && directs_check;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s2518.h2.result">§ 2518 qualification check</h2>
            <div class="cards">
                <div class="card ${state.is_written ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s2518.card.written">Written?</div>
                    <div class="value">${state.is_written ? 'YES' : 'NO'}</div>
                </div>
                <div class="card ${timing_satisfied ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s2518.card.timing">Within 9 months?</div>
                    <div class="value">${timing_satisfied ? 'YES' : 'NO'} (${state.days_since_transfer}d)</div>
                </div>
                <div class="card ${state.is_unconditional_irrevocable ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s2518.card.unconditional">Unconditional?</div>
                    <div class="value">${state.is_unconditional_irrevocable ? 'YES' : 'NO'}</div>
                </div>
                <div class="card ${!state.accepted_benefit ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s2518.card.benefit">No benefit accepted?</div>
                    <div class="value">${!state.accepted_benefit ? 'YES' : 'NO'}</div>
                </div>
                <div class="card ${passes_correctly && directs_check ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s2518.card.passes">Passes correctly?</div>
                    <div class="value">${passes_correctly && directs_check ? 'YES' : 'NO'}</div>
                </div>
                <div class="card ${all_satisfied ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s2518.card.qualifies">Qualified disclaimer?</div>
                    <div class="value">${all_satisfied ? 'YES' : 'NO'}</div>
                </div>
            </div>
            ${!all_satisfied ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s2518.failed_note">
                    Disclaimer FAILS § 2518 — treated as accepted + gift from disclaimant to next
                    beneficiary. Triggers § 2511 gift tax (lifetime exemption $13.99M 2024). State-law
                    disclaimer may still effective for non-tax purposes (creditor protection / Medicaid).
                </p>
            ` : ''}
        </div>
    `;
}
