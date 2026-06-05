// IRC § 51 — Work Opportunity Tax Credit (WOTC).
// Up to $2,400 per new hire from 9 targeted groups (veterans, ex-felons, TANF, SNAP, etc.).
// $9,600 enhanced for qualified veterans with service-connected disability.
// 40% on first $6K wages (25% if &lt; 400 hrs); 50% on 2nd-year wages for long-term TANF.
// Form 8850 pre-screening + Form 5884 + Form 3800 for general business credit.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    target_group: 'long_term_unemployed',
    wages_year_1: 0,
    wages_year_2: 0,
    hours_worked_year_1: 0,
    form_8850_filed_in_28_days: true,
    state_workforce_agency_certified: true,
    is_veteran: false,
    veteran_subgroup: 'none',
    is_ex_felon: false,
    is_long_term_tanf: false,
    is_long_term_unemployed: true,
    days_within_27_weeks_unemployment: 0,
    expired_zone: false,
};

export async function renderSection51(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s51.h1.title">// § 51 WOTC</span></h1>
        <p class="muted small" data-i18n="view.s51.hint.intro">
            Up to <strong>$2,400 per new hire</strong> from 9 targeted groups. <strong>$9,600 enhanced</strong>
            for qualified veterans with service-connected disability. <strong>40% on first $6K wages</strong>
            (25% if &lt; 400 hrs but ≥ 120); $0 if &lt; 120 hrs. <strong>50% on 2nd-year wages</strong> for
            long-term TANF. <strong>Form 8850</strong> pre-screening within 28 days of hire + Form 5884
            + Form 3800. <strong>Expired Dec 31 2025</strong> — historically renewed; awaiting congressional
            extension.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s51.h2.inputs">Inputs</h2>
            <form id="s51-form" class="inline-form">
                <label><span data-i18n="view.s51.label.group">Target group</span>
                    <select name="target_group">
                        <option value="long_term_unemployed" ${state.target_group === 'long_term_unemployed' ? 'selected' : ''}>Long-term unemployed (27 wks)</option>
                        <option value="snap_recipient" ${state.target_group === 'snap_recipient' ? 'selected' : ''}>SNAP recipient</option>
                        <option value="tanf_short" ${state.target_group === 'tanf_short' ? 'selected' : ''}>TANF short-term (9 months)</option>
                        <option value="tanf_long" ${state.target_group === 'tanf_long' ? 'selected' : ''}>TANF long-term (18 months) — 50% Yr2</option>
                        <option value="ex_felon" ${state.target_group === 'ex_felon' ? 'selected' : ''}>Ex-felon</option>
                        <option value="ssi_recipient" ${state.target_group === 'ssi_recipient' ? 'selected' : ''}>SSI recipient</option>
                        <option value="summer_youth" ${state.target_group === 'summer_youth' ? 'selected' : ''}>Summer youth (16-17 in EZ)</option>
                        <option value="vocational" ${state.target_group === 'vocational' ? 'selected' : ''}>Vocational rehab</option>
                        <option value="designated" ${state.target_group === 'designated' ? 'selected' : ''}>Designated community / EZ</option>
                        <option value="veteran_basic" ${state.target_group === 'veteran_basic' ? 'selected' : ''}>Veteran (basic)</option>
                        <option value="veteran_unemployed" ${state.target_group === 'veteran_unemployed' ? 'selected' : ''}>Veteran (unemployed 6 months)</option>
                        <option value="veteran_disabled" ${state.target_group === 'veteran_disabled' ? 'selected' : ''}>Veteran service-connected disability</option>
                    </select>
                </label>
                <label><span data-i18n="view.s51.label.year1">Year-1 wages ($)</span>
                    <input type="number" step="0.01" name="wages_year_1" value="${state.wages_year_1}"></label>
                <label><span data-i18n="view.s51.label.year2">Year-2 wages ($, long-term TANF only)</span>
                    <input type="number" step="0.01" name="wages_year_2" value="${state.wages_year_2}"></label>
                <label><span data-i18n="view.s51.label.hours">Hours worked Year 1</span>
                    <input type="number" step="0.01" name="hours_worked_year_1" value="${state.hours_worked_year_1}"></label>
                <label><span data-i18n="view.s51.label.form8850">Form 8850 filed in 28 days?</span>
                    <input type="checkbox" name="form_8850_filed_in_28_days" ${state.form_8850_filed_in_28_days ? 'checked' : ''}></label>
                <label><span data-i18n="view.s51.label.swa">State agency certified?</span>
                    <input type="checkbox" name="state_workforce_agency_certified" ${state.state_workforce_agency_certified ? 'checked' : ''}></label>
                <label><span data-i18n="view.s51.label.veteran">Veteran?</span>
                    <input type="checkbox" name="is_veteran" ${state.is_veteran ? 'checked' : ''}></label>
                <label><span data-i18n="view.s51.label.vet_sub">Veteran subgroup</span>
                    <select name="veteran_subgroup">
                        <option value="none" ${state.veteran_subgroup === 'none' ? 'selected' : ''}>Not veteran</option>
                        <option value="basic" ${state.veteran_subgroup === 'basic' ? 'selected' : ''}>Basic ($2,400)</option>
                        <option value="snap" ${state.veteran_subgroup === 'snap' ? 'selected' : ''}>SNAP-receiving ($2,400)</option>
                        <option value="unemployed_4wks" ${state.veteran_subgroup === 'unemployed_4wks' ? 'selected' : ''}>Unemployed 4+ weeks ($2,400)</option>
                        <option value="unemployed_6mo" ${state.veteran_subgroup === 'unemployed_6mo' ? 'selected' : ''}>Unemployed 6+ months ($5,600)</option>
                        <option value="disability_short" ${state.veteran_subgroup === 'disability_short' ? 'selected' : ''}>SC disability ($4,800)</option>
                        <option value="disability_long" ${state.veteran_subgroup === 'disability_long' ? 'selected' : ''}>SC disability unemployed 6 months ($9,600)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s51.label.felon">Ex-felon?</span>
                    <input type="checkbox" name="is_ex_felon" ${state.is_ex_felon ? 'checked' : ''}></label>
                <label><span data-i18n="view.s51.label.tanf">Long-term TANF (Year 2 eligible)?</span>
                    <input type="checkbox" name="is_long_term_tanf" ${state.is_long_term_tanf ? 'checked' : ''}></label>
                <label><span data-i18n="view.s51.label.long_unemployed">Long-term unemployed?</span>
                    <input type="checkbox" name="is_long_term_unemployed" ${state.is_long_term_unemployed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s51.label.days_27">Days unemployed within 27 wks</span>
                    <input type="number" step="1" name="days_within_27_weeks_unemployment" value="${state.days_within_27_weeks_unemployment}"></label>
                <label><span data-i18n="view.s51.label.expired">Expired (post-12/31/2025)?</span>
                    <input type="checkbox" name="expired_zone" ${state.expired_zone ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s51.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s51-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s51.h2.groups">9 targeted groups</h2>
            <ul class="muted small">
                <li data-i18n="view.s51.grp.tanf">TANF recipient (short-term 9 mo OR long-term 18 mo)</li>
                <li data-i18n="view.s51.grp.snap">SNAP recipient (food stamps for 3 of last 15 months)</li>
                <li data-i18n="view.s51.grp.unemployed">Long-term unemployed (27+ consecutive weeks; verified Form 9061)</li>
                <li data-i18n="view.s51.grp.veterans">Qualified veterans (basic + SNAP-receiving + unemployed + service-disabled)</li>
                <li data-i18n="view.s51.grp.felons">Ex-felons (released w/in 1 year)</li>
                <li data-i18n="view.s51.grp.dcr">Designated community resident (16-39 in empowerment zone)</li>
                <li data-i18n="view.s51.grp.voc">Vocational rehabilitation referral</li>
                <li data-i18n="view.s51.grp.summer">Summer youth (16-17, EZ, May 1 - September 15)</li>
                <li data-i18n="view.s51.grp.ssi">SSI recipient</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s51.h2.amounts">Credit amounts by group</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s51.th.group">Group</th>
                    <th data-i18n="view.s51.th.wage_cap">Wage cap</th>
                    <th data-i18n="view.s51.th.rate">Rate (≥ 400 hrs)</th>
                    <th data-i18n="view.s51.th.max">Max credit</th>
                </tr></thead>
                <tbody>
                    <tr><td>Most groups</td><td>$6,000</td><td>40%</td><td>$2,400</td></tr>
                    <tr><td>Veteran unemployed 6+ months</td><td>$14,000</td><td>40%</td><td>$5,600</td></tr>
                    <tr><td>Veteran SC disability + short</td><td>$12,000</td><td>40%</td><td>$4,800</td></tr>
                    <tr><td>Veteran SC disability + unemployed 6+ mo</td><td>$24,000</td><td>40%</td><td>$9,600</td></tr>
                    <tr><td>Long-term TANF Yr 1</td><td>$10,000</td><td>40%</td><td>$4,000</td></tr>
                    <tr><td>Long-term TANF Yr 2</td><td>$10,000</td><td>50%</td><td>$5,000</td></tr>
                    <tr><td>Summer youth (90 days)</td><td>$3,000</td><td>40%</td><td>$1,200</td></tr>
                    <tr><td>Hours 120-399 (any group)</td><td>—</td><td>25%</td><td>—</td></tr>
                    <tr><td>Hours &lt; 120</td><td>—</td><td>0%</td><td>0</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s51.h2.process">Form 8850 process (28-day deadline)</h2>
            <ol class="muted small">
                <li data-i18n="view.s51.proc.prescreen">Pre-screening: Form 8850 SIGNED BY EMPLOYEE on day of job offer</li>
                <li data-i18n="view.s51.proc.28days">FILE Form 8850 + ETA 9061 with State Workforce Agency WITHIN 28 DAYS of hire</li>
                <li data-i18n="view.s51.proc.swa_review">SWA reviews + certifies (free) or rejects within 60-90 days</li>
                <li data-i18n="view.s51.proc.certified">Once certified, claim on Form 5884 + Form 3800 GBC</li>
                <li data-i18n="view.s51.proc.deny">Denial: no appeal, but may retry with different group</li>
                <li data-i18n="view.s51.proc.missed">Missed 28-day: NO CREDIT for that employee — strict deadline</li>
                <li data-i18n="view.s51.proc.documentation">Documentation: DD-214 (veteran), public assistance records, ICE arrest record</li>
                <li data-i18n="view.s51.proc.thirdparty">Third-party services: 0.5-2% of credit fee, manage 28-day deadline</li>
            </ol>
        </div>
    `;
    document.getElementById('s51-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.target_group = fd.get('target_group');
        state.wages_year_1 = Number(fd.get('wages_year_1')) || 0;
        state.wages_year_2 = Number(fd.get('wages_year_2')) || 0;
        state.hours_worked_year_1 = Number(fd.get('hours_worked_year_1')) || 0;
        state.form_8850_filed_in_28_days = !!fd.get('form_8850_filed_in_28_days');
        state.state_workforce_agency_certified = !!fd.get('state_workforce_agency_certified');
        state.is_veteran = !!fd.get('is_veteran');
        state.veteran_subgroup = fd.get('veteran_subgroup');
        state.is_ex_felon = !!fd.get('is_ex_felon');
        state.is_long_term_tanf = !!fd.get('is_long_term_tanf');
        state.is_long_term_unemployed = !!fd.get('is_long_term_unemployed');
        state.days_within_27_weeks_unemployment = Number(fd.get('days_within_27_weeks_unemployment')) || 0;
        state.expired_zone = !!fd.get('expired_zone');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s51-output');
    if (!el) return;
    let wageCap = 6_000;
    let rate = 0.40;
    if (state.target_group === 'tanf_long') wageCap = 10_000;
    if (state.veteran_subgroup === 'unemployed_6mo') wageCap = 14_000;
    else if (state.veteran_subgroup === 'disability_short') wageCap = 12_000;
    else if (state.veteran_subgroup === 'disability_long') wageCap = 24_000;
    if (state.target_group === 'summer_youth') wageCap = 3_000;
    if (state.hours_worked_year_1 < 120) rate = 0;
    else if (state.hours_worked_year_1 < 400) rate = 0.25;
    const certifyTimely = state.form_8850_filed_in_28_days && state.state_workforce_agency_certified;
    const wages_eligible = Math.min(state.wages_year_1, wageCap);
    const year1Credit = certifyTimely && !state.expired_zone ? wages_eligible * rate : 0;
    let year2Credit = 0;
    if (state.is_long_term_tanf) {
        const wages_y2_eligible = Math.min(state.wages_year_2, 10_000);
        year2Credit = wages_y2_eligible * 0.50;
    }
    const totalCredit = year1Credit + year2Credit;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s51.h2.result">§ 51 WOTC computation</h2>
            <div class="cards">
                <div class="card ${certifyTimely ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s51.card.timely">28-day cert timely?</div>
                    <div class="value">${certifyTimely ? esc(t('view.s51.status.yes')) : esc(t('view.s51.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s51.card.wage_cap">Wage cap</div>
                    <div class="value">$${wageCap.toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s51.card.rate">Rate</div>
                    <div class="value">${(rate * 100).toFixed(0)}%</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s51.card.year1">Year 1 credit</div>
                    <div class="value">$${year1Credit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s51.card.year2">Year 2 credit (LTC TANF)</div>
                    <div class="value">$${year2Credit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s51.card.total">Total credit</div>
                    <div class="value">$${totalCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${!certifyTimely ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s51.late_note">
                    Missed 28-day Form 8850 deadline → NO CREDIT for this employee. Strict deadline,
                    no retroactive cure. Set up dedicated WOTC process or use 3rd-party service (Walton,
                    ADP, Equifax) to manage automatically on every hire.
                </p>
            ` : ''}
        </div>
    `;
}
