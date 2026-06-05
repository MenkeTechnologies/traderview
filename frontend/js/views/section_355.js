// IRC § 355 — Corporate Spin-Off / Split-Off / Split-Up.
// Tax-free distribution of CONTROLLED CORP STOCK if 5-yr active business + control + non-tax purpose.
// Spin-off: distribute on pro-rata basis (shareholders keep both). Split-off: surrender Distrib stock.
// Split-up: parent liquidates, distributes Sub1 + Sub2 to shareholders.
// Anti-abuse: § 355(d) 50% acquisition, § 355(e) "Morris Trust" (50% acq of either).

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    transaction_type: 'spinoff',
    distributing_basis: 0,
    controlled_basis: 0,
    controlled_fmv: 0,
    distributing_fmv: 0,
    five_year_active: true,
    business_purpose: true,
    not_device: true,
    control_distribution: true,
    continuity_50pct: true,
    s355d_50pct_acq: false,
    s355e_morris_trust: false,
    boot_received: 0,
    debt_assumed: 0,
};

export async function renderSection355(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s355.h1.title">// § 355 SPIN-OFF / SPLIT-OFF</span></h1>
        <p class="muted small" data-i18n="view.s355.hint.intro">
            Tax-free distribution of <strong>CONTROLLED CORP STOCK</strong>. Requirements: (1) <strong>5-yr active
            business</strong> EACH side, (2) <strong>control</strong> (80%) distributed, (3) <strong>not a device</strong>
            for dividend, (4) <strong>business purpose</strong>, (5) <strong>continuity of interest</strong>
            (50%+ owners). <strong>Spin-off:</strong> pro-rata (shareholders keep both). <strong>Split-off:</strong>
            surrender Distributing stock. <strong>Split-up:</strong> Distributing liquidates → shareholders get Sub1
            + Sub2. <strong>Anti-abuse:</strong> § 355(d) 50% acquisition triggers gain. § 355(e) Morris Trust.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s355.h2.inputs">Inputs</h2>
            <form id="s355-form" class="inline-form">
                <label><span data-i18n="view.s355.label.type">Transaction type</span>
                    <select name="transaction_type">
                        <option value="spinoff" ${state.transaction_type === 'spinoff' ? 'selected' : ''}>Spin-off</option>
                        <option value="splitoff" ${state.transaction_type === 'splitoff' ? 'selected' : ''}>Split-off</option>
                        <option value="splitup" ${state.transaction_type === 'splitup' ? 'selected' : ''}>Split-up</option>
                    </select>
                </label>
                <label><span data-i18n="view.s355.label.dist_basis">Distributing inside basis ($)</span>
                    <input type="number" step="0.01" name="distributing_basis" value="${state.distributing_basis}"></label>
                <label><span data-i18n="view.s355.label.ctrl_basis">Controlled inside basis ($)</span>
                    <input type="number" step="0.01" name="controlled_basis" value="${state.controlled_basis}"></label>
                <label><span data-i18n="view.s355.label.ctrl_fmv">Controlled FMV ($)</span>
                    <input type="number" step="0.01" name="controlled_fmv" value="${state.controlled_fmv}"></label>
                <label><span data-i18n="view.s355.label.dist_fmv">Distributing FMV ($)</span>
                    <input type="number" step="0.01" name="distributing_fmv" value="${state.distributing_fmv}"></label>
                <label><span data-i18n="view.s355.label.active">5-yr active business each side?</span>
                    <input type="checkbox" name="five_year_active" ${state.five_year_active ? 'checked' : ''}></label>
                <label><span data-i18n="view.s355.label.purpose">Business purpose?</span>
                    <input type="checkbox" name="business_purpose" ${state.business_purpose ? 'checked' : ''}></label>
                <label><span data-i18n="view.s355.label.device">Not a device test?</span>
                    <input type="checkbox" name="not_device" ${state.not_device ? 'checked' : ''}></label>
                <label><span data-i18n="view.s355.label.control">Control (80%) distributed?</span>
                    <input type="checkbox" name="control_distribution" ${state.control_distribution ? 'checked' : ''}></label>
                <label><span data-i18n="view.s355.label.continuity">Continuity ≥ 50%?</span>
                    <input type="checkbox" name="continuity_50pct" ${state.continuity_50pct ? 'checked' : ''}></label>
                <label><span data-i18n="view.s355.label.s355d">§ 355(d) 50%+ acquisition?</span>
                    <input type="checkbox" name="s355d_50pct_acq" ${state.s355d_50pct_acq ? 'checked' : ''}></label>
                <label><span data-i18n="view.s355.label.s355e">§ 355(e) Morris Trust (50% acq of either)?</span>
                    <input type="checkbox" name="s355e_morris_trust" ${state.s355e_morris_trust ? 'checked' : ''}></label>
                <label><span data-i18n="view.s355.label.boot">Boot received ($)</span>
                    <input type="number" step="0.01" name="boot_received" value="${state.boot_received}"></label>
                <label><span data-i18n="view.s355.label.debt">Debt assumed by Controlled ($)</span>
                    <input type="number" step="0.01" name="debt_assumed" value="${state.debt_assumed}"></label>
                <button class="primary" type="submit" data-i18n="view.s355.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s355-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s355.h2.requirements">§ 355 requirements (cumulative)</h2>
            <ol class="muted small">
                <li data-i18n="view.s355.req.active">Active Trade or Business 5-year rule: § 355(b) — BOTH Distributing + Controlled engaged for 5 years</li>
                <li data-i18n="view.s355.req.control">Distribute Control: 80% vote AND 80% value of Controlled stock</li>
                <li data-i18n="view.s355.req.device">Not Used Principally as Device for distribution of E&P (§ 355(a)(1)(B))</li>
                <li data-i18n="view.s355.req.purpose">Corporate business purpose (Reg § 1.355-2(b))</li>
                <li data-i18n="view.s355.req.continuity">Continuity of Interest: 50%+ Distributing shareholders retain stock in spun-off</li>
                <li data-i18n="view.s355.req.cobe">Continuity of Business Enterprise: BOTH sides continue historic business</li>
                <li data-i18n="view.s355.req.no_355d">No § 355(d) 50%+ disqualified stock acquisition</li>
                <li data-i18n="view.s355.req.no_355e">No § 355(e) "Morris Trust" 50% acquisition of either D or C within 2-yr window</li>
            </ol>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s355.h2.anti_abuse">Anti-abuse triggers</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s355.th.trigger">Trigger</th>
                    <th data-i18n="view.s355.th.consequence">Consequence</th>
                    <th data-i18n="view.s355.th.notes">Notes</th>
                </tr></thead>
                <tbody>
                    <tr><td>§ 355(d) — Disqualified stock 50%+ acq</td><td>Distributing recognizes gain (not loss)</td><td>5-yr lookback; both before + after distribution</td></tr>
                    <tr><td>§ 355(e) — Morris Trust</td><td>Distributing recognizes gain (not loss)</td><td>50% acq of EITHER D or C within 2-yr window</td></tr>
                    <tr><td>§ 355(f) — Intragroup</td><td>Special rules for consolidated group D-reorgs</td><td>Coordinate with reg § 1.1502 rules</td></tr>
                    <tr><td>"Device" test fail</td><td>Full taxable distribution</td><td>Disproportionate distribution + recent sales = device</td></tr>
                    <tr><td>"Hot stock"</td><td>Distributing recognizes gain on hot stock portion</td><td>Stock acquired in taxable transaction within 5 yrs (§ 355(a)(3)(B))</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s355.h2.notable_cases">Notable spin-off cases</h2>
            <ul class="muted small">
                <li data-i18n="view.s355.case.commscope_3com">CommScope (Avantek), 3Com (Palm), HP (HPE) → standard prep spins</li>
                <li data-i18n="view.s355.case.ebay_paypal">eBay → PayPal (2015): clean Type D divisive</li>
                <li data-i18n="view.s355.case.danaher_envista">Danaher → Envista (2019): public spin model</li>
                <li data-i18n="view.s355.case.dupont_chemours">DuPont → Chemours (2015): chemicals spin</li>
                <li data-i18n="view.s355.case.morris_trust_origin">Morris Trust v. Commissioner (1966): merger-spin combo upheld → § 355(e) enacted 1997</li>
                <li data-i18n="view.s355.case.kraft_mondelez">Kraft → Mondelēz (2012): snacks vs grocery</li>
                <li data-i18n="view.s355.case.private_letter">PLR practice: 6-9 months pre-spin work; ruling cost $40-50K</li>
                <li data-i18n="view.s355.case.no_ruling">"No-rule" areas: 2003 Rev. Proc. 2003-48 reduced rulings significantly</li>
            </ul>
        </div>
    `;
    document.getElementById('s355-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.transaction_type = fd.get('transaction_type');
        state.distributing_basis = Number(fd.get('distributing_basis')) || 0;
        state.controlled_basis = Number(fd.get('controlled_basis')) || 0;
        state.controlled_fmv = Number(fd.get('controlled_fmv')) || 0;
        state.distributing_fmv = Number(fd.get('distributing_fmv')) || 0;
        state.five_year_active = !!fd.get('five_year_active');
        state.business_purpose = !!fd.get('business_purpose');
        state.not_device = !!fd.get('not_device');
        state.control_distribution = !!fd.get('control_distribution');
        state.continuity_50pct = !!fd.get('continuity_50pct');
        state.s355d_50pct_acq = !!fd.get('s355d_50pct_acq');
        state.s355e_morris_trust = !!fd.get('s355e_morris_trust');
        state.boot_received = Number(fd.get('boot_received')) || 0;
        state.debt_assumed = Number(fd.get('debt_assumed')) || 0;
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s355-output');
    if (!el) return;
    const requirementsMet = state.five_year_active && state.business_purpose && state.not_device && state.control_distribution && state.continuity_50pct;
    const antiAbuseTriggered = state.s355d_50pct_acq || state.s355e_morris_trust;
    const qualifies = requirementsMet && !antiAbuseTriggered;
    const distributingGainNoSpinoff = Math.max(0, state.controlled_fmv - state.controlled_basis);
    const distributingGainRecognized = qualifies ? 0 : distributingGainNoSpinoff;
    const distributingTax = distributingGainRecognized * 0.21;
    const bootRecognized = qualifies ? state.boot_received : state.boot_received + distributingGainNoSpinoff;
    const bootTax = bootRecognized * 0.20;
    const totalTaxAvoided = qualifies ? distributingGainNoSpinoff * 0.21 : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s355.h2.result">§ 355 outcome</h2>
            <div class="cards">
                <div class="card ${qualifies ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s355.card.qualifies">§ 355 qualifies?</div>
                    <div class="value">${qualifies ? esc(t('view.s355.status.yes')) : esc(t('view.s355.status.no'))}</div>
                </div>
                <div class="card ${requirementsMet ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s355.card.requirements">Five requirements met?</div>
                    <div class="value">${requirementsMet ? esc(t('view.s355.status.yes')) : esc(t('view.s355.status.no'))}</div>
                </div>
                <div class="card ${antiAbuseTriggered ? 'neg' : 'pos'}">
                    <div class="label" data-i18n="view.s355.card.anti_abuse">Anti-abuse triggered?</div>
                    <div class="value">${antiAbuseTriggered ? esc(t('view.s355.status.yes')) : esc(t('view.s355.status.no'))}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s355.card.dist_gain">Distributing gain recog.</div>
                    <div class="value">$${distributingGainRecognized.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s355.card.tax_dist">Tax on Distributing (21%)</div>
                    <div class="value">$${distributingTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s355.card.boot_tax">Boot tax (20%)</div>
                    <div class="value">$${bootTax.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s355.card.savings">Total tax avoided</div>
                    <div class="value">$${totalTaxAvoided.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
            ${state.s355e_morris_trust ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s355.morris_note">
                    § 355(e) Morris Trust triggered: 50%+ acquisition of either D or C within 2-yr window
                    (before / after) causes Distributing to recognize gain (not loss). Plan against post-spin
                    acquisitions for at least 24 months to preserve tax-free status.
                </p>
            ` : ''}
        </div>
    `;
}
