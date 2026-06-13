// Employee disciplinary write-up generator — level → consequence + escalation,
// via /calc/employee-writeup. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
let LAST_DOC = null;

export async function renderEmployeeWriteup(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ew.h1.title">// EMPLOYEE WRITE-UP</span></h1>
        <p class="muted small" data-i18n="view.ew.hint.intro">
            Documents a workplace incident and the corrective action. The disciplinary level (verbal →
            written → final → termination) drives the consequence and the next-step escalation language.
            Drafting aid, not legal/HR advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.ew.h2.inputs">Write-up details</h2>
            <form id="ew-form" class="inline-form">
                <label><span data-i18n="view.ew.label.company">Company</span>
                    <input type="text" name="company_name" value=""></label>
                <label><span data-i18n="view.ew.label.employee">Employee</span>
                    <input type="text" name="employee_name" value=""></label>
                <label><span data-i18n="view.ew.label.title">Job title</span>
                    <input type="text" name="job_title" value="Technician"></label>
                <label><span data-i18n="view.ew.label.manager">Manager</span>
                    <input type="text" name="manager_name" value=""></label>
                <label><span data-i18n="view.ew.label.date">Incident date</span>
                    <input type="date" name="incident_date" value="2026-06-10" required></label>
                <label><span data-i18n="view.ew.label.type">Violation type</span>
                    <input type="text" name="violation_type" value="Attendance"></label>
                <label><span data-i18n="view.ew.label.desc">Description</span>
                    <input type="text" name="description" value="Three unexcused absences in two weeks."></label>
                <label><span data-i18n="view.ew.label.level">Disciplinary level</span>
                    <select name="level">
                        <option value="verbal" data-i18n="view.ew.level.verbal">Verbal warning</option>
                        <option value="written" selected data-i18n="view.ew.level.written">Written warning</option>
                        <option value="final" data-i18n="view.ew.level.final">Final warning</option>
                        <option value="termination" data-i18n="view.ew.level.termination">Termination</option>
                    </select></label>
                <label><span data-i18n="view.ew.label.prior">Prior warnings on file</span>
                    <input type="number" step="1" min="0" name="prior_warnings" value="1"></label>
                <label><span data-i18n="view.ew.label.corrective">Corrective action</span>
                    <input type="text" name="corrective_action" value=""></label>
            </form>
        </div>
        <div id="ew-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#ew-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            company_name: (fd.get('company_name') || '').trim(),
            employee_name: (fd.get('employee_name') || '').trim(),
            manager_name: (fd.get('manager_name') || '').trim(),
            job_title: (fd.get('job_title') || '').trim(),
            incident_date: fd.get('incident_date'),
            violation_type: (fd.get('violation_type') || '').trim(),
            description: (fd.get('description') || '').trim(),
            level: fd.get('level'),
            prior_warnings: Math.round(Number(fd.get('prior_warnings')) || 0),
            corrective_action: (fd.get('corrective_action') || '').trim(),
        };
        try {
            const doc = await api.calcEmployeeWriteup(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.ew.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);
    form.addEventListener('input', () => { live(); });
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function docToText(doc) {
    const lines = [doc.title.toUpperCase(), ''];
    for (const c of doc.clauses) lines.push(c.heading, c.body, '');
    return lines.join('\n');
}

function renderResult(mount, doc) {
    LAST_DOC = doc;
    const el = mount.querySelector('#ew-result');
    const isTerm = doc.level === 'Termination';
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card ${isTerm ? 'neg' : ''}"><div class="label" data-i18n="view.ew.card.level">Level</div>
                    <div class="value">${esc(doc.level)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ew.card.prior">Prior warnings</div>
                    <div class="value">${doc.prior_warnings}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="ew-copy" type="button" data-i18n="view.ew.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="ew-download" type="button" data-i18n="view.ew.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#ew-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.ew.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.ew.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#ew-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'employee-writeup.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
