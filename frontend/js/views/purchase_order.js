// Purchase order generator — buyer-side order with line items, discount, tax,
// shipping, and expected delivery date, via /calc/purchase-order. Shares the
// line-item math with the invoice/estimate. Printable/copyable.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const TEXT = [
    ['buyer_name', 'Buyer', 'Acme LLC'],
    ['buyer_address', 'Buyer address', '1 Industrial Way'],
    ['vendor_name', 'Vendor', 'Parts Co'],
    ['vendor_address', 'Vendor address', '9 Supply Rd'],
    ['po_number', 'PO #', 'PO-001'],
    ['ship_to', 'Ship to', '1 Industrial Way, Dock B'],
];
const DEFAULT_LINES = [
    ['Widget', 10, 150],
    ['Bracket', 2, 75],
];

const money = (n) => '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 });
const esc = (s) => String(s).replace(/[&<>"]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' }[c]));

let LAST_DOC = null;

function lineRow(desc = '', qty = 1, price = 0) {
    return `<tr class="po-line">
        <td><input type="text" class="po-desc" value="${esc(desc)}" placeholder="${esc(t('view.po.ph.desc'))}"></td>
        <td><input type="number" step="0.01" min="0" class="po-qty" value="${qty}"></td>
        <td><input type="number" step="0.01" min="0" class="po-price" value="${price}"></td>
        <td><button type="button" class="btn btn-secondary po-del" data-i18n="view.po.btn.del">✕</button></td>
    </tr>`;
}

export async function renderPurchaseOrder(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.po.h1.title">// PURCHASE ORDER GENERATOR</span></h1>
        <p class="muted small" data-i18n="view.po.hint.intro">
            The buyer-side document that authorizes a purchase from a vendor. It uses the same line-item
            math as the invoice — amounts, subtotal, discount, tax — adds optional shipping to the grand
            total, and computes the expected delivery date from your lead time. Copy or download the PO.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.po.h2.inputs">Order details</h2>
            <form id="po-form" class="inline-form">
                ${TEXT.map(([key, label, def]) => `
                    <label><span data-i18n="view.po.label.${key}">${label}</span>
                        <input type="text" name="${key}" value="${esc(def)}"></label>
                `).join('')}
                <label><span data-i18n="view.po.label.order_date">Order date</span>
                    <input type="date" name="order_date" value="2026-06-01" required></label>
                <label><span data-i18n="view.po.label.delivery_days">Lead time (days)</span>
                    <input type="number" step="1" min="0" name="delivery_days" value="7" required></label>
                <label><span data-i18n="view.po.label.discount_pct">Discount (%)</span>
                    <input type="number" step="0.01" min="0" name="discount_pct" value="0" required></label>
                <label><span data-i18n="view.po.label.tax_rate_pct">Tax rate (%)</span>
                    <input type="number" step="0.01" min="0" name="tax_rate_pct" value="0" required></label>
                <label><span data-i18n="view.po.label.shipping">Shipping ($)</span>
                    <input type="number" step="0.01" min="0" name="shipping_usd" value="50" required></label>
                <label><span data-i18n="view.po.label.notes">Notes</span>
                    <input type="text" name="notes" value=""></label>
            </form>
            <h3 data-i18n="view.po.h3.lines">Line items</h3>
            <table class="data-table" id="po-lines">
                <thead><tr>
                    <th data-i18n="view.po.col.desc">Description</th>
                    <th data-i18n="view.po.col.qty">Qty</th>
                    <th data-i18n="view.po.col.price">Unit price</th>
                    <th></th>
                </tr></thead>
                <tbody>${DEFAULT_LINES.map((l) => lineRow(...l)).join('')}</tbody>
            </table>
            <p>
                <button type="button" class="btn btn-secondary" id="po-add" data-i18n="view.po.btn.add">+ Add line</button>
            </p>
        </div>
        <div id="po-result" class="lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const linesBody = mount.querySelector('#po-lines tbody');
    const live = debounce(() => submit(mount, tok), 250);
    mount.querySelector('#po-add').addEventListener('click', () => {
        linesBody.insertAdjacentHTML('beforeend', lineRow());
        applyUiI18n(linesBody.lastElementChild);
        submit(mount, tok);
    });
    linesBody.addEventListener('click', (e) => {
        if (e.target.classList.contains('po-del')) {
            const rows = linesBody.querySelectorAll('.po-line');
            if (rows.length > 1) { e.target.closest('tr').remove(); submit(mount, tok); }
        }
    });
    mount.querySelector('#po-form').addEventListener('input', live);
    linesBody.addEventListener('input', live);
    submit(mount, tok);
}

async function submit(mount, tok) {
    const form = mount.querySelector('#po-form');
    const fd = new FormData(form);
    const body = { line_items: [] };
    for (const [key] of TEXT) body[key] = (fd.get(key) || '').trim();
    body.order_date = fd.get('order_date');
    body.delivery_days = Math.round(Number(fd.get('delivery_days')) || 0);
    body.discount_pct = Number(fd.get('discount_pct')) || 0;
    body.tax_rate_pct = Number(fd.get('tax_rate_pct')) || 0;
    body.shipping_usd = Number(fd.get('shipping_usd')) || 0;
    body.notes = (fd.get('notes') || '').trim();
    for (const row of mount.querySelectorAll('#po-lines .po-line')) {
        const description = row.querySelector('.po-desc').value.trim();
        const quantity = Number(row.querySelector('.po-qty').value) || 0;
        const unit_price_usd = Number(row.querySelector('.po-price').value) || 0;
        if (description || quantity || unit_price_usd) body.line_items.push({ description, quantity, unit_price_usd });
    }
    try {
        const doc = await api.calcPurchaseOrder(body);
        if (!viewIsCurrent(tok)) return;
        renderResult(mount, doc);
    } catch (err) {
        showToast(err.message || t('view.po.toast.error'), { level: 'error' });
    }
}

function docToText(doc) {
    const lines = [
        `PURCHASE ORDER ${doc.po_number}`,
        `Buyer: ${doc.buyer_name}${doc.buyer_address ? ', ' + doc.buyer_address : ''}`,
        `Vendor: ${doc.vendor_name}${doc.vendor_address ? ', ' + doc.vendor_address : ''}`,
        `Ordered: ${doc.order_date}   Expected: ${doc.expected_delivery_date} (${doc.delivery_days} days)`,
    ];
    if (doc.ship_to) lines.push(`Ship to: ${doc.ship_to}`);
    lines.push('');
    for (const l of doc.lines) {
        lines.push(`  ${l.description} — ${l.quantity} × ${money(l.unit_price_usd)} = ${money(l.amount_usd)}`);
    }
    lines.push('', `Subtotal: ${money(doc.subtotal_usd)}`);
    if (doc.discount_amount_usd > 0) lines.push(`Discount (${doc.discount_pct}%): -${money(doc.discount_amount_usd)}`);
    if (doc.tax_amount_usd > 0) lines.push(`Tax (${doc.tax_rate_pct}%): ${money(doc.tax_amount_usd)}`);
    if (doc.shipping_usd > 0) lines.push(`Shipping: ${money(doc.shipping_usd)}`);
    lines.push(`TOTAL: ${money(doc.total_usd)}`);
    if (doc.notes) lines.push('', `Notes: ${doc.notes}`);
    return lines.join('\n');
}

function renderResult(mount, doc) {
    LAST_DOC = doc;
    const el = mount.querySelector('#po-result');
    const rows = doc.lines.map((l) => `
        <tr><td>${esc(l.description)}</td><td>${l.quantity}</td>
            <td>${money(l.unit_price_usd)}</td><td>${money(l.amount_usd)}</td></tr>
    `).join('');
    const discountRow = doc.discount_amount_usd > 0
        ? `<tr><td colspan="3" class="num">${t('view.po.row.discount')} (${doc.discount_pct}%)</td><td class="neg">-${money(doc.discount_amount_usd)}</td></tr>`
        : '';
    const taxRow = doc.tax_amount_usd > 0
        ? `<tr><td colspan="3" class="num">${t('view.po.row.tax')} (${doc.tax_rate_pct}%)</td><td>${money(doc.tax_amount_usd)}</td></tr>`
        : '';
    const shipRow = doc.shipping_usd > 0
        ? `<tr><td colspan="3" class="num" data-i18n="view.po.row.shipping">Shipping</td><td>${money(doc.shipping_usd)}</td></tr>`
        : '';
    el.innerHTML = `
        <div class="chart-panel">
            <h2 data-i18n="view.po.h2.result">Purchase order</h2>
            <p>
                <button class="btn btn-secondary" id="po-copy" type="button" data-i18n="view.po.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="po-download" type="button" data-i18n="view.po.btn.download">Download .txt</button>
            </p>
            <p class="muted small">
                <strong>${esc(doc.po_number)}</strong> — ${esc(doc.buyer_name)} →
                ${esc(doc.vendor_name)} · <span data-i18n="view.po.expected">Expected</span> ${esc(doc.expected_delivery_date)}
            </p>
            <table class="data-table">
                <thead><tr>
                    <th data-i18n="view.po.col.desc">Description</th>
                    <th data-i18n="view.po.col.qty">Qty</th>
                    <th data-i18n="view.po.col.price">Unit price</th>
                    <th data-i18n="view.po.col.amount">Amount</th>
                </tr></thead>
                <tbody>
                    ${rows}
                    <tr><td colspan="3" class="num" data-i18n="view.po.row.subtotal">Subtotal</td><td>${money(doc.subtotal_usd)}</td></tr>
                    ${discountRow}
                    ${taxRow}
                    ${shipRow}
                    <tr class="emph"><td colspan="3" class="num" data-i18n="view.po.row.total">Total</td><td class="pos">${money(doc.total_usd)}</td></tr>
                </tbody>
            </table>
            ${doc.notes ? `<p class="muted small">${esc(doc.notes)}</p>` : ''}
        </div>
    `;
    applyUiI18n(el);

    el.querySelector('#po-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.po.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.po.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#po-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = (LAST_DOC.po_number || 'purchase-order') + '.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
