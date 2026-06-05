// IRC § 213 — Medical & Dental Expense Deduction (Schedule A).
// Itemize only. Deduct expenses > 7.5% of AGI. Spouse / dependent medical expenses count.
// Includes LTC services, prescription drugs, capital expenditures (e.g. wheelchair ramp).
// Cosmetic surgery, gym, vitamins generally NOT deductible. HSA-paid amounts NOT deductible (double-dip).

import { currentViewToken, viewIsCurrent } from '../app.js';

const AGI_FLOOR = 0.075;

let state = {
    agi: 0,
    insurance_premiums: 0,
    doctor_dentist: 0,
    hospital: 0,
    prescriptions: 0,
    ltc_services: 0,
    ltc_premiums_age_adjusted: 0,
    mileage_miles: 0,
    capital_expenditures: 0,
    hsa_reimbursed: 0,
    employer_reimbursed: 0,
    marginal_rate: 0.32,
};

const MEDICAL_MILEAGE_RATE_2024 = 0.21;

export async function renderSection213(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s213.h1.title">// § 213 MEDICAL EXPENSE DEDUCTION</span></h1>
        <p class="muted small" data-i18n="view.s213.hint.intro">
            Itemize only. Deduct medical / dental expenses <strong>&gt; 7.5% of AGI</strong>.
            Spouse + dependent expenses count. LTC services + LTC premiums (age-limited) qualify.
            Capital expenditures for medical necessity (wheelchair ramp, lift, custom kitchen)
            also count. <strong>NOT deductible:</strong> cosmetic, gym memberships, vitamins,
            HSA-paid amounts (no double-dip), employer-reimbursed.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s213.h2.inputs">Inputs</h2>
            <form id="s213-form" class="inline-form">
                <label><span data-i18n="view.s213.label.agi">AGI ($)</span>
                    <input type="number" step="0.01" name="agi" value="${state.agi}"></label>
                <label><span data-i18n="view.s213.label.premiums">Health insurance premiums (after-tax) ($)</span>
                    <input type="number" step="0.01" name="insurance_premiums" value="${state.insurance_premiums}"></label>
                <label><span data-i18n="view.s213.label.doctors">Doctors / dentists / hospitals ($)</span>
                    <input type="number" step="0.01" name="doctor_dentist" value="${state.doctor_dentist}"></label>
                <label><span data-i18n="view.s213.label.hospital">Surgery / hospital ($)</span>
                    <input type="number" step="0.01" name="hospital" value="${state.hospital}"></label>
                <label><span data-i18n="view.s213.label.prescriptions">Prescription drugs / insulin ($)</span>
                    <input type="number" step="0.01" name="prescriptions" value="${state.prescriptions}"></label>
                <label><span data-i18n="view.s213.label.ltc_services">LTC services ($)</span>
                    <input type="number" step="0.01" name="ltc_services" value="${state.ltc_services}"></label>
                <label><span data-i18n="view.s213.label.ltc_premiums">LTC premiums age-limited ($)</span>
                    <input type="number" step="0.01" name="ltc_premiums_age_adjusted" value="${state.ltc_premiums_age_adjusted}"></label>
                <label><span data-i18n="view.s213.label.miles">Medical mileage (miles)</span>
                    <input type="number" step="1" name="mileage_miles" value="${state.mileage_miles}"></label>
                <label><span data-i18n="view.s213.label.capital">Capital expenditures ($)</span>
                    <input type="number" step="0.01" name="capital_expenditures" value="${state.capital_expenditures}"></label>
                <label><span data-i18n="view.s213.label.hsa">HSA-reimbursed ($)</span>
                    <input type="number" step="0.01" name="hsa_reimbursed" value="${state.hsa_reimbursed}"></label>
                <label><span data-i18n="view.s213.label.employer">Employer-reimbursed ($)</span>
                    <input type="number" step="0.01" name="employer_reimbursed" value="${state.employer_reimbursed}"></label>
                <label><span data-i18n="view.s213.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s213.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s213-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s213.h2.qualifying">Qualifying expenses</h2>
            <ul class="muted small">
                <li data-i18n="view.s213.qual.doctors">Doctors, dentists, surgeons, chiropractors, psychiatrists, optometrists, podiatrists</li>
                <li data-i18n="view.s213.qual.prescription">Prescription drugs, insulin (OTC NOT)</li>
                <li data-i18n="view.s213.qual.hospital">Hospital + nursing home (medical care portion)</li>
                <li data-i18n="view.s213.qual.ambulance">Ambulance + medical mileage @ 21¢/mi (2024)</li>
                <li data-i18n="view.s213.qual.glasses">Eyeglasses, contacts, hearing aids, dentures</li>
                <li data-i18n="view.s213.qual.ltc">Qualified LTC services + age-limited premiums</li>
                <li data-i18n="view.s213.qual.capital">Capital expenditures (wheelchair ramp, stair lift, modified vehicle) — reduce by increase in property value</li>
                <li data-i18n="view.s213.qual.smoking">Smoking-cessation programs + prescription nicotine</li>
                <li data-i18n="view.s213.qual.special_school">Special education for learning disabilities (medical necessity)</li>
                <li data-i18n="view.s213.qual.fertility">Fertility treatment + reproductive medical procedures</li>
                <li data-i18n="view.s213.qual.transgender">Transgender medical care (Rev. Rul. 2010-31)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s213.h2.not_qualifying">NOT qualifying</h2>
            <ul class="muted small">
                <li data-i18n="view.s213.not.cosmetic">Cosmetic surgery (unless restorative)</li>
                <li data-i18n="view.s213.not.gym">Gym / health club memberships</li>
                <li data-i18n="view.s213.not.vitamins">Vitamins / supplements (general health)</li>
                <li data-i18n="view.s213.not.toothpaste">Toothpaste / personal hygiene</li>
                <li data-i18n="view.s213.not.maternity">Maternity clothes</li>
                <li data-i18n="view.s213.not.funeral">Funeral / burial expenses</li>
                <li data-i18n="view.s213.not.illegal">Illegal operations / drugs</li>
                <li data-i18n="view.s213.not.hsa_double">HSA-paid amounts (would be double-deduction)</li>
            </ul>
        </div>
    `;
    document.getElementById('s213-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.agi = Number(fd.get('agi')) || 0;
        state.insurance_premiums = Number(fd.get('insurance_premiums')) || 0;
        state.doctor_dentist = Number(fd.get('doctor_dentist')) || 0;
        state.hospital = Number(fd.get('hospital')) || 0;
        state.prescriptions = Number(fd.get('prescriptions')) || 0;
        state.ltc_services = Number(fd.get('ltc_services')) || 0;
        state.ltc_premiums_age_adjusted = Number(fd.get('ltc_premiums_age_adjusted')) || 0;
        state.mileage_miles = Number(fd.get('mileage_miles')) || 0;
        state.capital_expenditures = Number(fd.get('capital_expenditures')) || 0;
        state.hsa_reimbursed = Number(fd.get('hsa_reimbursed')) || 0;
        state.employer_reimbursed = Number(fd.get('employer_reimbursed')) || 0;
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s213-output');
    if (!el) return;
    const mileageDeduction = state.mileage_miles * MEDICAL_MILEAGE_RATE_2024;
    const grossExpenses = state.insurance_premiums + state.doctor_dentist + state.hospital
        + state.prescriptions + state.ltc_services + state.ltc_premiums_age_adjusted
        + mileageDeduction + state.capital_expenditures;
    const reimbursed = state.hsa_reimbursed + state.employer_reimbursed;
    const netExpenses = Math.max(0, grossExpenses - reimbursed);
    const floor = state.agi * AGI_FLOOR;
    const deductible = Math.max(0, netExpenses - floor);
    const taxSavings = deductible * state.marginal_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s213.h2.result">Calculation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s213.card.gross">Gross expenses</div>
                    <div class="value">$${grossExpenses.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s213.card.mileage">Mileage deduction</div>
                    <div class="value">$${mileageDeduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s213.card.reimbursed">Reimbursed (subtract)</div>
                    <div class="value">$${reimbursed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s213.card.net">Net expenses</div>
                    <div class="value">$${netExpenses.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s213.card.floor">7.5% AGI floor</div>
                    <div class="value">$${floor.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s213.card.deductible">Schedule A deductible</div>
                    <div class="value">$${deductible.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s213.card.savings">Tax savings</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
