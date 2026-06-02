// IRC § 6038D — Form 8938 Information Reporting of Specified Foreign Financial Assets.
// Individual + certain entities must report foreign financial assets > threshold.
// 2024 thresholds (single/MFJ): $50K end-year or $75K anytime / $100K or $150K (US res); $200K/$300K or $400K/$600K (foreign).
// $10,000 base penalty + $10K/30-day continuation (up to $50K) for failure to report.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    filing_status: 'single',
    is_us_resident: true,
    is_living_abroad: false,
    total_foreign_assets_end_year: 0,
    total_foreign_assets_anytime: 0,
    foreign_bank_accounts: 0,
    foreign_brokerage_accounts: 0,
    foreign_pension_accounts: 0,
    foreign_stock_directly: 0,
    foreign_partnership_interests: 0,
    foreign_corp_stock: 0,
    foreign_trust_beneficial: 0,
    foreign_real_estate: 0,
    foreign_insurance_value: 0,
    safe_deposit_box: 0,
    has_filed_fbar: false,
    fbar_threshold_met: false,
    is_form_8938_required: false,
    is_form_8938_filed: false,
    days_late: 0,
    continued_failure_30day: false,
    reasonable_cause: false,
    underpayment_understatement: 0,
    s6038d_30pct_penalty: false,
    fatca_3_year_sol: false,
    cooperative_country: true,
};

