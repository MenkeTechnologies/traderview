// IRC § 1362 — S-Corporation Election + Revocation + Termination.
// Form 2553 filed by 15th day of 3rd month of tax year for election in that year.
// ALL shareholders must consent in writing.
// Termination triggers: revocation, > 100 shareholders, ineligible shareholder, > 25% passive income (3 yrs).
// Late election relief: Rev. Proc. 2013-30 + Rev. Proc. 2022-19 (private letter rulings rarely needed).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    election_type: 'initial',
    target_effective_date: '',
    form_2553_filed_date: '',
    days_since_year_start: 0,
    all_shareholders_consent: true,
    number_of_shareholders: 0,
    has_ineligible_shareholder: false,
    has_more_than_one_class: false,
    passive_income_pct_yr_1: 0,
    passive_income_pct_yr_2: 0,
    passive_income_pct_yr_3: 0,
    has_e_and_p_from_c: true,
    late_election_qsst_eligible: false,
    revocation_filed: false,
    inadvertent_termination: false,
};

export async function renderSection1362(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1362.h1.title">// § 1362 S-CORP ELECTION</span></h1>
        <p class="muted small" data-i18n="view.s1362.hint.intro">
            <strong>Form 2553 filed by 15th day of 3rd month</strong> of tax year for election that year.
            <strong>ALL shareholders must consent</strong> in writing. <strong>Termination triggers:</strong>
            revocation, &gt; 100 shareholders, ineligible shareholder, &gt; 25% passive income (3 consecutive
            yrs with C-corp E&P). <strong>Late election relief:</strong> Rev. Proc. 2013-30 + Rev. Proc.
            2022-19 — relief often available within 3 yrs 75 days. <strong>5-year post-termination wait</strong>
            before re-electing (unless IRS consent). <strong>Inadvertent termination</strong> relief
            § 1362(f) — taxpayer + IRS in tandem.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1362.h2.inputs">Inputs</h2>
            <form id="s1362-form" class="inline-form">
                <label><span data-i18n="view.s1362.label.type">Election type</span>
                    <select name="election_type">
                        <option value="initial" ${state.election_type === 'initial' ? 'selected' : ''}>Initial election (Form 2553)</option>
                        <option value="late" ${state.election_type === 'late' ? 'selected' : ''}>Late election (Rev. Proc. 2013-30)</option>
                        <option value="revocation" ${state.election_type === 'revocation' ? 'selected' : ''}>Revocation</option>
                        <option value="termination_review" ${state.election_type === 'termination_review' ? 'selected' : ''}>Termination review</option>
                        <option value="inadvertent" ${state.election_type === 'inadvertent' ? 'selected' : ''}>Inadvertent termination relief</option>
                        <option value="qsub" ${state.election_type === 'qsub' ? 'selected' : ''}>QSub election (§ 1361(b)(3))</option>
                    </select>
                </label>
                <label><span data-i18n="view.s1362.label.effective">Target effective date</span>
                    <input type="date" name="target_effective_date" value="${state.target_effective_date}"></label>
                <label><span data-i18n="view.s1362.label.filed">Form 2553 filed date</span>
                    <input type="date" name="form_2553_filed_date" value="${state.form_2553_filed_date}"></label>
                <label><span data-i18n="view.s1362.label.days">Days since year start</span>
                    <input type="number" step="1" name="days_since_year_start" value="${state.days_since_year_start}"></label>
                <label><span data-i18n="view.s1362.label.consent">All shareholders consent?</span>
                    <input type="checkbox" name="all_shareholders_consent" ${state.all_shareholders_consent ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1362.label.count">Number of shareholders</span>
                    <input type="number" step="1" name="number_of_shareholders" value="${state.number_of_shareholders}"></label>
                <label><span data-i18n="view.s1362.label.ineligible">Has ineligible shareholder?</span>
                    <input type="checkbox" name="has_ineligible_shareholder" ${state.has_ineligible_shareholder ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1362.label.classes">More than one class of stock?</span>
                    <input type="checkbox" name="has_more_than_one_class" ${state.has_more_than_one_class ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1362.label.p1">Passive income % yr 1</span>
                    <input type="number" step="0.1" name="passive_income_pct_yr_1" value="${state.passive_income_pct_yr_1}"></label>
                <label><span data-i18n="view.s1362.label.p2">Passive income % yr 2</span>
                    <input type="number" step="0.1" name="passive_income_pct_yr_2" value="${state.passive_income_pct_yr_2}"></label>
                <label><span data-i18n="view.s1362.label.p3">Passive income % yr 3</span>
                    <input type="number" step="0.1" name="passive_income_pct_yr_3" value="${state.passive_income_pct_yr_3}"></label>
                <label><span data-i18n="view.s1362.label.e_p">Has C-corp E&P?</span>
                    <input type="checkbox" name="has_e_and_p_from_c" ${state.has_e_and_p_from_c ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1362.label.qsst">Late QSST eligibility?</span>
                    <input type="checkbox" name="late_election_qsst_eligible" ${state.late_election_qsst_eligible ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1362.label.revoke">Revocation filed?</span>
                    <input type="checkbox" name="revocation_filed" ${state.revocation_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1362.label.inadvertent">Inadvertent termination?</span>
                    <input type="checkbox" name="inadvertent_termination" ${state.inadvertent_termination ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s1362.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1362-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1362.h2.election_mechanics">Form 2553 election mechanics</h2>
            <ul class="muted small">
                <li data-i18n="view.s1362.mech.deadline">By 15th day of 3rd month of tax year (March 15 for calendar)</li>
                <li data-i18n="view.s1362.mech.late_75">75-day late election available with reasonable cause</li>
                <li data-i18n="view.s1362.mech.signatures">ALL shareholders + spouses (community property states) must sign</li>
                <li data-i18n="view.s1362.mech.entity_class">Entity must be eligible S corp: domestic, ≤ 100 shareholders, no ineligible, 1 class of stock</li>
                <li data-i18n="view.s1362.mech.tax_year">Choose acceptable tax year (calendar OR § 444 fiscal w/ § 7519 deposit)</li>
                <li data-i18n="view.s1362.mech.confirmation">IRS confirmation: CP261 notice (acceptance) within 60 days</li>
                <li data-i18n="view.s1362.mech.no_form_filed">If no CP261: contact IRS — common error</li>
                <li data-i18n="view.s1362.mech.qsst_esbt">QSST / ESBT trust elections: required to be eligible shareholders</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1362.h2.termination">Termination triggers</h2>
            <ul class="muted small">
                <li data-i18n="view.s1362.term.revocation">Revocation: majority shareholder consent + Form 2553A statement</li>
                <li data-i18n="view.s1362.term.over_100">&gt; 100 shareholders at any time</li>
                <li data-i18n="view.s1362.term.ineligible">Ineligible shareholder (corp, partnership, NRA, non-grantor trust)</li>
                <li data-i18n="view.s1362.term.classes">2+ classes of stock (voting / non-voting OK; different distribution rights NOT)</li>
                <li data-i18n="view.s1362.term.passive_income">&gt; 25% passive income 3 consecutive years + has C-corp E&P → automatic termination</li>
                <li data-i18n="view.s1362.term.inadvertent_relief">Inadvertent termination: § 1362(f) relief if cured promptly</li>
                <li data-i18n="view.s1362.term.5_year_wait">5-year wait before re-election (IRS consent required to shorten)</li>
                <li data-i18n="view.s1362.term.disposition">Liquidation / dissolution: terminates S election</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1362.h2.late_relief">Late election relief — Rev. Proc. 2013-30</h2>
            <ul class="muted small">
                <li data-i18n="view.s1362.lr.purpose">Cure missed election + missed terminating events without PLR</li>
                <li data-i18n="view.s1362.lr.scope">Scope: S-corp election, ESBT, QSST, QSub elections</li>
                <li data-i18n="view.s1362.lr.timing">Generally must request within 3 years 75 days of intended effective date</li>
                <li data-i18n="view.s1362.lr.requirements">Requirements: reasonable cause, return filed consistent w/ S, statement of facts</li>
                <li data-i18n="view.s1362.lr.late_filing">Late filing: attach Form 2553 to first late return + Rev. Proc. reference</li>
                <li data-i18n="view.s1362.lr.2022_19">Rev. Proc. 2022-19: updated guidance + expanded scope for common errors</li>
                <li data-i18n="view.s1362.lr.user_fee">No user fee — free relief if timely</li>
                <li data-i18n="view.s1362.lr.plr_alternative">PLR alternative: ~$30K+ fee + 6-9 months wait if outside automatic relief</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1362.h2.eligible_shareholders">Eligible shareholders (§ 1361(b)(1))</h2>
            <ul class="muted small">
                <li data-i18n="view.s1362.es.individuals">Individuals (US citizens / residents)</li>
                <li data-i18n="view.s1362.es.estates">Estates (during admin period)</li>
                <li data-i18n="view.s1362.es.qsst">Qualified Subchapter S Trust (QSST) — single beneficiary US person</li>
                <li data-i18n="view.s1362.es.esbt">Electing Small Business Trust (ESBT) — multi-beneficiary US persons</li>
                <li data-i18n="view.s1362.es.tax_exempt">Tax-exempt under § 501(c)(3) (post-1997)</li>
                <li data-i18n="view.s1362.es.iras">IRA shareholders (post-2017 for bank S-corps only)</li>
                <li data-i18n="view.s1362.es.grantor_trust">Grantor trust (during grantor's lifetime + 2 yrs after death)</li>
                <li data-i18n="view.s1362.es.no_corp">NOT eligible: corp, partnership, foreign trust, NRA, multi-beneficiary trust w/o ESBT</li>
            </ul>
        </div>
    `;
    document.getElementById('s1362-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.election_type = fd.get('election_type');
        state.target_effective_date = fd.get('target_effective_date');
        state.form_2553_filed_date = fd.get('form_2553_filed_date');
        state.days_since_year_start = Number(fd.get('days_since_year_start')) || 0;
        state.all_shareholders_consent = !!fd.get('all_shareholders_consent');
        state.number_of_shareholders = Number(fd.get('number_of_shareholders')) || 0;
        state.has_ineligible_shareholder = !!fd.get('has_ineligible_shareholder');
        state.has_more_than_one_class = !!fd.get('has_more_than_one_class');
        state.passive_income_pct_yr_1 = Number(fd.get('passive_income_pct_yr_1')) || 0;
        state.passive_income_pct_yr_2 = Number(fd.get('passive_income_pct_yr_2')) || 0;
        state.passive_income_pct_yr_3 = Number(fd.get('passive_income_pct_yr_3')) || 0;
        state.has_e_and_p_from_c = !!fd.get('has_e_and_p_from_c');
        state.late_election_qsst_eligible = !!fd.get('late_election_qsst_eligible');
        state.revocation_filed = !!fd.get('revocation_filed');
        state.inadvertent_termination = !!fd.get('inadvertent_termination');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1362-output');
    if (!el) return;
    const election_timely = state.days_since_year_start <= 75 && state.all_shareholders_consent;
    const election_valid = election_timely && !state.has_ineligible_shareholder && !state.has_more_than_one_class && state.number_of_shareholders <= 100;
    const passive_termination = state.has_e_and_p_from_c && state.passive_income_pct_yr_1 > 25 && state.passive_income_pct_yr_2 > 25 && state.passive_income_pct_yr_3 > 25;
    const auto_termination = passive_termination || state.has_ineligible_shareholder || state.has_more_than_one_class || state.number_of_shareholders > 100;
    const inadvertent_relief = state.inadvertent_termination;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1362.h2.result">§ 1362 outcome</h2>
            <div class="cards">
                <div class="card ${election_valid ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s1362.card.valid">Election valid?</div>
                    <div class="value">${election_valid ? esc(t('view.s1362.status.yes')) : esc(t('view.s1362.status.no'))}</div>
                </div>
                <div class="card ${election_timely ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s1362.card.timely">Timely filing?</div>
                    <div class="value">${election_timely ? esc(t('view.s1362.status.yes')) : esc(t('view.s1362.status.no'))}</div>
                </div>
                <div class="card ${state.number_of_shareholders > 100 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s1362.card.shareholders">≤ 100 shareholders?</div>
                    <div class="value">${state.number_of_shareholders <= 100 ? esc(t('view.s1362.status.yes')) : esc(t('view.s1362.status.no'))}</div>
                </div>
                <div class="card ${state.has_ineligible_shareholder ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s1362.card.eligible">All eligible?</div>
                    <div class="value">${state.has_ineligible_shareholder ? esc(t('view.s1362.status.no')) : esc(t('view.s1362.status.yes'))}</div>
                </div>
                <div class="card ${state.has_more_than_one_class ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s1362.card.one_class">One class of stock?</div>
                    <div class="value">${state.has_more_than_one_class ? esc(t('view.s1362.status.no')) : esc(t('view.s1362.status.yes'))}</div>
                </div>
                <div class="card ${passive_termination ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s1362.card.passive_term">Passive termination?</div>
                    <div class="value">${passive_termination ? esc(t('view.s1362.status.yes')) : esc(t('view.s1362.status.no'))}</div>
                </div>
                <div class="card ${auto_termination ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s1362.card.auto_term">Auto termination triggered?</div>
                    <div class="value">${auto_termination ? esc(t('view.s1362.status.yes')) : esc(t('view.s1362.status.no'))}</div>
                </div>
                <div class="card ${inadvertent_relief ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s1362.card.relief">Inadvertent termination relief?</div>
                    <div class="value">${inadvertent_relief ? esc(t('view.s1362.status.yes')) : esc(t('view.s1362.status.no'))}</div>
                </div>
            </div>
            ${auto_termination && !inadvertent_relief ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s1362.terminated_note">
                    S election TERMINATED automatically. 5-year wait before re-election (IRS consent required
                    to shorten). Apply for inadvertent termination relief under § 1362(f) if facts permit —
                    requires showing termination was inadvertent + cured promptly + reasonable. PLR application
                    typically required. Tax cost: C-corp double tax until cured.
                </p>
            ` : ''}
        </div>
    `;
}
