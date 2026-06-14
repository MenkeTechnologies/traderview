// Reporting-time pay generator — clamped half-shift minimum guarantee and
// additional owed, via /calc/reporting-time-pay.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const hrs = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderReportingTimePay(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.rtp.h1.title">// REPORTING-TIME PAY</span></h1>
        <p class="muted small" data-i18n="view.rtp.hint.intro">
            When an employee reports for a scheduled shift but is sent home early, California law guarantees
            pay for half the scheduled shift, with a two-hour minimum and a four-hour maximum, at the regular
            rate. It computes the reporting-time minimum, the hours guaranteed, and any additional pay owed
            beyond the hours actually worked. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.rtp.h2.inputs">Shift inputs</h2>
            <form id="rtp-form" class="inline-form">
                <label><span data-i18n="view.rtp.label.state">State</span>
                    <input type="text" name="state" value="California" required></label>
                <label><span data-i18n="view.rtp.label.employer">Employer</span>
                    <input type="text" name="employer_name" value=""></label>
                <label><span data-i18n="view.rtp.label.employee">Employee</span>
                    <input type="text" name="employee_name" value=""></label>
                <label><span data-i18n="view.rtp.label.scheduled">Scheduled hours</span>
                    <input type="number" step="0.5" min="0" name="scheduled_hours" value="8" required></label>
                <label><span data-i18n="view.rtp.label.worked">Hours worked</span>
                    <input type="number" step="0.25" min="0" name="hours_worked" value="1"></label>
                <label><span data-i18n="view.rtp.label.rate">Regular rate ($/hr)</span>
                    <input type="number" step="0.01" min="0" name="regular_rate_usd" value="20" required></label>
                <label><span data-i18n="view.rtp.label.min">Minimum hours</span>
                    <input type="number" step="0.5" min="0" name="min_hours" value="2"></label>
                <label><span data-i18n="view.rtp.label.max">Maximum hours</span>
                    <input type="number" step="0.5" min="0" name="max_hours" value="4"></label>
                <label><span data-i18n="view.rtp.label.shiftdate">Shift date</span>
                    <input type="date" name="shift_date" value="2026-06-20" required></label>
                <label><span data-i18n="view.rtp.label.date">Statement date</span>
                    <input type="date" name="date" value="2026-07-01" required></label>
                <label><span data-i18n="view.rtp.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.rtp.ph.statute'))}"></label>
            </form>
        </div>
        <div id="rtp-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#rtp-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            employer_name: (fd.get('employer_name') || '').trim(),
            employee_name: (fd.get('employee_name') || '').trim(),
            scheduled_hours: Number(fd.get('scheduled_hours')) || 0,
            hours_worked: Number(fd.get('hours_worked')) || 0,
            regular_rate_usd: Number(fd.get('regular_rate_usd')) || 0,
            min_hours: Number(fd.get('min_hours')) || 0,
            max_hours: Number(fd.get('max_hours')) || 0,
            shift_date: fd.get('shift_date'),
            date: fd.get('date'),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcReportingTimePay(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.rtp.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#rtp-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.rtp.card.guaranteed">Guaranteed pay</div>
                    <div class="value">${money(doc.guaranteed_pay_usd)}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.rtp.card.additional">Additional owed</div>
                    <div class="value">${money(doc.additional_owed_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.rtp.card.min">Reporting minimum</div>
                    <div class="value">${hrs(doc.reporting_min_hours)} h</div></div>
                <div class="card"><div class="label" data-i18n="view.rtp.card.hours">Guaranteed hours</div>
                    <div class="value">${hrs(doc.guaranteed_hours)} h</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="rtp-copy" type="button" data-i18n="view.rtp.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="rtp-download" type="button" data-i18n="view.rtp.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#rtp-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.rtp.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.rtp.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#rtp-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'reporting-time-pay.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
