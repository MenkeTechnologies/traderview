// IRC § 6721 / § 6722 / § 6723 / § 6724 — Information Return Penalties + Waivers.
// § 6721 Failure to file correct info return ($310/2024, $3.78M cap annual).
// § 6722 Failure to furnish correct payee statement ($310/2024 same cap).
// § 6723 Failure to comply with other info reporting ($60/failure, $250 cap).
// § 6724 Reasonable cause waiver. De minimis: 10 or 0.5% of forms cured by Aug 1.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const PENALTY_TIER_30_DAYS = 60;
const PENALTY_TIER_BY_AUG_1 = 130;
const PENALTY_TIER_AFTER_AUG_1 = 310;
const PENALTY_INTENTIONAL = 630;
const PENALTY_SMALL_BIZ_CAP = 1_261_000;
const PENALTY_LARGE_CAP = 3_780_000;
const PENALTY_INTENTIONAL_CAP = Number.POSITIVE_INFINITY;

let state = {
    return_kind: 'w2',
    total_forms: 0,
    days_late: 0,
    cured_within_30_days: 0,
    cured_by_aug_1: 0,
    not_filed_or_late_after_aug_1: 0,
    intentional_disregard: 0,
    is_small_business: false,
    has_reasonable_cause: false,
    de_minimis_safe_harbor: false,
};

