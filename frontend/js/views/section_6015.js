// IRC § 6015 — Innocent Spouse Relief.
// 3 forms: (b) Traditional Innocent Spouse, (c) Separation of Liability, (f) Equitable Relief.
// Relieves spouse of joint+several liability for tax on joint return attributable to other spouse.
// Form 8857. 2-year deadline for (b) + (c); equitable (f) within Code § 6502 CSED 10-yr period.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

let state = {
    filed_jointly: true,
    understatement_attributable_to_spouse: 0,
    requester_share_of_understatement: 0,
    knew_or_had_reason: false,
    is_separated_divorced: false,
    months_since_separated: 0,
    significant_benefit_received: false,
    economic_hardship_if_held_liable: false,
    abuse_history: false,
    spouse_willful_evasion: false,
    notice_received_date_months: 0,
    request_relief_type: 'b_innocent',
};

export async function renderSection6015(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s6015.h1.title">// § 6015 INNOCENT SPOUSE RELIEF</span></h1>
        <p class="muted small" data-i18n="view.s6015.hint.intro">
            3 forms: <strong>(b) Traditional Innocent Spouse</strong>, <strong>(c) Separation of
            Liability</strong>, <strong>(f) Equitable Relief</strong>. Relieves spouse of joint +
            several liability for tax on joint return attributable to other spouse.
            <strong>Form 8857</strong>. 2-year deadline for (b) + (c) from first IRS collection
            activity; (f) equitable within § 6502 CSED 10-yr period. Notice 2024-26 updated rules.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s6015.h2.inputs">Inputs</h2>
            <form id="s6015-form" class="inline-form">
                <label><span data-i18n="view.s6015.label.filed_joint">Filed joint return?</span>
                    <input type="checkbox" name="filed_jointly" ${state.filed_jointly ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6015.label.understatement">Understatement attributable to spouse ($)</span>
                    <input type="number" step="0.01" name="understatement_attributable_to_spouse" value="${state.understatement_attributable_to_spouse}"></label>
                <label><span data-i18n="view.s6015.label.requester_share">Requester's share of understatement ($)</span>
                    <input type="number" step="0.01" name="requester_share_of_understatement" value="${state.requester_share_of_understatement}"></label>
                <label><span data-i18n="view.s6015.label.knew">Knew or had reason to know?</span>
                    <input type="checkbox" name="knew_or_had_reason" ${state.knew_or_had_reason ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6015.label.separated">Currently separated / divorced?</span>
                    <input type="checkbox" name="is_separated_divorced" ${state.is_separated_divorced ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6015.label.months_separated">Months since separated</span>
                    <input type="number" step="1" name="months_since_separated" value="${state.months_since_separated}"></label>
                <label><span data-i18n="view.s6015.label.significant_benefit">Significant benefit received?</span>
                    <input type="checkbox" name="significant_benefit_received" ${state.significant_benefit_received ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6015.label.hardship">Economic hardship if held liable?</span>
                    <input type="checkbox" name="economic_hardship_if_held_liable" ${state.economic_hardship_if_held_liable ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6015.label.abuse">Domestic abuse history?</span>
                    <input type="checkbox" name="abuse_history" ${state.abuse_history ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6015.label.willful">Spouse's willful evasion?</span>
                    <input type="checkbox" name="spouse_willful_evasion" ${state.spouse_willful_evasion ? 'checked' : ''}></label>
                <label><span data-i18n="view.s6015.label.notice_months">Months since IRS notice</span>
                    <input type="number" step="1" name="notice_received_date_months" value="${state.notice_received_date_months}"></label>
                <label><span data-i18n="view.s6015.label.type">Relief type sought</span>
                    <select name="request_relief_type">
                        <option value="b_innocent" ${state.request_relief_type === 'b_innocent' ? 'selected' : ''}>(b) Traditional Innocent Spouse</option>
                        <option value="c_separation" ${state.request_relief_type === 'c_separation' ? 'selected' : ''}>(c) Separation of Liability</option>
                        <option value="f_equitable" ${state.request_relief_type === 'f_equitable' ? 'selected' : ''}>(f) Equitable Relief</option>
                    </select>
                </label>
                <button class="primary" type="submit" data-i18n="view.s6015.btn.compute">Compute</button>
            </form>
        </div>
        <div id="s6015-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6015.h2.comparison">Relief comparison</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s6015.th.type">Type</th>
                    <th data-i18n="view.s6015.th.who">Who qualifies</th>
                    <th data-i18n="view.s6015.th.deadline">Deadline</th>
                </tr></thead>
                <tbody>
                    <tr><td>(b)</td><td data-i18n="view.s6015.row.b_who">Joint return + understatement from spouse + no knowledge + inequitable to hold liable</td><td>2 yrs from 1st collection</td></tr>
                    <tr><td>(c)</td><td data-i18n="view.s6015.row.c_who">No longer married / separated 12+ mo; allocate as if MFS</td><td>2 yrs from 1st collection</td></tr>
                    <tr><td>(f)</td><td data-i18n="view.s6015.row.f_who">Catch-all when (b)/(c) unavailable; Rev. Proc. 2013-34 weighing factors</td><td>10-yr CSED window</td></tr>
                </tbody>
            </table>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6015.h2.equitable_factors">Rev. Proc. 2013-34 Equitable Relief factors</h2>
            <ul class="muted small">
                <li data-i18n="view.s6015.eq.marital_status">Marital status (separated / divorced favors)</li>
                <li data-i18n="view.s6015.eq.economic_hardship">Economic hardship (favors)</li>
                <li data-i18n="view.s6015.eq.knowledge">Knowledge or reason to know (against)</li>
                <li data-i18n="view.s6015.eq.legal_obligation">Legal obligation to pay (against if assumed)</li>
                <li data-i18n="view.s6015.eq.significant_benefit">Significant benefit (against)</li>
                <li data-i18n="view.s6015.eq.compliance">Compliance with current law (favors)</li>
                <li data-i18n="view.s6015.eq.health">Mental / physical health at filing (favors)</li>
                <li data-i18n="view.s6015.eq.abuse">Domestic abuse / financial control (strongly favors)</li>
            </ul>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s6015.h2.injured_spouse">Distinguish: § 6402 Injured Spouse (Form 8379)</h2>
            <p class="muted small" data-i18n="view.s6015.injured.body">
                <strong>Injured spouse:</strong> wants own SHARE of joint refund preserved when
                IRS plans to offset for spouse's separate debt (child support, student loans,
                back taxes). Filed simultaneously with return or up to 3 yrs after.
                <strong>Innocent spouse:</strong> wants relief from joint liability already
                assessed. Different forms, different facts, different timing.
            </p>
        </div>
    `;
    document.getElementById('s6015-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.filed_jointly = !!fd.get('filed_jointly');
        state.understatement_attributable_to_spouse = Number(fd.get('understatement_attributable_to_spouse')) || 0;
        state.requester_share_of_understatement = Number(fd.get('requester_share_of_understatement')) || 0;
        state.knew_or_had_reason = !!fd.get('knew_or_had_reason');
        state.is_separated_divorced = !!fd.get('is_separated_divorced');
        state.months_since_separated = Number(fd.get('months_since_separated')) || 0;
        state.significant_benefit_received = !!fd.get('significant_benefit_received');
        state.economic_hardship_if_held_liable = !!fd.get('economic_hardship_if_held_liable');
        state.abuse_history = !!fd.get('abuse_history');
        state.spouse_willful_evasion = !!fd.get('spouse_willful_evasion');
        state.notice_received_date_months = Number(fd.get('notice_received_date_months')) || 0;
        state.request_relief_type = fd.get('request_relief_type');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('s6015-output');
    if (!el) return;
    if (!state.filed_jointly) {
        el.innerHTML = `<div class="chart-panel"><p class="muted small neg" data-i18n="view.s6015.warning.not_joint">Innocent spouse relief only applies to joint returns. Use Form 1040-X to amend or consider Equitable Relief if Form 8857 still timely.</p></div>`;
        return;
    }
    const within2Years = state.notice_received_date_months <= 24;
    let bQualifies = within2Years && !state.knew_or_had_reason
        && state.understatement_attributable_to_spouse > 0 && !state.spouse_willful_evasion;
    let cQualifies = within2Years && state.is_separated_divorced && state.months_since_separated >= 12;
    let factorScore = 0;
    if (state.is_separated_divorced) factorScore += 2;
    if (state.economic_hardship_if_held_liable) factorScore += 2;
    if (!state.knew_or_had_reason) factorScore += 1;
    if (state.abuse_history) factorScore += 3;
    if (state.significant_benefit_received) factorScore -= 2;
    if (state.spouse_willful_evasion) factorScore += 1;
    let fLikelihood;
    if (factorScore >= 5) fLikelihood = 'view.s6015.likelihood.high';
    else if (factorScore >= 2) fLikelihood = 'view.s6015.likelihood.medium';
    else fLikelihood = 'view.s6015.likelihood.low';
    let recommendedRelief, recommendedAmount;
    if (bQualifies) {
        recommendedRelief = 'view.s6015.rel.b';
        recommendedAmount = state.requester_share_of_understatement;
    } else if (cQualifies) {
        recommendedRelief = 'view.s6015.rel.c';
        recommendedAmount = state.understatement_attributable_to_spouse;
    } else {
        recommendedRelief = 'view.s6015.rel.f';
        recommendedAmount = factorScore >= 2 ? state.requester_share_of_understatement * 0.50 : 0;
    }
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s6015.h2.result">Relief analysis</h2>
            <div class="cards">
                <div class="card ${within2Years ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s6015.card.timing">Within 2-yr deadline</div>
                    <div class="value">${within2Years ? esc(t('view.s6015.status.yes')) : esc(t('view.s6015.status.no'))}</div>
                </div>
                <div class="card ${bQualifies ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s6015.card.b_qualifies">(b) Innocent qualifies</div>
                    <div class="value">${bQualifies ? esc(t('view.s6015.status.yes')) : esc(t('view.s6015.status.no'))}</div>
                </div>
                <div class="card ${cQualifies ? 'pos' : 'neg'}">
                    <div class="label" data-i18n="view.s6015.card.c_qualifies">(c) Separation qualifies</div>
                    <div class="value">${cQualifies ? esc(t('view.s6015.status.yes')) : esc(t('view.s6015.status.no'))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6015.card.f_likelihood">(f) Equitable likelihood</div>
                    <div class="value">${esc(t(fLikelihood))}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.s6015.card.factor_score">Factor score</div>
                    <div class="value">${factorScore}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s6015.card.recommended">Recommended path</div>
                    <div class="value">${esc(t(recommendedRelief))}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s6015.card.recovered">Likely recovery</div>
                    <div class="value">$${recommendedAmount.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
    `;
}
