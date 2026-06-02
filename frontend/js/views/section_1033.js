// IRC § 1033 — Involuntary Conversion (Casualty / Theft / Condemnation).
// Defer gain if proceeds REINVESTED in qualified replacement property within 2-yr window (3-yr real prop).
// Real property condemnation: 3-year replacement period; § 1033(g) special rule.
// Federally declared disaster: 4-year window. Live stock from drought: 4-year.
// Basis of replacement = cost minus deferred gain. Holding period tacks.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    conversion_proceeds: 0,
    property_basis: 0,
    replacement_cost: 0,
    conversion_type: 'casualty',
    is_real_property: false,
    is_federally_declared_disaster: false,
    replacement_complete: true,
    same_or_similar: true,
    years_to_replace: 2,
    is_principal_residence: false,
    is_business_property: false,
    fmv_unrestricted: 0,
};

export async function renderSection1033(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1033.h1.title">// § 1033 INVOLUNTARY CONVERSION</span></h1>
        <p class="muted small" data-i18n="view.s1033.hint.intro">
            Defer gain if proceeds REINVESTED in <strong>qualified replacement property</strong> within
            <strong>2-yr window</strong> (3-yr real property condemnation, 4-yr federally declared disaster).
            <strong>"Same or similar use"</strong> test (property &amp; business owner-user); <strong>"like-kind"</strong>
            test (investor-lessor § 1033(g)). Basis of replacement = cost MINUS deferred gain. Holding
            period TACKS. § 121 may also apply for principal residence. § 165 alternative: deductible loss.
            Form 1040 + statement.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1033.h2.inputs">Inputs</h2>
            <form id="s1033-form" class="inline-form">
                <label><span data-i18n="view.s1033.label.proceeds">Conversion proceeds ($)</span>
                    <input type="number" step="10000" name="conversion_proceeds" value="${state.conversion_proceeds}"></label>
                <label><span data-i18n="view.s1033.label.basis">Property basis ($)</span>
                    <input type="number" step="10000" name="property_basis" value="${state.property_basis}"></label>
                <label><span data-i18n="view.s1033.label.replacement">Replacement cost ($)</span>
                    <input type="number" step="10000" name="replacement_cost" value="${state.replacement_cost}"></label>
                <label><span data-i18n="view.s1033.label.type">Conversion type</span>
                    <select name="conversion_type">
                        <option value="casualty" ${state.conversion_type === 'casualty' ? 'selected' : ''}>Casualty (fire, storm)</option>
                        <option value="theft" ${state.conversion_type === 'theft' ? 'selected' : ''}>Theft</option>
                        <option value="condemnation" ${state.conversion_type === 'condemnation' ? 'selected' : ''}>Condemnation / threat thereof</option>
                        <option value="seizure" ${state.conversion_type === 'seizure' ? 'selected' : ''}>Federal seizure / requisition</option>
                        <option value="destruction" ${state.conversion_type === 'destruction' ? 'selected' : ''}>Destruction by act of God</option>
                    </select>
                </label>
                <label><span data-i18n="view.s1033.label.real">Real property?</span>
                    <input type="checkbox" name="is_real_property" ${state.is_real_property ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1033.label.disaster">Federally declared disaster area?</span>
                    <input type="checkbox" name="is_federally_declared_disaster" ${state.is_federally_declared_disaster ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1033.label.complete">Replacement complete in window?</span>
                    <input type="checkbox" name="replacement_complete" ${state.replacement_complete ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1033.label.similar">Same or similar / like-kind?</span>
                    <input type="checkbox" name="same_or_similar" ${state.same_or_similar ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1033.label.years">Years to replace</span>
                    <input type="number" step="1" name="years_to_replace" value="${state.years_to_replace}"></label>
                <label><span data-i18n="view.s1033.label.principal">Principal residence (§ 121)?</span>
                    <input type="checkbox" name="is_principal_residence" ${state.is_principal_residence ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1033.label.business">Business property?</span>
                    <input type="checkbox" name="is_business_property" ${state.is_business_property ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1033.label.fmv">FMV at conversion (unrestricted) ($)</span>
                    <input type="number" step="10000" name="fmv_unrestricted" value="${state.fmv_unrestricted}"></label>
                <button class="primary" type="submit" data-i18n="view.s1033.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1033-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1033.h2.replacement_window">Replacement window by conversion type</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s1033.th.type">Type</th>
                    <th data-i18n="view.s1033.th.window">Replacement window</th>
                    <th data-i18n="view.s1033.th.start">Start of window</th>
                </tr></thead>
                <tbody>
                    <tr><td>Casualty / theft personal property</td><td>2 years</td><td>End of tax year of gain realization</td></tr>
                    <tr><td>Casualty / theft business property</td><td>2 years</td><td>End of tax year of gain realization</td></tr>
                    <tr><td>Condemnation real property used in trade / biz</td><td>3 years</td><td>End of tax year of gain realization</td></tr>
                    <tr><td>Federally declared disaster area</td><td>4 years (§ 1033(h))</td><td>End of tax year of gain realization</td></tr>
                    <tr><td>Live stock from drought (§ 1033(e))</td><td>4 years</td><td>End of tax year of gain realization</td></tr>
                    <tr><td>Principal residence in disaster (§ 1033(h)(1)(A))</td><td>4 years</td><td>End of tax year of gain realization</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1033.h2.qualified_replacement">"Qualified replacement" property standards</h2>
            <ul class="muted small">
                <li data-i18n="view.s1033.qual.owner_user">Owner-user: "same or similar use" — stricter test, functional equivalence</li>
                <li data-i18n="view.s1033.qual.investor_lessor">Investor-lessor: like-kind (§ 1033(g)) — broader category test</li>
                <li data-i18n="view.s1033.qual.condemned_real">Condemned real property: like-kind real prop in trade / biz / investment qualifies</li>
                <li data-i18n="view.s1033.qual.related_party">§ 1033(i) related party: limited to certain real property situations</li>
                <li data-i18n="view.s1033.qual.no_inventory">Inventory or stock in trade NOT eligible as replacement</li>
                <li data-i18n="view.s1033.qual.controlled_corp">§ 1033(a)(2)(A) replacement via 80%+ controlled corp acquisition allowed</li>
                <li data-i18n="view.s1033.qual.gain_only">Gain ONLY — loss not deferred under § 1033 (deductible under § 165)</li>
                <li data-i18n="view.s1033.qual.election_timely">Election: by filing return for year of gain (extended return + amended w/in 3 yrs)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1033.h2.s121">§ 121 home interaction</h2>
            <ul class="muted small">
                <li data-i18n="view.s1033.s121.exclusion">§ 121: $250K / $500K MFJ exclusion on principal residence sale (2 of 5 yr ownership + use)</li>
                <li data-i18n="view.s1033.s121.combine">Casualty of home: § 121 exclusion FIRST, then § 1033 deferral on remaining gain</li>
                <li data-i18n="view.s1033.s121.partial">Partial use § 121: reduced exclusion + § 1033 on remainder</li>
                <li data-i18n="view.s1033.s121.example">Example: $700K proceeds, $150K basis = $550K gain; § 121 excludes $500K → $50K § 1033 deferral on remainder</li>
                <li data-i18n="view.s1033.s121.window">§ 121 has no replacement window (used once per 2 yrs); § 1033 deferral additional</li>
            </ul>
        </div>
    `;
    document.getElementById('s1033-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.conversion_proceeds = Number(fd.get('conversion_proceeds')) || 0;
        state.property_basis = Number(fd.get('property_basis')) || 0;
        state.replacement_cost = Number(fd.get('replacement_cost')) || 0;
        state.conversion_type = fd.get('conversion_type');
        state.is_real_property = !!fd.get('is_real_property');
        state.is_federally_declared_disaster = !!fd.get('is_federally_declared_disaster');
        state.replacement_complete = !!fd.get('replacement_complete');
        state.same_or_similar = !!fd.get('same_or_similar');
        state.years_to_replace = Number(fd.get('years_to_replace')) || 0;
        state.is_principal_residence = !!fd.get('is_principal_residence');
        state.is_business_property = !!fd.get('is_business_property');
        state.fmv_unrestricted = Number(fd.get('fmv_unrestricted')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1033-output');
    if (!el) return;
    let windowYears = 2;
    if (state.conversion_type === 'condemnation' && state.is_real_property) windowYears = 3;
    if (state.is_federally_declared_disaster) windowYears = 4;
    const windowMet = state.years_to_replace <= windowYears;
    const qualifies = windowMet && state.replacement_complete && state.same_or_similar;
    const totalGain = Math.max(0, state.conversion_proceeds - state.property_basis);
    const reinvestedPortion = Math.min(state.replacement_cost, state.conversion_proceeds);
    const unreinvestedProceeds = state.conversion_proceeds - reinvestedPortion;
    const recognizedGain = qualifies ? Math.min(totalGain, unreinvestedProceeds) : totalGain;
    const deferredGain = qualifies ? Math.max(0, totalGain - recognizedGain) : 0;
    const newBasis = state.replacement_cost - deferredGain;
    const recognizedTax = recognizedGain * 0.20;
    const deferredValue = deferredGain * 0.20;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1033.h2.result">§ 1033 computation</h2>
            <div class="cards">
                <div class="card ${qualifies ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s1033.card.qualifies">Deferral qualifies?</div>
                    <div class="value">${qualifies ? esc(t('view.s1033.status.yes')) : esc(t('view.s1033.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1033.card.window">Replacement window</div>
                    <div class="value">${windowYears} ${esc(t('view.s1033.units.years'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1033.card.gain">Total realized gain</div>
                    <div class="value">$${totalGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1033.card.recognized">Recognized gain</div>
                    <div class="value">$${recognizedGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1033.card.deferred">Deferred gain</div>
                    <div class="value">$${deferredGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1033.card.new_basis">New basis (replacement)</div>
                    <div class="value">$${newBasis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1033.card.tax">Tax on recognized</div>
                    <div class="value">$${recognizedTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1033.card.value_deferred">NPV deferral benefit</div>
                    <div class="value">$${deferredValue.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${!qualifies && state.replacement_complete ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s1033.fail_note">
                    Deferral failed — replacement outside window OR not "same or similar" / "like-kind".
                    Full gain recognized currently. Consider amended return + extended election if facts allow.
                </p>
            ` : ''}
        </div>
    `;
}
