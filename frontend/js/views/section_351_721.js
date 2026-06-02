// IRC § 351 (C-corp formation) + § 721 (partnership formation) — Tax-Free Contribution.
// Transfer property to corp/partnership for ownership interest = no gain/loss.
// § 351: contributor + co-contributors collectively own ≥ 80% (control) post-transfer.
// § 721: any property contribution to partnership generally tax-free (no control test).
// Basis carries over. § 357 boot triggers gain if liabilities assumed > basis.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const CONTROL_THRESHOLD = 0.80;

let state = {
    entity_type: 'c_corp',
    property_fmv: 0,
    property_basis: 0,
    liabilities_assumed: 0,
    boot_received: 0,
    control_post_transfer_pct: 0,
    additional_contributors_property_fmv: 0,
    ownership_pct: 0,
    marginal_rate: 0.32,
    ltcg_rate: 0.20,
};

export async function renderSection351721(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s351.h1.title">// § 351 / § 721 TAX-FREE FORMATION</span></h1>
        <p class="muted small" data-i18n="view.s351.hint.intro">
            Contribute property to corp / partnership for ownership interest — generally no gain
            recognized. <strong>§ 351:</strong> contributor + co-contributors collectively own
            ≥ 80% post-transfer. <strong>§ 721:</strong> any contribution to partnership tax-free
            (no control test). <strong>§ 357 boot trap:</strong> liabilities assumed by entity in
            excess of contributor's basis triggers GAIN. <strong>§ 1032:</strong> entity recognizes
            no gain on issuance of its own stock for property.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s351.h2.inputs">Inputs</h2>
            <form id="s351-form" class="inline-form">
                <label><span data-i18n="view.s351.label.entity">Entity type</span>
                    <select name="entity_type">
                        <option value="c_corp" ${state.entity_type === 'c_corp' ? 'selected' : ''}>C-corp (§ 351)</option>
                        <option value="s_corp" ${state.entity_type === 's_corp' ? 'selected' : ''}>S-corp (§ 351)</option>
                        <option value="partnership" ${state.entity_type === 'partnership' ? 'selected' : ''}>Partnership / multi-LLC (§ 721)</option>
                        <option value="llc_disregarded" ${state.entity_type === 'llc_disregarded' ? 'selected' : ''}>Single-member LLC (disregarded, no event)</option>
                    </select>
                </label>
                <label><span data-i18n="view.s351.label.fmv">Property FMV ($)</span>
                    <input type="number" step="10000" name="property_fmv" value="${state.property_fmv}"></label>
                <label><span data-i18n="view.s351.label.basis">Property basis ($)</span>
                    <input type="number" step="10000" name="property_basis" value="${state.property_basis}"></label>
                <label><span data-i18n="view.s351.label.liabilities">Liabilities assumed by entity ($)</span>
                    <input type="number" step="10000" name="liabilities_assumed" value="${state.liabilities_assumed}"></label>
                <label><span data-i18n="view.s351.label.boot">Boot received (cash / non-stock) ($)</span>
                    <input type="number" step="1000" name="boot_received" value="${state.boot_received}"></label>
                <label><span data-i18n="view.s351.label.control">Your control % post-transfer</span>
                    <input type="number" step="0.01" name="control_post_transfer_pct" value="${state.control_post_transfer_pct}"></label>
                <label><span data-i18n="view.s351.label.additional">Other contributors property FMV ($)</span>
                    <input type="number" step="10000" name="additional_contributors_property_fmv" value="${state.additional_contributors_property_fmv}"></label>
                <label><span data-i18n="view.s351.label.your_pct">Your ownership %</span>
                    <input type="number" step="0.01" name="ownership_pct" value="${state.ownership_pct}"></label>
                <label><span data-i18n="view.s351.label.marginal">Ordinary marginal %</span>
                    <input type="number" step="0.01" name="marginal_rate" value="${state.marginal_rate}"></label>
                <label><span data-i18n="view.s351.label.ltcg">LTCG %</span>
                    <input type="number" step="0.01" name="ltcg_rate" value="${state.ltcg_rate}"></label>
                <button class="primary" type="submit" data-i18n="view.s351.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s351-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s351.h2.boot_rules">Boot + liability rules</h2>
            <ul class="muted small">
                <li data-i18n="view.s351.boot.cash">Cash + non-stock property received = BOOT, gain recognized to extent of boot</li>
                <li data-i18n="view.s351.boot.liabilities">Liabilities assumed = BOOT for § 357(c) gain if liabilities &gt; basis (corp only)</li>
                <li data-i18n="view.s351.boot.partnership_liability">Partnership liabilities: increase / decrease partner's basis under § 752</li>
                <li data-i18n="view.s351.boot.no_loss">NEVER recognize loss on § 351 / § 721 (only gain)</li>
                <li data-i18n="view.s351.boot.basis">Transferor basis in entity = property basis + gain - boot - liabilities</li>
                <li data-i18n="view.s351.boot.holding">Holding period of stock / interest TACKS</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s351.h2.differences">§ 351 vs § 721</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s351.th.feature">Feature</th>
                    <th>§ 351</th>
                    <th>§ 721</th>
                </tr></thead>
                <tbody>
                    <tr><td data-i18n="view.s351.row.entity">Entity</td><td>C-corp / S-corp</td><td>Partnership / multi-LLC</td></tr>
                    <tr><td data-i18n="view.s351.row.control">Control test</td><td>≥ 80% post-transfer</td><td>None</td></tr>
                    <tr><td data-i18n="view.s351.row.services">Services exchange</td><td>NOT § 351 (taxable)</td><td>§ 707(a)(2)(A) / 83(b) for profits interest</td></tr>
                    <tr><td data-i18n="view.s351.row.357c">§ 357(c) gain</td><td>Liabilities &gt; basis</td><td>Generally no (§ 752 basis includes liability)</td></tr>
                    <tr><td data-i18n="view.s351.row.disguised">Disguised sale</td><td>—</td><td>§ 707(a)(2)(B) within 2 yrs</td></tr>
                    <tr><td data-i18n="view.s351.row.basis_outside">Outside basis</td><td>Property basis ± boot</td><td>Property basis + share of debt</td></tr>
                </tbody>
            </table>
        </div>
    `;
    document.getElementById('s351-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.entity_type = fd.get('entity_type');
        state.property_fmv = Number(fd.get('property_fmv')) || 0;
        state.property_basis = Number(fd.get('property_basis')) || 0;
        state.liabilities_assumed = Number(fd.get('liabilities_assumed')) || 0;
        state.boot_received = Number(fd.get('boot_received')) || 0;
        state.control_post_transfer_pct = Number(fd.get('control_post_transfer_pct')) || 0;
        state.additional_contributors_property_fmv = Number(fd.get('additional_contributors_property_fmv')) || 0;
        state.ownership_pct = Number(fd.get('ownership_pct')) || 0;
        state.marginal_rate = Number(fd.get('marginal_rate')) || 0.32;
        state.ltcg_rate = Number(fd.get('ltcg_rate')) || 0.20;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s351-output');
    if (!el) return;
    const isCorp = state.entity_type === 'c_corp' || state.entity_type === 's_corp';
    const isPartnership = state.entity_type === 'partnership';
    const isDisregarded = state.entity_type === 'llc_disregarded';
    if (isDisregarded) {
        el.innerHTML = `
            <div class="chart-panel">
                <h2 data-i18n="view.s351.h2.result">Result</h2>
                <p class="muted" data-i18n="view.s351.note.disregarded">
                    Single-member LLC = disregarded entity — no taxable event, owner directly
                    holds property for tax purposes.
                </p>
            </div>
        `;
        return;
    }
    const embeddedGain = Math.max(0, state.property_fmv - state.property_basis);
    const passesControl = !isCorp || state.control_post_transfer_pct >= CONTROL_THRESHOLD * 100;
    let bootGain = 0;
    let s357c = 0;
    if (passesControl) {
        bootGain = Math.min(state.boot_received, embeddedGain);
        if (isCorp) {
            s357c = Math.max(0, state.liabilities_assumed - state.property_basis);
        }
    } else {
        bootGain = embeddedGain;
    }
    const totalGain = bootGain + s357c;
    const tax = totalGain * state.ltcg_rate;
    const newBasisInInterest = state.property_basis + totalGain - state.boot_received - (isCorp ? state.liabilities_assumed : 0);
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s351.h2.result">Result</h2>
            <div class="cards">
                <div class="card ${passesControl ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s351.card.passes_control">Passes 80% control (§ 351)</div>
                    <div class="value">${passesControl ? esc(t('view.s351.status.yes')) : esc(t('view.s351.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s351.card.embedded">Embedded gain</div>
                    <div class="value">$${embeddedGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card ${bootGain > 0 ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s351.card.boot_gain">Gain recognized (boot)</div>
                    <div class="value">$${bootGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                ${s357c > 0 ? `
                    <div class="card neg">
                        <div class="label" data-i18n="view.s351.card.357c">§ 357(c) liability gain</div>
                        <div class="value">$${s357c.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                    </div>
                ` : ''}
                <div class="card neg">
                    <div class="label" data-i18n="view.s351.card.tax">Federal tax</div>
                    <div class="value">$${tax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s351.card.deferred">Gain deferred</div>
                    <div class="value">$${(embeddedGain - bootGain).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s351.card.new_basis">New basis in equity interest</div>
                    <div class="value">$${newBasisInInterest.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
