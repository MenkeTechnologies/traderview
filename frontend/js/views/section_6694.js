// IRC § 6694 — Return Preparer Penalty.
// (a) Unreasonable position: lesser of $1,000 OR 50% of preparer's fee.
// (b) Willful or reckless: lesser of $5,000 OR 75% of preparer's fee.
// "Tax return preparer" = paid for substantial preparation. Also volunteers if signing.
// Reasonable basis standard: 33% chance position will be sustained. More-likely-than-not for tax shelters.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const SUBSEC_A_FLAT = 1_000;
const SUBSEC_A_PCT = 0.50;
const SUBSEC_B_FLAT = 5_000;
const SUBSEC_B_PCT = 0.75;

let state = {
    preparer_fee: 0,
    position_kind: 'reasonable_basis',
    is_tax_shelter: false,
    return_disclosed: false,
    has_reasonable_basis: true,
    willful_or_reckless: false,
    preparer_relied_on_taxpayer_info: false,
    return_count: 1,
};

export async function renderSection6694(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6694.h1.title">// § 6694 PREPARER PENALTY</span></h1>
        <p class="muted small" data-i18n="view.s6694.hint.intro">
            <strong>(a) Unreasonable position:</strong> lesser of <strong>$1,000 OR 50% of preparer's
            fee</strong>. <strong>(b) Willful or reckless:</strong> lesser of <strong>$5,000 OR 75%
            of preparer's fee</strong>. Reasonable-basis standard: ~33% chance of being sustained.
            <strong>Tax-shelter / reportable transaction:</strong> more-likely-than-not standard
            (&gt;50%). Disclosure (Form 8275 / Form 8275-R) on aggressive position avoids (a).
            Required Practitioner Registration (PTIN) annually.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6694.h2.inputs">Inputs</h2>
            <form id="s6694-form" class="inline-form">
                <label><span data-i18n="view.s6694.label.fee">Preparer's fee for return ($)</span>
                    <input type="number" step="0.01" name="preparer_fee" value="${state.preparer_fee}"></label>
                <label><span data-i18n="view.s6694.label.position">Position kind</span>
                    <select name="position_kind">
                        <option value="more_likely_than_not" ${state.position_kind === 'more_likely_than_not' ? 'selected' : ''}>More-likely-than-not (&gt; 50%)</option>
                        <option value="substantial_authority" ${state.position_kind === 'substantial_authority' ? 'selected' : ''}>Substantial authority (~40%)</option>
                        <option value="reasonable_basis" ${state.position_kind === 'reasonable_basis' ? 'selected' : ''}>Reasonable basis (~33%)</option>
                        <option value="frivolous" ${state.position_kind === 'frivolous' ? 'selected' : ''}>Frivolous</option>
                    </select>
                </label>
                <label><span data-i18n="view.s6694.label.shelter">Tax-shelter / reportable transaction?</span>
                    <input type="checkbox" name="is_tax_shelter" ${state.is_tax_shelter ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6694.label.disclosed">Position disclosed on Form 8275?</span>
                    <input type="checkbox" name="return_disclosed" ${state.return_disclosed ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6694.label.reasonable">Has reasonable basis?</span>
                    <input type="checkbox" name="has_reasonable_basis" ${state.has_reasonable_basis ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6694.label.willful">Willful / reckless?</span>
                    <input type="checkbox" name="willful_or_reckless" ${state.willful_or_reckless ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6694.label.relied">Preparer relied on taxpayer info?</span>
                    <input type="checkbox" name="preparer_relied_on_taxpayer_info" ${state.preparer_relied_on_taxpayer_info ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6694.label.return_count">Affected returns</span>
                    <input type="number" step="1" name="return_count" value="${state.return_count}"></label>
                <button class="primary" type="submit" data-i18n="view.s6694.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6694-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6694.h2.standards">Standards hierarchy</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s6694.th.standard">Standard</th>
                    <th data-i18n="view.s6694.th.threshold">Threshold</th>
                    <th data-i18n="view.s6694.th.use">When required</th>
                </tr></thead>
                <tbody>
                    <tr><td>More-Likely-Than-Not (MLTN)</td><td>&gt; 50%</td><td>Tax shelter, reportable transaction, listed transaction</td></tr>
                    <tr><td>Substantial Authority</td><td>~40%</td><td>Avoids § 6662 accuracy penalty for undisclosed positions</td></tr>
                    <tr><td>Reasonable Basis</td><td>~33%</td><td>Avoids § 6694(a) with disclosure on Form 8275</td></tr>
                    <tr><td>Realistic Possibility (RP)</td><td>~33%</td><td>Pre-2007 standard, mostly replaced</td></tr>
                    <tr><td>Frivolous</td><td>&lt; reasonable</td><td>§ 6702 $5k penalty + § 6673 &$25k against frivolous filer</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6694.h2.related">Related preparer penalties</h2>
            <ul class="muted small">
                <li data-i18n="view.s6694.rel.6695a">§ 6695(a) Failure to furnish copy of return: $60/failure (max $30k)</li>
                <li data-i18n="view.s6694.rel.6695b">§ 6695(b) Failure to sign return: $60/failure</li>
                <li data-i18n="view.s6694.rel.6695c">§ 6695(c) Failure to furnish PTIN: $60/failure</li>
                <li data-i18n="view.s6694.rel.6695g">§ 6695(g) Failure to comply with EITC due diligence: $635/failure (2024)</li>
                <li data-i18n="view.s6694.rel.6700">§ 6700 Abusive tax shelter promoter: 50% of gross income from activity</li>
                <li data-i18n="view.s6694.rel.6701">§ 6701 Aiding + abetting: $1k / $10k corp per document</li>
                <li data-i18n="view.s6694.rel.7407">§ 7407 Injunction against preparer (court action)</li>
                <li data-i18n="view.s6694.rel.7216">§ 7216 Unauthorized disclosure of taxpayer info: $1k + 1-yr prison</li>
            </ul>
        </div>
    `;
    document.getElementById('s6694-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.preparer_fee = Number(fd.get('preparer_fee')) || 0;
        state.position_kind = fd.get('position_kind');
        state.is_tax_shelter = !!fd.get('is_tax_shelter');
        state.return_disclosed = !!fd.get('return_disclosed');
        state.has_reasonable_basis = !!fd.get('has_reasonable_basis');
        state.willful_or_reckless = !!fd.get('willful_or_reckless');
        state.preparer_relied_on_taxpayer_info = !!fd.get('preparer_relied_on_taxpayer_info');
        state.return_count = Number(fd.get('return_count')) || 1;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6694-output');
    if (!el) return;
    let triggers_a = false;
    if (state.is_tax_shelter) {
        triggers_a = state.position_kind !== 'more_likely_than_not';
    } else if (state.return_disclosed) {
        triggers_a = !state.has_reasonable_basis;
    } else {
        triggers_a = state.position_kind === 'reasonable_basis' || state.position_kind === 'frivolous';
    }
    const triggers_b = state.willful_or_reckless;
    const penalty_a_per_return = triggers_a ? Math.min(SUBSEC_A_FLAT, state.preparer_fee * SUBSEC_A_PCT) : 0;
    const penalty_b_per_return = triggers_b ? Math.min(SUBSEC_B_FLAT, state.preparer_fee * SUBSEC_B_PCT) : 0;
    const penaltyPerReturn = Math.max(penalty_a_per_return, penalty_b_per_return);
    const totalPenalty = penaltyPerReturn * state.return_count;
    const reasonable_cause_defense = state.preparer_relied_on_taxpayer_info ? penaltyPerReturn * 0.5 : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6694.h2.result">Preparer penalty</h2>
            <div class="cards">
                <div class="card ${triggers_a ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6694.card.triggers_a">Triggers (a) unreasonable?</div>
                    <div class="value">${triggers_a ? esc(t('view.s6694.status.yes')) : esc(t('view.s6694.status.no'))}</div>
                </div>
                <div class="card ${triggers_b ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s6694.card.triggers_b">Triggers (b) willful?</div>
                    <div class="value">${triggers_b ? esc(t('view.s6694.status.yes')) : esc(t('view.s6694.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6694.card.fee">Fee × subsection rate</div>
                    <div class="value">$${(state.preparer_fee * (triggers_b ? SUBSEC_B_PCT : SUBSEC_A_PCT)).toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6694.card.per_return">Penalty per return</div>
                    <div class="value">$${penaltyPerReturn.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s6694.card.total">Total penalty</div>
                    <div class="value">$${totalPenalty.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s6694.card.defense">Reasonable cause defense $</div>
                    <div class="value">$${reasonable_cause_defense.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
