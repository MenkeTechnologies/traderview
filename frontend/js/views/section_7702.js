// IRC § 7702 — Life Insurance Contract Definition.
// Defines what qualifies as life insurance for tax purposes (cash value buildup tax-deferred).
// Must satisfy: (1) Cash Value Accumulation Test (CVAT) OR (2) Guideline Premium / Cash Value Corridor Test.
// Failure: cash value treated as ordinary income each year (tax-deferred status lost).
// § 7702A — Modified Endowment Contract (MEC): separate test prevents abuse via large premiums.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    cash_value: 0,
    death_benefit: 0,
    annual_premium: 0,
    insured_age: 0,
    is_term_insurance: false,
    is_universal_life: false,
    is_variable_universal_life: false,
    is_whole_life: false,
    test_used: 'gpt_cvct',
    cash_value_accumulation_actual: 0,
    cvat_cash_value_limit: 0,
    guideline_premium_limit: 0,
    cash_value_corridor_pct: 0,
    is_mec: false,
    seven_pay_test_premiums: 0,
    surrender_value: 0,
    pre_2021_rules: false,
    interest_rates_floor_2_pct: false,
};

export async function renderSection7702(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s7702.h1.title">// § 7702 LIFE INSURANCE CONTRACT</span></h1>
        <p class="muted small" data-i18n="view.s7702.hint.intro">
            Defines what qualifies as life insurance for tax purposes (<strong>cash value buildup
            tax-deferred</strong>). Must satisfy: (1) <strong>CVAT</strong> (Cash Value Accumulation Test)
            OR (2) <strong>GPT</strong> (Guideline Premium Test) + Cash Value Corridor. <strong>Failure:</strong>
            cash value taxed as ordinary income each year (tax-deferred status lost). <strong>§ 7702A — MEC</strong>
            (Modified Endowment Contract): separate test prevents abuse via large premiums. <strong>2021 update:</strong>
            interest rate floor lowered from 4% to 2% (Consolidated Appropriations Act 2021).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s7702.h2.inputs">Inputs</h2>
            <form id="s7702-form" class="inline-form">
                <label><span data-i18n="view.s7702.label.cash">Current cash value ($)</span>
                    <input type="number" step="0.01" name="cash_value" value="${state.cash_value}"></label>
                <label><span data-i18n="view.s7702.label.death">Death benefit ($)</span>
                    <input type="number" step="0.01" name="death_benefit" value="${state.death_benefit}"></label>
                <label><span data-i18n="view.s7702.label.premium">Annual premium ($)</span>
                    <input type="number" step="0.01" name="annual_premium" value="${state.annual_premium}"></label>
                <label><span data-i18n="view.s7702.label.age">Insured age</span>
                    <input type="number" step="1" name="insured_age" value="${state.insured_age}"></label>
                <label><span data-i18n="view.s7702.label.term">Term insurance?</span>
                    <input type="checkbox" name="is_term_insurance" ${state.is_term_insurance ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7702.label.ul">Universal life?</span>
                    <input type="checkbox" name="is_universal_life" ${state.is_universal_life ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7702.label.vul">Variable universal life?</span>
                    <input type="checkbox" name="is_variable_universal_life" ${state.is_variable_universal_life ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7702.label.whole">Whole life?</span>
                    <input type="checkbox" name="is_whole_life" ${state.is_whole_life ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7702.label.test">Test used</span>
                    <select name="test_used">
                        <option value="cvat" ${state.test_used === 'cvat' ? 'selected' : ''}>CVAT only</option>
                        <option value="gpt_cvct" ${state.test_used === 'gpt_cvct' ? 'selected' : ''}>GPT + Cash Value Corridor</option>
                        <option value="failed" ${state.test_used === 'failed' ? 'selected' : ''}>Failed (lost qualification)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s7702.label.cvat_actual">CVAT cash value actual ($)</span>
                    <input type="number" step="0.01" name="cash_value_accumulation_actual" value="${state.cash_value_accumulation_actual}"></label>
                <label><span data-i18n="view.s7702.label.cvat_limit">CVAT cash value limit ($)</span>
                    <input type="number" step="0.01" name="cvat_cash_value_limit" value="${state.cvat_cash_value_limit}"></label>
                <label><span data-i18n="view.s7702.label.gpt">Guideline premium limit ($)</span>
                    <input type="number" step="0.01" name="guideline_premium_limit" value="${state.guideline_premium_limit}"></label>
                <label><span data-i18n="view.s7702.label.corridor">Cash value corridor %</span>
                    <input type="number" step="0.01" name="cash_value_corridor_pct" value="${state.cash_value_corridor_pct}"></label>
                <label><span data-i18n="view.s7702.label.mec">Modified Endowment Contract (MEC)?</span>
                    <input type="checkbox" name="is_mec" ${state.is_mec ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7702.label.7pay">7-pay test cumulative premiums ($)</span>
                    <input type="number" step="0.01" name="seven_pay_test_premiums" value="${state.seven_pay_test_premiums}"></label>
                <label><span data-i18n="view.s7702.label.surrender">Surrender value ($)</span>
                    <input type="number" step="0.01" name="surrender_value" value="${state.surrender_value}"></label>
                <label><span data-i18n="view.s7702.label.pre_2021">Pre-2021 rules (4% floor)?</span>
                    <input type="checkbox" name="pre_2021_rules" ${state.pre_2021_rules ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7702.label.floor_2">Interest rate floor at 2%?</span>
                    <input type="checkbox" name="interest_rates_floor_2_pct" ${state.interest_rates_floor_2_pct ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s7702.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s7702-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7702.h2.cvat">CVAT — Cash Value Accumulation Test</h2>
            <ul class="muted small">
                <li data-i18n="view.s7702.cvat.principle">Cash value at any time MAY NOT exceed net single premium for death benefit at that point</li>
                <li data-i18n="view.s7702.cvat.formula">Net single premium based on minimum mortality (CSO tables) + minimum interest rate</li>
                <li data-i18n="view.s7702.cvat.interest_rate">Minimum interest rate: 4% (pre-2021), 2% / 60% AFR (post-2020)</li>
                <li data-i18n="view.s7702.cvat.mortality">Mortality: 2001 Commissioners' Standard Ordinary (CSO) — replaced 1980 CSO</li>
                <li data-i18n="view.s7702.cvat.advantage">CVAT advantage: more cash value buildup permitted (less restrictive)</li>
                <li data-i18n="view.s7702.cvat.disadvantage">CVAT disadvantage: must constantly increase death benefit or contract becomes non-qualifying</li>
                <li data-i18n="view.s7702.cvat.simplification">Calculation: per $1,000 of death benefit × insured's age (CSO table)</li>
                <li data-i18n="view.s7702.cvat.common">Common for: whole life, single premium policies</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7702.h2.gpt">GPT — Guideline Premium Test</h2>
            <ul class="muted small">
                <li data-i18n="view.s7702.gpt.test">Sum of premiums paid cannot exceed guideline single + guideline level premium limits</li>
                <li data-i18n="view.s7702.gpt.single">Guideline single premium: actuarial computation based on CSO + interest rate</li>
                <li data-i18n="view.s7702.gpt.level">Guideline level premium: amortized over insured's lifetime</li>
                <li data-i18n="view.s7702.gpt.cvct">PLUS Cash Value Corridor Test: cash value ≤ % of death benefit</li>
                <li data-i18n="view.s7702.gpt.corridor_ages">Corridor %: 250% at age 40, declines to 100% at age 95</li>
                <li data-i18n="view.s7702.gpt.common">Common for: universal life, variable life</li>
                <li data-i18n="view.s7702.gpt.advantage">GPT advantage: lower required death benefit relative to cash value</li>
                <li data-i18n="view.s7702.gpt.disadvantage">GPT disadvantage: less premium flexibility / lower cash accumulation</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7702.h2.changes_2021">2021 § 7702 changes (CAA 2021)</h2>
            <ul class="muted small">
                <li data-i18n="view.s7702.c.lower_floor">Interest rate floor lowered from 4% to 2% (60% of long-term AFR)</li>
                <li data-i18n="view.s7702.c.first_change">First major § 7702 update since 1984</li>
                <li data-i18n="view.s7702.c.purpose">Purpose: reflect modern low-interest environment</li>
                <li data-i18n="view.s7702.c.effect">Effect: ALLOWS HIGHER CASH VALUE relative to death benefit</li>
                <li data-i18n="view.s7702.c.products">Products developed: high cash value whole life designed under new rules</li>
                <li data-i18n="view.s7702.c.transition">Transition: existing policies grandfathered; new policies use new rules</li>
                <li data-i18n="view.s7702.c.industry">Industry impact: more flexible accumulation products available</li>
                <li data-i18n="view.s7702.c.future_review">Future: rates re-evaluated periodically based on AFR (no Congressional action)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7702.h2.consequences">Failure consequences + planning</h2>
            <ul class="muted small">
                <li data-i18n="view.s7702.fail.taxable">Failure: cash value increase taxed as ORDINARY INCOME each year</li>
                <li data-i18n="view.s7702.fail.no_death_benefit">Death benefit may still be tax-free under § 101</li>
                <li data-i18n="view.s7702.fail.no_loans_tax_free">Loans against cash value: now taxable (instead of tax-free)</li>
                <li data-i18n="view.s7702.fail.cure">Cure: adjust death benefit OR refund excess premiums</li>
                <li data-i18n="view.s7702.fail.mec">MEC § 7702A: distributions taxed FIFO income-first + 10% penalty &lt; age 59.5</li>
                <li data-i18n="view.s7702.fail.life_settlement">Life settlement: special § 1234A character rules</li>
                <li data-i18n="view.s7702.fail.investor_owned_life">Investor-owned life insurance (IOLI): § 264 disallows interest deduction</li>
                <li data-i18n="view.s7702.fail.charitable">Charitable: contribution of policy may be subject to § 170 limitations</li>
            </ul>
        </div>
    `;
    document.getElementById('s7702-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.cash_value = Number(fd.get('cash_value')) || 0;
        state.death_benefit = Number(fd.get('death_benefit')) || 0;
        state.annual_premium = Number(fd.get('annual_premium')) || 0;
        state.insured_age = Number(fd.get('insured_age')) || 0;
        state.is_term_insurance = !!fd.get('is_term_insurance');
        state.is_universal_life = !!fd.get('is_universal_life');
        state.is_variable_universal_life = !!fd.get('is_variable_universal_life');
        state.is_whole_life = !!fd.get('is_whole_life');
        state.test_used = fd.get('test_used');
        state.cash_value_accumulation_actual = Number(fd.get('cash_value_accumulation_actual')) || 0;
        state.cvat_cash_value_limit = Number(fd.get('cvat_cash_value_limit')) || 0;
        state.guideline_premium_limit = Number(fd.get('guideline_premium_limit')) || 0;
        state.cash_value_corridor_pct = Number(fd.get('cash_value_corridor_pct')) || 0;
        state.is_mec = !!fd.get('is_mec');
        state.seven_pay_test_premiums = Number(fd.get('seven_pay_test_premiums')) || 0;
        state.surrender_value = Number(fd.get('surrender_value')) || 0;
        state.pre_2021_rules = !!fd.get('pre_2021_rules');
        state.interest_rates_floor_2_pct = !!fd.get('interest_rates_floor_2_pct');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s7702-output');
    if (!el) return;
    const cvat_passes = state.test_used === 'cvat' && state.cash_value_accumulation_actual <= state.cvat_cash_value_limit;
    const cv_corridor_pct = state.death_benefit > 0 ? (state.cash_value / state.death_benefit * 100) : 0;
    const corridor_ok = cv_corridor_pct <= state.cash_value_corridor_pct;
    const gpt_passes = state.test_used === 'gpt_cvct' && state.annual_premium <= state.guideline_premium_limit && corridor_ok;
    const qualifies = cvat_passes || gpt_passes;
    const inside_buildup = state.cash_value - state.annual_premium;
    const tax_savings_on_buildup = inside_buildup > 0 ? inside_buildup * 0.37 : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s7702.h2.result">§ 7702 qualification check</h2>
            <div class="cards">
                <div class="card ${qualifies ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s7702.card.qualifies">Qualifies?</div>
                    <div class="value">${qualifies ? esc(t('view.s7702.status.yes')) : esc(t('view.s7702.status.no'))}</div>
                </div>
                <div class="card ${state.is_mec ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s7702.card.mec">MEC status?</div>
                    <div class="value">${state.is_mec ? esc(t('view.s7702.status.yes')) : esc(t('view.s7702.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s7702.card.corridor">CV / Death benefit %</div>
                    <div class="value">${cv_corridor_pct.toFixed(2)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s7702.card.required_corridor">Required corridor %</div>
                    <div class="value">${state.cash_value_corridor_pct.toFixed(0)}%</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s7702.card.inside_buildup">Inside buildup (estimate)</div>
                    <div class="value">$${inside_buildup.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s7702.card.savings">Tax savings on buildup (37%)</div>
                    <div class="value">$${tax_savings_on_buildup.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${!qualifies ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s7702.fail_note">
                    POLICY FAILED § 7702. Cash value buildup TAXED as ordinary income each year. Loans against
                    cash value: TAXABLE. Death benefit may still be tax-free under § 101(a). CURE: increase
                    death benefit (raise corridor) OR refund excess premiums + interest. Insurance company
                    should self-monitor + warn before failure. Consult tax + insurance counsel before policy
                    redesign.
                </p>
            ` : ''}
        </div>
    `;
}
