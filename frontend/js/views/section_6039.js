// IRC § 6039 — ISO/ESPP Information Reporting (Forms 3921 + 3922).
// Form 3921: ISO exercise (employee gets it, copy to IRS).
// Form 3922: ESPP transfer of legal title to ESPP shares.
// Employer must furnish to employee by Jan 31 + IRS copy by Feb 28 paper / Mar 31 e-file.

import { esc } from '../util.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    form_type: '3921',
    is_iso: false,
    is_espp: false,
    shares_exercised: 0,
    fmv_at_exercise: 0,
    fmv_at_grant: 0,
    exercise_price: 0,
    grant_date: '',
    exercise_date: '',
    espp_discount_pct: 15,
    is_qualifying_disposition: true,
    holding_period_satisfied: true,
    days_held: 0,
    is_disqualifying: false,
    amt_adjustment: 0,
    s421_b_gain: 0,
    forms_filed_count: 0,
    days_late: 0,
    intentional_disregard: false,
};

export async function renderSection6039(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6039.h1.title">// § 6039 ISO/ESPP INFORMATION REPORTING</span></h1>
        <p class="muted small" data-i18n="view.s6039.hint.intro">
            <strong>Form 3921</strong> (ISO exercise) + <strong>Form 3922</strong> (ESPP transfer of
            title). Employer must furnish copy to employee by <strong>Jan 31</strong> + IRS copy by
            <strong>Feb 28</strong> (paper) / <strong>Mar 31</strong> (e-file). <strong>ISO § 422:</strong>
            grant + 2-year + exercise + 1-year holding = qualifying disposition (cap gain only).
            <strong>ESPP § 423:</strong> 15% max discount, 2-year + 1-year holding for qualifying.
            <strong>§ 6721/6722 penalties:</strong> $310/form (2024) up to $3.78M for failure to
            file + furnish; $630/form intentional. <strong>AMT trap:</strong> ISO spread is § 56(b)(3)
            AMT preference item even when no regular tax.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6039.h2.inputs">Inputs</h2>
            <form id="s6039-form" class="inline-form">
                <label><span data-i18n="view.s6039.label.form">Form type</span>
                    <select name="form_type">
                        <option value="3921" ${state.form_type === '3921' ? 'selected' : ''}>Form 3921 (ISO exercise)</option>
                        <option value="3922" ${state.form_type === '3922' ? 'selected' : ''}>Form 3922 (ESPP transfer)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6039.label.iso">Is ISO?</span>
                    <input type="checkbox" name="is_iso" ${state.is_iso ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6039.label.espp">Is ESPP?</span>
                    <input type="checkbox" name="is_espp" ${state.is_espp ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6039.label.shares">Shares exercised</span>
                    <input type="number" step="1" name="shares_exercised" value="${state.shares_exercised}"></label>
                <label><span data-i18n="view.s6039.label.fmv_ex">FMV at exercise ($)</span>
                    <input type="number" step="0.01" name="fmv_at_exercise" value="${state.fmv_at_exercise}"></label>
                <label><span data-i18n="view.s6039.label.fmv_gr">FMV at grant ($)</span>
                    <input type="number" step="0.01" name="fmv_at_grant" value="${state.fmv_at_grant}"></label>
                <label><span data-i18n="view.s6039.label.price">Exercise price ($)</span>
                    <input type="number" step="0.01" name="exercise_price" value="${state.exercise_price}"></label>
                <label><span data-i18n="view.s6039.label.grant_date">Grant date</span>
                    <input type="date" name="grant_date" value="${state.grant_date}"></label>
                <label><span data-i18n="view.s6039.label.exercise_date">Exercise date</span>
                    <input type="date" name="exercise_date" value="${state.exercise_date}"></label>
                <label><span data-i18n="view.s6039.label.espp_discount">ESPP discount %</span>
                    <input type="number" step="0.1" name="espp_discount_pct" value="${state.espp_discount_pct}"></label>
                <label><span data-i18n="view.s6039.label.qualifying">Qualifying disposition?</span>
                    <input type="checkbox" name="is_qualifying_disposition" ${state.is_qualifying_disposition ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6039.label.holding">Holding period satisfied?</span>
                    <input type="checkbox" name="holding_period_satisfied" ${state.holding_period_satisfied ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6039.label.days">Days held</span>
                    <input type="number" step="1" name="days_held" value="${state.days_held}"></label>
                <label><span data-i18n="view.s6039.label.disqual">Disqualifying disposition?</span>
                    <input type="checkbox" name="is_disqualifying" ${state.is_disqualifying ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6039.label.amt">AMT adjustment ($)</span>
                    <input type="number" step="100" name="amt_adjustment" value="${state.amt_adjustment}"></label>
                <label><span data-i18n="view.s6039.label.s421b">§ 421(b) gain ($)</span>
                    <input type="number" step="100" name="s421_b_gain" value="${state.s421_b_gain}"></label>
                <label><span data-i18n="view.s6039.label.count">Forms filed count</span>
                    <input type="number" step="1" name="forms_filed_count" value="${state.forms_filed_count}"></label>
                <label><span data-i18n="view.s6039.label.late">Days late</span>
                    <input type="number" step="1" name="days_late" value="${state.days_late}"></label>
                <label><span data-i18n="view.s6039.label.intentional">Intentional disregard?</span>
                    <input type="checkbox" name="intentional_disregard" ${state.intentional_disregard ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6039.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6039-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6039.h2.iso">ISO qualifying disposition (§ 422)</h2>
            <ul class="muted small">
                <li data-i18n="view.s6039.iso.grant_holding">2 years from GRANT date</li>
                <li data-i18n="view.s6039.iso.exercise_holding">1 year from EXERCISE date</li>
                <li data-i18n="view.s6039.iso.both">BOTH must be satisfied for qualifying disposition</li>
                <li data-i18n="view.s6039.iso.tax_qual">Tax treatment: 100% long-term capital gain on sale</li>
                <li data-i18n="view.s6039.iso.disqual">Disqualifying disposition: ordinary income (FMV-strike) up to gain</li>
                <li data-i18n="view.s6039.iso.amt_trap">AMT trap: § 56(b)(3) — spread at exercise included in AMTI</li>
                <li data-i18n="view.s6039.iso.amt_basis">AMT basis adjustment: increase by spread → reduces future AMT gain</li>
                <li data-i18n="view.s6039.iso.amt_credit">§ 53 minimum tax credit allows future recovery of AMT paid</li>
                <li data-i18n="view.s6039.iso.s100k">$100K rule: first $100K ISO grant per year eligible (excess = NSO)</li>
                <li data-i18n="view.s6039.iso.cashless">Cashless exercise: same-day sale → disqualifying (always)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6039.h2.espp">ESPP qualifying disposition (§ 423)</h2>
            <ul class="muted small">
                <li data-i18n="view.s6039.espp.discount">15% maximum discount on lower of: FMV at grant OR FMV at purchase</li>
                <li data-i18n="view.s6039.espp.grant_holding">2 years from beginning of offering period</li>
                <li data-i18n="view.s6039.espp.purchase_holding">1 year from purchase date</li>
                <li data-i18n="view.s6039.espp.qualifying_treatment">Qualifying disposition: discount = ordinary income up to gain, balance = LTCG</li>
                <li data-i18n="view.s6039.espp.disqualifying_treatment">Disqualifying disposition: discount + spread = ordinary income</li>
                <li data-i18n="view.s6039.espp.s25k">$25,000 limit per offering period (FMV at grant basis)</li>
                <li data-i18n="view.s6039.espp.tax_form">W-2 reports compensation income on qualifying disposition</li>
                <li data-i18n="view.s6039.espp.lookback">Lookback feature: discount based on lower of two prices</li>
                <li data-i18n="view.s6039.espp.eligible">All 5%+ owner OR HCE-excluded employees may be excluded</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6039.h2.penalties">§ 6721/6722 information return penalties (2024)</h2>
            <table class="data-table">
                <thead><tr><th data-i18n="view.s6039.tbl.violation">Violation</th><th data-i18n="view.s6039.tbl.per_form">Per form</th><th data-i18n="view.s6039.tbl.cap">Annual cap</th></tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s6039.tbl.30days">Filed within 30 days</td><td>$60</td><td>$232,500</td></tr>
                    <tr><td data-i18n="view.s6039.tbl.aug1">Filed after 30 days but by Aug 1</td><td>$130</td><td>$664,500</td></tr>
                    <tr><td data-i18n="view.s6039.tbl.aftaug1">Filed after Aug 1 or not at all</td><td>$310</td><td>$3,783,000</td></tr>
                    <tr><td data-i18n="view.s6039.tbl.intentional">Intentional disregard</td><td>$630</td><td>NO CAP</td></tr>
                </tbody>
            </table>
        </div>
    `;
    document.getElementById('s6039-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.form_type = fd.get('form_type');
        state.is_iso = !!fd.get('is_iso');
        state.is_espp = !!fd.get('is_espp');
        state.shares_exercised = Number(fd.get('shares_exercised')) || 0;
        state.fmv_at_exercise = Number(fd.get('fmv_at_exercise')) || 0;
        state.fmv_at_grant = Number(fd.get('fmv_at_grant')) || 0;
        state.exercise_price = Number(fd.get('exercise_price')) || 0;
        state.grant_date = fd.get('grant_date') || '';
        state.exercise_date = fd.get('exercise_date') || '';
        state.espp_discount_pct = Number(fd.get('espp_discount_pct')) || 0;
        state.is_qualifying_disposition = !!fd.get('is_qualifying_disposition');
        state.holding_period_satisfied = !!fd.get('holding_period_satisfied');
        state.days_held = Number(fd.get('days_held')) || 0;
        state.is_disqualifying = !!fd.get('is_disqualifying');
        state.amt_adjustment = Number(fd.get('amt_adjustment')) || 0;
        state.s421_b_gain = Number(fd.get('s421_b_gain')) || 0;
        state.forms_filed_count = Number(fd.get('forms_filed_count')) || 0;
        state.days_late = Number(fd.get('days_late')) || 0;
        state.intentional_disregard = !!fd.get('intentional_disregard');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6039-output');
    if (!el) return;
    const spread = (state.fmv_at_exercise - state.exercise_price) * state.shares_exercised;
    let per_form = 0;
    if (state.intentional_disregard) per_form = 630;
    else if (state.days_late > 213) per_form = 310;
    else if (state.days_late > 30) per_form = 130;
    else if (state.days_late > 0) per_form = 60;
    const total_penalty = per_form * state.forms_filed_count;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6039.h2.result">§ 6039 reporting + tax impact</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s6039.card.form">Form</div>
                    <div class="value">${esc(state.form_type)}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6039.card.spread">Spread (FMV - strike) × shares</div>
                    <div class="value">$${spread.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6039.card.per">Per-form penalty</div>
                    <div class="value">$${per_form.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${total_penalty > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6039.card.total">Total § 6721 penalty</div>
                    <div class="value">$${total_penalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${state.is_iso ? 'warn' : ''}">
                    <div class="label" data-i18n="view.s6039.card.amt">AMT adjustment (ISO)</div>
                    <div class="value">$${state.is_iso ? spread.toLocaleString(undefined, { maximumFractionDigits: 0 }) : '0'}</div>
                </div>
            </div>
        </div>
    `;
}
