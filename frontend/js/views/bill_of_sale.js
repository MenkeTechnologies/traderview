// Bill of sale generator — sales tax + total consideration and the transfer /
// condition / title clauses, via /calc/bill-of-sale. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
let LAST_DOC = null;

export async function renderBillOfSale(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.bos.h1.title">// BILL OF SALE</span></h1>
        <p class="muted small" data-i18n="view.bos.hint.intro">
            Transfers ownership of personal property — a vehicle, equipment, a boat, business assets —
            from seller to buyer. It computes any sales tax and the total consideration, then assembles
            the document with the transfer, condition (as-is or warranted), and title clauses. Drafting
            aid, not legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.bos.h2.inputs">Sale details</h2>
            <form id="bos-form" class="inline-form">
                <label><span data-i18n="view.bos.label.state">Governing state</span>
                    <input type="text" name="governing_state" value="Florida" required></label>
                <label><span data-i18n="view.bos.label.seller">Seller name</span>
                    <input type="text" name="seller_name" value=""></label>
                <label><span data-i18n="view.bos.label.seller_address">Seller address</span>
                    <input type="text" name="seller_address" value=""></label>
                <label><span data-i18n="view.bos.label.buyer">Buyer name</span>
                    <input type="text" name="buyer_name" value=""></label>
                <label><span data-i18n="view.bos.label.buyer_address">Buyer address</span>
                    <input type="text" name="buyer_address" value=""></label>
                <label><span data-i18n="view.bos.label.item">Property description</span>
                    <input type="text" name="item_description" value="2019 Honda Civic, VIN 1HG..."></label>
                <label><span data-i18n="view.bos.label.price">Sale price ($)</span>
                    <input type="number" step="0.01" min="0" name="sale_price_usd" value="12000" required></label>
                <label><span data-i18n="view.bos.label.tax">Sales tax (%)</span>
                    <input type="number" step="0.001" min="0" name="sales_tax_pct" value="6"></label>
                <label><span data-i18n="view.bos.label.date">Date of sale</span>
                    <input type="date" name="sale_date" value="2026-06-13" required></label>
                <label><span data-i18n="view.bos.label.as_is">Sold as-is (no warranty)</span>
                    <input type="checkbox" name="as_is" checked></label>
                <label><span data-i18n="view.bos.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.bos.ph.statute'))}"></label>
            </form>
        </div>
        <div id="bos-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#bos-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            seller_name: (fd.get('seller_name') || '').trim(),
            seller_address: (fd.get('seller_address') || '').trim(),
            buyer_name: (fd.get('buyer_name') || '').trim(),
            buyer_address: (fd.get('buyer_address') || '').trim(),
            item_description: (fd.get('item_description') || '').trim(),
            sale_price_usd: Number(fd.get('sale_price_usd')) || 0,
            sales_tax_pct: Number(fd.get('sales_tax_pct')) || 0,
            as_is: fd.get('as_is') != null,
            sale_date: fd.get('sale_date'),
            governing_state: (fd.get('governing_state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcBillOfSale(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.bos.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#bos-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card pos"><div class="label" data-i18n="view.bos.card.total">Total consideration</div>
                    <div class="value">${money(doc.total_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.bos.card.price">Sale price</div>
                    <div class="value">${money(doc.sale_price_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.bos.card.tax">Sales tax</div>
                    <div class="value">${money(doc.sales_tax_usd)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="bos-copy" type="button" data-i18n="view.bos.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="bos-download" type="button" data-i18n="view.bos.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#bos-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.bos.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.bos.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#bos-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'bill-of-sale.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
