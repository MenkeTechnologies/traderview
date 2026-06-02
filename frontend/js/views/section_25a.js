// IRC § 25A American Opportunity Credit (AOTC) + Lifetime Learning Credit (LLC).
// AOTC: $2,500/student/yr (100% of first $2k + 25% of next $2k). First 4 yrs only.
// 40% refundable. MAGI phase-out: $80k-$90k single / $160k-$180k MFJ. Half-time enrollment.
// LLC: 20% of first $10k = $2k/return (NOT per student). Unlimited years. MAGI phase-out same.

import { esc } from '../util.js';
import { t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';

const AOTC_MAX = 2_500;
const AOTC_FIRST_TIER = 2_000;
const LLC_MAX = 2_000;
const LLC_PCT = 0.20;
const AOTC_PHASEOUT_LOW_SINGLE = 80_000;
const AOTC_PHASEOUT_HIGH_SINGLE = 90_000;
const AOTC_PHASEOUT_LOW_MFJ = 160_000;
const AOTC_PHASEOUT_HIGH_MFJ = 180_000;

let state = {
    filing_status: 'single',
    magi: 0,
    students: [],
    other_education_expenses: 0,
};

export async function renderSection25a(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.s25a.h1.title">// § 25A AOTC + LLC EDUCATION CREDITS</span></h1>
        <p class="muted small" data-i18n="view.s25a.hint.intro">
            <strong>AOTC:</strong> $2,500/student/year — first 4 undergraduate years, ≥ half-time,
            no felony drug conviction. <strong>40% refundable</strong>. <strong>LLC:</strong>
            $2,000 / RETURN (not per student) — 20% of first $10k. Unlimited years. Less generous
            but covers graduate / part-time. MAGI phase-outs: $80-90k single / $160-180k MFJ.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.s25a.h2.inputs">Inputs</h2>
            <form id="s25a-form" class="inline-form">
                <label><span data-i18n="view.s25a.label.filing">Filing status</span>
                    <select name="filing_status">
                        <option value="single" ${state.filing_status === 'single' ? 'selected' : ''}>Single / HoH</option>
                        <option value="mfj" ${state.filing_status === 'mfj' ? 'selected' : ''}>MFJ</option>
                    </select>
                </label>
                <label><span data-i18n="view.s25a.label.magi">MAGI ($)</span>
                    <input type="number" step="1000" name="magi" value="${state.magi}"></label>
                <button class="primary" type="submit" data-i18n="view.s25a.btn.compute">Compute</button>
            </form>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s25a.h2.add_student">Add student</h2>
            <form id="s25a-sform" class="inline-form">
                <label><span data-i18n="view.s25a.label.name">Student name</span>
                    <input type="text" name="name" required></label>
                <label><span data-i18n="view.s25a.label.expenses">Qualified expenses ($)</span>
                    <input type="number" step="100" name="expenses" required></label>
                <label><span data-i18n="view.s25a.label.year">Undergrad year (1-4)</span>
                    <input type="number" step="1" min="1" max="6" name="undergrad_year" value="1"></label>
                <label><span data-i18n="view.s25a.label.half_time">≥ Half-time?</span>
                    <input type="checkbox" name="half_time" checked></label>
                <label><span data-i18n="view.s25a.label.degree_seeking">Degree seeking?</span>
                    <input type="checkbox" name="degree_seeking" checked></label>
                <label><span data-i18n="view.s25a.label.felony_drug">Felony drug conviction?</span>
                    <input type="checkbox" name="felony_drug"></label>
                <button class="primary" type="submit" data-i18n="view.s25a.btn.add_student">Add</button>
            </form>
        </div>
        <div id="s25a-output"></div>
        <div class="chart-panel">
            <h2 data-i18n="view.s25a.h2.expenses">Qualified expenses</h2>
            <ul class="muted small">
                <li data-i18n="view.s25a.exp.tuition">Tuition + required fees</li>
                <li data-i18n="view.s25a.exp.books_aotc">AOTC ONLY: required books + supplies + equipment (whether or not from school)</li>
                <li data-i18n="view.s25a.exp.books_llc">LLC: required ONLY if purchased FROM school as condition of enrollment</li>
                <li data-i18n="view.s25a.exp.not_room">NOT room & board, transportation, insurance, medical, optional fees</li>
                <li data-i18n="view.s25a.exp.scholarship_offset">Reduce expenses by tax-free scholarships / grants / employer assistance</li>
                <li data-i18n="view.s25a.exp.529_no_double">529 plan distributions: don't double-claim same dollars (electable allocation)</li>
            </ul>
        </div>
    `;
    document.getElementById('s25a-form').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.filing_status = fd.get('filing_status');
        state.magi = Number(fd.get('magi')) || 0;
        renderOutput();
    });
    document.getElementById('s25a-sform').addEventListener('submit', (e) => {
        e.preventDefault();
        const fd = new FormData(e.target);
        state.students.push({
            id: crypto.randomUUID ? crypto.randomUUID() : String(Date.now()) + Math.random(),
            name: fd.get('name'),
            expenses: Number(fd.get('expenses')) || 0,
            undergrad_year: Number(fd.get('undergrad_year')) || 1,
            half_time: !!fd.get('half_time'),
            degree_seeking: !!fd.get('degree_seeking'),
            felony_drug: !!fd.get('felony_drug'),
        });
        e.target.reset();
        renderOutput();
    });
    renderOutput();
}

function phaseOutFactor() {
    const low = state.filing_status === 'mfj' ? AOTC_PHASEOUT_LOW_MFJ : AOTC_PHASEOUT_LOW_SINGLE;
    const high = state.filing_status === 'mfj' ? AOTC_PHASEOUT_HIGH_MFJ : AOTC_PHASEOUT_HIGH_SINGLE;
    if (state.magi <= low) return 1;
    if (state.magi >= high) return 0;
    return (high - state.magi) / (high - low);
}

function renderOutput() {
    const el = document.getElementById('s25a-output');
    if (state.students.length === 0) {
        if (el) el.innerHTML = '';
        return;
    }
    if (!el) return;
    const factor = phaseOutFactor();
    let aotcTotal = 0;
    const aotcByStudent = [];
    for (const s of state.students) {
        const eligibleAOTC = s.half_time && s.degree_seeking && !s.felony_drug && s.undergrad_year <= 4;
        if (eligibleAOTC) {
            const tier1 = Math.min(s.expenses, AOTC_FIRST_TIER) * 1.0;
            const tier2 = Math.min(Math.max(0, s.expenses - AOTC_FIRST_TIER), AOTC_FIRST_TIER) * 0.25;
            const raw = Math.min(tier1 + tier2, AOTC_MAX);
            const credit = raw * factor;
            aotcTotal += credit;
            aotcByStudent.push({ student: s, eligible: true, credit, refundable: credit * 0.40 });
        } else {
            aotcByStudent.push({ student: s, eligible: false, credit: 0, refundable: 0 });
        }
    }
    const totalQualifiedForLLC = state.students.reduce((s, st) => s + st.expenses, 0);
    const llcRaw = Math.min(totalQualifiedForLLC * LLC_PCT, LLC_MAX);
    const llcCredit = llcRaw * factor;
    const totalRefundable = aotcByStudent.reduce((s, x) => s + x.refundable, 0);
    const chosenStrategy = aotcTotal > llcCredit ? 'AOTC per student' : 'LLC (single $2k per return)';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.s25a.h2.result">Credit calculation</h2>
            <div class="cards">
                <div class="card">
                    <div class="label" data-i18n="view.s25a.card.factor">Phase-out factor</div>
                    <div class="value">${(factor * 100).toFixed(0)}%</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s25a.card.aotc_total">AOTC total</div>
                    <div class="value">$${aotcTotal.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s25a.card.refundable">Refundable portion (40%)</div>
                    <div class="value">$${totalRefundable.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s25a.card.llc">LLC alternative</div>
                    <div class="value">$${llcCredit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</div>
                </div>
                <div class="card pos">
                    <div class="label" data-i18n="view.s25a.card.best">Best strategy</div>
                    <div class="value">${esc(chosenStrategy)}</div>
                </div>
            </div>
        </div>
        <div class="chart-panel">
            <h2 data-i18n="view.s25a.h2.students">Students</h2>
            <table class="trades">
                <thead><tr>
                    <th data-i18n="view.s25a.th.name">Name</th>
                    <th data-i18n="view.s25a.th.expenses">Qualified expenses</th>
                    <th data-i18n="view.s25a.th.aotc_eligible">AOTC eligible</th>
                    <th data-i18n="view.s25a.th.credit">AOTC credit</th>
                    <th data-i18n="view.s25a.th.actions">Actions</th>
                </tr></thead>
                <tbody>${aotcByStudent.map(x => `
                    <tr>
                        <td>${esc(x.student.name)}</td>
                        <td>$${x.student.expenses.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td class="${x.eligible ? 'pos' : 'neg'}">${x.eligible ? esc(t('view.s25a.status.yes')) : esc(t('view.s25a.status.no'))}</td>
                        <td class="pos">$${x.credit.toLocaleString(undefined, { maximumFractionDigits: 0 })}</td>
                        <td><button class="link neg" data-del="${esc(x.student.id)}" data-i18n="view.s25a.btn.delete">delete</button></td>
                    </tr>
                `).join('')}</tbody>
            </table>
        </div>
    `;
    el.querySelectorAll('[data-del]').forEach(btn => {
        btn.addEventListener('click', () => {
            state.students = state.students.filter(s => s.id !== btn.dataset.del);
            renderOutput();
        });
    });
}
