// IRC § 1041 — Spousal + Divorce Transfers (No Gain or Loss).
// Transfers between spouses (or incident to divorce within 1 yr / pursuant to divorce decree) = NO GAIN OR LOSS.
// Recipient takes CARRYOVER BASIS from transferor (same as gift, but no gift tax).
// Special rule for transfers to trust for benefit of spouse: § 1041 applies.
// US recipient + nonresident alien spouse: § 1041 does NOT apply (limited).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    asset_fmv: 0,
    transferor_basis: 0,
    transfer_type: 'spouse_living',
    is_incident_to_divorce: false,
    is_within_one_year: false,
    is_pursuant_to_divorce_decree: false,
    is_nonresident_alien_spouse: false,
    is_trust_for_spouse: false,
    qualifying_alimony_pre_2019: false,
    post_tcja_alimony: false,
    liabilities_assumed: 0,
    is_appreciated: true,
    holding_period_days: 0,
    later_sale_price: 0,
};

export async function renderSection1041(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s1041.h1.title">// § 1041 SPOUSE / DIVORCE</span></h1>
        <p class="muted small" data-i18n="view.s1041.hint.intro">
            Transfers between spouses + transfers <strong>incident to divorce</strong> (within 1 yr OR pursuant
            to divorce decree) = <strong>NO GAIN OR LOSS</strong>. <strong>Recipient takes CARRYOVER BASIS</strong>
            from transferor. <strong>Holding period TACKS.</strong> <strong>Trust for spouse</strong> also
            qualifies. <strong>EXCEPTION:</strong> § 1041 does NOT apply if recipient is <strong>nonresident
            alien</strong> spouse (NRA carve-out). <strong>Post-TCJA 2017:</strong> alimony NO LONGER deductible
            / includible (for divorces post-12/31/2018). <strong>QDRO</strong> (Qualified Domestic Relations
            Order) for retirement assets — separate rules.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s1041.h2.inputs">Inputs</h2>
            <form id="s1041-form" class="inline-form">
                <label><span data-i18n="view.s1041.label.fmv">Asset FMV ($)</span>
                    <input type="number" step="0.01" name="asset_fmv" value="${state.asset_fmv}"></label>
                <label><span data-i18n="view.s1041.label.basis">Transferor's basis ($)</span>
                    <input type="number" step="0.01" name="transferor_basis" value="${state.transferor_basis}"></label>
                <label><span data-i18n="view.s1041.label.type">Transfer type</span>
                    <select name="transfer_type">
                        <option value="spouse_living" ${state.transfer_type === 'spouse_living' ? 'selected' : ''}>Living spouse (intact marriage)</option>
                        <option value="divorce_decree" ${state.transfer_type === 'divorce_decree' ? 'selected' : ''}>Pursuant to divorce decree</option>
                        <option value="within_one_year" ${state.transfer_type === 'within_one_year' ? 'selected' : ''}>Within 1 yr of divorce</option>
                        <option value="trust_for_spouse" ${state.transfer_type === 'trust_for_spouse' ? 'selected' : ''}>Trust for spouse</option>
                        <option value="qdro" ${state.transfer_type === 'qdro' ? 'selected' : ''}>QDRO retirement assets</option>
                        <option value="property_settlement" ${state.transfer_type === 'property_settlement' ? 'selected' : ''}>Property settlement</option>
                    </select>
                </label>
                <label><span data-i18n="view.s1041.label.incident">Incident to divorce?</span>
                    <input type="checkbox" name="is_incident_to_divorce" ${state.is_incident_to_divorce ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1041.label.one_yr">Within 1 yr?</span>
                    <input type="checkbox" name="is_within_one_year" ${state.is_within_one_year ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1041.label.decree">Pursuant to decree?</span>
                    <input type="checkbox" name="is_pursuant_to_divorce_decree" ${state.is_pursuant_to_divorce_decree ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1041.label.nra">Nonresident alien spouse?</span>
                    <input type="checkbox" name="is_nonresident_alien_spouse" ${state.is_nonresident_alien_spouse ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1041.label.trust">Trust for spouse?</span>
                    <input type="checkbox" name="is_trust_for_spouse" ${state.is_trust_for_spouse ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1041.label.alimony_pre">Pre-2019 alimony (deductible / income)?</span>
                    <input type="checkbox" name="qualifying_alimony_pre_2019" ${state.qualifying_alimony_pre_2019 ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1041.label.alimony_post">Post-TCJA alimony (non-deductible)?</span>
                    <input type="checkbox" name="post_tcja_alimony" ${state.post_tcja_alimony ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1041.label.liab">Liabilities assumed by recipient ($)</span>
                    <input type="number" step="0.01" name="liabilities_assumed" value="${state.liabilities_assumed}"></label>
                <label><span data-i18n="view.s1041.label.appreciated">Appreciated asset?</span>
                    <input type="checkbox" name="is_appreciated" ${state.is_appreciated ? 'checked' : ''}></label>
                <label><span data-i18n="view.s1041.label.holding">Holding period transferred (days)</span>
                    <input type="number" step="1" name="holding_period_days" value="${state.holding_period_days}"></label>
                <label><span data-i18n="view.s1041.label.sale">Later sale price ($)</span>
                    <input type="number" step="0.01" name="later_sale_price" value="${state.later_sale_price}"></label>
                <button class="primary" type="submit" data-i18n="view.s1041.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s1041-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1041.h2.scope">§ 1041 scope</h2>
            <ul class="muted small">
                <li data-i18n="view.s1041.scope.spouses">Between spouses (intact marriage): always § 1041</li>
                <li data-i18n="view.s1041.scope.incident">"Incident to divorce": within 1 yr OR pursuant to decree</li>
                <li data-i18n="view.s1041.scope.6_year">6-year presumption: pursuant to decree if w/in 6 yrs</li>
                <li data-i18n="view.s1041.scope.beyond_6">Beyond 6 yrs: rebuttable presumption — must show divorce nexus</li>
                <li data-i18n="view.s1041.scope.trust">Transfer to trust for spouse's benefit: § 1041</li>
                <li data-i18n="view.s1041.scope.installment">Installment note: § 1041 applies to original transfer; § 453B(g) for note received</li>
                <li data-i18n="view.s1041.scope.assumption">Liabilities &gt; basis (negative basis): § 1041 still applies (no recharacterization)</li>
                <li data-i18n="view.s1041.scope.nra_exception">NONRESIDENT ALIEN spouse: § 1041 does NOT apply (gain recognized)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1041.h2.alimony_tcja">Alimony TCJA 2017 change</h2>
            <ul class="muted small">
                <li data-i18n="view.s1041.alm.pre_2019">Pre-12/31/2018 divorces: alimony DEDUCTIBLE to payor; INCLUDIBLE to payee</li>
                <li data-i18n="view.s1041.alm.post_tcja">Post-12/31/2018: alimony NEITHER deductible nor includible</li>
                <li data-i18n="view.s1041.alm.modification">Modification of pre-2019 decree: keep old rules UNLESS new agreement specifies new</li>
                <li data-i18n="view.s1041.alm.child_support">Child support: NEVER deductible to payor / income to payee</li>
                <li data-i18n="view.s1041.alm.property_settlement">Property settlement (§ 1041): always tax-neutral (no income / deduction)</li>
                <li data-i18n="view.s1041.alm.lump_sum">Lump-sum payments: scrutinized — § 71(c) recharacterizes if disguised property settlement</li>
                <li data-i18n="view.s1041.alm.retiree">Retiree planning: marital settlement around retirement assets via QDRO is critical</li>
                <li data-i18n="view.s1041.alm.state_law">State law: not all states followed TCJA — some allow state-level deduction</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1041.h2.special_assets">Special asset categories</h2>
            <ul class="muted small">
                <li data-i18n="view.s1041.spec.principal_residence">Principal residence: § 1041 + § 121 ($250K/$500K exclusion) interaction</li>
                <li data-i18n="view.s1041.spec.iras">IRAs: transfer pursuant to divorce decree NOT taxable; QDRO not for IRA (use trustee-to-trustee)</li>
                <li data-i18n="view.s1041.spec.401k">401(k) / qualified plans: QDRO required — TAXABLE if not properly transferred</li>
                <li data-i18n="view.s1041.spec.business_interests">Business interests: pre-divorce valuation often required + tax-free shifts to spouse</li>
                <li data-i18n="view.s1041.spec.s_corp_stock">S-corp stock: § 1041 ok; recipient becomes shareholder for pass-through</li>
                <li data-i18n="view.s1041.spec.installment_note">Installment notes: § 453B(g) — recipient takes carryover basis + no gain on transfer</li>
                <li data-i18n="view.s1041.spec.options">Stock options / RSU: vested options to ex-spouse via § 1041; tax due at exercise by recipient</li>
                <li data-i18n="view.s1041.spec.deferred_comp">Deferred comp / pension splits: QDRO required to avoid acceleration; § 409A trap</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s1041.h2.basis_holding">Basis + holding period mechanics</h2>
            <ul class="muted small">
                <li data-i18n="view.s1041.basis.carryover">Recipient takes CARRYOVER basis = transferor's adjusted basis</li>
                <li data-i18n="view.s1041.basis.no_step_up">NO step-up to FMV (contrast § 1014 death)</li>
                <li data-i18n="view.s1041.basis.holding_tacks">Holding period TACKS from transferor's</li>
                <li data-i18n="view.s1041.basis.no_dual">NO dual basis (contrast § 1015 gift rule for depreciated property)</li>
                <li data-i18n="view.s1041.basis.gift_tax_inapplicable">NO gift tax adjustment (no gift tax paid since spousal transfer)</li>
                <li data-i18n="view.s1041.basis.liabilities">Liabilities assumed by recipient REDUCE basis (Crane rule)</li>
                <li data-i18n="view.s1041.basis.nondeductible">No transferor loss recognition even if FMV &lt; basis</li>
                <li data-i18n="view.s1041.basis.later_sale">Later sale by recipient: gain / loss based on carryover basis</li>
            </ul>
        </div>
    `;
    document.getElementById('s1041-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.asset_fmv = Number(fd.get('asset_fmv')) || 0;
        state.transferor_basis = Number(fd.get('transferor_basis')) || 0;
        state.transfer_type = fd.get('transfer_type');
        state.is_incident_to_divorce = !!fd.get('is_incident_to_divorce');
        state.is_within_one_year = !!fd.get('is_within_one_year');
        state.is_pursuant_to_divorce_decree = !!fd.get('is_pursuant_to_divorce_decree');
        state.is_nonresident_alien_spouse = !!fd.get('is_nonresident_alien_spouse');
        state.is_trust_for_spouse = !!fd.get('is_trust_for_spouse');
        state.qualifying_alimony_pre_2019 = !!fd.get('qualifying_alimony_pre_2019');
        state.post_tcja_alimony = !!fd.get('post_tcja_alimony');
        state.liabilities_assumed = Number(fd.get('liabilities_assumed')) || 0;
        state.is_appreciated = !!fd.get('is_appreciated');
        state.holding_period_days = Number(fd.get('holding_period_days')) || 0;
        state.later_sale_price = Number(fd.get('later_sale_price')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s1041-output');
    if (!el) return;
    const qualifies = !state.is_nonresident_alien_spouse;
    const builtInGain = Math.max(0, state.asset_fmv - state.transferor_basis);
    const gainAvoided = qualifies ? builtInGain : 0;
    const taxSavedTransfer = gainAvoided * 0.20;
    const recipientBasis = state.transferor_basis - state.liabilities_assumed;
    const laterGain = Math.max(0, state.later_sale_price - recipientBasis);
    const laterTax = laterGain * 0.20;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s1041.h2.result">§ 1041 outcome</h2>
            <div class="cards">
                <div class="card ${qualifies ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s1041.card.qualifies">§ 1041 qualifies?</div>
                    <div class="value">${qualifies ? esc(t('view.s1041.status.yes')) : esc(t('view.s1041.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1041.card.builtin">Built-in gain</div>
                    <div class="value">$${builtInGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1041.card.gain_avoided">Gain deferred at transfer</div>
                    <div class="value">$${gainAvoided.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s1041.card.tax_saved">Transfer tax saved (20%)</div>
                    <div class="value">$${taxSavedTransfer.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s1041.card.recipient_basis">Recipient's basis</div>
                    <div class="value">$${recipientBasis.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1041.card.later_gain">Later sale gain (recipient)</div>
                    <div class="value">$${laterGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s1041.card.later_tax">Later sale tax (20%)</div>
                    <div class="value">$${laterTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.is_nonresident_alien_spouse ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s1041.nra_note">
                    NONRESIDENT ALIEN spouse: § 1041 does NOT apply. Transferor recognizes FULL gain/loss
                    on transfer at FMV. Critical planning point — NRA spouse acquires basis = FMV but
                    transferor pays tax now. Consider NRA election to be treated as US resident (§ 6013(g))
                    to access § 1041, but creates US-tax on worldwide income for NRA spouse.
                </p>
            ` : ''}
        </div>
    `;
}
