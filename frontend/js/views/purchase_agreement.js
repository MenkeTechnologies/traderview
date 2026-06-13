// Real-estate purchase agreement generator — down payment + loan + earnest %,
// via /calc/purchase-agreement. Previews live as you type.

import { api } from '../api.js';
import { applyUiI18n, t } from '../i18n.js';
import { currentViewToken, viewIsCurrent } from '../app.js';
import { showToast } from '../toast.js';
import { debounce } from '../util.js';

const esc = (s) => String(s).replace(/[&<>]/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;' }[c]));
const money = (n) => (n == null ? '—' : '$' + Number(n).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 }));
const pct = (n) => (n == null ? '—' : Number(n).toLocaleString(undefined, { maximumFractionDigits: 2 }) + '%');
let LAST_DOC = null;

export async function renderPurchaseAgreement(mount, _appState) {
    const tok = currentViewToken();
    if (!viewIsCurrent(tok)) return;
    mount.innerHTML = `
        <h1 class="view-title"><span data-i18n="view.pa.h1.title">// PURCHASE AGREEMENT</span></h1>
        <p class="muted small" data-i18n="view.pa.hint.intro">
            A buyer's offer to purchase real property — distinct from the lease and bill-of-sale
            documents. It computes the down payment, the financed loan amount, and the earnest money as a
            percent of price, then assembles the agreement with contingency clauses. Drafting aid, not
            legal advice.
        </p>
        <div class="lpv-split">
        <div class="chart-panel">
            <h2 data-i18n="view.pa.h2.inputs">Offer details</h2>
            <form id="pa-form" class="inline-form">
                <label><span data-i18n="view.pa.label.state">State / jurisdiction</span>
                    <input type="text" name="state" value="Washington" required></label>
                <label><span data-i18n="view.pa.label.buyer">Buyer</span>
                    <input type="text" name="buyer_name" value=""></label>
                <label><span data-i18n="view.pa.label.seller">Seller</span>
                    <input type="text" name="seller_name" value=""></label>
                <label><span data-i18n="view.pa.label.property">Property address</span>
                    <input type="text" name="property_address" value=""></label>
                <label><span data-i18n="view.pa.label.price">Purchase price ($)</span>
                    <input type="number" step="1000" min="0" name="purchase_price_usd" value="400000" required></label>
                <label><span data-i18n="view.pa.label.earnest">Earnest money ($)</span>
                    <input type="number" step="100" min="0" name="earnest_money_usd" value="8000" required></label>
                <label><span data-i18n="view.pa.label.down">Down payment (%)</span>
                    <input type="number" step="0.1" min="0" max="100" name="down_payment_pct" value="20" required></label>
                <label><span data-i18n="view.pa.label.closing">Closing date</span>
                    <input type="date" name="closing_date" value="2026-09-30" required></label>
                <label><span data-i18n="view.pa.label.fin">Financing contingency</span>
                    <input type="checkbox" name="financing_contingency" checked></label>
                <label><span data-i18n="view.pa.label.insp">Inspection contingency</span>
                    <input type="checkbox" name="inspection_contingency" checked></label>
                <label><span data-i18n="view.pa.label.appr">Appraisal contingency</span>
                    <input type="checkbox" name="appraisal_contingency"></label>
                <label><span data-i18n="view.pa.label.statute">Statute citation (optional)</span>
                    <input type="text" name="statute_citation" value="" placeholder="${esc(t('view.pa.ph.statute'))}"></label>
            </form>
        </div>
        <div id="pa-result" class="chart-panel lpv-preview"></div>
        </div>
    `;
    applyUiI18n(mount);

    const form = mount.querySelector('#pa-form');
    const generate = async () => {
        const fd = new FormData(form);
        const body = {
            buyer_name: (fd.get('buyer_name') || '').trim(),
            seller_name: (fd.get('seller_name') || '').trim(),
            property_address: (fd.get('property_address') || '').trim(),
            purchase_price_usd: Number(fd.get('purchase_price_usd')) || 0,
            earnest_money_usd: Number(fd.get('earnest_money_usd')) || 0,
            down_payment_pct: Number(fd.get('down_payment_pct')) || 0,
            closing_date: fd.get('closing_date'),
            financing_contingency: fd.get('financing_contingency') != null,
            inspection_contingency: fd.get('inspection_contingency') != null,
            appraisal_contingency: fd.get('appraisal_contingency') != null,
            state: (fd.get('state') || '').trim(),
            statute_citation: (fd.get('statute_citation') || '').trim(),
        };
        try {
            const doc = await api.calcPurchaseAgreement(body);
            if (!viewIsCurrent(tok)) return;
            renderResult(mount, doc);
        } catch (err) {
            showToast(err.message || t('view.pa.toast.error'), { level: 'error' });
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
    const el = mount.querySelector('#pa-result');
    el.innerHTML = `
        <div class="lpv-bar">
            <div class="cards">
                <div class="card"><div class="label" data-i18n="view.pa.card.down">Down payment</div>
                    <div class="value">${money(doc.down_payment_usd)}</div></div>
                <div class="card pos"><div class="label" data-i18n="view.pa.card.loan">Loan amount</div>
                    <div class="value">${money(doc.loan_amount_usd)}</div></div>
                <div class="card"><div class="label" data-i18n="view.pa.card.earnest">Earnest money</div>
                    <div class="value">${money(doc.earnest_money_usd)} · ${pct(doc.earnest_money_pct)}</div></div>
            </div>
            <div class="btn-row-inline">
                <button class="btn btn-secondary" id="pa-copy" type="button" data-i18n="view.pa.btn.copy">Copy</button>
                <button class="btn btn-secondary" id="pa-download" type="button" data-i18n="view.pa.btn.download">Download .txt</button>
            </div>
        </div>
        <pre class="small">${esc(docToText(doc))}</pre>
    `;
    applyUiI18n(el);

    el.querySelector('#pa-copy').addEventListener('click', async () => {
        try {
            await navigator.clipboard.writeText(docToText(LAST_DOC));
            showToast(t('view.pa.toast.copied'), { level: 'success' });
        } catch (e) {
            showToast(t('view.pa.toast.copy_failed', { err: e.message || String(e) }), { level: 'error' });
        }
    });
    el.querySelector('#pa-download').addEventListener('click', () => {
        const blob = new Blob([docToText(LAST_DOC)], { type: 'text/plain' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = 'purchase-agreement.txt';
        a.click();
        URL.revokeObjectURL(url);
    });
}
