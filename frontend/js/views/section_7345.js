// IRC § 7345 — Passport Revocation for Seriously Delinquent Tax Debt.
// IRS certifies to State Department: $62,000 (2024, indexed) assessed federal tax debt + lien
// filed OR levy issued. State revokes or denies passport. Issued passports limited to direct US return.
// CP508C notice with right to reverse. § 7345(g) safety valve: pay or installment agreement.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const THRESHOLD_2024 = 62_000;

let state = {
    total_assessed_tax: 0,
    interest_and_penalties: 0,
    in_installment_agreement: false,
    in_offer_in_compromise: false,
    cdp_hearing_requested: false,
    innocent_spouse_pending: false,
    is_combat_zone: false,
    is_bankruptcy: false,
    foia_collection_due: false,
};

export async function renderSection7345(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s7345.h1.title">// § 7345 PASSPORT REVOCATION</span></h1>
        <p class="muted small" data-i18n="view.s7345.hint.intro">
            "Seriously delinquent tax debt" = <strong>&gt; $62,000 (2024, indexed)</strong> assessed
            federal tax + <strong>lien filed OR levy issued</strong>. IRS certifies to State Dept;
            passport revoked / denied. <strong>CP508C notice</strong> issued; right to reverse via
            payment / installment agreement / CDP hearing. Issued passports may be LIMITED to direct
            US return. <strong>Foreign / US-Mexican border crossings denied.</strong>
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s7345.h2.inputs">Inputs</h2>
            <form id="s7345-form" class="inline-form">
                <label><span data-i18n="view.s7345.label.assessed">Total assessed tax ($)</span>
                    <input type="number" step="1000" name="total_assessed_tax" value="${state.total_assessed_tax}"></label>
                <label><span data-i18n="view.s7345.label.interest">Interest + penalties ($)</span>
                    <input type="number" step="1000" name="interest_and_penalties" value="${state.interest_and_penalties}"></label>
                <label><span data-i18n="view.s7345.label.ia">Installment agreement in effect?</span>
                    <input type="checkbox" name="in_installment_agreement" ${state.in_installment_agreement ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7345.label.oic">Offer-in-Compromise pending?</span>
                    <input type="checkbox" name="in_offer_in_compromise" ${state.in_offer_in_compromise ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7345.label.cdp">CDP hearing requested?</span>
                    <input type="checkbox" name="cdp_hearing_requested" ${state.cdp_hearing_requested ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7345.label.innocent">Innocent spouse pending?</span>
                    <input type="checkbox" name="innocent_spouse_pending" ${state.innocent_spouse_pending ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7345.label.combat">Combat zone exclusion?</span>
                    <input type="checkbox" name="is_combat_zone" ${state.is_combat_zone ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7345.label.bankruptcy">In Title 11 bankruptcy?</span>
                    <input type="checkbox" name="is_bankruptcy" ${state.is_bankruptcy ? 'checked' : ''}></label>
                <label><span data-i18n="view.s7345.label.foia">CSED suspended (CDP / OIC / IA)?</span>
                    <input type="checkbox" name="foia_collection_due" ${state.foia_collection_due ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s7345.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s7345-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7345.h2.exclusions">§ 7345(b) exclusions from "seriously delinquent"</h2>
            <ul class="muted small">
                <li data-i18n="view.s7345.excl.ia">Currently paying under installment agreement</li>
                <li data-i18n="view.s7345.excl.oic">Pending Offer-in-Compromise</li>
                <li data-i18n="view.s7345.excl.cdp">CDP hearing requested + timely</li>
                <li data-i18n="view.s7345.excl.innocent">Innocent spouse election pending</li>
                <li data-i18n="view.s7345.excl.combat">Combat zone-related § 7508 deferral</li>
                <li data-i18n="view.s7345.excl.bankruptcy">In bankruptcy under Title 11</li>
                <li data-i18n="view.s7345.excl.identity">Identified by IRS as identity theft victim</li>
                <li data-i18n="view.s7345.excl.fed_disaster">Federal disaster area resident</li>
                <li data-i18n="view.s7345.excl.tao">Taxpayer Assistance Order (TAO) pending</li>
                <li data-i18n="view.s7345.excl.hardship">CNC (Currently Not Collectible) status</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s7345.h2.remedy">How to reverse</h2>
            <ul class="muted small">
                <li data-i18n="view.s7345.rem.pay_full">Pay in full — IRS reverses within 30 days</li>
                <li data-i18n="view.s7345.rem.ia">Enter installment agreement (Form 9465 / OPA)</li>
                <li data-i18n="view.s7345.rem.oic">Submit OIC (Form 656)</li>
                <li data-i18n="view.s7345.rem.cdp_request">Request CDP hearing (Form 12153 within 30 days of notice)</li>
                <li data-i18n="view.s7345.rem.expedite">Imminent international travel: contact Taxpayer Advocate for expedited reversal</li>
                <li data-i18n="view.s7345.rem.court">§ 7345(e) suit in district court / Tax Court (limited remedies)</li>
                <li data-i18n="view.s7345.rem.emergency">Emergency passport for direct US return: contact embassy/consulate</li>
            </ul>
        </div>
    `;
    document.getElementById('s7345-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.total_assessed_tax = Number(fd.get('total_assessed_tax')) || 0;
        state.interest_and_penalties = Number(fd.get('interest_and_penalties')) || 0;
        state.in_installment_agreement = !!fd.get('in_installment_agreement');
        state.in_offer_in_compromise = !!fd.get('in_offer_in_compromise');
        state.cdp_hearing_requested = !!fd.get('cdp_hearing_requested');
        state.innocent_spouse_pending = !!fd.get('innocent_spouse_pending');
        state.is_combat_zone = !!fd.get('is_combat_zone');
        state.is_bankruptcy = !!fd.get('is_bankruptcy');
        state.foia_collection_due = !!fd.get('foia_collection_due');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s7345-output');
    if (!el) return;
    const totalDebt = state.total_assessed_tax + state.interest_and_penalties;
    const exceedsThreshold = totalDebt > THRESHOLD_2024;
    const hasExclusion = state.in_installment_agreement || state.in_offer_in_compromise
        || state.cdp_hearing_requested || state.innocent_spouse_pending
        || state.is_combat_zone || state.is_bankruptcy || state.foia_collection_due;
    const seriouslyDelinquent = exceedsThreshold && !hasExclusion;
    const passportAtRisk = seriouslyDelinquent;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s7345.h2.result">Passport revocation analysis</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s7345.card.total">Total debt</div>
                    <div class="value">$${totalDebt.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s7345.card.threshold">2024 threshold</div>
                    <div class="value">$${THRESHOLD_2024.toLocaleString()}</div>
                </div>
                <div class="card ${exceedsThreshold ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s7345.card.exceeds">Exceeds threshold</div>
                    <div class="value">${exceedsThreshold ? esc(t('view.s7345.status.yes')) : esc(t('view.s7345.status.no'))}</div>
                </div>
                <div class="card ${hasExclusion ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s7345.card.has_exclusion">Has § 7345(b) exclusion</div>
                    <div class="value">${hasExclusion ? esc(t('view.s7345.status.yes')) : esc(t('view.s7345.status.no'))}</div>
                </div>
                <div class="card ${seriouslyDelinquent ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s7345.card.serious">"Seriously delinquent"?</div>
                    <div class="value">${seriouslyDelinquent ? esc(t('view.s7345.status.yes')) : esc(t('view.s7345.status.no'))}</div>
                </div>
                <div class="card ${passportAtRisk ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s7345.card.passport">Passport at risk</div>
                    <div class="value">${passportAtRisk ? esc(t('view.s7345.status.yes')) : esc(t('view.s7345.status.no'))}</div>
                </div>
            </div>
            ${passportAtRisk ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s7345.warning.act_now">
                    ACT NOW: enter installment agreement / OIC / CDP hearing BEFORE IRS certifies
                    to State Dept. Once certified (CP508C), passport revocation can be immediate
                    on next application / renewal. 30-day reversal after qualifying action.
                </p>
            ` : ''}
        </div>
    `;
}
