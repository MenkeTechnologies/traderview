// IRC § 4973 — Excise Tax on Excess Contributions to Tax-Favored Accounts.
// 6% excise tax per year on excess contributions to IRAs, HSAs, Coverdell ESAs, Archer MSAs.
// CUMULATIVE: tax continues each year until excess removed + earnings withdrawn.
// Correction methods: timely removal (by tax-filing deadline) avoids penalty + earnings withdrawn.
// Late removal: 6% excise tax on excess + earnings stay in account taxable.

import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    account_type: 'traditional_ira',
    contribution_made: 0,
    contribution_limit: 7_000,
    age_50_plus: false,
    catch_up_amount: 0,
    earnings_on_excess: 0,
    timely_correction_made: false,
    timely_correction_amount: 0,
    cumulative_years_excess: 0,
    current_year_amount: 0,
    prior_year_uncorrected: 0,
    is_roth: false,
    is_inherited_ira: false,
    spousal_election: false,
    tax_year: 2024,
    correction_window_days_remaining: 0,
};

export async function renderSection4973(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s4973.h1.title">// § 4973 EXCESS IRA / HSA</span></h1>
        <p class="muted small" data-i18n="view.s4973.hint.intro">
            <strong>6% excise tax per year</strong> on excess contributions to IRAs, HSAs, Coverdell ESAs,
            Archer MSAs. <strong>CUMULATIVE:</strong> tax continues EACH year until excess removed + earnings
            withdrawn. <strong>Timely correction</strong> (by tax-filing deadline + extensions): avoids
            penalty + earnings withdrawn as ordinary income. <strong>Late correction:</strong> 6% excise on
            excess + earnings stay in account taxable later. <strong>Form 5329</strong> Part III for IRAs,
            Part VII for HSAs, etc. <strong>Limit 2024:</strong> $7K IRA + $1K catch-up; $8,300 HSA family.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s4973.h2.inputs">Inputs</h2>
            <form id="s4973-form" class="inline-form">
                <label><span data-i18n="view.s4973.label.type">Account type</span>
                    <select name="account_type">
                        <option value="traditional_ira" ${state.account_type === 'traditional_ira' ? 'selected' : ''}>Traditional IRA</option>
                        <option value="roth_ira" ${state.account_type === 'roth_ira' ? 'selected' : ''}>Roth IRA</option>
                        <option value="hsa" ${state.account_type === 'hsa' ? 'selected' : ''}>HSA</option>
                        <option value="coverdell_esa" ${state.account_type === 'coverdell_esa' ? 'selected' : ''}>Coverdell ESA ($2K limit)</option>
                        <option value="archer_msa" ${state.account_type === 'archer_msa' ? 'selected' : ''}>Archer MSA</option>
                        <option value="sep_ira" ${state.account_type === 'sep_ira' ? 'selected' : ''}>SEP-IRA (employer mistake)</option>
                        <option value="simple_ira" ${state.account_type === 'simple_ira' ? 'selected' : ''}>SIMPLE IRA</option>
                    </select>
                </label>
                <label><span data-i18n="view.s4973.label.made">Contribution made ($)</span>
                    <input type="number" step="0.01" name="contribution_made" value="${state.contribution_made}"></label>
                <label><span data-i18n="view.s4973.label.limit">Contribution limit ($)</span>
                    <input type="number" step="0.01" name="contribution_limit" value="${state.contribution_limit}"></label>
                <label><span data-i18n="view.s4973.label.age_50">Age 50+?</span>
                    <input type="checkbox" name="age_50_plus" ${state.age_50_plus ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4973.label.catch">Catch-up amount ($)</span>
                    <input type="number" step="0.01" name="catch_up_amount" value="${state.catch_up_amount}"></label>
                <label><span data-i18n="view.s4973.label.earnings">Earnings on excess ($)</span>
                    <input type="number" step="0.01" name="earnings_on_excess" value="${state.earnings_on_excess}"></label>
                <label><span data-i18n="view.s4973.label.timely">Timely correction made?</span>
                    <input type="checkbox" name="timely_correction_made" ${state.timely_correction_made ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4973.label.timely_amt">Timely correction amount ($)</span>
                    <input type="number" step="0.01" name="timely_correction_amount" value="${state.timely_correction_amount}"></label>
                <label><span data-i18n="view.s4973.label.cumulative">Cumulative years excess</span>
                    <input type="number" step="1" name="cumulative_years_excess" value="${state.cumulative_years_excess}"></label>
                <label><span data-i18n="view.s4973.label.current">Current year amount ($)</span>
                    <input type="number" step="0.01" name="current_year_amount" value="${state.current_year_amount}"></label>
                <label><span data-i18n="view.s4973.label.prior">Prior year uncorrected ($)</span>
                    <input type="number" step="0.01" name="prior_year_uncorrected" value="${state.prior_year_uncorrected}"></label>
                <label><span data-i18n="view.s4973.label.roth">Roth IRA contributor?</span>
                    <input type="checkbox" name="is_roth" ${state.is_roth ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4973.label.inherited">Inherited IRA?</span>
                    <input type="checkbox" name="is_inherited_ira" ${state.is_inherited_ira ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4973.label.spousal">Spousal election (separate)?</span>
                    <input type="checkbox" name="spousal_election" ${state.spousal_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s4973.label.year">Tax year</span>
                    <input type="number" step="1" name="tax_year" value="${state.tax_year}"></label>
                <label><span data-i18n="view.s4973.label.window">Correction window days remaining</span>
                    <input type="number" step="1" name="correction_window_days_remaining" value="${state.correction_window_days_remaining}"></label>
                <button class="primary" type="submit" data-i18n="view.s4973.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s4973-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4973.h2.limits_2024">Contribution limits 2024</h2>
            <ul class="muted small">
                <li data-i18n="view.s4973.lim.ira_trad">Traditional + Roth IRA: $7,000 (combined; age 50+ add $1,000 catch-up = $8,000)</li>
                <li data-i18n="view.s4973.lim.ira_phase_roth">Roth IRA income limit: $146-$161K single / $230-$240K MFJ phase-out</li>
                <li data-i18n="view.s4973.lim.hsa_self">HSA self-only: $4,300 (age 55+ add $1,000 = $5,300)</li>
                <li data-i18n="view.s4973.lim.hsa_family">HSA family: $8,550 (age 55+ add $1,000 = $9,550)</li>
                <li data-i18n="view.s4973.lim.coverdell">Coverdell ESA: $2,000 per beneficiary (single $95-$110K income phase-out)</li>
                <li data-i18n="view.s4973.lim.sep">SEP-IRA: lesser of 25% comp or $69,000</li>
                <li data-i18n="view.s4973.lim.simple">SIMPLE IRA: $16,000 deferral + $3,500 catch-up = $19,500</li>
                <li data-i18n="view.s4973.lim.spousal">Spousal IRA: non-working spouse may contribute up to $7K if combined earned income</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4973.h2.correction_methods">Correction methods</h2>
            <ul class="muted small">
                <li data-i18n="view.s4973.corr.timely_withdraw">Timely withdrawal: by tax-filing deadline + extensions (typically Oct 15)</li>
                <li data-i18n="view.s4973.corr.must_remove_earnings">MUST REMOVE ALSO: net income attributable to excess (NIA)</li>
                <li data-i18n="view.s4973.corr.formula">NIA = excess × (adj. earnings / total balance) — Reg § 1.408-11</li>
                <li data-i18n="view.s4973.corr.recharacterize">Recharacterize traditional ↔ Roth: by Oct 15 (post-2017 TCJA limited)</li>
                <li data-i18n="view.s4973.corr.absorb">Absorb in next year: count toward NEXT year's contribution (limited use)</li>
                <li data-i18n="view.s4973.corr.cumulative">Cumulative: 6% × YEAR after year on uncorrected excess</li>
                <li data-i18n="view.s4973.corr.deemed_distribution">Late correction: excess remains; future withdrawal = ordinary income</li>
                <li data-i18n="view.s4973.corr.form_5329">Form 5329: file with 1040 to report excess + pay excise</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4973.h2.nia">NIA (Net Income Attributable) computation</h2>
            <ol class="muted small">
                <li data-i18n="view.s4973.nia.formula">NIA = Excess × [(Adj Cl Bal - Adj Op Bal) / Adj Op Bal]</li>
                <li data-i18n="view.s4973.nia.op_bal">Adj Op Bal = Account balance BEFORE excess + other contributions during period</li>
                <li data-i18n="view.s4973.nia.cl_bal">Adj Cl Bal = Account balance AFTER excess + distributions removed</li>
                <li data-i18n="view.s4973.nia.positive">If NIA positive: earnings on excess included in income (year of distribution)</li>
                <li data-i18n="view.s4973.nia.negative">If NIA negative: loss on excess (basis adjustment only)</li>
                <li data-i18n="view.s4973.nia.daily">Daily NIA computation often required for trustee tracking</li>
                <li data-i18n="view.s4973.nia.broker">Most brokers calculate NIA automatically when excess withdrawal requested</li>
                <li data-i18n="view.s4973.nia.10pct">Premature distribution under 59.5: 10% additional § 72(t) on NIA</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s4973.h2.common_causes">Common causes of excess contributions</h2>
            <ul class="muted small">
                <li data-i18n="view.s4973.cause.income_limit">Roth IRA income limit exceeded (high earner contributions)</li>
                <li data-i18n="view.s4973.cause.dual_employer">Multiple employer 401(k) deferrals (no aggregation tracked)</li>
                <li data-i18n="view.s4973.cause.hsa_partial">HSA partial year (eligibility changes mid-year + full contribution made)</li>
                <li data-i18n="view.s4973.cause.spousal_earned">Spousal IRA without sufficient earned income</li>
                <li data-i18n="view.s4973.cause.late_recharacterization">Late recharacterization (post-TCJA elimination of Roth → Traditional)</li>
                <li data-i18n="view.s4973.cause.payroll_error">Payroll error (employer SEP / SIMPLE over-contribution)</li>
                <li data-i18n="view.s4973.cause.misunderstood_limits">Misunderstanding combined IRA + 401(k) limits (separate buckets)</li>
                <li data-i18n="view.s4973.cause.foreign_excluded">Foreign earned income excluded (no qualifying income for IRA)</li>
            </ul>
        </div>
    `;
    document.getElementById('s4973-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.account_type = fd.get('account_type');
        state.contribution_made = Number(fd.get('contribution_made')) || 0;
        state.contribution_limit = Number(fd.get('contribution_limit')) || 0;
        state.age_50_plus = !!fd.get('age_50_plus');
        state.catch_up_amount = Number(fd.get('catch_up_amount')) || 0;
        state.earnings_on_excess = Number(fd.get('earnings_on_excess')) || 0;
        state.timely_correction_made = !!fd.get('timely_correction_made');
        state.timely_correction_amount = Number(fd.get('timely_correction_amount')) || 0;
        state.cumulative_years_excess = Number(fd.get('cumulative_years_excess')) || 0;
        state.current_year_amount = Number(fd.get('current_year_amount')) || 0;
        state.prior_year_uncorrected = Number(fd.get('prior_year_uncorrected')) || 0;
        state.is_roth = !!fd.get('is_roth');
        state.is_inherited_ira = !!fd.get('is_inherited_ira');
        state.spousal_election = !!fd.get('spousal_election');
        state.tax_year = Number(fd.get('tax_year')) || 0;
        state.correction_window_days_remaining = Number(fd.get('correction_window_days_remaining')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s4973-output');
    if (!el) return;
    const max_allowed = state.contribution_limit + (state.age_50_plus ? state.catch_up_amount : 0);
    const excess_current_year = Math.max(0, state.contribution_made - max_allowed);
    const total_uncorrected = excess_current_year + state.prior_year_uncorrected - (state.timely_correction_made ? state.timely_correction_amount : 0);
    const annual_excise_tax = total_uncorrected * 0.06;
    const cumulative_excise_tax = total_uncorrected * 0.06 * Math.max(1, state.cumulative_years_excess);
    const nia_taxable = state.earnings_on_excess;
    const nia_tax = nia_taxable * 0.37;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s4973.h2.result">§ 4973 excise computation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s4973.card.limit">Max allowed</div>
                    <div class="value">$${max_allowed.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${excess_current_year > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s4973.card.excess">Excess current year</div>
                    <div class="value">$${excess_current_year.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${total_uncorrected > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s4973.card.uncorrected">Total uncorrected</div>
                    <div class="value">$${total_uncorrected.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s4973.card.annual">Annual excise (6%)</div>
                    <div class="value">$${annual_excise_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s4973.card.cumulative">Cumulative excise</div>
                    <div class="value">$${cumulative_excise_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s4973.card.nia">NIA taxable</div>
                    <div class="value">$${nia_taxable.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s4973.card.nia_tax">NIA tax (37%)</div>
                    <div class="value">$${nia_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${total_uncorrected > 0 && state.correction_window_days_remaining > 0 ? `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s4973.correct_now_note">
                    CORRECTION WINDOW OPEN — ${state.correction_window_days_remaining} days remaining. Contact
                    IRA / HSA custodian to request "Return of Excess Contribution" by tax-filing deadline
                    (typically Oct 15). MUST also remove NIA (net income attributable). Avoid 6% excise tax
                    permanently if completed timely. Form 5329 Part III to report.
                </p>
            ` : total_uncorrected > 0 ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s4973.late_note">
                    LATE CORRECTION: 6% excise tax accrues EACH YEAR until excess removed. Either: (1) take
                    distribution now (taxable income) OR (2) absorb in future year's contribution limit if
                    eligible. Note: distribution counts toward income limits + may trigger § 72(t) 10% if
                    under age 59.5. Continued failure = 6% perpetual + cumulative.
                </p>
            ` : ''}
        </div>
    `;
}
