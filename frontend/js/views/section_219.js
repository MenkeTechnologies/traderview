// IRC § 219 — Traditional IRA Deduction Limits.
// 2024: $7,000 base ($8,000 if 50+). Phase-out if active participant in employer plan:
// Single: $77k-$87k. MFJ (both active): $123k-$143k. MFJ (spouse active): $230k-$240k.
// No income limit if neither spouse covered by employer plan.
// Roth IRA contributions covered by § 408A (different MAGI limits + no deduction).

import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const IRA_LIMIT_2024 = 7_000;
const CATCH_UP_50 = 1_000;
const SINGLE_PHASE_LOW = 77_000;
const SINGLE_PHASE_HIGH = 87_000;
const MFJ_BOTH_ACTIVE_LOW = 123_000;
const MFJ_BOTH_ACTIVE_HIGH = 143_000;
const MFJ_SPOUSE_ACTIVE_LOW = 230_000;
const MFJ_SPOUSE_ACTIVE_HIGH = 240_000;
const MFS_PHASE_LOW = 0;
const MFS_PHASE_HIGH = 10_000;

let state = {
    age: 35,
    spouse_age: 35,
    filing_status: 'single',
    self_active_in_plan: false,
    spouse_active_in_plan: false,
    magi: 0,
    intended_contribution: 0,
    earned_income: 0,
    marginal_rate: 0.32,
};