export async function renderSection6038D(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6038d.h1.title">// § 6038D FORM 8938 SPECIFIED FOREIGN ASSETS</span></h1>
        <p class="muted small" data-i18n="view.s6038d.hint.intro">
            <strong>Form 8938</strong> reports "specified foreign financial assets" (SFFA) when value
            exceeds threshold. <strong>2024 thresholds:</strong> Single (US-res) $50K end / $75K
            anytime; MFJ $100K / $150K; Single living abroad $200K / $300K; MFJ abroad $400K / $600K.
            <strong>NOT the same as FBAR (FinCEN 114)</strong> — different reporting regime, but
            overlap (file BOTH if both thresholds met). <strong>Base penalty:</strong> $10,000 +
            $10K/30-day continuation (max $50K). <strong>§ 6501(c)(8) ASED extension:</strong> normal
            3-year SOL extends to 6 years OR indefinite for omitted income > $5K from SFFA.
            <strong>§ 6662(j) 40% accuracy-related penalty</strong> on undisclosed SFFA understatements.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6038d.h2.inputs">Inputs</h2>
            <form id="s6038d-form" class="inline-form">
                <label><span data-i18n="view.s6038d.label.status">Filing status</span>
                    <select name="filing_status">
                        <option value="single" ${state.filing_status === 'single' ? 'selected' : ''}>Single / HOH</option>
                        <option value="mfj" ${state.filing_status === 'mfj' ? 'selected' : ''}>MFJ</option>
                        <option value="mfs" ${state.filing_status === 'mfs' ? 'selected' : ''}>MFS</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6038d.label.us_res">US resident?</span>
                    <input type="checkbox" name="is_us_resident" ${state.is_us_resident ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6038d.label.abroad">Living abroad?</span>
                    <input type="checkbox" name="is_living_abroad" ${state.is_living_abroad ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6038d.label.end_year">Total SFFA end-year ($)</span>
                    <input type="number" step="10000" name="total_foreign_assets_end_year" value="${state.total_foreign_assets_end_year}"></label>
                <label><span data-i18n="view.s6038d.label.anytime">Total SFFA anytime ($)</span>
                    <input type="number" step="10000" name="total_foreign_assets_anytime" value="${state.total_foreign_assets_anytime}"></label>
                <label><span data-i18n="view.s6038d.label.bank">Foreign bank accounts ($)</span>
                    <input type="number" step="10000" name="foreign_bank_accounts" value="${state.foreign_bank_accounts}"></label>
                <label><span data-i18n="view.s6038d.label.brokerage">Foreign brokerage ($)</span>
                    <input type="number" step="10000" name="foreign_brokerage_accounts" value="${state.foreign_brokerage_accounts}"></label>
                <label><span data-i18n="view.s6038d.label.pension">Foreign pension ($)</span>
                    <input type="number" step="10000" name="foreign_pension_accounts" value="${state.foreign_pension_accounts}"></label>
                <label><span data-i18n="view.s6038d.label.stock_direct">Foreign stock direct ($)</span>
                    <input type="number" step="10000" name="foreign_stock_directly" value="${state.foreign_stock_directly}"></label>
                <label><span data-i18n="view.s6038d.label.partnership">Foreign partnership ($)</span>
                    <input type="number" step="10000" name="foreign_partnership_interests" value="${state.foreign_partnership_interests}"></label>
                <label><span data-i18n="view.s6038d.label.corp">Foreign corp stock ($)</span>
                    <input type="number" step="10000" name="foreign_corp_stock" value="${state.foreign_corp_stock}"></label>
                <label><span data-i18n="view.s6038d.label.trust">Foreign trust beneficial ($)</span>
                    <input type="number" step="10000" name="foreign_trust_beneficial" value="${state.foreign_trust_beneficial}"></label>
                <label><span data-i18n="view.s6038d.label.real_estate">Foreign real estate ($)</span>
                    <input type="number" step="10000" name="foreign_real_estate" value="${state.foreign_real_estate}"></label>
                <label><span data-i18n="view.s6038d.label.insurance">Foreign insurance ($)</span>
                    <input type="number" step="10000" name="foreign_insurance_value" value="${state.foreign_insurance_value}"></label>
                <label><span data-i18n="view.s6038d.label.safe_dep">Safe deposit box ($)</span>
                    <input type="number" step="10000" name="safe_deposit_box" value="${state.safe_deposit_box}"></label>
                <label><span data-i18n="view.s6038d.label.fbar">FBAR filed?</span>
                    <input type="checkbox" name="has_filed_fbar" ${state.has_filed_fbar ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6038d.label.fbar_thresh">FBAR threshold met ($10K)?</span>
                    <input type="checkbox" name="fbar_threshold_met" ${state.fbar_threshold_met ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6038d.label.f8938_req">Form 8938 required?</span>
                    <input type="checkbox" name="is_form_8938_required" ${state.is_form_8938_required ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6038d.label.f8938_filed">Form 8938 filed?</span>
                    <input type="checkbox" name="is_form_8938_filed" ${state.is_form_8938_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6038d.label.days_late">Days late</span>
                    <input type="number" step="1" name="days_late" value="${state.days_late}"></label>
                <label><span data-i18n="view.s6038d.label.continued">Continued failure 30+ days?</span>
                    <input type="checkbox" name="continued_failure_30day" ${state.continued_failure_30day ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6038d.label.reasonable">Reasonable cause?</span>
                    <input type="checkbox" name="reasonable_cause" ${state.reasonable_cause ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6038d.label.understatement">Understatement ($)</span>
                    <input type="number" step="10000" name="underpayment_understatement" value="${state.underpayment_understatement}"></label>
                <label><span data-i18n="view.s6038d.label.s6038d_30">§ 6038D 30% accuracy penalty?</span>
                    <input type="checkbox" name="s6038d_30pct_penalty" ${state.s6038d_30pct_penalty ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6038d.label.fatca_sol">FATCA 3-yr SOL extension?</span>
                    <input type="checkbox" name="fatca_3_year_sol" ${state.fatca_3_year_sol ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6038d.label.country">Cooperative country?</span>
                    <input type="checkbox" name="cooperative_country" ${state.cooperative_country ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6038d.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6038d-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6038d.h2.thresholds">2024 Form 8938 thresholds</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s6038d.tbl.status">Status</th><th data-i18n="view.s6038d.tbl.end_year">End-year value</th><th data-i18n="view.s6038d.tbl.anytime">Anytime value</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s6038d.tbl.us_single">Single/HOH (US res)</td><td>$50,000</td><td>$75,000</td></tr>
                    <tr><td data-i18n="view.s6038d.tbl.us_mfj">MFJ (US res)</td><td>$100,000</td><td>$150,000</td></tr>
                    <tr><td data-i18n="view.s6038d.tbl.us_mfs">MFS (US res)</td><td>$50,000</td><td>$75,000</td></tr>
                    <tr><td data-i18n="view.s6038d.tbl.ab_single">Single (living abroad 330+ days)</td><td>$200,000</td><td>$300,000</td></tr>
                    <tr><td data-i18n="view.s6038d.tbl.ab_mfj">MFJ (both living abroad)</td><td>$400,000</td><td>$600,000</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6038d.h2.sffa">Specified Foreign Financial Assets (SFFA)</h2>
            <ul class="muted small">
                <li data-i18n="view.s6038d.sffa.financial_account">Financial accounts maintained by foreign financial institution</li>
                <li data-i18n="view.s6038d.sffa.stock_directly">Foreign stock or securities NOT held in financial account</li>
                <li data-i18n="view.s6038d.sffa.interest_foreign_entity">Interest in foreign entity (PFIC, foreign mutual fund, etc.)</li>
                <li data-i18n="view.s6038d.sffa.financial_instrument">Financial instrument or contract with foreign counterparty</li>
                <li data-i18n="view.s6038d.sffa.life_insurance">Life insurance contracts with cash value with foreign issuer</li>
                <li data-i18n="view.s6038d.sffa.foreign_pension">Foreign pension/retirement plan (Reg § 1.6038D-2T(c)(1))</li>
                <li data-i18n="view.s6038d.sffa.foreign_trust">Beneficial interest in foreign trust</li>
                <li data-i18n="view.s6038d.sffa.NOT_real_estate">NOT: real estate, foreign currency held directly, personal property</li>
                <li data-i18n="view.s6038d.sffa.NOT_sd_box">NOT: contents of safe deposit boxes (themselves not financial accounts)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6038d.h2.penalties">Penalties + SOL extension</h2>
            <ul class="muted small">
                <li data-i18n="view.s6038d.pen.base">$10,000 base penalty for failure to file</li>
                <li data-i18n="view.s6038d.pen.continued">$10,000 per 30-day period after IRS notice (capped at $50,000)</li>
                <li data-i18n="view.s6038d.pen.s6662j">§ 6662(j) 40% accuracy-related penalty on undisclosed SFFA understatement</li>
                <li data-i18n="view.s6038d.pen.s6501c8">§ 6501(c)(8) ASED extension: indefinite for omitted income</li>
                <li data-i18n="view.s6038d.pen.s6501e_1">§ 6501(e)(1)(A)(iii) — 6-year ASED for SFFA-related $5K+ omission</li>
                <li data-i18n="view.s6038d.pen.crim_no">No criminal penalty under § 6038D itself (vs § 6038A potentially $25K)</li>
                <li data-i18n="view.s6038d.pen.fbar_overlap">FBAR penalties separate (FinCEN 114): up to $13,481 or 50% balance/willful</li>
                <li data-i18n="view.s6038d.pen.reasonable_cause">Reasonable cause defense (§ 6038D(g)): "reasonable cause + not willful neglect"</li>
                <li data-i18n="view.s6038d.pen.foreign_law">Foreign law non-disclosure does NOT establish reasonable cause</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6038d.h2.fbar_vs_8938">FBAR vs Form 8938 comparison</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s6038d.tbl.attr">Attribute</th><th>FBAR (FinCEN 114)</th><th>Form 8938 (IRS)</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s6038d.tbl.authority">Authority</td><td>31 USC § 5314 (BSA)</td><td>26 USC § 6038D (FATCA/HIRE)</td></tr>
                    <tr><td data-i18n="view.s6038d.tbl.who">Who files</td><td data-i18n="view.s6038d.tbl.us_persons">US persons (broader)</td><td data-i18n="view.s6038d.tbl.individuals_certain">Individuals + certain entities</td></tr>
                    <tr><td data-i18n="view.s6038d.tbl.threshold">Threshold</td><td>$10,000 aggregate anytime</td><td>$50K-$600K (see table)</td></tr>
                    <tr><td data-i18n="view.s6038d.tbl.deadline">Deadline</td><td>April 15 (auto-extend Oct 15)</td><td>With Form 1040</td></tr>
                    <tr><td data-i18n="view.s6038d.tbl.penalty">Civil penalty</td><td>Up to $13,481 nonwillful / 50% balance willful</td><td>$10K + $10K/30-day continuation</td></tr>
                    <tr><td data-i18n="view.s6038d.tbl.criminal">Criminal</td><td>5 years + $250K</td><td>None directly</td></tr>
                    <tr><td data-i18n="view.s6038d.tbl.assets">Assets covered</td><td data-i18n="view.s6038d.tbl.financial_accounts">Financial accounts only</td><td data-i18n="view.s6038d.tbl.broader_sffa">Broader SFFA (stock, trust interests)</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6038d.h2.entity">Entity reporting (§ 6038D(a)(2))</h2>
            <ul class="muted small">
                <li data-i18n="view.s6038d.ent.corp">Domestic corporations: ≥ 50% owned by US persons</li>
                <li data-i18n="view.s6038d.ent.partnership">Domestic partnerships: > 50% ownership by US persons</li>
                <li data-i18n="view.s6038d.ent.trust">Domestic trusts: distribution power held by US person</li>
                <li data-i18n="view.s6038d.ent.threshold_corp">Entity threshold: $50K end-year / $75K anytime (lower than individual)</li>
                <li data-i18n="view.s6038d.ent.aggregate">Aggregate beneficial ownership tested</li>
                <li data-i18n="view.s6038d.ent.cfc">CFC reporting on Form 5471 separate from 8938</li>
                <li data-i18n="view.s6038d.ent.pfic">PFIC on Form 8621 separate from 8938</li>
                <li data-i18n="view.s6038d.ent.foreign_trust">Foreign trust grantors: Form 3520 + 3520-A</li>
            </ul>
        </div>
    `;
    document.getElementById('s6038d-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.filing_status = fd.get('filing_status');
        state.is_us_resident = !!fd.get('is_us_resident');
        state.is_living_abroad = !!fd.get('is_living_abroad');
        state.total_foreign_assets_end_year = Number(fd.get('total_foreign_assets_end_year')) || 0;
        state.total_foreign_assets_anytime = Number(fd.get('total_foreign_assets_anytime')) || 0;
        state.foreign_bank_accounts = Number(fd.get('foreign_bank_accounts')) || 0;
        state.foreign_brokerage_accounts = Number(fd.get('foreign_brokerage_accounts')) || 0;
        state.foreign_pension_accounts = Number(fd.get('foreign_pension_accounts')) || 0;
        state.foreign_stock_directly = Number(fd.get('foreign_stock_directly')) || 0;
        state.foreign_partnership_interests = Number(fd.get('foreign_partnership_interests')) || 0;
        state.foreign_corp_stock = Number(fd.get('foreign_corp_stock')) || 0;
        state.foreign_trust_beneficial = Number(fd.get('foreign_trust_beneficial')) || 0;
        state.foreign_real_estate = Number(fd.get('foreign_real_estate')) || 0;
        state.foreign_insurance_value = Number(fd.get('foreign_insurance_value')) || 0;
        state.safe_deposit_box = Number(fd.get('safe_deposit_box')) || 0;
        state.has_filed_fbar = !!fd.get('has_filed_fbar');
        state.fbar_threshold_met = !!fd.get('fbar_threshold_met');
        state.is_form_8938_required = !!fd.get('is_form_8938_required');
        state.is_form_8938_filed = !!fd.get('is_form_8938_filed');
        state.days_late = Number(fd.get('days_late')) || 0;
        state.continued_failure_30day = !!fd.get('continued_failure_30day');
        state.reasonable_cause = !!fd.get('reasonable_cause');
        state.underpayment_understatement = Number(fd.get('underpayment_understatement')) || 0;
        state.s6038d_30pct_penalty = !!fd.get('s6038d_30pct_penalty');
        state.fatca_3_year_sol = !!fd.get('fatca_3_year_sol');
        state.cooperative_country = !!fd.get('cooperative_country');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6038d-output');
    if (!el) return;
    let threshold_end, threshold_anytime;
    if (state.is_living_abroad) {
        threshold_end = state.filing_status === 'mfj' ? 400_000 : 200_000;
        threshold_anytime = state.filing_status === 'mfj' ? 600_000 : 300_000;
    } else {
        threshold_end = state.filing_status === 'mfj' ? 100_000 : 50_000;
        threshold_anytime = state.filing_status === 'mfj' ? 150_000 : 75_000;
    }
    const required = state.total_foreign_assets_end_year > threshold_end || state.total_foreign_assets_anytime > threshold_anytime;
    let penalty = 0;
    if (required && !state.is_form_8938_filed && !state.reasonable_cause) {
        penalty = 10_000;
        if (state.continued_failure_30day) {
            const continuation_periods = Math.floor(state.days_late / 30);
            penalty += Math.min(continuation_periods * 10_000, 50_000);
        }
    }
    const s6662j = state.s6038d_30pct_penalty ? state.underpayment_understatement * 0.40 : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6038d.h2.result">§ 6038D Form 8938 assessment</h2>
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.s6038d.card.threshold_end">End-year threshold</div><div class="value">$${threshold_end.toLocaleString()}</div></div>
                <div class="card"><div class="label" data-i18n="view.s6038d.card.threshold_any">Anytime threshold</div><div class="value">$${threshold_anytime.toLocaleString()}</div></div>
                <div class="card ${required ? 'warn' : 'pos'}"><div class="label" data-i18n="view.s6038d.card.required">Form 8938 required?</div><div class="value">${required ? 'YES' : 'NO'}</div></div>
                <div class="card ${penalty > 0 ? 'neg' : 'pos'}"><div class="label" data-i18n="view.s6038d.card.penalty">Failure-to-file penalty</div><div class="value">$${penalty.toLocaleString()}</div></div>
                <div class="card ${s6662j > 0 ? 'neg' : 'pos'}"><div class="label" data-i18n="view.s6038d.card.s6662j">§ 6662(j) 40% accuracy</div><div class="value">$${s6662j.toLocaleString()}</div></div>
            </div>
        </div>
    `;
}
