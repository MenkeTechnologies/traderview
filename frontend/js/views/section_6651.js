// IRC § 6651 — Failure to File / Failure to Pay Penalties.
// (a)(1) Failure to file: 5%/month of unpaid tax, max 25%. Min $485 (2024) if > 60 days late.
// (a)(2) Failure to pay: 0.5%/month of unpaid tax, max 25%.
// Combined: failure-to-file rate REDUCED to 4.5%/month when both apply.
// FRAUDULENT failure-to-file: 15%/month, max 75%.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const FTF_RATE = 0.05;
const FTF_FRAUDULENT_RATE = 0.15;
const FTP_RATE = 0.005;
const FTF_MAX = 0.25;
const FTF_FRAUDULENT_MAX = 0.75;
const MIN_LATE_FILING_2024 = 485;
const DEFAULT_INTEREST_RATE = 0.08;

let state = {
    tax_owed: 0,
    months_late_filing: 0,
    months_late_paying: 0,
    is_fraudulent_ftf: false,
    extension_filed: false,
    payment_plan_in_effect: false,
    first_time_abatement: false,
    reasonable_cause: false,
    is_60_plus_days_late: false,
};

export async function renderSection6651(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6651.h1.title">// § 6651 FAILURE TO FILE / PAY</span></h1>
        <p class="muted small" data-i18n="view.s6651.hint.intro">
            <strong>(a)(1) Failure to file:</strong> 5%/month of unpaid tax, max 25%. Min $485
            (2024) if &gt; 60 days late. <strong>(a)(2) Failure to pay:</strong> 0.5%/month, max
            25%. <strong>Combined: FTF rate REDUCED to 4.5%/month</strong> when both apply (so
            combined never exceeds 5%/month). <strong>FRAUDULENT FTF: 15%/month, max 75%</strong>.
            <strong>First-Time Abatement</strong> available for compliant taxpayers (penalty
            once / 3 yrs).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6651.h2.inputs">Inputs</h2>
            <form id="s6651-form" class="inline-form">
                <label><span data-i18n="view.s6651.label.tax">Tax owed ($)</span>
                    <input type="number" step="0.01" name="tax_owed" value="${state.tax_owed}"></label>
                <label><span data-i18n="view.s6651.label.ftf_months">Months late filing</span>
                    <input type="number" step="1" name="months_late_filing" value="${state.months_late_filing}"></label>
                <label><span data-i18n="view.s6651.label.ftp_months">Months late paying</span>
                    <input type="number" step="1" name="months_late_paying" value="${state.months_late_paying}"></label>
                <label><span data-i18n="view.s6651.label.fraudulent">Fraudulent FTF?</span>
                    <input type="checkbox" name="is_fraudulent_ftf" ${state.is_fraudulent_ftf ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6651.label.extension">Extension (Form 4868) filed?</span>
                    <input type="checkbox" name="extension_filed" ${state.extension_filed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6651.label.payment_plan">Payment plan in effect?</span>
                    <input type="checkbox" name="payment_plan_in_effect" ${state.payment_plan_in_effect ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6651.label.fta">First-time abatement eligible?</span>
                    <input type="checkbox" name="first_time_abatement" ${state.first_time_abatement ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6651.label.cause">Reasonable cause?</span>
                    <input type="checkbox" name="reasonable_cause" ${state.reasonable_cause ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6651.label.60_days">&gt; 60 days late?</span>
                    <input type="checkbox" name="is_60_plus_days_late" ${state.is_60_plus_days_late ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6651.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6651-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6651.h2.relief">Penalty relief options</h2>
            <ul class="muted small">
                <li data-i18n="view.s6651.rel.fta">First-Time Abatement: clean 3-yr record + currently filing/paying compliance</li>
                <li data-i18n="view.s6651.rel.reasonable">Reasonable cause: serious illness, natural disaster, records destroyed, IRS error</li>
                <li data-i18n="view.s6651.rel.combat">Combat zone § 7508 deferral: automatic + 180-day extension</li>
                <li data-i18n="view.s6651.rel.disaster">Federally-declared disaster: automatic extension per IRS announcement</li>
                <li data-i18n="view.s6651.rel.specific_situation">Specific situation-based: Form 843 written explanation</li>
                <li data-i18n="view.s6651.rel.ia_ftp">FTP rate halved (0.25%/mo) during installment agreement period</li>
                <li data-i18n="view.s6651.rel.no_fta_fraud">FTA + reasonable cause NOT available for fraudulent FTF</li>
                <li data-i18n="view.s6651.rel.ignorance">Ignorance of law NOT reasonable cause (United States v. Boyle 1985)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6651.h2.related">Related failure penalties</h2>
            <ul class="muted small">
                <li data-i18n="view.s6651.rel2.6654">§ 6654 Estimated tax underpayment (variable rate)</li>
                <li data-i18n="view.s6651.rel2.6651_d">§ 6651(d): rate INCREASED to 1%/month after IRS issues 10-day notice + levy threat</li>
                <li data-i18n="view.s6651.rel2.6651_h">§ 6651(h): rate REDUCED to 0.25%/month during installment agreement</li>
                <li data-i18n="view.s6651.rel2.6655">§ 6655 Corporate estimated tax underpayment</li>
                <li data-i18n="view.s6651.rel2.6656">§ 6656 Failure to deposit (employer trust fund taxes — 2-15% by lateness)</li>
                <li data-i18n="view.s6651.rel2.6672">§ 6672 Trust Fund Recovery Penalty (TFRP) — 100% of unpaid trust fund taxes</li>
                <li data-i18n="view.s6651.rel2.6721">§ 6721/6722 Failure to file/furnish information returns ($310/2024)</li>
            </ul>
        </div>
    `;
    document.getElementById('s6651-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.tax_owed = Number(fd.get('tax_owed')) || 0;
        state.months_late_filing = Number(fd.get('months_late_filing')) || 0;
        state.months_late_paying = Number(fd.get('months_late_paying')) || 0;
        state.is_fraudulent_ftf = !!fd.get('is_fraudulent_ftf');
        state.extension_filed = !!fd.get('extension_filed');
        state.payment_plan_in_effect = !!fd.get('payment_plan_in_effect');
        state.first_time_abatement = !!fd.get('first_time_abatement');
        state.reasonable_cause = !!fd.get('reasonable_cause');
        state.is_60_plus_days_late = !!fd.get('is_60_plus_days_late');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6651-output');
    if (!el) return;
    const ftpRate = state.payment_plan_in_effect ? FTP_RATE / 2 : FTP_RATE;
    const ftpPenalty = state.tax_owed * Math.min(ftpRate * state.months_late_paying, FTF_MAX);
    let ftfRate, ftfMax;
    if (state.is_fraudulent_ftf) {
        ftfRate = FTF_FRAUDULENT_RATE;
        ftfMax = FTF_FRAUDULENT_MAX;
    } else {
        const overlapping = Math.min(state.months_late_filing, state.months_late_paying);
        const onlyFtfMonths = state.months_late_filing - overlapping;
        const adjustedFtfRate = FTF_RATE - ftpRate;
        const cappedRate = Math.min(adjustedFtfRate * overlapping + FTF_RATE * onlyFtfMonths, FTF_MAX);
        ftfRate = adjustedFtfRate;
        ftfMax = FTF_MAX;
    }
    let ftfPenalty;
    if (state.is_fraudulent_ftf) {
        ftfPenalty = state.tax_owed * Math.min(FTF_FRAUDULENT_RATE * state.months_late_filing, FTF_FRAUDULENT_MAX);
    } else {
        const overlapping = Math.min(state.months_late_filing, state.months_late_paying);
        const onlyFtfMonths = Math.max(0, state.months_late_filing - state.months_late_paying);
        const overlappingPct = (FTF_RATE - ftpRate) * overlapping;
        const nonOverlappingPct = FTF_RATE * onlyFtfMonths;
        ftfPenalty = state.tax_owed * Math.min(overlappingPct + nonOverlappingPct, FTF_MAX);
    }
    const minLatePenalty = state.is_60_plus_days_late ? MIN_LATE_FILING_2024 : 0;
    const ftfPenaltyFinal = Math.max(ftfPenalty, ftfPenalty > 0 ? minLatePenalty : 0);
    const reliefApplies = (state.first_time_abatement || state.reasonable_cause) && !state.is_fraudulent_ftf;
    const finalFtf = reliefApplies ? 0 : ftfPenaltyFinal;
    const finalFtp = reliefApplies ? 0 : ftpPenalty;
    const interestEstimate = state.tax_owed * DEFAULT_INTEREST_RATE * (Math.max(state.months_late_filing, state.months_late_paying) / 12);
    const total = finalFtf + finalFtp + interestEstimate;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6651.h2.result">Penalty calculation</h2>
            <div class="cards">
                <div class="card neg">
                    <div class="label" data-i18n="view.s6651.card.ftf">Failure to File (a)(1)</div>
                    <div class="value">$${finalFtf.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6651.card.ftp">Failure to Pay (a)(2)</div>
                    <div class="value">$${finalFtp.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6651.card.interest">Interest (~8%)</div>
                    <div class="value">$${interestEstimate.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6651.card.total">Total exposure</div>
                    <div class="value">$${total.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${reliefApplies ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s6651.card.relief">Relief applied</div>
                    <div class="value">${reliefApplies ? esc(t('view.s6651.status.yes')) : esc(t('view.s6651.status.no'))}</div>
                </div>
            </div>
        </div>
    `;
}
