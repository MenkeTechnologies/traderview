// PTO balance statement generator — earned − used, capped, with payout value,
// via /calc/pto-balance.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const hrs = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderPtoBalance(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ptobal.h1.title">// PTO BALANCE</span></h1>
        <p class="muted small" data-i18n="view.ptobal.hint.intro">
            A point-in-time statement of an employee's paid-time-off balance — distinct from the PTO policy.
            It computes hours earned over the periods worked, subtracts hours used, applies any accrual cap,
            and values the remaining balance for a separation payout. Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.ptobal.h2.inputs">Balance inputs</h2>
            <form id="ptobal-form" class="inline-form">
                <label><span data-i18n="view.ptobal.label.state">State</span>
                    <input type="text" name="state" value="Delaware" required></label>
                <label><span data-i18n="view.ptobal.label.employer">Employer</span>
                    <input type="text" name="employer_name" value=""></label>
                <label><span data-i18n="view.ptobal.label.employee">Employee</span>
                    <input type="text" name="employee_name" value=""></label>
                <label><span data-i18n="view.ptobal.label.rate">Accrual (hours/period)</span>
                    <input type="number" step="0.01" min="0" name="accrual_rate_hours_per_period" value="5" required></label>
                <label><span data-i18n="view.ptobal.label.periods">Periods worked</span>
                    <input type="number" step="1" min="0" name="periods_worked" value="20" required></label>
                <label><span data-i18n="view.ptobal.label.used">Hours used</span>
                    <input type="number" step="1" min="0" name="hours_used" value="30"></label>
                <label><span data-i18n="view.ptobal.label.cap">Accrual cap (hours, 0 = none)</span>
                    <input type="number" step="1" min="0" name="accrual_cap_hours" value="80"></label>
                <label><span data-i18n="view.ptobal.label.hourly">Hourly rate ($)</span>
                    <input type="number" step="0.01" min="0" name="hourly_rate_usd" value="25"></label>
                <label><span data-i18n="view.ptobal.label.hpd">Hours per workday</span>
                    <input type="number" step="0.5" min="1" name="hours_per_day" value="8"></label>
                <label><span data-i18n="view.ptobal.label.date">As-of date</span>
                    <input type="date" name="as_of_date" value="2026-07-01" required></label>
                <label><span data-i18n="view.ptobal.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.ptobal.ph.statute'))}"></label>
            </form>
        </div>
        <div id="ptobal-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#ptobal-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            employer_name: (fd.get('employer_name') || '').trim(),
            employee_name: (fd.get('employee_name') || '').trim(),
            accrual_rate_hours_per_period: Number(fd.get('accrual_rate_hours_per_period')) || 0,
            periods_worked: Number(fd.get('periods_worked')) || 0,
            hours_used: Number(fd.get('hours_used')) || 0,
            accrual_cap_hours: Number(fd.get('accrual_cap_hours')) || 0,
            hourly_rate_usd: Number(fd.get('hourly_rate_usd')) || 0,
            hours_per_day: Number(fd.get('hours_per_day')) || 8,
            as_of_date: fd.get('as_of_date'),
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcPtoBalance(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.ptobal.toast.error'), { level: 'error' });
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
        ? ` <span class="muted small" data-i18n="view.ptobal.capped">(capped)</span>`
        : '';
    const el = mount.querySelector('#ptobal-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.ptobal.card.balance">Balance (hours)</div>
                    <div class="value">${hrs(doc.balance_hours)}${capNote}</div></div>
                <div class="card"><div class="label" data-i18n="view.ptobal.card.days">Balance (days)</div>
                    <div class="value">${hrs(doc.balance_days)}</div></div>
                <div class="card"><div class="label" data-i18n="view.ptobal.card.earned">Earned / used</div>
                    <div class="value">${hrs(doc.hours_earned)} / ${hrs(doc.hours_used)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.ptobal.card.payout">Payout value</div>
                    <div class="value">${money(doc.payout_value_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="ptobal-copy" type="button" data-i18n="view.ptobal.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="ptobal-download" type="button" data-i18n="view.ptobal.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#ptobal-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.ptobal.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.ptobal.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#ptobal-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'pto-balance.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
