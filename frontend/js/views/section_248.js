// IRC § 248 — Organizational Expenditures (Corporations).
// $5K immediate deduction in first tax year + remaining amortized over 180 months (15 yrs).
// Phaseout: $5K reduces $-for-$ when org exp > $50K → no immediate at $55K.
// Parallel § 195 startup expenditures + § 709 partnership organizational.
// Election made on first return; no separate election form required (Reg § 1.248-1).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    organizational_expenditures: 0,
    immediate_5k_deduction: 0,
    amortization_period_months: 180,
    first_tax_year: 2024,
    months_in_first_year: 0,
    is_corporation: true,
    is_partnership: false,
    s195_startup_exp: 0,
    s709_partnership_org: 0,
    type_of_expenditure: 'legal',
    incurred_before_business_began: true,
    capitalized_to_basis: false,
    election_made: true,
    accumulated_amortization: 0,
    sale_or_liquidation: false,
};

export async function renderSection248(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s248.h1.title">// § 248 ORGANIZATIONAL EXPENDITURES</span></h1>
        <p class="muted small" data-i18n="view.s248.hint.intro">
            <strong>$5K immediate deduction</strong> in first tax year + remaining amortized over <strong>180
            months</strong> (15 yrs). <strong>Phaseout:</strong> $5K reduces $-for-$ when org exp &gt; $50K
            → no immediate at $55K. <strong>Parallel:</strong> § 195 startup + § 709 partnership organizational.
            <strong>Eligible expenses:</strong> legal fees, accounting fees, state filing fees, incorporation
            costs. <strong>NOT eligible:</strong> stock issuance costs, transfer of assets, lobbying.
            <strong>Election:</strong> made on first return; no separate form (Reg § 1.248-1).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s248.h2.inputs">Inputs</h2>
            <form id="s248-form" class="inline-form">
                <label><span data-i18n="view.s248.label.expenditures">Org expenditures ($)</span>
                    <input type="number" step="1000" name="organizational_expenditures" value="${state.organizational_expenditures}"></label>
                <label><span data-i18n="view.s248.label.immediate">Immediate $5K deduction ($)</span>
                    <input type="number" step="100" name="immediate_5k_deduction" value="${state.immediate_5k_deduction}"></label>
                <label><span data-i18n="view.s248.label.amort">Amortization period (months)</span>
                    <input type="number" step="1" name="amortization_period_months" value="${state.amortization_period_months}"></label>
                <label><span data-i18n="view.s248.label.year">First tax year</span>
                    <input type="number" step="1" name="first_tax_year" value="${state.first_tax_year}"></label>
                <label><span data-i18n="view.s248.label.months">Months in first year (short period)</span>
                    <input type="number" step="1" name="months_in_first_year" value="${state.months_in_first_year}"></label>
                <label><span data-i18n="view.s248.label.corp">Corporation?</span>
                    <input type="checkbox" name="is_corporation" ${state.is_corporation ? 'checked' : ''}></label>
                <label><span data-i18n="view.s248.label.partnership">Partnership (§ 709)?</span>
                    <input type="checkbox" name="is_partnership" ${state.is_partnership ? 'checked' : ''}></label>
                <label><span data-i18n="view.s248.label.s195">§ 195 startup exp ($)</span>
                    <input type="number" step="1000" name="s195_startup_exp" value="${state.s195_startup_exp}"></label>
                <label><span data-i18n="view.s248.label.s709">§ 709 partnership org ($)</span>
                    <input type="number" step="1000" name="s709_partnership_org" value="${state.s709_partnership_org}"></label>
                <label><span data-i18n="view.s248.label.type">Type of expenditure</span>
                    <select name="type_of_expenditure">
                        <option value="legal" ${state.type_of_expenditure === 'legal' ? 'selected' : ''}>Legal fees</option>
                        <option value="accounting" ${state.type_of_expenditure === 'accounting' ? 'selected' : ''}>Accounting fees</option>
                        <option value="state_filing" ${state.type_of_expenditure === 'state_filing' ? 'selected' : ''}>State filing fees</option>
                        <option value="incorporation" ${state.type_of_expenditure === 'incorporation' ? 'selected' : ''}>Incorporation costs</option>
                        <option value="organizational_meeting" ${state.type_of_expenditure === 'organizational_meeting' ? 'selected' : ''}>Organizational meeting</option>
                        <option value="charter_drafting" ${state.type_of_expenditure === 'charter_drafting' ? 'selected' : ''}>Charter / bylaws drafting</option>
                        <option value="stock_issuance" ${state.type_of_expenditure === 'stock_issuance' ? 'selected' : ''}>Stock issuance (NOT § 248)</option>
                        <option value="capital_raising" ${state.type_of_expenditure === 'capital_raising' ? 'selected' : ''}>Capital raising (NOT § 248)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s248.label.before">Incurred before business began?</span>
                    <input type="checkbox" name="incurred_before_business_began" ${state.incurred_before_business_began ? 'checked' : ''}></label>
                <label><span data-i18n="view.s248.label.capitalized">Capitalized to basis?</span>
                    <input type="checkbox" name="capitalized_to_basis" ${state.capitalized_to_basis ? 'checked' : ''}></label>
                <label><span data-i18n="view.s248.label.election">Election made (default)?</span>
                    <input type="checkbox" name="election_made" ${state.election_made ? 'checked' : ''}></label>
                <label><span data-i18n="view.s248.label.acc_amort">Accumulated amortization ($)</span>
                    <input type="number" step="100" name="accumulated_amortization" value="${state.accumulated_amortization}"></label>
                <label><span data-i18n="view.s248.label.sale">Sale or liquidation?</span>
                    <input type="checkbox" name="sale_or_liquidation" ${state.sale_or_liquidation ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s248.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s248-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s248.h2.eligible">Eligible § 248 organizational expenditures</h2>
            <ul class="muted small">
                <li data-i18n="view.s248.eligible.legal">Legal fees: incorporation, drafting articles + bylaws, organizing meetings</li>
                <li data-i18n="view.s248.eligible.accounting">Accounting fees: setting up books + records + first audit</li>
                <li data-i18n="view.s248.eligible.state_filing">State filing fees: Articles of Incorporation, Statement of Information</li>
                <li data-i18n="view.s248.eligible.federal_filing">Federal filing fees: EIN, S-corp Form 2553, exempt org Form 1023</li>
                <li data-i18n="view.s248.eligible.organizational_meetings">Organizational meetings + temporary directors / officers compensation</li>
                <li data-i18n="view.s248.eligible.printing">Printing of stock certificates, corporate seals, minutes book</li>
                <li data-i18n="view.s248.eligible.charter">Drafting + amending corporate charter / bylaws</li>
                <li data-i18n="view.s248.eligible.registered_agent">Registered agent / resident agent setup fees</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s248.h2.not_eligible">NOT eligible (capitalized differently)</h2>
            <ul class="muted small">
                <li data-i18n="view.s248.ineligible.stock_issuance">Stock issuance costs: § 263(a) capital expenditure — reduce stock proceeds</li>
                <li data-i18n="view.s248.ineligible.transfer_assets">Transfer of assets to new corp: separate § 351 / § 1032 treatment</li>
                <li data-i18n="view.s248.ineligible.lobbying">Lobbying expenses: § 162(e) generally NOT deductible</li>
                <li data-i18n="view.s248.ineligible.startup">Startup expenses (§ 195): pre-opening operating expenses</li>
                <li data-i18n="view.s248.ineligible.reorganization">Reorganization / merger expenses: § 263 capital expenditure</li>
                <li data-i18n="view.s248.ineligible.fundraising">Investor presentations / road shows: § 263 capital</li>
                <li data-i18n="view.s248.ineligible.permits">Specific business permits (after formation): § 263 typically</li>
                <li data-i18n="view.s248.ineligible.market_studies">Market research / feasibility studies: § 195 startup</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s248.h2.s195_vs_248">§ 195 vs § 248 vs § 709 comparison</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s248.th.section">Section</th>
                    <th data-i18n="view.s248.th.scope">Scope</th>
                    <th data-i18n="view.s248.th.benefit">Benefit</th>
                </tr></thead>
                <tbody>
                    <tr><td>§ 248 Organizational (corp)</td><td>Corp formation legal / filing fees</td><td>$5K immediate + 180-mo amortization</td></tr>
                    <tr><td>§ 709 Organizational (PS)</td><td>Partnership formation</td><td>$5K immediate + 180-mo amortization (same)</td></tr>
                    <tr><td>§ 195 Startup expenses</td><td>Pre-opening operating expenses</td><td>$5K immediate + 180-mo amortization (same)</td></tr>
                    <tr><td>Combined cap</td><td>—</td><td>Each $5K = $5K + $5K + $5K total ($15K immediate)</td></tr>
                    <tr><td>Phaseout</td><td>Each section has own phaseout</td><td>Phases at $50K-$55K each</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s248.h2.election_amortization">Election + amortization mechanics</h2>
            <ul class="muted small">
                <li data-i18n="view.s248.elect.default">DEFAULT election: deemed elected if return reports per § 248 treatment</li>
                <li data-i18n="view.s248.elect.no_form">No separate election form — choice made on first return</li>
                <li data-i18n="view.s248.elect.amortization_begin">Amortization begins month business begins (active business start)</li>
                <li data-i18n="view.s248.elect.monthly">Monthly amortization: total / 180 months</li>
                <li data-i18n="view.s248.elect.short_period">Short first year: prorated based on months</li>
                <li data-i18n="view.s248.elect.election_capitalize">Election to CAPITALIZE entire amount (no current deduction): irrevocable</li>
                <li data-i18n="view.s248.elect.liquidation">Liquidation: unamortized balance currently deductible</li>
                <li data-i18n="view.s248.elect.sale_basis">Sale: unamortized balance reduces basis</li>
            </ul>
        </div>
    `;
    document.getElementById('s248-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.organizational_expenditures = Number(fd.get('organizational_expenditures')) || 0;
        state.immediate_5k_deduction = Number(fd.get('immediate_5k_deduction')) || 0;
        state.amortization_period_months = Number(fd.get('amortization_period_months')) || 0;
        state.first_tax_year = Number(fd.get('first_tax_year')) || 0;
        state.months_in_first_year = Number(fd.get('months_in_first_year')) || 0;
        state.is_corporation = !!fd.get('is_corporation');
        state.is_partnership = !!fd.get('is_partnership');
        state.s195_startup_exp = Number(fd.get('s195_startup_exp')) || 0;
        state.s709_partnership_org = Number(fd.get('s709_partnership_org')) || 0;
        state.type_of_expenditure = fd.get('type_of_expenditure');
        state.incurred_before_business_began = !!fd.get('incurred_before_business_began');
        state.capitalized_to_basis = !!fd.get('capitalized_to_basis');
        state.election_made = !!fd.get('election_made');
        state.accumulated_amortization = Number(fd.get('accumulated_amortization')) || 0;
        state.sale_or_liquidation = !!fd.get('sale_or_liquidation');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s248-output');
    if (!el) return;
    const phaseout_reduction = Math.max(0, state.organizational_expenditures - 50_000);
    const immediate = Math.max(0, 5_000 - phaseout_reduction);
    const remaining = state.organizational_expenditures - immediate;
    const monthly_amortization = remaining / state.amortization_period_months;
    const first_year_amortization = monthly_amortization * state.months_in_first_year;
    const total_first_year = immediate + first_year_amortization;
    const tax_savings = total_first_year * 0.21;
    const ineligible_types = ['stock_issuance', 'capital_raising'];
    const eligible = !ineligible_types.includes(state.type_of_expenditure);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s248.h2.result">§ 248 computation</h2>
            <div class="cards">
                <div class="card ${eligible ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s248.card.eligible">Eligible for § 248?</div>
                    <div class="value">${eligible ? esc(t('view.s248.status.yes')) : esc(t('view.s248.status.no'))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s248.card.immediate">Immediate deduction</div>
                    <div class="value">$${immediate.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${phaseout_reduction > 0 ? 'neg' : ''}">
                    <div class="label" data-i18n="view.s248.card.phaseout">Phaseout reduction</div>
                    <div class="value">$${phaseout_reduction.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s248.card.amortized">Amortized over ${state.amortization_period_months / 12} yrs</div>
                    <div class="value">$${remaining.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s248.card.monthly">Monthly amortization</div>
                    <div class="value">$${monthly_amortization.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s248.card.first_year_amort">First-year amortization</div>
                    <div class="value">$${first_year_amortization.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s248.card.total">First-year total deduction</div>
                    <div class="value">$${total_first_year.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s248.card.savings">Tax savings (21%)</div>
                    <div class="value">$${tax_savings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${phaseout_reduction >= 5_000 ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s248.phaseout_note">
                    Organizational expenditures EXCEED $55K phaseout — NO immediate $5K deduction. Entire
                    amount must be amortized over 180 months. Common for sophisticated corporate structures
                    with extensive legal, accounting, securities counsel work. Consider whether to fully
                    capitalize to basis instead (election made on first return).
                </p>
            ` : ''}
        </div>
    `;
}
