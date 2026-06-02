// Saver's Credit § 25B — Retirement Savings Contributions Credit.
// 10-50% credit on first $2k contributed to retirement accounts ($4k MFJ).
// Phaseout: 2024 single < $23k → 50%, $25k → 20%, $38,250 → 10%, $38,250+ → 0.
// MFJ doubles the thresholds. Up to $1k single / $2k MFJ. Non-refundable.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const SAVERS_2024_SINGLE = [
    [23_000,   0.50],
    [25_000,   0.20],
    [38_250,   0.10],
    [Infinity, 0.00],
];
const SAVERS_2024_HOH = [
    [34_500,   0.50],
    [37_500,   0.20],
    [57_375,   0.10],
    [Infinity, 0.00],
];
const SAVERS_2024_MFJ = [
    [46_000,   0.50],
    [50_000,   0.20],
    [76_500,   0.10],
    [Infinity, 0.00],
];
const MAX_CONTRIBUTION_FOR_CREDIT = 2_000;  // per spouse

let state = {
    filing: 'single',
    agi: 30_000,
    self_contribution: 2_000,
    spouse_contribution: 0,
    is_student: false,
    age: 30,
    dependent: false,
};

export async function renderSaversCredit(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.savers.h1.title">// SAVER'S CREDIT § 25B</span></h1>
        <p class="muted small" data-i18n="view.savers.hint.intro">
            10/20/50% credit on first $2,000 contributed to retirement accounts
            ($4,000 MFJ). Up to $1,000 single / $2,000 MFJ credit. Disqualifiers:
            full-time student, dependent claimed on someone else's return, under 18.
            Generous below $46k MFJ — overlooked because many qualifying people
            don't file. Non-refundable.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.savers.h2.inputs">Inputs</h2>
            <form id="sc-form" class="inline-form">
                <label><span data-i18n="view.savers.label.filing">Filing</span>
                    <select name="filing">
                        <option value="single" ${state.filing === 'single' ? 'selected' : ''}>Single</option>
                        <option value="hoh" ${state.filing === 'hoh' ? 'selected' : ''}>HoH</option>
                        <option value="mfj" ${state.filing === 'mfj' ? 'selected' : ''}>MFJ</option>
                    </select>
                </label>
                <label><span data-i18n="view.savers.label.agi">AGI ($)</span>
                    <input type="number" step="1000" name="agi" value="${state.agi}"></label>
                <label><span data-i18n="view.savers.label.self_contrib">Self retirement contributions ($)</span>
                    <input type="number" step="100" name="self_contribution" value="${state.self_contribution}"></label>
                <label><span data-i18n="view.savers.label.spouse_contrib">Spouse contributions ($)</span>
                    <input type="number" step="100" name="spouse_contribution" value="${state.spouse_contribution}"></label>
                <label><span data-i18n="view.savers.label.age">Age</span>
                    <input type="number" step="1" name="age" value="${state.age}" min="14" max="100"></label>
                <label><span data-i18n="view.savers.label.is_student">Full-time student?</span>
                    <input type="checkbox" name="is_student" ${state.is_student ? 'checked' : ''}></label>
                <label><span data-i18n="view.savers.label.dependent">Claimed as dependent?</span>
                    <input type="checkbox" name="dependent" ${state.dependent ? 'checked' : ''}></label>
                <button class="primary" type="submit" data-i18n="view.savers.btn.compute">Compute</button>
            </form>
        </div>
        <div id="sc-output"></div>
    `;
    document.getElementById('sc-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.filing = fd.get('filing');
        state.agi = Number(fd.get('agi')) || 0;
        state.self_contribution = Number(fd.get('self_contribution')) || 0;
        state.spouse_contribution = Number(fd.get('spouse_contribution')) || 0;
        state.age = Number(fd.get('age')) || 0;
        state.is_student = !!fd.get('is_student');
        state.dependent = !!fd.get('dependent');
        renderOutput();
    });
    renderOutput();
}

function renderOutput() {
    const el = document.getElementById('sc-output');
    if (!el) return;
    const disqualifiers = [];
    if (state.age < 18) disqualifiers.push(t('view.savers.dis.under_18'));
    if (state.is_student) disqualifiers.push(t('view.savers.dis.student'));
    if (state.dependent) disqualifiers.push(t('view.savers.dis.dependent'));

    const brackets = state.filing === 'mfj' ? SAVERS_2024_MFJ
        : state.filing === 'hoh' ? SAVERS_2024_HOH : SAVERS_2024_SINGLE;
    const creditRate = brackets.find(([cap, _]) => state.agi <= cap)?.[1] || 0;
    const selfQualifying = Math.min(state.self_contribution, MAX_CONTRIBUTION_FOR_CREDIT);
    const spouseQualifying = state.filing === 'mfj'
        ? Math.min(state.spouse_contribution, MAX_CONTRIBUTION_FOR_CREDIT)
        : 0;
    const credit = disqualifiers.length === 0
        ? (selfQualifying + spouseQualifying) * creditRate
        : 0;
    const cls = credit > 0 ? 'pos' : '';
    el.innerHTML = `
        <div class="chart-panel ${cls}">
            <h2 data-i18n="view.savers.h2.result">Credit calculation</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.savers.card.credit">Saver's Credit</div>
                    <div class="value">$${credit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.savers.card.rate">Credit rate @ AGI</div>
                    <div class="value">${(creditRate * 100).toFixed(0)}%</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.savers.card.qualifying">Qualifying contributions</div>
                    <div class="value">$${(selfQualifying + spouseQualifying).toLocaleString()}</div>
                </div>
            </div>
            ${disqualifiers.length > 0 ? `
                <h3 style="margin-top:10px" data-i18n="view.savers.h3.disqualifiers">Disqualifiers</h3>
                <ul class="muted small">${disqualifiers.map(d => `<li>${esc(d)}</li>`).join('')}</ul>
            ` : ''}
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.savers.h2.brackets">Phase-out brackets</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.savers.th.agi_range">AGI range</th>
                    <th data-i18n="view.savers.th.rate">Rate</th>
                </tr></thead>
                <tbody>${brackets.slice(0, -1).map(([cap, rate], i) => `
                    <tr>
                        <td>$${(i === 0 ? 0 : brackets[i - 1][0]).toLocaleString()} – $${cap.toLocaleString()}</td>
                        <td class="${state.agi <= cap && (i === 0 || state.agi > brackets[i - 1][0]) ? 'pos' : 'muted'}">${(rate * 100).toFixed(0)}%</td>
                    </tr>
                `).join('')}
                <tr>
                    <td>&gt; $${brackets[brackets.length - 2][0].toLocaleString()}</td>
                    <td class="${state.agi > brackets[brackets.length - 2][0] ? 'neg' : 'muted'}">0% (no credit)</td>
                </tr>
                </tbody>
            </table>
        </div>
    `;
}