export async function renderSection219(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s219.h1.title">// § 219 IRA DEDUCTION</span></h1>
        <p class="muted small" data-i18n="view.s219.hint.intro">
            <strong>2024: $7,000 base ($8,000 if 50+)</strong>. Phase-out if active participant in
            employer plan: <strong>Single: $77k-$87k. MFJ (both active): $123k-$143k. MFJ (spouse
            active): $230k-$240k.</strong> <strong>No income limit</strong> if neither spouse
            covered by employer plan. Spousal IRA: working spouse covers non-working with own
            earned income. Roth (§ 408A) has different MAGI limits + no deduction.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s219.h2.inputs">Inputs</h2>
            <form id="s219-form" class="inline-form">
                <label><span data-i18n="view.s219.label.age">Your age</span>
                    <input type="number" step="1" name="age" value="${state.age}"></label>
                <label><span data-i18n="view.s219.label.spouse_age">Spouse age</span>
                    <input type="number" step="1" name="spouse_age" value="${state.spouse_age}"></label>
                <label><span data-i18n="view.s219.label.filing">Filing status</span>
                    <select name="filing_status">
                        <option value="single" ${state.filing_status === 'single' ? 'selected' : ''}>Single / HoH</option>
                        <option value="mfj" ${state.filing_status === 'mfj' ? 'selected' : ''}>MFJ</option>
                        <option value="mfs" ${state.filing_status === 'mfs' ? 'selected' : ''}>MFS</option>
                    </select>
                </label>
                <label><span data-i18n="view.s219.label.self_active">You active in employer plan?</span>
                    <input type="checkbox" name="self_active_in_plan" ${state.self_active_in_plan ? 'checked' : ''}></label>
                <label><span data-i18n="view.s219.label.spouse_active">Spouse active in employer plan?</span>
                    <input type="checkbox" name="spouse_active_in_plan" ${state.spouse_active_in_plan ? 'checked' : ''}></label>
                <label><span data-i18n="view.s219.label.magi">MAGI ($)</span>
                    <input type="number" step="1000" name="magi" value="${state.magi}"></label>
                <label><span data-i18n="view.s219.label.contribution">Intended contribution ($)</span>
                    <input type="number" step="100" name="intended_contribution" value="${state.intended_contribution}"></label>
                <label><span data-i18n="view.s219.label.earned">Earned income ($)</span>
                    <input type="number" step="1000" name="earned_income" value="${state.earned_income}"></label>
                <label><span data-i18n="view.s219.label.marginal">Marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s219.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s219-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s219.h2.active_participant">"Active participant" definition</h2>
            <ul class="muted small">
                <li data-i18n="view.s219.ap.401k">Eligible to participate in employer 401(k) / 403(b) / 457(b) — even if no contributions made</li>
                <li data-i18n="view.s219.ap.defined_benefit">Eligible for employer defined-benefit pension</li>
                <li data-i18n="view.s219.ap.contributions">Received employer contributions during year (SEP, SIMPLE)</li>
                <li data-i18n="view.s219.ap.taft_hartley">Multi-employer plan / Taft-Hartley plan</li>
                <li data-i18n="view.s219.ap.federal">Federal Civil Service Retirement System / FERS</li>
                <li data-i18n="view.s219.ap.union">Union pension plans</li>
                <li data-i18n="view.s219.ap.box_13">W-2 Box 13 "Retirement plan" checkbox checked</li>
                <li data-i18n="view.s219.ap.exclusions">NOT: § 457(b) gov't only (after 2007), pure deferred comp, 401(k) you joined late in year</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s219.h2.nondeductible">Non-deductible contributions</h2>
            <ul class="muted small">
                <li data-i18n="view.s219.nd.form_8606">File Form 8606 every year to track basis (lifetime!)</li>
                <li data-i18n="view.s219.nd.pro_rata">Backdoor Roth: pro-rata rule applies if pre-tax IRA balance exists</li>
                <li data-i18n="view.s219.nd.cost_basis">Future Roth conversion: pro-rata between basis + earnings</li>
                <li data-i18n="view.s219.nd.beneficiary">Beneficiary inherits Form 8606 basis</li>
                <li data-i18n="view.s219.nd.50_penalty">Form 8606 failure to file: $50 penalty per year (Form 8606 standalone if no return)</li>
                <li data-i18n="view.s219.nd.no_earnings">Earnings still tax-deferred until distribution</li>
                <li data-i18n="view.s219.nd.spousal_separate">Spousal basis tracked separately on each return</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s219.h2.deadlines">Contribution deadlines</h2>
            <ul class="muted small">
                <li data-i18n="view.s219.dl.april_15">April 15 (or tax day) of following year for prior-year contribution</li>
                <li data-i18n="view.s219.dl.no_extension">Extension to file does NOT extend contribution deadline</li>
                <li data-i18n="view.s219.dl.designation">Must designate "prior year" with custodian</li>
                <li data-i18n="view.s219.dl.recharacterization">Roth conversion CANNOT be recharacterized (TCJA repeal)</li>
                <li data-i18n="view.s219.dl.excess_removal">Excess contribution removal by Oct 15 (extended) avoids 6% penalty</li>
                <li data-i18n="view.s219.dl.secure_2_simple">SECURE 2.0 SIMPLE IRA Roth designation 2023+</li>
            </ul>
        </div>
    `;
    document.getElementById('s219-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.age = Number(fd.get('age')) || 35;
        state.spouse_age = Number(fd.get('spouse_age')) || 35;
        state.filing_status = fd.get('filing_status');
        state.self_active_in_plan = !!fd.get('self_active_in_plan');
        state.spouse_active_in_plan = !!fd.get('spouse_active_in_plan');
        state.magi = Number(fd.get('magi')) || 0;
        state.intended_contribution = Number(fd.get('intended_contribution')) || 0;
        state.earned_income = Number(fd.get('earned_income')) || 0;
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s219-output');
    if (!el) return;
    const personalLimit = state.age >= 50 ? IRA_LIMIT_2024 + CATCH_UP_50 : IRA_LIMIT_2024;
    let phaseLow, phaseHigh, phaseApplies;
    if (state.filing_status === 'single' || state.filing_status === 'hoh') {
        if (state.self_active_in_plan) {
            phaseLow = SINGLE_PHASE_LOW;
            phaseHigh = SINGLE_PHASE_HIGH;
            phaseApplies = true;
        } else {
            phaseApplies = false;
        }
    } else if (state.filing_status === 'mfj') {
        if (state.self_active_in_plan) {
            phaseLow = MFJ_BOTH_ACTIVE_LOW;
            phaseHigh = MFJ_BOTH_ACTIVE_HIGH;
            phaseApplies = true;
        } else if (state.spouse_active_in_plan) {
            phaseLow = MFJ_SPOUSE_ACTIVE_LOW;
            phaseHigh = MFJ_SPOUSE_ACTIVE_HIGH;
            phaseApplies = true;
        } else {
            phaseApplies = false;
        }
    } else if (state.filing_status === 'mfs' && state.self_active_in_plan) {
        phaseLow = MFS_PHASE_LOW;
        phaseHigh = MFS_PHASE_HIGH;
        phaseApplies = true;
    } else {
        phaseApplies = false;
    }
    let factor;
    if (!phaseApplies) factor = 1;
    else if (state.magi <= phaseLow) factor = 1;
    else if (state.magi >= phaseHigh) factor = 0;
    else factor = (phaseHigh - state.magi) / (phaseHigh - phaseLow);
    const deductibleLimit = personalLimit * factor;
    const allowedContribution = Math.min(state.intended_contribution, personalLimit, state.earned_income);
    const deductibleAmount = Math.min(allowedContribution, deductibleLimit);
    const nonDeductiblePortion = allowedContribution - deductibleAmount;
    const taxSavings = deductibleAmount * state.marginal_rate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s219.h2.result">IRA deduction</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s219.card.cap">2024 cap</div>
                    <div class="value">$${personalLimit.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s219.card.factor">Phase-out factor</div>
                    <div class="value">${(factor * 100).toFixed(0)}%</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s219.card.allowed">Allowed contribution</div>
                    <div class="value">$${allowedContribution.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s219.card.deductible">Deductible amount</div>
                    <div class="value">$${deductibleAmount.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s219.card.nondeductible">Non-deductible portion</div>
                    <div class="value">$${nonDeductiblePortion.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s219.card.savings">Tax savings</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
