// IRC § 1298 — PFIC Special Rules (Attribution, Lookthrough, Reporting).
// § 1298(a): Attribution rules — PFIC stock owned through partnerships, S-corps, trusts, estates.
// § 1298(b)(1): "Once a PFIC, always a PFIC" — taint persists; cured only by purging.
// § 1298(b)(2): Startup exception — first year of new corp.
// § 1298(b)(8): CFC overlap — CFC US 10% shareholders escape PFIC regime.
// § 1298(f): Form 8621 reporting requirement for any direct / indirect US shareholder.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    us_shareholder_direct_pct: 0,
    held_via_partnership_pct: 0,
    held_via_s_corp_pct: 0,
    held_via_trust_pct: 0,
    held_via_estate_pct: 0,
    is_qualifying_startup: false,
    years_since_formation: 0,
    pfic_in_year_2: false,
    pfic_in_year_3: false,
    is_cfc_overlap: false,
    us_10pct_shareholder_of_cfc: false,
    purging_election_made: false,
    once_pfic_always_taint: true,
    form_8621_filed: false,
    reportable_threshold_met: false,
    cleansed_under_1298: false,
};

export async function renderSection1298(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1298.h1.title">// § 1298 PFIC SPECIAL RULES</span></h1>
        <p class="muted small" data-i18n="view.s1298.hint.intro">
            <strong>§ 1298(a) Attribution:</strong> PFIC stock owned through partnerships, S-corps, trusts,
            estates flows to ultimate US owner. <strong>§ 1298(b)(1) "Once PFIC, always PFIC":</strong>
            taint persists in shareholder's hands; cured only by PURGING (deemed sale at § 1291 rates).
            <strong>§ 1298(b)(2) Startup:</strong> first year of new corp may be exempt if not PFIC in
            first 3 years. <strong>§ 1298(b)(8) CFC overlap:</strong> CFC US 10% shareholders ESCAPE PFIC
            regime. <strong>§ 1298(f) Form 8621</strong> reporting requirement.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1298.h2.inputs">Inputs</h2>
            <form id="s1298-form" class="inline-form">
                <label><span data-i18n="view.s1298.label.direct">US direct ownership %</span>
                    <input type="number" step="0.01" name="us_shareholder_direct_pct" value="${state.us_shareholder_direct_pct}"></label>
                <label><span data-i18n="view.s1298.label.partnership">Held via partnership %</span>
                    <input type="number" step="0.01" name="held_via_partnership_pct" value="${state.held_via_partnership_pct}"></label>
                <label><span data-i18n="view.s1298.label.s_corp">Held via S-corp %</span>
                    <input type="number" step="0.01" name="held_via_s_corp_pct" value="${state.held_via_s_corp_pct}"></label>
                <label><span data-i18n="view.s1298.label.trust">Held via trust %</span>
                    <input type="number" step="0.01" name="held_via_trust_pct" value="${state.held_via_trust_pct}"></label>
                <label><span data-i18n="view.s1298.label.estate">Held via estate %</span>
                    <input type="number" step="0.01" name="held_via_estate_pct" value="${state.held_via_estate_pct}"></label>
                <label><span data-i18n="view.s1298.label.startup">Qualifying startup (1298(b)(2))?</span>
                    <input type="checkbox" name="is_qualifying_startup" ${state.is_qualifying_startup ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1298.label.years">Years since formation</span>
                    <input type="number" step="1" name="years_since_formation" value="${state.years_since_formation}"></label>
                <label><span data-i18n="view.s1298.label.y2">PFIC in year 2?</span>
                    <input type="checkbox" name="pfic_in_year_2" ${state.pfic_in_year_2 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1298.label.y3">PFIC in year 3?</span>
                    <input type="checkbox" name="pfic_in_year_3" ${state.pfic_in_year_3 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1298.label.cfc">CFC overlap (1298(b)(8))?</span>
                    <input type="checkbox" name="is_cfc_overlap" ${state.is_cfc_overlap ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1298.label.10pct">US 10%+ CFC shareholder?</span>
                    <input type="checkbox" name="us_10pct_shareholder_of_cfc" ${state.us_10pct_shareholder_of_cfc ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1298.label.purging">Purging election made?</span>
                    <input type="checkbox" name="purging_election_made" ${state.purging_election_made ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1298.label.taint">Once PFIC always PFIC taint?</span>
                    <input type="checkbox" name="once_pfic_always_taint" ${state.once_pfic_always_taint ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1298.label.form_8621">Form 8621 filed?</span>
                    <input type="checkbox" name="form_8621_filed" ${state.form_8621_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1298.label.threshold">Reportable threshold met ($25K)?</span>
                    <input type="checkbox" name="reportable_threshold_met" ${state.reportable_threshold_met ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1298.label.cleansed">Cleansed (sold + no PFIC since)?</span>
                    <input type="checkbox" name="cleansed_under_1298" ${state.cleansed_under_1298 ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s1298.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1298-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1298.h2.attribution">§ 1298(a) attribution rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s1298.att.partnership">Partnerships: pass through to partners pro-rata</li>
                <li data-i18n="view.s1298.att.s_corp">S-corps: pass to shareholders pro-rata</li>
                <li data-i18n="view.s1298.att.trust">Trusts: pass to beneficiaries by interest</li>
                <li data-i18n="view.s1298.att.estate">Estates: pass to beneficiaries</li>
                <li data-i18n="view.s1298.att.tiered">Tiered structure: cascade through all layers</li>
                <li data-i18n="view.s1298.att.grantor">Grantor trust: deemed owned by grantor for PFIC purposes</li>
                <li data-i18n="view.s1298.att.foreign_partnership">Foreign partnership: still PFIC pass-through to US owners</li>
                <li data-i18n="view.s1298.att.cooperative">Cooperative ownership rules: aggregate indirect interests</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1298.h2.startup_exc">§ 1298(b)(2) startup exception</h2>
            <ul class="muted small">
                <li data-i18n="view.s1298.start.first_year">FIRST YEAR of new active business corp</li>
                <li data-i18n="view.s1298.start.test">Test: not PFIC in any of first 3 yrs</li>
                <li data-i18n="view.s1298.start.year_1">If passes 3-yr test → year 1 retroactively NOT PFIC</li>
                <li data-i18n="view.s1298.start.year_1_pending">Year 1: tentatively PFIC (pending years 2-3 outcome)</li>
                <li data-i18n="view.s1298.start.no_active_business">No active business → not eligible for startup exception</li>
                <li data-i18n="view.s1298.start.pre_operating">Pre-operating phase common (no income but high assets)</li>
                <li data-i18n="view.s1298.start.cash_intensive">Cash-intensive R&D stage: typically PFIC despite eventual active business</li>
                <li data-i18n="view.s1298.start.failed_acquirer">Failed acquirer: holding-co structure may inadvertently trigger</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1298.h2.cfc_overlap">§ 1298(b)(8) CFC overlap exception</h2>
            <ul class="muted small">
                <li data-i18n="view.s1298.cfc.test">US 10% shareholder of CFC: NOT subject to PFIC regime</li>
                <li data-i18n="view.s1298.cfc.policy">Policy: subF + GILTI already address tax abuse; no double-rule</li>
                <li data-i18n="view.s1298.cfc.s951_a">§ 951(a) inclusions replace PFIC § 1291 / 1295 / 1296</li>
                <li data-i18n="view.s1298.cfc.test_when_no_longer">If US ownership drops below 10% → PFIC regime triggers</li>
                <li data-i18n="view.s1298.cfc.dual_residency">Dual residency exception: treaty-resident considered foreign for PFIC</li>
                <li data-i18n="view.s1298.cfc.form_5471">Form 5471 reporting replaces Form 8621 PFIC reporting</li>
                <li data-i18n="view.s1298.cfc.smaller_shareholder">Other US shareholders (&lt; 10%) still subject to PFIC</li>
                <li data-i18n="view.s1298.cfc.s962_election">§ 962 election available for individual US 10% shareholders</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1298.h2.taint">Persistent taint + purging</h2>
            <ul class="muted small">
                <li data-i18n="view.s1298.taint.basic">§ 1298(b)(1): foreign corp PFIC for any year in shareholder's hands = PFIC for ALL subsequent years</li>
                <li data-i18n="view.s1298.taint.corp_changes">Even if corp changes operations + ceases to be PFIC under tests, taint persists to that shareholder</li>
                <li data-i18n="view.s1298.taint.purge_deemed_sale">Purge via DEEMED SALE election: recognize gain currently at § 1291 rates → restart clean</li>
                <li data-i18n="view.s1298.taint.purge_qef_late">Late QEF election: also requires PURGING via deemed sale</li>
                <li data-i18n="view.s1298.taint.purge_mtm_late">MTM election (after PFIC status): apply purging gain recognition</li>
                <li data-i18n="view.s1298.taint.disposition_cleanses">Complete disposition (sale + 36 months pass): may cleanse</li>
                <li data-i18n="view.s1298.taint.death">Death of shareholder: § 1014 step-up + restart from heirs' perspective</li>
                <li data-i18n="view.s1298.taint.cleansing_election">§ 1298(b)(1)(B) cleansing election: deemed sale at § 1291 rates → cleanse</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1298.h2.reporting">§ 1298(f) Form 8621 reporting</h2>
            <ul class="muted small">
                <li data-i18n="view.s1298.rep.required">Any US person owning PFIC interest (direct or indirect) MUST file Form 8621</li>
                <li data-i18n="view.s1298.rep.threshold">Reportable: aggregate value > $25K ($50K MFJ) OR Pedigree Election Made</li>
                <li data-i18n="view.s1298.rep.timing">Filed with timely-filed 1040 / 1041 / 1065 / 1120 / 1120-S</li>
                <li data-i18n="view.s1298.rep.failure">Failure to file: $10K minimum + 25% understatement penalty</li>
                <li data-i18n="view.s1298.rep.statute_extension">Statute of limitations extends 3 yrs after filing</li>
                <li data-i18n="view.s1298.rep.per_pfic">Form 8621 PER PFIC each year (not aggregate)</li>
                <li data-i18n="view.s1298.rep.partner_share">Partners + S-corp shareholders file own Form 8621</li>
                <li data-i18n="view.s1298.rep.trustees">Grantor trust trustees file on behalf of grantor</li>
            </ul>
        </div>
    `;
    document.getElementById('s1298-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.us_shareholder_direct_pct = Number(fd.get('us_shareholder_direct_pct')) || 0;
        state.held_via_partnership_pct = Number(fd.get('held_via_partnership_pct')) || 0;
        state.held_via_s_corp_pct = Number(fd.get('held_via_s_corp_pct')) || 0;
        state.held_via_trust_pct = Number(fd.get('held_via_trust_pct')) || 0;
        state.held_via_estate_pct = Number(fd.get('held_via_estate_pct')) || 0;
        state.is_qualifying_startup = !!fd.get('is_qualifying_startup');
        state.years_since_formation = Number(fd.get('years_since_formation')) || 0;
        state.pfic_in_year_2 = !!fd.get('pfic_in_year_2');
        state.pfic_in_year_3 = !!fd.get('pfic_in_year_3');
        state.is_cfc_overlap = !!fd.get('is_cfc_overlap');
        state.us_10pct_shareholder_of_cfc = !!fd.get('us_10pct_shareholder_of_cfc');
        state.purging_election_made = !!fd.get('purging_election_made');
        state.once_pfic_always_taint = !!fd.get('once_pfic_always_taint');
        state.form_8621_filed = !!fd.get('form_8621_filed');
        state.reportable_threshold_met = !!fd.get('reportable_threshold_met');
        state.cleansed_under_1298 = !!fd.get('cleansed_under_1298');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1298-output');
    if (!el) return;
    const totalOwnership = state.us_shareholder_direct_pct + state.held_via_partnership_pct + state.held_via_s_corp_pct + state.held_via_trust_pct + state.held_via_estate_pct;
    const startup_qualifies = state.is_qualifying_startup && state.years_since_formation === 1 && !state.pfic_in_year_2 && !state.pfic_in_year_3;
    const cfc_excludes_pfic = state.is_cfc_overlap && state.us_10pct_shareholder_of_cfc;
    const taint_purgeable = state.once_pfic_always_taint && (state.purging_election_made || state.cleansed_under_1298);
    const reporting_required = state.reportable_threshold_met && !state.form_8621_filed;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1298.h2.result">§ 1298 outcome</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s1298.card.total">Total attributed ownership %</div>
                    <div class="value">${totalOwnership.toFixed(2)}%</div>
                </div>
                <div class="card ${startup_qualifies ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s1298.card.startup">Startup exception?</div>
                    <div class="value">${startup_qualifies ? esc(t('view.s1298.status.yes')) : esc(t('view.s1298.status.no'))}</div>
                </div>
                <div class="card ${cfc_excludes_pfic ? 'pos' : ''}">
                    <div class="label" data-i18n="view.s1298.card.cfc">CFC overlap excludes PFIC?</div>
                    <div class="value">${cfc_excludes_pfic ? esc(t('view.s1298.status.yes')) : esc(t('view.s1298.status.no'))}</div>
                </div>
                <div class="card ${state.once_pfic_always_taint ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s1298.card.taint">Persistent taint?</div>
                    <div class="value">${state.once_pfic_always_taint ? esc(t('view.s1298.status.yes')) : esc(t('view.s1298.status.no'))}</div>
                </div>
                <div class="card ${taint_purgeable ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s1298.card.purge">Taint purged?</div>
                    <div class="value">${taint_purgeable ? esc(t('view.s1298.status.yes')) : esc(t('view.s1298.status.no'))}</div>
                </div>
                <div class="card ${reporting_required ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s1298.card.reporting">Form 8621 required?</div>
                    <div class="value">${reporting_required ? esc(t('view.s1298.status.yes')) : esc(t('view.s1298.status.no'))}</div>
                </div>
            </div>
            ${reporting_required ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s1298.report_note">
                    Form 8621 REQUIRED but NOT FILED. Penalty: $10K minimum + 25% understatement penalty + statute
                    of limitations 3-yr extension. Streamlined Filing Compliance Procedures (SFCP) may apply for
                    delinquent reporting. Foreign mutual funds + ETFs almost always PFICs → annual filing per fund.
                </p>
            ` : ''}
        </div>
    `;
}
