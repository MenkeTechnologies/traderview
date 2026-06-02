// IRC § 7701(b) — Resident Alien Definition (Substantial Presence Test + Green Card).
// US resident alien if: (1) Green Card test OR (2) Substantial Presence Test.
// SPT: 31 days current year + 183 days weighted (current + 1/3 prior + 1/6 two prior).
// Closer connection exception: file Form 8840 if SPT met but closer connection to foreign country.
// Treaty tie-breaker: applies treaty residence rules to dual-resident situations.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    days_current_year: 0,
    days_one_year_prior: 0,
    days_two_years_prior: 0,
    has_green_card: false,
    green_card_first_year: false,
    closer_connection: false,
    foreign_tax_home: false,
    days_outside_us_with_treaty: 0,
    treaty_country: '',
    first_year_election: false,
    professional_athlete: false,
    student_or_teacher: false,
    exempt_individual: false,
    medical_condition_exception: false,
    days_lost_medical: 0,
};

export async function renderSection7701B(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s7701b.h1.title">// § 7701(b) RESIDENT ALIEN</span></h1>
        <p class="muted small" data-i18n="view.s7701b.hint.intro">
            US <strong>RESIDENT ALIEN</strong> if: (1) <strong>Green Card test</strong> (LPR status) OR (2)
            <strong>Substantial Presence Test (SPT)</strong>. <strong>SPT:</strong> <strong>31 days</strong>
            current year + <strong>183 days weighted</strong> (current + 1/3 prior + 1/6 two prior).
            <strong>Closer connection exception</strong> (Form 8840): SPT met but closer to foreign country.
            <strong>Treaty tie-breaker:</strong> dual residents — treaty residence rules apply.
            <strong>Resident alien:</strong> taxed on WORLDWIDE income. <strong>Nonresident alien:</strong>
            US-source only.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s7701b.h2.inputs">Inputs</h2>
            <form id="s7701b-form" class="inline-form">
                <label><span data-i18n="view.s7701b.label.cy">Days current year</span>
                    <input type="number" step="1" name="days_current_year" value="${state.days_current_year}"></label>
                <label><span data-i18n="view.s7701b.label.p1">Days one year prior</span>
                    <input type="number" step="1" name="days_one_year_prior" value="${state.days_one_year_prior}"></label>
                <label><span data-i18n="view.s7701b.label.p2">Days two years prior</span>
                    <input type="number" step="1" name="days_two_years_prior" value="${state.days_two_years_prior}"></label>
                <label><span data-i18n="view.s7701b.label.green">Has Green Card?</span>
                    <input type="checkbox" name="has_green_card" ${state.has_green_card ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7701b.label.green_first">First year green card?</span>
                    <input type="checkbox" name="green_card_first_year" ${state.green_card_first_year ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7701b.label.closer">Closer connection (Form 8840)?</span>
                    <input type="checkbox" name="closer_connection" ${state.closer_connection ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7701b.label.tax_home">Foreign tax home maintained?</span>
                    <input type="checkbox" name="foreign_tax_home" ${state.foreign_tax_home ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7701b.label.days_treaty">Treaty days outside US</span>
                    <input type="number" step="1" name="days_outside_us_with_treaty" value="${state.days_outside_us_with_treaty}"></label>
                <label><span data-i18n="view.s7701b.label.treaty_country">Treaty country</span>
                    <input type="text" name="treaty_country" value="${esc(state.treaty_country)}"></label>
                <label><span data-i18n="view.s7701b.label.first_year">First-year election § 7701(b)(4)?</span>
                    <input type="checkbox" name="first_year_election" ${state.first_year_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7701b.label.athlete">Professional athlete?</span>
                    <input type="checkbox" name="professional_athlete" ${state.professional_athlete ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7701b.label.student">Student or teacher (exempt indiv)?</span>
                    <input type="checkbox" name="student_or_teacher" ${state.student_or_teacher ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7701b.label.exempt">Exempt individual (J, F, M, Q visa)?</span>
                    <input type="checkbox" name="exempt_individual" ${state.exempt_individual ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7701b.label.medical">Medical condition exception?</span>
                    <input type="checkbox" name="medical_condition_exception" ${state.medical_condition_exception ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7701b.label.days_medical">Days lost to medical condition</span>
                    <input type="number" step="1" name="days_lost_medical" value="${state.days_lost_medical}"></label>
                <button class="primary" type="submit" data-i18n="view.s7701b.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s7701b-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7701b.h2.spt">Substantial Presence Test formula</h2>
            <ol class="muted small">
                <li data-i18n="view.s7701b.spt.31_min">Step 1: Count current-year days — must be ≥ 31 (else NOT resident regardless of formula)</li>
                <li data-i18n="view.s7701b.spt.weighted">Step 2: Weighted formula: current + 1/3 prior + 1/6 two prior</li>
                <li data-i18n="view.s7701b.spt.183">Step 3: If weighted total ≥ 183 → RESIDENT ALIEN</li>
                <li data-i18n="view.s7701b.spt.day_count">Day count: any day physically in US (even partial day = full day)</li>
                <li data-i18n="view.s7701b.spt.transit">Transit through US (&lt; 24 hrs): NOT counted</li>
                <li data-i18n="view.s7701b.spt.commuter">Daily commuters from Canada / Mexico: NOT counted</li>
                <li data-i18n="view.s7701b.spt.exempt_days">Exempt individual (F, J, M, Q visa) days don't count</li>
                <li data-i18n="view.s7701b.spt.medical">Medical condition keep days don't count if traveled w/ intent to leave</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7701b.h2.exemptions">Day-counting exemptions</h2>
            <ul class="muted small">
                <li data-i18n="view.s7701b.exc.foreign_govt">Foreign government employees (G-visa)</li>
                <li data-i18n="view.s7701b.exc.teacher_trainee">Teacher / trainee on J or Q visa (limited to 2 of past 6 yrs)</li>
                <li data-i18n="view.s7701b.exc.student">Student on F, J, M visa (limited to 5 calendar years)</li>
                <li data-i18n="view.s7701b.exc.athlete">Professional athlete competing in charitable sports event</li>
                <li data-i18n="view.s7701b.exc.medical">Medical condition preventing leaving (Form 8843)</li>
                <li data-i18n="view.s7701b.exc.in_transit">In transit through US ≤ 24 hrs</li>
                <li data-i18n="view.s7701b.exc.daily_commuter">Daily commuter from Canada / Mexico</li>
                <li data-i18n="view.s7701b.exc.fishing_vessel">Crew of foreign fishing vessel</li>
                <li data-i18n="view.s7701b.exc.dual_status">Dual-status year: residence may start / end mid-year</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7701b.h2.closer_connection">Closer connection exception (Form 8840)</h2>
            <ul class="muted small">
                <li data-i18n="view.s7701b.cc.spt_met">Available when SPT met but tax home is foreign + closer connection</li>
                <li data-i18n="view.s7701b.cc.183_in_year">NOT available if 183+ days in current year alone</li>
                <li data-i18n="view.s7701b.cc.factors">Factors: family, home, personal belongings, social ties, voting</li>
                <li data-i18n="view.s7701b.cc.no_green">CANNOT apply if Green Card applied for or pending</li>
                <li data-i18n="view.s7701b.cc.timely">Must file Form 8840 timely with Form 1040NR</li>
                <li data-i18n="view.s7701b.cc.tax_home_foreign">Tax home: principal place of business / employment</li>
                <li data-i18n="view.s7701b.cc.principal_residence">Principal place of residence: foreign</li>
                <li data-i18n="view.s7701b.cc.dual_residence">Dual residence: invoke treaty tie-breaker article instead</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7701b.h2.treaty">Treaty tie-breaker</h2>
            <ul class="muted small">
                <li data-i18n="view.s7701b.tt.dual_resident">Used when person is resident under BOTH US and foreign country domestic law</li>
                <li data-i18n="view.s7701b.tt.permanent_home">Test 1: Permanent home — country with permanent home wins</li>
                <li data-i18n="view.s7701b.tt.center_vital">Test 2: Center of vital interests — closer connections</li>
                <li data-i18n="view.s7701b.tt.habitual_abode">Test 3: Habitual abode</li>
                <li data-i18n="view.s7701b.tt.nationality">Test 4: Nationality</li>
                <li data-i18n="view.s7701b.tt.mutual_agreement">Test 5: Mutual agreement between competent authorities</li>
                <li data-i18n="view.s7701b.tt.us_treaty_residency">Treaty residency: tax as nonresident in non-residence country</li>
                <li data-i18n="view.s7701b.tt.disclosure_8833">Must file Form 8833 to invoke treaty position</li>
            </ul>
        </div>
    `;
    document.getElementById('s7701b-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.days_current_year = Number(fd.get('days_current_year')) || 0;
        state.days_one_year_prior = Number(fd.get('days_one_year_prior')) || 0;
        state.days_two_years_prior = Number(fd.get('days_two_years_prior')) || 0;
        state.has_green_card = !!fd.get('has_green_card');
        state.green_card_first_year = !!fd.get('green_card_first_year');
        state.closer_connection = !!fd.get('closer_connection');
        state.foreign_tax_home = !!fd.get('foreign_tax_home');
        state.days_outside_us_with_treaty = Number(fd.get('days_outside_us_with_treaty')) || 0;
        state.treaty_country = fd.get('treaty_country');
        state.first_year_election = !!fd.get('first_year_election');
        state.professional_athlete = !!fd.get('professional_athlete');
        state.student_or_teacher = !!fd.get('student_or_teacher');
        state.exempt_individual = !!fd.get('exempt_individual');
        state.medical_condition_exception = !!fd.get('medical_condition_exception');
        state.days_lost_medical = Number(fd.get('days_lost_medical')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s7701b-output');
    if (!el) return;
    const adjCurrent = Math.max(0, state.days_current_year - state.days_lost_medical);
    const weighted = adjCurrent + (state.days_one_year_prior / 3) + (state.days_two_years_prior / 6);
    const spt_met = adjCurrent >= 31 && weighted >= 183 && !state.exempt_individual;
    const is_resident_basic = state.has_green_card || spt_met;
    const closer_eligible = spt_met && state.foreign_tax_home && state.closer_connection && adjCurrent < 183;
    const final_is_resident = is_resident_basic && !closer_eligible;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s7701b.h2.result">§ 7701(b) determination</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s7701b.card.weighted">Weighted SPT days</div>
                    <div class="value">${weighted.toFixed(1)}</div>
                </div>
                <div class="card ${spt_met ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s7701b.card.spt_met">SPT met?</div>
                    <div class="value">${spt_met ? esc(t('view.s7701b.status.yes')) : esc(t('view.s7701b.status.no'))}</div>
                </div>
                <div class="card ${state.has_green_card ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s7701b.card.green">Green card?</div>
                    <div class="value">${state.has_green_card ? esc(t('view.s7701b.status.yes')) : esc(t('view.s7701b.status.no'))}</div>
                </div>
                <div class="card ${closer_eligible ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s7701b.card.closer">Closer connection avail?</div>
                    <div class="value">${closer_eligible ? esc(t('view.s7701b.status.yes')) : esc(t('view.s7701b.status.no'))}</div>
                </div>
                <div class="card ${final_is_resident ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s7701b.card.is_resident">Resident alien?</div>
                    <div class="value">${final_is_resident ? esc(t('view.s7701b.status.yes')) : esc(t('view.s7701b.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s7701b.card.filing">File form</div>
                    <div class="value">${final_is_resident ? '1040' : '1040NR'}</div>
                </div>
            </div>
            ${final_is_resident ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s7701b.resident_note">
                    RESIDENT ALIEN: subject to US tax on WORLDWIDE income. File Form 1040. May elect treaty
                    tie-breaker via Form 8833 if dual resident under treaty. Must comply with FBAR, Form 8938
                    if foreign assets &gt; $10K / $50K thresholds. § 911 FEIE may apply if working abroad.
                </p>
            ` : `
                <p class="muted small pos" style="margin-top:10px" data-i18n="view.s7701b.nonresident_note">
                    NONRESIDENT ALIEN: taxed only on US-source income + ECI. File Form 1040NR. 30% FDAP
                    withholding on US-source passive income. Capital gains on non-real-estate generally exempt.
                    § 871(m) dividend equivalent payments separately analyzed.
                </p>
            `}
        </div>
    `;
}
