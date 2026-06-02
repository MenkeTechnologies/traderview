// IRC § 6664 — Reasonable Cause + Good Faith Defense.
// Statutory defense against § 6662 accuracy-related + § 6663 civil fraud penalties.
// Requires (1) reasonable cause + (2) good faith. Burden on TAXPAYER.
// Factors: effort to assess proper liability, knowledge / experience, reliance on
// professional advice (specific criteria for valid reliance).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    underpayment_amount: 0,
    penalty_imposed: 0,
    relied_on_professional: false,
    professional_was_qualified: false,
    provided_all_information: false,
    professional_lacked_independence: false,
    relied_on_software: false,
    relied_on_irs_publication: false,
    illness_or_disaster: false,
    complex_issue: false,
    timely_compliance_efforts: false,
    documented_position: false,
    marginal_rate: 0.32,
};

export async function renderSection6664(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6664.h1.title">// § 6664 REASONABLE CAUSE DEFENSE</span></h1>
        <p class="muted small" data-i18n="view.s6664.hint.intro">
            Statutory defense against § 6662 accuracy + § 6663 civil fraud penalties. Requires
            <strong>(1) reasonable cause + (2) good faith</strong>. Burden on TAXPAYER. Factors:
            effort to assess proper liability, knowledge / experience, reliance on professional
            advice (with valid-reliance criteria). <strong>Doesn't apply to § 6707A reportable
            transaction penalty</strong> (strict liability).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6664.h2.inputs">Inputs</h2>
            <form id="s6664-form" class="inline-form">
                <label><span data-i18n="view.s6664.label.under">Underpayment ($)</span>
                    <input type="number" step="1000" name="underpayment_amount" value="${state.underpayment_amount}"></label>
                <label><span data-i18n="view.s6664.label.penalty">Penalty imposed ($)</span>
                    <input type="number" step="100" name="penalty_imposed" value="${state.penalty_imposed}"></label>
                <hr style="grid-column:1/-1">
                <label><span data-i18n="view.s6664.label.relied_pro">Relied on professional advice?</span>
                    <input type="checkbox" name="relied_on_professional" ${state.relied_on_professional ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6664.label.pro_qualified">Professional qualified (CPA, JD, EA)?</span>
                    <input type="checkbox" name="professional_was_qualified" ${state.professional_was_qualified ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6664.label.all_info">Provided ALL accurate info?</span>
                    <input type="checkbox" name="provided_all_information" ${state.provided_all_information ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6664.label.no_independence">Professional lacked independence?</span>
                    <input type="checkbox" name="professional_lacked_independence" ${state.professional_lacked_independence ? 'checked' : ''}></label>
                <hr style="grid-column:1/-1">
                <label><span data-i18n="view.s6664.label.software">Relied on tax software?</span>
                    <input type="checkbox" name="relied_on_software" ${state.relied_on_software ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6664.label.publication">Relied on IRS publication?</span>
                    <input type="checkbox" name="relied_on_irs_publication" ${state.relied_on_irs_publication ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6664.label.illness">Illness / disaster / events?</span>
                    <input type="checkbox" name="illness_or_disaster" ${state.illness_or_disaster ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6664.label.complex">Complex / novel issue?</span>
                    <input type="checkbox" name="complex_issue" ${state.complex_issue ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6664.label.timely">Timely compliance efforts?</span>
                    <input type="checkbox" name="timely_compliance_efforts" ${state.timely_compliance_efforts ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6664.label.documented">Documented position research?</span>
                    <input type="checkbox" name="documented_position" ${state.documented_position ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6664.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s6664.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6664-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6664.h2.reliance_factors">Valid professional-reliance criteria (Neonatology Associates v. Comm'r)</h2>
            <ol class="muted small">
                <li data-i18n="view.s6664.rel.qualified">Adviser was a competent professional with sufficient expertise</li>
                <li data-i18n="view.s6664.rel.necessary_info">Taxpayer provided necessary + accurate information</li>
                <li data-i18n="view.s6664.rel.good_faith_reliance">Taxpayer actually relied in good faith on adviser's judgment</li>
                <li data-i18n="view.s6664.rel.independence">Adviser had no inherent conflict of interest / lack of independence</li>
                <li data-i18n="view.s6664.rel.discussion">Substantive discussion of facts + applicable law occurred</li>
                <li data-i18n="view.s6664.rel.specific_advice">Advice specific to taxpayer's facts (not generic)</li>
                <li data-i18n="view.s6664.rel.boilerplate">Boilerplate opinions / kit-style memoranda generally don't qualify</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6664.h2.boyle">United States v. Boyle (1985) — limits</h2>
            <p class="muted small" data-i18n="view.s6664.boyle.body">
                Reliance on agent for FILING DEADLINE = NOT reasonable cause (Boyle). However,
                reliance on advice for SUBSTANTIVE TAX positions CAN be reasonable cause if
                Neonatology factors met. Distinction: "ministerial / clerical tasks" (taxpayer
                responsible) vs "substantive judgment" (reliance OK). 7th Circuit in West v.
                Comm'r reaffirmed but with nuanced application.
            </p>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6664.h2.factors_against">Factors against reasonable cause</h2>
            <ul class="muted small">
                <li data-i18n="view.s6664.fac.large_amount">Large dollar amount of underpayment relative to total income</li>
                <li data-i18n="view.s6664.fac.repeat_offender">Repeat offender / history of penalties</li>
                <li data-i18n="view.s6664.fac.adverse_circumstances">Position taken in adverse-circumstance avoidance pattern</li>
                <li data-i18n="view.s6664.fac.expert_taxpayer">Taxpayer is professional / expert with knowledge</li>
                <li data-i18n="view.s6664.fac.willful_blindness">Willful blindness — reckless disregard for facts</li>
                <li data-i18n="view.s6664.fac.not_documented">Failure to document position research</li>
                <li data-i18n="view.s6664.fac.shelter">Tax shelter / aggressive position</li>
                <li data-i18n="view.s6664.fac.late_response">Late response to IRS inquiries / non-cooperation</li>
            </ul>
        </div>
    `;
    document.getElementById('s6664-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.underpayment_amount = Number(fd.get('underpayment_amount')) || 0;
        state.penalty_imposed = Number(fd.get('penalty_imposed')) || 0;
        state.relied_on_professional = !!fd.get('relied_on_professional');
        state.professional_was_qualified = !!fd.get('professional_was_qualified');
        state.provided_all_information = !!fd.get('provided_all_information');
        state.professional_lacked_independence = !!fd.get('professional_lacked_independence');
        state.relied_on_software = !!fd.get('relied_on_software');
        state.relied_on_irs_publication = !!fd.get('relied_on_irs_publication');
        state.illness_or_disaster = !!fd.get('illness_or_disaster');
        state.complex_issue = !!fd.get('complex_issue');
        state.timely_compliance_efforts = !!fd.get('timely_compliance_efforts');
        state.documented_position = !!fd.get('documented_position');
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6664-output');
    if (!el) return;
    let score = 0;
    if (state.relied_on_professional && state.professional_was_qualified
        && state.provided_all_information && !state.professional_lacked_independence) {
        score += 5;
    }
    if (state.illness_or_disaster) score += 2;
    if (state.complex_issue) score += 2;
    if (state.timely_compliance_efforts) score += 1;
    if (state.documented_position) score += 2;
    if (state.relied_on_irs_publication) score += 1;
    if (state.relied_on_software) score += 0;
    let likelihood, likelihoodCls;
    if (score >= 7) { likelihood = 'view.s6664.likelihood.strong'; likelihoodCls = 'pos'; }
    else if (score >= 4) { likelihood = 'view.s6664.likelihood.moderate'; likelihoodCls = 'pos'; }
    else if (score >= 2) { likelihood = 'view.s6664.likelihood.weak'; likelihoodCls = 'neg'; }
    else { likelihood = 'view.s6664.likelihood.unlikely'; likelihoodCls = 'neg'; }
    const expectedAbatement = score >= 4 ? state.penalty_imposed : (score >= 2 ? state.penalty_imposed * 0.5 : 0);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6664.h2.result">Defense analysis</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s6664.card.score">Reasonable cause score</div>
                    <div class="value">${score}</div>
                </div>
                <div class="card ${likelihoodCls}">
                    <div class="label" data-i18n="view.s6664.card.likelihood">Likelihood of relief</div>
                    <div class="value">${esc(t(likelihood))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s6664.card.abatement">Expected penalty abatement</div>
                    <div class="value">$${expectedAbatement.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
