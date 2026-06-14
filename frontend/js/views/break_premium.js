// Meal & rest break premium generator — one hour of pay per missed break day,
// via /calc/break-premium.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderBreakPremium(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.breakprem.h1.title">// BREAK PREMIUM PAY</span></h1>
        <p class="muted small" data-i18n="view.breakprem.hint.intro">
            Under California Labor Code §226.7, an employer that fails to provide a compliant meal or rest
            break owes the employee one additional hour of pay at the regular rate for each workday the break
            is missed — a separate premium for meal and for rest, so up to two hours per day. It computes the
            meal premium, the rest premium, and the total. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.breakprem.h2.inputs">Premium inputs</h2>
            <form id="breakprem-form" class="inline-form">
                <label><span data-i18n="view.breakprem.label.state">State</span>
                    <input type="text" name="state" value="California" required></label>
                <label><span data-i18n="view.breakprem.label.employer">Employer</span>
                    <input type="text" name="employer_name" value=""></label>
                <label><span data-i18n="view.breakprem.label.employee">Employee</span>
                    <input type="text" name="employee_name" value=""></label>
                <label><span data-i18n="view.breakprem.label.rate">Regular rate ($/hr)</span>
                    <input type="number" step="0.01" min="0" name="regular_rate_usd" value="20" required></label>
                <label><span data-i18n="view.breakprem.label.meal">Meal-break violation days</span>
                    <input type="number" step="1" min="0" name="meal_violation_days" value="10"></label>
                <label><span data-i18n="view.breakprem.label.rest">Rest-break violation days</span>
                    <input type="number" step="1" min="0" name="rest_violation_days" value="5"></label>
                <label><span data-i18n="view.breakprem.label.period">Period</span>
                    <input type="text" name="period_label" value="Q2 2026"></label>
                <label><span data-i18n="view.breakprem.label.date">Statement date</span>
                    <input type="date" name="date" value="2026-07-01" required></label>
                <label><span data-i18n="view.breakprem.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.breakprem.ph.statute'))}"></label>
            </form>
        </div>
        <div id="breakprem-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#breakprem-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            employer_name: (fd.get('employer_name') || '').trim(),
            employee_name: (fd.get('employee_name') || '').trim(),
            regular_rate_usd: Number(fd.get('regular_rate_usd')) || 0,
            meal_violation_days: Number(fd.get('meal_violation_days')) || 0,
            rest_violation_days: Number(fd.get('rest_violation_days')) || 0,
            period_label: (fd.get('period_label') || '').trim(),
            date: fd.get('date'),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcBreakPremium(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.breakprem.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#breakprem-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.breakprem.card.total">Total premium</div>
                    <div class="value">${money(doc.total_premium_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.breakprem.card.meal">Meal premium</div>
                    <div class="value">${money(doc.meal_premium_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.breakprem.card.rest">Rest premium</div>
                    <div class="value">${money(doc.rest_premium_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.breakprem.card.hours">Premium hours</div>
                    <div class="value">${doc.total_premium_hours}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="breakprem-copy" type="button" data-i18n="view.breakprem.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="breakprem-download" type="button" data-i18n="view.breakprem.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#breakprem-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.breakprem.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.breakprem.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#breakprem-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'break-premium.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
