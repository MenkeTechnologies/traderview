// AMT § 55 Calculator.
// Parallel tax system: regular tax + preferences/adjustments = AMTI.
// AMTI - AMT exemption × 26% (up to $232,600 / $116,300 MFS, 2024) or 28%.
// Pay greater of regular tax or AMT. ISO exercise = single biggest AMT trigger
// (bargain element is AMT preference but not regular income at exercise).

import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const AMT_EXEMPTION_2024 = {
    single: 85_700,
    mfj: 133_300,
    mfs: 66_650,
};
const AMT_EXEMPT_PHASEOUT_2024 = {
    single: 609_350,
    mfj: 1_218_700,
    mfs: 609_350,
};
const AMT_26_28_BREAKPOINT_2024 = 232_600;
const AMT_26_28_BREAKPOINT_MFS = 116_300;

let state = {
    filing: 'single',
    regular_taxable_income: 250_000,
    regular_tax: 60_000,
    iso_bargain_element: 100_000,
    state_local_tax_addback: 25_000,
    private_activity_bond_interest: 0,
    other_preferences: 0,
};

export async function renderAmtCalc(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.amt.h1.title">// AMT § 55 CALCULATOR</span></h1>
        <p class="muted small" data-i18n="view.amt.hint.intro">
            Parallel tax system: regular taxable income + preferences/adjustments = AMTI.
            (AMTI − AMT exemption) × 26% (28% above $232,600) = tentative AMT.
            Pay greater of regular or AMT. <strong>ISO exercise = #1 AMT trigger</strong>.
            AMT paid generates MTC (Minimum Tax Credit) — carries forward against future
            regular tax owed.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.amt.h2.inputs">Inputs</h2>
            <form id="amt-form" class="inline-form">
                <label><span data-i18n="view.amt.label.filing">Filing</span>
                    <select name="filing">
                        <option value="single" ${state.filing === 'single' ? 'selected' : ''}>Single</option>
                        <option value="mfj" ${state.filing === 'mfj' ? 'selected' : ''}>MFJ</option>
                        <option value="mfs" ${state.filing === 'mfs' ? 'selected' : ''}>MFS</option>
                    </select>
                </label>
                <label><span data-i18n="view.amt.label.regular_taxable_income">Regular taxable income ($)</span>
                    <input type="number" step="1000" name="regular_taxable_income" value="${state.regular_taxable_income}"></label>
                <label><span data-i18n="view.amt.label.regular_tax">Regular tax ($)</span>
                    <input type="number" step="100" name="regular_tax" value="${state.regular_tax}"></label>
                <label><span data-i18n="view.amt.label.iso_bargain">ISO bargain element ($)</span>
                    <input type="number" step="1000" name="iso_bargain_element" value="${state.iso_bargain_element}"></label>
                <label><span data-i18n="view.amt.label.salt_addback">SALT addback ($)</span>
                    <input type="number" step="500" name="state_local_tax_addback" value="${state.state_local_tax_addback}"></label>
                <label><span data-i18n="view.amt.label.private_bond">Private-activity bond interest ($)</span>
                    <input type="number" step="100" name="private_activity_bond_interest" value="${state.private_activity_bond_interest}"></label>
                <label><span data-i18n="view.amt.label.other_preferences">Other AMT preferences ($)</span>
                    <input type="number" step="100" name="other_preferences" value="${state.other_preferences}"></label>
                <button class="primary" type="submit" data-i18n="view.amt.btn.compute">Compute</button>
            </form>
        </div>
        <div id="amt-output"></div>
    `;
    document.getElementById('amt-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.filing = fd.get('filing');
        state.regular_taxable_income = Number(fd.get('regular_taxable_income')) || 0;
        state.regular_tax = Number(fd.get('regular_tax')) || 0;
        state.iso_bargain_element = Number(fd.get('iso_bargain_element')) || 0;
        state.state_local_tax_addback = Number(fd.get('state_local_tax_addback')) || 0;
        state.private_activity_bond_interest = Number(fd.get('private_activity_bond_interest')) || 0;
        state.other_preferences = Number(fd.get('other_preferences')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('amt-output');
    if (!el) return;
    const exempt = AMT_EXEMPTION_2024[state.filing];
    const phaseoutStart = AMT_EXEMPT_PHASEOUT_2024[state.filing];
    const breakpoint = state.filing === 'mfs' ? AMT_26_28_BREAKPOINT_MFS : AMT_26_28_BREAKPOINT_2024;

    // Compute AMTI
    const preferences = state.iso_bargain_element + state.state_local_tax_addback
        + state.private_activity_bond_interest + state.other_preferences;
    const amti = state.regular_taxable_income + preferences;

    // Exemption phase-out: 25¢/$1 over phaseoutStart, fully phased out at $X
    const phaseoutExcess = Math.max(0, amti - phaseoutStart);
    const exemptionReduction = phaseoutExcess * 0.25;
    const effectiveExemption = Math.max(0, exempt - exemptionReduction);
    const amtiAfterExemption = Math.max(0, amti - effectiveExemption);

    // 26%/28% brackets
    const slice26 = Math.min(amtiAfterExemption, breakpoint);
    const slice28 = Math.max(0, amtiAfterExemption - breakpoint);
    const tentativeAmt = slice26 * 0.26 + slice28 * 0.28;

    // AMT owed = max(0, tentative AMT - regular tax)
    const amtOwed = Math.max(0, tentativeAmt - state.regular_tax);
    const totalTax = state.regular_tax + amtOwed;

    el.innerHTML = `
        <div class="chart-panel ${amtOwed > 0 ? 'neg' : 'pos'}">
            <h2 data-i18n="view.amt.h2.result">AMT calculation</h2>
            <div class="cards">
                <div class="card neg">
                    <div class="label" data-i18n="view.amt.card.amt_owed">AMT owed</div>
                    <div class="value">$${amtOwed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.amt.card.amti">AMTI</div>
                    <div class="value">$${amti.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.amt.card.exemption">Effective exemption</div>
                    <div class="value">$${effectiveExemption.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.amt.card.tentative_amt">Tentative AMT</div>
                    <div class="value">$${tentativeAmt.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.amt.card.regular_tax">Regular tax</div>
                    <div class="value">$${state.regular_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.amt.card.total_tax">Total tax (max)</div>
                    <div class="value">$${totalTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.amt.h2.breakdown">Breakdown</h2>
            <table class="trades"><tbody>
                <tr><td data-i18n="view.amt.row.regular_ti">Regular taxable income</td>
                    <td>$${state.regular_taxable_income.toLocaleString()}</td></tr>
                <tr><td data-i18n="view.amt.row.iso_pref">+ ISO bargain element</td>
                    <td>$${state.iso_bargain_element.toLocaleString()}</td></tr>
                <tr><td data-i18n="view.amt.row.salt_pref">+ SALT addback</td>
                    <td>$${state.state_local_tax_addback.toLocaleString()}</td></tr>
                <tr><td data-i18n="view.amt.row.bond_pref">+ Private-activity bond interest</td>
                    <td>$${state.private_activity_bond_interest.toLocaleString()}</td></tr>
                <tr><td data-i18n="view.amt.row.other_pref">+ Other preferences</td>
                    <td>$${state.other_preferences.toLocaleString()}</td></tr>
                <tr><td><strong data-i18n="view.amt.row.amti">= AMTI</strong></td>
                    <td><strong>$${amti.toLocaleString()}</strong></td></tr>
                <tr><td data-i18n="view.amt.row.base_exemption">Base exemption</td>
                    <td>$${exempt.toLocaleString()}</td></tr>
                <tr><td data-i18n="view.amt.row.phaseout">− Phase-out (25¢/$1 over $${phaseoutStart.toLocaleString()})</td>
                    <td>$${exemptionReduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td></tr>
                <tr><td><strong data-i18n="view.amt.row.eff_exempt">= Effective exemption</strong></td>
                    <td><strong>$${effectiveExemption.toLocaleString(undefined, { maximumFractionDigits: 0 })}</strong></td></tr>
                <tr><td data-i18n="view.amt.row.amti_post">AMTI after exemption</td>
                    <td>$${amtiAfterExemption.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td></tr>
                <tr><td data-i18n="view.amt.row.bracket_26">× 26% (up to $${breakpoint.toLocaleString()})</td>
                    <td>$${(slice26 * 0.26).toLocaleString(undefined, { maximumFractionDigits: 0 })}</td></tr>
                <tr><td data-i18n="view.amt.row.bracket_28">× 28% (above)</td>
                    <td>$${(slice28 * 0.28).toLocaleString(undefined, { maximumFractionDigits: 0 })}</td></tr>
                <tr><td><strong data-i18n="view.amt.row.tentative">= Tentative AMT</strong></td>
                    <td><strong>$${tentativeAmt.toLocaleString(undefined, { maximumFractionDigits: 0 })}</strong></td></tr>
                <tr><td data-i18n="view.amt.row.minus_regular">− Regular tax</td>
                    <td>$${state.regular_tax.toLocaleString()}</td></tr>
                <tr><td><strong data-i18n="view.amt.row.amt_owed_row">= AMT owed</strong></td>
                    <td><strong class="neg">$${amtOwed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</strong></td></tr>
            </tbody></table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.amt.h2.notes">Notes</h2>
            <ul class="muted small">
                <li data-i18n="view.amt.note.mtc">AMT paid generates a Minimum Tax Credit — recoverable against future regular tax (Form 8801)</li>
                <li data-i18n="view.amt.note.iso_basis">ISO AMT basis ≠ regular basis after exercise. Sale calc differs by FMV at exercise.</li>
                <li data-i18n="view.amt.note.salt_capped">SALT addback only matters if itemizing > standard deduction (post-TCJA most don't)</li>
                <li data-i18n="view.amt.note.amt_repeal">TCJA raised exemption — most middle-class taxpayers no longer hit AMT. ISO exercises + private-activity bonds still trigger it.</li>
                <li data-i18n="view.amt.note.same_day_sale">Same-day ISO exercise & sell = NSO treatment, no AMT preference</li>
            </ul>
        </div>
    `;
}
