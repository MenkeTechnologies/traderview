// IRC § 412 — Minimum Funding Standards for Defined Benefit Plans.
// DB plans must satisfy minimum funding to maintain qualified status under § 401(a).
// Computed under § 430 (single-employer) or § 431 (multi-employer).
// Funding shortfall: triggers PBGC variable-rate premium + § 4971 excise tax.
// Quarterly contributions required for at-risk plans.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    plan_assets: 0,
    funding_target: 0,
    target_normal_cost: 0,
    is_single_employer: true,
    is_multi_employer: false,
    is_at_risk: false,
    benefit_restrictions: false,
    pbgc_variable_premium: 0,
    pbgc_flat_premium: 0,
    s4971_excise_tax_pct: 10,
    accumulated_funding_deficiency: 0,
    quarterly_contributions: 0,
    contribution_deduction_limit: 0,
    fr_segment_rates: '5.0%',
    yield_curve_method: 'segment_rates',
};

export async function renderSection412(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s412.h1.title">// § 412 PENSION FUNDING</span></h1>
        <p class="muted small" data-i18n="view.s412.hint.intro">
            DB plans must satisfy <strong>minimum funding</strong> to maintain qualified status under § 401(a).
            <strong>§ 430</strong> single-employer / <strong>§ 431</strong> multi-employer. <strong>Funding
            shortfall:</strong> triggers PBGC variable-rate premium + <strong>§ 4971 excise tax (10% / 100%
            cumulative)</strong>. <strong>Quarterly contributions</strong> required if shortfall + at-risk
            status. <strong>At-risk:</strong> &lt; 80% / 70% funded → benefit restrictions + accelerated
            contributions. <strong>PBGC premium:</strong> $101 flat + 5.25% variable on shortfall (2025).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s412.h2.inputs">Inputs</h2>
            <form id="s412-form" class="inline-form">
                <label><span data-i18n="view.s412.label.assets">Plan assets ($)</span>
                    <input type="number" step="100000" name="plan_assets" value="${state.plan_assets}"></label>
                <label><span data-i18n="view.s412.label.target">Funding target ($)</span>
                    <input type="number" step="100000" name="funding_target" value="${state.funding_target}"></label>
                <label><span data-i18n="view.s412.label.normal">Target normal cost ($)</span>
                    <input type="number" step="10000" name="target_normal_cost" value="${state.target_normal_cost}"></label>
                <label><span data-i18n="view.s412.label.single">Single-employer (§ 430)?</span>
                    <input type="checkbox" name="is_single_employer" ${state.is_single_employer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s412.label.multi">Multi-employer (§ 431)?</span>
                    <input type="checkbox" name="is_multi_employer" ${state.is_multi_employer ? 'checked' : ''}></label>
                <label><span data-i18n="view.s412.label.at_risk">At-risk plan?</span>
                    <input type="checkbox" name="is_at_risk" ${state.is_at_risk ? 'checked' : ''}></label>
                <label><span data-i18n="view.s412.label.benefit">Benefit restrictions?</span>
                    <input type="checkbox" name="benefit_restrictions" ${state.benefit_restrictions ? 'checked' : ''}></label>
                <label><span data-i18n="view.s412.label.pbgc_var">PBGC variable premium ($)</span>
                    <input type="number" step="1000" name="pbgc_variable_premium" value="${state.pbgc_variable_premium}"></label>
                <label><span data-i18n="view.s412.label.pbgc_flat">PBGC flat premium ($)</span>
                    <input type="number" step="100" name="pbgc_flat_premium" value="${state.pbgc_flat_premium}"></label>
                <label><span data-i18n="view.s412.label.excise">§ 4971 excise tax %</span>
                    <input type="number" step="0.1" name="s4971_excise_tax_pct" value="${state.s4971_excise_tax_pct}"></label>
                <label><span data-i18n="view.s412.label.deficiency">Accumulated funding deficiency ($)</span>
                    <input type="number" step="10000" name="accumulated_funding_deficiency" value="${state.accumulated_funding_deficiency}"></label>
                <label><span data-i18n="view.s412.label.quarterly">Quarterly contributions ($)</span>
                    <input type="number" step="10000" name="quarterly_contributions" value="${state.quarterly_contributions}"></label>
                <label><span data-i18n="view.s412.label.limit">Contribution deduction limit ($)</span>
                    <input type="number" step="10000" name="contribution_deduction_limit" value="${state.contribution_deduction_limit}"></label>
                <label><span data-i18n="view.s412.label.segment">Segment rates (e.g. 5.0%)</span>
                    <input type="text" name="fr_segment_rates" value="${esc(state.fr_segment_rates)}"></label>
                <label><span data-i18n="view.s412.label.yield">Yield curve method</span>
                    <select name="yield_curve_method">
                        <option value="segment_rates" ${state.yield_curve_method === 'segment_rates' ? 'selected' : ''}>Segment rates</option>
                        <option value="full_yield" ${state.yield_curve_method === 'full_yield' ? 'selected' : ''}>Full yield curve</option>
                        <option value="hybrid" ${state.yield_curve_method === 'hybrid' ? 'selected' : ''}>Hybrid</option>
                    </select>
                </label>
                <button class="primary" type="submit" data-i18n="view.s412.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s412-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s412.h2.minimum_required">Minimum Required Contribution (MRC) — § 430</h2>
            <ol class="muted small">
                <li data-i18n="view.s412.mrc.target">Funding Target: PV of benefits accrued to date</li>
                <li data-i18n="view.s412.mrc.normal">Target Normal Cost: PV of benefits expected to accrue this year</li>
                <li data-i18n="view.s412.mrc.shortfall">Funding Shortfall: target - assets (if negative)</li>
                <li data-i18n="view.s412.mrc.amortization">Shortfall amortization: 7-year (15-year for funding waivers)</li>
                <li data-i18n="view.s412.mrc.mrc">MRC = normal cost + shortfall amortization + at-risk add-on</li>
                <li data-i18n="view.s412.mrc.at_risk">At-risk: additional load factor on target normal cost</li>
                <li data-i18n="view.s412.mrc.quarterly">Quarterly contributions required if shortfall exists</li>
                <li data-i18n="view.s412.mrc.due_date">Quarterly due dates: 15 days after quarter end</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s412.h2.at_risk">At-risk plan status (§ 430(i))</h2>
            <ul class="muted small">
                <li data-i18n="view.s412.atr.test">Test: Funded Target % &lt; 80% AND Funded % AT-RISK &lt; 70%</li>
                <li data-i18n="view.s412.atr.computation">At-risk computation: accelerated assumptions + load factor</li>
                <li data-i18n="view.s412.atr.benefit_restrictions">§ 436 benefit restrictions: no lump sums, no accruals if &lt; 60% funded</li>
                <li data-i18n="view.s412.atr.notice">Notice to participants: required when in at-risk + benefit restrictions</li>
                <li data-i18n="view.s412.atr.continue">Continued at-risk: 3 consecutive years → "at-risk" load factor permanently</li>
                <li data-i18n="view.s412.atr.exemption">Recent shortfall exemption: temporary relief in plan years 2014-2024 (extension acts)</li>
                <li data-i18n="view.s412.atr.transition">Transition: gradual rule application for newly at-risk plans</li>
                <li data-i18n="view.s412.atr.special">SECURE 2.0: clarifications + additional flexibility</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s412.h2.pbgc">PBGC premium structure (2025)</h2>
            <ul class="muted small">
                <li data-i18n="view.s412.pbgc.flat">Flat-rate premium: $101 per participant (single-employer DB)</li>
                <li data-i18n="view.s412.pbgc.variable">Variable-rate premium: 5.25% × unfunded vested benefits (capped at $652/participant)</li>
                <li data-i18n="view.s412.pbgc.multi">Multi-employer: $46 / per participant flat (no variable)</li>
                <li data-i18n="view.s412.pbgc.late">Late payment: 5% / month penalty + interest</li>
                <li data-i18n="view.s412.pbgc.standard_termination">Standard termination: full funding required (§ 4041)</li>
                <li data-i18n="view.s412.pbgc.distress">Distress termination: PBGC takeover</li>
                <li data-i18n="view.s412.pbgc.partition">SFA (Special Financial Assistance) Act 2021: $86B grant to underfunded multi-employer plans</li>
                <li data-i18n="view.s412.pbgc.form_5500">Form 5500 includes PBGC premium worksheet (Schedule SB)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s412.h2.s4971">§ 4971 excise tax for failure to fund</h2>
            <ul class="muted small">
                <li data-i18n="view.s412.exc.tier_1">Tier 1: 10% of accumulated funding deficiency annually</li>
                <li data-i18n="view.s412.exc.tier_2">Tier 2: 100% if not corrected within taxable period</li>
                <li data-i18n="view.s412.exc.minimum">Reportable on Form 5330 separately from Form 5500</li>
                <li data-i18n="view.s412.exc.fiduciary">Fiduciary liability: trustees may be personally liable</li>
                <li data-i18n="view.s412.exc.contribution_deductibility">Tax deduction available for contributions (subject to § 404 limit)</li>
                <li data-i18n="view.s412.exc.waiver">§ 412(c) waiver: temporary funding waiver (rare, limited duration)</li>
                <li data-i18n="view.s412.exc.cure_period">Cure period: usually 90 days to cure with payment</li>
                <li data-i18n="view.s412.exc.disqualification">Beyond cure: § 401(a) plan disqualification = all contributions taxable</li>
            </ul>
        </div>
    `;
    document.getElementById('s412-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.plan_assets = Number(fd.get('plan_assets')) || 0;
        state.funding_target = Number(fd.get('funding_target')) || 0;
        state.target_normal_cost = Number(fd.get('target_normal_cost')) || 0;
        state.is_single_employer = !!fd.get('is_single_employer');
        state.is_multi_employer = !!fd.get('is_multi_employer');
        state.is_at_risk = !!fd.get('is_at_risk');
        state.benefit_restrictions = !!fd.get('benefit_restrictions');
        state.pbgc_variable_premium = Number(fd.get('pbgc_variable_premium')) || 0;
        state.pbgc_flat_premium = Number(fd.get('pbgc_flat_premium')) || 0;
        state.s4971_excise_tax_pct = Number(fd.get('s4971_excise_tax_pct')) || 0;
        state.accumulated_funding_deficiency = Number(fd.get('accumulated_funding_deficiency')) || 0;
        state.quarterly_contributions = Number(fd.get('quarterly_contributions')) || 0;
        state.contribution_deduction_limit = Number(fd.get('contribution_deduction_limit')) || 0;
        state.fr_segment_rates = fd.get('fr_segment_rates');
        state.yield_curve_method = fd.get('yield_curve_method');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s412-output');
    if (!el) return;
    const funded_pct = state.funding_target > 0 ? (state.plan_assets / state.funding_target * 100) : 100;
    const shortfall = Math.max(0, state.funding_target - state.plan_assets);
    const shortfall_amortization_7yr = shortfall / 7;
    const at_risk_load = state.is_at_risk ? state.target_normal_cost * 0.40 : 0;
    const mrc = state.target_normal_cost + shortfall_amortization_7yr + at_risk_load;
    const pbgc_total = state.pbgc_flat_premium + state.pbgc_variable_premium;
    const excise_tax = state.accumulated_funding_deficiency * (state.s4971_excise_tax_pct / 100);
    const total_obligation = mrc + pbgc_total + excise_tax;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s412.h2.result">§ 412 funding computation</h2>
            <div class="cards">
                <div class="card ${funded_pct >= 100 ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s412.card.funded_pct">Funded %</div>
                    <div class="value">${funded_pct.toFixed(1)}%</div>
                </div>
                <div class="card ${shortfall > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s412.card.shortfall">Funding shortfall</div>
                    <div class="value">$${shortfall.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s412.card.amortization">7-yr amortization</div>
                    <div class="value">$${shortfall_amortization_7yr.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s412.card.mrc">Minimum Required Contribution</div>
                    <div class="value">$${mrc.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s412.card.pbgc">PBGC premium total</div>
                    <div class="value">$${pbgc_total.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s412.card.excise">§ 4971 excise tax</div>
                    <div class="value">$${excise_tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s412.card.total">Total funding obligation</div>
                    <div class="value">$${total_obligation.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.is_at_risk ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s412.at_risk_note">
                    AT-RISK STATUS: § 430 load factor adds 40% to target normal cost. § 436 benefit restrictions:
                    no lump sums (&lt; 80%); no future accruals (&lt; 60%); 50% lump sum available at 60-80%.
                    Notify participants. Address shortfall via: increase contributions, plan freezes, benefit
                    formula reduction (no clawback of accrued).
                </p>
            ` : ''}
        </div>
    `;
}
