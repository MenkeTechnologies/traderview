// Split-shift premium generator — one hour at minimum wage, offset by earnings
// above minimum wage, via /calc/split-shift-premium.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderSplitShiftPremium(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.splitshift.h1.title">// SPLIT-SHIFT PREMIUM</span></h1>
        <p class="muted small" data-i18n="view.splitshift.hint.intro">
            When an employer schedules a work day with an unpaid, non-meal gap (a split shift), California law
            owes the employee one additional hour of pay at the minimum wage — offset by the amount the
            employee's earnings for the day already exceed the minimum wage. It computes the one-hour premium,
            the offset, and the net premium owed. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.splitshift.h2.inputs">Premium inputs</h2>
            <form id="split-form" class="inline-form">
                <label><span data-i18n="view.splitshift.label.state">State</span>
                    <input type="text" name="state" value="California" required></label>
                <label><span data-i18n="view.splitshift.label.employer">Employer</span>
                    <input type="text" name="employer_name" value=""></label>
                <label><span data-i18n="view.splitshift.label.employee">Employee</span>
                    <input type="text" name="employee_name" value=""></label>
                <label><span data-i18n="view.splitshift.label.minwage">Minimum wage ($/hr)</span>
                    <input type="number" step="0.25" min="0" name="min_wage_usd" value="16" required></label>
                <label><span data-i18n="view.splitshift.label.rate">Regular rate ($/hr)</span>
                    <input type="number" step="0.01" min="0" name="regular_rate_usd" value="16" required></label>
                <label><span data-i18n="view.splitshift.label.hours">Hours worked</span>
                    <input type="number" step="0.25" min="0" name="hours_worked" value="8" required></label>
                <label><span data-i18n="view.splitshift.label.shiftdate">Shift date</span>
                    <input type="date" name="shift_date" value="2026-06-20" required></label>
                <label><span data-i18n="view.splitshift.label.date">Statement date</span>
                    <input type="date" name="date" value="2026-07-01" required></label>
                <label><span data-i18n="view.splitshift.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.splitshift.ph.statute'))}"></label>
            </form>
        </div>
        <div id="split-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#split-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            employer_name: (fd.get('employer_name') || '').trim(),
            employee_name: (fd.get('employee_name') || '').trim(),
            min_wage_usd: Number(fd.get('min_wage_usd')) || 0,
            regular_rate_usd: Number(fd.get('regular_rate_usd')) || 0,
            hours_worked: Number(fd.get('hours_worked')) || 0,
            shift_date: fd.get('shift_date'),
            date: fd.get('date'),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcSplitShiftPremium(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.splitshift.toast.error'), { level: 'error' });
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
    const offsetNote = doc.fully_offset
        ? ` <span class="muted small" data-i18n="view.splitshift.offset">(fully offset)</span>`
        : '';
    const el = mount.querySelector('#split-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.splitshift.card.net">Net premium</div>
                    <div class="value">${money(doc.net_premium_usd)}${offsetNote}</div></div>
                <div class="card"><div class="label" data-i18n="view.splitshift.card.onehour">One-hour premium</div>
                    <div class="value">${money(doc.one_hour_premium_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.splitshift.card.offset">Earnings above min</div>
                    <div class="value">${money(doc.earnings_above_min_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="split-copy" type="button" data-i18n="view.splitshift.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="split-download" type="button" data-i18n="view.splitshift.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#split-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.splitshift.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.splitshift.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#split-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'split-shift-premium.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
