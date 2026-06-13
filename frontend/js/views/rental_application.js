// Rental application generator — income-to-rent qualification + application
// clauses, via /calc/rental-application. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderRentalApplication(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.ra.h1.title">// RENTAL APPLICATION</span></h1>
        <p class="muted small" data-i18n="view.ra.hint.intro">
            The applicant intake a landlord collects before approving a tenancy. Beyond the
            applicant/employment fields, it runs the income qualification landlords screen on: the
            income-to-rent multiple and whether gross monthly income meets the required multiple of rent
            (commonly 3×). Drafting aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.ra.h2.inputs">Application details</h2>
            <form id="ra-form" class="inline-form">
                <label><span data-i18n="view.ra.label.applicant">Applicant name</span>
                    <input type="text" name="applicant_name" value=""></label>
                <label><span data-i18n="view.ra.label.premises">Property applied for</span>
                    <input type="text" name="premises_address" value=""></label>
                <label><span data-i18n="view.ra.label.rent">Monthly rent ($)</span>
                    <input type="number" step="0.01" min="0" name="monthly_rent_usd" value="2000" required></label>
                <label><span data-i18n="view.ra.label.income">Gross monthly income ($)</span>
                    <input type="number" step="0.01" min="0" name="gross_monthly_income_usd" value="6500" required></label>
                <label><span data-i18n="view.ra.label.multiple">Required income multiple (× rent)</span>
                    <input type="number" step="0.1" min="0" name="required_income_multiple" value="3" required></label>
                <label><span data-i18n="view.ra.label.employer">Employer</span>
                    <input type="text" name="employer" value=""></label>
                <label><span data-i18n="view.ra.label.position">Position</span>
                    <input type="text" name="position" value=""></label>
                <label><span data-i18n="view.ra.label.current">Current address</span>
                    <input type="text" name="current_address" value=""></label>
                <label><span data-i18n="view.ra.label.movein">Desired move-in</span>
                    <input type="date" name="move_in_date" value="2026-08-01"></label>
            </form>
        </div>
        <div id="ra-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#ra-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            applicant_name: (fd.get('applicant_name') || '').trim(),
            premises_address: (fd.get('premises_address') || '').trim(),
            monthly_rent_usd: Number(fd.get('monthly_rent_usd')) || 0,
            gross_monthly_income_usd: Number(fd.get('gross_monthly_income_usd')) || 0,
            required_income_multiple: Number(fd.get('required_income_multiple')) || 3,
            employer: (fd.get('employer') || '').trim(),
            position: (fd.get('position') || '').trim(),
            current_address: (fd.get('current_address') || '').trim(),
            move_in_date: fd.get('move_in_date') || '',
        };
        try {
            const doc = await api.calcRentalApplication(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.ra.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#ra-result');
    const qKey = doc.qualifies ? 'view.ra.status.qualifies' : 'view.ra.status.short';
    const qCls = doc.qualifies ? 'pos' : 'neg';
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card ${qCls}"><div class="label" data-i18n="view.ra.card.status">Income screen</div>
                    <div class="value" data-i18n="${qKey}"></div></div>
                <div class="card"><div class="label" data-i18n="view.ra.card.multiple">Income / rent</div>
                    <div class="value">${doc.income_multiple}×</div></div>
                <div class="card"><div class="label" data-i18n="view.ra.card.required">Required income</div>
                    <div class="value">${money(doc.required_income_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="ra-copy" type="button" data-i18n="view.ra.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="ra-download" type="button" data-i18n="view.ra.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#ra-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.ra.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.ra.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#ra-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'rental-application.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
