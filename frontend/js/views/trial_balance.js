// Trial balance generator — list accounts by debit/credit and verify totals are
// equal, via /calc/trial-balance.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

const SEED = [
    { name: 'Cash', debit: 5000, credit: 0 },
    { name: 'Accounts receivable', debit: 3000, credit: 0 },
    { name: 'Equipment', debit: 12000, credit: 0 },
    { name: 'Accounts payable', debit: 0, credit: 4000 },
    { name: 'Notes payable', debit: 0, credit: 6000 },
    { name: "Owner's capital", debit: 0, credit: 10000 },
];

function rowHtml(a) {
    return `
        <div class="mpb-row nec-row">
            <input type="text" class="tb-name" placeholder="${esc(t('view.tb.ph.name'))}" value="${esc(a.name || '')}">
            <input type="number" step="100" min="0" class="tb-debit" value="${a.debit}">
            <input type="number" step="100" min="0" class="tb-credit" value="${a.credit}">
            <button type="button" class="tb-del" data-i18n="view.tb.remove">Remove</button>
        </div>`;
}

export async function renderTrialBalance(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.tb.h1.title">// TRIAL BALANCE</span></h1>
        <p class="muted small" data-i18n="view.tb.hint.intro">
            Lists every general-ledger account with its debit or credit balance and verifies that total debits
            equal total credits — the basic check that the books are in balance under double-entry accounting.
            Drafting aid, not accounting advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.tb.h2.inputs">Ledger details</h2>
            <form id="tb-form" class="inline-form">
                <label><span data-i18n="view.tb.label.company">Company</span>
                    <input type="text" name="company_name" value="Acme Co" required></label>
                <label><span data-i18n="view.tb.label.asof">As-of date</span>
                    <input type="date" name="as_of_date" value="2026-06-30" required></label>
                <label><span data-i18n="view.tb.label.date">Prepared date</span>
                    <input type="date" name="date" value="2026-07-01" required></label>
            </form>
            <div class="mpb-head nec-head">
                <span data-i18n="view.tb.col.name">Account</span>
                <span data-i18n="view.tb.col.debit">Debit ($)</span>
                <span data-i18n="view.tb.col.credit">Credit ($)</span>
                <span></span>
            </div>
            <div id="tb-rows">${SEED.map(rowHtml).join('')}</div>
            <button type="button" id="tb-add" class="secondary" data-i18n="view.tb.add">+ Add account</button>
        </div>
        <div id="tb-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#tb-form');
    const rowsEl = mount.querySelector('#tb-rows');

    const generate = async () => {
        const accounts = [...rowsEl.querySelectorAll('.nec-row')].map((r) => ({
            name: (r.querySelector('.tb-name').value || '').trim(),
            debit_usd: Number(r.querySelector('.tb-debit').value) || 0,
            credit_usd: Number(r.querySelector('.tb-credit').value) || 0,
        })).filter((a) => a.name);
        const body = {
            company_name: (form.querySelector('[name="company_name"]').value || '').trim(),
            as_of_date: form.querySelector('[name="as_of_date"]').value,
            accounts,
            date: form.querySelector('[name="date"]').value,
        };
        try {
            const doc = await api.calcTrialBalance(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.tb.toast.error'), { level: 'error' });
        }
    };
    const live = debounce(generate, 250);

    mount.querySelector('#tb-add').addEventListener('click', () => {
        rowsEl.insertAdjacentHTML('beforeend', rowHtml({ name: '', debit: 0, credit: 0 }));
        applyUiI18n(rowsEl.lastElementChild);
        generate();
    });
    rowsEl.addEventListener('click', (e) => {
        if (e.target.classList.contains('tb-del')) {
            e.target.closest('.nec-row').remove();
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
    const rows = doc.rows.map((r) => `
        <tr><td>${esc(r.name)}</td><td>${r.debit_usd ? money(r.debit_usd) : ''}</td><td>${r.credit_usd ? money(r.credit_usd) : ''}</td></tr>
    `).join('');
    const statusKey = doc.balanced ? 'view.tb.balanced' : 'view.tb.unbalanced';
    const statusCls = doc.balanced ? 'pos' : 'neg';
    const el = mount.querySelector('#tb-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card ${statusCls}"><div class="label" data-i18n="view.tb.card.status">Status</div>
                    <div class="value" data-i18n="${statusKey}">${doc.balanced ? 'Balanced' : 'Out of balance'}</div></div>
                <div class="card"><div class="label" data-i18n="view.tb.card.debits">Total debits</div>
                    <div class="value">${money(doc.total_debits_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.tb.card.credits">Total credits</div>
                    <div class="value">${money(doc.total_credits_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.tb.card.diff">Difference</div>
                    <div class="value">${money(doc.difference_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="tb-copy" type="button" data-i18n="view.tb.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="tb-download" type="button" data-i18n="view.tb.btn.download">Download .txt</button>
            </div>
        </div>
        <table class="data-table">
            <thead><tr>
                <th data-i18n="view.tb.th.name">Account</th>
                <th data-i18n="view.tb.th.debit">Debit</th>
                <th data-i18n="view.tb.th.credit">Credit</th>
            </tr></thead>
            <tbody>${rows}</tbody>
        </table>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#tb-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.tb.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.tb.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#tb-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'trial-balance.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
