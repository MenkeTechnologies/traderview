// Timesheet generator — regular + overtime hours → gross pay, via
// /calc/timesheet. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const num = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderTimesheet(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ts.h1.title">// TIMESHEET</span></h1>
        <p class="muted small" data-i18n="view.ts.hint.intro">
            Records an hourly employee's hours for a pay period and computes gross pay: regular hours at
            the hourly rate plus overtime hours at the rate times the overtime multiplier (1.5× by
            default). Drafting aid, not payroll/tax advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.ts.h2.inputs">Timesheet details</h2>
            <form id="ts-form" class="inline-form">
                <label><span data-i18n="view.ts.label.company">Company</span>
                    <input type="text" name="company_name" value=""></label>
                <label><span data-i18n="view.ts.label.employee">Employee</span>
                    <input type="text" name="employee_name" value=""></label>
                <label><span data-i18n="view.ts.label.start">Period start</span>
                    <input type="date" name="period_start" value="2026-06-01" required></label>
                <label><span data-i18n="view.ts.label.end">Period end</span>
                    <input type="date" name="period_end" value="2026-06-14" required></label>
                <label><span data-i18n="view.ts.label.reg">Regular hours</span>
                    <input type="number" step="0.25" min="0" name="regular_hours" value="80" required></label>
                <label><span data-i18n="view.ts.label.rate">Hourly rate ($)</span>
                    <input type="number" step="0.01" min="0" name="hourly_rate_usd" value="25" required></label>
                <label><span data-i18n="view.ts.label.ot">Overtime hours</span>
                    <input type="number" step="0.25" min="0" name="overtime_hours" value="5"></label>
                <label><span data-i18n="view.ts.label.mult">OT multiplier</span>
                    <input type="number" step="0.1" min="1" name="overtime_multiplier" value="1.5"></label>
            </form>
        </div>
        <div id="ts-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#ts-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            company_name: (fd.get('company_name') || '').trim(),
            employee_name: (fd.get('employee_name') || '').trim(),
            period_start: fd.get('period_start'),
            period_end: fd.get('period_end'),
            regular_hours: Number(fd.get('regular_hours')) || 0,
            hourly_rate_usd: Number(fd.get('hourly_rate_usd')) || 0,
            overtime_hours: Number(fd.get('overtime_hours')) || 0,
            overtime_multiplier: Number(fd.get('overtime_multiplier')) || 1.5,
        };
        try {
            const doc = await api.calcTimesheet(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.ts.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#ts-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.ts.card.gross">Gross pay</div>
                    <div class="value">${money(doc.gross_pay_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ts.card.reg">Regular pay</div>
                    <div class="value">${money(doc.regular_pay_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ts.card.ot">Overtime pay</div>
                    <div class="value">${money(doc.overtime_pay_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ts.card.hours">Total hours</div>
                    <div class="value">${num(doc.total_hours)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="ts-copy" type="button" data-i18n="view.ts.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="ts-download" type="button" data-i18n="view.ts.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#ts-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.ts.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.ts.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#ts-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'timesheet.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
