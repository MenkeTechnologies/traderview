// IRC § 332 — Tax-Free Subsidiary Liquidation (≥ 80% Parent).
// Parent recognizes NO gain / loss on receipt of subsidiary's assets.
// Subsidiary recognizes NO gain / loss on distribution to parent (§ 337).
// Parent takes subsidiary's BASIS (§ 334(b)) — no step-up.
// Subsidiary's E&P + tax attributes (NOL, credits) carry over (§ 381).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    parent_ownership_pct: 0,
    subsidiary_fmv: 0,
    subsidiary_inside_basis: 0,
    parent_outside_basis: 0,
    sub_nol_carryforward: 0,
    sub_ep_accumulated: 0,
    sub_ftc_carryforward: 0,
    minority_shareholders: false,
    minority_amount: 0,
    insolvent_subsidiary: false,
    plan_adopted: true,
    completed_within_one_year: true,
};

export async function renderSection332(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s332.h1.title">// § 332 SUB LIQUIDATION (TAX-FREE)</span></h1>
        <p class="muted small" data-i18n="view.s332.hint.intro">
            <strong>§ 332:</strong> Parent corp recognizes NO gain / loss on liquidation of ≥ 80%-owned
            subsidiary. <strong>§ 337:</strong> Sub recognizes NO gain / loss on distribution to ≥ 80% parent.
            <strong>§ 334(b):</strong> Parent inherits SUB's INSIDE BASIS in assets — NO step-up. <strong>§ 381:</strong>
            Parent inherits NOL, E&P, credits, methods. Requires PLAN of liquidation + complete within ONE TAX
            YEAR (or 3 years per IRS approval). Minority shareholders: § 331 normal taxable. <strong>Insolvent
            sub:</strong> § 332 does NOT apply (parent recognizes loss under § 165).
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s332.h2.inputs">Inputs</h2>
            <form id="s332-form" class="inline-form">
                <label><span data-i18n="view.s332.label.ownership">Parent ownership %</span>
                    <input type="number" step="0.01" name="parent_ownership_pct" value="${state.parent_ownership_pct}"></label>
                <label><span data-i18n="view.s332.label.fmv">Subsidiary FMV ($)</span>
                    <input type="number" step="0.01" name="subsidiary_fmv" value="${state.subsidiary_fmv}"></label>
                <label><span data-i18n="view.s332.label.inside">Subsidiary inside basis ($)</span>
                    <input type="number" step="0.01" name="subsidiary_inside_basis" value="${state.subsidiary_inside_basis}"></label>
                <label><span data-i18n="view.s332.label.outside">Parent outside basis in sub stock ($)</span>
                    <input type="number" step="0.01" name="parent_outside_basis" value="${state.parent_outside_basis}"></label>
                <label><span data-i18n="view.s332.label.nol">Sub NOL carryforward ($)</span>
                    <input type="number" step="0.01" name="sub_nol_carryforward" value="${state.sub_nol_carryforward}"></label>
                <label><span data-i18n="view.s332.label.ep">Sub E&P accumulated ($)</span>
                    <input type="number" step="0.01" name="sub_ep_accumulated" value="${state.sub_ep_accumulated}"></label>
                <label><span data-i18n="view.s332.label.ftc">Sub FTC carryforward ($)</span>
                    <input type="number" step="0.01" name="sub_ftc_carryforward" value="${state.sub_ftc_carryforward}"></label>
                <label><span data-i18n="view.s332.label.minority">Minority shareholders?</span>
                    <input type="checkbox" name="minority_shareholders" ${state.minority_shareholders ? 'checked' : ''}></label>
                <label><span data-i18n="view.s332.label.minority_amt">Minority share amount ($)</span>
                    <input type="number" step="0.01" name="minority_amount" value="${state.minority_amount}"></label>
                <label><span data-i18n="view.s332.label.insolvent">Sub insolvent (assets &lt; liab)?</span>
                    <input type="checkbox" name="insolvent_subsidiary" ${state.insolvent_subsidiary ? 'checked' : ''}></label>
                <label><span data-i18n="view.s332.label.plan">Plan of liquidation adopted?</span>
                    <input type="checkbox" name="plan_adopted" ${state.plan_adopted ? 'checked' : ''}></label>
                <label><span data-i18n="view.s332.label.complete">Completed within 1 yr?</span>
                    <input type="checkbox" name="completed_within_one_year" ${state.completed_within_one_year ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s332.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s332-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s332.h2.requirements">§ 332 requirements</h2>
            <ol class="muted small">
                <li data-i18n="view.s332.req.ownership">Parent corp owns ≥ 80% of sub stock by vote AND value (§ 1504(a)(2))</li>
                <li data-i18n="view.s332.req.plan">Plan of liquidation adopted (formal corporate action)</li>
                <li data-i18n="view.s332.req.timing">Complete within 1 tax year of plan, OR up to 3 years w/ approval § 332(b)(3)</li>
                <li data-i18n="view.s332.req.full_payment">Property distributed in complete cancellation of stock</li>
                <li data-i18n="view.s332.req.cessation">Sub ceases to exist for tax purposes (often state-law dissolution)</li>
                <li data-i18n="view.s332.req.solvent">Sub must be solvent (assets ≥ liabilities)</li>
                <li data-i18n="view.s332.req.minority_taxable">Minority shareholders (&lt; 20%) get § 331 fully taxable treatment</li>
                <li data-i18n="view.s332.req.foreign_sub">Foreign sub: § 367(b) regulations apply — may trigger inclusion of all E&P</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s332.h2.s381">§ 381 carryover items</h2>
            <ul class="muted small">
                <li data-i18n="view.s332.s381.nol">NOL: parent inherits, subject to § 382 limitation if ownership change in sub</li>
                <li data-i18n="view.s332.s381.cap_loss">Capital loss carryover</li>
                <li data-i18n="view.s332.s381.ep">E&P (positive AND deficit)</li>
                <li data-i18n="view.s332.s381.methods">Accounting methods + inventory + installment method</li>
                <li data-i18n="view.s332.s381.cred">Tax credits (FTC, GBC, etc.)</li>
                <li data-i18n="view.s332.s381.depreciation">Depreciation schedules</li>
                <li data-i18n="view.s332.s381.deductions">Various items (charitable, capital exp recovery)</li>
                <li data-i18n="view.s332.s381.exception">SRLY rules limit pre-acq NOL use against post-acq income (cons. ret. only)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s332.h2.insolvent">Insolvent subsidiary path (§ 165 / § 1504(a)(2))</h2>
            <ul class="muted small">
                <li data-i18n="view.s332.insol.no_332">§ 332 NOT available if sub insolvent (no property distributed for stock)</li>
                <li data-i18n="view.s332.insol.s165">Parent recognizes worthless stock loss under § 165(g)(1) — ordinary if affiliated</li>
                <li data-i18n="view.s332.insol.s165_g3">§ 165(g)(3): affiliated group, ordinary loss treatment</li>
                <li data-i18n="view.s332.insol.no_337">Sub still recognizes gain / loss on asset distribution (§ 337 not avail)</li>
                <li data-i18n="view.s332.insol.no_381">NO § 381 carryover — sub attributes lost (NOL, E&P, credits)</li>
                <li data-i18n="view.s332.insol.timing">Worthlessness identified — usually formal dissolution or bankruptcy</li>
                <li data-i18n="view.s332.insol.tradeoff">Tradeoff: parent gets ordinary loss BUT sub's attributes lost forever</li>
            </ul>
        </div>
    `;
    document.getElementById('s332-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.parent_ownership_pct = Number(fd.get('parent_ownership_pct')) || 0;
        state.subsidiary_fmv = Number(fd.get('subsidiary_fmv')) || 0;
        state.subsidiary_inside_basis = Number(fd.get('subsidiary_inside_basis')) || 0;
        state.parent_outside_basis = Number(fd.get('parent_outside_basis')) || 0;
        state.sub_nol_carryforward = Number(fd.get('sub_nol_carryforward')) || 0;
        state.sub_ep_accumulated = Number(fd.get('sub_ep_accumulated')) || 0;
        state.sub_ftc_carryforward = Number(fd.get('sub_ftc_carryforward')) || 0;
        state.minority_shareholders = !!fd.get('minority_shareholders');
        state.minority_amount = Number(fd.get('minority_amount')) || 0;
        state.insolvent_subsidiary = !!fd.get('insolvent_subsidiary');
        state.plan_adopted = !!fd.get('plan_adopted');
        state.completed_within_one_year = !!fd.get('completed_within_one_year');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s332-output');
    if (!el) return;
    const ownershipMet = state.parent_ownership_pct >= 80;
    const qualifies = ownershipMet && state.plan_adopted && state.completed_within_one_year && !state.insolvent_subsidiary;
    const wouldBeGain = Math.max(0, state.subsidiary_fmv - state.parent_outside_basis);
    const wouldBeSubGain = Math.max(0, state.subsidiary_fmv - state.subsidiary_inside_basis);
    const inheritedAttributes = state.sub_nol_carryforward + state.sub_ftc_carryforward;
    const taxSavingsOnParent = qualifies ? wouldBeGain * 0.21 : 0;
    const taxSavingsOnSub = qualifies ? wouldBeSubGain * 0.21 : 0;
    const insolventLoss = state.insolvent_subsidiary ? state.parent_outside_basis : 0;
    const insolventTaxBenefit = insolventLoss * 0.21;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s332.h2.result">§ 332 outcome</h2>
            <div class="cards">
                <div class="card ${qualifies ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s332.card.qualifies">§ 332 qualifies?</div>
                    <div class="value">${qualifies ? esc(t('view.s332.status.yes')) : esc(t('view.s332.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s332.card.ownership_test">≥ 80% ownership</div>
                    <div class="value">${ownershipMet ? esc(t('view.s332.status.yes')) : esc(t('view.s332.status.no'))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s332.card.parent_savings">Parent tax avoided (21%)</div>
                    <div class="value">$${taxSavingsOnParent.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s332.card.sub_savings">Sub tax avoided § 337 (21%)</div>
                    <div class="value">$${taxSavingsOnSub.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s332.card.inherited">Attributes inherited</div>
                    <div class="value">$${inheritedAttributes.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${state.insolvent_subsidiary ? `
                    <div class="card pos">
                        <div class="label" data-i18n="view.s332.card.insolvent_loss">§ 165 worthless stock loss</div>
                        <div class="value">$${insolventLoss.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                    <div class="card pos">
                        <div class="label" data-i18n="view.s332.card.insolvent_benefit">Insolvent tax benefit (21%)</div>
                        <div class="value">$${insolventTaxBenefit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
            </div>
            ${qualifies && state.minority_shareholders ? `
                <p class="muted small" style="margin-top:10px" data-i18n="view.s332.minority_note">
                    Minority shareholders (&lt; 20%) DO NOT get § 332 — they recognize gain / loss under § 331
                    (sale or exchange treatment). Plan for separate tax computation at minority level.
                </p>
            ` : ''}
        </div>
    `;
}
