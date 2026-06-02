// IRC § 269 — Acquisition Made to Evade or Avoid Income Tax.
// IRS can DISALLOW DEDUCTIONS, CREDITS, ALLOWANCES from acquired corporation if PRIMARY purpose = tax avoidance.
// § 269(a)(1): person acquires control (50%) of corp w/ primary purpose tax-avoidance.
// § 269(a)(2): corp acquires property w/ carryover basis from another corp + primary purpose tax-avoidance.
// Overlap with § 382 NOL limit, § 384 (consolidated SRLY), § 7701(o) economic substance.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    type_acquisition: 'control_of_corp',
    target_nol_acquired: 0,
    target_credits_acquired: 0,
    target_capital_loss: 0,
    primary_purpose_tax_avoidance: false,
    business_purpose: true,
    s382_limit_already: false,
    consolidated_group_acquisition: false,
    pre_acquisition_business_continuity: true,
    acquirer_has_profits: 0,
    acquirer_pre_acq_tax_liability: 0,
    judicial_doctrines_apply: false,
};

export async function renderSection269(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s269.h1.title">// § 269 ANTI-ABUSE ACQUISITION</span></h1>
        <p class="muted small" data-i18n="view.s269.hint.intro">
            <strong>IRS DISALLOWS</strong> deductions, credits, allowances from acquired corp if PRIMARY purpose
            = <strong>tax avoidance</strong>. <strong>§ 269(a)(1):</strong> person acquires CONTROL (50%) of corp.
            <strong>§ 269(a)(2):</strong> corp acquires PROPERTY w/ carryover basis. <strong>Burden:</strong>
            IRS (initially), then taxpayer to rebut. <strong>Overlap:</strong> § 382 NOL limit, § 384 SRLY,
            § 7701(o) economic substance. § 269 PRE-EMPTS even when § 382 also limits — covers SRLY-style
            and entity-level transactions broader than § 382 ownership change.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s269.h2.inputs">Inputs</h2>
            <form id="s269-form" class="inline-form">
                <label><span data-i18n="view.s269.label.type">Acquisition type</span>
                    <select name="type_acquisition">
                        <option value="control_of_corp" ${state.type_acquisition === 'control_of_corp' ? 'selected' : ''}>§ 269(a)(1) Control of corp (50%)</option>
                        <option value="property" ${state.type_acquisition === 'property' ? 'selected' : ''}>§ 269(a)(2) Property w/ carryover basis</option>
                        <option value="consolidated_b" ${state.type_acquisition === 'consolidated_b' ? 'selected' : ''}>§ 269(b) Consolidated group filing</option>
                    </select>
                </label>
                <label><span data-i18n="view.s269.label.nol">Target NOL acquired ($)</span>
                    <input type="number" step="10000" name="target_nol_acquired" value="${state.target_nol_acquired}"></label>
                <label><span data-i18n="view.s269.label.credits">Target credits acquired ($)</span>
                    <input type="number" step="1000" name="target_credits_acquired" value="${state.target_credits_acquired}"></label>
                <label><span data-i18n="view.s269.label.cap_loss">Target capital loss ($)</span>
                    <input type="number" step="10000" name="target_capital_loss" value="${state.target_capital_loss}"></label>
                <label><span data-i18n="view.s269.label.purpose">Primary purpose tax avoidance?</span>
                    <input type="checkbox" name="primary_purpose_tax_avoidance" ${state.primary_purpose_tax_avoidance ? 'checked' : ''}></label>
                <label><span data-i18n="view.s269.label.business">Business purpose exists?</span>
                    <input type="checkbox" name="business_purpose" ${state.business_purpose ? 'checked' : ''}></label>
                <label><span data-i18n="view.s269.label.s382">§ 382 limit already applies?</span>
                    <input type="checkbox" name="s382_limit_already" ${state.s382_limit_already ? 'checked' : ''}></label>
                <label><span data-i18n="view.s269.label.cons">Consolidated group acquisition?</span>
                    <input type="checkbox" name="consolidated_group_acquisition" ${state.consolidated_group_acquisition ? 'checked' : ''}></label>
                <label><span data-i18n="view.s269.label.continuity">Pre-acq business continuity?</span>
                    <input type="checkbox" name="pre_acquisition_business_continuity" ${state.pre_acquisition_business_continuity ? 'checked' : ''}></label>
                <label><span data-i18n="view.s269.label.profits">Acquirer current-yr profits ($)</span>
                    <input type="number" step="10000" name="acquirer_has_profits" value="${state.acquirer_has_profits}"></label>
                <label><span data-i18n="view.s269.label.pre_tax">Acquirer pre-acq tax liability ($)</span>
                    <input type="number" step="10000" name="acquirer_pre_acq_tax_liability" value="${state.acquirer_pre_acq_tax_liability}"></label>
                <label><span data-i18n="view.s269.label.judicial">Other judicial doctrines apply?</span>
                    <input type="checkbox" name="judicial_doctrines_apply" ${state.judicial_doctrines_apply ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s269.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s269-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s269.h2.tests">"Primary purpose" determination factors</h2>
            <ul class="muted small">
                <li data-i18n="view.s269.test.tax_motive">Tax motive must be MORE THAN 50% of purpose (primary)</li>
                <li data-i18n="view.s269.test.business_purpose">Business purpose can SAVE acquisition even if tax savings significant</li>
                <li data-i18n="view.s269.test.shell_corps">Shell corps + dormant targets = high § 269 risk</li>
                <li data-i18n="view.s269.test.no_continuity">Acquisition + immediate business change = § 269 risk</li>
                <li data-i18n="view.s269.test.large_nol">Large NOL with no operations = clear § 269 territory</li>
                <li data-i18n="view.s269.test.bargain_purchase">Bargain purchase below FMV due to deferred tax assets = indicia of tax motive</li>
                <li data-i18n="view.s269.test.shareholders_continuity">Old shareholders' continuity reduces § 269 risk</li>
                <li data-i18n="view.s269.test.factual">Fact-intensive: written documents, internal memoranda, board minutes scrutinized</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s269.h2.coordination">Coordination with § 382 + § 384 + § 7701(o)</h2>
            <ul class="muted small">
                <li data-i18n="view.s269.coord.s382">§ 382: mechanical limit on NOL usage post-ownership change (annual ATE × LT rate)</li>
                <li data-i18n="view.s269.coord.s384">§ 384: SRLY rule limits pre-acquisition losses against post-acquisition built-in gains</li>
                <li data-i18n="view.s269.coord.s269_overlap">§ 269 PRE-EMPTS even when § 382 also applies — broader anti-abuse</li>
                <li data-i18n="view.s269.coord.s7701o">§ 7701(o) Economic Substance: codified — meaningful change required + non-tax purpose</li>
                <li data-i18n="view.s269.coord.s482">§ 482 transfer pricing: parallel anti-abuse for related-party transactions</li>
                <li data-i18n="view.s269.coord.s367">§ 367: outbound transfers anti-abuse for foreign acquisitions</li>
                <li data-i18n="view.s269.coord.s7704">§ 7704 PTP rules: prevent corporate shell using partnership structure</li>
                <li data-i18n="view.s269.coord.s356">§ 356 (boot in reorg) coordinates with § 269 in tax-free reorgs</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s269.h2.consequences">Consequences if § 269 applies</h2>
            <ul class="muted small">
                <li data-i18n="view.s269.cons.disallow_nol">DISALLOW NOL carryover (in whole or part — discretion of Commissioner)</li>
                <li data-i18n="view.s269.cons.disallow_credits">DISALLOW unused tax credits</li>
                <li data-i18n="view.s269.cons.disallow_deductions">DISALLOW depreciation, amortization, other deductions</li>
                <li data-i18n="view.s269.cons.disallow_capital_loss">DISALLOW capital loss carryover</li>
                <li data-i18n="view.s269.cons.partial">Partial disallowance possible — depends on facts</li>
                <li data-i18n="view.s269.cons.no_refund">Once disallowed, NO refund / re-allowance</li>
                <li data-i18n="view.s269.cons.discretion">Commissioner has broad discretion to disallow some but not all</li>
                <li data-i18n="view.s269.cons.parallel">Parallel: § 269A (personal service corp), § 269B (stapled entities)</li>
            </ul>
        </div>
    `;
    document.getElementById('s269-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.type_acquisition = fd.get('type_acquisition');
        state.target_nol_acquired = Number(fd.get('target_nol_acquired')) || 0;
        state.target_credits_acquired = Number(fd.get('target_credits_acquired')) || 0;
        state.target_capital_loss = Number(fd.get('target_capital_loss')) || 0;
        state.primary_purpose_tax_avoidance = !!fd.get('primary_purpose_tax_avoidance');
        state.business_purpose = !!fd.get('business_purpose');
        state.s382_limit_already = !!fd.get('s382_limit_already');
        state.consolidated_group_acquisition = !!fd.get('consolidated_group_acquisition');
        state.pre_acquisition_business_continuity = !!fd.get('pre_acquisition_business_continuity');
        state.acquirer_has_profits = Number(fd.get('acquirer_has_profits')) || 0;
        state.acquirer_pre_acq_tax_liability = Number(fd.get('acquirer_pre_acq_tax_liability')) || 0;
        state.judicial_doctrines_apply = !!fd.get('judicial_doctrines_apply');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s269-output');
    if (!el) return;
    const sec269Applies = state.primary_purpose_tax_avoidance && !state.business_purpose;
    const totalAttributes = state.target_nol_acquired + state.target_credits_acquired + state.target_capital_loss;
    const disallowedAttributes = sec269Applies ? totalAttributes : 0;
    const taxImpactIfAllowed = totalAttributes * 0.21;
    const taxImpactIfDisallowed = disallowedAttributes * 0.21;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s269.h2.result">§ 269 outcome</h2>
            <div class="cards">
                <div class="card ${sec269Applies ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s269.card.applies">§ 269 applies?</div>
                    <div class="value">${sec269Applies ? esc(t('view.s269.status.yes')) : esc(t('view.s269.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s269.card.total_attr">Total attributes acquired</div>
                    <div class="value">$${totalAttributes.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s269.card.disallowed">Disallowed by § 269</div>
                    <div class="value">$${disallowedAttributes.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s269.card.tax_if_allowed">Tax savings if allowed (21%)</div>
                    <div class="value">$${taxImpactIfAllowed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s269.card.tax_if_disallowed">Tax lost if disallowed (21%)</div>
                    <div class="value">$${taxImpactIfDisallowed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${sec269Applies ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s269.applies_note">
                    § 269 likely applies: PRIMARY purpose tax avoidance + no business purpose. IRS may
                    disallow NOL, credits, deductions in whole or part. Documents pre-acquisition: rebut
                    with business purpose evidence (synergies, market expansion, talent acquisition).
                    Pre-clearance: consider PLR before completing transaction.
                </p>
            ` : ''}
        </div>
    `;
}
