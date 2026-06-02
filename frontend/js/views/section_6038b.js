// IRC § 6038B + Form 926 — Transfer of Property to Foreign Corporation.
// US person transfers > $100k of cash OR any property to foreign corp must file Form 926.
// Penalty: 10% of FMV (max $100k per failure, no cap if intentional).
// Section 367 gain recognition: foreign corp can't be used to avoid US tax on appreciated property.
// Common trap: contributing US LLC interest to foreign holding co.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const PENALTY_PCT = 0.10;
const PENALTY_CAP_NEGLIGENT = 100_000;
const CASH_THRESHOLD = 100_000;

let state = {
    transfer_type: 'cash',
    fmv_transferred: 0,
    basis_in_property: 0,
    fmv_received_in_stock: 0,
    intentional: false,
    section_367_gain_required: false,
    is_active_trade_business: false,
    foreign_corp_treaty_country: false,
    your_ownership_pct: 0,
};

export async function renderSection6038b(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6038b.h1.title">// § 6038B FORM 926 FOREIGN TRANSFER</span></h1>
        <p class="muted small" data-i18n="view.s6038b.hint.intro">
            US person transferring &gt; $100k cash OR any property to foreign corp must file
            <strong>Form 926</strong>. <strong>Penalty: 10% of FMV</strong> (max $100k per failure;
            uncapped if intentional). <strong>§ 367(a):</strong> US-built appreciation on
            transferred property triggers gain (foreign corp can't be tax shelter). <strong>§ 367(d):</strong>
            intangibles get special "deemed royalty" treatment. <strong>§ 367(b):</strong>
            outbound reorganizations / spin-offs.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6038b.h2.inputs">Inputs</h2>
            <form id="s6038b-form" class="inline-form">
                <label><span data-i18n="view.s6038b.label.kind">Transfer type</span>
                    <select name="transfer_type">
                        <option value="cash">Cash</option>
                        <option value="stock_securities">US stock / securities</option>
                        <option value="real_property">Real property</option>
                        <option value="active_business">Active trade or business assets</option>
                        <option value="intangibles">Intangibles (IP, customer lists, etc.)</option>
                        <option value="inventory">Inventory</option>
                        <option value="installment_obligations">Installment obligations</option>
                        <option value="accounts_receivable">Accounts receivable</option>
                        <option value="depreciable_property">Depreciable property</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6038b.label.fmv">FMV transferred ($)</span>
                    <input type="number" step="1000" name="fmv_transferred" value="${state.fmv_transferred}"></label>
                <label><span data-i18n="view.s6038b.label.basis">Your basis in property ($)</span>
                    <input type="number" step="1000" name="basis_in_property" value="${state.basis_in_property}"></label>
                <label><span data-i18n="view.s6038b.label.stock_received">FMV stock received in exchange ($)</span>
                    <input type="number" step="1000" name="fmv_received_in_stock" value="${state.fmv_received_in_stock}"></label>
                <label><span data-i18n="view.s6038b.label.intentional">Intentional non-filing?</span>
                    <input type="checkbox" name="intentional" ${state.intentional ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6038b.label.atb">Active trade or business exception?</span>
                    <input type="checkbox" name="is_active_trade_business" ${state.is_active_trade_business ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6038b.label.treaty">Treaty country?</span>
                    <input type="checkbox" name="foreign_corp_treaty_country" ${state.foreign_corp_treaty_country ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6038b.label.ownership">Your ownership % post-transfer</span>
                    <input type="number" step="0.01" name="your_ownership_pct" value="${state.your_ownership_pct}"></label>
                <button class="primary" type="submit" data-i18n="view.s6038b.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6038b-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6038b.h2.367_categories">§ 367(a) treatment by property type</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s6038b.th.property">Property type</th>
                    <th data-i18n="view.s6038b.th.treatment">§ 367(a) treatment</th>
                </tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s6038b.row.atb_assets">Active trade or business assets</td><td>Generally exempt under § 367(a)(3)</td></tr>
                    <tr><td data-i18n="view.s6038b.row.stock_us">US stock</td><td>FULL gain recognition</td></tr>
                    <tr><td data-i18n="view.s6038b.row.inventory">Inventory</td><td>FULL gain recognition</td></tr>
                    <tr><td data-i18n="view.s6038b.row.dep_property">Depreciation recapture</td><td>FULL ordinary income recognized</td></tr>
                    <tr><td data-i18n="view.s6038b.row.intangibles">Intangibles (§ 367(d))</td><td>Deemed annual royalty stream over useful life</td></tr>
                    <tr><td data-i18n="view.s6038b.row.installment">Installment obligations</td><td>FULL gain (no deferral)</td></tr>
                    <tr><td data-i18n="view.s6038b.row.real_property">Real property</td><td>Foreign: FULL gain; US: FIRPTA + § 897 considerations</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6038b.h2.related_forms">Related international forms</h2>
            <ul class="muted small">
                <li data-i18n="view.s6038b.form.5471">Form 5471: ≥ 10% in CFC ongoing reporting (§ 6038)</li>
                <li data-i18n="view.s6038b.form.5472">Form 5472: foreign-owned US corp / disregarded entity (§ 6038A)</li>
                <li data-i18n="view.s6038b.form.8621">Form 8621: PFIC investments (§ 1295 QEF / § 1296 MTM)</li>
                <li data-i18n="view.s6038b.form.8865">Form 8865: foreign partnership (§ 6038)</li>
                <li data-i18n="view.s6038b.form.8858">Form 8858: foreign disregarded entity</li>
                <li data-i18n="view.s6038b.form.3520">Form 3520: foreign trust / large gift (§ 6048)</li>
                <li data-i18n="view.s6038b.form.fbar">FinCEN Form 114 (FBAR): foreign accounts &gt; $10k aggregate</li>
                <li data-i18n="view.s6038b.form.8938">Form 8938: foreign financial assets (FATCA § 6038D)</li>
            </ul>
        </div>
    `;
    document.getElementById('s6038b-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.transfer_type = fd.get('transfer_type');
        state.fmv_transferred = Number(fd.get('fmv_transferred')) || 0;
        state.basis_in_property = Number(fd.get('basis_in_property')) || 0;
        state.fmv_received_in_stock = Number(fd.get('fmv_received_in_stock')) || 0;
        state.intentional = !!fd.get('intentional');
        state.is_active_trade_business = !!fd.get('is_active_trade_business');
        state.foreign_corp_treaty_country = !!fd.get('foreign_corp_treaty_country');
        state.your_ownership_pct = Number(fd.get('your_ownership_pct')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6038b-output');
    if (!el) return;
    const requiresForm926 = (state.transfer_type === 'cash' && state.fmv_transferred > CASH_THRESHOLD)
        || state.transfer_type !== 'cash';
    const embeddedGain = Math.max(0, state.fmv_transferred - state.basis_in_property);
    const isCash = state.transfer_type === 'cash';
    const isAtbExempt = state.is_active_trade_business && state.transfer_type === 'active_business';
    const section367Gain = isCash ? 0 : (isAtbExempt ? 0 : embeddedGain);
    const penalty = requiresForm926
        ? (state.intentional ? state.fmv_transferred * PENALTY_PCT : Math.min(state.fmv_transferred * PENALTY_PCT, PENALTY_CAP_NEGLIGENT))
        : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6038b.h2.result">Form 926 + § 367 exposure</h2>
            <div class="cards">
                <div class="card ${requiresForm926 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6038b.card.requires">Form 926 required?</div>
                    <div class="value">${requiresForm926 ? esc(t('view.s6038b.status.yes')) : esc(t('view.s6038b.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6038b.card.embedded">Embedded gain</div>
                    <div class="value">$${embeddedGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${section367Gain > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6038b.card.367_gain">§ 367 forced gain</div>
                    <div class="value">$${section367Gain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${penalty > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6038b.card.penalty">Penalty if unfiled</div>
                    <div class="value">$${penalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
