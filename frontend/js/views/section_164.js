// IRC § 164 — Deduction for Taxes Paid (SALT + Foreign + Property).
// Itemized deduction for state + local income/sales tax + real property tax + personal property tax + foreign income tax.
// TCJA $10,000 SALT cap (2018-2025) — sunsets to no-cap pre-TCJA rules absent extension.
// Foreign income taxes: choose deduction OR § 27 foreign tax credit (FTC is usually better).

import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    filing_status: 'mfj',
    year: 2024,
    state_income_tax: 0,
    state_sales_tax: 0,
    use_sales_election: false,
    local_income_tax: 0,
    real_property_tax_residence: 0,
    real_property_tax_other: 0,
    personal_property_tax: 0,
    foreign_income_tax_paid: 0,
    use_foreign_tax_credit: true,
    federal_se_tax_paid: 0,
    is_business_property_tax: false,
    business_property_tax: 0,
    is_rental_property_tax: false,
    rental_property_tax: 0,
    salt_cap_workaround_ptet: false,
    ptet_paid: 0,
    s164a_5_state_business: 0,
    qualified_passive_activity: 0,
    is_amt_paying: false,
    s55_amt_calc: 0,
    obbba_extension: false,
};

export async function renderSection164(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s164.h1.title">// § 164 TAXES DEDUCTION + SALT CAP</span></h1>
        <p class="muted small" data-i18n="view.s164.hint.intro">
            <strong>§ 164</strong> itemized deduction for: (1) state + local income OR sales tax,
            (2) state + local real property tax, (3) state + local personal property tax, (4) foreign
            income tax (OR § 27 FTC). <strong>TCJA $10,000 SALT cap</strong> ($5K MFS) applies 2018-2025
            — combined cap on income/sales + property tax. <strong>Business + rental property tax
            NOT subject to cap</strong> — Schedule C / E / F deduction. <strong>State Pass-Through
            Entity Tax (PTET) workaround:</strong> ~36 states allow pass-through to elect entity-level
            state tax → SALT cap avoided. <strong>SALT cap sunsets Dec 31, 2025</strong> absent OBBBA
            extension. <strong>§ 164(a)(5) state business taxes</strong> separately fully deductible
            (treated as business expense).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s164.h2.inputs">Inputs</h2>
            <form id="s164-form" class="inline-form">
                <label><span data-i18n="view.s164.label.status">Filing status</span>
                    <select name="filing_status">
                        <option value="single" ${state.filing_status === 'single' ? 'selected' : ''}>Single</option>
                        <option value="mfj" ${state.filing_status === 'mfj' ? 'selected' : ''}>MFJ</option>
                        <option value="hoh" ${state.filing_status === 'hoh' ? 'selected' : ''}>HOH</option>
                        <option value="mfs" ${state.filing_status === 'mfs' ? 'selected' : ''}>MFS</option>
                    </select>
                </label>
                <label><span data-i18n="view.s164.label.year">Year</span>
                    <input type="number" step="1" name="year" value="${state.year}"></label>
                <label><span data-i18n="view.s164.label.state_inc">State income tax ($)</span>
                    <input type="number" step="0.01" name="state_income_tax" value="${state.state_income_tax}"></label>
                <label><span data-i18n="view.s164.label.state_sales">State sales tax ($)</span>
                    <input type="number" step="0.01" name="state_sales_tax" value="${state.state_sales_tax}"></label>
                <label><span data-i18n="view.s164.label.elect_sales">Elect sales tax?</span>
                    <input type="checkbox" name="use_sales_election" ${state.use_sales_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s164.label.local_inc">Local income tax ($)</span>
                    <input type="number" step="0.01" name="local_income_tax" value="${state.local_income_tax}"></label>
                <label><span data-i18n="view.s164.label.real_residence">Real property tax (residence) ($)</span>
                    <input type="number" step="0.01" name="real_property_tax_residence" value="${state.real_property_tax_residence}"></label>
                <label><span data-i18n="view.s164.label.real_other">Real property tax (other) ($)</span>
                    <input type="number" step="0.01" name="real_property_tax_other" value="${state.real_property_tax_other}"></label>
                <label><span data-i18n="view.s164.label.personal_prop">Personal property tax ($)</span>
                    <input type="number" step="0.01" name="personal_property_tax" value="${state.personal_property_tax}"></label>
                <label><span data-i18n="view.s164.label.foreign">Foreign income tax paid ($)</span>
                    <input type="number" step="0.01" name="foreign_income_tax_paid" value="${state.foreign_income_tax_paid}"></label>
                <label><span data-i18n="view.s164.label.use_ftc">Use FTC instead?</span>
                    <input type="checkbox" name="use_foreign_tax_credit" ${state.use_foreign_tax_credit ? 'checked' : ''}></label>
                <label><span data-i18n="view.s164.label.fed_se">Federal SE tax paid ($)</span>
                    <input type="number" step="0.01" name="federal_se_tax_paid" value="${state.federal_se_tax_paid}"></label>
                <label><span data-i18n="view.s164.label.business_prop">Business property tax?</span>
                    <input type="checkbox" name="is_business_property_tax" ${state.is_business_property_tax ? 'checked' : ''}></label>
                <label><span data-i18n="view.s164.label.business_amt">Business property tax ($)</span>
                    <input type="number" step="0.01" name="business_property_tax" value="${state.business_property_tax}"></label>
                <label><span data-i18n="view.s164.label.rental_prop">Rental property tax?</span>
                    <input type="checkbox" name="is_rental_property_tax" ${state.is_rental_property_tax ? 'checked' : ''}></label>
                <label><span data-i18n="view.s164.label.rental_amt">Rental property tax ($)</span>
                    <input type="number" step="0.01" name="rental_property_tax" value="${state.rental_property_tax}"></label>
                <label><span data-i18n="view.s164.label.ptet">PTET election?</span>
                    <input type="checkbox" name="salt_cap_workaround_ptet" ${state.salt_cap_workaround_ptet ? 'checked' : ''}></label>
                <label><span data-i18n="view.s164.label.ptet_amt">PTET paid ($)</span>
                    <input type="number" step="0.01" name="ptet_paid" value="${state.ptet_paid}"></label>
                <label><span data-i18n="view.s164.label.s164a5">§ 164(a)(5) state business ($)</span>
                    <input type="number" step="0.01" name="s164a_5_state_business" value="${state.s164a_5_state_business}"></label>
                <label><span data-i18n="view.s164.label.passive">Qualified passive activity ($)</span>
                    <input type="number" step="0.01" name="qualified_passive_activity" value="${state.qualified_passive_activity}"></label>
                <label><span data-i18n="view.s164.label.amt_paying">AMT paying?</span>
                    <input type="checkbox" name="is_amt_paying" ${state.is_amt_paying ? 'checked' : ''}></label>
                <label><span data-i18n="view.s164.label.s55_amt">§ 55 AMT calculation ($)</span>
                    <input type="number" step="0.01" name="s55_amt_calc" value="${state.s55_amt_calc}"></label>
                <label><span data-i18n="view.s164.label.obbba">OBBBA extension applies?</span>
                    <input type="checkbox" name="obbba_extension" ${state.obbba_extension ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s164.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s164-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s164.h2.deductible">Deductible vs nondeductible taxes</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s164.tbl.type">Tax type</th><th data-i18n="view.s164.tbl.deductible">Deductible?</th><th data-i18n="view.s164.tbl.where">Where</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s164.tbl.state_inc">State / local income tax</td><td>YES (SALT cap)</td><td>Schedule A</td></tr>
                    <tr><td data-i18n="view.s164.tbl.state_sales">State / local sales tax</td><td data-i18n="view.s164.tbl.alt_inc">YES alt to income (SALT cap)</td><td>Schedule A</td></tr>
                    <tr><td data-i18n="view.s164.tbl.real_prop">Real property tax</td><td>YES (SALT cap)</td><td>Schedule A</td></tr>
                    <tr><td data-i18n="view.s164.tbl.pers_prop">Personal property tax (ad valorem)</td><td>YES (SALT cap)</td><td>Schedule A</td></tr>
                    <tr><td data-i18n="view.s164.tbl.foreign_inc">Foreign income tax</td><td data-i18n="view.s164.tbl.or_ftc">YES OR § 27 FTC (choose)</td><td>Schedule A OR Form 1116</td></tr>
                    <tr><td data-i18n="view.s164.tbl.se_tax">Federal SE tax (employer share)</td><td data-i18n="view.s164.tbl.adjust">Adjustment (above-line)</td><td>Schedule 1</td></tr>
                    <tr><td data-i18n="view.s164.tbl.business_prop">Business property tax</td><td>YES (no SALT cap)</td><td>Schedule C / E / F</td></tr>
                    <tr><td data-i18n="view.s164.tbl.gas_tax">Federal gas tax</td><td>NO (§ 275)</td><td>—</td></tr>
                    <tr><td data-i18n="view.s164.tbl.fed_inc">Federal income tax</td><td>NO (§ 275)</td><td>—</td></tr>
                    <tr><td data-i18n="view.s164.tbl.fica">FICA employee share</td><td>NO (§ 275)</td><td>—</td></tr>
                    <tr><td data-i18n="view.s164.tbl.estate">Federal estate tax</td><td>NO</td><td>—</td></tr>
                    <tr><td data-i18n="view.s164.tbl.special">Special assessments (sewer, water)</td><td>NO (capital)</td><td>—</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s164.h2.salt_cap">$10,000 SALT cap mechanics</h2>
            <ul class="muted small">
                <li data-i18n="view.s164.salt.cap_2018">2018-2025: $10,000 cap on combined state + local income + sales + property</li>
                <li data-i18n="view.s164.salt.mfs">MFS = $5,000 cap</li>
                <li data-i18n="view.s164.salt.flat">Flat — NOT phased by income</li>
                <li data-i18n="view.s164.salt.business">Business property tax excluded — Schedule C / E / F fully deductible</li>
                <li data-i18n="view.s164.salt.rental">Rental property tax: Schedule E fully deductible (NOT subject to cap)</li>
                <li data-i18n="view.s164.salt.foreign">Foreign income tax excluded — separate § 27 FTC choice</li>
                <li data-i18n="view.s164.salt.ptet">PTET workaround: ~36 states (CA, NY, NJ, MA, etc.) — entity pays tax + deducts as business expense</li>
                <li data-i18n="view.s164.salt.notice_2020_75">Notice 2020-75 blessed PTET — IRS confirms entity-level state tax fully deductible</li>
                <li data-i18n="view.s164.salt.sunset_2025">Cap sunsets Dec 31, 2025 — back to no-cap pre-TCJA absent OBBBA extension</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s164.h2.ptet">PTET workaround mechanics</h2>
            <ol class="muted small">
                <li data-i18n="view.s164.ptet.election">Pass-through entity (S-corp / partnership) elects to pay state tax at entity level</li>
                <li data-i18n="view.s164.ptet.entity_dedn">Entity deducts state tax fully — reduces federal taxable income</li>
                <li data-i18n="view.s164.ptet.k1">K-1 income passes through reduced</li>
                <li data-i18n="view.s164.ptet.partner_credit">Partner gets state tax credit on personal return (for federal SALT cap purpose, NOT a deduction)</li>
                <li data-i18n="view.s164.ptet.net_benefit">Net benefit: full federal deduction for state tax + state tax avoided at individual level</li>
                <li data-i18n="view.s164.ptet.states_36">36 states + DC offer PTET as of 2024</li>
                <li data-i18n="view.s164.ptet.aging_state_credit">Some states allow partner refundable credit; others reduce state liability dollar-for-dollar</li>
                <li data-i18n="view.s164.ptet.estimated">Must pay estimated tax during year (most states)</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s164.h2.foreign">Foreign income tax — § 164 vs § 27 FTC</h2>
            <ul class="muted small">
                <li data-i18n="view.s164.foreign.deduction">§ 164 deduction: reduces taxable income (33%-37% benefit)</li>
                <li data-i18n="view.s164.foreign.ftc">§ 27 FTC: dollar-for-dollar reduction in federal tax (100% benefit)</li>
                <li data-i18n="view.s164.foreign.almost_always_ftc">FTC almost always preferred — except when limited by FTC limitations (§ 904 baskets)</li>
                <li data-i18n="view.s164.foreign.baskets">§ 904 baskets: GILTI, passive, general, foreign branch — limits FTC</li>
                <li data-i18n="view.s164.foreign.carryover">FTC carryback 1 year + carryforward 10 years</li>
                <li data-i18n="view.s164.foreign.high_tax_jurisdiction">Highly-taxed CFC: § 954(b)(4) high-tax exclusion may avoid GILTI</li>
                <li data-i18n="view.s164.foreign.foreign_tax">"Foreign income tax" defined broadly — includes income, war profits, excess profits</li>
                <li data-i18n="view.s164.foreign.s903_in_lieu">§ 903 in-lieu-of tax (gross receipts substitute) — also creditable</li>
            </ul>
        </div>
    `;
    document.getElementById('s164-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.filing_status = fd.get('filing_status');
        state.year = Number(fd.get('year')) || 0;
        state.state_income_tax = Number(fd.get('state_income_tax')) || 0;
        state.state_sales_tax = Number(fd.get('state_sales_tax')) || 0;
        state.use_sales_election = !!fd.get('use_sales_election');
        state.local_income_tax = Number(fd.get('local_income_tax')) || 0;
        state.real_property_tax_residence = Number(fd.get('real_property_tax_residence')) || 0;
        state.real_property_tax_other = Number(fd.get('real_property_tax_other')) || 0;
        state.personal_property_tax = Number(fd.get('personal_property_tax')) || 0;
        state.foreign_income_tax_paid = Number(fd.get('foreign_income_tax_paid')) || 0;
        state.use_foreign_tax_credit = !!fd.get('use_foreign_tax_credit');
        state.federal_se_tax_paid = Number(fd.get('federal_se_tax_paid')) || 0;
        state.is_business_property_tax = !!fd.get('is_business_property_tax');
        state.business_property_tax = Number(fd.get('business_property_tax')) || 0;
        state.is_rental_property_tax = !!fd.get('is_rental_property_tax');
        state.rental_property_tax = Number(fd.get('rental_property_tax')) || 0;
        state.salt_cap_workaround_ptet = !!fd.get('salt_cap_workaround_ptet');
        state.ptet_paid = Number(fd.get('ptet_paid')) || 0;
        state.s164a_5_state_business = Number(fd.get('s164a_5_state_business')) || 0;
        state.qualified_passive_activity = Number(fd.get('qualified_passive_activity')) || 0;
        state.is_amt_paying = !!fd.get('is_amt_paying');
        state.s55_amt_calc = Number(fd.get('s55_amt_calc')) || 0;
        state.obbba_extension = !!fd.get('obbba_extension');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s164-output');
    if (!el) return;
    const income_tax = state.use_sales_election ? state.state_sales_tax : state.state_income_tax;
    const total_subject_to_cap = income_tax + state.local_income_tax + state.real_property_tax_residence + state.real_property_tax_other + state.personal_property_tax;
    const cap = state.filing_status === 'mfs' ? 5000 : 10000;
    const salt_after_cap = state.obbba_extension ? Math.min(total_subject_to_cap, cap) : Math.min(total_subject_to_cap, cap);
    const business_full = state.business_property_tax + state.rental_property_tax + state.s164a_5_state_business;
    const foreign_deduction = state.use_foreign_tax_credit ? 0 : state.foreign_income_tax_paid;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s164.h2.result">§ 164 SALT cap + deduction</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s164.card.total">Subject to SALT cap</div><div class="value">$${total_subject_to_cap.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s164.card.cap">Cap</div><div class="value">$${cap.toLocaleString()}</div></div>
                <div class="card ${total_subject_to_cap > cap ? 'neg' : 'pos'}"><div class="label" data-i18n="view.s164.card.allowed">SALT allowed</div><div class="value">$${salt_after_cap.toLocaleString()}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.s164.card.business">Business (no cap)</div><div class="value">$${business_full.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s164.card.foreign">Foreign deduction</div><div class="value">$${foreign_deduction.toLocaleString()}</div></div>
                <div class="card ${total_subject_to_cap > cap ? 'neg' : 'pos'}"><div class="label" data-i18n="view.s164.card.lost">Lost to cap</div><div class="value">$${Math.max(0, total_subject_to_cap - cap).toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
