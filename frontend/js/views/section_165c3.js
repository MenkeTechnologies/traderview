// IRC § 165(c)(3) — Personal Casualty + Theft Loss (Federally-Declared Disaster only post-TCJA).
// TCJA 2018-2025: only federally-declared disaster casualty losses allowed for non-business.
// Loss = lesser of (1) basis - insurance OR (2) decline in FMV.
// Reduce by (a) $100 per casualty (b) 10% of AGI.
// § 165(i): elect to claim loss in PRIOR year (faster refund).

import { currentViewToken, viewIsCurrent } from '../app.js';

const PER_CASUALTY_REDUCTION = 100;
const AGI_FLOOR_PCT = 0.10;

let state = {
    basis_in_property: 0,
    fmv_before: 0,
    fmv_after: 0,
    insurance_reimbursement: 0,
    is_federally_declared_disaster: false,
    is_business_use: false,
    business_use_pct: 0,
    agi: 0,
    elect_prior_year: false,
    is_qualified_disaster_2024: false,
    marginal_rate: 0.32,
};

export async function renderSection165c3(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s165c3.h1.title">// § 165(c)(3) CASUALTY LOSS</span></h1>
        <p class="muted small" data-i18n="view.s165c3.hint.intro">
            <strong>TCJA 2018-2025:</strong> ONLY federally-declared disaster casualty losses
            allowed for non-business. Loss = lesser of <strong>(1) basis - insurance OR (2) decline
            in FMV</strong>. Reduce by <strong>(a) $100 per casualty</strong> + <strong>(b) 10%
            of AGI</strong>. <strong>§ 165(i) election:</strong> claim loss in PRIOR year for
            faster refund (must file by amended return or original return within statute).
            <strong>Qualified disaster losses (special):</strong> $500 floor + no 10% AGI test.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s165c3.h2.inputs">Inputs</h2>
            <form id="s165c3-form" class="inline-form">
                <label><span data-i18n="view.s165c3.label.basis">Property basis ($)</span>
                    <input type="number" step="1000" name="basis_in_property" value="${state.basis_in_property}"></label>
                <label><span data-i18n="view.s165c3.label.fmv_before">FMV before casualty ($)</span>
                    <input type="number" step="1000" name="fmv_before" value="${state.fmv_before}"></label>
                <label><span data-i18n="view.s165c3.label.fmv_after">FMV after casualty ($)</span>
                    <input type="number" step="1000" name="fmv_after" value="${state.fmv_after}"></label>
                <label><span data-i18n="view.s165c3.label.insurance">Insurance reimbursement ($)</span>
                    <input type="number" step="1000" name="insurance_reimbursement" value="${state.insurance_reimbursement}"></label>
                <label><span data-i18n="view.s165c3.label.disaster">Federally-declared disaster?</span>
                    <input type="checkbox" name="is_federally_declared_disaster" ${state.is_federally_declared_disaster ? 'checked' : ''}></label>
                <label><span data-i18n="view.s165c3.label.business">Business use property?</span>
                    <input type="checkbox" name="is_business_use" ${state.is_business_use ? 'checked' : ''}></label>
                <label><span data-i18n="view.s165c3.label.business_pct">Business use %</span>
                    <input type="number" step="1" name="business_use_pct" value="${state.business_use_pct}"></label>
                <label><span data-i18n="view.s165c3.label.agi">AGI ($)</span>
                    <input type="number" step="1000" name="agi" value="${state.agi}"></label>
                <label><span data-i18n="view.s165c3.label.prior_year">Elect § 165(i) prior year?</span>
                    <input type="checkbox" name="elect_prior_year" ${state.elect_prior_year ? 'checked' : ''}></label>
                <label><span data-i18n="view.s165c3.label.qualified">Qualified disaster (special rules)?</span>
                    <input type="checkbox" name="is_qualified_disaster_2024" ${state.is_qualified_disaster_2024 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s165c3.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s165c3.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s165c3-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s165c3.h2.qualifying_events">Qualifying casualty events</h2>
            <ul class="muted small">
                <li data-i18n="view.s165c3.qe.identifiable">Sudden, unexpected, unusual event</li>
                <li data-i18n="view.s165c3.qe.hurricane">Hurricane, tornado, flood, fire, earthquake (federally declared)</li>
                <li data-i18n="view.s165c3.qe.theft">Theft (post-TCJA: ONLY if federally declared disaster context)</li>
                <li data-i18n="view.s165c3.qe.embezzlement">Embezzlement</li>
                <li data-i18n="view.s165c3.qe.arson">Arson + vandalism</li>
                <li data-i18n="view.s165c3.qe.shipwreck">Shipwreck</li>
                <li data-i18n="view.s165c3.qe.terrorist">Terrorist attacks (federally designated)</li>
                <li data-i18n="view.s165c3.qe.no_progressive">NOT progressive damage (termite, mold, rust)</li>
                <li data-i18n="view.s165c3.qe.no_negligence">NOT taxpayer's intentional / negligent act</li>
                <li data-i18n="view.s165c3.qe.no_normal_wear">NOT normal wear + tear</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s165c3.h2.documentation">Documentation required</h2>
            <ul class="muted small">
                <li data-i18n="view.s165c3.doc.photographs">Photographs / video before + after</li>
                <li data-i18n="view.s165c3.doc.appraisal">Qualified appraisal of FMV decline</li>
                <li data-i18n="view.s165c3.doc.insurance">Insurance claim documents + settlement</li>
                <li data-i18n="view.s165c3.doc.disaster_declaration">FEMA disaster declaration #</li>
                <li data-i18n="view.s165c3.doc.repair_estimates">Repair estimates + receipts</li>
                <li data-i18n="view.s165c3.doc.basis">Original cost / basis documentation</li>
                <li data-i18n="view.s165c3.doc.form_4684">Form 4684 (Casualties + Thefts)</li>
                <li data-i18n="view.s165c3.doc.business">Business: Form 4684 + Form 4797 (gains/losses)</li>
            </ul>
        </div>
    `;
    document.getElementById('s165c3-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.basis_in_property = Number(fd.get('basis_in_property')) || 0;
        state.fmv_before = Number(fd.get('fmv_before')) || 0;
        state.fmv_after = Number(fd.get('fmv_after')) || 0;
        state.insurance_reimbursement = Number(fd.get('insurance_reimbursement')) || 0;
        state.is_federally_declared_disaster = !!fd.get('is_federally_declared_disaster');
        state.is_business_use = !!fd.get('is_business_use');
        state.business_use_pct = Number(fd.get('business_use_pct')) || 0;
        state.agi = Number(fd.get('agi')) || 0;
        state.elect_prior_year = !!fd.get('elect_prior_year');
        state.is_qualified_disaster_2024 = !!fd.get('is_qualified_disaster_2024');
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s165c3-output');
    if (!el) return;
    if (!state.is_federally_declared_disaster && !state.is_business_use) {
        el.innerHTML = `<div class="chart-panel"><p class="muted small neg" data-i18n="view.s165c3.warning.tcja">TCJA 2018-2025: only federally-declared disaster casualty losses allowed for non-business property. Personal loss otherwise NOT deductible.</p></div>`;
        return;
    }
    const fmvDecline = state.fmv_before - state.fmv_after;
    const loss = Math.min(state.basis_in_property, fmvDecline) - state.insurance_reimbursement;
    let netLoss = Math.max(0, loss);
    let allowableLoss;
    if (state.is_business_use) {
        allowableLoss = netLoss * (state.business_use_pct / 100);
    } else if (state.is_qualified_disaster_2024) {
        netLoss = Math.max(0, netLoss - 500);
        allowableLoss = netLoss;
    } else {
        netLoss = Math.max(0, netLoss - PER_CASUALTY_REDUCTION);
        allowableLoss = Math.max(0, netLoss - state.agi * AGI_FLOOR_PCT);
    }
    const taxSavings = allowableLoss * state.marginal_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s165c3.h2.result">Casualty loss calculation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s165c3.card.basis">Property basis</div>
                    <div class="value">$${state.basis_in_property.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s165c3.card.fmv_decline">FMV decline</div>
                    <div class="value">$${fmvDecline.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s165c3.card.lesser_of">Lesser of (basis vs decline)</div>
                    <div class="value">$${Math.min(state.basis_in_property, fmvDecline).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s165c3.card.insurance">Insurance</div>
                    <div class="value">$${state.insurance_reimbursement.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s165c3.card.net_loss">Net loss before floors</div>
                    <div class="value">$${netLoss.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s165c3.card.allowable">Allowable deduction</div>
                    <div class="value">$${allowableLoss.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s165c3.card.tax_savings">Tax savings</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
