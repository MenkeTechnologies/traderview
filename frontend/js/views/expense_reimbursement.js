// Expense reimbursement request generator — itemized expenses + mileage →
// total reimbursement, via /calc/expense-reimbursement. Live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

const SEED = [
    { desc: 'Hotel', amt: 120 },
    { desc: 'Meals', amt: 50 },
    { desc: 'Parking', amt: 30 },
];

function rowHtml(e) {
    return `
        <div class="mpb-row sdi-row">
            <input type="text" class="er-desc" placeholder="${esc(t('view.er.ph.desc'))}" value="${esc(e.desc || '')}">
            <input type="number" step="0.01" min="0" class="er-amt" placeholder="${esc(t('view.er.ph.amt'))}" value="${e.amt}">
            <button type="button" class="er-del" data-i18n="view.er.remove">Remove</button>
        </div>`;
}

export async function renderExpenseReimbursement(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.er.h1.title">// EXPENSE REIMBURSEMENT</span></h1>
        <p class="muted small" data-i18n="view.er.hint.intro">
            An employee's itemized claim for reimbursement of business expenses plus mileage. It sums the
            itemized expenses, computes the mileage reimbursement (business miles × the mileage rate), and
            totals the two, then assembles the request with an approval line. Drafting aid, not legal/tax
            advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.er.h2.inputs">Request details</h2>
            <form id="er-form" class="inline-form">
                <label><span data-i18n="view.er.label.company">Company</span>
                    <input type="text" name="company_name" value=""></label>
                <label><span data-i18n="view.er.label.employee">Employee</span>
                    <input type="text" name="employee_name" value=""></label>
                <label><span data-i18n="view.er.label.period">Period / purpose</span>
                    <input type="text" name="period" value="June 2026 client trip"></label>
                <label><span data-i18n="view.er.label.miles">Business miles</span>
                    <input type="number" step="0.1" min="0" name="business_miles" value="100"></label>
                <label><span data-i18n="view.er.label.rate">Mileage rate ($/mi)</span>
                    <input type="number" step="0.001" min="0" name="mileage_rate_usd" value="0.67"></label>
                <label><span data-i18n="view.er.label.date">Submitted date</span>
                    <input type="date" name="submitted_date" value="2026-06-30" required></label>
            </form>
            <div class="mpb-head sdi-head">
                <span data-i18n="view.er.col.desc">Expense</span>
                <span data-i18n="view.er.col.amt">Amount ($)</span>
                <span></span>
            </div>
            <div id="er-rows">${SEED.map(rowHtml).join('')}</div>
            <button type="button" id="er-add" class="secondary" data-i18n="view.er.add">+ Add expense</button>
        </div>
        <div id="er-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#er-form');
    const rowsEl = mount.querySelector('#er-rows');

    const generate = async () => {
        const fd = new FormData(form);
        const expenses = [...rowsEl.querySelectorAll('.sdi-row')].map((r) => ({
            description: (r.querySelector('.er-desc').value || '').trim(),
            amount_usd: Number(r.querySelector('.er-amt').value) || 0,
        })).filter((e) => e.description || e.amount_usd);
        const body = {
            company_name: (fd.get('company_name') || '').trim(),
            employee_name: (fd.get('employee_name') || '').trim(),
            period: (fd.get('period') || '').trim(),
            expenses,
            business_miles: Number(fd.get('business_miles')) || 0,
            mileage_rate_usd: Number(fd.get('mileage_rate_usd')) || 0,
            submitted_date: fd.get('submitted_date'),
        };
        try {
            const doc = await api.calcExpenseReimbursement(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.er.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);

    mount.querySelector('#er-add').addEventListener('click', () => {
        rowsEl.insertAdjacentHTML('beforeend', rowHtml({ desc: '', amt: '' }));
        applyUiI18n(rowsEl.lastElementChild);
        generate();
    });
    rowsEl.addEventListener('click', (e) => {
        if (e.target.classList.contains('er-del')) {
            e.target.closest('.sdi-row').remove();
            generate();
        }
    });
    form.addEventListener('input', () => { live(); });
    rowsEl.addEventListener('input', () => { live(); });
    generate();
}

function docToText(doc) {
    const lines = [doc.title.toUpperCase(), ''];
    for (const c of doc.clauses) lines.push(c.heading, c.body, '');
    return lines.join('\n');
}

function renderResult(mount, doc) {
    LAST_DOC = doc;
    const el = mount.querySelector('#er-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.er.card.total">Total reimbursement</div>
                    <div class="value">${money(doc.total_reimbursement_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.er.card.expenses">Expenses</div>
                    <div class="value">${money(doc.expenses_total_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.er.card.mileage">Mileage</div>
                    <div class="value">${money(doc.mileage_reimbursement_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="er-copy" type="button" data-i18n="view.er.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="er-download" type="button" data-i18n="view.er.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#er-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.er.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.er.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#er-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'expense-reimbursement.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
