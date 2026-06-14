// Final paycheck / waiting-time penalty generator — daily wage × days late
// (capped) plus unpaid final wages, via /calc/final-paycheck.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderFinalPaycheck(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.finalpay.h1.title">// FINAL PAYCHECK PENALTY</span></h1>
        <p class="muted small" data-i18n="view.finalpay.hint.intro">
            When an employer pays final wages late after a separation, many states impose a waiting-time
            penalty equal to the employee's daily wage for each day the payment is late, capped (California
            Labor Code §203 caps it at 30 days). It computes the daily wage, the capped penalty, and the total
            owed. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.finalpay.h2.inputs">Penalty inputs</h2>
            <form id="finalpay-form" class="inline-form">
                <label><span data-i18n="view.finalpay.label.state">State</span>
                    <input type="text" name="state" value="California" required></label>
                <label><span data-i18n="view.finalpay.label.employer">Employer</span>
                    <input type="text" name="employer_name" value=""></label>
                <label><span data-i18n="view.finalpay.label.employee">Employee</span>
                    <input type="text" name="employee_name" value=""></label>
                <label><span data-i18n="view.finalpay.label.wages">Unpaid final wages ($)</span>
                    <input type="number" step="50" min="0" name="final_wages_usd" value="1500"></label>
                <label><span data-i18n="view.finalpay.label.hourly">Hourly rate ($)</span>
                    <input type="number" step="0.01" min="0" name="hourly_rate_usd" value="25" required></label>
                <label><span data-i18n="view.finalpay.label.hpd">Hours per workday</span>
                    <input type="number" step="0.5" min="1" name="hours_per_day" value="8"></label>
                <label><span data-i18n="view.finalpay.label.dayslate">Days late</span>
                    <input type="number" step="1" min="0" name="days_late" value="10" required></label>
                <label><span data-i18n="view.finalpay.label.cap">Cap (days)</span>
                    <input type="number" step="1" min="0" name="cap_days" value="30"></label>
                <label><span data-i18n="view.finalpay.label.sepdate">Separation date</span>
                    <input type="date" name="separation_date" value="2026-06-01" required></label>
                <label><span data-i18n="view.finalpay.label.date">Statement date</span>
                    <input type="date" name="date" value="2026-07-01" required></label>
                <label><span data-i18n="view.finalpay.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.finalpay.ph.statute'))}"></label>
            </form>
        </div>
        <div id="finalpay-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#finalpay-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            employer_name: (fd.get('employer_name') || '').trim(),
            employee_name: (fd.get('employee_name') || '').trim(),
            final_wages_usd: Number(fd.get('final_wages_usd')) || 0,
            hourly_rate_usd: Number(fd.get('hourly_rate_usd')) || 0,
            hours_per_day: Number(fd.get('hours_per_day')) || 8,
            days_late: Number(fd.get('days_late')) || 0,
            cap_days: Number(fd.get('cap_days')) || 30,
            separation_date: fd.get('separation_date'),
            date: fd.get('date'),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcFinalPaycheck(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.finalpay.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);
    form.addEventListener('input', () => { live(); });
    form.addEventListener('submit', (e) => { e.preventDefault(); generate(); });
    generate();
}

function docToText(doc) {
    const lines = [doc.title.toUpperCase()];
    if (doc.statutory_citation) lines.push(doc.statutory_citation);
    lines.push('');
    for (const c of doc.clauses) lines.push(c.heading, c.body, '');
    return lines.join('\n');
}

function renderResult(mount, doc) {
    LAST_DOC = doc;
    const capNote = doc.cap_applied
        ? ` <span class="muted small" data-i18n="view.finalpay.capped">(capped)</span>`
        : '';
    const el = mount.querySelector('#finalpay-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.finalpay.card.total">Total owed</div>
                    <div class="value">${money(doc.total_owed_usd)}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.finalpay.card.penalty">Waiting-time penalty</div>
                    <div class="value">${money(doc.waiting_time_penalty_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.finalpay.card.days">Penalty days</div>
                    <div class="value">${doc.penalty_days}${capNote}</div></div>
                <div class="card"><div class="label" data-i18n="view.finalpay.card.daily">Daily wage</div>
                    <div class="value">${money(doc.daily_wage_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="finalpay-copy" type="button" data-i18n="view.finalpay.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="finalpay-download" type="button" data-i18n="view.finalpay.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#finalpay-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.finalpay.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.finalpay.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#finalpay-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'final-paycheck.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
