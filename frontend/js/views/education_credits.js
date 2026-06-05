// Education Credits — AOC (American Opportunity) + LLC (Lifetime Learning).
// AOC: up to $2,500/student/yr first 4 years undergrad, 40% refundable.
// LLC: up to $2,000/return any post-secondary year, NON-refundable.
// Can't claim both for the same student in same year. AGI phase-outs apply.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const AOC_MAX = 2_500;
const LLC_MAX = 2_000;
const LLC_QE_CAP = 10_000;  // 20% × $10k = $2k cap
const AOC_PHASE_OUT_SINGLE = [80_000, 90_000];
const AOC_PHASE_OUT_MFJ = [160_000, 180_000];
const LLC_PHASE_OUT_SINGLE = [80_000, 90_000];
const LLC_PHASE_OUT_MFJ = [160_000, 180_000];

let state = {
    filing: 'single',
    magi: 75_000,
    students: [
        { id: '1', name: 'Self', qualified_expenses: 0, undergrad_year: 1, eligible_aoc: true },
    ],
};

export async function renderEducationCredits(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.edu.h1.title">// EDUCATION CREDITS</span></h1>
        <p class="muted small" data-i18n="view.edu.hint.intro">
            AOC: up to $2,500/student/yr (first 4 years undergrad, 40% REFUNDABLE).
            LLC: up to $2,000/return (any post-secondary, non-refundable). Can't claim
            both for same student same year. AGI phase-outs apply.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.edu.h2.shared_inputs">Shared inputs</h2>
            <form id="ec-shared" class="inline-form">
                <label><span data-i18n="view.edu.label.filing">Filing status</span>
                    <select name="filing">
                        <option value="single" ${state.filing === 'single' ? 'selected' : ''}>Single / HoH</option>
                        <option value="mfj" ${state.filing === 'mfj' ? 'selected' : ''}>MFJ</option>
                    </select>
                </label>
                <label><span data-i18n="view.edu.label.magi">MAGI ($)</span>
                    <input type="number" step="0.01" name="magi" value="${state.magi}"></label>
                <button class="primary" type="submit" data-i18n="view.edu.btn.update">Update</button>
            </form>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.edu.h2.students">Students</h2>
            <form id="ec-add-student" class="inline-form">
                <label><span data-i18n="view.edu.label.name">Student name</span>
                    <input type="text" name="name" required></label>
                <label><span data-i18n="view.edu.label.qe">Qualified expenses ($)</span>
                    <input type="number" step="0.01" name="qualified_expenses" value="0" required></label>
                <label><span data-i18n="view.edu.label.undergrad_year">Undergrad year (1-4 or 0)</span>
                    <input type="number" step="1" name="undergrad_year" value="1" min="0" max="4"></label>
                <label><span data-i18n="view.edu.label.eligible_aoc">Eligible for AOC?</span>
                    <input type="checkbox" name="eligible_aoc" checked></label>
                <button class="primary" type="submit" data-i18n="view.edu.btn.add_student">Add student</button>
            </form>
            <table class="trades" style="margin-top:10px">
                <thead><tr>
                    <th data-i18n="view.edu.th.name">Name</th>
                    <th data-i18n="view.edu.th.qe">QE</th>
                    <th data-i18n="view.edu.th.undergrad_year">UG year</th>
                    <th data-i18n="view.edu.th.aoc_eligible">AOC?</th>
                    <th data-i18n="view.edu.th.actions">Actions</th>
                </tr></thead>
                <tbody id="ec-student-rows"></tbody>
            </table>
        </div>
        <div id="ec-output"></div>
    `;
    document.getElementById('ec-shared').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.filing = fd.get('filing');
        state.magi = Number(fd.get('magi')) || 0;
        render();
    });
    document.getElementById('ec-add-student').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.students.push({
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            name: fd.get('name'),
            qualified_expenses: Number(fd.get('qualified_expenses')) || 0,
            undergrad_year: Number(fd.get('undergrad_year')) || 0,
            eligible_aoc: !!fd.get('eligible_aoc'),
        });
        e.target.reset();
        e.target.querySelector('[name="qualified_expenses"]').value = 0;
        e.target.querySelector('[name="undergrad_year"]').value = 1;
        e.target.querySelector('[name="eligible_aoc"]').checked = true;
        render();
    });
    render();
}

function render() {
    renderStudentRows();
    renderOutput();
}

function renderStudentRows() {
    const el = document.getElementById('ec-student-rows');
    if (!el) return;
    el.innerHTML = state.students.map(s => `
        <tr>
            <td>${esc(s.name)}</td>
            <td>$${s.qualified_expenses.toLocaleString()}</td>
            <td>${s.undergrad_year || '—'}</td>
            <td class="${s.eligible_aoc ? 'pos' : 'neg'}">${s.eligible_aoc ? '✓' : '×'}</td>
            <td><button class="link neg" data-del="${esc(s.id)}" data-i18n="view.edu.btn.delete">delete</button></td>
        </tr>
    `).join('');
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.students = state.students.filter(s => s.id !== btn.dataset.del);
            render();
        });
    });
}

function renderOutput() {
    const el = document.getElementById('ec-output');
    if (!el) return;
    const aocPO = state.filing === 'mfj' ? AOC_PHASE_OUT_MFJ : AOC_PHASE_OUT_SINGLE;
    const llcPO = state.filing === 'mfj' ? LLC_PHASE_OUT_MFJ : LLC_PHASE_OUT_SINGLE;
    const aocPct = phaseOutPct(state.magi, aocPO);
    const llcPct = phaseOutPct(state.magi, llcPO);
    // AOC: 100% of first $2k + 25% of next $2k = max $2,500
    const perStudent = state.students.map(s => {
        const aocRaw = s.eligible_aoc && s.undergrad_year >= 1 && s.undergrad_year <= 4
            ? Math.min(2_000, s.qualified_expenses) + 0.25 * Math.max(0, Math.min(2_000, s.qualified_expenses - 2_000))
            : 0;
        return { ...s, aocRaw, aocApplied: aocRaw * aocPct };
    });
    const totalAOC = perStudent.reduce((sum, s) => sum + s.aocApplied, 0);
    const totalQEforLLC = state.students.reduce((sum, s) => sum + s.qualified_expenses, 0);
    const llcRaw = Math.min(totalQEforLLC, LLC_QE_CAP) * 0.20;
    const llcApplied = llcRaw * llcPct;
    const refundableAOC = totalAOC * 0.40;
    const nonRefundableAOC = totalAOC * 0.60;
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.edu.h2.aoc">American Opportunity Credit</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.edu.card.aoc_total">Total AOC</div>
                    <div class="value">$${totalAOC.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.edu.card.phaseout">Phase-out factor</div>
                    <div class="value">${(aocPct * 100).toFixed(0)}%</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.edu.card.refundable">Refundable (40%)</div>
                    <div class="value">$${refundableAOC.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.edu.card.non_refundable">Non-refundable</div>
                    <div class="value">$${nonRefundableAOC.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.edu.h2.llc">Lifetime Learning Credit</h2>
            <div class="cards">
                <div class="card pos">
                    <div class="label" data-i18n="view.edu.card.llc">LLC</div>
                    <div class="value">$${llcApplied.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.edu.card.qe_max">QE cap applied</div>
                    <div class="value">$${Math.min(totalQEforLLC, LLC_QE_CAP).toLocaleString()}</div>
                </div>
                <div class="card">
                    <div class="label" data-i18n="view.edu.card.phaseout">Phase-out factor</div>
                    <div class="value">${(llcPct * 100).toFixed(0)}%</div>
                </div>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.edu.h2.recommendation">Recommendation</h2>
            <p>
                <strong>${esc(t('view.edu.rec.use_aoc'))}</strong> ${esc(t('view.edu.rec.aoc_when'))}
                <br><strong>${esc(t('view.edu.rec.use_llc'))}</strong> ${esc(t('view.edu.rec.llc_when'))}
            </p>
            <p class="muted small" data-i18n="view.edu.rec.note">
                Can mix per student per year — claim AOC for the qualifying undergrad,
                LLC for the grad-school spouse. Same dollar can't double-count.
            </p>
        </div>
    `;
}

function phaseOutPct(magi, range) {
    if (magi <= range[0]) return 1;
    if (magi >= range[1]) return 0;
    return 1 - (magi - range[0]) / (range[1] - range[0]);
}
