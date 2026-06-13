// Business invoice generator — line items, discount, tax, due date, via
// /calc/invoice-generator. Renders a printable invoice you can copy or
// download as text.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';

const TEXT = [
    ['business_name', 'Your business', 'Acme LLC'],
    ['business_address', 'Your address', '1 Industrial Way'],
    ['client_name', 'Bill to', 'Beta Corp'],
    ['client_address', 'Client address', '2 Market St'],
    ['invoice_number', 'Invoice #', 'INV-001'],
];
const DEFAULT_LINES = [
    ['Consulting (hrs)', 10, 150],
    ['Materials', 2, 75],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));

let LAST_DOC = null;

function lineRow(desc = '', qty = 1, price = 0) {
    return `<tr class="inv-line">
        <td><input type="text" class="inv-desc" value="${esc(desc)}" placeholder="${esc(t('view.invoice.ph.desc'))}"></td>
        <td><input type="number" step="0.01" min="0" class="inv-qty" value="${qty}"></td>
        <td><input type="number" step="0.01" min="0" class="inv-price" value="${price}"></td>
        <td><button type="button" class="btn btn-secondary inv-del" data-i18n="view.invoice.btn.del">✕</button></td>
    </tr>`;
}

export async function renderInvoiceGenerator(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.invoice.h1.title">// BUSINESS INVOICE GENERATOR</span></h1>
        <p class="muted small" data-i18n="view.invoice.hint.intro">
            Build a professional invoice from your line items. It computes each line's
            amount, the subtotal, an optional discount off the subtotal, tax on the
            discounted subtotal, and the total — all rounded to cents so the numbers
            reconcile — plus the due date from your net-N payment terms. Copy or download
            the finished invoice.
        </p>
        <div class="chart-panel">
            <h2 data-i18n="view.invoice.h2.inputs">Invoice details</h2>
            <form id="inv-form" class="inline-form">
                ${TEXT.map(([key, label, def]) => `
                    <label><span data-i18n="view.invoice.label.${key}">${label}</span>
                        <input type="text" name="${key}" value="${esc(def)}" required></label>
                `).join('')}
                <label><span data-i18n="view.invoice.label.invoice_date">Invoice date</span>
                    <input type="date" name="invoice_date" value="2026-06-01" required></label>
                <label><span data-i18n="view.invoice.label.payment_terms_days">Payment terms (net days)</span>
                    <input type="number" step="1" min="0" name="payment_terms_days" value="30" required></label>
                <label><span data-i18n="view.invoice.label.discount_pct">Discount (%)</span>
                    <input type="number" step="0.01" min="0" name="discount_pct" value="0" required></label>
                <label><span data-i18n="view.invoice.label.tax_rate_pct">Tax rate (%)</span>
                    <input type="number" step="0.01" min="0" name="tax_rate_pct" value="0" required></label>
                <label class="inv-notes"><span data-i18n="view.invoice.label.notes">Notes</span>
                    <input type="text" name="notes" value=""></label>
            </form>
            <h3 data-i18n="view.invoice.h3.lines">Line items</h3>
            <table class="data-table" id="inv-lines">
                <thead><tr>
                    <th data-i18n="view.invoice.col.desc">Description</th>
                    <th data-i18n="view.invoice.col.qty">Qty</th>
                    <th data-i18n="view.invoice.col.price">Unit price</th>
                    <th></th>
                </tr></thead>
                <tbody>${DEFAULT_LINES.map((l) => lineRow(...l)).join('')}</tbody>
            </table>
            <p>
                <button type="button" class="btn btn-secondary" id="inv-add" data-i18n="view.invoice.btn.add">+ Add line</button>
                <button type="button" class="btn" id="inv-gen" data-i18n="view.invoice.btn.run">Generate invoice</button>
            </p>
        </div>
        <div id="inv-result"></div>
    `;
    applyUiI18n(mount);

    const linesBody = mount.querySelector('#inv-lines tbody');
    mount.querySelector('#inv-add').addEventListener('click', () => {
        linesBody.insertAdjacentHTML('beforeend', lineRow());
        applyUiI18n(linesBody.lastElementChild);
    });
    linesBody.addEventListener('click', (e) => {
        if (e.target.classList.contains('inv-del')) {
            const rows = linesBody.querySelectorAll('.inv-line');
            if (rows.length > 1) e.target.closest('tr').remove();
        }
    });

    mount.querySelector('#inv-gen').addEventListener('click', () => submit(mount, tok));
    submit(mount, tok);
}

async function submit(mount, tok) {
    const form = mount.querySelector('#inv-form');
    const fd = new FormData(form);
    const body = { line_items: [] };
    for (const [key] of TEXT) body[key] = (fd.get(key) || '').trim();
    body.invoice_date = fd.get('invoice_date');
    body.payment_terms_days = Math.round(Number(fd.get('payment_terms_days')) || 0);
    body.discount_pct = Number(fd.get('discount_pct')) || 0;
    body.tax_rate_pct = Number(fd.get('tax_rate_pct')) || 0;
    body.notes = (fd.get('notes') || '').trim();
    for (const row of mount.querySelectorAll('#inv-lines .inv-line')) {
        const description = row.querySelector('.inv-desc').value.trim();
        const quantity = Number(row.querySelector('.inv-qty').value) || 0;
        const unit_price_usd = Number(row.querySelector('.inv-price').value) || 0;
        if (description || quantity || unit_price_usd) body.line_items.push({ description, quantity, unit_price_usd });
    }
    try {
        const doc = await api.calcInvoiceGenerator(body);
        if (!viewIsCurrent(tok)) return;
        renderResult(mount, doc);
    } catch (err) {
        showToast(err.message || t('view.invoice.toast.error'), { level: 'error' });
    }
}

function docToText(doc) {
    const lines = [
        `INVOICE ${doc.invoice_number}`,
        `From: ${doc.business_name}${doc.business_address ? ', ' + doc.business_address : ''}`,
        `Bill to: ${doc.client_name}${doc.client_address ? ', ' + doc.client_address : ''}`,
        `Date: ${doc.invoice_date}   Due: ${doc.due_date} (Net ${doc.payment_terms_days})`,
        '',
    ];
    for (const l of doc.lines) {
        lines.push(`  ${l.description} — ${l.quantity} × ${money(l.unit_price_usd)} = ${money(l.amount_usd)}`);
    }
    lines.push('', `Subtotal: ${money(doc.subtotal_usd)}`);
    if (doc.discount_amount_usd > 0) lines.push(`Discount (${doc.discount_pct}%): -${money(doc.discount_amount_usd)}`);
    if (doc.tax_amount_usd > 0) lines.push(`Tax (${doc.tax_rate_pct}%): ${money(doc.tax_amount_usd)}`);
    lines.push(`TOTAL: ${money(doc.total_usd)}`);
    if (doc.notes) lines.push('', `Notes: ${doc.notes}`);
    return lines.join('\n');
}

function renderResult(mount, doc) {
    LAST_DOC = doc;
    const el = mount.querySelector('#inv-result');
    const rows = doc.lines.map((l) => `
        <tr><td>${esc(l.description)}</td><td>${l.quantity}</td>
            <td>${money(l.unit_price_usd)}</td><td>${money(l.amount_usd)}</td></tr>
    `).join('');
    const discountRow = doc.discount_amount_usd > 0
        ? `<tr><td colspan="3" class="num">${t('view.invoice.row.discount')} (${doc.discount_pct}%)</td><td class="neg">-${money(doc.discount_amount_usd)}</td></tr>`
        : '';
    const taxRow = doc.tax_amount_usd > 0
        ? `<tr><td colspan="3" class="num">${t('view.invoice.row.tax')} (${doc.tax_rate_pct}%)</td><td>${money(doc.tax_amount_usd)}</td></tr>`
        : '';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.invoice.h2.result">Invoice</h2>
            <p>
                <button class="btn btn-secondary" id="inv-copy" type="button" data-i18n="view.invoice.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="inv-download" type="button" data-i18n="view.invoice.btn.download">Download .txt</button>
            </p>
            <p class="muted small">
                <strong>${esc(doc.invoice_number)}</strong> — ${esc(doc.business_name)} →
                ${esc(doc.client_name)} · <span data-i18n="view.invoice.due">Due</span> ${esc(doc.due_date)}
                (<span data-i18n="view.invoice.net">Net</span> ${doc.payment_terms_days})
            </p>
            <table class="data-table">
                <thead><tr>
                    <th data-i18n="view.invoice.col.desc">Description</th>
                    <th data-i18n="view.invoice.col.qty">Qty</th>
                    <th data-i18n="view.invoice.col.price">Unit price</th>
                    <th data-i18n="view.invoice.col.amount">Amount</th>
                </tr></thead>
                <tbody>
                    ${rows}
                    <tr><td colspan="3" class="num" data-i18n="view.invoice.row.subtotal">Subtotal</td><td>${money(doc.subtotal_usd)}</td></tr>
                    ${discountRow}
                    ${taxRow}
                    <tr class="emph"><td colspan="3" class="num" data-i18n="view.invoice.row.total">Total</td><td class="pos">${money(doc.total_usd)}</td></tr>
                </tbody>
            </table>
            ${doc.notes ? `<p class="muted small">${esc(doc.notes)}</p>` : ''}
        </div>
    `;
    applyUiI18n(el);

    el.querySelector('#inv-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.invoice.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.invoice.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#inv-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = (LAST_DOC.invoice_number || 'invoice') + '.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
