// IRC § 901(j) — Sanctioned Country FTC Denial.
// No FTC allowed for taxes paid to countries on State Department sanctioned list.
// Currently sanctioned (2024): Iran, North Korea, Syria, Cuba, Venezuela (partial), Russia post-2022.
// FTC denial applies to ALL taxes paid (income, withholding, branch) to these countries.
// Conduit jurisdictions: routing income through unsanctioned countries doesn't restore FTC.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    country_name: '',
    is_sanctioned: false,
    foreign_tax_paid: 0,
    foreign_source_income: 0,
    direct_payment_to_country: true,
    routed_through_third_country: false,
    third_country_unsanctioned: false,
    direct_branch_in_country: false,
    cfc_in_country: false,
    treaty_provisions_apply: false,
    is_partial_sanction: false,
    license_obtained_ofac: false,
    pre_sanction_acquired: false,
    foreign_tax_year: 2024,
    eci_in_sanctioned_country: false,
};

export async function renderSection901J(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s901j.h1.title">// § 901(j) SANCTIONED FTC DENIAL</span></h1>
        <p class="muted small" data-i18n="view.s901j.hint.intro">
            <strong>NO FTC ALLOWED</strong> for taxes paid to countries on State Department sanctioned list.
            <strong>Currently sanctioned (2024):</strong> Iran, North Korea, Syria, Cuba, Venezuela (partial),
            Russia (post-2022, but treaty suspended). FTC denial applies to ALL taxes (income, withholding,
            branch) to these countries. <strong>Conduit jurisdictions:</strong> routing through unsanctioned
            countries DOES NOT restore FTC if income clearly tied to sanctioned source. <strong>OFAC
            licenses:</strong> obtained licenses for certain activities don't restore FTC.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s901j.h2.inputs">Inputs</h2>
            <form id="s901j-form" class="inline-form">
                <label><span data-i18n="view.s901j.label.country">Country name</span>
                    <input type="text" name="country_name" value="${esc(state.country_name)}"></label>
                <label><span data-i18n="view.s901j.label.sanctioned">Country sanctioned?</span>
                    <input type="checkbox" name="is_sanctioned" ${state.is_sanctioned ? 'checked' : ''}></label>
                <label><span data-i18n="view.s901j.label.tax">Foreign tax paid ($)</span>
                    <input type="number" step="1000" name="foreign_tax_paid" value="${state.foreign_tax_paid}"></label>
                <label><span data-i18n="view.s901j.label.income">Foreign source income ($)</span>
                    <input type="number" step="10000" name="foreign_source_income" value="${state.foreign_source_income}"></label>
                <label><span data-i18n="view.s901j.label.direct">Direct payment to country?</span>
                    <input type="checkbox" name="direct_payment_to_country" ${state.direct_payment_to_country ? 'checked' : ''}></label>
                <label><span data-i18n="view.s901j.label.routed">Routed through 3rd country?</span>
                    <input type="checkbox" name="routed_through_third_country" ${state.routed_through_third_country ? 'checked' : ''}></label>
                <label><span data-i18n="view.s901j.label.unsanctioned">3rd country unsanctioned?</span>
                    <input type="checkbox" name="third_country_unsanctioned" ${state.third_country_unsanctioned ? 'checked' : ''}></label>
                <label><span data-i18n="view.s901j.label.branch">Direct branch in country?</span>
                    <input type="checkbox" name="direct_branch_in_country" ${state.direct_branch_in_country ? 'checked' : ''}></label>
                <label><span data-i18n="view.s901j.label.cfc">CFC in country?</span>
                    <input type="checkbox" name="cfc_in_country" ${state.cfc_in_country ? 'checked' : ''}></label>
                <label><span data-i18n="view.s901j.label.treaty">Treaty provisions apply?</span>
                    <input type="checkbox" name="treaty_provisions_apply" ${state.treaty_provisions_apply ? 'checked' : ''}></label>
                <label><span data-i18n="view.s901j.label.partial">Partial sanction?</span>
                    <input type="checkbox" name="is_partial_sanction" ${state.is_partial_sanction ? 'checked' : ''}></label>
                <label><span data-i18n="view.s901j.label.ofac">OFAC license obtained?</span>
                    <input type="checkbox" name="license_obtained_ofac" ${state.license_obtained_ofac ? 'checked' : ''}></label>
                <label><span data-i18n="view.s901j.label.pre">Pre-sanction acquired?</span>
                    <input type="checkbox" name="pre_sanction_acquired" ${state.pre_sanction_acquired ? 'checked' : ''}></label>
                <label><span data-i18n="view.s901j.label.year">Foreign tax year</span>
                    <input type="number" step="1" name="foreign_tax_year" value="${state.foreign_tax_year}"></label>
                <label><span data-i18n="view.s901j.label.eci">ECI in sanctioned country?</span>
                    <input type="checkbox" name="eci_in_sanctioned_country" ${state.eci_in_sanctioned_country ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s901j.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s901j-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s901j.h2.current_list">Currently sanctioned countries (2024)</h2>
            <ul class="muted small">
                <li data-i18n="view.s901j.list.iran">Iran (Comprehensive: § 901(j)(2)(A))</li>
                <li data-i18n="view.s901j.list.north_korea">North Korea / DPRK (Comprehensive)</li>
                <li data-i18n="view.s901j.list.syria">Syria (Designated 1979)</li>
                <li data-i18n="view.s901j.list.cuba">Cuba (Designated 1962 — Helms-Burton Act + CACR)</li>
                <li data-i18n="view.s901j.list.venezuela">Venezuela (Partial — Maduro government 2019)</li>
                <li data-i18n="view.s901j.list.russia">Russia (Post-2022 invasion — treaty suspended)</li>
                <li data-i18n="view.s901j.list.belarus">Belarus (Connected sanctions)</li>
                <li data-i18n="view.s901j.list.crimea">Crimea + Donetsk + Luhansk (Ukraine occupied)</li>
                <li data-i18n="view.s901j.list.update_status">List updates via Executive Orders + State Department</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s901j.h2.scope">§ 901(j) scope + applications</h2>
            <ul class="muted small">
                <li data-i18n="view.s901j.scope.all_taxes">ALL taxes denied: income, withholding, VAT, customs duties, branch profits</li>
                <li data-i18n="view.s901j.scope.cfc_holding">CFCs in sanctioned country: subF + GILTI inclusions still apply</li>
                <li data-i18n="view.s901j.scope.no_credit">No FTC + no deduction (under § 901(j)) — pure tax loss</li>
                <li data-i18n="view.s901j.scope.section_960">§ 960 deemed-paid credit: also denied for sanctioned country taxes</li>
                <li data-i18n="view.s901j.scope.no_basket">No special basket — denial is absolute</li>
                <li data-i18n="view.s901j.scope.subF_double_tax">Combined effect: subF inclusion + foreign tax + no FTC = double taxation</li>
                <li data-i18n="view.s901j.scope.deduction_alternative">May claim deduction (subject to limit) instead — limited utility</li>
                <li data-i18n="view.s901j.scope.permanent">Designation permanent until State Department removes from list</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s901j.h2.workarounds">Workarounds + planning (limited)</h2>
            <ul class="muted small">
                <li data-i18n="view.s901j.work.exit">Exit sanctioned country operations: divest before designation hardens</li>
                <li data-i18n="view.s901j.work.deduct_loss">Take deduction (not credit) — subject to deduction limits + at-risk rules</li>
                <li data-i18n="view.s901j.work.no_routing">Routing through 3rd country DOES NOT restore FTC for sanctioned-source income</li>
                <li data-i18n="view.s901j.work.cfc_basis">CFC basis: still recognize basis decrease per § 961(b) for sanctioned-tax payments</li>
                <li data-i18n="view.s901j.work.gilti_avoidance">GILTI: high-tax election § 954(b)(4) DOES NOT eliminate sanctioned country tax</li>
                <li data-i18n="view.s901j.work.exclusion">No exception for OFAC-licensed activities (e.g., medical / humanitarian)</li>
                <li data-i18n="view.s901j.work.exit_strategy">Pre-divestiture: claim historic FTC for years before designation</li>
                <li data-i18n="view.s901j.work.dual_resident">Dual treaty residency may shift source to non-sanctioned country (rare)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s901j.h2.removal">Country removal from list</h2>
            <ul class="muted small">
                <li data-i18n="view.s901j.rem.process">Removal: Executive Order rescinding sanction → automatic § 901(j) removal</li>
                <li data-i18n="view.s901j.rem.historical">Historical removals: Libya (2006), Sudan (2017), various smaller adjustments</li>
                <li data-i18n="view.s901j.rem.transitional">Transitional rule: tax paid AFTER removal eligible for FTC</li>
                <li data-i18n="view.s901j.rem.prior_year_recovery">Cannot recover FTC for prior years (no retroactive)</li>
                <li data-i18n="view.s901j.rem.notice">IRS Notice often confirms FTC restored</li>
                <li data-i18n="view.s901j.rem.cuba_partial">Cuba: partial sanctions reductions in 2014-2016 + reversal 2017</li>
                <li data-i18n="view.s901j.rem.iran_jcpoa">Iran JCPOA period (2015-2017): partial relief; reverted with US withdrawal</li>
                <li data-i18n="view.s901j.rem.uncertainty">Political volatility: sanctions can change quickly with administration changes</li>
            </ul>
        </div>
    `;
    document.getElementById('s901j-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.country_name = fd.get('country_name');
        state.is_sanctioned = !!fd.get('is_sanctioned');
        state.foreign_tax_paid = Number(fd.get('foreign_tax_paid')) || 0;
        state.foreign_source_income = Number(fd.get('foreign_source_income')) || 0;
        state.direct_payment_to_country = !!fd.get('direct_payment_to_country');
        state.routed_through_third_country = !!fd.get('routed_through_third_country');
        state.third_country_unsanctioned = !!fd.get('third_country_unsanctioned');
        state.direct_branch_in_country = !!fd.get('direct_branch_in_country');
        state.cfc_in_country = !!fd.get('cfc_in_country');
        state.treaty_provisions_apply = !!fd.get('treaty_provisions_apply');
        state.is_partial_sanction = !!fd.get('is_partial_sanction');
        state.license_obtained_ofac = !!fd.get('license_obtained_ofac');
        state.pre_sanction_acquired = !!fd.get('pre_sanction_acquired');
        state.foreign_tax_year = Number(fd.get('foreign_tax_year')) || 0;
        state.eci_in_sanctioned_country = !!fd.get('eci_in_sanctioned_country');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s901j-output');
    if (!el) return;
    const ftc_denied = state.is_sanctioned && state.foreign_tax_paid > 0;
    const denial_amount = ftc_denied ? state.foreign_tax_paid : 0;
    const lost_ftc_value = denial_amount;
    const deduction_alternative = denial_amount * 0.21;
    const double_tax_amount = state.foreign_tax_paid + (state.foreign_source_income * 0.21);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s901j.h2.result">§ 901(j) determination</h2>
            <div class="cards">
                <div class="card ${ftc_denied ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s901j.card.denied">FTC denied?</div>
                    <div class="value">${ftc_denied ? esc(t('view.s901j.status.yes')) : esc(t('view.s901j.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s901j.card.tax_paid">Foreign tax paid</div>
                    <div class="value">$${state.foreign_tax_paid.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s901j.card.denial">FTC denial amount</div>
                    <div class="value">$${denial_amount.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s901j.card.lost_value">Lost FTC value (full)</div>
                    <div class="value">$${lost_ftc_value.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s901j.card.deduction">Deduction alternative (21%)</div>
                    <div class="value">$${deduction_alternative.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s901j.card.double">Double tax exposure</div>
                    <div class="value">$${double_tax_amount.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${ftc_denied ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s901j.denied_note">
                    FTC DENIED under § 901(j) for sanctioned country. Foreign tax becomes pure cost — only
                    21% recovery via deduction. CFC + subF / GILTI inclusion still applies. Routing through
                    unsanctioned 3rd country DOES NOT restore FTC. Consider divestiture before sanctions
                    harden; lobby for treaty protection (unlikely for hostile states).
                </p>
            ` : ''}
        </div>
    `;
}
