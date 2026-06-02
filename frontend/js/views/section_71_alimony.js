// IRC § 71 / § 215 (Alimony) + § 1041 (Spousal Transfers).
// TCJA 2018 ELIMINATED § 71 alimony income + § 215 alimony deduction for divorces / modifications
// AFTER 12/31/2018. Pre-2019 divorces: rules grandfathered unless modified.
// § 1041: spousal property transfers are NOT taxable (basis carries over).
// § 121: home sale exclusion preserved if you used as residence in last 5 yrs.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    divorce_year: 2025,
    is_modified_post_2018: false,
    is_modification_specifies_old_rule: false,
    payer_role: 'payer',
    annual_alimony: 0,
    payer_marginal_rate: 0.32,
    recipient_marginal_rate: 0.22,
    transfer_property_value: 0,
    transfer_property_basis: 0,
    home_used_as_residence_2_of_5: false,
};

export async function renderSection71Alimony(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.alimony.h1.title">// § 71 / § 1041 ALIMONY + TRANSFERS</span></h1>
        <p class="muted small" data-i18n="view.alimony.hint.intro">
            <strong>TCJA 2018 ELIMINATED</strong> § 71 alimony income + § 215 alimony deduction
            for divorces / modifications AFTER <strong>12/31/2018</strong>. Pre-2019 divorces:
            grandfathered unless modification explicitly removes old rules.
            <strong>§ 1041:</strong> spousal property transfers NOT taxable (basis carries over).
            <strong>§ 121:</strong> home sale exclusion preserved if used as residence 2 of last 5 yrs.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.alimony.h2.inputs">Inputs</h2>
            <form id="alimony-form" class="inline-form">
                <label><span data-i18n="view.alimony.label.divorce_year">Divorce year</span>
                    <input type="number" step="1" name="divorce_year" value="${state.divorce_year}"></label>
                <label><span data-i18n="view.alimony.label.modified_post_2018">Modified after 2018?</span>
                    <input type="checkbox" name="is_modified_post_2018" ${state.is_modified_post_2018 ? 'checked' : ''}></label>
                <label><span data-i18n="view.alimony.label.specifies_old">Modification specifies OLD rule applies?</span>
                    <input type="checkbox" name="is_modification_specifies_old_rule" ${state.is_modification_specifies_old_rule ? 'checked' : ''}></label>
                <label><span data-i18n="view.alimony.label.role">Your role</span>
                    <select name="payer_role">
                        <option value="payer" ${state.payer_role === 'payer' ? 'selected' : ''}>Payer</option>
                        <option value="recipient" ${state.payer_role === 'recipient' ? 'selected' : ''}>Recipient</option>
                    </select>
                </label>
                <label><span data-i18n="view.alimony.label.annual">Annual alimony ($)</span>
                    <input type="number" step="1000" name="annual_alimony" value="${state.annual_alimony}"></label>
                <label><span data-i18n="view.alimony.label.payer_rate">Payer marginal %</span>
                    <input type="number" step="0.01" name="payer_marginal_rate" value="${state.payer_marginal_rate}"></label>
                <label><span data-i18n="view.alimony.label.recipient_rate">Recipient marginal %</span>
                    <input type="number" step="0.01" name="recipient_marginal_rate" value="${state.recipient_marginal_rate}"></label>
                <hr style="grid-column:1/-1">
                <label><span data-i18n="view.alimony.label.property_value">Spousal transfer FMV ($)</span>
                    <input type="number" step="10000" name="transfer_property_value" value="${state.transfer_property_value}"></label>
                <label><span data-i18n="view.alimony.label.property_basis">Property basis ($)</span>
                    <input type="number" step="10000" name="transfer_property_basis" value="${state.transfer_property_basis}"></label>
                <label><span data-i18n="view.alimony.label.home_residence">Used home as principal residence 2 of last 5 yrs?</span>
                    <input type="checkbox" name="home_used_as_residence_2_of_5" ${state.home_used_as_residence_2_of_5 ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.alimony.btn.compute">Compute</button>
            </form>
        </div>
        <div id="alimony-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.alimony.h2.tcja_changes">TCJA 2018 changes</h2>
            <ul class="muted small">
                <li data-i18n="view.alimony.tcja.eliminated">Eliminated § 71 alimony income + § 215 deduction for post-2018 divorces</li>
                <li data-i18n="view.alimony.tcja.recipient_no_tax">Recipient no longer reports alimony as income (good for them)</li>
                <li data-i18n="view.alimony.tcja.payer_no_deduct">Payer cannot deduct alimony (bad for them)</li>
                <li data-i18n="view.alimony.tcja.savings_negotiation">Eliminates ~$10B/yr tax expenditure benefit</li>
                <li data-i18n="view.alimony.tcja.pre_2019_grandfather">Pre-2019 divorces: grandfathered unless modification explicitly opts in to new rules</li>
                <li data-i18n="view.alimony.tcja.no_state">State income tax may differ (varies by state)</li>
                <li data-i18n="view.alimony.tcja.section_682">§ 682 grantor trust alimony rules also repealed</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.alimony.h2.section_1041">§ 1041 spousal transfer rules</h2>
            <ul class="muted small">
                <li data-i18n="view.alimony.s1041.no_gain">Transfer between spouses incident to divorce: NO gain / loss</li>
                <li data-i18n="view.alimony.s1041.carryover_basis">Recipient takes transferor's basis (carryover)</li>
                <li data-i18n="view.alimony.s1041.holding_tacks">Holding period TACKS</li>
                <li data-i18n="view.alimony.s1041.appreciated">Transfer of appreciated property → recipient owes tax on sale (latent gain)</li>
                <li data-i18n="view.alimony.s1041.timing">Within 1 yr of divorce: automatic; up to 6 yrs if "incident to divorce"</li>
                <li data-i18n="view.alimony.s1041.installment">§ 453 installment sale to ex-spouse generally taxable to seller</li>
                <li data-i18n="view.alimony.s1041.qdros">QDROs distribute 401(k) tax-free if direct rollover</li>
                <li data-i18n="view.alimony.s1041.foreign">Non-citizen spouse: § 1041 doesn't apply → gain recognized</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.alimony.h2.child_support">Child support vs alimony</h2>
            <p class="muted small" data-i18n="view.alimony.cs.body">
                <strong>Child support is NEVER taxable</strong> to recipient + NEVER deductible by
                payer (both pre- and post-TCJA). § 71(c) anti-recharacterization rules prevent
                disguising alimony as child support to evade. Watch for "front-loading recapture"
                under § 71(f) — if amounts decrease &gt; $15k/yr in first 3 years, payer recaptures.
            </p>
        </div>
    `;
    document.getElementById('alimony-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.divorce_year = Number(fd.get('divorce_year')) || 2025;
        state.is_modified_post_2018 = !!fd.get('is_modified_post_2018');
        state.is_modification_specifies_old_rule = !!fd.get('is_modification_specifies_old_rule');
        state.payer_role = fd.get('payer_role');
        state.annual_alimony = Number(fd.get('annual_alimony')) || 0;
        state.payer_marginal_rate = Number(fd.get('payer_marginal_rate')) || 0.32;
        state.recipient_marginal_rate = Number(fd.get('recipient_marginal_rate')) || 0.22;
        state.transfer_property_value = Number(fd.get('transfer_property_value')) || 0;
        state.transfer_property_basis = Number(fd.get('transfer_property_basis')) || 0;
        state.home_used_as_residence_2_of_5 = !!fd.get('home_used_as_residence_2_of_5');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('alimony-output');
    if (!el) return;
    const oldRulesApply = state.divorce_year <= 2018 && (!state.is_modified_post_2018 || state.is_modification_specifies_old_rule);
    const payerDeductible = oldRulesApply;
    const recipientTaxable = oldRulesApply;
    const payerSavings = payerDeductible ? state.annual_alimony * state.payer_marginal_rate : 0;
    const recipientTax = recipientTaxable ? state.annual_alimony * state.recipient_marginal_rate : 0;
    const collectiveTaxAdvantage = payerSavings - recipientTax;
    const latentGain = Math.max(0, state.transfer_property_value - state.transfer_property_basis);
    const latentTaxToRecipient = latentGain * 0.20;  // assume LT cap gain
    const recipientNetReceived = state.transfer_property_value - latentTaxToRecipient;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.alimony.h2.result">Tax outcome</h2>
            <div class="cards">
                <div class="card ${oldRulesApply ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.alimony.card.old_rules">Pre-TCJA rules apply</div>
                    <div class="value">${oldRulesApply ? esc(t('view.alimony.status.yes')) : esc(t('view.alimony.status.no'))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.alimony.card.payer_savings">Payer tax savings</div>
                    <div class="value">$${payerSavings.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.alimony.card.recipient_tax">Recipient tax owed</div>
                    <div class="value">$${recipientTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.alimony.card.collective">Collective tax advantage</div>
                    <div class="value">$${collectiveTaxAdvantage.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.alimony.card.latent_gain">Latent gain in property</div>
                    <div class="value">$${latentGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.alimony.card.recipient_net">Recipient net (after latent tax)</div>
                    <div class="value">$${recipientNetReceived.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
