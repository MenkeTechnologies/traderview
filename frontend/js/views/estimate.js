// Estimate / quote generator — line items, discount, tax, valid-until date, via
// /calc/estimate. Shares the line-item math with the invoice. Renders a
// printable quote you can copy or download.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const TEXT = [
    ['business_name', 'Your business', 'Acme LLC'],
    ['business_address', 'Your address', '1 Industrial Way'],
    ['client_name', 'Quote for', 'Beta Corp'],
    ['client_address', 'Client address', '2 Market St'],
    ['estimate_number', 'Estimate #', 'EST-001'],
];
const DEFAULT_LINES = [
    ['Consulting (hrs)', 10, 150],
    ['Materials', 2, 75],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));

let LAST_DOC = null;

function lineRow(desc = '', qty = 1, price = 0) {
    return `<tr class="est-line">
        <td><input type="text" class="est-desc" value="${esc(desc)}" placeholder="${esc(t('view.estimate.ph.desc'))}"></td>
        <td><input type="number" step="0.01" min="0" class="est-qty" value="${qty}"></td>
        <td><input type="number" step="0.01" min="0" class="est-price" value="${price}"></td>
        <td><button type="button" class="btn btn-secondary est-del" data-i18n="view.estimate.btn.del">✕</button></td>
    </tr>`;
}

export async function renderEstimate(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.estimate.h1.title">// ESTIMATE / QUOTE GENERATOR</span></h1>
        <p class="muted small" data-i18n="view.estimate.hint.intro">
            A non-binding price quote to send before work is authorized. It uses the same line-item math
            as the invoice — each line's amount, the subtotal, an optional discount, tax on the discounted
            subtotal, and the total, all rounded to cents — plus a valid-until date from your validity
            window. Copy or download the finished quote.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.estimate.h2.inputs">Estimate details</h2>
            <form id="est-form" class="inline-form">
                ${TEXT.map(([key, label, def]) => `
                    <label><span data-i18n="view.estimate.label.${key}">${label}</span>
                        <input type="text" name="${key}" value="${esc(def)}" required></label>
                `).join('')}
                <label><span data-i18n="view.estimate.label.estimate_date">Estimate date</span>
                    <input type="date" name="estimate_date" value="2026-06-01" required></label>
                <label><span data-i18n="view.estimate.label.valid_days">Valid for (days)</span>
                    <input type="number" step="1" min="0" name="valid_days" value="30" required></label>
                <label><span data-i18n="view.estimate.label.discount_pct">Discount (%)</span>
                    <input type="number" step="0.01" min="0" name="discount_pct" value="0" required></label>
                <label><span data-i18n="view.estimate.label.tax_rate_pct">Tax rate (%)</span>
                    <input type="number" step="0.01" min="0" name="tax_rate_pct" value="0" required></label>
                <label><span data-i18n="view.estimate.label.notes">Notes</span>
                    <input type="text" name="notes" value=""></label>
            </form>
            <h3 data-i18n="view.estimate.h3.lines">Line items</h3>
            <table class="data-table" id="est-lines">
                <thead><tr>
                    <th data-i18n="view.estimate.col.desc">Description</th>
                    <th data-i18n="view.estimate.col.qty">Qty</th>
                    <th data-i18n="view.estimate.col.price">Unit price</th>
                    <th></th>
                </tr></thead>
                <tbody>${DEFAULT_LINES.map((l) => lineRow(...l)).join('')}</tbody>
            </table>
            <p>
                <button type="button" class="btn btn-secondary" id="est-add" data-i18n="view.estimate.btn.add">+ Add line</button>
            </p>
        </div>
        <div id="est-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const linesBody = mount.querySelector('#est-lines tbody');
    const live = debounce(() => submit(mount, tok), 250);
    mount.querySelector('#est-add').addEventListener('click', () => {
        linesBody.insertAdjacentHTML('beforeend', lineRow());
        applyUiI18n(linesBody.lastElementChild);
        submit(mount, tok);
    });
    linesBody.addEventListener('click', (e) => {
        if (e.target.classList.contains('est-del')) {
            const rows = linesBody.querySelectorAll('.est-line');
            if (rows.length > 1) { e.target.closest('tr').remove(); submit(mount, tok); }
        }
    });
    mount.querySelector('#est-form').addEventListener('input', live);
    linesBody.addEventListener('input', live);
    submit(mount, tok);
}