export async function renderSection6724(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6724.h1.title">// § 6721/22/24 INFO RETURN PENALTIES</span></h1>
        <p class="muted small" data-i18n="view.s6724.hint.intro">
            <strong>§ 6721 Failure to file</strong> + <strong>§ 6722 Failure to furnish payee</strong>
            stack. Penalties: <strong>$60/$130/$310 per return</strong> by tier (cured within 30 days / by Aug 1 / after). $630 intentional. Annual caps:
            <strong>$1.26M small biz / $3.78M large</strong>. <strong>§ 6724 reasonable cause waiver</strong>
            + <strong>De Minimis safe harbor</strong> (10 or 0.5% of forms cured by Aug 1, whichever less).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6724.h2.inputs">Inputs</h2>
            <form id="s6724-form" class="inline-form">
                <label><span data-i18n="view.s6724.label.kind">Return kind</span>
                    <select name="return_kind">
                        <option value="w2" ${state.return_kind === 'w2' ? 'selected' : ''}>W-2 (wages)</option>
                        <option value="1099_nec" ${state.return_kind === '1099_nec' ? 'selected' : ''}>1099-NEC (contractor)</option>
                        <option value="1099_misc" ${state.return_kind === '1099_misc' ? 'selected' : ''}>1099-MISC (rents, royalties, etc.)</option>
                        <option value="1099_div" ${state.return_kind === '1099_div' ? 'selected' : ''}>1099-DIV (dividends)</option>
                        <option value="1099_int" ${state.return_kind === '1099_int' ? 'selected' : ''}>1099-INT (interest)</option>
                        <option value="1099_b" ${state.return_kind === '1099_b' ? 'selected' : ''}>1099-B (broker)</option>
                        <option value="1099_k" ${state.return_kind === '1099_k' ? 'selected' : ''}>1099-K (payment apps)</option>
                        <option value="1099_r" ${state.return_kind === '1099_r' ? 'selected' : ''}>1099-R (retirement distrib)</option>
                        <option value="1095_c" ${state.return_kind === '1095_c' ? 'selected' : ''}>1095-C (ACA employer)</option>
                        <option value="other" ${state.return_kind === 'other' ? 'selected' : ''}>Other info return</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6724.label.total">Total forms required</span>
                    <input type="number" step="1" name="total_forms" value="${state.total_forms}"></label>
                <label><span data-i18n="view.s6724.label.cured_30">Cured within 30 days</span>
                    <input type="number" step="1" name="cured_within_30_days" value="${state.cured_within_30_days}"></label>
                <label><span data-i18n="view.s6724.label.cured_aug">Cured by Aug 1</span>
                    <input type="number" step="1" name="cured_by_aug_1" value="${state.cured_by_aug_1}"></label>
                <label><span data-i18n="view.s6724.label.after_aug">After Aug 1 / unfiled</span>
                    <input type="number" step="1" name="not_filed_or_late_after_aug_1" value="${state.not_filed_or_late_after_aug_1}"></label>
                <label><span data-i18n="view.s6724.label.intentional">Intentional disregard count</span>
                    <input type="number" step="1" name="intentional_disregard" value="${state.intentional_disregard}"></label>
                <label><span data-i18n="view.s6724.label.small_biz">Small business (avg gross receipts ≤ $5M)?</span>
                    <input type="checkbox" name="is_small_business" ${state.is_small_business ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6724.label.cause">Reasonable cause claim?</span>
                    <input type="checkbox" name="has_reasonable_cause" ${state.has_reasonable_cause ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6724.label.de_minimis">De minimis safe harbor met?</span>
                    <input type="checkbox" name="de_minimis_safe_harbor" ${state.de_minimis_safe_harbor ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s6724.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6724-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6724.h2.de_minimis">§ 6724(c) De Minimis Safe Harbor</h2>
            <p class="muted small" data-i18n="view.s6724.de_minimis.body">
                <strong>10 returns OR 0.5% of total, whichever is less</strong>, with INCORRECT
                information are PENALTY-FREE if cured by August 1. Excellent reason to do
                error-correcting Form W-2c / 1099-CORR by July 31. Doesn't apply to "intentional
                disregard" failures.
            </p>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6724.h2.reasonable_cause">§ 6724 Reasonable cause waiver</h2>
            <ul class="muted small">
                <li data-i18n="view.s6724.rc.significant_mitigating">Significant mitigating factors despite normal business care</li>
                <li data-i18n="view.s6724.rc.events_beyond_control">Events beyond filer's control (records destroyed, key employee disabled, etc.)</li>
                <li data-i18n="view.s6724.rc.acted_quickly">Acted quickly to correct upon discovery</li>
                <li data-i18n="view.s6724.rc.steps_to_avoid">Steps to avoid recurrence</li>
                <li data-i18n="view.s6724.rc.history">Prior compliance history</li>
                <li data-i18n="view.s6724.rc.reasonable_basis">Reasonable interpretation of ambiguous law</li>
                <li data-i18n="view.s6724.rc.no_warning">No advance IRS warning of the issue</li>
                <li data-i18n="view.s6724.rc.system_software">Software / system failure beyond filer's control</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6724.h2.related_penalties">Related penalties</h2>
            <ul class="muted small">
                <li data-i18n="view.s6724.rel.6651">§ 6651 Failure to file / pay TAX returns (separate from info returns)</li>
                <li data-i18n="view.s6724.rel.6672">§ 6672 Trust Fund Recovery Penalty (TFRP): 100% on unpaid trust fund taxes</li>
                <li data-i18n="view.s6724.rel.6723">§ 6723 Failure to comply with other info reporting ($60/failure)</li>
                <li data-i18n="view.s6724.rel.6722">§ 6722 Failure to furnish CORRECT payee statement to recipient</li>
                <li data-i18n="view.s6724.rel.intentional">Intentional disregard penalty: $630/return or 10% of amount required</li>
                <li data-i18n="view.s6724.rel.no_ssns">Failure to include correct SSNs: $310/failure</li>
                <li data-i18n="view.s6724.rel.electronic">Electronic filing requirement (≥ 10 returns 2024): $310/failure if paper instead</li>
                <li data-i18n="view.s6724.rel.payee_w9">Failure to obtain W-9 backup withholding $310/failure</li>
            </ul>
        </div>
    `;
    document.getElementById('s6724-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.return_kind = fd.get('return_kind');
        state.total_forms = Number(fd.get('total_forms')) || 0;
        state.cured_within_30_days = Number(fd.get('cured_within_30_days')) || 0;
        state.cured_by_aug_1 = Number(fd.get('cured_by_aug_1')) || 0;
        state.not_filed_or_late_after_aug_1 = Number(fd.get('not_filed_or_late_after_aug_1')) || 0;
        state.intentional_disregard = Number(fd.get('intentional_disregard')) || 0;
        state.is_small_business = !!fd.get('is_small_business');
        state.has_reasonable_cause = !!fd.get('has_reasonable_cause');
        state.de_minimis_safe_harbor = !!fd.get('de_minimis_safe_harbor');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6724-output');
    if (!el) return;
    const grossPenalty =
        state.cured_within_30_days * PENALTY_TIER_30_DAYS
        + state.cured_by_aug_1 * PENALTY_TIER_BY_AUG_1
        + state.not_filed_or_late_after_aug_1 * PENALTY_TIER_AFTER_AUG_1
        + state.intentional_disregard * PENALTY_INTENTIONAL;
    const cap = state.is_small_business ? PENALTY_SMALL_BIZ_CAP : PENALTY_LARGE_CAP;
    const cappedPenalty = state.intentional_disregard > 0
        ? grossPenalty
        : Math.min(grossPenalty, cap);
    const deMinimisRelief = state.de_minimis_safe_harbor
        ? Math.min(10, state.total_forms * 0.005) * PENALTY_TIER_AFTER_AUG_1
        : 0;
    const reasonableCauseRelief = state.has_reasonable_cause ? cappedPenalty * 0.5 : 0;
    const finalPenalty = Math.max(0, cappedPenalty - deMinimisRelief - reasonableCauseRelief);
    const stackedFor6722 = finalPenalty;  // § 6722 stacks
    const totalBothSections = finalPenalty + stackedFor6722;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6724.h2.result">Penalty calculation</h2>
            <div class="cards">
                <div class="card neg">
                    <div class="label" data-i18n="view.s6724.card.gross">Gross § 6721 penalty</div>
                    <div class="value">$${grossPenalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6724.card.cap">Annual cap</div>
                    <div class="value">$${cap.toLocaleString()}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s6724.card.de_minimis">De minimis relief</div>
                    <div class="value">$${deMinimisRelief.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s6724.card.rc">Reasonable cause relief</div>
                    <div class="value">$${reasonableCauseRelief.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6724.card.final_6721">Final § 6721 (filing)</div>
                    <div class="value">$${finalPenalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6724.card.6722">§ 6722 stacked (payee)</div>
                    <div class="value">$${stackedFor6722.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6724.card.total">Total exposure</div>
                    <div class="value">$${totalBothSections.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
