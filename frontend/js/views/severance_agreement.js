// Severance agreement generator — severance pay + total payout + release
// clauses (with ADEA timing for 40+), via /calc/severance. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderSeverance(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.sev.h1.title">// SEVERANCE AGREEMENT</span></h1>
        <p class="muted small" data-i18n="view.sev.hint.intro">
            Separates an employee with a severance payment in exchange for a release of claims. It
            computes severance pay from the weeks offered and the weekly salary, adds any accrued-PTO
            payout for the total, and assembles the release clauses — including the ADEA consideration and
            revocation windows when the employee is 40 or older. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.sev.h2.inputs">Severance details</h2>
            <form id="sev-form" class="inline-form">
                <label><span data-i18n="view.sev.label.company">Company</span>
                    <input type="text" name="company_name" value=""></label>
                <label><span data-i18n="view.sev.label.employee">Employee</span>
                    <input type="text" name="employee_name" value=""></label>
                <label><span data-i18n="view.sev.label.title">Job title</span>
                    <input type="text" name="job_title" value="Analyst"></label>
                <label><span data-i18n="view.sev.label.salary">Annual salary ($)</span>
                    <input type="number" step="100" min="0" name="annual_salary_usd" value="104000" required></label>
                <label><span data-i18n="view.sev.label.weeks">Severance weeks</span>
                    <input type="number" step="0.5" min="0" name="severance_weeks" value="8" required></label>
                <label><span data-i18n="view.sev.label.pto">Accrued PTO payout ($)</span>
                    <input type="number" step="0.01" min="0" name="accrued_pto_payout_usd" value="1000"></label>
                <label><span data-i18n="view.sev.label.date">Separation date</span>
                    <input type="date" name="separation_date" value="2026-06-30" required></label>
                <label><span data-i18n="view.sev.label.age">Employee 40 or older</span>
                    <input type="checkbox" name="age_40_or_over"></label>
                <label><span data-i18n="view.sev.label.state">State</span>
                    <input type="text" name="state" value="Illinois" required></label>
            </form>
        </div>
        <div id="sev-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#sev-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            company_name: (fd.get('company_name') || '').trim(),
            employee_name: (fd.get('employee_name') || '').trim(),
            job_title: (fd.get('job_title') || '').trim(),
            annual_salary_usd: Number(fd.get('annual_salary_usd')) || 0,
            severance_weeks: Number(fd.get('severance_weeks')) || 0,
            accrued_pto_payout_usd: Number(fd.get('accrued_pto_payout_usd')) || 0,
            separation_date: fd.get('separation_date'),
            age_40_or_over: fd.get('age_40_or_over') != null,
            state: (fd.get('state') || '').trim(),
        };
        try {
            const doc = await api.calcSeverance(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.sev.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#sev-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.sev.card.total">Total payout</div>
                    <div class="value">${money(doc.total_payout_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.sev.card.severance">Severance pay</div>
                    <div class="value">${money(doc.severance_pay_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.sev.card.weekly">Weekly pay</div>
                    <div class="value">${money(doc.weekly_pay_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="sev-copy" type="button" data-i18n="view.sev.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="sev-download" type="button" data-i18n="view.sev.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#sev-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.sev.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.sev.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#sev-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'severance-agreement.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