async function submit(mount, tok) {
    const form = mount.querySelector('#est-form');
    const fd = new FormData(form);
    const body = { line_items: [] };
    for (const [key] of TEXT) body[key] = (fd.get(key) || '').trim();
    body.estimate_date = fd.get('estimate_date');
    body.valid_days = Math.round(Number(fd.get('valid_days')) || 0);
    body.discount_pct = Number(fd.get('discount_pct')) || 0;
    body.tax_rate_pct = Number(fd.get('tax_rate_pct')) || 0;
    body.notes = (fd.get('notes') || '').trim();
    for (const row of mount.querySelectorAll('#est-lines .est-line')) {
        const description = row.querySelector('.est-desc').value.trim();
        const quantity = Number(row.querySelector('.est-qty').value) || 0;
        const unit_price_usd = Number(row.querySelector('.est-price').value) || 0;
        if (description || quantity || unit_price_usd) body.line_items.push({ description, quantity, unit_price_usd });
    }
    try {
        const doc = await api.calcEstimate(body);
        if (!viewIsCurrent(tok)) return;
        renderResult(mount, doc);
    } catch (err) {
        showToast(err.message || t('view.estimate.toast.error'), { level: 'error' });
    }
}

function docToText(doc) {
    const lines = [
        `ESTIMATE ${doc.estimate_number}`,
        `From: ${doc.business_name}${doc.business_address ? ', ' + doc.business_address : ''}`,
        `Quote for: ${doc.client_name}${doc.client_address ? ', ' + doc.client_address : ''}`,
        `Date: ${doc.estimate_date}   Valid until: ${doc.valid_until} (${doc.valid_days} days)`,
        '',
    ];
    for (const l of doc.lines) {
        lines.push(`  ${l.description} — ${l.quantity} × ${money(l.unit_price_usd)} = ${money(l.amount_usd)}`);
    }
    lines.push('', `Subtotal: ${money(doc.subtotal_usd)}`);
    if (doc.discount_amount_usd > 0) lines.push(`Discount (${doc.discount_pct}%): -${money(doc.discount_amount_usd)}`);
    if (doc.tax_amount_usd > 0) lines.push(`Tax (${doc.tax_rate_pct}%): ${money(doc.tax_amount_usd)}`);
    lines.push(`TOTAL: ${money(doc.total_usd)}`);
    lines.push('', 'This is an estimate, not a final invoice. Prices are valid through the date above.');
    if (doc.notes) lines.push('', `Notes: ${doc.notes}`);
    return lines.join('\n');
}

function renderResult(mount, doc) {
    LAST_DOC = doc;
    const el = mount.querySelector('#est-result');
    const rows = doc.lines.map((l) => `
        <tr><td>${esc(l.description)}</td><td>${l.quantity}</td>
            <td>${money(l.unit_price_usd)}</td><td>${money(l.amount_usd)}</td></tr>
    `).join('');
    const discountRow = doc.discount_amount_usd > 0
        ? `<tr><td colspan="3" class="num">${t('view.estimate.row.discount')} (${doc.discount_pct}%)</td><td class="neg">-${money(doc.discount_amount_usd)}</td></tr>`
        : '';
    const taxRow = doc.tax_amount_usd > 0
        ? `<tr><td colspan="3" class="num">${t('view.estimate.row.tax')} (${doc.tax_rate_pct}%)</td><td>${money(doc.tax_amount_usd)}</td></tr>`
        : '';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.estimate.h2.result">Estimate</h2>
            <p>
                <button class="btn btn-secondary" id="est-copy" type="button" data-i18n="view.estimate.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="est-download" type="button" data-i18n="view.estimate.btn.download">Download .txt</button>
            </p>
            <p class="muted small">
                <strong>${esc(doc.estimate_number)}</strong> — ${esc(doc.business_name)} →
                ${esc(doc.client_name)} · <span data-i18n="view.estimate.valid">Valid until</span> ${esc(doc.valid_until)}
            </p>
            <table class="data-table">
                <thead><tr>
                    <th data-i18n="view.estimate.col.desc">Description</th>
                    <th data-i18n="view.estimate.col.qty">Qty</th>
                    <th data-i18n="view.estimate.col.price">Unit price</th>
                    <th data-i18n="view.estimate.col.amount">Amount</th>
                </tr></thead>
                <tbody>
                    ${rows}
                    <tr><td colspan="3" class="num" data-i18n="view.estimate.row.subtotal">Subtotal</td><td>${money(doc.subtotal_usd)}</td></tr>
                    ${discountRow}
                    ${taxRow}
                    <tr class="emph"><td colspan="3" class="num" data-i18n="view.estimate.row.total">Total</td><td class="pos">${money(doc.total_usd)}</td></tr>
                </tbody>
            </table>
            <p class="muted small" data-i18n="view.estimate.disclaimer">This is an estimate, not a final invoice. Prices are valid through the date above.</p>
            ${doc.notes ? `<p class="muted small">${esc(doc.notes)}</p>` : ''}
        </div>
    `;
    applyUiI18n(el);

    el.querySelector('#est-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.estimate.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.estimate.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#est-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = (LAST_DOC.estimate_number || 'estimate') + '.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
