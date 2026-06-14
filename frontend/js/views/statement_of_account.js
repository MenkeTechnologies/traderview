// Statement of account generator — aggregates outstanding invoices into aging
// buckets (current/31–60/61–90/90+), via /calc/statement-of-account.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

const SEED = [
    { number: 'INV-001', date: '2026-06-01', amount: 1000 },
    { number: 'INV-002', date: '2026-05-01', amount: 2000 },
    { number: 'INV-003', date: '2026-03-15', amount: 1500 },
    { number: 'INV-004', date: '2026-01-01', amount: 500 },
];

function rowHtml(r) {
    return `
        <div class="mpb-row soa-row">
            <input type="text" class="soa-num" placeholder="${esc(t('view.soa.ph.number'))}" value="${esc(r.number || '')}">
            <input type="date" class="soa-date" value="${esc(r.date || '')}">
            <input type="number" step="0.01" min="0" class="soa-amount" value="${r.amount}">
            <button type="button" class="soa-del" data-i18n="view.soa.remove">Remove</button>
        </div>`;
}

export async function renderStatementOfAccount(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.soa.h1.title">// STATEMENT OF ACCOUNT</span></h1>
        <p class="muted small" data-i18n="view.soa.hint.intro">
            An accounts-receivable statement that aggregates a customer's outstanding invoices and ages each
            by days outstanding into standard buckets (current 0–30, 31–60, 61–90, 90+). It computes the
            per-bucket totals and the total balance due. Drafting aid, not legal/accounting advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.soa.h2.inputs">Statement details</h2>
            <form id="soa-form" class="inline-form">
                <label><span data-i18n="view.soa.label.company">From (company)</span>
                    <input type="text" name="company_name" value="Acme Supply Co" required></label>
                <label><span data-i18n="view.soa.label.customer">To (customer)</span>
                    <input type="text" name="customer_name" value="Beta Retail LLC" required></label>
                <label><span data-i18n="view.soa.label.date">Statement date</span>
                    <input type="date" name="statement_date" value="2026-06-13" required></label>
                <label><span data-i18n="view.soa.label.note">Note (optional)</span>
                    <input type="text" name="note" value="" placeholder="${esc(t('view.soa.ph.note'))}"></label>
            </form>
            <div class="mpb-head soa-head">
                <span data-i18n="view.soa.col.number">Invoice #</span>
                <span data-i18n="view.soa.col.date">Date</span>
                <span data-i18n="view.soa.col.amount">Amount ($)</span>
                <span></span>
            </div>
            <div id="soa-rows">${SEED.map(rowHtml).join('')}</div>
            <button type="button" id="soa-add" class="secondary" data-i18n="view.soa.add">+ Add invoice</button>
        </div>
        <div id="soa-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#soa-form');
    const rowsEl = mount.querySelector('#soa-rows');

    const generate = async () => {
        const fd = new FormData(form);
        const invoices = [...rowsEl.querySelectorAll('.soa-row')].map((r) => ({
            number: (r.querySelector('.soa-num').value || '').trim(),
            date: r.querySelector('.soa-date').value,
            amount_usd: Number(r.querySelector('.soa-amount').value) || 0,
        })).filter((inv) => inv.number && inv.date);
        const body = {
            company_name: (fd.get('company_name') || '').trim(),
            customer_name: (fd.get('customer_name') || '').trim(),
            statement_date: fd.get('statement_date'),
            invoices,
            note: (fd.get('note') || '').trim(),
        };
        try {
            const doc = await api.calcStatementOfAccount(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.soa.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);

    mount.querySelector('#soa-add').addEventListener('click', () => {
        rowsEl.insertAdjacentHTML('beforeend', rowHtml({ number: '', date: '', amount: 0 }));
        applyUiI18n(rowsEl.lastElementChild);
        generate();
    });
    rowsEl.addEventListener('click', (e) => {
        if (e.target.classList.contains('soa-del')) {
            e.target.closest('.soa-row').remove();
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
    const el = mount.querySelector('#soa-result');
    const rows = doc.rows.map((r) => `
        <tr><td>${esc(r.number)}</td><td>${esc(r.date)}</td><td>${money(r.amount_usd)}</td>
            <td>${r.days_outstanding}</td><td>${esc(r.bucket)}</td></tr>
    `).join('');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.soa.card.total">Total due</div>
                    <div class="value">${money(doc.total_due_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.soa.card.current">Current</div>
                    <div class="value">${money(doc.current_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.soa.card.b31">31–60</div>
                    <div class="value">${money(doc.b31_60_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.soa.card.b61">61–90</div>
                    <div class="value">${money(doc.b61_90_usd)}</div></div>
                <div class="card neg"><div class="label" data-i18n="view.soa.card.over90">90+</div>
                    <div class="value">${money(doc.over_90_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="soa-copy" type="button" data-i18n="view.soa.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="soa-download" type="button" data-i18n="view.soa.btn.download">Download .txt</button>
            </div>
        </div>
        <table class="data-table">
            <thead><tr>
                <th data-i18n="view.soa.th.number">Invoice #</th>
                <th data-i18n="view.soa.th.date">Date</th>
                <th data-i18n="view.soa.th.amount">Amount</th>
                <th data-i18n="view.soa.th.days">Days</th>
                <th data-i18n="view.soa.th.bucket">Bucket</th>
            </tr></thead>
            <tbody>${rows}</tbody>
        </table>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#soa-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.soa.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.soa.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#soa-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'statement-of-account.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
