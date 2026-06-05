// IRC § 367(a) — Outbound Transfer of TANGIBLE Property.
// US transferor recognizes GAIN on outbound transfer of property to foreign corp.
// § 367(a)(1) general rule: foreign transferee treated as NOT a corp → no § 351 deferral.
// § 367(a)(3) ACTIVE business exception (HISTORICAL, TCJA 2017 narrowed scope).
// Compare § 367(d) IP outbound — annual deemed royalty model (different mechanism).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    property_fmv: 0,
    transferor_basis: 0,
    type_property: 'tangible',
    is_inventory: false,
    is_depreciable_us_use: false,
    is_foreign_use_active: false,
    is_resale_outside_us: false,
    is_resale_in_us: false,
    transferee_country: 'cayman',
    is_cfc_recipient: true,
    domestic_transferor_corp: true,
    pre_tcja_election: false,
    gain_recognition_agreement: false,
    s367a3_active_business: false,
    s6038b_filed: false,
};

export async function renderSection367A(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s367a.h1.title">// § 367(a) OUTBOUND TANGIBLE</span></h1>
        <p class="muted small" data-i18n="view.s367a.hint.intro">
            US transferor recognizes <strong>GAIN</strong> on outbound transfer of property to foreign corp.
            <strong>§ 367(a)(1) general rule:</strong> foreign transferee treated as NOT a corp → no § 351
            deferral. <strong>§ 367(a)(3) ACTIVE business exception</strong> (HISTORICAL, TCJA 2017 NARROWED
            scope — for ACTIVE TRADE / BUSINESS US TANGIBLE property). <strong>Inventory + depreciable +
            installment obligations + receivables:</strong> NEVER qualify for exception. Compare § 367(d) IP
            outbound (annual deemed royalty). <strong>§ 6038B</strong> reporting required.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s367a.h2.inputs">Inputs</h2>
            <form id="s367a-form" class="inline-form">
                <label><span data-i18n="view.s367a.label.fmv">Property FMV ($)</span>
                    <input type="number" step="0.01" name="property_fmv" value="${state.property_fmv}"></label>
                <label><span data-i18n="view.s367a.label.basis">Transferor's basis ($)</span>
                    <input type="number" step="0.01" name="transferor_basis" value="${state.transferor_basis}"></label>
                <label><span data-i18n="view.s367a.label.type">Property type</span>
                    <select name="type_property">
                        <option value="tangible" ${state.type_property === 'tangible' ? 'selected' : ''}>Tangible (active use)</option>
                        <option value="inventory" ${state.type_property === 'inventory' ? 'selected' : ''}>Inventory (always gain)</option>
                        <option value="depreciable_us" ${state.type_property === 'depreciable_us' ? 'selected' : ''}>Depreciable US use (always gain)</option>
                        <option value="installment" ${state.type_property === 'installment' ? 'selected' : ''}>Installment obligation</option>
                        <option value="receivables" ${state.type_property === 'receivables' ? 'selected' : ''}>Accounts receivable</option>
                        <option value="foreign_currency" ${state.type_property === 'foreign_currency' ? 'selected' : ''}>Foreign currency / debt</option>
                        <option value="commodity" ${state.type_property === 'commodity' ? 'selected' : ''}>Commodity</option>
                    </select>
                </label>
                <label><span data-i18n="view.s367a.label.inventory">Inventory?</span>
                    <input type="checkbox" name="is_inventory" ${state.is_inventory ? 'checked' : ''}></label>
                <label><span data-i18n="view.s367a.label.depreciable">Depreciable US use property?</span>
                    <input type="checkbox" name="is_depreciable_us_use" ${state.is_depreciable_us_use ? 'checked' : ''}></label>
                <label><span data-i18n="view.s367a.label.foreign_active">Foreign use in ACTIVE trade?</span>
                    <input type="checkbox" name="is_foreign_use_active" ${state.is_foreign_use_active ? 'checked' : ''}></label>
                <label><span data-i18n="view.s367a.label.resale_outside">For resale outside US?</span>
                    <input type="checkbox" name="is_resale_outside_us" ${state.is_resale_outside_us ? 'checked' : ''}></label>
                <label><span data-i18n="view.s367a.label.resale_us">For resale in US?</span>
                    <input type="checkbox" name="is_resale_in_us" ${state.is_resale_in_us ? 'checked' : ''}></label>
                <label><span data-i18n="view.s367a.label.country">Transferee country</span>
                    <input type="text" name="transferee_country" value="${esc(state.transferee_country)}"></label>
                <label><span data-i18n="view.s367a.label.cfc">CFC recipient?</span>
                    <input type="checkbox" name="is_cfc_recipient" ${state.is_cfc_recipient ? 'checked' : ''}></label>
                <label><span data-i18n="view.s367a.label.transferor">Domestic transferor corp?</span>
                    <input type="checkbox" name="domestic_transferor_corp" ${state.domestic_transferor_corp ? 'checked' : ''}></label>
                <label><span data-i18n="view.s367a.label.pre_tcja">Pre-TCJA active business election?</span>
                    <input type="checkbox" name="pre_tcja_election" ${state.pre_tcja_election ? 'checked' : ''}></label>
                <label><span data-i18n="view.s367a.label.gra">Gain recognition agreement (5-yr)?</span>
                    <input type="checkbox" name="gain_recognition_agreement" ${state.gain_recognition_agreement ? 'checked' : ''}></label>
                <label><span data-i18n="view.s367a.label.s367a3">§ 367(a)(3) active business eligible?</span>
                    <input type="checkbox" name="s367a3_active_business" ${state.s367a3_active_business ? 'checked' : ''}></label>
                <label><span data-i18n="view.s367a.label.s6038b">§ 6038B reporting filed?</span>
                    <input type="checkbox" name="s6038b_filed" ${state.s6038b_filed ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s367a.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s367a-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s367a.h2.basic_rule">§ 367(a)(1) basic rule</h2>
            <ul class="muted small">
                <li data-i18n="view.s367a.basic.no_corp">Foreign transferee TREATED AS NOT A CORPORATION</li>
                <li data-i18n="view.s367a.basic.no_351">§ 351 non-recognition NOT AVAILABLE → gain recognized</li>
                <li data-i18n="view.s367a.basic.applies">Applies to: § 332, § 351, § 354, § 356, § 361, § 368</li>
                <li data-i18n="view.s367a.basic.policy">Policy: prevent erosion of US tax base on built-in gain</li>
                <li data-i18n="view.s367a.basic.alternative">Alternative: § 367(d) IP / § 482 transfer pricing for ongoing royalties</li>
                <li data-i18n="view.s367a.basic.foreign_to_foreign">§ 367(b) applies to foreign-to-foreign rearrangements</li>
                <li data-i18n="view.s367a.basic.character">Character of gain: same as if sold at FMV (ordinary, capital, recapture)</li>
                <li data-i18n="view.s367a.basic.holding_period">Holding period: as if disposed of (resets in transferee's hands)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s367a.h2.exceptions">§ 367(a)(3) historical exceptions (TCJA narrowed)</h2>
            <ul class="muted small">
                <li data-i18n="view.s367a.exc.active_pre_tcja">PRE-TCJA: active business exception EXEMPT property used in active foreign trade or business</li>
                <li data-i18n="view.s367a.exc.tcja_narrowed">TCJA 2017: REMOVED active business exception for IP, foreign goodwill, going concern</li>
                <li data-i18n="view.s367a.exc.remaining">REMAINING (post-TCJA): tangible property used in ACTIVE TRADE / BUSINESS outside US</li>
                <li data-i18n="view.s367a.exc.never_qualify">NEVER qualify: inventory, depreciable US-use property, installment notes, receivables, foreign currency obligations</li>
                <li data-i18n="view.s367a.exc.foreign_use">Foreign use: must be USED predominantly OUTSIDE US</li>
                <li data-i18n="view.s367a.exc.active_definition">Active TIB: § 1.367(a)-2 substantial managerial / operational activities</li>
                <li data-i18n="view.s367a.exc.recapture">Recapture upon disposition within 5 yrs (Gain Recognition Agreement)</li>
                <li data-i18n="view.s367a.exc.transferee_country">Transferee country: subject to subF, GILTI rules + ECI overlay</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s367a.h2.gra">Gain Recognition Agreement (GRA — § 367(a)(8))</h2>
            <ul class="muted small">
                <li data-i18n="view.s367a.gra.purpose">5-year deferral if transferor enters GRA + commits to recognize gain on triggering events</li>
                <li data-i18n="view.s367a.gra.triggering">Triggering events: sale of stock, dispositions of property, change in basis</li>
                <li data-i18n="view.s367a.gra.annual">Annual certifications + Form 8838 filing</li>
                <li data-i18n="view.s367a.gra.failure">Failure: full gain recognition retroactive + interest + penalties</li>
                <li data-i18n="view.s367a.gra.formal">Formal written agreement + IRS specific procedure</li>
                <li data-i18n="view.s367a.gra.s_corp">S-corp transferor: shareholders bear gain consequences</li>
                <li data-i18n="view.s367a.gra.6038b">Coordinate with § 6038B reporting (Form 926 — Return for U.S. Transferors of Property)</li>
                <li data-i18n="view.s367a.gra.lock_in">5-year period locked in — cannot revoke unless replaced by new GRA</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s367a.h2.types_treatment">Property type-by-type treatment</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s367a.th.type">Property type</th>
                    <th data-i18n="view.s367a.th.treatment">§ 367(a) treatment</th>
                    <th data-i18n="view.s367a.th.note">Note</th>
                </tr></thead>
                <tbody>
                    <tr><td>Inventory</td><td>Gain ALWAYS recognized</td><td>No active business exception</td></tr>
                    <tr><td>Depreciable US use</td><td>Gain ALWAYS recognized</td><td>Recapture preserved</td></tr>
                    <tr><td>Installment obligations</td><td>Gain ALWAYS recognized</td><td>Acceleration</td></tr>
                    <tr><td>Receivables</td><td>Gain ALWAYS recognized</td><td>Phantom income</td></tr>
                    <tr><td>Foreign currency obligations</td><td>Gain ALWAYS recognized</td><td>FX recharacterization</td></tr>
                    <tr><td>Tangible active foreign use</td><td>EXEMPT under § 367(a)(3) (post-TCJA narrow)</td><td>GRA available</td></tr>
                    <tr><td>IP / intangibles</td><td>§ 367(d) annual deemed royalty</td><td>NOT § 367(a)</td></tr>
                    <tr><td>Foreign goodwill</td><td>Gain recognition (TCJA repealed exception)</td><td>2017 change</td></tr>
                </tbody>
            </table>
        </div>
    `;
    document.getElementById('s367a-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.property_fmv = Number(fd.get('property_fmv')) || 0;
        state.transferor_basis = Number(fd.get('transferor_basis')) || 0;
        state.type_property = fd.get('type_property');
        state.is_inventory = !!fd.get('is_inventory');
        state.is_depreciable_us_use = !!fd.get('is_depreciable_us_use');
        state.is_foreign_use_active = !!fd.get('is_foreign_use_active');
        state.is_resale_outside_us = !!fd.get('is_resale_outside_us');
        state.is_resale_in_us = !!fd.get('is_resale_in_us');
        state.transferee_country = fd.get('transferee_country');
        state.is_cfc_recipient = !!fd.get('is_cfc_recipient');
        state.domestic_transferor_corp = !!fd.get('domestic_transferor_corp');
        state.pre_tcja_election = !!fd.get('pre_tcja_election');
        state.gain_recognition_agreement = !!fd.get('gain_recognition_agreement');
        state.s367a3_active_business = !!fd.get('s367a3_active_business');
        state.s6038b_filed = !!fd.get('s6038b_filed');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s367a-output');
    if (!el) return;
    const builtIn = Math.max(0, state.property_fmv - state.transferor_basis);
    const always_gain = state.is_inventory || state.is_depreciable_us_use || state.type_property === 'installment' || state.type_property === 'receivables' || state.type_property === 'foreign_currency';
    const eligible_exception = state.s367a3_active_business && state.is_foreign_use_active && !always_gain;
    const exempt = eligible_exception && state.gain_recognition_agreement;
    const recognized_gain = exempt ? 0 : builtIn;
    const tax_at_corp = recognized_gain * 0.21;
    const not_filed_penalty = !state.s6038b_filed ? 100_000 : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s367a.h2.result">§ 367(a) outcome</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s367a.card.builtin">Built-in gain</div>
                    <div class="value">$${builtIn.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${always_gain ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s367a.card.always_gain">Always gain (category)?</div>
                    <div class="value">${always_gain ? esc(t('view.s367a.status.yes')) : esc(t('view.s367a.status.no'))}</div>
                </div>
                <div class="card ${eligible_exception ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s367a.card.eligible">Active biz exempt eligible?</div>
                    <div class="value">${eligible_exception ? esc(t('view.s367a.status.yes')) : esc(t('view.s367a.status.no'))}</div>
                </div>
                <div class="card ${exempt ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s367a.card.exempt">Exempt (with GRA)?</div>
                    <div class="value">${exempt ? esc(t('view.s367a.status.yes')) : esc(t('view.s367a.status.no'))}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s367a.card.recognized">Gain recognized</div>
                    <div class="value">$${recognized_gain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s367a.card.tax">Corp tax (21%)</div>
                    <div class="value">$${tax_at_corp.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s367a.card.6038b_penalty">§ 6038B penalty (if no filing)</div>
                    <div class="value">$${not_filed_penalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${!state.s6038b_filed && recognized_gain > 0 ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s367a.no_filing_note">
                    § 6038B Form 926 REQUIRED for outbound transfers. Failure to file: $100,000 minimum
                    penalty (or 10% of FMV if higher, capped at $100K). Statute of limitations remains
                    open for 3 yrs after late filing. Penalty waived only for reasonable cause + GOOD FAITH effort.
                </p>
            ` : ''}
        </div>
    `;
}
