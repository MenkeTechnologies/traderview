// IRC § 530 — Coverdell Education Savings Account (ESA).
// $2,000/yr per beneficiary contribution cap. AGI phase-out: $95-110k single / $190-220k MFJ.
// Tax-free growth + tax-free for qualified K-12 + higher-ed expenses.
// Contributor's AGI matters, not beneficiary's. Beneficiary must use by age 30 (or roll to family).
// Coverdell ESA + 529 plan can be used together; less popular than 529 due to low contribution cap.

import { currentViewToken, viewIsCurrent } from '../app.js';

const CONTRIBUTION_CAP = 2_000;
const PHASEOUT_LOW_SINGLE = 95_000;
const PHASEOUT_HIGH_SINGLE = 110_000;
const PHASEOUT_LOW_MFJ = 190_000;
const PHASEOUT_HIGH_MFJ = 220_000;

let state = {
    contributor_filing_status: 'single',
    contributor_magi: 0,
    desired_contribution: 0,
    other_coverdell_contributions: 0,
    beneficiary_age: 5,
    annual_distributions: 0,
    qualified_k12: 0,
    qualified_higher_ed: 0,
    fed_marginal_rate: 0.32,
};

export async function renderSection530(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s530.h1.title">// § 530 COVERDELL ESA</span></h1>
        <p class="muted small" data-i18n="view.s530.hint.intro">
            <strong>$2,000/yr per beneficiary cap</strong> (aggregate across all contributors).
            AGI phase-out: <strong>$95-110k single / $190-220k MFJ</strong>. Tax-free growth +
            tax-free withdrawals for qualified K-12 + higher-ed expenses. Contributor's AGI
            matters, NOT beneficiary's. <strong>Beneficiary must use by age 30</strong> (or roll
            to family member). § 530 ESA + § 529 plan COMPLEMENTARY (use both); ESA covers
            broader K-12 expenses than 529.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s530.h2.inputs">Inputs</h2>
            <form id="s530-form" class="inline-form">
                <label><span data-i18n="view.s530.label.filing">Contributor filing status</span>
                    <select name="contributor_filing_status">
                        <option value="single" ${state.contributor_filing_status === 'single' ? 'selected' : ''}>Single / HoH</option>
                        <option value="mfj" ${state.contributor_filing_status === 'mfj' ? 'selected' : ''}>MFJ</option>
                    </select>
                </label>
                <label><span data-i18n="view.s530.label.magi">Contributor MAGI ($)</span>
                    <input type="number" step="1000" name="contributor_magi" value="${state.contributor_magi}"></label>
                <label><span data-i18n="view.s530.label.contrib">Your desired contribution ($)</span>
                    <input type="number" step="100" name="desired_contribution" value="${state.desired_contribution}"></label>
                <label><span data-i18n="view.s530.label.other">Other Coverdell contributions YTD ($)</span>
                    <input type="number" step="100" name="other_coverdell_contributions" value="${state.other_coverdell_contributions}"></label>
                <label><span data-i18n="view.s530.label.age">Beneficiary age</span>
                    <input type="number" step="1" name="beneficiary_age" value="${state.beneficiary_age}"></label>
                <label><span data-i18n="view.s530.label.dist">Total distributions ($)</span>
                    <input type="number" step="100" name="annual_distributions" value="${state.annual_distributions}"></label>
                <label><span data-i18n="view.s530.label.k12">Qualified K-12 expenses ($)</span>
                    <input type="number" step="100" name="qualified_k12" value="${state.qualified_k12}"></label>
                <label><span data-i18n="view.s530.label.higher_ed">Qualified higher-ed expenses ($)</span>
                    <input type="number" step="100" name="qualified_higher_ed" value="${state.qualified_higher_ed}"></label>
                <label><span data-i18n="view.s530.label.fed_rate">Federal marginal %</span>
                    <input type="number" step="0.01" name="fed_marginal_rate" value="${state.fed_marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s530.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s530-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s530.h2.qualified">Qualified expenses (broader than 529)</h2>
            <ul class="muted small">
                <li data-i18n="view.s530.qual.tuition">K-12 tuition + required fees + books</li>
                <li data-i18n="view.s530.qual.uniforms">Uniforms (if required by school)</li>
                <li data-i18n="view.s530.qual.transport">Transportation to/from school (school bus + carpool)</li>
                <li data-i18n="view.s530.qual.supplies">Educational supplies + computer + internet</li>
                <li data-i18n="view.s530.qual.tutoring">Tutoring + extended learning programs</li>
                <li data-i18n="view.s530.qual.before_after">Before / after school programs</li>
                <li data-i18n="view.s530.qual.special_needs">Special needs services</li>
                <li data-i18n="view.s530.qual.higher_ed">Higher ed: tuition / fees / books / room+board (similar to 529)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s530.h2.vs_529">ESA vs 529 comparison</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s530.th.feature">Feature</th>
                    <th>Coverdell ESA</th>
                    <th>§ 529 Plan</th>
                </tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s530.row.cap">Annual cap</td><td>$2,000/beneficiary</td><td>~$10-20k state-specific</td></tr>
                    <tr><td data-i18n="view.s530.row.income">Income phase-out</td><td>YES ($95-220k)</td><td>NO income limits</td></tr>
                    <tr><td data-i18n="view.s530.row.investments">Investment options</td><td>Self-directed (any)</td><td>State plan-limited menu</td></tr>
                    <tr><td data-i18n="view.s530.row.k12">K-12 expenses</td><td>BROAD: tuition + supplies + before/after</td><td>NARROW: tuition only ($10k/yr cap)</td></tr>
                    <tr><td data-i18n="view.s530.row.state_ded">State deduction</td><td>NO</td><td>Most states YES (residence-only often)</td></tr>
                    <tr><td data-i18n="view.s530.row.age_30">Age 30 rule</td><td>YES — must use or roll over</td><td>NO age limit</td></tr>
                    <tr><td data-i18n="view.s530.row.who_owns">Who controls</td><td>Donor-controlled</td><td>Owner (donor or other)</td></tr>
                    <tr><td data-i18n="view.s530.row.fafsa">FAFSA treatment</td><td>Asset of beneficiary (worst)</td><td>Asset of parent (better)</td></tr>
                </tbody>
            </table>
        </div>
    `;
    document.getElementById('s530-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.contributor_filing_status = fd.get('contributor_filing_status');
        state.contributor_magi = Number(fd.get('contributor_magi')) || 0;
        state.desired_contribution = Number(fd.get('desired_contribution')) || 0;
        state.other_coverdell_contributions = Number(fd.get('other_coverdell_contributions')) || 0;
        state.beneficiary_age = Number(fd.get('beneficiary_age')) || 5;
        state.annual_distributions = Number(fd.get('annual_distributions')) || 0;
        state.qualified_k12 = Number(fd.get('qualified_k12')) || 0;
        state.qualified_higher_ed = Number(fd.get('qualified_higher_ed')) || 0;
        state.fed_marginal_rate = Number(fd.get('fed_marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s530-output');
    if (!el) return;
    const low = state.contributor_filing_status === 'mfj' ? PHASEOUT_LOW_MFJ : PHASEOUT_LOW_SINGLE;
    const high = state.contributor_filing_status === 'mfj' ? PHASEOUT_HIGH_MFJ : PHASEOUT_HIGH_SINGLE;
    let factor;
    if (state.contributor_magi <= low) factor = 1;
    else if (state.contributor_magi >= high) factor = 0;
    else factor = (high - state.contributor_magi) / (high - low);
    const personalCap = CONTRIBUTION_CAP * factor;
    const aggregateRoom = Math.max(0, CONTRIBUTION_CAP - state.other_coverdell_contributions);
    const allowedContribution = Math.min(state.desired_contribution, personalCap, aggregateRoom);
    const over18Warning = state.beneficiary_age > 18 && state.beneficiary_age < 30;
    const over30Warning = state.beneficiary_age >= 30;
    const totalQualified = state.qualified_k12 + state.qualified_higher_ed;
    const nonQualified = Math.max(0, state.annual_distributions - totalQualified);
    const earningsRatio = 0.30;  // placeholder
    const taxableEarnings = nonQualified * earningsRatio;
    const fedTax = taxableEarnings * state.fed_marginal_rate;
    const tenPctPenalty = taxableEarnings * 0.10;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s530.h2.result">ESA outcome</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s530.card.factor">Phase-out factor</div>
                    <div class="value">${(factor * 100).toFixed(0)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s530.card.personal_cap">Personal cap (after phase-out)</div>
                    <div class="value">$${personalCap.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s530.card.allowed">Allowed contribution</div>
                    <div class="value">$${allowedContribution.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s530.card.qualified_dist">Qualified distributions</div>
                    <div class="value">$${totalQualified.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${nonQualified > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s530.card.non_qualified">Non-qualified</div>
                    <div class="value">$${nonQualified.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s530.card.fed_tax">Fed tax on earnings</div>
                    <div class="value">$${fedTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s530.card.penalty">10% penalty</div>
                    <div class="value">$${tenPctPenalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${over18Warning ? `
                <p class="muted small" style="margin-top:10px" data-i18n="view.s530.warning.18">
                    Beneficiary age 18-30: contributions allowed only for special-needs beneficiaries.
                    Must use balance by age 30 or transfer to family member.
                </p>
            ` : ''}
            ${over30Warning ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s530.warning.30">
                    Beneficiary age 30+: account must distribute within 30 days (taxable to extent
                    of earnings + 10% penalty) OR transfer to family member &lt; 30. Special-needs
                    exception applies.
                </p>
            ` : ''}
        </div>
    `;
}
