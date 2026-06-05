// IRC § 457 — Deferred Compensation for State/Local Gov't + Tax-Exempt Orgs.
// § 457(b) ELIGIBLE plan: $23,500 (2025) limit; SAME as § 401(k) but separate buckets — STACKABLE.
// § 457(f) INELIGIBLE plan: top-hat for nonprofits + tax-exempts; no contribution limit but vesting rules.
// § 457(b) gov't: rollover to IRA / 401(k) allowed; no early-withdrawal § 72(t) penalty.
// § 457(b) tax-exempt: NO rollover; subject to creditor claims.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const S457B_LIMIT_2025 = 23_500;
const CATCH_UP_50 = 7_500;
const CATCH_UP_60_63 = 11_250;
const TRIPLE_CATCH_UP_MAX = 2 * S457B_LIMIT_2025;

let state = {
    employer_type: 'governmental',
    annual_deferral: 0,
    employee_age: 0,
    age_60_63_special: false,
    triple_catch_up_last_3yrs: false,
    last_3_underutilized: 0,
    is_separated: false,
    is_457f_plan: false,
    s457f_vest_year: 0,
    s457f_value_at_vest: 0,
    s401k_concurrent: 0,
    s403b_concurrent: 0,
    rollover_to_ira: false,
};

export async function renderSection457(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s457.h1.title">// § 457 GOVT / NONPROFIT DEFER COMP</span></h1>
        <p class="muted small" data-i18n="view.s457.hint.intro">
            <strong>§ 457(b) ELIGIBLE plan:</strong> $23,500 (2025) limit — SAME as § 401(k) but SEPARATE buckets,
            STACKABLE for combined $47K+ for nonprofit / gov't. <strong>Triple catch-up</strong> last 3-yr before
            normal retirement: up to 2× annual limit. <strong>§ 457(f) INELIGIBLE:</strong> top-hat for nonprofits;
            no $$$ limit but VESTING (substantial risk of forfeiture) required for deferral. <strong>Gov't 457(b):</strong>
            rollover allowed; no § 72(t) early penalty. <strong>Tax-exempt 457(b):</strong> NO rollover;
            subject to CREDITORS of employer.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s457.h2.inputs">Inputs</h2>
            <form id="s457-form" class="inline-form">
                <label><span data-i18n="view.s457.label.employer">Employer type</span>
                    <select name="employer_type">
                        <option value="governmental" ${state.employer_type === 'governmental' ? 'selected' : ''}>Governmental (state/local/tribal)</option>
                        <option value="tax_exempt" ${state.employer_type === 'tax_exempt' ? 'selected' : ''}>Tax-exempt (501(c))</option>
                        <option value="federal" ${state.employer_type === 'federal' ? 'selected' : ''}>Federal (TSP separate)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s457.label.annual">Annual deferral ($)</span>
                    <input type="number" step="0.01" name="annual_deferral" value="${state.annual_deferral}"></label>
                <label><span data-i18n="view.s457.label.age">Employee age</span>
                    <input type="number" step="1" name="employee_age" value="${state.employee_age}"></label>
                <label><span data-i18n="view.s457.label.60_63">Age 60-63 special catch-up?</span>
                    <input type="checkbox" name="age_60_63_special" ${state.age_60_63_special ? 'checked' : ''}></label>
                <label><span data-i18n="view.s457.label.triple">Triple catch-up last 3 yrs?</span>
                    <input type="checkbox" name="triple_catch_up_last_3yrs" ${state.triple_catch_up_last_3yrs ? 'checked' : ''}></label>
                <label><span data-i18n="view.s457.label.underutilized">Last 3-yr underutilized ($)</span>
                    <input type="number" step="0.01" name="last_3_underutilized" value="${state.last_3_underutilized}"></label>
                <label><span data-i18n="view.s457.label.separated">Separated from service?</span>
                    <input type="checkbox" name="is_separated" ${state.is_separated ? 'checked' : ''}></label>
                <label><span data-i18n="view.s457.label.457f">§ 457(f) ineligible plan?</span>
                    <input type="checkbox" name="is_457f_plan" ${state.is_457f_plan ? 'checked' : ''}></label>
                <label><span data-i18n="view.s457.label.vest">§ 457(f) vest year</span>
                    <input type="number" step="1" name="s457f_vest_year" value="${state.s457f_vest_year}"></label>
                <label><span data-i18n="view.s457.label.value_vest">§ 457(f) value at vest ($)</span>
                    <input type="number" step="0.01" name="s457f_value_at_vest" value="${state.s457f_value_at_vest}"></label>
                <label><span data-i18n="view.s457.label.401k">Concurrent § 401(k) deferral ($)</span>
                    <input type="number" step="0.01" name="s401k_concurrent" value="${state.s401k_concurrent}"></label>
                <label><span data-i18n="view.s457.label.403b">Concurrent § 403(b) deferral ($)</span>
                    <input type="number" step="0.01" name="s403b_concurrent" value="${state.s403b_concurrent}"></label>
                <label><span data-i18n="view.s457.label.rollover">Roll over to IRA / 401(k)?</span>
                    <input type="checkbox" name="rollover_to_ira" ${state.rollover_to_ira ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s457.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s457-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s457.h2.b_vs_f">§ 457(b) vs § 457(f)</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s457.th.feature">Feature</th>
                    <th data-i18n="view.s457.th.b">§ 457(b) eligible</th>
                    <th data-i18n="view.s457.th.f">§ 457(f) ineligible</th>
                </tr></thead>
                <tbody>
                    <tr><td>Contribution limit</td><td>$23,500 (2025) + catch-up</td><td>UNLIMITED (no statutory cap)</td></tr>
                    <tr><td>Vesting required</td><td>NO</td><td>YES — substantial risk of forfeiture</td></tr>
                    <tr><td>Subject to creditors</td><td>Gov't: NO (trust); Tax-exempt: YES</td><td>YES (always — unfunded liability)</td></tr>
                    <tr><td>Rollover allowed</td><td>Gov't YES; Tax-exempt NO</td><td>NO</td></tr>
                    <tr><td>Eligibility</td><td>All employees</td><td>"Top-hat group" — highly comp / mgmt</td></tr>
                    <tr><td>§ 409A</td><td>EXEMPT</td><td>SUBJECT to § 409A on top</td></tr>
                    <tr><td>Tax at separation</td><td>Income tax + ordinary</td><td>Income tax at VEST not separation</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s457.h2.stacking">Stacking with § 401(k) / § 403(b)</h2>
            <ul class="muted small">
                <li data-i18n="view.s457.stack.separate">§ 457(b) is SEPARATE bucket from § 401(k) / § 403(b)</li>
                <li data-i18n="view.s457.stack.dual">Dual employee (gov't day job + nonprofit consulting): can defer FULL limit to BOTH</li>
                <li data-i18n="view.s457.stack.same_employer">Same employer: § 457(b) deferral SEPARATE from § 401(k) — both at full $23,500</li>
                <li data-i18n="view.s457.stack.example">Example: $23,500 × 2 = $47,000 deferral if both available</li>
                <li data-i18n="view.s457.stack.no_match">§ 457(b) typically NO employer match (gov't 401(k) replaces)</li>
                <li data-i18n="view.s457.stack.catch_up">Catch-up available SEPARATELY on each — additional $7,500 (50+) or $11,250 (60-63)</li>
                <li data-i18n="view.s457.stack.triple_only_457">Triple catch-up: ONLY in § 457(b) — not 401(k) / 403(b)</li>
                <li data-i18n="view.s457.stack.no_double_catch">Triple OR age-50 catch-up — NOT BOTH in same year</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s457.h2.triple">Triple catch-up rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s457.tcu.window">Last 3 years before normal retirement age (as defined in plan)</li>
                <li data-i18n="view.s457.tcu.calculation">Limit = MIN(annual limit, deferral underutilized in prior years)</li>
                <li data-i18n="view.s457.tcu.example">Example: 2025 limit $23,500; if underutilized $20K total in prior years → catch-up = $20K</li>
                <li data-i18n="view.s457.tcu.max">Max total deferral = $23,500 + $23,500 catch-up = $47,000</li>
                <li data-i18n="view.s457.tcu.no_combine">Cannot combine with age 50+ catch-up in same year</li>
                <li data-i18n="view.s457.tcu.gov_only">Available in BOTH gov't and tax-exempt § 457(b)</li>
                <li data-i18n="view.s457.tcu.normal_retirement">Normal retirement age: per plan (typically 65, can be earlier)</li>
                <li data-i18n="view.s457.tcu.documentation">Document underutilized: track all prior years to plan year</li>
            </ul>
        </div>
    `;
    document.getElementById('s457-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.employer_type = fd.get('employer_type');
        state.annual_deferral = Number(fd.get('annual_deferral')) || 0;
        state.employee_age = Number(fd.get('employee_age')) || 0;
        state.age_60_63_special = !!fd.get('age_60_63_special');
        state.triple_catch_up_last_3yrs = !!fd.get('triple_catch_up_last_3yrs');
        state.last_3_underutilized = Number(fd.get('last_3_underutilized')) || 0;
        state.is_separated = !!fd.get('is_separated');
        state.is_457f_plan = !!fd.get('is_457f_plan');
        state.s457f_vest_year = Number(fd.get('s457f_vest_year')) || 0;
        state.s457f_value_at_vest = Number(fd.get('s457f_value_at_vest')) || 0;
        state.s401k_concurrent = Number(fd.get('s401k_concurrent')) || 0;
        state.s403b_concurrent = Number(fd.get('s403b_concurrent')) || 0;
        state.rollover_to_ira = !!fd.get('rollover_to_ira');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s457-output');
    if (!el) return;
    let limit = S457B_LIMIT_2025;
    let catchUp = 0;
    if (state.triple_catch_up_last_3yrs) {
        catchUp = Math.min(state.last_3_underutilized, S457B_LIMIT_2025);
    } else if (state.age_60_63_special && state.employee_age >= 60 && state.employee_age <= 63) {
        catchUp = CATCH_UP_60_63;
    } else if (state.employee_age >= 50) {
        catchUp = CATCH_UP_50;
    }
    const maxAnnual = limit + catchUp;
    const isOver = state.annual_deferral > maxAnnual;
    const allowedDeferral = Math.min(state.annual_deferral, maxAnnual);
    const taxSavings = allowedDeferral * 0.37;
    const totalStacked = allowedDeferral + state.s401k_concurrent + state.s403b_concurrent;
    const s457fTaxAtVest = state.is_457f_plan ? state.s457f_value_at_vest * 0.37 : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s457.h2.result">§ 457 outcome</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s457.card.limit">Base limit (2025)</div>
                    <div class="value">$${limit.toLocaleString()}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s457.card.catchup">Catch-up</div>
                    <div class="value">$${catchUp.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s457.card.max">Max annual</div>
                    <div class="value">$${maxAnnual.toLocaleString()}</div>
                </div>
                <div class="card ${isOver ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s457.card.over">Over limit?</div>
                    <div class="value">${isOver ? esc(t('view.s457.status.yes')) : esc(t('view.s457.status.no'))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s457.card.allowed">Allowed deferral</div>
                    <div class="value">$${allowedDeferral.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s457.card.savings">Tax savings (37%)</div>
                    <div class="value">$${taxSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s457.card.stacked">Total stacked deferral</div>
                    <div class="value">$${totalStacked.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s457.card.f_tax">§ 457(f) tax at vest</div>
                    <div class="value">$${s457fTaxAtVest.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.is_457f_plan ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s457.f_note">
                    § 457(f) ineligible plan: tax at VEST not separation. Plan vesting carefully — accelerated
                    vesting triggers tax even if no distribution. Also subject to § 409A (additional 20% + interest
                    if violation). Common in nonprofit C-suite (university presidents, hospital CEOs, 501(c)(3)).
                </p>
            ` : ''}
        </div>
    `;
}
