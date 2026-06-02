// IRC § 368 — Tax-Free Reorganizations (7 Types).
// Type A: statutory merger / consolidation
// Type B: stock-for-stock (80% control acquired)
// Type C: stock-for-assets (substantially all assets, 80% in voting stock)
// Type D: divisive (spin-off / split-off via § 355) or acquisitive
// Type E: recapitalization
// Type F: mere change of identity / form / place
// Type G: bankruptcy reorganization

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    type: 'A',
    target_fmv: 0,
    target_basis: 0,
    consideration_voting_stock: 0,
    consideration_other: 0,
    boot_amount: 0,
    is_triangular: false,
    forward_or_reverse: 'forward',
    is_355_spin: false,
    is_bankruptcy_g: false,
    target_nol_carryforward: 0,
    target_value_for_s382: 0,
    continuity_of_interest: 100,
    continuity_of_business: true,
    business_purpose: true,
    plan_of_reorganization: true,
};

export async function renderSection368(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s368.h1.title">// § 368 REORGANIZATIONS</span></h1>
        <p class="muted small" data-i18n="view.s368.hint.intro">
            Seven types of <strong>tax-free reorganizations</strong> § 368(a)(1)(A)-(G). Common requirements:
            <strong>Continuity of Interest</strong> (40%+ stock consideration), <strong>Continuity of Business
            Enterprise</strong>, <strong>Business Purpose</strong>, <strong>Plan of Reorganization</strong>.
            Carry over: § 381 sub attributes (NOL, E&P, credits). <strong>§ 382 limitation</strong> on
            NOL post-ownership-change. Boot = non-stock consideration recognized as gain (not loss). Form
            8806 filed for B / C / D acquisitive reorgs.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s368.h2.inputs">Inputs</h2>
            <form id="s368-form" class="inline-form">
                <label><span data-i18n="view.s368.label.type">Reorg type</span>
                    <select name="type">
                        <option value="A" ${state.type === 'A' ? 'selected' : ''}>A — Statutory merger / consolidation</option>
                        <option value="B" ${state.type === 'B' ? 'selected' : ''}>B — Stock-for-stock (80%)</option>
                        <option value="C" ${state.type === 'C' ? 'selected' : ''}>C — Stock-for-assets</option>
                        <option value="D_acquisitive" ${state.type === 'D_acquisitive' ? 'selected' : ''}>D — Acquisitive</option>
                        <option value="D_divisive" ${state.type === 'D_divisive' ? 'selected' : ''}>D — Divisive (spin-off § 355)</option>
                        <option value="E" ${state.type === 'E' ? 'selected' : ''}>E — Recapitalization</option>
                        <option value="F" ${state.type === 'F' ? 'selected' : ''}>F — Change of form / identity</option>
                        <option value="G" ${state.type === 'G' ? 'selected' : ''}>G — Bankruptcy reorg</option>
                    </select>
                </label>
                <label><span data-i18n="view.s368.label.fmv">Target FMV ($)</span>
                    <input type="number" step="1000000" name="target_fmv" value="${state.target_fmv}"></label>
                <label><span data-i18n="view.s368.label.basis">Target basis ($)</span>
                    <input type="number" step="1000000" name="target_basis" value="${state.target_basis}"></label>
                <label><span data-i18n="view.s368.label.voting_stock">Voting stock consideration ($)</span>
                    <input type="number" step="100000" name="consideration_voting_stock" value="${state.consideration_voting_stock}"></label>
                <label><span data-i18n="view.s368.label.other">Other consideration ($)</span>
                    <input type="number" step="100000" name="consideration_other" value="${state.consideration_other}"></label>
                <label><span data-i18n="view.s368.label.boot">Boot amount ($)</span>
                    <input type="number" step="100000" name="boot_amount" value="${state.boot_amount}"></label>
                <label><span data-i18n="view.s368.label.triangular">Triangular reorg?</span>
                    <input type="checkbox" name="is_triangular" ${state.is_triangular ? 'checked' : ''}></label>
                <label><span data-i18n="view.s368.label.direction">Direction</span>
                    <select name="forward_or_reverse">
                        <option value="forward" ${state.forward_or_reverse === 'forward' ? 'selected' : ''}>Forward — target → acquirer</option>
                        <option value="reverse" ${state.forward_or_reverse === 'reverse' ? 'selected' : ''}>Reverse — acquirer → target</option>
                    </select>
                </label>
                <label><span data-i18n="view.s368.label.355">§ 355 spin-off planned?</span>
                    <input type="checkbox" name="is_355_spin" ${state.is_355_spin ? 'checked' : ''}></label>
                <label><span data-i18n="view.s368.label.bankruptcy">Bankruptcy Type G?</span>
                    <input type="checkbox" name="is_bankruptcy_g" ${state.is_bankruptcy_g ? 'checked' : ''}></label>
                <label><span data-i18n="view.s368.label.nol">Target NOL carryforward ($)</span>
                    <input type="number" step="100000" name="target_nol_carryforward" value="${state.target_nol_carryforward}"></label>
                <label><span data-i18n="view.s368.label.s382_value">§ 382 target value ($)</span>
                    <input type="number" step="100000" name="target_value_for_s382" value="${state.target_value_for_s382}"></label>
                <label><span data-i18n="view.s368.label.coi">Continuity of Interest %</span>
                    <input type="number" step="0.1" name="continuity_of_interest" value="${state.continuity_of_interest}"></label>
                <label><span data-i18n="view.s368.label.cobe">Continuity of Business?</span>
                    <input type="checkbox" name="continuity_of_business" ${state.continuity_of_business ? 'checked' : ''}></label>
                <label><span data-i18n="view.s368.label.bp">Business purpose?</span>
                    <input type="checkbox" name="business_purpose" ${state.business_purpose ? 'checked' : ''}></label>
                <label><span data-i18n="view.s368.label.plan">Plan of reorganization?</span>
                    <input type="checkbox" name="plan_of_reorganization" ${state.plan_of_reorganization ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.s368.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s368-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s368.h2.types">Reorg type detail</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s368.th.type">Type</th>
                    <th data-i18n="view.s368.th.consideration">Consideration</th>
                    <th data-i18n="view.s368.th.assets">Assets</th>
                    <th data-i18n="view.s368.th.notes">Notes</th>
                </tr></thead>
                <tbody>
                    <tr><td>A — Merger</td><td>Most flexible — cash + stock OK</td><td>All by operation of law</td><td>Triangular variations common</td></tr>
                    <tr><td>B — Stock swap</td><td>SOLELY voting stock</td><td>None — stock acquisition</td><td>Must acquire 80% control</td></tr>
                    <tr><td>C — Asset swap</td><td>Voting stock (80%+) + boot allowed</td><td>Substantially all (70%+ gross, 90%+ net)</td><td>Target distributes received stock</td></tr>
                    <tr><td>D — Acquisitive</td><td>Stock to target shareholders</td><td>Substantially all to controlled sub</td><td>Stock + § 355 distribution</td></tr>
                    <tr><td>D — Divisive</td><td>Stock of controlled corp to shareholders</td><td>5-yr active business each side</td><td>§ 355 spin-off / split-off / split-up</td></tr>
                    <tr><td>E — Recap</td><td>Internal — change cap structure</td><td>Same corp</td><td>Pref ↔ common, debt ↔ equity</td></tr>
                    <tr><td>F — Form change</td><td>State change / name change</td><td>Same corp</td><td>Most common: incorporation moves</td></tr>
                    <tr><td>G — Bankruptcy</td><td>Court-approved</td><td>Reorganization of debtor</td><td>§ 382 has special bankruptcy exceptions</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s368.h2.judicial">Judicial doctrines</h2>
            <ul class="muted small">
                <li data-i18n="view.s368.jd.coi">Continuity of Interest (COI): 40%+ stock consideration; measured at time of binding agreement (post-2005)</li>
                <li data-i18n="view.s368.jd.cobe">Continuity of Business Enterprise (COBE): continue HISTORIC business OR use HISTORIC assets in business</li>
                <li data-i18n="view.s368.jd.business_purpose">Business Purpose: corp-level (not solely tax-motivated). Gregory v. Helvering (1935)</li>
                <li data-i18n="view.s368.jd.plan">Plan of Reorganization: formal corporate action documenting steps</li>
                <li data-i18n="view.s368.jd.step_transaction">Step transaction: integrated steps treated as one (Smith v. Comm'r, Walden v. Comm'r)</li>
                <li data-i18n="view.s368.jd.substance_form">Substance over form: tax follows economic substance</li>
                <li data-i18n="view.s368.jd.economic_substance">§ 7701(o) codified economic substance doctrine — meaningful change required + non-tax purpose</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s368.h2.s382">§ 382 NOL limitation on ownership change</h2>
            <ul class="muted small">
                <li data-i18n="view.s368.s382.trigger">Ownership change: > 50 pct-point shift over 3-yr test period among 5% shareholders</li>
                <li data-i18n="view.s368.s382.limit">Annual NOL limit: target value × federal long-term tax-exempt rate (~4-5%)</li>
                <li data-i18n="view.s368.s382.continuity">2-year continuity-of-business requirement post-change</li>
                <li data-i18n="view.s368.s382.bankruptcy_exception">§ 382(l)(5): G reorg in bankruptcy — historic creditors as continuing shareholders</li>
                <li data-i18n="view.s368.s382.election">§ 382(l)(5)(H) election out: avoid sub's § 269 anti-abuse</li>
                <li data-i18n="view.s368.s382.built_in_gain">RBIG (recognized built-in gain) adjustments increase limit during 5-yr period</li>
                <li data-i18n="view.s368.s382.notice2003_65">Notice 2003-65 + 2019 proposed regs detail built-in gain measurement</li>
            </ul>
        </div>
    `;
    document.getElementById('s368-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.type = fd.get('type');
        state.target_fmv = Number(fd.get('target_fmv')) || 0;
        state.target_basis = Number(fd.get('target_basis')) || 0;
        state.consideration_voting_stock = Number(fd.get('consideration_voting_stock')) || 0;
        state.consideration_other = Number(fd.get('consideration_other')) || 0;
        state.boot_amount = Number(fd.get('boot_amount')) || 0;
        state.is_triangular = !!fd.get('is_triangular');
        state.forward_or_reverse = fd.get('forward_or_reverse');
        state.is_355_spin = !!fd.get('is_355_spin');
        state.is_bankruptcy_g = !!fd.get('is_bankruptcy_g');
        state.target_nol_carryforward = Number(fd.get('target_nol_carryforward')) || 0;
        state.target_value_for_s382 = Number(fd.get('target_value_for_s382')) || 0;
        state.continuity_of_interest = Number(fd.get('continuity_of_interest')) || 0;
        state.continuity_of_business = !!fd.get('continuity_of_business');
        state.business_purpose = !!fd.get('business_purpose');
        state.plan_of_reorganization = !!fd.get('plan_of_reorganization');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s368-output');
    if (!el) return;
    const coiSatisfied = state.continuity_of_interest >= 40;
    const allJudicial = coiSatisfied && state.continuity_of_business && state.business_purpose && state.plan_of_reorganization;
    const qualifies = allJudicial;
    const targetGainNoReorg = Math.max(0, state.target_fmv - state.target_basis);
    const bootRecognized = qualifies ? Math.min(state.boot_amount, targetGainNoReorg) : targetGainNoReorg;
    const deferredGain = qualifies ? Math.max(0, targetGainNoReorg - bootRecognized) : 0;
    const taxOnBoot = bootRecognized * 0.20;
    const taxDeferred = deferredGain * 0.20;
    const s382LongTermRate = 0.045;
    const annualS382Limit = state.target_value_for_s382 * s382LongTermRate;
    const yearsToConsumeNOL = state.target_nol_carryforward > 0 && annualS382Limit > 0 ?
        state.target_nol_carryforward / annualS382Limit : 0;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s368.h2.result">§ 368 outcome</h2>
            <div class="cards">
                <div class="card ${qualifies ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s368.card.qualifies">Reorg qualifies?</div>
                    <div class="value">${qualifies ? esc(t('view.s368.status.yes')) : esc(t('view.s368.status.no'))}</div>
                </div>
                <div class="card ${coiSatisfied ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s368.card.coi">COI ≥ 40%</div>
                    <div class="value">${coiSatisfied ? esc(t('view.s368.status.yes')) : esc(t('view.s368.status.no'))}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s368.card.boot_gain">Boot recognized gain</div>
                    <div class="value">$${bootRecognized.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s368.card.deferred">Deferred gain (tax-free)</div>
                    <div class="value">$${deferredGain.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card neg">
                    <div class="label" data-i18n="view.s368.card.tax_boot">Tax on boot (20%)</div>
                    <div class="value">$${taxOnBoot.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s368.card.tax_deferred">Tax deferred (20%)</div>
                    <div class="value">$${taxDeferred.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s368.card.s382_limit">§ 382 annual NOL limit</div>
                    <div class="value">$${annualS382Limit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s368.card.years_nol">Years to use NOL</div>
                    <div class="value">${yearsToConsumeNOL.toFixed(1)}</div>
                </div>
            </div>
            ${!qualifies ? `
                <p class="muted small neg" style="margin-top:10px" data-i18n="view.s368.no_qualify_note">
                    Reorg requirements NOT met → TAXABLE transaction. Full target gain recognized at
                    corporate level (20%+ LTCG) + shareholder level (sale treatment) → double tax.
                </p>
            ` : ''}
        </div>
    `;
}
